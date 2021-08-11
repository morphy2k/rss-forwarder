[![test-badge]][test-workflow]
[![crates-badge]][crates.io]

[test-workflow]: https://github.com/morphy2k/rss-forwarder/actions/workflows/test.yml
[crates.io]: https://crates.io/crates/rss-forwarder
[crates-badge]: https://img.shields.io/crates/v/rss-forwarder
[test-badge]: https://github.com/morphy2k/rss-forwarder/actions/workflows/test.yml/badge.svg

# RSS Forwarder

Checks RSS feeds for new entries and forwards them to different targets (sinks), like webhooks.

## Supported sinks

| Sink        | Type value     |
| ------------| :-------: |
| [Discord](#discord-sink) | `discord` |

*More is planned*

## Installation

### Official binary

[`jq`](https://stedolan.github.io/jq/) required!

```BASH
curl --proto '=https' --tlsv1.3 -LO "$(curl --proto '=https' --tlsv1.3 -sSf https://api.github.com/repos/morphy2k/rss-forwarder/releases/latest | jq -r ".assets[] | select(.name == \"rss-forwarder-linux-x86_64\") | .browser_download_url")"
chmod +x rss-forwarder-linux-x86_64
sudo mv rss-forwarder-linux-x86_64 /usr/local/bin/rss-forwarder
```

### Cargo

```BASH
cargo install rss-forwarder
```

### Container image

See [GitHub container package](https://github.com/morphy2k/rss-forwarder/pkgs/container/rss-forwarder)

## Usage

```TXT
USAGE: rss-forwarder [OPTIONS] <CONFIG_FILE>

OPTIONS:
--debug             Enables debug mode
-h, --help          Show help information
-v, --version       Show version info
```

[Examples](example)

## Configuration

The feed configuration is passed as a TOML file.

### Feed

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `url`      | string | Yes | | URL to the RSS feed |
| `interval`      | string      | No | 60s |  Specifies the time interval between checks. E.g. `10m`, `3h`, `1d`. |
| `sink` | object | Yes | | Sink options |

### Discord Sink

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `type` | string | Yes | | Sink type |
| `url` | string | Yes | | Discord webhook url |

### Example

```TOML
# Feed 1
[feeds.nyt-world]
url = "https://rss.nytimes.com/services/xml/rss/nyt/World.xml"
interval = "10m"
sink.type = "discord"
sink.url = "https://discord.com/api/webhooks/84175.../OZdejNBCL1..."

# Feed 2
[feeds.rust-blog]
url = "https://blog.rust-lang.org/feed.xml"
interval = "1m"

[feeds.rust-blog.sink]
type = "discord"
url = "https://discord.com/api/webhooks/84175.../OZdejNBCL1..."
```
