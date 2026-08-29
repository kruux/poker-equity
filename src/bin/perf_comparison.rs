use std::time::Instant;

use poker_calculator::cards::{Card, Deck};
use poker_calculator::variants::{
    fast_to_high, high_to_fast, FastHandRank, HighHandRank, Holdem, HoldemFast, Omaha, OmahaFast,
    PokerVariant,
};

/// Generates a random hand by creating a deck of 52 cards, shuffling it, and drawing the first `num_cards` cards.
fn random_hand(num_cards: usize) -> Vec<Card> {
    let mut deck = Deck::new();
    let mut hand = Vec::with_capacity(num_cards);
    for _ in 0..num_cards {
        if let Some(card) = deck.deal() {
            hand.push(card);
        } else {
            break; // In case the deck runs out, though it shouldn't
        }
    }
    hand
}

fn main() {
    const NUM_HANDS: usize = 50_000;

    // --- Hold'em Experiment ---
    println!("Measuring performance on Hold'em variant:");
    let mut hands_holdem = Vec::with_capacity(NUM_HANDS);
    // Hold'em hand has 7 cards (2 hole cards + 5 community cards typically)
    for _ in 0..NUM_HANDS {
        hands_holdem.push(random_hand(7));
    }

    let holdem_fast = HoldemFast;
    let fast_start = Instant::now();
    let fast_evals: Vec<FastHandRank> = hands_holdem
        .iter()
        .map(|hand| holdem_fast.evaluate_hand(hand))
        .collect();
    let fast_duration = fast_start.elapsed();

    let holdem_high = Holdem;
    let high_start = Instant::now();
    let high_evals: Vec<HighHandRank> = hands_holdem
        .iter()
        .map(|hand| holdem_high.evaluate_hand(hand))
        .collect();
    let high_duration = high_start.elapsed();

    // Validate that the translation mappings are consistent
    for (fast, high) in fast_evals.iter().zip(high_evals.iter()) {
        let translated_from_fast = fast_to_high(*fast);
        let translated_from_high = high_to_fast(high);
        if translated_from_fast != *high || translated_from_high != *fast {
            panic!("Mismatch in translation mapping between fast and high evaluations");
        }
    }

    println!(
        "Hold'em Fast evaluation time for {} hands: {:?}",
        NUM_HANDS, fast_duration
    );
    println!(
        "Hold'em High evaluation time for {} hands: {:?}",
        NUM_HANDS, high_duration
    );

    // --- Omaha Experiment ---
    println!("\nMeasuring performance on Omaha variant:");
    let mut hands_omaha = Vec::with_capacity(NUM_HANDS);
    // Omaha hand has 9 cards (4 hole cards + 5 community cards)
    for _ in 0..NUM_HANDS {
        hands_omaha.push(random_hand(9));
    }

    let omaha_fast = OmahaFast;
    let fast_start_omaha = Instant::now();
    let fast_evals_omaha: Vec<FastHandRank> = hands_omaha
        .iter()
        .map(|hand| omaha_fast.evaluate_hand(hand))
        .collect();
    let omaha_fast_duration = fast_start_omaha.elapsed();

    let omaha_high = Omaha;
    let high_start_omaha = Instant::now();
    let high_evals_omaha: Vec<HighHandRank> = hands_omaha
        .iter()
        .map(|hand| omaha_high.evaluate_hand(hand))
        .collect();
    let omaha_high_duration = high_start_omaha.elapsed();

    for (fast, high) in fast_evals_omaha.iter().zip(high_evals_omaha.iter()) {
        let translated_from_fast = fast_to_high(*fast);
        let translated_from_high = high_to_fast(high);
        if translated_from_fast != *high || translated_from_high != *fast {
            panic!("Mismatch in translation mapping for Omaha evaluations");
        }
    }

    println!(
        "Omaha Fast evaluation time for {} hands: {:?}",
        NUM_HANDS, omaha_fast_duration
    );
    println!(
        "Omaha High evaluation time for {} hands: {:?}",
        NUM_HANDS, omaha_high_duration
    );
}
