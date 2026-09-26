#!/usr/bin/env python3
"""Cards whose oracle says "target opponent" but whose body hits each opponent.

The reverse of `audit_each_opponent.py`: a single-opponent clause coded as a
fan-out is invisible in two-player games and wrong in every pod. Lists the
cards whose printed text names a target opponent while the body reaches
`EachOpponent` and never an opponent target (`OpponentPlayer`, a player
`Target(..)` slot).
"""
import json, re, pathlib, sys
sys.path.insert(0, 'scripts')
root = pathlib.Path('crabomination_catalog/src')
fn_re = re.compile(r'^pub fn (\w+)\(\) -> CardDefinition', re.M)
name_re = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')

def body_of(src, start):
    i = src.index("{", start); depth = 0
    while i < len(src):
        depth += {"{": 1, "}": -1}.get(src[i], 0)
        if depth == 0:
            return src[start:i + 1]
        i += 1
    return src[start:]

sc = json.load(open('scripts/.scryfall_cache.json'))
def oracle(e):
    faces = e.get('card_faces') or []
    return ' '.join([e.get('oracle_text') or ''] + [f.get('oracle_text') or '' for f in faces])
PAREN = re.compile(r'\([^)]*\)')
TARGET = re.compile(r'OpponentPlayer|PlayerRef::Target\(|Selector::Target\(|R::Player\b|SelectionRequirement::Player\b|TargetOpponent')
rows = []
seen = set()
for p in root.rglob('*.rs'):
    s = p.read_text()
    for mm in fn_re.finditer(s):
        body = body_of(s, mm.start())
        m = name_re.search(body)
        if not m:
            continue
        nm = m.group(1)
        e = sc.get(nm) or next((v for k, v in sc.items() if k.split(' // ')[0] == nm), None)
        if not isinstance(e, dict) or nm in seen:
            continue
        seen.add(nm)
        text = PAREN.sub('', oracle(e))
        if 'target opponent' not in text.lower() or 'each opponent' in text.lower():
            continue
        if 'EachOpponent' in body and not TARGET.search(body):
            rows.append((nm, mm.group(1), str(p).split('sets/')[-1]))
rows.sort()
print(f"{len(rows)} cards print \"target opponent\" and fan out to each opponent instead\n")
for nm, ident, f in rows:
    print(f"- {nm}  [{f}::{ident}]")
