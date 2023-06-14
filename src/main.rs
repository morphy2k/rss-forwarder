mod config;
mod error;
mod feed;
mod sink;
mod watcher;

use crate::{
    config::{Config, Feed},
    watcher::Watcher,
};

use std::{
    collections::HashMap,
    env,
    io::{stdout, IsTerminal},
    path::PathBuf,
    process,
    str::FromStr,
    time::Duration,
};

use error::Error;
use pico_args::Arguments;
use reqwest::Client;
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
    task::JoinSet,
};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static ALLOC: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
struct Args {
    config: PathBuf,
    format: LogFormat,
    no_color: bool,
    debug: bool,
    verbose: bool,
}

#[derive(Debug, Default)]
enum LogFormat {
    #[default]
    Full,
    Pretty,
    Compact,
    Json,
}

struct InvalidLogFormat;

impl std::fmt::Display for InvalidLogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid log format")
    }
}

impl FromStr for LogFormat {
    type Err = InvalidLogFormat;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let value = match s {
            "full" => LogFormat::Full,
            "pretty" => LogFormat::Pretty,
            "compact" => LogFormat::Compact,
            "json" => LogFormat::Json,
            _ => return Err(InvalidLogFormat),
        };

        Ok(value)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = match parse_args() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Argument error: {e}");
            process::exit(1);
        }
    };

    let subscriber = tracing_subscriber::fmt()
        .with_line_number(args.debug)
        .with_thread_ids(args.debug)
        .with_target(args.debug)
        .with_env_filter(parse_env_filter(args.debug, args.verbose))
        .with_ansi(stdout().is_terminal() && !args.no_color);

    match args.format {
        LogFormat::Full => subscriber.init(),
        LogFormat::Pretty => subscriber.pretty().init(),
        LogFormat::Compact => subscriber.compact().init(),
        LogFormat::Json => subscriber.json().init(),
    };

    let config = match Config::from_file(args.config).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error while reading config: {e}");
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
      --debug                Enables debug mode
      -f, --format <FORMAT>  Sets the log format (json, pretty, compact)
      --no-color             Disables colored output
      -h, --help             Show help information
      -v, --version          Show version info
";

fn print_help() {
    print!(
        "\
{NAME} v{VERSION} by {AUTHORS}
{DESCRIPTION}

    USAGE: {NAME} [OPTIONS] <CONFIG_FILE>

    {OPTIONS}
",
    );
}

fn parse_args() -> Result<Args> {
    let mut pargs = Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print_help();
        process::exit(0);
    }

    if pargs.contains(["-v", "--version"]) {
        println!("v{VERSION}");
        process::exit(0);
    }

    let args = Args {
        debug: pargs.contains("--debug"),
        verbose: pargs.contains("--verbose"),
        format: pargs
            .opt_value_from_str(["-f", "--format"])?
            .unwrap_or_default(),
        no_color: pargs.contains("--no-color"),
        config: pargs.free_from_str()?,
    };

    pargs.finish();

    Ok(args)
}

fn parse_env_filter(debug: bool, verbose: bool) -> EnvFilter {
    match (env::var("RUST_LOG").is_err(), debug, verbose) {
        (true, true, true) => EnvFilter::builder()
            .parse("debug")
            .expect("should be a valid directive"),
        (true, false, true) => EnvFilter::builder()
            .parse("info")
            .expect("should be a valid directive"),
        (true, true, false) => EnvFilter::builder()
            .parse("rss_forwarder=debug,reqwest=debug")
            .expect("should be a valid directive"),
        (true, false, false) => EnvFilter::builder()
            .parse("rss_forwarder=info")
            .expect("should be a valid directive"),
        (false, _, _) => EnvFilter::from_default_env(),
    }
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
