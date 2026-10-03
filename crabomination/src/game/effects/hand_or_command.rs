//! "From your hand or from the command zone onto the battlefield" (Next of
//! Kin): one pick over both zones, the controller's; headless, the priciest.

use super::EffectContext;
use crate::card::{CardId, SelectionRequirement};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    pub(super) fn put_from_hand_or_command_zone(
        &mut self,
        filter: &SelectionRequirement,
        then: Option<&Effect>,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let p = ctx.controller;
        let mut cands: Vec<(u32, CardId, String)> = self.players[p]
            .hand
            .iter()
            .chain(self.players[p].command.iter().filter(|c| c.owner == p))
            .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
            .map(|c| (c.definition.cost.cmc(), c.id, c.definition.name.to_string()))
            .collect();
        if cands.is_empty() {
            return Ok(());
        }
        cands.sort_by_key(|c| std::cmp::Reverse(c.0));
        let default = vec![cands[0].1];
        let Some(chosen) = self.choose_up_to_cards(
            p,
            "Put a card from your hand or the command zone onto the battlefield?".into(),
            ctx.source.unwrap_or(CardId(0)),
            cands.iter().map(|c| (c.1, c.2.clone())).collect(),
            1,
            PickValue::Gain,
            effect,
            default,
        ) else {
            return Ok(());
        };
        let Some(&cid) = chosen.first() else { return Ok(()) };
        let dest = ZoneDest::Battlefield { controller: PlayerRef::Seat(p), tapped: false };
        if self.players[p].hand.iter().any(|c| c.id == cid) {
            self.move_card_to(cid, &dest, ctx, events);
        } else if let Some(pos) = self.players[p].command.iter().position(|c| c.id == cid) {
            // `move_card_to` lifts a card out of the command zone only for its
            // own ability (CR 400.7); this one names the card it picked.
            let card = self.players[p].command.remove(pos);
            self.place_card_in_dest(card, p, &dest, events);
        } else {
            return Ok(());
        }
        self.scratch.last_moved_cards.push(cid);
        if let Some(then) = then {
            self.run_effect(then, ctx, events)?;
        }
        Ok(())
    }
}
