# Changelog

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
