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
