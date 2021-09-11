use std::convert::TryFrom;

use crate::{
    error::{Error, FeedError},
    feed::Item,
    Result,
};

use super::Sink;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
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
    async fn push<'a>(&self, items: &[&'a dyn Item]) -> Result<()> {
        let length = items.len();
        let limit = 10_usize;
        let chunk_count = (length as f64 / limit as f64).ceil() as usize;

        let mut chunks: Vec<Body> = Vec::with_capacity(chunk_count);
        for i in 0..chunk_count {
            let pos = i * limit;
            let mut chunk = Vec::new();
            for &v in &items[pos..(pos + limit).min(length)] {
                chunk.push(EmbedObject::try_from(v)?);
            }
            chunks.push(Body { embeds: chunk })
        }

        for v in chunks.iter() {
            self.client.post(self.url.as_ref()).json(v).send().await?;
        }

        Ok(())
    }

    async fn shutdown(self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Body<'a> {
    embeds: Vec<EmbedObject<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedObject<'a> {
    title: String,
    description: String,
    url: &'a str,
    timestamp: DateTime<FixedOffset>,
    author: EmbedAuthor<'a>,
    footer: EmbedFooter<'a>,
}

impl<'a> TryFrom<&'a dyn Item> for EmbedObject<'a> {
    type Error = Error;

    fn try_from(value: &'a dyn Item) -> std::result::Result<Self, Self::Error> {
        let embed = Self {
            title: value
                .title_as_text()
                .ok_or_else(|| FeedError::Item("title is missing".to_string()))?,
            description: value.description_as_text().unwrap_or_default(),
            url: value.link().unwrap_or_default(),
            timestamp: value.date(),
            author: EmbedAuthor::from(value),
            footer: EmbedFooter::from(value),
        };

        Ok(embed)
    }
}

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedAuthor<'a> {
    name: &'a str,
    url: Option<&'a str>,
}

impl<'a> From<&'a dyn Item> for EmbedAuthor<'a> {
    fn from(item: &'a dyn Item) -> Self {
        match item.authors().first() {
            Some(v) => Self {
                name: v.name,
                url: v.uri,
            },
            None => Self::default(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedImage<'a> {
    url: &'a str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedFooter<'a> {
    text: &'a str,
}

impl<'a> From<&'a dyn Item> for EmbedFooter<'a> {
    fn from(_: &'a dyn Item) -> Self {
        Self { text: "" }
    }
}
