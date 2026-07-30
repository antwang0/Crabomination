//! Oath of the Gatewatch (OGW) gap wave 2 — Cohort, support, and the Ally /
//! Equipment payoffs. Tests in `classic_sets/ogw`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{deal, draw, etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, StaticEffect,
    TriggeredAbility, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w};

fn types(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Cohort (OGW) — "{T}, Tap an untapped Ally you control: [effect]".
fn cohort(effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        tap_other_filter: Some(R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou)),
        effect,
        ..Default::default()
    }
}

fn ally(
    name: &'static str,
    c: crate::mana::ManaCost,
    p: i32,
    t: i32,
    extra: Vec<CreatureType>,
) -> CardDefinition {
    let mut ct = extra;
    ct.push(CreatureType::Ally);
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: types(ct),
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Akoum Flameseeker — {2}{R} 3/2 Human Shaman Ally. Cohort: loot 1.
pub fn akoum_flameseeker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cohort(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            draw(1),
        ]))],
        ..ally(
            "Akoum Flameseeker",
            cost(&[generic(2), r()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
        )
    }
}

/// Malakir Soothsayer — {4}{B} 4/4 Vampire Shaman Ally. Cohort: draw a card
/// and lose 1 life.
pub fn malakir_soothsayer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cohort(Effect::Seq(vec![
            draw(1),
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..ally(
            "Malakir Soothsayer",
            cost(&[generic(4), b()]),
            4,
            4,
            vec![CreatureType::Vampire, CreatureType::Shaman],
        )
    }
}

/// Ondu War Cleric — {1}{W} 2/2 Human Cleric Ally. Cohort: gain 2 life.
pub fn ondu_war_cleric() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cohort(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..ally(
            "Ondu War Cleric",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Human, CreatureType::Cleric],
        )
    }
}

/// Spawnbinder Mage — {3}{W} 2/4 Human Wizard Ally. Cohort: tap target
/// creature.
pub fn spawnbinder_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cohort(Effect::Tap {
            what: target_filtered(R::Creature),
        })],
        ..ally(
            "Spawnbinder Mage",
            cost(&[generic(3), w()]),
            2,
            4,
            vec![CreatureType::Human, CreatureType::Wizard],
        )
    }
}

/// Stoneforge Acolyte — {W} 1/2 Kor Artificer Ally. Cohort: dig four for an
/// Equipment.
pub fn stoneforge_acolyte() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cohort(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)),
            optional: true,
            rest_to_graveyard: false,
            take: None,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
        })],
        ..ally(
            "Stoneforge Acolyte",
            cost(&[w()]),
            1,
            2,
            vec![CreatureType::Kor, CreatureType::Artificer],
        )
    }
}

/// Zada's Commando — {1}{R} 2/1 Goblin Archer Ally with first strike. Cohort:
/// 1 damage to target opponent or planeswalker.
pub fn zadas_commando() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![cohort(deal(
            1,
            target_filtered(R::OpponentPlayer.or(R::HasCardType(CardType::Planeswalker))),
        ))],
        ..ally(
            "Zada's Commando",
            cost(&[generic(1), r()]),
            2,
            1,
            vec![CreatureType::Goblin, CreatureType::Archer],
        )
    }
}

/// Zulaport Chainmage — {3}{B} 4/2 Human Shaman Ally. Cohort: target opponent
/// loses 2 life.
pub fn zulaport_chainmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![cohort(Effect::LoseLife {
            who: target_filtered(R::OpponentPlayer),
            amount: Value::Const(2),
        })],
        ..ally(
            "Zulaport Chainmage",
            cost(&[generic(3), b()]),
            4,
            2,
            vec![CreatureType::Human, CreatureType::Shaman],
        )
    }
}

/// Joraga Auxiliary — {1}{G}{W} 2/3 Elf Soldier Ally. {4}{G}{W}: Support 2.
pub fn joraga_auxiliary() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g(), w()]),
            effect: Effect::SupportCounters {
                max_targets: 2,
                filter: R::Creature.and(R::OtherThanSource),
            },
            ..Default::default()
        }],
        ..ally(
            "Joraga Auxiliary",
            cost(&[generic(1), g(), w()]),
            2,
            3,
            vec![CreatureType::Elf, CreatureType::Soldier],
        )
    }
}

/// Relief Captain — {2}{W}{W} 3/2 Kor Knight Ally. ETB support 3.
pub fn relief_captain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SupportCounters {
            max_targets: 3,
            filter: R::Creature.and(R::OtherThanSource),
        })],
        ..ally(
            "Relief Captain",
            cost(&[generic(2), w(), w()]),
            3,
            2,
            vec![CreatureType::Kor, CreatureType::Knight],
        )
    }
}

/// Gladehart Cavalry — {5}{G}{G} 6/6 Elf Knight. ETB support 6; gain 2 life
/// whenever a creature you control with a +1/+1 counter dies.
pub fn gladehart_cavalry() -> CardDefinition {
    CardDefinition {
        name: "Gladehart Cavalry",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Elf, CreatureType::Knight]),
        power: 6,
        toughness: 6,
        triggered_abilities: vec![
            etb(Effect::SupportCounters {
                max_targets: 6,
                filter: R::Creature.and(R::OtherThanSource),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithCounter(CounterType::PlusOnePlusOne),
                    }),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Nissa's Judgment — {4}{G} Sorcery. Support 2, then each of your creatures
/// with a +1/+1 counter fights-damages a target opposing creature.
pub fn nissas_judgment() -> CardDefinition {
    CardDefinition {
        name: "Nissa's Judgment",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SupportCounters {
                max_targets: 2,
                filter: R::Creature,
            },
            Effect::EachDealsDamageEqualToPower {
                dealers: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                target: Selector::TargetFiltered {
                    slot: 2,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Unity of Purpose — {3}{U} Instant. Support 2, then untap each of your
/// creatures with a +1/+1 counter.
pub fn unity_of_purpose() -> CardDefinition {
    CardDefinition {
        name: "Unity of Purpose",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SupportCounters {
                max_targets: 2,
                filter: R::Creature,
            },
            Effect::Untap {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::WithCounter(CounterType::PlusOnePlusOne)),
                ),
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

/// Weapons Trainer — {R}{W} 3/2 Human Soldier Ally. Other creatures you
/// control get +1/+0 while you control an Equipment.
pub fn weapons_trainer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0 as long as you control an Equipment.",
            effect: StaticEffect::AnthemForFilterIf {
                filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                power: 1,
                toughness: 0,
                keywords: vec![],
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)
                            .and(R::ControlledByYou),
                    ),
                    n: Value::Const(1),
                },
            },
        }],
        ..ally(
            "Weapons Trainer",
            cost(&[r(), w()]),
            3,
            2,
            vec![CreatureType::Human, CreatureType::Soldier],
        )
    }
}

/// Stone Haven Outfitter — {1}{W} 2/2 Kor Artificer Ally. Equipped creatures
/// you control get +1/+1; draw when one dies.
pub fn stone_haven_outfitter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Equipped creatures you control get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou).and(R::IsEquipped),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsEquipped,
                },
            ),
            effect: draw(1),
        }],
        ..ally(
            "Stone Haven Outfitter",
            cost(&[generic(1), w()]),
            2,
            2,
            vec![CreatureType::Kor, CreatureType::Artificer],
        )
    }
}

/// Steppe Glider — {4}{W} 2/4 Elemental with flying and vigilance. {1}{W}:
/// target creature with a +1/+1 counter gains flying and vigilance.
pub fn steppe_glider() -> CardDefinition {
    CardDefinition {
        name: "Steppe Glider",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: types(vec![CreatureType::Elemental]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne)),
                    ),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// General Tazri — {4}{W} 3/4 legendary Human Ally. ETB tutors an Ally to
/// hand; {W}{U}{B}{R}{G} pumps your Allies by their colour count.
pub fn general_tazri() -> CardDefinition {
    let allies = Selector::EachPermanent(
        R::Creature
            .and(R::ControlledByYou)
            .and(R::HasCreatureType(CreatureType::Ally)),
    );
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: R::Creature.and(R::HasCreatureType(CreatureType::Ally)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::PumpPT {
                what: allies.clone(),
                power: Value::DistinctColorsAmong(Box::new(allies.clone())),
                toughness: Value::DistinctColorsAmong(Box::new(allies)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..ally(
            "General Tazri",
            cost(&[generic(4), w()]),
            3,
            4,
            vec![CreatureType::Human],
        )
    }
}
