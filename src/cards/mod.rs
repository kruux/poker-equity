mod card;
mod deck;

pub use card::{Card, Rank, Suit};
pub use deck::Deck;

#[cfg(test)]
mod tests {
    mod card_tests;
    mod deck_tests;
}
