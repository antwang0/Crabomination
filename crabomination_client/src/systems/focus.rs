//! Focus while choosing: while the viewer aims a spell or ability, picks
//! attackers or picks blockers, every card on the table that isn't one of
//! the choices dims, so the choices stand out on a busy board.
//!
//! The choices come from the view and the targeting session: a target's
//! legal set (`LegalTargets`, from the engine or the client's cast-time
//! evaluator), the server's `legal_attackers` and `legal_blockers` (with
//! what they act on: the planeswalkers and battles an attack may aim at,
//! the attackers a blocker may block). With no legal set to go by — a
//! filter the client can't evaluate — nothing dims.
//!
//! A card dims through its own face material (`card_front_material` makes
//! one per card), fading in and out rather than snapping; its counter chips
//! take a darkened twin of their shared material. Its screen-space overlays
//! — the P/T badge, keyword strip, counter labels and chips, pile count —
//! fade with it ([`fade_card_overlays`]): left bright, they held the eye on
//! the very cards that weren't choices.

use std::collections::HashSet;

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::TurnStep;
use crabomination::net::ClientView;

use crate::card::{BattlefieldCard, CardMeshAssets, FrontFaceMesh, GameCardId, StackCard};
use crate::game::{BlockingState, LegalTargets, TargetingState};
use crate::net_plugin::CurrentView;

/// How far a card that isn't a choice darkens: its face drawn at this share
/// of its brightness.
const DIMMED: f32 = 0.38;
/// A dimmed card's overlays are drawn at this share of their opacity.
const OVERLAY_DIMMED: f32 = 0.35;
/// Seconds a card takes to dim or come back.
const FADE_SECS: f32 = 0.18;

/// The cards the viewer may choose from right now, or `None` when they
/// aren't choosing (or there's no telling what's legal).
pub fn choices(cv: &ClientView, targeting: &TargetingState, legal: &LegalTargets, blocking: &BlockingState) -> Option<HashSet<CardId>> {
    // Aiming something: its legal targets. A set that holds only players
    // (Lightning Bolt at a face) leaves every card dim, which is right.
    if targeting.active {
        let enumerated = !legal.permanents.is_empty() || !legal.players.is_empty();
        return enumerated.then(|| legal.permanents.iter().copied().collect());
    }
    let viewer = cv.your_seat;
    // Picking attackers: the creatures that may attack, and the permanents
    // an attack may aim at.
    if cv.step == TurnStep::DeclareAttackers
        && cv.declares_attacks(viewer)
        && cv.priority == viewer
        && !cv.legal_attackers.is_empty()
    {
        let aimed_at = cv
            .battlefield
            .iter()
            .filter(|p| {
                p.controller != cv.active_player
                    && (p.is_planeswalker() || p.card_types.contains(&crabomination::card::CardType::Battle))
            })
            .map(|p| p.id);
        return Some(cv.legal_attackers.iter().copied().chain(aimed_at).collect());
    }
    // Picking blockers: the creatures that may block, and the attackers.
    if cv.step == TurnStep::DeclareBlockers
        && cv.declares_blocks(viewer)
        && !blocking.declared
        && !cv.legal_blockers.is_empty()
    {
        let attackers = cv.battlefield.iter().filter(|p| p.attacking).map(|p| p.id);
        return Some(cv.legal_blockers.iter().copied().chain(attackers).collect());
    }
    None
}

/// How far (0-1) a card has dimmed.
#[derive(Component, Default)]
pub struct FocusDim(f32);

/// The colour a fully dimmed card's face and chips are multiplied by.
pub fn dimmed_tint() -> Color {
    face_tint(1.0)
}

/// A screen-space overlay belonging to the card `CardId` on the table: it
/// fades with the card ([`fade_card_overlays`]).
#[derive(Component, Clone, Copy)]
pub struct CardOverlay(pub CardId);

/// The opacities an overlay node had before it faded, so they can be
/// scaled from and given back.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct FadeBase {
    background: Option<f32>,
    border: Option<f32>,
    text: Option<f32>,
    shadow: Option<f32>,
}

/// The face colour for a card dimmed `level` of the way. Linear, as the
/// material's own white is, so an undimmed face compares equal to it.
fn face_tint(level: f32) -> Color {
    let v = 1.0 + (DIMMED - 1.0) * level.clamp(0.0, 1.0);
    Color::LinearRgba(Color::srgb(v, v, v).to_linear())
}

/// Bevy system: dim every card on the table that isn't among the viewer's
/// [`choices`], fading each toward its level.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn apply_focus_dim(
    mut commands: Commands,
    time: Res<Time>,
    view: Res<CurrentView>,
    (targeting, legal, blocking): (Res<TargetingState>, Res<LegalTargets>, Res<BlockingState>),
    card_assets: Option<Res<CardMeshAssets>>,
    mut cards: Query<
        (Entity, &GameCardId, &Children, Option<&mut FocusDim>),
        Or<(With<BattlefieldCard>, With<StackCard>)>,
    >,
    faces: Query<
        (&MeshMaterial3d<StandardMaterial>, Has<FrontFaceMesh>),
        Or<(With<FrontFaceMesh>, With<crate::systems::counter_coins::CounterCoin>)>,
    >,
    coin_assets: Option<Res<crate::systems::counter_coins::CounterCoinAssets>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let focus = view.0.as_ref().and_then(|cv| choices(cv, &targeting, &legal, &blocking));
    let step = crate::systems::animate::anim_dt(&time) / FADE_SECS;
    let card_back = card_assets.as_ref().map(|a| a.back_material.id());
    for (entity, gid, children, dim) in &mut cards {
        let want = if focus.as_ref().is_some_and(|choices| !choices.contains(&gid.0)) { 1.0 } else { 0.0 };
        let level = match dim {
            Some(mut dim) => {
                if dim.0 == want {
                    // Settled: nothing to write unless the face was swapped
                    // (an MDFC turned over) since.
                    if want == 0.0 {
                        continue;
                    }
                } else {
                    dim.0 = if want > dim.0 { (dim.0 + step).min(want) } else { (dim.0 - step).max(want) };
                }
                dim.0
            }
            None if want == 0.0 => continue,
            None => {
                commands.entity(entity).try_insert(FocusDim(step.min(1.0)));
                step.min(1.0)
            }
        };
        let tint = face_tint(level);
        for child in children.iter() {
            let Ok((face, is_face)) = faces.get(child) else { continue };
            // A counter chip: the shared material or its darkened twin.
            if !is_face {
                if let Some(assets) = &coin_assets {
                    let want = assets.material(level >= 0.5);
                    if face.0 != *want {
                        commands.entity(child).try_insert(MeshMaterial3d(want.clone()));
                    }
                }
                continue;
            }
            // A face-down permanent shows the shared card back: dimming it
            // would dim every card back on the table.
            if Some(face.0.id()) == card_back {
                continue;
            }
            if materials.get(&face.0).is_some_and(|m| m.base_color != tint)
                && let Some(mut material) = materials.get_mut(&face.0)
            {
                material.base_color = tint;
            }
        }
    }
}

/// An overlay's opacity factor for a card dimmed `level` of the way.
fn overlay_factor(level: f32) -> f32 {
    1.0 + (OVERLAY_DIMMED - 1.0) * level.clamp(0.0, 1.0)
}

/// Bevy system (`PostUpdate`, after every overlay's own system has
/// painted it): fade each [`CardOverlay`] — its node and everything under
/// it — with its card's [`FocusDim`], and give the opacity back when the
/// card brightens. The alphas are scaled from those the overlay had before
/// it faded ([`FadeBase`]), so an overlay's own system repainting it
/// mid-fade doesn't compound the fade.
#[allow(clippy::type_complexity)]
pub fn fade_card_overlays(
    mut commands: Commands,
    cards: Query<(&GameCardId, &FocusDim)>,
    overlays: Query<(Entity, &CardOverlay)>,
    children: Query<&Children>,
    mut paint: Query<(
        Option<&mut BackgroundColor>,
        Option<&mut BorderColor>,
        Option<&mut TextColor>,
        Option<&mut TextShadow>,
        Option<&FadeBase>,
    )>,
) {
    let dims: std::collections::HashMap<CardId, f32> = cards.iter().map(|(g, d)| (g.0, d.0)).collect();
    for (root, overlay) in &overlays {
        let factor = overlay_factor(dims.get(&overlay.0).copied().unwrap_or(0.0));
        for entity in std::iter::once(root).chain(children.iter_descendants(root)) {
            let Ok((background, border, text, shadow, base)) = paint.get_mut(entity) else { continue };
            // Unfaded and not fading: nothing to do.
            if factor >= 1.0 && base.is_none() {
                continue;
            }
            let recorded = base.is_some();
            let base = base.copied().unwrap_or(FadeBase {
                background: background.as_ref().map(|c| c.0.alpha()),
                border: border.as_ref().map(|c| c.top.alpha()),
                text: text.as_ref().map(|c| c.0.alpha()),
                shadow: shadow.as_ref().map(|c| c.color.alpha()),
            });
            let fade = |c: &mut Color, a: Option<f32>| {
                if let Some(a) = a {
                    let want = a * factor;
                    if (c.alpha() - want).abs() > 1e-3 {
                        c.set_alpha(want);
                    }
                }
            };
            if let Some(mut c) = background {
                fade(&mut c.0, base.background);
            }
            if let Some(mut c) = border {
                let mut sides = [c.top, c.right, c.bottom, c.left];
                for side in &mut sides {
                    fade(side, base.border);
                }
                if [c.top, c.right, c.bottom, c.left] != sides {
                    [c.top, c.right, c.bottom, c.left] = sides;
                }
            }
            if let Some(mut c) = text {
                fade(&mut c.0, base.text);
            }
            if let Some(mut c) = shadow {
                fade(&mut c.color, base.shadow);
            }
            if factor >= 1.0 {
                commands.entity(entity).try_remove::<FadeBase>();
            } else if !recorded {
                commands.entity(entity).try_insert(base);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::card::CardId;

    fn permanent(id: u32, controller: usize) -> crabomination::net::PermanentView {
        let mut p = crate::systems::counter_tooltip::tests::make_permanent_view(0, 2);
        p.id = CardId(id);
        p.controller = controller;
        p.owner = controller;
        p
    }

    #[test]
    fn aiming_a_spell_leaves_its_legal_targets_lit() {
        let cv = ClientView { your_seat: 0, battlefield: vec![permanent(1, 1), permanent(2, 1)], ..Default::default() };
        let targeting = TargetingState { active: true, ..Default::default() };
        let mut legal = LegalTargets::default();
        legal.permanents.insert(CardId(1));
        let lit = choices(&cv, &targeting, &legal, &BlockingState::default()).unwrap();
        assert!(lit.contains(&CardId(1)) && !lit.contains(&CardId(2)));
        // No legal set to go by: nothing dims.
        assert!(choices(&cv, &targeting, &LegalTargets::default(), &BlockingState::default()).is_none());
        // Not choosing: nothing dims.
        assert!(choices(&cv, &TargetingState::default(), &legal, &BlockingState::default()).is_none());
    }

    #[test]
    fn picking_blocks_lights_the_blockers_and_the_attackers() {
        let mut attacker = permanent(1, 1);
        attacker.attacking = true;
        let cv = ClientView {
            your_seat: 0,
            active_player: 1,
            priority: 0,
            step: TurnStep::DeclareBlockers,
            battlefield: vec![attacker, permanent(2, 0), permanent(3, 0), permanent(4, 1)],
            legal_blockers: vec![CardId(2)],
            ..Default::default()
        };
        let lit = choices(&cv, &TargetingState::default(), &LegalTargets::default(), &BlockingState::default()).unwrap();
        assert_eq!(lit, HashSet::from([CardId(1), CardId(2)]));
        // Once the blocks are in, the table comes back.
        let declared = BlockingState { declared: true, ..Default::default() };
        assert!(choices(&cv, &TargetingState::default(), &LegalTargets::default(), &declared).is_none());
    }

    #[test]
    fn a_dimmed_cards_overlays_fade_and_come_back() {
        let mut app = App::new();
        app.add_systems(Update, fade_card_overlays);
        let card = app.world_mut().spawn((GameCardId(CardId(7)), FocusDim(1.0))).id();
        let badge = app
            .world_mut()
            .spawn((CardOverlay(CardId(7)), BackgroundColor(Color::WHITE.with_alpha(0.9)), TextColor(Color::BLACK)))
            .id();
        let alphas = |app: &App| {
            let e = app.world().entity(badge);
            (e.get::<BackgroundColor>().unwrap().0.alpha(), e.get::<TextColor>().unwrap().0.alpha())
        };
        // Faded from what it had, however many frames it stays faded.
        for _ in 0..3 {
            app.update();
            let (bg, ink) = alphas(&app);
            assert!((bg - 0.9 * OVERLAY_DIMMED).abs() < 1e-4 && (ink - OVERLAY_DIMMED).abs() < 1e-4);
        }
        // The card comes back: so does the overlay's opacity.
        app.world_mut().entity_mut(card).insert(FocusDim(0.0));
        app.update();
        let (bg, ink) = alphas(&app);
        assert!((bg - 0.9).abs() < 1e-4 && (ink - 1.0).abs() < 1e-4);
        assert!(app.world().entity(badge).get::<FadeBase>().is_none());
    }

    #[test]
    fn a_dimmed_face_is_darker_and_an_undimmed_one_untouched() {
        assert_eq!(face_tint(0.0), Color::WHITE);
        assert!((face_tint(1.0).to_srgba().red - DIMMED).abs() < 1e-5);
    }
}
