//! Floating modal-ish surfaces: the right-click ability context menu
//! and the alt-cast (pitch / evoke) picker. Both share the pattern of
//! "open conditionally on a state resource, despawn when state clears."

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::{GameAction, Target};

use crate::card::DeckPile;
use crate::game::{AbilityMenuState, TargetingState};
use crate::net_plugin::{CurrentView, NetOutbox};
use crate::systems::ui::RevealPopupState;
use crate::theme::{self, UiFonts};

// ── Ability menu (right-click context menu on own battlefield cards) ─────────

/// Root entity of the floating ability context menu.
#[derive(Component)]
pub struct AbilityMenu;

/// One item in the ability menu.
#[derive(Component)]
pub struct AbilityMenuItem {
    pub card_id: CardId,
    pub ability_index: usize,
}

/// Spawn or despawn the floating ability context menu based on `AbilityMenuState`.
pub fn spawn_ability_menu(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    menu_state: Res<AbilityMenuState>,
    existing: Query<Entity, With<AbilityMenu>>,
) {
    if !menu_state.is_changed() { return; }

    for e in &existing {
        commands.entity(e).despawn();
    }

    let Some(card_id) = menu_state.card_id else { return };
    let Some(cv) = view.0.as_ref() else { return };
    let Some(pv) = cv.battlefield.iter().find(|p| p.id == card_id) else { return };

    // Project (index, label, used_this_turn) so the menu can grey out
    // once-per-turn abilities that have already been activated. Includes
    // mana abilities so multi-colour lands surface all the pip choices
    // when opened via the left-click mana-source picker (see the
    // `mana_abilities.len() > 1` branch in `handle_game_input`).
    // An `opponents_only` ability (CR 602.5 — Detention Vortex's escape) can't
    // be activated by the permanent's controller, so grey it out in their own
    // menu just like a once-per-turn ability that's already been used.
    let viewer_controls = pv.controller == cv.your_seat;
    let abilities: Vec<(usize, String, bool)> = pv.abilities.iter()
        .map(|a| {
            let blocked_opp = a.opponents_only && viewer_controls;
            let label = if a.once_per_turn_used {
                format!("{}: {} (used)", a.cost_label, a.effect_label)
            } else {
                format!("{}: {}", a.cost_label, a.effect_label)
            };
            (a.index, label, a.once_per_turn_used || blocked_opp)
        })
        .collect();
    if abilities.is_empty() { return; }
    let card_name = pv.name.clone();

    let pos = menu_state.spawn_pos;

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(pos.x),
                top: Val::Px(pos.y),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG_SUNKEN),
            AbilityMenu,
        ))
        .with_children(|menu| {
            menu.spawn((
                Text::new(card_name),
                ui_fonts.tf(13.0),
                TextColor(theme::ACCENT_BLUE),
            ));
            for (ability_index, label, used) in abilities {
                let bg = if used {
                    // Darkened background for once-per-turn abilities
                    // already activated this turn — clicks still go
                    // through, but the engine returns
                    // `AbilityAlreadyUsedThisTurn`.
                    theme::PANEL_BG_RAISED
                } else {
                    theme::BUTTON_NEUTRAL_BG
                };
                let fg = if used {
                    theme::TEXT_MUTED
                } else {
                    theme::TEXT_PRIMARY
                };
                menu.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    AbilityMenuItem { card_id, ability_index },
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        ui_fonts.tf(13.0),
                        TextColor(fg),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}

/// Handle clicks on ability menu items.
pub fn handle_ability_menu(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut targeting: ResMut<TargetingState>,
    mut menu_state: ResMut<AbilityMenuState>,
    query: Query<(&Interaction, &AbilityMenuItem), Changed<Interaction>>,
) {
    for (interaction, item) in &query {
        if *interaction != Interaction::Pressed { continue; }

        let Some(cv) = &view.0 else { menu_state.card_id = None; continue };
        let Some(pv) = cv.battlefield.iter().find(|p| p.id == item.card_id) else { menu_state.card_id = None; continue };
        let Some(av) = pv.abilities.iter().find(|a| a.index == item.ability_index) else { menu_state.card_id = None; continue };

        if av.needs_target {
            targeting.active = true;
            targeting.pending_card_id = None;
            targeting.pending_ability_source = Some(item.card_id);
            targeting.pending_ability_index = Some(item.ability_index);
        } else if let Some(ob) = &outbox {
            ob.submit(GameAction::ActivateAbility {
                card_id: item.card_id,
                ability_index: item.ability_index,
                target: None,
                x_value: None,
            });
        }

        menu_state.card_id = None;
    }
}

// ── Alt-cast (pitch / evoke) modal ───────────────────────────────────────────

#[derive(Component)]
pub struct AltCastModal;

#[derive(Component)]
pub struct AltCastPitchButton {
    pub spell: CardId,
    /// `Some(card)` for pitch alt costs (exile a hand card); `None` for plain
    /// alternative costs (Surge/Awaken/Emerge/Spectacle/Overload).
    pub pitch: Option<CardId>,
}

#[derive(Component)]
pub struct AltCastCancelButton;

/// Spawn or despawn the alt-cast pitch picker based on `AltCastState`.
pub fn spawn_alt_cast_modal(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    state: Res<crate::game::AltCastState>,
    existing: Query<Entity, With<AltCastModal>>,
) {
    let want_open = state.pending.is_some();
    let is_open = !existing.is_empty();
    if !state.is_changed() && want_open == is_open {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    let Some(spell_id) = state.pending else { return };
    let Some(cv) = &view.0 else { return };

    // Look up the pending spell's alt-cost shape: pitch (exile a hand card)
    // vs. plain alternative (Surge/Awaken/Emerge/Spectacle/Overload).
    let (needs_pitch, alt_label) = cv
        .players
        .get(cv.your_seat)
        .and_then(|p| {
            p.hand.iter().find_map(|h| match h {
                crabomination::net::HandCardView::Known(k) if k.id == spell_id => {
                    Some((k.alt_cost_needs_pitch, k.alt_cost_label.clone()))
                }
                _ => None,
            })
        })
        .unwrap_or((true, String::new()));

    // Collect candidate pitch cards: every other Known card in the viewer's
    // hand. The engine validates the filter and returns InvalidPitchCard if
    // wrong color; we render every card so the player sees the full hand.
    let candidates: Vec<(CardId, String)> = cv
        .players
        .get(cv.your_seat)
        .map(|p| {
            p.hand
                .iter()
                .filter_map(|h| match h {
                    crabomination::net::HandCardView::Known(k) if k.id != spell_id => {
                        Some((k.id, k.name.clone()))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
            AltCastModal,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    min_width: Val::Px(360.0),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|panel| {
                let header = if needs_pitch {
                    "Cast for alternative cost — pick a card to exile:".to_string()
                } else if alt_label.is_empty() {
                    "Cast for alternative cost?".to_string()
                } else {
                    format!("Cast for alternative cost ({alt_label})?")
                };
                panel.spawn((
                    Text::new(header),
                    ui_fonts.tf(15.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
                if !needs_pitch {
                    // Plain alternative cost (Surge/Awaken/Emerge/Spectacle/
                    // Overload): no hand card to exile — one confirm button.
                    panel
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(theme::BUTTON_INFO_BG),
                            AltCastPitchButton { spell: spell_id, pitch: None },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new("Cast"),
                                ui_fonts.tf(13.0),
                                TextColor(theme::TEXT_PRIMARY),
                                bevy::picking::Pickable::IGNORE,
                            ));
                        });
                }
                if needs_pitch && candidates.is_empty() {
                    panel.spawn((
                        Text::new("(no other cards in hand to pitch)"),
                        ui_fonts.tf(13.0),
                        TextColor(theme::TEXT_SECONDARY),
                    ));
                }
                if needs_pitch {
                    for (pid, name) in candidates {
                        panel
                            .spawn((
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                    ..default()
                                },
                                BackgroundColor(theme::BUTTON_INFO_BG),
                                AltCastPitchButton { spell: spell_id, pitch: Some(pid) },
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new(name),
                                    ui_fonts.tf(13.0),
                                    TextColor(theme::TEXT_PRIMARY),
                                    bevy::picking::Pickable::IGNORE,
                                ));
                            });
                    }
                }
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_DANGER_BG),
                        AltCastCancelButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Cancel"),
                            ui_fonts.tf(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                            bevy::picking::Pickable::IGNORE,
                        ));
                    });
            });
        });
}

/// Click on a pitch button → submit `CastSpellAlternative`. Click cancel →
/// clear the pending alt-cast.
pub fn handle_alt_cast_buttons(
    mut state: ResMut<crate::game::AltCastState>,
    outbox: Option<Res<NetOutbox>>,
    pitch_q: Query<(&Interaction, &AltCastPitchButton), Changed<Interaction>>,
    cancel_q: Query<&Interaction, (Changed<Interaction>, With<AltCastCancelButton>)>,
) {
    if cancel_q.iter().any(|i| *i == Interaction::Pressed) {
        state.pending = None;
        return;
    }
    for (interaction, btn) in &pitch_q {
        if *interaction == Interaction::Pressed
            && let Some(outbox) = &outbox
        {
            outbox.submit(GameAction::CastSpellAlternative {
                card_id: btn.spell,
                pitch_card: btn.pitch,
                target: None,
                additional_targets: vec![],
                mode: None,
                x_value: None,
            });
            state.pending = None;
            return;
        }
    }
}

// ── Squad / Replicate / Multikicker pay-times stepper ───────────────────────

#[derive(Component)]
pub struct PayTimesModal;

#[derive(Component)]
pub struct PayTimesStepButton {
    /// +1 / -1 on the times counter.
    pub delta: i32,
}

#[derive(Component)]
pub struct PayTimesConfirmButton;

#[derive(Component)]
pub struct PayTimesCancelButton;

/// Marker on the "×N" readout text so the handler can update it in place.
#[derive(Component)]
pub struct PayTimesReadout;

/// Spawn or despawn the Squad/Replicate stepper based on `PayTimesState`.
pub fn spawn_pay_times_modal(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    state: Res<crate::game::PayTimesState>,
    existing: Query<Entity, With<PayTimesModal>>,
) {
    let want_open = state.pending.is_some();
    let is_open = !existing.is_empty();
    if want_open == is_open {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    let Some((spell_id, mechanic)) = state.pending else { return };
    let Some(cv) = &view.0 else { return };
    let name = cv
        .players
        .get(cv.your_seat)
        .and_then(|p| {
            p.hand.iter().find_map(|h| match h {
                crabomination::net::HandCardView::Known(k) if k.id == spell_id => {
                    Some(k.name.clone())
                }
                _ => None,
            })
        })
        .unwrap_or_default();
    let mechanic = mechanic.label();

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
            PayTimesModal,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    min_width: Val::Px(320.0),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(format!("Cast {name} paying its {mechanic} cost…")),
                    ui_fonts.tf(15.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|row| {
                        for (delta, label) in [(-1, "−"), (1, "+")] {
                            if delta < 0 {
                                row.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::BUTTON_INFO_BG),
                                    PayTimesStepButton { delta },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(theme::TEXT_PRIMARY),
                                        bevy::picking::Pickable::IGNORE,
                                    ));
                                });
                                row.spawn((
                                    Text::new(format!("×{}", state.times.max(1))),
                                    ui_fonts.tf(18.0),
                                    TextColor(theme::TEXT_PRIMARY),
                                    PayTimesReadout,
                                ));
                            } else {
                                row.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(theme::BUTTON_INFO_BG),
                                    PayTimesStepButton { delta },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(theme::TEXT_PRIMARY),
                                        bevy::picking::Pickable::IGNORE,
                                    ));
                                });
                            }
                        }
                    });
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_INFO_BG),
                        PayTimesConfirmButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Cast"),
                            ui_fonts.tf(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                            bevy::picking::Pickable::IGNORE,
                        ));
                    });
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_DANGER_BG),
                        PayTimesCancelButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Cancel"),
                            ui_fonts.tf(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                            bevy::picking::Pickable::IGNORE,
                        ));
                    });
            });
        });
}

/// Step / confirm / cancel handling for the pay-times stepper. Confirm
/// submits directly for untargeted spells; targeted ones arm the targeting
/// cursor with the pay-times rider so the click-submit routes through
/// the matching pay-N-times `CastSpell*` action.
#[allow(clippy::too_many_arguments)]
pub fn handle_pay_times_buttons(
    mut state: ResMut<crate::game::PayTimesState>,
    mut targeting: ResMut<TargetingState>,
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    step_q: Query<(&Interaction, &PayTimesStepButton), Changed<Interaction>>,
    confirm_q: Query<&Interaction, (Changed<Interaction>, With<PayTimesConfirmButton>)>,
    cancel_q: Query<&Interaction, (Changed<Interaction>, With<PayTimesCancelButton>)>,
    mut readout: Query<&mut Text, With<PayTimesReadout>>,
) {
    if cancel_q.iter().any(|i| *i == Interaction::Pressed) {
        state.pending = None;
        return;
    }
    for (interaction, btn) in &step_q {
        if *interaction == Interaction::Pressed {
            state.times = state.times.saturating_add_signed(btn.delta).max(1);
            for mut t in &mut readout {
                *t = Text::new(format!("×{}", state.times));
            }
        }
    }
    if confirm_q.iter().any(|i| *i == Interaction::Pressed)
        && let Some((spell_id, mechanic)) = state.pending
    {
        let times = state.times.max(1);
        let needs_target = view.0.as_ref().is_some_and(|cv| {
            cv.players.get(cv.your_seat).is_some_and(|p| {
                p.hand.iter().any(|h| matches!(h,
                    crabomination::net::HandCardView::Known(k)
                    if k.id == spell_id && k.needs_target))
            })
        });
        if needs_target {
            targeting.active = true;
            targeting.pending_card_id = Some(spell_id);
            targeting.back_face_pending = false;
            targeting.pending_pay_times = Some((times, mechanic));
        } else if let Some(outbox) = &outbox {
            outbox.submit(pay_times_cast_action(
                mechanic, spell_id, times, None, None,
            ));
        }
        state.pending = None;
    }
}

/// The `CastSpell*` action for a pay-N-times mechanic.
pub fn pay_times_cast_action(
    mechanic: crate::game::PayTimesMechanic,
    card_id: CardId,
    times: u32,
    target: Option<Target>,
    mode: Option<usize>,
) -> GameAction {
    use crate::game::PayTimesMechanic as M;
    match mechanic {
        M::Replicate => GameAction::CastSpellReplicate {
            card_id, times, target, additional_targets: vec![], mode, x_value: None,
        },
        M::Squad => GameAction::CastSpellSquad {
            card_id, times, target, additional_targets: vec![], mode, x_value: None,
        },
        M::Multikicker => GameAction::CastSpellMultikicked {
            card_id, times, target, additional_targets: vec![], mode, x_value: None,
        },
    }
}

// ── Reveal animation trigger ─────────────────────────────────────────────────

/// When RevealPopupState activates, start a RevealPeekAnimation on the top
/// deck card of the revealed player's pile.
#[allow(clippy::type_complexity)]
pub fn trigger_reveal_animation(
    mut reveal: ResMut<RevealPopupState>,
    deck_cards: Query<
        (Entity, &DeckPile, &Transform),
        (
            Without<crate::card::RevealPeekAnimation>,
            Without<crate::card::Animating>,
        ),
    >,
    mut commands: Commands,
) {
    let Some(player) = reveal.revealed_player.take() else {
        return;
    };

    // Top of the pile = highest index for that owner. face_up = 180° around
    // the card's local Y axis so the flip rotates along the long edge.
    let top = deck_cards
        .iter()
        .filter(|(_, dp, _)| dp.owner == player)
        .max_by_key(|(_, dp, _)| dp.index);
    if let Some((entity, _, transform)) = top {
        let face_up = transform.rotation * Quat::from_rotation_y(std::f32::consts::PI);
        commands
            .entity(entity)
            .insert(crate::card::Animating)
            .insert(crate::card::RevealPeekAnimation {
                progress: 0.0,
                speed: 0.6,
                start_rotation: transform.rotation,
                face_up_rotation: face_up,
                start_y: transform.translation.y,
            });
    }
}

// ── Split-card half picker (CR 709 / 702.102) ────────────────────────────────

/// Root entity of the split-card half-picker modal.
#[derive(Component)]
pub struct SplitCastModal;

/// A half-pick button: cast the right half or the fused whole.
#[derive(Component)]
pub struct SplitCastButton {
    pub spell: CardId,
    pub choice: crate::game::SplitCastChoice,
    pub needs_target: bool,
}

#[derive(Component)]
pub struct SplitCastCancelButton;

/// Spawn or despawn the split half-picker based on `SplitCastState`.
pub fn spawn_split_cast_modal(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<crate::theme::UiFonts>,
    state: Res<crate::game::SplitCastState>,
    existing: Query<Entity, With<SplitCastModal>>,
) {
    let want_open = state.pending.is_some();
    let is_open = !existing.is_empty();
    if !state.is_changed() && want_open == is_open {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    let Some(spell_id) = state.pending else { return };
    let Some(cv) = &view.0 else { return };
    let Some(known) = cv.players.get(cv.your_seat).and_then(|p| {
        p.hand.iter().find_map(|h| match h {
            crabomination::net::HandCardView::Known(k) if k.id == spell_id => Some(k.clone()),
            _ => None,
        })
    }) else {
        return;
    };

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            bevy::picking::Pickable::IGNORE,
            SplitCastModal,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    align_items: AlignItems::Center,
                    min_width: Val::Px(320.0),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new(format!("{} — cast which half?", known.name)),
                    ui_fonts.tf(15.0),
                    TextColor(theme::TEXT_PRIMARY),
                ));
                let button = |panel: &mut ChildSpawnerCommands,
                                  label: String,
                                  choice: crate::game::SplitCastChoice,
                                  needs_target: bool| {
                    panel
                        .spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                                ..default()
                            },
                            BackgroundColor(theme::BUTTON_INFO_BG),
                            SplitCastButton { spell: spell_id, choice, needs_target },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(label),
                                ui_fonts.tf(13.0),
                                TextColor(theme::TEXT_PRIMARY),
                                bevy::picking::Pickable::IGNORE,
                            ));
                        });
                };
                button(
                    panel,
                    format!("Cast right half ({})", known.split_right_cost_label),
                    crate::game::SplitCastChoice::Right,
                    known.split_right_needs_target,
                );
                // Fused with targets needs two target picks — beyond the
                // single-target cursor; offer Fused only when target-free.
                if known.split_fusable && !known.split_fused_needs_target {
                    button(
                        panel,
                        "Cast fused (both halves)".into(),
                        crate::game::SplitCastChoice::Fused,
                        false,
                    );
                }
                panel
                    .spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                            margin: UiRect::top(Val::Px(8.0)),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_DANGER_BG),
                        SplitCastCancelButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Cancel"),
                            ui_fonts.tf(13.0),
                            TextColor(theme::TEXT_PRIMARY),
                            bevy::picking::Pickable::IGNORE,
                        ));
                    });
            });
        });
}

/// Half-pick button → submit immediately (no target) or arm the targeting
/// cursor with `pending_split` so the eventual pick routes through
/// `CastSplitRight` / `CastSplitFused`.
pub fn handle_split_cast_buttons(
    mut state: ResMut<crate::game::SplitCastState>,
    mut targeting: ResMut<TargetingState>,
    outbox: Option<Res<NetOutbox>>,
    pick_q: Query<(&Interaction, &SplitCastButton), Changed<Interaction>>,
    cancel_q: Query<&Interaction, (Changed<Interaction>, With<SplitCastCancelButton>)>,
) {
    if cancel_q.iter().any(|i| *i == Interaction::Pressed) {
        state.pending = None;
        return;
    }
    for (interaction, btn) in &pick_q {
        if *interaction != Interaction::Pressed {
            continue;
        }
        state.pending = None;
        if btn.needs_target {
            targeting.active = true;
            targeting.pending_card_id = Some(btn.spell);
            targeting.pending_split = Some(btn.choice);
            return;
        }
        let Some(outbox) = &outbox else { return };
        let action = match btn.choice {
            crate::game::SplitCastChoice::Right => GameAction::CastSplitRight {
                card_id: btn.spell, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            crate::game::SplitCastChoice::Fused => GameAction::CastSplitFused {
                card_id: btn.spell, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
        };
        outbox.submit(action);
        return;
    }
}
