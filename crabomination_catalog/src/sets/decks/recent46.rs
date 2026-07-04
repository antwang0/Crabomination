//! Green value creatures and ramp payoffs. Tests in `tests/recent46.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_attack, on_dies, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic};

fn etb(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource), effect }
}

/// Return target card from your graveyard to your hand (never the source — once
/// Greenwarden exiles itself it can't return itself).
fn return_gy_card() -> Effect {
    Effect::Move {
        what: target_filtered(R::InYourGraveyard.and(R::OtherThanSource)),
        to: ZoneDest::Hand(PlayerRef::You),
    }
}

/// Greenwarden of Murasa — {4}{G}{G} 5/4 Elemental. ETB: may return target card
/// from your graveyard to your hand. Dies: may exile it; if you do, return
/// target card from your graveyard to your hand.
pub fn greenwarden_of_murasa() -> CardDefinition {
    CardDefinition {
        name: "Greenwarden of Murasa",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Return target card from your graveyard to your hand.".into(),
                body: Box::new(return_gy_card()),
            }),
            on_dies(Effect::MayDo {
                description: "Exile this; if you do, return target card from your graveyard to your hand.".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Exile },
                    return_gy_card(),
                ])),
            }),
        ],
        ..Default::default()
    }
}

/// Nantuko Vigilante — {3}{G} 3/2 Insect Druid Mutant. Morph {1}{G}. When turned
/// face up, destroy target artifact or enchantment.
pub fn nantuko_vigilante() -> CardDefinition {
    CardDefinition {
        name: "Nantuko Vigilante",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Druid, CreatureType::Mutant],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Morph(cost(&[generic(1), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        }],
        ..Default::default()
    }
}

/// Bramble Sovereign — {2}{G}{G} 4/4 Dryad. Whenever another nontoken creature
/// enters, you may pay {1}{G}. If you do, that creature's controller creates a
/// token that's a copy of it.
pub fn bramble_sovereign() -> CardDefinition {
    CardDefinition {
        name: "Bramble Sovereign",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dryad], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::OtherThanSource),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {1}{G} to copy that creature.".into(),
                mana_cost: cost(&[generic(1), g()]),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    count: Value::Const(1),
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Masked Admirers — {2}{G}{G} 3/2 Elf Shaman. ETB draw a card. Whenever you
/// cast a creature spell, you may pay {G}{G} to return this from your graveyard
/// to your hand.
pub fn masked_admirers() -> CardDefinition {
    CardDefinition {
        name: "Masked Admirers",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: Effect::MayPay {
                    description: "Pay {G}{G} to return Masked Admirers to your hand.".into(),
                    mana_cost: cost(&[g(), g()]),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Verdurous Gearhulk — {3}{G}{G} 4/4 Construct artifact, trample. ETB
/// distribute four +1/+1 counters among any number of target creatures you
/// control.
pub fn verdurous_gearhulk() -> CardDefinition {
    CardDefinition {
        name: "Verdurous Gearhulk",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(4),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature.and(R::ControlledByYou),
            max_targets: 4,
        })],
        ..Default::default()
    }
}

/// Pathbreaker Ibex — {4}{G}{G} 3/3 Goat. Whenever it attacks, creatures you
/// control gain trample and get +X/+X until end of turn, where X is the greatest
/// power among creatures you control.
pub fn pathbreaker_ibex() -> CardDefinition {
    let each_creature = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Pathbreaker Ibex",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goat], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            // X (greatest power among your creatures) is read once, then the
            // same +X/+X is layered onto every creature you control.
            Effect::PumpPT {
                what: each_creature(),
                power: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                toughness: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: each_creature(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Ghalta, Primal Hunger — {10}{G}{G} 12/12 legendary Elder Dinosaur, trample.
/// Costs {X} less to cast, where X is the total power of creatures you control.
pub fn ghalta_primal_hunger() -> CardDefinition {
    CardDefinition {
        name: "Ghalta, Primal Hunger",
        cost: cost(&[generic(10), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 12,
        toughness: 12,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the total power of creatures you control.",
            effect: StaticEffect::SelfCostReducedByTotalPower,
        }],
        ..Default::default()
    }
}

/// Lifecrafter's Bestiary — {3} Artifact. At your upkeep, scry 1. Whenever you
/// cast a creature spell, you may pay {G} to draw a card.
pub fn lifecrafters_bestiary() -> CardDefinition {
    CardDefinition {
        name: "Lifecrafter's Bestiary",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                ),
                effect: Effect::MayPay {
                    description: "Pay {G} to draw a card.".into(),
                    mana_cost: cost(&[g()]),
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}
