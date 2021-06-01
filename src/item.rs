use blake3::{Hash, Hasher};
use chrono::{DateTime, FixedOffset};

pub trait ExtItem {
    fn pub_date_as_datetime(&self) -> Option<DateTime<FixedOffset>>;

    fn compute_hash(&self) -> Hash;
}

impl ExtItem for rss::Item {
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
