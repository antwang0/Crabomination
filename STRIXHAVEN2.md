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

⏳ work remaining: ~40 printed STX cards (the Dean MDFCs, Codie/Extus/Blex/
Jadzi legends, study/hone-counter cards, blink/recursion spells, the
remaining X-spells — Exponential Growth now ✅ via `Effect::DoublePower`) — see
the "Remaining real STX cards" entry in `TODO.md`.
The modern_decks run added extras_14/15/16 (Campus lands, keyword creatures,
the Lessons batch, Blot Out the Sky / Serpentine Curve / Golden Ratio,
spell-copy/counter, and payoff creatures), mostly ✅.
