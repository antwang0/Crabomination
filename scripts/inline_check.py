#!/usr/bin/env python3
"""Is this callgrind row REAL, or a `release-fast` inlining artifact?

    python3 scripts/inline_check.py ManaCost::cmc CardInstance::power ...
    python3 scripts/inline_check.py --release target/release/bot_ladder \
        --fast target/profiling-fast/bot_ladder <symbols...>

PERF's standing trap: `profiling-fast` inherits `release-fast`, which is
`codegen-units = 16` with **no LTO**, so a callee the shipped `release` binary
(cgu 1 + thin LTO) inlines away still shows up here as a row with its full call
count. Ranking such a row buys the shipped binary nothing — an `#[inline]` on
it "wins" every A/B in PERF.md and changes nothing that ships.

The trap was filed for non-generic `crabomination_base` leaves. It is wider
than that: `LocalKey<T>::with` read 148,514 calls / 0.67 % of `cube` with
`FnOnce::call_once` under it, and `release` inlines the whole thread-local
access into `cp_pool::alloc` (115 instructions, zero such calls, 12 direct
`%fs:` references). **A std generic is exactly as much of an artifact as a base
crate leaf**, which is the opposite of what `cg_calls.py`'s docstring said
until 2026-09-16.

This settles it with **no build**: the `--bench` gate already builds
`target/release/bot_ladder` every run, so both binaries are on disk. For each
symbol it reports, per binary, whether the symbol survives at all and — when it
does — its size, instruction count and call count.

How to read the output:

* **absent in `release`, present in `profiling-fast`** — the row is an
  ARTIFACT. `release` inlined it. Do not rank it, and do not `#[inline]` it.
* **present in both** — the call is real in the shipped binary. Rank it by its
  BODY (instructions minus the prologue), not by the dump's call count.
* **absent in both** — it is inlined everywhere and the dump's row is somebody
  else's code attributed to a symbol that no longer exists.

⚠ Grep disassembly with `\\bcall\\b`, never `\\tcall`: objdump's column layout
makes the latter match nothing, which reads as "no calls" and turns a
refutation into a fabricated win. This script uses the former.
"""
import argparse
import re
import subprocess
import sys

CALL = re.compile(r"\bcall\b")
INSN = re.compile(r"^\s+[0-9a-f]+:")


def find_symbol(binary, name):
    """(addr, size) of the first symbol whose demangled name contains `name`."""
    try:
        out = subprocess.run(
            ["nm", "-C", "-S", binary], capture_output=True, text=True, check=False
        ).stdout
    except FileNotFoundError:
        sys.exit("nm not found")
    for line in out.splitlines():
        parts = line.split(maxsplit=3)
        if len(parts) == 4 and name in parts[3]:
            try:
                return int(parts[0], 16), int(parts[1], 16)
            except ValueError:
                continue
    return None


def body(binary, addr, size):
    """(instructions, calls, fs-segment refs) over one symbol's byte range."""
    out = subprocess.run(
        [
            "objdump", "-d", "--demangle",
            f"--start-address=0x{addr:x}", f"--stop-address=0x{addr + size:x}",
            binary,
        ],
        capture_output=True, text=True, check=False,
    ).stdout
    insns = [l for l in out.splitlines() if INSN.match(l)]
    return len(insns), sum(1 for l in insns if CALL.search(l)), sum(
        1 for l in insns if "%fs:" in l
    )


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("symbols", nargs="+")
    ap.add_argument("--release", default="target/release/bot_ladder")
    ap.add_argument("--fast", default="target/profiling-fast/bot_ladder")
    a = ap.parse_args()

    print(f"{'symbol':<44}{'release':>22}{'profiling-fast':>22}  verdict")
    for sym in a.symbols:
        cells, present = [], []
        for binary in (a.release, a.fast):
            hit = find_symbol(binary, sym)
            if hit is None:
                cells.append(f"{'inlined away':>22}")
                present.append(False)
                continue
            addr, size = hit
            n, calls, fs = body(binary, addr, size)
            extra = f" {fs}fs" if fs else ""
            cells.append(f"{n:>10} insn {calls:>2}call{extra:>5}")
            present.append(True)
        if present[0] and present[1]:
            verdict = "REAL — rank by the body"
        elif present[1] and not present[0]:
            verdict = "ARTIFACT — release inlines it"
        elif not any(present):
            verdict = "inlined in both — row is someone else's code"
        else:
            verdict = "odd: in release only"
        print(f"{sym[:43]:<44}{cells[0]}{cells[1]}  {verdict}")


if __name__ == "__main__":
    main()
