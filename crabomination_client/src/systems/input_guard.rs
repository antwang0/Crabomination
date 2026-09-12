//! One answer to "does a text surface own the keyboard right now?".
//!
//! Three surfaces capture typing mid-match: the chat bar (`T`), the debug
//! console's card field (`` ` ``), and the export prompt's message field
//! (`X`). `handle_export_prompt_input`'s docstring states the rule for all
//! of them — "the caller's input handlers should bail out early in that
//! same frame to avoid double-binding (typing `x` mid-message must not
//! re-open the prompt, etc.)".
//!
//! That rule was applied by hand at six call sites and missed at four, so
//! typing a chat line fired gameplay shortcuts: `k` in "keep my black
//! bird" submitted Keep on a mulligan prompt, `b` picked Black in a
//! ChooseColor prompt, `i` toggled the life graph and `[` halved animation
//! speed. The three hand-rolled unions also disagreed about whether the
//! export prompt counted.
//!
//! Use [`text_input_active`] as a run condition for a keyboard-only
//! system, or [`TextInputGuard`] inside a system that mixes pointer and
//! keyboard input — there, gating the whole system would break clicking a
//! modal button while chat happens to be open.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::systems::chat::ChatInputState;
use crate::systems::debug_console::DebugConsoleState;
use crate::systems::export_prompt::ExportPromptState;

/// Bundled read-only access to every keyboard-capturing text surface.
///
/// Held by systems that must still service pointer input while typing is
/// in progress; call [`TextInputGuard::typing`] around the keyboard reads
/// only.
#[derive(SystemParam)]
pub struct TextInputGuard<'w> {
    chat: Res<'w, ChatInputState>,
    console: Res<'w, DebugConsoleState>,
    export_prompt: Res<'w, ExportPromptState>,
}

impl TextInputGuard<'_> {
    /// ⚠ A system holding `ResMut` of any of the three resources above
    /// cannot also take this param — `ResMut<T>` + `Res<T>` in one system
    /// is Bevy error B0002, which compiles and then panics at schedule
    /// init. `handle_export_keypress` owns `ResMut<ExportPromptState>`
    /// and so checks `state.active` alongside the other two by hand.
    /// True while any text surface is capturing keystrokes.
    pub fn typing(&self) -> bool {
        self.chat.open || self.console.card_input_focused || self.export_prompt.active
    }
}

/// Run condition: true while a text surface is capturing keystrokes.
///
/// Gate a keyboard-only system with `.run_if(not(text_input_active))`.
pub fn text_input_active(
    chat: Res<ChatInputState>,
    console: Res<DebugConsoleState>,
    export_prompt: Res<ExportPromptState>,
) -> bool {
    chat.open || console.card_input_focused || export_prompt.active
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    /// `TextInputGuard` reads three resources, so a system that also holds
    /// `ResMut` of any of them is an illegal access pair (Bevy B0002).
    /// That conflict **compiles cleanly** and only panics when the schedule
    /// initialises the system — i.e. at launch, on a machine with a GPU,
    /// which CI and the routine container do not have. `handle_export_keypress`
    /// hit exactly this while the guard was being rolled out: it owns
    /// `ResMut<ExportPromptState>`.
    ///
    /// `System::initialize` is where the access set is built and the
    /// conflict is raised, and it needs no resources to be present — so
    /// initialising every guard-holding system here reproduces the launch
    /// check headlessly.
    fn assert_access_is_legal<M, S: IntoSystem<(), (), M>>(system: S) {
        let mut world = World::new();
        IntoSystem::into_system(system).initialize(&mut world);
    }

    #[test]
    fn every_text_input_guard_holder_has_a_legal_access_set() {
        use crate::systems::{camera_zoom, decision_ui, game_ui, kb_cursor, quality, ui};

        assert_access_is_legal(decision_ui::handle_mulligan_buttons);
        assert_access_is_legal(decision_ui::handle_choose_color_buttons);
        assert_access_is_legal(game_ui::handle_auto_pass_toggle);
        assert_access_is_legal(game_ui::handle_export_keypress);
        assert_access_is_legal(game_ui::handle_planar_die_keypress);
        assert_access_is_legal(game_ui::handle_reveal_conspiracy_keypress);
        assert_access_is_legal(camera_zoom::camera_focus_hotkeys);
        assert_access_is_legal(kb_cursor::handle_keyboard_cursor_input);
        assert_access_is_legal(quality::handle_settings_toggle);
        assert_access_is_legal(ui::graveyard_browser);
        assert_access_is_legal(ui::exile_browser);
    }

    /// The shared wheel handler is registered unconditionally in every app
    /// state, so its access set has to be legal too.
    #[test]
    fn the_shared_scroll_handler_has_a_legal_access_set() {
        assert_access_is_legal(crate::systems::scroll::handle_scroll);
    }

    /// The run condition is evaluated as part of the gated system's access
    /// set, so a condition/system clash is the same class of launch panic.
    #[test]
    fn run_condition_gated_systems_have_legal_access_sets() {
        use crate::systems::{animate, game_ui, ui};
        use bevy::ecs::schedule::common_conditions::not;

        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems((
            animate::adjust_animation_speed.run_if(not(super::text_input_active)),
            game_ui::toggle_life_graph.run_if(not(super::text_input_active)),
            ui::toggle_shortcut_help.run_if(not(super::text_input_active)),
        ));
        schedule.initialize(&mut world).expect("schedule initialises");
    }
}
