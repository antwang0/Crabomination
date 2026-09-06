import json,sys,glob,collections
root=sys.argv[1]; prefix=sys.argv[2]
self_face=collections.Counter(); opp_face=collections.Counter(); seatstat=[collections.Counter(),collections.Counter()]
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
        p=v['player']; nm=names.get(v['card_id'],'?'); j=i+1; s=o=0; ds=do=0
        while j<len(ev) and j<i+40:
            kk,vv=ev[j]
            if kk in ('SpellCast','TurnStarted','AttackerDeclared','StepChanged') and not (kk=='StepChanged' and vv in ('PostCombatMain','End')): 
                if kk!='StepChanged': break
            if kk=='DamageDealt' and vv.get('to_player') is not None:
                if vv['to_player']==p: s+=vv['amount']
                else: o+=vv['amount']
            if kk=='CardDiscarded':
                if vv['player']==p: ds+=1
                else: do+=1
            j+=1
        who='deck' if p==seat else 'opp'
        if s and not o: self_face[nm]+=1; seatstat[0 if who=='deck' else 1]['self_face_casts']+=1; seatstat[0 if who=='deck' else 1]['self_face_dmg']+=s
        if o and not s: opp_face[nm]+=1
        if ds and not do: seatstat[0 if who=='deck' else 1]['self_discard_casts']+=1
        seatstat[0 if who=='deck' else 1]['casts']+=1
print('deck seat:',dict(seatstat[0])); print('opp seats:',dict(seatstat[1]))
print('spells hitting own face (casts):',self_face.most_common(12))
print('spells hitting opp face (casts):',opp_face.most_common(12))
