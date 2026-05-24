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
| 1 | Callous Sell-Sword | 🟡 | 4/4 with ETB "sacrifice a creature; +(sacrificed power)/+0 EOT". Casualty 2 "copy this spell" half omitted (no copy primitive yet). |
| 4 | Chancellor of the Tangle | ✅ | 6/7 G. Opening-hand reveal registers a `YourNextMainPhase` delayed trigger that adds {G} on the controller's first main step (mana pools persist into main, so it can be spent that turn). |
| 4 | Copperline Gorge | ✅ | RG fastland (same conditional ETB-tap trigger). |
| 4 | Cosmogoyf | ✅ | Dynamic P/T = (distinct card types in all graveyards) / (count + 1) via injected layer-7 `SetPowerToughness` effect at `compute_battlefield` time. |
| 2 | Darkbore Pathway | ✅ | B/G MDFC. Front face is `Darkbore Pathway` (Swamp, taps {B}); back face is `Slitherbore Pathway` (Forest, taps {G}). Played via `PlayLand(id)` (front) or `PlayLandBack(id)` (back). |
| 4 | Devourer of Destiny | ✅ | 7/5 colorless Eldrazi. Real on-cast Scry 2 via `EventKind::SpellCast` + `EventScope::SelfSource` — the scry trigger goes on the stack above Devourer and resolves first (so a counter can't suppress the scry). Test: `devourer_of_destiny_etb_scries_two`. |
| 4 | Gemstone Caverns | ✅ | Opening-hand: starts in play untapped with one luck counter (modeled as a Charge counter — gameplay-equivalent here since only the mana ability reads it). Two activated abilities: `{T}: Add {C}` and `{T}, RemoveCounter Luck → Add 1 of any color`. |
| 4 | Gemstone Mine | ✅ | ETB self-source trigger adds 3 charge counters. `{T}` ability folds the cost into resolution: `RemoveCounter → AddMana(any one color) → If(charge ≤ 0, Move(Self → Graveyard))`. Taps three times then sacrifices itself. Tests: `gemstone_mine_etb_with_three_charge_counters`, `gemstone_mine_taps_three_times_then_sacrifices`. |
| 4 | Pact of Negation | ✅ | Counterspell + delayed `PayOrLoseGame` trigger on next upkeep. Auto-pays if affordable; eliminates caster otherwise. |
| 4 | Plunge into Darkness | 🟡 | `ChooseMode([SacrificeAndRemember + GainLife 3, LoseLife 4 + Search → Hand])`. Mode 0 sacrifices one creature for 3 life. Mode 1 pays 4 life and tutors any card (approximation of "pay X life, look at top X, take one"). Tests: `plunge_into_darkness_mode_one_pays_four_life_and_tutors`. |
| 4 | Serum Powder | ✅ | Mulligan helper: opening-hand reveals `MulliganHelper`, surfaced via `Decision::Mulligan.serum_powders`. Answering with `DecisionAnswer::SerumPowder(id)` exiles the entire hand and deals a fresh 7. Doesn't bump the London-mulligan ladder, so multiple powders stack. {3} artifact, {T}: Add {1} otherwise. |
| 4 | Spoils of the Vault | 🟡 | Wired to `Effect::RevealUntilFind { find: Any, cap: 10, life_per_revealed: 1 }`: walks the top of the library until a match (or cap), mills the misses, deducts 1 life per revealed card. With `find: Any` the very first card is taken (1 life). The "name a card" half is still pending a naming primitive. |
| 2 | Summoner's Pact | ✅ | Search green creature into hand + delayed `PayOrLoseGame` for {2}{G}{G} on next upkeep. |
| 1 | Swamp | ✅ | basic |
| 4 | Thud | ✅ | `Seq([SacrificeAndRemember, DealDamage(SacrificedPower)])` — auto-picks first eligible creature via `AutoDecider`. Sac-as-additional-cost approximated by sac-on-resolution. |

### BRG sideboard

| Count | Card | Status | Notes |
|---|---|---|---|
| 3 | Cavern of Souls | ✅ | Taps for {C}. New `Effect::NameCreatureType { what }` + `Decision::ChooseCreatureType` + `CardInstance.chosen_creature_type: Option<CreatureType>` field. ETB self-source trigger fires `NameCreatureType { what: This }`; `AutoDecider` picks Demon (matches Griselbrand). `caster_grants_uncounterable` only protects creature spells whose type matches the Cavern's named type — Caverns whose ETB hasn't yet resolved (`chosen_creature_type = None`) keep the legacy "any creature" behaviour as a fallback for fixture tests built via `add_card_to_battlefield`. Tests: `cavern_of_souls_makes_creatures_uncounterable`, `cavern_of_souls_does_not_protect_noncreature_spells`, `cavern_of_souls_only_protects_named_creature_type`, `cavern_of_souls_etb_picks_creature_type_via_decider`. The mana-provenance restriction (must spend mana from this Cavern) is still collapsed. |
| 4 | Chancellor of the Annex | ✅ | 5/6 W flying. Opening-hand reveal registers a `YourNextMainPhase` delayed trigger whose body `AddFirstSpellTax(EachOpponent, 1)` stamps one tax charge on each opponent. Each charge taxes that player's next spell {1} more (consumed at cast time via `consume_first_spell_tax`). The "doesn't resolve unless they pay" half collapses to "costs {1} more" — auto-applied; if they can't pay the cast fails outright. |
| 1 | Forest | ✅ | basic |
| 3 | Inquisition of Kozilek | 🟡 | `DiscardChosen(EachOpponent, Nonland ∧ ManaValueAtMost(3))`. Caster auto-picks first matching card. UI for the human picker still TODO. |
| 4 | Leyline of Sanctity | ✅ | Opening-hand: starts in play under owner's control via `OpeningHandEffect::StartInPlay`. New `StaticEffect::ControllerHasHexproof` + `player_has_static_hexproof` check in `check_target_legality` rejects opponent-controlled `Target::Player(_)` aimed at the leyline's controller. Tests: `leyline_of_sanctity_starts_in_play_after_mulligan`, `leyline_of_sanctity_grants_player_hexproof`. |


### Goryo's main deck

| Count | Card | Status | Notes |
|---|---|---|---|
| 4 | Atraxa, Grand Unifier | 🟡 | 7/7 Phyrexian Praetor with flying / vigilance / deathtouch / lifelink. ETB now uses `Value::DistinctTypesInTopOfLibrary { who: You, count: 10 }` — counts actual distinct card types in the top 10 of the controller's library and draws that many cards (instead of a flat 4). Reordering after the reveal is still flattened. |
| 1 | Cephalid Coliseum | ✅ | Tap for {U}; ETB tapped; `{2}{U}, {T}, Sacrifice: Each player draws three then discards three` wired (sacrifice modeled as the first step of the resolved effect). Threshold gate is omitted — the ability is always the post-threshold version. |
| 4 | Ephemerate | ✅ | `Seq([Exile target your creature, Move target back to battlefield])` + `Keyword::Rebound`. ETB triggers refire on flicker. Rebound: cast-from-hand spells with the keyword exile on resolution and schedule a `YourNextUpkeep` `DelayedTrigger` whose body re-runs the spell's effect with a fresh auto-target. Test: `ephemerate_rebound_exiles_then_recasts_next_upkeep`. |
| 4 | Faithful Mending | ✅ | `Seq([ChooseMode([Discard 2, Discard 1, Noop]), Draw 2, GainLife 2])` + `Keyword::Flashback({1}{B})`. Mode 0 is the full discard so AutoDecider/bot keep the gameplay-optimal pick (graveyard-fill); the UI can pick mode 1 or 2. |
| 3 | Flooded Strand | ✅ | fetchland |
| 3 | Force of Negation | ✅ | Counter noncreature spell; alt pitch cost wired with `not_your_turn_only: true`. `cast_spell_alternative` rejects the alt cast on the caster's own turn. |
| 1 | Godless Shrine | ✅ | WB shockland: `ChooseMode([LoseLife 2, Tap This])` ETB trigger. AutoDecider picks mode 0 (pay 2 life, ETB untapped). |
| 4 | Goryo's Vengeance | ✅ | Reanimate legendary creature → grant haste until end of turn → delayed exile-at-end-step. Full Oracle. |
| 1 | Griselbrand | ✅ | 7/7 Legendary Demon, flying + lifelink. Activated ability `Pay 7 life: Draw 7 cards` modeled as `Seq([LoseLife 7, Draw 7])`. |
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
| 3 | Consign to Memory | ✅ | `ChooseMode([CounterAbility, CounterSpell(IsSpellOnStack ∧ Legendary)])`. Mode 0 = counter target activated/triggered ability via `Effect::CounterAbility`; mode 1 = counter target legendary spell via `Effect::CounterSpell` over a Legendary-supertype filter. AutoDecider picks mode 0 (the Goryo's-matchup default); UI/bot can pick mode 1 with `mode: Some(1)`. Tests: `consign_to_memory_counters_targeted_trigger`, `consign_to_memory_mode_one_counters_legendary_spell`. |
| 2 | Damping Sphere | ✅ | Both halves wired: (1) `AdditionalCostAfterFirstSpell` taxes each spell after the first per-turn via `Player.spells_cast_this_turn`; (2) new `LandsTapColorlessOnly` static — `play_land` consults `lands_tap_colorless_only_active()` and downgrades multi-color/multi-mana lands' abilities to a single `{T}: Add {C}` on entry. Single-color basics pass through unchanged. Tests: `damping_sphere_taxes_each_spell_after_the_first`, `damping_sphere_resets_count_at_turn_start`, `damping_sphere_downgrades_dual_lands_to_colorless`, `damping_sphere_leaves_basic_lands_alone`. |
| 1 | Elesh Norn, Mother of Machines | ✅ | Static doubling/suppression for **both** self-source ETB triggers and `AnotherOfYours` ETB triggers (the on-cast-resolution path in `stack.rs`). Multiplier helper returns 0 when an opp Norn is in play (suppress); `1 + your_norns` otherwise. Tests: `elesh_norn_doubles_your_etb_triggers`, `elesh_norn_suppresses_opponent_etb_triggers`. |
| 3 | Mystical Dispute | ✅ | Regular cost {2}{U} or alt cost {U} (alt-cost `target_filter: HasColor(Blue)` enforces "blue-only" via stack-spell predicate evaluation). Effect is `Effect::CounterUnlessPaid { what, mana_cost: {3} }` — the engine auto-pays on behalf of the targeted spell's controller; if affordable the spell stays, otherwise it's countered. Tests: `mystical_dispute_alt_cost_requires_blue_target`, `mystical_dispute_does_not_counter_when_opponent_can_pay`, `mystical_dispute_counters_when_opponent_cannot_pay`. |
| 2 | Pest Control | ✅ | `Keyword::Convoke`. `ForEach(EachPermanent(Nonland))` body destroys the iterated permanent if `ManaValueOf(it) ≤ ConvergedValue`. Test: `pest_control_at_converge_three_destroys_higher_cmc` (still passes the original case via converge=2 from {W}{B}). |
| 2 | Teferi, Time Raveler | ✅ | 4-loyalty walker. **+1** wired: `Effect::GrantSorceriesAsFlash` flips a per-player flag the cast paths consult, letting the controller skip the sorcery-timing gate until their next turn (cleared in `do_untap`). **-3** wired: `Move(target nonland opp permanent → owner's hand) + Draw 1`. **Static "each opponent casts only at sorcery speed"** wired: `StaticEffect::OpponentsSorceryTimingOnly` + `player_locked_to_sorcery_timing(p)` consulted by all three cast paths to flip the timing gate. Tests: `teferi_minus_three_returns_target_and_draws`, `teferi_plus_one_grants_sorceries_as_flash_until_next_turn`, `teferi_static_locks_opponent_to_sorcery_timing`, `teferi_static_does_not_restrict_controllers_own_casts`. |
| 2 | Wrath of the Skies | ✅ | `{X}{W}{W}` + `Keyword::Convoke`. `ForEach(EachPermanent(Nonland))` body destroys the entity if `ManaValueOf(TriggerSource) == XFromCost`. Convoke creatures pay the X-portion as generic via tap. Tests: `wrath_of_the_skies_destroys_permanents_with_mana_value_x`, `convoke_taps_creature_to_pay_one_generic`. |

## Modern supplement (catalog::sets::decks::modern)

A pack of additional Modern-playable cards built entirely on existing
engine primitives — no engine changes required. Each entry has at least
one functionality test in `crabomination/src/tests/modern.rs` (registered
via `#[path = "../tests/modern.rs"] mod tests_modern` in `game::mod`).

| Card | Cost | Status | Notes |
|---|---|---|---|
| Ponder | {U} | ✅ | Approximated as `Scry 3 + Draw 1` (skips "may shuffle") |
| Manamorphose | {2} | ✅ | Hybrid {R/G}{R/G} pips collapsed to {2}; gives 2 mana of any colors + draws 1 |
| Sleight of Hand | {U} | ✅ | Approximated as `Scry 1 + Draw 1` |
| Faithless Looting | {R} | ✅ | `Draw 2 + Discard 2` with `Keyword::Flashback({2}{R})` |
| Sign in Blood | {B}{B} | ✅ | "Target player draws 2 + loses 2 life" via `target_filtered(Player)` threaded into both `Draw` and `LoseLife`. |
| Night's Whisper | {1}{B} | ✅ | Draw 2, lose 2 life |
| Duress | {B} | ✅ | `DiscardChosen(EachOpponent, Nonland ∧ Noncreature)` |
| Lava Spike | {R} | ✅ | 3 damage to target player; Arcane subtype |
| Lava Dart | {R} | 🟡 | Flashback cost approximated as `{0}` — engine has no "sacrifice a Mountain" alt-cost primitive |
| Unburial Rites | {3}{B} | ✅ | `Move(target creature → BF)` + `Keyword::Flashback({W}{B})` |
| Exhume | {1}{B} | ✅ (was 🟡) | Push (modern_decks): "each player reanimates a creature card from their graveyard" symmetry now wired via `ForEach(EachPlayer) → Move(take(1, Graveyard(Triggerer), Creature), Battlefield(Triggerer))`. Each player's auto-decider picks the top matching creature in their own gy. Tests: `exhume_reanimates_creature`, `exhume_each_player_reanimates_a_creature`. |
| Buried Alive | {2}{B} | ✅ | "Up to three" search wired via `Repeat(3, Search)`. Decider can answer `Search(None)` to opt out of any iteration. |
| Entomb | {B} | ✅ | `Search(Any → Graveyard)` |
| Burning-Tree Emissary | {2} | ✅ | Hybrid pips collapsed to {2}; ETB adds {R}{G} |
| Putrid Imp | {B} | 🟡 | Discard outlet wired (grants Menace EOT); madness flavor stubbed |
| Tarmogoyf | {1}{G} | ✅ | Same dynamic-P/T injection as Cosmogoyf — both names share the layer-7 `SetPowerToughness` site in `compute_battlefield`. |
| Veil of Summer | {G} | 🟡 | Cantrip half wired; "if blue/black spell cast" gate + uncounterable rider stubbed |
| Crop Rotation | {G} | ✅ | `Sacrifice(Land) + Search(Land → BF)` — sacrifice-as-additional-cost folded into resolution |
| Karakas | — | ✅ | `{T}: Add {W}` + `{T}: Move target legendary creature → owner's hand` |
| Bojuka Bog | — | ✅ | ETB-tapped + `Move(EachOpponent's graveyard → Exile)`. Uses `Move` (which handles `EntityRef::Card`) rather than `Effect::Exile` (battlefield-only) |
| Cathartic Reunion | {1}{R} | ✅ | `Discard 2 + Draw 3` — additional-cost discard folded into resolution. |
| Gitaxian Probe | — | ✅ | Phyrexian {U/Φ} simplified to {0} cost + `LoseLife 2 + Draw 1`. The opponent-hand peek is dropped (information-only effect). |
| Force Spike | {U} | ✅ | `Effect::CounterUnlessPaid {3=>{1}}` — engine auto-pays on the targeted spell's controller. |
| Vampiric Tutor | {B} | ✅ | `LoseLife 2 + Search(Any → Library{Top})`. |
| Sylvan Scrying | {1}{G} | ✅ | `Search(Land → Hand)` — clean reusable land-tutor. |
| Abrupt Decay | {B}{G} | ✅ | `Destroy(Nonland ∧ ManaValueAtMost(2))` + `Keyword::CantBeCountered`. |
| Kodama's Reach | {2}{G} | ✅ | `Seq([Search(BasicLand → BF tapped), Search(BasicLand → Hand)])`. |
| Lotus Petal | — | ✅ | `{T}, sac: Add one mana of any color` via `sac_cost: true`. |
| Tormod's Crypt | — | ✅ | `{T}, sac: exile each opponent's graveyard` (target-player half collapsed to each-opp). |
| Mishra's Bauble | — | ✅ | `{T}, sac: LookAtTop(You) + DelayUntil(YourNextUpkeep, Draw 1)`. |
| Stoneforge Mystic | {1}{W} | ✅ | ETB `Search(HasArtifactSubtype(Equipment) → Hand)`. The {1}{W}, {T}: equip ability is omitted (no equipment-attach activation primitive). |
| Qasali Pridemage | {G}{W} | ✅ | `{1}, sac: Destroy(Artifact ∨ Enchantment)`. Exalted omitted (no per-attack-power-pump primitive). |
| Greater Good | {2}{G}{G} | ✅ | Activated ability: `SacrificeAndRemember + Draw(SacrificedPower) + Discard 3` — reuses Thud's primitive. |
| Pyroclasm | {1}{R} | ✅ | `ForEach(Creature)` + `DealDamage 2`. Same shape as Anger of the Gods. |
| Day of Judgment | {2}{W}{W} | ✅ | `ForEach(Creature)` + `Destroy`. "Can't be regenerated" rider collapses. |
| Damnation | {2}{B}{B} | ✅ | Black mirror of Day of Judgment. |
| Mystical Tutor | {U} | ✅ | `Search(Instant ∨ Sorcery → Library{Top})`. |
| Worldly Tutor | {G} | ✅ | `Search(Creature → Library{Top})`. |
| Enlightened Tutor | {W} | ✅ | `Search(Artifact ∨ Enchantment → Library{Top})`. |
| Diabolic Tutor | {2}{B}{B} | ✅ | `Search(Any → Hand)` — sorcery-speed Demonic Tutor at higher CMC. |
| Imperial Seal | {B} | ✅ | `LoseLife 2 + Search(Any → Library{Top})` — sorcery-speed Vampiric Tutor. |
| Lightning Strike | {1}{R} | ✅ | `DealDamage(Target, 3)`. |
| Goblin Bombardment | {1}{R} | ✅ | Activated: `SacrificeAndRemember(Creature) + DealDamage(Target, 1)`. Sac is the cost; damage is fixed. |
| Wasteland | — | ✅ | Two abilities: `{T}: Add {C}` + `{T}, sac: Destroy nonbasic land`. |
| Strip Mine | — | ✅ | Same as Wasteland, but destroys *any* land. |
| Snuff Out | {3}{B} | ✅ | Destroy nonblack creature. Alt cost: pay 4 life (Swamp-control gate collapsed). |
| Unsummon | {U} | ✅ | `Move(target creature → Hand(OwnerOf))`. |
| Boomerang | {U}{U} | ✅ | `Move(target permanent → Hand(OwnerOf))` — wider than Unsummon (any permanent, including lands). |
| Cyclonic Rift | {1}{U} | 🟡 | Cast-time filter `Permanent ∧ Nonland ∧ ControlledByOpponent`; `Move → Hand(OwnerOf)`. Overload `{6}{U}` mode still ⏳. |
| Repeal | {X}{U} | ✅ | `If(ManaValueOf(Target) ≤ XFromCost, Move(target → Hand(OwnerOf)), Noop) + Draw 1`. CMC gate evaluated at resolution against the X paid. |
| Murder | {1}{B}{B} | ✅ | `Destroy(Creature)`. |
| Go for the Throat | {1}{B} | ✅ | `Destroy(Creature ∧ Not(Artifact))` — cast-time filter rejects artifact creatures. |
| Disfigure | {B} | ✅ | `PumpPT(-2/-2 EOT)` — kills a 2/2. |
| Languish | {2}{B}{B} | ✅ | `ForEach(EachPermanent(Creature)) + PumpPT(-2/-2 EOT)`. Sweeps 2/2s, leaves 4/4s. |
| Lay Down Arms | {W} | 🟡 | `Exile(Creature ∧ PowerAtMost(4))`. Plains-count cost-rebate clause collapsed (no count-based-cost-rebate primitive). |
| Smelt | {R} | ✅ | `Destroy(Artifact)`. |
| Banefire | {X}{R} | ✅ (was 🟡) | Push (modern_decks): `DealDamage(Target, XFromCost)` + "uncounterable if X ≥ 5" rider via the new `caster_grants_uncounterable_with_x` helper. At cast time `finalize_cast` flags `StackItem::Spell.uncounterable = true` for Banefire when X ≥ 5. Damage-can't-be-prevented half is a no-op (engine has no general damage-prevention layer). Tests: `banefire_deals_x_damage_to_creature`, `banefire_burns_a_player_face_for_x`, `banefire_uncounterable_at_x_five`, `banefire_counterable_below_x_five`. |
| Spectral Procession | {2}{W} | 🟡 | `CreateToken(3 × 1/1 white Spirit with Flying)`. Hybrid white-or-2-life pips collapsed to {2}{W} (most permissive). |
| Regrowth | {1}{G} | ✅ | `Move(target → Hand(You))` over `Any`. Auto-target prefers the caster's graveyard via `prefers_graveyard_source`. |
| Beast Within | {2}{G} | ✅ | `Seq([CreateToken(3/3 Beast to ControllerOf(Target)), Destroy(Permanent)])`. The token's controller is captured at cast time so it goes to the destroyed permanent's controller. |
| Grasp of Darkness | {B}{B} | ✅ | `PumpPT(-4/-4 EOT)`. Kills 4-toughness creatures regardless of color (vs Doom Blade's nonblack restriction). |
| Shatter | {1}{R} | ✅ | `Destroy(Artifact)`. Wider availability than Smelt (cube wants both). |
| Incinerate | {1}{R} | ✅ | `DealDamage(Target, 3)` — Lightning Strike twin. The "can't be regenerated" rider collapses (no observable regeneration site). |
| Searing Spear | {1}{R} | ✅ | `DealDamage(Target, 3)` — Lightning Strike twin in everything but name. |
| Flame Slash | {R} | ✅ | `DealDamage(target Creature, 4)` — sorcery; cast-time creature-only filter rejects players. |
| Roast | {1}{R} | ✅ | `DealDamage(target Creature ∧ Not(HasKeyword(Flying)), 5)` — cast-time filter rejects fliers. |
| Smother | {1}{B} | ✅ | `Destroy(Creature ∧ ManaValueAtMost(3))` — cast-time CMC gate. |
| Final Reward | {4}{B} | ✅ | `Exile(Creature)` — high-cost exile that beats indestructible / regen / recursion. |
| Holy Light | {W} | ✅ | `ForEach(Creature) + PumpPT(-1/-1 EOT)` — sweep small creatures for {W}. The "nonwhite" filter collapses (parallels Languish). |
| Mana Tithe | {W} | ✅ | `Effect::CounterUnlessPaid({1})` — white Force Spike. |
| Rampant Growth | {1}{G} | ✅ | `Search(IsBasicLand → Battlefield(You, tapped))`. |
| Cultivate | {2}{G} | ✅ | `Seq([Search(IsBasicLand → BF tapped), Search(IsBasicLand → Hand)])` — Kodama's Reach mirror. |
| Farseek | {1}{G} | ✅ | `Search(IsBasicLand → Battlefield(You, tapped))` — basic-only collapse of the Plains/Island/Swamp/Mountain dual fixer. |
| Sakura-Tribe Elder | {1}{G} | ✅ | 1/1 Snake; `{T}, sac: Search(IsBasicLand → BF tapped)` via `sac_cost: true`. |
| Wood Elves | {2}{G} | ✅ | 1/1 Elf Scout; ETB `Search(Land ∧ HasLandType(Forest) → BF untapped)` — note untapped, distinct from Rampant Growth. |
| Elvish Mystic | {G} | ✅ | 1/1 Elf Druid mana dork; `{T}: Add {G}` (Llanowar Elves twin). |
| Harmonize | {2}{G}{G} | ✅ | `Draw 3` — premium green refuel. |
| Concentrate | {2}{U}{U} | ✅ | `Draw 3` — blue mirror. |
| Severed Strands | {1}{B} | ✅ | `Seq([Sacrifice(Creature, 1), Destroy(target Creature), GainLife 2])` — sac-as-additional-cost folded into resolution; lifegain collapsed to a flat 2 (no `SacrificedToughness` value yet). |
| Anticipate | {1}{U} | ✅ | `Scry 2 + Draw 1` — slight under-count of "look at 3, take any" but preserves the smoothing axis. |
| Divination | {2}{U} | ✅ | `Draw 2` — staple blue card-advantage. |
| Ambition's Cost | {3}{B} | ✅ | `Draw 3 + LoseLife 3`. |
| Path of Peace | {2}{W} | ✅ | `Seq([GainLife 4 → ControllerOf(target), Destroy(target Creature)])` — sorcery removal that gives the opponent 4 life. |
| Despise | {B} | ✅ | `DiscardChosen(EachOpponent, Creature ∨ Planeswalker)`. |
| Distress | {B}{B} | ✅ | `DiscardChosen(EachOpponent, Nonland ∧ Noncreature)`. |
| Vryn Wingmare | {2}{W} | ✅ | 2/1 Bird Soldier Flying; `StaticEffect::AdditionalCostAfterFirstSpell(Noncreature, +1)` (Thalia mirror). |
| Lava Coil | {1}{R} | ✅ (was 🟡) | Push (modern_decks): "if it would die, exile instead" rider approximated via `Effect::If { cond: ValueAtMost(ToughnessOf(Target), 4), then: Exile, else_: DealDamage 4 }`. When the target's toughness ≤ 4 (the lethal case), the engine routes directly to exile; otherwise just deals 4 damage. The prior-damage-on-creature edge case isn't captured (no general damage-replacement-with-exile primitive). Tests: `lava_coil_kills_a_four_toughness`, `lava_coil_deals_damage_without_killing_a_five_toughness`. |
| Jaya's Greeting | {1}{R} | ✅ | `Seq([DealDamage(target Creature, 3), Scry 2])`. |
| Telling Time | {1}{U} | ✅ | `Scry 2 + Draw 1` — same approximation as Anticipate. |
| Read the Tides | {3}{U} | ✅ | `Draw 3` — Concentrate at off-color cost. |
| Last Gasp | {1}{B} | ✅ | `PumpPT(-3/-3 EOT)` — kills 3-toughness creatures. |
| Wild Mongrel | {1}{G} | 🟡 | 2/2 Hound; `Discard 1: +1/+1 EOT` (Psychic Frog mirror). The "becomes the color of your choice" half collapses. |
| Tear Asunder | {1}{B}{G} | 🟡 | `Destroy(Artifact ∨ Enchantment)`. Kicker {2} "destroy any nonland permanent" mode collapsed (alt-cost can't yet swap target filters at cast time). |
| Assassin's Trophy | {B}{G} | 🟡 | `Destroy(Permanent ∧ Nonland ∧ ControlledByOpponent)`. The "owner searches their library for a basic land" downside is omitted (Search always targets the caster). |
| Volcanic Fallout | {1}{R}{R} | ✅ (was 🟡) | Push (modern_decks): body unchanged (2 damage to each creature + each player). The "can't be countered" rider now lands via `Keyword::CantBeCountered`. `caster_grants_uncounterable_with_x` reads this keyword and flips `StackItem::Spell.uncounterable = true` at cast time. Tests: `volcanic_fallout_deals_two_to_each_creature_and_player`, `volcanic_fallout_is_uncounterable`. |
| Rout | {3}{W}{W} | 🟡 | `ForEach(Creature) + Destroy` — DoJ at +1 mana. Flash mode collapsed. |
| Plague Wind | {8}{B}{B} | 🟡 | `ForEach(Creature ∧ ControlledByOpponent) + Destroy` — one-sided creature sweep. Regen rider collapsed. |
| Carnage Tyrant | {4}{G}{G} | ✅ | 7/6 Dinosaur with `Trample + Hexproof + CantBeCountered`. The keyword gates `CounterSpell` resolution at the engine. |
| Krark-Clan Ironworks | {4} | ✅ | `Sac an artifact: Add {2}` — cost folded into resolution via `SacrificeAndRemember + AddMana`. Combo enabler in artifact-heavy decks. |
| Stone Rain | {2}{R} | ✅ | `Destroy(Land)` — three-mana land destruction. |
| Bone Splinters | {B} | ✅ | `Seq([SacrificeAndRemember(Creature ∧ ControlledByYou), Destroy(Creature)])`. Sac-as-additional-cost folded into resolution. |
| Hieroglyphic Illumination | {2}{U} | ✅ | `Draw 2` — instant-speed Divination. Cycling collapsed (no Cycling primitive yet). |
| Mortify | {1}{W}{B} | ✅ | `Destroy(Creature ∨ Enchantment)` — premium WB removal. |
| Maelstrom Pulse | {1}{B}{G} | 🟡 | `Destroy(Permanent ∧ Nonland)` — single-target version. The "all permanents with the same name" rider is collapsed (no name-match selector). |
| Mind Twist | {X}{B} | ✅ | `Discard { who: target Player, amount: XFromCost, random: true }` — bot pumps remaining mana into X via `max_affordable_x`. |
| Dismember | {1}{B}{B} | 🟡 | `PumpPT(-5/-5 EOT)` — Phyrexian-mana cost ({1}{B/Φ}{B/Φ}{B/Φ}) collapsed to flat black; the body kills 5-toughness creatures. |
| Echoing Truth | {1}{U} | 🟡 | `Move(target Permanent ∧ Nonland → Hand(OwnerOf))` — single-target bounce. The "all permanents with the same name" rider is collapsed. |
| Celestial Purge | {1}{W} | ✅ | `Exile(Permanent ∧ (HasColor(Black) ∨ HasColor(Red)))` — color-hate exile. |
| Earthquake | {X}{R} | ✅ | `Seq([ForEach(Creature ∧ Not(Flying)) → DealDamage X, ForEach(EachPlayer) → DealDamage X])`. |
| Glimpse the Unthinkable | {U}{B} | ✅ | `Mill(target Player, 10)` — premium two-mana mill. |
| Cling to Dust | {B} | 🟡 | `Seq([Move(target Any → Exile), If(EntityMatches Creature, GainLife 2)])`. Escape mode ({2}{B}, exile five other graveyard cards) collapsed (no escape-cost primitive). |
| Lumra, Bellow of the Woods | {4}{G}{G} | ✅ | 6/6 Trample. ETB returns every land card in your graveyard via `Move(EachMatching(Graveyard(You), Land) → Battlefield(You, tapped))`. Tests: `lumra_returns_all_lands_from_your_graveyard`, `lumra_etb_with_empty_graveyard_is_a_noop`. |
| Crabomination | {2}{U}{B} | ✅ | 3/4 Crab Horror. ETB mills 3 from each opponent; whenever a creature your opponents control dies, scry 1 (`CreatureDied + OpponentControl` trigger). Custom card. Test: `crabomination_etb_mills_each_opponent_three_cards`. |
| Chaos Warp | {2}{R} | 🟡 | `Move(target Permanent → Library(OwnerOf, Shuffled))`. The library actually reshuffles via the new `LibraryPosition::Shuffled` engine path. The "reveal top, cast if permanent" half is collapsed. Test: `chaos_warp_sends_target_permanent_to_owners_library`. |
| Elvish Reclaimer | {1}{G} | 🟡 | 1/2 Human Druid. `{T}, sac a land: Search(Land → BF)`. Sac-as-cost folded into resolution. Threshold-pump rider omitted. Test: `elvish_reclaimer_sacrifices_land_to_search_for_one`. |
| Rofellos, Llanowar Emissary | {G}{G} | ✅ (was 🟡) | Legendary 2/1 Elf Druid. `{T}: Add {G}{G} for each Forest you control` wired via `ManaPayload::OfColor(Green, Times(Const(2), CountOf(Forest ∧ ControlledByYou)))`. Tests: `rofellos_taps_for_two_green_mana`, `rofellos_taps_for_two_green_per_forest`, `rofellos_taps_for_zero_with_no_forests`. |
| Biorhythm | {4}{G}{G}{G} | 🟡 | `LoseLife(EachOpponent, 20) + GainLife(You, count(your creatures))`. Set-life-equal-to-X primitive doesn't exist, so each opponent loses a giant chunk and the caster gains life equal to their creature count. Test: `biorhythm_drops_each_opponent_to_zero_or_below`. |
| Karn, Scion of Urza | {4} | 🟡 | 5-loyalty Karn. **+1**: Draw 1 + mill 1 (the opp-pile-split is information-only at this engine fidelity). **-1**: ForEach Construct creature you control + AddCounter(+1/+1). **-2**: Create a 1/1 Construct token (the artifact-count scaling rider collapses). Tests: `karn_scion_of_urza_minus_two_creates_a_construct_token`, `karn_plus_one_draws_a_card_and_mills_one`. |
| Tezzeret, Cruel Captain | {3}{B} | 🟡 | 4-loyalty walker. **+1**: target creature gets -2/-2 EOT. **-2**: drain 2 life from each opponent. The ult is collapsed; "your artifact creatures get +1/+1" static is dropped. Tests: `tezzeret_minus_two_drains_each_opponent_for_two`, `tezzeret_plus_one_shrinks_target_creature`. |
| Balefire Dragon | {5}{R}{R} | 🟡 | 6/6 Flying. Combat-damage-to-player trigger sweeps each opp creature for 6 (the "that much damage" → fixed 6 collapse). Test: `balefire_dragon_combat_damage_burns_each_opp_creature`. |
| Greasewrench Goblin | {1}{R} | ✅ | 2/2 Haste. Treasure-on-death trigger via `EventKind::CreatureDied + SelfSource` + `treasure_token()`. The "can't block" Oracle rider is collapsed. Test: `greasewrench_goblin_creates_treasure_on_death`. |
| Cruel Somnophage | {1}{U}{B} | ✅ | 0/0 Phyrexian Horror; layer-7 `SetPowerToughness(grave_size, grave_size)` injection in `compute_battlefield`. Test: `cruel_somnophage_pt_scales_with_your_graveyard`. |
| Pentad Prism | {2} | 🟡 | ETB with 2 charge counters; remove a charge counter to add one mana of any color. Sunburst's "one counter per color of mana spent" collapses to a flat 2. Tests: `pentad_prism_etb_with_two_charge_counters`, `pentad_prism_removes_counter_to_add_one_mana_of_any_color`. |
| Vindicate | {1}{W}{B} | ✅ | Sorcery. Destroy target permanent (filter is `Permanent`, accepts lands too). Tests: `vindicate_destroys_target_permanent`, `vindicate_can_target_a_land`. |
| Anguished Unmaking | {1}{W}{B} | ✅ | Instant. Exile target nonland permanent + you lose 3 life. Test: `anguished_unmaking_exiles_and_caster_loses_three_life`. |
| Despark | {W}{B} | ✅ | Instant. Exile target permanent with mana value 4+. Tests: `despark_exiles_high_cmc_permanent`, `despark_rejects_low_cmc_target`. |
| Magma Spray | {R} | ✅ (was 🟡) | Push (claude/modern_decks, batch 102): the "exile if it would die" rider now wires the same `Effect::If { cond: ValueAtMost(ToughnessOf(Target), 2), then: Exile, else_: DealDamage 2 }` pattern as Lava Coil. The prior-damage-on-creature edge case isn't captured (no general damage-replacement-with-exile primitive). Tests: `magma_spray_deals_two_damage_to_creature`, `magma_spray_exiles_a_low_toughness_creature_via_if_branch`. |
| Skullcrack | {1}{R} | ✅ (was 🟡) | Push (claude/modern_decks): "Target player can't gain life this turn" rider now wired via new `Effect::LifeGainLockThisTurn` primitive + `Player.cannot_gain_life_this_turn` field (reset across all players in `do_untap`). The 3-damage Bolt rides on top via `Seq`. "Damage can't be prevented" rider is still omitted (no general damage-prevention layer). Tests: `skullcrack_deals_three_damage_to_player`, `skullcrack_locks_target_player_lifegain_for_the_turn`, `skullcrack_lifegain_lock_clears_at_next_untap`. |
| Fiery Impulse | {R} | ✅ (was 🟡) | Push (claude/modern_decks): spell-mastery scaling now wires via `Effect::If { cond: SelectorCountAtLeast(IS-in-your-graveyard, 2), then: DealDamage 3, else_: DealDamage 2 }`. Uses the existing `Selector::CardsInZone` + `Predicate::SelectorCountAtLeast` primitives. Tests: `fiery_impulse_deals_two_damage_to_creature`, `fiery_impulse_deals_three_damage_with_spell_mastery`, `fiery_impulse_deals_two_damage_without_spell_mastery`. |
| Searing Blood | {R}{R} | 🟡 | Instant. 2 damage to target creature. The "if it dies, deal 3 to controller" rider collapses (no "if-target-dies" delayed trigger primitive yet). Test: `searing_blood_deals_two_damage_to_creature`. |
| Crumble to Dust | {2}{R}{R} | 🟡 | Sorcery. Exile target nonbasic land. The "exile every card with the same name" rider collapses. Tests: `crumble_to_dust_exiles_nonbasic_land`, `crumble_to_dust_rejects_basic_land_target`. |
| Drown in the Loch | {U}{B} | 🟡 | Instant. ChooseMode([CounterSpell, Destroy(Creature ∨ Planeswalker)]). The "snow mana only" + "X = cards in opp's graveyard" gates collapse. Tests: `drown_in_the_loch_mode_zero_counters_a_spell`, `drown_in_the_loch_mode_one_destroys_creature`. |
| Cremate | {B} | ✅ | Instant. Exile target card in a graveyard + draw a card. Auto-target prefers graveyard cards via `prefers_graveyard_target`. Test: `cremate_exiles_graveyard_card_and_draws`. |
| Harrow | {2}{G} | ✅ | Instant. Sac a land + search 2 basics → BF untapped. Sac-as-additional-cost folded into resolution; net +1 land. Test: `harrow_sacrifices_land_and_searches_two_basics`. |
| Mortuary Mire | — | ✅ | Land. ETB-tapped. ETB: optionally put target creature card from your graveyard on top of your library. {T}: Add {B}. Test: `mortuary_mire_etb_taps_and_recurs_creature_card`. |
| Geier Reach Sanitarium | — | ✅ | Legendary Land. {T}: Add {C}. {1}, {T}: each player draws a card, then discards a card. Tests: `geier_reach_sanitarium_taps_for_colorless`, `geier_reach_sanitarium_wheel_ability_each_player_loots`. |
| Strangle | {R} | ✅ | Instant. 3 damage to target creature + Surveil 1. (Drown in Ichor's red mirror.) Test: `strangle_deals_three_damage_and_surveils`. |
| Dreadbore | {B}{R} | ✅ | Sorcery. Destroy target creature/planeswalker. Test: `dreadbore_destroys_target_creature`. |
| Bedevil | {B}{B}{R} | ✅ | Instant. Destroy target artifact/creature/planeswalker. Test: `bedevil_destroys_target_artifact`. |
| Tome Scour | {U} | ✅ | Sorcery. Target player puts the top 5 cards of their library into their graveyard. Test: `tome_scour_mills_target_player_five`. |
| Repulse | {2}{U} | ✅ | Instant. Bounce + Draw 1. Test: `repulse_returns_creature_and_draws`. |
| Visions of Beyond | {U} | ✅ (was 🟡) | Push (claude/modern_decks): the "draw 3 if a graveyard has 20+ cards" rider now wires via `Effect::If { cond: ValueAtLeast(MaxGraveyardSize, Const(20)), then: Draw(3), else_: Draw(1) }`. New `Value::MaxGraveyardSize` returns the largest graveyard among all alive players. Tests: `visions_of_beyond_draws_a_card`, `visions_of_beyond_draws_three_with_twenty_card_graveyard`, `visions_of_beyond_draws_one_with_nineteen_card_graveyard`. |
| Plummet | {1}{G} | ✅ | Instant. Destroy target flying creature. Cast-time filter rejects non-fliers. Tests: `plummet_destroys_target_flying_creature`, `plummet_rejects_non_flying_target`. |
| Strategic Planning | {1}{U} | 🟡 | Sorcery. Mill 3 + Draw 1. Approximation of "look at top 3, take 1, mill the rest" — gameplay-relevant graveyard-fill axis preserved. Test: `strategic_planning_mills_three_and_draws_one`. |
| Ravenous Rats | {1}{B} | ✅ | 1/1 Rat. ETB each opponent discards a card (random). Test: `ravenous_rats_etb_makes_each_opponent_discard`. |
| Brain Maggot | {1}{B} | 🟡 | 1/1 Spirit Insect. ETB strips a nonland card from each opponent's hand (Tidehollow Sculler approximation; "exile until LTB" still ⏳). Test: `brain_maggot_etb_strips_a_nonland_card`. |
| Bond of Discipline | {3}{W} | ✅ | Sorcery. Tap each opponent creature + your creatures gain lifelink EOT. Test: `bond_of_discipline_taps_each_opponent_creature_and_grants_lifelink`. |
| Sudden Edict | {1}{B} | ✅ | Instant. Target player sacrifices a creature; can't be countered (`Keyword::CantBeCountered`). Uses `target_filtered(Player)` so the bot's auto-target heuristic picks the opponent. Tests: `sudden_edict_forces_target_player_to_sacrifice`, `sudden_edict_cannot_be_countered`, `sudden_edict_rejects_creature_target`, `auto_target_for_sudden_edict_picks_opponent`. |

### Engine improvements that landed alongside

* **`Effect::Exile` now handles `EntityRef::Card`** in addition to
  `EntityRef::Permanent`. Battlefield exits keep the `PermanentExiled`
  event + leaves-the-battlefield path; cards in graveyards / hand /
  library route through `move_card_to(.., ZoneDest::Exile, ..)`.
  Previously graveyard-exile spells silently no-op'd.
* **Flashback-cast spells now exile on resolution** (matching Oracle).
  `cast_flashback` already set `card.kicked = true` as the marker, but
  `continue_spell_resolution` only checked for Rebound — flashback
  spells silently went to the graveyard. Test:
  `flashback_cast_exiles_spell_on_resolution`.
* **Tarmogoyf gets dynamic P/T** via the same layer-7 `SetPowerToughness`
  injection that already powered Cosmogoyf. Both names now match the
  hardcoded check in `compute_battlefield`.

## Engine features

| Feature | Status | Cards depending on it |
|---|---|---|
| Alternative pitch costs (pay life + exile a card) | ✅ | Engine + client. Force of Will, Force of Negation, Solitude (evoke). Right-click a hand card with `has_alternative_cost` → modal lets the player pick a pitch card. `evoke_sacrifice` flag on `AlternativeCost` schedules a self-sac trigger after ETB. `not_your_turn_only` flag rejects the alt cast on the caster's own turn (Force of Negation). |
| Pact-style deferred upkeep cost | ✅ | Pact of Negation, Summoner's Pact (built on `Effect::DelayUntil` + `Effect::PayOrLoseGame`) |
| Goryo's Vengeance: reanimate-then-exile-at-EOT | ✅ | Goryo's Vengeance (uses `DelayUntil(NextEndStep)` + `Exile { Target(0) }`) |
| Rebound (cast from exile next upkeep) | ✅ | Ephemerate. New `Keyword::Rebound` + `CardInstance.cast_from_hand` flag (set by `cast_spell` and `cast_spell_alternative`). `continue_spell_resolution` checks for the combo: rebound spells go to exile and register a `YourNextUpkeep` delayed trigger whose body is the spell's own effect. The fire path now auto-targets when the trigger has no captured target, so the rebound recast picks a fresh creature each fire. Modeled as effect-replay rather than true cast-from-exile (no on-cast triggers from the recast), which is gameplay-equivalent for Ephemerate. |
| Flicker (exile and return to play) | ✅ | Ephemerate. `Effect::Seq([Exile target, Move target → battlefield])` paired with the new `place_card_in_dest::Battlefield` calling `fire_self_etb_triggers` so the refired ETB actually fires. |
| Convoke / Converge cost-reduction | ✅ | New `GameAction::CastSpellConvoke { card_id, target, mode, x_value, convoke_creatures }` + internal `cast_spell_with_convoke`. Each listed creature is validated (untapped, controlled by caster, spell has `Keyword::Convoke`), tapped, and contributes {1} generic mana to the player's pool before paying the cost. Converge is computed by snapshotting the mana pool before paying and counting distinct colors that decreased; the count is stashed on `StackItem::Spell.converged_value` and threaded through to `EffectContext`. New `Value::ConvergedValue` reads it inside the spell's effect. Currently: convoke pips contribute generic only (don't raise converge). Used by Prismatic Ending, Pest Control, Wrath of the Skies. |
| Opening-hand effects (begin in play / replace draws) | ✅ | New `OpeningHandEffect` enum + `CardDefinition.opening_hand` field. `apply_opening_hand_effects` runs post-mulligan and dispatches: `StartInPlay { tapped, extra }` (Leyline, Gemstone Caverns), `RevealForDelayedTrigger { kind, body }` (Chancellor of the Tangle, Chancellor of the Annex), `MulliganHelper` (Serum Powder — surfaces in the mulligan decision). `Decision::Mulligan` gained a `serum_powders: Vec<CardId>` field; new `DecisionAnswer::SerumPowder(id)` exiles the hand and redraws. New `DelayedKind::YourNextMainPhase` so chancellor mana lands during main rather than getting emptied between steps. |
| Reveal-and-sort ETB (one of each card type) | 🟡 | Atraxa, Grand Unifier now uses `Value::DistinctTypesInTopOfLibrary` to draw N cards where N = distinct types in the top 10. Real reveal-then-multi-pick (typed library reorder + one-per-type pick UI) still ⏳. |
| Static cost increase + storm tax | ✅ | Damping Sphere's "second-and-onwards spells cost {1} more" wired via `StaticEffect::AdditionalCostAfterFirstSpell` + per-player `Player.spells_cast_this_turn` counter. The land-output clause is now wired via `StaticEffect::LandsTapColorlessOnly`: `play_land` checks for the active static and rewrites the entering land's mana abilities to `{T}: Add {C}`. Storm-style spell-count scaling still ⏳. |
| ETB-trigger replacement (suppress / double) | ✅ | Elesh Norn, Mother of Machines via `etb_trigger_multiplier(state, etb_controller)` — returns 0 (any opp Norn → suppress) or `1 + your_norns` (each Norn on your side adds an extra fire). Hooked into both `fire_self_etb_triggers` and the cast-resolution ETB push paths in `stack.rs` (self-source AND `AnotherOfYours`). `continue_trigger_resolution` re-picks a fresh auto-target if the stored one is no longer valid, so doubled triggers don't all fizzle on a single shared target. |
| Spell-timing restriction static | ✅ | New `StaticEffect::OpponentsSorceryTimingOnly` + companion `StaticEffect::ControllerSorceriesAsFlash`. `cast_spell`'s timing gate consults `player_locked_to_sorcery_timing` (rejects opponent instants/flash at non-main timing) and `player_has_sorceries_as_flash` (lets sorceries cast at instant speed for the granted player). Wires Teferi, Time Raveler's static and +1. |
| Uncounterable spell flag | 🟡 | `StackItem::Spell.uncounterable: bool` + `CounterSpell` respects it. Cavern of Souls now flags any creature spell its controller casts as uncounterable (approximation collapses "name a type" + mana provenance into "you control a Cavern → your creatures are uncounterable"). |
| Counter target *ability* (not spell) | ✅ | `Effect::CounterAbility { what }` removes the topmost matching `StackItem::Trigger` from the stack. Used by Consign to Memory's mode-0 branch; mode-1 falls through to the regular `CounterSpell` over a Legendary filter. |
| Charge-counter mana sources w/ self-sac | ✅ | Gemstone Mine. Activated ability folds the counter-removal cost into resolution and tail-checks `CountersOn(This, Charge) ≤ 0` to schedule a self-sac. |
| Shock-land ETB choice (tapped or 2 life) | ✅ | Godless Shrine, Hallowed Fountain, Watery Grave, Overgrown Tomb. ETB trigger is a `ChooseMode([LoseLife 2, Tap This])`; AutoDecider picks pay-2-life. Note: triggered ability, not a true replacement effect — the land is briefly available untapped before the trigger resolves. |
| Pathway / modal DFC mana abilities | ✅ | Blightstep Pathway, Darkbore Pathway. `CardDefinition.back_face: Option<Box<CardDefinition>>` carries the alternate face. `GameAction::PlayLandBack(CardId)` swaps the `CardInstance.definition` to the back face's definition before placing on battlefield, so all subsequent abilities/types come from the back. **Client-side flip UI**: right-click an MDFC hand card to toggle to its back face — the front-face mesh repaints with the back-face's Scryfall image and the next left-click submits `PlayLandBack` instead of `PlayLand`. (Bot still defaults to the front face.) |
| Surveil-land ETB-tapped + surveil 1 | ✅ | Meticulous Archive, Undercity Sewers, Shadowy Backstreet. `play_land` now fires self-source ETB triggers via `fire_self_etb_triggers` (lands skip the stack, so this site needs a hardcoded fire). |
| Fastland conditional ETB-tapped | ✅ | Blackcleave Cliffs, Blooming Marsh, Copperline Gorge. ETB trigger uses `Effect::If` over `Predicate::SelectorCountAtLeast` of "lands you control" (≥ 4 post-ETB). |
| Activated land mill (Cephalid Coliseum) | ✅ | Cephalid Coliseum: ActivatedAbility(`{2}{U}, {T}`, sacrifice-as-first-effect-step, then `Draw 3` and `Discard 3` for `EachPlayer`). |
| Tarmogoyf-style P/T from graveyard | ✅ | Cosmogoyf (via inline `compute_battlefield` injection of a layer-7 set-PT effect with the live graveyard card-type count). |
| X-cost creature side-effects | 🟡 | Callous Sell-Sword now ETBs via `Seq([SacrificeAndRemember, PumpPT { power: SacrificedPower, EOT }])`. Casualty's "copy this spell" branch still ⏳ (no spell-copy-modal primitive). |
| Sacrifice-as-cost effects | 🟡 | Thud ✅ via `SacrificeAndRemember` + `Value::SacrificedPower`; Plunge into Darkness still ⏳. |
| Reveal-until-find search | ✅ | New `Effect::RevealUntilFind { who, find, to, cap, life_per_revealed }` walks the top of `who`'s library until a match (up to `cap` reveals), mills the misses, and pays `life_per_revealed × revealed` life. Used by Spoils of the Vault. The "name a card" half (matching the named card via `find`) is still pending a naming primitive. |
| Loyalty abilities w/ static | ✅ | Teferi, Time Raveler **-3** wired (bounce + draw). **+1** wired via new `Player.sorceries_as_flash` flag + `Effect::GrantSorceriesAsFlash` setter, consulted by all three cast paths. **Static spell-timing veto** wired (`OpponentsSorceryTimingOnly` + `player_locked_to_sorcery_timing` in `cast_spell` / `cast_spell_alternative` / `cast_flashback`). |
| On-cast self triggers ("when you cast this …") | ✅ | Devourer of Destiny (Scry 2 on cast), Quantum Riddler (Draw 1 on cast). Each cast path (`cast_spell`, `cast_spell_alternative`, `cast_flashback`) collects `EventKind::SpellCast` + `EventScope::SelfSource` triggers off the just-cast card and pushes them onto the stack **above** the spell, so the trigger resolves first (and still fires if the spell itself is countered in response). |

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
