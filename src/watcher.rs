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
    last_date: DateTime<FixedOffset>,
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
            last_date: DateTime::default(),
        })
    }

    #[tracing::instrument(
        name = "watch",
        skip(self, kill),
        fields(
            url = %self.url,
            interval = ?self.interval,
            retry_limit = self.retry_limit,
        )
        level = "debug"
    )]
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
                Err(err) => {
                    if is_retriable(&err) && self.retries_left > 0 {
                        self.retries_left -= 1;
                        error!(
                            error = %err,
                            retries_left = self.retries_left,
                            "error while fetching feed",
                        );
                        continue;
                    } else {
                        return Err(err);
                    }
                }
            };

            let items = feed.items();

            let Some(last) = items.first() else {
                debug!("no items in feed");
                continue;
            };

            if self.last_date.timestamp() == 0 {
                debug!(
                    date = %last.date(),
                    "no date set, setting to last item date",
                );
                self.last_date = last.date();
                continue;
            }

            let Some(news) = self.get_new_items(&items) else {
                debug!(
                    since = %self.last_date,
                    "found no new items",
                );
                continue;
            };

            debug!(
                count = news.len(),
                since = %self.last_date,
                "found new items",
            );

            if let Err(err) = self.sink.push(news).await {
                if is_retriable(&err) && self.retries_left > 0 {
                    self.retries_left -= 1;
                    error!(
                        error = %err,
                        retries_left = self.retries_left,
                        "error while pushing items to sink",
                    );
                    continue;
                } else {
                    return Err(err);
                }
            }

            debug!(
                date = %last.date(),
                "updating last date",
            );
            self.last_date = last.date();

            if self.retries_left != self.retry_limit {
                debug!("resetting retries");
                self.retries_left = self.retry_limit;
            }
        }

        debug!("shutting down");

        self.sink.shutdown().await?;

        Ok(())
    }

    fn get_new_items<'a, I>(&self, items: &'a [I]) -> Option<&'a [I]>
    where
        I: FeedItem<'a>,
    {
        let mut idx = 0;
        for (i, item) in items.iter().enumerate() {
            if item.date().gt(&self.last_date) {
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
        debug!("fetching feed");

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
