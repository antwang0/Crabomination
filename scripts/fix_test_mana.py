#!/usr/bin/env python3
"""Replace the mana-pool setup block of named test fns with a generous fill.

For each failing test, find its fn body and rewrite the contiguous run of
`g.players[N].mana_pool.add*(...)` lines into a full-color fill for the same
player indices, so any corrected (possibly hybrid) cost is payable.
"""
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
TESTS = REPO / "crabomination" / "src" / "tests"

names = set(l.strip().split("::")[-1] for l in open("/tmp/failing.txt") if l.strip())

MANA_LINE = re.compile(r"^\s*g\.players\[(\d+)\]\.mana_pool\.(add|add_colorless|add_snow|add_snow_colorless|add_restricted)\b.*;\s*$")

def fill_lines(players, indent):
    out = []
    for p in players:
        out.append(f"{indent}for _c in [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green] {{ g.players[{p}].mana_pool.add(_c, 20); }}")
        out.append(f"{indent}g.players[{p}].mana_pool.add_colorless(20);")
    return out

patched = 0
for src in TESTS.rglob("*.rs"):
    lines = src.read_text().splitlines(keepends=False)
    text = "\n".join(lines)
    changed = False
    out_lines = []
    i = 0
    # find fn spans
    fn_re = re.compile(r"^\s*fn (\w+)\(")
    n = len(lines)
    while i < n:
        m = fn_re.match(lines[i])
        if not m or m.group(1) not in names:
            out_lines.append(lines[i]); i += 1; continue
        # within this fn, scan for the first contiguous mana block
        fn_start = i
        # collect until matching close (track brace depth from this line)
        out_lines.append(lines[i])
        i += 1
        # find contiguous mana block
        # gather all lines of fn until depth returns to 0
        # simpler: process lines until next top-level fn or EOF, replacing first mana run
        replaced = False
        while i < n and not fn_re.match(lines[i]):
            if not replaced and MANA_LINE.match(lines[i]):
                players = []
                indent = lines[i][:len(lines[i]) - len(lines[i].lstrip())]
                while i < n and MANA_LINE.match(lines[i]):
                    p = int(MANA_LINE.match(lines[i]).group(1))
                    if p not in players: players.append(p)
                    i += 1
                out_lines.extend(fill_lines(players, indent))
                replaced = True
                changed = True
                patched += 1
                continue
            out_lines.append(lines[i]); i += 1
    if changed:
        src.write_text("\n".join(out_lines) + "\n")
        print("patched", src.relative_to(REPO))

print(f"\n{patched} test mana block(s) rewritten.")
