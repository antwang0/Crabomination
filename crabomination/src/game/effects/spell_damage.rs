//! CR 615 — "If a spell you control would deal damage to an opponent, prevent
//! that damage and create a token for each 1 prevented" (Hostility), and the
//! "whenever a source you control deals damage" listeners a resolving spell
//! reaches (The Red Terror, Ghyrson Starn).

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

    /// "Whenever a [source] you control deals damage" (`YourControl` scope on
    /// `DealsDamage` / `DealsDamageToPlayer` / `DealsDamageToCreature`) for a
    /// RESOLVING SPELL source — the combat dispatcher only walks for a
    /// permanent, so a burn spell was never seen. Each delivery is its own
    /// batch, as a permanent's non-combat damage is.
    pub(crate) fn fire_spell_damage_listeners(&mut self, src: CardId, to: crate::game::types::Target, amount: u32) {
        use crate::effect::{EventKind, EventScope};
        use crate::game::effects::EntityRef;
        use crate::game::types::Target;
        if amount == 0 || self.battlefield_find(src).is_some() {
            return;
        }
        let Some(caster) = self.scratch.resolving_source.as_ref().and_then(|(id, c, ..)| (*id == src).then_some(*c)) else {
            return;
        };
        let specific = match to {
            Target::Player(_) => EventKind::DealsDamageToPlayer,
            Target::Permanent(_) => EventKind::DealsDamageToCreature,
        };
        let to_creature = match to {
            Target::Permanent(id) => self.battlefield_find(id).is_some_and(|c| self.computed_is_creature(c)),
            Target::Player(_) => true,
        };
        let stripped = self.stripped_permanents();
        let mut fires: Vec<(CardId, Effect)> = Vec::new();
        for c in self.battlefield.iter().filter(|c| c.controller == caster && !stripped.contains(&c.id)) {
            for ta in &c.definition.triggered_abilities {
                if ta.event.scope != EventScope::YourControl {
                    continue;
                }
                let kind_ok = ta.event.kind == EventKind::DealsDamage || (ta.event.kind == specific && to_creature);
                if !kind_ok {
                    continue;
                }
                let mut ctx = EffectContext::for_trigger(c.id, caster, Some(to.clone()), 0);
                ctx.trigger_source = Some(EntityRef::Card(src));
                ctx.event_amount = amount;
                if ta.event.filter.as_ref().is_none_or(|f| self.evaluate_predicate(f, &ctx)) {
                    fires.push((c.id, ta.effect.clone()));
                }
            }
        }
        for (listener, effect) in fires {
            self.push_stack(
                crate::game::TriggerPush::new(listener, caster, effect)
                    .target(Some(to.clone()))
                    .trigger_source(Some(EntityRef::Card(src)))
                    .trigger_player(match to {
                        Target::Player(p) => Some(p),
                        Target::Permanent(_) => None,
                    })
                    .event_amount(amount)
                    .build(),
            );
        }
    }
}
