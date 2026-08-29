use std::cmp::{min, Ordering};

use itertools::Itertools;

use crate::cards::{Card, Suit};

use super::{
    rankings::{high_score_from_parts, rank_key, shared_suit, OmahaHandRank},
    PokerType, PokerVariant,
};

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

/// The best score among every pairing of two hole cards with three of the
/// board.
///
/// The same walk as [`best_hand`], but reading a lookup table instead of
/// naming each candidate, which is what the sampling loop wants. Lower is
/// better, so the best pairing is the smallest.
/// One half of a pairing, worked out once and reused.
///
/// A key is the base-five rank key of those cards, and `suited` is the suit
/// they share with their rank mask, when they share one.
#[derive(Clone, Copy)]
struct Part {
    key: u32,
    suited: Option<(Suit, u16)>,
}

impl Part {
    fn of(cards: &[Card]) -> Self {
        Self {
            key: rank_key(cards),
            suited: shared_suit(cards),
        }
    }
}

/// The best score among every pairing of two hole cards with three of the
/// board.
///
/// The same walk as [`best_hand`], but reading a lookup table instead of
/// naming each candidate. Lower is better, so the best pairing is the
/// smallest.
///
/// Each half of a pairing is worked out once. Rank keys add, so a pairing's
/// key is one addition rather than five cards walked again; and a five-card
/// flush needs all five of one suit, so it is exactly the case where both
/// halves are of the same single suit. That turns sixty five-card
/// evaluations into sixty additions and sixty lookups, over ten board parts
/// and six hole parts worked out beforehand -- and the board's parts are the
/// same for every seat at the table.
pub(super) fn best_score(
    hole_count: usize,
    cards: &[Card],
    from_parts: impl Fn(u32, Option<u16>) -> u16,
) -> u32 {
    let split = cards.len().min(hole_count);
    let (hole_cards, board_cards) = cards.split_at(split);
    if hole_cards.len() < 2 || board_cards.len() < 3 {
        return u32::MAX;
    }

    // At most C(5,3) board triples and C(6,2) hole pairs.
    let mut board_parts = [Part { key: 0, suited: None }; 10];
    let mut boards = 0;
    for a in 0..board_cards.len() {
        for b in (a + 1)..board_cards.len() {
            for c in (b + 1)..board_cards.len() {
                board_parts[boards] = Part::of(&[board_cards[a], board_cards[b], board_cards[c]]);
                boards += 1;
            }
        }
    }

    let mut hole_parts = [Part { key: 0, suited: None }; 15];
    let mut holes = 0;
    for first in 0..hole_cards.len() {
        for second in (first + 1)..hole_cards.len() {
            hole_parts[holes] = Part::of(&[hole_cards[first], hole_cards[second]]);
            holes += 1;
        }
    }

    let mut best = u32::MAX;
    for hole in &hole_parts[..holes] {
        for board in &board_parts[..boards] {
            // Five of one suit means both halves of one suit, and the same
            // one. Anything else goes to the rank table.
            let flush = match (hole.suited, board.suited) {
                (Some((hole_suit, hole_mask)), Some((board_suit, board_mask)))
                    if hole_suit == board_suit =>
                {
                    Some(hole_mask | board_mask)
                }
                _ => None,
            };
            best = best.min(from_parts(hole.key + board.key, flush) as u32);
        }
    }

    best
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

            fn score(&self, cards: &[Card]) -> u32 {
                best_score($hole, cards, high_score_from_parts)
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
    "omaha",
    "Four hole cards, two of which play."
);
omaha_variant!(
    OmahaFive,
    5,
    "5-Card Omaha",
    "omaha_five",
    "Five hole cards, two of which play."
);
omaha_variant!(
    OmahaSix,
    6,
    "6-Card Omaha",
    "omaha_six",
    "Six hole cards, two of which play."
);
omaha_variant!(
    Courchevel,
    5,
    "Courchevel",
    "courchevel",
    "Five-card Omaha where the first board card is dealt face up before the\n\
     betting. Not a variant of its own: the same game, plus a rule that at\n\
     least one board card must be known, which lives in its equity validation."
);
