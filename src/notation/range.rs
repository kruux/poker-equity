use crate::cards::{Card, Rank, Suit};

use super::error::NotationErrorKind;

/// Whether a two-card term is restricted to sharing a suit.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Suitedness {
    Suited,
    Offsuit,
    Any,
}

/// A range term such as `AKs`, `AKo`, `AK` or `22`, normalised so that `high`
/// is never below `low`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Combo {
    high: Rank,
    low: Rank,
    suitedness: Suitedness,
}

impl Combo {
    fn is_pair(self) -> bool {
        self.high == self.low
    }

    /// The gap between the two ranks; zero for a pair, one for connectors.
    fn gap(self) -> u8 {
        self.high.to_value() - self.low.to_value()
    }

    /// Every concrete two-card hand this term admits.
    fn cards(self) -> Vec<[Card; 2]> {
        let mut out = Vec::new();
        if self.is_pair() {
            let suits = Suit::all();
            for i in 0..suits.len() {
                for j in (i + 1)..suits.len() {
                    out.push([
                        Card::new(suits[i], self.high),
                        Card::new(suits[j], self.low),
                    ]);
                }
            }
            return out;
        }
        for high_suit in Suit::all() {
            for low_suit in Suit::all() {
                let shares_suit = high_suit == low_suit;
                let wanted = match self.suitedness {
                    Suitedness::Suited => shares_suit,
                    Suitedness::Offsuit => !shares_suit,
                    Suitedness::Any => true,
                };
                if wanted {
                    out.push([
                        Card::new(high_suit, self.high),
                        Card::new(low_suit, self.low),
                    ]);
                }
            }
        }
        out
    }
}

/// A rank from its value in `2..=14`.
fn rank_at(value: u8) -> Rank {
    Rank::from_index(value - 2)
}

/// Reads one range term, or `None` when the text is not one at all.
///
/// A term is a range only when it matches this grammar exactly: two ranks,
/// then an optional `s` or `o`. That is what keeps `Ts` (the ten of spades,
/// one rank and a suit) apart from `AKs` (a range, two ranks and a marker).
fn parse_combo(term: &str) -> Option<Result<Combo, NotationErrorKind>> {
    let chars: Vec<char> = term.chars().collect();
    if chars.len() < 2 || chars.len() > 3 {
        return None;
    }
    let first = Rank::from_char(chars[0]).ok()?;
    let second = Rank::from_char(chars[1]).ok()?;

    let suitedness = match chars.get(2).map(|c| c.to_ascii_lowercase()) {
        None => Suitedness::Any,
        Some('s') => Suitedness::Suited,
        Some('o') => Suitedness::Offsuit,
        Some(_) => return None,
    };

    let (high, low) = if first.to_value() >= second.to_value() {
        (first, second)
    } else {
        (second, first)
    };

    if high == low && suitedness != Suitedness::Any {
        return Some(Err(NotationErrorKind::PairWithSuitedness));
    }

    Some(Ok(Combo {
        high,
        low,
        suitedness,
    }))
}

/// Walks `combo` upwards, the way `+` does: `22+` is every pair, `A2s+` runs
/// to `AKs`, `KTs+` stops below the king.
fn walk_up(combo: Combo) -> Vec<Combo> {
    let mut out = Vec::new();
    if combo.is_pair() {
        for value in combo.high.to_value()..=Rank::Ace.to_value() {
            out.push(Combo {
                high: rank_at(value),
                low: rank_at(value),
                ..combo
            });
        }
        return out;
    }
    for value in combo.low.to_value()..combo.high.to_value() {
        out.push(Combo {
            low: rank_at(value),
            ..combo
        });
    }
    out
}

/// Walks between two terms, the way `JTs-76s` does. Both ends must share a
/// gap and a suitedness, and either may be written first.
fn walk_between(a: Combo, b: Combo) -> Result<Vec<Combo>, NotationErrorKind> {
    if a.suitedness != b.suitedness || a.gap() != b.gap() {
        return Err(NotationErrorKind::MismatchedRangeEnds);
    }
    let (lower, upper) = if a.high.to_value() <= b.high.to_value() {
        (a, b)
    } else {
        (b, a)
    };
    let gap = lower.gap();
    let mut out = Vec::new();
    for value in lower.high.to_value()..=upper.high.to_value() {
        out.push(Combo {
            high: rank_at(value),
            low: rank_at(value - gap),
            suitedness: lower.suitedness,
        });
    }
    Ok(out)
}

/// Reads a whole range term into the hands it names.
///
/// Returns `None` when `term` is not a range at all, so the caller can fall
/// back to reading it as a list of slots. Returns `Some(Err(..))` when it is
/// plainly meant as a range but cannot be one.
pub(crate) fn parse_range(term: &str) -> Option<Result<Vec<[Card; 2]>, NotationErrorKind>> {
    if term.contains('%') {
        return Some(Err(NotationErrorKind::PercentageRange));
    }

    let combos = if let Some((left, right)) = term.split_once('-') {
        let left = match parse_combo(left)? {
            Ok(combo) => combo,
            Err(kind) => return Some(Err(kind)),
        };
        let right = match parse_combo(right)? {
            Ok(combo) => combo,
            Err(kind) => return Some(Err(kind)),
        };
        match walk_between(left, right) {
            Ok(combos) => combos,
            Err(kind) => return Some(Err(kind)),
        }
    } else if let Some(base) = term.strip_suffix('+') {
        match parse_combo(base)? {
            Ok(combo) => walk_up(combo),
            Err(kind) => return Some(Err(kind)),
        }
    } else {
        match parse_combo(term)? {
            Ok(combo) => vec![combo],
            Err(kind) => return Some(Err(kind)),
        }
    };

    if combos.is_empty() {
        return Some(Err(NotationErrorKind::EmptyRange));
    }

    let mut hands = Vec::new();
    for combo in combos {
        hands.extend(combo.cards());
    }
    Some(Ok(hands))
}
