use crate::{
    error::{Error, FeedError},
    feed::item::{FeedItem, TryFromItem},
    Result,
};

use super::Sink;

use async_trait::async_trait;
use reqwest::{Client, IntoUrl, Url};
use serde::Serialize;
use slack_bk::{
    blocks::{Block, Context, ContextElement, Divider, Header, Section},
    composition::{MarkdownText, PlainText, Text},
    elements::{Button, Element},
};
use tokio::time::{self, Duration};

#[derive(Debug)]
pub struct Slack {
    url: Url,
    client: Client,
}

impl Slack {
    pub fn new<T: IntoUrl>(url: T, client: Client) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            client,
        })
    }
}

#[async_trait]
impl Sink for Slack {
    async fn push<'a, T>(&self, items: &'a [T]) -> Result<()>
    where
        T: FeedItem<'a>,
    {
        let length = items.len();
        let limit = 10_usize;
        let chunk_count = (length as f64 / limit as f64).ceil() as usize;

        let mut chunks: Vec<Body> = Vec::with_capacity(chunk_count);
        for i in 0..chunk_count {
            let pos = i * limit;
            let chunk = items[pos..(pos + limit).min(length)]
                .iter()
                .map(ItemBlockCollection::try_from_item)
                .collect::<Result<Vec<ItemBlockCollection>>>()?
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

    async fn shutdown(self) -> Result<()> {
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
    type Error = Error;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error> {
        let header = Header {
            text: Text::PlainText(PlainText {
                text: value
                    .title_as_text()
                    .ok_or_else(|| FeedError::Item("title is missing".to_string()))?,
                emoji: false,
            }),
            block_id: None,
        };

        let section = Section {
            text: Text::PlainText(PlainText {
                text: value
                    .description_as_text()
                    .unwrap_or_else(|| "No description...".to_string()),
                emoji: false,
            })
            .into(),
            fields: Vec::default(),
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

        let date = value.date().format("%d %b %Y %I:%M %p %Z");

        let footer = if let Some(s) = value.source() {
            let src = if let Some(url) = s.url {
                format!("<{}|{}>", url, s.title)
            } else {
                s.title.to_string()
            };
            format!("{} | _{}_", src, date)
        } else {
            date.to_string()
        };

        let context = Context {
            elements: vec![ContextElement::Text(Text::Markdown(MarkdownText {
                text: footer,
                verbatim: false,
            }))],
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
