// Where a hand's key lands in the lookup tables.
//
// This file is compiled into the crate *and* included by build.rs, so the
// generator and the evaluator cannot disagree about where a key belongs. It
// deliberately has no dependencies beyond integer arithmetic.

/// The rank table is indexed in two steps, because the keys are far too
/// sparse to index directly: a base-five rank multiset runs to 1.1 billion
/// while only 73,775 of those values are hands.
///
/// So the keys are scattered into buckets, and each bucket carries a
/// displacement chosen at build time that moves its handful of keys onto
/// slots nobody else wanted. Two multiplies and two array reads, with no
/// probing and no key comparison, and the table itself holds nothing but
/// scores.
pub const BUCKET_BITS: u32 = 14;

/// The slot count, as a power of two so that wrapping is a mask. Chosen to
/// leave the table a little over half empty, which is what makes the
/// displacement search terminate quickly.
pub const SLOT_BITS: u32 = 17;

/// How many buckets the keys are scattered into.
pub const BUCKET_COUNT: usize = 1 << BUCKET_BITS;
/// How many slots each kernel's score table holds.
pub const SLOT_COUNT: usize = 1 << SLOT_BITS;

/// Golden-ratio and splitmix constants: any odd multiplier spreads the high
/// bits, and these two are the usual choices.
const SCATTER: u64 = 0x9E37_79B9_7F4A_7C15;
const STRIDE: u64 = 0xD6E8_FEB8_6659_FD93;

/// Which bucket a key belongs to.
pub const fn bucket_of(key: u32) -> usize {
    ((key as u64).wrapping_mul(SCATTER) >> (64 - BUCKET_BITS)) as usize
}

/// Where a key sits once its bucket's displacement is applied.
///
/// The stride is forced odd so that successive displacements walk the whole
/// table rather than a cycle of it.
pub const fn slot_of(key: u32, displacement: u16) -> usize {
    let start = ((key as u64).wrapping_mul(SCATTER) >> (64 - SLOT_BITS)) as usize;
    let stride = ((key as u64).wrapping_mul(STRIDE) | 1) as usize;
    start.wrapping_add((displacement as usize).wrapping_mul(stride)) & (SLOT_COUNT - 1)
}
