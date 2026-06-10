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

No 🟡 partials remain. Kasmina, Enigma Sage is now fully faithful:
loyalty 2, the "each other planeswalker you control has Kasmina's loyalty
abilities" static (`StaticEffect::OtherPlaneswalkersHaveSourceLoyaltyAbilities`),
and the real -8 (search a color-sharing instant/sorcery, exile, cast free).
