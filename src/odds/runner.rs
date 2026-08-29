use rayon::prelude::*;

use crate::{
    error::PokerError,
    variants::{EquityCalculation, PokerVariant},
};

use super::{
    chunk::{ChunkResult, PlayerEquity},
    request::{run_chunk, run_exact, run_exact_within, EquityRequest, EXACT_DEAL_LIMIT},
};

/// How long to keep sampling.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Target {
    /// Stop after this many deals.
    Samples(u64),
    /// Keep going until every seat's standard error is at most this, as a
    /// fraction of the pot. Answers a question a sample count cannot: "make
    /// this tight enough to trust".
    StandardError(f64),
    /// Walk every deal, falling back to sampling when the space is too large.
    Exact,
}

/// How far along a run is, handed to the progress callback between chunks.
#[derive(Debug, Clone)]
pub struct Progress {
    /// Deals completed so far.
    pub samples: u64,
    /// The results as they stand.
    pub equities: Vec<PlayerEquity>,
    /// The widest standard error across the seats, which is what
    /// `Target::StandardError` is waiting on.
    pub worst_std_error: f64,
    /// The share of attempted deals that could be used.
    ///
    /// One in the ordinary case. It falls when hands compete for scarce
    /// cards, and a low figure is worth showing: it is the difference between
    /// a slow answer and one that looks stuck.
    pub acceptance: f64,
}

/// How many deals to run between progress reports.
///
/// A chunk this size takes a few milliseconds, which is a good repaint rate
/// and a fine granularity for a user to cancel at.
const CHUNK: u64 = 50_000;

/// Deals to run before deciding whether the standard error target is in
/// reach, so that an early lucky chunk cannot stop the run.
const MINIMUM_SAMPLES: u64 = CHUNK * 4;

/// Runs a request until the target is met, and returns the answer.
///
/// ```no_run
/// # use poker_calculator::{odds::{equity, EquityRequest, Target}, variants::Holdem};
/// # let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "", "")?;
/// let result = equity(&request, Target::Samples(500_000))?;
/// println!("{:.2}%", result.equities()[0].percent());
/// # Ok::<(), poker_calculator::error::PokerError>(())
/// ```
///
/// Use [`equity_with_progress`] to watch a long run as it goes, or
/// [`run_chunk`](crate::odds::run_chunk) to drive the loop yourself.
pub fn equity<V>(request: &EquityRequest<V>, target: Target) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
{
    equity_with_progress(request, target, |_progress| {})
}

/// Runs a request to a target, handing `on_progress` the results so far after
/// every batch of deals.
///
/// That is where a caller repaints its table and decides whether the user has
/// cancelled. A batch is a few milliseconds, so it is a fine rate for both.
///
/// ```no_run
/// # use poker_calculator::{odds::{equity_with_progress, EquityRequest, Target}, variants::Holdem};
/// # let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "", "")?;
/// let result = equity_with_progress(&request, Target::Samples(500_000), |progress| {
///     println!("{} deals so far: {:.2}%", progress.samples, progress.equities[0].percent());
/// })?;
/// # Ok::<(), poker_calculator::error::PokerError>(())
/// ```
///
/// Threads default to something polite: several calculators may be open at
/// once and one must not starve the others.
pub fn equity_with_progress<V, F>(
    request: &EquityRequest<V>,
    target: Target,
    mut on_progress: F,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
    F: FnMut(&Progress),
{
    if target == Target::Exact {
        if let Some(result) = run_exact(request)? {
            report(&mut on_progress, &result);
            return Ok(result);
        }
        // Too large to walk, so sample instead, tightly.
        return sample_until(request, Target::StandardError(0.0005), on_progress);
    }

    // A spot with fewer deals than the caller was going to sample is cheaper
    // to walk than to sample, and comes back without an error bar. Asking for
    // a precision rather than a count says nothing about how much work is
    // acceptable, so there the only limit is what can be walked at all.
    let budget = match target {
        Target::Samples(wanted) => wanted as usize,
        _ => EXACT_DEAL_LIMIT,
    };
    if let Some(result) = run_exact_within(request, budget)? {
        report(&mut on_progress, &result);
        return Ok(result);
    }

    sample_until(request, target, on_progress)
}

/// Samples in parallel batches until the target is met.
fn sample_until<V, F>(
    request: &EquityRequest<V>,
    target: Target,
    mut on_progress: F,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
    F: FnMut(&Progress),
{
    let threads = std::thread::available_parallelism()
        .map(|n| n.get().min(4))
        .unwrap_or(1);

    let mut total = ChunkResult::empty(request.players());
    let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;

    loop {
        // One batch per thread, each with its own seed, merged afterwards.
        let seeds: Vec<u64> = (0..threads as u64)
            .map(|offset| seed.wrapping_add(offset.wrapping_mul(0x0100_0000_01B3)))
            .collect();
        seed = seed.wrapping_add(threads as u64 * 0x0100_0000_01B3);

        let batches = seeds
            .par_iter()
            .map(|&seed| run_chunk(request, CHUNK, seed))
            .collect::<Result<Vec<_>, PokerError>>()?;
        for batch in &batches {
            total.merge(batch);
        }

        report(&mut on_progress, &total);

        match target {
            Target::Samples(wanted) if total.samples >= wanted => return Ok(total),
            Target::StandardError(wanted) => {
                if total.samples >= MINIMUM_SAMPLES && worst_error(&total) <= wanted {
                    return Ok(total);
                }
            }
            Target::Samples(_) => {}
            Target::Exact => unreachable!("exact is handled before sampling"),
        }
    }
}

/// The widest standard error across the seats.
fn worst_error(result: &ChunkResult) -> f64 {
    result
        .equities()
        .iter()
        .map(|player| player.std_error)
        .fold(0.0, f64::max)
}

/// Hands the caller the results as they stand.
fn report<F: FnMut(&Progress)>(on_progress: &mut F, result: &ChunkResult) {
    on_progress(&Progress {
        samples: result.samples,
        equities: result.equities(),
        worst_std_error: worst_error(result),
        acceptance: result.acceptance(),
    });
}
