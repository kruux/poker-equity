//! Cross-checks the two high-hand evaluators against each other.
//!
//! `HighHandRank` names a hand (`Straight(Nine)`); `FastHandRank` scores it
//! against a table generated in `build.rs`. They are written independently, so
//! wherever they disagree about which of two hands is better, exactly one of
//! them is wrong. These tests find that disagreement without comparing every
//! hand against every other, by checking that the two are related by a single
//! order-reversing bijection:
//!
//!   - every `HighHandRank` gets exactly one score,
//!   - every score names exactly one `HighHandRank`,
//!   - and sorting hands best-first sorts the scores lowest-first.
//!
//! Those three together are equivalent to the two evaluators agreeing on every
//! pairwise comparison.

use std::collections::HashMap;

use poker_equity::cards::{Card, CardSet, Rank, Suit};
use poker_equity::variants::{
    deuce_seven_score, high_score, low_a5_score, short_deck_score, DeuceSevenRank, HighHandRank,
    LowHandRank, PokerVariant, ShortDeck,
};

fn deck() -> Vec<Card> {
    let mut cards = Vec::with_capacity(52);
    for suit in Suit::all() {
        for rank in Rank::all() {
            cards.push(Card::new(suit, rank));
        }
    }
    cards
}

/// Compares a kernel's table against the evaluator that names its hands,
/// returning whatever they disagree about.
///
/// `name` is the slow, readable evaluator and `score` is the table. Both are
/// written independently, so wherever they disagree about which of two hands
/// is better, exactly one of them is wrong.
///
/// Every one of these rankings orders hands greatest-is-best, while every
/// table scores them lowest-is-best, so what is checked is that the two are
/// related by a single order-reversing bijection.
fn disagreements<R, N, S>(hands: impl Iterator<Item = Vec<Card>>, name: N, score: S) -> Vec<String>
where
    R: std::fmt::Debug + PartialOrd,
    N: Fn(&[Card]) -> R,
    S: Fn(&[Card]) -> u16,
{
    // Hands are identified by their debug form, which carries every field
    // and so distinguishes hands that merely print alike.
    let mut score_of: HashMap<String, (u16, Vec<Card>)> = HashMap::new();
    let mut named_by: HashMap<u16, (String, Vec<Card>)> = HashMap::new();
    let mut ranked: Vec<(R, u16)> = Vec::new();
    let mut problems = Vec::new();

    let show = |cards: &[Card]| {
        cards
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    };

    for hand in hands {
        let rank = name(&hand);
        let label = format!("{:?}", rank);
        let found = score(&hand);

        match score_of.get(&label) {
            Some((seen, first)) if *seen != found => {
                if problems.len() < 8 {
                    problems.push(format!(
                        "one hand, two scores: {} scores {} as {} but {} as {}",
                        label,
                        show(first),
                        seen,
                        show(&hand),
                        found
                    ));
                }
            }
            Some(_) => {}
            None => {
                score_of.insert(label.clone(), (found, hand.clone()));
                ranked.push((rank, found));
            }
        }

        match named_by.get(&found) {
            Some((seen, first)) if *seen != label => {
                if problems.len() < 8 {
                    problems.push(format!(
                        "one score, two hands: score {} is {} for {} but {} for {}",
                        found,
                        seen,
                        show(first),
                        label,
                        show(&hand)
                    ));
                }
            }
            Some(_) => {}
            None => {
                named_by.insert(found, (label, hand));
            }
        }
    }

    // Best hand first; the scores that go with them must climb from zero.
    ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    for pair in ranked.windows(2) {
        let ((better, better_score), (worse, worse_score)) = (&pair[0], &pair[1]);
        if better_score >= worse_score && problems.len() < 16 {
            problems.push(format!(
                "order reversed: {:?} beats {:?}, but scores {} against {} \
                 (lower scores are better hands)",
                better, worse, better_score, worse_score
            ));
        }
    }

    problems
}

/// Fails with everything that disagreed, or passes quietly.
fn report(problems: Vec<String>, checked: usize, what: &str) {
    if !problems.is_empty() {
        panic!(
            "the table and the reference evaluator disagree on {} ({} checked):\n  {}",
            what,
            checked,
            problems.join("\n  ")
        );
    }
}

/// Every five-card hand from `deck`.
fn five_card_hands(deck: &[Card]) -> Vec<Vec<Card>> {
    let mut hands = Vec::new();
    for a in 0..deck.len() {
        for b in (a + 1)..deck.len() {
            for c in (b + 1)..deck.len() {
                for d in (c + 1)..deck.len() {
                    for e in (d + 1)..deck.len() {
                        hands.push(vec![deck[a], deck[b], deck[c], deck[d], deck[e]]);
                    }
                }
            }
        }
    }
    hands
}

/// Every five-card hand. This is the gate the plan asks for: nothing about a
/// lookup table is self-evident.
#[test]
#[ignore = "exhaustive sweep; run with --ignored"]
fn five_card_hands_exhaustive() {
    let deck = deck();
    let mut hands = Vec::new();
    for a in 0..deck.len() {
        for b in (a + 1)..deck.len() {
            for c in (b + 1)..deck.len() {
                for d in (c + 1)..deck.len() {
                    for e in (d + 1)..deck.len() {
                        hands.push(vec![deck[a], deck[b], deck[c], deck[d], deck[e]]);
                    }
                }
            }
        }
    }
    assert_eq!(hands.len(), 2_598_960, "C(52,5)");
    let checked = hands.len();
    report(
        disagreements(hands.into_iter(), HighHandRank::evaluate, high_score),
        checked,
        "five-card hands",
    );
}

/// Every seven-card hand from a reduced deck of the ace and the deuce through
/// the six, in all four suits. Exhaustive over that subspace, and the subspace
/// is chosen to hold the shapes a five-card sweep cannot reach: wheels, low
/// straights, and a pair of aces sitting above a straight.
#[test]
fn seven_card_hands_from_a_reduced_deck() {
    let keep = [
        Rank::Ace,
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
    ];
    let deck: Vec<Card> = deck()
        .into_iter()
        .filter(|c| keep.contains(&c.rank()))
        .collect();
    assert_eq!(deck.len(), 24, "six ranks in four suits");

    let mut hands = Vec::new();
    for a in 0..deck.len() {
        for b in (a + 1)..deck.len() {
            for c in (b + 1)..deck.len() {
                for d in (c + 1)..deck.len() {
                    for e in (d + 1)..deck.len() {
                        for f in (e + 1)..deck.len() {
                            for g in (f + 1)..deck.len() {
                                hands.push(vec![
                                    deck[a], deck[b], deck[c], deck[d], deck[e], deck[f], deck[g],
                                ]);
                            }
                        }
                    }
                }
            }
        }
    }
    let checked = hands.len();
    report(
        disagreements(hands.into_iter(), HighHandRank::evaluate, high_score),
        checked,
        "seven-card hands from a reduced deck",
    );
}

/// Seven-card hands drawn from the whole deck, for the breadth the reduced
/// deck gives up. The generator is a fixed-seed xorshift rather than `rand`,
/// so a failure here reproduces exactly.
#[test]
fn seven_card_hands_sampled() {
    let deck = deck();
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    let mut hands = Vec::with_capacity(200_000);
    for _ in 0..200_000 {
        let mut chosen = [0usize; 7];
        let mut taken = 0;
        while taken < 7 {
            let pick = (next() % 52) as usize;
            if !chosen[..taken].contains(&pick) {
                chosen[taken] = pick;
                taken += 1;
            }
        }
        hands.push(chosen.iter().map(|&i| deck[i]).collect::<Vec<Card>>());
    }
    let checked = hands.len();
    report(
        disagreements(hands.into_iter(), HighHandRank::evaluate, high_score),
        checked,
        "sampled seven-card hands",
    );
}

/// The specific shape that a five-card sweep cannot reach: a pair sitting
/// above a straight. Kept as its own test so a failure names the bug rather
/// than burying it in a sweep.
#[test]
fn a_pair_above_a_straight_does_not_raise_it() {
    let nine_high = Card::parse_field("Ah As 9d 8c 7s 6h 5d").unwrap();
    let ten_high = Card::parse_field("Td 9c 8s 7h 6d 2c 3h").unwrap();

    assert_eq!(
        HighHandRank::evaluate(&nine_high),
        HighHandRank::Straight(Rank::Nine),
        "aces alongside 9-8-7-6-5 is a nine-high straight, not an ace-high one"
    );
    assert!(
        HighHandRank::evaluate(&ten_high) > HighHandRank::evaluate(&nine_high),
        "a ten-high straight beats a nine-high straight"
    );
}

/// Every five-card hand, checked against the deuce-to-seven table.
///
/// This ranking is the high hand upside down, with the ace forced high, so
/// `A5432` has to come out a bad high-card hand rather than a straight and
/// `A5432` suited a flush rather than a straight flush.
#[test]
#[ignore = "exhaustive sweep; run with --ignored"]
fn deuce_seven_hands_exhaustive() {
    let hands = five_card_hands(&deck());
    let checked = hands.len();
    report(
        disagreements(
            hands.into_iter(),
            DeuceSevenRank::evaluate,
            deuce_seven_score,
        ),
        checked,
        "five-card deuce-to-seven hands",
    );
}

/// Every five-card hand from the thirty-six card deck, checked against the
/// short-deck table. The ace plays low below the six here, and a flush beats
/// a full house.
#[test]
#[ignore = "exhaustive sweep; run with --ignored"]
fn short_deck_hands_exhaustive() {
    let short: Vec<Card> = CardSet::SHORT_DECK.iter().collect();
    assert_eq!(short.len(), 36);
    let hands = five_card_hands(&short);
    let checked = hands.len();
    report(
        disagreements(
            hands.into_iter(),
            |cards| ShortDeck.evaluate_hand(cards),
            short_deck_score,
        ),
        checked,
        "five-card short-deck hands",
    );
}

/// Every five-card hand, checked against the ace-to-five low table.
///
/// Suits never matter to this ranking, so the table is keyed on ranks alone;
/// the sweep still walks real cards, which is what would catch it if suits
/// leaked in.
#[test]
#[ignore = "exhaustive sweep; run with --ignored"]
fn low_hands_exhaustive() {
    let hands = five_card_hands(&deck());
    let checked = hands.len();
    report(
        disagreements(hands.into_iter(), LowHandRank::evaluate, low_a5_score),
        checked,
        "five-card ace-to-five lows",
    );
}

/// The same three kernels over seven-card hands, where a hand can hold more
/// than it plays. Sampled rather than exhaustive, from a fixed seed.
#[test]
fn every_kernel_agrees_on_seven_card_hands() {
    let deck = deck();
    let mut state: u64 = 0xA5A5_1234_DEAD_BEEF;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };

    let mut hands = Vec::with_capacity(60_000);
    for _ in 0..60_000 {
        let mut chosen = [0usize; 7];
        let mut taken = 0;
        while taken < 7 {
            let pick = (next() % 52) as usize;
            if !chosen[..taken].contains(&pick) {
                chosen[taken] = pick;
                taken += 1;
            }
        }
        hands.push(chosen.iter().map(|&i| deck[i]).collect::<Vec<Card>>());
    }

    let checked = hands.len();
    report(
        disagreements(hands.iter().cloned(), HighHandRank::evaluate, high_score),
        checked,
        "sampled seven-card high hands",
    );
    report(
        disagreements(hands.iter().cloned(), LowHandRank::evaluate, low_a5_score),
        checked,
        "sampled seven-card lows",
    );
    report(
        disagreements(
            hands.into_iter(),
            |cards| ShortDeck.evaluate_hand(cards),
            short_deck_score,
        ),
        checked,
        "sampled seven-card short-deck hands",
    );
}
