#!/usr/bin/env python3
"""Does a shipped card's NAME belong to a real card?

Every other oracle-backed audit keys on the name, so a factory whose name
matches no oracle entry is audited by NONE of them — `audit_printed_body` drops
it as `nocache` and reads it as "a synthesized card", which most of them are.
This is that skip, split into its two populations: a name the oracle does not
know, in a file that is not one of the synthesized sets, is either a typo or an
invention, and both are worth knowing about.

A name is not cosmetic. It is what "cards named …" reads (Meddling Mage, the
legend rule, a `Search` by name), what the deck lists key on, and what
`server::vocab_snapshot` assigns an embedding index to — so a misspelling both
breaks those and hides the card from every column that would price it.

    python3 scripts/audit_card_names.py            # the table
    python3 scripts/audit_card_names.py --all      # include the synthesized sets

⚠ **THE NAME IS THE ONE `audit_printed_body` READS** — this calls its
`resolve_card_name`, it does not re-implement it. A local `name:` scan picks up
the nested `TokenDefinition { name: "Zombie" }` of every token-making card and
reports 400 rows of "Zombie", and a *copy* of the head-string heuristic
disagreed with the original the same day it was written, which is exactly the
class of bug both files exist to catch.

⚠ **A RENAME IS A VOCAB EDIT.** `crabomination/src/server/vocab_snapshot.rs` is
append-only by contract: the old name keeps its index for ever and the new one
is appended, so a rename is safe for a trained net's *indices* but hands it a
token it never saw. Batch renames, and say so in the commit.

COVERAGE, 2026-09-12: **17,461 named factories outside the expected sets — 0
spelling, 0 unknown, 0 duplicate**, with 6 reviewed unknowns and 3 reviewed
duplicate pairs. It opened at 2 / 12 / 5. Both misspellings hid a WRONG COST,
and both had a correct DUPLICATE of the same card shipped beside them, which is
the whole argument for the column — a name nobody audits is where a second,
worse copy of a card lives:

  * **Victims of Night** — the card is "Victim of Night", `{B}{B}`; the engine
    had `{1}{B}{B}`, and the cost column had never seen the card.
  * **Sabertooth Tiger** — "Sabretooth Tiger", `{2}{R}`; the engine had
    `{3}{R}`. And `recent77.rs` already shipped the card correctly, so the
    rename turned the row into a DUPLICATE row and the fix was a delete.
    `Victims of Night` was the same shape: `mod_set/instants.rs` had the real
    `victim_of_night()` forty lines below it.

A third came out of the same reading without a rename: **Surging Æther** keeps
the printed ligature the oracle de-ligatured in 2016, so it was `nocache` too —
`{2}{U}` where the card prints `{3}{U}`, and returning target *creature* where
it prints *permanent*. The `pub fn` slug resolves it now.

⚠ **`resolve_card_name` prefers the EXACT `pub fn` slug over a prefix guess**,
which is what brought those in: the head-string heuristic matches on a prefix,
so `fn sliver_queen` took the `"Sliver"` of its own token and `fn
goblin_marshal` took `"Goblin"`. 27 factories came back into every column.

EXPECTED, each with its reason in `SYNTHESIZED` / `REVIEWED` below: the
generated Strixhaven catalog, the Planechase planes (the oracle bulk carries 7
of hundreds), the Vanguard avatars (deliberately suffixed), and six invented
cards in real-set files.
"""
import argparse
import difflib
import importlib.util
import pathlib
import re
import sys
import unicodedata

ROOT = pathlib.Path(__file__).resolve().parent.parent

# Expected populations, with the reason each is one.
SYNTHESIZED = (
    # Generated (`scripts/gen_strixhaven2.py`, `STRIXHAVEN2.md`); most of these
    # cards do not exist, which is the point of the set.
    "sets/stx/",
    # Planechase PLANES. Scryfall's oracle bulk carries 7 of the hundreds
    # printed, so a plane missing from the cache says nothing about the card.
    "sets/ohop.rs",
    # A Vanguard avatar named after a card is deliberately suffixed " Avatar":
    # the oracle lookup cannot tell the two apart otherwise (Maraxus of Keld is
    # both), which is the same reason `audit_printed_body` drops them.
    "sets/vanguard.rs",
)

# TWO FACTORIES UNDER ONE NAME, reviewed once. Each pair is the same card
# implemented twice with different primitives, so picking the survivor is a
# body-by-body read rather than a delete — filed, not fixed. A NEW pair is the
# signal, and the two that are gone were found exactly this way: `Victim of
# Night` had a misspelled duplicate with a wrong cost forty lines above it, and
# `Sabretooth Tiger` had one whose name hid it from the oracle entirely.
REVIEWED_DUPLICATES = {
    "Kroxa, Titan of Death's Hunger": "modern.rs ships `kroxa` and "
                                      "`kroxa_titan_of_deaths_hunger`",
    "Uro, Titan of Nature's Wrath": "modern.rs ships `uro` and "
                                    "`uro_titan_of_natures_wrath`",
    "Niv-Mizzet, Parun": "modern.rs::niv_mizzet_parun and "
                         "recent91.rs::nivmizzet_parun",
}

# A name the oracle does not know, reviewed ONCE and kept so a NEW row is the
# signal — the same device as `audit_printed_body`'s `REVIEWED_KEYWORDS`.
REVIEWED = {
    "Putrefy (Modern)": "a deliberate second printing under a disambiguated name",
    "Cam and Farrik, Havoc Duo": "invented card in a deck file",
    "Sadistic Slash": "invented Mayhem card",
    "Sylvan Spellbomb": "invented spellbomb (the real land-fetching one is Horizon Spellbomb)",
    "Mistral Charge": "invented trick",
    "Top of the Class": "invented SOS enchantment",
}

_spec = importlib.util.spec_from_file_location(
    "audit_printed_body", pathlib.Path(__file__).resolve().parent / "audit_printed_body.py")
apb = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(apb)


def key(name: str) -> str:
    """A name reduced to what a typo cannot hide behind.

    Case, punctuation, accents and the Æ/Ae split all differ between the
    catalog and the oracle without the CARD differing — `Surging Æther` is
    `Surging Aether` in the bulk data — so those match here and are reported
    apart from a real miss.
    """
    n = unicodedata.normalize("NFKD", name.replace("Æ", "Ae").replace("æ", "ae"))
    return re.sub(r"[^a-z0-9]", "", n.lower())


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--all", action="store_true",
                    help="include the synthesized sets (sets/stx/)")
    args = ap.parse_args()

    real = {k: v for k, v in apb.CACHE.items()
            if isinstance(v, dict) and v.get("type_line")}
    by_key = {}
    for n in real:
        by_key.setdefault(key(n), n)
    # difflib over 35 k names per miss is minutes; bucket by the first three
    # letters of the key, which no plausible typo crosses.
    buckets = {}
    for k, n in by_key.items():
        buckets.setdefault(k[:3], []).append(n)

    rows, checked, reviewed = [], 0, 0
    seen = {}
    for path in sorted(apb.CATALOG.rglob("*.rs")):
        rel = path.relative_to(apb.CATALOG).as_posix()
        if not args.all and any(rel.startswith(s) for s in SYNTHESIZED):
            continue
        src = path.read_text()
        starts = [(m.start(), m.group(1)) for m in apb.FN.finditer(src)]
        for i, (pos, fname) in enumerate(starts):
            end = starts[i + 1][0] if i + 1 < len(starts) else len(src)
            raw = src[pos:end]
            nxt = apb.ANYFN.search(raw, raw.find("\n") + 1)
            if nxt:
                raw = raw[:nxt.start()]
            body = apb.top_fields(raw)
            if body is None:
                hm = re.match(r"\s*(?:[A-Za-z_]+::)*([a-z0-9_]+)\s*\(",
                              raw[raw.find("\n") + 1:])
                body = ".." + hm.group(1) + "(" if hm else None
            name = apb.resolve_card_name(body, raw, fname)
            if name is None or " // " in name:
                continue  # `noname` / `split`; not this column's populations
            checked += 1
            # ⚠ ONE NAME, TWO CARDS. The pool can hold both, "cards named …"
            # sees two, and the legend rule counts them together — and a
            # duplicate is where a wrong body hides, because the correct
            # sibling passes every column beside it.
            if name in seen and name not in REVIEWED_DUPLICATES:
                rows.append(("duplicate", name, f"{rel}::{fname}", seen[name]))
            seen.setdefault(name, f"{rel}::{fname}")
            if name in real:
                continue
            k = key(name)
            if k in by_key:
                rows.append(("spelling", name, f"{rel}::{fname}", by_key[k]))
                continue
            if name in REVIEWED:
                reviewed += 1
                continue
            near = difflib.get_close_matches(name, buckets.get(k[:3], []),
                                             n=1, cutoff=0.8)
            rows.append(("unknown", name, f"{rel}::{fname}",
                         near[0] if near else "(nothing close)"))

    for kind, name, where, near in sorted(rows):
        label = "also at" if kind == "duplicate" else "closest oracle name:"
        print(f"  {kind:9} {name!r}\n      {where}\n      {label} {near!r}")
    print(f"# {checked} named factories outside the expected sets — "
          f"**{sum(1 for r in rows if r[0] == 'spelling')} spelling, "
          f"{sum(1 for r in rows if r[0] == 'unknown')} unknown to the oracle, "
          f"{sum(1 for r in rows if r[0] == 'duplicate')} duplicate names** "
          f"({reviewed} reviewed unknown, {len(REVIEWED_DUPLICATES)} reviewed "
          f"duplicates)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
