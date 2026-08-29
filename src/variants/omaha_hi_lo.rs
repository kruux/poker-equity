use itertools::Itertools;

use crate::cards::Card;

use super::{
    rankings::{HighHandRank, HiLoHandRank, LowHandRank},
    PokerType, PokerVariant,
};

/// Scores a split-pot Omaha holding.
///
/// The two-from-hand rule applies to each half separately, and the two halves
/// need not use the same cards: a player can make a straight with one pair of
/// hole cards and a wheel low with another. So both halves are searched over
/// the same combinations rather than one being derived from the other.
fn best_hi_lo(hole_count: usize, cards: &[Card]) -> HiLoHandRank {
    let split = cards.len().min(hole_count);
    let (hole_cards, board_cards) = cards.split_at(split);

    let mut best_high: Option<HighHandRank> = None;
    let mut best_low: Option<LowHandRank> = None;

    for hole in hole_cards.iter().combinations(2) {
        for board in board_cards.iter().combinations(3) {
            let five: Vec<Card> = hole.iter().chain(board.iter()).map(|&card| *card).collect();

            let high = HighHandRank::evaluate(&five);
            if best_high.as_ref().is_none_or(|best| high > *best) {
                best_high = Some(high);
            }

            // Only a qualifying low counts, and a qualifying low always has
            // five distinct ranks, so a paired low never reaches here.
            let low = LowHandRank::evaluate(&five);
            if low.is_eight_or_better() && best_low.as_ref().is_none_or(|best| low > *best) {
                best_low = Some(low);
            }
        }
    }

    HiLoHandRank {
        high: best_high.unwrap_or(HighHandRank::Incomplete(
            hole_cards.len().min(2) + board_cards.len().min(3),
        )),
        low: best_low,
    }
}

/// Defines a split-pot Omaha variant. They differ only in how many cards a
/// player holds.
macro_rules! omaha_hi_lo_variant {
    ($name:ident, $hole:expr, $label:expr, $key:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl PokerVariant for $name {
            type HandRank = HiLoHandRank;

            fn poker_type(&self) -> PokerType {
                PokerType::Community
            }

            fn max_cards(&self) -> usize {
                $hole + 5
            }

            fn hole_cards(&self) -> usize {
                $hole
            }

            fn board_cards(&self) -> usize {
                5
            }

            fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
                best_hi_lo($hole, cards)
            }

            fn to_string(&self) -> String {
                $label.to_string()
            }

            fn key(&self) -> &'static str {
                $key
            }
        }
    };
}

omaha_hi_lo_variant!(
    OmahaHiLo,
    4,
    "Omaha Hi/Lo",
    "omaha_hi_lo",
    "Four hole cards, split between the best high hand and the best\n\
     eight-or-better low."
);
omaha_hi_lo_variant!(
    OmahaFiveHiLo,
    5,
    "5-Card Omaha Hi/Lo",
    "omaha_five_hi_lo",
    "Five-card Omaha, split between high and an eight-or-better low."
);
omaha_hi_lo_variant!(
    CourchevelHiLo,
    5,
    "Courchevel Hi/Lo",
    "courchevel_hi_lo",
    "Five-card Omaha hi/lo with the first board card dealt face up before\n\
     the betting."
);
