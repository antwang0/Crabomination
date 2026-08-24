#!/usr/bin/env python3
"""Which *callers* a hot function's calls actually come from, two or three
frames up.

`cg_edges.py` gives one level: it says `computed_permanent` called
`gather_continuous_effects_inner` 20,374 times and stops there, which is the
level at which every caller looks the same. The question a freeze-scope
candidate actually asks is "whose 20,374", and no one-level table answers it
— the fifty-fourth pass spent a build and a callgrind run on a scope that
removed exactly zero gathers because it inferred the answer from Ir/call
instead.

Run callgrind with `--separate-callers=N` and every function gets one entry
per calling context of depth N; this reads those apart and sums the calls per
context.

    RUST_MIN_STACK=33554432 valgrind --tool=callgrind --separate-callers=3 \\
      --callgrind-out-file=cg.sc.out target-probe/profiling-fast/bot_ladder \\
      --a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed
    python3 scripts/cg_symbolize.py cg.sc.out \\
      target-probe/profiling-fast/bot_ladder > cg.sc.sym.out
    python3 scripts/cg_contexts.py cg.sc.sym.out gather_continuous_effects_inner

`--separate-callers=3` costs nothing in run time and roughly doubles the dump
size. The contexts print innermost-first (`callee <- caller <- caller`), and
the counts sum to the function's total call count, which is the check that
nothing was dropped.
"""

import collections
import re
import sys


def contexts(path, needle):
    """(context, calls, ir) for every calling context of `needle`."""
    calls = collections.Counter()
    ir = collections.Counter()
    names = {}
    cur = None
    pending = 0
    with open(path) as fh:
        for line in fh:
            line = line.rstrip("\n")
            m = re.match(r"^cfn=\((\d+)\)(?: (.*))?$", line)
            if m:
                if m.group(2):
                    names[m.group(1)] = m.group(2)
                cur = names.get(m.group(1), "")
                continue
            m = re.match(r"^calls=(\d+)", line)
            if m:
                pending = int(m.group(1))
                continue
            if pending and cur and line and line[0].isdigit():
                if cur.split("'", 1)[0].endswith(needle) or needle in cur.split("'", 1)[0]:
                    parts = line.split()
                    ctx = cur.split("'", 1)[1] if "'" in cur else "(no context)"
                    calls[ctx] += pending
                    ir[ctx] += int(parts[1]) if len(parts) > 1 else 0
                pending = 0
                continue
            m = re.match(r"^fn=\((\d+)\)(?: (.*))?$", line)
            if m and m.group(2):
                names[m.group(1)] = m.group(2)
    return calls, ir


def short(ctx):
    trim = (
        ("crabomination::", ""),
        ("<impl crabomination::game::GameState>::", ""),
        ("game::GameState::", ""),
    )
    out = []
    for frame in ctx.split("'"):
        for a, b in trim:
            frame = frame.replace(a, b)
        out.append(frame)
    return " <- ".join(out)


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    path, needle = sys.argv[1], sys.argv[2]
    rows = int(sys.argv[3]) if len(sys.argv) > 3 else 25
    calls, _ = contexts(path, needle)
    if not calls:
        sys.exit(
            f"no calls to *{needle}* with a context — was the dump taken with "
            "--separate-callers=N, and symbolized?"
        )
    total = sum(calls.values())
    print(f"# {total:,} calls to *{needle}*, by calling context")
    for ctx, n in calls.most_common(rows):
        print(f"{n:8,d}  {short(ctx)}")
    shown = sum(n for _, n in calls.most_common(rows))
    if shown < total:
        print(f"# {total - shown:,} calls in {len(calls) - rows} further contexts")


if __name__ == "__main__":
    main()
