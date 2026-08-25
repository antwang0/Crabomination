#!/usr/bin/env python3
"""Rank two callgrind dumps of the same tip by the RATIO of their self shares.

PERF's "Which pool a change moves" describes this device and calls it "one
script"; there was no script, and every pass that used it re-derived the join
by hand. It found the sixty-second pass's second commit
(`layers::affected_includes_gated`, 0.61 % of `cube` against 0.12 % of `sos`,
**5.08x**) and it pointed the sixty-third at `pick_blocks_inner` (2.09x).

    python3 scripts/cg_ratio.py cg.cube.out cg.sos.out --floor 0.45

A row that is 0.61 % of one pool is nowhere near the top of any table and
nobody would look at it. Five times the share *per instruction* on the
grant-heavy pool says the work is pool-specific and structural rather than
diffuse — which is a pointer to a shape, not a size. **Confirm with Ir/call
before costing anything**: the ratio says "this pool does more of it", never
"this is big".

Both dumps must come from the same binary at the same tip, or the join is
comparing two different programs. `--floor` drops rows below that percent of
the numerator pool, because a row at 0.01 % has a meaningless ratio and a
row truncated out of one dump reads as an infinite one — which is why the
device wants `--rows 0` semantics (this script reads the parse directly, so
it has them by construction).
"""
import argparse
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from cg_edges import parse  # noqa: E402


def shares(path):
    self_cost, _, _, total, declared = parse(path)
    if declared and abs(declared - total) > max(1, declared // 1000):
        print(
            f"# WARNING {path}: self lines sum to {total:,}, "
            f"dump declares {declared:,}",
            file=sys.stderr,
        )
    return {name: 100.0 * c / total for name, c in self_cost.items()}, total


def main():
    ap = argparse.ArgumentParser(add_help=True)
    ap.add_argument("numerator", help="dump whose share goes on top (usually cube)")
    ap.add_argument("denominator", help="dump whose share goes below (usually sos)")
    ap.add_argument(
        "--floor",
        type=float,
        default=0.45,
        help="drop rows below this percent of the numerator pool (default 0.45)",
    )
    ap.add_argument("--rows", type=int, default=25, help="rows to print, 0 = all")
    args = ap.parse_args()

    num, num_total = shares(args.numerator)
    den, den_total = shares(args.denominator)
    print(f"# numerator   {args.numerator}  {num_total:,} Ir")
    print(f"# denominator {args.denominator}  {den_total:,} Ir")
    print(f"# floor {args.floor} % of the numerator; the two totals do not compare")

    # A row absent from the denominator is not an infinite ratio — it is a row
    # that pool never executes, which is a stronger finding than a large one.
    # It gets its own section so the number column stays honest.
    ranked, only = [], []
    for name, n in num.items():
        if n < args.floor:
            continue
        d = den.get(name, 0.0)
        (only if d == 0.0 else ranked).append((name, n, d))
    ranked.sort(key=lambda r: r[1] / r[2], reverse=True)

    print(f"\n{'num%':>6} {'den%':>6} {'x':>6}  row")
    limit = len(ranked) if args.rows == 0 else args.rows
    for name, n, d in ranked[:limit]:
        print(f"{n:6.2f} {d:6.2f} {n / d:6.2f}  {name}")
    if len(ranked) > limit:
        print(f"# ... {len(ranked) - limit:,} more rows above the floor, not shown")
    if only:
        print(f"\n# {len(only)} row(s) above the floor with NO denominator cost at all:")
        for name, n, _ in sorted(only, key=lambda r: -r[1]):
            print(f"{n:6.2f}      -      -  {name}")


if __name__ == "__main__":
    main()
