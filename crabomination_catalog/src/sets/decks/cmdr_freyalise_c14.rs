//! Commander: the cards the **Guided By Nature** precon (C14, Freyalise,
//! Llanowar's Fury) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Siege Behemoth** — every creature you control assigns as though
//!   unblocked while it attacks; the per-creature "you may" is always yes.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, cost, g, generic, x};
use crate::sets::{enters_tapped, tap_add, tap_add_colorless};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn token(name: &str, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn elf_warrior() -> TokenDefinition {
    token("Elf Warrior", vec![CreatureType::Elf, CreatureType::Warrior], 1, 1)
}

fn wolf() -> TokenDefinition {
    token("Wolf", vec![CreatureType::Wolf], 2, 2)
}

fn land(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], ..Default::default() }
}

/// Assault Suit — +2/+2, haste, can't attack you or your planeswalkers, can't
/// be sacrificed; each opponent's upkeep you may lend it to them. Equip {3}.
pub fn assault_suit() -> CardDefinition {
    CardDefinition {
        name: "Assault Suit",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Haste, Keyword::CantBeSacrificed],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature can't attack you or planeswalkers you control.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: true,
                filter: Some(R::IsHostOfSource),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl)
                .with_filter(Predicate::SelectorExists(Selector::AttachedTo(Box::new(
                    Selector::This,
                )))),
            effect: Effect::MayDo {
                description: "Lend the equipped creature to that player until end of turn?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::AttachedTo(Box::new(Selector::This)),
                        to: Some(PlayerRef::ActivePlayer),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap {
                        what: Selector::AttachedTo(Box::new(Selector::This)),
                        up_to: None,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Creeperhulk — {1}{G}: a creature of yours becomes a 5/5 trampler.
pub fn creeperhulk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Seq(vec![
                Effect::SetBasePT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeywords {
                    what: Selector::Target(0),
                    keywords: vec![Keyword::Trample],
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Creeperhulk",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Plant, CreatureType::Elemental],
            5,
            5,
        )
    }
}

/// Drove of Elves — hexproof; */* = green permanents you control.
pub fn drove_of_elves() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::Permanent.and(R::HasColor(Color::Green))),
        }),
        ..creature("Drove of Elves", cost(&[generic(3), g()]), vec![CreatureType::Elf], 0, 0)
    }
}

/// Gargoyle Castle — {C}; {5}, {T}, sacrifice: a 3/4 flying Gargoyle.
pub fn gargoyle_castle() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Gargoyle".into(),
                        power: 3,
                        toughness: 4,
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Gargoyle],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
        ],
        ..land("Gargoyle Castle")
    }
}

/// Grave Sifter — each player names a type and returns every card of it from
/// their graveyard to their hand.
pub fn grave_sifter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::EachPlayerChoosesCreatureTypeThen {
                then: Box::new(Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::IsTypeChosenThisWay,
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
                per_player: true,
            },
        )],
        ..creature(
            "Grave Sifter",
            cost(&[generic(5), g()]),
            vec![CreatureType::Elemental, CreatureType::Beast],
            5,
            7,
        )
    }
}

/// Grim Flowering — draw a card per creature card in your graveyard.
pub fn grim_flowering() -> CardDefinition {
    CardDefinition {
        name: "Grim Flowering",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Creature,
            })),
        },
        ..Default::default()
    }
}

/// Haunted Fengraf — {C}; {3}, {T}, sacrifice: a random creature card from
/// your graveyard to your hand.
pub fn haunted_fengraf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Move {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..land("Haunted Fengraf")
    }
}

/// Havenwood Battleground — enters tapped; {G}, or {T} and sacrifice for {G}{G}.
pub fn havenwood_battleground() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_add(Color::Green),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Green, Color::Green]),
                },
                ..Default::default()
            },
        ],
        ..land("Havenwood Battleground")
    }
}

/// Hunting Triad — three 1/1 Elf Warriors; reinforce 3—{3}{G}.
pub fn hunting_triad() -> CardDefinition {
    CardDefinition {
        name: "Hunting Triad",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Kindred, CardType::Sorcery],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf], ..Default::default() },
        keywords: vec![Keyword::Reinforce(3, cost(&[generic(3), g()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: Arc::new(elf_warrior()),
        },
        ..Default::default()
    }
}

/// Lifeblood Hydra — enters with X counters; when it dies, gain life and
/// draw cards equal to its power.
pub fn lifeblood_hydra() -> CardDefinition {
    let power = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: power() },
            Effect::Draw { who: Selector::You, amount: power() },
        ]))],
        ..creature(
            "Lifeblood Hydra",
            cost(&[x(), g(), g(), g()]),
            vec![CreatureType::Hydra],
            0,
            0,
        )
    }
}

/// Loreseeker's Stone — {3}, {T}: draw three, {1} more per card in your hand.
pub fn loreseekers_stone() -> CardDefinition {
    CardDefinition {
        name: "Loreseeker's Stone",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            mana_cost_increase: Some(Value::HandSizeOf(PlayerRef::You)),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Siege Behemoth — hexproof; while it attacks, your creatures may assign
/// combat damage as though unblocked. ⚠ Always assigns that way.
pub fn siege_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is attacking, each creature you control \
                          may assign its combat damage as though it weren't blocked.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::AssignsDamageAsThoughUnblocked,
                }),
            },
        }],
        ..creature("Siege Behemoth", cost(&[generic(5), g(), g()]), vec![CreatureType::Beast], 7, 4)
    }
}

/// Sylvan Offering — you and a chosen opponent each get an X/X Treefolk; you
/// and a chosen opponent each get X Elf Warriors.
pub fn sylvan_offering() -> CardDefinition {
    let treefolk = || {
        Arc::new(TokenDefinition {
            dynamic_pt: Some((Value::XFromCost, Value::XFromCost)),
            ..token("Treefolk", vec![CreatureType::Treefolk], 0, 0)
        })
    };
    let both = |count: Value, def: fn() -> Arc<TokenDefinition>| {
        Effect::ChooseOpponentThen {
            then: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: count.clone(), definition: def() },
                Effect::CreateToken {
                    who: PlayerRef::ChosenPlayerOfSource,
                    count,
                    definition: def(),
                },
            ])),
        }
    };
    CardDefinition {
        name: "Sylvan Offering",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            both(Value::ONE, treefolk),
            both(Value::XFromCost, || Arc::new(elf_warrior())),
        ]),
        ..Default::default()
    }
}

/// Wave of Vitriol — everyone sacrifices their artifacts, enchantments and
/// nonbasic lands; each may fetch a basic (tapped) per land sacrificed.
pub fn wave_of_vitriol() -> CardDefinition {
    let doomed = || {
        R::Artifact.or(R::Enchantment).or(R::Land.and(R::HasSupertype(crate::card::Supertype::Basic).negate()))
    };
    CardDefinition {
        name: "Wave of Vitriol",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAllMatching {
                who: Selector::Player(PlayerRef::EachPlayer),
                filter: doomed(),
            },
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::SearchUpToN {
                    who: PlayerRef::Triggerer,
                    filter: R::Land.and(R::HasSupertype(crate::card::Supertype::Basic)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::Triggerer, tapped: true },
                    count: Value::SacrificedThisResolutionBy {
                        who: PlayerRef::Triggerer,
                        filter: R::Land,
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Wolfcaller's Howl — each upkeep, a 2/2 Wolf per opponent holding four or
/// more cards.
pub fn wolfcallers_howl() -> CardDefinition {
    CardDefinition {
        name: "Wolfcaller's Howl",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::OpponentsWithHandSizeAtLeast(4),
                definition: Arc::new(wolf()),
            },
        }],
        ..Default::default()
    }
}

/// Wren's Run Packmaster — champion an Elf; {2}{G}: a 2/2 Wolf; your Wolves
/// have deathtouch.
pub fn wrens_run_packmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Champion {
            filter: R::HasCreatureType(CreatureType::Elf),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(wolf()),
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Wolves you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Wolf).and(R::ControlledByYou),
                ),
                keyword: Keyword::Deathtouch,
            },
        }],
        ..creature(
            "Wren's Run Packmaster",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            5,
            5,
        )
    }
}
