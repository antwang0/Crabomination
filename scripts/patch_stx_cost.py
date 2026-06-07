#!/usr/bin/env python3
"""Auto-correct STX card cost literals to match the Scryfall cache.

Span-based: collects (start,end,replacement) for each offending cost literal
using absolute file offsets, then rebuilds the file by splicing — so duplicate
cost strings on different cards never collide.
"""
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
STX = REPO / "crabomination_catalog" / "src" / "sets" / "stx"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items()}
sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_stx_drift import FUNC_RE, sym_to_str, cost_to_scry, norm

def front_cost(card):
    ref = card.get("mana_cost", "")
    if not ref and card.get("card_faces"):
        ref = card["card_faces"][0].get("mana_cost", "")
    return ref

def sym_to_helper(tok):
    t = tok.strip("{}")
    if t.isdigit(): return f"generic({t})"
    if t == "X": return "x()"
    if t == "C": return "colorless(1)"
    if t in "WUBRG": return {"W":"w()","U":"u()","B":"b()","R":"r()","G":"g()"}[t]
    if "/" in t:
        a, b = t.split("/")
        cn = {"W":"Color::White","U":"Color::Blue","B":"Color::Black","R":"Color::Red","G":"Color::Green"}
        if b == "P": return f"phyrexian({cn[a]})"
        if a.isdigit(): return f"mono_hybrid({a}, {cn[b]})"
        return f"hybrid({cn[a]}, {cn[b]})"
    return None

def cost_expr(mc):
    toks = re.findall(r"\{[^}]+\}", mc)
    parts, cless = [], 0
    for t in toks:
        if t == "{C}": cless += 1; continue
        if cless: parts.append(f"colorless({cless})"); cless = 0
        h = sym_to_helper(t)
        if h is None: return None
        parts.append(h)
    if cless: parts.append(f"colorless({cless})")
    return "cost(&[" + ", ".join(parts) + "])"

targets = set(sys.argv[1:])
changed = 0
for src in sorted(STX.glob("*.rs")):
    text = src.read_text()
    spans = []  # (start, end, replacement)
    for m in FUNC_RE.finditer(text):
        fn = m.group(1)
        start = m.end()
        nxt = FUNC_RE.search(text, start)
        body_end = nxt.start() if nxt else len(text)
        body = text[start:body_end]
        nm = re.search(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
        if not nm: continue
        card = CACHE_LC.get(nm.group(1).lower())
        if not card: continue
        ref = front_cost(card)
        if not ref: continue
        cm = re.search(r"cost:\s*cost\(&\[(.*?)\]\)", body, re.S)
        if not cm: continue
        syms = [sym_to_str(c) for c in re.split(r",(?![^()]*\))", cm.group(1)) if c.strip()]
        scost = cost_to_scry(syms)
        if scost is None: continue
        if norm(scost) == norm(ref): continue
        if targets and fn not in targets: continue
        new_expr = cost_expr(ref)
        if not new_expr: continue
        abs_start = start + cm.start()
        abs_end = start + cm.end()
        spans.append((abs_start, abs_end, f"cost: {new_expr}"))
        print(f"FIX {nm.group(1)} ({src.name}::{fn}): {scost} -> {ref}")
        changed += 1
    if spans:
        spans.sort()
        out, prev = [], 0
        for s, e, rep in spans:
            out.append(text[prev:s]); out.append(rep); prev = e
        out.append(text[prev:])
        src.write_text("".join(out))
print(f"\n{changed} cost literal(s) patched.")
