use crate::sink::SinkOptions;

use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub feeds: HashMap<String, Feed>,
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
