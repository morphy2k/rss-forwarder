# Examples

## Set up an systemd service

### Install binary

```BASH
curl -O https://github.com/morphy2k/rss-forwarder/releases/download/<VERSION>/rss-forwarder-linux-x86_64
sudo chmod +x rss-forwarder-linux-x86_64
sudo mv rss-forwarder-linux-x86_64 /usr/local/bin/rss-forwarder
```

*Replace `<VERSION>` with the [latest release](https://github.com/morphy2k/rss-forwarder/releases/latest)*

### Add service user

```BASH
sudo useradd -r rss-forwarder-user -s /sbin/nologin
```

### Add config

```BASH
sudo mkdir /etc/rss-forwarder && sudo chown rss-forwarder-user:rss-forwarder-user /etc/rss-forwarder
sudo touch /etc/default/rss-forwarder
```

Save your config as `/etc/rss-forwarder/config.toml`

### Install service

```BASH
curl -O https://raw.githubusercontent.com/morphy2k/rss-forwarder/master/example/rss-forwarder.service && sudo mv rss-forwarder.service /etc/systemd/system
```

```BASH
sudo systemctl daemon-reload
sudo systemctl enable rss-forwarder
sudo systemctl start rss-forwarder
```
