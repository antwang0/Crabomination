//! General Jarkeld — switch the blockers of two blocked attackers (CR 509.1,
//! 506.4: a creature can change what it blocks only by an effect).

use super::EffectContext;
use crate::card::CardId;
use crate::effect::Selector;
use crate::game::GameState;

impl GameState {
    /// `Effect::SwitchBlockers`.
    pub(super) fn switch_blockers(&mut self, first: &Selector, second: &Selector, ctx: &EffectContext) {
        let pick = |s: &mut Self, sel: &Selector| {
            s.resolve_selector(sel, ctx).into_iter().find_map(|e| e.as_permanent_id())
        };
        let (Some(a), Some(b)) = (pick(self, first), pick(self, second)) else { return };
        if a == b || !self.attacking.iter().any(|x| x.attacker == a) || !self.attacking.iter().any(|x| x.attacker == b) {
            return;
        }
        let blockers_of = |s: &Self, att: CardId| -> Vec<CardId> {
            s.block_map.iter().filter(|(_, list)| list.contains(&att)).map(|(blk, _)| *blk).collect()
        };
        let (of_a, of_b) = (blockers_of(self, a), blockers_of(self, b));
        if of_a.is_empty() || of_b.is_empty() {
            return;
        }
        // "If each of those creatures could be blocked by all creatures that
        // the other is blocked by."
        if !of_a.iter().all(|&blk| self.blocker_can_block_attacker(blk, b))
            || !of_b.iter().all(|&blk| self.blocker_can_block_attacker(blk, a))
        {
            return;
        }
        let movers: Vec<CardId> = of_a
            .iter()
            .chain(of_b.iter())
            .copied()
            .filter(|blk| of_a.contains(blk) != of_b.contains(blk))
            .collect();
        for blk in movers {
            if let Some(list) = self.block_map.get_mut(&blk) {
                for att in list.iter_mut() {
                    if *att == a {
                        *att = b;
                    } else if *att == b {
                        *att = a;
                    }
                }
            }
        }
        // The CR 510.1c orders and assignments were for the old blocks.
        self.combat_damage_order.remove(&a);
        self.combat_damage_order.remove(&b);
        self.combat_damage_assignment.remove(&a);
        self.combat_damage_assignment.remove(&b);
    }
}
