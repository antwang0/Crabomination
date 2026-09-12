//! Shared visual language for the 2-D HUD/menu/modal overlay.
//!
//! Every 2-D UI surface (menus, modals, HUD panels, tooltips, draft picker,
//! decision prompts) should source colors and fonts from here. 3-D scene
//! materials (gizmos, card materials, counter coins) live in their own
//! modules — those colors have domain meaning (MTG colors, counter kinds)
//! and aren't part of the chrome.

use bevy::prelude::*;
use bevy::text::{FontFeatureTag, FontFeatures};

// ── Fonts ────────────────────────────────────────────────────────────────────

/// Path to the single UI font. All HUD/menu/modal text uses this.
pub const FONT_PATH: &str = "fonts/MiranoExtendedFreebie-Light.ttf";

/// Tabular (uniform-width) figures for HUD numbers. This game changes numbers
/// in place constantly — life totals, P/T, counters, mana, timers — and with
/// proportional digits each change reflows the text node's width, jittering
/// neighbours. `tnum` keeps every digit the same advance width so a `20 → 9 →
/// 19` swing doesn't shift layout.
///
/// NOTE: this is an OpenType *font* feature — it only takes effect if the
/// active font ships a `tnum` table. The bundled `MiranoExtendedFreebie-Light`
/// currently exposes only `kern`, so this is **inert until a tnum-capable font
/// is used**; the hook lives here so it activates automatically on a font swap.
fn ui_font_features() -> FontFeatures {
    FontFeatures::builder()
        .enable(FontFeatureTag::TABULAR_FIGURES)
        .build()
}

/// Loaded handles for the UI font. Insert at startup, then accept
/// `Res<UiFonts>` in any UI setup system and call `ui_fonts.tf(size)`
/// instead of constructing `TextFont` directly.
#[derive(Resource, Clone)]
pub struct UiFonts {
    pub sans: Handle<Font>,
}

impl UiFonts {
    /// Build a `TextFont` using the project's standard font at `size` px.
    pub fn tf(&self, size: f32) -> TextFont {
        TextFont {
            font: self.sans.clone().into(),
            font_size: FontSize::Px(size),
            font_features: ui_font_features(),
            ..default()
        }
    }
}

/// Plugin that inserts [`UiFonts`] eagerly during `App::build()`.
///
/// Must be added AFTER `DefaultPlugins` (needs `AssetServer`) but before any
/// plugin that registers `OnEnter` systems that access `UiFonts` — because
/// Bevy fires the initial `StateTransition` (and thus `OnEnter`) *before*
/// `PreStartup`, so a `Startup` system would be too late.
pub struct UiFontsPlugin;

impl bevy::app::Plugin for UiFontsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        let font = app.world().resource::<AssetServer>().load(FONT_PATH);
        app.world_mut().insert_resource(UiFonts { sans: font });
        app.add_systems(bevy::app::Update, update_hover_tint);
    }
}

// ── Surfaces ─────────────────────────────────────────────────────────────────

/// Dim full-screen scrim behind modal dialogs.
pub const OVERLAY_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.70);
/// Heavier scrim used by tooltips and popups (more opaque).
pub const OVERLAY_BG_HEAVY: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
/// Light overlay (cards-grid backgrounds, less attention-stealing).
pub const OVERLAY_BG_LIGHT: Color = Color::srgba(0.0, 0.0, 0.0, 0.50);

/// Canonical modal / panel body background.
pub const PANEL_BG: Color = Color::srgba(0.06, 0.06, 0.12, 0.97);
/// Slightly lighter panel section (header strips, list rows).
pub const PANEL_BG_RAISED: Color = Color::srgba(0.10, 0.10, 0.18, 1.0);
/// Slightly darker panel section (stats strips, sub-panels).
pub const PANEL_BG_SUNKEN: Color = Color::srgba(0.08, 0.08, 0.14, 1.0);

/// HUD strip background (semi-transparent, no panel border).
pub const HUD_BG: Color = Color::srgba(0.0, 0.0, 0.0, 0.78);
/// HUD background tinted blue — "active" / "info" feel.
#[allow(dead_code)]
pub const HUD_BG_INFO: Color = Color::srgba(0.0, 0.10, 0.22, 0.82);
/// HUD background tinted red — error or danger banner.
pub const HUD_BG_DANGER: Color = Color::srgba(0.25, 0.0, 0.0, 0.82);

/// Inactive text-input / non-selected toggle.
pub const FIELD_BG: Color = Color::srgba(0.16, 0.16, 0.22, 1.0);
/// Focused text-input / selected toggle.
pub const FIELD_BG_FOCUSED: Color = Color::srgba(0.28, 0.28, 0.50, 1.0);

// ── Buttons ──────────────────────────────────────────────────────────────────

/// Neutral button (no strong intent — e.g. "Export", "Cancel" in non-danger contexts).
pub const BUTTON_NEUTRAL_BG: Color = Color::srgba(0.18, 0.18, 0.28, 1.0);
/// Neutral button — hover/hot state.
pub const BUTTON_NEUTRAL_HOT: Color = Color::srgba(0.30, 0.30, 0.42, 1.0);

/// Primary affirmative action (Play, Keep, Confirm, Continue).
pub const BUTTON_PRIMARY_BG: Color = Color::srgba(0.18, 0.45, 0.20, 1.0);

/// Secondary action — informational / lobby (Host, New Game).
pub const BUTTON_INFO_BG: Color = Color::srgba(0.20, 0.30, 0.55, 1.0);

/// Cautionary / orange action (Join LAN, switch tab on).
pub const BUTTON_WARN_BG: Color = Color::srgba(0.45, 0.30, 0.15, 1.0);

/// Destructive / cancellation (Quit, Mulligan, Load Debug State).
pub const BUTTON_DANGER_BG: Color = Color::srgba(0.45, 0.20, 0.20, 1.0);

/// Purple accent — spectate-bots, set-tab.
pub const BUTTON_ACCENT_BG: Color = Color::srgba(0.32, 0.20, 0.45, 1.0);

/// Amber — opponent has a spell on the stack and you must respond.
pub const BUTTON_URGENT_BG: Color = Color::srgba(0.75, 0.55, 0.05, 1.0);

/// Half-alpha tint for buttons that don't apply right now (priority held
/// by another player).
pub const BUTTON_DISABLED_BG: Color = Color::srgba(0.08, 0.15, 0.25, 0.5);

/// Used inside modal choice lists to mark a toggle as "selected".
pub const BUTTON_SELECTED_BG: Color = Color::srgba(0.60, 0.25, 0.25, 0.95);

/// Muted-green "selected / verified / active filter" tint used inside
/// picker rows (audit ✓-rows, active pool filter, on toggle). Less
/// attention-stealing than `BUTTON_SELECTED_BG` (red) — fits states
/// that mean "this is good / done" rather than "this is chosen".
pub const BUTTON_SELECTED_GOOD_BG: Color = Color::srgba(0.18, 0.30, 0.20, 1.0);

/// Modal "card tile / row" background — un-selected entries in a grid
/// of cards (scry/search/discard/put-on-library pickers).
pub const PANEL_TILE_BG: Color = Color::srgba(0.20, 0.20, 0.24, 0.95);

/// Tertiary action background — small arrow / nudge buttons inside
/// modals (Scry reorder arrows, Serum Powder).
pub const BUTTON_TERTIARY_BG: Color = Color::srgba(0.25, 0.30, 0.40, 0.95);
/// Tertiary action — disabled state.
pub const BUTTON_TERTIARY_BG_DISABLED: Color = Color::srgba(0.15, 0.15, 0.18, 0.6);

// ── Accents ──────────────────────────────────────────────────────────────────

/// Warm cream gold — used for titles and "your turn" hints.
pub const ACCENT_GOLD: Color = Color::srgb(1.0, 0.85, 0.55);
/// Bright yellow — active phase / current step marker.
pub const ACCENT_YELLOW: Color = Color::srgb(1.0, 0.88, 0.0);
/// Bright orange — spell badge on the stack, pass-priority "yours".
pub const ACCENT_ORANGE: Color = Color::srgb(1.0, 0.65, 0.20);
/// Sky blue — trigger badge on the stack, info accents.
pub const ACCENT_BLUE: Color = Color::srgb(0.45, 0.75, 1.0);
/// Bright green — selectable / available indicator.
pub const ACCENT_GREEN: Color = Color::srgb(0.30, 0.70, 0.35);

// ── Text ─────────────────────────────────────────────────────────────────────

pub const TEXT_PRIMARY: Color = Color::WHITE;
/// Standard body text — slightly off-white to reduce eye strain on dark panels.
pub const TEXT_BODY: Color = Color::srgba(0.85, 0.85, 0.85, 1.0);
/// Secondary / label text.
pub const TEXT_SECONDARY: Color = Color::srgba(0.70, 0.70, 0.70, 1.0);
/// Muted / disabled text.
pub const TEXT_MUTED: Color = Color::srgba(0.55, 0.55, 0.55, 0.80);
/// Placeholder / hint text in input fields.
pub const TEXT_PLACEHOLDER: Color = Color::srgba(0.65, 0.65, 0.65, 1.0);
/// Pale blue — used for the viewer's own status line.
pub const TEXT_INFO: Color = Color::srgb(0.65, 0.88, 1.0);
/// Pale red — used for opponent status, damage events in the log.
pub const TEXT_DANGER: Color = Color::srgb(1.0, 0.55, 0.55);
/// Mid-green — used for life-gain events in the log.
pub const TEXT_GOOD: Color = Color::srgb(0.55, 0.95, 0.55);

// ── Geometry ─────────────────────────────────────────────────────────────────

/// Standard pill-corner radius for buttons and small chips.
pub const RADIUS_BUTTON: Val = Val::Px(4.0);
/// Standard rounded-rect radius for menus and modal panels. (HUD strips
/// stay square-cornered — they shouldn't read as floating cards.)
pub const RADIUS_PANEL: Val = Val::Px(8.0);

// ── Hover tint ───────────────────────────────────────────────────────────────

/// Attach to any `Button` whose background should brighten on hover/press.
/// `update_hover_tint` watches `Changed<Interaction>` and applies the
/// stored idle/hot colors. Do *not* attach to buttons whose background
/// is already state-driven (Pass-priority colour swap, format toggles,
/// quality presets, focused text fields) — the two systems would fight.
#[derive(Component, Clone, Copy)]
pub struct HoverTint {
    pub idle: Color,
    pub hot: Color,
}

impl HoverTint {
    /// Derive a hot colour by lightening `idle` ~12% per channel. Keeps
    /// theme variants visually related without per-call tuning.
    pub fn new(idle: Color) -> Self {
        let s = idle.to_srgba();
        let hot = Color::srgba(
            (s.red + 0.12).min(1.0),
            (s.green + 0.12).min(1.0),
            (s.blue + 0.12).min(1.0),
            s.alpha,
        );
        Self { idle, hot }
    }
}

/// Bevy system: brighten `HoverTint` buttons on Hovered/Pressed,
/// restore idle on None.
pub fn update_hover_tint(
    mut q: Query<(&Interaction, &HoverTint, &mut BackgroundColor), Changed<Interaction>>,
) {
    for (interaction, tint, mut bg) in &mut q {
        *bg = BackgroundColor(match *interaction {
            Interaction::Hovered | Interaction::Pressed => tint.hot,
            Interaction::None => tint.idle,
        });
    }
}

// ── Z layers ─────────────────────────────────────────────────────────────────

/// The one ordering of every 2-D surface in the client.
///
/// Bevy stacks siblings by spawn order unless a node carries a
/// `GlobalZIndex`. Before this module existed the badge band was coherent
/// — eight modules each independently defined `const … Z: i32 = -1` — but
/// the band above the HUD was nine magic numbers across twelve files, and
/// **every modal spawned with no index at all**: `decision_ui`, `popups`,
/// `game_over`, `game_ui`, `lobby_ui`, `menu` and `audit` all sat at an
/// implicit `0` alongside the HUD, ordered by whichever happened to spawn
/// last. It worked, by accident of spawn order. `quality.rs` had already
/// paid for the fragility — its escape menu used `1000` specifically
/// because "the mulligan / decision modals carry no z-index, so they'd
/// otherwise render on top of this since they spawn later".
///
/// Values are spaced by ten so a new surface can slot between two
/// existing ones without renumbering. The absolute numbers carry no
/// meaning — only the order does, so prefer adding a named constant over
/// reusing a neighbour's.
///
/// The previous scheme's order is preserved exactly: this is a monotonic
/// remap of `-1, 0, 30, 35, 40, 45, 46, 60, 100, 1000`, with `MODAL` and
/// `CARD_PEEK` inserted where the implicit-`0` surfaces used to land.
pub mod layer {
    /// In-world overlays reprojected onto cards each frame (P/T, keyword
    /// strips, counter labels, token / regen / lock / agenda / free-cast
    /// badges). Deliberately *beneath* all 2-D chrome so popups,
    /// tooltips and modals draw over them.
    pub const CARD_OVERLAY: i32 = -10;

    /// The persistent HUD — player panels, phase chart, game log, stack
    /// panel, buttons. This is Bevy's default, so HUD nodes need not set
    /// it; the constant exists to give the band a name and to anchor the
    /// ordering assertion below.
    pub const HUD: i32 = 0;

    /// Modal surfaces: decision prompts, the graveyard / exile browsers,
    /// the game-over screen, the ability and alt-cast menus, the lobby,
    /// main menu, draft and audit panels. Everything here is mutually
    /// exclusive in practice; a surface that must sit over another modal
    /// belongs in a band above, not deeper into this one.
    pub const MODAL: i32 = 10;

    /// Card inspection that must stay legible over a modal — the Alt-hold
    /// peek popup and the pile tooltip. Above [`MODAL`] so reading a card
    /// works from inside a decision prompt.
    pub const CARD_PEEK: i32 = 20;

    /// Cursor-following card preview for log lines and stack tiles.
    pub const HOVER_PREVIEW: i32 = 30;

    /// Full-screen tint that must colour everything below it but eat no
    /// clicks (the low-life vignette).
    pub const SCREEN_TINT: i32 = 40;

    /// Status banners: pending cast, spectator notice.
    pub const BANNER: i32 = 50;

    /// Banners the player must see over any surface, because they explain
    /// why the game is not responding — reconnect, rope timer, chess clock.
    pub const BANNER_URGENT: i32 = 60;

    /// The chat input bar: above the banners so the line being typed is
    /// never covered.
    pub const CHAT: i32 = 70;

    /// Prompts that own the keyboard and must clear everything under them
    /// — the export-state prompt and the menu's settings panel.
    pub const TOP_PROMPT: i32 = 80;

    /// Transient full-screen feedback that is allowed to frame even a
    /// top prompt: the damage vignette, the draft alt-hold tooltip.
    /// Click-through by construction (`Pickable::IGNORE`).
    pub const SCREEN_FLASH: i32 = 90;

    /// The in-game escape menu. The pause surface — always reachable and
    /// always on top.
    pub const ESCAPE_MENU: i32 = 100;

    // ── The ordering contract, enforced at compile time ──────────────────
    //
    // The table's whole contract is its *order*; the numbers carry no
    // meaning. Spelling the sequence out here means a reorder has to
    // restate the intended order rather than silently reshuffling which
    // surface covers which — and it fails the *build*, not a test run,
    // which is the same reason `game/layers.rs` asserts its struct sizes
    // this way.
    //
    // `CARD_OVERLAY < HUD` is the one eight other modules depend on:
    // `pt_label`'s header states it as "rendered beneath other UI so peek
    // popups, tooltips, and modals always draw on top of it".
    const _: () = assert!(CARD_OVERLAY < HUD);
    const _: () = assert!(HUD < MODAL);
    const _: () = assert!(MODAL < CARD_PEEK);
    const _: () = assert!(CARD_PEEK < HOVER_PREVIEW);
    const _: () = assert!(HOVER_PREVIEW < SCREEN_TINT);
    const _: () = assert!(SCREEN_TINT < BANNER);
    const _: () = assert!(BANNER < BANNER_URGENT);
    const _: () = assert!(BANNER_URGENT < CHAT);
    const _: () = assert!(CHAT < TOP_PROMPT);
    const _: () = assert!(TOP_PROMPT < SCREEN_FLASH);
    const _: () = assert!(SCREEN_FLASH < ESCAPE_MENU);
}
