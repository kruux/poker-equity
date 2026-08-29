use std::collections::HashSet;

use crate::variants::*;

/// Every row of the table in PLAN section 4, checked against what the library
/// actually implements.
///
/// The key is fpdb's own database category, so a mismatch here is a game the
/// tracker asks for and does not get.
#[test]
fn test_every_variant_row_exists() {
    // key, label, hole cards, board cards, deck size
    let rows: Vec<(&str, usize, usize, u32)> = vec![
        (Holdem.key(), Holdem.hole_cards(), Holdem.board_cards(), Holdem.deck().len()),
        (ShortDeck.key(), ShortDeck.hole_cards(), ShortDeck.board_cards(), ShortDeck.deck().len()),
        (Omaha.key(), Omaha.hole_cards(), Omaha.board_cards(), Omaha.deck().len()),
        (OmahaFive.key(), OmahaFive.hole_cards(), OmahaFive.board_cards(), OmahaFive.deck().len()),
        (OmahaSix.key(), OmahaSix.hole_cards(), OmahaSix.board_cards(), OmahaSix.deck().len()),
        (OmahaHiLo.key(), OmahaHiLo.hole_cards(), OmahaHiLo.board_cards(), OmahaHiLo.deck().len()),
        (OmahaFiveHiLo.key(), OmahaFiveHiLo.hole_cards(), OmahaFiveHiLo.board_cards(), OmahaFiveHiLo.deck().len()),
        (Courchevel.key(), Courchevel.hole_cards(), Courchevel.board_cards(), Courchevel.deck().len()),
        (CourchevelHiLo.key(), CourchevelHiLo.hole_cards(), CourchevelHiLo.board_cards(), CourchevelHiLo.deck().len()),
        (SevenCardStud.key(), SevenCardStud.hole_cards(), SevenCardStud.board_cards(), SevenCardStud.deck().len()),
        (StudHiLo.key(), StudHiLo.hole_cards(), StudHiLo.board_cards(), StudHiLo.deck().len()),
        (Razz.key(), Razz.hole_cards(), Razz.board_cards(), Razz.deck().len()),
        (DeuceSeven.key(), DeuceSeven.hole_cards(), DeuceSeven.board_cards(), DeuceSeven.deck().len()),
        (Badugi.key(), Badugi.hole_cards(), Badugi.board_cards(), Badugi.deck().len()),
    ];

    let expected: Vec<(&str, usize, usize, u32)> = vec![
        ("holdem", 2, 5, 52),
        ("6_holdem", 2, 5, 36),
        ("omahahi", 4, 5, 52),
        ("5_omahahi", 5, 5, 52),
        ("6_omahahi", 6, 5, 52),
        ("omahahilo", 4, 5, 52),
        ("5_omaha8", 5, 5, 52),
        ("cour_hi", 5, 5, 52),
        ("cour_hilo", 5, 5, 52),
        ("studhi", 7, 0, 52),
        ("studhilo", 7, 0, 52),
        ("razz", 7, 0, 52),
        ("27_3draw", 5, 0, 52),
        ("badugi", 4, 0, 52),
    ];

    assert_eq!(rows.len(), 14, "the plan lists fourteen games");
    assert_eq!(rows, expected);

    let keys: HashSet<&str> = rows.iter().map(|row| row.0).collect();
    assert_eq!(keys.len(), 14, "every key is its own game");
}

/// Every variant names itself, so a caller can label a table without a
/// lookup of its own.
#[test]
fn test_every_variant_has_a_label() {
    let labels = [
        Holdem.to_string(),
        ShortDeck.to_string(),
        Omaha.to_string(),
        OmahaFive.to_string(),
        OmahaSix.to_string(),
        OmahaHiLo.to_string(),
        OmahaFiveHiLo.to_string(),
        Courchevel.to_string(),
        CourchevelHiLo.to_string(),
        SevenCardStud.to_string(),
        StudHiLo.to_string(),
        Razz.to_string(),
        DeuceSeven.to_string(),
        Badugi.to_string(),
    ];
    for label in &labels {
        assert!(!label.is_empty());
    }
    // Short deck names its ruleset, since rooms differ on it.
    assert!(
        ShortDeck.to_string().contains("flush over full house"),
        "the ruleset has to be named: {}",
        ShortDeck.to_string()
    );
}
