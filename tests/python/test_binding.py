"""Checks the Python binding against the same figures the Rust tests use.

Build the module first:

    cargo build --release --features python
    cp target/release/libpoker_equity.so <somewhere>/poker_equity.so

then run this with that directory on PYTHONPATH:

    PYTHONPATH=<somewhere> python3 tests/python/test_binding.py

It is not wired into `cargo test`, because building an extension module and
loading it into an interpreter is a different job from running the library's
own tests, and pyo3's extension-module feature deliberately leaves the Python
symbols unresolved so a Rust test binary cannot link it.
"""

import sys

import poker_equity as pe

failures = []


def check(condition, message):
    if condition:
        print(f"  ok   {message}")
    else:
        print(f"  FAIL {message}")
        failures.append(message)


def close(actual, expected, tolerance, message):
    check(abs(actual - expected) < tolerance, f"{message}: {actual:.4f} ~ {expected:.4f}")


print("every game is reachable by key")
keys = {v["key"] for v in pe.variants()}
check(len(keys) == 15, f"fifteen games, got {len(keys)}")
check("omaha_five_hi_lo" in keys, "five-card Omaha hi/lo is there")
check(not any(k[0].isdigit() for k in keys), "no key starts with a digit")

print("\nexact results carry no error bar")
result = pe.exact_from_text("holdem", ["AhAd", "KsKc"], "2c 7d 9h")
check(result["exact"], "reported as exact")
check(result["samples"] == 990, f"990 boards, got {result['samples']}")
close(result["players"][0]["equity"] * 100, 91.6162, 0.001, "aces against kings on 2-7-9")
check(result["players"][0]["std_error"] == 0.0, "standard error is exactly zero")

print("\nsampling lands on the same answer")
sampled = pe.chunk_from_text("holdem", ["AhAd", "KsKc"], "2c 7d 9h", samples=200_000, seed=11)
close(sampled["players"][0]["equity"] * 100, 91.6162, 0.5, "within half a point of exact")

print("\nequities divide one pot")
for key, hands, board in [
    ("holdem", ["AhKh", "QsQd"], ""),
    ("omaha", ["AhAdKsQc", "JhTc9s8d"], ""),
    ("omaha_hi_lo", ["Ah2c3d4s", "KhKsQhQs"], "5c 6d 8h"),
    ("stud", ["Ah2c3d", "Qs Qd Js"], ""),
    ("deuce_seven", ["Th8c4s2h", "9d7h4h2d"], ""),
    ("five_card_draw", ["AhKhQh7h", "9s8c7d6s"], ""),
    ("badugi", ["Ac2d3h", "4s6s7d"], ""),
]:
    result = pe.chunk_from_text(key, hands, board, samples=20_000, seed=5)
    total = sum(p["equity"] for p in result["players"])
    close(total, 1.0, 1e-9, f"{key} equities sum")

print("\nbatches spread across threads")
check(pe.default_thread_count() >= 1, "a default thread count is offered")
one = pe.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=200_000, seed=5, threads=1)
many = pe.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=200_000, seed=5, threads=4)
check(one["samples"] == many["samples"] == 200_000, "both ran every deal asked for")
close(one["players"][0]["equity"], many["players"][0]["equity"], 0.01,
      "one thread and four agree")

print("\nthe raw sums cross, so batches can be merged here")
a = pe.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=100_000, seed=1)
b = pe.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=100_000, seed=2)
for field in ("share_sum", "share_square_sum", "low_share_sum",
              "win_count", "tie_count", "scoop_count",
              "high_win_count", "high_tie_count", "low_win_count", "low_tie_count"):
    check(field in a, f"{field} is returned")

print("\neach half of a split pot is counted on its own")
# Two ways to half the pot on one board: the low alone against the high alone,
# and both halves split between mirror images. Same equity, different halves.
apart = pe.exact_from_text("omaha_hi_lo", ["Ac4d9sTs", "KdQcJcJd"], "2c 3d 7h Kh Ks")
together = pe.exact_from_text("omaha_hi_lo", ["Ac4dKdQc", "As4cKcQd"], "2c 3d 7h Kh Ks")
halves = ("high_win", "high_tie", "low_win", "low_tie")
for result in (apart, together):
    close(result["players"][0]["equity"], 0.5, 1e-12, "half the pot either way")
check([apart["players"][0][h] for h in halves] == [0, 0, 1, 0], "the low alone")
check([apart["players"][1][h] for h in halves] == [1, 0, 0, 0], "the high alone")
check([together["players"][0][h] for h in halves] == [0, 1, 0, 1], "both halves split")
check(apart["low_win_count"] == [1.0, 0.0], "the counts cross as well as the fractions")
# Outside a split game the high half is the whole pot.
check(a["high_win_count"] == a["win_count"], "hold'em's high wins are its wins")
check(a["low_win_count"] == [0.0, 0.0], "and it has no low")
merged = sum(x + y for x, y in zip(a["share_sum"][:1], b["share_sum"][:1]))
close(merged / (a["samples"] + b["samples"]),
      a["players"][0]["equity"], 0.01, "hand-merged batches give the same equity")

print("\nexact answers from masks, not only from text")
hands = [[[1 << pe.card_index("AhKh"[i:i+2]) for i in (0, 2)]],
         [[1 << pe.card_index("QsQd"[i:i+2]) for i in (0, 2)]]]
board = [1 << pe.card_index(c) for c in ("2c", "7d", "9h")]
walked = pe.exact("holdem", hands, board, 0)
check(walked is not None and walked["exact"], "a river-ish spot comes back exact")
check(walked["samples"] == 990, f"990 run-outs, got {walked['samples']}")
check(walked["players"][0]["std_error"] == 0.0, "and carries no error bar")

print("\nthe mask API agrees with the text API")
masks = [[[1 << pe.card_index("Ah"), 1 << pe.card_index("Kh")]],
         [[1 << pe.card_index("Qs"), 1 << pe.card_index("Qd")]]]
by_mask = pe.chunk("holdem", masks, [], 0, 200_000, 3)
by_text = pe.chunk_from_text("holdem", ["AhKh", "QsQd"], samples=200_000, seed=3)
close(by_mask["players"][0]["equity"], by_text["players"][0]["equity"], 1e-12,
      "identical, deal for deal")

print("\ncard indices are rank * 4 + suit")
check(pe.card_index("2c") == 0, "the deuce of clubs is zero")
check(pe.card_index("Ah") == 12 * 4 + 2, "the ace of hearts")
check(pe.card_name(0) == "2c", "and back again")
check(pe.full_deck() == (1 << 52) - 1, "the deck is fifty-two bits")

print("\nthe parser hands back masks")
check(len(pe.parse_hand_field("AKs", 2)) == 4, "AKs is four combinations")
check(len(pe.parse_hand_field("22+", 2)) == 78, "22+ is every pair")
check(pe.parse_hand_field("2 c", 2)[0] == [0xF, 0x1111111111111], "a deuce and a club")
check(len(pe.parse_hand_field("Ah 2c 3d", 7, allow_short=True)[0]) == 3,
      "a stud field may be short")
check(pe.parse_dead_cards("Ah Kd") == (1 << pe.card_index("Ah")) | (1 << pe.card_index("Kd")),
      "dead cards are one mask")

print("\nbad input raises, with the span")
for bad, slots in [("AhXh", 2), ("AhAh", 2), ("22s", 2), ("top 15%", 2)]:
    try:
        pe.parse_hand_field(bad, slots)
        check(False, f"{bad!r} should have raised")
    except ValueError as error:
        check("\n" in str(error), f"{bad!r} raises with an underlined span")

try:
    pe.chunk_from_text("no_such_game", ["AhKh", "QsQd"])
    check(False, "an unknown game should raise")
except ValueError as error:
    check("no game called" in str(error), "an unknown game names itself")

print("\nthe exported sums rebuild the answer the engine gives")
# A caller that merges batches on this side averages the sums itself, so the
# sums and the engine must agree about the divisor. They do not agree if the
# caller reaches for `samples`: that is the deal count, and what the deals are
# worth is `weight_sum`. The two are equal for almost every request, which is
# what makes the mistake worth a test -- it costs nothing until a spot is
# contended enough for the sampler to weigh its deals, and then the equities
# come back hundreds of times too small without anything looking wrong.
for label, game, hands in [
    ("hold'em, nothing contended", "holdem", ["AhKh", "QsQd"]),
    ("stud, mildly contended", "stud", ["A23", "456"]),
    ("stud, weighted", "stud", ["A23", "456", "789", "TJQ"]),
    ("hold'em, weighted", "holdem", ["c c"] * 5),
]:
    result = pe.chunk_from_text(game, hands, samples=100_000, seed=5)
    engine = [seat["equity"] for seat in result["players"]]
    divisor = result["weight_sum"]
    rebuilt = [total / divisor for total in result["share_sum"]]
    check(
        all(abs(a - b) < 1e-12 for a, b in zip(engine, rebuilt)),
        f"{label}: share_sum / weight_sum is the engine's equity",
    )
    close(sum(engine), 1.0, 1e-9, f"{label}: equities sum to one")

    # And the error bar, which needs the other two sums and cannot be
    # recovered from the shares alone.
    for seat, mean in enumerate(rebuilt):
        spread = (
            result["share_square_sum"][seat]
            - 2.0 * mean * result["square_weight_share_sum"][seat]
            + mean * mean * result["weight_square_sum"]
        )
        check(
            abs(max(spread, 0.0) ** 0.5 / divisor - result["players"][seat]["std_error"]) < 1e-15,
            f"{label}: seat {seat}'s error bar rebuilds from the sums",
        )

    check("weighted" in result, f"{label}: says whether it was weighted")
    check(
        result["effective_samples"] <= result["samples"] + 1e-9,
        f"{label}: effective samples never exceed the deal count",
    )
    if not result["weighted"]:
        check(
            abs(result["weight_sum"] - result["samples"]) < 1e-9,
            f"{label}: an unweighted run weighs one per deal",
        )

print()
if failures:
    print(f"{len(failures)} checks failed")
    sys.exit(1)
print("all checks passed")
