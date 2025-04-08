[![test-badge]][test-workflow]
[![crates-badge]][crates.io]

[test-workflow]: https://github.com/morphy2k/rss-forwarder/actions/workflows/test.yml
[crates.io]: https://crates.io/crates/rss-forwarder
[crates-badge]: https://img.shields.io/crates/v/rss-forwarder
[test-badge]: https://github.com/morphy2k/rss-forwarder/actions/workflows/test.yml/badge.svg

# RSS Forwarder

Checks RSS/Atom feeds for new entries and forwards them to different targets (called "sinks"), such as webhooks or applications/scripts.

## Supported sinks

| Sink        | Type value | Description |
| ------------| :-------: | ----------- |
| [Discord](#discord-sink) | `discord` | Discord webhook |
| [Slack](#slack-sink) | `slack` | Slack webhook |
| [Custom](#custom-sink) | `custom` | JSON stream to stdin |

## Supported platforms

| Platform | Architecture | Image* |
| -------- | ------------ | :----: |
| Linux | x86_64, aarch64 | ✅ |
| macOS | x86_64, aarch64 | ❌ |
| Windows | - | ❌ |

*\* Indicates whether a container (Docker) image is available*

## Installation

### Official binary

You can download the latest binary from the [releases page](https://github.com/morphy2k/rss-forwarder/releases)

### Cargo

```BASH
cargo install rss-forwarder@<version>
# or from source
cargo install --git https://github.com/morphy2k/rss-forwarder.git
```

### Container image

See [GitHub container package](https://github.com/morphy2k/rss-forwarder/pkgs/container/rss-forwarder)

## Usage

```TXT
USAGE: rss-forwarder [OPTIONS] <CONFIG_FILE>

OPTIONS:
  -f, --format <FORMAT>  Log format: full, pretty, compact, json (default: full)
  --color <WHEN>         Colorize output: auto, always, never (default: auto)
  --debug                Enables debug mode
  --verbose              Enables verbose mode
  -h, --help             Show this help message
  -v, --version          Show version information
```

[Examples](example)

## Configuration

The feed configuration is passed as a TOML file.

### Default Settings

The `default` section defines global settings that apply to all feeds:

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `interval`  | string      | No | 60s |  Specifies the time interval between checks. E.g. `10m`, `3h`, `1d`. |
| `retry_limit` | uint      | No | 10 |  Specifies the retries after certain errors. |
| `sink` | object | No | | Default sink options to use for all feeds |

### Feeds

The `feeds` section is an array that contains individual feed configurations:

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `url`      | string | Yes | | URL to the RSS feed |
| `interval`  | string      | No | From default |  Override the default interval for this feed |
| `retry_limit` | uint      | No | From default |  Override the default retry limit for this feed |
| `sink` | object | No | From default | Override sink options for this feed |

### Discord Sink

Sends feed items to a [Discord webhook](https://support.discord.com/hc/en-us/articles/228383668-Intro-to-Webhooks)

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `type` | string | Yes | | Sink type |
| `url` | string | Yes | | Discord webhook URL |

### Slack Sink

Sends feed items to a [Slack webhook](https://api.slack.com/messaging/webhooks)

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `type` | string | Yes | | Sink type |
| `url` | string | Yes | | Slack webhook URL |

### Custom Sink

Streams feed items in [NDJSON](https://en.wikipedia.org/wiki/JSON_streaming#Line-delimited_JSON) to stdin.

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `type` | string | Yes | | Sink type |
| `command` | string | Yes | | Program path |
| `arguments` | [string] | No | | Arguments to pass to the program. |

#### JSON Example

```JSON
{
    "title": "Item Example",
    "description": "This is an example",
    "content": "Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua.",
    "link": "https://example.com/news/item-example",
    "date": "2021-09-08T23:12:05+02:00",
    "authors": [
        {
            "name": "Jane Doe",
            "email": "jane@example.com",
            "uri": "https://example.com/author/jane-doe"
        }
    ]
}
```

### Config Example

```TOML
# Default settings for all feeds
[default]
interval = "5m"
retry_limit = 10

# Default sink configuration
[default.sink]
type = "discord"
url = "https://discord.com/api/webhooks/84175.../OZdejNBCL1..."

# Feed 1
[[feeds]]
url = "https://github.blog/all.atom"
interval = "10m"  # Override default interval
retry_limit = 5   # Override default retry limit

# Feed 2
[[feeds]]
url = "https://blog.rust-lang.org/feed.xml"
interval = "1m"   # Override default interval
sink.type = "custom"  # Override default sink
sink.command = "bash"
sink.arguments = ["-c", "cat - >> ./rust-blog.log"]

# Feed 3 - uses all default settings
[[feeds]]
url = "https://example.com/feed.xml"
```
