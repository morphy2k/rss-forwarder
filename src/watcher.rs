use crate::{
    error::Error,
    feed::{Feed, Item},
    sink::Sink,
    Result,
};

use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use log::{debug, error};
use reqwest::{Client, IntoUrl, Url};
use tokio::sync::broadcast::Receiver;

const DEFAULT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct Watcher<T: Sink> {
    url: Url,
    sink: T,
    interval: Duration,
    client: Client,
    last_date: Option<DateTime<FixedOffset>>,
}

impl<'a, T: Sink> Watcher<T> {
    pub fn new<U: IntoUrl>(
        url: U,
        sink: T,
        interval: Option<Duration>,
        client: Client,
    ) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            sink,
            interval: interval.unwrap_or(DEFAULT_INTERVAL),
            client,
            last_date: None,
        })
    }

    pub async fn watch(mut self, mut kill: Receiver<()>) -> Result<()> {
        let mut interval = tokio::time::interval(self.interval);

        loop {
            tokio::select! {
                biased;
                _ = kill.recv() => break,
                _ = interval.tick() => {},
            };

            let feed = match self.fetch().await {
                Ok(c) => c,
                Err(e) => {
                    if is_timeout(&e) {
                        error!("Timeout while getting items: {}", e);
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };

            let items = feed.items()?;

            if items.is_empty() {
                continue;
            }

            let last = items.first().unwrap();

            if self.last_date.is_none() {
                self.last_date = last.date.into();
            }

            let news = match self.get_new_items(&items) {
                Some(v) => v,
                None => continue,
            };

            debug!("pushing {} items from \"{}\"", news.len(), feed.title());

            if let Err(err) = self.sink.push(news).await {
                if is_timeout(&err) {
                    error!("Timeout while pushing items: {}", err);
                    continue;
                } else {
                    return Err(err);
                }
            }

            self.last_date = last.date.into();
        }

        self.sink.shutdown().await?;

        Ok(())
    }

    fn get_new_items(&self, items: &'a [Item]) -> Option<&'a [Item]> {
        let mut idx = 0;

        for (i, item) in items.iter().enumerate() {
            if item.date.gt(&self.last_date.unwrap()) {
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

    async fn fetch(&self) -> Result<Feed> {
        let res = self.client.get(self.url.as_ref()).send().await?;
        let body = res.error_for_status()?.bytes().await?;

        let feed = Feed::read_from(&body[..])?;

        Ok(feed)
    }
}

fn is_timeout(err: &Error) -> bool {
    match err {
        Error::Request(e) => e.is_timeout(),
        _ => false,
    }
}
