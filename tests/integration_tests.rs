//! The public surface, used the way a program depending on this crate would.
//!
//! Everything else in the suite runs inside the crate and can reach private
//! items. This file cannot, so it is what catches something useful having been
//! left unexported.

use poker_equity::{
    cards::CardSet,
    error::PokerError,
    notation::parse_hand,
    odds::{equity, run_exact, EquityRequest, Target},
    variants::{DeuceSeven, Holdem, PokerVariant},
};

/// The ordinary way in: name the game, the hands and the board as text.
#[test]
fn test_a_question_can_be_asked_in_text() -> Result<(), PokerError> {
    // A pat 7-5 against a one-card draw, with the discarded ace named dead.
    // Nothing here needs sampling: the draw takes one card from forty-four.
    let request = EquityRequest::from_text(DeuceSeven, &["7d5h4c3s2h", "5s4h3d2c"], "", "Ad")?;
    let result = run_exact(&request)?.expect("one card from a known deck");

    assert!(result.exact, "a walked answer carries no error bar");
    let shares = result.equities();
    assert!((shares[0].percent() - 96.428571).abs() < 1e-6);
    assert!((shares[1].percent() - 3.571429).abs() < 1e-6);
    assert_eq!(shares[0].std_error, 0.0, "an exact answer has no spread");

    Ok(())
}

/// Sampling and walking must agree, which is the only check that catches a
/// biased sampler: comparing one sampled run to another compares two runs
/// that are wrong in the same way.
#[test]
fn test_sampling_lands_where_enumeration_says_it_should() -> Result<(), PokerError> {
    let request = EquityRequest::from_text(Holdem, &["AhKh", "2h2d"], "", "")?;

    let walked = run_exact(&request)?.expect("1,712,304 boards is walkable");
    let sampled = equity(&request, Target::Samples(500_000))?;

    for (seat, (exact, drawn)) in walked.equities().iter().zip(sampled.equities()).enumerate() {
        let window = 5.0 * drawn.std_error;
        assert!(
            (exact.equity - drawn.equity).abs() <= window,
            "seat {}: walked {:.4}%, sampled {:.4}% with a window of {:.4}",
            seat,
            exact.percent(),
            drawn.percent(),
            window * 100.0
        );
    }

    Ok(())
}

/// The mask form, for a caller that has already parsed the cards itself.
#[test]
fn test_a_question_can_be_asked_in_masks() -> Result<(), PokerError> {
    let hero = parse_hand("AhKh", Holdem.hole_cards())?;
    let villain = parse_hand("QsQd", Holdem.hole_cards())?;
    let board: Vec<CardSet> = Vec::new();

    let request = EquityRequest::from_masks(Holdem, &[hero, villain], &board, CardSet::EMPTY)?;
    let result = equity(&request, Target::Samples(50_000))?;

    let total: f64 = result.equities().iter().map(|player| player.equity).sum();
    assert!((total - 1.0).abs() < 1e-9, "the pot summed to {}", total);

    Ok(())
}

/// A refusal is a typed error a caller can match on, not a string.
#[test]
fn test_a_bad_question_comes_back_as_an_error() {
    use poker_equity::error::{EquityError, GameError};

    assert!(matches!(
        EquityRequest::from_text(Holdem, &["AhKh"], "", ""),
        Err(PokerError::Equity(EquityError::NoPlayers))
    ));

    assert!(matches!(
        EquityRequest::from_text(Holdem, &["AhKh", "AhQs"], "", ""),
        Err(PokerError::Game(GameError::DuplicateCard(_)))
    ));
}
