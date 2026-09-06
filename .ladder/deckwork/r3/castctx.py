import json,sys,glob,collections
root=sys.argv[1]; prefix=sys.argv[2]; targets=sys.argv[3].split(';'); show=int(sys.argv[4]) if len(sys.argv)>4 else 0
kinds=collections.Counter()
stats={t:collections.defaultdict(list) for t in targets}
shown=0
for path in sorted(glob.glob(root+'/*.jsonl')):
    names={}; seat=None; turn=0; active=None
    hand=[7,7]; mana=[0,0]; manacol=[set(),set()]
    events=[]
    for line in open(path):
        try: d=json.loads(line)
        except: continue
        if 'replay' in d:
            seat=next((i for i,p in enumerate(d['players']) if p.startswith(prefix)),None); continue
        if 'n' in d: names.update({int(k):v for k,v in d['n'].items()})
        for ev in d.get('e',[]):
            if isinstance(ev,str): k,v=ev,None
            else: k,v=next(iter(ev.items()))
            events.append((k,v))
    for i,(k,v) in enumerate(events):
        kinds[k]+=1
        if k=='TurnStarted': active=v['player']; turn=v['turn']; mana=[0,0]; manacol=[set(),set()]
        elif k=='CardDrawn': hand[v['player']]+=1
        elif k=='LandPlayed': hand[v['player']]-=1
        elif k=='CardDiscarded': hand[v['player']]-=1
        elif k=='ManaAdded': mana[v['player']]+=1; manacol[v['player']].add(v.get('color'))
        elif k=='SpellCast':
            p=v['player']; hand[p]-=1
            nm=names.get(v['card_id'],'?')
            if p==seat and nm in stats:
                opp=1-p
                # look ahead until the next SpellCast/TurnStarted/StepChanged-to-combat
                disc=0; dmg=[]; drew=0; entered=[]; j=i+1
                while j<len(events) and j<i+40:
                    kk,vv=events[j]
                    if kk in ('SpellCast','TurnStarted'): break
                    if kk=='CardDiscarded' and vv['player']==opp: disc+=1
                    if kk=='DamageDealt': dmg.append((vv.get('amount'),'player' if vv.get('to_player') is not None else names.get(vv.get('to_card_id',vv.get('card_id',-1)),'?')))
                    if kk=='CardDrawn' and vv['player']==p: drew+=1
                    if kk=='PermanentEntered': entered.append(names.get(vv['card_id'],'?'))
                    j+=1
                stats[nm]['rows'].append(dict(turn=turn,active=active==p,mana=mana[p],cols=len([c for c in manacol[p] if c]),opp_hand=hand[opp],disc=disc,dmg=dmg,drew=drew,entered=entered))
                if shown<show:
                    shown+=1; print(path.split('/')[-1],'T',turn,nm,'mana',mana[p],'opp_hand',hand[opp]); print('   ',[ (kk, (vv if not isinstance(vv,dict) else {a:(names.get(b,b) if a.endswith('card_id') else b) for a,b in vv.items()})) for kk,vv in events[i+1:min(i+14,len(events))]])
            mana[p]=0; manacol[p]=set()
print('kinds:',sorted(kinds.items(),key=lambda t:-t[1]))
for t,st in stats.items():
    rows=st['rows']
    if not rows: print(t,'no casts'); continue
    n=len(rows)
    print(f"\n{t}: {n} casts; mean turn {sum(r['turn'] for r in rows)/n:.1f}; mean mana {sum(r['mana'] for r in rows)/n:.2f}; mana dist {dict(sorted(collections.Counter(r['mana'] for r in rows).items()))}")
    print(f"  opp hand at cast dist {dict(sorted(collections.Counter(max(r['opp_hand'],0) for r in rows).items()))}; discards dist {dict(sorted(collections.Counter(r['disc'] for r in rows).items()))}")
    print(f"  drew dist {dict(sorted(collections.Counter(r['drew'] for r in rows).items()))}; entered {collections.Counter(e for r in rows for e in r['entered']).most_common(8)}")
    print(f"  dmg {collections.Counter(str(x) for r in rows for x in r['dmg']).most_common(10)}")
