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

A handful of 🟡 partials remain (riders dropped, core wired):
- 🟡 **Retriever Phoenix** — ETB Learn; the graveyard "learn → return this
  instead" recursion replacement is dropped.
- 🟡 **Kasmina, Enigma Sage** — +2 Scry, -8 tutor-to-hand wired; the -X Fractal
  is approximated as a fixed -2 (two counters; no variable-X loyalty path yet)
  and the "other PWs gain Kasmina's abilities" static is dropped.
- 🟡 **The Biblioplex** — `{T}: Add {C}` + `{2},{T}: dig for an IS card`; the
  "else may bottom" half collapses to an auto-bottom of the non-IS top card.

⏳ work remaining: ~40 printed STX cards (the Dean MDFCs, Codie/Extus/Blex/
Jadzi legends, study/hone-counter cards, blink/recursion spells, the
remaining X-spells) — see the "Remaining real STX cards" entry in `TODO.md`.
The modern_decks run added extras_14/15/16 (Campus lands, keyword creatures,
the Lessons batch, Blot Out the Sky / Serpentine Curve / Golden Ratio,
spell-copy/counter, and payoff creatures), mostly ✅.
