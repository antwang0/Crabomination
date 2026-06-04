use bevy::prelude::*;

use crate::card::{
    BattlefieldCard, Card, CardBorderHighlight, CardFrontTexture, CardHighlightAssets, CardHovered,
    CastableHighlight, DeckCard, DeckPile, DyingHighlight, GameCardId, GraveyardPile, HandCard,
    PileHovered, CARD_THICKNESS,
};
use crate::game::GraveyardBrowserState;
use crate::net_plugin::CurrentView;
use crate::theme::{self, UiFonts};

/// Tracks a pending top-card reveal popup.
#[derive(Resource, Default)]
pub struct RevealPopupState {
    /// Asset path of the card to show, or None when no popup is active.
    pub card_path: Option<String>,
    /// Which player's top card was revealed (sets the animation target).
    pub revealed_player: Option<usize>,
}
use crate::scryfall;

#[derive(Component)]
pub struct PeekPopup;

#[derive(Component)]
pub struct GraveyardBrowser;

/// One card tile inside the graveyard browser. Stores the card's
/// display name so the hover-name-tooltip system can render it.
#[derive(Component)]
pub struct GraveyardCardItem {
    pub name: String,
}

/// Marker for the live name tooltip rendered above the graveyard
/// browser whenever the user hovers a card tile.
#[derive(Component)]
pub struct GraveyardCardNameTooltip;

#[derive(Component)]
pub struct RevealPopup;

#[derive(Component)]
pub struct PileTooltip {
    /// The message currently rendered. Tracked so `pile_tooltip` can
    /// detect when the hovered pile changes and refresh the text instead
    /// of leaving a stale tooltip pinned (e.g. dragging the cursor from
    /// one player's deck to the other's graveyard).
    pub msg: String,
}

pub fn highlight_hovered_cards(
    mut commands: Commands,
    added: Query<Entity, (With<Card>, Added<CardHovered>)>,
    mut removed: RemovedComponents<CardHovered>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    borders: Query<&CardBorderHighlight>,
) {
    let Some(assets) = highlight_assets else {
        return;
    };

    for card_entity in &added {
        let offset = CARD_THICKNESS / 2.0 + 0.001;
        let back_border = commands
            .spawn((
                Mesh3d(assets.border_mesh.clone()),
                MeshMaterial3d(assets.border_material.clone()),
                Transform::from_xyz(0.0, 0.0, -offset),
                Pickable::IGNORE,
            ))
            .id();
        let front_border = commands
            .spawn((
                Mesh3d(assets.border_mesh.clone()),
                MeshMaterial3d(assets.border_material.clone()),
                Transform::from_xyz(0.0, 0.0, offset),
                Pickable::IGNORE,
            ))
            .id();
        commands
            .entity(card_entity)
            .insert(CardBorderHighlight(back_border, front_border))
            .add_children(&[back_border, front_border]);
    }

    for card_entity in removed.read() {
        if let Ok(border) = borders.get(card_entity) {
            commands.entity(border.0).despawn();
            commands.entity(border.1).despawn();
            commands.entity(card_entity).remove::<CardBorderHighlight>();
        }
    }
}

/// Spawn / despawn a "playable now" border around each viewer hand card,
/// in one of two colours: **green** for cards castable for their normal
/// cost (`castable_hand`), **cyan** for cards playable only via an
/// alternative path — Dash, Blitz, pitch/exile, kicker, or Suspend
/// (`dashable_hand` / `blitzable_hand` / `pitchable_hand` / `kickable_hand` /
/// `suspendable_hand`, minus anything already hard-castable). All are computed server-side via the engine's
/// `would_accept` dry-run, so they already reflect timing, mana, taxes,
/// and target availability. Mirrors the hover-border / put-on-library
/// highlight pattern.
///
/// Suppressed while the targeting cursor is active — there the gold
/// valid-target borders own the hand visuals and a second border set would
/// only stack and z-fight. The sets are also naturally empty when the
/// viewer lacks priority (you can't cast then anyway).
#[allow(clippy::type_complexity)]
pub fn update_castable_highlights(
    mut commands: Commands,
    view: Res<CurrentView>,
    targeting: Res<crate::game::TargetingState>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    hand_cards: Query<(Entity, &GameCardId, Option<&CastableHighlight>), With<HandCard>>,
    // A card played/discarded mid-highlight keeps its entity (HandCard is
    // *removed*, not despawned — see `sync_game_visuals`), so its green
    // border would otherwise linger on the battlefield/graveyard. Strip the
    // highlight from anything that left the hand still wearing it.
    left_hand: Query<(Entity, &CastableHighlight), Without<HandCard>>,
) {
    let Some(assets) = highlight_assets else { return };

    for (entity, h) in &left_hand {
        commands.entity(h.back).despawn();
        commands.entity(h.front).despawn();
        commands.entity(entity).remove::<CastableHighlight>();
    }

    // `hard` = castable for the normal cost (green). `alt` = playable only
    // via an alternative path — Dash (CR 702.110), an exile-to-pitch
    // ability (Force of Will / Spirit Guides), or kicker (CR 702.32) — that
    // ISN'T already hard-castable (cyan). A card hard-castable *and*
    // kickable stays green: you can just cast it, and kicker is an opt-in
    // rather than an alternative path.
    let (hard, alt): (
        std::collections::HashSet<crabomination::card::CardId>,
        std::collections::HashSet<crabomination::card::CardId>,
    ) = if targeting.active {
        (Default::default(), Default::default())
    } else if let Some(cv) = view.0.as_ref() {
        let hard: std::collections::HashSet<crabomination::card::CardId> =
            cv.castable_hand.iter().copied().collect();
        let alt = cv
            .dashable_hand
            .iter()
            .chain(cv.blitzable_hand.iter())
            .chain(cv.pitchable_hand.iter())
            .chain(cv.kickable_hand.iter())
            .chain(cv.suspendable_hand.iter())
            .chain(cv.foretellable_hand.iter())
            .chain(cv.plottable_hand.iter())
            .chain(cv.adventurable_hand.iter())
            .copied()
            .filter(|id| !hard.contains(id))
            .collect();
        (hard, alt)
    } else {
        (Default::default(), Default::default())
    };

    for (entity, gid, marker) in &hand_cards {
        // Desired colour: Some(false) = green hard-castable,
        // Some(true) = cyan alt-cost, None = no border.
        let desired = if hard.contains(&gid.0) {
            Some(false)
        } else if alt.contains(&gid.0) {
            Some(true)
        } else {
            None
        };
        match (desired, marker) {
            // Already showing the right colour — nothing to do.
            (Some(is_alt), Some(h)) if h.alt == is_alt => {}
            // No longer playable (or targeting took over) — tear down.
            (None, Some(h)) => {
                commands.entity(h.back).despawn();
                commands.entity(h.front).despawn();
                commands.entity(entity).remove::<CastableHighlight>();
            }
            // New highlight, or the category flipped (re-tint) — (re)spawn.
            (Some(is_alt), prev) => {
                if let Some(h) = prev {
                    commands.entity(h.back).despawn();
                    commands.entity(h.front).despawn();
                }
                let mat = if is_alt {
                    assets.alt_castable_material.clone()
                } else {
                    assets.castable_material.clone()
                };
                // Sits slightly proud of the hover border (0.001) so a card
                // that is both playable and hovered shows both rings without
                // z-fighting.
                let offset = CARD_THICKNESS / 2.0 + 0.0018;
                let back = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(mat.clone()),
                        Transform::from_xyz(0.0, 0.0, -offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                let front = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(mat),
                        Transform::from_xyz(0.0, 0.0, offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert(CastableHighlight { back, front, alt: is_alt })
                    .add_children(&[back, front]);
            }
            (None, None) => {}
        }
    }
}

/// Spawn / despawn a red "will die in combat" border around each
/// battlefield creature in the view's `combat_preview.dying_creatures`
/// set. The engine computes that set from the *current* attacker/blocker
/// assignment, so the markers update live as the player declares attacks
/// and blocks — letting them read every projected trade at a glance
/// before committing. Mirrors `update_castable_highlights`.
#[allow(clippy::type_complexity)]
pub fn update_dying_highlights(
    mut commands: Commands,
    view: Res<CurrentView>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    bf_cards: Query<(Entity, &GameCardId, Option<&DyingHighlight>), With<BattlefieldCard>>,
    // A creature that actually dies keeps its entity while it animates to
    // the graveyard (BattlefieldCard is removed, not despawned — see
    // `sync_game_visuals`), so strip the border from anything that left
    // the battlefield still wearing it.
    left_bf: Query<(Entity, &DyingHighlight), Without<BattlefieldCard>>,
) {
    let Some(assets) = highlight_assets else { return };

    for (entity, h) in &left_bf {
        commands.entity(h.back).despawn();
        commands.entity(h.front).despawn();
        commands.entity(entity).remove::<DyingHighlight>();
    }

    let dying: std::collections::HashSet<crabomination::card::CardId> = view
        .0
        .as_ref()
        .and_then(|cv| cv.combat_preview.as_ref())
        .map(|cp| cp.dying_creatures.iter().copied().collect())
        .unwrap_or_default();

    for (entity, gid, marker) in &bf_cards {
        let should = dying.contains(&gid.0);
        match (should, marker) {
            (true, None) => {
                // Offset between the hover gold (0.001) and castable green
                // (0.0018) borders so a doomed, hovered creature shows all
                // its rings without z-fighting.
                let offset = CARD_THICKNESS / 2.0 + 0.0015;
                let back = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.dying_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                let front = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.dying_material.clone()),
                        Transform::from_xyz(0.0, 0.0, offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert(DyingHighlight { back, front })
                    .add_children(&[back, front]);
            }
            (false, Some(h)) => {
                commands.entity(h.back).despawn();
                commands.entity(h.front).despawn();
                commands.entity(entity).remove::<DyingHighlight>();
            }
            _ => {}
        }
    }
}

/// Root marker for the keyboard-shortcut help overlay.
#[derive(Component)]
pub struct ShortcutHelpPanel;

/// Key → action reference, grouped into sections. Sourced from the actual
/// input handlers (`game_ui`, `kb_cursor`, `decision_ui`, `animate`,
/// `camera_zoom`, `debug_console`, `buttons`) — keep in sync when a
/// binding changes.
const HELP_SECTIONS: &[(&str, &[(&str, &str)])] = &[
    (
        "General",
        &[
            ("F1  ·  ?", "Toggle this help"),
            ("Esc", "Cancel / close / back"),
            ("Ctrl (hold)", "Zoom camera to hovered card"),
            ("Alt (hold)", "Enlarge card · counters & P/T"),
            ("[  ]  \\", "Animation: slower · faster · reset"),
            ("`", "Debug console"),
            ("X", "Export game state"),
        ],
    ),
    (
        "Your turn",
        &[
            ("Space", "Pass priority"),
            ("E", "End turn"),
            ("N", "Next turn (skip to your main)"),
            ("A", "Attack all · confirm attackers"),
            ("P", "Pass · skip · proceed"),
        ],
    ),
    (
        "Hand & cards",
        &[
            ("Click · Enter", "Play / cast / select target"),
            ("Tab · arrows · WASD", "Move selection (Shift+Tab back)"),
            ("F", "Flip double-faced card"),
            ("L", "Cast for alternative cost"),
            ("M", "Open ability menu"),
            ("C", "Cycle the selected card"),
            ("Right-click", "Cancel · flip card"),
        ],
    ),
    (
        "Mulligan",
        &[
            ("K", "Keep hand"),
            ("M", "Mulligan"),
            ("P", "Serum Powder"),
        ],
    ),
];

/// Toggle the keyboard-shortcut overlay on F1 or `?` (Shift+/); Esc (or
/// the same keys) closes it. Stateless — the panel entity *is* the open
/// flag, so there's nothing to fall out of sync if `OnExit(InGame)`
/// despawns the overlay while it's up.
pub fn toggle_shortcut_help(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<ShortcutHelpPanel>>,
) {
    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    let open_key =
        keyboard.just_pressed(KeyCode::F1) || (shift && keyboard.just_pressed(KeyCode::Slash));

    if let Ok(panel) = existing.single() {
        if open_key || keyboard.just_pressed(KeyCode::Escape) {
            commands.entity(panel).despawn();
        }
    } else if open_key {
        spawn_shortcut_help(&mut commands, &ui_fonts);
    }
}

/// Build the centered shortcut panel: a dimmed scrim over a two-column
/// reference grid. Tagged `InGameRoot` so it's cleaned up on exit even if
/// left open.
fn spawn_shortcut_help(commands: &mut Commands, ui_fonts: &UiFonts) {
    let tf = |s: f32| ui_fonts.tf(s);
    let mid = HELP_SECTIONS.len().div_ceil(2);

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
            BackgroundColor(theme::OVERLAY_BG),
            ShortcutHelpPanel,
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(22.0)),
                    row_gap: Val::Px(14.0),
                    border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|panel| {
                panel.spawn((
                    Text::new("Keyboard Shortcuts"),
                    tf(22.0),
                    TextColor(theme::ACCENT_GOLD),
                ));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(32.0),
                        align_items: AlignItems::FlexStart,
                        ..default()
                    })
                    .with_children(|cols| {
                        for chunk in [&HELP_SECTIONS[..mid], &HELP_SECTIONS[mid..]] {
                            cols.spawn(Node {
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(12.0),
                                min_width: Val::Px(330.0),
                                ..default()
                            })
                            .with_children(|col| {
                                for (header, rows) in chunk {
                                    col.spawn((
                                        Text::new(*header),
                                        tf(14.0),
                                        TextColor(theme::ACCENT_BLUE),
                                    ));
                                    for (key, desc) in *rows {
                                        col.spawn(Node {
                                            flex_direction: FlexDirection::Row,
                                            column_gap: Val::Px(10.0),
                                            align_items: AlignItems::Center,
                                            ..default()
                                        })
                                        .with_children(|row| {
                                            row.spawn(Node {
                                                min_width: Val::Px(126.0),
                                                ..default()
                                            })
                                            .with_children(|kc| {
                                                kc.spawn((
                                                    Text::new(*key),
                                                    tf(13.0),
                                                    TextColor(theme::ACCENT_YELLOW),
                                                ));
                                            });
                                            row.spawn((
                                                Text::new(*desc),
                                                tf(13.0),
                                                TextColor(theme::TEXT_BODY),
                                            ));
                                        });
                                    }
                                }
                            });
                        }
                    });

                // Card-border colour legend — documents the 3-D highlight
                // vocabulary, and pairs every colour with a label so it
                // reads without relying on hue alone. Swatches are hollow
                // (a coloured border) to mirror the on-card highlight.
                // Colours match the materials in `card::spawn`.
                panel.spawn((Text::new("Card borders"), tf(14.0), TextColor(theme::ACCENT_BLUE)));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        flex_wrap: FlexWrap::Wrap,
                        column_gap: Val::Px(20.0),
                        row_gap: Val::Px(6.0),
                        ..default()
                    })
                    .with_children(|legend| {
                        for (color, label) in [
                            (Color::srgb(0.20, 0.90, 0.35), "Castable now"),
                            (Color::srgb(0.15, 0.80, 0.95), "Dash / pitch / kicker"),
                            (Color::srgb(0.95, 0.12, 0.12), "Will die in combat"),
                            (Color::srgb(1.0, 0.85, 0.0), "Hover / target"),
                        ] {
                            legend
                                .spawn(Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: Val::Px(7.0),
                                    ..default()
                                })
                                .with_children(|row| {
                                    row.spawn((
                                        Node {
                                            width: Val::Px(16.0),
                                            height: Val::Px(16.0),
                                            border: UiRect::all(Val::Px(2.5)),
                                            ..default()
                                        },
                                        BorderColor::all(color),
                                        BackgroundColor(Color::NONE),
                                    ));
                                    row.spawn((Text::new(label), tf(13.0), TextColor(theme::TEXT_BODY)));
                                });
                        }
                    });

                panel.spawn((
                    Text::new("Press F1, ?, or Esc to close"),
                    tf(12.0),
                    TextColor(theme::TEXT_MUTED),
                ));
            });
        });
}

/// Standard Magic card aspect ratio (height / width).
const CARD_ASPECT_RATIO: f32 = 88.0 / 63.0;
const POPUP_WIDTH: f32 = 340.0;
const POPUP_HEIGHT: f32 = POPUP_WIDTH * CARD_ASPECT_RATIO;

pub fn peek_popup(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    hovered_cards: Query<&CardFrontTexture, (With<Card>, With<CardHovered>)>,
    existing_popup: Query<Entity, With<PeekPopup>>,
    asset_server: Res<AssetServer>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let should_show = alt_held && !hovered_cards.is_empty();

    if should_show && existing_popup.is_empty() {
        if let Some(front_texture) = hovered_cards.iter().next() {
            let texture: Handle<Image> = asset_server.load(&front_texture.0);
            // Full-screen overlay, flex-centered, with dim background.
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
                    BackgroundColor(theme::OVERLAY_BG_LIGHT),
                    Pickable::IGNORE,
                    PeekPopup,
                ))
                .with_children(|parent| {
                    parent.spawn((
                        ImageNode {
                            image: texture,
                            ..default()
                        },
                        Node {
                            width: Val::Px(POPUP_WIDTH),
                            height: Val::Px(POPUP_HEIGHT),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
    } else if !should_show {
        for entity in &existing_popup {
            commands.entity(entity).despawn();
        }
    }
}

const BROWSER_CARD_WIDTH: f32 = 220.0;
const BROWSER_CARD_HEIGHT: f32 = BROWSER_CARD_WIDTH * CARD_ASPECT_RATIO;
const BROWSER_COLS: u32 = 4;

pub fn graveyard_browser(
    mut commands: Commands,
    mut state: ResMut<GraveyardBrowserState>,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<GraveyardBrowser>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    overlay_interaction: Query<&Interaction, (With<GraveyardBrowser>, With<Button>)>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }

    // Close when the dim overlay (outside the panel) is clicked.
    for interaction in &overlay_interaction {
        if *interaction == Interaction::Pressed {
            state.open = false;
        }
    }

    let should_show = state.open;

    if should_show && existing.is_empty() {
        let owner = state.owner;
        let owner_name: String = view
            .0
            .as_ref()
            .and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {owner}"));
        let owner_label = format!("{owner_name}'s");

        // (name, recast-badge): badge is "Flashback {N}" / "Retrace" for
        // cards castable from the graveyard, surfaced from the view's new
        // recast flags so players can see their options at a glance.
        let card_names: Vec<(String, Option<String>)> = view.0.as_ref()
            .map(|cv| cv.players[owner].graveyard.iter().map(|c| {
                let badge = if let Some(fb) = &c.flashback_cost {
                    Some(format!("Flashback {{{}}}", fb.cmc()))
                } else if c.retrace {
                    Some("Retrace".to_string())
                } else {
                    None
                };
                (c.name.clone(), badge)
            }).collect())
            .unwrap_or_default();
        let count = card_names.len();

        let panel_width = BROWSER_CARD_WIDTH * BROWSER_COLS as f32 + 80.0;

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
                GraveyardBrowser,
            ))
            .id();

        let panel = commands
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(16.0)),
                    row_gap: Val::Px(12.0),
                    width: Val::Px(panel_width),
                    max_height: Val::Percent(85.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .id();

        commands.entity(root).add_child(panel);

        commands.entity(panel)
            .with_children(|panel| {
                // Header row
                panel
                    .spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            align_items: AlignItems::Center,
                            margin: UiRect::bottom(Val::Px(8.0)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|row| {
                        row.spawn((
                            Text::new(format!("{owner_label} Graveyard ({count} cards)")),
                            ui_fonts.tf(18.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });

                // Card grid
                if card_names.is_empty() {
                    panel.spawn((
                        Text::new("Empty"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::TEXT_SECONDARY),
                        Pickable::IGNORE,
                    ));
                } else {
                    panel
                        .spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(8.0),
                                row_gap: Val::Px(8.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|grid| {
                            for (name, badge) in &card_names {
                                let path = scryfall::card_asset_path(name);
                                let texture: Handle<Image> = asset_server.load(&path);
                                // Each tile is a Button so Bevy's
                                // `Interaction` component reports
                                // hover/press. The ImageNode inside
                                // is `Pickable::IGNORE` so events
                                // bubble up to the button rather than
                                // landing on the image directly. The
                                // outer button captures clicks too,
                                // which prevents accidental close-on-
                                // click since the dim overlay used to
                                // swallow clicks that passed through
                                // the cards.
                                grid.spawn((
                                    Button,
                                    Node {
                                        width: Val::Px(BROWSER_CARD_WIDTH),
                                        height: Val::Px(BROWSER_CARD_HEIGHT),
                                        ..default()
                                    },
                                    GraveyardCardItem { name: name.clone() },
                                ))
                                .with_children(|tile| {
                                    tile.spawn((
                                        ImageNode { image: texture, ..default() },
                                        Node {
                                            width: Val::Percent(100.0),
                                            height: Val::Percent(100.0),
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ));
                                    // Recast badge pinned to the bottom of
                                    // the tile (Flashback / Retrace).
                                    if let Some(label) = badge {
                                        tile.spawn((
                                            Node {
                                                position_type: PositionType::Absolute,
                                                bottom: Val::Px(2.0),
                                                left: Val::Px(2.0),
                                                right: Val::Px(2.0),
                                                justify_content: JustifyContent::Center,
                                                ..default()
                                            },
                                            BackgroundColor(theme::OVERLAY_BG),
                                            Pickable::IGNORE,
                                        ))
                                        .with_children(|b| {
                                            b.spawn((
                                                Text::new(label.clone()),
                                                ui_fonts.tf(10.0),
                                                TextColor(theme::TEXT_PRIMARY),
                                                Pickable::IGNORE,
                                            ));
                                        });
                                    }
                                });
                            }
                        });
                }

                // Close hint
                panel.spawn((
                    Text::new("Click outside or press Esc to close"),
                    ui_fonts.tf(11.0),
                    TextColor(theme::TEXT_MUTED),
                    Pickable::IGNORE,
                ));
            });
    } else if !should_show {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

pub fn pile_tooltip(
    mut commands: Commands,
    view: Res<CurrentView>,
    ui_fonts: Res<UiFonts>,
    deck_hovered: Query<(), (With<DeckCard>, With<CardHovered>)>,
    pile_hovered: Query<&DeckPile, With<PileHovered>>,
    gy_hovered: Query<&GraveyardPile, With<PileHovered>>,
    existing: Query<(Entity, &PileTooltip)>,
) {
    let cv = view.0.as_ref();
    let lib_size = |owner: usize| -> usize {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.library.size)
            .unwrap_or(0)
    };
    let gy_count = |owner: usize| -> usize {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.graveyard.len())
            .unwrap_or(0)
    };
    let player_name = |owner: usize| -> String {
        cv.and_then(|cv| cv.players.iter().find(|p| p.seat == owner))
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {owner}"))
    };
    let viewer = cv.map(|cv| cv.your_seat).unwrap_or(0);

    let text = if !deck_hovered.is_empty() {
        // Hovering a face-up viewer-deck card visual.
        let count = lib_size(viewer);
        Some(format!("{} library: {count} card{}", player_name(viewer), if count == 1 { "" } else { "s" }))
    } else if let Some(pile) = pile_hovered.iter().next() {
        let count = lib_size(pile.owner);
        Some(format!("{} library: {count} card{}", player_name(pile.owner), if count == 1 { "" } else { "s" }))
    } else if let Some(gy) = gy_hovered.iter().next() {
        let count = gy_count(gy.owner);
        Some(format!("{}'s graveyard: {count} card{} — click to browse", player_name(gy.owner), if count == 1 { "" } else { "s" }))
    } else {
        None
    };

    if let Some(msg) = text {
        // If a tooltip is already pinned with the *same* text, leave it.
        // Otherwise despawn the stale one so we can respawn with the
        // current message (fixes the cursor-moved-between-piles staleness).
        let up_to_date = existing.iter().any(|(_, t)| t.msg == msg);
        if !up_to_date {
            for (e, _) in existing.iter() {
                commands.entity(e).despawn();
            }
            // Pin the tooltip's *center* to the screen midline. Without
            // the negative margin Bevy positions the node's left edge at
            // 50% and the tooltip drifts right of center (worse the
            // longer the string). 140px is half the widest tooltip text
            // ("Player N's graveyard: NN cards — click to browse") at
            // the 14px font, which is close enough for visual centering
            // without measuring text width per frame.
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(130.0),
                    left: Val::Percent(50.0),
                    margin: UiRect::left(Val::Px(-140.0)),
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BackgroundColor(theme::OVERLAY_BG_HEAVY),
                PileTooltip { msg: msg.clone() },
            ))
            .with_children(|p| {
                p.spawn((
                    Text::new(msg),
                    ui_fonts.tf(14.0),
                    TextColor(theme::TEXT_PRIMARY),
                    Pickable::IGNORE,
                ));
            });
        }
    } else {
        for (entity, _) in &existing {
            commands.entity(entity).despawn();
        }
    }
}

/// Show the name of the graveyard card currently being hovered as
/// a tooltip pinned to the top of the browser panel. Despawns when
/// the cursor leaves all card tiles, or when the browser closes.
pub fn graveyard_card_hover_name(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    items: Query<(&Interaction, &GraveyardCardItem)>,
    existing: Query<Entity, With<GraveyardCardNameTooltip>>,
    browser: Query<(), With<GraveyardBrowser>>,
    mut current_text: Query<&mut Text, With<GraveyardCardNameTooltip>>,
) {
    // No browser open → drop the tooltip.
    if browser.is_empty() {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    }

    let hovered_name = items
        .iter()
        .find(|(i, _)| matches!(**i, Interaction::Hovered | Interaction::Pressed))
        .map(|(_, item)| item.name.clone());

    let Some(name) = hovered_name else {
        for e in &existing {
            commands.entity(e).despawn();
        }
        return;
    };

    // Tooltip already exists — just refresh its text.
    if let Ok(mut t) = current_text.single_mut() {
        if t.0 != name {
            t.0 = name;
        }
        return;
    }

    // Spawn the tooltip pinned near the top of the screen so it's
    // always visible even when scrolling deep into a long graveyard.
    // Marker lives on the Text entity (parent + child) — keeping it
    // single-entity makes both the despawn and the text-refresh
    // queries straightforward.
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                left: Val::Percent(50.0),
                margin: UiRect::left(Val::Px(-110.0)),
                width: Val::Px(220.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(8.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_HEAVY),
            Pickable::IGNORE,
            Text::new(name),
            ui_fonts.tf(16.0),
            TextColor(theme::ACCENT_GOLD),
            GraveyardCardNameTooltip,
        ));
}

/// Drives the top-card reveal popup — stays visible until the user clicks anywhere.
pub fn reveal_popup(
    mut commands: Commands,
    mut state: ResMut<RevealPopupState>,
    existing: Query<Entity, With<RevealPopup>>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    mouse: Res<ButtonInput<MouseButton>>,
) {
    if state.card_path.is_some() {
        // Dismiss on any mouse click.
        if mouse.just_pressed(MouseButton::Left) {
            state.card_path = None;
            for e in &existing { commands.entity(e).despawn(); }
            return;
        }

        if existing.is_empty() {
            let path = state.card_path.clone().unwrap();
            let texture: Handle<Image> = asset_server.load(path);
            commands
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Percent(50.0),
                        top: Val::Percent(18.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(theme::OVERLAY_BG_HEAVY),
                    Pickable::IGNORE,
                    RevealPopup,
                ))
                .with_children(|p| {
                    p.spawn((
                        Text::new("Goblin Guide reveals: (click to dismiss)"),
                        ui_fonts.tf(14.0),
                        TextColor(theme::ACCENT_ORANGE),
                        Pickable::IGNORE,
                    ));
                    p.spawn((
                        ImageNode { image: texture, ..default() },
                        Node {
                            width: Val::Px(POPUP_WIDTH * 0.6),
                            height: Val::Px(POPUP_HEIGHT * 0.6),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                });
        }
    } else {
        for e in &existing { commands.entity(e).despawn(); }
    }
}
