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

No 🟡 partials are outstanding. ⏳ work remaining: ~60 printed STX cards
are still unimplemented (the Dean MDFCs, Codie/Extus/Blex/Jadzi legends,
several Lessons, and the X-spells) — see the "Remaining real STX cards"
entry in `TODO.md` for the catalog-diff recipe. The modern_decks run
added the extras_14 batch (Campus lands, Access Tunnel, Archway Commons,
keyword creatures, and assorted spells), all ✅.
