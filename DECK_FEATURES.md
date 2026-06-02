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

### BRG main deck

| Count | Card | Status | Notes |
|---|---|---|---|
| 1 | Callous Sell-Sword | 🟡 | 4/4 with ETB "sacrifice a creature; +(sacrificed power)/+0 EOT". Casualty 2 "copy this spell" half omitted (no copy primitive yet). |
| 4 | Plunge into Darkness | 🟡 | `ChooseMode([SacrificeAndRemember + GainLife 3, LoseLife 4 + Search → Hand])`. Mode 0 sacrifices one creature for 3 life. Mode 1 pays 4 life and tutors any card (approximation of "pay X life, look at top X, take one"). Tests: `plunge_into_darkness_mode_one_pays_four_life_and_tutors`. |
| 4 | Spoils of the Vault | 🟡 | Wired to `Effect::RevealUntilFind { find: Any, cap: 10, life_per_revealed: 1 }`: walks the top of the library until a match (or cap), mills the misses, deducts 1 life per revealed card. With `find: Any` the very first card is taken (1 life). The "name a card" half is still pending a naming primitive. |

### BRG sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
## Modern supplement (catalog::sets::decks::modern)

A pack of additional Modern-playable cards built entirely on existing
engine primitives — no engine changes required. Each entry has at least
one functionality test in `crabomination/src/tests/modern.rs` (registered
via `#[path = "../tests/modern.rs"] mod tests_modern` in `game::mod`).

| Card | Cost | Status | Notes |
|---|---|---|---|
| Wild Mongrel | {1}{G} | 🟡 | 2/2 Hound; `Discard 1: +1/+1 EOT` (Psychic Frog mirror). The "becomes the color of your choice" half collapses. |
| Karn, Scion of Urza | {4} | 🟡 | 5-loyalty Karn. **+1**: Draw 1 + mill 1 (the opp-pile-split is information-only at this engine fidelity). **-1**: ForEach Construct creature you control + AddCounter(+1/+1). **-2**: Create a 1/1 Construct token (the artifact-count scaling rider collapses). Tests: `karn_scion_of_urza_minus_two_creates_a_construct_token`, `karn_plus_one_draws_a_card_and_mills_one`. |
| Tezzeret, Cruel Captain | {3}{B} | 🟡 | 4-loyalty walker. **+1**: target creature gets -2/-2 EOT. **-2**: drain 2 life from each opponent. Static "your artifact creatures get +1/+1" wired; the ult remains collapsed. Tests: `tezzeret_minus_two_drains_each_opponent_for_two`, `tezzeret_plus_one_shrinks_target_creature`. |
| Drown in the Loch | {U}{B} | 🟡 | Instant. ChooseMode([CounterSpell, Destroy(Creature ∨ Planeswalker)]). The "snow mana only" + "X = cards in opp's graveyard" gates collapse. Tests: `drown_in_the_loch_mode_zero_counters_a_spell`, `drown_in_the_loch_mode_one_destroys_creature`. |

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Cavern of Souls flags any creature spell its controller casts as uncounterable. Turn-scoped grants ride `Player.spells_uncounterable_this_turn` (`Effect::GrantSpellsUncounterableThisTurn`) — Veil of Summer ✅. (Mana-provenance / name-a-type gates still collapse.) |
| X-cost creature side-effects | 🟡 | Callous Sell-Sword now ETBs via `Seq([SacrificeAndRemember, PumpPT { power: SacrificedPower, EOT }])`. Casualty's "copy this spell" branch still ⏳ (no spell-copy-modal primitive). |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`; Plunge into Darkness still ⏳. Flashback-with-additional-cost (Lava Dart sac-a-Mountain, Dread Return sac-three) ✅ via `flashback_additional_cost_for_name` + `cast_flashback`. |

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
