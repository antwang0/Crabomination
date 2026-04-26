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
| 4 | Chancellor of the Tangle | ⏳ | 6/7 G; opening-hand reveal adds {G} |
| 4 | Copperline Gorge | ✅ | RG fastland (same conditional ETB-tap trigger). |
| 4 | Cosmogoyf | ✅ | Dynamic P/T = (distinct card types in all graveyards) / (count + 1) via injected layer-7 `SetPowerToughness` effect at `compute_battlefield` time. |
| 2 | Darkbore Pathway | ✅ | B/G MDFC. Front face is `Darkbore Pathway` (Swamp, taps {B}); back face is `Slitherbore Pathway` (Forest, taps {G}). Played via `PlayLand(id)` (front) or `PlayLandBack(id)` (back). |
| 4 | Devourer of Destiny | 🟡 | 7/5 colorless Eldrazi. Scry-on-cast approximated as ETB Scry 2 (gameplay-equivalent except dig-past-counter). Test: `devourer_of_destiny_etb_scries_two`. |
| 4 | Gemstone Caverns | ⏳ | Opening-hand: ETB with luck counter |
| 4 | Gemstone Mine | ✅ | ETB self-source trigger adds 3 charge counters. `{T}` ability folds the cost into resolution: `RemoveCounter → AddMana(any one color) → If(charge ≤ 0, Move(Self → Graveyard))`. Taps three times then sacrifices itself. Tests: `gemstone_mine_etb_with_three_charge_counters`, `gemstone_mine_taps_three_times_then_sacrifices`. |
| 4 | Pact of Negation | ✅ | Counterspell + delayed `PayOrLoseGame` trigger on next upkeep. Auto-pays if affordable; eliminates caster otherwise. |
| 4 | Plunge into Darkness | 🟡 | `ChooseMode([SacrificeAndRemember + GainLife 3, Noop])`. Mode 0 sacrifices one creature for 3 life (instead of "any number"). Mode 1 (pay-X-life-look-at-X) still ⏳. |
| 4 | Serum Powder | ⏳ | Mulligan helper: exile hand, draw new |
| 4 | Spoils of the Vault | 🟡 | Approximated as `Search(Any → Hand) + LoseLife(3)`. Skips the reveal-until-find / variable life cost (no naming primitive yet). |
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
| 4 | Atraxa, Grand Unifier | 🟡 | 7/7 Phyrexian Praetor with flying / vigilance / deathtouch / lifelink. ETB reveal-and-sort approximated as `Draw 4` (rough average yield). |
| 1 | Cephalid Coliseum | ✅ | Tap for {U}; ETB tapped; `{2}{U}, {T}, Sacrifice: Each player draws three then discards three` wired (sacrifice modeled as the first step of the resolved effect). Threshold gate is omitted — the ability is always the post-threshold version. |
| 4 | Ephemerate | 🟡 | `Seq([Exile target your creature, Move target back to battlefield])`. ETB triggers refire (engine `place_card_in_dest::Battlefield` now calls `fire_self_etb_triggers`). Rebound (cast-from-exile next upkeep) still ⏳. |
| 4 | Faithful Mending | 🟡 | `Seq([Discard 2 (you), Draw 2, GainLife 2])` + `Keyword::Flashback({1}{B})`. "Up to two" still approximated. |
| 3 | Flooded Strand | ✅ | fetchland |
| 3 | Force of Negation | ✅ | Counter noncreature spell; alt pitch cost wired with `not_your_turn_only: true`. `cast_spell_alternative` rejects the alt cast on the caster's own turn. |
| 1 | Godless Shrine | ✅ | WB shockland: `ChooseMode([LoseLife 2, Tap This])` ETB trigger. AutoDecider picks mode 0 (pay 2 life, ETB untapped). |
| 4 | Goryo's Vengeance | ✅ | Reanimate legendary creature → grant haste until end of turn → delayed exile-at-end-step. Full Oracle. |
| 1 | Griselbrand | ⏳ | 7/7 Demon flying lifelink; pay 7 life draw 7 |
| 1 | Hallowed Fountain | ✅ | WU shockland (same `ChooseMode` pay-2-or-tap ETB trigger). |
| 1 | Island | ✅ | basic |
| 3 | Marsh Flats | ✅ | fetchland |
| 1 | Meticulous Archive | ⏳ | UW surveil land |
| 1 | Overgrown Tomb | ✅ | BG shockland (same ETB trigger). |
| 1 | Plains | ✅ | basic |
| 4 | Polluted Delta | ✅ | fetchland |
| 4 | Prismatic Ending | 🟡 | `Effect::Exile` on `target_filtered(Permanent ∧ Nonland ∧ ManaValueAtMost(1))`. Converged value pinned to 1 (one white pip); convoke + dynamic converge still ⏳. |
| 4 | Psychic Frog | ✅ | 1/3 flying. Two activated abilities: `Discard a card → +1/+1 EOT` and `Sacrifice → each opponent mills 4`. Costs are folded into the resolved effect (cost-as-first-step approximation). |
| 4 | Quantum Riddler | ✅ | UB 4/4 flying with ETB Draw 1 (approximation of "When you cast this, draw a card"). |
| 1 | Shadowy Backstreet | ⏳ | UB surveil land |
| 4 | Solitude | ✅ | 3/2 flash flying lifelink + ETB exile target opponent's creature + evoke (pitch a white card; sacrifice on ETB). "Nonwhite" filter approximated as "any creature". |
| 1 | Swamp | ✅ | basic |
| 3 | Thoughtseize | 🟡 | `Seq([DiscardChosen(EachOpponent, Nonland), LoseLife 2])`. Caster auto-picks first matching card. |
| 1 | Undercity Sewers | ⏳ | UB surveil land |
| 1 | Watery Grave | ✅ | UB shockland (same ETB trigger). |

### Goryo's sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
| 3 | Consign to Memory | ⏳ | Counter activated/triggered ability |
| 2 | Damping Sphere | ⏳ | Static: extra mana to cast 2nd+ spell each turn |
| 1 | Elesh Norn, Mother of Machines | ⏳ | Static: ETB triggers, doubles yours |
| 3 | Mystical Dispute | 🟡 | Regular cost {2}{U} counters any spell. Alt cost {U} via `AlternativeCost.target_filter: HasColor(Blue)` — engine extends `evaluate_requirement_static` to look up stack spells so `HasColor` works on a stack target. The "unless they pay {3}" rider still ⏳. Test: `mystical_dispute_alt_cost_requires_blue_target`. |
| 2 | Pest Control | 🟡 | `Effect::Destroy` on every nonland permanent with mana value ≤ 2 (the spell's pinned converged value). Convoke + variable converge still ⏳. |
| 2 | Teferi, Time Raveler | 🟡 | 4-loyalty walker. **-3** wired: `Move(target nonland opp permanent → owner's hand) + Draw 1`. +1 (sorceries-as-flash until your next turn) and the static spell-timing restriction still ⏳. Test: `teferi_minus_three_returns_target_and_draws`. |
| 2 | Wrath of the Skies | 🟡 | Cost is `{X}{W}{W}`. `ForEach(EachPermanent(Nonland))` body destroys the entity if `Value::ManaValueOf(TriggerSource) == XFromCost`. X is now threaded through `StackItem::Spell`. Convoke still ⏳. Test: `wrath_of_the_skies_destroys_permanents_with_mana_value_x`. |

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Alternative pitch costs (pay life + exile a card) | ✅ | Engine + client. Force of Will, Force of Negation, Solitude (evoke). Right-click a hand card with `has_alternative_cost` → modal lets the player pick a pitch card. `evoke_sacrifice` flag on `AlternativeCost` schedules a self-sac trigger after ETB. `not_your_turn_only` flag rejects the alt cast on the caster's own turn (Force of Negation). |
| Pact-style deferred upkeep cost | ✅ | Pact of Negation, Summoner's Pact (built on `Effect::DelayUntil` + `Effect::PayOrLoseGame`) |
| Goryo's Vengeance: reanimate-then-exile-at-EOT | ✅ | Goryo's Vengeance (uses `DelayUntil(NextEndStep)` + `Exile { Target(0) }`) |
| Rebound (cast from exile next upkeep) | ⏳ | Ephemerate |
| Flicker (exile and return to play) | ✅ | Ephemerate. `Effect::Seq([Exile target, Move target → battlefield])` paired with the new `place_card_in_dest::Battlefield` calling `fire_self_etb_triggers` so the refired ETB actually fires. |
| Convoke / Converge cost-reduction | 🟡 | Prismatic Ending, Pest Control: converged value is pinned (1 / 2 respectively). Wrath of the Skies uses X-from-cost (no convoke). Variable converge / convoke mana still ⏳. |
| Opening-hand effects (begin in play / replace draws) | ⏳ | Chancellor of the Tangle, Chancellor of the Annex, Leyline of Sanctity, Gemstone Caverns, Serum Powder |
| Reveal-and-sort ETB (one of each card type) | 🟡 | Atraxa, Grand Unifier approximates as ETB Draw 4. Real reveal-then-multi-pick (one per card type) still ⏳. |
| Static cost increase + storm tax | ⏳ | Damping Sphere |
| ETB-trigger replacement (suppress / double) | ⏳ | Elesh Norn |
| Spell-timing restriction static | ⏳ | Teferi, Time Raveler |
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Wiring on Cavern (name-a-type ETB + per-cast tagging) still TODO. |
| Counter target *ability* (not spell) | ⏳ | Consign to Memory |
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

## Implementation log (most recent first)

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
