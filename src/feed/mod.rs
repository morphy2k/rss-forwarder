pub mod item;

use crate::error::FeedError;

use self::item::{FeedItem, Item};

use std::{cmp::Reverse, io::BufRead};

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
                _ => return Err(e.into()),
            },
        };

        Ok(feed)
    }

    pub fn title(&self) -> &str {
        match self {
            Feed::Rss(c) => &c.title,
            Feed::Atom(f) => &f.title.value,
        }
    }

    pub fn items(&'a self) -> Vec<Item<'a>> {
        let mut items: Vec<Item<'a>> = match self {
            Feed::Rss(c) => c.items().iter().map(|v| Item::Rss(v)).collect(),
            Feed::Atom(f) => f.entries().iter().map(|v| Item::Atom(v)).collect(),
        };

        items.sort_unstable_by_key(|v| Reverse(v.date()));

        items
    }
}
