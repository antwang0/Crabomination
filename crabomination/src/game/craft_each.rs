//! CR 702.167a — "Craft with a Dinosaur, a Merfolk, a Pirate, and a Vampire"
//! (`ActivatedAbility::craft_exile_each`): one distinct object per
//! requirement, from among permanents the activator controls and/or cards in
//! their graveyard. A creature card that is both a Pirate and a Vampire
//! fills one slot, so the picks are a matching, found by backtracking over
//! the (at most a handful of) requirements. Graveyard cards are tried first,
//! then the lowest-power permanents, so board pieces stay where they can.

use crate::card::{CardId, SelectionRequirement};
use crate::game::GameState;

impl GameState {
    /// One distinct pick per requirement, `(id, from_graveyard)`, or `None`
    /// when no assignment covers them all. `source` is never picked.
    pub(crate) fn craft_each_picks(
        &self,
        p: usize,
        source: CardId,
        each: &[SelectionRequirement],
    ) -> Option<Vec<(CardId, bool)>> {
        let mut pool: Vec<(CardId, bool, i32)> = self.players[p]
            .graveyard
            .iter()
            .filter(|c| c.id != source)
            .map(|c| (c.id, true, i32::MIN))
            .collect();
        let mut board: Vec<(CardId, bool, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.id != source && c.controller == p)
            .map(|c| (c.id, false, self.computed_permanent(c.id).map_or(0, |cp| cp.power)))
            .collect();
        board.sort_by_key(|&(id, _, power)| (power, id));
        pool.extend(board);
        let fits: Vec<Vec<usize>> = each
            .iter()
            .map(|req| {
                (0..pool.len())
                    .filter(|&i| {
                        let (id, in_gy, _) = pool[i];
                        let card = if in_gy {
                            self.players[p].graveyard.iter().find(|c| c.id == id)
                        } else {
                            self.battlefield.iter().find(|c| c.id == id)
                        };
                        card.is_some_and(|c| self.evaluate_requirement_on_card(req, c, p))
                    })
                    .collect()
            })
            .collect();
        let mut chosen: Vec<usize> = Vec::with_capacity(each.len());
        fn assign(fits: &[Vec<usize>], chosen: &mut Vec<usize>) -> bool {
            let Some(options) = fits.get(chosen.len()) else { return true };
            for &i in options {
                if !chosen.contains(&i) {
                    chosen.push(i);
                    if assign(fits, chosen) {
                        return true;
                    }
                    chosen.pop();
                }
            }
            false
        }
        assign(&fits, &mut chosen).then(|| chosen.iter().map(|&i| (pool[i].0, pool[i].1)).collect())
    }
}
