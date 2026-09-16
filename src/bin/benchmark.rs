//! Reports how many showdowns a second each variant manages.
//!
//! This is a report, not a gate. Perf assertions in CI are flaky and get
//! muted, which is worse than not having them; the only assertion anywhere is
//! the very loose floor in the test suite that catches a debug build being
//! shipped as a release one.
//!
//! Run with `cargo run --release --bin benchmark`.

use std::time::Instant;

use poker_equity::{
    cards::Card,
    odds::{equity, run_chunk, EquityRequest, Target},
    variants::*,
};

/// Times a variant that the chunked API can sample.
macro_rules! time_variant {
    ($variant:expr, $hands:expr, $board:expr, $deals:expr) => {{
        let request = EquityRequest::from_text($variant, $hands, $board, "")
            .expect("the benchmark's own spots should be valid");
        // A warm-up pass, so the first timing does not pay for the tables
        // being pulled into cache.
        let _ = run_chunk(&request, 2_000, 1);

        let started = Instant::now();
        let result = run_chunk(&request, $deals, 7).expect("sampling should not fail");
        let elapsed = started.elapsed();

        report(
            &$variant.to_string(),
            $hands.len(),
            result.samples as f64 / elapsed.as_secs_f64(),
        );
    }};
}

/// Reports a figure that belongs to no particular seat count.
fn report_wide(label: &str, per_second: f64) {
    println!("{:28}          {:>13.0} showdowns/s", label, per_second);
}

fn report(label: &str, seats: usize, per_second: f64) {
    let short: String = label.chars().take(26).collect();
    println!(
        "{:28} {:>2} seats  {:>13.0} showdowns/s",
        short, seats, per_second
    );
}

fn main() {
    println!("One core. Showdowns a second, including evaluation and pot splitting.\n");

    time_variant!(Holdem, &["AhKh", "QsQd"], "", 400_000);
    time_variant!(Holdem, &["AhKh", "QsQd"], "", 200_000);
    time_variant!(ShortDeck, &["AhKh", "QsQd"], "", 200_000);
    time_variant!(
        Holdem,
        &["AhKh", "QsQd", "7c2d", "JsTs", "9h9c", "4s4d"],
        "",
        200_000
    );
    println!();
    time_variant!(Omaha, &["AhKh7c2d", "QsQdJsTd"], "", 40_000);
    time_variant!(Omaha, &["AhKh7c2d", "QsQdJsTd"], "", 100_000);
    time_variant!(OmahaFive, &["AhKh7c2d3c", "QsQdJsTd4h"], "", 30_000);
    time_variant!(OmahaSix, &["AhKh7c2d3c5s", "QsQdJsTd4h6h"], "", 20_000);
    time_variant!(OmahaHiLo, &["Ah2c3d4s", "QsQdJsTd"], "", 20_000);
    time_variant!(OmahaFiveHiLo, &["Ah2c3d4s5c", "QsQdJsTd9h"], "", 20_000);
    // Courchevel deals its first board card face up, so a board of one is the
    // spot it is actually played from.
    time_variant!(Courchevel, &["AhKh7c2d3c", "QsQdJsTd4h"], "8s", 30_000);
    time_variant!(CourchevelHiLo, &["Ah2c3d4s5c", "QsQdJsTd9h"], "8s", 20_000);
    // Six-handed, where sharing the board's ten three-card halves across the
    // table rather than working them out per seat pays the most.
    time_variant!(
        Omaha,
        &["AhKh7c2d", "QsQdJsTd", "9c8c7d6d", "AsAd5h4h", "KsQh9s8h", "3c3d2h2s"],
        "",
        20_000
    );
    println!();
    time_variant!(Stud, &["AhKh7c", "QsQdJs"], "", 200_000);
    time_variant!(StudHiLo, &["Ah2c3d", "QsQdJs"], "", 100_000);
    time_variant!(Razz, &["Ah2c3d", "4s5h7c"], "", 100_000);
    println!();
    time_variant!(DeuceSeven, &["Th8c4s2h", "9d7h4h2d"], "", 100_000);
    time_variant!(Badugi, &["Ac2d3h", "4s6s7d"], "", 200_000);

    // The two answers a variant gives: `score` orders a hand by reading a
    // table, `evaluate_hand` names it by walking the cards. The loop uses the
    // first; the second is what a person reads and what the tables are
    // checked against.
    println!("\nThe two answers, without dealing or pot splitting:");
    let seven = Card::parse_field("Ah Kh Qs Qd 2c 7d 9s").expect("valid cards");

    let rounds = 2_000_000;
    let started = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(Holdem.score(&seven));
    }
    println!(
        "{:28}          {:>13.0} hands/s",
        "score (table lookup)",
        rounds as f64 / started.elapsed().as_secs_f64()
    );

    let started = Instant::now();
    for _ in 0..rounds {
        std::hint::black_box(Holdem.evaluate_hand(&seven));
    }
    println!(
        "{:28}          {:>13.0} hands/s",
        "evaluate_hand (names it)",
        rounds as f64 / started.elapsed().as_secs_f64()
    );

    // The whole machine, which is what a caller gets by raising the thread
    // count. Threads share nothing while they sample, so this should be the
    // single-thread figure above multiplied by the cores there are.
    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    println!("\nAcross every core ({} of them):", cores);
    let deals = 4_000_000;
    let request = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")
        .unwrap()
        .with_threads(cores);
    let started = Instant::now();
    equity(&request, Target::Samples(deals)).unwrap();
    report_wide(
        "hold'em, all cores",
        deals as f64 / started.elapsed().as_secs_f64(),
    );
}
