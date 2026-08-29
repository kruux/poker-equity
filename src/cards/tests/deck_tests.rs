use crate::cards::{Card, CardSet, Deck};
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
fn test_deals_in_a_random_order() {
    // A set has no order, so what matters is that successive deals are not
    // the same sequence. Two full decks agreeing on all ten cards would be
    // one chance in 52*51*...*43.
    let mut first = Deck::new();
    let mut second = Deck::new();
    let ten = |deck: &mut Deck| -> Vec<Card> { (0..10).filter_map(|_| deck.deal()).collect() };
    assert_ne!(ten(&mut first), ten(&mut second));
}

#[test]
fn test_deals_every_card_exactly_once() {
    let mut deck = Deck::new();
    let mut seen = CardSet::EMPTY;
    while let Some(card) = deck.deal() {
        assert!(!seen.contains(card), "{} dealt twice", card);
        seen.insert(card);
    }
    assert_eq!(seen, CardSet::FULL_DECK);
    assert_eq!(deck.remaining_cards(), 0);
}
