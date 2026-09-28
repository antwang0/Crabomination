#!/usr/bin/env python3
"""The delayed end-step verb: printed vs modelled.

"Sacrifice it at the beginning of the next end step" modelled as an exile (or
a move to the graveyard, or a destroy) skips dies-vs-exile and "whenever you
sacrifice" payoffs; "exile it" modelled as a sacrifice lets the creature die
(Kiki-Jiki exiled its copy, Corpse Dance sacrificed a creature of its own
choosing, Junkyo Bell destroyed, Wings of Hubris dropped the clause). A row
names a card whose oracle prints the verb and whose definition holds no
delayed end-step body with it. `--gate` exits 1 on any row.

    target/debug/dump_cards --grep "" > /tmp/all.tsv
    python3 scripts/audit_eot_verb.py /tmp/all.tsv [--pod NAMES] [--gate]
"""

import json
import os
import re
import sys

CACHE = os.path.join(os.path.dirname(os.path.abspath(__file__)), ".scryfall_cache.json")
# Read and right: a dedicated effect carries the delayed verb itself.
DEDICATED = {
    "Echo Chamber",  # TokenCopyOfOpponentChoice exiles
    "Emperor of Bones",  # the reanimation half is omitted (its doc)
    "Feather, the Redeemed",  # MarkExileReturnOnResolve
    "Flickerform",  # FlickerHostWithAuras
    "Hungry for More",  # the token's own end-step sacrifice
    "Ignorant Bliss",  # Effect::IgnorantBliss
    "Manaform Hellkite",  # the token's own end-step exile
    "Mirror March",  # FlipUntilLossThenTokenCopies exiles
    "Purphoros's Intervention",  # the token's own end-step sacrifice
    "Random Encounter",  # MillDeployCreaturesUntilEndStep
}


def oracle(c):
    t = c.get("oracle_text") or ""
    if c.get("card_faces"):
        t += "\n".join(f.get("oracle_text", "") for f in c["card_faces"])
    return re.sub(r"\([^)]*\)", "", t)


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    pod = None
    if "--pod" in sys.argv:
        pod = {ln.strip() for ln in open(sys.argv[sys.argv.index("--pod") + 1], encoding="utf-8")}
    PHR = re.compile(r'(?:(sacrifice|exile|return)\s+(?:it|them|that token|those tokens|that creature|the token|the tokens|those creatures|that permanent|it to its owner\'s hand)[^.]{0,40}?at the beginning of the next end step|at the beginning of the next end step, (sacrifice|exile|return))',re.I)
    out=[]
    for line in open(sys.argv[1], encoding="utf-8"):
        if '\t' not in line: continue
        name,dbg=line.rstrip('\n').split('\t',1)
        c = cache.get(name)
        if not isinstance(c,dict): continue
        if (pod is not None and name not in pod) or name in DEDICATED: continue
        verbs=set(v.lower() for m in PHR.finditer(oracle(c)) for v in m.groups() if v)
        if not verbs: continue
        # definition verbs inside delayed end-step bodies
        have=set()
        for m in re.finditer(r'(?:DelayUntil \{ kind: (?:Your)?NextEndStep[^,]*, body:|AtNextEndStep \{ body:) (?:If \{ cond: [^,]*, then: )?(?:Seq\(\[)?(\w+)(?: \{ what: \w+[^,]*, to: (\w+))?',dbg):
            b,to=m.group(1),m.group(2)
            if b.startswith('Sacrific'): have.add('sacrifice')
            elif b.startswith('Exile'): have.add('exile')
            elif b=='Move' and to in ('Exile','ExileWithSourceStamp'): have.add('exile')
            elif b=='Move' and to=='Graveyard': have.add('sacrifice?')
            elif b in('Move','Return','Bounce','ReturnToOwner'): have.add('return?')
            else: have.add(b)
        for k,v in [('SacrificeAtNextEndStep','sacrifice'),('SacrificeLastCreatedTokensAtNextEndStep','sacrifice'),('ExileLastCreatedTokensAtNextEndStep','exile'),('ExileAtNextEndStep','exile'),('ExileReturnNextEndStep','return?'),('ExileReturnToOwnerNextEndStep','return?'),('ReturnToOwnersHandAtNextEndStep','return?'),('sacrifice_eot: true','sacrifice'),('sacrifice_at_next_end_step: true','sacrifice'),('return_eot: true','return?'),('return_to_hand_eot: true','return?'),('return_at_end_step: true','return?')]:
            if k in dbg: have.add(v)
        for m in re.finditer(r'CreateTokenCopiesHasteSac \{',dbg):
            e=re.search(r'exile: (true|false)',dbg[m.end():m.end()+600])
            if e: have.add('exile' if e.group(1)=='true' else 'sacrifice')
        # a delayed body anywhere: read the verbs inside the next 400 chars
        for m in re.finditer(r'(?:DelayUntil(?:WithCapture)? \{ kind: (?:Your)?NextEndStep|AtNextEndStep \{)',dbg):
            seg=dbg[m.end():m.end()+400]
            if re.search(r'\bSacrific',seg): have.add('sacrifice')
            if re.search(r'\bExile\b|to: Exile',seg): have.add('exile')
            if re.search(r'to: (Hand|Battlefield)|Return',seg): have.add('return?')
        miss=[v for v in verbs if v not in have and not (v=='return' and 'return?' in have)]
        if miss:
            out.append(f"{name}\tprinted={sorted(verbs)} have={sorted(have)}")
    print("\n".join(sorted(out)))
    print(f"{len(out)} rows", file=sys.stderr)
    if "--gate" in sys.argv and out:
        sys.exit(1)


if __name__ == "__main__":
    main()
