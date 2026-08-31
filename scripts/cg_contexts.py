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
    """(calls, ir, program_total) for every calling context of `needle`.

    `ir` is the *inclusive* cost callgrind charges to the call, so a context's
    row is what removing that call site would remove — which is the number a
    candidate's ceiling is read off.
    """
    calls = collections.Counter()
    ir = collections.Counter()
    total = 0
    names = {}
    cur = None
    pending = 0
    # See `cg_edges.parse`: `positions: instr line` puts two position columns
    # ahead of the counts, and the first of them can be `*` or `+n`, neither
    # of which `str.isdigit` accepts. Reading column 1 as the cost, or gating
    # the line on a leading digit, drops the edge and leaves `pending` armed.
    positions = 1
    with open(path) as fh:
        for line in fh:
            line = line.rstrip("\n")
            if line.startswith("positions:"):
                positions = len(line.split(":", 1)[1].split()) or 1
                continue
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
            # A `calls=` line is always followed by exactly one cost line, and
            # its position column is `N`, `*`, `+N` or `-N` — subposition
            # compression, which this read as "not a cost line" until the
            # hundred-and-thirteenth pass. It then left `pending` armed and
            # charged the call to whichever later line happened to start with
            # a digit, under-counting `event_matches_spec` by 28x.
            #
            # And the cost is column `positions`, not column 1: an instruction
            # dump has *two* position columns, so column 1 there is the second
            # position, not a count.
            if pending and cur and line and (line[0].isdigit() or line[0] in "+-*"):
                parts = line.split()
                if len(parts) > positions and needle in cur.split("'", 1)[0]:
                    ctx = cur.split("'", 1)[1] if "'" in cur else "(no context)"
                    calls[ctx] += pending
                    try:
                        ir[ctx] += int(parts[positions])
                    except ValueError:
                        pass
                pending = 0
                continue
            m = re.match(r"^fn=\((\d+)\)(?: (.*))?$", line)
            if m and m.group(2):
                names[m.group(1)] = m.group(2)
                continue
            m = re.match(r"^summary: (\d+)", line)
            if m:
                total = int(m.group(1))
    return calls, ir, total


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
    calls, ir, program = contexts(path, needle)
    if not calls:
        sys.exit(
            f"no calls to *{needle}* with a context — was the dump taken with "
            "--separate-callers=N, and symbolized?"
        )
    total = sum(calls.values())
    total_ir = sum(ir.values())
    pct = f" = {100 * total_ir / program:.2f} % of the program" if program else ""
    print(
        f"# {total:,} calls to *{needle}*, {total_ir:,} inclusive Ir{pct}, "
        "by calling context (ranked by Ir)"
    )
    ranked = sorted(ir.items(), key=lambda kv: -kv[1])[:rows]
    for ctx, cost in ranked:
        share = f"{100 * cost / program:5.2f} %" if program else "      "
        print(f"{calls[ctx]:8,d}  {cost:12,d}  {share}  {short(ctx)}")
    shown_calls = sum(calls[c] for c, _ in ranked)
    if shown_calls < total:
        print(
            f"# {total - shown_calls:,} calls / "
            f"{total_ir - sum(c for _, c in ranked):,} Ir "
            f"in {len(calls) - len(ranked)} further contexts"
        )


if __name__ == "__main__":
    main()
