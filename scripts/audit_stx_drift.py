#!/usr/bin/env python3
"""Diff STX catalog card literals (name/cost/P-T/type) against the Scryfall cache.

Scans crabomination_catalog/src/sets/stx/*.rs, extracts each card factory's
name + cost(&[...]) + power/toughness, normalizes the cost to a Scryfall-style
mana string, and reports cards whose cost or P/T disagree with the cache.
"""
import json
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
STX = REPO / "crabomination_catalog" / "src" / "sets" / "stx"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items()}

COLOR = {"White": "W", "Blue": "U", "Black": "B", "Red": "R", "Green": "G"}

def sym_to_str(call):
    call = call.strip()
    m = re.match(r"generic\((\d+)\)", call)
    if m: return ("generic", int(m.group(1)))
    m = re.match(r"colorless\((\d+)\)", call)
    if m: return ("c", int(m.group(1)))
    if call == "x()": return ("X",)
    for fn, sym in [("w","W"),("u","U"),("b","B"),("r","R"),("g","G")]:
        if call == f"{fn}()": return (sym,)
    m = re.match(r"hybrid\(Color::(\w+),\s*Color::(\w+)\)", call)
    if m: return ("hy", COLOR[m.group(1)], COLOR[m.group(2)])
    m = re.match(r"mono_hybrid\((\d+),\s*Color::(\w+)\)", call)
    if m: return ("mh", int(m.group(1)), COLOR[m.group(2)])
    m = re.match(r"phyrexian\(Color::(\w+)\)", call)
    if m: return ("ph", COLOR[m.group(1)])
    return ("?", call)

def cost_to_scry(syms):
    out = ""
    for s in syms:
        if s[0] == "generic": out += "{%d}" % s[1]
        elif s[0] == "c": out += "{C}" * s[1]
        elif s[0] == "X": out += "{X}"
        elif s[0] == "hy": out += "{%s/%s}" % (s[1], s[2])
        elif s[0] == "mh": out += "{%d/%s}" % (s[1], s[2])
        elif s[0] == "ph": out += "{%s/P}" % s[1]
        elif s[0] == "?": return None
        else: out += "{%s}" % s[0]
    return out

def norm(mc):
    # normalize a scryfall mana_cost string into a comparable token multiset
    return tuple(sorted(re.findall(r"\{[^}]+\}", mc or "")))

FUNC_RE = re.compile(r"pub fn (\w+)\(\)\s*->\s*CardDefinition\s*\{")

issues = []
for src in sorted(STX.glob("*.rs")):
    text = src.read_text()
    # split into factory bodies by locating each fn and the next fn
    for m in FUNC_RE.finditer(text):
        start = m.end()
        nxt = FUNC_RE.search(text, start)
        body = text[start: nxt.start() if nxt else len(text)]
        nm = re.search(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
        if not nm: continue
        name = nm.group(1)
        cm = re.search(r"cost:\s*cost\(&\[(.*?)\]\)", body, re.S)
        if not cm: continue
        syms = [sym_to_str(c) for c in re.split(r",(?![^()]*\))", cm.group(1)) if c.strip()]
        scost = cost_to_scry(syms)
        if scost is None: continue  # unparseable
        card = CACHE_LC.get(name.lower())
        if not card: continue
        # An MDFC's cache entry is keyed by its full name; a back-face factory
        # def (e.g. Augusta, Embrose) carries its own face cost, so accept a
        # match against EITHER printed face rather than only card_faces[0].
        refs = []
        if card.get("mana_cost"):
            refs.append(card["mana_cost"])
        for face in card.get("card_faces", []) or []:
            if face.get("mana_cost"):
                refs.append(face["mana_cost"])
        if not refs:
            continue  # token / costless — skip
        if all(norm(scost) != norm(r) for r in refs):
            issues.append((src.name, m.group(1), name, scost, "|".join(refs), "COST"))
            continue
        # P/T check for creatures
        pm = re.search(r"power:\s*(-?\d+)", body)
        tm = re.search(r"toughness:\s*(-?\d+)", body)
        if pm and tm and card.get("power") is not None:
            cp, ct = card.get("power"), card.get("toughness")
            try:
                if str(int(pm.group(1))) != str(cp) or str(int(tm.group(1))) != str(ct):
                    issues.append((src.name, m.group(1), name, f"{pm.group(1)}/{tm.group(1)}", f"{cp}/{ct}", "PT"))
            except ValueError:
                pass

print(f"{len(issues)} drift(s):\n")
for f, fn, name, got, ref, kind in issues:
    print(f"[{kind}] {name}  ({f}::{fn})")
    print(f"    got={got!r}  scryfall={ref!r}")
