use crate::Result;

use async_trait::async_trait;
use serde::Deserialize;

pub mod discord;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SinkType {
    Discord { url: String },
}

#[async_trait]
pub trait Sink {
    async fn push(&self, items: &[rss::Item]) -> Result<()>;
}
