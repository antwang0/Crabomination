//! One modal for every decision that puts cards in an order: scry, surveil
//! and rearranging the top of a library (with a second pile where there is
//! one), the push order of simultaneous triggers (CR 603.3b), and the order
//! a source deals its combat damage (CR 510.1c). Each was its own copy of
//! the same row of tiles, with its own ← / → button and handler over its own
//! list. One [`spawn_order_modal`] lays out any of them from [`OrderTile`]s,
//! and one [`OrderMove`] button and handler moves a card in whichever list
//! its decision keeps.
//!
//! A change — a move, or a scry card sent to its second pile — respawns the
//! modal from the working order, so a tile's caption always comes from the
//! decision: toggling a surveilled card used to relabel it "Bottom" in place
//! rather than "Graveyard".

use super::*;

/// The working order a decision keeps in `DecisionUiState`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrderList {
    /// `scry`: the cards and which pile each is in.
    Scry,
    /// `trigger_order`: index 0 is pushed first, so the last resolves first.
    Triggers,
    /// `damage_order`: index 0 is dealt damage first.
    DamageOrder,
}

/// ← / →: move `card` `delta` places in `list`.
#[derive(Component)]
pub struct OrderMove {
    pub list: OrderList,
    pub card: CardId,
    pub delta: i32,
}

/// One tile of an ordered modal.
pub struct OrderTile {
    pub id: CardId,
    pub name: String,
    /// Under the art: its place ("1.") or its pile ("Top", "Graveyard").
    pub caption: String,
    /// `Some(in the second pile)` when a click sends the card to the other
    /// pile (scry's bottom, surveil's graveyard).
    pub second_pile: Option<bool>,
}

/// The ordered modal: `prompt`, then the tiles in order, each with ← / →,
/// then Confirm.
pub(super) fn spawn_order_modal(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    prompt: &str,
    list: OrderList,
    tiles: &[OrderTile],
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
            GlobalZIndex(theme::layer::MODAL),
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
            // A hovered card's preview sits beside the panel, not over the
            // next card in the row.
            crate::systems::ui_card_hover::PreviewAnchor,
        ))
        .id();
    commands.entity(root).add_child(panel);

    let last = tiles.len().saturating_sub(1);
    commands.entity(panel).with_children(|panel| {
        panel.spawn((Text::new(prompt), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY)));
        panel.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), ..default() }).with_children(|row| {
            for (i, tile) in tiles.iter().enumerate() {
                let texture: Handle<Image> = asset_server.load(scryfall::card_asset_path(&tile.name));
                row.spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|col| {
                    let mut card = col.spawn((
                        Button,
                        crate::systems::ui_card_hover::UiCardHover::card(&tile.name, Some(tile.id)),
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(CARD_W),
                            padding: UiRect::all(Val::Px(6.0)),
                            row_gap: Val::Px(4.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if tile.second_pile == Some(true) { theme::BUTTON_SELECTED_BG } else { MODAL_TILE_BG }),
                    ));
                    if tile.second_pile.is_some() {
                        card.insert(ScryToggleButton { card_id: tile.id });
                    }
                    card.with_children(|c| {
                        c.spawn((
                            ImageNode { image: texture, ..default() },
                            Node { width: Val::Px(CARD_W - 12.0), height: Val::Px(CARD_H - 12.0), ..default() },
                            Pickable::IGNORE,
                        ));
                        c.spawn((Text::new(tile.caption.clone()), ui_fonts.tf(14.0), TextColor(theme::TEXT_PRIMARY), Pickable::IGNORE));
                    });
                    col.spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(8.0), ..default() }).with_children(|r| {
                        for (label, delta, disabled) in [("←", -1, i == 0), ("→", 1, i == last)] {
                            r.spawn((
                                Button,
                                Node { padding: UiRect::axes(Val::Px(14.0), Val::Px(6.0)), ..default() },
                                BackgroundColor(if disabled { REORDER_BG_DISABLED } else { REORDER_BG }),
                                OrderMove { list, card: tile.id, delta },
                            ))
                            .with_children(|b| {
                                b.spawn((
                                    Text::new(label),
                                    ui_fonts.tf(16.0),
                                    TextColor(if disabled { theme::TEXT_MUTED } else { theme::TEXT_PRIMARY }),
                                    Pickable::IGNORE,
                                ));
                            });
                        }
                    });
                });
            }
        });
        spawn_confirm_button(panel, ui_fonts);
    });
}

/// Move the item `is` picks `delta` places along `order`, clamped to its
/// ends; whether it moved.
fn shift<T>(order: &mut [T], is: impl Fn(&T) -> bool, delta: i32) -> bool {
    let Some(at) = order.iter().position(is) else { return false };
    let to = (at as i32 + delta).clamp(0, order.len() as i32 - 1) as usize;
    order.swap(at, to);
    at != to
}

/// Respawn the open modal from the working order.
fn respawn(commands: &mut Commands, state: &mut DecisionUiState, modal: &Query<Entity, With<DecisionModal>>) {
    for e in modal {
        commands.entity(e).despawn();
    }
    state.spawned_for = None;
}

/// ← / → on any ordered modal.
pub fn handle_order_moves(
    mut commands: Commands,
    mut state: ResMut<DecisionUiState>,
    moves: Query<(&Interaction, &OrderMove), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
) {
    for (interaction, mv) in &moves {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let moved = match mv.list {
            OrderList::Scry => shift(&mut state.scry, |(id, _)| *id == mv.card, mv.delta),
            OrderList::Triggers => shift(&mut state.trigger_order, |id| *id == mv.card, mv.delta),
            OrderList::DamageOrder => shift(&mut state.damage_order, |id| *id == mv.card, mv.delta),
        };
        if moved {
            respawn(&mut commands, &mut state, &modal);
        }
    }
}

/// A click on a scry tile sends its card to the other pile.
pub fn handle_scry_toggles(
    mut commands: Commands,
    mut state: ResMut<DecisionUiState>,
    toggles: Query<(&Interaction, &ScryToggleButton), Changed<Interaction>>,
    modal: Query<Entity, With<DecisionModal>>,
) {
    for (interaction, toggle) in &toggles {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if let Some(entry) = state.scry.iter_mut().find(|(id, _)| *id == toggle.card_id) {
            entry.1 = !entry.1;
            respawn(&mut commands, &mut state, &modal);
        }
    }
}

/// The scry modal's prompt and tiles: `Scry` sends cards to the bottom,
/// `Surveil` to the graveyard, `Rearrange` only reorders.
pub(super) fn scry_tiles(
    order: &[(CardId, bool)],
    names: &std::collections::HashMap<CardId, String>,
    mode: crabomination::decision::ScryMode,
) -> (String, Vec<OrderTile>) {
    use crabomination::decision::ScryMode;
    let (verb, second): (&str, Option<&str>) = match mode {
        ScryMode::Scry => ("Scry", Some("Bottom")),
        ScryMode::Surveil => ("Surveil", Some("Graveyard")),
        ScryMode::Rearrange => ("Rearrange top", None),
    };
    let n = order.len();
    let prompt = match second {
        Some(pile) => format!("{verb} {n}: click card to toggle {pile}  ·  ← → to reorder  ·  left = top of library"),
        None => format!("{verb} {n}: ← → to reorder  ·  left = top of library"),
    };
    let tiles = order
        .iter()
        .map(|(id, in_second)| OrderTile {
            id: *id,
            name: names.get(id).cloned().unwrap_or_default(),
            caption: match second {
                Some(pile) if *in_second => pile.to_string(),
                _ => "Top".to_string(),
            },
            second_pile: second.map(|_| *in_second),
        })
        .collect();
    (prompt, tiles)
}

/// Tiles for a plain ordering, captioned by place.
pub(super) fn numbered_tiles(order: &[CardId], name_of: impl Fn(CardId) -> String) -> Vec<OrderTile> {
    order
        .iter()
        .enumerate()
        .map(|(i, id)| OrderTile { id: *id, name: name_of(*id), caption: format!("{}.", i + 1), second_pile: None })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::decision::ScryMode;

    /// A move swaps neighbours and stops at the ends.
    #[test]
    fn a_move_stops_at_the_ends() {
        let mut order = vec![1, 2, 3];
        assert!(shift(&mut order, |x| *x == 1, 1));
        assert_eq!(order, [2, 1, 3]);
        assert!(!shift(&mut order, |x| *x == 3, 1), "already last");
        assert!(!shift(&mut order, |x| *x == 9, -1), "not in the order");
        assert_eq!(order, [2, 1, 3]);
    }

    /// A surveilled card sent to its second pile reads "Graveyard", a
    /// scried one "Bottom"; rearranging has no second pile to send it to.
    #[test]
    fn the_second_pile_is_named_by_the_decision() {
        let (a, b) = (CardId(1), CardId(2));
        let names = [(a, "Opt".to_string()), (b, "Ponder".to_string())].into_iter().collect();
        let order = [(a, true), (b, false)];
        let (_, surveil) = scry_tiles(&order, &names, ScryMode::Surveil);
        assert_eq!((surveil[0].caption.as_str(), surveil[1].caption.as_str()), ("Graveyard", "Top"));
        assert_eq!(surveil[0].second_pile, Some(true));
        let (_, scry) = scry_tiles(&order, &names, ScryMode::Scry);
        assert_eq!(scry[0].caption, "Bottom");
        let (prompt, rearrange) = scry_tiles(&order, &names, ScryMode::Rearrange);
        assert!(rearrange.iter().all(|t| t.second_pile.is_none() && t.caption == "Top"));
        assert!(prompt.starts_with("Rearrange top 2"), "{prompt}");
    }
}
