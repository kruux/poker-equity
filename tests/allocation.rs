//! The deal loop must not reach for the allocator once it is running.
//!
//! Setting a chunk up allocates -- the hands, the buffers, the samplers -- and
//! that is fine, because it happens once. What must not happen is an
//! allocation per deal: it is the difference between a loop that scales with
//! the deals asked for and one that scales with the allocator's mood.
//!
//! So this counts allocations rather than timing anything. A timing assertion
//! would be flaky on a loaded machine; a count is exact.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use poker_equity::{
    odds::{run_chunk, EquityRequest},
    variants::{Badugi, Holdem, Omaha, OmahaHiLo, Razz},
};

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

/// The system allocator, keeping a tally.
struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
}

#[global_allocator]
static ALLOCATOR: Counting = Counting;

/// How many times a run of `samples` deals asks the allocator for memory.
macro_rules! allocations_for {
    ($variant:expr, $hands:expr, $samples:expr, $seed:expr) => {{
        let request =
            EquityRequest::from_text($variant, $hands, "", "").expect("the spot should be valid");
        // A warm-up, so that whatever the first run grows into is already
        // grown by the time anything is counted.
        run_chunk(&request, 1_000, 1).expect("sampling should not fail");

        let before = ALLOCATIONS.load(Ordering::Relaxed);
        run_chunk(&request, $samples, $seed).expect("sampling should not fail");
        ALLOCATIONS.load(Ordering::Relaxed) - before
    }};
}

/// Ten times the deals for the same number of allocations, which can only be
/// true if the loop allocates none of them.
#[test]
fn test_the_deal_loop_does_not_grow_with_the_deals() {
    let ten_thousand = allocations_for!(Holdem, &["AhKh", "QsQd"], 10_000, 2);
    // Setting a chunk up costs about fifteen; a loop that allocated even
    // once a deal would cost ten thousand.
    assert!(
        ten_thousand < 100,
        "setting a chunk up took {} allocations, which is more than setup",
        ten_thousand
    );
    let hundred_thousand = allocations_for!(Holdem, &["AhKh", "QsQd"], 100_000, 3);
    assert_eq!(
        ten_thousand, hundred_thousand,
        "hold'em allocated {} times for ten thousand deals and {} for a hundred thousand",
        ten_thousand, hundred_thousand
    );

    // Omaha scores sixty pairings a hand and badugi fifteen subsets, and
    // neither may build anything to do it.
    let ten_thousand = allocations_for!(Omaha, &["AhKh7c2d", "QsQdJsTd"], 10_000, 2);
    let hundred_thousand = allocations_for!(Omaha, &["AhKh7c2d", "QsQdJsTd"], 100_000, 3);
    assert_eq!(
        ten_thousand, hundred_thousand,
        "omaha allocated {} times for ten thousand deals and {} for a hundred thousand",
        ten_thousand, hundred_thousand
    );

    // A split game scores two halves a seat, and a stud game deals its cards
    // over several streets; neither may allocate to do it.
    let ten_thousand = allocations_for!(OmahaHiLo, &["Ah2c3d4s", "QsQdJsTd"], 10_000, 2);
    let hundred_thousand = allocations_for!(OmahaHiLo, &["Ah2c3d4s", "QsQdJsTd"], 100_000, 3);
    assert_eq!(
        ten_thousand, hundred_thousand,
        "omaha hi/lo allocated {} times for ten thousand deals and {} for a hundred thousand",
        ten_thousand, hundred_thousand
    );

    let ten_thousand = allocations_for!(Razz, &["Ah2c3d", "4s5h7c"], 10_000, 2);
    let hundred_thousand = allocations_for!(Razz, &["Ah2c3d", "4s5h7c"], 100_000, 3);
    assert_eq!(
        ten_thousand, hundred_thousand,
        "razz allocated {} times for ten thousand deals and {} for a hundred thousand",
        ten_thousand, hundred_thousand
    );

    let ten_thousand = allocations_for!(Badugi, &["Ac2d3h4s", "5s6h7d8c"], 10_000, 2);
    let hundred_thousand = allocations_for!(Badugi, &["Ac2d3h4s", "5s6h7d8c"], 100_000, 3);
    assert_eq!(
        ten_thousand, hundred_thousand,
        "badugi allocated {} times for ten thousand deals and {} for a hundred thousand",
        ten_thousand, hundred_thousand
    );
}
