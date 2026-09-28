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

The map for the next Commander run — **audited 2026-09-18 against the tree and
against the shipped CR text (now `crabomination/MagicCompRules_20260925.txt`; the 2026-09-25 edition replaced 2026-04-17 on 2026-09-28 — its 605.1a / 800.4i / battle changes are in), not
against this file's history**. Don't re-audit; update rows as they move.
Per-deck card completion lives in `DECK_FEATURES.md`.

### Format core (CR 903)
| Feature | State | Where |
|---|---|---|
| `Format::Commander`: 40 life, 100-card, singleton, command zone at start | ✅ | `format.rs` (`FormatRules`, `validate_commander_deck`), `game/mod.rs::apply_format`, `seat_commanders` |
| Commander designation persists across zones; a copy is not a commander | ✅ | `player.rs::Player::commanders` (`CardId`s, zone-independent); `GameState::is_commander`. A token/copy has its own `CardId`, so exclusion is by construction |
| Commander tax {2} per prior cast, as an additional cost | ✅ | `actions.rs::cast_from_command_zone` — added before `cost_reduction_for_spell` / `reduce_by_cost`, so reductions and the floor see it |
| CR 903.9a graveyard/exile return as an SBA (dies triggers fire first) | ✅ | `stack.rs::commander_zone_return_sba`, `commander_return_declined`. Waits for the resolution (CR 704.3): effect bodies sweep with `check_state_based_actions_mid_resolution`, and no return runs while a decision is pending — Danse Macabre reanimates a sacrificed commander (`cr_704_3_commander_return_waits_for_the_resolution`) |
| CR 903.9b hand/library return as a *replacement* | ✅ | `replacement.rs`, registered per commander by `seat_commanders`; `Decision::CommanderRedirect` |
| "Whenever your commander is put into the command zone" (both routes) | ✅ *since 2026-09-24* | `GameEvent`/`EventKind::CommanderPutIntoCommandZone`, queued by `note_commander_to_command_zone` from `commander_zone_return_sba` and `place_card_at_resolved_zone`; Myth Unbound (`cr_903_9_myth_unbound_draws_on_both_routes_home`) |
| Commander damage, 21 from one commander, per (commander, player) | ✅ | `game/mod.rs::commander_damage` + `record_commander_damage`; combat damage only (CR 903.10a) |
| CR 903.4 color identity: cost, rules text, indicator, back faces | ✅ | `color_identity.rs`; audited against Scryfall for every implemented card (3 known card gaps in the audit's `KNOWN_IDENTITY_GAPS`; a unit-variant static like `FiveColorAlternativeCost` is matched by name — Jodah) |
| Deck validation: singleton, identity subset, legal commander, ban list, **CR 903.5e no sideboard** | ✅ | `format.rs::validate_commander_deck` (CR 903.5a counts the commander), `commanders_may_pair`, `COMMANDER_BANNED` (Scryfall's list, 2026-09-18). ⚠ CR 903.5e is checked *here*, not in `FormatRules`: `validate_deck_refs` takes a flat card list (which is what lets 903.5a count the commander with the 99) and so never sees a sideboard. CR 903.5d falls out of the identity walk's `basic_land_identity` |
| CR 903.3 what a commander may *be*: legendary creature, **Vehicle**, or **Spacecraft with a power/toughness box** | ✅ | `format.rs::is_legal_commander` + `is_spacecraft_with_pt` (a station card's printed box is its band's `pt`, CR 721.2b). Shorikai / Parhelion II / The Seriema pass, The Eternity Elevator — a station card whose bands only add mana — does not. `CommanderDeckError::IllegalCommander` |
| CR 903.5d a card with a basic land type is bounded by the commander's identity | ✅ *by folding* | `color_identity.rs::basic_land_identity` adds the type's color to the card's identity rather than checking 903.5d separately. Same answer for the subset check, and it is what matches Scryfall's `color_identity` — which is what the whole-catalog audit ratchets against |
| The old mana-production restriction | n/a | removed from the CR years ago; correctly absent. ⚠ It is **not** CR 903.11 any more — that number now holds "traditional cards from outside the game cannot be brought into a Commander game" (903.11a adds the same-name and identity gates). Unreachable here: CR 903.5e leaves no sideboard, so `Effect::WishToHand` / `WishToLibrary` find nothing outside the game and fall through to the caster's own exile |
| "Any color in your commander's color identity" mana | ✅ | `ManaPayload::AnyColorInCommanderIdentity`, `GameState::commander_identity_colors`; Command Tower / Arcane Signet / Commander's Sphere / Path of Ancestry / Opal Palace. A colourless commander adds **no mana, and not {C}** (rulings 2020-11-10). ⚠ A seat with *no* commander keeps "any colour" where **CR 903.4f** says the quality is undefined and the ability does nothing — deliberate, for the cube pool's fixing, and what keeps the two-player traces byte-identical. Unreachable in a Commander game, where every seat has one |
| "If you control **a** commander" = any player's commander (CR 903.3) | ✅ *since 2026-09-23* | `Predicate::YouControlACommander` / `PlayerControlsACommander { who }` via `GameState::is_commander`; it read only your own designations, so a stolen commander never counted (Akroma's / Jeska's Will, Crimson Honor Guard) |
| "If you control a commander **as you cast** this spell" (the Will cycle, 12 cards) | ✅ *since 2026-09-27* | `CardData::cast_controlling_commander`, stamped in `actions.rs` beside entwine (CR 601.2b) and read by `Predicate::YouControlledACommanderAsCast`; a copy keeps it. Tests `core_rules::commander_cards::cr_601_2b_*` |
| Commander to hand from the command zone (Command Beacon) | ✅ | `Effect::CommanderToHand`, `game/effects/commander.rs` |
| A **melded** commander is still the commander (Gisela's ruling, CR 903.3 + 701.37) | ✅ *since 2026-09-24* | `GameState::commander_card_of` maps the melded permanent to its commander component: `is_commander`, `is_own_commander_object` ("if you control your commander") and the combat commander-damage tally read it, so Brisela's hits land on Gisela's card. Leaving splits `meld_parts` and only the commander card goes home. Tests `cmdr_gisela::a_melded_commander_is_still_the_commander` and `a_bounced_melded_commander_can_go_home` (CR 903.9b: the parts move one by one and the commander card's replacement applies); pod seat 33 plays it |
| A **merged** (mutated) commander is still the commander (CR 903.3c) | ✅ *since 2026-09-27* | `commander_card_of` / `is_own_commander_object` read `mutate_stack` beside `meld_parts`; leaving scatters the parts and only the commander card goes home. `cmdr_otrimi::cr_903_3c_a_merged_commander_is_still_the_commander` |
| Casting a commander **for mutate** from the command zone (CR 903.8 + 702.140a) | ✅ *since 2026-09-27* | `actions.rs::cast_mutate` routes a command-zone commander through `cast_from_command_zone`, tax on top; the bot offers it (`server/mutate.rs`, pods only). `cmdr_otrimi::cr_903_8_a_commander_mutates_from_the_command_zone_under_its_tax` |
| CR 106.6 mana provenance — "when that mana is spent to …" | ✅ | `SpendRestriction::{CommanderTypeScry, CommanderCastCounters}` + `is_rider()`; `game/effects/commander.rs::{note_commander_mana_riders, push_commander_mana_scry, spell_kind_for}`; Path of Ancestry / Opal Palace. CR 106.6a's per-mana count included — `spent_restrictions` carries `(restriction, pips)`. `SpendRestriction::allows` answers the rider half once by short-circuiting on `is_rider()`, and the bot's `available_mana` counts floating rider mana (`ManaPool::rider_amount`) — it read the restricted bucket as unspendable wholesale, so a seat holding Path of Ancestry's green reported `by_color [0,0,0,0,0]` |
| CR 903.8 a commander tax paid in **life** (Liesa, Shroud of Dusk) | ✅ *since 2026-09-26* | `StaticEffect::CommanderTaxPaidInLife`, read by `actions.rs::commander_tax_for` on both command-zone cast paths; CR 119.4 gates the life (an alternative cost's life included). The bot skips a life tax that leaves it at 4 or less (`cr_903_8_liesa_pays_her_commander_tax_in_life`) |
| Casting a commander for an **alternative** cost from the command zone | ✅ | `actions.rs::cast_spell_alternative_from` over `AltCastZone` — one body for both zones; CR 903.8's tax is pushed ahead of the reductions. `GameAction::CastFromCommandZone { alternative, pitch_card }` |
| "Can be your commander" on a non-creature (planeswalker commanders) | ✅ | `CardDefinition::can_be_commander`, read by `validate_commander_deck`; Freyalise, Llanowar's Fury in `sets::cmdr`; **all 22 Commander-legal "can be your commander" cards are implemented as of 2026-09-28** (`01PdWYgx` added Jeska, Tevesh Szat, Sivitri, Elminster and Tasha in `decks::cmdr_legends2`). **Piloted, not only validated, since 2026-09-19**: `pod::decks::FREYALISE_MAIN` is the seventh target deck and `cr_903_3a_a_planeswalker_commander_seat_plays_a_pod_game` runs it. Two things fall out of a planeswalker commander and both hold — it is recast from the command zone under the CR 903.8 tax like any other, and its (commander, player) damage tally stays at zero all game, because CR 903.10a counts **combat** damage and a planeswalker deals none |

### Multiplayer foundation (CR 800-series)
| Feature | State | Where |
|---|---|---|
| CR 800.4a a departed seat is not a player | ✅ | `GameState::living_seats` beside `seats_in_turn_order_from` and `resolve_players`; `default_hostile_opponent` for the ranked "an opponent". `scripts/audit_seat_walks.py` gates **two** columns — a loop that hands a dead seat a *question* (votes, ballots, offers) and a walk that hands it a *role* (`LowestLife` was always the departed seat; nine open-coded "first opponent" fallbacks, three of them picking a defending player). Both 0 |
| CR 506.2 the **defending player** as a filter | ✅ | `SelectionRequirement::ControlledByDefendingPlayer` + `defending_player_in_combat` (`effects/eval.rs`), which `PlayerRef::DefendingPlayer` also ends in so the two cannot disagree. 32 cards printed the clause and were modelled `ControlledByOpponent` — exact in a duel, one seat of several in a pod; `scripts/audit_defending_player.py` is the ratchet at 0/32. The clause is not always on the attacker: an instant cast in combat, an Aura/Equipment whose host attacks, and a trigger firing off another creature's attack all resolve through the host-then-single-defender fallbacks, and a post-teardown trigger (Kusari-Gama) reads the damaged blocker's controller instead |
| N seats (2..N), turn rotation, APNAP ordering | ✅ | `game/mod.rs`, `multi_player_game`; tests in `core_rules/multiplayer.rs` (76) and `cr_801.rs` |
| Attack any opponent / their planeswalkers, per-attacker defender | ✅ | `game/combat.rs`. The bot's defender pick (`bot.rs::attack_target_player`) falls back to the most hostile *attackable* seat (`pod_attack::attackable_or`, CR 508.1a): under Mystic Barrier a barred pick declared nothing and a 947-creature board never attacked |
| CR 601.2c "for each opponent, … target X *that player* controls" | ✅ | `Effect::ForEachOpponentTarget` — a constraint on the chosen set (one target per controller, capped at the opponent count), enforced by the auto-target picker, the cast path and resolution. The five Primordials, Grasp of Fate, Omega, Tempted by the Oriq; tests in `core_rules/per_opponent_targets.rs` |
| CR 603.2c "whenever **one or more** … deal combat damage to a player" | ✅ | `EventSpec::once_per_batch` plus `combat.rs::BatchSubject` — the per-attacker walk collapses to one fire **per damaged player**, so an alpha strike across a pod is one event a defending seat, not one for the step. Ratcheted over the catalog by `catalog_registration::every_batched_damage_trigger_fires_once_a_batch` (three accepted spellings: the flag, `once_per_turn`, or a `FromYourGraveyard` scope the graveyard walk already dedupes). Malcolm, Keen-Eyed Navigator's "a Treasure for each opponent dealt damage" is exactly this count |
| CR 603.2c "whenever one or more" on the other event kinds (exile, discard, sacrifice, tap, attack a player / a planeswalker) | ✅ | `EventSpec::once_per_batch` on the dispatcher, the defender-side attack walk and the `OpponentOfYoursAttacked` walk (Jolene); `batch_counts_subjects` for "for each card discarded this way" (Captain Howler). `scripts/audit_one_or_more.py --gate` reads the printed wording against the flag (0) |
| CR 400.7 a card leaving a graveyard is always reported | ✅ | `note_left_graveyard` / `note_exiled_from_graveyard` / `note_returned_to_hand_from_graveyard` at every graveyard exit (~40 sites were silent: hop casts, delve and exile costs, shuffles, the commander's return to the command zone) — "cards leave your graveyard", "put into exile from a graveyard" and the per-turn tally see them all A card exiled **from a library** is reported too (2026-09-28): `EventKind::CardExiledFrom(exile_from::{BATTLEFIELD, GRAVEYARD, LIBRARY})`, `note_exiled_from_library` (Laelia). |
| CR 511.3 combatants stay in combat until the end of combat step **ends** | ✅ *since 2026-09-23* | `combat.rs::remove_all_from_combat`, run by `advance_step` leaving `EndCombat`; `combat_damage_dealt()` keeps a repeated `resolve_combat` a no-op. It used to tear combat down as regular damage was dealt |
| CR 103.5c a multiplayer game's first mulligan is free | ✅ *since 2026-09-27* | `GameState::mulligan_cards_owed`, read by the Keep path and the bot's keep heuristics. CR 103.8c (no first-turn draw skip past two seats) is `skip_first_draw`. `core_rules::multiplayer::cr_103_5c_first_multiplayer_mulligan_is_free` |
| CR 800.1 — how many players | ✅ *2..=64* | `game::MAX_SEATS` (every per-seat mask is a `u64`), asserted in `GameState::new`; `bot_ladder` clamps `--seats`/`--pod-decks` (`core_rules::multiplayer::cr_800_1_a_game_holds_at_most_max_seats_players`) |
| CR 508.1d "during [player]'s next turn, creatures they control attack [this] if able" | ✅ *since 2026-09-24* | `Effect::LureCreaturesToSourceNextTurn` + `PlayerCold.attack_lure`, `game/attack_lure.rs`; enforced in `combat.rs`'s declaration, honoured by the bot's `restore_forced_attackers` and target pass (Gideon Jura; `recent_b::cmdr_isperia`) |
| CR 506.3 "attacking **you**" / "attacks **you**" is one seat's attackers | ✅ *since 2026-09-27* | `SelectionRequirement::{IsAttackingYou, IsAttackingYouOrYourPlaneswalker}` and the defender-side `EventScope::ControllerAttackedByOpponent`; nine cards read any attacker (Soul Snare, Hunting Kavu, Flash Foliage, Watchdog, Qasali Ambusher, Briar Patch, Barbed Foliage, Garruk's emblem). `scripts/audit_attacking_you.py --gate` (0 open, 37 cards) The defender-side TRIGGER scopes split the same way (2026-09-28): `EventScope::ControllerAttackedByOpponent` is "you or a planeswalker you control", `ControllerAttackedDirectlyByOpponent` a bare "attacks you", `ControllerPlaneswalkerAttackedByOpponent` the planeswalker only — `scripts/audit_attack_scope.py --gate`. |
| CR 503.1 / 513.1 "**each** upkeep / end step" is every seat's step; "another creature" is anyone's | ✅ *since 2026-09-27* | For `StepBegins`, `YourControl` / `ActivePlayer` / `SelfSource` all mean the controller's turn, so a printed "each upkeep" scoped that way fired 1 turn in N (Tendershoot Dryad, Séance Board, Joined Researchers; Manaform Hellkite's "next end step"; Instill Furor's "your" read as every turn). Enters / dies / attacks triggers printed over anyone's creature shipped as `AnotherOfYours` (Reaper of the Wilds, Wirewood Hivemaster, Najeela, Skyboon Evangelist). `scripts/audit_step_scope.py --gate` over `dump_cards --grep StepBegins` (0 rows; 5 filtered rows checked) |
| CR 508.1 "whenever **an opponent** attacks" (`YouAttack` / `OpponentControl`) | ✅ *since 2026-09-27* | `combat.rs` — the attack walk admitted only the attacker's own triggers and `AnyPlayer` observers (Tahngarth, First Mate's ability was dead); `JoinCombatAttacking` joins an attack its new controller already makes |
| CR 509.1a — only a defending player declares blocks | ✅ | `combat.rs::may_declare_blocks` |
| CR 506.3 / 508.1b "attacks a player / one of your opponents" is not a planeswalker attack | ✅ *since 2026-09-27* | `SelectionRequirement::IsAttackingOpponentPlayer` (the player) vs `IsAttackingAnOpponent` (player or their planeswalker — only Gahiji and Roar of Resistance print that); nine cards moved (Blast-Furnace Hellkite, Scriv, Skyboon Evangelist, Death Kiss, Martial Impetus, Genestealer Locus, Neyali, Demonic Covenant, Bitter Work). `Predicate::PlayerAttackedByMatching` names "those players" of a batched attack (Zurzoth); `Effect::CantBeBlockedByPlayer` scopes an evasion grant to one seat (The Black Gate) |
| CR 506.2 "attacks **you**" is a different count from "attacks" | ✅ | `Predicate::AttackedDefenderWithCountAtLeast { who, defender, at_least, include_planeswalkers }`. The undirected `AttackedWithCountAtLeast` counts a player's whole declaration wherever it is pointed, which is the same question only at two seats; Mangara, the Diplomat and Trouble in Pairs both gate on two or more attackers aimed at *you* and would otherwise fire off an opponent swinging at a third player. `include_planeswalkers` is a printed difference between those two cards, not a knob — a creature attacking your planeswalker is attacking the planeswalker. ⚠ Reads `GameState.attacking`, which is filled *during* the declaration, so on a `ControllerAttackedByOpponent` trigger the gate goes inside the effect as an `Effect::If`, not in the `EventSpec` filter |
| CR 603.2 a defender-side attack trigger's own condition | ✅ *since 2026-09-19* | `combat.rs` — the `ControllerAttackedByOpponent` walk was the one of five that built its listeners without `t.event.filter`, so a `.with_filter` on such a trigger compiled and was never read (Reveille Squad's printed "if this creature is untapped"). ENGINE_BACKLOG's fiftieth find |
| CR 614 "if an opponent would begin an extra turn, they skip it instead" | ✅ | `StaticEffect::OpponentsSkipExtraTurns`, read at the one place `Player.extra_turns` is consumed (`stack.rs::end_turn`). The charge is **spent** even though the turn is skipped — a replacement on *beginning* the turn, and leaving it banked makes one Time Warp re-offer the same turn at every pass. Trouble in Pairs |
| CR 121.2a a draw doubler with a printed exception | ✅ | `StaticEffect::ControllerDrawsDoubledExceptFirstEachDrawStep` — Alhammarret's Archive / Teferi's Ageless Insight's "except the first one you draw in each of **your** draw steps", read off `Player::cards_drawn_this_step`. Distinct from `ControllerDrawsDoubled` (Thought Reflection), whose doc used to name the Archive by mistake |
| "For **each** colour among permanents you control, add one mana of that colour" (Vivid) | ✅ | `ManaPayload::OneOfEachColorAmongYourPermanents` — Bloom Tender, Faeburrow Elder. ⚠ Distinct from `AnyColorAmongYourPermanents` (Meteor Crater), which reads the same set and adds **one** mana chosen from it; the two agree exactly at one colour, so a mono-colour test cannot separate them. The union walk is shared (`GameState::colors_among_your_permanents`) |
| CR 400.7 — a blink is a new object | 🟡 *in every respect a card can read* | `Effect::ExileAndReturnToOwner` moves the same instance out and back: tapped state, counters, attachments, summoning sickness and damage all reset, but the **`CardId` survives**. Anything keyed on the id (a delayed trigger, an `exiled_with` link) still matches where the rules say it lost track. Unobservable on all five shipped users, which are friendly self-blinks; ENGINE_BACKLOG carries it. ✅ *2026-09-27*: a **captured** delayed return (`Effect::DelayUntilWithCapture`) now stamps the card's `battlefield_timestamp` and runs under `Predicate::TargetIsCapturedObject`, so a card that came back and died again is not returned by its old trigger (Liliana's emblem looped a Phantasmal Image to the action cap; `cr_recent104::cr_400_7_a_stale_delayed_return_*`) ✅ *2026-09-28*: keywords / triggers a `Duration::Permanent` grant baked into the definition, and a permanent "loses [keyword]", end with the object (`bake_grant`; row below). ✅ *2026-09-28*: a TIMED trigger grant (`granted_triggers_timed`) carries the object's `battlefield_timestamp` (`GrantedTrigger::stamp`), so a Rabid Attack / Feign Death grant fires on the object's death and not after a same-turn return (`cr_recent106::cr_400_7_a_timed_trigger_grant_ends_with_the_object`) |
| CR 614.2 a damage multiplier that is not a power of two | ✅ | `StaticEffect::MultiplyDamageFromYourSources { factor }` — Fiery Emancipation / City on Fire, both ×3. The funnel accumulates *doublings* (`amount << doublers >> halvers`), so a triple rides its own accumulator and composes by multiplication (two of them is ×9). ⚠ A new damage-scaling static must also be listed in `static_effect_scales_damage`, or the `scale_damage_to` presence gate skips it silently; its `debug_assert_eq!` is what catches that in tests |
| CR 702.16 protection from a set the *game state* owns | ✅ | `Keyword::ProtectionFromColorsOutsideCommanderIdentity` — Commander's Plate's "each color that's not in your commander's color identity", the complement of a colour set no `SelectionRequirement` can name. Four separate gates (the `ProtectionKind` damage/equip/aura funnel, ability targeting, the cast-time spell gate, blocking), each with its own test. Reads `GameState::commander_identity_set`, the **raw** identity — `commander_identity_colors`' all-five fallback would mean no protection at all |
| CR 800.4a a player leaving: objects, stack items, control effects, the command zone, combat, a pending ask | ✅ | `stack.rs::objects_leave_with_player`, driven for **every** seat that is out by `Player::left_game` rather than by the loss SBA's own `newly_eliminated` list. The five arms that set `eliminated` directly (an unpaid Pact, `Effect::LoseGame`, a win-the-game effect, the graveyard-exile damage replacement) never ran the pass, and the sweep's `if eliminated { continue }` skipped them for ever ✅ *2026-09-28*: `pod::play_pod_game` takes `CRAB_POD_CONCEDE=<n>` — a seat concedes (CR 104.3a) at a random point, n per 10,000 actions, off its own seed; a peek answered after its library left and a concession that read as a draw (CR 104.2a) fixed from it (`cr_800_4a_pods_survive_concessions_at_arbitrary_points`) |
| CR 800.4a (closing) / 800.4j priority never rests on a seat that left | ✅ | `mod.rs::priority_recipient`, read by `give_priority_to_active` and the untap-entry seed; `objects_leave_with_player` moves the grant already made. The turn itself continues — `active_player_idx` is left alone, which is what 800.4j says |
| CR 800.4a — a payer who leaves mid-payment takes their spell off the stack | ✅ *since 2026-09-28* | `effects/mod.rs::stack_pos_of` re-finds the countered item after an "unless [they] pay" cost (Ward, `CounterUnlessPaid`) instead of trusting the pre-payment index — an 8-seat pod panicked when City of Brass took a payer's last life. Auto-tap refuses a source that would kill its payer at ≤ 3 life (`actions.rs::mana_source_self_harm`). Tests `core_rules::multiplayer::*_payer_leaving_mid_payment_*` |
| CR 800.4a + 610.3 — a departed player's "until this leaves" exiles end; their permanents' leaving triggers others' leave abilities | ✅ *since 2026-09-28* | `stack.rs::objects_leave_with_player` runs `note_left_without_dying` for each departing permanent (Twilight Drover), then `return_linked_exiles` / `phase_in_held_by` after the seat's own cards are gone (Cast Out's ruling). An old-style "When this leaves the battlefield, return" is a trigger the departed player can't use: `StaticEffect::ExileReturnIsLeaveTrigger` (21 cards by wording) keeps those cards exiled. `concede` now dispatches its events. |
| CR 400.7 — a permanent grant ends with the object | ✅ *since 2026-09-28* | `CardInstance::bake_grant` / `revert_baked_grants` (card.rs), restored in `movement.rs::on_left_battlefield`: `Duration::Permanent` keyword and trigger grants rode the card into its next zone. Serra Paragon's exile rider rides it. Test `core_rules::cr_recent106::cr_400_7_permanent_grants_end_when_the_object_leaves` |
| CR 800.4b no token, and no control change, for a departed player | ✅ | `mod.rs::mint_token_with_counters`, `mod.rs::change_control` |
| CR 800.4d a departed player's triggered abilities aren't put on the stack | ✅ | `mod.rs::push_pending_trigger` (the one funnel for step / event / delayed pushes); `objects_leave_with_player` drops the `delayed_triggers` behind it |
| CR 800.4e no combat damage is assigned to a seat that has left | ✅ *by 800.4a* | `stack.rs::objects_leave_with_player` removes every attacker whose defending player left, so the reachable shape — the first-strike sub-step kills the defender and the regular sub-step follows — assigns nothing. Test `cr_800_4e_no_combat_damage_is_assigned_to_a_seat_that_left_mid_combat` |
| CR 800.4k a departed seat's turn does not begin | ✅ | `mod.rs::next_alive_seat`, read by `stack.rs::end_turn`. Test `cr_800_4k_a_departed_seats_turn_does_not_begin` |
| CR 800.4c a control effect ending with the default controller gone | ✅ | `mod.rs::revert_temporary_control` — the reversion site, not `objects_leave_with_player`: 800.4a's revert only touches permanents the departing seat controls *at that moment*, and in the three-deep shape (A owns X, B takes it permanently, C takes it until end of turn, B leaves) it controls none. `change_control` refuses the move under 800.4b and returns `None`, so without this C simply kept X for the rest of the game; now it is exiled. 800.4c's "no other effect giving control to another player" is asked of the **whole** registry rather than of the entries already processed, so it does not depend on registration order |
| CR 800.4m "until that player's next turn" ends when that turn *would* have begun | ✅ | `mod.rs::departed_seats_skipped_into_this_turn` → the Untap arm's expiry in `stack.rs`. Covers `UntilYourNextTurn`, `UntilYourNextUpkeep` (no upkeep of theirs is left) and CR 615's damage locks |
| CR 800.4f/g/h a choice owed by a departed player | ✅ | `game/departed.rs` — `route_ask` at the six `ask_seat_*` helpers. **800.4f**: a cost, or whether to pay one, is not paid and nobody is asked (`OptionalKind::is_cost`, the classifier the card already declares for the headless policy). **800.4g**: any other choice is re-seated by the object's controller — another *opponent* (`default_hostile_opponent`) where the departed chooser was one, the controller otherwise. **800.4h** needs no arm: every rule-required ask the engine suspends on (mulligan, cleanup discard, combat-damage order) names the asking seat's own cards, which 800.4a has already removed, so `objects_leave_with_player` still drops one. ⚠ The *pending* case is still a drop rather than a re-seat: an ask already materialized when its seat leaves needs the legal set re-derived against the post-departure board, which is a different capability. Rare now that no ask is posed to a departed seat in the first place |
| CR 800.4a — a seat that has left is not one of "each player" | ✅ | `mod.rs::seats_in_turn_order_from` (live seats in turn order from a given seat — a printed "starting with you, each player …", and `next_alive_seat` for "the player to their left") plus `resolve_players(&PlayerRef::EachPlayer, ctx)`, which is CR 101.4's order *and* the liveness filter. Six hand-written seat-index walks did neither; two were live bugs — `Effect::Vote` counted a departed seat's ballot (CR 800.4g re-seated its ask onto a live opponent, who answered, so nothing ever stalled) and Grenzo's Rebuttal aimed "the player to their left" at a board 800.4a had emptied. Ratcheted by `scripts/audit_seat_walks.py` (**0**), whose scope is the shape: a walk is a finding only when its body *asks* or *accumulates one entry per seat*. ENGINE_BACKLOG's forty-seventh find ✅ *2026-09-28*: `apnap_sort` orders seats but never filters them, and fifteen fan-outs fed it `0..players.len()` (Truce, Ice Cave, the number-naming duels) — they take `living_seats()`; `audit_seat_walks.py` flags the shape (test `cr_800_4a_apnap_fan_outs_skip_a_departed_seat`). A departed seat's emblems leave with it (CR 114.2); `controllers_next_turn_number` walks live seats (`cr_800_4a_your_next_turn_skips_a_departed_seats_turn`) |
| CR 800.4 — no ask is ever posed to a seat that has left | ✅ | `departed.rs::seat_prompts` (`wants_ui && is_alive`), the one predicate behind `seat_suspends` and every suspend site. It replaced 102 hand-written `players[x].wants_ui` reads, none of which asked whether the seat was in the game; four-seat pods posed a tribute question to a departed seat in 4.4 % of games. Audited on every pod game by a `debug_assert!` in `pod::play_one_pod_game` |
| Which opponent an open choice aims at, at N > 2 | ✅ | `mod.rs::default_hostile_opponent` (scored by `hostile_opponent_score`; the bot's face attackers past a kill spill to the next-ranked seat, `server/pod_attack.rs`, CR 508.1b) — one ranked answer, shared by `bot.rs::attack_target_player` (the defender) and `targeting.rs` (an auto-filled "target opponent"). CR 903.10a commander race, then lowest effective life, then fewest untapped blockers; ties by seat index, so a fixed seed reproduces the run. One candidate in a duel, so the 1v1 answer and the golden traces are unchanged. The two positional picks it replaces both named seats that had already left the game, and the bot's named a *teammate* at 2HG. The auto-targeter's half was wired only once the gate audit's forty-second find closed (it was the bot's mana *estimate*, not the engine — see the CR 106.6 row); `first_alive_opponent_of` had no caller left and is gone. Aggregate over 32,000 pod games a side: identical at two seats, turns/game -0.06/0.00 at three, -0.24/-0.17 at four, -0.37/-0.43 at five, 100 % decided and zero stalls on both |
| Free-for-all last-player-standing, simultaneous-loss draw | ✅ | `team.rs`, `stack.rs` |
| CR 104.3a + 119.4 — a seat kept in the game below 0 life ("can't lose": Herald of Eternal Dawn, Platinum Angel) still casts and activates | ✅ *since 2026-09-28* | `game/cast_cost.rs::life_payable` — paying 0 life is always legal; every life gate (hand cast, zone casts, Phyrexian retry, activations, equip, attack taxes) reads it. A 3-seat pod sat 663 turns at the action cap: two Heralds, both seats below −10,000, Swords to Plowshares refused `InsufficientLife`. Tests `core_rules::multiplayer::cr_119_4_*`, `bot_removes_the_permanent_keeping_a_dead_seat_alive` |
| CR 603.4 — a trigger condition naming "that player" reads the event being checked | ✅ *since 2026-09-28* | `EffectContext::event_player` (`with_event_player`: a player subject, else the event's actor) at the dispatch pre-checks, the push check and the cast-trigger walk; `PlayerRef::TriggerEventPlayer` prefers it over the resolution scratch, which held the LAST resolution's seat. Pain Distributor / Mind's Dilation ("a player casts THEIR first spell") at N > 2. Tests `core_rules::multiplayer::cr_603_4_*`, `cmdr_gimbal::cr_603_4_*` |
| CR 508.5 — "defending player" outlives the attacked planeswalker / battle | ✅ *since 2026-09-28* | `GameState::attacked_permanent_defenders` records each attacked planeswalker / battle's defender as the attack is made (`note_attack_defender` at every `attacking` push; cleared with it); `defender_for` falls back to it, so a Storm the Citadel'd creature that killed Chandra still destroys her controller's artifact. |
| CR 601.2c — an optional removal slot declines its own side | ✅ *since 2026-09-28* | `targeting.rs::declines_own_side_pick` (`Effect::permanent_slot_is_hostile`, `SelectionRequirement::excludes_opponents_side`) at the picker's single entry point. |
| CR 121.2 / 601.2 — "an opponent draws / casts THEIR Nth … each turn" is per opponent | ✅ *since 2026-09-28* | `EventKind::NthCardDrawnThisTurn(n)` or the caster's own `SpellsCastThisTurn`, never `once_per_turn` (one fire for the whole table) or the active player's tally: Faerie Mastermind, Erayo's Essence, The Unagi. `scripts/audit_their_nth.py --gate` (0 rows) |
| CR 800.4i last-known information about a departed player | ⚠ *by unreachability* | Not modelled: a departed seat's zones are emptied by `objects_leave_with_player`, so a count derived from them reads 0 rather than the last known value. Near-unreachable — `resolve_players_unranged` filters `is_alive()` out of **every** fan-out (`EachPlayer`, `EachOpponent`, `EachOpponentExceptTriggerer`, `OpponentsWhoVotedDifferently`), so only an effect naming a *specific* departed seat could see it. 800.4i's second sentence (actions a departed player took) already works: `spell_names_cast_this_turn` and the rest live on `PlayerData` and are not cleared. Its third (2026-09-25 text) — a departed player's "last turn" actions count only until their next turn would have begun — is `stack.rs`'s rotation clearing `attacked_players_this_turn` for every departed seat it walks past (Avenge; `core_rules::multiplayer::cr_800_4i_…`) |
| CR 101.4 a per-player fan-out finishes when a body **suspends** | ✅ | `effects/mod.rs::splice_after_suspend` — a suspending body parks only its own remaining effect, so the loop around it used to abandon every seat it had not reached. Invisible in a duel (one iteration), three quarters of a four-seat pod. Twenty arms taken; the tail has to name its seats **inside** the effect (`PlayerRef::Seat(q)`) because a parked continuation resumes under the stack item's context. Ratcheted by `scripts/audit_loop_splice.py` (13/13/0, plus a staleness half that fails on an allowlist entry whose site is gone). `Effect::BindScratch` is the sibling pin for state that lives on `GameState` rather than in the context — the ballot a `VoteTally::PerVote` run belongs to, a results-table arm's die face — and closed `Vote`'s `PerVote` half and `RollDie`. The sequential *pair* (`run_piles_then_clear`) is the same defect without a loop and is closed by its splice alone. The arms still open are in ENGINE_BACKLOG's forty-third find |
| Multiplayer mulligan, no first-turn draw skip at 3+ | ✅ | `core_rules/multiplayer.rs` |
| A printed "**choose an opponent**" clause resolves to ONE seat | ✅ | `PlayerRef::HostileOpponent` → `mod.rs::default_hostile_opponent`. Nineteen catalog sites handed a **fan-out** ref to an effect arm that resolves `who` through the singular `resolve_player`, which answers with the first seat of the set and drops the rest — exact in a duel, seat order deciding the controller's choice in a pod, and for five of them ("each opponent sacrifices / discards / may scry") only ONE opponent was reached at all. `Effect::EachPlayerDoes` is the fan-out those arms cannot do for themselves. Ratcheted by `scripts/audit_singular_fanout.py`, which pairs the 102 singular-resolving arms with the catalog sites that feed one: **19 → 2**, both allowlisted with their reason. ENGINE_BACKLOG's fifty-first find |
| "Choose an opponent" made at once — Gift (CR 702.174a), a Siege's protector (310.11a), Tribute (702.104a), Demonstrate (702.144a) | ✅ *since 2026-09-26* | `effects/choose_player.rs::choose_opponent_at_once` — a non-suspending seat ballot (headless pick: the opponent with the fewest creatures), shared by the four; each used to take the lowest-numbered opponent (or, for Gift, all of them). Gift bodies read `PlayerRef::ChosenPlayerOfSource`. Tests `multiplayer::cr_702_174a_*`, `cr_310_11a_*`, `cr_702_104a_*`, `cr_702_144a_*` |
| "An opponent chooses / gains / of your choice" without a target — Murmurs from Beyond, Development, the opponent-split search, the two opponent-picks-a-card arms, the OpponentGainsLife splice cost, Khârn the Betrayer | ✅ *since 2026-09-26* | the same `choose_opponent_at_once` ballot; each took the lowest seat (Khârn: the next in turn order). Tests `multiplayer::cr_601_2h_splice_*`, `cmdr_abaddon::cr_615_kharn_goes_to_*` |
| "Choose three, repeats allowed" chosen as cast (CR 700.2d) — the seven Confluences, Profane / Silumgar's Command, Saheeli's Artistry | ✅ *since 2026-09-26* | `Effect::ChooseModesCast` (cast via `GameAction::CastSpellSpree`, a target per targeted instance); were `ChooseN` with a fixed default. The bot counters with a modal counter mode (`modal_counter_cast`) and fills X. Doomsday Confluence (X modes) still chooses at resolution |
| CR 704.5w / 704.5x a battle's protector left, or a stolen Siege protects itself | ✅ *since 2026-09-26* | `choose_player.rs::reseat_battle_protectors`, run after the loss SBA's departures in `stack.rs` |
| CR 702.141a each encore token attacks *its own* opponent | ✅ *since 2026-09-26* | `effects/mod.rs` `Effect::EncoreTokens` — `chosen_player` + `Keyword::MustAttackChosenPlayer` (the Raving Dead requirement) until end of turn; the bot's `chosen_attack_target` reads the computed keyword, so a granted one is aimed too |
| CR 508.1 "whenever a player attacks" | ✅ *since 2026-09-26* | `EventScope::AnyPlayerAttacks`, dispatched in `combat.rs` once per declaration (attacking player in `Target(0)`); Mirkwood Trapper |
| CR 726.4 the heir to a departed holder's initiative *takes* it (CR 726.2 venture) | ✅ *since 2026-09-26* | `stack.rs::objects_leave_with_player` → `take_initiative` |
| CR 605.3b auto-tap never leaves a mana ability's color prompt pending | ✅ *since 2026-09-26* | `actions.rs::auto_tap_for_cost_inner` — the generic-pip loop forces a prompting seat's any-color source synchronous, like the colored loop. It used to add nothing and orphan the prompt; answered later, the prompt minted mana from an untapped land (a 400-cast ward loop at seven seats) |
| CR 903.8 a *granted* alternative cost for a commander cast from the command zone | ✅ *since 2026-09-26* | `actions.rs::effective_alternative_cost_in` offers Rooftop Storm's {0} (`CastFilteredSpellsFree`) at `AltCastZone::Command`, tax on top; the bot's command-zone candidates read the effective cost, not the printed one (Kentaro, Henzie too) |
| CR 701.6a moving a spell is not countering it | ✅ *since 2026-09-26* | `Effect::MoveSpellToZone` — `CounterSpellToZone`'s lift without the `uncounterable` skip; 15 "return / put / exile target spell" cards and Commit // Memory. Test `cr_rules::cr_701_6a_*` |
| "Do this only once each turn" limits the action, not the trigger | ✅ *since 2026-09-26* | `Predicate::SourceDoneThisTurn` + `Effect::MarkDoneThisTurn` via `shortcut::{may_once_each_turn, once_each_turn_on_take}`; 11 cards |
| Triggers that function from exile while a card exiled itself (Cosima's voyage) | ✅ *since 2026-09-26* | `TriggerZone::WhileSelfExiled` (gathered by the suspended-exile walk in `mod.rs`, keyed on `exiled_with == id`), `Effect::Voyage` in `effects/voyage.rs`; bot answers in `server/voyage.rs` (`recent_b::cmdr_esika::cosima_voyages_and_comes_home_with_its_counters`) |
| CR 701.40a turning face up for the mana cost is manifest's / cloak's rule only | ✅ *since 2026-09-26* | `CardInstance::turn_up_by_morph_only`, set by Missy's Cyberman path and `Effect::TurnFaceDown` (Ixidron), read by `actions.rs::turn_up_mana_cost`. An Omarthis turned up for {0} under Missy looped 3,812 times to a pod action cap |
| A printed "**target** opponent / target player" clause resolves to ONE seat | ✅ | `scripts/audit_target_opponent.py` — the mirror of `audit_each_opponent`, and the half that only bites at 3+ seats: 126 implemented cards modelled the clause as `PlayerRef::EachOpponent`, which is the same object in a duel and the whole table in a pod (Blood Artist draining 3 a death, Thoughtseize stripping every hand, Bojuka Bog exiling every graveyard). 126 → **1** (Consumed by Greed, whose gift branch already owns slot 0). Three were in a target deck (Endurance, Indulgent Tormentor, Nihil Spellbomb) and carry N-seat regression tests. CARD_BACKLOG's "TARGET-clause class" has the four traps, including the two engine ones: a player slot aims at the **caster** unless the seat's `hostile_player_targets` flag is on (it is, in `EvalWeights::default()`), and `friendliness_of_targeting_children` read `any` over a `Seq`'s children, so a rider aimed at the same seat ("…then draws a card") made the whole clause read as a gift |

### Commander-variant mechanics
| Feature | State | Where |
|---|---|---|
| Partner, "Partner with", Choose a Background | ✅ | `Keyword::{Partner, PartnerWith, ChooseABackground}`, `format::commanders_may_pair`. CR 702.124b/d in play too: both commanders seated, tax per commander (`commander_cast_count` keyed by `CardId`) and damage per (commander, player) — tested, and run by the pod's Krark/Rograkh seat. **Choose a Background is piloted too since 2026-09-19**: `pod::decks::ZELLIX_*` is the eighth target deck and `cr_702_124k_a_background_pair_plays_a_pod_game` runs it, which is the only seat whose second commander is a legendary *enchantment* — `is_legal_commander` rejects it alone and only `is_background_pair` lets it in, CR 702.124c combines {U} with {R}, and CR 702.124d's second tally stays at zero because CR 903.10a counts combat damage. ⚠ The rule is **702.124k**; four doc comments in `format.rs` and one in `card.rs` cited 702.124j, which is "Partner with [name]" |
| "Partner with" search trigger | ✅ | `shortcut::partner_with_search` (CR 702.124c); Sylvia Brightspear + Khorvath Brightflame |
| Partner—[text] (Friends forever, Survivors, Character select, Father & son) | ✅ | `Keyword::PartnerLabel(label)` + `commanders_may_pair`'s label equality (CR 702.124i) — one keyword for all four labels; Elmar + Sophina are the implemented Friends forever pair |
| Doctor's companion | ✅ | `Keyword::DoctorsCompanion` + `format::is_doctor_pair` (CR 702.124m's "no other creature types" is the teeth); `CreatureType::TimeLord`; The Third Doctor + Graham O'Brien in `sets/cmdr.rs` |
| Monarch, the initiative, goad, myriad, melee, voting / council's dilemma, tempting offer, join forces | ✅ *piloted* | `effect.rs` (`IsMonarch`, `HasInitiative`, `Goad` — read only through `GameState::goaders` (`game/goad.rs`: resolved, held per CR 611.2b, and attached `AttachedIsGoaded`), `Myriad`, `MeleeOpponentCount`, `VoteTally`, `TemptingOffer`, `JoinForces`), `dungeons.rs`. **Piloted, not only tested, since 2026-09-20**: a 2026-09-20 census found that **none of the nine target decks played any of them**, so none had ever resolved in bot self-play. `pod::decks::ADRIANA_MAIN` is the tenth deck and carries 27 such cards; `cr_702_122_the_multiplayer_native_seat_plays_a_pod_game` asserts the seven are still in the 99 and that a four-seat pod finishes |
| "Whenever a player attacks one of your opponents" (CR 508.1) | ✅ | `EventScope::OpponentOfYoursAttacked`, dispatched in `combat.rs` after the declaration: once per attacked player per listener that player opposes; attacker in `Target(0)`, attacked opponent as `Triggerer`. Breena and Combat Calligrapher (seat 90), tests `recent_b/cmdr_breena.rs` |
| Join forces | ✅ | `Effect::JoinForces` (CR 207.2c) over `ask_seat_amount` — turn order from the controller, every ask before every payment; Minds Aglow / Collective Voyage / Mana-Charged Dragon |
| Eminence (abilities that function from the command zone) | ✅ | CR 113.6b on two axes: triggers via `EventSpec.zone: TriggerZone::{Printed, CommandZoneToo, CommandZoneOnly}` (step gather in `stack.rs`, SpellCast dispatch in `actions.rs`, event dispatcher in `mod.rs`, the "whenever you attack" walk in `combat.rs` — Sidar Jabari), statics via `CardDefinition.statics_in_command_zone` + `CardInstance::command_zone_statics_active` (six walks). Edgar Markov / Arahbo / Oloro / The Ur-Dragon in `sets/cmdr.rs` |
| Commander ninjutsu | ✅ *piloted since 2026-09-20* | `Keyword::CommanderNinjutsu` (CR 702.49d) + `move_card_to`'s CR 408 command-zone arm; Yuriko, the Tiger's Shadow, who is now the **ninth pod deck** rather than only a fixture. The cost auto-taps (CR 602.2b) and the bot has a `pick_ninjutsu` candidate |
| Lieutenant / "if you control your commander" | ✅ | `Predicate::ControlsOwnCommander`; Thunderfoot Baloth (three static halves) and **Loyal Apprentice** (a begin-combat trigger whose body is wrapped in the predicate — `TriggeredAbility` has no intervening-`if` field, so the gate goes inside the effect as `Effect::If`). Its Thopter *gains* haste through `Selector::LastCreatedToken` rather than having it printed |
| "Commander creatures you control …" as a static filter | ✅ | `SelectionRequirement::IsCommander` — a board fact, so it reads a commander under anyone's control and the printed "you control" is the anthem's own scoping. Falthis (`Commanders you control have …`) and **Bastion Protector**, whose filter also carries `R::Creature` because the printed noun is "Commander **creatures**" and a planeswalker commander gets nothing |
| "Pay life equal to [the commanders' identity]" as a COST | ✅ | `ActivatedAbility::life_cost_value: Option<Value>` + `Value::CommandersColorIdentityCount` (CR 903.4, the union over a pair by CR 702.124c); **War Room**. Evaluated once during activation so the pre-flight gate and the payment agree. ⚠ It reads the **raw** identity, not `GameState::commander_identity_colors` — that helper answers "any colour" for a seat with no commander on purpose (so `AnyColorInCommanderIdentity` keeps a cube fixing land fixing), and as a cost that fallback is five life for a card |
| "The monarch controls enchanted creature" (CR 725) | ✅ *since 2026-09-26* | `StaticEffect::MonarchControlsEnchanted` + `GameState::sync_monarch_control` (`effects/politics.rs`), run from `set_monarch` and CR 725.4's succession; Fealty to the Realm. |
| Most-built commanders and top-1000 staples in the catalog | 🟡 *2026-09-26* | `decks::cmdr_legends` (Aragorn, Zur, Flubs, Galadriel, Arcades, Tiamat, Sauron, Morcant, Jodah, Thranduil, Queza, Child of Alara, Liesa, Urtet, Isshin, Tergrid, Najeela) and `decks::cmdr_top1000` (Xorn, Peregrin Took, Flowering of the White Tree, Lotho, Borne Upon a Wind, Imp's Mischief, Boromir, Jaheira, Torment of Hailfire, High Tide, Beseech the Mirror, Ojer Taq). COMMANDER_BACKLOG.md is stale (generated 2026-09-20) — re-run `scripts/commander_backlog.py`; Excalibur added (a filtered-only equip cost now works, CR 702.6a); Archway of Innovation rides the new `Effect::NextSpellGainsImproviseThisTurn` — the §2 top-1000 list is now fully in the catalog. 2026-09-28 (`01PdWYgx`): backlog regenerated; `decks::cmdr_legends2` adds 16 of its §1 commanders (Reaper King, Shroofus, Kratos, Myrel, Doran, Gargos, Syr Gwyn — `StaticEffect::EquipmentYouControlEquipZeroFor`, CR 702.6c — Jodah Archmage Eternal, Raggadragga, Alexios, Narset, Thalia and The Gitrog Monster, Rocco, Anti-Venom, Sonic, Iron Man): top-300 commanders 247 → 263 / 300. Rounded-off parts in INCOMPLETE_CARDS' session `015BCEt5` table |

### Simulation & tooling
| Feature | State | Where |
|---|---|---|
| N-seat pod runner | ✅ | `pod/mod.rs` — `build_pod_template`, `play_one_pod_game`, `run_pod_games` |
| One hundred and eighty-three legal target decks, a hundred and seventy-three official lists | ✅ | `pod/decks.rs` — seat 183 **The Tenth Doctor + Rose Tyler (URW) Timey-Wimey (WHO)**, seat 182 **The Fourth Doctor + Sarah Jane Smith (GWU) Blast from the Past (WHO)**, seat 181 **Davros, Dalek Creator (UBR) Masters of Evil (WHO)**, seat 180 **Heroes in a Half Shell (WUBRG) Turtle Power! (TMC)**, seat 179 **Abaddon the Despoiler (UBR) The Ruinous Powers (40K)**, seat 178 **The Thirteenth Doctor + Yasmin Khan (GUR) Paradox Power (WHO)**, seat 176 **Doctor Doom, King of Latveria (UBR) Doom Prevails (MSC)**, seat 175 **The Swarmlord (GUR) Tyranid Swarm (40K)**, seat 174 **Captain America, Team Leader (URW) Avengers Assemble (MSC)**, seat 173 **Dogmeat (RGW) Scrappy Survivors (PIP)**, seat 171 **Invisible Woman (WURG — her {R}{G}{W}{U} rules text, CR 903.4) The Fantastic Four (MSC)**, seat 170 **Caesar (RWB) Hail, Caesar (PIP)**, seat 169 **Dr. Madison Li (URW) Science! (PIP)**, seat 168 **Esika (WUBRG) From Cute to Brute (SLD)**, seat 166 **The Wise Mothman (BGU) Mutant Menace (PIP)**, seat 165 **Tidus (GWU) Counter Blitz (FIC)**, seat 164 **Auntie Ool (BRG) Blight Curse (ECC)**, seat 163 **Y'shtola (WUB) Scions & Spellcraft (FIC)**, seat 162 **Frodo + Sam (WBG) Food and Fellowship (LTC)**, seat 161 **Jace (WUBR, a planeswalker commander) Multiverse Reforged (FRC)**, seat 159 **Mirko (UB) Revenant Recon (MKC)**, seat 158 **Admiral Brass (UBR) Ahoy Mateys (LCC)**, seat 156 **Lathril (BG) Elven Empire (KHC)**, seat 155 **Pantlaza (RGW) Veloci-Ramp-Tor (LCC)**, seat 153 **Sevinne (URW) Mystic Intellect (C19)**, seat 150 **Galadriel (GU) Elven Council (LTC)**, seat 149 **Felothar (WBG) Abzan Armor (TDC)**, seat 140 **Anhelo (UBR) Maestros Massacre (NCC)**, seat 137 **Zimone (GU) Quandrix Unlimited (SOC)**, seat 135 **Kaust (RGW) Deadly Disguise (MKC)**, seat 132 **Sefris (WUB) Dungeons of Death (AFC)**, seat 148 **Ashling (WUBRG) Dance of the Elements (ECC)**, seat 147 **Éowyn (URW) Riders of Rohan (LTC)**, seat 167 **T'Challa (GW) Wakanda Forever (MSC)**, seat 160 **Terra, Herald of Hope (RWB) Revival Trance (FIC)**, seat 157 **Cloud, Ex-SOLDIER (RGW) Limit Break (FIC)**, seat 152 **Anje Falkenrath (BR) Merciless Rage (C19)**, seat 146 **Perrie (GWU) Bedecked Brokers (NCC)**, seat 145 **Hakbal (GU) Explorers of the Deep (LCC)**, seat 144 **Satya (URW) Creative Energy (M3C)**, seat 143 **Kamiz (WUB) Obscura Operation (NCC)**, seat 142 **Kadena (BGU) Faceless Menace (C19)**, seat 141 **Prosper (BR) Planar Portal (AFC)**, seat 138 **Saheeli, Radiant Creator (GUR) Living Energy (DRC)**, seat 136 **Nalia (WB) Party Time (CLB)**, seat 129 **Gonti (BGU) Grand Larceny (OTC)**, seat 125 **Henzie (BRG) Riveteers Rampage (NCC)**, seat 130 **Prossh (BRG) Power Hungry (C13)**, seat 133 **Zimone (GU) Jump Scare! (DSC)**, seat 131 **Morophon (WUBRG) Everyone's Invited! (SLD)**, seat 128 **Gavi (URW) Timeless Wisdom (C20)**, seat 126 **Oloro (WUB) Eternal Bargain (C13)**, seat 121 **Sidar Jabari (WUB) Cavalry Charge (MOC)**, seat 120 **Leinore (GW) Coven Counters (MIC)**, seat 124 **Millicent (WU) Spirit Squadron (VOC)**, seat 122 **Omo (GU) Tricky Terrain (M3C)**, seat 119 **Otrimi (BGU) Enhanced Evolution (C20)**, seat 117 **Kalamax (GUR) Arcane Maelstrom (C20)**, seat 118 **Aminatou (WUB) Miracle Worker (DSC)**, seat 116 **Olivia (RWB) Most Wanted (OTC)**, seat 113 **Zurgo (RWB) Mardu Surge (TDC)**, seat 111 **Jirina (RWB) Ruthless Regiment (C20)**, seat 109 **Chishiro (RG) Upgrades Unleashed (NEC)**, seat 106 **Galea (GWU) Aura of Courage (AFC)**, seat 102 **Estrid (GWU, a planeswalker commander) Adaptive Enchantment (C18)**, seat 97 **Eshki (GUR) Temur Roar (TDC)**, seat 95 **Lathliss (R) Reign of Dragons (FDC)**, seat 93 **Nelly Borca (RW) Blame Game (MKC)**, seat 51 **Yidris (UBRG) Entropic Uprising (C16)**, seat 56 **Ghired (RGW) Primal Genesis (C19)** and seat 59 **Valgavoth (BR) Endless Punishment (DSC)**, seat 64 **Saskia (WBRG) Open Hostility (C16)**, seat 70 **Ranar (WU) Phantom Premonition (KHC)**, seat 75 **Hazel (BG) Squirreled Away (BLC)**, seat 81 **Derevi (GWU) Evasive Maneuvers (C13)**, seat 87 **Atarka (RG) Draconic Destruction (SCD)**, seat 94 **Zinnia (URW) Family Matters (BLC)**, seat 96 **Ellivere (GW) Virtue and Valor (WOC)**, seat 101 **Kasla (URW) Divine Convocation (MOC)**, seat 103 **Kathril (WBG) Symbiotic Swarm (C20)**, seat 110 **Jared (WUBRG, a planeswalker commander) Painbow (DMC)**, seat 65 **Isperia (WU) First Flight (SCD)**, seat 66 **Inalla (UBR) Arcane Wizardry (C17)**, seat 68 **Anikthea (WBG) Enduring Enchantments (CMM)**, seat 69 **Urza (WUB) Urza's Iron Alliance (BRC)**, seat 72 **Faldorn (RG) Exit from Exile (CLB)**, seat 76 **Saheeli (UR, the sixth planeswalker commander) Exquisite Invention (C18)**, seat 83 **Mishra (UBR) Mishra's Burnished Banner (BRC)**, seat 84 **Gimbal (GUR) Tinker Time (MOC)**, seat 85 **Dihada (RWB, the seventh planeswalker commander) Legends' Legacy (DMC)**, seat 91 **Commodore Guff (URW, the eighth planeswalker commander) Planeswalker Party (CMM)**, seat 100 **Ulalek (WUBRG) Eldrazi Incursion (M3C)**, seat 108 **Zhulodok (C) Eldrazi Unbound (CMM)**, seat 114 **Aminatou (WUB, the ninth planeswalker commander) Subjective Reality (C18)**, seat 77 **Winter (BG) Death Toll (DSC)**, seat 89 **Jeleva (UBR) Mind Seize (C13)**, seat 92 **Go-Shintai (WUBRG) 20 Ways to Win (SLD)**, seat 98 **Kitt Kanto (RGW) Cabaretti Cacophony (NCC)**, seat 104 **Vrondiss (RG) Draconic Rage (AFC)**, seat 107 **Firkraag (UR) Draconic Dissent (CLB)**, seat 112 **Zaffai (UR) Prismari Performance (C21)** (past `MAX_SEATS`: `--pod-decks` only); seat 41 is **Neyali (RW) Rebellion Rising (ONC)** and seat 42 **Teferi (U, the fifth planeswalker commander) Peer Through Time (C14)**, seat 46 **Kalemne (RW) Wade into Battle (C15)**, seat 49 **Kardur (BR) Chaos Incarnate (SCD)**, seat 53 **Kaalia (RWB) Heavenly Inferno (CMD)**, seat 55 **Zedruu (URW) Political Puppets (CMD)**; **Teval (BGU) is Sultai Arisen (TDC)**, **N'ghathrod (UB) is Mind Flayarrrs (CLB)**, **Clavileño (WB) is Blood Rites (LCC)**, **Zndrsplt/Okaun (UR) is Heads I Win, Tails You Lose (SLD)** **Zada (R) is Goblin Storm (SLD)** and **Gisa (B) is Wretched Ranks (FDC)** and **Ghalta (G) is Tramplesaurus Rex (FDC)** and **Sai (U) is Keen Engineering (FDC)** and **Aesi (GU) is Reap the Tides (CMR)** and **Ixhel (WBG) is Corrupting Influence (ONC)** and **Anowon (UB) is Sneak Attack (ZNC)** and **Shiko and Narset (URW) is Jeskai Striker (TDC)** and **Sliver Gravemother (WUBRG) is Sliver Swarm (CMM)** and **Stella Lee (UR) is Quick Draw (OTC)** and a second **Edgar Markov (BRW) seat is Vampiric Bloodlust (C17)** and **Giada (W) is Calling All Angels (FDC)** and a second **Freyalise (G) seat is Guided by Nature (C14)** and **Bello (RG) is Animated Army (BLC)** and **Gisa and Geralf (UB) is Grave Danger (SCD)**, **Nahiri (W) is Forged in Stone (C14)**, **Disa (BRG) is Graveyard Overdrive (M3C)**, **Ezuri (GU) is Swell the Host (C15)**, **Gisela (W, the meld commander) is Angels (SLD)**, **Daretti (R) is Built From Scratch (C14)**, **Ob Nixilis (B) is Sworn to Darkness (C14)**, **Strefan (BR) is Vampiric Bloodline (VOC)** **Meren (BG) is Plunder the Graves (C15)** **Mizzix (UR) is Seize Control (C15)** and **Adrix and Nev (GU) is Quantum Quandrix (C21)** and **Daxos (WB) is Call the Spirits (C15)** and **Osgir (RW) is Lorehold Legacies (C21)** and **Ghave (WBG) is Counterpunch (CMD)** and **Wyleth (RW) is Arm for Battle (CMR)** and **Tegwyll (UB) is Fae Dominion (WOC)** and **Breya (WUBR) is Invent Superiority (C16)** and **Kardur (BR) is Chaos Incarnate (SCD)** and **Temmet (WUB) is Eternal Might (DRC)** and **Obuun (RGW) is Land's Wrath (ZNC)** and **Kynaios and Tiro (RGWU) is Stalwart Unity (C16)** and **Emmara (GW) is Token Triumph (SCD)** and **Rin and Seri (RGW) is Raining Cats and Dogs (SLD)** and **Brimaz (WB) is Growing Threat (MOC)** and **Bright-Palm (RGW) is Call for Backup (MOC)** and **Hearthhull (BRG) is World Shaper (EOC)** and **Ms. Bumbleflower (GWU) is Peace Offering (BLC)** and **Marath (RGW) is Nature of the Beast (C13)** (seats 29-40, 43-45, 47-50, 54, 57, 60, 63, 67, 74, 79, 82 and 88; the other sessions' seats 41, 42, 46, 51-53, 55, 56, 58, 59, 61, 62, 64-66, 68-73, 75-78, 80, 81 and 83-87 are listed in DECK_FEATURES), each card for card from MTGJSON's deck files (`scripts/precon_scan.py` ranks every precon by missing cards), appended after `pod_field(10)`..`pod_field(87)` (`--seats 11`..`88`, clamped to the engine's 64-seat mask `MAX_SEATS` — `--pod-decks` reaches the rest; the pod budget counts plays, `max(seats, 4) × 1,000`, since priority passes grow with seats²). Teval is the graveyard seat (Kotis's once-a-turn graveyard cast, CR 106.6 graveyard-only mana, Steward's granted land abilities); N'ghathrod the Horror mill-and-steal seat (CR 601.2b pay-one-of costs, CR 702.62b suspended triggers, CR 614 exile-if-it-would-leave). The ten before them: Sigarda GW / Judith BR / Hanna UW / Tatyova GU / Krark+Rograkh R (the Partner seat), **Edgar Markov BRW** — the first three-colour identity and the first **Eminence** commander in self-play, appended after `pod_field(5)` so no existing reading moves (`--seats 6` reaches it; it wins 52.1 % there, which DECK_FEATURES records as a finding about Eminence rather than a deck to tune) — and **Freyalise G**, the **planeswalker-commander** seat (CR 903.3a), appended after `pod_field(6)` for the same reason (`--seats 7`) — and **Zellix + Passionate Archaeologist UR**, the **Choose a Background** seat (CR 702.124k), appended after `pod_field(7)` (`--seats 8`) and the field's first Izzet identity; it runs the bond cycle's Training Center, a tapland in a duel and a dual in a pod. ⚠ Two tests used to find their deck with `last()`; appending broke the Partner one, and "two commanders" stopped being unique when the Background pair landed — the Partner test now asks for two *creature* commanders (CR 702.124h) and the planeswalker test for exactly *one* non-creature commander. Validated by the suite. Judith retuned 2026-09-19 (6.2 → 14.6 % of four-seat pods; DECK_FEATURES carries the experiment, whose reusable half is that a *better* aristocrats list measured worse — a one-ply material evaluator cannot price a value engine). Each runs the full colorless staple set: Sol Ring, Command Tower, Arcane Signet, Commander's Sphere, Mind Stone, Path of Ancestry, Opal Palace — and each two-color seat its guild Signet + Talisman (mono-red Krark takes neither: both cycles are two-color in CR 903.4 identity) — and **Yuriko, the Tiger's Shadow UB**, the **commander ninjutsu** seat (CR 702.49d), appended after `pod_field(8)` for the fourth time for the same reason (`--seats 9` reaches it; `bot_ladder`'s seat clamp is now `target_decks().len()` rather than a literal 8). It is the only seat whose commander leaves the command zone by an action that is **not a cast**, so CR 903.8's tax never applies to that route and `commander_cast_count` stays where it was — asserted by `cr_702_49d_a_commander_ninjutsu_seat_plays_a_pod_game`. The bot's `pick_ninjutsu` already read the command zone, so the swap is taken in real games — and **Adriana, Captain of the Guard RW**, the **multiplayer-native** seat, appended after `pod_field(9)` for the fifth time for the same reason (`--seats 10`). It exists because a census found that **not one of the nine decks above played a single multiplayer-native mechanic** — monarch (CR 725), goad (CR 701.15), melee (CR 702.121), voting (CR 701.38), tempting offer and join forces (ability words, CR 207.2c), the initiative (CR 726) and myriad (CR 702.116) were all implemented and all unreached by self-play. A mechanic the field never plays is a mechanic self-play never crashes on |
| 4-player Commander demo state | ✅ | `demo.rs::build_commander_state_seeded`, the first four of the same eight decks |
| Commander pod mode in `bot_ladder` | ✅ | `bot_ladder --commander [--seats N]`. **Runs on a debug build since `88f07f22`** — the worker was on the 2 MiB `scope.spawn` default and the first game of every batch aborted the process, so every pod figure committed before it is a `release-fast` statement and only that (ENGINE_BACKLOG's sixty-second find) |
| Per-deck card coverage in a pod run | ✅ | `bot_ladder --commander --card-census` — totals the action census by card and names, per seated deck, what a run never cast, played or activated. First run (10 seats, 2,000 games): **nine of the ten target decks played every card in the list**; **all ten now play every card in the list**, and the four official lists played all 100 on their first runs (14 seats, 1,000 games, seed 9940, 672 distinct cards). The first run left one — Yuriko's Agony Warp, castable by no path, since the hand sweep drops a `is_combat_trick` card unconditionally and the trick picker folded its two target slots into one pump aimed at our own creature (sixty-third find, fixed). 434 distinct cards -> 435 on the same seed |
| Dead triggers in a pod run | ✅ *since 2026-09-27* | `--card-census` prints **"never triggered"** — a played card none of whose printed triggers reached the stack, with its kind/scope list (`ActionCensus::note_triggers`). ⚠ Census with `--a dflt`: without it the pilot is `baseline` |
| Golden outcomes for fixed-seed Commander pods | ✅ | `pod::tests::cr_903_seeded_pod_outcomes_match_the_committed_table` — three seeds' (winner, turns, actions) committed, cross-process. The triple rather than a line-per-action trace: a pod game is ~2,000 actions, so a real trace is a 400 KB file every Commander commit re-blesses, and it moves on the same changes |
| Trigger chains off targets can't recurse without bound | ✅ *since 2026-09-26* | `GameState::push_pending_trigger` dispatches a pushed trigger's "becomes the target" events in place only eight deep, then queues them for the next dispatch (CR 603.3b). Two opposing Scalelord Reckoners overflowed the pod thread's stack (`cr_603_3b_facing_scalelord_reckoners_do_not_recurse_forever`) |
| A spell-copy chain can't grow the stack without bound | ✅ *since 2026-09-26* | `recommend::MAX_STACK` (512 *spells* — triggers aren't counted, a 771-creature attack puts 771 there legitimately): `copy_stack_spell_controlled` copies nothing past it and `stop_reason` ends the game as a `BoardCap` at it. Venser, Fervent Forger copying a Replication Technique that copies Venser reached 1,692 stack items and one action never returned (`a_spell_copy_chain_stops_at_the_stack_bound`) |
| A token batch can't ask for an unbounded mint | ✅ *since 2026-09-26* | `GameState::doubled_token_count` / `scaled_token_count` — every token mint loop (create, copy, populate, attacking, Incubator, fight-each) doubles per CR 614.13 doubler, triples a creature batch per Ojer Taq (`StaticEffect::TripleCreatureTokens`), and caps at `recommend::MAX_BATTLEFIELD`; an 8-seat pod hung on 2^32 Adrix and Nev copies (`cr_614_13_forty_token_doublers_mint_a_bounded_batch`) |
| Bot takes every decision the pod decks introduce | 🟡 | Grant-then-cast and impulse abilities (`server/grant_cast.rs`, 2026-09-27: Serpent's Soul-Jar, Emry, Yasmin Khan, Oracle's Vault — the activation is taken when a card it grants is castable and that cast beats passing); command-zone activations (`server/command_zone.rs`: Derevi's tax-free put-in, main phase or an opponent's end step). Channel lands (`server/channel.rs`, 2026-09-27: a spare one is channelled when the dry run beats the board without it). `server/bot.rs` `cast_candidates` — escape, replicate, buyback, entwine, squad, fuse, casualty, bargain and retrace got candidate blocks 2026-09-23; a short payment floats spend-restricted sources (`pay_with_restricted_sources`, CR 106.6); an X spell's targets are picked at its X (Finale of Promise) and a pay-X-life X is sized from life (Toxic Deluge); a friendly pump prefers Zada. A 24-seat `--card-census` plays every card of all 24 lists. Room doors (unlock/cast, `server/room.rs`) and a defender's fog (`server/fog.rs`: held from the sweep, cast after blocks against dangerous unblocked damage, Prismatic Strands' tap-flashback and color) since 2026-09-26, Commander games only. **Never-activated abilities (`--card-census`, 6 seats × every deck, seed 206001): 906 → 784** after `server/manland.rs` (creature-lands, 46 → 4) and `server/evasion.rs` (targeted keyword grants, 151 → 73); then **→ 735** with `server/selection_sink.rs` (end-step scry/surveil/loot: scry 29 → 5, surveil 10 → 1) and `server/regenerate.rs` (a regeneration shield against a regenerable destroy: 21 → 13); `server/counter_sink.rs` (Forge of Heroes) after that. Mutate since 2026-09-27 (`server/mutate.rs`: the Otrimi seat), and Sudden Substitution's exchange (`spell_response::pick_substitution_response`, sim-gated) — a 183-deck census (6 seats × 150) now leaves only Semester's End unplayed, and its sweeper-shield path works (the bot seldom holds {3}{W} up). Open: waterbend / sacrifice-reduce, none in a pod list (ENGINE_BACKLOG). Optional loops (CR 732.2a): `pod/mod.rs::RepeatGuard` — past 64 activations of one ability of one permanent in a turn a seat passes instead (Woe Strider × Prowling Geistcatcher × Gravespawn Sovereign ran 2,723 in a main phase); pods only |
| A net pilot in a pod | ⛔ by design | the observation encoder is two-seat (`encode_state_inner`'s `1 - seat`, `sources: &[_; 2]`). Since 2026-09-26 `net_eval` answers `None` for any state that is not two seats (`net_for_state`), so a net profile in a pod plays its material leaf rather than panicking — the client's local pods and the hosted lobby both seated the net-leaf search in every pod seat, and seat 2's first searched main phase underflowed. Pods now seat the heuristic default; `bot_ladder --commander` refuses a net profile (it would be the material leaf under the net's name) and runs the material-leaf search: `mcts-dflt-256` in a `dflt` field 0.87x [0.56x, 1.18x], 92 games at ~52 CPU-s a game. Design notes for an N-seat encoding are in `ML_NOTES.md`; widening it retrains every net |
| Measure a bot change in a pod | ✅ *since 2026-09-26* | `bot_ladder --commander --a A --b B`: `A` in one seat, `B` in the rest, each deal once with `A` in every seat (`pod::run_pod_hero_games`); A's share of wins against the 1/N a mirror gives, standard error over deal groups. The mirror null is exact (25.0 % ± 0.0). `dflt` 1.47x in a `baseline` field, `baseline` 0.63x in a `dflt` field. Measured null (not adopted): ranking opponents by board power (1.01x over five fields) and "kill a seat you can before anything else" (1.00x, near-zero incidence); a pre-combat pass spending an attacker's {T} ability when it beats the unblocked hit (Brash Taunter's fight, Bloodline Keeper; 1.00x at seeds 8100 and 8200 over decks 1-4 and 6/16/49/50, −3 % pod wall clock) — ⚠ **the `dflt` pilot already activates both; `--card-census` without `--a` runs `baseline`, so its never-activated list is `baseline`'s, not the default's. Census with `--a dflt`.** Adopted at zero incidence as the rules fix: chump a commander about to deal its 21st point (CR 903.10a). Adopted on the A/B: votes and option asks by settled outcome (`EvalWeights::option_eval` — a bot voted option 0 every time), 25.11 % vs 25.00 % over 48,000 games on the eleven vote-heavy groups, none down; the attack sim's crack-back through every opponent's turn, not the next seat's (`EvalWeights::pod_horizon`), 1.02x at four seats and 1.07x at six over 100,800 games, pods ~1.4-2.3x slower and ~10 % longer. Measured and left off: attacking the table leader (`EvalWeights::leader_target`) — 1.17x for one seat, but pods 40-57 % longer when every seat does it, and the lifegain decks that win half their six-seat pods (Gisela 64 %, Y'shtola 58 %) do not move; pricing creature keywords in pods (`pod_keyword_pct`) is null both for one seat and for the table (ML_NOTES) |

## Tier 1 — High-leverage engine primitives

Each unblocks a large swath of cards.

1. 🟡 **Replacement-effect framework.** `replacement.rs` models zone-change
   replacements (Commander → command zone); the rest is per-card. Shipped:
   enters-under-an-opponent's-control (`CardDefinition.enters_under_opponent_control`,
   applied at the battlefield hop before any ETB trigger reads a controller —
   Captive Audience), "can't be regenerated this turn" (CR 701.15g —
   `Effect::CantBeRegeneratedThisTurn` blanks existing and future shields),
   enters-tapped (`StaticEffect::EntersTapped`, incl. self-source) and the
   enters-*untapped* override (`StaticEffect::LandsEnterUntapped` — Spelunking),
   exile-instead
   for non-cast creatures (Containment Priest), opponent-creature-dies → exile
   (Valentin), graveyard → exile hate (`ExileCardsBoundForGraveyard` via
   `route_to_graveyard` — Rest in Peace, Leyline of the Void), counter-lock
   (Solemnity), counter/damage doubling (Doubling Season, Furnace of Rath),
   damage prevention as shields (`prevention_shields`), per-source combat shields
   (Maze of Ith), damage redirection (Palisade Giant), draw doubling (Thought
   Reflection), damage halving (Ghosts of the Innocent), creature-ETB control
   steal (Gather Specimens), as-enters choice-of-P/T-and-keyword
   (`enters_as_choice`, CR 614 — Corrupted Shapeshifter, applied before the
   first SBA so a printed `*/*` never dies as a 0/0), skip-step and skip-turn.
   Counter-placement replacements (Hardened Scales, Doubling Season, Mowu's
   self-scoped `ExtraPlusOneCounterOnSelf`) now also apply on the **proliferate**
   path (CR 614.16), via `scaled_counter_count_on`. The draw branch is closed
   (skip, exile-and-play, redirect, doubling, empty-hand bonus, dredge,
   `MayReplaceDrawWithTutor` and `MayReplaceDrawWithRevealUntilKind`). Still to generalize: as-a-copy ETB. A *general* as-enters one-shot now
   ships (`CardDefinition.as_enters_effect`, resolved pre-SBA — Ixidron). (Devouring Hellion / Rescuer Sphinx's
   as-enters reflexive shape now ship via `devour` / a reflexive ETB.)
   **CR 614.12 is a closed class as of 2026-09-20 (ENGINE_BACKLOG's
   forty-eighth find, 89 → 7).** `game::as_enters::apply_as_enters_replacements`
   is the ONE funnel all four battlefield-entry paths run — the cast
   (`stack.rs`), the universal move (`effects/movement.rs`), the **land drop**
   (`actions.rs::play_land`) and the **token mint** (`mod.rs::
   mint_token_with_counters`); before it, each of the three appliers was wired
   to a different subset, so a played Cavern of Souls named nothing and a
   token copy had no trigger to fire (CR 614.12's own example).
   `ResumeContext::LandEntry` is the land drop's continuation — the one entry
   path that can neither park on a stack item nor be replayed — and
   `actions::finish_land_entry` is what it resumes into.
   `Effect::AsEntersChooseMode` is the modal ask that works without a stack
   item (`Effect::ChooseMode` reads `ctx.mode`, which an entry does not have),
   and `Effect::SourceEntersTapped` the "it enters tapped" branch that emits
   no tap event. ⚠ A `wants_ui` seat is asked for real only on the land drop;
   the other three paths still drive the ask through `resolve_effect_driven`.
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
- 🟡 **Targeting refinements:** **CR 601.2c's cross-slot restrictions** both
  ship as `SelectionRequirement` atoms read out of
  `GameState::target_slots_scratch`: `SameControllerAsTargetSlot` ("two target
  creatures controlled by the same player") and, since 2026-09-20,
  `OtherThanTargetSlot` — the printed word "**another**" between two slots
  (ENGINE_BACKLOG's fifty-third find; `scripts/audit_another_target.py` is the
  ratchet). The cast path, the activation path and the auto-picker all enforce
  them; the picker's `already_picked` was only ever a preference.
  `SelectionRequirement::OnBattlefield` is the on-board half of a **mixed**
  zone clause, whose graveyard half `scripts/audit_target_zone.py` ratchets
  (the fifty-second find). Resolution-time legality re-check (608.2b) ships
  for single/multi-target spells and Auras, and now resolves `{X}`-from-cost
  target filters (Hearth Kami's "artifact with mana value X" via
  `ManaValueExactlyXFromCost`). "Up to N targets" ships via
  `Effect::ApplyToTargets` (Sea God's Scorn bounce-3, Wrap in Flames
  1-to-each-of-3, Elemental Expressionism bounce-2); an **optional single slot
  alongside a required one** ships via `Effect::OptionalTargets { min, body }`
  (Primal Might's required pumped creature + optional fight target, Boom Box's
  three optional destroy slots). Protection now gates spells
  *and* abilities (CR 702.16c — `ability_target_has_protection`) across color /
  creatures / creature-type (Kitsune Riftwalker, Yawgmoth, Baneslayer) /
  spell-subtype / **multicolored** (`ProtectionFromMulticolored` — Stonecoil
  Serpent) / **monocolored** (`ProtectionFromMonocolored` — Guardian of the
  Guildpact), and combat damage (CR 702.16e — `damage_prevented_by_protection`
  on both attacker→blocker and blocker→attacker). Multi-kind slots ship —
  a spell can target a permanent in one slot and a *player* in another, with
  `Selector::ControlledBy { who: Target(n) }` declaring slot `n` as a player
  target (How to Start a Riot, Sokka's Haiku's spell+land slots). Ignore-hexproof
  statics ship: creature-only (`IgnoreOpponentsCreatureHexproof` — Glaring
  Spotlight) and broad players+permanents (`IgnoreOpponentsHexproof` — Kaya,
  Bane of the Dead); the server view surfaces player hexproof per-viewer and the
  client targeting filter mirrors both. Remaining: "target each".
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
  mana abilities); ⏳ Experience counters → actually **✅** (`CounterType::
  Experience`); ✅ Poison/Toxic, Devotion, Ascend/city's blessing; ✅ Monarch;
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

0. **Next set to close.** Bloomburrow, Duskmourn, Outlaws of Thunder Junction,
   **Edge of Eternities**, **Final Fantasy** and **Conspiracy** (CNS) are all
   closed (`set_gaps.py blb dsk otj eoe fin cns` is empty). The Odyssey
   block, the Onslaught block (**ONS**,
   **LGN**, **SCG**), the Mirrodin block (**MRD**, **DST**, **5DN**), the
   Kamigawa block, **Mirrodin Besieged**, **New Phyrexia** (the Scars block
   is closed), **Legends** (273 cards, `sets::leg`–`leg7`), **Antiquities**
   (64 cards, `sets::atq`), **Arabian Nights** (63 cards, `sets::arn`) and
   **The Dark** (97 cards, `sets::drk`/`drk2`) and **Homelands** (`sets::hml`–
   `hml3`), **Conspiracy: Take the Crown** (CN2), **Murders at Karlov
   Manor** (MKM) and **Stronghold** (STH) are all at zero. The
   **whole Tempest block is closed** too (`set_gaps.py tmp sth exo` is empty).
   **Weatherlight (WTH) is closed** too (`set_gaps.py wth` at zero —
   `sets::wth` + `sets::wth2`, tests in `classic_sets/wth`), which finishes
   the Mirage block's third set and gives cumulative upkeep, banding and
   phasing their first real card coverage. **Visions (VIS) is closed** too (`set_gaps.py vis` at zero —
   `sets::vis` + `sets::vis2`, tests in `classic_sets/vis`), which finishes
   the Mirage block's second set. **Mirage (MIR) itself is the live front**
   (`set_gaps.py mir` at 17 after this push, `sets::mir`–`mir5`); Coldsnap
   (CSP) is open in parallel. Each of MIR's last 17 is blocked on one
   primitive — TODO.md → "Mirage residue" names them card by card.
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
