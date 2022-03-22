mod config;
mod error;
mod feed;
mod sink;
mod watcher;

use crate::{
    config::{Config, Feed},
    watcher::Watcher,
};

use std::{collections::HashMap, env, path::PathBuf, process, time::Duration};

use error::Error;
use futures::future;
use pico_args::Arguments;
use reqwest::Client;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinHandle,
};
use tracing::{error, info};

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
            eprintln!("Argument error: {}", e);
            process::exit(1);
        }
    };

    if env::var("RUST_LOG").is_err() {
        if args.debug {
            env::set_var("RUST_LOG", "debug");
        } else {
            env::set_var("RUST_LOG", "info");
        }
    }
    tracing_subscriber::fmt::init();

    let config = match Config::from_file(args.config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error while reading config: {}", e);
            process::exit(1);
        }
    };

    let client = build_client()?;

    let tasks = watch_feeds(config.feeds, client)?;

    future::try_join_all(tasks).await?;

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

    let (tx, _) = broadcast::channel(tasks.capacity());

    for (name, config) in feeds.into_iter() {
        let sink = config.sink.sink(&client)?;
        let watcher = Watcher::new(
            config.url,
            sink,
            config.interval,
            client.clone(),
            config.retry_limit,
        )?;

        let rx = tx.subscribe();

        tasks.push(tokio::spawn(async move {
            info!("Start watcher for \"{}\"", name);

            if let Err(e) = watcher.watch(rx).await {
                error!(feed =? name, error =? e, "Watcher stopped with an error");
                return Err(e);
            }

            info!("Watcher for \"{}\" has stopped", name);

            Ok(())
        }));
    }

    tokio::spawn(async move {
        let mut sig_int = signal(SignalKind::interrupt()).unwrap();
        let mut sig_term = signal(SignalKind::terminate()).unwrap();

        tokio::select! {
            _ = sig_int.recv() => {},
            _ = sig_term.recv() => {},
        };

        tx.send(()).unwrap();
    });

    Ok(tasks)
}
