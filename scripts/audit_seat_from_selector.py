#!/usr/bin/env python3
"""Which asking arms re-derive the ASKED SEAT on their re-run.

An arm that suspends hands `ask_seat_*` an `Effect` to re-run when the answer
comes back, and almost every one hands it the arm's own `effect`. That is right
for the arm's *work* and wrong for the arm's *seat* whenever the seat came out of
a selector: the re-run resolves the selector again, against a board the first
pass — or an earlier sibling in the same `Seq` — has already changed.

Ghost Quarter is the worked example (ENGINE_BACKLOG's seventeenth find). Its
ability is `Seq[Destroy target land, MayDoBy{ ControllerOf(Target(0)), … }]`, so
by the time the arm asks, the land the selector points at is in a graveyard; on
the re-run the selector could resolve to nothing at all, the arm returned at its
own `let Some(seat)`, and the compensation search never happened. A sweep found
it as an answer-log leak, which is the symptom of returning past a replayed
answer.

**THE FIX IS IN THE ASK HELPER NOW, NOT IN THE ARMS.** Every `ask_seat_*`
queues `effect.with_asked_seat(seat)` instead of `effect.clone()`, and
`Effect::with_asked_seat` (crabomination_base) writes `PlayerRef::Seat(seat)`
into the `who` field of the arms listed in its one match. So an arm is fixed by
appearing in that list, and this audit reads the list rather than re-deriving
it — the script and the code cannot disagree about what is covered.

The columns:

  * **pinned** — the variant is in `with_asked_seat`'s arm list. Nothing to do.
  * **controller** — the ask is routed to `ctx.controller`, not to the
    selector-derived seat (`Fateseal`, `ChooseFromHandToTopOfLibrary`,
    `GuessColorCountInHand`, `MoveChosen` look at another seat's cards and ask
    the *resolving* seat about them). `ctx.controller` is resolution state, not
    a board read, so it is stable across a re-run: not this class, and pinning
    `who` there would rewrite the victim instead of the asker.
  * **loop** — the asks are inside `for … in self.resolve_players(who, …)`. The
    same re-derivation applies to the *list*, and one seat pinned into it would
    drop every other seat's question, so `with_asked_seat` must not cover these.
    Closing them needs a seat-LIST ref; none of the eight has a shipped card
    that reaches the failure (the scan below) and they are listed, not
    allowlisted.
  * **open** — a single-seat arm outside the pin list. A defect; add the arm.

The catalog scan behind "no shipped card reaches it", for whoever repeats it:
every `who: PlayerRef::(ControllerOf|OwnerOf)(Selector::Target(N))` in the
catalog whose enclosing card has an earlier `Destroy` / `Exile` / `Sacrifice`,
then read each hit to see whether the arm it feeds is one of these (most feed
`Effect::Search`, whose resume applies the pick rather than re-running the arm).
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "crabomination/src/game/effects/mod.rs"
QUERY = ROOT / "crabomination_base/src/effect/query.rs"

ASKS = (
    "self.ask_seat_bool(",
    "self.ask_seat_amount(",
    "self.ask_seat_cards_logged(",
    "self.ask_seat_target_logged(",
    "self.ask_seat_option(",
    "self.ask_seat_cards(",
    "self.choose_up_to_cards(",
)


def pinned_variants() -> set:
    """The arm list of `Effect::with_asked_seat`, read off the source."""
    text = QUERY.read_text()
    m = re.search(r"pub fn with_asked_seat\(&self, seat: usize\) -> Effect \{(.*?)\n    \}", text, re.S)
    if not m:
        sys.exit("with_asked_seat not found — the audit reads its arm list")
    return set(re.findall(r"Effect::([A-Za-z0-9_]+) \{ who", m.group(1)))


def indent(line: str) -> int:
    return len(line) - len(line.lstrip())


def ask_seat_arg(body, j):
    """The seat argument of the `ask_seat_*` call starting at body[j]."""
    for line in body[j + 1 : j + 4]:
        arg = line.strip().rstrip(",")
        if not arg or arg.startswith("&mut cursor"):
            continue
        return arg
    return ""


def main() -> int:
    lines = SRC.read_text().split("\n")
    # Split into scopes: each `Effect::Variant .. => {` arm, and each `fn`.
    starts = []
    for i, line in enumerate(lines):
        st = line.strip()
        m = re.match(r"(?:pub(?:\(crate\))? )?fn ([a-z0-9_]+)", st)
        if m:
            starts.append((m.group(1), i, indent(line)))
            continue
        if "=> {" in st:
            v = re.search(r"Effect::([A-Za-z0-9_]+)", st)
            if v:
                starts.append((v.group(1), i, indent(line)))
                continue
            # A destructuring head spread over several lines closes with a bare
            # `} => {`; without this the arm is invisible and its whole body is
            # read as part of the arm ABOVE it (which is how a `who`-less
            # `TapAnyNumberThenCounters` got flagged for a selector-derived
            # seat that belonged to the arm two below it).
            if st.startswith("}"):
                for k in range(i - 1, max(i - 60, -1), -1):
                    if indent(lines[k]) != indent(line):
                        continue
                    v = re.match(r"Effect::([A-Za-z0-9_]+)\s*\{", lines[k].strip())
                    if v:
                        starts.append((v.group(1), k, indent(line)))
                    break
    scopes = []
    for k, (name, i, ind) in enumerate(starts):
        end = starts[k + 1][1] if k + 1 < len(starts) else len(lines)
        scopes.append((name, i, end))

    pinned_list = pinned_variants()
    buckets = {"pinned": [], "controller": [], "loop": [], "open": []}
    for name, a, b in scopes:
        body = lines[a:b]
        text = "\n".join(body)
        if not re.search(r"resolve_players?\(who", text):
            continue
        # An ask whose continuation argument is the bare `effect`.
        asked, seat_arg = None, ""
        for j, line in enumerate(body):
            if any(p in line for p in ASKS):
                window = "\n".join(body[j : j + 14])
                if re.search(r"^\s+effect,\s*$", window, re.M) or re.search(r"\beffect\)", window):
                    asked, seat_arg = a + j + 1, ask_seat_arg(body, j)
                    break
        if asked is None:
            continue
        # The binding the singular resolver wrote, if any.
        single = re.search(r"let Some\((?:mut )?([a-z_][a-z0-9_]*)\) = self\.resolve_player\(who", text)
        # The plural resolver means a loop over seats unless the arm takes one
        # seat off the front (`MayPayBy`'s `.first()`), whether the `for` reads
        # it inline or through a `let seats = …` above.
        in_loop = "resolve_players(who" in text and not re.search(
            r"resolve_players\(who[^\n]*\)\s*\.first\(\)", text
        )
        row = (name, a + 1, asked, seat_arg)
        if name in pinned_list:
            buckets["pinned"].append(row)
        elif in_loop:
            buckets["loop"].append(row)
        elif single is None or seat_arg != single.group(1):
            buckets["controller"].append(row)
        else:
            buckets["open"].append(row)

    for key, blurb in (
        ("open", "OPEN — a single-seat arm outside `with_asked_seat`'s list; add the arm"),
        ("loop", "loop over `resolve_players` — the LIST is re-derived; needs a seat-list ref, not this pin"),
        ("controller", "asks `ctx.controller`, which is resolution state and stable across a re-run — not this class"),
        ("pinned", "covered by `Effect::with_asked_seat`"),
    ):
        rows = buckets[key]
        print(f"{len(rows)} {key}: {blurb}")
        for name, arm_ln, ask_ln, seat in rows:
            print(f"    {name:44} arm at {arm_ln}, ask {seat!r} at {ask_ln}")
        print()

    print(
        f"{len(buckets['open'])} open / {len(buckets['loop'])} loop / "
        f"{len(buckets['controller'])} controller-asked / {len(buckets['pinned'])} pinned"
    )
    return 1 if buckets["open"] else 0


if __name__ == "__main__":
    sys.exit(main())
