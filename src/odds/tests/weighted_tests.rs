//! The weighted dealing path, and the line that keeps it away from
//! everything else.
//!
//! Two properties matter here and they fail in different ways. A wrong weight
//! moves the answer, which is caught by walking a spot and comparing. A wrong
//! error bar leaves the answer where it is and reports it as more certain
//! than it is, which is worse, and is caught by the same comparison read in
//! standard errors rather than in points.

use crate::error::PokerError;
use crate::odds::{run_chunk, run_exact, ChunkResult, EquityRequest};
use crate::variants::{Holdem, Omaha, Stud};

/// The board that makes a six-club spot small enough to walk: every card
/// named, so only the seats' clubs are still to be dealt.
const FULL_BOARD: &str = "2h 7d 9s Ts Jd";

/// A weighted answer must be the answer, and it is checked where the answer
/// is known rather than where it is convenient.
///
/// Four seats each wanting two clubs, on a board with nothing left to come,
/// is both contended enough to be dealt the weighted way and small enough to
/// enumerate. So the same request is walked and sampled, and the two must
/// agree -- in points, which catches a wrong weight, and in standard errors,
/// which catches an error bar that has been quietly narrowed.
#[test]
fn test_weighted_sampling_agrees_with_enumeration() -> Result<(), PokerError> {
    let mut request = EquityRequest::from_text(Holdem, &["c c"; 4], FULL_BOARD, "")?;
    // The policy is tested below; what is tested here is the arithmetic, so
    // the path is asked for rather than waited for.
    assert!(request.force_weighted(), "this spot can be weighted");

    let walked = run_exact(&request)?.expect("four club seats on a full board can be walked");
    let drawn = run_chunk(&request, 150_000, 17)?;

    assert!(walked.exact, "the walked result is the exact one");
    for (seat, (exact, sampled)) in walked.equities().iter().zip(drawn.equities()).enumerate() {
        assert!(
            (exact.equity - sampled.equity).abs() <= 4.0 * sampled.std_error,
            "seat {}: walking gives {:.4}% and weighted dealing gives {:.4}% +/- {:.4}, \
             which is {:.1} standard errors apart",
            seat,
            exact.percent(),
            sampled.percent(),
            sampled.margin_percent(),
            (exact.equity - sampled.equity).abs() / sampled.std_error.max(f64::MIN_POSITIVE)
        );
        assert!(sampled.std_error > 0.0, "a sampled answer carries an error bar");
    }
    Ok(())
}

/// Seats asking for exactly the same thing must be given exactly the same
/// answer.
///
/// This is the cheapest check there is on a weighted sampler and the hardest
/// to fool: the true answer is known by symmetry alone, without enumerating
/// anything, so a weight that favours the seats dealt earlier shows up here
/// even in spots far too large to walk. Those are the spots that could not be
/// answered at all before.
#[test]
fn test_identical_seats_split_evenly_when_weighted() -> Result<(), PokerError> {
    let cases: [(&str, ChunkResult, usize); 3] = [
        (
            "four stud seats on the same three ranks",
            {
                let request = EquityRequest::from_text(Stud, &["A23"; 4], "", "")?;
                assert!(request.is_weighted(), "this spot stalls without weighting");
                run_chunk(&request, 30_000, 5)?
            },
            4,
        ),
        (
            "six hold'em seats each on two clubs",
            {
                let request = EquityRequest::from_text(Holdem, &["c c"; 6], "", "")?;
                assert!(request.is_weighted(), "this spot stalls without weighting");
                run_chunk(&request, 30_000, 5)?
            },
            6,
        ),
        (
            "four stud seats on the seven low ranks",
            {
                let request = EquityRequest::from_text(Stud, &["A 2 3 4 5 6 7"; 4], "", "")?;
                assert!(request.is_weighted(), "this spot stalls without weighting");
                run_chunk(&request, 30_000, 5)?
            },
            4,
        ),
    ];

    for (label, result, seats) in cases {
        let even = 1.0 / seats as f64;
        for (seat, equity) in result.equities().iter().enumerate() {
            assert!(
                (equity.equity - even).abs() <= 4.0 * equity.std_error,
                "{}: seat {} got {:.4}% where symmetry demands {:.4}% (+/- {:.4})",
                label,
                seat,
                equity.percent(),
                even * 100.0,
                equity.margin_percent()
            );
        }
        let total: f64 = result.equities().iter().map(|e| e.equity).sum();
        assert!((total - 1.0).abs() < 1e-9, "{}: equities summed to {}", label, total);
    }
    Ok(())
}

/// Weighting is for spots that have stopped working, and nothing else.
///
/// The floor is a containment decision as much as a performance one: a bug in
/// the weighted path must not be able to reach a hand somebody actually
/// holds. So the ordinary spots are named here, and they must all be dealt
/// the old way however fast the alternative might be.
#[test]
fn test_ordinary_spots_are_never_weighted() -> Result<(), PokerError> {
    let holdem: [&[&str]; 4] = [
        &["AhKh", "QsQd"],
        &["A *", "* *"],
        &["AKs", "22+"],
        &["AhKh", "QsQd", "7c7d", "JsTh", "4c4h", "9s8s"],
    ];
    for hands in holdem {
        let request = EquityRequest::from_text(Holdem, hands, "", "")?;
        assert!(!request.is_weighted(), "{:?} is an ordinary spot", hands);
    }

    let request = EquityRequest::from_text(Omaha, &["AA**", "KK**"], "", "")?;
    assert!(!request.is_weighted(), "Omaha aces against kings keeps 82% of its deals");

    let request = EquityRequest::from_text(Stud, &["A23", "456"], "", "")?;
    assert!(!request.is_weighted(), "stud A23 against 456 keeps a third of its deals");

    Ok(())
}

/// And the spots below the floor must take it, since that is the whole point.
///
/// These all answer today, slowly. What earns them the weighted path is not
/// that rejection has stopped working but that it has become the slower way
/// to the same answer, by between two and eleven times.
#[test]
fn test_spots_below_the_floor_are_weighted() -> Result<(), PokerError> {
    let request = EquityRequest::from_text(Holdem, &["c c"; 5], "", "")?;
    assert!(request.is_weighted(), "five club seats keep 1.1% of their deals");

    let request = EquityRequest::from_text(Stud, &["A23", "456", "789", "TJQ"], "", "")?;
    assert!(request.is_weighted(), "four stud seats naming ranks keep 2.4% of their deals");

    let request = EquityRequest::from_text(Omaha, &["c***"; 6], "", "")?;
    assert!(request.is_weighted(), "six Omaha seats each wanting a club keep 1.9%");

    Ok(())
}

/// A weight is a relative thing, so scaling every deal's weight must change
/// nothing at all -- not the equities, and not the error bar.
///
/// This pins the arithmetic rather than the sampler. It is what makes the
/// choice of reference in `deal_weighted` a matter of numerical comfort
/// instead of a decision that moves the answer.
#[test]
fn test_a_constant_weight_changes_nothing() {
    let shares = [[1.0, 0.0], [0.0, 1.0], [0.5, 0.5], [1.0, 0.0], [0.0, 1.0]];
    let none = [0.0, 0.0];

    let mut plain = ChunkResult::empty(2);
    let mut scaled = ChunkResult::empty(2);
    for deal in &shares {
        plain.record(deal, &none);
        scaled.record_weighted(deal, &none, 0.125);
    }

    for (seat, (one, other)) in plain.equities().iter().zip(scaled.equities()).enumerate() {
        assert!(
            (one.equity - other.equity).abs() < 1e-12,
            "seat {}: {} against {}",
            seat,
            one.equity,
            other.equity
        );
        assert!(
            (one.std_error - other.std_error).abs() < 1e-12,
            "seat {}: error bar {} against {}",
            seat,
            one.std_error,
            other.std_error
        );
        assert!((one.win - other.win).abs() < 1e-12, "seat {} win rate moved", seat);
    }
    assert_eq!(
        plain.effective_samples().round(),
        scaled.effective_samples().round(),
        "scaling every weight cannot change what the pile is worth"
    );
}

/// Uneven weights must cost precision, and must say so.
///
/// The effective sample size is the whole reason a weighted answer can be
/// trusted: it is what the error bar is divided by, and if it quietly stayed
/// equal to the deal count the answer would look more certain than it is.
#[test]
fn test_uneven_weights_cost_effective_samples() {
    let shares = [[1.0, 0.0], [0.0, 1.0]];
    let none = [0.0, 0.0];

    let mut even = ChunkResult::empty(2);
    let mut uneven = ChunkResult::empty(2);
    for index in 0..100 {
        let deal = &shares[index % 2];
        even.record(deal, &none);
        // One deal in fifty carries fifty times the weight of the rest.
        uneven.record_weighted(deal, &none, if index % 50 == 0 { 50.0 } else { 1.0 });
    }

    assert_eq!(even.samples, uneven.samples, "both piles hold the same deals");
    assert!(
        (even.effective_samples() - 100.0).abs() < 1e-9,
        "unweighted deals are worth their count: {}",
        even.effective_samples()
    );
    assert!(
        uneven.effective_samples() < 60.0,
        "a pile carried by a few deals is worth less than its count: {}",
        uneven.effective_samples()
    );
    assert!(
        uneven.equities()[0].std_error > even.equities()[0].std_error,
        "and the error bar has to widen to match"
    );
}

/// What each path actually costs, spot by spot, in time per usable answer.
///
/// This is what the floor should be set from. Rejection pays for the deals it
/// throws away; weighting pays a higher price per deal and throws almost none
/// away. Run with `cargo test --release -- --ignored --nocapture both_paths`.
#[test]
#[ignore = "a measurement, not an assertion; run with --ignored --nocapture"]
fn measure_both_paths() {
    use std::time::Instant;

    use crate::variants::{PokerVariant, EquityCalculation};

    fn timed<V: PokerVariant + EquityCalculation + Copy>(
        label: &str,
        variant: V,
        fields: &[&str],
        samples: u64,
    ) {
        let build = || EquityRequest::from_text(variant, fields, "", "").unwrap();

        let mut rejecting = build();
        rejecting.force_rejecting();
        let start = Instant::now();
        let reject = match run_chunk(&rejecting, samples, 5) {
            Ok(chunk) => {
                let nanos = start.elapsed().as_secs_f64() * 1e9 / chunk.samples as f64;
                Some((chunk.acceptance(), nanos))
            }
            Err(_) => None,
        };

        let mut weighted = build();
        if !weighted.force_weighted() {
            println!("{:<32} cannot be weighted", label);
            return;
        }
        let start = Instant::now();
        let weigh = match run_chunk(&weighted, samples, 5) {
            Ok(chunk) => {
                let seconds = start.elapsed().as_secs_f64();
                Some((
                    chunk.effective_samples() / chunk.samples as f64,
                    seconds * 1e9 / chunk.effective_samples(),
                ))
            }
            Err(_) => None,
        };

        let chosen = build().is_weighted();
        match (reject, weigh) {
            (Some((acceptance, reject_ns)), Some((yielded, weigh_ns))) => println!(
                "{:<32} reject {:>7.3}% kept {:>9.0} ns | weight {:>6.1}% ESS {:>8.0} ns | \
                 {} {} {:.1}x",
                label,
                100.0 * acceptance,
                reject_ns,
                100.0 * yielded,
                weigh_ns,
                if chosen { "chose weighted" } else { "chose rejecting" },
                // Choosing the slower path only matters when it is properly
                // slower. Within a fifth the two are the same speed, and the
                // floor deliberately leaves those spots where they are.
                if (weigh_ns < reject_ns) == chosen
                    || (reject_ns / weigh_ns).max(weigh_ns / reject_ns) < 1.5
                {
                    "OK"
                } else {
                    "WRONG,"
                },
                (reject_ns / weigh_ns).max(weigh_ns / reject_ns),
            ),
            (None, Some((yielded, weigh_ns))) => println!(
                "{:<32} reject STALLS            | weight {:>6.1}% ESS {:>8.0} ns | {}",
                label,
                100.0 * yielded,
                weigh_ns,
                if chosen { "chose weighted OK" } else { "chose rejecting WRONG" }
            ),
            _ => println!("{:<32} no comparison", label),
        }
    }

    println!();
    timed("holdem AhKh vs QsQd", Holdem, &["AhKh", "QsQd"], 100_000);
    timed("holdem 'A *' vs '* *'", Holdem, &["A *", "* *"], 100_000);
    timed("omaha AA** vs KK**", Omaha, &["AA**", "KK**"], 50_000);
    timed("stud A23 vs 456", Stud, &["A23", "456"], 50_000);
    timed("stud A23/456/789/TJQ", Stud, &["A23", "456", "789", "TJQ"], 20_000);
    timed("stud 4x 'A**'", Stud, &["A**"; 4], 20_000);
    timed("holdem 5x 'c c'", Holdem, &["c c"; 5], 20_000);
    timed("holdem 'A c' + 3x 'c c'", Holdem, &["A c", "c c", "c c", "c c"], 20_000);
    timed("omaha 6x 'c***'", Omaha, &["c***"; 6], 20_000);
    timed("stud 4x 'A23'", Stud, &["A23"; 4], 20_000);
    timed("holdem 6x 'c c'", Holdem, &["c c"; 6], 20_000);
    timed("stud 4x low seven", Stud, &["A 2 3 4 5 6 7"; 4], 20_000);
    println!();
}
