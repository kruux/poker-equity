use rayon::prelude::*;

use crate::{
    error::{EquityError, PokerError},
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

/// How many threads to spread a batch over when the caller does not say.
///
/// Polite rather than greedy: several calculators may be open at once, and
/// one must not starve the others or the machine they are running on. A
/// caller that knows better passes its own count to [`run_batch`].
pub fn default_threads() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get().min(4))
        .unwrap_or(1)
}

/// Runs `samples` deals spread across `threads`, and returns them merged.
///
/// This is [`run_chunk`](crate::odds::run_chunk) with the machine behind it.
/// `run_chunk` is one thread by design -- pure, stateless, and cheap to
/// reason about -- which is the right shape for a caller driving its own
/// loop, but it leaves fifteen cores idle on a machine with sixteen.
///
/// Nothing is called back and no target is pursued. The caller gets a
/// `ChunkResult` and decides what to do next, which means cancelling between
/// batches costs nothing and nothing runs on a thread the caller did not
/// expect.
///
/// ```no_run
/// # use poker_equity::{odds::{run_batch, default_threads, ChunkResult, EquityRequest},
/// #                        variants::Holdem};
/// # let request = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")?;
/// let mut total = ChunkResult::empty(2);
/// for round in 0..10 {
///     total.merge(&run_batch(&request, 200_000, round, default_threads())?);
///     // repaint, and stop here if the user has had enough
/// }
/// # Ok::<(), poker_equity::error::PokerError>(())
/// ```
///
/// The work is split over the threads, so the same `seed` and the same
/// `threads` give the same deals; a different thread count divides the deals
/// differently and so draws different ones. Reproducing a run means matching
/// both.
pub fn run_batch<V>(
    request: &EquityRequest<V>,
    samples: u64,
    seed: u64,
    threads: usize,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
{
    let threads = threads.max(1).min(samples.max(1) as usize);
    let each = samples / threads as u64;
    let remainder = samples % threads as u64;

    let pieces: Vec<(u64, u64)> = (0..threads as u64)
        .map(|piece| {
            let count = each + u64::from(piece < remainder);
            // A distinct seed per piece, derived so that the same call gives
            // the same deals whatever the thread count.
            (count, seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(piece))
        })
        .collect();

    let batches = pieces
        .par_iter()
        .map(|&(count, seed)| run_chunk(request, count, seed))
        .collect::<Result<Vec<_>, PokerError>>()?;

    let mut total = ChunkResult::empty(request.players());
    for batch in &batches {
        total.merge(batch);
    }
    Ok(total)
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
/// # use poker_equity::{odds::{equity, EquityRequest, Target}, variants::Holdem};
/// # let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "", "")?;
/// let result = equity(&request, Target::Samples(500_000))?;
/// println!("{:.2}%", result.equities()[0].percent());
/// # Ok::<(), poker_equity::error::PokerError>(())
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
/// # use poker_equity::{odds::{equity_with_progress, EquityRequest, Target}, variants::Holdem};
/// # let request = EquityRequest::from_text(Holdem, &["AhAd", "KsKc"], "", "")?;
/// let result = equity_with_progress(&request, Target::Samples(500_000), |progress| {
///     println!("{} deals so far: {:.2}%", progress.samples, progress.equities[0].percent());
/// })?;
/// # Ok::<(), poker_equity::error::PokerError>(())
/// ```
///
/// The run spreads over [`EquityRequest::threads`], which starts at
/// [`default_threads`] and is changed with
/// [`with_threads`](EquityRequest::with_threads). Nothing has to be passed to
/// get the default.
pub fn equity_with_progress<V, F>(
    request: &EquityRequest<V>,
    target: Target,
    mut on_progress: F,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
    F: FnMut(&Progress),
{
    // A precision target is pursued by sampling until it is met, and nothing
    // caps that loop but the target itself -- so a target sampling can never
    // meet is not a slow run, it is a run that does not end. Zero is the one
    // a caller reaches for by accident, meaning "exactly right"; that is
    // `Target::Exact`, and the error says so. NaN is worse, since every
    // comparison against it is false and the loop cannot even be seen to be
    // failing.
    if let Target::StandardError(wanted) = target {
        if !(wanted.is_finite() && wanted > 0.0) {
            return Err(EquityError::UnreachableTarget(wanted).into());
        }
    }

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
    let threads = request.threads();
    let mut total = ChunkResult::empty(request.players());
    let mut round: u64 = 0;

    loop {
        total.merge(&run_batch(request, CHUNK * threads as u64, round, threads)?);
        round += 1;

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
