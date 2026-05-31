# Strixhaven implementation tracker

Two adjacent catalogs: **Secrets of Strixhaven (SOS)** (`catalog::sets::sos`)
and **Strixhaven: School of Mages (STX)** (`catalog::sets::stx`).

All ✅-done cards have been elided — the tables below list only the
remaining **🟡 partial** and **⏳ todo** work. For full per-card history see
`git log -- crabomination/src/catalog/sets/{stx,sos}/`.

## Legend

- 🟡 partial — body wired with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Recently landed engine primitives

These primitives closed a batch of SOS 🟡/⏳ riders (all green in
`tests::sos`):

- **Spend-restricted mana** (`ManaPool::pay_for_spell` + `SpendRestriction`
  + `ManaPayload::Restricted`) — "Spend this mana only to cast instant and
  sorcery spells." Finishes Tablet of Discovery, Abstract Paintmage,
  Hydro-Channeler, Great Hall of the Biblioplex, Resonating Lute.
- **Death replacement** (`Effect::ExileIfWouldDieThisTurn`,
  `GameState::dies_to_exile_eot`) — "If that creature would die this turn,
  exile it instead." Finishes Wilt in the Heat (and correctly spares
  indestructible creatures / catches later-this-turn deaths).
- **Transient granted flashback** (`CardInstance::granted_flashback_eot` +
  `Effect::GrantFlashbackThisTurn`) — finishes the SOS "Flashback" instant
  (recast at the card's own mana cost, exile on resolve).
- **Miracle alt-cost** (`CardInstance::granted_alt_cast_cost_eot` +
  `Effect::GrantMiracle`) — finishes Lorehold, the Historian's "miracle
  {2}" grant (cast the first IS card drawn for {2}).
- **Granted cascade + cast-from-hand stamping** (`Predicate::CastFromHand`
  now read off the actual cast spell) — finishes Quandrix, the Proof's
  "instant and sorcery spells you cast from your hand have cascade."
- **Can't-be-copied** (`Keyword::CantBeCopied`, honored by
  `Effect::CopySpell`) — finishes Choreographed Sparks's "this spell can't
  be copied" rider.

## Known engine gaps surfaced by these catalogs

- **Lessons sideboard / Learn** — Eyetwitch, Pest Summoning, Hunt for
  Specimens, Field Trip, Igneous Inspiration. Approximated as `Draw 1`.
- **Multi-target prompts on instants/sorceries** — "choose one or both"
  with a target per chosen mode now works via `Effect::ChooseN`'s per-mode
  target slots (Steal the Show). The remaining gap is *divided* targeting
  within a single mode/effect (Vibrant Outburst, Snow Day, Devious
  Cover-Up, Crackle with Power, Magma Opus — divided-damage / split-N
  slots), distinct from the bag-of-targets primitives.

All printed SOS cards are now ✅ — the only remaining SOS/STX item is the
shared Learn / Lessons-sideboard mechanic below.

### Engine pieces driven by STX

- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
