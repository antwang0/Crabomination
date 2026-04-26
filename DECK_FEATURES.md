# Deck Implementation Tracker

Tracking the work to make these two decks fully playable:
- **BRG combo** (Cosmogoyf + Thud, Pact-style)
- **Goryo's Vengeance reanimator**

Both decks are wired as the default demo match (`crabomination::demo::build_demo_state` — P0 = BRG, P1 = Goryo's).

## Legend

- ✅ done
- 🟡 partial — card exists with simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Cards

### BRG main deck

| Count | Card | Status | Notes |
|---|---|---|---|
| 4 | Blackcleave Cliffs | ⏳ | BR fastland (ETB tapped if 2+ lands) |
| 2 | Blightstep Pathway | ⏳ | Modal land: B or R face |
| 4 | Blooming Marsh | ⏳ | BG fastland |
| 1 | Callous Sell-Sword | ⏳ | R creature with X-cost on cast |
| 4 | Chancellor of the Tangle | ⏳ | 6/7 G; opening-hand reveal adds {G} |
| 4 | Copperline Gorge | ⏳ | RG fastland |
| 4 | Cosmogoyf | ✅ | Dynamic P/T = (distinct card types in all graveyards) / (count + 1) via injected layer-7 `SetPowerToughness` effect at `compute_battlefield` time. |
| 2 | Darkbore Pathway | ⏳ | Modal land: B or G face |
| 4 | Devourer of Destiny | ⏳ | 7/5 Eldrazi colorless creature |
| 4 | Gemstone Caverns | ⏳ | Opening-hand: ETB with luck counter |
| 4 | Gemstone Mine | ⏳ | Any color, 3 charge counters |
| 4 | Pact of Negation | ✅ | Counterspell + delayed `PayOrLoseGame` trigger on next upkeep. Auto-pays if affordable; eliminates caster otherwise. |
| 4 | Plunge into Darkness | 🟡 | `ChooseMode([SacrificeAndRemember + GainLife 3, Noop])`. Mode 0 sacrifices one creature for 3 life (instead of "any number"). Mode 1 (pay-X-life-look-at-X) still ⏳. |
| 4 | Serum Powder | ⏳ | Mulligan helper: exile hand, draw new |
| 4 | Spoils of the Vault | ⏳ | Name + reveal until found, lose life |
| 2 | Summoner's Pact | ✅ | Search green creature into hand + delayed `PayOrLoseGame` for {2}{G}{G} on next upkeep. |
| 1 | Swamp | ✅ | basic |
| 4 | Thud | ✅ | `Seq([SacrificeAndRemember, DealDamage(SacrificedPower)])` — auto-picks first eligible creature via `AutoDecider`. Sac-as-additional-cost approximated by sac-on-resolution. |

### BRG sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
| 3 | Cavern of Souls | 🟡 | Tap for {C} works. Engine now has `StackItem::Spell.uncounterable` flag and `CounterSpell` skips them — but Cavern doesn't actually mark cast spells as uncounterable yet (needs name-a-type ETB choice + per-spell tagging at cast time). |
| 4 | Chancellor of the Annex | ⏳ | 5/6 W; opening-hand reveal: opponent's first spell costs {1} more |
| 1 | Forest | ✅ | basic |
| 3 | Inquisition of Kozilek | 🟡 | `DiscardChosen(EachOpponent, Nonland ∧ ManaValueAtMost(3))`. Caster auto-picks first matching card. UI for the human picker still TODO. |
| 4 | Leyline of Sanctity | ⏳ | Opening-hand: starts in play; you have hexproof |

### Goryo's main deck

| Count | Card | Status | Notes |
|---|---|---|---|
| 4 | Atraxa, Grand Unifier | ⏳ | 7/7 Phyrexian Praetor; ETB reveal-7-and-sort |
| 1 | Cephalid Coliseum | ⏳ | Tap for U; activated mill ability |
| 4 | Ephemerate | ⏳ | Flicker creature; rebound |
| 4 | Faithful Mending | 🟡 | `Seq([Discard 2 (you), Draw 2, GainLife 2])` + `Keyword::Flashback({1}{B})`. "Up to two" still approximated. |
| 3 | Flooded Strand | ✅ | fetchland |
| 3 | Force of Negation | 🟡 | Counter noncreature spell; alt pitch cost wired (engine only — no client UI to invoke alt cost). "Not your turn" timing on alt cost not enforced. |
| 1 | Godless Shrine | ⏳ | WB shockland |
| 4 | Goryo's Vengeance | ✅ | Reanimate legendary creature → grant haste until end of turn → delayed exile-at-end-step. Full Oracle. |
| 1 | Griselbrand | ⏳ | 7/7 Demon flying lifelink; pay 7 life draw 7 |
| 1 | Hallowed Fountain | ⏳ | WU shockland |
| 1 | Island | ✅ | basic |
| 3 | Marsh Flats | ✅ | fetchland |
| 1 | Meticulous Archive | ⏳ | UW surveil land |
| 1 | Overgrown Tomb | ⏳ | BG shockland |
| 1 | Plains | ✅ | basic |
| 4 | Polluted Delta | ✅ | fetchland |
| 4 | Prismatic Ending | ⏳ | Convoke; exile permanent CMC ≤ converge |
| 4 | Psychic Frog | ⏳ | UB Frog 1/3 flying; discard ability |
| 4 | Quantum Riddler | ⏳ | UB; cantrip on cast |
| 1 | Shadowy Backstreet | ⏳ | UB surveil land |
| 4 | Solitude | ✅ | 3/2 flash flying lifelink + ETB exile target opponent's creature + evoke (pitch a white card; sacrifice on ETB). "Nonwhite" filter approximated as "any creature". |
| 1 | Swamp | ✅ | basic |
| 3 | Thoughtseize | 🟡 | `Seq([DiscardChosen(EachOpponent, Nonland), LoseLife 2])`. Caster auto-picks first matching card. |
| 1 | Undercity Sewers | ⏳ | UB surveil land |
| 1 | Watery Grave | ⏳ | UB shockland |

### Goryo's sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
| 3 | Consign to Memory | ⏳ | Counter activated/triggered ability |
| 2 | Damping Sphere | ⏳ | Static: extra mana to cast 2nd+ spell each turn |
| 1 | Elesh Norn, Mother of Machines | ⏳ | Static: ETB triggers, doubles yours |
| 3 | Mystical Dispute | ⏳ | Counter blue spell; alt cost {U} if blue |
| 2 | Pest Control | ⏳ | Convoke; destroy creatures CMC ≤ converge |
| 2 | Teferi, Time Raveler | ⏳ | Planeswalker; static + +1 / -3 |
| 2 | Wrath of the Skies | ⏳ | Convoke; destroy permanents CMC ≤ converge |

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Alternative pitch costs (pay life + exile a card) | ✅ | Engine + client. Force of Will, Force of Negation, Solitude (evoke). Right-click a hand card with `has_alternative_cost` → modal lets the player pick a pitch card. `evoke_sacrifice` flag on `AlternativeCost` schedules a self-sac trigger after ETB. |
| Pact-style deferred upkeep cost | ✅ | Pact of Negation, Summoner's Pact (built on `Effect::DelayUntil` + `Effect::PayOrLoseGame`) |
| Goryo's Vengeance: reanimate-then-exile-at-EOT | ✅ | Goryo's Vengeance (uses `DelayUntil(NextEndStep)` + `Exile { Target(0) }`) |
| Rebound (cast from exile next upkeep) | ⏳ | Ephemerate |
| Flicker (exile and return to play) | ⏳ | Ephemerate |
| Convoke / Converge cost-reduction | ⏳ | Prismatic Ending, Wrath of the Skies, Pest Control |
| Opening-hand effects (begin in play / replace draws) | ⏳ | Chancellor of the Tangle, Chancellor of the Annex, Leyline of Sanctity, Gemstone Caverns, Serum Powder |
| Reveal-and-sort ETB (one of each card type) | ⏳ | Atraxa, Grand Unifier |
| Static cost increase + storm tax | ⏳ | Damping Sphere |
| ETB-trigger replacement (suppress / double) | ⏳ | Elesh Norn |
| Spell-timing restriction static | ⏳ | Teferi, Time Raveler |
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Wiring on Cavern (name-a-type ETB + per-cast tagging) still TODO. |
| Counter target *ability* (not spell) | ⏳ | Consign to Memory |
| Charge-counter mana sources w/ self-sac | ⏳ | Gemstone Mine |
| Shock-land ETB choice (tapped or 2 life) | ⏳ | Godless Shrine, Hallowed Fountain, Watery Grave, Overgrown Tomb |
| Pathway / modal DFC mana abilities | ⏳ | Blightstep Pathway, Darkbore Pathway |
| Surveil-land ETB-tapped + surveil 1 | ⏳ | Meticulous Archive, Undercity Sewers, Shadowy Backstreet |
| Fastland conditional ETB-tapped | ⏳ | Blackcleave Cliffs, Blooming Marsh, Copperline Gorge |
| Activated land mill (Cephalid Coliseum) | ⏳ | Cephalid Coliseum |
| Tarmogoyf-style P/T from graveyard | ✅ | Cosmogoyf (via inline `compute_battlefield` injection of a layer-7 set-PT effect with the live graveyard card-type count). |
| X-cost creature side-effects | ⏳ | Callous Sell-Sword |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`; Plunge into Darkness still ⏳. |
| Reveal-until-find search | ⏳ | Spoils of the Vault |
| Loyalty abilities w/ static | 🟡 | Teferi, Time Raveler (only -X/+1 supported) |

## Implementation log (most recent first)

- **Five-feature batch (Goryo's haste, Faithful Mending flashback, chosen-discard, Plunge modal, uncounterable flag)**:
  - Goryo's Vengeance now appends `GrantKeyword(Haste, EndOfTurn)` to the reanimated creature ahead of the EOT-exile delayed trigger.
  - Faithful Mending wires `Keyword::Flashback({1}{B})`. The hand→graveyard / cast-from-graveyard flashback path was already implemented in the engine.
  - New `Effect::DiscardChosen { from, count, filter }` lets the **caster** pick from the target player's hand using a `SelectionRequirement`. AutoDecider currently picks the first matching card. Wires Inquisition of Kozilek (filter `Nonland ∧ ManaValueAtMost(3)`) and Thoughtseize (`Nonland`). Tests: `inquisition_of_kozilek_picks_low_cmc_nonland`, `thoughtseize_picks_nonland_and_costs_two_life`.
  - Plunge into Darkness modeled as `ChooseMode([SacrificeAndRemember + GainLife 3, Noop])`. Reuses the Thud sacrifice primitive. AutoDecider picks mode 0.
  - `StackItem::Spell` gained an `uncounterable: bool` field; `Effect::CounterSpell` now skips uncounterable stack items. Cavern of Souls hookup (name-a-type, per-cast tagging) is still TODO but the engine flag is in place.
- **Five-feature batch (Solitude evoke, Thud, Cosmogoyf, Faithful Mending discard, alt-cast UI)**:
  - `AlternativeCost` gained `evoke_sacrifice: bool`. `cast_spell_alternative` marks the resulting `CardInstance.evoked = true`; `resolve_top_of_stack` pushes a self-`Move`-to-graveyard trigger after the ETB triggers, so Solitude exiles its target then sacs itself. Tests: `solitude_evoke_exiles_target_then_sacrifices_self`.
  - New `Effect::SacrificeAndRemember { who, filter }` + `Value::SacrificedPower` + transient `GameState::sacrificed_power` slot (reset at the top of each `resolve_effect`). Thud is `Seq([SacrificeAndRemember(creature you control), DealDamage(SacrificedPower) → target])`. Tests: `thud_sacrifices_creature_and_deals_damage_equal_to_its_power`.
  - Cosmogoyf gets a per-frame layer-7 `SetPowerToughness(N, N+1)` injected in `compute_battlefield`, where N = `GameState::distinct_card_types_in_all_graveyards()`. Tests: `cosmogoyf_pt_scales_with_card_types_in_graveyards`.
  - Faithful Mending now does `Seq([Discard 2 (you), Draw 2, GainLife 2])` — gameplay-equivalent to "discard up to 2" for the standard line of play.
  - Client alt-cast UI: `AltCastState { pending: Option<CardId> }` resource; right-clicking a hand card with `has_alternative_cost` opens a centered modal listing every other hand card as a pitch button; clicking submits `CastSpellAlternative`. Cancel button clears state. `KnownCard.has_alternative_cost` was added to the wire and populated in `view::known_card`.
- **Delayed triggers + `PayOrLoseGame`** — added `GameState::delayed_triggers: Vec<DelayedTrigger>` plus `Effect::DelayUntil { kind, body }` and `Effect::PayOrLoseGame { mana_cost, life_cost }`. `fire_step_triggers` (called on each step transition) drains matching delayed triggers off the queue and pushes them onto the stack. The `body` runs on the trigger's controller and can reference the originally-captured `Selector::Target(0)`. `PayOrLoseGame` auto-taps mana sources, deducts life, and eliminates the controller via SBA on failure. `auto_tap_for_cost` now temporarily overrides `priority.player_with_priority` so it works correctly during trigger resolution. Wires Pact of Negation, Summoner's Pact (✅), Goryo's Vengeance (🟡 — haste still TODO). Tests: `pact_of_negation_eliminates_caster_if_unpaid_on_next_upkeep`, `pact_of_negation_lets_caster_live_when_they_can_pay`, `goryos_vengeance_exiles_creature_at_end_step`. Also extended `evaluate_requirement_static` to look in graveyards/exile so reanimate-style spells can validate their targets.
- **Alternative pitch costs (engine path)** — added `CardDefinition::alternative_cost: Option<AlternativeCost>` ({ mana, life, exile_filter }) and `GameAction::CastSpellAlternative { card_id, pitch_card, target, mode, x_value }` plus a parallel `cast_spell_alternative` handler. Force of Will and Force of Negation both expose pitch alts. Tests: `force_of_will_pitches_a_blue_card_to_counter_a_spell`, `force_of_will_rejects_non_blue_pitch_card`. **Client-side UI to choose alt-cast is still TODO** — currently only the engine path is reachable.
- **Catalog stubs** for both decks under `crabomination/src/catalog/sets/decks/{lands,creatures,spells}.rs`. All cards present with correct costs / types / P/T / keywords; many effects are `Effect::Noop` per per-card TODOs in the table above.
- **Demo wiring** — `build_demo_state` now uses the BRG (P0) and Goryo's (P1) 60-card decks.

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
