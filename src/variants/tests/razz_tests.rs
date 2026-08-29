use std::cmp::Ordering;

use crate::{
    cards::Rank,
    error::PokerError,
    hand::Hand,
    variants::{Razz, RazzHandRank},
};

#[test]
fn test_hand_ranks() -> Result<(), PokerError> {
    // Test ascending ordering (after duplicates removed)
    let hand = Hand::from_str(Razz, "Ah 2h 3h 4h 5h 6h 7h")?;
    let RazzHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks.len(), 5, "Should keep only 5 ranks");
    assert_eq!(ranks[0], Rank::Five);
    assert_eq!(ranks[1], Rank::Four);
    assert_eq!(ranks[2], Rank::Three);
    assert_eq!(ranks[3], Rank::Two);
    assert_eq!(ranks[4], Rank::Ace);

    // Only four ranks to work with, so one of them must be played twice.
    // The lowest available pair is chosen, leaving 5-4-3 as the kickers.
    let hand = Hand::from_str(Razz, "2h 2d 3h 3d 4h 4d 5h")?;
    let RazzHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks.len(), 5, "Always plays five cards");
    assert_eq!(ranks[0], Rank::Five);
    assert_eq!(ranks[1], Rank::Four);
    assert_eq!(ranks[2], Rank::Three);
    assert_eq!(ranks[3], Rank::Two);
    assert_eq!(ranks[4], Rank::Two);

    // Only three ranks. Two pair beats trips, so 2-2-3-3-4 is played rather
    // than 2-2-2-3-4.
    let hand = Hand::from_str(Razz, "2h 2d 2c 3h 3d 3c 4h")?;
    let RazzHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks.len(), 5, "Always plays five cards");
    assert_eq!(ranks[0], Rank::Four);
    assert_eq!(ranks[1], Rank::Three);
    assert_eq!(ranks[2], Rank::Three);
    assert_eq!(ranks[3], Rank::Two);
    assert_eq!(ranks[4], Rank::Two);

    // Test high cards (K, Q, J)
    let hand = Hand::from_str(Razz, "Kh Qh Jh Th 9h")?;
    let RazzHandRank::Low(ranks) = hand.evaluate();
    assert_eq!(ranks[0], Rank::King);
    assert_eq!(ranks[1], Rank::Queen);
    assert_eq!(ranks[2], Rank::Jack);
    assert_eq!(ranks[3], Rank::Ten);
    assert_eq!(ranks[4], Rank::Nine);

    // Test typical low cards with Ace
    let hand = Hand::from_str(Razz, "5h 4h 3h 2h Ah")?;
    let RazzHandRank::Low(ranks) = hand.evaluate();
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

    // A complete hand beats an incomplete one, even a paired complete hand.
    // These two share their distinct ranks, but only the first plays five
    // cards.
    let hand7 = Hand::from_str(Razz, "2h 2d 3c 3s 4h")?; // Plays 4-3-3-2-2
    let hand8 = Hand::from_str(Razz, "2c 3h 4d")?; // Only three cards
    assert!(
        hand7 > hand8,
        "a five-card low beats a three-card holding"
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

#[test]
fn test_empty_hand_comparison() -> Result<(), PokerError> {
    // Test empty HandRanks
    let empty1 = RazzHandRank::Low(vec![]);
    let empty2 = RazzHandRank::Low(vec![]);
    let some_hand = RazzHandRank::Low(vec![Rank::Two]);

    // Empty hands should be equal
    assert_eq!(empty1.partial_cmp(&empty2), Some(Ordering::Equal));

    // Empty hand should be less than any non-empty hand
    assert_eq!(empty1.partial_cmp(&some_hand), Some(Ordering::Less));
    assert_eq!(some_hand.partial_cmp(&empty1), Some(Ordering::Greater));

    // Test empty Hand instances
    let empty_hand1 = Hand::from_str(Razz, "")?;
    let empty_hand2 = Hand::from_str(Razz, "")?;
    let one_card_hand = Hand::from_str(Razz, "2h")?;

    // Empty hands should be equal
    assert_eq!(empty_hand1, empty_hand2);
    assert!(!(empty_hand1 > empty_hand2));
    assert!(!(empty_hand1 < empty_hand2));

    // Any hand with cards should beat an empty hand
    assert!(one_card_hand > empty_hand1);
    assert!(empty_hand1 < one_card_hand);

    Ok(())
}

/// Paired lows are ranked among themselves by pair rank and then kickers, all
/// reversed so that lower wins. The two examples are the ones given by
/// Wikipedia's "Lowball (poker)" and Upswing's lowball rankings.
#[test]
fn test_paired_lows_rank_by_pair_then_kickers() -> Result<(), PokerError> {
    // Same pair, so the kickers decide: 6-4-2 is lower than 6-5-A.
    let lower_kicker = Hand::from_str(Razz, "3h 3d 6c 4s 2h")?;
    let higher_kicker = Hand::from_str(Razz, "3c 3s 6d 5h Ac")?;
    assert!(
        lower_kicker > higher_kicker,
        "3-3-6-4-2 beats 3-3-6-5-A"
    );

    // Different pairs, so the pair decides. The ace is the lowest card, so a
    // pair of aces is the lowest pair there is.
    let pair_of_aces = Hand::from_str(Razz, "Ah Ad 9c 5s 3h")?;
    let pair_of_deuces = Hand::from_str(Razz, "2h 2d 5c 4s 3d")?;
    assert!(
        pair_of_aces > pair_of_deuces,
        "A-A-9-5-3 beats 2-2-5-4-3"
    );

    // ...but both lose to any hand with no pair at all, however high.
    let no_pair = Hand::from_str(Razz, "Kh Jd 8c 6s 4h")?;
    assert!(no_pair > pair_of_aces, "K-J-8-6-4 beats a pair of aces");
    assert!(no_pair > pair_of_deuces, "K-J-8-6-4 beats a pair of deuces");

    Ok(())
}

/// Two holdings can share their distinct ranks and still not be equal: which
/// duplicate each is forced to play decides the pot. Reducing a low to its
/// distinct ranks made these two chop.
#[test]
fn test_same_distinct_ranks_are_not_a_tie() -> Result<(), PokerError> {
    // Both hold only 2, 3, 4 and 5, so both must play a pair.
    let pair_of_deuces = Hand::from_str(Razz, "2c 2d 2h 3c 3d 4c 5d")?; // plays 5-4-3-2-2
    let pair_of_fours = Hand::from_str(Razz, "4h 4s 4d 5h 5s 3h 2s")?; // plays 5-4-4-3-2

    assert_eq!(pair_of_deuces.evaluate().ranks().len(), 5);
    assert_eq!(pair_of_fours.evaluate().ranks().len(), 5);
    assert!(
        pair_of_deuces > pair_of_fours,
        "a pair of deuces beats a pair of fours: {} against {}",
        pair_of_deuces.evaluate(),
        pair_of_fours.evaluate()
    );

    Ok(())
}
