//! Commander: the cards the **Seize Control** precon (C15, Mizzix of the
//! Izmagnus) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EntersAsCopy, Keyword,
    MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cascade, myriad, target_filtered, target_n};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LookPick, PlayerRef, Predicate, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, ManaSymbol, cost, generic, hybrid, r, u, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

/// Vivid Crag — tapped, two charge counters; {R}, or any colour off a counter.
pub fn vivid_crag() -> CardDefinition {
    super::cmdr_aesi::vivid("Vivid Crag", Color::Red)
}

/// Jace's Archivist — {U}, {T}: each player discards their hand, then draws
/// as many as the most anyone discarded (Windfall on a stick).
pub fn jaces_archivist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(100),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::MaxCardsDiscardedThisEffectByAnyPlayer,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Jace's Archivist",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Desperate Ravings — draw two, then discard at random; flashback {2}{U}.
pub fn desperate_ravings() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(ManaCost {
            symbols: vec![ManaSymbol::Generic(2), ManaSymbol::Colored(Color::Blue)],
        })],
        ..spell(
            "Desperate Ravings",
            cost(&[generic(1), r()]),
            true,
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
            ]),
        )
    }
}

/// Call the Skybreaker — a 5/5 blue-red flying Elemental; retrace.
pub fn call_the_skybreaker() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..spell(
            "Call the Skybreaker",
            cost(&[generic(5), ur(), ur()]),
            false,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Elemental".into(),
                    power: 5,
                    toughness: 5,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue, Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elemental],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                }),
            },
        )
    }
}

/// Etherium-Horn Sorcerer — cascade; {1}{U}{R}: return it to its owner's hand.
pub fn etherium_horn_sorcerer() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![cascade(6)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u(), r()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..creature(
            "Etherium-Horn Sorcerer",
            cost(&[generic(4), u(), r()]),
            vec![CreatureType::Minotaur, CreatureType::Wizard, CreatureType::Sorcerer],
            3,
            6,
        )
    }
}

fn red_token(name: &str, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// Aethersnatch — gain control of target spell; you may choose new targets.
pub fn aethersnatch() -> CardDefinition {
    spell(
        "Aethersnatch",
        cost(&[generic(4), u(), u()]),
        true,
        Effect::Seq(vec![
            Effect::GainControlOfSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ChooseNewTargetsForSpell { what: target_n(0) },
        ]),
    )
}

/// Awaken the Sky Tyrant — when an opponent's source damages you, sacrifice
/// it; if you do, a 5/5 red flying Dragon.
pub fn awaken_the_sky_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Awaken the Sky Tyrant",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou),
            // Only the first of several triggers finds it to sacrifice.
            effect: Effect::If {
                cond: Predicate::SourceOnBattlefield,
                then: Box::new(Effect::Seq(vec![
                    Effect::SacrificeSource,
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: Arc::new(red_token(
                            "Dragon",
                            vec![CreatureType::Dragon],
                            5,
                            5,
                            vec![Keyword::Flying],
                        )),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Meteor Blast — 4 damage to each of X targets.
pub fn meteor_blast() -> CardDefinition {
    spell(
        "Meteor Blast",
        cost(&[x(), r(), r(), r()]),
        false,
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 8,
                filter: R::Creature.or(R::Planeswalker).or(R::Player),
                effect: Box::new(Effect::DealDamage { to: target_n(0), amount: Value::Const(4) }),
            }),
        },
    )
}

/// Mystic Confluence — choose three, repeats allowed: counter unless {3},
/// bounce a creature, draw. The default picks are counter-unless-{3} and draw
/// two, so it is cast as an answer to a spell.
pub fn mystic_confluence() -> CardDefinition {
    spell(
        "Mystic Confluence",
        cost(&[generic(3), u(), u()]),
        true,
        Effect::ChooseN {
            picks: vec![0, 2, 2],
            modes: vec![
                Effect::CounterUnlessPaid {
                    what: target_filtered(R::IsSpellOnStack),
                    mana_cost: cost(&[generic(3)]),
                    exile: false,
                    extra_generic: None,
                    if_paid: None,
                },
                Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ],
        },
    )
}

/// Rite of the Raging Storm — each upkeep, that player gets a 5/1 trample
/// haste Lightning Rager that dies at end step; Ragers can't attack you.
pub fn rite_of_the_raging_storm() -> CardDefinition {
    let rager = TokenDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::SacrificeSource,
        }],
        ..red_token(
            "Lightning Rager",
            vec![CreatureType::Elemental],
            5,
            1,
            vec![Keyword::Trample, Keyword::Haste],
        )
    };
    CardDefinition {
        name: "Rite of the Raging Storm",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures named Lightning Rager can't attack you or planeswalkers you control.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: true,
                filter: Some(R::HasName("Lightning Rager".into())),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::CreateToken {
                who: PlayerRef::ActivePlayer,
                count: Value::ONE,
                definition: Arc::new(rager),
            },
        }],
        ..Default::default()
    }
}

/// Seal of the Guildpact — choose two colours as it enters; your spells cost
/// {1} less for each of them they are.
pub fn seal_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Seal of the Guildpact",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        as_enters_effect: Some(Effect::ChooseTwoColorsForSource),
        static_abilities: vec![StaticAbility {
            description: "Each spell you cast costs {1} less to cast for each of the chosen colors it is.",
            effect: StaticEffect::ChosenColorsSpellCostReduction,
        }],
        ..Default::default()
    }
}

/// Stolen Goods — target opponent exiles until a nonland card; you may cast
/// it free this turn.
pub fn stolen_goods() -> CardDefinition {
    spell(
        "Stolen Goods",
        cost(&[generic(3), u()]),
        false,
        Effect::TargetPlayerThen {
            filter: R::Player.and(R::ControlledByOpponent),
            then: Box::new(Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::Target(0),
                duration: MayPlayDuration::EndOfThisTurn,
                free: true,
                hand_unless_mv_below: None,
                grant_to_exiling_player: false,
            }),
        },
    )
}

/// Gigantoplasm — may enter as a copy of any creature, except it has
/// "{X}: This creature has base power and toughness X/X."
pub fn gigantoplasm() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature,
            extra_activated: vec![ActivatedAbility {
                mana_cost: cost(&[x()]),
                effect: Effect::SetBasePT {
                    what: Selector::This,
                    power: Value::XFromCost,
                    toughness: Value::XFromCost,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..creature("Gigantoplasm", cost(&[generic(3), u()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Lone Revenant — hexproof; on combat damage to a player, if you control no
/// other creatures, look at the top four and take one.
pub fn lone_revenant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                )))),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(4),
                ..Default::default()
            })),
        }],
        ..creature("Lone Revenant", cost(&[generic(3), u(), u()]), vec![CreatureType::Spirit], 4, 4)
    }
}

/// Warchief Giant — haste, myriad.
pub fn warchief_giant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![myriad()],
        ..creature(
            "Warchief Giant",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            5,
            3,
        )
    }
}
