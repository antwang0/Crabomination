#!/usr/bin/env python3
"""Read sites that match a static effect bare, for effects the catalog gates.

A static under `WhileYourTurn` / `WhileCondition` / `WhileCountersAtLeast` /
`WhileClassLevelAtLeast` is live only through `GameState::active_static` (or
`StaticEffect::ungated()` for a definition-only lane). A read site that
matches `sa.effect` directly never sees the gated copy: Nahiri's equip
discount, Festival of Embers, Artist's Talent's +2, Gruul Spellbreaker's
player hexproof, Limited Resources and Rock Jockey's land locks, Momo's
discount were all dead that way (2026-09-28).

For each effect variant the catalog wraps, every engine site naming it is
listed unless `active_static` / `ungated` / an explicit peel appears within
the 25 lines above it. Heuristic: the layer system (`game/mod.rs`'s
`static_layer` table) and the bot's evaluators are known readers of their
own; triage the rest.

    cargo build -p crabomination --bin dump_cards
    target/debug/dump_cards --grep "inner: " > /tmp/inner.tsv
    python3 scripts/audit_gate_peel.py /tmp/inner.tsv
"""

import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
WRAPPERS = ("WhileYourTurn", "WhileNotYourTurn", "WhileCondition", "WhileCountersAtLeast", "WhileClassLevelAtLeast")


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    by = {}
    for line in open(sys.argv[1]):
        name, dbg = line.split("\t", 1)
        for m in re.finditer(r"(?:%s) \{" % "|".join(WRAPPERS), dbg):
            mm = re.search(r"inner: ([A-Z][A-Za-z]*)", dbg[m.end():m.end() + 3000])
            if mm and mm.group(1) not in WRAPPERS:
                by.setdefault(mm.group(1), set()).add(name)
    files = glob.glob(os.path.join(ROOT, "crabomination/src/**/*.rs"), recursive=True)
    src = {f: open(f).read().split("\n") for f in files}
    rows = 0
    for v, names in sorted(by.items()):
        bad = []
        for f, lines in src.items():
            for i, l in enumerate(lines):
                s = l.strip()
                if s.startswith("//") or s.startswith("|") or not re.search(r"(?:StaticEffect|SE)::%s\b" % v, l):
                    continue
                ctx = "\n".join(lines[max(0, i - 25):i + 3])
                if not re.search(r"active_static|ungated|WhileClassLevelAtLeast|inner|peel", ctx):
                    bad.append(f"{os.path.relpath(f, ROOT)}:{i + 1}")
        if bad:
            rows += 1
            print(f"{v:40} {', '.join(sorted(names)[:3]):60} {' '.join(bad[:4])}")
    print(f"{rows} rows")


main()
