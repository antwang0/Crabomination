#!/usr/bin/env python3
"""Paired wall-clock A/B between two `bot_ladder` binaries, with an honest
statement of what it can and cannot resolve.

Every pass on this branch that wanted a clock number has hand-rolled the same
loop — run A, run B, alternate a few times, quote best-of — and that estimator
is wrong in a way that matters. `cae6b605` measured **-1.946 % in Ir on
`--decks sos`**; nine alternated pairs of the same workload read

    best-of        base 56.9 s   tip 58.3 s    +2.5 % (the tip looks slower)
    median ratio                               +0.3 % (nothing)

**Best-of picks the luckiest run on each side, so it reports the difference
between two extreme order statistics of an unknown distribution** — with a
6.5 % within-binary spread on this box that is mostly which side happened to
catch the quiet minute. This runs an **ABBA** schedule (which cancels linear
host drift exactly within a block), pairs each block's two A runs against its
two B runs, and reports the **mean of the per-block ratios with a 95 % t
confidence interval** — so a result comes with the effect size the sample can
actually distinguish, not just a sign.

    python3 scripts/ab_wall.py --bin-a /tmp/base/bot_ladder \\
        --bin-b target/release-fast/bot_ladder --blocks 5 \\
        -- --a gang --b gang --games 4000 --decks sos --seed 11 --threads 4

Everything after `--` goes to both binaries verbatim. `--bench` works too;
the timing is read from `wall_s` when the run prints one and from the
ladder's `NN decided, … in X.Ys` line otherwise.

**It also checks that the two binaries agree on what they played.** A
behaviour-preserving change must produce the same `decisions` (and the same
decided/undecided split); a mismatch is printed as a FAILURE and the timing
is not reported, because a faster binary that plays a different game is not a
faster binary.

Rules this file exists to enforce:

- Never quote a single run. The same base binary read 129.1 and 156.5
  games/s minutes apart (pass 54) and 56.9 and 60.6 s on the same workload
  (`cae6b605`).
- Never quote best-of across binaries. Use the paired block ratios.
- Say the resolution. If the CI straddles zero the honest answer is "flat",
  not the sign of the difference.
- **Run the null control** (`--bin-a X --bin-b X`) on the same workload and
  the same block count before trusting a verdict. A null that comes back
  significant means the box moved, not the code.

Calibration on the routine box (Intel Xeon @ 2.10GHz, 4 cores),
`--games 2000 --decks sos --threads 4`, ~30 s a run:

    8 blocks, base vs tip   mean +0.18 %   CI -1.64 .. +2.00 %   FLAT
    8 blocks, null control  mean -0.40 %   CI -2.45 .. +1.66 %   FLAT

**THE RESOLUTION IS A PROPERTY OF THE WORKLOAD, NOT OF THE BOX**, and the
`sos` line above was read as the latter for four passes. `--games 2000
--decks fixed --threads 4`, same box, same 8 blocks:

    8 blocks, null control  mean +0.05 %   CI -0.29 .. +0.39 %   FLAT

**+/-0.34 %, six times finer**, because that workload's within-binary spread
is 1.6 % against `sos`'s 6.5 %. So: **run the null on the workload you are
about to quote**, and read its "cannot resolve anything smaller than" line
rather than carrying a number over from another pool. What survives from the
`sos` calibration is the shape of the mistake, not its magnitude.

Four blocks is *not* enough: the same pair read `+1.26 %` with a half-range
"resolution" of `+/-0.67 %` at four blocks, which is a significant verdict on
an effect that eight blocks and the null both call flat.
"""

import argparse
import re
import statistics
import subprocess
import sys
import time

# `NN decided, MM undecided, in 12.3s` — the ladder's own timer, which excludes
# process start-up and deck construction the same way on both sides.
LADDER_TIME = re.compile(r"\bin ([0-9.]+)s\b")
# `--bench`'s own row wins when present: it is the same clock, already parsed.
BENCH_TIME = re.compile(r"^\s*wall_s\s+([0-9.]+)\s*$", re.M)
DECISIONS = re.compile(r"^\s*decisions\s+(\d+)\s*$", re.M)
SPLIT = re.compile(r"^(\d+) decided, (\d+) undecided", re.M)

# Two-sided 95 % t critical values by degrees of freedom (n - 1), as
# (df, t) in ascending df. `t95` takes the largest entry at or below the
# sample's df, so a missing df is conservative rather than a KeyError.
T95 = [(2, 4.303), (3, 3.182), (4, 2.776), (5, 2.571), (6, 2.447), (7, 2.365),
       (8, 2.306), (9, 2.262), (10, 2.228), (11, 2.201), (12, 2.179),
       (13, 2.160), (14, 2.145), (15, 2.131), (19, 2.093), (29, 2.045)]


def t95(df):
    return next(t for d, t in reversed(T95) if d <= df)


def run(binary, args):
    """One run. Returns (seconds, fingerprint) or exits on a non-zero status."""
    t0 = time.monotonic()
    p = subprocess.run([binary, *args], capture_output=True, text=True)
    wall = time.monotonic() - t0
    if p.returncode != 0:
        sys.exit(f"{binary} exited {p.returncode}:\n{p.stderr[-2000:]}")
    out = p.stdout
    m = BENCH_TIME.search(out) or LADDER_TIME.search(out)
    secs = float(m.group(1)) if m else wall
    # What the run played, for the behaviour guard. `decisions` is only on
    # `--bench`; the decided/undecided split is on every ladder printout.
    d = DECISIONS.search(out)
    s = SPLIT.search(out)
    fingerprint = (d.group(1) if d else None, s.groups() if s else None)
    return secs, fingerprint


def main():
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("--bin-a", required=True, help="baseline binary")
    ap.add_argument("--bin-b", required=True, help="candidate binary")
    ap.add_argument(
        "--blocks",
        type=int,
        default=4,
        help="ABBA blocks; each is two runs of each binary (default 4)",
    )
    ap.add_argument(
        "--warmup",
        action="store_true",
        help="one discarded run of each binary first (cold page cache on a "
        "140 MB binary is worth a second or two)",
    )
    ap.add_argument("rest", nargs=argparse.REMAINDER)
    args = ap.parse_args()
    workload = args.rest[1:] if args.rest and args.rest[0] == "--" else args.rest
    if not workload:
        sys.exit("nothing after `--`: give the bot_ladder arguments to run")

    print(f"# A {args.bin_a}")
    print(f"# B {args.bin_b}")
    print(f"# {' '.join(workload)}")
    print(f"# {args.blocks} ABBA blocks = {4 * args.blocks} runs")

    if args.warmup:
        run(args.bin_a, workload)
        run(args.bin_b, workload)
        print("# warmup discarded")

    a_all, b_all, ratios, prints = [], [], [], set()
    for i in range(args.blocks):
        # ABBA: the mean of the two A runs and the mean of the two B runs sit
        # at the same point in time, so a linear drift across the block
        # cancels instead of landing on whichever binary went first.
        a1, fa1 = run(args.bin_a, workload)
        b1, fb1 = run(args.bin_b, workload)
        b2, fb2 = run(args.bin_b, workload)
        a2, fa2 = run(args.bin_a, workload)
        prints.update([fa1, fb1, fb2, fa2])
        a, b = (a1 + a2) / 2, (b1 + b2) / 2
        a_all += [a1, a2]
        b_all += [b1, b2]
        ratios.append(b / a)
        print(
            f"block {i + 1}  A {a1:6.2f} {a2:6.2f} -> {a:6.2f}   "
            f"B {b1:6.2f} {b2:6.2f} -> {b:6.2f}   B/A {b / a:.4f}"
        )

    if len(prints) > 1:
        print("\nFAILURE: the two binaries did not play the same games:")
        for f in sorted(prints, key=str):
            print(f"  decisions={f[0]} split={f[1]}")
        sys.exit("a faster binary that plays a different game is not a faster binary")

    if len(ratios) < 3:
        sys.exit(
            "\nrefusing a verdict from fewer than 3 blocks: with one or two "
            "ratios there is nothing to read a spread off, and a number with "
            "no resolution attached is what this script exists to replace."
        )
    mean = statistics.fmean(ratios)
    med = statistics.median(ratios)
    faster = sum(1 for r in ratios if r < 1.0)
    # 95 % CI on the mean block ratio. The half-range this used to print is
    # not a resolution: with four samples it under-reports the spread often
    # enough to call a null run significant, which is the failure a
    # `--bin-a X --bin-b X` control catches. df = n - 1.
    half = t95(len(ratios) - 1) * statistics.stdev(ratios) / len(ratios) ** 0.5
    print(f"\nB/A per block   {'  '.join(f'{r:.4f}' for r in ratios)}")
    print(f"mean B/A        {mean:.4f}  ({100 * (mean - 1):+.2f} %)")
    print(f"median B/A      {med:.4f}  ({100 * (med - 1):+.2f} %)")
    print(f"blocks B faster {faster}/{len(ratios)}")
    print(
        f"A spread        {min(a_all):.2f}-{max(a_all):.2f} s "
        f"({100 * (max(a_all) - min(a_all)) / min(a_all):.1f} %)"
    )
    print(
        f"B spread        {min(b_all):.2f}-{max(b_all):.2f} s "
        f"({100 * (max(b_all) - min(b_all)) / min(b_all):.1f} %)"
    )
    print(
        f"95 % CI         {100 * (mean - half - 1):+.2f} % .. "
        f"{100 * (mean + half - 1):+.2f} %   (t on {len(ratios)} block ratios)"
    )
    if abs(mean - 1) <= half:
        print(
            f"verdict         FLAT — the CI straddles 0. This sample cannot "
            f"resolve anything smaller than +/-{100 * half:.2f} %."
        )
    else:
        print(f"verdict         B is {100 * abs(mean - 1):.2f} % "
              f"{'faster' if mean < 1 else 'SLOWER'} than A")
    print(
        "# run `--bin-a X --bin-b X` on the same workload before trusting a "
        "verdict: a null that comes back significant means the box moved, not "
        "the code."
    )


if __name__ == "__main__":
    main()
