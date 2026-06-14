#!/usr/bin/env python3
"""STX/STA type-line + keyword drift audit against scripts/.scryfall_cache.json.

Companion to audit_stx_drift.py (which checks only cost + P/T). This adds the
two fields that audit can't see: creature subtypes and printed keywords. Only
card names present in the cache are checked; purely synthesized names are
skipped. Run after refreshing the cache from real Scryfall."""
import json, re
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
STX = REPO / "crabomination_catalog" / "src" / "sets" / "stx"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}

# Scryfall keywords the engine represents as Keyword:: variants on the body.
ENGINE_KW = {"Flying","Vigilance","Menace","Trample","Deathtouch","Lifelink",
    "First strike","Double strike","Reach","Defender","Haste","Hexproof","Flash",
    "Ward","Indestructible","Shroud","Skulk","Horsemanship","Protection","Prowess",
    "Changeling"}

def vec_field(body, field):
    """Bracket-balanced contents of `field: vec![ ... ]`, or None."""
    i = body.find(field + ":")
    if i < 0: return None
    j = body.find("vec![", i)
    if j < 0 or j - i > 40: return None
    k, depth = j + 5, 1
    while k < len(body) and depth:
        if body[k] == "[": depth += 1
        elif body[k] == "]": depth -= 1
        k += 1
    return body[j + 5:k - 1]

def ref_type_lines(card):
    out = [card.get("type_line")]
    for f in card.get("card_faces", []) or []:
        out.append(f.get("type_line"))
    return [t for t in out if t]

def ref_subtypes(card):
    out = set()
    for tl in ref_type_lines(card):
        if "—" in tl:
            out |= set(tl.split("—", 1)[1].replace("//", " ").split())
    return out

def ref_keywords(card):
    out = set(card.get("keywords", []) or [])
    for f in card.get("card_faces", []) or []:
        out |= set(f.get("keywords", []) or [])
    return out & ENGINE_KW

FUNC = re.compile(r"pub fn (\w+)\(\)\s*->\s*CardDefinition\s*\{")
type_d, kw_d = [], []
for src in sorted(STX.glob("*.rs")):
    text = src.read_text()
    for m in FUNC.finditer(text):
        nxt = FUNC.search(text, m.end())
        body = text[m.end():nxt.start() if nxt else len(text)]
        nm = re.search(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
        if not nm: continue
        name = nm.group(1)
        card = CACHE_LC.get(name.lower())
        if not card: continue
        tag = f"{name}  ({src.name}::{m.group(1)})"
        # creature subtypes — only for cards that are creatures in the cache
        ctv = vec_field(body, "creature_types")
        if ctv is not None and "Creature" in " ".join(ref_type_lines(card)):
            code_ct = set(re.findall(r"CreatureType::(\w+)", ctv))
            ref_ct = ref_subtypes(card)
            if code_ct and ref_ct and code_ct != ref_ct and not code_ct.issubset(ref_ct):
                type_d.append((tag, sorted(code_ct), sorted(ref_ct)))
        # keywords
        kwv = vec_field(body, "keywords")
        if kwv is not None:
            code_kw = set()
            for k in re.findall(r"Keyword::(\w+)", kwv):
                code_kw.add({"FirstStrike": "First strike", "DoubleStrike": "Double strike"}.get(k, k))
            code_kw &= ENGINE_KW
            rk = ref_keywords(card)
            if code_kw != rk:
                kw_d.append((tag, sorted(code_kw), sorted(rk)))

print(f"=== CREATURE-TYPE drift ({len(type_d)}) ===")
for t, code, ref in type_d:
    print(f"  {t}\n    code={code}  scryfall={ref}")
print(f"\n=== KEYWORD drift ({len(kw_d)}) ===")
for t, code, ref in kw_d:
    print(f"  {t}\n    code={code}  scryfall={ref}")
print(f"\nTOTAL: type={len(type_d)} keyword={len(kw_d)}")
