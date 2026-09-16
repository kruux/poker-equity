# poker-equity

An equity engine for poker. Give it a game, some hands, a board and some dead
cards, and it tells you what share of the pot each player wins.

**Beta, and written almost entirely by AI.** This is 0.1 and should be read
as one. The API is not settled: names, signatures and the notation may all
change before 1.0, and a few corners of the behaviour are still being decided.
Pin an exact version rather than a range. The hand rankings are cross-checked
against [pokerkit](https://github.com/uoftcprg/pokerkit) for the games whose
hand spaces are small enough to check — see [Correctness](#correctness) — but
nothing here has been used in anger yet.

## Install

Nothing is published to crates.io or PyPI yet, so both sides come from a
clone.

```sh
git clone https://github.com/kruux/poker-equity && cd poker-equity
```

**Rust.** Point a dependency at the clone, or at the repository:

```toml
poker-equity = { path = "../poker-equity" }
# or        = { git = "https://github.com/kruux/poker-equity" }
```

Then a whole program is:

```rust
use poker_equity::{odds::{equity, EquityRequest, Target}, variants::Holdem};

fn main() -> Result<(), poker_equity::error::PokerError> {
    let request = EquityRequest::from_text(Holdem, &["AhKh", "QsQd"], "", "")?;
    let result = equity(&request, Target::Samples(500_000))?;
    println!("{:.2}%", result.equities()[0].percent());   // about 46.2%
    Ok(())
}
```

**Python.** The binding is real and works — it is the wheel that is missing, so
the module gets built from the clone and put where the interpreter looks. Any
CPython from 3.10 up, since the extension is `abi3-py310`:

```sh
cargo build --release --features python
```

Cargo leaves it in `target/release` under a name Python will not import, so the
copy is what matters. It must be called `poker_equity`, and the extension
Python wants is not always the one Cargo wrote:

| Platform | Cargo writes | Copy it to |
|---|---|---|
| Linux | `libpoker_equity.so` | `poker_equity.so` |
| macOS | `libpoker_equity.dylib` | `poker_equity.so` |
| Windows | `poker_equity.dll` | `poker_equity.pyd` |

macOS is the one that catches people out: CPython loads extension modules named
`.so` there too, and ignores a `.dylib`. Its linker also wants a flag before it
will accept the module, which `.cargo/config.toml` passes. Then, on Linux:

```sh
mkdir -p ~/lib
cp target/release/libpoker_equity.so ~/lib/poker_equity.so
export PYTHONPATH=~/lib
python3 -c "import poker_equity; print(len(poker_equity.variants()))"   # 14
```

Copying it into a virtualenv's `site-packages`, or into the directory you run
from, works just as well — `PYTHONPATH` is only the least invasive of them.

```python
import poker_equity as pc

r = pc.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=500_000)
for seat in r["players"]:
    print(f"{seat['equity']:.4f} +/- {seat['std_error']:.4f}")
```

More of it in [Python](#python).

## Games

Showdowns a second, per thread: dealing, evaluating and splitting the pot,
end to end. Measured heads-up on one core of an AMD Ryzen 7 9800X3D. Run
`cargo run --release --bin benchmark` for your own machine.

| Key | Game | Per thread |
|---|---|--:|
| `holdem` | Hold'em | 6.86 M/s |
| `short_deck` | Short Deck Hold'em — 36 cards, a flush beats a full house | 7.14 M/s |
| `omaha` | Omaha | 2.86 M/s |
| `omaha_five` | 5-Card Omaha | 2.15 M/s |
| `omaha_six` | 6-Card Omaha | 1.67 M/s |
| `omaha_hi_lo` | Omaha Hi/Lo, eight or better | 1.77 M/s |
| `omaha_five_hi_lo` | 5-Card Omaha Hi/Lo | 1.29 M/s |
| `courchevel` | Courchevel | 2.26 M/s |
| `courchevel_hi_lo` | Courchevel Hi/Lo | 1.33 M/s |
| `stud` | Seven-Card Stud | 4.52 M/s |
| `stud_hi_lo` | Seven-Card Stud Hi/Lo | 4.12 M/s |
| `razz` | Razz | 4.84 M/s |
| `deuce_seven` | 2-7 Lowball, single draw | 10.55 M/s |
| `badugi` | Badugi, single draw | 5.35 M/s |

Threads share nothing while they sample, so multiply by however many you give
it. More seats cost more: six-handed hold'em runs at 3.51 M/s a thread, and
six-handed Omaha at 1.31 M/s.

A run spreads over four threads unless told otherwise, which is polite rather
than greedy: several calculators may be open at once and one must not starve
the others. `EquityRequest::with_threads` takes as many as you want to give
it, and nothing needs passing to get the default.

Both draw games model **one** draw. Equity in triple draw is undefined without
a drawing strategy — a made eight-low and a four-card draw are not comparable
until you say how the draw resolves — so one draw is modelled and said so,
rather than a number published from an invented model.

## Examples

Two games, to show that the shape of the question does not change with the
game being asked about.

### Hold'em: ace-king suited against a pair of queens

```rust
use poker_equity::{odds::{equity, EquityRequest, Target}, variants::Holdem};

let hero = "AhKh";        // one field per seat
let villain = "QsQd";
let board = "";           // nothing dealt yet
let dead = "";            // no cards out of play

let request = EquityRequest::from_text(Holdem, &[hero, villain], board, dead)?;
let result = equity(&request, Target::Samples(500_000))?;

for (seat, player) in result.equities().iter().enumerate() {
    println!("seat {}: {:.2}% ± {:.2}", seat, player.percent(), player.margin_percent());
}
// seat 0: 46.20% ± 0.13
// seat 1: 53.80% ± 0.13
```

Seats come back in the order they were passed in, so seat 0 is `hero`.

### 2-7 draw: a pat nine against a one-card draw

Change the variant, and the fields mean what that game means by them:

```rust
use poker_equity::{odds::{equity, EquityRequest, Target}, variants::DeuceSeven};

let villain = "9s7d5c4h2s";   // five cards: a made nine-low, standing pat
let hero = "8h6d4s3c";        // four cards, so one is still to come
let board = "";               // a draw game never has a board
let dead = "Kc";              // the king hero threw away

let request = EquityRequest::from_text(DeuceSeven, &[villain, hero], board, dead)?;
let result = equity(&request, Target::Samples(1_000_000))?;
// seat 0 (villain): 78.57% ± 0.00
// seat 1 (hero):    21.43% ± 0.00
```

Three things differ from the hold'em example, and every one of them is the
game rather than the API.

**A draw or stud field may be short.** Five cards is a hand standing pat; four
is a hand drawing one, and the card still to come is dealt. A community game
may *not* do this. Hold'em deals both hole cards at once, so a field holding
one card is a miscount rather than a hand in progress, and is an error — write
`A*` there if you mean one known card and one unknown.

**Discards are dead cards.** A card thrown away is out of the deck but was
seen, so it belongs in the dead field rather than nowhere.

**There is no error bar**, though a million deals were asked for. Only
forty-two cards can arrive, so the whole question is forty-two deals wide and
was walked rather than sampled: `result.samples` comes back as 42. Nine of
those cards — the deuces, fives and sevens still in the deck — give hero a
better low than a nine, and 9/42 is 21.43% exactly.

Hands may be only partly known — `AKs`, `2 c`, `A**` are all valid — and the
answer comes with an error bar, or none at all when the spot was small enough
to walk rather than sample. To follow a long run while it goes, see
[Watching a long run](#watching-a-long-run). For how the hand rankings are
checked, see [Correctness](#correctness).

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

### What a wildcard costs

A card you name exactly gets dealt. A card you only describe -- `A`, `c`, `*`
-- has to be *found*: the engine deals, checks the cards fit every seat, and
deals again if they do not. The answer is the same either way; only the time changes.

It is worst in stud, where each player holds seven private cards -- four-handed
stud claims twenty-eight of the fifty-two before anything is scored.

| Spot | Deals tried per simulation |
|---|---|
| `AA` vs `KK`, hold'em | 1 |
| `AA**` vs `KK**`, Omaha | 1.2 |
| `A23` vs `456`, stud | 2.9 |
| two `AA**` at a six-handed Omaha table | 6.5 |
| `A23` any number of ways, razz | 1 |

**Where searching stops working** -- below one kept deal in twenty -- the
engine deals each seat from what is left instead, and weights the deal to undo
the bias that introduces. Same answer, sooner, and spots that were refused
outright now answer. Read `effective_samples` there rather than `acceptance`:
it is what the error bar is divided by, and `is_weighted` says which applies.
Such an average is consistent rather than exactly unbiased, by a `1/n` term
under the bar's `1/√n`.

**Razz needs no suits.** It has no flushes and never looks at a suit, so `A23`
and `Ah2h3h` ask the same question -- and the engine pins the suits for you
when you leave them off. Razz runs at one deal per simulation however many
players are in the hand, so writing suits there buys nothing.

Everywhere else, writing the suits you actually mean is still the cheapest
thing you can do. A real hand has suits; naming them is both more faithful to
the spot and quicker to answer than either way of dealing it.

The underlying problem, if you want to read about it, is sampling a uniformly
random bipartite matching -- one deal that satisfies every seat at once, drawn
without favouring any of them.

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

Percentage ranges (`top 15%`) are not supported. There is no one ordering of
starting hands by strength — it changes with the game, the table size and
whose chart you trust — so naming the hands is the honest way to ask.

### Boards, dead cards, and hands still being dealt

A board is **how much of it you know**, not which street you are on. Nought to
five cards, wildcards allowed anywhere among them:

```rust
""              // nothing showing
"2c 7d 9h"      // a flop
"2c 7d 9h * Ks" // flop and river known, turn not
"2c 7d 9h Ts 4c"// finished: who won?
```

So "what if the turn is a heart" is `"2c 7d 9h h"`, and a finished board is a
fair question with an exact answer. Courchevel is the one game with a floor:
its first board card is face up before the betting, so a Courchevel request
showing nothing is refused — that spot is five-card Omaha, not Courchevel.

Write `"*"` there if the card has been dealt and you have not seen it. That
is a fair question, and it is how to ask what the turned card was worth: run
the spot with the card named, run it again with `*`, and the gap between them
is the answer. It comes back as the five-card Omaha number, because that is
what Courchevel with an unknown first card is.

Dead cards must be exact, because "a club is dead" does not say *which* club
and every reading changes the answer. They must also be cards the game
actually deals — a deuce is not dead in short deck, it is a mistake about the
game, and is refused in the dead field exactly as it would be in a hand.

How many cards a field may name is three rules, not one, because the games
deal differently:

| | A short field | Why |
|---|---|---|
| Community | refused | both hole cards arrive at once |
| Stud | allowed, **equally for every seat** | the whole table is on the same street |
| Draw | allowed, seat by seat | a short field *is* the draw |

Stud is the one worth care. Third street is three cards for everybody, so
`A23` against `2345` is not a table caught mid-deal, it is a miscount, and it
is refused. A draw game is the opposite: five against four is a pat hand
against a one-card draw, and saying so is the point of the notation.

In stud and the draw games, cards arrive over time, so a field says what a
player holds **now** and whatever is missing is still to come:

```rust
EquityRequest::from_text(SevenCardStud, &["Ah2c3d", "QsQdJs"], "", "")?;
//                        three known, four still to be dealt

EquityRequest::from_text(DeuceSeven, &["7h5c4d3s", "9h8c6d5h2c"], "", "Kd")?;
//                        Hero draws one; the king he threw is dead. Villain stands pat.
```

A community game wants every hole card, because they are all dealt at once:
there is no moment at which a hold'em player holds one card. So a short field
there is a miscount and is rejected. To say "an ace and something I cannot
see", write the unknown card as a wildcard — `A*`, not `A`.

## Getting an answer

`equity` takes a target, which is how you say what "good enough" means:

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
use poker_equity::odds::equity_with_progress;

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

`progress.effective_samples` is what the run is worth so far — equal to
`progress.samples` unless the deals were weighted, and what the error bar is
divided by.

### Driving the loop yourself

If you want to own the loop — because cancelling, or threading, or merging
results across machines is your business rather than the library's — the layer
underneath is a single batch that returns sums:

```rust
use poker_equity::odds::{run_chunk, ChunkResult};

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
error bar free rather than something to compute separately. The deals' weights
are summed the same way, so `effective_samples` merges as cleanly as the rest.

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
already holds masks can skip the parser entirely:

```rust
use poker_equity::{cards::{Card, CardSet, Rank, Suit}, notation::HandSpec,
                       odds::EquityRequest, variants::Holdem};

assert_eq!(Card::new(Suit::Heart, Rank::Ace).index(), 50);   // rank * 4 + suit

let queen_of_spades = CardSet::from_cards(&["Qs".parse::<Card>()?]);

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

## Speed

The per-game figures are in [Games](#games) above.

Underneath, a hand is scored by two array reads:

| | Hands/s |
|---|--:|
| `score` — reads the lookup tables | 99,726,346 |
| `evaluate_hand` — names the hand | 1,956,053 |

Both are kept. `score` is what the sampling loop compares; `evaluate_hand` is
what tells a player they have two pair, and is the independent second
implementation the tables are checked against.

The tables total 1,104 KB — four rankings sharing one perfect hash — and are
generated by `build.rs` into `OUT_DIR`. Nothing generated is committed.

## Correctness

Hand ranking is checked three ways. None of it stands in for use: the library
is new and has not yet been run in earnest by anyone.

**Against a reference evaluator, exhaustively.** Every lookup table is swept
against the readable evaluator beside it — all 2,598,960 five-card hands per
ranking, plus seven-card hands. `cargo test --release -- --ignored`.

**Against an outside implementation.** The two above were both written here,
from one reading of the rules, so they catch a mistake in either but not a rule
misunderstood in both. `validation/` checks the rankings against
[pokerkit](https://github.com/uoftcprg/pokerkit), as far as each game allows:
every five-card hand of every ranking and every four-card badugi holding are
walked exhaustively, while seven-card hands are sampled and Omaha — 4.6 ×
10¹¹ deals — cannot be walked at all. Everything checked agrees. See
`validation/README.md`.

**Against arithmetic.** Sampling is checked against exact enumeration within
four standard errors, which is the only test that catches a sampler that is
uniform over the wrong thing. Equities sum to one; permuting seats permutes
the answer; a hand against its own mirror splits exactly; a wildcard narrowed
to a single card equals naming that card.

Weighted dealing needs this most, since a wrong weight moves the answer
without moving anything a reader would notice. Where a spot is too large to
walk it is checked by symmetry: seats asking for the same thing must be given
the same equity.

## Python

Four ways in — two that parse the notation, two that take masks. Building the
module is under [Install](#install).

| Call | Does |
|---|---|
| `chunk_from_text(variant, hands, board="", dead="", samples=100_000, seed=0, threads=0)` | samples that many deals |
| `exact_from_text(variant, hands, board="", dead="")` | walks every deal, or returns `None` when there are too many |
| `chunk(variant, hands, board, dead, samples, seed=0, threads=0)` | the same by mask |
| `exact(variant, hands, board, dead)` | likewise |

```python
import poker_equity as pc

r = pc.exact_from_text("holdem", ["AhAd", "KsKc"], "2c 7d 9h")
print(r["players"][0]["equity"])        # 0.916161...
print(r["exact"], r["samples"])         # True 990

# or masks, skipping the parser: hands[seat][alternative][slot]
hands = [[[1 << pc.card_index("Ah"), 1 << pc.card_index("Kh")]],
         [[1 << pc.card_index("Qs"), 1 << pc.card_index("Qd")]]]
r = pc.chunk("holdem", hands, [], 0, 200_000, seed=3)
```

Every seat in `r["players"]` carries `equity`, `win`, `tie`, `low_equity`,
`scoop` and `std_error`. Beside them sit the run's own totals — `samples`,
`weight_sum`, `share_sum`, `acceptance`, `effective_samples`, `weighted` —
which are sums rather than averages, so calling again with another seed and
adding them is how you sample further. There is no precision target here as
there is in Rust: ask for a count.

`abi3-py310`, so one wheel per platform covers CPython 3.10 upward. The GIL is
released around sampling, so a long batch does not block the interpreter.
`pc.variants()` lists every game with how it deals, so a caller needs no table
of its own, and `pc.parse_hand_field` hands back masks so another parser can
be checked against this one.

**Averaging the sums yourself: divide by `weight_sum`, not `samples`.** The two
are equal for almost every request, so the wrong one costs nothing until a
spot is weighted, then returns equities hundreds of times too small.

```python
r = pc.chunk_from_text("stud", ["A23", "456", "789", "TJQ"], samples=200_000)
equity = [total / r["weight_sum"] for total in r["share_sum"]]   # not / samples

spread = (r["share_square_sum"][0]
          - 2 * equity[0] * r["square_weight_share_sum"][0]
          + equity[0] ** 2 * r["weight_square_sum"])
std_error = max(spread, 0.0) ** 0.5 / r["weight_sum"]
```

`r["weighted"]` and `r["effective_samples"]` are that run's own figures;
`tests/python/test_binding.py` checks this rebuild against the engine's.

## Building

```sh
cargo test                              # 234 tests, about ten seconds
cargo test --release -- --ignored       # the exhaustive sweeps
cargo run --release --bin benchmark     # speed, per game
```

The tests spread across every core, which is not always what you want on the
machine you are also using. `scripts/quiet` runs the same commands pinned to
four cores at the lowest priority, and costs about fifteen percent of wall
time:

```sh
scripts/quiet test
QUIET_CORES=2 scripts/quiet run --release --bin benchmark
```

Tests build at `opt-level = 2`, because the equity tests run a hundred
thousand Monte Carlo deals apiece and that is minutes unoptimised against
seconds optimised.

`build.rs` reports what it generated — how full the perfect hash is, how many
distinct values each kernel has, what the tables weigh — when asked:

```sh
POKER_EQUITY_BUILD_STATS=1 cargo build
```

`tests/fixtures/notation.tsv` is the grammar's source of truth — a plain table
of input and expected output, with a row for every rule above. It is a flat
file rather than Rust so that another implementation of the notation can be
checked against the same rows.
