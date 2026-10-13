//! Commander: two-card gaps of EDHREC's top-2000 commanders (the 2026-10-13
//! rescan), tenth file — Grist, Jyoti, Jolrael, Brigone, Storvald, Naban
//! and Matoya. Tests in `tests/recent_b/cmdr_rank2k_j.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, renown, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w, Color, ManaCost};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn pump_eot(what: Selector, p: Value, t: Value) -> Effect {
    Effect::PumpPT { what, power: p, toughness: t, duration: Duration::EndOfTurn }
}

fn scry(n: i32) -> Effect {
    Effect::Scry { who: PlayerRef::You, amount: Value::Const(n) }
}

fn lands_you_control() -> Value {
    Value::count(Selector::EachPermanent(yours(R::Land)))
}

/// Cryptic Annelid — {3}{U} 1/4 Worm Beast; entering scries 1, then 2, then
/// 3.
pub fn cryptic_annelid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![scry(1), scry(2), scry(3)]))],
        ..creature("Cryptic Annelid", cost(&[generic(3), u()]), vec![CreatureType::Worm, CreatureType::Beast], 1, 4)
    }
}

/// Devouring Swarm — {1}{B}{B} 2/1 flying Insect; sacrifice a creature: +1/+1
/// until end of turn.
pub fn devouring_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: pump_eot(Selector::This, Value::ONE, Value::ONE),
            ..Default::default()
        }],
        ..creature("Devouring Swarm", cost(&[generic(1), b(), b()]), vec![CreatureType::Insect], 2, 1)
    }
}

/// Emerge Unscathed — {W} instant: a creature you control gains protection
/// from the color of your choice until end of turn. Rebound.
pub fn emerge_unscathed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Emerge Unscathed",
            cost(&[w()]),
            CardType::Instant,
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(yours(R::Creature)),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Giant's Grasp — {2}{U}{U} Aura, enchant Giant you control. Entering, it
/// gains you control of target nonland permanent for as long as it remains
/// on the battlefield.
pub fn giants_grasp() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: target_filtered(R::Permanent.and(R::Nonland)),
        })],
        ..spell(
            "Giant's Grasp",
            cost(&[generic(2), u(), u()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(yours(R::Creature.and(R::HasCreatureType(CreatureType::Giant)))),
            },
        )
    }
}

/// Kraul Harpooner — {1}{G} 3/2 reach Insect Warrior. Undergrowth — entering,
/// choose up to one target flier you don't control; it gets +X/+0 (X =
/// creature cards in your graveyard), then may fight that creature.
pub fn kraul_harpooner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                pump_eot(
                    Selector::This,
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
                    Value::ZERO,
                ),
                Effect::MayDo {
                    description: "Have Kraul Harpooner fight that creature?".into(),
                    body: Box::new(Effect::Fight {
                        attacker: Selector::This,
                        defender: target_filtered(
                            R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::ControlledByOpponent),
                        ),
                    }),
                },
            ])),
        })],
        ..creature(
            "Kraul Harpooner",
            cost(&[generic(1), g()]),
            vec![CreatureType::Insect, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Liege of the Tangle — {6}{G}{G} 8/8 trample Elemental. Combat damage to a
/// player: any number of target lands you control get an awakening counter
/// and are 8/8 green Elemental creatures while they have one.
pub fn liege_of_the_tangle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: yours(R::Land),
                effect: Box::new(Effect::Seq(vec![
                    Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Awakening, amount: Value::ONE },
                    Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::Const(8),
                        toughness: Value::Const(8),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![],
                        duration: Duration::WhileHasCounter(CounterType::Awakening),
                    },
                    Effect::BecomeColor {
                        what: Selector::Target(0),
                        colors: vec![Color::Green],
                        duration: Duration::WhileHasCounter(CounterType::Awakening),
                        additive: false,
                    },
                ])),
            },
        }],
        ..creature("Liege of the Tangle", cost(&[generic(6), g(), g()]), vec![CreatureType::Elemental], 8, 8)
    }
}

/// Mystic Speculation — {U} sorcery, buyback {2}: scry 3.
pub fn mystic_speculation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(cost(&[generic(2)]))],
        ..spell("Mystic Speculation", cost(&[u()]), CardType::Sorcery, scry(3))
    }
}

/// Outland Colossus — {3}{G}{G} 6/6 Giant; renown 6; can't be blocked by more
/// than one creature.
pub fn outland_colossus() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        triggered_abilities: vec![renown(6)],
        ..creature("Outland Colossus", cost(&[generic(3), g(), g()]), vec![CreatureType::Giant], 6, 6)
    }
}

/// Pol Jamaar, Illusionist — {4}{U}{U} legendary 4/5 flying Human Illusion
/// Wizard; entering, choose a creature type and draw a card per creature you
/// control of that type.
pub fn pol_jamaar_illusionist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(yours(R::Creature.and(R::IsSourceChosenCreatureType)))),
            }),
        })],
        ..creature(
            "Pol Jamaar, Illusionist",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Human, CreatureType::Illusion, CreatureType::Wizard],
            4,
            5,
        )
    }
}

/// Tatyova, Steward of Tides — {G}{G}{U} legendary 3/3 Merfolk Druid. Land
/// creatures you control have flying. Whenever a land you control enters, if
/// you control seven or more lands, up to one target land you control becomes
/// a 3/3 Elemental creature with haste (still a land).
pub fn tatyova_steward_of_tides() -> CardDefinition {
    let seven_lands = || Predicate::ValueAtLeast(lands_you_control(), Value::Const(7));
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Land creatures you control have flying.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Land.and(R::Creature))),
                keyword: Keyword::Flying,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                seven_lands(),
            ])),
            effect: Effect::If {
                cond: seven_lands(),
                then: Box::new(Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::BecomeCreature {
                        what: target_filtered(yours(R::Land)),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![Keyword::Haste],
                        duration: Duration::Permanent,
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Tatyova, Steward of Tides",
            cost(&[g(), g(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Wings of the Cosmos — {W} instant: target creature gets +1/+3 and flying
/// until end of turn; untap it.
pub fn wings_of_the_cosmos() -> CardDefinition {
    spell(
        "Wings of the Cosmos",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump_eot(target_filtered(R::Creature), Value::ONE, Value::Const(3)),
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
    )
}

/// Winter Eladrin — {2}{U} 2/2 Faerie Elf Wizard. Gust of Wind — entering,
/// return up to one other target creature to its owner's hand.
pub fn winter_eladrin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        ..creature(
            "Winter Eladrin",
            cost(&[generic(2), u()]),
            vec![CreatureType::Faerie, CreatureType::Elf, CreatureType::Wizard],
            2,
            2,
        )
    }
}
