use crate::{
    error::FeedError,
    feed::item::{FeedItem, TryFromItem},
    Result,
};

use super::Sink;

use async_trait::async_trait;
use reqwest::{Client, Url};
use serde::Serialize;
use slack_bk::{
    blocks::{Block, Context, ContextElement, Divider, Header, Section},
    composition::{MarkdownText, PlainText, Text},
    elements::{Button, Element},
};
use tokio::time::{self, Duration};
use tracing::debug;

#[derive(Debug)]
pub struct Slack {
    url: Url,
    client: Client,
}

impl Slack {
    pub fn new(url: Url, client: Client) -> Result<Self> {
        Ok(Self { url, client })
    }
}

#[async_trait]
impl Sink for Slack {
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
                .map(ItemBlockCollection::try_from_item)
                .collect::<std::result::Result<Vec<ItemBlockCollection>, FeedError>>()?
                .into_iter()
                .flatten()
                .collect();

            chunks.push(Body { blocks: chunk });
        }

        for (i, v) in chunks.iter().enumerate() {
            self.client
                .post(self.url.as_ref())
                .json(v)
                .send()
                .await?
                .error_for_status()?;

            if i != chunks.len() - 1 {
                time::sleep(Duration::from_millis(1000)).await;
            }
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
struct Body {
    blocks: Vec<Block>,
}

type ItemBlockCollection = [Block; 4];

impl<'a, T> TryFromItem<'a, T> for ItemBlockCollection
where
    T: FeedItem<'a>,
{
    type Error = FeedError;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error> {
        let header = Header {
            text: Text::PlainText(PlainText {
                text: value
                    .title_as_text()?
                    .ok_or_else(|| FeedError::Item("title is missing".to_string()))?,
                emoji: false,
            }),
            block_id: None,
        };

        let section = Section {
            text: Text::PlainText(PlainText {
                text: value
                    .description_as_text()?
                    .unwrap_or_else(|| "...".to_string()),
                emoji: false,
            })
            .into(),
            accessory: Element::Button(Button {
                text: Text::PlainText(PlainText {
                    text: ":link: Open".to_string(),
                    emoji: true,
                }),
                action_id: "button-action".to_string(),
                url: value.link().map(|s| s.to_string()),
                ..Default::default()
            })
            .into(),
            ..Default::default()
        };

        let mut ctx_elements = Vec::with_capacity(3);

        if let Some(a) = value.authors().first() {
            let author = if let Some(url) = a.uri {
                format!("<{}|{}>", url, a.name)
            } else {
                a.name.to_string()
            };

            ctx_elements.push(ContextElement::Text(Text::Markdown(MarkdownText {
                text: author,
                verbatim: false,
            })));
        }

        if let Some(s) = value.source() {
            let src = if let Some(url) = s.url {
                format!("<{}|{}>", url, s.title)
            } else {
                s.title.to_string()
            };

            ctx_elements.push(ContextElement::Text(Text::Markdown(MarkdownText {
                text: src,
                verbatim: false,
            })));
        }

        ctx_elements.push(ContextElement::Text(Text::PlainText(PlainText {
            text: value.date().format("%d %b %Y %I:%M %p %Z").to_string(),
            emoji: false,
        })));

        let context = Context {
            elements: ctx_elements,
            ..Default::default()
        };

        Ok([
            Block::Header(header),
            Block::Section(section),
            Block::Context(context),
            Block::Divider(Divider::default()),
        ])
    }
}
