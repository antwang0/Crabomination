//! A Foundations (FDN) batch of commons/uncommons reusing existing primitives:
//! threshold + landfall triggers, a once-per-turn first-lifegain payoff, a
//! graveyard-return ETB, conditional attack pumps, a draw-matters counter, and
//! a kicker combat trick. Tests in `tests/recent120.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Crypt Feaster — {3}{B} 3/4 Zombie with menace. Threshold — whenever it
/// attacks, if seven or more cards are in your graveyard, it gets +2/+0.
pub fn crypt_feaster() -> CardDefinition {
    CardDefinition {
        name: "Crypt Feaster",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::ThresholdActive { who: PlayerRef::You }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// A 1/1 green Elf Warrior — Elfsworn Giant's landfall token.
fn elf_warrior_token() -> TokenDefinition {
    TokenDefinition {
        name: "Elf Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Elfsworn Giant — {3}{G}{G} 5/3 Giant with reach. Landfall — whenever a land
/// you control enters, create a 1/1 green Elf Warrior.
pub fn elfsworn_giant() -> CardDefinition {
    CardDefinition {
        name: "Elfsworn Giant",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: elf_warrior_token(),
            },
        }],
        ..Default::default()
    }
}

/// Elvish Regrower — {2}{G}{G} 4/3 Elf Druid. ETB: return target permanent card
/// from your graveyard to your hand.
pub fn elvish_regrower() -> CardDefinition {
    CardDefinition {
        name: "Elvish Regrower",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::PermanentCard),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Courageous Goblin — {1}{R} 2/2 Goblin. Whenever it attacks while you control
/// a creature with power 4 or greater, it gets +1/+0 and gains menace.
pub fn courageous_goblin() -> CardDefinition {
    CardDefinition {
        name: "Courageous Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Eager Trufflesnout — {2}{G} 4/2 Boar with trample. Whenever it deals combat
/// damage to a player, create a Food token.
pub fn eager_trufflesnout() -> CardDefinition {
    CardDefinition {
        name: "Eager Trufflesnout",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::food_token(),
            },
        }],
        ..Default::default()
    }
}

/// A 1/1 white Cat — Cat Collector's lifegain payoff token.
fn cat_token() -> TokenDefinition {
    TokenDefinition {
        name: "Cat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        ..Default::default()
    }
}

/// Cat Collector — {2}{W} 3/2 Human Citizen. ETB: create a Food. The first time
/// you gain life during each of your turns, create a 1/1 white Cat.
pub fn cat_collector() -> CardDefinition {
    CardDefinition {
        name: "Cat Collector",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crabomination_base::tokens::food_token(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You))
                    .once_per_turn(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: cat_token(),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dawnwing Marshal — {1}{W} 2/2 Cat Soldier with flying. {4}{W}: creatures you
/// control get +1/+1 until end of turn.
pub fn dawnwing_marshal() -> CardDefinition {
    CardDefinition {
        name: "Dawnwing Marshal",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Clinquant Skymage — {3}{U} 1/1 Bird Wizard with flying. Whenever you draw a
/// card, put a +1/+1 counter on it.
pub fn clinquant_skymage() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Clinquant Skymage",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Elementalist Adept — {1}{U} 2/1 Human Wizard with flash and prowess.
pub fn elementalist_adept() -> CardDefinition {
    CardDefinition {
        name: "Elementalist Adept",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Prowess],
        ..Default::default()
    }
}

/// Divine Resilience — {W} Instant with kicker {2}{W}. Target creature you
/// control gains indestructible until end of turn; if kicked, each creature you
/// control does instead. ("Any number of target creatures" is modeled as all
/// your creatures when kicked.)
pub fn divine_resilience() -> CardDefinition {
    CardDefinition {
        name: "Divine Resilience",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(2), w()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            }),
        },
        ..Default::default()
    }
}
