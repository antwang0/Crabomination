# Deck Implementation Tracker

Tracking the work to make these two decks fully playable:
- **BRG combo** (Cosmogoyf + Thud, Pact-style)
- **Goryo's Vengeance reanimator**

Both decks are wired as the default demo match (`crabomination::demo::build_demo_state` — P0 = BRG, P1 = Goryo's).

**Done (✅) cards and engine features have been elided** — only the
remaining 🟡/⏳ work is listed below. Full per-card history is in git.

## Legend

- 🟡 partial — card exists with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

### BRG main deck / sideboard

All BRG cards are ✅ and elided. Full per-card history is in git.

## Modern supplement (catalog::sets::decks::modern)

A pack of additional Modern-playable cards built entirely on existing
engine primitives — no engine changes required. Each entry has at least
one functionality test in `crabomination/src/tests/modern.rs` (registered
via `#[path = "../tests/modern.rs"] mod tests_modern` in `game::mod`).

| Card | Cost | Status | Notes |
|---|---|---|---|
| Karn, Scion of Urza | {4} | 🟡 | 5-loyalty Karn. **+1**: Draw 1 + mill 1 (the opp-pile-split is information-only at this engine fidelity). **-1**: ForEach Construct creature you control + AddCounter(+1/+1). **-2**: Create a 0/0 Construct token that gets +1/+1 for each artifact you control (via `StaticEffect::PumpSelfByControlledPermanents`). Tests: `karn_scion_of_urza_minus_two_creates_a_construct_token`, `karn_plus_one_draws_a_card_and_mills_one`. |
| Tezzeret, Cruel Captain | {3}{B} | 🟡 | 4-loyalty walker. **+1**: target creature gets -2/-2 EOT. **-2**: drain 2 life from each opponent. Static "your artifact creatures get +1/+1" wired; the ult remains collapsed. Tests: `tezzeret_minus_two_drains_each_opponent_for_two`, `tezzeret_plus_one_shrinks_target_creature`. |

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Cavern of Souls flags any creature spell its controller casts as uncounterable. Turn-scoped grants ride `Player.spells_uncounterable_this_turn` (`Effect::GrantSpellsUncounterableThisTurn`) — Veil of Summer ✅. (Mana-provenance / name-a-type gates still collapse.) |
| X-cost creature side-effects | 🟡 | Thud / Burn at the Stake ride `SacrificeAndRemember` + `Value::SacrificedPower`. Casualty's "copy this spell" branch still ⏳ (no Casualty cost-mode primitive); Adventure cost-modes (Burn Together) ⏳. |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`. Variable-count sacrifice ✅ via `Effect::SacrificeAnyNumber` + `Decision::ChooseAmount` (Plunge into Darkness). Flashback-with-additional-cost (Lava Dart sac-a-Mountain, Dread Return sac-three) ✅ via `flashback_additional_cost_for_name` + `cast_flashback`. |
| Variable-count / pay-any-amount choices | ✅ | `Decision::ChooseAmount` (number 0..=max) backs `Effect::SacrificeAnyNumber` (sacrifice any number) and `Effect::PayLifeLookTake` (pay X life, dig X, take one, exile rest). Entwine modeled as `Keyword::Kicker` + `SpellWasKicked` branch (Plunge into Darkness). |
| Suspend (CR 702.62) | ✅ | `Keyword::Suspend(n, cost)` + `GameAction::Suspend` + `process_suspend` (tick at owner's upkeep, free-cast when last time counter comes off). Rift Bolt, Ancestral Vision, Lotus Bloom, Search for Tomorrow, Errant Ephemeron, Riftwing Cloudskate. Creature-suspend haste + a UI/targeting prompt for the free cast are TODO.md follow-ups. |
| Foretell (CR 702.143) | ✅ | `CardDefinition.foretell_cost` + `GameAction::Foretell` ({2}, exile face-down) + `GameAction::CastForetold` (cast from exile for the foretell cost on a later turn; `GameState.foretold_this_turn` gate). Saw It Coming, Doomskar, Behold the Multiverse, Demon Bolt, Augury Raven. `PlayerView.foretellable_hand` + cyan client highlight; no Bevy modal yet (TODO.md). |

## Plan

Implementation phases (work top-down; each phase unlocks more deck behavior):

1. **Catalog stubs** for all listed cards. Correct cost / types / P/T / keywords; effects = `Effect::Noop` where engine support is missing. Both decks become *playable as bodies*.
2. **Wire `demo.rs`** so the singleplayer match starts with these 60-card decks (P0 = BRG, P1 = Goryo's).
3. **Tractable engine features** that unlock multiple cards:
   - Alternative pitch costs.
   - Shock-land ETB choice.
   - Surveil-land ETB-tapped + surveil.
   - Fastland conditional ETB-tapped.
   - Convoke / Converge.
4. **Card-specific engine features**:
   - Pact deferred upkeep costs.
   - Rebound (Ephemerate).
   - Goryo's exile-at-EOT.
   - Atraxa reveal-and-sort.
   - Static effects (Damping Sphere, Elesh Norn, Teferi).
   - Counter-an-ability (Consign to Memory).
5. **Opening-hand effects** (Chancellor, Leyline, Gemstone Caverns, Serum Powder) — these need pre-game mulligan-window machinery.

Treat the table as the source of truth. When promoting a card from ⏳ to 🟡 or ✅, also flip its dependent engine feature row.
