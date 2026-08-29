# poker-calculator — what the library needs to be

A specification, not a diff. It describes the finished library; how much of the current tree
survives is an implementation decision. Where the current code has already settled a
question well, that is noted so the answer is not re-derived.

The consumer that drives the requirements is **fpdb3**, a poker tracker whose odds calculator
this becomes. But the library must stand alone: it takes text, it answers questions about
poker, and it knows nothing about fpdb.

---

## 1. What it is

A poker equity engine. Given a game, some hands (possibly partly specified), a board and some
dead cards, it reports **what share of the pot each player wins**.

Two properties matter above all others:

1. **Speed.** The sampling loop is the product. A fast evaluator called from a slow loop is a
   slow calculator.
2. **Correctness under partial information.** "Any deuce", "any club", "any card" are
   first-class inputs, not an afterthought — and getting their *distribution* right is
   subtler than it looks (§7).

### Non-goals for now

- **No publishing.** Not to crates.io, not to PyPI, until the build is solid. Design for it
  (§10) but do not do it.
- No solver, no range construction, no GTO, no hand-history parsing.

---

## 2. Card representation

**A card is a `u8` in `0..52`, laid out as `rank * 4 + suit`.**

- Ranks `0..13` are `23456789TJQKA` — index *is* rank order, ace high. The low kernels
  re-map the ace rather than reordering this.
- Suits `0..4` are `cdhs`. Suit order is arbitrary and must never be compared.

**A set of cards is a `u64` bitmask**, bit *n* being card *n*. This is the type the whole
engine speaks: a rank wildcard is a constant mask, a suit wildcard is a constant mask, "what
is left in the deck" is one `&` and one `!`.

> This encoding is **binding**. fpdb's Python side already produces exactly these indices and
> masks, so the FFI boundary carries them with no translation. Changing it breaks that for no
> gain.

`struct Card { suit: Suit, rank: Rank }` and `Vec<Card>` are the wrong shape for the hot path:
they cost a pointer chase and an allocation where a `u64` costs a register. Keep a rich card
type for display and for the public text API if it helps; the engine must not see it.

---

## 3. Text notation

The library parses its own input. This is what makes it generally useful rather than an
fpdb component, and the grammar below is normative.

### 3.1 Card patterns

| Written | Admits |
|---|---|
| `Ah` | the ace of hearts |
| `A` | any ace |
| `c` | any club |
| `*` | any card |

Every pattern is a `u64` mask. A concrete card is a single-bit mask; the parser has no other
notion of "known" versus "unknown".

**Case-insensitive input, canonical output.** No rank glyph is also a suit glyph in either
case — there is no rank `C`/`D`/`H`/`S`, no suit `t`/`j`/`q`/`k`/`a` — so case can be folded
before parsing and cannot be load-bearing. Output is always uppercase rank, lowercase suit.

### 3.2 The one binding rule

**A rank binds to a suit that immediately follows it. Nothing else binds.**

`2c` is the two of clubs; `2 c` is a deuce and a club. Both are well-formed and they differ by
a whole card, so the wrong reading yields a plausible wrong answer rather than an error. This
rule is the reason the parser can be written with one glyph of lookahead and no backtracking.

**`*` never binds.** It is always exactly one slot meaning "any card". This is what makes
`AA**` unambiguously four slots rather than an argument over whether `A*` was one. A writer
who means a single wildcard and types `A*` gets two slots; when that makes the count wrong,
the error must say why.

| Written | Slots |
|---|---|
| `AhKh` | `Ah` `Kh` |
| `2c` | `2c` |
| `2 c` | `2` `c` |
| `cc` | `c` `c` |
| `AhKh2` | `Ah` `Kh` `2` |
| `AA**` | `A` `A` `*` `*` |

### 3.3 Ranges

`AKs`, `AKo`, `AK`, `22+`, `A2s+`, `JTs-67s` — recognised **only where a hand holds exactly
two cards**. In Omaha and stud the range grammar does not exist, which is what makes the
collision between `AKs` the range and `A` `Ks` the two slots impossible there rather than
merely unlikely.

Where it does exist, a term is a range only if it matches the range grammar *exactly*.

- `+` raises the lower card: `22+` is every pair; `A2s+` is `A2s`…`AKs`; `KTs+` is
  `KTs`/`KJs`/`KQs`.
- To walk a gap, name both ends: `JTs-67s`. Both ends must share a gap, and either may be
  written first.
- Percentage ranges (`top 15%`) are **out of scope** — they need a hand-strength ordering that
  is a product decision, not a parsing one. Refuse them with a clear message.

### 3.4 Alternatives and slots

A hand field is a **comma-separated list of alternatives**, each alternative a whole hand.
Within one alternative, whitespace separates slots. So `AKs, 22` is ten combinations, and
`2 c` is one two-card hand.

This gives one internal shape for the whole grammar:

```rust
/// One hand field: alternatives, each a per-slot mask.
pub struct HandSpec {
    pub alternatives: Vec<Vec<u64>>,
}
```

`AhKh` is one alternative of two single-bit masks; `AKs` is four such alternatives; `2 c` is
one alternative of two multi-bit masks. **The sampler never asks which kind it was handed.**

### 3.5 Boards and dead cards

- A board may be **short** — a flop is three of five — and may hold wildcards. That is how
  Courchevel and "what if the turn is a heart" are both expressed.
- **Dead cards must be exact.** "A club is dead" does not say *which* club and every reading
  changes the answer, so refuse it rather than guess.

### 3.6 Masks in, too

Everything above must also be constructible directly from masks, with no text involved:

```rust
HandSpec::from_slots(&[deuce_mask, club_mask]);
EquityRequest::from_masks(variant, hands, board, dead);
```

fpdb calls the mask API; anyone else can call either.

### 3.7 One grammar, two parsers

fpdb keeps its own Python parser, because a text field must validate as you type whether or
not the native library is installed. Two implementations of one grammar drift.

**Ship a conformance fixture** — a plain data file (JSON or TSV) of `input → canonical output`
and `input → error` pairs, checked into this repo and read by tests on both sides. It is the
grammar's only source of truth. Every rule in §3 needs a row.

---

## 4. Variants

Every game is **one row of data**, not a module. What separates hold'em from razz is how many
cards a player gets, whether there is a board, and which kernel reads the result.

```rust
pub struct VariantSpec {
    pub key: &'static str,        // matches fpdb's DB category
    pub label: &'static str,
    pub deck: u64,                // 52-bit, or 36-bit for short deck
    pub dealing: Dealing,         // Community { board: u8 } | Private { cards: u8 } | Draw
    pub hole_cards: u8,
    pub use_exactly: Option<u8>,  // Some(2) for the Omaha rule
    pub hi: Option<Kernel>,
    pub lo: Option<Kernel>,
    pub lo_qualifier: Option<u8>, // eight-or-better, as a rank index
    pub board_known_preflop: u8,  // Courchevel
    pub draws: u8,
}
```

Required rows, with fpdb's category keys:

| Key | Game | Hole | Board | Dealing | Hi | Lo |
|---|---|---|---|---|---|---|
| `holdem` | Hold'em | 2 | 5 | community | high | — |
| `6_holdem` | Short Deck | 2 | 5 | community | high-short | — |
| `omahahi` | Omaha | 4 | 5 | community | high (2 of 4) | — |
| `5_omahahi` | 5-Card Omaha | 5 | 5 | community | high (2 of 5) | — |
| `6_omahahi` | 6-Card Omaha | 6 | 5 | community | high (2 of 6) | — |
| `omahahilo` | Omaha Hi/Lo | 4 | 5 | community | high | A-5, 8-or-better |
| `5_omaha8` | 5-Card Omaha Hi/Lo | 5 | 5 | community | high | A-5, 8-or-better |
| `cour_hi` | Courchevel | 5 | 5 | community | high | — |
| `cour_hilo` | Courchevel Hi/Lo | 5 | 5 | community | high | A-5, 8-or-better |
| `studhi` | Seven-Card Stud | 7 | 0 | private | high | — |
| `studhilo` | Stud Hi/Lo | 7 | 0 | private | high | A-5, 8-or-better |
| `razz` | Razz | 7 | 0 | private | — | A-5, no qualifier |
| `27_3draw` | 2-7 Triple Draw | 5 | 0 | draw | — | 2-7 |
| `badugi` | Badugi | 4 | 0 | draw | — | badugi |

### Courchevel is not a variant

It is five-card Omaha where one board card is face up before the betting. Implement 5-card
Omaha and 5-card Omaha hi/lo, then Courchevel is that row plus `board_known_preflop: 1` —
a validation rule (at least one board card must be given) and a label. No new evaluation
code, no new sampling code.

### Short deck

PokerStars/GG ordering: 36 cards (sixes and up), **a flush beats a full house**, **trips beat
a straight**, and `A6789` is the low straight. Name the ruleset in the label, since rooms
differ.

### Draw games

Model a **single draw**: each player discards once and draws replacements from the remaining
deck. Equity in triple draw is undefined without a drawing strategy — a made eight-low and a
four-card draw are not comparable until you say how the draw resolves — so model one draw and
say so, rather than publishing a number derived from an invented model.

Which cards a player discards is itself an assumption. Default to a documented heuristic (keep
the best made hand or the best four-card draw), and let the caller override with an explicit
discard list.

---

## 5. Ranking kernels

**There are three, plus a re-parameterisation.** This is the single most important structural
claim in this document, because it is what makes the variant list tractable.

| Kernel | Definition | Serves |
|---|---|---|
| `High` | the five-card high hand | hold'em, Omaha, stud — and, **read upside down**, 2-7 lowball |
| `LowA5` | the same generator with the ace below the deuce, straights and flushes switched off, ordering inverted | razz bare; the hi/lo games with a qualifier |
| `Badugi` | largest rank-distinct *and* suit-distinct subset, then compare | badugi |
| `HighShort` | `High`'s generator over 36 cards with the categories permuted | short deck |

Two traps worth stating, because both are commonly got wrong:

- **2-7 lowball is not a rank-mask lookup.** It counts straights and flushes *against* you, so
  it must see suits. It is the high kernel with its ordering reversed and the ace forced high.
- **Razz is not a rank-mask lookup either.** Seven cards can hold only four distinct ranks
  (`2233445`), which razz ranks as a paired hand. A 13-bit rank-mask table cannot express that.

### The `use_exactly` rule

Omaha requires **exactly two** hole cards and three board cards. With 4/5/6 hole cards that is
`C(n,2) * C(5,3)` = 60/100/150 five-card evaluations per player per deal. This is the
dominant cost in Omaha and deserves its own optimisation pass; it is why Omaha currently runs
at 200 k hands/s against hold'em's 13 M.

### Hi/lo pot splitting

Split games award **fractional shares**, and quartering is the case that breaks naive
implementations: two players split the high half and one of them also takes the low, so that
player has 75% and the other 25% — neither "won" nor "tied" in any countable sense.

Compute shares directly. Never derive them from win/tie counts afterwards.

---

## 6. Lookup tables

The current `hand_rank_table.rs` is **78,501 lines** and `rank_translation.rs` is **22,926** —
together 94% of the source tree. They are generated data checked in as Rust source, and they
are consulted through `phf::Map`, which hashes on every evaluation.

### Requirements

1. **No generated table in the source tree.** Generate in `build.rs` into `OUT_DIR`, and
   `include!` or `include_bytes!` the result. The *generator* is what gets reviewed; a
   78k-line literal cannot be.
2. **Array indexing in the hot path, not hashing.** The flush lookup is a 13-bit rank mask —
   a dense `[u16; 8192]`, 16 KB, indexed directly. The non-flush lookup should be a dense
   table behind a computed index (a perfect hash over the rank-multiset key), not a
   `phf::Map` probe.
3. **Budget: ≤ 400 KB of tables total.** The 2+2 approach (a 130 MB `HandRanks.dat`) is
   obsolete — it wins on sequential access and loses by ~14× under the random access a Monte
   Carlo actually performs, because it thrashes cache. Sub-megabyte perfect-hash evaluators
   have won.
4. **Verified exhaustively.** A test enumerates all `C(52,5)` = 2,598,960 five-card hands and
   checks the table's ordering against a slow, obviously-correct reference evaluator. Same for
   the low, 2-7, badugi and short-deck kernels over their spaces. This is the gate; nothing
   about a lookup table is self-evident.

### Reference points

| Evaluator | Table size | Throughput |
|---|---|---|
| **this library today** | ~1 MB of `phf` | **13.2 M/s** (hold'em, 7-card) |
| OMPEval | 200 KB | 270–520 M/s random access |
| holdem-hand-evaluator | 212 KB | ~1.2 G/s sequential |
| 2+2 `HandRanks.dat` | 130 MB | 19 M/s random |

There is **20–40× headroom** in the evaluator before any other change.

---

## 7. Sampling — the part that is subtly wrong if rushed

### 7.1 Do not rebuild the deck per iteration

The per-simulation cost must be: mask out what is known once, then draw. Building a 52-card
`Vec`, removing known cards one at a time through a fallible call, and shuffling all 52 — per
iteration — dominates everything else.

Per deal, the loop should touch nothing but integers:

```
available = variant.deck & !dead & !board_fixed & !hole_fixed
draw k cards from `available`
evaluate
accumulate
```

Accumulate into a `Vec<f64>` indexed by seat. Never a `HashMap<String, _>` — hashing a player
name per simulation is real, measurable cost for no benefit.

### 7.2 The multiplicity bias

**This is the failure mode that testing usually misses, because the natural test compares
Monte Carlo to Monte Carlo and both are wrong in the same way.**

Sampling one card per slot independently and rejecting on collision is uniform over *ordered
assignments*. A hand is an **unordered set**, and when slot masks overlap only partially, sets
do not have equal multiplicity.

Concretely, with slot masks `M₁ = {A,B,C}` and `M₂ = {A,B}`: the set `{A,B}` is reachable two
ways (A→1,B→2 and B→1,A→2), while `{C,A}` and `{C,B}` are reachable one way each. Ordered
sampling over-weights `{A,B}` by 2×. The equities come out self-consistent, stable, and wrong.

**The correct sampler:** draw a random *k*-subset from the union of the slot masks, and accept
it only if a perfect matching exists between the drawn cards and the slots. Uniform over valid
sets by construction. With k ≤ 7 a greedy augmenting-path matching is nanoseconds.

Where every slot is unconstrained — board runouts, stud deals, most of the work — this reduces
to a plain draw with ~100% acceptance, so the general path costs nothing in the common case.

**Guard the rejection rate.** Measure acceptance over the first chunk; if it falls below ~1%,
switch to enumerating the constrained slots and sampling only the free ones. A sampler must
never spin unboundedly on a nearly-infeasible input.

### 7.3 Feasibility, up front

A sampler given constraints that no deal satisfies does not fail — it spins forever, and the
caller's UI hangs.

Check before sampling, and check with a **matching, not a count**. "Five deuces" is caught by
counting; "two slots that both admit only the ace of hearts" is not. Hall's condition over the
(slot → allowed cards) bipartite graph, decided by greedy augmenting paths, is microseconds on
the ≤30 slots a full table can produce.

Where a player has several alternatives the check must be **one-sided**: it may prove a
request impossible, and must never reject one that some combination of alternatives would
satisfy.

### 7.4 Exact enumeration

When the sample space is small enough — a river spot, a heads-up turn, most all-in
confrontations — **enumerate it**. Exact results need no error bar, and reporting a confidence
interval on a deterministic answer is a bug.

Exact mode is also the only test that catches sampler bias (§9), so it must exist for hold'em
from the beginning, not arrive with the last variant.

Pick a threshold on the number of deals (a few million) and switch automatically, with an
override.

---

## 8. The equity API

Two layers. The lower one is what fpdb drives; the upper one is what makes the library
pleasant on its own.

### 8.1 A stateless chunk function

```rust
pub fn run_chunk(req: &EquityRequest, samples: u64, seed: u64) -> Result<ChunkResult, Error>;
```

Pure, stateless, no callbacks, no cancellation token, no interior mutability. The caller loops
and decides when to stop.

This is deliberate. fpdb drives it from Python: chunk, merge, repaint the table, check whether
the user cancelled, repeat. Cancellation is then instant at chunk granularity with no atomic
threaded through the loop and nothing calling back into Python from a worker thread. A 250k
chunk is 5–20 ms, which is a fine cancellation granularity and a good repaint rate.

```rust
pub struct ChunkResult {
    pub samples: u64,
    pub share_sum: Vec<f64>,        // fraction of pot, summed
    pub share_square_sum: Vec<f64>, // for the standard error
    pub win_count: Vec<u64>,        // taken outright
    pub tie_count: Vec<u64>,        // shared in any way
    pub scoop_count: Vec<u64>,      // both halves
    pub low_share_sum: Vec<f64>,    // the low half alone
    pub exact: bool,
}
```

**Sums, not averages**, so chunks merge by addition. **Sum of squares**, so the confidence
interval costs nothing — without it a user reads a 0.3% gap between two hands as meaningful.

### 8.2 A convenience runner

```rust
pub fn equity(req: &EquityRequest, target: Target, on_progress: impl Fn(&Progress)) -> Result<EquityResult, Error>;

pub enum Target {
    Samples(u64),
    StandardError(f64),  // run until the interval is this tight
    Exact,
}
```

Internally: rayon across chunks, progress callback between them. Thread count configurable and
defaulting to something polite (`min(4, available_parallelism)`) — several calculators may be
open at once, and one must not starve the others.

### 8.3 Results

```rust
pub struct PlayerEquity {
    pub equity: f64,      // share of the pot. The number that matters.
    pub win: f64,
    pub tie: f64,
    pub low_equity: f64,
    pub scoop: f64,
    pub std_error: f64,   // zero when exact
}
```

`equity` leads because it is the answer. Wins and ties are colour.

### 8.4 Errors

A typed error enum, and **parse errors must carry a byte offset and length** so a caller can
underline the offending span instead of showing a dialog that loses the position. This is the
difference between a syntax people can use and one they cannot.

Distinguish at minimum: unparseable text (with span), a card named twice (which card, which
two places), constraints that admit no deal (which field), and a card not in this game's deck.

---

## 9. Testing

Four independent layers. No one of them is sufficient, and the ordering matters — the first
two prove internal consistency, only the third catches a self-consistent implementation of the
wrong game.

1. **Exhaustive kernel verification.** All 2,598,960 five-card hands, ordering checked against
   a slow reference evaluator. Full sweep behind `#[ignore]` or a feature; a fixed-seed sample
   in the default run. Repeat per kernel over its space.
2. **Exact versus Monte Carlo**, agreeing within four standard errors. **This is the only test
   that catches the multiplicity bias in §7.2**, which is why exact mode cannot wait.
3. **Published reference equities**, asserted in *exact* mode to ±0.05%. AA vs KK preflop is
   81.95 / 17.09 / 0.96. Include AKs vs 22, AA vs 78s, and one canonical figure per variant
   family.
4. **Properties.** Equities sum to 100%. Permuting players permutes equities. A hand against
   itself is 50/50. An irrelevant dead card moves nothing beyond noise. A wildcard admitting
   exactly one card equals naming that card — *this one directly tests the wildcard path
   against the concrete path.*

Plus: the §3.7 conformance fixture for the grammar, and a benchmark binary reporting
samples/s per variant. Benchmarks must **not** be assertions in CI — perf gates are flaky. One
exception: a very loose floor (say >1 M hold'em evals/s) that exists solely to catch a debug
build being shipped.

---

## 10. Python binding

Not yet built, but the shape constrains the crate layout now.

- A plain `--lib` crate. The PyO3 wrapper lives in **one module** (`src/py.rs`) behind a
  `python` feature, so the core never depends on Python and a future standalone binary or a
  C ABI is additive rather than a rewrite.
- `abi3-py314`: one wheel per platform covering every future CPython. (Free-threaded builds
  are a separate wheel and are explicitly out of scope.)
- The boundary carries `u64` masks and `u8` card indices. Text may cross it for the
  convenience API, but the mask API is what fpdb uses.
- Release the GIL around the sampling loop.
- **Do not publish** until the build is solid: no crates.io, no PyPI, no CI wheel matrix yet.

---

## 11. Order of work

Each step should leave the library working and testable.

1. **Card and mask core.** `u8` indices, `u64` masks, the deck constants. Everything else
   rests on this, and it is the one thing that must not be revised later.
2. **The notation parser**, to §3, with the conformance fixture.
3. **The high kernel**, tables generated in `build.rs`, verified exhaustively against a
   reference evaluator. This is where the 20–40× is.
4. **The sampler**: masks, the unbiased *k*-subset draw, feasibility, and `run_chunk`.
   Hold'em end to end. Exact enumeration in the same step.
5. **The variant table** and the community games: Omaha 4/5/6 with `use_exactly`, then
   Courchevel as a validation rule on 5-card Omaha.
6. **The low kernel**: A-5 with and without the qualifier, fractional pot splitting, and the
   private dealing model — Omaha hi/lo, stud, stud hi/lo, razz.
7. **2-7 and badugi** (single draw), and short deck.
8. **The PyO3 layer.**

---

## 12. What is worth keeping

The current tree has already paid for several answers:

- **The variant test suites.** `src/variants/tests/` and `src/odds/tests/` encode real
  knowledge about what each game's rules actually are. They are worth more than the code they
  test and should survive any rewrite.
- **The base-5 rank-multiset key** (`RANK_KEYS` as powers of five) is a sound way to encode a
  rank multiset uniquely. The problem is `phf::Map`, not the key.
- **The flush-key idea** — a 13-bit rank mask per suit — is right, and wants to be a dense
  array.
- **The rayon chunking** in `EquityCalculator::calculate` is the right shape. What it does
  *inside* each iteration is the problem.
- **`Card::from_str`** already handles whitespace-insensitive parsing of concrete cards; the
  grammar in §3 is a superset of it.

The things that should not survive: `Vec<Card>` in the inner loop, `Deck::new()` per
simulation, `HashMap<String, f64>` keyed by player name per simulation, generated tables as
checked-in source, and equity reported without an error bar.
