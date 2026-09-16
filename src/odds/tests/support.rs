//! Shared machinery for the per-game equity suites.
//!
//! Every suite asks the same two questions -- what is each seat's share, and
//! is it the share it should be -- so the counting and the arithmetic live
//! here and each game's file holds only its own spots.
//!
//! Two ways of answering, and the choice is made by the size of the space
//! rather than by preference. Where every deal can be walked the figure is
//! exact, and the test has no window and no seed to argue about. Where it
//! cannot, the spot is sampled over several fixed seeds.
//!
//! Several seeds rather than one, because a single seed tests a single path
//! through the generator. That catches a wrong answer -- a bias moves every
//! seed at once -- but says nothing about a fault that needs a particular run
//! of cards to show itself. Fixed seeds rather than random ones, because a
//! test that fails one run in fifty gets muted rather than investigated.
//!
//! These use [`run_chunk`] rather than [`equity`](crate::odds::equity):
//! `equity` divides its work by the machine's thread count, and the division
//! decides which deals are drawn. Fine for an answer, no good for a test that
//! has to mean the same thing on a laptop and on a build server.

use crate::{
    error::PokerError,
    odds::{run_chunk, run_exact, EquityRequest},
    variants::{EquityCalculation, PokerVariant},
};

/// The seeds a sampled spot is checked over.
pub(super) const SEEDS: [u64; 4] = [11, 29, 71, 113];

/// How many deals each of those seeds draws.
pub(super) const DEALS: u64 = 200_000;

/// Checks a spot small enough to walk, where the answer is exact.
///
/// Exact means exact: there is no error bar, no seed and nothing to be
/// unlucky about. The comparison still allows a millionth of a percentage
/// point, because that is the precision the figures are written to rather
/// than a tolerance on the answer -- an exact share is a ratio of whole
/// numbers and most of them do not terminate, so 89.898989 is where the
/// printing stopped and not where the number does.
///
/// Panics if the space turns out to be too large to walk, which is a mistake
/// in the test rather than a fact about the spot: such a spot wants
/// [`assert_sampled`].
pub(super) fn assert_walked<V>(
    request: &EquityRequest<V>,
    expected: &[f64],
    what: &str,
) -> Result<(), PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    let result = run_exact(request)?.expect("small enough to walk");
    let got: Vec<f64> = result.equities().iter().map(|p| p.equity * 100.0).collect();
    let quoted: Vec<(f64, f64)> = expected.iter().map(|want| (*want, 1e-6)).collect();
    assert_shares(&got, &quoted, what);
    Ok(())
}

/// Checks a spot that has to be sampled, once per seed and once on average.
///
/// Each run has to land inside its own window, which is five standard errors
/// wide and so cannot be missed by luck. Their average has four times the
/// deals behind it and so half the standard error, and is held to half the
/// width -- which is what would catch a small bias every individual run could
/// absorb.
pub(super) fn assert_sampled<V>(
    request: &EquityRequest<V>,
    expected: &[(f64, f64)],
    what: &str,
) -> Result<(), PokerError>
where
    V: PokerVariant + EquityCalculation,
{
    let runs: Vec<Vec<f64>> = SEEDS
        .iter()
        .map(|&seed| {
            let result = run_chunk(request, DEALS, seed)?;
            Ok(result.equities().iter().map(|p| p.equity * 100.0).collect())
        })
        .collect::<Result<_, PokerError>>()?;

    for (seed, run) in SEEDS.iter().zip(&runs) {
        assert_shares(run, expected, &format!("{} (seed {})", what, seed));
    }

    let mean: Vec<f64> = (0..expected.len())
        .map(|seat| runs.iter().map(|run| run[seat]).sum::<f64>() / runs.len() as f64)
        .collect();
    let tighter: Vec<(f64, f64)> = expected.iter().map(|(v, w)| (*v, w / 2.0)).collect();
    assert_shares(
        &mean,
        &tighter,
        &format!("{} (averaged over {} runs)", what, runs.len()),
    );
    Ok(())
}

/// Checks a set of shares seat by seat, and that the pot came out whole.
///
/// Each expectation is a value and the window allowed around it.
pub(super) fn assert_shares(got: &[f64], expected: &[(f64, f64)], what: &str) {
    assert_eq!(got.len(), expected.len(), "{}: one share per seat", what);
    for (seat, (found, (want, window))) in got.iter().zip(expected).enumerate() {
        assert!(
            (found - want).abs() <= *window,
            "{}: seat {} took {:.4}%, expected {:.4}% within {:.3}",
            what,
            seat,
            found,
            want,
            window
        );
    }
    let total: f64 = got.iter().sum();
    assert!(
        (total - 100.0).abs() < 1e-9,
        "{}: shares summed to {}",
        what,
        total
    );
}
