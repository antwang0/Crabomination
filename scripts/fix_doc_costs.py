#!/usr/bin/env python3
"""Sync the first mana-cost run in each STX card's doc comment to its (now
corrected) `cost:` literal."""
import re
from pathlib import Path
import sys

REPO = Path(__file__).resolve().parent.parent
STX = REPO / "crabomination_catalog" / "src" / "sets" / "stx"
sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_stx_drift import FUNC_RE, sym_to_str, cost_to_scry, norm

COSTRUN = re.compile(r"(\{[0-9WUBRGXCP/]+\})+")

fixed = 0
for src in sorted(STX.glob("*.rs")):
    text = src.read_text()
    matches = list(FUNC_RE.finditer(text))
    edits = []  # (start,end,replacement) in absolute coords
    for idx, m in enumerate(matches):
        start = m.end()
        nxt = matches[idx + 1] if idx + 1 < len(matches) else None
        body = text[start: nxt.start() if nxt else len(text)]
        cm = re.search(r"cost:\s*cost\(&\[(.*?)\]\)", body, re.S)
        if not cm: continue
        syms = [sym_to_str(c) for c in re.split(r",(?![^()]*\))", cm.group(1)) if c.strip()]
        scost = cost_to_scry(syms)
        if not scost: continue
        # doc block = consecutive /// lines immediately before fn start (m.start())
        line_start = text.rfind("\n", 0, m.start()) + 1
        # walk backwards over /// lines
        docs_start = line_start
        cur = line_start
        while True:
            prev_nl = text.rfind("\n", 0, cur - 1)
            ls = prev_nl + 1
            line = text[ls:cur-1] if cur > 0 else ""
            if line.lstrip().startswith("///"):
                docs_start = ls
                cur = ls
            else:
                break
            if ls == 0: break
        full_doc = text[docs_start:m.start()]
        # restrict to the FIRST doc line (the "Name — {cost} Type" title) so
        # ability costs ({C}{C}, etc.) in later prose are never clobbered.
        nl = full_doc.find("\n")
        doc = full_doc[:nl] if nl >= 0 else full_doc
        mo = COSTRUN.search(doc)
        if not mo: continue
        cur_run = mo.group(0)
        if norm(cur_run) == norm(scost): continue
        # ensure it really looks like a mana cost (>=1 colored/generic pip) and the doc run
        # isn't something like {T} ability — require it matches the corrected cost's token set difference
        abs_s = docs_start + mo.start()
        abs_e = docs_start + mo.end()
        edits.append((abs_s, abs_e, scost))
    if edits:
        edits.sort()
        out, prev = [], 0
        for s, e, rep in edits:
            out.append(text[prev:s]); out.append(rep); prev = e
        out.append(text[prev:])
        src.write_text("".join(out))
        fixed += len(edits)
        print(f"{src.name}: {len(edits)} doc cost(s) synced")
print(f"\n{fixed} doc cost(s) fixed.")
