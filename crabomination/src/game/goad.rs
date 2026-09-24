//! CR 701.15 — who a creature is goaded by. Four sources: a resolved goad
//! (`goaded_by`, until the goader's next turn), a held goad (`goad_holds`,
//! "as long as", CR 611.2b), an attached Aura or Equipment that says the
//! creature "is goaded" (`StaticEffect::AttachedIsGoaded`), and a board-wide
//! static (Baeloth Barrityl, Mocking Doppelganger). Every reader of goad asks
//! here so the four can't disagree.

use smallvec::SmallVec;

use super::GameState;
use crate::card::{CardInstance, GoadHold};
use crate::effect::StaticEffect;

impl GameState {
    /// The players goading `c`, deduplicated, in first-seen order.
    pub fn goaders(&self, c: &CardInstance) -> SmallVec<[usize; 4]> {
        let mut out: SmallVec<[usize; 4]> = SmallVec::new();
        let mut add = |p: usize| {
            if !out.contains(&p) {
                out.push(p);
            }
        };
        if !c.cold_pristine() {
            for &p in &c.goaded_by {
                add(p);
            }
            for &(p, hold) in &c.goad_holds {
                if self.goad_hold_active(c, hold) {
                    add(p);
                }
            }
        }
        for a in self.battlefield.iter() {
            if a.attached_to == Some(c.id) && self.attaches_goad(a) {
                add(a.controller);
            }
            if !a.definition.static_abilities.is_empty() && self.board_goads(a, c) {
                add(a.controller);
            }
        }
        out
    }

    /// Does `a` carry a board-wide goad static that reaches `c`? Baeloth's
    /// lesser-power clause and Mocking Doppelganger's same-name rider.
    fn board_goads(&self, a: &CardInstance, c: &CardInstance) -> bool {
        a.definition.static_abilities.iter().any(|sa| match self.active_static(&sa.effect, a) {
            Some(StaticEffect::OpponentCreaturesWithLesserPowerAreGoaded) => {
                c.definition.is_creature()
                    && !self.same_team(a.controller, c.controller)
                    && c.power() < a.power()
            }
            Some(StaticEffect::OthersNamedLikeThisAreGoaded) => {
                c.id != a.id && c.definition.is_creature() && c.definition.name == a.definition.name
            }
            _ => false,
        })
    }

    /// Any board-wide goad static on the battlefield (the presence half of
    /// `board_goads`).
    fn board_goad_present(&self, a: &CardInstance) -> bool {
        a.definition.static_abilities.iter().any(|sa| {
            matches!(
                sa.effect,
                StaticEffect::OpponentCreaturesWithLesserPowerAreGoaded
                    | StaticEffect::OthersNamedLikeThisAreGoaded
            )
        })
    }

    /// Is `c` goaded by anyone?
    pub fn is_goaded(&self, c: &CardInstance) -> bool {
        !self.goaders(c).is_empty()
    }

    /// Is `c` goaded by player `p`?
    pub fn goaded_by_player(&self, c: &CardInstance, p: usize) -> bool {
        self.goaders(c).contains(&p)
    }

    /// Could any creature on the battlefield be goaded? The cheap board gate
    /// the attack-requirement passes open on.
    pub fn any_goad_present(&self) -> bool {
        self.battlefield.iter().any(|c| {
            c.cold_any(|k| !k.goaded_by.is_empty() || !k.goad_holds.is_empty())
                || (c.attached_to.is_some() && self.attaches_goad(c))
                || self.board_goad_present(c)
        })
    }

    /// Immortal Obligation — while its duty counter holds, `c` can't attack
    /// `p` or a permanent `p` controls, nor block a creature `p` controls.
    pub(crate) fn obligated_to(&self, c: &CardInstance, p: usize) -> bool {
        c.cold_any(|k| {
            k.goad_holds
                .iter()
                .any(|&(q, h)| q == p && matches!(h, GoadHold::Obligation(kind) if c.counter_count(kind) > 0))
        })
    }

    fn attaches_goad(&self, a: &CardInstance) -> bool {
        a.definition
            .static_abilities
            .iter()
            .any(|sa| matches!(self.active_static(&sa.effect, a), Some(StaticEffect::AttachedIsGoaded)))
    }

    fn goad_hold_active(&self, c: &CardInstance, hold: GoadHold) -> bool {
        match hold {
            GoadHold::WhileOnBattlefield(src) => self.battlefield_find(src).is_some(),
            GoadHold::Obligation(kind) => c.counter_count(kind) > 0,
        }
    }
}
