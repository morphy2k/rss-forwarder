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
use pico_args::Arguments;
use reqwest::Client;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinSet,
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
            env::set_var("RUST_LOG", "rss_forwarder=debug,reqwest=debug");
        } else {
            env::set_var("RUST_LOG", "rss_forwarder=info");
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

    let mut tasks = watch_feeds(config.feeds, client)?;
    let mut task_failed = false;
    while let Some(res) = tasks.join_next().await {
        let abort = if let Ok(r) = res { r.is_err() } else { true };
        if abort && !task_failed {
            tasks.abort_all();
            task_failed = true;
        }
    }

    if task_failed {
        error!("Terminate due to a faulty watcher");
        process::exit(1);
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

fn watch_feeds(feeds: HashMap<String, Feed>, client: Client) -> Result<JoinSet<Result<()>>> {
    let mut tasks = JoinSet::new();

    let (tx, _) = broadcast::channel(feeds.len());

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

        tasks.spawn(async move {
            info!("Start watcher for \"{}\"", name);

            if let Err(e) = watcher.watch(rx).await {
                error!(feed =? name, error =? e, "Watcher stopped with an error");
                return Err(e);
            }

            Ok(())
        });
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
