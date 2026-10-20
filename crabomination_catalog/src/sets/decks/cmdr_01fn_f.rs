//! Commander: partner-pair closers, batch three (`01FnAVVR`). Tests in
//! `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};

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

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn on_cast_color(color: Color, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(color))),
        effect,
    }
}

/// Snooping Newsie — {U}{B} 2/2 Human Rogue. ETB: mill two. With five or more
/// mana values among cards in your graveyard, +1/+1 and lifelink.
pub fn snooping_newsie() -> CardDefinition {
    let threshold =
        || Predicate::ValueAtLeast(Value::DistinctManaValuesInGraveyard(PlayerRef::You), Value::Const(5));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Mill { who: Selector::You, amount: Value::Const(2) })],
        static_abilities: vec![
            StaticAbility {
                description: "As long as there are five or more mana values among cards in your graveyard, this creature gets +1/+1.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::PumpPT { applies_to: Selector::This, power: 1, toughness: 1 }),
                },
            },
            StaticAbility {
                description: "… and has lifelink.",
                effect: StaticEffect::WhileCondition {
                    condition: threshold(),
                    inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Lifelink }),
                },
            },
        ],
        ..creature("Snooping Newsie", cost(&[u(), b()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 2)
    }
}

/// Putrefax — {3}{G}{G} 5/3 trample, haste, infect Phyrexian Horror,
/// sacrificed at the beginning of the end step.
pub fn putrefax() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Infect],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::SacrificeSource,
        }],
        ..creature("Putrefax", cost(&[generic(3), g(), g()]), vec![CreatureType::Phyrexian, CreatureType::Horror], 5, 3)
    }
}

/// Gravelgill Duo — {2}{U/B} 2/1 Merfolk Rogue Warrior. A blue spell you
/// cast: +1/+1; a black one: fear, each until end of turn.
pub fn gravelgill_duo() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_cast_color(Color::Blue, pump(Selector::This, 1, 1)),
            on_cast_color(
                Color::Black,
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Fear, duration: Duration::EndOfTurn },
            ),
        ],
        ..creature(
            "Gravelgill Duo",
            ManaCost::new(vec![generic(2), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Merfolk, CreatureType::Rogue, CreatureType::Warrior],
            2,
            1,
        )
    }
}

/// Prophet of the Peak — {6} 5/5 Cat artifact creature. ETB: scry 2.
pub fn prophet_of_the_peak() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        ..creature("Prophet of the Peak", cost(&[generic(6)]), vec![CreatureType::Cat], 5, 5)
    }
}

/// Plunder — {4}{R} sorcery, suspend 4—{1}{R}. Destroy target artifact or
/// land.
pub fn plunder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(4, cost(&[generic(1), r()]))],
        ..spell(
            "Plunder",
            cost(&[generic(4), r()]),
            CardType::Sorcery,
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Land)) },
        )
    }
}

/// Madblind Mountain — Mountain land, enters tapped. {R}, {T}: shuffle your
/// library, with two or more red permanents.
pub fn madblind_mountain() -> CardDefinition {
    CardDefinition {
        name: "Madblind Mountain",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Mountain], ..Default::default() },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasColor(Color::Red).and(R::ControlledByYou)),
                    n: Value::Const(2),
                }),
                effect: Effect::ShuffleLibrary { who: PlayerRef::You },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Baleful Stare — {2}{U} sorcery. Target opponent reveals their hand; draw
/// a card for each Mountain and red card in it.
pub fn baleful_stare() -> CardDefinition {
    spell(
        "Baleful Stare",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            crate::effect::shortcut::declare_target_opponent(),
            Effect::RevealHand { who: PlayerRef::Target(0) },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CardsInHandMatching {
                    who: PlayerRef::Target(0),
                    filter: R::HasLandType(LandType::Mountain).or(R::HasColor(Color::Red)),
                },
            },
        ]),
    )
}

/// Snow Fortress — {5} 0/4 Wall artifact creature, defender. {1}: +1/+0;
/// {1}: +0/+1; {3}: 1 damage to target creature without flying attacking
/// you.
pub fn snow_fortress() -> CardDefinition {
    let one = |effect: Effect| ActivatedAbility { mana_cost: cost(&[generic(1)]), effect, ..Default::default() };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![
            one(pump(Selector::This, 1, 0)),
            one(pump(Selector::This, 0, 1)),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::IsAttackingYou).and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature("Snow Fortress", cost(&[generic(5)]), vec![CreatureType::Wall], 0, 4)
    }
}

/// Divine Smite — {1}{W} instant. Target creature or planeswalker an
/// opponent controls phases out; a black one is exiled instead.
pub fn divine_smite() -> CardDefinition {
    spell(
        "Divine Smite",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::EntityMatches {
                what: target_filtered(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
                filter: R::HasColor(Color::Black),
            },
            then: Box::new(Effect::Exile { what: Selector::Target(0) }),
            else_: Box::new(Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: false }),
        },
    )
}

/// Spiral into Solitude — {1}{W} Aura. Enchanted creature can't attack or
/// block. {1}{W}, blight 1, sacrifice it: exile enchanted creature.
pub fn spiral_into_solitude() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Spiral into Solitude",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: [Keyword::CantAttack, Keyword::CantBlock]
            .into_iter()
            .map(|keyword| StaticAbility {
                description: "Enchanted creature can't attack or block.",
                effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword },
            })
            .collect(),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            blight_cost: 1,
            sac_cost: true,
            effect: Effect::Exile { what: enchanted() },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skullmulcher — {4}{G} 3/3 Elemental, devour 1; it draws a card for each
/// creature devoured.
pub fn skullmulcher() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Creature.and(R::OtherThanSource),
            per_each: Box::new(Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ])),
        }),
        ..creature("Skullmulcher", cost(&[generic(4), g()]), vec![CreatureType::Elemental], 3, 3)
    }
}

/// Screaming Swarm — {5}{U} 4/4 flying Bird Horror. Attacking with any
/// creatures, target player mills that many. {2}{U}: from your graveyard
/// into your library second from the top.
pub fn screaming_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::count(Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou))),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::FromTop(1) } },
            ..Default::default()
        }],
        ..creature("Screaming Swarm", cost(&[generic(5), u()]), vec![CreatureType::Bird, CreatureType::Horror], 4, 4)
    }
}
