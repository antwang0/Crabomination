//! UI for pending player-choice decisions (scry, color choice, searches…).
//!
//! Reads the pending decision from `CurrentView` (server-projected) and
//! submits answers via `NetOutbox`. During the mulligan phase `CurrentView`
//! is empty and no modal is shown.

use bevy::prelude::*;

use crabomination::{
    card::CardId,
    decision::DecisionAnswer,
    game::{GameAction, Target},
    net::DecisionWire,
};

use crate::game::{GameLog, LegalTargets};
use crate::net_plugin::{cast_action_card_id, CurrentView, NetOutbox, PendingManaCast};
use crate::scryfall;
use crate::theme::{self, HoverTint, UiFonts};

#[derive(Component)]
pub struct DecisionModal;

#[derive(Component)]
pub struct ScryToggleButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct ScryReorderButton {
    pub card_id: CardId,
    pub delta: i32,
}

#[derive(Component)]
pub struct DecisionConfirmButton;

/// ← / → reorder button for the OrderTriggers modal (CR 603.3b). Moves the
/// trigger `delta` slots in the stack-push ordering.
#[derive(Component)]
pub struct TriggerReorderButton {
    pub source: CardId,
    pub delta: i32,
}

/// ← / → reorder button for the CombatDamageOrder modal (CR 510.1c). Moves
/// the blocker `delta` slots in the damage-assignment ordering.
#[derive(Component)]
pub struct DamageOrderReorderButton {
    pub blocker: CardId,
    pub delta: i32,
}

/// +/- stepper for the AssignCombatDamage modal (CR 510.1d). Adjusts the
/// damage assigned to one blocker.
#[derive(Component)]
pub struct DamageAssignButton {
    pub blocker: CardId,
    pub delta: i32,
}

#[derive(Component)]
pub struct MulliganKeepButton;

#[derive(Component)]
pub struct MulliganTakeButton;

#[derive(Component)]
pub struct SearchSelectButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct PutOnLibrarySelectButton {
    pub card_id: CardId,
}

#[derive(Component)]
pub struct DiscardSelectButton {
    pub card_id: CardId,
}

/// While a hand-card visual entity is selected for bottoming during the
/// PutOnLibrary phase, this component stores the two border child entities
/// drawn around it. Stored separately from `CardBorderHighlight` so the
/// hover-highlight system (which manages its own `CardBorderHighlight`)
/// doesn't conflict.
#[derive(Component)]
pub struct PutOnLibraryHighlight {
    back: Entity,
    front: Entity,
}

/// Local UI state tracked during an in-flight decision. Cleared when the
/// server's `pending_decision` goes back to `None`.
#[derive(Resource, Default)]
pub struct DecisionUiState {
    /// For Scry: per-card "send to bottom" flags (false = keep on top).
    pub scry: Vec<(CardId, bool)>,
    /// For SearchLibrary: the card the player selected (None = failed search).
    pub search_selected: Option<CardId>,
    /// For PutOnLibrary / Mulligan bottoming: ordered list of selected card IDs.
    pub put_on_library: Vec<CardId>,
    /// For Discard (Inquisition / Thoughtseize picker): selected card IDs.
    pub discard_selected: Vec<CardId>,
    /// For OrderTriggers (CR 603.3b): the working stack-push order of the
    /// controller's simultaneous triggers (index 0 pushed first).
    pub trigger_order: Vec<CardId>,
    /// For CombatDamageOrder (CR 510.1c): the working blocker damage order
    /// (index 0 takes damage first).
    pub damage_order: Vec<CardId>,
    /// For AssignCombatDamage (CR 510.1d): the working per-blocker damage
    /// split.
    pub damage_assign: Vec<(CardId, u32)>,
    /// For ChooseModes (CR 700.2d "choose N" / Escalate): the working
    /// selected-mode set, in pick order. `None` = not yet seeded from the
    /// wire default (distinct from "player deselected everything").
    pub modes_selected: Option<Vec<u8>>,
    /// For ChooseAmount: the working value (kept within `0..=max`).
    pub amount: u32,
    /// For DivideDamage (CR 601.2d): working per-target amounts, parallel
    /// to the wire's target list.
    pub divide: Vec<u32>,
    /// CardId the modal was last spawned for — avoids respawning each frame.
    pub spawned_for: Option<DecisionKey>,
}

/// Fingerprint of a pending decision. Used to detect when a new decision
/// arrived (so the modal respawns) vs. the same one still showing.
#[derive(Clone, PartialEq, Eq)]
pub enum DecisionKey {
    Scry(Vec<CardId>),
    Search(Vec<CardId>),
    PutOnLibrary(Vec<CardId>),
    Discard(Vec<CardId>, u32),
    Mulligan(Vec<CardId>, usize),
    ChooseColor(CardId),
    /// `Decision::ChooseTarget` — keyed by the legal-target list so a
    /// re-spawned decision with a different legal set is treated as
    /// new (the old highlight-set needs to clear).
    ChooseTarget(CardId, Vec<Target>),
    /// `Decision::Learn` — keyed by the offered Lesson ids.
    Learn(Vec<CardId>),
    /// `Decision::OrderTriggers` — keyed by the trigger source ids.
    OrderTriggers(Vec<CardId>),
    /// `Decision::CombatDamageOrder` (CR 510.1c) — keyed by attacker + blockers.
    CombatDamageOrder(CardId, Vec<CardId>),
    /// `Decision::AssignCombatDamage` (CR 510.1d) — keyed by attacker + blockers.
    AssignCombatDamage(CardId, Vec<CardId>),
    /// `Decision::ChooseCards` — keyed by the candidate ids + (min, max) so a
    /// re-posed pick (e.g. a chained second cost) re-spawns the modal.
    ChooseCards(Vec<CardId>, u32, u32),
    /// `Decision::OptionalTrigger` — a yes/no prompt (e.g. the float-spend
    /// confirmation), keyed by the source + description.
    OptionalTrigger(CardId, String),
    /// `Decision::NameCard` (CR 201.3) — keyed by the asking source.
    NameCard(CardId),
    /// `Decision::ChooseModes` (CR 700.2d) — keyed by source + shape.
    ChooseModes(CardId, usize, usize),
    /// Resolution-time `Decision::ChooseMode` for a modal trigger (Riot,
    /// Fabricate) — keyed by source + mode count.
    ChooseModeTrigger(CardId, usize),
    /// `Decision::ChooseAmount` — keyed by source + bound + prompt.
    ChooseAmount(CardId, u32, String),
    /// `Decision::DivideDamage` (CR 601.2d) — keyed by source + total +
    /// target list.
    DivideDamage(CardId, u32, Vec<Target>),
    /// `Decision::ChooseCreatureType` — keyed by the asking source.
    ChooseCreatureType(CardId),
}

fn decision_key(decision: &DecisionWire) -> Option<DecisionKey> {
    match decision {
        DecisionWire::Scry { cards, .. } => Some(DecisionKey::Scry(
            cards.iter().map(|(id, _)| *id).collect(),
        )),
        DecisionWire::SearchLibrary { candidates, .. } => Some(DecisionKey::Search(
            candidates.iter().map(|(id, _)| *id).collect(),
        )),
        DecisionWire::PutOnLibrary { hand, .. } => Some(DecisionKey::PutOnLibrary(
            hand.iter().map(|(id, _)| *id).collect(),
        )),
        DecisionWire::Discard { hand, count, .. } => Some(DecisionKey::Discard(
            hand.iter().map(|(id, _)| *id).collect(),
            *count,
        )),
        DecisionWire::Mulligan { hand, mulligans_taken, .. } => Some(DecisionKey::Mulligan(
            hand.iter().map(|(id, _)| *id).collect(),
            *mulligans_taken,
        )),
        DecisionWire::ChooseColor { source, .. } => Some(DecisionKey::ChooseColor(*source)),
        DecisionWire::Learn { lessons, .. } => {
            Some(DecisionKey::Learn(lessons.iter().map(|(id, _)| *id).collect()))
        }
        DecisionWire::ChooseTarget { source, legal, .. } => {
            Some(DecisionKey::ChooseTarget(*source, legal.clone()))
        }
        DecisionWire::OrderTriggers { triggers, .. } => {
            Some(DecisionKey::OrderTriggers(triggers.iter().map(|(id, _)| *id).collect()))
        }
        DecisionWire::CombatDamageOrder { attacker, blockers } => {
            Some(DecisionKey::CombatDamageOrder(
                *attacker,
                blockers.iter().map(|(id, _)| *id).collect(),
            ))
        }
        DecisionWire::AssignCombatDamage { attacker, blockers, .. } => {
            Some(DecisionKey::AssignCombatDamage(
                *attacker,
                blockers.iter().map(|(id, _, _)| *id).collect(),
            ))
        }
        DecisionWire::ChooseCards { candidates, min, max, .. } => Some(DecisionKey::ChooseCards(
            candidates.iter().map(|(id, _)| *id).collect(),
            *min,
            *max,
        )),
        DecisionWire::OptionalTrigger { source, description } => {
            Some(DecisionKey::OptionalTrigger(*source, description.clone()))
        }
        DecisionWire::NameCard { source, .. } => Some(DecisionKey::NameCard(*source)),
        DecisionWire::ChooseModes { source, num_modes, count, .. } => {
            Some(DecisionKey::ChooseModes(*source, *num_modes, *count))
        }
        DecisionWire::ChooseMode { source, num_modes, .. } => {
            Some(DecisionKey::ChooseModeTrigger(*source, *num_modes))
        }
        DecisionWire::ChooseAmount { source, max, prompt } => {
            Some(DecisionKey::ChooseAmount(*source, *max, prompt.clone()))
        }
        DecisionWire::DivideDamage { source, total, targets } => {
            Some(DecisionKey::DivideDamage(*source, *total, targets.clone()))
        }
        DecisionWire::ChooseCreatureType { source, .. } => {
            Some(DecisionKey::ChooseCreatureType(*source))
        }
        _ => None,
    }
}

const CARD_ASPECT_RATIO: f32 = 88.0 / 63.0;
const CARD_W: f32 = 180.0;
const CARD_H: f32 = CARD_W * CARD_ASPECT_RATIO;

// Aliases that keep the existing call sites short and locally
// self-documenting. The actual values live in `theme.rs` so the modal
// look stays in sync with the rest of the chrome.
use theme::PANEL_TILE_BG as MODAL_TILE_BG;
use theme::BUTTON_TERTIARY_BG as REORDER_BG;
use theme::BUTTON_TERTIARY_BG_DISABLED as REORDER_BG_DISABLED;

/// Spawn or despawn the decision modal based on the server view. Only shows
/// for decisions owned by P0 (your_seat).
pub fn spawn_decision_ui(
    mut commands: Commands,
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut targeting: ResMut<crate::game::TargetingState>,
    mut legal_targets: ResMut<LegalTargets>,
    existing: Query<Entity, With<DecisionModal>>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    pending_mana_cast: Res<PendingManaCast>,
    outbox: Option<Res<NetOutbox>>,
) {
    let Some(cv) = &view.0 else {
        // Mulligan / no view yet — tear down any existing modal.
        for e in &existing {
            commands.entity(e).despawn();
        }
        state.scry.clear();
        state.search_selected = None;
        state.put_on_library.clear();
        state.discard_selected.clear();
        state.modes_selected = None;
        state.amount = 0;
        state.divide.clear();
        state.spawned_for = None;
        // Clear any decision-driven targeting flag so the cursor
        // doesn't stay armed across a state change.
        if targeting.pending_decision_target {
            targeting.active = false;
            targeting.pending_decision_target = false;
        }
        legal_targets.permanents.clear();
        legal_targets.players.clear();
        legal_targets.enumerated = false;
        legal_targets.source_name.clear();
        legal_targets.description.clear();
        return;
    };

    let pending = match &cv.pending_decision {
        Some(pd) if pd.acting_player == cv.your_seat => pd,
        _ => {
            for e in &existing {
                commands.entity(e).despawn();
            }
            if state.spawned_for.is_some() {
                state.scry.clear();
                state.search_selected = None;
                state.put_on_library.clear();
                state.discard_selected.clear();
                state.modes_selected = None;
                state.amount = 0;
                state.divide.clear();
                state.spawned_for = None;
            }
            if targeting.pending_decision_target {
                targeting.active = false;
                targeting.pending_decision_target = false;
            }
            legal_targets.permanents.clear();
            legal_targets.players.clear();
            legal_targets.source_name.clear();
            legal_targets.description.clear();
            return;
        }
    };

    let wire = match &pending.decision {
        Some(d) => d,
        None => return,
    };

    let key = match decision_key(wire) {
        Some(k) => k,
        None => return,
    };

    if state.spawned_for.as_ref() == Some(&key) {
        return;
    }

    for e in &existing {
        commands.entity(e).despawn();
    }

    match wire {
        DecisionWire::Scry { cards, mode, .. } => {
            if state.scry.is_empty() {
                state.scry = cards.iter().map(|(id, _)| (*id, false)).collect();
            }
            state.spawned_for = Some(key);
            let name_map: std::collections::HashMap<CardId, &str> =
                cards.iter().map(|(id, n)| (*id, n.as_str())).collect();
            let ordered: Vec<(CardId, String, bool)> = state
                .scry
                .iter()
                .map(|(id, bottom)| (*id, name_map[id].to_string(), *bottom))
                .collect();
            spawn_scry_modal(&mut commands, &asset_server, &ui_fonts, &ordered, *mode);
        }
        DecisionWire::SearchLibrary { candidates, eligible, .. } => {
            state.search_selected = None;
            state.spawned_for = Some(key);
            spawn_search_modal(&mut commands, &asset_server, &ui_fonts, candidates, eligible);
        }
        DecisionWire::PutOnLibrary { count, hand, .. } => {
            state.put_on_library.clear();
            state.spawned_for = Some(key);
            spawn_put_on_library_modal(&mut commands, &asset_server, &ui_fonts, hand, *count);
        }
        DecisionWire::Mulligan { hand, mulligans_taken, serum_powders, .. } => {
            state.spawned_for = Some(key);
            spawn_mulligan_modal(
                &mut commands,
                &asset_server,
                &ui_fonts,
                hand,
                *mulligans_taken,
                serum_powders,
            );
        }
        DecisionWire::ChooseColor { legal, .. } => {
            state.spawned_for = Some(key);
            spawn_choose_color_modal(&mut commands, &ui_fonts, legal);
        }
        DecisionWire::Learn { lessons, hand, .. } => {
            state.spawned_for = Some(key);
            spawn_learn_modal(&mut commands, &ui_fonts, lessons, hand);
        }
        DecisionWire::Discard { count, hand, .. } => {
            state.discard_selected.clear();
            state.spawned_for = Some(key);
            let title = format!("Choose {count} card(s) to discard");
            spawn_card_picker_modal(&mut commands, &asset_server, &ui_fonts, &title, hand);
        }
        DecisionWire::ChooseCards { prompt, candidates, .. } => {
            state.discard_selected.clear();
            state.spawned_for = Some(key);
            spawn_card_picker_modal(&mut commands, &asset_server, &ui_fonts, prompt, candidates);
        }
        DecisionWire::OptionalTrigger { source, description } => {
            state.spawned_for = Some(key);
            // The CR 601.2g "spend leftover floating mana, or keep it and tap
            // lands?" confirmation fires for the very card the player is
            // actively assembling mana to pay for via the manual-tap flow —
            // its `source` is that spell/ability. They tapped those sources
            // *to* pay this cost, so being asked whether to spend the mana
            // they just tapped is pure noise. Auto-answer "spend" and skip the
            // modal in that case; show it normally otherwise.
            let paying_for_this = pending_mana_cast
                .0
                .as_ref()
                .is_some_and(|pc| cast_action_card_id(&pc.action) == *source);
            if paying_for_this {
                if let Some(outbox) = &outbox {
                    outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Bool(true)));
                }
            } else {
                spawn_optional_modal(&mut commands, &ui_fonts, description);
            }
        }
        DecisionWire::NameCard { source_name, suggestions, .. } => {
            state.spawned_for = Some(key);
            spawn_name_card_modal(&mut commands, &ui_fonts, source_name, suggestions);
        }
        DecisionWire::OrderTriggers { triggers, .. } => {
            if state.trigger_order.is_empty() {
                state.trigger_order = triggers.iter().map(|(id, _)| *id).collect();
            }
            state.spawned_for = Some(key);
            let name_map: std::collections::HashMap<CardId, &str> =
                triggers.iter().map(|(id, n)| (*id, n.as_str())).collect();
            let ordered: Vec<(CardId, String)> = state
                .trigger_order
                .iter()
                .map(|id| (*id, name_map.get(id).copied().unwrap_or("Triggered ability").to_string()))
                .collect();
            spawn_order_triggers_modal(&mut commands, &asset_server, &ui_fonts, &ordered);
        }
        DecisionWire::CombatDamageOrder { attacker, blockers } => {
            if state.damage_order.is_empty() {
                state.damage_order = blockers.iter().map(|(id, _)| *id).collect();
            }
            state.spawned_for = Some(key);
            let name_of = |id: CardId| -> String {
                blockers
                    .iter()
                    .find(|(b, _)| *b == id)
                    .map(|(_, n)| n.clone())
                    .unwrap_or_else(|| "Creature".to_string())
            };
            let attacker_name = cv
                .battlefield
                .iter()
                .find(|p| p.id == *attacker)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Attacker".to_string());
            let ordered: Vec<(CardId, String)> =
                state.damage_order.iter().map(|id| (*id, name_of(*id))).collect();
            spawn_damage_order_modal(&mut commands, &asset_server, &ui_fonts, &attacker_name, &ordered);
        }
        DecisionWire::AssignCombatDamage { attacker, attacker_power, blockers } => {
            if state.damage_assign.is_empty() {
                // Seed with the default lethal-in-order split so the modal
                // opens on the engine's fallback.
                let mut left = *attacker_power;
                state.damage_assign = blockers
                    .iter()
                    .map(|(id, _, lethal)| {
                        let give = (*lethal).min(left);
                        left -= give;
                        (*id, give)
                    })
                    .collect();
                if let Some(last) = state.damage_assign.last_mut() {
                    last.1 += left;
                }
            }
            state.spawned_for = Some(key);
            let attacker_name = cv
                .battlefield
                .iter()
                .find(|p| p.id == *attacker)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "Attacker".to_string());
            spawn_damage_assign_modal(
                &mut commands,
                &asset_server,
                &ui_fonts,
                &attacker_name,
                *attacker_power,
                blockers,
                &state.damage_assign,
            );
        }
        DecisionWire::ChooseModes { source, count, num_modes, default, mode_texts, .. } => {
            if state.modes_selected.is_none() {
                state.modes_selected = Some(default.clone());
            }
            state.spawned_for = Some(key);
            let selected = state.modes_selected.clone().unwrap_or_default();
            let title = view_card_name(cv, *source, "Modal effect");
            spawn_choose_modes_modal(
                &mut commands,
                &ui_fonts,
                &title,
                *num_modes,
                *count,
                mode_texts,
                &selected,
            );
        }
        DecisionWire::ChooseMode { source, num_modes, mode_texts } => {
            state.spawned_for = Some(key);
            let title = view_card_name(cv, *source, "Triggered ability");
            spawn_choose_trigger_mode_modal(&mut commands, &ui_fonts, &title, *num_modes, mode_texts);
        }
        DecisionWire::ChooseAmount { prompt, max, .. } => {
            state.amount = 0;
            state.spawned_for = Some(key);
            spawn_choose_amount_modal(&mut commands, &ui_fonts, prompt, *max, state.amount);
        }
        DecisionWire::DivideDamage { source, total, targets } => {
            if state.divide.len() != targets.len() {
                state.divide = crabomination::decision::even_damage_split(*total, targets.len());
            }
            state.spawned_for = Some(key);
            let title = view_card_name(cv, *source, "Divided damage");
            let rows: Vec<String> = targets
                .iter()
                .map(|t| match t {
                    Target::Permanent(id) => view_card_name(cv, *id, "Permanent"),
                    Target::Player(s) => cv
                        .players
                        .iter()
                        .find(|p| p.seat == *s)
                        .map(|p| p.name.clone())
                        .unwrap_or_else(|| format!("Player {s}")),
                })
                .collect();
            spawn_divide_damage_modal(&mut commands, &ui_fonts, &title, *total, &rows, &state.divide);
        }
        DecisionWire::ChooseCreatureType { suggestions, .. } => {
            state.spawned_for = Some(key);
            spawn_creature_type_modal(&mut commands, &ui_fonts, suggestions);
        }
        DecisionWire::ChooseTarget { legal, source_name, description, .. } => {
            // No modal — reuse the existing in-scene targeting cursor.
            // Flipping `pending_decision_target` flags `handle_game_input`
            // to submit picks as `DecisionAnswer::Target` instead of
            // wrapping them in `CastSpell` / `ActivateAbility`.
            state.spawned_for = Some(key);
            targeting.active = true;
            targeting.pending_card_id = None;
            targeting.pending_ability_source = None;
            targeting.pending_ability_index = None;
            targeting.back_face_pending = false;
            targeting.pending_decision_target = true;
            legal_targets.permanents.clear();
            legal_targets.players.clear();
            // The server handed us the authoritative legal list.
            legal_targets.enumerated = true;
            for t in legal {
                match t {
                    Target::Permanent(id) => {
                        legal_targets.permanents.insert(*id);
                    }
                    Target::Player(s) => {
                        legal_targets.players.insert(*s);
                    }
                }
            }
            legal_targets.source_name = source_name.clone();
            legal_targets.description = description.clone();
        }
        _ => {}
    }
}

fn spawn_scry_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    ordered: &[(CardId, String, bool)],
    mode: crabomination::decision::ScryMode,
) {
    use crabomination::decision::ScryMode;
    // The label and prompt the second bucket gets depends on the mode: Scry
    // bottoms, Surveil mills, Rearrange keeps everything on top (no toggle).
    let (verb, second_bucket): (&str, Option<&str>) = match mode {
        ScryMode::Scry => ("Scry", Some("Bottom")),
        ScryMode::Surveil => ("Surveil", Some("Graveyard")),
        ScryMode::Rearrange => ("Rearrange top", None),
    };
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(16.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let n = ordered.len();
    commands.entity(panel).with_children(|panel| {
        let prompt = match second_bucket {
            Some(bucket) => format!(
                "{verb} {n}: click card to toggle {bucket}  ·  ← → to reorder  ·  left = top of library"
            ),
            None => format!("{verb} {n}: ← → to reorder  ·  left = top of library"),
        };
        panel.spawn((
            Text::new(prompt),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (i, (card_id, name, is_bottom)) in ordered.iter().enumerate() {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    let at_left = i == 0;
                    let at_right = i == n - 1;

                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        // Rearrange has no second bucket, so the tile isn't a
                        // toggle — it only shows the card and its top-position.
                        let tile_label = match second_bucket {
                            Some(bucket) if *is_bottom => bucket,
                            _ => "Top",
                        };
                        let mut tile = col.spawn((
                            Button,
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(CARD_W),
                                padding: UiRect::all(Val::Px(6.0)),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(if *is_bottom && second_bucket.is_some() {
                                theme::BUTTON_SELECTED_BG
                            } else {
                                MODAL_TILE_BG
                            }),
                        ));
                        if second_bucket.is_some() {
                            tile.insert(ScryToggleButton { card_id: *card_id });
                        }
                        tile.with_children(|cb| {
                            cb.spawn((
                                ImageNode { image: texture, ..default() },
                                Node {
                                    width: Val::Px(CARD_W - 12.0),
                                    height: Val::Px(CARD_H - 12.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cb.spawn((
                                Text::new(tile_label),
                                ui_fonts.tf(14.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });

                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for (label, delta, disabled) in
                                [("←", -1i32, at_left), ("→", 1, at_right)]
                            {
                                r.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if disabled {
                                        REORDER_BG_DISABLED
                                    } else {
                                        REORDER_BG
                                    }),
                                    ScryReorderButton {
                                        card_id: *card_id,
                                        delta,
                                    },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(if disabled {
                                            theme::TEXT_MUTED
                                        } else {
                                            theme::TEXT_PRIMARY
                                        }),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// CR 603.3b — modal letting the viewer order their own simultaneous
/// triggers. Renders each trigger as a card tile with ← / → reorder
/// buttons; index 0 (leftmost) is pushed onto the stack first, so the
/// rightmost trigger resolves first (LIFO). Confirm submits the order.
fn spawn_order_triggers_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    ordered: &[(CardId, String)],
) {
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(20.0)),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let n = ordered.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(
                "Order your triggers  ·  ← → to reorder  ·  rightmost resolves first",
            ),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (i, (source, name)) in ordered.iter().enumerate() {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    let at_left = i == 0;
                    let at_right = i == n - 1;

                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(CARD_W),
                                padding: UiRect::all(Val::Px(6.0)),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(MODAL_TILE_BG),
                        ))
                        .with_children(|cb| {
                            cb.spawn((
                                ImageNode { image: texture, ..default() },
                                Node {
                                    width: Val::Px(CARD_W - 12.0),
                                    height: Val::Px(CARD_H - 12.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cb.spawn((
                                Text::new(format!("{}.", i + 1)),
                                ui_fonts.tf(14.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });

                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for (label, delta, disabled) in
                                [("←", -1i32, at_left), ("→", 1, at_right)]
                            {
                                r.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if disabled {
                                        REORDER_BG_DISABLED
                                    } else {
                                        REORDER_BG
                                    }),
                                    TriggerReorderButton { source: *source, delta },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(if disabled {
                                            theme::TEXT_MUTED
                                        } else {
                                            theme::TEXT_PRIMARY
                                        }),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// CR 510.1c — modal letting the attacking player order blockers for damage
/// assignment. Same layout as the trigger-order modal: cards in a row,
/// ← / → to reorder, leftmost is damaged first.
fn spawn_damage_order_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    attacker_name: &str,
    ordered: &[(CardId, String)],
) {
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(20.0)),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let n = ordered.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "{attacker_name}: order blockers for damage  ·  ← → to reorder  ·  leftmost damaged first",
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (i, (blocker, name)) in ordered.iter().enumerate() {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    let at_left = i == 0;
                    let at_right = i == n - 1;

                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(CARD_W),
                                padding: UiRect::all(Val::Px(6.0)),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(MODAL_TILE_BG),
                        ))
                        .with_children(|cb| {
                            cb.spawn((
                                ImageNode { image: texture, ..default() },
                                Node {
                                    width: Val::Px(CARD_W - 12.0),
                                    height: Val::Px(CARD_H - 12.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cb.spawn((
                                Text::new(format!("{}.", i + 1)),
                                ui_fonts.tf(14.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });

                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for (label, delta, disabled) in
                                [("←", -1i32, at_left), ("→", 1, at_right)]
                            {
                                r.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(if disabled {
                                        REORDER_BG_DISABLED
                                    } else {
                                        REORDER_BG
                                    }),
                                    DamageOrderReorderButton { blocker: *blocker, delta },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(if disabled {
                                            theme::TEXT_MUTED
                                        } else {
                                            theme::TEXT_PRIMARY
                                        }),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// CR 510.1d — modal letting the attacking player split combat damage among
/// blockers with per-blocker +/- steppers. The engine validates the split and
/// falls back to lethal-in-order if it breaks the ordering rule.
#[allow(clippy::too_many_arguments)]
fn spawn_damage_assign_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    attacker_name: &str,
    attacker_power: u32,
    blockers: &[(CardId, String, u32)],
    assign: &[(CardId, u32)],
) {
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(16.0),
                padding: UiRect::all(Val::Px(20.0)),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let assigned: u32 = assign.iter().map(|(_, a)| *a).sum();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "{attacker_name}: assign combat damage  ·  {assigned} / {attacker_power} assigned",
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(12.0),
                ..default()
            })
            .with_children(|row| {
                for (blocker, name, lethal) in blockers {
                    let amount = assign
                        .iter()
                        .find(|(id, _)| id == blocker)
                        .map(|(_, a)| *a)
                        .unwrap_or(0);
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);

                    row.spawn(Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|col| {
                        col.spawn((
                            Node {
                                flex_direction: FlexDirection::Column,
                                width: Val::Px(CARD_W),
                                padding: UiRect::all(Val::Px(6.0)),
                                row_gap: Val::Px(4.0),
                                align_items: AlignItems::Center,
                                ..default()
                            },
                            BackgroundColor(MODAL_TILE_BG),
                        ))
                        .with_children(|cb| {
                            cb.spawn((
                                ImageNode { image: texture, ..default() },
                                Node {
                                    width: Val::Px(CARD_W - 12.0),
                                    height: Val::Px(CARD_H - 12.0),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ));
                            cb.spawn((
                                Text::new(format!("{amount} dmg  (lethal {lethal})")),
                                ui_fonts.tf(14.0),
                                TextColor(if amount >= *lethal {
                                    theme::TEXT_GOOD
                                } else {
                                    theme::TEXT_PRIMARY
                                }),
                                Pickable::IGNORE,
                            ));
                        });

                        col.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            ..default()
                        })
                        .with_children(|r| {
                            for (label, delta) in [("−", -1i32), ("+", 1)] {
                                r.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)),
                                        ..default()
                                    },
                                    BackgroundColor(REORDER_BG),
                                    DamageAssignButton { blocker: *blocker, delta },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new(label),
                                        ui_fonts.tf(16.0),
                                        TextColor(theme::TEXT_PRIMARY),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

fn spawn_search_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    candidates: &[(CardId, String)],
    eligible: &Option<Vec<CardId>>,
) {
    // Search candidates are typically the entire library (60 cards). The
    // generic CARD_W of 180px would overflow the viewport vertically; use
    // a compact size for this dialog and scroll the grid when it spills.
    const SEARCH_CARD_W: f32 = 110.0;
    let search_card_h = SEARCH_CARD_W * CARD_ASPECT_RATIO;

    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(85.0),
                max_height: Val::Percent(90.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    let count = candidates.len();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "Search your library — click a card to select it ({count} card{s})",
                s = if count == 1 { "" } else { "s" }
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        // Scrollable grid: bounded height + Overflow::scroll_y so the
        // mouse wheel pages through long candidate lists. Without the
        // bound the panel sizes itself to the children and overflows the
        // window.
        panel
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(8.0),
                    row_gap: Val::Px(8.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::FlexStart,
                    max_height: Val::Vh(70.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                Pickable::default(),
            ))
            .with_children(|row| {
                for (card_id, name) in candidates {
                    // Revealed-but-not-pickable cards (Impulse non-matches,
                    // duplicate names) render greyed and ignore clicks.
                    let pickable =
                        eligible.as_ref().is_none_or(|ok| ok.contains(card_id));
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    let mut tile = row.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(SEARCH_CARD_W),
                            padding: UiRect::all(Val::Px(4.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(MODAL_TILE_BG),
                    ));
                    if pickable {
                        tile.insert((Button, SearchSelectButton { card_id: *card_id }));
                    } else {
                        tile.insert(Pickable::IGNORE);
                    }
                    tile.with_children(|cb| {
                        cb.spawn((
                            ImageNode {
                                image: texture,
                                color: if pickable {
                                    Color::WHITE
                                } else {
                                    Color::srgba(0.45, 0.45, 0.45, 0.8)
                                },
                                ..default()
                            },
                            Node {
                                width: Val::Px(SEARCH_CARD_W - 8.0),
                                height: Val::Px(search_card_h - 8.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        cb.spawn((
                            Text::new(name.clone()),
                            ui_fonts.tf(10.0),
                            TextColor(if pickable {
                                theme::TEXT_PRIMARY
                            } else {
                                theme::TEXT_MUTED
                            }),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Discard picker. Used in two cases that share the same wire format:
///
/// - **Self-discard** (Charging Strifeknight, Faithless Looting, Frantic
///   Search, etc.) — `Effect::Discard { random: false }`. The modal shows
///   the player's own hand.
/// - **Inquisition / Thoughtseize** — `Effect::DiscardChosen`. The caster
///   sees the target opponent's hand and picks for them.
///
/// Generic "pick cards from a list + Confirm" modal. Shared by the discard
/// decision and the `ChooseCards` decision (forced sacrifice / graveyard
/// exile-as-cost). `title` is the prompt shown above the grid; selection state
/// lives in `DecisionUiState::discard_selected` and the per-tile
/// `DiscardSelectButton` (reused as the generic multi-card-pick buffer), with
/// the min/max enforced by `handle_discard_select` / `handle_confirm` off the
/// live decision.
fn spawn_card_picker_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    title: &str,
    candidates: &[(CardId, String)],
) {
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(16.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(90.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(title.to_string()),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        panel
            .spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: Val::Px(12.0),
                row_gap: Val::Px(12.0),
                justify_content: JustifyContent::Center,
                ..default()
            })
            .with_children(|row| {
                for (card_id, name) in candidates {
                    let path = scryfall::card_asset_path(name);
                    let texture: Handle<Image> = asset_server.load(&path);
                    row.spawn((
                        Button,
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(CARD_W),
                            padding: UiRect::all(Val::Px(6.0)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(MODAL_TILE_BG),
                        DiscardSelectButton { card_id: *card_id },
                    ))
                    .with_children(|cb| {
                        cb.spawn((
                            ImageNode { image: texture, ..default() },
                            Node {
                                width: Val::Px(CARD_W - 12.0),
                                height: Val::Px(CARD_H - 12.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        cb.spawn((
                            Text::new(name.clone()),
                            ui_fonts.tf(12.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });
        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(18.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Compact PutOnLibrary banner. Card selection happens by clicking the 3D
/// hand cards directly; this banner shows the prompt, the running count of
/// selected cards, and a Confirm button.
fn spawn_put_on_library_modal(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    _hand: &[(CardId, String)],
    count: usize,
) {
    let root = commands
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
            // Pass-through transparent overlay so the 3D hand cards remain
            // clickable through the unfilled regions of the root.
            bevy::picking::Pickable::IGNORE,
            DecisionModal,
        ))
        .id();

    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(16.0)),
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();

    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "Click {count} card{} from your hand to put on the bottom of your library."
                ,
                if count == 1 { "" } else { "s" }
            )),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        panel.spawn((
            Text::new(format!("0 / {count} selected")),
            ui_fonts.tf(14.0),
            TextColor(theme::ACCENT_GOLD),
            PutOnLibraryCountText,
        ));

        panel
            .spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                DecisionConfirmButton,
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("Confirm"),
                    ui_fonts.tf(16.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
    });
}

/// Marker for the live "X / N selected" text inside the PutOnLibrary banner.
#[derive(Component)]
pub struct PutOnLibraryCountText;

/// One Serum-Powder-style helper button. Carries the powder card's ID so
/// the click handler can submit `DecisionAnswer::SerumPowder(id)`.
#[derive(Component, Debug, Clone, Copy)]
pub struct MulliganSerumPowderButton(pub CardId);

/// Compact mulligan banner. The hand itself is rendered in 3D on the table —
/// this banner just shows the prompt and the Keep / Mulligan / Serum Powder
/// buttons. `serum_powders` is one CardId per Serum-Powder-style helper
/// currently in hand; renders one button each.
fn spawn_mulligan_modal(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    _hand: &[(CardId, String)],
    mulligans_taken: usize,
    serum_powders: &[CardId],
) {
    let root = commands.spawn((
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
        // Transparent overlay — the 3D hand stays visible AND clickable
        // through the empty regions of the root. Pickable::IGNORE makes the
        // root pass-through; the panel below has BackgroundColor and so
        // re-acquires picking just where its rect is.
        bevy::picking::Pickable::IGNORE,
        DecisionModal,
    )).id();

    let panel = commands.spawn((
        Node {
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            row_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(theme::PANEL_BG),
    )).id();
    commands.entity(root).add_child(panel);

    let title = if mulligans_taken == 0 {
        "Keep this opening hand?".to_string()
    } else {
        format!("Mulligan {mulligans_taken} — keep this hand?")
    };

    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title), ui_fonts.tf(18.0), TextColor(theme::TEXT_PRIMARY)));
        p.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(16.0), ..default() })
        .with_children(|btns| {
            btns.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                MulliganKeepButton,
            ))
            .with_children(|b| { b.spawn((Text::new("Keep (K)"), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY))); });

            btns.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_DANGER_BG),
                HoverTint::new(theme::BUTTON_DANGER_BG),
                MulliganTakeButton,
            ))
            .with_children(|b| { b.spawn((Text::new("Mulligan (M)"), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY))); });

            // One button per Serum-Powder-style helper currently in hand.
            // Clicking submits DecisionAnswer::SerumPowder(id) — exiles the
            // hand and deals a fresh seven without bumping the mulligan
            // ladder. Multiple powders in hand stack as separate buttons.
            for (idx, powder_id) in serum_powders.iter().enumerate() {
                let label = if serum_powders.len() == 1 {
                    "Serum Powder".to_string()
                } else {
                    format!("Serum Powder #{}", idx + 1)
                };
                btns.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                        ..default()
                    },
                    BackgroundColor(REORDER_BG),
                    MulliganSerumPowderButton(*powder_id),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_PRIMARY),
                    ));
                });
            }
        });
    });
}

/// Handle clicks on put-on-library candidate cards: add/remove from ordered selection.
#[allow(clippy::type_complexity)]
pub fn handle_put_on_library_select(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<
        (&Interaction, &PutOnLibrarySelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let Some(cv) = &view.0 else { return };
    let required_count = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
        Some(DecisionWire::PutOnLibrary { count, .. }) => *count,
        _ => return,
    };

    for (interaction, btn, mut bg) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed { continue; }
        let id = btn.card_id;
        if let Some(pos) = state.put_on_library.iter().position(|&x| x == id) {
            state.put_on_library.remove(pos);
            *bg = BackgroundColor(MODAL_TILE_BG);
        } else if state.put_on_library.len() < required_count {
            state.put_on_library.push(id);
            *bg = BackgroundColor(theme::BUTTON_SELECTED_BG);
        }
    }
}

/// Handle clicks on Inquisition/Thoughtseize discard candidate cards.
/// Toggles inclusion in the selection up to `count` cards (taken from
/// the live `DecisionWire::Discard` count). Selected cards highlight in
/// `theme::BUTTON_SELECTED_BG`; clicking a selected card unselects it.
#[allow(clippy::type_complexity)]
pub fn handle_discard_select(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<
        (&Interaction, &DiscardSelectButton, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let required_count = match view
        .0
        .as_ref()
        .and_then(|v| v.pending_decision.as_ref())
        .and_then(|p| p.decision.as_ref())
    {
        Some(DecisionWire::Discard { count, .. }) => *count as usize,
        // ChooseCards reuses the same multi-select grid; `max` is the cap.
        Some(DecisionWire::ChooseCards { max, .. }) => *max as usize,
        _ => return,
    };
    for (interaction, btn, mut bg) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let id = btn.card_id;
        if let Some(pos) = state.discard_selected.iter().position(|&x| x == id) {
            state.discard_selected.remove(pos);
            *bg = BackgroundColor(MODAL_TILE_BG);
        } else if state.discard_selected.len() < required_count {
            state.discard_selected.push(id);
            *bg = BackgroundColor(theme::BUTTON_SELECTED_BG);
        }
    }
}

/// Handle clicks on search candidate cards: highlight the selected card and
/// clear the highlight on whichever card was previously selected. The model
/// only carries `Option<CardId>`, so without resetting the prior button the
/// UI would show every clicked card as still selected even though only the
/// latest click counts.
#[allow(clippy::type_complexity)]
pub fn handle_search_select(
    mut state: ResMut<DecisionUiState>,
    mut buttons: Query<(&Interaction, &SearchSelectButton, &mut BackgroundColor), With<Button>>,
) {
    let pressed = buttons
        .iter()
        .find_map(|(i, btn, _)| (*i == Interaction::Pressed).then_some(btn.card_id));
    let Some(picked) = pressed else { return };
    state.search_selected = Some(picked);
    for (_, btn, mut bg) in buttons.iter_mut() {
        *bg = BackgroundColor(if btn.card_id == picked {
            theme::BUTTON_SELECTED_BG
        } else {
            MODAL_TILE_BG
        });
    }
}

/// Handle clicks on the scry toggle buttons: flip the card's Top/Bottom state
/// and update its label + background color.
#[allow(clippy::type_complexity)]
pub fn handle_scry_toggles(
    mut state: ResMut<DecisionUiState>,
    mut toggles: Query<
        (&Interaction, &ScryToggleButton, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    mut texts: Query<&mut Text>,
) {
    for (interaction, button, mut bg, children) in toggles.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(entry) = state.scry.iter_mut().find(|(id, _)| *id == button.card_id) else {
            continue;
        };
        entry.1 = !entry.1;
        let going_bottom = entry.1;
        *bg = BackgroundColor(if going_bottom { theme::BUTTON_SELECTED_BG } else { MODAL_TILE_BG });
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = if going_bottom { "Bottom".into() } else { "Top".into() };
            }
        }
    }
}

/// Handle clicks on ← / → reorder buttons: swap the card in the ordering and
/// respawn the modal to reflect the new positions.
pub fn handle_scry_reorder(
    mut state: ResMut<DecisionUiState>,
    query: Query<(&Interaction, &ScryReorderButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(pos) = state.scry.iter().position(|(id, _)| *id == btn.card_id) else {
            continue;
        };
        let new_pos =
            (pos as i32 + btn.delta).clamp(0, state.scry.len() as i32 - 1) as usize;
        if new_pos != pos {
            state.scry.swap(pos, new_pos);
        }
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle clicks on the OrderTriggers ← / → buttons: swap the trigger in
/// the push ordering and respawn the modal to reflect the new positions.
pub fn handle_trigger_reorder(
    mut state: ResMut<DecisionUiState>,
    query: Query<(&Interaction, &TriggerReorderButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(pos) = state.trigger_order.iter().position(|id| *id == btn.source) else {
            continue;
        };
        let new_pos =
            (pos as i32 + btn.delta).clamp(0, state.trigger_order.len() as i32 - 1) as usize;
        if new_pos != pos {
            state.trigger_order.swap(pos, new_pos);
        }
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle clicks on the CombatDamageOrder ← / → buttons: swap the blocker in
/// the damage ordering and respawn the modal to reflect the new positions.
pub fn handle_damage_order_reorder(
    mut state: ResMut<DecisionUiState>,
    query: Query<(&Interaction, &DamageOrderReorderButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
    mut commands: Commands,
) {
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(pos) = state.damage_order.iter().position(|id| *id == btn.blocker) else {
            continue;
        };
        let new_pos =
            (pos as i32 + btn.delta).clamp(0, state.damage_order.len() as i32 - 1) as usize;
        if new_pos != pos {
            state.damage_order.swap(pos, new_pos);
        }
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle clicks on the AssignCombatDamage +/- steppers: adjust one
/// blocker's share (capped so the total never exceeds the attacker's power)
/// and respawn the modal.
pub fn handle_damage_assign_buttons(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    query: Query<(&Interaction, &DamageAssignButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
    mut commands: Commands,
) {
    let Some(cv) = &view.0 else { return };
    let Some(power) = cv.pending_decision.as_ref().and_then(|p| match p.decision.as_ref() {
        Some(DecisionWire::AssignCombatDamage { attacker_power, .. }) => Some(*attacker_power),
        _ => None,
    }) else {
        return;
    };
    for (interaction, btn) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let total: u32 = state.damage_assign.iter().map(|(_, a)| *a).sum();
        let Some(entry) = state.damage_assign.iter_mut().find(|(id, _)| *id == btn.blocker)
        else {
            continue;
        };
        if btn.delta > 0 && total < power {
            entry.1 += 1;
        } else if btn.delta < 0 && entry.1 > 0 {
            entry.1 -= 1;
        } else {
            continue;
        }
        for e in &modal {
            commands.entity(e).despawn();
        }
        state.spawned_for = None;
    }
}

/// Handle the Confirm button: build the appropriate answer based on which
/// decision is pending and submit it to the server via NetOutbox.
pub fn handle_confirm(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut log: ResMut<GameLog>,
    mut state: ResMut<DecisionUiState>,
    confirm: Query<&Interaction, (Changed<Interaction>, With<DecisionConfirmButton>)>,
) {
    for interaction in &confirm {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(cv) = &view.0 else { continue };
        let wire = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
            Some(d) => d,
            None => continue,
        };

        let answer = match wire {
            DecisionWire::Scry { .. } => {
                let mut kept_top = Vec::new();
                let mut bottom = Vec::new();
                for (id, going_bottom) in &state.scry {
                    if *going_bottom { bottom.push(*id); } else { kept_top.push(*id); }
                }
                DecisionAnswer::ScryOrder { kept_top, bottom }
            }
            DecisionWire::SearchLibrary { .. } => {
                DecisionAnswer::Search(state.search_selected)
            }
            DecisionWire::PutOnLibrary { count, .. } => {
                if state.put_on_library.len() < *count { continue; }
                DecisionAnswer::PutOnLibrary(state.put_on_library.clone())
            }
            DecisionWire::Discard { count, .. } => {
                if state.discard_selected.len() < *count as usize { continue; }
                DecisionAnswer::Discard(state.discard_selected.clone())
            }
            DecisionWire::ChooseCards { min, .. } => {
                // Require at least `min` selected; the per-tile handler already
                // caps selection at `max`.
                if state.discard_selected.len() < *min as usize { continue; }
                DecisionAnswer::Cards(state.discard_selected.clone())
            }
            DecisionWire::OrderTriggers { .. } => {
                DecisionAnswer::TriggerOrder(state.trigger_order.clone())
            }
            DecisionWire::CombatDamageOrder { .. } => {
                DecisionAnswer::DamageOrder(state.damage_order.clone())
            }
            DecisionWire::AssignCombatDamage { .. } => {
                DecisionAnswer::CombatDamageAssignment(state.damage_assign.clone())
            }
            DecisionWire::ChooseModes { .. } => {
                // The engine sanitises (dropping dupes/out-of-range, falling
                // back to the card default when empty), so an under-picked
                // set is safe to send.
                DecisionAnswer::Modes(state.modes_selected.clone().unwrap_or_default())
            }
            DecisionWire::ChooseAmount { .. } => DecisionAnswer::Amount(state.amount),
            DecisionWire::DivideDamage { total, .. } => {
                // CR 601.2d — the split must use the whole total; hold the
                // modal open until the remaining-pool readout hits zero.
                if state.divide.iter().sum::<u32>() != *total { continue; }
                DecisionAnswer::DamageDivision(state.divide.clone())
            }
            _ => continue,
        };

        if let Some(outbox) = &outbox {
            outbox.submit(GameAction::SubmitDecision(answer));
        }
        log.push("Decision submitted.");
        state.scry.clear();
        state.search_selected = None;
        state.put_on_library.clear();
        state.discard_selected.clear();
        state.trigger_order.clear();
        state.damage_order.clear();
        state.damage_assign.clear();
        state.modes_selected = None;
        state.amount = 0;
        state.divide.clear();
        state.spawned_for = None;
    }
}


/// Update the live "X / N selected" text in the PutOnLibrary banner.
pub fn update_put_on_library_count_text(
    view: Res<CurrentView>,
    state: Res<DecisionUiState>,
    mut q: Query<&mut Text, With<PutOnLibraryCountText>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(count) = cv.pending_decision.as_ref().and_then(|p| match p.decision.as_ref() {
        Some(DecisionWire::PutOnLibrary { count, .. }) => Some(*count),
        _ => None,
    }) else {
        return;
    };
    let selected = state.put_on_library.len();
    for mut t in &mut q {
        t.0 = format!("{selected} / {count} selected");
    }
}

/// Sync the `PutOnLibraryHighlight` marker component on the viewer's hand
/// cards to mirror `state.put_on_library`. Spawns/despawns gold border child
/// meshes so the player can see which cards are currently selected to bottom.
#[allow(clippy::type_complexity)]
pub fn update_put_on_library_visuals(
    mut commands: Commands,
    state: Res<DecisionUiState>,
    view: Res<CurrentView>,
    highlight_assets: Option<Res<crate::card::CardHighlightAssets>>,
    hand_cards: Query<
        (Entity, &crate::card::GameCardId, Option<&PutOnLibraryHighlight>),
        With<crate::card::HandCard>,
    >,
) {
    // Active only while a PutOnLibrary decision is pending for the viewer.
    let active = view
        .0
        .as_ref()
        .and_then(|cv| {
            let pd = cv.pending_decision.as_ref()?;
            if pd.acting_player != cv.your_seat {
                return None;
            }
            match pd.decision.as_ref()? {
                DecisionWire::PutOnLibrary { .. } => Some(()),
                _ => None,
            }
        })
        .is_some();

    let Some(assets) = highlight_assets else { return };
    let selected: std::collections::HashSet<CardId> = if active {
        state.put_on_library.iter().copied().collect()
    } else {
        std::collections::HashSet::new()
    };

    for (entity, gid, marker) in &hand_cards {
        let should_be_selected = selected.contains(&gid.0);
        match (should_be_selected, marker) {
            (true, None) => {
                let offset = crate::card::CARD_THICKNESS / 2.0 + 0.0015;
                let back = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.border_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -offset),
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                let front = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.border_material.clone()),
                        Transform::from_xyz(0.0, 0.0, offset),
                        bevy::picking::Pickable::IGNORE,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert(PutOnLibraryHighlight { back, front })
                    .add_children(&[back, front]);
            }
            (false, Some(highlight)) => {
                commands.entity(highlight.back).despawn();
                commands.entity(highlight.front).despawn();
                commands.entity(entity).remove::<PutOnLibraryHighlight>();
            }
            _ => {}
        }
    }
}

/// Click on a 3D hand card while a PutOnLibrary decision is pending: toggle
/// the card's membership in `state.put_on_library`.
pub fn handle_put_on_library_hand_click(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mouse: Res<bevy::input::ButtonInput<MouseButton>>,
    hovered_hand: Query<&crate::card::GameCardId, (With<crate::card::CardHovered>, With<crate::card::HandCard>)>,
) {
    let Some(cv) = &view.0 else { return };
    let pending = match cv.pending_decision.as_ref() {
        Some(p) if p.acting_player == cv.your_seat => p,
        _ => return,
    };
    let Some(DecisionWire::PutOnLibrary { count, .. }) = pending.decision.as_ref() else { return };
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(gid) = hovered_hand.iter().next() else { return };
    let id = gid.0;
    if let Some(pos) = state.put_on_library.iter().position(|x| *x == id) {
        state.put_on_library.remove(pos);
    } else if state.put_on_library.len() < *count {
        state.put_on_library.push(id);
    }
}

/// Handle Keep / Mulligan / Serum Powder button presses (and keyboard
/// shortcuts K / M / P for the first-listed powder).
pub fn handle_mulligan_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    keep_q: Query<&Interaction, (Changed<Interaction>, With<MulliganKeepButton>)>,
    mull_q: Query<&Interaction, (Changed<Interaction>, With<MulliganTakeButton>)>,
    powder_q: Query<
        (&Interaction, &MulliganSerumPowderButton),
        (Changed<Interaction>, With<Button>),
    >,
) {
    let Some(cv) = &view.0 else { return };
    let serum_powders = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
        Some(DecisionWire::Mulligan { serum_powders, .. }) => serum_powders.clone(),
        _ => return,
    };
    let Some(outbox) = outbox else { return };

    let keep = keep_q.iter().any(|i| *i == Interaction::Pressed) || keyboard.just_pressed(KeyCode::KeyK);
    let mull = mull_q.iter().any(|i| *i == Interaction::Pressed) || keyboard.just_pressed(KeyCode::KeyM);
    let pressed_powder = powder_q
        .iter()
        .find_map(|(int, btn)| (*int == Interaction::Pressed).then_some(btn.0))
        .or_else(|| {
            // P shortcut → consume the first listed powder.
            (keyboard.just_pressed(KeyCode::KeyP)).then(|| serum_powders.first().copied()).flatten()
        });

    if keep {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Keep));
        state.spawned_for = None;
    } else if mull {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::TakeMulligan));
        state.spawned_for = None;
    } else if let Some(id) = pressed_powder {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::SerumPowder(id)));
        state.spawned_for = None;
    }
}

// ── ChooseColor modal (Black Lotus / Birds of Paradise) ─────────────────────

#[derive(Component)]
pub struct ChooseColorButton(pub crabomination::mana::Color);

/// A Yes/No button for a `Decision::OptionalTrigger` prompt; `.0` is the
/// answer submitted (`true` = yes).
#[derive(Component)]
pub struct OptionalChoiceButton(pub bool);

/// A yes/no confirmation modal for `Decision::OptionalTrigger` — currently the
/// "spend your floating mana, or tap lands?" prompt. `description` carries the
/// specifics; the buttons answer `Bool(true)` / `Bool(false)`.
fn spawn_optional_modal(commands: &mut Commands, ui_fonts: &UiFonts, description: &str) {
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(16.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(70.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(description.to_string()),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            ..default()
        })
        .with_children(|row| {
            for (answer, label, bg) in [
                (true, "Yes", theme::BUTTON_PRIMARY_BG),
                (false, "No", theme::BUTTON_INFO_BG),
            ] {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(bg),
                    HoverTint::new(bg),
                    OptionalChoiceButton(answer),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        ui_fonts.tf(18.0),
                        TextColor(theme::TEXT_PRIMARY),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
    });
}

/// Submit the `Bool` answer for a pending `OptionalTrigger` when its Yes/No
/// button is clicked.
pub fn handle_optional_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &OptionalChoiceButton), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    if !matches!(
        cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()),
        Some(DecisionWire::OptionalTrigger { .. })
    ) {
        return;
    }
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Bool(btn.0)));
            state.spawned_for = None;
            return;
        }
    }
}

fn spawn_choose_color_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    legal: &[crabomination::mana::Color],
) {
    use crabomination::mana::Color as ManaColor;
    let root = commands
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
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(14.0),
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Choose a color"),
            ui_fonts.tf(18.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(10.0),
            ..default()
        })
        .with_children(|row| {
            for c in legal {
                let (bg, label) = match c {
                    ManaColor::White => (Color::srgb(0.92, 0.92, 0.78), "White (W)"),
                    ManaColor::Blue  => (Color::srgb(0.30, 0.55, 0.90), "Blue (U)"),
                    ManaColor::Black => (Color::srgb(0.20, 0.20, 0.24), "Black (B)"),
                    ManaColor::Red   => (Color::srgb(0.85, 0.28, 0.20), "Red (R)"),
                    ManaColor::Green => (Color::srgb(0.25, 0.60, 0.30), "Green (G)"),
                };
                let text_color = if matches!(c, ManaColor::White) {
                    Color::BLACK
                } else {
                    Color::WHITE
                };
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(bg),
                    ChooseColorButton(*c),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        ui_fonts.tf(14.0),
                        TextColor(text_color),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        });
    });
}

// ── Name-a-card modal (CR 201.3) ─────────────────────────────────────────────

/// One suggestion (or the "name nothing" decline) in the NameCard modal.
#[derive(Component)]
pub struct NameCardPickButton(pub String);

fn spawn_name_card_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    source_name: &str,
    suggestions: &[String],
) {
    let root = commands
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
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(8.0),
                align_items: AlignItems::Stretch,
                min_width: Val::Px(280.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(format!("{source_name} — choose a card name")),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        for name in suggestions.iter().take(8) {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(theme::BUTTON_INFO_BG),
                NameCardPickButton(name.clone()),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(name.clone()),
                    ui_fonts.tf(13.0),
                    TextColor(theme::TEXT_PRIMARY),
                    bevy::picking::Pickable::IGNORE,
                ));
            });
        }
        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(7.0)),
                justify_content: JustifyContent::Center,
                margin: UiRect::top(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(theme::BUTTON_TERTIARY_BG),
            NameCardPickButton(String::new()),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Name nothing"),
                ui_fonts.tf(13.0),
                TextColor(theme::TEXT_SECONDARY),
                bevy::picking::Pickable::IGNORE,
            ));
        });
    });
}

/// Click a NameCard suggestion → submit the name (empty = name nothing).
pub fn handle_name_card_buttons(
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &NameCardPickButton), Changed<Interaction>>,
) {
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::NamedCard(btn.0.clone())));
            state.spawned_for = None;
            return;
        }
    }
}

// ── Learn modal (Lessons sideboard) ─────────────────────────────────────────

#[derive(Component)]
pub struct LearnFetchButton(pub CardId);
#[derive(Component)]
pub struct LearnRummageButton(pub CardId);
#[derive(Component)]
pub struct LearnDeclineButton;

fn spawn_learn_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    lessons: &[(CardId, String)],
    hand: &[(CardId, String)],
) {
    let root = commands
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
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(10.0),
                align_items: AlignItems::Stretch,
                min_width: Val::Px(280.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);

    // A labelled, full-width pick button. Returns it so the caller tags it
    // with the right marker component.
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Learn — reveal a Lesson, or discard a card to draw"),
            ui_fonts.tf(16.0),
            TextColor(theme::TEXT_PRIMARY),
        ));

        p.spawn((
            Text::new("Reveal a Lesson into your hand:"),
            ui_fonts.tf(13.0),
            TextColor(theme::TEXT_SECONDARY),
        ));
        for (id, name) in lessons {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.25, 0.45, 0.30)),
                LearnFetchButton(*id),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(name.clone()),
                    ui_fonts.tf(14.0),
                    TextColor(Color::WHITE),
                    bevy::picking::Pickable::IGNORE,
                ));
            });
        }

        if !hand.is_empty() {
            p.spawn((
                Text::new("Or discard a card to draw:"),
                ui_fonts.tf(13.0),
                TextColor(theme::TEXT_SECONDARY),
            ));
            for (id, name) in hand {
                p.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.40, 0.30, 0.20)),
                    LearnRummageButton(*id),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(format!("Discard {name}")),
                        ui_fonts.tf(13.0),
                        TextColor(Color::WHITE),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        }

        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(Color::srgb(0.30, 0.30, 0.34)),
            LearnDeclineButton,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Decline"),
                ui_fonts.tf(14.0),
                TextColor(Color::WHITE),
                bevy::picking::Pickable::IGNORE,
            ));
        });
    });
}

/// Click handler for the Learn modal's three action types. Submits the
/// chosen `LearnChoice` as the pending decision's answer.
pub fn handle_learn_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    fetch: Query<(&Interaction, &LearnFetchButton), Changed<Interaction>>,
    rummage: Query<(&Interaction, &LearnRummageButton), Changed<Interaction>>,
    decline: Query<&Interaction, (Changed<Interaction>, With<LearnDeclineButton>)>,
) {
    use crabomination::decision::LearnChoice;
    let Some(cv) = &view.0 else { return };
    if !matches!(
        cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()),
        Some(DecisionWire::Learn { .. })
    ) {
        return;
    }
    let Some(outbox) = outbox else { return };
    let submit = |choice: LearnChoice, state: &mut DecisionUiState| {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Learn(choice)));
        state.spawned_for = None;
    };
    for (interaction, btn) in &fetch {
        if *interaction == Interaction::Pressed {
            submit(LearnChoice::FetchLesson(btn.0), &mut state);
            return;
        }
    }
    for (interaction, btn) in &rummage {
        if *interaction == Interaction::Pressed {
            submit(LearnChoice::Rummage { discard: btn.0 }, &mut state);
            return;
        }
    }
    for interaction in &decline {
        if *interaction == Interaction::Pressed {
            submit(LearnChoice::Decline, &mut state);
            return;
        }
    }
}

// ── Mode pick (modal "Choose one —" spells) ───────────────────────────────

/// Marker on a modal-spell `Modes:` panel; despawned together when the
/// pick is submitted or cancelled.
#[derive(Component)]
pub struct ModalCastModal;

#[derive(Component)]
pub struct ModalCastButton(pub usize);

#[derive(Component)]
pub struct ModalCastCancel;

/// Spawn / despawn the "Choose one —" picker for modal spells. Driven by
/// the [`crate::game::PendingModalCast`] resource: when its `card_id`
/// becomes `Some`, a modal lists every mode's short description; clicking
/// a button either casts immediately (mode has no target) or arms the
/// targeting cursor with `pending_mode` set.
pub fn spawn_mode_pick_ui(
    mut commands: Commands,
    pending: Res<crate::game::PendingModalCast>,
    existing: Query<Entity, With<ModalCastModal>>,
    ui_fonts: Res<UiFonts>,
) {
    if pending.card_id.is_none() {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }
    if existing.iter().next().is_some() {
        return;
    }
    let root = commands
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
            BackgroundColor(theme::OVERLAY_BG),
            Button,
            ModalCastModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Stretch,
                min_width: Val::Px(360.0),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);
    let name = pending.card_name.clone();
    let modes = pending.modes.clone();
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(format!("{name} — choose one")),
            ui_fonts.tf(18.0),
            TextColor(theme::TEXT_PRIMARY),
        ));
        for (idx, (desc, needs_target)) in modes.iter().enumerate() {
            let label = if desc.is_empty() {
                format!("Mode {}", idx + 1)
            } else if *needs_target {
                format!("{}. {} (pick target)", idx + 1, desc)
            } else {
                format!("{}. {}", idx + 1, desc)
            };
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                ModalCastButton(idx),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(label),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        }
        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_TERTIARY_BG),
            HoverTint::new(theme::BUTTON_TERTIARY_BG),
            ModalCastCancel,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Cancel"),
                ui_fonts.tf(12.0),
                TextColor(theme::TEXT_SECONDARY),
                Pickable::IGNORE,
            ));
        });
    });
}

pub fn handle_mode_pick_buttons(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<crate::game::PendingModalCast>,
    mut targeting: ResMut<crate::game::TargetingState>,
    mut legal_targets: ResMut<crate::game::LegalTargets>,
    mut esc_consumed: ResMut<crate::systems::quality::EscConsumed>,
    btns: Query<(&Interaction, &ModalCastButton), Changed<Interaction>>,
    cancels: Query<&Interaction, (Changed<Interaction>, With<ModalCastCancel>)>,
) {
    if pending.card_id.is_none() {
        return;
    }
    // Esc dismisses the modal pick — sibling to the Cancel button.
    // Eat the keypress via `EscConsumed` so the same Esc doesn't also
    // close the settings panel / trigger any other Esc-bound action.
    if keyboard.just_pressed(KeyCode::Escape) {
        pending.card_id = None;
        pending.card_name.clear();
        pending.modes.clear();
        esc_consumed.0 = true;
        return;
    }
    for i in &cancels {
        if *i == Interaction::Pressed {
            pending.card_id = None;
            pending.card_name.clear();
            pending.modes.clear();
            return;
        }
    }
    let Some(outbox) = outbox else { return };
    for (i, btn) in &btns {
        if *i != Interaction::Pressed {
            continue;
        }
        let idx = btn.0;
        let needs_target = pending.modes.get(idx).map(|(_, n)| *n).unwrap_or(false);
        let card_id = pending.card_id.unwrap();
        if needs_target {
            targeting.active = true;
            targeting.pending_card_id = Some(card_id);
            targeting.pending_mode = Some(idx);
            targeting.back_face_pending = false;
            // Populate the highlight set from the chosen mode's slot-0
            // filter so the user sees rings on legal creatures (the
            // earlier path left it empty, which was the source of the
            // "highlights players but not creatures" bug).
            if let (Some(cv), name) = (&view.0, pending.card_name.clone())
                && let Some(legal) = crate::systems::legal_target_filter::enumerate_for_cast(
                    cv,
                    &name,
                    Some(idx),
                )
            {
                *legal_targets = legal;
            }
        } else {
            outbox.submit(GameAction::CastSpell {
                card_id,
                target: None,
                additional_targets: vec![],
                mode: Some(idx),
                x_value: None,
            });
        }
        pending.card_id = None;
        pending.card_name.clear();
        pending.modes.clear();
        return;
    }
}

/// Click handler for color buttons in the `ChooseColor` modal. Submits the
/// chosen color as the pending decision's answer. Also accepts the W/U/B/R/G
/// keyboard shortcuts (only for colors actually offered).
pub fn handle_choose_color_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    buttons: Query<(&Interaction, &ChooseColorButton), Changed<Interaction>>,
) {
    use crabomination::mana::Color as ManaColor;
    let Some(cv) = &view.0 else { return };
    let legal = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
        Some(DecisionWire::ChooseColor { legal, .. }) => legal.clone(),
        _ => return,
    };
    let Some(outbox) = outbox else { return };
    // Keyboard shortcut — submit the matching color if it's on offer.
    let key_color = if keyboard.just_pressed(KeyCode::KeyW) { Some(ManaColor::White) }
        else if keyboard.just_pressed(KeyCode::KeyU) { Some(ManaColor::Blue) }
        else if keyboard.just_pressed(KeyCode::KeyB) { Some(ManaColor::Black) }
        else if keyboard.just_pressed(KeyCode::KeyR) { Some(ManaColor::Red) }
        else if keyboard.just_pressed(KeyCode::KeyG) { Some(ManaColor::Green) }
        else { None };
    if let Some(c) = key_color
        && legal.contains(&c)
    {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Color(c)));
        state.spawned_for = None;
        return;
    }
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Color(btn.0)));
            state.spawned_for = None;
            return;
        }
    }
}

// ── Resolution-time choice modals (CR 700.2d / 601.2d / amounts / types) ─────
//
// These five answer the decisions the engine raises through the
// stash-and-rerun suspend path (`PendingEffectState::*AnswerPending`):
// "choose N" / Escalate mode sets, deferred modal-trigger picks, "choose a
// number", divided damage, and creature-type choices.

/// Display name for `id` resolved from the live view — battlefield first,
/// then known stack items — falling back to `fallback`.
fn view_card_name(
    cv: &crabomination::net::ClientView,
    id: CardId,
    fallback: &str,
) -> String {
    cv.battlefield
        .iter()
        .find(|p| p.id == id)
        .map(|p| p.name.clone())
        .or_else(|| {
            cv.stack.iter().find_map(|s| match s {
                crabomination::net::StackItemView::Known(k) if k.source == id => {
                    Some(k.name.clone())
                }
                _ => None,
            })
        })
        .unwrap_or_else(|| fallback.to_string())
}

fn mode_label(idx: usize, texts: &[String]) -> String {
    match texts.get(idx) {
        Some(t) if !t.is_empty() => format!("{}. {}", idx + 1, t),
        _ => format!("Mode {}", idx + 1),
    }
}

/// Toggle for one mode in the ChooseModes modal. Flips membership in
/// `DecisionUiState::modes_selected`.
#[derive(Component)]
pub struct ChooseModesToggle(pub u8);

/// Single-pick button in the deferred trigger ChooseMode modal — submits
/// `DecisionAnswer::Mode` immediately.
#[derive(Component)]
pub struct TriggerModeButton(pub usize);

/// −/+ stepper in the ChooseAmount modal.
#[derive(Component)]
pub struct AmountStepButton(pub i64);

/// Marker for the ChooseAmount modal's live value readout.
#[derive(Component)]
pub struct AmountValueText;

/// −/+ stepper for one target row of the DivideDamage modal.
#[derive(Component)]
pub struct DivideDamageButton {
    pub index: usize,
    pub delta: i32,
}

/// Marker for one target row's live amount readout (`.0` = target index).
#[derive(Component)]
pub struct DivideValueText(pub usize);

/// Marker for the DivideDamage modal's "unassigned damage" readout.
#[derive(Component)]
pub struct DivideRemainingText;

/// Pick button in the ChooseCreatureType modal — submits immediately.
#[derive(Component)]
pub struct CreatureTypePickButton(pub crabomination::card::CreatureType);

/// Shared scaffold: full-screen centering root + column panel, returning the
/// panel entity for the caller to fill.
fn spawn_modal_panel(commands: &mut Commands, min_width: f32) -> Entity {
    let root = commands
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
            DecisionModal,
        ))
        .id();
    let panel = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                min_width: Val::Px(min_width),
                border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                ..default()
            },
            BackgroundColor(theme::PANEL_BG),
        ))
        .id();
    commands.entity(root).add_child(panel);
    panel
}

fn spawn_confirm_button(panel: &mut ChildSpawnerCommands, ui_fonts: &UiFonts) {
    panel
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_PRIMARY_BG),
            HoverTint::new(theme::BUTTON_PRIMARY_BG),
            DecisionConfirmButton,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Confirm"),
                ui_fonts.tf(18.0),
                TextColor(theme::TEXT_PRIMARY),
                bevy::picking::Pickable::IGNORE,
            ));
        });
}

fn spawn_choose_modes_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    title: &str,
    num_modes: usize,
    count: usize,
    mode_texts: &[String],
    selected: &[u8],
) {
    let panel = spawn_modal_panel(commands, 380.0);
    // `count == num_modes` is the Escalate shape ("choose one or more,
    // paying for extras"); an exact-N pick otherwise.
    let prompt = if count == num_modes {
        format!("{title} — choose one or more")
    } else {
        format!("{title} — choose {count}")
    };
    let title_owned = prompt;
    let rows: Vec<(u8, String, bool)> = (0..num_modes)
        .map(|i| {
            (i as u8, mode_label(i, mode_texts), selected.contains(&(i as u8)))
        })
        .collect();
    let fonts18 = ui_fonts.tf(18.0);
    let fonts14 = ui_fonts.tf(14.0);
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title_owned), fonts18.clone(), TextColor(theme::TEXT_PRIMARY)));
        for (idx, label, is_on) in rows {
            let bg = if is_on { theme::BUTTON_SELECTED_BG } else { theme::BUTTON_NEUTRAL_BG };
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                BackgroundColor(bg),
                ChooseModesToggle(idx),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(label),
                    fonts14.clone(),
                    TextColor(theme::TEXT_PRIMARY),
                    bevy::picking::Pickable::IGNORE,
                ));
            });
        }
        spawn_confirm_button(p, ui_fonts);
    });
}

fn spawn_choose_trigger_mode_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    title: &str,
    num_modes: usize,
    mode_texts: &[String],
) {
    let panel = spawn_modal_panel(commands, 380.0);
    let title_owned = format!("{title} — choose one");
    let labels: Vec<String> = (0..num_modes).map(|i| mode_label(i, mode_texts)).collect();
    let fonts18 = ui_fonts.tf(18.0);
    let fonts14 = ui_fonts.tf(14.0);
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title_owned), fonts18.clone(), TextColor(theme::TEXT_PRIMARY)));
        for (idx, label) in labels.into_iter().enumerate() {
            p.spawn((
                Button,
                Node {
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(10.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    align_self: AlignSelf::Stretch,
                    ..default()
                },
                BackgroundColor(theme::BUTTON_PRIMARY_BG),
                HoverTint::new(theme::BUTTON_PRIMARY_BG),
                TriggerModeButton(idx),
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new(label),
                    fonts14.clone(),
                    TextColor(theme::TEXT_PRIMARY),
                    bevy::picking::Pickable::IGNORE,
                ));
            });
        }
    });
}

fn spawn_choose_amount_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    prompt: &str,
    max: u32,
    value: u32,
) {
    let panel = spawn_modal_panel(commands, 320.0);
    let prompt_owned = format!("{prompt} (0–{max})");
    let fonts18 = ui_fonts.tf(18.0);
    let fonts16 = ui_fonts.tf(16.0);
    let fonts22 = ui_fonts.tf(22.0);
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(prompt_owned), fonts18.clone(), TextColor(theme::TEXT_PRIMARY)));
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(12.0),
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|row| {
            for (label, delta) in
                [("−5", -5i64), ("−", -1), ("+", 1), ("+5", 5), ("Max", i64::MAX)]
            {
                // Value readout sits between the − and + clusters.
                if delta == 1 {
                    row.spawn((
                        Text::new(value.to_string()),
                        fonts22.clone(),
                        TextColor(theme::ACCENT_GOLD),
                        AmountValueText,
                        Node {
                            min_width: Val::Px(56.0),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                    ));
                }
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    AmountStepButton(delta),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        fonts16.clone(),
                        TextColor(theme::TEXT_PRIMARY),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        });
        spawn_confirm_button(p, ui_fonts);
    });
}

fn spawn_divide_damage_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    title: &str,
    total: u32,
    target_names: &[String],
    amounts: &[u32],
) {
    let panel = spawn_modal_panel(commands, 380.0);
    let title_owned = format!("{title} — divide {total} damage");
    let assigned: u32 = amounts.iter().sum();
    let fonts18 = ui_fonts.tf(18.0);
    let fonts16 = ui_fonts.tf(16.0);
    let fonts14 = ui_fonts.tf(14.0);
    let rows: Vec<(usize, String, u32)> = target_names
        .iter()
        .enumerate()
        .map(|(i, n)| (i, n.clone(), amounts.get(i).copied().unwrap_or(0)))
        .collect();
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title_owned), fonts18.clone(), TextColor(theme::TEXT_PRIMARY)));
        for (index, name, amount) in rows {
            p.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(10.0),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Stretch,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            })
            .with_children(|row| {
                row.spawn((
                    Text::new(name),
                    fonts14.clone(),
                    TextColor(theme::TEXT_BODY),
                ));
                row.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(8.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|stepper| {
                    for (label, delta) in [("−", -1i32), ("+", 1)] {
                        if delta == 1 {
                            stepper.spawn((
                                Text::new(amount.to_string()),
                                fonts16.clone(),
                                TextColor(theme::ACCENT_GOLD),
                                DivideValueText(index),
                                Node {
                                    min_width: Val::Px(32.0),
                                    justify_content: JustifyContent::Center,
                                    ..default()
                                },
                            ));
                        }
                        stepper.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                                ..default()
                            },
                            BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                            HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                            DivideDamageButton { index, delta },
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(label),
                                fonts16.clone(),
                                TextColor(theme::TEXT_PRIMARY),
                                bevy::picking::Pickable::IGNORE,
                            ));
                        });
                    }
                });
            });
        }
        p.spawn((
            Text::new(format!("Unassigned: {}", total - assigned.min(total))),
            fonts14.clone(),
            TextColor(theme::TEXT_SECONDARY),
            DivideRemainingText,
        ));
        spawn_confirm_button(p, ui_fonts);
    });
}

fn spawn_creature_type_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    suggestions: &[crabomination::card::CreatureType],
) {
    let panel = spawn_modal_panel(commands, 420.0);
    let fonts18 = ui_fonts.tf(18.0);
    let fonts14 = ui_fonts.tf(14.0);
    let types: Vec<crabomination::card::CreatureType> = suggestions.to_vec();
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new("Choose a creature type"),
            fonts18.clone(),
            TextColor(theme::TEXT_PRIMARY),
        ));
        p.spawn(Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            max_width: Val::Px(560.0),
            justify_content: JustifyContent::Center,
            ..default()
        })
        .with_children(|grid| {
            for ct in types {
                grid.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                    CreatureTypePickButton(ct),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(format!("{ct:?}")),
                        fonts14.clone(),
                        TextColor(theme::TEXT_PRIMARY),
                        bevy::picking::Pickable::IGNORE,
                    ));
                });
            }
        });
    });
}

/// Flip one mode's membership in the working ChooseModes selection and
/// restyle the toggle in place. An exact-N decision (count < num_modes)
/// evicts the oldest pick once over budget, so the player can always click
/// their way to a legal set.
pub fn handle_choose_modes_toggle(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mut toggles: Query<
        (&Interaction, &ChooseModesToggle, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut all_toggles: Query<(&ChooseModesToggle, &mut BackgroundColor), Without<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some((count, num_modes)) =
        cv.pending_decision.as_ref().and_then(|p| match p.decision.as_ref() {
            Some(DecisionWire::ChooseModes { count, num_modes, .. }) => {
                Some((*count, *num_modes))
            }
            _ => None,
        })
    else {
        return;
    };
    let mut evicted: Option<u8> = None;
    for (interaction, toggle, mut bg) in toggles.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let selected = state.modes_selected.get_or_insert_with(Vec::new);
        if let Some(pos) = selected.iter().position(|&m| m == toggle.0) {
            selected.remove(pos);
            *bg = BackgroundColor(theme::BUTTON_NEUTRAL_BG);
        } else {
            selected.push(toggle.0);
            *bg = BackgroundColor(theme::BUTTON_SELECTED_BG);
            // Exact-N picks: drop the oldest selection once over budget.
            if count < num_modes && selected.len() > count {
                evicted = Some(selected.remove(0));
            }
        }
    }
    if let Some(old) = evicted {
        for (toggle, mut bg) in all_toggles.iter_mut() {
            if toggle.0 == old {
                *bg = BackgroundColor(theme::BUTTON_NEUTRAL_BG);
            }
        }
    }
}

/// Submit a deferred trigger-mode pick the moment its button is clicked.
pub fn handle_trigger_mode_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &TriggerModeButton), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    if !matches!(
        cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()),
        Some(DecisionWire::ChooseMode { .. })
    ) {
        return;
    }
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Mode(btn.0)));
            state.spawned_for = None;
            return;
        }
    }
}

/// Adjust the working ChooseAmount value (clamped to `0..=max`) and update
/// the readout in place.
pub fn handle_amount_buttons(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &AmountStepButton), Changed<Interaction>>,
    mut value_text: Query<&mut Text, With<AmountValueText>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(max) = cv.pending_decision.as_ref().and_then(|p| match p.decision.as_ref() {
        Some(DecisionWire::ChooseAmount { max, .. }) => Some(*max),
        _ => None,
    }) else {
        return;
    };
    let mut changed = false;
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let next = if btn.0 == i64::MAX {
            max as i64
        } else {
            state.amount as i64 + btn.0
        };
        state.amount = next.clamp(0, max as i64) as u32;
        changed = true;
    }
    if changed {
        for mut t in &mut value_text {
            t.0 = state.amount.to_string();
        }
    }
}

/// Adjust one target's share of a divided-damage split (each target keeps
/// ≥1 per CR 601.2d; the total never exceeds the spell's amount) and update
/// the row + remaining readouts in place.
pub fn handle_divide_damage_buttons(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &DivideDamageButton), Changed<Interaction>>,
    mut row_texts: Query<(&mut Text, &DivideValueText), Without<DivideRemainingText>>,
    mut remaining_text: Query<&mut Text, (With<DivideRemainingText>, Without<DivideValueText>)>,
) {
    let Some(cv) = &view.0 else { return };
    let Some(total) = cv.pending_decision.as_ref().and_then(|p| match p.decision.as_ref() {
        Some(DecisionWire::DivideDamage { total, .. }) => Some(*total),
        _ => None,
    }) else {
        return;
    };
    let mut changed = false;
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let assigned: u32 = state.divide.iter().sum();
        let Some(v) = state.divide.get_mut(btn.index) else { continue };
        if btn.delta > 0 && assigned < total {
            *v += 1;
            changed = true;
        } else if btn.delta < 0 && *v > 1 {
            *v -= 1;
            changed = true;
        }
    }
    if changed {
        for (mut t, marker) in &mut row_texts {
            if let Some(v) = state.divide.get(marker.0) {
                t.0 = v.to_string();
            }
        }
        let assigned: u32 = state.divide.iter().sum();
        for mut t in &mut remaining_text {
            t.0 = format!("Unassigned: {}", total.saturating_sub(assigned));
        }
    }
}

/// Submit a creature-type pick the moment its button is clicked.
pub fn handle_creature_type_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &CreatureTypePickButton), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    if !matches!(
        cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()),
        Some(DecisionWire::ChooseCreatureType { .. })
    ) {
        return;
    }
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::CreatureType(btn.0)));
            state.spawned_for = None;
            return;
        }
    }
}
