//! Apocalypse (APC), closing waves — the Volver kicker cycle, the Flagbearers,
//! the split cards and the remaining wedge utility. Tests in `classic_sets/apc2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// An Aura that enchants a creature.
fn aura(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..enchantment(name, c)
    }
}

/// The permanent this Aura is attached to.
fn enchanted() -> Selector {
    Selector::attached_to(Selector::This)
}

/// A 1/1 green Saproling.
pub(crate) fn saproling() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// How many lands with the given basic type the controller has.
fn lands_you_control(land: LandType) -> Value {
    Value::PermanentCountControlledByMatching(PlayerRef::You, R::HasLandType(land))
}

/// Living Airship — {3}{U} 2/3 flier that regenerates for {2}{G}.
pub fn living_airship() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Living Airship", cost(&[generic(3), u()]), vec![CreatureType::Metathran], 2, 3)
    }
}

/// Llanowar Dead — {B}{G} 2/2 that taps for black.
pub fn llanowar_dead() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black]),
            },
            ..Default::default()
        }],
        ..creature(
            "Llanowar Dead",
            cost(&[b(), g()]),
            vec![CreatureType::Zombie, CreatureType::Elf],
            2,
            2,
        )
    }
}

/// Minotaur Tactician — {3}{R} 1/1 haste that grows off allied colours.
pub fn minotaur_tactician() -> CardDefinition {
    let grows_with = |c: Color| StaticAbility {
        description: "Gets +1/+1 as long as you control a creature of an allied colour.",
        effect: StaticEffect::WhileCondition {
            condition: Predicate::SelectorExists(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasColor(c)),
            )),
            inner: Box::new(StaticEffect::PumpPT {
                applies_to: Selector::This,
                power: 1,
                toughness: 1,
            }),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![grows_with(Color::White), grows_with(Color::Blue)],
        ..creature(
            "Minotaur Tactician",
            cost(&[generic(3), r()]),
            vec![CreatureType::Minotaur],
            1,
            1,
        )
    }
}

/// Minotaur Illusionist — {3}{U}{R} 3/4 that dodges removal or throws itself.
pub fn minotaur_illusionist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Shroud,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                sac_cost: true,
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::This,
                    target: target_filtered(R::Creature),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Minotaur Illusionist",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Putrid Warrior — {W}{B} 2/2 whose damage swings everyone's life total.
pub fn putrid_warrior() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::ChooseN {
                picks: vec![0],
                modes: vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::ONE,
                    },
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::EachPlayer),
                        amount: Value::ONE,
                    },
                ],
            },
        }],
        ..creature(
            "Putrid Warrior",
            cost(&[w(), b()]),
            vec![CreatureType::Zombie, CreatureType::Soldier, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Martyrs' Tomb — {2}{W}{B}. Life into damage prevention.
pub fn martyrs_tomb() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::PreventNextDamage {
                target: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Martyrs' Tomb", cost(&[generic(2), w(), b()]))
    }
}

/// Tahngarth's Glare — {R}. Both players restack three; each rearrangement is
/// made by that library's owner.
pub fn tahngarths_glare() -> CardDefinition {
    sorcery(
        "Tahngarth's Glare",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::RearrangeTop { who: PlayerRef::Target(0), amount: Value::Const(3) },
            Effect::RearrangeTop { who: PlayerRef::You, amount: Value::Const(3) },
        ]),
    )
}

/// Manacles of Decay — {1}{W} Aura. Pins a creature down, with colour riders.
pub fn manacles_of_decay() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack],
            ..Default::default()
        }),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: enchanted(),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::GrantKeyword {
                    what: enchanted(),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..aura("Manacles of Decay", cost(&[generic(1), w()]))
    }
}

/// Yavimaya's Embrace — {5}{G}{U}{U} Aura. Steal it and make it bigger.
pub fn yavimayas_embrace() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains { what: enchanted() })],
        ..aura("Yavimaya's Embrace", cost(&[generic(5), g(), u(), u()]))
    }
}

/// Soul Link — {1}{W}{B} Aura. Damage the creature deals or takes feeds you.
pub fn soul_link() -> CardDefinition {
    let gain = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::EnchantedBySource),
        effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
    };
    CardDefinition {
        triggered_abilities: vec![gain(EventKind::DealsDamage), gain(EventKind::DealtDamage)],
        ..aura("Soul Link", cost(&[generic(1), w(), b()]))
    }
}

/// Planar Despair — {3}{B}{B}. Domain sweeper.
pub fn planar_despair() -> CardDefinition {
    let per_domain =
        || Value::Times(Box::new(Value::Const(-1)), Box::new(Value::DomainCount(PlayerRef::You)));
    sorcery(
        "Planar Despair",
        cost(&[generic(3), b(), b()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature),
            power: per_domain(),
            toughness: per_domain(),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Mask of Intolerance — {2}. Punishes greedy mana bases.
pub fn mask_of_intolerance() -> CardDefinition {
    CardDefinition {
        name: "Mask of Intolerance",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::ValueAtLeast(
                    Value::DomainCount(PlayerRef::ActivePlayer),
                    Value::Const(4),
                )),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

/// Whirlpool Warrior — {2}{U} 2/2 that redraws your hand, then everyone's.
pub fn whirlpool_warrior() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ShuffleHandsDrawSame { who: PlayerRef::You })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect: Effect::ShuffleHandsDrawSame { who: PlayerRef::EachPlayer },
            ..Default::default()
        }],
        ..creature(
            "Whirlpool Warrior",
            cost(&[generic(2), u()]),
            vec![CreatureType::Merfolk, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Symbiotic Deployment — {2}{G}. Trades your draw step for creature taps.
pub fn symbiotic_deployment() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Skip your draw step.",
            effect: StaticEffect::ControllerSkipsDrawStep,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_n_filter: Some((R::Creature, 2)),
            effect: draw(1),
            ..Default::default()
        }],
        ..enchantment("Symbiotic Deployment", cost(&[generic(2), g()]))
    }
}

/// Wild Research — {2}{R}. Tutors an enchantment or an instant, at a price.
pub fn wild_research() -> CardDefinition {
    let dig = |pip, filter| ActivatedAbility {
        mana_cost: cost(&[generic(1), pip]),
        effect: Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
        ]),
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![dig(w(), R::Enchantment), dig(u(), R::HasCardType(CardType::Instant))],
        ..enchantment("Wild Research", cost(&[generic(2), r()]))
    }
}

/// Guided Passage — {G}{U}{R}. Three cards, but an opponent picks them.
pub fn guided_passage() -> CardDefinition {
    let pick = |filter| Effect::SearchPickedBy {
        who: PlayerRef::You,
        picker: PlayerRef::EachOpponent,
        filter,
        to: ZoneDest::Hand(PlayerRef::You),
    };
    sorcery(
        "Guided Passage",
        cost(&[g(), u(), r()]),
        Effect::Seq(vec![
            pick(R::Creature),
            pick(R::Land),
            pick(R::Not(Box::new(R::Creature.or(R::Land)))),
        ]),
    )
}

/// Last Stand — {W}{U}{B}{R}{G}. Five basic-land payoffs at once.
pub fn last_stand() -> CardDefinition {
    sorcery(
        "Last Stand",
        cost(&[w(), u(), b(), r(), g()]),
        Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer },
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(lands_you_control(LandType::Swamp)),
                ),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: lands_you_control(LandType::Mountain),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: lands_you_control(LandType::Forest),
                definition: saproling(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(lands_you_control(LandType::Plains)),
                ),
            },
            Effect::Draw { who: Selector::You, amount: lands_you_control(LandType::Island) },
            Effect::Discard {
                who: Selector::You,
                amount: lands_you_control(LandType::Island),
                random: false,
            },
        ]),
    )
}
