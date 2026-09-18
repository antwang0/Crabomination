#!/usr/bin/env python3
"""Cards whose oracle says "each opponent" but whose body never names one.

The method that found Esper Sentinel and Adeline, widened from the pod decks
to the whole catalog: read the printed text for a per-opponent clause, read
the factory body for a fan-out reference, and list the ones with the first
and not the second.
"""
import json, re, pathlib

root = pathlib.Path('crabomination_catalog/src')
fn_re = re.compile(r'^pub fn (\w+)\(\) -> CardDefinition', re.M)
name_re = re.compile(r'name:\s*"((?:[^"\\]|\\.)*)"')
# Every `fn` in a file, not only the card factories: a card that says "each
# opponent loses 1 life" usually says it through a local `drain()` helper, and
# reading the factory body alone calls that a gap.
any_fn_re = re.compile(r'^(?:pub )?(?:const )?fn (\w+)', re.M)
file_src = {}
bodies = {}
for p in root.rglob('*.rs'):
    s = p.read_text()
    file_src[str(p)] = s
    fns = [(m.start(), m.group(1)) for m in fn_re.finditer(s)]
    for i, (pos, ident) in enumerate(fns):
        end = fns[i + 1][0] if i + 1 < len(fns) else len(s)
        body = s[pos:end]
        m = name_re.search(body)
        if m:
            nm = m.group(1).replace('\\"', '"').replace("\\'", "'")
            bodies.setdefault(nm, (ident, str(p), body))

sc = json.load(open('scripts/.scryfall_cache.json'))
# A clause that is per-opponent and therefore needs a fan-out.
CLAUSE = re.compile(r'each opponent|each other player|each of your opponents', re.I)
# Trigger *timing* ("at the beginning of each opponent's upkeep", "during each
# other player's untap step") is the event system's job, not a fan-out — the
# ability already fires once per such step. Only the clauses that name
# opponents as the *recipients* of an effect need one.
TIMING = re.compile(
    r"(at the beginning of|during|before|after)[^.]{0,40}each (opponent|other player)",
    re.I)
# Reminder text in parentheses is not rules text for this purpose.
PAREN = re.compile(r'\([^)]*\)')
FANOUT = re.compile(
    r'EachOpponent|ForEachOpponent|EachOtherPlayer|EachPlayerDoes|EachPlayer\b'
    r'|OpponentCount|AllOpponents|Opponents\b'
    # the lowercase `effect::shortcut` helpers that mint the same refs
    r'|each_opponent|each_other_player|all_opponents|opponents\(|opponent_count'
    r'|EachPlayerMatching|EachOther|OpponentsLoseLife|Extort|extort')
rows = []
for nm, (ident, path, body) in bodies.items():
    e = sc.get(nm)
    if not isinstance(e, dict):
        continue
    text = PAREN.sub('', e.get('oracle_text') or '')
    clauses = [l for l in re.split(r'[.\n]', text) if CLAUSE.search(l)]
    clauses = [c for c in clauses if not TIMING.search(c)]
    if not clauses:
        continue
    # Inline every same-file helper the body calls (one level is enough —
    # the helpers are leaves).
    src = file_src[path]
    reach = body
    for callee in set(re.findall(r'\b([a-z_][a-z0-9_]*)\(', body)):
        m = re.search(r'^(?:pub )?fn ' + callee + r'\b', src, re.M)
        if m:
            nxt = re.search(r'^(?:pub )?fn ', src[m.end():], re.M)
            reach += src[m.start(): m.end() + (nxt.start() if nxt else len(src))]
    if FANOUT.search(reach):
        continue
    rows.append((nm, ident, path.split('/')[-1], clauses[0].strip()))
rows.sort()
print(f"{len(rows)} implemented cards print a per-opponent clause with no fan-out ref in the body\n")
for nm, ident, f, clause in rows:
    print(f"- {nm}  [{f}::{ident}]")
    print(f"    {clause[:170]}")
