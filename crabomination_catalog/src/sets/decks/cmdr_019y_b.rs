//! Commander: 2026-10-17 (`019Y6J4R`) closers, second batch — double-faced
//! closers and the Living Breakthrough mana-value lock. Tests in
//! `tests/recent_b/cmdr_019y_b.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardId, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Waterbend, Zone,
};
use crate::effect::shortcut::{craft, etb, on_attack};
use crate::effect::{Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, w};
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

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

/// Ravenous Demon // Archdemon of Greed — {3}{B}{B} 4/4 Demon. Sacrifice a
/// Human: transform it (sorcery speed). Archdemon: 9/9 flying, trample; at your
/// upkeep sacrifice a Human — if you can't, tap it and it deals 9 damage to you.
pub fn ravenous_demon() -> CardDefinition {
    let human = || R::Creature.and(R::HasCreatureType(CreatureType::Human)).and(R::ControlledByYou);
    let archdemon = CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![your_upkeep(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(human())),
            then: Box::new(Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Human)),
            }),
            else_: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::This },
                Effect::DealDamage { to: Selector::You, amount: Value::Const(9) },
            ])),
        })],
        color_indicator: vec![Color::Black],
        ..creature("Archdemon of Greed", ManaCost::default(), vec![CreatureType::Demon], 9, 9)
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((human(), 1)),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(archdemon)),
        ..creature("Ravenous Demon", cost(&[generic(3), b(), b()]), vec![CreatureType::Demon], 4, 4)
    }
}

fn golem_4_4() -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Golem".into(),
            power: 4,
            toughness: 4,
            card_types: vec![CardType::Artifact, CardType::Creature],
            colors: vec![Color::White, Color::Blue],
            subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
            ..Default::default()
        }),
    }
}

/// Master's Guide-Mural // Master's Manufactory — {3}{W}{U} artifact. ETB: a
/// 4/4 white and blue Golem artifact creature. Craft with artifact {4}{W}{W}{U}.
/// Manufactory: {T}: a 4/4 Golem; only if an artifact entered under your
/// control this turn (this one returning transformed counts, CR 702.167c).
pub fn masters_guide_mural() -> CardDefinition {
    let manufactory = CardDefinition {
        name: "Master's Manufactory",
        card_types: vec![CardType::Artifact],
        color_indicator: vec![Color::White, Color::Blue],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::ArtifactsEnteredThisTurn(PlayerRef::You),
                Value::ONE,
            )),
            effect: golem_4_4(),
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Master's Guide-Mural",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(golem_4_4())],
        activated_abilities: vec![craft(cost(&[generic(4), w(), w(), u()]), R::Artifact, 1)],
        back_face: Some(Box::new(manufactory)),
        ..Default::default()
    }
}

/// Foggy Swamp Visions — {1}{B}{B} sorcery. Waterbend {X}. Exile X target
/// creature cards from graveyards; a token copy of each, sacrificed at the
/// beginning of your next end step.
pub fn foggy_swamp_visions() -> CardDefinition {
    CardDefinition {
        name: "Foggy Swamp Visions",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        waterbend: Some(Waterbend { amount: Value::XFromCost, optional: false }),
        effect: Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 20,
                min_targets: 0,
                filter: R::Creature.and(R::InGraveyard),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::Target(0),
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
                    },
                    Effect::SacrificeAtNextEndStep { what: Selector::LastCreatedToken },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Inventive Iteration // Living Breakthrough — {3}{U} Saga. I: return up to
/// one target creature or planeswalker to its owner's hand. II: return an
/// artifact card from your graveyard to your hand; if you can't, draw. III:
/// exile it, return it transformed. Living Breakthrough: 3/3 flying Moonfolk;
/// whenever you cast a spell, opponents can't cast spells of its mana value
/// until your next turn.
pub fn inventive_iteration() -> CardDefinition {
    let your_artifacts =
        || Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Artifact };
    let breakthrough = CardDefinition {
        name: "Living Breakthrough",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        color_indicator: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moonfolk], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::LockOpponentsManaValueUntilYourNextTurn {
                mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Inventive Iteration",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (
                1,
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.or(R::Planeswalker),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                },
            ),
            (
                2,
                Effect::If {
                    cond: Predicate::SelectorExists(your_artifacts()),
                    then: Box::new(Effect::MoveChosen {
                        from: your_artifacts(),
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                },
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(breakthrough)),
        ..Default::default()
    }
}

/// Cemetery Illuminator — {1}{U}{U} 2/3 flying Spirit. Entering or attacking,
/// exile a card from a graveyard (linked). You may look at the top card of your
/// library; once each turn you may cast a spell from there if it shares a card
/// type with a card exiled with it.
pub fn cemetery_illuminator() -> CardDefinition {
    let exile_one = || Effect::MoveChosen {
        from: Selector::CardsInZone { who: PlayerRef::EachPlayer, zone: Zone::Graveyard, filter: R::Any },
        filter: None,
        count: Value::ONE,
        up_to: false,
        to: ZoneDest::ExileWithSourceStamp,
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(exile_one()), on_attack(exile_one())],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "Once each turn, you may cast a spell from the top of your library if it shares a card type with a card exiled with this creature.",
                effect: StaticEffect::PlayFromLibraryTopOncePerTurn {
                    filter: R::Land.negate().and(R::SharesCardTypeWithExiledBySource),
                },
            },
        ],
        ..creature("Cemetery Illuminator", cost(&[generic(1), u(), u()]), vec![CreatureType::Spirit], 2, 3)
    }
}

/// Gwen Stacy // Ghost-Spider — {1}{R} legendary 2/1. ETB: exile the top card
/// of your library; you may play it while you control her. {2}{U}{R}{W}:
/// transform (sorcery). Ghost-Spider: 4/4 flying, vigilance, haste; playing a
/// land or casting a spell from exile adds a +1/+1 counter; remove two counters
/// from it: exile the top card, you may play it this turn.
pub fn gwen_stacy() -> CardDefinition {
    let counter = || Effect::AddCounter {
        what: Selector::This,
        kind: crate::card::CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    let ghost_spider = CardDefinition {
        name: "Ghost-Spider",
        cost: cost(&[generic(2), u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellFromExile),
                effect: counter(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::EnteredFromExileThisTurn,
                    },
                ),
                effect: counter(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_among_filter: Some((None, 2, R::IsSource)),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..creature(
            "Ghost-Spider",
            ManaCost::default(),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            4,
            4,
        )
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::WhileHolderControlsSource { source: CardId(0) },
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), r(), w()]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(ghost_spider)),
        ..creature(
            "Gwen Stacy",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Performer, CreatureType::Hero],
            2,
            1,
        )
    }
}
