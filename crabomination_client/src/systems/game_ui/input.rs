//! The viewer's input: clicks and keys on the table and the HUD, turned into
//! game actions (casts, targeting, attacks, blocks, decisions).

use super::*;

/// Bundled mutable resources for `handle_game_input` to stay within Bevy's 16-param limit.
#[derive(bevy::ecs::system::SystemParam)]
pub struct GameInputResources<'w> {
    pub log: ResMut<'w, GameLog>,
    pub targeting: ResMut<'w, TargetingState>,
    pub blocking: ResMut<'w, BlockingState>,
    pub attacking: ResMut<'w, crate::game::AttackingState>,
    pub reveal: ResMut<'w, RevealPopupState>,
    pub menu_state: ResMut<'w, AbilityMenuState>,
    pub ff: ResMut<'w, FastForward>,
    pub alt_cast: ResMut<'w, crate::game::AltCastState>,
    pub flipped_hand: ResMut<'w, crate::game::FlippedHandCards>,
    pub card_names: ResMut<'w, crate::game::CardNames>,
    pub export_prompt: ResMut<'w, crate::systems::export_prompt::ExportPromptState>,
    pub debug_console: ResMut<'w, crate::systems::debug_console::DebugConsoleState>,
    pub chat: ResMut<'w, crate::systems::chat::ChatInputState>,
    pub legal_targets: ResMut<'w, crate::game::LegalTargets>,
    pub modal_cast: ResMut<'w, crate::game::PendingModalCast>,
    pub pay_times: ResMut<'w, crate::game::PayTimesState>,
    pub split_cast: ResMut<'w, crate::game::SplitCastState>,
    pub spree_cast: ResMut<'w, crate::game::SpreeCastState>,
    pub helper_tap: ResMut<'w, crate::game::HelperTapState>,
    pub hand_menu: ResMut<'w, hand_menu::HandMenuState>,
    /// Arbitrated Escape ownership, read at the three Esc sites below
    /// (blocker plan, attacker plan, targeting) so one press cancels one
    /// thing. Right-click keeps its own unarbitrated path.
    pub esc: Res<'w, crate::systems::esc::EscFocus>,
    /// A drag let go over what it acts on: taken as a click there
    /// (`systems::drag_act`).
    pub drag: Res<'w, crate::systems::drag_act::DragAct>,
}

/// Default seat for a freshly added attacker (and for "Attack All"): the
/// first *surviving* opponent in seat order. Eliminated players are no
/// longer in the game (CR 800.4a) and can't be attacked - without the
/// filter, a 4-player game whose first opponent has died defaults every
/// attack at the dead seat and the declaration bounces.
pub(crate) fn default_attack_target_seat(cv: &crabomination::net::ClientView) -> usize {
    cv.players
        .iter()
        .filter(|p| !p.eliminated)
        .map(|p| p.seat)
        // The attackers are the active player's, so the defender is anyone
        // else — which is the viewer when a `combat_chooser` declares for them.
        .find(|s| *s != cv.active_player)
        .unwrap_or(cv.your_seat)
}

/// Scan wire events for `TopCardRevealed` and arm the reveal popup if found.
fn check_reveal_wire(events: &[crabomination::net::GameEventWire], reveal: &mut RevealPopupState) {
    for ev in events {
        if let crabomination::net::GameEventWire::TopCardRevealed { player, card_name, .. } = ev {
            reveal.card_path = Some(crate::scryfall::card_asset_path(card_name));
            reveal.revealed_player = Some(*player);
        }
    }
}

// ── Player 0 input ────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn handle_game_input(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut r: GameInputResources,
    server_events: Res<LatestServerEvents>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered_hand: Query<&GameCardId, (With<CardHovered>, With<HandCard>)>,
    hovered_bf: Query<(&GameCardId, &CardOwner), (With<CardHovered>, With<BattlefieldCard>)>,
    hovered_stack: Query<&GameCardId, (With<CardHovered>, With<StackCard>, Without<HandCard>)>,
    hovered_target_zone: Query<&PlayerTargetZone, With<CardHovered>>,
    hovered_command_zone: Query<
        (&GameCardId, &crate::card::CommandZoneCard),
        With<CardHovered>,
    >,
    btns: Res<ButtonState>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
) {
    let log = &mut *r.log;
    let targeting = &mut *r.targeting;
    let blocking = &mut *r.blocking;
    let attacking = &mut *r.attacking;
    let reveal = &mut *r.reveal;
    let menu_state = &mut *r.menu_state;
    let ff = &mut *r.ff;
    let card_names = &mut *r.card_names;
    let legal_targets = &mut *r.legal_targets;
    let modal_cast = &mut *r.modal_cast;
    let esc = &*r.esc;

    // Refresh the card-name lookup from the current view so the event
    // formatter can resolve any CardId it sees. Hand / battlefield /
    // graveyard / stack are all included; opponent face-down hand
    // entries stay anonymous.
    if let Some(cv) = view.0.as_ref() {
        for player in &cv.players {
            for h in &player.hand {
                if let crabomination::net::HandCardView::Known(k) = h {
                    card_names.by_id.insert(k.id, k.name.clone());
                }
            }
            for g in &player.graveyard {
                card_names.by_id.insert(g.id, g.name.clone());
            }
        }
        for c in &cv.battlefield {
            card_names.by_id.insert(c.id, c.name.clone());
        }
        for item in &cv.stack {
            if let crabomination::net::StackItemView::Known(k) = item {
                card_names.by_id.insert(k.source, k.name.clone());
            }
        }
    }

    // Update game log from server events. A `TurnStarted` becomes a
    // turn-divider row (#5); every other event coalesces consecutive
    // duplicates via `push_event` (#7).
    let view_ref = view.0.as_ref();
    for ev in &server_events.0 {
        check_reveal_wire(std::slice::from_ref(ev), reveal);
        if let crabomination::net::GameEventWire::TurnStarted { player, turn } = ev {
            let who = view_ref
                .map(|cv| player_name(cv, *player))
                .unwrap_or_else(|| format!("P{player}"));
            log.push_divider(format!("──  Turn {turn} · {who}  ──"));
            // CR 103.5 — name the die-roll winner once, at the top of the
            // log, where the player is already looking.
            if *turn == 1
                && let Some(cv) = view_ref
            {
                let starter = player_name(cv, cv.starting_player);
                let you = cv.starting_player == cv.your_seat;
                // A game line, like the divider above it: a take-back of the
                // mulligan takes both out.
                log.push_event(
                    if you {
                        "You are on the play.".to_string()
                    } else {
                        format!("{starter} is on the play — you are on the draw.")
                    },
                    crate::theme::TEXT_BODY,
                );
            }
            continue;
        }
        // Internal/no-display events format to an empty body — skip them
        // rather than emitting a blank log row.
        let body = format_event(ev, card_names, view_ref);
        if body.is_empty() {
            continue;
        }
        let card = event_primary_card(ev)
            .map(|id| (id, card_names.get(id)))
            // "#N" placeholders mean the id never resolved to a real name
            // (hidden zones) — nothing to preview.
            .filter(|(_, n)| !n.starts_with('#'))
            .map(|(id, n)| crate::systems::ui_card_hover::UiCardHover::card(&n, Some(id)));
        log.push_event_with_card(
            format!("{}{}", event_glyph(ev), body),
            event_color(ev),
            card,
        );
    }

    let Some(cv) = &view.0 else { return };
    let Some(outbox) = outbox else { return };
    let your_seat = cv.your_seat;
    // Spectators occupy no seat (`your_seat == SPECTATOR_SEAT`) — they have no
    // gameplay shortcuts and indexing `players[your_seat]` below would panic.
    if your_seat >= cv.players.len() {
        return;
    }

    // Blocking mode is a once-per-combat declaration: it ends as soon as
    // this seat's blocks are in. Reset the flag the moment the step moves
    // on so a second combat phase (Aggravated Assault) blocks normally.
    if cv.step != TurnStep::DeclareBlockers {
        blocking.declared = false;
    }

    // While the export-state prompt is open the dedicated prompt system
    // owns the keyboard — bail out so typing the bug-description doesn't
    // also trigger gameplay shortcuts (Space → pass priority, etc.).
    if r.export_prompt.active {
        return;
    }

    // Same deal while the debug console's card-name field is focused —
    // typing into the buffer shouldn't fire `KeyCode::KeyA` (Attack All)
    // or other gameplay shortcuts.
    if r.debug_console.card_input_focused {
        return;
    }

    // And while the chat input bar is open.
    if r.chat.open {
        return;
    }

    // While a decision is pending for this viewer (mulligan, scry, search,
    // put-on-library, …), drop into decision-handling mode: the dedicated
    // decision UI systems own input — including 3D hand-card clicks for
    // PutOnLibrary — so we early-out here to avoid double-handling.
    //
    // Exception: `Decision::ChooseTarget` is satisfied via the in-scene
    // targeting cursor (highlights legal battlefield permanents, click
    // to pick), so we let the targeting branch below run and submit the
    // answer via `GameAction::SubmitDecision` instead of `CastSpell`.
    if let Some(pd) = &cv.pending_decision
        && pd.acting_player == your_seat
    {
        let is_choose_target = matches!(
            pd.decision.as_ref(),
            Some(crabomination::net::DecisionWire::ChooseTarget { .. })
        );
        if !is_choose_target {
            return;
        }
    }
    {
        // Click-or-Enter: a single boolean that captures either a
        // mouse-left click or the keyboard "activate" key. Used at
        // every `mouse.just_pressed(MouseButton::Left)` site so the
        // keyboard cursor (see `systems::kb_cursor`) drives the same
        // gameplay paths as the mouse.
        let activate = mouse.just_pressed(MouseButton::Left)
            || keyboard.just_pressed(KeyCode::Enter)
            || keyboard.just_pressed(KeyCode::NumpadEnter)
            || r.drag.release_click;

        // ── Blocking (defending against any opponent's attack) ──────────────
        // `!blocking.declared`: after the declaration is submitted the rest
        // of the declare-blockers step is a normal priority window (CR 509.4)
        // — falling through lets the viewer cast a combat trick before
        // damage instead of clicks being eaten by blocker selection.
        if cv.step == TurnStep::DeclareBlockers
            && cv.declares_blocks(your_seat)
            && cv.priority == your_seat
            && !blocking.declared
        {
            let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
            if mouse.just_pressed(MouseButton::Right)
                || esc.owns(crate::systems::esc::EscSurface::BlockerPlan)
            {
                blocking.selected_blocker = None;
                return;
            }
            if activate
                && let Some((game_id, owner)) = hovered_bf.iter().next()
            {
                if cv.may_declare_blocker(your_seat, owner.0) {
                    let already_assigned = blocking.assignments.iter().any(|(b, _)| *b == game_id.0);
                    let is_creature = cv.battlefield.iter().any(|c| c.id == game_id.0 && c.is_creature());
                    if is_creature && !already_assigned {
                        blocking.selected_blocker = Some(game_id.0);
                    }
                } else if let Some(blocker_id) = blocking.selected_blocker {
                    let is_attacker = cv.battlefield.iter().any(|c| c.id == game_id.0 && c.attacking);
                    // CR 509.1b — an illegal pairing would make the whole
                    // declaration illegal, so drop it here rather than let the
                    // server reject the batch.
                    if is_attacker && cv.block_is_legal(blocker_id, game_id.0) {
                        blocking.assignments.push((blocker_id, game_id.0));
                        blocking.selected_blocker = None;
                    }
                }
            }
            if pass {
                let assignments = std::mem::take(&mut blocking.assignments);
                blocking.selected_blocker = None;
                blocking.declared = true;
                // Submit the declaration *only* — even when it's empty, and
                // never bundled with a pass.
                //
                // CR 509.4: declaring blockers ends the declaration and
                // restarts the priority round with the active player, so the
                // defender gets a real window before combat damage. Two ways
                // this used to be thrown away:
                //   • Declining to block sent a bare `PassPriority`. That was
                //     the second pass of the round, so the engine auto-declared
                //     no blockers and advanced straight to CombatDamage — no
                //     window at all. An explicit empty declaration keeps the
                //     step alive instead.
                //   • After a real declaration the bundled pass was sent when
                //     priority had already moved to the active player, so the
                //     server rejected it ("seat N may not act now") and logged
                //     an error for a keypress the player had every right to.
                outbox.submit(GameAction::DeclareBlockers(assignments));
            }
            return;
        }

        // ── Attacker selection (our own DeclareAttackers, our priority) ─────
        //
        // Flow:
        //   • click an own creature → toggle in/out of the plan with a
        //     default target (next opponent). The toggled-in creature is
        //     stored as `last_added` so the next defender-click reassigns
        //     *its* target.
        //   • click an opponent's planeswalker → reassign `last_added` to
        //     that PW.
        //   • click an opponent's player disc (`PlayerTargetZone`) or 2-D
        //     HUD chip (`PlayerHudPanel`) → reassign `last_added` to that
        //     player.
        //   • Esc / right-click → clear plan.
        //   • `A` / Attack button (handled below) submits the plan, falling
        //     back to "attack all eligible at next opp" when empty.
        if cv.step == TurnStep::DeclareAttackers
            && cv.declares_attacks(your_seat)
            && cv.priority == your_seat
        {
            if mouse.just_pressed(MouseButton::Right)
                || esc.owns(crate::systems::esc::EscSurface::AttackPlan)
            {
                attacking.clear();
                // This used to fall through deliberately, so one Esc both
                // cleared the plan and closed an open ability menu. The
                // menu is an `EscSurface::Picker` and outranks the attack
                // plan, so that pairing is now two presses — which is the
                // point of one action per press.
            } else if activate {
                use crabomination::card::{CardType, Keyword};
                use crabomination::game::AttackTarget;
                let next_opp = default_attack_target_seat(cv);

                let mut consumed = false;
                if let Some((game_id, owner)) = hovered_bf.iter().next() {
                    let bf = cv.battlefield.iter().find(|c| c.id == game_id.0);
                    if cv.may_declare_attacker(your_seat, owner.0) {
                        let eligible = bf
                            .map(|c| {
                                c.is_creature()
                                    && !c.tapped
                                    && (!c.summoning_sick
                                        || c.keywords.contains(&Keyword::Haste))
                                    && (!c.keywords.contains(&Keyword::Defender)
                                        || c.can_attack_despite_defender)
                            })
                            .unwrap_or(false);
                        if eligible {
                            if attacking.contains(game_id.0) {
                                attacking.remove(game_id.0);
                            } else {
                                attacking
                                    .plan
                                    .push((game_id.0, AttackTarget::Player(next_opp)));
                                attacking.last_added = Some(game_id.0);
                            }
                            consumed = true;
                        }
                    } else {
                        // CR 506.2 — a "can't be attacked" walker (The
                        // Aetherspark while attached) isn't a legal redirect,
                        // so the click shouldn't arm a doomed declaration.
                        let is_pw = bf
                            .map(|c| {
                                c.card_types.contains(&CardType::Planeswalker)
                                    && !c.keywords.contains(&Keyword::CantBeAttacked)
                            })
                            .unwrap_or(false);
                        if is_pw
                            && attacking.set_target_for_last_added(
                                AttackTarget::Planeswalker(game_id.0),
                            )
                        {
                            consumed = true;
                        }
                    }
                }
                if !consumed
                    && let Some(zone) = hovered_target_zone.iter().next()
                    && zone.0 != cv.active_player
                    && attacking.set_target_for_last_added(AttackTarget::Player(zone.0))
                {
                    consumed = true;
                }
                if consumed {
                    return;
                }
            }
            // 2-D chip click during DeclareAttackers reassigns the
            // last-added attacker's defender. Mirrors the 3-D disc path
            // but always visible and easy to hit.
            if let Some(seat) = btns.player_chip
                && seat != cv.active_player
                && attacking.set_target_for_last_added(
                    crabomination::game::AttackTarget::Player(seat),
                )
            {
                return;
            }
        }

        // Off-priority input is dropped — with one exception: a pending
        // decision *acting on this seat*. A resolving opponent spell can
        // suspend on the viewer's choice ("each player sacrifices a creature
        // of their choice" — the bot's Social Snub) while priority still
        // sits with the opponent mid-resolution; the engine routes the
        // answer by `acting_player`, not by priority. Gating on priority
        // alone ate every click on the highlighted creatures and the pick
        // was unanswerable (recorded replay 2026-08-27: the game hung on
        // the sacrifice and the player quit). Trigger picks on your own
        // turn never hit this — acting player and priority coincide there.
        let deciding = cv
            .pending_decision
            .as_ref()
            .is_some_and(|pd| pd.acting_player == your_seat);
        if cv.game_over.is_some() || (cv.priority != your_seat && !deciding) {
            return;
        }

        // ── Targeting mode ────────────────────────────────────────────────────
        if targeting.active {
            if mouse.just_pressed(MouseButton::Right)
                || esc.owns(crate::systems::esc::EscSurface::Targeting)
            {
                // Esc / right-click during a decision-driven target
                // pick can't "cancel" — the engine is blocked waiting
                // for an answer. Tell the player why nothing happened
                // (a swallowed Esc otherwise reads as a bug) and keep
                // the pick armed. Spell / ability targeting can still
                // cancel back to normal mode.
                if !targeting.pending_decision_target {
                    cancel_targeting(targeting, legal_targets);
                } else if legal_targets.declinable {
                    // CR 601.4d — an "up to N targets" slot may be left
                    // empty. Escape is the Skip control for it; the engine
                    // ends target selection and finishes the cast.
                    outbox.submit(GameAction::SubmitDecision(
                        crabomination::decision::DecisionAnswer::DeclineTarget,
                    ));
                    cancel_targeting(targeting, legal_targets);
                } else {
                    let source = if legal_targets.source_name.is_empty() {
                        "the game".to_string()
                    } else {
                        legal_targets.source_name.clone()
                    };
                    log.push_event(
                        format!("{source} needs a target — this choice can't be cancelled."),
                        crate::theme::ACCENT_ORANGE,
                    );
                }
                return;
            }
            if activate {
                let is_ability_target = targeting.pending_ability_source.is_some();
                let cast_back = targeting.back_face_pending;
                let is_decision = targeting.pending_decision_target;
                // CR 115.4 — a spell on the stack is a legal target for a
                // counterspell. Stack cards carry `StackCard`, not
                // `BattlefieldCard`, so the permanent loop below never saw
                // them and "counter target spell" had nothing to click.
                for game_id in &hovered_stack {
                    let target = Target::Permanent(game_id.0);
                    if is_decision {
                        if !legal_targets.permanents.contains(&game_id.0) {
                            continue;
                        }
                        submit_decision_target(&outbox, cv, target);
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        let legal_set_populated = !legal_targets.permanents.is_empty()
                            || !legal_targets.players.is_empty();
                        if legal_set_populated
                            && !legal_targets.permanents.contains(&game_id.0)
                        {
                            continue;
                        }
                        outbox.submit(build_pending_cast(
                            pending_id,
                            Some(target),
                            targeting.pending_mode,
                            cast_back,
                            targeting.pending_pay_times,
                            targeting.pending_split,
                            targeting.pending_gift,
                            targeting.pending_omen,
                            targeting.pending_kicked,
                            targeting.pending_kicker_options.clone(),
                            targeting.pending_spree_modes.clone(),
                            targeting.pending_helpers.clone(),
                            targeting.pending_cast_variant,
                        ));
                    } else if let (Some(src), Some(idx)) =
                        (targeting.pending_ability_source, targeting.pending_ability_index)
                    {
                        outbox.submit(ability_target_action(targeting, src, idx, target));
                    } else {
                        continue;
                    }
                    cancel_targeting(targeting, legal_targets);
                    return;
                }
                for (game_id, _owner) in &hovered_bf {
                    let target = Target::Permanent(game_id.0);
                    // Equip (CR 702.6) takes precedence: the pending session
                    // is moving an Equipment onto the clicked creature.
                    if let Some(equipment) = targeting.pending_equip_source {
                        outbox.submit(GameAction::Equip { equipment, target: game_id.0 });
                        cancel_targeting(targeting, legal_targets);
                        return;
                    }
                    // CR 702.152 — Reconfigure moves an Equipment creature
                    // onto the clicked creature.
                    if let Some(equipment) = targeting.pending_reconfigure_source {
                        outbox.submit(GameAction::Reconfigure {
                            equipment,
                            target: Some(game_id.0),
                        });
                        cancel_targeting(targeting, legal_targets);
                        return;
                    }
                    if is_decision {
                        // Engine-driven `ChooseTarget`: only submit when
                        // the click lands on an enumerated legal target,
                        // so the user can't accidentally fire a no-op
                        // decision that bounces back with a GameError.
                        if !legal_targets.permanents.contains(&game_id.0) {
                            continue;
                        }
                        submit_decision_target(&outbox, cv, target);
                        cancel_targeting(targeting, legal_targets);
                        return;
                    } else if is_ability_target {
                        if let (Some(src), Some(idx)) = (targeting.pending_ability_source, targeting.pending_ability_index) {
                            outbox.submit(ability_target_action(targeting, src, idx, target));
                            cancel_targeting(targeting, legal_targets);
                            return;
                        }
                    } else if let Some(src) = targeting.pending_prepare_source {
                        // SOS Prepare — targeted prepare spell cast.
                        outbox.submit(GameAction::CastPrepareSpell {
                            creature_id: src, target: Some(target),
                            additional_targets: vec![], mode: targeting.pending_mode, x_value: None,
                        });
                        cancel_targeting(targeting, legal_targets);
                        return;
                    } else if targeting.pending_reinforce
                        && let Some(pending_id) = targeting.pending_card_id
                    {
                        // CR 702.77 — Reinforce discards the card to put
                        // +1/+1 counters on the picked creature; it is not a
                        // cast, so it never goes through `build_pending_cast`.
                        outbox.submit(GameAction::Reinforce { card_id: pending_id, target });
                        cancel_targeting(targeting, legal_targets);
                        return;
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        // Gate on the catalog-enumerated legal set when
                        // it was populated. Empty set falls back to
                        // "permissive" — same as ability targeting that
                        // we can't pre-enumerate.
                        let legal_set_populated = !legal_targets.permanents.is_empty()
                            || !legal_targets.players.is_empty();
                        let target_is_legal = match target {
                            Target::Permanent(id) => legal_targets.permanents.contains(&id),
                            Target::Player(s) => legal_targets.players.contains(&s),
                        };
                        if legal_set_populated && !target_is_legal {
                            continue;
                        }
                        let mode = targeting.pending_mode;
                        let action = build_pending_cast(
                            pending_id, Some(target), mode, cast_back, targeting.pending_pay_times,
                            targeting.pending_split,
                            targeting.pending_gift,
                            targeting.pending_omen,
                            targeting.pending_kicked,
                            targeting.pending_kicker_options.clone(),
                            targeting.pending_spree_modes.clone(),
                            targeting.pending_helpers.clone(),
                            targeting.pending_cast_variant,
                        );
                        outbox.submit(action);
                        cancel_targeting(targeting, legal_targets);
                        return;
                    }
                }
                for zone in &hovered_target_zone {
                    let target = Target::Player(zone.0);
                    if is_decision {
                        if !legal_targets.players.contains(&zone.0) {
                            continue;
                        }
                        submit_decision_target(&outbox, cv, target);
                        cancel_targeting(targeting, legal_targets);
                        return;
                    } else if is_ability_target {
                        if let (Some(src), Some(idx)) = (targeting.pending_ability_source, targeting.pending_ability_index) {
                            outbox.submit(ability_target_action(targeting, src, idx, target));
                            cancel_targeting(targeting, legal_targets);
                            return;
                        }
                    } else if let Some(src) = targeting.pending_prepare_source {
                        // SOS Prepare — targeted prepare spell cast.
                        outbox.submit(GameAction::CastPrepareSpell {
                            creature_id: src, target: Some(target),
                            additional_targets: vec![], mode: targeting.pending_mode, x_value: None,
                        });
                        cancel_targeting(targeting, legal_targets);
                        return;
                    } else if let Some(pending_id) = targeting.pending_card_id {
                        // Gate on the catalog-enumerated legal set when
                        // it was populated. Empty set falls back to
                        // "permissive" — same as ability targeting that
                        // we can't pre-enumerate.
                        let legal_set_populated = !legal_targets.permanents.is_empty()
                            || !legal_targets.players.is_empty();
                        let target_is_legal = match target {
                            Target::Permanent(id) => legal_targets.permanents.contains(&id),
                            Target::Player(s) => legal_targets.players.contains(&s),
                        };
                        if legal_set_populated && !target_is_legal {
                            continue;
                        }
                        let mode = targeting.pending_mode;
                        let action = build_pending_cast(
                            pending_id, Some(target), mode, cast_back, targeting.pending_pay_times,
                            targeting.pending_split,
                            targeting.pending_gift,
                            targeting.pending_omen,
                            targeting.pending_kicked,
                            targeting.pending_kicker_options.clone(),
                            targeting.pending_spree_modes.clone(),
                            targeting.pending_helpers.clone(),
                            targeting.pending_cast_variant,
                        );
                        outbox.submit(action);
                        cancel_targeting(targeting, legal_targets);
                        return;
                    }
                }
            }
            // 2-D HUD chip click — same Player target submission as the
            // 3-D disc above, but driven by `poll_player_chip_clicks` so
            // it fires independent of `activate` (the Button's Pressed
            // transition is the click signal).
            if let Some(seat) = btns.player_chip {
                let target = Target::Player(seat);
                let is_ability_target = targeting.pending_ability_source.is_some();
                let cast_back = targeting.back_face_pending;
                let is_decision = targeting.pending_decision_target;
                if is_decision {
                    if !legal_targets.players.contains(&seat) {
                        return;
                    }
                    submit_decision_target(&outbox, cv, target);
                    cancel_targeting(targeting, legal_targets);
                    return;
                } else if is_ability_target {
                    if let (Some(src), Some(idx)) = (
                        targeting.pending_ability_source,
                        targeting.pending_ability_index,
                    ) {
                        outbox.submit(GameAction::ActivateAbility {
                            card_id: src,
                            ability_index: idx,
                            target: Some(target),
                            additional_targets: Vec::new(),
                            x_value: None,
                            mode: targeting.pending_ability_mode,
                        });
                        cancel_targeting(targeting, legal_targets);
                        return;
                    }
                } else if let Some(src) = targeting.pending_prepare_source {
                    // SOS Prepare — targeted prepare spell from the
                    // ability menu; the picked target completes the cast.
                    outbox.submit(GameAction::CastPrepareSpell {
                        creature_id: src,
                        target: Some(target),
                        additional_targets: vec![],
                        mode: targeting.pending_mode,
                        x_value: None,
                    });
                    cancel_targeting(targeting, legal_targets);
                    return;
                } else if let Some(pending_id) = targeting.pending_card_id {
                    let legal_set_populated = !legal_targets.permanents.is_empty()
                        || !legal_targets.players.is_empty();
                    if legal_set_populated && !legal_targets.players.contains(&seat) {
                        return;
                    }
                    let mode = targeting.pending_mode;
                    let action = build_pending_cast(
                        pending_id, Some(target), mode, cast_back, targeting.pending_pay_times,
                            targeting.pending_split,
                            targeting.pending_gift,
                            targeting.pending_omen,
                            targeting.pending_kicked,
                            targeting.pending_kicker_options.clone(),
                            targeting.pending_spree_modes.clone(),
                            targeting.pending_helpers.clone(),
                            targeting.pending_cast_variant,
                    );
                    outbox.submit(action);
                    cancel_targeting(targeting, legal_targets);
                    return;
                }
            }
            return;
        }

        // ── Normal input ──────────────────────────────────────────────────────

        // Right-click P0 battlefield card → ability menu.
        // Right-click P0 hand card with an alt cost → alt-cast pitch modal.
        // Right-click P0 hand card with a back face (MDFC) → flip face.
        // Keyboard-only equivalents (act on the keyboard cursor's
        // currently-selected card via the same hovered_* queries —
        // `sync_kb_hover_marker` mirrors KeyboardSelected into
        // CardHovered):
        //   F = flip MDFC face         (was: right-click hand)
        //   L = open alt-cost modal    (was: right-click hand)
        //   M = open ability menu      (was: right-click bf)
        let kb_flip = keyboard.just_pressed(KeyCode::KeyF);
        let kb_alt = keyboard.just_pressed(KeyCode::KeyL);
        let kb_menu = keyboard.just_pressed(KeyCode::KeyM);
        let any_right = mouse.just_pressed(MouseButton::Right);
        // Right-click is "do whatever's most useful for this card": the
        // first quick-play shape `hand_menu::right_click` finds for a hand
        // card (the card's chip names it, `hand_chips`), the ability menu on
        // the battlefield. With keyboard we expose the three actions as
        // distinct keys (F / L / M) so the user can target whichever they
        // actually want — the right-click branch keeps its cascade.
        if any_right {
            if let Some(game_id) = hovered_hand.iter().next() {
                let card_id = game_id.0;
                let known: Option<crabomination::net::KnownCard> =
                    cv.players[your_seat].hand.iter().find_map(|h| match h {
                        crabomination::net::HandCardView::Known(k) if k.id == card_id => {
                            Some(k.clone())
                        }
                        _ => None,
                    });
                if let Some(k) = known {
                    use hand_menu::RightClick;
                    match hand_menu::right_click(cv, &k) {
                        RightClick::Pitch => {
                            // From-hand mana ability (Elvish / Simian Spirit
                            // Guide): the Guides' pitch is their only ability,
                            // so index 0; the engine validates the `from_hand`
                            // flag either way.
                            outbox.submit(GameAction::ActivateAbility {
                                card_id,
                                ability_index: 0,
                                target: None,
                                additional_targets: vec![],
                                x_value: None, mode: None,
                            });
                        }
                        RightClick::AltCost => {
                            r.alt_cast.pending = Some(card_id);
                            r.alt_cast.from_command_zone = false;
                        }
                        RightClick::PayTimes(mechanic) => {
                            r.pay_times.pending = Some((card_id, mechanic));
                            r.pay_times.times = 1;
                        }
                        RightClick::Spree => {
                            // Confirm casts via CastSpellSpree.
                            r.spree_cast.pending = Some(card_id);
                            r.spree_cast.labels = k.spree_mode_labels.clone();
                            r.spree_cast.selected = vec![false; k.spree_mode_labels.len()];
                            r.spree_cast.single_mode = k.spree_single_mode;
                        }
                        RightClick::SplitRight => r.split_cast.pending = Some(card_id),
                        // Arm targeting first when the gifted effect needs a
                        // target; otherwise fire now.
                        RightClick::Gift if k.gift_needs_target => {
                            targeting.active = true;
                            targeting.pending_card_id = Some(card_id);
                            targeting.pending_gift = true;
                        }
                        RightClick::Gift => outbox.submit(GameAction::CastGift {
                            card_id, target: None, additional_targets: vec![],
                            mode: None, x_value: None,
                        }),
                        RightClick::Omen if k.omen_needs_target => {
                            targeting.active = true;
                            targeting.pending_card_id = Some(card_id);
                            targeting.pending_omen = true;
                        }
                        RightClick::Omen => outbox.submit(GameAction::CastOmen {
                            card_id, target: None, additional_targets: vec![],
                            mode: None, x_value: None,
                        }),
                        RightClick::Splice => {
                            // Tick which partners in hand to splice on, then
                            // Cast. Spliced clause targets are auto-aimed
                            // server-side.
                            let splicers = cv.spliceable_hand.iter()
                                .find(|(host, _)| *host == card_id)
                                .map(|(_, s)| s.as_slice())
                                .unwrap_or_default();
                            let candidates: Vec<(CardId, String)> = splicers
                                .iter()
                                .filter_map(|sid| {
                                    cv.players.get(your_seat)?.hand.iter().find_map(|h| match h {
                                        crabomination::net::HandCardView::Known(hk)
                                            if hk.id == *sid =>
                                        {
                                            Some((*sid, hk.name.clone()))
                                        }
                                        _ => None,
                                    })
                                })
                                .collect();
                            r.helper_tap.open(card_id, crate::game::HelperMechanic::Splice, candidates, None);
                        }
                        RightClick::Helpers(mechanic) => {
                            // Tick untapped creatures (convoke / waterbend) or
                            // artifacts (improvise / waterbend) to help pay,
                            // then Cast.
                            let want_creatures = k.has_convoke || k.has_waterbend;
                            let want_artifacts = k.has_improvise || k.has_waterbend;
                            let candidates = helper_candidates(cv, |c| {
                                want_creatures && c.card_types.contains(&CardType::Creature)
                                    || want_artifacts && c.card_types.contains(&CardType::Artifact)
                            });
                            let cap = k.waterbend_amount
                                .filter(|_| mechanic == crate::game::HelperMechanic::Waterbend);
                            r.helper_tap.open(card_id, mechanic, candidates, cap);
                        }
                        RightClick::Conspire => {
                            let candidates = conspire_candidates(cv, &k);
                            r.helper_tap.open(card_id, crate::game::HelperMechanic::Conspire, candidates, Some(2));
                        }
                        RightClick::KickerOptions => {
                            // Pay the largest affordable "and/or" kicker subset
                            // (each rider is pure upside).
                            let kickers = cv.kicker_option_sets.iter()
                                .find(|(id, _)| *id == card_id)
                                .and_then(|(_, sets)| sets.iter().max_by_key(|s| s.len()).cloned())
                                .unwrap_or_default();
                            if k.needs_target {
                                targeting.active = true;
                                targeting.pending_card_id = Some(card_id);
                                targeting.pending_kicker_options = kickers;
                            } else {
                                outbox.submit(GameAction::CastSpellKickers {
                                    card_id, kickers, target: None, additional_targets: vec![],
                                    mode: None, x_value: None,
                                });
                            }
                        }
                        // Cast with the optional cost paid (`CastSpellKicked`),
                        // arming targeting first when the effect needs one.
                        RightClick::Kicked if k.needs_target => {
                            targeting.active = true;
                            targeting.pending_card_id = Some(card_id);
                            targeting.pending_kicked = true;
                        }
                        RightClick::Kicked => outbox.submit(GameAction::CastSpellKicked {
                            card_id, target: None, additional_targets: vec![],
                            mode: None, x_value: None,
                        }),
                        RightClick::Flip => {
                            if !r.flipped_hand.flipped.insert(card_id) {
                                r.flipped_hand.flipped.remove(&card_id);
                            }
                        }
                        RightClick::Menu => {
                            // No quick-play shape fits. Rather than doing
                            // nothing — which is what Foretell, Plot, Suspend,
                            // Bestow, Reinforce, morph and the Room doors used
                            // to do — offer every play the engine says is legal.
                            r.hand_menu.card_id = Some(card_id);
                            r.hand_menu.spawn_pos = windows.single().ok()
                                .and_then(|w| w.cursor_position())
                                .unwrap_or(Vec2::new(400.0, 300.0));
                        }
                    }
                }
            } else if let Some((game_id, owner)) = hovered_bf.iter().next() {
                if owner.0 == your_seat {
                    let card_id = game_id.0;
                    let has_menu_entry = cv.battlefield.iter().find(|c| c.id == card_id)
                        .is_some_and(has_ability_menu_entry);
                    if has_menu_entry {
                        menu_state.card_id = Some(card_id);
                        menu_state.spawn_pos = windows.single().ok()
                            .and_then(|w| w.cursor_position())
                            .unwrap_or(Vec2::new(400.0, 300.0));
                    } else {
                        menu_state.card_id = None;
                    }
                } else {
                    menu_state.card_id = None;
                }
            } else {
                menu_state.card_id = None;
            }
        }

        // Keyboard-specific: F flips the selected hand MDFC face.
        if kb_flip
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let has_back = cv.players[your_seat].hand.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == card_id && k.back_face_name.is_some())
            });
            if has_back && !r.flipped_hand.flipped.insert(card_id) {
                r.flipped_hand.flipped.remove(&card_id);
            }
        }

        // Keyboard-specific: L opens the alt-cost modal for the selected
        // hand card if it has one.
        if kb_alt
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let has_alt = cv.players[your_seat].hand.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == card_id && k.has_alternative_cost && k.alt_cost_available)
            });
            if has_alt {
                r.alt_cast.pending = Some(card_id);
                r.alt_cast.from_command_zone = false;
            }
        }

        // Right-click (or L) on your own commander in the command zone opens
        // the same alt-cost modal for dash / offering / Fist of Suns casts
        // (CR 601.2f); a plain left-click still casts it for its mana cost.
        if (any_right || kb_alt)
            && let Some((game_id, cz_card)) = hovered_command_zone.iter().next()
            && cz_card.owner == your_seat
            && cv.players[your_seat].command.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == game_id.0 && k.has_alternative_cost && k.alt_cost_available)
            })
        {
            r.alt_cast.pending = Some(game_id.0);
            r.alt_cast.from_command_zone = true;
        }

        // Keyboard-specific: C activates Cycling on the selected hand
        // card (CR 702.29). Submits `GameAction::Cycle` directly — the
        // server pays the cycling cost from the floated mana pool,
        // discards the card to graveyard, and draws.
        if keyboard.just_pressed(KeyCode::KeyC)
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let has_cycling = cv.players[your_seat].hand.iter().any(|h| {
                matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == card_id && k.has_cycling)
            });
            if has_cycling {
                outbox.submit(GameAction::Cycle { card_id, x_value: None });
            } else {
                // No plain Cycling — fall back to Landcycling (CR 702.29e) if
                // the card has it (fetch a land of the named type to hand).
                let has_landcycling = cv.players[your_seat].hand.iter().any(|h| {
                    matches!(h,
                        crabomination::net::HandCardView::Known(k)
                        if k.id == card_id && k.has_landcycling)
                });
                if has_landcycling {
                    outbox.submit(GameAction::Landcycle { card_id });
                }
            }
        }

        // Keyboard-specific: M opens the play-option menu for the hovered
        // hand card. Right-click still quick-plays whichever alternative
        // its cascade prefers, so M is how a player reaches the *other*
        // ways to play a card that has more than one.
        if kb_menu
            && let Some(game_id) = hovered_hand.iter().next()
        {
            let card_id = game_id.0;
            let playable = cv
                .players
                .get(your_seat)
                .is_some_and(|p| p.hand.iter().any(|h| h.id() == card_id))
                && !hand_menu::hand_play_options(cv, card_id).is_empty();
            if playable {
                r.hand_menu.card_id = Some(card_id);
                r.hand_menu.spawn_pos = windows.single().ok()
                    .and_then(|w| w.cursor_position())
                    .unwrap_or(Vec2::new(400.0, 300.0));
            }
        }

        // Keyboard-specific: M opens the ability menu for the selected
        // viewer-controlled battlefield card.
        if kb_menu
            && let Some((game_id, owner)) = hovered_bf.iter().next()
            && owner.0 == your_seat
        {
            let card_id = game_id.0;
            let has_non_mana = cv.battlefield.iter().find(|c| c.id == card_id)
                .is_some_and(has_ability_menu_entry);
            if has_non_mana {
                menu_state.card_id = Some(card_id);
                // Centre on the window if there's no cursor position.
                menu_state.spawn_pos = windows.single().ok()
                    .map(|w| Vec2::new(w.width() * 0.5, w.height() * 0.5))
                    .unwrap_or(Vec2::new(400.0, 300.0));
            }
        }

        // Keyboard-specific: E begins equip targeting on the hovered
        // viewer-controlled Equipment (CR 702.6). Sets
        // `pending_equip_source`; the next battlefield-creature click
        // submits `GameAction::Equip { equipment, target }`.
        if keyboard.just_pressed(KeyCode::KeyE)
            && let Some((game_id, owner)) = hovered_bf.iter().next()
            && owner.0 == your_seat
        {
            let card_id = game_id.0;
            let is_equippable = cv
                .battlefield
                .iter()
                .find(|c| c.id == card_id)
                .is_some_and(|c| c.equippable);
            if is_equippable {
                targeting.active = true;
                targeting.pending_equip_source = Some(card_id);
            }
        }

        // Close the ability menu when the user clicks anywhere else.
        // Not bound to Enter — the menu itself is mouse-driven today
        // and pressing Enter to close it would conflict with future
        // keyboard-menu support. Esc clears the cursor (handled in
        // `kb_cursor::handle_keyboard_cursor_input`).
        if mouse.just_pressed(MouseButton::Left) && menu_state.card_id.is_some() {
            menu_state.card_id = None;
        }

        let in_main = matches!(cv.step, TurnStep::PreCombatMain | TurnStep::PostCombatMain);
        if activate
            && let Some(game_id) = hovered_hand.iter().next()
        {
            // Find card details from the view.
            if let Some(card) = cv.players[your_seat].hand.iter().find_map(|h| {
                if let crabomination::net::HandCardView::Known(k) = h && k.id == game_id.0 { return Some(k.clone()); } None
            }) {
                use crabomination::card::CardType;
                let is_flipped = r.flipped_hand.flipped.contains(&card.id);
                let has_back = card.back_face_name.is_some();
                // Outside a main phase only instant-speed plays make
                // sense: instants always, and flipped MDFCs whenever the
                // engine says the back is castable right now (its type
                // isn't on the wire — `back_castable_hand` already
                // encodes the timing check). Everything else keeps the
                // old in-main gate so a stray click on a sorcery during
                // combat doesn't spam server rejections.
                let instant_speed = card.card_types.contains(&CardType::Instant)
                    || (is_flipped && cv.back_castable_hand.contains(&card.id));
                if in_main || instant_speed {
                    if card.card_types.contains(&CardType::Land) {
                        if is_flipped {
                            outbox.submit(GameAction::PlayLandBack(card.id));
                        } else {
                            outbox.submit(GameAction::PlayLand(card.id));
                        }
                    } else if has_back && is_flipped {
                        // Non-land MDFC played via its back face (the engine's
                        // `cast_spell_back_face` swaps the definition before
                        // validating cost / type / effect). The *back* face's
                        // own targeting flag decides whether to arm the cursor
                        // — `needs_target` describes only the front.
                        if card.back_needs_target {
                            targeting.active = true;
                            targeting.pending_card_id = Some(card.id);
                            targeting.back_face_pending = true;
                        } else {
                            outbox.submit(GameAction::CastSpellBack {
                                card_id: card.id, target: None, additional_targets: vec![], mode: None, x_value: None,
                            });
                        }
                    } else if !card.modal_descriptions.is_empty() {
                        // Modal "Choose one —" spell: pop the mode-pick modal
                        // and defer the cast until a mode is selected.
                        modal_cast.card_id = Some(card.id);
                        modal_cast.card_name = card.name.clone();
                        modal_cast.modes = card
                            .modal_descriptions
                            .iter()
                            .zip(card.modal_needs_target.iter().chain(std::iter::repeat(&false)))
                            .map(|(d, nt)| (d.clone(), *nt))
                            .collect();
                    } else if card.needs_target {
                        let legal = crate::systems::legal_target_filter::enumerate_for_cast(
                            cv,
                            &card.name,
                            None,
                        );
                        // If we enumerated the filter and nothing is legal (e.g.
                        // Beaming Defiance with no creatures you control), don't
                        // arm the targeting cursor — just tell the player.
                        let no_targets = legal
                            .as_ref()
                            .is_some_and(|l| l.permanents.is_empty() && l.players.is_empty());
                        if no_targets {
                            log.push(format!("No legal targets for {}.", card.name));
                        } else {
                            targeting.active = true;
                            targeting.pending_card_id = Some(card.id);
                            targeting.back_face_pending = false;
                            if let Some(l) = legal {
                                *legal_targets = l;
                            }
                        }
                    } else if cv.prototypable_hand.contains(&card.id)
                        && !cv.castable_hand.contains(&card.id)
                    {
                        // CR 702.160 — the full cost isn't affordable but the
                        // prototype cost is; click casts it for the smaller,
                        // colored face (no on-cast target).
                        outbox.submit(GameAction::CastPrototype { card_id: card.id, target: None, additional_targets: vec![], mode: None, x_value: None });
                    } else {
                        outbox.submit(GameAction::CastSpell { card_id: card.id, target: None, additional_targets: vec![], mode: None, x_value: None });
                    }
                }
            }
        }

        // ── Left-click own permanent → activate mana ability ──────────────
        // Lets the user manually select mana sources before casting (basic
        // lands, dual lands, mana-rock creatures). When the permanent has
        // exactly one mana ability that doesn't need a target, fire it
        // immediately. Multiple mana abilities open the ability menu so
        // the user can pick which colour. Permanents without any mana
        // ability fall through (right-click + M still open the non-mana
        // menu for those). Skips tapped permanents — there's nothing to
        // activate.
        if activate
            && menu_state.card_id.is_none()
            && !targeting.active
            && let Some((game_id, owner)) = hovered_bf.iter().next()
            && owner.0 == your_seat
            && let Some(perm) = cv.battlefield.iter().find(|c| c.id == game_id.0)
            && !perm.tapped
        {
            let mana_abilities: Vec<&crabomination::net::AbilityView> = perm
                .abilities
                .iter()
                .filter(|a| a.is_mana && !a.needs_target)
                .collect();
            match mana_abilities.len() {
                1 => {
                    outbox.submit(GameAction::ActivateAbility {
                        card_id: perm.id,
                        ability_index: mana_abilities[0].index,
                        target: None,
                        additional_targets: Vec::new(),
                        x_value: None, mode: None,
                    });
                }
                n if n > 1 => {
                    menu_state.card_id = Some(perm.id);
                    menu_state.spawn_pos = windows.single().ok()
                        .and_then(|w| w.cursor_position())
                        .unwrap_or(Vec2::new(400.0, 300.0));
                }
                _ => {}
            }
        }

        // ── Command zone click → cast from CZ ───────────────────────────
        // Phase L: clicking a card in the viewer's own command zone
        // routes through `CastFromCommandZone` so the {2}×N commander
        // tax is added on top of the printed cost. Targeted commanders
        // (rare — most commanders are vanilla legendaries) fall back to
        // the targeting prompt the same way `CastSpell` does. Foreign
        // command zones aren't clickable.
        if in_main && activate
            && let Some((game_id, cz_card)) = hovered_command_zone.iter().next()
            && cz_card.owner == your_seat
            && let Some(card) = cv.players[your_seat].command.iter().find_map(|h| {
                if let crabomination::net::HandCardView::Known(k) = h && k.id == game_id.0 {
                    return Some(k.clone());
                }
                None
            })
        {
            {
                // CR 902.5 — a Vanguard avatar is never cast; clicking it
                // activates its first command-zone ability instead.
                if let Some(ability) = card.zone_abilities.first() {
                    if ability.needs_target {
                        targeting.active = true;
                        targeting.pending_ability_source = Some(card.id);
                        targeting.pending_ability_index = Some(ability.index);
                        targeting.pending_ability_mode = None;
                        targeting.pending_ability_is_loyalty = false;
                        targeting.back_face_pending = false;
                    } else {
                        outbox.submit(GameAction::ActivateAbility {
                            card_id: card.id,
                            ability_index: ability.index,
                            target: None,
                            additional_targets: vec![],
                            mode: None,
                            x_value: None,
                        });
                    }
                } else if card.needs_target {
                    // Reuse the targeting cursor; the `CommandZone` variant
                    // makes the eventual pick submit `CastFromCommandZone`
                    // (tax included) rather than a hand `CastSpell`, which
                    // the engine rejects for a card outside the hand.
                    targeting.active = true;
                    targeting.pending_card_id = Some(card.id);
                    targeting.back_face_pending = false;
                    targeting.pending_cast_variant =
                        Some(crate::game::HandCastVariant::CommandZone);
                } else {
                    outbox.submit(GameAction::CastFromCommandZone {
                        card_id: card.id,
                        target: None,
                        additional_targets: vec![],
                        mode: None,
                        x_value: None,
                        // The alt-cost cast (CR 601.2f) is the right-click /
                        // L modal; a left-click pays the regular cost.
                        alternative: false,
                        pitch_card: None,
                    });
                }
            }
        }

        // Attack All / Confirm Attack (A). If the viewer has hand-picked
        // attackers via the per-creature click flow, submit *that* plan;
        // otherwise fall back to "attack all eligible at the next opponent"
        // so the one-key shortcut still works for single-opponent games.
        let attack = keyboard.just_pressed(KeyCode::KeyA) || btns.attack;
        if attack && cv.step == TurnStep::DeclareAttackers {
            use crabomination::game::{Attack, AttackTarget};
            let mut attacks: Vec<Attack> = if !attacking.plan.is_empty() {
                attacking
                    .plan
                    .iter()
                    .map(|(attacker, target)| Attack { attacker: *attacker, target: *target })
                    .collect()
            } else {
                let next_opp = default_attack_target_seat(cv);
                use crabomination::card::Keyword;
                cv.battlefield
                    .iter()
                    .filter(|c| {
                        c.owner == your_seat
                            && c.is_creature()
                            && !c.tapped
                            && (!c.summoning_sick || c.keywords.contains(&Keyword::Haste))
                            && (!c.keywords.contains(&Keyword::Defender)
                                || c.can_attack_despite_defender)
                    })
                    .map(|c| Attack {
                        attacker: c.id,
                        target: AttackTarget::Player(next_opp),
                    })
                    .collect()
            };
            // CR 506.2 — Silent Arbiter caps the combat; an over-sized batch
            // is rejected outright, so keep the biggest attackers.
            if let Some(cap) = cv.max_attackers_per_combat
                && attacks.len() > cap as usize
            {
                attacks.sort_by_key(|a| {
                    -cv.battlefield.iter().find(|c| c.id == a.attacker).map_or(0, |c| c.power)
                });
                attacks.truncate(cap as usize);
            }
            attacking.clear();
            outbox.submit(GameAction::DeclareAttackers(attacks));
        }

        // Pass Priority (Space)
        let pass = keyboard.just_pressed(KeyCode::Space) || btns.pass;
        if pass {
            outbox.submit(GameAction::PassPriority);
        }

        // End Turn (E) — set fast-forward to skip to opponent's turn
        let end_turn = keyboard.just_pressed(KeyCode::KeyE) || btns.end_turn;
        if end_turn {
            ff.end_turn = true;
            outbox.submit(GameAction::PassPriority);
        }

        // Next Turn (N) — fast-forward through opponent's turn to our next main
        let next_turn = keyboard.just_pressed(KeyCode::KeyN) || btns.next_turn;
        if next_turn {
            ff.next_turn = true;
            outbox.submit(GameAction::PassPriority);
        }

    }
}

/// Build the cast action for a pending spell-targeting session, honoring the
/// MDFC back-face flag and any Squad / Replicate / Multikicker pay-times rider.
#[allow(clippy::too_many_arguments)]
fn build_pending_cast(
    card_id: CardId,
    target: Option<Target>,
    mode: Option<usize>,
    cast_back: bool,
    pay_times: Option<(u32, crate::game::PayTimesMechanic)>,
    split: Option<crate::game::SplitCastChoice>,
    gift: bool,
    omen: bool,
    kicked: bool,
    kicker_options: Vec<u8>,
    spree_modes: Option<Vec<u8>>,
    helpers: Option<(Vec<CardId>, crate::game::HelperMechanic)>,
    variant: Option<crate::game::HandCastVariant>,
) -> GameAction {
    // An alternative cast shape chosen from the hand play-option menu
    // (Bestow, an Adventure half, Buyback, ...). Exclusive with the flag
    // soup below — the menu arms exactly one at a time.
    if let Some(variant) = variant {
        return variant.action(card_id, target);
    }
    // CR 702.51 / 702.126 / 701.67 — tapped helpers ride their own cast action.
    if let Some((helpers, mechanic)) = helpers {
        return popups::helper_cast_action(mechanic, card_id, helpers, target, mode);
    }
    // CR 702.172 — a Spree/Tiered cast carries its chosen mode indices.
    if let Some(spree_modes) = spree_modes {
        return GameAction::CastSpellSpree {
            card_id, spree_modes, target, additional_targets: vec![], x_value: None,
        };
    }
    // CR 702.165 — a promised Gift is its own cast action; it can't combine
    // with the split / pay-times / back-face shapes.
    if gift {
        return GameAction::CastGift {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        };
    }
    // CR 702.183 — an Omen half is its own cast action.
    if omen {
        return GameAction::CastOmen {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        };
    }
    // CR 702.33b — an "and/or" kicker cast carries its chosen option indices.
    if !kicker_options.is_empty() {
        return GameAction::CastSpellKickers {
            card_id, kickers: kicker_options, target,
            additional_targets: vec![], mode, x_value: None,
        };
    }
    // CR 702.32 / 702.166 — Kicker / Offspring paid via the kicked cast path.
    if kicked {
        return GameAction::CastSpellKicked {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        };
    }
    match (split, pay_times) {
        (Some(crate::game::SplitCastChoice::Right), _) => GameAction::CastSplitRight {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        },
        (Some(crate::game::SplitCastChoice::Fused), _) => GameAction::CastSplitFused {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        },
        (None, Some((times, mechanic))) => {
            popups::pay_times_cast_action(mechanic, card_id, times, target, mode)
        }
        (None, None) if cast_back => GameAction::CastSpellBack {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        },
        (None, None) => GameAction::CastSpell {
            card_id, target, additional_targets: vec![], mode, x_value: None,
        },
    }
}

/// Does right-clicking this permanent open the ability menu? True for a
/// non-mana activated ability, a planeswalker's loyalty abilities (CR 606
/// — these live in their own list, and a walker often has *no* activated
/// abilities at all, which used to leave it with no menu and so no way to
/// be used), or a prepared preparation card's "Cast <spell>" entry (SOS
/// Prepare — a vanilla prepared creature has no abilities but still offers
/// its spell).
pub(crate) fn has_ability_menu_entry(c: &crabomination::net::PermanentView) -> bool {
    c.abilities.iter().any(|a| !a.is_mana)
        || !c.loyalty_abilities.is_empty()
        || (c.prepare_spell_name.is_some()
            && c.counters.iter().any(|(k, n)| {
                *k == crabomination::card::CounterType::Prepared && *n > 0
            }))
}

/// CR 702.78 — the creatures that can pay `spell`'s conspire: untapped,
/// yours, and sharing a colour with it (its mana cost's colours; the engine
/// checks the pair either way).
fn conspire_candidates(cv: &crabomination::net::ClientView, spell: &crabomination::net::KnownCard) -> Vec<(CardId, String)> {
    let colors = spell.cost.color_set();
    helper_candidates(cv, |c| {
        c.card_types.contains(&CardType::Creature) && c.colors.iter().any(|col| colors.contains(*col))
    })
}

/// The viewer's untapped permanents `fits` accepts, as the helper picker
/// lists them: "Name (2/2)" for a creature, the name for anything else.
fn helper_candidates(
    cv: &crabomination::net::ClientView,
    fits: impl Fn(&crabomination::net::PermanentView) -> bool,
) -> Vec<(CardId, String)> {
    cv.battlefield
        .iter()
        .filter(|c| c.controller == cv.your_seat && !c.tapped && fits(c))
        .map(|c| {
            let label = if c.card_types.contains(&CardType::Creature) {
                format!("{} ({}/{})", c.name, c.power, c.toughness)
            } else {
                c.name.clone()
            };
            (c.id, label)
        })
        .collect()
}

/// The action an ability-targeting session submits for `target`. Loyalty
/// abilities (CR 606) live in their own list on the permanent, with their
/// own index space, so they route through `ActivateLoyaltyAbility` —
/// sending index 1 of a walker's loyalty list as `ActivateAbility` would
/// hit an unrelated activated ability, or nothing at all.
/// Answer a `Decision::ChooseTarget` with `target`.
///
/// When the decision is a cast-time extra target slot (the engine's
/// `CastExtraTargetPick` suspend for a multi-target spell such as Knockout
/// Maneuver), the pick is also folded into the cast action the outbox is
/// holding. The engine replays that cast server-side with the slot filled;
/// if the replay then bounces with `ManualTapRequired`, the held action has
/// to carry the target too, or every mana tap re-poses this same prompt.
///
/// Keyed on the decision's `extra_cast_slot` flag. It used to compare
/// `description` against `EXTRA_CAST_TARGET_PROMPT`, so the fold-back went
/// silent the moment that prompt started naming the slot it asks about —
/// and Vibrant Outburst went back to re-prompting on every mana tap.
fn submit_decision_target(
    outbox: &crate::net_plugin::NetOutbox,
    cv: &crabomination::net::ClientView,
    target: Target,
) {
    use crabomination::net::DecisionWire;
    if let Some(DecisionWire::ChooseTarget { source, extra_cast_slot: true, .. }) =
        cv.pending_decision.as_ref().and_then(|pd| pd.decision.as_ref())
    {
        outbox.patch_last_cast_extra_target(*source, target.clone());
    }
    outbox.submit(GameAction::SubmitDecision(
        crabomination::decision::DecisionAnswer::Target(target),
    ));
}

fn ability_target_action(
    targeting: &TargetingState,
    source: CardId,
    ability_index: usize,
    target: Target,
) -> GameAction {
    if targeting.pending_ability_is_loyalty {
        GameAction::ActivateLoyaltyAbility {
            card_id: source,
            ability_index,
            target: Some(target),
            x_value: None,
        }
    } else {
        GameAction::ActivateAbility {
            card_id: source,
            ability_index,
            target: Some(target),
            additional_targets: Vec::new(),
            x_value: None,
            mode: targeting.pending_ability_mode,
        }
    }
}

fn cancel_targeting(targeting: &mut TargetingState, legal: &mut crate::game::LegalTargets) {
    targeting.active = false;
    targeting.pending_card_id = None;
    targeting.pending_ability_source = None;
    targeting.pending_ability_index = None;
    targeting.pending_ability_mode = None;
    targeting.pending_ability_is_loyalty = false;
    targeting.back_face_pending = false;
    targeting.pending_decision_target = false;
    targeting.pending_mode = None;
    targeting.pending_equip_source = None;
    targeting.pending_reconfigure_source = None;
    targeting.pending_prepare_source = None;
    targeting.pending_pay_times = None;
    targeting.pending_spree_modes = None;
    targeting.pending_split = None;
    targeting.pending_gift = false;
    targeting.pending_omen = false;
    targeting.pending_kicked = false;
    targeting.pending_kicker_options.clear();
    targeting.pending_helpers = None;
    targeting.pending_cast_variant = None;
    targeting.pending_reinforce = false;
    legal.permanents.clear();
    legal.players.clear();
    legal.source_name.clear();
    legal.description.clear();
    legal.declinable = false;
}

#[cfg(test)]
mod conspire_tests {
    use super::*;
    use crabomination::catalog;

    /// CR 702.78 — the conspire picker offers the untapped creatures you
    /// control that share a colour with the spell, and nothing else: not a
    /// creature of another colour, not a tapped one, not an opponent's.
    #[test]
    fn conspire_offers_your_untapped_creatures_that_share_its_colour() {
        let mut g = crabomination::game::two_player_game();
        let spell = g.add_card_to_hand(0, catalog::burn_trail());
        let red_a = g.add_card_to_battlefield(0, catalog::raging_goblin());
        let red_b = g.add_card_to_battlefield(0, catalog::goblin_guide());
        let tapped = g.add_card_to_battlefield(0, catalog::monastery_swiftspear());
        g.add_card_to_battlefield(0, catalog::grizzly_bears());
        g.add_card_to_battlefield(1, catalog::raging_goblin());
        if let Some(c) = g.battlefield_find_mut(tapped) {
            c.tapped = true;
        }
        let cv = crabomination::server::view::project(&g, 0);
        let k = cv.players[0].hand.iter().find_map(|h| match h {
            crabomination::net::HandCardView::Known(k) if k.id == spell => Some(k.clone()),
            _ => None,
        });
        let ids: Vec<CardId> = conspire_candidates(&cv, &k.expect("in hand")).into_iter().map(|(id, _)| id).collect();
        assert_eq!(ids, [red_a, red_b]);
    }

    /// The picker holds Cast until exactly two are ticked, and the pair
    /// rides `CastSpellConspire`.
    #[test]
    fn conspire_casts_with_exactly_two() {
        use crate::game::{HelperMechanic, HelperTapState};
        let (spell, a, b, c) = (CardId(1), CardId(2), CardId(3), CardId(4));
        let mut state = HelperTapState::default();
        state.open(spell, HelperMechanic::Conspire, vec![(a, "A".into()), (b, "B".into()), (c, "C".into())], Some(2));
        assert!(!state.ready());
        state.selected[0] = true;
        assert!(!state.ready(), "one isn't enough");
        state.selected[2] = true;
        assert!(state.ready());
        assert!(matches!(
            popups::helper_cast_action(HelperMechanic::Conspire, spell, vec![a, c], None, None),
            GameAction::CastSpellConspire { card_id, conspire_creatures, .. }
                if card_id == spell && conspire_creatures == [a, c]
        ));
        // Convoke takes any number.
        state.open(spell, HelperMechanic::Convoke, vec![(a, "A".into())], None);
        assert!(state.ready());
    }
}
