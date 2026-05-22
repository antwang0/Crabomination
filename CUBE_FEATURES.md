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
| Guardian Scalelord | ✅ (was ⏳) | Push (modern_decks): 4/4 Dragon Flying. `Attacks/SelfSource` trigger → `MayDo(GrantKeyword(Flying, EOT, target friendly creature))`. AutoDecider declines optional rider by default; scripted decider can opt in. Tests: `guardian_scalelord_attack_grants_flying_to_target_friendly`, `guardian_scalelord_declines_optional_grant_by_default`. |
| Serra Angel | ✅ | Already in `crate::catalog::serra_angel` (4/4 flying + vigilance). |
| Intervention Pact | ⏳ | Free prevent-damage + delayed `PayOrLoseGame` upkeep cost (reuses Pact primitive). |
| Isolate | ✅ | Exile target permanent with mana value 1 (`ManaValueAtLeast(1) ∧ ManaValueAtMost(1)` filter). |
| Tempt with Bunnies | ⏳ | Tempting offer (chain-creating) — needs multi-player choice primitive. |
| Static Prison | ✅ (was 🟡) | `{X}{2}{W}` Enchantment. Push (modern_decks): ETB now stamps `Value::XFromCost` Stun counters on the **target** (CR 122.1d) — was previously stamping them on Static Prison itself. The engine's existing stun-counter mechanic consumes one counter per untap step, keeping the target tapped for X turn cycles. Combined with the immediate ETB tap, this matches the printed "while it has stun counters, target doesn't untap" rider exactly. Tests: `static_prison_etb_taps_target`, `static_prison_x2_etb_adds_two_stun_counters_to_target`. |
| Virtue of Loyalty | ⏳ | Adventure + enchantment side. Needs Adventure cost-mode primitive. |
| Healing Salve | ✅ | {W} Instant. Gain 3 life on target. Damage-prevention mode collapsed (no prevention-shield primitive). Test: `healing_salve_gives_three_life`. |
| Raise the Alarm | ✅ | {1}{W} Instant. Create two 1/1 white Soldier tokens via `CreateToken`. Test: `raise_the_alarm_creates_two_soldier_tokens`. |
| Wall of Omens | ✅ | Push (claude/modern_decks, NEW): {1}{W} Creature — Wall 0/4 with Defender. ETB Draw 1. Test: `wall_of_omens_etbs_and_draws`. |
| White Sun's Zenith | ✅ | Push (claude/modern_decks, NEW): {X}{W}{W}{W} Instant. Create X 2/2 white Cat tokens via `CreateToken { count: XFromCost }`. "Shuffle into library" rider collapses. Test: `white_suns_zenith_creates_x_cat_tokens`. |

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
| Frantic Search | ✅ (was 🟡) | Draw 2, discard 2, untap up to three of your tapped lands. The `Effect::Untap { up_to: Some(Value::Const(3)) }` field caps the untap count to 3 (push V engine primitive), matching the printed Oracle exactly. |
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
| Snapcaster Mage | ✅ | Push (claude/modern_decks, NEW): {1}{U} Creature — Human Wizard 2/1 Flash. ETB stamps `GrantMayPlay { exile_after: true, EndOfThisTurn }` on a target IS card in your gy. Same shape as Flashback (the spell). Approximation: free cast (no `MayPlayPermission.alt_cost`). Tests: `snapcaster_mage_etb_grants_may_play_on_gy_is_card`, `snapcaster_mage_is_a_two_one_flash_wizard`. |
| Tale's End | ✅ | Push (claude/modern_decks, NEW): {1}{U} Instant. `ChooseMode([CounterSpell(Legendary), CounterAbility])`. Mode 0 counters target legendary spell; mode 1 counters any ability. "Saga" clause collapses to the ability-counter half. Test: `tales_end_counters_a_legendary_spell`. |
| Hydroblast | ✅ | Push (claude/modern_decks, NEW): {U} Instant. ChooseMode counter red spell or destroy red permanent. AutoDecider picks mode 0. Test: `hydroblast_counters_a_red_spell`. |
| Blue Elemental Blast | ✅ | Push (claude/modern_decks, NEW): {U} Instant. Functionally Hydroblast. Test: `blue_elemental_blast_counters_a_red_spell`. |
| Stroke of Genius | ✅ | Push (claude/modern_decks, NEW): {X}{2}{U} Instant. Target player draws X cards. Reads `XFromCost`. Test: `stroke_of_genius_draws_x_cards`. |

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
| Wishclaw Talisman | 🟡 | Push (claude/modern_decks batch 102): {1}{B} Artifact. ETB with 3 charge counters. `{T}, Remove a charge counter: Search your library for a card → hand.` Tutor body + cost wired. The "opp gains control" downside is engine-wide ⏳ (no `GainControlBy { who: opp }` variant — `Effect::GainControl` always uses the activator's controller). Tests: `wishclaw_talisman_enters_with_three_charge_counters`, `wishclaw_talisman_searches_and_consumes_a_charge_counter`. |
| Parallax Dementia | ⏳ | Fading + reanimate; needs fade counters. |
| Parallax Nexus | ⏳ | Fading + hand-strip. |
| Unholy Annex // Ritual Chamber | ⏳ | DFC enchantment land. |
| Toxic Deluge | ✅ | Push (claude/modern_decks, NEW): {2}{B}{B} Sorcery. Pay X life, each creature gets -X/-X EOT. Cost-as-first-step (life payment folded into resolution). Test: `toxic_deluge_sweeps_creatures_for_x_two`. |
| Demonic Consultation | ✅ | Push (claude/modern_decks, NEW): {B} Instant. Mill 6 + search any 1 card → hand. The "name a card / exile until match" rider collapses; misses route to graveyard instead of exile (strictly more recoverable). Test: `demonic_consultation_mills_six_and_searches`. |
| Phyrexian Reclamation | ✅ | Push (claude/modern_decks, NEW): {B}{B} Enchantment. `{1}{B}, Pay 2 life`: Move target creature card from your graveyard → hand. Test: `phyrexian_reclamation_returns_creature_for_one_b_two_life`. |
| Ophiomancer | ✅ | Push (claude/modern_decks, NEW): {2}{B} Creature — Human Shaman 1/1. Beginning of each upkeep, create a 1/1 black Snake token with Deathtouch. "If you control no Snakes" gate collapses (always fires). Test: `ophiomancer_mints_a_snake_each_upkeep`. |
| Black Sun's Zenith | ✅ | Push (claude/modern_decks, NEW): {X}{B} Sorcery. `ForEach creature + AddCounter(-1/-1, X)`. Test: `black_suns_zenith_puts_x_minus_one_counters_on_each_creature`. |
| Hymn to Tourach | ✅ | Push (claude/modern_decks, registered in cube — card lives in `fem::sorceries`): {B}{B} Sorcery. Target player discards 2 random cards via `Effect::Discard { random: true }`. |

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
| Tarfire | ✅ (was 🟡) | Push (modern_decks): the printed type line — "Kindred Instant — Goblin" — is now fully wired. `CardType::Kindred` + `CardType::Instant` + `CreatureType::Goblin` subtype. Goblin-tribal payoffs can recognise Tarfire on the stack. Tests: `tarfire_deals_two_damage_to_player`, `tarfire_deals_two_damage_to_creature`, `tarfire_carries_kindred_and_goblin_subtype`. |
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
| Pyroblast | ✅ | Push (claude/modern_decks, NEW): {R} Instant. ChooseMode counter blue spell or destroy blue permanent. Tests: `pyroblast_counters_a_blue_spell`, `pyroblast_rejects_non_blue_spell_target`. |
| Red Elemental Blast | ✅ | Push (claude/modern_decks, NEW): {R} Instant. Functionally Pyroblast (Alpha printing). Test: `red_elemental_blast_counters_a_blue_spell`. |
| Red Sun's Zenith | ✅ | Push (claude/modern_decks, NEW): {X}{R} Instant. X damage to any target via `DealDamage(amount: XFromCost)`. "Exile-if-would-die" and "shuffle into library" riders collapse. Test: `red_suns_zenith_deals_x_damage_to_target`. |

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
| Rofellos, Llanowar Emissary | ✅ (was 🟡) | Push (claude/modern_decks): `{G}{G}` Legendary 2/1 Elf Druid. `{T}: Add {G}{G} for each Forest you control` now wired faithfully via `ManaPayload::OfColor(Green, Times(Const(2), CountOf(Forest ∧ ControlledByYou)))`. The dynamic Forest count is read live at resolution time. Tests: `rofellos_taps_for_two_green_mana` (1 Forest → 2 green), `rofellos_taps_for_two_green_per_forest` (3 Forests → 6 green), `rofellos_taps_for_zero_with_no_forests` (degenerate 0-Forest case). |
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
| Nature's Claim | ✅ | `{G}` Instant. Destroy target artifact or enchantment; its controller gains 4 life (`Destroy + GainLife(ControllerOf(target), 4)`). Lives in `mod_set::natures_claim`. |
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
| Three Visits | ✅ | Push (claude/modern_decks, NEW): {1}{G} Sorcery. Identical to Nature's Lore (search Forest → BF untapped). Includes the duplicate so green ramp shells can run the full eight-copy package. Test: `three_visits_fetches_a_forest_to_battlefield`. |
| Wall of Roots | ✅ | Push (claude/modern_decks, NEW): {1}{G} Creature — Plant Wall 0/5 with Defender. Once-per-turn `0: -0/-1 toughness counter + Add {G}`. Approximation: permanent `Duration::Permanent` pump as a -1 toughness stand-in. Test: `wall_of_roots_taps_for_green_with_pump_cost`. |
| Channel | ✅ | Push (claude/modern_decks, NEW): {G} Sorcery (Kamigawa). Approximated as `Seq([LoseLife 1, AddMana 1])`. The printed "until end of turn, pay 1 life for {1}" alt-payment static is collapsed. Test: `channel_pays_one_life_for_one_mana`. |
| Sylvan Library | ✅ | Push (claude/modern_decks, NEW): {1}{G} Enchantment. Beginning of your draw step, MayDo: Draw 1 + LoseLife 4. Approximation: the "draw 2 extra, return 2 unless you pay 8" loop collapses to "draw 1 / lose 4." Test: `sylvan_library_offers_draw_in_exchange_for_four_life`. |
| Yavimaya Elder | ✅ | Push (claude/modern_decks, NEW): {1}{G}{G} Creature — Human Druid 2/1. Dies trigger: MayDo Search basic land → hand twice. `{2}{G}`, sac: Draw 1. Tests: `yavimaya_elder_dies_searches_two_basics`, `yavimaya_elder_sac_draws_a_card`. |
| Pernicious Deed | ✅ | Push (claude/modern_decks, NEW): {1}{B}{G} Enchantment. `{X}, Sacrifice this`: destroy each artifact, creature, and enchantment with mana value X or less. Wired via `sac_cost: true` activation, X paid through the new `GameAction::ActivateAbility.x_value` field (engine extension). Test: `pernicious_deed_destroys_low_cmc_permanents`. |
| Green Sun's Zenith | ✅ | Push (claude/modern_decks, NEW): {X}{G} Sorcery. `Search(Creature ∧ Green → BF untapped)`. "Shuffle into library" rider collapses. Test: `green_suns_zenith_tutors_green_creature_with_cmc_x`. |

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
| Trinisphere | 🟡 | Push (claude/modern_decks batch 102): {3} Artifact body wired as a vanilla 3-mana artifact. The "spells cost at least {3}" minimum-cost static is engine-wide ⏳ (the engine has `AdditionalCostAfterFirstSpell` for cost-tax, but no minimum-cost-floor primitive yet). Ships in the colorless pool. Test: `trinisphere_is_a_three_mana_artifact`. |
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
| Howling Mine | ✅ | Push (claude/modern_decks, NEW): {2} Artifact. At the beginning of each player's draw step, that player draws an additional card (`StepBegins(Draw)/AnyPlayer` → `Draw(ActivePlayer)`). The "if untapped" gate collapses. Test: `howling_mine_draws_an_extra_card_each_turn`. |
| Ashiok, Nightmare Weaver | 🟡 | Push (claude/modern_decks batch 102): {1}{U}{B} 3-loyalty Planeswalker. **+2**: target opponent mills 3 (the "exiled with Ashiok" linkage is engine-wide ⏳ — milled cards land in opp graveyard). **-1**: Exile target opp creature (the "create a copy" half collapses). **-10**: Approximated as `WinGame { You }` (the "each opp draws 7 from exile" plinker ultimate is dropped). Tests: `ashiok_nightmare_weaver_plus_two_mills_opponent_three`, `ashiok_nightmare_weaver_minus_one_exiles_creature`. |
| Master of Death | ⏳ | UB recursion + discard. |
| Fallen Shinobi | ⏳ | Ninjitsu + reveal-and-take. |
| Bloodtithe Harvester | 🟡 | ETB and attack triggers each create a Blood token. Sac-Blood ping ability omitted (no sac-of-other-permanent activation primitive). |
| Terminate | ✅ | Already in catalog (destroy can't-regenerate). |
| Carnage Interpreter | ⏳ | TBD. |
| Kolaghan's Command | 🟡 | Push (claude/modern_decks batch 102): {1}{B}{R} Instant. Modal — `ChooseMode([discard+reanimate, ping+destroy-artifact, discard+ping])`. The printed "choose two of four" multi-mode picker (CR 700.2d) collapses to three bundled pairs. AutoDecider picks mode 0. Test: `kolaghans_command_mode_zero_discard_plus_reanimate`. |
| Master of Cruelties | 🟡 | Push (claude/modern_decks batch 102): {2}{B}{R} 1/4 First Strike Deathtouch Demon. Attack trigger sets the defending player's life to 1 (via `Effect::SetLifeTotal`). The "can attack only alone" combat restriction and "deals no combat damage this turn" rider are dropped (no engine primitives) — combined with the deathtouch ping, the net play pattern matches the printed kill condition. Test: `master_of_cruelties_attack_sets_opp_life_to_one`. |
| Territorial Kavu | ✅ | Push (claude/modern_decks batch 102): {2}{R}{G} 3/2 Kavu. `LandPlayed` + `OpponentControl` trigger → `AddCounter(+1/+1, Self)`. Test: `territorial_kavu_grows_when_opponent_plays_a_land`. |
| Bloodbraid Challenger | ⏳ | Cascade. |
| Qasali Pridemage | ⏳ | Exalted + sac to destroy artifact/enchantment. |
| Knight of the Reliquary | ⏳ | Sac-land tutor scaling P/T. |
| Brightglass Gearhulk | ⏳ | TBD. |
| Growing Ranks | ⏳ | Populate token-copy on upkeep. |
| Torsten, Founder of Benalia | ⏳ | TBD. |
| Tidehollow Sculler | 🟡 | {W}{B} 2/2 Zombie. ETB picks a nonland card from a target opponent's hand and sends it to their graveyard (approximation of "exile until this leaves"). The "return when this leaves" clause is omitted (no exile-until-LTB primitive yet). Reuses `DiscardChosen`. Test: `tidehollow_sculler_etb_takes_an_opponent_card`. |
| Gift of Orzhova | ⏳ | Aura — flying + lifelink. |
| Stillmoon Cavalier | ✅ | Push (claude/modern_decks batch 102): {1}{W}{B} 2/2 Zombie Knight with four activated abilities — `{W}: gain flying EOT`, `{B}: gain first strike EOT`, `{1}{W}: gain protection from black EOT`, `{1}{B}: gain protection from white EOT`. All four use `Effect::GrantKeyword(EndOfTurn)`. Tests: `stillmoon_cavalier_grants_flying_eot`, `stillmoon_cavalier_grants_protection_from_black_eot`. |
| Sorin, Grim Nemesis | 🟡 | Push (claude/modern_decks batch 102): {4}{B}{B} 6-loyalty Planeswalker. **+1**: Draw 1 + Lose 3 life (approximation; reveal/MV-life-loss/conditional-token chain dropped). **-X**: ping (the X-cost loyalty path uses `Value::XFromCost` against creature/PW + 1 gain life). **-9**: drain 10 from each opponent (the printed "X = cards in opp's graveyard" scaling collapses). Tests: `sorin_grim_nemesis_plus_one_draws_and_loses_three_life`, `sorin_grim_nemesis_minus_nine_drains_each_opponent`. |
| Expressive Iteration | ⏳ | UR look-at-top-3 multi-pick. |
| Talisman of Creativity | ✅ | UR mana rock — see Artifacts row. |
| Pinnacle Emissary | ⏳ | TBD. |
| Saheeli Rai | 🟡 | Push (claude/modern_decks batch 102): {1}{U}{R} 3-loyalty Planeswalker. **+1**: Scry 1 + ping each opponent for 1 (the "and each PW they control" half drops — no `EachOpponentsPlaneswalker` selector). **-2**: Create a token copy of target friendly creature/artifact, grant haste, delay-trigger Exile at next end step. **-7**: Same body fired twice (the emblem-recurring "each end step" auto-recur is approximated). Tests: `saheeli_rai_plus_one_pings_each_opponent`, `saheeli_rai_minus_two_creates_haste_copy`. |
| Tempest Angler | ⏳ | TBD. |
| Abrupt Decay | ⏳ | BG removal: destroy nonland with mana value ≤ 3, can't be countered. |
| Assassin's Trophy | ⏳ | BG removal — opp searches for basic. |
| Broodspinner | ⏳ | TBD. |
| Tear Asunder | ⏳ | BG kicker. |
| Wight of the Reliquary | ⏳ | Land-tutor sac variant. |
| The Gitrog Monster | ⏳ | Land-as-discard / dredge engine. |
| Talisman of Conviction | ✅ | RW mana rock — see Artifacts row. |
| Wear // Tear | 🟡 | Push (claude/modern_decks batch 102): {1}{R} Sorcery, single-spell approximation that destroys target artifact OR enchantment. The Split-Card primitive (CR 709) is engine-wide ⏳ — both halves collapse to a single faithful effect at the Wear cost. Fuse mode is dropped. Test: `wear_tear_destroys_target_artifact`. |
| Zirda, the Dawnwaker | ⏳ | Companion + activated-cost reduction. Needs Companion primitive. |
| Talisman of Curiosity | ✅ | GU mana rock — see Artifacts row. |
| Lonis, Genetics Expert | ⏳ | Investigate + Clue draw. |
| Tamiyo, Collector of Tales | 🟡 | Push (claude/modern_decks batch 102): {2}{G}{U} 4-loyalty Planeswalker. **-2**: Return target card from gy → hand. **-3**: Search library → hand (the "same name as a card in target opponent's graveyard" filter is engine-wide ⏳ — falls back to `Any`). **-7**: Draw 4 (the "distinct nonland types in gy" scaling drops). The static "spells your opps control can't make you discard or sac" is engine-wide ⏳. Test: `tamiyo_collector_minus_two_returns_card_from_graveyard`. |
| Sab-Sunen, Luxa Embodied | ⏳ | TBD. |
| Koma, Cosmos Serpent | ⏳ | Token-on-upkeep + sac counters. |
| Kestia, the Cultivator | ⏳ | Aura/enchantment matters. |
| Messenger Falcons | ⏳ | TBD. |
| Dakkon, Shadow Slayer | ⏳ | TBD. |
| Urza, Chief Artificer | ⏳ | Planeswalker / commander. |
| Geyadrone Dihada | 🟡 | Push (claude/modern_decks batch 102): {2}{B}{R} 3-loyalty Planeswalker. **+1**: Each opp loses 1 + you draw 1 (the "if you have less life than an opp, reset loyalty" rider drops — no loyalty-set primitive). **-3**: Threaten — `GainControl(EOT) + Untap + GrantKeyword(Haste, EOT)`. **-7**: Each opp loses 10 (half-life approximation). Tests: `geyadrone_dihada_plus_one_drains_each_opponent_for_one`, `geyadrone_dihada_minus_three_steals_creature`. |
| Lord Xander, the Collector | 🟡 | Push (claude/modern_decks batch 102): {3}{U}{B}{R} 6/6 Flying Legendary Vampire Demon Noble. ETB makes target opp discard 3 (`DiscardChosen`). Attack trigger mills each opp 8 (the "half their library" scaling collapses to a fixed midgame value). Die trigger makes each opp sacrifice 3 nonland permanents (the "half their permanents" scaling collapses). Test: `lord_xander_the_collector_etb_makes_opponent_discard_three`. |
| Korvold, Fae-Cursed King | ✅ | Push (claude/modern_decks batch 102): {2}{B}{R}{G} 4/4 Flying Legendary Dragon Noble. `PermanentSacrificed` + `YourControl` trigger → AddCounter(+1/+1, This) + Draw 1. The new `EventKind::PermanentSacrificed` / `GameEvent::PermanentSacrificed` (CR 701.16) ships alongside this card and fires for every sacrifice resolution regardless of card type — so Korvold catches Treasure-sac / Clue-sac / Food-sac / land-sac / creature-sac uniformly. Tests: `korvold_fae_cursed_king_triggers_on_sacrifice`, `korvold_fae_cursed_king_triggers_on_artifact_sacrifice_via_permanent_event`. |
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
