//! `Effect::ExileThenRevealUntilSharedType` (Chaotic Transformation): the
//! exile twin of Reweave's sacrifice. The shared-card-type filter is built
//! from the permanent's computed types as it is exiled, and its controller
//! then — not the caster — reveals from their own library until a card that
//! shares a card type with it, puts that card onto the battlefield under
//! their control, and shuffles the rest back (CR 701.20 / 400.7).

use super::GameState;
use super::effects::EffectContext;
use super::types::{GameError, GameEvent};
use crate::card::{CardType, SelectionRequirement};
use crate::effect::{Effect, PlayerRef, RevealMissDest, Selector, Value, ZoneDest};

impl GameState {
    pub(crate) fn resolve_exile_then_reveal_until_shared_type(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(cid) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(seat) = self.battlefield_find(cid).map(|c| c.controller) else {
            return Ok(());
        };
        let types: Vec<CardType> = self
            .computed_permanent(cid)
            .map(|cp| cp.card_types().to_vec())
            .unwrap_or_else(|| self.battlefield_find(cid).map(|c| c.definition.card_types.clone()).unwrap_or_default());
        self.run_effect(&Effect::Exile { what: Selector::ExactObjects(vec![cid]) }, ctx, events)?;
        if self.battlefield_find(cid).is_some() {
            // Not exiled (a replacement kept it): nothing was "exiled this way".
            return Ok(());
        }
        let Some(find) = types.into_iter().map(SelectionRequirement::HasCardType).reduce(|a, b| a.or(b)) else {
            return Ok(());
        };
        self.run_effect(
            &Effect::RevealUntilFind {
                who: PlayerRef::Seat(seat),
                find,
                to: ZoneDest::Battlefield { controller: PlayerRef::Seat(seat), tapped: false },
                cap: Value::Const(500),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::ShuffleIntoLibrary,
            },
            ctx,
            events,
        )
    }
}
