use std::fmt;

use super::{Card, Rank, Suit};

/// A set of cards, held as one bit per card.
///
/// Bit *n* is set when the card with index *n* is in the set, so the whole
/// deck fits in a `u64` with twelve bits to spare. This is the type the
/// engine speaks: removing every known card from the deck is one `AND`,
/// counting what is left is one instruction, and a constraint like "any club"
/// is a constant.
///
/// It is a set, not a sequence -- there is no order and no duplicates. Where
/// the old `Vec<Card>` deck was shuffled and popped, a `CardSet` is drawn
/// from directly.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct CardSet(u64);

impl CardSet {
    /// Every card in a standard deck.
    pub const FULL_DECK: Self = Self((1 << 52) - 1);

    /// No cards at all.
    pub const EMPTY: Self = Self(0);

    /// The thirty-six card deck short-deck games use: sixes and up.
    pub const SHORT_DECK: Self = Self(Self::FULL_DECK.0 & !((1 << (4 * 4)) - 1));

    /// Builds a set from the raw bits, keeping only the fifty-two card bits.
    pub fn from_bits(bits: u64) -> Self {
        Self(bits & Self::FULL_DECK.0)
    }

    /// The raw bits, for the fpdb boundary.
    pub fn bits(self) -> u64 {
        self.0
    }

    /// Collects `cards` into a set. Repeats collapse, as in any set.
    pub fn from_cards(cards: &[Card]) -> Self {
        let mut set = Self::EMPTY;
        for &card in cards {
            set.insert(card);
        }
        set
    }

    /// Every card of one rank, the mask behind a pattern like `A`.
    pub fn of_rank(rank: Rank) -> Self {
        Self(0b1111 << ((rank.to_value() - 2) * 4))
    }

    /// Every card of one suit, the mask behind a pattern like `c`.
    pub fn of_suit(suit: Suit) -> Self {
        // One bit every four, thirteen times, shifted to the suit.
        Self(0x1_1111_1111_1111 << (suit as u8))
    }

    /// Whether `card` is in the set.
    pub fn contains(self, card: Card) -> bool {
        self.0 & (1 << card.index()) != 0
    }

    /// Adds `card`. Adding a card already present changes nothing.
    pub fn insert(&mut self, card: Card) {
        self.0 |= 1 << card.index();
    }

    /// Removes `card`. Removing a card that is absent changes nothing.
    pub fn remove(&mut self, card: Card) {
        self.0 &= !(1 << card.index());
    }

    /// The cards in `self` that are not in `other`. One instruction.
    pub fn without(self, other: Self) -> Self {
        Self(self.0 & !other.0)
    }

    /// The cards in either set.
    pub fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// The cards in both sets.
    pub fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    /// Whether the two sets share no card.
    pub fn is_disjoint(self, other: Self) -> bool {
        self.0 & other.0 == 0
    }

    /// How many cards are in the set.
    pub fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Whether the set holds no cards.
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The `index`th card in the set, counting from the lowest, or `None`
    /// when the set holds fewer than that.
    ///
    /// This is how a card is drawn: pick a number below [`len`](Self::len)
    /// and take that card, which is uniform over the set without needing the
    /// set to be shuffled.
    pub fn nth(self, index: u32) -> Option<Card> {
        if index >= self.len() {
            return None;
        }
        let mut bits = self.0;
        for _ in 0..index {
            bits &= bits - 1; // clear the lowest set bit
        }
        Card::from_index(bits.trailing_zeros() as u8)
    }

    /// The cards in the set, lowest index first.
    pub fn iter(self) -> impl Iterator<Item = Card> {
        let mut bits = self.0;
        std::iter::from_fn(move || {
            if bits == 0 {
                return None;
            }
            let index = bits.trailing_zeros() as u8;
            bits &= bits - 1;
            Card::from_index(index)
        })
    }
}

impl FromIterator<Card> for CardSet {
    fn from_iter<I: IntoIterator<Item = Card>>(cards: I) -> Self {
        let mut set = Self::EMPTY;
        for card in cards {
            set.insert(card);
        }
        set
    }
}

impl fmt::Display for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut first = true;
        for card in self.iter() {
            if !first {
                write!(f, " ")?;
            }
            write!(f, "{}", card)?;
            first = false;
        }
        Ok(())
    }
}
