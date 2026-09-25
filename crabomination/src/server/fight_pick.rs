//! An optional fight ("fights up to one target creature") is worth taking
//! only against a creature the fight kills: anything else just deals damage
//! back to the fighter. Apex Altisaur's enrage re-fights whenever it is dealt
//! damage, and an 8-seat pod fought an indestructible Zetalpa 7,538 times
//! (`--pod-decks 162,..,155 --seed 9403 --first 795`).

use crate::decision::DecisionAnswer;
use crate::effect::Effect;
use crate::game::GameState;
use crate::game::types::Target;

/// True when a triggered ability of `source` fights through an optional
/// target slot (`OptionalTargets { Fight }`).
pub(super) fn optional_fight_source(state: &GameState, source: crate::card::CardId) -> bool {
    fn fights(e: &Effect) -> bool {
        match e {
            Effect::OptionalTargets { body, .. } => matches!(**body, Effect::Fight { .. }),
            Effect::Seq(v) => v.iter().any(fights),
            _ => false,
        }
    }
    state.battlefield_find(source).is_some_and(|c| c.definition.triggered_abilities.iter().any(|t| fights(&t.effect)))
}

/// The most valuable legal creature the fighter's damage destroys, or a
/// decline when there is none.
pub(super) fn pick_killing_fight(state: &GameState, source: crate::card::CardId, legal: &[Target]) -> DecisionAnswer {
    let Some(me) = state.computed_permanent(source) else { return DecisionAnswer::DeclineTarget };
    let deathtouch = me.keywords().contains(&crate::card::Keyword::Deathtouch);
    let kill = |t: &Target| {
        let Target::Permanent(id) = t else { return None };
        let c = state.computed_permanent(*id)?;
        let damage = state.battlefield_find(*id).map_or(0, |b| b.damage as i32);
        let dies = !c.keywords().contains(&crate::card::Keyword::Indestructible)
            && me.power > 0
            && (deathtouch || c.toughness - damage <= me.power);
        dies.then_some((c.power + c.toughness, t.clone()))
    };
    legal
        .iter()
        .filter_map(kill)
        .max_by_key(|(v, _)| *v)
        .map_or(DecisionAnswer::DeclineTarget, |(_, t)| DecisionAnswer::Target(t))
}
