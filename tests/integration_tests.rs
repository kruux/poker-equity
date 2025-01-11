// In tests/integration_tests.rs
use poker_calculator::{
    cards::Card, error::PokerError, hand::Hand, odds::EquityCalculator, variants::DeuceSeven,
};

#[test]
fn test_pat_vs_drawing_scenario() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(DeuceSeven, 10000);

    // The nuts pat
    let alice_hand = Hand::from_str(DeuceSeven, "7d5h4c3s2h")?;

    // Drawing one to beat it
    let bob_hand = Hand::from_str(DeuceSeven, "Ad5s4h3d2c")?;
    let bob_discard = Card::from_str("Ad")?;

    calculator.add_draw_player("Alice".to_string(), alice_hand, None)?;
    calculator.add_draw_player("Bob".to_string(), bob_hand, Some(bob_discard))?;

    let results = calculator.calculate(drop)?;

    // Known percentages for this scenario
    assert!((results["Alice"] - 96.43).abs() < 1.0);
    assert!((results["Bob"] - 3.57).abs() < 1.0);

    Ok(())
}

#[test]
fn test_multiway_pat_and_drawing() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(DeuceSeven, 10000);

    // Three players:
    // Alice: pat with 75432
    // Bob: drawing one to 75432
    // Charlie: drawing two to 432

    let alice_hand = Hand::from_str(DeuceSeven, "7d5h4c3s2h")?;

    let bob_hand = Hand::from_str(DeuceSeven, "Ad5s4h3d2c")?;
    let bob_discard = Card::from_str("Ad")?;

    let charlie_hand = Hand::from_str(DeuceSeven, "4d3h2d")?; // Drawing 2 cards

    calculator.add_draw_player("Alice".to_string(), alice_hand, None)?;
    calculator.add_draw_player("Bob".to_string(), bob_hand, Some(bob_discard))?;
    calculator.add_draw_player("Charlie".to_string(), charlie_hand, None)?;

    let results = calculator.calculate(drop)?;

    // Alice should have best equity
    assert!(results["Alice"] > results["Bob"]);
    assert!(results["Alice"] > results["Charlie"]);
    // Bob should have better equity than Charlie
    assert!(results["Bob"] > results["Charlie"]);

    Ok(())
}

#[test]
fn test_dead_cards_impact() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(DeuceSeven, 10000);

    // Alice pat with 75432
    let alice_hand = Hand::from_str(DeuceSeven, "7d5h4c3s2h")?;

    // Bob drawing one, but key cards are dead
    let bob_hand = Hand::from_str(DeuceSeven, "Ad5s4h3d2c")?;
    let bob_discard = Card::from_str("Ad")?;

    // Add two of the three remaining 7s as dead cards
    let dead_cards = Card::from_str("7h7s")?;

    calculator.add_draw_player("Alice".to_string(), alice_hand, None)?;
    calculator.add_draw_player("Bob".to_string(), bob_hand, Some(bob_discard))?;
    calculator.add_dead_cards(dead_cards)?;

    let results = calculator.calculate(drop)?;

    // Only one 7 left in deck, so Bob's equity should be very low
    assert!(results["Bob"] < 2.0);

    Ok(())
}
