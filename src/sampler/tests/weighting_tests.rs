//! Measures what live-pool weighted dealing would cost, spot by spot.
//!
//! This decides one design question: when is drawing each seat from what is
//! actually left, and weighting the deal to undo the bias that introduces,
//! cheaper than today's draw-and-reject?
//!
//! Today costs `1 / acceptance` deals per usable sample. Weighting costs
//! `1 / ESS`, where the effective sample size says how many independent deals
//! a weighted pile is really worth. So weighting wins exactly when ESS beats
//! acceptance, and this measures both.
//!
//! Not a correctness test -- it asserts only that the measurement ran. Run it
//! with `cargo test --release -- --ignored --nocapture weighting`.

use rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::cards::CardSet;
use crate::error::{EquityError, PokerError};
use crate::notation::parse_hand_up_to;
use crate::odds::{run_chunk, EquityRequest};
use crate::sampler::shape::ShapePlan;
use crate::variants::{EquityCalculation, Holdem, Omaha, PokerVariant, Stud};

/// One seat's slots, padded to the cards the game deals.
fn seat_slots(field: &str, hole: usize) -> Vec<Vec<CardSet>> {
    let spec = parse_hand_up_to(field, hole).expect("the fields here are all valid");
    spec.alternatives
        .into_iter()
        .map(|mut slots| {
            while slots.len() < hole {
                slots.push(CardSet::FULL_DECK);
            }
            slots
        })
        .collect()
}

/// Deals every seat from the live deck and returns ln(weight), or `None` when
/// a seat found nothing left to take.
///
/// The weight is the product of how many hands each seat could have taken.
/// The board is left out on purpose: its slots take any card, so it always
/// has `C(cards left, board size)` choices and always removes the same number
/// of cards, which is a constant factor and cancels.
fn deal_weighted(seats: &[Vec<Vec<CardSet>>], rng: &mut SmallRng) -> Option<f64> {
    let mut available = CardSet::FULL_DECK;
    let mut ln_weight = 0.0;
    let mut cards = Vec::new();

    for alternatives in seats {
        let slots = &alternatives[rng.gen_range(0..alternatives.len())];
        let plan = ShapePlan::build(slots, available)?;
        if plan.total() == 0 {
            return None;
        }
        ln_weight += (plan.total() as f64).ln();

        cards.clear();
        plan.draw(rng, &mut cards);
        for &card in &cards {
            available.remove(card);
        }
    }
    Some(ln_weight)
}

/// The share of deals that survive today, or `None` when the sampler gives up.
fn acceptance_today<V>(variant: V, fields: &[&str]) -> Option<f64>
where
    V: PokerVariant + EquityCalculation + Copy,
{
    let request = EquityRequest::from_text(variant, fields, "", "").ok()?;
    match run_chunk(&request, 20_000, 5) {
        Ok(result) => Some(result.samples as f64 / result.attempts as f64),
        Err(PokerError::Equity(EquityError::SamplingStalled { .. })) => Some(0.0),
        Err(_) => None,
    }
}

fn measure<V>(label: &str, variant: V, fields: &[&str], hole: usize, trials: usize)
where
    V: PokerVariant + EquityCalculation + Copy,
{
    let Some(acceptance) = acceptance_today(variant, fields) else {
        println!("{:<38} refused at construction", label);
        return;
    };

    let seats: Vec<Vec<Vec<CardSet>>> = fields.iter().map(|f| seat_slots(f, hole)).collect();
    let mut rng = SmallRng::seed_from_u64(11);
    let ln_weights: Vec<f64> = (0..trials).filter_map(|_| deal_weighted(&seats, &mut rng)).collect();

    if ln_weights.is_empty() {
        println!("{:<38} today {:>7.3}%  |  no deal completed", label, 100.0 * acceptance);
        return;
    }

    let max_ln = ln_weights.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let weights: Vec<f64> = ln_weights.iter().map(|l| (l - max_ln).exp()).collect();
    let sum: f64 = weights.iter().sum();
    let sum_sq: f64 = weights.iter().map(|w| w * w).sum();
    let ess = (sum * sum / sum_sq) / weights.len() as f64;

    // Weighting does not abolish rejection: a seat can still find nothing
    // left it can use, and that deal is thrown away exactly as it is today.
    // The yield is what survives *and* counts, so both terms belong in it.
    let completion = ln_weights.len() as f64 / trials as f64;
    let yielded = ess * completion;

    let verdict = if acceptance == 0.0 {
        "weighted -- stalls today".to_string()
    } else if yielded > acceptance * 1.5 {
        format!("weighted, {:.0}x faster", yielded / acceptance)
    } else {
        "keep today's".to_string()
    };

    println!(
        "{:<38} today {:>7.3}%  |  ESS {:>6.2}% x {:>6.2}% kept = {:>6.2}%  |  {}",
        label,
        100.0 * acceptance,
        100.0 * ess,
        100.0 * completion,
        100.0 * yielded,
        verdict
    );
}

#[test]
#[ignore = "a measurement, not an assertion; run with --ignored --nocapture"]
fn measure_where_weighting_pays() {
    println!(
        "\n{:<38} {:>13}  |  {:>38}  |  verdict",
        "spot", "kept today", "weighted yield (ESS x completion)"
    );
    println!("{}", "-".repeat(120));

    println!("\nEveryday spots\n");
    measure("holdem AhKh vs QsQd", Holdem, &["AhKh", "QsQd"], 2, 4000);
    measure("holdem 'A *' vs '* *'", Holdem, &["A *", "* *"], 2, 4000);
    measure("omaha AA** vs KK**", Omaha, &["AA**", "KK**"], 4, 4000);
    measure("stud A23 vs 456", Stud, &["A23", "456"], 7, 4000);

    println!("\nBorderline: slow today, but they do answer\n");
    measure("stud A23/456/789/TJQ", Stud, &["A23", "456", "789", "TJQ"], 7, 4000);
    measure("stud 4x 'A**'", Stud, &["A**"; 4], 7, 4000);
    measure("stud A23/A23/***/***", Stud, &["A23", "A23", "***", "***"], 7, 4000);
    measure("stud A23/A23/A23/***", Stud, &["A23", "A23", "A23", "***"], 7, 4000);
    measure("omaha 2x AA** + 4x ****", Omaha, &["AA**", "AA**", "****", "****", "****", "****"], 4, 4000);
    measure("omaha 6x 'c***'", Omaha, &["c***"; 6], 4, 4000);
    measure("holdem 'A c' + 3x 'c c'", Holdem, &["A c", "c c", "c c", "c c"], 2, 4000);
    measure("holdem 5x 'c c'", Holdem, &["c c"; 5], 2, 4000);

    println!("\nStalls today\n");
    measure("stud 4x 'A23'", Stud, &["A23"; 4], 7, 4000);
    measure("stud 4x 'A 2 3 4 5 6 7'", Stud, &["A 2 3 4 5 6 7"; 4], 7, 1000);
    measure("holdem 6x 'c c'", Holdem, &["c c"; 6], 2, 4000);
    measure("omaha 6x 'cc**'", Omaha, &["cc**"; 6], 4, 4000);
    println!();
}

/// Where a weighted deal's time actually goes.
///
/// A plan is rebuilt from nothing for every seat of every deal, so this is
/// the cost that decides whether weighting is worth choosing at all. Run with
/// `cargo test --release -- --ignored --nocapture plan_cost`.
#[test]
#[ignore = "a measurement, not an assertion; run with --ignored --nocapture"]
fn measure_plan_cost() {
    use std::time::Instant;

    use crate::cards::Rank;

    let ranks = |rs: &[Rank]| -> Vec<CardSet> {
        rs.iter().map(|&r| CardSet::of_rank(r)).collect()
    };
    let pad = |mut slots: Vec<CardSet>, to: usize| {
        while slots.len() < to {
            slots.push(CardSet::FULL_DECK);
        }
        slots
    };

    use Rank::*;
    let cases: Vec<(&str, Vec<CardSet>)> = vec![
        ("holdem 'c c'", vec![CardSet::of_suit(crate::cards::Suit::Club); 2]),
        ("stud 'A23' padded to seven", pad(ranks(&[Ace, Two, Three]), 7)),
        ("stud 'A 2 3 4 5 6 7'", ranks(&[Ace, Two, Three, Four, Five, Six, Seven])),
    ];

    println!();
    for (label, slots) in cases {
        let pool = CardSet::FULL_DECK;
        let plan = ShapePlan::build(&slots, pool).expect("these all plan");
        let (atoms, shapes) = plan.size_hint();

        let rounds = 2_000;
        let start = Instant::now();
        for _ in 0..rounds {
            std::hint::black_box(ShapePlan::build(&slots, pool));
        }
        let build = start.elapsed().as_secs_f64() * 1e9 / rounds as f64;

        let mut rng = SmallRng::seed_from_u64(1);
        let mut out = Vec::new();
        let start = Instant::now();
        for _ in 0..rounds {
            out.clear();
            plan.draw(&mut rng, &mut out);
        }
        let draw = start.elapsed().as_secs_f64() * 1e9 / rounds as f64;

        println!(
            "{:<30} {:>3} groups {:>6} shapes | build {:>8.0} ns | draw {:>7.0} ns",
            label, atoms, shapes, build, draw
        );
    }
    println!();
}

/// Re-weighing a plan must give exactly what rebuilding it would.
///
/// This is the assumption the whole optimisation rests on: that which groups
/// the slots cut the deck into, and which shapes can fill them, depend on the
/// slots and not on the deck. If that is ever false, a weighted deal is
/// counting hands that are not there, every weight is wrong, and the equities
/// go quietly wrong with them -- so it is checked against the slow path it
/// replaced rather than assumed.
#[test]
fn test_reweighing_matches_rebuilding() {
    use rand::seq::SliceRandom;

    use crate::cards::{Rank, Suit};

    let ranks = |rs: &[Rank]| -> Vec<CardSet> { rs.iter().map(|&r| CardSet::of_rank(r)).collect() };
    let pad = |mut slots: Vec<CardSet>, to: usize| {
        while slots.len() < to {
            slots.push(CardSet::FULL_DECK);
        }
        slots
    };

    use Rank::*;
    let cases: Vec<(&str, Vec<CardSet>)> = vec![
        ("two clubs", vec![CardSet::of_suit(Suit::Club); 2]),
        ("A23 padded to seven", pad(ranks(&[Ace, Two, Three]), 7)),
        ("the seven low ranks", ranks(&[Ace, Two, Three, Four, Five, Six, Seven])),
        ("an ace and a club", vec![CardSet::of_rank(Ace), CardSet::of_suit(Suit::Club)]),
        ("AA** in Omaha", pad(ranks(&[Ace, Ace]), 4)),
        ("a named card among wildcards", pad(vec![CardSet::from_cards(&["Ah".parse().unwrap()])], 5)),
    ];

    let mut rng = SmallRng::seed_from_u64(7);
    let mut groups = Vec::new();
    let mut cumulative = Vec::new();

    for (label, slots) in cases {
        let plan = ShapePlan::build(&slots, CardSet::FULL_DECK).expect("these all plan");

        // Whole decks, and decks with anything from a few to most of the
        // cards already dealt out of them.
        for removed in [0usize, 1, 5, 13, 26, 39, 44] {
            let mut deck: Vec<_> = CardSet::FULL_DECK.iter().collect();
            deck.shuffle(&mut rng);
            let mut pool = CardSet::FULL_DECK;
            for &card in deck.iter().take(removed) {
                pool.remove(card);
            }

            let reweighed = plan.weigh(pool, &mut groups, &mut cumulative);
            let rebuilt = ShapePlan::build(&slots, pool).map(|fresh| fresh.total()).unwrap_or(0);

            assert_eq!(
                reweighed, rebuilt,
                "{}: with {} cards gone, re-weighing counted {} hands and rebuilding counted {}",
                label, removed, reweighed, rebuilt
            );
        }
    }
}
