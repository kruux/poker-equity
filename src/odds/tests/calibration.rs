//! Recomputes every equity the suite asserts, so the figures can be checked
//! or reset without leaving the repository.
//!
//! Ignored by default: it walks or samples millions of deals and exists to be
//! run when an expectation is set, not on every build.
//!
//!     cargo test --release calibration -- --ignored --nocapture
//!
//! Each line comes back one of two ways. `EXACT` means the space was walked,
//! so the figure is the answer and the test that uses it needs no window.
//! `SAMPLE` means it was measured, and the `+-` is the window a test should
//! allow: five standard errors at the deal count the suite actually runs, so
//! an unlucky run cannot fail. Compare a line against the assertion that
//! quotes it; a gap of more than the window is a real disagreement.

use crate::{
    error::PokerError,
    odds::{equity, run_exact_within, EquityRequest, Target},
    variants::*,
};

/// How many deals a measured figure is taken over.
const TRUTH_DEALS: u64 = 5_000_000;

/// How many deals the suite runs, which is what sets the window.
const TEST_DEALS: f64 = 200_000.0;

/// Prints one spot, walked if it can be and measured if it cannot.
///
/// The walk is capped at [`TRUTH_DEALS`] rather than let run to the engine's
/// own limit: past that, sampling is both faster and accurate enough to set a
/// window from.
fn report<V: PokerVariant + EquityCalculation + Send + Sync>(
    variant: V,
    hands: &[&str],
    board: &str,
    dead: &str,
) -> Result<(), PokerError> {
    let request = EquityRequest::from_text(variant, hands, board, dead)?;
    let label = format!(
        "{} {:?}{}{}",
        variant.key(),
        hands,
        if board.is_empty() {
            String::new()
        } else {
            format!(" board {}", board)
        },
        if dead.is_empty() {
            String::new()
        } else {
            format!(" dead {}", dead)
        },
    );

    match run_exact_within(&request, TRUTH_DEALS as usize)? {
        Some(result) => {
            let shares: Vec<String> = result
                .equities()
                .iter()
                .map(|player| format!("{:.6}", player.percent()))
                .collect();
            println!(
                "EXACT  {:<62} [{}]  ({} deals)",
                label,
                shares.join(", "),
                result.samples
            );
        }
        None => {
            let result = equity(&request, Target::Samples(TRUTH_DEALS))?;
            let shares: Vec<String> = result
                .equities()
                .iter()
                .map(|player| {
                    let window =
                        5.0 * (player.equity * (1.0 - player.equity) / TEST_DEALS).sqrt() * 100.0;
                    format!("{:.3} +- {:.2}", player.percent(), window)
                })
                .collect();
            println!("SAMPLE {:<62} [{}]", label, shares.join(", "));
        }
    }
    Ok(())
}

#[test]
#[ignore = "recalibration; run with --ignored --nocapture"]
fn recompute_every_expectation() -> Result<(), PokerError> {
    println!();

    // Hold'em -- every one of these is walked.
    report(Holdem, &["AhKh", "2h2d"], "", "")?;
    report(Holdem, &["AhKh", "2h2d"], "AcKcQc", "")?;
    report(Holdem, &["AhKh", "2h2d"], "AcKcQcJc", "")?;
    report(Holdem, &["AcTs", "6c7c"], "", "")?;
    report(Holdem, &["AcTs", "6c7c"], "Jc8c3h", "")?;

    // Seven-card stud.
    report(Stud, &["AhKhQhJhTh9s8s", "2c3c4c5c6c7d8d"], "", "")?;
    report(Stud, &["AhKhQhJhTh", "AsKsQsJsTs"], "", "")?;
    report(Stud, &["AhAcAdAs2h", "KhKc2c3d4s"], "", "")?;
    report(Stud, &["AsAhAd", "Qs7h2d"], "", "")?;
    report(Stud, &["AsAhAd", "7c6c5c"], "", "")?;
    report(Stud, &["2s2h2d", "7c6c5c", "Th9h8h"], "", "")?;
    report(Stud, &["7h7c2s", "AhJh9d"], "", "")?;

    // Razz.
    report(Razz, &["Ah2h3h4h5h", "Kh2d3d4d5d"], "", "")?;
    report(Razz, &["2h3h4h5h6h", "2d3d4d5d6d"], "", "")?;
    report(Razz, &["2c2d2h3c3d4c5d", "4h4s4d5h5s3h2s"], "", "")?;
    report(Razz, &["Ah2h3h", "4d5d6d", "7c8c9c"], "", "")?;
    report(Razz, &["Ah2h3h", "5h6h7h"], "", "")?;
    report(Razz, &["Ah2h3h", "4h5h6h"], "", "Ad2d3d")?;

    // Seven-card stud hi/lo.
    report(StudHiLo, &["AhAcAdKhQc", "KsKcKdQhJc"], "", "")?;
    report(StudHiLo, &["AhAcAdKhKc", "8h6c4d3h2c"], "", "")?;
    report(StudHiLo, &["8h7h4h3h2h", "KhKc9d8c7c"], "", "")?;
    report(StudHiLo, &["KhKd8c7d6h3s2c", "KsKc8d7h6s5c3d"], "", "")?;
    report(StudHiLo, &["8h6c4d3h2cKhKd", "8d6h4c3d2hQhQd"], "", "")?;
    report(
        StudHiLo,
        &["AhAcAdKhQc", "8h6c4d3h2c", "7h4c3d2hAs"],
        "",
        "",
    )?;
    report(
        StudHiLo,
        &["KhKc9d", "QhQc8d", "Ah3c7d", "2h4c6d", "3h5c4d", "4h6c8h"],
        "",
        "",
    )?;

    // 2-7 lowball, single draw -- all walked, since there is no board.
    report(DeuceSeven, &["7d5h4c3s2h", "7c5s4h3d2c"], "", "")?;
    report(
        DeuceSeven,
        &["7d5h4c3s2h", "7c5s4h3d2c", "7s5d4d3h2s"],
        "",
        "",
    )?;
    report(DeuceSeven, &["7d5h4c3s2h", "7c6s4h3d2c"], "", "")?;
    report(DeuceSeven, &["7d5h4c3s2h", "5s4h3d2c"], "", "Ad")?;
    report(DeuceSeven, &["7d5h4c3s2h", "5s4h3d2c"], "", "Ad 7h 7s")?;
    report(DeuceSeven, &["7d5h4c3s2h", "4h2c"], "", "Ad Kd")?;
    report(DeuceSeven, &["7d5h4c3s2h", "4h2c"], "", "Ks Qd")?;
    report(DeuceSeven, &["9h8c4d2h", "9d7h5s2d"], "", "As Kd")?;
    report(DeuceSeven, &["Th8c4s2h", "9d7h4h2d"], "", "Kd Ks")?;

    // Badugi.
    report(Badugi, &["Ac2d3h5s", "4s7d8h"], "", "6s")?;

    // Omaha, walked from the flop on and sampled before it.
    report(Omaha, &["AhAsKdQc", "JhTc9s8d"], "AdAc2h", "")?;
    report(Omaha, &["AsAhKdQc", "JhTc9s8d"], "Ac2h3s", "")?;
    report(Omaha, &["KsKhQdJc", "TsTh9s9h"], "Kc2h3s4d", "")?;
    report(Omaha, &["JsTs9h8h", "AsAcKdQc"], "7s6c5h", "")?;
    report(Omaha, &["KsKhQsQh", "JcTc9d8d", "5s5h4s4h"], "Kc7d2s", "")?;
    report(Omaha, &["Ah3d4s5c", "KdKc7h8s"], "KhQhJh2c", "")?;
    report(Omaha, &["9s9h8s8h", "AsKdQcJh"], "", "")?;
    report(Omaha, &["TsTh9s9h", "AsKdQcJh", "5c5d4c4d"], "", "")?;
    report(Omaha, &["JsTs9h8h", "AsKsAhKh"], "", "")?;

    // Courchevel, which starts with one board card face up.
    report(Courchevel, &["AhAdKsQcJh", "9h8c7s6d5h"], "2c", "")?;

    Ok(())
}
