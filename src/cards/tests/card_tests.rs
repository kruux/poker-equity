use crate::cards::{Card, Rank, Suit};
use crate::error::CardError;

#[test]
fn test_card_from_str_valid_inputs() -> Result<(), CardError> {
    // Test each rank
    assert_eq!(Card::from_str("Ah")?.len(), 1);
    assert_eq!(Card::from_str("Ah")?[0], Card::new(Suit::Heart, Rank::Ace));
    assert_eq!(Card::from_str("Kh")?.len(), 1);
    assert_eq!(Card::from_str("Kh")?[0], Card::new(Suit::Heart, Rank::King));
    assert_eq!(Card::from_str("Qh")?.len(), 1);
    assert_eq!(
        Card::from_str("Qh")?[0],
        Card::new(Suit::Heart, Rank::Queen)
    );
    assert_eq!(Card::from_str("Jh")?.len(), 1);
    assert_eq!(Card::from_str("Jh")?[0], Card::new(Suit::Heart, Rank::Jack));
    assert_eq!(Card::from_str("Th")?.len(), 1);
    assert_eq!(Card::from_str("Th")?[0], Card::new(Suit::Heart, Rank::Ten));
    assert_eq!(Card::from_str("9h")?.len(), 1);
    assert_eq!(Card::from_str("9h")?[0], Card::new(Suit::Heart, Rank::Nine));
    assert_eq!(Card::from_str("8h")?.len(), 1);
    assert_eq!(
        Card::from_str("8h")?[0],
        Card::new(Suit::Heart, Rank::Eight)
    );
    assert_eq!(Card::from_str("7h")?.len(), 1);
    assert_eq!(
        Card::from_str("7h")?[0],
        Card::new(Suit::Heart, Rank::Seven)
    );
    assert_eq!(Card::from_str("6h")?.len(), 1);
    assert_eq!(Card::from_str("6h")?[0], Card::new(Suit::Heart, Rank::Six));
    assert_eq!(Card::from_str("5h")?.len(), 1);
    assert_eq!(Card::from_str("5h")?[0], Card::new(Suit::Heart, Rank::Five));
    assert_eq!(Card::from_str("4h")?.len(), 1);
    assert_eq!(Card::from_str("4h")?[0], Card::new(Suit::Heart, Rank::Four));
    assert_eq!(Card::from_str("3h")?.len(), 1);
    assert_eq!(
        Card::from_str("3h")?[0],
        Card::new(Suit::Heart, Rank::Three)
    );
    assert_eq!(Card::from_str("2h")?.len(), 1);
    assert_eq!(Card::from_str("2h")?[0], Card::new(Suit::Heart, Rank::Two));
    // ... etc

    // Test each suit
    assert_eq!(Card::from_str("As")?.len(), 1);
    assert_eq!(Card::from_str("Ah")?.len(), 1);
    assert_eq!(Card::from_str("Ad")?.len(), 1);
    assert_eq!(Card::from_str("Ac")?.len(), 1);

    Ok(())
}

#[test]
fn test_card_from_str_multiple_cards() -> Result<(), CardError> {
    let cards = Card::from_str("AhKdQc")?;
    assert_eq!(cards.len(), 3);
    Ok(())
}

#[test]
fn test_card_from_str_with_spaces() -> Result<(), CardError> {
    let cards = Card::from_str("Ah Kd Qc ")?;
    assert_eq!(cards.len(), 3);
    Ok(())
}

#[test]
fn test_card_from_str_invalid_rank() {
    let result = Card::from_str("Xh");
    assert!(matches!(result, Err(CardError::InvalidRank('X'))));
}

#[test]
fn test_card_from_str_invalid_suit() {
    let result = Card::from_str("Ax");
    assert!(matches!(result, Err(CardError::InvalidSuit('x'))));
}

#[test]
fn test_card_matches() -> Result<(), CardError> {
    let card1 = Card::from_str("Ah")?[0];
    let card2 = Card::from_str("Ah")?[0];
    let card3 = Card::from_str("As")?[0];

    assert!(card1.matches(&card2));
    assert!(!card1.matches(&card3));
    Ok(())
}
