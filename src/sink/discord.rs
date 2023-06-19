use crate::{
    error::{Error, FeedError},
    feed::item::{FeedItem, TryFromItem},
    Result,
};

use super::Sink;

use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use reqwest::{Client, IntoUrl, Url};
use serde::Serialize;
use tracing::debug;

const PROVIDER: EmbedProvider<'static> = EmbedProvider {
    name: env!("CARGO_PKG_NAME"),
    url: Some(env!("CARGO_PKG_REPOSITORY")),
};

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
    #[tracing::instrument(
        name = "push",
        skip(self, items),
        fields(
            url = %self.url,
        )
        level = "debug"
    )]
    async fn push<'a, T>(&self, items: &'a [T]) -> Result<()>
    where
        T: FeedItem<'a>,
    {
        let length = items.len();
        let limit = 10_usize;
        let chunk_count = (length as f64 / limit as f64).ceil() as usize;

        debug!(count = length, chunks = chunk_count, "pushing items");

        let mut chunks: Vec<Body> = Vec::with_capacity(chunk_count);
        for i in 0..chunk_count {
            let pos = i * limit;
            let chunk = items[pos..(pos + limit).min(length)]
                .iter()
                .map(EmbedObject::try_from_item)
                .collect::<Result<Vec<EmbedObject>>>()?;

            chunks.push(Body { embeds: chunk });
        }

        for v in chunks.iter() {
            self.client
                .post(self.url.as_ref())
                .json(v)
                .send()
                .await?
                .error_for_status()?;
        }

        Ok(())
    }

    #[tracing::instrument(
        name = "shutdown",
        skip(self),
        fields(
            url = %self.url,
        )
        level = "debug"
    )]
    async fn shutdown(self) -> Result<()> {
        debug!("shutting down");
        Ok(())
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Body<'a> {
    embeds: Vec<EmbedObject<'a>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedObject<'a> {
    title: String,
    description: String,
    url: &'a str,
    timestamp: DateTime<FixedOffset>,
    author: EmbedAuthor<'a>,
    footer: EmbedFooter<'a>,
    provider: EmbedProvider<'a>,
}

impl<'a, T> TryFromItem<'a, T> for EmbedObject<'a>
where
    T: FeedItem<'a>,
{
    type Error = Error;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error> {
        let embed = Self {
            title: value
                .title_as_text()
                .ok_or_else(|| FeedError::Item("title is missing".to_string()))?,
            description: value.description_as_text().unwrap_or_default(),
            url: value.link().unwrap_or_default(),
            timestamp: value.date(),
            author: EmbedAuthor::try_from_item(value)?,
            footer: EmbedFooter::try_from_item(value)?,
            provider: PROVIDER,
        };

        Ok(embed)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct EmbedAuthor<'a> {
    name: &'a str,
    url: Option<&'a str>,
}

impl<'a, T> TryFromItem<'a, T> for EmbedAuthor<'a>
where
    T: FeedItem<'a>,
{
    type Error = Error;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error> {
        Ok(match value.authors().first() {
            Some(v) => Self {
                name: v.name,
                url: v.uri,
            },
            None => Self::default(),
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedImage<'a> {
    url: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedFooter<'a> {
    text: &'a str,
}

impl<'a, T> TryFromItem<'a, T> for EmbedFooter<'a>
where
    T: FeedItem<'a>,
{
    type Error = Error;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error> {
        Ok(Self {
            text: match value.source() {
                Some(v) => v.title,
                None => "",
            },
        })
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
struct EmbedProvider<'a> {
    name: &'a str,
    url: Option<&'a str>,
}
