use atom_syndication::TextType;
use chrono::{DateTime, FixedOffset};
use serde::Serialize;

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
    #[inline]
    fn title(&self) -> Option<&str> {
        self.title()
    }

    #[inline]
    fn title_as_text(&self) -> Option<String> {
        self.title()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        self.description()
    }

    #[inline]
    fn description_as_text(&self) -> Option<String> {
        self.description
            .as_ref()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    #[inline]
    fn content(&self) -> Option<&str> {
        self.content()
    }

    #[inline]
    fn content_as_text(&self) -> Option<String> {
        self.content
            .as_ref()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
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
}

impl<'a> FeedItem<'a> for atom_syndication::Entry {
    #[inline]
    fn title(&self) -> Option<&str> {
        Some(self.title())
    }

    #[inline]
    fn title_as_text(&self) -> Option<String> {
        if self.title().r#type == TextType::Html {
            html2text::from_read(self.title().value.as_bytes(), usize::MAX).into()
        } else {
            Some(self.title().value.to_owned())
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

    #[inline]
    fn content(&self) -> Option<&str> {
        match self.content() {
            Some(v) => v.value.as_deref(),
            None => None,
        }
    }

    #[inline]
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
    #[inline]
    fn title(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::title(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::title(e),
        }
    }

    #[inline]
    fn title_as_text(&self) -> Option<String> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::title_as_text(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::title_as_text(e),
        }
    }

    #[inline]
    fn description(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::description(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::description(e),
        }
    }

    #[inline]
    fn description_as_text(&self) -> Option<String> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::description_as_text(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::description_as_text(e),
        }
    }

    #[inline]
    fn content(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::content(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::content(e),
        }
    }

    #[inline]
    fn content_as_text(&self) -> Option<String> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::content_as_text(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::content_as_text(e),
        }
    }

    #[inline]
    fn link(&self) -> Option<&str> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::link(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::link(e),
        }
    }

    #[inline]
    fn date(&self) -> DateTime<FixedOffset> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::date(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::date(e),
        }
    }

    #[inline]
    fn authors(&self) -> Vec<Author> {
        match self {
            Item::Rss(i) => <rss::Item as FeedItem>::authors(i),
            Item::Atom(e) => <atom_syndication::Entry as FeedItem>::authors(e),
        }
    }
}
