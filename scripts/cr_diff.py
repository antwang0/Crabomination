#!/usr/bin/env python3
"""Diff two Comprehensive Rules editions rule by rule, and name the Rust sites
that cite each rule whose text moved.

A new edition renumbers silently: `audit_cr_citations.py` only sees numbers that
no longer exist, not numbers that now mean something else (2026-09-25: 704.5w
went from "a battle with no protector" to "a non-Siege battle at 0 defense",
701.50c from connive LKI to APNAP order). This lists CHANGED / ADDED / REMOVED
rules and, for every CHANGED or REMOVED one, each `CR <number>` citation in the
tree — read those against the new text.

    curl -sSL "https://media.wizards.com/2026/downloads/MagicCompRules%20YYYYMMDD.txt" -o new.txt
    git show HEAD:crabomination/MagicCompRules_<old>.txt > old.txt
    python3 scripts/cr_diff.py old.txt new.txt [--cites-only]
"""

import os
import re
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
RULE = re.compile(r"^(\d{3}\.\d+[a-z]*)\.? (.*)")


def load(path):
    rules = {}
    for line in open(path, encoding="utf-8-sig"):
        m = RULE.match(line.rstrip("\n").replace("\r", ""))
        if m:
            rules.setdefault(m.group(1), m.group(2).strip())
    return rules


def cites(rule):
    pat = r"CR " + re.escape(rule) + r"([^0-9a-z]|$)"
    out = subprocess.run(
        ["git", "grep", "-nE", pat, "--", "*.rs"], cwd=ROOT, capture_output=True, text=True
    ).stdout
    return [l for l in out.splitlines() if l]


def key(rule):
    sec, sub = rule.split(".", 1)
    num = re.match(r"\d+", sub).group(0)
    return (int(sec), int(num), sub[len(num):])


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    if len(args) != 2:
        sys.exit(__doc__)
    old, new = load(args[0]), load(args[1])
    changed = sorted((r for r in new if r in old and old[r] != new[r]), key=key)
    added = sorted((r for r in new if r not in old), key=key)
    removed = sorted((r for r in old if r not in new), key=key)
    print(f"# {len(changed)} changed, {len(added)} added, {len(removed)} removed")
    cites_only = "--cites-only" in sys.argv
    for kind, rules in (("CHANGED", changed), ("REMOVED", removed)):
        for r in rules:
            sites = cites(r)
            if cites_only and not sites:
                continue
            print(f"\n## {kind} {r} ({len(sites)} citations)")
            print(f"- {old[r][:200]}")
            if r in new:
                print(f"+ {new[r][:200]}")
            for s in sites[:8]:
                print(f"    {s[:160]}")
    if not cites_only:
        for r in added:
            print(f"\n## ADDED {r}\n+ {new[r][:200]}")


main()
