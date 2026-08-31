//! Seven-card stud hi/lo, eight or better, end to end.
//!
//! The pot is split between the best high hand and the best qualifying low,
//! and a low only qualifies on five distinct ranks of eight or lower. Where no
//! low qualifies the high takes everything, which is why so many of these
//! figures are not the halves they look like they should be.
//!
//! Walked where the space allows and sampled where it does not; how the
//! sampling is checked is in [`super::support`].
//!
//! The figures came from the tests of the older engine, which measured each
//! over a hundred thousand deals. Every one was confirmed here, and where the
//! space could be walked the exact value has replaced the measured one.

use super::support::{assert_sampled, assert_walked};
use crate::{
    error::{EquityError, PokerError},
    odds::EquityRequest,
    variants::StudHiLo,
};

/// A stud hi/lo spot: no board and no dead cards, so the hands are all of it.
fn spot(hands: &[&str]) -> Result<EquityRequest<StudHiLo>, PokerError> {
    EquityRequest::from_text(StudHiLo, hands, "", "")
}

/// What the engine refuses before dealing anything.
#[test]
fn test_a_bad_stud_hi_lo_request_is_refused() -> Result<(), PokerError> {
    assert!(matches!(
        spot(&["AhAcAd"]),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // The whole table is on the same street, so fields must match.
    assert!(matches!(
        spot(&["AhAcAd", "KhKcKdKs"]),
        Err(PokerError::Equity(EquityError::UnequalHandSizes))
    ));

    // And third street is the earliest street there is.
    assert!(matches!(
        spot(&["AhAc", "KhKc"]),
        Err(PokerError::Equity(EquityError::NotEnoughCards(2)))
    ));

    Ok(())
}

/// Spots where every deal can be walked, so the figure is exact.
#[test]
fn test_stud_hi_lo_equities_that_can_be_walked() -> Result<(), PokerError> {
    // Trip aces against trip kings, two cards to come each. Neither can make
    // a qualifying low from what they hold, so the whole pot goes high and
    // the aces are a heavy favourite for all of it.
    assert_walked(
        &spot(&["AhAcAdKhQc", "KsKcKdQhJc"])?,
        &[74.956074, 25.043926],
        "trip aces against trip kings, no low in sight",
    )?;

    // A full house against a made eight-low. The high hand cannot make a low
    // and the low hand cannot make a better high, so each takes its own half
    // of the pot every time -- fifty each, and not by measurement.
    assert_walked(
        &spot(&["AhAcAdKhKc", "8h6c4d3h2c"])?,
        &[50.0, 50.0],
        "a full house against a made eight-low",
    )?;

    // A made flush that is also a made eight-low against two pair. Scooping
    // both halves is what makes a hand like this worth so much more than its
    // high alone.
    assert_walked(
        &spot(&["8h7h4h3h2h", "KhKc9d8c7c"])?,
        &[96.336847, 3.663153],
        "a flush that is also an eight-low",
    )?;

    // Two complete hands, both a pair of kings with an 8-7-6 kicker, so the
    // high is a dead tie. Hero's 8-7-6-3-2 low beats Villain's 8-7-6-5-3, so
    // Hero takes half the high half and all of the low half.
    //
    // Grouping ties by equality rather than by the ordering used to sort them
    // once split these two apart and handed the first seat the whole high.
    assert_walked(
        &spot(&["KhKd8c7d6h3s2c", "KsKc8d7h6s5c3d"])?,
        &[75.0, 25.0],
        "equal highs, one better low",
    )?;

    // The same shape reached the other way round: identical lows, one better
    // high.
    assert_walked(
        &spot(&["8h6c4d3h2cKhKd", "8d6h4c3d2hQhQd"])?,
        &[75.0, 25.0],
        "equal lows, one better high",
    )?;

    Ok(())
}

/// Third street, where the space is far too large to walk.
#[test]
fn test_stud_hi_lo_equities_on_third_street() -> Result<(), PokerError> {
    // Trip aces against a made eight-low against a better low draw. The high
    // hand is not the biggest share: the low draw that is also live for high
    // takes more of the pot than the trips do.
    assert_sampled(
        &spot(&["AhAcAdKhQc", "8h6c4d3h2c", "7h4c3d2hAs"])?,
        &[(38.324, 0.54), (13.402, 0.38), (48.274, 0.56)],
        "trip aces against two lows",
    )?;

    // A full ring on third street, four high-only hands and two low. This is
    // the shape that has to be sampled: six seats with four cards to come
    // each is more combinations than there are atoms to count them with.
    assert_sampled(
        &spot(&["KhKc9d", "QhQc8d", "Ah3c7d", "2h4c6d", "3h5c4d", "4h6c8h"])?,
        &[
            (18.609, 0.44),
            (12.954, 0.38),
            (14.668, 0.40),
            (17.732, 0.43),
            (21.936, 0.46),
            (14.102, 0.39),
        ],
        "six-handed on third street",
    )?;

    Ok(())
}
