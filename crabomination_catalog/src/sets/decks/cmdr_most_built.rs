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
use crate::mana::{b, cost, g, generic, r, u, Color, ManaCost};

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
