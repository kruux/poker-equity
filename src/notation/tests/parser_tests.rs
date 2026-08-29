use crate::cards::{Card, CardSet, Rank, Suit};
use crate::notation::{parse_board, parse_dead, parse_hand, HandSpec, NotationErrorKind};

/// Every hand a spec admits, as sorted text, for comparing two specs that
/// were written differently.
fn combinations(spec: &HandSpec) -> Vec<String> {
    let mut out: Vec<String> = spec
        .alternatives
        .iter()
        .map(|alternative| {
            alternative
                .iter()
                .map(|slot| slot.to_string())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    out.sort();
    out
}

#[test]
fn test_range_sizes() {
    // The counts every poker player knows, which is what makes them a good
    // check on the expansion.
    let count = |text: &str| parse_hand(text, 2).unwrap().alternatives.len();
    assert_eq!(count("AKs"), 4, "four suited combinations");
    assert_eq!(count("AKo"), 12, "twelve offsuit combinations");
    assert_eq!(count("AK"), 16, "sixteen in all");
    assert_eq!(count("22"), 6, "six ways to hold a pair");
    assert_eq!(count("22+"), 13 * 6, "every pair");
    assert_eq!(count("A2s+"), 12 * 4, "A2s through AKs");
    assert_eq!(count("KTs+"), 3 * 4, "KTs, KJs and KQs only");
    assert_eq!(count("JTs-76s"), 5 * 4, "five suited connectors");
    assert_eq!(count("AKs, 22"), 4 + 6, "alternatives add up");
}

/// A `+` on a non-pair raises the lower card and leaves the higher alone, so
/// it can never run past the card above it.
#[test]
fn test_plus_stops_below_the_high_card() {
    let spec = parse_hand("KTs+", 2).unwrap();
    for alternative in &spec.alternatives {
        let high = alternative[0].iter().next().unwrap();
        let low = alternative[1].iter().next().unwrap();
        assert_eq!(high.rank(), Rank::King);
        assert!(
            low.rank().to_value() < Rank::King.to_value(),
            "{} is not below a king",
            low
        );
        assert_eq!(high.suit(), low.suit(), "suited");
    }
}

/// Either end of a `-` range may be written first, and a term's own ranks may
/// be written in either order.
#[test]
fn test_ranges_normalise_the_order_they_are_written_in() {
    assert_eq!(
        combinations(&parse_hand("JTs-76s", 2).unwrap()),
        combinations(&parse_hand("76s-JTs", 2).unwrap()),
    );
    assert_eq!(
        combinations(&parse_hand("AKs", 2).unwrap()),
        combinations(&parse_hand("KAs", 2).unwrap()),
    );
}

/// The rule that decides `2c` from `2 c`, checked as masks rather than text.
#[test]
fn test_binding_is_immediate_only() {
    let bound = parse_hand("2c", 1).unwrap();
    assert_eq!(bound.alternatives[0].len(), 1);
    assert_eq!(bound.alternatives[0][0].len(), 1, "one named card");

    let loose = parse_hand("2 c", 2).unwrap();
    assert_eq!(loose.alternatives[0][0], CardSet::of_rank(Rank::Two));
    assert_eq!(loose.alternatives[0][1], CardSet::of_suit(Suit::Club));
    assert_eq!(loose.alternatives[0][0].len(), 4);
    assert_eq!(loose.alternatives[0][1].len(), 13);
}

/// A wildcard that admits exactly one card must be the same thing as naming
/// that card. This is the property in PLAN section 9.4 that tests the
/// wildcard path against the concrete path.
#[test]
fn test_a_wildcard_admitting_one_card_equals_naming_it() {
    let ace_of_spades = Card::new(Suit::Spade, Rank::Ace);

    // "A" narrowed to the spades is exactly "As".
    let any_ace = CardSet::of_rank(Rank::Ace);
    let spades = CardSet::of_suit(Suit::Spade);
    let narrowed = any_ace.intersection(spades);
    assert_eq!(narrowed, CardSet::from_cards(&[ace_of_spades]));

    let named = parse_hand("As", 1).unwrap();
    assert_eq!(named, HandSpec::from_slots(&[narrowed]));
}

/// Errors carry the span of the text at fault, so a caller can underline it.
#[test]
fn test_errors_point_at_the_offending_text() {
    let error = parse_hand("AhXh", 2).unwrap_err();
    assert_eq!(error.kind, NotationErrorKind::UnknownGlyph('X'));
    assert_eq!(error.offset, 2, "the X, not the field");
    assert_eq!(error.len, 1);

    // The second naming is the one reported, since the first was legitimate.
    let error = parse_hand("AhKh Ah", 3).unwrap_err();
    assert_eq!(
        error.kind,
        NotationErrorKind::DuplicateCard("Ah".to_string())
    );
    assert_eq!(error.offset, 5);
    assert_eq!(error.len, 2);

    // A span in a later alternative is still measured from the start of the
    // whole field.
    let error = parse_hand("AhKh, Qs?d", 2).unwrap_err();
    assert_eq!(error.kind, NotationErrorKind::UnknownGlyph('?'));
    assert_eq!(error.offset, 8);

    let drawn = error.underline("AhKh, Qs?d");
    assert!(drawn.contains("        ^"), "underline sits under the ?: {}", drawn);
}

/// A miscount around a `*` says why, rather than only reporting the number.
#[test]
fn test_a_wildcard_miscount_explains_itself() {
    let error = parse_hand("A*", 1).unwrap_err();
    assert_eq!(
        error.kind,
        NotationErrorKind::WildcardSlotCount {
            expected: 1,
            found: 2
        }
    );
    assert!(
        error.to_string().contains("never binds"),
        "got: {}",
        error
    );

    // Without a wildcard it is a plain miscount.
    let error = parse_hand("AhKhQh", 2).unwrap_err();
    assert_eq!(
        error.kind,
        NotationErrorKind::WrongSlotCount {
            expected: 2,
            found: 3
        }
    );
}

/// Boards may be short and may hold wildcards; dead cards may do neither.
#[test]
fn test_board_and_dead_card_rules() {
    let flop = parse_board("Kh Qh Jh", 5).unwrap();
    assert_eq!(flop.len(), 3, "a flop is three of five");

    let with_wildcard = parse_board("Kh Qh h", 5).unwrap();
    assert_eq!(with_wildcard[2], CardSet::of_suit(Suit::Heart));

    assert!(parse_board("", 5).unwrap().is_empty());

    let dead = parse_dead("Ah Kd").unwrap();
    assert_eq!(dead.len(), 2);
    assert!(dead.contains(Card::new(Suit::Heart, Rank::Ace)));

    for wildcard in ["c", "A", "*"] {
        assert_eq!(
            parse_dead(wildcard).unwrap_err().kind,
            NotationErrorKind::DeadCardNotExact,
            "{:?} does not say which card is dead",
            wildcard
        );
    }
}

/// The mask API in PLAN section 3.6, for callers that never touch text.
#[test]
fn test_specs_can_be_built_from_masks() {
    let deuces = CardSet::of_rank(Rank::Two);
    let clubs = CardSet::of_suit(Suit::Club);
    let spec = HandSpec::from_slots(&[deuces, clubs]);

    assert_eq!(spec.slot_count(), Some(2));
    assert_eq!(spec.to_string(), "2 c", "renders in the notation");
    assert_eq!(spec, parse_hand("2 c", 2).unwrap(), "and matches the text");

    assert_eq!(spec.covered_cards(), deuces.union(clubs));
    assert!(spec.is_satisfiable());

    // A slot no card can fill makes the whole alternative impossible.
    let impossible = HandSpec::from_slots(&[deuces, CardSet::EMPTY]);
    assert!(!impossible.is_satisfiable());
}

/// Alternatives that disagree about how many cards they hold cannot be
/// sampled, so the count is reported as unknown.
#[test]
fn test_slot_count_needs_agreement() {
    let ragged = HandSpec::from_alternatives(vec![
        vec![CardSet::FULL_DECK],
        vec![CardSet::FULL_DECK, CardSet::FULL_DECK],
    ]);
    assert_eq!(ragged.slot_count(), None);
    assert_eq!(HandSpec::default().slot_count(), None);
}
