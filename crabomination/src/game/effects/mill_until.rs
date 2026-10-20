//! `Effect::MillUntilMatching` — "mills a card, then repeats this process
//! until a [filter] card or N cards have been put into their graveyard this
//! way, whichever comes first" (Helm of Obedience, CR 701.13).

use crate::card::SelectionRequirement;
use crate::effect::{Selector, Value};
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::{GameError, GameEvent, GameState};

impl GameState {
    pub(super) fn mill_until_matching(
        &mut self,
        who: &Selector,
        filter: &SelectionRequirement,
        max: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let max = self.evaluate_value(max, ctx).max(0) as usize;
        for ent in self.resolve_selector(who, ctx) {
            let EntityRef::Player(p) = ent else { continue };
            for _ in 0..max {
                if self.players[p].library.is_empty() {
                    break;
                }
                let card = self.players[p].library.remove(0);
                let hit = self.off_battlefield_card_matches(filter, &card, ctx.controller);
                let cid = card.id;
                if !self.route_to_graveyard(card, events) {
                    self.note_milled(p, cid);
                    events.push(GameEvent::CardMilled { player: p, card_id: cid });
                }
                self.scratch.last_moved_cards.push(cid);
                if hit {
                    break;
                }
            }
        }
        Ok(())
    }
}
