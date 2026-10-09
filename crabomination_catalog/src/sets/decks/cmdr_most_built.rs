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
            remove_keywords: vec![],
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
/// then it returns front face up (chapter III's permission covers a card put
/// into a graveyard later that turn too).
pub fn urabrask() -> CardDefinition {
    let any_graveyard_spells = Effect::CastFromGraveyardsThisTurn {
        any_graveyard: true,
        filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
        exile_after: true,
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
/// planeswalker and you gain X life. As an additional cost you may exile any
/// number of black cards from your hand, {2} less each.
pub fn march_of_wretched_sorrow() -> CardDefinition {
    CardDefinition {
        name: "March of Wretched Sorrow",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![crate::card::StaticAbility {
            description: "As an additional cost to cast this spell, you may exile any number of black cards from your hand. This spell costs {2} less to cast for each card exiled this way.",
            effect: crate::effect::StaticEffect::ExileFromHandCostReduction {
                per: 2,
                filter: R::HasColor(crate::mana::Color::Black),
            },
        }],
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

// ── Satoru Umezawa (UB) ─────────────────────────────────────────────────────

fn ninja_token() -> Arc<crate::card::TokenDefinition> {
    Arc::new(crate::card::TokenDefinition {
        name: "Ninja".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Ninja], ..Default::default() },
        keywords: vec![Keyword::Unblockable],
        ..Default::default()
    })
}

/// Satoru Umezawa — creature cards in your hand have ninjutsu {2}{U}{B};
/// activating a ninjutsu ability (once a turn) looks at the top three and
/// takes one, the rest to the bottom.
pub fn satoru_umezawa() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each creature card in your hand has ninjutsu {2}{U}{B}.",
            effect: StaticEffect::HandCreaturesHaveNinjutsu(cost(&[generic(2), u(), b()])),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(
                    EventKind::AbilityActivatedWith(crate::effect::ActivationTrait::Ninjutsu),
                    EventScope::YourControl,
                )
            },
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                ..Default::default()
            })),
        }],
        ..legend(
            "Satoru Umezawa",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Ninja],
            2,
            4,
        )
    }
}

/// Ancient Silver Dragon — flying; combat damage to a player rolls a d20 and
/// draws that many, and you have no maximum hand size for the rest of the game.
pub fn ancient_silver_dragon() -> CardDefinition {
    CardDefinition {
        name: "Ancient Silver Dragon",
        cost: cost(&[generic(6), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RollDie {
                    sides: 20,
                    count: Value::ONE,
                    modifier: Value::Const(0),
                    reroll_at_most: 0,
                    results: vec![(1, 20, Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount })],
                    ignore_lowest: 0,
                    on_doubles: None,
                },
                Effect::SetNoMaxHandSize { who: Selector::You },
            ]),
        }],
        ..Default::default()
    }
}

/// Gudul Lurker — can't be blocked; megamorph {U}.
pub fn gudul_lurker() -> CardDefinition {
    CardDefinition {
        name: "Gudul Lurker",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Salamander], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Megamorph(cost(&[u()])), Keyword::Unblockable],
        ..Default::default()
    }
}

/// Ingenious Infiltrator — ninjutsu {U}{B}; a Ninja of yours dealing combat
/// damage to a player draws you a card.
pub fn ingenious_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Ingenious Infiltrator",
        cost: cost(&[generic(2), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Ninjutsu(cost(&[u(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasCreatureType(CreatureType::Ninja) },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Mist-Cloaked Herald — can't be blocked.
pub fn mist_cloaked_herald() -> CardDefinition {
    CardDefinition {
        name: "Mist-Cloaked Herald",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        ..Default::default()
    }
}

/// Thousand-Faced Shadow — ninjutsu {2}{U}{U}, flying; entering from your
/// hand while attacking makes a tapped, attacking token copy of another
/// target attacking creature.
pub fn thousand_faced_shadow() -> CardDefinition {
    CardDefinition {
        name: "Thousand-Faced Shadow",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(2), u(), u()])), Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource).with_filter(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::EnteredFromHandThisTurn,
            }),
            // "If it's attacking": ninjutsu puts it into combat after the
            // move that fires this, so the check is read as it resolves.
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::This, filter: R::IsAttacking },
                then: Box::new(Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: true,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                    no_mana_cost: false,
                    enters_with_counters: None,
                    remove_keywords: vec![],
                },
                Effect::JoinCombatAttackingChosen {
                    what: Selector::LastCreatedTokens,
                    cleanup: crate::effect::AttackingTokenCleanup::None,
                },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Cunning Evasion — a creature of yours becoming blocked may return to its
/// owner's hand.
pub fn cunning_evasion() -> CardDefinition {
    CardDefinition {
        name: "Cunning Evasion",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Return the blocked creature to its owner's hand?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Kaito Shizuki — phases out at your end step the turn he entered; +1: draw,
/// then discard unless you attacked this turn; −2: an unblockable 1/1 Ninja;
/// −7: an emblem tutoring a blue or black creature onto the battlefield
/// whenever a creature of yours deals combat damage to a player.
pub fn kaito_shizuki() -> CardDefinition {
    CardDefinition {
        name: "Kaito Shizuki",
        cost: cost(&[generic(1), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: crate::card::Subtypes {
            planeswalker_subtypes: vec![crate::card::PlaneswalkerSubtype::Kaito],
            ..Default::default()
        },
        base_loyalty: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn }),
            effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
        }],
        loyalty_abilities: vec![
            crate::effect::LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
                        then: Box::new(Effect::Noop),
                        else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                    },
                ]),
                ..Default::default()
            },
            crate::effect::LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: ninja_token() },
                ..Default::default()
            },
            crate::effect::LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Kaito Shizuki".into(),
                    statics: vec![],
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
                        effect: Effect::Search {
                            who: PlayerRef::You,
                            filter: R::Creature.and(R::HasColor(Color::Blue).or(R::HasColor(Color::Black))),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Volo, Guide to Monsters (GU) ────────────────────────────────────────────

/// A creature card sharing no creature type with a creature you control.
fn new_creature_kind() -> R {
    R::Creature.and(R::Not(Box::new(R::SharesCreatureTypeWithCreatureYouControl)))
}

/// Volo, Guide to Monsters — a creature spell sharing no creature type with a
/// creature you control or a creature card in your graveyard is copied.
pub fn volo_guide_to_monsters() -> CardDefinition {
    let unseen = new_creature_kind().and(R::Not(Box::new(R::SharesCreatureTypeWithCreatureCardInYourGraveyard)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(unseen)),
            effect: Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..legend(
            "Volo, Guide to Monsters",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Volo, Itinerant Scholar — enters with Volo's Journal, which notes a new
/// creature type per creature spell; {2},{T} draws one per noted type.
pub fn volo_itinerant_scholar() -> CardDefinition {
    let journal = crate::card::TokenDefinition {
        name: "Volo's Journal".into(),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: crate::card::Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Book],
            ..Default::default()
        },
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::NoteCreatureTypeOf { what: Selector::TriggerSource },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::ChooseABackground],
        triggered_abilities: vec![etb(crate::effect::shortcut::mint_token(journal, 1))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::NotedCreatureTypesOf(Box::new(target_filtered(
                    R::ControlledByYou.and(R::HasName("Volo's Journal".into())),
                ))),
            },
            ..Default::default()
        }],
        ..legend(
            "Volo, Itinerant Scholar",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Radagast the Brown — a nontoken creature of yours entering digs its mana
/// value deep for a creature card of a type you don't have.
pub fn radagast_the_brown() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::NotToken) },
            ),
            effect: Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                pick_filter: Some(new_creature_kind()),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
        }],
        ..legend(
            "Radagast the Brown",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Avatar, CreatureType::Wizard],
            2,
            5,
        )
    }
}

/// Silverback Elder — each creature spell you cast: destroy an artifact or
/// enchantment, dig five for a tapped land, or gain 4 life.
pub fn silverback_elder() -> CardDefinition {
    CardDefinition {
        name: "Silverback Elder",
        cost: cost(&[generic(2), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Shaman],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::ChooseMode(vec![
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                Effect::DigForLandToBattlefield { count: Value::Const(5) },
                crate::effect::shortcut::gain_life(4),
            ]),
        }],
        ..Default::default()
    }
}

/// Dutiful Replicator — enters: you may pay {1}; when you do, copy target
/// token you control not named Dutiful Replicator.
pub fn dutiful_replicator() -> CardDefinition {
    CardDefinition {
        name: "Dutiful Replicator",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::AssemblyWorker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {1} to copy a token you control?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::ReflexiveTrigger {
                body: Box::new(crate::effect::shortcut::token_copy_of(
                    PlayerRef::You,
                    Value::ONE,
                    target_filtered(
                        R::ControlledByYou
                            .and(R::IsToken)
                            .and(R::Not(Box::new(R::HasName("Dutiful Replicator".into())))),
                    ),
                )),
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

// ── Dr. Eggman (BRU) ────────────────────────────────────────────────────────

fn robot_types() -> crate::card::Subtypes {
    crate::card::Subtypes { creature_types: vec![CreatureType::Robot, CreatureType::Villain], ..Default::default() }
}

/// Dr. Eggman — flying; at your end step draw, then each opponent discards or
/// lets you put a Construct, Robot, or Vehicle card from your hand onto the
/// battlefield (CR 701.55 villainous choice).
pub fn dr_eggman() -> CardDefinition {
    let me = PlayerRef::ControllerOf(Box::new(Selector::This));
    let deploy = Effect::EachPlayerDoes {
        who: me,
        body: Box::new(Effect::MayDo {
            description: "Put a Construct, Robot, or Vehicle card from your hand onto the battlefield?".into(),
            body: Box::new(Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Construct)
                    .or(R::HasCreatureType(CreatureType::Robot))
                    .or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Vehicle)),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            }),
        }),
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::VillainousChoice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    option_a: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                    option_b: Box::new(deploy),
                },
            ]),
        }],
        ..legend(
            "Dr. Eggman",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            3,
            6,
        )
    }
}

/// Blitzwing, Adaptive Assailant — living metal; flying or indestructible at
/// random each of your combats; converts after it connects.
pub fn blitzwing_adaptive_assailant() -> CardDefinition {
    CardDefinition {
        name: "Blitzwing, Adaptive Assailant",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: crate::card::Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        color_indicator: vec![Color::Black],
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::LivingMetal],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::ChooseModeAtRandom(
                    [Keyword::Flying, Keyword::Indestructible]
                        .into_iter()
                        .map(|keyword| Effect::GrantKeyword {
                            what: Selector::This,
                            keyword,
                            duration: crate::effect::Duration::EndOfTurn,
                        })
                        .collect(),
                ),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Transform { what: Selector::This },
            },
        ],
        ..Default::default()
    }
}

/// Blitzwing, Cruel Tormentor — at your end step target opponent loses the
/// life they lost this turn again; with none lost, it converts.
pub fn blitzwing_cruel_tormentor() -> CardDefinition {
    let lost = || Value::LifeLostThisTurn(PlayerRef::Target(0));
    CardDefinition {
        name: "Blitzwing, Cruel Tormentor",
        cost: cost(&[generic(5), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 6,
        toughness: 5,
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(3), b()]),
            converted: true,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(lost(), Value::ONE),
                then: Box::new(Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: lost() }),
                else_: Box::new(Effect::Transform { what: Selector::This }),
            },
        }],
        back_face: Some(Box::new(blitzwing_adaptive_assailant())),
        ..Default::default()
    }
}

/// Cityscape Leveler — cast and attack triggers destroy up to one nonland
/// permanent, whose controller gets a tapped Powerstone; unearth {8}.
pub fn cityscape_leveler() -> CardDefinition {
    let level = || Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: R::Nonland,
        effect: Box::new(Effect::DestroyThenVictimControllersMakeToken {
            what: Selector::Target(0),
            definition: Arc::new(crate::card::TokenDefinition {
                tapped: true,
                ..crabomination_base::tokens::powerstone_token()
            }),
            no_regen: false,
        }),
    };
    CardDefinition {
        name: "Cityscape Leveler",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: crate::card::Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            crate::effect::shortcut::on_cast(level()),
            crate::effect::shortcut::on_attack(level()),
        ],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(8)]))],
        ..Default::default()
    }
}

/// Krang, Utrom Warlord — a 9/9 that gives its four keywords to your other
/// artifact creatures.
pub fn krang_utrom_warlord() -> CardDefinition {
    let keywords = vec![Keyword::Flying, Keyword::Trample, Keyword::Indestructible, Keyword::Haste];
    CardDefinition {
        name: "Krang, Utrom Warlord",
        cost: cost(&[generic(9)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: crate::card::Subtypes {
            creature_types: vec![CreatureType::Utrom, CreatureType::Robot],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        keywords: keywords.clone(),
        static_abilities: vec![StaticAbility {
            description: "Other artifact creatures you control have flying, trample, indestructible, and haste.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                power: 0,
                toughness: 0,
                keywords,
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Ultron, Artificial Malevolence — another nontoken artifact of yours
/// entering may be copied for {2}; a noncreature copy is a 2/2 Robot Villain.
pub fn ultron_artificial_malevolence() -> CardDefinition {
    let animated = Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source: Selector::TriggerSource,
        extra_creature_types: vec![CreatureType::Robot, CreatureType::Villain],
        extra_card_types: vec![CardType::Creature],
        override_pt: Some((2, 2)),
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    };
    CardDefinition {
        name: "Ultron, Artificial Malevolence",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: robot_types(),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::NotToken) },
            ),
            effect: Effect::MayPay {
                description: "Pay {2} to copy that artifact?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                    then: Box::new(crate::effect::shortcut::token_copy_of(
                        PlayerRef::You,
                        Value::ONE,
                        Selector::TriggerSource,
                    )),
                    else_: Box::new(animated),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Ultron, Machine Overlord — flying; other Robots and Constructs you control
/// get +2/+2.
pub fn ultron_machine_overlord() -> CardDefinition {
    CardDefinition {
        name: "Ultron, Machine Overlord",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: robot_types(),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Robots and Constructs you control get +2/+2.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Robot)
                    .or(R::HasCreatureType(CreatureType::Construct))
                    .and(R::Creature)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                power: 2,
                toughness: 2,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Desynchronization — bounce every nonland permanent that isn't historic
/// (CR 700.6: artifacts, legendaries and Sagas are).
pub fn desynchronization() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Saga));
    CardDefinition {
        name: "Desynchronization",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::EachPermanent(R::Nonland.and(R::Not(Box::new(historic)))),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

// ── Omnath, Locus of All (WUBRG) ────────────────────────────────────────────

/// Omnath, Locus of All — lost unspent mana becomes black; each first main
/// phase the top card goes to hand, and one with three or more colored mana
/// symbols may be revealed for three mana of its colors first.
pub fn omnath_locus_of_all() -> CardDefinition {
    let top = Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would lose unspent mana, that mana becomes black instead.",
            effect: StaticEffect::UnspentManaBecomesBlack,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches { what: top.clone(), filter: R::ColoredManaSymbolsAtLeast(3) },
                    then: Box::new(Effect::MayDo {
                        description: "Reveal the top card for three mana of its colors?".into(),
                        body: Box::new(Effect::AddManaAmongColorsOf { what: top.clone(), amount: Value::Const(3) }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Move { what: top, to: ZoneDest::Hand(PlayerRef::You) },
            ]),
        }],
        ..legend(
            "Omnath, Locus of All",
            ManaCost::new(vec![
                crate::mana::w(),
                u(),
                crate::mana::phyrexian(Color::Black),
                r(),
                g(),
            ]),
            vec![CreatureType::Phyrexian, CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Chromatic Orrery — mana of yours spends as any color; {T}: {C}{C}{C}{C}{C};
/// {5}, {T}: draw a card per color among your permanents.
pub fn chromatic_orrery() -> CardDefinition {
    CardDefinition {
        name: "Chromatic Orrery",
        cost: cost(&[generic(7)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "You may spend mana as though it were mana of any color.",
            effect: StaticEffect::YouMaySpendManaAsAnyColor,
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(5)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::ControlledByYou))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wrenn and Realmbreaker — your lands tap for any color; +1 animates a land
/// until your next turn; −2 mills three for a permanent card; −7 emblem:
/// play lands and cast permanent spells from your graveyard.
pub fn wrenn_and_realmbreaker() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    use crate::effect::Duration;
    CardDefinition {
        name: "Wrenn and Realmbreaker",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: crate::card::Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Wrenn],
            ..Default::default()
        },
        base_loyalty: 4,
        static_abilities: vec![StaticAbility {
            description: "Lands you control have \"{T}: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::BecomeCreature {
                        what: target_filtered(R::Land.and(R::ControlledByYou)),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![Keyword::Vigilance, Keyword::Hexproof, Keyword::Haste],
                        duration: Duration::UntilNextTurn,
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::MillThenToHandN {
                    amount: Value::Const(3),
                    filter: R::PermanentCard,
                    take: Value::ONE,
                    otherwise: None,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Wrenn and Realmbreaker".into(),
                    statics: vec![
                        StaticAbility {
                            description: "You may play lands from your graveyard.",
                            effect: StaticEffect::MayPlayLandsFromGraveyard,
                        },
                        StaticAbility {
                            description: "You may cast permanent spells from your graveyard.",
                            effect: StaticEffect::CastFromGraveyardMatching { filter: R::PermanentCard.and(R::Nonland) },
                        },
                    ],
                    triggered: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Invasion of Alara // Awaken the Maelstrom — a defense-7 Siege whose entry
/// digs two cheap nonland cards (cast one free, keep one); defeated, its
/// all-colors sorcery back face is cast free (CR 310.12b).
pub fn invasion_of_alara() -> CardDefinition {
    let maelstrom = CardDefinition {
        name: "Awaken the Maelstrom",
        card_types: vec![CardType::Sorcery],
        color_indicator: vec![Color::White, Color::Blue, Color::Black, Color::Red, Color::Green],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::TargetFiltered { slot: 0, filter: R::Player }, amount: Value::Const(2) },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Artifact,
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
            Effect::ChooseOneAmong {
                what: Selector::EachPermanent(R::ControlledByYou),
                chooser: PlayerRef::You,
                chosen: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::SeparatedPile { chosen: true },
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
                    remove_keywords: vec![],
                }),
                other: Box::new(Effect::Noop),
            },
            Effect::DistributeCountersAmong {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                total: Value::Const(3),
                counter: CounterType::PlusOnePlusOne,
            },
            Effect::Destroy {
                what: Selector::TargetFiltered { slot: 1, filter: R::Permanent.and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Alara",
        cost: ManaCost::new(vec![crate::mana::w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Battle],
        subtypes: crate::card::Subtypes {
            battle_subtypes: vec![crate::card::BattleSubtype::Siege],
            ..Default::default()
        },
        defense: 7,
        triggered_abilities: vec![etb(Effect::ExileUntilCastOneTakeOne {
            count: 2,
            filter: R::Nonland.and(R::ManaValueAtMost(4)),
        })],
        back_face: Some(Box::new(maelstrom)),
        ..Default::default()
    }
}
