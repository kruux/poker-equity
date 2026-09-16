//! Writes this library's hand scores out, so an outside evaluator can be
//! asked whether it agrees.
//!
//! The library's own sweeps check the lookup tables against the evaluator
//! that names the hands. Both were written here, from one reading of the
//! rules, so they can only catch a mistake in one of them -- a rule
//! misunderstood in both survives every sweep. This exists to settle that
//! from outside; see `validation/README.md`.
//!
//! Only the scores are written, in the order the hands enumerate, so the two
//! sides agree on which hand each number belongs to without exchanging the
//! hands themselves. Cards are numbered `rank * 4 + suit`, ranks
//! `23456789TJQKA` and suits `cdhs`, and hands come out in ascending
//! combination order.
//!
//! Run with `cargo run --release --bin export_scores -- <directory>`.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use poker_equity::cards::{Card, CardSet};
use poker_equity::variants::{
    deuce_seven_score, high_score, low_a5_score, short_deck_score, Badugi, Omaha, PokerVariant,
};

/// Every combination of `size` cards from `deck`, in ascending order.
fn combinations(deck: &[Card], size: usize) -> Vec<Vec<Card>> {
    let mut out = Vec::new();
    let mut chosen = Vec::with_capacity(size);

    fn walk(
        deck: &[Card],
        size: usize,
        from: usize,
        chosen: &mut Vec<Card>,
        out: &mut Vec<Vec<Card>>,
    ) {
        if chosen.len() == size {
            out.push(chosen.clone());
            return;
        }
        for index in from..deck.len() {
            chosen.push(deck[index]);
            walk(deck, size, index + 1, chosen, out);
            chosen.pop();
        }
    }

    walk(deck, size, 0, &mut chosen, &mut out);
    out
}

/// Writes one score per hand, as little-endian `u16`s.
fn write(path: &Path, scores: &[u16]) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    for score in scores {
        file.write_all(&score.to_le_bytes())?;
    }
    file.flush()
}

/// Numbers the distinct values a scoreless ranking produces, best first.
///
/// Badugi has no lookup table, so its ordering is turned into indices here
/// the same way a table would have numbered them.
fn dense_ranks<T, F>(hands: &[Vec<Card>], evaluate: F) -> Vec<u16>
where
    T: PartialOrd,
    F: Fn(&[Card]) -> T,
{
    let values: Vec<T> = hands.iter().map(|hand| evaluate(hand)).collect();
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| {
        values[b]
            .partial_cmp(&values[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut scores = vec![0u16; values.len()];
    let mut score: u16 = 0;
    for (position, &index) in order.iter().enumerate() {
        if position > 0 {
            let previous = order[position - 1];
            if values[previous]
                .partial_cmp(&values[index])
                .unwrap_or(std::cmp::Ordering::Equal)
                != std::cmp::Ordering::Equal
            {
                score += 1;
            }
        }
        scores[index] = score;
    }
    scores
}

/// Deals random hands from `deck`, from a fixed seed so a disagreement can be
/// looked at again.
fn sample(deck: &[Card], size: usize, count: usize, seed: u64) -> Vec<Vec<Card>> {
    let mut state = seed;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    let mut hands = Vec::with_capacity(count);
    for _ in 0..count {
        let mut chosen: Vec<usize> = Vec::with_capacity(size);
        while chosen.len() < size {
            let pick = (next() % deck.len() as u64) as usize;
            if !chosen.contains(&pick) {
                chosen.push(pick);
            }
        }
        hands.push(chosen.into_iter().map(|i| deck[i]).collect());
    }
    hands
}

/// Writes hands and their scores as text, for the checks whose hands are
/// sampled rather than enumerated and so cannot be implied by their order.
fn write_hands(
    path: &Path,
    hands: &[Vec<Card>],
    split: usize,
    scores: &[u16],
) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    for (hand, score) in hands.iter().zip(scores) {
        let text = |cards: &[Card]| {
            cards
                .iter()
                .map(|card| card.to_string())
                .collect::<Vec<_>>()
                .join("")
        };
        if split == 0 {
            writeln!(file, "{}\t{}", text(hand), score)?;
        } else {
            // Omaha and the like, where the hole cards and the board are
            // handed over separately.
            writeln!(
                file,
                "{}\t{}\t{}",
                text(&hand[..split]),
                text(&hand[split..]),
                score
            )?;
        }
    }
    file.flush()
}

fn main() -> std::io::Result<()> {
    let out = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: export_scores <directory>");
        std::process::exit(2);
    });
    let out = Path::new(&out);
    std::fs::create_dir_all(out)?;

    let full: Vec<Card> = CardSet::FULL_DECK.iter().collect();
    let short: Vec<Card> = CardSet::SHORT_DECK.iter().collect();

    let five = combinations(&full, 5);
    println!("{} five-card hands from the full deck", five.len());
    for (name, score) in [
        ("high", high_score as fn(&[Card]) -> u16),
        ("deuce_seven", deuce_seven_score),
        ("low_a5", low_a5_score),
    ] {
        let scores: Vec<u16> = five.iter().map(|hand| score(hand)).collect();
        write(&out.join(format!("{}.bin", name)), &scores)?;
        println!("  wrote {}", name);
    }

    let short_five = combinations(&short, 5);
    println!("{} five-card hands from the short deck", short_five.len());
    let scores: Vec<u16> = short_five
        .iter()
        .map(|hand| short_deck_score(hand))
        .collect();
    write(&out.join("short_deck.bin"), &scores)?;
    println!("  wrote short_deck");

    // Seven-card hands, where a holding can hold more than it plays. This is
    // where the multi-card table keys live, and where the straight bug this
    // library once had would have shown.
    for (name, score, deck) in [
        ("seven_high", high_score as fn(&[Card]) -> u16, &full),
        ("seven_low", low_a5_score, &full),
        ("seven_short_deck", short_deck_score, &short),
    ] {
        let hands = sample(deck, 7, 300_000, 0x51E7_2C0D_E5EE_D001);
        let scores: Vec<u16> = hands.iter().map(|hand| score(hand)).collect();
        write_hands(&out.join(format!("{}.tsv", name)), &hands, 0, &scores)?;
        println!("{} sampled seven-card hands for {}", hands.len(), name);
    }

    // Omaha, where exactly two hole cards play with exactly three of the
    // board. That rule is not in any table; it is in how the hand is built.
    let deals = sample(&full, 9, 200_000, 0x0A_4A_11_5E_ED);
    let scores: Vec<u16> = deals.iter().map(|deal| Omaha.score(deal) as u16).collect();
    write_hands(&out.join("omaha.tsv"), &deals, 4, &scores)?;
    println!("{} Omaha deals", deals.len());

    // Badugi has no table, so its own evaluator is what gets checked.
    let four = combinations(&full, 4);
    println!("{} four-card hands", four.len());
    let scores = dense_ranks(&four, |hand| Badugi.evaluate_hand(hand));
    write(&out.join("badugi.bin"), &scores)?;
    println!("  wrote badugi");

    Ok(())
}
