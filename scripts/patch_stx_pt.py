#!/usr/bin/env python3
"""Correct STX creature power/toughness literals to match the Scryfall cache.
Also syncs a `P/T` token in the doc-comment title line."""
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
STX = REPO / "crabomination_catalog" / "src" / "sets" / "stx"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items()}
sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_stx_drift import FUNC_RE

def front_pt(card):
    if card.get("power") is not None:
        return card.get("power"), card.get("toughness")
    if card.get("card_faces"):
        f = card["card_faces"][0]
        if f.get("power") is not None:
            return f.get("power"), f.get("toughness")
    return None, None

changed = 0
for src in sorted(STX.glob("*.rs")):
    text = src.read_text()
    matches = list(FUNC_RE.finditer(text))
    edits = []
    for idx, m in enumerate(matches):
        start = m.end()
        nxt = matches[idx + 1] if idx + 1 < len(matches) else None
        end = nxt.start() if nxt else len(text)
        body = text[start:end]
        nm = re.search(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
        if not nm: continue
        card = CACHE_LC.get(nm.group(1).lower())
        if not card: continue
        cp, ct = front_pt(card)
        if cp is None: continue
        try:
            cpi, cti = int(cp), int(ct)
        except (ValueError, TypeError):
            continue  # */* etc.
        pm = re.search(r"power:\s*(-?\d+)", body)
        tm = re.search(r"toughness:\s*(-?\d+)", body)
        if not (pm and tm): continue
        if int(pm.group(1)) == cpi and int(tm.group(1)) == cti: continue
        edits.append((start + pm.start(1), start + pm.end(1), str(cpi)))
        edits.append((start + tm.start(1), start + tm.end(1), str(cti)))
        # doc P/T on the title line
        line_start = text.rfind("\n", 0, m.start()) + 1
        cur = line_start
        docs_start = line_start
        while True:
            prev_nl = text.rfind("\n", 0, cur - 1)
            ls = prev_nl + 1
            line = text[ls:cur-1]
            if line.lstrip().startswith("///"):
                docs_start = ls; cur = ls
            else:
                break
            if ls == 0: break
        first_line_end = text.find("\n", docs_start)
        title = text[docs_start:first_line_end]
        dpt = re.search(r"\b(\d+)/(\d+)\b", title)
        if dpt and (int(dpt.group(1)) != cpi or int(dpt.group(2)) != cti):
            edits.append((docs_start + dpt.start(), docs_start + dpt.end(), f"{cpi}/{cti}"))
        changed += 1
        print(f"FIX PT {nm.group(1)} ({src.name}::{m.group(1)}): {pm.group(1)}/{tm.group(1)} -> {cpi}/{cti}")
    if edits:
        edits.sort()
        out, prev = [], 0
        for s, e, rep in edits:
            out.append(text[prev:s]); out.append(rep); prev = e
        out.append(text[prev:])
        src.write_text("".join(out))
print(f"\n{changed} creature(s) P/T-corrected.")
