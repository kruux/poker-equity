use crate::cards::{Card, Deck};
use crate::error::CardError;

#[test]
fn test_new_deck() {
    let deck = Deck::new();
    assert_eq!(deck.remaining_cards(), 52);
}

#[test]
fn test_deal_card() -> Result<(), CardError> {
    let mut deck = Deck::new();
    let card = deck.deal();
    assert!(card.is_some());
    assert_eq!(deck.remaining_cards(), 51);
    Ok(())
}

#[test]
fn test_remove_card() -> Result<(), CardError> {
    let mut deck = Deck::new();
    let card = Card::from_str("Ah")?[0];
    deck.remove_card(&card)?;
    assert_eq!(deck.remaining_cards(), 51);

    // Try removing same card again
    assert!(matches!(
        deck.remove_card(&card),
        Err(CardError::CardNotFound(_))
    ));
    Ok(())
}

#[test]
fn test_shuffle() {
    let mut deck1 = Deck::new();
    let deck2 = Deck::new();
    deck1.shuffle();

    // Note: There's a tiny chance this could fail even with a good shuffle
    // That chance is 52! however. So if it fails it's almost guaranteed to be bad rng
    assert_ne!(deck1.cards(), deck2.cards());
}
