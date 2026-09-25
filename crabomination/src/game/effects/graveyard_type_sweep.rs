//! Grime Gorger's attack trigger: "exile up to one card of each card type
//! from defending player's graveyard. Put a +1/+1 counter on this creature
//! for each card exiled this way."

use crate::card::{CardId, CardType, CounterType};
use crate::effect::{PlayerRef, Selector, Value, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::ExileOnePerCardTypeFromGraveyardGrow`. A card with several
    /// types stands for one of them (CR 205.2a); cards with fewer types are
    /// placed first, so the pick takes as many cards as the types allow.
    pub(super) fn exile_one_per_card_type_from_graveyard_grow(
        &mut self,
        who: &PlayerRef,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(seat) = self.resolve_player(who, ctx) else { return Ok(()) };
        let mut cards: Vec<(CardId, Vec<CardType>)> = self.players[seat]
            .graveyard
            .iter()
            .map(|c| (c.id, c.definition.card_types.clone()))
            .filter(|(_, t)| !t.is_empty())
            .collect();
        cards.sort_by_key(|(_, t)| t.len());
        let mut used: Vec<CardType> = Vec::new();
        let mut picks = Vec::new();
        for (id, types) in cards {
            if let Some(t) = types.iter().find(|t| !used.contains(t)) {
                used.push(t.clone());
                picks.push(id);
            }
        }
        let n = picks.len() as i32;
        for id in picks {
            self.move_card_to(id, &ZoneDest::Exile, ctx, events);
        }
        if n > 0 {
            self.run_effect(
                &crate::effect::Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(n),
                },
                ctx,
                events,
            )?;
        }
        Ok(())
    }
}
