# Changelog

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
- Fixed and improved behaivor on errors.
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
