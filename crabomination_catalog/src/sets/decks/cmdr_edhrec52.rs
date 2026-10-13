//! Commander: three-card gaps for Mister Fantastic (`cmdr_edhrec41` is the
//! previous batch of this run). Tests in `tests/recent_b/cmdr_edhrec52.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Effect, PlayerRef};
use crate::mana::{cost, g, generic, r, w, ManaCost};
use std::sync::Arc;

fn hero(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Hero], ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn your_hero() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Hero)).and(R::ControlledByYou)
}

/// Human Torch, Johnny Storm — {2}{R} legendary 2/2 Human Hero, flying.
/// Whenever you draw a card, if you control another Hero, 1 damage to target
/// opponent. Power-up — {6}{R}: three +1/+1 counters on it (CR 702.193).
pub fn human_torch_johnny_storm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(Predicate::ValueAtLeast(
                Value::count(Selector::EachPermanent(your_hero().and(R::OtherThanSource))),
                Value::ONE,
            )),
            effect: Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6), r()]),
            power_up: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..hero("Human Torch, Johnny Storm", cost(&[generic(2), r()]), 2, 2)
    }
}

/// Invisible Woman, Sue Storm — {4}{W} legendary 2/5 Human Hero, lifelink.
/// Whenever you put one or more +1/+1 counters on one or more other Heroes
/// you control, you may create a 0/4 colorless Wall with defender.
pub fn invisible_woman_sue_storm() -> CardDefinition {
    let wall = TokenDefinition {
        name: "Wall".into(),
        power: 0,
        toughness: 4,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        keywords: vec![Keyword::Defender],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: your_hero().and(R::OtherThanSource),
                })
                .once_per_batch(),
            effect: Effect::MayDo {
                description: "Create a 0/4 Wall with defender?".into(),
                body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(wall) }),
            },
        }],
        ..hero("Invisible Woman, Sue Storm", cost(&[generic(4), w()]), 2, 5)
    }
}

/// The Thing, Ben Grimm — {5}{G} legendary 7/7 Human Hero, trample. Whenever
/// one or more Heroes you control deal damage to a player, two +1/+1
/// counters on it.
pub fn the_thing_ben_grimm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: your_hero() })
                .once_per_batch(),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
        }],
        ..hero("The Thing, Ben Grimm", cost(&[generic(5), g()]), 7, 7)
    }
}
