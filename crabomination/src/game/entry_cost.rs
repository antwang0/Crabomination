//! CR 614.1c — entry-cost replacements: "If this would enter, [cost]
//! instead. If you do, put it onto the battlefield. If you don't, put it into
//! its owner's graveyard" (Mox Diamond, Lotus Vale, Scorched Ruins).
//!
//! Asked and paid *before* the permanent is placed, so a refused entry never
//! happens: no ETB, no "whenever a permanent enters" (Mox Diamond's ruling).
//! Every entry hop reads it — the cast (`stack.rs`), the move
//! (`effects/movement.rs`), the land drop (`actions.rs::place_land_card`) and
//! the token mint (`mod.rs::mint_token_with_counters`, after a copy applies).

use crate::card::{CardInstance, WardCost};
use crate::decision::{Decision, DecisionAnswer, OptionalKind, PayFor};
use crate::effect::StaticEffect;
use crate::game::effects::EffectContext;
use crate::game::{GameEvent, GameState, Target};

impl GameState {
    /// The hot-path probe every entry hop runs: one static scan, no clone.
    pub(crate) fn has_entry_cost(card: &CardInstance) -> bool {
        card.definition.static_abilities.iter().any(|sa| matches!(sa.effect, StaticEffect::EntersOnlyIfPaid { .. }))
    }

    fn entry_cost(card: &CardInstance) -> Option<(WardCost, bool)> {
        card.definition.static_abilities.iter().find_map(|sa| match &sa.effect {
            StaticEffect::EntersOnlyIfPaid { cost, optional } => Some((cost.clone(), *optional)),
            _ => None,
        })
    }

    /// Pay `card`'s entry cost for `controller`. `true` when the entry is
    /// refused — unpaid, unpayable, or (`optional`) declined — and the caller
    /// puts the card into its owner's graveyard instead.
    pub(crate) fn entry_refused(&mut self, card: &CardInstance, controller: usize, events: &mut Vec<GameEvent>) -> bool {
        let Some((cost, optional)) = Self::entry_cost(card) else { return false };
        if controller >= self.players.len() {
            return true;
        }
        if optional {
            if !self.entry_discard_payable(&cost, controller, card) {
                return true;
            }
            let decision = Decision::OptionalTrigger {
                source: card.id,
                description: format!("Pay {}'s entry cost? (If you don't, it goes to the graveyard.)", card.definition.name),
                kind: OptionalKind::PayMana { cost: None, purpose: PayFor::SaveSpell },
            };
            if !matches!(self.ask_entering(controller, &decision), DecisionAnswer::Bool(true)) {
                return true;
            }
        }
        let ctx = EffectContext::for_ability(card.id, controller, None);
        !self.try_pay_ward_cost(controller, &cost, &ctx, events)
    }

    /// A cheap read of whether a "you may" entry cost could be paid, so the
    /// question is only put to a player who has a choice. Non-discard costs
    /// read as payable and the payment itself answers.
    fn entry_discard_payable(&self, cost: &WardCost, controller: usize, card: &CardInstance) -> bool {
        let WardCost::DiscardMatching(filter, n) = cost else { return true };
        let have = self.players[controller]
            .hand
            .iter()
            .filter(|c| self.evaluate_requirement_static(filter, &Target::Permanent(c.id), controller, Some(card.id)))
            .count();
        have >= *n as usize
    }
}
