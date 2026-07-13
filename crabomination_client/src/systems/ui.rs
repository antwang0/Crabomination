use bevy::prelude::*;

use crate::card::{
    ActivatableHighlight, BattlefieldCard, Card, CardBorderHighlight, CardFrontTexture,
    CardHighlightAssets, CardHovered, CastableHighlight, DeckCard, DeckPile, DyingHighlight,
    GameCardId, GraveyardPile, HandCard, PileHovered, CARD_THICKNESS,
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

/// Alt-held centered card peek. Stores the front-texture path currently
/// shown so the popup rebuilds when the hovered card changes under a held
/// Alt (instead of showing the first card until Alt is released).
#[derive(Component)]
pub struct PeekPopup {
    path: String,
}

#[derive(Component)]
pub struct GraveyardBrowser;

/// Root marker for the exile-zone browser popup (toggled with V).
#[derive(Component)]
pub struct ExileBrowser;

/// One card tile inside the graveyard browser. Stores the card's
/// display name so the hover-name-tooltip system can render it, and an
/// optional graveyard-recast action for click-to-cast tiles.
#[derive(Component)]
pub struct GraveyardCardItem {
    pub name: String,
    /// `Some` when this is the viewer's own castable graveyard card —
    /// clicking the tile submits the recast (Flashback / Mayhem / Harmonize /
    /// Disturb / Retrace / Escape) with auto-chosen targets.
    pub recast: Option<(crabomination::card::CardId, GraveyardRecast)>,
}

/// Which graveyard-cast action a browser tile submits on click.
#[derive(Clone, Copy)]
pub enum GraveyardRecast {
    Flashback,
    Mayhem,
    Harmonize,
    Disturb,
    Retrace,
    /// Escape with this many other graveyard cards to exile (auto-picked).
    Escape(u32),
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
/// alternative path — Dash, Blitz, pitch/exile, kicker, Suspend, or a live
/// Miracle window
/// (`dashable_hand` / `blitzable_hand` / `pitchable_hand` / `kickable_hand` /
/// `suspendable_hand` / `miracle_hand`, minus anything already hard-castable). All are computed server-side via the engine's
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
    // Hand entities spawn via deferred commands, landing a frame after the
    // view change that caused them — they must still get their border.
    added_hand: Query<(), Added<HandCard>>,
) {
    let Some(assets) = highlight_assets else { return };

    for (entity, h) in &left_hand {
        commands.entity(h.back).despawn();
        commands.entity(h.front).despawn();
        commands.entity(entity).remove::<CastableHighlight>();
    }

    // The castable sets only change with the view or the targeting mode;
    // skip rebuilding the two HashSets (~20 chained iterators) otherwise.
    // The `left_hand` cleanup above stays unguarded: HandCard removal is a
    // deferred command that can land a frame after the view change.
    if !view.is_changed() && !targeting.is_changed() && added_hand.is_empty() {
        return;
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
            // Back-face-only castable MDFCs read as an alternative
            // path: flip (right-click / F), then cast.
            .chain(cv.back_castable_hand.iter())
            .chain(cv.blitzable_hand.iter())
            .chain(cv.warpable_hand.iter())
            .chain(cv.pitchable_hand.iter())
            .chain(cv.kickable_hand.iter())
            .chain(cv.suspendable_hand.iter())
            .chain(cv.foretellable_hand.iter())
            .chain(cv.plottable_hand.iter())
            .chain(cv.adventurable_hand.iter())
            .chain(cv.omenable_hand.iter())
            .chain(cv.splittable_right_hand.iter())
            .chain(cv.miracle_hand.iter())
            .chain(cv.morphable_hand.iter())
            .chain(cv.reinforceable_hand.iter())
            .chain(cv.discard_activatable_hand.iter())
            .chain(cv.squadable_hand.iter())
            .chain(cv.spreeable_hand.iter())
            .chain(cv.replicatable_hand.iter())
            .chain(cv.conspirable_hand.iter())
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

/// Spawn / despawn a violet "can activate now" border around each
/// battlefield permanent the viewer controls that has an ability they could
/// activate right now (`activatable_permanents`). The engine computes the
/// set via a `would_accept` dry-run — so it already honors timing, mana, tap
/// state, target availability, and priority (the list is empty off-priority,
/// and excludes pure mana abilities). This is what makes "crack a fetch land
/// on the opponent's end step" visible: the land glows the moment you receive
/// priority. Mirrors `update_dying_highlights`.
///
/// Suppressed while the targeting cursor is active so the gold valid-target
/// borders own the visuals and a second ring set doesn't z-fight.
#[allow(clippy::type_complexity)]
pub fn update_activatable_highlights(
    mut commands: Commands,
    view: Res<CurrentView>,
    targeting: Res<crate::game::TargetingState>,
    highlight_assets: Option<Res<CardHighlightAssets>>,
    bf_cards: Query<(Entity, &GameCardId, Option<&ActivatableHighlight>), With<BattlefieldCard>>,
    // A permanent that leaves the battlefield (sacrificed fetch land, etc.)
    // keeps its entity briefly while it animates away (BattlefieldCard is
    // removed, not despawned — see `sync_game_visuals`), so strip the border
    // from anything that left the battlefield still wearing it.
    left_bf: Query<(Entity, &ActivatableHighlight), Without<BattlefieldCard>>,
) {
    let Some(assets) = highlight_assets else { return };

    for (entity, h) in &left_bf {
        commands.entity(h.back).despawn();
        commands.entity(h.front).despawn();
        commands.entity(entity).remove::<ActivatableHighlight>();
    }

    let activatable: std::collections::HashSet<crabomination::card::CardId> = if targeting.active {
        Default::default()
    } else {
        view.0
            .as_ref()
            // Face-down permanents the viewer can turn face up (CR 708.5) glow
            // alongside permanents with an activatable ability.
            .map(|cv| {
                cv.activatable_permanents
                    .iter()
                    .chain(cv.turn_up_able.iter())
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    };

    for (entity, gid, marker) in &bf_cards {
        let should = activatable.contains(&gid.0);
        match (should, marker) {
            (true, None) => {
                // Offset proud of the hover gold (0.001), dying red (0.0015),
                // and castable green (0.0018) borders so an activatable,
                // hovered permanent shows every ring without z-fighting.
                let offset = CARD_THICKNESS / 2.0 + 0.0021;
                let back = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.activatable_material.clone()),
                        Transform::from_xyz(0.0, 0.0, -offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                let front = commands
                    .spawn((
                        Mesh3d(assets.border_mesh.clone()),
                        MeshMaterial3d(assets.activatable_material.clone()),
                        Transform::from_xyz(0.0, 0.0, offset),
                        Pickable::IGNORE,
                    ))
                    .id();
                commands
                    .entity(entity)
                    .insert(ActivatableHighlight { back, front })
                    .add_children(&[back, front]);
            }
            (false, Some(h)) => {
                commands.entity(h.back).despawn();
                commands.entity(h.front).despawn();
                commands.entity(entity).remove::<ActivatableHighlight>();
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
            ("Hover", "Preview card beside cursor"),
            ("Ctrl (hold)", "Zoom camera to hovered card"),
            ("Alt (hold)", "Centered card · counters & P/T"),
            ("[  ]  \\", "Animation: slower · faster · reset"),
            ("`", "Debug console"),
            ("X", "Export game state"),
            ("G", "Browse your graveyard (click a pile for others')"),
            ("V", "Browse the exile zone"),
            ("Click phase chart", "Cycle stop / skip for that step"),
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
            ("H", "Hold priority (auto-pass on/off)"),
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
    hovered_cards: Query<(&CardFrontTexture, Option<&GameCardId>), (With<Card>, With<CardHovered>)>,
    card_names: Res<crate::game::CardNames>,
    existing_popup: Query<(Entity, &PeekPopup)>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let hovered = hovered_cards.iter().next();
    let should_show = alt_held && hovered.is_some();

    if !should_show {
        for (entity, _) in &existing_popup {
            commands.entity(entity).despawn();
        }
        return;
    }
    let Some((front_texture, card_id)) = hovered else { return };
    // Same card already showing — nothing to do; a different card under a
    // still-held Alt rebuilds the popup with the new faces.
    if let Ok((entity, popup)) = existing_popup.single() {
        if popup.path == front_texture.0 {
            return;
        }
        commands.entity(entity).despawn();
    }

    // The catalog knows whether this card has a back face (MDFC/TDFC) and
    // supplies the rules-text lines — both keyed by the card's name.
    let name = card_id.map(|g| card_names.get(g.0));
    let def = name.as_deref().and_then(crabomination::catalog::lookup_by_name);
    let back_path = def
        .as_ref()
        .and_then(|d| d.back_face.as_ref())
        .map(|b| scryfall::card_back_face_asset_path(b.name));
    let info = name.as_deref().map(hover_info_lines).unwrap_or_default();

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
                column_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(theme::OVERLAY_BG_LIGHT),
            Pickable::IGNORE,
            PeekPopup { path: front_texture.0.clone() },
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
            // Double-faced card: show the other face alongside, so the
            // battlefield/opponent side of a DFC is finally readable.
            if let Some(back_path) = back_path {
                let back_texture: Handle<Image> = asset_server.load(&back_path);
                parent.spawn((
                    ImageNode {
                        image: back_texture,
                        ..default()
                    },
                    Node {
                        width: Val::Px(POPUP_WIDTH),
                        height: Val::Px(POPUP_HEIGHT),
                        ..default()
                    },
                    Pickable::IGNORE,
                ));
            }
            // Rules-text panel — the big peek is the one place with room
            // for the full type line / keyword reminders, and cards render
            // art-only, so surface them here too (not just on the small
            // cursor-side preview).
            if !info.is_empty() {
                parent
                    .spawn((
                        Node {
                            width: Val::Px(300.0),
                            max_height: Val::Px(POPUP_HEIGHT),
                            flex_direction: FlexDirection::Column,
                            padding: UiRect::all(Val::Px(12.0)),
                            row_gap: Val::Px(4.0),
                            overflow: Overflow::clip_y(),
                            border_radius: BorderRadius::all(theme::RADIUS_PANEL),
                            ..default()
                        },
                        BackgroundColor(theme::PANEL_BG),
                        Pickable::IGNORE,
                    ))
                    .with_children(|panel| {
                        for (text, is_reminder) in info {
                            let color = if is_reminder {
                                theme::TEXT_SECONDARY
                            } else {
                                theme::TEXT_PRIMARY
                            };
                            panel.spawn((
                                Text::new(text),
                                ui_fonts.tf(13.0),
                                TextColor(color),
                                Pickable::IGNORE,
                            ));
                        }
                    });
            }
        });
}

/// Marker for the automatic hover card-zoom preview (Arena-style). Stores
/// the asset path currently shown so `hover_card_preview` can tell when the
/// hovered card changed (→ rebuild with new art) versus merely moved
/// (→ reposition the existing node).
#[derive(Component)]
pub struct HoverCardPreview {
    path: String,
}

pub(crate) const HOVER_PREVIEW_WIDTH: f32 = 230.0;
pub(crate) const HOVER_PREVIEW_HEIGHT: f32 = HOVER_PREVIEW_WIDTH * CARD_ASPECT_RATIO;
/// Gap kept between the preview and both the cursor and the viewport edges.
pub(crate) const HOVER_PREVIEW_MARGIN: f32 = 16.0;

/// Top-left (x, y) in logical px for a `pw × ph` preview placed beside the
/// cursor. Picks whichever horizontal side has more room so the preview
/// never covers the card the cursor is resting on, then clamps both axes so
/// the whole card stays on screen. Vertically centered on the cursor.
pub(crate) fn preview_anchor(cursor: Vec2, win: Vec2, pw: f32, ph: f32, margin: f32) -> (f32, f32) {
    let x_max = (win.x - pw - margin).max(margin);
    // More room to the right → sit to the cursor's right, else to its left.
    let x = if win.x - cursor.x >= cursor.x {
        (cursor.x + margin).clamp(margin, x_max)
    } else {
        (cursor.x - margin - pw).clamp(margin, x_max)
    };
    let y_max = (win.y - ph - margin).max(margin);
    let y = (cursor.y - ph * 0.5).clamp(margin, y_max);
    (x, y)
}

/// Rules-text lines for the hover preview's info panel, resolved from the
/// catalog by card name. Since cards render art-only (no printed text box),
/// this is where a player learns what a hovered card actually does: the
/// type line, P/T, and each printed keyword with its CR reminder text
/// ("Menace — can only be blocked by two or more creatures."). Returns
/// `(text, is_reminder)` pairs; reminder lines render dimmer. Empty when
/// the name isn't in the catalog (tokens, fictional placeholders).
/// Live characteristic-override notes for a battlefield permanent whose
/// computed state diverges from its printed card (continuous effects —
/// Ichthyomorphosis, Heliod's Punishment, Turn to Frog). Appended below the
/// printed `hover_info_lines` so the player sees what the permanent *is now*,
/// not just what it was printed as. Returns dimmed `(text, true)` reminder
/// lines; empty when nothing diverges.
fn live_override_lines(
    name: &str,
    lost_all_abilities: bool,
    creature_subtypes: &[crabomination::card::CreatureType],
    colors: &[crabomination::mana::Color],
) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    if lost_all_abilities {
        out.push(("Now: all abilities removed".to_string(), true));
    }
    // Computed creature subtypes that differ from the printed ones (a
    // type-changing aura / animation). Only note it when there's an actual
    // change so vanilla creatures stay quiet.
    let printed = crabomination::catalog::lookup_by_name(name);
    let printed_types = printed
        .as_ref()
        .map(|d| d.subtypes.creature_types.clone())
        .unwrap_or_default();
    if !creature_subtypes.is_empty() && creature_subtypes != printed_types.as_slice() {
        let line = creature_subtypes
            .iter()
            .map(|t| format!("{t:?}"))
            .collect::<Vec<_>>()
            .join(" ");
        out.push((format!("Now: {line}"), true));
    }
    // Computed colors that differ from the printed card's colors (a
    // color-changing effect — Turn to Frog / Snakeform become blue/green).
    if let Some(def) = printed.as_ref() {
        let mut printed_colors = def.cost.colors();
        for c in &def.color_indicator {
            if !printed_colors.contains(c) {
                printed_colors.push(*c);
            }
        }
        if colors != printed_colors.as_slice() {
            let label = if colors.is_empty() {
                "colorless".to_string()
            } else {
                colors
                    .iter()
                    .map(|c| format!("{c:?}").to_lowercase())
                    .collect::<Vec<_>>()
                    .join(" and ")
            };
            out.push((format!("Now: {label}"), true));
        }
    }
    out
}

fn hover_info_lines(name: &str) -> Vec<(String, bool)> {
    let Some(def) = crabomination::catalog::lookup_by_name(name) else {
        return Vec::new();
    };
    let mut lines: Vec<(String, bool)> = Vec::new();
    let types = def
        .card_types
        .iter()
        .map(|t| format!("{t:?}"))
        .collect::<Vec<_>>()
        .join(" ");
    let subtypes = def
        .subtypes
        .creature_types
        .iter()
        .map(|t| format!("{t:?}"))
        .collect::<Vec<_>>()
        .join(" ");
    let mut type_line = if subtypes.is_empty() {
        types
    } else {
        format!("{types} — {subtypes}")
    };
    if def.is_creature() {
        type_line.push_str(&format!("  ·  {}/{}", def.power, def.toughness));
    }
    if !type_line.is_empty() {
        lines.push((type_line, false));
    }
    for kw in &def.keywords {
        let label = crate::systems::counter_tooltip::keyword_label(kw);
        match crate::systems::counter_tooltip::keyword_reminder(kw) {
            Some(reminder) => lines.push((format!("{label} — {reminder}"), true)),
            None => lines.push((label, true)),
        }
    }
    // CR 702.139c — surface the companion's deck-construction restriction.
    if let Some(rule) = &def.companion {
        let text = crate::systems::counter_tooltip::companion_restriction_text(rule);
        lines.push((format!("Companion — {text}"), true));
    }
    // Oracle-ish ability text (roadmap Tier 8): statics carry printed
    // descriptions; triggered/activated/loyalty abilities are phrased from
    // their event + effect shapes. Empty phrasings are skipped.
    for sa in &def.static_abilities {
        if !sa.description.is_empty() {
            lines.push((sa.description.to_string(), true));
        }
    }
    for ta in &def.triggered_abilities {
        let body = ta.effect.effect_short_text();
        if body.is_empty() {
            continue;
        }
        lines.push((format!("{} {body}.", event_phrase(&ta.event)), true));
    }
    for aa in &def.activated_abilities {
        let body = aa.effect.effect_short_text();
        if body.is_empty() {
            continue;
        }
        let mut cost = Vec::new();
        let mana = aa.mana_cost.summary();
        if !mana.is_empty() {
            cost.push(mana);
        }
        if aa.tap_cost {
            cost.push("{T}".into());
        }
        if aa.life_cost > 0 {
            cost.push(format!("Pay {} life", aa.life_cost));
        }
        if aa.sac_cost {
            cost.push("Sacrifice this".into());
        }
        if aa.exile_self_cost {
            cost.push("Exile this from your graveyard".into());
        }
        if cost.is_empty() {
            cost.push("{0}".into());
        }
        let mut line = format!("{}: {body}.", cost.join(", "));
        // Surface the zone / timing restrictions the cost line alone hides
        // (Unearth, Renew, the SOS graveyard-return cycle, …).
        if aa.from_graveyard && !aa.exile_self_cost {
            line.push_str(" (from your graveyard)");
        }
        if aa.sorcery_speed {
            line.push_str(" (sorcery speed only)");
        }
        lines.push((line, true));
    }
    for la in &def.loyalty_abilities {
        let body = la.effect.effect_short_text();
        if body.is_empty() {
            continue;
        }
        let cost = if la.x_cost {
            "−X".to_string()
        } else {
            format!("{:+}", la.loyalty_cost)
        };
        lines.push((format!("[{cost}]: {body}."), true));
    }
    if !def.is_permanent() {
        let body = def.effect.effect_short_text();
        if !body.is_empty() {
            lines.push((first_upper(&body), true));
        }
    }
    lines
}

/// Phrase a trigger's firing condition ("When this enters," …). Covers the
/// common event kinds; falls back to a generic "Triggered ability:".
fn event_phrase(spec: &crabomination::card::EventSpec) -> String {
    use crabomination::effect::{EventKind as K, EventScope as S};
    let self_src = matches!(spec.scope, S::SelfSource);
    match &spec.kind {
        K::EntersBattlefield if self_src => "When this enters,".into(),
        K::EntersBattlefield => "Whenever a permanent enters,".into(),
        K::CreatureDied if self_src => "When this dies,".into(),
        K::CreatureDied => "Whenever a creature dies,".into(),
        K::Attacks if self_src => "Whenever this attacks,".into(),
        K::Attacks => "Whenever a creature attacks,".into(),
        K::Blocks => "Whenever this blocks,".into(),
        K::DealsCombatDamageToPlayer => "Whenever this deals combat damage to a player,".into(),
        K::SpellCast if self_src => "When you cast this spell,".into(),
        K::SpellCast => "Whenever a spell is cast,".into(),
        K::CardDrawn => "Whenever a card is drawn,".into(),
        K::LandPlayed => "Whenever a land enters,".into(),
        K::LifeGained if matches!(spec.scope, S::OpponentControl) => {
            "Whenever an opponent gains life,".into()
        }
        K::LifeGained => "Whenever life is gained,".into(),
        K::LifeLost if matches!(spec.scope, S::OpponentControl) => {
            "Whenever an opponent loses life,".into()
        }
        K::LifeLost => "Whenever a player loses life,".into(),
        K::YouAttack => "Whenever you attack,".into(),
        K::AttacksAndIsntBlocked if self_src => "Whenever this attacks and isn't blocked,".into(),
        K::DealsCombatDamageToCreature => "Whenever this deals combat damage to a creature,".into(),
        K::DealtDamage if self_src => "Whenever this is dealt damage,".into(),
        K::CreatureOrArtifactDied => "Whenever a creature or artifact you control dies,".into(),
        K::ScriedOrSurveiled if self_src => "Whenever you scry or surveil,".into(),
        K::Proliferated => "Whenever you proliferate,".into(),
        K::AnyCounterAdded if self_src => "Whenever a counter is put on this,".into(),
        K::StepBegins(step) => format!("At the beginning of {step:?},").to_lowercase().replacen("at", "At", 1),
        K::TurnedFaceUp => "When this is turned face up,".into(),
        K::Transformed => "When this transforms,".into(),
        _ => "Triggered ability:".into(),
    }
}

/// Uppercase the first character (effect phrasings are lowercase fragments).
fn first_upper(s: &str) -> String {
    let mut cs = s.chars();
    match cs.next() {
        Some(c) => c.to_uppercase().collect::<String>() + cs.as_str(),
        None => String::new(),
    }
}

/// Arena-style automatic card-zoom preview: while a card is hovered (and Alt
/// isn't held — Alt drives the centered detailed peek + counter tooltip in
/// `peek_popup` / `update_alt_tooltip`), show an enlarged copy of its face
/// beside the cursor without dimming the board, plus a type-line/keyword
/// reminder panel resolved from the catalog. Roadmap Tier 7 #1 / Tier 8
/// reminder text.
#[allow(clippy::type_complexity)]
pub fn hover_card_preview(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    hovered_cards: Query<(&CardFrontTexture, Option<&GameCardId>), (With<Card>, With<CardHovered>)>,
    card_names: Res<crate::game::CardNames>,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    mut existing: Query<(Entity, &mut Node, &HoverCardPreview)>,
) {
    let alt_held = keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight);
    let despawn_all = |commands: &mut Commands, existing: &Query<(Entity, &mut Node, &HoverCardPreview)>| {
        for (e, _, _) in existing.iter() {
            commands.entity(e).despawn();
        }
    };

    let Ok(window) = windows.single() else {
        despawn_all(&mut commands, &existing);
        return;
    };

    // Desired preview: front texture of the hovered card, when the cursor is
    // on-screen and Alt isn't held.
    let desired: Option<(String, Option<crabomination::card::CardId>, Vec2)> = match (alt_held, window.cursor_position()) {
        (false, Some(c)) => hovered_cards
            .iter()
            .next()
            .map(|(t, id)| (t.0.clone(), id.map(|g| g.0), c)),
        _ => None,
    };

    let Some((path, card_id, cursor)) = desired else {
        despawn_all(&mut commands, &existing);
        return;
    };

    let mut info = card_id
        .map(|id| hover_info_lines(&card_names.get(id)))
        .unwrap_or_default();
    // Double-faced cards: the other face is on the Alt peek — say so here,
    // since this small preview only shows the front.
    if let Some(id) = card_id
        && crabomination::catalog::lookup_by_name(&card_names.get(id))
            .is_some_and(|d| d.back_face.is_some())
    {
        info.push(("Double-faced — hold Alt to see both faces.".to_string(), true));
    }
    // Append live characteristic-override notes (type-changing / ability-
    // stripping continuous effects) from the current battlefield view.
    if let (Some(id), Some(cv)) = (card_id, view.0.as_ref())
        && let Some(pv) = cv.battlefield.iter().find(|p| p.id == id)
    {
        info.extend(live_override_lines(
            &pv.name,
            pv.lost_all_abilities,
            &pv.creature_subtypes,
            &pv.colors,
        ));
    }
    // Estimated panel height so the anchor keeps the whole column on screen
    // (reminder lines wrap to ~2 rows at this width).
    let info_height: f32 = if info.is_empty() {
        0.0
    } else {
        16.0 + info
            .iter()
            .map(|(text, _)| if text.len() > 44 { 34.0 } else { 18.0 })
            .sum::<f32>()
    };

    let win = Vec2::new(window.width(), window.height());
    let (x, y) = preview_anchor(
        cursor,
        win,
        HOVER_PREVIEW_WIDTH,
        HOVER_PREVIEW_HEIGHT + info_height,
        HOVER_PREVIEW_MARGIN,
    );

    if let Ok((entity, mut node, marker)) = existing.single_mut() {
        node.left = Val::Px(x);
        node.top = Val::Px(y);
        // Same card still hovered — repositioning above is all we need.
        if marker.path == path {
            return;
        }
        // Hovered card changed — rebuild with the new art below.
        commands.entity(entity).despawn();
    }

    let texture: Handle<Image> = asset_server.load(&path);
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(x),
                top: Val::Px(y),
                width: Val::Px(HOVER_PREVIEW_WIDTH),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            Pickable::IGNORE,
            HoverCardPreview { path: path.clone() },
            crate::systems::game_ui::InGameRoot,
        ))
        .with_children(|col| {
            col.spawn((
                Node {
                    width: Val::Px(HOVER_PREVIEW_WIDTH),
                    height: Val::Px(HOVER_PREVIEW_HEIGHT),
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                    ..default()
                },
                BorderColor::all(theme::ACCENT_GOLD),
                ImageNode { image: texture, ..default() },
                Pickable::IGNORE,
            ));
            if !info.is_empty() {
                col.spawn((
                    Node {
                        width: Val::Px(HOVER_PREVIEW_WIDTH),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        row_gap: Val::Px(3.0),
                        border_radius: BorderRadius::all(theme::RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL_BG),
                    Pickable::IGNORE,
                ))
                .with_children(|panel| {
                    for (text, is_reminder) in info {
                        let color = if is_reminder {
                            theme::TEXT_SECONDARY
                        } else {
                            theme::TEXT_PRIMARY
                        };
                        panel.spawn((
                            Text::new(text),
                            ui_fonts.tf(12.0),
                            TextColor(color),
                            Pickable::IGNORE,
                        ));
                    }
                });
            }
        });
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
    chat: Res<crate::systems::chat::ChatInputState>,
) {
    if keyboard.just_pressed(KeyCode::Escape) && state.open {
        state.open = false;
    }
    // `G` toggles the viewer's own graveyard (clicking a pile still browses
    // any player's). Suppressed while chat has the keyboard and while a
    // decision modal is up — ChooseColor binds G to green mana.
    if keyboard.just_pressed(KeyCode::KeyG)
        && !chat.open
        && view
            .0
            .as_ref()
            .is_some_and(|cv| cv.pending_decision.is_none())
    {
        let your_seat = view.0.as_ref().map(|cv| cv.your_seat).unwrap_or(0);
        if state.open && state.owner == your_seat {
            state.open = false;
        } else {
            state.open = true;
            state.owner = your_seat;
        }
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

        // (name, recast-badge, click-to-cast): badge is "Flashback {N}" /
        // "Retrace" for cards castable from the graveyard; when the browser
        // shows the viewer's own graveyard the tile also carries the recast
        // action so clicking it casts the card.
        let own_graveyard = view.0.as_ref().is_some_and(|cv| cv.your_seat == owner);
        let card_names: Vec<(String, Option<String>, Option<(crabomination::card::CardId, GraveyardRecast)>)> =
            view.0.as_ref()
            .map(|cv| cv.players[owner].graveyard.iter().map(|c| {
                let (badge, recast) = if let Some(fb) = &c.flashback_cost {
                    (Some(format!("Flashback {{{}}}", fb.cmc())), Some(GraveyardRecast::Flashback))
                } else if let Some(mh) = &c.mayhem_cost {
                    (Some(format!("Mayhem {{{}}}", mh.cmc())), Some(GraveyardRecast::Mayhem))
                } else if let Some(hm) = &c.harmonize_cost {
                    (Some(format!("Harmonize {{{}}}", hm.cmc())), Some(GraveyardRecast::Harmonize))
                } else if let Some(db) = &c.disturb_cost {
                    (Some(format!("Disturb {{{}}}", db.cmc())), Some(GraveyardRecast::Disturb))
                } else if let Some((esc, n)) = &c.escape {
                    (Some(format!("Escape {{{}}} +{n} exile", esc.cmc())), Some(GraveyardRecast::Escape(*n)))
                } else if c.retrace {
                    (Some("Retrace".to_string()), Some(GraveyardRecast::Retrace))
                } else {
                    (None, None)
                };
                (c.name.clone(), badge, recast.filter(|_| own_graveyard).map(|r| (c.id, r)))
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
                            for (name, badge, recast) in &card_names {
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
                                    GraveyardCardItem {
                                        name: name.clone(),
                                        recast: *recast,
                                    },
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
                    Text::new("Press Esc or click outside to close"),
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

/// Exile-zone browser: V toggles a popup listing every exiled card with
/// badges for the engine state that matters (face down, may-play grants,
/// Cipher encodes, linked "until X leaves" exiles). Mirrors the graveyard
/// browser's layout; tiles reuse `GraveyardCardItem` so the shared
/// hover-name tooltip works.
pub fn exile_browser(
    mut commands: Commands,
    view: Res<CurrentView>,
    asset_server: Res<AssetServer>,
    ui_fonts: Res<UiFonts>,
    existing: Query<Entity, With<ExileBrowser>>,
    keyboard: Res<ButtonInput<KeyCode>>,
    overlay_interaction: Query<&Interaction, (With<ExileBrowser>, With<Button>)>,
    chat: Res<crate::systems::chat::ChatInputState>,
) {
    let key_v = !chat.open && keyboard.just_pressed(KeyCode::KeyV);
    let close_requested = keyboard.just_pressed(KeyCode::Escape)
        || overlay_interaction
            .iter()
            .any(|i| *i == Interaction::Pressed);
    if !existing.is_empty() && (close_requested || key_v) {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
        return;
    }
    if !existing.is_empty() || !key_v {
        return;
    }
    let Some(cv) = view.0.as_ref() else { return };

    // (display name, badge, face_down) per exiled card.
    let entries: Vec<(String, Option<String>, bool)> = cv
        .exile
        .iter()
        .map(|c| {
            let mut badges: Vec<String> = Vec::new();
            if c.face_down {
                badges.push("Face down".to_string());
            }
            if let Some(p) = c.may_play_recipient {
                let who = if p == cv.your_seat { "you".to_string() } else { format!("P{p}") };
                // Surface airbend's flat {2} / miracle / Hostage-Taker alt cost.
                badges.push(match c.may_play_alt_cost {
                    Some(n) => format!("May play ({who}) for {{{n}}}"),
                    None => format!("May play ({who})"),
                });
            }
            if c.encoded_on.is_some() {
                badges.push("Cipher: encoded".to_string());
            }
            if c.exiled_by.is_some() {
                badges.push("Returns when exiler leaves".to_string());
            }
            let badge = (!badges.is_empty()).then(|| badges.join(" · "));
            (c.name.clone(), badge, c.face_down)
        })
        .collect();
    let count = entries.len();
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
            ExileBrowser,
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

    commands.entity(panel).with_children(|panel| {
        panel.spawn((
            Text::new(format!("Exile ({count} cards)")),
            ui_fonts.tf(18.0),
            TextColor(theme::TEXT_PRIMARY),
            Pickable::IGNORE,
        ));
        if entries.is_empty() {
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
                    for (name, badge, face_down) in &entries {
                        let tile_children = |tile: &mut ChildSpawnerCommands| {
                            if !face_down {
                                let texture: Handle<Image> =
                                    asset_server.load(scryfall::card_asset_path(name));
                                tile.spawn((
                                    ImageNode { image: texture, ..default() },
                                    Node {
                                        width: Val::Percent(100.0),
                                        height: Val::Percent(100.0),
                                        ..default()
                                    },
                                    Pickable::IGNORE,
                                ));
                            } else {
                                // Masked card: a blank tile with the name.
                                tile.spawn((
                                    Text::new(name.clone()),
                                    ui_fonts.tf(12.0),
                                    TextColor(theme::TEXT_SECONDARY),
                                    Pickable::IGNORE,
                                ));
                            }
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
                        };
                        grid.spawn((
                            Button,
                            Node {
                                width: Val::Px(BROWSER_CARD_WIDTH),
                                height: Val::Px(BROWSER_CARD_HEIGHT),
                                ..default()
                            },
                            GraveyardCardItem { name: name.clone(), recast: None },
                        ))
                        .with_children(tile_children);
                    }
                });
        }
        panel.spawn((
            Text::new("Press V or Esc, or click outside to close"),
            ui_fonts.tf(11.0),
            TextColor(theme::TEXT_MUTED),
            Pickable::IGNORE,
        ));
    });
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

/// Click-to-cast for the viewer's own castable graveyard tiles: submits the
/// recast action carried by the tile (targets are auto-chosen server-side;
/// Escape auto-picks its exile fodder) and closes the browser.
pub fn graveyard_recast_click(
    items: Query<(&Interaction, &GraveyardCardItem), Changed<Interaction>>,
    mut state: ResMut<GraveyardBrowserState>,
    view: Res<CurrentView>,
    outbox: Option<Res<crate::net_plugin::NetOutbox>>,
) {
    let Some(outbox) = outbox else { return };
    for (interaction, item) in &items {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let Some((card_id, recast)) = &item.recast else { continue };
        use crabomination::game::GameAction;
        let action = match recast {
            GraveyardRecast::Flashback => GameAction::CastFlashback {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            GraveyardRecast::Mayhem => GameAction::CastMayhem {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            GraveyardRecast::Harmonize => GameAction::CastHarmonize {
                card_id: *card_id, tap_creature: None, target: None,
                additional_targets: vec![], mode: None, x_value: None,
            },
            GraveyardRecast::Disturb => GameAction::CastDisturb {
                card_id: *card_id, target: None, additional_targets: vec![],
            },
            GraveyardRecast::Retrace => GameAction::CastRetrace {
                card_id: *card_id, target: None, additional_targets: vec![],
                mode: None, x_value: None,
            },
            GraveyardRecast::Escape(n) => {
                // Auto-pick the first N *other* cards of the viewer's
                // graveyard as the exile payment.
                let exile_cards: Vec<crabomination::card::CardId> = view.0.as_ref()
                    .map(|cv| {
                        cv.players[state.owner].graveyard.iter()
                            .filter(|c| c.id != *card_id)
                            .take(*n as usize)
                            .map(|c| c.id)
                            .collect()
                    })
                    .unwrap_or_default();
                if exile_cards.len() < *n as usize {
                    continue; // not enough fodder — leave the browser open
                }
                GameAction::CastEscape {
                    card_id: *card_id, exile_cards, target: None,
                    additional_targets: vec![], mode: None, x_value: None,
                }
            }
        };
        outbox.submit(action);
        state.open = false;
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
    browser: Query<(), Or<(With<GraveyardBrowser>, With<ExileBrowser>)>>,
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

#[cfg(test)]
mod tests {
    use super::{
        hover_info_lines, live_override_lines, preview_anchor, HOVER_PREVIEW_HEIGHT,
        HOVER_PREVIEW_MARGIN, HOVER_PREVIEW_WIDTH,
    };
    use bevy::math::Vec2;
    use crabomination::card::CreatureType;

    /// An Unearth ability's hover line is annotated as a sorcery-speed
    /// graveyard ability.
    #[test]
    fn unearth_ability_line_notes_graveyard_and_sorcery_speed() {
        let lines = hover_info_lines("Viscera Dragger");
        assert!(
            lines.iter().any(|(t, _)| t.contains("from your graveyard")
                && t.contains("sorcery speed only")),
            "expected a graveyard / sorcery-speed annotated line, got {lines:?}"
        );
    }

    #[test]
    fn override_lines_note_fish_and_lost_abilities() {
        // Serra Angel turned into a 0/1 Fish with no abilities (Ichthyomorphosis).
        let lines = live_override_lines(
            "Serra Angel",
            true,
            &[CreatureType::Fish],
            &[crabomination::mana::Color::Blue],
        );
        assert!(lines.iter().any(|(t, _)| t.contains("abilities removed")));
        assert!(lines.iter().any(|(t, _)| t.contains("Fish")));
        assert!(lines.iter().any(|(t, _)| t.contains("blue")));
    }

    #[test]
    fn override_lines_quiet_for_unchanged_creature() {
        // A Grizzly Bears still a Bear with its abilities → no override notes.
        let lines = live_override_lines(
            "Grizzly Bears",
            false,
            &[CreatureType::Bear],
            &[crabomination::mana::Color::Green],
        );
        assert!(lines.is_empty(), "no notes when nothing diverges");
    }

    const PW: f32 = HOVER_PREVIEW_WIDTH;
    const PH: f32 = HOVER_PREVIEW_HEIGHT;
    const M: f32 = HOVER_PREVIEW_MARGIN;

    #[test]
    fn preview_sits_right_of_cursor_on_left_half() {
        let win = Vec2::new(1600.0, 1000.0);
        let (x, _) = preview_anchor(Vec2::new(200.0, 500.0), win, PW, PH, M);
        // Cursor on the left → preview to its right, never covering the card.
        assert!(x >= 200.0, "expected preview right of cursor, got x={x}");
    }

    #[test]
    fn preview_sits_left_of_cursor_on_right_half() {
        let win = Vec2::new(1600.0, 1000.0);
        let cursor_x = 1400.0;
        let (x, _) = preview_anchor(Vec2::new(cursor_x, 500.0), win, PW, PH, M);
        // Cursor on the right → preview's right edge is left of the cursor.
        assert!(x + PW <= cursor_x, "expected preview left of cursor, got x={x}");
    }

    #[test]
    fn preview_stays_within_viewport() {
        let win = Vec2::new(1600.0, 1000.0);
        // Sweep cursor across extreme positions; the card must never clip.
        for &(cx, cy) in &[
            (0.0, 0.0),
            (1600.0, 1000.0),
            (0.0, 1000.0),
            (1600.0, 0.0),
            (800.0, 500.0),
        ] {
            let (x, y) = preview_anchor(Vec2::new(cx, cy), win, PW, PH, M);
            assert!(x >= M && x + PW <= win.x - M + f32::EPSILON, "x out of bounds: {x}");
            assert!(y >= M && y + PH <= win.y - M + f32::EPSILON, "y out of bounds: {y}");
        }
    }

    #[test]
    fn preview_centers_vertically_on_cursor_when_room() {
        let win = Vec2::new(1600.0, 1000.0);
        let (_, y) = preview_anchor(Vec2::new(200.0, 500.0), win, PW, PH, M);
        // Mid-screen cursor → card centered on it (top = cursor.y - ph/2).
        assert!((y - (500.0 - PH * 0.5)).abs() < 0.5, "expected centered, got y={y}");
    }
}

// ── Low-life vignette ─────────────────────────────────────────────────────────

/// Marker for the screen-edge danger frame shown when the viewer's life
/// is critically low.
#[derive(Component)]
pub struct LowLifeVignette;

/// Life total at or below which the danger frame shows. Flat threshold —
/// "one burn spell from dead" reads the same in every format.
const LOW_LIFE_THRESHOLD: i32 = 5;

/// Pulse a red frame around the screen edge while the viewer's life is
/// critically low — a peripheral-vision "you are dying" signal that
/// doesn't depend on reading the life total.
pub fn low_life_vignette(
    mut commands: Commands,
    view: Res<CurrentView>,
    time: Res<Time>,
    mut existing: Query<(Entity, &mut BorderColor), With<LowLifeVignette>>,
) {
    let low = view
        .0
        .as_ref()
        .filter(|cv| cv.game_over.is_none())
        .and_then(|cv| cv.players.iter().find(|p| p.seat == cv.your_seat))
        .is_some_and(|p| p.life <= LOW_LIFE_THRESHOLD);

    match (low, existing.iter_mut().next()) {
        (true, Some((_, mut border))) => {
            let pulse = 0.25 + 0.20 * (time.elapsed_secs() * std::f32::consts::TAU / 1.6).sin().abs();
            *border = BorderColor::all(Color::srgba(0.85, 0.10, 0.10, pulse));
        }
        (true, None) => {
            commands.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(0.0),
                    top: Val::Px(0.0),
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    border: UiRect::all(Val::Px(7.0)),
                    ..default()
                },
                BorderColor::all(Color::srgba(0.85, 0.10, 0.10, 0.3)),
                LowLifeVignette,
                Pickable::IGNORE,
                crate::systems::game_ui::InGameRoot,
                GlobalZIndex(35),
            ));
        }
        (false, Some((e, _))) => {
            commands.entity(e).despawn();
        }
        (false, None) => {}
    }
}
