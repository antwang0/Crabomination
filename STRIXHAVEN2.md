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

- **~195 cards ship with documented approximations** (≈66 SOS, ≈129 STX —
  grep the catalogs for `Approximation` / `omitted` / `dropped`). Most capture
  the headline play pattern; a handful are near-vanilla/stub (e.g. Arclight
  Phoenix, Possibility Storm, Hindering Light, Detention Sphere) and **Mavinda,
  Students' Advocate stays 🟡** (gy-cast modeled as a {0} activated ability, not
  the printed static).
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
