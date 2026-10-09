#!/usr/bin/env python3
"""Cards whose Oracle names "creature or Vehicle" but whose body never says
Vehicle (a bug-class scanner, like `audit_effect_oracle.py`).

2026-10-09 it found Fire Nation Engineer's raid counter, Gas Guzzler's
sacrifice and Agonasaur Rex's cycle counters, all creature-only. A row is a
lead: read the card first.

    python3 scripts/audit_vehicle_targets.py
"""
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import audit_effect_oracle as a  # noqa: E402

PHRASE = r"creatures? (?:and/)?or Vehicles?|Vehicles? (?:and/)?or creatures?"


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
            if not o or not re.search(PHRASE, o, re.I) or "vehicle" in code.lower():
                continue
            hit = re.search(rf"[^.\n]*(?:{PHRASE})[^.\n]*", o, re.I).group(0)
            rows.append((fn, str(f.relative_to(a.ROOT)), hit.strip()[:140]))
    for r in rows:
        print(" | ".join(r))
    print(f"# {len(rows)} rows")


if __name__ == "__main__":
    main()
