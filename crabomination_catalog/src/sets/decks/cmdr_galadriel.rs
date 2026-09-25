//! Commander: the cards the **Elven Council** precon (LTC, Galadriel,
//! Elven-Queen) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_galadriel.rs`.
//!
//! Residuals (each also on its card):
//! - **Celeborn the Wise** — +1/+1 per scry or surveil, not per card looked
//!   at.
//! - **Elrond of the White Council** — the voter's creature is the engine's
//!   pick, and it may attack its owner.
//! - **Sail into the West** — on embark every player wheels; the "may"
//!   isn't offered.
//! - **Gandalf, Westward Voyager** — the opponents' top cards are read, not
//!   revealed.
//! - **Mirkwood Trapper** — the shrunk attacker is the first one declared,
//!   not a target; its second ability (an attacker's +2/+0 when you aren't
//!   attacked) isn't implemented.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword,
    LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, VoteOption, VoteTally, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, x};
use crate::sets::tap_add_any_color;
use crabomination_base::tokens::treasure_token;
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
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

fn elf() -> R {
    R::HasCreatureType(CreatureType::Elf)
}

fn token(name: &str, color: Color, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.to_string(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        keywords,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn elf_warriors(count: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count,
        definition: token("Elf Warrior", Color::Green, vec![CreatureType::Elf, CreatureType::Warrior], 1, 1, vec![]),
    }
}

fn bird() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: token("Bird", Color::Blue, vec![CreatureType::Bird], 2, 2, vec![Keyword::Flying]),
    }
}

fn treasure() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treasure_token()) }
}

fn draw(who: PlayerRef, n: i32) -> Effect {
    Effect::Draw { who: Selector::Player(who), amount: Value::Const(n) }
}

fn scry(who: PlayerRef, amount: Value) -> Effect {
    Effect::Scry { who, amount }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// "Whenever you cast a spell with mana value 5 or greater".
fn big_spell_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::ManaValueAtLeast(5))),
        effect,
    }
}

/// Arwen, Weaver of Hope — each other creature you control enters with
/// additional +1/+1 counters equal to Arwen's toughness.
pub fn arwen_weaver_of_hope() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control enters with additional +1/+1 counters equal to Arwen's toughness.",
            effect: StaticEffect::OtherCreaturesEnterWithCountersEqualToSourceToughness {
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        ..creature(
            "Arwen, Weaver of Hope",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            2,
            1,
        )
    })
}

/// Asceticism — creatures you control have hexproof; {1}{G}: regenerate
/// target creature.
pub fn asceticism() -> CardDefinition {
    CardDefinition {
        name: "Asceticism",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have hexproof.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature), keyword: Keyword::Hexproof },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Celeborn the Wise — attacking with Elves, scry 1; scrying grows it.
///
/// ⚠ Residual: +1/+1 per scry or surveil, not per card looked at.
pub fn celeborn_the_wise() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource)
                    .with_filter(Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: elf() }),
                effect: scry(PlayerRef::You, Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature("Celeborn the Wise", cost(&[generic(3), g()]), vec![CreatureType::Elf, CreatureType::Noble], 3, 3)
    })
}

/// Colossal Whale — islandwalk; attacking, you may exile target creature the
/// defending player controls until it leaves.
pub fn colossal_whale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island)],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Exile a creature the defending player controls until Colossal Whale leaves?".into(),
            body: Box::new(Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                return_to: ExileReturnZone::Battlefield,
            }),
        })],
        ..creature("Colossal Whale", cost(&[generic(5), u(), u()]), vec![CreatureType::Whale], 5, 5)
    }
}

/// Círdan the Shipwright — vigilance; entering or attacking, secret council
/// for a player: a card per vote received, and a free permanent for each
/// player nobody voted for.
pub fn cirdan_the_shipwright() -> CardDefinition {
    let council = || Effect::SecretCouncilPlayerVote {
        per_vote: Box::new(draw(PlayerRef::Triggerer, 1)),
        unvoted: Box::new(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::Triggerer,
            filter: R::PermanentCard,
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: None,
        }),
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(council()), on_attack(council())],
        ..creature(
            "Círdan the Shipwright",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            3,
            4,
        )
    })
}

/// Elrond of the White Council — secret council: each aid vote puts a +1/+1
/// counter on each creature you control; each fellowship vote hands you one
/// of the voter's creatures.
///
/// ⚠ Residual: the voter's creature is the engine's pick, and it may attack
/// its owner.
pub fn elrond_of_the_white_council() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![etb(Effect::Vote {
            tally: VoteTally::PerVote,
            options: vec![
                VoteOption::new(
                    "aid",
                    Effect::AddCounter { what: yours(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                ),
                VoteOption::new(
                    "fellowship",
                    Effect::GainControl {
                        what: Selector::Take {
                            inner: Box::new(Selector::ControlledBy { who: PlayerRef::CurrentVoter, filter: R::Creature }),
                            count: Box::new(Value::ONE),
                        },
                        to: None,
                        duration: Duration::Permanent,
                    },
                ),
            ],
        })],
        ..creature(
            "Elrond of the White Council",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            3,
            3,
        )
    })
}

/// Erestor of the Council — after a vote, opponents who agreed with you make
/// Treasures, you scry one per opponent who didn't, and you draw.
pub fn erestor_of_the_council() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::VotingFinished, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::OpponentsWhoVotedTheSame),
                    body: Box::new(Effect::CreateToken {
                        who: PlayerRef::Triggerer,
                        count: Value::ONE,
                        definition: Arc::new(treasure_token()),
                    }),
                },
                scry(PlayerRef::You, Value::PlayersIn(PlayerRef::OpponentsWhoVotedDifferently)),
                draw(PlayerRef::You, 1),
            ]),
        }],
        ..creature(
            "Erestor of the Council",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            2,
            4,
        )
    })
}

/// Galadhrim Ambush — an Elf Warrior per attacking creature; non-Elves deal
/// no combat damage this turn.
pub fn galadhrim_ambush() -> CardDefinition {
    spell(
        "Galadhrim Ambush",
        cost(&[generic(3), g()]),
        true,
        Effect::Seq(vec![
            elf_warriors(Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttacking))))),
            Effect::PreventAllCombatDamageByMatchingThisTurn { filter: R::Creature.and(elf().negate()) },
        ]),
    )
}

/// Galadriel, Elven-Queen — at the beginning of combat on your turn, if
/// another Elf entered under your control this turn, will of the council:
/// dominion tempts you with the Ring and grows your Ring-bearer; guidance
/// (or a tie) draws a card.
pub fn galadriel_elven_queen() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl).with_filter(
                Predicate::CreatureEnteredThisTurnMatching { who: PlayerRef::You, filter: elf().and(R::OtherThanSource) },
            ),
            effect: Effect::Vote {
                tally: VoteTally::Majority,
                options: vec![
                    VoteOption::new(
                        "dominion",
                        Effect::Seq(vec![
                            Effect::RingTempts { who: PlayerRef::You },
                            Effect::AddCounter {
                                what: Selector::RingBearerOf(PlayerRef::You),
                                kind: CounterType::PlusOnePlusOne,
                                amount: Value::ONE,
                            },
                        ]),
                    ),
                    VoteOption::new("guidance", draw(PlayerRef::You, 1)),
                ],
            },
        }],
        ..creature(
            "Galadriel, Elven-Queen",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Noble],
            4,
            5,
        )
    })
}

/// Gandalf, Westward Voyager — a big spell cast: if an opponent's top card
/// shares a card type with it, copy it and each opponent draws; otherwise you
/// draw.
///
/// ⚠ Residual: the opponents' top cards are read, not revealed.
pub fn gandalf_westward_voyager() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![big_spell_cast(Effect::If {
            cond: Predicate::AnOpponentsTopCardSharesCardTypeWith(Selector::TriggerSource),
            then: Box::new(Effect::Seq(vec![
                Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
                draw(PlayerRef::EachOpponent, 1),
            ])),
            else_: Box::new(draw(PlayerRef::You, 1)),
        })],
        ..legendary(creature(
            "Gandalf, Westward Voyager",
            cost(&[generic(3), g(), u()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            5,
            5,
        ))
    }
}

/// Haldir, Lórien Lieutenant — enters with X counters; vigilance; {5}{G}:
/// other Elves get vigilance and +1/+1 per counter on Haldir.
pub fn haldir_lorien_lieutenant() -> CardDefinition {
    let others = || yours(elf().and(R::OtherThanSource));
    let n = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g()]),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: others(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                Effect::PumpPT { what: others(), power: n(), toughness: n(), duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Haldir, Lórien Lieutenant",
            cost(&[x(), g()]),
            vec![CreatureType::Elf, CreatureType::Soldier],
            0,
            0,
        )
    })
}

/// Learn from the Past — target player shuffles their graveyard into their
/// library; draw a card.
pub fn learn_from_the_past() -> CardDefinition {
    spell(
        "Learn from the Past",
        cost(&[generic(3), u()]),
        true,
        Effect::Seq(vec![
            Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::Target(0) },
            draw(PlayerRef::You, 1),
        ]),
    )
}

/// Legolas Greenleaf — reach; can't be blocked by power 2 or less; grows as
/// other legends enter; draws on connecting.
pub fn legolas_greenleaf() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::CantBeBlockedByPowerAtMost(2)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                    },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: draw(PlayerRef::You, 1),
            },
        ],
        ..creature(
            "Legolas Greenleaf",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Archer],
            2,
            2,
        )
    })
}

/// Lothlórien Blade — equipped creature attacking deals damage equal to its
/// power to target creature the defending player controls; equip Elf {2};
/// equip {5}.
pub fn lothlorien_blade() -> CardDefinition {
    CardDefinition {
        name: "Lothlórien Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(5)]))],
        equip_filtered_cost: Some((elf(), cost(&[generic(2)]))),
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::DealDamageFrom {
                    source: Selector::This,
                    to: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Mirkwood Elk — trample; entering or attacking, return an Elf card from
/// your graveyard and gain life equal to its power.
pub fn mirkwood_elk() -> CardDefinition {
    let recur = || {
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(elf().and(R::InYourGraveyard)), to: ZoneDest::Hand(PlayerRef::You) },
            Effect::GainLife { who: Selector::You, amount: Value::PowerOf(Box::new(Selector::LastMoved)) },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(recur()), on_attack(recur())],
        ..creature("Mirkwood Elk", cost(&[generic(5), g()]), vec![CreatureType::Elk], 6, 6)
    }
}

/// Mirkwood Trapper — a player attacking you shrinks one of their attackers
/// by 2 power.
///
/// ⚠ Residual: the shrunk attacker is the first one declared, not a target;
/// the second ability (an attacker's +2/+0 when you aren't attacked) isn't
/// implemented.
pub fn mirkwood_trapper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_batch: true,
                ..EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent)
            },
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Mirkwood Trapper",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Elf, CreatureType::Scout],
            1,
            4,
        )
    }
}

/// Model of Unity — after a vote, you and each opponent who agreed with you
/// may scry 2; {T}: add one mana of any color.
pub fn model_of_unity() -> CardDefinition {
    CardDefinition {
        name: "Model of Unity",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_add_any_color()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::VotingFinished, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::MayDo { description: "Scry 2?".into(), body: Box::new(scry(PlayerRef::You, Value::Const(2))) },
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::OpponentsWhoVotedTheSame),
                    body: Box::new(Effect::MayDoBy {
                        who: PlayerRef::Triggerer,
                        description: "Scry 2?".into(),
                        body: Box::new(scry(PlayerRef::Triggerer, Value::Const(2))),
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Radagast, Wizard of Wilds — ward {1}; your Beasts and Birds have ward {1};
/// a big spell cast makes a 3/3 Beast or a 2/2 flying Bird.
pub fn radagast_wizard_of_wilds() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Ward(WardCost::generic(1))],
        static_abilities: vec![StaticAbility {
            description: "Beasts and Birds you control have ward {1}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::HasCreatureType(CreatureType::Beast).or(R::HasCreatureType(CreatureType::Bird))),
                keyword: Keyword::Ward(WardCost::generic(1)),
            },
        }],
        triggered_abilities: vec![big_spell_cast(Effect::ChooseMode(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: token("Beast", Color::Green, vec![CreatureType::Beast], 3, 3, vec![]),
            },
            bird(),
        ]))],
        ..creature(
            "Radagast, Wizard of Wilds",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            3,
            5,
        )
    })
}

/// Sail into the West — will of the council: return (each player takes back
/// up to two cards, and this is exiled) or embark (a tie too: each player may
/// wheel for seven).
///
/// ⚠ Residual: on embark every player wheels; the "may" isn't offered.
pub fn sail_into_the_west() -> CardDefinition {
    spell(
        "Sail into the West",
        cost(&[generic(2), g(), u()]),
        true,
        Effect::Vote {
            tally: VoteTally::Majority,
            options: vec![
                VoteOption::new(
                    "return",
                    Effect::Seq(vec![
                        Effect::ForEach {
                            selector: Selector::Player(PlayerRef::EachPlayer),
                            body: Box::new(Effect::MoveChosen {
                                from: Selector::CardsInZone { who: PlayerRef::Triggerer, zone: Zone::Graveyard, filter: R::Any },
                                filter: None,
                                count: Value::Const(2),
                                up_to: true,
                                to: ZoneDest::Hand(PlayerRef::Triggerer),
                            }),
                        },
                        Effect::ExileResolvingSpell,
                    ]),
                ),
                VoteOption::new(
                    "embark",
                    Effect::ForEach {
                        selector: Selector::Player(PlayerRef::EachPlayer),
                        // Unconditional: a `MayDoBy` inside the ballot would
                        // replay the vote's answers (one answer-log channel).
                        body: Box::new(Effect::Seq(vec![
                            Effect::Discard {
                                who: Selector::Player(PlayerRef::Triggerer),
                                amount: Value::HandSizeOf(PlayerRef::Triggerer),
                                random: false,
                            },
                            draw(PlayerRef::Triggerer, 7),
                        ])),
                    },
                ),
            ],
        },
    )
}

/// Song of Eärendil — I: scry 2, draw two. II: a Treasure and a 2/2 flying
/// Bird. III: a flying counter on each creature you control without flying.
pub fn song_of_earendil() -> CardDefinition {
    CardDefinition {
        name: "Song of Eärendil",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::Seq(vec![scry(PlayerRef::You, Value::Const(2)), draw(PlayerRef::You, 2)])),
            (2, Effect::Seq(vec![treasure(), bird()])),
            (
                3,
                Effect::AddKeywordCounter {
                    what: yours(R::Creature.and(R::HasKeyword(Keyword::Flying).negate())),
                    keyword: Keyword::Flying,
                    amount: Value::ONE,
                },
            ),
        ],
        ..Default::default()
    }
}

/// Trap the Trespassers — secret council for a creature you don't control:
/// a stun counter per vote, and each voted creature taps.
pub fn trap_the_trespassers() -> CardDefinition {
    spell(
        "Trap the Trespassers",
        cost(&[generic(2), u()]),
        true,
        Effect::SecretCouncilPermanentVote {
            filter: R::Creature.and(R::ControlledByYou.negate()),
            per_vote: Box::new(Effect::Seq(vec![
                Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                Effect::Tap { what: Selector::Target(0) },
            ])),
        },
    )
}

/// Travel Through Caradhras — council's dilemma: a basic land onto the
/// battlefield tapped per Redhorn Pass vote, a card back from your graveyard
/// per Mines of Moria vote; exile it.
pub fn travel_through_caradhras() -> CardDefinition {
    spell(
        "Travel Through Caradhras",
        cost(&[generic(5), g()]),
        false,
        Effect::Seq(vec![
            Effect::Vote {
                tally: VoteTally::PerVote,
                options: vec![
                    VoteOption::new(
                        "Redhorn Pass",
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: R::IsBasicLand,
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                        },
                    ),
                    VoteOption::new(
                        "Mines of Moria",
                        Effect::MoveChosen {
                            from: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Any },
                            filter: None,
                            count: Value::ONE,
                            up_to: false,
                            to: ZoneDest::Hand(PlayerRef::You),
                        },
                    ),
                ],
            },
            Effect::ExileResolvingSpell,
        ]),
    )
}

/// Windswift Slice — your creature deals damage equal to its power to a
/// creature you don't control; an Elf Warrior per point of excess damage.
pub fn windswift_slice() -> CardDefinition {
    spell(
        "Windswift Slice",
        cost(&[generic(2), g()]),
        true,
        Effect::Seq(vec![
            Effect::DealDamageFrom {
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou.negate()) },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
            elf_warriors(Value::ExcessDamageDealtThisResolution),
        ]),
    )
}
