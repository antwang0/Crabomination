//! CR 702.85a — does a spell just cast have cascade? Printed, or given by a
//! "when you cast" cascade trigger whose condition holds for it: the catalog
//! writes "[the first spell …] has cascade" (Wild-Magic Sorcerer, Maelstrom
//! Nexus, Bloodbraid Marauder) as such a trigger, so the keyword never lands
//! on the spell itself.

use super::effects::{EffectContext, EntityRef};
use super::{GameState, StackItem};
use crate::card::{CardId, Keyword, TriggeredAbility};
use crate::effect::{Effect, EventKind, EventScope};

impl GameState {
    /// `Predicate::CastSpellHasCascade` — the `SpellCast` trigger's spell.
    pub(crate) fn cast_spell_has_cascade(&self, ctx: &EffectContext) -> bool {
        let Some(EntityRef::Card(cid)) = ctx.trigger_source else { return false };
        let Some((card, caster)) = self.stack.iter().find_map(|si| match si {
            StackItem::Spell { card, caster, .. } if card.id == cid => Some((card, *caster)),
            _ => None,
        }) else {
            return false;
        };
        if card.definition.keywords.contains(&Keyword::Cascade) {
            return true;
        }
        let grants = |t: &TriggeredAbility, scope: EventScope, source: CardId| {
            t.event.kind == EventKind::SpellCast
                && t.event.scope == scope
                && matches!(t.effect, Effect::Cascade { .. })
                && t.event.filter.as_ref().is_none_or(|f| {
                    let mut c = EffectContext::for_trigger(source, caster, None, 0);
                    c.trigger_source = Some(EntityRef::Card(cid));
                    self.evaluate_predicate(f, &c)
                })
        };
        card.definition.triggered_abilities.iter().any(|t| grants(t, EventScope::SelfSource, cid))
            || self.battlefield.iter().filter(|p| p.controller == caster).any(|p| {
                p.definition.triggered_abilities.iter().any(|t| grants(t, EventScope::YourControl, p.id))
            })
    }
}
