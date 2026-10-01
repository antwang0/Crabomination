//! The hand-card play-option menu.
//!
//! Right-clicking a hand card runs a *priority cascade* in
//! `handle_game_input`: the first alternative shape that matches wins, and
//! everything below it on the chain is unreachable. Mechanics that were
//! never given a branch at all — Foretell, Plot, Suspend, Bestow, an
//! Adventure half, Reinforce, a Room's doors, morph — had no way to be
//! played from the client even though the engine and the wire view both
//! offered them.
//!
//! This menu is the general surface: one row per way the engine says the
//! card can be played right now, so nothing is shadowed by a higher-
//! priority branch. It opens on `M`, and as the cascade's final `else` —
//! so right-click keeps its established quick-play meaning where it had
//! one, and opens the menu for exactly the cards that used to do nothing.

use bevy::prelude::*;
use crabomination::card::CardId;
use crabomination::game::GameAction;
use crabomination::net::{ClientView, KnownCard};

use crate::game::{HandCastVariant, HelperMechanic, PayTimesMechanic, TargetingState};
use crate::net_plugin::{CurrentView, NetOutbox};
use crate::theme::{self, UiFonts};

/// Which hand card the menu is open for, and where to draw it.
#[derive(Resource, Default)]
pub struct HandMenuState {
    pub card_id: Option<CardId>,
    pub spawn_pos: Vec2,
}

/// Root entity of the floating hand-card menu.
#[derive(Component)]
pub struct HandMenu;

/// One row of the hand-card menu.
#[derive(Component)]
pub struct HandMenuItem {
    pub card_id: CardId,
    pub option: HandPlayOption,
}

/// A way to play a card out of hand. Each maps to one `GameAction`, or to
/// arming a picker that eventually submits one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandPlayOption {
    /// The plain cast, and the MDFC back face.
    Cast,
    CastBack,
    /// Casts that share the standard `{card_id, target, …}` action shape and
    /// so route through the ordinary targeting flow.
    Variant(HandCastVariant),
    /// Zero-argument special actions.
    Foretell,
    Plot,
    Suspend,
    /// CR 702.36 — cast face down as a 2/2 for {3}.
    Morph,
    /// CR 702.77 — from-hand reinforce; takes a creature target.
    Reinforce,
    /// A from-hand activated ability whose cost is discarding the card.
    DiscardAbility,
    /// CR 709.5 — cast a Room, choosing which door to unlock.
    RoomDoor { right: bool },
}

/// The menu row label for an option.
fn option_label(option: HandPlayOption, k: &KnownCard) -> String {
    match option {
        HandPlayOption::Cast => "Cast".to_string(),
        HandPlayOption::CastBack => match &k.back_face_name {
            Some(n) => format!("Cast {n} (back face)"),
            None => "Cast back face".to_string(),
        },
        HandPlayOption::Variant(v) => v.label().to_string(),
        HandPlayOption::Foretell => "Foretell (exile face down)".to_string(),
        HandPlayOption::Plot => "Plot".to_string(),
        HandPlayOption::Suspend => "Suspend".to_string(),
        HandPlayOption::Morph => "Cast face down (morph)".to_string(),
        HandPlayOption::Reinforce => "Reinforce".to_string(),
        HandPlayOption::DiscardAbility => "Activate (discard this)".to_string(),
        HandPlayOption::RoomDoor { right } => {
            format!("Unlock the {} door", if right { "right" } else { "left" })
        }
    }
}

/// Does this option need the player to pick a target before it submits?
fn option_needs_target(option: HandPlayOption, k: &KnownCard) -> bool {
    match option {
        HandPlayOption::Cast => k.needs_target,
        HandPlayOption::CastBack => k.back_needs_target,
        // The engine validates; an Aura-shaped cast always wants a host, and
        // Reinforce always takes a creature.
        HandPlayOption::Variant(HandCastVariant::Bestow) | HandPlayOption::Reinforce => true,
        HandPlayOption::Variant(_) => k.needs_target,
        HandPlayOption::Foretell
        | HandPlayOption::Plot
        | HandPlayOption::Suspend
        | HandPlayOption::Morph
        | HandPlayOption::DiscardAbility
        | HandPlayOption::RoomDoor { .. } => false,
    }
}

/// Every way `card_id` can be played from hand right now, in menu order.
///
/// Pure over the view so it can be tested without a window: the whole
/// point of the menu is that the *set* is right, and that set is exactly
/// what the engine's affordance probes already published.
pub fn hand_play_options(cv: &ClientView, card_id: CardId) -> Vec<HandPlayOption> {
    use HandCastVariant as V;
    let has = |set: &[CardId]| set.contains(&card_id);
    let mut out = Vec::new();
    if has(&cv.castable_hand) {
        out.push(HandPlayOption::Cast);
    }
    if has(&cv.back_castable_hand) {
        out.push(HandPlayOption::CastBack);
    }
    for (set, variant) in [
        (&cv.bestowable_hand, V::Bestow),
        (&cv.adventurable_hand, V::Adventure),
        (&cv.adventure_exile, V::AdventureCreature),
        (&cv.buyback_hand, V::Buyback),
        (&cv.bargainable_hand, V::Bargain),
        (&cv.prototypable_hand, V::Prototype),
        (&cv.castable_plotted, V::Plotted),
    ] {
        if has(set) {
            out.push(HandPlayOption::Variant(variant));
        }
    }
    if has(&cv.foretellable_hand) {
        out.push(HandPlayOption::Foretell);
    }
    if has(&cv.plottable_hand) {
        out.push(HandPlayOption::Plot);
    }
    if has(&cv.suspendable_hand) {
        out.push(HandPlayOption::Suspend);
    }
    if has(&cv.morphable_hand) {
        out.push(HandPlayOption::Morph);
    }
    if has(&cv.reinforceable_hand) {
        out.push(HandPlayOption::Reinforce);
    }
    if has(&cv.discard_activatable_hand) {
        out.push(HandPlayOption::DiscardAbility);
    }
    // CR 709.5 — `room_castable_hand` carries a per-card door bitmask
    // (bit 0 = left, bit 1 = right); offer only the doors it names.
    if let Some((_, doors)) = cv.room_castable_hand.iter().find(|(id, _)| *id == card_id) {
        if doors & 0b01 != 0 {
            out.push(HandPlayOption::RoomDoor { right: false });
        }
        if doors & 0b10 != 0 {
            out.push(HandPlayOption::RoomDoor { right: true });
        }
    }
    out
}

/// What right-clicking a hand card does: the first quick-play shape in the
/// cascade that fits it, else this menu. `handle_game_input` acts on it and
/// `hand_chips` spells it out on the card, so the hint and the click can't
/// disagree.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RightClick {
    /// Pitch it for its mana (Elvish / Simian Spirit Guide).
    Pitch,
    /// Open the alternative-cost picker.
    AltCost,
    /// Step how many times to pay squad / replicate / multikicker.
    PayTimes(PayTimesMechanic),
    /// CR 702.172 — pick Spree / Tiered modes.
    Spree,
    /// Cast a split card's right half.
    SplitRight,
    /// CR 702.165 — cast it promising its gift.
    Gift,
    /// CR 702.183 — cast its Omen half.
    Omen,
    /// CR 702.47 — pick the Splice cards to splice onto it.
    Splice,
    /// CR 702.51 / 702.126 / 701.67 — pick what helps pay.
    Helpers(HelperMechanic),
    /// CR 702.78 — pick the two creatures to conspire with.
    Conspire,
    /// CR 702.33b — cast it with the largest affordable set of its kickers.
    KickerOptions,
    /// CR 702.32 / 702.166 — cast it with its kicker or offspring paid.
    Kicked,
    /// Turn an MDFC to its other face.
    Flip,
    /// Open this menu.
    Menu,
}

/// The right-click for `k`, a card in the viewer's hand, in cascade order.
pub fn right_click(cv: &ClientView, k: &KnownCard) -> RightClick {
    let id = k.id;
    if cv.pitchable_hand.contains(&id) {
        RightClick::Pitch
    } else if k.has_alternative_cost && k.alt_cost_available {
        RightClick::AltCost
    } else if cv.squadable_hand.contains(&id) {
        RightClick::PayTimes(PayTimesMechanic::Squad)
    } else if cv.replicatable_hand.contains(&id) {
        RightClick::PayTimes(PayTimesMechanic::Replicate)
    } else if cv.multikickable_hand.contains(&id) {
        RightClick::PayTimes(PayTimesMechanic::Multikicker)
    } else if cv.spreeable_hand.contains(&id) && !k.spree_mode_labels.is_empty() {
        RightClick::Spree
    } else if cv.splittable_right_hand.contains(&id) {
        RightClick::SplitRight
    } else if k.has_gift {
        RightClick::Gift
    } else if k.has_omen && cv.omenable_hand.contains(&id) {
        RightClick::Omen
    } else if cv.spliceable_hand.iter().any(|(host, _)| *host == id) {
        RightClick::Splice
    } else if cv.convokable_hand.contains(&id) || k.has_waterbend {
        RightClick::Helpers(if k.has_waterbend && !k.has_convoke && !k.has_improvise {
            HelperMechanic::Waterbend
        } else {
            HelperMechanic::Convoke
        })
    } else if cv.conspirable_hand.contains(&id) {
        RightClick::Conspire
    } else if cv.kicker_option_sets.iter().any(|(card, _)| *card == id) {
        RightClick::KickerOptions
    } else if cv.kickable_hand.contains(&id) {
        RightClick::Kicked
    } else if k.back_face_name.is_some() {
        RightClick::Flip
    } else {
        RightClick::Menu
    }
}

impl RightClick {
    /// What the click does, as the card's hint says it: `None` when it does
    /// nothing a left-click doesn't (a menu of just "Cast", or no menu).
    /// `flipped` is whether the card shows its back face.
    pub fn hint(self, cv: &ClientView, k: &KnownCard, flipped: bool) -> Option<String> {
        Some(match self {
            RightClick::Pitch => "Pitch it for mana".into(),
            RightClick::AltCost if k.alt_cost_label.is_empty() => "Alternative cost".into(),
            RightClick::AltCost => format!("Alt. cost: {}", k.alt_cost_label),
            RightClick::PayTimes(m) => m.label().into(),
            RightClick::Spree => "Choose its modes".into(),
            RightClick::SplitRight => "Cast the right half".into(),
            RightClick::Gift if k.gift_label.is_empty() => "Promise the gift".into(),
            RightClick::Gift => format!("Gift {}", k.gift_label),
            RightClick::Omen if k.omen_label.is_empty() => "Cast its Omen".into(),
            RightClick::Omen => format!("Cast {}", k.omen_label),
            RightClick::Splice => "Splice onto it".into(),
            RightClick::Helpers(HelperMechanic::Waterbend) => "Waterbend".into(),
            RightClick::Helpers(_) => match (k.has_convoke, k.has_improvise) {
                (true, true) => "Convoke / improvise",
                (false, true) => "Improvise",
                _ => "Convoke",
            }
            .into(),
            RightClick::Conspire => "Conspire".into(),
            RightClick::KickerOptions | RightClick::Kicked => {
                let offspring = crate::card::oracle::printed(&k.name)
                    .is_some_and(|faces| faces.iter().flat_map(|f| f.paragraphs()).any(|p| p.starts_with("Offspring")));
                if offspring { "Pay offspring" } else { "Pay kicker" }.into()
            }
            RightClick::Flip => match (&k.back_face_name, flipped) {
                (Some(_), true) => format!("Flip to {}", k.name),
                (Some(back), false) => format!("Flip to {back}"),
                (None, _) => return None,
            },
            RightClick::Menu => match hand_play_options(cv, k.id).as_slice() {
                [] | [HandPlayOption::Cast] => return None,
                _ => "Ways to play".into(),
            },
        })
    }
}

/// Spawn or despawn the hand menu from `HandMenuState`.
pub fn spawn_hand_menu(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    menu_state: Res<HandMenuState>,
    existing: Query<Entity, With<HandMenu>>,
    ui_scale: Res<UiScale>,
) {
    if !menu_state.is_changed() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    let Some(card_id) = menu_state.card_id else { return };
    let Some(cv) = view.0.as_ref() else { return };
    let Some(k) = known_card(cv, card_id) else { return };
    let options = hand_play_options(cv, card_id);
    if options.is_empty() {
        return;
    }
    // The cursor, in window px; a node's `Val::Px` is scaled by `UiScale`.
    let pos = menu_state.spawn_pos / ui_scale.0;
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
            HandMenu,
        ))
        .with_children(|menu| {
            menu.spawn((
                Text::new(k.name.clone()),
                ui_fonts.tf(13.0),
                TextColor(theme::ACCENT_BLUE),
            ));
            for option in options {
                menu.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                    HandMenuItem { card_id, option },
                ))
                .with_children(|b| {
                    b.spawn((
                        crate::mana_text::mana_text(option_label(option, &k), 13.0, theme::TEXT_PRIMARY),
                        Pickable::IGNORE,
                    ));
                });
            }
        });
}

/// Apply a clicked menu row: submit immediately, or arm the in-scene
/// targeting cursor for the options that take a target.
pub fn handle_hand_menu(
    outbox: Option<Res<NetOutbox>>,
    view: Res<CurrentView>,
    mut targeting: ResMut<TargetingState>,
    mut menu_state: ResMut<HandMenuState>,
    query: Query<(&Interaction, &HandMenuItem), Changed<Interaction>>,
) {
    for (interaction, item) in &query {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some(cv) = view.0.as_ref() else { continue };
        let Some(k) = known_card(cv, item.card_id) else { continue };
        if option_needs_target(item.option, &k) {
            targeting.active = true;
            targeting.pending_card_id = Some(item.card_id);
            targeting.back_face_pending = item.option == HandPlayOption::CastBack;
            targeting.pending_cast_variant = match item.option {
                HandPlayOption::Variant(v) => Some(v),
                _ => None,
            };
            targeting.pending_reinforce = item.option == HandPlayOption::Reinforce;
        } else if let Some(ob) = &outbox {
            ob.submit(untargeted_action(item.card_id, item.option));
        }
        menu_state.card_id = None;
    }
}

/// The action for an option that needs no target.
fn untargeted_action(card_id: CardId, option: HandPlayOption) -> GameAction {
    match option {
        HandPlayOption::Foretell => GameAction::Foretell { card_id },
        HandPlayOption::Plot => GameAction::Plot { card_id },
        HandPlayOption::Suspend => GameAction::Suspend { card_id },
        HandPlayOption::Morph => GameAction::CastFaceDown { card_id },
        HandPlayOption::DiscardAbility => GameAction::ActivateDiscardAbility { card_id },
        HandPlayOption::RoomDoor { right } => GameAction::CastRoomDoor { card_id, right },
        HandPlayOption::CastBack => GameAction::CastSpellBack {
            card_id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
        HandPlayOption::Variant(v) => v.action(card_id, None),
        // Reinforce always takes a target, so it never reaches here.
        HandPlayOption::Cast | HandPlayOption::Reinforce => GameAction::CastSpell {
            card_id,
            target: None,
            additional_targets: vec![],
            mode: None,
            x_value: None,
        },
    }
}

fn known_card(cv: &ClientView, card_id: CardId) -> Option<KnownCard> {
    cv.players.get(cv.your_seat)?.hand.iter().find_map(|h| match h {
        crabomination::net::HandCardView::Known(k) if k.id == card_id => Some(k.clone()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_view() -> ClientView {
        ClientView { your_seat: 0, ..Default::default() }
    }

    #[test]
    fn options_follow_the_engines_affordance_sets() {
        // The menu must offer exactly what the engine published — these are
        // the mechanics that previously had no client path at all.
        let id = CardId(7);
        let mut cv = empty_view();
        cv.foretellable_hand = vec![id];
        cv.plottable_hand = vec![id];
        cv.suspendable_hand = vec![id];
        cv.morphable_hand = vec![id];
        cv.bestowable_hand = vec![id];
        cv.reinforceable_hand = vec![id];
        cv.room_castable_hand = vec![(id, 0b11)];
        let opts = hand_play_options(&cv, id);
        assert!(opts.contains(&HandPlayOption::Foretell));
        assert!(opts.contains(&HandPlayOption::Plot));
        assert!(opts.contains(&HandPlayOption::Suspend));
        assert!(opts.contains(&HandPlayOption::Morph));
        assert!(opts.contains(&HandPlayOption::Reinforce));
        assert!(opts.contains(&HandPlayOption::Variant(HandCastVariant::Bestow)));
        assert!(opts.contains(&HandPlayOption::RoomDoor { right: false }));
        assert!(opts.contains(&HandPlayOption::RoomDoor { right: true }));
        // A card the engine offers nothing for gets no menu.
        assert!(hand_play_options(&cv, CardId(99)).is_empty());
    }

    #[test]
    fn untargeted_options_map_to_their_actions() {
        let id = CardId(3);
        assert!(matches!(
            untargeted_action(id, HandPlayOption::Foretell),
            GameAction::Foretell { card_id } if card_id == id
        ));
        assert!(matches!(
            untargeted_action(id, HandPlayOption::Plot),
            GameAction::Plot { card_id } if card_id == id
        ));
        assert!(matches!(
            untargeted_action(id, HandPlayOption::Suspend),
            GameAction::Suspend { card_id } if card_id == id
        ));
        assert!(matches!(
            untargeted_action(id, HandPlayOption::Morph),
            GameAction::CastFaceDown { card_id } if card_id == id
        ));
        assert!(matches!(
            untargeted_action(id, HandPlayOption::RoomDoor { right: true }),
            GameAction::CastRoomDoor { right: true, .. }
        ));
        assert!(matches!(
            untargeted_action(id, HandPlayOption::DiscardAbility),
            GameAction::ActivateDiscardAbility { card_id } if card_id == id
        ));
    }
}
