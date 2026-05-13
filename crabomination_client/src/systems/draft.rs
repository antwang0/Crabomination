//! 8-player booster-draft client flow against 7 bots.
//!
//! Bevy systems wired up at `AppState::Drafting`. Three sequential
//! sub-phases live behind a single `DraftSession` resource:
//!
//! 1. **Drafting** — current pack laid out as a 5×3 grid; the user
//!    clicks a card to pick it. All 7 bots auto-pick simultaneously
//!    via `crabomination::draft::bot_pick`. Packs pass left (rounds
//!    1 + 3) or right (round 2). After 3 packs × 15 picks, transitions
//!    to deckbuilding.
//!
//! 2. **Deckbuilding** — main / sideboard tabs of all 45 picks plus
//!    +/- buttons for each basic land color. Clicking a card moves it
//!    between main and sideboard. The "Confirm Deck" button activates
//!    once the main deck has at least 40 cards (drafted + basics).
//!
//! 3. **Opponent select** — the 7 bots' decks are shown by color
//!    identity ("P3 (UR)", "P4 (BG)", …). Clicking one writes the
//!    chosen player + opponent decks into the `DraftedDecks` resource
//!    and transitions to `AppState::InGame`. `start_net_session_from_menu`
//!    reads `DraftedDecks` instead of rerolling a fresh state.
//!
//! The whole flow is client-side — no server / wire protocol. The
//! draft picks happen in-process and the resulting decks are fed
//! directly into the existing in-process bot match infrastructure.

use std::collections::HashMap;

use bevy::prelude::*;

use crabomination::cube::CardFactory;
use crabomination::draft::{
    PACKS_PER_SEAT, PACK_SIZE, POD_SIZE, basic_land_factory, bot_pick, draft_pool,
    enforce_copy_cap, generate_pack, suggest_basic_split, suggest_main_deck, top_two_colors,
};
// Bevy's `Color` lives in the prelude (`use bevy::prelude::*` above).
// Import the engine's mana color under an alias to keep the two
// distinct everywhere in this file.
use crabomination::mana::Color as ManaColor;

use crate::menu::{AppState, DraftedDecks};
use crate::scryfall;

// ── Constants ────────────────────────────────────────────────────────────────

const PANEL_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.97);
const HEADER_BG: Color = Color::srgba(0.10, 0.10, 0.18, 1.0);
const STATS_BG: Color = Color::srgba(0.08, 0.08, 0.14, 1.0);
const TILE_BG: Color = Color::srgba(0.16, 0.16, 0.22, 1.0);
const TAB_BG_OFF: Color = Color::srgba(0.16, 0.16, 0.22, 1.0);
const TAB_BG_ON: Color = Color::srgba(0.45, 0.30, 0.15, 1.0);
const ACCENT_GOLD: Color = Color::srgb(1.0, 0.85, 0.55);
const ACCENT_GREEN: Color = Color::srgb(0.30, 0.70, 0.35);
const PICK_CARD_W: f32 = 170.0;
const PICK_CARD_H: f32 = PICK_CARD_W * (88.0 / 63.0);
const PICKS_TAB_CARD_W: f32 = 120.0;
const PICKS_TAB_CARD_H: f32 = PICKS_TAB_CARD_W * (88.0 / 63.0);
const DECKBUILD_CARD_W: f32 = 90.0;
const DECKBUILD_CARD_H: f32 = DECKBUILD_CARD_W * (88.0 / 63.0);
/// Mouse-wheel scroll speed. One line ≈ this many pixels along the
/// scrollable axis. `MouseScrollUnit::Line` deltas (typical hardware
/// mice) get this scaled directly; `Pixel` deltas (touchpads /
/// high-resolution wheels) are passed through 1:1.
const SCROLL_LINE_PX: f32 = 60.0;

// ── Resources / phase enum ───────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DraftPhase {
    Drafting,
    Deckbuilding,
    OpponentSelect,
}

/// Which view the user is currently looking at on the drafting
/// screen. The pack tab shows the current 15-card booster (large
/// tiles, click to pick); the picks tab shows everything they've
/// drafted so far (smaller tiles, view-only).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DraftTab {
    #[default]
    Pack,
    Picks,
}

/// In-progress draft state. Spawned `OnEnter(AppState::Drafting)`,
/// removed `OnExit`. Owns the per-seat pack + pick piles for the
/// drafting phase and the staged main / sideboard / basics during
/// deckbuilding.
#[derive(Resource)]
pub struct DraftSession {
    pub pool: Vec<CardFactory>,
    pub phase: DraftPhase,

    // Drafting state
    pub user_seat: usize,
    pub pack_round: u32,
    pub pick_in_round: u32,
    pub packs: Vec<Vec<CardFactory>>,
    pub picks: Vec<Vec<CardFactory>>,
    pub current_tab: DraftTab,

    // Deckbuilding state (only populated once draft completes)
    pub main: Vec<CardFactory>,
    pub sideboard: Vec<CardFactory>,
    pub basics: HashMap<ManaColor, u32>,
    pub player_colors: [ManaColor; 2],

    // Opponent select
    pub chosen_opponent: Option<usize>,
}

impl DraftSession {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        let pool = draft_pool();
        let packs: Vec<Vec<CardFactory>> = (0..POD_SIZE)
            .map(|_| generate_pack(&pool, &mut rng))
            .collect();
        let picks: Vec<Vec<CardFactory>> = (0..POD_SIZE).map(|_| Vec::new()).collect();
        Self {
            pool,
            phase: DraftPhase::Drafting,
            user_seat: 0,
            pack_round: 1,
            pick_in_round: 1,
            packs,
            picks,
            current_tab: DraftTab::Pack,
            main: Vec::new(),
            sideboard: Vec::new(),
            basics: HashMap::new(),
            player_colors: [ManaColor::White, ManaColor::Blue],
            chosen_opponent: None,
        }
    }

    pub fn user_pack(&self) -> &[CardFactory] {
        &self.packs[self.user_seat]
    }

    /// Advance one pick: user picks `user_pack_index`, all 7 bots
    /// auto-pick from their packs, then packs pass according to the
    /// current round's direction. If packs empty out, opens the next
    /// round of packs (or transitions phases on round 4).
    pub fn process_user_pick(&mut self, user_pack_index: usize) {
        if user_pack_index >= self.packs[self.user_seat].len() {
            return;
        }
        // 1. User picks.
        let card = self.packs[self.user_seat].remove(user_pack_index);
        self.picks[self.user_seat].push(card);

        // 2. Each bot auto-picks from its current pack.
        for seat in 0..POD_SIZE {
            if seat == self.user_seat {
                continue;
            }
            let pack = &self.packs[seat];
            if let Some(idx) = bot_pick(pack, &self.picks[seat]) {
                let card = self.packs[seat].remove(idx);
                self.picks[seat].push(card);
            }
        }

        // 3. Pass packs (left for rounds 1 + 3, right for round 2),
        //    or open new packs if all packs are empty.
        let any_card_left = self.packs.iter().any(|p| !p.is_empty());
        if any_card_left {
            self.pass_packs();
            self.pick_in_round += 1;
        } else {
            // Round complete.
            self.pack_round += 1;
            self.pick_in_round = 1;
            if self.pack_round > PACKS_PER_SEAT {
                self.transition_to_deckbuilding();
            } else {
                self.open_new_round();
            }
        }
    }

    fn pass_packs(&mut self) {
        // Round 1 + 3: pass left  (seat N → seat N-1).
        // Round 2: pass right     (seat N → seat N+1).
        // Indexing-wise, "pass left" means seat N's pack moves to
        // seat N-1's hands, so we rotate the `packs` vec right (the
        // pack that was at index N ends up at index N-1).
        let pass_left = self.pack_round != 2;
        if pass_left {
            self.packs.rotate_left(1);
        } else {
            self.packs.rotate_right(1);
        }
    }

    fn open_new_round(&mut self) {
        let mut rng = rand::rng();
        for pack in self.packs.iter_mut() {
            *pack = generate_pack(&self.pool, &mut rng);
        }
    }

    fn transition_to_deckbuilding(&mut self) {
        // Auto-suggest main + sideboard split from the user's picks.
        let user_picks = self.picks[self.user_seat].clone();
        let (suggested_main, suggested_sb) = suggest_main_deck(&user_picks, 23);
        self.main = suggested_main;
        self.sideboard = suggested_sb;
        self.player_colors = top_two_colors(&self.picks[self.user_seat]);
        self.basics = suggest_basic_split(&self.main, self.player_colors, 17);
        self.phase = DraftPhase::Deckbuilding;
    }

    /// Move card at `index` from main deck to sideboard.
    pub fn move_to_sideboard(&mut self, index: usize) {
        if index < self.main.len() {
            let card = self.main.remove(index);
            self.sideboard.push(card);
            self.refresh_basics_suggestion();
        }
    }

    /// Move card at `index` from sideboard to main deck.
    pub fn move_to_main(&mut self, index: usize) {
        if index < self.sideboard.len() {
            let card = self.sideboard.remove(index);
            self.main.push(card);
            self.refresh_basics_suggestion();
        }
    }

    /// Adjust basic-land count for one color. `delta` may be negative
    /// (saturating clamp at zero, no upper bound).
    pub fn adjust_basics(&mut self, color: ManaColor, delta: i32) {
        let current = self.basics.get(&color).copied().unwrap_or(0) as i32;
        let next = (current + delta).max(0) as u32;
        self.basics.insert(color, next);
    }

    /// Re-suggest basic split after a deckbuild edit. Called whenever
    /// the user moves a card in/out of main — keeps the basic count
    /// in sync with the new main deck's color requirements. The user
    /// can override afterwards via the +/- buttons; this just re-bases
    /// the auto-suggestion.
    fn refresh_basics_suggestion(&mut self) {
        // Preserve the currently-chosen basic colors but re-weight
        // them. The user's chosen color split (player_colors) is
        // sticky — we don't re-pick top_two_colors mid-build.
        let total_basics: u32 = self.basics.values().sum();
        let target = if total_basics == 0 { 17 } else { total_basics };
        self.basics = suggest_basic_split(&self.main, self.player_colors, target);
    }

    /// Total cards in main deck once basics are added.
    pub fn main_total(&self) -> u32 {
        self.main.len() as u32 + self.basics.values().copied().sum::<u32>()
    }

    /// Build the final shuffled deck list (main spells + basics) for
    /// the player. Applied at "Play Match" time.
    pub fn build_player_deck(&self) -> Vec<CardFactory> {
        let mut deck: Vec<CardFactory> = self.main.clone();
        for (&color, &count) in &self.basics {
            let basic = basic_land_factory(color);
            for _ in 0..count {
                deck.push(basic);
            }
        }
        enforce_copy_cap(deck)
    }

    /// Build a deck for an opponent bot seat. Same auto-suggestion
    /// pipeline used for the player's initial main split, but applied
    /// to the bot's own picks.
    pub fn build_opponent_deck(&self, seat: usize) -> (Vec<CardFactory>, [ManaColor; 2]) {
        let picks = &self.picks[seat];
        let (main, _sb) = suggest_main_deck(picks, 23);
        let colors = top_two_colors(picks);
        let basics = suggest_basic_split(&main, colors, 17);
        let mut deck = main;
        for (color, count) in basics {
            let basic = basic_land_factory(color);
            for _ in 0..count {
                deck.push(basic);
            }
        }
        (enforce_copy_cap(deck), colors)
    }
}

// ── Marker components ────────────────────────────────────────────────────────

#[derive(Component)]
struct DraftRoot;

#[derive(Component, Clone, Copy)]
struct PackCardButton {
    pack_index: usize,
}

#[derive(Component, Clone, Copy)]
struct DraftTabButton(DraftTab);

#[derive(Component, Clone, Copy)]
struct DeckbuildCardButton {
    in_main: bool,
    index: usize,
}

#[derive(Component, Clone, Copy)]
struct BasicAdjustButton {
    color: ManaColor,
    delta: i32,
}

#[derive(Component, Clone, Copy)]
struct OpponentChoiceButton {
    seat: usize,
}

#[derive(Component)]
struct ConfirmDeckButton;

#[derive(Component)]
struct PlayMatchButton;

#[derive(Component)]
struct DraftBackToMenuButton;

/// Marker for nodes that respond to mouse-wheel scrolling. Carries no
/// data — the wheel handler queries `(&DraftScrollable, &mut
/// ScrollPosition, &ComputedNode, &GlobalTransform)` and routes the
/// scroll delta to whichever scrollable's bounds contain the cursor.
#[derive(Component)]
struct DraftScrollable;

// ── Plugin ───────────────────────────────────────────────────────────────────

pub struct DraftPlugin;

impl Plugin for DraftPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Drafting), enter_drafting)
            .add_systems(OnExit(AppState::Drafting), exit_drafting)
            .add_systems(
                Update,
                (
                    refresh_phase_ui,
                    handle_pack_clicks,
                    handle_tab_clicks,
                    handle_deckbuild_clicks,
                    handle_basic_buttons,
                    handle_confirm_deck,
                    handle_opponent_clicks,
                    handle_play_match,
                    handle_back_to_menu,
                    handle_draft_scroll,
                )
                    .run_if(in_state(AppState::Drafting)),
            );
    }
}

// ── Lifecycle ────────────────────────────────────────────────────────────────

fn enter_drafting(mut commands: Commands) {
    commands.insert_resource(DraftSession::new());
}

fn exit_drafting(mut commands: Commands, root: Query<Entity, With<DraftRoot>>) {
    commands.remove_resource::<DraftSession>();
    for e in &root {
        commands.entity(e).despawn();
    }
}

// ── Render dispatch ──────────────────────────────────────────────────────────

/// Re-spawn the draft UI when the phase changes (or on first frame).
/// Also re-spawns the drafting / deckbuilding screens on every pick
/// so the live state (current pack, main deck, basics) stays in sync.
/// Cheap because each phase's tree is at most ~70 entities.
fn refresh_phase_ui(
    mut commands: Commands,
    session: Option<Res<DraftSession>>,
    asset_server: Res<AssetServer>,
    existing: Query<Entity, With<DraftRoot>>,
) {
    let Some(session) = session else { return };
    if !session.is_changed() && !existing.is_empty() {
        return;
    }
    for e in &existing {
        commands.entity(e).despawn();
    }
    match session.phase {
        DraftPhase::Drafting => spawn_drafting_screen(&mut commands, &asset_server, &session),
        DraftPhase::Deckbuilding => {
            spawn_deckbuilding_screen(&mut commands, &asset_server, &session)
        }
        DraftPhase::OpponentSelect => {
            spawn_opponent_select_screen(&mut commands, &asset_server, &session)
        }
    }
}

// ── Drafting screen ─────────────────────────────────────────────────────────

fn spawn_drafting_screen(
    commands: &mut Commands,
    asset_server: &AssetServer,
    session: &DraftSession,
) {
    let root = spawn_root(commands);
    let user_picks = &session.picks[session.user_seat];
    commands
        .entity(root)
        .with_children(|root| {
            // ── Header: counters on the left, tab toggle on the right ──
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(20.0),
                    ..default()
                },
                BackgroundColor(HEADER_BG),
                Pickable::IGNORE,
            ))
            .with_children(|h| {
                h.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(2.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|col| {
                    col.spawn((
                        Text::new(format!(
                            "Draft — Pack {}/{}, Pick {}/{}",
                            session.pack_round,
                            PACKS_PER_SEAT,
                            session.pick_in_round,
                            PACK_SIZE,
                        )),
                        TextFont { font_size: 22.0, ..default() },
                        TextColor(ACCENT_GOLD),
                        Pickable::IGNORE,
                    ));
                    col.spawn((
                        Text::new(format!("Picked {} cards", user_picks.len())),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                        Pickable::IGNORE,
                    ));
                });

                // Tab toggle: Pack | Picks
                h.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(6.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|tabs| {
                    spawn_tab_button(tabs, "Pack", DraftTab::Pack, session.current_tab);
                    spawn_tab_button(
                        tabs,
                        &format!("Picks ({})", user_picks.len()),
                        DraftTab::Picks,
                        session.current_tab,
                    );
                });
            });

            // ── Body: scrollable, fills remaining vertical space ──
            // No `Pickable::IGNORE` here — the scroll-wheel handler
            // needs to find this node by cursor position. Children
            // (cards, tab buttons) still receive clicks via Bevy's
            // standard topmost-pick UI bubbling.
            root.spawn((
                Node {
                    flex_grow: 1.0,
                    width: Val::Percent(100.0),
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                DraftScrollable,
            ))
            .with_children(|body| {
                match session.current_tab {
                    DraftTab::Pack => spawn_pack_grid(body, asset_server, session),
                    DraftTab::Picks => spawn_picks_grid(body, asset_server, user_picks),
                }
            });

            // ── Stats footer: always visible, summarises user's picks ──
            spawn_stats_footer(root, user_picks);
        });
}

fn spawn_pack_grid(
    body: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    session: &DraftSession,
) {
    body.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(12.0),
            row_gap: Val::Px(12.0),
            padding: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|grid| {
        for (i, factory) in session.user_pack().iter().enumerate() {
            let name = factory().name.to_string();
            spawn_pack_card_tile(grid, asset_server, &name, i);
        }
    });
}

fn spawn_picks_grid(
    body: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    picks: &[CardFactory],
) {
    if picks.is_empty() {
        body.spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(40.0)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("No picks yet — pick a card from the Pack tab to start."),
                TextFont { font_size: 16.0, ..default() },
                TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                Pickable::IGNORE,
            ));
        });
        return;
    }
    body.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            column_gap: Val::Px(8.0),
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(20.0)),
            justify_content: JustifyContent::Center,
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
    ))
    .with_children(|grid| {
        // Latest pick first — most useful order for tracking what
        // just landed in your pile.
        for factory in picks.iter().rev() {
            let path = scryfall::card_asset_path(factory().name);
            let texture: Handle<Image> = asset_server.load(&path);
            grid.spawn((
                Node {
                    width: Val::Px(PICKS_TAB_CARD_W),
                    height: Val::Px(PICKS_TAB_CARD_H),
                    ..default()
                },
                Pickable::IGNORE,
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
            });
        }
    });
}

fn spawn_tab_button(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    tab: DraftTab,
    current: DraftTab,
) {
    let on = tab == current;
    parent
        .spawn((
            Button,
            Node {
                padding: UiRect::axes(Val::Px(18.0), Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(if on { TAB_BG_ON } else { TAB_BG_OFF }),
            DraftTabButton(tab),
        ))
        .with_children(|b| {
            b.spawn((
                Text::new(label),
                TextFont { font_size: 14.0, ..default() },
                TextColor(if on {
                    Color::WHITE
                } else {
                    Color::srgba(0.85, 0.85, 0.85, 1.0)
                }),
                Pickable::IGNORE,
            ));
        });
}

fn spawn_pack_card_tile(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    card_name: &str,
    pack_index: usize,
) {
    let path = scryfall::card_asset_path(card_name);
    let texture: Handle<Image> = asset_server.load(&path);
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(PICK_CARD_W),
                height: Val::Px(PICK_CARD_H),
                ..default()
            },
            BackgroundColor(TILE_BG),
            PackCardButton { pack_index },
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
        });
}

// ── Stats footer ────────────────────────────────────────────────────────────

/// Compact stats summary of the user's picks: color distribution
/// (W U B R G), curve histogram (0 / 1 / 2 / 3 / 4 / 5 / 6+ CMC),
/// and card-type breakdown (Creature / Spell / Land / Other).
/// Always visible across both tabs so the player can monitor their
/// shape while picking.
fn spawn_stats_footer(root: &mut ChildSpawnerCommands, picks: &[CardFactory]) {
    let stats = compute_pick_stats(picks);
    root.spawn((
        Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            column_gap: Val::Px(24.0),
            ..default()
        },
        BackgroundColor(STATS_BG),
        Pickable::IGNORE,
    ))
    .with_children(|footer| {
        // Colors block.
        footer
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|col| {
                col.spawn((
                    Text::new("Colors"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    Pickable::IGNORE,
                ));
                col.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    for (label, count, color) in [
                        ("W", stats.colors[0], Color::srgb(0.95, 0.92, 0.78)),
                        ("U", stats.colors[1], Color::srgb(0.55, 0.75, 0.95)),
                        ("B", stats.colors[2], Color::srgb(0.65, 0.55, 0.75)),
                        ("R", stats.colors[3], Color::srgb(0.95, 0.55, 0.45)),
                        ("G", stats.colors[4], Color::srgb(0.55, 0.85, 0.55)),
                    ] {
                        row.spawn((
                            Text::new(format!("{label}:{count}")),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(color),
                            Pickable::IGNORE,
                        ));
                    }
                });
            });

        // Curve block.
        footer
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|col| {
                col.spawn((
                    Text::new("Curve"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    Pickable::IGNORE,
                ));
                col.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    for (label, count) in [
                        ("0", stats.curve[0]),
                        ("1", stats.curve[1]),
                        ("2", stats.curve[2]),
                        ("3", stats.curve[3]),
                        ("4", stats.curve[4]),
                        ("5", stats.curve[5]),
                        ("6+", stats.curve[6]),
                    ] {
                        row.spawn((
                            Text::new(format!("{label}:{count}")),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgba(0.95, 0.95, 0.95, 1.0)),
                            Pickable::IGNORE,
                        ));
                    }
                });
            });

        // Card-type block.
        footer
            .spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|col| {
                col.spawn((
                    Text::new("Types"),
                    TextFont { font_size: 12.0, ..default() },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 1.0)),
                    Pickable::IGNORE,
                ));
                col.spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    for (label, count) in [
                        ("Creatures", stats.creatures),
                        ("Spells", stats.spells),
                        ("Lands", stats.lands),
                        ("Other", stats.other),
                    ] {
                        row.spawn((
                            Text::new(format!("{label}:{count}")),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(Color::srgba(0.95, 0.95, 0.95, 1.0)),
                            Pickable::IGNORE,
                        ));
                    }
                });
            });
    });
}

/// Aggregated stats for the picks footer. `colors` is a [W,U,B,R,G]
/// histogram of colored pip counts (so a `{W}{U}` card contributes 1
/// to W and 1 to U); `curve[i]` is the count of cards with CMC `i`,
/// or `≥6` in slot 6.
struct PickStats {
    colors: [u32; 5],
    curve: [u32; 7],
    creatures: u32,
    spells: u32,
    lands: u32,
    other: u32,
}

fn compute_pick_stats(picks: &[CardFactory]) -> PickStats {
    use crabomination::card::CardType;
    use crabomination::mana::ManaSymbol;
    let mut s = PickStats {
        colors: [0; 5],
        curve: [0; 7],
        creatures: 0,
        spells: 0,
        lands: 0,
        other: 0,
    };
    let color_idx = |c: ManaColor| match c {
        ManaColor::White => 0,
        ManaColor::Blue => 1,
        ManaColor::Black => 2,
        ManaColor::Red => 3,
        ManaColor::Green => 4,
    };
    for f in picks {
        let def = f();
        // Colored pips (hybrids count for both halves; Phyrexian
        // counts its colored side; generic / X / colorless skipped).
        for sym in &def.cost.symbols {
            match sym {
                ManaSymbol::Colored(c) | ManaSymbol::Phyrexian(c) => {
                    s.colors[color_idx(*c)] += 1;
                }
                ManaSymbol::Hybrid(a, b) => {
                    s.colors[color_idx(*a)] += 1;
                    s.colors[color_idx(*b)] += 1;
                }
                _ => {}
            }
        }
        // Curve — lands skipped; otherwise CMC bucketed 0..=5 then 6+.
        if !def.card_types.contains(&CardType::Land) {
            let cmc = def.cost.cmc().min(6) as usize;
            s.curve[cmc] += 1;
        }
        // Card type — multi-typed cards count as creature first
        // (creatures dominate gameplay decisions), then spell, land,
        // other.
        if def.card_types.contains(&CardType::Creature) {
            s.creatures += 1;
        } else if def.card_types.contains(&CardType::Instant)
            || def.card_types.contains(&CardType::Sorcery)
        {
            s.spells += 1;
        } else if def.card_types.contains(&CardType::Land) {
            s.lands += 1;
        } else {
            s.other += 1;
        }
    }
    s
}

// ── Deckbuilding screen ─────────────────────────────────────────────────────

fn spawn_deckbuilding_screen(
    commands: &mut Commands,
    asset_server: &AssetServer,
    session: &DraftSession,
) {
    let root = spawn_root(commands);
    commands
        .entity(root)
        .with_children(|root| {
            // Header.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(12.0)),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(HEADER_BG),
                Pickable::IGNORE,
            ))
            .with_children(|h| {
                h.spawn((
                    Text::new("Build Your Deck"),
                    TextFont { font_size: 22.0, ..default() },
                    TextColor(ACCENT_GOLD),
                    Pickable::IGNORE,
                ));
                let total = session.main_total();
                let label = format!(
                    "Main: {} ({} spells + {} basics)  ·  Sideboard: {}",
                    total,
                    session.main.len(),
                    total as usize - session.main.len(),
                    session.sideboard.len(),
                );
                h.spawn((
                    Text::new(label),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                    Pickable::IGNORE,
                ));
            });

            // Body: main + sideboard side by side.
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    column_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    width: Val::Percent(100.0),
                    flex_grow: 1.0,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|body| {
                // Main panel.
                body.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(8.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        flex_basis: Val::Percent(60.0),
                        max_height: Val::Px(600.0),
                        overflow: Overflow::scroll_y(),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.10, 0.14, 0.18, 1.0)),
                    ScrollPosition::default(),
                    DraftScrollable,
                ))
                .with_children(|main| {
                    main.spawn((
                        Text::new("Main deck — click a card to move it to the sideboard"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                        Pickable::IGNORE,
                    ));
                    main.spawn((
                        Node {
                            flex_direction: FlexDirection::Row,
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: Val::Px(6.0),
                            row_gap: Val::Px(6.0),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ))
                    .with_children(|grid| {
                        for (i, factory) in session.main.iter().enumerate() {
                            spawn_deckbuild_tile(grid, asset_server, factory().name, true, i);
                        }
                    });
                });

                // Sideboard + basic-land controls.
                body.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(12.0),
                        flex_basis: Val::Percent(40.0),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|right| {
                    // Basic land controls.
                    right.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(6.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.14, 0.18, 1.0)),
                        Pickable::IGNORE,
                    ))
                    .with_children(|panel| {
                        panel.spawn((
                            Text::new("Basic lands"),
                            TextFont { font_size: 14.0, ..default() },
                            TextColor(ACCENT_GOLD),
                            Pickable::IGNORE,
                        ));
                        for color in
                            [ManaColor::White, ManaColor::Blue, ManaColor::Black, ManaColor::Red, ManaColor::Green]
                        {
                            let count = session.basics.get(&color).copied().unwrap_or(0);
                            spawn_basic_row(panel, color, count);
                        }
                    });

                    // Sideboard list.
                    right.spawn((
                        Node {
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(8.0),
                            padding: UiRect::all(Val::Px(10.0)),
                            max_height: Val::Px(400.0),
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.10, 0.14, 0.18, 1.0)),
                        ScrollPosition::default(),
                        DraftScrollable,
                    ))
                    .with_children(|sb| {
                        sb.spawn((
                            Text::new("Sideboard — click a card to move it to the main deck"),
                            TextFont { font_size: 13.0, ..default() },
                            TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                            Pickable::IGNORE,
                        ));
                        sb.spawn((
                            Node {
                                flex_direction: FlexDirection::Row,
                                flex_wrap: FlexWrap::Wrap,
                                column_gap: Val::Px(6.0),
                                row_gap: Val::Px(6.0),
                                ..default()
                            },
                            Pickable::IGNORE,
                        ))
                        .with_children(|grid| {
                            for (i, factory) in session.sideboard.iter().enumerate() {
                                spawn_deckbuild_tile(grid, asset_server, factory().name, false, i);
                            }
                        });
                    });

                    // Confirm button.
                    let confirm_enabled = session.main_total() >= 40;
                    let confirm_label = if confirm_enabled {
                        format!("Confirm Deck ({} cards) →", session.main_total())
                    } else {
                        format!(
                            "Need at least 40 cards (currently {})",
                            session.main_total()
                        )
                    };
                    right.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(10.0)),
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        BackgroundColor(if confirm_enabled {
                            ACCENT_GREEN
                        } else {
                            Color::srgba(0.30, 0.30, 0.35, 0.8)
                        }),
                        ConfirmDeckButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new(confirm_label),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                });
            });
        });
}

fn spawn_deckbuild_tile(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    card_name: &str,
    in_main: bool,
    index: usize,
) {
    let path = scryfall::card_asset_path(card_name);
    let texture: Handle<Image> = asset_server.load(&path);
    parent
        .spawn((
            Button,
            Node {
                width: Val::Px(DECKBUILD_CARD_W),
                height: Val::Px(DECKBUILD_CARD_H),
                ..default()
            },
            BackgroundColor(TILE_BG),
            DeckbuildCardButton { in_main, index },
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
        });
}

fn spawn_basic_row(parent: &mut ChildSpawnerCommands, color: ManaColor, count: u32) {
    let label = match color {
        ManaColor::White => "Plains",
        ManaColor::Blue => "Island",
        ManaColor::Black => "Swamp",
        ManaColor::Red => "Mountain",
        ManaColor::Green => "Forest",
    };
    parent
        .spawn((
            Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(8.0),
                align_items: AlignItems::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|row| {
            row.spawn((
                Text::new(format!("{label}:")),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                Node { min_width: Val::Px(70.0), ..default() },
                Pickable::IGNORE,
            ));
            // Minus button.
            row.spawn((
                Button,
                Node {
                    width: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.40, 0.20, 0.20, 1.0)),
                BasicAdjustButton { color, delta: -1 },
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("−"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
            // Count.
            row.spawn((
                Text::new(format!("{count}")),
                TextFont { font_size: 14.0, ..default() },
                TextColor(Color::WHITE),
                Node { min_width: Val::Px(28.0), ..default() },
                Pickable::IGNORE,
            ));
            // Plus button.
            row.spawn((
                Button,
                Node {
                    width: Val::Px(28.0),
                    padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.20, 0.40, 0.20, 1.0)),
                BasicAdjustButton { color, delta: 1 },
            ))
            .with_children(|b| {
                b.spawn((
                    Text::new("+"),
                    TextFont { font_size: 16.0, ..default() },
                    TextColor(Color::WHITE),
                    Pickable::IGNORE,
                ));
            });
        });
}

// ── Opponent select screen ──────────────────────────────────────────────────

fn spawn_opponent_select_screen(
    commands: &mut Commands,
    _asset_server: &AssetServer,
    session: &DraftSession,
) {
    let root = spawn_root(commands);
    commands
        .entity(root)
        .with_children(|root| {
            // Header.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(6.0),
                    ..default()
                },
                BackgroundColor(HEADER_BG),
                Pickable::IGNORE,
            ))
            .with_children(|h| {
                h.spawn((
                    Text::new("Pick Your Opponent"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(ACCENT_GOLD),
                    Pickable::IGNORE,
                ));
                h.spawn((
                    Text::new("Each bot drafted its own deck. Click one to play against them."),
                    TextFont { font_size: 14.0, ..default() },
                    TextColor(Color::srgba(0.85, 0.85, 0.85, 1.0)),
                    Pickable::IGNORE,
                ));
            });

            // Grid of 7 opponent buttons (seats 1..8, skipping the user).
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    column_gap: Val::Px(16.0),
                    row_gap: Val::Px(16.0),
                    padding: UiRect::all(Val::Px(20.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|grid| {
                for seat in 0..POD_SIZE {
                    if seat == session.user_seat {
                        continue;
                    }
                    let colors = top_two_colors(&session.picks[seat]);
                    let label = format!(
                        "Bot {seat}\n{}{}\n{} picks",
                        color_short(colors[0]),
                        color_short(colors[1]),
                        session.picks[seat].len()
                    );
                    grid.spawn((
                        Button,
                        Node {
                            width: Val::Px(180.0),
                            height: Val::Px(120.0),
                            padding: UiRect::all(Val::Px(12.0)),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(TILE_BG),
                        OpponentChoiceButton { seat },
                    ))
                    .with_children(|tile| {
                        tile.spawn((
                            Text::new(label),
                            TextFont { font_size: 16.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                }
            });

            // Selected-opponent confirmation row + Play button.
            if let Some(seat) = session.chosen_opponent {
                let colors = top_two_colors(&session.picks[seat]);
                root.spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: Val::Px(10.0),
                        padding: UiRect::all(Val::Px(20.0)),
                        ..default()
                    },
                    Pickable::IGNORE,
                ))
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!(
                            "Selected: Bot {seat} ({}{})",
                            color_short(colors[0]),
                            color_short(colors[1]),
                        )),
                        TextFont { font_size: 18.0, ..default() },
                        TextColor(ACCENT_GOLD),
                        Pickable::IGNORE,
                    ));
                    row.spawn((
                        Button,
                        Node {
                            padding: UiRect::axes(Val::Px(28.0), Val::Px(12.0)),
                            ..default()
                        },
                        BackgroundColor(ACCENT_GREEN),
                        PlayMatchButton,
                    ))
                    .with_children(|b| {
                        b.spawn((
                            Text::new("Play Match →"),
                            TextFont { font_size: 18.0, ..default() },
                            TextColor(Color::WHITE),
                            Pickable::IGNORE,
                        ));
                    });
                });
            }

            // Back-to-menu escape hatch.
            root.spawn((
                Node {
                    width: Val::Percent(100.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                Pickable::IGNORE,
            ))
            .with_children(|row| {
                row.spawn((
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.30, 0.30, 0.35, 0.95)),
                    DraftBackToMenuButton,
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new("Back to Menu"),
                        TextFont { font_size: 14.0, ..default() },
                        TextColor(Color::WHITE),
                        Pickable::IGNORE,
                    ));
                });
            });
        });
}

// ── Click handlers ──────────────────────────────────────────────────────────

fn handle_pack_clicks(
    mut session: Option<ResMut<DraftSession>>,
    buttons: Query<(&Interaction, &PackCardButton), Changed<Interaction>>,
) {
    let Some(session) = session.as_mut() else { return };
    if !matches!(session.phase, DraftPhase::Drafting) {
        return;
    }
    // Picks tab is view-only; ignore pack clicks while it's active.
    if !matches!(session.current_tab, DraftTab::Pack) {
        return;
    }
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            session.process_user_pick(btn.pack_index);
            // One pick per click — bail to avoid double-consuming.
            return;
        }
    }
}

fn handle_tab_clicks(
    mut session: Option<ResMut<DraftSession>>,
    buttons: Query<(&Interaction, &DraftTabButton), Changed<Interaction>>,
) {
    let Some(session) = session.as_mut() else { return };
    if !matches!(session.phase, DraftPhase::Drafting) {
        return;
    }
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed && session.current_tab != btn.0 {
            session.current_tab = btn.0;
            return;
        }
    }
}

fn handle_deckbuild_clicks(
    mut session: Option<ResMut<DraftSession>>,
    buttons: Query<(&Interaction, &DeckbuildCardButton), Changed<Interaction>>,
) {
    let Some(session) = session.as_mut() else { return };
    if !matches!(session.phase, DraftPhase::Deckbuilding) {
        return;
    }
    for (interaction, btn) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if btn.in_main {
            session.move_to_sideboard(btn.index);
        } else {
            session.move_to_main(btn.index);
        }
        return;
    }
}

fn handle_basic_buttons(
    mut session: Option<ResMut<DraftSession>>,
    buttons: Query<(&Interaction, &BasicAdjustButton), Changed<Interaction>>,
) {
    let Some(session) = session.as_mut() else { return };
    if !matches!(session.phase, DraftPhase::Deckbuilding) {
        return;
    }
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            session.adjust_basics(btn.color, btn.delta);
            return;
        }
    }
}

fn handle_confirm_deck(
    mut session: Option<ResMut<DraftSession>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<ConfirmDeckButton>)>,
) {
    let Some(session) = session.as_mut() else { return };
    if !matches!(session.phase, DraftPhase::Deckbuilding) {
        return;
    }
    for interaction in &buttons {
        if *interaction == Interaction::Pressed && session.main_total() >= 40 {
            session.phase = DraftPhase::OpponentSelect;
            return;
        }
    }
}

fn handle_opponent_clicks(
    mut session: Option<ResMut<DraftSession>>,
    buttons: Query<(&Interaction, &OpponentChoiceButton), Changed<Interaction>>,
) {
    let Some(session) = session.as_mut() else { return };
    if !matches!(session.phase, DraftPhase::OpponentSelect) {
        return;
    }
    for (interaction, btn) in &buttons {
        if *interaction == Interaction::Pressed {
            session.chosen_opponent = Some(btn.seat);
            return;
        }
    }
}

fn handle_play_match(
    mut commands: Commands,
    session: Option<Res<DraftSession>>,
    mut next_state: ResMut<NextState<AppState>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<PlayMatchButton>)>,
) {
    let Some(session) = session else { return };
    if !matches!(session.phase, DraftPhase::OpponentSelect) {
        return;
    }
    let Some(opp_seat) = session.chosen_opponent else {
        return;
    };
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            let player_deck = session.build_player_deck();
            let (opp_deck, opp_colors) = session.build_opponent_deck(opp_seat);
            let opponent_label = format!(
                "Bot {opp_seat} ({}{})",
                color_short(opp_colors[0]),
                color_short(opp_colors[1]),
            );
            commands.insert_resource(DraftedDecks {
                player_deck,
                opponent_deck: opp_deck,
                opponent_label,
            });
            next_state.set(AppState::InGame);
            return;
        }
    }
}

fn handle_back_to_menu(
    mut next_state: ResMut<NextState<AppState>>,
    buttons: Query<&Interaction, (Changed<Interaction>, With<DraftBackToMenuButton>)>,
) {
    for interaction in &buttons {
        if *interaction == Interaction::Pressed {
            next_state.set(AppState::Menu);
            return;
        }
    }
}

/// Mouse-wheel scrolling for `DraftScrollable` panels. Looks up the
/// cursor's window position, walks every scrollable node, and routes
/// the wheel delta to whichever node's screen-space bounds contain
/// the cursor. `ScrollPosition.y` is clamped at the lower bound; the
/// upper bound is left to Bevy's layout system, which trims
/// out-of-range scroll positions on the next frame.
///
/// Both `Line` and `Pixel` scroll units are supported — most desktop
/// mice emit `Line(±1)` per detent; trackpads / Wayland surfaces
/// emit `Pixel` deltas directly. The line variant is multiplied by
/// `SCROLL_LINE_PX` so each detent advances roughly one card row.
fn handle_draft_scroll(
    mut wheel: MessageReader<bevy::input::mouse::MouseWheel>,
    windows: Query<&Window>,
    mut scrollables: Query<
        (&ComputedNode, &GlobalTransform, &mut ScrollPosition),
        With<DraftScrollable>,
    >,
) {
    use bevy::input::mouse::MouseScrollUnit;
    // Aggregate this frame's deltas into a single scalar before the
    // bounds check, so a flurry of wheel events on the same frame
    // doesn't trip the per-event clamp pass repeatedly.
    let mut delta_px = 0.0f32;
    for ev in wheel.read() {
        delta_px += match ev.unit {
            MouseScrollUnit::Line => -ev.y * SCROLL_LINE_PX,
            MouseScrollUnit::Pixel => -ev.y,
        };
    }
    if delta_px == 0.0 {
        return;
    }
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };
    for (computed, gtf, mut scroll) in &mut scrollables {
        // `ComputedNode::size()` is in logical pixels; the global
        // translation is the node's center in screen space.
        let size = computed.size();
        let center = gtf.translation().truncate();
        let half = size * 0.5;
        let inside = cursor.x >= center.x - half.x
            && cursor.x <= center.x + half.x
            && cursor.y >= center.y - half.y
            && cursor.y <= center.y + half.y;
        if !inside {
            continue;
        }
        // Clamp against content height. `ComputedNode` exposes the
        // content size only indirectly via `content_size()`; for
        // safety we just clamp the lower bound (no upper bound) and
        // let Bevy's layout system trim invalid values on the next
        // frame (see `ScrollPosition`'s docstring — the layout
        // system normalises out-of-range positions).
        let new_y = (scroll.0.y + delta_px).max(0.0);
        if (new_y - scroll.0.y).abs() > f32::EPSILON {
            scroll.0.y = new_y;
        }
        // Only the topmost hovered scrollable consumes the scroll —
        // bail to keep nested scrollables from double-scrolling on
        // the same wheel tick.
        return;
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

fn spawn_root(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(PANEL_BG),
            DraftRoot,
        ))
        .id()
}

fn color_short(c: ManaColor) -> &'static str {
    match c {
        ManaColor::White => "W",
        ManaColor::Blue => "U",
        ManaColor::Black => "B",
        ManaColor::Red => "R",
        ManaColor::Green => "G",
    }
}
