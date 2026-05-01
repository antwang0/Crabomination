# Cube Implementation Tracker

Tracking the work to make this Modern-style cube fully playable. The cube
is built around a mix of efficient creatures, interaction, value engines,
and combo lines — see the maybeboard at the end of this file.

The catalog already implements the cards used by the BRG / Goryo's demo
decks (`DECK_FEATURES.md` is the source of truth there); some of those
cards overlap with the cube and are flagged ✅ here. Most of the cube is
still ⏳.

## Legend

- ✅ done — already wired in `crate::catalog` with full functionality
- 🟡 partial — exists with a simplified or stub effect; key behavior missing
- ⏳ todo — not yet implemented

## Cards

### White

| Card | Status | Notes |
|---|---|---|
| Descendant of Storms | ⏳ | Spirit token + tap-trigger; needs the Spirit creature-token primitive. |
| Cathar Commando | ✅ | Flash + {1}, sac: destroy artifact/enchantment (`ActivatedAbility::sac_cost`). |
| Containment Priest | 🟡 | Body wired: 2/2 W flash. The replacement effect ("nontoken creatures entering not from a spell get exiled instead") needs an ETB-replacement primitive the engine doesn't have yet — the body is in the cube as a flash flier replacement until the primitive lands. Test: `containment_priest_is_a_flash_two_two`. |
| Lion Sash | ⏳ | Equipment + grow via exile-from-graveyard. Needs equipment + counters wiring. |
| Elite Spellbinder | ⏳ | ETB look-at-opp-hand + cost-tax static while in play. Reuses `AdditionalCostAfterFirstSpell`-style hooks. |
| Enduring Innocence | ⏳ | Lifelink + draw-on-creature-ETB trigger; "Roomba"-style return-from-exile post-death. Needs ETB-other listener + delayed self-revive. |
| Flickerwisp | ✅ | 3/1 Flying; ETB exile target permanent + `DelayUntil(NextEndStep, Move-back-to-OwnerOf)`. |
| Heliod, Sun-Crowned | 🟡 | 3/4 Legendary Indestructible Creature/Enchantment. **Activated** `{1}{W}: target creature gains lifelink until end of turn` wired. **Triggered** "whenever you gain life, put a +1/+1 counter on target creature you control with lifelink" wired via `LifeGained`+`YourControl` + `AddCounter` on a `Creature ∧ ControlledByYou ∧ HasKeyword(Lifelink)` filter — the Walking-Ballista combo line is now reachable. Devotion-based "isn't a creature unless devotion ≥ 5" still ⏳. Tests: `heliod_sun_crowned_grants_lifelink_until_end_of_turn`, `heliod_adds_plus_one_counter_when_you_gain_life_with_lifelink`. |
| Loran of the Third Path | ✅ | 1/3 Vigilance; ETB destroy artifact/enchantment; {T}: you and target opponent each draw a card. |
| Ranger-Captain of Eos | 🟡 | ETB tutor for ≤1-CMC creature. The sac-for-no-noncreature-spells static is omitted (no sac-as-cost activation primitive). |
| Restoration Angel | ✅ | Flash + ETB exile-and-return target non-Angel creature you control (`Exile + Move-back` flicker pattern). |
| Guardian Scalelord | ⏳ | Flying + grant flying to attackers via attack trigger. |
| Serra Angel | ✅ | Already in `crate::catalog::serra_angel` (4/4 flying + vigilance). |
| Intervention Pact | ⏳ | Free prevent-damage + delayed `PayOrLoseGame` upkeep cost (reuses Pact primitive). |
| Isolate | ✅ | Exile target permanent with mana value 1 (`ManaValueAtLeast(1) ∧ ManaValueAtMost(1)` filter). |
| Tempt with Bunnies | ⏳ | Tempting offer (chain-creating) — needs multi-player choice primitive. |
| Static Prison | 🟡 | `{X}{2}{W}` Enchantment. ETB stamps `Value::XFromCost` Stun counters on itself **and** taps the targeted permanent immediately. The "while it has stun counters, target doesn't untap" suppression and the upkeep counter-removal step still ⏳ (no untap-replacement primitive). The `XFromCost` → trigger context propagation is also still pending; for now the counter count reads as 0 inside the ETB trigger. Tap-target half is fully functional. Test: `static_prison_etb_taps_target`. |
| Virtue of Loyalty | ⏳ | Adventure + enchantment side. Needs Adventure cost-mode primitive. |
| Healing Salve | ✅ | {W} Instant. Gain 3 life on target. Damage-prevention mode collapsed (no prevention-shield primitive). Test: `healing_salve_gives_three_life`. |
| Raise the Alarm | ✅ | {1}{W} Instant. Create two 1/1 white Soldier tokens via `CreateToken`. Test: `raise_the_alarm_creates_two_soldier_tokens`. |

### Blue

| Card | Status | Notes |
|---|---|---|
| Mockingbird | ⏳ | ETB copy-the-name-of a creature you control. |
| Dandân | 🟡 | {2}{U} 4/1 Fish. Wired with the "if you control no Island at your upkeep, sacrifice this" downside (an upkeep `If(Not(SelectorExists(Island ∧ ControlledByYou)), Move(self → Graveyard))` trigger). The "can attack only if defending player controls an Island" half is omitted (no per-attacker target restriction). Tests: `dandan_sacrifices_at_upkeep_when_no_island`, `dandan_stays_in_play_with_an_island`. |
| Phantasmal Image | ⏳ | ETB enter-as-copy of any creature. Needs token-copy-of-permanent primitive. |
| Thundertrap Trainer | ⏳ | Trap/discount creature. |
| Tishana's Tidebinder | ✅ | {1}{U}{U} 3/2 Merfolk Wizard Flash. ETB counters target activated/triggered ability of a nonland permanent (reuses `Effect::CounterAbility`). |
| Quantum Riddler | ✅ | UB 4/4 flying + on-cast Draw 1 (already in catalog). |
| Deadeye Navigator | ⏳ | Soulbond + activated flicker. Reuses Flicker primitive; needs Soulbond. |
| Pact of Negation | ✅ | Free counter + delayed `PayOrLoseGame` upkeep. |
| Consider | ✅ | Surveil 1 then Draw 1 (reuses Surveil + Draw primitives). |
| Spell Snare | ✅ | Counter target spell with mana value 2 (CounterSpell + `ManaValueAt{Most,Least}(2)` sandwich). |
| Cancel | ✅ | {1}{U}{U} Instant. Counter target spell. Test: `cancel_counters_a_spell`. |
| Annul | ✅ | {U} Instant. Counter target artifact or enchantment spell — cast-time filter rejects instants/sorceries/creatures. Test: `annul_rejects_instant_target_at_cast_time`. |
| Swan Song | 🟡 | Counter enchantment/instant/sorcery; 2/2 Flying Bird token. (Token goes to caster's opponents — equivalent in 2-player; engine has no `ControllerOf` lookup for stack/graveyard cards.) |
| Thought Scour | ✅ | Mill 2 + draw 1. |
| Consult the Star Charts | ⏳ | Look-at-top-N + draw — needs Foretell-adjacent decision. |
| Daze | 🟡 | Counter target spell unless its controller pays {1}. The "return an Island" alt cost is omitted (alt-cost model only supports exile-from-hand). |
| Lose Focus | 🟡 | Counter target spell unless its controller pays {2} (`Effect::CounterUnlessPaid`). Castable at full {U} cost; the Delve cost-reduction is omitted. Tests: `lose_focus_counters_when_controller_cannot_pay_two`, `lose_focus_does_not_counter_when_controller_can_pay_two`. |
| Frantic Search | 🟡 | Draw 2, discard 2, untap your tapped lands (approximation: untaps every tapped land you control rather than "up to three"). |
| Cryptic Command | 🟡 | `{1}{U}{U}{U}` Instant. **Choose two** is collapsed to a single `ChooseMode` of four bundled pairs: `[counter+bounce, counter+tap-opp-creatures, counter+draw, bounce+draw]`. AutoDecider picks mode 0 (counter+bounce). The "tap all creatures your opponents control" half uses `ForEach + Tap`. A multi-pick "choose any two" mode primitive is the gap. Tests: `cryptic_command_counter_plus_bounce_resolves`, `cryptic_command_mode_two_counter_and_draw`. |
| Paradoxical Outcome | ✅ | Return each non-land permanent you control + draw equal (`ForEach + Move + Draw 1`). |
| Turnabout | ✅ | {2}{U}{U} Instant. `ChooseMode` over six tap/untap branches (artifact / creature / land × tap / untap), each operating against `EachPermanent(Type ∧ ControlledByOpponent)`. AutoDecider picks mode 4 (tap all opponent lands). Test: `turnabout_mode_four_taps_all_opponent_lands`. |
| Gush | ⏳ | Free draw if pitching two Islands; alternative cost variant. |
| Gather Specimens | ⏳ | Replace creature ETB control-shift. Replacement effect primitive. |
| Mirrorform | ⏳ | Aura + clone target. |
| Dig Through Time | ⏳ | Delve + look at top 7, take 2. Multi-pick primitive. |
| Windfall | 🟡 | Each player discards their hand and draws 7 (the dynamic "max discarded" yield is collapsed to a constant — same simplification as Wheel of Fortune). Test: `windfall_discards_both_hands_and_draws_seven`. |
| Mind's Desire | ⏳ | Storm + cast-from-top. Needs Storm count + cast-from-top primitive. |
| Upheaval | ✅ | Return all permanents to their owners' hands (`ForEach + Move → Hand(OwnerOf)`). |
| Treasure Cruise | 🟡 | Draw 3 wired at full {7}{U} cost. Delve cost reduction is omitted. Test: `treasure_cruise_draws_three_at_full_cost`. |
| Aether Spellbomb | ✅ | {U}, sac: return target creature to hand. {1}, sac: draw a card. Both via `ActivatedAbility::sac_cost`. |
| The Everflowing Well | ⏳ | Saga land flip; needs Saga lore counters + DFC. |
| Proft's Eidetic Memory | ⏳ | Investigate + scaling +1/+1. Needs investigate (Clue). |
| Back to Basics | ⏳ | Static "nonbasic lands don't untap". Needs land-untap restriction. |
| Opposition | ⏳ | Tap creatures to tap permanents. |
| Parallax Tide | ⏳ | Fading + exile lands. Needs fade counters. |
| Omniscience | ⏳ | "Cast spells without paying". Needs free-cast static. |
| Shelldock Isle | ⏳ | Hideaway land (DFC-like setup). |
| Sink into Stupor | ⏳ | Counter + DFC into Lair land. |
| Concealing Curtains | ⏳ | DFC into Revealing Eye creature. |

### Black

| Card | Status | Notes |
|---|---|---|
| Moonshadow | ⏳ | Faerie / discard support. |
| Bitterbloom Bearer | ✅ | {1}{B} 1/2 Faerie Wizard with Flying. Self-source ETB creates a 1/1 black Faerie creature token with flying via `Effect::CreateToken` + a one-off `TokenDefinition`. Test: `bitterbloom_bearer_etb_creates_a_faerie_token`. |
| Bloodghast | ✅ | {B}{B} 2/1 Vampire Spirit. Wired via the new `EventScope::FromYourGraveyard`: a `LandPlayed` + `FromYourGraveyard` trigger fires off the graveyard copy and `Move`s `Selector::This` back to the battlefield. The "haste while opp ≤ 10 life" rider is omitted (no conditional-keyword static). Tests: `bloodghast_returns_from_graveyard_when_you_play_a_land`, `bloodghast_has_landfall_return_trigger`. |
| Golgari Thug | ⏳ | Dredge 4. Needs Dredge primitive. |
| Mai, Scornful Striker | ⏳ | New creature — abilities TBD. |
| Mutated Cultist | ⏳ | Mutate primitive needed. |
| Silversmote Ghoul | ✅ | {1}{B} 2/1 Zombie. `LifeGained` + `FromYourGraveyard` trigger returns Selector::This from graveyard to battlefield. Test: `silversmote_ghoul_returns_from_graveyard_on_lifegain`. |
| Ichorid | 🟡 | {B} 3/1 Horror with Haste. `StepBegins(Upkeep)` + `FromYourGraveyard` trigger returns Selector::This to the battlefield, then schedules a `NextEndStep` delayed exile (reuses Goryo's reanimate-then-exile pattern). The "opponent's graveyard contains a black creature" gate is omitted (no graveyard-color trigger filter yet). Test: `ichorid_returns_at_upkeep_then_exiles_at_end_step`. |
| Necrotic Ooze | ⏳ | Gains all activated abilities of creatures in graveyards. Big ability-borrow primitive. |
| Indulgent Tormentor | 🟡 | `{3}{B}{B}` 5/3 Demon Flying. End-step trigger drains 3 life from each opponent. The full Oracle gives the opponent a choice ("you draw a card or sacrifice a creature; if not, lose 3 life") — we resolve straight to the most punishing line (lose 3) since the multi-player choice primitive isn't wired. Test: `indulgent_tormentor_drains_each_opponent_at_end_step`. |
| Crabomination | ✅ | {2}{U}{B} 3/4 Crab Horror legendary. ETB mills 3 from each opponent; on opp-creature-death scry 1. Custom card. Test: `crabomination_etb_mills_each_opponent_three_cards`. |
| Doomsday Excruciator | ⏳ | Doomsday-adjacent. |
| Metamorphosis Fanatic | ⏳ | Unknown — TBD. |
| Slaughter Pact | ✅ | Destroy nonblack creature + delayed `PayOrLoseGame` upkeep ({2}{B}). |
| Deadly Dispute | ✅ | `{1}{B}` Sorcery. Sac-as-additional-cost folded into resolution as `SacrificeAndRemember(Creature ∨ Artifact, ControlledByYou)` followed by `Draw 2 + CreateToken(Treasure)`. Test: `deadly_dispute_sacrifices_and_creates_treasure_and_draws_two`. |
| Corpse Dance | ⏳ | Buyback + reanimate creature top of grave. |
| Baleful Mastery | ⏳ | Exile target nonland; opp may draw 2 to halve cost. Modal alt-cost. |
| Bloodchief's Thirst | 🟡 | `{B}` Sorcery. Base mode: destroy target creature/planeswalker with mana value 2 or less. The kicker `{1}{B}` mode (mana value ≤ 6) is still ⏳ — the alt-cost path can swap target filters but doesn't yet override the destroy filter on cast. Tests: `bloodchiefs_thirst_destroys_low_cmc_creature`, `bloodchiefs_thirst_rejects_high_cmc_target`. |
| Bone Shards | ✅ | {B} Instant. `ChooseMode([Sacrifice creature, Discard 1])` then destroy target creature. Cost-as-first-step approximation of "additional cost". |
| Disentomb | ✅ | Return target creature card to hand (Move from graveyard). |
| Hero's Downfall | ✅ | {1}{B}{B} Instant. Destroy target creature or planeswalker. Test: `heros_downfall_destroys_target_creature`. |
| Cast Down | ✅ | {1}{B} Instant. Destroy target nonlegendary creature — cast-time filter rejects legendary targets. Tests: `cast_down_destroys_nonlegendary_creature`, `cast_down_rejects_legendary_creature`. |
| Mind Rot | ✅ | {2}{B} Sorcery. Target player discards two cards (random — engine's chosen-discard primitive only handles caster-side picks). Test: `mind_rot_discards_two_from_target`. |
| Raise Dead | ✅ | {B} Sorcery. Return target creature card from your graveyard to your hand. Auto-target prefers caster's graveyard via `Effect::prefers_graveyard_source`. Test: `raise_dead_returns_creature_from_graveyard`. |
| Collective Brutality | ⏳ | Escalate-modal removal. |
| Drown in Ichor | ✅ | 3 damage to target creature + Surveil 1. |
| Fell | ✅ | Destroy target tapped creature + Surveil 2. |
| Night's Whisper | ✅ | Pay 2 life + Draw 2 (already in `decks::modern`). |
| Dread Return | 🟡 | {2}{B}{B} Sorcery. Wired as `Move(target creature card → Battlefield(You))`. The flashback "as an additional cost, sacrifice three creatures" half is omitted (no flashback-with-additional-cost primitive yet) — the regular cast is fully functional. Test: `dread_return_reanimates_target_creature_from_graveyard`. |
| Blasphemous Edict | ✅ | Each player sacrifices a creature (`Sacrifice` + `EachPlayer`). |
| Wishclaw Talisman | ⏳ | Wish-style tutor with downside. |
| Parallax Dementia | ⏳ | Fading + reanimate; needs fade counters. |
| Parallax Nexus | ⏳ | Fading + hand-strip. |
| Unholy Annex // Ritual Chamber | ⏳ | DFC enchantment land. |

### Red

| Card | Status | Notes |
|---|---|---|
| Blazing Rootwalla | ⏳ | Madness creature. Needs Madness. |
| Greasewrench Goblin | ✅ | {1}{R} 2/2 Haste. Treasure-on-death trigger via `EventKind::CreatureDied + SelfSource` + the shared `treasure_token()` helper (1-mana-of-any-color, sac on tap). The "can't block" Oracle rider collapses (no per-attacker block-restriction primitive yet). Tests: `greasewrench_goblin_enters_with_haste`, `greasewrench_goblin_creates_treasure_on_death`. |
| Grim Lavamancer | 🟡 | {R} 1/1 Human Wizard. {R}, {T}: deals 2 damage to any target. The "exile two cards from your graveyard" cost is approximated away — pending an exile-from-graveyard primitive. |
| Marauding Mako | ✅ | `{U}{B}` 2/2 Fish (Shark not in `CreatureType`). Triggered ability listens for `CardDiscarded`+`YourControl` and adds a +1/+1 counter to itself — captures the discard-payoff payoff faithfully. Available cross-pool when both U and B are picked. Test: `marauding_mako_grows_when_you_discard`. |
| Orcish Lumberjack | ✅ | {R} 1/1 Goblin Druid. `{T}, sacrifice a Forest: Add {G}{G}{G}` — Forest sac folded into resolution as Crop Rotation does. Test: `orcish_lumberjack_sacrifices_forest_for_three_green`. |
| Voldaren Epicure | ✅ | ETB: create a Blood token + 1 damage to each opponent (`ForEach EachOpponent`). |
| Amped Raptor | ⏳ | ETB cast spell from top. |
| Cam and Farrik, Havoc Duo | ⏳ | Dual creature. |
| Dreadhorde Arcanist | ⏳ | Attack-trigger flashback from grave. Reuses Flashback. |
| Magda, Brazen Outlaw | ⏳ | Treasure-on-tap + tutor. |
| Robber of the Rich | ⏳ | Cast-from-opp-library. Big primitive. |
| Anje's Ravager | ⏳ | Madness payoff. |
| Death-Greeter's Champion | ⏳ | Aggressive creature. |
| Detective's Phoenix | ⏳ | Recurring Phoenix variant. |
| Simian Spirit Guide | 🟡 | `{2}{R}` 2/2 Ape Spirit. Body wired; the alt-cost "exile from hand to add {R}" half is still ⏳ — the existing `AlternativeCost` path replaces the entire spell's resolution, so an alt-cost mana ability would need a new "alt cast = mana ability" mode. Available in any red pool. |
| Arclight Phoenix | ⏳ | Three-spell-cast trigger from graveyard. Needs spells-cast-this-turn count + recursion. |
| Goldspan Dragon | 🟡 | 4/4 Flying Haste; attack-trigger Treasure (using the now-functional Treasure mana ability). "Becomes target of a spell" trigger and the Treasure-2-mana static rider are omitted. |
| Shivan Dragon | ✅ | Already in catalog. |
| Balefire Dragon | 🟡 | {5}{R}{R} 6/6 Flying. `DealsCombatDamageToPlayer + SelfSource` trigger sweeps each opp creature for 6. The "that much damage" → fixed 6 collapse holds at unboosted-power play; pump-effect interactions don't retroactively boost the trigger payload. Test: `balefire_dragon_combat_damage_burns_each_opp_creature`. |
| Pact of the Titan | ✅ | 4/4 red Giant token + delayed `PayOrLoseGame` upkeep ({4}{R}). |
| Tarfire | 🟡 | 2 damage to any target. Tribal type omitted (engine has no Tribal card type). |
| Chaos Warp | 🟡 | {2}{R} Instant. `Move(target Permanent → Library(OwnerOf, Shuffled))` — the new `LibraryPosition::Shuffled` engine path actually reshuffles the library. The "reveal top, cast if permanent" half is collapsed (info-only without a cast-from-top pipeline). Test: `chaos_warp_sends_target_permanent_to_owners_library`. |
| Big Score | ✅ | Discard + 2 Treasure + Draw 2. Treasure tokens are now fully functional — each carries its `{T}, Sac: Add one mana of any color` activated ability via `TokenDefinition::activated_abilities`. |
| Mine Collapse | ✅ | {2}{R} sorcery. Sacrifice a Mountain on resolution (cost-as-first-step), deal 4 damage to any target. Test: `mine_collapse_sacrifices_mountain_and_deals_four`. |
| Fireblast | 🟡 | {4}{R}{R} instant. 4 damage to any target — regular cost wired. Sacrifice-two-Mountains alt cost still ⏳ (no sacrifice-as-alt-cost on `AlternativeCost` today). Test: `fireblast_deals_four_to_any_target`. |
| Tormenting Voice | ✅ | {1}{R} Sorcery. Discard 1, draw 2. Cost-as-first-step (additional-cost discard folded into resolution). Test: `tormenting_voice_discards_one_and_draws_two`. |
| Wild Guess | ✅ | {2}{R} Sorcery. Same loot pattern as Tormenting Voice at +1 mana. Test: `wild_guess_discards_one_and_draws_two`. |
| Thrill of Possibility | ✅ | {1}{R} Instant. Instant-speed Tormenting Voice. Test: `thrill_of_possibility_is_an_instant_loot_2`. |
| Volcanic Hammer | ✅ | {1}{R} Sorcery. 3 damage to any target. Test: `volcanic_hammer_deals_three_to_creature`. |
| Slagstorm | ✅ | {2}{R} Sorcery. `ChooseMode([ForEach creature → 3 dmg, ForEach player → 3 dmg])`. AutoDecider picks mode 0. Tests: `slagstorm_mode_zero_sweeps_creatures`, `slagstorm_mode_one_burns_each_player`. |
| Stoke the Flames | ✅ | {4}{R} Instant with `Keyword::Convoke`. 4 damage to any target. Test: `stoke_the_flames_deals_four_at_full_cost`. |
| Pyrokinesis | 🟡 | {4}{R}{R} Instant. Pitch-cost alt cast: exile a red card from hand → 4 damage. The "divide 4 damage among any number of creatures" half is approximated as a single 4-damage hit. |
| Vandalblast | 🟡 | Single-target artifact destruction; Overload {4}{R} mode omitted (no overload primitive yet). |
| Legion Extruder | ⏳ | Equip-ish artifact. |
| Sundering Eruption | ✅ | MDFC: front is `{1}{R}` sorcery dealing 3 damage to a creature/planeswalker; back face Mount Tyrhus is a Mountain that ETBs tapped and taps for {R}. |

### Green

| Card | Status | Notes |
|---|---|---|
| Basking Rootwalla | ⏳ | Madness creature. |
| Elvish Reclaimer | 🟡 | {1}{G} 1/2 Human Druid. `{T}, sac a land: Search(Land → BF)`. Sac-as-cost folded into resolution. Threshold-pump rider (3/4 with 7+ in graveyard) is omitted. Test: `elvish_reclaimer_sacrifices_land_to_search_for_one`. |
| Haywire Mite | ✅ | {2}, sac: destroy artifact/enchantment/planeswalker + gain 1 life (`ActivatedAbility::sac_cost`). |
| Sylvan Safekeeper | ✅ | {G} 1/1 Human Wizard. Sacrifice a Forest: target creature gains shroud EOT. Sac-of-other-land cost folded into resolution. |
| Basking Broodscale | ⏳ | Eldrazi token-maker. |
| Cankerbloom | ✅ | {1}{G} 2/2 Fungus. {G}, Sac this: destroy target artifact/enchantment, then proliferate. |
| Reclamation Sage | ✅ | {2}{G} 2/1 Elf Shaman. ETB destroy target artifact/enchantment (same shape as Loran of the Third Path's ETB). Test: `reclamation_sage_etb_destroys_artifact`. |
| Acidic Slime | ✅ | {3}{G}{G} 2/2 Ooze with Deathtouch. ETB destroy target artifact, enchantment, or land. Test: `acidic_slime_etb_destroys_land`. |
| Collector Ouphe | ⏳ | Static "artifact abilities can't be activated". |
| Fanatic of Rhonas | ✅ | {G} 1/1 Snake. `{G},{T}: Add {G}{G}` (net +{G} per activation). Test: `fanatic_of_rhonas_taps_for_two_green_after_paying_one`. |
| Keen-Eyed Curator | ⏳ | Graveyard hate + counter pump. |
| Rofellos, Llanowar Emissary | 🟡 | {G}{G} Legendary 0/1 Elf Druid. `{T}: Add {G}{G}` (Fanatic-of-Rhonas pattern). Forest-count multiplier collapses to flat 2 — net +2 mana per activation rather than +N where N = Forests you control. Test: `rofellos_taps_for_two_green_mana`. |
| Satyr Wayfinder | 🟡 | {1}{G} 1/1 Satyr Druid. ETB mills 4 (`Effect::Mill 4`). The "may take a land from among them" half is collapsed to the graveyard-fill outcome — gameplay-relevant for reanimator/dredge shells. Test: `satyr_wayfinder_etb_mills_four`. |
| Sylvan Caryatid | ✅ | 0/3 Hexproof Defender; {T}: Add one mana of any color. |
| Elvish Spirit Guide | 🟡 | {2}{G} 2/2 Elf Spirit body wired. The "exile from hand: add {G}" alt-mana ability needs a hand-activated-ability primitive (`activate_ability` only walks the battlefield today); promote to ✅ once that lands. Test: `elvish_spirit_guide_is_a_two_two_elf_spirit`. |
| Enduring Vitality | ⏳ | Roomba-style return on death + creature mana untap. |
| Hauntwoods Shrieker | ⏳ | Token + transform. |
| Mossborn Hydra | ⏳ | Hydra +1/+1 counter scaling. |
| Mutable Explorer | ⏳ | Mutate primitive. |
| Sentinel of the Nameless City | 🟡 | Vigilance + attack-trigger 1/1 green Citizen token. Ward 2 omitted (keyword exists but not enforced at targeting time); Plant subtype dropped (no `Plant` in `CreatureType`). |
| Tireless Tracker | 🟡 | Filtered ETB-other trigger: when a land enters under your control, investigate (create a Clue). Sac-Clue +1/+1 ability omitted (no sac-of-other-permanent activation primitive). |
| Ursine Monstrosity | ⏳ | Adapt-style P/T scaling. |
| Baloth Prime | ⏳ | TBD. |
| Icetill Explorer | ⏳ | TBD. |
| Mightform Harmonizer | ⏳ | TBD. |
| Ouroboroid | ⏳ | TBD. |
| Sowing Mycospawn | ⏳ | Eldrazi land-search. |
| Vengevine | ⏳ | Recurring on two-creatures-cast-this-turn. |
| Elder Gargaroth | ⏳ | Trigger-on-attack/block creature. |
| Golgari Grave-Troll | ⏳ | Dredge 6. |
| Railway Brawler | ⏳ | Train (vehicle-like). |
| Conclave Sledge-Captain | ⏳ | TBD. |
| Lumra, Bellow of the Woods | ✅ | {4}{G}{G} 6/6 Trample Legendary Elemental. ETB returns every land card in your graveyard via `Move(EachMatching(Graveyard(You), Land) → Battlefield(You, tapped))`. Tests: `lumra_returns_all_lands_from_your_graveyard`, `lumra_etb_with_empty_graveyard_is_a_noop`. |
| Zopandrel, Hunger Dominus | ⏳ | TBD. |
| Apex Devastator | ⏳ | Cascade x4 (cascade primitive). |
| Summoner's Pact | ✅ | Already wired (search green creature + delayed Pact upkeep). |
| Eternal Witness | ✅ | `{1}{G}{G}` 2/1 Human Shaman with ETB "return target card from your graveyard to your hand". Auto-target now picks the graveyard card via the new `Effect::prefers_graveyard_source` classification (`Move(target → Hand(You))` is reanimate-class). Tests: `eternal_witness_etb_trigger_present`, `eternal_witness_etb_returns_graveyard_card_via_auto_target`. |
| Nature's Claim | ⏳ | Destroy artifact/enchantment + 4 life. |
| Archdruid's Charm | ⏳ | Modal — destroy land/artifact, search creature, +1/+1 counters. |
| Finale of Devastation | ⏳ | Tutor + pump scaling with X. |
| Life from the Loam | ⏳ | Return up-to-3 lands; Dredge 3. |
| Nature's Lore | ✅ | Search Forest, put onto battlefield untapped. |
| Kodama's Reach | ⏳ | Two-basic ramp. |
| Biorhythm | 🟡 | {4}{G}{G}{G} Sorcery. `LoseLife(EachOpponent, 20) + GainLife(You, count(your creatures))`. Set-life-total-to-X primitive doesn't exist; we drop each opp by a chunk that beats their starting life total instead. Test: `biorhythm_drops_each_opponent_to_zero_or_below`. |
| Esika's Chariot | ⏳ | Vehicle + crew + token. |
| Springleaf Parade | ⏳ | TBD. |
| Up the Beanstalk | ✅ | ETB Draw 1 + filtered SpellCast trigger (mana value ≥ 5 → Draw 1). |
| Aluren | ⏳ | Free-cast 3 or less creatures. |
| Greater Good | ⏳ | Sac creature + Draw P. |
| Shifting Woodland | ⏳ | DFC land. |

### Artifacts & Planeswalkers (mono / colorless)

| Card | Status | Notes |
|---|---|---|
| Ornithopter | ✅ | {0} Artifact creature 0/2 with Flying. |
| Ornithopter of Paradise | ✅ | {1} Artifact creature 0/2 Flying; {T}: Add one mana of any color. |
| Glaring Fleshraker | ⏳ | TBD. |
| Tezzeret, Cruel Captain | 🟡 | {3}{B} 4-loyalty walker. **+1**: target creature gets -2/-2 EOT. **-2**: drain 2 life from each opponent (you gain 2). Ult is collapsed; the "your artifact creatures get +1/+1" static is dropped. Tests: `tezzeret_minus_two_drains_each_opponent_for_two`, `tezzeret_plus_one_shrinks_target_creature`. |
| Karn, Scion of Urza | 🟡 | {4} 5-loyalty Karn. **+1**: Draw 1 + mill 1 (the opp-pile-split is information-only at this engine fidelity). **-1**: ForEach Construct creature you control + AddCounter(+1/+1). **-2**: Create a 1/1 Construct artifact creature token (artifact-count scaling rider collapses). Tests: `karn_scion_of_urza_minus_two_creates_a_construct_token`, `karn_plus_one_draws_a_card_and_mills_one`. |
| Kozilek's Command | ⏳ | Modal Eldrazi instant. |
| Eldrazi Confluence | ⏳ | Modal x3. |
| Chalice of the Void | ⏳ | X charge counters; counter spells with mana value X. |
| Zuran Orb | ✅ | {0} Artifact. Sacrifice a land: gain 2 life. Cost-as-first-step folded into resolution. |
| Candelabra of Tawnos | ⏳ | Untap N lands for {X}. |
| Chromatic Star | ✅ | {1} Artifact. {1}, T, Sac: add one mana of any color. Cantrips on `PermanentLeavesBattlefield` (engine extension — see notes). |
| Ghost Vacuum | ✅ | {2} Artifact. `{2}, {T}: Move(target → Exile)` over `Any`. Auto-target prefers a graveyard card via the new `Effect::prefers_graveyard_target` (Move-to-Exile) heuristic — without it, the bot would auto-target a battlefield permanent. The "draw on every card going to your graveyard" rider on later printings is omitted (not on the original). Tests: `ghost_vacuum_exiles_target_card_from_graveyard`, `ghost_vacuum_auto_target_picks_graveyard_card_when_present`. |
| Lavaspur Boots | ⏳ | Equipment grants haste. |
| Pithing Needle | ⏳ | Name-a-card; activated abilities can't be activated. Needs name-a-card primitive (Cavern shares). |
| Shuko | ⏳ | Equipment with free-equip. |
| Soul-Guide Lantern | ✅ | {1} Artifact. {T}: exile a card from each opponent's graveyard (approximation of "target opponent" — equivalent in 2-player). {2}, T, Sac: each player exiles their graveyard, draw 1. |
| Agatha's Soul Cauldron | ⏳ | Borrow activated abilities of exiled creatures. |
| Fellwar Stone | 🟡 | {T}: Add one mana of any color. (Approximation: drops the "matches opponent's lands" restriction — engine has no per-source mana provenance yet.) |
| Mesmeric Orb | ⏳ | Mill-on-untap symmetric. |
| Millstone | ✅ | {2}, {T}: target player mills 2. |
| Mind Stone | ✅ | {T}: Add {C}. {1}, {T}, Sacrifice this: Draw a card. Both abilities wired (uses `ActivatedAbility::sac_cost`). |
| Pentad Prism | 🟡 | {2} Artifact. ETB with 2 charge counters; remove a charge counter to add one mana of any color. Sunburst's "one counter per color of mana spent" collapses to a flat 2. Tests: `pentad_prism_etb_with_two_charge_counters`, `pentad_prism_removes_counter_to_add_one_mana_of_any_color`. |
| Smuggler's Copter | ⏳ | Vehicle + crew + loot trigger. Needs Vehicle primitive. |
| Coalition Relic | 🟡 | {3} Artifact. `{T}: Add one mana of any color`. The charge-counter rider ("{T}: put a charge counter; remove three to add WUBRG") is omitted — no charge-to-mana-burst primitive yet. Tap-for-any-color half is fully functional. Test: `coalition_relic_taps_for_one_mana_of_any_color`. |
| Monument to Endurance | ⏳ | Graveyard-recursion artifact. |
| Nettlecyst | ⏳ | Living-equipment + token. |
| Sword of Body and Mind | ⏳ | Equipment + protection + token + mill. |
| Trinisphere | ⏳ | Static "spells cost at least {3}". Reuses cost-tax static. |
| Helm of the Host | ⏳ | Equipment that token-copies on attack. |
| The Mightstone and Weakstone | ⏳ | Modal artifact (assemble). |
| Coveted Jewel | ⏳ | Mana + force-attack control mechanic. |
| The Endstone | ⏳ | TBD. |
| Portal to Phyrexia | ⏳ | Big sac-3 + reanimate. |
| Talisman of Progress | ✅ | {2} Artifact. Three activated abilities: `{T}: Add {C}` (index 0); `{T}: Add {W}` and `{T}: Add {U}` (indices 1 + 2), each costing 1 life. Color choice exposed as separate ability indices for explicit picker. Test: `talisman_of_progress_taps_for_colorless_or_one_of_w_or_u`. |
| Talisman of Conviction | ✅ | RW mirror of Talisman of Progress (`{T}: Add {C}` + `{T}: Add {R}` + `{T}: Add {W}`, each colored ability costs 1 life). Built on the shared `talisman_cycle` helper. Test: `talisman_of_conviction_taps_for_red_costing_one_life`. |
| Talisman of Creativity | ✅ | UR mirror of Talisman of Progress. Test: `talisman_of_creativity_taps_for_blue_or_red_costing_one_life`. |
| Talisman of Curiosity | ✅ | GU mirror of Talisman of Progress. Test: `talisman_of_curiosity_taps_for_green_costing_one_life`. |

### Multicolor

| Card | Status | Notes |
|---|---|---|
| Shorikai, Genesis Engine | ⏳ | Vehicle-walker hybrid; loots on activate. |
| Cruel Somnophage | ✅ | {1}{U}{B} 0/0 Phyrexian Horror; layer-7 `SetPowerToughness(your_grave_size, your_grave_size)` injection in `compute_battlefield` (same hardcoded site that powers Tarmogoyf/Cosmogoyf). Test: `cruel_somnophage_pt_scales_with_your_graveyard`. |
| Talisman of Dominance | ✅ | {2} Artifact. UB mirror of Talisman of Progress: `{T}: Add {C}` + `{T}: Add {U}` + `{T}: Add {B}` (each colored ability costs 1 life). Test: `talisman_of_dominance_taps_for_blue_costing_one_life`. |
| Ashiok, Nightmare Weaver | ⏳ | Planeswalker — exile/scry/copy. |
| Master of Death | ⏳ | UB recursion + discard. |
| Fallen Shinobi | ⏳ | Ninjitsu + reveal-and-take. |
| Bloodtithe Harvester | 🟡 | ETB and attack triggers each create a Blood token. Sac-Blood ping ability omitted (no sac-of-other-permanent activation primitive). |
| Terminate | ✅ | Already in catalog (destroy can't-regenerate). |
| Carnage Interpreter | ⏳ | TBD. |
| Kolaghan's Command | ⏳ | Modal x2. |
| Master of Cruelties | ⏳ | First-strike + life-to-1 attack trigger. |
| Territorial Kavu | ⏳ | Color-matters. |
| Bloodbraid Challenger | ⏳ | Cascade. |
| Qasali Pridemage | ⏳ | Exalted + sac to destroy artifact/enchantment. |
| Knight of the Reliquary | ⏳ | Sac-land tutor scaling P/T. |
| Brightglass Gearhulk | ⏳ | TBD. |
| Growing Ranks | ⏳ | Populate token-copy on upkeep. |
| Torsten, Founder of Benalia | ⏳ | TBD. |
| Tidehollow Sculler | 🟡 | {W}{B} 2/2 Zombie. ETB picks a nonland card from a target opponent's hand and sends it to their graveyard (approximation of "exile until this leaves"). The "return when this leaves" clause is omitted (no exile-until-LTB primitive yet). Reuses `DiscardChosen`. Test: `tidehollow_sculler_etb_takes_an_opponent_card`. |
| Gift of Orzhova | ⏳ | Aura — flying + lifelink. |
| Stillmoon Cavalier | ⏳ | Mana abilities for protection toggling. |
| Sorin, Grim Nemesis | ⏳ | Planeswalker. |
| Expressive Iteration | ⏳ | UR look-at-top-3 multi-pick. |
| Talisman of Creativity | ✅ | UR mana rock — see Artifacts row. |
| Pinnacle Emissary | ⏳ | TBD. |
| Saheeli Rai | ⏳ | UR planeswalker. |
| Tempest Angler | ⏳ | TBD. |
| Abrupt Decay | ⏳ | BG removal: destroy nonland with mana value ≤ 3, can't be countered. |
| Assassin's Trophy | ⏳ | BG removal — opp searches for basic. |
| Broodspinner | ⏳ | TBD. |
| Tear Asunder | ⏳ | BG kicker. |
| Wight of the Reliquary | ⏳ | Land-tutor sac variant. |
| The Gitrog Monster | ⏳ | Land-as-discard / dredge engine. |
| Talisman of Conviction | ✅ | RW mana rock — see Artifacts row. |
| Wear // Tear | ⏳ | Split-card; needs Split-card primitive. |
| Zirda, the Dawnwaker | ⏳ | Companion + activated-cost reduction. Needs Companion primitive. |
| Talisman of Curiosity | ✅ | GU mana rock — see Artifacts row. |
| Lonis, Genetics Expert | ⏳ | Investigate + Clue draw. |
| Tamiyo, Collector of Tales | ⏳ | Planeswalker. |
| Sab-Sunen, Luxa Embodied | ⏳ | TBD. |
| Koma, Cosmos Serpent | ⏳ | Token-on-upkeep + sac counters. |
| Kestia, the Cultivator | ⏳ | Aura/enchantment matters. |
| Messenger Falcons | ⏳ | TBD. |
| Dakkon, Shadow Slayer | ⏳ | TBD. |
| Urza, Chief Artificer | ⏳ | Planeswalker / commander. |
| Geyadrone Dihada | ⏳ | Planeswalker. |
| Lord Xander, the Collector | ⏳ | ETB / attack / death triggers. |
| Korvold, Fae-Cursed King | ⏳ | Sac-trigger draw + +1/+1. |
| Temur Ascendancy | 🟡 | Filtered ETB trigger (creatures w/ power ≥ 4 entering under your control → Draw 1). Static "creatures you control have haste" currently grants haste to all your creatures rather than only ≥ 4 power (selector-decomposer doesn't yet thread `PowerAtLeast` into static-effect targeting). |
| Loot, the Pathfinder | ⏳ | TBD. |
| Dragonback Assault | ⏳ | TBD. |
| Rediscover the Way | ⏳ | TBD. |
| Shiko and Narset, Unified | ⏳ | TBD. |
| Awaken the Honored Dead | ⏳ | Reanimate-multiple. |
| Fangkeeper's Familiar | ⏳ | TBD. |
| Teval, Arbiter of Virtue | ⏳ | TBD. |
| Muldrotha, the Gravetide | ⏳ | Cast from graveyard each turn (one of each card type). |
| Rakshasa's Bargain | 🟡 | Pay 4 life + Draw 4. The "exile creature card from your graveyard" alternate additional cost is folded away (modal additional-cost not modeled). |
| Omnath, Locus of Creation | ⏳ | Landfall-quad-color. |
| Atraxa, Grand Unifier | 🟡 | Already wired with ETB Draw 4 approximation. Real reveal-and-sort still ⏳. |
| Leyline of the Guildpact | ⏳ | "Your permanents are all colors." Needs is-all-colors primitive. |
| Golos, Tireless Pilgrim | ⏳ | ETB land-search; activated cast-from-top. |
| Maelstrom Archangel | ⏳ | Combat-damage cast-free trigger. |
| Maelstrom Nexus | ⏳ | Cascade-on-first-spell static. |
| Ramos, Dragon Engine | ⏳ | Charge counters per-color spells; convert to mana. |

### Lands

| Card | Status | Notes |
|---|---|---|
| Celestial Colonnade | ⏳ | UW manland. |
| Meticulous Archive | ✅ | UW surveil land (catalog). |
| Razortide Bridge | 🟡 | UW Bridge artifact-land. ETB tapped, typed as Plains + Island, `{T}: Add {C}`. Each bridge tracked in CUBE_FEATURES.md is its own factory; the helper `bridge_land()` reduces duplication. The "every basic land type" half is collapsed to the two relevant types (enough for fetchland + Nature's-Lore-style searches). Tests: `mistvault_bridge_etbs_tapped_with_dual_basic_typing`, `drossforge_bridge_taps_for_colorless`, `all_bridges_etb_tapped_and_carry_two_basic_land_types`. |
| Seachrome Coast | ✅ | UW fastland (reuses fastland trigger). |
| Creeping Tar Pit | ⏳ | UB manland. |
| Darkslick Shores | ✅ | UB fastland. |
| Mistvault Bridge | 🟡 | UB Bridge — see Razortide Bridge row. |
| Undercity Sewers | ✅ | UB surveil land (catalog). |
| Blackcleave Cliffs | ✅ | BR fastland (catalog). |
| Blazemire Verge | ⏳ | BR DFC verge. |
| Drossforge Bridge | 🟡 | BR Bridge — see Razortide Bridge row. |
| Raucous Theater | ✅ | BR surveil land. ETB tapped + surveil 1; reuses `dual_land_with` + `etb_tap_then_surveil_one` (modern_decks-11). |
| Commercial District | ✅ | RW surveil land. ETB tapped + surveil 1 (modern_decks-11). |
| Copperline Gorge | ✅ | RG fastland (catalog). |
| Slagwoods Bridge | 🟡 | RG Bridge — see Razortide Bridge row. |
| Thornspire Verge | ⏳ | RG verge. |
| Horizon Canopy | ⏳ | GW horizon canopy: pay 1 + life to draw. |
| Lush Portico | ✅ | GW surveil land. ETB tapped + surveil 1 (modern_decks-11). |
| Razorverge Thicket | ✅ | GW fastland. |
| Thornglint Bridge | 🟡 | GW Bridge — see Razortide Bridge row. |
| Bleachbone Verge | ⏳ | WB verge. |
| Concealed Courtyard | ✅ | WB fastland. |
| Goldmire Bridge | 🟡 | WB Bridge — see Razortide Bridge row. |
| Shadowy Backstreet | ✅ | WB surveil land (catalog). |
| Riverpyre Verge | ⏳ | UR verge. |
| Silverbluff Bridge | 🟡 | UR Bridge — see Razortide Bridge row. |
| Spirebluff Canal | ✅ | UR fastland. |
| Thundering Falls | ✅ | UR surveil land. ETB tapped + surveil 1 (modern_decks-11). |
| Blooming Marsh | ✅ | BG fastland (catalog). |
| Darkmoss Bridge | 🟡 | BG Bridge — see Razortide Bridge row. |
| Underground Mortuary | ✅ | BG surveil land. ETB tapped + surveil 1 (modern_decks-11). |
| Wastewood Verge | ⏳ | BG verge. |
| Elegant Parlor | ✅ | RG surveil land (Murders at Karlov Manor — fixing the spec's RW typo). ETB tapped + surveil 1 (modern_decks-11). |
| Inspiring Vantage | ✅ | RW fastland. |
| Rustvale Bridge | 🟡 | RW Bridge — see Razortide Bridge row. |
| Sunbaked Canyon | ⏳ | RW horizon canopy. |
| Botanical Sanctum | ✅ | UG fastland. |
| Hedge Maze | ✅ | UG surveil land. ETB tapped + surveil 1 (modern_decks-11). |
| Tanglepool Bridge | 🟡 | UG Bridge — see Razortide Bridge row. |
| Waterlogged Grove | ⏳ | UG horizon canopy. |
| Twisted Landscape | ⏳ | Tri-color landcycle. |
| Sheltering Landscape | ⏳ | Tri-color landcycle. |
| Bountiful Landscape | ⏳ | Tri-color landcycle. |
| Ancient Den | ✅ | Artifact land — Plains. {T}: Add {W}. |
| Cloudpost | 🟡 | Locus land. ETB tapped, `{T}: Add {C}`. The per-Locus mana scaling collapses to a flat colorless source (no per-source mana scaling primitive yet). Test: `cloudpost_etbs_tapped_and_taps_for_colorless`. |
| Darksteel Citadel | ✅ | Indestructible artifact land. {T}: Add {C}. |
| Evolving Wilds | ✅ | ETB tapped, `{T}, sac: Search basic land → BF tapped`. Same `sac_cost: true` shape as Wasteland's destruction half. Test: `evolving_wilds_sacrifices_to_search_basic`. |
| Exotic Orchard | ⏳ | Mana matching opponents' lands. |
| Glimmerpost | 🟡 | Locus land. ETB tapped + gain 1 life, `{T}: Add {C}`. The "1 life for each Locus you control" scaling collapses to a flat 1 (gameplay-relevant on a normal board, where Locus density is at most 2). Tests: `glimmerpost_etbs_tapped_and_grants_one_life`, `glimmerpost_taps_for_colorless_after_untap`. |
| Great Furnace | ✅ | Artifact-Mountain. {T}: Add {R}. |
| Lotus Field | 🟡 | ETB tapped + Sacrifice 2 lands. `{T}: Add 3 mana of one color`. The "untapped" qualifier on the ETB sac is collapsed (Sacrifice filter doesn't expose tapped state). Tests: `lotus_field_etb_sacrifices_two_lands`, `lotus_field_taps_for_three_of_one_color`. |
| Planar Nexus | ⏳ | Tri-color rainbow. |
| Power Depot | ⏳ | Charge-counter mana storage. |
| Rishadan Port | ⏳ | Tap-to-tap-opp-land. |
| Seat of the Synod | ✅ | Artifact-Island. {T}: Add {U}. |
| Talon Gates of Madara | ⏳ | TBD. |
| Three Tree City | ⏳ | TBD. |
| Tree of Tales | ✅ | Artifact-Forest. {T}: Add {G}. |
| Trenchpost | ⏳ | TBD. |
| Vault of Whispers | ✅ | Artifact-Swamp. {T}: Add {B}. |

## Implementation log (most recent first)

- **modern_decks-15: 12 cube cards + edict-class auto-target +
  ability_cost_label sacrifice rider**:
  - **12 new cards** in `catalog::sets::decks::modern`:
    - **Burn**: `strangle` ({R} 3 dmg + Surveil 1).
    - **Removal**: `dreadbore` ({B}{R} sorcery destroy
      creature/planeswalker), `bedevil` ({B}{B}{R} instant destroy
      artifact/creature/planeswalker).
    - **Mill**: `tome_scour` ({U} sorcery target player mills 5).
    - **Bounce / cantrip**: `repulse` ({2}{U}), `visions_of_beyond`
      ({U} cantrip).
    - **Removal**: `plummet` ({1}{G} destroy flying creature).
    - **Graveyard fill**: `strategic_planning` ({1}{U} mill 3 + draw 1).
    - **Discard creatures**: `ravenous_rats` ({1}{B} 1/1 ETB
      discard), `brain_maggot` ({1}{B} 1/1 ETB strip nonland).
    - **Tempo/lifegain**: `bond_of_discipline` ({3}{W} tap each opp
      creature + lifelink).
    - **Edict**: `sudden_edict` ({1}{B} target player sacs;
      uncounterable).
  - **Engine: surface Sacrifice's player filter in
    `primary_target_filter`** — see DECK_FEATURES.md for the
    full rationale.
  - **UI/view: `ability_cost_label` advertises sacrifice rider** —
    sacrifice-cost activated abilities now render "Sac" in the
    cost label so the UI doesn't undercount the activation cost.
  - **Cube wiring**: white +1, blue +4, black +3, red +1, green +1,
    BR cross +2.
  - 17 new tests; lib suite 627 → 644 (+17).

- **modern_decks-13: 12 cube cards + bot loyalty + library shuffle + view loyalty**:
  - **12 new / promoted cards** in `catalog::sets::decks::modern`:
    - **Reanimator finisher**: `lumra_bellow_of_the_woods` — 6/6 Trample
      legendary; ETB returns all lands from your graveyard.
    - **UB midrange**: `crabomination` — custom card; mill-on-ETB,
      scry on opp creature death; `cruel_somnophage` — 0/0 with P/T
      = your graveyard size (Tarmogoyf-pattern compute-time injection).
    - **Removal / chaos**: `chaos_warp` — shuffle target permanent into
      its owner's library (the new `LibraryPosition::Shuffled` engine
      path actually reshuffles).
    - **Land tutor**: `elvish_reclaimer` — sac a land for any land.
    - **Mana**: `rofellos_llanowar_emissary` — `{T}: Add {G}{G}` (Forest
      multiplier collapsed); `pentad_prism` — charge-counter mana rock
      (Sunburst collapsed to flat 2).
    - **Big swing**: `biorhythm` — drop each opp by 20 life; `balefire_dragon`
      — 6/6 Flying with combat-damage opp-creature sweep.
    - **Planeswalkers**: `karn_scion_of_urza` (Construct theme),
      `tezzeret_cruel_captain` (drain + small removal).
    - **Promotion**: `greasewrench_goblin` 🟡 → ✅ (Treasure-on-death
      trigger now wired via the existing `treasure_token()` helper).
  - **Engine: bot activates planeswalker loyalty abilities** —
    `RandomBot::main_phase_action` walked planeswalkers but never picked
    a loyalty ability. New `pick_loyalty_ability` helper sorts by
    descending loyalty cost (preserves +1 over -2), filters out
    abilities the walker can't afford, auto-targets, and dry-runs each
    candidate against `would_accept`.
  - **Engine: `LibraryPosition::Shuffled` actually shuffles**. Pre-fix
    fell through to `push` (effectively bottom). Now we push then
    `library.shuffle(&mut rng)`.
  - **UI/server: planeswalker loyalty abilities surface in `PermanentView`**.
    New `LoyaltyAbilityView` wire type + `view::project_loyalty_abilities`
    populator. Pre-fix the wire view only carried activated abilities.
  - **Engine: `PlaneswalkerSubtype::Tezzeret`** added.
  - **Cube wiring**: green pool +4 (Elvish Reclaimer, Rofellos, Biorhythm,
    Lumra), red pool +2 (Chaos Warp, Balefire Dragon), black pool +1
    (Tezzeret), colorless pool +2 (Karn Scion of Urza, Pentad Prism),
    UB cross +2 (Crabomination, Cruel Somnophage). 11 new names listed
    in the cube prefetch test for early failure on art lookup.
  - 17 new tests; lib suite 587 → 604 (+17).

- **modern_decks-11: 14 new cube cards (surveil lands + multicolor removal + sweepers + body + mana engine)**:
  - **7 new surveil lands** finishing the Murders at Karlov Manor
    cycle: `underground_mortuary` (BG), `lush_portico` (GW),
    `hedge_maze` (UG), `thundering_falls` (UR),
    `commercial_district` (RW), `raucous_theater` (BR), and
    `elegant_parlor` (RG — corrected from CUBE_FEATURES's RW typo;
    the printed card is Gruul). All seven reuse the existing
    `dual_land_with` + `etb_tap_then_surveil_one` helpers from
    `sets/mod.rs`.
  - **7 new spell/creature cards**: `tear_asunder` (BG artifact/
    enchantment removal — kicker collapsed), `assassins_trophy`
    (BG opp permanent destruction — basic-search downside
    collapsed), `volcanic_fallout` (RR sweeper + 2-each-player),
    `rout` (WW DoJ-mirror), `plague_wind` (BB one-sided sweeper),
    `carnage_tyrant` (GG 7/6 with Trample/Hexproof/CantBeCountered),
    `krark_clan_ironworks` (sac-an-artifact for {2}, cost folded
    into resolution).
  - **Cube wiring**: each card placed in its primary color pool
    (white +1: Rout; red +1: Volcanic Fallout; black +1: Plague
    Wind; green +1: Carnage Tyrant; BG cross-pool +2: Tear Asunder
    + Assassin's Trophy in both Black and Green branches);
    KCI in `colorless_pool`. The 14 new card names are explicitly
    listed in the cube prefetch test so the Scryfall art is loaded
    on startup.
  - **Cleanup**: `view::ability_effect_label` wildcard arm gets a
    documenting comment.
  - 15 new tests in `tests/modern.rs` (12 card-functionality + 3
    surveil-land coverage); lib suite 545 → 559+.

- **modern_decks-10: 16 cube cards (Locus / bridges / utility artifacts) + graveyard-target auto-target**:
  - **16 new cards** in `catalog::sets::decks::modern`, all built on
    existing engine primitives (no engine changes required for the
    cards themselves):
    - **Locus + utility lands** (4): `glimmerpost` (Locus, ETB tap +
      1 life + `{T}: Add {C}`), `cloudpost` (Locus, ETB tap +
      `{T}: Add {C}`), `lotus_field` (ETB tap + sac 2 lands + `{T}:
      Add 3 of one color`), `evolving_wilds` (ETB tap + `{T}, sac:
      search basic`).
    - **Bridge cycle** (10): `mistvault_bridge` (UB), `drossforge_bridge`
      (BR), `razortide_bridge` (UW), `goldmire_bridge` (WB),
      `silverbluff_bridge` (UR), `tanglepool_bridge` (UG),
      `slagwoods_bridge` (RG), `thornglint_bridge` (GW),
      `darkmoss_bridge` (BG), `rustvale_bridge` (RW). All ETB tapped,
      typed by the two basic land types most relevant to the color
      pair, `{T}: Add {C}`. Helper `bridge_land()` collapses the
      duplication.
    - **Utility artifacts** (2): `coalition_relic` (`{T}: Add one
      mana of any color`; charge-counter→WUBRG burst rider omitted),
      `ghost_vacuum` (`{2}, {T}: exile target card from a graveyard`).
  - **Engine: graveyard-target auto-target preference**: renamed
    `prefers_graveyard_source` → `prefers_graveyard_target` and
    extended it to also match `Move target → Exile`. The
    `auto_target_for_effect` heuristic now walks graveyards (in
    primary→secondary order) before the battlefield when the flag
    is set, so Ghost Vacuum's `Any`-filtered Move-to-Exile picks an
    opp's graveyard card rather than a battlefield permanent. The
    walk respects `prefers_friendly_target` ordering — Disentomb /
    Reanimate (friendly) hit the caster's graveyard first; Ghost
    Vacuum (hostile) hits the opp's. Tests:
    `ghost_vacuum_auto_target_picks_graveyard_card_when_present`
    (new), all five existing auto-target tests still pass.
  - **Cube wiring**: 4 utility lands + 2 utility artifacts go in
    `colorless_pool`. Each bridge lives in both relevant cross-color
    extension blocks (mirroring the talisman cycle).
  - 13 new tests; lib suite 532 → 545.

- **modern_decks-9: 10 more cube cards + Mind Rot auto-target fix**:
  - 10 new cards in `catalog::sets::decks::modern` and the
    appropriate cube pools — see DECK_FEATURES.md modern_decks-9
    entry for the per-card breakdown. Headline pickups:
    **discard** (Despise, Distress), **burn** (Lava Coil, Jaya's
    Greeting), **draw** (Telling Time, Read the Tides), **cost
    tax** (Vryn Wingmare), **removal** (Last Gasp), **graveyard
    enabler** (Wild Mongrel).
  - **Engine: `primary_target_filter` now covers Discard / Mill /
    Draw / Drain / AddPoison** — fixes Mind Rot which previously
    fell through the match arms in the bot's `auto_target_for_effect`
    heuristic, leaving the spell unplayable.
  - 12 new tests; lib suite 520 → 532.

- **modern_decks-8: 23 new cube cards**:
  - 23 new card factories in `catalog::sets::decks::modern` and the
    appropriate cube pools — see DECK_FEATURES.md modern_decks-8 entry
    for the per-card breakdown. Headline pickups: **green ramp**
    (Rampant Growth, Cultivate, Farseek, Sakura-Tribe Elder, Wood Elves,
    Elvish Mystic), **card draw** (Harmonize, Concentrate, Anticipate,
    Divination, Ambition's Cost), **burn** (Incinerate, Searing Spear,
    Flame Slash, Roast), **removal** (Smother, Final Reward, Holy Light,
    Mana Tithe, Path of Peace, Severed Strands).
  - 25 new functionality tests; lib suite 495 → 520.

- **modern_decks-7: 16 new cube cards + auto-target Player-skip + clippy/server cleanup**:
  - 16 new cards added to `catalog::sets::decks::modern` and the
    appropriate cube pools — see DECK_FEATURES.md modern_decks-7 entry
    for the per-card breakdown.
  - **Engine: `auto_target_for_effect` now skips Player targets for
    permanent-only effects**. Previously an `Any`-filtered Move
    (Regrowth) would auto-target a player and silently fizzle since
    `Effect::Move` only consumes EntityRef::{Permanent,Card}. The new
    `Effect::accepts_player_target()` classifier gates the Player rung,
    routing the heuristic straight to the graveyard / battlefield /
    exile fall-through for permanent-targeting effects. See
    DECK_FEATURES.md for the full rationale.
  - **Cleanup**: clippy `items_after_test_module` on the standalone
    server (test module moved to bottom of `main.rs`); explicit
    `#[allow(clippy::too_many_arguments)]` on
    `continue_spell_resolution` with a comment explaining why the
    8-arg signature is intentional.
  - 24 new tests; lib suite 471 → 495.

- **modern_decks-6: 15 new cube cards + graveyard-source auto-target + server log peering**:
  - **New cards** in `catalog::sets::decks::modern` (all built on existing primitives):
    - **Red discard-loot**: `tormenting_voice` ({1}{R}), `wild_guess` ({2}{R}), `thrill_of_possibility` ({1}{R} instant). All three model "discard 1, draw 2" cost-as-first-step (additional-cost discard folded into resolution).
    - **Burn**: `volcanic_hammer` ({1}{R} sorcery 3-dmg), `slagstorm` ({2}{R} modal: ForEach Creature ⨯ 3 / ForEach Player ⨯ 3), `stoke_the_flames` ({4}{R} convoke 4-dmg).
    - **Counters**: `cancel` ({1}{U}{U} counter target spell), `annul` ({U} counter artifact/enchantment spell).
    - **Black removal/utility**: `heros_downfall` ({1}{B}{B} destroy creature/PW), `cast_down` ({1}{B} destroy nonlegendary creature), `mind_rot` ({2}{B} target player discards 2), `raise_dead` ({B} return creature card from graveyard to hand).
    - **White**: `healing_salve` ({W} +3 life), `raise_the_alarm` ({1}{W} create two 1/1 Soldier tokens).
    - **Green ETB destroy**: `reclamation_sage` ({2}{G} 2/1 Elf Shaman ETB destroy artifact/enchantment), `acidic_slime` ({3}{G}{G} 2/2 Deathtouch Ooze ETB destroy artifact/enchantment/land).
  - **Cube wiring**: each card placed in its primary color's `*_pool` (mono-color cards in the always-available bucket).
  - **Engine: graveyard-source preference in `auto_target_for_effect`**: new `Effect::prefers_graveyard_source` classifies `Move(target → Hand(You))` and `Move(target → Battlefield(You))` as reanimate-class. The auto-target now walks the *caster's graveyard* first for these effects, so the random bot stops Disentomb-bouncing the opp's living bear into its own hand. `prefers_friendly_target` got the same Move-into-your-zone arm, so the friendly heuristic picks up reanimate too. Tests: `auto_target_disentomb_prefers_creature_in_your_graveyard`, `auto_target_raise_dead_prefers_creature_in_your_graveyard`.
  - **Engine cleanup**: `Subtypes` had a hand-rolled `PartialEq` impl enumerating each `Vec` field — replaced with `#[derive(PartialEq, Eq)]` (auto-derive does the same field-by-field compare). Folded a nested `if-let` in `bot.rs` into `&&`. Added `#[allow(clippy::borrowed_box)]` on `serialize_decider` since serde-derive needs the `&Box<T>` shape.
  - **Engine: add `Ooze` and `Plant` to `CreatureType`**: required so Acidic Slime can carry its canonical Ooze subtype rather than collapsing into Beast.
  - **Server: peer-stamped match-end logs**: `run_pair_match` now records both peer addresses on the match-end log line ("pair match ended (A ↔ B)"). Wrap-failure on either seat now drops the surviving stream so the live client gets EOF instead of a hung socket.
  - **Tests**: 18 new functionality tests in `tests/modern.rs` (one or two per new card) + 2 auto-target tests in `tests/game.rs`. Suite: 449 → 471 (+22 incl. server).

- **modern_decks-5: 12 new cards + auto-target friendly preference + server format warning**:
  - **New cards** in `catalog::sets::decks::modern` — all built on existing engine primitives:
    - **Talisman cycle** (RW / UR / GU): `talisman_of_conviction`, `talisman_of_creativity`, `talisman_of_curiosity` — extracted shared `talisman_cycle` helper that mirrors the Talisman of Progress / Dominance shape.
    - **Edict cycle**: `innocent_blood` ({B}, each player sacs), `diabolic_edict` ({1}{B}, target player sacs), `geths_verdict` ({1}{B}, target sacs + 1 life drain).
    - **Burn / interaction**: `magma_jet` ({1}{R} 2 dmg + scry 2), `remand` ({1}{U} counter + bounce + cantrip), `read_the_bones` ({2}{B} scry-2 / draw-2 / lose-2), `ancient_grudge` ({R} destroy artifact + flashback {G}), `tragic_slip` ({B} -13/-13 EOT).
    - **Body**: `storm_crow` ({1}{U} 1/2 flying Bird).
  - **Cube wiring**: each new card is in the appropriate color pool (Talismans on relevant cross-pool branches; the rest in their primary color's mono-pool).
  - **Engine: friendly-target preference in `auto_target_for_effect`**: new `Effect::prefers_friendly_target` classifies non-negative `PumpPT` / `GrantKeyword` / `+1/+1 AddCounter` as friendly. The auto-target heuristic now picks the *caster's* permanent first for friendly buffs (Vines of Vastwood, Reckless Charge, Ephemerate), and falls back to the existing opponent-first behavior for hostile effects. Without this, the random bot would pump the opp's bear with Vines instead of its own. Tests: `auto_target_prefers_friendly_permanent_for_buff_effect`, `auto_target_pumppt_debuff_falls_back_to_hostile`.
  - **Engine: `add_card_to_graveyard` test helper**: mirrors `add_card_to_hand` / `add_card_to_battlefield` / `add_card_to_library` so reanimate / flashback / dredge fixtures can seed graveyards directly without casting + resolving the spell first.
  - **Server: warn on unrecognized `CRAB_FORMAT`**: previously any value other than `"cube"` silently fell back to demo (so a typo like `Cube` was invisible). The new arm logs a one-line warning to stderr listing the valid values. Three new tests in `crabomination_server::main::tests` covering default, explicit cube, and typo fallback.
  - 16 new tests across `tests/modern.rs` (13 cards), `tests/game.rs` (2 friendly/hostile auto-target), and `crabomination_server` (3 env tests). Suite: 421 → 423 in `crabomination`, plus 3 in the server.

- **10-card batch + `EventScope::FromYourGraveyard`**:
  - **Engine: `EventScope::FromYourGraveyard`**. New scope variant on `EventScope` for triggers that fire from the source card's *owner's graveyard*. `dispatch_triggers_for_events` (mod.rs) now walks every player's graveyard for triggers with this scope in addition to the battlefield; `fire_step_triggers` (stack.rs) does the same for `EventKind::StepBegins`. The scope's "actor" check in `event_matches_spec` uses the card's `owner` (not `controller`, since graveyard cards don't have a meaningful controller). Promotes Bloodghast / Ichorid / Silversmote Ghoul to playable recursion creatures.
  - **Bloodghast** (⏳ → ✅): `LandPlayed` + `FromYourGraveyard` trigger that returns `Selector::This` to the battlefield. The "haste while opp ≤ 10" rider is omitted (no conditional-keyword static yet). Two tests.
  - **Ichorid** (⏳ → 🟡): `StepBegins(Upkeep)` + `FromYourGraveyard` trigger reanimates + schedules a delayed end-step exile (reuses Goryo's reanimate-then-exile pattern). The "opponent has black creature in graveyard" gate is omitted.
  - **Silversmote Ghoul** (⏳ → ✅): `LifeGained` + `FromYourGraveyard` trigger.
  - **Heliod, Sun-Crowned** (🟡 → 🟡 + counter combo): wired the "whenever you gain life, put a +1/+1 counter on target creature you control with lifelink" trigger via a `Creature ∧ ControlledByYou ∧ HasKeyword(Lifelink)` filter — the Walking-Ballista combo line now resolves end-to-end.
  - **Bitterbloom Bearer** (⏳ → ✅): {1}{B} 1/2 Faerie Wizard with Flying. Self-source ETB creates a 1/1 black Faerie token with flying via `Effect::CreateToken` + a one-off `TokenDefinition`.
  - **Dandân** (⏳ → 🟡): {2}{U} 4/1 Fish. Upkeep `If(Not(Island ∧ ControlledByYou), self → graveyard)` trigger captures the downside half. The "can attack only if defending player controls an Island" half is still ⏳.
  - **Turnabout** (⏳ → ✅): {2}{U}{U} Instant. Six-mode `ChooseMode` (artifact / creature / land × tap / untap), each operating against `EachPermanent(Type ∧ ControlledByOpponent)`.
  - **Dread Return** (⏳ → 🟡): {2}{B}{B} Sorcery `Move(target creature card → Battlefield(You))`. Flashback's "sac-three-creatures additional cost" still ⏳.
  - **Tidehollow Sculler** (⏳ → 🟡): {W}{B} 2/2 Zombie ETB DiscardChosen against opp's hand. The "exile until LTB / return on LTB" clause is omitted (no exile-until-LTB primitive yet).
  - 10 new tests; 408/408 passing.
- **Cube-card batch: 10 new cards (Memnite, Fanatic of Rhonas, Greasewrench Goblin, Orcish Lumberjack, Mine Collapse, Elvish Spirit Guide, Satyr Wayfinder, Fireblast, Talisman of Progress/Dominance) + Consign mode 1 test**:
  - **New cards** in `catalog::sets::decks::modern` — all built on existing primitives:
    - **Memnite** ({0} 1/1 Construct artifact creature) — vanilla body. Required adding `CreatureType::Construct` (and `Golem`) to the enum.
    - **Fanatic of Rhonas** ({G} 1/1 Snake) — `{G},{T}: Add {G}{G}`.
    - **Greasewrench Goblin** ({1}{R} 2/2 Haste) — vanilla body; Treasure-on-death ⏳.
    - **Orcish Lumberjack** ({R} 1/1 Goblin Druid) — `{T}, sacrifice a Forest: Add {G}{G}{G}` (sac as first effect step, like Crop Rotation).
    - **Mine Collapse** ({2}{R} sorcery) — sacrifice a Mountain, deal 4 damage to any target.
    - **Elvish Spirit Guide** ({2}{G} 2/2 Elf Spirit) — vanilla body; the exile-from-hand alt-mana ability is gated on a future hand-activation primitive.
    - **Satyr Wayfinder** ({1}{G} 1/1 Satyr Druid) — ETB mills 4.
    - **Fireblast** ({4}{R}{R} instant) — 4 damage; alt-cost sacrifice-two-Mountains ⏳.
    - **Talisman of Progress** ({2} artifact) — `{T}: Add {C}` + `{T}: Add {W}` + `{T}: Add {U}` (each colored ability costs 1 life).
    - **Talisman of Dominance** — UB mirror.
  - **Consign to Memory mode 1**: existing `ChooseMode([CounterAbility, CounterSpell(Legendary)])` was already wired in code but not yet covered by a test. Added `consign_to_memory_mode_one_counters_legendary_spell` exercising the mode-1 branch (counter Thalia, Guardian of Thraben as it goes on the stack). Promoted to ✅ in DECK_FEATURES.md.
  - 11 new tests; 397/397 passing.
- **modern_decks-4: 13 new cards + activated-ability target gate**:
  - **New cards** in `catalog::sets::decks::modern` (built on existing
    engine primitives, no new effects required):
    - **Sweepers**: `pyroclasm` ({1}{R}), `day_of_judgment` ({2}{W}{W}),
      `damnation` ({2}{B}{B}). `ForEach(Creature)` + `DealDamage` /
      `Destroy` (Anger of the Gods shape).
    - **Tutor cycle**: `mystical_tutor` ({U}), `worldly_tutor` ({G}),
      `enlightened_tutor` ({W}), `diabolic_tutor` ({2}{B}{B}),
      `imperial_seal` ({B}). All reuse the existing
      `Search(... → Library{Top})` / `Hand` primitive (same as Vampiric
      Tutor).
    - **Burn**: `lightning_strike` ({1}{R}), `goblin_bombardment`
      ({1}{R} enchantment, sac creature → 1 damage).
    - **Lands**: `wasteland` ({T}, sac → destroy nonbasic),
      `strip_mine` ({T}, sac → destroy any land). Both ride on the
      existing two-ability `{T}: Add {C}` + `sac_cost`-style
      destruction.
    - **Pitch**: `snuff_out` ({3}{B}, or pay 4 life: destroy nonblack
      creature). Reuses `AlternativeCost.life_cost`.
  - **Engine: activated-ability target filter gate**: `activate_ability`
    now enforces `target_filter_for_slot(0)` against the chosen target,
    mirroring the gate already in `cast_spell`. Wasteland on a Plains,
    Goblin Bombardment with a missing target, etc. all return
    `SelectionRequirementViolated` instead of silently fizzling at
    resolution.
  - **Cube wiring**: each new card added to its color pool. Wasteland
    and Strip Mine live in `colorless_pool` (universally useful utility
    lands).
  - 19 new tests in `tests/modern.rs` (370 total in the suite).

- **Cube-card batch (10 promotions)**:
  - **Cryptic Command** (blue ⏳ → 🟡): `{1}{U}{U}{U}` Instant. Single-target ChooseMode of four bundled mode pairs (counter+bounce / counter+tap / counter+draw / bounce+draw). The "tap each opponent's creatures" half is `ForEach + Tap`. AutoDecider picks mode 0 (counter+bounce). Multi-pick "choose any two" still ⏳.
  - **Deadly Dispute** (black ⏳ → ✅): sac-as-additional-cost folded into resolution: `SacrificeAndRemember(Creature ∨ Artifact, ControlledByYou)` → `Draw 2` → `CreateToken(Treasure)`.
  - **Bloodchief's Thirst** (black ⏳ → 🟡): base mode wired (destroy creature/PW with mana value ≤ 2). Kicker `{1}{B}` → mana value ≤ 6 still ⏳.
  - **Heliod, Sun-Crowned** (white ⏳ → 🟡): 3/4 Indestructible body + activated `{1}{W}: target creature gains lifelink EOT`. Devotion-based "isn't a creature unless ≥ 5" + counter-combo trigger ⏳.
  - **Indulgent Tormentor** (black ⏳ → 🟡): 5/3 Demon Flying with end-step trigger draining 3 from each opponent. Multi-player choice (draw vs sac vs lose-3) ⏳.
  - **Eternal Witness** (green new): 2/1 Human Shaman with ETB return-from-graveyard. Auto-target prefers battlefield over graveyard, so the ETB needs UI-driven targeting; structural test only.
  - **Marauding Mako** (UB cross-pool ⏳ → ✅): 2/2 Fish that listens for `CardDiscarded`+`YourControl` and adds a +1/+1 counter to itself.
  - **Simian Spirit Guide** (red ⏳ → 🟡): vanilla 2/2 body. Alt-cost "exile from hand to add {R}" still ⏳ (alt-cost replaces effect, would need a "alt = mana ability" mode).
  - **Containment Priest** (white ⏳ → 🟡): vanilla 2/2 W flash body. Replacement effect needs a creature-ETB-replacement primitive.
  - **Static Prison** (white ⏳ → 🟡): `{X}{2}{W}` Enchantment. ETB stamps Stun counters + taps target permanent. The XFromCost → trigger context propagation still ⏳ (so the counter count reads as 0); the tap-target half is fully functional.
  - All ten new cards are wired into their cube color pools (`color_pool` in `cube.rs`); Marauding Mako lives on the U×B and B×U cross-pools.
  - 11 new tests in `tests/modern.rs`; 353 engine tests pass total.

## Engine features needed

The cube draws on a much wider mechanic set than the BRG / Goryo's pair.
This list is the work needed to bring most of the cube online; the engine
features already implemented (Pact / Flashback / Convoke / Rebound / etc.)
are listed in `DECK_FEATURES.md`.

| Feature | Status | Cards depending on it |
|---|---|---|
| Sacrifice-as-activation-cost (`ActivatedAbility::sac_cost`) | ✅ | Mind Stone, Aether Spellbomb, Cathar Commando, Haywire Mite — and any token-sac mana ability (Treasure, Food, Blood, Clue). |
| Token activated abilities (`TokenDefinition::activated_abilities`) | ✅ | Treasure (`{T}, Sac: Add any color`), Food (`{2}, {T}, Sac: Gain 3 life`), Clue (`{2}, Sac: Draw 1`), Blood (loot). |
| Trigger-filter enforcement (`EventSpec::filter` evaluated in `dispatch_triggers_for_events` + `fire_spell_cast_triggers`) | ✅ | Up the Beanstalk, Temur Ascendancy, and any future "whenever you cast a spell with property X" / "whenever a creature enters with property Y" trigger. |
| Equipment + equip-cost activated ability | ⏳ | Lion Sash, Shuko, Lavaspur Boots, Nettlecyst, Sword of Body and Mind, Helm of the Host. |
| Vehicles + Crew | ⏳ | Smuggler's Copter, Esika's Chariot, Shorikai, Pinnacle Emissary, Coveted Jewel-adjacent. |
| Madness | ⏳ | Anje's Ravager, Blazing Rootwalla, Basking Rootwalla. |
| Cycling | ⏳ | Aether Spellbomb (cycle), Sundering Eruption-adjacent. |
| Adventure (cost-mode duality) | ⏳ | Virtue of Loyalty. |
| Mutate | ⏳ | Mutated Cultist, Mutable Explorer. |
| Cascade | ⏳ | Bloodbraid Challenger, Apex Devastator, Maelstrom Nexus. |
| Storm count + cast-from-top | ⏳ | Mind's Desire (Storm), Amped Raptor / Robber of the Rich (cast-from-top). |
| Delve cost reduction | ⏳ | Treasure Cruise, Dig Through Time, Lose Focus. |
| Ninjitsu | ⏳ | Fallen Shinobi (any future ninjas). |
| Soulbond | ⏳ | Deadeye Navigator. |
| Companion (deck-construction restriction + start-side mana cost) | ⏳ | Zirda, the Dawnwaker. |
| Saga lore counters + DFC | ⏳ | The Everflowing Well; future sagas. |
| Hideaway lands | ⏳ | Shelldock Isle. |
| Horizon-canopy "pay 1 + life to draw" lands | ⏳ | Horizon Canopy, Sunbaked Canyon, Waterlogged Grove. |
| Verge / surveil land family expansion | ⏳ | Each color pair's `*verge` and surveil-land entry. |
| Bridge dual-typed lands | 🟡 | All 10 bridges (Mistvault / Drossforge / Razortide / Goldmire / Silverbluff / Tanglepool / Slagwoods / Thornglint / Darkmoss / Rustvale) ship as ETB-tapped `{T}: Add {C}` lands typed by their two relevant basic land types via the shared `bridge_land()` helper. The "every basic land type" half is collapsed to the two types most relevant to the bridge's color pair. |
| Locus mana scaling | 🟡 | Cloudpost / Glimmerpost present as ETB-tapped colorless lands; the per-Locus mana scaling rider is dropped (no per-source-count mana payload). Glimmerpost's lifegain is wired (flat 1 life). |
| ETB-replacement effects (suppress entirely) | ⏳ | Containment Priest, Static Prison-adjacent, Gather Specimens. |
| Spell-tax statics ("costs {1} more", "costs at least {3}") | 🟡 | Damping Sphere wired (`AdditionalCostAfterFirstSpell`); Trinisphere needs a "minimum cost" flavor. Elite Spellbinder reuses the existing tax static. |
| Land-untap restriction static | ⏳ | Back to Basics. |
| "Cast spells without paying mana" static | ⏳ | Omniscience, Maelstrom Archangel (combat-damage variant), Aluren (free-cast under-3 creatures). |
| Name-a-card / name-a-creature-type primitive | 🟡 | Cavern of Souls has the ETB ChooseMode framing but the chosen type doesn't gate mana provenance yet. Pithing Needle would need the same primitive. |
| Token-copy of permanent | ⏳ | Phantasmal Image, Helm of the Host, Mockingbird, Growing Ranks (populate). |
| Multi-pick decisions over revealed library cards | 🟡 | Atraxa Draw-4 stand-in is wired. Reveal-and-sort by card type, Dig Through Time, Mind's Desire all need a richer multi-pick decision. |
| Investigate + Clue token | ⏳ | Tireless Tracker, Lonis, Proft's Eidetic Memory. |
| Dredge | ⏳ | Golgari Grave-Troll, Golgari Thug, Life from the Loam, Gitrog. |
| Landfall trigger | 🟡 | Bloodghast wired via the new `EventScope::FromYourGraveyard` (graveyard-source `LandPlayed` trigger). Standard battlefield-side landfall (Omnath) still uses the existing `LandPlayed` + `YourControl` path; both are functional. |
| Pact-style upkeep cost (`PayOrLoseGame`) | ✅ | Engine primitive already exists; reuse for Slaughter Pact, Pact of the Titan, Intervention Pact, etc. |
| Counter-spell / counter-ability primitives | ✅ | `CounterSpell`, `CounterUnlessPaid`, `CounterAbility` are all wired (Spell Snare, Daze, Tishana's Tidebinder, Mystical Dispute reuse them). |
| Flashback / Rebound | ✅ | Wired for Faithful Mending and Ephemerate; reusable by Dread Return, Dreadhorde Arcanist. |
| Surveil + Scry primitives | ✅ | Already wired (Quantum Riddler, surveil lands, Consider). |
| Loyalty abilities w/ static | 🟡 | Teferi works fully; needs further variants (Ashiok exile-and-cast, Karn -2 fetch, Saheeli token-copy, Sorin drain, Tezzeret etheric, Tamiyo). |
| Split / DFC modal cards (`Wear // Tear`) | ⏳ | Wear // Tear, Sundering Eruption (sorcery side), most "// Land" DFCs in the cube. |
| Living-weapon-on-ETB token | ⏳ | Nettlecyst, Sword-of-Body-and-Mind-adjacent. |
| Protection-from-color (more colors) | 🟡 | Engine has `Keyword::Protection(Color)`. Cards like Stillmoon Cavalier need toggle abilities + multi-color protection. |
| Charge counters as mana storage | 🟡 | Gemstone Mine wires charge counters + sac-on-empty. Coalition Relic, Power Depot, Pentad Prism, Chalice of the Void all reuse the same primitive. |
| `PermanentLeavesBattlefield` self-source triggers | ✅ | `remove_to_graveyard_with_triggers` now collects both `CreatureDied` and `PermanentLeavesBattlefield` self-source triggers (the former still gates on creature-only). Powers Chromatic Star's cantrip-on-leave and any future non-creature die-trigger. |
| `Selector::CardsInZone` over multi-player refs | ✅ | The resolver now uses `resolve_players` (multi) so `EachPlayer` / `EachOpponent` aggregate cards across every matching seat. Powers Soul-Guide Lantern's mass graveyard exile. Previously the single-player `resolve_player` returned `None` and silently produced an empty list. |

## Plan

The cube is far too large to wire card-by-card on its own. The most
leverage comes from finishing the engine features above, then sweeping
groups of cards in batches. Suggested order:

1. **Token primitives** (Treasure / Blood / Clue / Food). Unlocks ~25
   cube cards in one batch.
2. **Equipment + Vehicles** (equip cost / crew). Unlocks Lion Sash,
   Shuko, Sword of …, Helm, Nettlecyst, Lavaspur Boots, Smuggler's
   Copter, Esika's Chariot.
3. **Cascade / Storm / Madness / Delve / Cycling** are each fairly small
   targeted features that unlock 3–5 cards apiece.
4. **DFC / Split-card / Adventure** infrastructure unlocks a wide swath
   of modern lands plus Wear // Tear, Sundering Eruption, etc.
5. **Multi-pick decisions** (reveal-and-sort, Dig Through Time, Atraxa
   real ETB, Mind's Desire) — promotes Atraxa to ✅ and unlocks several
   blue payoff spells.
6. **Counters / charge-mana / proliferate** for Mossborn Hydra, Heliod,
   Coalition Relic, Power Depot, Cankerbloom, etc.

Promote individual cards from ⏳ to 🟡 → ✅ as their dependent feature
lands. The status table above is the source of truth; update it
alongside the engine row when an effect ships.

## Maybeboard (raw)

Source list, preserved verbatim:

```
Descendant of Storms
Cathar Commando
Containment Priest
Lion Sash
Elite Spellbinder
Enduring Innocence
Flickerwisp
Heliod, Sun-Crowned
Loran of the Third Path
Ranger-Captain of Eos
Restoration Angel
Guardian Scalelord
Serra Angel
Intervention Pact
Isolate
Tempt with Bunnies
Static Prison
Virtue of Loyalty
Mockingbird
Dandân
Phantasmal Image
Thundertrap Trainer
Tishana's Tidebinder
Quantum Riddler
Deadeye Navigator
Pact of Negation
Consider
Spell Snare
Swan Song
Thought Scour
Consult the Star Charts
Daze
Lose Focus
Frantic Search
Cryptic Command
Paradoxical Outcome
Turnabout
Gush
Gather Specimens
Mirrorform
Dig Through Time
Windfall
Mind's Desire
Upheaval
Treasure Cruise
Aether Spellbomb
The Everflowing Well
Proft's Eidetic Memory
Back to Basics
Opposition
Parallax Tide
Omniscience
Shelldock Isle
Sink into Stupor
Concealing Curtains
Moonshadow
Bitterbloom Bearer
Bloodghast
Golgari Thug
Mai, Scornful Striker
Mutated Cultist
Silversmote Ghoul
Ichorid
Necrotic Ooze
Indulgent Tormentor
Crabomination
Doomsday Excruciator
Metamorphosis Fanatic
Slaughter Pact
Deadly Dispute
Corpse Dance
Baleful Mastery
Bloodchief's Thirst
Bone Shards
Disentomb
Collective Brutality
Drown in Ichor
Fell
Night's Whisper
Dread Return
Blasphemous Edict
Wishclaw Talisman
Parallax Dementia
Parallax Nexus
Unholy Annex // Ritual Chamber
Blazing Rootwalla
Greasewrench Goblin
Grim Lavamancer
Marauding Mako
Orcish Lumberjack
Voldaren Epicure
Amped Raptor
Cam and Farrik, Havoc Duo
Dreadhorde Arcanist
Magda, Brazen Outlaw
Robber of the Rich
Anje's Ravager
Death-Greeter's Champion
Detective's Phoenix
Simian Spirit Guide
Arclight Phoenix
Goldspan Dragon
Shivan Dragon
Balefire Dragon
Pact of the Titan
Tarfire
Chaos Warp
Big Score
Mine Collapse
Fireblast
Pyrokinesis
Vandalblast
Legion Extruder
Sundering Eruption
Basking Rootwalla
Elvish Reclaimer
Haywire Mite
Sylvan Safekeeper
Basking Broodscale
Cankerbloom
Collector Ouphe
Fanatic of Rhonas
Keen-Eyed Curator
Rofellos, Llanowar Emissary
Satyr Wayfinder
Sylvan Caryatid
Elvish Spirit Guide
Enduring Vitality
Hauntwoods Shrieker
Mossborn Hydra
Mutable Explorer
Sentinel of the Nameless City
Tireless Tracker
Ursine Monstrosity
Baloth Prime
Icetill Explorer
Mightform Harmonizer
Ouroboroid
Sowing Mycospawn
Vengevine
Elder Gargaroth
Golgari Grave-Troll
Railway Brawler
Conclave Sledge-Captain
Lumra, Bellow of the Woods
Zopandrel, Hunger Dominus
Apex Devastator
Summoner's Pact
Nature's Claim
Archdruid's Charm
Finale of Devastation
Life from the Loam
Nature's Lore
Kodama's Reach
Biorhythm
Esika's Chariot
Springleaf Parade
Up the Beanstalk
Aluren
Greater Good
Shifting Woodland
Ornithopter
Ornithopter of Paradise
Glaring Fleshraker
Tezzeret, Cruel Captain
Karn, Scion of Urza
Kozilek's Command
Eldrazi Confluence
Chalice of the Void
Zuran Orb
Candelabra of Tawnos
Chromatic Star
Ghost Vacuum
Lavaspur Boots
Pithing Needle
Shuko
Soul-Guide Lantern
Agatha's Soul Cauldron
Fellwar Stone
Mesmeric Orb
Millstone
Mind Stone
Pentad Prism
Smuggler's Copter
Coalition Relic
Monument to Endurance
Nettlecyst
Sword of Body and Mind
Trinisphere
Helm of the Host
The Mightstone and Weakstone
Coveted Jewel
The Endstone
Portal to Phyrexia
Talisman of Progress
Shorikai, Genesis Engine
Cruel Somnophage
Talisman of Dominance
Ashiok, Nightmare Weaver
Master of Death
Fallen Shinobi
Bloodtithe Harvester
Terminate
Carnage Interpreter
Kolaghan's Command
Master of Cruelties
Territorial Kavu
Bloodbraid Challenger
Qasali Pridemage
Knight of the Reliquary
Brightglass Gearhulk
Growing Ranks
Torsten, Founder of Benalia
Tidehollow Sculler
Gift of Orzhova
Stillmoon Cavalier
Sorin, Grim Nemesis
Expressive Iteration
Talisman of Creativity
Pinnacle Emissary
Saheeli Rai
Tempest Angler
Abrupt Decay
Assassin's Trophy
Broodspinner
Tear Asunder
Wight of the Reliquary
The Gitrog Monster
Talisman of Conviction
Wear // Tear
Zirda, the Dawnwaker
Talisman of Curiosity
Lonis, Genetics Expert
Tamiyo, Collector of Tales
Sab-Sunen, Luxa Embodied
Koma, Cosmos Serpent
Kestia, the Cultivator
Messenger Falcons
Dakkon, Shadow Slayer
Urza, Chief Artificer
Geyadrone Dihada
Lord Xander, the Collector
Korvold, Fae-Cursed King
Temur Ascendancy
Loot, the Pathfinder
Dragonback Assault
Rediscover the Way
Shiko and Narset, Unified
Awaken the Honored Dead
Fangkeeper's Familiar
Teval, Arbiter of Virtue
Muldrotha, the Gravetide
Rakshasa's Bargain
Omnath, Locus of Creation
Atraxa, Grand Unifier
Leyline of the Guildpact
Golos, Tireless Pilgrim
Maelstrom Archangel
Maelstrom Nexus
Ramos, Dragon Engine
Celestial Colonnade
Meticulous Archive
Razortide Bridge
Seachrome Coast
Creeping Tar Pit
Darkslick Shores
Mistvault Bridge
Undercity Sewers
Blackcleave Cliffs
Blazemire Verge
Drossforge Bridge
Raucous Theater
Commercial District
Copperline Gorge
Slagwoods Bridge
Thornspire Verge
Horizon Canopy
Lush Portico
Razorverge Thicket
Thornglint Bridge
Bleachbone Verge
Concealed Courtyard
Goldmire Bridge
Shadowy Backstreet
Riverpyre Verge
Silverbluff Bridge
Spirebluff Canal
Thundering Falls
Blooming Marsh
Darkmoss Bridge
Underground Mortuary
Wastewood Verge
Elegant Parlor
Inspiring Vantage
Rustvale Bridge
Sunbaked Canyon
Botanical Sanctum
Hedge Maze
Tanglepool Bridge
Waterlogged Grove
Twisted Landscape
Sheltering Landscape
Bountiful Landscape
Ancient Den
Cloudpost
Darksteel Citadel
Evolving Wilds
Exotic Orchard
Glimmerpost
Great Furnace
Lotus Field
Planar Nexus
Power Depot
Rishadan Port
Seat of the Synod
Talon Gates of Madara
Three Tree City
Tree of Tales
Trenchpost
Vault of Whispers
```
