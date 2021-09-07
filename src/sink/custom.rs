use crate::{error::Error, feed::Item, Result};

use super::Sink;

use std::process::Stdio;

use async_trait::async_trait;

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
    kill_tx: Sender<()>,
    data_tx: Sender<Vec<u8>>,
}

impl Custom {
    pub fn new<C, A>(cmd: C, args: A) -> Result<Self>
    where
        C: AsRef<str>,
        A: IntoIterator<Item = String>,
    {
        let mut cmd = Command::new(cmd.as_ref())
            .args(args)
            .stdin(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let mut stdin = cmd
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
            process: cmd,
            stdin_task: task,
            kill_tx,
            data_tx,
        })
    }
}

#[async_trait]
impl Sink for Custom {
    async fn push(&self, items: &[Item]) -> Result<()> {
        for item in items {
            let mut obj = serde_json::to_vec(&item)?;
            obj.extend_from_slice(b"\n");

            if self.data_tx.send(obj).await.is_err() {
                return Err(Error::Sink("broken stdin task".to_string()));
            }
        }

        Ok(())
    }

    async fn shutdown(mut self) -> Result<()> {
        if !self.kill_tx.is_closed() {
            self.kill_tx.send(()).await.unwrap();
        }
        self.stdin_task.await??;
        self.process.wait().await?;

        Ok(())
    }
}
