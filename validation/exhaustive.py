"""Checks every hand of a game against pokerkit, without holding them.

`cross_check.py` sorts every hand it looks at, so it needs them all in memory
at once. That is fine for five-card hands and impossible for seven, where
there are 133,784,560 of them. This walks the same ground in chunks and keeps
only what it has to.

What it keeps is the *mapping*, not the hands: which of our scores goes with
which of pokerkit's ranks. There are a few thousand distinct hand values in
any of these rankings, so that dictionary stays small however many hands pass
through it. A hand is checked against the mapping and discarded. Two things
are then true at the end if nothing has complained:

  - every score of ours means one rank of theirs, and the reverse, so we and
    pokerkit cut the hands into the same classes;
  - sorted by our score, their ranks move in one direction throughout.

Together those say the two rankings are the same ranking.

Work is split by the first two cards of a hand and handed to a pool, because
pokerkit evaluates around eight thousand seven-card hands a second and there
is no other way to get through a hundred and thirty million of them. Our own
side scores seven million a second, so it never waits.

    PYTHONPATH=<where poker_calculator.so is> \\
        .venv/bin/python validation/exhaustive.py high

Add `--limit N` to stop after roughly N hands, which is how to see that it
works before committing to the whole run.
"""

import argparse
import itertools
import multiprocessing
import sys
import time
from pathlib import Path

import poker_calculator as pc
from pokerkit.hands import (
    BadugiHand,
    RegularLowHand,
    ShortDeckHoldemHand,
    StandardHighHand,
    StandardLowHand,
)

FULL_DECK = [rank + suit for rank in "23456789TJQKA" for suit in "cdhs"]
SHORT_DECK = [card for card in FULL_DECK if card[0] not in "2345"]

# name -> (our kernel, deck, cards per hand, pokerkit class, what it covers)
GAMES = {
    "high": ("high", FULL_DECK, 5, StandardHighHand,
             "every five-card high hand"),
    "seven_high": ("high", FULL_DECK, 7, StandardHighHand,
                   "every seven-card hand, high"),
    "deuce_seven": ("deuce_seven", FULL_DECK, 5, StandardLowHand,
                    "every five-card deuce-to-seven hand"),
    "low_a5": ("low_a5", FULL_DECK, 5, RegularLowHand,
               "every five-card ace-to-five low"),
    "seven_low": ("low_a5", FULL_DECK, 7, RegularLowHand,
                  "every seven-card hand, ace-to-five low"),
    "short_deck": ("short_deck", SHORT_DECK, 5, ShortDeckHoldemHand,
                   "every five-card short-deck hand"),
    "seven_short_deck": ("short_deck", SHORT_DECK, 7, ShortDeckHoldemHand,
                         "every seven-card short-deck hand"),
    "badugi": ("badugi", FULL_DECK, 4, BadugiHand,
               "every four-card badugi holding"),
}

# How many hands to score at once. Large enough that crossing into Rust costs
# nothing, small enough that a chunk's hands fit comfortably in memory.
BATCH = 20_000


def count_hands(deck_size, size):
    """How many hands the walk will visit."""
    total = 1
    for step in range(size):
        total = total * (deck_size - step) // (step + 1)
    return total


def check_batch(game, hands):
    """Scores one batch both ways, returning the classes seen and any clashes."""
    kernel, _, _, evaluator, _ = GAMES[game]
    written = ["".join(hand) for hand in hands]

    ours = pc.score_batch(kernel, written)
    theirs = [evaluator.from_game(hand).entry.index for hand in written]

    ours_to_theirs, theirs_to_ours, clashes = {}, {}, []
    for hand, mine, yours in zip(written, ours, theirs):
        seen = ours_to_theirs.setdefault(mine, (yours, hand))
        if seen[0] != yours:
            clashes.append(f"we score {hand} and {seen[1]} alike ({mine}), "
                           f"pokerkit does not ({yours} against {seen[0]})")
        seen = theirs_to_ours.setdefault(yours, (mine, hand))
        if seen[0] != mine:
            clashes.append(f"pokerkit ranks {hand} and {seen[1]} alike ({yours}), "
                           f"we do not ({mine} against {seen[0]})")
    return ours_to_theirs, theirs_to_ours, clashes


def check_slice(task):
    """Walks every hand beginning with one pair of cards."""
    game, first, second = task
    _, deck, size, _, _ = GAMES[game]

    prefix = [deck[first], deck[second]]
    rest = deck[second + 1:]

    ours_to_theirs, theirs_to_ours, clashes = {}, {}, []
    seen = 0
    batch = []
    for tail in itertools.combinations(rest, size - 2):
        batch.append(prefix + list(tail))
        if len(batch) >= BATCH:
            merge(check_batch(game, batch), ours_to_theirs, theirs_to_ours, clashes)
            seen += len(batch)
            batch = []
    if batch:
        merge(check_batch(game, batch), ours_to_theirs, theirs_to_ours, clashes)
        seen += len(batch)

    return ours_to_theirs, theirs_to_ours, clashes, seen


def merge(found, ours_to_theirs, theirs_to_ours, clashes):
    """Folds one batch's classes into the running ones."""
    mine, yours, problems = found
    clashes.extend(problems)
    for score, entry in mine.items():
        seen = ours_to_theirs.setdefault(score, entry)
        if seen[0] != entry[0]:
            clashes.append(f"we score {entry[1]} and {seen[1]} alike ({score}), "
                           f"pokerkit does not ({entry[0]} against {seen[0]})")
    for rank, entry in yours.items():
        seen = theirs_to_ours.setdefault(rank, entry)
        if seen[0] != entry[0]:
            clashes.append(f"pokerkit ranks {entry[1]} and {seen[1]} alike ({rank}), "
                           f"we do not ({entry[0]} against {seen[0]})")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("game", choices=sorted(GAMES))
    parser.add_argument("--workers", type=int, default=multiprocessing.cpu_count())
    parser.add_argument("--limit", type=int, default=0,
                        help="stop after roughly this many hands, for a trial run")
    parser.add_argument("--out", type=Path, default=Path("disagreements.txt"))
    args = parser.parse_args()

    kernel, deck, size, evaluator, covers = GAMES[args.game]
    total = count_hands(len(deck), size)
    print(f"{args.game}: {covers}")
    print(f"  {total:,} hands, {args.workers} workers")

    # Split the walk by the first two cards, which gives many more pieces
    # than workers. The pieces are wildly uneven -- a hand starting with the
    # two lowest cards has far more ways to finish than one starting with the
    # two highest -- so the big ones go first. Left until last, a single large
    # piece would still be running long after everything else had finished.
    tasks = sorted(
        ((args.game, first, second)
         for first in range(len(deck) - size + 1)
         for second in range(first + 1, len(deck) - size + 2)),
        key=lambda task: count_hands(len(deck) - task[2] - 1, size - 2),
        reverse=True,
    )

    ours_to_theirs, theirs_to_ours, clashes = {}, {}, []
    seen = 0
    done = 0
    started = time.time()

    with multiprocessing.Pool(args.workers) as pool:
        for mine, yours, problems, counted in pool.imap_unordered(check_slice, tasks):
            merge((mine, yours, problems), ours_to_theirs, theirs_to_ours, clashes)
            seen += counted
            done += 1
            elapsed = time.time() - started
            # Every worker is busy from the start, but a piece only counts
            # when it finishes, so the rate reads low until the first few
            # land and then settles.
            rate = seen / max(elapsed, 1e-9)
            left = (total - seen) / rate if rate > 0 else 0
            print(f"\r  {seen:,}/{total:,} ({seen / total:5.1%})  "
                  f"{done}/{len(tasks)} pieces  {rate:,.0f}/s  "
                  f"{left / 60:.0f} min left  "
                  f"{len(ours_to_theirs):,} classes  {len(clashes)} clashes"
                  f"{' ' * 8}",
                  end="", flush=True)
            if args.limit and seen >= args.limit:
                print("\n  stopping early, as asked")
                pool.terminate()
                break
    print()

    # Sorted by our score, their ranks must move in one direction throughout.
    ordered = sorted((score, entry[0]) for score, entry in ours_to_theirs.items())
    direction = 0
    for (_, a), (_, b) in zip(ordered, ordered[1:]):
        if a != b:
            direction = 1 if b > a else -1
            break
    for (score_a, a), (score_b, b) in zip(ordered, ordered[1:]):
        if direction * (b - a) <= 0:
            clashes.append(f"order breaks between our scores {score_a} and {score_b}: "
                           f"pokerkit has {a} then {b}")

    print(f"  {len(ours_to_theirs):,} classes, ours and theirs "
          f"{'agree' if not clashes else 'DISAGREE'}")

    if clashes:
        args.out.write_text("\n".join(clashes))
        print(f"  {len(clashes):,} disagreements written to {args.out}")
        for clash in clashes[:5]:
            print(f"    {clash}")
        return 1

    print(f"  every hand agrees with pokerkit "
          f"({'partial run' if args.limit else 'exhaustive'})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
