//! `Effect::LoseUnspentMana` — "that player loses all unspent mana" (Mana
//! Short, Worldpurge). An effect empties the whole pool, kept mana included:
//! the CR 500.4 / 106.4 keepers (Upwelling, Kruphix, Savage Ventmaw) only
//! stop the step-end empty. Yurlok of Scorch Thrash burns the loss.

use crate::card::StaticEffect;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::effects::EffectContext;
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(super) fn lose_unspent_mana(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let burn = self.battlefield.iter().find_map(|c| {
            c.definition
                .static_abilities
                .iter()
                .any(|sa| matches!(sa.effect, StaticEffect::PlayersLoseLifeForUnspentMana))
                .then_some((c.id, c.controller))
        });
        for p in self.resolve_players(who, ctx) {
            let Some(player) = self.players.get(p) else { continue };
            let lost = player.mana_pool.total();
            if lost == 0 && player.kept_mana_this_turn.is_empty() && player.firebending_kept_red == 0 {
                continue;
            }
            let player = &mut self.players[p];
            player.mana_pool.empty();
            player.kept_mana_this_turn = Default::default();
            player.firebending_kept_red = 0;
            if let Some((src, ctrl)) = burn.filter(|_| lost > 0) {
                let loss = Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Seat(p)),
                    amount: Value::Const(lost.min(i32::MAX as u32) as i32),
                };
                self.run_effect(&loss, &EffectContext::for_ability(src, ctrl, None), events)?;
            }
        }
        Ok(())
    }
}
