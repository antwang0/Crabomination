//! "[Player] exiles a card from their hand; if it's a creature card, they
//! create a token copy of it" (Arcane Artisan).

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::effects::{EffectContext, EntityRef};
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::ExileFromHandCopyCreature` — each resolved player picks a
    /// card from their hand (a bot its highest-mana-value creature, else its
    /// cheapest card) and exiles it; a creature card becomes a token copy
    /// under that player.
    pub(super) fn exile_from_hand_copy_creature(
        &mut self,
        who: &Selector,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let seats: Vec<usize> = self
            .resolve_selector(who, ctx)
            .into_iter()
            .filter_map(|e| match e {
                EntityRef::Player(p) => Some(p),
                _ => None,
            })
            .collect();
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut picks: Vec<(usize, CardId)> = Vec::new();
        for seat in seats {
            let hand = &self.players[seat].hand;
            if hand.is_empty() {
                continue;
            }
            let candidates: Vec<(CardId, String)> =
                hand.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
            let auto = hand
                .iter()
                .filter(|c| c.definition.is_creature())
                .max_by_key(|c| c.definition.cost.cmc())
                .or_else(|| hand.iter().min_by_key(|c| c.definition.cost.cmc()))
                .map(|c| vec![c.id])
                .unwrap_or_default();
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                seat,
                "Exile a card from your hand (a creature card becomes a token copy)".into(),
                source,
                candidates,
                1,
                1,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            if let Some(id) = picked.first() {
                picks.push((seat, *id));
            }
        }
        self.clear_answer_log();
        for (seat, id) in picks {
            let Some(card) = self.players[seat].remove_from_hand(id) else { continue };
            let creature = card.definition.is_creature();
            self.note_exiled_from_hand_or_by(seat);
            self.exile.push(card);
            events.push(GameEvent::PermanentExiled { card_id: id });
            if creature {
                self.run_effect(
                    &Effect::CreateTokenCopyOf {
                        who: PlayerRef::Seat(seat),
                        count: Value::ONE,
                        source: Selector::ExactObjects(vec![id]),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                    ctx,
                    events,
                )?;
            }
        }
        Ok(())
    }
}
