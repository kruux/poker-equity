# poker-calculator

An equity engine for poker. Give it a game, some hands, a board and some dead
cards, and it tells you what share of the pot each player wins.

Hands may be only partly known — `AKs`, `2 c`, `A**` are all valid — and the
answer comes back with an error bar, or with none at all when the spot was
small enough to enumerate rather than sample.

Fourteen games, from hold'em to badugi. Every hand ranking is checked
exhaustively against an independent implementation; see
[Correctness](#correctness).

```rust
use poker_calculator::{odds::{equity, EquityRequest, Target}, variants::Holdem};

let request = EquityRequest::from_text(
    Holdem,
    &["AhAd", "KsKc"],   // one field per seat
    "2c 7d 9h",          // the board so far
    "",                  // dead cards
)?;

let result = equity(&request, Target::Exact)?;
for (seat, player) in result.equities().iter().enumerate() {
    println!("seat {}: {:.2}%", seat, player.percent());
}
// seat 0: 91.62%
// seat 1: 8.38%
```

Only 990 run-outs are possible there, so `Target::Exact` walks all of them and
the standard error comes back as exactly zero.

## Writing hands

A card is a rank and a suit: uppercase rank, lowercase suit on the way out,
but anything on the way in. Beyond that, four patterns:

| Written | Means |
|---|---|
| `Ah` | the ace of hearts |
| `A` | any ace |
| `c` | any club |
| `*` | any card |

**A rank binds to a suit immediately after it, and nothing else binds.** So
`2c` is one card and `2 c` is two — a deuce and a club. Both are valid and
they differ by a whole card, so the wrong reading gives a plausible wrong
answer rather than an error, which is why the rule is worth knowing.

`*` never binds, which is what makes `AA**` unambiguously four cards. If that
makes the count wrong, the error says so.

```rust
"AhKh"      // two named cards
"A Kh"      // any ace, and the king of hearts
"2 c"       // a deuce and a club — two cards
"2c"        // the deuce of clubs — one card
"AA**"      // two aces and two unknowns (Omaha)
```

### Ranges

Where a hand holds exactly two cards, a field may be a range instead:

```rust
"AKs"       // suited ace-king, four combinations
"AKo"       // offsuit, twelve
"AK"        // either, sixteen
"22+"       // every pair
"A2s+"      // A2s through AKs
"KTs+"      // KTs, KJs, KQs
"JTs-76s"   // the suited connectors between them
"AKs, 22"   // alternatives, comma separated — ten combinations
```

Ranges exist only where a hand is two cards. In Omaha and stud there is no
range grammar at all, which is what makes the collision between `AKs` the
range and `A` `Ks` the two cards impossible there rather than merely unlikely.

Percentage ranges (`top 15%`) are refused: they need a hand-strength ordering
that is a product decision, not a parsing one.

### Boards, dead cards, and hands still being dealt

A board may be short and may hold wildcards — that is how "what if the turn is
a heart" is asked. Dead cards must be exact, because "a club is dead" does not
say *which* club and every reading changes the answer.

In stud and the draw games, cards arrive over time, so a field says what a
player holds **now** and whatever is missing is still to come:

```rust
EquityRequest::from_text(SevenCardStud, &["Ah2c3d", "QsQdJs"], "", "")?;
//                        three known, four still to be dealt

EquityRequest::from_text(DeuceSeven, &["7h5c4d3s", "9h8c6d5h2c"], "", "Kd")?;
//                        Hero draws one; the king he threw is dead. Villain stands pat.
```

Community games want every hole card, since they are all dealt at once — a
short field there is a miscount, and an unknown card is a wildcard.

## Getting an answer

The simple way. Say how good an answer you want, and get one:

```rust
use poker_calculator::odds::{equity, Target};

// half a million deals
let result = equity(&request, Target::Samples(500_000))?;

for (seat, player) in result.equities().iter().enumerate() {
    println!("seat {}: {:.2}% ± {:.2}", seat, player.percent(), player.margin_percent());
}
```

| Target | Runs until |
|---|---|
| `Target::Samples(500_000)` | half a million deals are done |
| `Target::StandardError(0.001)` | the answer is that precise — "make this good enough to trust" |
| `Target::Exact` | every possible deal has been walked, falling back to tight sampling when there are too many |

`StandardError` is usually the one you want. A sample count is a guess at how
long precision takes; a precision target just asks for the precision.

If a spot has fewer possible deals than you asked to sample, it is walked
instead of sampled — `AhAd` against `KsKc` on a flop has only 990 run-outs, so
asking for half a million deals gets all 990 and a `std_error` of exactly
zero. You are never given more work than you asked for: a spot with two
million possible deals is sampled, not walked, when you asked for five
hundred thousand.

`StandardError` and `Exact` put no cap on the work, so they walk whatever can
be walked — an exact answer beats any error bar.

### Watching a long run

A run of millions of deals takes long enough that you will want to repaint a
table while it goes, and to notice if the user cancelled. Pass a function to
be called after each batch:

```rust
use poker_calculator::odds::equity_with_progress;

let result = equity_with_progress(&request, Target::Samples(5_000_000), |progress| {
    println!("{} deals: {:.2}% (± {:.2}), keeping {:.0}% of deals",
        progress.samples,
        progress.equities[0].percent(),
        progress.equities[0].margin_percent(),
        progress.acceptance * 100.0);
})?;
```

`|progress| { ... }` is Rust's syntax for a function written inline — the
names between the bars are its arguments, and the braces are its body. So
that one takes a `Progress` and prints from it. A batch is a few milliseconds,
which is a good rate to repaint at and a fine granularity to cancel at.

`progress.acceptance` is the share of attempted deals that could be used. It
is 1.0 unless hands are competing for the same cards — several seats all
wanting a five when only two are left — and a low figure is the difference
between an answer that is slow and one that looks stuck.

### Driving the loop yourself

If you want to own the loop — because cancelling, or threading, or merging
results across machines is your business rather than the library's — the layer
underneath is a single batch that returns sums:

```rust
use poker_calculator::odds::{run_chunk, ChunkResult};

let mut total = ChunkResult::empty(2);
for seed in 0..10 {
    total.merge(&run_chunk(&request, 50_000, seed)?);
    if user_cancelled() { break; }
}
// 500,000 deals, or fewer if the user stopped it
```

`run_chunk` is pure and stateless: no callbacks, no cancellation token, no
shared state. Everything in `ChunkResult` is a **sum**, so batches merge by
addition, and the sums of squares are carried too — which is what makes the
error bar free rather than something to compute separately.

### What comes back

```rust
pub struct PlayerEquity {
    pub equity: f64,      // share of the pot — the number that matters
    pub win: f64,         // taken outright
    pub tie: f64,         // shared
    pub low_equity: f64,  // the low half alone, in split games
    pub scoop: f64,       // both halves
    pub std_error: f64,   // zero when the answer was enumerated
}
```

Equity leads because it is the answer — it is what the money does over time.
Wins and ties are colour. `percent()` and `margin_percent()` give the two
numbers a table usually shows: the equity, and the half-width of a 95%
interval around it.

## The mask API

Text is a convenience. Underneath, a card is a `u8` and a set of cards is a
`u64` with one bit per card, and that is what the engine speaks. A caller that
already has masks — fpdb does — can skip the parser entirely:

```rust
use poker_calculator::{cards::{Card, CardSet, Rank, Suit}, notation::HandSpec,
                       odds::EquityRequest, variants::Holdem};

assert_eq!(Card::new(Suit::Heart, Rank::Ace).index(), 50);   // rank * 4 + suit

let queen_of_spades = CardSet::from_cards(&Card::from_str("Qs")?);

let request = EquityRequest::from_masks(
    Holdem,
    &[
        // any ace and any club
        HandSpec::from_slots(&[CardSet::of_rank(Rank::Ace), CardSet::of_suit(Suit::Club)]),
        // the queen of spades and anything
        HandSpec::from_slots(&[queen_of_spades, CardSet::FULL_DECK]),
    ],
    &[],                    // no board yet
    CardSet::EMPTY,         // nothing dead
)?;
```

A named card is a mask with one bit; "any ace" is a mask with four; "any card"
is all fifty-two. **The sampler never asks which kind it was handed**, which is
why a wildcard costs nothing on the fast path.

The layout is worth knowing if you are building masks yourself: index is
`rank * 4 + suit`, ranks `0..13` running `23456789TJQKA` and suits `0..4`
running `cdhs`. So a rank is four adjacent bits and a suit is every fourth bit:

```
of_rank(Ace)  = 0b1111 << 48
of_suit(Club) = 0x1_1111_1111_1111
FULL_DECK     = (1 << 52) - 1
```

Both APIs are the same request — `from_text` parses into exactly what
`from_masks` takes, once, before any card is dealt. A test runs both from the
same seed and compares the raw sums.

## Games

| Key | Game | Hole | Board |
|---|---|---|---|
| `holdem` | Hold'em | 2 | 5 |
| `short_deck` | Short deck — 36 cards, a flush beats a full house | 2 | 5 |
| `omaha` | Omaha | 4 | 5 |
| `omaha_five` | Five-card Omaha | 5 | 5 |
| `omaha_six` | Six-card Omaha | 6 | 5 |
| `omaha_hi_lo` | Omaha hi/lo, eight or better | 4 | 5 |
| `omaha_five_hi_lo` | Five-card Omaha hi/lo | 5 | 5 |
| `courchevel` | Courchevel | 5 | 5 |
| `courchevel_hi_lo` | Courchevel hi/lo | 5 | 5 |
| `stud` | Seven-card stud | 7 | — |
| `stud_hi_lo` | Stud hi/lo | 7 | — |
| `razz` | Razz | 7 | — |
| `deuce_seven` | 2-7 lowball, single draw | 5 | — |
| `badugi` | Badugi, single draw | 4 | — |

Courchevel is not a variant of its own: it is five-card Omaha with the first
board card face up before the betting, so it is the same evaluation plus one
rule about what a legal board looks like.

Both draw games model **one** draw. Equity in triple draw is undefined without
a drawing strategy — a made eight-low and a four-card draw are not comparable
until you say how the draw resolves — so one draw is modelled and said so,
rather than a number published from an invented model.

## Speed

Deals a second on one core, including dealing, evaluation and splitting the
pot. Run `cargo run --release --bin benchmark` for your own machine.

| Game | Deals/s | | Game | Deals/s |
|---|--:|---|---|--:|
| Hold'em | 3,707,232 | | Omaha | 287,869 |
| Short deck | 3,983,484 | | Five-card Omaha | 175,836 |
| Stud | 2,833,198 | | Six-card Omaha | 119,820 |
| Stud hi/lo | 2,461,528 | | Omaha hi/lo | 155,637 |
| Razz | 2,900,203 | | Badugi | 851,080 |
| 2-7 single draw | 5,365,421 | | Hold'em, six seats | 1,746,337 |

Hold'em across sixteen cores runs at **33.2 million** deals a second.

The Omaha family is an order of magnitude behind the rest, and unavoidably:
its rule is that exactly two hole cards play with exactly three of the board,
which is 60 five-card evaluations per player per deal at four hole cards, 100
at five and 150 at six.

Underneath, a hand is scored by two array reads:

| | Hands/s |
|---|--:|
| `score` — reads the lookup tables | 95,691,138 |
| `evaluate_hand` — names the hand | 1,863,222 |

Both are kept. `score` is what the sampling loop compares; `evaluate_hand` is
what tells a player they have two pair, and is the independent second
implementation the tables are checked against.

The tables total 1,104 KB — four rankings sharing one perfect hash — and are
generated by `build.rs` into `OUT_DIR`. Nothing generated is committed.

## Correctness

Hands being ranked correctly matters more than any of the above, so it is
checked three ways.

**Against a reference evaluator, exhaustively.** Every lookup table is swept
against the readable evaluator beside it — all 2,598,960 five-card hands per
ranking, plus seven-card hands. `cargo test --release -- --ignored`.

**Against an outside implementation, exhaustively.** The two above were both
written here, from one reading of the rules, so they catch a mistake in either
but not a rule misunderstood in both. `validation/` settles that against
[pokerkit](https://github.com/uoftcprg/pokerkit): every five-card hand of every
ranking, every four-card badugi holding, and sampled seven-card and Omaha
hands. All agree, and the number of distinct hand classes matches exactly on
both sides — the two libraries cut the hands the same way, not merely
compatibly. See `validation/README.md`.

**Against arithmetic.** Sampling is checked against exact enumeration within
four standard errors, which is the only test that catches a sampler that is
uniform over the wrong thing. Equities sum to one; permuting seats permutes
the answer; a hand against its own mirror splits exactly; a wildcard narrowed
to a single card equals naming that card.

## Python

```sh
cargo build --release --features python
cp target/release/libpoker_calculator.so somewhere/poker_calculator.so
```

```python
import poker_calculator as pc

r = pc.exact_from_text("holdem", ["AhAd", "KsKc"], "2c 7d 9h")
print(r["players"][0]["equity"])        # 0.916161...
print(r["exact"], r["samples"])         # True 990

# or masks, which is what fpdb passes
hands = [[[1 << pc.card_index("Ah"), 1 << pc.card_index("Kh")]],
         [[1 << pc.card_index("Qs"), 1 << pc.card_index("Qd")]]]
r = pc.chunk("holdem", hands, [], 0, 200_000, seed=3)
```

`abi3-py314`, so one wheel per platform covers every future CPython. The GIL
is released around sampling. `pc.variants()` lists every game with how it
deals, so a caller needs no table of its own, and `pc.parse_hand_field` hands
back masks so another parser can be checked against this one.

Nothing is published to crates.io or PyPI yet.

## Building

```sh
cargo test                              # 222 tests, about 20 seconds
cargo test --release -- --ignored       # the exhaustive sweeps
cargo run --release --bin benchmark     # speed, per game
```

Tests build at `opt-level = 2`, because the equity tests run a hundred
thousand Monte Carlo deals apiece and that is minutes unoptimised against
seconds optimised.

`tests/fixtures/notation.tsv` is the grammar's source of truth — a plain table
of input and expected output, with a row for every rule above. fpdb's own
parser reads the same file, so a change that lands on only one side shows up
as a failing test rather than as a disagreement in the field.
