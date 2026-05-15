//! Shared visual language for the 2-D HUD/menu/modal overlay.
//!
//! Every 2-D UI surface (menus, modals, HUD panels, tooltips, draft picker,
//! decision prompts) should source colors and fonts from here. 3-D scene
//! materials (gizmos, card materials, counter coins) live in their own
//! modules — those colors have domain meaning (MTG colors, counter kinds)
//! and aren't part of the chrome.

use bevy::prelude::*;

// ── Fonts ────────────────────────────────────────────────────────────────────

/// Path to the single UI font. All HUD/menu/modal text uses this.
pub const FONT_PATH: &str = "fonts/MiranoExtendedFreebie-Light.ttf";

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
            font: self.sans.clone(),
            font_size: size,
            ..default()
        }
    }
}

/// Startup system: loads the UI font and inserts `UiFonts`.
pub fn init_ui_fonts(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(UiFonts {
        sans: asset_server.load(FONT_PATH),
    });
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

/// Used inside modal choice lists to mark a toggle as "selected".
pub const BUTTON_SELECTED_BG: Color = Color::srgba(0.60, 0.25, 0.25, 0.95);

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
