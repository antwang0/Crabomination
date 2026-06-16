#!/usr/bin/env python3
"""Regenerate stat-assertion expectations after a (verified-correct) catalog stat
change. Runs the test suite, parses each `assert_eq!(actual, expected)` failure,
and rewrites the `expected` literal on the panic line to the engine's actual
output. Use ONLY when the new actual values are known-correct (e.g. fixed to the
Scryfall cache) — it bakes whatever the engine now produces into the tests.

  python3 scripts/regen_test_assertions.py            # iterate until stable
  python3 scripts/regen_test_assertions.py --once

Handles 2-tuples `(a, b)` and scalars. Anything it can't patch is reported for
manual fixing. Re-runs up to 6 rounds to settle cascades.
"""
import re, subprocess, sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
BLOCK = re.compile(
    r"panicked at ([^:]+):(\d+):\d+:\s*\n\s*assertion `left == right` failed(?:: [^\n]*)?\n\s*left: ([^\n]+)\n\s*right: ([^\n]+)")

def run_tests():
    p = subprocess.run(["cargo", "test", "-p", "crabomination", "--lib"],
                       cwd=REPO, capture_output=True, text=True)
    return p.stdout + p.stderr

def patch_line(path, lineno, left, right):
    f = REPO / path.replace("crabomination/src/game/../", "crabomination/src/")
    if not f.exists():
        f = REPO / path
    lines = f.read_text().splitlines(keepends=True)
    if lineno - 1 >= len(lines): return False, "line OOB"
    line = lines[lineno - 1]
    # tuple form
    mt = re.fullmatch(r"\((-?\d+),\s*(-?\d+)\)", right.strip())
    ml = re.fullmatch(r"\((-?\d+),\s*(-?\d+)\)", left.strip())
    if mt and ml:
        pat = re.compile(r"\(\s*%s\s*,\s*%s\s*\)" % (re.escape(mt.group(1)), re.escape(mt.group(2))))
        new, n = pat.subn(f"({ml.group(1)}, {ml.group(2)})", line, count=1)
        if n:
            lines[lineno - 1] = new; f.write_text("".join(lines)); return True, None
        return False, f"tuple {right} not found on line"
    # scalar form
    if re.fullmatch(r"-?\d+", right.strip()) and re.fullmatch(r"-?\d+", left.strip()):
        r, l = right.strip(), left.strip()
        # prefer the expected arg: `, <R>` not followed by another digit
        for pat in (re.compile(r"(,\s*)%s(?![\d.])" % re.escape(r)),
                    re.compile(r"(?<![\d.])%s(?![\d.])" % re.escape(r))):
            new, n = pat.subn(lambda m: (m.group(1) if m.lastindex else "") + l, line, count=1)
            if n:
                lines[lineno - 1] = new; f.write_text("".join(lines)); return True, None
        return False, f"scalar {right} not found"
    return False, f"unhandled repr left={left!r} right={right!r}"

def main():
    once = "--once" in sys.argv
    for rnd in range(1, 7):
        out = run_tests()
        m = re.search(r"test result: (\w+)\. (\d+) passed; (\d+) failed", out)
        fails = int(m.group(3)) if m else -1
        blocks = BLOCK.findall(out)
        print(f"round {rnd}: failed={fails}, assert-blocks={len(blocks)}")
        if not blocks:
            print("no more assert_eq failures (remaining failures, if any, are non-assertion)"); return
        patched, problems = 0, []
        seen = set()
        for path, ln, left, right in blocks:
            key = (path, ln)
            if key in seen: continue
            seen.add(key)
            ok, err = patch_line(path, int(ln), left, right)
            if ok: patched += 1
            else: problems.append(f"  {path}:{ln}  {err}")
        print(f"  patched {patched}")
        for p in problems: print(p)
        if once or patched == 0: return

if __name__ == "__main__":
    main()
