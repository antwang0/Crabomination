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
against the shipped CR text (`crabomination/MagicCompRules_20260417.txt`), not
against this file's history**. Don't re-audit; update rows as they move.
Per-deck card completion lives in `DECK_FEATURES.md`.

### Format core (CR 903)
| Feature | State | Where |
|---|---|---|
| `Format::Commander`: 40 life, 100-card, singleton, command zone at start | ✅ | `format.rs` (`FormatRules`, `validate_commander_deck`), `game/mod.rs::apply_format`, `seat_commanders` |
| Commander designation persists across zones; a copy is not a commander | ✅ | `player.rs::Player::commanders` (`CardId`s, zone-independent); `GameState::is_commander`. A token/copy has its own `CardId`, so exclusion is by construction |
| Commander tax {2} per prior cast, as an additional cost | ✅ | `actions.rs::cast_from_command_zone` — added before `cost_reduction_for_spell` / `reduce_by_cost`, so reductions and the floor see it |
| CR 903.9a graveyard/exile return as an SBA (dies triggers fire first) | ✅ | `stack.rs::commander_zone_return_sba`, `commander_return_declined` |
| CR 903.9b hand/library return as a *replacement* | ✅ | `replacement.rs`, registered per commander by `seat_commanders`; `Decision::CommanderRedirect` |
| "Whenever your commander is put into the command zone" (both routes) | ✅ *since 2026-09-24* | `GameEvent`/`EventKind::CommanderPutIntoCommandZone`, queued by `note_commander_to_command_zone` from `commander_zone_return_sba` and `place_card_at_resolved_zone`; Myth Unbound (`cr_903_9_myth_unbound_draws_on_both_routes_home`) |
| Commander damage, 21 from one commander, per (commander, player) | ✅ | `game/mod.rs::commander_damage` + `record_commander_damage`; combat damage only (CR 903.10a) |
| CR 903.4 color identity: cost, rules text, indicator, back faces | ✅ | `color_identity.rs`; audited against Scryfall for all 18,058 implemented cards (15 known card gaps listed in the audit's `KNOWN_IDENTITY_GAPS`) |
| Deck validation: singleton, identity subset, legal commander, ban list, **CR 903.5e no sideboard** | ✅ | `format.rs::validate_commander_deck` (CR 903.5a counts the commander), `commanders_may_pair`, `COMMANDER_BANNED` (Scryfall's list, 2026-09-18). ⚠ CR 903.5e is checked *here*, not in `FormatRules`: `validate_deck_refs` takes a flat card list (which is what lets 903.5a count the commander with the 99) and so never sees a sideboard. CR 903.5d falls out of the identity walk's `basic_land_identity` |
| CR 903.3 what a commander may *be*: legendary creature, **Vehicle**, or **Spacecraft with a power/toughness box** | ✅ | `format.rs::is_legal_commander` + `is_spacecraft_with_pt` (a station card's printed box is its band's `pt`, CR 721.2b). Shorikai / Parhelion II / The Seriema pass, The Eternity Elevator — a station card whose bands only add mana — does not. `CommanderDeckError::IllegalCommander` |
| CR 903.5d a card with a basic land type is bounded by the commander's identity | ✅ *by folding* | `color_identity.rs::basic_land_identity` adds the type's color to the card's identity rather than checking 903.5d separately. Same answer for the subset check, and it is what matches Scryfall's `color_identity` — which is what the whole-catalog audit ratchets against |
| The old mana-production restriction | n/a | removed from the CR years ago; correctly absent. ⚠ It is **not** CR 903.11 any more — that number now holds "traditional cards from outside the game cannot be brought into a Commander game" (903.11a adds the same-name and identity gates). Unreachable here: CR 903.5e leaves no sideboard, so `Effect::WishToHand` / `WishToLibrary` find nothing outside the game and fall through to the caster's own exile |
| "Any color in your commander's color identity" mana | ✅ | `ManaPayload::AnyColorInCommanderIdentity`, `GameState::commander_identity_colors`; Command Tower / Arcane Signet / Commander's Sphere / Path of Ancestry / Opal Palace. A colourless commander adds **no mana, and not {C}** (rulings 2020-11-10). ⚠ A seat with *no* commander keeps "any colour" where **CR 903.4f** says the quality is undefined and the ability does nothing — deliberate, for the cube pool's fixing, and what keeps the two-player traces byte-identical. Unreachable in a Commander game, where every seat has one |
| "If you control **a** commander" = any player's commander (CR 903.3) | ✅ *since 2026-09-23* | `Predicate::YouControlACommander` / `PlayerControlsACommander { who }` via `GameState::is_commander`; it read only your own designations, so a stolen commander never counted (Akroma's / Jeska's Will, Crimson Honor Guard) |
| Commander to hand from the command zone (Command Beacon) | ✅ | `Effect::CommanderToHand`, `game/effects/commander.rs` |
| A **melded** commander is still the commander (Gisela's ruling, CR 903.3 + 701.37) | ✅ *since 2026-09-24* | `GameState::commander_card_of` maps the melded permanent to its commander component: `is_commander`, `is_own_commander_object` ("if you control your commander") and the combat commander-damage tally read it, so Brisela's hits land on Gisela's card. Leaving splits `meld_parts` and only the commander card goes home. Tests `cmdr_gisela::a_melded_commander_is_still_the_commander` and `a_bounced_melded_commander_can_go_home` (CR 903.9b: the parts move one by one and the commander card's replacement applies); pod seat 33 plays it |
| CR 106.6 mana provenance — "when that mana is spent to …" | ✅ | `SpendRestriction::{CommanderTypeScry, CommanderCastCounters}` + `is_rider()`; `game/effects/commander.rs::{note_commander_mana_riders, push_commander_mana_scry, spell_kind_for}`; Path of Ancestry / Opal Palace. CR 106.6a's per-mana count included — `spent_restrictions` carries `(restriction, pips)`. `SpendRestriction::allows` answers the rider half once by short-circuiting on `is_rider()`, and the bot's `available_mana` counts floating rider mana (`ManaPool::rider_amount`) — it read the restricted bucket as unspendable wholesale, so a seat holding Path of Ancestry's green reported `by_color [0,0,0,0,0]` |
| Casting a commander for an **alternative** cost from the command zone | ✅ | `actions.rs::cast_spell_alternative_from` over `AltCastZone` — one body for both zones; CR 903.8's tax is pushed ahead of the reductions. `GameAction::CastFromCommandZone { alternative, pitch_card }` |
| "Can be your commander" on a non-creature (planeswalker commanders) | ✅ | `CardDefinition::can_be_commander`, read by `validate_commander_deck`; Freyalise, Llanowar's Fury in `sets::cmdr` (32 more such cards exist, none implemented). **Piloted, not only validated, since 2026-09-19**: `pod::decks::FREYALISE_MAIN` is the seventh target deck and `cr_903_3a_a_planeswalker_commander_seat_plays_a_pod_game` runs it. Two things fall out of a planeswalker commander and both hold — it is recast from the command zone under the CR 903.8 tax like any other, and its (commander, player) damage tally stays at zero all game, because CR 903.10a counts **combat** damage and a planeswalker deals none |

### Multiplayer foundation (CR 800-series)
| Feature | State | Where |
|---|---|---|
| CR 800.4a a departed seat is not a player | ✅ | `GameState::living_seats` beside `seats_in_turn_order_from` and `resolve_players`; `default_hostile_opponent` for the ranked "an opponent". `scripts/audit_seat_walks.py` gates **two** columns — a loop that hands a dead seat a *question* (votes, ballots, offers) and a walk that hands it a *role* (`LowestLife` was always the departed seat; nine open-coded "first opponent" fallbacks, three of them picking a defending player). Both 0 |
| CR 506.2 the **defending player** as a filter | ✅ | `SelectionRequirement::ControlledByDefendingPlayer` + `defending_player_in_combat` (`effects/eval.rs`), which `PlayerRef::DefendingPlayer` also ends in so the two cannot disagree. 32 cards printed the clause and were modelled `ControlledByOpponent` — exact in a duel, one seat of several in a pod; `scripts/audit_defending_player.py` is the ratchet at 0/32. The clause is not always on the attacker: an instant cast in combat, an Aura/Equipment whose host attacks, and a trigger firing off another creature's attack all resolve through the host-then-single-defender fallbacks, and a post-teardown trigger (Kusari-Gama) reads the damaged blocker's controller instead |
| N seats (2..N), turn rotation, APNAP ordering | ✅ | `game/mod.rs`, `multi_player_game`; tests in `core_rules/multiplayer.rs` (76) and `cr_801.rs` |
| Attack any opponent / their planeswalkers, per-attacker defender | ✅ | `game/combat.rs` |
| CR 601.2c "for each opponent, … target X *that player* controls" | ✅ | `Effect::ForEachOpponentTarget` — a constraint on the chosen set (one target per controller, capped at the opponent count), enforced by the auto-target picker, the cast path and resolution. The five Primordials, Grasp of Fate, Omega, Tempted by the Oriq; tests in `core_rules/per_opponent_targets.rs` |
| CR 603.2c "whenever **one or more** … deal combat damage to a player" | ✅ | `EventSpec::once_per_batch` plus `combat.rs::BatchSubject` — the per-attacker walk collapses to one fire **per damaged player**, so an alpha strike across a pod is one event a defending seat, not one for the step. Ratcheted over the catalog by `catalog_registration::every_batched_damage_trigger_fires_once_a_batch` (three accepted spellings: the flag, `once_per_turn`, or a `FromYourGraveyard` scope the graveyard walk already dedupes). Malcolm, Keen-Eyed Navigator's "a Treasure for each opponent dealt damage" is exactly this count |
| CR 511.3 combatants stay in combat until the end of combat step **ends** | ✅ *since 2026-09-23* | `combat.rs::remove_all_from_combat`, run by `advance_step` leaving `EndCombat`; `combat_damage_dealt()` keeps a repeated `resolve_combat` a no-op. It used to tear combat down as regular damage was dealt |
| CR 800.1 — how many players | ✅ *2..=64* | `game::MAX_SEATS` (every per-seat mask is a `u64`), asserted in `GameState::new`; `bot_ladder` clamps `--seats`/`--pod-decks` (`core_rules::multiplayer::cr_800_1_a_game_holds_at_most_max_seats_players`) |
| CR 508.1d "during [player]'s next turn, creatures they control attack [this] if able" | ✅ *since 2026-09-24* | `Effect::LureCreaturesToSourceNextTurn` + `PlayerCold.attack_lure`, `game/attack_lure.rs`; enforced in `combat.rs`'s declaration, honoured by the bot's `restore_forced_attackers` and target pass (Gideon Jura; `recent_b::cmdr_isperia`) |
| CR 509.1a — only a defending player declares blocks | ✅ | `combat.rs::may_declare_blocks` |
| CR 506.2 "attacks **you**" is a different count from "attacks" | ✅ | `Predicate::AttackedDefenderWithCountAtLeast { who, defender, at_least, include_planeswalkers }`. The undirected `AttackedWithCountAtLeast` counts a player's whole declaration wherever it is pointed, which is the same question only at two seats; Mangara, the Diplomat and Trouble in Pairs both gate on two or more attackers aimed at *you* and would otherwise fire off an opponent swinging at a third player. `include_planeswalkers` is a printed difference between those two cards, not a knob — a creature attacking your planeswalker is attacking the planeswalker. ⚠ Reads `GameState.attacking`, which is filled *during* the declaration, so on a `ControllerAttackedByOpponent` trigger the gate goes inside the effect as an `Effect::If`, not in the `EventSpec` filter |
| CR 603.2 a defender-side attack trigger's own condition | ✅ *since 2026-09-19* | `combat.rs` — the `ControllerAttackedByOpponent` walk was the one of five that built its listeners without `t.event.filter`, so a `.with_filter` on such a trigger compiled and was never read (Reveille Squad's printed "if this creature is untapped"). ENGINE_BACKLOG's fiftieth find |
| CR 614 "if an opponent would begin an extra turn, they skip it instead" | ✅ | `StaticEffect::OpponentsSkipExtraTurns`, read at the one place `Player.extra_turns` is consumed (`stack.rs::end_turn`). The charge is **spent** even though the turn is skipped — a replacement on *beginning* the turn, and leaving it banked makes one Time Warp re-offer the same turn at every pass. Trouble in Pairs |
| CR 121.2a a draw doubler with a printed exception | ✅ | `StaticEffect::ControllerDrawsDoubledExceptFirstEachDrawStep` — Alhammarret's Archive / Teferi's Ageless Insight's "except the first one you draw in each of **your** draw steps", read off `Player::cards_drawn_this_step`. Distinct from `ControllerDrawsDoubled` (Thought Reflection), whose doc used to name the Archive by mistake |
| "For **each** colour among permanents you control, add one mana of that colour" (Vivid) | ✅ | `ManaPayload::OneOfEachColorAmongYourPermanents` — Bloom Tender, Faeburrow Elder. ⚠ Distinct from `AnyColorAmongYourPermanents` (Meteor Crater), which reads the same set and adds **one** mana chosen from it; the two agree exactly at one colour, so a mono-colour test cannot separate them. The union walk is shared (`GameState::colors_among_your_permanents`) |
| CR 400.7 — a blink is a new object | 🟡 *in every respect a card can read* | `Effect::ExileAndReturnToOwner` moves the same instance out and back: tapped state, counters, attachments, summoning sickness and damage all reset, but the **`CardId` survives**. Anything keyed on the id (a delayed trigger, an `exiled_with` link) still matches where the rules say it lost track. Unobservable on all five shipped users, which are friendly self-blinks; ENGINE_BACKLOG carries it |
| CR 614.2 a damage multiplier that is not a power of two | ✅ | `StaticEffect::MultiplyDamageFromYourSources { factor }` — Fiery Emancipation / City on Fire, both ×3. The funnel accumulates *doublings* (`amount << doublers >> halvers`), so a triple rides its own accumulator and composes by multiplication (two of them is ×9). ⚠ A new damage-scaling static must also be listed in `static_effect_scales_damage`, or the `scale_damage_to` presence gate skips it silently; its `debug_assert_eq!` is what catches that in tests |
| CR 702.16 protection from a set the *game state* owns | ✅ | `Keyword::ProtectionFromColorsOutsideCommanderIdentity` — Commander's Plate's "each color that's not in your commander's color identity", the complement of a colour set no `SelectionRequirement` can name. Four separate gates (the `ProtectionKind` damage/equip/aura funnel, ability targeting, the cast-time spell gate, blocking), each with its own test. Reads `GameState::commander_identity_set`, the **raw** identity — `commander_identity_colors`' all-five fallback would mean no protection at all |
| CR 800.4a a player leaving: objects, stack items, control effects, the command zone, combat, a pending ask | ✅ | `stack.rs::objects_leave_with_player`, driven for **every** seat that is out by `Player::left_game` rather than by the loss SBA's own `newly_eliminated` list. The five arms that set `eliminated` directly (an unpaid Pact, `Effect::LoseGame`, a win-the-game effect, the graveyard-exile damage replacement) never ran the pass, and the sweep's `if eliminated { continue }` skipped them for ever |
| CR 800.4a (closing) / 800.4j priority never rests on a seat that left | ✅ | `mod.rs::priority_recipient`, read by `give_priority_to_active` and the untap-entry seed; `objects_leave_with_player` moves the grant already made. The turn itself continues — `active_player_idx` is left alone, which is what 800.4j says |
| CR 800.4b no token, and no control change, for a departed player | ✅ | `mod.rs::mint_token_with_counters`, `mod.rs::change_control` |
| CR 800.4d a departed player's triggered abilities aren't put on the stack | ✅ | `mod.rs::push_pending_trigger` (the one funnel for step / event / delayed pushes); `objects_leave_with_player` drops the `delayed_triggers` behind it |
| CR 800.4e no combat damage is assigned to a seat that has left | ✅ *by 800.4a* | `stack.rs::objects_leave_with_player` removes every attacker whose defending player left, so the reachable shape — the first-strike sub-step kills the defender and the regular sub-step follows — assigns nothing. Test `cr_800_4e_no_combat_damage_is_assigned_to_a_seat_that_left_mid_combat` |
| CR 800.4k a departed seat's turn does not begin | ✅ | `mod.rs::next_alive_seat`, read by `stack.rs::end_turn`. Test `cr_800_4k_a_departed_seats_turn_does_not_begin` |
| CR 800.4c a control effect ending with the default controller gone | ✅ | `mod.rs::revert_temporary_control` — the reversion site, not `objects_leave_with_player`: 800.4a's revert only touches permanents the departing seat controls *at that moment*, and in the three-deep shape (A owns X, B takes it permanently, C takes it until end of turn, B leaves) it controls none. `change_control` refuses the move under 800.4b and returns `None`, so without this C simply kept X for the rest of the game; now it is exiled. 800.4c's "no other effect giving control to another player" is asked of the **whole** registry rather than of the entries already processed, so it does not depend on registration order |
| CR 800.4m "until that player's next turn" ends when that turn *would* have begun | ✅ | `mod.rs::departed_seats_skipped_into_this_turn` → the Untap arm's expiry in `stack.rs`. Covers `UntilYourNextTurn`, `UntilYourNextUpkeep` (no upkeep of theirs is left) and CR 615's damage locks |
| CR 800.4f/g/h a choice owed by a departed player | ✅ | `game/departed.rs` — `route_ask` at the six `ask_seat_*` helpers. **800.4f**: a cost, or whether to pay one, is not paid and nobody is asked (`OptionalKind::is_cost`, the classifier the card already declares for the headless policy). **800.4g**: any other choice is re-seated by the object's controller — another *opponent* (`default_hostile_opponent`) where the departed chooser was one, the controller otherwise. **800.4h** needs no arm: every rule-required ask the engine suspends on (mulligan, cleanup discard, combat-damage order) names the asking seat's own cards, which 800.4a has already removed, so `objects_leave_with_player` still drops one. ⚠ The *pending* case is still a drop rather than a re-seat: an ask already materialized when its seat leaves needs the legal set re-derived against the post-departure board, which is a different capability. Rare now that no ask is posed to a departed seat in the first place |
| CR 800.4a — a seat that has left is not one of "each player" | ✅ | `mod.rs::seats_in_turn_order_from` (live seats in turn order from a given seat — a printed "starting with you, each player …", and `next_alive_seat` for "the player to their left") plus `resolve_players(&PlayerRef::EachPlayer, ctx)`, which is CR 101.4's order *and* the liveness filter. Six hand-written seat-index walks did neither; two were live bugs — `Effect::Vote` counted a departed seat's ballot (CR 800.4g re-seated its ask onto a live opponent, who answered, so nothing ever stalled) and Grenzo's Rebuttal aimed "the player to their left" at a board 800.4a had emptied. Ratcheted by `scripts/audit_seat_walks.py` (**0**), whose scope is the shape: a walk is a finding only when its body *asks* or *accumulates one entry per seat*. ENGINE_BACKLOG's forty-seventh find |
| CR 800.4 — no ask is ever posed to a seat that has left | ✅ | `departed.rs::seat_prompts` (`wants_ui && is_alive`), the one predicate behind `seat_suspends` and every suspend site. It replaced 102 hand-written `players[x].wants_ui` reads, none of which asked whether the seat was in the game; four-seat pods posed a tribute question to a departed seat in 4.4 % of games. Audited on every pod game by a `debug_assert!` in `pod::play_one_pod_game` |
| Which opponent an open choice aims at, at N > 2 | ✅ | `mod.rs::default_hostile_opponent` (scored by `hostile_opponent_score`; the bot's face attackers past a kill spill to the next-ranked seat, `server/pod_attack.rs`, CR 508.1b) — one ranked answer, shared by `bot.rs::attack_target_player` (the defender) and `targeting.rs` (an auto-filled "target opponent"). CR 903.10a commander race, then lowest effective life, then fewest untapped blockers; ties by seat index, so a fixed seed reproduces the run. One candidate in a duel, so the 1v1 answer and the golden traces are unchanged. The two positional picks it replaces both named seats that had already left the game, and the bot's named a *teammate* at 2HG. The auto-targeter's half was wired only once the gate audit's forty-second find closed (it was the bot's mana *estimate*, not the engine — see the CR 106.6 row); `first_alive_opponent_of` had no caller left and is gone. Aggregate over 32,000 pod games a side: identical at two seats, turns/game -0.06/0.00 at three, -0.24/-0.17 at four, -0.37/-0.43 at five, 100 % decided and zero stalls on both |
| Free-for-all last-player-standing, simultaneous-loss draw | ✅ | `team.rs`, `stack.rs` |
| CR 800.4i last-known information about a departed player | ⚠ *by unreachability* | Not modelled: a departed seat's zones are emptied by `objects_leave_with_player`, so a count derived from them reads 0 rather than the last known value. Near-unreachable — `resolve_players_unranged` filters `is_alive()` out of **every** fan-out (`EachPlayer`, `EachOpponent`, `EachOpponentExceptTriggerer`, `OpponentsWhoVotedDifferently`), so only an effect naming a *specific* departed seat could see it. 800.4i's second sentence (actions a departed player took) already works: `spell_names_cast_this_turn` and the rest live on `PlayerData` and are not cleared |
| CR 101.4 a per-player fan-out finishes when a body **suspends** | ✅ | `effects/mod.rs::splice_after_suspend` — a suspending body parks only its own remaining effect, so the loop around it used to abandon every seat it had not reached. Invisible in a duel (one iteration), three quarters of a four-seat pod. Twenty arms taken; the tail has to name its seats **inside** the effect (`PlayerRef::Seat(q)`) because a parked continuation resumes under the stack item's context. Ratcheted by `scripts/audit_loop_splice.py` (13/13/0, plus a staleness half that fails on an allowlist entry whose site is gone). `Effect::BindScratch` is the sibling pin for state that lives on `GameState` rather than in the context — the ballot a `VoteTally::PerVote` run belongs to, a results-table arm's die face — and closed `Vote`'s `PerVote` half and `RollDie`. The sequential *pair* (`run_piles_then_clear`) is the same defect without a loop and is closed by its splice alone. The arms still open are in ENGINE_BACKLOG's forty-third find |
| Multiplayer mulligan, no first-turn draw skip at 3+ | ✅ | `core_rules/multiplayer.rs` |
| A printed "**choose an opponent**" clause resolves to ONE seat | ✅ | `PlayerRef::HostileOpponent` → `mod.rs::default_hostile_opponent`. Nineteen catalog sites handed a **fan-out** ref to an effect arm that resolves `who` through the singular `resolve_player`, which answers with the first seat of the set and drops the rest — exact in a duel, seat order deciding the controller's choice in a pod, and for five of them ("each opponent sacrifices / discards / may scry") only ONE opponent was reached at all. `Effect::EachPlayerDoes` is the fan-out those arms cannot do for themselves. Ratcheted by `scripts/audit_singular_fanout.py`, which pairs the 102 singular-resolving arms with the catalog sites that feed one: **19 → 2**, both allowlisted with their reason. ENGINE_BACKLOG's fifty-first find |
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

### Simulation & tooling
| Feature | State | Where |
|---|---|---|
| N-seat pod runner | ✅ | `pod/mod.rs` — `build_pod_template`, `play_one_pod_game`, `run_pod_games` |
| One hundred and twenty-six legal target decks, a hundred and sixteen official lists | ✅ | `pod/decks.rs` — seat 126 **Oloro (WUB) Eternal Bargain (C13)**, seat 121 **Sidar Jabari (WUB) Cavalry Charge (MOC)**, seat 120 **Leinore (GW) Coven Counters (MIC)**, seat 124 **Millicent (WU) Spirit Squadron (VOC)**, seat 122 **Omo (GU) Tricky Terrain (M3C)**, seat 119 **Otrimi (BGU) Enhanced Evolution (C20)**, seat 117 **Kalamax (GUR) Arcane Maelstrom (C20)**, seat 118 **Aminatou (WUB) Miracle Worker (DSC)**, seat 116 **Olivia (RWB) Most Wanted (OTC)**, seat 113 **Zurgo (RWB) Mardu Surge (TDC)**, seat 111 **Jirina (RWB) Ruthless Regiment (C20)**, seat 109 **Chishiro (RG) Upgrades Unleashed (NEC)**, seat 106 **Galea (GWU) Aura of Courage (AFC)**, seat 102 **Estrid (GWU, a planeswalker commander) Adaptive Enchantment (C18)**, seat 97 **Eshki (GUR) Temur Roar (TDC)**, seat 95 **Lathliss (R) Reign of Dragons (FDC)**, seat 93 **Nelly Borca (RW) Blame Game (MKC)**, seat 51 **Yidris (UBRG) Entropic Uprising (C16)**, seat 56 **Ghired (RGW) Primal Genesis (C19)** and seat 59 **Valgavoth (BR) Endless Punishment (DSC)**, seat 64 **Saskia (WBRG) Open Hostility (C16)**, seat 70 **Ranar (WU) Phantom Premonition (KHC)**, seat 75 **Hazel (BG) Squirreled Away (BLC)**, seat 81 **Derevi (GWU) Evasive Maneuvers (C13)**, seat 87 **Atarka (RG) Draconic Destruction (SCD)**, seat 94 **Zinnia (URW) Family Matters (BLC)**, seat 96 **Ellivere (GW) Virtue and Valor (WOC)**, seat 101 **Kasla (URW) Divine Convocation (MOC)**, seat 103 **Kathril (WBG) Symbiotic Swarm (C20)**, seat 110 **Jared (WUBRG, a planeswalker commander) Painbow (DMC)**, seat 65 **Isperia (WU) First Flight (SCD)**, seat 66 **Inalla (UBR) Arcane Wizardry (C17)**, seat 68 **Anikthea (WBG) Enduring Enchantments (CMM)**, seat 69 **Urza (WUB) Urza's Iron Alliance (BRC)**, seat 72 **Faldorn (RG) Exit from Exile (CLB)**, seat 76 **Saheeli (UR, the sixth planeswalker commander) Exquisite Invention (C18)**, seat 83 **Mishra (UBR) Mishra's Burnished Banner (BRC)**, seat 84 **Gimbal (GUR) Tinker Time (MOC)**, seat 85 **Dihada (RWB, the seventh planeswalker commander) Legends' Legacy (DMC)**, seat 91 **Commodore Guff (URW, the eighth planeswalker commander) Planeswalker Party (CMM)**, seat 100 **Ulalek (WUBRG) Eldrazi Incursion (M3C)**, seat 108 **Zhulodok (C) Eldrazi Unbound (CMM)**, seat 114 **Aminatou (WUB, the ninth planeswalker commander) Subjective Reality (C18)**, seat 77 **Winter (BG) Death Toll (DSC)**, seat 89 **Jeleva (UBR) Mind Seize (C13)**, seat 92 **Go-Shintai (WUBRG) 20 Ways to Win (SLD)**, seat 98 **Kitt Kanto (RGW) Cabaretti Cacophony (NCC)**, seat 104 **Vrondiss (RG) Draconic Rage (AFC)**, seat 107 **Firkraag (UR) Draconic Dissent (CLB)**, seat 112 **Zaffai (UR) Prismari Performance (C21)** (past `MAX_SEATS`: `--pod-decks` only); seat 41 is **Neyali (RW) Rebellion Rising (ONC)** and seat 42 **Teferi (U, the fifth planeswalker commander) Peer Through Time (C14)**, seat 46 **Kalemne (RW) Wade into Battle (C15)**, seat 49 **Kardur (BR) Chaos Incarnate (SCD)**, seat 53 **Kaalia (RWB) Heavenly Inferno (CMD)**, seat 55 **Zedruu (URW) Political Puppets (CMD)**; **Teval (BGU) is Sultai Arisen (TDC)**, **N'ghathrod (UB) is Mind Flayarrrs (CLB)**, **Clavileño (WB) is Blood Rites (LCC)**, **Zndrsplt/Okaun (UR) is Heads I Win, Tails You Lose (SLD)** **Zada (R) is Goblin Storm (SLD)** and **Gisa (B) is Wretched Ranks (FDC)** and **Ghalta (G) is Tramplesaurus Rex (FDC)** and **Sai (U) is Keen Engineering (FDC)** and **Aesi (GU) is Reap the Tides (CMR)** and **Ixhel (WBG) is Corrupting Influence (ONC)** and **Anowon (UB) is Sneak Attack (ZNC)** and **Shiko and Narset (URW) is Jeskai Striker (TDC)** and **Sliver Gravemother (WUBRG) is Sliver Swarm (CMM)** and **Stella Lee (UR) is Quick Draw (OTC)** and a second **Edgar Markov (BRW) seat is Vampiric Bloodlust (C17)** and **Giada (W) is Calling All Angels (FDC)** and a second **Freyalise (G) seat is Guided by Nature (C14)** and **Bello (RG) is Animated Army (BLC)** and **Gisa and Geralf (UB) is Grave Danger (SCD)**, **Nahiri (W) is Forged in Stone (C14)**, **Disa (BRG) is Graveyard Overdrive (M3C)**, **Ezuri (GU) is Swell the Host (C15)**, **Gisela (W, the meld commander) is Angels (SLD)**, **Daretti (R) is Built From Scratch (C14)**, **Ob Nixilis (B) is Sworn to Darkness (C14)**, **Strefan (BR) is Vampiric Bloodline (VOC)** **Meren (BG) is Plunder the Graves (C15)** **Mizzix (UR) is Seize Control (C15)** and **Adrix and Nev (GU) is Quantum Quandrix (C21)** and **Daxos (WB) is Call the Spirits (C15)** and **Osgir (RW) is Lorehold Legacies (C21)** and **Ghave (WBG) is Counterpunch (CMD)** and **Wyleth (RW) is Arm for Battle (CMR)** and **Tegwyll (UB) is Fae Dominion (WOC)** and **Breya (WUBR) is Invent Superiority (C16)** and **Kardur (BR) is Chaos Incarnate (SCD)** and **Temmet (WUB) is Eternal Might (DRC)** and **Obuun (RGW) is Land's Wrath (ZNC)** and **Kynaios and Tiro (RGWU) is Stalwart Unity (C16)** and **Emmara (GW) is Token Triumph (SCD)** and **Rin and Seri (RGW) is Raining Cats and Dogs (SLD)** and **Brimaz (WB) is Growing Threat (MOC)** and **Bright-Palm (RGW) is Call for Backup (MOC)** and **Hearthhull (BRG) is World Shaper (EOC)** and **Ms. Bumbleflower (GWU) is Peace Offering (BLC)** and **Marath (RGW) is Nature of the Beast (C13)** (seats 29-40, 43-45, 47-50, 54, 57, 60, 63, 67, 74, 79, 82 and 88; the other sessions' seats 41, 42, 46, 51-53, 55, 56, 58, 59, 61, 62, 64-66, 68-73, 75-78, 80, 81 and 83-87 are listed in DECK_FEATURES), each card for card from MTGJSON's deck files (`scripts/precon_scan.py` ranks every precon by missing cards), appended after `pod_field(10)`..`pod_field(87)` (`--seats 11`..`88`, clamped to the engine's 64-seat mask `MAX_SEATS` — `--pod-decks` reaches the rest; the pod budget counts plays, `max(seats, 4) × 1,000`, since priority passes grow with seats²). Teval is the graveyard seat (Kotis's once-a-turn graveyard cast, CR 106.6 graveyard-only mana, Steward's granted land abilities); N'ghathrod the Horror mill-and-steal seat (CR 601.2b pay-one-of costs, CR 702.62b suspended triggers, CR 614 exile-if-it-would-leave). The ten before them: Sigarda GW / Judith BR / Hanna UW / Tatyova GU / Krark+Rograkh R (the Partner seat), **Edgar Markov BRW** — the first three-colour identity and the first **Eminence** commander in self-play, appended after `pod_field(5)` so no existing reading moves (`--seats 6` reaches it; it wins 52.1 % there, which DECK_FEATURES records as a finding about Eminence rather than a deck to tune) — and **Freyalise G**, the **planeswalker-commander** seat (CR 903.3a), appended after `pod_field(6)` for the same reason (`--seats 7`) — and **Zellix + Passionate Archaeologist UR**, the **Choose a Background** seat (CR 702.124k), appended after `pod_field(7)` (`--seats 8`) and the field's first Izzet identity; it runs the bond cycle's Training Center, a tapland in a duel and a dual in a pod. ⚠ Two tests used to find their deck with `last()`; appending broke the Partner one, and "two commanders" stopped being unique when the Background pair landed — the Partner test now asks for two *creature* commanders (CR 702.124h) and the planeswalker test for exactly *one* non-creature commander. Validated by the suite. Judith retuned 2026-09-19 (6.2 → 14.6 % of four-seat pods; DECK_FEATURES carries the experiment, whose reusable half is that a *better* aristocrats list measured worse — a one-ply material evaluator cannot price a value engine). Each runs the full colorless staple set: Sol Ring, Command Tower, Arcane Signet, Commander's Sphere, Mind Stone, Path of Ancestry, Opal Palace — and each two-color seat its guild Signet + Talisman (mono-red Krark takes neither: both cycles are two-color in CR 903.4 identity) — and **Yuriko, the Tiger's Shadow UB**, the **commander ninjutsu** seat (CR 702.49d), appended after `pod_field(8)` for the fourth time for the same reason (`--seats 9` reaches it; `bot_ladder`'s seat clamp is now `target_decks().len()` rather than a literal 8). It is the only seat whose commander leaves the command zone by an action that is **not a cast**, so CR 903.8's tax never applies to that route and `commander_cast_count` stays where it was — asserted by `cr_702_49d_a_commander_ninjutsu_seat_plays_a_pod_game`. The bot's `pick_ninjutsu` already read the command zone, so the swap is taken in real games — and **Adriana, Captain of the Guard RW**, the **multiplayer-native** seat, appended after `pod_field(9)` for the fifth time for the same reason (`--seats 10`). It exists because a census found that **not one of the nine decks above played a single multiplayer-native mechanic** — monarch (CR 725), goad (CR 701.15), melee (CR 702.121), voting (CR 701.38), tempting offer and join forces (ability words, CR 207.2c), the initiative (CR 726) and myriad (CR 702.116) were all implemented and all unreached by self-play. A mechanic the field never plays is a mechanic self-play never crashes on |
| 4-player Commander demo state | ✅ | `demo.rs::build_commander_state_seeded`, the first four of the same eight decks |
| Commander pod mode in `bot_ladder` | ✅ | `bot_ladder --commander [--seats N]`. **Runs on a debug build since `88f07f22`** — the worker was on the 2 MiB `scope.spawn` default and the first game of every batch aborted the process, so every pod figure committed before it is a `release-fast` statement and only that (ENGINE_BACKLOG's sixty-second find) |
| Per-deck card coverage in a pod run | ✅ | `bot_ladder --commander --card-census` — totals the action census by card and names, per seated deck, what a run never cast, played or activated. First run (10 seats, 2,000 games): **nine of the ten target decks played every card in the list**; **all ten now play every card in the list**, and the four official lists played all 100 on their first runs (14 seats, 1,000 games, seed 9940, 672 distinct cards). The first run left one — Yuriko's Agony Warp, castable by no path, since the hand sweep drops a `is_combat_trick` card unconditionally and the trick picker folded its two target slots into one pump aimed at our own creature (sixty-third find, fixed). 434 distinct cards -> 435 on the same seed |
| Golden outcomes for fixed-seed Commander pods | ✅ | `pod::tests::cr_903_seeded_pod_outcomes_match_the_committed_table` — three seeds' (winner, turns, actions) committed, cross-process. The triple rather than a line-per-action trace: a pod game is ~2,000 actions, so a real trace is a 400 KB file every Commander commit re-blesses, and it moves on the same changes |
| Bot takes every decision the pod decks introduce | 🟡 | `server/bot.rs` `cast_candidates` — escape, replicate, buyback, entwine, squad, fuse, casualty, bargain and retrace got candidate blocks 2026-09-23; a short payment floats spend-restricted sources (`pay_with_restricted_sources`, CR 106.6); an X spell's targets are picked at its X (Finale of Promise) and a pay-X-life X is sized from life (Toxic Deluge); a friendly pump prefers Zada. A 24-seat `--card-census` plays every card of all 24 lists. Open: 6 cast variants no bot emits (ENGINE_BACKLOG, none in a pod list) |
| A net / MCTS pilot in a pod | ⛔ by design | the observation encoder is two-seat (`encode_state_inner`'s `1 - seat`, `sources: &[_; 2]`); `bot_ladder --commander` refuses one above two seats rather than index out of bounds. Design notes for an N-seat encoding are in `ML_NOTES.md`; widening it retrains every net |

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
  `ColoredSpellTax` and the turn-scoped pool).
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
  (cleared on clean exit / match end).
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
  text; menu "Play Deck vs Bot" loads and validates). Remaining ⏳: export,
  .dec/.cod, paste-from-clipboard, choosing opponent's deck.
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
- **CR conformance batches**: `core_rules/cr_recent87`–`cr_recent98` — CR
  120.8, 613.11 (`effective_max_hand_size`), 701.19a, 604.4, 611.2c, 509.1c,
  704.5m, 702.26c, 707.4, 116.2b/116.3, 717.2/717.4/717.5.
- **Targeting**: 125 of 164 silent target-walker gaps closed, the rest
  ratcheted by `core_rules/target_walkers`; reflexive-cost target walking
  (`MaySacrifice` / `MaySacrificeSource`); `resolution_causer`.
- **Affordances / client**: free-cast hand affordance + "FREE" chip,
  regeneration-shield chip, SOS Special Guests naming.
- **Build**: `crabomination_client` type-checks in cloud sessions.

Open work lives in the tiers above; residual per-set approximations live in
`CARD_BACKLOG.md`.
