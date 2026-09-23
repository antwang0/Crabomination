//! Commander: the cards the **Goblin Storm** Secret Lair deck (Zada, Hedron
//! Grinder) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_zndrsplt.rs` (the Secret Lair module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::catalog::sets::{enters_tapped_unless_you_control, tap_add, tap_add_colorless};
use crate::effect::shortcut::{boast, etb, on_dies, target_any};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, StaticAbility, StaticEffect,
};
use crate::mana::{Color, cost, generic, r};
use std::sync::Arc;

fn goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    }
}

fn goblin(name: &'static str, mana: crate::mana::ManaCost, p: i32, t: i32, extra: Vec<CreatureType>) -> CardDefinition {
    let mut types = vec![CreatureType::Goblin];
    types.extend(extra);
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

fn your_creatures() -> Selector {
    Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature }
}

/// Battle Hymn — {1}{R} Instant. Add {R} for each creature you control.
pub fn battle_hymn() -> CardDefinition {
    CardDefinition {
        name: "Battle Hymn",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColor(Color::Red, Value::count(your_creatures())),
        },
        ..Default::default()
    }
}

/// Broadside Bombardiers — {2}{R} Creature — Goblin Pirate 2/2. Menace, haste.
/// Boast — Sacrifice another creature or artifact: This creature deals damage
/// equal to 2 plus the sacrificed permanent's mana value to any target.
pub fn broadside_bombardiers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature.or(R::Artifact), 1)),
            ..boast(
                crate::mana::ManaCost::default(),
                Effect::DealDamage {
                    to: target_any(),
                    amount: Value::Sum(vec![Value::Const(2), Value::SacrificedManaValue]),
                },
            )
        }],
        ..goblin("Broadside Bombardiers", cost(&[generic(2), r()]), 2, 2, vec![CreatureType::Pirate])
    }
}

/// Castle Embereth — Land. This land enters tapped unless you control a
/// Mountain. {T}: Add {R}. {1}{R}{R}, {T}: Creatures you control get +1/+0
/// until end of turn.
pub fn castle_embereth() -> CardDefinition {
    CardDefinition {
        name: "Castle Embereth",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped_unless_you_control(LandType::Mountain)],
        activated_abilities: vec![
            tap_add(Color::Red),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r(), r()]),
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: your_creatures(),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn hasty_soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Frontline Heroism — {2}{R} Enchantment. When this enters, create a 1/1 red
/// Soldier with haste. Whenever you cast a spell that targets only a single
/// creature you control, create a 1/1 red Soldier with haste, then copy that
/// spell. The copy targets that token.
pub fn frontline_heroism() -> CardDefinition {
    let soldier = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(hasty_soldier()),
    };
    CardDefinition {
        name: "Frontline Heroism",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(soldier()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellTargetsOnlyOneMatching(R::Creature.and(R::ControlledByYou)),
                ),
                effect: Effect::Seq(vec![
                    soldier(),
                    Effect::CopySpellTargeting {
                        what: Selector::TriggerSource,
                        target: Selector::LastCreatedToken,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// General Kreat, the Boltbringer — {2}{R} Legendary Creature — Goblin Soldier
/// 2/2. Whenever one or more Goblins you control attack, create a 1/1 red
/// Goblin creature token that's tapped and attacking. Whenever another
/// creature you control enters, General Kreat deals 1 damage to each opponent.
pub fn general_kreat_the_boltbringer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Goblin),
                    })
                    .once_per_batch(),
                effect: Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(goblin_token()),
                    cleanup: Default::default(),
                    defender: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
            },
        ],
        ..goblin(
            "General Kreat, the Boltbringer",
            cost(&[generic(2), r()]),
            2,
            2,
            vec![CreatureType::Soldier],
        )
    }
}

/// Haze of Rage — {1}{R} Sorcery. Buyback {2}. Creatures you control get +1/+0
/// until end of turn. Storm.
pub fn haze_of_rage() -> CardDefinition {
    CardDefinition {
        name: "Haze of Rage",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(2)])), Keyword::Storm],
        effect: Effect::PumpPT {
            what: your_creatures(),
            power: Value::ONE,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Kher Keep — Legendary Land. {T}: Add {C}. {1}{R}, {T}: Create a 0/1 red
/// Kobold creature token named Kobolds of Kher Keep.
pub fn kher_keep() -> CardDefinition {
    CardDefinition {
        name: "Kher Keep",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), r()]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Kobolds of Kher Keep".into(),
                        power: 0,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Kobold],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rundvelt Hordemaster — {1}{R} Creature — Goblin Warrior 1/1. Other Goblins
/// you control get +1/+1. Whenever this creature or another Goblin you control
/// dies, exile the top card of your library. If it's a Goblin creature card,
/// you may cast that card until the end of your next turn.
pub fn rundvelt_hordemaster() -> CardDefinition {
    let exile_top = Effect::Seq(vec![
        Effect::ExileTopOfLibrary {
            who: Selector::You,
            amount: Value::ONE,
            link_to_source: false,
            face_down: false,
        },
        Effect::If {
            cond: Predicate::EntityMatches {
                what: Selector::LastMoved,
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Goblin)),
            },
            then: Box::new(Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            }),
            else_: Box::new(Effect::Noop),
        },
    ]);
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Goblins you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Goblin)
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![
            on_dies(exile_top.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Goblin),
                    }),
                effect: exile_top,
            },
        ],
        ..goblin("Rundvelt Hordemaster", cost(&[generic(1), r()]), 1, 1, vec![CreatureType::Warrior])
    }
}

/// Siege-Gang Lieutenant — {3}{R} Creature — Goblin 2/2. Lieutenant — At the
/// beginning of combat on your turn, if you control your commander, create two
/// 1/1 red Goblin creature tokens. Those tokens gain haste until end of turn.
/// {2}, Sacrifice a Goblin: This creature deals 1 damage to any target.
///
/// The sacrificed Goblin is another one (`sac_other_filter`); the printed cost
/// admits the Lieutenant itself.
pub fn siege_gang_lieutenant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat),
                EventScope::YourControl,
            )
            .with_filter(Predicate::ControlsOwnCommander { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: Arc::new(goblin_token()),
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedTokens,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..goblin("Siege-Gang Lieutenant", cost(&[generic(3), r()]), 2, 2, vec![])
    }
}

/// Throne of Eldraine — {5} Legendary Artifact. As Throne of Eldraine enters,
/// choose a color. {T}: Add four mana of the chosen color. Spend this mana only
/// to cast monocolored spells of that color. {3}, {T}: Draw two cards. Spend
/// only mana of the chosen color to activate this ability.
///
/// Approximation: the draw ability's "only mana of the chosen color" payment
/// rider is not enforced.
pub fn throne_of_eldraine() -> CardDefinition {
    let pip = || Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::RestrictedToChosenColorMono(Box::new(ManaPayload::ChosenColorOfSource)),
    };
    CardDefinition {
        name: "Throne of Eldraine",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![pip(), pip(), pip(), pip()]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
