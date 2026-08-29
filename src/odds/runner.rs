use rayon::prelude::*;

use crate::{
    error::PokerError,
    variants::{EquityCalculation, PokerVariant},
};

use super::{
    chunk::{ChunkResult, PlayerEquity},
    request::{run_chunk, run_exact, EquityRequest},
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
}

/// How many deals to run between progress reports.
///
/// A chunk this size takes a few milliseconds, which is a good repaint rate
/// and a fine granularity for a user to cancel at.
const CHUNK: u64 = 50_000;

/// Deals to run before deciding whether the standard error target is in
/// reach, so that an early lucky chunk cannot stop the run.
const MINIMUM_SAMPLES: u64 = CHUNK * 4;

/// Runs a request to a target, reporting progress between chunks.
///
/// Threads default to something polite: several calculators may be open at
/// once and one must not starve the others.
pub fn equity<V, F>(
    request: &EquityRequest<V>,
    target: Target,
    on_progress: F,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
    F: Fn(&Progress) + Send + Sync,
{
    if target == Target::Exact {
        if let Some(result) = run_exact(request)? {
            report(&on_progress, &result);
            return Ok(result);
        }
        // Too large to walk, so sample instead, tightly.
        return sample_until(request, Target::StandardError(0.0005), on_progress);
    }

    // Small spots are cheaper to enumerate than to sample, and come back
    // without an error bar.
    if let Some(result) = run_exact(request)? {
        report(&on_progress, &result);
        return Ok(result);
    }

    sample_until(request, target, on_progress)
}

/// Samples in parallel batches until the target is met.
fn sample_until<V, F>(
    request: &EquityRequest<V>,
    target: Target,
    on_progress: F,
) -> Result<ChunkResult, PokerError>
where
    V: PokerVariant + EquityCalculation + Send + Sync,
    F: Fn(&Progress) + Send + Sync,
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

        report(&on_progress, &total);

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

fn report<F: Fn(&Progress)>(on_progress: &F, result: &ChunkResult) {
    on_progress(&Progress {
        samples: result.samples,
        equities: result.equities(),
        worst_std_error: worst_error(result),
    });
}
