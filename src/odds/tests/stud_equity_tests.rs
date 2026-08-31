//! Seven-card stud, end to end.
//!
//! Where both hands are named down to the last card or two, the space is
//! small enough to walk and the figure is exact -- 671,580 deals when two
//! seats each have two cards to come. Where a seat is still on third street
//! with four to come, the space is billions and the answer is sampled. How
//! that sampling is checked is in [`super::support`].
//!
//! The figures came from the tests of the older engine, which measured them
//! over a hundred thousand deals apiece. Every one was confirmed here, and
//! the exact value replaces the measured one wherever the space could be
//! walked.

use super::support::{assert_sampled, assert_walked};
use crate::{
    error::{EquityError, PokerError},
    odds::EquityRequest,
    variants::Stud,
};

/// A stud spot: no board and no dead cards, so the hands are the whole of it.
fn spot(hands: &[&str]) -> Result<EquityRequest<Stud>, PokerError> {
    EquityRequest::from_text(Stud, hands, "", "")
}

/// What the engine refuses before dealing anything.
#[test]
fn test_a_bad_stud_request_is_refused() -> Result<(), PokerError> {
    let build = |hands: &[&str]| spot(hands).map(|_| ());

    assert!(matches!(
        build(&["AsKsQs"]),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Every live player is on the same street, so fields must match.
    assert!(matches!(
        build(&["AsKsQs", "AdKd"]),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // Seven cards is the whole hand.
    assert!(build(&["AsKsQsJsTs9s8s7s", "AdKdQdJdTd9d8d7d"]).is_err());

    // A card cannot sit in two hands.
    assert!(build(&["AsKsQs", "AsKdQd"]).is_err());

    Ok(())
}

/// Hands complete or nearly complete, where every deal can be walked.
#[test]
fn test_stud_equities_that_can_be_walked() -> Result<(), PokerError> {
    // Seven cards each: nothing left to deal, and the royal flush wins.
    assert_walked(
        &spot(&["AhKhQhJhTh9s8s", "2c3c4c5c6c7d8d"])?,
        &[100.0, 0.0],
        "a made royal against a made straight flush",
    )?;

    // Two royal flushes already made, two cards to come each. Nothing either
    // player draws can change it, so they split every deal.
    assert_walked(
        &spot(&["AhKhQhJhTh", "AsKsQsJsTs"])?,
        &[50.0, 50.0],
        "two royal flushes",
    )?;

    // Quad aces against two pair at best: no runout saves the kings.
    assert_walked(
        &spot(&["AhAcAdAs2h", "KhKc2c3d4s"])?,
        &[100.0, 0.0],
        "quad aces against kings",
    )?;

    Ok(())
}

/// Third street, where four cards are still to come for each seat and the
/// space is far too large to walk.
#[test]
fn test_stud_equities_on_third_street() -> Result<(), PokerError> {
    // Rolled-up aces against three unconnected cards: as close to a lock as
    // third street gets, and still not one.
    assert_sampled(
        &spot(&["AsAhAd", "Qs7h2d"])?,
        &[(97.945, 0.16), (2.055, 0.16)],
        "rolled-up aces against nothing",
    )?;

    // The same aces against three to a flush, which is worth ten times what
    // three broken cards were.
    assert_sampled(
        &spot(&["AsAhAd", "7c6c5c"])?,
        &[(76.652, 0.47), (23.348, 0.47)],
        "rolled-up aces against a three-flush",
    )?;

    // Rolled-up deuces against two three-flushes: the trips are still the
    // favourite, and the two draws split what is left almost evenly.
    assert_sampled(
        &spot(&["2s2h2d", "7c6c5c", "Th9h8h"])?,
        &[(58.937, 0.55), (20.917, 0.45), (20.147, 0.45)],
        "rolled-up deuces against two three-flushes",
    )?;

    // A small pair against three overcards, which is the closest thing to a
    // coin flip stud offers on third street.
    assert_sampled(
        &spot(&["7h7c2s", "AhJh9d"])?,
        &[(58.326, 0.55), (41.674, 0.55)],
        "a pair of sevens against three overcards",
    )?;

    Ok(())
}
