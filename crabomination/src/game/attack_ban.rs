//! "[That player] can't attack you or permanents you control during their
//! next turn" (CR 508.1, The Second Doctor) — a ban held on the attacker's
//! `Player`, live on their first turn after it was registered.

use crate::effect::PlayerRef;
use crate::game::GameState;
use crate::game::effects::EffectContext;

impl GameState {
    /// Registers `who`'s ban on attacking `ctx.controller` and their permanents.
    pub(crate) fn ban_attacks_next_turn(&mut self, who: &PlayerRef, ctx: &EffectContext) {
        let Some(a) = self.resolve_player(who, ctx) else { return };
        let (d, turn) = (ctx.controller, self.turn_number);
        if a == d {
            return;
        }
        if let Some(p) = self.players.get_mut(a)
            && !p.next_turn_attack_bans.contains(&(d, turn))
        {
            p.next_turn_attack_bans.push((d, turn));
        }
    }

    /// True when `seat` may not attack `defender` (or what they control) now.
    pub(crate) fn next_turn_attack_banned(&self, seat: usize, defender: usize) -> bool {
        self.players
            .get(seat)
            .is_some_and(|p| p.next_turn_attack_bans.iter().any(|&(d, t)| d == defender && self.turn_number > t))
    }

    /// Cleanup: the active player's bans that were live this turn end.
    pub(crate) fn expire_attack_bans(&mut self) {
        let (active, turn) = (self.active_player_idx, self.turn_number);
        if let Some(p) = self.players.get_mut(active)
            && !p.next_turn_attack_bans.is_empty()
        {
            p.next_turn_attack_bans.retain(|&(_, t)| turn <= t);
        }
    }
}
