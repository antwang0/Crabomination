//! Summary Dismissal — "Exile all other spells and counter all abilities."
//! Exiling is not countering, so an uncounterable spell goes too (CR 701.5a
//! only stops *countering*); every triggered and activated ability on the
//! stack is countered (CR 701.5b).

use super::EffectContext;
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent, StackItem};

impl GameState {
    /// `Effect::ExileAllOtherSpellsCounterAllAbilities`.
    pub(super) fn exile_all_other_spells_counter_all_abilities(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let src = ctx.source;
        let doomed: Vec<usize> = self
            .stack
            .iter()
            .enumerate()
            .filter(|(_, si)| match si {
                StackItem::Spell { card, .. } => Some(card.id) != src,
                StackItem::Trigger { .. } => true,
            })
            .map(|(i, _)| i)
            .collect();
        for pos in doomed.into_iter().rev() {
            if let StackItem::Spell { card, .. } = self.stack.remove(pos)
                // A copy of a spell ceases to exist (CR 707.10a).
                && !card.is_token
            {
                let id = card.id;
                self.exile.push(*card);
                events.push(GameEvent::PermanentExiled { card_id: id });
            }
        }
        Ok(())
    }
}
