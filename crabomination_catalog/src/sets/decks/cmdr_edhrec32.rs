//! Commander: the last gaps of Ganax (solo and with Acolyte of Bahamut),
//! Aragorn Hornburg Hero, Alisaie Leveilleur (solo and with Alphinaud), Gor
//! Muldrak and The Capitoline Triad (`cmdr_edhrec30` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec32.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::MayPlayDuration;
use crate::effect::shortcut::{flurry, investigate, renown, target_filtered};
use crate::effect::{Effect, KeywordAct, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn background(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        ..Default::default()
    }
}

fn your_commanders() -> R {
    R::Creature.and(R::IsCommander).and(R::OwnedByYou)
}

fn at_your_end_step(filter: Option<Predicate>, effect: Effect) -> TriggeredAbility {
    let event = EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl);
    TriggeredAbility { event: match filter { Some(f) => event.with_filter(f), None => event }, effect }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

// ── Ganax, Astral Hunter ─────────────────────────────────────────────────────

/// Dragon Cultist — {4}{R} Background. Commander creatures you own have "At
/// the beginning of your end step, if a source you controlled dealt 5 or more
/// damage this turn, create a 4/4 red Dragon creature token with flying."
pub fn dragon_cultist() -> CardDefinition {
    let dragon = token("Dragon", 4, 4, Color::Red, vec![CreatureType::Dragon], vec![Keyword::Flying]);
    let trigger = at_your_end_step(
        Some(Predicate::SourceYouControlledDealtDamageThisTurnAtLeast(5)),
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(dragon) },
    );
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"At the beginning of your end step, if a source you \
                          controlled dealt 5 or more damage this turn, create a 4/4 red Dragon creature token with \
                          flying.\"",
            effect: StaticEffect::GrantTriggeredAbility { filter: your_commanders(), ability: Box::new(trigger) },
        }],
        ..background("Dragon Cultist", cost(&[generic(4), r()]))
    }
}

/// Acolyte of Bahamut — {1}{G} Background. Commander creatures you own have
/// "The first Dragon spell you cast each turn costs {2} less to cast."
pub fn acolyte_of_bahamut() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you own have \"The first Dragon spell you cast each turn costs {2} \
                          less to cast.\"",
            effect: StaticEffect::GrantStaticAbility {
                filter: your_commanders(),
                ability: Box::new(StaticAbility {
                    description: "The first Dragon spell you cast each turn costs {2} less to cast.",
                    effect: StaticEffect::FirstMatchingSpellEachTurnCostsLess {
                        filter: R::HasCreatureType(CreatureType::Dragon),
                        amount: 2,
                    },
                }),
            },
        }],
        ..background("Acolyte of Bahamut", cost(&[generic(1), g()]))
    }
}

// ── Aragorn, Hornburg Hero ───────────────────────────────────────────────────

/// Andúril, Narsil Reforged — {2} legendary Equipment. Ascend. Whenever
/// equipped creature attacks, put a +1/+1 counter on each creature you
/// control, or two with the city's blessing. Equip {3}.
pub fn anduril_narsil_reforged() -> CardDefinition {
    let counters = |n: i32| Effect::AddCounter {
        what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Andúril, Narsil Reforged",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        static_abilities: vec![crate::sets::ascend()],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::HasCityBlessing { who: PlayerRef::You },
                    then: Box::new(counters(2)),
                    else_: Box::new(counters(1)),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Valeron Wardens — {2}{G} 1/3 Human Monk. Renown 2. Whenever a creature you
/// control becomes renowned, draw a card.
pub fn valeron_wardens() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            renown(2),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Performed(KeywordAct::BecameRenowned), EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Valeron Wardens",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Monk],
            1,
            3,
        )
    }
}

// ── Alisaie Leveilleur ───────────────────────────────────────────────────────

/// Sunseed Nurturer — {2}{W} 1/1 Human Druid Wizard. At the beginning of your
/// end step, if you control a creature with power 5 or greater, you may gain
/// 2 life. {T}: Add {C}.
pub fn sunseed_nurturer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![at_your_end_step(
            Some(Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::PowerAtLeast(5)),
            })),
            Effect::MayDo {
                description: "Gain 2 life".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
            },
        )],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..creature(
            "Sunseed Nurturer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Druid, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Lavinia, Foil to Conspiracy — {1}{W/U}{W/U} legendary 2/3 Human Detective.
/// Vigilance. Your second spell each turn investigates. {T}: Add {C}{C};
/// activate only during an opponent's turn.
pub fn lavinia_foil_to_conspiracy() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![flurry(investigate(1))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
            ..Default::default()
        }],
        ..creature(
            "Lavinia, Foil to Conspiracy",
            cost(&[generic(1), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            3,
        )
    }
}

/// Monk Class — {W}{U} Class. L1: your second spell each turn costs {1} less.
/// L2 ({W}{U}): return up to one target nonland permanent to its owner's hand.
/// L3 ({1}{W}{U}): at your upkeep, exile the top card of your library; while
/// exiled you may cast it once you've cast another spell this turn.
pub fn monk_class() -> CardDefinition {
    let level_up = |mana: ManaCost, from: u8| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    CardDefinition {
        name: "Monk Class",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Class], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "The second spell you cast each turn costs {1} less to cast.",
            effect: StaticEffect::CostReductionNthSpell { filter: R::Any, nth: 2, amount: 1 },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::ClassLevelReached, EventScope::SelfSource)
                    .with_filter(Predicate::SourceClassLevelIs(2)),
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Permanent.and(R::Nonland)),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                    .with_filter(Predicate::SourceClassLevelAtLeast(3)),
                // "You may CAST this card": a land exiled this way stays put.
                effect: Effect::Seq(vec![
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: MayPlayDuration::WhileExiledAfterAnotherSpell,
                        pay_any_color: false,
                        max_mana_value: None,
                        pay_own_cost: true,
                        uncast_penalty: None,
                    },
                    Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
                ]),
            },
        ],
        activated_abilities: vec![level_up(cost(&[w(), u()]), 1), level_up(cost(&[generic(1), w(), u()]), 2)],
        ..Default::default()
    }
}

// ── Gor Muldrak, Amphinologist ───────────────────────────────────────────────

/// Gor Muldrak, Amphinologist — {1}{G}{U} legendary 3/2 Human Scout. You and
/// permanents you control have protection from Salamanders. At the beginning
/// of your end step, each player who controls the fewest creatures creates a
/// 4/3 blue Salamander Warrior creature token.
pub fn gor_muldrak_amphinologist() -> CardDefinition {
    let salamander =
        token("Salamander Warrior", 4, 3, Color::Blue, vec![CreatureType::Salamander, CreatureType::Warrior], vec![]);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "You have protection from Salamanders.",
                effect: StaticEffect::ControllerProtectionFromMatching {
                    filter: R::HasCreatureType(CreatureType::Salamander),
                },
            },
            StaticAbility {
                description: "Permanents you control have protection from Salamanders.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::ControlledByYou),
                    keyword: Keyword::ProtectionFromCreatureType(CreatureType::Salamander),
                },
            },
        ],
        triggered_abilities: vec![at_your_end_step(
            None,
            Effect::CreateToken {
                who: PlayerRef::EachWithFewestControlled(Box::new(R::Creature)),
                count: Value::ONE,
                definition: Arc::new(salamander),
            },
        )],
        ..creature(
            "Gor Muldrak, Amphinologist",
            cost(&[generic(1), g(), u()]),
            vec![CreatureType::Human, CreatureType::Scout],
            3,
            2,
        )
    }
}

/// Metamorphic Alteration — {1}{U} Aura (enchant creature). As it enters,
/// choose a creature; enchanted creature is a copy of the chosen creature
/// (CR 707.2) — applied with the attachment, values locked as it first applies.
pub fn metamorphic_alteration() -> CardDefinition {
    CardDefinition {
        name: "Metamorphic Alteration",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        as_enters_effect: Some(Effect::ChoosePermanentForSource { filter: R::Creature }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is a copy of the chosen creature.",
            effect: StaticEffect::AttachedIsCopyOfChosen,
        }],
        ..Default::default()
    }
}

// ── The Capitoline Triad ─────────────────────────────────────────────────────

/// The Capitoline Triad — {10} legendary 7/7 God Artificer. Costs {1} less
/// per historic card in your graveyard. Exile any number of historic cards
/// from your graveyard with total mana value 30 or greater: you get an emblem
/// with "Creatures you control have base power and toughness 9/9."
pub fn the_capitoline_triad() -> CardDefinition {
    let anthem = StaticAbility {
        description: "Creatures you control have base power and toughness 9/9.",
        effect: StaticEffect::SetBasePtForFilter {
            applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: 9,
            toughness: 9,
        },
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each historic card in your graveyard.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::count(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::historic(),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            exile_other_filter: Some((R::historic(), 1)),
            exile_other_total_mv: Some(30),
            effect: Effect::CreateEmblem {
                who: PlayerRef::You,
                name: "The Capitoline Triad".into(),
                triggered: vec![],
                statics: vec![anthem],
            },
            ..Default::default()
        }],
        ..creature(
            "The Capitoline Triad",
            cost(&[generic(10)]),
            vec![CreatureType::God, CreatureType::Artificer],
            7,
            7,
        )
    }
}

/// Shardless Outlander — {7} 6/5 artifact creature Construct Scout. Trample.
/// Basic landcycling {2}.
pub fn shardless_outlander() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![
            Keyword::Trample,
            Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand))),
        ],
        ..creature(
            "Shardless Outlander",
            cost(&[generic(7)]),
            vec![CreatureType::Construct, CreatureType::Scout],
            6,
            5,
        )
    }
}
