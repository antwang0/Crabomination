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
- **Multi-target prompts on instants/sorceries** — recurring 🟡 reason
  across SOS/STX (Vibrant Outburst, Snow Day, Devious Cover-Up, Crackle
  with Power, Magma Opus). Divided-damage / per-mode multi-target slots
  remain a gap distinct from the bag-of-targets primitives.

## Red

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Steal the Show | {2}{R} | Sorcery |  | Choose one or both — / • Target player discards any number of cards, then draws that many cards. / • Steal the Show deals damage equal to the number of instant and sorcery cards in your graveyard to target creature or planeswalker. | 🟡 | Per-mode behavior is faithful (mode 0 = `DiscardAnyNumber` then draw-that-many via `Value::CardsDiscardedThisEffect`; mode 1 = damage = IS-cards in your gy). Only the "choose one **or both**" rider collapses to a single mode pick (no multi-mode-pick that fills two target slots). |

## Lorehold (Red-White)

| Card | Mana Cost | Type | P/T | Oracle Text | Status | Notes |
|---|---|---|---|---|---|---|
| Molten Note | {X}{R}{W} | Sorcery |  | Molten Note deals damage to target creature equal to the amount of mana spent to cast this spell. Untap all creatures you control. / Flashback {6}{R}{W} | 🟡 | X+2 damage to creature + untap all your creatures + Flashback. Only edge left: a flashback cast reads X=0, so its damage is undercounted (the "mana spent" model only tracks the {X} pip). |

### Engine pieces driven by STX

- ⏳ **Lesson sideboard model** — Eyetwitch, Hunt for Specimens, Pest
  Summoning all use Learn at some point. Currently approximated as
  `Draw 1`.
