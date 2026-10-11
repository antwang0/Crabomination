//! CR 723 — controlling another player. The whole-turn grant (Mindslaver) is
//! `apply_pending_player_control`; this is the shared slot write and the
//! CR 723.2 limited-duration form, one combat phase (Secret of Bloodbending).

use super::GameState;

impl GameState {
    /// Point `seat`'s `controlled_by` slot at `controller` (`None` clears it).
    /// CR 723.4 — a new controller sees what the controlled player sees,
    /// starting with their hand.
    pub(crate) fn set_player_controller(&mut self, seat: usize, controller: Option<usize>) {
        if seat >= self.players.len() {
            return;
        }
        if self.controlled_by.len() < self.players.len() {
            self.controlled_by.resize(self.players.len(), None);
        }
        self.controlled_by[seat] = controller;
        if let Some(c) = controller
            && !self.hands_revealed_to.contains(&(c, seat))
        {
            self.hands_revealed_to.push((c, seat));
        }
    }

    /// `Effect::ControlPlayerNextCombat` — `controller` controls `seat` during
    /// `seat`'s next combat phase. CR 723.1a — a newer grant for the same seat
    /// overwrites the older one.
    pub(crate) fn queue_combat_control(&mut self, seat: usize, controller: usize) {
        if seat == controller || !self.players.get(seat).is_some_and(|p| p.is_alive()) {
            return;
        }
        retain_cold!(self.pending_combat_control, |(c, _)| *c != seat);
        self.pending_combat_control.push((seat, controller));
    }

    /// CR 723.2 — the active player's combat phase is beginning: a pending
    /// combat grant on them takes effect now. Called as BeginCombat starts.
    pub(crate) fn begin_combat_control(&mut self) {
        let ap = self.active_player_idx;
        let Some(pos) = self.pending_combat_control.iter().rposition(|(c, _)| *c == ap) else { return };
        let (_, controller) = self.pending_combat_control.remove(pos);
        retain_cold!(self.pending_combat_control, |(c, _)| *c != ap);
        // An extra combat phase that begins while one grant is live (none can:
        // the restore runs as each combat phase ends) keeps the oldest slot.
        if self.combat_control_restore.is_none() {
            self.combat_control_restore = Some((ap, self.controlled_by.get(ap).copied().flatten()));
        }
        self.set_player_controller(ap, Some(controller));
    }

    /// CR 723.2 — the combat phase ended: the slot goes back to what it was
    /// (a whole-turn Mindslaver grant resumes; otherwise the player is theirs).
    pub(crate) fn end_combat_control(&mut self) {
        // Read first: a `take` on the cold group unshares it (CLAUDE.md, CoW).
        let Some((seat, prev)) = self.combat_control_restore else { return };
        self.combat_control_restore = None;
        // CR 800.4b — never hand the seat back to a controller who has left.
        let prev = prev.filter(|&c| self.players.get(c).is_some_and(|p| p.is_alive()));
        self.set_player_controller(seat, prev);
    }

    /// CR 800.4a/b — `p` left the game: no combat grant names them on either
    /// side any longer.
    pub(crate) fn drop_combat_control_for(&mut self, p: usize) {
        retain_cold!(self.pending_combat_control, |(c, ctrl)| *c != p && *ctrl != p);
        if self.combat_control_restore.is_some_and(|(seat, _)| seat == p) {
            self.combat_control_restore = None;
        }
    }
}
