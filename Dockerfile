FROM rust:1.68.1 as builder

ENV PKG_CONFIG_ALLOW_CROSS=1

WORKDIR /usr/src/rss-forwarder
COPY . .
RUN cargo install --path .

FROM gcr.io/distroless/cc-debian11

LABEL repository="https://github.com/morphy2k/rss-forwarder"
LABEL maintainer="Markus Wiegand <mail@morphy2k.dev>"

LABEL org.opencontainers.image.source="https://github.com/morphy2k/rss-forwarder"

COPY --from=builder /usr/local/cargo/bin/rss-forwarder /usr/local/bin/rss-forwarder

CMD ["rss-forwarder", "/data/config.toml"]
