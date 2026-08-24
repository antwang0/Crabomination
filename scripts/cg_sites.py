#!/usr/bin/env python3
"""What an *inlined* function costs, and at which call sites.

`cg_edges.py` ranks functions and `cg_lines.py` ranks source lines, and
between them they miss the shape that hides the most: a small function that
is always inlined has **no row of its own** and no line of its own — its cost
is scattered across every caller. `battlefield_find` is 556 call sites of
`self.battlefield.iter().find(|c| c.id == id)`; it never appeared in fifty-two
passes of profiles, and it is **4.03 % of the simulator**.

This reads the inline chain `addr2line -i` gives for every hot address, keeps
the ones whose chain mentions NEEDLE anywhere, and groups them by the frame
*just outside* NEEDLE — i.e. by call site.

    RUST_MIN_STACK=33554432 valgrind --tool=callgrind --dump-instr=yes \\
      --callgrind-out-file=cg.instr.out target-probe/profiling-lines/bot_ladder \\
      --a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed
    python3 scripts/cg_sites.py cg.instr.out \\
      target-probe/profiling-lines/bot_ladder battlefield_find

`--dump-instr=yes` and the `profiling-lines` profile are both required, for
the reasons `cg_lines.py`'s docstring gives (DWARF packed into the binary,
instruction subpositions in the dump). Several needles at once: comma-separate
them.

**Read the number as a floor, not an estimate** (fifty-third pass). Each
address is charged only its own instructions, so a scan's per-element loads
and `Arc` deref land in `slice::iter`'s frames rather than here: the two
`battlefield_find` sites this found at 0.35 % between them measured
**-0.611 %** when they were removed.
"""
import collections
import os
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cg_lines as C  # noqa: E402


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    cg, binary, needles = sys.argv[1], sys.argv[2], sys.argv[3].split(",")
    cost, total_seen = C.parse_instr(cg, binary)
    ranked = cost.most_common()
    hot = [a for a, _ in ranked[:60000]]
    dropped = sum(v for _, v in ranked[60000:])
    total = sum(cost.values())
    bias, hit, tried = C.pick_bias(binary, hot)
    if hit * 2 < tried:
        sys.exit(f"{binary}: fewer than half the hot addresses resolve at any known bias")
    print(
        f"# {total:,} Ir resolved in {binary} ({100 * total / max(total_seen, 1):.1f}% of the "
        f"run's {total_seen:,}); bias 0x{bias:x}, {hit}/{tried} sampled addresses in a symbol"
    )
    if dropped:
        print(
            f"# cap: {len(ranked) - 60000:,} colder addresses unresolved, {dropped:,} Ir "
            f"({100 * dropped / total:.1f}%) — a site under the cap is missing from every table below"
        )
    frames = C.resolve(binary, hot, bias)
    for needle in needles:
        sites = collections.Counter()
        tot = 0
        for a in hot:
            fr = frames.get(a) or []
            # Outermost frame that still names the needle: the needle may
            # inline into an inliner that also names it (recursion, a `'2`
            # monomorphization), and the call site is one frame further out.
            idx = None
            for i, f in enumerate(fr):
                if needle in (f[0] or ""):
                    idx = i
            if idx is None:
                continue
            tot += cost[a]
            out = fr[idx + 1] if idx + 1 < len(fr) else fr[idx]
            loc = (out[1] or "??").split("/")[-1]
            sites[f"{loc:<24} {(out[0] or '?')[:72]}"] += cost[a]
        print(f"\n### {needle}: {tot:,} Ir ({100 * tot / total:.2f}% of the run)")
        rows = sites.most_common()
        for name, v in rows[:25]:
            print(f"{v:>12,} ({100 * v / total:5.2f}%)  {name}")
        rest = rows[25:]
        if rest:
            r = sum(v for _, v in rest)
            print(f"# ... {len(rest):,} more sites, {r:,} Ir between them ({100 * r / total:.2f}%)")


if __name__ == "__main__":
    main()
