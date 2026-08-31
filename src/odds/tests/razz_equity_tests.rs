//! Razz, end to end.
//!
//! Ace-to-five lowball dealt like stud: seven cards a seat, best five count,
//! straights and flushes are ignored and the ace is low. The wheel `A2345` is
//! the best hand there is.
//!
//! The figures came from the tests of the older engine, which measured each
//! over a hundred thousand deals. Every one was confirmed here, and where the
//! space could be walked the exact value has replaced the measured one.

use super::support::{assert_sampled, assert_walked};
use crate::{
    error::{EquityError, PokerError},
    odds::EquityRequest,
    variants::Razz,
};

/// A razz spot, with cards that are out of play named separately.
fn spot(hands: &[&str], dead: &str) -> Result<EquityRequest<Razz>, PokerError> {
    EquityRequest::from_text(Razz, hands, "", dead)
}

/// What the engine refuses before dealing anything.
#[test]
fn test_a_bad_razz_request_is_refused() -> Result<(), PokerError> {
    assert!(matches!(
        spot(&["AsKsQs"], ""),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Razz deals street by street, but it deals to the whole table at once,
    // so two seats holding different numbers of cards is a miscount.
    assert!(matches!(
        spot(&["AsKsQs", "AdKd"], ""),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // A card cannot sit in two hands.
    assert!(spot(&["AsKsQs", "AsKdQd"], "").is_err());

    Ok(())
}

/// Spots settled or nearly settled, where every deal can be walked.
#[test]
fn test_razz_equities_that_can_be_walked() -> Result<(), PokerError> {
    // A made wheel against a king-high draw, two cards to come each. The
    // wheel cannot be beaten, only tied, and the king can only get there by
    // pairing nothing and drawing two more wheel cards of its own.
    assert_walked(
        &spot(&["Ah2h3h4h5h", "Kh2d3d4d5d"], "")?,
        &[93.031359, 6.968641],
        "a made wheel against a king",
    )?;

    // The same five ranks in different suits. Razz ignores suits, so swapping
    // hearts for diamonds maps one hand onto the other and neither can hold
    // an edge -- exactly half each, and not as a matter of measurement.
    assert_walked(
        &spot(&["2h3h4h5h6h", "2d3d4d5d6d"], "")?,
        &[50.0, 50.0],
        "the same six-low twice",
    )?;

    // Two complete hands over the same four distinct ranks. This is not a
    // chop: both play a pair, and the lower pair wins outright -- 5-4-3-2-2
    // beats 5-4-4-3-2.
    assert_walked(
        &spot(&["2c2d2h3c3d4c5d", "4h4s4d5h5s3h2s"], "")?,
        &[100.0, 0.0],
        "a pair of deuces against a pair of fours",
    )?;

    Ok(())
}

/// Third street, with four cards still to come for each seat.
#[test]
fn test_razz_equities_on_third_street() -> Result<(), PokerError> {
    // Three to a wheel against the two next-best starts. The best hand is the
    // favourite and the gap between second and third is far wider than the
    // gap between first and second.
    assert_sampled(
        &spot(&["Ah2h3h", "4d5d6d", "7c8c9c"], "")?,
        &[(43.809, 0.55), (38.827, 0.54), (17.364, 0.42)],
        "A23 against 456 against 789",
    )?;

    // The same start heads-up against a 567. Three ranks better is worth
    // about five points here, which is less than it looks: four cards are
    // still to come and either hand can pair.
    assert_sampled(
        &spot(&["Ah2h3h", "5h6h7h"], "")?,
        &[(55.361, 0.56), (44.639, 0.56)],
        "A23 against 567",
    )?;

    // A23 against 456, with the other ace, deuce and trey already gone. The
    // dead cards hurt the hand that wanted to pair its own low ranks least,
    // and A23 goes from a small favourite to a large one.
    assert_sampled(
        &spot(&["Ah2h3h", "4h5h6h"], "Ad2d3d")?,
        &[(61.912, 0.54), (38.088, 0.54)],
        "A23 against 456 with the low cards thinning out",
    )?;

    Ok(())
}
