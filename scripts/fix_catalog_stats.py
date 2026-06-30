#!/usr/bin/env python3
"""Correct catalog card stats (cost, P/T, creature subtypes) to the real Scryfall
cache. Keyword fixes are intentionally NOT done here — they need per-card judgment
(conditional/granted keywords) and are handled separately.

  python3 scripts/fix_catalog_stats.py <set> [<set>...] [--apply]

Without --apply it's a dry run (prints the diff it would make). Custom/original
cards (no Scryfall truth) are skipped via EXCLUDE. Only plain-integer P/T and
`cost(&[...])`-form costs are touched; CDA P/T and exotic cost builders are left.
"""
import json, re, sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
SETS = REPO / "crabomination_catalog" / "src" / "sets"
CACHE = json.load(open(Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}
EXCLUDE = {"Cosmogoyf", "Crabomination"}

card_rs = (REPO / "crabomination_base/src/card.rs").read_text()
_enum = re.sub(r"//.*", "", re.search(r"enum CreatureType\s*\{(.*?)\n\}", card_rs, re.S).group(1))
VALID_CT = set(re.findall(r"\b([A-Z][A-Za-z]+)\b", _enum))

COLORNAME = {"W": "White", "U": "Blue", "B": "Black", "R": "Red", "G": "Green"}

def cost_to_rust(mc):
    """Scryfall mana_cost string -> 'generic(2), b(), b()' builder args, or None."""
    toks = re.findall(r"\{([^}]+)\}", mc or "")
    out, colorless = [], 0
    for t in toks:
        if t.isdigit(): out.append(f"generic({int(t)})")
        elif t == "C": colorless += 1
        elif t == "X": out.append("x()")
        elif t in "WUBRG": out.append(f"{t.lower()}()")
        elif re.fullmatch(r"[WUBRG]/[WUBRG]", t):
            a, b = t.split("/"); out.append(f"hybrid(Color::{COLORNAME[a]}, Color::{COLORNAME[b]})")
        elif re.fullmatch(r"\d/[WUBRG]", t):
            n, c = t.split("/"); out.append(f"mono_hybrid({n}, Color::{COLORNAME[c]})")
        elif re.fullmatch(r"[WUBRG]/P", t):
            out.append(f"phyrexian(Color::{COLORNAME[t[0]]})")
        else:
            return None  # snow / unknown — don't touch
    if colorless: out.append(f"colorless({colorless})")
    return ", ".join(out)

def creature_face_subtypes(card):
    faces = card.get("card_faces") or []
    for tl in [f.get("type_line") for f in faces] + [card.get("type_line")]:
        if tl and "Creature" in tl and "—" in tl:
            return tl.split("—", 1)[1].split("//")[0].split()
    return None

def find_vec(body, field):
    i = body.find(field + ":")
    if i < 0: return None
    j = body.find("vec![", i)
    if j < 0 or j - i > 40: return None
    k, d = j + 5, 1
    while k < len(body) and d:
        if body[k] == "[": d += 1
        elif body[k] == "]": d -= 1
        k += 1
    return (j, k, body[j + 5:k - 1])

FUNC = re.compile(r"pub fn (\w+)\(\)\s*->\s*CardDefinition\s*\{")

def find_toplevel_cost(body):
    """Span (inner_args, start, end) of the depth-1 `cost: cost(&[...])` field of
    the CardDefinition literal — never a nested `cost:`/`mana_cost:` inside an
    ability defined in a preceding `let` binding. None if absent or not in
    `cost(&[...])` form."""
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
            start, d = k + m.end(), 1
            kk = start
            while kk < len(body) and d:
                if body[kk] == "[": d += 1
                elif body[kk] == "]": d -= 1
                kk += 1
            return (body[start:kk-1], start, kk-1)
        k += 1
    return None

MANA_HELPERS = ("generic", "colorless", "x", "w", "u", "b", "r", "g",
                "hybrid", "mono_hybrid", "phyrexian")

def ensure_mana_imports(text):
    """Add any cost helpers (and Color) used in the file to its top-level
    `use crate::mana::{...};` line so rebuilt costs compile."""
    needed = set()
    for h in MANA_HELPERS:
        if re.search(r"\b%s\(" % h, text): needed.add(h)
    if "Color::" in text: needed.add("Color")
    m = re.search(r"use crate::mana::\{([^}]*)\};", text)
    if not m: return text  # unusual import shape — leave (compile will flag)
    have = {n.strip() for n in m.group(1).split(",") if n.strip()}
    add = needed - have
    if not add: return text
    merged = sorted(have | add)
    return text[:m.start()] + "use crate::mana::{" + ", ".join(merged) + "};" + text[m.end():]

def run(set_names, apply):
    targets = []
    for s in set_names:
        p = SETS / s
        targets += list(p.rglob("*.rs")) if p.is_dir() else [SETS / f"{s}.rs"]
    counts = {"cost": 0, "pt": 0, "type": 0}
    for src in sorted(set(targets)):
        if not src.exists(): continue
        text = src.read_text(); edits = []
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end()); b0 = m.end(); body = text[b0:nxt.start() if nxt else len(text)]
            nm = re.search(r'name:\s*"((?:[^"\\]|\\.)*)"', body)
            if not nm or nm.group(1) in EXCLUDE: continue
            card = CACHE_LC.get(nm.group(1).lower())
            if not card: continue
            # cost
            tlc = find_toplevel_cost(body)
            if tlc:
                cur_args, cstart, cend = tlc
                # The code's `cost:` is the front/left half. For split/MDFC cards
                # the cache's top-level mana_cost is the combined "A // B" — use
                # the first face's cost instead so we don't sum both halves.
                faces = card.get("card_faces") or []
                ref0 = faces[0].get("mana_cost") if faces and faces[0].get("mana_cost") else card.get("mana_cost")
                refs = [ref0] if ref0 and "//" not in ref0 else []
                if refs:
                    new = cost_to_rust(refs[0])
                    if new is not None and _norm_args(cur_args) != _norm_cost(refs[0]) and _norm_args(new) == _norm_cost(refs[0]):
                        edits.append((b0 + cstart, b0 + cend, new)); counts["cost"] += 1
            # P/T
            rp, rt = str(card.get("power")), str(card.get("toughness"))
            if re.fullmatch(r"-?\d+", rp) and re.fullmatch(r"-?\d+", rt):
                pm = re.search(r"(power:\s*)(-?\d+)", body); tm = re.search(r"(toughness:\s*)(-?\d+)", body)
                if pm and tm and (pm.group(2), tm.group(2)) != (rp, rt):
                    edits.append((b0 + pm.start(2), b0 + pm.end(2), rp))
                    edits.append((b0 + tm.start(2), b0 + tm.end(2), rt)); counts["pt"] += 1
            # creature subtypes
            ctf = find_vec(body, "creature_types"); sub = creature_face_subtypes(card)
            if ctf and sub is not None:
                code = set(re.findall(r"CreatureType::(\w+)", ctf[2]))
                if code and set(sub) and code != set(sub) and not code.issubset(set(sub)) and all(t in VALID_CT for t in sub):
                    new = "vec![" + ", ".join(f"CreatureType::{t}" for t in sub) + "]"
                    edits.append((b0 + ctf[0], b0 + ctf[1], new)); counts["type"] += 1
        if edits and apply:
            out = text
            for s, e, new in sorted(edits, reverse=True): out = out[:s] + new + out[e:]
            out = ensure_mana_imports(out)
            src.write_text(out)
    print(("APPLIED" if apply else "DRY-RUN") + f"  cost={counts['cost']} pt={counts['pt']} type={counts['type']}")

def _norm_cost(mc): return tuple(sorted(re.findall(r"\{[^}]+\}", mc or "")))
def _norm_args(args):
    # rebuild scryfall tokens from a 'generic(2), b(), colorless(1)' arg string
    toks = []
    for a in re.split(r",(?![^()]*\))", args):
        a = a.strip()
        m = re.match(r"generic\((\d+)\)", a);  toks += (["{%d}" % int(m.group(1))] if m else [])
        m = re.match(r"colorless\((\d+)\)", a); toks += (["{C}"] * int(m.group(1)) if m else [])
        if a == "x()": toks.append("{X}")
        for fn, s in [("w","W"),("u","U"),("b","B"),("r","R"),("g","G")]:
            if a == f"{fn}()": toks.append("{%s}" % s)
        m = re.match(r"hybrid\(Color::(\w+),\s*Color::(\w+)\)", a)
        if m: toks.append("{%s/%s}" % ({v:k for k,v in COLORNAME.items()}[m.group(1)], {v:k for k,v in COLORNAME.items()}[m.group(2)]))
        m = re.match(r"mono_hybrid\((\d+),\s*Color::(\w+)\)", a)
        if m: toks.append("{%s/%s}" % (m.group(1), {v:k for k,v in COLORNAME.items()}[m.group(2)]))
        m = re.match(r"phyrexian\(Color::(\w+)\)", a)
        if m: toks.append("{%s/P}" % {v:k for k,v in COLORNAME.items()}[m.group(1)])
    return tuple(sorted(toks))

if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if a != "--apply"]
    run(args, "--apply" in sys.argv)
