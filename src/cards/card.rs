use enum_iterator::Sequence;
use std::fmt;

use crate::error::CardError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    suit: Suit,
    rank: Rank,
}

impl Card {
    pub fn new(suit: Suit, rank: Rank) -> Self {
        Self { suit, rank }
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
        if len % 2 != 0 {
            return Err(CardError::InvalidFormat(
                "Uneven number of chars".to_string(),
            ));
        }
        for i in (0..len - 1).step_by(2) {
            let rank_char = cards_chars[i];
            let rank = Rank::from_char(rank_char)?;
            let suit_char = cards_chars[i + 1];
            let suit = Suit::from_char(suit_char)?;
            let card = Card { suit, rank };
            cards.push(card);
        }

        Ok(cards)
    }

    // Check if two Cards are the same
    pub fn matches(&self, other: &Card) -> bool {
        self.rank == other.rank && self.suit == other.suit
    }

    /// Get rank of card
    pub fn rank(&self) -> Rank {
        return self.rank;
    }

    /// Get suit of card
    pub fn suit(&self) -> Suit {
        return self.suit;
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let suit = self.suit;
        let rank = self.rank;
        let suit_char = Suit::to_char(suit);
        let rank_char = Rank::to_char(rank);
        write!(f, "{}{}", rank_char, suit_char)
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
            Suit::Club => return 'c',
            Suit::Diamond => return 'd',
            Suit::Heart => return 'h',
            Suit::Spade => return 's',
        }
    }

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
            Rank::Two => return '2',
            Rank::Three => return '3',
            Rank::Four => return '4',
            Rank::Five => return '5',
            Rank::Six => return '6',
            Rank::Seven => return '7',
            Rank::Eight => return '8',
            Rank::Nine => return '9',
            Rank::Ten => return 'T',
            Rank::Jack => return 'J',
            Rank::Queen => return 'Q',
            Rank::King => return 'K',
            Rank::Ace => return 'A',
        }
    }

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
