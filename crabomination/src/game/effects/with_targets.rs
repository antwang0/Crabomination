//! `Effect::WithTargets` — a body run with chosen objects bound as its
//! targets (a resolution-time choice is not a target, but a body that reads
//! "the chosen land" reads it where targets live).

use crate::effect::{Effect, Selector};
use crate::game::GameState;
use crate::game::effects::{EffectContext, EntityRef, rewrap_parked};
use crate::game::types::{GameError, GameEvent, Target};

impl GameState {
    pub(crate) fn run_with_targets(
        &mut self,
        what: &Selector,
        body: &Effect,
        ctx: &EffectContext,
        events: &mut Vec<GameEvent>,
    ) -> Result<(), GameError> {
        let ents = self.resolve_selector(what, ctx);
        let targets: Vec<Target> = ents
            .iter()
            .map(|e| match *e {
                EntityRef::Player(p) => Target::Player(p),
                EntityRef::Permanent(id) | EntityRef::Card(id) => Target::Permanent(id),
            })
            .collect();
        let mut sub = ctx.clone();
        sub.targets = targets.to_vec();
        self.run_effect(body, &sub, events)?;
        // A parked continuation resumes under the stack item's context: keep
        // the binding by naming the same objects again.
        let ids: Vec<crate::card::CardId> = targets
            .iter()
            .filter_map(|t| match t {
                Target::Permanent(id) => Some(*id),
                Target::Player(_) => None,
            })
            .collect();
        rewrap_parked(&mut self.suspend_signal, |carried| Effect::WithTargets {
            what: Selector::ExactObjects(ids),
            body: Box::new(carried),
        });
        Ok(())
    }
}
