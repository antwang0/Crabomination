//! CR 702.167a — "Craft with a Dinosaur, a Merfolk, a Pirate, and a Vampire"
//! (`ActivatedAbility::craft_exile_each`): one distinct object per
//! requirement, from among permanents the activator controls and/or cards in
//! their graveyard. A creature card that is both a Pirate and a Vampire
//! fills one slot, so the picks are a matching, found by backtracking over
//! the (at most a handful of) requirements. Graveyard cards are tried first,
//! then the lowest-power permanents, so board pieces stay where they can.
//! `craft_cost_picks` is the plain "craft with N [filter]" sibling, shared by
//! the activation's pre-flight and the bot's craft sink.

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

impl GameState {
    /// "Craft with `n` that share a card type" (Eye of Ojer Taq): `n` objects
    /// with a card type in common, `(id, from_graveyard)`, or `None`. The
    /// type with the most graveyard candidates wins (ties: the printed type
    /// order), and within it graveyard cards come first, then the
    /// lowest-power permanents.
    pub(crate) fn craft_sharing_type_picks(&self, p: usize, source: CardId, n: u32) -> Option<Vec<(CardId, bool)>> {
        use crate::card::CardType as T;
        const TYPES: [T; 8] = [
            T::Artifact,
            T::Creature,
            T::Enchantment,
            T::Instant,
            T::Sorcery,
            T::Land,
            T::Planeswalker,
            T::Battle,
        ];
        let n = n as usize;
        let gy: Vec<(CardId, &[T])> = self.players[p]
            .graveyard
            .iter()
            .filter(|c| c.id != source)
            .map(|c| (c.id, c.definition.card_types.as_slice()))
            .collect();
        let mut board: Vec<(CardId, i32)> = self
            .battlefield
            .iter()
            .filter(|c| c.id != source && c.controller == p)
            .map(|c| (c.id, self.computed_permanent(c.id).map_or(0, |cp| cp.power)))
            .collect();
        board.sort_by_key(|&(id, power)| (power, id));
        let board_has = |id: CardId, t: &T| self.computed_permanent(id).is_some_and(|cp| cp.card_types().contains(t));
        TYPES
            .iter()
            .filter_map(|t| {
                let mut picks: Vec<(CardId, bool)> =
                    gy.iter().filter(|(_, ts)| ts.contains(t)).map(|(id, _)| (*id, true)).take(n).collect();
                let from_gy = picks.len();
                picks.extend(board.iter().filter(|(id, _)| board_has(*id, t)).map(|(id, _)| (*id, false)).take(n - from_gy));
                (picks.len() == n).then_some((from_gy, picks))
            })
            .max_by_key(|(from_gy, _)| *from_gy)
            .map(|(_, picks)| picks)
    }

    /// "Craft with `count` [filter]" (`ActivatedAbility::craft_exile_cost`):
    /// `count` *other* objects from among `p`'s permanents and graveyard,
    /// graveyard cards first, then the lowest-power permanents; `None` when
    /// too few match. `count` 0 is "one or more": every matching graveyard
    /// card (the most a P/T or colour CDA can read — Wretched Bonemass,
    /// Sunbird Effigy), else one permanent, a nonland one when there is any.
    pub(crate) fn craft_cost_picks(
        &self,
        p: usize,
        source: CardId,
        filter: &SelectionRequirement,
        count: u32,
    ) -> Option<Vec<(CardId, bool)>> {
        let gy: Vec<CardId> = self.players[p]
            .graveyard
            .iter()
            .filter(|c| c.id != source)
            .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
            .map(|c| c.id)
            .collect();
        let mut bf: Vec<CardId> = self
            .battlefield
            .iter()
            .filter(|c| c.id != source && c.controller == p)
            .filter(|c| self.evaluate_requirement_on_card(filter, c, p))
            .map(|c| c.id)
            .collect();
        let count = match count {
            0 if gy.is_empty() && bf.is_empty() => return None,
            0 if gy.is_empty() => {
                let nonland: Vec<CardId> = bf
                    .iter()
                    .copied()
                    .filter(|id| self.battlefield_find(*id).is_some_and(|c| !c.definition.is_land()))
                    .collect();
                if !nonland.is_empty() {
                    bf = nonland;
                }
                1
            }
            0 => gy.len(),
            n => n as usize,
        };
        if gy.len() + bf.len() < count {
            return None;
        }
        let mut picks: Vec<(CardId, bool)> = gy.into_iter().take(count).map(|id| (id, true)).collect();
        if picks.len() < count {
            let need = count - picks.len();
            picks.extend(self.auto_pick_lowest_power(&bf, need).into_iter().map(|id| (id, false)));
        }
        Some(picks)
    }
}
