use std::cmp::Ordering;

use super::{HighHandRank, LowHandRank};
use crate::variants::HasLow;

/// A split-pot hand: what it is worth for high, and for low if it qualifies.
///
/// Ordering compares only the high half, since that is what decides the high
/// half of the pot; the low is reached through [`HasLow`](crate::variants::HasLow).
#[derive(Debug, PartialEq)]
pub struct HiLoHandRank {
    pub high: HighHandRank,
    pub low: Option<LowHandRank>,
}

impl HiLoHandRank {
    /// Pairs a high hand with a qualifying low, if there is one.
    pub fn new(high: HighHandRank, low: Option<LowHandRank>) -> Self {
        Self { high, low }
    }

    /// The high half.
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
