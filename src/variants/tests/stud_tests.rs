use crate::{
    cards::Rank,
    hand::Hand,
    variants::{HighHandRank, SevenCardStud},
};

#[test]
fn test_wheel_straight() {
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "As 2h 3d 4c 5h 7s 8d").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::Straight(Rank::Five)); // A2345 should be recognized
}

#[test]
fn test_best_of_seven() {
    // Two possible straights: 23456 and 34567. Should pick the higher one
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "2h 3d 4c 5h 6s 7s Ad").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::Straight(Rank::Seven));
}

#[test]
fn test_best_flush() {
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "2h 4h 6h 8h Th Jh 3s").unwrap();
    if let HighHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks[0], Rank::Jack); // Should pick highest 5 hearts
    } else {
        panic!("Expected a flush!");
    }
}

#[test]
fn test_not_straight_flush() {
    // This hand has:
    // - Flush in hearts: Ah Kh Qh Jh 2h
    // - Straight: 9c Td Je Qh Kh
    // But it's NOT a straight flush!
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh 2h 9c Td").unwrap();
    println!("Hand evaluates to: {}", hand.evaluate());
    // Current code would incorrectly say this is a straight flush
    // because it separately finds "there's a flush" and "there's a straight"
    assert_ne!(hand.evaluate(), HighHandRank::StraightFlush(Rank::King));

    // It should actually be evaluated as a flush
    if let HighHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks[0], Rank::Ace); // Highest heart
    } else {
        panic!("Should be a flush!");
    }
}

#[test]
fn test_actual_straight_flush() {
    // This is a genuine straight flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "9h Th Jh Qh Kh 2c 3c").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::StraightFlush(Rank::King));
}

#[test]
fn test_straight_flush() {
    // Regular straight flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "9h Th Jh Qh Kh 2c 3d").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::StraightFlush(Rank::King));

    // Wheel straight flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah 2h 3h 4h 5h Kc Qd").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::StraightFlush(Rank::Five));

    // A-6 straight flush (should be 6 high)
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah 2h 3h 4h 5h 6h Kd").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::StraightFlush(Rank::Six));

    // Not a straight flush (has flush and straight but not together)
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh 2h 9c Td").unwrap();
    assert!(!matches!(hand.evaluate(), HighHandRank::StraightFlush(_)));
}

#[test]
fn test_four_of_a_kind() {
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad As Kh Qc Jd").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FourOfAKind(Rank::Ace, Rank::King)
    );

    // Test with potential straight/flush doesn't override quads
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "7h 7c 7s 7d 8h 9h Th").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FourOfAKind(Rank::Seven, Rank::Ten)
    );
}

#[test]
fn test_full_house() {
    // Regular full house
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kc 2c 3d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FullHouse(Rank::Ace, Rank::King)
    );

    // Two sets of trips
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kc Kd Qc").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FullHouse(Rank::Ace, Rank::King)
    );

    // Trips and two pairs
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kc Qh Qc").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FullHouse(Rank::Ace, Rank::King)
    );
}

#[test]
fn test_flush() {
    // Basic flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh 9h 2c 3d").unwrap();
    if let HighHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[4], Rank::Nine); // Should take A K Q J 9
    } else {
        panic!("Should be a flush!");
    }

    // Six card flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "2h Kh Qh Jh 9h Ah 3d").unwrap();
    if let HighHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[4], Rank::Nine); // Should still take best 5
    } else {
        panic!("Should be a flush!");
    }

    // Seven card flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh 2h 7h 9h").unwrap();
    if let HighHandRank::Flush(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[4], Rank::Nine); // Should still take best 5
    } else {
        panic!("Should be a flush!");
    }
}

#[test]
fn test_straight() {
    // Regular straight
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "9c Th Jd Qh Kc 2c 3d").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::Straight(Rank::King));

    // Wheel straight (A2345)
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ac 2h 3d 4h 5c Kc Qd").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::Straight(Rank::Five));

    // A-6 straight
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ac 2h 3d 4h 5c 6d Kd").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::Straight(Rank::Six));

    // Multiple straights - should find highest
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "5c 6h 7d 8h 9c Th Jd").unwrap();
    assert_eq!(hand.evaluate(), HighHandRank::Straight(Rank::Jack));
}

#[test]
fn test_three_of_a_kind() {
    // Basic three of a kind
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Qh Jc 9c 2d").unwrap();
    if let HighHandRank::ThreeOfAKind(rank, kickers) = hand.evaluate() {
        assert_eq!(rank, Rank::Ace);
        assert_eq!(kickers, vec![Rank::Queen, Rank::Jack]);
    } else {
        panic!("Expected ThreeOfAKind, got different hand rank");
    }

    // if let DeuceSevenRank::Pair(rank, kickers) = pair_with_kickers.evaluate() {
    //     assert_eq!(rank, Rank::Seven);
    //     assert_eq!(kickers, vec![Rank::Ace, Rank::Five, Rank::Two]);
    // } else {
    //     panic!("Expected Pair, got different hand rank");
    // }

    // Three of a kind with potential straight draw
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "7h 7c 7d 8h 9c Tc 2d").unwrap();
    if let HighHandRank::ThreeOfAKind(rank, kickers) = hand.evaluate() {
        assert_eq!(rank, Rank::Seven);
        assert_eq!(kickers, vec![Rank::Ten, Rank::Nine]);
    } else {
        panic!("Expected ThreeOfAKind, got different hand rank");
    }

    // Three of a kind with potential flush draw
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "7h 7c 7d 2h 3h 4h 9c").unwrap();
    if let HighHandRank::ThreeOfAKind(rank, kickers) = hand.evaluate() {
        assert_eq!(rank, Rank::Seven);
        assert_eq!(kickers, vec![Rank::Nine, Rank::Four]);
    } else {
        panic!("Expected ThreeOfAKind, got different hand rank");
    }
}

#[test]
fn test_two_pair() {
    // Regular two pair
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac 5h 5c Qc Jc 2d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::Ace, Rank::Five, Rank::Queen)
    );

    // Three pairs - should take highest two
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Ah Ac Qh Qc 2d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::Ace, Rank::King, Rank::Queen)
    );

    // Pairs with multiple kicker options
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Qs Js Kh Kc 3d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::Ace, Rank::King, Rank::Queen)
    );
}

#[test]
fn test_pair() {
    // Basic pair
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Kh Qc Jc 9c 7d").unwrap();
    if let HighHandRank::Pair(rank, kickers) = hand.evaluate() {
        assert_eq!(rank, Rank::Ace);
        assert_eq!(kickers[0], Rank::King);
        assert_eq!(kickers.len(), 3);
    } else {
        panic!("Should be a pair!");
    }

    // Pair with potential straight draw
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "7h 7c 8h 9c Tc 2c 3d").unwrap();
    if let HighHandRank::Pair(rank, _) = hand.evaluate() {
        assert_eq!(rank, Rank::Seven);
    } else {
        panic!("Should be a pair!");
    }

    // Pair with potential flush draw
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "7h 7c 2h 3h 4h 9c Td").unwrap();
    if let HighHandRank::Pair(rank, _) = hand.evaluate() {
        assert_eq!(rank, Rank::Seven);
    } else {
        panic!("Should be a pair!");
    }
}

#[test]
fn test_high_card() {
    // Basic high card
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kc Qh Jc 9c 7c 2d").unwrap();
    if let HighHandRank::HighCard(ranks) = hand.evaluate() {
        assert_eq!(ranks.len(), 5);
        assert_eq!(ranks[0], Rank::Ace);
        assert_eq!(ranks[4], Rank::Nine); // Should take A K Q J 9
    } else {
        panic!("Should be high card!");
    }

    // High card with almost straight
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah 3c 4h 5c 6c 8c 9d").unwrap();
    if let HighHandRank::HighCard(ranks) = hand.evaluate() {
        assert_eq!(ranks[0], Rank::Ace);
    } else {
        panic!("Should be high card!");
    }

    // High card with almost flush
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah 3h 4h 5h 7c 8c 9d").unwrap();
    if let HighHandRank::HighCard(ranks) = hand.evaluate() {
        assert_eq!(ranks[0], Rank::Ace);
    } else {
        panic!("Should be high card!");
    }
}

#[test]
fn test_two_pair_edge_cases() {
    // Three pairs in descending order
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Qh Qc Jh Jc 2d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::King, Rank::Queen, Rank::Jack)
    );

    // Middle pair is highest
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Qh Qc Kh Kc Jh Jc 2d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::King, Rank::Queen, Rank::Jack)
    );

    // Lowest pair is highest
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Jh Jc Kh Kc Qh Qc 2d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::King, Rank::Queen, Rank::Jack)
    );

    // Multiple kicker options
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Qh Qc Ah 9c 7d").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::TwoPair(Rank::King, Rank::Queen, Rank::Ace)
    );
}

#[test]
fn test_full_house_edge_cases() {
    // Multiple trips available
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kc Kd Qc").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FullHouse(Rank::Ace, Rank::King)
    );

    // Multiple pairs available for the lower part
    let hand = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kc Qh Qc").unwrap();
    assert_eq!(
        hand.evaluate(),
        HighHandRank::FullHouse(Rank::Ace, Rank::King)
    );
}

#[test]
fn test_detailed_hand_ranking() {
    // Create groups of each hand type
    // Straight Flushes
    let sf_high = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh Th 2c 3d").unwrap();
    let sf_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh Th 4s 5d").unwrap();
    let sf_low = Hand::<SevenCardStud>::from_str(SevenCardStud, "9h 8h 7h 6h 5h 2c 3d").unwrap();

    // Four of a Kind
    let quad_high = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad As Kh 2c 3d").unwrap();
    let quad_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad As Ks 4c 5d").unwrap();
    let quad_low = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Kd Ks 2h 3c 4d").unwrap();

    // Full House
    let fh_high = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kc 2c 3d").unwrap();
    let fh_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Kd 4c 5d").unwrap();
    let fh_low = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Kd Qh Qc 2c 3d").unwrap();

    // Flush
    let flush_high =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh 9h 2c 3d").unwrap();
    let flush_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qh Jh 9h 4c 5d").unwrap();
    let flush_low = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Qh Jh Th 8h 2c 3d").unwrap();

    // Straight
    let straight_high =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kc Qd Js Th 2c 3d").unwrap();
    let straight_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kc Qd Js Th 4c 5d").unwrap();
    let straight_low =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Kc Qd Js Th 9h 2c 3d").unwrap();

    // Three of a Kind
    let trips_high =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Qc 2c 3d").unwrap();
    let trips_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Ad Kh Qc 4c 5d").unwrap();
    let trips_low = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Kd Qh Jc 2c 3d").unwrap();

    // Two Pair
    let two_pair_high =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Kh Kc Qc 2c 3d").unwrap();
    let two_pair_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Kh Kc Qc 4c 5d").unwrap();
    let two_pair_low =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Qh Qc Jh Jc Tc 2c 3d").unwrap();

    // One Pair
    let pair_high = Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Kh Qc Jc 2c 3d").unwrap();
    let pair_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Ac Kh Qc Jc 4c 5d").unwrap();
    let pair_low = Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Kc Qh Jc Tc 2c 3d").unwrap();

    // High Card
    let high_card_high =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qc Jc 9c 7c 3d").unwrap();
    let high_card_high_diff =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Ah Kh Qc Jc 9c 8c 4d").unwrap();
    let high_card_low =
        Hand::<SevenCardStud>::from_str(SevenCardStud, "Kh Qh Jc Tc 8c 7c 3d").unwrap();

    // Test equal hands
    assert_eq!(sf_high, sf_high_diff);
    assert_eq!(quad_high, quad_high_diff);
    assert_eq!(fh_high, fh_high_diff);
    assert_eq!(flush_high, flush_high_diff);
    assert_eq!(straight_high, straight_high_diff);
    assert_eq!(trips_high, trips_high_diff);
    assert_eq!(two_pair_high, two_pair_high_diff);
    assert_eq!(pair_high, pair_high_diff);
    assert_eq!(high_card_high, high_card_high_diff);

    // Test within same rank
    assert!(sf_high > sf_low);
    assert!(quad_high > quad_low);
    assert!(fh_high > fh_low);
    assert!(flush_high > flush_low);
    assert!(straight_high > straight_low);
    assert!(trips_high > trips_low);
    assert!(two_pair_high > two_pair_low);
    assert!(pair_high > pair_low);
    assert!(high_card_high > high_card_low);

    // Original tests for different hand types
    assert!(sf_high > quad_high);
    assert!(quad_high > fh_high);
    assert!(fh_high > flush_high);
    assert!(flush_high > straight_high);
    assert!(straight_high > trips_high);
    assert!(trips_high > two_pair_high);
    assert!(two_pair_high > pair_high);
    assert!(pair_high > high_card_high);

    // Test transitivity
    assert!(sf_high > high_card_high);
    assert!(quad_high > pair_high);
    assert!(fh_high > two_pair_high);
}
