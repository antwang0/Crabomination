//! "Return any number of cards with different mana values from your
//! graveyard to your hand" (Seasons Past).

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// `Effect::ReturnDistinctManaValuesFromGraveyard` — the player picks any
    /// number of their graveyard cards; a second card of a mana value already
    /// picked is dropped (CR 202.3). A headless seat takes one card of every
    /// mana value, nonland first. `max` caps the picks (Long Rest's X); the
    /// returned cards are published on `Selector::LastMoved`.
    pub(super) fn return_distinct_mana_values_from_graveyard(
        &mut self,
        who: &PlayerRef,
        max: Option<&crate::effect::Value>,
        effect: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let cards: Vec<(CardId, u32, bool, String)> = self.players[p]
            .graveyard
            .iter()
            .map(|c| (c.id, c.definition.cost.cmc(), c.definition.is_land(), c.definition.name.to_string()))
            .collect();
        if cards.is_empty() {
            return Ok(());
        }
        let mut auto: Vec<CardId> = Vec::new();
        let mut taken: Vec<u32> = Vec::new();
        for land in [false, true] {
            for (id, mv, is_land, _) in &cards {
                if *is_land == land && !taken.contains(mv) {
                    taken.push(*mv);
                    auto.push(*id);
                }
            }
        }
        let cap = max.map_or(cards.len(), |v| self.evaluate_value(v, ctx).max(0) as usize).min(cards.len());
        if cap == 0 {
            return Ok(());
        }
        // A capped pick keeps the greatest mana values first.
        if cap < auto.len() {
            let mv_of = |id: &CardId| cards.iter().find(|(c, ..)| c == id).map_or(0, |c| c.1);
            auto.sort_by_key(|id| std::cmp::Reverse(mv_of(id)));
            auto.truncate(cap);
        }
        let candidates: Vec<(CardId, String)> = cards.iter().map(|(id, _, _, n)| (*id, n.clone())).collect();
        let mut cursor = 0;
        let source = ctx.source.unwrap_or(CardId(0));
        let max = cap as u32;
        let Some(picked) = self.ask_seat_cards_logged(
            &mut cursor,
            p,
            "Return cards with different mana values".into(),
            source,
            candidates,
            0,
            max,
            PickValue::Gain,
            effect,
            auto,
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let mut seen: Vec<u32> = Vec::new();
        for id in picked {
            let Some(&(_, mv, _, _)) = cards.iter().find(|(c, ..)| *c == id) else { continue };
            if seen.contains(&mv) {
                continue;
            }
            if seen.len() == cap {
                break;
            }
            seen.push(mv);
            self.move_card_to(id, &ZoneDest::Hand(PlayerRef::Seat(p)), ctx, events);
            if self.players[p].hand.iter().any(|c| c.id == id) {
                self.scratch.last_moved_cards.push(id);
            }
        }
        Ok(())
    }
}
