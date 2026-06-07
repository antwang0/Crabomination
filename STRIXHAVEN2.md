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
- 🟡 **Reject** — counters unless pay {3}; the exile-instead-of-graveyard
  rider on a successful counter is dropped (no `CounterUnlessPaid` exile flag).
- 🟡 **Oriq Loremage** — `{T}: search → graveyard`; the "if IS, +1/+1 counter"
  rider is dropped (search result type isn't surfaced back).
- 🟡 **Devouring Tendrils** — one-sided power-damage; the delayed "gain 2 when
  it dies this turn" rider is dropped (`WhenTargetDiesThisTurn` watches slot 0).
- 🟡 **Retriever Phoenix** — ETB Learn; the graveyard "learn → return this
  instead" recursion replacement is dropped.
- 🟡 **Elemental Masterpiece** — two 4/4s; the discard-from-hand-for-Treasure
  ability is dropped (no from-hand discard-cost mana rider).
- 🟡 **Detention Vortex** — locks attack/block + activated abilities (CR
  602.5c); the opponent-only `{3}: Destroy this Aura` escape clause is dropped.
- 🟡 **Kasmina, Enigma Sage** — +2 Scry, -8 tutor-to-hand wired; the -X Fractal
  is approximated as a fixed -2 (two counters; no variable-X loyalty path yet)
  and the "other PWs gain Kasmina's abilities" static is dropped.
- 🟡 **The Biblioplex** — `{T}: Add {C}` + `{2},{T}: dig for an IS card`; the
  "else may bottom" half collapses to an auto-bottom of the non-IS top card.
- 🟡 **Deadly Brew** — each player sacrifices; the "if you sacrificed" gate on
  the graveyard-return collapses (auto-picks a permanent).
- 🟡 **Dramatic Finale** — token anthem + nontoken-death Inkling; the "only
  once each turn" limiter on the death trigger is dropped.

⏳ work remaining: ~40 printed STX cards (the Dean MDFCs, Codie/Extus/Blex/
Jadzi legends, study/hone-counter cards, blink/recursion spells, the
remaining X-spells) — see the "Remaining real STX cards" entry in `TODO.md`.
The modern_decks run added extras_14/15/16 (Campus lands, keyword creatures,
the Lessons batch, Blot Out the Sky / Serpentine Curve / Golden Ratio,
spell-copy/counter, and payoff creatures), mostly ✅.
