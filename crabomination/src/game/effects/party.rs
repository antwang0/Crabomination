//! CR 700.8 — parties: "up to one each of Cleric, Rogue, Warrior, and
//! Wizard", one creature per role (a Changeling fills any). Everyone's
//! Invited!'s Stick Together ("each player chooses a party from among
//! creatures they control, then sacrifices the rest") and Harper Recruiter
//! ("you may reveal a Cleric card, a Rogue card, a Warrior card, and/or a
//! Wizard card from among them").

use super::EffectContext;
use crate::card::{CardId, CardInstance, CreatureType as CT, Keyword};
use crate::decision::PickValue;
use crate::effect::{Effect, PlayerRef};
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
            .filter(|c| c.controller == seat && self.computed_is_creature(c))
            .filter_map(|c| self.computed_permanent(c.id).map(|cp| (c.id, cp)))
            .map(|(id, cp)| {
                let changeling = cp.keywords().has_kw(&Keyword::Changeling);
                let roles = std::array::from_fn(|i| changeling || cp.subtypes().creature_types.contains(&ROLES[i]));
                (id, cp.power, roles)
            })
            .collect()
    }

    /// CR 700.8 — the party `seat` chooses from `cands` (id, roles; in the
    /// headless preference order). A pick that is a party stands as made, up
    /// to one per role; any other is completed to a largest party, picks first.
    /// `None` is a suspend.
    #[allow(clippy::too_many_arguments)]
    fn ask_party(
        &mut self,
        cursor: &mut usize,
        seat: usize,
        prompt: &str,
        cands: &[(CardId, String, [bool; 4])],
        effect: &Effect,
        source: CardId,
    ) -> Option<Vec<CardId>> {
        let roles: Vec<[bool; 4]> = cands.iter().map(|c| c.2).collect();
        let auto: Vec<CardId> = largest_party(&roles).into_iter().map(|i| cands[i].0).collect();
        let picked = self.ask_seat_cards_logged(
            cursor,
            seat,
            prompt.into(),
            source,
            cands.iter().map(|c| (c.0, c.1.clone())).collect(),
            0,
            4,
            PickValue::Gain,
            effect,
            auto,
        )?;
        let picked_roles: Vec<[bool; 4]> =
            picked.iter().filter_map(|id| cands.iter().find(|c| c.0 == *id)).map(|c| c.2).collect();
        if largest_party(&picked_roles).len() == picked.len() {
            return Some(picked);
        }
        let mut order: Vec<usize> = (0..cands.len()).collect();
        order.sort_by_key(|&i| !picked.contains(&cands[i].0));
        let ordered: Vec<[bool; 4]> = order.iter().map(|&i| cands[i].2).collect();
        Some(largest_party(&ordered).into_iter().map(|k| cands[order[k]].0).collect())
    }

    /// `Effect::EachPlayerKeepsPartySacrificesRest` (Stick Together): in
    /// APNAP order each player chooses a party (CR 101.4; headless, the
    /// strongest creatures that make a largest one), then every other creature
    /// they control is sacrificed at once (CR 608.2c).
    pub(super) fn each_player_keeps_party_sacrifices_rest(
        &mut self,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
        effect: &Effect,
    ) -> Result<(), GameError> {
        let seats = self.apnap_sort(self.living_seats().collect());
        let source = ctx.source.unwrap_or(CardId(0));
        let mut cursor = 0;
        let mut doomed: Vec<(CardId, usize)> = Vec::new();
        for p in seats {
            let mut mine = self.creature_roles(p);
            mine.sort_by_key(|&(id, power, _)| (std::cmp::Reverse(power), id));
            let cands: Vec<(CardId, String, [bool; 4])> = mine
                .iter()
                .map(|&(id, _, r)| (id, self.battlefield_find(id).map_or(String::new(), |c| c.definition.name.to_string()), r))
                .collect();
            let Some(keep) = self.ask_party(&mut cursor, p, "Choose a party to keep", &cands, effect, source) else {
                return Ok(());
            };
            doomed.extend(mine.iter().filter(|m| !keep.contains(&m.0)).map(|m| (m.0, p)));
        }
        self.clear_answer_log();
        for (cid, who) in doomed {
            self.sacrifice_one(cid, who, events);
        }
        Ok(())
    }

    /// `Effect::LookTopTakeParty { who, count }` (Harper Recruiter): look at
    /// the top `count`, the looker may reveal a party of Cleric / Rogue /
    /// Warrior / Wizard cards among them and take it; the rest go on the
    /// bottom in a random order.
    pub(super) fn look_top_take_party(
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
        let cands: Vec<(CardId, String, [bool; 4])> = self.players[p].library[..n]
            .iter()
            .map(|c| (c.id, c.definition.name.to_string(), card_roles(c)))
            .filter(|c| c.2.iter().any(|&r| r))
            .collect();
        let mut cursor = 0;
        let source = ctx.source.unwrap_or(CardId(0));
        let Some(keep) = self.ask_party(&mut cursor, p, "Reveal a party to put into your hand", &cands, effect, source)
        else {
            return Ok(());
        };
        self.clear_answer_log();
        let top: Vec<CardInstance> = self.players[p].library.drain(..n).collect();
        let mut rest = Vec::new();
        for card in top {
            if keep.contains(&card.id) {
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
