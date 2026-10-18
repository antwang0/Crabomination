//! CR 732.2a — an optional loop a copier runs by re-aiming a copy at itself.
//! Naru Meha copying a Displace / Ghostly Flicker that is still on the stack
//! blinks Naru, whose new ETB copies the original again: real rules let the
//! copy's controller stop by keeping the original targets, which CR 400.7 made
//! illegal, so that copy does nothing. A blink keeps its `CardId` here (the
//! FEATURE_ROADMAP CR 400.7 row), so the kept targets would still be legal and
//! the loop never ended (pod seed 924424, 77002 g2: 52,838 actions to the cap).

use super::GameState;
use crate::card::CardId;

/// Copies one trigger source may make of one original spell in a turn. Each
/// one needs a fresh trigger of the same source, so a finite turn never comes
/// near it.
pub const COPIER_REPEATS_PER_TURN: u32 = 8;

impl GameState {
    /// Count a copy `copier` makes of `original`; `false` once that pair is
    /// past [`COPIER_REPEATS_PER_TURN`] this turn — the controller stops the
    /// loop and the copy is not made.
    pub(crate) fn admit_copier_repeat(&mut self, copier: CardId, original: CardId) -> bool {
        let turn = self.turn_number;
        let n = match self.cold.copier_repeat {
            Some((t, c, o, n)) if t == turn && c == copier && o == original => n + 1,
            _ => 1,
        };
        self.cold.copier_repeat = Some((turn, copier, original, n));
        n <= COPIER_REPEATS_PER_TURN
    }
}
