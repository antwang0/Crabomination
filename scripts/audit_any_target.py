#!/usr/bin/env python3
"""Damage / prevention slots that bound NOTHING (CR 115.4).

"Any target" is a creature, player, planeswalker or battle
(`SelectionRequirement::any_target()`, `shortcut::target_any()`). A slot
written `TargetFiltered { filter: Any }` or a bare `Target(n)` on a spell,
activated or loyalty ability accepts a land or an artifact too (Lightning
Bolt took a Forest). Triggered abilities are skipped: their bare slot is
often a pre-bound "that creature". `--gate` exits 1 on any row.

    target/debug/dump_cards --effects > /tmp/ef.tsv
    python3 scripts/audit_any_target.py /tmp/ef.tsv [--gate]
"""

import re
import sys

KINDS = r"(?:DealDamage|PreventNextDamage|PreventNextDamageAndGainLife|ExileIfWouldDieThisTurn)"
# Risk Factor's "may have it deal 4 damage to them" is the opponent's pick;
# the synthesised STX cards' definitions ARE their spec (no printing).
ALLOW = {"Risk Factor"}
SYNTH = re.compile(r"\((?:b\d+|Batch \d+)\)$")
BAD = re.compile(KINDS + r" \{ (?:to|target|what): (?:Target\((\d)\)|TargetFiltered \{ slot: (\d), filter: Any \})")


def main():
    out = []
    for line in open(sys.argv[1], encoding="utf-8"):
        parts = line.rstrip("\n").split("\t", 2)
        if len(parts) != 3:
            continue
        name, where, dbg = parts
        if where.startswith("triggered") or "ApplyToTargets" in dbg or name in ALLOW or SYNTH.search(name):
            continue
        for m in BAD.finditer(dbg):
            slot = m.group(1) or m.group(2)
            # A slot another effect of the ability already bounds is fine
            # ("… to target creature. If it would die, exile it").
            bounded = re.search(r"TargetFiltered \{ slot: %s, filter: (?!Any \})" % slot, dbg)
            if not bounded:
                out.append(f"{name}\t{where}")
    print("\n".join(sorted(set(out))))
    print(f"# {len(set(out))} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
