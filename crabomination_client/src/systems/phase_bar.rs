//! Per-step priority stops (roadmap Tier 7 #2/#8), layered onto the
//! existing left-edge phase chart.
//!
//! Clicking a chart row cycles a stop override for that step, scoped to the
//! *kind of turn currently shown* (your turn vs. opponents' — MTGO models
//! stops the same way):
//!
//! - **Auto** (default) — `auto_advance_p0`'s built-in behavior: pass
//!   bookkeeping windows, hold when you could act or an opponent spell is
//!   on the stack.
//! - **Stop** — always hold priority at this step, even with nothing to do
//!   (e.g. "always stop at my opponent's end step").
//! - **Skip** — pass this window even when you could act (e.g. never stop
//!   at your own upkeep despite holding an activatable land). Explicit
//!   fast-forwards and opponent spells on the stack still hold.
//!
//! The visual markers ("[stop]" / "[skip]" suffixes and row tints) are
//! rendered by `game_ui::update_phase_chart`, which already rewrites the
//! chart rows every frame.

use std::collections::HashMap;

use bevy::prelude::*;

use crabomination::game::TurnStep;

use crate::net_plugin::CurrentView;
use crate::systems::game_ui::PhaseStepLabel;

/// Per-step auto-pass override. See module docs. Serialized into
/// `config.toml` (`gameplay.stops_my` / `stops_opp`) so stops survive
/// restarts.
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StopMode {
    #[default]
    Auto,
    Always,
    Skip,
}

impl StopMode {
    fn cycled(self) -> StopMode {
        match self {
            StopMode::Auto => StopMode::Always,
            StopMode::Always => StopMode::Skip,
            StopMode::Skip => StopMode::Auto,
        }
    }
}

/// Stop overrides, kept separately for the viewer's own turns and
/// opponents' turns (you usually want different stops defending than
/// attacking).
#[derive(Resource, Default)]
pub struct StopConfig {
    pub my: HashMap<TurnStep, StopMode>,
    pub opp: HashMap<TurnStep, StopMode>,
}

impl StopConfig {
    pub fn mode(&self, my_turn: bool, step: TurnStep) -> StopMode {
        let map = if my_turn { &self.my } else { &self.opp };
        map.get(&step).copied().unwrap_or_default()
    }

    fn cycle(&mut self, my_turn: bool, step: TurnStep) -> StopMode {
        let map = if my_turn { &mut self.my } else { &mut self.opp };
        let next = map.get(&step).copied().unwrap_or_default().cycled();
        if next == StopMode::Auto {
            map.remove(&step);
        } else {
            map.insert(step, next);
        }
        next
    }
}

/// Cycle the clicked phase-chart row's stop mode for the kind of turn
/// currently shown. Untap and Cleanup have no priority window (CR 502.4 /
/// 514.3), so clicks on them are ignored.
pub fn handle_phase_chart_clicks(
    view: Res<CurrentView>,
    mut stops: ResMut<StopConfig>,
    mut ff: ResMut<crate::systems::game_ui::FastForward>,
    mut log: ResMut<crate::game::GameLog>,
    mouse: Res<ButtonInput<MouseButton>>,
    clicked: Query<(&Interaction, &PhaseStepLabel), Changed<Interaction>>,
    hovered: Query<(&Interaction, &PhaseStepLabel)>,
) {
    let Some(cv) = &view.0 else { return };
    let my_turn = cv.active_player == cv.your_seat;
    // Right-click: click-to-advance — auto-pass priority until the game
    // reaches the clicked step (cleared on arrival, or by re-clicking).
    if mouse.just_pressed(MouseButton::Right) {
        for (interaction, row) in &hovered {
            if *interaction == Interaction::None {
                continue;
            }
            if ff.pass_until == Some(row.0) {
                ff.pass_until = None;
                log.push(format!("Cancelled pass-until {:?}", row.0));
            } else if row.0 == cv.step {
                log.push(format!("Already at {:?}", row.0));
            } else {
                ff.pass_until = Some(row.0);
                log.push(format!("Passing until {:?}", row.0));
            }
        }
        return;
    }
    for (interaction, row) in &clicked {
        if *interaction != Interaction::Pressed {
            continue;
        }
        if matches!(row.0, TurnStep::Untap | TurnStep::Cleanup) {
            continue;
        }
        let mode = stops.cycle(my_turn, row.0);
        let turn_kind = if my_turn { "your turns" } else { "opponents' turns" };
        log.push(format!(
            "{:?} on {turn_kind}: {}",
            row.0,
            match mode {
                StopMode::Auto => "auto-pass (default)",
                StopMode::Always => "always stop",
                StopMode::Skip => "skip even with plays",
            }
        ));
    }
}
