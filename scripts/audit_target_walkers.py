#!/usr/bin/env python3
"""Which `Effect` wrappers each target walker forgets to recurse into.

`Effect::requires_target` is an exhaustive match, so the compiler makes it
name every wrapper — which is why it is also the ORACLE this audit checks the
cast-time slot walker against (see LATENT). Its five siblings in
`effect/query.rs` —
`primary_target_filter`, `prefers_graveyard_target`,
`may_target_offboard_card`, `accepts_player_target` and the cast-time slot
walk `target_filter_for_slot_in_mode_kicked` — end in `_ => …`, so a wrapper
they do not name answers the fallback silently.

⚠ **The sixth walker had no column here until a card fell through it.**
`target_filter_for_slot_in_mode_kicked` is the one the cast path asks "what may
slot N point at?", and wrapping Fractured Identity's token mint in
`Effect::EachPlayerDoes` — to fix a *different* bug — took its slot 0 out of the
cast prompt entirely. Five audited walkers and one unaudited one is how that
happens; see LATENT below for why its column is a surface report rather than a
`--check` failure.

**Read the fallback before reading a column as a bug list.** The three that
RESTRICT (`prefers_graveyard_target`, `may_target_offboard_card` -> `false`;
`primary_target_filter` -> `None`) now END in `Effect::for_each_inner`, the
one shared recursion, which `core_rules::target_walkers::the_shared_
recursion_names_every_effect_wrapper` holds it at every wrapper. **An unnamed
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
    "target_filter_for_slot_in_mode_kicked",
]

# The sixth walker's regime is the strictest one (`_ => None`, no
# `for_each_inner` deferral), so an unnamed wrapper means "a target slot
# declared inside this wrapper's body is not surfaced at cast time" — which is
# *correct* for the half of the list that chooses its targets later
# (`Reflexive`, `ReflexiveTrigger`, the `Whenever…ThisTurn` delayed triggers)
# and a latent gap for the rest.
#
# ✅ **That per-wrapper judgement does not have to be made here: it is already
# made, once, in `requires_target`.** An arm there that recurses into an inner
# `Effect` is the statement "a target in this body is chosen at cast time";
# the cast path then asks THIS walker for that slot's filter and
# `check_target_legality`'s fallback is restrictive, so a wrapper in that set
# and not in this walker is a declared slot the spell cannot fill. The audit
# derives the set (`descends_in_requires_target`) and counts only the
# disagreement, so `--check` gates the two walkers against each other. It was
# 22 of 50 when the check was added; naming those 22 took it to 0.
#
# ⚠ **The shipped gate for this walker is the catalog, not this list** —
# `core_rules::cr_rules::cr_601_2c_every_catalog_target_filter_is_surfaced`
# serializes every factory's effect, collects the `TargetFiltered` slots and
# demands each one come back out of this walker. It found `EachPlayerDoes` the
# hour Fractured Identity was wrapped in one. This column exists because the
# walker had no column at all while its five siblings each had one, and an
# unaudited walker is how a wrapper goes unnamed for a hundred passes.
LATENT = {"target_filter_for_slot_in_mode_kicked"}

# A walker whose match lives in a private inner fn, because its public half
# folds something else in. `accepts_player_target` ORs a player-only slot-0
# filter over the body classification (the 26 uncastable bodies), so the
# `_ => true` match this audit reads is `accepts_player_target_by_body`.
INNER = {
    "accepts_player_target": "accepts_player_target_by_body",
    # The cast-time slot walk is a private inner `fn` too: the public half
    # folds the kicker/mode arguments in before recursing.
    "target_filter_for_slot_in_mode_kicked": "eff_find",
}

# (walker, wrapper) pairs whose omission has been reviewed and is deliberate.
# Add a line here only with the reason; an unreviewed pair belongs in the
# report, not in this list.
ALLOWED: dict[str, set[str]] = {}

# The wrapper whose omission from the LATENT walker is *correct*: one whose
# body picks its own targets later (a reflexive or delayed trigger), so no
# cast-time slot is declared for it. The audit derives this rather than
# listing it — `requires_target` has already made the judgement for all 132,
# and a wrapper it descends into declares a cast-time target by definition.
# See `descends_in_requires_target`.


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


def _arm_rhs(body: str, variant: str) -> str | None:
    """The right-hand side of the `match` arm that names `variant`.

    Arms are grouped with `|`, so the pattern may be several lines above the
    `=>`. Cut at the next line that starts a new arm at the same indent or
    less — the file's formatting is regular enough for that, and the only
    question asked of the result is whether it recurses.
    """
    m = re.search(r"(?m)^(\s*)\|?\s*Effect::" + variant + r"\b", body)
    if not m:
        return None
    indent = len(m.group(1))
    arrow = body.find("=>", m.end())
    if arrow < 0:
        return None
    rest = body[arrow + 2 :].split("\n")
    out = []
    for line in rest:
        stripped = line.lstrip()
        if (
            out
            and stripped
            and len(line) - len(stripped) <= indent
            and (stripped.startswith(("Effect::", "|", "_ =>")))
        ):
            break
        out.append(line)
    return "\n".join(out)


def descends_in_requires_target(body: str, wrappers: list[str]) -> set[str]:
    """Wrappers whose `requires_target` arm recurses into an inner `Effect`.

    That recursion is the statement "a target inside this body is chosen at
    cast time". The cast path then asks the slot walker for that slot's
    filter, so a wrapper in this set that the slot walker does not name is a
    declared slot with no discoverable filter — and `check_target_legality`'s
    fallback is restrictive, so the spell rejects targets it should accept.
    """
    out = set()
    for w in wrappers:
        rhs = _arm_rhs(body, w)
        if rhs and ".requires_target()" in rhs:
            out.add(w)
    return out


def walker_bodies() -> dict[str, str]:
    src = QUERY.read_text()
    return {f: _brace_body(src, src.index("fn " + INNER.get(f, f) + "(")) for f in WALKERS}


def main() -> int:
    show_all = "--all" in sys.argv
    check = "--check" in sys.argv
    wrappers = wrapper_variants()
    bodies = walker_bodies()
    shared = _brace_body(QUERY.read_text(), QUERY.read_text().index("pub fn for_each_inner"))
    print(f"{len(wrappers)} `Effect` wrappers (a variant with an `Effect` in its fields)")
    gaps = 0
    for f in WALKERS:
        latent_missing: list[str] = []
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
            # ⚠ COUNTED, NOT QUOTED. This line used to read "(130/130)" as a
            # literal, so deleting an arm from `for_each_inner` left the report
            # unchanged — the one number that says the shared recursion is
            # still complete could not move. `core_rules::target_walkers::
            # the_shared_recursion_names_every_effect_wrapper` is the gate; this
            # is the same count, so the two cannot disagree in silence.
            named = sum(
                1 for w in wrappers if re.search(r"\bEffect::" + w + r"\b", shared)
            )
            regime, counted = (
                f"deferred to for_each_inner ({named}/{len(wrappers)}) — "
                + ("covered" if named == len(wrappers) else "⚠ THE SHARED RECURSION IS INCOMPLETE"),
                0 if named == len(wrappers) else len(wrappers) - named,
            )
        elif re.search(r"_ => true", bodies[f]):
            regime, counted = "fallback `true` — permitted, not a gap", 0
        elif f in LATENT:
            # Not every omission here is a gap — half the list chooses its
            # targets later and correctly declares no cast-time slot. The
            # judgement is already made, once, in `requires_target`: a
            # wrapper it recurses into carries a cast-time target, so a
            # wrapper in that set and not in this walker is an inconsistency
            # between two walkers that must agree. That is the counted half.
            descends = descends_in_requires_target(bodies["requires_target"], wrappers)
            inconsistent = sorted(set(missing) & descends)
            regime = (
                "fallback RESTRICTS — "
                + (
                    "consistent with `requires_target`"
                    if not inconsistent
                    else f"⚠ {len(inconsistent)} "
                    + ("DECLARES" if len(inconsistent) == 1 else "DECLARE")
                    + " A CAST-TIME TARGET `requires_target` RECURSES INTO "
                    "AND THIS WALK DOES NOT"
                )
                + "; the catalog gate is "
                "`cr_601_2c_every_catalog_target_filter_is_surfaced`"
            )
            counted = len(inconsistent)
            # Report BOTH numbers: "0 unnamed" would be a lie (28 wrappers
            # are unnamed and correctly so), and a bare "28 unnamed" is the
            # noise that hid the real 22 for a hundred passes.
            regime = f"{len(missing) - counted} of them correctly — " + regime
            latent_missing = inconsistent if inconsistent else ([] if not show_all else missing)
        else:
            regime, counted = "fallback RESTRICTS — these are gaps", len(missing)
        gaps += counted
        print(f"\n{f}: {len(missing)} unnamed of {len(wrappers)}  [{regime}]")
        listed = latent_missing if f in LATENT else missing
        if show_all or counted or f in LATENT:
            for w in listed:
                print(f"    {w}")
    if check and gaps:
        print(f"\n{gaps} unreviewed (walker, wrapper) pairs", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
