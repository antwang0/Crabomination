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

# Keywords the engine spells with a payload Scryfall reports plainly.
KMAP.update({
    "HexproofFromColor": "Hexproof",
    "HexproofFromMonocolored": "Hexproof",
    "HexproofFromMulticolored": "Hexproof",
    "HexproofFromAbilities": "Hexproof",
    "ProtectionFromMatching": "Protection",
    "ProtectionFromManaValueExcept": "Protection",
    "ProtectionFromManaValueParity": "Protection",
    "ProtectionFromOwnColors": "Protection",
})

KW_FN = re.compile(r"\nfn (\w+)\(\)\s*->\s*Keyword\s*\{\s*Keyword::(\w+)")


def kw_fn_table(text):
    """`{fn_name: variant}` for a file-local `fn ward_1() -> Keyword`."""
    return {m.group(1): m.group(2) for m in KW_FN.finditer(text)}


def top_level_items(vec_body):
    """The comma-separated elements of a `vec![…]` body, at bracket depth 0."""
    out, depth, cur = [], 0, ""
    for ch in vec_body:
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


def code_keywords(vec_body, kwfns):
    """The PRINTED keyword of each top-level element.

    A flat `Keyword::(\w+)` scan over the whole vector reads the keyword inside
    a payload as if the card had it: `Keyword::CantBeBlockedExceptBy(Box::new(
    R::HasKeyword(Keyword::Flying)))` made Treetop Scout, Treetop Rangers,
    Elven Riders, Spire Tracer and Silhana Ledgewalker all read as fliers, and
    every one of them was already correct (2026-08-30).
    """
    out = set()
    for item in top_level_items(vec_body):
        item = item.strip()
        m = re.match(r"(?:\w+::)*Keyword::(\w+)", item)
        if m:
            out.add(KMAP.get(m.group(1), m.group(1)))
            continue
        m = re.match(r"(\w+)\(\)", item)
        if m and m.group(1) in kwfns:
            v = kwfns[m.group(1)]
            out.add(KMAP.get(v, v))
    return out & ENGINE_KW


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
    """Same rule as `toplevel_cost_args`: the card's own literal, not a bound
    one's."""
    m = own_field(body, r"keywords:")
    return vec_after(body, m.start()) if m else None

def toplevel_cost_args(body):
    """The depth-1 `cost: cost(&[...])` inner args of the card's own
    `CardDefinition` literal — never a nested cost (an ability's) and never a
    *bound* literal's (an MDFC back face in a preceding `let`). Goes through
    `own_field`, which owns both of those rules."""
    m = own_field(body, r"cost:\s*cost\(&\[")
    if m is None:
        return None
    start, d, kk = m.end(), 1, m.end()
    while kk < len(body) and d:
        if body[kk] == "[": d += 1
        elif body[kk] == "]": d -= 1
        kk += 1
    return body[start:kk-1]

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

# CR 205.4 / 205.2a — the words left of the em dash, split into the two
# groups the engine models as `Supertype` and `CardType`. `Tribal` is the
# pre-2021 printing of `Kindred`. Anything outside these two sets (a Dungeon,
# a Sticker, an Attraction's "Artifact Attraction" oddities) makes the line
# unreadable for this audit and the card is SKIPPED, not flagged.
# The variant NAME, however the file spells the path. `modern.rs` writes
# `Sup::Legendary` (`use crate::card::Supertype as Sup`), and a scan keyed on
# `Supertype::` read an empty list on every one of those — 30-odd correct
# legends reported as missing the supertype (2026-08-30).
ST_VARIANT = re.compile(r"\b(?:\w+::)*(Legendary|Basic|Snow|World|Ongoing)\b")
CT_VARIANT = re.compile(
    r"\b(?:\w+::)*(Land|Creature|Artifact|Enchantment|Planeswalker|Battle|Instant"
    r"|Sorcery|Kindred|Scheme|Vanguard|Conspiracy|Plane|Phenomenon)\b"
)

REF_SUPERS = {"Basic", "Legendary", "Snow", "World", "Ongoing"}
REF_TYPES = {
    "Artifact", "Battle", "Creature", "Enchantment", "Instant", "Kindred",
    "Tribal", "Land", "Planeswalker", "Sorcery", "Scheme", "Vanguard",
    "Conspiracy", "Plane", "Phenomenon",
}


def type_line_ref(card, face=None):
    """`(supertypes, card_types)` off the type line, or None when unreadable.

    A split / MDFC line ("Creature — Human // Instant") describes two objects,
    so without a matched face there is no single answer and this returns None
    rather than a guess.
    """
    tl = (face or card).get("type_line") or ""
    if not tl or (face is None and "//" in tl):
        return None
    left = tl.split("—", 1)[0].strip()
    words = left.split()
    supers, types = set(), set()
    for w in words:
        if w in REF_SUPERS:
            supers.add(w)
        elif w in REF_TYPES:
            types.add("Kindred" if w == "Tribal" else w)
        else:
            return None
    return (supers, types) if types else None


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
        # Skip a literal that is BOUND rather than returned — `let aura_back =
        # CardDefinition { … }`, `back_face: Some(CardDefinition { … })`. Both
        # carry the same `name:` as the card, so the first-literal scan read
        # Bronzehide Lion's Aura back face as the card itself and called a
        # Creature an Enchantment (2026-08-30).
        before = body[: cm.start()].rstrip()
        if before[-1:] in ("=", "("):
            continue
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

# Literals that can legitimately carry a *printed* characteristic. Anything
# else nested inside the helper's `CardDefinition { … }` — an `Effect`, a
# `StaticAbility`, an `ActivatedAbility` — carries its own `cost:` / `power:`
# and must not claim the helper's parameter of that name.
PRINTED_NEST = ("CardDefinition", "Subtypes")
LIT_NAME = re.compile(r"([A-Za-z_][A-Za-z_0-9]*)\s*$")


def own_helper_fields(body, field_re):
    """Every `field_re` match inside the helper's own `CardDefinition { … }`
    whose enclosing literals are all in `PRINTED_NEST`.

    `helper_table` used a flat `findall`, so `elder_dragon`'s
    `SacrificeSourceUnlessPay { cost: upkeep_cost }` claimed the `cost`
    mapping and the four Legends Elder Dragons read as cost drift against
    their own upkeep (2026-08-30). `creature_types` genuinely sits one level
    down inside `Subtypes { … }`, so this is a name check, not a depth cap.
    """
    pat = re.compile(field_re)
    if not re.search(r"\bCardDefinition\s*\{", body):
        # A helper that only forwards (`bfz::ally` -> `creature(name, …)`) has
        # no literal to be inside of; the flat scan is the whole scan there,
        # and it is what maps `name` for the 106 cards those helpers build.
        yield from pat.finditer(body)
        return
    for cm in re.finditer(r"\bCardDefinition\s*\{", body):
        # Same bound-literal rule as `own_field`.
        if body[: cm.start()].rstrip()[-1:] in ("=", "("):
            continue
        stack, i = ["CardDefinition"], cm.end()
        while i < len(body) and stack:
            ch = body[i]
            if ch == "{":
                # A struct literal is `Name {`; anything else (`if cond {`, a
                # bare block, a closure body) is transparent and inherits the
                # frame it sits in — `tor2::dreams` builds its base through
                # `..if sorcery_speed { sorcery(name, …) }`.
                nm = LIT_NAME.search(body[:i])
                lit = nm.group(1) if nm and nm.group(1)[:1].isupper() else stack[-1]
                stack.append(lit)
                i += 1
                continue
            if ch == "}":
                stack.pop()
                i += 1
                continue
            if all(n in PRINTED_NEST for n in stack):
                m = pat.match(body, i)
                if m:
                    yield m
            i += 1


VECFN = re.compile(
    r"\nfn (\w+)\(\)\s*->\s*Vec<(Supertype|CardType)>\s*\{\s*vec!\[([^\]]*)\]", re.S
)


def vec_fn_table(text):
    """`{fn_name: literal}` for a file-local `fn legendary() -> Vec<Supertype>`.

    `chk2.rs` writes `supertypes: legendary()` on 26 cards. A reader that only
    understands `vec![Supertype::Legendary]` sees an empty list there and
    reports every one of them as a missing supertype — which is how this audit
    first read 25 correct Kamigawa legends as defects (2026-08-30).
    """
    return {m.group(1): m.group(3) for m in VECFN.finditer(text)}


def field_vec(body, field, vecfns):
    """The `vec![…]` body for `field` at depth 1, resolving a `field: helper()`
    indirection through `vecfns`. `None` when the field is absent."""
    m = own_field(body, re.escape(field) + r":")
    if m is None:
        return None
    v = vec_after(body, m.start())
    if v is not None:
        return v
    call = re.match(re.escape(field) + r":\s*(\w+)\(\)", body[m.start():])
    return vecfns.get(call.group(1)) if call else None


def helper_table(text):
    """`({helper_name: {field: positional_index}}, {helper_name: {field: literal}})`.

    The second map is for the fields a helper sets to a *constant* rather than
    from a parameter — `card_types`, `supertypes` — which is how every
    `fn creature` / `fn sorcery` / `fn legend` in this catalog spells them.
    """
    out, consts = {}, {}
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
            # `power: p,` (explicit) or `name,` (shorthand field init). Take the
            # LAST assignment: a helper that also builds a DFC back face
            # (`vanilla_werewolf`) writes the back's fields first and returns
            # the front, so the first match would audit the wrong face.
            hits = [
                m.group(1)
                for m in own_helper_fields(body, r"\b" + field + r":\s*(\w+)\s*[,}]")
            ] or [
                m.group(1) for m in own_helper_fields(body, r"\b(" + field + r")\s*,")
            ]
            hits = [h for h in hits if h in params]
            if hits:
                mapping[field] = params.index(hits[-1])
        # A helper that layers on another helper (`legend` → `creature`)
        # forwards its own parameters; resolve one level.
        base = re.search(r"\.\.(\w+)\(", body)
        if base and base.group(1) in out:
            for field, idx in out[base.group(1)].items():
                arg = split_args(call_args(body, base.group(1)))
                if idx < len(arg) and arg[idx].strip() in params:
                    mapping.setdefault(field, params.index(arg[idx].strip()))
        # `card_types` and `supertypes` are almost never a helper *parameter*
        # — a `fn creature(...)` writes `card_types: vec![CardType::Creature]`
        # as a literal, and a `fn legend(...)` adds
        # `supertypes: vec![Supertype::Legendary]` on top of it. Record those
        # so the two type columns can see the ~11 k cards built through a
        # helper instead of skipping them.
        for field in ("card_types", "supertypes"):
            for mm in own_helper_fields(body, r"\b" + field + r":\s*vec!\["):
                v = vec_after(body, mm.start())
                # A *constant* is a plain list of `CardType::X`. `recent311`'s
                # `spell` helper writes `vec![if sorcery { Sorcery } else
                # { Instant }]`, which is a parameter wearing a literal's
                # clothes — capturing it read 85 cards as both types at once.
                var = ST_VARIANT if field == "supertypes" else CT_VARIANT
                if v is not None and re.fullmatch(r"\s*(?:[\w:]+\s*,?\s*)+", v) and var.findall(v):
                    consts.setdefault(m.group(1), {})[field] = v
                break
        if base and base.group(1) in consts:
            for field, v in consts[base.group(1)].items():
                consts.setdefault(m.group(1), {}).setdefault(field, v)
        if mapping:
            out[m.group(1)] = mapping
    return out, consts

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

def inline_helper_call(body, helpers, consts=None):
    """Splice a `..helper(args)` call's fields into the body's own literal.

    Returns `body` unchanged when it already carries its own `name:` field or
    uses no known helper — the field scans then behave exactly as before.

    **Every spliced field is guarded on the body not already having one at
    depth 1.** The splice is inserted at the *front* of the card's own
    `CardDefinition { … }`, and a depth-1 scan takes the first hit, so an
    unguarded splice would shadow a card that overrides the helper
    (`CardDefinition { power: 3, ..creature("X", …, 2, 2) }`) and manufacture
    the drift the audit is looking for.
    """
    consts = consts or {}
    if card_def_name(body) is not None:
        return body
    m = re.search(r"\.\.(\w+)\(", body)
    fname = m.group(1) if m else None
    if fname is None:
        # A bare `helper("Name", …)` tail with no struct wrapper.
        m2 = re.search(r"^\s*(\w+)\(", body, re.M)
        fname = m2.group(1) if m2 else None
    if fname not in helpers and fname not in consts:
        return body
    args = split_args(call_args(body, fname))
    fields = []
    for field, lit in consts.get(fname, {}).items():
        if own_field(body, re.escape(field) + r":") is None:
            fields.append(f"{field}: vec![{lit}],")
    for field, idx in helpers.get(fname, {}).items():
        if idx >= len(args):
            continue
        if own_field(body, re.escape(field) + r":") is not None:
            continue
        fields.append(f"{field}: {args[idx].strip()},")
    if not fields:
        return body
    cd = body.find("CardDefinition {")
    if cd < 0:
        return "CardDefinition {" + "".join(fields) + "}" + body
    at = cd + len("CardDefinition {")
    return body[:at] + "".join(fields) + body[at:]

def audit():
    per_set = {}      # set -> dict(checked, cost[], pt[], type[], kw[])
    for src in sorted(SETS.rglob("*.rs")):
        s = set_of(src)
        d = per_set.setdefault(s, {"checked": 0, "cost": [], "pt": [], "type": [], "ct": [], "st": [], "kw": []})
        text = src.read_text()
        helpers, hconsts = helper_table(text)
        vecfns = vec_fn_table(text)
        kwfns = kw_fn_table(text)
        for m in FUNC.finditer(text):
            nxt = FUNC.search(text, m.end()); body = text[m.end():nxt.start() if nxt else len(text)]
            # Stop at the next TOP-LEVEL `fn`, not only at the next `pub fn`:
            # a private helper defined between two card factories is otherwise
            # inside this card's body, and its own `CardDefinition { card_types:
            # vec![CardType::Land] }` answered for the card above it. That read
            # Dominaria's Judgment and Pay No Heed — both `instant(...)` one-liners
            # — as Lands (2026-08-30).
            cut = re.search(r"\n(?:pub(?:\([^)]*\))?\s+)?fn \w+", body)
            if cut:
                body = body[: cut.start()]
            body = strip_token_literals(body)
            body = inline_helper_call(body, helpers, hconsts)
            nm = card_def_name(body)
            if not nm: continue
            card = CACHE_LC.get(nm.group(1).lower())
            if not card: continue
            face = face_for(card, nm.group(1))
            d["checked"] += 1
            tag = (nm.group(1), src.name, m.group(1))
            # cost. A card with no `cost:` at all and a `..Default::default()`
            # tail is castable for free — the same "an omitted field is a
            # value" reading the P/T block below spells out. Lands and the
            # other genuinely costless faces answer `""` on both sides.
            cargs = toplevel_cost_args(body)
            if (cargs is None
                    and own_field(body, r"cost:") is None
                    and own_field(body, r"\.\.Default::default\(\)") is not None):
                cargs = ""
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
            # P/T. ⚠ **An omitted field is a value, not an absence.** Requiring
            # both `power:` and `toughness:` skipped every card that spells one
            # of them and lets `..Default::default()` supply 0 for the other —
            # and 0 is a printed characteristic like any number. Paradise Druid
            # shipped as a 0/2 (printed 2/1) with its doc comment agreeing, so
            # `audit_doc_drift` could not see it either. The `..Default::
            # default()` tail is the gate: without it the missing field could
            # come from a helper this file's table does not know, and a
            # defaulted 0 would be invented rather than read.
            pm = own_field(body, r"power:\s*(-?\d+)")
            tm = own_field(body, r"toughness:\s*(-?\d+)")
            defaulted = own_field(body, r"\.\.Default::default\(\)") is not None
            # ⚠ A Spacecraft's printed P/T is its station band's, and the card
            # is a 0/0 non-creature until it is stationed. Scryfall reports the
            # band number in `power`/`toughness`, so all 21 EOE Spacecraft read
            # as 0/0-vs-N/M defects without this gate.
            if own_field(body, r"station:\s*vec!\[") is not None:
                pm = tm = None
                defaulted = False
            got_p = pm.group(1) if pm else ("0" if defaulted else None)
            got_t = tm.group(1) if tm else ("0" if defaulted else None)
            ref_pt = face if face is not None and face.get("power") is not None else card
            numeric = lambda v: v is not None and re.fullmatch(r"-?\d+", str(v))
            if got_p and got_t and numeric(ref_pt.get("power")) and numeric(ref_pt.get("toughness")):
                if (got_p, got_t) != (str(ref_pt["power"]), str(ref_pt["toughness"])):
                    d["pt"].append((tag, f"{got_p}/{got_t}", f"{ref_pt['power']}/{ref_pt['toughness']}"))
            # creature subtypes
            ctm = next(own_helper_fields(body, r"creature_types:"), None)
            ctv = vec_after(body, ctm.start()) if ctm else None
            ref_ct = creature_subtypes_ref(card, face)
            if ctv is not None and ref_ct is not None:
                code_ct = set(re.findall(r"CreatureType::(\w+)", ctv))
                if code_ct and ref_ct and code_ct != ref_ct and not code_ct.issubset(ref_ct):
                    d["type"].append((tag, sorted(code_ct), sorted(ref_ct)))
            # card types and supertypes (CR 205.2a / 205.4)
            ref_tl = type_line_ref(card, face)
            if ref_tl is not None:
                ref_supers, ref_types = ref_tl
                ctypes = field_vec(body, "card_types", vecfns)
                if ctypes is not None:
                    code_types = set(CT_VARIANT.findall(ctypes))
                    # A command-zone object shares its name with a normal card
                    # (the Vanguard avatar "Maraxus of Keld" and the Legends
                    # creature), and the cache is keyed by name, so the
                    # reference is about the other one. Skip, don't flag.
                    if code_types & {"Vanguard", "Scheme", "Plane", "Phenomenon", "Conspiracy"} \
                            and not (code_types & ref_types):
                        code_types = set()
                    if code_types and code_types != ref_types:
                        d["ct"].append((tag, sorted(code_types), sorted(ref_types)))
                    stv = field_vec(body, "supertypes", vecfns) or ""
                    code_supers = set(ST_VARIANT.findall(stv))
                    if code_types and code_supers != ref_supers:
                        d["st"].append((tag, sorted(code_supers), sorted(ref_supers)))
            # keywords (top-level only)
            kwv = toplevel_keywords(body)
            if kwv is not None:
                code_kw = code_keywords(kwv, kwfns)
                rk = ref_keywords(card, face)
                if rk is not None and code_kw != rk:
                    d["kw"].append((tag, sorted(code_kw), sorted(rk)))
    return per_set

def main():
    per_set = audit()
    detail = sys.argv[1] if len(sys.argv) > 1 else None
    
    if detail:
        d = per_set.get(detail)
        if not d: sys.exit(f"no such set '{detail}' (have: {', '.join(sorted(per_set))})")
        for dim in ("cost", "pt", "type", "ct", "st", "kw"):
            print(f"\n=== {dim.upper()} drift in {detail} ({len(d[dim])}) ===")
            for tag, got, ref in d[dim]:
                print(f"  {tag[0]}  ({tag[1]}::{tag[2]})\n    code={got}  scryfall={ref}")
    else:
        dims = ("cost", "pt", "type", "ct", "st", "kw")
        print(f"{'set':<12}{'checked':>8}{'cost':>6}{'P/T':>6}{'sub':>6}{'type':>6}{'super':>6}{'kw':>6}")
        print("-" * 56)
        tot = {"checked": 0, **{k: 0 for k in dims}}
        for s in sorted(per_set, key=lambda s: -sum(len(per_set[s][k]) for k in dims)):
            d = per_set[s]
            if not d["checked"]: continue
            for k in tot: tot[k] += d["checked"] if k == "checked" else len(d[k])
            print(f"{s:<12}{d['checked']:>8}" + "".join(f"{len(d[k]):>6}" for k in dims))
        print("-" * 56)
        print(f"{'TOTAL':<12}{tot['checked']:>8}" + "".join(f"{tot[k]:>6}" for k in dims))
        print("\nDetail for a set:  python3 scripts/audit_catalog_stats.py <set>")


if __name__ == "__main__":
    main()
