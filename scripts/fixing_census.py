"""Fixing census over CRAB_DUMP_TRACES files from a splash deck's games.
Our seat = the seat whose library holds Extended Absence."""
import sys, os, re, collections
d = sys.argv[1]
BLACK = {"Extended Absence", "Break Under Pressure"}
BSRC = {"Swamp", "Stingerquill Annex"}
C = collections.Counter()
fetched = collections.Counter(); sws_took = collections.Counter(); marwyn_got = collections.Counter()
act_re = re.compile(r'^\s*\d+\. p(\d) (.*?)\s+~~\s+T(\d+) (\w+)')
snap_re = re.compile(r'^# snap T(\d+) (\w+) active p(\d) \|\| p0 .*? hand \[(.*?)\] crt .*\|\| p1 .*? hand \[(.*?)\] crt')
def hand_names(h): return [c.rsplit(':', 1)[0] for c in filter(None, h.split('; '))]
for fn in os.listdir(d):
    lines = open(os.path.join(d, fn)).read().splitlines()
    names = {}; seat = None
    for l in lines:
        m = re.match(r'# start p(\d) library: (.*)', l)
        if m:
            got = set()
            for it in re.split(r', (?=\d+=)', m.group(2)):
                i, n = it.split('=', 1); names[int(i)] = n; got.add(n)
            if {"Extended Absence", "Murmuring Volume", "Guiding Hydra"} <= got: seat = int(m.group(1))
    if seat is None: continue
    C['games'] += 1
    bsrc = 0; prev_hand = None; pending = None; starved_turns = set(); black_seen = False; cast_black = set()
    for l in lines:
        m = snap_re.match(l)
        if m:
            hand = hand_names(m.group(4 + seat)); active = int(m.group(3)) == seat; step = m.group(2)
            if pending and prev_hand is not None:
                new = [x for x in hand if x not in prev_hand or hand.count(x) > prev_hand.count(x)]
                for x in set(new):
                    pending[1][x] += 1
                pending = None
            if any(x in BLACK for x in hand):
                black_seen = True
                if bsrc == 0 and active and step == 'PreCombatMain':
                    starved_turns.add(int(m.group(1)))
            prev_hand = hand
            continue
        m = act_re.match(l)
        if not m or int(m.group(1)) != seat: continue
        a = m.group(2)
        ids = [int(x) for x in re.findall(r'CardId\((\d+)\)', a)]
        nm = names.get(ids[0], '?') if ids else ''
        if a.startswith('PlayLand') and nm in BSRC: bsrc += 1
        if a.startswith('Landcycle'):
            C['landcycle ' + nm] += 1; pending = ('lc', fetched)
        if a.startswith('CastSpell') and nm == 'Something Worth Saving':
            C['sws cast'] += 1; pending = ('sws', sws_took)
        if a.startswith('CastSpell') and nm in BLACK: C['cast ' + nm] += 1
        if a.startswith('CastSpell') and nm == 'Murmuring Volume': C['cast Volume'] += 1
        if a.startswith('ActivateAbility') and nm == 'Marwyn, the Preserver':
            C['marwyn activations'] += 1; pending = ('mar', marwyn_got)
        if 'ScryOrder' in a:
            mm = re.search(r'bottom: \[(.*?)\]', a)
            binned = [names.get(int(x), '?') for x in re.findall(r'(\d+)', mm.group(1))] if mm else []
            C['surveil decisions'] += 1
            for b in binned:
                if b in BSRC:
                    C['binned a B source'] += 1
                    if bsrc == 0 and prev_hand and any(x in BLACK for x in prev_hand): C['binned a B source while starved with black in hand'] += 1
    end = [l for l in lines if l.startswith(f'# end p{seat} hand:')]
    if end:
        for b in BLACK:
            if b in end[0]: C['stranded ' + b] += 1
    if black_seen: C['games a black spell reached hand'] += 1
    if starved_turns: C['games with a starved turn'] += 1; C['starved turns'] += len(starved_turns)
for k, v in sorted(C.items()): print(f"{k:55} {v}")
print("landcycle fetched:", dict(fetched)); print("SWS took:", dict(sws_took)); print("Marwyn returned:", dict(marwyn_got))
