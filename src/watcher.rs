use crate::{
    error::Error,
    feed::{item::FeedItem, Feed},
    sink::Sink,
    Result,
};

use std::time::Duration;

use chrono::{DateTime, FixedOffset};
use reqwest::{Client, IntoUrl, Url};
use tokio::sync::broadcast::Receiver;
use tracing::{debug, error};

const DEFAULT_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct Watcher<T: Sink> {
    url: Url,
    sink: T,
    interval: Duration,
    client: Client,
    retry_limit: usize,
    retries_left: usize,
    last_date: Option<DateTime<FixedOffset>>,
}

impl<T: Sink> Watcher<T> {
    pub fn new<U: IntoUrl>(
        url: U,
        sink: T,
        interval: Option<Duration>,
        client: Client,
        retry_limit: usize,
    ) -> Result<Self> {
        Ok(Self {
            url: url.into_url()?,
            sink,
            interval: interval.unwrap_or(DEFAULT_INTERVAL),
            client,
            retry_limit,
            retries_left: retry_limit,
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
                    if is_retriable(&e) && self.retries_left > 0 {
                        error!(error =? e, "error while getting items");
                        self.retries_left -= 1;
                        continue;
                    } else {
                        return Err(e);
                    }
                }
            };

            let items = feed.items();

            if items.is_empty() {
                continue;
            }

            let last = items.first().unwrap();

            if self.last_date.is_none() {
                self.last_date = last.date().into();
            }

            let news = match self.get_new_items(&items) {
                Some(v) => v,
                None => continue,
            };

            debug!(
                feed = feed.title(),
                count = news.len(),
                "pushing items from feed"
            );

            if let Err(err) = self.sink.push(news).await {
                if is_retriable(&err) && self.retries_left > 0 {
                    error!(error =? err, "error while pushing items");
                    self.retries_left -= 1;
                    continue;
                } else {
                    return Err(err);
                }
            }

            self.last_date = last.date().into();

            if self.retries_left != self.retry_limit {
                self.retries_left = self.retry_limit;
            }
        }

        self.sink.shutdown().await?;

        Ok(())
    }

    fn get_new_items<'a, I>(&self, items: &'a [I]) -> Option<&'a [I]>
    where
        I: FeedItem<'a>,
    {
        let mut idx = 0;

        for (i, item) in items.iter().enumerate() {
            if item.date().gt(&self.last_date.unwrap()) {
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

fn is_retriable(err: &Error) -> bool {
    match err {
        Error::Request(e) => e.is_timeout() || e.is_connect() || e.is_status(),
        _ => false,
    }
}
