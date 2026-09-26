//! CR 701.14 — "create a token for each [creature]; each of those tokens
//! fights a different one of those creatures" (Ezuri's Predation). One token
//! is paired with each creature read at resolution; a token doubler's extra
//! tokens (CR 614.13) enter too but have no creature of their own to fight.

use crate::card::{SelectionRequirement, TokenDefinition};
use crate::effect::{Effect, Selector};
use crate::game::effects::{EffectContext, token_card_arc};
use crate::game::types::Target;
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(crate) fn create_tokens_to_fight_each(
        &mut self,
        filter: &SelectionRequirement,
        definition: &TokenDefinition,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let you = ctx.controller;
        let prey: Vec<_> = self
            .battlefield
            .iter()
            .filter(|c| {
                self.evaluate_requirement_static(filter, &Target::Permanent(c.id), you, ctx.source)
            })
            .map(|c| c.id)
            .collect();
        let per_creature =
            self.scaled_token_count(you, 1, definition.card_types.contains(&crate::card::CardType::Creature));
        let def = token_card_arc(definition);
        let mut pairs = Vec::with_capacity(prey.len());
        for &victim in &prey {
            let mut first = None;
            for _ in 0..per_creature {
                let id = self.mint_token_onto_battlefield(def.clone(), you, definition.tapped, events);
                first.get_or_insert(id);
            }
            if let Some(token) = first.filter(|&t| self.battlefield_find(t).is_some()) {
                pairs.push((token, victim));
            }
        }
        let fight = Effect::Fight { attacker: Selector::Target(0), defender: Selector::Target(1) };
        for (token, victim) in pairs {
            let fctx = EffectContext {
                targets: vec![Target::Permanent(token), Target::Permanent(victim)],
                ..ctx.clone()
            };
            self.run_effect(&fight, &fctx, events)?;
        }
        Ok(())
    }
}
