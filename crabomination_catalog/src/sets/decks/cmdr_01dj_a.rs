//! Commander: 2026-10-19 (`01DJHkWk`) one-deck closers. Tests in
//! `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn loot() -> Effect {
    Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::ONE,
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::ONE,
            random: false,
        },
    ])
}

fn tap_ability(effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect,
        ..Default::default()
    }
}

/// Ghastlord of Fugue — {U/B}x5 4/4 unblockable Spirit Avatar. Its combat
/// damage to a player reveals their hand; you choose a card and they exile it.
pub fn ghastlord_of_fugue() -> CardDefinition {
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        name: "Ghastlord of Fugue",
        cost: cost(&[ub(), ub(), ub(), ub(), ub()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Avatar],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RevealHand {
                    who: PlayerRef::TriggerEventPlayer,
                },
                Effect::MoveChosen {
                    from: Selector::CardsInZone {
                        who: PlayerRef::TriggerEventPlayer,
                        zone: Zone::Hand,
                        filter: R::Any,
                    },
                    filter: None,
                    count: Value::ONE,
                    up_to: false,
                    to: ZoneDest::Exile,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Power of Persuasion — {2}{U} sorcery. Target creature an opponent
/// controls; roll a d20: 1-9 bounce it; 10-19 its owner puts it on top or
/// bottom of their library; 20 you control it until the end of your next turn.
pub fn power_of_persuasion() -> CardDefinition {
    let owner = || PlayerRef::OwnerOf(Box::new(Selector::Target(0)));
    CardDefinition {
        name: "Power of Persuasion",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RollDie {
            sides: 20,
            count: Value::ONE,
            modifier: Value::Const(0),
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![
                (
                    1,
                    9,
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        to: ZoneDest::Hand(owner()),
                    },
                ),
                (
                    10,
                    19,
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Library {
                            who: owner(),
                            pos: LibraryPosition::OwnerChoice,
                        },
                    },
                ),
                (
                    20,
                    20,
                    Effect::GainControl {
                        what: Selector::Target(0),
                        to: None,
                        duration: Duration::UntilEndOfYourNextTurn,
                    },
                ),
            ],
            on_doubles: None,
        },
        ..Default::default()
    }
}

/// Stingerquill Annex — Land. Enters tapped unless you control a
/// planeswalker. {T}: Add {B} or {R}.
pub fn stingerquill_annex() -> CardDefinition {
    super::cmdr_oct17_g::walker_dual("Stingerquill Annex", Color::Black, Color::Red)
}

/// Ominous Asylum — Land. Enters tapped. {T}: Add {B} or {R}. {4}, {T}:
/// Surveil 1.
pub fn ominous_asylum() -> CardDefinition {
    let mut d = crate::sets::dual_land_untyped("Ominous Asylum", Color::Black, Color::Red, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(4)]),
        effect: Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::ONE,
        },
        ..Default::default()
    });
    d
}

/// Devoted Grafkeeper // Departed Soulkeeper — {W}{U} 2/1 Human Peasant.
/// Enters: mill two. Whenever you cast a spell from your graveyard, tap
/// target creature you don't control. Disturb {1}{W}{U}: a 3/1 flying Spirit
/// that can block only fliers.
pub fn devoted_grafkeeper() -> CardDefinition {
    let back = CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        color_indicator: vec![Color::White, Color::Blue],
        ..creature(
            "Departed Soulkeeper",
            ManaCost::default(),
            vec![CreatureType::Spirit],
            3,
            1,
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Disturb(cost(&[generic(1), w(), u()]))],
        triggered_abilities: vec![
            etb(Effect::Mill {
                who: Selector::You,
                amount: Value::Const(2),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::CastSpellFromGraveyard,
                        Predicate::Not(Box::new(Predicate::CastSpellNotOwnedByYou)),
                    ]),
                ),
                effect: Effect::Tap {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                },
            },
        ],
        back_face: Some(Box::new(back)),
        ..creature(
            "Devoted Grafkeeper",
            cost(&[w(), u()]),
            vec![CreatureType::Human, CreatureType::Peasant],
            2,
            1,
        )
    }
}

/// Overwhelmed Archivist // Archive Haunt — {2}{U} 3/2 Human Wizard. Enters:
/// draw, then discard. Disturb {3}{U}: a 2/1 flying Spirit Wizard that loots
/// whenever it attacks.
pub fn overwhelmed_archivist() -> CardDefinition {
    let back = CardDefinition {
        keywords: vec![Keyword::Flying],
        color_indicator: vec![Color::Blue],
        triggered_abilities: vec![on_attack(loot())],
        ..creature(
            "Archive Haunt",
            ManaCost::default(),
            vec![CreatureType::Spirit, CreatureType::Wizard],
            2,
            1,
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Disturb(cost(&[generic(3), u()]))],
        triggered_abilities: vec![etb(loot())],
        back_face: Some(Box::new(back)),
        ..creature(
            "Overwhelmed Archivist",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Alluring Siren — {1}{U} 1/1 Siren. {T}: target creature an opponent
/// controls attacks you this turn if able (CR 508.1d).
pub fn alluring_siren() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tap_ability(Effect::MustAttackPlayerThisTurn {
            attacker: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            defender: Selector::You,
        })],
        ..creature(
            "Alluring Siren",
            cost(&[generic(1), u()]),
            vec![CreatureType::Siren],
            1,
            1,
        )
    }
}

/// Courtly Provocateur — {2}{U} 1/1 Human Wizard. {T}: target creature
/// attacks this turn if able. {T}: target creature blocks this turn if able.
pub fn courtly_provocateur() -> CardDefinition {
    let force = |keyword| {
        tap_ability(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword,
            duration: Duration::EndOfTurn,
        })
    };
    CardDefinition {
        activated_abilities: vec![force(Keyword::MustAttack), force(Keyword::MustBlock)],
        ..creature(
            "Courtly Provocateur",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Minotaur, Roxxon CEO — {4}{B}{B} legendary 4/4 Minotaur Villain. Whenever
/// it or another nontoken creature dies (anyone's), you create a 2/1 black
/// Villain with menace.
pub fn minotaur_roxxon_ceo() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Villain".into(),
                    power: 2,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Villain],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Menace],
                    ..Default::default()
                }),
            },
        }],
        ..creature(
            "Minotaur, Roxxon CEO",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Minotaur, CreatureType::Villain],
            4,
            4,
        )
    }
}

/// Vraska's Final Mercy — {B}{B} sorcery. Choose one: lose 2 life and destroy
/// target creature or planeswalker; or lose 2 life and Empower Jace 6.
pub fn vraskas_final_mercy() -> CardDefinition {
    let lose2 = || Effect::LoseLife {
        who: Selector::You,
        amount: Value::Const(2),
    };
    CardDefinition {
        name: "Vraska's Final Mercy",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                lose2(),
                Effect::Destroy {
                    what: target_filtered(R::Creature.or(R::Planeswalker)),
                },
            ]),
            Effect::Seq(vec![
                lose2(),
                Effect::EmpowerJace {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                },
            ]),
        ]),
        ..Default::default()
    }
}
