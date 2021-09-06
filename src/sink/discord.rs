use crate::{
    feed::{ExtItem, Item},
    Result,
};

use super::Sink;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset, Utc};
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
    async fn push(&self, items: &[Item]) -> Result<()> {
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
    timestamp: DateTime<FixedOffset>,
    author: EmbedAuthor,
    footer: EmbedFooter,
}

impl From<&Item> for EmbedObject {
    fn from(item: &Item) -> Self {
        Self {
            title: item.title_as_text(),
            description: item.description_as_text().unwrap_or_default(),
            url: item
                .links
                .first()
                .map(|s| s.to_string())
                .unwrap_or_default(),
            timestamp: item.date,
            author: EmbedAuthor::from(item),
            footer: EmbedFooter::from(item),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedAuthor {
    name: String,
    url: Option<String>,
}

impl From<&Item> for EmbedAuthor {
    fn from(item: &Item) -> Self {
        let author = item.authors.first();

        Self {
            name: author.map(|v| v.name.to_owned()).unwrap_or_default(),
            url: match author {
                Some(v) => v.uri.to_owned(),
                None => None,
            },
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

impl From<&Item> for EmbedFooter {
    fn from(_: &Item) -> Self {
        Self {
            text: "".to_string(),
        }
    }
}
