//! Commander: 2026-10-19 (`01DJHkWk`) — four-gap closers: Storm, Shaker of
//! Skies (Bulette, Ironhoof Ox, Zodiac Ox, Vivien's Invocation); Ezekiel
//! Sims, Spider-Totem (Caldaia Strongarm, Gloomwidow, Gloomwidow's Feast,
//! Blizzard Brawl); Rex, Cyber-Hound (Aether Theorist, Electrostatic
//! Pummeler, Minister of Inquiries, Consulate Surveillance).
//! Tests in `tests/recent_b/cmdr_01dj_b.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w};
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

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn energy(n: i32) -> Effect {
    Effect::AddEnergy(Value::Const(n))
}

// ── Storm, Shaker of Skies ──────────────────────────────────────────────────

/// Bulette — {3}{G} 3/3 Beast. At the beginning of your end step, if a
/// creature died this turn, a +1/+1 counter on it.
pub fn bulette() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::CreatureDiedThisTurnMatching { filter: R::Creature }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Bulette", cost(&[generic(3), g()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Ironhoof Ox — {3}{G}{G} 4/4 Ox; can't be blocked by more than one
/// creature.
pub fn ironhoof_ox() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        ..creature("Ironhoof Ox", cost(&[generic(3), g(), g()]), vec![CreatureType::Ox], 4, 4)
    }
}

/// Zodiac Ox — {3}{G} 3/3 Ox; swampwalk.
pub fn zodiac_ox() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature("Zodiac Ox", cost(&[generic(3), g()]), vec![CreatureType::Ox], 3, 3)
    }
}

/// Vivien's Invocation — {5}{G}{G} sorcery. Look at the top seven; you may put
/// a creature card from among them onto the battlefield; the rest go to the
/// bottom in a random order. When a creature is put onto the battlefield this
/// way, it deals damage equal to its power to target creature an opponent
/// controls.
pub fn viviens_invocation() -> CardDefinition {
    spell(
        "Vivien's Invocation",
        cost(&[generic(5), g(), g()]),
        false,
        Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::Const(7),
            filter: R::Creature,
            then: Some(Box::new(Effect::Reflexive {
                body: Box::new(Effect::DealDamageFrom {
                    source: Selector::LastMoved,
                    to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    amount: Value::PowerOf(Box::new(Selector::LastMoved)),
                }),
            })),
            max: Some(1),
            tapped: false,
            exile_rest: false,
            rest_to_graveyard: false,
            mandatory: false,
        },
    )
}

// ── Ezekiel Sims, Spider-Totem ──────────────────────────────────────────────

/// Caldaia Strongarm — {4}{G} 2/3 Human Warrior. Enters: two +1/+1 counters
/// on target creature. Blitz {3}{G}.
pub fn caldaia_strongarm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), g()]),
            blitz: true,
            ..Default::default()
        }),
        ..creature(
            "Caldaia Strongarm",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// Gloomwidow — {2}{G} 3/3 reach Spider; can block only creatures with
/// flying.
pub fn gloomwidow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::CanBlockOnlyFlying],
        ..creature("Gloomwidow", cost(&[generic(2), g()]), vec![CreatureType::Spider], 3, 3)
    }
}

/// Gloomwidow's Feast — {3}{G} instant. Destroy target creature with flying.
/// If it was blue or black, create a 1/2 green Spider token with reach.
pub fn gloomwidows_feast() -> CardDefinition {
    let spider = TokenDefinition {
        name: "Spider".into(),
        power: 1,
        toughness: 2,
        colors: vec![Color::Green],
        keywords: vec![Keyword::Reach],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        ..Default::default()
    };
    spell(
        "Gloomwidow's Feast",
        cost(&[generic(3), g()]),
        true,
        Effect::Seq(vec![
            // The colour is read before the destroy moves the creature.
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
                },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(spider),
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
            },
        ]),
    )
}

/// Blizzard Brawl — {G} snow sorcery. Target creature you control and target
/// creature you don't control; with three or more snow permanents, yours gets
/// +1/+0 and indestructible until end of turn. Then they fight.
pub fn blizzard_brawl() -> CardDefinition {
    let mine = Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature.and(R::ControlledByYou),
    };
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        ..spell(
            "Blizzard Brawl",
            cost(&[g()]),
            false,
            Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::HasSupertype(Supertype::Snow).and(R::ControlledByYou)),
                        n: Value::Const(3),
                    },
                    then: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: mine.clone(),
                            power: Value::ONE,
                            toughness: Value::Const(0),
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: mine.clone(),
                            keyword: Keyword::Indestructible,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Fight {
                    attacker: mine,
                    defender: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))),
                    },
                },
            ]),
        )
    }
}

// ── Rex, Cyber-Hound ────────────────────────────────────────────────────────

/// Aether Theorist — {1}{U} 1/3 Vedalken Rogue. Enters: {E}{E}{E}. {T}, pay
/// {E}: scry 1.
pub fn aether_theorist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(energy(3))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 1,
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Aether Theorist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Vedalken, CreatureType::Rogue],
            1,
            3,
        )
    }
}

/// Electrostatic Pummeler — {3} 1/1 artifact creature Construct. Enters:
/// {E}{E}{E}. Pay {E}{E}{E}: +X/+X until end of turn, X = its power.
pub fn electrostatic_pummeler() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(energy(3))],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 3,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Electrostatic Pummeler", cost(&[generic(3)]), vec![CreatureType::Construct], 1, 1)
    }
}

/// Minister of Inquiries — {U} 1/2 Vedalken Advisor. Enters: {E}{E}. {T}, pay
/// {E}: target player mills three.
pub fn minister_of_inquiries() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(energy(2))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 1,
            effect: Effect::Mill {
                who: target_filtered(R::Player),
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..creature(
            "Minister of Inquiries",
            cost(&[u()]),
            vec![CreatureType::Vedalken, CreatureType::Advisor],
            1,
            2,
        )
    }
}

/// Consulate Surveillance — {3}{W} enchantment. Enters: {E}{E}{E}{E}. Pay
/// {E}{E}: prevent all damage a source of your choice would deal to you this
/// turn.
pub fn consulate_surveillance() -> CardDefinition {
    CardDefinition {
        name: "Consulate Surveillance",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(energy(4))],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 2,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: true,
                exile_top_per_prevented: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
