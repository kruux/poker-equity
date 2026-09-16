//! Instruction counts for the paths a refactor could quietly make slower.
//!
//! Every benchmark here is single-threaded and seeded. CodSpeed measures
//! these under callgrind and counts instructions rather than seconds, so the
//! figure does not depend on which machine ran it or what else that machine
//! was doing -- but callgrind serialises threads, and rayon's interleaving is
//! not reproducible anyway, so `run_chunk` is the right unit: one batch of
//! deals, on this thread, from a fixed seed.
//!
//! Sample counts are small on purpose. The comparison is between two commits
//! measuring the same work, not between this and a real run, and every
//! instruction counted is one callgrind has to walk.

use std::hint::black_box;

use codspeed_criterion_compat::{criterion_group, criterion_main, Criterion};

use poker_equity::{
    cards::Card,
    odds::{run_chunk, EquityRequest},
    variants::{Badugi, Holdem, Omaha, OmahaHiLo, PokerVariant, Razz, Stud},
};

/// The hand kernels, one seven-card holding each, with no dealing around it.
///
/// A ranking change shows up here first: this is the lookup and the tiebreak
/// and nothing else.
fn kernels(c: &mut Criterion) {
    let mut group = c.benchmark_group("kernel");

    let seven = Card::parse_field("Ah Kh Qs Qd 2c 7d 9s").expect("valid cards");
    group.bench_function("high", |b| {
        b.iter(|| black_box(Holdem.evaluate_hand(black_box(&seven))))
    });

    let low = Card::parse_field("Ah 2d 3c 5s 7h 9d Jc").expect("valid cards");
    group.bench_function("razz", |b| {
        b.iter(|| black_box(Razz.evaluate_hand(black_box(&low))))
    });

    let four = Card::parse_field("Ah 2d 3c 5s").expect("valid cards");
    group.bench_function("badugi", |b| {
        b.iter(|| black_box(Badugi.evaluate_hand(black_box(&four))))
    });

    group.finish();
}

/// A batch of deals per game: dealing, evaluating and splitting the pot.
///
/// The games differ in what costs them. Omaha pairs two hole cards with three
/// of the board sixty ways, the split-pot games score every deal twice, and
/// stud deals more cards than it shows.
fn showdowns(c: &mut Criterion) {
    let mut group = c.benchmark_group("showdown");

    macro_rules! batch {
        ($name:literal, $variant:expr, $hands:expr) => {
            let request = EquityRequest::from_text($variant, $hands, "", "").expect("a valid spot");
            group.bench_function($name, |b| {
                b.iter(|| black_box(run_chunk(black_box(&request), 1_000, 7)))
            });
        };
    }

    batch!("holdem", Holdem, &["AhKh", "QsQd"]);
    batch!("omaha", Omaha, &["AhKhQsJs", "2c2d7h8h"]);
    batch!("omaha_hi_lo", OmahaHiLo, &["AhKhQsJs", "2c2d7h8h"]);
    batch!("stud", Stud, &["A23", "456", "789"]);
    batch!("badugi", Badugi, &["Ah2d3c5s", "Kh Qd Jc 9s"]);

    group.finish();
}

criterion_group!(benches, kernels, showdowns);
criterion_main!(benches);
