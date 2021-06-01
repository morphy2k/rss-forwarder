use crate::{item::ExtItem, sink::Sink, Result};

use std::time::Duration;

use blake3::Hash;
use chrono::{DateTime, FixedOffset};
use log::debug;
use reqwest::{Client, IntoUrl, Url};
use rss::{Channel, Item};

const DEFAULT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct Watcher<T: Sink> {
    url: Url,
    sink: T,
    interval: Duration,
    client: Client,
    last_date: Option<DateTime<FixedOffset>>,
    last_hash: Option<Hash>,
}

impl<'a, T: Sink> Watcher<T> {
    pub fn new<U: IntoUrl>(url: U, sink: T, interval: Option<Duration>) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            sink,
            interval: interval.unwrap_or(DEFAULT_INTERVAL),
            client: Client::builder().build()?,
            last_date: None,
            last_hash: None,
        })
    }

    pub async fn watch(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;
            let channel = self.fetch().await?;
            let items = channel.items();

            if items.is_empty() {
                continue;
            }

            let last = items.first().unwrap();

            if self.last_hash.is_none() && self.last_date.is_none() {
                self.last_date = last.pub_date_as_datetime();
                self.last_hash = Some(last.compute_hash());
            }

            let news = match self.get_new_items(items) {
                Some(v) => v,
                None => continue,
            };

            debug!("pushing {} items from \"{}\"", news.len(), channel.title());

            self.sink.push(news).await?;

            self.last_date = last.pub_date_as_datetime();
            self.last_hash = Some(last.compute_hash());
        }
    }

    fn get_new_items(&self, items: &'a [Item]) -> Option<&'a [Item]> {
        let mut idx = 0;

        for (i, item) in items.iter().enumerate() {
            let is_new = if self.last_date.is_some() && item.pub_date().is_some() {
                item.pub_date_as_datetime()
                    .unwrap()
                    .gt(&self.last_date.unwrap())
            } else {
                item.compute_hash() != self.last_hash.unwrap()
            };

            if is_new {
                idx = i;
            } else {
                if i == 0 {
                    return None;
                }
                break;
            }
        }

        Some(&items[..=idx])
    }

    async fn fetch(&self) -> Result<Channel> {
        let res = self.client.get(self.url.as_ref()).send().await?;
        let body = res.error_for_status()?.bytes().await?;

        let channel = Channel::read_from(&body[..])?;

        Ok(channel)
    }
}
