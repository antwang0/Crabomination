#!/usr/bin/env python3
"""Which `Effect` wrappers each target walker forgets to recurse into.

`Effect::requires_target` is an exhaustive match, so the compiler makes it
name every wrapper. Its four siblings in `effect/query.rs` —
`primary_target_filter`, `prefers_graveyard_target`,
`may_target_offboard_card`, `accepts_player_target` — end in `_ => …`, so a
wrapper they do not name answers the fallback silently.

**Read the fallback before reading a column as a bug list.** The three that
RESTRICT (`prefers_graveyard_target`, `may_target_offboard_card` -> `false`;
`primary_target_filter` -> `None`) now END in `Effect::for_each_inner`, the
one shared recursion, which `core_rules::target_walkers::the_shared_
recursion_names_every_effect_wrapper` holds at 130 of 130. **An unnamed
wrapper there is COVERED, not lost** — it is handled generically instead of
by an arm of its own, which is the whole point of the hundredth pass's fix.
`accepts_player_target` does not defer and does not need to: its fallback is
`_ => true`, a deliberate conservative default (the legality gate still
rejects a mismatch), so its unnamed wrappers are permitted.

So the "unnamed" column is no longer a defect census for any of the four. It
is now a *shape* report: which wrappers each walker treats specially rather
than generically. The report says which regime each walker is in. That is the drift the
eighty-sixth pass found the hard way: a `Move` out of a graveyard wrapped in
something the off-board gate did not name resolves targetless in the training
path, where nothing watches.

    python3 scripts/audit_target_walkers.py            # gaps only
    python3 scripts/audit_target_walkers.py --all      # every wrapper, per walker
    python3 scripts/audit_target_walkers.py --check    # exit 1 if a gap is not allowed

A *wrapper* is an `Effect` variant with an `Effect` somewhere in its fields.
"Names it" is a substring test for `Effect::<Variant>` in the walker's body:
naming a wrapper in an explicit `false` arm counts, because the point is that
the decision was made rather than inherited from the fallback.

ALLOWED below is the reviewed set — a wrapper whose body genuinely cannot
carry the thing that walker looks for. Everything else the audit prints is a
question nobody has answered.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EFFECT = ROOT / "crabomination_base" / "src" / "effect.rs"
QUERY = ROOT / "crabomination_base" / "src" / "effect" / "query.rs"

WALKERS = [
    "requires_target",
    "primary_target_filter",
    "prefers_graveyard_target",
    "may_target_offboard_card",
    "accepts_player_target",
]

# (walker, wrapper) pairs whose omission has been reviewed and is deliberate.
# Add a line here only with the reason; an unreviewed pair belongs in the
# report, not in this list.
ALLOWED: dict[str, set[str]] = {}


def _brace_body(src: str, start: int) -> str:
    """The `{ … }` block starting at or after `start`, brace-matched."""
    i = src.index("{", start)
    depth = 0
    for j in range(i, len(src)):
        if src[j] == "{":
            depth += 1
        elif src[j] == "}":
            depth -= 1
            if depth == 0:
                return src[i : j + 1]
    raise ValueError("unbalanced braces")


def _strip_comments(s: str) -> str:
    return re.sub(r"(?m)^\s*(///|//!|//|#\[).*$", "", s)


def wrapper_variants() -> list[str]:
    src = EFFECT.read_text()
    body = _brace_body(src, re.search(r"\npub enum Effect \{", src).start())[1:-1]
    out, depth, cur = [], 0, ""
    for ch in body:
        if ch in "{([":
            depth += 1
        elif ch in "})]":
            depth -= 1
        cur += ch
        if ch == "," and depth == 0:
            out.append(cur)
            cur = ""
    if cur.strip():
        out.append(cur)
    wrappers = []
    for v in out:
        clean = _strip_comments(v)
        name = re.search(r"\b([A-Z]\w*)\s*[\{\(,]", clean)
        if name and re.search(r"\bEffect\b", clean):
            wrappers.append(name.group(1))
    return wrappers


def walker_bodies() -> dict[str, str]:
    src = QUERY.read_text()
    return {f: _brace_body(src, src.index("pub fn " + f)) for f in WALKERS}


def main() -> int:
    show_all = "--all" in sys.argv
    check = "--check" in sys.argv
    wrappers = wrapper_variants()
    bodies = walker_bodies()
    print(f"{len(wrappers)} `Effect` wrappers (a variant with an `Effect` in its fields)")
    gaps = 0
    for f in WALKERS:
        allowed = ALLOWED.get(f, set())
        missing = [
            w
            for w in wrappers
            if not re.search(r"\bEffect::" + w + r"\b", bodies[f]) and w not in allowed
        ]
        # How the walker treats a wrapper it does not name. Only the first
        # regime is a defect census; the report used to print all three the
        # same way, which is what made 100-odd covered wrappers read as gaps.
        if not missing:
            # `requires_target`'s outer `match self` has no `_` arm, so the
            # compiler names every wrapper for it. Checked this way rather
            # than by looking for `_ =>`: its three nested helpers each end
            # in one, and that mislabelled it as a restricting fallback.
            regime, counted = "every wrapper named — exhaustive", 0
        elif "for_each_inner" in bodies[f]:
            regime, counted = "deferred to for_each_inner (130/130) — covered", 0
        elif re.search(r"_ => true", bodies[f]):
            regime, counted = "fallback `true` — permitted, not a gap", 0
        else:
            regime, counted = "fallback RESTRICTS — these are gaps", len(missing)
        gaps += counted
        print(f"\n{f}: {len(missing)} unnamed of {len(wrappers)}  [{regime}]")
        if show_all or counted:
            for w in missing:
                print(f"    {w}")
    if check and gaps:
        print(f"\n{gaps} unreviewed (walker, wrapper) pairs", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
