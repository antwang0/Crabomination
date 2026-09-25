//! "When this dies, exile it and choose target creature an opponent
//! controls. When that creature leaves the battlefield, return this card from
//! exile to the battlefield under its owner's control" (Lucius the Eternal) —
//! a delayed leaves-the-battlefield trigger watching the chosen creature
//! (CR 603.7), whose body returns the source card only while it is still in
//! exile (CR 400.7: a card that left exile is a new object).

use super::{EffectContext, EntityRef};
use crate::card::SelectionRequirement;
use crate::effect::{Effect, PlayerRef, Predicate, Selector, ZoneDest};
use crate::game::GameState;
use crate::game::types::{DelayedKind, DelayedTrigger, GameError, Target};

impl GameState {
    /// `Effect::ReturnSourceWhenTargetLeaves` — `what` names the watched
    /// permanent; the source card is `ctx.source`.
    pub(super) fn return_source_when_target_leaves(
        &mut self,
        what: &Selector,
        ctx: &EffectContext,
    ) -> Result<(), GameError> {
        let Some(source) = ctx.source else { return Ok(()) };
        let Some(watched) = self.resolve_selector(what, ctx).into_iter().find_map(|e| match e {
            EntityRef::Permanent(id) => Some(id),
            _ => None,
        }) else {
            return Ok(());
        };
        self.delayed_triggers.push(DelayedTrigger {
            controller: ctx.controller,
            source,
            kind: DelayedKind::WhenCardLeavesBattlefield(watched),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: SelectionRequirement::InExile },
                then: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        tapped: false,
                    },
                }),
                else_: Box::new(Effect::Noop),
            },
            target: Some(Target::Permanent(source)),
            bound_token: None,
            bound_subject: None,
            fires_once: true,
            expires_after_turn: None,
        });
        Ok(())
    }
}
