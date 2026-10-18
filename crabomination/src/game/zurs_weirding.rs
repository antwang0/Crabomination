//! Zur's Weirding — "If a player would draw a card, they reveal it instead.
//! Then any other player may pay 2 life. If a player does, put that card into
//! its owner's graveyard. Otherwise, that player draws a card." (CR 121.2a /
//! 614; `StaticEffect::DrawsRevealedOthersMayPayLifeToBin`, read as
//! `DrawDig::OthersMayBin` in `draw_one_outcome`.) The other players are
//! asked in APNAP order and the first who pays bins the card; a seat whose
//! life can't cover it isn't asked (CR 119.4). A draw replacement has no stack
//! item to park a prompt on, so a prompting seat's answer is the headless one.

use super::GameState;
use super::types::GameEvent;
use crate::card::CardId;
use crate::effect::StaticEffect;

impl GameState {
    /// The life a Zur's Weirding on the battlefield asks for, and its source.
    pub(crate) fn others_may_bin_draw(&self) -> Option<(CardId, u32)> {
        self.battlefield.iter().find_map(|c| {
            c.definition.static_abilities.iter().find_map(|sa| match self.active_static(&sa.effect, c) {
                Some(StaticEffect::DrawsRevealedOthersMayPayLifeToBin { life }) => Some((c.id, *life)),
                _ => None,
            })
        })
    }

    /// Reveal `p`'s top card; `true` when another player paid and it was
    /// binned (the draw replaced), `false` to draw it as usual.
    pub(crate) fn apply_others_may_bin_draw(&mut self, p: usize, events: &mut Vec<GameEvent>) -> bool {
        use crate::decision::{Decision, DecisionAnswer, OptionalKind, PayFor};
        let Some((src, life)) = self.others_may_bin_draw() else { return false };
        let Some(top) = self.players[p].library.first() else { return false };
        let (id, name) = (top.id, top.definition.name);
        let others: Vec<usize> = self.living_seats().filter(|&q| q != p).collect();
        for q in self.apnap_sort(others) {
            if !self.cost_life_payable(q, self.players[q].life, life) {
                continue;
            }
            let pays = matches!(
                self.decider.decide(&Decision::OptionalTrigger {
                    source: src,
                    description: format!("Pay {life} life to put {name} into its owner's graveyard?"),
                    kind: OptionalKind::PayLife { life, purpose: PayFor::DenyEffect },
                }),
                DecisionAnswer::Bool(true)
            );
            if !pays {
                continue;
            }
            self.pay_life_cost(q, life);
            let card = self.players[p].library.remove(0);
            if !self.route_to_graveyard(card, events) {
                events.push(GameEvent::CardRevealedIntoGraveyard { player: p, card_id: id });
            }
            return true;
        }
        false
    }
}
