//! Planeshift (PLS) — 2001. The Invasion block's middle set: Domain, the Kavu,
//! the Lair lands and the Dragon Charms. Tests in `classic_sets/pls`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility,
    Value,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, target_any, target_filtered},
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

/// Allied Strategies — {4}{U}. Domain draw for the targeted player.
pub fn allied_strategies() -> CardDefinition {
    sorcery(
        "Allied Strategies",
        cost(&[generic(4), u()]),
        Effect::Draw {
            who: Selector::TargetFiltered { slot: 0, filter: R::Player },
            amount: Value::DomainCount(PlayerRef::Target(0)),
        },
    )
}

/// Alpha Kavu — {2}{G} 2/2 that turns another Kavu into a wall.
pub fn alpha_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::HasCreatureType(CreatureType::Kavu)),
                power: Value::Const(-1),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Alpha Kavu", cost(&[generic(2), g()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Amphibious Kavu — {2}{G} 2/2 that swells against blue and black.
pub fn amphibious_kavu() -> CardDefinition {
    let pump = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(3),
        toughness: Value::Const(3),
        duration: Duration::EndOfTurn,
    };
    let blue_or_black = Predicate::SelectorExists(Selector::MatchingAmong {
        inner: Box::new(Selector::BlockingCreatures),
        filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
    });
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource)
                    .with_filter(blue_or_black),
                effect: pump.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                    Predicate::SelectorExists(Selector::MatchingAmong {
                        inner: Box::new(Selector::BlockedAttacker),
                        filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
                    }),
                ),
                effect: pump,
            },
        ],
        ..creature("Amphibious Kavu", cost(&[generic(2), g()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Ancient Spider — {2}{G}{W} 2/5 first strike + reach.
pub fn ancient_spider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Reach],
        ..creature(
            "Ancient Spider",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Spider],
            2,
            5,
        )
    }
}

/// Arctic Merfolk — {1}{U} 1/1. Kicker—Return a creature you control.
pub fn arctic_merfolk() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::ReturnToHand {
            filter: R::Creature,
            count: 1,
        }),
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..creature("Arctic Merfolk", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Aura Blast — {1}{W}. Enchantment removal that replaces itself.
pub fn aura_blast() -> CardDefinition {
    instant(
        "Aura Blast",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            draw(1),
        ]),
    )
}

/// Aurora Griffin — {3}{W} 2/2 flier that whitewashes a permanent.
pub fn aurora_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::BecomeColor {
                what: target_filtered(R::Permanent),
                colors: vec![Color::White],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature("Aurora Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Bog Down — {2}{B}. Kicker—Sacrifice two lands for a third card.
pub fn bog_down() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 2,
        }),
        ..sorcery(
            "Bog Down",
            cost(&[generic(2), b()]),
            Effect::Discard {
                who: Selector::TargetFiltered { slot: 0, filter: R::Player },
                amount: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(3)),
                    else_: Box::new(Value::Const(2)),
                },
                random: false,
            },
        )
    }
}

/// Caldera Kavu — {2}{R} 2/2 that pumps off black and recolours off green.
pub fn caldera_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
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
        ..creature("Caldera Kavu", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Cloud Cover — {2}{W}{U}. Blinks whatever they aim at.
pub fn cloud_cover() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            ),
            effect: Effect::MayDo {
                description: "Return that permanent to its owner's hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                }),
            },
        }],
        ..enchantment("Cloud Cover", cost(&[generic(2), w(), u()]))
    }
}

/// Confound — {1}{U}. Counters the removal aimed at your creature.
pub fn confound() -> CardDefinition {
    instant(
        "Confound",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsCreature)),
            },
            draw(1),
        ]),
    )
}

/// Crosis's Charm — {U}{B}{R}. Bounce, kill, or shatter.
pub fn crosiss_charm() -> CardDefinition {
    instant(
        "Crosis's Charm",
        cost(&[u(), b(), r()]),
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            Effect::Destroy { what: target_filtered(R::Artifact) },
        ]),
    )
}

/// Darigaaz's Charm — {B}{R}{G}. Regrow, burn, or pump.
pub fn darigaazs_charm() -> CardDefinition {
    instant(
        "Darigaaz's Charm",
        cost(&[b(), r(), g()]),
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Daring Leap — {1}{W}{U}. A combat trick with two keywords attached.
pub fn daring_leap() -> CardDefinition {
    instant(
        "Daring Leap",
        cost(&[generic(1), w(), u()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeywords {
                what: target_filtered(R::Creature),
                keywords: vec![Keyword::Flying, Keyword::FirstStrike],
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Dark Suspicions — {2}{B}{B}. Taxes every hand bigger than yours.
pub fn dark_suspicions() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::NonNeg(Box::new(Value::Diff(
                    Box::new(Value::HandSizeOf(PlayerRef::ActivePlayer)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ))),
            },
        }],
        ..enchantment("Dark Suspicions", cost(&[generic(2), b(), b()]))
    }
}

/// Deadapult — {2}{R}. Turns Zombies into burn.
pub fn deadapult() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Zombie), 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..enchantment("Deadapult", cost(&[generic(2), r()]))
    }
}

/// Death Bomb — {3}{B}. A creature for a creature, plus two life.
pub fn death_bomb() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..instant(
            "Death Bomb",
            cost(&[generic(3), b()]),
            Effect::Seq(vec![
                Effect::DestroyNoRegen {
                    what: target_filtered(
                        R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    ),
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                },
            ]),
        )
    }
}

/// Destructive Flow — {B}{R}{G}. Nonbasics stop being free.
pub fn destructive_flow() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::IsNonbasicLand,
            },
        }],
        ..enchantment("Destructive Flow", cost(&[b(), r(), g()]))
    }
}

/// Disciple of Kangee — {2}{W} 2/2 that lends flight and blue.
pub fn disciple_of_kangee() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::BecomeColor {
                    what: target_filtered(R::Creature),
                    colors: vec![Color::Blue],
                    duration: Duration::EndOfTurn,
                    additive: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Disciple of Kangee",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Dominaria's Judgment — {2}{W}. Domain protection for the whole team.
pub fn dominarias_judgment() -> CardDefinition {
    let clause = |land, color| Effect::If {
        cond: Predicate::SelectorExists(Selector::EachPermanent(
            R::HasLandType(land).and(R::ControlledByYou),
        )),
        then: Box::new(Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Protection(color),
            duration: Duration::EndOfTurn,
        }),
        else_: Box::new(Effect::Noop),
    };
    instant(
        "Dominaria's Judgment",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            clause(LandType::Plains, Color::White),
            clause(LandType::Island, Color::Blue),
            clause(LandType::Swamp, Color::Black),
            clause(LandType::Mountain, Color::Red),
            clause(LandType::Forest, Color::Green),
        ]),
    )
}

/// The Lair cycle: a tri-land that bounces a real land to stay on the field.
fn lair(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Lair], ..Default::default() },
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessReturn {
            filter: R::Land.and(R::Not(Box::new(R::HasLandType(LandType::Lair)))),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(colors, Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crosis's Catacombs — the Grixis Lair.
pub fn crosiss_catacombs() -> CardDefinition {
    lair("Crosis's Catacombs", vec![Color::Blue, Color::Black, Color::Red])
}

/// Darigaaz's Caldera — the Jund Lair.
pub fn darigaazs_caldera() -> CardDefinition {
    lair("Darigaaz's Caldera", vec![Color::Black, Color::Red, Color::Green])
}

/// Dromar's Cavern — the Esper Lair.
pub fn dromars_cavern() -> CardDefinition {
    lair("Dromar's Cavern", vec![Color::White, Color::Blue, Color::Black])
}

/// Dromar's Charm — {W}{U}{B}. Life, a counter, or a shrink.
pub fn dromars_charm() -> CardDefinition {
    instant(
        "Dromar's Charm",
        cost(&[w(), u(), b()]),
        Effect::ChooseMode(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
            Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Ertai's Trickery — {U}. Punishes the kicked spell.
pub fn ertais_trickery() -> CardDefinition {
    instant(
        "Ertai's Trickery",
        cost(&[u()]),
        Effect::If {
            cond: Predicate::CastSpellWasKicked,
            then: Box::new(Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack),
            }),
            else_: Box::new(Effect::Noop),
        },
    )
}

/// Ertai, the Corrupted — {2}{W}{U}{B} 3/4 that eats a permanent per counter.
pub fn ertai_the_corrupted() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature.or(R::Enchantment), 1)),
            effect: Effect::CounterSpell { what: target_filtered(R::IsSpellOnStack) },
            ..Default::default()
        }],
        ..creature(
            "Ertai, the Corrupted",
            cost(&[generic(2), w(), u(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Escape Routes — {2}{U}. Rebuys your white and black creatures.
pub fn escape_routes() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasColor(Color::White).or(R::HasColor(Color::Black))),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..enchantment("Escape Routes", cost(&[generic(2), u()]))
    }
}

/// Exotic Disease — {4}{B}. Domain drain.
pub fn exotic_disease() -> CardDefinition {
    sorcery(
        "Exotic Disease",
        cost(&[generic(4), b()]),
        Effect::Drain {
            from: Selector::TargetFiltered { slot: 0, filter: R::Player },
            to: Selector::You,
            amount: Value::DomainCount(PlayerRef::You),
        },
    )
}

/// Fleetfoot Panther — {1}{G}{W} 3/4 flash that rebuys an ETB.
pub fn fleetfoot_panther() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasColor(Color::Green).or(R::HasColor(Color::White))),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Fleetfoot Panther",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Cat],
            3,
            4,
        )
    }
}

/// Gaea's Herald — {1}{G} 1/1. Creature spells resolve.
pub fn gaeas_herald() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature spells can't be countered.",
            effect: StaticEffect::CreatureSpellsCantBeCountered,
        }],
        ..creature("Gaea's Herald", cost(&[generic(1), g()]), vec![CreatureType::Elf], 1, 1)
    }
}

/// Gaea's Might — {G}. Domain pump.
pub fn gaeas_might() -> CardDefinition {
    instant(
        "Gaea's Might",
        cost(&[g()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::DomainCount(PlayerRef::You),
            toughness: Value::DomainCount(PlayerRef::You),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Gainsay — {1}{U}. The blue mirror-breaker.
pub fn gainsay() -> CardDefinition {
    instant(
        "Gainsay",
        cost(&[generic(1), u()]),
        Effect::CounterSpell {
            what: target_filtered(R::IsSpellOnStack.and(R::HasColor(Color::Blue))),
        },
    )
}

/// Gerrard's Command — {G}{W}. Untap and pump.
pub fn gerrards_command() -> CardDefinition {
    instant(
        "Gerrard's Command",
        cost(&[g(), w()]),
        Effect::Seq(vec![
            Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Hobble — {2}{W} Aura. Pins a creature down and cantrips.
pub fn hobble() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(draw(1))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::CantAttack],
            ..Default::default()
        }),
        ..enchantment("Hobble", cost(&[generic(2), w()]))
    }
}

/// Dralnu's Crusade — {1}{B}{R}. Every Goblin is a bigger black Zombie.
pub fn dralnus_crusade() -> CardDefinition {
    let goblins = || Selector::EachPermanent(R::HasCreatureType(CreatureType::Goblin));
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "All Goblins get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: goblins(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "All Goblins are black.",
                effect: StaticEffect::SetColorOfMatching {
                    applies_to: goblins(),
                    color: Color::Black,
                },
            },
            StaticAbility {
                description: "All Goblins are Zombies in addition to their other types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: goblins(),
                    creature_type: CreatureType::Zombie,
                },
            },
        ],
        ..enchantment("Dralnu's Crusade", cost(&[generic(1), b(), r()]))
    }
}

/// Falling Timber — {2}{G}. Kicker—Sacrifice a land for a second fog.
pub fn falling_timber() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        ..instant(
            "Falling Timber",
            cost(&[generic(2), g()]),
            Effect::Seq(vec![
                Effect::PreventCombatDamageByTargetThisTurn { target: target_filtered(R::Creature) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::PreventCombatDamageByTargetThisTurn {
                        target: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Honorable Scout — {W} 1/1 that punishes a black-red board.
pub fn honorable_scout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::PermanentCountControlledByMatching(
                    PlayerRef::Target(0),
                    R::Creature.and(R::HasColor(Color::Black).or(R::HasColor(Color::Red))),
                )),
            ),
        })],
        ..creature(
            "Honorable Scout",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Horned Kavu — {R}{G} 3/4 that rebuys an ETB.
pub fn horned_kavu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature("Horned Kavu", cost(&[r(), g()]), vec![CreatureType::Kavu], 3, 4)
    }
}

/// Hull Breach — {R}{G}. One, the other, or both.
pub fn hull_breach() -> CardDefinition {
    sorcery(
        "Hull Breach",
        cost(&[r(), g()]),
        Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy {
                    what: Selector::TargetFiltered { slot: 1, filter: R::Enchantment },
                },
            ]),
        ]),
    )
}

/// Hunting Drake — {4}{U} 2/2 flier that decks a red or green creature.
pub fn hunting_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Creature.and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
            ),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        })],
        ..creature("Hunting Drake", cost(&[generic(4), u()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Implode — {4}{R}. Land destruction that replaces itself.
pub fn implode() -> CardDefinition {
    sorcery(
        "Implode",
        cost(&[generic(4), r()]),
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Land) },
            draw(1),
        ]),
    )
}

/// Insolence — {2}{R} Aura. Tapping it costs its controller two.
pub fn insolence() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::attached_to(
                    Selector::This,
                )))),
                amount: Value::Const(2),
            },
        }],
        ..enchantment("Insolence", cost(&[generic(2), r()]))
    }
}

/// Kavu Recluse — {2}{R} 2/2 that turns a land into a Forest.
pub fn kavu_recluse() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeBasicLand {
                what: target_filtered(R::Land),
                land_type: LandType::Forest,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Kavu Recluse", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Keldon Mantle — {1}{R} Aura. Three colours, three modes.
pub fn keldon_mantle() -> CardDefinition {
    let enchanted = || Selector::attached_to(Selector::This);
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::Regenerate { what: enchanted() },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::PumpPT {
                    what: enchanted(),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                effect: Effect::GrantKeyword {
                    what: enchanted(),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..enchantment("Keldon Mantle", cost(&[generic(1), r()]))
    }
}

/// Lava Zombie — {1}{B}{R} 4/3 that rebuys an ETB and firebreathes.
pub fn lava_zombie() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(
                R::Creature
                    .and(R::ControlledByYou)
                    .and(R::HasColor(Color::Black).or(R::HasColor(Color::Red))),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Lava Zombie", cost(&[generic(1), b(), r()]), vec![CreatureType::Zombie], 4, 3)
    }
}

/// Maggot Carrier — {B} 1/1 that bleeds the table on arrival.
pub fn maggot_carrier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::ONE,
        })],
        ..creature("Maggot Carrier", cost(&[b()]), vec![CreatureType::Zombie], 1, 1)
    }
}
