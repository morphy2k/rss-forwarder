use blake3::{Hash, Hasher};
use chrono::{DateTime, FixedOffset};

pub trait ExtItem {
    fn title_as_text(&self) -> Option<String>;

    fn description_as_text(&self) -> Option<String>;

    fn content_as_text(&self) -> Option<String>;

    fn pub_date_as_datetime(&self) -> Option<DateTime<FixedOffset>>;

    fn compute_hash(&self) -> Hash;
}

impl ExtItem for rss::Item {
    fn title_as_text(&self) -> Option<String> {
        self.title()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn description_as_text(&self) -> Option<String> {
        self.description()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn content_as_text(&self) -> Option<String> {
        self.content()
            .map(|s| html2text::from_read(s.as_bytes(), usize::MAX))
    }

    fn pub_date_as_datetime(&self) -> Option<DateTime<FixedOffset>> {
        self.pub_date()
            .map(|v| DateTime::parse_from_rfc2822(v).unwrap())
    }

    fn compute_hash(&self) -> Hash {
        let mut hasher = Hasher::new();

        hasher.update(self.title().unwrap().as_ref());
        hasher.update(self.link().unwrap().as_ref());

        hasher.finalize()
    }
}
