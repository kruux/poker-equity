//! 2-7 lowball, single draw, end to end.
//!
//! The worst poker hand wins: no straights, no flushes, and the ace is always
//! high, so `75432` unsuited is the best hand in the game and `A2345` is a bad
//! one. One draw only, which is the game this library models.
//!
//! A draw game is where a short field earns its keep. Five cards is a pat
//! hand; four cards is a player drawing one, and the cards thrown away are
//! named as dead so they leave the deck without pretending they were never
//! seen. Almost every spot here is small enough to walk outright: a seat
//! drawing one has forty-odd cards to come and no board at all.
//!
//! The figures came from the tests of the older engine, which measured each
//! over a hundred thousand deals. Every one was confirmed here, and where the
//! space could be walked the exact value has replaced the measured one.

use super::support::assert_walked;
use crate::{
    error::{GameError, PokerError},
    odds::EquityRequest,
    variants::DeuceSeven,
};

/// A 2-7 spot: the hands as they stand, and the cards already thrown away.
fn spot(hands: &[&str], dead: &str) -> Result<EquityRequest<DeuceSeven>, PokerError> {
    EquityRequest::from_text(DeuceSeven, hands, "", dead)
}

/// Two hands already made, where nothing is left to deal.
#[test]
fn test_pat_hands_are_settled_before_they_start() -> Result<(), PokerError> {
    // The same five ranks in different suits. 2-7 ignores suits entirely, so
    // this is a chop and there is nothing to sample: one deal, split.
    assert_walked(
        &spot(&["7d5h4c3s2h", "7c5s4h3d2c"], "")?,
        &[50.0, 50.0],
        "the nuts twice over",
    )?;

    // Three of them, and the pot divides in three.
    assert_walked(
        &spot(&["7d5h4c3s2h", "7c5s4h3d2c", "7s5d4d3h2s"], "")?,
        &[33.333333, 33.333333, 33.333333],
        "the nuts three times over",
    )?;

    // 7-5-4-3-2 against 7-6-4-3-2. One rank apart at the second card, and
    // that is the whole hand: the seven-five wins every time.
    assert_walked(
        &spot(&["7d5h4c3s2h", "7c6s4h3d2c"], "")?,
        &[100.0, 0.0],
        "a seven-five against a seven-six",
    )?;

    Ok(())
}

/// A pat hand against a player drawing, which is what the notation is for.
#[test]
fn test_a_pat_hand_against_a_draw() -> Result<(), PokerError> {
    // The nuts against a one-card draw to the same hand. The drawer threw
    // away an ace and needs one of the three remaining sixes to chop -- and
    // nothing at all can win, because 75432 cannot be beaten.
    assert_walked(
        &spot(&["7d5h4c3s2h", "5s4h3d2c"], "Ad")?,
        &[96.428571, 3.571429],
        "the nuts against a one-card draw",
    )?;

    // The same draw with two of those sixes already gone. Three outs become
    // one, and the draw's share falls with them.
    assert_walked(
        &spot(&["7d5h4c3s2h", "5s4h3d2c"], "Ad 7h 7s")?,
        &[98.75, 1.25],
        "the same draw with its outs thinning out",
    )?;

    // Drawing three to 4-2 against the nuts. Every one of the three cards has
    // to come good and the best it can do even then is a chop, which is worth
    // about a tenth of a percent.
    assert_walked(
        &spot(&["7d5h4c3s2h", "4h2c"], "Ad Kd")?,
        &[99.890609, 0.109391],
        "drawing three against the nuts",
    )?;

    // Two more cards out of the deck, and the answer does not move at all --
    // the same figure to every decimal. A king and a queen are cards the
    // draw could never have used, so removing them removes none of its outs.
    assert_walked(
        &spot(&["7d5h4c3s2h", "4h2c"], "Ks Qd")?,
        &[99.890609, 0.109391],
        "dead high cards, which change nothing",
    )?;

    Ok(())
}

/// Both seats drawing one, which is where the game is actually played.
#[test]
fn test_two_one_card_draws() -> Result<(), PokerError> {
    // Nine-eight against nine-seven, each throwing away a broadway card.
    // Villain is drawing to the better hand and is the small favourite; the
    // gap is under five points because both miss far more often than not.
    assert_walked(
        &spot(&["9h8c4d2h", "9d7h5s2d"], "As Kd")?,
        &[45.470383, 54.529617],
        "a nine-eight draw against a nine-seven draw",
    )?;

    // Ten-eight against nine-seven, the same shape a rank apart. Being one
    // card worse to start with is worth about seven points here.
    assert_walked(
        &spot(&["Th8c4s2h", "9d7h4h2d"], "Kd Ks")?,
        &[41.811847, 58.188153],
        "a ten-eight draw against a nine-seven draw",
    )?;

    Ok(())
}

/// A card cannot be thrown away and held at the same time.
#[test]
fn test_a_card_cannot_be_dead_and_in_a_hand() -> Result<(), PokerError> {
    let refused = spot(&["7d5h4c3s2h", "9d7h5s2d"], "7d").unwrap_err();
    assert!(
        matches!(&refused, PokerError::Game(GameError::DuplicateCard(card))
                 if card.to_string() == "7d"),
        "the seven of diamonds is both held and discarded, got {:?}",
        refused
    );
    Ok(())
}
