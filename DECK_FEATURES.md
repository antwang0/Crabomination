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
| 3 | Inquisition of Kozilek | 🟡 | `DiscardChosen(EachOpponent, Nonland ∧ ManaValueAtMost(3))`. Caster auto-picks first matching card. UI for the human picker still TODO. |

### Goryo's main deck

| Count | Card | Status | Notes |
|---|---|---|---|
| 4 | Atraxa, Grand Unifier | 🟡 | 7/7 Phyrexian Praetor with flying / vigilance / deathtouch / lifelink. ETB now uses `Value::DistinctTypesInTopOfLibrary { who: You, count: 10 }` — counts actual distinct card types in the top 10 of the controller's library and draws that many cards (instead of a flat 4). Reordering after the reveal is still flattened. |

## Modern supplement (catalog::sets::decks::modern)

A pack of additional Modern-playable cards built entirely on existing
engine primitives — no engine changes required. Each entry has at least
one functionality test in `crabomination/src/tests/modern.rs` (registered
via `#[path = "../tests/modern.rs"] mod tests_modern` in `game::mod`).

| Card | Cost | Status | Notes |
|---|---|---|---|
| Lava Dart | {R} | 🟡 | Flashback cost approximated as `{0}` — engine has no "sacrifice a Mountain" alt-cost primitive |
| Veil of Summer | {G} | 🟡 | Cantrip half wired; "if blue/black spell cast" gate + uncounterable rider stubbed |
| Lay Down Arms | {W} | 🟡 | `Exile(Creature ∧ PowerAtMost(4))`. Plains-count cost-rebate clause collapsed (no count-based-cost-rebate primitive). |
| Wild Mongrel | {1}{G} | 🟡 | 2/2 Hound; `Discard 1: +1/+1 EOT` (Psychic Frog mirror). The "becomes the color of your choice" half collapses. |
| Tear Asunder | {1}{B}{G} | 🟡 | `Destroy(Artifact ∨ Enchantment)`. Kicker {2} "destroy any nonland permanent" mode collapsed (alt-cost can't yet swap target filters at cast time). |
| Rout | {3}{W}{W} | 🟡 | `ForEach(Creature) + Destroy` — DoJ at +1 mana. Flash mode collapsed. |
| Chaos Warp | {2}{R} | 🟡 | `Move(target Permanent → Library(OwnerOf, Shuffled))`. The library actually reshuffles via the new `LibraryPosition::Shuffled` engine path. The "reveal top, cast if permanent" half is collapsed. Test: `chaos_warp_sends_target_permanent_to_owners_library`. |
| Karn, Scion of Urza | {4} | 🟡 | 5-loyalty Karn. **+1**: Draw 1 + mill 1 (the opp-pile-split is information-only at this engine fidelity). **-1**: ForEach Construct creature you control + AddCounter(+1/+1). **-2**: Create a 1/1 Construct token (the artifact-count scaling rider collapses). Tests: `karn_scion_of_urza_minus_two_creates_a_construct_token`, `karn_plus_one_draws_a_card_and_mills_one`. |
| Tezzeret, Cruel Captain | {3}{B} | 🟡 | 4-loyalty walker. **+1**: target creature gets -2/-2 EOT. **-2**: drain 2 life from each opponent. Static "your artifact creatures get +1/+1" wired; the ult remains collapsed. Tests: `tezzeret_minus_two_drains_each_opponent_for_two`, `tezzeret_plus_one_shrinks_target_creature`. |
| Crumble to Dust | {2}{R}{R} | 🟡 | Sorcery. Exile target nonbasic land. The "exile every card with the same name" rider collapses. Tests: `crumble_to_dust_exiles_nonbasic_land`, `crumble_to_dust_rejects_basic_land_target`. |
| Drown in the Loch | {U}{B} | 🟡 | Instant. ChooseMode([CounterSpell, Destroy(Creature ∨ Planeswalker)]). The "snow mana only" + "X = cards in opp's graveyard" gates collapse. Tests: `drown_in_the_loch_mode_zero_counters_a_spell`, `drown_in_the_loch_mode_one_destroys_creature`. |

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Reveal-and-sort ETB (one of each card type) | 🟡 | Atraxa, Grand Unifier now uses `Value::DistinctTypesInTopOfLibrary` to draw N cards where N = distinct types in the top 10. Real reveal-then-multi-pick (typed library reorder + one-per-type pick UI) still ⏳. |
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Cavern of Souls now flags any creature spell its controller casts as uncounterable (approximation collapses "name a type" + mana provenance into "you control a Cavern → your creatures are uncounterable"). |
| X-cost creature side-effects | 🟡 | Callous Sell-Sword now ETBs via `Seq([SacrificeAndRemember, PumpPT { power: SacrificedPower, EOT }])`. Casualty's "copy this spell" branch still ⏳ (no spell-copy-modal primitive). |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`; Plunge into Darkness still ⏳. |

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
