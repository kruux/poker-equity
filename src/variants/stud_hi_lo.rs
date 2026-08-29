use super::{HighHandRank, LowHandRank, StudHiLoHandRank};
use crate::{
    cards::Card,
    variants::{PokerType, PokerVariant},
};

#[derive(Debug, Clone, Copy)]
pub struct StudHiLo;
impl StudHiLo {
    /// Returns the low hand if it qualifies under the eight-or-better rule,
    /// and `None` otherwise.
    fn qualify_for_low(&self, low_rank: &LowHandRank) -> Option<LowHandRank> {
        low_rank.is_eight_or_better().then(|| low_rank.clone())
    }
}

impl PokerVariant for StudHiLo {
    type HandRank = StudHiLoHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Stud
    }

    fn max_cards(&self) -> usize {
        7
    }

    fn hole_cards(&self) -> usize {
        7
    }

    fn to_string(&self) -> String {
        "Stud Hi/Lo".to_string()
    }

    fn key(&self) -> &'static str {
        "stud_hi_lo"
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        let high = HighHandRank::evaluate(cards);
        let low = self.qualify_for_low(&LowHandRank::evaluate(cards));

        StudHiLoHandRank { high, low }
    }
}
