use super::{
    rankings::{high_score, low_a5_score, EIGHT_OR_BETTER_LIMIT},
    HighHandRank, LowHandRank, StudHiLoHandRank,
};
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

    fn score(&self, cards: &[Card]) -> u32 {
        high_score(cards) as u32
    }

    fn low_score(&self, cards: &[Card]) -> Option<u32> {
        let best = low_a5_score(cards) as u32;
        (best < EIGHT_OR_BETTER_LIMIT as u32).then_some(best)
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        let high = HighHandRank::evaluate(cards);
        let low = self.qualify_for_low(&LowHandRank::evaluate(cards));

        StudHiLoHandRank { high, low }
    }
}
