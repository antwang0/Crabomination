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
COST = re.compile(r"^cost: cost\(&\[([^\]]*)\]\),", re.M)
POWER = re.compile(r"^power: (-?\d+),", re.M)
TOUGH = re.compile(r"^toughness: (-?\d+),", re.M)


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
    return "".join(sorted(re.findall(r"\{[^}]*\}", cost.upper())))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--rows", type=int, default=40)
    args = ap.parse_args()

    checked = wrong_cost = wrong_pt = missing_flag = 0
    skip = {"nocache": 0, "faces": 0, "split": 0, "star": 0, "nonliteral": 0,
            "noname": 0, "notaspell": 0}
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
            # `name:` field, so take the first string literal — but only from
            # the first few lines, or the first TOKEN a factory defines is read
            # as the card ("Spirit" and "Centaur" both showed up that way).
            head = "\n".join(raw.split("\n")[:4])
            name = nm.group(1) if nm else (ANYNAME.search(head).group(1)
                                           if ANYNAME.search(head) else None)
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
                rows.append(("no-cost", name, f"{path.name}::{fname}",
                             (body_cost(shipped.group(1)) or "?") if shipped
                             else "{} and castable for free",
                             "no printed mana cost"))
            if body is None:
                skip["nonliteral"] += 1
                continue
            cm = COST.search(body)
            if not cm:
                skip["nonliteral"] += 1
                continue
            got = body_cost(cm.group(1))
            if got is None:
                skip["nonliteral"] += 1
                continue
            checked += 1
            want = card.get("mana_cost", "")
            if norm(got) != norm(want):
                wrong_cost += 1
                rows.append(("cost", name, f"{path.name}::{fname}", got or "{}", want or "{}"))
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
          f"{missing_flag} missing `no_mana_cost`**")
    print("# skipped — " + ", ".join(f"{k} {v}" for k, v in sorted(skip.items())))
    return 0


if __name__ == "__main__":
    sys.exit(main())
