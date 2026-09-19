use crate::{
    cards::Rank,
    error::{CardError, PokerError},
    hand::Hand,
    variants::{high_score, FiveCardDraw, HighHandRank, PokerVariant},
};

/// Five cards is a whole hand, and a sixth is refused.
#[test]
fn test_a_hand_holds_five_cards() -> Result<(), PokerError> {
    assert_eq!(Hand::from_str(FiveCardDraw, "AhKdQc2s3d")?.num_cards(), 5);
    assert!(matches!(
        Hand::from_str(FiveCardDraw, "AhKdQc2s3d4h").unwrap_err(),
        PokerError::Card(CardError::TooManyCards(6))
    ));
    Ok(())
}

/// The ranking is the ordinary high one, best hand last.
#[test]
fn test_hands_rank_high() -> Result<(), PokerError> {
    let ladder = [
        "Ah Kd 9c 5s 3d", // ace high
        "2h 2d 9c 5s 3d", // a pair
        "2h 2d 3c 3s 5d", // two pair
        "2h 2d 2c 5s 3d", // trips
        "Ah 2d 3c 4s 5d", // the wheel, a straight
        "Th Jd Qc Ks Ad", // broadway
        "2h 4h 6h 8h Th", // a flush
        "2h 2d 2c 3s 3d", // a full house
        "2h 2d 2c 2s 3d", // quads
        "Ah 2h 3h 4h 5h", // the steel wheel
        "Th Jh Qh Kh Ah", // a royal flush
    ];
    for pair in ladder.windows(2) {
        let lower = Hand::from_str(FiveCardDraw, pair[0])?;
        let higher = Hand::from_str(FiveCardDraw, pair[1])?;
        assert!(higher > lower, "{} should beat {}", pair[1], pair[0]);
    }
    Ok(())
}

/// Names come from the high evaluator, and a short hand is incomplete.
#[test]
fn test_hands_are_named_as_high_hands() -> Result<(), PokerError> {
    assert_eq!(
        Hand::from_str(FiveCardDraw, "Ah 2d 3c 4s 5d")?.evaluate(),
        HighHandRank::Straight(Rank::Five)
    );
    assert_eq!(
        Hand::from_str(FiveCardDraw, "Kh Kd 7c 7s 2d")?.evaluate(),
        HighHandRank::TwoPair(Rank::King, Rank::Seven, Rank::Two)
    );
    assert_eq!(
        Hand::from_str(FiveCardDraw, "Kh Kd 7c")?.evaluate(),
        HighHandRank::Incomplete(3)
    );
    Ok(())
}

/// The score the sampler compares is the high table's, lower being better.
#[test]
fn test_score_is_the_high_score() -> Result<(), PokerError> {
    let royal = Hand::from_str(FiveCardDraw, "Th Jh Qh Kh Ah")?;
    let flush = Hand::from_str(FiveCardDraw, "2h 4h 6h 8h Th")?;
    assert_eq!(
        FiveCardDraw.score(royal.cards()),
        high_score(royal.cards()) as u32
    );
    assert!(FiveCardDraw.score(royal.cards()) < FiveCardDraw.score(flush.cards()));
    Ok(())
}
