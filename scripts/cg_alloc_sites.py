#!/usr/bin/env python3
"""Name the SOURCE LINE of every allocation a function makes — the census
column `cg_growth.py` cannot produce.

    python3 scripts/cg_alloc_sites.py cg.instr.out <binary> <function-substring>

`cg_growth.py` ranks the *callers* of `finish_grow`, which is where a growth is
charged once `grow_one` is inlined into the engine — so it says
"`declare_blockers`, 19,616 growths" and stops. This says which of that
function's buffers they are.

**Both inputs are load-bearing.**

* The dump must come from `valgrind --tool=callgrind --dump-instr=yes`, so
  every call edge carries the calling *instruction*, not just a line number.
* The binary must be **`profiling-lines`**, not `profiling-fast`.
  `profiling-fast` is `split-debuginfo = "unpacked"`, which puts
  `.debug_info` — and with it every `DW_TAG_inlined_subroutine` — in `.dwo`
  files `addr2line` does not read. It still answers, and the answer is the
  *outermost* frame's line: every site in `resolve_combat` came back as
  `combat.rs:2590`, the one call this function makes. Packed DWARF gives the
  real chain and the same eight addresses resolve to eight different lines.

The innermost frame whose file is in the tree is the one printed, which is what
puts `IdMap::entry_or_default`'s own line on the row rather than the caller's:
a container's `push` is the honest answer to "what allocated", and the caller
is one `--rows` away in `cg_growth.py`. PERF `(-158)` is the reading that
produced this script.
"""
import re
import subprocess
import sys
from collections import defaultdict

ALLOC = re.compile(
    r"finish_grow|grow_one|do_reserve_and_handle|__rust_alloc|RawVecInner|::malloc|::realloc"
)


def resolve(tbl, tok):
    m = re.match(r"\((\d+)\)\s*(.*)", tok)
    if not m:
        return tok
    i = m.group(1)
    if m.group(2):
        tbl[i] = m.group(2)
    return tbl.get(i, i)


def sites(dump, needle):
    names, out = {}, defaultdict(int)
    cur_fn = pend = None
    pendn = cur_instr = 0
    skip = (
        "fl=", "fi=", "fe=", "cfi=", "cfl=", "ob=", "cob=", "#", "desc:",
        "positions:", "events:", "summary:", "version:", "creator:", "pid:",
        "cmd:", "part:", "totals:",
    )
    for raw in open(dump, errors="replace"):
        line = raw.rstrip("\n")
        if line.startswith("fn="):
            cur_fn, pend, cur_instr = resolve(names, line[3:]), None, 0
            continue
        if line.startswith("cfn="):
            pend = resolve(names, line[4:])
            continue
        if line.startswith("calls="):
            pendn = int(line.split("=")[1].split()[0])
            continue
        if line.startswith(skip) or not line.strip():
            continue
        parts = line.split()
        if len(parts) >= 3:
            tok = parts[0]
            if tok.startswith("+"):
                cur_instr += int(tok[1:])
            elif tok.startswith("-"):
                cur_instr -= int(tok[1:])
            elif tok.startswith("0x"):
                cur_instr = int(tok, 16)
            if pend and cur_fn and needle in cur_fn and ALLOC.search(pend):
                out[cur_instr] += pendn
        pend = None
    return out


def main():
    if len(sys.argv) < 4:
        sys.exit(__doc__)
    dump, binp, needle = sys.argv[1], sys.argv[2], sys.argv[3]
    rows = sorted(sites(dump, needle).items(), key=lambda kv: -kv[1])
    total = sum(n for _, n in rows)
    for addr, n in rows[: int(sys.argv[4]) if len(sys.argv) > 4 else 15]:
        chain = subprocess.run(
            ["addr2line", "-e", binp, "-i", "-C", "-f", hex(addr)],
            capture_output=True, text=True,
        ).stdout.splitlines()
        frames = [(chain[i], chain[i + 1]) for i in range(0, len(chain) - 1, 2)]
        engine = [(f, l) for f, l in frames if "crabomination" in l]
        fn, loc = engine[0] if engine else (frames[-1] if frames else ("?", "?"))
        print(f"{n:>8,}  {loc.split('/')[-1]:32}  {fn[:70]}")
    print(f"{total:>8,}  TOTAL allocations charged to *{needle}*")


if __name__ == "__main__":
    main()
