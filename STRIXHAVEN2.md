# Strixhaven implementation tracker

Two catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`) and
**Strixhaven: School of Mages (STX)** (`catalog::sets::stx`).

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
- **Cost + P/T match real Scryfall** (`audit_stx_drift.py` → 0 drift). The new
  `audit_stx_types.py` adds type-line + keyword checks: of **49 creature-type +
  24 keyword drifts** found on 2026-06-14, **47 types + 5 keywords were fixed**
  (e.g. Mavinda Cleric+Vigilance → Bird Advisor+Flying; Beledros Demon → Elder
  Dragon), full suite green. **Left:** 4 types (3 synthesized cards coupled to
  Pest/Fractal/Inkling synergy tests, + Eccentric Apprentice's *Tiefling* type
  which lacks an enum variant) and 19 keywords (mostly conditional/granted
  keywords the catalog models as base keywords — case-by-case, not a sweep).
  Caveat: many fixed cards are fabricated-real-name collisions whose **bodies are
  still synthesized** — correct stats ≠ faithful card. Run `audit_stx_types.py`
  for the remaining list. See TODO.md → "Fabricated real-name STX cards".

Full per-card history: `git log -- crabomination_catalog/src/sets/{stx,sos}/`.
