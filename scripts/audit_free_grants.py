#!/usr/bin/env python3
"""Cards whose may-play grant is FREE while the oracle says the cast is paid.

`GrantMayPlay { pay_own_cost: false }` and `ExileTopAndGrantMayPlay { pay_any_color:
false, pay_own_cost: false }` stamp no cost, and the permission cast
(`CastFromZoneWithoutPaying`) then costs nothing. Lists every factory with such a
grant whose oracle has no "without paying" (or MV-capped free) clause, and —
marked `ANY` — every grant paying its mana value in any color whose oracle
never says "any type" / "any color" (Chandra, Torch of Defiance's +1 was one). Seven pod
cards had it (Whispersteel Dagger, Heartless Conscription, Yasmin Khan, …); the
list should stay empty. `POD` marks a target-deck card.
"""
import re, json, pathlib
sc=json.load(open('scripts/.scryfall_cache.json'))
def oracle(n):
    c=sc.get(n)
    if not isinstance(c,dict):
        for k,v in sc.items():
            if isinstance(v,dict) and k.split(' // ')[0]==n: c=v;break
    if not isinstance(c,dict): return None
    faces=c.get('card_faces') or []
    return ' '.join([c.get('oracle_text') or '']+[f.get('oracle_text') or '' for f in faces])
dk=open('crabomination/src/pod/decks.rs').read()
fn_re=re.compile(r'^pub fn (\w+)\(\) -> (?:crate::card::)?CardDefinition',re.M)
for f in sorted(pathlib.Path('crabomination_catalog/src').rglob('*.rs')):
    s=f.read_text()
    starts=[m for m in fn_re.finditer(s)]
    for i,m in enumerate(starts):
        body=s[m.start(): starts[i+1].start() if i+1<len(starts) else len(s)]
        free=False
        for g in re.finditer(r'(?<!ExileTopAnd)GrantMayPlay \{(.{0,600}?)\n\s*\}',body,re.S):
            if 'pay_own_cost: false' in g.group(1): free=True
        for g in re.finditer(r'ExileTopAndGrantMayPlay \{(.{0,900}?)\n\s*\}',body,re.S):
            if 'pay_own_cost: false' in g.group(1) and 'pay_any_color: false' in g.group(1): free=True
        if not free: continue
        nm=re.search(r'name:\s*"([^"]+)"',body) or re.search(r'\(\s*"([^"]+)"',body)
        name=nm.group(1) if nm else m.group(1)
        t=oracle(name)
        if t is None: continue
        if re.search(r'without paying|mana value [0-9X]+ or less|pay \{0\}',t): continue
        pod='POD' if re.search(r'\b'+m.group(1)+r'\b',dk) else '   '
        print(pod, f.relative_to('crabomination_catalog/src'), m.group(1),'|',re.sub(r'\s+',' ',t)[:150])

for f in sorted(pathlib.Path('crabomination_catalog/src').rglob('*.rs')):
    s=f.read_text()
    starts=[m for m in fn_re.finditer(s)]
    for i,m in enumerate(starts):
        body=s[m.start(): starts[i+1].start() if i+1<len(starts) else len(s)]
        anyc=False
        for g in re.finditer(r'(?<!ExileTopAnd)GrantMayPlay \{(.{0,600}?)\n\s*\}',body,re.S):
            if 'pay_own_cost: true' in g.group(1) and 'any_color: true' in g.group(1): anyc=True
        for g in re.finditer(r'ExileTopAndGrantMayPlay \{(.{0,900}?)\n\s*\}',body,re.S):
            if 'pay_any_color: true' in g.group(1): anyc=True
        if not anyc: continue
        nm=re.search(r'name:\s*"([^"]+)"',body) or re.search(r'\(\s*"([^"]+)"',body)
        name=nm.group(1) if nm else m.group(1)
        t=oracle(name)
        if t is None or re.search(r'any type|any color',t): continue
        pod='POD' if re.search(r'\b'+m.group(1)+r'\b',dk) else '   '
        print('ANY', pod, f.relative_to('crabomination_catalog/src'), m.group(1),'|',re.sub(r'\s+',' ',t)[:150])
