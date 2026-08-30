use crate::cards::{Card, Rank};

use super::{rankings::LowHandRank, PokerType, PokerVariant};

/// The best badugi in a holding.
///
/// A badugi is a set of cards with no two sharing a rank and no two sharing a
/// suit. The largest such set plays, so a four-card badugi beats any
/// three-card one however low; within a size the cards are compared from the
/// top down and lower wins, with the ace low. The best hand is a rainbow
/// A-2-3-4.
///
/// With four cards there are fifteen subsets to consider, so they are all
/// tried rather than reasoned about.
fn best_badugi(cards: &[Card]) -> LowHandRank {
    let mut best: Option<LowHandRank> = None;

    for subset in 1..(1u32 << cards.len()) {
        let chosen: Vec<Card> = cards
            .iter()
            .enumerate()
            .filter(|(index, _)| subset & (1 << index) != 0)
            .map(|(_, card)| *card)
            .collect();

        if !all_distinct(&chosen) {
            continue;
        }

        let rank = LowHandRank::from_played(chosen.iter().map(|card| card.rank()).collect());
        if best.as_ref().is_none_or(|found| rank > *found) {
            best = Some(rank);
        }
    }

    best.unwrap_or_else(|| LowHandRank::Low(Vec::new()))
}

/// The same best badugi as [`best_badugi`], as the number that orders it.
///
/// This is the one the sampling loop calls, so it never builds a hand: a
/// subset is two bitmasks, one of the ranks it uses and one of the suits, and
/// a repeat in either is what disqualifies it. The rank mask that proves the
/// subset legal then spells out its cards from the top down for free, so
/// there is nothing to sort and nothing to allocate.
///
/// Lower is better, as everywhere else a score is compared. The size leads,
/// since a bigger badugi beats any smaller one however low, and the cards
/// follow highest first.
fn badugi_key(cards: &[Card]) -> u32 {
    let mut best = u32::MAX;

    for subset in 1..(1u32 << cards.len()) {
        let mut ranks: u16 = 0;
        let mut suits: u8 = 0;
        let mut size = 0u32;
        let mut rainbow = true;

        for (index, card) in cards.iter().enumerate() {
            if subset & (1 << index) == 0 {
                continue;
            }
            // The ace plays low, so it sorts below the deuce.
            let value = if card.rank() == Rank::Ace {
                1
            } else {
                card.rank().to_value()
            };
            // A card's index is `rank * 4 + suit`.
            let suit = 1u8 << (card.index() % 4);
            if ranks & (1 << value) != 0 || suits & suit != 0 {
                rainbow = false;
                break;
            }
            ranks |= 1 << value;
            suits |= suit;
            size += 1;
        }

        if !rainbow {
            continue;
        }

        let mut key = 4 - size.min(4);
        let mut rest = ranks;
        for _ in 0..4 {
            // The highest card left, or nothing when the badugi is short --
            // and a short one has already lost on size, so what pads it out
            // cannot change the order.
            let value = if rest == 0 {
                0
            } else {
                let top = 15 - rest.leading_zeros();
                rest &= !(1 << top);
                top
            };
            key = (key << 4) | value;
        }

        best = best.min(key);
    }

    best
}

/// Whether no two cards share a rank and no two share a suit.
fn all_distinct(cards: &[Card]) -> bool {
    for (index, card) in cards.iter().enumerate() {
        for other in &cards[index + 1..] {
            if card.rank() == other.rank() || card.suit() == other.suit() {
                return false;
            }
        }
    }
    true
}

/// Four cards, drawing for the lowest set of cards with no repeated rank and
/// no repeated suit.
#[derive(Debug, Clone, Copy)]
pub struct Badugi;

impl PokerVariant for Badugi {
    type HandRank = LowHandRank;

    fn poker_type(&self) -> PokerType {
        PokerType::Draw
    }

    fn max_cards(&self) -> usize {
        4
    }

    fn hole_cards(&self) -> usize {
        4
    }

    fn evaluate_hand(&self, cards: &[Card]) -> Self::HandRank {
        best_badugi(cards)
    }

    /// Badugi has no lookup table -- it needs suits, so a rank key will not
    /// do -- so the ordering is packed into a number instead. Fewer cards is
    /// worse, then a higher card is worse.
    fn score(&self, cards: &[Card]) -> u32 {
        badugi_key(cards)
    }

    fn to_string(&self) -> String {
        "Badugi (single draw)".to_string()
    }

    fn key(&self) -> &'static str {
        "badugi"
    }
}
