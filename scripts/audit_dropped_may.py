#!/usr/bin/env python3
"""Cards whose oracle text says "you may" and whose definition has no optional
primitive — the printed choice dropped, so the effect fires unconditionally.

A dropped "may" is not cosmetic. `Aura Shards` reads "whenever a creature
enters under your control, you **may** destroy target artifact or
enchantment"; without the may, a board where the only legal target is your
own permanent forces you to blow it up, because a trigger's targets are
mandatory once it triggers and the "may" was the only out.

    python3 scripts/audit_dropped_may.py            # ranked list
    python3 scripts/audit_dropped_may.py --count    # just the totals

⚠ A library search's "may" is NOT a finding — CR 701.19c lets a player decline
to find in a *hidden* zone, and every `Effect::Search` surfaces an `Option`
answer, so the printed choice is already there. A graveyard or exile search is
a public zone and keeps its finding. See `is_hidden_zone_search`.

Reads `scripts/.scryfall_cache.json` (offline; `scripts/fetch_oracle.py`
fills it). Cards whose name is not in the cache are **skipped and counted
separately** — the catalog carries thousands of synthesized `(b###)` names
with no oracle to be wrong against, and flagging those would bury the real
findings.

The optional-primitive list below is what the engine spells a printed choice
with. It is deliberately generous: a false negative here costs nothing, a
false positive costs a reader's time. Same for `ORACLE_SKIP`, the "you may"
phrasings that are keyword reminder text or a cast-time permission rather
than a resolution choice.
"""

import json
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CATALOG = os.path.join(ROOT, "crabomination_catalog", "src")
CACHE = os.path.join(ROOT, "scripts", ".scryfall_cache.json")

# Every way a definition can carry a printed choice.
OPTIONAL = (
    "MayDo", "MayDoBy", "MayDoElse", "MayPay", "MayTap", "MayDiscard",
    "MayReturn",
    "MayCast", "MayPayOrElse", "MayReveal", "MaySacrifice", "Optional",
    "ChooseMode", "ChooseN", "Escalate", "TapOrUntap", "may_", "_may",
    # Optionality the engine spells as a *shape* rather than a `May…` name.
    # Every one of these was a false positive on a card that was already
    # correct when the list was audited by hand (2026-08-30): Devouring Greed
    # and Plumb the Forbidden are `SacrificeAnyNumber`, Voltage Surge's
    # "you may sacrifice an artifact" is its `kicker_action_cost`, and a
    # "you may put … from among them" is a `LookPick` with `up_to: true` or a
    # `Selector::one_of` the picker may decline.
    "AnyNumber", "up_to", "kicker_action_cost", "one_of", "OneOf",
    "SacrificeOrPayLife", "min_targets: 0",
)

# Optionality the ENGINE spells in the RESOLVER rather than in the definition.
#
# The list above reads the definition's `{:?}` shape, so an effect whose
# printed "may" is modelled one level down — by the arm that resolves it —
# reads as a dropped one. `Effect::PutFromHandOntoBattlefield` is the whole of
# it at the `(-312)` tip (24 of 223 findings): it has no `up_to` field and no
# `May…` wrapper, and its arm opens with `choose_up_to_cards(.., 0, ..)` under
# a comment that says why ("Always optional (\"you may\"): min 0"). The
# decision exists with a minimum of zero, so declining is available and the
# word is modelled.
#
# ⚠ **The exemption is conditioned on the ENGINE and re-checked on every run**
# — the same discipline as the CR 701.19c search carve-out below, which is
# conditioned on the definition actually carrying a search. If the arm stops
# calling its helper, `resolver_optional()` refuses the exemption and the
# findings come back rather than staying hidden.
ENGINE_EFFECTS = os.path.join(
    ROOT, "crabomination", "src", "game", "effects", "mod.rs"
)
RESOLVER_OPTIONAL = {
    # Effect variant: (the oracle phrase it covers, the helper that makes the
    # arm optional)
    "PutFromHandOntoBattlefield": ("you may put", "choose_up_to_cards"),
    # "You may put any number of … from among them into your hand" (Nashi,
    # Gather the Pack, Smuggler's Surprise). The arm builds its
    # `Decision::ChooseCards` with `min: 0`, so taking none is available and
    # is what makes Nashi's "if you put no cards into your hand this way"
    # branch reachable at all.
    "MillThenToHandN": ("you may put", "min: 0"),
}

# ⚠ **`LookPickToHand` IS THE NEXT ONE AND IT IS NOT SAFE AT THIS GRANULARITY
# — measured, not assumed (2026-09-14).** Its `take == 1` path builds a
# `Decision::SearchLibrary`, whose answer is an `Option` with a `None`
# headless default, so declining is available whatever `optional` says — the
# same CR 701.19c argument as the search carve-out below, and it would clear
# 11 findings. The `take > 1` path is the opposite: `min: if *optional { 0 }
# else { take }` forces the picks, so those keep their finding.
#
# What stops it is the PHRASE, not the effect: "you may put" on those same
# cards also matches the *rest* disposition rather than the pick — Bucolic
# Ranch's "you may put it on the bottom of your library" and Break Out's "you
# may put it onto the battlefield and it gains haste", both real choices the
# pick's optionality says nothing about. A prefix match would hide them.
# Taking this exemption needs the span matched to the pick ("from among them",
# "from among the … cards") rather than to the effect variant.


def resolver_optional():
    """`{variant: phrase}` for each exemption the engine still earns.

    Reads the arm out of `effects/mod.rs` by brace matching from
    `Effect::<Variant>` and keeps the exemption only while the named helper is
    in it. A variant whose arm cannot be found, or no longer calls the helper,
    is dropped from the map with a note on stderr — the findings it was
    hiding come back on the next line of output.
    """
    src = open(ENGINE_EFFECTS, encoding="utf-8").read()
    out = {}
    for variant, (phrase, helper) in RESOLVER_OPTIONAL.items():
        m = re.search(r"Effect::%s\s*[{(]" % re.escape(variant), src)
        if not m:
            print(f"# exemption REFUSED: no `Effect::{variant}` arm", file=sys.stderr)
            continue
        i, depth = m.end() - 1, 0
        opens, closes = ("{", "}") if src[i] == "{" else ("(", ")")
        while i < len(src):
            if src[i] == opens:
                depth += 1
            elif src[i] == closes:
                depth -= 1
                if depth == 0:
                    break
            i += 1
        # The arm body runs from the pattern's close to the next `Effect::`
        # at the same level; a window is enough for a containment test and
        # cannot reach the next variant's helper by accident at this size.
        arm = src[i : i + 4000]
        if helper not in arm:
            print(
                f"# exemption REFUSED: `Effect::{variant}`'s arm no longer calls "
                f"`{helper}`",
                file=sys.stderr,
            )
            continue
        out[variant] = phrase
    return out


# "you may" phrasings that are not a resolution choice the effect tree owns.
ORACLE_SKIP = (
    "you may cast",            # alternate-cost / from-exile permissions
    "you may play",
    "you may pay",             # MayPay, and also ward/kicker reminder text
    "you may put this card",   # suspend / foretell reminder text
    "you may look at",         # reminder text on scry-likes
    "you may exile it from",   # flashback-ish reminder text
    "you may have",            # copy-target reminder ("you may choose new targets")
    "you may choose new targets",
    "you may reveal",          # most reveals are engine-implicit
    "as you may",
    # CR 103.6 — a Leyline's "you may begin the game with it on the
    # battlefield" is a *pre-game* permission, not a resolution choice the
    # effect tree owns, and no `May…` wrapper could carry it.
    "you may begin the game",
    # CR 106.6 — "you may spend mana as though it were mana of any colour to
    # cast/activate …" is a permission on *how you pay*, announced with the
    # payment, not a choice the resolving effect owns.
    "you may spend mana",
    # "You may activate one of its loyalty abilities …" (The Chain Veil) is a
    # **permission** granted for the turn, exercised by taking an action
    # later, not a choice the resolving effect owns.
    "you may activate",
    # Conspiracy draft-time text ("immediately after the draft, you may …")
    # describes a step this engine has no zone for at all.
    "immediately after the draft",
    # CR 508.1g / 701.43d — "you may exert this creature as it attacks" is an
    # **optional cost to attack**, announced as attackers are declared
    # (`GameAction::DeclareAttackersExerting`, or the engine's policy when the
    # declaration is silent). The effect tree carries only the *linked* "when
    # you do", which rides `EventKind::Exerted` and so cannot fire without it.
    "you may exert",
)

# A "you may … rather than pay this spell's mana cost" is an alternative cost
# (`AlternativeCost` / `AdditionalCastCost`), announced at cast time, not a
# resolution choice the effect tree owns. Flare of Malice and Fireblast are
# implemented, not flattened.
ORACLE_SKIP_CONTAINS = ("rather than pay",)

# CR 701.19c — **a library search's printed "may" is already modeled, by the
# rules rather than by a primitive.** "If a player is searching a hidden zone
# for cards with stated qualities… that player isn't required to find some or
# all of those cards." Every `Effect::Search` surfaces
# `Decision::SearchLibrary`, whose answer is an `Option` and whose headless
# default is `Search(None)`, so declining is available on a *mandatory* search
# too — which means "you may search your library" and "search your library"
# resolve identically and the dropped "may" changes nothing.
#
# It was 56 of 309 findings, the single largest bucket, and every one a false
# positive. ⚠ **The exemption is the hidden zone, not the word "search"**: a
# graveyard or exile search is a public zone, where a player MUST find if able,
# so "you may search your library **and/or graveyard**" keeps its finding.
SEARCH_MAY = re.compile(r"^you may search (your|target player's|a player's) librar")
PUBLIC_ZONE = ("graveyard", "exile", "battlefield")


# The exemption is only sound if the definition actually *searches* — a card
# that dropped the search entirely has a different defect (a missing effect,
# which `audit_incomplete` owns) and must not be hidden here.
SEARCH_PRIMITIVE = ("Search", "search_")


def is_hidden_zone_search(span: str, body: str) -> bool:
    """A "you may search" that reaches only a hidden zone (CR 701.19c), on a
    definition that does carry a search."""
    if not SEARCH_MAY.match(span):
        return False
    head = span.split(" for ", 1)[0]
    if any(z in head for z in PUBLIC_ZONE):
        return False
    return any(tok in body for tok in SEARCH_PRIMITIVE)


# `..creature("Name", …)` — a struct-update base, the card's own helper.
BASE_NAME = re.compile(r'\.\.\s*\w+\(\s*\n?\s*"([^"]+)"')
def call_names(body):
    """Names from `ident("Name", …)` calls at the body's **own** depth.

    `fated("Fated Conflagration", …)` is the whole body; `spell("Will of the
    Sultai", …)` is the tail expression after a `let`. Both are the card's own
    helper. Depth is what separates them from the **token** a card mints,
    which is always nested inside a struct literal or another call — an
    unanchored regex took Conqueror's Pledge for "Kor Soldier" and Urza's Saga
    for "Construct", and an anchored one missed every body with a `let` in
    front of the call (126 factories).
    """
    out, depth, i = [], 0, 0
    while i < len(body):
        c = body[i]
        if c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif depth == 1 and (c.isalpha() or c == "_"):
            ident = re.match(r"[A-Za-z_]\w*", body[i:]).group(0)
            j = i + len(ident)
            m = re.match(r'\s*\(\s*\n?\s*"([^"]+)"', body[j:])
            # `.method("…")` and `field: name("…")` are not the card's helper.
            if m and not body[:i].rstrip().endswith((".", ":")):
                out.append(m.group(1))
            i = j
            continue
        i += 1
    return out


def slug(name):
    """"Pestilent Cauldron" -> "pestilent_cauldron", the factory-name shape.

    The apostrophe is **dropped**, not separated: this catalog writes
    Acolyte's Reward as `acolytes_reward`, so turning `\'` into `_` made the
    slug preference miss on all 1,162 possessive names and fall back to
    whichever literal came first.
    """
    return re.sub(r"[^a-z0-9]+", "_", name.lower().replace("'", "").replace("\u2019", "")).strip("_")


def _brace_body(src, open_at):
    """The `{…}` block starting at `open_at`, by brace matching."""
    depth, i = 0, open_at
    while i < len(src):
        if src[i] == "{":
            depth += 1
        elif src[i] == "}":
            depth -= 1
            if depth == 0:
                return src[open_at : i + 1]
        i += 1
    return src[open_at:]


# A card factory that builds its ability through a same-file helper carries
# none of the helper's shape in its own body, so a `min_targets: 0` or a
# `MayDo` one call down reads as a dropped "may". The Primordial cycle was the
# whole of it when this was written: `primordial_etb` wraps every one of the
# five in `ApplyToTargets { min_targets: 0, .. }` and all five were findings.
# Inline every same-file `fn` the body names, twice, so a helper that calls a
# helper (the cycle bodies in `decks/lands.rs`) resolves too.
HELPER_DEPTH = 2


def helper_bodies(src):
    """`({helper: body}, {factory: body})` for the file's `fn`s.

    The two maps are inlined on different triggers: a helper on any call, a
    factory only in struct-update position (`..myr_retriever()`), which is how
    this catalog spells "this card is that one with three fields changed"
    (Junk Diver / Myr Retriever). A factory name can also appear as an
    ordinary call — a token's definition, a land cycle's base — and inlining
    it there would hide a finding rather than clear one.
    """
    helpers, factories = {}, {}
    for m in re.finditer(r"\bfn (\w+)\s*(?:<[^>]*>)?\s*\(", src):
        i = src.find("{", m.end())
        if i < 0:
            continue
        target = (
            factories
            if re.match(r"pub fn \w+\(\) -> CardDefinition \{", src[m.start() - 4 : i + 1])
            else helpers
        )
        target[m.group(1)] = _brace_body(src, i)
    return helpers, factories


def with_helpers(body, helpers, factories):
    """`body` plus the same-file bodies it builds itself out of."""
    seen, frontier, parts = set(), [body], [body]
    for _ in range(HELPER_DEPTH):
        nxt = []
        for chunk in frontier:
            calls = set(re.findall(r"\b(\w+)\s*\(", chunk))
            bases = set(re.findall(r"\.\.\s*(\w+)\s*\(\s*\)", chunk))
            for call in calls | bases:
                if call in seen:
                    continue
                body_of = helpers.get(call) or (factories.get(call) if call in bases else None)
                if body_of is None:
                    continue
                seen.add(call)
                nxt.append(body_of)
        if not nxt:
            break
        parts.extend(nxt)
        frontier = nxt
    return "\n".join(parts)


def defs_in(path):
    """(fn_name, card_name, body, path) for every `pub fn … -> CardDefinition`.

    A factory body often holds **several** `name:` literals — a transform back
    face, a token it mints, a `let` for the other half of an MDFC — and the
    first one is not reliably the card's. Taking it blind is how a name-keyed
    audit ends up scoring `pestilent_cauldron` against Restorative Burst's
    oracle. Prefer the literal whose slug matches the function name, which is
    this catalog's naming convention; fall back to the first only when none
    does, and say so by yielding it anyway (the caller's cache lookup is the
    second filter).

    ⚠ A third of this catalog names the card **positionally**, not with a
    `name:` field — `..creature("Sidar Jabari", cost(…), …)` as a struct
    update, or `fated("Fated Conflagration", …)` as the whole body. Keying
    only on `name:` skipped **7,030 of 22,134 factories**, and every ratchet
    built on this reader was reporting its column over two thirds of the
    catalog. `BASE_NAME` and `CALL_NAME` are those two shapes; the
    struct-update one is the most reliable of the three, because `..helper()`
    is by construction the card's own base rather than a token it mints.
    """
    src = open(path, encoding="utf-8").read()
    helpers, factories = helper_bodies(src)
    for m in re.finditer(r"pub fn (\w+)\(\) -> CardDefinition \{", src):
        start = m.end() - 1
        body = _brace_body(src, start)
        fn = m.group(1)
        field = re.findall(r'name:\s*"([^"]+)"', body)
        base = BASE_NAME.findall(body)
        call = call_names(body)
        own = next((n for n in field + base + call if slug(n) == fn), None)
        name = own or (base[:1] or field[:1] or call[:1] or [None])[0]
        if name is None:
            continue
        yield fn, name, with_helpers(body, helpers, factories), path


def main():
    cache = json.load(open(CACHE, encoding="utf-8"))
    # A few entries in the cache are bare strings (a negative-lookup marker
    # from an older fetcher); they carry no oracle text and are not findings.
    lower = {k.lower(): v for k, v in cache.items() if isinstance(v, dict)}
    for k, v in list(cache.items()):
        if isinstance(v, dict):
            lower.setdefault(k.split(" // ")[0].lower(), v)

    resolver_exempt = resolver_optional()
    hits, uncached, checked, exempted = [], 0, 0, 0
    for dirpath, _, files in os.walk(CATALOG):
        for f in files:
            if not f.endswith(".rs"):
                continue
            for fn, name, body, path in defs_in(os.path.join(dirpath, f)):
                card = lower.get(name.lower())
                if card is None:
                    uncached += 1
                    continue
                checked += 1
                # Reminder text is where most "you may" phrasings live —
                # Modular's "you may put its +1/+1 counters on…", Exert,
                # Casualty, Collect evidence. The keyword itself is what the
                # definition implements, so a parenthesised span is never the
                # finding. Scryfall puts reminder text in parentheses.
                oracle = re.sub(r"\([^)]*\)", " ", card.get("oracle_text") or "").lower()
                if "you may" not in oracle:
                    continue
                spans = [
                    s for s in re.findall(r"you may[^.;\n]*", oracle)
                    if not any(s.startswith(p) for p in ORACLE_SKIP)
                    and not any(p in s for p in ORACLE_SKIP_CONTAINS)
                    and not is_hidden_zone_search(s, body)
                ]
                if not spans:
                    continue
                if any(tok in body for tok in OPTIONAL):
                    continue
                # …and the optionality the resolver owns rather than the
                # definition. Matched on the phrase as well as the variant so
                # a card carrying one of these plus a *different* dropped
                # "may" keeps its finding.
                if any(
                    f"Effect::{v}" in body and any(s.startswith(phrase) for s in spans)
                    for v, phrase in resolver_exempt.items()
                ):
                    exempted += 1
                    continue
                hits.append((name, fn, os.path.relpath(path, ROOT), spans[0][:80]))

    hits.sort()
    if "--count" not in sys.argv:
        for name, fn, path, span in hits:
            print(f"{path}::{fn}\n    {name}: “{span}”")
    print(
        f"# {len(hits)} definitions with a dropped 'you may', "
        f"{checked} checked against the oracle cache, {uncached} skipped as uncached "
        f"(synthesized names have no oracle), {exempted} exempt because the "
        f"RESOLVER carries the choice ({', '.join(sorted(resolver_exempt)) or 'none'})"
    )


if __name__ == "__main__":
    main()
