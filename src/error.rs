#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("feed error: {0}")]
    Feed(#[from] FeedError),
    #[error("sink error: {0}")]
    Sink(String),
    #[error("json error: {0}")]
    Json(#[from] serde_json::error::Error),
    #[error("task error: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("argument error: {0}")]
    Argument(#[from] pico_args::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum FeedError {
    #[error("item error: {0}")]
    Item(String),
    #[error("rss error: {0}")]
    Rss(#[from] rss::Error),
    #[error("atom error: {0}")]
    Atom(#[from] atom_syndication::Error),
    #[error("html2text error: {0}")]
    Html2Text(#[from] html2text::Error),
}
