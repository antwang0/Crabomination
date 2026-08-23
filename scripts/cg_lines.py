#!/usr/bin/env python3
"""Per-source-line annotation of a callgrind dump, without valgrind's DWARF.

`callgrind_annotate --auto=yes` is dead in the routine image (valgrind never
reads the binary's debug info — see `cg_symbolize.py`), which costs the
"read the *file:function* rows, not just function totals" method PERF.md
leans on. `.debug_line` and the DWARF5 skeleton units *are* in the binary
though, so `addr2line` resolves an instruction address to file:line and to
the inline chain. Run callgrind with instruction positions and annotate here:

    RUST_MIN_STACK=33554432 valgrind --tool=callgrind --dump-instr=yes \\
      --callgrind-out-file=cg.instr.out target/profiling-fast/bot_ladder \\
      --a gang --b gang --games 6 --threads 1 --seed 1 --decks fixed
    python3 scripts/cg_lines.py cg.instr.out target/profiling-fast/bot_ladder
    python3 scripts/cg_lines.py cg.instr.out target/profiling-fast/bot_ladder \\
        --in dispatch_triggers_for_events

Costs are *self* cost per line (callgrind's per-instruction counts), so a
line that calls something shows only its own dispatch. `--in NEEDLE` keeps
only addresses whose inline chain mentions NEEDLE, which is how to read one
function's body when everything around it is inlined.

`--dump-instr=yes` is not optional: without it the dump carries no `instr`
subposition and this script has nothing to read. It used to print
"0 instruction addresses, 0 Ir" and exit 0 on such a dump, which reads as
"nothing is hot" rather than "wrong input"; it now refuses at the
`positions:` header.
"""
import collections
import re
import subprocess
import sys

BIAS = 0x108000


def parse_instr(path):
    """Sum Ir per absolute instruction address.

    `positions: instr line` puts two subpositions on every cost line; with no
    DWARF the line half is always 0, so only the instruction one is read. An
    `instr` subposition is hexadecimal, absolute (`0x…`), relative (`+n`/`-n`)
    or `*` for unchanged.
    """
    cost = collections.Counter()
    addr = 0
    in_call = False
    positions = None
    for line in open(path, errors="replace"):
        if line.startswith("positions:"):
            positions = line.split(":", 1)[1].split()
            if "instr" not in positions:
                sys.exit(
                    f"{path}: `positions: {' '.join(positions)}` — this dump has no "
                    "instruction subpositions, so there is nothing to annotate. "
                    "Re-run callgrind with `--dump-instr=yes`."
                )
            continue
        if line.startswith("calls="):
            in_call = True
            continue
        c = line[:1]
        if not (c.isdigit() or c in "+-*"):
            in_call = False
            continue
        parts = line.split()
        if len(parts) < 3:
            in_call = False
            continue
        pos = parts[0]
        try:
            if pos == "*":
                pass
            elif pos[0] in "+-":
                addr += int(pos, 16)
            else:
                addr = int(pos, 16)
            ir = int(parts[2])
        except ValueError:
            in_call = False
            continue
        # A cost line right after `calls=` is the callee's inclusive cost at
        # the call site, not this instruction's own work.
        if not in_call:
            cost[addr] += ir
        in_call = False
    return cost


def resolve(binary, addrs):
    """addr2line the whole set at once; returns addr -> [(func, file:line), …]."""
    inp = "\n".join(f"0x{a - BIAS:x}" for a in addrs)
    out = subprocess.run(
        ["addr2line", "-e", binary, "-f", "-C", "-i", "-a"],
        input=inp,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    res = {}
    cur = None
    frames = []
    for line in out:
        if line.startswith("0x"):
            if cur is not None:
                res[cur] = frames
            cur = int(line, 16) + BIAS
            frames = []
        elif frames is not None and cur is not None:
            if len(frames) and frames[-1][1] is None:
                frames[-1] = (frames[-1][0], line)
            else:
                frames.append((line, None))
    if cur is not None:
        res[cur] = frames
    return res


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    cg, binary = sys.argv[1], sys.argv[2]
    needle = None
    if "--in" in sys.argv:
        needle = sys.argv[sys.argv.index("--in") + 1]
    cost = parse_instr(cg)
    total = sum(cost.values())
    if total == 0:
        sys.exit(f"{cg}: no instruction costs parsed — is this a callgrind dump?")
    print(f"# {len(cost):,} instruction addresses, {total:,} Ir")
    # Resolving every address is slow and pointless; the tail is noise.
    hot = [a for a, _ in cost.most_common(60000)]
    frames = resolve(binary, hot)
    lines = collections.Counter()
    for a in hot:
        fr = frames.get(a) or [("??", "??")]
        if needle and not any(needle in (f[0] or "") for f in fr):
            continue
        # addr2line -i prints the innermost inline frame first. Grouping by it
        # buries everything in `copy_nonoverlapping` / `dealloc`; group by the
        # *outermost* frame — the real function — and keep the innermost's
        # file:line as the "what it is doing there" column.
        func = fr[-1][0]
        loc = (fr[0][1] or "??").replace("/rustc/", "rustc:").split("/")[-1]
        lines[f"{loc:<28} {func[:88]}"] += cost[a]
    shown = sum(lines.values())
    print(f"# {shown:,} Ir shown ({100 * shown / max(total, 1):.1f}%)")
    for name, v in lines.most_common(45):
        print(f"{v:>12,} ({100 * v / total:5.2f}%)  {name}")


if __name__ == "__main__":
    main()
