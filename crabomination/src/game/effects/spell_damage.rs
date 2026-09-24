//! CR 615 — "If a spell you control would deal damage to an opponent, prevent
//! that damage and create a token for each 1 prevented" (Hostility).

use crate::card::CardId;
use crate::effect::{Effect, PlayerRef, StaticEffect, Value};
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;

impl GameState {
    /// Prevent `amount` damage the resolving spell `source` would deal to
    /// `victim` when an opponent of `victim` who controls that spell holds a
    /// `SpellDamageToOpponentsBecomesTokens` permanent, minting its tokens.
    /// Returns `true` when the damage was prevented.
    pub(crate) fn spell_damage_becomes_tokens(
        &mut self,
        source: Option<CardId>,
        victim: usize,
        amount: u32,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        if amount == 0 || self.damage_cant_be_prevented_this_turn {
            return false;
        }
        let (Some(src), Some(caster)) = (source, self.resolving_spell_caster) else { return false };
        if self.scratch.resolving_source.as_ref().is_none_or(|(id, ..)| *id != src)
            || !self.opponents_of(caster).contains(&victim)
        {
            return false;
        }
        let Some((holder, token)) = self.battlefield.iter().filter(|c| c.controller == caster).find_map(|c| {
            c.definition.static_abilities.iter().find_map(|sa| match &sa.effect {
                StaticEffect::SpellDamageToOpponentsBecomesTokens { token } => Some((c.id, token.clone())),
                _ => None,
            })
        }) else {
            return false;
        };
        let ctx = EffectContext::for_ability(holder, caster, None);
        let _ = self.run_effect(
            &Effect::CreateToken { who: PlayerRef::You, count: Value::Const(amount as i32), definition: token },
            &ctx,
            events,
        );
        true
    }
}
