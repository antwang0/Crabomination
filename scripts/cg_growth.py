#!/usr/bin/env python3
"""Rank the allocator's *growth* callers by growths per CALL — PERF's
(-103)/(-108) discriminator, as one command instead of a two-table join.

    python3 scripts/cg_growth.py cg.out [rows]

**Why the division and not the count.** `RawVec` reaches the allocator two
ways and both land in `finish_grow`:

* `grow_one`                — the `push` path
* `do_reserve_and_handle`   — the `reserve` / `extend` / `append` path

A caller's *growth count* mixes a **first** push, which a `reserve` only
**moves** into the reserve, with a **re-growth**, which a reserve **removes**
outright. Only the second is worth taking. Divide the row by the caller's own
call count and the ones above ~1.5 are the re-growth rows; at ~1.0 and below
the buffer reaches the heap once and a reserve buys nothing (see PERF's
`(-80)` row 2, and `(-103)` for the four rows the rule took).

**Both tables, not one.** `(-103)` divided `grow_one` only and closed with
"the table has no row left that the rule takes" — true, and it left
`do_reserve_and_handle` undivided, where `auto_tap_for_cost_inner` sat at 1.72
a call and shipped at -0.34 / -0.29 / -0.36 % (`(-108)`). **`Vec::append` is a
reserve site** — it reserves `other.len()` every time — so an append loop
walks the growth ladder exactly as pushes would and appears in *no* push
census at all.

The `calls` column is the caller's own incoming call count, folded over every
edge that reaches it, exactly as `cg_calls.py` computes it. A row whose caller
has no incoming edges in the dump (a leaf entered only from `main`) prints
`-` rather than a ratio: the division is meaningless there, not zero.
"""
import collections
import re
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from cg_edges import parse  # noqa: E402  (same parse, one source of truth)

GROWTH = ("grow_one", "do_reserve_and_handle")


def main() -> None:
    if len(sys.argv) < 2:
        print(__doc__)
        raise SystemExit(2)
    path = sys.argv[1]
    rows = int(sys.argv[2]) if len(sys.argv) > 2 else 25
    _self_cost, edge_cost, edge_calls, total, _declared = parse(path)

    # Incoming call count per function, over every edge that reaches it.
    incoming: collections.Counter = collections.Counter()
    for (caller, callee), n in edge_calls.items():
        incoming[callee] += n

    # Growths per caller, over both growth paths, folded to the caller.
    growths: collections.Counter = collections.Counter()
    growth_ir: collections.Counter = collections.Counter()
    for (caller, callee), n in edge_calls.items():
        if any(g in callee for g in GROWTH):
            growths[caller] += n
            growth_ir[caller] += edge_cost.get((caller, callee), 0)

    print(f"# total Ir {total:,}")
    print(f"# {sum(growths.values()):,} growths over {len(growths)} callers, "
          f"ranked by growths per call")
    print(f"{'growths':>10}  {'calls':>10}  {'per call':>8}  {'Ir (incl)':>13}  name")
    def key(fn: str) -> float:
        c = incoming.get(fn, 0)
        return growths[fn] / c if c else -1.0
    for fn in sorted(growths, key=key, reverse=True)[:rows]:
        c = incoming.get(fn, 0)
        per = f"{growths[fn] / c:8.2f}" if c else "       -"
        print(f"{growths[fn]:>10,}  {c:>10,}  {per}  {growth_ir[fn]:>13,}  {fn}")
    if len(growths) > rows:
        print(f"# ... {len(growths) - rows} further callers not shown")


if __name__ == "__main__":
    main()
