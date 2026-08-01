//! Apocalypse (APC) — 2001. The Invasion block's wedge-colour finale: the
//! Disciple / Sanctuary / Volver cycles, the Bloodfire sacrifice creatures and
//! the kicker spells. Tests in `classic_sets/apc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, pump_target, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
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

/// "You control a permanent of this colour."
fn controls_color(c: Color) -> Predicate {
    Predicate::SelectorExists(Selector::EachPermanent(R::HasColor(c).and(R::ControlledByYou)))
}

/// A Disciple's `{cost}, {T}: [effect]` line.
fn disciple_ability(pip: crate::mana::ManaSymbol, effect: Effect) -> ActivatedAbility {
    ActivatedAbility { mana_cost: cost(&[pip]), tap_cost: true, effect, ..Default::default() }
}

/// The Sanctuary cycle: an upkeep trigger with a bigger payoff when you
/// control both of the wedge's off-colours.
fn sanctuary(
    name: &'static str,
    c: ManaCost,
    a: Color,
    b_color: Color,
    small: Effect,
    big: Effect,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::Any(vec![controls_color(a), controls_color(b_color)])),
            effect: Effect::If {
                cond: Predicate::All(vec![controls_color(a), controls_color(b_color)]),
                then: Box::new(big),
                else_: Box::new(small),
            },
        }],
        ..enchantment(name, c)
    }
}

/// Ana Disciple — {G} 1/1. Rents out flying, or shaves power.
pub fn ana_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            disciple_ability(
                u(),
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ),
            disciple_ability(b(), pump_target(-2, 0)),
        ],
        ..creature(
            "Ana Disciple",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Ceta Disciple — {U} 1/1. Pumps, or fixes.
pub fn ceta_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            disciple_ability(r(), pump_target(2, 0)),
            disciple_ability(
                g(),
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ),
        ],
        ..creature(
            "Ceta Disciple",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Dega Disciple — {W} 1/1. Shaves power, or lends it.
pub fn dega_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            disciple_ability(b(), pump_target(-2, 0)),
            disciple_ability(r(), pump_target(2, 0)),
        ],
        ..creature(
            "Dega Disciple",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Ana Sanctuary — {2}{G}. +1/+1, or +5/+5 with both off-colours out.
pub fn ana_sanctuary() -> CardDefinition {
    sanctuary(
        "Ana Sanctuary",
        cost(&[generic(2), g()]),
        Color::Blue,
        Color::Black,
        pump_target(1, 1),
        pump_target(5, 5),
    )
}

/// Ceta Sanctuary — {2}{U}. Loots, or digs two deep.
pub fn ceta_sanctuary() -> CardDefinition {
    sanctuary(
        "Ceta Sanctuary",
        cost(&[generic(2), u()]),
        Color::Red,
        Color::Green,
        Effect::Seq(vec![draw(1), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
        Effect::Seq(vec![draw(2), Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }]),
    )
}

/// Dega Sanctuary — {2}{W}. Two life, or four.
pub fn dega_sanctuary() -> CardDefinition {
    sanctuary(
        "Dega Sanctuary",
        cost(&[generic(2), w()]),
        Color::Black,
        Color::Red,
        Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
    )
}

/// Angelfire Crusader — {3}{W} 2/3 with red firebreathing.
pub fn angelfire_crusader() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Angelfire Crusader",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Knight],
            2,
            3,
        )
    }
}

/// A Bloodfire body: `{R}, Sacrifice this: it deals N damage to [filter]`.
fn bloodfire(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_cost: true,
            effect,
            ..Default::default()
        }],
        ..creature(name, c, types, p, t)
    }
}

/// Bloodfire Dwarf — {R} 1/1. A one-point ground sweeper.
pub fn bloodfire_dwarf() -> CardDefinition {
    bloodfire(
        "Bloodfire Dwarf",
        cost(&[r()]),
        vec![CreatureType::Dwarf],
        1,
        1,
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasKeyword(
                Keyword::Flying,
            ))))),
            amount: Value::ONE,
        },
    )
}

/// Bloodfire Kavu — {2}{R}{R} 2/2. A two-point sweeper.
pub fn bloodfire_kavu() -> CardDefinition {
    bloodfire(
        "Bloodfire Kavu",
        cost(&[generic(2), r(), r()]),
        vec![CreatureType::Kavu],
        2,
        2,
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature),
            amount: Value::Const(2),
        },
    )
}

/// Bloodfire Colossus — {6}{R}{R} 6/6. Six to everything and everyone.
pub fn bloodfire_colossus() -> CardDefinition {
    bloodfire(
        "Bloodfire Colossus",
        cost(&[generic(6), r(), r()]),
        vec![CreatureType::Giant],
        6,
        6,
        Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::Const(6),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(6),
            },
        ]),
    )
}

/// Bloodfire Infusion — {2}{R}. The host's power becomes a board sweep.
pub fn bloodfire_infusion() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[r()]),
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::SacrificedPower,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Bloodfire Infusion", cost(&[generic(2), r()]))
    }
}

/// Coastal Drake — {2}{U} 2/1 flier that answers Kavu.
pub fn coastal_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::HasCreatureType(CreatureType::Kavu)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..creature("Coastal Drake", cost(&[generic(2), u()]), vec![CreatureType::Drake], 2, 1)
    }
}

/// Consume Strength — {1}{B}{G}. Moves two points across the table.
pub fn consume_strength() -> CardDefinition {
    instant(
        "Consume Strength",
        cost(&[generic(1), b(), g()]),
        Effect::Seq(vec![
            pump_target(2, 2),
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Death Grasp — {X}{W}{B}. X damage, X life.
pub fn death_grasp() -> CardDefinition {
    sorcery(
        "Death Grasp",
        cost(&[x(), w(), b()]),
        Effect::Seq(vec![
            Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
            Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
        ]),
    )
}

/// Divine Light — {W}. A fog for your side of the board.
pub fn divine_light() -> CardDefinition {
    sorcery(
        "Divine Light",
        cost(&[w()]),
        Effect::PreventAllDamageThisTurn {
            target: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            redirect_to: None,
        },
    )
}

/// Dwarven Landslide — {3}{R}. A land, or two when kicked.
pub fn dwarven_landslide() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), r()]))],
        kicker_action_cost: Some(crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        ..sorcery(
            "Dwarven Landslide",
            cost(&[generic(3), r()]),
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Land) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Destroy {
                        what: Selector::TargetFiltered { slot: 1, filter: R::Land },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Dwarven Patrol — {2}{R} 4/2 that only unlocks off your off-colour spells.
pub fn dwarven_patrol() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Red),
                })),
            ),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature("Dwarven Patrol", cost(&[generic(2), r()]), vec![CreatureType::Dwarf], 4, 2)
    }
}

/// Ebony Treefolk — {1}{B}{G} 3/3 that pumps off both its off-colours.
pub fn ebony_treefolk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Ebony Treefolk",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Treefolk],
            3,
            3,
        )
    }
}

/// A "reveal the top four, keep the matching ones" ETB (the APC tribal cycle).
fn reveal_four_for(filter: R) -> TriggeredAbility {
    etb(Effect::RevealTopTakeMatchingToHand {
        who: PlayerRef::You,
        count: Value::Const(4),
        filter,
    })
}

/// Enlistment Officer — {3}{W} 2/3 first strike that digs for Soldiers.
pub fn enlistment_officer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![reveal_four_for(R::HasCreatureType(CreatureType::Soldier))],
        ..creature(
            "Enlistment Officer",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Kavu Howler — {4}{G}{G} 4/5 that digs for Kavu.
pub fn kavu_howler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![reveal_four_for(R::HasCreatureType(CreatureType::Kavu))],
        ..creature("Kavu Howler", cost(&[generic(4), g(), g()]), vec![CreatureType::Kavu], 4, 5)
    }
}

/// Grave Defiler — {3}{B} 2/1 that digs for Zombies and regenerates.
pub fn grave_defiler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![reveal_four_for(R::HasCreatureType(CreatureType::Zombie))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Grave Defiler", cost(&[generic(3), b()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Evasive Action — {1}{U}. A Domain-scaled Mana Leak.
pub fn evasive_action() -> CardDefinition {
    instant(
        "Evasive Action",
        cost(&[generic(1), u()]),
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: ManaCost::default(),
            exile: false,
            extra_generic: Some(Value::DomainCount(PlayerRef::You)),
        },
    )
}

/// Glade Gnarr — {5}{G} 4/4 that punishes blue.
pub fn glade_gnarr() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasColor(Color::Blue),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Glade Gnarr", cost(&[generic(5), g()]), vec![CreatureType::Beast], 4, 4)
    }
}

/// Goblin Legionnaire — {R}{W} 2/2. Two damage, or a two-point shield.
pub fn goblin_legionnaire() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                sac_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                sac_cost: true,
                effect: Effect::PreventNextDamage {
                    target: target_any(),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Goblin Legionnaire",
            cost(&[r(), w()]),
            vec![CreatureType::Goblin, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Haunted Angel — {2}{W} 3/3 flier that arms everyone else on the way out.
pub fn haunted_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Exile { what: Selector::This },
                Effect::CreateToken {
                    who: PlayerRef::EachOpponent,
                    count: Value::ONE,
                    definition: crate::card::TokenDefinition {
                        name: "Angel".to_string(),
                        power: 3,
                        toughness: 3,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Black],
                        keywords: vec![Keyword::Flying],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Angel],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
            ]),
        }],
        ..creature("Haunted Angel", cost(&[generic(2), w()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Helionaut — {2}{W} 1/2 flier that also fixes.
pub fn helionaut() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Helionaut",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Jilt — {1}{U}. A bounce that also burns when kicked.
pub fn jilt() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), r()]))],
        ..instant(
            "Jilt",
            cost(&[generic(1), u()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Jungle Barrier — {2}{G}{U} 2/6 defender that replaces itself.
pub fn jungle_barrier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(draw(1))],
        ..creature(
            "Jungle Barrier",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Plant, CreatureType::Wall],
            2,
            6,
        )
    }
}

/// Kavu Glider — {2}{R} 2/1 that rents toughness and flying.
pub fn kavu_glider() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ZERO,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Kavu Glider", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 1)
    }
}

/// Kavu Mauler — {4}{G}{G} 4/4 trample that scales with the Kavu horde.
pub fn kavu_mauler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Kavu).and(R::IsAttacking),
                    )),
                    filter: R::Not(Box::new(R::IsSource)),
                },
                toughness: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Kavu).and(R::IsAttacking),
                    )),
                    filter: R::Not(Box::new(R::IsSource)),
                },
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Kavu Mauler", cost(&[generic(4), g(), g()]), vec![CreatureType::Kavu], 4, 4)
    }
}

/// Diversionary Tactics — {3}{W}. Two of your creatures tap one of theirs.
pub fn diversionary_tactics() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature.and(R::ControlledByYou), 2)),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..enchantment("Diversionary Tactics", cost(&[generic(3), w()]))
    }
}

/// Foul Presence — {2}{B}. The host shrinks, and shrinks others.
pub fn foul_presence() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: -1,
            toughness: -1,
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Foul Presence", cost(&[generic(2), b()]))
    }
}

/// Flowstone Charger — {2}{R}{W} 2/5 that trades toughness for reach on the swing.
pub fn flowstone_charger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Flowstone Charger",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Beast],
            2,
            5,
        )
    }
}

/// Gerrard's Verdict — {W}{B}. Two cards off the top of their hand, and life
/// for every land among them.
pub fn gerrards_verdict() -> CardDefinition {
    sorcery(
        "Gerrard's Verdict",
        cost(&[w(), b()]),
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::CountOf(Box::new(Selector::DiscardedThisResolution {
                        filter: R::Land,
                    }))),
                    Box::new(Value::Const(3)),
                ),
            },
        ]),
    )
}

/// Dodecapod — {4} 3/3. Discarding it to an opponent's effect deploys it
/// bigger instead.
pub fn dodecapod() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        opponent_discard_deploys: Some((CounterType::PlusOnePlusOne, 2)),
        ..creature("Dodecapod", cost(&[generic(4)]), vec![CreatureType::Golem], 3, 3)
    }
}
