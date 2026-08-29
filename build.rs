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

use std::collections::HashMap;
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

/// The highest straight in a rank mask, as a rank index.
///
/// The ace plays low as well as high, so `A5432` is a five-high straight.
fn straight_high(mask: u16) -> Option<u8> {
    // A non-wheel straight is topped by a six or better.
    for high in (4..13u8).rev() {
        let run = 0b11111u16 << (high - 4);
        if mask & run == run {
            return Some(high);
        }
    }
    const WHEEL: u16 = (1 << 12) | 0b1111; // A, 5, 4, 3, 2
    if mask & WHEEL == WHEEL {
        return Some(3); // five high
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
fn eval_no_flush(counts: &[u8; 13]) -> RefRank {
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

    if let Some(high) = straight_high(mask) {
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
fn eval_flush(mask: u16) -> RefRank {
    if let Some(high) = straight_high(mask) {
        return RefRank::new(STRAIGHT_FLUSH, &[high]);
    }
    let top: Vec<u8> = (0..13u8).rev().filter(|&r| mask & (1 << r) != 0).take(5).collect();
    RefRank::new(FLUSH, &top)
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

    // Non-flush keys: a hand of five to seven cards reaches the rank table
    // only once the flush check has missed, so every multiset in that range
    // needs an entry.
    let mut hand_ranks: HashMap<u32, RefRank> = HashMap::new();
    for size in 5..=7u8 {
        for counts in enumerate_multisets(size) {
            let key: u32 = (0..13)
                .map(|r| counts[r] as u32 * RANK_KEYS[r])
                .sum();
            hand_ranks.insert(key, eval_no_flush(&counts));
        }
    }

    // Flush keys: a thirteen-bit rank mask for one suit. Fewer than five bits
    // is not a flush and must fall through to the rank table.
    let mut flush_ranks: HashMap<u32, RefRank> = HashMap::new();
    for mask in 0u16..(1 << 13) {
        if mask.count_ones() >= 5 {
            flush_ranks.insert(mask as u32, eval_flush(mask));
        }
    }

    // One dense numbering across both tables, best hand first, so that a flush
    // score and a rank score are directly comparable. Equal hands share a score.
    let mut all: Vec<(bool, u32, RefRank)> = hand_ranks
        .iter()
        .map(|(&key, &rank)| (false, key, rank))
        .chain(flush_ranks.iter().map(|(&key, &rank)| (true, key, rank)))
        .collect();
    all.sort_by(|a, b| b.2.cmp(&a.2).then(a.1.cmp(&b.1)));

    let mut hand_scores: Vec<(u32, u16)> = Vec::with_capacity(hand_ranks.len());
    let mut flush_scores: Vec<(u32, u16)> = Vec::with_capacity(flush_ranks.len());
    let mut score_to_rank: Vec<(u16, RefRank)> = Vec::new();
    let mut score: u16 = 0;
    let mut previous: Option<RefRank> = None;
    for (is_flush, key, rank) in all {
        if let Some(prev) = previous {
            if prev != rank {
                score += 1;
            }
        }
        if previous != Some(rank) {
            score_to_rank.push((score, rank));
        }
        previous = Some(rank);
        if is_flush {
            flush_scores.push((key, score));
        } else {
            hand_scores.push((key, score));
        }
    }

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is set by cargo");
    write_lookup_tables(Path::new(&out_dir), &hand_scores, &flush_scores)?;
    println!(
        "cargo:warning=lookup tables: {} rank keys in {} slots, {} KB total",
        hand_scores.len(),
        SLOT_COUNT,
        (SLOT_COUNT * 2 + BUCKET_COUNT * 2 + 8192 * 2) / 1024
    );
    write_translation_maps(Path::new(&out_dir), &score_to_rank)?;
    Ok(())
}

/// The value stored where no hand belongs, and the score a hand too short to
/// evaluate reports.
const NOTHING: u16 = u16::MAX;

/// Writes the two lookup tables, plus the displacements the rank table needs.
///
/// The scores go out as raw little-endian `u16`s rather than as Rust source.
/// A hundred and thirty thousand array literals would be slow to compile and
/// enormous to read, and nobody reads a generated table anyway -- the
/// generator above is the part worth reviewing.
fn write_lookup_tables(
    out_dir: &Path,
    hand_scores: &[(u32, u16)],
    flush_scores: &[(u32, u16)],
) -> std::io::Result<()> {
    // Flush keys are thirteen-bit rank masks, so they index a dense table
    // directly with nothing computed at all.
    let mut flushes = vec![NOTHING; 1 << 13];
    for (key, score) in flush_scores {
        flushes[*key as usize] = *score;
    }
    write_u16s(&out_dir.join("flush_scores.bin"), &flushes)?;

    let (displacements, values) = build_perfect_hash(hand_scores);
    write_u16s(&out_dir.join("hand_displacements.bin"), &displacements)?;
    write_u16s(&out_dir.join("hand_scores.bin"), &values)?;

    let mut file = BufWriter::new(File::create(out_dir.join("hand_rank_table.rs"))?);
    writeln!(
        file,
        "/// Scores for every rank multiset, behind the perfect hash in\n\
         /// `table_index`. Little-endian `u16`s.\n\
         pub(crate) static HAND_SCORES: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/hand_scores.bin\"));"
    )?;
    writeln!(
        file,
        "/// The displacement each bucket needs to clear its collisions.\n\
         pub(crate) static HAND_DISPLACEMENTS: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/hand_displacements.bin\"));"
    )?;
    writeln!(
        file,
        "/// Scores for every thirteen-bit flush mask, indexed directly.\n\
         pub(crate) static FLUSH_SCORES: &[u8] = include_bytes!(concat!(env!(\"OUT_DIR\"), \"/flush_scores.bin\"));"
    )?;
    file.flush()
}

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
/// This is the usual displacement construction. Keys are scattered into
/// buckets; each bucket is then given a displacement that shifts its handful
/// of keys onto slots nobody has claimed. Crowded buckets are placed first,
/// because they have the fewest arrangements left to them once the table
/// fills up -- leaving them until last is what makes a search like this fail.
fn build_perfect_hash(entries: &[(u32, u16)]) -> (Vec<u16>, Vec<u16>) {
    let mut buckets: Vec<Vec<(u32, u16)>> = vec![Vec::new(); BUCKET_COUNT];
    for &(key, score) in entries {
        buckets[bucket_of(key)].push((key, score));
    }

    let mut order: Vec<usize> = (0..BUCKET_COUNT).collect();
    order.sort_by_key(|&bucket| std::cmp::Reverse(buckets[bucket].len()));

    let mut displacements = vec![0u16; BUCKET_COUNT];
    let mut values = vec![NOTHING; SLOT_COUNT];
    let mut taken = vec![false; SLOT_COUNT];
    let mut slots = Vec::new();

    for bucket in order {
        if buckets[bucket].is_empty() {
            continue;
        }

        let displacement = (0..=u16::MAX)
            .find(|&displacement| {
                slots.clear();
                buckets[bucket].iter().all(|&(key, _)| {
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

        for &(key, score) in &buckets[bucket] {
            let slot = slot_of(key, displacement);
            taken[slot] = true;
            values[slot] = score;
        }
        displacements[bucket] = displacement;
    }

    let placed = values.iter().filter(|&&v| v != NOTHING).count();
    let distinct: std::collections::HashSet<u16> = entries.iter().map(|(_, score)| *score).collect();
    println!(
        "cargo:warning=perfect hash: {} keys into {} slots ({}% full), \
         largest displacement {}, {} distinct scores",
        placed,
        SLOT_COUNT,
        placed * 100 / SLOT_COUNT,
        displacements.iter().max().copied().unwrap_or(0),
        distinct.len(),
    );

    (displacements, values)
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
