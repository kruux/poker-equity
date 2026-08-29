use crate::error::PokerError;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use crate::odds::{equity, equity_with_progress, EquityRequest, Progress, Target};
use crate::variants::Holdem;

fn request(hands: &[&str], board: &str) -> EquityRequest<Holdem> {
    EquityRequest::from_text(Holdem, hands, board, "").unwrap()
}

/// A spot small enough to walk comes back exact, without an error bar, even
/// when a sample count was asked for -- enumerating it is both cheaper and
/// better than sampling it.
#[test]
fn test_small_spots_come_back_exact() {
    let result = equity(
        &request(&["AhAd", "KsKc"], "2c 7d 9h"),
        Target::Samples(1_000_000)
    )
    .unwrap();

    assert!(result.exact, "990 boards is cheaper to walk than to sample");
    assert_eq!(result.samples, 990);
    let equities = result.equities();
    assert_eq!(equities[0].std_error, 0.0);
    assert!((equities[0].percent() - 91.6162).abs() < 0.001);
}

/// Asking for a sample count on a spot too wide to enumerate gets at least
/// that many deals.
#[test]
fn test_a_sample_target_is_met() {
    let wanted = 120_000;
    let result = equity(
        &request(&["A K", "Q J"], ""),
        Target::Samples(wanted)
    )
    .unwrap();

    assert!(!result.exact);
    assert!(
        result.samples >= wanted,
        "asked for {} deals, got {}",
        wanted,
        result.samples
    );
}

/// Asking for a precision runs until it is reached, rather than guessing at a
/// sample count.
#[test]
fn test_a_precision_target_is_met() {
    let wanted = 0.0008;
    let result = equity(
        &request(&["AhKh", "22"], ""),
        Target::StandardError(wanted)
    )
    .unwrap();

    for player in result.equities() {
        assert!(
            player.std_error <= wanted,
            "asked for {} but a seat came back with {}",
            wanted,
            player.std_error
        );
    }
}

/// Progress arrives between chunks, with the results as they stand, so a
/// caller can repaint a table and check whether the user cancelled.
#[test]
fn test_progress_is_reported_as_it_goes() {
    let calls = AtomicUsize::new(0);
    let last: Mutex<Option<Progress>> = Mutex::new(None);

    let result = equity_with_progress(
        &request(&["A K", "Q J"], ""),
        // Enough deals to need several batches, so that progress really is
        // reported as the run goes rather than only at the end.
        Target::Samples(400_000),
        |progress| {
            calls.fetch_add(1, Ordering::Relaxed);
            *last.lock().unwrap() = Some(progress.clone());
        },
    )
    .unwrap();

    assert!(
        calls.load(Ordering::Relaxed) > 1,
        "expected a report per batch, got {}",
        calls.load(Ordering::Relaxed)
    );

    let last = last.lock().unwrap().clone().expect("a final report");
    assert_eq!(last.samples, result.samples);
    assert_eq!(last.equities.len(), 2);
    // The reported worst error is the one the precision target waits on.
    assert!(last.worst_std_error > 0.0);
    let total: f64 = last.equities.iter().map(|player| player.equity).sum();
    assert!((total - 1.0).abs() < 1e-9);
}

/// Exact asked for on a spot too wide falls back to sampling tightly rather
/// than refusing.
#[test]
fn test_exact_falls_back_to_sampling_when_it_must() {
    let result = equity(&request(&["**", "**"], ""), Target::Exact).unwrap();
    assert!(!result.exact, "the space is too large to have been walked");
    for player in result.equities() {
        assert!(player.std_error <= 0.0005);
    }
}

/// A batch spread over threads is the same work as one run in a line, and
/// the same answer.
#[test]
fn test_a_batch_spreads_without_changing_the_answer() -> Result<(), PokerError> {
    use crate::odds::{default_threads, run_batch, run_chunk};

    let request = request(&["AhKh", "QsQd"], "");

    let alone = run_chunk(&request, 200_000, 9)?;
    let spread = run_batch(&request, 200_000, 9, default_threads())?;

    assert_eq!(spread.samples, alone.samples, "the same number of deals");

    // Different deals, since the split changes which ones are drawn, but the
    // same answer to within what sampling allows.
    let gap = (alone.equities()[0].equity - spread.equities()[0].equity).abs();
    let slack = 4.0 * (alone.equities()[0].std_error + spread.equities()[0].std_error);
    assert!(gap <= slack, "{:.5} apart, against {:.5} of slack", gap, slack);

    // One thread is one chunk, deal for deal.
    let single = run_batch(&request, 50_000, 4, 1)?;
    let chunked = run_chunk(&request, 50_000, 4)?;
    assert_eq!(single.samples, chunked.samples);

    // Asking for more threads than deals does not lose any.
    assert_eq!(run_batch(&request, 3, 1, 64)?.samples, 3);

    Ok(())
}
