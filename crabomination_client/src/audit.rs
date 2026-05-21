//! Card-audit mode: lets a developer step through every implemented
//! card, boot a tailored test match for it, and record which cards
//! have been hand-verified.
//!
//! Flow:
//! 1. Menu → "Audit Cards" → [`AppState::Audit`].
//! 2. The picker shows every card factory from the cube + SoS + demo
//!    pools (deduped by function pointer), one row per card with a ✓
//!    next to verified ones. Clicking a row writes the card name into
//!    [`AuditTarget`] and transitions to `AppState::InGame`.
//! 3. The in-game match-builder sees `AuditTarget::Some(...)` and
//!    constructs a state via [`build_audit_state`]: viewer starts
//!    with 5 of each basic land on the battlefield, the target card
//!    in hand, and an opponent stocked with chump blockers / targets.
//! 4. A "Mark Verified" HUD button writes the card name to
//!    `<repo>/debug/audited_cards.json` and returns to the picker.
//!
//! Persistence is plain JSON (one sorted list of card names) so the
//! file is human-readable and easy to grep/edit between sessions.

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::Mutex;

use bevy::prelude::*;
use crabomination::card::CardDefinition;
use crabomination::catalog::{forest, grizzly_bears, island, mountain, plains, swamp};
use crabomination::cube::all_cube_cards;
use crabomination::game::GameState;
use crabomination::player::Player;
use crabomination::sos_mode::all_sos_cards;

/// Engine-side card factory type. Defined locally because the cube /
/// sos modules each define their own private alias and we don't want
/// to widen their privacy just for the audit catalog.
type CardFactory = fn() -> CardDefinition;

/// Set by the audit picker before transitioning to `AppState::InGame`.
/// `start_net_session_from_menu` consumes it to build an audit-tailored
/// `GameState`; the in-game HUD checks it to decide whether to render
/// the "Mark Verified" button.
#[derive(Resource, Default, Clone)]
pub struct AuditTarget(pub Option<String>);

/// In-memory mirror of the persisted set, loaded once at startup and
/// kept in sync as the user marks cards verified.
#[derive(Resource, Default, Clone)]
pub struct AuditedCards(pub BTreeSet<String>);

/// Path of the persistence file. We use the same `debug/` directory the
/// state-export feature writes to, so it shows up next to the JSON
/// snapshots the developer is already debugging from.
pub fn audit_file_path() -> PathBuf {
    PathBuf::from("debug").join("audited_cards.json")
}

/// Best-effort load — missing file or parse error returns an empty set
/// so the first-time experience just shows everything unverified.
pub fn load_audited_cards() -> AuditedCards {
    let path = audit_file_path();
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return AuditedCards::default();
    };
    let names: Vec<String> = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("audit: failed to parse {} ({e}); starting fresh", path.display());
            return AuditedCards::default();
        }
    };
    AuditedCards(names.into_iter().collect())
}

/// Persist the current verified set as a sorted JSON array. Best-effort
/// — failures are logged but don't crash the app, since losing the
/// audit log is recoverable (re-verify a handful of cards).
pub fn save_audited_cards(set: &AuditedCards) {
    let path = audit_file_path();
    if let Some(dir) = path.parent()
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        eprintln!("audit: failed to create {} ({e})", dir.display());
        return;
    }
    let names: Vec<&String> = set.0.iter().collect();
    match serde_json::to_string_pretty(&names) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&path, json) {
                eprintln!("audit: failed to write {} ({e})", path.display());
            }
        }
        Err(e) => eprintln!("audit: failed to encode JSON ({e})"),
    }
}

/// Card-pool tag used by the audit picker's "filter by set" row. The
/// engine doesn't tag `CardDefinition`s with an expansion code, but the
/// audit catalog is itself assembled from a handful of named pools
/// (cube, SoS Strixhaven, the demo decks) — that grouping is what the
/// `AuditPool` filter exposes. A card can belong to multiple pools
/// (e.g. cube + demo) — the row's `pools` field carries every pool
/// that contains it.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum AuditPool {
    Cube,
    Sos,
    BrgDemo,
    GoryoDemo,
    RofellosCommander,
}

impl AuditPool {
    pub const ALL: &'static [AuditPool] = &[
        AuditPool::Cube,
        AuditPool::Sos,
        AuditPool::BrgDemo,
        AuditPool::GoryoDemo,
        AuditPool::RofellosCommander,
    ];

    pub fn label(self) -> &'static str {
        match self {
            AuditPool::Cube => "Cube",
            AuditPool::Sos => "SoS",
            AuditPool::BrgDemo => "BRG Demo",
            AuditPool::GoryoDemo => "Goryo Demo",
            AuditPool::RofellosCommander => "Rofellos",
        }
    }
}

/// One catalog entry: card display name + factory + the pools it
/// belongs to. The picker filters rows by `pools` and orders them by
/// name. Sorted/deduped at first-build time and cached for the
/// session.
#[derive(Clone)]
pub struct CatalogEntry {
    pub name: String,
    pub factory: CardFactory,
    pub pools: Vec<AuditPool>,
}

/// Sets which pool the picker is currently filtered to. `None` shows
/// every catalog entry; `Some(pool)` keeps only entries whose
/// `pools` contains it. Lives as a `Resource` so the picker can read
/// and write it from button-click handlers without a side channel.
#[derive(Resource, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuditPoolFilter(pub Option<AuditPool>);

/// Catalog cache: deferred-build because `all_cube_cards` allocates
/// and we want a fresh list per audit session in case the catalog
/// grows mid-build.
static CATALOG_CACHE: Mutex<Option<Vec<CatalogEntry>>> = Mutex::new(None);

pub fn catalog() -> Vec<CatalogEntry> {
    let mut guard = CATALOG_CACHE.lock().expect("audit catalog mutex");
    if guard.is_none() {
        // Build a (factory_ptr → set-of-pools) map so a card that
        // appears in multiple pools (cube + demo, say) keeps all of
        // its memberships rather than losing the later ones to dedup.
        let mut pools_by_fn: std::collections::HashMap<usize, Vec<AuditPool>> =
            std::collections::HashMap::new();
        let mut order: Vec<CardFactory> = Vec::new();

        let mut tag = |factories: &[CardFactory], pool: AuditPool| {
            for &f in factories {
                let key = f as usize;
                let entry = pools_by_fn.entry(key).or_default();
                if !entry.contains(&pool) {
                    entry.push(pool);
                }
                if !order.iter().any(|g| *g as usize == key) {
                    order.push(f);
                }
            }
        };
        let cube: Vec<CardFactory> = all_cube_cards();
        tag(&cube, AuditPool::Cube);
        let sos: Vec<CardFactory> = all_sos_cards();
        tag(&sos, AuditPool::Sos);
        tag(crabomination::demo::brg_combo_deck(), AuditPool::BrgDemo);
        tag(crabomination::demo::goryos_vengeance_deck(), AuditPool::GoryoDemo);
        tag(
            crabomination::demo::rofellos_commander_main(),
            AuditPool::RofellosCommander,
        );

        let mut entries: Vec<CatalogEntry> = order
            .into_iter()
            .map(|f| CatalogEntry {
                name: f().name.to_string(),
                factory: f,
                pools: pools_by_fn
                    .get(&(f as usize))
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect();
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        entries.dedup_by(|a, b| a.name == b.name);
        *guard = Some(entries);
    }
    guard.as_ref().unwrap().clone()
}

/// Build an audit-tailored two-player state for `card_name`:
///
/// * P0 (viewer): 20 life, 5 of each basic land on the battlefield
///   untapped (so any colour of mana is one click away), the target
///   card in hand, 6 more basic lands padding the hand, and a library
///   of basic lands so the game doesn't deck-out on draw.
/// * P1 (bot): 20 life, 4 of each basic land + a couple of vanilla
///   creatures on the battlefield so spells that want a target
///   (creature- or player-target) have something to point at.
///
/// Returns `None` when `card_name` doesn't match any factory in the
/// catalog — keeps callers from panicking on a stale persisted entry.
pub fn build_audit_state(card_name: &str) -> Option<GameState> {
    let cat = catalog();
    let factory = cat
        .iter()
        .find(|e| e.name == card_name)
        .map(|e| e.factory)?;

    let mut state = GameState::new(vec![
        Player::new(0, "Auditor"),
        Player::new(1, "Dummy"),
    ]);

    // ── Viewer side ─────────────────────────────────────────────────
    // 5 of each basic land in play, untapped.
    for land in [plains, island, swamp, mountain, forest] {
        for _ in 0..5 {
            state.add_card_to_battlefield(0, land());
        }
    }
    // Target card in hand.
    state.add_card_to_hand(0, factory());
    // A few extra basics in hand so the hand isn't suspiciously empty.
    for _ in 0..6 {
        state.add_card_to_hand(0, forest());
    }
    // Library padding — basic lands won't crash the engine and avoid
    // any "library out" state-based action mid-audit.
    for _ in 0..40 {
        state.add_card_to_library(0, forest());
    }

    // ── Opponent side ─────────────────────────────────────────────
    // 4 of each basic land in play so the bot can in theory cast
    // counterspells / responses against the audit card. (Whether the
    // bot does anything useful is a separate concern.)
    for land in [plains, island, swamp, mountain, forest] {
        for _ in 0..4 {
            state.add_card_to_battlefield(1, land());
        }
    }
    // A couple of vanilla 2/2 creatures (Grizzly Bears) so
    // single-target spells have something legal to point at.
    state.add_card_to_battlefield(1, grizzly_bears());
    state.add_card_to_battlefield(1, grizzly_bears());
    for _ in 0..40 {
        state.add_card_to_library(1, plains());
    }

    state.players[0].wants_ui = true;
    state.players[1].wants_ui = true;

    Some(state)
}

/// Convenience: mark a card as verified and persist immediately.
/// Returns `true` if the card was newly added, `false` if it was
/// already in the set.
pub fn mark_verified(set: &mut AuditedCards, card_name: String) -> bool {
    let inserted = set.0.insert(card_name);
    if inserted {
        save_audited_cards(set);
    }
    inserted
}

/// Inverse: drop a card from the verified set and persist. Useful
/// when the user discovers a previously-verified card actually broke
/// after a subsequent change.
pub fn unmark_verified(set: &mut AuditedCards, card_name: &str) -> bool {
    let removed = set.0.remove(card_name);
    if removed {
        save_audited_cards(set);
    }
    removed
}

// ─── Picker UI ───────────────────────────────────────────────────────

use crate::menu::{AppState, PendingNetMode, NetMode, MatchFormat};
use crate::theme::{self, HoverTint, UiFonts, RADIUS_BUTTON, RADIUS_PANEL};

/// Root entity for the audit picker; despawned `OnExit(Audit)`.
#[derive(Component)]
pub struct AuditPickerRoot;

/// One row in the picker — clicking it sets `AuditTarget` and
/// transitions into `InGame`.
#[derive(Component)]
pub struct AuditPickRow {
    pub card_name: String,
}

#[derive(Component)]
pub struct AuditBackButton;

#[derive(Component)]
pub struct AuditUnverifyRow {
    pub card_name: String,
}

/// Pool-filter toggle button. `None` means "show every catalog
/// entry"; `Some(pool)` filters to entries containing that pool.
#[derive(Component)]
pub struct AuditPoolButton(pub Option<AuditPool>);

/// System: spawn the picker on `OnEnter(Audit)`.
pub fn spawn_audit_picker(
    mut commands: Commands,
    ui_fonts: Res<UiFonts>,
    verified: Res<AuditedCards>,
    filter: Res<AuditPoolFilter>,
) {
    build_picker_ui(&mut commands, &ui_fonts, &verified, *filter);
}

/// Picker-spawning core. Extracted out of `spawn_audit_picker` so the
/// unverify and pool-filter handlers can rebuild the panel without
/// going through the state-machine re-entry dance.
fn build_picker_ui(
    commands: &mut Commands,
    ui_fonts: &UiFonts,
    verified: &AuditedCards,
    filter: AuditPoolFilter,
) {
    let tf = |size: f32| ui_fonts.tf(size);
    let all_cards = catalog();
    // Filter the visible rows by the selected pool. `None` keeps
    // everything; counts always reflect the post-filter view so the
    // header reads "x/y verified" within the current scope.
    let cards: Vec<CatalogEntry> = match filter.0 {
        None => all_cards.clone(),
        Some(pool) => all_cards
            .iter()
            .filter(|e| e.pools.contains(&pool))
            .cloned()
            .collect(),
    };
    let total = cards.len();
    let done = cards.iter().filter(|e| verified.0.contains(&e.name)).count();

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
            AuditPickerRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(20.0)),
                    row_gap: Val::Px(10.0),
                    align_items: AlignItems::Stretch,
                    width: Val::Px(560.0),
                    max_height: Val::Percent(90.0),
                    border_radius: BorderRadius::all(RADIUS_PANEL),
                    ..default()
                },
                BackgroundColor(theme::PANEL_BG),
            ))
            .with_children(|panel| {
                // Header row: title + verified counter.
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|hdr| {
                    hdr.spawn((
                        Text::new("Card Audit"),
                        tf(22.0),
                        TextColor(theme::ACCENT_GOLD),
                    ));
                    hdr.spawn((
                        Text::new(format!("{done}/{total} verified")),
                        tf(13.0),
                        TextColor(theme::TEXT_SECONDARY),
                    ));
                });

                panel.spawn((
                    Text::new(
                        "Click a card to launch a tailored test match (5 of each basic in play, target in hand).",
                    ),
                    tf(12.0),
                    TextColor(theme::TEXT_MUTED),
                ));

                // Pool filter row. "All" (None) plus one button per
                // pool. Selected pool reads as the green active tint.
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(6.0),
                    flex_wrap: FlexWrap::Wrap,
                    row_gap: Val::Px(6.0),
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("Set:"),
                        tf(12.0),
                        TextColor(theme::TEXT_SECONDARY),
                    ));
                    let pool_button = |row: &mut ChildSpawnerCommands,
                                       pool: Option<AuditPool>,
                                       label: &str| {
                        let active = filter.0 == pool;
                        let bg = if active {
                            Color::srgba(0.18, 0.30, 0.20, 1.0)
                        } else {
                            theme::BUTTON_NEUTRAL_BG
                        };
                        row.spawn((
                            Button,
                            Node {
                                padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                                border_radius: BorderRadius::all(RADIUS_BUTTON),
                                ..default()
                            },
                            BackgroundColor(bg),
                            HoverTint::new(bg),
                            AuditPoolButton(pool),
                        ))
                        .with_children(|b| {
                            b.spawn((
                                Text::new(label.to_string()),
                                tf(12.0),
                                TextColor(theme::TEXT_PRIMARY),
                                Pickable::IGNORE,
                            ));
                        });
                    };
                    pool_button(row, None, "All");
                    for &pool in AuditPool::ALL {
                        pool_button(row, Some(pool), pool.label());
                    }
                });

                // Scrollable list.
                panel.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        overflow: Overflow::scroll_y(),
                        padding: UiRect::axes(Val::Px(2.0), Val::Px(4.0)),
                        flex_grow: 1.0,
                        min_height: Val::Px(360.0),
                        border_radius: BorderRadius::all(RADIUS_BUTTON),
                        ..default()
                    },
                    BackgroundColor(theme::PANEL_BG_SUNKEN),
                ))
                .with_children(|list| {
                    for entry in &cards {
                        let name = &entry.name;
                        let is_done = verified.0.contains(name);
                        let row_bg = if is_done {
                            Color::srgba(0.18, 0.30, 0.20, 1.0)
                        } else {
                            theme::PANEL_BG_RAISED
                        };
                        list.spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(8.0),
                            align_items: AlignItems::Center,
                            ..default()
                        })
                        .with_children(|row| {
                            // Main pick button (full width minus the
                            // unverify button next to it).
                            row.spawn((
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                                    flex_grow: 1.0,
                                    border_radius: BorderRadius::all(RADIUS_BUTTON),
                                    ..default()
                                },
                                BackgroundColor(row_bg),
                                HoverTint::new(row_bg),
                                AuditPickRow {
                                    card_name: name.clone(),
                                },
                            ))
                            .with_children(|b| {
                                let check = if is_done { "✓  " } else { "    " };
                                let pools_tag = if entry.pools.is_empty() {
                                    String::new()
                                } else {
                                    let labels: Vec<&str> =
                                        entry.pools.iter().map(|p| p.label()).collect();
                                    format!("   [{}]", labels.join(", "))
                                };
                                b.spawn((
                                    Text::new(format!("{}{}{}", check, name, pools_tag)),
                                    tf(13.0),
                                    TextColor(theme::TEXT_PRIMARY),
                                    Pickable::IGNORE,
                                ));
                            });

                            // Unverify button — only useful when the
                            // card is currently in the verified set.
                            if is_done {
                                row.spawn((
                                    Button,
                                    Node {
                                        padding: UiRect::axes(Val::Px(8.0), Val::Px(6.0)),
                                        border_radius: BorderRadius::all(RADIUS_BUTTON),
                                        ..default()
                                    },
                                    BackgroundColor(theme::BUTTON_DANGER_BG),
                                    HoverTint::new(theme::BUTTON_DANGER_BG),
                                    AuditUnverifyRow {
                                        card_name: name.clone(),
                                    },
                                ))
                                .with_children(|b| {
                                    b.spawn((
                                        Text::new("×"),
                                        tf(13.0),
                                        TextColor(theme::TEXT_PRIMARY),
                                        Pickable::IGNORE,
                                    ));
                                });
                            }
                        });
                    }
                });

                // Back button.
                panel.spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::FlexEnd,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(8.0)),
                            border_radius: BorderRadius::all(RADIUS_BUTTON),
                            ..default()
                        },
                        BackgroundColor(theme::BUTTON_NEUTRAL_BG),
                        HoverTint::new(theme::BUTTON_NEUTRAL_BG),
                        AuditBackButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Back to Menu"),
                            tf(14.0),
                            TextColor(theme::TEXT_PRIMARY),
                            Pickable::IGNORE,
                        ));
                    });
                });
            });
        });
}

pub fn despawn_audit_picker(
    mut commands: Commands,
    roots: Query<Entity, With<AuditPickerRoot>>,
) {
    for e in &roots {
        commands.entity(e).despawn();
    }
}

/// Click a pick row → set `AuditTarget` and boot the in-game scene.
/// Click the back button → return to the main menu.
/// Click an unverify ×  → drop that card from the verified set and
/// re-spawn the picker so the row repaints.
/// Click a pool-filter button → change `AuditPoolFilter` and respawn.
#[allow(clippy::too_many_arguments)]
pub fn handle_audit_picker(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut pending: ResMut<PendingNetMode>,
    mut target: ResMut<AuditTarget>,
    mut verified: ResMut<AuditedCards>,
    mut filter: ResMut<AuditPoolFilter>,
    pick_q: Query<(&Interaction, &AuditPickRow), Changed<Interaction>>,
    back_q: Query<&Interaction, (Changed<Interaction>, With<AuditBackButton>)>,
    unverify_q: Query<(&Interaction, &AuditUnverifyRow), Changed<Interaction>>,
    pool_q: Query<(&Interaction, &AuditPoolButton), Changed<Interaction>>,
    picker_roots: Query<Entity, With<AuditPickerRoot>>,
    ui_fonts: Res<UiFonts>,
) {
    if back_q.iter().any(|i| *i == Interaction::Pressed) {
        next_state.set(AppState::Menu);
        return;
    }
    for (interaction, btn) in &pool_q {
        if *interaction == Interaction::Pressed && filter.0 != btn.0 {
            filter.0 = btn.0;
            for e in &picker_roots {
                commands.entity(e).despawn();
            }
            build_picker_ui(&mut commands, &ui_fonts, &verified, *filter);
            return;
        }
    }
    for (interaction, row) in &unverify_q {
        if *interaction == Interaction::Pressed {
            unmark_verified(&mut verified, &row.card_name);
            // Repaint by despawning and respawning the picker. Cheap
            // since the panel rebuilds every entry from the catalog.
            for e in &picker_roots {
                commands.entity(e).despawn();
            }
            build_picker_ui(&mut commands, &ui_fonts, &verified, *filter);
            return;
        }
    }
    for (interaction, row) in &pick_q {
        if *interaction == Interaction::Pressed {
            target.0 = Some(row.card_name.clone());
            pending.0 = Some((NetMode::LocalBot, MatchFormat::Modern));
            next_state.set(AppState::InGame);
            return;
        }
    }
}
