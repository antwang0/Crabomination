#!/usr/bin/env python3
"""Step triggers whose seat scope disagrees with the printed "whose step".

CR 503.1 / 513.1 — "at the beginning of each upkeep" / "each end step" / "the
next end step" is every player's step; "your upkeep" is the controller's. For
`StepBegins` the engine reads `YourControl` / `ActivePlayer` / `SelfSource` as
"the controller's turn", `AnyPlayer` as every turn and `OpponentControl` as
the opponents'. A duel hides the difference for half the steps; a pod for
three quarters of them (Tendershoot Dryad, Séance Board, Manaform Hellkite).

Reads the real definitions (granted and token triggers included):

    cargo build --profile release-fast -p crabomination --bin dump_cards
    target/release-fast/dump_cards --grep StepBegins > /tmp/step.tsv
    python3 scripts/audit_step_scope.py /tmp/step.tsv          # the rows
    python3 scripts/audit_step_scope.py /tmp/step.tsv --gate   # exit 1 on any

A row whose trigger carries a filter (an intervening "if it's your turn",
`from_opponent()`, the host's controller being active) is reported apart and
not gated: the filter is where the seat is decided.
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

STEPS = {
    "Upkeep": "upkeep",
    "End": "end step",
    "Draw": "draw step",
    "BeginCombat": "combat",
    "PreCombatMain": "(?:precombat|first) main phase",
    "PostCombatMain": "(?:postcombat|second) main phase",
}
MINE = {"YourControl", "ActivePlayer", "SelfSource"}
WHOSE = re.compile(
    r"at the beginning of (?:the )?(your |each opponent's |each player's |each |that player's |"
    r"enchanted player's |the chosen player's |target opponent's |each of your |each other player's |"
    r"the upkeep of |its controller's |enchanted creature's controller's |their )?(?:next )?{step}"
    r"( on your turn| on each of your turns| of each opponent's turn|,? if it's your turn|"
    r" on each opponent's turn|,? if it's not your turn|,? if it's an opponent's turn)?"
)
# Cards whose extra scope is a second, correctly-scoped trigger or a state
# approximation the oracle phrase can't see.
ALLOWLIST = {
    # "Your upkeep" pays; the `AnyPlayer` twin is the untap-step lock.
    "Island Fish Jasconius", "Serendib Djinn", "Drop of Honey",
}


def kinds_in(oracle, step):
    out = set()
    for m in re.finditer(WHOSE.pattern.format(step=step), oracle):
        w, t = (m.group(1) or "").strip(), (m.group(2) or "").strip()
        if oracle[m.end():].startswith(" of "):
            out.add("other")  # "the upkeep of enchanted creature's controller"
        elif "next " + step in m.group(0) and not w:
            out.add("next")  # "the next end step": a delayed trigger, or any
        elif w in ("your", "each of your") or ("your turn" in t and "not" not in t):
            out.add("mine")
        elif "opponent" in w or "opponent" in t or "not your" in t:
            out.add("opp")
        elif w in ("each", "each player's", ""):
            out.add("any")
        else:
            out.add("other")
    return out


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    gate = "--gate" in sys.argv
    if not args:
        sys.exit(__doc__)
    cache = {k: v for k, v in json.load(open(CACHE)).items() if isinstance(v, dict)}
    by_name = {}
    for k, v in cache.items():
        by_name.setdefault(k.lower(), v)
        by_name.setdefault(k.split(" // ")[0].lower(), v)
    rows, filtered = [], []
    for line in open(args[0]):
        name, dbg = line.rstrip("\n").split("\t", 1)
        card = by_name.get(name.lower())
        if not card or name in ALLOWLIST:
            continue
        oracle = card.get("oracle_text") or "\n".join(
            f.get("oracle_text", "") for f in card.get("card_faces", [])
        )
        oracle = re.sub(r"\([^)]*\)", "", oracle).lower()
        for step, phrase in STEPS.items():
            scopes = set()
            for m in re.finditer(
                r"StepBegins\(" + step + r"\), scope: (\w+), filter: (None|Some).*?actor_is_opponent: (\w+)", dbg
            ):
                s = "OpponentControl" if m.group(3) == "true" else m.group(1)
                if s.startswith("FromYourGraveyard"):
                    continue
                k = "mine" if s in MINE else "any" if s == "AnyPlayer" else "opp" if s == "OpponentControl" else s
                scopes.add((k, m.group(2) == "Some"))
            kinds = kinds_in(oracle, phrase)
            if not scopes or not kinds or "other" in kinds:
                continue
            have = {k for k, _ in scopes}
            # "The next end step" alone wants every turn; beside a printed
            # "your end step" it is a delayed trigger the scan can't see.
            want = kinds - {"next"} or {"any"}
            if have == want:
                continue
            row = f"{name:45} {step:7} def={sorted(k for k, _ in scopes)} printed={sorted(kinds)}"
            (filtered if any(f for _, f in scopes) else rows).append(row)
    for r in rows:
        print(r)
    if filtered:
        print(f"-- {len(filtered)} filtered (not gated):")
        for r in filtered:
            print("   " + r)
    print(f"{len(rows)} rows")
    if gate and rows:
        sys.exit(1)


main()
