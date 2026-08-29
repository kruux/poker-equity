use std::time::Instant;
use poker_calculator::{cards::Card, variants::{Omaha, OmahaFive, OmahaSix, PokerVariant, high_score}};

#[test]
fn where_does_omaha_time_go() {
    let nine = Card::from_str("Ah Kh 7c 2d Qs Jd Ts 5h 3c").unwrap();
    let five = Card::from_str("Ah Kh 7c 2d Qs").unwrap();
    let n = 1_000_000u32;

    let t = Instant::now();
    for _ in 0..n { std::hint::black_box(high_score(&five)); }
    let one = n as f64 / t.elapsed().as_secs_f64();
    println!("one five-card lookup      {:>12.0}/s   ({:.1} ns)", one, 1e9 / one);

    let t = Instant::now();
    for _ in 0..n { std::hint::black_box(Omaha.score(&nine)); }
    let omaha = n as f64 / t.elapsed().as_secs_f64();
    println!("Omaha score (60 of them)  {:>12.0}/s   ({:.1} ns)", omaha, 1e9 / omaha);
    println!("  -> {:.1} ns per inner evaluation, against {:.1} ns for the lookup alone",
             1e9 / omaha / 60.0, 1e9 / one);

    let ten = Card::from_str("Ah Kh 7c 2d 3c Qs Jd Ts 5h 4d").unwrap();
    let t = Instant::now();
    for _ in 0..n { std::hint::black_box(OmahaFive.score(&ten)); }
    println!("5-card Omaha (100)        {:>12.0}/s", n as f64 / t.elapsed().as_secs_f64());

    let eleven = Card::from_str("Ah Kh 7c 2d 3c 9s Qs Jd Ts 5h 4d").unwrap();
    let t = Instant::now();
    for _ in 0..n { std::hint::black_box(OmahaSix.score(&eleven)); }
    println!("6-card Omaha (150)        {:>12.0}/s", n as f64 / t.elapsed().as_secs_f64());
}
