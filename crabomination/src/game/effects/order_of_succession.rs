//! Order of Succession — "choose left or right; each player chooses a creature
//! controlled by the next player in that direction and gains control of it"
//! (CR 101.4's seating order; left is the next seat in turn order).

use super::EffectContext;
use crate::card::CardId;
use crate::game::GameState;

impl GameState {
    /// `Effect::EachPlayerTakesCreatureOfNext`. Every chooser takes the next
    /// living player's most valuable creature; the caster picks the direction
    /// that nets it the most (left on a tie). The control changes happen
    /// together, after every choice is made.
    pub(super) fn each_player_takes_creature_of_next(&mut self, ctx: &EffectContext) {
        let n = self.players.len();
        let alive: Vec<usize> = (0..n).filter(|&p| self.players[p].is_alive()).collect();
        if alive.len() < 2 {
            return;
        }
        let value = |id: CardId| {
            self.battlefield_find(id).map_or(0, |c| {
                let (p, t) = self.computed_permanent(id).map_or((0, 0), |cp| (cp.power, cp.toughness));
                i64::from(c.definition.cost.cmc()) * 10_000 + i64::from(p.max(0)) * 100 + i64::from(t.max(0))
            })
        };
        let best_of = |seat: usize| {
            self.battlefield
                .iter()
                .filter(|c| c.controller == seat && self.computed_permanent(c.id).is_some_and(|cp| cp.card_types().contains(&crate::card::CardType::Creature)))
                .map(|c| c.id)
                .max_by_key(|&id| value(id))
        };
        let plan = |step: usize| -> Vec<(usize, CardId)> {
            alive
                .iter()
                .enumerate()
                .filter_map(|(i, &p)| best_of(alive[(i + step) % alive.len()]).map(|c| (p, c)))
                .collect()
        };
        let me = ctx.controller;
        let net = |moves: &[(usize, CardId)]| -> i64 {
            moves
                .iter()
                .map(|&(p, c)| {
                    let owner_now = self.battlefield_find(c).map(|x| x.controller);
                    if p == me {
                        value(c)
                    } else if owner_now == Some(me) {
                        -value(c)
                    } else {
                        0
                    }
                })
                .sum()
        };
        let left = plan(1);
        let right = plan(alive.len() - 1);
        let moves = if net(&right) > net(&left) { right } else { left };
        for (p, c) in moves {
            self.change_control(c, p);
        }
    }
}
