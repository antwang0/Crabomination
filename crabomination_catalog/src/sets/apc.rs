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
        distinct_powers: false,
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

/// Necra Disciple — {B} 1/1. Fixes, or a one-point shield.
pub fn necra_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            disciple_ability(
                g(),
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ),
            disciple_ability(
                w(),
                Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
            ),
        ],
        ..creature(
            "Necra Disciple",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Raka Disciple — {R} 1/1. A one-point shield, or rented flying.
pub fn raka_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            disciple_ability(
                w(),
                Effect::PreventNextDamage { target: target_any(), amount: Value::ONE },
            ),
            disciple_ability(
                u(),
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        ..creature(
            "Raka Disciple",
            cost(&[r()]),
            vec![CreatureType::Minotaur, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Necra Sanctuary — {2}{B}. One life, or three.
pub fn necra_sanctuary() -> CardDefinition {
    sanctuary(
        "Necra Sanctuary",
        cost(&[generic(2), b()]),
        Color::Green,
        Color::White,
        Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
        Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) },
    )
}

/// Raka Sanctuary — {2}{R}. One damage, or three.
pub fn raka_sanctuary() -> CardDefinition {
    sanctuary(
        "Raka Sanctuary",
        cost(&[generic(2), r()]),
        Color::White,
        Color::Blue,
        Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
        Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
    )
}

/// Mournful Zombie — {2}{B} 2/1 that rents out a point of life.
pub fn mournful_zombie() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![disciple_ability(
            w(),
            Effect::GainLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
        )],
        ..creature("Mournful Zombie", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Orim's Thunder — {2}{W}. Naturalize that also burns when kicked.
pub fn orims_thunder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[r()]))],
        ..instant(
            "Orim's Thunder",
            cost(&[generic(2), w()]),
            Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment)),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                        amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// A Penumbra body: it leaves a black copy of itself behind.
fn penumbra(
    name: &'static str,
    token_name: &str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crate::card::TokenDefinition {
                    name: token_name.to_string(),
                    power: p,
                    toughness: t,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes { creature_types: types.clone(), ..Default::default() },
                    ..Default::default()
                },
            },
        }],
        ..creature(name, c, types, p, t)
    }
}

/// Penumbra Bobcat — {2}{G} 2/1 that leaves a black 2/1 Cat.
pub fn penumbra_bobcat() -> CardDefinition {
    penumbra("Penumbra Bobcat", "Cat", cost(&[generic(2), g()]), vec![CreatureType::Cat], 2, 1)
}

/// Penumbra Kavu — {4}{G} 3/3 that leaves a black 3/3 Kavu.
pub fn penumbra_kavu() -> CardDefinition {
    penumbra("Penumbra Kavu", "Kavu", cost(&[generic(4), g()]), vec![CreatureType::Kavu], 3, 3)
}

/// Quagmire Druid — {2}{B} 2/2. A creature buys an enchantment kill.
pub fn quagmire_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature(
            "Quagmire Druid",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Quicksilver Dagger — {1}{U}{R}. The host pings and cantrips.
pub fn quicksilver_dagger() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::Target(0)),
                        amount: Value::ONE,
                    },
                    draw(1),
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..enchantment("Quicksilver Dagger", cost(&[generic(1), u(), r()]))
    }
}

/// Razorfin Hunter — {U}{R} 1/1 pinger.
pub fn razorfin_hunter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Razorfin Hunter",
            cost(&[u(), r()]),
            vec![CreatureType::Merfolk, CreatureType::Goblin],
            1,
            1,
        )
    }
}

/// Reef Shaman — {U} 0/2. Retypes a land for a turn.
pub fn reef_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LandsBecomeChosenBasicType {
                what: target_filtered(R::Land),
                duration: Duration::EndOfTurn,
                            from_chosen_basic: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Reef Shaman",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Shaman],
            0,
            2,
        )
    }
}

/// Shimmering Mirage — {1}{U}. Retype a land, then cantrip.
pub fn shimmering_mirage() -> CardDefinition {
    instant(
        "Shimmering Mirage",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::LandsBecomeChosenBasicType {
                what: target_filtered(R::Land),
                duration: Duration::EndOfTurn,
                            from_chosen_basic: false,
            },
            draw(1),
        ]),
    )
}

/// Smash — {2}{R}. Artifact removal that replaces itself.
pub fn smash() -> CardDefinition {
    instant(
        "Smash",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![Effect::Destroy { what: target_filtered(R::Artifact) }, draw(1)]),
    )
}

/// Savage Gorilla — {4}{G} 3/3. Trades itself for a shrink and a card.
pub fn savage_gorilla() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), b()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                draw(1),
            ]),
            ..Default::default()
        }],
        ..creature("Savage Gorilla", cost(&[generic(4), g()]), vec![CreatureType::Ape], 3, 3)
    }
}

/// Shield of Duty and Reason — {W}. Protection from green and blue.
pub fn shield_of_duty_and_reason() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::Protection(Color::Green),
                Keyword::Protection(Color::Blue),
            ],
            ..Default::default()
        }),
        ..enchantment("Shield of Duty and Reason", cost(&[w()]))
    }
}

/// Spectral Lynx — {1}{W} 2/1 pro-green that regenerates for {B}.
pub fn spectral_lynx() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Green)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Spectral Lynx",
            cost(&[generic(1), w()]),
            vec![CreatureType::Cat, CreatureType::Spirit],
            2,
            1,
        )
    }
}

/// Spiritmonger — {3}{B}{G} 6/6 that grows off combat and dodges removal.
pub fn spiritmonger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::BecomeChosenColor {
                    what: Selector::This,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Spiritmonger",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Squee's Embrace — {R}{W}. +2/+2, and the host comes back when it dies.
pub fn squees_embrace() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
            }],
            ..Default::default()
        }),
        ..enchantment("Squee's Embrace", cost(&[r(), w()]))
    }
}

/// Strength of Night — {2}{G}. A team pump, bigger on Zombies when kicked.
pub fn strength_of_night() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[b()]))],
        ..instant(
            "Strength of Night",
            cost(&[generic(2), g()]),
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::PumpPT {
                        what: Selector::EachPermanent(
                            R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou),
                        ),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Suffocating Blast — {1}{U}{U}{R}. A counter plus three damage.
pub fn suffocating_blast() -> CardDefinition {
    instant(
        "Suffocating Blast",
        cost(&[generic(1), u(), u(), r()]),
        Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: Value::Const(3),
            },
        ]),
    )
}

/// Sylvan Messenger — {3}{G} 2/2 trample that digs for Elves.
pub fn sylvan_messenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![reveal_four_for(R::HasCreatureType(CreatureType::Elf))],
        ..creature("Sylvan Messenger", cost(&[generic(3), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

/// Tidal Courier — {3}{U} 1/2 that digs for Merfolk and can fly.
pub fn tidal_courier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![reveal_four_for(R::HasCreatureType(CreatureType::Merfolk))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Tidal Courier", cost(&[generic(3), u()]), vec![CreatureType::Merfolk], 1, 2)
    }
}

/// Temporal Spring — {1}{G}{U}. Puts any permanent on top of its library.
pub fn temporal_spring() -> CardDefinition {
    sorcery(
        "Temporal Spring",
        cost(&[generic(1), g(), u()]),
        Effect::Move {
            what: target_filtered(R::Any),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: crate::effect::LibraryPosition::Top,
            },
        },
    )
}

/// Tranquil Path — {4}{G}. Sweeps enchantments and replaces itself.
pub fn tranquil_path() -> CardDefinition {
    sorcery(
        "Tranquil Path",
        cost(&[generic(4), g()]),
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Enchantment) },
            draw(1),
        ]),
    )
}

/// Tundra Kavu — {2}{R} 2/2. Retypes a land toward Plains or Island.
pub fn tundra_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LandsBecomeChosenBasicType {
                what: target_filtered(R::Land),
                duration: Duration::EndOfTurn,
                            from_chosen_basic: false,
            },
            ..Default::default()
        }],
        ..creature("Tundra Kavu", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Unnatural Selection — {1}{U}. Rewrites a creature's type for a turn.
pub fn unnatural_selection() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::BecomeChosenCreatureType {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
                excluded: vec![],
            },
            ..Default::default()
        }],
        ..enchantment("Unnatural Selection", cost(&[generic(1), u()]))
    }
}

/// Urborg Elf — {1}{G} 1/1 that taps for the Ana wedge.
pub fn urborg_elf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(
                    vec![Color::Black, Color::Green, Color::Blue],
                    Value::ONE,
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Urborg Elf",
            cost(&[generic(1), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Urborg Uprising — {4}{B}. Two creatures back, plus a card.
pub fn urborg_uprising() -> CardDefinition {
    sorcery(
        "Urborg Uprising",
        cost(&[generic(4), b()]),
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::InGraveyard.and(R::Creature),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            draw(1),
        ]),
    )
}

/// Whirlpool Rider — {1}{U} 1/1 that refreshes your hand.
pub fn whirlpool_rider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ShuffleHandsDrawSame { who: PlayerRef::You })],
        ..creature("Whirlpool Rider", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Whirlpool Drake — {3}{U} 2/2 flier that refreshes on the way in and out.
pub fn whirlpool_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::ShuffleHandsDrawSame { who: PlayerRef::You }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::ShuffleHandsDrawSame { who: PlayerRef::You },
            },
        ],
        ..creature("Whirlpool Drake", cost(&[generic(3), u()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Overgrown Estate — {W}{B}{G}. Lands into life.
pub fn overgrown_estate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..enchantment("Overgrown Estate", cost(&[w(), b(), g()]))
    }
}

/// Powerstone Minefield — {2}{R}{W}. Combat costs everyone two.
pub fn powerstone_minefield() -> CardDefinition {
    let bite = Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
                effect: bite.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer),
                effect: bite,
            },
        ],
        ..enchantment("Powerstone Minefield", cost(&[generic(2), r(), w()]))
    }
}

/// A 1/1 green Saproling.
fn saproling() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
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

/// Aether Mutation — {3}{G}{U}. A bounce that pays out in Saprolings.
pub fn aether_mutation() -> CardDefinition {
    sorcery(
        "Aether Mutation",
        cost(&[generic(3), g(), u()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ManaValueOf(Box::new(Selector::Target(0))),
                definition: saproling(),
            },
        ]),
    )
}

/// Death Mutation — {6}{B}{G}. Removal that pays out in Saprolings.
pub fn death_mutation() -> CardDefinition {
    sorcery(
        "Death Mutation",
        cost(&[generic(6), b(), g()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ManaValueOf(Box::new(Selector::Target(0))),
                definition: saproling(),
            },
        ]),
    )
}

/// Desolation Angel — {3}{B}{B} 5/4 flier. Your lands, or everyone's.
pub fn desolation_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[w(), w()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy { what: Selector::EachPermanent(R::Land) }),
            else_: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            }),
        })],
        ..creature("Desolation Angel", cost(&[generic(3), b(), b()]), vec![CreatureType::Angel], 5, 4)
    }
}

/// Desolation Giant — {2}{R}{R} 3/3. Your board, or everyone's.
pub fn desolation_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[w(), w()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::IsSource)))),
            }),
            else_: Box::new(Effect::Destroy {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::Not(Box::new(R::IsSource))),
                ),
            }),
        })],
        ..creature("Desolation Giant", cost(&[generic(2), r(), r()]), vec![CreatureType::Giant], 3, 3)
    }
}

/// Brass Herald — {6} 2/2. Names a type, digs for it, and lords it.
pub fn brass_herald() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        triggered_abilities: vec![etb(Effect::RevealTopTakeMatchingToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            filter: R::Creature.and(R::IsSourceChosenCreatureType),
            distinct_powers: false,
        })],
        static_abilities: vec![StaticAbility {
            description: "Creatures of the chosen type get +1/+1.",
            effect: StaticEffect::AnthemForChosenType {
                power: 1,
                toughness: 1,
                exclude_source: false,
                opponents: false,
                all_players: false,
                per_counter: None,
            },
        }],
        ..creature("Brass Herald", cost(&[generic(6)]), vec![CreatureType::Golem], 2, 2)
    }
}

/// Dragon Arch — {5}. Deploys multicolored creatures straight from hand.
pub fn dragon_arch() -> CardDefinition {
    CardDefinition {
        name: "Dragon Arch",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::Multicolored),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fervent Charge — {1}{R}{W}{B}. Every attack comes in bigger.
pub fn fervent_charge() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..enchantment("Fervent Charge", cost(&[generic(1), r(), w(), b()]))
    }
}

/// Fungal Shambler — {4}{B}{G}{U} 6/4 trample that trades hits for cards.
pub fn fungal_shambler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
            ]),
        }],
        ..creature(
            "Fungal Shambler",
            cost(&[generic(4), b(), g(), u()]),
            vec![CreatureType::Fungus, CreatureType::Beast],
            6,
            4,
        )
    }
}

/// Gerrard Capashen — {3}{W}{W} 3/4. Taxes their hand, and taps blockers.
pub fn gerrard_capashen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::HandSizeOf(PlayerRef::Target(0)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::IsAttacking,
            }),
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Gerrard Capashen",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Goblin Trenches — {1}{R}{W}. Lands into pairs of Goblin Soldiers.
pub fn goblin_trenches() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crate::card::TokenDefinition {
                    name: "Goblin Soldier".to_string(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red, Color::White],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin, CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..enchantment("Goblin Trenches", cost(&[generic(1), r(), w()]))
    }
}

/// Last Caress — {2}{B}. A one-point drain that replaces itself.
pub fn last_caress() -> CardDefinition {
    sorcery(
        "Last Caress",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
            draw(1),
        ]),
    )
}

/// Lightning Angel — {1}{U}{R}{W} 3/4 with all three keywords.
pub fn lightning_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        ..creature(
            "Lightning Angel",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Angel],
            3,
            4,
        )
    }
}
