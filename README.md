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

| Platform | Architecture | Container image |
| -------- | ------------ | :---------------: |
| Linux | x86_64, aarch64 | ✅ |
| macOS | x86_64, aarch64 | ❌ |
| Windows | - | ❌ |

## Installation

### Official binary

Download the binary from the [releases page](https://github.com/morphy2k/rss-forwarder/releases)

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

### Feed

| Field        | Type | Required | Default | Description  |
| -------------|:----:|:--------:|:--------:| ----------- |
| `url`      | string | Yes | | URL to the RSS feed |
| `interval`  | string      | No | 60s |  Specifies the time interval between checks. E.g. `10m`, `3h`, `1d`. |
| `retry_limit` | uint      | No | 10 |  Specifies the retries after certain errors. |
| `sink` | object | Yes | | Sink options |

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
# Feed 1
[feeds.github-blog]
url = "https://github.blog/all.atom"
interval = "10m"
retry_limit = 5
sink.type = "discord"
sink.url = "https://discord.com/api/webhooks/84175.../OZdejNBCL1..."

# Feed 2
[feeds.rust-blog]
url = "https://blog.rust-lang.org/feed.xml"
interval = "1m"

[feeds.rust-blog.sink]
type = "custom"
command = "bash"
arguments = ["-c", "cat - >> ./rust-blog.log"]
```
