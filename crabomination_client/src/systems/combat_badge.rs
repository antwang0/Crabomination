//! Combat chips: ⚔ on each attacker and 🛡 on each blocker, at the top edge
//! of the card as seen — or, when another card lies over that edge, at the
//! top of the part of the card that shows.
//!
//! Attackers were marked by crossed swords and blockers by diamonds drawn in
//! gizmo lines over the middle of the card, across its art, and the block
//! picker's "blocked / unblocked / selected" said the same thing again with
//! diamonds in three colours. A chip keeps the art clear, follows the card
//! when it taps or lunges, hides with the rest of the card's overlays when
//! another card lies over it (`card::cover`) by moving down to what shows,
//! and changes colour with the
//! card's part in the combat. The arrows (`systems::arrows`) say what is
//! attacking or blocking what.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::TurnStep;
use crabomination::net::ClientView;

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, GameCardId};
use crate::game::{AttackingState, BlockingState};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::InGameRoot;
use crate::theme::UiFonts;

/// Over the card's other overlays, under popups and modals.
const CHIP_Z: i32 = crate::theme::layer::CARD_OVERLAY + 1;

/// A card's part in the current combat.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    /// Attacking, or in the viewer's attack plan.
    Attacking,
    /// An attacker the viewer, picking blocks, has not yet blocked.
    Unblocked,
    /// An attacker the viewer's block plan blocks.
    Blocked,
    /// A declared blocker.
    Blocking,
    /// A blocker in the viewer's plan, not yet declared.
    PlannedBlocker,
    /// The blocker the viewer has picked up and is about to assign.
    SelectedBlocker,
}

impl Role {
    fn glyph(self) -> &'static str {
        match self {
            Role::Attacking | Role::Unblocked | Role::Blocked => "⚔",
            Role::Blocking | Role::PlannedBlocker | Role::SelectedBlocker => "🛡",
        }
    }

    /// The same colours as the arrows: orange attacks, cyan declared blocks,
    /// green the viewer's planned blocks, gold the pick in hand, red an
    /// attacker still getting through.
    fn colour(self) -> Color {
        match self {
            Role::Attacking => Color::srgb(1.0, 0.5, 0.2),
            Role::Unblocked => Color::srgb(1.0, 0.32, 0.3),
            Role::Blocked | Role::PlannedBlocker => Color::srgb(0.3, 0.92, 0.45),
            Role::Blocking => Color::srgb(0.3, 0.85, 1.0),
            Role::SelectedBlocker => Color::srgb(1.0, 0.85, 0.2),
        }
    }
}

/// Every card with a part in the combat, and what it is. A card that both
/// attacks and blocks can't happen; if a view ever said so, the block wins,
/// being the later declaration.
pub fn roles(cv: &ClientView, blocking: &BlockingState, plan: &AttackingState) -> HashMap<CardId, Role> {
    let mut roles = HashMap::new();
    for c in cv.battlefield.iter().filter(|c| c.attacking) {
        roles.insert(c.id, Role::Attacking);
    }
    // The viewer's attack plan shows the moment a creature is picked, not
    // only once the attack is in.
    if cv.step == TurnStep::DeclareAttackers && cv.active_player == cv.your_seat {
        for (attacker, _) in &plan.plan {
            roles.insert(*attacker, Role::Attacking);
        }
    }
    let in_combat = matches!(
        cv.step,
        TurnStep::DeclareBlockers | TurnStep::FirstStrikeDamage | TurnStep::CombatDamage
    );
    if !in_combat {
        return roles;
    }
    for c in cv.battlefield.iter().filter(|c| !c.blocking_attackers.is_empty()) {
        roles.insert(c.id, Role::Blocking);
    }
    let picking = cv.step == TurnStep::DeclareBlockers && cv.declares_blocks(cv.your_seat);
    if !picking {
        return roles;
    }
    for c in cv.battlefield.iter().filter(|c| c.attacking) {
        let blocked = blocking.assignments.iter().any(|(_, a)| *a == c.id);
        roles.insert(c.id, if blocked { Role::Blocked } else { Role::Unblocked });
    }
    for (blocker, _) in &blocking.assignments {
        roles.insert(*blocker, Role::PlannedBlocker);
    }
    if let Some(blocker) = blocking.selected_blocker {
        roles.insert(blocker, Role::SelectedBlocker);
    }
    roles
}

/// The chip's glyph size on a card `card_width` UI px across.
fn chip_font_size(card_width: f32) -> f32 {
    (card_width * 0.17).round().clamp(16.0, 34.0)
}

/// The chip's disc is its colour; the glyph and rim are that colour, dark.
fn ink(colour: Color) -> Color {
    let c = colour.to_srgba();
    Color::srgb(c.red * 0.2, c.green * 0.2, c.blue * 0.2)
}

/// Steps from the top edge (as seen) toward the opposite edge in which the
/// chip looks for the first part of the card that shows.
const PLACE_STEPS: usize = 8;

/// How far an arrow ending at a chip stops short of its centre, as seen, in
/// card widths: a little past the chip's radius, so the arrow meets its rim
/// instead of running under it.
pub(crate) const CHIP_CLEARANCE: f32 = 0.2;

/// The card's edge midpoints, in card-local space, in opposite pairs
/// (`i ^ 1` is across from `i`).
const EDGES: [Vec3; 4] = [
    Vec3::new(0.0, CARD_HEIGHT / 2.0, 0.0),
    Vec3::new(0.0, -CARD_HEIGHT / 2.0, 0.0),
    Vec3::new(CARD_WIDTH / 2.0, 0.0, 0.0),
    Vec3::new(-CARD_WIDTH / 2.0, 0.0, 0.0),
];

/// Of the card's edge midpoints (`EDGES`, projected to the screen, `None`
/// for one behind the camera), the index of the one highest on the
/// screen: the top as seen, whichever way the card is turned or tapped.
fn top_as_seen(projected: [Option<Vec2>; 4]) -> Option<usize> {
    projected
        .iter()
        .enumerate()
        .filter_map(|(i, p)| p.map(|p| (i, p.y)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(i, _)| i)
}

/// Where a card's combat chip sits, in the card's local space: the middle of
/// its top edge as seen or, when another card lies over that edge, the
/// first point below it that shows. On the printed bottom edge (an
/// opponent's card, upside down as seen) it sits toward the printed left,
/// clear of the P/T badge and the keyword strip that runs from it.
/// `project` takes a card-local point to the screen and `covered` says
/// whether one is under another card. `None` when the card is behind the
/// camera or covered entirely.
pub(crate) fn chip_local(project: impl Fn(Vec3) -> Option<Vec2>, covered: impl Fn(Vec3) -> bool) -> Option<Vec3> {
    let top = top_as_seen(EDGES.map(&project))?;
    let aside = if top == 1 { Vec3::X * -CARD_WIDTH * 0.25 } else { Vec3::ZERO };
    let (from, to) = (EDGES[top] + aside, EDGES[top ^ 1] + aside);
    (0..PLACE_STEPS)
        .map(|k| from.lerp(to, k as f32 / PLACE_STEPS as f32))
        // A point a little inside, so a neighbour that only touches the
        // edge doesn't count.
        .find(|&local| !covered(local * 0.9))
}

/// Screen-space chip marking a card's part in the combat.
#[derive(Component)]
pub struct CombatChip {
    card: CardId,
    role: Role,
}

/// Reconcile the combat chips with the view and the viewer's combat plans.
/// Runs every frame in `AppState::InGame`, after the combat lunge has moved
/// the cards.
#[allow(clippy::too_many_arguments)]
pub fn sync_combat_chips(
    mut commands: Commands,
    view: Res<CurrentView>,
    (blocking, plan): (Res<BlockingState>, Res<AttackingState>),
    ui_fonts: Res<UiFonts>,
    cards: Query<(Entity, &GameCardId, &GlobalTransform), With<BattlefieldCard>>,
    cover_cards: crate::card::cover::CoverQuery,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_scale: Res<UiScale>,
    mut chips: Query<(
        Entity,
        &mut CombatChip,
        &mut Node,
        &mut TextColor,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut TextFont,
    )>,
) {
    let wanted = view.0.as_ref().map(|cv| roles(cv, &blocking, &plan)).unwrap_or_default();
    let Ok((camera, cam_xform)) = camera_q.single() else { return };
    let cover = crate::card::cover::CardCover::new(cam_xform, &cover_cards);

    // card → where its chip goes on screen and its glyph size; a card with
    // another lying over its top edge is left out, and its chip hides.
    let mut placed: HashMap<CardId, (Vec2, f32)> = HashMap::new();
    for (e, gid, gtf) in &cards {
        if !wanted.contains_key(&gid.0) {
            continue;
        }
        let project = |local: Vec3| {
            crate::theme::project_to_ui(camera, cam_xform, &ui_scale, gtf.transform_point(local))
        };
        let Some(local) = chip_local(project, |local| cover.hides_local(e, gtf, local)) else { continue };
        // The glyph is sized to the card's printed width as seen, tapped
        // or not.
        let (Some(at), Some(left), Some(right)) = (project(local), project(EDGES[3]), project(EDGES[2])) else {
            continue;
        };
        placed.insert(gid.0, (at, chip_font_size(left.distance(right))));
    }

    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, mut chip, mut node, mut ink, mut background, mut border, mut font) in &mut chips {
        let Some(&role) = wanted.get(&chip.card) else {
            commands.entity(e).despawn();
            continue;
        };
        seen.insert(chip.card);
        if chip.role != role {
            chip.role = role;
            ink.0 = self::ink(role.colour());
            *background = BackgroundColor(role.colour());
            *border = BorderColor::all(self::ink(role.colour()));
            commands.entity(e).try_insert(crate::theme::OverlayPulse::default());
        }
        match placed.get(&chip.card) {
            Some(&(at, size)) => {
                crate::theme::place_overlay(&mut node, Some(at));
                if font.font_size != FontSize::Px(size) {
                    font.font_size = FontSize::Px(size);
                }
            }
            None => crate::theme::place_overlay(&mut node, None),
        }
    }

    for (&card, &role) in wanted.iter().filter(|(id, _)| !seen.contains(id)) {
        let (at, size) = placed.get(&card).copied().unwrap_or((Vec2::splat(-1000.0), 16.0));
        commands.spawn((
            CombatChip { card, role },
            crate::systems::focus::CardOverlay(card),
            Text::new(role.glyph()),
            ui_fonts.tf(size),
            TextColor(ink(role.colour())),
            BackgroundColor(role.colour()),
            BorderColor::all(ink(role.colour())),
            TextShadow { offset: Vec2::new(0.6, 0.0), color: ink(role.colour()) },
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(at.x),
                top: Val::Px(at.y),
                padding: UiRect::axes(Val::Px(7.0), Val::Px(3.0)),
                border: UiRect::all(Val::Px(2.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(999.0)),
                display: if placed.contains_key(&card) { Display::Flex } else { Display::None },
                ..default()
            },
            // Centred on the edge: left/top are the edge's midpoint.
            UiTransform::from_translation(Val2::percent(-50.0, -50.0)),
            Pickable::IGNORE,
            GlobalZIndex(CHIP_Z),
            InGameRoot,
            crate::theme::OverlayPulse::default(),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::game::AttackTarget;

    fn permanent(id: u32) -> crabomination::net::PermanentView {
        let mut p = crate::systems::counter_tooltip::tests::make_permanent_view(0, 2);
        p.id = CardId(id);
        p
    }

    #[test]
    fn the_block_picker_shows_what_is_blocked_and_what_gets_through() {
        let (mut a, mut b) = (permanent(1), permanent(2));
        a.attacking = true;
        b.attacking = true;
        let cv = ClientView {
            your_seat: 1,
            active_player: 0,
            step: TurnStep::DeclareBlockers,
            battlefield: vec![a, b, permanent(3), permanent(4)],
            ..Default::default()
        };
        let blocking = BlockingState {
            selected_blocker: Some(CardId(4)),
            assignments: vec![(CardId(3), CardId(1))],
            ..Default::default()
        };
        let picked = roles(&cv, &blocking, &AttackingState::default());
        assert_eq!(picked[&CardId(1)], Role::Blocked);
        assert_eq!(picked[&CardId(2)], Role::Unblocked);
        assert_eq!(picked[&CardId(3)], Role::PlannedBlocker);
        assert_eq!(picked[&CardId(4)], Role::SelectedBlocker);

        // The attacker's side sees the declared blocks, not the defender's
        // picker.
        let mut blocker = permanent(3);
        blocker.blocking_attackers = vec![CardId(1)];
        let cv = ClientView { your_seat: 0, battlefield: vec![cv.battlefield[0].clone(), blocker], ..cv };
        let declared = roles(&cv, &BlockingState::default(), &AttackingState::default());
        assert_eq!(declared[&CardId(1)], Role::Attacking);
        assert_eq!(declared[&CardId(3)], Role::Blocking);
    }

    #[test]
    fn a_picked_attacker_is_marked_before_the_attack_is_declared() {
        let cv = ClientView {
            your_seat: 0,
            active_player: 0,
            step: TurnStep::DeclareAttackers,
            battlefield: vec![permanent(1), permanent(2)],
            ..Default::default()
        };
        let plan = AttackingState { plan: vec![(CardId(2), AttackTarget::Player(1))], last_added: None };
        let marked = roles(&cv, &BlockingState::default(), &plan);
        assert_eq!(marked.get(&CardId(2)), Some(&Role::Attacking));
        assert_eq!(marked.get(&CardId(1)), None);
    }

    #[test]
    fn the_chip_sits_on_the_edge_highest_on_screen() {
        let at = |y: f32| Some(Vec2::new(100.0, y));
        // Upright: the card's own top edge.
        assert_eq!(top_as_seen([at(10.0), at(90.0), at(50.0), at(50.0)]), Some(0));
        // Tapped a quarter turn: a long side is on top.
        assert_eq!(top_as_seen([at(50.0), at(50.0), at(10.0), at(90.0)]), Some(2));
        // An edge behind the camera is never chosen.
        assert_eq!(top_as_seen([None, at(90.0), at(50.0), None]), Some(2));
    }
}
