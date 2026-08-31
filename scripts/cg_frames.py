#!/usr/bin/env python3
"""Which hot functions pay a stack frame they almost never use.

PERF `(-129)` found `event_matches_spec` entering through
`push r15/r14/r12/rbx; sub $0x28` on all 1,028,014 of its calls because one
arm of its 0x70-way jump table calls `computed_permanent`. LLVM will not
shrink-wrap a prologue past an indirect branch, so every hot arm paid for
the cold one — 12 of 26.7 Ir a call, and outlining that arm was worth
-0.48 % of `cube` and -0.72 % of `sealed` on its own.

That shape is mechanical, so this finds the rest of it: join a callgrind
dump's call counts against the binary's disassembly and rank by
`calls x prologue instructions`, showing how many `call` sites the body has.

    python3 scripts/cg_frames.py cg.out target/profiling-fast/bot_ladder

**A row is a candidate, not a finding — except when `out/call` is ~0.** Two
columns decide it, and the second is the one that turns a guess into a
measurement:

* `body calls` is *static*: how many `call` sites the disassembly has. 0 means
  LLVM wanted callee-saved registers for its own arithmetic — nothing to
  outline, the frame **is** the code (`CardInstance::toughness`).
* `out/call` is *dynamic*: how many calls the function actually made per
  invocation on this workload. **`body calls` > 0 with `out/call` ~ 0 is the
  shape, proved rather than suspected** — every one of those invocations paid
  a prologue for a call it never made, and outlining the calling paths behind
  an `#[inline(never)]` half removes it. `ability_strip_off_battlefield` reads
  3 / 0.00 (`(-136)`); `can_grant_keyword` reads 2 / 0.88 and is *not* the
  shape, which cost nothing to learn because the column said so.
* A recursive callee counts in `out/call`, honestly: recursion needs the frame
  too. `requirement_live_leaves` reads 0.54 and `(-126)`'s rule prices the
  split as a likely loss at that share.

`--min-calls` (default 50,000) and `--rows` (default 30) bound the report.
"""

import collections
import re
import subprocess
import sys

PROLOGUE = re.compile(r"^\s+[0-9a-f]+:\s+(push|sub)\s+(%r[a-z0-9]+|\$0x[0-9a-f]+,%rsp)")
INSN = re.compile(r"^\s+[0-9a-f]+:\s+(\S+)")
HEADER = re.compile(r"^[0-9a-f]+ <(.+)>:$")


def call_counts(path):
    """(incoming, outgoing) calls per function name, folded over contexts."""
    counts = collections.Counter()
    out = collections.Counter()
    names = {}
    cur = None
    pend = None
    for line in open(path, errors="replace"):
        line = line.rstrip("\n")
        m = re.match(r"^cfn=\((\d+)\)(?: (.*))?$", line)
        if m:
            if m.group(2):
                names[m.group(1)] = m.group(2)
            pend = names.get(m.group(1), "")
            continue
        m = re.match(r"^fn=\((\d+)\)(?: (.*))?$", line)
        if m:
            if m.group(2):
                names[m.group(1)] = m.group(2)
            cur = names.get(m.group(1), "").split("'", 1)[0]
            continue
        if line.startswith("calls=") and pend:
            n = int(line.split("=", 1)[1].split()[0])
            counts[pend.split("'", 1)[0]] += n
            if cur:
                out[cur] += n
            pend = None
    return counts, out


def frames(binary):
    """(prologue_insns, call_sites, size) per mangled symbol."""
    out = subprocess.run(
        ["objdump", "-d", "--no-show-raw-insn", binary],
        capture_output=True, text=True, check=True,
    ).stdout
    info = {}
    cur = None
    prologue = 0
    in_prologue = True
    calls = 0
    size = 0
    for line in out.splitlines():
        m = HEADER.match(line)
        if m:
            if cur:
                info[cur] = (prologue, calls, size)
            cur, prologue, in_prologue, calls, size = m.group(1), 0, True, 0, 0
            continue
        if cur is None:
            continue
        m = INSN.match(line)
        if not m:
            continue
        size += 1
        if in_prologue and PROLOGUE.match(line):
            prologue += 1
        elif in_prologue:
            in_prologue = False
        if m.group(1).startswith("call"):
            calls += 1
    if cur:
        info[cur] = (prologue, calls, size)
    return info


def demangle(sym):
    """`_ZN13crabomination…17h<hash>E` -> the path, hash dropped."""
    m = re.match(r"^_ZN(.*)17h[0-9a-f]{16}E$", sym)
    body = m.group(1) if m else sym
    out, i = [], 0
    while i < len(body):
        n = re.match(r"(\d+)", body[i:])
        if not n:
            break
        k = len(n.group(1))
        out.append(body[i + k : i + k + int(n.group(1))])
        i += k + int(n.group(1))
    return "::".join(out) if out else sym


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    dump, binary = sys.argv[1], sys.argv[2]
    rows = 30
    min_calls = 50_000
    for a in sys.argv[3:]:
        if a.startswith("--rows="):
            rows = int(a.split("=", 1)[1])
        elif a.startswith("--min-calls="):
            min_calls = int(a.split("=", 1)[1])
    counts, outgoing = call_counts(dump)
    info = frames(binary)
    # The dump carries demangled names; the disassembly carries mangled ones.
    by_name = {}
    for sym, v in info.items():
        by_name.setdefault(demangle(sym), []).append(v)
    ranked = []
    for name, n in counts.items():
        if n < min_calls:
            continue
        hit = by_name.get(name)
        if not hit:
            continue
        prologue, calls, size = max(hit, key=lambda v: v[2])
        if prologue == 0:
            continue
        ranked.append((n * prologue * 2, n, prologue, calls, size, outgoing[name] / n, name))
    ranked.sort(reverse=True)
    print(
        f"# {len(ranked)} functions with >= {min_calls:,} calls and a prologue, "
        "ranked by calls x prologue Ir (push+pop, so x2)"
    )
    print(f"{'~Ir in frame':>13}  {'calls':>10}  {'prol':>4}  {'body calls':>10}  "
          f"{'out/call':>8}  {'insns':>6}  function")
    for cost, n, prologue, calls, size, per, name in ranked[:rows]:
        print(f"{cost:13,d}  {n:10,d}  {prologue:4d}  {calls:10d}  {per:8.2f}  "
              f"{size:6d}  {name}")


if __name__ == "__main__":
    main()
