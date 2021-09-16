pub mod custom;
pub mod discord;
pub mod slack;

use crate::{feed::item::FeedItem, Result};

use self::{custom::Custom, discord::Discord, slack::Slack};

use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SinkOptions {
    Discord {
        url: String,
    },
    Slack {
        url: String,
    },
    Custom {
        command: String,
        #[serde(default)]
        arguments: Vec<String>,
    },
}

impl SinkOptions {
    pub fn sink(self, client: &Client) -> Result<AnySink> {
        let sink = match self {
            SinkOptions::Discord { url } => AnySink::Discord(Discord::new(url, client.clone())?),
            SinkOptions::Slack { url } => AnySink::Slack(Slack::new(url, client.clone())?),
            SinkOptions::Custom { command, arguments } => {
                AnySink::Custom(Custom::new(command, arguments)?)
            }
        };

        Ok(sink)
    }
}

#[async_trait]
pub trait Sink {
    async fn push<'a, T>(&self, items: &'a [T]) -> Result<()>
    where
        T: FeedItem<'a>;

    async fn shutdown(mut self) -> Result<()>;
}

#[derive(Debug)]
pub enum AnySink {
    Discord(discord::Discord),
    Slack(slack::Slack),
    Custom(custom::Custom),
}

#[async_trait]
impl Sink for AnySink {
    #[inline]
    async fn push<'a, T>(&self, items: &'a [T]) -> Result<()>
    where
        T: FeedItem<'a>,
    {
        match self {
            AnySink::Discord(s) => s.push(items).await,
            AnySink::Slack(s) => s.push(items).await,
            AnySink::Custom(s) => s.push(items).await,
        }
    }

    #[inline]
    async fn shutdown(self) -> Result<()> {
        match self {
            AnySink::Discord(s) => s.shutdown().await,
            AnySink::Slack(s) => s.shutdown().await,
            AnySink::Custom(s) => s.shutdown().await,
        }
    }
}
