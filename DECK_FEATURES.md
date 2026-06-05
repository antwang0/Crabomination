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

All Modern-supplement cards are ✅ and elided. Karn, Scion of Urza and
Tezzeret, Cruel Captain are now wired to their real oracle text (reveal-two /
opponent-chooses + silver counters; artifact-ETB loyalty trigger + 0/−3/−7).
Full per-card history is in git.

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Cavern of Souls flags any creature spell its controller casts as uncounterable. Turn-scoped grants ride `Player.spells_uncounterable_this_turn` (`Effect::GrantSpellsUncounterableThisTurn`) — Veil of Summer ✅. (Mana-provenance / name-a-type gates still collapse.) |
| X-cost creature side-effects | 🟡 | Thud / Burn at the Stake ride `SacrificeAndRemember` + `Value::SacrificedPower`. Casualty (CR 702.153) ✅ via `Keyword::Casualty(n)` + `GameAction::CastSpellCasualty` (sacrifice-to-copy); Adventure (CR 715) ✅. |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`. Variable-count sacrifice ✅ via `Effect::SacrificeAnyNumber` + `Decision::ChooseAmount` (Plunge into Darkness). Flashback-with-additional-cost (Lava Dart sac-a-Mountain, Dread Return sac-three) ✅ via `flashback_additional_cost_for_name` + `cast_flashback`. |

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
