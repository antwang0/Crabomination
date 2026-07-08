//! Modern-deck staples batch 112 — {Q} untap costs (CR 107.17), Phyrexian
//! Unlife's ≤0-life mode, and spellslinger bodies. Tests in
//! `tests/recent112.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement, StaticAbility, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::cast_is_noncreature;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, Value};
use crate::mana::{b, cost, g, generic, hybrid, u, w, Color, ManaCost};

/// Pili-Pala — {2} 1/1 flying Scarecrow. {2}, {Q}: Add one mana of any
/// color (CR 107.17).
pub fn pili_pala() -> CardDefinition {
    CardDefinition {
        name: "Pili-Pala",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scarecrow],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            untap_self_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Phyrexian Unlife — {2}{W} Enchantment. You don't lose at 0 or less life;
/// at ≤ 0 life all damage hits you as though its source had infect.
pub fn phyrexian_unlife() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Unlife",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "You don't lose the game for having 0 or less life; at 0 or less life, damage is dealt to you as though its source had infect.",
            effect: StaticEffect::ControllerDoesntLoseFromLife,
        }],
        ..Default::default()
    }
}

/// Salvage Titan — {4}{B}{B} 6/4; sacrifice three artifacts instead of
/// paying; exile three artifact cards from your graveyard to return it.
pub fn salvage_titan() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Salvage Titan",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 6,
        toughness: 4,
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            sacrifice_permanents: Some((SelectionRequirement::Artifact, 3)),
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            exile_other_filter: Some((SelectionRequirement::Artifact, 3)),
            effect: Effect::Move {
                what: Selector::This,
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Qasali Ambusher — {1}{G}{W} 2/3 reach; free flash cast while a creature
/// attacks you and you control a Forest and a Plains (attacker check is
/// any-opponent's in multiplayer).
pub fn qasali_ambusher() -> CardDefinition {
    use crate::card::{AlternativeCost, LandType};
    CardDefinition {
        name: "Qasali Ambusher",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            flash: true,
            condition: Some(Predicate::All(vec![
                Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::IsAttacking)
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
                Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::HasLandType(LandType::Forest)
                        .and(SelectionRequirement::ControlledByYou),
                )),
                Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::HasLandType(LandType::Plains)
                        .and(SelectionRequirement::ControlledByYou),
                )),
            ])),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Boros Reckoner — {R/W}{R/W}{R/W} 3/3; damage dealt to it bounces to any
/// target; {R/W}: first strike EOT.
pub fn boros_reckoner() -> CardDefinition {
    CardDefinition {
        name: "Boros Reckoner",
        cost: cost(&[
            hybrid(Color::Red, Color::White),
            hybrid(Color::Red, Color::White),
            hybrid(Color::Red, Color::White),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::TriggerEventAmount,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::Red, Color::White)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blistercoil Weird — {U/R} 1/1; an instant/sorcery cast pumps it +1/+1
/// EOT and untaps it.
pub fn blistercoil_weird() -> CardDefinition {
    CardDefinition {
        name: "Blistercoil Weird",
        cost: cost(&[hybrid(Color::Blue, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Weird], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::This, up_to: None },
        ]))],
        ..Default::default()
    }
}

/// Sage of the Falls — {4}{U} 2/5; a non-Human creature you control
/// entering (itself included) offers a loot.
pub fn sage_of_the_falls() -> CardDefinition {
    CardDefinition {
        name: "Sage of the Falls",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Not(Box::new(
                            SelectionRequirement::HasCreatureType(CreatureType::Human),
                        ))),
                },
            ),
            effect: Effect::MayDo {
                description: "Draw a card, then discard a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Elusive Spellfist — {1}{U} 1/3; a noncreature cast gives +1/+0 and
/// unblockable until end of turn.
pub fn elusive_spellfist() -> CardDefinition {
    CardDefinition {
        name: "Elusive Spellfist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}
