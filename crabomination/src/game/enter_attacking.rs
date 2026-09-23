//! CR 508.4 — a permanent put onto the battlefield attacking (Mobilize,
//! Myriad, Ninjutsu, "create a token that's tapped and attacking") was never
//! declared as an attacker: it is an attacking creature, but "whenever ~
//! attacks" abilities don't trigger for it. Every such site funnels through
//! here so none emits `GameEvent::AttackerDeclared` — the event the Attacks
//! triggers read. A General Kreat with a Goblin army otherwise minted a
//! token per token per attack, without bound.

use super::GameState;
use super::types::{Attack, AttackTarget};
use crate::card::CardId;

impl GameState {
    /// Mark `id` (already on the battlefield) as attacking `target`, without
    /// declaring it (CR 508.4). Returns false if it isn't on the battlefield.
    pub(crate) fn put_into_combat_attacking(&mut self, id: CardId, target: AttackTarget) -> bool {
        let Some(c) = self.battlefield.find_by_id_mut(id) else { return false };
        c.attacked_this_turn = true;
        self.attacking.push(Attack { attacker: id, target });
        true
    }
}
