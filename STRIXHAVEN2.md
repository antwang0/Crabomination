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

- **Lessons sideboard / Learn** — 🟡 **wired end-to-end (engine + cube).**
  `Player.sideboard` holds Lessons "outside the game"; `Effect::Learn`
  surfaces `Decision::Learn` (reveal a Lesson into hand / discard-to-draw /
  decline) and falls back to `Draw 1` when no Lessons sideboard is
  configured. **All Strixhaven Learn cards** now use `Effect::Learn` (the
  four canonical ones plus the Lessons that themselves Learn and Professor
  of Symbology). `cube::build_cube_state` gives every seat the standard
  Lessons sideboard via `cube::lessons_sideboard()`, so Learn fetches in
  real cube games. The client UI suspend flow is wired too: a `wants_ui`
  player's Learn suspends on `Decision::Learn` (`PendingEffectState::LearnPending`)
  and the client renders a modal (reveal-a-Lesson / discard-to-draw /
  decline) that submits `DecisionAnswer::Learn(LearnChoice)`. Every
  deck-build path (cube, formats, **and draft**) seats the standard Lessons
  sideboard, so Learn fetches resolve in all game modes.
- **Multi-target prompts on instants/sorceries** — "choose one or both"
  with a target per chosen mode works via `Effect::ChooseN`'s per-mode
  target slots (Steal the Show). **Divided damage** ("deal N damage divided
  as you choose among 1+ targets") is now wired via `Effect::DealDamageDivided`
  + `Decision::DivideDamage` (Forked Bolt, Pyrokinesis, Crackle with Power,
  Magma Opus; AutoDecider spreads evenly). The remaining gap is *non-damage*
  split-N riders (Snow Day's divided tap/scry, Devious Cover-Up's split-mill).

All printed SOS cards are now ✅, and the Learn / Lessons-sideboard mechanic
is wired end-to-end (engine + every Learn card + cube/format/draft sideboards
+ client UI modal).
