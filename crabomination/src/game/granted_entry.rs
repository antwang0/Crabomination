//! CR 614.12 — a permanent's "enters with" replacements are read off the
//! characteristics it will have on the battlefield, which includes what other
//! permanents' static abilities grant it: Twins of Discord's "each other
//! colorless creature you control has bloodthirst 2" applies to one entering
//! from anywhere.

use super::GameState;
use super::types::Target;
use crate::card::{CardId, Keyword};
use crate::effect::{Selector, StaticEffect};

impl GameState {
    /// The bloodthirst other permanents' `GrantKeyword` statics give `cid`.
    pub(crate) fn granted_bloodthirst(&self, cid: CardId) -> u32 {
        let mut n = 0;
        for src in self.battlefield.iter().filter(|c| c.id != cid) {
            for sa in &src.definition.static_abilities {
                if let StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(filter), keyword: Keyword::Bloodthirst(k) } =
                    &sa.effect
                    && self.evaluate_requirement_static(filter, &Target::Permanent(cid), src.controller, Some(src.id))
                {
                    n += *k;
                }
            }
        }
        n
    }
}
