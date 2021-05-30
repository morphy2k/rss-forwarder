use crate::{sink::Sink, Result};

use std::time::Duration;

use chrono::{DateTime, Utc};
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
    last_check: DateTime<Utc>,
}

impl<'a, T: Sink> Watcher<T> {
    pub fn new<U: IntoUrl>(url: U, sink: T, interval: Option<Duration>) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            sink,
            interval: interval.unwrap_or(DEFAULT_INTERVAL),
            client: Client::builder().build()?,
            last_check: Utc::now(),
        })
    }

    pub async fn watch(&mut self) -> Result<()> {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            interval.tick().await;
            let channel = self.fetch().await?;

            // pub_date conversion impl ?
            let items = match self.get_new_items(channel.items()) {
                Some(v) => v,
                None => continue,
            };

            self.last_check = Utc::now();

            debug!("pushing {} items from \"{}\"", items.len(), channel.title());

            self.sink.push(items).await?;
        }
    }

    fn get_new_items(&self, items: &'a [Item]) -> Option<&'a [Item]> {
        let mut idx = 0;
        for (i, item) in items.iter().enumerate() {
            if DateTime::parse_from_rfc2822(item.pub_date().unwrap())
                .unwrap()
                .ge(&self.last_check)
            {
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
        let body = self
            .client
            .get(self.url.as_ref())
            .send()
            .await?
            .bytes()
            .await?;
        let channel = Channel::read_from(&body[..])?;

        Ok(channel)
    }
}
