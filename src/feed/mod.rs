pub mod item;

use crate::error::FeedError;

use self::item::{FeedItem, Item, Source};

use std::{cmp::Reverse, io::BufRead};

use tracing::debug;

#[derive(Debug)]
pub enum Feed {
    Rss(rss::Channel),
    Atom(atom_syndication::Feed),
}

impl<'a> Feed {
    pub fn read_from<R>(reader: R) -> Result<Self, FeedError>
    where
        R: BufRead + Copy,
    {
        let feed = match rss::Channel::read_from(reader) {
            Ok(channel) => Self::Rss(channel),
            Err(e) => match e {
                rss::Error::InvalidStartTag => {
                    let feed = atom_syndication::Feed::read_from(reader)?;
                    Self::Atom(feed)
                }
                _ => return Err(e)?,
            },
        };

        debug!(
            format = %if feed.is_rss() { "RSS" } else { "Atom" },
            items = feed.items().len(),
            "parsed feed"
        );

        Ok(feed)
    }

    pub fn title(&self) -> &str {
        match self {
            Feed::Rss(c) => &c.title,
            Feed::Atom(f) => &f.title.value,
        }
    }

    pub fn link(&self) -> Option<&str> {
        match self {
            Feed::Rss(c) => Some(c.link()),
            Feed::Atom(f) => f
                .links()
                .iter()
                .find(|s| s.rel() == "alternate")
                .map(|s| s.href()),
        }
    }

    pub fn items(&'a self) -> Vec<Item<'a>> {
        let source: Source<'a> = Source {
            title: self.title(),
            url: self.link(),
        };

        let mut items: Vec<Item<'a>> = match self {
            Feed::Rss(c) => c
                .items()
                .iter()
                .map(|v| Item::Rss {
                    source: source.clone(),
                    item: v,
                })
                .collect(),
            Feed::Atom(f) => f
                .entries()
                .iter()
                .map(|v| Item::Atom {
                    source: source.clone(),
                    entry: v,
                })
                .collect(),
        };

        items.sort_unstable_by_key(|v| Reverse(v.date()));

        items
    }

    /// Returns `true` if the feed is [`Rss`].
    ///
    /// [`Rss`]: Feed::Rss
    #[must_use]
    pub fn is_rss(&self) -> bool {
        matches!(self, Self::Rss(..))
    }

    /// Returns `true` if the feed is [`Atom`].
    ///
    /// [`Atom`]: Feed::Atom
    #[must_use]
    pub fn is_atom(&self) -> bool {
        matches!(self, Self::Atom(..))
    }
}
