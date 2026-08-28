#!/usr/bin/env python3
"""Rank every function in a callgrind dump by **incoming call count**, with
its self Ir and its self Ir/call.

    python3 scripts/cg_calls.py cg.out [rows]

PERF's standing rules name this device — *"rank the dump by call count and
read the Ir/call column"* — and it is the one that found `Option::or_else`
(2,187,078 calls at ~5 Ir apiece, invisible to a self table, a callee table
and a line profile alike). It had no script for eight passes; every pass that
used it re-derived the join by hand from `cg_edges.py --callers`.

How to read the Ir/call column:

* **A million calls at single-digit Ir/call is pure call overhead**, and the
  only question is which kind. A non-generic `crabomination_base` callee is a
  *profile artifact* — `release`'s thin LTO inlines it, and `release-fast`
  (which `profiling-fast` inherits) does not, so `CardDefinition::is_creature`
  reads a million calls here and none in the shipped binary. A std generic the
  local inliner declined is *real*, and the fix is restructuring the call
  site, never an `#[inline]`.
* **A whole-board presence walk divides into card visits here**, which is what
  says whether its row is body or iteration: pass 89 found
  `creature_type_change_in_scope`'s closure at 27,794 calls x 642 Ir, i.e. ~20
  cards at ~22 Ir, and a memo on the per-card body moved it 1 %.
* **Three rows at the same call count with one an order of magnitude dearer**
  is the tell that the dear one is doing work the other two are not.

`cg_edges.py`'s tables are per *edge*; this folds them to the callee, so a
function reached from thirty sites appears once with its total.
"""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cg_edges  # noqa: E402


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    path = sys.argv[1]
    rows = int(sys.argv[2]) if len(sys.argv) > 2 else 40
    self_cost, _edge_cost, edge_calls, total, declared = cg_edges.parse(path)
    if declared is not None and declared != total:
        print(f"# WARNING: dump's own totals: line says {declared:,} — parse is incomplete")
    calls = collections.Counter()
    for (_caller, callee), n in edge_calls.items():
        calls[callee] += n
    print(f"# total Ir {total:,}")
    print(f"{'calls':>12} {'self Ir':>14} {'Ir/call':>9}  name")
    shown = calls.most_common(rows) if rows else calls.most_common()
    for name, n in shown:
        s = self_cost.get(name, 0)
        print(f"{n:>12,} {s:>14,} {s / n:>9.1f}  {name[:100]}")
    rest = len(calls) - len(shown)
    if rest:
        print(f"# ... {rest:,} more rows not shown")


if __name__ == "__main__":
    main()
