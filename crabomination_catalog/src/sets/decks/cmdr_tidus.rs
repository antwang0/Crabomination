//! Commander: the cards the **Counter Blitz** precon (FIC, Tidus, Yuna's
//! Guardian) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_tidus.rs`.
//!
//! Residuals (each also on its card):
//! - **Endless Detour** — the kind of target (spell, permanent, graveyard
//!   card) is chosen as a mode.
//! - **Lulu, Stern Guardian** — the stunned attacker is chosen on
//!   resolution.
//! - **Rikku, Resourceful Guardian** — "can't be blocked by creatures your
//!   opponents control" is unblockable (the same in free-for-all).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, ExileReturnZone, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, investigate, on_you_attack, target_filtered};
use crate::effect::{
    CounteredSpellZone, Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, RevealMissDest, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w, x};
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

/// A Saga creature (the FIC "Summon:" cycle).
fn summon(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    chapters: Vec<(u32, Effect)>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        saga_chapters: chapters,
        ..Default::default()
    }
}

fn may(description: &str, body: Effect) -> Effect {
    Effect::MayDo { description: description.into(), body: Box::new(body) }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn gain(n: i32) -> Effect {
    Effect::GainLife { who: Selector::You, amount: Value::Const(n) }
}

fn plus(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn begin_combat_on_your_turn(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
        effect,
    }
}

fn your_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl), effect }
}

/// "Put a +1/+1 counter on each of up to `n` target creatures you control."
fn counters_on_up_to(n: u8, filter: R) -> Effect {
    Effect::ApplyToTargets {
        max_targets: n,
        min_targets: 0,
        filter,
        effect: Box::new(plus(Selector::Target(0), Value::ONE)),
    }
}

fn stun(what: Selector) -> Effect {
    Effect::AddCounter { what, kind: CounterType::Stun, amount: Value::ONE }
}

// ── Commander ───────────────────────────────────────────────────────────────

/// Tidus, Yuna's Guardian — each combat on your turn, move a counter between
/// two of your creatures; Cheer: your countered creatures connecting draw and
/// proliferate, once a turn.
pub fn tidus_yunas_guardian() -> CardDefinition {
    let mut cheer = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::WithAnyCounter) },
        ),
        effect: may("Draw a card and proliferate?", Effect::Seq(vec![draw(Value::ONE), Effect::Proliferate])),
    };
    cheer.event.once_per_turn = true;
    legendary(CardDefinition {
        triggered_abilities: vec![
            begin_combat_on_your_turn(may(
                "Move a counter from one creature you control onto another?",
                Effect::MoveOneCounter {
                    from: target_filtered(yours(R::Creature).and(R::WithAnyCounter)),
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: yours(R::Creature).and(R::OtherThanTargetSlot(0)),
                    },
                },
            )),
            cheer,
        ],
        ..creature(
            "Tidus, Yuna's Guardian",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    })
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Auron, Venerated Guardian — vigilance; attacking grows it, and when it
/// does, exiles a smaller creature of the defending player's until Auron
/// leaves.
pub fn auron_venerated_guardian() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                plus(Selector::This, Value::ONE),
                // CR 603.7 — "when you do" targets as the reflexive trigger is
                // put on the stack, after the counter.
                Effect::ReflexiveTrigger {
                    body: Box::new(Effect::ExileUntilSourceLeaves {
                        what: target_filtered(
                            R::Creature.and(R::ControlledByDefendingPlayer).and(R::PowerLessThanSource),
                        ),
                        return_to: ExileReturnZone::Battlefield,
                    }),
                },
            ]),
        }],
        ..creature(
            "Auron, Venerated Guardian",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Spirit, CreatureType::Samurai],
            2,
            5,
        )
    })
}

/// Chocobo Knights — whenever you attack, your creatures with counters gain
/// double strike until end of turn.
pub fn chocobo_knights() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_you_attack(Effect::GrantKeyword {
            what: Selector::EachPermanent(yours(R::Creature).and(R::WithAnyCounter)),
            keyword: Keyword::DoubleStrike,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Chocobo Knights", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Gatta and Luzzu — flash; on entry, damage to a creature you control this
/// turn is prevented and becomes +1/+1 counters.
pub fn gatta_and_luzzu() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PreventAllDamageThisTurnWithCounters {
            target: target_filtered(yours(R::Creature)),
        })],
        ..creature(
            "Gatta and Luzzu",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    })
}

/// Generous Patron — support 2 on entry; your counters on a creature you
/// don't control draw you a card.
pub fn generous_patron() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::SupportCounters { max_targets: 2, filter: R::Creature.and(R::OtherThanSource) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YouPutCounters).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ControlledByYou.negate()),
                    },
                ),
                effect: draw(Value::ONE),
            },
        ],
        ..creature("Generous Patron", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Advisor], 1, 4)
    }
}

/// Kimahri, Valiant Guardian — vigilance; each combat on your turn it grows,
/// taps an opponent's creature, and may become a copy of it (keeping its
/// name, vigilance and this ability).
pub fn kimahri_valiant_guardian() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![begin_combat_on_your_turn(Effect::Seq(vec![
            plus(Selector::This, Value::ONE),
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            may(
                "Have Kimahri become a copy of that creature?",
                Effect::BecomeCopyKeepingName {
                    what: Selector::This,
                    source: Selector::Target(0),
                    keywords: vec![Keyword::Vigilance],
                },
            ),
        ]))],
        ..creature(
            "Kimahri, Valiant Guardian",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            3,
            3,
        )
    })
}

/// Lord Jyscal Guado — flying; at each end step, investigate if you put a
/// counter on a creature this turn.
pub fn lord_jyscal_guado() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CounterPutOnCreatureThisTurn),
            effect: investigate(1),
        }],
        ..creature(
            "Lord Jyscal Guado",
            cost(&[generic(1), w()]),
            vec![CreatureType::Spirit, CreatureType::Cleric],
            2,
            1,
        )
    })
}

/// Lulu, Stern Guardian — an opponent attacking you stuns one of its
/// attackers; {3}{U}: proliferate.
///
/// ⚠ Residual: the attacker is chosen as the trigger resolves, not as it's
/// put on the stack.
pub fn lulu_stern_guardian() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).once_per_batch(),
            // The attack dispatch binds the attacking player into slot 0, so
            // the creature is picked as the trigger resolves.
            effect: Effect::Reflexive { body: Box::new(stun(target_filtered(R::Creature.and(R::IsAttackingYou)))) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Proliferate,
            ..Default::default()
        }],
        ..creature(
            "Lulu, Stern Guardian",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    })
}

/// Maester Seymour — each combat on your turn, counters equal to its power
/// on another creature you control; {3}{G}{G}: monstrosity X, X the counters
/// among your creatures.
pub fn maester_seymour() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![begin_combat_on_your_turn(plus(
            target_filtered(yours(R::Creature).and(R::OtherThanSource)),
            Value::PowerOf(Box::new(Selector::This)),
        ))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g(), g()]),
            effect: Effect::Monstrosity {
                n: Value::TotalCountersOn { what: Box::new(Selector::EachPermanent(yours(R::Creature))) },
            },
            ..Default::default()
        }],
        ..creature(
            "Maester Seymour",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Cleric],
            1,
            3,
        )
    })
}

/// O'aka, Traveling Merchant — {T}, remove a counter from a nonland
/// permanent you control: draw a card.
pub fn oaka_traveling_merchant() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_among_filter: Some((None, 1, R::Nonland)),
            effect: draw(Value::ONE),
            ..Default::default()
        }],
        ..creature(
            "O'aka, Traveling Merchant",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            1,
            2,
        )
    })
}

/// Rikku, Resourceful Guardian — your counters make a creature unblockable
/// this turn; Steal: move a counter from an opponent's creature to yours.
///
/// ⚠ Residual: "can't be blocked by creatures your opponents control" is
/// unblockable, which it is in free-for-all.
pub fn rikku_resourceful_guardian() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AnyCounterAdded, EventScope::YouPutCounters)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                .once_per_batch(),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            effect: Effect::MoveOneCounter {
                from: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::WithAnyCounter)),
                to: Selector::TargetFiltered { slot: 1, filter: yours(R::Creature) },
            },
            ..Default::default()
        }],
        ..creature(
            "Rikku, Resourceful Guardian",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            3,
        )
    })
}

/// Shelinda, Yevon Acolyte — lifelink; another creature of yours entering
/// gets a counter if it's smaller than Shelinda, else Shelinda does.
pub fn shelinda_yevon_acolyte() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerLessThanSource },
                then: Box::new(plus(Selector::TriggerSource, Value::ONE)),
                else_: Box::new(plus(Selector::This, Value::ONE)),
            },
        }],
        ..creature(
            "Shelinda, Yevon Acolyte",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    })
}

/// Sin, Unending Cataclysm — flying, trample; as it enters, strips the
/// counters from any number of artifacts, creatures and enchantments and
/// enters with twice that many; dying hands its counters to one of your
/// creatures and shuffles it into its owner's library.
pub fn sin_unending_cataclysm() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        as_enters_effect: Some(Effect::Seq(vec![
            Effect::RemoveAllCountersFromAnyNumber {
                filter: R::Artifact.or(R::Creature).or(R::Enchantment),
            },
            plus(Selector::This, Value::Times(Box::new(Value::Const(2)), Box::new(Value::CountersRemovedThisEffect))),
        ])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::MoveAllCounters { from: Selector::This, to: target_filtered(yours(R::Creature)) },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Library { who: PlayerRef::OwnerOf(Box::new(Selector::This)), pos: LibraryPosition::Shuffled },
                },
            ]),
        }],
        ..creature(
            "Sin, Unending Cataclysm",
            cost(&[generic(5), g(), u()]),
            vec![CreatureType::Leviathan, CreatureType::Avatar],
            5,
            5,
        )
    })
}

/// Tromell, Seymour's Butler — your other nontoken creatures enter with an
/// extra +1/+1 counter; {1}, {T}: proliferate once per nontoken creature of
/// yours that entered this turn.
pub fn tromell_seymours_butler() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each other nontoken creature you control enters with an additional +1/+1 counter on it.",
            effect: StaticEffect::MatchingEntersWithExtraCounters {
                filter: R::Creature.and(R::IsToken.negate()).and(R::OtherThanSource),
                kind: CounterType::PlusOnePlusOne,
                amount: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Repeat {
                count: Value::count(Selector::EachPermanent(
                    yours(R::Creature).and(R::IsToken.negate()).and(R::EnteredThisTurn),
                )),
                body: Box::new(Effect::Proliferate),
            },
            ..Default::default()
        }],
        ..creature(
            "Tromell, Seymour's Butler",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Advisor],
            2,
            3,
        )
    })
}

/// Wakka, Devoted Guardian — reach, trample; connecting breaks one of that
/// player's artifacts and grows Wakka; Blitzball Captain: a turn Wakka got a
/// counter, your other creatures get one at your end step.
pub fn wakka_devoted_guardian() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: R::Artifact.and(R::ControlledByTriggerPlayer),
                        effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    },
                    plus(Selector::This, Value::ONE),
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::SourceGainedCounterThisTurn),
                effect: plus(Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource)), Value::ONE),
            },
        ],
        ..creature(
            "Wakka, Devoted Guardian",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    })
}

/// Yuna, Grand Summoner — Grand Summon: {T} for any color, and your next
/// creature spell this turn enters with two more +1/+1 counters; another
/// permanent of yours dying with counters may pass that many on.
pub fn yuna_grand_summoner() -> CardDefinition {
    legendary(CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                Effect::GrantNextCreatureSpellCounters { kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::TriggerSourceHadCounters),
            effect: may(
                "Put that many +1/+1 counters on target creature?",
                plus(
                    target_filtered(R::Creature),
                    Value::TotalCountersOn { what: Box::new(Selector::TriggerSource) },
                ),
            ),
        }],
        ..creature(
            "Yuna, Grand Summoner",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            5,
        )
    })
}

// ── Summons (Saga creatures) ────────────────────────────────────────────────

/// Summon: Ixion — first strike. I: exile an opponent's creature until this
/// leaves. II, III: a counter on each of up to two of your creatures, gain 2.
pub fn summon_ixion() -> CardDefinition {
    let grow = || Effect::Seq(vec![counters_on_up_to(2, yours(R::Creature)), gain(2)]);
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..summon(
            "Summon: Ixion",
            cost(&[generic(2), w()]),
            vec![CreatureType::Unicorn],
            3,
            3,
            vec![
                (
                    1,
                    Effect::ExileUntilSourceLeaves {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                        return_to: ExileReturnZone::Battlefield,
                    },
                ),
                (2, grow()),
                (3, grow()),
            ],
        )
    }
}

/// Summon: Magus Sisters — haste. I, II, III: one at random of three
/// counters, a shield counter and 3 life, or a fight.
pub fn summon_magus_sisters() -> CardDefinition {
    let random = || {
        Effect::ChooseModeAtRandom(vec![
            plus(target_filtered(R::Creature), Value::Const(3)),
            Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::Shield, amount: Value::ONE },
                gain(3),
            ]),
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Fight { attacker: Selector::This, defender: Selector::Target(0) }),
            },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Haste],
        ..summon(
            "Summon: Magus Sisters",
            cost(&[generic(4), g()]),
            vec![CreatureType::Faerie],
            5,
            5,
            vec![(1, random()), (2, random()), (3, random())],
        )
    }
}

/// Summon: Valefor — flying. I: each opponent bounces their greatest-mana-
/// value creature. II, III, IV: tap up to one creature and stun it.
pub fn summon_valefor() -> CardDefinition {
    let stun_one = || Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: R::Creature,
        effect: Box::new(Effect::Seq(vec![Effect::Tap { what: Selector::Target(0) }, stun(Selector::Target(0))])),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..summon(
            "Summon: Valefor",
            cost(&[generic(4), u()]),
            vec![CreatureType::Drake],
            5,
            4,
            vec![
                (
                    1,
                    Effect::EachPlayerReturnsAMatchingPermanent {
                        filter: R::Creature.and(R::HasGreatestManaValueAmongControlled(Box::new(R::Creature))),
                        opponents: true,
                    },
                ),
                (2, stun_one()),
                (3, stun_one()),
                (4, stun_one()),
            ],
        )
    }
}

/// Summon: Yojimbo — vigilance. I: exile an opponent's artifact, enchantment
/// or tapped creature. II, III: Ghostly Prison until your next turn. IV: a
/// Treasure per opponent with a creature of power 4 or greater.
pub fn summon_yojimbo() -> CardDefinition {
    let tax = || Effect::TaxAttackersUntilYourNextTurn { amount: Value::Const(2) };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..summon(
            "Summon: Yojimbo",
            cost(&[generic(3), w()]),
            vec![CreatureType::Samurai],
            5,
            5,
            vec![
                (
                    1,
                    Effect::Move {
                        what: target_filtered(
                            R::Artifact
                                .or(R::Enchantment)
                                .or(R::Creature.and(R::Tapped))
                                .and(R::ControlledByOpponent),
                        ),
                        to: ZoneDest::Exile,
                    },
                ),
                (2, tax()),
                (3, tax()),
                (
                    4,
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::OpponentsControllingAnyOf(Box::new(Selector::EachPermanent(
                            R::Creature.and(R::PowerAtLeast(4)),
                        ))),
                        definition: Arc::new(treasure_token()),
                    },
                ),
            ],
        )
    }
}

// ── Noncreature spells ──────────────────────────────────────────────────────

/// Blitzball Stadium — support X on entry; Go for the Goal!: {3}, {T}: a
/// creature can't be blocked this turn and draws a card per kind of counter
/// on it when it connects.
pub fn blitzball_stadium() -> CardDefinition {
    CardDefinition {
        name: "Blitzball Stadium",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::CapTargetsAt {
            amount: Value::XFromCost,
            body: Box::new(counters_on_up_to(10, R::Creature)),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::GrantTriggeredAbility {
                    what: target_filtered(R::Creature),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                        effect: draw(Value::CounterKindsAmong { who: PlayerRef::You, filter: R::IsSource }),
                    }),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Endless Detour — the owner of a spell, nonland permanent or graveyard card
/// puts it on the top or bottom of their library.
///
/// ⚠ Residual: the kind of target is chosen as a mode.
pub fn endless_detour() -> CardDefinition {
    let to_library = |what: Selector| Effect::Move {
        what,
        to: ZoneDest::Library {
            who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
            pos: LibraryPosition::OwnerChoice,
        },
    };
    CardDefinition {
        name: "Endless Detour",
        cost: cost(&[g(), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpellToZone {
                what: target_filtered(R::IsSpellOnStack),
                zone: CounteredSpellZone::OwnerLibraryTopOrBottom,
            },
            to_library(target_filtered(R::Permanent.and(R::Nonland))),
            to_library(target_filtered(R::Any.from_any_graveyard())),
        ]),
        ..Default::default()
    }
}

/// Fight Rigging — hideaway 5; each combat on your turn, a counter on one of
/// your creatures, then with a power-7 creature you may play the hidden card
/// free.
pub fn fight_rigging() -> CardDefinition {
    CardDefinition {
        name: "Fight Rigging",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(5) }),
            begin_combat_on_your_turn(Effect::Seq(vec![
                plus(target_filtered(yours(R::Creature)), Value::ONE),
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(
                        yours(R::Creature).and(R::PowerAtLeast(7)),
                    )),
                    then: Box::new(may(
                        "Play the exiled card without paying its mana cost?",
                        Effect::CastWithoutPayingImmediate {
                            what: Selector::CardExiledWithSource,
                            source_zone: crate::card::Zone::Exile,
                            exile_after: false,
                            copy: false,
                            reduce_generic: 0,
                            pay_own_cost: false,
                        },
                    )),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        ],
        ..Default::default()
    }
}

/// Protection Magic — a shield counter on each of up to three creatures.
pub fn protection_magic() -> CardDefinition {
    CardDefinition {
        name: "Protection Magic",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Shield,
                amount: Value::ONE,
            }),
        },
        ..Default::default()
    }
}

/// Sphere Grid — your creatures connecting grow; your creatures with +1/+1
/// counters have reach and trample.
pub fn sphere_grid() -> CardDefinition {
    let unlock = |keyword: Keyword| StaticAbility {
        description: "Creatures you control with +1/+1 counters on them have reach and trample.",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(yours(R::Creature).and(R::WithCounter(CounterType::PlusOnePlusOne))),
            keyword,
        },
    };
    CardDefinition {
        name: "Sphere Grid",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: plus(Selector::TriggerSource, Value::ONE),
        }],
        static_abilities: vec![unlock(Keyword::Reach), unlock(Keyword::Trample)],
        ..Default::default()
    }
}

/// Summoner's Sending — at your end step, you may exile a creature card from
/// a graveyard for a 1/1 flying Spirit, with a counter if the card cost 4+.
pub fn summoners_sending() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Summoner's Sending",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![your_end_step(may(
            "Exile a creature card from a graveyard for a Spirit?",
            Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Creature.from_any_graveyard()), to: ZoneDest::Exile },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(spirit) },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::LastExiledManaValue, Value::Const(4)),
                    then: Box::new(plus(Selector::LastCreatedToken, Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        ))],
        ..Default::default()
    }
}

/// Yuna's Decision — choose one: sacrifice a creature to draw and put a
/// creature and/or land from hand onto the battlefield; or return one or two
/// permanent cards from your graveyard to your hand.
pub fn yunas_decision() -> CardDefinition {
    let deploy = |filter: R, what: &str| {
        may(
            &format!("Put a {what} card from your hand onto the battlefield?"),
            Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Hand, filter },
                filter: None,
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        )
    };
    CardDefinition {
        name: "Yuna's Decision",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::MaySacrifice {
                description: "Sacrifice a creature?".into(),
                filter: R::Creature,
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    draw(Value::ONE),
                    deploy(R::Creature, "creature"),
                    deploy(R::Land, "land"),
                ])),
                else_: None,
            },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::PermanentCard.from_your_graveyard(),
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
            },
        ]),
        ..Default::default()
    }
}

/// Yuna's Whistle — dig to a creature card for your hand, then X +1/+1
/// counters on one of your creatures, X its mana value.
pub fn yunas_whistle() -> CardDefinition {
    CardDefinition {
        name: "Yuna's Whistle",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(500),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::BottomRandom,
            },
            // CR 603.7 — "when you reveal a creature card this way": the
            // reflexive half runs inline so it can still read the find.
            Effect::If {
                cond: Predicate::SelectorExists(Selector::LastMoved),
                then: Box::new(Effect::Reflexive {
                    body: Box::new(plus(
                        target_filtered(yours(R::Creature)),
                        Value::ManaValueOf(Box::new(Selector::LastMoved)),
                    )),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
