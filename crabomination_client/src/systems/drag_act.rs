//! Drag to act: aim a spell by dragging it onto its target, send an
//! attacker by dragging it onto the player or planeswalker it attacks, and
//! block by dragging a blocker onto the attacker.
//!
//! Everything already worked by clicks, two in a row: the card, then what it
//! acts on. A drag is those two clicks in one gesture. The press is the
//! first click, as ever (it arms the spell, picks the attacker or the
//! blocker); letting go over something else is the second — this module
//! notices the drag and, on release, has the input handler take it as a
//! click on whatever is under the pointer ([`DragAct::release_click`], or
//! the player's HUD panel via `ButtonState::player_chip`). A press and
//! release without moving is still just the click it always was.
//!
//! While a creature is dragged, an arrow runs from it to the pointer,
//! snapping to what the release would pick (`gizmos::draw_drag_arrow`). A
//! spell being aimed has its own arrow already.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use crabomination::card::CardId;
use crabomination::game::{AttackTarget, TurnStep};
use crabomination::net::ClientView;

use crate::card::{BattlefieldCard, CardHovered, GameCardId, HandCard, StackCard};
use crate::game::{AttackingState, BlockingState, TargetingState};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::{ButtonState, PlayerHudPanel};

/// How far (UI px) the pointer must travel with the button down before a
/// press becomes a drag.
const DRAG_START: f32 = 12.0;

/// What the viewer is choosing, which decides what a drag can do.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DragMode {
    /// Aiming a spell or ability: let go over its target.
    Aim,
    /// Picking attackers: let go over the player, planeswalker or battle.
    Attack,
    /// Picking blockers: let go over the attacker.
    Block,
}

/// The viewer's current [`DragMode`], if a drag could do anything.
pub fn drag_mode(cv: &ClientView, targeting: &TargetingState, blocking: &BlockingState) -> Option<DragMode> {
    let viewer = cv.your_seat;
    if cv.step == TurnStep::DeclareBlockers && cv.declares_blocks(viewer) && cv.priority == viewer && !blocking.declared {
        return Some(DragMode::Block);
    }
    if cv.step == TurnStep::DeclareAttackers && cv.declares_attacks(viewer) && cv.priority == viewer {
        return Some(DragMode::Attack);
    }
    targeting.active.then_some(DragMode::Aim)
}

/// Whether letting a drag go over the battlefield card `over` should click
/// it. Aiming: any card (the handler checks it's a legal target). Picking
/// attackers: not one of the viewer's own creatures, which a click would
/// add to the attack. Picking blockers: an attacker.
pub fn release_clicks(mode: DragMode, cv: &ClientView, over: CardId) -> bool {
    let Some(card) = cv.battlefield.iter().find(|c| c.id == over) else {
        // A spell on the stack (counter magic).
        return mode == DragMode::Aim;
    };
    match mode {
        DragMode::Aim => true,
        DragMode::Attack => card.controller != cv.active_player,
        DragMode::Block => card.attacking,
    }
}

/// The drag in progress, if any, and what its release does this frame.
#[derive(Resource, Default)]
pub struct DragAct {
    /// Where the left button went down (UI px) and the card under it.
    press: Option<(Vec2, Option<CardId>)>,
    /// The pointer has travelled far enough since the press to be a drag.
    dragging: bool,
    /// This frame the drag was let go over a card it acts on: the input
    /// handler takes the release as a click there.
    pub release_click: bool,
    /// Held without a button down: the layout harness's staged drag.
    staged: bool,
}

impl DragAct {
    /// The card being dragged, once the press has become a drag.
    pub fn dragged(&self) -> Option<CardId> {
        self.dragging.then_some(self.press.and_then(|(_, card)| card)).flatten()
    }

    /// Hold `card` mid-drag, for the layout harness's screenshot: it stays
    /// held though no button is down.
    pub(crate) fn stage(&mut self, card: CardId) {
        self.press = Some((Vec2::ZERO, Some(card)));
        self.dragging = true;
        self.staged = true;
    }
}

/// Bevy system: follow the left button through a drag and, on release over
/// something the drag acts on, arm the click for `handle_game_input` (which
/// runs after it). Runs after the HUD panels' own click poll, so a release
/// over a player's panel can stand in for a press on it.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn track_drag(
    mut drag: ResMut<DragAct>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
    view: Res<CurrentView>,
    (targeting, blocking, mut attacking, mut buttons): (
        Res<TargetingState>,
        Res<BlockingState>,
        ResMut<AttackingState>,
        ResMut<ButtonState>,
    ),
    hovered: Query<&GameCardId, (With<CardHovered>, Or<(With<BattlefieldCard>, With<HandCard>, With<StackCard>)>)>,
    hovered_bf: Query<&GameCardId, (With<CardHovered>, Or<(With<BattlefieldCard>, With<StackCard>)>)>,
    panels: Query<(&Interaction, &PlayerHudPanel)>,
) {
    drag.release_click = false;
    let cursor = windows.single().ok().and_then(Window::cursor_position).map(|p| p / ui_scale.0);
    if mouse.just_pressed(MouseButton::Left) {
        let under = hovered.iter().next().map(|g| g.0);
        drag.press = cursor.map(|at| (at, under));
        drag.dragging = false;
        return;
    }
    let Some((from, grabbed)) = drag.press else { return };
    let Some(cv) = view.0.as_ref() else { return };
    let mode = drag_mode(cv, &targeting, &blocking);

    if mouse.pressed(MouseButton::Left) || drag.staged {
        if !drag.dragging && cursor.is_some_and(|at| at.distance(from) > DRAG_START) {
            drag.dragging = true;
            // The press toggled the attacker: off again, if it was already
            // in the plan. Dragging it means sending it, so put it back
            // (for the release to aim) — if it may attack at all.
            if mode == Some(DragMode::Attack)
                && let Some(card) = grabbed
                && cv.legal_attackers.contains(&card)
                && !attacking.contains(card)
            {
                attacking.plan.push((card, AttackTarget::Player(default_defender(cv))));
                attacking.last_added = Some(card);
            }
        }
        return;
    }

    // Let go.
    let was_dragging = std::mem::take(&mut drag.dragging);
    drag.press = None;
    let Some(mode) = mode.filter(|_| was_dragging) else { return };
    if let Some(over) = hovered_bf.iter().map(|g| g.0).find(|id| Some(*id) != grabbed) {
        drag.release_click = release_clicks(mode, cv, over);
        return;
    }
    // Over a player's HUD panel: aiming at them, or sending the attacker
    // at them. (The panel only reads as pressed for a press on it.)
    if matches!(mode, DragMode::Aim | DragMode::Attack)
        && let Some(seat) = panels.iter().find(|(i, _)| **i == Interaction::Hovered).map(|(_, p)| p.seat)
        && seat < cv.players.len()
    {
        buttons.player_chip = Some(seat);
    }
}

/// The player an attacker goes at before the viewer aims it: the next
/// opponent in turn order that can be attacked.
fn default_defender(cv: &ClientView) -> usize {
    crate::systems::game_ui::default_attack_target_seat(cv)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn permanent(id: u32, controller: usize) -> crabomination::net::PermanentView {
        let mut p = crate::systems::counter_tooltip::tests::make_permanent_view(0, 2);
        p.id = CardId(id);
        p.controller = controller;
        p.owner = controller;
        p
    }

    fn combat(step: TurnStep, active: usize) -> ClientView {
        let mut attacker = permanent(1, 1);
        attacker.attacking = step == TurnStep::DeclareBlockers;
        ClientView {
            your_seat: 0,
            active_player: active,
            priority: 0,
            step,
            battlefield: vec![attacker, permanent(2, 0), permanent(3, 1)],
            ..Default::default()
        }
    }

    /// A headless app running [`track_drag`] over `cv`, with a window to
    /// point in and the battlefield's cards spawned.
    fn app(cv: ClientView) -> App {
        let mut app = App::new();
        app.init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(UiScale(1.0))
            .init_resource::<TargetingState>()
            .init_resource::<BlockingState>()
            .init_resource::<AttackingState>()
            .init_resource::<ButtonState>()
            .init_resource::<DragAct>()
            .add_systems(Update, track_drag);
        for p in &cv.battlefield {
            app.world_mut().spawn((GameCardId(p.id), BattlefieldCard { is_land: false, is_token: false }));
        }
        app.insert_resource(CurrentView(Some(cv)));
        app.world_mut().spawn((Window::default(), PrimaryWindow));
        app
    }

    /// Put the pointer at `at` over `card`, with the left button `down`,
    /// and run a frame.
    fn frame(app: &mut App, at: Vec2, card: Option<u32>, down: bool) {
        let world = app.world_mut();
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        windows.single_mut(world).unwrap().set_cursor_position(Some(at));
        let mut hovered = world.query_filtered::<Entity, With<CardHovered>>();
        for e in hovered.iter(world).collect::<Vec<_>>() {
            world.entity_mut(e).remove::<CardHovered>();
        }
        if let Some(id) = card {
            let mut cards = world.query::<(Entity, &GameCardId)>();
            let e = cards.iter(world).find(|(_, g)| g.0 == CardId(id)).map(|(e, _)| e).unwrap();
            world.entity_mut(e).insert(CardHovered);
        }
        let mut mouse = world.resource_mut::<ButtonInput<MouseButton>>();
        mouse.clear();
        if down {
            mouse.press(MouseButton::Left);
        } else {
            mouse.release(MouseButton::Left);
        }
        app.update();
    }

    #[test]
    fn letting_a_blocker_go_over_an_attacker_clicks_it() {
        let mut app = app(combat(TurnStep::DeclareBlockers, 1));
        // Press on the blocker, drag, let go over the attacker.
        frame(&mut app, Vec2::new(100.0, 400.0), Some(2), true);
        frame(&mut app, Vec2::new(150.0, 300.0), None, true);
        frame(&mut app, Vec2::new(200.0, 200.0), Some(1), false);
        assert!(app.world().resource::<DragAct>().release_click);
        // The next frame it's spent.
        frame(&mut app, Vec2::new(200.0, 200.0), Some(1), false);
        assert!(!app.world().resource::<DragAct>().release_click);

        // A press and release on the spot is just the click it always was.
        frame(&mut app, Vec2::new(100.0, 400.0), Some(2), true);
        frame(&mut app, Vec2::new(104.0, 402.0), Some(1), false);
        assert!(!app.world().resource::<DragAct>().release_click);
    }

    #[test]
    fn dragging_a_planned_attacker_keeps_it_in_the_attack() {
        let mut cv = combat(TurnStep::DeclareAttackers, 0);
        cv.legal_attackers = vec![CardId(2)];
        let mut app = app(cv);
        app.world_mut().resource_mut::<AttackingState>().plan.push((CardId(2), AttackTarget::Player(1)));
        frame(&mut app, Vec2::new(100.0, 400.0), Some(2), true);
        // The input handler's click took it out of the attack...
        app.world_mut().resource_mut::<AttackingState>().remove(CardId(2));
        // ...and the drag puts it back, ready to aim.
        frame(&mut app, Vec2::new(160.0, 300.0), None, true);
        let attacking = app.world().resource::<AttackingState>();
        assert!(attacking.contains(CardId(2)));
        assert_eq!(attacking.last_added, Some(CardId(2)));
        assert_eq!(app.world().resource::<DragAct>().dragged(), Some(CardId(2)));
        // Let go over the opponent's creature: a click there.
        frame(&mut app, Vec2::new(200.0, 150.0), Some(3), false);
        assert!(app.world().resource::<DragAct>().release_click);
    }

    #[test]
    fn a_drag_acts_only_where_its_click_would_mean_the_same() {
        let (aim, idle) = (TargetingState { active: true, ..Default::default() }, TargetingState::default());
        let blocks = combat(TurnStep::DeclareBlockers, 1);
        assert_eq!(drag_mode(&blocks, &idle, &BlockingState::default()), Some(DragMode::Block));
        // A blocker goes onto an attacker, not onto another creature.
        assert!(release_clicks(DragMode::Block, &blocks, CardId(1)));
        assert!(!release_clicks(DragMode::Block, &blocks, CardId(3)));
        // Once the blocks are in, a drag does nothing.
        let declared = BlockingState { declared: true, ..Default::default() };
        assert_eq!(drag_mode(&blocks, &idle, &declared), None);

        let attacks = combat(TurnStep::DeclareAttackers, 0);
        assert_eq!(drag_mode(&attacks, &idle, &BlockingState::default()), Some(DragMode::Attack));
        // An attacker goes onto an opponent's permanent, not another of
        // the viewer's creatures (a click there would add it to the attack).
        assert!(release_clicks(DragMode::Attack, &attacks, CardId(3)));
        assert!(!release_clicks(DragMode::Attack, &attacks, CardId(2)));

        // Aiming: any card; the handler checks the target is legal.
        let main = combat(TurnStep::PreCombatMain, 0);
        assert_eq!(drag_mode(&main, &aim, &BlockingState::default()), Some(DragMode::Aim));
        assert!(release_clicks(DragMode::Aim, &main, CardId(2)));
        assert_eq!(drag_mode(&main, &idle, &BlockingState::default()), None);
    }
}
