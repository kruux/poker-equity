use std::collections::HashMap;

use rand::{rngs::StdRng, SeedableRng};

use crate::cards::{Card, CardSet, Rank, Suit};
use crate::error::PokerError;
use crate::sampler::SlotSampler;

fn set(text: &str) -> CardSet {
    CardSet::from_cards(&Card::from_str(text).unwrap())
}

/// Draws many deals and counts how often each *set* of cards came up.
fn tally(sampler: &SlotSampler, available: CardSet, draws: usize) -> HashMap<String, usize> {
    let mut rng = StdRng::seed_from_u64(0x5EED_5EED);
    let mut counts = HashMap::new();
    let mut out = Vec::new();
    let mut attempts = 0;

    while counts.values().sum::<usize>() < draws {
        attempts += 1;
        assert!(attempts < draws * 1000, "sampler is rejecting almost everything");
        if !sampler.draw(available, &mut rng, &mut out) {
            continue;
        }
        // Order within a deal carries no meaning, so key on the set.
        let mut names: Vec<String> = out.iter().map(|c| c.to_string()).collect();
        names.sort();
        *counts.entry(names.join(" ")).or_insert(0) += 1;
    }
    counts
}

/// The failure mode PLAN section 7.2 warns about, in the smallest case that
/// shows it.
///
/// Slot masks `{2c,2d,2h}` and `{2c,2d}` admit three sets: `2c 2d`, `2c 2h`
/// and `2d 2h`. Sampling one card per slot and rejecting collisions is
/// uniform over *ordered* assignments, of which there are four -- (2c,2d),
/// (2d,2c), (2h,2c), (2h,2d) -- so it hands `2c 2d` half the weight instead
/// of a third. The equities that come out are self-consistent, stable, and
/// wrong.
#[test]
fn test_overlapping_slots_are_uniform_over_sets() {
    let slots = [set("2c 2d 2h"), set("2c 2d")];
    let draws = 120_000;

    for sampler in [
        SlotSampler::new(&slots, CardSet::FULL_DECK),
        SlotSampler::forcing_draw_and_test(&slots, CardSet::FULL_DECK),
    ] {
        let counts = tally(&sampler, CardSet::FULL_DECK, draws);
        assert_eq!(counts.len(), 3, "three sets are possible: {:?}", counts);

        for (cards, &count) in &counts {
            let share = count as f64 / draws as f64;
            assert!(
                (share - 1.0 / 3.0).abs() < 0.01,
                "{} strategy gave {} a share of {:.4}, expected 0.3333 -- \
                 an ordered sampler gives the overlapping pair 0.5",
                sampler.strategy(),
                cards,
                share
            );
        }
    }
}

/// A wider overlap, where the ordered sampler's error is less obvious but
/// still there.
#[test]
fn test_uniformity_holds_with_a_wider_overlap() {
    // Four cards, one slot taking any of them and one taking two.
    let slots = [set("2c 2d 2h 2s"), set("2c 2d")];
    let draws = 150_000;
    let sampler = SlotSampler::new(&slots, CardSet::FULL_DECK);
    let counts = tally(&sampler, CardSet::FULL_DECK, draws);

    // Sets: {2c,2d}, {2c,2h}, {2c,2s}, {2d,2h}, {2d,2s}. Not {2h,2s}: the
    // second slot cannot take either.
    assert_eq!(counts.len(), 5, "{:?}", counts);
    assert!(!counts.contains_key("2h 2s"), "neither card fits the second slot");
    for (cards, &count) in &counts {
        let share = count as f64 / draws as f64;
        assert!(
            (share - 0.2).abs() < 0.01,
            "{} got {:.4}, expected 0.2000",
            cards,
            share
        );
    }
}

/// Where nothing is constrained the sampler takes the free path, which is the
/// common case: boards, stud deals, and most hands.
#[test]
fn test_unconstrained_slots_take_the_free_path() {
    let slots = vec![CardSet::FULL_DECK; 5];
    let sampler = SlotSampler::new(&slots, CardSet::FULL_DECK);
    assert_eq!(sampler.strategy(), "free");

    let mut rng = StdRng::seed_from_u64(1);
    let mut out = Vec::new();
    for _ in 0..1000 {
        assert!(sampler.draw(CardSet::FULL_DECK, &mut rng, &mut out));
        assert_eq!(out.len(), 5);
        let unique: CardSet = out.iter().copied().collect();
        assert_eq!(unique.len(), 5, "a deal never repeats a card");
    }
}

/// A wildcard admitting exactly one card must behave as naming that card.
#[test]
fn test_a_one_card_slot_always_yields_that_card() -> Result<(), PokerError> {
    let ace_of_spades = Card::new(Suit::Spade, Rank::Ace);
    let slots = [set("As"), CardSet::FULL_DECK];
    let sampler = SlotSampler::new(&slots, CardSet::FULL_DECK);

    let mut rng = StdRng::seed_from_u64(7);
    let mut out = Vec::new();
    for _ in 0..500 {
        if sampler.draw(CardSet::FULL_DECK, &mut rng, &mut out) {
            assert!(
                out.contains(&ace_of_spades),
                "the named card must be in every deal: {:?}",
                out
            );
        }
    }
    Ok(())
}

/// Cards taken earlier in the same deal are off the table.
#[test]
fn test_cards_already_dealt_are_not_drawn_again() {
    let slots = [CardSet::of_rank(Rank::Ace), CardSet::of_rank(Rank::King)];
    let sampler = SlotSampler::new(&slots, CardSet::FULL_DECK);

    // Three aces and three kings gone: only one of each remains.
    let taken = set("Ah Ad Ac Kh Kd Kc");
    let available = CardSet::FULL_DECK.without(taken);

    let mut rng = StdRng::seed_from_u64(11);
    let mut out = Vec::new();
    let mut successes = 0;
    for _ in 0..2000 {
        if sampler.draw(available, &mut rng, &mut out) {
            successes += 1;
            let drawn: CardSet = out.iter().copied().collect();
            assert!(drawn.is_disjoint(taken), "drew a card already gone: {:?}", out);
            assert_eq!(drawn, set("As Ks"), "only one ace and one king are left");
        }
    }
    assert!(successes > 0, "the only remaining pair should still be drawable");
}

/// A group of slots no deal can fill never yields one.
#[test]
fn test_impossible_slots_never_draw() {
    // Both slots want the ace of hearts, and there is one of those.
    let slots = [set("Ah"), set("Ah")];
    let sampler = SlotSampler::new(&slots, CardSet::FULL_DECK);

    let mut rng = StdRng::seed_from_u64(3);
    let mut out = Vec::new();
    for _ in 0..1000 {
        assert!(
            !sampler.draw(CardSet::FULL_DECK, &mut rng, &mut out),
            "one card cannot fill two slots"
        );
    }
}

/// Constrained slots over a small space are listed rather than drawn for,
/// since drawing would reject nearly everything.
#[test]
fn test_small_constrained_spaces_are_listed() {
    let clubs_and_aces = [CardSet::of_suit(Suit::Club), CardSet::of_rank(Rank::Ace)];
    let sampler = SlotSampler::new(&clubs_and_aces, CardSet::FULL_DECK);
    assert_eq!(sampler.strategy(), "listed");

    // A wildcard opens the space back up to the whole deck.
    let wide = [CardSet::of_rank(Rank::Ace); 7];
    assert_eq!(SlotSampler::new(&wide, CardSet::FULL_DECK).strategy(), "listed");
}

/// Filling the tightest slot first is a tempting way to sample, and it is
/// wrong. This is the measurement that says so.
///
/// The sampler does fix *forced* slots first -- a slot with exactly one
/// candidate takes that card in every valid deal, so setting it aside removes
/// no possibility. The tempting next step is to carry on that way and fill
/// the partly constrained slots before the wildcards. That is a different
/// thing entirely: a slot with a choice changes how many ways the rest can go
/// once it has chosen.
///
/// With `A *` -- any ace, and any card -- filling the ace slot first picks
/// evenly among four aces and then a second card from the fifty-one left. A
/// hand holding two aces is reachable twice over, because either ace could
/// have been the one chosen first, so it collects double the weight it should.
#[test]
fn test_filling_the_tightest_slot_first_would_bias_the_draw() {
    let slots = [CardSet::of_rank(Rank::Ace), CardSet::FULL_DECK];

    let every_set = SlotSampler::new(&slots, CardSet::FULL_DECK)
        .all_sets(CardSet::FULL_DECK, 1_000_000)
        .expect("small enough to list");
    let two_aces = every_set
        .iter()
        .filter(|set| set.iter().filter(|card| card.rank() == Rank::Ace).count() == 2)
        .count();
    let truth = two_aces as f64 / every_set.len() as f64;
    assert_eq!(every_set.len(), 198);
    assert_eq!(two_aces, 6, "six ways to hold two of the four aces");

    // What the sampler does: draw a subset, keep it if the slots can be
    // filled from it.
    let sampler = SlotSampler::new(&slots, CardSet::FULL_DECK);
    let mut rng = StdRng::seed_from_u64(11);
    let mut out = Vec::new();
    let (mut drawn, mut paired) = (0u32, 0u32);
    for _ in 0..400_000 {
        if sampler.draw(CardSet::FULL_DECK, &mut rng, &mut out) {
            drawn += 1;
            if out.iter().filter(|card| card.rank() == Rank::Ace).count() == 2 {
                paired += 1;
            }
        }
    }
    let measured = paired as f64 / drawn as f64;

    assert!(
        (measured - truth).abs() < 0.002,
        "the sampler should hold two aces {:.4}% of the time, and holds them {:.4}%",
        truth * 100.0,
        measured * 100.0
    );

    // And the ordering that would have been wrong: fill the ace slot first,
    // then take any second card. Worked out exactly rather than sampled.
    // Each ordered pair has probability 1/4 * 1/51; a two-ace hand is two
    // such pairs, a one-ace hand only one.
    let ordered_pairs = 4.0 * 51.0;
    let biased = (two_aces as f64 * 2.0) / ordered_pairs;
    assert!(
        biased > truth * 1.9,
        "filling the ace slot first should nearly double the rate of two-ace \
         hands: {:.4}% against the true {:.4}%",
        biased * 100.0,
        truth * 100.0
    );
}
