#!/usr/bin/env python3
"""Target controller scopes: printed "target X you control" vs the definition's filter.

Leads, not a gate (multi-word types and "you own" read as mismatches). Rows:
`YOURS->ANY` — the oracle's every target is "you control" and a definition
target filter never says `ControlledByYou`; `ANY->YOURS` — the reverse. Found
"you own" shipped as "you control" (Slip On the Ring, Charming Prince, Sword
of Hearth and Home) and Demonic Dread's invented body.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_target_controller.py /tmp/all.tsv [--pod]
"""
import json,re,sys
sys.path.insert(0, __import__('os').path.dirname(__import__('os').path.abspath(__file__)))
from audit_counter_kinds import CACHE, oracle_of, pod_idents, snake
S=None
c=json.load(open(CACHE))
pod=pod_idents()
only_pod='--pod' in sys.argv
PHRASE=re.compile(r'\btarget (creature|permanent|artifact|land|enchantment|nonland permanent|artifact or creature)( you control| you don\'t control| an opponent controls| target player controls| that player controls| defending player controls)?\b',re.I)
def filters(dbg):
    # every TargetFiltered filter text (balanced parens)
    out=[]
    for m in re.finditer(r'TargetFiltered \{ slot: \d+, filter: ',dbg):
        i=m.end(); depth=0; j=i
        while j<len(dbg):
            ch=dbg[j]
            if ch in '({[': depth+=1
            elif ch in ')}]':
                if depth==0: break
                depth-=1
            elif ch==',' and depth==0: break
            j+=1
        out.append(dbg[i:j])
    return out
for line in open(sys.argv[1]):
    name,dbg=line.rstrip('\n').split('\t',1)
    if only_pod and snake(name) not in pod: continue
    card=c.get(name)
    if not isinstance(card,dict): continue
    o=oracle_of(card)
    ph=[(m.group(1).lower(),(m.group(2) or '').strip()) for m in PHRASE.finditer(o)]
    if not ph: continue
    fs=filters(dbg)
    if not fs: continue
    yours=[p for p in ph if p[1]=='you control']
    free=[p for p in ph if p[1]=='']
    fy=[f for f in fs if 'ControlledByYou' in f and 'Not(ControlledByYou)' not in f]
    ff=[f for f in fs if 'Controlled' not in f and 'Opponent' not in f and 'Owned' not in f]
    if yours and not free and ff and not fy:
        print('YOURS->ANY\t'+('POD ' if snake(name) in pod else '')+name+'\t'+' | '.join(ff)[:160])
    if free and not yours and fy and not ff:
        print('ANY->YOURS\t'+('POD ' if snake(name) in pod else '')+name+'\t'+' | '.join(fy)[:160])
