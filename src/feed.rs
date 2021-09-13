use crate::error::FeedError;

use std::{cmp::Reverse, io::BufRead};

use atom_syndication::TextType;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;

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

pub trait FeedItem<'a>: Sync {
    fn title(&'a self) -> Option<&str>;

    fn title_as_text(&'a self) -> Option<String>;

    fn description(&'a self) -> Option<&str>;

    fn description_as_text(&'a self) -> Option<String>;

    fn content(&'a self) -> Option<&str>;

    fn content_as_text(&'a self) -> Option<String>;

    fn link(&'a self) -> Option<&str>;

    fn date(&'a self) -> DateTime<FixedOffset>;

    fn authors(&'a self) -> Vec<Author>;
}

pub trait TryFromItem<'a, T>
where
    T: FeedItem<'a>,
    Self: Sized,
{
    type Error: std::error::Error;

    fn try_from_item(value: &'a T) -> std::result::Result<Self, Self::Error>;
}

impl<'a> FeedItem<'a> for rss::Item {
    fn title(&self) -> Option<&str> {
        self.title()
    }

    fn title_as_text(&self) -> Option<String> {
        self.title()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn description(&self) -> Option<&str> {
        self.description()
    }

    fn description_as_text(&self) -> Option<String> {
        self.description
            .as_ref()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn content(&self) -> Option<&str> {
        self.content()
    }

    fn content_as_text(&self) -> Option<String> {
        self.content
            .as_ref()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn link(&self) -> Option<&str> {
        self.link()
    }

    fn date(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc2822(self.pub_date().expect("missing pub date")).unwrap()
    }

    fn authors(&self) -> Vec<Author> {
        match self.author() {
            Some(v) => vec![Author {
                name: v,
                email: None,
                uri: None,
            }],
            None => Vec::default(),
        }
    }
}

impl<'a> FeedItem<'a> for atom_syndication::Entry {
    fn title(&self) -> Option<&str> {
        Some(self.title())
    }

    fn title_as_text(&self) -> Option<String> {
        if self.title().r#type == TextType::Html {
            html2text::from_read(self.title().value.as_bytes(), usize::MAX).into()
        } else {
            Some(self.title().value.to_owned())
        }
    }

    fn description(&self) -> Option<&str> {
        match self.summary() {
            Some(v) => Some(&v.value),
            None => None,
        }
    }

    fn description_as_text(&self) -> Option<String> {
        if let Some(v) = self.summary() {
            if v.r#type == TextType::Html {
                html2text::from_read(v.value.as_bytes(), usize::MAX).into()
            } else {
                Some(v.value.to_owned())
            }
        } else {
            None
        }
    }

    fn content(&self) -> Option<&str> {
        match self.content() {
            Some(v) => v.value.as_deref(),
            None => None,
        }
    }

    fn content_as_text(&self) -> Option<String> {
        if let Some(v) = self.content() {
            if v.content_type() == Some("html") {
                v.value
                    .as_ref()
                    .map(|v| html2text::from_read(v.as_bytes(), usize::MAX))
            } else {
                v.value.to_owned()
            }
        } else {
            None
        }
    }

    fn link(&self) -> Option<&str> {
        self.links()
            .iter()
            .find(|s| s.rel() == "alternate")
            .map(|s| s.href())
    }

    fn date(&self) -> DateTime<FixedOffset> {
        self.updated
    }

    fn authors(&self) -> Vec<Author> {
        self.authors()
            .iter()
            .map(|v| Author {
                name: v.name(),
                email: v.email(),
                uri: v.uri(),
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
pub struct Author<'a> {
    pub name: &'a str,
    pub email: Option<&'a str>,
    pub uri: Option<&'a str>,
}

pub enum Item<'a> {
    Rss(&'a rss::Item),
    Atom(&'a atom_syndication::Entry),
}

impl<'a> FeedItem<'a> for Item<'a> {
    fn title(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::title(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::title(e),
        }
    }

    fn title_as_text(&self) -> Option<String> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::title_as_text(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::title_as_text(e),
        }
    }

    fn description(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::description(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::description(e),
        }
    }

    fn description_as_text(&self) -> Option<String> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::description_as_text(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::description_as_text(e),
        }
    }

    fn content(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::content(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::content(e),
        }
    }

    fn content_as_text(&self) -> Option<String> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::content_as_text(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::content_as_text(e),
        }
    }

    fn link(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::link(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::link(e),
        }
    }

    fn date(&self) -> DateTime<FixedOffset> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::date(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::date(e),
        }
    }

    fn authors(&self) -> Vec<Author> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::authors(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::authors(e),
        }
    }
}
