#!/usr/bin/env python3
"""Which asking arms can leave `scratch.resolution_answer_log` behind.

The channel is per-resolution: an arm that asks more than one question suspends,
the seat's answer is appended, the arm re-runs and each ask replays its slot by
`cursor`. Every arm is supposed to `clear_answer_log()` once its last ask is in;
one that returns without doing so leaves its answers for the next arm to replay
(ENGINE_BACKLOG's tenth and eleventh finds).

`resolve_effect_into` now drops the channel at the outermost exit, so a leak
cannot cross into the next *resolution* — but inside one resolution (a `Seq` of
two asking arms, or an arm and the nested resolution it runs) the per-arm clear
is still the only guard. This is the static half of that check; the runtime half
is `CRAB_ANSWER_LOG=warn|strict` on a debug build, which names the card.

Per `let mut cursor = 0` block: the asks it makes, whether it clears, and the
completing paths that can leave the channel dirty —

  NO-CLEAR  the arm never clears at all (Menacing Ogre, Phyrexian Splicer)
  ERR?      a `?` propagates a `GameError` past an ask, which no arm clears on
  RET       some other early return past the ask with no clear before it
  PRE       a MUTATION before the arm's first ask — the re-run repeats it, so a
            suspending seat pays, flips or destroys twice
  MID       a MUTATION inside the LOOP that asks, which is the same defect one
            level in: each suspend repeats every earlier iteration's mutation
            (Fade Away charged the first seat once per later seat's suspend;
            `MayPayRepeatedly` re-pays quadratically; `CoinFlipDestroyLoop`
            re-rolls its coin). The fix is two passes — ask everyone, then
            mutate; ENGINE_BACKLOG has the recipe and the per-arm list

The suspend path (the `None` / `else` arm of an ask, a few lines below it) is
NOT a leak — the log is the resume's replay — and is filtered out, which is why
the runtime half is the authority: this one trades a false negative there for a
readable list. The two `ERR?` rows are listed, not fixed per arm — the net at the
resolution's exit is what makes an unwind harmless, for all 63 arms at once.

Reading at the eighteenth pass: **71 arms, 8 suspicious — 0 NO-CLEAR, 1 ERR?,
2 PRE, 8 MID.** It was 69 / 7 at the seventeenth (and 63 / 10 before that, with
11 MID before the worked example); the two arms and the row between the two
readings are `ExileUntilDuplicateName`, explained below.

Every remaining row is explained, which is the point of quoting the number:

  * four MIDs (`OtherPlayerMayPayToCounter`, `PlayersMayAccept`,
    `AnyPlayerMayExileFromGraveyard`, `AnyPlayerMayAccept`) are TAIL CALLS that
    `return` out of the loop, so nothing they mutate precedes a later ask;
  * `CoinFlipDestroyLoop`'s PRE and MID are terminal `break`s, and its flip and
    repeat cost are carried across the suspend instead (`flip_one_coin_logged`
    plus `answer_already_acted_on`) — the static walk cannot see either;
  * `TradeSecrets`' two MIDs are its draws, which are what the ask is ABOUT, so
    they cannot move after it; the arm skips the rounds its channel says are
    already performed;
  * `ExileUntilDuplicateName`'s PRE and MID are the same shape one card over
    (Tainted Pact): the exile IS what the ask is about, so it cannot move after
    it. Its "exiled this way" set is read back off the zone (`exiled_with =
    source`) rather than from a local `Vec`, and the round accounting is
    `TradeSecrets`' — one logged answer is one round already performed;
  * the one ERR? is an unwind, which the net at the resolution's exit makes
    harmless for all 69 arms at once.

So a NEW row is the signal, not the count.

**Injection (eighteenth pass):** deleting `MayDoBy`'s `clear_answer_log()`
takes this 8 suspicious -> **9**.

"""

import re
import sys

PATH = "crabomination/src/game/effects/mod.rs"
ASK = re.compile(
    r"\b(ask_seat_bool|ask_seat_amount|ask_seat_option|ask_seat_cards_logged"
    r"|ask_seat_cards|choose_up_to_cards|ask_ward_discards|ask_mana_sources)\b"
)
CLEAR = re.compile(r"\bclear_answer_log\(\)")
LOOP = re.compile(r"^\s*(for |while |loop\s*\{|'[a-z_]+: loop)")
# A re-run repeats everything the arm did before the ask it suspended on, so a
# mutation there happens again — `MayPayRepeatedly` pays its mana between asks and
# re-pays once per suspend. These are the engine's own mutators, matched on the
# call and not on the receiver, so a helper that mutates through one of them is
# caught too.
MUTATE = re.compile(
    r"\b(move_card_to|destroy_permanent|adjust_life|adjust_life_applied|pay_mana_cost"
    r"|pay_mana_cost_with_picks|shuffle_library|draw_cards|draw_one|mill_cards"
    r"|add_counters|remove_counters|send_to_graveyard|exile_card|sacrifice_permanent"
    r"|tap_permanent|untap_permanent|run_effect|create_token|ante_top_card)\s*\("
)
DECL = re.compile(r"^(\s*)let mut cursor = 0")
FN = re.compile(r"\s*(?:pub(?:\(crate\))?\s+)?(?:async\s+)?fn (\w+)")


ARM = re.compile(r"^\s*(?:\|\s*)?Effect::([A-Za-z0-9_]+)")
# The arms that reach an asking helper instead of asking inline; the helper name
# is not an `Effect` variant, so the walk up from its `cursor` stops at the `fn`.
HELPER_ARMS = {
    "run_destroy_each_unless_pays_life": ["DestroyEachUnlessPaysLife"],
    "run_discard_unless_put_on_top": [
        "DiscardUnlessPutCardOnTop",
        "SacrificeSourceUnlessSacrificeTotalPower",
    ],
    "run_each_unless_pays": ["ReturnEachUnlessPays", "SacrificeEachUnlessPays"],
    "run_look_top_may_bottom_all": ["LookTopMayBottomAllElse"],
    "run_pay_or_sacrifice_source": ["SacrificeSourceUnlessPay"],
    "run_separate_into_piles": ["SeparateIntoPiles"],
}


def log_arm_variants(lines):
    """Every `Effect` variant whose arm uses the answer log.

    `--variants` prints it as the Rust list that
    `core_rules::answer_log_nesting::no_shipped_card_nests_two_answer_log_arms`
    gates on: the log is one channel per resolution and `run_effect` recursion
    does not open a new one, so a card nesting two of these has its inner arm
    replay the outer's answers.
    """
    out = set()
    for i, l in enumerate(lines):
        if not DECL.match(l):
            continue
        for j in range(i - 1, max(0, i - 400), -1):
            m = ARM.match(lines[j])
            if m:
                out.add(m.group(1))
                break
            f = FN.match(lines[j])
            if f:
                out.update(HELPER_ARMS.get(f.group(1), []))
                break
    return sorted(out)


def enclosing_fn(lines):
    out, cur = {}, None
    for i, l in enumerate(lines):
        m = FN.match(l)
        if m:
            cur = m.group(1)
        out[i] = cur
    return out


def block_end(lines, start, indent):
    for j in range(start + 1, len(lines)):
        s = lines[j]
        if not s.strip():
            continue
        ind = len(s) - len(s.lstrip())
        if ind < indent:
            return j
    return len(lines)


def main():
    lines = open(PATH).read().split("\n")
    if "--variants" in sys.argv:
        names = log_arm_variants(lines)
        print(f"    // {len(names)} arms, from scripts/audit_answer_log.py --variants")
        for n in names:
            print(f'        "{n}",')
        return
    fn_at = enclosing_fn(lines)
    rows = []
    for i, l in enumerate(lines):
        m = DECL.match(l)
        if not m:
            continue
        indent = len(m.group(1))
        body = lines[i : block_end(lines, i, indent)]
        asks = [k for k, s in enumerate(body) if ASK.search(s)]
        clears = [k for k, s in enumerate(body) if CLEAR.search(s)]
        if not asks:
            rows.append((i + 1, fn_at[i], "NO-ASK", 0, 0, []))
            continue
        # Mutations BEFORE the first ask: the re-run repeats them.
        pre = [
            (i + 1 + k, body[k].strip()[:88])
            for k in range(asks[0])
            if MUTATE.search(body[k]) and not body[k].lstrip().startswith("//")
        ]
        risky = [(ln, "PRE   " + t) for ln, t in pre]
        # MID: the innermost loop enclosing the first ask, and any mutation in it
        # after that ask — every suspend re-runs the arm, so each earlier
        # iteration's mutation happens again.
        loop_at = None
        aind = len(body[asks[0]]) - len(body[asks[0]].lstrip())
        for j in range(asks[0] - 1, -1, -1):
            if LOOP.match(body[j]) and (len(body[j]) - len(body[j].lstrip())) < aind:
                loop_at = (j, len(body[j]) - len(body[j].lstrip()))
                break
        if loop_at is not None:
            j, li = loop_at
            stop = len(body)
            for k in range(j + 1, len(body)):
                t = body[k]
                if t.strip() and (len(t) - len(t.lstrip())) <= li:
                    stop = k
                    break
            risky += [
                (i + 1 + k, "MID   " + body[k].strip()[:88])
                for k in range(asks[0] + 1, stop)
                if MUTATE.search(body[k]) and not body[k].lstrip().startswith("//")
            ]
        for k in range(asks[0], len(body)):
            s = body[k]
            if k in asks:
                continue
            is_ret = re.search(r"\breturn\b", s) is not None
            is_try = re.search(r"\?;?\s*$", s.rstrip()) is not None
            if not (is_ret or is_try) or any(c <= k for c in clears):
                continue
            # An ask spans up to ~20 lines of arguments, so the window has to
            # reach back past them to see the `else {` / `None =>` that makes
            # this return the suspend path.
            near = [a for a in asks if 0 <= k - a <= 25]
            window = "\n".join(body[near[-1] : k + 1]) if near else ""
            if is_ret and near and re.search(r"None =>|else \{|let Some\(|suspend", window):
                continue  # the suspend path: the log IS the resume's replay
            risky.append((i + 1 + k, ("ERR? " if is_try else "RET  ") + s.strip()[:88]))
        rows.append((
            i + 1, fn_at[i], "ok" if clears else "NO-CLEAR", len(asks), len(clears), risky,
        ))

    asking = [r for r in rows if r[2] != "NO-ASK"]
    bad = [r for r in rows if r[2] == "NO-CLEAR" or r[5]]
    print(f"{len(rows)} cursor blocks, {len(asking)} with asks, {len(bad)} suspicious\n")
    for line, fn, status, nasks, nclears, risky in bad:
        print(f"{PATH}:{line}  fn {fn}  [{status}] asks={nasks} clears={nclears}")
        for rl, txt in risky[:6]:
            print(f"    ~{rl}: {txt}")


if __name__ == "__main__":
    main()
