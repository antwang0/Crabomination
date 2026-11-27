//! Commander: closers for Khan (Despot by Design) and Kirk (Commanding
//! Officer)'s EDHREC average decks and the Fugitive Doctor // Adric pair
//! (scan of 2026-11-27). Tests in `tests/recent_b/cmdr_01wr6s_a.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Value, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, r, w, x};

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn damage(to: Selector, n: i32) -> Effect {
    Effect::DealDamage { to, amount: Value::Const(n) }
}

/// Khan, Despot by Design — {4}{R} legendary 5/4 Human Rogue, trample. ETB:
/// 1 damage to each opponent and each creature your opponents control.
pub fn khan_despot_by_design() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            damage(Selector::Player(PlayerRef::EachOpponent), 1),
            damage(Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)), 1),
        ]))],
        ..creature(
            "Khan, Despot by Design",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            5,
            4,
        )
    }
}

/// Spatial Anomaly — land. {T}: {C}. {4}{R}, {T}, sacrifice it: 2 damage to
/// target creature; a tapped New Planet land token ("{T}: any color").
pub fn spatial_anomaly() -> CardDefinition {
    let planet = Arc::new(TokenDefinition {
        name: "New Planet".into(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        name: "Spatial Anomaly",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    damage(target_filtered(R::Creature), 2),
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: planet },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Pah-Wraith — {5}{R}{R} 4/4 Wraith, first strike, haste. Attacks:
/// creatures you control get +X/+0, X = your differently named lands.
pub fn pah_wraith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::DistinctNamesControlledMatching(R::Land),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Pah-Wraith", cost(&[generic(5), r(), r()]), vec![CreatureType::Wraith], 4, 4)
    }
}

/// Tal Shiar Analyst — {2}{R} 3/2 Romulan Scientist. {T}: 1 damage to each
/// opponent. Whenever you cast a noncreature spell, untap it.
pub fn tal_shiar_analyst() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: damage(Selector::Player(PlayerRef::EachOpponent), 1),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::Creature)) },
            ),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..creature(
            "Tal Shiar Analyst",
            cost(&[generic(2), r()]),
            vec![CreatureType::Romulan, CreatureType::Scientist],
            3,
            2,
        )
    }
}

/// Khaaaaaaaaaaaannn! — {X}{R} instant. Twice X damage to target creature;
/// if it would die this turn, exile it instead.
pub fn khaaaaaaaaaaaannn() -> CardDefinition {
    spell(
        "Khaaaaaaaaaaaannn!",
        cost(&[x(), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]),
    )
}

/// Arrow Storm — {3}{R}{R} sorcery. 4 damage to any target. Raid — if you
/// attacked this turn, 5 damage instead, and it can't be prevented.
pub fn arrow_storm() -> CardDefinition {
    spell(
        "Arrow Storm",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::If {
            cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::DamageCantBePrevented { body: Box::new(damage(target_filtered(R::any_target()), 5)) }),
            else_: Box::new(damage(target_filtered(R::any_target()), 4)),
        },
    )
}

/// Cinder Storm — {6}{R} sorcery. 7 damage to any target.
pub fn cinder_storm() -> CardDefinition {
    spell("Cinder Storm", cost(&[generic(6), r()]), CardType::Sorcery, damage(target_filtered(R::any_target()), 7))
}

/// Heirs of Stromkirk — {2}{R}{R} 2/2 Vampire, intimidate. Combat damage to a
/// player: a +1/+1 counter on it.
pub fn heirs_of_stromkirk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature("Heirs of Stromkirk", cost(&[generic(2), r(), r()]), vec![CreatureType::Vampire], 2, 2)
    }
}

/// Skirk Shaman — {1}{R}{R} 2/2 Goblin Shaman; can't be blocked except by
/// artifact creatures and/or red creatures.
pub fn skirk_shaman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(R::Artifact.or(R::HasColor(Color::Red))))],
        ..creature(
            "Skirk Shaman",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Goblin, CreatureType::Shaman],
            2,
            2,
        )
    }
}

/// Starfleet Security — {R} 1/1 Human Officer, haste. Dies during your turn:
/// investigate.
pub fn starfleet_security() -> CardDefinition {
    let mut dies = on_dies(investigate(1));
    dies.event = dies.event.with_filter(Predicate::IsTurnOf(PlayerRef::You));
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![dies],
        ..creature("Starfleet Security", cost(&[r()]), vec![CreatureType::Human, CreatureType::Officer], 1, 1)
    }
}

/// Stun Sniper — {R}{W} 1/1 Human Archer. {1}, {T}: 1 damage to target
/// creature; tap it.
pub fn stun_sniper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                damage(target_filtered(R::Creature), 1),
                Effect::Tap { what: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature("Stun Sniper", cost(&[r(), w()]), vec![CreatureType::Human, CreatureType::Archer], 1, 1)
    }
}

/// The Naked Time — {1}{R} Saga. I — a basic Mountain card from your library
/// onto the battlefield tapped. II — whenever a creature you control enters
/// this turn, a +1/+1 counter on it and haste until end of turn. III —
/// sacrifice a land.
pub fn the_naked_time() -> CardDefinition {
    CardDefinition {
        name: "The Naked Time",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand.and(R::HasLandType(LandType::Mountain)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ),
            (
                2,
                Effect::CreaturesYouControlEnteringThisTurn {
                    body: Box::new(Effect::Seq(vec![
                        Effect::AddCounter {
                            what: Selector::TriggerSource,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource,
                            keyword: Keyword::Haste,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            ),
            (3, Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Land }),
        ],
        ..Default::default()
    }
}

/// Neck Snap — {3}{W} instant. Destroy target attacking or blocking creature.
pub fn neck_snap() -> CardDefinition {
    spell(
        "Neck Snap",
        cost(&[generic(3), w()]),
        CardType::Instant,
        Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))) },
    )
}

/// Giant Space Amoeba — {4}{U} 3/3 flying Germ. Attacks: target creature
/// defending player controls gets -3/-0 until end of turn, then a token copy
/// of this creature.
pub fn giant_space_amoeba() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                power: Value::Const(-3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            crate::effect::shortcut::token_copy_of(PlayerRef::You, Value::ONE, Selector::This),
        ]))],
        ..creature("Giant Space Amoeba", cost(&[generic(4), crate::mana::u()]), vec![CreatureType::Germ], 3, 3)
    }
}

/// Cold-Water Snapper — {5}{U} 4/5 Turtle, hexproof.
pub fn cold_water_snapper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        ..creature("Cold-Water Snapper", cost(&[generic(5), crate::mana::u()]), vec![CreatureType::Turtle], 4, 5)
    }
}

/// Stream of Thought — {U} sorcery. Target player mills four; you shuffle up
/// to four cards from your graveyard into your library. Replicate {2}{U}{U}.
pub fn stream_of_thought() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        keywords: vec![Keyword::Replicate(cost(&[generic(2), u(), u()]))],
        ..spell(
            "Stream of Thought",
            cost(&[u()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::Mill { who: target_filtered(R::Player), amount: Value::Const(4) },
                Effect::ShuffleGraveyardCardsIntoLibrary {
                    who: PlayerRef::You,
                    filter: R::Any,
                    max: Value::Const(4),
                    to_top: false,
                },
            ]),
        )
    }
}
