//! One picker for every decision that asks for cards: a library search, the
//! cards to put on the bottom after a mulligan, a discard, and the engine's
//! generic "choose cards" (a sacrifice, an exile-from-graveyard cost, a
//! reveal that takes some of what it shows). Each was its own modal with its
//! own selection state, click handler and count rule; a [`CardPick`] now
//! describes any of them — the candidates, which may be picked, how many —
//! and one set of systems runs it.
//!
//! **Where you pick follows where the cards are.** Cards in your own hand are
//! picked in the 3-D hand, under a banner — bottoming after a mulligan, a
//! discard, a "discard a card" cost — as the mulligan itself is decided over
//! the hand on the table. Cards anywhere else (your library, an opponent's
//! hand, a graveyard) are picked in a grid of tiles. A discard used to show
//! your own hand a second time, in a grid, while the cards put back after a
//! mulligan were clicked in the 3-D hand: the same act, two ways.

use super::*;
use crabomination::net::{ClientView, HandCardView};

/// A pick-cards decision, whatever asked for it.
#[derive(Clone, Debug, PartialEq)]
pub struct CardPick {
    /// What the banner or grid says above the cards.
    pub prompt: String,
    pub candidates: Vec<(CardId, String)>,
    /// `Some`: only these may be picked; the rest still show, dimmed (a
    /// reveal that takes only some of what it shows).
    pub eligible: Option<Vec<CardId>>,
    pub min: usize,
    pub max: usize,
    /// Every candidate is a card in the viewer's own hand, so it is picked
    /// there, in the 3-D hand.
    pub in_hand: bool,
    /// What Confirm says with cards picked.
    pub verb: &'static str,
    /// What it says with none picked, where none is an answer.
    pub none: &'static str,
}

fn cards(n: usize) -> String {
    if n == 1 { "1 card".to_string() } else { format!("{n} cards") }
}

impl CardPick {
    /// Whether `id` may be picked.
    pub fn can_pick(&self, id: CardId) -> bool {
        self.candidates.iter().any(|(c, _)| *c == id) && self.eligible.as_ref().is_none_or(|e| e.contains(&id))
    }

    /// Whether `picked` cards answer it.
    pub fn ready(&self, picked: usize) -> bool {
        (self.min..=self.max).contains(&picked)
    }

    /// The running count under the prompt.
    pub fn count_line(&self, picked: usize) -> String {
        match (self.min, self.max) {
            (min, max) if min == max => format!("{picked} / {max} chosen"),
            (0, max) => format!("{picked} chosen (up to {max})"),
            (min, max) => format!("{picked} chosen ({min} to {max})"),
        }
    }

    /// The Confirm button's label with `picked` cards picked.
    pub fn confirm_label(&self, picked: usize) -> &'static str {
        if picked == 0 && self.min == 0 { self.none } else { self.verb }
    }
}

/// The card pick `wire` asks for, or `None` when it asks for something else.
pub fn card_pick(cv: &ClientView, wire: &DecisionWire) -> Option<CardPick> {
    let hand: Vec<CardId> = cv
        .players
        .iter()
        .find(|p| p.seat == cv.your_seat)
        .map(|p| p.hand.iter().filter(|h| matches!(h, HandCardView::Known(_))).map(HandCardView::id).collect())
        .unwrap_or_default();
    let in_hand = |cards: &[(CardId, String)]| !cards.is_empty() && cards.iter().all(|(id, _)| hand.contains(id));
    Some(match wire {
        DecisionWire::SearchLibrary { candidates, eligible, .. } => CardPick {
            prompt: format!("Search your library: choose a card ({})", cards(candidates.len())),
            candidates: candidates.clone(),
            eligible: eligible.clone(),
            min: 0,
            max: 1,
            in_hand: false,
            verb: "Take",
            none: "Find nothing",
        },
        DecisionWire::PutOnLibrary { count, hand, .. } => CardPick {
            prompt: format!("Put {} from your hand on the bottom of your library", cards(*count)),
            candidates: hand.clone(),
            eligible: None,
            min: *count,
            max: *count,
            in_hand: in_hand(hand),
            verb: "Put on bottom",
            none: "Put on bottom",
        },
        DecisionWire::Discard { count, hand, .. } => {
            // A hand smaller than the count discards what it has.
            let n = (*count as usize).min(hand.len());
            let own = in_hand(hand);
            CardPick {
                prompt: if own { format!("Discard {}", cards(n)) } else { format!("Choose {} to discard", cards(n)) },
                candidates: hand.clone(),
                eligible: None,
                min: n,
                max: n,
                in_hand: own,
                verb: "Discard",
                none: "Discard",
            }
        }
        DecisionWire::ChooseCards { prompt, candidates, eligible, min, max, .. } => CardPick {
            prompt: prompt.clone(),
            candidates: candidates.clone(),
            eligible: eligible.clone(),
            min: *min as usize,
            max: *max as usize,
            in_hand: in_hand(candidates),
            verb: "Choose",
            none: "Choose none",
        },
        _ => return None,
    })
}

/// The answer `picked` gives the card pick `wire` asks for.
pub fn pick_answer(wire: &DecisionWire, picked: &[CardId]) -> Option<DecisionAnswer> {
    Some(match wire {
        DecisionWire::SearchLibrary { .. } => DecisionAnswer::Search(picked.first().copied()),
        DecisionWire::PutOnLibrary { .. } => DecisionAnswer::PutOnLibrary(picked.to_vec()),
        DecisionWire::Discard { .. } => DecisionAnswer::Discard(picked.to_vec()),
        DecisionWire::ChooseCards { .. } => DecisionAnswer::Cards(picked.to_vec()),
        _ => return None,
    })
}

/// A click on `id`: unpick it if picked; else pick it — in place of the one
/// picked when only one may be, or while there's room under `max`.
pub fn toggle(picked: &mut Vec<CardId>, id: CardId, max: usize) {
    if let Some(at) = picked.iter().position(|p| *p == id) {
        picked.remove(at);
    } else if max == 1 {
        *picked = vec![id];
    } else if picked.len() < max {
        picked.push(id);
    }
}

/// The viewer's pending card pick, with the decision that asked for it.
fn pending_pick(cv: &ClientView) -> Option<(CardPick, &DecisionWire)> {
    let pd = cv.pending_decision.as_ref().filter(|pd| pd.acting_player == cv.your_seat)?;
    let wire = pd.decision.as_ref()?;
    Some((card_pick(cv, wire)?, wire))
}

/// A card tile in the pick grid.
#[derive(Component)]
pub struct CardPickTile {
    pub card_id: CardId,
}

/// The running "N / M chosen" line.
#[derive(Component)]
pub struct CardPickCount;

/// The Confirm button's label, which says what confirming does.
#[derive(Component)]
pub struct CardPickConfirmLabel;

/// The border drawn around a picked card in the 3-D hand: its two meshes,
/// kept apart from the hover highlight's `CardBorderHighlight`.
#[derive(Component)]
pub struct PickHighlight {
    back: Entity,
    front: Entity,
}

/// Put up `pick`'s surface: a banner over the 3-D hand, or a grid of tiles.
pub(super) fn spawn_card_pick(
    commands: &mut Commands,
    asset_server: &AssetServer,
    ui_fonts: &UiFonts,
    pick: &CardPick,
    cancellable: bool,
) {
    if pick.in_hand {
        spawn_hand_banner(commands, ui_fonts, pick, cancellable);
    } else {
        spawn_grid(commands, asset_server, ui_fonts, pick, cancellable);
    }
}

/// The count and the Confirm / Cancel row under a pick.
fn spawn_footer(panel: &mut ChildSpawnerCommands, ui_fonts: &UiFonts, pick: &CardPick, cancellable: bool) {
    panel.spawn((Text::new(pick.count_line(0)), ui_fonts.tf(14.0), TextColor(theme::ACCENT_GOLD), CardPickCount));
    panel
        .spawn(Node { flex_direction: FlexDirection::Row, column_gap: Val::Px(12.0), align_items: AlignItems::Center, ..default() })
        .with_children(|row| {
            row.spawn((
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
                let ready = pick.ready(0);
                b.spawn((
                    Text::new(pick.confirm_label(0)),
                    ui_fonts.tf(16.0),
                    TextColor(if ready { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED }),
                    Pickable::IGNORE,
                    CardPickConfirmLabel,
                ));
            });
            if cancellable {
                spawn_cancel_button(row, ui_fonts);
            }
        });
}

/// The banner over the 3-D hand: the cards are clicked where they are.
fn spawn_hand_banner(commands: &mut Commands, ui_fonts: &UiFonts, pick: &CardPick, cancellable: bool) {
    // Pass-through: the hand stays clickable around the panel.
    let panel = spawn_modal_panel(commands, 320.0);
    commands.entity(panel).with_children(|p| {
        p.spawn((Text::new(pick.prompt.clone()), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY)));
        p.spawn((Text::new("Click the cards in your hand."), ui_fonts.tf(13.0), TextColor(theme::TEXT_SECONDARY)));
        spawn_footer(p, ui_fonts, pick, cancellable);
    });
}

/// More candidates than this, and the tiles shrink so a library's worth
/// fits; the grid scrolls past that.
const COMPACT_PAST: usize = 10;
const COMPACT_W: f32 = 110.0;

/// The grid of tiles for cards outside the viewer's hand.
fn spawn_grid(commands: &mut Commands, asset_server: &AssetServer, ui_fonts: &UiFonts, pick: &CardPick, cancellable: bool) {
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
                padding: UiRect::all(Val::Px(20.0)),
                row_gap: Val::Px(12.0),
                align_items: AlignItems::Center,
                max_width: Val::Percent(90.0),
                max_height: Val::Percent(90.0),
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

    let (tile_w, text) = if pick.candidates.len() > COMPACT_PAST { (COMPACT_W, 10.0) } else { (CARD_W, 12.0) };
    commands.entity(panel).with_children(|panel| {
        panel.spawn((Text::new(pick.prompt.clone()), ui_fonts.tf(16.0), TextColor(theme::TEXT_PRIMARY)));
        // Scrollable: a bounded height, `Overflow::scroll_y` and the
        // `Scrollable` marker that drives `ScrollPosition`. Without
        // `min_height: 0` flexbox's default `min-height: auto` outranks
        // `max_height` and the grid grows to fit anyway.
        panel
            .spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(10.0),
                    row_gap: Val::Px(10.0),
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::FlexStart,
                    max_height: Val::Vh(70.0),
                    min_height: Val::Px(0.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                Pickable::default(),
                crate::systems::scroll::Scrollable::default(),
            ))
            .with_children(|grid| {
                for (card_id, name) in &pick.candidates {
                    let legal = pick.can_pick(*card_id);
                    let texture: Handle<Image> = asset_server.load(scryfall::card_asset_path(name));
                    // Every tile previews (an unpickable one shows why it
                    // isn't); only a pickable one carries the tile marker, so
                    // a click on another does nothing at all.
                    let mut tile = grid.spawn((
                        Button,
                        crate::systems::ui_card_hover::UiCardHover::card(name, Some(*card_id)),
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Px(tile_w),
                            padding: UiRect::all(Val::Px(5.0)),
                            row_gap: Val::Px(3.0),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(if legal { MODAL_TILE_BG } else { theme::PANEL_BG_SUNKEN }),
                    ));
                    if legal {
                        tile.insert(CardPickTile { card_id: *card_id });
                    }
                    tile.with_children(|t| {
                        t.spawn((
                            ImageNode {
                                image: texture,
                                color: if legal { Color::WHITE } else { Color::srgba(0.45, 0.45, 0.5, 0.55) },
                                ..default()
                            },
                            Node {
                                width: Val::Px(tile_w - 10.0),
                                height: Val::Px((tile_w - 10.0) * CARD_ASPECT_RATIO),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ));
                        t.spawn((
                            Text::new(if legal { name.clone() } else { format!("{name}  (can't choose)") }),
                            ui_fonts.tf(text),
                            TextColor(if legal { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED }),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });
        spawn_footer(panel, ui_fonts, pick, cancellable);
    });
}

/// A click on a grid tile picks or unpicks its card.
pub fn handle_card_pick_tiles(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    tiles: Query<(&Interaction, &CardPickTile), Changed<Interaction>>,
) {
    let Some(cv) = &view.0 else { return };
    let Some((pick, _)) = pending_pick(cv) else { return };
    for (interaction, tile) in &tiles {
        if *interaction == Interaction::Pressed && pick.can_pick(tile.card_id) {
            toggle(&mut state.picked, tile.card_id, pick.max);
        }
    }
}

/// A click on a card in the 3-D hand picks or unpicks it, while a pick of
/// the viewer's own hand cards is up. (`handle_game_input` stands aside
/// while any decision is pending, so the click doesn't play the card.)
pub fn handle_card_pick_hand_click(
    view: Res<CurrentView>,
    mut state: ResMut<DecisionUiState>,
    mouse: Res<ButtonInput<MouseButton>>,
    hovered_hand: Query<&crate::card::GameCardId, (With<crate::card::CardHovered>, With<crate::card::HandCard>)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }
    let Some(cv) = &view.0 else { return };
    let Some((pick, _)) = pending_pick(cv).filter(|(pick, _)| pick.in_hand) else { return };
    let Some(id) = hovered_hand.iter().next().map(|g| g.0) else { return };
    if pick.can_pick(id) {
        toggle(&mut state.picked, id, pick.max);
    }
}

/// Keep the pick's readout with the picks: each tile's colour, the count,
/// and the Confirm label (greyed until the pick is a full answer).
#[allow(clippy::type_complexity)]
pub fn update_card_pick_readout(
    view: Res<CurrentView>,
    state: Res<DecisionUiState>,
    mut tiles: Query<(&CardPickTile, &mut BackgroundColor)>,
    mut count: Query<&mut Text, (With<CardPickCount>, Without<CardPickConfirmLabel>)>,
    mut confirm: Query<(&mut Text, &mut TextColor), With<CardPickConfirmLabel>>,
) {
    if !state.is_changed() && !view.is_changed() {
        return;
    }
    let Some(cv) = &view.0 else { return };
    let Some((pick, _)) = pending_pick(cv) else { return };
    let n = state.picked.len();
    for (tile, mut bg) in &mut tiles {
        let want = if state.picked.contains(&tile.card_id) { theme::BUTTON_SELECTED_BG } else { MODAL_TILE_BG };
        if bg.0 != want {
            bg.0 = want;
        }
    }
    for mut text in &mut count {
        text.0 = pick.count_line(n);
    }
    for (mut text, mut color) in &mut confirm {
        text.0 = pick.confirm_label(n).to_string();
        color.0 = if pick.ready(n) { theme::TEXT_PRIMARY } else { theme::TEXT_MUTED };
    }
}

/// Ring each picked card in the 3-D hand with the gold border, while a pick
/// of the viewer's own hand cards is up; take the rings off after.
#[allow(clippy::type_complexity)]
pub fn update_card_pick_highlights(
    mut commands: Commands,
    state: Res<DecisionUiState>,
    view: Res<CurrentView>,
    highlight_assets: Option<Res<crate::card::CardHighlightAssets>>,
    hand_cards: Query<(Entity, &crate::card::GameCardId, Option<&PickHighlight>), With<crate::card::HandCard>>,
) {
    let Some(assets) = highlight_assets else { return };
    let in_hand = view.0.as_ref().and_then(pending_pick).is_some_and(|(pick, _)| pick.in_hand);
    for (entity, gid, marker) in &hand_cards {
        match (in_hand && state.picked.contains(&gid.0), marker) {
            (true, None) => {
                let offset = crate::card::CARD_THICKNESS / 2.0 + 0.0015;
                let ring = |commands: &mut Commands, z: f32| {
                    commands
                        .spawn((
                            Mesh3d(assets.border_mesh.clone()),
                            MeshMaterial3d(assets.border_material.clone()),
                            Transform::from_xyz(0.0, 0.0, z),
                            Pickable::IGNORE,
                        ))
                        .id()
                };
                let (back, front) = (ring(&mut commands, -offset), ring(&mut commands, offset));
                commands.entity(entity).insert(PickHighlight { back, front }).add_children(&[back, front]);
            }
            (false, Some(ring)) => {
                commands.entity(ring.back).despawn();
                commands.entity(ring.front).despawn();
                commands.entity(entity).remove::<PickHighlight>();
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crabomination::catalog;

    fn named(ids: &[CardId]) -> Vec<(CardId, String)> {
        ids.iter().map(|id| (*id, format!("Card {}", id.0))).collect()
    }

    /// A view with three cards in the viewer's hand and two in seat 1's.
    fn table() -> (ClientView, Vec<CardId>, Vec<CardId>) {
        let mut g = crabomination::game::two_player_game();
        let mine: Vec<CardId> = (0..3).map(|_| g.add_card_to_hand(0, catalog::grizzly_bears())).collect();
        let theirs: Vec<CardId> = (0..2).map(|_| g.add_card_to_hand(1, catalog::grizzly_bears())).collect();
        (crabomination::server::view::project(&g, 0), mine, theirs)
    }

    /// Your own hand is picked in the 3-D hand, an opponent's in a grid —
    /// the same Discard decision either way.
    #[test]
    fn a_discard_is_picked_where_the_cards_are() {
        let (cv, mine, theirs) = table();
        let own = card_pick(&cv, &DecisionWire::Discard { player: 0, count: 2, hand: named(&mine) }).unwrap();
        assert!(own.in_hand);
        assert_eq!((own.prompt.as_str(), own.min, own.max), ("Discard 2 cards", 2, 2));
        let other = card_pick(&cv, &DecisionWire::Discard { player: 1, count: 1, hand: named(&theirs) }).unwrap();
        assert!(!other.in_hand);
        assert_eq!(other.prompt, "Choose 1 card to discard");
        // Cards from anywhere but the hand — a graveyard — are a grid too.
        let gy = DecisionWire::ChooseCards {
            source: mine[0],
            prompt: "Exile a card".into(),
            candidates: named(&[CardId(900)]),
            eligible: None,
            min: 1,
            max: 1,
        };
        assert!(!card_pick(&cv, &gy).unwrap().in_hand);
        // A hand smaller than the count discards what it has.
        let short = card_pick(&cv, &DecisionWire::Discard { player: 0, count: 5, hand: named(&mine) }).unwrap();
        assert_eq!((short.min, short.max), (3, 3));
    }

    /// A search may find nothing, and Confirm says so until a card is picked;
    /// an ineligible candidate can't be picked.
    #[test]
    fn a_search_may_find_nothing() {
        let (cv, ..) = table();
        let ids = [CardId(901), CardId(902)];
        let search = DecisionWire::SearchLibrary { player: 0, candidates: named(&ids), eligible: Some(vec![ids[1]]) };
        let pick = card_pick(&cv, &search).unwrap();
        assert!(pick.ready(0) && pick.ready(1) && !pick.ready(2));
        assert_eq!((pick.confirm_label(0), pick.confirm_label(1)), ("Find nothing", "Take"));
        assert_eq!(pick.count_line(0), "0 chosen (up to 1)");
        assert!(!pick.can_pick(ids[0]) && pick.can_pick(ids[1]));
        assert_eq!(pick_answer(&search, &[]), Some(DecisionAnswer::Search(None)));
        assert_eq!(pick_answer(&search, &[ids[1]]), Some(DecisionAnswer::Search(Some(ids[1]))));
    }

    /// Picking only one swaps the pick; picking several stops at the cap;
    /// a second click unpicks.
    #[test]
    fn toggling_picks() {
        let (a, b, c) = (CardId(1), CardId(2), CardId(3));
        let mut one = vec![];
        toggle(&mut one, a, 1);
        toggle(&mut one, b, 1);
        assert_eq!(one, [b]);
        let mut two = vec![];
        for id in [a, b, c] {
            toggle(&mut two, id, 2);
        }
        assert_eq!(two, [a, b]);
        toggle(&mut two, a, 2);
        assert_eq!(two, [b]);
    }

    /// Each decision gets its own answer from the same picks.
    #[test]
    fn the_answer_follows_the_decision() {
        let picks = [CardId(1), CardId(2)];
        let hand = named(&picks);
        let put = DecisionWire::PutOnLibrary { player: 0, count: 2, hand: hand.clone() };
        assert_eq!(pick_answer(&put, &picks), Some(DecisionAnswer::PutOnLibrary(picks.to_vec())));
        let discard = DecisionWire::Discard { player: 0, count: 2, hand };
        assert_eq!(pick_answer(&discard, &picks), Some(DecisionAnswer::Discard(picks.to_vec())));
        assert_eq!(pick_answer(&DecisionWire::CoinFlip { player: 0 }, &picks), None);
    }
}
