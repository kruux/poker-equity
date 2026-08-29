use crate::cards::Card;

#[derive(Debug)]
pub enum PokerError {
    Card(CardError),
    Game(GameError),
    Equity(EquityError),
}
// Implement From to allow automatic conversion
impl From<CardError> for PokerError {
    fn from(error: CardError) -> Self {
        PokerError::Card(error)
    }
}
impl From<GameError> for PokerError {
    fn from(error: GameError) -> Self {
        PokerError::Game(error)
    }
}
impl From<EquityError> for PokerError {
    fn from(error: EquityError) -> Self {
        PokerError::Equity(error)
    }
}

#[derive(Debug)]
pub enum GameError {
    PlayerNotFound(String),
    InactivePlayer(String),
    InvalidDiscard(Card),
    DuplicateCard(Card),
    NotEnoughCards,
}

impl GameError {
    pub fn description(&self) -> String {
        match self {
            GameError::PlayerNotFound(name) => format!("Player '{}' not found", name),
            GameError::DuplicateCard(card) => format!("Found duplicated card: {}", card),
            GameError::InvalidDiscard(card) => format!("Card not in hand {}", card),
            GameError::InactivePlayer(name) => format!("Player '{}' is not active", name),
            GameError::NotEnoughCards => "Not enough cards in deck".to_string(),
        }
    }
}

#[derive(Debug)]
pub enum CardError {
    IncompleteHand(usize),
    InvalidComparison,
    InvalidFormat(String),
    InvalidRank(char),
    InvalidSuit(char),
    TooManyCards(usize),
    TooManyDiscards(usize),
    CardNotFound(Card),
}

impl CardError {
    pub fn description(&self) -> String {
        match self {
            CardError::IncompleteHand(n) => format!("All hands not dealt yet: {}", n),
            CardError::InvalidComparison => "Error comparing hands".to_string(),
            CardError::InvalidFormat(s) => format!("{}", s),
            CardError::InvalidRank(c) => format!("Invalid rank character: {}", c),
            CardError::InvalidSuit(c) => format!("Invalid suit character: {}", c),
            CardError::TooManyCards(n) => format!("Too many cards in Vec: {}", n),
            CardError::TooManyDiscards(n) => format!("Cannot discard more than 5 cards: {}", n),
            CardError::CardNotFound(card) => format!("Card not found {}", card),
        }
    }
}

#[derive(Debug)]
pub enum EquityError {
    NoPlayers,
    NotEnoughCards(usize),
    InvalidSimulationCount(usize),
    UnequalHandSizes,
    InvalidCommunityCards(usize),
    /// No deal satisfies the request, with the field at fault named.
    Infeasible(String),
    /// A notation field could not be read.
    Notation(crate::notation::NotationError),
}

impl EquityError {
    pub fn description(&self) -> String {
        match self {
            EquityError::NoPlayers => format!("Need atleast 2 players for a simulation"),
            EquityError::NotEnoughCards(n) => format!("Not enough cards in hand: {}", n),
            EquityError::InvalidSimulationCount(n) => format!("Invalid number of simulations: {n}"),
            EquityError::UnequalHandSizes => format!("Starting hands with different sizes"),
            EquityError::Infeasible(field) => {
                format!("No deal can satisfy {}", field)
            }
            EquityError::Notation(error) => format!("{}", error),
            EquityError::InvalidCommunityCards(n) => {
                format!("Invalid number of community cards: {n}")
            }
        }
    }
}

impl From<crate::notation::NotationError> for PokerError {
    fn from(error: crate::notation::NotationError) -> Self {
        PokerError::Equity(EquityError::Notation(error))
    }
}
