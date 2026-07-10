//! MH3-adjacent staples reusing existing primitives: kicker ETB counters
//! (Untamed Kavu) and mana-spent token minting (Manaform Hellkite —
//! `Value::CastSpellManaSpent` + a mint-time `TokenDefinition.dynamic_pt` +
//! self-exile at the next end step). Tests in `tests/recent116.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::etb;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::TurnStep;
use crate::mana::{cost, g, generic, r, Color};

/// Untamed Kavu — {1}{G} 2/2 Kavu with vigilance and trample. Kicker {3}; if
/// kicked, it enters with three +1/+1 counters.
pub fn untamed_kavu() -> CardDefinition {
    CardDefinition {
        name: "Untamed Kavu",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kavu], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Kicker(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Manaform Hellkite — {2}{R}{R} 4/4 Dragon with flying. Whenever you cast a
/// noncreature spell, create an X/X red Dragon Illusion with flying and haste
/// (X = mana spent to cast that spell), then exile it at the next end step.
pub fn manaform_hellkite() -> CardDefinition {
    let illusion = TokenDefinition {
        name: "Dragon Illusion".into(),
        power: 0,
        toughness: 0,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Illusion],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying, Keyword::Haste],
        dynamic_pt: Some((Value::CastSpellManaSpent, Value::CastSpellManaSpent)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ExileSource,
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Manaform Hellkite",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: illusion,
            },
        }],
        ..Default::default()
    }
}
