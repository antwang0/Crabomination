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
| 4 | Blackcleave Cliffs | ✅ | BR fastland: ETB-trigger checks lands-you-control ≥ 4 (post-ETB) and self-taps. |
| 2 | Blightstep Pathway | ✅ | B/R MDFC. Front face is `Blightstep Pathway` (Swamp, taps {B}); back face is `Searstep Pathway` (Mountain, taps {R}). Played via `PlayLand(id)` (front) or `PlayLandBack(id)` (back). |
| 4 | Blooming Marsh | ✅ | BG fastland (same conditional ETB-tap trigger). |
| 1 | Callous Sell-Sword | ⏳ | R creature with X-cost on cast |
| 4 | Chancellor of the Tangle | ✅ | 6/7 G. `start_of_game_effect: AddMana(G)` adds {G} to its owner's pool when the start-of-game pass fires after mulligan. Test: `chancellor_of_the_tangle_grants_one_green_at_start_of_game`. |
| 4 | Copperline Gorge | ✅ | RG fastland (same conditional ETB-tap trigger). |
| 4 | Cosmogoyf | ✅ | Dynamic P/T = (distinct card types in all graveyards) / (count + 1) via injected layer-7 `SetPowerToughness` effect at `compute_battlefield` time. |
| 2 | Darkbore Pathway | ✅ | B/G MDFC. Front face is `Darkbore Pathway` (Swamp, taps {B}); back face is `Slitherbore Pathway` (Forest, taps {G}). Played via `PlayLand(id)` (front) or `PlayLandBack(id)` (back). |
| 4 | Devourer of Destiny | ✅ | 7/5 colorless Eldrazi. Real on-cast Scry 2 via `EventKind::SpellCast` + `EventScope::SelfSource` — the scry trigger goes on the stack above Devourer and resolves first (so a counter can't suppress the scry). Test: `devourer_of_destiny_etb_scries_two`. |
| 4 | Gemstone Caverns | ✅ | Legendary Land. `start_of_game_effect` moves it onto the controller's battlefield; taps for any-one color. Luck-counter / remove-counter-on-tap clauses are simplified — the activated ability runs indefinitely. Test: `gemstone_caverns_starts_in_play`. |
| 4 | Gemstone Mine | ✅ | ETB self-source trigger adds 3 charge counters. `{T}` ability folds the cost into resolution: `RemoveCounter → AddMana(any one color) → If(charge ≤ 0, Move(Self → Graveyard))`. Taps three times then sacrifices itself. Tests: `gemstone_mine_etb_with_three_charge_counters`, `gemstone_mine_taps_three_times_then_sacrifices`. |
| 4 | Pact of Negation | ✅ | Counterspell + delayed `PayOrLoseGame` trigger on next upkeep. Auto-pays if affordable; eliminates caster otherwise. |
| 4 | Plunge into Darkness | 🟡 | `ChooseMode([SacrificeAndRemember + GainLife 3, LoseLife 3 + Search(Any → Hand)])`. Mode 0 sacrifices one creature for 3 life. Mode 1 approximates "pay X life, look at X, pick one" with a flat 3-life tutor. AutoDecider picks mode 0. The variable-X choose-life-amount decision still ⏳. |
| 4 | Serum Powder | ⏳ | Mulligan helper: exile hand, draw new |
| 4 | Spoils of the Vault | 🟡 | Approximated as `Search(Any → Hand) + LoseLife(3)`. Skips the reveal-until-find / variable life cost (no naming primitive yet). |
| 2 | Summoner's Pact | ✅ | Search green creature into hand + delayed `PayOrLoseGame` for {2}{G}{G} on next upkeep. |
| 1 | Swamp | ✅ | basic |
| 4 | Thud | ✅ | `Seq([SacrificeAndRemember, DealDamage(SacrificedPower)])` — auto-picks first eligible creature via `AutoDecider`. Sac-as-additional-cost approximated by sac-on-resolution. |

### BRG sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
| 3 | Cavern of Souls | 🟡 | Taps for {C}. Cast paths (`cast_spell`/`cast_spell_alternative`/`cast_flashback`) now flag any creature spell as `StackItem::Spell.uncounterable = true` if the caster controls a Cavern; `CounterSpell` skips those. The "name a type" gate + mana-provenance restriction is collapsed (any Cavern protects any creature) — acceptable for the demo deck. Tests: `cavern_of_souls_makes_creatures_uncounterable`, `cavern_of_souls_does_not_protect_noncreature_spells`. |
| 4 | Chancellor of the Annex | ✅ | 5/6 W flier. `start_of_game_effect: ScheduleFirstSpellTax(EachOpponent, 1)` sets `Player.first_spell_tax_remaining = 1` on every opponent; their first cast pays {1} more (consumed on cast). The "doesn't resolve unless they pay" Oracle is collapsed to a flat cost increase. Test: `chancellor_of_the_annex_taxes_opponents_first_spell`. |
| 1 | Forest | ✅ | basic |
| 3 | Inquisition of Kozilek | ✅ | `DiscardChosen(EachOpponent, Nonland ∧ ManaValueAtMost(3))`. Engine now suspends the resolution with a `Decision::Discard` when the caster has `wants_ui`, surfacing the target's filtered hand to the picker. New client modal `spawn_discard_modal` + `handle_discard_select` lets the human click which card to take. AutoDecider still falls back to first-matching for bots. Tests: `inquisition_of_kozilek_picks_low_cmc_nonland`, `inquisition_suspends_for_caster_ui_and_applies_chosen_discard`. |
| 4 | Leyline of Sanctity | ✅ | `start_of_game_effect: Move(This → Battlefield)` puts it into play from the opening hand. New `StaticEffect::ControllerHasHexproof` makes its controller un-targetable by opponents (consulted in `check_target_legality` on `Target::Player`). Self-targets bypass. Tests: `leyline_of_sanctity_starts_in_play_and_grants_player_hexproof`, `leyline_of_sanctity_hexproof_does_not_apply_to_self`. |


### Goryo's main deck

| Count | Card | Status | Notes |
|---|---|---|---|
| 4 | Atraxa, Grand Unifier | 🟡 | 7/7 Phyrexian Praetor with flying / vigilance / deathtouch / lifelink. ETB reveal-and-sort approximated as `Draw 4` (rough average yield). |
| 1 | Cephalid Coliseum | ✅ | Tap for {U}; ETB tapped; `{2}{U}, {T}, Sacrifice: Each player draws three then discards three` wired (sacrifice modeled as the first step of the resolved effect). Threshold gate is omitted — the ability is always the post-threshold version. |
| 4 | Ephemerate | ✅ | `Seq([Exile target your creature, Move target back to battlefield])` + `Keyword::Rebound`. ETB triggers refire on flicker. Rebound: cast-from-hand spells with the keyword exile on resolution and schedule a `YourNextUpkeep` `DelayedTrigger` whose body re-runs the spell's effect with a fresh auto-target. Test: `ephemerate_rebound_exiles_then_recasts_next_upkeep`. |
| 4 | Faithful Mending | ✅ | `Seq([ChooseMode([Discard 2, Discard 1, Noop]), Draw 2, GainLife 2])` + `Keyword::Flashback({1}{B})`. Mode 0 is the full discard so AutoDecider/bot keep the gameplay-optimal pick (graveyard-fill); the UI can pick mode 1 or 2. |
| 3 | Flooded Strand | ✅ | fetchland |
| 3 | Force of Negation | ✅ | Counter noncreature spell; alt pitch cost wired with `not_your_turn_only: true`. `cast_spell_alternative` rejects the alt cast on the caster's own turn. |
| 1 | Godless Shrine | ✅ | WB shockland: `ChooseMode([LoseLife 2, Tap This])` ETB trigger. AutoDecider picks mode 0 (pay 2 life, ETB untapped). |
| 4 | Goryo's Vengeance | ✅ | Reanimate legendary creature → grant haste until end of turn → delayed exile-at-end-step. Full Oracle. |
| 1 | Griselbrand | ⏳ | 7/7 Demon flying lifelink; pay 7 life draw 7 |
| 1 | Hallowed Fountain | ✅ | WU shockland (same `ChooseMode` pay-2-or-tap ETB trigger). |
| 1 | Island | ✅ | basic |
| 3 | Marsh Flats | ✅ | fetchland |
| 1 | Meticulous Archive | ✅ | UW surveil land. ETB-tapped + Surveil 1 wired via `etb_tap_then_surveil_one` self-source ETB trigger. |
| 1 | Overgrown Tomb | ✅ | BG shockland (same ETB trigger). |
| 1 | Plains | ✅ | basic |
| 4 | Polluted Delta | ✅ | fetchland |
| 4 | Prismatic Ending | ✅ | `Keyword::Convoke`. Targets `Permanent ∧ Nonland`; resolves as `If(ManaValueOf(Target) ≤ ConvergedValue, Exile, Noop)`. Converge is computed at cast time from distinct colors of mana spent. Test: `prismatic_ending_at_converge_one_only_exiles_one_drops`. |
| 4 | Psychic Frog | ✅ | 1/3 flying. Two activated abilities: `Discard a card → +1/+1 EOT` and `Sacrifice → each opponent mills 4`. Costs are folded into the resolved effect (cost-as-first-step approximation). |
| 4 | Quantum Riddler | ✅ | UB 4/4 flying with real on-cast Draw 1 via `SpellCast`+`SelfSource`. Cantrip resolves above the spell, so it fires even if Quantum Riddler itself is countered. Tests: `quantum_riddler_on_cast_draws_a_card`, `quantum_riddler_on_cast_draws_even_if_countered`. |
| 1 | Shadowy Backstreet | ✅ | WB surveil land. ETB-tapped + Surveil 1. |
| 4 | Solitude | ✅ | 3/2 flash flying lifelink + ETB exile target opponent's creature + evoke (pitch a white card; sacrifice on ETB). "Nonwhite" filter approximated as "any creature". |
| 1 | Swamp | ✅ | basic |
| 3 | Thoughtseize | ✅ | `Seq([DiscardChosen(EachOpponent, Nonland), LoseLife 2])`. Picker UI shared with Inquisition: when the caster has `wants_ui`, the engine suspends so the human picks; bots auto-pick the first matching card. |
| 1 | Undercity Sewers | ✅ | UB surveil land. ETB-tapped + Surveil 1. |
| 1 | Watery Grave | ✅ | UB shockland (same ETB trigger). |

### Goryo's sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
| 3 | Consign to Memory | 🟡 | New `Effect::CounterAbility { what }` removes the topmost remaining `StackItem::Trigger` whose `source` matches the targeted permanent. Wired against an instant target filter of `Permanent`. The "OR counter target legendary spell" branch isn't modeled yet. Test: `consign_to_memory_counters_targeted_trigger`. |
| 2 | Damping Sphere | 🟡 | Cost-tax half wired: `StaticEffect::AdditionalCostAfterFirstSpell { filter: Any, amount: 1 }`. Cast paths consult the new `Player.spells_cast_this_turn` (per-player) and tack a generic mana onto the cost when ≥ 1. The "lands that tap for >1 mana enter producing only {C}" half is still ⏳. Tests: `damping_sphere_taxes_each_spell_after_the_first`, `damping_sphere_resets_count_at_turn_start`. |
| 1 | Elesh Norn, Mother of Machines | ✅ | Static doubling/suppression for **both** self-source ETB triggers and `AnotherOfYours` ETB triggers (the on-cast-resolution path in `stack.rs`). Multiplier helper returns 0 when an opp Norn is in play (suppress); `1 + your_norns` otherwise. Tests: `elesh_norn_doubles_your_etb_triggers`, `elesh_norn_suppresses_opponent_etb_triggers`. |
| 3 | Mystical Dispute | ✅ | Regular cost {2}{U} or alt cost {U} (alt-cost `target_filter: HasColor(Blue)` enforces "blue-only" via stack-spell predicate evaluation). Effect is `Effect::CounterUnlessPaid { what, mana_cost: {3} }` — the engine auto-pays on behalf of the targeted spell's controller; if affordable the spell stays, otherwise it's countered. Tests: `mystical_dispute_alt_cost_requires_blue_target`, `mystical_dispute_does_not_counter_when_opponent_can_pay`, `mystical_dispute_counters_when_opponent_cannot_pay`. |
| 2 | Pest Control | ✅ | `Keyword::Convoke`. `ForEach(EachPermanent(Nonland))` body destroys the iterated permanent if `ManaValueOf(it) ≤ ConvergedValue`. Test: `pest_control_at_converge_three_destroys_higher_cmc` (still passes the original case via converge=2 from {W}{B}). |
| 2 | Teferi, Time Raveler | 🟡 | 4-loyalty walker. **-3** wired: `Move(target nonland opp permanent → owner's hand) + Draw 1`. +1 (sorceries-as-flash until your next turn) and the static spell-timing restriction still ⏳. Test: `teferi_minus_three_returns_target_and_draws`. |
| 2 | Wrath of the Skies | ✅ | `{X}{W}{W}` + `Keyword::Convoke`. `ForEach(EachPermanent(Nonland))` body destroys the entity if `ManaValueOf(TriggerSource) == XFromCost`. Convoke creatures pay the X-portion as generic via tap. Tests: `wrath_of_the_skies_destroys_permanents_with_mana_value_x`, `convoke_taps_creature_to_pay_one_generic`. |

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Alternative pitch costs (pay life + exile a card) | ✅ | Engine + client. Force of Will, Force of Negation, Solitude (evoke). Right-click a hand card with `has_alternative_cost` → modal lets the player pick a pitch card. `evoke_sacrifice` flag on `AlternativeCost` schedules a self-sac trigger after ETB. `not_your_turn_only` flag rejects the alt cast on the caster's own turn (Force of Negation). |
| Pact-style deferred upkeep cost | ✅ | Pact of Negation, Summoner's Pact (built on `Effect::DelayUntil` + `Effect::PayOrLoseGame`) |
| Goryo's Vengeance: reanimate-then-exile-at-EOT | ✅ | Goryo's Vengeance (uses `DelayUntil(NextEndStep)` + `Exile { Target(0) }`) |
| Rebound (cast from exile next upkeep) | ✅ | Ephemerate. New `Keyword::Rebound` + `CardInstance.cast_from_hand` flag (set by `cast_spell` and `cast_spell_alternative`). `continue_spell_resolution` checks for the combo: rebound spells go to exile and register a `YourNextUpkeep` delayed trigger whose body is the spell's own effect. The fire path now auto-targets when the trigger has no captured target, so the rebound recast picks a fresh creature each fire. Modeled as effect-replay rather than true cast-from-exile (no on-cast triggers from the recast), which is gameplay-equivalent for Ephemerate. |
| Flicker (exile and return to play) | ✅ | Ephemerate. `Effect::Seq([Exile target, Move target → battlefield])` paired with the new `place_card_in_dest::Battlefield` calling `fire_self_etb_triggers` so the refired ETB actually fires. |
| Convoke / Converge cost-reduction | ✅ | New `GameAction::CastSpellConvoke { card_id, target, mode, x_value, convoke_creatures }` + internal `cast_spell_with_convoke`. Each listed creature is validated (untapped, controlled by caster, spell has `Keyword::Convoke`), tapped, and contributes {1} generic mana to the player's pool before paying the cost. Converge is computed by snapshotting the mana pool before paying and counting distinct colors that decreased; the count is stashed on `StackItem::Spell.converged_value` and threaded through to `EffectContext`. New `Value::ConvergedValue` reads it inside the spell's effect. Currently: convoke pips contribute generic only (don't raise converge). Used by Prismatic Ending, Pest Control, Wrath of the Skies. |
| Opening-hand effects (begin in play / replace draws) | 🟡 | New `CardDefinition.start_of_game_effect: Option<Effect>`. `advance_mulligan` fires every player's hand-cards' effects after all keeps. `move_card_to` extended to find cards in hand so `Move(This → Battlefield)` works for hand cards. Wired: Leyline of Sanctity, Gemstone Caverns, Chancellor of the Tangle, Chancellor of the Annex. Serum Powder's mulligan-time exile-and-redraw still ⏳ (different mechanic — happens during mulligan rather than after it). |
| Reveal-and-sort ETB (one of each card type) | 🟡 | Atraxa, Grand Unifier approximates as ETB Draw 4. Real reveal-then-multi-pick (one per card type) still ⏳. |
| Static cost increase + storm tax | 🟡 | Damping Sphere's "second-and-onwards spells cost {1} more" wired via `StaticEffect::AdditionalCostAfterFirstSpell { filter, amount }` + new per-player `Player.spells_cast_this_turn` counter (reset on `do_untap`). Cast paths walk active static abilities and tack the tax onto the cost. Storm-style scaling and Damping Sphere's land-output clause still ⏳. |
| ETB-trigger replacement (suppress / double) | ✅ | Elesh Norn, Mother of Machines via `etb_trigger_multiplier(state, etb_controller)` — returns 0 (any opp Norn → suppress) or `1 + your_norns` (each Norn on your side adds an extra fire). Hooked into both `fire_self_etb_triggers` and the cast-resolution ETB push paths in `stack.rs` (self-source AND `AnotherOfYours`). `continue_trigger_resolution` re-picks a fresh auto-target if the stored one is no longer valid, so doubled triggers don't all fizzle on a single shared target. |
| Spell-timing restriction static | ⏳ | Teferi, Time Raveler |
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Cavern of Souls now flags any creature spell its controller casts as uncounterable (approximation collapses "name a type" + mana provenance into "you control a Cavern → your creatures are uncounterable"). |
| Counter target *ability* (not spell) | 🟡 | New `Effect::CounterAbility { what }` removes the topmost matching `StackItem::Trigger` from the stack. Used by Consign to Memory. Targeting is single-source-permanent; multi-trigger picker UI still ⏳. |
| Charge-counter mana sources w/ self-sac | ✅ | Gemstone Mine. Activated ability folds the counter-removal cost into resolution and tail-checks `CountersOn(This, Charge) ≤ 0` to schedule a self-sac. |
| Shock-land ETB choice (tapped or 2 life) | ✅ | Godless Shrine, Hallowed Fountain, Watery Grave, Overgrown Tomb. ETB trigger is a `ChooseMode([LoseLife 2, Tap This])`; AutoDecider picks pay-2-life. Note: triggered ability, not a true replacement effect — the land is briefly available untapped before the trigger resolves. |
| Pathway / modal DFC mana abilities | ✅ | Blightstep Pathway, Darkbore Pathway. `CardDefinition.back_face: Option<Box<CardDefinition>>` carries the alternate face. `GameAction::PlayLandBack(CardId)` swaps the `CardInstance.definition` to the back face's definition before placing on battlefield, so all subsequent abilities/types come from the back. **Client-side flip UI**: right-click an MDFC hand card to toggle to its back face — the front-face mesh repaints with the back-face's Scryfall image and the next left-click submits `PlayLandBack` instead of `PlayLand`. (Bot still defaults to the front face.) |
| Surveil-land ETB-tapped + surveil 1 | ✅ | Meticulous Archive, Undercity Sewers, Shadowy Backstreet. `play_land` now fires self-source ETB triggers via `fire_self_etb_triggers` (lands skip the stack, so this site needs a hardcoded fire). |
| Fastland conditional ETB-tapped | ✅ | Blackcleave Cliffs, Blooming Marsh, Copperline Gorge. ETB trigger uses `Effect::If` over `Predicate::SelectorCountAtLeast` of "lands you control" (≥ 4 post-ETB). |
| Activated land mill (Cephalid Coliseum) | ✅ | Cephalid Coliseum: ActivatedAbility(`{2}{U}, {T}`, sacrifice-as-first-effect-step, then `Draw 3` and `Discard 3` for `EachPlayer`). |
| Tarmogoyf-style P/T from graveyard | ✅ | Cosmogoyf (via inline `compute_battlefield` injection of a layer-7 set-PT effect with the live graveyard card-type count). |
| X-cost creature side-effects | ⏳ | Callous Sell-Sword |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`; Plunge into Darkness still ⏳. |
| Reveal-until-find search | 🟡 | Spoils of the Vault approximates as `Search(Any → Hand) + LoseLife(3)`. Naming primitive + reveal-until-find loop still ⏳. |
| Loyalty abilities w/ static | 🟡 | Teferi, Time Raveler **-3** wired (bounce + draw). +1 (sorcery-as-flash) and the static spell-timing veto still ⏳. |
| On-cast self triggers ("when you cast this …") | ✅ | Devourer of Destiny (Scry 2 on cast), Quantum Riddler (Draw 1 on cast). Each cast path (`cast_spell`, `cast_spell_alternative`, `cast_flashback`) collects `EventKind::SpellCast` + `EventScope::SelfSource` triggers off the just-cast card and pushes them onto the stack **above** the spell, so the trigger resolves first (and still fires if the spell itself is countered in response). |

## Implementation log (most recent first)

- **Inquisition of Kozilek + Thoughtseize UI picker**:
  - **Engine**: `Effect::DiscardChosen` now builds a `Decision::Discard { player: caster, count, hand: target's filtered hand }` when resolving. If the caster has `wants_ui`, resolution suspends via `suspend_signal` with new `PendingEffectState::DiscardChosenPending { target_player }`. `apply_pending_effect_answer` consumes the `DecisionAnswer::Discard(Vec<CardId>)`, removing each chosen card from the target's hand and adding it to their graveyard. Bots still hit the AutoDecider path (first matching card).
  - **Client**: new `DiscardSelectButton` marker, `DecisionUiState.discard_selected: Vec<CardId>`, and `DecisionKey::Discard(Vec<CardId>, u32)` fingerprint. New `spawn_discard_modal` renders the target's hand as Scryfall-textured tiles; clicking toggles selection up to `count`. `handle_discard_select` updates the highlight state. `handle_confirm` already builds `DecisionAnswer::Discard(state.discard_selected.clone())` once `count` cards are selected.
  - Test: `inquisition_suspends_for_caster_ui_and_applies_chosen_discard` exercises the full suspend/submit/apply roundtrip with a deliberately mixed target hand (lands + low-CMC nonlands + a high-CMC nonland) to confirm filtering works through the new path.
- **Opening-hand effects** (Leyline of Sanctity, Gemstone Caverns, Chancellor of the Tangle, Chancellor of the Annex):
  - **Engine plumbing**: New `CardDefinition.start_of_game_effect: Option<Effect>`. New `GameState::fire_start_of_game_effects` walks every player's opening hand and resolves each card's effect with `controller = card.owner` and `ctx.source = card.id` so `Selector::This` resolves to the hand card. `advance_mulligan` calls it once when all players have kept. `move_card_to` was extended to find cards in hand as the final fallback so `Move(This → Battlefield)` works for hand-resident cards. Workspace-wide sed propagated `start_of_game_effect: None,` to every `CardDefinition` literal.
  - **`StaticEffect::ControllerHasHexproof`** + player-target hexproof check: `check_target_legality` now reshaped to handle `Target::Player`, walking battlefield static abilities for hexproof granted to the player. Self-targets bypass.
  - **`Effect::ScheduleFirstSpellTax { who, amount }`** + new `Player.first_spell_tax_remaining: u32`: Chancellor-of-the-Annex-style "next spell costs {N} more" — consumed on the next cast via `extra_cost_for_spell` and cleared on payment.
  - **Plunge into Darkness mode 1**: now `LoseLife(3) + Search(Any → Hand)` — flat-3 approximation of "pay X life, look at X, pick one".
  - Tests: `leyline_of_sanctity_starts_in_play_and_grants_player_hexproof`, `leyline_of_sanctity_hexproof_does_not_apply_to_self`, `chancellor_of_the_tangle_grants_one_green_at_start_of_game`, `gemstone_caverns_starts_in_play`, `chancellor_of_the_annex_taxes_opponents_first_spell`.
- **Convoke + Converge** (Prismatic Ending, Pest Control, Wrath of the Skies):
  - **Convoke**: new `GameAction::CastSpellConvoke { card_id, target, mode, x_value, convoke_creatures }` dispatches to `cast_spell_with_convoke`. Each convoke creature must be untapped + controlled by the caster + the spell must have `Keyword::Convoke`; the helper taps each and adds {1} colorless to the caster's pool before the regular cost-payment loop. Convoke creatures contribute generic only (full Oracle also allows one mana of the creature's color identity — collapsed to generic for simplicity). Existing `cast_spell` is now a thin wrapper that calls in with `&[]`.
  - **Converge**: new `converge_count(before, after)` helper compares two `ManaPool` snapshots and returns the count of distinct colors that decreased. `cast_spell_with_convoke` runs it after `pay()` succeeds and stashes the result on `StackItem::Spell.converged_value`. `EffectContext` gained the field (with a new `for_spell_full` constructor) and `Value::ConvergedValue` reads it. `ResumeContext::Spell` carries it through suspend/resume.
  - **Cards**: Wrath of the Skies, Prismatic Ending, Pest Control all gained `Keyword::Convoke`. Pest Control is now `ForEach(Nonland)` + `If(ManaValueOf(it) ≤ ConvergedValue, Destroy)`. Prismatic Ending has cast-time filter `Permanent ∧ Nonland` (no CMC constraint) and resolution-time `If(ManaValueOf(Target) ≤ ConvergedValue, Exile)`. Existing `prismatic_ending_exiles_one_drop_only` test was updated to reflect the new "cast accepts any nonland; resolution gates by converge" semantics. Tests: `convoke_taps_creature_to_pay_one_generic`, `convoke_rejects_tapped_creature`, `pest_control_at_converge_three_destroys_higher_cmc`, `prismatic_ending_at_converge_one_only_exiles_one_drops`.
- **Finishing 🟡 cards: Ephemerate rebound, Mystical Dispute "unless pay {3}", Elesh Norn AnotherOfYours**:
  - **Ephemerate rebound**: new `Keyword::Rebound` + `CardInstance.cast_from_hand` flag (set by `cast_spell` and `cast_spell_alternative`; `cast_flashback` deliberately does not). `continue_spell_resolution` detects the combo and (instead of graveyarding) exiles the card and pushes a `YourNextUpkeep` `DelayedTrigger` whose body is the spell's effect, with `target: None` so it re-picks at fire time. The delayed-trigger fire path in `stack.rs` now auto-targets via `auto_target_for_effect` when the registered target is None. Test: `ephemerate_rebound_exiles_then_recasts_next_upkeep`.
  - **Mystical Dispute unless-pay-{3}**: new `Effect::CounterUnlessPaid { what, mana_cost }`. At resolution the engine temporarily overrides `priority.player_with_priority` to the targeted spell's controller and tries to auto-tap + pay; if it succeeds the spell stays, otherwise it's countered. Tests: `mystical_dispute_does_not_counter_when_opponent_can_pay`, `mystical_dispute_counters_when_opponent_cannot_pay`.
  - **Elesh Norn `AnotherOfYours` ETBs**: extended the cast-resolution AnotherOfYours push site in `stack.rs` to also clone triggers `etb_trigger_multiplier(self, caster)` times — same multiplier logic that already governed self-source ETBs, since the listener's controller is the caster.
- **Elesh Norn ETB-trigger replacement**:
  - New `actions::etb_trigger_multiplier(state, etb_controller)` returns the per-trigger fire count: 0 if any opponent of `etb_controller` has an Elesh Norn (suppressed), otherwise `1 + your_norns`. Both ETB push sites — `fire_self_etb_triggers` (used by `play_land` and battlefield-zone moves) and `resolve_top_of_stack`'s cast path — clone the trigger that many times.
  - `continue_trigger_resolution` now re-picks a fresh `auto_target` when the trigger's stored target is no longer legal at resolution time (e.g. the prior copy of an Elesh-Norn-doubled Solitude ETB just exiled the first creature — the second copy retargets to the next eligible). Without this, doubled targeted triggers would fizzle on the same shared target. The check uses `target_filter_for_slot(0)` against the live battlefield state.
  - Covers self-source ETBs only; the `AnotherOfYours` scope path in `stack.rs` still pushes once. Tests: `elesh_norn_doubles_your_etb_triggers`, `elesh_norn_suppresses_opponent_etb_triggers`.
- **Damping Sphere + Cavern of Souls + Consign to Memory**:
  - **Damping Sphere cost-tax**: new `StaticEffect::AdditionalCostAfterFirstSpell { filter, amount }` + per-player `Player.spells_cast_this_turn` (reset in `do_untap` alongside `lands_played_this_turn`). New `extra_cost_for_spell` helper walks every battlefield permanent's static abilities and totals the tax for the cast spell when the caster has already cast ≥ 1 spell this turn. `cast_spell` adds the tax as a generic mana symbol on the cost before paying. Tests: `damping_sphere_taxes_each_spell_after_the_first`, `damping_sphere_resets_count_at_turn_start`. The "lands that tap for >1 mana enter producing only {C}" half of the Oracle isn't modeled (no land-tap-output replacement primitive yet). The tax currently only applies on `cast_spell`, not on `cast_spell_alternative`/`cast_flashback`.
  - **Cavern of Souls uncounterable wiring**: new `GameState::caster_grants_uncounterable(caster, card)` returns true if the caster controls any "Cavern of Souls" and the cast card is a creature. All three cast paths (`cast_spell`, `cast_spell_alternative`, `cast_flashback`) consult it and stamp `StackItem::Spell.uncounterable = true`. The existing `CounterSpell` resolver already skipped uncounterable items, so creatures cast under a Cavern just stop being legal counter targets. The "name a type" decision and the mana-provenance gate are skipped — any Cavern protects any creature you cast. Tests: `cavern_of_souls_makes_creatures_uncounterable`, `cavern_of_souls_does_not_protect_noncreature_spells`.
  - **Consign to Memory + `Effect::CounterAbility`**: new effect variant that takes a `what: Selector` resolving to a permanent (the ability's source) and removes the topmost-on-stack `StackItem::Trigger` whose `source` matches. Walks the stack top-down so the most recent matching trigger is the one countered. Consign to Memory targets `Permanent` and uses this effect. The "OR counter target legendary spell" half of Consign's Oracle isn't modeled. Test: `consign_to_memory_counters_targeted_trigger` exercises Devourer of Destiny's on-cast Scry trigger getting countered while Devourer itself still resolves.
- **On-cast self triggers + Faithful Mending up-to-2 + surveil-land status reconciliation**:
  - **Engine: on-cast self triggers**. New `collect_self_cast_triggers(card)` helper + `GameState::push_on_cast_triggers`. Each cast path (`cast_spell`, `cast_spell_alternative`, `cast_flashback`) now pulls the just-cast card's `EventKind::SpellCast` + `EventScope::SelfSource` triggers and pushes them onto the stack above the spell — they resolve before the spell itself, and still fire when the spell is countered in response. Promotes Devourer of Destiny (🟡 → ✅, real Scry-on-cast instead of ETB approximation) and Quantum Riddler (cantrip is now actually on-cast, not ETB). Test: `quantum_riddler_on_cast_draws_even_if_countered` exercises the counter-the-spell-but-trigger-still-resolves path. The existing Devourer Scry test is unchanged (the order of effects against an empty board is gameplay-equivalent between ETB and on-cast).
  - **Faithful Mending → ✅**: discard step now exposes a `ChooseMode([Discard 2, Discard 1, Noop])` so the player can pick how many to dump. Mode 0 is the full discard, so AutoDecider/bot keep the optimal play (graveyard fill).
  - **Surveil-land table reconciliation**: Meticulous Archive, Shadowy Backstreet, Undercity Sewers were already wired (since the lands batch added `etb_tap_then_surveil_one`) but still listed as ⏳; promoted to ✅.
- **Gemstone Mine + Teferi -3 + Mystical Dispute alt cost**:
  - **Gemstone Mine**: ETB self-source trigger adds 3 charge counters. `{T}` ability is `Seq([RemoveCounter(This, Charge, 1), AddMana(AnyOneColor), If(CountersOn(This, Charge) ≤ 0, Move(This → Graveyard))])` — counter-removal cost is folded into the resolved effect (cost-as-first-step), and the tail check sacrifices the land when the last counter comes off. Natural progression: 3 counters → 3 taps → sac. Tests: `gemstone_mine_etb_with_three_charge_counters`, `gemstone_mine_taps_three_times_then_sacrifices`.
  - **Teferi, Time Raveler -3**: `LoyaltyAbility { loyalty_cost: -3, effect: Seq([Move(target nonland opp permanent → owner's hand), Draw 1]) }`. The static spell-timing restriction and the +1 (sorcery-as-flash) still need engine support — the -3 is the first interactive Teferi ability that actually fires. Test: `teferi_minus_three_returns_target_and_draws`. Required engine fix below.
  - **Engine: zone-dest player resolution**: `move_card_to` now pre-flattens any selector-based `PlayerRef` (`OwnerOf`/`ControllerOf`) inside the destination `ZoneDest` against the active ctx **before** removing the card from its source zone. Previously `place_card_in_dest` built a bare ctx and resolved `OwnerOf(Selector::Target(0))` against an empty target list, so bouncing a target permanent to its owner's hand silently lost the owner. New `PlayerRef::Seat(usize)` is used as the flattened, ctx-independent encoding.
  - **Engine: stack-spell predicate evaluation**: `evaluate_requirement_static` falls through to the stack (in addition to battlefield → graveyards → exile) when looking up the targeted card. Lets `HasColor`/`HasCardType`/etc. read from a stack spell — needed for Mystical Dispute's "alt cost only if blue".
  - **AlternativeCost.target_filter**: new `Option<SelectionRequirement>` field that adds an extra target check applied only on the alt-cast path. Mystical Dispute's regular {2}{U} counters any spell; via alt cost {U} it requires `HasColor(Blue)`. Test: `mystical_dispute_alt_cost_requires_blue_target`.
- **MDFC pathways (Blightstep / Darkbore)**:
  - **Engine plumbing**: New `CardDefinition.back_face: Option<Box<CardDefinition>>` carries the alternate face's full definition. New `GameAction::PlayLandBack(CardId)` plays the card via its back face — `play_land_with_face(card_id, back_face=true)` swaps `CardInstance.definition` to the back face before pushing onto the battlefield, so all subsequent abilities, types, and mana abilities are the back's. ETB triggers fire via the existing `fire_self_etb_triggers` hook against the swapped definition. Default `PlayLand(id)` continues to play the front face (no behavior change for non-MDFC cards).
  - **Catalog**: Pathways now expose only the front face's land type / single-color mana ability via a new `pathway_face` builder; the front's `back_face` is set to a separately-built back face. Blightstep Pathway → Searstep Pathway (Swamp/B → Mountain/R); Darkbore Pathway → Slitherbore Pathway (Swamp/B → Forest/G). Tests: `pathway_front_face_taps_for_front_color_only`, `pathway_back_face_taps_for_back_color_only`, `play_land_back_rejects_non_mdfc`.
  - **Wire**: `KnownCard.back_face_name: Option<String>` populated from `card.definition.back_face.as_ref().map(|b| b.name)`. The viewer can use it to render a "flip" affordance on hand cards.
  - **Client UI**: New `FlippedHandCards` resource tracks which hand cards the viewer flipped. Right-click on an MDFC hand card (no alt cost) toggles membership; left-click on a flipped land submits `PlayLandBack` instead of `PlayLand`. New `FrontFaceMesh` marker component on the front child mesh lets `sync_flipped_hand_cards` find and repaint just the front face when the flip state changes — the card's Scryfall image swaps to the back face's image. Stale flip entries are dropped automatically when a card leaves the viewer's hand.
- **Wrath of the Skies + Spoils of the Vault + Atraxa approximations**:
  - **Engine plumbing**: `StackItem::Spell` now carries `x_value: u32`, set from the `CastSpell` / `CastSpellAlternative` / `CastFlashback` actions and threaded into `EffectContext.x_value` at resolution time. Previously `continue_spell_resolution` hard-coded `x_value: 0`, so `Value::XFromCost` always read 0 inside spell effects. `ResumeContext::Spell` also gained the field so suspend/resume preserves it.
  - **New `Value::ManaValueOf(Box<Selector>)`** — evaluates to the CMC of the first card the selector resolves to (battlefield → graveyard → hand → library → exile fallback). Lets effects filter / branch on a permanent's mana value at runtime instead of via a fixed `SelectionRequirement::ManaValueAtMost(u32)`.
  - **Wrath of the Skies**: cost set to `{X}{W}{W}`. Effect is `ForEach(EachPermanent(Nonland))` that destroys the iterated permanent when `ManaValueOf(TriggerSource) == XFromCost` (modeled via `Predicate::All([ValueAtLeast, ValueAtMost])` since there's no `ValueEquals` primitive). Convoke still ⏳. Test: `wrath_of_the_skies_destroys_permanents_with_mana_value_x`.
  - **Spoils of the Vault**: approximated as `Seq([Search(Any → Hand), LoseLife 3])` — the caster picks any library card directly, and the variable life cost is flattened to 3 (rough average reveal count for a 60-card deck). Test: `spoils_of_the_vault_tutors_and_loses_three_life`.
  - **Atraxa, Grand Unifier**: ETB approximated as `Draw 4` (rough yield of "reveal top 10, take one of each card type" in a typical reanimator pile). Test: `atraxa_grand_unifier_etb_draws_four`.
- **Combined claude-branches batch (lands + cantrips + flicker + sweepers + Force of Negation timing)**:
  - **Engine plumbing**: New `GameState::fire_self_etb_triggers(card_id, controller)` helper. `play_land` and `place_card_in_dest::Battlefield` both call it so triggered abilities on lands and on flickered/reanimated creatures actually fire. `place_card_in_dest::Battlefield` also clears damage / pump bonuses / `attached_to` so a permanent re-entering the battlefield is the brand-new object MTG rule 400.7 demands.
  - **Fastlands** (Blackcleave Cliffs, Blooming Marsh, Copperline Gorge): ETB trigger uses `Effect::If` over `Predicate::SelectorCountAtLeast` (≥ 4 lands-you-control, post-ETB). Tests: `fastland_enters_untapped_with_few_lands`, `fastland_enters_tapped_with_many_lands`.
  - **Shocklands** (Godless Shrine, Hallowed Fountain, Watery Grave, Overgrown Tomb): self-source ETB `ChooseMode([LoseLife 2, Tap This])`. AutoDecider picks mode 0 = pay 2 life. Tests: `watery_grave_pays_two_life_and_stays_untapped`, `shockland_enters_untapped_paying_2_life`.
  - **Surveil lands** now actually fire their `etb_tap_then_surveil_one` triggered ability (previously silent because lands skipped the cast-resolution path).
  - **Cephalid Coliseum**: second ActivatedAbility `{2}{U}, {T}, Sacrifice → each player draws three then discards three`. Sac modeled as the first step of the resolved effect. Test: `cephalid_coliseum_sacrifices_for_each_player_to_draw_then_discard_three`.
  - **Quantum Riddler**: ETB Draw 1 (approximation of the on-cast cantrip). Test: `quantum_riddler_etb_draws_a_card`.
  - **Psychic Frog**: two ActivatedAbilities — `Discard a card → +1/+1 EOT` and `Sacrifice → each opponent mills 4`. Costs folded into resolution. Tests: `psychic_frog_discard_pumps_until_end_of_turn`, `psychic_frog_sacrifice_mills_each_opponent_four`.
  - **Ephemerate**: `Seq([Exile target your creature, Move target back to battlefield])`. Refires the creature's self-source ETB triggers via the new battlefield-entry hook. Rebound still ⏳. Tests: `ephemerate_flickers_target_creature_back_to_battlefield`, `ephemerate_refires_solitude_etb_via_place_card_on_battlefield`.
  - **Pest Control**: `Effect::Destroy` over every nonland permanent with `ManaValueAtMost(2)`. Test: `pest_control_destroys_low_cmc_nonland_permanents`.
  - **Prismatic Ending**: `Effect::Exile` on `target_filtered(Permanent ∧ Nonland ∧ ManaValueAtMost(1))`. Test: `prismatic_ending_exiles_one_drop_only`.
  - **Force of Negation** alt cost: added `AlternativeCost.not_your_turn_only: bool`. `cast_spell_alternative` rejects the alt cast when `active_player_idx == caster`. Tests: `force_of_negation_alt_cost_blocked_on_your_turn`, `force_of_negation_alt_cost_works_on_opponents_turn`.
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
