# Cross-checking against an outside evaluator

The library's own sweeps in `tests/kernel_verification.rs` check each lookup
table against the evaluator that names its hands. Both were written here, from
one reading of the rules, so those sweeps catch a mistake in either one — but
a rule misunderstood in *both* survives all of them. They did their job once
already: they caught the low kernel's generator packing a hand's pairing shape
as a bitmask, which had every paired low in the wrong order. What they cannot
do is tell you the rules themselves are right.

This checks that from outside, against
[pokerkit](https://github.com/uoftcprg/pokerkit) — a research library that
implements each of these rankings separately, including the lowball and
split-pot ones most evaluators leave out.

Nothing here is part of the library or its build. It is meant to be run before
shipping, and again whenever a ranking changes.

## Running it

```sh
python3 -m venv .venv
.venv/bin/pip install pokerkit

cargo run --release --bin export_scores -- /tmp/scores
.venv/bin/python validation/cross_check.py /tmp/scores
```

Only scores cross between the two sides. For the exhaustive checks the hands
are implied by their order — both sides number cards `rank * 4 + suit` and
walk combinations in ascending order — and for the sampled ones the cards are
written out alongside the score.

## What it covers

Exhaustively, every hand in the space:

| Check | Hands | Against |
|---|---|---|
| `high` | all 2,598,960 five-card hands | `StandardHighHand` |
| `deuce_seven` | all 2,598,960 | `StandardLowHand` |
| `low_a5` | all 2,598,960 | `RegularLowHand` |
| `short_deck` | all 376,992 from the thirty-six | `ShortDeckHoldemHand` |
| `badugi` | all 270,725 four-card hands | `BadugiHand` |

Sampled, from a fixed seed, for the spaces too large to walk:

| Check | Hands | Covers |
|---|---|---|
| `seven_high` | 300,000 | holdings that hold more than they play |
| `seven_low` | 300,000 | the same, for lows |
| `seven_short_deck` | 300,000 | the same, over thirty-six cards |
| `omaha` | 200,000 deals | exactly two hole cards with exactly three of the board |

The seven-card checks matter more than their sampling suggests: the straight
bug this library once carried lived only in hands of more than five cards,
where a duplicate rank could sit above a straight.

## Going exhaustive on the rest

`cross_check.py` sorts every hand it looks at, so it needs them all in memory.
That caps it at five-card hands. `exhaustive.py` walks the same ground in
chunks and keeps only the *mapping* — which of our scores goes with which of
pokerkit's ranks — which is a few thousand entries however many hands pass
through. Hands are checked and discarded, so memory is flat.

```sh
cargo build --release --features python
cp target/release/libpoker_calculator.so /tmp/pymod/poker_calculator.so

PYTHONPATH=/tmp/pymod .venv/bin/python validation/exhaustive.py seven_high
```

Add `--limit N` for a trial run. Disagreements go to a file rather than the
screen, so a run that finds a systematic fault does not bury the summary.

The wall clock is set entirely by pokerkit, which evaluates about eight
thousand seven-card hands a second; our side scores seven million, so it never
waits. Measured at about 45,000 hands a second across sixteen cores:

| Check | Hands | Roughly |
|---|---|---|
| `badugi` | 270,725 | seconds |
| `high`, `deuce_seven`, `low_a5` | 2,598,960 each | a minute each |
| `short_deck` | 376,992 | seconds |
| `seven_short_deck` | 8,347,680 | 3 minutes |
| `seven_high` | 133,784,560 | ~50 minutes |
| `seven_low` | 133,784,560 | ~50 minutes |

So every game can be checked exhaustively in about two hours, once.

## Omaha is the exception

Omaha cannot be walked. A deal is four hole cards and five board cards, so
there are `C(52,4) × C(48,5)` = **4.6 × 10¹¹** of them. At the rate above that
is a hundred and forty years, and no amount of chunking changes it — the
problem is the size of the space, not the memory.

What can be said instead is that Omaha adds very little to what is already
proved. Its score is the best of the sixty ways to pair two hole cards with
three of the board, and each of those sixty is an ordinary five-card
evaluation that the exhaustive `high` check already covers. The only untested
part is the pairing itself, which is a fixed enumeration that does not depend
on which cards arrive: it produces the same sixty subsets whatever the deal.
A sample tests that, and a small one suffices, because a fault in a fixed
enumeration shows up on nearly every hand rather than hiding in a corner of
the space. The same argument covers five- and six-card Omaha and the split-pot
variants.

## Reading a disagreement

The two libraries number hands differently — pokerkit counts upwards from the
worst hand for a high ranking and from the best for a low one, while its
comparison operators normalise the two. The script works out which way round
each ranking runs by asking the operator, rather than assuming, and reports
that on each line. What it compares is the *order*: sorted by our score, their
ranks must fall, and hands we call equal they must call equal too.

A disagreement names both hands. Neither side is authoritative — pokerkit can
be wrong as well — so the next step is to work out what the rule actually is,
not to change this library to match.
