#!/usr/bin/env python3
"""Factories whose own `keywords:` field silently drops keywords a helper set.

    python3 scripts/audit_helper_keywords.py

A card file's private helper (`fn companion(mut def) { def.keywords.push(..) }`,
`fn ninja(..) -> CardDefinition { CardDefinition { keywords: vec![..], .. } }`)
adds keywords to the definition it builds; a factory that then writes
`CardDefinition { keywords: vec![..], ..helper(..) }` replaces the whole list,
so the helper's keywords vanish (Donna Noble and Amy Pond lost Doctor's
companion and could not be seated with their Doctor). Prints
`file:line factory helper [helper keywords]` for each hit; exit 1 if any.
"""
import glob
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def helper_keywords(text):
    """Private helpers that set keywords themselves: name -> keyword text."""
    out = {}
    for m in re.finditer(r"\nfn (\w+)\(([^)]*)\)[^{]*\{(.*?)\n\}", text, re.S):
        name, params, body = m.group(1), m.group(2), m.group(3)
        kws = re.findall(r"\.keywords\.push\(([^;]*)\);", body)
        lit = re.search(r"\bkeywords: (vec!\[[^\]]+\])", body)
        if lit and "keywords" not in params:
            kws.append(lit.group(1))
        if kws:
            out[name] = kws
    return out


def keyword_names(kws):
    """`Keyword::Crew(crew)` -> `Keyword::Crew`, one per keyword the helper sets."""
    return [k for text in kws for k in re.findall(r"Keyword::\w+", text)]


def printed_keywords(body, cache):
    """Scryfall's keyword list for the first card name in the factory body,
    normalized (`Doctor's companion` -> `doctorscompanion`); None if unknown."""
    for lit in re.findall(r'"((?:[^"\\]|\\.)*)"', body):
        card = cache.get(lit)
        if card is not None:
            return {re.sub(r"[^a-z]", "", k.lower()) for k in card.get("keywords", [])}
    return None


def main():
    import json
    cache = json.load(open(os.path.join(ROOT, "scripts", ".scryfall_cache.json"), encoding="utf-8"))
    hits = 0
    for p in sorted(glob.glob(os.path.join(ROOT, "crabomination_catalog", "src", "**", "*.rs"), recursive=True)):
        text = open(p, encoding="utf-8").read()
        helpers = helper_keywords(text)
        if not helpers:
            continue
        for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{(.*?)\n\}", text, re.S):
            body = m.group(2)
            if not re.search(r"^        keywords:", body, re.M):
                continue
            own = re.search(r"^        keywords: (vec!\[.*?\]),$", body, re.M | re.S)
            own = own.group(1) if own else ""
            for h in [x for x in re.findall(r"\.\.(?:legendary\()?(\w+)\(", body) if x in helpers]:
                # The card's name is the first literal of the helper call
                # (`..companion(creature("Amy Pond", ..))`), not of the body
                # (`PartnerWith("Rory Williams")` comes first).
                printed = printed_keywords(body[body.index(".." ):], cache)
                dropped = [
                    k for k in keyword_names(helpers[h])
                    if k not in own and (printed is None or k.split("::")[1].lower() in printed)
                ]
                if h in helpers and dropped:
                    line = text.count("\n", 0, m.start()) + 1
                    print(f"{os.path.relpath(p, ROOT)}:{line} {m.group(1)} {h} {helpers[h]}")
                    hits += 1
    print(f"{hits} factories drop a helper's keywords", file=sys.stderr)
    sys.exit(1 if hits else 0)


if __name__ == "__main__":
    main()
