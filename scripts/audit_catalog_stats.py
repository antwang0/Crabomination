#!/usr/bin/env python3
"""Catalog-wide stat audit vs the real Scryfall cache (scripts/.scryfall_cache.json).

Scans every card factory under crabomination_catalog/src/sets/ and checks four
printed-stat dimensions against the cache: mana cost, power/toughness, creature
subtypes, and keywords. Generalizes audit_stx_drift.py (cost+P/T, STX only) and
audit_stx_types.py (type+keywords, STX only) to the whole catalog.

  python3 scripts/audit_catalog_stats.py              # per-set summary table
  python3 scripts/audit_catalog_stats.py SET           # detail for one set (e.g. sos, thb)

Only card names present in the cache are checked; synthesized-only names are
skipped. Keyword check reads the top-level CardDefinition.keywords field only
(so conditional/granted keywords nested in statics/equip-bonuses/tokens aren't
flagged). DFC keyword/type refs union across faces.
"""
import json, re, sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SETS = REPO / "crabomination_catalog" / "src" / "sets"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}

COLOR = {"White": "W", "Blue": "U", "Black": "B", "Red": "R", "Green": "G"}
ENGINE_KW = {"Flying","Vigilance","Menace","Trample","Deathtouch","Lifelink","First strike",
 "Double strike","Reach","Defender","Haste","Hexproof","Flash","Ward","Indestructible",
 "Shroud","Skulk","Horsemanship","Protection","Prowess","Changeling","Fear","Intimidate"}
KMAP = {"FirstStrike": "First strike", "DoubleStrike": "Double strike"}

def sym(call):
    call = call.strip()
    m = re.match(r"generic\((\d+)\)", call)
    if m: return "{%d}" % int(m.group(1))
    m = re.match(r"colorless\((\d+)\)", call)
    if m: return "{C}" * int(m.group(1))
    if call == "x()": return "{X}"
    for fn, s in [("w","W"),("u","U"),("b","B"),("r","R"),("g","G")]:
        if call == f"{fn}()": return "{%s}" % s
    m = re.match(r"hybrid\(Color::(\w+),\s*Color::(\w+)\)", call)
    if m: return "{%s/%s}" % (COLOR[m.group(1)], COLOR[m.group(2)])
    m = re.match(r"mono_hybrid\((\d+),\s*Color::(\w+)\)", call)
    if m: return "{%d/%s}" % (int(m.group(1)), COLOR[m.group(2)])
    m = re.match(r"phyrexian\(Color::(\w+)\)", call)
    if m: return "{%s/P}" % COLOR[m.group(1)]
    return None

def norm(mc): return tuple(sorted(re.findall(r"\{[^}]+\}", mc or "")))

def vec_after(body, i):
    j = body.find("vec![", i)
    if j < 0 or j - i > 40: return None
    k, d = j + 5, 1
    while k < len(body) and d:
        if body[k] == "[": d += 1
        elif body[k] == "]": d -= 1
        k += 1
    return body[j + 5:k - 1]

def toplevel_keywords(body):
    cd = body.find("CardDefinition {")
    if cd < 0: return None
    depth, k = 1, cd + len("CardDefinition {")
    while k < len(body) and depth:
        ch = body[k]
        if ch == "{": depth += 1
        elif ch == "}": depth -= 1
        elif depth == 1 and body.startswith("keywords:", k):
            return vec_after(body, k)
        k += 1
    return None

def toplevel_cost_args(body):
    """The depth-1 `cost: cost(&[...])` inner args of the CardDefinition literal,
    never a nested cost (e.g. an ability cost in a preceding `let`). None if
    absent / not in cost(&[...]) form."""
    cd = body.find("CardDefinition {")
    if cd < 0: return None
    depth, k = 1, cd + len("CardDefinition {")
    while k < len(body) and depth:
        ch = body[k]
        if ch == "{": depth += 1
        elif ch == "}": depth -= 1
        elif depth == 1 and body.startswith("cost:", k) and body[k-1] in " \t\n,{(":
            m = re.match(r"cost:\s*cost\(&\[", body[k:])
            if not m: return None
            start, d, kk = k + m.end(), 1, k + m.end()
            while kk < len(body) and d:
                if body[kk] == "[": d += 1
                elif body[kk] == "]": d -= 1
                kk += 1
            return body[start:kk-1]
        k += 1
    return None

def creature_subtypes_ref(card):
    faces = card.get("card_faces") or []
    for tl in [f.get("type_line") for f in faces] + [card.get("type_line")]:
        if tl and "Creature" in tl and "—" in tl:
            return set(tl.split("—", 1)[1].split("//")[0].split())
    return None

def ref_keywords(card):
    out = set(card.get("keywords", []) or [])
    for f in card.get("card_faces", []) or []:
        out |= set(f.get("keywords", []) or [])
    return out & ENGINE_KW

def set_of(path):
    rel = path.relative_to(SETS)
    return rel.parts[0] if len(rel.parts) > 1 else rel.stem

FUNC = re.compile(r"pub fn (\w+)\(\)\s*->\s*CardDefinition\s*\{")

def audit():
    per_set = {}      # set -> dict(checked, cost[], pt[], type[], kw[])
    for src in sorted(SETS.rglob("*.rs")):
        s = set_of(src)
        d = per_set.setdefault(s, {"checked": 0, "cost": [], "pt": [], "type": [], "kw": []})
        text = src.read_text()
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end()); body = text[m.end():nxt.start() if nxt else len(text)]
            nm = re.search(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
            if not nm: continue
            card = CACHE_LC.get(nm.group(1).lower())
            if not card: continue
            d["checked"] += 1
            tag = (nm.group(1), src.name, m.group(1))
            # cost
            cargs = toplevel_cost_args(body)
            if cargs is not None:
                syms = [sym(c) for c in re.split(r",(?![^()]*\))", cargs) if c.strip()]
                if all(syms):
                    got = "".join(syms)
                    refs = [card.get("mana_cost")] + [f.get("mana_cost") for f in card.get("card_faces", []) or []]
                    refs = [r for r in refs if r]
                    if refs and all(norm(got) != norm(r) for r in refs):
                        d["cost"].append((tag, got, "|".join(refs)))
            # P/T
            pm = re.search(r"power:\s*(-?\d+)", body); tm = re.search(r"toughness:\s*(-?\d+)", body)
            if pm and tm and card.get("power") is not None:
                if (pm.group(1), tm.group(1)) != (str(card["power"]), str(card["toughness"])):
                    d["pt"].append((tag, f"{pm.group(1)}/{tm.group(1)}", f"{card['power']}/{card['toughness']}"))
            # creature subtypes
            i = body.find("creature_types:")
            ctv = vec_after(body, i) if i >= 0 else None
            ref_ct = creature_subtypes_ref(card)
            if ctv is not None and ref_ct is not None:
                code_ct = set(re.findall(r"CreatureType::(\w+)", ctv))
                if code_ct and ref_ct and code_ct != ref_ct and not code_ct.issubset(ref_ct):
                    d["type"].append((tag, sorted(code_ct), sorted(ref_ct)))
            # keywords (top-level only)
            kwv = toplevel_keywords(body)
            if kwv is not None:
                code_kw = {KMAP.get(k, k) for k in re.findall(r"Keyword::(\w+)", kwv)} & ENGINE_KW
                rk = ref_keywords(card)
                if code_kw != rk:
                    d["kw"].append((tag, sorted(code_kw), sorted(rk)))
    return per_set

per_set = audit()
detail = sys.argv[1] if len(sys.argv) > 1 else None

if detail:
    d = per_set.get(detail)
    if not d: sys.exit(f"no such set '{detail}' (have: {', '.join(sorted(per_set))})")
    for dim in ("cost", "pt", "type", "kw"):
        print(f"\n=== {dim.upper()} drift in {detail} ({len(d[dim])}) ===")
        for tag, got, ref in d[dim]:
            print(f"  {tag[0]}  ({tag[1]}::{tag[2]})\n    code={got}  scryfall={ref}")
else:
    print(f"{'set':<12}{'checked':>8}{'cost':>6}{'P/T':>6}{'type':>6}{'kw':>6}")
    print("-" * 44)
    tot = {"checked": 0, "cost": 0, "pt": 0, "type": 0, "kw": 0}
    for s in sorted(per_set, key=lambda s: -(len(per_set[s]["cost"]) + len(per_set[s]["pt"]) + len(per_set[s]["type"]) + len(per_set[s]["kw"]))):
        d = per_set[s]
        if not d["checked"]: continue
        for k in tot: tot[k] += d["checked"] if k == "checked" else len(d[k])
        print(f"{s:<12}{d['checked']:>8}{len(d['cost']):>6}{len(d['pt']):>6}{len(d['type']):>6}{len(d['kw']):>6}")
    print("-" * 44)
    print(f"{'TOTAL':<12}{tot['checked']:>8}{tot['cost']:>6}{tot['pt']:>6}{tot['type']:>6}{tot['kw']:>6}")
    print("\nDetail for a set:  python3 scripts/audit_catalog_stats.py <set>")
