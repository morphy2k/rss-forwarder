# Changelog

## v0.7.1

- Dependencies updated

## v0.7.0

_Changes since [v0.7.0-beta.3](#v070-beta3)_

- Dependencies updated

## v0.7.0-beta.3

- Binaries and container images for multiple platforms added [#181](https://github.com/morphy2k/rss-forwarder/pull/181)
- Dockerfile fixed and updated to Debian 12 [d7e26bd](<https://github.com/morphy2k/rss-forwarder/commit/d7e26bd318d2d50a24b9ccdb61568cd73294febd>)
- Dependencies updated

## v0.7.0-beta.2

- HTTP header `Accept` default set to `application/atom+xml, application/rss+xml, application/xml, text/xml` [#179](https://github.com/morphy2k/rss-forwarder/pull/178)
- Rust to 1.75 updated
- Dependencies updated

## v0.7.0-beta.1

- Status error handling improved. Retries no longer take place for client errors (4xx) [4a8eda1](<https://github.com/morphy2k/rss-forwarder/commit/4a8eda155eb1a2ed9b399adccac7248d2da7652a>)
- Rust to 1.74.1 updated
- Dependencies updated

## v0.7.0-alpha.1

- Log formats **full**, **pretty**, **compact** and **json** added [b91d0be](https://github.com/morphy2k/rss-forwarder/commit/b91d0be8e56969643d66b40f34ffbd0d9ec9302d)
- Ability to control the ANSI color output added [b91d0be](https://github.com/morphy2k/rss-forwarder/commit/b91d0be8e56969643d66b40f34ffbd0d9ec9302d)
- Verbose flag added [b91d0be](https://github.com/morphy2k/rss-forwarder/commit/b91d0be8e56969643d66b40f34ffbd0d9ec9302d)
- Log messages improved [b91d0be](https://github.com/morphy2k/rss-forwarder/commit/b91d0be8e56969643d66b40f34ffbd0d9ec9302d)
- SOCKS5 proxy support added [ad4cc89](https://github.com/morphy2k/rss-forwarder/commit/ad4cc89beabdffaa0237ee2ca4eded88dcc339c7)
- HTTP compression **GZIP**, **Deflate** and **Brotli** support added [ad4cc89](https://github.com/morphy2k/rss-forwarder/commit/ad4cc89beabdffaa0237ee2ca4eded88dcc339c7)
- Default TLS library changed to [`rustls`](https://github.com/rustls/rustls) [ad4cc89](https://github.com/morphy2k/rss-forwarder/commit/ad4cc89beabdffaa0237ee2ca4eded88dcc339c7)
- Default DNS resolver changed to [`trust-dns`](https://github.com/bluejekyll/trust-dns) [ad4cc89](https://github.com/morphy2k/rss-forwarder/commit/ad4cc89beabdffaa0237ee2ca4eded88dcc339c7)
- Memory allocator changed to [`mimalloc`](https://github.com/microsoft/mimalloc) [33530e1](https://github.com/morphy2k/rss-forwarder/commit/33530e18b9f1a90cea38d664f74fd0e9df9595df)
- Rust to 1.70 updated
- Dependencies updated

## v0.6.1

- Dependencies updated (includes security fixes)

## v0.6.0

_Changes since [v0.6.0-beta.0](#v060-beta0)_

- Rust updated to 1.67
- Dependencies updated

### BREAKING

Process is terminated when a watcher stops with an error.

## v0.6.0-beta.0

- Exclude jemalloc for MSVC target ~~(This should make it possible to build on Windows)~~ [fd98808](https://github.com/morphy2k/rss-forwarder/commit/fd98808d737de1e8d5e4c8e13abe9e6b5034c7f3)
- Fixed and improved behavior on errors.
- Rust updated to 1.66
- Some internal minor improvements
- Dependencies updated

## v0.6.0-alpha.0

- Job handling updated. Errors now lead to the termination of the process [71320b9](https://github.com/morphy2k/rss-forwarder/commit/71320b9da4a2036e7440691bff59a2c76e930386)
- Logging improved [e7022a8](https://github.com/morphy2k/rss-forwarder/commit/e7022a877e52d8dcdf01ed7c37d5e6de20623604)
- Rust updated to 1.59
- Distroless Docker image updated to Debian 11
- Dependencies updated

## v0.5.1

- Rust image updated (fixes container image building) [b3099b8](https://github.com/morphy2k/rss-forwarder/commit/b3099b8)

## v0.5.0

- Migration to Rust 2021 edition (MSRV 1.56)
- Mitigations for [RUSTSEC-2020-0071](https://rustsec.org/advisories/RUSTSEC-2020-0071)
- Dependencies updated

## v0.4.1

- Keyword removed to match rules (fixes crates.io publishing) [7edbb9e](https://github.com/morphy2k/rss-forwarder/commit/7edbb9e)

## v0.4.0

- Slack webhook sink added [#6](https://github.com/morphy2k/rss-forwarder/pull/6)
- Feed module improved and refactored [3c5ccd6...90730a6](https://github.com/morphy2k/rss-forwarder/compare/3c5ccd6...90730a6)
- Feed level metadata to item added [2f8d31f](https://github.com/morphy2k/rss-forwarder/commit/2f8d31f)
- Metadata to Discord object added [b8b23ce](https://github.com/morphy2k/rss-forwarder/commit/b8b23ce)
- Discord sink returns an error on bad status [b5a6737](https://github.com/morphy2k/rss-forwarder/commit/b5a6737)
- Rust version changed to **v1.55** [3c5ccd6](https://github.com/morphy2k/rss-forwarder/commit/3c5ccd6)
- Small improvements to the code
- Dependencies updated

### BREAKING

The JSON output of the sink **Custom** now contains only the link to reference.

```DIFF
{
    ...
-    "links": ["https://example.com/news/item-example"],
+    "link": "https://example.com/news/item-example",
    ...
}
```

## v0.3.0

- Atom feed support added [#5](https://github.com/morphy2k/rss-forwarder/pull/5)
- **Custom** sink added [#4](https://github.com/morphy2k/rss-forwarder/pull/4)
- Retry on certain errors [6872ac](https://github.com/morphy2k/rss-forwarder/commit/6872ac)
- Dependencies updated

## v0.2.0

- HTML to text conversion added ([#3](https://github.com/morphy2k/rss-forwarder/pull/3))
- Error messages improved [fb1deb3](https://github.com/morphy2k/rss-forwarder/commit/fb1deb3) [411f55a](https://github.com/morphy2k/rss-forwarder/commit/411f55a)
- Graceful shutdown improved [634a351](https://github.com/morphy2k/rss-forwarder/commit/634a351)
- Unused dependency features disabled [975a8f2](https://github.com/morphy2k/rss-forwarder/commit/975a8f2) [db254d1](https://github.com/morphy2k/rss-forwarder/commit/db254d1)
- Dockerfile port exposing removed [5c354b0](https://github.com/morphy2k/rss-forwarder/commit/5c354b0)
- Dependencies updated
