use crate::{sink::SinkOptions, Result};

use std::{collections::HashMap, path::Path, time::Duration};

use serde::Deserialize;
use tokio::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub feeds: HashMap<String, Feed>,
}

impl Config {
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = fs::read_to_string(path).await?;
        let config = toml::from_str(&file)?;

        Ok(config)
    }
}

#[derive(Debug, Deserialize)]
pub struct Feed {
    pub url: String,
    pub sink: SinkOptions,
    #[serde(default, with = "humantime_serde")]
    pub interval: Option<Duration>,
    #[serde(default = "retry_limit_default")]
    pub retry_limit: usize,
}

const fn retry_limit_default() -> usize {
    10
}
