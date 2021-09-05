use crate::{error::Error, item::ExtItem, Result};

use super::Sink;

use std::{convert::TryFrom, process::Stdio};

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
use serde::Serialize;
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    sync::mpsc::{self, Sender},
    task::JoinHandle,
};

#[derive(Debug)]
pub struct Custom {
    process: Child,
    stdin_task: JoinHandle<Result<()>>,
    kill_signal: Sender<()>,
    tx: Sender<Item>,
}

impl Custom {
    pub fn new(cmd: String, args: Vec<String>) -> Result<Self> {
        let mut cmd = Command::new(&cmd)
            .args(&args)
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let mut stdin = cmd.stdin.take().unwrap();
        let (item_tx, mut item_rx) = mpsc::channel(10);
        let (kill_tx, mut kill_rx) = mpsc::channel(1);

        let task = tokio::spawn(async move {
            loop {
                let item = tokio::select! {
                    biased;
                    _ = kill_rx.recv() => break,
                    v = item_rx.recv() => v.unwrap(),
                };

                let json = serde_json::to_string(&item).unwrap();
                stdin.write_all(format!("{}\n", json).as_bytes()).await?;
                stdin.flush().await?;
            }

            stdin.shutdown().await?;

            Ok(())
        });

        Ok(Self {
            process: cmd,
            stdin_task: task,
            kill_signal: kill_tx,
            tx: item_tx,
        })
    }
}

#[async_trait]
impl Sink for Custom {
    async fn push(&self, items: &[rss::Item]) -> Result<()> {
        for item in items {
            let item = Item::try_from(item)?;
            self.tx.send(item).await.unwrap();
        }

        Ok(())
    }

    async fn shutdown(mut self) -> Result<()> {
        if !self.kill_signal.is_closed() {
            self.kill_signal.send(()).await.unwrap();
        }
        self.stdin_task.await??;
        self.process.wait().await?;

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct Item {
    title: String,
    description: String,
    url: String,
    date: DateTime<FixedOffset>,
}

impl TryFrom<&rss::Item> for Item {
    type Error = Error;

    fn try_from(value: &rss::Item) -> std::result::Result<Self, Self::Error> {
        let item = Item {
            title: value
                .title_as_text()
                .ok_or_else(|| Error::InvalidItem("title is missing".to_string()))?,
            description: value
                .description_as_text()
                .ok_or_else(|| Error::InvalidItem("description is missing".to_string()))?,
            url: value
                .link()
                .ok_or_else(|| Error::InvalidItem("link is missing".to_string()))?
                .to_owned(),
            date: value
                .pub_date_as_datetime()
                .unwrap_or_else(|| Utc::now().into()),
        };

        Ok(item)
    }
}
