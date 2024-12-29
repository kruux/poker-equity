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
    InvalidSimulationCount,
}
