//! Omaha, end to end.
//!
//! Four hole cards, five board cards, and exactly two from hand with exactly
//! three from the board -- which is the rule that makes Omaha its own game
//! rather than hold'em with more cards, and the one worth testing hardest.
//!
//! Once the flop is out there are few enough boards left to walk them all, so
//! most of these are exact. Preflop there are 1,086,008 boards for each of the
//! ways the two hands can sit, which is past what is worth walking in a test,
//! so those are sampled -- see [`super::support`] for how.
//!
//! The figures came from the tests of the older engine, which measured each
//! over a hundred thousand deals. Every one was confirmed here, and where the
//! space could be walked the exact value has replaced the measured one.

use super::support::{assert_sampled, assert_walked};
use crate::{
    error::{EquityError, GameError, PokerError},
    odds::{run_exact, EquityRequest},
    variants::{Courchevel, Omaha, OmahaFive, OmahaSix},
};

/// An Omaha spot: the hands, and however much of the board is out.
fn spot(hands: &[&str], board: &str) -> Result<EquityRequest<Omaha>, PokerError> {
    EquityRequest::from_text(Omaha, hands, board, "")
}

/// What the engine refuses before dealing anything.
#[test]
fn test_a_bad_omaha_request_is_refused() -> Result<(), PokerError> {
    // Equity is a comparison, so one hand is not a question.
    assert!(matches!(
        spot(&["AsKsQcJd"], ""),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Omaha deals all four hole cards at once, so a short field is a miscount
    // rather than a hand still being dealt. An unknown card is a wildcard.
    for short in [&["Ah", "KsQcJdTs"], &["9h8h7h", "AsKsQcJd"]] {
        assert!(
            spot(short, "").is_err(),
            "{:?} is not four cards a seat",
            short
        );
    }
    assert!(spot(&["2h3h4h5h6h", "TsJsQcKd"], "").is_err(), "five is too many");

    // Wildcards are how an unknown hole card is named, and they are fine.
    assert!(spot(&["Ah**Kd", "TsJsQc9d"], "").is_ok());

    // A card cannot sit in two places, and the refusal names it.
    for (hands, board) in [
        (["AsKsQcJd", "As2d3h4c"], ""),
        (["AsKsQcJd", "Th9h8c7d"], "As2c3d"),
    ] {
        let refused = spot(&hands, board).unwrap_err();
        assert!(
            matches!(&refused, PokerError::Game(GameError::DuplicateCard(card))
                     if card.to_string() == "As"),
            "{:?} on {:?} uses the ace of spades twice, got {:?}",
            hands,
            board,
            refused
        );
    }

    // Six board cards is more board than Omaha has.
    assert!(spot(&["AsKsQcJd", "Th9h8c7d"], "2c3d4h5s6c7s").is_err());

    Ok(())
}

/// Boards already out, where every runout can be walked.
#[test]
fn test_omaha_equities_with_a_board() -> Result<(), PokerError> {
    // Quad aces on the flop. Nothing the board can do changes the winner, and
    // the answer is not 99.99 but exactly a hundred.
    assert_walked(
        &spot(&["AhAsKdQc", "JhTc9s8d"], "AdAc2h")?,
        &[100.0, 0.0],
        "quad aces on the flop",
    )?;

    // Trip aces against a rundown on an ace-high flop. The rundown needs a
    // straight and the board has to cooperate twice.
    assert_walked(
        &spot(&["AsAhKdQc", "JhTc9s8d"], "Ac2h3s")?,
        &[98.902439, 1.097561],
        "trip aces against a rundown",
    )?;

    // Trip kings on the turn against two pair with no draw left: one card to
    // come and none of them help.
    assert_walked(
        &spot(&["KsKhQdJc", "TsTh9s9h"], "Kc2h3s4d")?,
        &[100.0, 0.0],
        "trip kings on the turn",
    )?;

    // A rundown on a connected flop against aces. This is the spot Omaha is
    // played for: the big pair is a heavy underdog to a hand that has flopped
    // a straight already.
    assert_walked(
        &spot(&["JsTs9h8h", "AsAcKdQc"], "7s6c5h")?,
        &[91.097561, 8.902439],
        "a made straight against aces",
    )?;

    // Three-handed on a king-high flop: a set of kings against a rundown
    // against small pairs looking for a set of their own.
    assert_walked(
        &spot(&["KsKhQsQh", "JcTc9d8d", "5s5h4s4h"], "Kc7d2s")?,
        &[71.621622, 23.873874, 4.504505],
        "a set of kings three ways",
    )?;

    Ok(())
}

/// The exactly-two rule, which is what separates Omaha from hold'em.
///
/// Hero holds one heart against a three-heart board, so no river can give
/// Hero a flush: a flush needs two hearts from hand. Villain's trip kings are
/// already unbeatable. Were the rule not enforced, Hero would flush on any of
/// the nine remaining hearts and take a share.
#[test]
fn test_one_hole_heart_never_flushes() -> Result<(), PokerError> {
    assert_walked(
        &spot(&["Ah3d4s5c", "KdKc7h8s"], "KhQhJh2c")?,
        &[0.0, 100.0],
        "one heart against a three-heart board",
    )
}

/// Preflop, where the space is past what a test should walk.
#[test]
fn test_omaha_equities_preflop() -> Result<(), PokerError> {
    // Double-suited nines and eights against big cards rainbow. The little
    // double-suited hand is the favourite, which surprises people used to
    // hold'em: its cards work together and the big hand's do not.
    assert_sampled(
        &spot(&["9s9h8s8h", "AsKdQcJh"], "")?,
        &[(55.597, 0.56), (44.403, 0.56)],
        "double-suited nines and eights against big cards",
    )?;

    // Three-handed: tens and nines against big cards against small pairs. The
    // hand in the middle is the one that suffers -- both of the others have
    // pairs that can flop a set.
    assert_sampled(
        &spot(&["TsTh9s9h", "AsKdQcJh", "5c5d4c4d"], "")?,
        &[(43.508, 0.55), (21.217, 0.46), (35.275, 0.53)],
        "tens and nines against big cards against small pairs",
    )?;

    // Double-suited aces and kings against a rundown, which is about as close
    // to a preflop lock as Omaha gets: two to one, and no more.
    assert_sampled(
        &spot(&["JsTs9h8h", "AsKsAhKh"], "")?,
        &[(33.639, 0.53), (66.361, 0.53)],
        "a rundown against double-suited aces and kings",
    )?;

    Ok(())
}

/// Courchevel deals its first board card face up before the betting, so a
/// one-card board is a real spot and an empty one cannot happen.
#[test]
fn test_courchevel_starts_with_a_card_on_the_table() -> Result<(), PokerError> {
    let hands = ["AhAdKsQcJh", "9h8c7s6d5h"];

    assert!(matches!(
        EquityRequest::from_text(Courchevel, &hands, "", ""),
        Err(PokerError::Equity(EquityError::NotEnoughBoardCards { least: 1, found: 0 }))
    ));

    // One card is exactly where Courchevel starts, and four board cards to
    // come is still few enough to walk: 101,270 of them.
    assert_walked(
        &EquityRequest::from_text(Courchevel, &hands, "2c", "")?,
        &[61.102005, 38.897995],
        "aces against a rundown with a deuce showing",
    )?;

    Ok(())
}

/// Once the flop is out, Courchevel and five-card Omaha are the same spot and
/// must give the same answer. Walked, so there is no error bar to hide a
/// difference in.
#[test]
fn test_courchevel_and_five_card_omaha_agree_after_the_flop() -> Result<(), PokerError> {
    let hands = ["AhAdKsQcJh", "9h8c7s6d5h"];
    let board = "2c 7d 9d";

    let courchevel = run_exact(&EquityRequest::from_text(Courchevel, &hands, board, "")?)?
        .expect("small enough to enumerate");
    let omaha = run_exact(&EquityRequest::from_text(OmahaFive, &hands, board, "")?)?
        .expect("small enough to enumerate");

    assert_eq!(courchevel.samples, omaha.samples);
    for (seat, (a, b)) in courchevel.equities().iter().zip(omaha.equities()).enumerate() {
        assert!(
            (a.equity - b.equity).abs() < 1e-12,
            "seat {} differs: {:.10}% against {:.10}%",
            seat,
            a.percent(),
            b.percent()
        );
    }
    Ok(())
}

/// Five- and six-card Omaha deal more hole cards but play the same two.
#[test]
fn test_bigger_omaha_hands_still_split_the_pot_sensibly() -> Result<(), PokerError> {
    let five = run_exact(&EquityRequest::from_text(
        OmahaFive,
        &["AhAdKsQcJh", "9h8c7s6d5h"],
        "2c 7d 9d",
        "",
    )?)?
    .expect("small enough to enumerate");
    let six = run_exact(&EquityRequest::from_text(
        OmahaSix,
        &["AhAdKsQcJh2h", "9h8c7s6d5h3c"],
        "2c 7d 9d",
        "",
    )?)?
    .expect("small enough to enumerate");

    for (label, result) in [("five", five), ("six", six)] {
        let total: f64 = result.equities().iter().map(|player| player.equity).sum();
        assert!(
            (total - 1.0).abs() < 1e-9,
            "{}-card Omaha equities summed to {}",
            label,
            total
        );
        assert!(result.samples > 0);
    }
    Ok(())
}
