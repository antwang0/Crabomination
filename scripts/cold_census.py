#!/usr/bin/env python3
"""Which hot functions read a `CardCold` field, and which of them have taken
`cold_pristine()` yet.

    python3 scripts/cold_census.py cg.out [rows]
    python3 scripts/cold_census.py cg.out --field named_card

PERF `(-349)`..`(-351)` put one byte on `CardData` — "has this object's cold
group ever been written" — and a reader of a cold field can open on it instead
of chasing the group's pointer and loading a length. Three families took it
(the cleanup guards, the keyword asks, the layer pass) and this is the join
that says who else should: **140 functions read a cold field and only 54 have
a self row in a six-game `cube` dump**, so a site in a body the profile never
enters is not a perf question.

The join was hand-rolled twice before it was a script — it is what found
`compute_permanent_pass`, the largest engine row in the program.

Reads the field list out of `CardCold` itself, so a field added to the group
appears here with no edit. Columns:

    self Ir   the enclosing function's self cost in the dump (0 = never ran)
    sites     how many cold-field reads the function's body contains
    gate      `yes` if the body already mentions `cold_pristine`
    fields    which members it reads

⚠ **The attribution is by enclosing `fn`, so it over-counts**: a read inside a
per-card static walk gets the whole walk's row (PERF's note on
`gather_continuous_effects_inner`). Read the site before believing a row.

⚠ A write (`card.field = ..`, `.push(..)`, `&mut`) is not a gate question —
this filters the obvious write forms out, but the `sites` column is a
starting point for a grep, not a count to quote.
"""
import collections
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import cg_edges  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CARD_RS = os.path.join(ROOT, "crabomination_base", "src", "card.rs")
CRATES = [
    os.path.join(ROOT, "crabomination", "src"),
    os.path.join(ROOT, "crabomination_base", "src"),
]

# `.field` followed by anything that makes it a write, not a read.
WRITE_TAIL = re.compile(r"^\s*(=[^=]|\+=|-=|\.push|\.clear\(|\.retain|\.insert|\.remove|\.sort|\.extend|\.pop|\.drain|\.truncate|\.take\(|\.replace\(|\.get_or_insert)")
FN_DECL = re.compile(r"^(\s*)(?:pub(?:\([^)]*\))?\s+)?(?:const\s+|async\s+|unsafe\s+|extern\s+\"[^\"]*\"\s+)*fn\s+([A-Za-z_][A-Za-z0-9_]*)")


def cold_fields():
    """The `CardCold` member names, read off the struct definition."""
    with open(CARD_RS, encoding="utf-8") as fh:
        lines = fh.read().splitlines()
    out = []
    inside = False
    for line in lines:
        if line.startswith("pub struct CardCold {"):
            inside = True
            continue
        if inside:
            if line.startswith("}"):
                break
            m = re.match(r"\s+pub ([a-z_][a-z0-9_]*):", line)
            if m:
                out.append(m.group(1))
    return out


def scan(fields):
    """-> {fn_name: (sites, {fields}, has_gate)} over the engine crates."""
    pat = re.compile(r"\.(" + "|".join(sorted(fields, key=len, reverse=True)) + r")\b")
    hits = collections.defaultdict(lambda: [0, set()])
    gated = set()
    for crate in CRATES:
        for dirpath, _dirs, names in os.walk(crate):
            for name in names:
                if not name.endswith(".rs"):
                    continue
                path = os.path.join(dirpath, name)
                with open(path, encoding="utf-8") as fh:
                    lines = fh.read().splitlines()
                # Enclosing `fn` per line: the most recent declaration whose
                # indent is shallower than the line's own body indent.
                stack = []  # (indent, name)
                current = []
                for line in lines:
                    m = FN_DECL.match(line)
                    if m:
                        indent = len(m.group(1))
                        while stack and stack[-1][0] >= indent:
                            stack.pop()
                        stack.append((indent, m.group(2)))
                    current.append(stack[-1][1] if stack else "<none>")
                for line, fn in zip(lines, current):
                    if "cold_pristine" in line:
                        gated.add(fn)
                    for m in pat.finditer(line):
                        if WRITE_TAIL.match(line[m.end():]):
                            continue
                        hits[fn][0] += 1
                        hits[fn][1].add(m.group(1))
    return {fn: (n, fs, fn in gated) for fn, (n, fs) in hits.items()}


def self_by_short_name(path):
    """Fold the dump's self cost onto bare function names."""
    self_cost, _ec, _eca, total, _d = cg_edges.parse(path)
    folded = collections.Counter()
    for sym, cost in self_cost.items():
        # `crabomination::game::stack::GameState::foo::{{closure}}` -> `foo`
        base = sym.split("::{{")[0]
        short = base.split("::")[-1]
        if short.startswith("h") and len(short) == 17:
            short = base.split("::")[-2] if "::" in base else short
        folded[short] += cost
    return folded, total


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    path = sys.argv[1]
    rows = 40
    only = None
    rest = sys.argv[2:]
    if "--field" in rest:
        only = rest[rest.index("--field") + 1]
    elif rest:
        rows = int(rest[0])

    fields = cold_fields()
    if only:
        if only not in fields:
            sys.exit(f"{only!r} is not a CardCold member; members: {' '.join(fields)}")
        fields = [only]
    hits = scan(fields)
    folded, total = self_by_short_name(path)

    ranked = sorted(hits.items(), key=lambda kv: -folded.get(kv[0], 0))
    ran = [r for r in ranked if folded.get(r[0], 0)]
    print(f"# {len(fields)} CardCold members, {len(hits)} functions read one, "
          f"{len(ran)} have a self row; dump total Ir {total:,}")
    print(f"# {sum(1 for _fn, (_n, _f, g) in ran if g)} of the {len(ran)} that run are already gated")
    print(f"{'self Ir':>14} {'%':>6} {'sites':>6}  {'gate':<5} name / fields")
    for fn, (n, fs, gate) in ranked[:rows]:
        s = folded.get(fn, 0)
        pct = 100.0 * s / total if total else 0.0
        print(f"{s:>14,} {pct:>6.2f} {n:>6}  {'yes' if gate else '-':<5} "
              f"{fn}  [{' '.join(sorted(fs))}]")
    if len(ranked) > rows:
        print(f"# ... {len(ranked) - rows:,} more (self Ir 0 unless stated)")


if __name__ == "__main__":
    main()
