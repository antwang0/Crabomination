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

Both listings say what they left out: the 60,000-address resolution cap and
the 45-row print are reported with the Ir they dropped, so a table that stops
at its limit cannot read as a complete one. `--rows N` (`0` = no cap) lifts
the print cap, which is what a fallback-chain read needs — a chain of narrow
passes shows up as a run of *identically costed* rows well below row 45, and
the default print cannot see it.

**Two things this script got wrong until the fiftieth pass, and both produced
a plausible table rather than an error.** It summed the instruction addresses
of *every* object the process mapped — libc, ld.so, libm, libgcc, valgrind's
preloads, 16.5 % of the run — and resolved them against this binary's DWARF;
and it hardcoded a `0x108000` load bias where the `profiling-lines` binary
needs `0`. Together they put 36 % of the run under `??` and gave the rest
whichever Rust symbol sat at the wrong offset: `Effect::clone` read
**35,279,138 Ir / 2.65 %** against the **0.5 M** its own call edges account
for. The `--tree`-truncation note in PERF that blamed lld's identical-code
folding for a bogus `drift::sort` row is more likely to have been this.
It now keeps only the annotated binary's object, picks the bias that resolves
the most addresses, prints the hit rate, and **refuses** below half.

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

# Candidate PIE load biases, as `cg_symbolize.py` uses. Which one a run needs
# is not fixed — a hardcoded 0x108000 resolved a third of this binary's hot
# addresses to `??` and silently mapped the rest onto the wrong symbols.
BIASES = (0, 0x108000, 0x400000)


def parse_instr(path, binary):
    """Sum Ir per absolute instruction address **of `binary` only**.

    `positions: instr line` puts two subpositions on every cost line; with no
    DWARF the line half is always 0, so only the instruction one is read. An
    `instr` subposition is hexadecimal, absolute (`0x…`), relative (`+n`/`-n`)
    or `*` for unchanged.

    A dump spans every object the process mapped — libc, ld.so, libm, libgcc,
    valgrind's own preloads — and their addresses mean nothing against this
    binary's DWARF. `ob=(N) path` names an object and a bare `ob=(N)` switches
    back to one already named, so the id of the annotated binary is tracked and
    every other object's cost is dropped. It is still *counted* into the
    running relative position, because `+n`/`-n` subpositions chain through the
    lines being skipped. Folding them in instead put 36 % of the run under
    `??` and gave the rest whichever Rust symbol happened to sit at the same
    offset — `Effect::clone` read 35 M there against the 0.5 M its own call
    edges account for.
    """
    import os

    want = os.path.realpath(binary)
    cost = collections.Counter()
    names = {}
    ours = None
    cur_is_ours = True
    total_seen = 0
    addr = 0
    in_call = False
    positions = None
    for line in open(path, errors="replace"):
        if line.startswith("ob="):
            m = re.match(r"ob=\((\d+)\)(?:\s+(.*))?", line.rstrip("\n"))
            if m:
                oid = m.group(1)
                if m.group(2):
                    names[oid] = m.group(2).strip()
                    if os.path.realpath(names[oid]) == want:
                        ours = oid
                cur_is_ours = oid == ours
            continue
        if line.startswith("cob="):
            # A *callee's* object; it names ids too, and the binary's own id is
            # often introduced here first.
            m = re.match(r"cob=\((\d+)\)(?:\s+(.*))?", line.rstrip("\n"))
            if m and m.group(2):
                names[m.group(1)] = m.group(2).strip()
                if os.path.realpath(m.group(2).strip()) == want:
                    ours = m.group(1)
            continue
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
            total_seen += ir
            if cur_is_ours:
                cost[addr] += ir
        in_call = False
    if ours is None:
        sys.exit(
            f"{path}: no `ob=` record names {binary} — this dump is of a "
            "different binary, so every address would resolve to the wrong "
            "symbol. Annotate the binary the dump was taken from."
        )
    return cost, total_seen


def pick_bias(binary, addrs):
    """The load bias that resolves the most of `addrs` to a real symbol.

    `cg_symbolize.py` has always auto-detected this; this script hardcoded
    0x108000 and never said how well it worked. On the `profiling-lines`
    binary the right answer is 0, and the wrong one left a third of the hot
    addresses at `??` while quietly mapping the rest onto neighbouring
    symbols. Returns `(bias, resolved, tried)` so the caller can report it and
    refuse a bad one.
    """
    sample = addrs[:400]
    best = (0, -1)
    for bias in BIASES:
        out = subprocess.run(
            ["addr2line", "-e", binary, "-f", "-C"],
            input="\n".join(f"0x{a - bias:x}" for a in sample),
            capture_output=True,
            text=True,
        ).stdout.splitlines()
        hit = sum(1 for n in out[0::2] if n.strip() != "??")
        if hit > best[1]:
            best = (bias, hit)
    return best[0], best[1], len(sample)


def resolve(binary, addrs, bias):
    """addr2line the whole set at once; returns addr -> [(func, file:line), …]."""
    inp = "\n".join(f"0x{a - bias:x}" for a in addrs)
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
            cur = int(line, 16) + bias
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
    rows = 45
    if "--rows" in sys.argv:
        rows = int(sys.argv[sys.argv.index("--rows") + 1])
        if rows <= 0:
            rows = None
    cost, total_seen = parse_instr(cg, binary)
    total = sum(cost.values())
    if total == 0:
        sys.exit(f"{cg}: no instruction costs parsed — is this a callgrind dump?")
    print(
        f"# {len(cost):,} instruction addresses in {binary}, {total:,} Ir "
        f"({100 * total / max(total_seen, 1):.1f}% of the run's {total_seen:,}; "
        "the rest is libc, ld.so and the other mapped objects)"
    )
    # Resolving every address is slow and pointless; the tail is noise. Say how
    # much of the program the cap left behind rather than implying it is all
    # here — the nineteenth robustness filter's rule for every capped listing.
    ranked = cost.most_common()
    hot = [a for a, _ in ranked[:60000]]
    dropped = sum(v for _, v in ranked[60000:])
    if dropped:
        print(
            f"# cap: {len(ranked) - 60000:,} colder addresses unresolved, "
            f"{dropped:,} Ir ({100 * dropped / total:.1f}%)"
        )
    bias, hit, tried = pick_bias(binary, hot)
    print(f"# load bias 0x{bias:x}, {hit}/{tried} sampled addresses in a symbol")
    if hit * 2 < tried:
        sys.exit(
            f"{binary}: fewer than half the hot addresses resolve at any known "
            f"bias ({BIASES}). Annotating anyway would attribute cost to "
            "whatever symbol sits at the wrong offset; fix the bias first."
        )
    frames = resolve(binary, hot, bias)
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
    print(f"# {shown:,} Ir grouped ({100 * shown / max(total, 1):.1f}% of the run)")
    ranked_lines = lines.most_common()
    for name, v in ranked_lines[:rows]:
        print(f"{v:>12,} ({100 * v / total:5.2f}%)  {name}")
    rest = ranked_lines[rows:] if rows else []
    if rest:
        rest_ir = sum(v for _, v in rest)
        print(
            f"# ... {len(rest):,} more lines not shown, {rest_ir:,} Ir between "
            f"them ({100 * rest_ir / total:.2f}%)"
        )


if __name__ == "__main__":
    main()
