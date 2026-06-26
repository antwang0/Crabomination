import sys
from pathlib import Path
REPO = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO / "scripts"))
from verify_cards import ScryfallCache, scryfall_get, DEFAULT_CACHE
cache = ScryfallCache(DEFAULT_CACHE)
for name in sys.argv[1:]:
    d = scryfall_get(name, cache)
    if not d:
        print(f"NOT FOUND: {name}"); continue
    print(f"=== {d.get('name')} === {d.get('mana_cost','')} | {d.get('type_line','')} | {d.get('power','')}/{d.get('toughness','')}")
    print(d.get('oracle_text','') or '')
    print()
