//! "Target opponent reveals X cards from their hand. You choose one of those
//! cards. That player exiles it." (Taster of Wares) —
//! `Effect::RevealFromHandExileChosen`.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::{DecisionAnswer, PickValue};
use crate::effect::{Effect, PlayerRef, Value, ZoneDest};
use crate::game::GameState;
use crate::game::types::{GameError, GameEvent};

impl GameState {
    /// The revealer picks which `count` cards to show (a headless seat shows
    /// the ones it would discard first), then the controller names one of
    /// them, which its owner exiles (`LastMoved`). Every ask comes before the
    /// card moves, so a suspended ask replays the same hand.
    pub(super) fn reveal_from_hand_exile_chosen(
        &mut self,
        who: &PlayerRef,
        count: &Value,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(self.players[p].hand.len());
        if n == 0 {
            return Ok(());
        }
        let source = ctx.source.unwrap_or(CardId(0));
        let hand: Vec<(CardId, String)> =
            self.players[p].hand.iter().map(|c| (c.id, c.definition.name.to_string())).collect();
        let mut cursor = 0;
        let mut shown: Vec<CardId> = Vec::new();
        if n == hand.len() {
            shown = hand.iter().map(|(id, _)| *id).collect();
        } else if self.seat_suspends(p) {
            let auto: Vec<CardId> = hand.iter().take(n).map(|(id, _)| *id).collect();
            let Some(ids) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                format!("Reveal {n} from your hand"),
                source,
                hand.clone(),
                n as u32,
                n as u32,
                PickValue::Cost,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            shown = ids;
        } else if let DecisionAnswer::Discard(ids) =
            self.decider.decide(&crate::decision::Decision::Discard { player: p, count: n as u32, hand: hand.clone() })
        {
            for id in ids {
                if hand.iter().any(|(h, _)| *h == id) && !shown.contains(&id) {
                    shown.push(id);
                }
            }
        }
        for (id, _) in &hand {
            if shown.len() >= n {
                break;
            }
            if !shown.contains(id) {
                shown.push(*id);
            }
        }
        shown.truncate(n);
        // The chooser's ballot leads with the costliest card.
        shown.sort_by_key(|id| {
            std::cmp::Reverse(self.players[p].hand.iter().find(|c| c.id == *id).map_or(0, |c| c.definition.cost.cmc()))
        });
        let ballot: Vec<String> =
            shown.iter().filter_map(|id| hand.iter().find(|(h, _)| h == id).map(|(_, name)| name.clone())).collect();
        let Some(idx) =
            self.ask_seat_option(&mut cursor, ctx.controller, "Choose a card to exile".to_string(), source, ballot, effect)
        else {
            return Ok(());
        };
        self.clear_answer_log();
        let Some(&cid) = shown.get(idx) else { return Ok(()) };
        let Some(i) = self.players[p].hand.iter().position(|c| c.id == cid) else { return Ok(()) };
        let card = self.players[p].hand.remove(i);
        self.note_exiled_from_hand_or_by(p);
        self.place_card_in_dest(card, p, &ZoneDest::Exile, events);
        // CR 607.2a — linked to the source that exiled it.
        if let Some(c) = self.exile.iter_mut().find(|c| c.id == cid) {
            c.exiled_with = Some(source);
        }
        self.scratch.last_moved_cards = vec![cid];
        Ok(())
    }
}
