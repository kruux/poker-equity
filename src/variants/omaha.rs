use std::cmp::{min, Ordering};

use itertools::Itertools;

use crate::cards::Card;

use super::{rankings::OmahaHandRank, PokerType, PokerVariant};

/// Scores an Omaha holding: exactly two hole cards with exactly three board
/// cards, whichever pairing is best.
///
/// That rule is the whole game, and it can only ever cost a player -- a hand
/// that would be a flush in hold'em is not one here unless two of its suited
/// cards are in the hand. With `n` hole cards it is `C(n,2) * C(5,3)`
/// five-card evaluations: 60 at four cards, 100 at five, 150 at six. That
/// dominates the cost of an Omaha deal and is why the variant runs an order
/// of magnitude behind hold'em.
fn best_hand(hole_count: usize, cards: &[Card]) -> OmahaHandRank {
    let split = cards.len().min(hole_count);
    let (hole_cards, board_cards) = cards.split_at(split);

    hole_cards
        .iter()
        .combinations(2)
        .flat_map(|hole| {
            board_cards.iter().combinations(3).map(move |board| {
                let five: Vec<Card> = hole.iter().chain(board.iter()).map(|&card| *card).collect();
                OmahaHandRank::evaluate(&five)
            })
        })
        .max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
        .unwrap_or_else(|| {
            // Too few cards to make five. Report how many the best hand would
            // have held, so incomplete hands still order among themselves.
            OmahaHandRank::Incomplete(min(hole_cards.len(), 2) + min(board_cards.len(), 3))
        })
}

/// Defines an Omaha variant, which differ only in how many cards a player
/// holds. The two-from-hand rule and the five-card board are the same
/// throughout.
macro_rules! omaha_variant {
    ($name:ident, $hole:expr, $label:expr, $key:expr, $doc:expr) => {
        #[doc = $doc]
        #[derive(Debug, Clone, Copy)]
        pub struct $name;

        impl PokerVariant for $name {
            type HandRank = OmahaHandRank;

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
                best_hand($hole, cards)
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

omaha_variant!(
    Omaha,
    4,
    "Omaha",
    "omahahi",
    "Four hole cards, two of which play."
);
omaha_variant!(
    OmahaFive,
    5,
    "5-Card Omaha",
    "5_omahahi",
    "Five hole cards, two of which play."
);
omaha_variant!(
    OmahaSix,
    6,
    "6-Card Omaha",
    "6_omahahi",
    "Six hole cards, two of which play."
);
omaha_variant!(
    Courchevel,
    5,
    "Courchevel",
    "cour_hi",
    "Five-card Omaha where the first board card is dealt face up before the\n\
     betting. Not a variant of its own: the same game, plus a rule that at\n\
     least one board card must be known, which lives in its equity validation."
);
