#!/usr/bin/env python3
"""Paired `--bench` A/B: a wall-clock throughput delta off a host that cannot
give one in a single run.

PERF's Baseline has said "do not read a throughput delta off `--bench`" for
many passes, and for one run of each side that is right — `games_per_s` reads
212-407 across a day here on the same binary. What defeats it is *pairing*:
alternate the two binaries A/B/A/B..., so every drift the host has (frequency,
a neighbour, page cache) lands on both sides in the same proportion, and take
the median of the per-pair ratios.

    python3 scripts/bench_ab.py /tmp/base_bl /tmp/cand_bl [reps]

Resolution, measured at the hundred-and-sixteenth pass: 16 pairs resolved a
change callgrind priced at -2.011 % Ir as **+2.62 % median games/s** (paired
mean +1.47 %, per-pair sd 4.6 points) where the single-run spread was 237-276.
So: it separates ~2 % from zero, and it does NOT separate 0.3 % from zero.
**Ir is still the signal for anything smaller** — this is the confirmation that
an Ir win is a wall-clock win, not a replacement for the Ir reading.

⚠ Both binaries must be built under the *same* profile and feature set, and
`--bench` pins everything else (decks, seeds, thread count).
"""
import re
import statistics as st
import subprocess
import sys

GAMES_PER_S = re.compile(r"^\s*games_per_s\s+([\d.]+)", re.M)


def one(binary: str) -> float:
    out = subprocess.run([binary, "--bench"], capture_output=True, text=True).stdout
    m = GAMES_PER_S.search(out)
    if not m:
        sys.exit(f"{binary}: no games_per_s line — did it run?")
    return float(m.group(1))


def main() -> None:
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    a_bin, b_bin = sys.argv[1], sys.argv[2]
    reps = int(sys.argv[3]) if len(sys.argv) > 3 else 16
    a, b = [], []
    for i in range(reps):
        a.append(one(a_bin))
        b.append(one(b_bin))
        print(f"  {i + 1:>3}  A {a[-1]:8.2f}   B {b[-1]:8.2f}", flush=True)
    ratios = [100 * (y - x) / x for x, y in zip(a, b)]
    for name, xs in (("A " + a_bin, a), ("B " + b_bin, b)):
        print(
            f"{name:<40} median {st.median(xs):8.2f}  mean {st.mean(xs):8.2f}"
            f"  min {min(xs):7.2f}  max {max(xs):7.2f}  sd {st.pstdev(xs):6.2f}"
        )
    print(
        f"{'paired B/A':<40} median {st.median(ratios):+7.2f} %"
        f"  mean {st.mean(ratios):+7.2f} %  sd {st.pstdev(ratios):6.2f}"
    )
    print("  (< ~1 % is inside this instrument's noise — quote Ir for those)")


if __name__ == "__main__":
    main()
