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
        Target::Samples(1_000_000),
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
    let result = equity(&request(&["A K", "Q J"], ""), Target::Samples(wanted)).unwrap();

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
    let result = equity(&request(&["AhKh", "22"], ""), Target::StandardError(wanted)).unwrap();

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
    assert!(
        gap <= slack,
        "{:.5} apart, against {:.5} of slack",
        gap,
        slack
    );

    // One thread is one chunk, deal for deal.
    let single = run_batch(&request, 50_000, 4, 1)?;
    let chunked = run_chunk(&request, 50_000, 4)?;
    assert_eq!(single.samples, chunked.samples);

    // Asking for more threads than deals does not lose any.
    assert_eq!(run_batch(&request, 3, 1, 64)?.samples, 3);

    Ok(())
}

/// The thread count rides on the request, so the default costs nothing to
/// use and changing it costs one call. Whatever it is set to is what the
/// run actually spreads over.
#[test]
fn test_a_request_carries_its_own_thread_count() -> Result<(), PokerError> {
    use crate::odds::{default_threads, equity, EquityRequest, Target};
    use crate::variants::Holdem;

    let request = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")?;
    assert_eq!(
        request.threads(),
        default_threads(),
        "a fresh request starts at the polite default"
    );

    let mine = request.clone().with_threads(2);
    assert_eq!(mine.threads(), 2);
    assert_eq!(
        request.threads(),
        default_threads(),
        "the original is untouched"
    );

    // The count is held to what the machine has: none is one, and more than
    // there are cores is every core.
    let cores = std::thread::available_parallelism().map_or(1, |cores| cores.get());
    assert_eq!(request.clone().with_threads(0).threads(), 1);
    assert_eq!(request.clone().with_threads(usize::MAX).threads(), cores);

    let mut later = request.clone();
    later.set_threads(3);
    assert_eq!(later.threads(), 3);

    // The count is used, not just stored: the same seeds over a different
    // number of threads divide the deals differently, so the answers agree
    // without the deals being the same.
    let one = equity(&request.clone().with_threads(1), Target::Samples(200_000))?;
    let two = equity(&mine, Target::Samples(200_000))?;
    let gap = (one.equities()[0].equity - two.equities()[0].equity).abs();
    let slack = 4.0 * (one.equities()[0].std_error + two.equities()[0].std_error);
    assert!(
        gap <= slack,
        "{:.5} apart, against {:.5} of slack",
        gap,
        slack
    );

    Ok(())
}

/// A precision target sampling can never reach is refused, rather than
/// pursued forever.
///
/// Zero is the one a caller reaches for by accident, meaning "exactly right";
/// NaN is worse, because every comparison against it is false, so the run
/// cannot even be seen to be failing. Both used to sample until killed.
#[test]
fn test_an_unreachable_precision_is_refused() {
    use crate::error::EquityError;

    // A spot too large to walk, so the target is what decides when to stop.
    let wide = request(&["AhKh", "QsQd", "**"], "");

    for wanted in [0.0, -0.001, f64::NAN, f64::INFINITY] {
        assert!(
            matches!(
                equity(&wide, Target::StandardError(wanted)),
                Err(PokerError::Equity(EquityError::UnreachableTarget(_)))
            ),
            "a standard error target of {} should be refused",
            wanted
        );
    }

    // A positive, finite one is still perfectly ordinary.
    assert!(equity(
        &request(&["AhKh", "QsQd"], "2c 7d 9h"),
        Target::StandardError(0.01)
    )
    .is_ok());
}
