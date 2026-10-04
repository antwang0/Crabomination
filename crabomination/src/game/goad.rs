//! CR 701.15 — who a creature is goaded by. Four sources: a resolved goad
//! (`goaded_by`, until the goader's next turn), a held goad (`goad_holds`,
//! "as long as", CR 611.2b), an attached Aura or Equipment that says the
//! creature "is goaded" (`StaticEffect::AttachedIsGoaded`), and a board-wide
//! static (Baeloth Barrityl, Mocking Doppelganger). Every reader of goad asks
//! here so the four can't disagree.

use smallvec::SmallVec;

use super::GameState;
use super::types::GameEvent;
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
                // CR 613.4c — the powers compared are the current ones,
                // anthems included (the layer pass reads printed here). A
                // snapshot off the battlefield (a dying goaded creature's
                // last-known information) keeps its own power.
                let power = |x: &CardInstance| {
                    if self.battlefield.find_by_id(x.id).is_some() { self.effective_power_on(x) } else { x.power() }
                };
                self.computed_is_creature(c)
                    && !self.same_team(a.controller, c.controller)
                    && power(c) < power(a)
            }
            Some(StaticEffect::OthersNamedLikeThisAreGoaded) => {
                c.id != a.id && self.computed_is_creature(c) && c.definition.name == a.definition.name
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
            GoadHold::WhileControlledBy(seat) => c.controller == usize::from(seat),
        }
    }

    /// CR 611.2b — "for as long as they control it" ends for good once
    /// control moves: drop the hold, so getting the creature back doesn't
    /// re-arm it (Vislor Turlough).
    pub(crate) fn release_lapsed_control_goads(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::ControlChanged { card_id, .. } = ev else { continue };
            let lapsed = |c: &CardInstance| {
                c.cold_any(|k| {
                    k.goad_holds.iter().any(|&(_, h)| matches!(h, GoadHold::WhileControlledBy(s) if usize::from(s) != c.controller))
                })
            };
            if !self.battlefield_find(*card_id).is_some_and(lapsed) {
                continue;
            }
            if let Some(c) = self.battlefield_find_mut(*card_id) {
                let now = c.controller;
                c.goad_holds.retain(|&(_, h)| !matches!(h, GoadHold::WhileControlledBy(s) if usize::from(s) != now));
            }
        }
    }

    /// CR 611.2b — an obligation ends for good when its last counter comes
    /// off: drop the hold, so a counter put back later doesn't re-arm it
    /// (Immortal Obligation's duty counter).
    pub(crate) fn release_spent_obligations(&mut self, events: &[GameEvent]) {
        for ev in events {
            let GameEvent::CounterRemoved { card_id, counter_type, .. } = ev else { continue };
            let spent = |c: &CardInstance| {
                c.counter_count(*counter_type) == 0
                    && c.cold_any(|k| k.goad_holds.iter().any(|&(_, h)| h == GoadHold::Obligation(*counter_type)))
            };
            if !self.battlefield_find(*card_id).is_some_and(spent) {
                continue;
            }
            if let Some(c) = self.battlefield_find_mut(*card_id) {
                c.goad_holds.retain(|&(_, h)| h != GoadHold::Obligation(*counter_type));
            }
        }
    }
}
