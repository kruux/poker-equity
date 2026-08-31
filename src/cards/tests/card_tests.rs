use crate::cards::{Card, Rank, Suit};
use crate::error::CardError;

#[test]
fn test_card_from_str_valid_inputs() -> Result<(), CardError> {
    // Test each rank
    assert_eq!(Card::parse_field("Ah")?.len(), 1);
    assert_eq!(Card::parse_field("Ah")?[0], Card::new(Suit::Heart, Rank::Ace));
    assert_eq!(Card::parse_field("Kh")?.len(), 1);
    assert_eq!(Card::parse_field("Kh")?[0], Card::new(Suit::Heart, Rank::King));
    assert_eq!(Card::parse_field("Qh")?.len(), 1);
    assert_eq!(
        Card::parse_field("Qh")?[0],
        Card::new(Suit::Heart, Rank::Queen)
    );
    assert_eq!(Card::parse_field("Jh")?.len(), 1);
    assert_eq!(Card::parse_field("Jh")?[0], Card::new(Suit::Heart, Rank::Jack));
    assert_eq!(Card::parse_field("Th")?.len(), 1);
    assert_eq!(Card::parse_field("Th")?[0], Card::new(Suit::Heart, Rank::Ten));
    assert_eq!(Card::parse_field("9h")?.len(), 1);
    assert_eq!(Card::parse_field("9h")?[0], Card::new(Suit::Heart, Rank::Nine));
    assert_eq!(Card::parse_field("8h")?.len(), 1);
    assert_eq!(
        Card::parse_field("8h")?[0],
        Card::new(Suit::Heart, Rank::Eight)
    );
    assert_eq!(Card::parse_field("7h")?.len(), 1);
    assert_eq!(
        Card::parse_field("7h")?[0],
        Card::new(Suit::Heart, Rank::Seven)
    );
    assert_eq!(Card::parse_field("6h")?.len(), 1);
    assert_eq!(Card::parse_field("6h")?[0], Card::new(Suit::Heart, Rank::Six));
    assert_eq!(Card::parse_field("5h")?.len(), 1);
    assert_eq!(Card::parse_field("5h")?[0], Card::new(Suit::Heart, Rank::Five));
    assert_eq!(Card::parse_field("4h")?.len(), 1);
    assert_eq!(Card::parse_field("4h")?[0], Card::new(Suit::Heart, Rank::Four));
    assert_eq!(Card::parse_field("3h")?.len(), 1);
    assert_eq!(
        Card::parse_field("3h")?[0],
        Card::new(Suit::Heart, Rank::Three)
    );
    assert_eq!(Card::parse_field("2h")?.len(), 1);
    assert_eq!(Card::parse_field("2h")?[0], Card::new(Suit::Heart, Rank::Two));
    // ... etc

    // Test each suit
    assert_eq!(Card::parse_field("As")?.len(), 1);
    assert_eq!(Card::parse_field("Ah")?.len(), 1);
    assert_eq!(Card::parse_field("Ad")?.len(), 1);
    assert_eq!(Card::parse_field("Ac")?.len(), 1);

    Ok(())
}

#[test]
fn test_card_from_str_multiple_cards() -> Result<(), CardError> {
    let cards = Card::parse_field("AhKdQc")?;
    assert_eq!(cards.len(), 3);
    Ok(())
}

#[test]
fn test_card_from_str_with_spaces() -> Result<(), CardError> {
    let cards = Card::parse_field("Ah Kd Qc ")?;
    assert_eq!(cards.len(), 3);
    Ok(())
}

#[test]
fn test_card_from_str_invalid_rank() {
    let result = Card::parse_field("Xh");
    assert!(matches!(result, Err(CardError::InvalidRank('X'))));
}

#[test]
fn test_card_from_str_invalid_suit() {
    let result = Card::parse_field("Ax");
    assert!(matches!(result, Err(CardError::InvalidSuit('x'))));
}

#[test]
fn test_card_matches() -> Result<(), CardError> {
    let card1 = Card::parse_field("Ah")?[0];
    let card2 = Card::parse_field("Ah")?[0];
    let card3 = Card::parse_field("As")?[0];

    assert!(card1.matches(&card2));
    assert!(!card1.matches(&card3));
    Ok(())
}

/// One string, one card -- and a field of cards refused as such.
#[test]
fn test_a_single_card_parses_through_from_str() -> Result<(), CardError> {
    use crate::cards::{Rank, Suit};

    assert_eq!("Ah".parse::<Card>()?, Card::new(Suit::Heart, Rank::Ace));
    // Whitespace is ignored here as it is in a whole field.
    assert_eq!(" 2c ".parse::<Card>()?, Card::new(Suit::Club, Rank::Two));

    // Two cards is a field, and wants parse_field.
    assert!("AhKh".parse::<Card>().is_err());
    assert_eq!(Card::parse_field("AhKh")?.len(), 2);

    // No cards is not one card. An empty field is nought cards and no error,
    // which is how "no dead cards" is written; it used to underflow instead.
    assert!("".parse::<Card>().is_err());
    assert!(Card::parse_field("")?.is_empty());
    assert!(Card::parse_field("   ")?.is_empty());

    Ok(())
}
