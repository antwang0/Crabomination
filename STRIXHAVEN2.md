# Strixhaven implementation tracker

Two adjacent catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`)
and **Strixhaven: School of Mages (STX)** (`catalog::sets::stx`).

The Learn / Lessons-sideboard mechanic is wired end-to-end (engine + every
Learn card + cube/format/draft sideboards + client UI modal).

**Prepare mechanic (June 2026 rework):** the original "all SOS cards ✅"
claim was wrong for the 36 preparation cards — they had been modeled as
hand-castable MDFCs, which is not the printed mechanic. They now use
`CardDefinition.prepare_spell` + `GameAction::CastPrepareSpell` (cast a
copy off the prepared battlefield creature; casting unprepares it), with
fronts audited against Scryfall oracle. See `.claude/prepared.md` for the
mechanic reference and TODO.md → "Prepare Mechanic (SOS)" for residual
per-card approximations.

For full per-card history see
`git log -- crabomination/src/catalog/sets/{stx,sos}/`.

## Legend

- 🟡 partial — body wired with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

No 🟡 partials remain. Kasmina, Enigma Sage is now fully faithful:
loyalty 2, the "each other planeswalker you control has Kasmina's loyalty
abilities" static (`StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities`),
and the real -8 (search a color-sharing instant/sorcery, exile, cast free).
