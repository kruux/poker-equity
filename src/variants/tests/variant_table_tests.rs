use std::collections::HashSet;

use crate::variants::*;

/// Every game the library claims to support, checked against what it
/// actually implements: how many cards it deals a player, how many to the
/// board, and how big its deck is.
///
/// The keys are one word per idea, family first -- `omaha_five_hi_lo`, not
/// `5_omahahi` -- so that they sort into families and can all be identifiers.
#[test]
fn test_every_variant_row_exists() {
    // key, label, hole cards, board cards, deck size
    let rows: Vec<(&str, usize, usize, u32)> = vec![
        (
            Holdem.key(),
            Holdem.hole_cards(),
            Holdem.board_cards(),
            Holdem.deck().len(),
        ),
        (
            ShortDeck.key(),
            ShortDeck.hole_cards(),
            ShortDeck.board_cards(),
            ShortDeck.deck().len(),
        ),
        (
            Omaha.key(),
            Omaha.hole_cards(),
            Omaha.board_cards(),
            Omaha.deck().len(),
        ),
        (
            OmahaFive.key(),
            OmahaFive.hole_cards(),
            OmahaFive.board_cards(),
            OmahaFive.deck().len(),
        ),
        (
            OmahaSix.key(),
            OmahaSix.hole_cards(),
            OmahaSix.board_cards(),
            OmahaSix.deck().len(),
        ),
        (
            OmahaHiLo.key(),
            OmahaHiLo.hole_cards(),
            OmahaHiLo.board_cards(),
            OmahaHiLo.deck().len(),
        ),
        (
            OmahaFiveHiLo.key(),
            OmahaFiveHiLo.hole_cards(),
            OmahaFiveHiLo.board_cards(),
            OmahaFiveHiLo.deck().len(),
        ),
        (
            Courchevel.key(),
            Courchevel.hole_cards(),
            Courchevel.board_cards(),
            Courchevel.deck().len(),
        ),
        (
            CourchevelHiLo.key(),
            CourchevelHiLo.hole_cards(),
            CourchevelHiLo.board_cards(),
            CourchevelHiLo.deck().len(),
        ),
        (
            Stud.key(),
            Stud.hole_cards(),
            Stud.board_cards(),
            Stud.deck().len(),
        ),
        (
            StudHiLo.key(),
            StudHiLo.hole_cards(),
            StudHiLo.board_cards(),
            StudHiLo.deck().len(),
        ),
        (
            Razz.key(),
            Razz.hole_cards(),
            Razz.board_cards(),
            Razz.deck().len(),
        ),
        (
            DeuceSeven.key(),
            DeuceSeven.hole_cards(),
            DeuceSeven.board_cards(),
            DeuceSeven.deck().len(),
        ),
        (
            Badugi.key(),
            Badugi.hole_cards(),
            Badugi.board_cards(),
            Badugi.deck().len(),
        ),
    ];

    let expected: Vec<(&str, usize, usize, u32)> = vec![
        ("holdem", 2, 5, 52),
        ("short_deck", 2, 5, 36),
        ("omaha", 4, 5, 52),
        ("omaha_five", 5, 5, 52),
        ("omaha_six", 6, 5, 52),
        ("omaha_hi_lo", 4, 5, 52),
        ("omaha_five_hi_lo", 5, 5, 52),
        ("courchevel", 5, 5, 52),
        ("courchevel_hi_lo", 5, 5, 52),
        ("stud", 7, 0, 52),
        ("stud_hi_lo", 7, 0, 52),
        ("razz", 7, 0, 52),
        ("deuce_seven", 5, 0, 52),
        ("badugi", 4, 0, 52),
    ];

    assert_eq!(rows.len(), 14, "the plan lists fourteen games");
    assert_eq!(rows, expected);

    let keys: HashSet<&str> = rows.iter().map(|row| row.0).collect();
    assert_eq!(keys.len(), 14, "every key is its own game");

    for key in &keys {
        assert!(
            key.chars()
                .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()),
            "{} is not lowercase and underscores",
            key
        );
        assert!(
            !key.starts_with(|c: char| c.is_ascii_digit()),
            "{} cannot be an identifier",
            key
        );
    }
}

/// The name every game goes by, at all three layers at once.
///
/// A game is named three times over -- as a Rust type, as the key it is
/// stored and configured under, and as the label a person reads -- and the
/// three drifted apart once already: a `SevenCardStud` type answering to
/// `stud`, and a `ShortDeck` type answering to `holdem_short_deck`. Writing
/// them out together is what makes the next drift deliberate.
#[test]
fn test_the_three_names_of_every_game_line_up() {
    let table: Vec<(&str, &str, &str)> = vec![
        // Rust type          key                 label
        ("Holdem", Holdem.key(), Holdem.to_string().leak()),
        ("ShortDeck", ShortDeck.key(), ShortDeck.to_string().leak()),
        ("Omaha", Omaha.key(), Omaha.to_string().leak()),
        ("OmahaFive", OmahaFive.key(), OmahaFive.to_string().leak()),
        ("OmahaSix", OmahaSix.key(), OmahaSix.to_string().leak()),
        ("OmahaHiLo", OmahaHiLo.key(), OmahaHiLo.to_string().leak()),
        (
            "OmahaFiveHiLo",
            OmahaFiveHiLo.key(),
            OmahaFiveHiLo.to_string().leak(),
        ),
        (
            "Courchevel",
            Courchevel.key(),
            Courchevel.to_string().leak(),
        ),
        (
            "CourchevelHiLo",
            CourchevelHiLo.key(),
            CourchevelHiLo.to_string().leak(),
        ),
        ("Stud", Stud.key(), Stud.to_string().leak()),
        ("StudHiLo", StudHiLo.key(), StudHiLo.to_string().leak()),
        ("Razz", Razz.key(), Razz.to_string().leak()),
        (
            "DeuceSeven",
            DeuceSeven.key(),
            DeuceSeven.to_string().leak(),
        ),
        ("Badugi", Badugi.key(), Badugi.to_string().leak()),
    ];

    let expected: Vec<(&str, &str, &str)> = vec![
        ("Holdem", "holdem", "Hold'em"),
        (
            "ShortDeck",
            "short_deck",
            "Short Deck Hold'em (flush beats full house)",
        ),
        ("Omaha", "omaha", "Omaha"),
        ("OmahaFive", "omaha_five", "5-Card Omaha"),
        ("OmahaSix", "omaha_six", "6-Card Omaha"),
        ("OmahaHiLo", "omaha_hi_lo", "Omaha Hi/Lo"),
        ("OmahaFiveHiLo", "omaha_five_hi_lo", "5-Card Omaha Hi/Lo"),
        ("Courchevel", "courchevel", "Courchevel"),
        ("CourchevelHiLo", "courchevel_hi_lo", "Courchevel Hi/Lo"),
        ("Stud", "stud", "Seven-Card Stud"),
        ("StudHiLo", "stud_hi_lo", "Seven-Card Stud Hi/Lo"),
        ("Razz", "razz", "Razz"),
        ("DeuceSeven", "deuce_seven", "2-7 Lowball (single draw)"),
        ("Badugi", "badugi", "Badugi (single draw)"),
    ];

    assert_eq!(table, expected);

    // A game whose type name spells out what its key abbreviates, or the
    // other way about, is how the two drifted before. The key should be the
    // type name in snake case, allowing for the family words that have no
    // separator in the type.
    for (type_name, key, _) in &table {
        let flattened = key.replace('_', "").to_lowercase();
        assert_eq!(
            type_name.to_lowercase(),
            flattened,
            "{} answers to {:?}, which is not the same name",
            type_name,
            key
        );
    }
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
        Stud.to_string(),
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
        ShortDeck.to_string().contains("flush beats full house"),
        "the ruleset has to be named: {}",
        ShortDeck.to_string()
    );

    // Both draw games model one draw, not three. Equity in triple draw is
    // undefined without a drawing strategy, so the label says which it is.
    assert!(DeuceSeven.to_string().contains("single draw"));
    assert!(Badugi.to_string().contains("single draw"));
}
