//! Commander: the cards the **Everyone's Invited!** Secret Lair Commander
//! deck (SLD, Morophon, the Boundless) needed beyond what the catalog had.
//! Tests in `tests/recent_b/cmdr_morophon.rs`.
//!
//! Residuals (each also on its card):
//! - **Amoeboid Changeling, Nameless Inversion, Shields of Velis Vel** — "all
//!   creature types" is a Changeling grant and "loses all creature types"
//!   empties the type line and strips Changeling until end of turn (a later
//!   grant the same turn doesn't restore it).
//! - **Moritte of the Frost** — the two counters come from an entry trigger,
//!   and a noncreature copy has changeling too (harmless).
//! - **Unsettled Mariner** — ward {1} on your permanents; you aren't
//!   protected, and it doesn't stack with a printed ward.
//! - **Stick Together, Harper Recruiter** — the party is the engine's pick
//!   (a largest one, strongest creatures first).

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, investigate, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, mono_hybrid, r, u, w};
use crabomination_base::tokens::food_token;
use std::sync::Arc;

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

/// A 1/1 Shapeshifter with changeling — six of the list's bodies.
fn changeling(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        ..creature(name, mana, vec![CreatureType::Shapeshifter], 1, 1)
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// A Kindred Shapeshifter instant with changeling (Crib Swap's frame).
fn kindred_instant(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Kindred, CardType::Instant],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Changeling],
        ..spell(name, mana, CardType::Instant, effect)
    }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn make(count: Value, definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(definition) }
}

/// "gains all creature types until end of turn" — a Changeling grant.
fn gain_all_types(what: Selector) -> Effect {
    Effect::GrantKeyword { what, keyword: Keyword::Changeling, duration: Duration::EndOfTurn }
}

/// "loses all creature types until end of turn" — an empty type line, and
/// no Changeling to answer every type anyway.
fn lose_all_types(what: Selector) -> Effect {
    Effect::Seq(vec![
        Effect::BecomeCreatureType { what: what.clone(), creature_types: vec![], duration: Duration::EndOfTurn },
        Effect::LoseKeyword { what, keyword: Keyword::Changeling, duration: Duration::EndOfTurn },
    ])
}

/// Amoeboid Changeling — {T}: a creature gains, or loses, all creature types.
pub fn amoeboid_changeling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: gain_all_types(target_filtered(R::Creature)),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: lose_all_types(target_filtered(R::Creature)),
                ..Default::default()
            },
        ],
        ..changeling("Amoeboid Changeling", cost(&[generic(1), u()]))
    }
}

/// Brenard, Ginger Sculptor — Food and Golem creatures of yours get +2/+2
/// and trample; a dying nontoken creature of yours may come back as a 1/1
/// Food Golem token copy.
pub fn brenard_ginger_sculptor() -> CardDefinition {
    let gain = ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        tap_cost: true,
        sac_cost: true,
        effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![static_ab(
            "Each creature you control that's a Food or a Golem gets +2/+2 and has trample.",
            StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou).and(
                    R::HasArtifactSubtype(ArtifactSubtype::Food).or(R::HasCreatureType(CreatureType::Golem)),
                ),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Trample],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
            ),
            effect: Effect::MayDo {
                description: "Exile it to make a Food Golem token copy?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::TriggerSource,
                        extra_creature_types: vec![CreatureType::Golem],
                        extra_card_types: vec![CardType::Artifact],
                        override_pt: Some((1, 1)),
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                    Effect::StampTokenCopyExceptions {
                        artifact_subtypes: vec![ArtifactSubtype::Food],
                        activated: vec![gain],
                    },
                ])),
            },
        }],
        ..legend(
            "Brenard, Ginger Sculptor",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Fire-Belly Changeling — {R}: +1/+0, at most twice a turn.
pub fn fire_belly_changeling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            max_activations_per_turn: Some(2),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..changeling("Fire-Belly Changeling", cost(&[generic(1), r()]))
    }
}

/// Guardian Gladewalker — a +1/+1 counter on a creature as it enters.
pub fn guardian_gladewalker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..changeling("Guardian Gladewalker", cost(&[generic(1), g()]))
    }
}

/// Harper Recruiter — on attack, a party from the top four. Residual: the
/// engine picks the party.
pub fn harper_recruiter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::LookTopTakeParty { who: PlayerRef::You, count: Value::Const(4) })],
        ..creature("Harper Recruiter", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 1)
    }
}

/// Moritte of the Frost — may enter as a copy of a permanent you control,
/// legendary and snow, a creature one with two more +1/+1 counters and
/// changeling. Residual: the counters come from an entry trigger.
pub fn moritte_of_the_frost() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary, Supertype::Snow],
        keywords: vec![Keyword::Changeling],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Permanent.and(R::ControlledByYou),
            legendary: true,
            extra_supertypes: vec![Supertype::Snow],
            extra_keywords: vec![Keyword::Changeling],
            extra_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::This, filter: R::Creature },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            }],
            ..Default::default()
        }),
        ..creature(
            "Moritte of the Frost",
            cost(&[generic(2), g(), u(), u()]),
            vec![CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Mothdust Changeling — tap a creature you control: flying until end of turn.
pub fn mothdust_changeling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Creature.and(R::ControlledByYou), 1)),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..changeling("Mothdust Changeling", cost(&[u()]))
    }
}

/// Nameless Inversion — +3/-3 and loses all creature types.
pub fn nameless_inversion() -> CardDefinition {
    kindred_instant(
        "Nameless Inversion",
        cost(&[generic(1), b()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            lose_all_types(Selector::Target(0)),
        ]),
    )
}

/// Raise the Palisade — choose a creature type; bounce every creature not
/// of it.
pub fn raise_the_palisade() -> CardDefinition {
    spell(
        "Raise the Palisade",
        cost(&[generic(4), u()]),
        CardType::Sorcery,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::Move {
                what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::IsSourceChosenCreatureType)))),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        },
    )
}

/// Realmbreaker, the Invasion Tree — an opponent mills three and you take a
/// land of theirs (exiled if it would leave); {10}: any number of Praetors.
pub fn realmbreaker_the_invasion_tree() -> CardDefinition {
    CardDefinition {
        name: "Realmbreaker, the Invasion Tree",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Mill { who: target_filtered(R::OpponentPlayer), amount: Value::Const(3) },
                    Effect::MoveChosen {
                        from: Selector::CardsInZone {
                            who: PlayerRef::Target(0),
                            zone: Zone::Graveyard,
                            filter: R::Land,
                        },
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    },
                    Effect::ExileIfLeavesBattlefield { what: Selector::LastMoved },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(10)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::SearchAnyNumber {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Praetor),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Shapesharer — {2}{U}: a Shapeshifter becomes a copy of a creature until
/// your next turn.
pub fn shapesharer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::BecomeCopyOfFor {
                what: Selector::TargetFiltered { slot: 0, filter: R::HasCreatureType(CreatureType::Shapeshifter) },
                source: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                duration: Duration::UntilNextTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..changeling("Shapesharer", cost(&[generic(1), u()]))
    }
}

/// Shields of Velis Vel — a player's creatures get +0/+1 and all creature
/// types until end of turn.
pub fn shields_of_velis_vel() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    kindred_instant(
        "Shields of Velis Vel",
        cost(&[w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: theirs(),
                power: Value::Const(0),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            gain_all_types(theirs()),
        ]),
    )
}

/// Skeletal Changeling — {1}{B}: regenerate.
pub fn skeletal_changeling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..changeling("Skeletal Changeling", cost(&[generic(1), b()]))
    }
}

/// Sophia, Dogged Detective — Tiny on entry; artifact tokens feed Dogs
/// counters; Dogs hitting a player make a Food and a Clue.
pub fn sophia_dogged_detective() -> CardDefinition {
    let tiny = TokenDefinition {
        name: "Tiny".into(),
        power: 2,
        toughness: 2,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Detective],
            ..Default::default()
        },
        keywords: vec![Keyword::Trample],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(make(Value::ONE, tiny)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Dog),
                    },
                ),
                effect: Effect::Seq(vec![make(Value::ONE, food_token()), investigate(1)]),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact.and(R::IsToken), 1)),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Dog).and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend(
            "Sophia, Dogged Detective",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            3,
            4,
        )
    }
}

/// Spoils of Adventure — {1} less per party creature; 3 life, three cards.
pub fn spoils_of_adventure() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "This spell costs {1} less to cast for each creature in your party.",
            StaticEffect::SelfCostReducedByValue { amount: Value::PartyCount },
        )],
        ..spell(
            "Spoils of Adventure",
            cost(&[generic(4), w(), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            ]),
        )
    }
}

/// Stick Together — each player keeps a party and sacrifices the rest of
/// their creatures. Residual: the party is the engine's pick.
pub fn stick_together() -> CardDefinition {
    spell(
        "Stick Together",
        cost(&[generic(3), w(), w()]),
        CardType::Sorcery,
        Effect::EachPlayerKeepsPartySacrificesRest,
    )
}

/// Tazri, Beacon of Unity — {1} less per party creature; a hybrid-pip dig
/// for up to two Clerics, Rogues, Warriors, Wizards or Allies.
pub fn tazri_beacon_of_unity() -> CardDefinition {
    let party_or_ally = R::HasCreatureType(CreatureType::Cleric)
        .or(R::HasCreatureType(CreatureType::Rogue))
        .or(R::HasCreatureType(CreatureType::Warrior))
        .or(R::HasCreatureType(CreatureType::Wizard))
        .or(R::HasCreatureType(CreatureType::Ally));
    CardDefinition {
        static_abilities: vec![static_ab(
            "This spell costs {1} less to cast for each creature in your party.",
            StaticEffect::SelfCostReducedByValue { amount: Value::PartyCount },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[
                mono_hybrid(2, Color::Blue),
                mono_hybrid(2, Color::Black),
                mono_hybrid(2, Color::Red),
                mono_hybrid(2, Color::Green),
            ]),
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(6),
                pick_filter: Some(party_or_ally),
                take: Some(Value::Const(2)),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..legend(
            "Tazri, Beacon of Unity",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            6,
        )
    }
}

/// The Bears of Littjara — I: a 2/2 changeling Shapeshifter; II: your
/// Shapeshifters become base 4/4; III: each 4-power creature of yours hits
/// up to one creature or planeswalker.
pub fn the_bears_of_littjara() -> CardDefinition {
    let shifter = TokenDefinition {
        name: "Shapeshifter".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Changeling],
        ..Default::default()
    };
    CardDefinition {
        name: "The Bears of Littjara",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, make(Value::ONE, shifter)),
            (
                2,
                Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByYou).and(R::HasCreatureType(CreatureType::Shapeshifter)),
                    effect: Box::new(Effect::SetBasePT {
                        what: Selector::Target(0),
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        duration: Duration::Permanent,
                    }),
                },
            ),
            (
                3,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::EachDealsDamageEqualToPower {
                        dealers: Selector::EachPermanent(
                            R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                        ),
                        target: target_filtered(R::Creature.or(R::Planeswalker)),
                    }),
                },
            ),
        ],
        ..Default::default()
    }
}

/// Unsettled Mariner — your permanents have ward {1}. Residual: you aren't
/// protected, and it doesn't stack with a printed ward.
pub fn unsettled_mariner() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Whenever you or a permanent you control becomes the target of a spell or ability an opponent controls, counter that spell or ability unless its controller pays {1}.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::ControlledByYou),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(1)]))),
            },
        )],
        ..CardDefinition {
            subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
            power: 2,
            toughness: 2,
            ..changeling("Unsettled Mariner", cost(&[w(), u()]))
        }
    }
}
