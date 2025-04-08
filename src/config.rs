use crate::{sink::SinkOptions, Result};

use std::{path::Path, time::Duration};

use serde::Deserialize;
use tokio::fs;
use url::Url;

const fn retry_limit_default() -> usize {
    10
}

const fn interval_default() -> Duration {
    Duration::from_secs(60)
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub default: Default,
    pub feeds: Vec<Feed>,
}

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::read_to_string(path).await?;
        let config = toml::from_str(&file)?;

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct Default {
    pub sink: Option<SinkOptions>,
    #[serde(default = "interval_default", with = "humantime_serde")]
    pub interval: Duration,
    #[serde(default = "retry_limit_default")]
    pub retry_limit: usize,
}

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub url: Url,
    pub sink: Option<SinkOptions>,
    pub interval: Option<Duration>,
    pub retry_limit: Option<usize>,
}
