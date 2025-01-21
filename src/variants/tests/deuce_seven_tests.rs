use crate::{
    cards::{Card, Rank},
    error::{CardError, PokerError},
    hand::Hand,
    variants::{DeuceSeven, DeuceSevenRank},
};

#[test]
fn test_new_empty_hand() {
    let hand = Hand::new(DeuceSeven);
    assert_eq!(hand.num_cards(), 0);
}

#[test]
fn test_add_card() -> Result<(), CardError> {
    let mut hand = Hand::new(DeuceSeven);
    let card = Card::from_str("Ah")?[0];
    hand.add_card(card)?;
    assert_eq!(hand.num_cards(), 1);
    Ok(())
}

#[test]
fn test_hand_from_str() -> Result<(), PokerError> {
    // Test without spaces
    let hand = Hand::from_str(DeuceSeven, "AhKdQc2s3d")?;
    assert_eq!(hand.num_cards(), 5);

    // Test with spaces
    let hand_with_spaces = Hand::from_str(DeuceSeven, "Ah Kd Qc 2s 3d ")?;
    assert_eq!(hand.num_cards(), 5);

    // Make sure the cards are the same in both variants
    for i in 0..5 {
        assert!(hand.cards()[i].matches(&hand_with_spaces.cards()[i]));
    }

    Ok(())
}

/// When creating a hand with 6 cards or more expect an error
#[test]
fn hand_from_str_with_too_many_cards() -> Result<(), PokerError> {
    assert!(matches!(
        Hand::from_str(DeuceSeven, "AhKdQc2s3d4h").unwrap_err(),
        PokerError::Card(CardError::TooManyCards(6))
    ));

    Ok(())
}

#[test]
fn test_hand_comparison() -> Result<(), PokerError> {
    // 75432 vs 76432
    let hand1 = Hand::from_str(DeuceSeven, "7d5h4c3s2h")?;
    let hand2 = Hand::from_str(DeuceSeven, "7c6s4h3d2c")?;

    assert!(hand1 > hand2); // 75432 better than 76432 in 2-7
    Ok(())
}

#[test]
fn test_two_pair_order() -> Result<(), PokerError> {
    // Make sure pairs are ordered by rank value (higher first)
    let hand1 = Hand::from_str(DeuceSeven, "7c7h2s2dAh")?;
    assert!(matches!(
        hand1.evaluate(),
        DeuceSevenRank::TwoPair(Rank::Seven, Rank::Two, Rank::Ace)
    ));

    // Test different order in input gives same result
    let hand2 = Hand::from_str(DeuceSeven, "2s2d7c7hAh")?;
    assert!(matches!(
        hand2.evaluate(),
        DeuceSevenRank::TwoPair(Rank::Seven, Rank::Two, Rank::Ace)
    ));

    Ok(())
}

#[test]
fn test_hand_evaluation() -> Result<(), PokerError> {
    // Test straight flush (worst hand in 2-7)
    let straight_flush = Hand::from_str(DeuceSeven, "7h6h5h4h3h")?;
    assert!(matches!(
        straight_flush.evaluate(),
        DeuceSevenRank::StraightFlush(_)
    ));

    // Test four of a kind
    let quads = Hand::from_str(DeuceSeven, "7c7h7s7dAh")?;
    assert!(matches!(
        quads.evaluate(),
        DeuceSevenRank::FourOfAKind(Rank::Seven, Rank::Ace)
    ));

    // Test full house
    let full_house = Hand::from_str(DeuceSeven, "7c7h7sAhAd")?;
    assert!(matches!(
        full_house.evaluate(),
        DeuceSevenRank::FullHouse(Rank::Seven, Rank::Ace)
    ));

    // Test flush
    let flush = Hand::from_str(DeuceSeven, "Ah7h5h4h2h")?;
    assert!(matches!(flush.evaluate(), DeuceSevenRank::Flush(_)));

    // Test straight (remember A2345 is not a straight in 2-7!)
    let straight = Hand::from_str(DeuceSeven, "7c6h5s4d3h")?;
    assert!(matches!(
        straight.evaluate(),
        DeuceSevenRank::Straight(Rank::Seven)
    ));

    // Test three of a kind
    let trips = Hand::from_str(DeuceSeven, "7c7h7sAh2d")?;
    if let DeuceSevenRank::ThreeOfAKind(rank, kickers) = trips.evaluate() {
        assert_eq!(rank, Rank::Seven);
        assert_eq!(kickers, vec![Rank::Ace, Rank::Two]);
    } else {
        panic!("Expected ThreeOfAKind, got different hand rank");
    }

    // Test two pair
    let two_pair = Hand::from_str(DeuceSeven, "7c7h2s2dAh")?;
    let two_pair_rank = two_pair.evaluate();
    assert_eq!(
        two_pair_rank,
        DeuceSevenRank::TwoPair(Rank::Seven, Rank::Two, Rank::Ace)
    );

    // Test one pair
    let pair = Hand::from_str(DeuceSeven, "7c7h3s4d2h")?;
    assert!(matches!(
        pair.evaluate(),
        DeuceSevenRank::Pair(Rank::Seven, _)
    ));

    // Test high card (best possible 2-7 hand: 7532A)
    let high_card = Hand::from_str(DeuceSeven, "7c5h3s2dAh")?;
    assert!(matches!(high_card.evaluate(), DeuceSevenRank::HighCard(_)));

    Ok(())
}

#[test]
fn test_not_straight() -> Result<(), PokerError> {
    // A2345 should not be detected as a straight in 2-7
    let wheel = Hand::from_str(DeuceSeven, "Ah2h3h4h5h")?;
    assert!(matches!(wheel.evaluate(), DeuceSevenRank::Flush(_))); // Should be evaluated as just a flush

    let wheel_mixed = Hand::from_str(DeuceSeven, "Ac2h3d4s5h")?;
    assert!(matches!(
        wheel_mixed.evaluate(),
        DeuceSevenRank::HighCard(_)
    )); // Just a high card hand

    Ok(())
}

#[test]
fn test_kickers_order() -> Result<(), PokerError> {
    // Test that kickers are properly ordered in pair
    let pair_with_kickers = Hand::from_str(DeuceSeven, "7c7hAh5h2d")?;
    if let DeuceSevenRank::Pair(rank, kickers) = pair_with_kickers.evaluate() {
        assert_eq!(rank, Rank::Seven);
        assert_eq!(kickers, vec![Rank::Ace, Rank::Five, Rank::Two]);
    } else {
        panic!("Expected Pair, got different hand rank");
    }

    Ok(())
}

#[test]
fn test_straight_flush_comparison() -> Result<(), PokerError> {
    // Lower straight flush beats higher straight flush in 2-7
    let hand1 = Hand::from_str(DeuceSeven, "7h6h5h4h3h")?; // 7-high straight flush
    let hand2 = Hand::from_str(DeuceSeven, "8h7h6h5h4h")?; // 8-high straight flush
    assert!(hand1 > hand2);
    Ok(())
}

#[test]
fn test_four_of_kind_comparison() -> Result<(), PokerError> {
    // Lower quads beats higher quads
    let hand1 = Hand::from_str(DeuceSeven, "2c2h2s2dAh")?;
    let hand2 = Hand::from_str(DeuceSeven, "3c3h3s3dAh")?;
    assert!(hand1 > hand2);
    Ok(())
}

#[test]
fn test_full_house_comparison() -> Result<(), PokerError> {
    // Compare trips first, then pairs
    let hand1 = Hand::from_str(DeuceSeven, "2c2h2sAhAd")?; // 2s full of Aces
    let hand2 = Hand::from_str(DeuceSeven, "3c3h3sKhKd")?; // 3s full of Kings
    assert!(hand1 > hand2);

    // Same trips, different pairs
    let hand3 = Hand::from_str(DeuceSeven, "2c2h2sKhKd")?; // 2s full of Kings
    assert!(hand1 < hand3);

    Ok(())
}

#[test]
fn test_flush_comparison() -> Result<(), PokerError> {
    // Compare each card in order
    let hand1 = Hand::from_str(DeuceSeven, "7h5h4h3h2h")?;
    let hand2 = Hand::from_str(DeuceSeven, "7h6h4h3h2h")?;
    assert!(hand1 > hand2);
    Ok(())
}

#[test]
fn test_straight_comparison() -> Result<(), PokerError> {
    // Lower straight beats higher straight
    let hand1 = Hand::from_str(DeuceSeven, "6c5h4s3d2h")?;
    let hand2 = Hand::from_str(DeuceSeven, "7c6h5s4d3h")?;
    assert!(hand1 > hand2);
    Ok(())
}

#[test]
fn test_three_of_kind_comparison() -> Result<(), PokerError> {
    // Lower trips beats higher trips
    let hand1 = Hand::from_str(DeuceSeven, "2c2h2sAhKd")?;
    let hand2 = Hand::from_str(DeuceSeven, "3c3h3sAhKd")?;
    assert!(hand1 > hand2);
    Ok(())
}

#[test]
fn test_two_pair_comparison() -> Result<(), PokerError> {
    // Compare high pair first
    let hand1 = Hand::from_str(DeuceSeven, "3c3h2s2dAh")?;
    let hand2 = Hand::from_str(DeuceSeven, "4c4h2s2dKh")?;
    assert!(hand1 > hand2);

    // Same high pair, compare low pair
    let hand3 = Hand::from_str(DeuceSeven, "3c3h4s4dQh")?;
    assert!(hand1 > hand3);

    // Same pairs, compare kicker
    let hand4 = Hand::from_str(DeuceSeven, "3c3h2s2dKh")?;
    assert!(hand1 < hand4);

    Ok(())
}

#[test]
fn test_pair_comparison() -> Result<(), PokerError> {
    // Lower pair beats higher pair
    let hand1 = Hand::from_str(DeuceSeven, "2c2h7s5d3h")?;
    let hand2 = Hand::from_str(DeuceSeven, "3c3h7s5d4h")?;
    assert!(hand1 > hand2);

    // Same pair, compare kickers
    let hand3 = Hand::from_str(DeuceSeven, "2c2h8s5d3h")?;
    // 753 beats 853
    assert!(hand1 > hand3);

    Ok(())
}

#[test]
fn test_high_card_comparison() -> Result<(), PokerError> {
    // Best possible 2-7 hand vs slightly worse
    let hand1 = Hand::from_str(DeuceSeven, "7c5h3s2dAh")?; // 7532A
    let hand2 = Hand::from_str(DeuceSeven, "7c6h3s2dAh")?; // 7632A
    assert!(hand1 > hand2);
    Ok(())
}

#[test]
fn test_different_hand_types() -> Result<(), PokerError> {
    // High card beats pair in 2-7
    let high_card = Hand::from_str(DeuceSeven, "7c5h3s2dAh")?;
    let pair = Hand::from_str(DeuceSeven, "2c2h7s5d3h")?;
    assert!(high_card > pair);

    // Pair beats two pair
    let two_pair = Hand::from_str(DeuceSeven, "2c2h3s3dAh")?;
    assert!(pair > two_pair);
    Ok(())
}
