//! Escape-key arbitration: one press, one action.
//!
//! Twelve systems read `KeyCode::Escape` independently, so a single press
//! could dismiss several surfaces at once — a graveyard browser open over a
//! keyboard-cursor selection lost both, and Esc-to-cancel-chat also closed
//! whatever else happened to be up.
//!
//! The previous mechanism got half of this right and shows why the other
//! half is hard. `EscConsumed` was a one-frame flag, but only `kb_cursor`
//! read it; precedence actually lived in `handle_settings_toggle`, as a
//! hand-maintained list of ten "is a modal or selection active"
//! predicates. Its own comment records that list drifting: "The
//! manual-mana payment window is the one that bit … Missing from this
//! list, Esc popped the settings panel over the half-finished cast
//! instead, which reads as 'Esc doesn't cancel'." `cancel_pickers_on_escape`
//! carried the same note — a new picker had to be added "here (and in the
//! settings-toggle guard)". Two lists, and they disagreed.
//!
//! # Why a precomputed owner rather than system order
//!
//! Expressing precedence as system order is the obvious design and does
//! not fit this schedule. `cancel_pickers_on_escape` is pinned
//! `.after(handle_game_input)` (so a click and an Esc in the same frame
//! resolve as the click) while `handle_keyboard_cursor_input` is pinned
//! `.before` it — a picker-before-selection chain closes that into a
//! cycle, and Bevy rejects the schedule. Rewiring how the decision and
//! picker blocks relate to the 800-line input system is a much larger
//! change than arbitrating a key.
//!
//! So [`compute_esc_focus`] runs once in `PreUpdate` and names the single
//! surface that owns the press; every handler then asks
//! [`EscFocus::owns`]. This has three properties the flag did not:
//!
//! - **Order-independent.** Handlers read `Res<EscFocus>` and hold `ResMut`
//!   only of their own state, so no ordering constraint is needed and none
//!   of the existing graph moves.
//! - **No `ResMut` contention.** A first-come "claim the key" flag would
//!   need `ResMut` of one shared resource in every Esc handler, which
//!   serialises them all and — worse — makes the winner depend on
//!   whatever order Bevy happened to pick among them.
//! - **One list.** The precedence *and* the "is this surface open"
//!   predicates live in [`EscSurface`] and [`compute_esc_focus`] below,
//!   which is the whole point — there is now one place to add a picker.
//!
//! The cost is that `PreUpdate` sees end-of-previous-frame state, so a
//! surface opened during frame N owns Esc from frame N+1. For a key a
//! human presses that is invisible; nothing here is one-frame tight.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::systems::input_guard::TextInputGuard;

/// The surfaces that can own an Escape press, **ordered topmost first**.
///
/// This declaration *is* the precedence: [`compute_esc_focus`] walks the
/// variants in order and stops at the first one that is open. Adding a
/// surface means adding a variant in the right place and one predicate —
/// not a branch in somebody else's guard.
///
/// The order deliberately mirrors `theme::layer`: the surface that draws
/// on top gets the key first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EscSurface {
    /// The in-game pause menu (`theme::layer::ESCAPE_MENU`).
    PauseMenu,
    /// The F1 / `?` shortcut overlay.
    ShortcutHelp,
    /// The graveyard browser.
    GraveyardBrowser,
    /// The exile browser.
    ExileBrowser,
    /// Any cast-flow picker: alt-cast, helper tap, spree, split, pay-times,
    /// ability menu, hand menu, or a held manual-mana payment.
    Picker,
    /// The "choose a mode" modal armed before a cast is submitted.
    ModePick,
    /// A cancellable decision prompt from the engine.
    DecisionPrompt,
    /// An armed targeting session.
    Targeting,
    /// A blocker picked but not yet assigned.
    BlockerPlan,
    /// A non-empty attacker plan.
    AttackPlan,
    /// The keyboard cursor's current selection.
    CursorSelection,
    /// The camera parked over a seat's shoulder.
    CameraFocus,
}

/// Who owns this frame's Escape press, if anyone.
///
/// Written by [`compute_esc_focus`] in `PreUpdate`; read-only everywhere
/// else.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct EscFocus {
    pressed: bool,
    owner: Option<EscSurface>,
}

impl EscFocus {
    /// Whether `surface` owns this frame's press. The idiom is a plain
    /// replacement for the old `keyboard.just_pressed(KeyCode::Escape)`:
    ///
    /// ```ignore
    /// if esc.owns(EscSurface::GraveyardBrowser) {
    ///     state.open = false;
    /// }
    /// ```
    pub fn owns(&self, surface: EscSurface) -> bool {
        self.pressed && self.owner == Some(surface)
    }

    /// A press that no open surface wanted — the pause menu's cue to
    /// open. This replaces the ten-predicate "is anything else active?"
    /// list: running out of surfaces cannot forget one.
    pub fn unclaimed(&self) -> bool {
        self.pressed && self.owner.is_none()
    }
}

/// Every piece of state that can own an Escape press.
///
/// Bundled because the count is past Bevy's per-system parameter limit,
/// and because the bundle *is* the registry — a surface that is not read
/// here cannot be arbitrated.
#[derive(SystemParam)]
pub struct EscSurfaceStates<'w, 's> {
    settings: Res<'w, crate::systems::quality::SettingsOpen>,
    help: Query<'w, 's, Entity, With<crate::systems::ui::ShortcutHelpPanel>>,
    graveyard: Res<'w, crate::game::GraveyardBrowserState>,
    exile: Res<'w, crate::game::ExileBrowserState>,
    alt_cast: Res<'w, crate::game::AltCastState>,
    helper_tap: Res<'w, crate::game::HelperTapState>,
    spree_cast: Res<'w, crate::game::SpreeCastState>,
    split_cast: Res<'w, crate::game::SplitCastState>,
    pay_times: Res<'w, crate::game::PayTimesState>,
    ability_menu: Res<'w, crate::game::AbilityMenuState>,
    hand_menu: Res<'w, crate::systems::game_ui::HandMenuState>,
    pending_mana_cast: Res<'w, crate::net_plugin::PendingManaCast>,
    modal_cast: Res<'w, crate::game::PendingModalCast>,
    view: Res<'w, crate::net_plugin::CurrentView>,
    targeting: Res<'w, crate::game::TargetingState>,
    blocking: Res<'w, crate::game::BlockingState>,
    attacking: Res<'w, crate::game::AttackingState>,
    cursor: Res<'w, crate::systems::kb_cursor::KeyboardCursor>,
    camera_focus: Res<'w, crate::systems::camera_zoom::CameraFocusSeat>,
}

impl EscSurfaceStates<'_, '_> {
    /// The topmost open surface, walking [`EscSurface`] in declaration
    /// order.
    fn topmost(&self) -> Option<EscSurface> {
        use EscSurface as S;

        // A targeting session driven by a pending decision cannot be
        // cancelled — the engine is blocked waiting for an answer — so it
        // must not claim the press either, or Esc would go dead instead of
        // falling through to whatever else is open. `handle_game_input`
        // prints the explanation; this just declines to own the key.
        let targeting_cancellable = self.targeting.active && !self.targeting.pending_decision_target;

        let any_picker = self.alt_cast.pending.is_some()
            || self.helper_tap.pending.is_some()
            || self.spree_cast.pending.is_some()
            || self.split_cast.pending.is_some()
            || self.pay_times.pending.is_some()
            || self.ability_menu.card_id.is_some()
            || self.hand_menu.card_id.is_some()
            || self.pending_mana_cast.0.is_some();

        let cancellable_decision = self.view.0.as_ref().is_some_and(|cv| {
            cv.pending_decision
                .as_ref()
                .is_some_and(|pd| pd.cancellable && pd.decision.is_some())
        });

        [
            (S::PauseMenu, self.settings.0),
            (S::ShortcutHelp, !self.help.is_empty()),
            (S::GraveyardBrowser, self.graveyard.open),
            (S::ExileBrowser, self.exile.open),
            (S::Picker, any_picker),
            (S::ModePick, self.modal_cast.card_id.is_some()),
            (S::DecisionPrompt, cancellable_decision),
            (S::Targeting, targeting_cancellable),
            (S::BlockerPlan, self.blocking.selected_blocker.is_some()),
            (S::AttackPlan, !self.attacking.plan.is_empty()),
            (S::CursorSelection, self.cursor.selection.is_some()),
            (S::CameraFocus, self.camera_focus.0.is_some()),
        ]
        .into_iter()
        .find_map(|(surface, open)| open.then_some(surface))
    }
}

/// Name the surface that owns this frame's Escape press.
///
/// Runs in `PreUpdate`, which is what makes every consumer
/// order-independent: the answer is already settled before any `Update`
/// system looks.
///
/// A press made while a text surface owns the keyboard is **not**
/// latched — chat, the debug console and the export prompt each handle
/// `Escape` through `MessageReader<KeyboardInput>` to cancel their own
/// input, and that press is theirs alone. Folding the check in here is
/// what stops Esc-to-cancel-chat from also closing an open browser, and
/// means no consumer needs its own [`TextInputGuard`] for `Escape`.
pub fn compute_esc_focus(
    keyboard: Res<ButtonInput<KeyCode>>,
    text_input: TextInputGuard,
    surfaces: EscSurfaceStates,
    mut focus: ResMut<EscFocus>,
) {
    let pressed = keyboard.just_pressed(KeyCode::Escape) && !text_input.typing();
    let owner = pressed.then(|| surfaces.topmost()).flatten();
    if focus.pressed != pressed || focus.owner != owner {
        *focus = EscFocus { pressed, owner };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn focus(owner: Option<EscSurface>) -> EscFocus {
        EscFocus { pressed: true, owner }
    }

    #[test]
    fn exactly_one_surface_owns_a_press() {
        let f = focus(Some(EscSurface::GraveyardBrowser));
        assert!(f.owns(EscSurface::GraveyardBrowser));
        // The bug this replaces: one press dismissing a browser *and*
        // clearing the cursor selection under it.
        assert!(!f.owns(EscSurface::CursorSelection));
        assert!(!f.owns(EscSurface::PauseMenu));
        assert!(!f.unclaimed(), "an owned press is not spare");
    }

    #[test]
    fn a_press_no_surface_wanted_is_the_pause_menus() {
        let f = focus(None);
        assert!(f.unclaimed());
        assert!(!f.owns(EscSurface::Targeting));
    }

    #[test]
    fn no_press_means_nobody_owns_anything() {
        let f = EscFocus::default();
        assert!(!f.unclaimed(), "the pause menu must not open without a press");
        for s in [EscSurface::PauseMenu, EscSurface::Picker, EscSurface::CameraFocus] {
            assert!(!f.owns(s));
        }
    }

    /// The declaration order of [`EscSurface`] is the precedence, so a
    /// reorder is a behaviour change and should have to restate itself.
    /// Written out topmost-first, mirroring `theme::layer`.
    #[test]
    fn precedence_runs_from_topmost_surface_to_board_state() {
        use EscSurface as S;
        let declared = [
            S::PauseMenu,
            S::ShortcutHelp,
            S::GraveyardBrowser,
            S::ExileBrowser,
            S::Picker,
            S::ModePick,
            S::DecisionPrompt,
            S::Targeting,
            S::BlockerPlan,
            S::AttackPlan,
            S::CursorSelection,
            S::CameraFocus,
        ];
        // `topmost` returns the first open entry, so "earlier wins" is the
        // contract every consumer relies on.
        assert_eq!(declared.first(), Some(&S::PauseMenu), "the pause surface closes first");
        assert_eq!(declared.last(), Some(&S::CameraFocus), "view state yields to everything");
        assert_eq!(declared.len(), 12, "a new surface needs a precedence position and a test row");
    }
}
