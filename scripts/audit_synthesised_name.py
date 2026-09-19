#!/usr/bin/env python3
"""Factories whose doc says "synthesised" under a name Scryfall owns.

CARD_BACKLOG has known this shape since the fifty-fourth pass — "a synthesised
card wearing a printed card's name" — and explains why it survives review: the
*characteristics* get corrected against Scryfall at some point (so
`audit_catalog_stats` and `audit_printed_body` both read zero on the card) and
the ability never does, because no column compared abilities. The three
INVENTED columns (`audit_invented_may`, `audit_invented_trigger`,
`audit_invented_rider`) catch the ones whose body shape gives them away. This
is the rest of the population, by the one signal that names all of them: the
factory's own doc comment.

    python3 scripts/audit_synthesised_name.py           # the list
    python3 scripts/audit_synthesised_name.py --count   # totals only
    python3 scripts/audit_synthesised_name.py --files   # by file

⚠ **This is a reading list, not a proof, and it reads 0 the day the docs are
rewritten.** Of the 27 rows at its first run, **eight** were wrong cards and
**nineteen** were correct bodies under a doc describing the card they used to
be — `audit_doc_drift`'s 338-stale-comments population arriving by a
different route. Work a row by reading the oracle against the **body**, never
against the doc.

⚠ And a clean row is not a clean card: a factory with no such comment can
still be wrong. The comment is a tell, not a gate — the three body-shaped
columns above are what prove anything.

**Three reader traps, all general to any doc-keyed audit, all measured here:**
the card's own NAME can contain the tell (Ichor Synthesizer); a **repair
note** is not a claim and is per-DOC rather than per-match ("it shipped as a
synthesised X … the catalog's own rule is that a synthesised card carries a
name the oracle does not" has two matches and one repair word); and a `///`
comment **wraps**, so any multi-word phrase has to be matched against a
flattened doc or the line break defeats it.
"""

import json
import os
import re
import sys
import unicodedata
from collections import Counter

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

# ⚠ Two false-positive classes, both measured on the first run (56 -> the
# number below). ① The card's own NAME can contain the word — Experimental
# Synthesizer, Ichor Synthesizer, Dyadrine, Synthesis Amalgam — so the name is
# stripped from the doc before matching, and the tell is the participle
# (`synthesised`), never the noun. ② A doc that says the card *used to* be
# synthesised is a repair note, not a claim; `REPAIRED` is the look-behind.
TELL = re.compile(r"synthesi[sz]ed|\binvented\b|made-up|fabricated", re.I)
REPAIRED = re.compile(
    r"(shipped|was|were|used to|no longer|had been|stopped|replaces|instead of)\b[^.]{0,60}$", re.I
)
# ⚠ The look-behind is per-match and a repair note is per-DOC: "it shipped as
# a synthesised X … the catalog's own rule is that a synthesised card carries
# a name the oracle does not" has two matches and only the first is preceded
# by a repair word. A doc carrying any of these anywhere is a repair note.
REPAIRED_DOC = re.compile(
    r"it shipped|shipped until|shipped as|used to (be|read|call)|it replaced|was wrong|"
    r"the previous|no longer|⚠",
    re.I,
)


def slug(name):
    folded = unicodedata.normalize("NFKD", name)
    folded = "".join(c for c in folded if not unicodedata.combining(c))
    return re.sub(r"[^a-z0-9]+", "_", folded.lower().replace("'", "")).strip("_")


def factories(path):
    src = open(path, encoding="utf-8").read()
    for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
        doc = []
        for line in src[: m.start()].rstrip().split("\n")[::-1]:
            if line.lstrip().startswith("//"):
                doc.append(line)
            else:
                break
        start, depth, i = m.end() - 1, 0, m.end() - 1
        while i < len(src):
            if src[i] == "{":
                depth += 1
            elif src[i] == "}":
                depth -= 1
                if depth == 0:
                    break
            i += 1
        body = src[start : i + 1]
        fn = m.group(1)
        own = next(
            (n for n in re.findall(r'"((?:[^"\\]|\\.)*)"', body) if slug(n) == fn), None
        )
        if own:
            yield fn, own, "\n".join(doc[::-1]), body


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    rows, by_file = [], Counter()
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            path = os.path.join(dirpath, f)
            for fn, name, doc, _body in factories(path):
                # ① the card's own name is not evidence about the card, and
                # neither is an inline code span — this script's own file name
                # contains the tell, so every repair note that cites it read
                # as a fresh claim until backticked spans were stripped.
                # ⚠ And the doc is normalised first: a `///` comment wraps,
                # so "the doc used\n/// to call it synthesised" defeats any
                # multi-word phrase the moment the line breaks inside it.
                flat = re.sub(r"^\s*//[/!]?", " ", doc, flags=re.M)
                clean = re.sub(r"\s+", " ", re.sub(r"`[^`]*`", " ", flat)).replace(name, " ")
                if REPAIRED_DOC.search(clean):
                    continue
                hit = next(
                    (
                        m
                        for m in TELL.finditer(clean)
                        # ② a repair note reads "it shipped as a synthesised …".
                        if not REPAIRED.search(clean[max(0, m.start() - 70) : m.start()])
                    ),
                    None,
                )
                if hit is None:
                    continue
                if not isinstance(cache.get(name), dict):
                    continue  # a made-up name over a made-up card is fine
                rel = os.path.relpath(path, ROOT)
                rows.append((rel, fn, name))
                by_file[rel] += 1

    rows.sort()
    if "--files" in sys.argv:
        for rel, n in by_file.most_common():
            print(f"{n:5d}  {rel}")
    elif "--count" not in sys.argv:
        for rel, fn, name in rows:
            print(f"{rel}::{fn}\n    {name}")
    print(
        f"# {len(rows)} factories whose doc says synthesised under a name Scryfall "
        f"owns, over {len(by_file)} files"
    )
    return 1 if ("--check" in sys.argv and rows) else 0


if __name__ == "__main__":
    sys.exit(main())
