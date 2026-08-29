//! Generates the five-card lookup tables consulted by `FastHandRank`.
//!
//! The tables are written into `OUT_DIR` and `include!`d by
//! `src/variants/rankings/{hand_rank_table,rank_translation}.rs`, so no
//! generated source is checked in.
//!
//! The generator walks the *key space* rather than the deck: every rank
//! multiset of five to seven cards (at most four of any rank) and every
//! thirteen-bit flush mask of five or more bits. That is about sixty thousand
//! keys instead of a hundred and fifty-six million hands, and it covers the
//! same set of keys exactly.
//!
//! The evaluator below is deliberately independent of `HighHandRank`. The two
//! are cross-checked against each other by the test suite; sharing code here
//! would mean a bug in one silently agrees with the other.

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

// The evaluator and this generator have to agree exactly about where a key
// lands, so they read the arithmetic from the same file rather than each
// keeping a copy.
include!("src/variants/rankings/table_index.rs");

/// Rank indices: 0 = Two, 12 = Ace. Index *is* rank order, ace high.
const RANK_IDENTS: [&str; 13] = [
    "Two", "Three", "Four", "Five", "Six", "Seven", "Eight", "Nine", "Ten", "Jack", "Queen",
    "King", "Ace",
];

/// Base-five place values. With at most four of any rank the digits never
/// carry, so a multiset maps to exactly one sum.
const RANK_KEYS: [u32; 13] = [
    1,
    5,
    25,
    125,
    625,
    3_125,
    15_625,
    78_125,
    390_625,
    1_953_125,
    9_765_625,
    48_828_125,
    244_140_625,
];

const STRAIGHT_FLUSH: u8 = 9;
const FOUR_OF_A_KIND: u8 = 8;
const FULL_HOUSE: u8 = 7;
const FLUSH: u8 = 6;
const STRAIGHT: u8 = 5;
const THREE_OF_A_KIND: u8 = 4;
const TWO_PAIR: u8 = 3;
const PAIR: u8 = 2;
const HIGH_CARD: u8 = 1;

/// A hand's strength, as the generator sees it.
///
/// `tiebreak` holds the ranks that decide hands of the same category, highest
/// first, stored as `rank index + 1` so that an unused slot is zero and sorts
/// below every real rank. Deriving `Ord` then gives the whole comparison:
/// category first, then the tiebreak ranks in order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct RefRank {
    category: u8,
    tiebreak: [u8; 5],
}

impl RefRank {
    fn new(category: u8, ranks: &[u8]) -> Self {
        let mut tiebreak = [0u8; 5];
        for (slot, &rank) in tiebreak.iter_mut().zip(ranks) {
            *slot = rank + 1;
        }
        Self {
            category,
            tiebreak,
        }
    }

    /// The matching `HighHandRank` variant, as Rust source.
    fn to_high_hand_rank(self) -> String {
        let r = |i: usize| format!("Rank::{}", RANK_IDENTS[(self.tiebreak[i] - 1) as usize]);
        match self.category {
            STRAIGHT_FLUSH => format!("HighHandRank::StraightFlush({})", r(0)),
            FOUR_OF_A_KIND => format!("HighHandRank::FourOfAKind({}, {})", r(0), r(1)),
            FULL_HOUSE => format!("HighHandRank::FullHouse({}, {})", r(0), r(1)),
            FLUSH => format!(
                "HighHandRank::Flush([{}, {}, {}, {}, {}])",
                r(0),
                r(1),
                r(2),
                r(3),
                r(4)
            ),
            STRAIGHT => format!("HighHandRank::Straight({})", r(0)),
            THREE_OF_A_KIND => {
                format!("HighHandRank::ThreeOfAKind({}, [{}, {}])", r(0), r(1), r(2))
            }
            TWO_PAIR => format!("HighHandRank::TwoPair({}, {}, {})", r(0), r(1), r(2)),
            PAIR => format!(
                "HighHandRank::Pair({}, [{}, {}, {}])",
                r(0),
                r(1),
                r(2),
                r(3)
            ),
            HIGH_CARD => format!(
                "HighHandRank::HighCard([{}, {}, {}, {}, {}])",
                r(0),
                r(1),
                r(2),
                r(3),
                r(4)
            ),
            other => unreachable!("unknown hand category {}", other),
        }
    }
}

/// The ace playing low in a full deck: A-5-4-3-2.
const WHEEL: u16 = (1 << 12) | 0b1111;

/// The ace playing low in a short deck: A-9-8-7-6. There is nothing below a
/// six to make the usual wheel from.
const SHORT_WHEEL: u16 = (1 << 12) | (0b1111 << 4);

/// The highest straight in a rank mask, as a rank index.
///
/// `wheel` is the hand the ace plays low in, or `None` where it does not play
/// low at all -- deuce-to-seven counts the ace as high always, which is why
/// `A5432` is a bad high card hand there rather than a straight.
fn straight_high(mask: u16, wheel: Option<u16>) -> Option<u8> {
    // A non-wheel straight is topped by a six or better.
    for high in (4..13u8).rev() {
        let run = 0b11111u16 << (high - 4);
        if mask & run == run {
            return Some(high);
        }
    }
    if let Some(wheel) = wheel {
        if mask & wheel == wheel {
            // The highest card of the wheel other than the ace tops it: a
            // five in a full deck, a nine in a short one.
            return Some(15 - (wheel & !(1 << 12)).leading_zeros() as u8);
        }
    }
    None
}

/// Ranks present, highest first.
fn ranks_desc(counts: &[u8; 13]) -> Vec<u8> {
    (0..13u8)
        .rev()
        .filter(|&r| counts[r as usize] > 0)
        .collect()
}

/// The best five-card hand from a rank multiset, given that no flush is
/// possible. This is the table consulted once the flush check has missed.
fn eval_no_flush(counts: &[u8; 13], wheel: Option<u16>) -> RefRank {
    let mask: u16 = (0..13)
        .filter(|&r| counts[r] > 0)
        .map(|r| 1u16 << r)
        .fold(0, |a, b| a | b);
    let present = ranks_desc(counts);
    let highest = |exclude: &[u8], n: usize| -> Vec<u8> {
        present
            .iter()
            .copied()
            .filter(|r| !exclude.contains(r))
            .take(n)
            .collect()
    };

    let quads = present.iter().copied().find(|&r| counts[r as usize] >= 4);
    if let Some(quad) = quads {
        let kicker = highest(&[quad], 1);
        return RefRank::new(FOUR_OF_A_KIND, &[quad, kicker[0]]);
    }

    let trips = present.iter().copied().find(|&r| counts[r as usize] >= 3);
    if let Some(trip) = trips {
        // A second trips can serve as the pair, so test for two or more.
        if let Some(pair) = present
            .iter()
            .copied()
            .find(|&r| r != trip && counts[r as usize] >= 2)
        {
            return RefRank::new(FULL_HOUSE, &[trip, pair]);
        }
    }

    if let Some(high) = straight_high(mask, wheel) {
        return RefRank::new(STRAIGHT, &[high]);
    }

    if let Some(trip) = trips {
        let kickers = highest(&[trip], 2);
        return RefRank::new(THREE_OF_A_KIND, &[trip, kickers[0], kickers[1]]);
    }

    let pairs: Vec<u8> = present
        .iter()
        .copied()
        .filter(|&r| counts[r as usize] >= 2)
        .collect();
    if pairs.len() >= 2 {
        let (high, low) = (pairs[0], pairs[1]);
        // A third pair is still a candidate kicker.
        let kicker = highest(&[high, low], 1);
        return RefRank::new(TWO_PAIR, &[high, low, kicker[0]]);
    }
    if pairs.len() == 1 {
        let kickers = highest(&[pairs[0]], 3);
        return RefRank::new(PAIR, &[pairs[0], kickers[0], kickers[1], kickers[2]]);
    }

    RefRank::new(HIGH_CARD, &present[..5])
}

/// The best five-card hand from the ranks of a single suit holding five or
/// more cards. Such a hand always plays as a flush or better.
fn eval_flush(mask: u16, wheel: Option<u16>) -> RefRank {
    if let Some(high) = straight_high(mask, wheel) {
        return RefRank::new(STRAIGHT_FLUSH, &[high]);
    }
    let top: Vec<u8> = (0..13u8).rev().filter(|&r| mask & (1 << r) != 0).take(5).collect();
    RefRank::new(FLUSH, &top)
}

/// The rankings this library scores hands under.
///
/// Each is a way of reading a hand, and each gets its own score table over
/// the same keys, so that a variant's hot path is a lookup whichever ranking
/// it plays by.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kernel {
    /// The five-card high hand: hold'em, Omaha, stud.
    High,
    /// The high hand read upside down, with the ace forced high so that
    /// `A5432` is a bad high-card hand rather than a straight, and straights
    /// and flushes counting against you.
    DeuceSeven,
    /// The high hand over thirty-six cards: a flush beats a full house, and
    /// the ace plays low below the six.
    ShortDeck,
    /// The ace-to-five low: straights and flushes do not count and the ace is
    /// the lowest card, so suits never matter and there is no flush table.
    LowA5,
}

impl Kernel {
    /// What the file holding this kernel's scores is called.
    fn name(self) -> &'static str {
        match self {
            Kernel::High => "high",
            Kernel::DeuceSeven => "deuce_seven",
            Kernel::ShortDeck => "short_deck",
            Kernel::LowA5 => "low_a5",
        }
    }

    /// The hand the ace plays low in, if it plays low at all.
    fn wheel(self) -> Option<u16> {
        match self {
            Kernel::High => Some(WHEEL),
            Kernel::ShortDeck => Some(SHORT_WHEEL),
            // Deuce-to-seven counts the ace as high, always.
            Kernel::DeuceSeven => None,
            Kernel::LowA5 => None,
        }
    }

    /// Whether a flush is a thing in this ranking at all.
    fn has_flushes(self) -> bool {
        !matches!(self, Kernel::LowA5)
    }

    /// Scores a rank multiset, given that no flush is present.
    fn rank_value(self, counts: &[u8; 13]) -> RefRank {
        match self {
            Kernel::LowA5 => eval_low(counts),
            _ => eval_no_flush(counts, self.wheel()),
        }
    }

    /// Scores a suit's ranks, where five or more of them are held.
    fn flush_value(self, mask: u16) -> RefRank {
        eval_flush(mask, self.wheel())
    }

    /// Which of two hands this ranking prefers, best first.
    fn better_first(self, a: &RefRank, b: &RefRank) -> std::cmp::Ordering {
        match self {
            // The best high hand wins.
            Kernel::High => b.cmp(a),
            // The worst high hand wins, which is the whole of the game.
            Kernel::DeuceSeven => a.cmp(b),
            // As High, but a flush outranks a full house.
            Kernel::ShortDeck => short_deck_order(b).cmp(&short_deck_order(a)),
            // The lowest low wins, and the encoding already sorts that way.
            Kernel::LowA5 => a.cmp(b),
        }
    }
}

/// A short-deck hand's place, with the flush lifted above the full house.
fn short_deck_order(rank: &RefRank) -> (u8, [u8; 5]) {
    let category = match rank.category {
        FLUSH => FULL_HOUSE,
        FULL_HOUSE => FLUSH,
        other => other,
    };
    (category, rank.tiebreak)
}

/// Scores a rank multiset as an ace-to-five low.
///
/// Suits never matter here, so there is no flush to check. The five cards
/// that play are whichever five make the lowest hand, and pairing is what
/// hurts: any hand with no pair beats any hand with one, one pair beats two
/// pair, and so on.
fn eval_low(counts: &[u8; 13]) -> RefRank {
    /// Where a rank sits in a low hand: the ace is the lowest card.
    fn low_value(rank: u8) -> u8 {
        if rank == 12 {
            1
        } else {
            rank + 2
        }
    }

    // Every five-card sub-multiset, scored, keeping the best. At seven cards
    // this is a handful of candidates, and trying them all is cheaper than
    // reasoning about which duplicate to keep.
    let mut ranks: Vec<u8> = Vec::new();
    for (rank, &count) in counts.iter().enumerate() {
        for _ in 0..count {
            ranks.push(rank as u8);
        }
    }

    let mut best: Option<RefRank> = None;
    let held = ranks.len();
    for choice in 0u32..(1 << held) {
        if choice.count_ones() != 5.min(held as u32) {
            continue;
        }
        let played: Vec<u8> = (0..held)
            .filter(|i| choice & (1 << i) != 0)
            .map(|i| ranks[i])
            .collect();

        // How the played ranks group up, largest first. Comparing those
        // lists is the category order: no pair is best, then one pair, two
        // pair, trips, a full house and quads.
        let mut group = [0u8; 13];
        for &rank in &played {
            group[rank as usize] += 1;
        }
        let mut sizes: Vec<u8> = group.iter().copied().filter(|&n| n > 0).collect();
        sizes.sort_unstable_by(|a, b| b.cmp(a));
        let category = match sizes.as_slice() {
            [1, 1, 1, 1, 1] => 0,
            [2, 1, 1, 1] => 1,
            [2, 2, 1] => 2,
            [3, 1, 1] => 3,
            [3, 2] => 4,
            [4, 1] => 5,
            // Fewer than five cards held, which only happens on a short
            // holding. Order those after every complete hand.
            _ => 6,
        };

        // Within a category the biggest group decides first, then the higher
        // card, and a lower card is the better low.
        let mut ordered = played.clone();
        ordered.sort_by(|a, b| {
            group[*b as usize]
                .cmp(&group[*a as usize])
                .then(low_value(*b).cmp(&low_value(*a)))
        });
        let mut tiebreak = [0u8; 5];
        for (slot, rank) in tiebreak.iter_mut().zip(&ordered) {
            *slot = low_value(*rank);
        }

        let candidate = RefRank {
            category,
            tiebreak,
        };
        if best.as_ref().is_none_or(|found| candidate < *found) {
            best = Some(candidate);
        }
    }

    best.unwrap_or(RefRank {
        category: u8::MAX,
        tiebreak: [u8::MAX; 5],
    })
}

/// Every rank multiset of `size` cards holding at most four of any rank.
fn enumerate_multisets(size: u8) -> Vec<[u8; 13]> {
    fn walk(counts: &mut [u8; 13], rank: usize, left: u8, out: &mut Vec<[u8; 13]>) {
        if rank == 13 {
            if left == 0 {
                out.push(*counts);
            }
            return;
        }
        for taken in 0..=left.min(4) {
            counts[rank] = taken;
            walk(counts, rank + 1, left - taken, out);
        }
        counts[rank] = 0;
    }
    let mut out = Vec::new();
    walk(&mut [0u8; 13], 0, size, &mut out);
    out
}

fn main() -> std::io::Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/variants/rankings/table_index.rs");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    let out_dir = Path::new(&out_dir);

    // Every rank multiset of five to seven cards. A hand reaches the rank
    // table only once the flush check has missed, so all of them are needed.
    let mut multisets: Vec<(u32, [u8; 13])> = Vec::new();
    for size in 5..=7u8 {
        for counts in enumerate_multisets(size) {
            let key: u32 = (0..13).map(|r| counts[r] as u32 * RANK_KEYS[r]).sum();
            multisets.push((key, counts));
        }
    }

    // One placement, shared by every kernel. The keys are the same whichever
    // ranking reads them, so the displacements are worked out once and each
    // kernel only needs its own array of scores.
    let keys: Vec<u32> = multisets.iter().map(|(key, _)| *key).collect();
    let displacements = place_keys(&keys);
    write_u16s(&out_dir.join("hand_displacements.bin"), &displacements)?;

    let flush_masks: Vec<u16> = (0u16..(1 << 13)).filter(|m| m.count_ones() >= 5).collect();

    let mut generated = Vec::new();
    for kernel in [
        Kernel::High,
        Kernel::DeuceSeven,
        Kernel::ShortDeck,
        Kernel::LowA5,
    ] {
        let scored = score_kernel(kernel, &multisets, &flush_masks);

        let mut hand_values = vec![NOTHING; SLOT_COUNT];
        for (key, score) in &scored.hands {
            hand_values[slot_of(*key, displacements[bucket_of(*key)])] = *score;
        }
        write_u16s(
            &out_dir.join(format!("{}_hand_scores.bin", kernel.name())),
            &hand_values,
        )?;
        generated.push((kernel.name(), "hand_scores"));

        if kernel.has_flushes() {
            let mut flush_values = vec![NOTHING; 1 << 13];
            for (mask, score) in &scored.flushes {
                flush_values[*mask as usize] = *score;
            }
            write_u16s(
                &out_dir.join(format!("{}_flush_scores.bin", kernel.name())),
                &flush_values,
            )?;
            generated.push((kernel.name(), "flush_scores"));
        }

        if kernel == Kernel::High {
            write_translation_maps(out_dir, &scored.by_score)?;
        }

        if kernel == Kernel::LowA5 {
            // A low qualifies for the eight-or-better half when it has five
            // distinct ranks, none above an eight. Those are the best lows
            // there are, so they occupy the scores below a threshold rather
            // than being scattered -- which turns the qualifier into a
            // comparison instead of a second evaluation.
            let qualifying = scored
                .by_score
                .iter()
                .filter(|(_, rank)| {
                    rank.category == 0 && rank.tiebreak.iter().all(|&value| value <= 8)
                })
                .count();
            let mut file = BufWriter::new(File::create(out_dir.join("low_qualifier.rs"))?);
            writeln!(
                file,
                "/// The first score that is *not* an eight-or-better low.\n\
                 ///\n\
                 /// Qualifying lows are the best lows there are, so they take\n\
                 /// the scores below this and the check is one comparison.\n\
                 pub(crate) const EIGHT_OR_BETTER_LIMIT: u16 = {};",
                qualifying
            )?;
            file.flush()?;
            println!("cargo:warning=eight-or-better lows: {} of them", qualifying);
        }

        println!(
            "cargo:warning={} kernel: {} distinct hand values",
            kernel.name(),
            scored.by_score.len()
        );
    }

    write_table_module(out_dir, &generated)?;

    let kilobytes = (SLOT_COUNT * 2 * 4 + BUCKET_COUNT * 2 + 8192 * 2 * 3) / 1024;
    println!("cargo:warning=lookup tables: {} KB in total", kilobytes);
    Ok(())
}

/// One kernel's scores, and what each score means.
struct Scored {
    hands: Vec<(u32, u16)>,
    flushes: Vec<(u16, u16)>,
    by_score: Vec<(u16, RefRank)>,
}

/// Numbers every hand this kernel can see, best first, so that a flush score
/// and a rank score are directly comparable and equal hands share a score.
fn score_kernel(kernel: Kernel, multisets: &[(u32, [u8; 13])], flush_masks: &[u16]) -> Scored {
    let mut all: Vec<(bool, u32, RefRank)> = multisets
        .iter()
        .map(|(key, counts)| (false, *key, kernel.rank_value(counts)))
        .collect();
    if kernel.has_flushes() {
        all.extend(
            flush_masks
                .iter()
                .map(|mask| (true, *mask as u32, kernel.flush_value(*mask))),
        );
    }

    all.sort_by(|a, b| kernel.better_first(&a.2, &b.2).then(a.1.cmp(&b.1)));

    let mut scored = Scored {
        hands: Vec::new(),
        flushes: Vec::new(),
        by_score: Vec::new(),
    };
    let mut score: u16 = 0;
    let mut previous: Option<RefRank> = None;
    for (is_flush, key, rank) in all {
        if let Some(prev) = previous {
            if prev != rank {
                score += 1;
            }
        }
        if previous != Some(rank) {
            scored.by_score.push((score, rank));
        }
        previous = Some(rank);
        if is_flush {
            scored.flushes.push((key as u16, score));
        } else {
            scored.hands.push((key, score));
        }
    }
    scored
}

/// Writes the module that names every generated table.
fn write_table_module(out_dir: &Path, generated: &[(&str, &str)]) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(out_dir.join("hand_rank_table.rs"))?);
    writeln!(
        file,
        "/// The displacement each bucket needs to clear its collisions,\n\
         /// shared by every kernel because they all key on the same hands.\n\
         pub(crate) static HAND_DISPLACEMENTS: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/hand_displacements.bin\"));"
    )?;
    for (kernel, kind) in generated {
        writeln!(
            file,
            "pub(crate) static {}_{}: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/{}_{}.bin\"));",
            kernel.to_uppercase(),
            kind.to_uppercase(),
            kernel,
            kind
        )?;
    }
    file.flush()
}

/// The value stored where no hand belongs, and the score a hand too short to
/// evaluate reports.
const NOTHING: u16 = u16::MAX;

/// Writes a slice of scores as little-endian bytes.
fn write_u16s(path: &Path, values: &[u16]) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(path)?);
    for value in values {
        file.write_all(&value.to_le_bytes())?;
    }
    file.flush()
}

/// Places every key in its own slot, and reports the displacement each bucket
/// needed to get there.
///
/// The placement depends only on the keys, not on what they are worth, so
/// every kernel shares it and only the arrays of scores differ.
///
/// This is the usual displacement construction. Keys are scattered into
/// buckets; each bucket is then given a displacement that shifts its handful
/// of keys onto slots nobody has claimed. Crowded buckets are placed first,
/// because they have the fewest arrangements left to them once the table
/// fills up -- leaving them until last is what makes a search like this fail.
fn place_keys(keys: &[u32]) -> Vec<u16> {
    let mut buckets: Vec<Vec<u32>> = vec![Vec::new(); BUCKET_COUNT];
    for &key in keys {
        buckets[bucket_of(key)].push(key);
    }

    let mut order: Vec<usize> = (0..BUCKET_COUNT).collect();
    order.sort_by_key(|&bucket| std::cmp::Reverse(buckets[bucket].len()));

    let mut displacements = vec![0u16; BUCKET_COUNT];
    let mut taken = vec![false; SLOT_COUNT];
    let mut slots = Vec::new();

    for bucket in order {
        if buckets[bucket].is_empty() {
            continue;
        }

        let displacement = (0..=u16::MAX)
            .find(|&displacement| {
                slots.clear();
                buckets[bucket].iter().all(|&key| {
                    let slot = slot_of(key, displacement);
                    let free = !taken[slot] && !slots.contains(&slot);
                    if free {
                        slots.push(slot);
                    }
                    free
                })
            })
            .unwrap_or_else(|| {
                panic!(
                    "no displacement seats bucket {} of {} keys; the table needs to be larger",
                    bucket,
                    buckets[bucket].len()
                )
            });

        for &key in &buckets[bucket] {
            taken[slot_of(key, displacement)] = true;
        }
        displacements[bucket] = displacement;
    }

    println!(
        "cargo:warning=perfect hash: {} keys into {} slots ({}% full), largest displacement {}",
        keys.len(),
        SLOT_COUNT,
        keys.len() * 100 / SLOT_COUNT,
        displacements.iter().max().copied().unwrap_or(0),
    );

    displacements
}

fn write_translation_maps(
    out_dir: &Path,
    score_to_rank: &[(u16, RefRank)],
) -> std::io::Result<()> {
    let mut file = BufWriter::new(File::create(out_dir.join("rank_translation.rs"))?);

    writeln!(file, "use super::{{FastHandRank, HighHandRank}};")?;
    writeln!(file, "use crate::cards::Rank;")?;
    writeln!(file)?;

    writeln!(
        file,
        "pub fn high_to_fast(high_rank: &HighHandRank) -> FastHandRank {{"
    )?;
    writeln!(file, "    match high_rank {{")?;
    for (score, rank) in score_to_rank {
        writeln!(
            file,
            "        {} => FastHandRank({}),",
            rank.to_high_hand_rank(),
            score
        )?;
    }
    writeln!(
        file,
        "        HighHandRank::Incomplete(_) => FastHandRank(u16::MAX),"
    )?;
    writeln!(
        file,
        "        other => panic!(\"invalid HighHandRank value {{:?}}\", other),"
    )?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;
    writeln!(file)?;

    writeln!(
        file,
        "pub fn fast_to_high(fast_rank: FastHandRank) -> HighHandRank {{"
    )?;
    writeln!(file, "    match fast_rank.0 {{")?;
    for (score, rank) in score_to_rank {
        writeln!(file, "        {} => {},", score, rank.to_high_hand_rank())?;
    }
    writeln!(
        file,
        "        other => panic!(\"invalid FastHandRank score {{}}\", other),"
    )?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    file.flush()
}
