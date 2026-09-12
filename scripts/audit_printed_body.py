#!/usr/bin/env python3
"""The fourteenth column: does a card's own COST and BODY match what it prints?

`audit_doc_drift.py` reads the doc comment and compares it with the body and
the oracle, so it audits a factory only where the doc *states* a cost or a
P/T in the codebase's `Name — {2}{R} 3/3 …` shape. This reads the body alone,
so it covers the factories that shape misses: a doc that omits the cost, a doc
that is prose, a factory with no doc at all.

Every other column audit (activated-ability costs, timing, trigger events,
scopes, filters, amounts, ...) found shipped cards at the wrong value; this
one is the card's own price and body, which is the one field every other
column assumes is right.

    python3 scripts/audit_printed_body.py            # the table
    python3 scripts/audit_printed_body.py --rows 0   # every row

COVERAGE, 2026-09-12: **14,873 factories compared**, from 10,270. The gap the
2026-09-11 header described ("219 real spells over ~40 bespoke helpers") was a
mis-read of its own skip counts; the reader had three holes and none of them was
the helper signatures:

  * **the `..base` struct-update form — 2,890 factories, 13 % of the catalog.**
    `CardDefinition { activated_abilities: .., ..creature("Name", cost(&[r()]),
    ..) }` has a literal, so the helper fallback never ran, and the literal has
    no `cost:` of its own, so the literal read found nothing. `base_struct_cost`
    reads it, anchored on the card's own name.
  * **`crate::mana::w()` and multi-line costs** — a third of the catalog spells
    the symbols with their path, and `top_fields` emits one line per field, so
    `cost: cost(&[\n  generic(3),\n])` arrived as three lines. ~180 factories.
  * **factories named only by their `pub fn`** — 1,607 more, resolved through
    `CACHE_SLUG`. Safer than the head-string heuristic it backs up, not less:
    no string from the body is involved.

**THE TYPE LINE IS RARELY IN THE LITERAL**, which is why the column read only
10,409 factories and now reads **14,064**. Three idioms carry it somewhere else
and all three are followed (`resolve_type_line`):

  * the **base-struct** form — `..creature("Name", cost, types, 1, 1)`,
    `..god_weapon(..)`, `..legend(..)`;
  * the **wrapper** form — `legend(CardDefinition { .. })` over a
    `fn legend(mut def) { def.supertypes = ..; def }`, six Invasion legends;
  * a helper whose own base is another helper, up to five links.

When a link cannot be found the card is SKIPPED rather than reported: the first
cut of this column reported 77 rows of "supertypes: (none)" and every one was a
`..legend(..)` supplying what the literal never claimed.

What is left, with the reason:
  * `notyped` (809, from 4,464) — a chain this cannot follow.
  * `nocache` (3,772) — synthesized cards the oracle has never heard of.
  * `noname` (2,392) and `nonliteral` (405) — the bare-symbol helpers
    (`zubera("Name", r(), ..)`) and factories whose name is in neither place.
  * `faces` / `split` / `star` / `notaspell` — documented below, all deliberate.
    `notaspell` also drops a Vanguard avatar named after a card: Maraxus of Keld
    is both, and the oracle lookup cannot tell them apart.

**Proved by injection, not by its own zero** — one per idiom, because a gate
that cannot fail is worse than no gate. Breaking Agent of Stromgald's
`..creature("Agent of Stromgald", cost(&[r()]), ..)` to `{4}{R}{R}` reports the
cost row; Karn, Scion of Urza broken to `{3}` / `Creature` / no supertype
reports all three; `god_weapon`'s `supertypes:` removed reports Spear of
Heliod, Whip of Erebos and Hammer of Purphoros; and `fn legend(mut def)`'s
assignment emptied reports the six Invasion legends. **That last one silently
PASSED at first** — `legend` is defined in five files with three different
shapes, and the resolver took whichever came first, so an assigning wrapper was
reading a base-struct one's supertypes. Same-file first now.

⚠ SKIPPED, with the reason, and the skip counts are printed:
  * multi-face cards (`card_faces` in the oracle) — the factory's `cost:` is
    the front face and the comparison needs a face the body does not name;
  * split cards (`Name // Name`) for the same reason;
  * `*` / `*+1` power or toughness (characteristic-defining) — the body holds
    a placeholder the oracle cannot adjudicate;
  * a `cost:` that is not a literal `cost(&[..])` of symbol constructors
    (`with_x_value`, a `const`, a builder call);
  * a name the cache does not hold (synthesised cards, tokens).
"""
import argparse
import json
import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
CATALOG = ROOT / "crabomination_catalog" / "src"
CACHE = json.load(open(pathlib.Path(__file__).resolve().parent / ".scryfall_cache.json"))
CACHE_LC = {k.lower(): v for k, v in CACHE.items() if isinstance(v, dict)}
# The cache keyed the way a factory name is spelled: lowercase, letters and
# digits only. A factory whose `CardDefinition` carries no `name:` and whose
# head holds no string the old heuristic accepted is still named by its own
# `pub fn` — 1,682 of 1,692 such factories resolve here, and resolving the
# FUNCTION name against the oracle is safer than reading a string out of the
# body, which is how a Spirit token's name once became the card's.
CACHE_SLUG = {re.sub(r"[^a-z0-9]", "", k.lower()): v
              for k, v in CACHE.items() if isinstance(v, dict)}
CACHE_SLUG_KEYS = {k.replace("_", "") for k in ()} | set(CACHE_SLUG)

FN = re.compile(r"^pub fn ([a-z0-9_]+)\(\) -> (?:[A-Za-z_]+::)*CardDefinition \{", re.M)
# ⚠ THE FIELDS MUST COME FROM THE `CardDefinition` LITERAL, NOT THE FACTORY
# BLOCK. A factory commonly opens with a `let token = TokenDefinition { power:
# 1, toughness: 1, .. }` at the same indent, and an `ActivatedAbility` has a
# `cost:` — reading the block by regex gave the Spirit token's 1/1 as
# Heron-Blessed Geist's body and called 117 correct cards wrong. Brace-match
# the literal and read only its own depth-1 fields.
LITERAL = re.compile(r"(?:[A-Za-z_]+::)*CardDefinition \{")
ANYNAME = re.compile(r'"((?:[^"\\]|\\.)*)"')
NAME = re.compile(r'^name: "((?:[^"\\]|\\.)*)",', re.M)
# ⚠ MULTI-LINE AND PATH-TOLERANT. `top_fields` emits one line per depth-1
# field, so a `cost: cost(&[\n  generic(3),\n  r(),\n])` arrives as three
# lines and a single-line pattern misses it; and half the catalog spells the
# symbols `crate::mana::w()` rather than `w()`. Both were counted as
# `nonliteral` — 180-odd factories on the second alone.
COST = re.compile(r"^cost: (?:crate::mana::)?cost\(&\[(.*?)\]\)", re.M | re.S)
HELPER = re.compile(r'"(?:[^"\\]|\\.)*",\s*(?:crate::mana::)?cost\(&\[([^\]]*)\]\)')


def base_struct_cost(raw: str, name: str):
    """`CardDefinition { .., ..creature("Name", cost(&[r()]), ..) }` — the
    struct-update form, where the literal carries the card's *extras* and the
    base comes from a helper call in the `..base` position.

    **2,890 factories, 13 % of the catalog, and the audit skipped every one**:
    they have a `CardDefinition` literal, so the helper fallback never ran, and
    the literal has no `cost:` field of its own, so the literal read found
    nothing. Anchored on the card's own name rather than on "the first string
    followed by a cost", because an activated ability inside the same block can
    be built by a helper that takes a name too.
    """
    pat = re.compile(
        re.escape('"' + name + '"') + r",\s*(?:crate::mana::)?cost\(&\[(.*?)\]\)", re.S
    )
    m = pat.search(raw)
    return m.group(1) if m else None
# ⚠ ALIAS-TOLERANT ON PURPOSE. The catalog spells these three ways —
# `CardType::Creature`, an aliased `CT::Creature` / `Sup::Legendary`, and a
# helper (`supertypes: legendary()`, `subtypes: types(vec![..])`). A matcher
# keyed on `Supertype::` reported 49 wrong type lines and every one was an
# alias or a helper, clustered in two files, which is the tell.
TYPES = re.compile(r"^card_types: (.*)$", re.M)
SUPER = re.compile(r"^supertypes: (.*)$", re.M)
WORD = re.compile(r"\b([A-Z][a-z]+)\b")
HELPERS = {"legendary": "Legendary", "basic": "Basic", "snow": "Snow"}
POWER = re.compile(r"^power: (-?\d+),", re.M)
TOUGH = re.compile(r"^toughness: (-?\d+),", re.M)


# Every `fn` in the catalog that returns a `CardDefinition` — the helpers a
# factory defers to, not just the factories themselves.
ANYFN = re.compile(r"^(?:pub(?:\(crate\))? )?fn ([a-z0-9_]+)[(<]", re.M)
RET = re.compile(r"->\s*(?:[A-Za-z_]+::)*CardDefinition\s*\{")
# `helper(CardDefinition { .. })` — the WRAPPER idiom, as opposed to the
# `..helper(..)` base-struct one. `fn legend(mut def) { def.supertypes = ..; def }`
# is the shape, and six Invasion legends are built that way.
WRAPS = re.compile(r"^\s*([a-z0-9_]+)\(\s*(?:[A-Za-z_]+::)*CardDefinition\s*\{", re.M)
ASSIGNS = re.compile(r"\bdef\.(supertypes|card_types)\s*=")


def top_fields(block: str):
    """The depth-1 text of the first `CardDefinition { .. }` literal in `block`,
    one field per line with the indent stripped, or None when there is none.

    Depth counts BRACES ONLY — `cost(&[generic(4), w()])` has to survive whole,
    and a nested `TokenDefinition { power: 1 }` has to not. String literals are
    skipped while counting, because half the catalog's prompts contain `{2}`.
    """
    # Skip the signature line: `pub fn x() -> CardDefinition {` matches the
    # literal regex too, and reading the function body as the literal put every
    # field one level too deep (21,799 factories read as nameless).
    after_sig = block.find("\n") + 1
    m = LITERAL.search(block, after_sig)
    if not m:
        return None
    i, depth, out, line, in_str = m.end(), 1, [], [], False
    while i < len(block) and depth:
        c = block[i]
        if in_str:
            if c == "\\":
                if depth == 1:
                    line.append(block[i:i + 2])
                i += 2
                continue
            if c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                break
        if c == "\n" and not in_str:
            if depth == 1:
                out.append("".join(line).strip())
            line = []
        elif depth == 1:
            line.append(c)
        i += 1
    out.append("".join(line).strip())
    return "\n".join(out)


CARD_TYPES = {"Land", "Creature", "Artifact", "Enchantment", "Planeswalker",
              "Battle", "Instant", "Sorcery", "Kindred", "Tribal", "Scheme",
              "Vanguard"}
SUPERTYPES = {"Basic", "Legendary", "Snow", "World", "Ongoing"}
NOT_A_SPELL = ("Token", "Vanguard", "Scheme", "Plane ", "Phenomenon",
               "Conspiracy", "Emblem", "Dungeon", "Attraction", "Stickers")
SYM = re.compile(r"(\w+)\(([^()]*)\)")
LETTER = {"w": "W", "u": "U", "b": "B", "r": "R", "g": "G"}
COLOR = {"Color::White": "W", "Color::Blue": "U", "Color::Black": "B",
         "Color::Red": "R", "Color::Green": "G"}


def body_cost(src: str):
    """`cost(&[generic(3), r()])` -> "{3}{R}", or None when not literal."""
    out = []
    pos = 0
    # `crate::mana::w()` is the same symbol as `w()`; the paths are what sits
    # "between the symbols" for a third of the catalog.
    src = src.replace("crate::mana::", "")
    for m in SYM.finditer(src):
        if src[pos:m.start()].strip(" ,\n"):
            return None  # something between the symbols we do not model
        pos = m.end()
        fn, arg = m.group(1), m.group(2).strip()
        if fn in LETTER and not arg:
            out.append("{%s}" % LETTER[fn])
        elif fn == "x" and not arg:
            out.append("{X}")
        elif fn == "generic":
            if not arg.isdigit():
                return None
            if arg != "0" or not out:
                out.append("{%s}" % arg)
        elif fn == "colorless":
            if not arg.isdigit():
                return None
            out.append("{C}" * int(arg))
        elif fn == "snow_mana" and not arg:
            out.append("{S}")
        elif fn == "colored":
            if arg not in COLOR:
                return None
            out.append("{%s}" % COLOR[arg])
        elif fn == "hybrid":
            a = [COLOR.get(p.strip()) for p in arg.split(",")]
            if len(a) != 2 or None in a:
                return None
            out.append("{%s/%s}" % (a[0], a[1]))
        elif fn == "phyrexian":
            if arg not in COLOR:
                return None
            out.append("{%s/P}" % COLOR[arg])
        elif fn == "mono_hybrid":
            parts = [p.strip() for p in arg.split(",")]
            if len(parts) != 2 or not parts[0].isdigit() or parts[1] not in COLOR:
                return None
            out.append("{%s/%s}" % (parts[0], COLOR[parts[1]]))
        elif fn == "phyrexian_hybrid":
            a = [COLOR.get(p.strip()) for p in arg.split(",")]
            if len(a) != 2 or None in a:
                return None
            out.append("{%s/%s/P}" % (a[0], a[1]))
        else:
            return None
    if src[pos:].strip(" ,\n"):
        return None
    return "".join(out)


def norm(cost: str) -> str:
    """A `ManaCost` is a multiset, and Scryfall prints pips in a canonical
    WUBRG-rotated order the engine does not model — `{2}{U}{B}{G}` and
    `{2}{B}{G}{U}` are the same cost. Compare the sorted symbols, or the audit
    reports five pip-order rows for every real one (it did)."""
    # `{0}` and an empty cost are the SAME symbol list to the engine — an
    # empty `ManaCost`. What tells Ornithopter (`{0}`, castable) from Living
    # End (no cost, not castable) is `no_mana_cost`, which is the check above,
    # so comparing them here would report Claws of Gix forever.
    return "".join(sorted(p for p in re.findall(r"\{[^}]*\}", cost.upper()) if p != "{0}"))


def helper_body(blk: str):
    """`top_fields` for a HELPER, whose signature spans several lines.

    `top_fields` skips one line and then takes the first `CardDefinition {` it
    finds — right for a factory (`pub fn x() -> CardDefinition {` is one line),
    and wrong for `fn god_weapon(\n  name: &'static str,\n ..\n) ->
    CardDefinition {`, where that first match IS the function body's brace and
    the read comes back as `"\n}\n"`. Start after the return type instead.
    """
    m = RET.search(blk)
    return top_fields("fn _()\n" + blk[m.end():]) if m else None


def build_helper_index(catalog):
    """`name -> [(path, block)]` for every fn in the catalog that returns a
    `CardDefinition`, so a factory's `..base` can be resolved to its types."""
    index = {}
    for path in sorted(catalog.rglob("*.rs")):
        text = path.read_text()
        hits = [(m.start(), m.group(1)) for m in ANYFN.finditer(text)]
        for i, (pos, fn) in enumerate(hits):
            end = hits[i + 1][0] if i + 1 < len(hits) else len(text)
            blk = text[pos:end]
            if RET.search(blk[:400]) or ASSIGNS.search(blk):
                index.setdefault(fn, []).append((path, blk))
    return index


def parse_type_line(body: str):
    """`(card_types, supertypes, base_helper, declares_types, declares_supers)`
    off one depth-1 field text."""
    tm, sm = TYPES.search(body), SUPER.search(body)
    got_t = {w for w in WORD.findall(tm.group(1)) if w in CARD_TYPES} if tm else set()
    got_t = {"Kindred" if t == "Tribal" else t for t in got_t}
    got_s = set()
    if sm:
        got_s = {w for w in WORD.findall(sm.group(1)) if w in SUPERTYPES}
        got_s |= {v for k, v in HELPERS.items() if k + "()" in sm.group(1)}
    base = None
    for line in body.split("\n"):
        if line.startswith("..") and not line.startswith("..Default::default"):
            m = re.match(r"\.\.([a-z0-9_]+)\s*\(", line)
            if m:
                base = m.group(1)
    return got_t, got_s, base, tm is not None, sm is not None


def resolve_type_line(index, path, body, raw, depth=0):
    """The card's types and supertypes, following `..base` and wrapper helpers.

    Returns `(types, supers, ok)`. `ok` is False when a link in the chain is a
    helper this cannot find — saying nothing beats reporting the reader's own
    blind spot, which is how 77 `..legend(..)` cards read as "supertypes:
    (none)" the first time this column was opened.
    """
    got_t, got_s, base, has_t, has_s = parse_type_line(body)
    ok = True
    # The wrapper idiom sits OUTSIDE the literal, so it is read off `raw`.
    if raw is not None and depth == 0:
        after_sig = raw[raw.find("\n") + 1:]
        wm = WRAPS.match(after_sig)
        if wm:
            # Same-file first: `legend` is defined in five files with three
            # different shapes, and taking whichever came first made an
            # ASSIGNING wrapper read the supertypes of somebody else's
            # base-struct one — the injection that proved it silently passed.
            cands = [c for c in index.get(wm.group(1), []) if c[0] == path] \
                or index.get(wm.group(1), [])
            for wpath, wblk in cands:
                if ASSIGNS.search(wblk) or (helper_body(wblk) or ""):
                    wb = helper_body(wblk) or ""
                    wt, ws, _, wht, whs = parse_type_line(wb)
                    if not has_s and (whs or "def.supertypes" in wblk):
                        # An assigning wrapper's own value, or its literal's.
                        got_s = ws or {
                            w for w in WORD.findall(wblk) if w in SUPERTYPES
                        }
                        has_s = True
                    if not has_t and wht:
                        got_t, has_t = wt, True
                    break
            else:
                ok = False
    if base:
        cands = [b for pth, b in index.get(base, []) if pth == path] \
            or [b for _, b in index.get(base, [])]
        if not cands or depth >= 5:
            return got_t, got_s, False
        bt, bs, bok = resolve_type_line(index, path, helper_body(cands[0]) or "", None, depth + 1)
        ok = ok and bok
        if not has_t:
            got_t = bt
        if not has_s:
            got_s = bs
    # Every card has card types; an empty set means the chain was not readable,
    # not that the card has none. Supertypes are genuinely optional, so they
    # cannot carry this test.
    return got_t, got_s, ok and bool(got_t)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=40)
    args = ap.parse_args()

    index = build_helper_index(CATALOG)
    checked = wrong_cost = wrong_pt = missing_flag = wrong_types = 0
    skip = {"nocache": 0, "faces": 0, "split": 0, "star": 0, "nonliteral": 0,
            "noname": 0, "notaspell": 0, "notyped": 0}
    rows = []
    for path in sorted(CATALOG.rglob("*.rs")):
        src = path.read_text()
        starts = [(m.start(), m.group(1)) for m in FN.finditer(src)]
        for i, (pos, fname) in enumerate(starts):
            end = starts[i + 1][0] if i + 1 < len(starts) else len(src)
            raw = src[pos:end]
            body = top_fields(raw)
            if body is None:
                skip["noname"] += 1
                continue
            nm = NAME.search(body) if body is not None else None
            # A helper-built factory (`sorcery("Name", cost(..), ..)`) has no
            # `name:` field, so take a string literal from its head — but check
            # it against the FUNCTION NAME, or the first token a factory defines
            # is read as the card ("Spirit" and "Centaur" both showed up that
            # way, and a 4-line window to dodge them lost 3,900 real factories).
            name = nm.group(1) if nm else None
            if name is None:
                head = "\n".join(raw.split("\n")[:14])
                slug = fname.replace("_", "")
                for cand in ANYNAME.findall(head):
                    if re.sub(r"[^a-z0-9]", "", cand.lower()).startswith(slug[:10]) \
                       or slug.startswith(re.sub(r"[^a-z0-9]", "", cand.lower())[:10]):
                        name = cand
                        break
            if name is None and fname.replace("_", "") in CACHE_SLUG_KEYS:
                name = CACHE_SLUG[fname.replace("_", "")]["name"]
            if name is None:
                skip["noname"] += 1
                continue
            if " // " in name:
                skip["split"] += 1
                continue
            card = CACHE.get(name) or CACHE_LC.get(name.lower())
            if not isinstance(card, dict):
                skip["nocache"] += 1
                continue
            if card.get("card_faces"):
                skip["faces"] += 1
                continue
            # Tokens, Vanguards, schemes and the rest of the non-deck types all
            # print no mana cost and none of them is ever cast.
            tl = card.get("type_line") or ""
            if tl == "Card" or any(w in tl for w in NOT_A_SPELL):
                skip["notaspell"] += 1
                continue
            # CR 202.1b / 601.3e — a card with NO printed mana cost cannot be
            # cast by paying one, and its mana value is 0. The engine enforces
            # that off `no_mana_cost`, so a card missing the flag is either
            # castable for an invented price (a `cost:` that is not printed) or
            # castable for FREE (no `cost:` at all). Lands are excluded: they
            # print no mana cost by definition and the cast path stops them
            # with `CannotCastLand`. This check runs on EVERY factory, literal
            # or helper-built, because the flag is a plain block-level field.
            if (card.get("mana_cost", "") == "" and "no_mana_cost: true" not in raw
                    and "Land" not in (card.get("type_line") or "")):
                missing_flag += 1
                shipped = COST.search(body) if body is not None else None
                shipped_src = shipped.group(1) if shipped else base_struct_cost(raw, name)
                rows.append(("no-cost", name, f"{path.name}::{fname}",
                             (body_cost(shipped_src) or "?") if shipped_src
                             else "{} and castable for free",
                             "no printed mana cost"))
            # A helper-built factory passes the cost positionally:
            # `sorcery("Name", cost(&[generic(2), r()]), effect)`. Read that
            # form too, or 6,312 factories — most of the older sets — are
            # audited by nobody.
            cm = COST.search(body) if body is not None else HELPER.search(raw)
            cost_src = cm.group(1) if cm else base_struct_cost(raw, name)
            if cost_src is None:
                skip["nonliteral"] += 1
                continue
            got = body_cost(cost_src)
            if got is None:
                skip["nonliteral"] += 1
                continue
            checked += 1
            want = card.get("mana_cost", "")
            # A card that prints NO mana cost has nothing to compare against,
            # and the `no_mana_cost` check above is the one that owns it. This
            # also keeps the helper form honest: `blighted("Blighted Steppe",
            # cost(&[generic(3), w()]), ..)` passes the ACTIVATED ability's
            # cost, and the card is a land, so the only sound reading of it is
            # "this card prints no mana cost".
            if want == "":
                pass
            elif norm(got) != norm(want):
                wrong_cost += 1
                rows.append(("cost", name, f"{path.name}::{fname}", got or "{}", want or "{}"))
            if body is None:
                continue  # helper form: only the cost is positional here
            # The printed TYPE LINE. A Sorcery shipped as an Instant is castable
            # at instant speed; a missing Legendary is a legend rule that never
            # fires. Subtypes are left alone — the enums are per-kind and the
            # mapping is a second audit.
            line = (card.get("type_line") or "").split("—")[0]
            want_t = {w for w in line.replace("//", " ").split() if w in CARD_TYPES}
            want_s = {w for w in line.split() if w in SUPERTYPES}
            # ⚠ THE TYPE LINE IS RARELY IN THE LITERAL. Three idioms carry it
            # somewhere else, and a reader that only looks at the literal
            # reports its own blind spot: the BASE-STRUCT form
            # (`..creature("Name", cost, types, 1, 1)`, `..god_weapon(..)`),
            # the WRAPPER form (`legend(CardDefinition { .. })`, six Invasion
            # legends), and a helper whose own base is another helper. All
            # three are followed now; when a link cannot be found the card is
            # skipped rather than reported, because 77 rows of "supertypes:
            # (none)" the first time this column was opened were every one of
            # them `..legend(..)` supplying what the literal never claimed.
            got_t, got_s, resolved = resolve_type_line(index, path, body, raw)
            if not resolved:
                skip["notyped"] += 1
                continue
            # A Vanguard avatar named after a card is not that card, and the
            # oracle lookup cannot tell them apart (Maraxus of Keld is both).
            if "avatar" in (raw[raw.find("\n"):] or "") and "Vanguard" not in (card.get("type_line") or ""):
                if re.search(r"\.\.avatar\s*\(", raw):
                    skip["notaspell"] += 1
                    continue
            if got_t != want_t and want_t:
                wrong_types += 1
                rows.append(("types", name, f"{path.name}::{fname}",
                             " ".join(sorted(got_t)) or "(none)",
                             " ".join(sorted(want_t))))
            if got_s != want_s:
                wrong_types += 1
                rows.append(("super", name, f"{path.name}::{fname}",
                             " ".join(sorted(got_s)) or "(none)",
                             " ".join(sorted(want_s)) or "(none)"))
            op, ot = card.get("power"), card.get("toughness")
            pm, tm = POWER.search(body), TOUGH.search(body)
            if op is None or ot is None or pm is None or tm is None:
                continue
            if not (op.lstrip("-").isdigit() and ot.lstrip("-").isdigit()):
                skip["star"] += 1
                continue
            if (pm.group(1), tm.group(1)) != (op, ot):
                wrong_pt += 1
                rows.append(("p/t", name, f"{path.name}::{fname}",
                             f"{pm.group(1)}/{tm.group(1)}", f"{op}/{ot}"))

    shown = rows if args.rows == 0 else rows[: args.rows]
    for kind, name, where, got, want in shown:
        print(f"  {kind}  {name}\n      {where}\n      body {got}   oracle {want}")
    if len(rows) > len(shown):
        print(f"  ... {len(rows) - len(shown)} more (--rows 0)")
    print(f"# {checked} factories compared against the oracle: "
          f"**{wrong_cost} wrong cost, {wrong_pt} wrong P/T, "
          f"{missing_flag} missing `no_mana_cost`, {wrong_types} wrong type line**")
    print("# skipped — " + ", ".join(f"{k} {v}" for k, v in sorted(skip.items())))
    return 0


if __name__ == "__main__":
    sys.exit(main())
