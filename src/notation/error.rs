use std::fmt;

/// What went wrong while reading a notation field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotationErrorKind {
    /// A character that is neither a rank, a suit, `*`, nor punctuation.
    UnknownGlyph(char),
    /// A rank or suit was expected and the field ended instead.
    UnexpectedEnd,
    /// The field, or one of its alternatives, is empty.
    Empty,
    /// The same card appears twice in one alternative.
    DuplicateCard(String),
    /// The alternative has the wrong number of slots for this game.
    WrongSlotCount { expected: usize, found: usize },
    /// The wrong number of slots, where a `*` is the likely reason: `*` is
    /// always exactly one card and never binds to the rank before it.
    WildcardSlotCount { expected: usize, found: usize },
    /// A range was written where the game has no range grammar.
    RangesNotAllowed,
    /// A pair cannot be marked suited or offsuit.
    PairWithSuitedness,
    /// The two ends of a `-` range do not share a gap.
    MismatchedRangeEnds,
    /// A `+` or `-` range that names no hand at all.
    EmptyRange,
    /// Percentage ranges are deliberately not supported.
    PercentageRange,
    /// A dead card was written as a wildcard, which does not say which card
    /// is dead.
    DeadCardNotExact,
    /// More cards were written than the field can hold.
    TooManySlots { limit: usize, found: usize },
}

/// Picks the singular or plural wording for `count`.
fn plural(count: usize, one: &'static str, many: &'static str) -> &'static str {
    if count == 1 {
        one
    } else {
        many
    }
}

impl NotationErrorKind {
    /// A stable short name, used by the conformance fixture so that both
    /// implementations of the grammar can agree on which error was raised
    /// without agreeing on its wording.
    pub fn name(&self) -> &'static str {
        match self {
            Self::UnknownGlyph(_) => "UnknownGlyph",
            Self::UnexpectedEnd => "UnexpectedEnd",
            Self::Empty => "Empty",
            Self::DuplicateCard(_) => "DuplicateCard",
            Self::WrongSlotCount { .. } => "WrongSlotCount",
            Self::WildcardSlotCount { .. } => "WildcardSlotCount",
            Self::RangesNotAllowed => "RangesNotAllowed",
            Self::PairWithSuitedness => "PairWithSuitedness",
            Self::MismatchedRangeEnds => "MismatchedRangeEnds",
            Self::EmptyRange => "EmptyRange",
            Self::PercentageRange => "PercentageRange",
            Self::DeadCardNotExact => "DeadCardNotExact",
            Self::TooManySlots { .. } => "TooManySlots",
        }
    }
}

impl fmt::Display for NotationErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::UnknownGlyph(c) => write!(f, "'{}' is not a rank, a suit, or '*'", c),
            Self::UnexpectedEnd => write!(f, "the field ends in the middle of a card"),
            Self::Empty => write!(f, "nothing to read here"),
            Self::DuplicateCard(card) => write!(f, "{} is named twice in one hand", card),
            Self::WrongSlotCount { expected, found } => write!(
                f,
                "this game deals {} {}, but {} {} written",
                expected,
                plural(*expected, "card", "cards"),
                found,
                plural(*found, "was", "were"),
            ),
            Self::WildcardSlotCount { expected, found } => write!(
                f,
                "this game deals {} {}, but {} {} written: '*' is always \
                 exactly one card and never binds to the rank before it, so \
                 'A*' is two cards, not one",
                expected,
                plural(*expected, "card", "cards"),
                found,
                plural(*found, "was", "were"),
            ),
            Self::RangesNotAllowed => {
                write!(f, "ranges are only written where a hand holds two cards")
            }
            Self::PairWithSuitedness => write!(f, "a pair cannot be suited or offsuit"),
            Self::MismatchedRangeEnds => {
                write!(f, "both ends of a range must share the same gap")
            }
            Self::EmptyRange => write!(f, "this range names no hand"),
            Self::PercentageRange => write!(
                f,
                "percentage ranges need a hand-strength ordering this library does not define"
            ),
            Self::DeadCardNotExact => write!(
                f,
                "a dead card must name one card, since every reading changes the answer"
            ),
            Self::TooManySlots { limit, found } => {
                write!(f, "at most {} cards fit here, but {} were written", limit, found)
            }
        }
    }
}

/// A notation error, carrying the span of the text at fault.
///
/// `offset` and `len` are byte positions into the field as it was written, so
/// a caller can underline the problem rather than only describing it. Case is
/// folded per character while reading, which never moves a byte, so the span
/// always lines up with the original input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotationError {
    pub kind: NotationErrorKind,
    pub offset: usize,
    pub len: usize,
}

impl NotationError {
    /// An error of `kind`, covering `len` bytes from `offset`.
    pub fn new(kind: NotationErrorKind, offset: usize, len: usize) -> Self {
        Self { kind, offset, len }
    }

    /// The offending text underlined, for a terminal or a log.
    pub fn underline(&self, input: &str) -> String {
        format!(
            "{}\n{}{}\n{}",
            input,
            " ".repeat(self.offset),
            "^".repeat(self.len.max(1)),
            self.kind
        )
    }
}

impl fmt::Display for NotationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (at byte {})", self.kind, self.offset)
    }
}

impl std::error::Error for NotationError {}
