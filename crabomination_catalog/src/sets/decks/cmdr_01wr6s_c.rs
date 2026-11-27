//! Commander: closers for Una Chin-Riley, Hologram Janeway, Kirk Man of
//! Action, Lore, Gabriel Lorca Mirror Menace and Borg Queen Perfection
//! Manifest's EDHREC average decks (scan of 2026-11-27). Tests in
//! `tests/recent_b/cmdr_01wr6s_a.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, KeywordAct, ManaPayload, PlayerRef, StaticEffect, Value, ZoneDest};
use crate::mana::{Color, ManaCost, SpendRestriction, b, cost, g, generic, hybrid, u, w};

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn artifact_creature(name: &'static str, mana: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, mana, vec![CreatureType::Borg], p, t)
    }
}

/// "Beam me up {cost}": flashback that also returns a creature you control to
/// its owner's hand.
fn beam_me_up(def: CardDefinition, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(mana)],
        flashback_additional_cost: vec![AdditionalCastCost::ReturnToHand {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
            count_x: false,
        }],
        ..def
    }
}

fn sorcery(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

/// Una Chin-Riley, Number One — {4}{W} legendary 4/5 Illyrian Officer.
/// Whenever you attack while creatures you control have total power 8 or
/// greater, target attacking creature gets +2/+0 and gains first strike
/// until end of turn.
pub fn una_chin_riley_number_one() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8))),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..legend(
            "Una Chin-Riley, Number One",
            cost(&[generic(4), w()]),
            vec![CreatureType::Illyrian, CreatureType::Officer],
            4,
            5,
        )
    }
}

/// Pure Energy — {1}{W} sorcery. Distribute two +1/+1 counters among one or
/// two target creatures. Beam me up {2}{W}.
pub fn pure_energy() -> CardDefinition {
    beam_me_up(
        sorcery(
            "Pure Energy",
            cost(&[generic(1), w()]),
            Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 2,
            },
        ),
        cost(&[generic(2), w()]),
    )
}

/// Phlox, Unorthodox Doctor — {2}{G}{W} legendary 3/4 Denobulan Doctor,
/// trample. Whenever a creature you control with a counter on it deals
/// combat damage to a player, draw a card.
pub fn phlox_unorthodox_doctor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::WithAnyCounter) },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Phlox, Unorthodox Doctor",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Denobulan, CreatureType::Doctor],
            3,
            4,
        )
    }
}

/// The Inner Light — {3}{G}{W} Saga. I — return target green or white
/// creature card from your graveyard to the battlefield. II — choose target
/// creature; whenever it deals combat damage to a player this turn, you draw
/// that many cards. III — gain life equal to the greatest power among
/// creatures you control; create a Treasure.
pub fn the_inner_light() -> CardDefinition {
    let green_or_white = R::HasColor(Color::Green).or(R::HasColor(Color::White));
    CardDefinition {
        name: "The Inner Light",
        cost: cost(&[generic(3), g(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Move {
                    what: target_filtered(R::Creature.and(green_or_white).from_your_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ),
            (
                2,
                // A +0/+0 pump names slot 0's filter for the target walkers
                // (the delayed trigger only carries the slot).
                Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::ZERO,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::WhenTargetDealsCombatDamageToPlayerThisTurn {
                        slot: 0,
                        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount }),
                        this_combat: false,
                    },
                ]),
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)) },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(crate::game::effects::treasure_token()),
                    },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Lore, Positronic Provocateur — {1}{U/B}{U/B} legendary 2/3 artifact
/// Android Rogue. ETB, choose one — target creature gets +1/+0 and can't be
/// blocked this turn; or surveil 2.
pub fn lore_positronic_provocateur() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
        ]))],
        ..legend(
            "Lore, Positronic Provocateur",
            cost(&[generic(1), hybrid(Color::Blue, Color::Black), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Android, CreatureType::Rogue],
            2,
            3,
        )
    }
}

/// Contingency Plan — {1}{U} sorcery. Surveil 5.
pub fn contingency_plan() -> CardDefinition {
    sorcery(
        "Contingency Plan",
        cost(&[generic(1), u()]),
        Effect::Surveil { who: PlayerRef::You, amount: Value::Const(5) },
    )
}

/// Energize — {1}{U} sorcery. Return target creature an opponent controls to
/// its owner's hand. Beam me up {3}{U}.
pub fn energize() -> CardDefinition {
    beam_me_up(
        sorcery(
            "Energize",
            cost(&[generic(1), u()]),
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        ),
        cost(&[generic(3), u()]),
    )
}

/// Gabriel Lorca, Mirror Menace — {4}{B} legendary 5/4 Human Rogue Officer,
/// menace. Whenever you face a dilemma, you may pay {B}; if you do, return
/// this card from your graveyard to your hand (a graveyard trigger).
pub fn gabriel_lorca_mirror_menace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(KeywordAct::FacedDilemma), EventScope::FromYourGraveyard),
            effect: Effect::MayPay {
                description: "Pay {B}: return Gabriel Lorca from your graveyard to your hand".into(),
                mana_cost: cost(&[b()]),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                else_: None,
            },
        }],
        ..legend(
            "Gabriel Lorca, Mirror Menace",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Officer],
            5,
            4,
        )
    }
}

/// Life of Toshiro Umezawa // Memory of Toshiro — {1}{B} Saga. I, II —
/// choose one: target creature +2/+2, target creature -1/-1 (until end of
/// turn), or gain 2 life. III — exile it, return it transformed. Back: 2/3
/// Human Samurai enchantment creature; {T}, pay 1 life: {B} for instants and
/// sorceries only.
pub fn life_of_toshiro_umezawa() -> CardDefinition {
    let pump = |p: i32| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(p),
        toughness: Value::Const(p),
        duration: Duration::EndOfTurn,
    };
    let chapter = || {
        Effect::ChooseMode(vec![pump(2), pump(-1), Effect::GainLife { who: Selector::You, amount: Value::Const(2) }])
    };
    let memory = CardDefinition {
        name: "Memory of Toshiro",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Samurai], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Black])),
                    SpendRestriction::InstantSorceryOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Life of Toshiro Umezawa",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![(1, chapter()), (2, chapter()), (3, Effect::ExileSelfReturnTransformed)],
        back_face: Some(Box::new(memory)),
        ..Default::default()
    }
}

/// Debt to the Kami — {2}{B} instant. Choose one — target opponent exiles a
/// creature they control; or target opponent exiles an enchantment they
/// control.
pub fn debt_to_the_kami() -> CardDefinition {
    let exiles = |filter: R| Effect::TargetPlayerThen {
        filter: R::OpponentPlayer,
        then: Box::new(Effect::PlayerExilesPermanents { who: PlayerRef::Target(0), count: Value::ONE, filter }),
    };
    CardDefinition {
        name: "Debt to the Kami",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![exiles(R::Creature), exiles(R::Enchantment)]),
        ..Default::default()
    }
}

/// Collective Drone — {1}{B} */2 artifact Borg; power is the number of
/// artifact creatures you control.
pub fn collective_drone() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatchingPower {
            base_p: 0,
            base_t: 2,
            filter: Box::new(R::Artifact.and(R::Creature)),
        }),
        ..artifact_creature("Collective Drone", cost(&[generic(1), b()]), 0, 2)
    }
}

/// Organic Avulsion Unit — {2}{B} 2/3 artifact Borg. {2}: artifact creatures
/// you control gain menace until end of turn.
pub fn organic_avulsion_unit() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact_creature("Organic Avulsion Unit", cost(&[generic(2), b()]), 2, 3)
    }
}

/// Overflow Submatrix — {2}{B} 3/3 artifact Borg. Once during each of your
/// turns, you may cast an artifact creature spell from your graveyard; it
/// enters with a finality counter.
pub fn overflow_submatrix() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast an artifact creature spell from your graveyard. If you cast a spell this way, that artifact creature enters with a finality counter on it.",
            effect: StaticEffect::GraveyardCastOncePerTurn {
                filter: R::Artifact.and(R::Creature),
                exile_after: false,
                mv_at_most_counters: None,
                enters_tapped: false,
                finality: true,
            },
        }],
        ..artifact_creature("Overflow Submatrix", cost(&[generic(2), b()]), 3, 3)
    }
}
