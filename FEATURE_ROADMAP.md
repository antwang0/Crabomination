# Feature Roadmap — MTGO / Arena / XMage parity

A prioritized capabilities roadmap (engine fidelity + UX + infra), derived from
a codebase analysis against the three reference clients. Per-card status lives
in `CUBE_FEATURES.md` / `DECK_FEATURES.md` / `STRIXHAVEN2.md`; the
approximations log is `CARD_BACKLOG.md`.

Legend: ✅ done · 🟡 partial · ⏳ not started. Markers are a point-in-time read —
re-verify before picking up an item.

---

## Already shipped (don't re-propose)

Moved to `SHIPPED.md` (size trigger). Check it before proposing anything.

## Commander status (CR 903 + the 800-series it rests on)

The map for the next Commander run — audited 2026-09-18 against the tree and
the shipped CR text (now `crabomination/MagicCompRules_20261106.txt`, re-pinned
2026-11-26: 713 Meld inserted, 713-733 cited as 714-734, 903.5e / 903.5f). Don't re-audit; update rows as they
move. Per-deck card completion lives in `DECK_FEATURES.md`; what each Commander
session changed lives in ENGINE_BACKLOG under its session id. Last touched
2026-11-01 by `01WRuXc4Bu` (no row moved: Star Trek seats 6163-6188).

### Format core (CR 903)
| Feature | State | Where |
|---|---|---|
| `Format::Commander`: 40 life, 100-card, singleton, command zone at start | ✅ | `format.rs` (`FormatRules`, `validate_commander_deck`), `game/mod.rs::apply_format`, `seat_commanders` |
| Rulebreaker deck construction ("a deck with this commander can have … of any color identity" / "no maximum deck size") | ✅ *since 2026-10-15* | `StaticEffect::RulebreakerAnyColorIdentity { cards }` / `RulebreakerNoMaxDeckSize`, read by `format::validate_commander_deck` only (Maular, Seluma, The Everforger, Whtz; `RulebreakerCards::Auras` — The Unluckiest Planeswalker; `OfCreatureType(Phyrexian)` — Valko; `Lands` — Grizzlegom), and `RulebreakerSpellsOneExtraColor` (Tolabow: the deck's instants and sorceries share one extra color, widened by the color most of them need). CR 903.5e (only from the commander: `core_rules/commander_mana.rs` `cr_903_5e_*`); `recent_b/cmdr_oct15_a.rs`, `recent_b/cmdr_016x_a.rs`, `recent_b/trek_singles8.rs` (`cr_903_5c_*`) |
| Commander designation persists across zones; a copy is not a commander | ✅ | `player.rs::Player::commanders` (zone-independent `CardId`s); `GameState::is_commander`. A token/copy has its own `CardId` |
| Commander tax {2} per prior cast, as an additional cost | ✅ | `actions.rs::cast_from_command_zone` — added before `cost_reduction_for_spell` / `reduce_by_cost`, so reductions and the floor see it |
| CR 903.9a graveyard/exile return as an SBA (dies triggers fire first) | ✅ | `commander_return.rs` (`commander_zone_return_sba`, `commander_return_declined`). A death replaced into exile (Sanctifier en-Vec, Kalitas, Rest in Peace) fires none of the commander's own dies triggers (`death_was_replaced`, `stack.rs`; `cr_700_4_a_replaced_death_fires_no_dies_trigger_of_its_own`); the exile return is still offered. A prompting owner is asked through a pending `CommanderRedirect` once the action settles (`pose_commander_return`, `ResumeContext::CommanderReturn`; `cr_903_9a_prompting_owner_*`), other seats by the decider. Waits for the resolution (CR 704.3): `check_state_based_actions_mid_resolution` runs no return and skips the orphan-Aura check (`cr_704_3_commander_return_waits_for_the_resolution`; Danse Macabre, Shifting Shadow) |
| CR 903.9b hand/library return as a *replacement* | ✅ | `replacement.rs`, registered per commander by `seat_commanders`; `Decision::CommanderRedirect`. Every direct push goes through `commander_return.rs::commander_zone_redirect{,_all}` (countered commander spells, Glimpse of Tomorrow, shuffles, Head Games, Brainstorm-style put-backs; `cr_recent77::cr_903_9b_*`); a debug pod invariant (`pod/mod.rs`) names any new bypass. A prompting owner is offered the redirect once the action settles (`commander_redirect_offers`, `multiplayer::cr_903_9b_prompting_owner_*`); residual: the rest of that resolution sees the card in hand/library |
| "Whenever your commander is put into the command zone" (both routes) | ✅ *since 2026-09-24* | `GameEvent`/`EventKind::CommanderPutIntoCommandZone`, queued by `note_commander_to_command_zone`; Myth Unbound (`cr_903_9_myth_unbound_draws_on_both_routes_home`) |
| Commander damage, 21 from one commander, per (commander, player) | ✅ | `game/mod.rs::commander_damage` + `record_commander_damage`; combat damage only (CR 903.10a) |
| CR 903.4 color identity: cost, rules text, indicator, back faces | ✅ | `color_identity.rs`; audited against Scryfall for every implemented card (3 known gaps in `KNOWN_IDENTITY_GAPS`; a unit-variant static like `FiveColorAlternativeCost` is matched by name — Jodah). A predefined token's reminder-text mana adds no identity (CR 111.10, `is_reminder_text_token`) |
| CR 903.4b a commander that chooses its color before the game (Clara Oswald, The Prismatic Piper, Faceless One) | ✅ *Piper/Faceless One since 2026-10-16 (`0128tsCy`)* | `StaticEffect::CommanderChoosesColorBeforeGame`; `format::chosen_commander_color`, `GameState::choose_commander_colors`. EDHREC averages of a chooser pair mix chosen colors — seat only those needing ≤ 1 extra color |
| Deck validation: singleton, identity subset, legal commander, ban list, CR 903.5e no sideboard | ✅ | `format.rs::validate_commander_deck` (CR 903.5a counts the commander), `commanders_may_pair`, `COMMANDER_BANNED` (Scryfall's list, 2026-09-18). ⚠ CR 903.5e is checked here, not in `FormatRules`: `validate_deck_refs` takes a flat card list and never sees a sideboard |
| CR 903.3 what a commander may be: legendary creature, Vehicle, or Spacecraft with a power/toughness box | ✅ | `format.rs::is_legal_commander` + `is_spacecraft_with_pt` (CR 721.2b). Shorikai / Parhelion II / The Seriema pass; The Eternity Elevator does not. `CommanderDeckError::IllegalCommander` |
| CR 903.5d a card with a basic land type is bounded by the commander's identity | ✅ *by folding* | `color_identity.rs::basic_land_identity` adds the type's color to the identity — matches Scryfall's `color_identity`, which the catalog audit ratchets against |
| The old mana-production restriction | n/a | removed from the CR. ⚠ CR 903.11 now holds "cards from outside the game cannot be brought into a Commander game" (903.11a); unreachable here — no sideboard, so `Effect::WishToHand` / `WishToLibrary` find nothing |
| "Any color in your commander's color identity" mana | ✅ | `ManaPayload::AnyColorInCommanderIdentity`, `GameState::commander_identity_colors`; Command Tower / Arcane Signet / Commander's Sphere / Path of Ancestry / Opal Palace. A colourless commander adds no mana, not {C} (rulings 2020-11-10). ⚠ A seat with no commander keeps "any colour" where CR 903.4f says the ability does nothing — deliberate, for the cube pool's fixing and the two-player golden traces; unreachable in a Commander game |
| A deck-construction choice printed on a card (Cryptic Spires' "circle two colors") | ⚠ *approximated since 2026-10-18 (`019Y6J4R`)* | `ManaPayload::CircledColors` reads the commander identity: exact for one or two colors, its first two (WUBRG) otherwise — no deck-building choice step (INCOMPLETE_CARDS row) |
| "If you control a commander" = any player's commander (CR 903.3) | ✅ *since 2026-09-23* | `Predicate::YouControlACommander` / `PlayerControlsACommander { who }` via `GameState::is_commander` (Akroma's / Jeska's Will, Crimson Honor Guard) |
| "If you control a commander as you cast this spell, you may choose both" (the Will cycle, 11 cards) | ✅ *cast-time since 2026-09-28* | `CardDefinition::modes_widen: ModesWiden::If(YouControlACommander)` on a `ChooseModesCast`, read in `cast_spell_spree` (CR 601.2b / 700.2a); the bot offers the both-mode cast. `ModesWiden::Kicked` is the Inscriptions' "if kicked, any number" (CR 702.33d). `cast_controlling_commander` → `Predicate::YouControlledACommanderAsCast` stays for "as you cast" riders. Tests `core_rules::commander_cards::cr_601_2b_*` |
| CR 601.2 "if [X] as you cast this spell" on any card | ✅ *since 2026-10-01* | `CardDefinition::as_cast_condition` (stamped by `actions.rs::stamp_as_cast_condition` in each cast body) → `CardData::cast_condition_held` → `Predicate::CastConditionHeld`. Orator of Ojutai, Foul-Tongue Invocation (`shortcut::dragon_as_cast`), Steer Clear, Faerie Fencing |
| A Background's "Commander creatures you own have [a static]" | ✅ *since 2026-10-04* | Triggers via `GrantTriggeredAbility` (Noble Heritage); statics via `StaticEffect::GrantStaticAbility` (`game/static_grant.rs`): baked into each matching permanent with `bake_grant` (not copiable, CR 707.2; gone as it leaves, CR 400.7), synced at each SBA check — so it follows a stolen commander (`cmdr_familiars::inspiring_leaders_anthem_follows_the_commander`). A grant reaches a permanent from the first SBA check after it matches, not mid-resolution |
| Commander to hand from the command zone (Command Beacon) | ✅ | `Effect::CommanderToHand`, `game/effects/commander.rs` |
| A melded commander is still the commander (Gisela's ruling, CR 903.3 + 701.37) | ✅ *since 2026-09-24* | `GameState::commander_card_of` maps the melded permanent to its commander component for `is_commander`, `is_own_commander_object` and commander damage. Leaving splits `meld_parts`; only the commander card goes home (`cmdr_gisela::a_melded_commander_is_still_the_commander`, `a_bounced_melded_commander_can_go_home`). A meld kept off the battlefield exiles both cards (CR 712.4; `cmdr_gisela::a_meld_kept_off_the_battlefield_exiles_both_cards`). Pod seat 33 plays it |
| A merged (mutated) commander is still the commander (CR 903.3c) | ✅ *since 2026-09-27* | `commander_card_of` / `is_own_commander_object` read `mutate_stack`; only the commander card goes home. `cmdr_otrimi::cr_903_3c_a_merged_commander_is_still_the_commander` |
| Casting a commander for mutate from the command zone (CR 903.8 + 702.140a) | ✅ *since 2026-09-27* | `actions.rs::cast_mutate` routes through `cast_from_command_zone`, tax on top; the bot offers it (`server/mutate.rs`, pods only). `cmdr_otrimi::cr_903_8_a_commander_mutates_from_the_command_zone_under_its_tax` |
| CR 106.6 mana provenance — "when that mana is spent to …" | ✅ | `SpendRestriction::{CommanderTypeScry, CommanderCastCounters}` + `is_rider()`; `game/effects/commander.rs::{note_commander_mana_riders, push_commander_mana_scry, spell_kind_for}`; Path of Ancestry / Opal Palace. CR 106.6a per-mana counts via `spent_restrictions` `(restriction, pips)`. The bot's `available_mana` counts floating rider mana (`ManaPool::rider_amount`) |
| CR 903.8 a commander tax paid in life (Liesa, Shroud of Dusk) | ✅ *since 2026-09-26* | `StaticEffect::CommanderTaxPaidInLife`, read by `actions.rs::commander_tax_for` on both command-zone cast paths; CR 119.4 gates the life. The bot skips a life tax that leaves it at 4 or less (`cr_903_8_liesa_pays_her_commander_tax_in_life`) |
| Every hand-pipeline cast variant from the command zone — kicker / multikicker / bestow / waterbend / convoke / improvise / delve (CR 903.8; convoke and delve apply after the total, CR 702.51a / 702.66a) | ✅ *since 2026-10-21 (`01KabMxS`)* | `game/helper_cast.rs::cast_hopped_from_command_zone` — hops a commander named by `GameAction::command_zone_hop_card` into the hand (`HopFrom::CommandZone`); `cast_spell_with_convoke` adds the tax. Also `CastSpellConvokeDelve`, graveyard cast permissions (`cast_with_helpers`), `CardDefinition::no_mana_spend` (Hogaak). Bot: `server/helper_cast.rs` and a kicked command-zone candidate. ⏳ the bot never builds a waterbend cast (any zone) |
| Casting a commander for an alternative cost from the command zone | ✅ | `actions.rs::cast_spell_alternative_from` over `AltCastZone`; CR 903.8's tax is pushed ahead of the reductions. `GameAction::CastFromCommandZone { alternative, pitch_card }` |
| CR 101.2 cast locks bind a commander cast (and every zone cast) | ✅ *since 2026-10-17 (`019Y6J4R`); a free cast a lock forbids is skipped per CR 608.2c since 2026-10-18 (`01MWhJbe`, `actions.rs::cast_card_for_free`)* | `game/cast_locks.rs` (`spell_name_locked`, `cast_lock_for`, `zone_cast_lock`): the hand path's name and player/board locks (Cease-Fire, Ranger-Captain, Grid Monitor, Meddling Mage, Llawan, Codie, Gaddock Teeg, Living Breakthrough) are asked by the command-zone, alternative-cost, flashback / escape / foretell / retrace / harmonize and may-play casts (`commander_cards::cr_101_2_*`). Open: half-card casts (adventure, omen, split, aftermath) and disturb check only their own type locks |
| CR 903.11 "outside the game" in Commander: a wish finds nothing (no sideboard) and never reaches into exile | ✅ *since 2026-10-18 (`019Y6J4R`)* | `Effect::WishToHand` is sideboard-only; Karn, the Great Creator's exile half is `WishToHandOrFaceUpExile`. `core_rules::commander_cards::cr_400_11_a_wish_does_not_reach_into_exile` |
| "Can be your commander" on a non-creature (planeswalker commanders) | ✅ | `CardDefinition::can_be_commander`, read by `validate_commander_deck`; all 22 Commander-legal "can be your commander" cards are implemented (2026-09-28). Piloted: `pod::decks::FREYALISE_MAIN` (`cr_903_3a_a_planeswalker_commander_seat_plays_a_pod_game`). Recast under the CR 903.8 tax like any other; its commander-damage tally stays zero (CR 903.10a counts combat damage) |

### Multiplayer foundation (CR 800-series)
| Feature | State | Where |
|---|---|---|
| CR 800.4a a departed seat is not a player | ✅ | `GameState::living_seats` beside `seats_in_turn_order_from` and `resolve_players`; `default_hostile_opponent` for the ranked "an opponent". `scripts/audit_seat_walks.py` gates two columns — a loop that hands a dead seat a *question* (votes, ballots, offers) and a walk that hands it a *role*. Both 0 |
| CR 506.2 the defending player as a filter | ✅ | `SelectionRequirement::ControlledByDefendingPlayer` + `defending_player_in_combat` (`effects/eval.rs`), which `PlayerRef::DefendingPlayer` also ends in. `scripts/audit_defending_player.py` is the ratchet (0/32). Off-attacker cases (a combat instant, an Aura/Equipment whose host attacks, another creature's attack trigger) use the host-then-single-defender fallbacks; a post-teardown trigger (Kusari-Gama) reads the damaged blocker's controller |
| N seats (2..N), turn rotation, APNAP ordering | ✅ | `game/mod.rs`, `multi_player_game`; tests in `core_rules/multiplayer.rs` and `cr_801.rs`. A declaration's attack triggers (defender-side and dispatcher) are stable-sorted into APNAP (`game/attack_apnap.rs`, `cr_603_3b_a_defenders_attack_trigger_resolves_first`) |
| Attack any opponent / their planeswalkers, per-attacker defender | ✅ | `game/combat.rs`. Defender-reading prohibitions (Island Sanctuary, Arboria, filtered can't-attack-you) are one walker, `attack_target_prohibited`; the bot re-aims a barred attacker (`pod_attack::reaim_prohibited_attacks`) and its defender pick (`bot.rs::attack_target_player`) falls back to the most hostile attackable seat (`pod_attack::attackable_or`, CR 508.1a) |
| "A player you're attacking" (CR 506.2) | ✅ *since 2026-10-16 (`01VDk2LU`)* | `PlayerRef::EachPlayerYouAreAttacking` — live seats the controller's creatures attack directly, APNAP order (Undercellar Sweep). A fan-out defender for created-attacking tokens resolves to the first such seat (CR 508.4) |
| CR 601.2c "for each opponent, … target X that player controls" | ✅ | `Effect::ForEachOpponentTarget` — one target per controller, capped at the opponent count, enforced by the auto-target picker, the cast path and resolution (the Primordials, Grasp of Fate, Omega, Tempted by the Oriq; `core_rules/per_opponent_targets.rs`). `ForEachPlayerTarget` (graveyards keyed by owner — Geth's Summons, Nils, Demonic Junker) reaches the slot-filter and hostility walkers too |
| CR 603.12 — a printed "When you do" payoff is a reflexive trigger (its own stack object, targeting after the gate) | ✅ *seated decks done 2026-10-26 (`01WvrAp6`); 7 open* | `Effect::{Reflexive, ReflexiveTrigger}`; `scripts/audit_when_you_do.py` ranks the unconverted by pod seats. With it: `ResolutionCarry.last_created_token` ("create a token. When you do, attach it" — Dáin, Forum Filibuster); a reflexive's hostile player slot aims at an opponent (`push_reflexive_trigger`); the bot declines a "may [cost]. When you do" whose payoff has no legal target (`bot.rs::reflexive_payoff_untargetable`); a modal payoff picks its mode as it pushes (CR 603.3c; Atraxa's Skitterfang). Open: Guide of Souls / Savai / Awaken the Honored Dead / Tester of the Tangential (cube — frozen), Miasma Demon |
| CR 603.7 — a reflexive trigger's "up to that many" | ✅ *2026-10-03* | `Effect::ReflexiveTrigger` under `WithX` caps its picks at the carried X (`slot_past_x_cap` over `CapTargetsAtX`): Loamcrafter Faun, Glissa's Retriever |
| CR 603.2c "whenever one or more … deal combat damage to a player" | ✅ | `EventSpec::once_per_batch` plus `combat.rs::BatchSubject` — one fire per damaged player. Ratchet `catalog_registration::every_batched_damage_trigger_fires_once_a_batch` (accepted spellings: the flag, `once_per_turn`, or a `FromYourGraveyard` scope). Malcolm, Keen-Eyed Navigator counts exactly this |
| CR 603.2c "whenever one or more" on the other event kinds (exile, discard, sacrifice, tap, attack a player / a planeswalker) | ✅ | `EventSpec::once_per_batch` on the dispatcher, the defender-side attack walk and the `OpponentOfYoursAttacked` walk (Jolene); `batch_counts_subjects` for "for each card discarded this way" (Captain Howler). `scripts/audit_one_or_more.py --gate` (0) |
| CR 400.7 a card leaving a graveyard is always reported | ✅ | `note_left_graveyard` / `note_exiled_from_graveyard` / `note_returned_to_hand_from_graveyard` at every graveyard exit (hop casts, delve and exile costs, shuffles, the commander's return). A card exiled from a library too: `EventKind::CardExiledFrom(exile_from::{BATTLEFIELD, GRAVEYARD, LIBRARY})`, `note_exiled_from_library` (Laelia) |
| CR 508.4 / 802.2 a creature put onto the battlefield attacking may attack any opponent; CR 508.2 "at the beginning of your next declare attackers step" | ✅ *since 2026-10-19 (`01GwrDKP`)* | `game/enter_attacking.rs::join_combat_attacking_chosen` (attacked seats lead, then every opponent in turn order); `game/declare_attackers_delayed.rs`. Noctis (`recent_b::cmdr_01gw_a::noctis_warp_strikes_back_into_combat`) |
| CR 506.4 / 506.4c an attacked planeswalker that changes control leaves combat | ✅ *since 2026-10-18 (`015XaGkQ`)* | `GameState::change_control` re-aims its attackers at a ghost id with the CR 508.5 defender recorded; `core_rules::multiplayer::cr_506_4_an_attacked_planeswalker_that_changes_control_leaves_combat` |
| CR 104.4b a drain loop against a seat that cannot lose is a draw | ✅ *since 2026-10-18 (`015XaGkQ`)* | `stack.rs::resolve_top_of_stack` counts it as growth for `growth_loop.rs`; `cr_recent113::cr_104_4b_a_life_drain_loop_against_a_seat_that_cannot_lose_is_a_draw` |
| CR 732.2a a finite mandatory loop that only moves life is shortcut, not run to the action cap | ✅ *since 2026-10-25 (`claude/modern_decks`, this run)* | `game/drain_shortcut.rs`: past the exact watch's period a life-free digest arms; a repeating cycle is applied k times at once, one short of the first that could take a living seat to 0. `cr_recent113::cr_732_2a_*` |
| CR 511.3 combatants stay in combat until the end of combat step ends | ✅ *since 2026-09-23* | `combat.rs::remove_all_from_combat`, run by `advance_step` leaving `EndCombat`; `combat_damage_dealt()` keeps a repeated `resolve_combat` a no-op |
| CR 103.5c a multiplayer game's first mulligan is free | ✅ *since 2026-09-27* | `GameState::mulligan_cards_owed`, read by the Keep path and the bot. CR 103.8c is `skip_first_draw`. `core_rules::multiplayer::cr_103_5c_first_multiplayer_mulligan_is_free` |
| CR 702.16j "protection from each of your opponents" on a permanent | ✅ *since 2026-10-21 (`01Jywzky`)* | `Keyword::ProtectionFromEachOpponent` (`ProtectionKind::EachOpponent`) at the targeting, ability-source, blocking and damage sites. Figure of Fable; `recent_b::cmdr_01jy_a::cr_702_16j_*` |
| CR 502.3 an emblem's per-opponent untap cap | ✅ *since 2026-10-21 (`01Jywzky`)* | `StaticEffect::OpponentsMaxUntapsPerStep`, read from the owner's emblems by the untap step. Dovin Baan's −7 |
| CR 800.1 — how many players | ✅ *2..=64* | `game::MAX_SEATS` (per-seat masks are `u64`), asserted in `GameState::new`; `bot_ladder` clamps `--seats`/`--pod-decks` (`core_rules::multiplayer::cr_800_1_a_game_holds_at_most_max_seats_players`) |
| CR 508.1d "during [player]'s next turn, creatures they control attack [this] if able" | ✅ *since 2026-09-24* | `Effect::LureCreaturesToSourceNextTurn` + `PlayerCold.attack_lure`, `game/attack_lure.rs`; the bot's `restore_forced_attackers` honours it (Gideon Jura; `recent_b::cmdr_isperia`) |
| CR 508.1 / 508.1d player-level next-turn requirements (Rowan Kenrith, The Second Doctor); "if that creature had to attack this combat" (Firkraag) | ✅ *since 2026-10-04 (`016kc94f`)* | `Player::attacks_if_able_next_turn` + `side_attacks_if_able` (`game/attack_lure.rs`); `Player::next_turn_attack_bans` (`game/attack_ban.rs`, read by `player_cant_be_attacked_at_all`, the declare check and `permanent_cant_be_attacked`); `ColdState::had_to_attack` + `R::HadToAttackThisCombat`. Tests `cr_508_1d_rowan_kenrith_*`, `the_second_doctor_bars_*`, `cr_508_1d_firkraag_reads_*` |
| CR 508.1 "can't attack you or planeswalkers you control this turn" and "each opponent who attacked you … can't cast spells" | ✅ *since 2026-10-21 (`01KPxZ3H`)* | `Effect::CantAttackPlayerOrWalkersThisTurn` (`cant_attack_player_this_turn` + `ColdState::cant_attack_walkers_this_turn`); `StaticEffect::OpponentsWhoAttackedYouCantCast` over `PlayerCold::attacked_walkers_of_this_turn`. Sandswirl Wanderglyph (`recent_b::cmdr_01kp_a::cr_508_1_sandswirl_wanderglyph_*`) |
| CR 707.10c "for each other opponent, copy that spell" aimed at that player or their permanent | ✅ *since 2026-10-21 (`01KPxZ3H`)* | `Effect::CopySpellForEachOtherOpponent` (`copy_redirect.rs`); `R::SpellTargetsOnlyOneOpponentOrTheirs` gates the watcher; the pod bot arms it ahead of a hostile spell (`server/next_spell_copy.rs`). Zevlor |
| CR 717.2 a pod seat whose deck opens Attractions brings an Attraction deck | ✅ *since 2026-10-21 (`01KPxZ3H`)* | `pod::POD_ATTRACTIONS` (CR 717.2a's ten or more), seated by `build_pod_template_from` when any card reaches `Effect::OpenAnAttraction`, shuffled per game; the card-conservation audit counts it and the junkyard (CR 717.6). `cr_717_2_a_seat_that_opens_attractions_brings_an_attraction_deck` |
| CR 106.3 "{S} of any of that spell's colors was spent" — snow mana by colour | ✅ *since 2026-10-21 (`01KPxZ3H`)* | `ManaPool::snow_by` (per-bucket snow provenance), settled snow-first at payment (CR 601.2h) into `CardData::cast_snow_colors`; `Predicate::SnowOfItsColorSpentToCast`. Boreal Outrider. Tests `snow_colours_spent_follow_the_bucket_that_paid`, `cr_106_3_boreal_outrider_counts_snow_of_the_spells_colour` |
| CR 500.8 "at the beginning of that combat" — the combat an effect banked | ✅ *since 2026-10-04 (`016kc94f`)* | `game/added_combat.rs` (`DelayedKind::CombatNumberThisTurn`, `Effect::AtTheAddedCombat`): Moraug, Last Night Together. Tests `moraug_untaps_only_*`, `cr_500_8_last_night_together_*` |
| CR 611.3a a static's conditional play window over "cards exiled with this" ends with its source | ✅ *since 2026-10-04 (`016kc94f`)* | `MayPlayDuration::{HolderTurnsAfterNontokenSacrifice, HolderTurnsAfterOpponentLostLife}`; `end_source_bound_grants` (`effects/life_loss_grants.rs`). Evendo Brushrazer, Theater of Horrors |
| CR 702.11d-e / 702.14a / 702.16 "has [keyword] as long as a card … has it" borrows the donor's variant | ✅ *since 2026-10-04 (`016kc94f`)* | `StaticEffect::GainKeywordsOfCards { from: KeywordDonors }` + `Keyword::answers_listed`: Death-Mask Duplicant, Cairn Wanderer, Rayami |
| CR 205.3 / 613.1d / 613.3 "is every creature type" is a layer-4 type effect | ✅ *since 2026-10-05 (`01S6ztPx`)* | `StaticEffect::EveryCreatureType`, `Effect::GainAllCreatureTypes`, `Modification::EveryCreatureType` (`game/layers.rs`): a computed changeling after layer-6 removals, overwritten by a later layer-4 set; a printed changeling's CDA survives "loses all abilities" (`cmdr_omo::every_creature_type_survives_losing_all_abilities`, `cmdr_morophon::amoeboid_changeling_type_effects_follow_timestamps`) |
| CR 608.2b a zone-named card target that left its zone is unaffected; a modal mode's cross-slot atoms count from its own first slot | ✅ *since 2026-10-05 (`01S6ztPx`)* | `ApplyToTargets` re-checks off-battlefield card targets (`SelectionRequirement::without_cross_slot`); `spree_targets::chosen_mode_first_slot` + `shift_target_slots`. Loaming Shaman, Quandrix Command (`stx::part_01::quandrix_command_*`) |
| Multiplayer non-target player choices: "choose an opponent" led by a stat, "choose a player at random", "that player or a planeswalker that player controls", protection from each opponent | ✅ *since 2026-10-05 (`01S6ztPx`)* | `Effect::ChooseOpponentLedBy` (Sandstone Oracle), `Effect::WithRandomPlayer` + `ChooseOneAmong` (Wildfire Devils), `Effect::DealDamageToPlayerOrPlaneswalker` (Vial Smasher, Curse of the Pierced Heart), `GainProtectionFromPlayer` over `EachOpponent` (Cliffside Rescuer, CR 702.16j). Each is a seat ballot (`effects/choose_player.rs`) |
| CR 702.16b/e "you have protection from [quality]" on a player | ✅ *since 2026-10-13 (`019gJJoX`)* | `StaticEffect::ControllerProtectionFromMatching`: damage prevented on the Sphere-of-Purity lane (`movement.rs`), matching sources can't target (`actions.rs::player_protected_from_source`). `PlayerRef::EachWithFewestControlled`. Gor Muldrak |
| CR 723 controlling another player: a whole turn (Mindslaver, Emrakul, Cruel Entertainment) and one combat phase (CR 723.2, Secret of Bloodbending); CR 723.5 in bot pods | ✅ *since 2026-10-11 (`01Lnhgw3`)* | `GameState::controlled_by` / `pending_player_control` / `pending_combat_control` (`game/player_control.rs`); server routing via `acting_seat_for`; pods play the controller's policy (`pod/controlled.rs`). Tests `cr_723_*` (`core_rules/cr_recent43.rs`), `cr_723_5_controller_declines_*` |
| CR 601.2a a spell cast from the graveyard / library top through a hop into the hand is not "cast from your hand"; a graveyard creature-cast permission | ✅ *since 2026-10-04 (`016kc94f`)* | `actions.rs` — the four cast finalizers exclude the hopped card from `cast_from_hand`; `Player::graveyard_creature_cast_turn` + `graveyard_creature_cast_available` (Chainer). Tests `cr_601_2a_a_graveyard_cast_is_not_a_cast_from_hand`, `chainer_hastes_*` |
| CR 712.4 / 707 a transformed permanent that became a copy leaves as its front face; CR 712.14a "return it transformed" when it isn't a DFC any more stays put | ✅ *since 2026-10-04 (`016kc94f`)* | `stack.rs` leave funnel order; `ExileSelfReturnTransformed` total. `cr_712_14a_olag_copying_chandra_*` |
| CR 601.2c / 115.3 an unfillable optional trigger slot stays a hole; two "any number of target" groups side by side | ✅ *since 2026-10-04 (`016kc94f`)* | `auto_extra_distinct_slot_targets` (`game/mod.rs`) pushes `TARGET_HOLE`s and resets its used-set when a `SlotGroups` filter changes; `declines_gift_pick`. Filigree Vector (`cr_601_2c_filigree_vector_*`, `filigree_vector_targets_its_counters`) |
| CR 611.3a / 205.1a a static grants a card type to spells you control and cards you own off the battlefield | ✅ *since 2026-10-04 (`016kc94f`)* | `StaticEffect::OwnedCardsOffBattlefieldHaveCardType`; `off_battlefield_card_type_adds` / `off_battlefield_card_matches` (`game/creature_type.rs`). Biotransference (`cr_611_3a_biotransference_*`, `cr_611_3a_anrakyr_*`) |
| CR 506.3 "attacking you" / "attacks you" is one seat's attackers | ✅ *since 2026-09-27* | `SelectionRequirement::{IsAttackingYou, IsAttackingYouOrYourPlaneswalker}`; `scripts/audit_attacking_you.py --gate` (0 open, 37 cards). Trigger scopes: `EventScope::ControllerAttackedByOpponent` ("you or a planeswalker you control"), `ControllerAttackedDirectlyByOpponent` (bare "attacks you"), `ControllerPlaneswalkerAttackedByOpponent`; `scripts/audit_attack_scope.py --gate` |
| CR 503.1 / 513.1 "each upkeep / end step" is every seat's step; "another creature" is anyone's | ✅ *since 2026-09-27* | For `StepBegins`, `YourControl` / `ActivePlayer` / `SelfSource` all mean the controller's turn, so a printed "each upkeep" must not use them (Tendershoot Dryad, Séance Board, Joined Researchers, Manaform Hellkite, Instill Furor fixed); anyone's-creature enters / dies / attacks triggers must not be `AnotherOfYours` (Reaper of the Wilds, Wirewood Hivemaster, Najeela, Skyboon Evangelist). `scripts/audit_step_scope.py --gate` over `dump_cards --grep StepBegins` (0 rows) |
| CR 508.1 "whenever an opponent attacks" (`YouAttack` / `OpponentControl`) | ✅ *since 2026-09-27* | `combat.rs` — the attack walk admits opponents' observers, not only the attacker's own and `AnyPlayer` (Tahngarth, First Mate); `JoinCombatAttacking` joins an attack its new controller already makes |
| CR 506.2 each attacker's own defending player | ✅ *since 2026-10-17 (`yl8zjb`)* | `PlayerRef::DefendingPlayerOf(selector)` — `DefendingPlayer` reads the source's defender, so a fan-out over attackers aimed at different seats needs the per-subject form. Jurin (`jurin_pumps_each_attacker_by_its_own_defender`) |
| CR 509.1a — only a defending player declares blocks | ✅ | `combat.rs::may_declare_blocks` |
| CR 508.1 / 614.1a "you can't be attacked except by …" until your next turn, one seat of N | ✅ *since 2026-10-14 (`01PxmVAG`)* | `Player.attackable_only_by_until_next_turn`, stamped by Island Sanctuary's draw replacement (`game/draw_skip_shield.rs`), enforced per player-targeted attack in `declare_attackers`, cleared as that seat's turn begins; the pod spill planner leaves a shielded seat alone (`server/pod_attack.rs`). `recent_b::cmdr_rank6k_g` |
| CR 508.1d requirements naming a defender obey the most they can — a tie is a free pick | ✅ *since 2026-10-06* | `game/attack_requirements.rs` (engine and bot share the score); `Effect::MustAttackPlayerThisCombat` (Ruhan); `core_rules::cr_recent112`, `recent_b::cmdr_zedruu::ruhan_*` |
| CR 508.1d / 509.1c creature-level requirements against an attacker or blocker cap: the most obeyed | ✅ *since 2026-10-07* | `combat.rs::attack_cap_leaves_no_room` / `block_cap_shuts_out` — a bound creature a full Silent Arbiter / Crawlspace cap shuts out is excused; the bot trims to the cap keeping bound attackers. A goaded creature that can't attack alone stays home only when no other creature can join it (the bot adds a partner); Okk likewise; Magnetic Web groups likewise. `cr_recent115` |
| CR 118.9a a printed alternative cost or a granted one | ✅ *since 2026-10-07* | `GameAction::CastSpellGrantedAlternative` + `granted_alternative_cost_in` (Henzie's blitz); CR 702.175 printed and granted offspring both paid (`GRANTED_OFFSPRING_OPTION`, Zinnia) |
| CR 506.4 a combatant that stops being a creature leaves combat | ✅ *since 2026-10-06* | `stack.rs::remove_noncreature_combatants` in the sweep (`cr_recent44::cr_506_4_an_attacker_that_stops_being_a_creature_leaves_combat`; a debug-pod invariant). A blocker killed paying its own block tax leaves combat with its attacker still blocked; an attacker sacrificed to an attack tax is dropped (`cr_recent117`) |
| CR 608.2d a resolution-time "choose a card / permanent" is the chooser's | ✅ *since 2026-10-02* | `Effect::ChooseOneAmong` / `ChooseSomeAmong` (`effects/choose_one.rs`; `shortcut::choose_one_then` / `choose_some_then`); a forced pick asks `min = 1`; CR 701.21a `MaySacrifice` asks which. An opponent choosing a gain for the controller hands over the least (Neyam); per-player picks in seat loops ask each seat (`ask_seat_cards_logged` / `ask_seat_option`); `MoveAnyNumberOfCounters`; `DistributeCountersAmong`; a trigger's "up to X" binds X from its event (CR 603.3d). Every parked ask is answered by the seat it prompted (`PendingEffectState::Seated`, gated by `structural_audit::a_parked_ask_names_the_seat_it_prompted`). Left: INCOMPLETE_CARDS' "Resolution-time picks" section |
| CR 508.4 a creature put onto the battlefield attacking, no defender named: its controller chooses | ✅ *since 2026-10-02* | `Effect::JoinCombatAttackingChosen` (`game/enter_attacking.rs`) on every catalog "tapped and attacking" that names no defender. `CreateTokenAttacking` / Mobilize / Myriad keep their own defender logic |
| CR 406.3 "exile … face down; you may look at it" | ✅ *since 2026-10-02* | `Effect::ExileFaceDown { body }`; `view.rs::exile_entry`'s looker; cleared in `finalize_cast` and `movement.rs::move_card_to`'s exile exit. Intet, Kheru Mind-Eater, Predators' Hour, Gonti and others |
| CR 506.3 / 508.1b "attacks a player / one of your opponents" is not a planeswalker attack | ✅ *since 2026-09-27* | `SelectionRequirement::IsAttackingOpponentPlayer` (the player) vs `IsAttackingAnOpponent` (player or their planeswalker — only Gahiji and Roar of Resistance). `Predicate::PlayerAttackedByMatching` names "those players" (Zurzoth); `Effect::CantBeBlockedByPlayer` (The Black Gate) |
| CR 506.2 "attacks you" is a different count from "attacks" | ✅ | `Predicate::AttackedDefenderWithCountAtLeast { who, defender, at_least, include_planeswalkers }` (Mangara, the Diplomat; Trouble in Pairs); `include_planeswalkers` is a printed difference between those two cards. ⚠ Reads `GameState.attacking`, filled during the declaration, so on a `ControllerAttackedByOpponent` trigger the gate goes inside the effect as an `Effect::If`, not in the `EventSpec` filter |
| CR 603.2 a defender-side attack trigger's own condition | ✅ *since 2026-09-19* | `combat.rs` — the `ControllerAttackedByOpponent` walk reads `t.event.filter` (Reveille Squad). ENGINE_BACKLOG's fiftieth find |
| CR 603.2 a listener that leaves in the same resolution still saw what happened before it left | ✅ *since 2026-10-07* | `game/departed_listeners.rs`: enters / life / draws / discards / counter listeners gone by dispatch fire for each event between their entry and departure (`cr_603_2_a_listener_the_resolution_removes_saw_the_entries_before`, `cr_recent113`); "whenever a creature dies" too, incl. a noncreature listener destroyed with the creatures (CR 603.10a; `simultaneous_deaths.rs`, `cr_recent116`); once-each-turn listeners stay within budget (`cr_recent110::cr_603_3d_*`); cast triggers need nothing (`cr_recent118::cr_603_2_a_cast_listener_*`). Sequential deaths: `468917403` (ENGINE_BACKLOG) |
| CR 614 "if an opponent would begin an extra turn, they skip it instead" | ✅ | `StaticEffect::OpponentsSkipExtraTurns`, read where `Player.extra_turns` is consumed (`stack.rs::end_turn`). The charge is spent even though the turn is skipped (else one Time Warp re-offers forever). Trouble in Pairs |
| CR 121.2a a draw doubler with a printed exception | ✅ | `StaticEffect::ControllerDrawsDoubledExceptFirstEachDrawStep` (Alhammarret's Archive, Teferi's Ageless Insight), read off `Player::cards_drawn_this_step`. Distinct from `ControllerDrawsDoubled` (Thought Reflection) |
| "For each colour among permanents you control, add one mana of that colour" (Vivid) | ✅ | `ManaPayload::OneOfEachColorAmongYourPermanents` — Bloom Tender, Faeburrow Elder. ⚠ Distinct from `AnyColorAmongYourPermanents` (Meteor Crater), which adds one mana chosen from the same set; a mono-colour test cannot separate them. Shared walk: `GameState::colors_among_your_permanents` |
| CR 400.7 — a blink is a new object | 🟡 *in every respect a card can read* | `Effect::ExileAndReturnToOwner` moves the same instance out and back: state resets, but the `CardId` survives, so anything keyed on the id could still match where the rules say it lost track (unobservable on the shipped friendly self-blinks; ENGINE_BACKLOG carries it). Closed around it: a captured delayed return (`Effect::DelayUntilWithCapture`) checks `battlefield_timestamp` via `Predicate::TargetIsCapturedObject` (`cr_recent104::cr_400_7_a_stale_delayed_return_*`); `Duration::Permanent` grants and "loses [keyword]" end with the object (`bake_grant`); timed trigger grants carry the stamp (`GrantedTrigger::stamp`, `cr_recent106::cr_400_7_a_timed_trigger_grant_ends_with_the_object`); leaving exile drops the exiler's link (`exile_return.rs::take_from_exile`, `cr_recent118`), as does a card cast from exile or leaving the battlefield (`cr_recent118::cr_400_7_a_card_cast_from_exile_*`, `cmdr_prosper::cr_400_7_a_card_cast_from_the_pile_*`); every leave path sheds the whole old object (`leave_battlefield_state`), a delayed return from exile runs only while the card is exiled (`linked_return::exiled_card_only`); trigger and spell targets are re-checked per slot against their zone (`StackItem::Trigger::bf_slots`, `CardInstance::cast_bf_slots`, CR 608.2b; an illegal slot is a hole, `stack_ability.rs::remark_stack_targets`) |
| CR 614.2 a damage multiplier that is not a power of two | ✅ | `StaticEffect::MultiplyDamageFromYourSources { factor }` — Fiery Emancipation / City on Fire, ×3, on its own accumulator beside the doublings (two is ×9). ⚠ A new damage-scaling static must be listed in `static_effect_scales_damage`, or the `scale_damage_to` presence gate skips it silently (its `debug_assert_eq!` catches that in tests) |
| CR 702.16 protection from a set the game state owns | ✅ | `Keyword::ProtectionFromColorsOutsideCommanderIdentity` — Commander's Plate. Four gates (the `ProtectionKind` damage/equip/aura funnel, ability targeting, cast-time spell gate, blocking), each tested, all through `GameState::outside_commander_identity`; a seat with no commander has no protection (CR 903.4f) |
| CR 800.4a a player leaving: objects, stack items, control effects, the command zone, combat, a pending ask | ✅ | `stack.rs::objects_leave_with_player`, driven for every seat that is out by `Player::left_game`. `pod::play_pod_game` takes `CRAB_POD_CONCEDE=<n>` — a seat concedes (CR 104.3a) at a random point, n per 10,000 actions (`cr_800_4a_pods_survive_concessions_at_arbitrary_points`). Their cards in other seats' zones and phased-out permanents leave too; the sweep repeats once per departure (CR 704.3); a debug pod invariant asserts nothing outlives its departed owner or controller (`pod/mod.rs`). A spell they control but don't own is exiled; a per-seat ask on another seat's resolution runs on past them (`departed.rs::resume_past_departed_asker`); a permanent that entered under them is exiled and a stolen one reverts (`CardData::pre_effect_controller`). Tests `multiplayer::cr_800_4a_a_spell_the_departed_*`, `cr_800_4a_a_departed_voters_*`, `cr_800_4a_a_permanent_that_entered_*`, `cr_800_4a_a_steal_ends_and_returns_*` |
| CR 800.4a (closing) / 800.4j priority never rests on a seat that left | ✅ | `mod.rs::priority_recipient`, read by `give_priority_to_active` and the untap-entry seed; the turn itself continues (`active_player_idx` untouched, per 800.4j) |
| CR 800.4a — a payer who leaves mid-payment takes their spell off the stack | ✅ *since 2026-09-28* | `effects/mod.rs::stack_pos_of` re-finds the countered item after an "unless [they] pay" cost; auto-tap refuses a source that would kill its payer at ≤ 3 life (`actions.rs::mana_source_self_harm`). Tests `core_rules::multiplayer::*_payer_leaving_mid_payment_*` |
| CR 800.4a + 610.3 — a departed player's "until this leaves" exiles end; their permanents' leaving triggers others' leave abilities | ✅ *since 2026-09-28* | `stack.rs::objects_leave_with_player` runs `note_left_without_dying` (Twilight Drover), then `return_linked_exiles` / `phase_in_held_by` (Cast Out's ruling). An old-style "When this leaves the battlefield, return" trigger: `StaticEffect::ExileReturnIsLeaveTrigger` (21 cards) keeps those cards exiled. `concede` dispatches its events |
| CR 610.3 a source that exiles itself "until this leaves" returns at once | ✅ *since 2026-10-08* | `ExileUntilSourceLeaves` / `EachPlayerExilesChosenUntilSourceLeaves` run `return_linked_exiles` when the source left in the same move; `cr_recent118::cr_610_3_a_source_that_exiles_itself_*` |
| CR 613.7d every battlefield entry is timestamped as it enters | ✅ *since 2026-10-08* | resolving spell, tokens, a played land, persist / undying, opening-hand starts; `cr_recent118::cr_613_7d_*`, `cr_704_5m_an_aura_whose_host_became_a_land_is_shed` |
| CR 603.2 "whenever you or a permanent you control becomes the target" | ✅ *since 2026-10-25* | `GameEvent::PlayerBecameTarget` from every targeting site, heard only by `EventScope::YouOrYourPermanentTargetedByOpponent` (Leovold, Rayne). ⚠ `BecameTarget` with `OpponentControl` / `YourControl` means *this* permanent; use a `Your*Targeted*` scope for anything wider. Tests `cmdr_seated_gaps::{leovold,rayne}_*` |
| CR 115.1 / 113.7 an ability on the stack is a target of its own | ✅ *since 2026-10-03* | `game/stack_ability.rs` — `StackItem::Trigger::ability_id` (`ABILITY_ID_BASE`), stamped by `push_stack`; `evaluate_requirement_on_ability`. Copy / counter / retarget resolvers (Strionic, Tawnos, Stifle, Reroute, Emissary of Grudges) take the id; a copy is a new id (CR 707.10). `cr_recent110::cr_115_1_*` |
| CR 603.3d a trigger's target filter reads its own numbers as it goes on the stack | ✅ *since 2026-10-03* | `game/trigger_time.rs` — `WithX` settled at trigger time; `OtherThanTriggerSubject`. Puca's Covenant |
| CR 603.3d a prompting seat names the targets of every trigger; CR 608.2c chosen modes run in printed order | ✅ *since 2026-10-06* | `game/trigger_slots.rs` (`trigger_asks_targets` / `queue_trigger_asking`, `prompt_trigger_backlog`) for every trigger kind; modal per-mode slots (`next_modal_slot`); `ChooseN` sorts its run (`core_rules::cr_recent111`). A combat-damage "any target" is picked, and later slots filled (Sword of Fire and Ice, Niv-Mizzet, Guildpact). A targeted `Effect::Reflexive` goes on the stack for every seat (`push_reflexive_trigger`) carrying the gating event's scratch (CR 603.7c `ScratchBinding::Resolution`) and subject / amount (`cr_recent111::cr_603_7*`); untargeted "that player"-bound payoffs too (Nihiloor, Dokuchi Silencer); only a sourceless effect resolves one inline |
| CR 603.3d "you may … Do this only once each turn" is spent by the take, not the trigger | ✅ *since 2026-10-29 (`01WRuXc6Fk`)* | `shortcut::once_each_turn_on_take` / `Predicate::SourceDoneThisTurn` + `Effect::MarkDoneThisTurn`; ratchet `catalog_registration::every_optional_once_each_turn_is_spent_by_the_take` |
| CR 601.2c "target player" + "X target cards" side by side | ✅ *since 2026-10-03* | `Effect::SlotGroups` — members own consecutive target slots; `Effect::slot_group_owner` translates every slot walker. Suffer the Past. ⚠ Targets still can't have holes: two "any number" groups in a row (Filigree Vector) stay open |
| CR 702.62a Suspend X | ✅ *since 2026-10-03* | `Keyword::Suspend(0, cost-with-X)` + `GameAction::SuspendX`; bots size X (`server/suspend.rs::pick_suspend_x`). Aeon Chronicler. The client has no SuspendX control yet |
| A trigger an object is granted is that object's ability (CR 113.10) | ✅ *since 2026-10-03* | `Effect::AsSubjectsAbility` — re-homed in `push_pending_trigger`. Grothama |
| Aboleth Spawn's trigger sits on the ability it copies | ✅ *since 2026-10-03* | `game/probing_telepathy.rs` (2022-06-10 rulings) |
| An Aura a plain "return it to the battlefield" puts there enters attached (CR 303.4f / 303.4i) | ✅ *since 2026-10-03* | `aura_hosts_for_move` in `game/effects/attach_from_zone.rs`; nothing to enchant, it stays put. Rise to Glory, Brotherhood Outcast |
| CR 510.2 "whenever ~ deals combat damage" (recipient-agnostic) is one event per dealer per damage step | ✅ *since 2026-10-03* | `BatchSubject::Dealer` in `game/combat.rs`; noncombat too — a damage-only sentence is one batch (`effects::damage_batch`, `BatchSubject::NoncombatDealer`; `cr_603_2c_a_noncombat_sweep_fires_deals_damage_once`) |
| CR 701.30 clash: "whenever you win a clash", "clash with defending player", APNAP decisions | ✅ *since 2026-10-03* | `GameEvent::ClashWon` / `EventKind::WonContest { clash }` (a payload on the coin-flip bit — the kind mask is full); `ClashWithOpponent { with }`. Marvo |
| CR 400.7 — a permanent grant ends with the object | ✅ *since 2026-09-28* | `CardInstance::bake_grant` / `revert_baked_grants` (card.rs), restored in `movement.rs::on_left_battlefield`. `core_rules::cr_recent106::cr_400_7_permanent_grants_end_when_the_object_leaves` |
| CR 800.4b no token, and no control change, for a departed player | ✅ | `mod.rs::mint_token_with_counters`, `mod.rs::change_control` |
| CR 800.4d a departed player's triggered abilities aren't put on the stack | ✅ | `mod.rs::push_pending_trigger` (the one funnel); `objects_leave_with_player` drops their `delayed_triggers` |
| CR 800.4e no combat damage is assigned to a seat that has left | ✅ *by 800.4a* | `stack.rs::objects_leave_with_player` removes every attacker whose defending player left. `cr_800_4e_no_combat_damage_is_assigned_to_a_seat_that_left_mid_combat` |
| CR 800.4k a departed seat's turn does not begin | ✅ | `mod.rs::next_alive_seat`, read by `stack.rs::end_turn`. `cr_800_4k_a_departed_seats_turn_does_not_begin` |
| CR 800.4c a control effect ending with the default controller gone | ✅ | `mod.rs::revert_temporary_control` — the reversion site, since in the three-deep shape (A owns X, B takes it, C takes it until end of turn, B leaves) the departing seat controls nothing; X is exiled. "No other effect giving control" is asked of the whole registry, so it doesn't depend on registration order |
| CR 800.4m "until that player's next turn" ends when that turn would have begun | ✅ | `mod.rs::departed_seats_skipped_into_this_turn` → the Untap arm's expiry in `stack.rs`: `UntilYourNextTurn`, `UntilYourNextUpkeep`, CR 615 damage locks, `UntilEndOfYourNextTurn` layer effects, temporary copies and seat-keyed registries (Single Combat, spell taxes, Academic Probation, Lightning/Jeska damage windows, Teyo). `cr_800_4m_a_departed_players_cast_lock_lifts` |
| CR 800.4a — a seat leaving between a suspended multi-seat ask and its replay | ✅ *2026-10-04* | `game/answer_log.rs`: each `resolution_answer_log` entry carries its seat; `ask_seat_*` skips a departed seat's entries on replay. Arms that act between asks ask first; `logged_gate` logs a board-dependent skip. Tests `cr_800_4a_replay_skips_a_departed_seats_answer`, `cr_800_4a_agitator_ant_survives_a_seat_leaving_between_asks` |
| CR 408.1 — a commander in the command zone is its owner's object | ✅ *2026-10-04* | `controller = owner` at the three command-zone pushes (`commander_return.rs::return_commander`, the 903.9b redirect, `ZoneDest::Command`); eminence readers key on it. `cr_408_1_a_stolen_commander_goes_home_under_its_owner` |
| CR 702.49a ninjutsu / commander ninjutsu window | ✅ *2026-10-04* | `mod.rs::ninjutsu` — declare blockers through end of combat. Bots still only try it in declare blockers. `cr_702_49a_ninjutsu_after_first_strike_damage_and_at_end_of_combat` |
| CR 810.5 / 810.9 — team games: "you" is one player, "opponent" the other team | ✅ *2026-10-04* | `events.rs` scope matcher compares the seat, not the team (Pridemate / Sanguine Bond rulings); spell taxes, Leyline of the Void, Sigarda's shield use `same_team`. Tests `cr_810_9_a_teammates_life_gain_doesnt_trigger_yours`, `cr_102_3_a_teammates_tax_and_leyline_skip_you` |
| CR 800.4f/g/h a choice owed by a departed player | ✅ | `game/departed.rs` — `route_ask` at the six `ask_seat_*` helpers. 800.4f: a cost is not paid and nobody is asked (`OptionalKind::is_cost`). 800.4g: any other choice is re-seated by the object's controller — another opponent (`default_hostile_opponent`) where the chooser was one. 800.4h needs no arm (rule-required asks name the seat's own cards, already removed). ⚠ An ask already pending when its seat leaves is still dropped, not re-seated (needs the legal set re-derived) — rare now that no ask is posed to a departed seat |
| CR 800.4a — a seat that has left is not one of "each player" | ✅ | `mod.rs::seats_in_turn_order_from` (+ `next_alive_seat`) and `resolve_players(&PlayerRef::EachPlayer, ctx)` (CR 101.4 order and the liveness filter); fan-outs into `apnap_sort` take `living_seats()`. Ratchet `scripts/audit_seat_walks.py` (0) — a walk is a finding when its body asks or accumulates one entry per seat. A departed seat's emblems leave with it (CR 114.2). Tests `cr_800_4a_apnap_fan_outs_skip_a_departed_seat`, `cr_800_4a_your_next_turn_skips_a_departed_seats_turn`. ENGINE_BACKLOG's forty-seventh find |
| CR 800.4 — no ask is ever posed to a seat that has left | ✅ | `departed.rs::seat_prompts` (`wants_ui && is_alive`), the one predicate behind `seat_suspends` and every suspend site; audited on every pod game by a `debug_assert!` in `pod::play_one_pod_game` |
| Which opponent an open choice aims at, at N > 2 | ✅ | `mod.rs::default_hostile_opponent` (`hostile_opponent_score`: CR 903.10a commander race, then lowest effective life, then fewest untapped blockers; ties by seat index) — shared by `bot.rs::attack_target_player` and `targeting.rs`'s auto-filled "target opponent"; the bot's face attackers past a kill spill to the next-ranked seat (`server/pod_attack.rs`, CR 508.1b). One candidate in a duel, so golden traces are unchanged |
| Free-for-all last-player-standing, simultaneous-loss draw | ✅ | `team.rs`, `stack.rs` |
| CR 104.4b / 732.4 mandatory loops are draws — exact repeats, whole-turn repeats, and growing loops that mint a permanent every cycle | ✅ *growing: 2026-10-09* | `stack.rs` (exact and turn watches), `game/growth_loop.rs` (400 consecutive board-growing trigger resolutions in one step without the stack getting shallower); `core_rules::cr_recent76::cr_104_4b_a_growing_mandatory_loop_is_a_draw` |
| CR 104.3a + 119.4 — a seat kept in the game below 0 life still casts and activates | ✅ *since 2026-09-28* | `game/cast_cost.rs::life_payable` — paying 0 life is always legal; every life gate reads it. Tests `core_rules::multiplayer::cr_119_4_*`, `bot_removes_the_permanent_keeping_a_dead_seat_alive` |
| CR 603.4 — a trigger condition naming "that player" reads the event being checked | ✅ *since 2026-09-28* | `EffectContext::event_player` (`with_event_player`) at the dispatch pre-checks, push check and cast-trigger walk; `PlayerRef::TriggerEventPlayer` prefers it over the resolution scratch. Pain Distributor / Mind's Dilation. Tests `core_rules::multiplayer::cr_603_4_*`, `cmdr_gimbal::cr_603_4_*` |
| CR 508.5 — "defending player" outlives the attacked planeswalker / battle | ✅ *since 2026-09-28* | `GameState::attacked_permanent_defenders` (`note_attack_defender` at every `attacking` push); `defender_for` falls back to it (Storm the Citadel) |
| CR 601.2c — an optional removal slot declines its own side | ✅ *since 2026-09-28* | `targeting.rs::declines_own_side_pick` (`Effect::permanent_slot_is_hostile`, `SelectionRequirement::excludes_opponents_side`) |
| CR 121.2 / 601.2 — "an opponent draws / casts their Nth … each turn" is per opponent | ✅ *since 2026-09-28* | `EventKind::NthCardDrawnThisTurn(n)` or the caster's own `SpellsCastThisTurn`, never `once_per_turn` or the active player's tally: Faerie Mastermind, Erayo's Essence, The Unagi. `scripts/audit_their_nth.py --gate` (0) |
| CR 800.4i last-known information about a departed player | ⚠ *by unreachability* | Not modelled: a departed seat's zones are emptied, so a count from them reads 0. Near-unreachable — `resolve_players_unranged` filters `is_alive()` out of every fan-out, so only an effect naming a specific departed seat could see it. Actions they took already persist (`spell_names_cast_this_turn` etc. on `PlayerData`); "last turn" actions count until their next turn would have begun (`stack.rs` clears `attacked_players_this_turn` for each departed seat it rotates past; Avenge; `core_rules::multiplayer::cr_800_4i_…`) |
| CR 101.4 a per-player fan-out finishes when a body suspends | ✅ | `effects/mod.rs::splice_after_suspend`; the tail names its seats inside the effect (`PlayerRef::Seat(q)`), because a parked continuation resumes under the stack item's context. Ratchet `scripts/audit_loop_splice.py` (13/13/0, plus a staleness half). `Effect::BindScratch` pins `GameState` state the same way (`VoteTally::PerVote`, `RollDie`); `run_piles_then_clear` is closed by its splice. Open arms: ENGINE_BACKLOG's forty-third find |
| Multiplayer mulligan, no first-turn draw skip at 3+ | ✅ | `core_rules/multiplayer.rs` |
| A printed "choose an opponent" clause resolves to one seat | ✅ | `PlayerRef::HostileOpponent` → `mod.rs::default_hostile_opponent`; `Effect::EachPlayerDoes` is the fan-out singular-`who` arms can't do themselves. Ratchet `scripts/audit_singular_fanout.py` (2, both allowlisted). ENGINE_BACKLOG's fifty-first find |
| A printed "target opponent" / "that player" is one seat, not `EachOpponent` | ✅ | `TargetPlayerThen` + `ControlledBy { Target(0) }`, `shortcut::drain_target_opponent`, `R::ControlledByTriggerPlayer`. Scan: oracle "target opponent" with no player slot and an `EachOpponent` body; `grep -i "1v1\|two-player\|duel"` in catalog docs |
| "Choose an opponent" made at once — Gift (CR 702.174a), a Siege's protector (310.11a), Tribute (702.104a), Demonstrate (702.144a) | ✅ *since 2026-09-26* | `effects/choose_player.rs::choose_opponent_at_once` — a non-suspending seat ballot (headless: the opponent with the fewest creatures). Gift bodies read `PlayerRef::ChosenPlayerOfSource`. Tests `multiplayer::cr_702_174a_*`, `cr_310_11a_*`, `cr_702_104a_*`, `cr_702_144a_*` |
| "An opponent chooses / gains / of your choice" without a target — Murmurs from Beyond, Development, the opponent-split search, the opponent-picks-a-card arms, the OpponentGainsLife splice cost, Khârn the Betrayer | ✅ *since 2026-09-26* | the same `choose_opponent_at_once` ballot. Tests `multiplayer::cr_601_2h_splice_*`, `cmdr_abaddon::cr_615_kharn_goes_to_*` |
| "Choose three, repeats allowed" chosen as cast (CR 700.2d) — the Confluences, Profane / Silumgar's Command, Saheeli's Artistry | ✅ *since 2026-09-26* | `Effect::ChooseModesCast` (`GameAction::CastSpellSpree`, a target per targeted instance). The bot counters with a modal counter mode (`modal_counter_cast`) and fills X. Doomsday Confluence (X modes) still chooses at resolution |
| CR 704.5w / 704.5x a battle's protector left, or a stolen Siege protects itself | ✅ *since 2026-09-26* | `choose_player.rs::reseat_battle_protectors`, after the loss SBA's departures in `stack.rs` |
| CR 702.141a each encore token attacks its own opponent | ✅ *since 2026-09-26* | `Effect::EncoreTokens` — `chosen_player` + `Keyword::MustAttackChosenPlayer` until end of turn; the bot's `chosen_attack_target` reads the computed keyword |
| CR 508.1 "whenever a player attacks" | ✅ *since 2026-09-26* | `EventScope::AnyPlayerAttacks`, once per declaration (attacking player in `Target(0)`); Mirkwood Trapper |
| CR 726.4 the heir to a departed holder's initiative takes it (CR 726.2 venture) | ✅ *since 2026-09-26* | `stack.rs::objects_leave_with_player` → `take_initiative` |
| CR 605.3b auto-tap never leaves a mana ability's color prompt pending | ✅ *since 2026-09-26* | `actions.rs::auto_tap_for_cost_inner` — the generic-pip loop forces a prompting seat's any-color source synchronous |
| CR 903.8 a granted alternative cost for a commander cast from the command zone | ✅ *since 2026-09-26* | `actions.rs::effective_alternative_cost_in` offers Rooftop Storm's {0} at `AltCastZone::Command`, tax on top; the bot reads the effective cost (Kentaro, Henzie too) |
| CR 701.6a moving a spell is not countering it | ✅ *since 2026-09-26* | `Effect::MoveSpellToZone` — no `uncounterable` skip; 15 "return / put / exile target spell" cards and Commit // Memory. `cr_rules::cr_701_6a_*` |
| "Do this only once each turn" limits the action, not the trigger | ✅ *since 2026-09-26* | `Predicate::SourceDoneThisTurn` + `Effect::MarkDoneThisTurn` via `shortcut::{may_once_each_turn, once_each_turn_on_take}` |
| Triggers that function from exile while a card exiled itself (Cosima's voyage) | ✅ *since 2026-09-26* | `TriggerZone::WhileSelfExiled` (keyed on `exiled_with == id`), `Effect::Voyage` in `effects/voyage.rs`; bot in `server/voyage.rs` (`recent_b::cmdr_esika::cosima_voyages_and_comes_home_with_its_counters`) |
| CR 701.40a turning face up for the mana cost is manifest's / cloak's rule only | ✅ *since 2026-09-26* | `CardInstance::turn_up_by_morph_only` (Missy's Cyberman path, `Effect::TurnFaceDown`), read by `actions.rs::turn_up_mana_cost` |
| A printed "any number of target players / opponents" is a target set, not a fan-out | ✅ *since 2026-10-26 (`01HuMdRo`)* | `ApplyToTargets { max_targets: 8, min_targets: 0, filter: R::Player \| R::OpponentPlayer }` with `Selector::Target(0)` per seat: Priest of Forgotten Gods, Hollow Marauder, Thraben Charm mode 2, Officious Interrogation (`cost_per_extra_target`, CR 601.2f). `Effect::player_slot_is_hostile` reads an all-hostile `Seq` as hostile. The remaining printed-but-single rows are in INCOMPLETE's "any number of target" list |
| A printed "target opponent / target player" clause resolves to one seat | ✅ | `scripts/audit_target_opponent.py` (mirror of `audit_each_opponent`): 1 left (Consumed by Greed, whose gift branch owns slot 0). CARD_BACKLOG's "TARGET-clause class" has the four traps, incl. the two engine ones: a player slot aims at the caster unless the seat's `hostile_player_targets` flag is on (it is in `EvalWeights::default()`), and `friendliness_of_targeting_children` once read `any` over a `Seq` |
| CR 502.4 no priority in the untap step: a trigger there waits for the upkeep | ✅ *since 2026-10-17 (`01Htv6cF`)* | `stack.rs::leave_untap_step` advances to the upkeep instead of resolving the stack top |
| CR 601.2c an Adventure's / Omen's own targets obey their filters | ✅ *since 2026-10-17 (`01Htv6cF`)* | `actions.rs::half_targets_violate` (Burglar's Plot's `SharesCardTypeWithTargetSlot`) |
| CR 603.10 a leaves-the-battlefield trigger's filter reads last-known P/T | ✅ *since 2026-10-17 (`01Htv6cF`)* | `effects/eval.rs::trigger_subject_lki` — `Value::PowerOf`/`ToughnessOf` in a `CreatureDied` filter read `leaves_bf_lki` (Deathknell Berserker) |
| CR 606.1 a loyalty ability is an activated ability on the stack | ✅ *since 2026-10-20 (`01DDeUtc`)* | `game/mod.rs::activate_loyalty_ability` pushes with `TriggerPush::activated(true)`; Chandra's Regulator (`recent_b/cmdr_01dd_a.rs::chandras_regulator_copies_a_chandra_ability`) |
| CR 400.7 a blinked permanent forgets its "one that hasn't been chosen" record | ✅ *since 2026-10-20 (`01DDeUtc`)* | `CardInstance::leave_battlefield_state` clears `modes_chosen`; `cr_700_2_sol_kanar_walks_its_modes_then_leaves` |

### Commander-variant mechanics
| Feature | State | Where |
|---|---|---|
| Partner, "Partner with", Choose a Background | ✅ | `Keyword::{Partner, PartnerWith, ChooseABackground}`, `format::commanders_may_pair`. CR 702.124b/d in play: both commanders seated, tax per commander (`commander_cast_count` keyed by `CardId`), damage per (commander, player). Piloted: the Krark/Rograkh seat and `pod::decks::ZELLIX_*` (`cr_702_124k_a_background_pair_plays_a_pod_game`) — `is_legal_commander` rejects a Background alone, only `is_background_pair` lets it in. ⚠ The rule is 702.124k; four doc comments in `format.rs` and one in `card.rs` cited 702.124j ("Partner with [name]") |
| "Partner with" search trigger | ✅ | `shortcut::partner_with_search` (CR 702.124c); Sylvia Brightspear + Khorvath Brightflame |
| Partner—[text] (Friends forever, Survivors, Character select, Father & son) | ✅ | `Keyword::PartnerLabel(label)` + `commanders_may_pair`'s label equality (CR 702.124i); Elmar + Sophina |
| Doctor's companion | ✅ | `Keyword::DoctorsCompanion` + `format::is_doctor_pair` (CR 702.124m); `CreatureType::TimeLord`; The Third Doctor + Graham O'Brien in `sets/cmdr.rs` |
| Monarch, the initiative, goad, myriad, melee, voting / council's dilemma, tempting offer, join forces | ✅ *piloted* | `effect.rs` (`IsMonarch`, `HasInitiative`, `Goad` — read only through `GameState::goaders` (`game/goad.rs`; CR 611.2b; `AttachedIsGoaded`), `Myriad`, `MeleeOpponentCount`, `VoteTally`, `TemptingOffer`, `JoinForces`), `dungeons.rs`. Piloted by `pod::decks::ADRIANA_MAIN` (27 such cards); `cr_702_122_the_multiplayer_native_seat_plays_a_pod_game` asserts they stay in the 99 |
| "Whenever a player attacks one of your opponents" (CR 508.1) | ✅ | `EventScope::OpponentOfYoursAttacked`, after the declaration: once per attacked player per listener that player opposes; attacker in `Target(0)`, attacked opponent as `Triggerer`. Breena, Combat Calligrapher (`recent_b/cmdr_breena.rs`) |
| Join forces | ✅ | `Effect::JoinForces` (CR 207.2c) over `ask_seat_amount` — turn order from the controller, every ask before every payment; Minds Aglow / Collective Voyage / Mana-Charged Dragon |
| Eminence (abilities that function from the command zone) | ✅ | CR 113.6b: triggers via `EventSpec.zone: TriggerZone::{Printed, CommandZoneToo, CommandZoneOnly}` (step gather in `stack.rs`, SpellCast dispatch in `actions.rs`, event dispatcher in `mod.rs`, the "whenever you attack" walk in `combat.rs`); statics via `CardDefinition.statics_in_command_zone` + `CardInstance::command_zone_statics_active`. Edgar Markov / Arahbo / Oloro / The Ur-Dragon / Sidar Jabari |
| Commander ninjutsu | ✅ *piloted since 2026-09-20* | `Keyword::CommanderNinjutsu` (CR 702.49d) + `move_card_to`'s CR 408 command-zone arm; Yuriko is a pod deck. The cost auto-taps (CR 602.2b); bot candidate `pick_ninjutsu` |
| Lieutenant / "if you control your commander" | ✅ | `Predicate::ControlsOwnCommander`; Thunderfoot Baloth, Loyal Apprentice (`TriggeredAbility` has no intervening-`if` field, so the gate is an `Effect::If` inside the body; its Thopter gains haste via `Selector::LastCreatedToken`) |
| "Commander creatures you control …" as a static filter | ✅ | `SelectionRequirement::IsCommander` — a board fact, reads a commander under anyone's control. Falthis, Bastion Protector (also `R::Creature`: a planeswalker commander gets nothing) |
| "Pay life equal to [the commanders' identity]" as a cost | ✅ | `ActivatedAbility::life_cost_value: Option<Value>` + `Value::CommandersColorIdentityCount` (CR 903.4, union over a pair by CR 702.124c); War Room. Evaluated once during activation. ⚠ It reads the raw identity, not `GameState::commander_identity_colors` (whose no-commander "any colour" fallback would make it five life) |
| "The monarch controls enchanted creature" (CR 725) | ✅ *since 2026-09-26* | `StaticEffect::MonarchControlsEnchanted` + `GameState::sync_monarch_control` (`effects/politics.rs`), from `set_monarch` and CR 725.4 succession; Fealty to the Realm |
| CR 725.4 succession skips a seat that can't become the monarch this turn | ✅ *since 2026-10-13* | `stack.rs` leave-game handoff picks the first living seat outside `cant_become_monarch_this_turn` (Jared Carthalion) |
| Most-built commanders and top-1000 staples in the catalog | 🟡 *2026-09-26* | `decks::cmdr_legends`, `decks::cmdr_legends2`, `decks::cmdr_top1000`. Coverage: `COMMANDER_BACKLOG.md` (re-run `scripts/commander_backlog.py`). Rounded-off parts in INCOMPLETE_CARDS' session `015BCEt5` table |
| Sneak permanents enter attacking (CR 702.190b) | ✅ *2026-10-11* | `AlternativeCost::sneak` + `GameState::sneak_defenders` (Raphael, the Nightwatcher; Dark Leo & Shredder; seat 393) |
| Granted and land mayhem (CR 702.187) | ✅ *2026-10-11* | `StaticEffect::GraveyardNonlandHaveMayhem` via `GameState::effective_mayhem_cost`; a land with mayhem is played from the graveyard the turn it was discarded (Green Goblin, Oscorp Industries; seat 394) |
| Excess noncombat damage as a trigger (CR 120.10) | ✅ *2026-10-11* | `EventKind::DealtExcessNoncombatDamage` / `GameEvent::ExcessNoncombatDamage` (Fall of Cair Andros; seat 397) |
| "One or more of your opponents are attacked" by anyone (CR 506.2) | ✅ *2026-10-11* | `Predicate::AnOpponentIsAttacked` over a `YouAttack`/`AnyPlayer` trigger (Party Dude; seat 398) |
| "Doesn't untap during its controller's next untap step" (CR 502.3) | ✅ *2026-10-12* | `Effect::SkipNextUntap` (not a stun counter — CR 122.1d is a different rule); spent only on its controller's own untap step (`stack.rs::do_untap`) |
| Freerunning gated on an Assassin's or commander's hit (CR 702.173a) | ✅ *2026-10-12* | `Player::freerunning_unlocked_this_turn` (combat.rs), `Predicate::FreerunningUnlocked`, `shortcut::freerunning`; `StaticEffect::GrantFreerunningToSpells` (Ezio) |
| A commander "shuffled into its library instead" of dying keeps its CR 903.9b option (CR 614.6) | ✅ *2026-10-12* | `GameState::shuffle_into_library_instead` (Progenitus; seat 529) |
| Playing another player's cards for a turn — their library top (Xanathar) or hand (Sen Triplets), CR 108.3 | ✅ *2026-10-12* | `Player::may_play_library_top_of` / `may_play_hand_of`, `foreign_library_top_seat` / `foreign_hand_seat` on the cast and land paths; the pod bot uses both (seats 513-514) |
| An emblem's "damage can't reduce your life below 1" (CR 114.4) | ✅ *2026-10-12* | `clamp_damage_to_life_floor` reads emblem statics (Serra the Benevolent) |
| "Can't transform" statics (CR 701.27) | ✅ *2026-10-12* | `StaticEffect::CantTransform`, read by `transform_permanent` (Immerwolf; seat 406) |
| "Attacked this turn by an Assassin you controlled"; "hasn't dealt damage yet" | ✅ *2026-10-12* | `Player::attacked_by_assassins_of` / `Predicate::AttackedByYourAssassinThisTurn` (Ramses); `CardInstance::has_dealt_damage` / `Predicate::SourceHasDealtDamage` (`game/damage_dealers.rs`) — seat 405 |
| Double-faced permanents can't be turned face down (CR 712.16) | ✅ *2026-10-12* | `CardInstance::is_double_faced`, checked by `TurnFaceDown` / `TurnFaceDownAs` (Ixidron, Illithid Harvester; seat 403) |
| Die results modified to 0, ignore-all-but-highest, a once-a-turn result shift (CR 706.2 / 706.6) | ✅ *2026-10-12* | `RollDie` floors a modified result at 0 (The Deck of Many Things); `Effect::IgnoreAllButHighestRoll` (Iron Mastiff); `StaticEffect::PayLifeToShiftDieOnceEachTurn` (`effects/dice_choices.rs`) — seat 404 |

### Simulation & tooling
| Feature | State | Where |
|---|---|---|
| N-seat pod runner | ✅ | `pod/mod.rs` — `build_pod_template`, `play_one_pod_game`, `run_pod_games` |
| 4-player Commander demo state | ✅ | `demo.rs::build_commander_state_seeded`, the first four target decks |
| Commander pod mode in `bot_ladder` | ✅ | `bot_ladder --commander [--seats N] [--pod-random]`. `--pod-random` (2026-11-26) draws each game's seats from every target deck by (seed, game index, deck count) and prints a `--pod-decks .. --first I --games 1` replay for an undecided game and, through a panic hook, for a panic — the sweep that reaches the 6,195 decks `--seats N`'s first-N field never seats (its first 9,000 games found two engine bugs, ENGINE_BACKLOG `01WRuXc3U3`). Runs on a debug build since `88f07f22` (the worker was on the 2 MiB `scope.spawn` default), so every pod figure committed before it is a `release-fast` statement only (ENGINE_BACKLOG's sixty-second find) |
| 6,260 legal target decks, a hundred and eighty official lists | ✅ | `pod/decks.rs`, appended in order so no existing `--seats N` reading moves; past `MAX_SEATS` (64) only `--pod-decks` reaches a seat. The pod budget counts plays, `max(seats, 4) × 1,000`. Ranges: seats 1-10 the original field — Sigarda GW, Judith BR, Hanna UW, Tatyova GU, Krark+Rograkh R (Partner), Edgar Markov BRW (Eminence, `--seats 6`), Freyalise G (planeswalker commander, CR 903.3a, `--seats 7`), Zellix + Passionate Archaeologist UR (Choose a Background, CR 702.124k, `--seats 8`), Yuriko UB (commander ninjutsu, CR 702.49d — the commander leaves the command zone without a cast, so `commander_cast_count` stays put; `cr_702_49d_a_commander_ninjutsu_seat_plays_a_pod_game`, `--seats 9`), Adriana RW (the multiplayer-native mechanics, `--seats 10`); each runs the colorless staples (Sol Ring, Command Tower, Arcane Signet, Commander's Sphere, Mind Stone, Path of Ancestry, Opal Palace) and each two-color seat its Signet + Talisman. Seats 11-183 official precons, card for card from MTGJSON's deck files (`scripts/precon_scan.py` ranks precons by missing cards); seats 184 on EDHREC average decks (`json.edhrec.com/pages/average-decks/<slug>`, `scripts/edhrec_deck.py`, `scripts/seat_decks.py` writes a seat); 4855-4858 the last four official Commander decks (an MTGJSON scan over all 198 found the rest seated); 4859-6053 every filterable zero-gap legal pair deck; 6089-6093 the last legal unseated EDHREC singles; 6094-6096 the Star Trek precons (from WotC's decklist page; TRC is Commander-legal from 2026-11-13); 6097-6188 Star Trek EDHREC average decks (`decks::trek_singles`..`7`, 2026-10-28..11-01); 6189-6205 the 2026-11-26 EDHREC singles that missed only their commander or one or two cards (`decks::trek_singles8`, DECK_FEATURES `01WRuXc3U3`); 6206 Massimo (`01WRuXc5Gy`); 6207-6251 every complete legal EDHREC pair deck left unseated, less the chosen-color commanders'; 6252-6260 the chosen-color pairs (Clara Oswald / Prismatic Piper / Faceless One) whose average fits one chosen color (2026-11-26). Which session seated which range, and each deck's residuals: DECK_FEATURES' generated table and ENGINE_BACKLOG by session id. ⚠ A test that finds its deck with `last()` breaks on the next append: the Partner test asks for two creature commanders (CR 702.124h), the planeswalker test for exactly one non-creature commander. Judith's retune (DECK_FEATURES): a *better* aristocrats list measured worse — a one-ply material evaluator cannot price a value engine |
| Per-deck card coverage in a pod run | ✅ | `bot_ladder --commander --card-census` — totals the action census by card and names, per seated deck, what a run never cast, played or activated. All ten original target decks and the official lists play every card |
| Pod state invariants beyond panics (audit build) | ✅ *2026-10-08* | `pod/mod.rs`, checked after every settled action under `debug_assertions`: CR 400.7 (a card off the battlefield sheds battlefield state), CR 514.2 / 500.4 / 302.6 at each turn boundary, CR 511.3 / 506.2 / 506.4 combat, CR 725.4 / 726.4, CR 704.5d, a stack spell carries no battlefield state, a library card is face up, no card is transformed in exile save a battle, no stack object belongs to a departed seat — beside the older CR 400.1 / 704.3 / 613.7d / 610.3 / 800.4 / 903 ones. Finds: ENGINE_BACKLOG "tenth / twelfth / eighteenth Commander run" |
| Dead triggers in a pod run | ✅ *since 2026-09-27* | `--card-census` prints "never triggered" — a played card none of whose printed triggers reached the stack (`ActionCensus::note_triggers`). ⚠ Census with `--a dflt`: without it the pilot is `baseline` |
| Golden outcomes for fixed-seed Commander pods | ✅ | `pod::tests::cr_903_seeded_pod_outcomes_match_the_committed_table` — three seeds' (winner, turns, actions), cross-process. A per-action trace would be a ~400 KB file every Commander commit re-blesses |
| Trigger chains off targets can't recurse without bound | ✅ *since 2026-09-26* | `GameState::push_pending_trigger` dispatches "becomes the target" events in place only eight deep, then queues them (CR 603.3b); `cr_603_3b_facing_scalelord_reckoners_do_not_recurse_forever` |
| A spell-copy chain can't grow the stack without bound | ✅ *since 2026-09-26* | `recommend::MAX_STACK` (512 spells; triggers aren't counted): `copy_stack_spell_controlled` copies nothing past it and `stop_reason` ends the game as a `BoardCap` (`a_spell_copy_chain_stops_at_the_stack_bound`) |
| A token batch can't ask for an unbounded mint | ✅ *since 2026-09-26* | `GameState::doubled_token_count` / `scaled_token_count` — every mint loop doubles per CR 614.13 doubler, triples per Ojer Taq (`StaticEffect::TripleCreatureTokens`), and caps at `recommend::MAX_BATTLEFIELD` (`cr_614_13_forty_token_doublers_mint_a_bounded_batch`) |
| A pod seat can't recast a legend it already controls without bound | ✅ *2026-10-16* | `pod/mod.rs::RepeatGuard::admit_duplicate_legend` — two such casts a turn (CR 704.5j); `pod::tests::duplicate_legend_casts_are_capped_per_turn` |
| A Commander bot can't run an optional activation loop to the board cap | ✅ *2026-10-14* | `server/activation_cap.rs`: past 40 non-mana activations in one turn (`Player.nonmana_activations_this_turn`) a Commander seat passes — the activation twin of the twelve-combat attack brake (CR 104.4b). Mana abilities stay free; two-player play is unchanged (`--bench` byte-identical) |
| Bot takes every decision the pod decks introduce | 🟡 | Human-facing: a trigger's further "up to N" targets, per-slot filters past an `OptionalTargets` minimum, a targeted reflexive trigger (CR 603.7d) and a cast's off-board extra slot are posed slot by slot (`game/trigger_slots.rs`, CR 601.2c), leading with the engine fill the bot takes; per-opponent fan-outs keep the engine fill. Resolution-time picks (`effects/choose_one.rs`, `effects/keep_one.rs`): a gain defaults to the bot's best, a harm to an opponent's priciest permanent. Bot modules: `server/grant_cast.rs` (grant-then-cast, impulse), `server/command_zone.rs` (Derevi), `server/channel.rs`, `server/room.rs`, `server/fog.rs`, `server/manland.rs`, `server/evasion.rs`, `server/selection_sink.rs`, `server/regenerate.rs`, `server/counter_sink.rs`, `server/mutate.rs`, `server/tap_maker.rs`, `donate_draw.rs`, `upkeep_recursion.rs`, `graveyard_to_top.rs`, `regrowth_sink.rs`, `server/untap_ramp.rs`, `server/graveyard_cast.rs`, `server/hideaway.rs`, `server/loop_hazard.rs` (the bot never starts its own CR 104.4b mandatory loop), `transform_sink::pick_creature_upgrade_transform`, `pick_stack_response`, `spell_response::pick_substitution_response`; `cast_candidates` blocks for escape, replicate, buyback, entwine, squad, fuse, casualty, bargain, retrace; `pay_with_restricted_sources` (CR 106.6); X spells target at their X; reconfigure in `pick_equip`; `trim_blocks_to_payable_tax`; `pod_attack::x_held_for_attack_tax` (CR 508.1g); `MayPayGenericUpTo` taps untapped sources. Optional loops (CR 732.2a): `pod/mod.rs::RepeatGuard` — past 64 activations of one ability of one permanent in a turn a seat passes (pods only; Equip / Reconfigure / Crew / Saddle / waterbend count). A 183-deck census leaves only Semester's End unplayed. Open: waterbend / sacrifice-reduce, none in a pod list (ENGINE_BACKLOG). Never-activated counts per pass: ENGINE_BACKLOG |
| A net pilot in a pod | ⏳ planned (the v5 net: ⛔ by design) | the observation encoder is two-seat (`encode_state_inner`'s `1 - seat`, `sources: &[_; 2]`). `net_eval` answers `None` for any state that is not two seats (`net_for_state`), so a net profile in a pod plays its material leaf; pods seat the heuristic default; `bot_ladder --commander` refuses a net profile and runs the material-leaf search (`mcts-dflt-256` in a `dflt` field 0.87x [0.56x, 1.18x]). The plan for one net at every seat count is ML_NOTES "One net for every mode" (2026-09-28): step 1 — the table net, `server::encode_table`, parity in both crates — is built and untrained; steps 2-6 are recorded there |
| Debug pod state invariants (crash-freedom beyond "no panic") | ✅ *2026-10-06* | `pod/mod.rs::play_pod_game`, after every action, `debug_assertions` only: no commander in a library outside a pending CR 903.9b offer; `duplicate_zone_id` (CR 400.1); CR 800.4a; once settled, no ≤ 0-toughness creature (704.5f) and no attachment to a vanished host (704.5m/n). Run them: `cargo build --bin bot_ladder` (debug), `--commander --pod-decks …` per deck group, `CRAB_ANSWER_LOG=strict`. `CRAB_POD_FUZZ=<n per 10,000>` (`pod/fuzz.rs`; `CRAB_LADDER_FUZZ` for 2-player ladder games) swaps a seat's step for a random alternative so the invariants see lines the tuned bots never take; the CR 903.9b library check stands down under fuzz |
| Measure a bot change in a pod | ✅ *since 2026-09-26* | `bot_ladder --commander --a A --b B`: `A` in one seat, `B` in the rest, each deal once with `A` in every seat (`pod::run_pod_hero_games`); A's win share against the mirror's 1/N (exact null), standard error over deal groups. ⚠ `--card-census` without `--a` runs `baseline`, so its never-activated list is `baseline`'s, not the default's — census with `--a dflt`. Adopted: chumping a commander about to deal its 21st point (CR 903.10a); votes and option asks by settled outcome (`EvalWeights::option_eval`); the attack sim's crack-back through every opponent's turn (`EvalWeights::pod_horizon`). Measured and left off: `EvalWeights::leader_target`, `pod_keyword_pct`, opponent ranking by board power, kill-first, pre-combat {T}-ability spends. Numbers: ML_NOTES |

## Tier 1 — High-leverage engine primitives

Each unblocks a large swath of cards.

1. 🟡 **Replacement-effect framework.** `replacement.rs` models zone-change
   replacements (Commander → command zone); the rest is per-card. Shipped:
   enters-under-an-opponent's-control (`CardDefinition.enters_under_opponent_control`
   — Captive Audience), "can't be regenerated this turn" (CR 701.15g —
   `Effect::CantBeRegeneratedThisTurn`), enters-tapped (`StaticEffect::EntersTapped`)
   and the enters-untapped override (`StaticEffect::LandsEnterUntapped` —
   Spelunking), exile-instead for non-cast creatures (Containment Priest),
   opponent-creature-dies → exile (Valentin), graveyard → exile hate
   (`ExileCardsBoundForGraveyard` via `route_to_graveyard` — Rest in Peace,
   Leyline of the Void), counter-lock (Solemnity), counter/damage doubling
   (Doubling Season, Furnace of Rath), prevention shields (`prevention_shields`),
   per-source combat shields (Maze of Ith), damage redirection (Palisade Giant),
   draw doubling (Thought Reflection), damage halving (Ghosts of the Innocent),
   creature-ETB control steal (Gather Specimens), as-enters choice of P/T and
   keyword (`enters_as_choice`, applied before the first SBA — Corrupted
   Shapeshifter), a general as-enters one-shot (`CardDefinition.as_enters_effect`
   — Ixidron), skip-step and skip-turn. Counter-placement replacements (Hardened
   Scales, Doubling Season, Mowu's `ExtraPlusOneCounterOnSelf`) apply on the
   proliferate path too (CR 614.16, `scaled_counter_count_on`). The draw branch
   is closed (skip, exile-and-play, redirect, doubling, empty-hand bonus, dredge,
   `MayReplaceDrawWithTutor`, `MayReplaceDrawWithRevealUntilKind`). Still to
   generalize: as-a-copy ETB.
   **CR 614.12 is a closed class (2026-09-20, ENGINE_BACKLOG's forty-eighth
   find).** `game::as_enters::apply_as_enters_replacements` is the one funnel
   all four battlefield-entry paths run — the cast (`stack.rs`), the universal
   move (`effects/movement.rs`), the land drop (`actions.rs::play_land`) and
   the token mint (`mod.rs::mint_token_with_counters`).
   `ResumeContext::LandEntry` is the land drop's continuation (it can neither
   park on a stack item nor be replayed) and resumes into
   `actions::finish_land_entry`. `Effect::AsEntersChooseMode` is the modal ask
   without a stack item; `Effect::SourceEntersTapped` the "it enters tapped"
   branch that emits no tap event. ⚠ A `wants_ui` seat is asked for real only
   on the land drop; the other three paths still drive the ask through
   `resolve_effect_driven`.
2. ✅ **Multi-pick / "choose N" decisions.** `Decision::ChooseModes`;
   pick-from-revealed via `Effect::LookPickToHand` (Impulse, Strategic Planning).
3. ✅ **Player-chosen combat damage assignment order.**
   `Decision::CombatDamageOrder` prompts the attacker (510.1c).
4. ✅ **Linked "until this leaves play" exile** (603.6e).
   `Effect::ExileUntilSourceLeaves` + `return_linked_exiles` (Banisher Priest,
   Fiend Hunter, Oblivion Ring, Brain Maggot, Tidehollow Sculler). Monarch-linked
   sibling (CR 724 — `Effect::ExileUntilOpponentMonarch` + `ExileLink.monarch_guard`,
   returns when the monarchy moves rather than when the source leaves; Palace Jailer).
5. ✅ **Copy of a permanent (clone).** `Effect::BecomeCopyOf` +
   `enters_as_copy` ship Clone, Phantasmal Image, Mirror Image, Stunt Double;
   token copies via `CreateTokenCopyOf` (CR 707.2 copiable values only, CR
   707.2e non-legendary rider — Helm of the Host); continuous "becomes a copy"
   via `BecomeCopyOfFor` (Mirrorform, Vesuva).
6. ✅ **Copy-a-spell-on-the-stack.** `Effect::CopySpell` /
   `CopySpellMayChooseTargets` (new-target choice) — Storm cards, Reverberate, Fork.

## Tier 2 — Engine rules fidelity (beyond Tier 1)

- ✅ **APNAP trigger ordering** — inter-player (`apnap_rank`) plus
  same-controller ordering with a real server suspend (`ResumeContext::
  TriggerOrder`), so networked seats are prompted.
- ✅ **Block-trigger conformance (CR 509.3a–e)** — "whenever this blocks" /
  "becomes blocked" fire once per creature under a multi-block; the per-object
  wordings reach every partner from one instance; `EventKind::BlocksNOrMore` /
  `BecomesBlockedByNOrMore` gate on the finished assignment (Lairwatch Giant).
- 🟡 **Divided damage / counters** — `Effect::DealDamageDivided` +
  `Effect::DistributeCounters` (Jugan) share `Decision::DivideDamage` (the modal
  is noun-aware). Forked Bolt, Pyrokinesis, Crackle with Power. Remaining:
  "choose targets as it resolves". The prevention sibling ships as
  `Effect::PreventNextDamageDivided` (Serra's Hymn), sharing the same
  `Decision::DivideDamage`.
- 🟡 **Targeting refinements:** CR 601.2c's cross-slot restrictions are
  `SelectionRequirement` atoms read from `GameState::target_slots_scratch`:
  `SameControllerAsTargetSlot` and `OtherThanTargetSlot` (the printed
  "another" between two slots; ratchet `scripts/audit_another_target.py`),
  enforced by the cast path, the activation path and the auto-picker.
  `SelectionRequirement::OnBattlefield` is the on-board half of a mixed zone
  clause (graveyard half ratcheted by `scripts/audit_target_zone.py`).
  Resolution-time re-check (608.2b) covers single/multi-target spells, Auras
  and `{X}`-from-cost filters (`ManaValueExactlyXFromCost` — Hearth Kami).
  "Up to N targets" is `Effect::ApplyToTargets`; an optional slot beside a
  required one is `Effect::OptionalTargets { min, body }` (Primal Might, Boom
  Box). The cast path requires every required slot of a multi-slot spell
  (`Effect::required_target_slots`, CR 601.2c; ratchet
  `spells_requiring_two_targets_print_no_optional_target`). Protection gates
  spells and abilities (CR 702.16c — `ability_target_has_protection`) across
  color / creatures / creature-type / spell-subtype / multicolored
  (`ProtectionFromMulticolored`) / monocolored (`ProtectionFromMonocolored`),
  and combat damage both ways (CR 702.16e — `damage_prevented_by_protection`);
  it is re-checked as the spell or ability resolves (CR 608.2b —
  `spell_protection_blocks`, `ability_target_newly_untargetable`, which also
  reads shroud / hexproof), as are colour hexproof (`spell_color_hexproof_blocks`)
  and a player's protection; protection stops the controller's own spells too.
  Multi-kind slots ship (`Selector::ControlledBy { who: Target(n) }` declares
  slot `n` a player target — How to Start a Riot, Sokka's Haiku).
  Ignore-hexproof statics: `IgnoreOpponentsCreatureHexproof` (Glaring
  Spotlight) and `IgnoreOpponentsHexproof` (Kaya, Bane of the Dead); the server
  view and client targeting filter mirror both. Remaining: "target each".
- 🟡 **Continuous-effect breadth:** layer-3 text-changing ✅ (Trait Doctoring);
  land-type statics ✅ (Blood Moon, Urborg); layer-4 granted supertype ✅
  (`Modification::AddSupertype` — the Ring-bearer's Legendary rider, CR 701.54c);
  layer-4 set-creature-types ✅ as a one-shot (`Effect::BecomeCreatureType` —
  Turn to Frog / Snakeform / Polymorphist's Jest) **and** the CR 613.8 type-lord
  dependency (a retyped creature is now seen by `AllWithCreatureType` lords via
  a `gate_types` second pass); layer-4 add-creature-type + layer-7b
  `SetPowerToughnessToManaValue` animating non-Aura enchantments to `MV/MV`
  creatures ✅ (`StaticEffect::NonAuraEnchantmentsAreCreatures` — Opalescence,
  Starfield of Nyx; the 5+-enchantment gate is materialized state-aware).
  Remaining: CDA corners, full text-box swaps, "becomes a copy of" layer
  interaction, type-gated `CardMatch` lords.
- 🟡 **Static ability framework:** cost-reduction statics, "you may play"
  permissions, anthem stacking incl. disjunctive multi-type lords (Blex);
  devotion-gated god states (`NotCreatureWhileDevotionBelow`) + devotion
  bonuses (`StaticEffect::DevotionBonus` — Altar of the Pantheon, CR 700.5);
  keyword loss (`LoseKeyword` — Nowhere to Run); live-recompute `GrantKeyword`
  **and `PumpPT`** statics over combat state (`IsAttacking`/`IsModified` —
  Bone-Cairn Butcher's "attacking tokens have deathtouch" and Orcish
  Oriflamme's "attacking creatures you control get +1/+0"); turn-gated statics
  (`StaticEffect::WhileYourTurn`, CR 611.2 — general on both the live and pure
  gather paths; Blacksmith's Talent L3); an opponent-scoped spell tax
  (`OpponentSpellsCostMore` — Grand Arbiter Augustin IV, exempts the controller)
  and its turn-gated sibling that also taxes non-mana abilities
  (`OpponentActivityCostsMoreOnYourTurn` — Tithe Taker, spell half in
  `extra_cost_for_spell` + ability half in `effective_ability_mana_cost`)
  and a multicolored-only spend restriction (`SpendRestriction::MulticoloredSpell`
  — Pillar of the Paruns). Remaining: broader "you may play", devotion-gated
  non-type states.
- 🟡 **Replacement of life/draw/damage events** (ties to Tier-1 #1). Life-loss
  doubling (`OpponentLifeLossDoubledDuringYourTurn` — Bloodletter) and scoped
  unpreventable combat damage (`ControllerCreaturesCombatDamageCantBePrevented`
  — Questing Beast) now ride the `adjust_life` / prevention chokepoints.
  Noncombat-only damage doubling (`DoubleNoncombatDamageToOpponents` — Solphim,
  Mayhem Dominus) rides the `deal_damage_to_from` funnel and stacks with the
  global Furnace-of-Rath doubler (combat damage stays exempt). Life-gain
  replacements now cover both a flat bonus (`LifeGainBonus` — Honor Troll) and a
  multiplier (`LifeGainMultiplier` — Rhox Faithmender), multiplier applied first
  (CR 614), neither firing on a 0-gain (CR 119.10). Hellbent all-your-sources
  damage doubling (`DoubleYourSourcesDamageWhileHellbent` — Anthem of Rakdos)
  rides `scale_damage_to`, gated on the controller's empty hand.
- ✅ **Regeneration shields & "next time" prevention** as proper shields.
- 🟡 **Damage marking vs. wither/−1−1, lethal/indestructible** audited against
  CR 120/704. (Wither/Infect damage-as-counters already ships; lethal-by-power
  `StaticEffect::LethalDamageByPower` — Zilortha — now overrides the toughness
  threshold in the SBA. **Excess damage** (CR 120.10) is tracked per resolution
  in `deal_damage_to_from` — `Predicate::ExcessDamageDealtThisResolution` gates
  "if excess damage was dealt this way" (Orbital Plunge). **CR 120.4a
  redirection ships**: `Effect::{DealDamageExcessToController, DealDamageExcessTo}`
  split the event before it happens, so the creature takes exactly lethal
  (deathtouch-aware via `lethal_damage_needed`). Remaining: the broader
  marking-interplay audit.)
- ✅ **Prevention funnel is single-entry** — CR 615.5/615.8: chosen-source
  shields (`damage_prevented_sources`, carrying a life-gain beneficiary and a
  one-instance flag) are applied inside `apply_prevention_shields`, and combat
  no longer short-circuits a fully-prevented dealer, so a shield's riders fire
  on combat damage and `DamagePrevented` is emitted uniformly (615.13).
  Hallow (turn-long + life refund), Awe Strike (next instance only).
- ✅ **Attack restrictions by board state** — `CantAttackUnlessLandCount`
  (Harbor Serpent's five Islands) and `CantAttackUnlessOpponentDamaged`
  (Bloodcrazed Goblin) join the CR 508.1a gate list in `declare_attackers`.
- 🟡 **Loyalty fidelity:** loyalty-set effects ✅, proliferate on loyalty ✅
  (`CounterType::Loyalty`, test `cr_701_34_proliferate_adds_loyalty_counter`),
  combat damage to a planeswalker removes loyalty ✅ (CR 306.9, test
  `cr_306_9_combat_damage_to_planeswalker_removes_loyalty`); multi-target
  loyalty abilities now auto-fill slots 1.. (`auto_extra_targets_for` — Domri
  Rade's −2 two-target fight). Remaining: "any time" activation riders;
  UI-chosen (rather than auto-picked) extra loyalty targets.
- ✅ **State-based action coverage:** ±1/±1 annihilation ✅, counter caps ✅,
  legend rule ✅, saga sacrifice ✅, world rule ✅, illegally-attached Aura ✅
  (704.5n — host fails the printed enchant filter). Dungeons ✅ (CR 309/701.49
  — `base::dungeons`, `Effect::Venture`, `decks::afr`; rooms resolve inline).
  Battle-defeat SBA ✅ (CR 704.5x — a Siege with no defense counters is defeated,
  `stack.rs`). No remaining SBA gap of note.

## Tier 3 — Object model & zones

- ✅ **Battle card type** (CR 310) — `CardType::Battle` + `BattleSubtype::Siege`,
  defense counters (CR 310.7), protector choice (CR 310.6), attack-your-own-Siege
  (`AttackTarget::Battle`), **both combat and noncombat** damage remove defense
  counters (CR 310.10 — the noncombat path mirrors the planeswalker loyalty
  strip in `deal_damage_to_from`; Onakke Javelineer's ping), defeat→exile/
  transform SBA (CR 704.5x). 6 MOM Invasions in `decks::mom`; tests in
  `tests/mom.rs`. Remaining: multiplayer protector choice.
- ✅ **Sagas** (714). `saga_chapters` + `saga_advance` (History of Benalia, The
  Eldest Reborn); DFC sagas ✅ (`ExileSelfReturnTransformed` — Fable of the
  Mirror-Breaker); Read Ahead ✅ (702.155 starting-chapter choice).
- ✅ **Split cards** (709) + **Fuse** — `CardDefinition.split`,
  `CastSplitRight`/`CastSplitFused` (Wear // Tear).
- ✅ **Adventure** (715) — `CardDefinition.adventure` + `CastAdventure` (Bonecrusher
  Giant, Brazen Borrower, Murderous Rider, …).
- 🟡 **Classes / Cases / Backgrounds.** **Rooms ship** (709.5 — `room` +
  `CastRoomDoor`/`UnlockRoomDoor`; Unholy Annex // Ritual Chamber). **Cases ship**
  (MKM — `CardDefinition.case` + `CaseData.to_solve`/`solved_*` +
  `CardInstance.case_solved`; solved at the controller's end step via
  `process_case_solves`, `EventKind::CaseSolved` drives "whenever you solve a
  Case"). Six Cases + Case File Auditor in `decks::recent242`. Remaining:
  Classes (levels) and Backgrounds.
- ✅ **Leveler cards** (702.87 — `level_bands`; Student of Warfare).
- ✅ **Transforming DFCs** (712) — `Effect::Transform` toggles the active face in
  place, round-trips through serde/snapshot (Delver, Concealing Curtains).
  Remaining: DFC sagas.
- ✅ **Meld** (701.37) — `Effect::Meld` + `meld_parts`, unmelds on leave (Urza +
  Mightstone/Weakstone → Urza, Planeswalker).
- ✅ **Flip cards** (Kamigawa, CR 711) — `flip_face` + `Effect::Flip` +
  `GameEvent::Flipped`; ki counters; flip in place, revert off-battlefield
  (711.6); `damaged_by_this_turn` source tracking; `flip_when_has_keyword`
  CR 603.8 state-triggered flip (Student of Elements). Whole CHK flip cycle
  ships (Cunning Bandit … Bushi Tenderfoot, Kitsune Mystic + Autumn-Tail's
  two-target aura-move, Nezumi Graverobber, Student of Elements).
  **Prototype** (CR 702.160) ✅ — `CardDefinition.prototype` +
  `GameAction::CastPrototype`: cast a colorless artifact creature for its
  smaller, colored prototype cost/size, keeping abilities/types (the BRO
  cycle: Goring Warplow, Steel Seraph, Phyrexian Fleshgorger, …).
  **Omen** ✅ (`CardDefinition.omen` + `omen_casting` — the Adventure-style
  alternate instant/sorcery half).
- 🟡 **Face-down permanents** (708) — `face_up_def` stashes the real card; Manifest
  / ManifestDread + `TurnFaceUp`; Morph/Megamorph cast-face-down ✅. Remaining:
  Disguise/Cloak edge cases (both core paths ship — see Tier 4).
- 🟡 **Ante / conspiracy / sticker / attraction** zones. Ante ✅ (CR 407 — see
  "Recently closed"); dungeons ✅ (CR 309); **attractions ✅** (CR 717 — the
  command-zone Attraction deck + junkyard, `Effect::OpenAnAttraction`, the
  precombat-main roll-to-visit turn-based action, and Visit triggers keyed to
  each card's lit-up numbers). Stickers 🟡 (CR 123 — **name stickers only**:
  `crabomination_base::sticker` + `Effect::PutNameSticker`, stand-in sheets;
  _____ Goblin). Remaining: conspiracy; ability / P-T / art stickers, tickets.
- ✅ **Emblems** as command-zone objects — `Player.emblems` + `CreateEmblem`,
  carrying both triggered and **static (anthem) abilities** (Vivien Reid's −8;
  synthesized into continuous effects in `gather_continuous_effects`).
- ⏳ **Sideboard zone** + "from outside the game" (wishes, companions).

## Tier 4 — Keyword & ability mechanics (the long tail)

Each a small targeted feature; sweep batch by batch.

- **High frequency / modern staples:** ✅ Madness, ✅ Escape, ✅ Adventure,
  ✅ Soulbond, ✅ Mutate (CR 702.140 — `CardDefinition.mutate` +
  `GameAction::CastMutate`; merges onto a non-Human host you own, unions
  abilities, scatters on leave, `EventKind::Mutated` triggers — the Ikoria
  cycle), ✅ Companion ({3} sideboard→hand + `companion`
  deck-construction validation, full Ikoria cycle),
  ✅ Foretell, ✅ Disturb, ✅ Daybound/Nightbound (keywords + day/night +
  502.2 transition + DFC auto-flip), ✅ Decayed, ✅ Blitz, ✅ Casualty, ✅ Connive,
  ✅ Backup, ✅ Bargain,
  ✅ Craft (CR 702.169 — `shortcut::craft`: sorcery-speed activated ability
    pairing `craft_exile_cost` (exile N other objects from among permanents you
    control and/or graveyard cards) with `Effect::ExileSelfReturnTransformed`;
    LCI batch in `sets::lci` — Tithing Blade, Visage of Dread, Spring-Loaded
    Sawblades, Waterlogged Hulk),
  ✅ Disguise/Cloak, ✅ Plot, ✅ Saddle,
  ✅ Gift (CR 702.165 — `CardDefinition.gift` + `GameAction::CastGift`; promise
  the gift and resolve the enhanced `gifted_effect`, incl. target-broadening —
  Into the Flood Maw, Long River's Pull),
  ✅ Survival (CR 702.180 — "at your second main phase, if tapped …" as a
  `StepBegins(PostCombatMain)`/`ActivePlayer` trigger under a tapped
  intervening-`if`; Bloomburrow Survivor batch),
  ✅ Omen (CR 702.183 — `CardDefinition.omen` + `GameAction::CastOmen` +
    `CardInstance.omen_casting`; cast the creature card as its instant/sorcery
    Omen half, which shuffles into the owner's library on resolution *or*
    counter via the `route_to_graveyard` funnel — the Tarkir Regent/Stormbrood
    Dragon cycle),
  ✅ Offspring, ✅ Impending, ✅ Ninjutsu, ✅ Embalm / Eternalize,
  ✅ Exhaust (activate-only-once activated abilities — Camera Launcher),
  ✅ Mayhem (CR 702.187 — `Keyword::Mayhem` + `GameAction::CastMayhem` reusing
    the flashback exile-after machinery, gated on `Player.discarded_this_turn`;
    "if the mayhem cost was paid" riders via `cast_via_mayhem`/`SpellWasMayhem`),
  ✅ Harmonize (CR 702.180 — `Keyword::Harmonize` + `GameAction::CastHarmonize`:
    graveyard recast with optional tap-a-creature generic discount, exile-after),
  ✅ Web-slinging (CR 702.188 — alt-cost: pay cost + return a tapped creature),
  ✅ Flurry (`shortcut::flurry` — "your second spell each turn" trigger over
    `SpellsCastThisTurnEquals`),
  ✅ Job Select (CR 702.182 — living-weapon-shaped Equipment minting a 1/1 Hero),
  ✅ Renew (graveyard-exile activated ability via `from_graveyard` +
    `exile_self_cost`), ✅ Mobilize / Mobilize X (`shortcut::mobilize`,
    `mobilize_value`), ✅ Seek (CR 701.52 — `Effect::Seek`, random library pick),
    ✅ Time Travel (CR 701.56 — `Effect::TimeTravel`: removes time counters from
    the player's suspended cards / adds to vanishing permanents; bot heuristic,
    per-object UI choice is a follow-up),
    ✅ Villainous Choice (CR 701.55 — `Effect::VillainousChoice`: each chooser
    in APNAP order takes the lesser-self-harm option; impossible options dodge),
    ✅ **The Ring tempts you / Ring-bearer** (CR 701.54 — `Effect::RingTempts`
    + `Player.{ring_temptations,ring_bearer}`; the four cumulative emblem
    abilities applied off the level: can't-be-blocked-by-greater-power (1+),
    attack-loot (2+), blocked-creature-sacrifice (3+, via
    `Effect::SacrificeAtEndOfCombat`), combat-damage drain (4+). `decks::ltr`
    LTR batch + `EventKind::RingTempted` for "choose a Ring-bearer" payoffs.
    Bearer auto-picked (highest power); per-player UI choice is a TODO.md
    follow-up).
- **Counter / +1+1 matters:** ✅ Proliferate, Bolster, Adapt, Training, Evolve,
  Mentor, Modular, Graft, Outlast, Renown, Bloodthirst, Monstrosity, Devour,
  Amass — all via `shortcut::*` builders.
- **Cast-from-elsewhere:** ✅ play-from-library-top statics (Courser, Oracle of
  Mul Daya, Mystic Forge), ✅ Suspend (creature-suspend haste + free-cast target
  UI are follow-ups), ✅ Forecast, ✅ Hideaway, ✅ Aftermath (`CastAftermath`),
  ✅ Unearth (CR 702.84 — `shortcut::unearth`: a `from_graveyard` sorcery-speed
  ability that returns the card with haste + an end-step exile; the bot offers
  graveyard-activated abilities, the client hover panel labels them).
- **Combat-flavor:** ✅ Bushido, Flanking, Rampage, Provoke, Battle Cry, Exalted,
  Frenzy, Melee, Dash, Boast, Afflict, Enlist, Mobilize, Myriad, Amass,
  **Exert** (CR 701.43 — an *optional cost to attack* per CR 508.1g, announced
  by `GameAction::DeclareAttackersExerting`; the "when you do" is **linked**
  (CR 701.43d / 607.2h) and rides `EventKind::Exerted`, so an unexerted attack
  gets neither the bonus nor the skipped untap. A silent declaration falls to
  `exert_pays_off`, which takes it only when the linked bonus can do something;
  the `wants_ui` modal is the residual, in INCOMPLETE_CARDS),
  Assigns-combat-damage-by-toughness (`AssignsCombatDamageByToughness`, CR 510.1c
  — Doran, Tapestry Warden, Bill the Pony).
- **Value/ETB:** ✅ Investigate, Fabricate, Riot, Raid, Afterlife, Explore, Squad,
  Forage, Endure, Exploit, Extort, Support, Suspect, Discover, Collect Evidence,
  Expend, Valiant, Cohort (Munda's Vanguard, Drana's Chosen — tap-another-Ally
  activation cost).
- **Leaves-battlefield LKI:** ✅ 603.10 — `Value::PowerOf`/`ToughnessOf` read a
  dying object's last-known P/T (Goldvein Hydra, Cacophony Scamp).
- **Spell-matters:** ✅ Escalate, Splice, Replicate, Cipher, Surge, Spectacle,
  Addendum, Demonstrate, Conspire; Overload ships as an alt-cost.
- **Resource systems:** ✅ Energy ({E} pool + HUD chip; Kaladesh set; energy-gated
  mana abilities); ✅ Experience counters (`CounterType::Experience`); ✅ Poison/Toxic, Devotion, Ascend/city's blessing; ✅ Monarch;
  ✅ Day/Night (502.2 turn-based transition + Daybound/Nightbound DFC auto-flip
  + `EventKind::DayNightChanged` "day becomes night / night becomes day"
  triggers — Brimstone Vandal); ✅ Coven (`Predicate::CovenActive` — 3+ creatures
  with different powers; HUD "✸ coven" chip);
  ✅ **Ability-word conditions** (CR 207.2c — `Predicate::{ThresholdActive,
  MetalcraftActive, FerociousActive, HellbentActive, FormidableActive}`,
  PlayerView flags + shared HUD chips; `sets::decks::abilitywords` /`recent68`);
  ✅ **Descend** (LCI — `SelectionRequirement::ControllerDescend(n)` +
  `Predicate::{DescendActive,DescendedThisTurn}` count permanent cards in the
  graveyard / "descended this turn" per CR 700.11; `DynamicPt::
  PermanentCardsInControllerGraveyard` for fathomless descent; PlayerView
  `descend_count` + HUD "⛏ descend N" chip; `sets::lci` batch);
  ✅ **Speed / "Start your engines!"** (CR 702.179 — `Player.speed` 0–4,
  `Keyword::StartYourEngines`, life-loss increment, `Predicate::SpeedAtLeast`
  for "Max speed —"; DFT batch in `decks::recent`); ✅ Ring-bearer (CR 701.54,
  `decks::ltr`);
  ✅ **Commit a crime** (CR 700.13 — `EventKind::CommittedCrime` fires when you
  cast a spell / activate an ability targeting an opponent, their permanents/
  cards, or a spell they control; `Player.committed_crime_this_turn` +
  `Predicate::CommittedCrimeThisTurn`), ✅ **Pack tactics**
  (`Predicate::AttackedWithTotalPowerAtLeast`), ✅ **Outlaws**
  (`SelectionRequirement::IsOutlaw` + `Predicate::ControlsOutlaw`) — OTJ batch in
  `decks::recent20`. ✅ **Corrupted** (CR 702.166 — `Predicate::CorruptedActive`:
  an opponent has 3+ poison; ONE batch in `sets::one` — Apostle of Invasion,
  Bonepicker Skirge, Vivisection Evangelist, Sinew Dancer, Fleshless Gladiator).
- **Fading family:** ✅ Fading, Vanishing (`process_fading_vanishing`). Remaining:
  Parallax Dementia's steal-on-leave rider.
- **Older mechanics:** ✅ Soulshift, Epic, Umbra armor, Affinity, Entwine, Buyback,
  Miracle, Bloodrush, Unleash, Scavenge, Transmute, Bestow, Tribute, Offering
  (CR 702.48 — `AlternativeCost.offering` + `ManaCost::reduce_by_cost`; the
  Kamigawa Patron cycle), ✅ Recover (CR 702.58 — `shortcut::recover`: a
  `CreatureDied / FromYourGraveyard` trigger gating a `MayPay` that returns the
  card from the graveyard or exiles it; Coldsnap I/S in `decks::recent`).
  Spiritcraft "cast a Spirit or Arcane spell" triggers
  ride `SelectionRequirement::HasSpellSubtype` + `shortcut::spiritcraft`.
  ✅ Blight (CR 701.68 — `Effect::Blight`: put N -1/-1 counters on a creature
  you control; `WardCost::Blight` is the Ward—Blight variant — Auntie Ool,
  Blighted Blackthorn, TLA).
  ✅ Haunt (CR 702.55 — `Effect::HauntCreature` + `DelayedKind::
  WhenHauntedCreatureDies`: a dying creature / resolved I/S is exiled haunting a
  creature, firing its haunt body when that creature dies; Guildpact cycle in
  `catalog::sets::gpt`). ✅ Ripple (CR 702.20 — `Effect::Ripple` +
  `shortcut::ripple`: a cast trigger that reveals the top N, free-casts
  same-named copies, and bottoms the rest; Coldsnap Surging cards).

## Tier 5 — Mana & cost system

- ✅ **Typed spend restrictions / provenance riders** — `SpellKind` +
  `SpendRestriction` (Cavern of Souls, Power Depot). Remaining ⏳: per-source
  restrictions beyond these (filter lands).
- ✅ **Minimum-cost floor** (`StaticEffect::SpellCostFloor` via
  `apply_spell_cost_floor`, applied after every reduction — Trinisphere) and
  **cost-increase statics** (`extra_cost_for_spell` walks nine flavours plus
  `ColoredSpellTax` and the turn-scoped pool). Every cast path pays them
  (2026-09-28): zone casts, paid may-play grants and the half-card casts
  go through `game/cast_cost.rs` (`add_spell_taxes`, `half_spell_probe`).
- 🟡 **Conditional / additional costs** as a general modal layer. Card-intrinsic
  target-conditional reduction ships (`self_cost_reduction_if_target` — Ride's
  End's "{3} less if it targets a tapped permanent", generic-only / colored-pip
  safe). Board-state / per-turn-counter scaling reductions ship too
  (`self_cost_reduction_if_control` — Pearl of Wisdom;
  `StaticEffect::SelfCostReducedPer{Discard,CreatureAttacked}ThisTurn` — Hollow
  One, Search Party Captain). Source-power-scaled reduction of *other* spells
  ships (`StaticEffect::CostReductionBySourcePower` — Golden-Tail Trainer), and
  affinity-style `SelfCostReducedPerPermanentMatching` now honors board-state
  filters (Walking Skyscraper "per modified creature"). Remaining: per-mode
  Spree costs.
- ✅ **{X} in activated abilities** — `activate_ability` pays
  `mana_cost.with_x_value(x)` (Necropolis Fiend, Kasmina's `-X`). Remaining ⏳:
  **delve/convoke colored** contribution.
- ✅ **Snow-mana-only** (`ManaSymbol::Snow`, paid from the snow pool with
  `ManaError::InsufficientSnow`). Remaining ⏳: **mana-value-X** cost gates.

## Tier 6 — Combat fidelity

- ✅ **Damage assignment order** (Tier-1 #3) + **trample math** with
  multiple/deathtouch blockers — `default_damage_split` assigns lethal in
  order (deathtouch lethal = 1, CR 702.2e) and tramples the remainder
  (CR 510.1c/702.19g). Tests: `cr_702_2e_trample_deathtouch_*`,
  `cr_702_19g_*`.
- ✅ **Banding** (CR 509.2 / 510.1c / 702.22) — a banding blocker routes the
  attacker's combat-damage order + assignment to the *defending* player
  (Benalish Hero), and **attacking bands** ship:
  `GameAction::DeclareAttackersBanded` validates 702.22c/d, `attack_bands`
  persists the band (702.22e), removal from combat drops a member (702.22f),
  and a block on any member spreads across the band (702.22h). Surfaced to
  clients via `ClientView.attack_bands`. **"Bands with other [quality]"**
  ships as `Keyword::BandsWithOther(SelectionRequirement)` — band legality
  without plain banding (702.22d), the defender's damage division against a
  two-strong quality band (702.22j), the active player dividing a band
  blocker's damage (702.22k), and a payload-agnostic removal so "loses all
  'bands with other' abilities" (Shelkin Brownie, Tolaria) works.
- ✅ **Multiple combat phases** — `AdditionalCombatPhase` (Hellkite Charger) +
  post-main insertion (Relentless Assault). First-combat detection
  (`combat_phases_this_turn` + `Predicate::IsFirstCombatPhaseThisTurn`) gates
  "if it's the first combat phase" riders so extra combats don't loop (Genji
  Glove). **Additional end steps** (CR 500.7 — `Effect::AdditionalEndStep` +
  `end_steps_this_turn` + `Predicate::IsFirstEndStepThisTurn`; Y'shtola Rhul).
  Repeated phases are surfaced to UIs via `ClientView.extra_phase`.
- ✅ **"Whenever you attack"** (CR 508) — `EventKind::YouAttack` fires once per
  combat for the attacking player (not per-attacker), via `shortcut::on_you_attack`.
  Replaces the old `Attacks/YourControl + once_per_turn` approximation on
  Razorkin Hordecaller, Inti, Gut, Raffine, Most Valuable Slayer, Lionheart Glimmer.
- 🟡 **"Must/can't attack/block" restrictions** — `Keyword::{CantAttack,CantBlock,
  AttacksAlone,CantAttackAlone,MustBeBlocked,AllMustBlock,MustAttack,MustBlock}`, Goad;
  power-based evasion (`CantBeBlockedByPowerLess` — Formation Breaker;
  fixed-threshold `CantBeBlockedByPowerAtMost(n)` — Questing Beast);
  turn-scoped defender-bypass grant (`AttackDespiteDefenderThisTurn` — Krotiq
  Nestguard); count-gated attack+block (`CantAttackOrBlockUnlessYouControlCount`
  — Topiary Stomper's "unless you control seven or more lands", with `attack_only` / `block_only` facets
  (Lambholt Pacifist / Olog-hai Crusher), honored in combat, affordances, bot,
  and the legal-blocker gate); hand-size-gated
  (`CantAttackOrBlockUnlessHandSizeAtMost` — Hazoret), delirium-gated
  (`CantAttackOrBlockUnlessDelirium` — Patchwork Beastie), descend-gated
  (`CantAttackOrBlockUnlessDescend(n)` — The Ancient One, via `descend_count`)
  and cost-gated (CR 508.1g / 509.1d–f — `CantAttackOrBlockUnlessPay(n)`,
  Oppressive Rays; charged to the attacker's/blocker's own controller from the
  Propaganda tax pool). Open: granted must-attack with future-turn duration,
  multiplayer goad-target clause.
- ⏳ **Planeswalker / Battle as attack targets** UI + redirection.
- ✅ **Goad**, **Lure**, **Provoke**, **Ninjutsu swap**.
- ✅ **Multiplayer attack options** (CR 802 / 803) — `GameState.attack_option`
  picks between "every opponent is a defending player" (the Free-for-All
  default) and attack-left / attack-right, which narrow the legal defender to
  the nearest living opponent in that direction (a dead neighbour means no
  legal attack at all). Surfaced as `ClientView.attackable_players` and honored
  by the client's attacker-pick highlight. Tests `cr_802_*` / `cr_803_*`.
  ✅ **CR 801 limited range of influence** — a per-seat `range_of_influence`
  with a turn-start `range_matrix` snapshot (801.2/801.2c), enforced on
  attacks (801.3), targeting (801.4), activation (801.6) and effect fan-out
  (801.10); surfaced as `PlayerView.in_your_range`. ✅ **CR 809 Emperor**
  (`set_emperor_variant` — seating, 2/1 ranges, deploy creatures,
  adjacent-only attacks, a team falling with its emperor) and ✅ **CR 811
  Alternating Teams** (`set_alternating_teams`). Remaining ⏳: CR 807's
  rotating Grand Melee ranges.

## Tier 7 — UI / UX core (the Arena "feel" gap)

1. ✅ **Card-zoom hover preview** — `hover_card_preview` (flips side to avoid
   covering the card); Alt-hold drives the centered detailed peek, which shows
   both faces of a DFC side by side plus the catalog rules-text panel (the
   small preview flags DFCs with a "hold Alt" hint line).
2. ✅ **Stops / auto-yield config** — `auto_advance_p0` smart default + per-step
   Stop/Skip overrides on the phase chart (`StopConfig`), separate for your turns
   vs. opponents'.
3. 🟡 **Combat math / damage preview** — `combat_preview` projects life swing +
   dying creatures (first/double strike — incl. double strike's two damage steps
   for face/trample/lifelink — deathtouch spread, trample, protection),
   layer-aware, with planeswalker-target rows; the client HUD flags a projected
   life total ≤ 0 with a "☠ LETHAL" tag. Remaining: multi-blocker damage-order
   nuance.
4. ⏳ **Undo / mana-tap rollback** — undo un-committed taps before a spell locks in.
5. ✅ **Targeting arrows on the stack** — `draw_stack_arrows` (primary +
   additional-target slots; counter magic points at its spell).
6. ✅ **Hold-priority toggle** — `H` / "Auto-pass" flips
   `FastForward::manual_priority`. Shift-hold-after-your-spell ⏳.
7. ✅ **Stack visualization** — the stack renders as a visual zone; per-item
   respond/resolve affordances ⏳.
8. ✅ **Phase bar / step indicator** — left-edge chart, clickable stop markers,
   right-click "pass until this step".
9. 🟡 **Resolution-time decisions for humans** — via the stash-and-rerun suspend:
   ✅ ChooseModes, modal triggers, MayDo, DivideDamage, ChooseAmount, creature-type
   choices, seat-routed yes/no asks (rhystic, Tribute, Browbeat, MayPay),
   CommanderRedirect (yes/no modal), ChooseLegendToKeep (pick-one modal),
   CoinFlip/DieRoll (client-rolled "Flip"/"Roll dN" button) — every
   `DecisionWire` variant now has a client UI (the match is exhaustive, no
   wildcard, so new variants are compile errors instead of client freezes).
   Remaining ⏳: modal triggers with targeting modes, non-Bool
   opponent-owned picks.

## Tier 8 — UI / UX quality-of-life

- ✅ Browsable **graveyard / exile** zones (`V` toggles exile, with source
  annotations); library shows a count chip only.
- ✅ **Search / Scry / Surveil / Mulligan** picker UIs (top/bottom toggles, reorder
  buttons). Drag-and-drop reorder ⏳.
- ✅ **London mulligan** bottoming; Serum Powder gets its own button.
- ✅ **Floating life deltas**; per-turn life-history sparkline (`I` toggles a
  per-seat, one-column-per-turn panel — `game_ui::life_graph`).
- ✅ **Commander-damage HUD** (903.10a) — per-source `⚔ <cmdr> N/21` chip,
  amber→red near loss.
- ✅ **P/T + loyalty badges** — modified creatures get a floating `P/T` badge;
  planeswalkers always carry a `◆loyalty` badge (`systems/pt_label`).
- ⏳ **Hand sorting / auto-tap prefs / "play tapped land" prompt**.
- ✅ **Squad / Replicate pay-N stepper**; impending countdown badge; NameCard
  picker.
- ✅ **Reminder text & rules tooltips** — hover info panel from the catalog
  (type line, P/T, keyword reminders, oracle-ish ability panel).
- 🟡 **Hotkey legend** ✅ (F1 / `?`); remappable keys ⏳.
- 🟡 **Highlight legal plays** — `ClientView` carries castable/pitchable/kickable/spliceable
  hand, activatable permanents, legal attackers/blockers (step-aware). Remaining:
  per-target hint layers.
- ⏳ **Animations & SFX** polish; board-state pings/alerts.
- ✅ **Settings menu** (window/resolution/quality/gameplay, persisted);
  audio/accessibility tabs ⏳.
- ✅ **Battlefield organization** — identical tokens pile with ×N badges.

## Tier 9 — Multiplayer & social

- ✅ **Lobby / matchmaking** — LAN lobby browser (create/join/spectate, host bot
  add/remove). Remaining ⏳: join-by-code over internet, quick-match.
- ✅ **Reconnect / resume** — resume tokens + backoff retry + full snapshot
  restore + a "reconnecting (N/10)…" client banner; tokens persist to disk /
  localStorage so a crashed client gets a menu "Rejoin Last Match" button
  (cleared on clean exit / match end). A seat that drops while the rest of
  the table plays on gets the same 60 s (`RECONNECT_GRACE`), then concedes
  (CR 104.3a) — a pod used to wait on it forever, the rope being off by
  default; a non-reconnectable match (LAN host) concedes it at once. The
  table hears each step as a log line (`ServerMsg::Notice`).
- ✅ **Spectator mode** (read-only `ClientView` stream).
- ✅ **Player identity** — editable display name reaches every seat + log lines,
  persisted across launches.
- 🟡 **Chat** — free in-match chat ships (`T`), and lobby-phase chat relays
  to lobby members (same `T` input + a lobby panel). Remaining ⏳: emotes, mute.
- ✅ **Timers** — per-action rope (`CRAB_ACTION_TIMEOUT_SECS`) + per-game
  chess clock (`CRAB_CHESS_CLOCK_SECS`: per-seat match budget, flag fall
  concedes; `ServerMsg::Clock` + a client m:ss chip).
- ⏳ **Friends / invites / ratings / leaderboards**.
- ⏳ **Free-for-all politics** UI for 3+ player tables.

## Tier 10 — Formats & match structure

- ⏳ **Best-of-3 + sideboarding** flow.
- 🟡 **Deck legality validation** — size/copy/singleton/Commander-identity ✅,
  ban + restricted lists ✅ (`format::validate_deck`), companion deck
  restrictions ✅ (`format::companion_restriction_met`, CR 702.139c). Remaining:
  per-set legality pools (Standard rotation), Pauper rarity.
- ⏳ **More 60-card formats** (Modern/Pioneer/Legacy/Vintage/Pauper — mostly
  banlist/pool config).
- ⏳ **Limited match rules** (40-card, basic-land access).
- ⏳ **Multiplayer variants** (Planechase, Archenemy, Oathbreaker, Star, Emperor).
- ⏳ **Casual toggles** (free mulligans, vanguard).

## Tier 11 — Limited (draft / sealed)

- ✅/🟡 **Draft + cube** exist. Extend with:
- ⏳ **Sealed**, ⏳ **bot drafters** (signal/pick heuristics), ⏳ **draft variants**
  (Winston/Rochester/Grid/…), ⏳ **set-based draft**, ⏳ **draft replay / pick
  history**. ✅ **Deck export** — the deckbuilding screen's "Save Deck" writes
  the staged main+sideboard as importable decklist text to
  `<config_dir>/crabomination/decks/` (localStorage on wasm).

## Tier 12 — Deckbuilding & collection

- ⏳ **In-app deck builder** (search, curve view, legality, sample-hand).
- 🟡 **Import / export** — import ships (`decklist::parse_decklist`, Arena/MTGO
  text; the menu's decklist "From File" / "From Clipboard" load and validate, and a
  Commander lobby takes a pasted list — the web build's only way in). Names
  match past case, accents and typographic quotes (`decklist::fold_name`),
  and a list that can't play opens a report of **every** problem, each
  unknown card with the names it probably meant ("Lightnig Bolt", "Atraxa"),
  where the status line used to name four. Commander: the menu's deck
  picker (`deck_picker.rs`) chooses your stock deck and each bot's from the
  183 stock lists, with search and each deck's power tier (its measured win
  share under bot play, `pod::power`); a bot left on Random is dealt near
  your deck's power. Remaining ⏳: export, .dec/.cod, a warning
  for cards whose implementation is partial (no runtime list —
  INCOMPLETE_CARDS is prose).
- ⏳ **Deck stats** (curve, pips, type breakdown).
- ⏳ **Collection tracking**; ⏳ **Scryfall-like card search** over the catalog.

## Tier 13 — AI

Moved to `ML_NOTES.md` (size trigger): the bot/net gate history, the adopted
profiles and the documented dead ends, and — since 2026-08-23 — the encoder
and net follow-up list (belief-head recall, encoder v7, the static-anthem
P/T hole, the feature-occupancy precondition, the next-round candidates).

## Tier 14 — Replays, analysis & observability

- 🟡 **Action-log replay viewer** — the capture side ships: `CRAB_REPLAY_DIR`
  appends one JSONL replay per match (header/players, one line per broadcast
  event batch, footer). Remaining: the viewer.
- ✅ **Game history / match results persistence** — `CRAB_MATCH_LOG` appends
  one JSON line per finished match (lobby/bot/pair paths;
  `crabomination_server::history`); `CRAB_MATCH_LOG_MAX_BYTES` caps the live
  file and rotates it to `<path>.1`.
- ⏳ **Export game to shareable file** (formalize the audit-snapshot workflow).
- ⏳ **In-game "what happened" log filtering** (by player/zone/type).

## Tier 15 — Accessibility

- ⏳ **Colorblind-safe** indicators, **text scaling / high-contrast /
  reduced-motion**, **full keyboard play**, **screen-reader narration**, **"full
  control" mode** (never auto-skip).

## Tier 16 — Infra, correctness & content tooling

- ⏳ **Seeded / deterministic RNG** surfaced for reproducible games.
- ⏳ **Snapshot round-trip property tests** + **action-sequence fuzzing**.
- 🟡 **Crash-recovery / autosave** — a match that panics writes a `GameState`
  plus the panic message to `CRAB_CRASH_DUMP_DIR`
  (`crabomination_server::crash_dump`, atomic write + newest-N retention).
  The dumped state is the **live checkpoint**: the server hands every match a
  `SnapshotSink`, which the actor republishes after each accepted action, so a
  panic on turn 15 dumps turn 15 (falling back to the pre-match capture when
  nothing published). Remaining: resume-from-dump.
- ⏳ **Card-scripting DSL** to reduce catalog boilerplate.
- ⏳ **Set / Scryfall import pipeline** (`scripts/verify_cards.py` exists — extend).
- ⏳ **Card art / image pipeline**.
- ✅ **Rules-engine conformance suite** mapped to CR sections — `scripts/
  cr_coverage.py` generates `CR_COVERAGE.md` (section → title, subrules tested,
  test count, plus the untested-section gap list) from the `cr_<section>_` test
  names. Regenerate it after adding conformance tests; do not hand-edit.
- ✅ **Operator telemetry endpoint** — `CRAB_STATUS_BIND` HTTP `/healthz` +
  `/status` (uptime, rolling match stats, slot accounting).

---

## Suggested sequencing

0. **Next set to close.** **Mirage (MIR) is the live front** (`set_gaps.py mir`
   at 17, `sets::mir`–`mir5`); each of its last 17 is blocked on one primitive —
   CARD_BACKLOG.md → "Mirage residue" names them card by card. Coldsnap (CSP) is open
   in parallel. At zero (`set_gaps.py` empty): BLB, DSK, OTJ, EOE, FIN, CNS,
   CN2, MKM, the Odyssey, Onslaught (ONS / LGN / SCG), Mirrodin (MRD / DST /
   5DN), Kamigawa, Scars (incl. MBS, NPH) and Tempest (TMP / STH / EXO) blocks,
   Legends (`sets::leg`–`leg7`), Antiquities (`sets::atq`), Arabian Nights
   (`sets::arn`), The Dark (`sets::drk`/`drk2`), Homelands (`sets::hml`–`hml3`),
   Weatherlight (`sets::wth` + `wth2`, tests in `classic_sets/wth`) and Visions
   (`sets::vis` + `vis2`, tests in `classic_sets/vis`).
1. **Replacement-effect framework** (Tier-1 #1) — highest-leverage primitive still
   open.
2. **Card-zoom + stops/auto-yield + combat-math preview** (Tier-7 #1–3) — the trio
   that most closes the Arena "feel" gap.
3. **Best-of-3 + sideboard + deck legality** (Tier 10) — makes constructed
   competitive.
4. **Static-ability framework** — broad correctness wins. (Mana provenance
   shipped; see "Already shipped".)
5. **Smarter AI blocking** (Tier 13) — biggest single-player upgrade.
6. Then the **Tier-4 mechanic sweep** and **Tier-3 object-model** features, batch
   by batch.
7. **Replays, spectator, social, accessibility** as the product matures.

## Recently closed — index

Per-push prose lived here and violated the trackers' "no per-push changelogs"
rule; `git log -p -- FEATURE_ROADMAP.md` is the record. What closed, terse:

- **Classic sets closed**: Tempest block (TMP / STH / EXO), Weatherlight,
  Visions, Mirage (waves 1–7, `set_gaps.py mir` 275 → 17), Coldsnap opened
  (123 → 91), MKM closed.
- **Combat**: per-attacker block legality (`legal_block_targets` +
  `ClientView.legal_block_targets`); block chooser end to end
  (`ClientView.block_chooser`, `bot::forced_blocks` on a MustBlock board).
- **A card's face in its zone** (`CardInstance::face_view`, `GameState::face_view_of`):
  CR 709.3b / 709.4b / 715.3b / 702.103b / 202.3e — Adventure, Omen, split
  half, bestow and X spells on the stack; split cards / Rooms off it
  (`core_rules/cr_recent107`); a Room permanent's unlocked doors (CR 709.5).
- **Animation colors** (CR 105.3): `shortcut::colored_animation` — creature-lands,
  Genju, Restless / AFR / Urza's Legacy lands and Oko's Elk take their printed
  colors. **Cast from exile** (`Keyword::ExileCast`): Squee, Eternal Scourge.
- **CR conformance batches**: `core_rules/cr_recent87`–`cr_recent98` — CR
  120.8, 613.11 (`effective_max_hand_size`), 701.19a, 604.4, 611.2c, 509.1c,
  704.5m, 702.26c, 707.4, 116.2b/116.3, 717.2/717.4/717.5.
- **Targeting**: 125 of 164 silent target-walker gaps closed, the rest
  ratcheted by `core_rules/target_walkers`; reflexive-cost target walking
  (`MaySacrifice` / `MaySacrificeSource`); `resolution_causer`.
- **Affordances / client**: free-cast hand affordance + "FREE" chip,
  regeneration-shield chip, SOS Special Guests naming.
- **Build**: `crabomination_client` type-checks in cloud sessions.
- **Abilities**: CR 113.10b over every battlefield trigger walk — a stripped
  permanent fires no step, attack or combat-damage trigger
  (`GameState::stripped_permanents`).

Open work lives in the tiers above; residual per-set approximations live in
`CARD_BACKLOG.md`.
