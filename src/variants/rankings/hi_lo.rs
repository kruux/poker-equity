use std::cmp::Ordering;

use super::{HighHandRank, LowHandRank};
use crate::variants::HasLow;

#[derive(Debug, PartialEq)]
pub struct HiLoHandRank {
    pub high: HighHandRank,
    pub low: Option<LowHandRank>,
}

impl HiLoHandRank {
    pub fn new(high: HighHandRank, low: Option<LowHandRank>) -> Self {
        Self { high, low }
    }

    pub fn high(&self) -> &HighHandRank {
        &self.high
    }
}

impl HasLow for HiLoHandRank {
    fn low(&self) -> Option<&LowHandRank> {
        self.low.as_ref()
    }
}

impl PartialOrd for HiLoHandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.high.partial_cmp(&other.high)
    }
}
