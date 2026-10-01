//! Chips over a card's top-left corner, saying what its border can't.
//!
//! - **FREE** over a hand card a standing static lets you cast for nothing
//!   (`PlayerView.free_castable_hand`: Omniscience, Aluren, Conspiracy
//!   Unraveler). It folds into the same cyan alt-cast border as Dash, kicker
//!   and the rest, so the border alone can't say *why* the card is playable —
//!   or that it costs nothing.
//! - **🖰** over a hand card whose right-click does something particular
//!   ([`hand_menu::right_click`]: pitch it, pay its kicker, conspire, flip it
//!   to its back face, …). Right-click was a priority cascade with nothing
//!   on the card to say which branch it would take; the chip names the
//!   branch — "🖰 Pay kicker" — while the card is hovered or picked by the
//!   keyboard cursor, and is the glyph alone otherwise. A hovered permanent
//!   of yours whose right-click opens its ability menu gets "🖰 Abilities".
//!
//! The top-left corner is the part of a hand card the fan never covers.
//! Mechanism mirrors `agenda_badge`: a screen-space node reprojected from
//! the card's world position, reconciled every frame.

use std::collections::{HashMap, HashSet};

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::TurnStep;
use crabomination::net::{ClientView, HandCardView};

use crate::MainCamera;
use crate::card::{BattlefieldCard, CARD_HEIGHT, CARD_WIDTH, CardHovered, CardOwner, GameCardId, HandCard};
use crate::game::{BlockingState, FlippedHandCards, TargetingState};
use crate::net_plugin::CurrentView;
use crate::systems::game_ui::{InGameRoot, hand_menu};
use crate::theme::{self, UiFonts};

/// Same band as the P/T and token badges — popups and modals still win.
const CHIP_Z: i32 = crate::theme::layer::CARD_OVERLAY;
/// Tuck the chips just above the card's projected top-left corner.
const CHIP_OFFSET_X: f32 = 8.0;
const CHIP_OFFSET_Y: f32 = 14.0;
/// The right-click glyph (a two-button mouse).
const MOUSE: &str = "🖰";
/// The same cyan as the alt-cast border (`card::spawn`), so the hint reads
/// as belonging to it.
const HINT_COLOUR: Color = Color::srgb(0.45, 0.88, 1.0);

/// What a card's chips say.
#[derive(Clone, Debug, Default, PartialEq)]
struct Chips {
    free: bool,
    /// The right-click hint: `Some("")` is the glyph alone, `Some(what)` the
    /// glyph and what the click does.
    right_click: Option<String>,
}

/// The row of chips over one card.
#[derive(Component)]
pub struct CardChips {
    card: CardId,
    chips: Chips,
}

/// Does a right-click reach the play dispatch right now? Targeting,
/// attack and block picking each take the right button for themselves
/// ("clear the plan", "cancel"), and off priority it does nothing — the
/// same gates `handle_game_input` passes through first.
fn right_click_plays(cv: &ClientView, targeting: &TargetingState, blocking: &BlockingState) -> bool {
    let seat = cv.your_seat;
    let picking_blocks =
        cv.step == TurnStep::DeclareBlockers && cv.declares_blocks(seat) && cv.priority == seat && !blocking.declared;
    let picking_attacks = cv.step == TurnStep::DeclareAttackers && cv.declares_attacks(seat) && cv.priority == seat;
    cv.game_over.is_none()
        && cv.priority == seat
        && cv.pending_decision.is_none()
        && !targeting.active
        && !picking_blocks
        && !picking_attacks
}

/// Each hand card's chips, `hovered` naming the one to spell out.
fn hand_chips(cv: &ClientView, plays: bool, hovered: Option<CardId>, flipped: &FlippedHandCards) -> HashMap<CardId, Chips> {
    let Some(me) = cv.players.get(cv.your_seat) else { return HashMap::new() };
    me.hand
        .iter()
        .filter_map(|h| match h {
            HandCardView::Known(k) => Some(k),
            _ => None,
        })
        .filter_map(|k| {
            let right_click = plays
                .then(|| hand_menu::right_click(cv, k).hint(cv, k, flipped.flipped.contains(&k.id)))
                .flatten()
                .map(|what| if hovered == Some(k.id) { what } else { String::new() });
            let chips = Chips { free: cv.free_castable_hand.contains(&k.id), right_click };
            (chips != Chips::default()).then_some((k.id, chips))
        })
        .collect()
}

/// Reconcile the chips with the view. Runs every frame in
/// `AppState::InGame`.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn sync_hand_chips(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    targeting: Res<TargetingState>,
    blocking: Res<BlockingState>,
    flipped: Res<FlippedHandCards>,
    hand: Query<(&GameCardId, &GlobalTransform, Has<CardHovered>), With<HandCard>>,
    battlefield: Query<(&GameCardId, &CardOwner, &GlobalTransform), (With<BattlefieldCard>, With<CardHovered>)>,
    camera_q: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    ui_scale: Res<UiScale>,
    mut rows: Query<(Entity, &CardChips, &mut Node)>,
) {
    let Some(cv) = &view.0 else {
        for (e, _, _) in &mut rows {
            commands.entity(e).despawn();
        }
        return;
    };
    let Ok((camera, cam_xform)) = camera_q.single() else { return };

    let plays = right_click_plays(cv, &targeting, &blocking);
    let hovered = hand.iter().find(|(_, _, h)| *h).map(|(id, ..)| id.0);
    let mut desired = hand_chips(cv, plays, hovered, &flipped);
    let corner_local = Vec3::new(-CARD_WIDTH / 2.0, CARD_HEIGHT / 2.0, 0.0);
    let mut corner: HashMap<CardId, Vec3> = hand
        .iter()
        .filter(|(id, ..)| desired.contains_key(&id.0))
        .map(|(id, gtf, _)| (id.0, gtf.transform_point(corner_local)))
        .collect();
    // The hovered permanent: the input handler opens its ability menu for
    // one you own that has an entry.
    if plays && let Some((id, owner, gtf)) = battlefield.iter().next() {
        let menu = owner.0 == cv.your_seat
            && cv.battlefield.iter().find(|p| p.id == id.0).is_some_and(super::game_ui::has_ability_menu_entry);
        if menu {
            desired.insert(id.0, Chips { free: false, right_click: Some("Abilities".into()) });
            corner.insert(id.0, gtf.transform_point(corner_local));
        }
    }

    let anchor = |world: Vec3| -> Option<Vec2> {
        crate::theme::project_to_ui(camera, cam_xform, &ui_scale, world)
            .map(|v| Vec2::new(v.x - CHIP_OFFSET_X, v.y - CHIP_OFFSET_Y))
    };

    let mut seen: HashSet<CardId> = HashSet::new();
    for (e, row, mut node) in &mut rows {
        if desired.get(&row.card) != Some(&row.chips) {
            commands.entity(e).despawn();
            continue;
        }
        seen.insert(row.card);
        crate::theme::place_overlay(&mut node, corner.get(&row.card).copied().and_then(anchor));
    }

    for (id, chips) in desired.into_iter().filter(|(id, _)| !seen.contains(id)) {
        let at = corner.get(&id).copied().and_then(anchor);
        spawn_row(&mut commands, &ui_fonts, id, chips, at);
    }
}

fn spawn_row(commands: &mut Commands, ui_fonts: &UiFonts, card: CardId, chips: Chips, at: Option<Vec2>) {
    let chip = || {
        (
            BackgroundColor(Color::srgba(0.05, 0.05, 0.10, 0.92)),
            Node {
                padding: UiRect::axes(Val::Px(5.0), Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                ..default()
            },
            Pickable::IGNORE,
        )
    };
    let (left, top, display) = match at {
        Some(at) => (at.x, at.y, Display::Flex),
        None => (-1000.0, -1000.0, Display::None),
    };
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(left),
                top: Val::Px(top),
                display,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(4.0),
                ..default()
            },
            Pickable::IGNORE,
            GlobalZIndex(CHIP_Z),
            InGameRoot,
            CardChips { card, chips: chips.clone() },
        ))
        .with_children(|row| {
            if chips.free {
                row.spawn(chip()).with_children(|c| {
                    c.spawn((Text::new("FREE"), ui_fonts.tf(13.0), TextColor(theme::ACCENT_GOLD), Pickable::IGNORE));
                });
            }
            if let Some(what) = &chips.right_click {
                row.spawn(chip()).with_children(|c| {
                    let text = if what.is_empty() { MOUSE.to_string() } else { format!("{MOUSE} {what}") };
                    crate::mana_text::spawn_text(c, ui_fonts, &text, 13.0, HINT_COLOUR).insert(Pickable::IGNORE);
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::catalog;

    /// The chip says what the click does, in full only on the hovered card;
    /// a card whose right-click does nothing a left-click doesn't gets none,
    /// and FREE rides beside the hint on the same card.
    #[test]
    fn a_hand_card_chip_names_its_right_click_when_hovered() {
        let mut g = crabomination::game::two_player_game();
        let kicker = g.add_card_to_hand(0, catalog::burst_lightning());
        let mdfc = g.add_card_to_hand(0, catalog::shatterskull_smashing());
        let plain = g.add_card_to_hand(0, catalog::grizzly_bears());
        let free = g.add_card_to_hand(0, catalog::lightning_bolt());
        let mut cv = crabomination::server::view::project(&g, 0);
        cv.priority = 0;
        cv.kickable_hand = vec![kicker, free];
        cv.castable_hand = vec![plain];
        cv.free_castable_hand = vec![free];
        let flipped = FlippedHandCards::default();

        let chips = hand_chips(&cv, true, Some(kicker), &flipped);
        assert_eq!(chips[&kicker].right_click.as_deref(), Some("Pay kicker"));
        assert_eq!(chips[&mdfc].right_click.as_deref(), Some(""), "glyph only off hover");
        assert!(!chips.contains_key(&plain), "a menu of just Cast is no hint");
        assert_eq!(chips[&free], Chips { free: true, right_click: Some(String::new()) });

        let chips = hand_chips(&cv, true, Some(mdfc), &flipped);
        assert_eq!(chips[&mdfc].right_click.as_deref(), Some("Flip to Shatterskull, the Hammer Pass"));
        let flipped = FlippedHandCards { flipped: [mdfc].into() };
        let chips = hand_chips(&cv, true, Some(mdfc), &flipped);
        assert_eq!(chips[&mdfc].right_click.as_deref(), Some("Flip to Shatterskull Smashing"));

        // While right-click belongs to something else, only FREE shows.
        let chips = hand_chips(&cv, false, Some(kicker), &flipped);
        assert_eq!(chips.keys().copied().collect::<Vec<_>>(), [free]);
        assert_eq!(chips[&free].right_click, None);
    }

    /// Targeting and holding no priority each leave right-click with
    /// nothing to play, so the hint goes; holding priority in a main phase
    /// it shows.
    #[test]
    fn the_hint_shows_only_while_right_click_plays() {
        let g = crabomination::game::two_player_game();
        let mut cv = crabomination::server::view::project(&g, 0);
        cv.priority = 0;
        cv.step = TurnStep::PreCombatMain;
        let blocking = BlockingState::default();
        let mut targeting = TargetingState::default();
        assert!(right_click_plays(&cv, &targeting, &blocking));
        targeting.active = true;
        assert!(!right_click_plays(&cv, &targeting, &blocking));
        targeting.active = false;
        cv.priority = 1;
        assert!(!right_click_plays(&cv, &targeting, &blocking), "off priority");
    }
}
