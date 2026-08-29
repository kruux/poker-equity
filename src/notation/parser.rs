use crate::cards::{Card, CardSet, Rank, Suit};

use super::error::{NotationError, NotationErrorKind};
use super::hand_spec::HandSpec;
use super::range::parse_range;

/// Splits a field into its comma-separated alternatives, keeping the byte
/// offset of each so that an error can point back into the original text.
fn split_terms(text: &str) -> Vec<(&str, usize)> {
    let mut terms = Vec::new();
    let mut start = 0;
    for (offset, ch) in text.char_indices() {
        if ch == ',' {
            terms.push(trimmed(&text[start..offset], start));
            start = offset + ch.len_utf8();
        }
    }
    terms.push(trimmed(&text[start..], start));
    terms
}

/// Trims a term, moving its offset along with it.
fn trimmed(term: &str, offset: usize) -> (&str, usize) {
    let lead = term.len() - term.trim_start().len();
    (term.trim(), offset + lead)
}

/// Reads a whitespace-separated run of card patterns into one mask per slot.
///
/// This is where the binding rule lives: a rank takes the suit that
/// *immediately* follows it and nothing else binds, so `2c` is one slot and
/// `2 c` is two. `*` never binds, which is what makes `AA**` unambiguously
/// four slots.
fn parse_slots(text: &str, base: usize) -> Result<Vec<CardSet>, NotationError> {
    let glyphs: Vec<(usize, char)> = text.char_indices().collect();
    let mut slots = Vec::new();
    let mut named: Vec<Card> = Vec::new();
    let mut i = 0;

    while i < glyphs.len() {
        let (offset, glyph) = glyphs[i];

        if glyph.is_whitespace() {
            i += 1;
            continue;
        }

        if glyph == '*' {
            slots.push(CardSet::FULL_DECK);
            i += 1;
            continue;
        }

        if let Ok(rank) = Rank::from_char(glyph) {
            // A suit binds only when it is the very next character.
            if let Some(&(next_offset, next)) = glyphs.get(i + 1) {
                if next_offset == offset + glyph.len_utf8() {
                    if let Ok(suit) = Suit::from_char(next) {
                        let card = Card::new(suit, rank);
                        if named.contains(&card) {
                            return Err(NotationError::new(
                                NotationErrorKind::DuplicateCard(card.to_string()),
                                base + offset,
                                glyph.len_utf8() + next.len_utf8(),
                            ));
                        }
                        named.push(card);
                        slots.push(CardSet::from_cards(&[card]));
                        i += 2;
                        continue;
                    }
                }
            }
            slots.push(CardSet::of_rank(rank));
            i += 1;
            continue;
        }

        if let Ok(suit) = Suit::from_char(glyph) {
            slots.push(CardSet::of_suit(suit));
            i += 1;
            continue;
        }

        return Err(NotationError::new(
            NotationErrorKind::UnknownGlyph(glyph),
            base + offset,
            glyph.len_utf8(),
        ));
    }

    Ok(slots)
}

/// Reads one alternative, as a range where the game has ranges and as a list
/// of slots otherwise.
fn parse_alternative(
    term: &str,
    offset: usize,
    slots: usize,
    exact: bool,
) -> Result<Vec<Vec<CardSet>>, NotationError> {
    if term.is_empty() {
        return Err(NotationError::new(NotationErrorKind::Empty, offset, 0));
    }

    let span = term.len();

    if slots == 2 {
        if let Some(range) = parse_range(term) {
            let hands = range.map_err(|kind| NotationError::new(kind, offset, span))?;
            return Ok(HandSpec::from_combos(&hands).alternatives);
        }
    } else if term.contains(['+', '-', '%']) {
        // Those glyphs only ever mean a range, and ranges are written only
        // where a hand holds two cards.
        return Err(NotationError::new(
            NotationErrorKind::RangesNotAllowed,
            offset,
            span,
        ));
    }

    let parsed = parse_slots(term, offset)?;
    let miscounted = if exact {
        parsed.len() != slots
    } else {
        parsed.is_empty() || parsed.len() > slots
    };
    if miscounted {
        // A miscount around a `*` almost always means the writer expected it
        // to bind, so say so rather than only reporting the number.
        let kind = if term.contains('*') {
            NotationErrorKind::WildcardSlotCount {
                expected: slots,
                found: parsed.len(),
            }
        } else {
            NotationErrorKind::WrongSlotCount {
                expected: slots,
                found: parsed.len(),
            }
        };
        return Err(NotationError::new(kind, offset, span));
    }
    Ok(vec![parsed])
}

/// Reads a hand field for a game that deals `slots` cards to a player.
///
/// Ranges such as `AKs` or `22+` are recognised only when `slots` is two,
/// which is what makes the collision between `AKs` the range and `A` `Ks` the
/// two slots impossible in Omaha and stud rather than merely unlikely.
pub fn parse_hand(text: &str, slots: usize) -> Result<HandSpec, NotationError> {
    parse_field(text, slots, true)
}

/// Reads a hand field that may name fewer cards than the game deals.
///
/// Where a player's cards arrive over time -- stud dealt street by street, a
/// draw game where cards are exchanged -- the field says what is held now and
/// the rest are still to come. A five-card draw hand stands pat; a three-card
/// one draws two.
pub fn parse_hand_up_to(text: &str, slots: usize) -> Result<HandSpec, NotationError> {
    parse_field(text, slots, false)
}

fn parse_field(text: &str, slots: usize, exact: bool) -> Result<HandSpec, NotationError> {
    let mut alternatives = Vec::new();
    for (term, offset) in split_terms(text) {
        alternatives.extend(parse_alternative(term, offset, slots, exact)?);
    }
    Ok(HandSpec::from_alternatives(alternatives))
}

/// Reads a board, which may be short and may hold wildcards.
///
/// A board is one hand, not a list of alternatives, so commas are not
/// accepted here.
pub fn parse_board(text: &str, limit: usize) -> Result<Vec<CardSet>, NotationError> {
    if let Some(offset) = text.find(',') {
        return Err(NotationError::new(
            NotationErrorKind::UnknownGlyph(','),
            offset,
            1,
        ));
    }
    let (term, offset) = trimmed(text, 0);
    if term.is_empty() {
        return Ok(Vec::new());
    }
    let slots = parse_slots(term, offset)?;
    if slots.len() > limit {
        return Err(NotationError::new(
            NotationErrorKind::TooManySlots {
                limit,
                found: slots.len(),
            },
            offset,
            term.len(),
        ));
    }
    Ok(slots)
}

/// Reads dead cards, which must be named exactly.
///
/// "A club is dead" does not say *which* club, and every reading changes the
/// answer, so a wildcard is refused rather than guessed at.
pub fn parse_dead(text: &str) -> Result<CardSet, NotationError> {
    let (term, offset) = trimmed(text, 0);
    if term.is_empty() {
        return Ok(CardSet::EMPTY);
    }

    // Re-walk the term so the error can point at the offending pattern
    // rather than at the field as a whole.
    let mut dead = CardSet::EMPTY;
    let mut cursor = offset;
    for pattern in term.split_whitespace() {
        let at = text[cursor..]
            .find(pattern)
            .map(|found| cursor + found)
            .unwrap_or(cursor);
        cursor = at + pattern.len();

        for slot in parse_slots(pattern, at)? {
            if slot.len() != 1 {
                return Err(NotationError::new(
                    NotationErrorKind::DeadCardNotExact,
                    at,
                    pattern.len(),
                ));
            }
            dead = dead.union(slot);
        }
    }
    Ok(dead)
}
