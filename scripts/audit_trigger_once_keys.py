#!/usr/bin/env python3
"""A card sets `once_per_turn` / `once_per_batch`; does the SITE that pushes
its trigger read the flag?

`EventSpec::once_per_turn` (CR 603.3d, "only once each turn") and
`once_per_batch` (CR 603.2c, "whenever **one or more** …") are fields a card
sets and a *push site* has to honour. `dispatch_triggers_for_events` honours
them, and so does `combat.rs`'s attacker-side declaration loop. Seven other
hand-written walks over `definition.triggered_abilities` do not read the flag
at all, and until 2026-09-19 an eighth — the **defender**-side
`ControllerAttackedByOpponent` walk — did not either.

⚠⚠ **That one had a card with the flag set and a code comment claiming the
fix.** Coveted Jewel's says "CR 603.2c … one fire for the declaration. Without
the flag three unblocked attackers drew that opponent nine cards." The flag
was set; the site threw it away. Nothing caught it, because every gate in the
tree asks whether the *card* is right.

So this column asks the other question, and it is a **cross-check, not a
grep**: for every `(EventKind, EventScope)` pair a catalog card carries a once
key on, is the site that pushes that pair one that reads the key? The seven
latent sites are listed below with the pairs they own; a finding means a new
card just landed on one of them.

    python3 scripts/audit_trigger_once_keys.py            # the reading
    python3 scripts/audit_trigger_once_keys.py --sites    # the latent surface
    python3 scripts/audit_trigger_once_keys.py --check    # exit 1 on a finding

⚠ **A latent site is a surface, not a queue** (`audit_target_walkers`' sixth
column, same shape). There is nothing to fix on these until a card asks for
it — and fixing one speculatively costs a hot-path read per push for a
behaviour nothing exercises.
"""

import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "crabomination", "src")
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")

# (EventKind, EventScope) pairs pushed by a hand-written walk that does NOT
# read the once key, with the site that owns each. Read by hand 2026-09-19.
LATENT = {
    ("Attacks", "YouAttack-family"): "combat.rs — the `YouAttack` walk; it already runs once per declaration, so only `once_per_turn` would mean anything",
    ("YouAttack", "*"): "combat.rs — as above",
    ("ControllerDealtCombatDamage", "SelfSource"): "combat.rs — the per-damaging-source walk; a printed 'one or more creatures deal combat damage to you' would want the batch key",
    ("DoorUnlocked", "*"): "actions.rs — the Room door walk reads the door's own ability list",
    ("BecomesPlotted", "*"): "actions.rs — `push_plot_triggers`",
    ("Mutated", "SelfSource"): "stack.rs — the merged-pile walk over the host's unioned definition",
    ("YourInstantOrSorceryDealtDamage", "YourControl"): "effects/movement.rs — guarded by its own `spell_damage_trigger_fired` latch, which is a per-damage-event dedupe and not a per-turn or per-batch one",
    ("EntersBattlefield", "SelfSource"): "actions.rs — a permanent enters once, and the `multiplier` loop beside it is Panharmonicon, where a once key would be wrong",
}

SPEC_CALL = re.compile(
    r"EventSpec::new\(\s*EventKind::(\w+)[^)]*,\s*EventScope::(\w+)\s*\)"
    r"((?:\s*\.\w+\([^;]*?\))*)"
)
SPEC_LITERAL = re.compile(
    r"EventSpec\s*\{[^}]*?once_per_(batch|turn):\s*true[^}]*?"
    r"EventKind::(\w+),\s*EventScope::(\w+)",
    re.S,
)


def keyed_pairs():
    """{(kind, scope): count} for every catalog trigger carrying a once key."""
    out = {}
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            s = open(os.path.join(dirpath, f), encoding="utf-8").read()
            for m in SPEC_CALL.finditer(s):
                kind, scope, chain = m.group(1), m.group(2), m.group(3)
                if "once_per_batch" in chain or "once_per_turn" in chain:
                    out[(kind, scope)] = out.get((kind, scope), 0) + 1
            for m in SPEC_LITERAL.finditer(s):
                key = (m.group(2), m.group(3))
                out[key] = out.get(key, 0) + 1
    return out


def main():
    if "--sites" in sys.argv:
        for (kind, scope), why in sorted(LATENT.items()):
            print(f"{kind}/{scope}\n    {why}")
        print(f"# {len(LATENT)} latent sites")
        return 0

    pairs = keyed_pairs()
    hits = []
    for (kind, scope), n in sorted(pairs.items()):
        for lk, ls in LATENT:
            if lk == kind and ls in ("*", scope):
                hits.append((kind, scope, n, LATENT[(lk, ls)]))
    for kind, scope, n, why in hits:
        print(f"{kind}/{scope}: {n} card(s) set a once key on a pair no push site reads")
        print(f"    {why}")
    print(
        f"# {len(hits)} keyed pairs landing on a latent push site, "
        f"over {len(pairs)} keyed (kind, scope) pairs in the catalog"
    )
    return 1 if ("--check" in sys.argv and hits) else 0


if __name__ == "__main__":
    sys.exit(main())
