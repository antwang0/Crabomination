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

⏳ work remaining printed STX cards, grouped by blocker:
  (Hone counters ✅ — `CounterType::Hone` + `Effect::HoneFromHand` + the
  `GameState::process_hone` upkeep tick → {4}-less cast-from-exile grant;
  Nassari rides `ExileTopAndGrantMayPlay { EachOpponent }` +
  `Predicate::CastSpellFromExile`. Uvilda//Nassari shipped; Nassari's
  "spend mana as any color" clause is dropped. Study counters ✅ —
  `CounterType::Study`, Kianne//Imbraham.)
- **Continuous "becomes a copy" layer-1**: Echoing Equation (Augmenter's back).
- **Cast-from-top impulse + can't-cast-permanents static**: Codie.
- 🟡 **Awaken the Blood Avatar**: the optional "sacrifice any number, {2} less
  each" cast cost is dropped (no variable-sacrifice cost reduction yet).
- **Misc primitives**: Ecological Appreciation (up-to-four variable targets +
  opponent split), Jadzi, Flamescroll//Revel (opponent ability-activation
  trigger + spell-lock). (Radiant Scrollwielder ✅ —
  `StaticEffect::YourInstantSorcerySpellsHaveLifelink` + non-combat lifelink,
  CR 702.15; upkeep random-recur via auto-pick.)
