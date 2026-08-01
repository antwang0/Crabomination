//! Nemesis (NMS), second wave. Tests in `classic_sets/nms2`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest,
    shortcut::{target_any, target_filtered},
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

fn spellshaper(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    ability_cost: ManaCost,
    effect: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ability_cost,
            discard_cost: Some((R::Any, 1)),
            effect,
            ..Default::default()
        }],
        ..creature(name, c, types, 1, 1)
    }
}

fn controls_land(who_is_you: bool, land: LandType) -> Predicate {
    let ctrl = if who_is_you { R::ControlledByYou } else { R::ControlledByOpponent };
    Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::HasLandType(land).and(ctrl)),
        n: Value::ONE,
    }
}

/// "Whenever this becomes blocked, you may have it deal damage equal to its
/// power to target creature. If you do, it assigns no combat damage this turn."
fn laccolith(name: &'static str, c: ManaCost, types: Vec<CreatureType>, p: i32, t: i32)
-> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::MayDealPowerThenNoCombatDamage {
                dealer: Selector::This,
                to: target_filtered(R::Creature),
            },
        }],
        ..creature(name, c, types, p, t)
    }
}

pub fn laccolith_whelp() -> CardDefinition {
    laccolith("Laccolith Whelp", cost(&[r()]), vec![CreatureType::Beast], 1, 1)
}

pub fn laccolith_grunt() -> CardDefinition {
    laccolith("Laccolith Grunt", cost(&[generic(2), r()]), vec![CreatureType::Beast], 2, 2)
}

pub fn laccolith_warrior() -> CardDefinition {
    laccolith(
        "Laccolith Warrior",
        cost(&[generic(2), r(), r()]),
        vec![CreatureType::Beast, CreatureType::Warrior],
        3,
        3,
    )
}

pub fn laccolith_titan() -> CardDefinition {
    laccolith("Laccolith Titan", cost(&[generic(5), r(), r()]), vec![CreatureType::Beast], 6, 6)
}

/// Laccolith Rig — {R} Aura. Bolts the Laccolith trigger onto anything.
pub fn laccolith_rig() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::EnchantedBySource),
            effect: Effect::MayDealPowerThenNoCombatDamage {
                dealer: Selector::AttachedTo(Box::new(Selector::This)),
                to: target_filtered(R::Creature),
            },
        }],
        ..enchantment("Laccolith Rig", cost(&[r()]))
    }
}

/// Oraxid — {3}{U}. A 2/3 wall of protection from red.
pub fn oraxid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        ..creature(
            "Oraxid",
            cost(&[generic(3), u()]),
            vec![CreatureType::Crab, CreatureType::Beast],
            2,
            3,
        )
    }
}

/// Sneaky Homunculus — {1}{U}. Only smaller creatures interact with it.
pub fn sneaky_homunculus() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantBlockPowerAtLeast(2),
            Keyword::CantBeBlockedByPowerAtLeast(2),
        ],
        ..creature(
            "Sneaky Homunculus",
            cost(&[generic(1), u()]),
            vec![CreatureType::Homunculus, CreatureType::Illusion],
            1,
            1,
        )
    }
}

/// Animate Land — {G}. A land becomes a 3/3 for the turn.
pub fn animate_land() -> CardDefinition {
    instant(
        "Animate Land",
        cost(&[g()]),
        Effect::BecomeCreature {
            what: target_filtered(R::Land),
            power: Value::Const(3),
            toughness: Value::Const(3),
            creature_types: vec![CreatureType::Elemental],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
    )
}

/// Arc Mage — {2}{R} Spellshaper. Two damage, split as you like.
pub fn arc_mage() -> CardDefinition {
    spellshaper(
        "Arc Mage",
        cost(&[generic(2), r()]),
        vec![CreatureType::Human, CreatureType::Spellshaper],
        cost(&[generic(2), r()]),
        Effect::DealDamageDivided {
            total: Value::Const(2),
            filter: R::Any,
            max_targets: 2,
            retaliate_to_source: false,
        },
    )
}

/// Defender en-Vec — {3}{W}. Fading 4 spent two damage at a time.
pub fn defender_en_vec() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fading(4)],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Fade, 1)),
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Defender en-Vec",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            4,
        )
    }
}

/// Fanatical Devotion — {2}{W}. Creatures buy each other's regenerations.
pub fn fanatical_devotion() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..enchantment("Fanatical Devotion", cost(&[generic(2), w()]))
    }
}

/// Massacre — {2}{B}{B}. Free when they have a Plains and you have a Swamp.
pub fn massacre() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            condition: Some(Predicate::All(vec![
                controls_land(false, LandType::Plains),
                controls_land(true, LandType::Swamp),
            ])),
            ..Default::default()
        }),
        ..sorcery(
            "Massacre",
            cost(&[generic(2), b(), b()]),
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Mind Swords — {1}{B}. Everyone loses two cards; a Swamp lets you pay in
/// creatures.
pub fn mind_swords() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            sacrifice_permanents: Some((R::Creature, 1)),
            condition: Some(controls_land(true, LandType::Swamp)),
            ..Default::default()
        }),
        ..sorcery(
            "Mind Swords",
            cost(&[generic(1), b()]),
            Effect::ExileFromHand {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(2),
            },
        )
    }
}

/// Nesting Wurm — {4}{G}{G}. Digs up its own siblings.
pub fn nesting_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::etb(search_same_name("Nesting Wurm"))],
        ..creature("Nesting Wurm", cost(&[generic(4), g(), g()]), vec![CreatureType::Wurm], 4, 3)
    }
}

/// Skyshroud Sentinel — {2}{G}. The Elf half of the same "find three" shape.
pub fn skyshroud_sentinel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(search_same_name(
            "Skyshroud Sentinel",
        ))],
        ..creature("Skyshroud Sentinel", cost(&[generic(2), g()]), vec![CreatureType::Elf], 1, 1)
    }
}

fn search_same_name(name: &str) -> Effect {
    Effect::SearchUpToN {
        who: PlayerRef::You,
        filter: R::HasName(name.to_string()),
        to: ZoneDest::Hand(PlayerRef::You),
        count: Value::Const(3),
    }
}

/// Rathi Assassin — {2}{B}{B}. Kills tapped nonblack creatures and chains
/// Mercenaries.
pub fn rathi_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), b(), b()]),
                effect: Effect::Destroy {
                    what: target_filtered(
                        R::Creature
                            .and(R::Tapped)
                            .and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::PermanentCard
                        .and(R::HasCreatureType(CreatureType::Mercenary))
                        .and(R::ManaValueAtMost(3)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Rathi Assassin",
            cost(&[generic(2), b(), b()]),
            vec![
                CreatureType::Phyrexian,
                CreatureType::Zombie,
                CreatureType::Mercenary,
                CreatureType::Assassin,
            ],
            2,
            2,
        )
    }
}

/// Predator, Flagship — {5}. Hands out flight, then shoots it down.
pub fn predator_flagship() -> CardDefinition {
    CardDefinition {
        name: "Predator, Flagship",
        cost: cost(&[generic(5)]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(5)]),
                effect: Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rupture — {2}{R}. A sacrificed creature's power sprays the ground.
pub fn rupture() -> CardDefinition {
    sorcery(
        "Rupture",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::SacrificeAndRemember { who: PlayerRef::You, filter: R::Creature },
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::SacrificedPower,
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::SacrificedPower,
            },
        ]),
    )
}

/// Saproling Cluster — {1}{G}. A discard outlet anyone at the table may use.
pub fn saproling_cluster() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            any_player: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Saproling".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Green],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Saproling],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..enchantment("Saproling Cluster", cost(&[generic(1), g()]))
    }
}

/// Spiritual Asylum — {2}{W}{W}. Total protection, until you swing.
pub fn spiritual_asylum() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures and lands you control have shroud.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.or(R::Land),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Shroud],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::SacrificeSource,
        }],
        ..enchantment("Spiritual Asylum", cost(&[generic(2), w(), w()]))
    }
}

/// Lin Sivvi, Defiant Hero — {1}{W}{W}. The Rebel engine, plus recursion.
pub fn lin_sivvi_defiant_hero() -> CardDefinition {
    CardDefinition {
        supertypes: vec![crate::card::Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[x()]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::PermanentCard
                        .and(R::HasCreatureType(CreatureType::Rebel))
                        .and(R::ManaValueAtMostXFromCost),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::InGraveyard
                            .and(R::HasCreatureType(CreatureType::Rebel)),
                    },
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: LibraryPosition::Bottom,
                    },
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Lin Sivvi, Defiant Hero",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            1,
            3,
        )
    }
}

/// Wild Mammoth — {2}{G}. Always defects to whoever has the biggest board.
pub fn wild_mammoth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::PlayerControlsMostOf {
                    who: PlayerRef::EachPlayer,
                    filter: R::Creature,
                }),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::MostCreatures),
                duration: Duration::Permanent,
            },
        }],
        ..creature("Wild Mammoth", cost(&[generic(2), g()]), vec![CreatureType::Elephant], 3, 4)
    }
}
