//! Balduvian Warlord — "Remove target blocking creature from combat.
//! Creatures it was blocking that hadn't become blocked by another creature
//! this combat become unblocked, then it blocks an attacking creature of
//! your choice." (`Effect::RemoveBlockerThenReblock`.) "Hadn't become blocked
//! by another creature" reads the blockers still in combat; the new block is
//! made by the effect, not declared, so no "whenever ~ blocks" trigger fires.

use super::EffectContext;
use crate::card::CardId;
use crate::decision::PickValue;
use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::types::GameError;

impl GameState {
    pub(super) fn remove_blocker_then_reblock(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let Some(blocker) = self.resolve_selector(what, ctx).into_iter().find_map(|e| e.as_permanent_id()) else {
            return Ok(());
        };
        let Some(was_blocking) = self.block_map.get(&blocker).cloned() else { return Ok(()) };
        self.remove_from_combat(blocker);
        for a in was_blocking {
            if self.blockers_of(a).is_empty() {
                self.blocked_attackers.retain(|x| *x != a);
            }
        }
        if self.battlefield_find(blocker).is_none() {
            return Ok(());
        }
        let mut attackers: Vec<(CardId, String, i32)> = self
            .attacking
            .iter()
            .filter_map(|a| {
                let c = self.battlefield_find(a.attacker)?;
                let power = self.computed_permanent(a.attacker).map_or(0, |cp| cp.power);
                Some((a.attacker, c.definition.name.to_string(), power))
            })
            .collect();
        if attackers.is_empty() {
            return Ok(());
        }
        attackers.sort_by_key(|&(id, _, power)| (std::cmp::Reverse(power), id));
        let mut cursor = 0;
        let Some(pick) = self.ask_seat_cards_logged(
            &mut cursor,
            ctx.controller,
            "Choose an attacking creature for it to block".into(),
            ctx.source.unwrap_or(CardId(0)),
            attackers.iter().map(|a| (a.0, a.1.clone())).collect(),
            1,
            1,
            PickValue::Gain,
            effect,
            vec![attackers[0].0],
        ) else {
            return Ok(());
        };
        self.clear_answer_log();
        let Some(&chosen) = pick.first() else { return Ok(()) };
        self.block_map.insert(blocker, smallvec::smallvec![chosen]);
        if !self.blocked_attackers.contains(&chosen) {
            self.blocked_attackers.push(chosen);
        }
        Ok(())
    }
}
