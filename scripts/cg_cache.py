#!/usr/bin/env python3
"""Rank a **cachegrind** dump's functions by one event — the columns
callgrind never records.

`callgrind` counts Ir, and the 248 legs before this script were priced on
it. Ir is blind to two costs a superscalar core pays for: an instruction
fetch that misses L1i (`I1mr`) and a conditional branch it guessed wrong
(`Bcm`, ~15-20 cycles each). At the `(-248)` tip on `cube` those are
76 M I1 misses and 37 M mispredicts against 1.89 G Ir — either one, at a
plausible per-event cycle cost, is a comparable share of wall clock to the
instructions themselves. `cg_annotate` groups by file and prints every
column at once; this aggregates by *function*, one column, top N.

    valgrind --tool=cachegrind --cache-sim=yes --branch-sim=yes \\
      --cachegrind-out-file=cache.out target/profiling-fast/bot_ladder \\
      --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
    python3 scripts/cg_cache.py cache.out Bcm        # cond. mispredicts
    python3 scripts/cg_cache.py cache.out I1mr 40    # L1i misses, 40 rows
    python3 scripts/cg_cache.py cache.out Bcm --rate # mispredicts / Bc

Columns are the dump's own `events:` line (Ir I1mr ILmr Dr D1mr DLmr Dw
D1mw DLmw Bc Bcm Bi Bim). `--rate` divides the column by its denominator
(Bcm/Bc, Bim/Bi, I1mr/Ir, D1mr/Dr, D1mw/Dw) so a *small* function with a
badly-predicted branch ranks; without it the table is share of the total.
"""
import sys


def main() -> None:
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    path, col = sys.argv[1], sys.argv[2]
    rows = 30
    rate = "--rate" in sys.argv
    for a in sys.argv[3:]:
        if a.isdigit():
            rows = int(a)
    events: list[str] = []
    per_fn: dict[str, list[int]] = {}
    fn = None
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if line.startswith("events:"):
                events = line.split()[1:]
                continue
            if line.startswith("fn="):
                fn = line[3:].strip()
                continue
            if fn is None or not line[:1].isdigit():
                continue
            parts = line.split()
            vals = [int(x) for x in parts[1:]]
            acc = per_fn.setdefault(fn, [0] * len(events))
            for i, v in enumerate(vals):
                acc[i] += v
    if col not in events:
        sys.exit(f"no column {col}; the dump has {' '.join(events)}")
    ci = events.index(col)
    denom_of = {"Bcm": "Bc", "Bim": "Bi", "I1mr": "Ir", "ILmr": "Ir",
                "D1mr": "Dr", "DLmr": "Dr", "D1mw": "Dw", "DLmw": "Dw"}
    total = sum(v[ci] for v in per_fn.values())
    di = events.index(denom_of[col]) if rate and col in denom_of else None
    print(f"# {col} total {total:,} over {len(per_fn):,} functions"
          + (f"; ranked by {col}/{events[di]} (>= 20,000 {events[di]})" if di is not None else ""))
    if di is not None:
        ranked = sorted(
            ((v[ci] / v[di], v[ci], v[di], k) for k, v in per_fn.items() if v[di] >= 20_000),
            reverse=True,
        )[:rows]
        for r, n, d, k in ranked:
            print(f"  {r * 100:6.2f} %  {n:>12,} / {d:>13,}  {k[:110]}")
        return
    ranked = sorted(((v[ci], v[events.index('Ir')], k) for k, v in per_fn.items()), reverse=True)[:rows]
    for n, ir, k in ranked:
        print(f"  {n:>12,} ({n * 100 / total:5.2f} %)  Ir {ir:>14,}  {k[:110]}")


if __name__ == "__main__":
    main()
