# Strixhaven implementation tracker

Two adjacent catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`)
and **Strixhaven: School of Mages (STX)** (`catalog::sets::stx`).

All printed SOS cards are ✅, and the Learn / Lessons-sideboard mechanic is
wired end-to-end (engine + every Learn card + cube/format/draft sideboards +
client UI modal). No 🟡/⏳ card work is outstanding here.

For full per-card history see
`git log -- crabomination/src/catalog/sets/{stx,sos}/`.

## Legend

- 🟡 partial — body wired with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

A couple of 🟡 partials remain (riders dropped, core wired):
- 🟡 **Kasmina, Enigma Sage** — +2 Scry; -X Fractal now ✅ via variable-X
  loyalty (`LoyaltyAbility.x_cost` + `Value::XFromCost`); -8 stays tutor-to-hand
  (the IS-sharing-color / cast-free chain is dropped) and the "other PWs gain
  Kasmina's abilities" static is dropped.

⏳ work remaining: ~16 printed STX cards (most remaining Dean/legend MDFCs,
study/hone-counter cards, and a handful of single-faced cards blocked on
primitives — Draconic Intervention, Fervent Mastery, Radiant Scrollwielder,
Codie, Elite Spellbinder, Ecological Appreciation). See the "Remaining real STX
cards" entry in `TODO.md` for the per-card blockers.
✅ this run: First Day of Class (turn-scoped enters trigger), Shaile // Embrose
(the first Dean MDFC, via `SelectionRequirement::EnteredThisTurn`); fixed stale
docs/subtypes on Illuminate History, Rise of Extus (already faithful), Daemogoth
Titan.
