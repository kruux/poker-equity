use crate::cards::Card;

/// Anything that can go wrong, whatever part it came from.
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

/// A move or a deal that the rules do not allow.
#[derive(Debug)]
pub enum GameError {
    PlayerNotFound(String),
    InactivePlayer(String),
    InvalidDiscard(Card),
    DuplicateCard(Card),
    /// A card that this game's deck does not contain -- a deuce in short
    /// deck, say. Naming it is a mistake about the game, not about the hand.
    NotInDeck(Card),
    NotEnoughCards,
}

impl GameError {
    /// What went wrong, in a sentence.
    pub fn description(&self) -> String {
        match self {
            GameError::PlayerNotFound(name) => format!("Player '{}' not found", name),
            GameError::DuplicateCard(card) => format!("Found duplicated card: {}", card),
            GameError::InvalidDiscard(card) => format!("Card not in hand {}", card),
            GameError::InactivePlayer(name) => format!("Player '{}' is not active", name),
            GameError::NotInDeck(card) => {
                format!("{} is not in this game's deck", card)
            }
            GameError::NotEnoughCards => "Not enough cards in deck".to_string(),
        }
    }
}

/// A card, or a group of cards, that could not be read or used.
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
    /// What went wrong, in a sentence.
    pub fn description(&self) -> String {
        match self {
            CardError::IncompleteHand(n) => format!("All hands not dealt yet: {}", n),
            CardError::InvalidComparison => "Error comparing hands".to_string(),
            CardError::InvalidFormat(s) => s.to_string(),
            CardError::InvalidRank(c) => format!("Invalid rank character: {}", c),
            CardError::InvalidSuit(c) => format!("Invalid suit character: {}", c),
            CardError::TooManyCards(n) => format!("Too many cards in Vec: {}", n),
            CardError::TooManyDiscards(n) => format!("Cannot discard more than 5 cards: {}", n),
            CardError::CardNotFound(card) => format!("Card not found {}", card),
        }
    }
}

/// A request the engine cannot answer as asked.
#[derive(Debug)]
pub enum EquityError {
    NoPlayers,
    NotEnoughCards(usize),
    InvalidSimulationCount(usize),
    UnequalHandSizes,
    InvalidCommunityCards(usize),
    /// A board shorter than the game's floor: how many it shows before the
    /// betting, and how many were named. Only Courchevel has a floor at all.
    NotEnoughBoardCards { least: usize, found: usize },
    /// A precision no run can reach. A standard error falls as `1/sqrt(n)`,
    /// so it approaches zero without arriving, and a target of zero -- or of
    /// anything that is not a positive, finite number -- is a promise the
    /// sampler cannot keep rather than a goal it can pursue.
    UnreachableTarget(f64),
    /// More seats than the deck can deal: how many were asked for, and how
    /// many the game has room for.
    TooManyPlayers { asked: usize, room: usize },
    /// No deal satisfies the request, with the field at fault named.
    Infeasible(String),
    /// A notation field could not be read.
    Notation(crate::notation::NotationError),
}

impl EquityError {
    /// What went wrong, in a sentence.
    pub fn description(&self) -> String {
        match self {
            EquityError::NoPlayers => "Need atleast 2 players for a simulation".to_string(),
            EquityError::NotEnoughCards(n) => format!("Not enough cards in hand: {}", n),
            EquityError::InvalidSimulationCount(n) => format!("Invalid number of simulations: {n}"),
            EquityError::UnequalHandSizes => "Starting hands with different sizes".to_string(),
            EquityError::TooManyPlayers { asked, room } => format!(
                "{} players is more than this game can deal; the deck seats {}",
                asked, room
            ),
            EquityError::Infeasible(field) => {
                format!("No deal can satisfy {}", field)
            }
            EquityError::Notation(error) => format!("{}", error),
            EquityError::InvalidCommunityCards(n) => {
                format!("Invalid number of community cards: {n}")
            }
            EquityError::NotEnoughBoardCards { least, found } => format!(
                "this game turns {} board card{} face up before the betting, so a board \
                 of {} is not a spot that occurs; write `*` for a card that has been \
                 dealt and not yet seen",
                least,
                if *least == 1 { "" } else { "s" },
                found
            ),
            EquityError::UnreachableTarget(wanted) => format!(
                "a standard error target must be positive and finite, and {} is not; \
                 sampling narrows the error as 1/sqrt(n), so it would never arrive. Ask \
                 for Target::Exact if what you want is no error bar at all",
                wanted
            ),
        }
    }
}

impl From<crate::notation::NotationError> for PokerError {
    fn from(error: crate::notation::NotationError) -> Self {
        PokerError::Equity(EquityError::Notation(error))
    }
}

// Each error type already knows how to describe itself; these make that
// description the standard one, so the errors print through `{}`, work with
// `?` into a `Box<dyn Error>`, and can be handed to any library that expects
// an ordinary Rust error.

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::fmt::Display for CardError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::fmt::Display for EquityError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description())
    }
}

impl std::fmt::Display for PokerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PokerError::Card(error) => write!(f, "{}", error),
            PokerError::Game(error) => write!(f, "{}", error),
            PokerError::Equity(error) => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for GameError {}
impl std::error::Error for CardError {}
impl std::error::Error for EquityError {}

impl std::error::Error for PokerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PokerError::Card(error) => Some(error),
            PokerError::Game(error) => Some(error),
            PokerError::Equity(error) => Some(error),
        }
    }
}
