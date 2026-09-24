//! CR 702.143 — foretell beyond the keyword: a card foretold by an effect
//! with a granted foretell cost (Ethereal Valkyrie), which cards count as
//! foretold (Niko Defies Destiny), and what the special action costs
//! (Ranar the Ever-Watchful's free first foretell).

use crate::card::{CardId, CardInstance};
use crate::decision::PickValue;
use crate::effect::{Effect, StaticEffect};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};
use crate::mana::{ManaCost, ManaSymbol};

impl GameState {
    /// The foretell cost an effect gave `id` (Ethereal Valkyrie), if any.
    pub(crate) fn granted_foretell_cost(&self, id: CardId) -> Option<&ManaCost> {
        self.granted_foretell_costs.iter().find(|(c, _)| *c == id).map(|(_, m)| m)
    }

    /// CR 702.143 — `c` (in exile) is foretold: face down with a foretell
    /// cost, printed or granted.
    pub(crate) fn is_foretold(&self, c: &CardInstance) -> bool {
        c.face_down && (c.definition.foretell_cost.is_some() || self.granted_foretell_cost(c.id).is_some())
    }

    /// The cheaper of a foretold card's foretell costs (CR 702.143 — a card
    /// Ethereal Valkyrie foretold may have two, and may be cast for either).
    pub(crate) fn foretold_cast_cost(&self, c: &CardInstance) -> Option<ManaCost> {
        let granted = self.granted_foretell_cost(c.id).cloned();
        match (c.definition.foretell_cost.clone(), granted) {
            (Some(a), Some(b)) => Some(if b.cmc() < a.cmc() { b } else { a }),
            (a, b) => a.or(b),
        }
    }

    /// CR 702.143a — the foretell special action's cost for `seat`: {2}, or
    /// {0} for the first card they foretell this turn under Ranar. A card
    /// foretold by an effect (Ethereal Valkyrie) wasn't foretold by its owner,
    /// so it doesn't use the free one up.
    pub(crate) fn foretell_action_cost(&self, seat: usize) -> ManaCost {
        let free = self.battlefield.iter().any(|c| {
            c.controller == seat
                && c.definition
                    .static_abilities
                    .iter()
                    .any(|sa| matches!(sa.effect, StaticEffect::FirstForetellEachTurnFree))
        }) && !self.foretold_this_turn.iter().any(|id| {
            self.granted_foretell_cost(*id).is_none()
                && self.exile.iter().any(|c| c.id == *id && c.owner == seat)
        });
        ManaCost { symbols: if free { vec![] } else { vec![ManaSymbol::Generic(2)] } }
    }

    /// `Effect::ForetellFromHand` — the controller exiles a card from their
    /// hand face down; it becomes foretold with its mana cost less {`reduce`}
    /// generic as its foretell cost (Ethereal Valkyrie; the reduction can't
    /// touch colored pips).
    pub(super) fn foretell_from_hand(
        &mut self,
        reduce: u32,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let seat = ctx.controller;
        let candidates: Vec<(CardId, String)> = self.players[seat]
            .hand
            .iter()
            .map(|c| (c.id, c.definition.name.to_string()))
            .collect();
        if candidates.is_empty() {
            self.clear_answer_log();
            return Ok(());
        }
        // A bot foretells its most expensive spell: the discount is worth the
        // most there, and a land can never be cast from exile.
        let auto = self.players[seat]
            .hand
            .iter()
            .filter(|c| !c.definition.is_land())
            .max_by_key(|c| c.definition.cost.cmc())
            .or_else(|| self.players[seat].hand.first())
            .map(|c| vec![c.id])
            .unwrap_or_default();
        let mut cursor = 0;
        let Some(picked) = self.ask_seat_cards_logged(
            &mut cursor,
            seat,
            "Exile a card from your hand face down; it becomes foretold".into(),
            ctx.source.unwrap_or(CardId(0)),
            candidates,
            1,
            1,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let Some(id) = picked.first().copied() else { return Ok(()) };
        let Some(mut card) = self.players[seat].remove_from_hand(id) else {
            return Ok(());
        };
        let mut cost = card.definition.cost.clone();
        cost.reduce_generic(reduce);
        card.face_down = true;
        self.exile.push(card);
        self.foretold_this_turn.insert(id);
        self.granted_foretell_costs.retain(|(c, _)| *c != id);
        self.granted_foretell_costs.push((id, cost));
        self.note_exiled_from_hand_or_by(seat);
        events.push(GameEvent::PermanentExiled { card_id: id });
        Ok(())
    }
}
