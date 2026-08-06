//! A twenty-ninth wave — Tarkir: Dragonstorm (TDM) staples on existing
//! primitives: enters-with / move-on-death counters, distribute counters +
//! Harmonize, prowess, sac-cost pumps, two-target ETB removal, modal burn,
//! self-bounce firebreathing, behold-proxy value, and "you've cast another
//! spell" cost reduction / copy. Tests in `crabomination/src/tests/recent29.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement, Selector, StaticAbility,
    StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    count, deal, draw, etb, etb_loot, gain_life, mint_treasures, on_attack, on_dies, on_other_dies,
    on_you_attack, prowess, target_filtered,
};
use crate::effect::{LookPick, Duration, LibraryPosition, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, mono_hybrid, r, u, w};

/// Ainok Wayfarer — {1}{G} 1/1 Dog Scout. ETB: mill three, you may take a land
/// among them. (The "+1/+1 counter if you take no land" rider is dropped.)
pub fn ainok_wayfarer() -> CardDefinition {
    CardDefinition {
        name: "Ainok Wayfarer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(3),
            rest_to_graveyard: true,
            pick_filter: Some(SelectionRequirement::Land),
    ..Default::default()
})))],
        ..Default::default()
    }
}

/// Delta Bloodflies — {1}{B} 1/2 Insect with flying. Whenever it attacks, if
/// you control a creature with a counter on it, each opponent loses 1 life.
pub fn delta_bloodflies() -> CardDefinition {
    CardDefinition {
        name: "Delta Bloodflies",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithAnyCounter),
                ),
                n: Value::ONE,
            },
            then: Box::new(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Meticulous Artisan — {3}{R} 3/3 Djinn Artificer with prowess. ETB: create a
/// Treasure token.
pub fn meticulous_artisan() -> CardDefinition {
    CardDefinition {
        name: "Meticulous Artisan",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Artificer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![prowess(), etb(mint_treasures(1))],
        ..Default::default()
    }
}

/// Iridescent Tiger — {4}{R} 3/4 Cat. ETB: add {W}{U}{B}{R}{G}. (The "if you
/// cast it" gate is dropped — minor.)
pub fn iridescent_tiger() -> CardDefinition {
    CardDefinition {
        name: "Iridescent Tiger",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![
                Color::White,
                Color::Blue,
                Color::Black,
                Color::Red,
                Color::Green,
            ]),
        })],
        ..Default::default()
    }
}

/// Unburied Earthcarver — {1}{B} 2/2 Human Warrior. {2}, Sacrifice another
/// creature: Put a +1/+1 counter on this creature.
pub fn unburied_earthcarver() -> CardDefinition {
    CardDefinition {
        name: "Unburied Earthcarver",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unrooted Ancestor — {2}{B} 3/2 Spirit Cleric with flash. {1}, Sacrifice
/// another creature: This creature gains indestructible until end of turn. Tap
/// it.
pub fn unrooted_ancestor() -> CardDefinition {
    CardDefinition {
        name: "Unrooted Ancestor",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap {
                    what: Selector::This,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gurmag Rakshasa — {4}{B}{B} 5/5 Demon with menace. ETB: target creature an
/// opponent controls gets -2/-2 and target creature you control gets +2/+2.
pub fn gurmag_rakshasa() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Rakshasa",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Fleeting Effigy — {R} 2/2 Elemental with haste. At the beginning of your end
/// step, return it to its owner's hand. {2}{R}: It gets +2/+0 until end of turn.
pub fn fleeting_effigy() -> CardDefinition {
    CardDefinition {
        name: "Fleeting Effigy",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Host of the Hereafter — {2}{B}{G} 2/2 Zombie Warlock. Enters with two +1/+1
/// counters. Whenever it or another creature you control dies, if it had
/// counters, move its counters onto up to one target creature you control.
pub fn host_of_the_hereafter() -> CardDefinition {
    let move_counters = || Effect::MoveAllCounters {
        from: Selector::TriggerSource,
        to: target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
    };
    CardDefinition {
        name: "Host of the Hereafter",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![on_dies(move_counters()), on_other_dies(move_counters())],
        ..Default::default()
    }
}

/// Overwhelming Surge — {2}{R} Instant. Choose one or both — deal 3 damage to
/// target creature; and/or destroy target noncreature artifact.
pub fn overwhelming_surge() -> CardDefinition {
    CardDefinition {
        name: "Overwhelming Surge",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                deal(
                    3,
                    Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature,
                    },
                ),
                Effect::Destroy {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Artifact
                            .and(SelectionRequirement::Creature.negate()),
                    },
                },
            ],
        },
        ..Default::default()
    }
}

/// Focus the Mind — {4}{U} Instant. Costs {2} less if you've cast another spell
/// this turn. Draw three cards, then discard a card.
pub fn focus_the_mind() -> CardDefinition {
    CardDefinition {
        name: "Focus the Mind",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "Costs {2} less if you've cast another spell this turn.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::ONE,
                },
                amount: 2,
            },
        }],
        effect: Effect::Seq(vec![
            draw(3),
            Effect::Discard {
                who: Selector::You,
                amount: Value::ONE,
                random: false,
            },
        ]),
        ..Default::default()
    }
}

/// Sage of the Skies — {2}{W} 2/3 Human Monk with flying and lifelink. When you
/// cast this spell, if you've cast another spell this turn, copy it.
pub fn sage_of_the_skies() -> CardDefinition {
    CardDefinition {
        name: "Sage of the Skies",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::CopySpell {
                    what: Selector::This,
                    count: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Marshal of the Lost — {2}{W}{B} 3/3 Orc Warrior with deathtouch. Whenever
/// you attack, target creature gets +X/+X where X is the number of attackers.
pub fn marshal_of_the_lost() -> CardDefinition {
    CardDefinition {
        name: "Marshal of the Lost",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_you_attack(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: count(Selector::EachPermanent(SelectionRequirement::IsAttacking)),
            toughness: count(Selector::EachPermanent(SelectionRequirement::IsAttacking)),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Embermouth Sentinel — {2} 2/1 Chimera artifact creature. ETB: search your
/// library for a basic land and put it on top. (The "onto the battlefield if
/// you control a Dragon" branch is dropped.)
pub fn embermouth_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Embermouth Sentinel",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Chimera],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
        })],
        ..Default::default()
    }
}

/// Rainveil Rejuvenator — {3}{G} 2/4 Elephant Druid. ETB: you may mill three.
/// {T}: Add {G} for each point of this creature's power.
pub fn rainveil_rejuvenator() -> CardDefinition {
    CardDefinition {
        name: "Rainveil Rejuvenator",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Mill {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Green, Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Synchronized Charge — {1}{G} Sorcery. Distribute two +1/+1 counters among
/// one or two target creatures you control. (The until-EOT vigilance/trample
/// rider on counter-bearing creatures is dropped.) Harmonize {4}{G}.
pub fn synchronized_charge() -> CardDefinition {
    CardDefinition {
        name: "Synchronized Charge",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Harmonize(cost(&[generic(4), g()]))],
        effect: Effect::DistributeCounters {
            total: Value::Const(2),
            counter: CounterType::PlusOnePlusOne,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            max_targets: 2,
        },
        ..Default::default()
    }
}

/// Tersa Lightshatter — {2}{R} 3/3 Orc Wizard with haste. ETB: draw a card,
/// then discard a card. (The graveyard-7 attack rider is dropped.)
pub fn tersa_lightshatter() -> CardDefinition {
    CardDefinition {
        name: "Tersa Lightshatter",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![etb_loot()],
        ..Default::default()
    }
}

/// Watcher of the Wayside — {3} 3/2 Golem artifact creature. ETB: each opponent
/// mills two cards. You gain 2 life. ("target player" → each opponent, 2P.)
pub fn watcher_of_the_wayside() -> CardDefinition {
    CardDefinition {
        name: "Watcher of the Wayside",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            gain_life(2),
        ]))],
        ..Default::default()
    }
}

/// Temur Tawnyback — {2/G}{2/U}{2/R} 4/3 Beast. ETB: draw a card, then discard
/// a card.
pub fn temur_tawnyback() -> CardDefinition {
    CardDefinition {
        name: "Temur Tawnyback",
        cost: cost(&[
            mono_hybrid(2, Color::Green),
            mono_hybrid(2, Color::Blue),
            mono_hybrid(2, Color::Red),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb_loot()],
        ..Default::default()
    }
}

/// Teeming Dragonstorm — {3}{W} Enchantment. ETB: create two 2/2 white Soldier
/// tokens. Whenever a Dragon you control enters, return this to its owner's
/// hand.
pub fn teeming_dragonstorm() -> CardDefinition {
    use crate::card::TokenDefinition;
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Teeming Dragonstorm",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: soldier,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
            },
        ],
        ..Default::default()
    }
}
