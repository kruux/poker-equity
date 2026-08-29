use std::cmp::Ordering;

use crate::{
    error::PokerError,
    hand::Hand,
    variants::{Omaha, OmahaHandRank},
};

#[test]
fn test_empty_hand_comparison() -> Result<(), PokerError> {
    // Directly test incomplete Omaha hand rankings.
    let empty_rank = OmahaHandRank::Incomplete(0);
    let one_card_rank = OmahaHandRank::Incomplete(1);

    // Incomplete hands with the same card count should be equal.
    assert_eq!(empty_rank.partial_cmp(&empty_rank), Some(Ordering::Equal));
    // An incomplete hand with more cards beats one with fewer cards.
    assert_eq!(empty_rank.partial_cmp(&one_card_rank), Some(Ordering::Less));
    assert_eq!(
        one_card_rank.partial_cmp(&empty_rank),
        Some(Ordering::Greater)
    );

    // Now test by evaluating hands via Hand::from_str.
    // These functions should yield the appropriate Incomplete variant if there aren't enough cards.
    let empty_hand1 = Hand::from_str(Omaha, "")?;
    let empty_hand2 = Hand::from_str(Omaha, "")?;
    let one_card_hand = Hand::from_str(Omaha, "2h")?;

    // Evaluate and check that an empty hand yields Incomplete(0)
    match empty_hand1.evaluate() {
        OmahaHandRank::Incomplete(n) => {
            assert_eq!(n, 0, "Empty hand should evaluate to Incomplete(0)");
        }
        r => panic!("Expected Incomplete(0) for an empty hand, got {:?}", r),
    }

    // Evaluate and check that a one-card hand yields Incomplete(1)
    match one_card_hand.evaluate() {
        OmahaHandRank::Incomplete(n) => {
            assert_eq!(n, 1, "A one-card hand should evaluate to Incomplete(1)");
        }
        r => panic!("Expected Incomplete(1) for a one-card hand, got {:?}", r),
    }

    // Comparison of Hand instances.
    // Two empty evaluated hands should be equal.
    assert_eq!(empty_hand1, empty_hand2);
    assert!(!(empty_hand1 > empty_hand2));
    assert!(!(empty_hand1 < empty_hand2));

    // A hand with a card should beat an empty hand.
    assert!(one_card_hand > empty_hand1);
    assert!(empty_hand1 < one_card_hand);

    Ok(())
}
