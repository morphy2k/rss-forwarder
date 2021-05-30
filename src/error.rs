#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("task error: {0}")]
    Task(#[from] tokio::task::JoinError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("rss error: {0}")]
    Rss(#[from] rss::Error),
    #[error("toml error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("argument error: {0}")]
    Argument(#[from] pico_args::Error),
}
