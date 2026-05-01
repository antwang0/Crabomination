#!/usr/bin/env python3
"""Summarize a Crabomination debug-state JSON dump.

Picks out the most useful fields for debugging:
- Top-level: turn, step, active player, priority holder, alive players
- Each player: life, hand size, library size, graveyard size,
  battlefield permanents (with key state: tapped, summoning_sick, counters)
- Stack: kind + name + controller + target for each item
- Pending decision: kind + acting player + key fields
- Suspend signal / delayed triggers / opening-hand hints if present
"""

import json
import sys
from pathlib import Path

if len(sys.argv) < 2:
    print("usage: summarize_debug.py <path-to-state-*.json>", file=sys.stderr)
    sys.exit(2)

path = Path(sys.argv[1])
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

doc = json.loads(path.read_text(encoding="utf-8"))

# Some debug exports wrap the state under a top-level key (e.g. {"state": ...})
state = doc.get("state") or doc.get("full_state") or doc

print(f"=== {path.name} ===")
print(f"Turn       : {state.get('turn_number', '?')}")
print(f"Step       : {state.get('step', '?')}")
print(f"Active     : {state.get('active_player_idx', '?')}")
prio = state.get("priority", {})
print(f"Priority   : seat={prio.get('player_with_priority', '?')}  "
      f"passes={prio.get('consecutive_passes', '?')}")
go = state.get("game_over")
if go:
    print(f"Game over  : {go}")

print()
players = state.get("players", [])
for idx, p in enumerate(players):
    # Seat may not be on the player JSON; the index is authoritative.
    seat = p.get("seat", idx)
    name = p.get("name", "?")
    life = p.get("life", "?")
    hand = p.get("hand", []) or []
    library = p.get("library", []) or []
    graveyard = p.get("graveyard", []) or []
    eliminated = p.get("eliminated", False)
    wants_ui = p.get("wants_ui", False)
    drawn = p.get("cards_drawn_this_turn", 0)
    gained = p.get("life_gained_this_turn", 0)
    pool = p.get("mana_pool", {})
    pool_str = " ".join(
        f"{k[0].upper()}{v}"
        for k, v in pool.items()
        if isinstance(v, (int, float)) and v
    ) or "-"
    print(f"-- Seat {seat} ({name}) {'[OUT]' if eliminated else ''} "
          f"{'[UI]' if wants_ui else '[bot]'}")
    print(f"   life={life} hand={len(hand)} lib={len(library)} gy={len(graveyard)}"
          f" pool={pool_str} drawn-this-turn={drawn} gained-this-turn={gained}")
    if hand:
        names = [(c.get("name") or c.get("definition", {}).get("name", "?")) for c in hand]
        print(f"   hand: {names}")
    if graveyard:
        names = [(c.get("name") or c.get("definition", {}).get("name", "?")) for c in graveyard]
        print(f"   gy:   {names[:8]}{'...' if len(names) > 8 else ''}")

print()
bf = state.get("battlefield", []) or []
print(f"Battlefield ({len(bf)} permanents):")
for c in bf:
    cname = c.get("name") or c.get("definition", {}).get("name", "?")
    cid = c.get("id", "?")
    ctrl = c.get("controller", "?")
    owner = c.get("owner", "?")
    tapped = c.get("tapped", False)
    sick = c.get("summoning_sick", False)
    pwr = c.get("power_bonus", 0)
    tuf = c.get("toughness_bonus", 0)
    # `counters` is `Vec<(CounterType, u32)>` → list of [key, value] pairs.
    counters = c.get("counters", []) or []
    flags = []
    if tapped: flags.append("tap")
    if sick: flags.append("sick")
    if pwr or tuf: flags.append(f"pump+{pwr}/+{tuf}")
    if counters:
        bits = []
        for entry in counters:
            if isinstance(entry, (list, tuple)) and len(entry) == 2:
                bits.append(f"{entry[0]}={entry[1]}")
            elif isinstance(entry, dict):
                for k, v in entry.items():
                    bits.append(f"{k}={v}")
        if bits:
            flags.append(",".join(bits))
    if owner != ctrl:
        flags.append(f"owner={owner}")
    flag_str = " " + " ".join(flags) if flags else ""
    print(f"  [#{cid}] {cname} (ctrl={ctrl}){flag_str}")

print()
stack = state.get("stack", []) or []
print(f"Stack ({len(stack)} items, top is last):")
for i, item in enumerate(stack):
    if "Spell" in item:
        s = item["Spell"]
        card = s.get("card", {})
        name = card.get("definition", {}).get("name") or "?"
        caster = s.get("caster", "?")
        tgt = s.get("target")
        x = s.get("x_value", 0)
        cv = s.get("converged_value", 0)
        print(f"  [{i}] SPELL {name} caster={caster} target={tgt} x={x} cv={cv}")
    elif "Trigger" in item:
        t = item["Trigger"]
        src = t.get("source", "?")
        ctrl = t.get("controller", "?")
        tgt = t.get("target")
        x = t.get("x_value", 0)
        print(f"  [{i}] TRIGGER source=#{src} ctrl={ctrl} target={tgt} x={x}")
    else:
        print(f"  [{i}] {item!r}"[:200])

print()
pd = state.get("pending_decision")
if pd:
    print("Pending decision:")
    print(f"  acting_player: {pd.get('acting_player') or pd.get('decision', {}).get('acting_player')}")
    decision = pd.get("decision")
    if isinstance(decision, dict):
        for kind, body in decision.items():
            print(f"  kind: {kind}")
            if isinstance(body, dict):
                for k, v in body.items():
                    if isinstance(v, list) and len(v) > 6:
                        print(f"    {k}: <{len(v)} items>")
                    else:
                        print(f"    {k}: {v}")
    else:
        print(f"  decision: {decision!r}"[:400])
else:
    print("Pending decision: (none)")

print()
ss = state.get("suspend_signal")
if ss:
    print("Suspend signal: present (engine paused mid-resolution)")
    print(f"  {str(ss)[:300]}")

dt = state.get("delayed_triggers", [])
if dt:
    print(f"\nDelayed triggers: {len(dt)}")
    for d in dt[:5]:
        print(f"  source=#{d.get('source','?')} ctrl={d.get('controller','?')} "
              f"kind={d.get('kind','?')} fires_once={d.get('fires_once','?')}")

print()
top = doc.get("kind")
if top:
    print(f"(dump kind: {top})")
elapsed = doc.get("elapsed_secs")
if elapsed is not None:
    print(f"(elapsed since last progress: {elapsed}s)")
