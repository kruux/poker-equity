//! Seven-card stud, end to end.
//!
//! Where both hands are named down to the last card or two, the space is
//! small enough to walk and the figure is exact -- 671,580 deals when two
//! seats each have two cards to come. Where a seat is still on third street
//! with four to come, the space is billions and the answer is sampled.
//!
//! The sampled ones run several seeds rather than one. A single seed tests a
//! single path through the generator, which catches a wrong answer -- a bias
//! moves every seed at once -- but says nothing about a fault that needs a
//! particular run of cards to show itself. Several seeds test several paths,
//! stay reproducible on every machine, and still cannot fail by luck.
//!
//! They use `run_chunk` rather than `equity` for the same reason: `equity`
//! divides its work by the machine's thread count, and the division decides
//! which deals are drawn. Fine for an answer, no good for a test that has to
//! mean the same thing on a laptop and on a build server.
//!
//! The figures came from the tests of the older engine, which measured them
//! over a hundred thousand deals apiece. Every one was confirmed here, and
//! the exact value replaces the measured one wherever the space could be
//! walked.

use crate::{
    error::{EquityError, PokerError},
    odds::{run_chunk, run_exact, EquityRequest},
    variants::Stud,
};

/// The exact equities of a spot, as percentages.
fn walked(hands: &[&str]) -> Result<Vec<f64>, PokerError> {
    let request = EquityRequest::from_text(Stud, hands, "", "")?;
    let result = run_exact(&request)?.expect("small enough to walk");
    Ok(result.equities().iter().map(|p| p.equity * 100.0).collect())
}

/// How many independent runs a sampled spot is checked over.
const SEEDS: [u64; 4] = [11, 29, 71, 113];

/// The sampled equities of a spot: every seat, over every seed.
///
/// Returns one row per seed, so a caller can check each run against its own
/// window and the average of them against a tighter one.
fn sampled(hands: &[&str]) -> Result<Vec<Vec<f64>>, PokerError> {
    let request = EquityRequest::from_text(Stud, hands, "", "")?;
    SEEDS
        .iter()
        .map(|&seed| {
            let result = run_chunk(&request, 200_000, seed)?;
            Ok(result.equities().iter().map(|p| p.equity * 100.0).collect())
        })
        .collect()
}

/// Checks every run of a sampled spot, and their average.
///
/// Each run has to land inside its own window, which is five standard errors
/// and so cannot be missed by luck. Their average has four times the deals
/// behind it, so it is held to half the width -- which is what would catch a
/// small bias that every individual run could absorb.
fn assert_sampled(hands: &[&str], expected: &[(f64, f64)], what: &str) -> Result<(), PokerError> {
    let runs = sampled(hands)?;
    for (seed, run) in SEEDS.iter().zip(&runs) {
        assert_shares(run, expected, &format!("{} (seed {})", what, seed));
    }

    let mean: Vec<f64> = (0..expected.len())
        .map(|seat| runs.iter().map(|run| run[seat]).sum::<f64>() / runs.len() as f64)
        .collect();
    let tighter: Vec<(f64, f64)> = expected.iter().map(|(v, w)| (*v, w / 2.0)).collect();
    assert_shares(&mean, &tighter, &format!("{} (averaged over {} runs)", what, runs.len()));
    Ok(())
}

/// Checks a set of shares against what is expected, seat by seat, and that
/// the pot came out whole.
fn assert_shares(got: &[f64], expected: &[(f64, f64)], what: &str) {
    assert_eq!(got.len(), expected.len(), "one share per seat");
    for (seat, (found, (want, window))) in got.iter().zip(expected).enumerate() {
        assert!(
            (found - want).abs() <= *window,
            "{}: seat {} took {:.3}%, expected {:.3}% within {:.2}",
            what,
            seat,
            found,
            want,
            window
        );
    }
    let total: f64 = got.iter().sum();
    assert!((total - 100.0).abs() < 1e-9, "{}: shares summed to {}", what, total);
}

/// What the engine refuses before dealing anything.
#[test]
fn test_a_bad_stud_request_is_refused() -> Result<(), PokerError> {
    let build = |hands: &[&str]| EquityRequest::from_text(Stud, hands, "", "").map(|_| ());

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
    assert_shares(
        &walked(&["AhKhQhJhTh9s8s", "2c3c4c5c6c7d8d"])?,
        &[(100.0, 1e-9), (0.0, 1e-9)],
        "a made royal against a made straight flush",
    );

    // Two royal flushes already made, two cards to come each. Nothing either
    // player draws can change it, so they split every deal.
    assert_shares(
        &walked(&["AhKhQhJhTh", "AsKsQsJsTs"])?,
        &[(50.0, 1e-9), (50.0, 1e-9)],
        "two royal flushes",
    );

    // Quad aces against two pair at best: no runout saves the kings.
    assert_shares(
        &walked(&["AhAcAdAs2h", "KhKc2c3d4s"])?,
        &[(100.0, 1e-9), (0.0, 1e-9)],
        "quad aces against kings",
    );

    Ok(())
}

/// Third street, where four cards are still to come for each seat and the
/// space is far too large to walk.
#[test]
fn test_stud_equities_on_third_street() -> Result<(), PokerError> {
    // Rolled-up aces against three unconnected cards: as close to a lock as
    // third street gets, and still not one.
    assert_sampled(
        &["AsAhAd", "Qs7h2d"],
        &[(97.945, 0.16), (2.055, 0.16)],
        "rolled-up aces against nothing",
    )?;

    // The same aces against three to a flush, which is worth ten times what
    // three broken cards were.
    assert_sampled(
        &["AsAhAd", "7c6c5c"],
        &[(76.652, 0.47), (23.348, 0.47)],
        "rolled-up aces against a three-flush",
    )?;

    // Rolled-up deuces against two three-flushes: the trips are still the
    // favourite, and the two draws split what is left almost evenly.
    assert_sampled(
        &["2s2h2d", "7c6c5c", "Th9h8h"],
        &[(58.937, 0.55), (20.917, 0.45), (20.147, 0.45)],
        "rolled-up deuces against two three-flushes",
    )?;

    // A small pair against three overcards, which is the closest thing to a
    // coin flip stud offers on third street.
    assert_sampled(
        &["7h7c2s", "AhJh9d"],
        &[(58.326, 0.55), (41.674, 0.55)],
        "a pair of sevens against three overcards",
    )?;

    Ok(())
}
