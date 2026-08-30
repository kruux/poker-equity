//! The Python binding, and the only place this crate knows Python exists.
//!
//! It lives in one module behind the `python` feature so the core never
//! depends on PyO3, and so a standalone binary or a C ABI would be an
//! addition rather than a rewrite.
//!
//! The boundary carries `u64` masks and `u8` card indices, which is what
//! a caller's Python side is likely to hold already, so nothing is translated
//! crossing it.
//! Text may cross too, for the convenience calls.

use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};

use crate::cards::{Card, CardSet};
use crate::error::PokerError;
use crate::notation::{parse_dead, parse_hand, parse_hand_up_to, HandSpec};
use crate::odds::{
    default_threads, run_batch, run_chunk, run_exact, ChunkResult, EquityRequest,
};
use crate::variants::*;

/// Runs `body` with the variant named by `key` bound to `variant`.
///
/// The engine is generic over the variant, which is what keeps each game's
/// rules in its own type rather than in a runtime branch. Python needs to
/// pick one by name, so the body is expanded once per game here -- the only
/// place the two worlds have to meet.
macro_rules! with_variant {
    ($key:expr, |$variant:ident| $body:expr) => {
        match $key {
            "holdem" => { let $variant = Holdem; $body }
            "short_deck" => { let $variant = ShortDeck; $body }
            "omaha" => { let $variant = Omaha; $body }
            "omaha_five" => { let $variant = OmahaFive; $body }
            "omaha_six" => { let $variant = OmahaSix; $body }
            "omaha_hi_lo" => { let $variant = OmahaHiLo; $body }
            "omaha_five_hi_lo" => { let $variant = OmahaFiveHiLo; $body }
            "courchevel" => { let $variant = Courchevel; $body }
            "courchevel_hi_lo" => { let $variant = CourchevelHiLo; $body }
            "stud" => { let $variant = Stud; $body }
            "stud_hi_lo" => { let $variant = StudHiLo; $body }
            "razz" => { let $variant = Razz; $body }
            "deuce_seven" => { let $variant = DeuceSeven; $body }
            "badugi" => { let $variant = Badugi; $body }
            other => Err(PyValueError::new_err(format!(
                "no game called {:?}; call variants() for the list",
                other
            ))),
        }
    };
}

/// Turns a library error into something Python can raise.
fn to_py(error: PokerError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/// Builds the per-slot masks for one player from Python's nested lists.
fn hand_spec(alternatives: Vec<Vec<u64>>) -> HandSpec {
    HandSpec::from_alternatives(
        alternatives
            .into_iter()
            .map(|slots| slots.into_iter().map(CardSet::from_bits).collect())
            .collect(),
    )
}

/// Renders a result as a plain dictionary, so the caller needs no class from
/// this module to read it.
///
/// Both the worked-out figures and the raw sums cross. The sums are what let
/// a caller merge batches on the Python side -- they add, which averages do
/// not -- and `share_square_sum` in particular is what a standard error is
/// computed from, so dropping it would mean a caller that merges its own
/// batches could not put an error bar on the result.
fn to_dict(py: Python<'_>, result: &ChunkResult) -> PyResult<Py<PyDict>> {
    let out = PyDict::new(py);
    out.set_item("samples", result.samples)?;
    out.set_item("exact", result.exact)?;
    out.set_item("attempts", result.attempts)?;
    out.set_item("acceptance", result.acceptance())?;

    // The raw sums, one entry per seat. These add across batches.
    out.set_item("share_sum", result.share_sum.clone())?;
    out.set_item("share_square_sum", result.share_square_sum.clone())?;
    out.set_item("low_share_sum", result.low_share_sum.clone())?;
    out.set_item("win_count", result.win_count.clone())?;
    out.set_item("tie_count", result.tie_count.clone())?;
    out.set_item("scoop_count", result.scoop_count.clone())?;

    let players = PyList::empty(py);
    for player in result.equities() {
        let seat = PyDict::new(py);
        seat.set_item("equity", player.equity)?;
        seat.set_item("win", player.win)?;
        seat.set_item("tie", player.tie)?;
        seat.set_item("low_equity", player.low_equity)?;
        seat.set_item("scoop", player.scoop)?;
        seat.set_item("std_error", player.std_error)?;
        players.append(seat)?;
    }
    out.set_item("players", players)?;
    Ok(out.into())
}

/// Every game this library plays.
///
/// Each entry carries the key to pass back in, a label to show, and how the
/// game deals, so a caller can build its own menu without a table of its own.
#[pyfunction]
fn variants(py: Python<'_>) -> PyResult<Py<PyList>> {
    let out = PyList::empty(py);
    for key in [
        "holdem",
        "short_deck",
        "omaha",
        "omaha_five",
        "omaha_six",
        "omaha_hi_lo",
        "omaha_five_hi_lo",
        "courchevel",
        "courchevel_hi_lo",
        "stud",
        "stud_hi_lo",
        "razz",
        "deuce_seven",
        "badugi",
    ] {
        let entry: PyResult<Py<PyDict>> = with_variant!(key, |variant| {
            let entry = PyDict::new(py);
            entry.set_item("key", variant.key())?;
            entry.set_item("label", variant.to_string())?;
            entry.set_item("hole_cards", variant.hole_cards())?;
            entry.set_item("board_cards", variant.board_cards())?;
            entry.set_item("deck", variant.deck().bits())?;
            entry.set_item("deck_size", variant.deck().len())?;
            Ok(entry.into())
        });
        out.append(entry?)?;
    }
    Ok(out.into())
}

/// Samples a batch of deals, taking masks.
///
/// `hands` is one entry per seat, each a list of whole alternatives, each
/// alternative one mask per card the player holds. A named card is a mask
/// with one bit; "any ace" is a mask with four.
///
/// The sampling runs with the GIL released, so a caller can keep repainting
/// while it works.
#[pyfunction]
#[pyo3(signature = (variant, hands, board, dead, samples, seed=0, threads=0))]
fn chunk(
    py: Python<'_>,
    variant: &str,
    hands: Vec<Vec<Vec<u64>>>,
    board: Vec<u64>,
    dead: u64,
    samples: u64,
    seed: u64,
    threads: usize,
) -> PyResult<Py<PyDict>> {
    let threads = if threads == 0 { default_threads() } else { threads };
    let specs: Vec<HandSpec> = hands.into_iter().map(hand_spec).collect();
    let board: Vec<CardSet> = board.into_iter().map(CardSet::from_bits).collect();
    let dead = CardSet::from_bits(dead);

    let result = with_variant!(variant, |game| {
        let request =
            EquityRequest::from_masks(game, &specs, &board, dead).map_err(to_py)?;
        py.detach(|| run_batch(&request, samples, seed, threads))
            .map_err(to_py)
    })?;

    to_dict(py, &result)
}

/// Walks every deal, taking masks, or returns `None` when there are too many.
///
/// The mask twin of [`exact_from_text`]. A caller passing masks could not ask
/// for an exact answer at all before this, which is backwards: exact matters
/// most on a river spot, and a river spot is exactly when someone has the
/// calculator open.
#[pyfunction]
#[pyo3(signature = (variant, hands, board, dead))]
fn exact(
    py: Python<'_>,
    variant: &str,
    hands: Vec<Vec<Vec<u64>>>,
    board: Vec<u64>,
    dead: u64,
) -> PyResult<Option<Py<PyDict>>> {
    let specs: Vec<HandSpec> = hands.into_iter().map(hand_spec).collect();
    let board: Vec<CardSet> = board.into_iter().map(CardSet::from_bits).collect();
    let dead = CardSet::from_bits(dead);

    let result = with_variant!(variant, |game| {
        let request =
            EquityRequest::from_masks(game, &specs, &board, dead).map_err(to_py)?;
        py.detach(|| run_exact(&request)).map_err(to_py)
    })?;

    match result {
        Some(result) => Ok(Some(to_dict(py, &result)?)),
        None => Ok(None),
    }
}

/// How many threads a batch spreads over when none is asked for.
#[pyfunction]
fn default_thread_count() -> usize {
    default_threads()
}

/// Samples a batch of deals, taking this library's notation.
///
/// The convenience form: `["AhKh", "QsQd"]` for hold'em, `"Kh Qh Jh"` for a
/// flop. The mask form is the one to reach for from a program that already
/// holds masks; this one is for everything else.
#[pyfunction]
#[pyo3(signature = (variant, hands, board="", dead="", samples=100_000, seed=0, threads=0))]
fn chunk_from_text(
    py: Python<'_>,
    variant: &str,
    hands: Vec<String>,
    board: &str,
    dead: &str,
    samples: u64,
    seed: u64,
    threads: usize,
) -> PyResult<Py<PyDict>> {
    let threads = if threads == 0 { default_threads() } else { threads };
    let fields: Vec<&str> = hands.iter().map(String::as_str).collect();

    let result = with_variant!(variant, |game| {
        let request = EquityRequest::from_text(game, &fields, board, dead).map_err(to_py)?;
        py.detach(|| run_batch(&request, samples, seed, threads))
            .map_err(to_py)
    })?;

    to_dict(py, &result)
}

/// Walks every deal instead of sampling, or returns `None` when there are too
/// many to walk.
///
/// An exact result carries a standard error of zero, because there is nothing
/// uncertain about it.
#[pyfunction]
#[pyo3(signature = (variant, hands, board="", dead=""))]
fn exact_from_text(
    py: Python<'_>,
    variant: &str,
    hands: Vec<String>,
    board: &str,
    dead: &str,
) -> PyResult<Option<Py<PyDict>>> {
    let fields: Vec<&str> = hands.iter().map(String::as_str).collect();

    let result = with_variant!(variant, |game| {
        let request = EquityRequest::from_text(game, &fields, board, dead).map_err(to_py)?;
        py.detach(|| run_exact(&request)).map_err(to_py)
    })?;

    match result {
        Some(result) => Ok(Some(to_dict(py, &result)?)),
        None => Ok(None),
    }
}

/// Reads a hand field into masks, which is how another implementation of
/// the notation can be checked against this one without running a whole
/// simulation.
///
/// `slots` is how many cards the game deals a player. Where cards arrive over
/// time -- stud, a draw game -- a shorter field is allowed and means the rest
/// are still to come.
#[pyfunction]
#[pyo3(signature = (text, slots, allow_short=false))]
fn parse_hand_field(text: &str, slots: usize, allow_short: bool) -> PyResult<Vec<Vec<u64>>> {
    let read = if allow_short { parse_hand_up_to } else { parse_hand };
    let spec = read(text, slots)
        .map_err(|error| PyValueError::new_err(error.underline(text)))?;
    Ok(spec
        .alternatives
        .into_iter()
        .map(|slots| slots.into_iter().map(|slot| slot.bits()).collect())
        .collect())
}

/// Reads dead cards into one mask. They must be named exactly.
#[pyfunction]
fn parse_dead_cards(text: &str) -> PyResult<u64> {
    parse_dead(text)
        .map(|set| set.bits())
        .map_err(|error| PyValueError::new_err(error.underline(text)))
}

/// The index of a named card, as `rank * 4 + suit`.
#[pyfunction]
fn card_index(text: &str) -> PyResult<u8> {
    let cards = Card::from_str(text).map_err(|error| PyValueError::new_err(error.to_string()))?;
    match cards.as_slice() {
        [card] => Ok(card.index()),
        _ => Err(PyValueError::new_err(format!(
            "{:?} names {} cards, not one",
            text,
            cards.len()
        ))),
    }
}

/// The name of a card index, uppercase rank and lowercase suit.
#[pyfunction]
fn card_name(index: u8) -> PyResult<String> {
    Card::from_index(index)
        .map(|card| card.to_string())
        .ok_or_else(|| PyValueError::new_err(format!("{} is not a card index", index)))
}

/// Scores a batch of hands against one of the ranking kernels.
///
/// Lower is better. This exists for `validation/`, which asks an outside
/// evaluator whether it agrees; it takes whole batches because crossing this
/// boundary a hundred million times, one hand at a time, would cost more than
/// the comparison itself.
///
/// `kernel` is one of `high`, `deuce_seven`, `low_a5`, `short_deck` or
/// `badugi`. Cards are written as usual: `"AhKhQsQd2c7d9s"`.
#[pyfunction]
fn score_batch(py: Python<'_>, kernel: &str, hands: Vec<String>) -> PyResult<Vec<u32>> {
    let score: fn(&[Card]) -> u32 = match kernel {
        "high" => |cards| crate::variants::high_score(cards) as u32,
        "deuce_seven" => |cards| crate::variants::deuce_seven_score(cards) as u32,
        "low_a5" => |cards| crate::variants::low_a5_score(cards) as u32,
        "short_deck" => |cards| crate::variants::short_deck_score(cards) as u32,
        "badugi" => |cards| Badugi.score(cards),
        "omaha" => |cards| Omaha.score(cards),
        other => {
            return Err(PyValueError::new_err(format!(
                "no kernel called {:?}",
                other
            )))
        }
    };

    // Parsing and scoring are pure arithmetic, so the GIL is not needed.
    let parsed = hands
        .iter()
        .map(|hand| Card::from_str(hand).map_err(|error| PyValueError::new_err(error.to_string())))
        .collect::<PyResult<Vec<_>>>()?;

    Ok(py.detach(|| parsed.iter().map(|cards| score(cards)).collect()))
}

/// A mask holding every card in a standard deck.
#[pyfunction]
fn full_deck() -> u64 {
    CardSet::FULL_DECK.bits()
}

#[pymodule]
fn poker_calculator(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(variants, module)?)?;
    module.add_function(wrap_pyfunction!(chunk, module)?)?;
    module.add_function(wrap_pyfunction!(chunk_from_text, module)?)?;
    module.add_function(wrap_pyfunction!(exact, module)?)?;
    module.add_function(wrap_pyfunction!(exact_from_text, module)?)?;
    module.add_function(wrap_pyfunction!(default_thread_count, module)?)?;
    module.add_function(wrap_pyfunction!(parse_hand_field, module)?)?;
    module.add_function(wrap_pyfunction!(parse_dead_cards, module)?)?;
    module.add_function(wrap_pyfunction!(card_index, module)?)?;
    module.add_function(wrap_pyfunction!(card_name, module)?)?;
    module.add_function(wrap_pyfunction!(score_batch, module)?)?;
    module.add_function(wrap_pyfunction!(full_deck, module)?)?;
    Ok(())
}
