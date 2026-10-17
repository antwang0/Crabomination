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
    /// mana value, nonland first.
    pub(super) fn return_distinct_mana_values_from_graveyard(
        &mut self,
        who: &PlayerRef,
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
        let candidates: Vec<(CardId, String)> = cards.iter().map(|(id, _, _, n)| (*id, n.clone())).collect();
        let mut cursor = 0;
        let source = ctx.source.unwrap_or(CardId(0));
        let max = cards.len() as u32;
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
            seen.push(mv);
            self.move_card_to(id, &ZoneDest::Hand(PlayerRef::Seat(p)), ctx, events);
        }
        Ok(())
    }
}
