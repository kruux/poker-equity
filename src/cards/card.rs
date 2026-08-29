use enum_iterator::Sequence;
use std::fmt;

use crate::error::CardError;

/// A single card, stored as an index in `0..52`.
///
/// The layout is `rank * 4 + suit`, with ranks `0..13` running `23456789TJQKA`
/// and suits `0..4` running `cdhs`. That is the encoding fpdb's Python side
/// already produces, so the two sides need no translation, and it is what lets
/// a set of cards be a single `u64` -- see [`CardSet`].
///
/// The index is an implementation detail everywhere except that boundary:
/// construct with [`Card::new`] and read with [`Card::rank`] and
/// [`Card::suit`], which cost an arithmetic operation each.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Card(u8);

impl Card {
    /// The number of cards in a full deck.
    pub const COUNT: u8 = 52;

    /// Builds a card from its suit and rank.
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self((rank.to_value() - 2) * 4 + suit as u8)
    }

    /// Builds a card from its index, or `None` if the index is not a card.
    ///
    /// Use this at the fpdb boundary, where indices arrive from outside.
    pub fn from_index(index: u8) -> Option<Self> {
        (index < Self::COUNT).then_some(Self(index))
    }

    /// This card's index in `0..52`, as `rank * 4 + suit`.
    pub fn index(self) -> u8 {
        self.0
    }

    /// Takes a string e.g. "AdKc" and returns a vec of Cards
    /// Turn into a vec of chars for easier handling
    /// Also remove any whitespace so you can call both "AhKh" and "Ah Kh"
    /// and get the same result
    pub fn from_str(cards_str: &str) -> Result<Vec<Card>, CardError> {
        let cards_chars: Vec<char> = cards_str.chars().filter(|c| !c.is_whitespace()).collect();
        let mut cards: Vec<Card> = Vec::<Card>::new();
        // Make sure it's even to avoid breaking the for loop
        let len = cards_chars.len();
        if !len.is_multiple_of(2) {
            return Err(CardError::InvalidFormat(
                "Uneven number of chars".to_string(),
            ));
        }
        for i in (0..len - 1).step_by(2) {
            let rank_char = cards_chars[i];
            let rank = Rank::from_char(rank_char)?;
            let suit_char = cards_chars[i + 1];
            let suit = Suit::from_char(suit_char)?;
            cards.push(Card::new(suit, rank));
        }

        Ok(cards)
    }

    /// Whether this is the same card as `other`.
    pub fn matches(&self, other: &Card) -> bool {
        self == other
    }

    /// Get rank of card
    pub fn rank(&self) -> Rank {
        Rank::from_index(self.0 / 4)
    }

    /// Get suit of card
    pub fn suit(&self) -> Suit {
        Suit::from_index(self.0 % 4)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}{}",
            Rank::to_char(self.rank()),
            Suit::to_char(self.suit())
        )
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Sequence)]
pub enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl Suit {
    /// Takes a char and returns Suit or CardError if char doesn't match a suit
    pub fn from_char(c: char) -> Result<Suit, CardError> {
        match c.to_ascii_lowercase() {
            'c' => Ok(Suit::Club),
            'd' => Ok(Suit::Diamond),
            'h' => Ok(Suit::Heart),
            's' => Ok(Suit::Spade),
            _ => Err(CardError::InvalidSuit(c)),
        }
    }

    /// Takes a suit and returns the matching char
    pub fn to_char(suit: Suit) -> char {
        match suit {
            Suit::Club => 'c',
            Suit::Diamond => 'd',
            Suit::Heart => 'h',
            Suit::Spade => 's',
        }
    }

    /// The suit at `index` in `0..4`, ordered `cdhs`.
    ///
    /// Panics above three, which [`Card`] cannot produce.
    pub fn from_index(index: u8) -> Suit {
        Suit::all()[index as usize]
    }

    /// Every suit, in the `cdhs` order the card index uses.
    pub fn all() -> [Suit; 4] {
        [Suit::Club, Suit::Diamond, Suit::Heart, Suit::Spade]
    }
}
/// A card's rank. The discriminant is the rank's value, so the derived
/// ordering is the natural one: `Two` is lowest and `Ace` is highest.
///
/// Games that rank low hands invert this at the point of comparison rather
/// than here, so that one rank type serves every variant.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Sequence)]
pub enum Rank {
    Two = 2,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    pub fn to_value(&self) -> u8 {
        *self as u8
    }

    /// Takes a char and returns Rank or CardError if char doesn't match a rank
    pub fn from_char(c: char) -> Result<Rank, CardError> {
        match c.to_ascii_uppercase() {
            '2' => Ok(Rank::Two),
            '3' => Ok(Rank::Three),
            '4' => Ok(Rank::Four),
            '5' => Ok(Rank::Five),
            '6' => Ok(Rank::Six),
            '7' => Ok(Rank::Seven),
            '8' => Ok(Rank::Eight),
            '9' => Ok(Rank::Nine),
            'T' => Ok(Rank::Ten),
            'J' => Ok(Rank::Jack),
            'Q' => Ok(Rank::Queen),
            'K' => Ok(Rank::King),
            'A' => Ok(Rank::Ace),
            _ => Err(CardError::InvalidRank(c)),
        }
    }

    /// Takes a rank and returns the matching char
    pub fn to_char(rank: Rank) -> char {
        match rank {
            Rank::Two => '2',
            Rank::Three => '3',
            Rank::Four => '4',
            Rank::Five => '5',
            Rank::Six => '6',
            Rank::Seven => '7',
            Rank::Eight => '8',
            Rank::Nine => '9',
            Rank::Ten => 'T',
            Rank::Jack => 'J',
            Rank::Queen => 'Q',
            Rank::King => 'K',
            Rank::Ace => 'A',
        }
    }

    /// The rank at `index` in `0..13`, where zero is the deuce.
    ///
    /// Panics above twelve, which [`Card`] cannot produce.
    pub fn from_index(index: u8) -> Rank {
        Rank::all()[index as usize]
    }

    /// Every rank, lowest first.
    pub fn all() -> [Rank; 13] {
        [
            Rank::Two,
            Rank::Three,
            Rank::Four,
            Rank::Five,
            Rank::Six,
            Rank::Seven,
            Rank::Eight,
            Rank::Nine,
            Rank::Ten,
            Rank::Jack,
            Rank::Queen,
            Rank::King,
            Rank::Ace,
        ]
    }
}

// In card.rs, add Display for Rank
impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Self::to_char(*self))
    }
}
