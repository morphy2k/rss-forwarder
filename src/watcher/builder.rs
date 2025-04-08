use crate::{sink::Sink, Result};

use super::{Watcher, DEFAULT_INTERVAL};

use std::time::Duration;

use chrono::DateTime;
use reqwest::Client;
use url::Url;

pub struct WatcherBuilder<State>(State);

impl WatcherBuilder<WantsUrl> {
    pub fn with_client(client: Client) -> Self {
        WatcherBuilder(WantsUrl(client))
    }
}

impl Default for WatcherBuilder<WantsUrl> {
    fn default() -> Self {
        WatcherBuilder(WantsUrl(Client::default()))
    }
}

pub struct WantsUrl(Client);

impl WatcherBuilder<WantsUrl> {
    pub fn url(self, url: Url) -> WatcherBuilder<WantsSink> {
        WatcherBuilder(WantsSink(self.0 .0, url))
    }
}

pub struct WantsSink(Client, Url);

impl WatcherBuilder<WantsSink> {
    pub fn sink<T: Sink>(self, sink: T) -> WatcherBuilder<Ready<T>> {
        WatcherBuilder(Ready {
            url: self.0 .1,
            sink,
            client: self.0 .0,
            interval: DEFAULT_INTERVAL,
            retry_limit: 3,
        })
    }
}

pub struct Ready<T: Sink> {
    url: Url,
    sink: T,
    client: Client,
    interval: Duration,
    retry_limit: usize,
}

impl<T: Sink> WatcherBuilder<Ready<T>> {
    pub fn interval(self, interval: Duration) -> WatcherBuilder<Ready<T>> {
        WatcherBuilder(Ready { interval, ..self.0 })
    }

    pub fn retry_limit(self, retry_limit: usize) -> WatcherBuilder<Ready<T>> {
        WatcherBuilder(Ready {
            retry_limit,
            ..self.0
        })
    }

    pub fn build(self) -> Result<Watcher<T>> {
        let data = self.0;
        Ok(Watcher {
            url: data.url,
            sink: data.sink,
            client: data.client,
            interval: data.interval,
            retry_limit: data.retry_limit,
            retries_left: data.retry_limit,
            last_date: DateTime::default(),
        })
    }
}
