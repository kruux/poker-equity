//! Hold'em, end to end: what the engine refuses, and what it answers.
//!
//! Every equity here is walked rather than sampled. Both hands are named, so
//! the only thing left to deal is the board, and there are few enough boards
//! to visit all of them -- 1,712,304 for a preflop spot, 990 for a flop. That
//! makes these assertions exact: no tolerance, no seed, no flake.
//!
//! The figures were carried over from the tests of the older engine, which
//! measured them over a hundred thousand deals apiece. Walking the space
//! confirmed every one of them to within its sampling error, and the exact
//! value is what is asserted now.

use crate::{
    error::{EquityError, GameError, PokerError},
    notation::NotationErrorKind,
    odds::{run_exact, EquityRequest},
    variants::Holdem,
};

/// The exact equities of a spot, as percentages.
fn walked(hands: &[&str], board: &str) -> Result<Vec<f64>, PokerError> {
    let request = EquityRequest::from_text(Holdem, hands, board, "")?;
    let result = run_exact(&request)?.expect("these spots are small enough to walk");
    Ok(result
        .equities()
        .iter()
        .map(|player| player.equity * 100.0)
        .collect())
}

/// Asserts the walked equities are the expected ones, to a millionth of a
/// percentage point -- which is the precision the figures below are written
/// to, and some of them do not terminate: 890/990 is 89.898989...
///
/// An exact answer has no error bar, so this is not a tolerance for sampling.
/// It is the width of the decimal the number is quoted at.
fn assert_walked(hands: &[&str], board: &str, expected: &[f64]) -> Result<(), PokerError> {
    let got = walked(hands, board)?;
    assert_eq!(got.len(), expected.len(), "one share per seat");
    for (seat, (found, want)) in got.iter().zip(expected).enumerate() {
        assert!(
            (found - want).abs() < 1e-6,
            "{:?} on {:?}: seat {} took {:.6}%, expected {:.6}%",
            hands,
            board,
            seat,
            found,
            want
        );
    }
    // And the pot is handed out exactly once.
    let total: f64 = got.iter().sum();
    assert!((total - 100.0).abs() < 1e-9, "the shares summed to {}", total);
    Ok(())
}

/// A request the engine cannot answer is refused when it is built, before any
/// card is dealt, and the refusal says which mistake was made.
#[test]
fn test_a_bad_holdem_request_is_refused_with_a_reason() -> Result<(), PokerError> {
    let build = |hands: &[&str], board: &str, dead: &str| {
        EquityRequest::from_text(Holdem, hands, board, dead).map(|_| ())
    };

    // Equity is a comparison, so one hand is not a question.
    assert!(matches!(
        build(&[], "", ""),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));
    assert!(matches!(
        build(&["AhKh"], "", ""),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    // Hold'em deals both hole cards at once, so a field of one card is a
    // miscount rather than a hand in progress. An unknown card is `A*`.
    let one_card = build(&["Ah", "Ks"], "", "").unwrap_err();
    assert!(
        matches!(
            &one_card,
            PokerError::Equity(EquityError::Notation(e))
                if matches!(e.kind, NotationErrorKind::WrongSlotCount { expected: 2, found: 1 })
        ),
        "got {:?}",
        one_card
    );
    assert!(build(&["AhKhQh", "2h2d"], "", "").is_err(), "three is too many");
    assert!(build(&["", "AhKh"], "", "").is_err(), "an empty field is not a hand");

    // A card is in one place or none. Whichever two places claim it, the
    // error names the card rather than saying the request cannot be met.
    for (label, hands, board, dead) in [
        ("two seats", &["AhKh", "AhQd"][..], "", ""),
        ("a seat and the board", &["AhKh", "2h2d"][..], "AhQcJc", ""),
        ("a seat and the dead cards", &["AhKh", "2h2d"][..], "", "Ah"),
    ] {
        let error = build(hands, board, dead).unwrap_err();
        assert!(
            matches!(error, PokerError::Game(GameError::DuplicateCard(card))
                     if card.to_string() == "Ah"),
            "{} claiming Ah gave {:?}",
            label,
            error
        );
    }

    // Five cards is the whole board.
    assert!(build(&["AhKh", "2h2d"], "2c3c4c5c6c7c", "").is_err());

    Ok(())
}

/// The equities everyone knows, walked rather than trusted.
#[test]
fn test_holdem_equities_preflop() -> Result<(), PokerError> {
    // The coin flip that is not quite a coin flip: two overcards against the
    // smallest pair.
    assert_walked(&["AhKh", "2h2d"], "", &[49.702389, 50.297611])?;

    // A third hand takes from both, and not evenly: the queens beat the
    // deuces for most of what the deuces lose.
    assert_walked(
        &["AhKh", "2h2d", "QcQd"],
        "",
        &[38.198271, 16.934208, 44.867521],
    )?;

    // Ace-ten offsuit against suited connectors, the classic "am I really
    // ahead" spot. Ahead, but by less than the ace suggests.
    assert_walked(&["AcTs", "6c7c"], "", &[59.748911, 40.251089])?;

    Ok(())
}

/// The same hands once the board starts to arrive.
#[test]
fn test_holdem_equities_with_a_board() -> Result<(), PokerError> {
    // Two pair against a pair of deuces: 990 run-outs, and the deuces need
    // one of the two left.
    assert_walked(&["AhKh", "2h2d"], "AcKcQc", &[89.898990, 10.101010])?;

    // One card to come, forty-four of them, seven of which save the deuces.
    assert_walked(&["AhKh", "2h2d"], "AcKcQcJc", &[84.090909, 15.909091])?;

    // A wet flop turns the ace-high hand from a favourite into a coin flip:
    // the flush draw and the open-ender together are worth almost exactly
    // what the ace was worth.
    assert_walked(&["AcTs", "6c7c"], "Jc8c3h", &[51.010101, 48.989899])?;

    // Three ways, where the draw is the favourite and the made pair is not.
    assert_walked(
        &["AhKd", "JsTs", "5h5c"],
        "Qc9h4s",
        &[11.849391, 50.387597, 37.763012],
    )?;

    Ok(())
}

/// A board that already decides the hand leaves nothing to deal.
#[test]
fn test_a_settled_board_is_one_deal() -> Result<(), PokerError> {
    let request = EquityRequest::from_text(Holdem, &["AhKh", "2h2d"], "AcKcQcJcTc", "")?;
    let result = run_exact(&request)?.expect("nothing left to deal");

    assert_eq!(result.samples, 1, "a full board is one deal");
    assert!(result.exact);
    assert_eq!(result.equities()[0].std_error, 0.0, "no error bar on a certainty");

    // The board plays: a royal flush in clubs, which neither player improves
    // on, so they split it.
    assert!((result.equities()[0].equity - 0.5).abs() < 1e-12);
    assert!((result.equities()[1].equity - 0.5).abs() < 1e-12);

    Ok(())
}
