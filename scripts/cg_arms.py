#!/usr/bin/env python3
"""Which *arms* of a big `match` actually run, from a jump table and an
instruction-level callgrind dump.

`cg_edges.py` ranks functions, `cg_lines.py` ranks source lines and
`cg_contexts.py` splits callers — and none of them can answer "of this
205-arm dispatch, which arms are hot?". A jump-table `match` compiles to one
indirect branch, so every arm shares a line region and a caller; the only
thing that separates them is the *address* each table entry points at.

This reads the table out of the binary, then reads the Ir charged to each
target address out of a `--dump-instr=yes` dump. An instruction's Ir is its
execution count, so the count at an arm's first instruction **is** how many
times that arm was taken.

    RUST_MIN_STACK=33554432 valgrind --tool=callgrind --dump-instr=yes \\
      --callgrind-out-file=cg.instr.out target/profiling-fast/bot_ladder \\
      --a gang --b gang --games 6 --threads 1 --seed 1 --decks cube
    python3 scripts/cg_arms.py cg.instr.out target/profiling-fast/bot_ladder \\
      evaluate_requirement_static_hinted \\
      --variants crabomination_base/src/card.rs:SelectionRequirement

Recursion: callgrind gives a self-recursive function a second context named
`fn'2`, and every deeper level folds into it. Both are summed here and the
split is printed in the header, so "how much of this function is its own
recursion" is read off the same run.

**Three things this cannot see, and each is a wrong reading waiting to
happen.**

* **LLVM merges identical arms.** Several table entries land on one address,
  and that address's count is their *sum*. Merged groups print as one row
  with every name on it; do not divide.
* **An arm's first instruction is not the arm.** The count is entries into
  the arm, not its cost. A cheap arm taken a million times and a dear one
  taken a thousand times both read as their entry count — join against
  `cg_lines.py` for cost.
* **An arm whose body is a constant has no code**, so its table entry points
  straight at the shared epilogue and its row reads the *whole* call count.
  `SelectionRequirement::Any => true` prints 100 % for that reason. Rows at
  exactly 100 % are flagged `(epilogue?)`; read them as "not an arm".
* **The table is found by disassembling the dispatch**, i.e. by pattern, not
  by DWARF. The header prints the table address and entry count it decoded;
  if the count does not match the enum's variant count the pattern missed and
  the table is the wrong one.
"""
import argparse
import collections
import re
import struct
import subprocess
import sys

# `lea <disp>(%rip),%rX` ... `movslq (%rX,%rY,4),%rZ` — the rustc/LLVM
# jump-table dispatch on x86-64. The `cmp $N` just above it is the arm count.
LEA_RE = re.compile(r"^\s*([0-9a-f]+):\s+(?:[0-9a-f]{2} )+\s*lea\s+(-?0x[0-9a-f]+)\(%rip\)")
CMP_RE = re.compile(r"^\s*[0-9a-f]+:\s+(?:[0-9a-f]{2} )+\s*cmp\s+\$(0x[0-9a-f]+),")
MOVSLQ_RE = re.compile(r"movslq\s+\(%r[0-9a-z]+,%r[0-9a-z]+,4\)")


def symbol_range(binary, needle):
    """(start, end, name) of the one `nm` symbol whose name contains NEEDLE."""
    out = subprocess.run(
        ["nm", "-C", "--defined-only", "-S", binary], capture_output=True, text=True
    ).stdout
    hits = []
    for line in out.splitlines():
        parts = line.split(" ", 3)
        if len(parts) < 4 or needle not in parts[3]:
            continue
        if "{{closure}}" in parts[3] or "::" + needle + "::" in parts[3]:
            continue
        try:
            hits.append((int(parts[0], 16), int(parts[1], 16), parts[3].strip()))
        except ValueError:
            continue
    if not hits:
        sys.exit(f"no symbol matching {needle!r} in {binary}")
    hits.sort(key=lambda h: -h[1])
    start, size, name = hits[0]
    return start, start + size, name


def section_map(binary):
    """[(vaddr, size, file_off)] for the allocated sections, for vaddr->offset."""
    out = subprocess.run(["readelf", "-SW", binary], capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        m = re.match(r"\s*\[\s*\d+\]\s+(\S+)\s+(\S+)\s+([0-9a-f]+)\s+([0-9a-f]+)\s+([0-9a-f]+)", line)
        if m and m.group(2) != "NOBITS":
            rows.append((int(m.group(3), 16), int(m.group(5), 16), int(m.group(4), 16)))
    return rows


def vaddr_to_off(sections, vaddr):
    for base, size, off in sections:
        if base <= vaddr < base + size:
            return off + (vaddr - base)
    return None


def find_table(binary, start, end):
    """Decode the dispatch: returns (table_vaddr, n_entries)."""
    out = subprocess.run(
        ["objdump", "-d", f"--start-address={start}", f"--stop-address={min(end, start + 0x200)}",
         binary],
        capture_output=True, text=True,
    ).stdout
    lines = out.splitlines()
    for i, line in enumerate(lines):
        if not MOVSLQ_RE.search(line):
            continue
        for j in range(i - 1, max(i - 6, -1), -1):
            m = LEA_RE.match(lines[j])
            if not m:
                continue
            pc = int(m.group(1), 16)
            # objdump prints the instruction length via the byte column; the
            # rip-relative target is `next_pc + disp`, and objdump already
            # resolves it in the trailing `# <addr>` comment.
            tail = re.search(r"#\s*([0-9a-f]+)\s", lines[j])
            table = int(tail.group(1), 16) if tail else pc + int(m.group(2), 16)
            n = None
            for k in range(j - 1, max(j - 8, -1), -1):
                mc = CMP_RE.match(lines[k])
                if mc:
                    n = int(mc.group(1), 16) + 1
                    break
            return table, n
    return None, None


def read_table(binary, sections, table_vaddr, n):
    off = vaddr_to_off(sections, table_vaddr)
    if off is None:
        sys.exit(f"jump table vaddr {table_vaddr:#x} is in no section")
    with open(binary, "rb") as f:
        f.seek(off)
        raw = f.read(n * 4)
    return [table_vaddr + struct.unpack_from("<i", raw, i * 4)[0] for i in range(n)]


def variant_names(spec):
    """`path.rs:EnumName` -> the variant names in declaration order."""
    path, enum = spec.rsplit(":", 1)
    src = open(path).read().splitlines()
    out, depth, inside = [], 0, False
    for line in src:
        if not inside:
            if re.match(rf"^\s*pub enum {enum}\b", line) or re.match(rf"^\s*enum {enum}\b", line):
                inside = True
            continue
        if line.startswith("}"):
            break
        m = re.match(r"^    ([A-Z][A-Za-z0-9_]*)\s*[,({]", line)
        if m:
            out.append(m.group(1))
    return out


def dump_costs(dump, fn_needle):
    """addr -> Ir, summed over every context of the matching function.

    Returns (costs, per_context_entry_counts). The entry count of a context is
    the Ir at its lowest address, which is its first instruction.
    """
    ids, names = set(), {}
    for line in open(dump):
        m = re.match(r"c?fn=\((\d+)\)\s+(.+)", line)
        if m and fn_needle in m.group(2) and "{{closure}}" not in m.group(2):
            ids.add(m.group(1))
            names[m.group(1)] = m.group(2).strip()
    costs = collections.Counter()
    contexts = {}
    cur, addr, skip = None, 0, False
    for line in open(dump):
        line = line.rstrip("\n")
        if line.startswith("fn="):
            m = re.match(r"fn=\((\d+)\)", line)
            cur = m.group(1) if m and m.group(1) in ids else None
            addr, skip = 0, False
            continue
        if line.startswith("calls="):
            # The next cost line is the callee's whole subtree, not this
            # instruction's own count — see `cg_edges.py`'s docstring.
            skip = True
            continue
        if line.startswith(("cfn=", "cfi=", "cfl=", "fi=", "fe=", "fl=", "ob=")):
            continue
        if cur is None or not line or line[0] not in "0x+-*123456789":
            continue
        m = re.match(r"(0x[0-9a-f]+|\+\d+|-\d+|\*)\s+\S+\s+(\d+)", line)
        if not m:
            continue
        pos = m.group(1)
        if pos.startswith("0x"):
            addr = int(pos, 16)
        elif pos != "*":
            addr += int(pos)
        if skip:
            skip = False
            continue
        ir = int(m.group(2))
        costs[addr] += ir
        c = contexts.setdefault(cur, [addr, 0])
        if addr <= c[0]:
            c[0], c[1] = addr, ir
    return costs, {names[k]: v[1] for k, v in contexts.items()}


def sweep(dump, rows):
    """Rank every function by its *fixed* per-call cost.

    An instruction whose execution count equals the function's call count runs
    on every call: the prologue, the dispatch, the shared epilogue. Summed,
    that is what a caller pays before the body does anything, and it is the
    one component only *fewer calls* can move. Nothing else in this directory
    measures it.

    The entry instruction is taken to be the lowest address in the block, and
    its Ir to be the call count; a function whose cold blocks are laid out
    below its entry would read low, not high.
    """
    per = collections.defaultdict(collections.Counter)
    names, cur, addr, skip = {}, None, 0, False
    total = 0
    for line in open(dump, errors="replace"):
        line = line.rstrip("\n")
        if line.startswith("summary:") or line.startswith("totals:"):
            total = max(total, int(line.split(":", 1)[1].split()[0]))
            continue
        m = re.match(r"c?fn=\((\d+)\)(?:\s+(.*))?", line)
        if m:
            if m.group(2):
                names.setdefault(m.group(1), m.group(2))
            if line.startswith("fn="):
                cur, addr, skip = m.group(1), 0, False
            continue
        if line.startswith("calls="):
            skip = True
            continue
        if line.startswith(("cfn=", "cfi=", "cfl=", "fi=", "fe=", "fl=", "ob=", "cob=")):
            continue
        if cur is None or not line or line[0] not in "0x+-*123456789":
            continue
        m = re.match(r"(0x[0-9a-f]+|\+\d+|-\d+|\*)\s+\S+\s+(\d+)", line)
        if not m:
            continue
        pos = m.group(1)
        if pos.startswith("0x"):
            addr = int(pos, 16)
        elif pos != "*":
            addr += int(pos)
        if skip:
            skip = False
            continue
        per[cur][addr] += int(m.group(2))
    out = []
    for fid, costs in per.items():
        if not costs:
            continue
        calls = costs[min(costs)]
        if calls < 1000:
            continue
        fixed = [v for v in costs.values() if v == calls]
        out.append((sum(fixed), len(fixed), calls, sum(costs.values()), names.get(fid, fid)))
    out.sort(reverse=True)
    print(f"# fixed per-call cost, {total:,} Ir total run")
    print(f"  {'fixed Ir':>13} {'share':>7} {'n':>4} {'calls':>11} {'self Ir':>13} {'of self':>8}  name")
    for fixed, n, calls, self_ir, name in out[:rows]:
        print(
            f"  {fixed:>13,} {100.0 * fixed / total:6.3f}% {n:>4} {calls:>11,} "
            f"{self_ir:>13,} {100.0 * fixed / self_ir:7.1f}%  {name[:70]}"
        )
    rest = out[rows:]
    print(f"# ... {len(rest):,} more functions, {sum(r[0] for r in rest):,} fixed Ir between them")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("dump")
    ap.add_argument("binary", nargs="?")
    ap.add_argument("function", nargs="?")
    ap.add_argument("--variants", help="path.rs:EnumName for arm labels")
    ap.add_argument("--rows", type=int, default=40, help="0 for all")
    ap.add_argument(
        "--sweep",
        action="store_true",
        help="rank every function by fixed per-call cost instead of reading one match",
    )
    args = ap.parse_args()

    if args.sweep:
        sweep(args.dump, args.rows or 10**9)
        return
    if not args.binary or not args.function:
        sys.exit("need BINARY and FUNCTION unless --sweep")

    start, end, name = symbol_range(args.binary, args.function)
    table, n = find_table(args.binary, start, end)
    if table is None:
        sys.exit("no jump-table dispatch found in the first 512 bytes of the function")
    sections = section_map(args.binary)
    targets = read_table(args.binary, sections, table, n)
    labels = variant_names(args.variants) if args.variants else [str(i) for i in range(n)]
    costs, contexts = dump_costs(args.dump, args.function)

    total_entries = sum(contexts.values())
    print(f"{name}")
    print(f"  {start:#x}..{end:#x}  {end - start:,} bytes; table {table:#x}, {n} entries")
    for ctx, c in sorted(contexts.items(), key=lambda kv: -kv[1]):
        tag = "recursive" if ctx.rstrip("0123456789").endswith("'") else "outermost"
        print(f"  {c:>12,}  {tag:<9}  {ctx.split('::')[-1]}")
    print(f"  {total_entries:>12,}  calls, all contexts")
    if n != len(labels):
        print(f"  ⚠ table has {n} entries, {args.variants} has {len(labels)} variants")

    groups = collections.defaultdict(list)
    for i, t in enumerate(targets):
        groups[t].append(labels[i] if i < len(labels) else str(i))
    rows = sorted(((costs.get(t, 0), t, ns) for t, ns in groups.items()), reverse=True)
    print()
    print(f"  {'entries':>12}  {'share':>7}  arm (merged arms share one row)")
    shown = 0
    for ir, t, ns in rows:
        if args.rows and shown >= args.rows:
            break
        if ir == 0:
            break
        shown += 1
        pct = 100.0 * ir / total_entries if total_entries else 0.0
        flag = " (epilogue?)" if ir == total_entries else ""
        print(f"  {ir:>12,}  {pct:6.2f}%  {t:#x}{flag}  {' | '.join(ns)}")
    dropped = sum(ir for ir, _, _ in rows[shown:])
    cold = sum(1 for ir, _, _ in rows if ir == 0)
    print(f"  ... {dropped:,} entries in rows past the cap; {cold} arms never taken")

    # The instructions every call runs — prologue, dispatch, epilogue. On a
    # jump-table `match` this is the function's *fixed* per-call cost, and it
    # is the one component no arm-level change can touch: only calling the
    # function less often can.
    body = sum(v for a, v in costs.items() if start <= a < end)
    fixed = {a: v for a, v in costs.items() if start <= a < end and v == total_entries}
    if body and total_entries:
        print()
        print(
            f"  fixed per call: {len(fixed)} instructions run on every call = "
            f"{sum(fixed.values()):,} Ir, {100.0 * sum(fixed.values()) / body:.1f} % of the "
            f"function's {body:,} self Ir ({len(fixed)} Ir/call)"
        )


if __name__ == "__main__":
    main()
