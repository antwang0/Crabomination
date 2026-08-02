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
KMAP = {
    "FirstStrike": "First strike",
    "DoubleStrike": "Double strike",
    # Scryfall reports every parameterized protection as plain "Protection".
    "ProtectionFromCreatureType": "Protection",
    "ProtectionFromSpellSubtype": "Protection",
    "ProtectionFromCardType": "Protection",
    "ProtectionFromColoredSpells": "Protection",
    "ProtectionFromCreatures": "Protection",
    "ProtectionFromEverything": "Protection",
    "ProtectionFromInstants": "Protection",
    "ProtectionFromMonocolored": "Protection",
    "ProtectionFromMulticolored": "Protection",
    "ProtectionFromSpells": "Protection",
}

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

def norm(mc):
    """Mana cost as an order-insensitive symbol multiset. An empty cost and
    `{0}` are the same object (`cost(&[])` is how the catalog spells {0})."""
    syms = tuple(sorted(re.findall(r"\{[^}]+\}", mc or "")))
    return () if syms == ("{0}",) else syms

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

def face_for(card, name):
    """The `card_faces` entry whose name matches `name`, else None.

    A flip / DFC cache entry carries the FRONT face's stats at the top level,
    so a back-face definition (Tok-Tok, Stabwhisker) has to be compared
    against its own face or every one of them reads as drift.
    """
    for f in card.get("card_faces") or []:
        if f.get("name", "").lower() == name.lower():
            return f
    return None

# Scryfall spells a few creature types with punctuation the enum drops.
TYPE_ALIAS = {"Assembly-Worker": "AssemblyWorker", "Time Lord": "TimeLord"}

def creature_subtypes_ref(card, face=None):
    faces = [face] if face else (card.get("card_faces") or [])
    for tl in [f.get("type_line") for f in faces] + ([] if face else [card.get("type_line")]):
        if tl and "Creature" in tl and "—" in tl:
            raw = tl.split("—", 1)[1].split("//")[0].strip()
            for k, v in TYPE_ALIAS.items():
                raw = raw.replace(k, v)
            return set(raw.split())
    return None

def ref_keywords(card, face=None):
    # Scryfall lists flip-card keywords at the card level only, so a face with
    # an empty list carries no usable reference — signal "skip" with None.
    if face is not None:
        return set(face["keywords"]) & ENGINE_KW if face.get("keywords") else None
    out = set(card.get("keywords", []) or [])
    for f in card.get("card_faces", []) or []:
        out |= set(f.get("keywords", []) or [])
    return out & ENGINE_KW

def set_of(path):
    rel = path.relative_to(SETS)
    return rel.parts[0] if len(rel.parts) > 1 else rel.stem

FUNC = re.compile(r"pub fn (\w+)\(\)\s*->\s*CardDefinition\s*\{")
CTOR = re.compile(r"\b(CardDefinition|TokenDefinition)\s*\{")
NAME = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')

def strip_token_literals(body):
    """`body` with every balanced `TokenDefinition { … }` span blanked out.

    Token literals carry their own name/power/toughness/subtypes/keywords; left
    in place they shadow the card's own fields for every token-minting card.
    """
    out, i = [], 0
    for m in re.finditer(r"\bTokenDefinition\s*\{", body):
        if m.start() < i:
            continue
        out.append(body[i:m.start()])
        j, d = m.end(), 1
        while j < len(body) and d:
            if body[j] == "{":
                d += 1
            elif body[j] == "}":
                d -= 1
            j += 1
        i = j
    out.append(body[i:])
    return "".join(out)

def own_field(body, field):
    """Match for `field:` at the top level of the function's own
    `CardDefinition { … }` literal.

    A nested struct can carry the same key — `StaticAbility { PumpPT { power: 3
    } }` shadows a card's printed `power:` — so the scan tracks brace depth from
    the CardDefinition literal and only accepts depth-1 hits.
    """
    for cm in re.finditer(r"\bCardDefinition\s*\{", body):
        depth, i = 1, cm.end()
        while i < len(body) and depth:
            ch = body[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    break
            elif depth == 1:
                m = re.compile(field).match(body, i)
                if m:
                    return m
            i += 1
    return None

def card_def_name(body):
    """First `name:` belonging to a CardDefinition, skipping token literals.

    A card that mints a token defines the token first (Roxanne's Meteorite,
    Crib Swap's Shapeshifter), so the naive "first name in the body" reads the
    token's name and audits the wrong card.
    """
    return own_field(body, r'name:\s*"((?:[^"\\]|\\.)*)"')

# ── Local card-shape helpers ────────────────────────────────────────────────
#
# Most set modules build their cards through a file-local helper
# (`fn creature(name, cost, types, p, t) -> CardDefinition`) and pass the
# printed stats positionally: `..creature("Aven Brigadier", cost(&[…]), …)`.
# Those cards carry no top-level `name:` / `power:` field, so the audit used to
# skip them entirely — most of the classic sets went unchecked. `HELPERS`
# records, per file, which positional parameter of each helper feeds which
# CardDefinition field; `inline_helper_call` then splices the call's arguments
# into a synthetic literal the existing field scans can read.

HELPER_DEF = re.compile(
    r"\nfn (\w+)\(([^)]*)\)\s*->\s*CardDefinition\s*\{", re.S
)
FIELD_FROM_PARAM = {
    "name": "name",
    "cost": "cost",
    "power": "power",
    "toughness": "toughness",
    "creature_types": "creature_types",
}

def helper_table(text):
    """`{helper_name: {field: positional_index}}` for one source file."""
    out = {}
    for m in HELPER_DEF.finditer(text):
        params = [
            p.split(":")[0].strip()
            for p in re.split(r",(?![^<>()]*[>)])", m.group(2))
            if p.strip()
        ]
        j, depth = m.end(), 1
        while j < len(text) and depth:
            depth += (text[j] == "{") - (text[j] == "}")
            j += 1
        body = text[m.end() : j]
        mapping = {}
        for field in ("name", "cost", "power", "toughness", "creature_types"):
            # `power: p,` (explicit) or `name,` (shorthand field init).
            fm = re.search(r"\b" + field + r":\s*(\w+)\s*[,}]", body) or (
                re.search(r"\b(" + field + r")\s*,", body)
            )
            if fm and fm.group(1) in params:
                mapping[field] = params.index(fm.group(1))
        # A helper that layers on another helper (`legend` → `creature`)
        # forwards its own parameters; resolve one level.
        base = re.search(r"\.\.(\w+)\(", body)
        if base and base.group(1) in out:
            for field, idx in out[base.group(1)].items():
                arg = split_args(call_args(body, base.group(1)))
                if idx < len(arg) and arg[idx].strip() in params:
                    mapping.setdefault(field, params.index(arg[idx].strip()))
        if mapping:
            out[m.group(1)] = mapping
    return out

def call_args(body, fname):
    """The raw argument text of the first `fname(` call in `body`."""
    m = re.search(r"\b" + re.escape(fname) + r"\s*\(", body)
    if not m:
        return ""
    i, depth = m.end(), 1
    while i < len(body) and depth:
        depth += (body[i] == "(") - (body[i] == ")")
        i += 1
    return body[m.end() : i - 1]

def split_args(argtext):
    out, depth, cur = [], 0, ""
    for ch in argtext:
        if ch in "([{":
            depth += 1
        elif ch in ")]}":
            depth -= 1
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        out.append(cur)
    return out

def inline_helper_call(body, helpers):
    """Rewrite `..helper(args)` into the fields the helper would have set.

    Returns `body` unchanged when it already carries its own `name:` field or
    uses no known helper — the field scans then behave exactly as before.
    """
    if card_def_name(body) is not None:
        return body
    m = re.search(r"\.\.(\w+)\(", body)
    fname = m.group(1) if m else None
    if fname is None:
        # A bare `helper("Name", …)` tail with no struct wrapper.
        m2 = re.search(r"^\s*(\w+)\(", body, re.M)
        fname = m2.group(1) if m2 else None
    if fname not in helpers:
        return body
    args = split_args(call_args(body, fname))
    fields = []
    for field, idx in helpers[fname].items():
        if idx >= len(args):
            continue
        val = args[idx].strip()
        if field == "creature_types":
            fields.append(f"creature_types: {val},")
        else:
            fields.append(f"{field}: {val},")
    if not fields:
        return body
    return "CardDefinition {" + "".join(fields) + "}" + body

def audit():
    per_set = {}      # set -> dict(checked, cost[], pt[], type[], kw[])
    for src in sorted(SETS.rglob("*.rs")):
        s = set_of(src)
        d = per_set.setdefault(s, {"checked": 0, "cost": [], "pt": [], "type": [], "kw": []})
        text = src.read_text()
        helpers = helper_table(text)
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end()); body = text[m.end():nxt.start() if nxt else len(text)]
            body = strip_token_literals(body)
            body = inline_helper_call(body, helpers)
            nm = card_def_name(body)
            if not nm: continue
            card = CACHE_LC.get(nm.group(1).lower())
            if not card: continue
            face = face_for(card, nm.group(1))
            d["checked"] += 1
            tag = (nm.group(1), src.name, m.group(1))
            # cost
            cargs = toplevel_cost_args(body)
            if cargs is not None:
                syms = [sym(c) for c in re.split(r",(?![^()]*\))", cargs) if c.strip()]
                if all(syms):
                    got = "".join(syms)
                    refs = [card.get("mana_cost")] + [f.get("mana_cost") for f in card.get("card_faces", []) or []]
                    if face is not None and face.get("mana_cost") is not None:
                        refs = [face.get("mana_cost")]
                    refs = [r for r in refs if r]
                    if refs and all(norm(got) != norm(r) for r in refs):
                        d["cost"].append((tag, got, "|".join(refs)))
            # P/T
            pm = own_field(body, r"power:\s*(-?\d+)")
            tm = own_field(body, r"toughness:\s*(-?\d+)")
            ref_pt = face if face is not None and face.get("power") is not None else card
            numeric = lambda v: v is not None and re.fullmatch(r"-?\d+", str(v))
            if pm and tm and numeric(ref_pt.get("power")) and numeric(ref_pt.get("toughness")):
                if (pm.group(1), tm.group(1)) != (str(ref_pt["power"]), str(ref_pt["toughness"])):
                    d["pt"].append((tag, f"{pm.group(1)}/{tm.group(1)}", f"{ref_pt['power']}/{ref_pt['toughness']}"))
            # creature subtypes
            i = body.find("creature_types:")
            ctv = vec_after(body, i) if i >= 0 else None
            ref_ct = creature_subtypes_ref(card, face)
            if ctv is not None and ref_ct is not None:
                code_ct = set(re.findall(r"CreatureType::(\w+)", ctv))
                if code_ct and ref_ct and code_ct != ref_ct and not code_ct.issubset(ref_ct):
                    d["type"].append((tag, sorted(code_ct), sorted(ref_ct)))
            # keywords (top-level only)
            kwv = toplevel_keywords(body)
            if kwv is not None:
                code_kw = {KMAP.get(k, k) for k in re.findall(r"Keyword::(\w+)", kwv)} & ENGINE_KW
                rk = ref_keywords(card, face)
                if rk is not None and code_kw != rk:
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
