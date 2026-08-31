#!/usr/bin/env python3
"""Which `Effect::requires_target` arms read only *some* of their variant's
target-bearing fields.

`audit_target_walkers.py` asks whether a walker recurses into a **wrapper** —
an `Effect` inside an `Effect`. This asks the other half of the same question:
inside one arm, does the walker look at every field that can *hold* a target?

**The bug this was written for.** `Effect::GainControl { what, to, duration }`
read `sel_has_target(what)` and ignored `to`, so "**target opponent** gains
control of this creature" (Witch Engine, Wishclaw Talisman, Risky Move —
`to: Some(PlayerRef::Target(n))`, against `to: None` for "you") answered
"targets nothing". It surfaced only because CR 605.1a's rewrite of
`is_mana_ability` made `!requires_target()` decide whether Witch Engine's
`{T}: Add {B}{B}{B}{B}` is a mana ability, and the wrong answer there is a
mana ability resolving with no target chosen. Nothing else asked.

A field is *target-bearing* when its type mentions `Selector`, `PlayerRef`,
`Value` or `Target` — the four types that can carry a `Target(n)` slot. The
arm "reads" a field when the arm's text names it. Both halves are syntactic,
which is the point: this is a report of what the walker looks at, not a proof
about what it concludes.

**The unread list on its own is 182 rows and would be useless**, because a
field's *type* can carry a slot without any card ever putting one there: a
`Value` that is always a `Const` count, a `PlayerRef` the engine writes as
`You`. So the report is a **join against the catalog** — the same device
`audit_oracle_verbs` uses — and only names a field some shipped card actually
aims. That is **21 rows**, and the six read at the hundred-and-fifteenth pass
were all real: Autumn Willow's whole ability is
`WaiveShroudForPlayerThisTurn { player: Target(0) }` sitting in a
`| … => false` group.

    python3 scripts/audit_target_fields.py           # the aimed rows
    python3 scripts/audit_target_fields.py --all     # + the unaimed ones
    python3 scripts/audit_target_fields.py --check   # exit 1 on an aimed gap

**Read the join before reading a row as a bug.** An unread field is only a
wrong *answer* when nothing else in the card's effect tree names the same
slot: `Effect::Seq([Tap { what: target_filtered(..) }, SacrificeAtNextEndStep
{ what: Target(0) }])` is covered by its sibling. The row says the walker did
not look, not that it concluded wrongly.

`ALLOWED` is the reviewed set: a (variant, field) a reviewer has established
cannot carry a live slot. Everything else printed is a question nobody has
answered.
"""

import re
import sys
from pathlib import Path

BASE = Path(__file__).resolve().parent.parent
EFFECT = BASE / "crabomination_base" / "src" / "effect.rs"
QUERY = BASE / "crabomination_base" / "src" / "effect" / "query.rs"

TARGETY = re.compile(r"\b(Selector|PlayerRef|Value|Target)\b")

# (variant, field) pairs a reviewer has signed off. Keep the reason on the row.
ALLOWED: dict[tuple[str, str], str] = {}


def enum_body(text: str, name: str) -> str:
    i = text.index(f"pub enum {name} {{")
    j = text.index("{", i)
    depth, k = 1, j + 1
    while depth:
        c = text[k]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
        k += 1
    lines = [
        l
        for l in text[j + 1 : k - 1].split("\n")
        if not l.lstrip().startswith("//") and not l.lstrip().startswith("#[")
    ]
    return "\n".join(lines)


def variants(body: str) -> dict[str, list[tuple[str, str]]]:
    """`{variant: [(field, type)]}` for the *named*-field variants.

    Tuple variants are skipped: their fields have no names for an arm to bind,
    so "does the arm read this field" is not a question this instrument can
    ask about them.
    """
    out: dict[str, list[tuple[str, str]]] = {}
    marks = [(m.start(), m.group(1)) for m in re.finditer(r"(?m)^    ([A-Z]\w*)", body)]
    for idx, (pos, name) in enumerate(marks):
        end = marks[idx + 1][0] if idx + 1 < len(marks) else len(body)
        seg = body[pos:end]
        brace = seg.find("{")
        if brace < 0 or (0 <= seg.find("(") < brace):
            out[name] = []
            continue
        depth, k = 1, brace + 1
        while depth and k < len(seg):
            if seg[k] == "{":
                depth += 1
            elif seg[k] == "}":
                depth -= 1
            k += 1
        fields = re.findall(r"(\w+)\s*:\s*([^,\n]+)", seg[brace + 1 : k - 1])
        out[name] = [(f, ty.strip()) for f, ty in fields]
    return out


def arms(text: str, fn: str) -> dict[str, str]:
    """`{variant: the arm's source text}` for one walker's `match self` block.

    Split at depth 0 of the match block, not by regex: an arm's *pattern* is
    full of braces (`Effect::GainControl { what, .. }`) and the obvious
    "look back to the last `{`" heuristic lands inside the destructuring and
    loses the variant name — which is how the first version of this script
    reported zero gaps on a tree that had one.

    A shared arm (`A | B | C => expr`) gives every variant on its left the
    same text, which is right: it reads the same fields for all of them.
    """
    i = text.index(f"pub fn {fn}(")
    j = text.index("match self {", i)
    j = text.index("{", j + 5)
    depth, k = 1, j + 1
    while depth:
        if text[k] == "{":
            depth += 1
        elif text[k] == "}":
            depth -= 1
        k += 1
    body = text[j + 1 : k - 1]

    out: dict[str, str] = {}
    depth = 0
    start = 0
    fat = -1
    i = 0
    while i < len(body):
        c = body[i]
        if c in "{([":
            depth += 1
        elif c in "})]":
            depth -= 1
        elif depth == 0 and c == "=" and body[i : i + 2] == "=>":
            if fat < 0:
                fat = i
            i += 1
        elif depth == 0 and c == "," and fat >= 0:
            pat, arm = body[start:fat], body[fat + 2 : i]
            for n in re.findall(r"\bEffect::(\w+)", pat):
                out[n] = out.get(n, "") + " " + pat + " " + arm
            start, fat = i + 1, -1
        i += 1
    if fat >= 0:
        pat, arm = body[start:fat], body[fat + 2 :]
        for n in re.findall(r"\bEffect::(\w+)", pat):
            out[n] = out.get(n, "") + " " + pat + " " + arm
    return out



SETS = BASE / "crabomination_catalog" / "src"
SHORTCUT = BASE / "crabomination_base" / "src" / "effect"


def catalog_target_fields() -> set[tuple[str, str]]:
    """`{(variant, field)}` a **shipped card** actually puts a target into.

    The unread-field list on its own is 182 rows and nearly all of them are a
    `Value` count or a `PlayerRef::You` that no card aims: a field's *type*
    can carry a `Target(n)` without any card ever putting one there, and a
    walker that never reads it is then correct by accident rather than wrong.
    This is the join that separates the two — the same device
    `audit_oracle_verbs` uses, a code question answered against the corpus
    rather than against the code's own shape.
    """
    out: set[tuple[str, str]] = set()
    files = list(SETS.rglob("*.rs")) + list(SHORTCUT.rglob("*.rs"))
    for f in files:
        text = f.read_text()
        for m in re.finditer(r"\bEffect::(\w+)\s*\{", text):
            v = m.start()
            i = text.index("{", m.start())
            depth, k = 1, i + 1
            while depth and k < len(text):
                if text[k] in "{([":
                    depth += 1
                elif text[k] in "})]":
                    depth -= 1
                k += 1
            inner = text[i + 1 : k - 1]
            # split the literal's fields at depth 0
            depth, start, name = 0, 0, None
            for idx, c in enumerate(inner):
                if c in "{([":
                    depth += 1
                elif c in "})]":
                    depth -= 1
                elif depth == 0 and c == ":" and name is None:
                    name = inner[start:idx].strip().split()[-1] if inner[start:idx].strip() else None
                    val_start = idx + 1
                elif depth == 0 and c == ",":
                    if name and re.search(r"\bTarget\b", inner[val_start:idx]):
                        out.add((m.group(1), name))
                    start, name = idx + 1, None
            if name and re.search(r"\bTarget\b", inner[val_start:]):
                out.add((m.group(1), name))
    return out


def main() -> int:
    show_all = "--all" in sys.argv
    check = "--check" in sys.argv
    vs = variants(enum_body(EFFECT.read_text(), "Effect"))
    ar = arms(QUERY.read_text(), "requires_target")
    aimed = catalog_target_fields()
    gaps: list[tuple[str, str, str]] = []
    named = 0
    for v, fields in sorted(vs.items()):
        ty_fields = [(f, ty) for f, ty in fields if TARGETY.search(ty)]
        if not ty_fields:
            continue
        named += 1
        arm = ar.get(v)
        if arm is None:
            continue
        unread = [
            (f, ty) for f, ty in ty_fields if not re.search(r"\b" + f + r"\b", arm)
        ]
        if show_all:
            print(f"{v}: {len(ty_fields) - len(unread)}/{len(ty_fields)} read")
        for f, ty in unread:
            if (v, f) in ALLOWED:
                continue
            gaps.append((v, f, ty, (v, f) in aimed))
    live = [g for g in gaps if g[3]]
    print(
        f"# {named} variants with a target-bearing field; {len(gaps)} unread, "
        f"**{len(live)} of them aimed by a shipped card**"
    )
    for v, f, ty, _ in live:
        print(f"  AIMED  Effect::{v}  .{f}: {ty}")
    if show_all:
        for v, f, ty, a in gaps:
            if not a:
                print(f"  unread Effect::{v}  .{f}: {ty}")
    return 1 if (check and live) else 0


if __name__ == "__main__":
    sys.exit(main())
