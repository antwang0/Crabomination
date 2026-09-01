#!/usr/bin/env python3
"""Two levels of caller for a callgrind row: who calls X, and who calls them.

`cg_edges.py --callers` answers one level, which is not enough when the row is
a std generic: the immediate caller is another std generic (`Vec::from_iter`,
`SmallVec::extend`) and the engine function that owns the cost is one hop
further up. Chaining `--callers` by hand fails on a **demangled** dump because
callgrind merges every monomorphization of `Vec::from_iter` into one row, so
the second hop cannot tell which of them fed the first.

    RUST_MIN_STACK=33554432 valgrind --tool=callgrind --demangle=no \\
      --callgrind-out-file=cg.nd target/profiling-fast/bot_ladder \\
      --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
    python3 scripts/cg_chain.py cg.nd 'FnMut'           # the shim (-156) found
    python3 scripts/cg_chain.py cg.nd '__rust_alloc' --top 6 --up 4

`--demangle=no` is required, not optional: the whole point is that the
intermediate rows separate by their legacy-mangling hash suffix. Names are
matched as substrings against the mangled symbol, so `FnMut`, `from_iter` and
a bare `17h<hash>E` all work.

Costs are callgrind's own: `calls` is the edge's call count and `Ir` is
inclusive of the callee's subtree.
"""
import re
import sys


def parse(path):
    """(names, edges) — edges[(caller_id, callee_id)] = [calls, inclusive Ir]."""
    names, edges = {}, {}
    cur = pend = None
    with open(path) as fh:
        for line in fh:
            line = line.rstrip("\n")
            m = re.match(r"^fn=\((\d+)\)(?: (.*))?$", line)
            if m:
                if m.group(2):
                    names[m.group(1)] = m.group(2)
                cur, pend = m.group(1), None
                continue
            m = re.match(r"^cfn=\((\d+)\)(?: (.*))?$", line)
            if m:
                if m.group(2):
                    names[m.group(1)] = m.group(2)
                pend = m.group(1)
                continue
            m = re.match(r"^calls=(\d+)", line)
            if m and pend is not None:
                edges.setdefault((cur, pend), [0, 0])[0] += int(m.group(1))
                continue
            if pend is not None and re.match(r"^[\d+*-]", line):
                parts = line.split()
                if len(parts) >= 2:
                    try:
                        edges[(cur, pend)][1] += int(parts[1])
                    except ValueError:
                        pass
                pend = None
    return names, edges


def callers_of(edges, ids):
    out = []
    for (a, b), (calls, ir) in edges.items():
        if b in ids:
            out.append((calls, ir, a))
    out.sort(reverse=True)
    return out


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    path, needle = sys.argv[1], sys.argv[2]
    top = int(sys.argv[sys.argv.index("--top") + 1]) if "--top" in sys.argv else 4
    up = int(sys.argv[sys.argv.index("--up") + 1]) if "--up" in sys.argv else 3
    names, edges = parse(path)
    seed = {i for i, n in names.items() if needle in n}
    if not seed:
        sys.exit(f"no symbol matching {needle!r} — is the dump --demangle=no?")
    total = sum(c for c, _, _ in callers_of(edges, seed))
    print(f"# {len(seed)} symbols match {needle!r}; {total:,} incoming calls")
    for calls, ir, a in callers_of(edges, seed)[:top]:
        print(f"\n{calls:>9,} calls {ir:>14,} Ir  {names.get(a, '?')[:100]}")
        for c2, ir2, b in callers_of(edges, {a})[:up]:
            print(f"    <- {c2:>7,} {ir2:>12,}  {names.get(b, '?')[:96]}")


if __name__ == "__main__":
    main()
