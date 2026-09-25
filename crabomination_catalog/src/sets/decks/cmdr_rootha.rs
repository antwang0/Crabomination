//! Commander: the cards the **Prismari Artistry** precon (SOC, Rootha,
//! Mastering the Moment) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_rootha.rs`.
//!
//! Residuals (each also on its card):
//! - **Abstract Performance** — the "face-down" pile is exiled face up (only
//!   the chooser's prompt hides it); the chooser is the hostile opponent.
//! - **Plargg and Nassari** — the vetoing opponent is the hostile opponent,
//!   not one you choose.
//!
//! Surge to Victory and Redoubled Stormsinger are in this list too; they
//! landed first with Prismari Performance (`cmdr_zaffai`) and Mardu Surge
//! (`cmdr_zurgo`).

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LandType, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{cascade, etb, myriad, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, Color, ManaCost};

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

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn become_prepared() -> Effect {
    Effect::AddCounterCapped {
        what: Selector::This,
        kind: crate::card::CounterType::Prepared,
        amount: Value::ONE,
        cap: Value::ONE,
    }
}

/// "Whenever you cast an instant or sorcery spell [matching `extra`]".
fn on_is_cast(extra: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(instant_or_sorcery().and(extra))),
        effect,
    }
}

fn elemental(p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: "Elemental".into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn token_copy_of(source: Selector, enters_tapped: bool) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
    }
}

fn haste_eot(what: Selector) -> Effect {
    Effect::GrantKeyword { what, keyword: Keyword::Haste, duration: Duration::EndOfTurn }
}

fn island_mountain(name: &'static str) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Island, LandType::Mountain], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add(Color::Blue), crate::sets::tap_add(Color::Red)],
        ..Default::default()
    }
}

/// Rootha, Mastering the Moment — at the beginning of combat on your turn, if
/// you've cast an instant or sorcery this turn, create an X/X flying, haste
/// Elemental, X the greatest mana value among them.
pub fn rootha_mastering_the_moment() -> CardDefinition {
    let x = Value::GreatestInstantOrSorceryManaValueCastThisTurn(PlayerRef::You);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer)
                .with_filter(Predicate::ValueAtLeast(Value::InstantsOrSorceriesCastThisTurn(PlayerRef::You), Value::ONE)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    dynamic_pt: Some((x.clone(), x)),
                    ..elemental(0, 0, vec![Keyword::Flying, Keyword::Haste])
                }),
            },
        }],
        ..creature(
            "Rootha, Mastering the Moment",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Orc, CreatureType::Sorcerer],
            3,
            4,
        )
    }
}

/// Muddle, the Ever-Changing — casting an instant or sorcery turns it into a
/// copy of up to one target nonlegendary creature you control until end of
/// turn, except it has myriad.
pub fn muddle_the_ever_changing() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_is_cast(
            R::Any,
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: target_filtered(
                            R::Creature
                                .and(R::ControlledByYou)
                                .and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
                        ),
                        duration: Duration::EndOfTurn,
                        non_legendary: false,
                    },
                    Effect::GrantTriggeredAbility {
                        what: Selector::This,
                        trigger: Box::new(myriad()),
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        )],
        ..creature(
            "Muddle, the Ever-Changing",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Elemental, CreatureType::Otter, CreatureType::Shapeshifter],
            3,
            3,
        )
    }
}

/// Inspired Skypainter // Maestro's Gift — flying; entering, and creature
/// tokens of yours dealing combat damage to a player, prepare it. Maestro's
/// Gift copies a creature of yours as a hasty token.
pub fn inspired_skypainter() -> CardDefinition {
    let maestros_gift = CardDefinition {
        name: "Maestro's Gift",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            token_copy_of(target_filtered(R::Creature.and(R::ControlledByYou)), false),
            haste_eot(Selector::LastCreatedToken),
        ]),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(become_prepared()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::IsToken),
                    })
                    .once_per_batch(),
                effect: become_prepared(),
            },
        ],
        prepare_spell: Some(Arc::new(maestros_gift)),
        ..creature("Inspired Skypainter", cost(&[u(), r()]), vec![CreatureType::Lizard, CreatureType::Wizard], 2, 2)
    }
}

/// Abstract Performance — two piles of four, an opponent bins one; cast a
/// spell from the other free, the rest to hand. Residual: the "face-down"
/// pile is exiled face up; the chooser is the hostile opponent.
pub fn abstract_performance() -> CardDefinition {
    let kept = || Selector::ExiledThisResolution { filter: R::Any };
    CardDefinition {
        name: "Abstract Performance",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::FaceDownFaceUpPiles { count: 4 },
            Effect::CastAnyOrderWithoutPaying {
                what: kept(),
                source_zone: Zone::Exile,
                filter: None,
                cap: Some(Value::ONE),
                total_mana_value: None,
            },
            Effect::Move { what: kept(), to: ZoneDest::Hand(PlayerRef::You) },
        ]),
        ..Default::default()
    }
}

/// Dirgur Focusmage // Braingeyser — your instants and sorceries cost {1}
/// less; casting one with mana value 5+ from your hand prepares it.
pub fn dirgur_focusmage() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: instant_or_sorcery(), amount: 1 },
        }],
        triggered_abilities: vec![on_is_cast(
            R::ManaValueAtLeast(5).and(R::Not(Box::new(R::SpellNotCastFromHand))),
            become_prepared(),
        )],
        prepare_spell: Some(Arc::new(super::braingeyser())),
        ..creature("Dirgur Focusmage", cost(&[generic(2), u()]), vec![CreatureType::Djinn, CreatureType::Monk], 1, 4)
    }
}

/// Leitmotif Composer — combat damage to a player draws; an instant or sorcery
/// with mana value 5+ copies it; {2}{U}: Composers can't be blocked this turn.
pub fn leitmotif_composer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            on_is_cast(R::ManaValueAtLeast(5), token_copy_of(Selector::This, false)),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::GrantKeywordToMatchingThisTurn {
                filter: R::Creature.and(R::HasName("Leitmotif Composer".into())),
                keyword: Keyword::Unblockable,
            },
            ..Default::default()
        }],
        ..creature("Leitmotif Composer", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Bard], 2, 2)
    }
}

/// Furygale Flocking — {1} less per instant and sorcery card in your
/// graveyard; for each opponent, two 3/3 flying Elementals that attack that
/// opponent this turn if able, with haste until end of turn.
pub fn furygale_flocking() -> CardDefinition {
    CardDefinition {
        name: "Furygale Flocking",
        cost: cost(&[generic(8), r(), r()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each instant and sorcery card in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerGraveyardCardMatching { filter: instant_or_sorcery(), per: 1 },
        }],
        // One token at a time: `LastCreatedTokens` spans the resolution, so
        // a pair made for the second opponent would re-aim the first pair.
        effect: Effect::ForEachOpponent {
            body: Box::new(Effect::Seq(
                (0..2)
                    .flat_map(|_| {
                        [
                            Effect::CreateToken {
                                who: PlayerRef::You,
                                count: Value::ONE,
                                definition: Arc::new(elemental(3, 3, vec![Keyword::Flying])),
                            },
                            Effect::MustAttackPlayerThisTurn {
                                attacker: Selector::LastCreatedToken,
                                defender: Selector::Player(PlayerRef::Triggerer),
                            },
                            haste_eot(Selector::LastCreatedToken),
                        ]
                    })
                    .collect(),
            )),
        },
        ..Default::default()
    }
}

/// Prismari Pianist — each instant or sorcery you cast makes a 1/1 Elemental,
/// three if its mana value is 5 or greater.
pub fn prismari_pianist() -> CardDefinition {
    let make = |n: i32| Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(elemental(1, 1, vec![])),
    };
    CardDefinition {
        triggered_abilities: vec![
            on_is_cast(R::ManaValueAtLeast(5), make(3)),
            on_is_cast(R::Not(Box::new(R::ManaValueAtLeast(5))), make(1)),
        ],
        ..creature("Prismari Pianist", cost(&[generic(1), r(), r()]), vec![CreatureType::Djinn, CreatureType::Bard], 2, 1)
    }
}

/// Renegade Bull — trample; +X/+0 per instant or sorcery cast (X its mana
/// value); attacking exiles up to one instant or sorcery card from your
/// graveyard and casts a copy free.
pub fn renegade_bull() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            on_is_cast(
                R::Any,
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ),
            // "Up to one": with no instant or sorcery card the trigger simply
            // has no target and is removed (CR 603.3d).
            on_attack(Effect::Seq(vec![
                Effect::ExileWithSource { what: target_filtered(instant_or_sorcery().from_your_graveyard()) },
                Effect::CopyCardAndCastFree { what: Selector::Target(0) },
            ])),
        ],
        ..creature("Renegade Bull", cost(&[generic(4), r()]), vec![CreatureType::Ox], 0, 5)
    }
}

/// Coastal Peak — Island Mountain; enters tapped; cycling {2}.
pub fn coastal_peak() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..island_mountain("Coastal Peak")
    }
}

/// Molten Tributary — Island Mountain; enters tapped.
pub fn molten_tributary() -> CardDefinition {
    CardDefinition { static_abilities: vec![crate::sets::enters_tapped()], ..island_mountain("Molten Tributary") }
}

/// Turbulent Springs — Island Mountain; enters tapped unless your opponents
/// control eight or more lands.
pub fn turbulent_springs() -> CardDefinition {
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
        ..island_mountain("Turbulent Springs")
    }
}

/// Determined Iteration — at the beginning of combat on your turn, populate;
/// the token gains haste and is sacrificed at the next end step.
pub fn determined_iteration() -> CardDefinition {
    CardDefinition {
        name: "Determined Iteration",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            // Populate makes nothing without a creature token, and then there
            // is no "token created this way" to hasten or sacrifice.
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Creature.and(R::IsToken).and(R::ControlledByYou)),
                    n: Value::ONE,
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::Populate { who: PlayerRef::You },
                    haste_eot(Selector::LastCreatedToken),
                    Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Mirrorwing Dragon — flying; a player's instant or sorcery targeting only it
/// is copied for each other creature that player controls it could target.
pub fn mirrorwing_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: instant_or_sorcery().and(R::SpellTargetsOnlySource),
            }),
            effect: Effect::CopySpellForEachOtherLegalCreature {
                what: Selector::TriggerSource,
                casters_creatures: true,
            },
        }],
        ..creature("Mirrorwing Dragon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 4, 5)
    }
}

/// Plargg and Nassari — your upkeep: each player exiles until a nonland card;
/// an opponent vetoes one; cast up to two of the others free. Residual: the
/// vetoing opponent is the hostile one.
pub fn plargg_and_nassari() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::ExileTopUntilNonland { who: PlayerRef::You },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::ExileTopUntilNonland { who: PlayerRef::Triggerer }),
                },
                Effect::OpponentVetoesOne {
                    what: Selector::ExiledThisResolution { filter: R::Nonland },
                    then: Box::new(Effect::CastAnyOrderWithoutPaying {
                        what: Selector::SeparatedPile { chosen: false },
                        source_zone: Zone::Exile,
                        filter: None,
                        cap: Some(Value::Const(2)),
                        total_mana_value: None,
                    }),
                },
            ]),
        }],
        ..creature("Plargg and Nassari", cost(&[generic(3), r(), r()]), vec![CreatureType::Orc, CreatureType::Efreet], 5, 4)
    }
}

/// Volcanic Salvo — {X} less, X your creatures' total power; 6 damage to each
/// of up to two target creatures and/or planeswalkers.
pub fn volcanic_salvo() -> CardDefinition {
    CardDefinition {
        name: "Volcanic Salvo",
        cost: cost(&[generic(10), r(), r()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the total power of creatures you control.",
            effect: StaticEffect::SelfCostReducedByTotalPower,
        }],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(6) }),
        },
        ..Default::default()
    }
}

/// Throes of Chaos — cascade; retrace.
pub fn throes_of_chaos() -> CardDefinition {
    CardDefinition {
        name: "Throes of Chaos",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cascade, Keyword::Retrace],
        triggered_abilities: vec![cascade(4)],
        ..Default::default()
    }
}
