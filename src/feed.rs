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

    pub fn items(&'a self) -> Vec<&'a dyn Item> {
        let mut items: Vec<&'a dyn Item> = match self {
            Feed::Rss(c) => c.items().iter().map(|v| v as &dyn Item).collect(),
            Feed::Atom(f) => f.entries().iter().map(|v| v as &dyn Item).collect(),
        };

        items.sort_unstable_by_key(|v| Reverse(v.date()));

        items
    }
}

pub trait Item: Sync {
    fn title(&self) -> Option<&str>;

    fn title_as_text(&self) -> Option<String>;

    fn description(&self) -> Option<&str>;

    fn description_as_text(&self) -> Option<String>;

    fn content(&self) -> Option<&str>;

    fn content_as_text(&self) -> Option<String>;

    fn link(&self) -> Option<&str>;

    fn date(&self) -> DateTime<FixedOffset>;

    fn authors(&self) -> Vec<Author>;
}

impl Item for rss::Item {
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

impl Item for atom_syndication::Entry {
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
