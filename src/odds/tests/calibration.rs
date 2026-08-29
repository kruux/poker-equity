//! Measures every Monte Carlo expectation in the suite, so that a claimed
//! figure can be centred on the truth and its window set from the spread a
//! run of that size actually shows.
//!
//! Ignored by default: it runs millions of deals per spot and exists to be
//! run once when the expectations are set, not on every build.
//!
//!     cargo test --release calibration -- --ignored --nocapture

use crate::cards::Card;
use crate::error::PokerError;
use crate::hand::Hand;
use crate::odds::EquityCalculator;
use crate::variants::*;

/// How many deals the truth is measured over.
const TRUTH: usize = 2_000_000;

/// How many runs of the test's own size the spread is measured over.
const RUNS: usize = 12;

/// A window whose nearer edge is closer than this fails now and then.
const SAFE: f64 = 4.5;

fn judge(label: &str, truth: &std::collections::HashMap<String, f64>,
         runs: &[std::collections::HashMap<String, f64>],
         claims: &[(&str, f64, f64)]) {
    for (name, claimed, tolerance) in claims {
        let values: Vec<f64> = runs.iter().map(|r| r[*name]).collect();
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let sigma = (values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                     / (values.len() - 1).max(1) as f64).sqrt().max(1e-9);
        let edge = (tolerance - (truth[*name] - claimed).abs()) / sigma;
        if edge < SAFE {
            // Machine readable, so the fix can be applied rather than typed:
            // test, seat, what is claimed now, and what it should be.
            println!(
                "PATCH\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}",
                label, name, claimed, tolerance, truth[*name], sigma);
        }
    }
}

fn calibrate_plain<V>(variant: V, label: &str, deals: usize, seats: &[(&str, &str)],
                      _board: &str, dead: &str, claims: &[(&str, f64, f64)])
where V: PokerVariant + EquityCalculation + Send + Sync {
    let run = |n: usize| -> Result<_, PokerError> {
        let mut c = EquityCalculator::new(variant, n);
        for (name, cards) in seats {
            c.add_player(name.to_string(), Hand::from_str(variant, cards)?)?;
        }
        if !dead.is_empty() { c.add_dead_cards(Card::from_str(dead)?)?; }
        c.calculate(std::mem::drop)
    };
    let Ok(truth) = run(TRUTH) else { println!("  {} -- could not run", label); return; };
    let runs: Vec<_> = (0..RUNS).filter_map(|_| run(deals).ok()).collect();
    judge(label, &truth, &runs, claims);
}

fn calibrate_board<V>(variant: V, label: &str, deals: usize, seats: &[(&str, &str)],
                      board: &str, dead: &str, claims: &[(&str, f64, f64)])
where V: PokerVariant + EquityCalculation + CommunityCardGame + Send + Sync {
    let run = |n: usize| -> Result<_, PokerError> {
        let mut c = EquityCalculator::new(variant, n);
        for (name, cards) in seats {
            c.add_player(name.to_string(), Hand::from_str(variant, cards)?)?;
        }
        if !board.is_empty() { c.set_community_cards(Card::from_str(board)?)?; }
        if !dead.is_empty() { c.add_dead_cards(Card::from_str(dead)?)?; }
        c.calculate(std::mem::drop)
    };
    let Ok(truth) = run(TRUTH) else { println!("  {} -- could not run", label); return; };
    let runs: Vec<_> = (0..RUNS).filter_map(|_| run(deals).ok()).collect();
    judge(label, &truth, &runs, claims);
}

#[test]
#[ignore = "measures millions of deals per spot; run when setting expectations"]
fn calibrate_every_monte_carlo_expectation() {
    println!("\nspots whose nearer window edge is under {} sigma:\n", SAFE);
    calibrate_board(
        Holdem, "test_holdem_known_equities", 100000,
        &[("Hero", "Ah Kh"), ("Villain", "2h 2d"), ("Hero", "Ah Kh"), ("Villain1", "2h 2d"), ("Villain2", "Qc Qd"), ("Hero", "Ah Kh"), ("Villain", "2h 2d"), ("Hero", "Ah Kh"), ("Villain", "2h 2d")], "Ac Kc Qc Jc", "",
        &[("Hero", 49.7, 0.5), ("Villain", 50.3, 0.5), ("Hero", 38.2, 0.5), ("Villain1", 16.93, 0.5), ("Villain2", 44.87, 0.5), ("Hero", 89.9, 0.5), ("Villain", 10.1, 0.5), ("Hero", 84.09, 0.5), ("Villain", 15.91, 0.5)],
    );
    calibrate_plain(
        Holdem, "test_complex_drawing_hands", 100000,
        &[("Hero", "Ac Ts"), ("Villain", "6c 7c")], "", "",
        &[("Hero", 59.75, 0.5), ("Villain", 40.25, 0.5)],
    );
    calibrate_board(
        Holdem, "test_drawing_hands_on_wet_flop", 100000,
        &[("Hero", "Ac Ts"), ("Villain", "6c 7c")], "Jc 8c 3h", "",
        &[("Hero", 51.01, 0.5), ("Villain", 48.99, 0.5)],
    );
    calibrate_board(
        Holdem, "test_multiway_drawing_scenario", 100000,
        &[("HighCards", "Ah Kd"), ("StraightDraw", "Js Ts"), ("PocketPair", "5h 5c")], "Qc 9h 4s", "",
        &[("HighCards", 11.85, 0.5), ("StraightDraw", 50.39, 0.5), ("PocketPair", 37.76, 0.5)],
    );
    calibrate_board(
        Omaha, "test_omaha_known_equities", 100000,
        &[("Hero", "Ks Kh Qd Jc"), ("Villain", "Ts Th 9s 9h"), ("Hero", "Ks Kh Qd Jc"), ("Villain1", "As Kd Qc Jh"), ("Villain2", "5c 5d 4c 4d"), ("Hero", "Ks Kh Qd Jc"), ("Villain", "Ts Th 9s 9h"), ("Hero", "Ks Kh Qd Jc"), ("Villain", "Ts Th 9s 9h")], "Kc 2h 3s 4d", "",
        &[("Hero", 55.6, 0.5), ("Villain", 44.4, 0.5), ("Hero", 43.51, 0.5), ("Villain1", 21.22, 0.5), ("Villain2", 35.27, 0.5), ("Hero", 98.9, 0.5), ("Villain", 1.1, 0.5), ("Hero", 100.0, 0.001), ("Villain", 0.0, 0.001)],
    );
    calibrate_plain(
        Omaha, "test_complex_drawing_hands", 100000,
        &[("Hero", "Js Ts 9h 8h"), ("Villain", "As Ks Ah Kh")], "", "",
        &[("Hero", 33.64, 0.5), ("Villain", 66.36, 0.5)],
    );
    calibrate_board(
        Omaha, "test_drawing_hands_on_wet_flop", 100000,
        &[("Hero", "Js Ts 9h 8h"), ("Villain", "As Ac Kd Qc")], "7s 6c 5h", "",
        &[("Hero", 91.1, 0.5), ("Villain", 8.9, 0.5)],
    );
    calibrate_board(
        Omaha, "test_multiway_drawing_scenario", 100000,
        &[("Kings", "Ks Kh Qs Qh"), ("Rundown", "Jc Tc 9d 8d"), ("SmallPairs", "5s 5h 4s 4h")], "Kc 7d 2s", "",
        &[("Kings", 71.62, 0.5), ("Rundown", 23.87, 0.5), ("SmallPairs", 4.51, 0.5)],
    );
    calibrate_board(
        Omaha, "test_one_hole_heart_never_flushes", 20000,
        &[("Hero", "Ah 3d 4s 5c"), ("Villain", "Kd Kc 7h 8s")], "Kh Qh Jh 2c", "",
        &[("Villain", 100.0, 0.0001)],
    );
    calibrate_plain(
        Courchevel, "test_courchevel_requires_a_board_card", 1000,
        &[], "", "",
        &[("Villain", 100.0, 0.001)],
    );
    calibrate_plain(
        Razz, "test_razz_equity_wheel_vs_king_low", 100000,
        &[("Wheel", "Ah 2h 3h 4h 5h"), ("King", "Kh 2d 3d 4d 5d")], "", "",
        &[("Wheel", 93.035, 0.35), ("King", 6.965, 0.35)],
    );
    calibrate_plain(
        Razz, "test_razz_equity_tied_hands", 100000,
        &[("Player1", "2h 3h 4h 5h 6h"), ("Player2", "2d 3d 4d 5d 6d")], "", "",
        &[("Player1", 50.0, 0.5), ("Player2", 50.0, 0.5)],
    );
    calibrate_plain(
        Razz, "test_razz_equity_three_players", 100000,
        &[("Low", "Ah 2h 3h"), ("Mid", "4d 5d 6d"), ("High", "7c 8c 9c")], "", "",
        &[("Low", 43.806, 1.0), ("Mid", 38.805, 1.0), ("High", 17.389, 1.0)],
    );
    calibrate_plain(
        Razz, "test_razz_equity_drawing_hands", 100000,
        &[("LowDraw", "Ah 2h 3h"), ("MidDraw", "5h 6h 7h")], "", "",
        &[("LowDraw", 55.343, 0.75), ("MidDraw", 44.657, 0.75)],
    );
    calibrate_plain(
        Razz, "test_razz_dead_cards", 100000,
        &[("LowDraw", "Ah 2h 3h"), ("MidDraw", "4h 5h 6h")], "", "Ad 2d 3d",
        &[("LowDraw", 61.9, 1.0), ("MidDraw", 38.1, 1.0)],
    );
    calibrate_plain(
        Razz, "test_paired_low_wins_outright", 10,
        &[("Hero", "2c 2d 2h 3c 3d 4c 5d"), ("Villain", "4h 4s 4d 5h 5s 3h 2s")], "", "",
        &[("Hero", 100.0, 0.001)],
    );
    calibrate_plain(
        Stud, "test_equity_identical_hands", 100000,
        &[("Alice", "Ah Kh Qh Jh Th"), ("Bob", "As Ks Qs Js Ts")], "", "",
        &[("Alice", 50.0, 0.5), ("Bob", 50.0, 0.5)],
    );
    calibrate_plain(
        Stud, "test_equity_quads_vs_pair", 100000,
        &[("Alice", "Ah Ac Ad As 2h"), ("Bob", "Kh Kc 2c 3d 4s")], "", "",
        &[("Alice", 100.0, 0.5), ("Bob", 0.0, 0.5)],
    );
    calibrate_plain(
        Stud, "test_equity_trips_vs_three_cards", 100000,
        &[("Alice", "As Ah Ad"), ("Bob", "Qs 7h 2d")], "", "",
        &[("Alice", 97.92, 0.5), ("Bob", 2.08, 0.5)],
    );
    calibrate_plain(
        Stud, "test_equity_trips_vs_flush_draw", 100000,
        &[("Alice", "As Ah Ad"), ("Bob", "7c 6c 5c")], "", "",
        &[("Alice", 76.61, 0.5), ("Bob", 23.39, 0.5)],
    );
    calibrate_plain(
        Stud, "test_equity_three_way_trips_vs_draws", 100000,
        &[("Alice", "2s 2h 2d"), ("Bob", "7c 6c 5c"), ("Charlie", "Th 9h 8h")], "", "",
        &[("Alice", 58.98, 0.5), ("Bob", 20.89, 0.5), ("Charlie", 20.13, 0.5)],
    );
    calibrate_plain(
        Stud, "test_close_equity_stud", 100000,
        &[("Alice", "7h 7c 2s"), ("Bob", "Ah Jh 9d")], "", "",
        &[("Alice", 58.24, 0.5), ("Bob", 41.76, 0.5)],
    );
    calibrate_plain(
        StudHiLo, "test_high_only_equity", 100000,
        &[("Aces", "Ah Ac Ad Kh Qc"), ("Kings", "Ks Kc Kd Qh Jc")], "", "",
        &[("Aces", 74.96, 0.5), ("Kings", 25.04, 0.5)],
    );
    calibrate_plain(
        StudHiLo, "test_split_pot_equity", 100000,
        &[("FullHouse", "Ah Ac Ad Kh Kc"), ("LowHand", "8h 6c 4d 3h 2c")], "", "",
        &[("FullHouse", 50.0, 0.001), ("LowHand", 50.0, 0.001)],
    );
    calibrate_plain(
        StudHiLo, "test_scoop_equity", 100000,
        &[("Flush", "8h 7h 4h 3h 2h"), ("Pair", "Kh Kc 9d 8c 7c")], "", "",
        &[("Flush", 96.35, 0.5), ("Pair", 3.65, 0.5)],
    );
    calibrate_plain(
        StudHiLo, "test_three_way_equity", 100000,
        &[("Aces", "Ah Ac Ad Kh Qc"), ("EightLow", "8h 6c 4d 3h 2c"), ("BetterLow", "7h 4c 3d 2h As")], "", "",
        &[("Aces", 38.35, 0.5), ("EightLow", 13.37, 0.5), ("BetterLow", 48.28, 0.5)],
    );
    calibrate_plain(
        StudHiLo, "test_low_split_equity", 100000,
        &[("BestHigh", "8h 6c 4d 3h 2c Kh Kd"), ("SameLow", "8d 6h 4c 3d 2h Qh Qd")], "", "",
        &[("BestHigh", 75.0, 0.001), ("SameLow", 25.0, 0.001)],
    );
    calibrate_plain(
        StudHiLo, "test_six_player_three_card_equity", 100000,
        &[("Kings", "Kh Kc 9d"), ("Queens", "Qh Qc 8d"), ("A37", "Ah 3c 7d"), ("246", "2h 4c 6d"), ("345", "3h 5c 4d"), ("468", "4h 6c 8h")], "", "",
        &[("Kings", 18.67, 0.5), ("Queens", 13.0, 0.5), ("A37", 14.6, 0.5), ("246", 17.73, 0.5), ("345", 21.96, 0.5), ("468", 14.04, 0.5)],
    );
    calibrate_plain(
        StudHiLo, "test_equal_highs_split_the_high_half", 10,
        &[("Hero", "Kh Kd 8c 7d 6h 3s 2c"), ("Villain", "Ks Kc 8d 7h 6s 5c 3d")], "", "",
        &[("Hero", 75.0, 0.001), ("Villain", 25.0, 0.001)],
    );
    println!("\nanything above is worth recentring on its measured truth");
}
