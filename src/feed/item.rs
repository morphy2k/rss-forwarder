use atom_syndication::TextType;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;

pub trait FeedItem<'a>: Sync {
    fn title(&'a self) -> Option<&str>;

    fn title_as_text(&'a self) -> Result<Option<String>, html2text::Error>;

    fn description(&'a self) -> Option<&str>;

    fn description_as_text(&'a self) -> Result<Option<String>, html2text::Error>;

    fn content(&'a self) -> Option<&str>;

    fn content_as_text(&'a self) -> Result<Option<String>, html2text::Error>;

    fn link(&'a self) -> Option<&str>;

    fn date(&'a self) -> DateTime<FixedOffset>;

    fn authors(&'a self) -> Vec<Author>;

    /// Feed metadata
    fn source(&'a self) -> Option<&Source>;
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
    #[inline]
    fn title(&self) -> Option<&str> {
        self.title()
    }

    #[inline]
    fn title_as_text(&self) -> Result<Option<String>, html2text::Error> {
        self.title()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
            .transpose()
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        self.description()
    }

    #[inline]
    fn description_as_text(&self) -> Result<Option<String>, html2text::Error> {
        self.description()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
            .transpose()
    }

    #[inline]
    fn content(&self) -> Option<&str> {
        self.content()
    }

    #[inline]
    fn content_as_text(&self) -> Result<Option<String>, html2text::Error> {
        self.content()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
            .transpose()
    }

    #[inline]
    fn link(&self) -> Option<&str> {
        self.link()
    }

    #[inline]
    fn date(&self) -> DateTime<FixedOffset> {
        DateTime::parse_from_rfc2822(self.pub_date().expect("missing pub date")).unwrap()
    }

    #[inline]
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

    fn source(&'a self) -> Option<&Source> {
        None
    }
}

impl<'a> FeedItem<'a> for atom_syndication::Entry {
    #[inline]
    fn title(&self) -> Option<&str> {
        Some(self.title())
    }

    #[inline]
    fn title_as_text(&self) -> Result<Option<String>, html2text::Error> {
        if self.title().r#type == TextType::Html {
            html2text::from_read(self.title().as_bytes(), usize::MAX).map(Some)
        } else {
            Ok(Some(self.title().to_string()))
        }
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        match self.summary() {
            Some(v) => Some(&v.value),
            None => None,
        }
    }

    #[inline]
    fn description_as_text(&self) -> Result<Option<String>, html2text::Error> {
        match self.summary() {
            Some(v) if v.r#type == TextType::Html => {
                html2text::from_read(v.as_bytes(), usize::MAX).map(Some)
            }
            Some(v) => Ok(Some(v.value.clone())),
            None => Ok(None),
        }
    }

    #[inline]
    fn content(&self) -> Option<&str> {
        match self.content() {
            Some(v) => v.value.as_deref(),
            None => None,
        }
    }

    #[inline]
    fn content_as_text(&self) -> Result<Option<String>, html2text::Error> {
        match self.content().and_then(|v| v.value()) {
            Some(v) if self.content().unwrap().content_type() == Some("html") => {
                html2text::from_read(v.as_bytes(), usize::MAX).map(Some)
            }
            Some(v) => Ok(Some(v.to_string())),
            None => Ok(None),
        }
    }

    #[inline]
    fn link(&self) -> Option<&str> {
        self.links()
            .iter()
            .find(|s| s.rel() == "alternate")
            .map(|s| s.href())
    }

    fn date(&self) -> DateTime<FixedOffset> {
        self.updated
    }

    #[inline]
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

    fn source(&'a self) -> Option<&Source> {
        None
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Author<'a> {
    pub name: &'a str,
    pub email: Option<&'a str>,
    pub uri: Option<&'a str>,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct Source<'a> {
    pub title: &'a str,
    pub url: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub enum Item<'a> {
    Rss {
        source: Source<'a>,
        item: &'a rss::Item,
    },
    Atom {
        source: Source<'a>,
        entry: &'a atom_syndication::Entry,
    },
}

impl<'a> FeedItem<'a> for Item<'a> {
    #[inline]
    fn title(&self) -> Option<&str> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::title(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::title(entry),
        }
    }

    #[inline]
    fn title_as_text(&self) -> Result<Option<String>, html2text::Error> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::title_as_text(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::title_as_text(entry),
        }
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::description(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::description(entry),
        }
    }

    #[inline]
    fn description_as_text(&self) -> Result<Option<String>, html2text::Error> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::description_as_text(item),
            Item::Atom { entry, .. } => {
                <atom_syndication::Entry as FeedItem>::description_as_text(entry)
            }
        }
    }

    #[inline]
    fn content(&self) -> Option<&str> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::content(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::content(entry),
        }
    }

    #[inline]
    fn content_as_text(&self) -> Result<Option<String>, html2text::Error> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::content_as_text(item),
            Item::Atom { entry, .. } => {
                <atom_syndication::Entry as FeedItem>::content_as_text(entry)
            }
        }
    }

    #[inline]
    fn link(&self) -> Option<&str> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::link(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::link(entry),
        }
    }

    #[inline]
    fn date(&self) -> DateTime<FixedOffset> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::date(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::date(entry),
        }
    }

    #[inline]
    fn authors(&self) -> Vec<Author> {
        match self {
            Item::Rss { item, .. } => <rss::Item as FeedItem>::authors(item),
            Item::Atom { entry, .. } => <atom_syndication::Entry as FeedItem>::authors(entry),
        }
    }

    #[inline]
    fn source(&'a self) -> Option<&Source> {
        match self {
            Item::Rss { source, .. } => Some(source),
            Item::Atom { source, .. } => Some(source),
        }
    }
}
