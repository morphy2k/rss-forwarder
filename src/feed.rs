use crate::error::FeedError;

use std::{convert::TryFrom, io::BufRead};

use blake3::{Hash, Hasher};
use chrono::{DateTime, FixedOffset};
use serde::Serialize;

#[derive(Debug)]
pub enum Feed {
    Rss(rss::Channel),
    Atom(atom_syndication::Feed),
}

impl Feed {
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

    pub fn items(&self) -> Result<Vec<Item>, FeedError> {
        let result: Result<Vec<Item>, FeedError> = match self {
            Feed::Rss(c) => c.items().iter().map(Item::try_from).collect(),
            Feed::Atom(f) => f.entries().iter().map(Item::try_from).collect(),
        };

        let mut items = result?;
        items.sort_unstable_by(|a, b| b.date.cmp(&a.date));

        Ok(items)
    }
}

#[derive(Debug, Serialize)]
pub struct Item {
    pub title: String,
    pub description: Option<String>,
    pub content: Option<String>,
    pub links: Vec<String>,
    pub date: DateTime<FixedOffset>,
    pub authors: Vec<Author>,
}

impl TryFrom<&rss::Item> for Item {
    type Error = FeedError;

    fn try_from(value: &rss::Item) -> Result<Self, Self::Error> {
        let item = Self {
            title: value
                .title
                .to_owned()
                .ok_or_else(|| FeedError::Item("title is missing".to_string()))?,
            description: value.description.to_owned(),
            content: value.content.to_owned(),
            links: value.link.to_owned().map(|s| vec![s]).unwrap_or_default(),
            date: match value.pub_date() {
                Some(v) => DateTime::parse_from_rfc2822(v).unwrap(),
                None => return Err(FeedError::Item("rss pube date is missing".to_string())),
            },
            authors: vec![Author::try_from(value)?],
        };

        Ok(item)
    }
}

impl TryFrom<&atom_syndication::Entry> for Item {
    type Error = FeedError;

    fn try_from(value: &atom_syndication::Entry) -> Result<Self, Self::Error> {
        let authors = value
            .authors()
            .iter()
            .map(Author::try_from)
            .collect::<Result<Vec<Author>, Self::Error>>()?;

        let item = Self {
            title: value.title.value.to_owned(),
            description: value.summary.to_owned().map(|s| s.value),
            content: match value.content.to_owned() {
                Some(s) => s.value,
                None => None,
            },
            links: value.links().iter().map(|v| v.href.to_owned()).collect(),
            date: value.updated,
            authors,
        };

        Ok(item)
    }
}

#[derive(Debug, Serialize)]
pub struct Author {
    pub name: String,
    pub email: Option<String>,
    pub uri: Option<String>,
}

impl TryFrom<&rss::Item> for Author {
    type Error = FeedError;

    fn try_from(value: &rss::Item) -> Result<Self, Self::Error> {
        let author = Self {
            name: value.author.to_owned().unwrap_or_default(),
            email: value.author.to_owned(),
            uri: None,
        };

        Ok(author)
    }
}

impl TryFrom<&atom_syndication::Person> for Author {
    type Error = FeedError;

    fn try_from(value: &atom_syndication::Person) -> Result<Self, Self::Error> {
        let author = Self {
            name: value.name.to_owned(),
            email: value.email.to_owned(),
            uri: value.uri.to_owned(),
        };

        Ok(author)
    }
}

pub trait ExtItem {
    fn title_as_text(&self) -> String;

    fn description_as_text(&self) -> Option<String>;

    fn content_as_text(&self) -> Option<String>;

    fn compute_hash(&self) -> Hash;
}

impl ExtItem for Item {
    fn title_as_text(&self) -> String {
        html2text::from_read(self.title.as_bytes(), usize::MAX)
    }

    fn description_as_text(&self) -> Option<String> {
        self.description
            .as_ref()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn content_as_text(&self) -> Option<String> {
        self.content
            .as_ref()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn compute_hash(&self) -> Hash {
        let mut hasher = Hasher::new();

        hasher.update(self.title.as_ref());
        if let Some(v) = self.links.first() {
            hasher.update(v.as_ref());
        }

        hasher.finalize()
    }
}
