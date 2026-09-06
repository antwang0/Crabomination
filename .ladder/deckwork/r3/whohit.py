import json,sys,glob,collections
root=sys.argv[1]; prefix=sys.argv[2]; targets=set(sys.argv[3].split(';'))
out={t:collections.Counter() for t in targets}
allspells=collections.Counter()  # (name) -> counter of 'self_discard'/'opp_discard'
for path in sorted(glob.glob(root+'/*.jsonl')):
    names={}; ev=[]; seat=None
    for line in open(path):
        d=json.loads(line)
        if 'replay' in d: seat=next((i for i,p in enumerate(d['players']) if p.startswith(prefix)),None); continue
        if 'n' in d: names.update({int(k):v for k,v in d['n'].items()})
        for e in d.get('e',[]):
            k,v=next(iter(e.items())) if isinstance(e,dict) else (e,None); ev.append((k,v))
    for i,(k,v) in enumerate(ev):
        if k!='SpellCast': continue
        p=v['player']; nm=names.get(v['card_id'],'?')
        j=i+1; rec=collections.Counter()
        while j<len(ev) and j<i+40:
            kk,vv=ev[j]
            if kk in ('SpellCast','TurnStarted','AttackerDeclared'): break
            if kk=='CardDiscarded': rec['discard_self' if vv['player']==p else 'discard_opp']+=1
            if kk=='DamageDealt' and vv.get('to_player') is not None: rec['dmg_self' if vv['to_player']==p else 'dmg_opp']+=vv['amount']
            if kk=='CardDrawn': rec['draw_self' if vv['player']==p else 'draw_opp']+=1
            if kk=='LifeGained': rec['gain_self' if vv['player']==p else 'gain_opp']+=vv['amount']
            if kk=='LifeLost': rec['lose_self' if vv['player']==p else 'lose_opp']+=vv['amount']
            j+=1
        if nm in targets and p==seat:
            out[nm]['casts']+=1
            for a,b in rec.items(): out[nm][a]+=b
            out[nm]['casts_with_self_discard']+= 1 if rec['discard_self'] else 0
            out[nm]['casts_with_opp_discard']+= 1 if rec['discard_opp'] else 0
        if rec['discard_self'] and not rec['discard_opp']:
            allspells[nm]+=1
for t,c in out.items(): print(t, dict(c))
print("\nany-seat casts followed by caster-only discards, top 15:", allspells.most_common(15))
