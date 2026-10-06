//! CR 508.1d — the requirements that name *which* defender a creature
//! attacks: a goad's "a player other than [the goader]" (one per goader, CR
//! 701.15b-c), "attacks that player" (`MustAttackChosenPlayer`), and a lure's
//! "attacks [this planeswalker]" (Gideon Jura, Gideon Battle-Forged). They
//! are scored together, so an attack obeys the most of them it can: two
//! requirements naming different defenders leave a tie, and either is legal.

use smallvec::SmallVec;

use super::GameState;
use super::combat::attack_static_scan;
use super::types::{Attack, AttackTarget};
use crate::card::{CardId, Keyword, KeywordSlice};

impl GameState {
    /// How many of `id`'s defender-naming requirements an attack on `target`
    /// obeys. `goaders` is [`Self::goaders`] of `id`, taken once by the caller.
    fn attack_target_score(&self, p: usize, id: CardId, goaders: &[usize], target: AttackTarget) -> u32 {
        let mut score = 0;
        if let AttackTarget::Player(q) = target {
            score += goaders.iter().filter(|&&g| g != q).count() as u32;
            if self.chosen_attack_player(p, id) == Some(q) {
                score += 1;
            }
            score += self.attack_player_requirements.iter().filter(|&&(c, r)| c == id && r == q).count() as u32;
        }
        if let AttackTarget::Planeswalker(pw) = target {
            score += u32::from(self.attack_lure_of(p) == Some(pw));
            score += u32::from(self.creature_lure_of(p, id) == Some(pw));
        }
        score
    }

    /// The live opponent a `MustAttackChosenPlayer` creature is bound to
    /// attack (Raving Dead, an encore token, Ruhan).
    pub(crate) fn chosen_attack_player(&self, p: usize, id: CardId) -> Option<usize> {
        let q = self.battlefield_find(id)?.chosen_player?;
        (self.players.get(q).is_some_and(|pl| pl.is_alive())
            && !self.same_team(p, q)
            && self.computed_permanent(id).is_some_and(|cp| cp.keywords().has_kw(&Keyword::MustAttackChosenPlayer)))
        .then_some(q)
    }

    /// CR 508.1d — `id` must attack a live opponent this combat ("attacks
    /// that player this combat if able", Ruhan of the Fomori).
    pub(crate) fn must_attack_a_player_this_combat(&self, p: usize, id: CardId) -> bool {
        self.attack_player_requirements.iter().any(|&(c, q)| {
            c == id && q != p && self.players.get(q).is_some_and(|pl| pl.is_alive()) && !self.same_team(p, q)
        })
    }

    /// Whether `id` carries any defender-naming requirement this combat.
    fn has_attack_target_requirement(&self, p: usize, id: CardId, goaders: &[usize]) -> bool {
        !goaders.is_empty()
            || self.must_attack_a_player_this_combat(p, id)
            || self.chosen_attack_player(p, id).is_some()
            || self.attack_lure_of(p).is_some()
            || self.creature_lure_of(p, id).is_some()
    }

    /// Every defender `id` can attack without breaking a restriction or
    /// paying a cost (CR 508.1c/d — neither counts toward a requirement),
    /// each with its score. Players in turn order from `p`, then the lured
    /// planeswalkers. Empty when `id` has no defender-naming requirement.
    pub(crate) fn attack_target_options(&self, p: usize, id: CardId) -> SmallVec<[(AttackTarget, u32); 8]> {
        let goaders = self.battlefield_find(id).map(|c| self.goaders(c)).unwrap_or_default();
        self.attack_target_options_with(p, id, &goaders)
    }

    fn attack_target_options_with(
        &self,
        p: usize,
        id: CardId,
        goaders: &[usize],
    ) -> SmallVec<[(AttackTarget, u32); 8]> {
        let mut out: SmallVec<[(AttackTarget, u32); 8]> = SmallVec::new();
        if self.battlefield_find(id).is_none() || !self.has_attack_target_requirement(p, id, goaders) {
            return out;
        }
        let kws: Vec<Keyword> =
            self.computed_permanent(id).map(|cp| cp.keywords().to_vec()).unwrap_or_default();
        let statics = attack_static_scan(self);
        let taxed = self.attack_tax_possible(statics);
        let free = |target: AttackTarget, d: usize| {
            self.attacker_target_block(p, id, &kws, Some(d)).is_none()
                && (!taxed || {
                    let a = Attack { attacker: id, target };
                    self.attack_tax_for(std::slice::from_ref(&a), statics, |_| {
                        self.attack_block_keyword_tax(id, &kws, true)
                    }) == 0
                })
        };
        for q in self.seats_in_turn_order_from(p) {
            let target = AttackTarget::Player(q);
            if q != p && !self.same_team(p, q) && self.players[q].is_alive() && free(target, q) {
                out.push((target, self.attack_target_score(p, id, goaders, target)));
            }
        }
        for pw in [self.attack_lure_of(p), self.creature_lure_of(p, id)].into_iter().flatten() {
            let target = AttackTarget::Planeswalker(pw);
            if let Some(d) = self.battlefield_find(pw).map(|w| w.controller)
                && !out.iter().any(|(t, _)| *t == target)
                && free(target, d)
            {
                out.push((target, self.attack_target_score(p, id, goaders, target)));
            }
        }
        out
    }

    /// CR 508.1d — an attack on `target` obeys as many of `id`'s
    /// defender-naming requirements as any other defender it could attack.
    pub(crate) fn attack_target_obeys_most(&self, p: usize, id: CardId, target: AttackTarget) -> bool {
        let goaders = self.battlefield_find(id).map(|c| self.goaders(c)).unwrap_or_default();
        let options = self.attack_target_options_with(p, id, &goaders);
        let Some(best) = options.iter().map(|&(_, s)| s).max() else { return true };
        self.attack_target_score(p, id, &goaders, target) >= best
    }
}
