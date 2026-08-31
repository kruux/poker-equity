//! Nothing a caller can type may panic.
//!
//! Every entry point here takes text or masks from outside, and the contract
//! is the same for all of them: a `Result`, never a panic. A library that
//! aborts the process on a typo cannot be used behind a server or a UI, and
//! the caller has no way to defend against it.
//!
//! This is a regression test rather than a proof. It is cheap enough to run on
//! every build -- eighty thousand cases in a tenth of a second -- and it has
//! already earned its place: `Card::parse_field("")` used to panic, because
//! nought characters is an even number, so the length check passed and the
//! loop then computed `0 - 1`.
//!
//! Run it in debug as well as release. The test profile keeps debug
//! assertions on, which is what turns an arithmetic overflow into a panic
//! this test can see; in release it would wrap silently.

use poker_equity::{
    cards::{Card, CardSet},
    notation::{parse_board, parse_dead, parse_hand, parse_hand_up_to},
    odds::{run_chunk, EquityRequest},
    variants::*,
};

/// Runs one case, and records it if it panicked rather than returning.
///
/// Catching the panic rather than letting it fail the test outright is what
/// makes a failure useful: one run reports every input that panicked, instead
/// of stopping at the first and hiding the rest.
macro_rules! guard {
    ($found:expr, $label:expr, $body:expr) => {{
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = $body;
        }))
        .is_err()
        {
            $found.push($label);
        }
    }};
}

/// Reports every case that panicked, or passes.
fn settle(found: Vec<String>, what: &str) {
    assert!(
        found.is_empty(),
        "{} panicked on {} input{}:\n  {}",
        what,
        found.len(),
        if found.len() == 1 { "" } else { "s" },
        found.join("\n  ")
    );
}

/// Text a caller might plausibly send, and some nobody would.
///
/// The hand-written entries are the shapes with a reason to be tried: empty,
/// whitespace, half a card, a card and a half, wildcards alone and in company,
/// case variations, separators in the wrong places, and characters from
/// outside the alphabet entirely. The generated pairs then cover every
/// two-character string over the alphabet the grammar actually uses, which is
/// where an off-by-one in the parser would show.
fn junk() -> Vec<String> {
    let mut out: Vec<String> = [
        "", " ", "  ", "\t", "\n", "A", "Ah", "AhK", "AhKh", "*", "**", "***", "A*", "*h", "AA",
        "hh", "1h", "0", "10h", "Th", "AhAh", "AhAhAh", "A h", "a h", "ah", "AH", "aH", ",", ",,",
        "Ah,", ",Ah", "Ah,,Kh", "AKs", "AKo", "AK", "22", "2 c", "T9s,", "-", "Ah-Kh", "+", "%",
        "\\", "\u{0}", "\u{7f}", "é", "🂡", "Ah🂡",
    ]
    .into_iter()
    .map(String::from)
    .collect();
    out.push("x".repeat(1000));

    let alphabet = "AKQJT98765432cdhs*, xX10";
    for a in alphabet.chars() {
        for b in alphabet.chars() {
            out.push(format!("{}{}", a, b));
        }
    }
    out
}

/// The notation, which is the surface a caller touches first and most.
#[test]
fn test_no_field_however_malformed_can_panic() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut found = Vec::new();

    for text in junk() {
        // Zero slots through more than any game deals, since the slot count
        // comes from the variant and a new variant could widen it.
        for slots in 0..=8usize {
            guard!(found, format!("parse_hand({:?}, {})", text, slots), parse_hand(&text, slots));
            guard!(
                found,
                format!("parse_hand_up_to({:?}, {})", text, slots),
                parse_hand_up_to(&text, slots)
            );
            guard!(found, format!("parse_board({:?}, {})", text, slots), parse_board(&text, slots));
        }
        guard!(found, format!("parse_dead({:?})", text), parse_dead(&text));
        guard!(found, format!("Card::parse_field({:?})", text), Card::parse_field(&text));
        guard!(found, format!("{:?}.parse::<Card>()", text), text.parse::<Card>());
    }

    std::panic::set_hook(hook);
    settle(found, "the notation");
}

/// Whole requests, built and then dealt from.
///
/// Building is swept exhaustively because that is where the checks live and it
/// costs nothing. Dealing is run on a capped number of the requests that
/// built, since a panic in the deal loop shows on the first deal or not at
/// all.
#[test]
fn test_no_request_however_odd_can_panic() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut found = Vec::new();

    let fields = [
        "", "Ah", "AhKh", "AhKhQh", "AhKhQhJh", "AhKhQhJhTh", "AhKhQhJhTh9h", "AhKhQhJhTh9h8h",
        "AhKhQhJhTh9h8h7h", "**", "AKs", "AhAh", "*",
    ];
    let boards = ["", "2c", "2c3d", "2c3d4h", "2c3d4h5s", "2c3d4h5s6c", "2c3d4h5s6c7d", "*", "*****"];
    let deads = ["", "2h", "Ah", "Ah2h3h4h5h6h7h8h9hTh"];

    macro_rules! sweep {
        ($variant:expr) => {{
            let mut dealt = 0usize;
            for hero in fields {
                for villain in fields {
                    for board in boards {
                        for dead in deads {
                            let label = format!(
                                "{} {:?} against {:?}, board {:?}, dead {:?}",
                                $variant.key(),
                                hero,
                                villain,
                                board,
                                dead
                            );
                            guard!(found, label, {
                                if let Ok(request) =
                                    EquityRequest::from_text($variant, &[hero, villain], board, dead)
                                {
                                    if dealt < 400 {
                                        dealt += 1;
                                        let _ = run_chunk(&request, 8, 7);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }};
    }

    sweep!(Holdem);
    sweep!(ShortDeck);
    sweep!(Omaha);
    sweep!(OmahaHiLo);
    sweep!(Courchevel);
    sweep!(Stud);
    sweep!(StudHiLo);
    sweep!(Razz);
    sweep!(DeuceSeven);
    sweep!(Badugi);

    std::panic::set_hook(hook);
    settle(found, "requests");
}

/// The mask form, where a caller can hand over bits the notation would never
/// have produced -- every bit set, no bits set, or bits outside the deck.
#[test]
fn test_no_mask_however_odd_can_panic() {
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let mut found = Vec::new();

    let odd = [
        CardSet::EMPTY,
        CardSet::FULL_DECK,
        CardSet::SHORT_DECK,
        CardSet::from_bits(u64::MAX),
        CardSet::from_bits(1),
    ];

    for slot in odd {
        for board_len in 0..=6usize {
            let board: Vec<CardSet> = vec![slot; board_len];
            for dead in odd {
                let label = format!("board of {} x {:#x}, dead {:#x}", board_len, slot.bits(), dead.bits());
                guard!(found, label, {
                    let spec = parse_hand("AhKh", 2).expect("a fixed, valid field");
                    EquityRequest::from_masks(Holdem, &[spec.clone(), spec], &board, dead)
                });
            }
        }
    }

    std::panic::set_hook(hook);
    settle(found, "the mask form");
}
