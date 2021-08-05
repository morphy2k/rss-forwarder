use crate::{item::ExtItem, Result};

use super::Sink;

use async_trait::async_trait;
use chrono::Utc;
use reqwest::{Client, IntoUrl, Url};
use serde::Serialize;

#[derive(Debug)]
pub struct Discord {
    url: Url,
    client: Client,
}

impl Discord {
    pub fn new<T: IntoUrl>(url: T, client: Client) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            client,
        })
    }
}

#[async_trait]
impl Sink for Discord {
    async fn push(&self, items: &[rss::Item]) -> Result<()> {
        let length = items.len();
        let limit = 10_usize;
        let chunk_count = (length as f64 / limit as f64).ceil() as usize;

        let mut chunks: Vec<Body> = Vec::with_capacity(chunk_count);
        for i in 0..chunk_count {
            let pos = i * limit;
            let mut chunk = Vec::new();
            for v in &items[pos..(pos + limit).min(length)] {
                chunk.push(EmbedObject::from(v));
            }
            chunks.push(Body { embeds: chunk })
        }

        for v in chunks.iter() {
            self.client.post(self.url.as_ref()).json(v).send().await?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Body {
    embeds: Vec<EmbedObject>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedObject {
    title: String,
    description: String,
    url: String,
    timestamp: String,
    author: EmbedAuthor,
    footer: EmbedFooter,
}

impl From<&rss::Item> for EmbedObject {
    fn from(item: &rss::Item) -> Self {
        Self {
            title: item.title().unwrap().to_owned(),
            description: item.description().unwrap().to_owned(),
            url: item.link().unwrap().to_owned(),
            timestamp: item
                .pub_date_as_datetime()
                .unwrap_or_else(|| Utc::now().into())
                .to_rfc3339(),
            author: EmbedAuthor::from(item),
            footer: EmbedFooter::from(item),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedAuthor {
    name: Option<String>,
    url: Option<String>,
}

impl From<&rss::Item> for EmbedAuthor {
    fn from(item: &rss::Item) -> Self {
        Self {
            name: item.author().map(|v| v.to_string()),
            url: None,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedImage {
    url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedFooter {
    text: String,
}

impl From<&rss::Item> for EmbedFooter {
    fn from(item: &rss::Item) -> Self {
        Self {
            text: match item.source() {
                Some(v) => v.title().unwrap_or(""),
                None => "",
            }
            .to_string(),
        }
    }
}
