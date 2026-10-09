#!/usr/bin/env python3
"""Type unions a card's body dropped (a bug-class scanner, like
`audit_effect_oracle.py`): Oracle names "creature or Vehicle" and the body
never says Vehicle, or "player or planeswalker" and the body never says
Planeswalker (nor `target_any`).

2026-10-09 it found Fire Nation Engineer's raid counter, Gas Guzzler's
sacrifice and Agonasaur Rex's cycle counters (creature-only), and Lava Spike,
Skullcrack, Viashino Pyromancer and Telim'Tor's Darts (player-only). A row is
a lead: read the card first.

    python3 scripts/audit_type_unions.py
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import audit_effect_oracle as a  # noqa: E402

# (Oracle phrase, a body word that covers it, body tokens that also cover it)
CHECKS = [
    (r"creatures? (?:and/)?or Vehicles?|Vehicles? (?:and/)?or creatures?", "vehicle", ()),
    (r"target player or planeswalker", "planeswalker", ("target_any", "any_target")),
]


def main():
    low = a.oracle_index()
    rows = []
    for f in sorted((a.ROOT / "crabomination_catalog" / "src").rglob("*.rs")):
        t = f.read_text(encoding="utf-8")
        for m in re.finditer(r"\npub fn ([a-z0-9_]+)\(\) -> CardDefinition", t):
            fn = m.group(1)
            code = re.sub(r"//[^\n]*", "", a.own_body(t, m.start()))
            names = [n for n in re.findall(r'name: "([^"]+)"', code) if a.slug(n) == fn]
            if not names:
                continue
            o = low.get(names[0].lower())
            if not o:
                continue
            for phrase, word, helpers in CHECKS:
                if not re.search(phrase, o, re.I) or word in code.lower() or any(h in code for h in helpers):
                    continue
                hit = re.search(rf"[^.\n]*(?:{phrase})[^.\n]*", o, re.I).group(0)
                rows.append((fn, str(f.relative_to(a.ROOT)), hit.strip()[:140]))
    for r in rows:
        print(" | ".join(r))
    print(f"# {len(rows)} rows")


if __name__ == "__main__":
    main()
