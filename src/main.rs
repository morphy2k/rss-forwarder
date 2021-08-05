mod config;
mod error;
mod item;
mod sink;
mod watcher;

use config::{Config, Feed};
use sink::{discord::Discord, SinkType};
use watcher::Watcher;

use std::{collections::HashMap, env, path::PathBuf, process, time::Duration};

use error::Error;
use log::info;
use pico_args::Arguments;
use reqwest::Client;
use tokio::{fs, task::JoinHandle};

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Args {
    config: PathBuf,
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    if args.debug {
        env::set_var("RUST_LOG", "debug");
    } else if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    let file = fs::read(args.config).await?;
    let config = toml::from_slice::<Config>(&file[..])?;

    let client = build_client()?;

    let tasks = watch_feeds(config.feeds, client)?;
    for handle in tasks.into_iter() {
        handle.await??;
    }

    Ok(())
}

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

fn build_client() -> Result<Client> {
    let client = Client::builder()
        .timeout(DEFAULT_TIMEOUT)
        .user_agent(USER_AGENT)
        .build()?;

    Ok(client)
}

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

const OPTIONS: &str = "\
    OPTIONS:
    --debug             Enables debug mode
    -h, --help          Show help information
    -v, --version       Show version info
";

fn print_help() {
    print!(
        "\
{} v{} by {}
{}

    USAGE: {:} [OPTIONS] <CONFIG_FILE>

    {}
",
        NAME, VERSION, AUTHORS, DESCRIPTION, NAME, OPTIONS,
    );
}

fn parse_args() -> Result<Args> {
    let mut pargs = Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print_help();
        process::exit(0);
    }

    if pargs.contains(["-v", "--version"]) {
        println!("v{}", VERSION);
        process::exit(0);
    }

    let args = Args {
        debug: pargs.contains("--debug"),
        config: pargs.free_from_str()?,
    };

    pargs.finish();

    Ok(args)
}

type Task<T> = JoinHandle<Result<T>>;

fn watch_feeds(feeds: HashMap<String, Feed>, client: Client) -> Result<Vec<Task<()>>> {
    let mut tasks = Vec::with_capacity(feeds.len());

    for (name, config) in feeds.into_iter() {
        let sink = match config.sink {
            SinkType::Discord { url } => Discord::new(url, client.clone())?,
        };

        let mut watcher = Watcher::new(config.url, sink, config.interval, client.clone())?;

        tasks.push(tokio::spawn(async move {
            info!("Watching {}", name);
            watcher.watch().await
        }));
    }

    Ok(tasks)
}
