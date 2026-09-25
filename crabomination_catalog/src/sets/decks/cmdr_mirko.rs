//! Commander: the cards the **Revenant Recon** precon (MKC, Mirko, Obsessive
//! Theorist) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_mirko.rs`.
//!
//! Residuals (each also on its card):
//! - **Marvo, Deep Operative** — "whenever you win a clash" rides its own
//!   attack clash (the deck's only clash) rather than triggering on any clash,
//!   and the clash is with the most hostile opponent, not the defending player.
//! - **Watcher of Hours** — removing the last time counter casts it at once, so
//!   that removal doesn't surveil.
//! - **Whispering Snitch** — "for the first time each turn" reads as once per
//!   turn, so a surveil before it entered doesn't use up the turn's trigger.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CaseData, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, SplitCard,
    SplitHalf, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{counter_target_spell, etb, investigate, on_attack, target_filtered, token_copy_of};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, hybrid, u, x, Color, ManaCost};

use super::super::tap_add_colorless;

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn surveil(n: i32) -> Effect {
    Effect::Surveil { who: PlayerRef::You, amount: Value::Const(n) }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

/// CR 701.42 — "whenever you surveil".
fn on_surveil(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Surveilled, EventScope::YourControl), effect }
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn plus_one_on_self() -> Effect {
    Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }
}

fn your_graveyard(filter: R) -> Selector {
    Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter }
}

/// "Return a creature card from your graveyard to the battlefield" — the
/// caster's pick, a headless seat taking the greatest power; `finality` adds
/// the finality counter (CR 122.1h) to the returned card.
fn reanimate_one(finality: bool) -> Effect {
    let back = Effect::MoveChosen {
        from: Selector::TakeGreatestPower {
            inner: Box::new(your_graveyard(R::Creature)),
            count: Box::new(Value::Const(i32::MAX)),
        },
        filter: None,
        count: Value::ONE,
        up_to: false,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    if !finality {
        return back;
    }
    Effect::Seq(vec![
        back,
        Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Finality, amount: Value::ONE },
    ])
}

fn ub() -> crate::mana::ManaSymbol {
    hybrid(Color::Blue, Color::Black)
}

// ── Commander ───────────────────────────────────────────────────────────────

/// Mirko, Obsessive Theorist — {1}{U}{B} Legendary Creature — Vampire
/// Detective 1/3. Flying, vigilance. Whenever you surveil, put a +1/+1 counter
/// on Mirko. At the beginning of your end step, you may return target creature
/// card with power less than Mirko's from your graveyard to the battlefield
/// with a finality counter on it.
pub fn mirko_obsessive_theorist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![
            on_surveil(plus_one_on_self()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Return a creature card with lesser power with a finality counter?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::PowerLessThanSource)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::Finality,
                            amount: Value::ONE,
                        },
                    ])),
                },
            },
        ],
        ..legend(
            "Mirko, Obsessive Theorist",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Vampire, CreatureType::Detective],
            1,
            3,
        )
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Marvo, Deep Operative — {3}{U}{B} Legendary Creature — Octopus Rogue 1/8.
/// Whenever Marvo attacks, clash with defending player. Whenever you win a
/// clash, draw a card, then you may cast a spell from your hand with mana
/// value 8 or less without paying its mana cost.
///
/// ⚠ Residual: the win payoff rides Marvo's own clash (the deck's only one),
/// and the clash is with the most hostile opponent.
pub fn marvo_deep_operative() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::ClashWithOpponent {
            on_win: Box::new(Effect::Seq(vec![
                draw(1),
                Effect::MayCastFromHandFreeMatching {
                    filter: R::Nonland,
                    max_mv: Value::Const(8),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend(
            "Marvo, Deep Operative",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Octopus, CreatureType::Rogue],
            1,
            8,
        )
    }
}

/// Copy Catchers — {1}{U} Creature — Faerie 2/1. Flying. Whenever you
/// surveil, you may pay {1}{U}. If you do, create a token that's a copy of
/// this creature.
pub fn copy_catchers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_surveil(Effect::MayPay {
            description: "Pay {1}{U} to copy Copy Catchers?".into(),
            mana_cost: cost(&[generic(1), u()]),
            body: Box::new(token_copy_of(PlayerRef::You, Value::ONE, Selector::This)),
            else_: None,
        })],
        ..creature("Copy Catchers", cost(&[generic(1), u()]), vec![CreatureType::Faerie], 2, 1)
    }
}

/// Final-Word Phantom — {2}{U} Creature — Spirit Detective 1/4. Flash, flying.
/// During each opponent's end step, you may cast spells as though they had
/// flash.
pub fn final_word_phantom() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "During each opponent's end step, you may cast spells as though they had flash.",
            effect: StaticEffect::WhileNotYourTurn {
                inner: Box::new(StaticEffect::WhileCondition {
                    condition: Predicate::CurrentStepIs(TurnStep::End),
                    inner: Box::new(StaticEffect::ControllerSpellsHaveFlash { filter: R::Any }),
                }),
            },
        }],
        ..creature(
            "Final-Word Phantom",
            cost(&[generic(2), u()]),
            vec![CreatureType::Spirit, CreatureType::Detective],
            1,
            4,
        )
    }
}

/// Watcher of Hours — {5}{U} Creature — Sphinx 6/6. Flying, ward {3}.
/// Whenever you remove a time counter from this card while it's exiled,
/// surveil 1. Suspend 6—{1}{U}.
///
/// ⚠ Residual: removing the last time counter casts it at once, so that
/// removal doesn't surveil.
pub fn watcher_of_hours() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Ward(WardCost::generic(3)),
            Keyword::Suspend(6, cost(&[generic(1), u()])),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::Time), EventScope::SelfSource)
                .while_suspended(),
            effect: surveil(1),
        }],
        ..creature("Watcher of Hours", cost(&[generic(5), u()]), vec![CreatureType::Sphinx], 6, 6)
    }
}

/// Eye of Duskmantle — {5}{B}{B} Creature — Eye 3/8. Flying, lifelink. You may
/// play lands and cast spells from among cards in your graveyard you've
/// surveilled this turn. If you cast a spell this way, you pay life equal to
/// its mana value rather than paying its mana cost.
pub fn eye_of_duskmantle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "You may play cards you surveilled into your graveyard this turn, paying life for spells.",
            effect: StaticEffect::MayPlaySurveilledThisTurnForLife,
        }],
        ..creature("Eye of Duskmantle", cost(&[generic(5), b(), b()]), vec![CreatureType::Eye], 3, 8)
    }
}

/// Unshakable Tail — {2}{B} Creature — Zombie Detective 3/2. When this
/// creature enters and at the beginning of your upkeep, surveil 1. Whenever
/// one or more creature cards are put into your graveyard from your library,
/// investigate. {2}, Sacrifice a Clue: Return this card from your graveyard to
/// your hand.
pub fn unshakable_tail() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(surveil(1)),
            your_upkeep(surveil(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardMilled, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                    .once_per_batch(),
                effect: investigate(1),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            from_graveyard: true,
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Clue), 1)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature(
            "Unshakable Tail",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Detective],
            3,
            2,
        )
    }
}

/// Sphinx of the Second Sun — {6}{U}{U} Creature — Sphinx 6/6. Flying. At the
/// beginning of each of your postcombat main phases, there is an additional
/// beginning phase after this phase.
pub fn sphinx_of_the_second_sun() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl),
            effect: Effect::AdditionalBeginningPhase { count: Value::ONE },
        }],
        ..creature("Sphinx of the Second Sun", cost(&[generic(6), u(), u()]), vec![CreatureType::Sphinx], 6, 6)
    }
}

/// Dimir Spybug — {U}{B} Creature — Insect 1/1. Flying, menace. Whenever you
/// surveil, put a +1/+1 counter on this creature.
pub fn dimir_spybug() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Menace],
        triggered_abilities: vec![on_surveil(plus_one_on_self())],
        ..creature("Dimir Spybug", cost(&[u(), b()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Thoughtbound Phantasm — {U} Creature — Spirit 2/2. Defender. Whenever you
/// surveil, put a +1/+1 counter on this creature. As long as it has three or
/// more +1/+1 counters on it, it can attack as though it didn't have defender.
pub fn thoughtbound_phantasm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![on_surveil(plus_one_on_self())],
        static_abilities: vec![StaticAbility {
            description: "With three or more +1/+1 counters, it can attack as though it didn't have defender.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::SourceHasCountersAtLeast { counter: CounterType::PlusOnePlusOne, n: 3 },
            },
        }],
        ..creature("Thoughtbound Phantasm", cost(&[u()]), vec![CreatureType::Spirit], 2, 2)
    }
}

/// Whispering Snitch — {1}{B} Creature — Vampire Rogue 1/3. Whenever you
/// surveil for the first time each turn, this creature deals 1 damage to each
/// opponent and you gain 1 life.
///
/// ⚠ Residual: read as once per turn, so a surveil before it entered doesn't
/// use up the turn's trigger.
pub fn whispering_snitch() -> CardDefinition {
    let mut trigger = on_surveil(Effect::Seq(vec![
        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        Effect::GainLife { who: Selector::You, amount: Value::ONE },
    ]));
    trigger.event.once_per_turn = true;
    CardDefinition {
        triggered_abilities: vec![trigger],
        ..creature(
            "Whispering Snitch",
            cost(&[generic(1), b()]),
            vec![CreatureType::Vampire, CreatureType::Rogue],
            1,
            3,
        )
    }
}

/// Lazav, the Multifarious — {U}{B} Legendary Creature — Shapeshifter 1/3.
/// When Lazav enters, surveil 1. {X}: Lazav becomes a copy of target creature
/// card in your graveyard with mana value X, except its name is Lazav, the
/// Multifarious, it's legendary in addition to its other types, and it has
/// this ability.
pub fn lazav_the_multifarious() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(surveil(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            effect: Effect::BecomeCopyKeepingIdentity {
                what: Selector::This,
                source: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueExactlyXFromCost)),
            },
            ..Default::default()
        }],
        ..legend(
            "Lazav, the Multifarious",
            cost(&[u(), b()]),
            vec![CreatureType::Shapeshifter],
            1,
            3,
        )
    }
}

// ── Noncreature permanents ──────────────────────────────────────────────────

/// Case of the Shifting Visage — {1}{U}{U} Enchantment — Case. At the
/// beginning of your upkeep, surveil 1. To solve — fifteen or more cards in
/// your graveyard. Solved — Whenever you cast a nonlegendary creature spell,
/// copy that spell.
pub fn case_of_the_shifting_visage() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Case], ..Default::default() },
        triggered_abilities: vec![your_upkeep(surveil(1))],
        case: Some(Box::new(CaseData {
            to_solve: Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(15)),
            solved_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(
                        R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
                    ),
                ),
                effect: Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
            }],
            ..Default::default()
        })),
        ..enchantment("Case of the Shifting Visage", cost(&[generic(1), u(), u()]))
    }
}

/// Foreboding Steamboat — {3}{B}{B} Artifact — Vehicle 5/7. When this Vehicle
/// enters, each player chooses two nontoken, non-Vehicle creatures they
/// control. Exile them until this Vehicle leaves the battlefield. Whenever
/// this Vehicle attacks, put a card exiled with it into its owner's graveyard.
/// If you do, investigate. Crew 2.
pub fn foreboding_steamboat() -> CardDefinition {
    let exiled = || Selector::CardExiledWithSource;
    CardDefinition {
        name: "Foreboding Steamboat",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Crew(2)],
        triggered_abilities: vec![
            etb(Effect::EachPlayerExilesChosenUntilSourceLeaves {
                count: Value::Const(2),
                filter: R::Creature
                    .and(R::Not(Box::new(R::IsToken)))
                    .and(R::Not(Box::new(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)))),
            }),
            on_attack(Effect::If {
                cond: Predicate::SelectorExists(exiled()),
                // An opponent's card first: it is theirs that stays gone.
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::take(
                            Selector::Both(
                                Box::new(Selector::MatchingAmong {
                                    inner: Box::new(exiled()),
                                    filter: R::Not(Box::new(R::OwnedByYou)),
                                }),
                                Box::new(exiled()),
                            ),
                            Value::ONE,
                        ),
                        to: ZoneDest::Graveyard,
                    },
                    investigate(1),
                ])),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..Default::default()
    }
}

/// Disinformation Campaign — {1}{U}{B} Enchantment. When it enters, you draw a
/// card and each opponent discards a card. Whenever you surveil, return it to
/// its owner's hand.
pub fn disinformation_campaign() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                draw(1),
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                    random: false,
                },
            ])),
            on_surveil(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        ],
        ..enchantment("Disinformation Campaign", cost(&[generic(1), u(), b()]))
    }
}

/// Enhanced Surveillance — {1}{U} Enchantment. You may look at an additional
/// two cards each time you surveil. Exile this enchantment: Shuffle your
/// graveyard into your library.
pub fn enhanced_surveillance() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may look at an additional two cards each time you surveil.",
            effect: StaticEffect::SurveilLooksExtra { n: 2 },
        }],
        activated_abilities: vec![ActivatedAbility {
            exile_self_cost: true,
            effect: Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::You },
            ..Default::default()
        }],
        ..enchantment("Enhanced Surveillance", cost(&[generic(1), u()]))
    }
}

/// Tocasia's Dig Site — Land. {T}: Add {C}. {3}, {T}: Surveil 1.
pub fn tocasias_dig_site() -> CardDefinition {
    CardDefinition {
        name: "Tocasia's Dig Site",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: surveil(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Instants and sorceries ──────────────────────────────────────────────────

/// Charnel Serenade — {4}{B}{B} Sorcery. Surveil 3, then return a creature
/// card from your graveyard to the battlefield with a finality counter on it.
/// Exile Charnel Serenade with three time counters on it. Suspend 3—{2}{B}.
pub fn charnel_serenade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Suspend(3, cost(&[generic(2), b()]))],
        ..spell(
            "Charnel Serenade",
            cost(&[generic(4), b(), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![surveil(3), reanimate_one(true), Effect::ExileSelfSuspended]),
        )
    }
}

/// Counterpoint — {3}{U}{B} Instant. Counter target spell. You may cast a
/// creature, instant, sorcery, or planeswalker spell from your graveyard with
/// mana value less than or equal to that spell's mana value without paying
/// its mana cost.
pub fn counterpoint() -> CardDefinition {
    let kinds = R::Creature
        .or(R::HasCardType(CardType::Instant))
        .or(R::HasCardType(CardType::Sorcery))
        .or(R::Planeswalker);
    spell(
        "Counterpoint",
        cost(&[generic(3), u(), b()]),
        CardType::Instant,
        // X is read before the counter, while that spell is on the stack.
        Effect::WithX {
            x: Value::ManaValueOf(Box::new(Selector::Target(0))),
            body: Box::new(Effect::Seq(vec![
                counter_target_spell(),
                Effect::CastWithoutPayingImmediate {
                    what: Selector::take(your_graveyard(kinds.and(R::ManaValueAtMostXFromCost)), Value::ONE),
                    source_zone: crate::card::Zone::Graveyard,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ])),
        },
    )
}

/// Mission Briefing — {U}{U} Instant. Surveil 2, then choose an instant or
/// sorcery card in your graveyard. You may cast it this turn. If that spell
/// would be put into your graveyard, exile it instead.
pub fn mission_briefing() -> CardDefinition {
    spell(
        "Mission Briefing",
        cost(&[u(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            surveil(2),
            Effect::GrantMayPlay {
                what: Selector::take(
                    your_graveyard(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                    Value::ONE,
                ),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: true,
                pay_own_cost: true,
                any_color: false,
            },
        ]),
    )
}

/// Notion Rain — {1}{U}{B} Sorcery. Surveil 2, then draw two cards. Notion
/// Rain deals 2 damage to you.
pub fn notion_rain() -> CardDefinition {
    spell(
        "Notion Rain",
        cost(&[generic(1), u(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            surveil(2),
            draw(2),
            Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Pile On — {3}{B} Instant. Convoke. Destroy target creature or
/// planeswalker. Surveil 2.
pub fn pile_on() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        ..spell(
            "Pile On",
            cost(&[generic(3), b()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
                surveil(2),
            ]),
        )
    }
}

/// Ephara's Dispersal — {2}{U} Instant. This spell costs {2} less to cast if it
/// targets an attacking creature. Return target creature to its owner's hand.
/// Surveil 2.
pub fn epharas_dispersal() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::IsAttacking, 2)),
        ..spell(
            "Ephara's Dispersal",
            cost(&[generic(2), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                surveil(2),
            ]),
        )
    }
}

/// Connive // Concoct — {2}{U/B}{U/B} Sorcery // {3}{U}{B} Sorcery. Connive:
/// gain control of target creature with power 2 or less. Concoct: surveil 3,
/// then return a creature card from your graveyard to the battlefield.
pub fn connive_concoct() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), u(), b()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![surveil(3), reanimate_one(false)]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..spell(
            "Connive // Concoct",
            cost(&[generic(2), ub(), ub()]),
            CardType::Sorcery,
            Effect::GainControl {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                to: None,
                duration: Duration::Permanent,
            },
        )
    }
}

/// Discovery // Dispersal — {1}{U/B} Sorcery // {3}{U}{B} Instant. Discovery:
/// surveil 2, then draw a card. Dispersal: each opponent returns a nonland
/// permanent they control with the greatest mana value among permanents they
/// control to its owner's hand, then discards a card.
pub fn discovery_dispersal() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), u(), b()]),
                card_types: vec![CardType::Instant],
                effect: Effect::EachPlayerDoes {
                    who: PlayerRef::EachOpponent,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::GreatestManaValueControlledMatching {
                                who: PlayerRef::You,
                                filter: R::Nonland,
                            },
                            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                        },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ])),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..spell(
            "Discovery // Dispersal",
            cost(&[generic(1), ub()]),
            CardType::Sorcery,
            Effect::Seq(vec![surveil(2), draw(1)]),
        )
    }
}
