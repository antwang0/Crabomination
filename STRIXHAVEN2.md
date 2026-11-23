# Strixhaven implementation tracker

Two catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`) and
**Strixhaven: School of Mages (STX)** (`catalog::sets::stx`). Per-card history:
`git log -- crabomination_catalog/src/sets/{stx,sos}/`.

## Status

- **STX** base set is 100% implemented. The **Mystical Archive (STA)**
  companion (`catalog::sets::stx::sta`) is complete: Infuriate, Blue Sun's
  Zenith, Abundant Harvest, Urza's Rage, Natural Order, Tainted Pact
  (`Effect::ExileUntilDuplicateName`), Mizzix's Mastery (exile-I/S-from-gy +
  free-cast + overload via `effect_override`) — tests in `tests/stx/part_31.rs`.
- **SOS Special Guests (SPG)** ship: `sos_mode::sos_special_guests` names all
  eleven, and `generate_sos_pack` gives the sheet its own slot at the printed
  1-in-64 rate. The vocab grew, so trained value nets need a retrain — see
  CARD_BACKLOG.md → "SOS Special Guests (SPG)".
- **Learn / Lessons-sideboard** is wired end to end (engine, every Learn card,
  cube/format/draft sideboards, client modal).
- **Prepare** (reworked June 2026): the 36 preparation cards use
  `CardDefinition.prepare_spell` + `GameAction::CastPrepareSpell` — you cast a
  *copy* off the prepared battlefield creature, which unprepares it (the old,
  wrong model was hand-castable MDFCs). Fronts audited against Scryfall; bots
  cast prepare spells. Mechanic: `.claude/prepared.md`; per-card
  approximations: TODO.md → "Prepare Mechanic (SOS)".
- **Kasmina, Enigma Sage** is fully faithful: loyalty 2, the "other
  planeswalkers share Kasmina's loyalty abilities" static
  (`StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities`), and the
  real -8.
- **Stats match Scryfall.** `audit_catalog_stats.py stx` (cost, P/T, type line,
  keywords; replaced `audit_stx_drift.py` / `audit_stx_types.py`) is the live
  check. Remaining flags are non-bugs: 3 types (synthesized cards coupled to
  Pest/Fractal/Inkling synergy tests) and 1 keyword (Lone Rider — a DFC
  back-face Trample artifact; the modeled front is correct). Conditional /
  granted keywords modelled via statics (Leech Fanatic, Sticky Fingers, …) are
  correct and not flagged.
- **All 248 real-named STX cards** were verified clause by clause against the
  local Scryfall cache (`scripts/.scryfall_cache.json`) and their synthesized
  bodies rewritten to the printed design (2026-07-16), coupled tests included;
  the closing verification found one mismatch (Umbral Juke's Inkling, now the
  real 2/1 via the shared `stx_inkling_token()` helper).

"No partials remain" is true only in the tracker sense (no card is marked
🟡/⏳). Documented approximations remain in source — a case-insensitive grep
for `Approximation|omitted|dropped` (23 lines in SOS and 21 in STX as of
2026-08-01); most capture the headline play pattern, and narrow residuals are
stated on each card's doc comment.

## Open residuals (engine-wide)

- **Miracle window** is step-bounded (`MayPlayDuration::EndOfThisStep` +
  `MayPlayPermission.miracle`) rather than trigger-resolution-exact: the caster
  may act within the granting step before deciding.
- **`LookPickToHand` declines** are honored for UI players on `optional: true`
  cards, but the AutoDecider harness still auto-fills (bot policy).
- **Land plays through may-play grants** (Archaic's Agony's exiled cards play
  at full cost; land plays remain an engine gap).
- **`NameCard` land-name restriction**: Petrified Hamlet's name prompt doesn't
  restrict to land names.
- The in-source documented approximations above.

## Audit record

The audits behind the status, newest first (detail in git):

- **2026-07-16 fabricated-bodies sweep** — the 248 real-named STX cards above.
- **2026-07-15 STX approximation sweep** — 44 flagged sites: 33 fixed outright,
  then the eight blocked ones (Approach of the Second Sun, Transforming
  Flourish, Multiple Choice via `Predicate::ChoseModesAtLeast`, Fractal Bloom
  via `DistributeCountersAmongLastCreated`, Channeled Force, Quandrix Command
  mode 3, Stormwild Capridor, Strategic Planning). The college Commands are
  true choose-two `ChooseModesCast` modals; cast-time multi-mode selection
  (`Effect::ChooseModesCast`, sharing `CastSpellSpree` plumbing) covers Moment
  of Reckoning and Choreographed Sparks; "pay X life" is a cast-time
  additional cost (`additional_cost_pay_x_life`); printed "you may choose new
  targets" copies use `Effect::CopySpellMayChooseTargets`.
- **2026-07-14/15 SOS full-mechanics audit** — all 256 SOS card functions
  against their doc-comment oracle (~100 had a deviation). Engine fixes from
  it: `Effect::Search` shuffles the searched library even on a decline (CR
  701.19c; library destinations exempt — `cr_701_19c_search_shuffles_library_even_on_decline`);
  "one or more cards leave your graveyard" fires once per batch; "one or more
  creatures … deal combat damage" graveyard triggers fire once per damage
  sub-step; triggered abilities that declare a target emit `BecameTarget`;
  `PlayerRef::ControllerOf` on a dead permanent uses death-time LKI (CR
  608.2h); `StaticEffect::GrantStormToISSpells` (Prismari);
  `CardDefinition::self_cost_reduction_cost_if_target`;
  `Predicate::FirstLifeGainThisTurn`; `EntersTappedUnless` reads the cast's
  stamped X; `RevealMissDest::BottomRandom` shuffles before bottoming; the
  eight collapsed "target player/opponent" slots restored as real player
  targets (with `CreateToken`/`CreateTokenCopyOf` receiver slots validated at
  cast); `Effect::MayPayX`; `CastAnyOrderWithoutPaying`. The SOS catalog's
  audited gaps are closed except the open residuals above.
- **2026-07-12** — the per-card status tables were removed (their scripts —
  `audit_strixhaven2.py`, `list_sos_ok.py`, `sos_ok_factory_map.py`,
  `gen_strixhaven2.py` — followed on 2026-09-28; git has them); 15 gameplay
  bugs fixed and the sos_mode pools backfilled with every implemented card.
- **2026-06-14/15 stats sweep** — 47 creature types + 20 keywords fixed.
