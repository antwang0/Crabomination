//! Commander: the cards the **First Flight** precon (SCD, Isperia, Supreme
//! Judge) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_isperia.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType,
    LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{encore, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{cost, generic, u, w, Color, ManaCost};
use std::sync::Arc;

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

fn flier(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Flying], ..creature(name, mana, types, p, t) }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn flying() -> R {
    R::HasKeyword(Keyword::Flying)
}

fn your_fliers() -> R {
    R::Creature.and(flying()).and(R::ControlledByYou)
}

/// "Whenever this creature attacks, [effect]."
fn on_attack(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect }
}

fn pump_each(filter: R, power: i32, toughness: i32) -> Effect {
    Effect::PumpPT {
        what: Selector::EachPermanent(filter),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }
}

fn thopter(color: Option<Color>) -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: color.into_iter().collect(),
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

/// Angler Turtle — {5}{U}{U} 5/7 hexproof Turtle. Creatures your opponents
/// control attack each combat if able.
pub fn angler_turtle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![StaticAbility {
            description: "Creatures your opponents control attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                keyword: Keyword::MustAttack,
            },
        }],
        ..creature("Angler Turtle", cost(&[generic(5), u(), u()]), vec![CreatureType::Turtle], 5, 7)
    }
}

/// Aven Gagglemaster — {3}{W}{W} 4/3 flying Bird Warrior. ETB: gain 2 life
/// for each creature you control with flying.
pub fn aven_gagglemaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::CountOf(Box::new(Selector::EachPermanent(your_fliers())))),
            ),
        })],
        ..flier("Aven Gagglemaster", cost(&[generic(3), w(), w()]), vec![CreatureType::Bird, CreatureType::Warrior], 4, 3)
    }
}

/// Cartographer's Hawk — {1}{W} 2/1 flying Bird. Combat damage to a player
/// who controls more lands than you: return it to its owner's hand, and if
/// you do you may fetch a Plains tapped.
pub fn cartographers_hawk() -> CardDefinition {
    let lands = |who| Value::PermanentCountControlledByMatching(who, R::Land);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            // The damaged player is slot 0.
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::Diff(Box::new(lands(PlayerRef::Target(0))), Box::new(lands(PlayerRef::You))),
                    Value::ONE,
                ),
                then: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                    Effect::MayDo {
                        description: "Search your library for a Plains card?".into(),
                        body: Box::new(Effect::SearchUpToN {
                            who: PlayerRef::You,
                            filter: R::HasLandType(LandType::Plains),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                            count: Value::ONE,
                        }),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..flier("Cartographer's Hawk", cost(&[generic(1), w()]), vec![CreatureType::Bird], 2, 1)
    }
}

/// Ever-Watching Threshold — {2}{U} Enchantment. Whenever an opponent attacks
/// you and/or a planeswalker you control, draw a card.
pub fn ever_watching_threshold() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.2c — one fire per declaration, however many attack.
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).once_per_batch(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..enchantment("Ever-Watching Threshold", cost(&[generic(2), u()]))
    }
}

/// Favorable Winds — {1}{U} Enchantment. Creatures you control with flying
/// get +1/+1.
pub fn favorable_winds() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with flying get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: Selector::EachPermanent(your_fliers()), power: 1, toughness: 1 },
        }],
        ..enchantment("Favorable Winds", cost(&[generic(1), u()]))
    }
}

/// Gideon Jura — {3}{W}{W} loyalty 6. +2: during target opponent's next turn,
/// their creatures attack Gideon if able. −2: destroy target tapped creature.
/// 0: he becomes a 6/6 Human Soldier creature until end of turn and damage to
/// him is prevented.
pub fn gideon_jura() -> CardDefinition {
    CardDefinition {
        name: "Gideon Jura",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Gideon], ..Default::default() },
        base_loyalty: 6,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::LureCreaturesToSourceNextTurn { who: target_filtered(R::OpponentPlayer) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(6),
                        toughness: Value::Const(6),
                        creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::PreventAllDamageThisTurn { target: Selector::This, redirect_to: None },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Gravitational Shift — {3}{U}{U} Enchantment. Creatures with flying get
/// +2/+0; creatures without flying get −2/−0.
pub fn gravitational_shift() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Creatures with flying get +2/+0.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(flying())),
                    power: 2,
                    toughness: 0,
                },
            },
            StaticAbility {
                description: "Creatures without flying get -2/-0.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(flying())))),
                    power: -2,
                    toughness: 0,
                },
            },
        ],
        ..enchantment("Gravitational Shift", cost(&[generic(3), u(), u()]))
    }
}

/// Inspired Sphinx — {5}{U}{U} 5/5 flying Sphinx. ETB: draw a card for each
/// opponent you have. {3}{U}: a 1/1 colorless flying Thopter.
pub fn inspired_sphinx() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::OpponentCount })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(thopter(None)) },
            ..Default::default()
        }],
        ..flier("Inspired Sphinx", cost(&[generic(5), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

/// Kangee's Lieutenant — {2}{W} 1/1 flying Bird Soldier. Attacking: attacking
/// creatures with flying get +1/+1. Encore {5}{W}.
pub fn kangees_lieutenant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(pump_each(R::IsAttacking.and(flying()), 1, 1))],
        activated_abilities: vec![encore(cost(&[generic(5), w()]))],
        ..flier("Kangee's Lieutenant", cost(&[generic(2), w()]), vec![CreatureType::Bird, CreatureType::Soldier], 1, 1)
    }
}

/// Kangee, Sky Warden — {3}{W}{U} 3/3 legendary Bird Wizard; flying,
/// vigilance. Attacking: attacking fliers get +2/+0. Blocking: blocking fliers
/// get +0/+2.
pub fn kangee_sky_warden() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![
            on_attack(pump_each(R::IsAttacking.and(flying()), 2, 0)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: pump_each(R::IsBlocking.and(flying()), 0, 2),
            },
        ],
        ..creature(
            "Kangee, Sky Warden",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Moorland Haunt — Land. {T}: {C}. {W}{U}, {T}, exile a creature card from
/// your graveyard: a 1/1 white flying Spirit.
pub fn moorland_haunt() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Moorland Haunt",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[w(), u()]),
                exile_other_filter: Some((R::Creature, 1)),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spirit) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Remorseful Cleric — {1}{W} 2/1 flying Spirit Cleric. Sacrifice it: exile
/// target player's graveyard.
pub fn remorseful_cleric() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
            ..Default::default()
        }],
        ..flier("Remorseful Cleric", cost(&[generic(1), w()]), vec![CreatureType::Spirit, CreatureType::Cleric], 2, 1)
    }
}

/// Sharding Sphinx — {4}{U}{U} 4/4 flying artifact Sphinx. Whenever an
/// artifact creature you control deals combat damage to a player, you may
/// create a 1/1 blue flying Thopter.
pub fn sharding_sphinx() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCardType(CardType::Artifact) },
            ),
            effect: Effect::MayDo {
                description: "Create a 1/1 blue Thopter?".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(thopter(Some(Color::Blue))),
                }),
            },
        }],
        ..flier("Sharding Sphinx", cost(&[generic(4), u(), u()]), vec![CreatureType::Sphinx], 4, 4)
    }
}

/// Sphinx of Enlightenment — {4}{U}{U} 5/5 flying Sphinx. ETB: target
/// opponent draws a card and you draw three.
pub fn sphinx_of_enlightenment() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..flier("Sphinx of Enlightenment", cost(&[generic(4), u(), u()]), vec![CreatureType::Sphinx], 5, 5)
    }
}

/// Steel-Plume Marshal — {3}{W}{W} 3/3 flying Bird Soldier. Attacking: other
/// attacking creatures you control with flying get +2/+2.
pub fn steel_plume_marshal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(pump_each(
            R::IsAttacking.and(flying()).and(R::ControlledByYou).and(R::OtherThanSource),
            2,
            2,
        ))],
        ..flier("Steel-Plume Marshal", cost(&[generic(3), w(), w()]), vec![CreatureType::Bird, CreatureType::Soldier], 3, 3)
    }
}

/// Thunderclap Wyvern — {2}{W}{U} 2/3 Drake; flash, flying. Other creatures
/// you control with flying get +1/+1.
pub fn thunderclap_wyvern() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(your_fliers().and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        }],
        ..creature("Thunderclap Wyvern", cost(&[generic(2), w(), u()]), vec![CreatureType::Drake], 2, 3)
    }
}

/// Tide Skimmer — {3}{U} 2/3 flying Drake. Whenever you attack with two or
/// more creatures with flying, draw a card.
pub fn tide_skimmer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::IsAttacking.and(flying()).and(R::ControlledByYou)),
                    n: Value::Const(2),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..flier("Tide Skimmer", cost(&[generic(3), u()]), vec![CreatureType::Drake], 2, 3)
    }
}

/// Windreader Sphinx — {5}{U}{U} 3/7 flying Sphinx. Whenever a creature with
/// flying attacks (anyone's), you may draw a card.
pub fn windreader_sphinx() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: flying() }),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..flier("Windreader Sphinx", cost(&[generic(5), u(), u()]), vec![CreatureType::Sphinx], 3, 7)
    }
}
