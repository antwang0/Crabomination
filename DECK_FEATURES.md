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
| Exhume | {1}{B} | 🟡 | Models "you reanimate" only; symmetrical "each player reanimates" not yet wired |
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
| Banefire | {X}{R} | 🟡 | `DealDamage(Target, XFromCost)`. The "uncounterable when X ≥ 5" rider is omitted (no conditional-uncounterable primitive yet). |
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
| Lava Coil | {1}{R} | 🟡 | `DealDamage(target Creature, 4)`. The "exile if it would die" rider collapses (no per-LTB replacement). |
| Jaya's Greeting | {1}{R} | ✅ | `Seq([DealDamage(target Creature, 3), Scry 2])`. |
| Telling Time | {1}{U} | ✅ | `Scry 2 + Draw 1` — same approximation as Anticipate. |
| Read the Tides | {3}{U} | ✅ | `Draw 3` — Concentrate at off-color cost. |
| Last Gasp | {1}{B} | ✅ | `PumpPT(-3/-3 EOT)` — kills 3-toughness creatures. |
| Wild Mongrel | {1}{G} | 🟡 | 2/2 Hound; `Discard 1: +1/+1 EOT` (Psychic Frog mirror). The "becomes the color of your choice" half collapses. |
| Tear Asunder | {1}{B}{G} | 🟡 | `Destroy(Artifact ∨ Enchantment)`. Kicker {2} "destroy any nonland permanent" mode collapsed (alt-cost can't yet swap target filters at cast time). |
| Assassin's Trophy | {B}{G} | 🟡 | `Destroy(Permanent ∧ Nonland ∧ ControlledByOpponent)`. The "owner searches their library for a basic land" downside is omitted (Search always targets the caster). |
| Volcanic Fallout | {1}{R}{R} | 🟡 | `Seq([ForEach(Creature) → DealDamage 2, ForEach(EachPlayer) → DealDamage 2])`. Uncounterable rider dropped. |
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
| Rofellos, Llanowar Emissary | {G}{G} | 🟡 | Legendary 0/1 Elf Druid. `{T}: Add {G}{G}` (Forest-count multiplier collapsed to flat 2). Test: `rofellos_taps_for_two_green_mana`. |
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
| Magma Spray | {R} | 🟡 | Instant. 2 damage to target creature. The "exile if it would die" rider collapses (no per-LTB replacement primitive — same simplification as Lava Coil). Test: `magma_spray_deals_two_damage_to_creature`. |
| Skullcrack | {1}{R} | 🟡 | Instant. 3 damage to target player. "Can't gain life this turn" rider collapsed. Test: `skullcrack_deals_three_damage_to_player`. |
| Fiery Impulse | {R} | 🟡 | Instant. 2 damage to target creature. Spell-mastery scaling (3 damage if 2+ instants in graveyard) collapsed. Test: `fiery_impulse_deals_two_damage_to_creature`. |
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
| Visions of Beyond | {U} | 🟡 | Instant. Draw 1. The "draw 3 if a graveyard has 20+ cards" rider collapses (no graveyard-size predicate yet). Test: `visions_of_beyond_draws_a_card`. |
| Plummet | {1}{G} | ✅ | Instant. Destroy target flying creature. Cast-time filter rejects non-fliers. Tests: `plummet_destroys_target_flying_creature`, `plummet_rejects_non_flying_target`. |
| Strategic Planning | {1}{U} | 🟡 | Sorcery. Mill 3 + Draw 1. Approximation of "look at top 3, take 1, mill the rest" — gameplay-relevant graveyard-fill axis preserved. Test: `strategic_planning_mills_three_and_draws_one`. |
| Ravenous Rats | {1}{B} | ✅ | 1/1 Rat. ETB each opponent discards a card (random). Test: `ravenous_rats_etb_makes_each_opponent_discard`. |
| Brain Maggot | {1}{B} | 🟡 | 1/1 Spirit Insect. ETB strips a nonland card from each opponent's hand (Tidehollow Sculler approximation; "exile until LTB" still ⏳). Test: `brain_maggot_etb_strips_a_nonland_card`. |
| Bond of Discipline | {3}{W} | ✅ | Sorcery. Tap each opponent creature + your creatures gain lifelink EOT. Test: `bond_of_discipline_taps_each_opponent_creature_and_grants_lifelink`. |
| Sudden Edict | {1}{B} | ✅ | Instant. Target player sacrifices a creature; can't be countered (`Keyword::CantBeCountered`). Uses `target_filtered(Player)` so the bot's auto-target heuristic picks the opponent. Tests: `sudden_edict_forces_target_player_to_sacrifice`, `sudden_edict_cannot_be_countered`, `sudden_edict_rejects_creature_target`, `auto_target_for_sudden_edict_picks_opponent`. |
| Talisman of Hierarchy | {2} | ✅ | WB Talisman. Reuses `talisman_cycle`. Test: `talisman_of_hierarchy_taps_for_black_costing_one_life`. |
| Talisman of Impulse | {2} | ✅ | RG Talisman. Test: `talisman_of_impulse_taps_for_green_costing_one_life`. |
| Talisman of Indulgence | {2} | ✅ | BR Talisman. Test: `talisman_of_indulgence_colorless_tap_costs_no_life`. |
| Talisman of Resilience | {2} | ✅ | BG Talisman. Test: `talisman_of_resilience_taps_for_black_costing_one_life`. |
| Talisman of Unity | {2} | ✅ | GW Talisman. Closes the 10-pair Talisman cycle. Test: `talisman_of_unity_taps_for_white_costing_one_life`. |
| Pristine Talisman | {3} | ✅ | Artifact. {T}: Add {C}. {T}: Gain 1 life. Tests: `pristine_talisman_lifegain_ability_grants_one_life`, `pristine_talisman_mana_ability_doesnt_change_life`. |
| Wayfarer's Bauble | {1} | ✅ | Artifact. {2},{T},Sac: Search basic land → BF tapped. Sac-as-cost folded into resolution. Test: `wayfarers_bauble_searches_a_basic_land`. |
| Burnished Hart | {3} | ✅ | 2/2 Construct artifact creature. {3},{T},Sac: Search up to 2 basic lands → BF tapped. Tests: `burnished_hart_searches_two_basic_lands`, `burnished_hart_is_a_two_two_construct`. |
| Adarkar Wastes | — | ✅ | Land. WU painland. New `painland(name, type_a, type_b, c1, c2)` helper bundles `{T}: Add {C}` (no life cost) plus two colored taps that each lose 1 life. Tests: `adarkar_wastes_colorless_tap_costs_no_life`, `adarkar_wastes_blue_tap_costs_one_life`. |
| Underground River | — | ✅ | Land. UB painland. Test: `underground_river_taps_for_black_costing_one_life`. |
| Sulfurous Springs | — | ✅ | Land. BR painland. Test: `sulfurous_springs_taps_for_red_costing_one_life`. |
| Karplusan Forest | — | ✅ | Land. RG painland. Test: `karplusan_forest_taps_for_green_costing_one_life`. |
| Brushland | — | ✅ | Land. GW painland. Closes the ally cycle. Test: `brushland_taps_for_white_costing_one_life`. |
| Exploration | {G} | ✅ | Enchantment. **First catalog card** to exercise the engine's `StaticEffect::ExtraLandPerTurn`. Push XLVIII wired the static into a new `GameState::player_can_play_land` predicate (CR 305.2). Multiple copies stack. Tests: `baseline_player_caps_at_one_land_per_turn`, `exploration_allows_a_second_land_play_in_one_turn`, `two_explorations_grant_three_land_plays`, `exploration_does_not_help_the_opponent`. |
| Seal of Fire | {R} | ✅ NEW | Push L. Enchantment. Sac-as-cost activation deals 2 damage to any target. Wired via `ActivatedAbility { sac_cost: true, mana_cost: 0, effect: DealDamage(any_target, 2) }`. Test: `seal_of_fire_sac_deals_two_to_opponent`. |
| Seal of Cleansing | {1}{W} | ✅ NEW | Push L. Enchantment. Sac-as-cost activation destroys an artifact or enchantment. Mirror of Seal of Fire in white. Test: `seal_of_cleansing_destroys_target_enchantment`. |
| Phyrexian Walker | {0} | ✅ NEW | Push L. 0/3 Construct artifact creature. Free chump-blocker — slots into Affinity shells alongside Memnite (1/1) and Ornithopter (0/2 Flying). Test: `phyrexian_walker_is_a_zero_three_artifact_creature`. |
| Honor of the Pure | {1}{W} | ✅ NEW | Push L. Enchantment. White creatures you control get +1/+1. **First catalog card** to use the new `colors_any` filter on `AffectedPermanents::All` (push L engine wire — extends `affected_from_requirement` to extract `R::HasColor(c)` pips). Test: `honor_of_the_pure_pumps_white_creatures_you_control`. |
| Crusade | {W}{W} | ✅ NEW | Push L. Enchantment. White creatures get +1/+1 (symmetric — affects both your and opp's whites). Test: `crusade_pumps_white_creatures_both_sides`. |
| Bad Moon | {1}{B} | ✅ NEW | Push L. Enchantment. Black creatures get +1/+1 (symmetric). Test: `bad_moon_pumps_black_creatures_both_sides`. |
| Lightning Axe | {R} | 🟡 NEW | Push L. Instant. `additional_discard_cost: Some(1)` + 5 damage to creature. The "or pay 5 life" alt-cost half collapsed (engine's `AlternativeCost` doesn't yet swap discard-for-life). Tests: `lightning_axe_discards_a_card_and_burns_for_five`, `lightning_axe_rejects_with_empty_hand`. |
| Skred | {R} | 🟡 NEW | Push L. Instant. Flat 3 damage to creature (snow-permanent scaling collapsed; matches printed at typical 3-Mountain build floor). Test: `skred_deals_three_damage_to_target_creature`. |
| Soul's Attendant | {W} | ✅ NEW | Push L. 1/1 Human Cleric. Soul Warden mirror — gains 1 life when another creature ETBs. Tests: `souls_attendant_gains_life_on_other_creature_etb`, `souls_attendant_does_not_trigger_on_self_etb`. |
| Dragon's Claw | {2} | ✅ NEW | Push L. Artifact. Whenever a player casts a red spell, you gain 1 life. `EventScope::AnyPlayer + Predicate::EntityMatches(TriggerSource, HasColor(Red))`. Tests: `dragons_claw_triggers_on_any_red_spell_cast`, `dragons_claw_skips_non_red_spell_cast`. |
| Wurm's Tooth | {2} | ✅ NEW | Push L. Artifact. Green slot in the Mirrodin colour-protection 5-card cycle. Test: `wurms_tooth_triggers_on_green_spell_cast`. |
| Kraken's Eye | {2} | ✅ NEW | Push L. Artifact. Blue slot in the Mirrodin cycle. Same shape as Dragon's Claw. |
| Angel's Feather | {2} | ✅ NEW | Push L. Artifact. White slot in the Mirrodin cycle. |
| Demon's Horn | {2} | ✅ NEW | Push L. Artifact. Black slot in the Mirrodin cycle — closes the 5-card cycle (W/U/B/R/G). |

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

## Implementation log (most recent first)

- **modern_decks-15: 12 new cube cards + edict-class auto-target +
  ability_cost_label sacrifice rider**:
  - **12 new cards** in `catalog::sets::decks::modern`, all built on
    existing engine primitives:
    - **Burn**: `strangle` ({R} 3 dmg + Surveil 1).
    - **Removal**: `dreadbore` ({B}{R} sorcery destroy
      creature/planeswalker), `bedevil` ({B}{B}{R} instant destroy
      artifact/creature/planeswalker).
    - **Mill**: `tome_scour` ({U} sorcery target player mills 5).
    - **Bounce + cantrip**: `repulse` ({2}{U} bounce + draw),
      `visions_of_beyond` ({U} cantrip — graveyard-size predicate
      collapsed).
    - **Removal**: `plummet` ({1}{G} instant destroy flying creature
      — cast-time filter rejects non-fliers).
    - **Graveyard fill**: `strategic_planning` ({1}{U} sorcery mill
      3 + draw 1).
    - **Discard creatures**: `ravenous_rats` ({1}{B} 1/1 ETB each
      opponent discards), `brain_maggot` ({1}{B} 1/1 ETB strip
      nonland — Tidehollow Sculler approximation).
    - **Tempo / lifegain**: `bond_of_discipline` ({3}{W} sorcery
      tap each opp creature + your creatures gain lifelink EOT).
    - **Edict**: `sudden_edict` ({1}{B} instant target player
      sacrifices a creature; can't be countered).
  - **Engine: surface Sacrifice's player filter in
    `primary_target_filter`**. Pre-fix `Effect::Sacrifice` was missing
    from `primary_target_filter`, so the bot's `auto_target_for_effect`
    returned None for edict-class spells whose `who` slot uses
    `target_filtered(SelectionRequirement::Player)`. New
    `Effect::Sacrifice { who, .. } => sel_filter(who)` arm threads the
    Player filter through. Bare `Selector::Target(0)` still returns
    None (sel_filter doesn't classify it), so existing edicts that
    pre-date the filter primitive (Diabolic Edict, Geth's Verdict)
    keep their explicit-target casting contract. New edicts should
    prefer the filter form. Test:
    `auto_target_for_sudden_edict_picks_opponent`.
  - **UI/view: `ability_cost_label` advertises sacrifice rider**.
    Sacrifice-cost activated abilities (Lotus Petal, Wasteland, Strip
    Mine, Tormod's Crypt, Mind Stone's draw, Mishra's Bauble, Aether
    Spellbomb, Cathar Commando, Haywire Mite, Sakura-Tribe Elder,
    Cankerbloom, Greater Good, Greasewrench Goblin's treasure, etc.)
    were rendering only their tap+mana costs in the wire view —
    making them look free for the listed cost. The label now appends
    "Sac" when `ActivatedAbility.sac_cost` is set so the UI tooltip
    matches the actual activation cost. Tap-cost ordering moved
    after mana so the rendered string matches the canonical MTG cost
    order (mana then tap then other costs): "{1}, {T}, Sac" rather
    than "{T}, {1}". Test:
    `ability_cost_label_includes_sacrifice_marker`.
  - **Cube wiring**: white +1 (Bond of Discipline); blue +4 (Tome
    Scour, Repulse, Visions of Beyond, Strategic Planning); black +3
    (Ravenous Rats, Brain Maggot, Sudden Edict); red +1 (Strangle);
    green +1 (Plummet); BR cross-pool +2 (Dreadbore + Bedevil — both
    appear in the black and red pools when paired with the other
    color). Cube prefetch test extended.
  - 17 new tests; lib suite 627 → 644 (+17 — 14 card-functionality
    + 3 engine/UI regression).

- **modern_decks-14: 13 new cube cards + mode-aware target filter + bot modal enumeration + 22 catalog/test parity fixes**:
  - **13 new cards** in `catalog::sets::decks::modern`, all built on
    existing engine primitives:
    - **WB removal**: `vindicate` ({1}{W}{B} sorcery destroy any
      permanent — lands too); `anguished_unmaking` ({1}{W}{B} instant
      exile nonland + lose 3 life); `despark` ({W}{B} instant exile
      mana value ≥ 4 permanent).
    - **Red burn**: `magma_spray` ({R} 2 dmg to creature),
      `skullcrack` ({1}{R} 3 dmg to player), `fiery_impulse` ({R}
      2 dmg to creature), `searing_blood` ({R}{R} 2 dmg to creature),
      `crumble_to_dust` ({2}{R}{R} exile nonbasic land).
    - **Modal removal/counter**: `drown_in_the_loch` ({U}{B} instant
      ChooseMode counter-spell or destroy creature).
    - **Graveyard hate**: `cremate` ({B} instant exile graveyard +
      cantrip; auto-target prefers graveyard cards).
    - **Green ramp**: `harrow` ({2}{G} instant sacrifice land +
      search 2 basics into play untapped — sac folded into resolution).
    - **Lands**: `mortuary_mire` (ETB-tapped Swamp; ETB recurs
      target creature from graveyard to top of library);
      `geier_reach_sanitarium` (legendary land, {T}: Add {C} +
      {1}, {T}: each player loot 1).
  - **Engine: mode-aware target filter for ChooseMode spells**.
    Pre-fix `target_filter_for_slot` returned the FIRST matching
    filter across all modes for a `ChooseMode`, so Drown in the
    Loch's mode 1 (destroy creature) cast with a creature target
    failed because the check matched mode 0's `IsSpellOnStack`
    filter. New `target_filter_for_slot_in_mode(slot, mode)` threads
    the chosen mode in; `cast_spell` and `cast_spell_alternative`
    use it. Activated abilities aren't typically modal so unchanged.
    `primary_target_filter` also walks `ChooseMode` to surface the
    first viable filter for UI/bot target narrowing. Tests:
    `drown_in_the_loch_mode_one_destroys_creature` (would fail under
    the legacy filter).
  - **Bot: modal spell enumeration**. `RandomBot::main_phase_action`
    pre-fix passed `mode: None` for every spell, so Drown in the
    Loch always picked mode 0 (counter spell) — dead in any state
    where no opp spell is on the stack. New `modal_mode_count` +
    `mode_branch` helpers enumerate each mode of a `ChooseMode`
    (top-level or wrapped in `Seq`); the bot tries each and the
    `would_accept` dry-run filter keeps any that pass legality.
    Concrete win: bot picks mode 1 (destroy creature) when no opp
    spell is on the stack. Tests:
    `bot_picks_alternate_mode_for_modal_spell`,
    `modal_mode_count_helper`.
  - **Catalog/test parity sweep (22 → 0 failing tests)**. The lib
    suite had 22 pre-existing test failures, all from
    catalog/test cost / P/T / keyword mismatches that drifted out
    of sync with the doc-comment Oracle costs:
    - **Catalog corrections** (Oracle-faithful): Spectral
      Procession `{2}{W}` (was `{3}{W}{W}{W}`), Devourer of Destiny
      `{5}, 7/5` (was `{7}, 6/6`), Solitude `{3}{W}` (was `{3}{W}{W}`),
      Holy Light `{W}` (was `{2}{W}`), Path of Peace `{2}{W}` (was
      `{3}{W}`), Read the Tides `{3}{U}` (was `{4}{U}{U}`),
      Hieroglyphic Illumination `{2}{U}` (was `{3}{U}`), Loran of
      the Third Path `{1}{W}` (was `{2}{W}`), Lumra Trample (was
      Vigilance + Reach), Biorhythm `{4}{G}{G}{G}` (was `{6}{G}{G}`),
      Sundering Eruption `{1}{R}` (was `{2}{R}`), Static Prison
      `{X}{2}{W}` Enchantment (was `{X}{U}`).
    - **Test corrections**: Quantum Riddler pay `{3}{U}{B}` (was
      `{1}{U}{B}`), Ancient Grudge `{1}{R}` (was `{R}`), Stoke the
      Flames `{2}{R}{R}` (was `{4}{R}`), Rakshasa's Bargain
      `{4}{B}{B}` (was `{4}{B}`), Reanimate Atraxa 7 life (was 8 —
      catalog Atraxa is `{3}{G}{W}{U}{B}` = CMC 7, not 8).
  - **Cube wiring**: WB cross-pool +3 (Vindicate, Anguished
    Unmaking, Despark); red pool +5 (Magma Spray, Skullcrack,
    Fiery Impulse, Searing Blood, Crumble to Dust); UB cross-pool
    +1 (Drown in the Loch); black pool +1 (Cremate); green pool +1
    (Harrow); colorless pool +2 (Mortuary Mire, Geier Reach
    Sanitarium). Cube prefetch test extended.
  - 21 new tests; lib suite 585 → 627 (+42 — 21 new + 22 fixed
    pre-existing + others).

- **modern_decks-13: 12 new cube cards + bot loyalty activation + library-shuffle fix + planeswalker view**:
  - **12 new / upgraded cards** in `catalog::sets::decks::modern`, all
    built on existing engine primitives (with two minor injections):
    - **Reanimator finisher**: `lumra_bellow_of_the_woods` ({4}{G}{G}
      6/6 Trample legendary; ETB returns every land card in your
      graveyard via `Move(EachMatching(Graveyard(You), Land) → BF tapped)`).
    - **UB midrange**: `crabomination` ({2}{U}{B} 3/4 Crab Horror
      legendary, ETB mills each opponent 3, on opp-creature-death scry 1
      — custom card), `cruel_somnophage` ({1}{U}{B} 0/0 Phyrexian Horror,
      P/T = your graveyard size via the same compute-time injection
      Tarmogoyf uses).
    - **Removal / chaos**: `chaos_warp` ({2}{R} instant, shuffle target
      permanent into owner's library via the new `LibraryPosition::Shuffled`
      engine path).
    - **Land tutor**: `elvish_reclaimer` ({1}{G} 1/2 Druid, `{T}, sac a
      land: Search(Land → BF)`).
    - **Mana**: `rofellos_llanowar_emissary` ({G}{G} 0/1 legendary,
      `{T}: Add {G}{G}` — Forest scaling collapsed), `pentad_prism`
      ({2} artifact, ETB with 2 charge counters; remove counter for any
      one color).
    - **Big swing**: `biorhythm` ({4}{G}{G}{G} sorcery, drops each
      opponent 20 life and gains life equal to your creatures —
      approximation of "each player's life total becomes their creature
      count"), `balefire_dragon` ({5}{R}{R} 6/6 Flying, combat-damage
      trigger sweeps opp creatures for 6).
    - **Planeswalkers**: `karn_scion_of_urza` ({4} 5-loyalty walker;
      +1 draw+mill, -1 buff Constructs, -2 make a 1/1 Construct),
      `tezzeret_cruel_captain` ({3}{B} 4-loyalty walker; +1 -2/-2 a
      creature, -2 drain 2 each opponent).
    - **Promotion**: `greasewrench_goblin` 🟡 → ✅ (Treasure-on-death
      trigger now wired via the `treasure_token()` helper).
  - **Engine: bot activates planeswalker loyalty abilities**.
    `RandomBot::main_phase_action` walked planeswalkers under the bot's
    control but never picked a loyalty ability. New `pick_loyalty_ability`
    helper sorts the walker's abilities by descending loyalty cost
    (preserving +1 over -2), filters out abilities the walker can't
    afford, auto-targets via `auto_target_for_effect`, and dry-runs each
    candidate against `would_accept`. Test:
    `bot_activates_planeswalker_loyalty_ability` — Karn picks +1
    (preserves the walker) over -1 / -2.
  - **Engine: `LibraryPosition::Shuffled` actually shuffles**. The
    `move_card_to` arm for `ZoneDest::Library { pos: Shuffled, .. }`
    fell through to a `push` (effectively bottom-of-library), exposing
    a deterministic ordering across cards that semantically should
    randomize. Now we push then `library.shuffle(&mut rng)` so Chaos
    Warp / similar effects land their target at a random library
    position. No test added (the engine's shuffle is randomized on
    purpose), but Chaos Warp is exercised via
    `chaos_warp_sends_target_permanent_to_owners_library`.
  - **UI/server: planeswalker loyalty abilities surface in `PermanentView`**.
    New `LoyaltyAbilityView` wire type carries `{ index, loyalty_cost,
    effect_label, needs_target }`. `view::project_loyalty_abilities`
    populates it for each planeswalker. Pre-fix the wire view only
    carried activated abilities, leaving any UI client blind to walker
    abilities — the bot was the only path that activated them. Test:
    `planeswalker_loyalty_abilities_appear_in_view`.
  - **Engine: `PlaneswalkerSubtype::Tezzeret`** added (the existing
    `karn_scion_of_urza` reuses the existing `PlaneswalkerSubtype::Karn`).
  - **Cube wiring**: green pool +4 (Elvish Reclaimer, Rofellos, Biorhythm,
    Lumra), red pool +2 (Chaos Warp, Balefire Dragon), black pool +1
    (Tezzeret), colorless pool +2 (Karn Scion of Urza, Pentad Prism),
    UB cross +2 (Crabomination, Cruel Somnophage). Prefetch test
    enumerates the 11 new names for early failure on art lookup.
  - 17 new tests; lib suite 587 → 604 (+17).

- **modern_decks-12: 12 new cube cards + bot X-cost handling + auto_target Seq/If fix + Color::short_name**:
  - **12 new cards** in `catalog::sets::decks::modern`, all built on
    existing engine primitives:
    - **Land destruction**: `stone_rain` ({2}{R}, destroy land).
    - **Creature removal**: `bone_splinters` ({B}, sac+destroy),
      `mortify` ({1}{W}{B}, creature/enchantment),
      `maelstrom_pulse` ({1}{B}{G}, nonland permanent),
      `dismember` ({1}{B}{B}, -5/-5 EOT — Phyrexian collapse),
      `celestial_purge` ({1}{W}, exile B/R permanent).
    - **Bounce**: `echoing_truth` ({1}{U}, return nonland permanent).
    - **Card draw**: `hieroglyphic_illumination` ({2}{U}, draw 2 —
      cycling collapsed).
    - **Discard**: `mind_twist` ({X}{B}, target player discards X
      random).
    - **Mass damage**: `earthquake` ({X}{R}, X damage to each
      non-flier and each player).
    - **Mill**: `glimpse_the_unthinkable` ({U}{B}, target mills 10).
    - **Graveyard hate**: `cling_to_dust` ({B}, exile + 2 life if
      creature). Uses `Effect::Move(target → Exile)` so the
      auto-target heuristic walks graveyards via
      `prefers_graveyard_target`, and `Predicate::EntityMatches`
      reads the exiled card's type from the post-move zone.
  - **Engine: bot X-cost handling**: `RandomBot::main_phase_action`
    pre-fix always passed `x_value: None` (treated as 0), so Banefire
    dealt 0 damage, Mind Twist discarded nothing, and Earthquake was
    a no-op when the bot fired them. New `max_affordable_x` helper
    computes the largest X the caster can pay from their current
    mana pool given the fixed (non-X) portion plus any per-card tax
    (Damping Sphere etc.). Detection considers both an explicit `{X}`
    pip in the cost (Wrath of the Skies) **and** an `XFromCost`
    reference inside the effect tree (Banefire / Earthquake / Mind
    Twist — these have flat fixed costs in the catalog because the
    engine had no `Value::XFromCost` wiring at the time they were
    added; the X mana still flows through `EffectContext.x_value`).
    `effect_uses_x` walks Effect / Predicate / Value combinators
    recursively so nested `XFromCost` references inside Seq / If /
    ChooseMode / ForEach / Repeat / DelayUntil all surface.
  - **Engine: auto_target Seq/If primary-child**:
    `Effect::accepts_player_target` for `Seq` and `If` was an
    unconditional `any`/`||` over the children, which over-reported
    when a trailing branch (e.g. a conditional `GainLife`) accepted
    Player but the leading branch dictated the actual target slot.
    Concrete repro: Cling to Dust's effect tree is
    `Seq([Move(target → Exile), If(EntityMatches Creature, GainLife)])`.
    The Move's `Any` filter is the spell's actual target slot, but
    `accepts_player_target` returned `true` because the trailing
    `GainLife` could take a Player — the bot picked Player(opp), the
    legality check passed (Any matches Player), and resolution
    silently fizzled (Move only consumes Permanent/Card refs). Fix:
    in Seq, defer to the first child whose `primary_target_filter`
    is set; in If, prefer the `then` branch. Falls back to the old
    `||` behavior when no child surfaces a filter.
  - **UI: `Color::short_name()` + `impl Display`**: cost-label
    formatter was rendering colored mana pips via Rust's `Debug` —
    `{Red}`, `{White}` — instead of the standard MTG `{R}`, `{W}`.
    New `Color::short_name()` returns the single-letter abbreviation;
    new `impl fmt::Display for Color` dispatches to it so format
    strings can interpolate `{c}` for the pip text.
    `view::ability_cost_label` switches to `{c}` interpolation.
    `cube::color_pair_name` collapses to the new helper.
  - **Cube wiring**: red gets Stone Rain + Earthquake; black gets
    Bone Splinters + Mind Twist + Dismember + Cling to Dust; blue gets
    Hieroglyphic Illumination + Echoing Truth; white gets Celestial
    Purge; Mortify added to the WB cross-pool; Maelstrom Pulse added
    to the BG cross-pool; Glimpse the Unthinkable in the UB
    cross-pool. Prefetch test extended.
  - 27 new tests; lib suite 559 → 587 (+28; cube prefetch test
    accounts for one additional pass-through).

- **modern_decks-11: 14 new cube cards (surveil lands + multicolor removal + sweepers)**:
  - **7 surveil lands** filling out the Murders at Karlov Manor cycle
    in `catalog::sets::decks::modern`: `underground_mortuary` (BG),
    `lush_portico` (GW), `hedge_maze` (UG), `thundering_falls` (UR),
    `commercial_district` (RW), `raucous_theater` (BR), and
    `elegant_parlor` (RG — fixed from the spec's RW typo). All seven
    reuse the existing `dual_land_with` + `etb_tap_then_surveil_one`
    helpers from `sets/mod.rs`, so the wiring is a one-liner per land.
  - **7 spell/creature cards** built on existing engine primitives:
    - **BG removal**: `tear_asunder` ({1}{B}{G} destroy
      artifact/enchantment — kicker mode collapsed),
      `assassins_trophy` ({B}{G} destroy any opp non-land permanent —
      basic-search downside collapsed).
    - **Sweepers**: `volcanic_fallout` ({1}{R}{R} 2 dmg to each
      creature and player via dual ForEach; uncounterable rider
      dropped), `rout` ({3}{W}{W} DoJ+1 sorcery), `plague_wind`
      ({8}{B}{B} one-sided creature sweep).
    - **Body**: `carnage_tyrant` ({4}{G}{G} 7/6 Dinosaur with Trample
      + Hexproof + `Keyword::CantBeCountered` — the engine respects
      the keyword in `CounterSpell`).
    - **Mana engine**: `krark_clan_ironworks` ({4} Artifact, sac an
      artifact: add {2}; cost folded into resolution via
      `SacrificeAndRemember + AddMana`).
  - **Cube wiring**: surveil lands enter both relevant cross-color
    pools (mirror of the talisman cycle); BG cards (Tear Asunder,
    Assassin's Trophy) sit in both Black and Green pools when paired
    cross-color; mono-color cards in their primary pool. KCI joins
    `colorless_pool`. The cube prefetch test explicitly enumerates
    all 14 new names so the Scryfall art prefetch catches them.
  - **Cleanup**: `view::ability_effect_label` wildcard arm gets a
    comment explaining it's intentionally non-exhaustive (so adding
    a new `Effect` variant doesn't break the build, but flagged
    cards prompt a dedicated arm).
  - 15 new tests; lib suite 545 → 559+ (12 card-functionality + 3
    surveil-land coverage).

- **modern_decks-10: 16 new cube cards + graveyard-target auto-target**:
  - **16 new cards** in `catalog::sets::decks::modern` covering the
    Locus / bridge / utility-artifact axis the cube was missing:
    - **Lands** (14): `glimmerpost` (Locus, ETB tap + 1 life),
      `cloudpost` (Locus, ETB tap), `lotus_field` (ETB tap + sac 2
      lands; tap for 3 of one color), `evolving_wilds` (ETB tap +
      `{T}, sac: search basic`), and the full 10-bridge cycle
      (`mistvault_bridge` / `drossforge_bridge` / `razortide_bridge`
      / `goldmire_bridge` / `silverbluff_bridge` / `tanglepool_bridge`
      / `slagwoods_bridge` / `thornglint_bridge` / `darkmoss_bridge`
      / `rustvale_bridge`). All bridges share the `bridge_land()`
      helper — ETB tapped, two basic land types, `{T}: Add {C}`.
    - **Utility artifacts** (2): `coalition_relic` (`{T}: Add one
      mana of any color`), `ghost_vacuum` (`{2}, {T}: exile target
      card from a graveyard`).
  - **Engine: graveyard-target auto-target preference**: renamed
    `Effect::prefers_graveyard_source` → `prefers_graveyard_target`
    and extended it to also match `Move target → Exile`. The
    `auto_target_for_effect` heuristic now walks graveyards (in
    `primary_player`→`secondary_player` order) BEFORE the
    battlefield when the flag is set. Reanimate spells (Disentomb,
    Raise Dead, Reanimate, Goryo's) keep their friendly-graveyard-
    first preference; graveyard-hate spells like Ghost Vacuum hit
    the opp's graveyard first. Without this fix the bot's
    auto-target was picking a battlefield permanent for Ghost
    Vacuum (filter `Any`) and silently exiling it. Test:
    `ghost_vacuum_auto_target_picks_graveyard_card_when_present`.
  - **Cube wiring**: utility lands + artifacts in `colorless_pool`;
    each bridge lands in both relevant cross-color extension
    blocks (talisman-style).
  - 13 new tests; lib suite 532 → 545.

- **modern_decks-9: 10 more cards + engine `primary_target_filter` fix**:
  - **10 new cards** in `catalog::sets::decks::modern`, all on
    existing primitives:
    - **Discard**: `despise` ({B}, creature/PW only),
      `distress` ({B}{B}, nonland-noncreature).
    - **Cost-tax body**: `vryn_wingmare` ({2}{W} 2/1 Flying;
      noncreature-spell {1} tax — Thalia mirror).
    - **Burn**: `lava_coil` ({1}{R}, 4 damage to creature),
      `jayas_greeting` ({1}{R} instant, 3 damage + scry 2).
    - **Card draw**: `telling_time` ({1}{U} scry 2 + draw 1),
      `read_the_tides` ({3}{U} draw 3 — Concentrate at off-color).
    - **Removal**: `last_gasp` ({1}{B} -3/-3 EOT — Disfigure +1).
    - **Discard outlet**: `wild_mongrel` ({1}{G} 2/2 Hound; discard
      a card → +1/+1 EOT — Psychic Frog mirror).
  - **Engine: `primary_target_filter` covers Discard / Mill / Draw /
    Drain / AddPoison**. Pre-fix Mind Rot's bare `Selector::Target(0)`
    fell through the match arms and `auto_target_for_effect`
    returned None — the random bot couldn't cast Mind Rot. Adding
    the player-side cases lets the heuristic surface the filter via
    `target_filtered(Player)`. While here, switched Mind Rot's
    selector to use the explicit filter for self-documentation.
    Tests: `auto_target_mind_rot_picks_opponent_player`,
    `auto_target_player_side_effects_resolve_via_filter`.
  - **Refactor**: the 23 modern_decks-8 cards (and the 10 added
    here) use `..Default::default()` for irrelevant fields. -253
    lines vs the boilerplate-heavy form. Same behavior — verified
    by 532 lib tests still passing.
  - 12 new tests; lib suite 520 → 532.

- **modern_decks-8: 23 new cards (claude/modern_decks)**:
  - **23 new cards** in `catalog::sets::decks::modern`, all built on
    existing engine primitives (no engine changes required for the
    cards themselves):
    - **Burn**: `incinerate` ({1}{R}), `searing_spear` ({1}{R}),
      `flame_slash` ({R} sorcery, 4 dmg to a creature),
      `roast` ({1}{R} sorcery, 5 dmg to a non-flier).
    - **Black removal**: `smother` ({1}{B}, destroy creature with
      cmc ≤ 3), `final_reward` ({4}{B}, exile creature).
    - **White**: `holy_light` ({W}, all creatures -1/-1 EOT),
      `mana_tithe` ({W}, white Force Spike),
      `path_of_peace` ({2}{W}, destroy + opp gains 4 life via
      `PlayerRef::ControllerOf`).
    - **Green ramp**: `rampant_growth` ({1}{G}),
      `cultivate` ({2}{G}, Kodama's Reach mirror),
      `farseek` ({1}{G}, basic-only collapse),
      `sakura_tribe_elder` ({1}{G} Snake; tap-and-sac search basic),
      `wood_elves` ({2}{G} ETB Forest-tutor untapped),
      `elvish_mystic` ({G} mana dork twin of Llanowar Elves).
    - **Card draw**: `harmonize` ({2}{G}{G} Draw 3),
      `concentrate` ({2}{U}{U} Draw 3),
      `anticipate` ({1}{U} Scry 2 + Draw 1),
      `divination` ({2}{U} Draw 2),
      `ambitions_cost` ({3}{B} Draw 3 + Lose 3 life).
    - **Misc**: `severed_strands` ({1}{B} sac creature, destroy
      creature, gain 2 life — sac-as-additional-cost folded into
      resolution).
  - **Cube wiring**: each card placed in its primary color pool
    (white +3, blue +3, black +4, red +4, green +6 — no two-color
    cards in this batch).
  - **Tests**: 25 new functionality tests in `tests/modern.rs`.
    Lib suite 495 → 520.

- **modern_decks-7: 16 new cards + auto-target Player-skip + clippy/server cleanup**:
  - **16 new cards** in `catalog::sets::decks::modern` (all built on
    existing engine primitives — no engine changes required for the cards
    themselves):
    - **Bounce**: `unsummon` ({U}), `boomerang` ({U}{U}), `cyclonic_rift`
      ({1}{U}, opp-only filter; overload mode still ⏳), `repeal`
      ({X}{U}, `If(ManaValueOf(Target) ≤ XFromCost, bounce, Noop) +
      Draw 1`).
    - **Removal**: `murder` ({1}{B}{B}), `go_for_the_throat` ({1}{B}, no
      artifact creatures), `disfigure` ({B}, -2/-2), `languish` ({2}{B}{B},
      sweeper -2/-2), `lay_down_arms` ({W}, exile creature with
      power ≤ 4), `smelt` ({R}, artifact destroy), `shatter` ({1}{R},
      artifact destroy).
    - **Burn**: `banefire` ({X}{R}, X to any target).
    - **Tokens**: `spectral_procession` ({2}{W}, three 1/1 white Spirit
      flying tokens — hybrid pips collapsed).
    - **Recursion**: `regrowth` ({1}{G}, any-card from your graveyard to
      hand).
    - **Combo removal**: `beast_within` ({2}{G}, destroy permanent + 3/3
      Beast token to that permanent's controller via
      `PlayerRef::ControllerOf`), `grasp_of_darkness` ({B}{B}, -4/-4 EOT).
  - **Engine: skip Player target in `auto_target_for_effect` for
    permanent-only effects**. The auto-target heuristic short-circuited
    at the Player(controller)/Player(opp) candidates before looking at
    permanents. For effects whose filter accepted a Player ref (e.g.
    `Any` on Regrowth's Move-target), a Player target was returned even
    though `Effect::Move` only consumes EntityRef::{Permanent,Card} —
    the cast then silently fizzled at resolution. Fix: new
    `Effect::accepts_player_target()` classifies effects by whether a
    Player target is meaningful (DealDamage/GainLife/LoseLife/Discard/
    Drain/Mill/AddPoison/Draw → yes; Move/Destroy/Exile/Tap/PumpPT/etc.
    → no). The auto-target heuristic skips the Player rung when this
    returns false. Tests: `auto_target_regrowth_skips_player_in_favor_of_graveyard_card`,
    `auto_target_boomerang_picks_a_permanent_not_a_player`.
  - **Engine cleanup**: `continue_spell_resolution` had 8 args (clippy
    warning); annotated with `#[allow(clippy::too_many_arguments)]` +
    rationale comment (the spell-state quartet must be threaded across
    suspend/resume so the spell can re-run with original cast-time
    choices, and the two callers both unpack directly from
    StackItem::Spell / ResumeContext::Spell — wrapping in a struct
    doesn't reduce coupling).
  - **Server**: moved `mod tests` to the bottom of
    `crabomination_server::main` to silence
    `clippy::items_after_test_module`.
  - **Cube wiring**: each card placed in its primary color pool
    (blue +4, black +5 incl. Grasp, red +3 incl. Shatter, white +2,
    green +2 incl. Beast Within).
  - 24 new tests; suite 471 → 495 (+24).

- **modern_decks-5: 12 new cards + friendly-target heuristic + CRAB_FORMAT validation**:
  - **12 new cards** in `catalog::sets::decks::modern` (all built on existing primitives):
    - Talisman cycle (`talisman_of_conviction`, `talisman_of_creativity`, `talisman_of_curiosity`) — shared `talisman_cycle` helper.
    - Edict cycle (`innocent_blood`, `diabolic_edict`, `geths_verdict`).
    - Burn / interaction (`magma_jet`, `remand`, `read_the_bones`, `ancient_grudge`, `tragic_slip`).
    - Vanilla flying body (`storm_crow`).
  - **Engine: `Effect::prefers_friendly_target`** — classifies non-negative `PumpPT` / `GrantKeyword` / `+1/+1 AddCounter` as friendly buffs. `auto_target_for_effect` now prefers the *caster*'s permanent first for friendly effects, opp first for hostile. Random bot stops pumping the opp's bear with Vines of Vastwood / Reckless Charge.
  - **Engine: `add_card_to_graveyard` test helper** to seed reanimate / flashback / dredge fixtures.
  - **Server: warn on unrecognized `CRAB_FORMAT`** — silent fallback was masking typos. Three new tests in `crabomination_server::main::tests`.
  - 16 new tests; lib suite 421 → 423; server suite +3.

- **modern_decks-3: 13 new cards + engine/server/bot improvements (claude/modern_decks)**:
  - **13 new cards** in `catalog::sets::decks::modern` (built entirely on existing engine primitives, no engine changes required):
    - `cathartic_reunion`, `gitaxian_probe`, `force_spike`, `vampiric_tutor`, `sylvan_scrying`, `abrupt_decay`, `kodamas_reach`, `lotus_petal`, `tormods_crypt`, `mishras_bauble`, `stoneforge_mystic`, `qasali_pridemage`, `greater_good`.
    - 16 new functionality tests in `tests/modern.rs` (one per card; some have multiple tests covering both happy/rejection paths).
  - **Engine: opp-preference auto-target**: `auto_target_for_effect` now does a two-pass battlefield scan — first only opponent-controlled permanents, then everything. Stops the random bot from auto-destroying its own creatures with hostile spells like Doom Blade, Abrupt Decay, etc. Test: `auto_target_prefers_opponent_permanent_for_hostile_effect`.
  - **`Target` derives `PartialEq + Eq`**: lets test fixtures and downstream code compare `Target` values directly.
  - **UI/view: `ability_effect_label` completeness**: added labels for `LookAtTop`, `ShuffleGraveyardIntoLibrary`, `PutOnLibraryFromHand`, `RevealTopAndDrawIf`, `CopySpell`, `GainControl`, `ResetCreature`, `BecomeBasicLand`, `Attach`, `GrantSorceriesAsFlash`, `NameCreatureType`. Activated abilities using these effects now render with a meaningful label instead of the catch-all "Activate".
  - **Server bot: free-mana-rock activation**: `RandomBot` now finds and activates free-tap mana rocks (Sol Ring, Mind Stone, Fellwar Stone-class) after lands are exhausted. Sac-cost mana sources (Lotus Petal, Chromatic Star) and color-choice abilities (Ornithopter of Paradise) are filtered out — both can mis-trigger or block on `ChooseColor`. Tests: `bot_taps_free_mana_rock_after_lands`, `bot_does_not_tap_sac_cost_mana_source`, `bot_does_not_tap_color_choice_mana_source`.
  - **Standalone server: accept back-off + Format::Copy simplification**: `crabomination_server::main` factored its TCP `accept` calls behind `accept_with_backoff`, which logs the OS error and sleeps 100ms before returning so a transient EAGAIN/EMFILE doesn't burn CPU. Also dropped the `Arc<Format>` wrapping — `Format` is `Copy`, so each match thread captures a fresh copy via the `move` closure.

- **modern_decks-2: 10 new cards + DECK_FEATURES finishes + engine cleanup**:
  - **DECK_FEATURES finishes** (🟡 → ✅):
    - **Buried Alive** now uses `Repeat(3, Search(Creature → Graveyard))` to
      cover the real "up to three" Oracle. The decider can answer
      `DecisionAnswer::Search(None)` to opt out of any iteration — the
      existing `do_search` pathway already honors a `None` answer. Tests:
      `buried_alive_searches_creature_into_graveyard`,
      `buried_alive_pulls_up_to_three_creatures`.
    - **Sign in Blood** is now a real "target player" effect — both
      `Draw` and `LoseLife` run against `target_filtered(Player)` /
      `Selector::Player(PlayerRef::Target(0))`. Auto-target picks the
      opponent when no manual target is supplied; the existing
      "self-cantrip" line still works by passing `Target::Player(0)`
      explicitly. Tests:
      `sign_in_blood_draws_two_loses_two_life`,
      `sign_in_blood_can_target_opponent`.
    - **Reanimate** (Tempest) — the engine's `Tempest` `reanimate` was
      a stub that just moved the card. Replaced with the real Oracle:
      `LoseLife(ManaValueOf(Target(0))) + Move(target → BF)`. The
      life-loss runs first so `Value::ManaValueOf` resolves while the
      target is still in the graveyard. Tests:
      `reanimate_puts_creature_into_play_and_pays_cmc_life`,
      `reanimate_life_cost_scales_with_mana_value`.
  - **CUBE_FEATURES new cards** (⏳ → ✅):
    - **Bone Shards** ({B} instant) — `ChooseMode([Sacrifice, Discard]) +
      Destroy(target creature)`. Modal additional cost folded into
      resolution.
    - **Pyrokinesis** ({4}{R}{R} instant) — pitch-cost alt cast: exile
      a red card from hand → 4 damage. Reuses
      `AlternativeCost.exile_filter`.
    - **Tishana's Tidebinder** ({1}{U}{U} 3/2 Merfolk Wizard Flash) — ETB
      counters a target activated/triggered ability of a nonland
      permanent. Reuses `Effect::CounterAbility`.
    - **Sylvan Safekeeper** ({G} 1/1 Human Wizard) — Sacrifice a Forest:
      target creature gains shroud EOT. Cost folded into resolution.
    - **Grim Lavamancer** ({R} 1/1 Human Wizard) — `{R}, {T}: 2 damage to
      any target`. Graveyard-exile cost still 🟡 pending an exile-from-
      graveyard primitive.
    - **Zuran Orb** ({0} artifact) — sac a land for 2 life.
    - **Chromatic Star** ({1} artifact) — sac for any color of mana +
      cantrip on leaving the battlefield.
    - **Soul-Guide Lantern** ({1} artifact) — both abilities wired
      (target-opp graveyard exile approximated as each-opp).
    - **Cankerbloom** ({1}{G} 2/2 Fungus) — sac to destroy artifact/
      enchantment + proliferate.
  - **Engine: non-creature die-triggers fire**:
    `remove_to_graveyard_with_triggers` now also collects
    `EventKind::PermanentLeavesBattlefield` self-source triggers, in
    addition to `CreatureDied`. The latter still gates on creature-only
    so Solitude evoke-sac etc. behave the same; the new path lets
    Chromatic Star (and any future non-creature die-trigger) fire when
    sacrificed/destroyed instead of silently fizzling.
  - **Engine: `Selector::CardsInZone` aggregates across players**: the
    resolver was using single-player `resolve_player`, which returns
    `None` on multi-player refs and silently produced an empty list.
    Switched to multi-player `resolve_players` so `EachPlayer` /
    `EachOpponent` collects cards from every matching seat. Powers
    Soul-Guide Lantern's mass graveyard exile and any future
    "each-player's-graveyard" effect.
  - **Engine cleanup**: dropped `player_has_sorceries_as_flash` (the
    static-flag detour was never reached — only the per-player flag set
    by `Effect::GrantSorceriesAsFlash` is consulted) and the unused
    `ActivatedAbility`/`ManaCost` imports inside `callous_sell_sword`.
    All warnings now resolve.
  - **Server: format selector**: `crabomination_server` now honors
    `CRAB_FORMAT=cube` to build matches with random two-color cube decks
    (`build_cube_state`) instead of the default BRG/Goryo's demo. Format
    is read once at boot and threaded into match threads via `Arc<Format>`.
  - **Cube wiring**: 9 of the 10 new cards are now part of the cube card
    pool (Reanimate already was, but as a stub).
  - 16 new tests in `tests/modern.rs` (104 → 120 tests in that file).
    Total suite: 320 tests pass.

- **Modern shocklands + creatures + auxiliary instants on top of `mod_set`**:
  - **Lands** (6 new): Sacred Foundry, Steam Vents, Stomping Ground,
    Temple Garden, Breeding Pool, Blood Crypt — all six remaining Ravnica
    shocklands, sharing the demo decks' `shockland_pay_two_or_tap` ETB
    trigger. The trigger and `dual_land_with` / `fastland_etb_conditional_tap`
    / `etb_tap_then_surveil_one` / `etb_tap` helpers are extracted from
    `decks/lands.rs` into `sets/mod.rs` so the new modern lands reuse them.
  - **Creatures / enchantments** (4): Thalia, Guardian of Thraben (via
    `StaticEffect::AdditionalCostAfterFirstSpell` filtered to Noncreature),
    Dark Confidant (upkeep draw + flat 2-life approximation), Bloodghast
    (Haste — the "haste while opp ≤ 10" gating still ⏳), and Phyrexian
    Arena.
  - **Auxiliary instants** (6, in `mod_set/spells.rs`): Disenchant,
    Naturalize, Nature's Claim (controller-lifegain via
    `PlayerRef::ControllerOf`), Negate, Dispel (instant-only filter), and
    Dovin's Veto (uses the new `Keyword::CantBeCountered` flag — see below).
  - **Engine: spell-level uncounterable keyword**:
    `caster_grants_uncounterable` now also returns true for any spell whose
    definition lists `Keyword::CantBeCountered`. The cast paths already
    stamp `StackItem::Spell.uncounterable = true` from this helper, and
    both `CounterSpell` and `CounterUnlessPaid` already skip flagged
    items. Wires Dovin's Veto.
  - **Engine: `auto_target_for_effect` looks in graveyards/exile**: the
    auto-target helper used by triggered abilities and the rebound
    re-target path now falls through battlefield → each player's graveyard
    → exile when no battlefield permanent satisfies the requirement. Lets
    reanimate-style spells (Goryo's Vengeance, Animate Dead, Reanimate)
    auto-pick a graveyard target when no manual one is supplied.
  - **Server**: standalone `crabomination_server` now cleanly
    `shutdown(Both)`s seat 0 when seat 1's accept fails, instead of
    dropping silently — the orphaned client gets EOF instead of a hung
    socket.
  - 13 new tests in `tests/modern.rs` covering shockland typing + life
    payment, Disenchant, Nature's Claim controller lifegain, Negate /
    Dispel target restrictions, Dovin's Veto's uncounterable flag,
    Thalia's tax, and Phyrexian Arena's upkeep trigger.


- **Opening-hand effects + Damping Sphere lands + Teferi +1**:
  - **Opening-hand effects**: new `OpeningHandEffect` enum (`StartInPlay { tapped, extra }`, `RevealForDelayedTrigger { kind, body }`, `MulliganHelper`) + `CardDefinition.opening_hand` field. `apply_opening_hand_effects` runs post-mulligan and dispatches: pulls the card to the battlefield (Leyline, Gemstone Caverns), or registers a delayed trigger that fires later (Chancellor of the Tangle, Chancellor of the Annex). Powers: Leyline of Sanctity (with new `StaticEffect::ControllerHasHexproof` + `player_has_static_hexproof` check on `Target::Player(_)` legality), Gemstone Caverns (with luck counter + dual {T} abilities), Chancellor of the Tangle (mana ritual on first main via new `DelayedKind::YourNextMainPhase`), Chancellor of the Annex (cost-tax via new `Player.first_spell_tax_charges` + `Effect::AddFirstSpellTax` + `consume_first_spell_tax` in cast paths), Serum Powder (mulligan-helper flag surfaced via new `Decision::Mulligan.serum_powders` field + new `DecisionAnswer::SerumPowder(id)`; client renders one button per powder ID).
  - **Damping Sphere lands clause**: new `StaticEffect::LandsTapColorlessOnly` + `play_land` consults `lands_tap_colorless_only_active()` and rewrites multi-mana lands' abilities to a single `{T}: Add {C}` on entry. Single-color basics pass through. Tests: `damping_sphere_downgrades_dual_lands_to_colorless`, `damping_sphere_leaves_basic_lands_alone`.
  - **Teferi +1 sorceries-as-flash**: new `Player.sorceries_as_flash` flag + `Effect::GrantSorceriesAsFlash`. All three cast paths consult the flag to bypass the sorcery-timing gate; `do_untap` clears it on the controller's next turn. Test: `teferi_plus_one_grants_sorceries_as_flash_until_next_turn`.
  - **Spoils of the Vault → real reveal-until-find**: new `Effect::RevealUntilFind { who, find, to, cap, life_per_revealed }`. Walks the top of the library until a match (or cap), mills the misses, deducts `life_per_revealed × revealed` life. With `find: Any` exactly one card is taken — the "name a card" half is still pending a naming primitive. Tests: `spoils_of_the_vault_reveals_until_find`, `reveal_until_find_caps_at_n_when_no_match`.
  - **Plunge into Darkness mode 1**: pay 4 life + tutor any card (approximation of "pay X life, look at top X, take one"). Test: `plunge_into_darkness_mode_one_pays_four_life_and_tutors`.
  - **Atraxa per-card-type ETB**: new `Value::DistinctTypesInTopOfLibrary { who, count }` — counts distinct card types in the top N of `who`'s library. Atraxa now draws that count instead of a flat 4. Tests: `atraxa_grand_unifier_etb_draws_per_distinct_type`, `atraxa_grand_unifier_draws_per_card_type_diverse_library`.
  - **Callous Sell-Sword ETB**: ETB sacrifices a controlled creature, gets `+(SacrificedPower)/+0` until end of turn. Reuses the Thud `SacrificeAndRemember` + `Value::SacrificedPower` primitives. Casualty's "copy this spell" branch still ⏳.
  - **Engine cleanup**: consolidated three duplicated `effect_needs_target`-style helpers (one in `view.rs`, one in `bot.rs`, one inline) into the canonical `Effect::requires_target`. The duplicates each missed a different subset of target-bearing effects. Extended `view.rs:ability_effect_label` to walk Seq/If/ChooseMode/ForEach/Repeat combinators and added labels for every `Effect` variant the catalog uses.
  - **Bug fix**: `deal_to_hand` was using `library.pop()` (which removes from the bottom) instead of `library.remove(0)` (top). Visible only on test fixtures that populate libraries deterministically; production always shuffles.
  - **Client UI**: mulligan modal renders one button per Serum Powder ID with the keyboard shortcut **P**.
  - **Tests**: 18 new tests (160 → 178).
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
