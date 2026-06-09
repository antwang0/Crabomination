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

✅ recent: Shaile // Embrose, Plargg // Augusta (Dean MDFC — loot + reveal-cast /
tapped+untapped anthems + untap-on-attack; tapped/untapped anthem filters now
live), Extus // Awaken the Blood Avatar (magecraft gy-return + each-opp-sac +
3/6 Avatar), Mascot Exhibition (corrected to the real {7} Lesson). Emergent
Sequence, Torrent Sculptor // Flamethrower Sonata, Blex // Search for Blex.

✅ also: Rowan//Will, Mila//Lukka (planeswalker MDFCs), Valentin//Lisette
(`StaticEffect::ExileDyingOpponentCreatures` death-replacement + reflexive).
Demonstrate cycle ✅ — Creative / Excavation / Healing / Incarnation /
Replication Technique + Transforming Flourish (`Effect::Demonstrate`, CR
702.150).

All previously-blocked printed STX cards now ship:
- ✅ Echoing Equation (Augmenter's back) — `Effect::BecomeCopyOfFor`
  (CR 707.2 continuous copy with EOT/leave revert).
- ✅ Codie, Vociferous Codex — `ControllerCantCastPermanentSpells` +
  `OnYourNextSpellCastThisTurn` + filtered `Discover` impulse.
- ✅ Ecological Appreciation (`Effect::SearchSplitWithOpponent`),
  Jadzi // Journey to the Oracle (pay-{1} top-of-library cast +
  put-lands-from-hand + self-return), Flamescroll // Revel in Silence
  (opponent non-mana-activation trigger + `SilencePlayersThisTurn`).
