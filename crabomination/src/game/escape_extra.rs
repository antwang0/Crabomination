//! CR 702.138 — an escape cost that also exiles a permanent you control
//! (`CardDefinition::escape_exile_permanent`, Lunar Hatchling's "Exile a land
//! you control"). The pick rides in `CastEscape::exile_cards` beside the
//! graveyard cards, so the action shape is unchanged.

use crate::card::{CardId, CardInstance};
use crate::effect::ZoneDest;
use crate::game::GameState;
use crate::game::effects::EffectContext;
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    /// Splits `exile_cards` into the graveyard picks and the permanent the
    /// escape cost exiles. A card without the cost passes through unchanged;
    /// with it, a battlefield id in the list is the pick (it must match and
    /// be yours), else the cheapest match — tapped first, then lowest mana
    /// value. No match makes the cast illegal (CR 601.2h).
    pub(crate) fn split_escape_permanent(
        &self,
        p: usize,
        card: &CardInstance,
        exile_cards: &[CardId],
    ) -> Result<(Vec<CardId>, Option<CardId>), GameError> {
        let Some(filter) = card.definition.escape_exile_permanent.as_ref() else {
            return Ok((exile_cards.to_vec(), None));
        };
        let matches = |c: &CardInstance| {
            c.controller == p && self.evaluate_requirement_static(filter, &Target::Permanent(c.id), p, Some(card.id))
        };
        let (on_board, graveyard): (Vec<CardId>, Vec<CardId>) =
            exile_cards.iter().partition(|id| self.battlefield_find(**id).is_some());
        let pick = match on_board.as_slice() {
            [] => self
                .battlefield
                .iter()
                .filter(|c| matches(c))
                .min_by_key(|c| (!c.tapped, c.definition.cost.cmc(), c.id))
                .map(|c| c.id),
            [id] => self.battlefield_find(*id).filter(|c| matches(c)).map(|c| c.id),
            _ => None,
        };
        let pick = pick.ok_or(GameError::SelectionRequirementViolated)?;
        Ok((graveyard, Some(pick)))
    }

    /// Pays the permanent half of the escape cost.
    pub(crate) fn exile_escape_permanent(&mut self, p: usize, id: CardId, events: &mut Vec<GameEvent>) {
        let ctx = EffectContext::for_spell(p, None, 0, 0);
        self.move_card_to(id, &ZoneDest::Exile, &ctx, events);
    }
}
