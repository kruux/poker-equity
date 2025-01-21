use crate::{
    cards::Rank,
    error::PokerError,
    hand::Hand,
    variants::{HasLow, HighHandRank, LowHandRank, StudHiLo},
};

#[test]
fn test_straight_flush() {
    // Test straight flush (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "9h Th Jh Qh Kh 2c 3c").unwrap();
    let rank = hand.evaluate();
    assert_eq!(rank.high(), &HighHandRank::StraightFlush(Rank::King));
    assert!(rank.low().is_none()); // Too high for a low hand

    // Test wheel straight flush with low
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "5h 4h 3h 2h Ah 6c 7d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(rank.high(), &HighHandRank::StraightFlush(Rank::Five));
    if let Some(LowHandRank::Low(low_ranks)) = rank.low() {
        assert_eq!(low_ranks.len(), 5);
        assert_eq!(low_ranks[4], Rank::Ace); // Ace is best for low
    } else {
        panic!("Should have a low hand!");
    }
}

#[test]
fn test_four_of_a_kind() {
    // High quads (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah As Ad Ac Kh Qc Jd").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::FourOfAKind(Rank::Ace, Rank::King)
    );
    assert!(rank.low().is_none()); // Too many of same rank for low

    // Low quads (still no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2s 2d 2c 3h 4c 5d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::FourOfAKind(Rank::Two, Rank::Five)
    );
    assert!(rank.low().is_none()); // Too many of same rank for low
}

#[test]
fn test_full_house() {
    // High full house (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah As Ad Kh Ks 2c 3d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(rank.high(), &HighHandRank::FullHouse(Rank::Ace, Rank::King));
    assert!(rank.low().is_none()); // Too many of same rank for low

    // Low full house (still no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2s 2d 3h 3s 4c 5d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::FullHouse(Rank::Two, Rank::Three)
    );
    assert!(rank.low().is_none()); // Too many of same rank for low
}

#[test]
fn test_flush() {
    // High flush (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah Kh Qh Jh 9h 2c 3d").unwrap();
    let rank = hand.evaluate();
    if let HighHandRank::Flush(ranks) = rank.high() {
        assert_eq!(ranks[0], Rank::Ace); // Best flush card
    } else {
        panic!("Expected a flush!");
    }
    assert!(rank.low().is_none()); // No low with such high cards

    // Flush with low
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 4h 6h 8h Ah 3c 5d").unwrap();
    let rank = hand.evaluate();
    if let HighHandRank::Flush(ranks) = rank.high() {
        assert_eq!(ranks[0], Rank::Ace);
    } else {
        panic!("Expected a flush!");
    }
    if let Some(LowHandRank::Low(low_ranks)) = rank.low() {
        assert_eq!(low_ranks.len(), 5);
        assert_eq!(low_ranks[4], Rank::Ace); // Ace is best for low
    } else {
        panic!("Should have a low hand!");
    }
}

#[test]
fn test_straight() {
    // Broadway straight (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Th Jc Qd Ks Ah 2c 3d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(rank.high(), &HighHandRank::Straight(Rank::Ace));
    assert!(rank.low().is_none()); // Too high for a low hand

    // Wheel straight with low
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ac 2d 3h 4s 5c 7h 8d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(rank.high(), &HighHandRank::Straight(Rank::Five)); // Wheel
    if let Some(LowHandRank::Low(low_ranks)) = rank.low() {
        assert_eq!(low_ranks.len(), 5);
        assert_eq!(low_ranks[4], Rank::Ace);
    } else {
        panic!("Should have a low hand!");
    }
}

#[test]
fn test_three_of_a_kind() {
    // High trips (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah Ad Ac Kh Qs Js 9d").unwrap();
    let rank = hand.evaluate();
    if let HighHandRank::ThreeOfAKind(rank, kickers) = rank.high() {
        assert_eq!(*rank, Rank::Ace);
        assert_eq!(*kickers, vec![Rank::King, Rank::Queen]);
    } else {
        panic!("Expected ThreeOfAKind, got different hand rank");
    }
    assert!(rank.low().is_none()); // Too high for a low hand

    // Low trips (no low possible due to trips)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2d 2c Ah Kd Qs Js").unwrap();
    let rank = hand.evaluate();
    if let HighHandRank::ThreeOfAKind(rank, kickers) = rank.high() {
        assert_eq!(*rank, Rank::Two);
        assert_eq!(*kickers, vec![Rank::Ace, Rank::King]);
    } else {
        panic!("Expected ThreeOfAKind, got different hand rank");
    }

    assert!(rank.low().is_none()); // Too many deuces for a low
}

#[test]
fn test_two_pair() {
    // High two pair (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah Ac Kh Kc Qc Js 9d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::TwoPair(Rank::Ace, Rank::King, Rank::Queen)
    );
    assert!(rank.low().is_none()); // Too high for a low hand

    // Low two pair with low hand
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2c 3h 3c Ah 4d 6s").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::TwoPair(Rank::Three, Rank::Two, Rank::Ace)
    );
    if let Some(LowHandRank::Low(low_ranks)) = rank.low() {
        assert_eq!(low_ranks.len(), 5);
        assert_eq!(low_ranks[4], Rank::Ace);
    } else {
        panic!("Should have a low hand!");
    }
}

#[test]
fn test_one_pair() {
    // High pair (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah 9c Kh Qc Js 7d 9c").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::Pair(Rank::Nine, vec![Rank::Ace, Rank::King, Rank::Queen])
    );
    assert!(rank.low().is_none()); // Too high for a low hand

    // Low pair with low hand
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2c 3h 4c 5d 7s 8d").unwrap();
    let rank = hand.evaluate();
    assert_eq!(
        rank.high(),
        &HighHandRank::Pair(Rank::Two, vec![Rank::Eight, Rank::Seven, Rank::Five])
    );
    if let Some(LowHandRank::Low(low_ranks)) = rank.low() {
        assert_eq!(low_ranks.len(), 5);
        assert_eq!(low_ranks[0], Rank::Seven);
    } else {
        panic!("Should have a low hand!");
    }
}

#[test]
fn test_high_card() {
    // High cards only (no low possible)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah Kc Qd Js 3h 9c 8d").unwrap();
    let rank = hand.evaluate();
    if let HighHandRank::HighCard(ranks) = rank.high() {
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[1], Rank::King);
        assert_eq!(ranks[2], Rank::Queen);
        assert_eq!(ranks[3], Rank::Jack);
        assert_eq!(ranks[4], Rank::Nine);
    } else {
        panic!("Expected high card!");
    }
    assert!(rank.low().is_none()); // No low possible with these cards

    // High card with good low
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah 2c 3d 4s 6h 7c 8d").unwrap();
    let rank = hand.evaluate();
    if let HighHandRank::HighCard(ranks) = rank.high() {
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[1], Rank::Eight);
        assert_eq!(ranks[2], Rank::Seven);
        assert_eq!(ranks[3], Rank::Six);
        assert_eq!(ranks[4], Rank::Four);
    } else {
        panic!("Expected high card!");
    }
    // Should qualify for low: A,2,3,4,6
    if let Some(LowHandRank::Low(low_ranks)) = rank.low() {
        assert_eq!(low_ranks.len(), 5);
        assert_eq!(low_ranks[4], Rank::Ace);
        assert_eq!(low_ranks[3], Rank::Two);
        assert_eq!(low_ranks[2], Rank::Three);
        assert_eq!(low_ranks[1], Rank::Four);
        assert_eq!(low_ranks[0], Rank::Six);
    } else {
        panic!("Should have a low hand!");
    }
}

#[test]
fn test_hand_comparison() {
    // Create a variety of hands with and without low possibilities

    // Best possible - Royal flush (no low)
    let royal = Hand::<StudHiLo>::from_str(StudHiLo, "Ah Kh Qh Jh Th 2c 3c").unwrap();

    // Straight flush with low
    let sf_with_low = Hand::<StudHiLo>::from_str(StudHiLo, "5h 4h 3h 2h Ah 6c 7d").unwrap();

    // Quads (no low possible)
    let quads = Hand::<StudHiLo>::from_str(StudHiLo, "Ah As Ad Ac 2h 3c 4d").unwrap();

    // Full house (no low possible)
    let full_house = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2s 2d 3h 3c 4d 5s").unwrap();

    // Flush with low
    let flush_with_low = Hand::<StudHiLo>::from_str(StudHiLo, "2h 3h 4h 5h 7h Ac 8d").unwrap();

    // Straight with low (wheel)
    let wheel = Hand::<StudHiLo>::from_str(StudHiLo, "Ah 2c 3d 4s 5h 6c 7d").unwrap();

    // Just a low hand
    let low_only = Hand::<StudHiLo>::from_str(StudHiLo, "Ah 2c 3d 4s 5h 8c Td").unwrap();

    // Verify hand rankings respect traditional high hand order
    assert!(royal > sf_with_low);
    assert!(sf_with_low > quads);
    assert!(quads > full_house);
    assert!(full_house > flush_with_low);
    assert!(flush_with_low > wheel);
    assert!(wheel > low_only);

    // Verify that having a low doesn't affect high hand rankings
    assert!(sf_with_low > quads);
    assert!(wheel > low_only);
}

#[test]
fn test_low_hand_qualification() {
    // Test perfect low
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah 2c 3d 4s 5h 7c 8d").unwrap();
    assert!(hand.evaluate().low().is_some());

    // Test no low (all high cards)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Th Jc Qd Ks Ah 9c 8d").unwrap();
    println!("Low: {:?}", hand.evaluate().low());
    assert!(hand.evaluate().low().is_none());

    // Test almost qualifying (only 4 cards under 8)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "Ah 2c 3d 4s 9h Tc Jd").unwrap();
    assert!(hand.evaluate().low().is_none());

    // Test not qualifying due to pairs
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "2h 2c 3d 4s 5h 6c 7d").unwrap();
    assert!(hand.evaluate().low().is_some());

    // Test highest possible qualifying low (8,7,6,5,4)
    let hand = Hand::<StudHiLo>::from_str(StudHiLo, "8h 7c 6d 5s 4h 9c Td").unwrap();
    if let Some(LowHandRank::Low(ranks)) = hand.evaluate().low() {
        assert_eq!(ranks[0], Rank::Eight);
    } else {
        panic!("Should qualify as a low hand!");
    }
}

#[test]
fn test_hand_equality() -> Result<(), PokerError> {
    // Straight flush - both King high in hearts
    let sf1 = Hand::from_str(StudHiLo, "Kh Qh Jh Th 9h")?;
    let sf2 = Hand::from_str(StudHiLo, "Kh Qh Jh Th 9h")?;
    assert_eq!(sf1, sf2);

    // Four of a kind and full house impossible to have equal hands
    // since all cards of that rank would be used

    // Flush - AKQ73 hearts
    let flush1 = Hand::from_str(StudHiLo, "Ah Kh Qh 7h 3h")?;
    let flush2 = Hand::from_str(StudHiLo, "Ah Kh Qh 7h 3h")?;
    let different_flush = Hand::from_str(StudHiLo, "Ah Kh Qh 7h 4h")?; // Different lowest card
    assert_eq!(flush1, flush2);
    assert_ne!(flush1, different_flush);

    // Straight - both 9 high
    let straight1 = Hand::from_str(StudHiLo, "9h 8c 7d 6s 5h")?;
    let straight2 = Hand::from_str(StudHiLo, "9c 8h 7s 6h 5c")?;
    let different_straight = Hand::from_str(StudHiLo, "Th 9c 8d 7s 6h")?; // Higher straight
    assert_eq!(straight1, straight2);
    assert_ne!(straight1, different_straight);

    // Three of a kind - trips 8s with AK kickers
    let trips1 = Hand::from_str(StudHiLo, "8h 8c 8d Ah Kc")?;
    let trips2 = Hand::from_str(StudHiLo, "8h 8s 8d Ac Kh")?;
    let different_trips = Hand::from_str(StudHiLo, "8h 8c 8d Ah Qc")?; // Different kicker
    assert_eq!(trips1, trips2);
    assert_ne!(trips1, different_trips);

    // Two pair - Jacks and twos with Ace kicker
    let two_pair1 = Hand::from_str(StudHiLo, "Jh Jc 2h 2c Ad")?;
    let two_pair2 = Hand::from_str(StudHiLo, "Js Jd 2d 2s Ah")?;
    let different_two_pair = Hand::from_str(StudHiLo, "Jh Jc 2h 2c Kd")?; // Different kicker
    assert_eq!(two_pair1, two_pair2);
    assert_ne!(two_pair1, different_two_pair);

    // Pair - eights with AQJ kickers
    let pair1 = Hand::from_str(StudHiLo, "8h 8c Ah Qc Jd")?;
    let pair2 = Hand::from_str(StudHiLo, "8d 8s Ac Qh Js")?;
    let different_pair = Hand::from_str(StudHiLo, "8h 8c Ah Qc Td")?; // Different kicker
    assert_eq!(pair1, pair2);
    assert_ne!(pair1, different_pair);

    // High card - AKQ75
    let high1 = Hand::from_str(StudHiLo, "Ah Kc Qd 7s 5h")?;
    let high2 = Hand::from_str(StudHiLo, "As Kh Qc 7d 5c")?;
    let different_high = Hand::from_str(StudHiLo, "Ah Kc Qd 7s 6h")?; // Different lowest card
    assert_eq!(high1, high2);
    assert_ne!(high1, different_high);

    Ok(())
}
