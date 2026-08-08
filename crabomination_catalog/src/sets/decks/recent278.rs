//! DMU/SNC/NEO gap batch — kicker payoffs, an attack-untapper, an alliance
//! grower, two library-dig spells, and an exile-for-Treasure. All on existing
//! primitives. Tests in `tests/recent_b/recent278.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::card::{EventKind, EventScope, EventSpec, Predicate};
use crate::effect::shortcut::{etb, on_you_attack, target_filtered};
use crate::effect::{LookPick, Duration, Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Bog Badger — {2}{G} 3/3 Badger, kicker {B}. When it enters, if it was
/// kicked, creatures you control gain menace until end of turn.
pub fn bog_badger() -> CardDefinition {
    CardDefinition {
        name: "Bog Badger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Kicker(cost(&[b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Colossal Growth — {1}{G} Instant, kicker {R}. Target creature gets +3/+3;
/// if kicked, instead +4/+4 with trample and haste, until end of turn.
pub fn colossal_growth() -> CardDefinition {
    CardDefinition {
        name: "Colossal Growth",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[r()]))],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeywords {
                        what: Selector::Target(0),
                        keywords: vec![Keyword::Trample, Keyword::Haste],
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Civic Gardener — {1}{G} 2/2 Human Citizen. Whenever it attacks, untap target
/// creature or land.
pub fn civic_gardener() -> CardDefinition {
    CardDefinition {
        name: "Civic Gardener",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![on_you_attack(Effect::Untap {
            what: target_filtered(R::Creature.or(R::Land)),
            up_to: None,
        })],
        ..Default::default()
    }
}

/// Celebrity Fencer — {3}{W} 3/2 Elf Druid. Alliance — whenever another creature
/// you control enters, put a +1/+1 counter on it.
pub fn celebrity_fencer() -> CardDefinition {
    CardDefinition {
        name: "Celebrity Fencer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Commune with Spirits — {G} Sorcery. Look at the top four cards; reveal an
/// enchantment or land from among them to your hand, the rest to the bottom in
/// a random order.
pub fn commune_with_spirits() -> CardDefinition {
    CardDefinition {
        name: "Commune with Spirits",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::Enchantment.or(R::Land)),
            take: Some(Value::ONE),
            optional: true,
            rest_bottom_random: true,
    ..Default::default()
})),
        ..Default::default()
    }
}

/// Case the Joint — {3}{U} Instant. Draw two cards, then look at the top card of
/// each player's library.
pub fn case_the_joint() -> CardDefinition {
    CardDefinition {
        name: "Case the Joint",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::LookAtTop {
                who: PlayerRef::EachPlayer,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Buy Your Silence — {4}{W} Sorcery. Exile target nonland permanent. Its
/// controller creates a Treasure token.
pub fn buy_your_silence() -> CardDefinition {
    CardDefinition {
        name: "Buy Your Silence",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::ONE,
                definition: Box::new(crabomination_base::tokens::treasure_token()),
            },
            Effect::Move {
                what: target_filtered(R::Nonland),
                to: ZoneDest::Exile,
            },
        ]),
        ..Default::default()
    }
}
