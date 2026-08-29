use std::cmp::Ordering;

use crate::{
    cards::Rank,
    error::PokerError,
    hand::Hand,
    variants::{rankings::HoldemHandRank, Holdem},
};

#[test]
fn test_straight_flush() -> Result<(), PokerError> {
    // Test royal flush
    let hand = Hand::from_str(Holdem, "Ah Kh Qh Jh Th 2c 3d")?;
    assert_eq!(hand.evaluate(), HoldemHandRank::StraightFlush(Rank::Ace));

    // Test mid straight flush
    let hand = Hand::from_str(Holdem, "9h 8h 7h 6h 5h 2c 3d")?;
    assert_eq!(hand.evaluate(), HoldemHandRank::StraightFlush(Rank::Nine));

    // Test low straight flush
    let hand = Hand::from_str(Holdem, "6h 5h 4h 3h 2h Kc Qd")?;
    assert_eq!(hand.evaluate(), HoldemHandRank::StraightFlush(Rank::Six));

    Ok(())
}

#[test]
fn test_four_of_a_kind() -> Result<(), PokerError> {
    // Test quad aces
    let hand = Hand::from_str(Holdem, "Ah Ac Ad As Kh 2c 3d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::FourOfAKind(Rank::Ace, Rank::King)
    );

    // Test quad twos
    let hand = Hand::from_str(Holdem, "2h 2c 2d 2s Ah Kc Qd")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::FourOfAKind(Rank::Two, Rank::Ace)
    );

    // Test quad tens
    let hand = Hand::from_str(Holdem, "Th Tc Td Ts Kh Qc 2d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::FourOfAKind(Rank::Ten, Rank::King)
    );

    Ok(())
}

#[test]
fn test_full_house() -> Result<(), PokerError> {
    // Test twos full of threes
    let hand = Hand::from_str(Holdem, "2s 2h 3h 3c Ks Jd 2c")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::FullHouse(Rank::Two, Rank::Three)
    );

    // Test kings full of deuces
    let hand = Hand::from_str(Holdem, "Kh Kc Kd 2h 2c As Qd")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::FullHouse(Rank::King, Rank::Two)
    );

    // Test fives full of aces
    let hand = Hand::from_str(Holdem, "5h 5c 5d 2s 2d Ah Ac")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::FullHouse(Rank::Five, Rank::Ace)
    );

    Ok(())
}

#[test]
fn test_flush() -> Result<(), PokerError> {
    // Test ace-high flush
    let hand = Hand::from_str(Holdem, "Ah Kh 7h 4h 2h Qs Jd")?;
    if let HoldemHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[4], Rank::Two);
    } else {
        panic!("Expected a flush!");
    }

    // Test king-high flush
    let hand = Hand::from_str(Holdem, "Kh 9h Qh 6h 3h As 2d")?;
    if let HoldemHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::King);
        assert_eq!(ranks[1], Rank::Queen);
        assert_eq!(ranks[2], Rank::Nine);
        assert_eq!(ranks[3], Rank::Six);
        assert_eq!(ranks[4], Rank::Three);
    } else {
        panic!("Expected a flush!");
    }

    // Test seven-high flush
    let hand = Hand::from_str(Holdem, "5h 4h 2h As Kd 7h 6h")?;
    if let HoldemHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Seven);
        assert_eq!(ranks[1], Rank::Six);
        assert_eq!(ranks[2], Rank::Five);
        assert_eq!(ranks[3], Rank::Four);
        assert_eq!(ranks[4], Rank::Two);
    } else {
        panic!("Expected a flush!");
    }

    Ok(())
}

#[test]
fn test_straight() -> Result<(), PokerError> {
    // Test broadway straight
    let hand = Hand::from_str(Holdem, "Ah Kc Qd Js Th 2c 3d")?;
    assert_eq!(hand.evaluate(), HoldemHandRank::Straight(Rank::Ace));

    // Test mid straight
    let hand = Hand::from_str(Holdem, "9h 8c 7d 6s 5h Ac 2d")?;
    assert_eq!(hand.evaluate(), HoldemHandRank::Straight(Rank::Nine));

    // Test wheel straight
    let hand = Hand::from_str(Holdem, "5h 4c 3d 2s Ah Kc Qd")?;
    assert_eq!(hand.evaluate(), HoldemHandRank::Straight(Rank::Five));

    Ok(())
}

#[test]
fn test_three_of_a_kind() -> Result<(), PokerError> {
    // Test three aces
    let hand = Hand::from_str(Holdem, "Ah Ac Ad Kh Qc 2s 3d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::ThreeOfAKind(Rank::Ace, [Rank::King, Rank::Queen])
    );

    // Test three fives
    let hand = Hand::from_str(Holdem, "5h 5c 5d Ah Kc Qs 2d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::ThreeOfAKind(Rank::Five, [Rank::Ace, Rank::King])
    );

    // Test three deuces
    let hand = Hand::from_str(Holdem, "2h 2c 2d Ah Kc Qs Jd")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::ThreeOfAKind(Rank::Two, [Rank::Ace, Rank::King])
    );

    Ok(())
}

#[test]
fn test_two_pair() -> Result<(), PokerError> {
    // Test aces and kings
    let hand = Hand::from_str(Holdem, "Ah Ac Kh Kc Qc 2s 3d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::TwoPair(Rank::Ace, Rank::King, Rank::Queen)
    );

    // Test tens and fives
    let hand = Hand::from_str(Holdem, "Th Tc 5h 5c Ac 2s 3d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::TwoPair(Rank::Ten, Rank::Five, Rank::Ace)
    );

    // Test threes and twos
    let hand = Hand::from_str(Holdem, "3h 3c 2h 2c Ac Ks Qd")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::TwoPair(Rank::Three, Rank::Two, Rank::Ace)
    );

    Ok(())
}

#[test]
fn test_one_pair() -> Result<(), PokerError> {
    // Test pair of aces
    let hand = Hand::from_str(Holdem, "Ah Ac Kh Qc Js 2c 3d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::Pair(Rank::Ace, [Rank::King, Rank::Queen, Rank::Jack])
    );

    // Test pair of tens
    let hand = Hand::from_str(Holdem, "Th Tc Ah Kc Qc 2s 3d")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::Pair(Rank::Ten, [Rank::Ace, Rank::King, Rank::Queen])
    );

    // Test pair of deuces
    let hand = Hand::from_str(Holdem, "2h 2c Ah 9c Qc Js Td")?;
    assert_eq!(
        hand.evaluate(),
        HoldemHandRank::Pair(Rank::Two, [Rank::Ace, Rank::Queen, Rank::Jack])
    );

    Ok(())
}

#[test]
fn test_high_card() -> Result<(), PokerError> {
    // Test ace high
    let hand = Hand::from_str(Holdem, "Ah Jc 9c 7s 2d Kc Qh")?;
    if let HoldemHandRank::HighCard(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[1], Rank::King);
        assert_eq!(ranks[2], Rank::Queen);
        assert_eq!(ranks[3], Rank::Jack);
        assert_eq!(ranks[4], Rank::Nine);
    } else {
        panic!("Expected high card!");
    }

    // Test king high
    let hand = Hand::from_str(Holdem, "Kh Qc Jh Tc 8c 6s 2d")?;
    if let HoldemHandRank::HighCard(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::King);
        assert_eq!(ranks[1], Rank::Queen);
        assert_eq!(ranks[2], Rank::Jack);
        assert_eq!(ranks[3], Rank::Ten);
        assert_eq!(ranks[4], Rank::Eight);
    } else {
        panic!("Expected high card!");
    }

    // Test queen high
    let hand = Hand::from_str(Holdem, "Qh Jc Th 9c 4s 2d 7c")?;
    if let HoldemHandRank::HighCard(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Queen);
        assert_eq!(ranks[1], Rank::Jack);
        assert_eq!(ranks[2], Rank::Ten);
        assert_eq!(ranks[3], Rank::Nine);
        assert_eq!(ranks[4], Rank::Seven);
    } else {
        panic!("Expected high card!");
    }

    Ok(())
}

#[test]
fn test_equality_and_transitivity() -> Result<(), PokerError> {
    // Test straight flush equality and transitivity
    let royal1 = Hand::from_str(Holdem, "Ah Kh Qh Jh Th 2c 3d")?;
    let royal2 = Hand::from_str(Holdem, "Ah Kh Qh Jh Th 4s 5d")?;
    let lower_sf = Hand::from_str(Holdem, "9h 8h 7h 6h 5h 2c 3d")?;
    assert_eq!(royal1.evaluate(), royal2.evaluate());
    assert!(royal1 == royal2);
    assert!(royal1 > lower_sf);

    // Test four of a kind equality and transitivity
    let quads1 = Hand::from_str(Holdem, "Ah Ac Ad As Kh 2c 3d")?;
    let quads2 = Hand::from_str(Holdem, "Ah Ac Ad As Kh 4c 5d")?;
    let lower_quads = Hand::from_str(Holdem, "Kh Kc Kd Ks Ah 2c 3d")?;
    assert_eq!(quads1.evaluate(), quads2.evaluate());
    assert!(quads1 == quads2);
    assert!(quads1 > lower_quads);

    // Test full house equality and transitivity
    let boat1 = Hand::from_str(Holdem, "Ah Ac Ad Kh Kc 2c 3d")?;
    let boat2 = Hand::from_str(Holdem, "Ah Ac Ad Kh Kc 4s 5d")?;
    let lower_boat = Hand::from_str(Holdem, "Kh Kc Kd Ah Ac 2s 3d")?;
    assert_eq!(boat1.evaluate(), boat2.evaluate());
    assert!(boat1 == boat2);
    assert!(boat1 > lower_boat);

    // Test flush equality and transitivity
    let flush1 = Hand::from_str(Holdem, "Ah Kh Qh Jh 9h 2c 3d")?;
    let flush2 = Hand::from_str(Holdem, "Ah Kh Qh Jh 9h 4s 5d")?;
    let lower_flush = Hand::from_str(Holdem, "Kh Qh Jh Th 8h 2c 3d")?;
    assert_eq!(flush1.evaluate(), flush2.evaluate());
    assert!(flush1 == flush2);
    assert!(flush1 > lower_flush);

    // Test straight equality and transitivity
    let straight1 = Hand::from_str(Holdem, "Ah Kc Qd Js Th 2c 3d")?;
    let straight2 = Hand::from_str(Holdem, "Ah Kc Qd Js Th 4h 5s")?;
    let lower_straight = Hand::from_str(Holdem, "Kc Qd Js Th 9h 2c 3d")?;
    assert_eq!(straight1.evaluate(), straight2.evaluate());
    assert!(straight1 == straight2);
    assert!(straight1 > lower_straight);

    // Test three of a kind equality and transitivity
    let trips1 = Hand::from_str(Holdem, "Ah Ac Ad Kh Qc 2s 3d")?;
    let trips2 = Hand::from_str(Holdem, "Ah Ac Ad Kh Qc 4s 5d")?;
    let lower_trips = Hand::from_str(Holdem, "Kh Kc Kd Ah Qc 2s 3d")?;
    assert_eq!(trips1.evaluate(), trips2.evaluate());
    assert!(trips1 == trips2);
    assert!(trips1 > lower_trips);

    // Test two pair equality and transitivity
    let two_pair1 = Hand::from_str(Holdem, "Ah Ac Kh Kc Qc 2s 3d")?;
    let two_pair2 = Hand::from_str(Holdem, "Ah Ac Kh Kc Qc 4s 5d")?;
    let lower_two_pair = Hand::from_str(Holdem, "Kh Kc Qh Qc Ac 2s 3d")?;
    assert_eq!(two_pair1.evaluate(), two_pair2.evaluate());
    assert!(two_pair1 == two_pair2);
    assert!(two_pair1 > lower_two_pair);

    // Test one pair equality and transitivity
    let pair1 = Hand::from_str(Holdem, "Ah Ac Kh Qc Js 2c 3d")?;
    let pair2 = Hand::from_str(Holdem, "Ah Ac Kh Qc Js 4s 5d")?;
    let lower_pair = Hand::from_str(Holdem, "Kh Kc Ah Qc Js 2c 3d")?;
    assert_eq!(pair1.evaluate(), pair2.evaluate());
    assert!(pair1 == pair2);
    assert!(pair1 > lower_pair);

    // Test high card equality and transitivity
    let high1 = Hand::from_str(Holdem, "Ah Kc Qh Jc 9c 2s 3d")?;
    let high2 = Hand::from_str(Holdem, "Ah Kc Qh Jc 9c 4s 5d")?;
    let lower_high = Hand::from_str(Holdem, "Kh Qc Jh 8c 9c 2s 3d")?;
    assert_eq!(high1.evaluate(), high2.evaluate());
    assert!(high1 == high2);
    assert!(high1 > lower_high);

    Ok(())
}

#[test]
fn test_hand_comparisons() -> Result<(), PokerError> {
    // Test different hand types against each other
    let royal = Hand::from_str(Holdem, "Ah Kh Qh Jh Th 2c 3d")?;
    let quads = Hand::from_str(Holdem, "Ah Ac Ad As Kh Qc Jd")?;
    let boat = Hand::from_str(Holdem, "Ah Ac Ad Kh Kc Qs Jd")?;
    let flush = Hand::from_str(Holdem, "Ah Kh Qh Jh 9h 2c 3d")?;
    let straight = Hand::from_str(Holdem, "Ah Kc Qd Js Th 2c 3d")?;
    let trips = Hand::from_str(Holdem, "Ah Ac Ad Kh Qc Js 2d")?;
    let two_pair = Hand::from_str(Holdem, "Ah Ac Kh Kc Qc Js 2d")?;
    let pair = Hand::from_str(Holdem, "Ah Ac Kh Qc Js 9c 2d")?;
    let high_card = Hand::from_str(Holdem, "Ah Kc Qh Jc 9c 7c 2d")?;

    assert!(royal > quads);
    assert!(quads > boat);
    assert!(boat > flush);
    assert!(flush > straight);
    assert!(straight > trips);
    assert!(trips > two_pair);
    assert!(two_pair > pair);
    assert!(pair > high_card);

    Ok(())
}

#[test]
fn test_empty_hand_comparison() -> Result<(), PokerError> {
    // First, test direct construction of incomplete Hold'em hand rankings.
    let empty_rank = HoldemHandRank::Incomplete(0);
    let one_card_rank = HoldemHandRank::Incomplete(1);

    // Incomplete hands with the same card count should be equal.
    assert_eq!(empty_rank.partial_cmp(&empty_rank), Some(Ordering::Equal));
    // An incomplete hand with more cards should beat one with fewer cards.
    assert_eq!(empty_rank.partial_cmp(&one_card_rank), Some(Ordering::Less));
    assert_eq!(
        one_card_rank.partial_cmp(&empty_rank),
        Some(Ordering::Greater)
    );

    // Now test the evaluation of hands using Hand::from_str.
    // These functions now evaluate to an Incomplete variant when not enough cards are present.
    let empty_hand1 = Hand::from_str(Holdem, "")?;
    let empty_hand2 = Hand::from_str(Holdem, "")?;
    let one_card_hand = Hand::from_str(Holdem, "2h")?;

    // Here we test that evaluating these hands produces the correct Incomplete result.
    match empty_hand1.evaluate() {
        HoldemHandRank::Incomplete(n) => {
            assert_eq!(n, 0, "Empty hand should evaluate to Incomplete(0)");
        }
        r => panic!("Expected Incomplete(0) for an empty hand, got {:?}", r),
    }
    match one_card_hand.evaluate() {
        HoldemHandRank::Incomplete(n) => {
            assert_eq!(n, 1, "A one-card hand should evaluate to Incomplete(1)");
        }
        r => panic!("Expected Incomplete(1) for a one-card hand, got {:?}", r),
    }

    // Finally, compare Hand instances.
    // Empty hands should be equal.
    assert_eq!(empty_hand1, empty_hand2);
    assert!(!(empty_hand1 > empty_hand2));
    assert!(!(empty_hand1 < empty_hand2));

    // A hand with a card should compare as greater than an empty hand.
    assert!(one_card_hand > empty_hand1);
    assert!(empty_hand1 < one_card_hand);

    Ok(())
}
