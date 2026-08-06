# Strixhaven implementation tracker

> The SOS **Special Guests (SPG)** sheet ships: `sos_mode::sos_special_guests`
> names all eleven, and `generate_sos_pack` gives it its own slot at the
> printed 1-in-64 rate. The vocab grew, so trained value nets need a retrain
> — see TODO.md → "SOS Special Guests (SPG)".

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

- **Documented approximations remain** (as of 2026-08-01, a case-insensitive
  grep for `Approximation` / `omitted` / `dropped` matches 23 lines in the SOS
  catalog and 21 in STX). Five came off the list on modern_decks: Mage Hunters'
  Onslaught (a real turn-scoped floating watcher), Deadly Brew (the printed
  "another permanent card"), Ambitious Augmenter (counters of any kind), and
  stale notes on Emil and Harmonized Trio, both already faithful. Most capture
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

**Still open** (tracked, not fixed): the miracle window is step-bounded
rather than trigger-resolution-exact (the caster may act within the
granting step before deciding); `LookPickToHand` declines are honored
for UI players on `optional: true` cards (30 "you may put/reveal" cards
flipped) but the AutoDecider harness still auto-fills (bot policy);
plus the in-source documented approximations (grep
`Approximation|omitted|dropped`).

2026-07-15 cont.: `RevealMissDest::BottomRandom` genuinely shuffles the
miss batch before bottoming; Aziza's tap-three is a real all-or-nothing
cost with a `ChooseCards` pick of which creatures tap
(`TapUpToValue { exact: true }` behind a three-untapped `If` gate); the
eight collapsed "target player/opponent" slots are restored as real
player targets (Practiced Offense, Mathemagics, Mind Roots, Borrowed
Knowledge, Echocasting Symposium's two-slot token gift, Professor
Dellian Fel's emblem, Emeritus of Truce, Joined Researchers // Secret
Rendezvous), with `CreateToken`/`CreateTokenCopyOf` player-receiver
slots surfaced for cast-time validation. Nita, Forum Conciliator is
fully faithful (real IS-card target, pay-own-cost-any-color cast, and
`sac_other_filter` — the old `sac_cost` wrongly sacrificed Nita
herself); Visionary's Dance's `{2}, Discard this card:` line is a real
from-hand activation (`from_hand` + `discard_self_cost` + the
look-2-split); Zimone's Experiment is the printed SINGLE look at five
with typed routing (`picked_lands_to_battlefield` — lands enter tapped,
creatures to hand, any mix of up to two) and a genuinely random bottom
(`rest_bottom_random`); Archaic's Agony's excess-damage exile rider is
wired (`ExcessDamageDealtThisResolution` → exile top-N → play until the
end of your next turn at full cost; land plays through may-play remain
an engine gap); Mana Sculpt banks {C} equal to the countered spell's
actually-PAID mana at the caster's next main phase
(`CounteredSpellManaSpent` + `AddManaAtNextMainPhase`); Tester of the
Tangential's "you may pay {X}" is a real resolution-time amount pick
(`Effect::MayPayX` — ChooseAmount capped by floated mana, 0 declines,
body reads the chosen X via `XFromCost`); Great Hall's `{5}` is a real
PERMANENT 2/4 Wizard animation with the granted magecraft pump;
Petrified Hamlet ships all four printed abilities (ETB `NameCard`, the
CR 201.3 lockout, `GrantActivatedAbility` on `NamedBySource` lands —
residual: the name prompt doesn't restrict to land names);
Improvisation Capstone's free casts are order-choosable
(`CastAnyOrderWithoutPaying` re-offers declined cards); and bots now
cast prepare spells (`CastPrepareSpell` candidates in bot.rs).

The SOS catalog's audited gaps are now closed except: land plays
through may-play grants, the miracle window's step-bounded (rather than
trigger-exact) timing, and the NameCard land-name restriction — all
engine-wide residuals noted above.

## 2026-07-15 STX approximation sweep

A 4-agent pass over the STX catalog's 44 flagged approximation sites
fixed 33 cards outright (each verified against its doc-comment oracle,
with new/updated tests): the four college Commands are true choose-two
`ChooseModesCast` modals over their real printed modes; Past in Flames
is a real flashback grant (pay own cost, exile on resolve); Tend the
Pests sacrifices at CAST time; Whirlwind Denial sweeps the whole stack;
Pursuit of Knowledge pays a real remove-4-Study-counters cost;
Velomachus's reveal cap reads its LIVE power
(`SelectionRequirement::resolve_source_power` threading in
`RevealUntilFind`); plus MayPay/MayPayLife optionality, once-per-turn
gates, Battle targets, token-discounts, and doc-only corrections.

2026-07-15 follow-up: all eight blocked sites are now closed —
Approach of the Second Sun (per-name lifetime cast tally +
`PutResolvingSpellInLibraryFromTop(6)`; the single-copy recur loop is
exact), Transforming Flourish (`grant_to_exiling_player` on the
impulse), Multiple Choice (`ChooseModesCast` + the new
`Predicate::ChoseModesAtLeast` chosen-mode-set gate), Fractal Bloom
(`DistributeCountersAmongLastCreated` — UI-chosen split, even split for
bots), Channeled Force (player-ref `Value`s surface target slots via
`val_find`), Quandrix Command mode 3 (the shuffle's `who` is a real
player target; card picks stay resolution-time — no rules-visible
difference in the graveyard), Stormwild Capridor
(`PreventNoncombatDamageToSelfAddCounters` in the noncombat damage
funnel), and Strategic Planning (was already faithful; stale pointer
comment removed). Remaining narrow nuances are stated inline on each
card's doc comment.

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

## 2026-07-16 fabricated-bodies sweep (real-name STX cards)

All 248 real-named STX cards were verified clause-by-clause against the
local Scryfall cache (`scripts/.scryfall_cache.json`) and their
synthesized effect bodies rewritten to the printed design; coupled
tests (including CR-conformance fixtures and view-label tests that used
synthesized cards) were rewritten to assert real behavior. A final
read-only 248-card verification pass found exactly one residual
mismatch (Umbral Juke's Inkling token was the SOS 1/1 instead of the
real 2/1 — fixed, with a shared `stx_inkling_token()` helper). Cards
with narrow doc-named residuals (resolution-time picks, alt-cost
riders, name-strip riders, "another" exclusions, Commands'
target-player collapses) state their nuance inline. TODO.md's
"Fabricated real-name STX cards" item is closed.
