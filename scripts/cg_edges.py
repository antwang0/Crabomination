#!/usr/bin/env python3
"""Aggregate a callgrind dump into self costs and call edges.

`callgrind_annotate --tree` truncates caller lists at its threshold, which
silently drops the rows that matter for an allocation hunt (the forty-eighth
pass lost `finish_grow` and `finalize_cast` out of `__rust_alloc`'s caller
block that way). This reads the dump directly, so a table is complete.

    python3 scripts/cg_symbolize.py cg.out target/profiling-fast/bot_ladder \
        > cg.sym.out
    python3 scripts/cg_edges.py cg.sym.out                # top self costs
    python3 scripts/cg_edges.py cg.sym.out --callers __rust_alloc
    python3 scripts/cg_edges.py cg.sym.out --callees finalize_cast
    python3 scripts/cg_edges.py cg.sym.out --callers __rust_alloc --rows 0

Names match on substring, so `--callers grow_one` folds every
monomorphization. Edge costs are inclusive of the callee's subtree, as
callgrind records them; rank an allocation table by *calls*, not Ir.

The *parse* is complete; the *printed table* is capped at `--rows` (default
40, `0` for all) and `show` says what the cap left out. The docstring's own
"so a table is complete" above is about the parse and used to read as a claim
about the print — which is the same failure as the `--tree` truncation this
script was written to escape, and is what the nineteenth robustness filter
found here.

The program total is the sum of the *self* lines only. Summing every cost
line adds each call edge's whole subtree on top and lands ~18x high, which
is what this script did until the forty-ninth pass: it printed 28.1 G for a
1.54 G run, so every share it reported was an order out (`dispatch_triggers
_for_events` read 0.30 % where it is 5.5 %). The header now cross-checks the
sum against the dump's own `totals:` line and says so when they disagree.
"""
import collections
import re
import sys


def parse(path):
    self_cost = collections.Counter()
    edge_cost = collections.Counter()
    edge_calls = collections.Counter()
    names = {}
    declared = None
    cur = None
    pend = None
    pend_calls = 0
    total = 0
    for line in open(path, errors="replace"):
        if line.startswith("fn="):
            m = re.match(r"fn=\((\d+)\)(?:\s+(.*))?", line.rstrip("\n"))
            if m:
                if m.group(2):
                    names[m.group(1)] = m.group(2)
                cur = names.get(m.group(1), m.group(1))
            pend = None
            continue
        if line.startswith("cfn="):
            m = re.match(r"cfn=\((\d+)\)(?:\s+(.*))?", line.rstrip("\n"))
            if m:
                if m.group(2):
                    names[m.group(1)] = m.group(2)
                pend = names.get(m.group(1), m.group(1))
            continue
        if line.startswith("calls="):
            pend_calls = int(line.split("=", 1)[1].split()[0])
            continue
        if line and (line[0].isdigit() or line[0] in "+-*"):
            parts = line.split()
            if len(parts) < 2:
                continue
            try:
                cost = int(parts[1])
            except ValueError:
                continue
            if pend is not None:
                # A call edge: `cost` is the callee's whole subtree, so it
                # must not join the program total (see the module docstring).
                edge_cost[(cur, pend)] += cost
                edge_calls[(cur, pend)] += pend_calls
                pend = None
            else:
                self_cost[cur] += cost
                total += cost
            continue
        if line.startswith(("fl=", "fi=", "fe=", "ob=", "cob=", "cfi=", "cfl=")):
            continue
        if line.startswith("totals:"):
            declared = int(line.split(":", 1)[1].split()[0])
            continue
    return self_cost, edge_cost, edge_calls, total, declared


def fold(counter_cost, counter_calls, key_idx, needle):
    """Sum edges whose other end matches `needle`, grouped by the near end."""
    cost = collections.Counter()
    calls = collections.Counter()
    for k, v in counter_cost.items():
        if needle in k[1 - key_idx]:
            cost[k[key_idx]] += v
            calls[k[key_idx]] += counter_calls[k]
    return cost, calls


def show(rows, total, fmt, limit=40):
    """Print at most `limit` rows and say what was left out.

    A table that stops at its limit without saying so reads as complete, which
    is the failure that cost the forty-eighth pass two fictional allocator
    leads (`callgrind_annotate --tree` truncating its caller block). PERF's
    "no silent caps" rule is written for workflows; the nineteenth robustness
    filter extended it to the tools.
    """
    for i, (name, v) in enumerate(rows):
        if i >= limit:
            rest = rows[limit:]
            print(
                f"# ... {len(rest):,} more rows not shown, "
                f"{sum(x[1] for x in rest):,} between them"
                f"{f' ({100 * sum(x[1] for x in rest) / total:.2f}%)' if total else ''}"
            )
            return
        print(fmt(name, v))


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    argv = sys.argv[1:]
    # `--rows 0` prints the whole ranked list. A cap that cannot be lifted
    # still hides its tail from the one reader who went looking for it.
    rows = None
    if "--rows" in argv:
        i = argv.index("--rows")
        rows = int(argv[i + 1])
        del argv[i : i + 2]
    path = argv[0]
    mode = argv[1] if len(argv) > 1 else None
    needle = argv[2] if len(argv) > 2 else None
    self_cost, edge_cost, edge_calls, total, declared = parse(path)

    def cap(default):
        """The row limit for one table: `--rows` if given, else its default."""
        if rows is None:
            return default
        return len(self_cost) + len(edge_cost) if rows == 0 else rows

    print(f"# total Ir {total:,}")
    if declared is not None and declared != total:
        print(f"# WARNING: dump's own totals: line says {declared:,} — parse is incomplete")
    if mode in ("--callers", "--callees"):
        # --callers X: rows are callers (near end 0). --callees X: rows are callees.
        idx = 0 if mode == "--callers" else 1
        cost, calls = fold(edge_cost, edge_calls, idx, needle)
        print(f"# {mode[2:]} of *{needle}*: {sum(calls.values()):,} calls")
        # Ranked by *calls*, so the "rows not shown" tally counts calls too.
        show(
            calls.most_common(),
            sum(calls.values()),
            lambda name, c: f"{c:>9,} {cost[name]:>14,}  {name[:110]}",
            limit=cap(40),
        )
    elif mode == "--inclusive":
        inc = collections.Counter()
        for (caller, callee), v in edge_cost.items():
            if needle is None or needle in callee:
                inc[callee] += v
        show(
            inc.most_common(),
            None,
            lambda name, v: f"{v:>14,} ({100 * v / total:5.2f}%)  {name[:110]}",
            limit=cap(40),
        )
    else:
        show(
            self_cost.most_common(),
            total,
            lambda name, v: f"{v:>14,} ({100 * v / total:5.2f}%)  {name[:110]}",
            limit=cap(45),
        )


if __name__ == "__main__":
    main()
