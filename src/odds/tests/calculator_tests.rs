use crate::{
    cards::{Card, Hand},
    error::{GameError, PokerError},
    odds::EquityCalculator,
};

#[test]
fn test_equity_pat_vs_pat() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(1000);

    // Two players both pat with 75432
    let alice_hand = Hand::from_str("7d5h4c3s2h")?;
    let bob_hand = Hand::from_str("7c5s4h3d2c")?;

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?; // Empty vec means no discards
    calculator.add_player("Bob".to_string(), bob_hand, vec![])?;

    let results = calculator.calculate()?;

    assert!((results["Alice"] - 50.0).abs() < 0.1); // Allow small floating point difference
    assert!((results["Bob"] - 50.0).abs() < 0.1);

    Ok(())
}

#[test]
fn test_equity_pat_vs_drawing_to_seven() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(10000);

    let alice_hand = Hand::from_str("7d5h4c3s2h")?;
    let bob_hand = Hand::from_str("Ad5s4h3d2c")?;
    let bob_discard = Card::from_str("Ad")?;

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?;
    calculator.add_player("Bob".to_string(), bob_hand, bob_discard)?;

    let results = calculator.calculate()?;

    println!("Alice: {}", results["Alice"]);
    println!("Bob: {}", results["Bob"]);

    assert!((results["Alice"] - 96.43).abs() < 1.0);
    assert!((results["Bob"] - 3.57).abs() < 1.0);

    Ok(())
}

#[test]
fn test_dead_cards() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(10000);

    // Alice pat with 75432
    let alice_hand = Hand::from_str("7d5h4c3s2h")?;
    // Bob with A5432, discarding A to draw to a 7
    let bob_hand = Hand::from_str("Ad5s4h3d2c")?;
    let discard = Card::from_str("Ad")?;

    // Add two of the remaining 7s as dead cards
    let dead_cards = Card::from_str("7h7s")?;

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?;
    calculator.add_player("Bob".to_string(), bob_hand, discard)?;
    calculator.add_dead_cards(dead_cards)?;

    let results = calculator.calculate()?;

    // Now Bob only has 1/40 chance of hitting the last 7
    // 1/40 * 0.5 = 1.25% equity
    assert!((results["Alice"] - 98.75).abs() < 1.0);
    assert!((results["Bob"] - 1.25).abs() < 1.0);

    Ok(())
}

#[test]
fn test_three_way_tie() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(10000);

    // Three players all pat with 75432
    let alice_hand = Hand::from_str("7d5h4c3s2h")?;
    let bob_hand = Hand::from_str("7c5s4h3d2c")?;
    let charlie_hand = Hand::from_str("7s5d4d3h2s")?;

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?;
    calculator.add_player("Bob".to_string(), bob_hand, vec![])?;
    calculator.add_player("Charlie".to_string(), charlie_hand, vec![])?;

    let results = calculator.calculate()?;

    // Each player should get exactly 33.33%
    assert!((results["Alice"] - 33.33).abs() < 1.0);
    assert!((results["Bob"] - 33.33).abs() < 1.0);
    assert!((results["Charlie"] - 33.33).abs() < 1.0);

    Ok(())
}

#[test]
fn test_drawing_multiple_cards() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(10000);

    // Alice pat with 75432
    let alice_hand = Hand::from_str("7d5h4c3s2h")?;
    // Bob drawing three to 42
    let bob_hand = Hand::from_str("Ad Kd 4h 2c")?;
    let bob_discard = Card::from_str("AdKd")?;
    // First draw 9/43 outs. Second draw 6/42 outs Third draw 3/41 outs
    // When all those hit there's 50% equity
    // Gives roughly 0.1% equity

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?;
    calculator.add_player("Bob".to_string(), bob_hand, bob_discard)?;

    let results = calculator.calculate()?;

    // Bob needs to hit very specific cards to tie/win
    // Could calculate exact equity but it's quite small
    assert!(results["Alice"] > 99.0);
    assert!(results["Bob"] < 1.0);

    Ok(())
}

#[test]
fn test_slightly_better_pat() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(1000);

    // Alice pat with 75432
    let alice_hand = Hand::from_str("7d5h4c3s2h")?;
    // Bob pat with 76432 (slightly worse)
    let bob_hand = Hand::from_str("7c6s4h3d2c")?;

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?;
    calculator.add_player("Bob".to_string(), bob_hand, vec![])?;

    let results = calculator.calculate()?;

    // Alice should win 100%
    assert!((results["Alice"] - 100.0).abs() < 0.1);
    assert!((results["Bob"] - 0.0).abs() < 0.1);

    Ok(())
}

#[test]
fn test_duplicate_dead_card() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(1000);

    let hand = Hand::from_str("7d5h4c3s2h")?;
    calculator.add_player("Alice".to_string(), hand, vec![])?;

    // Try to add a dead card that's in Alice's hand
    let result = calculator
        .add_dead_cards(Card::from_str("7d").unwrap())
        .unwrap_err();
    assert!(matches!(
        result,
        PokerError::Game(GameError::DuplicateCard(_))
    ));

    Ok(())
}

#[test]
fn test_deck_removal() -> Result<(), PokerError> {
    let mut calculator = EquityCalculator::new(1000);

    // Alice has a pat 75432
    let alice_hand = Hand::from_str("7d5h4c3s2h")?;

    // Bob has 42 (will draw 3)
    let bob_hand = Hand::from_str("4h2c")?;

    // Two exposed cards (like in stud)
    let dead_cards = Card::from_str("KsQd")?;

    calculator.add_player("Alice".to_string(), alice_hand, vec![])?;
    calculator.add_player("Bob".to_string(), bob_hand, vec![])?;
    calculator.add_dead_cards(dead_cards)?;

    // At this point the deck should have:
    // 52 - 5 (Alice's cards) - 2 (Bob's cards) - 2 (dead cards) = 43 cards

    let results = calculator.calculate()?;

    // Bob needs specific cards to beat 75432
    assert!(results["Alice"] > 95.0);
    assert!(results["Bob"] < 5.0);

    Ok(())
}
