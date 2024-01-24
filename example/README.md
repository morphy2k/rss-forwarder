# Examples

## Send items as email

The following simple example shows how you can send feed items as an email.

### Configure mail script

Configure and save the following content as `send.sh`

Check out the [cURL docs](https://everything.curl.dev/usingcurl/smtp)

```BASH
#!/bin/bash

set -e
set -o pipefail

# SMTP config
URL=smtps://mail.example.com

## For production use .netrc (https://everything.curl.dev/usingcurl/netrc)
USER=username@example.com
PASSWORD=password

FROM=$USER
TO=receiver@example.com

while read -r LINE; do
   SUBJECT=$(echo ${LINE} | jq '.title')
   LINK=$(echo ${LINE} | jq '.link')
   BODY=$(echo ${LINE} | jq 'if .content != null then .content else .description end')
   echo "Subject: $SUBJECT\n\n$BODY\n\n$LINK" | curl --ssl-reqd \
    --silent \
    --url $URL \
    --user "$USER:$PASSWORD" \
    --mail-from $FROM \
    --mail-rcpt $TO \
    -T -
done

exit 0
```

### Add the script to your sink

```TOML
[feeds.FEED_NAME.sink]
type = "custom"
command = "bash"
arguments = ["-C", "./send.sh"]
```

## Set up an systemd service

### Install binary

```BASH
export VERSION=<VERSION>
export TARGET=<TARGET>

curl -O https://github.com/morphy2k/rss-forwarder/releases/download/v${VERSION}/${TARGET}.tar.gz
tar -xzf ${TARGET}.tar.gz
cd ${TARGET}
sha256sum -c rss-forwarder.sha256
sudo chmod +x rss-forwarder
sudo mv rss-forwarder /usr/local/bin/rss-forwarder
```

*Replace `<VERSION>` and `<TARGET>` with the version and target of your choice.*

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
