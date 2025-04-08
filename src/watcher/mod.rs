mod builder;

use crate::{
    build_client,
    config::Config,
    error::{ConfigError, Error, WatcherError},
    feed::{item::FeedItem, Feed},
    sink::{AnySink, Sink},
    Result,
};

use std::time::Duration;

use builder::WatcherBuilder;
use chrono::{DateTime, FixedOffset};
use reqwest::{Client, Url};
use tokio::{
    sync::broadcast::{self, Receiver, Sender},
    task::JoinSet,
};
use tracing::{debug, error, info};

const DEFAULT_INTERVAL: Duration = Duration::from_secs(60);

pub struct WatcherCollection<S: Sink> {
    scheduled_watchers: Vec<Watcher<S>>,
    kill_tx: Sender<()>,
}

impl<S: Sink + 'static> WatcherCollection<S> {
    pub async fn wait(mut self) -> Result<()> {
        let mut watchers = JoinSet::new();

        for (idx, watcher) in self.scheduled_watchers.drain(..).enumerate() {
            let rx = self.kill_tx.subscribe();

            info!(
                index = idx,
                url = %watcher.url,
                "starting watcher",
            );

            watchers.spawn(async move {
                if let Err(err) = watcher.watch(rx).await {
                    error!(
                        index = idx,
                        error = %err,
                        "shutting down watcher due to an error",
                    );
                    return Err(err);
                }

                Ok(())
            });
        }

        let mut failed = false;

        while let Some(res) = watchers.join_next().await {
            let abort = if let Ok(r) = res { r.is_err() } else { true };
            if abort && !failed {
                watchers.abort_all();
                failed = true;
            }
        }

        if failed {
            return Err(WatcherError::Failed)?;
        }

        Ok(())
    }

    pub fn shutdown_handle(&self) -> Sender<()> {
        self.kill_tx.clone()
    }
}

impl<S: Sink + 'static> From<Vec<Watcher<S>>> for WatcherCollection<S> {
    fn from(watchers: Vec<Watcher<S>>) -> Self {
        let (kill_tx, _) = broadcast::channel(watchers.len());

        Self {
            scheduled_watchers: watchers,
            kill_tx,
        }
    }
}

impl TryFrom<Config> for WatcherCollection<AnySink> {
    type Error = Error;

    fn try_from(Config { default, feeds }: Config) -> Result<Self> {
        let client = build_client()?;

        let mut scheduled_watchers = Vec::new();

        for feed in feeds.into_iter() {
            let sink = feed
                .sink
                .unwrap_or(default.sink.clone().ok_or(ConfigError::MissingSink)?)
                .sink(&client)?;

            let watcher = WatcherBuilder::with_client(client.clone())
                .url(feed.url)
                .sink(sink)
                .interval(feed.interval.unwrap_or(default.interval))
                .retry_limit(feed.retry_limit.unwrap_or(default.retry_limit))
                .build()?;

            scheduled_watchers.push(watcher);
        }

        Ok(Self::from(scheduled_watchers))
    }
}

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
        Error::Request(e) if e.is_timeout() || e.is_connect() => true,
        Error::Request(e) if e.is_status() => {
            let status = e.status().unwrap();
            status.is_server_error()
        }
        _ => false,
    }
}
