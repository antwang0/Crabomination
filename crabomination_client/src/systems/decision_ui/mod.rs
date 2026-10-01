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

mod card_pick;
mod choice;
mod order;
pub use card_pick::*;
pub use choice::*;
pub use order::*;
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
pub struct DecisionConfirmButton;

/// Marker for the Cancel button on a cancellable decision modal.
#[derive(Component)]
pub struct DecisionCancelButton;

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

/// Standing answers for optional-trigger prompts, set via the modal's
/// "Always Yes / Always No" buttons (Arena's auto-yes). Keyed by
/// (source, description) so a permanent with several different optional
/// triggers tracks each separately. Reset at match start by
/// `start_net_session_from_menu`; each auto-answer logs a line so the
/// player can tell why no prompt appeared.
///
/// CR 903.9b commander redirects ("send it to the command zone instead?")
/// share the map under [`COMMANDER_REDIRECT_KEY`], keyed by the commander, so
/// "Always Yes" on one commander doesn't answer for its partner.
#[derive(Resource, Default)]
pub struct AutoOptionalAnswers(pub std::collections::HashMap<(CardId, String), bool>);

/// The description slot a standing CR 903.9b commander-redirect answer is
/// stored under in [`AutoOptionalAnswers`]. Not a real trigger description,
/// so it can never collide with one.
pub const COMMANDER_REDIRECT_KEY: &str = "\u{0}commander-redirect";

impl AutoOptionalAnswers {
    /// The standing answer for `commander`'s redirect prompt, if one was set.
    pub fn commander_redirect(&self, commander: CardId) -> Option<bool> {
        self.0.get(&(commander, COMMANDER_REDIRECT_KEY.to_string())).copied()
    }

    /// Remember `answer` for every later redirect prompt of `commander`.
    pub fn set_commander_redirect(&mut self, commander: CardId, answer: bool) {
        self.0.insert((commander, COMMANDER_REDIRECT_KEY.to_string()), answer);
    }
}

/// Local UI state tracked during an in-flight decision. Cleared when the
/// server's `pending_decision` goes back to `None`.
#[derive(Resource, Default)]
pub struct DecisionUiState {
    /// For Scry: per-card "send to bottom" flags (false = keep on top).
    pub scry: Vec<(CardId, bool)>,
    /// For a card pick (search, put-on-library, discard, choose cards —
    /// `card_pick`): the cards picked, in pick order.
    pub picked: Vec<CardId>,
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
    /// For ChooseCreatureTypePair (CR 612.1): the type picked first (the one
    /// being replaced). `None` until the player picks it.
    pub type_pair_from: Option<crabomination::card::CreatureType>,
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
    /// `Decision::ChooseOption` (a CR 701.38 ballot) — keyed by source + the
    /// option list, so a re-ask with different words re-spawns the modal.
    ChooseOption(CardId, Vec<String>),
    /// `Decision::DivideDamage` (CR 601.2d) — keyed by source + total +
    /// target list.
    DivideDamage(CardId, u32, Vec<Target>),
    /// `Decision::ChooseCreatureType` — keyed by the asking source.
    ChooseCreatureType(CardId),
    /// `Decision::ChooseCreatureTypePair` (CR 612.1) — keyed by the target,
    /// plus the staged first pick so the second prompt respawns.
    ChooseCreatureTypePair(CardId, Option<crabomination::card::CreatureType>),
    /// `Decision::CoinFlip` (CR 705) — keyed by the flipping player.
    CoinFlip(usize),
    /// `Decision::DieRoll` (CR 706) — keyed by player + die size.
    DieRoll(usize, u8),
    /// `Decision::CommanderRedirect` (CR 903.9b) — keyed by the commander.
    CommanderRedirect(CardId),
    /// `Decision::ChooseLegendToKeep` (CR 704.5j) — keyed by the duplicates.
    ChooseLegendToKeep(Vec<CardId>),
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
        DecisionWire::ChooseOption { source, options, .. } => {
            Some(DecisionKey::ChooseOption(*source, options.clone()))
        }
        DecisionWire::DivideDamage { source, total, targets, .. } => {
            Some(DecisionKey::DivideDamage(*source, *total, targets.clone()))
        }
        DecisionWire::ChooseCreatureType { source, .. } => {
            Some(DecisionKey::ChooseCreatureType(*source))
        }
        DecisionWire::ChooseCreatureTypePair { source, .. } => {
            Some(DecisionKey::ChooseCreatureTypePair(*source, None))
        }
        DecisionWire::CoinFlip { player } => Some(DecisionKey::CoinFlip(*player)),
        DecisionWire::DieRoll { player, sides } => {
            Some(DecisionKey::DieRoll(*player, *sides))
        }
        DecisionWire::CommanderRedirect { commander, .. } => {
            Some(DecisionKey::CommanderRedirect(*commander))
        }
        DecisionWire::ChooseLegendToKeep { duplicates, .. } => Some(
            DecisionKey::ChooseLegendToKeep(duplicates.iter().map(|(id, _)| *id).collect()),
        ),
        // No wildcard: adding a DecisionWire variant must extend this match
        // (an unhandled decision freezes the game for a human seat).
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

/// Whether a targeting session with no decision behind it — a spell or
/// ability the viewer picked from hand or board, whose legal targets the
/// click filled in (`legal_target_filter::enumerate_for_cast`) — is open.
/// Its legal set stays: [`spawn_decision_ui`] used to clear the set on every
/// frame without a decision for the viewer, which is every frame of such a
/// session, so the legal-target rings never showed and a click anywhere
/// clickable was taken as a target.
fn casting_keeps_legal_targets(targeting: &crate::game::TargetingState) -> bool {
    targeting.active && !targeting.pending_decision_target
}

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
    auto_answers: Res<AutoOptionalAnswers>,
    mut log: ResMut<GameLog>,
) {
    let Some(cv) = &view.0 else {
        // Mulligan / no view yet — tear down any existing modal.
        for e in &existing {
            commands.entity(e).despawn();
        }
        state.scry.clear();
        state.picked.clear();
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
                state.picked.clear();
                state.modes_selected = None;
                state.amount = 0;
                state.divide.clear();
                state.type_pair_from = None;
                state.spawned_for = None;
            }
            if targeting.pending_decision_target {
                targeting.active = false;
                targeting.pending_decision_target = false;
            }
            if !casting_keeps_legal_targets(&targeting) {
                legal_targets.permanents.clear();
                legal_targets.players.clear();
                legal_targets.source_name.clear();
                legal_targets.description.clear();
            }
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
            let names = cards.iter().cloned().collect();
            let (prompt, tiles) = scry_tiles(&state.scry, &names, *mode);
            spawn_order_modal(&mut commands, &asset_server, &ui_fonts, &prompt, OrderList::Scry, &tiles);
        }
        // Every card pick, one picker (`card_pick`): in the 3-D hand when
        // the cards are the viewer's own hand, a grid otherwise.
        DecisionWire::SearchLibrary { .. }
        | DecisionWire::PutOnLibrary { .. }
        | DecisionWire::Discard { .. }
        | DecisionWire::ChooseCards { .. } => {
            state.picked.clear();
            state.spawned_for = Some(key);
            if let Some(pick) = card_pick(cv, wire) {
                spawn_card_pick(&mut commands, &asset_server, &ui_fonts, &pick, pending.cancellable);
            }
        }
        DecisionWire::Mulligan { hand, mulligans_taken, serum_powders, .. } => {
            state.spawned_for = Some(key);
            // Whether you're on the play decides how greedy a keep is —
            // it belongs on the mulligan screen, which is where the
            // decision is actually made.
            let on_the_play = cv.starting_player == cv.your_seat;
            let starter = cv
                .players
                .iter()
                .find(|p| p.seat == cv.starting_player)
                .map(|p| p.name.clone())
                .unwrap_or_else(|| format!("Player {}", cv.starting_player));
            spawn_mulligan_modal(
                &mut commands,
                &asset_server,
                &ui_fonts,
                hand,
                *mulligans_taken,
                serum_powders,
                on_the_play,
                &starter,
            );
        }
        DecisionWire::ChooseColor { legal, .. } => {
            spawn_choose_color_modal(&mut commands, &ui_fonts, &key, legal);
            state.spawned_for = Some(key);
        }
        DecisionWire::Learn { lessons, hand, .. } => {
            spawn_learn_modal(&mut commands, &ui_fonts, &key, lessons, hand);
            state.spawned_for = Some(key);
        }
        DecisionWire::OptionalTrigger { source, description } => {
            state.spawned_for = Some(key.clone());
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
            // A standing "Always Yes / Always No" for this exact prompt
            // answers without a modal (with a log line, so the player can
            // tell why no prompt appeared).
            let standing = auto_answers.0.get(&(*source, description.clone())).copied();
            if paying_for_this {
                if let Some(outbox) = &outbox {
                    outbox.submit_auto(GameAction::SubmitDecision(DecisionAnswer::Bool(true)));
                }
            } else if let Some(answer) = standing {
                if let Some(outbox) = &outbox {
                    outbox.submit_auto(GameAction::SubmitDecision(DecisionAnswer::Bool(answer)));
                    log.push_event(
                        format!(
                            "Auto-answered \"{description}\": {} (set via Always)",
                            if answer { "Yes" } else { "No" },
                        ),
                        theme::TEXT_SECONDARY,
                    );
                }
            } else {
                spawn_optional_modal(&mut commands, &ui_fonts, &key, description, true);
            }
        }
        DecisionWire::NameCard { source_name, suggestions, restriction, .. } => {
            spawn_name_card_modal(&mut commands, &ui_fonts, &key, source_name, suggestions, restriction.as_deref());
            state.spawned_for = Some(key);
        }
        DecisionWire::OrderTriggers { triggers, .. } => {
            if state.trigger_order.is_empty() {
                state.trigger_order = triggers.iter().map(|(id, _)| *id).collect();
            }
            state.spawned_for = Some(key);
            let name_of = |id: CardId| {
                triggers.iter().find(|(t, _)| *t == id).map_or_else(|| "Triggered ability".to_string(), |(_, n)| n.clone())
            };
            let tiles = numbered_tiles(&state.trigger_order, name_of);
            let prompt = "Order your triggers  ·  ← → to reorder  ·  rightmost resolves first";
            spawn_order_modal(&mut commands, &asset_server, &ui_fonts, prompt, OrderList::Triggers, &tiles);
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
            let tiles = numbered_tiles(&state.damage_order, name_of);
            let prompt = format!(
                "{attacker_name}: order {} for damage  ·  ← → to reorder  ·  leftmost damaged first",
                damage_recipient_noun(cv, *attacker),
            );
            spawn_order_modal(&mut commands, &asset_server, &ui_fonts, &prompt, OrderList::DamageOrder, &tiles);
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
                damage_recipient_noun(cv, *attacker),
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
            let title = view_card_name(cv, *source, "Triggered ability");
            spawn_choose_trigger_mode_modal(&mut commands, &ui_fonts, &key, &title, *num_modes, mode_texts);
            state.spawned_for = Some(key);
        }
        DecisionWire::ChooseAmount { prompt, max, .. } => {
            state.amount = 0;
            state.spawned_for = Some(key);
            let cancellable = cv.pending_decision.as_ref().is_some_and(|pd| pd.cancellable);
            spawn_choose_amount_modal(
                &mut commands, &ui_fonts, prompt, *max, state.amount, cancellable,
            );
        }
        DecisionWire::ChooseOption { source, prompt, options } => {
            let title = format!("{} — {prompt}", view_card_name(cv, *source, "Ballot"));
            spawn_option_ballot_modal(&mut commands, &ui_fonts, &key, &title, options);
            state.spawned_for = Some(key);
        }
        DecisionWire::DivideDamage { source, total, targets, noun } => {
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
            spawn_divide_damage_modal(&mut commands, &ui_fonts, &title, *total, noun, &rows, &state.divide);
        }
        DecisionWire::ChooseCreatureType { suggestions, excluded, .. } => {
            state.spawned_for = Some(key);
            spawn_creature_type_modal(
                &mut commands,
                &ui_fonts,
                "Choose a creature type",
                suggestions,
                excluded,
            );
        }
        // CR 612.1 — two picks in sequence: the type being replaced (no
        // exclusion), then its replacement (Wall barred).
        DecisionWire::ChooseCreatureTypePair { suggestions, excluded, .. } => {
            state.spawned_for = Some(key);
            match state.type_pair_from {
                None => spawn_creature_type_modal(
                    &mut commands,
                    &ui_fonts,
                    "Choose the creature type to replace",
                    suggestions,
                    &[],
                ),
                Some(from) => spawn_creature_type_modal(
                    &mut commands,
                    &ui_fonts,
                    &format!("Replace {from:?} with"),
                    suggestions,
                    excluded,
                ),
            }
        }
        DecisionWire::CoinFlip { .. } => {
            state.spawned_for = Some(key);
            spawn_randomizer_modal(&mut commands, &ui_fonts, "Flip a coin", "Flip", 2);
        }
        DecisionWire::DieRoll { sides, .. } => {
            state.spawned_for = Some(key);
            spawn_randomizer_modal(
                &mut commands,
                &ui_fonts,
                &format!("Roll a d{sides}"),
                &format!("Roll d{sides}"),
                *sides,
            );
        }
        DecisionWire::CommanderRedirect { commander, would_be } => {
            state.spawned_for = Some(key.clone());
            let name = view_card_name(cv, *commander, "Your commander");
            let zone = match would_be {
                crabomination::card::Zone::Graveyard => "your graveyard",
                crabomination::card::Zone::Exile => "exile",
                crabomination::card::Zone::Hand => "your hand",
                crabomination::card::Zone::Library => "your library",
                _ => "that zone",
            };
            // A standing "Always Yes / Always No" for this commander answers
            // without a modal, logged like an auto-answered trigger.
            if let Some(answer) = auto_answers.commander_redirect(*commander) {
                if let Some(outbox) = &outbox {
                    outbox.submit_auto(GameAction::SubmitDecision(DecisionAnswer::Bool(answer)));
                    log.push_event(
                        format!(
                            "Auto-answered: {name} {} (set via Always)",
                            if answer {
                                "returns to the command zone".to_string()
                            } else {
                                format!("goes to {zone}")
                            },
                        ),
                        theme::TEXT_SECONDARY,
                    );
                }
            } else {
                spawn_optional_modal(
                    &mut commands,
                    &ui_fonts,
                    &key,
                    &format!("Send {name} to the command zone instead of {zone}?"),
                    true,
                );
            }
        }
        DecisionWire::ChooseLegendToKeep { name, duplicates, .. } => {
            spawn_legend_keep_modal(&mut commands, &ui_fonts, &key, name, duplicates);
            state.spawned_for = Some(key);
        }
        DecisionWire::ChooseTarget { legal, source_name, description, optional, .. } => {
            // No modal — reuse the existing in-scene targeting cursor.
            // Flipping `pending_decision_target` flags `handle_game_input`
            // to submit picks as `DecisionAnswer::Target` instead of
            // wrapping them in `CastSpell` / `ActivateAbility`.
            state.spawned_for = Some(key);
            targeting.active = true;
            targeting.pending_card_id = None;
            targeting.pending_ability_source = None;
            targeting.pending_ability_index = None;
            targeting.pending_ability_mode = None;
            targeting.pending_ability_is_loyalty = false;
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
            legal_targets.declinable = *optional;
        }
    }
}

/// CR 509.2 / 510.1e — the damage-order and assignment modals are shared by
/// both sides of combat: an attacker ordering its blockers, or a multi-block
/// blocker ordering the attackers it blocks. Pick the noun off the live view.
fn damage_recipient_noun(cv: &crabomination::net::ClientView, source: CardId) -> &'static str {
    let is_blocker = cv
        .battlefield
        .iter()
        .find(|p| p.id == source)
        .is_some_and(|p| !p.attacking && !p.blocking_attackers.is_empty());
    if is_blocker { "attackers" } else { "blockers" }
}

/// CR 510.1d / 510.1e — modal letting the damage source's controller split
/// combat damage among the recipients with per-recipient +/- steppers. The
/// engine validates the split and falls back to lethal-in-order if it breaks
/// the ordering rule.
#[allow(clippy::too_many_arguments)]
fn spawn_damage_assign_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    attacker_name: &str,
    recipients: &str,
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
            GlobalZIndex(theme::layer::MODAL)
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
            // A hovered card's preview sits beside the panel, not over
            // the next card in the row.
            crate::systems::ui_card_hover::PreviewAnchor,
        ))
        .id();

    commands.entity(root).add_child(panel);

    let assigned: u32 = assign.iter().map(|(_, a)| *a).sum();
    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!(
                "{attacker_name}: assign combat damage to {recipients}  ·  {assigned} / {attacker_power} assigned",
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
                            Button,
                            crate::systems::ui_card_hover::UiCardHover::card(name, Some(*blocker)),
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
    on_the_play: bool,
    starter_name: &str,
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
        GlobalZIndex(theme::layer::MODAL)
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

    // CR 103.8a — the player on the play skips their first draw, which is
    // the whole reason a marginal hand plays differently from each seat.
    let (play_line, play_color) = if on_the_play {
        ("\u{25b6}  You are on the play (no first draw)".to_string(), theme::ACCENT_GOLD)
    } else {
        (format!("\u{25c0}  {starter_name} is on the play — you are on the draw"), theme::ACCENT_BLUE)
    };

    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title), ui_fonts.tf(18.0), TextColor(theme::TEXT_PRIMARY)));
        p.spawn((Text::new(play_line), ui_fonts.tf(14.0), TextColor(play_color)));
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

/// Back out of a cancellable decision — the Cancel button or Escape.
///
/// Only offered when the engine flags the pending decision as the input
/// half of a cast / activation replay that hasn't paid anything
/// (`PendingDecisionView::cancellable`). Without it, opening Divergent
/// Equation's {X} picker with no mana was a dead end: every answer failed
/// the replayed cast, and the failure put the same modal straight back.
pub fn handle_decision_cancel(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut log: ResMut<GameLog>,
    mut state: ResMut<DecisionUiState>,
    esc: Res<crate::systems::esc::EscFocus>,
    cancel: Query<&Interaction, (Changed<Interaction>, With<DecisionCancelButton>)>,
) {
    let Some(cv) = &view.0 else { return };
    if !cv.pending_decision.as_ref().is_some_and(|pd| pd.cancellable && pd.decision.is_some()) {
        return;
    }
    let clicked = cancel.iter().any(|i| *i == Interaction::Pressed);
    if !clicked && !esc.owns(crate::systems::esc::EscSurface::DecisionPrompt) {
        return;
    }
    if let Some(outbox) = &outbox {
        outbox.submit(GameAction::SubmitDecision(DecisionAnswer::CancelAction));
    }
    // The modal despawns when the server's next view arrives with no
    // pending decision; clear the working state so a later prompt of the
    // same kind doesn't inherit these picks.
    state.scry.clear();
    state.picked.clear();
    state.modes_selected = None;
    state.amount = 0;
    state.divide.clear();
    log.push("Cancelled.");
}

/// Handle the Confirm button: build the appropriate answer based on which
/// decision is pending and submit it to the server via NetOutbox.
pub fn handle_confirm(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut log: ResMut<GameLog>,
    mut state: ResMut<DecisionUiState>,
    mut pending_cast: ResMut<PendingManaCast>,
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
            DecisionWire::SearchLibrary { .. }
            | DecisionWire::PutOnLibrary { .. }
            | DecisionWire::Discard { .. }
            | DecisionWire::ChooseCards { .. } => {
                // Held open until the picks answer it; the click handlers
                // already cap them at the most it takes.
                let Some(pick) = card_pick(cv, wire) else { continue };
                if !pick.ready(state.picked.len()) { continue; }
                // A slot-0 target in a graveyard / exile (the engine's
                // `CastSlot0TargetPick`): stamp the pick onto the held copies
                // of that action, or a `ManualTapRequired` bounce re-arms it
                // targetless and every mana tap re-poses this picker.
                if let DecisionWire::ChooseCards { source, prompt, .. } = wire
                    && prompt.ends_with(crabomination::decision::OFFBOARD_TARGET_PROMPT_SUFFIX)
                    && let Some(picked) = state.picked.first().copied()
                {
                    let target = Target::Permanent(picked);
                    if let Some(outbox) = &outbox {
                        outbox.patch_last_cast_target(*source, target.clone());
                    }
                    if let Some(pc) = pending_cast.0.as_mut()
                        && crate::net_plugin::cast_action_card_id(&pc.action) == *source
                        && let Some(slot) =
                            crate::net_plugin::cast_action_target_mut(&mut pc.action)
                        && slot.is_none()
                    {
                        *slot = Some(target);
                    }
                }
                let Some(answer) = pick_answer(wire, &state.picked) else { continue };
                answer
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
            DecisionWire::ChooseAmount { source, .. } => {
                // If this amount is the X for a suspended cast (the engine's
                // `CastXPick`), stamp it onto the client-side copies of that
                // cast so a `ManualTapRequired` bounce re-arms with the chosen
                // X instead of re-posing this prompt on every mana tap.
                if let Some(outbox) = &outbox {
                    outbox.patch_last_cast_x(*source, state.amount);
                }
                if let Some(pc) = pending_cast.0.as_mut()
                    && crate::net_plugin::cast_action_card_id(&pc.action) == *source
                {
                    crate::net_plugin::patch_cast_x(&mut pc.action, state.amount);
                }
                DecisionAnswer::Amount(state.amount)
            }
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
        state.picked.clear();
        state.trigger_order.clear();
        state.damage_order.clear();
        state.damage_assign.clear();
        state.modes_selected = None;
        state.amount = 0;
        state.divide.clear();
        state.spawned_for = None;
    }
}


/// Handle Keep / Mulligan / Serum Powder button presses (and keyboard
/// shortcuts K / M / P for the first-listed powder).
pub fn handle_mulligan_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    keyboard: Res<ButtonInput<KeyCode>>,
    text_input: crate::systems::input_guard::TextInputGuard,
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

    // Pointer input stays live while a text surface owns the keyboard —
    // clicking Keep with the chat bar open should still work. Only the
    // K / M / P shortcuts are suppressed, so typing "keep my black bird"
    // into chat can't mulligan the hand out from under the player.
    let key = |k: KeyCode| !text_input.typing() && keyboard.just_pressed(k);
    let keep = keep_q.iter().any(|i| *i == Interaction::Pressed) || key(KeyCode::KeyK);
    let mull = mull_q.iter().any(|i| *i == Interaction::Pressed) || key(KeyCode::KeyM);
    let pressed_powder = powder_q
        .iter()
        .find_map(|(int, btn)| (*int == Interaction::Pressed).then_some(btn.0))
        .or_else(|| {
            // P shortcut → consume the first listed powder.
            key(KeyCode::KeyP).then(|| serum_powders.first().copied()).flatten()
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

// ── Yes / no (optional triggers, CR 903.9b commander redirects) ─────────────

/// "Always Yes / Always No" on the yes/no modal (`choice::spawn_optional_modal`):
/// answers now AND records a standing answer for this exact prompt.
#[derive(Component)]
pub struct OptionalAlwaysButton(pub bool);

/// An "Always" click on a pending `OptionalTrigger` or `CommanderRedirect`
/// (CR 903.9b — both yes/no asks sharing the same modal): answer, and record
/// the standing answer. The one-shot Yes / No are `DecisionChoice`s.
pub fn handle_optional_always(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    mut auto_answers: ResMut<AutoOptionalAnswers>,
    always_buttons: Query<(&Interaction, &OptionalAlwaysButton), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    let decision = cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref());
    if !matches!(
        decision,
        Some(DecisionWire::OptionalTrigger { .. } | DecisionWire::CommanderRedirect { .. })
    ) {
        return;
    }
    let Some(outbox) = outbox else { return };
    // "Always" answers now and records the standing answer; future
    // identical prompts auto-answer in `spawn_decision_ui`.
    for (interaction, btn) in &always_buttons {
        if *interaction == Interaction::Pressed {
            match decision {
                Some(DecisionWire::OptionalTrigger { source, description }) => {
                    auto_answers.0.insert((*source, description.clone()), btn.0);
                }
                Some(DecisionWire::CommanderRedirect { commander, .. }) => {
                    auto_answers.set_commander_redirect(*commander, btn.0);
                }
                _ => {}
            }
            outbox.submit(GameAction::SubmitDecision(DecisionAnswer::Bool(btn.0)));
            state.spawned_for = None;
            return;
        }
    }
}

/// "Flip" / "Roll dN" button for a pending CoinFlip or DieRoll. The engine
/// delegates the randomness to the decider (CR 705/706), so the client rolls
/// locally when clicked; `sides == 2` means a coin (answered as `Bool`).
#[derive(Component)]
pub struct RandomizerButton {
    pub sides: u8,
}

/// One-button modal for the randomness decisions: the player clicks
/// Flip/Roll and the client submits a locally generated result. No
/// heads-or-1s picking — the button IS the randomizer.
fn spawn_randomizer_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    title: &str,
    button_label: &str,
    sides: u8,
) {
    let panel = spawn_modal_panel(commands, 260.0);
    let title = title.to_string();
    let button_label = button_label.to_string();
    let fonts18 = ui_fonts.tf(18.0);
    commands.entity(panel).with_children(|p| {
        p.spawn((
            Text::new(title),
            fonts18.clone(),
            TextColor(theme::TEXT_PRIMARY),
        ));
        p.spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(24.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_PRIMARY_BG),
            HoverTint::new(theme::BUTTON_PRIMARY_BG),
            RandomizerButton { sides },
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(button_label),
                fonts18,
                TextColor(theme::TEXT_PRIMARY),
                bevy::picking::Pickable::IGNORE,
            ));
        });
    });
}

/// Roll/flip on click and submit the result. The outcome lands in the game
/// log via the CoinFlipWon/Lost / DiceRolled events, so no local reveal step.
pub fn handle_randomizer_buttons(
    view: Res<CurrentView>,
    outbox: Option<Res<NetOutbox>>,
    mut state: ResMut<DecisionUiState>,
    buttons: Query<(&Interaction, &RandomizerButton), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    let is_coin = match cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref()) {
        Some(DecisionWire::CoinFlip { .. }) => true,
        Some(DecisionWire::DieRoll { .. }) => false,
        _ => return,
    };
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            let answer = if is_coin {
                DecisionAnswer::Bool(rand::random::<bool>())
            } else {
                let sides = btn.sides.max(2);
                DecisionAnswer::DieRoll(rand::random_range(1..=sides))
            };
            outbox.submit(GameAction::SubmitDecision(answer));
            state.spawned_for = None;
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
    mut pending: ResMut<crate::game::PendingModalCast>,
    mut targeting: ResMut<crate::game::TargetingState>,
    mut legal_targets: ResMut<crate::game::LegalTargets>,
    esc: Res<crate::systems::esc::EscFocus>,
    btns: Query<(&Interaction, &ModalCastButton), Changed<Interaction>>,
    cancels: Query<&Interaction, (Changed<Interaction>, With<ModalCastCancel>)>,
) {
    if pending.card_id.is_none() {
        return;
    }
    // Esc dismisses the modal pick — sibling to the Cancel button.
    // Owning the press is what stops the same Esc from also closing the
    // pause menu or clearing the keyboard cursor.
    if esc.owns(crate::systems::esc::EscSurface::ModePick) {
        pending.card_id = None;
        pending.card_name.clear();
        pending.modes.clear();
        pending.prepare_source = None;
        return;
    }
    for i in &cancels {
        if *i == Interaction::Pressed {
            pending.card_id = None;
            pending.card_name.clear();
            pending.modes.clear();
            pending.prepare_source = None;
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
        // A mode-pick decision without a card id shouldn't happen, but a
        // malformed/mismatched server decision must not panic the client.
        let Some(card_id) = pending.card_id else { continue };
        // SOS Prepare — the pick belongs to a prepared creature's inset
        // spell, which casts through its own action.
        let prepare_source = pending.prepare_source;
        if needs_target {
            targeting.active = true;
            targeting.pending_mode = Some(idx);
            targeting.back_face_pending = false;
            targeting.pending_card_id = prepare_source.is_none().then_some(card_id);
            targeting.pending_prepare_source = prepare_source;
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
        } else if let Some(creature_id) = prepare_source {
            outbox.submit(GameAction::CastPrepareSpell {
                creature_id,
                target: None,
                additional_targets: vec![],
                mode: Some(idx),
                x_value: None,
            });
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
        pending.prepare_source = None;
        return;
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
            GlobalZIndex(theme::layer::MODAL)
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

/// Cancel control for a decision the engine says is abandonable (a cast or
/// activation suspended before it paid anything — `PendingDecisionView
/// ::cancellable`). Escape does the same thing; the button is here because
/// a modal with no visible way out reads as a hang.
fn spawn_cancel_button(panel: &mut ChildSpawnerCommands, ui_fonts: &UiFonts) {
    panel
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                ..default()
            },
            BackgroundColor(theme::BUTTON_NEUTRAL_BG),
            HoverTint::new(theme::BUTTON_NEUTRAL_BG),
            DecisionCancelButton,
        ))
        .with_children(|b| {
            b.spawn((
                Text::new("Cancel (Esc)"),
                ui_fonts.tf(16.0),
                TextColor(theme::TEXT_SECONDARY),
                bevy::picking::Pickable::IGNORE,
            ));
        });
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
    commands.entity(panel).with_children(|p| {
        crate::mana_text::spawn_text(p, ui_fonts, &title_owned, 18.0, theme::TEXT_PRIMARY);
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
                crate::mana_text::spawn_text(b, ui_fonts, &label, 14.0, theme::TEXT_PRIMARY)
                    .insert(bevy::picking::Pickable::IGNORE);
            });
        }
        spawn_confirm_button(p, ui_fonts);
    });
}

fn spawn_choose_amount_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    prompt: &str,
    max: u32,
    value: u32,
    cancellable: bool,
) {
    let panel = spawn_modal_panel(commands, 320.0);
    let prompt_owned = format!("{prompt} (0–{max})");
    let fonts16 = ui_fonts.tf(16.0);
    let fonts22 = ui_fonts.tf(22.0);
    commands.entity(panel).with_children(|p| {
        crate::mana_text::spawn_text(p, ui_fonts, &prompt_owned, 18.0, theme::TEXT_PRIMARY);
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
        if cancellable {
            spawn_cancel_button(p, ui_fonts);
        }
    });
}

fn spawn_divide_damage_modal(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    title: &str,
    total: u32,
    noun: &str,
    target_names: &[String],
    amounts: &[u32],
) {
    let panel = spawn_modal_panel(commands, 380.0);
    // Pluralise the divided noun: "5 damage" / "5 +1/+1 counters".
    let label = if noun == "damage" { format!("{total} damage") } else { format!("{total} {noun}s") };
    let title_owned = format!("{title} — divide {label}");
    let assigned: u32 = amounts.iter().sum();
    let fonts16 = ui_fonts.tf(16.0);
    let fonts14 = ui_fonts.tf(14.0);
    let rows: Vec<(usize, String, u32)> = target_names
        .iter()
        .enumerate()
        .map(|(i, n)| (i, n.clone(), amounts.get(i).copied().unwrap_or(0)))
        .collect();
    commands.entity(panel).with_children(|p| {
        crate::mana_text::spawn_text(p, ui_fonts, &title_owned, 18.0, theme::TEXT_PRIMARY);
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
    title: &str,
    suggestions: &[crabomination::card::CreatureType],
    excluded: &[crabomination::card::CreatureType],
) {
    let panel = spawn_modal_panel(commands, 420.0);
    let fonts18 = ui_fonts.tf(18.0);
    let fonts14 = ui_fonts.tf(14.0);
    // CR 205.3m — a forbidden type is never offered, and the title says so
    // ("other than Wall").
    let types: Vec<crabomination::card::CreatureType> =
        suggestions.iter().copied().filter(|t| !excluded.contains(t)).collect();
    let title = if excluded.is_empty() {
        title.to_string()
    } else {
        let names: Vec<String> = excluded.iter().map(|t| format!("{t:?}")).collect();
        format!("{title} (not {})", names.join(" or "))
    };
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(title), fonts18.clone(), TextColor(theme::TEXT_PRIMARY)));
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
    let wire = cv.pending_decision.as_ref().and_then(|p| p.decision.as_ref());
    let pair = match wire {
        Some(DecisionWire::ChooseCreatureType { .. }) => false,
        Some(DecisionWire::ChooseCreatureTypePair { .. }) => true,
        _ => return,
    };
    let Some(outbox) = outbox else { return };
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match (pair, state.type_pair_from) {
            // CR 612.1 first pick — stage it and respawn for the replacement.
            (true, None) => state.type_pair_from = Some(btn.0),
            (true, Some(from)) => {
                outbox.submit(GameAction::SubmitDecision(DecisionAnswer::CreatureTypePair(
                    from, btn.0,
                )));
                state.type_pair_from = None;
            }
            _ => outbox
                .submit(GameAction::SubmitDecision(DecisionAnswer::CreatureType(btn.0))),
        }
        state.spawned_for = None;
        return;
    }
}

#[cfg(test)]
mod commander_redirect_tests {
    use super::*;

    /// CR 903.9b — "Always" on the redirect prompt is remembered per
    /// commander, and never answers an optional trigger from the same card.
    #[test]
    fn standing_redirect_answer_is_per_commander() {
        let mut auto = AutoOptionalAnswers::default();
        let (a, b) = (CardId(1), CardId(2));
        assert_eq!(auto.commander_redirect(a), None);
        auto.set_commander_redirect(a, true);
        auto.set_commander_redirect(b, false);
        assert_eq!(auto.commander_redirect(a), Some(true));
        assert_eq!(auto.commander_redirect(b), Some(false));
        // An optional trigger on the same card is a different prompt.
        assert_eq!(auto.0.get(&(a, "When this enters, draw".to_string())), None);
        // A new game starts from a fresh resource (menu.rs re-inserts it).
        assert_eq!(AutoOptionalAnswers::default().commander_redirect(a), None);
    }
}

#[cfg(test)]
mod legal_target_tests {
    use super::*;

    /// A spell's own targeting session keeps the legal set its click filled
    /// in; a decision's, or no session at all, lets it go.
    #[test]
    fn a_cast_keeps_its_legal_targets_while_it_is_aimed() {
        let mut targeting = crate::game::TargetingState { active: true, ..Default::default() };
        assert!(casting_keeps_legal_targets(&targeting));
        targeting.pending_decision_target = true;
        assert!(!casting_keeps_legal_targets(&targeting));
        targeting = crate::game::TargetingState::default();
        assert!(!casting_keeps_legal_targets(&targeting));
    }
}
