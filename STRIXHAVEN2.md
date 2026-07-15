# Strixhaven implementation tracker

Two catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`) and
**Strixhaven: School of Mages (STX)** (`catalog::sets::stx`). The STX base set
is 100% implemented; the **Mystical Archive (STA)** companion lives in
`catalog::sets::stx::sta` and is complete: Infuriate, Blue Sun's Zenith,
Abundant Harvest, Urza's Rage, Natural Order, Tainted Pact
(`Effect::ExileUntilDuplicateName`), Mizzix's Mastery (exile-I/S-from-gy +
free-cast + overload via `effect_override`) — tests in `tests/stx/part_31.rs`.

Learn / Lessons-sideboard is wired end-to-end (engine, every Learn card,
cube/format/draft sideboards, client modal).

**Prepare mechanic (reworked June 2026):** the 36 preparation cards were
originally (wrongly) modeled as hand-castable MDFCs. They now use
`CardDefinition.prepare_spell` + `GameAction::CastPrepareSpell` — you cast a
*copy* off the prepared battlefield creature, which unprepares it. Fronts are
audited against Scryfall oracle. See `.claude/prepared.md` for the mechanic
and TODO.md → "Prepare Mechanic (SOS)" for per-card approximations.

Kasmina, Enigma Sage is fully faithful: loyalty 2, the "other planeswalkers
share Kasmina's loyalty abilities" static
(`StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities`), and the real -8.

## Status caveats (2026-06-14 audit)

"No partials remain" is true only in the narrow tracker sense (no card is marked
🟡/⏳ in the status tables). In practice:

- **Documented approximations remain** (as of 2026-07-12, a case-insensitive
  grep for `Approximation` / `omitted` / `dropped` matches 51 lines in the SOS
  catalog and 44 in STX). Most capture
  the headline play pattern. Possibility Storm (cast-trigger exile-and-dig via
  `Effect::PossibilityStorm`) and Hindering Light (target-restricted counter)
  are now fully wired. Arclight Phoenix (gy-return on 3+ I/S spells) and
  Detention Sphere (exile-until-it-leaves) are now fully wired. Mavinda,
  Students' Advocate is now ✅ (gy-cast modeled as a once-per-turn {2} ability
  granting a pay-own-cost, exile-after may-play — total = spell cost + {2},
  matching the printed surcharge; only the "targets a single creature"
  sub-filter is approximated).
- **Stats match real Scryfall.** `audit_stx_drift.py` (cost + P/T) and the new
  `audit_stx_types.py` (type line + keywords) are both clean. The 2026-06-14/15
  sweep fixed **47 creature types + 20 keywords** (e.g. Mavinda Cleric+Vigilance →
  Bird Advisor+Flying; Beledros Demon → Elder Dragon; Disciplined Duelist →
  Double strike; Inkfathom Witch → Fear), full suite green (8551). Remaining
  flags are non-bugs: 3 types (synthesized cards coupled to Pest/Fractal/Inkling
  *synergy tests*) and 1 keyword (Lone Rider — a benign DFC back-face Trample
  artifact; the modeled front face is correct). Conditional/granted keywords the
  catalog models via
  statics (Leech Fanatic, Sticky Fingers, …) are correct and no longer flagged.
  Caveat: many fixed cards are fabricated-real-name collisions whose **bodies are
  still synthesized** — correct stats ≠ faithful card. See TODO.md → "Fabricated
  real-name STX cards".

Full per-card history: `git log -- crabomination_catalog/src/sets/{stx,sos}/`.

## 2026-07-14 full-mechanics audit (approximations = incorrect)

An 8-agent sweep audited all 256 SOS card functions clause-by-clause against
their doc-comment oracle text (~100 cards had at least one deviation:
3 false engine claims in doc comments, ~38 undocumented drifts, ~60 documented
approximations). Fixed in the same pass:

**Engine-level**
- `Effect::Search` now shuffles the searched library (CR 701.19c), including
  declines; library destinations (Goblin Recruiter) are exempt because the
  chained multi-pick search would bury earlier picks. Regression:
  `cr_701_19c_search_shuffles_library_even_on_decline`.
- "One or more cards leave your graveyard" triggers now fire once per
  simultaneous batch (dedup at trigger dispatch; emission stays per-card for
  the tally + client mirror). Fixes Ark of Hunger, Owlin Historian, Hardened
  Academic, Spirit Mascot, Garrison Excavator, Kirol, and tarkir's Attuned
  Hunter.
- "One or more creatures you control deal combat damage to a player"
  graveyard triggers (Killian's Confidence) fire once per damage sub-step,
  not once per attacker.
- Triggered abilities that DECLARE a target slot now emit `BecameTarget`
  (cast + activated paths already did) — Tenured Concocter fires on
  opponents' triggered abilities. Effects whose "target" slot is an
  event-bound subject (Horobi's "destroy it") correctly do not.
- `PlayerRef::ControllerOf` on a dead permanent resolves via death-time LKI
  (CR 608.2h) instead of falling back to owner — Erode, Foolish Fate, and
  Harsh Annotation now hit the right player for stolen creatures.
- New `StaticEffect::GrantStormToISSpells`: Prismari, the Inspiration grants
  REAL storm at cast time (copy count can't be inflated by responses).
- New `CardDefinition::self_cost_reduction_cost_if_target` (colored-aware
  mandatory reduction via `reduce_by_cost`).
- New `Predicate::FirstLifeGainThisTurn` + per-turn flag: "gain life for the
  first time each turn" (Leech Collector) keys on the turn's actual first
  gain, not the listener's first observed gain.
- `EntersTappedUnless` now reads the cast's stamped X (Slumbering Trudge).

**Per-card** (previously undocumented drift, now faithful): school taplands
(EntersTapped replacement + stray basic land types dropped), 7 intervening-
'if' cards moved to trigger-time `event.filter` (Living History, Primary
Research, Joined Researchers, Emeritus of Woe, Scheming Silvertongue,
Emeritus of Abundance, Scolding Administrator), Quandrix the Proof (real
`Effect::Cascade` — declined hit is bottomed, cap derived not hardcoded),
Magmablood Archaic + Slumbering Trudge (true enters-with replacements),
Pox Plague (clause-by-clause across players), Emeritus of Conflict (Bolt =
`target_any()`), Brush Off / Run Behind / Ajani's Response / Wilt in the
Heat / Orysa (mandatory reductions, alt-costs deleted), Environmental
Scientist ("may search" honored), Moseo / Ascendant Dustspeaker / Startled
Relic Sloth ("up to one target" declinable via `ApplyToTargets min 0`),
Paradox Surveyor (`LookPickToHand` — sees all five, rest bottomed).

**Still open** (tracked, not fixed): `LookPickToHand` declines auto-fill
under the AutoDecider; the miracle window is step-bounded rather than
trigger-resolution-exact (the caster may act within the granting step
before deciding); plus the in-source documented approximations (grep
`Approximation|omitted|dropped`).

2026-07-15 cont.: `RevealMissDest::BottomRandom` genuinely shuffles the
miss batch before bottoming; Aziza's tap-three is a real all-or-nothing
cost with a `ChooseCards` pick of which creatures tap
(`TapUpToValue { exact: true }` behind a three-untapped `If` gate).

2026-07-15 follow-ups: Tam + Stone Docent use the real Gorgon/Chimera
types; "pay X life" is a true cast-time additional cost
(`additional_cost_pay_x_life` — Vicious Rivalry, Fix What's Broken); the
five copy cards with printed "you may choose new targets" (Lumaret's
Favor, Choreographed Sparks mode 0, Aziza, Mica, Silverquill's casualty)
now use `Effect::CopySpellMayChooseTargets`; and true cast-time
multi-mode selection ships via `Effect::ChooseModesCast` (shares
`CastSpellSpree` plumbing, per-instance target slots) — Moment of
Reckoning ("up to four, repeats allowed") and Choreographed Sparks
("one or both") are fully faithful. Miracle (intrinsic + Lorehold's
grant) is now a real step-bounded, timing-exempt window
(`MayPlayDuration::EndOfThisStep` + `MayPlayPermission.miracle`) instead
of an until-EOT discount.

## Removed status tables (2026-07-12)

The per-card status tables formerly in this file were removed. The three
scripts `scripts/audit_strixhaven2.py`, `scripts/list_sos_ok.py`, and
`scripts/sos_ok_factory_map.py` parsed those tables and are retained only as
historical artifacts. A 2026-07-12 audit fixed 15 gameplay bugs and backfilled
the sos_mode pools (elders Quandrix the Proof + Silverquill the Disputant and
all other implemented-but-unpooled cards).
