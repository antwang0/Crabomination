//! Hurkyl, Master Wizard — "reveal the top five cards of your library. For
//! each card type among noncreature spells you've cast this turn, you may
//! put a card of that type from among the revealed cards into your hand. Put
//! the rest on the bottom of your library in a random order."
//! (`Effect::RevealTopTakeOnePerCastType`.) One pick, held to one card per
//! type: a card fills one type slot, Kuhn's augmenting paths decide whether
//! the picked set fits.

use super::EffectContext;
use crate::card::{CardId, CardInstance, CardType};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef};
use crate::game::GameState;
use crate::game::types::GameError;

/// A largest set of `fits` rows (one per card, in preference order) that
/// takes distinct type slots: the chosen row indices.
fn largest_distinct(fits: &[Vec<bool>]) -> Vec<usize> {
    fn augment(i: usize, fits: &[Vec<bool>], seen: &mut [bool], slot_of: &mut [Option<usize>]) -> bool {
        for s in 0..seen.len() {
            if fits[i][s] && !seen[s] {
                seen[s] = true;
                if slot_of[s].is_none_or(|j| augment(j, fits, seen, slot_of)) {
                    slot_of[s] = Some(i);
                    return true;
                }
            }
        }
        false
    }
    let slots = fits.first().map_or(0, Vec::len);
    let mut slot_of = vec![None; slots];
    let mut out = Vec::new();
    for i in 0..fits.len() {
        let mut seen = vec![false; slots];
        if augment(i, fits, &mut seen, &mut slot_of) {
            out.push(i);
        }
    }
    out
}

impl GameState {
    /// The card types among noncreature spells `p` cast this turn.
    fn noncreature_cast_types_this_turn(&self, p: usize) -> Vec<CardType> {
        let mut out: Vec<CardType> = Vec::new();
        for &id in &self.players[p].spell_ids_cast_this_turn {
            let Some(c) = self.find_card_anywhere(id) else { continue };
            let def = &c.definition;
            if def.card_types.contains(&CardType::Creature) {
                continue;
            }
            for t in &def.card_types {
                if *t != CardType::Land && !out.contains(t) {
                    out.push(t.clone());
                }
            }
        }
        out
    }

    pub(super) fn reveal_top_take_one_per_cast_type(
        &mut self,
        who: &PlayerRef,
        count: &crate::effect::Value,
        ctx: &EffectContext,
        effect: &Effect,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(self.players[p].library.len());
        if n == 0 {
            return Ok(());
        }
        let types = self.noncreature_cast_types_this_turn(p);
        let fits = |c: &CardInstance| -> Vec<bool> { types.iter().map(|t| c.definition.card_types.contains(t)).collect() };
        // Best first: a higher mana value is the likelier keeper.
        let mut cands: Vec<(CardId, String, Vec<bool>, u32)> = self.players[p].library[..n]
            .iter()
            .map(|c| (c.id, c.definition.name.to_string(), fits(c), c.definition.cost.cmc()))
            .filter(|c| c.2.iter().any(|&f| f))
            .collect();
        cands.sort_by_key(|c| std::cmp::Reverse(c.3));
        let keep: Vec<CardId> = if cands.is_empty() {
            Vec::new()
        } else {
            let rows: Vec<Vec<bool>> = cands.iter().map(|c| c.2.clone()).collect();
            let auto: Vec<CardId> = largest_distinct(&rows).into_iter().map(|i| cands[i].0).collect();
            let mut cursor = 0;
            let source = ctx.source.unwrap_or(CardId(0));
            let Some(picked) = self.ask_seat_cards_logged(
                &mut cursor,
                p,
                "Put up to one card of each cast type into your hand".into(),
                source,
                cands.iter().map(|c| (c.0, c.1.clone())).collect(),
                0,
                types.len() as u32,
                PickValue::Gain,
                effect,
                auto,
            ) else {
                return Ok(());
            };
            self.clear_answer_log();
            // Keep the pick only as far as it fits distinct types, in the
            // order picked.
            let mut order: Vec<usize> = (0..cands.len()).filter(|&i| picked.contains(&cands[i].0)).collect();
            order.sort_by_key(|&i| picked.iter().position(|id| *id == cands[i].0));
            let rows: Vec<Vec<bool>> = order.iter().map(|&i| cands[i].2.clone()).collect();
            largest_distinct(&rows).into_iter().map(|k| cands[order[k]].0).collect()
        };
        let top: Vec<CardInstance> = self.players[p].library.drain(..n).collect();
        let mut rest = Vec::new();
        for card in top {
            if keep.contains(&card.id) {
                self.put_in_hand(p, card, crate::card::Zone::Library);
            } else {
                rest.push(card);
            }
        }
        rest.shuffle(&mut self.rng.draw());
        self.players[p].library.extend(rest);
        Ok(())
    }
}
