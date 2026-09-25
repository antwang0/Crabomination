//! Commander: the cards the **Lorehold Spirit** precon (SOC, Quintorius,
//! History Chaser) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_quintorius.rs`.
//!
//! Residuals (each also on its card):
//! - **Ao, the Dawn Sky** — the unpicked cards stay on top, not bottomed at
//!   random.
//! - **Quintorius, Loremaster** — the exiled card is cast as the ability
//!   resolves (not any time this turn), and goes to the graveyard, not the
//!   bottom of the library.
//! - **Serra Paragon** — lands from the graveyard aren't counted against the
//!   once-a-turn limit, and the dies-exile-gain-2 rider isn't granted.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, VoteOption, VoteTally, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, hybrid, r, w, Color, ManaCost};

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

fn mountain_plains(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Mountain, LandType::Plains], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add(Color::Red), crate::sets::tap_add(Color::White)],
        ..Default::default()
    }
}

fn rw_spirit() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Spirit".into(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    })
}

/// Balefire Liege — other red and other white creatures of yours get +1/+1
/// each; a red spell deals 3 to a player or planeswalker; a white spell gains
/// 3 life.
pub fn balefire_liege() -> CardDefinition {
    let lord = |c: Color, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::HasColor(c)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            power: 1,
            toughness: 1,
        },
    };
    let on_cast = |c: Color, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::HasColor(c))),
        effect,
    };
    CardDefinition {
        static_abilities: vec![
            lord(Color::Red, "Other red creatures you control get +1/+1."),
            lord(Color::White, "Other white creatures you control get +1/+1."),
        ],
        triggered_abilities: vec![
            on_cast(
                Color::Red,
                Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::Const(3) },
            ),
            on_cast(Color::White, Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
        ],
        ..creature(
            "Balefire Liege",
            cost(&[generic(2), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Spirit, CreatureType::Horror],
            2,
            4,
        )
    }
}

/// Glittering Massif — Mountain Plains; enters tapped; cycling {2}.
pub fn glittering_massif() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..mountain_plains("Glittering Massif")
    }
}

/// Sacred Peaks — Mountain Plains; enters tapped.
pub fn sacred_peaks() -> CardDefinition {
    CardDefinition { static_abilities: vec![crate::sets::enters_tapped()], ..mountain_plains("Sacred Peaks") }
}

/// Turbulent Steppe — Mountain Plains; enters tapped unless your opponents
/// control eight or more lands.
pub fn turbulent_steppe() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless your opponents control eight or more lands.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                    n: Value::Const(8),
                },
            },
        }],
        ..mountain_plains("Turbulent Steppe")
    }
}


fn left_your_graveyard(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl).once_per_batch(),
        effect,
    }
}

fn make_spirit() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: rw_spirit() }
}

fn spirits_you_control() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Spirit)).and(R::ControlledByYou))
}

fn your_graveyard(filter: R) -> Selector {
    Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter }
}

/// Quintorius, History Chaser — a planeswalker commander; cards leaving your
/// graveyard make a 3/2 Spirit; +1: rummage two, mill one; −4: Spirits get
/// double strike and vigilance.
pub fn quintorius_history_chaser() -> CardDefinition {
    CardDefinition {
        name: "Quintorius, History Chaser",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Quintorius], ..Default::default() },
        base_loyalty: 5,
        can_be_commander: true,
        triggered_abilities: vec![left_your_graveyard(make_spirit())],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::MayDiscard {
                    description: "Discard a card to draw two, then mill one?".into(),
                    count: Value::ONE,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                        Effect::Mill { who: Selector::You, amount: Value::ONE },
                    ])),
                    else_: None,
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword { what: spirits_you_control(), keyword: Keyword::DoubleStrike, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: spirits_you_control(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                ]),
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}

/// Advanced Reconstruction — Class. L1: your first main mills one, then
/// exiles a random graveyard card you may play this turn. L2: cards leaving
/// your graveyard deal 2 to each opponent. L3: spells from outside your hand
/// cost {2} less.
pub fn advanced_reconstruction() -> CardDefinition {
    let level_up = |from: u8| ActivatedAbility {
        mana_cost: cost(&[generic(1), r()]),
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        name: "Advanced Reconstruction",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::ActivePlayer),
                effect: Effect::Seq(vec![
                    Effect::Mill { who: Selector::You, amount: Value::ONE },
                    Effect::Move {
                        what: Selector::TakeRandom { inner: Box::new(your_graveyard(R::Any)), count: Box::new(Value::ONE) },
                        to: ZoneDest::Exile,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        any_color: false,
                        pay_own_cost: false,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                    .once_per_batch()
                    .with_filter(Predicate::SourceClassLevelAtLeast(2)),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast from anywhere other than your hand cost {2} less to cast.",
            effect: StaticEffect::WhileClassLevelAtLeast {
                n: 3,
                inner: Box::new(StaticEffect::NonHandCastCostReduction { amount: 2 }),
            },
        }],
        activated_abilities: vec![level_up(1), level_up(2)],
        ..Default::default()
    }
}

/// Ao, the Dawn Sky — flying, vigilance; dying picks: nonland permanents of
/// total mana value 4 or less from the top seven, or two +1/+1 counters on
/// each creature and Vehicle of yours. Residual: the rest stay on top.
pub fn ao_the_dawn_sky() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::MoveWithinTotalManaValue {
                    from: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(7) },
                    filter: R::Nonland.and(R::PermanentCard),
                    cap: Value::Const(4),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    max_count: None,
                },
                Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::Creature.or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Vehicle)).and(R::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..creature("Ao, the Dawn Sky", cost(&[generic(3), w(), w()]), vec![CreatureType::Dragon, CreatureType::Spirit], 5, 4)
    }
}

/// Augusta, Order Returned — flying, vigilance; attacking, each player exiles
/// a card from their graveyard, and an attacker gets a +1/+1 counter per
/// nonland card exiled.
pub fn augusta_order_returned() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::ExileFromGraveyard { who: PlayerRef::You, count: Value::ONE, filter: R::Any },
            Effect::ForEachOpponent {
                body: Box::new(Effect::ExileFromGraveyard { who: PlayerRef::Triggerer, count: Value::ONE, filter: R::Any }),
            },
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::ExiledThisResolution { filter: R::Nonland })),
            },
        ]))],
        ..creature("Augusta, Order Returned", cost(&[generic(2), w()]), vec![CreatureType::Spirit, CreatureType::Advisor], 1, 3)
    }
}

/// Ceaseless Conflict — destroy all creatures, then a 3/2 Spirit for each
/// nontoken creature of yours destroyed.
pub fn ceaseless_conflict() -> CardDefinition {
    CardDefinition {
        name: "Ceaseless Conflict",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Creature) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::DestroyedThisResolution {
                    filter: R::Creature.and(R::OwnedByYou),
                })),
                definition: rw_spirit(),
            },
        ]),
        ..Default::default()
    }
}

/// Currency Converter — a discarded card may be exiled with it; {2}, {T}:
/// loot; {T}: an exiled card goes to its owner's graveyard for a Treasure
/// (land) or a 2/2 Rogue (nonland).
pub fn currency_converter() -> CardDefinition {
    let rogue = TokenDefinition {
        name: "Rogue".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rogue], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Currency Converter",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::ExileWithSource { what: Selector::TriggerSource },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Take { inner: Box::new(Selector::CardExiledWithSource), count: Box::new(Value::ONE) },
                        to: ZoneDest::Graveyard,
                    },
                    Effect::If {
                        cond: Predicate::SelectorCountAtLeast {
                            sel: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: R::Land },
                            n: Value::ONE,
                        },
                        then: Box::new(Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::ONE,
                            definition: Arc::new(crabomination_base::tokens::treasure_token()),
                        }),
                        else_: Box::new(Effect::If {
                            cond: Predicate::SelectorCountAtLeast { sel: Selector::LastMoved, n: Value::ONE },
                            then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(rogue) }),
                            else_: Box::new(Effect::Noop),
                        }),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Drumbellower — flying; your creatures untap in each other player's untap
/// step too.
pub fn drumbellower() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Untap all creatures you control during each other player's untap step.",
            effect: StaticEffect::UntapYoursEachUntapStepFiltered(R::Creature),
        }],
        ..creature("Drumbellower", cost(&[generic(2), w()]), vec![CreatureType::Spirit], 2, 1)
    }
}

/// Excava, the Risen Past — flying, haste; attacking returns an artifact,
/// creature or non-Aura enchantment card with mana value 3 or less as a 1/1
/// flying Spirit with a finality counter.
pub fn excava_the_risen_past() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        // "Up to one": with no card the trigger has no target and is removed.
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Artifact
                            .or(R::Creature)
                            .or(R::Enchantment.and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)))))
                            .and(R::ManaValueAtMost(3))
                            .from_your_graveyard(),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Finality, amount: Value::ONE },
                Effect::BecomeCreature {
                    what: Selector::LastMoved,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    creature_types: vec![CreatureType::Spirit],
                    keywords: vec![Keyword::Flying],
                    duration: Duration::Permanent,
                },
            ]))],
        ..creature("Excava, the Risen Past", cost(&[generic(2), r(), w()]), vec![CreatureType::Spirit, CreatureType::Horse], 3, 3)
    }
}

/// Fateful Tempest — council's dilemma: each past vote mills one and deals
/// its mana value to each opponent; each present vote exiles one playable
/// until the end of your next turn.
pub fn fateful_tempest() -> CardDefinition {
    CardDefinition {
        name: "Fateful Tempest",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Vote {
            options: vec![
                VoteOption::new(
                    "past",
                    Effect::Seq(vec![
                        Effect::Mill { who: Selector::You, amount: Value::ONE },
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::TotalManaValueOf(Box::new(Selector::LastMoved)),
                        },
                    ]),
                ),
                VoteOption::new(
                    "present",
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: false,
                        uncast_penalty: None,
                    },
                ),
            ],
            tally: VoteTally::PerVote,
        },
        ..Default::default()
    }
}

/// Guardian of Faith — flash, vigilance; entering phases out any number of
/// your other creatures.
pub fn guardian_of_faith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            effect: Box::new(Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: false }),
        })],
        ..creature("Guardian of Faith", cost(&[generic(1), w(), w()]), vec![CreatureType::Spirit, CreatureType::Knight], 3, 2)
    }
}

/// Mistveil Plains — Plains; enters tapped; {W}, {T}: a card from your
/// graveyard to the bottom of your library, with two white permanents.
pub fn mistveil_plains() -> CardDefinition {
    CardDefinition {
        name: "Mistveil Plains",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Plains], ..Default::default() },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::White),
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasColor(Color::White).and(R::ControlledByYou)),
                    n: Value::Const(2),
                }),
                effect: Effect::Move {
                    what: target_filtered(R::Any.from_your_graveyard()),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Naktamun Lorespinner // Wheel of Fortune — your upkeep prepares it while a
/// player has one or fewer cards in hand.
pub fn naktamun_lorespinner() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer).with_filter(
                Predicate::Any(vec![
                    Predicate::ValueAtLeast(Value::OpponentsWithHandSizeAtMost(1), Value::ONE),
                    Predicate::ValueAtLeast(Value::Const(1), Value::HandSizeOf(PlayerRef::You)),
                ]),
            ),
            effect: Effect::AddCounterCapped { what: Selector::This, kind: CounterType::Prepared, amount: Value::ONE, cap: Value::ONE },
        }],
        prepare_spell: Some(Arc::new(crate::sets::lea::wheel_of_fortune())),
        ..creature("Naktamun Lorespinner", cost(&[generic(2), r()]), vec![CreatureType::Jackal, CreatureType::Wizard], 3, 3)
    }
}

/// Quintorius, Loremaster — vigilance; your end step exiles a noncreature,
/// nonland card from your graveyard and makes a Spirit; {1}{R}{W}, {T},
/// sacrifice a Spirit: cast a card exiled with it free. Residual: cast on
/// resolution, and it isn't bottomed afterward.
pub fn quintorius_loremaster() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::ExileWithSource {
                    what: target_filtered(R::Noncreature.and(R::Nonland).from_your_graveyard()),
                },
                make_spirit(),
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), w()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature.and(R::HasCreatureType(CreatureType::Spirit)), 1)),
            effect: Effect::CastWithoutPayingImmediate {
                what: Selector::Take { inner: Box::new(Selector::CardExiledWithSource), count: Box::new(Value::ONE) },
                source_zone: Zone::Exile,
                exile_after: false,
                pay_own_cost: false,
                copy: false,
                reduce_generic: 0,
            },
            ..Default::default()
        }],
        ..creature(
            "Quintorius, Loremaster",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Elephant, CreatureType::Cleric],
            3,
            5,
        )
    }
}

/// Relic Retriever — first strike; each end step after a card left your
/// graveyard this turn makes a Treasure.
pub fn relic_retriever() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                .with_filter(Predicate::CardsLeftGraveyardThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        }],
        ..creature("Relic Retriever", cost(&[generic(1), r()]), vec![CreatureType::Spirit, CreatureType::Monkey], 2, 1)
    }
}

/// Serra Paragon — flying; once each of your turns, cast a permanent spell
/// with mana value 3 or less from your graveyard; lands from it too.
/// Residual: lands don't share the limit, and the rider isn't granted.
pub fn serra_paragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Once during each of your turns, you may cast a permanent spell with mana value 3 or less from your graveyard.",
                effect: StaticEffect::GraveyardCastOncePerTurn {
                    mv_at_most_counters: None,
                    filter: R::PermanentCard.and(R::Nonland).and(R::ManaValueAtMost(3)),
                    exile_after: false,
                },
            },
            StaticAbility {
                description: "You may play a land from your graveyard.",
                effect: StaticEffect::MayPlayLandsFromGraveyard,
            },
        ],
        ..creature("Serra Paragon", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 3, 4)
    }
}

/// Spirit of Resilience — cards leaving your graveyard put a +1/+1 counter on
/// it; it may become a copy of an artifact or creature card among them until
/// end of turn.
pub fn spirit_of_resilience() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![left_your_graveyard(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.or(R::Creature) },
                then: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature("Spirit of Resilience", cost(&[generic(2), r()]), vec![CreatureType::Spirit, CreatureType::Warrior], 2, 2)
    }
}

/// Vanguard of the Restless — flying; your Spirits get +1/+1 per cast of your
/// commander from the command zone; a Spirit of yours entering may pay
/// {2}{W} to return it from your graveyard.
pub fn vanguard_of_the_restless() -> CardDefinition {
    let casts = Value::CommanderCastsFromCommandZone(PlayerRef::You);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Spirits you control get +1/+1 for each time you've cast your commander from the command zone this game.",
            effect: StaticEffect::PumpPTByValue { applies_to: spirits_you_control(), power: casts.clone(), toughness: casts },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::FromYourGraveyard).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Spirit)).and(R::ControlledByYou),
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {2}{W} to return Vanguard of the Restless?".into(),
                mana_cost: cost(&[generic(2), w()]),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: None,
            },
        }],
        ..creature("Vanguard of the Restless", cost(&[generic(2), w()]), vec![CreatureType::Spirit, CreatureType::Knight], 2, 2)
    }
}
