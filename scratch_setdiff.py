import sys, re, json, urllib.request, urllib.parse, time
from pathlib import Path

REPO = Path("/home/user/Crabomination")
CAT = REPO/"crabomination_catalog"/"src"

def catalog_names():
    names = set()
    for f in CAT.rglob("*.rs"):
        for m in re.finditer(r'"((?:[^"\\]|\\.)*)"', f.read_text(encoding="utf-8", errors="replace")):
            names.add(m.group(1))
    return names

def fetch_set(setcode):
    cards=[]
    url=f"https://api.scryfall.com/cards/search?q="+urllib.parse.quote(f"set:{setcode} -is:reprint")
    while url:
        req=urllib.request.Request(url, headers={"User-Agent":"crabomination/1.0","Accept":"application/json"})
        data=json.load(urllib.request.urlopen(req, timeout=20))
        for c in data["data"]:
            if c.get("layout") in ("token","art_series","emblem"): continue
            cards.append(c["name"])
        url=data.get("next_page")
        time.sleep(0.1)
    return cards

cat=catalog_names()
def norm(n): return n.split(" //")[0]
for setcode in sys.argv[1:]:
    cards=fetch_set(setcode)
    missing=[c for c in cards if norm(c) not in cat and c not in cat]
    print(f"=== {setcode}: {len(cards)} cards, {len(missing)} missing ===")
    for c in missing: print("  ", c)
