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

The suspend path (the `None` / `else` arm of an ask, a few lines below it) is
NOT a leak — the log is the resume's replay — and is filtered out, which is why
the runtime half is the authority: this one trades a false negative there for a
readable list. The two `ERR?` rows are listed, not fixed per arm — the net at the
resolution's exit is what makes an unwind harmless, for all 63 arms at once.

Reading at the eleventh find: **63 arms, 0 NO-CLEAR, 2 ERR?.**
"""

import re

PATH = "crabomination/src/game/effects/mod.rs"
ASK = re.compile(
    r"\b(ask_seat_bool|ask_seat_amount|ask_seat_option|ask_seat_cards_logged"
    r"|ask_seat_cards|choose_up_to_cards|ask_ward_discards|ask_mana_sources)\b"
)
CLEAR = re.compile(r"\bclear_answer_log\(\)")
DECL = re.compile(r"^(\s*)let mut cursor = 0")
FN = re.compile(r"\s*(?:pub(?:\(crate\))?\s+)?(?:async\s+)?fn (\w+)")


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
        risky = []
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
