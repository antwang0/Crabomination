#!/usr/bin/env python3
"""Engine loops that walk SEAT INDICES instead of the players in the game.

CR 800.4a: a player who has left the game is not a player. Every fan-out that
goes through `resolve_players` already knows this — `EachPlayer`,
`EachOpponent`, `EachTeammate` and the rest all filter `is_alive()` — but a
hand-written `for seat in 0..self.players.len()` or `(controller + i) % n`
walk does not, and **both shapes are the same list in a duel and in a pod that
has lost nobody**, which is why six of them survived review.

What they cost when a seat *has* left (all four found 2026-09-19):

  * `Effect::Vote` — CR 701.38a's ballot. The departed seat's ask was
    re-seated onto a live opponent by CR 800.4g, that opponent answered, and
    the answer was tallied as the dead seat's vote. A four-seat pod down to
    two decided a will-of-the-council four votes to nothing.
  * `EachPlayerDestroysChosenFromLeftNeighbor` — Grenzo's Rebuttal's "the
    player to their left" is the next *player*. Aiming at the next seat index
    aimed at a board CR 800.4a had already emptied, so the strip did nothing.
  * `GoblinGame` — a departed seat's hidden count entered `fewest`, deciding
    who paid half their life.
  * `EachPlayerMayDiscardUpToThenDamage` — Mind Bomb offered a departed seat a
    trade it could not take (800.4a emptied its hand) and then dealt it the
    full three.

The safe sources are `GameState::seats_in_turn_order_from` (a printed
"starting with you, each player …", and `next_alive_seat` for "the player to
their left") and `resolve_players(&PlayerRef::EachPlayer, ctx)` (CR 101.4
APNAP order). A loop that reads neither and carries no `is_alive` guard of its
own is a finding.

Scope: loops whose body **asks a seat** (`ask_seat_*`) or **accumulates one
entry per seat**, because those are the two ways a dead seat's participation
becomes visible. A walk that only clears per-player state is not a finding —
a departed seat's state still has to be cleared.

Run: `python3 scripts/audit_seat_walks.py`   (exit 1 on any finding)
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
FILES = [
    "crabomination/src/game/effects/mod.rs",
    "crabomination/src/game/stack.rs",
    "crabomination/src/game/combat.rs",
    "crabomination/src/game/mod.rs",
    "crabomination/src/game/actions.rs",
]

# The two walk shapes. `RANGE` is "every seat index"; `ROTATE` is "every seat
# index, starting somewhere" — the one that looks like turn order and is not.
RANGE = re.compile(r"for (\w+) in 0\.\.(?:self\.)?players\.len\(\)\s*\{")
ROTATE = re.compile(r"let (\w+) = \([^)]*\+ \w+\) % (?:n|self\.players\.len\(\));")

# A body that asks is a body where a dead seat is handed a question.
ASKS = re.compile(r"ask_seat_\w+|seat_prompts|ask_player")
# A body that pushes one entry per seat is a body where a dead seat's answer
# reaches a tally (`fewest`, a ballot, an ordered list).
TALLIES = re.compile(r"\.push\(\(\s*\w+\s*,|votes\[|counts\.push|answers\.push")
# What makes a walk legitimate.
GUARD = re.compile(r"is_alive\(\)|\.eliminated|left_game|seats_in_turn_order_from|resolve_players\(")


def body_of(src, start):
    """The brace-matched block that begins at the first `{` after `start`."""
    i = src.index("{", start)
    depth = 0
    for k in range(i, len(src)):
        if src[k] == "{":
            depth += 1
        elif src[k] == "}":
            depth -= 1
            if depth == 0:
                return src[i : k + 1]
    return src[i:]


def enclosing_arm(src, at):
    """The nearest `Effect::Name` / `fn name` above `at`, for the report."""
    head = src[:at]
    m = None
    for m in re.finditer(r"Effect::(\w+)[ {]|fn (\w+)\(", head):
        pass
    if not m:
        return "?"
    return m.group(1) or m.group(2)


def main():
    findings = []
    for rel in FILES:
        path = ROOT / rel
        src = path.read_text()
        for pat in (RANGE, ROTATE):
            for m in pat.finditer(src):
                # A rotation is a statement, not a block: take the enclosing
                # `for` loop's body instead.
                if pat is ROTATE:
                    head = src.rfind("for ", 0, m.start())
                    if head == -1:
                        continue
                    body = body_of(src, head)
                else:
                    body = body_of(src, m.start())
                if GUARD.search(body):
                    continue
                if not (ASKS.search(body) or TALLIES.search(body)):
                    continue
                line = src[: m.start()].count("\n") + 1
                findings.append((rel, line, enclosing_arm(src, m.start()), m.group(0).strip()))

    for rel, line, arm, text in findings:
        print(f"- {rel}:{line}  {arm}\n    {text}")
    print(
        f"\n{len(findings)} seat-index walks that ask or tally without a CR 800.4a guard"
    )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main())
