//! CR 700.8 — parties: "up to one each of Cleric, Rogue, Warrior, and
//! Wizard", one creature per role (a Changeling fills any). Everyone's
//! Invited!'s Stick Together ("each player chooses a party from among
//! creatures they control, then sacrifices the rest") and Harper Recruiter
//! ("you may reveal a Cleric card, a Rogue card, a Warrior card, and/or a
//! Wizard card from among them").

use super::EffectContext;
use crate::card::{CardId, CardInstance, CreatureType as CT, Keyword};
use crate::effect::PlayerRef;
use crate::game::{GameState, KeywordSlice};
use crate::game::types::{GameError, GameEvent};

const ROLES: [CT; 4] = [CT::Cleric, CT::Rogue, CT::Warrior, CT::Wizard];

/// A largest party from `roles` (one `[bool; 4]` per candidate, in the
/// order candidates should be preferred): the indices of the chosen ones.
/// Kuhn's augmenting paths, so a Cleric Wizard fills whichever slot leaves
/// room for the rest; earlier candidates win ties.
pub(crate) fn largest_party(roles: &[[bool; 4]]) -> Vec<usize> {
    fn augment(role: usize, roles: &[[bool; 4]], seen: &mut [bool], to_role: &mut [Option<usize>]) -> bool {
        for (ci, has) in roles.iter().enumerate() {
            if has[role] && !seen[ci] {
                seen[ci] = true;
                if to_role[ci].is_none() || augment(to_role[ci].unwrap(), roles, seen, to_role) {
                    to_role[ci] = Some(role);
                    return true;
                }
            }
        }
        false
    }
    let mut to_role: Vec<Option<usize>> = vec![None; roles.len()];
    for role in 0..4 {
        let mut seen = vec![false; roles.len()];
        augment(role, roles, &mut seen, &mut to_role);
    }
    (0..roles.len()).filter(|&i| to_role[i].is_some()).collect()
}

/// A card's printed roles (off the battlefield — Harper Recruiter's library).
fn card_roles(c: &CardInstance) -> [bool; 4] {
    let changeling = c.definition.keywords.has_kw(&Keyword::Changeling);
    std::array::from_fn(|i| changeling || c.definition.subtypes.creature_types.contains(&ROLES[i]))
}

impl GameState {
    /// `seat`'s creatures, each with the roles its computed type line fills.
    fn creature_roles(&self, seat: usize) -> Vec<(CardId, i32, [bool; 4])> {
        self.battlefield
            .iter()
            .filter(|c| c.controller == seat && c.definition.is_creature())
            .filter_map(|c| self.computed_permanent(c.id).map(|cp| (c.id, cp)))
            .map(|(id, cp)| {
                let changeling = cp.keywords().has_kw(&Keyword::Changeling);
                let roles = std::array::from_fn(|i| changeling || cp.subtypes().creature_types.contains(&ROLES[i]));
                (id, cp.power, roles)
            })
            .collect()
    }

    /// `Effect::EachPlayerKeepsPartySacrificesRest` (Stick Together): in
    /// APNAP order each player keeps a largest party — the strongest
    /// creatures that make one — and every other creature they control is
    /// sacrificed at once.
    pub(super) fn each_player_keeps_party_sacrifices_rest(
        &mut self,
        _ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let seats = self.apnap_sort((0..self.players.len()).collect());
        let mut doomed: Vec<(CardId, usize)> = Vec::new();
        for p in seats {
            let mut mine = self.creature_roles(p);
            mine.sort_by_key(|&(id, power, _)| (std::cmp::Reverse(power), id));
            let roles: Vec<[bool; 4]> = mine.iter().map(|m| m.2).collect();
            let keep = largest_party(&roles);
            doomed.extend(mine.iter().enumerate().filter(|(i, _)| !keep.contains(i)).map(|(_, m)| (m.0, p)));
        }
        for (cid, who) in doomed {
            self.sacrifice_one(cid, who, events);
        }
        Ok(())
    }

    /// `Effect::LookTopTakeParty { who, count }` (Harper Recruiter): look at
    /// the top `count`, put a largest party of Cleric / Rogue / Warrior /
    /// Wizard cards into hand, the rest on the bottom in a random order.
    pub(super) fn look_top_take_party(
        &mut self,
        who: &PlayerRef,
        count: &crate::effect::Value,
        ctx: &EffectContext,
        _events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        use rand::seq::SliceRandom;
        let Some(p) = self.resolve_player(who, ctx) else { return Ok(()) };
        let n = (self.evaluate_value(count, ctx).max(0) as usize).min(self.players[p].library.len());
        if n == 0 {
            return Ok(());
        }
        let top: Vec<CardInstance> = self.players[p].library.drain(..n).collect();
        let roles: Vec<[bool; 4]> = top.iter().map(card_roles).collect();
        let keep = largest_party(&roles);
        let mut rest = Vec::new();
        for (i, card) in top.into_iter().enumerate() {
            if keep.contains(&i) {
                self.players[p].hand.push(card);
            } else {
                rest.push(card);
            }
        }
        rest.shuffle(&mut self.rng.draw());
        self.players[p].library.extend(rest);
        Ok(())
    }
}
