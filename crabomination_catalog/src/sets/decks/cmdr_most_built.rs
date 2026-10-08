//! Commander: most-built commanders missing from the catalog
//! (COMMANDER_BACKLOG §1), each with the cards its EDHREC average deck
//! needed beyond what the catalog had, seated as a pod deck. Tests in
//! `tests/recent_b/cmdr_most_built.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, if_discarded, magecraft, target_filtered, with_copies};
use crate::effect::{Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, x, Color, ManaCost};
use crabomination_base::tokens::treasure_token;
use std::sync::Arc;

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn keeps_red_mana() -> StaticAbility {
    StaticAbility {
        description: "You don't lose unspent red mana as steps and phases end.",
        effect: StaticEffect::UnspentColorManaPersists(Color::Red),
    }
}

// ── Ob Nixilis, Captive Kingpin (BR) ────────────────────────────────────────

/// Ob Nixilis, Captive Kingpin — flying, trample; one or more opponents each
/// losing exactly 1 life grows it and impulse-draws a card until your next
/// end step.
pub fn ob_nixilis_captive_kingpin() -> CardDefinition {
    CardDefinition {
        name: "Ob Nixilis, Captive Kingpin",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl).once_per_batch_losing_exactly(1),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    duration: MayPlayDuration::UntilYourNextEndStep,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mount Doom — legendary land: {T}, pay 1 life for {B} or {R}; {1}{B}{R},
/// {T}: 1 damage to each opponent; {5}{B}{R}, {T}, sacrifice it and a
/// legendary artifact: choose up to two creatures (not targets), destroy the
/// rest, as a sorcery.
pub fn mount_doom() -> CardDefinition {
    CardDefinition {
        name: "Mount Doom",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                life_cost: 1,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Black, Color::Red], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), r()]),
                tap_cost: true,
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), b(), r()]),
                tap_cost: true,
                sac_cost: true,
                sac_other_filter: Some((R::Artifact.and(R::HasSupertype(Supertype::Legendary)), 1)),
                sorcery_speed: true,
                effect: Effect::ChooseSomeAmong {
                    what: Selector::EachPermanent(R::Creature),
                    chooser: PlayerRef::You,
                    count: Value::Const(2),
                    up_to: true,
                    chosen: Box::new(Effect::Noop),
                    other: Box::new(Effect::Destroy { what: Selector::SeparatedPile { chosen: false } }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Manabarbs — whenever a player taps a land for mana, 1 damage to that
/// player (a separate trigger per land, not a mana ability).
pub fn manabarbs() -> CardDefinition {
    CardDefinition {
        name: "Manabarbs",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Shadow of the Goblin — first main phase: discard a card, and if you do,
/// draw one; a land played or a spell cast from anywhere but your hand deals
/// 1 damage to each opponent.
pub fn shadow_of_the_goblin() -> CardDefinition {
    let ping = || Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE };
    CardDefinition {
        name: "Shadow of the Goblin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    if_discarded(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                ]),
            },
            TriggeredAbility {
                // CR 305.1 — `LandPlayed`'s amount is 2 for a land PLAYED from
                // anywhere but the hand (1 from hand, 0 put onto the battlefield).
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl)
                    .with_filter(Predicate::ValueEquals(Value::TriggerEventAmount, Value::Const(2))),
                effect: ping(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::SpellNotCastFromHand },
                ),
                effect: ping(),
            },
        ],
        ..Default::default()
    }
}

// ── Storm, Force of Nature (GUR) ────────────────────────────────────────────

/// Storm, Force of Nature — flying, vigilance; combat damage to a player
/// gives the next instant or sorcery you cast this turn storm (CR 702.40).
pub fn storm_force_of_nature() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::NextInstantOrSorceryGainsStormThisTurn,
        }],
        ..legend(
            "Storm, Force of Nature",
            cost(&[generic(1), g(), u(), r()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            3,
            4,
        )
    }
}

/// Ashling, Flame Dancer — keeps unspent red mana; Magecraft: discard, then
/// draw; the second resolution this turn deals 2 to each opponent and each
/// creature they control, the third adds {R}{R}{R}{R}.
pub fn ashling_flame_dancer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![keeps_red_mana()],
        triggered_abilities: vec![with_copies(magecraft(Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::NthResolutionThisTurn {
                branches: vec![
                    Effect::Noop,
                    Effect::Seq(vec![
                        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                        Effect::DealDamage {
                            to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                            amount: Value::Const(2),
                        },
                    ]),
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red; 4]) },
                ],
            },
        ])))],
        ..legend(
            "Ashling, Flame Dancer",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            4,
            4,
        )
    }
}

/// Electro, Assaulting Battery — flying; keeps unspent red mana; an instant
/// or sorcery cast adds {R}; leaving the battlefield, you may pay {X}, and
/// when you do it deals X damage to target player.
pub fn electro_assaulting_battery() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![keeps_red_mana()],
        triggered_abilities: vec![
            magecraft(Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::MayPayX {
                    description: "Pay {X} to deal X damage to target player?".into(),
                    body: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::DealDamage {
                            to: target_filtered(R::Player),
                            amount: Value::XFromCost,
                        }),
                    }),
                },
            },
        ],
        ..legend(
            "Electro, Assaulting Battery",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Ice Storm — destroy target land.
pub fn ice_storm() -> CardDefinition {
    CardDefinition {
        name: "Ice Storm",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: target_filtered(R::Land) },
        ..Default::default()
    }
}

// ── Indoraptor, the Perfect Hybrid (BGR) ────────────────────────────────────

fn dinosaur() -> R {
    R::HasCreatureType(CreatureType::Dinosaur)
}

/// CR 702.130 — Enrage: "whenever this creature is dealt damage".
fn enrage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource), effect }
}

/// Indoraptor, the Perfect Hybrid — bloodthirst X (X = damage dealt to your
/// opponents this turn), menace; enrage: a random opponent sacrifices a
/// nontoken creature of their choice or takes damage equal to its power.
pub fn indoraptor_the_perfect_hybrid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::DamageTakenThisTurn(PlayerRef::EachOpponent))),
        triggered_abilities: vec![enrage(Effect::WithRandomOpponent {
            body: Box::new(Effect::Punisher {
                chooser: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                options: vec![Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::You),
                    count: Value::ONE,
                    filter: R::Creature.and(R::IsToken.negate()),
                }],
                otherwise: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                }),
            }),
        })],
        ..legend(
            "Indoraptor, the Perfect Hybrid",
            cost(&[generic(1), crate::mana::hybrid(Color::Black, Color::Green), r()]),
            vec![CreatureType::Dinosaur, CreatureType::Mutant],
            3,
            1,
        )
    }
}

/// Forerunner of the Empire — may tutor a Dinosaur to the top on entry; a
/// Dinosaur of yours entering may have it deal 1 damage to each creature.
pub fn forerunner_of_the_empire() -> CardDefinition {
    CardDefinition {
        name: "Forerunner of the Empire",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::MayDo {
                description: "Search your library for a Dinosaur card to put on top?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: dinosaur(),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: dinosaur() }),
                effect: Effect::MayDo {
                    description: "Deal 1 damage to each creature?".into(),
                    body: Box::new(Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::ONE }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Polyraptor — enrage: create a token that's a copy of it (2018-01-19
/// rulings: from last-known copiable values if it already died).
pub fn polyraptor() -> CardDefinition {
    CardDefinition {
        name: "Polyraptor",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![enrage(Effect::CreateTokenCopyOf {
            who: PlayerRef::You,
            count: Value::ONE,
            source: Selector::This,
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
            extra_keywords: vec![],
            no_mana_cost: false,
            enters_with_counters: None,
        })],
        ..Default::default()
    }
}

/// Silverclad Ferocidons — enrage: each opponent sacrifices a permanent of
/// their choice.
pub fn silverclad_ferocidons() -> CardDefinition {
    CardDefinition {
        name: "Silverclad Ferocidons",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 8,
        toughness: 5,
        triggered_abilities: vec![enrage(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Permanent,
        })],
        ..Default::default()
    }
}

// ── Shadow the Hedgehog (BR) ────────────────────────────────────────────────

/// Shadow the Hedgehog — haste; it or another creature of yours with flash
/// or haste dying draws a card; Chaos Control: your spells cast with
/// artifact mana have split second (CR 702.61).
pub fn shadow_the_hedgehog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Each spell you cast has split second if mana from an artifact was spent to cast it.",
            effect: StaticEffect::YourSpellsHaveSplitSecondIfArtifactManaSpent,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsSource.or(R::HasKeyword(Keyword::Flash)).or(R::HasKeyword(Keyword::Haste)),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Shadow the Hedgehog",
            cost(&[b(), b(), r(), r()]),
            vec![CreatureType::Hedgehog, CreatureType::Mercenary],
            4,
            2,
        )
    }
}

/// Knuckles the Echidna — double strike, trample, haste; combat damage to a
/// player by one or more of your creatures makes a Treasure; an upkeep with
/// thirty or more artifacts wins the game.
pub fn knuckles_the_echidna() -> CardDefinition {
    let thirty = || {
        Predicate::ValueAtLeast(
            Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
            Value::Const(30),
        )
    };
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treasure_token()) },
            },
            TriggeredAbility {
                // CR 603.4 — intervening "if": checked as it triggers and again
                // as it resolves.
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(thirty()),
                effect: Effect::If {
                    cond: thirty(),
                    then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..legend(
            "Knuckles the Echidna",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Echidna, CreatureType::Warrior],
            2,
            4,
        )
    }
}

/// Lagomos, Hand of Hatred — a hasty 2/1 trampling Elemental each combat on
/// your turn, sacrificed at the next end step; {T}: tutor, only once five
/// creatures died this turn.
pub fn lagomos_hand_of_hatred() -> CardDefinition {
    let elemental = Arc::new(crate::card::TokenDefinition {
        name: "Elemental".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        keywords: vec![Keyword::Trample, Keyword::Haste],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elemental },
                Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(Value::CreaturesDiedThisTurnTotal, Value::Const(5))),
            effect: Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..legend(
            "Lagomos, Hand of Hatred",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            3,
        )
    }
}

/// Smaug, Wicked Worm — flying; enters with a tapped Treasure per artifact
/// your opponents control; a spell cast with Treasure mana draws you a card
/// and costs you 1 life.
pub fn smaug_wicked_worm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent)))),
                definition: Arc::new(crate::card::TokenDefinition { tapped: true, ..treasure_token() }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastWithTreasureMana { what: Selector::TriggerSource }),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
        ],
        ..legend("Smaug, Wicked Worm", cost(&[generic(3), b(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Super State — Aura (enchant creature you control): base 9/9 with flying,
/// first strike, trample and haste; its combat damage to an opponent is dealt
/// again to each other opponent.
pub fn super_state() -> CardDefinition {
    CardDefinition {
        name: "Super State",
        cost: cost(&[generic(7)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: crate::card::Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            set_base_pt: Some((9, 9)),
            keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Trample, Keyword::Haste],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::DealDamageFrom {
                    source: Selector::This,
                    to: Selector::Player(PlayerRef::EachOpponentExceptTriggerer),
                    amount: Value::TriggerEventAmount,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Ojer Axonil, Deepest Might (R) ──────────────────────────────────────────

/// Ojer Axonil, Deepest Might // Temple of Power — trample; a red source of
/// yours deals an opponent at least Ojer's power in noncombat damage; dies
/// into the Temple, which taps for {R} and transforms back for {2}{R} once
/// your red sources dealt 4+ noncombat damage this turn (sorcery speed).
pub fn ojer_axonil_deepest_might() -> CardDefinition {
    let temple = CardDefinition {
        name: "Temple of Power",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), r()]),
                sorcery_speed: true,
                condition: Some(Predicate::ValueAtLeast(
                    Value::RedNoncombatDamageDealtThisTurn(PlayerRef::You),
                    Value::Const(4),
                )),
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "If a red source you control would deal an amount of noncombat damage less than Ojer Axonil's power to an opponent, that source deals damage equal to Ojer Axonil's power instead.",
            effect: StaticEffect::RaiseRedNoncombatDamageToOpponentsToPower,
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::ReturnSelfTransformedTappedToOwner)],
        back_face: Some(Box::new(temple)),
        ..legend("Ojer Axonil, Deepest Might", cost(&[generic(2), r(), r()]), vec![CreatureType::God], 4, 4)
    }
}

/// Chandra's Incinerator — costs {X} less (X = noncombat damage dealt to your
/// opponents this turn); trample; a source of yours dealing an opponent
/// noncombat damage has it deal that much to a creature or planeswalker
/// that player controls.
pub fn chandras_incinerator() -> CardDefinition {
    CardDefinition {
        name: "Chandra's Incinerator",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the total amount of noncombat damage dealt to your opponents this turn.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::NoncombatDamageTakenThisTurn(PlayerRef::EachOpponent),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::YourSourceDamagedOpponent),
            effect: Effect::DealDamage {
                to: target_filtered(
                    R::Creature.or(R::HasCardType(CardType::Planeswalker)).and(R::ControlledByTriggerPlayer),
                ),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Defiler of Instinct — first strike; red permanent spells may pay 2 life
/// for {R}; casting one deals 1 damage to any target.
pub fn defiler_of_instinct() -> CardDefinition {
    let red_permanent = || R::HasColor(Color::Red).and(R::PermanentCard);
    CardDefinition {
        name: "Defiler of Instinct",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Kavu],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "Red permanent spells may pay 2 life for {R}.",
            effect: StaticEffect::PhyrexianPipForSpells { filter: red_permanent(), color: Color::Red },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(red_permanent())),
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Urabrask // The Great Work — first strike; an instant or sorcery cast
/// pings target opponent for 1 and adds {R}; {R}, after three such spells
/// this turn: exile it and return it transformed (sorcery speed). The Saga:
/// 3 to target opponent and each creature they control; three Treasures;
/// cast instants and sorceries from any graveyard this turn (exiled after),
/// then it returns front face up. Chapter III is approximated: the permission
/// is stamped on the cards in graveyards as it resolves, so one put there
/// later that turn is not castable.
pub fn urabrask() -> CardDefinition {
    let any_graveyard_spells = Effect::GrantMayPlay {
        what: Selector::CardsInZone {
            who: PlayerRef::EachPlayer,
            zone: crate::card::Zone::Graveyard,
            filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        },
        duration: MayPlayDuration::EndOfThisTurn,
        to_owner: false,
        exile_after: true,
        pay_own_cost: true,
        any_color: false,
    };
    let saga = CardDefinition {
        name: "The Great Work",
        card_types: vec![CardType::Enchantment],
        subtypes: crate::card::Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::Const(3) },
                    Effect::DealDamage {
                        to: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                        amount: Value::Const(3),
                    },
                ]),
            ),
            (2, Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: Arc::new(treasure_token()) }),
            (3, Effect::Seq(vec![any_graveyard_spells, Effect::ExileSelfReturnFrontFace])),
        ],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::ONE },
            Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sorcery_speed: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::InstantsOrSorceriesCastThisTurn(PlayerRef::You),
                Value::Const(3),
            )),
            effect: Effect::ExileSelfReturnTransformed,
            ..Default::default()
        }],
        back_face: Some(Box::new(saga)),
        ..legend("Urabrask", cost(&[generic(2), r(), r()]), vec![CreatureType::Phyrexian, CreatureType::Praetor], 4, 4)
    }
}

/// Burning Earth — a nonbasic land tapped for mana deals its tapper 1 damage.
pub fn burning_earth() -> CardDefinition {
    CardDefinition {
        name: "Burning Earth",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Land.and(R::IsBasicLand.negate()),
            }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Virtue of Courage // Embereth Blaze — a source of yours dealing an
/// opponent noncombat damage may exile that many cards from your library's
/// top to play this turn. Adventure: 2 damage to any target.
pub fn virtue_of_courage() -> CardDefinition {
    CardDefinition {
        name: "Virtue of Courage",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Embereth Blaze",
            cost: cost(&[generic(1), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(2) },
        })),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::YourSourceDamagedOpponent),
            effect: Effect::MayDo {
                description: "Exile that many cards from the top of your library to play this turn?".into(),
                body: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::TriggerEventAmount,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: true,
                    uncast_penalty: None,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Rowan, Scion of War (BR) ────────────────────────────────────────────────

fn black_or_red() -> R {
    R::HasColor(Color::Black).or(R::HasColor(Color::Red))
}

/// Rowan, Scion of War — menace; {T}: black and/or red spells you cast this
/// turn cost {X} less, X the life you lost this turn as it resolves
/// (2023-09-01 rulings). Sorcery speed.
pub fn rowan_scion_of_war() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::SpellsCostLessThisTurnByValue {
                filter: black_or_red(),
                amount: Value::TotalLifeLostThisTurn(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..legend(
            "Rowan, Scion of War",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            2,
        )
    }
}

/// Vilis, Broker of Blood — flying; {B}, pay 2 life: target creature gets
/// -1/-1; whenever you lose life, draw that many cards.
pub fn vilis_broker_of_blood() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 2,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..legend("Vilis, Broker of Blood", cost(&[generic(5), b(), b(), b()]), vec![CreatureType::Demon], 8, 8)
    }
}

/// March of Wretched Sorrow — {X}{B}: X damage to target creature or
/// planeswalker and you gain X life. (The optional "exile black cards from
/// your hand, {2} less each" additional cost is omitted.)
pub fn march_of_wretched_sorrow() -> CardDefinition {
    CardDefinition {
        name: "March of Wretched Sorrow",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature.or(R::HasCardType(CardType::Planeswalker))),
                amount: Value::XFromCost,
            },
            Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Battle at the Bridge — {X}{B} sorcery with improvise: target creature
/// gets -X/-X and you gain X life.
pub fn battle_at_the_bridge() -> CardDefinition {
    CardDefinition {
        name: "Battle at the Bridge",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Improvise],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Negate(Box::new(Value::XFromCost)),
                toughness: Value::Negate(Box::new(Value::XFromCost)),
                duration: crate::effect::Duration::EndOfTurn,
            },
            Effect::GainLife { who: Selector::You, amount: Value::XFromCost },
        ]),
        ..Default::default()
    }
}

/// Inspired Tinkering — exile your top three to play until the end of your
/// next turn, and create three Treasures.
pub fn inspired_tinkering() -> CardDefinition {
    CardDefinition {
        name: "Inspired Tinkering",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(3),
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: Arc::new(treasure_token()) },
        ]),
        ..Default::default()
    }
}

/// Peer into the Abyss — target player draws half their library and loses
/// half their life, each rounded up (2020-06-23 rulings).
pub fn peer_into_the_abyss() -> CardDefinition {
    CardDefinition {
        name: "Peer into the Abyss",
        cost: cost(&[generic(4), b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(R::Player),
                amount: Value::HalfLibrarySizeRoundedUp(PlayerRef::Target(0)),
            },
            Effect::LoseHalfLife { who: Selector::Player(PlayerRef::Target(0)), rounded_up: true },
        ]),
        ..Default::default()
    }
}

// ── Deadpool, Trading Card (BR) ─────────────────────────────────────────────

/// Deadpool, Trading Card — as it enters, you may exchange its text box with
/// another creature's (CR 612 / 613.1c); your upkeep costs you 3 life; {3},
/// sacrifice it: each other player draws a card.
pub fn deadpool_trading_card() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::MayDo {
            description: "Exchange Deadpool's text box with another creature's?".into(),
            body: Box::new(Effect::ChooseOneAmong {
                what: Selector::EachPermanent(R::Creature.and(R::OtherThanSource)),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::ExchangeTextBoxes {
                    a: Selector::This,
                    b: Selector::SeparatedPile { chosen: true },
                }),
                other: Box::new(Effect::Noop),
            }),
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            ..Default::default()
        }],
        ..legend(
            "Deadpool, Trading Card",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Mutant, CreatureType::Mercenary, CreatureType::Hero],
            5,
            3,
        )
    }
}

/// Elturel Survivors — trample, myriad; while attacking, +X/+0 where X is
/// the number of lands the defending player controls.
pub fn elturel_survivors() -> CardDefinition {
    CardDefinition {
        name: "Elturel Survivors",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Tiefling, CreatureType::Peasant],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is attacking, it gets +X/+0, where X is the number of lands defending player controls.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                inner: Box::new(StaticEffect::PumpSelfByValue {
                    amount: Value::CountOf(Box::new(Selector::ControlledBy { who: PlayerRef::DefendingPlayer, filter: R::Land })),
                    per_power: 1,
                    per_toughness: 0,
                }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Myriad,
        }],
        ..Default::default()
    }
}
