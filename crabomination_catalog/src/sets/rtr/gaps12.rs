//! Return to Ravnica (RTR) gap wave 13: Slaughter Games (name-exile across all
//! zones) and Guild Feud (dueling top-three reveal). Tests in `classic_sets/rtr`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    ExileReturnZone, Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
    Value,
};
use crate::effect::{Effect, PlayerRef, Selector};
use crate::game::TurnStep;
use crate::mana::{b, cost, generic, hybrid, r, w, Color};

/// Slaughter Games — {2}{B}{R} Sorcery that can't be countered. Choose a
/// nonland card name; exile every card with that name from target opponent's
/// graveyard, hand, and library, then they shuffle.
pub fn slaughter_games() -> CardDefinition {
    CardDefinition {
        name: "Slaughter Games",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::NameCardExileMatchingAllZones,
        ..Default::default()
    }
}

/// Guild Feud — {5}{R} Enchantment. At your upkeep, target opponent reveals
/// three, may deploy a creature (rest to their graveyard); you do the same; if
/// two creatures enter, they fight.
pub fn guild_feud() -> CardDefinition {
    CardDefinition {
        name: "Guild Feud",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::GuildFeud,
        }],
        ..Default::default()
    }
}

/// Grave Betrayal — {5}{B}{B} Enchantment. Whenever a creature you don't control
/// dies, return it under your control at the next end step with an extra +1/+1
/// counter, as a black Zombie in addition to its other types.
pub fn grave_betrayal() -> CardDefinition {
    CardDefinition {
        name: "Grave Betrayal",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::GraveBetrayalRegister,
        }],
        ..Default::default()
    }
}

/// Angel of Serenity — {4}{W}{W}{W} 5/6 Angel. Flying; ETB exile up to three
/// target creatures until it leaves (returning them to their owners' hands on
/// leave). (The alternate "creature cards from graveyards" targets are omitted
/// — the battlefield-removal half is modeled.)
pub fn angel_of_serenity() -> CardDefinition {
    CardDefinition {
        name: "Angel of Serenity",
        cost: cost(&[generic(4), w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::Target(0),
                    return_to: ExileReturnZone::Hand,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Azor's Elocutors — {3}{W/U}{W/U} 3/5 Human Advisor. At your upkeep add a
/// filibuster counter, then win if it has five or more; a source dealing (combat)
/// damage to you removes one. (Noncombat damage doesn't remove — approximated.)
pub fn azors_elocutors() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Azor's Elocutors",
        cost: cost(&[generic(3), wu(), wu()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Filibuster,
                        amount: Value::ONE,
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Filibuster },
                            Value::Const(5),
                        ),
                        then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource),
                effect: Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Filibuster,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}
