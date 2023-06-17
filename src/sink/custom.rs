use crate::{
    error::{Error, FeedError},
    feed::item::{Author, FeedItem, TryFromItem},
    Result,
};

use super::Sink;

use std::process::Stdio;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};
use tracing::debug;

#[derive(Debug)]
pub struct Custom {
    command: String,
    arguments: Vec<String>,
    process: Child,
    stdin_task: JoinHandle<Result<()>>,
    kill_tx: Sender<()>,
    data_tx: Sender<Vec<u8>>,
}

impl Custom {
    pub fn new(cmd: String, args: Vec<String>) -> Result<Self> {
        let mut process = Command::new(&cmd)
            .args(&args)
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let mut stdin = process
            .stdin
            .take()
            .ok_or_else(|| Error::Sink("stdin not captured".to_string()))?;

        let (data_tx, mut data_rx) = mpsc::channel(10);
        let (kill_tx, mut kill_rx) = mpsc::channel(1);

        let task = tokio::spawn(async move {
            loop {
                let data: Vec<u8> = tokio::select! {
                    biased;
                    _ = kill_rx.recv() => break,
                    v = data_rx.recv() => v.unwrap(),
                };

                stdin.write_all(&data).await?;
                stdin.flush().await?;
            }

            stdin.shutdown().await?;

            Ok(())
        });

        Ok(Self {
            command: cmd,
            arguments: args,
            process,
            stdin_task: task,
            kill_tx,
            data_tx,
        })
    }
}

#[async_trait]
impl Sink for Custom {
    #[tracing::instrument(
        name = "push",
        skip(self, items),
        fields(
            pid = self.process.id(),
            command = %self.command,
            arguments = %self.arguments.join(" "),
        ),
        level = "debug"
    )]
    async fn push<'a, T>(&self, items: &'a [T]) -> Result<()>
    where
        T: FeedItem<'a>,
    {
        debug!(count = items.len(), "pushing items");

        for item in items {
            let obj = Object::try_from_item(item)?;
            let mut json = serde_json::to_vec(&obj)?;
            json.extend_from_slice(b"\n");

            if self.data_tx.send(json).await.is_err() {
                return Err(Error::Sink("broken stdin task".to_string()));
            }
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "shutdown",
        skip(self),
        fields(
            pid = self.process.id(),
            commad = %self.command,
            arguments = %self.arguments.join(" "),
        ),
        level = "debug"
    )]
    async fn shutdown(mut self) -> Result<()> {
        debug!("shutting down");

        if !self.kill_tx.is_closed() {
            self.kill_tx.send(()).await.unwrap();
        }
        self.stdin_task.await??;
        self.process.wait().await?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct Object<'a> {
    title: &'a str,
    description: Option<&'a str>,
    content: Option<&'a str>,
    link: &'a str,
    date: DateTime<FixedOffset>,
    authors: Vec<Author<'a>>,
}

impl<'a, T> TryFromItem<'a, T> for Object<'a>
where
    T: FeedItem<'a>,
{
    type Error = Error;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error> {
        let obj = Self {
            title: value
                .title()
                .ok_or_else(|| FeedError::Item("title is missing".to_string()))?,
            description: value.description(),
            content: value.content(),
            link: value
                .link()
                .ok_or_else(|| FeedError::Item("missing link".to_string()))?,
            date: value.date(),
            authors: value.authors(),
        };

        Ok(obj)
    }
}
