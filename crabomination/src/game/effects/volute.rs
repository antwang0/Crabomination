//! Spellweaver Volute — an Aura on an instant card in a graveyard, modelled
//! as a link: the enchanted card is the Aura's `chosen_permanent`.

use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, ZoneDest};
use crate::game::effects::EffectContext;
use crate::game::types::GameEvent;
use crate::game::{GameError, GameState};

impl GameState {
    /// Instant cards in any graveyard, other than `except` (priciest first).
    fn graveyard_instants(&self, except: Option<CardId>) -> Vec<(CardId, String)> {
        let mut out: Vec<(u32, CardId, String)> = self
            .players
            .iter()
            .flat_map(|p| p.graveyard.iter())
            .filter(|c| c.definition.is_instant() && Some(c.id) != except)
            .map(|c| (c.definition.cost.cmc(), c.id, c.definition.name.to_string()))
            .collect();
        out.sort_by_key(|&(mv, id, _)| (std::cmp::Reverse(mv), id));
        out.into_iter().map(|(_, id, n)| (id, n)).collect()
    }

    fn in_a_graveyard(&self, id: CardId) -> bool {
        self.players.iter().any(|p| p.graveyard.iter().any(|c| c.id == id && c.definition.is_instant()))
    }

    /// `Effect::SpellweaverVolute`. `attach`: the enters half (choose the
    /// enchanted card). Otherwise the sorcery-cast half: copy the enchanted
    /// card, offer the copy free, and on a cast exile the card and move to
    /// another instant card in a graveyard. With nothing to enchant the Aura
    /// goes to the graveyard (CR 704.5m, checked here).
    pub(super) fn spellweaver_volute(
        &mut self,
        attach: bool,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let Some(src) = ctx.source else { return Ok(()) };
        let Some(linked) = self.battlefield_find(src).map(|c| c.chosen_permanent) else { return Ok(()) };
        let current = linked.filter(|&id| self.in_a_graveyard(id));
        if !attach && current.is_none() {
            self.move_card_to(src, &ZoneDest::Graveyard, ctx, events);
            return Ok(());
        }
        let cands = self.graveyard_instants(current);
        let pick = if cands.is_empty() {
            None
        } else {
            let auto = vec![cands[0].0];
            let Some(picked) = self.choose_up_to_cards(
                ctx.controller,
                if attach { "Enchant an instant card in a graveyard" } else { "Move to another instant card" }.into(),
                src,
                cands.clone(),
                1,
                PickValue::Gain,
                effect,
                auto.clone(),
            ) else {
                return Ok(());
            };
            picked.into_iter().find(|id| cands.iter().any(|(c, _)| c == id)).or(auto.first().copied())
        };
        if !attach {
            let Some(card) = current else { return Ok(()) };
            if !self.copy_card_and_cast_free(card, ctx, events) {
                return Ok(());
            }
            self.move_card_to(card, &ZoneDest::Exile, ctx, events);
        }
        let still_here = self.battlefield_find(src).is_some();
        match pick.filter(|&id| self.in_a_graveyard(id)) {
            Some(id) if still_here => {
                if let Some(c) = self.battlefield_find_mut(src) {
                    c.chosen_permanent = Some(id);
                }
            }
            _ if still_here => self.move_card_to(src, &ZoneDest::Graveyard, ctx, events),
            _ => {}
        }
        Ok(())
    }
}
