use crate::{
    cards::Rank,
    error::PokerError,
    hand::Hand,
    variants::{LowHandRank, Razz},
};

#[test]
fn test_hand_ranks() -> Result<(), PokerError> {
    // Test ascending ordering (after duplicates removed)
    let hand = Hand::from_str(Razz, "Ah 2h 3h 4h 5h 6h 7h")?;
    let LowHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks.len(), 5, "Should keep only 5 ranks");
    assert_eq!(ranks[0], Rank::Five);
    assert_eq!(ranks[1], Rank::Four);
    assert_eq!(ranks[2], Rank::Three);
    assert_eq!(ranks[3], Rank::Two);
    assert_eq!(ranks[4], Rank::Ace);

    // Test with duplicate ranks
    let hand = Hand::from_str(Razz, "2h 2d 3h 3d 4h 4d 5h")?;
    let LowHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks.len(), 4, "Should have 4 unique ranks");
    assert_eq!(ranks[0], Rank::Five);
    assert_eq!(ranks[1], Rank::Four);
    assert_eq!(ranks[2], Rank::Three);
    assert_eq!(ranks[3], Rank::Two);

    // Test with multiple duplicates of the same rank
    let hand = Hand::from_str(Razz, "2h 2d 2c 3h 3d 3c 4h")?;
    let LowHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks.len(), 3, "Should have 3 unique ranks");
    assert_eq!(ranks[0], Rank::Four);
    assert_eq!(ranks[1], Rank::Three);
    assert_eq!(ranks[2], Rank::Two);

    // Test high cards (K, Q, J)
    let hand = Hand::from_str(Razz, "Kh Qh Jh Th 9h")?;
    let LowHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks[0], Rank::King);
    assert_eq!(ranks[1], Rank::Queen);
    assert_eq!(ranks[2], Rank::Jack);
    assert_eq!(ranks[3], Rank::Ten);
    assert_eq!(ranks[4], Rank::Nine);

    // Test typical low cards with Ace
    let hand = Hand::from_str(Razz, "5h 4h 3h 2h Ah")?;
    let LowHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks[0], Rank::Five);
    assert_eq!(ranks[1], Rank::Four);
    assert_eq!(ranks[2], Rank::Three);
    assert_eq!(ranks[3], Rank::Two);
    assert_eq!(ranks[4], Rank::Ace);

    Ok(())
}

#[test]
fn test_hand_comparisons() -> Result<(), PokerError> {
    // Test where all cards match except last one
    let hand1 = Hand::from_str(Razz, "9h 8d 7c 6s Ah")?;
    let hand2 = Hand::from_str(Razz, "9c 8h 7d 6c 2s")?;
    assert!(hand1 > hand2, "9876A should beat 98762");

    // Test that 5 card hands beat 4 card hands
    let hand3 = Hand::from_str(Razz, "Kh Kd 7c As 5h Jh")?; // Becomes KJ75A
    let hand4 = Hand::from_str(Razz, "2c 3h 4d 5c")?; // Becomes 5432
    assert!(hand3 > hand4, "5 card hand should beat 4 card hand");

    // Test duplicates are properly removed
    let hand5 = Hand::from_str(Razz, "2h 2d 2c As Kh")?; // Becomes K2A
    let hand6 = Hand::from_str(Razz, "3h 3d 4c 5s 6h")?; // Becomes 6543
    assert!(hand6 > hand5, "4 card hand should beat 3 card hand");

    // Test identical hands with different duplicate patterns
    let hand7 = Hand::from_str(Razz, "2h 2d 3c 3s 4h")?; // Becomes 432
    let hand8 = Hand::from_str(Razz, "2c 3h 4d")?; // Becomes 432
    assert_eq!(
        hand7.evaluate(),
        hand8.evaluate(),
        "Same ranks should be equal"
    );

    Ok(())
}

#[test]
fn test_detailed_razz_ranking() -> Result<(), PokerError> {
    // Two unique card hands (worse because more pairs)
    let two_rank_high = Hand::from_str(Razz, "AhAcAdAs2h2c2d")?;
    let two_rank_high_diff = Hand::from_str(Razz, "AhAcAdAs2h2c2s")?;
    let two_rank_low = Hand::from_str(Razz, "AhAcAdAs3h3c3d")?;

    // Three unique card hands
    let three_rank_high = Hand::from_str(Razz, "AhAcAd2h2c3h")?;
    let three_rank_high_diff = Hand::from_str(Razz, "AhAcAd2h2c3s")?;
    let three_rank_low = Hand::from_str(Razz, "AhAcAd2h2c5s")?;

    // Four unique card hands
    let four_rank_high = Hand::from_str(Razz, "AhAc2h2c3d4s")?;
    let four_rank_high_diff = Hand::from_str(Razz, "AhAc2h2c3d4h")?;
    let four_rank_low = Hand::from_str(Razz, "AhAc2h2c3d6s")?;

    // Five unique card hands (best type of hand in Razz)
    let five_rank_high = Hand::from_str(Razz, "Ah2h3h4h5hKsQd")?; // Perfect low: A2345
    let five_rank_high_diff = Hand::from_str(Razz, "Ac2c3c4c5cJsTd")?;
    let five_rank_low = Hand::from_str(Razz, "Ah2h3h4h6hKsQd")?; // A2346

    // Test equal hands
    assert!(two_rank_high == two_rank_high_diff);
    assert!(three_rank_high == three_rank_high_diff);
    assert!(four_rank_high == four_rank_high_diff);
    assert!(five_rank_high == five_rank_high_diff);

    // Test within same rank count (remember in Razz lower is better)
    assert!(two_rank_high > two_rank_low); // AAAA222 > AAAA333
    assert!(three_rank_high > three_rank_low); // AAA22[3] > AAA22[5]
    assert!(four_rank_high > four_rank_low); // AA23[4] > AA23[6]
    assert!(five_rank_high > five_rank_low); // A2345 > A2346

    // Test between different rank counts (fewer duplicates is better)
    assert!(five_rank_high > four_rank_high); // 5 unique > 4 unique
    assert!(four_rank_high > three_rank_high); // 4 unique > 3 unique
    assert!(three_rank_high > two_rank_high); // 3 unique > 2 unique

    // Test transitivity
    assert!(five_rank_high > two_rank_high); // Best possible > Worst possible
    assert!(four_rank_high > two_rank_high); // Middle ranks transitive
    assert!(five_rank_high > three_rank_high); // Best beats middle

    Ok(())
}
