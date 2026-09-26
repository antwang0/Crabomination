//! CR 508.4 — a permanent put onto the battlefield attacking (Mobilize,
//! Myriad, Ninjutsu, "create a token that's tapped and attacking") was never
//! declared as an attacker: it is an attacking creature, but "whenever ~
//! attacks" abilities don't trigger for it. Every such site funnels through
//! here so none emits `GameEvent::AttackerDeclared` — the event the Attacks
//! triggers read. A General Kreat with a Goblin army otherwise minted a
//! token per token per attack, without bound.
//!
//! Also the CR 506.3 read of an attack's defender ("creatures attacking you").

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

    /// Whether `id` is attacking the player `seat` itself (CR 506.3) — not
    /// a planeswalker or battle of theirs.
    pub(crate) fn creature_is_attacking_seat(&self, id: CardId, seat: usize) -> bool {
        self.attacking.iter().any(|a| a.attacker == id && a.target == AttackTarget::Player(seat))
    }

    /// `id` attacks the player `source` last chose (its `chosen_player`).
    pub(crate) fn attacking_chosen_player_of(&self, id: CardId, source: Option<CardId>) -> bool {
        source
            .and_then(|s| self.battlefield_find(s))
            .and_then(|c| c.chosen_player)
            .is_some_and(|p| self.creature_is_attacking_seat(id, p))
    }

    /// `id` attacks the player the firing trigger event named.
    pub(crate) fn attacking_trigger_player(&self, id: CardId) -> bool {
        self.trigger_event_player_scratch.is_some_and(|p| self.creature_is_attacking_seat(id, p))
    }

    /// `id` attacks an opponent of `seat`, or a planeswalker one controls.
    pub(crate) fn creature_is_attacking_an_opponent_of(&self, id: CardId, seat: usize) -> bool {
        self.attacking.iter().any(|a| {
            a.attacker == id
                && match a.target {
                    AttackTarget::Player(p) => !self.same_team(p, seat),
                    AttackTarget::Planeswalker(pw) => self
                        .battlefield_find(pw)
                        .is_some_and(|c| !self.same_team(c.controller, seat)),
                    AttackTarget::Battle(_) => false,
                }
        })
    }

    /// Attacking an opponent of `seat` directly (not a planeswalker or
    /// battle of theirs).
    pub(crate) fn creature_is_attacking_opponent_player(&self, id: CardId, seat: usize) -> bool {
        self.attacking
            .iter()
            .any(|a| a.attacker == id && matches!(a.target, AttackTarget::Player(p) if !self.same_team(p, seat)))
    }
}
