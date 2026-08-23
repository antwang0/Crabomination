#!/usr/bin/env python3
"""Put function names back into a callgrind dump.

Valgrind 3.22 in the routine image does not read the symbol table of
`target/profiling-fast/bot_ladder` at all (`-v` never prints "Reading syms
from" for it), so every frame in `cg.out` is named `0x<addr>` and
`callgrind_annotate` prints `???:0x…`. The addresses are the ELF vaddrs plus
valgrind's PIE load base (0x108000 on amd64), so the symbol table resolves
them exactly once the base is subtracted. The base is auto-detected by
picking the bias that lands the most addresses inside a `FUNC` symbol.

    python3 scripts/cg_symbolize.py cg.out target/profiling-fast/bot_ladder \
        > cg.sym.out
    callgrind_annotate --auto=no --threshold=95 cg.sym.out
    callgrind_annotate --auto=no --tree=caller --threshold=99 cg.sym.out

Line-level (`--auto=yes`) annotation still needs DWARF valgrind can read and
is not restored by this. Function totals, call counts and `--tree=caller` /
`--tree=calling` all work.
"""
import bisect
import re
import subprocess
import sys


def load_syms(binary):
    out = subprocess.run(
        ["readelf", "-sW", binary], capture_output=True, text=True, check=True
    ).stdout
    syms = {}
    for line in out.splitlines():
        f = line.split()
        if len(f) < 8 or f[3] != "FUNC":
            continue
        try:
            addr = int(f[1], 16)
            size = int(f[2])
        except ValueError:
            continue
        if addr == 0:
            continue
        name = f[7].split("@")[0]
        # Prefer the entry with a real size; ties keep the first seen.
        prev = syms.get(addr)
        if prev is None or (prev[0] == 0 and size > 0):
            syms[addr] = (size, name)
    items = sorted((a, s, n) for a, (s, n) in syms.items())
    return [i[0] for i in items], items


def demangle(names):
    """Batch-demangle through c++filt; legacy Rust mangling is C++-compatible."""
    if not names:
        return {}
    p = subprocess.run(
        ["c++filt"], input="\n".join(names), capture_output=True, text=True
    )
    if p.returncode != 0:
        return {n: n for n in names}
    out = p.stdout.splitlines()
    if len(out) != len(names):
        return {n: n for n in names}
    return dict(zip(names, out))


HEX = re.compile(r"^(c?fn=\(\d+\)\s+)(0x[0-9a-f]+)(.*)$")


BIASES = (0x108000, 0, 0x400000)


def main():
    if len(sys.argv) not in (3, 4):
        sys.exit(__doc__)
    cg, binary = sys.argv[1], sys.argv[2]
    addrs, items = load_syms(binary)

    def inside(a):
        i = bisect.bisect_right(addrs, a) - 1
        if i < 0:
            return None
        base, size, sym = items[i]
        return sym if size and a < base + size else None

    lines = open(cg, errors="replace").read().splitlines()
    seen = sorted({m.group(2) for m in map(HEX.match, lines) if m})

    if len(sys.argv) == 4:
        bias = int(sys.argv[3], 0)
    else:
        sample = [int(h, 16) for h in seen]
        bias = max(BIASES, key=lambda b: sum(inside(a - b) is not None for a in sample))
        hits = sum(inside(a - bias) is not None for a in sample)
        print(f"# bias 0x{bias:x}, {hits}/{len(sample)} addresses in a FUNC", file=sys.stderr)

    cache = {}

    def resolve(hexaddr):
        if hexaddr in cache:
            return cache[hexaddr]
        a = int(hexaddr, 16) - bias
        i = bisect.bisect_right(addrs, a) - 1
        name = hexaddr
        if i >= 0:
            base, size, sym = items[i]
            if size and a < base + size:
                name = sym
        cache[hexaddr] = name
        return name

    resolved = [resolve(m.group(2)) for m in map(HEX.match, lines) if m]
    names = demangle(sorted(set(resolved)))

    out = []
    for line in lines:
        m = HEX.match(line)
        if m:
            sym = resolve(m.group(2))
            # Keep the `'2` recursion suffix callgrind appends.
            out.append(f"{m.group(1)}{names.get(sym, sym)}{m.group(3)}")
        else:
            out.append(line)
    sys.stdout.write("\n".join(out) + "\n")


if __name__ == "__main__":
    main()
