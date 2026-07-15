//! MH2 Food/Eggs + artifact-combo batch: Cookbook, Asmor, Urza, Tezzeret,
//! Second Sunrise. Tests in `tests/recent107.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EquipBonus, EquipScale, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement,
    Selector, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered, unearth};
use crate::card::AlternativeCost;
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, StaticEffect, ZoneDest,
};
use crate::mana::{b, cost, generic, hybrid, r, u, w, Color};

/// Cranial Ram — {B}{R} Living weapon Equipment. +X/+1, X = your artifacts.
/// Equip {2}.
pub fn cranial_ram() -> CardDefinition {
    CardDefinition {
        name: "Cranial Ram",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            toughness: 1,
            scale: Some(EquipScale {
                filter: SelectionRequirement::Artifact,
                per_power: 1,
                per_toughness: 0,
                ..Default::default()
            }),
            ..Default::default()
        }),
        // Living weapon (CR 702.92) — the Bonehoard germ pattern.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Phyrexian Germ".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Phyrexian],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// The Underworld Cookbook — {1} Book. {T}, discard: Food. {4},{T},sac:
/// raise a creature card to hand.
pub fn the_underworld_cookbook() -> CardDefinition {
    CardDefinition {
        name: "The Underworld Cookbook",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((SelectionRequirement::Any, 1)),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::food_token(),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(4)]),
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Asmoranomardicadaistinaculdacar — no mana cost; castable for {B/R} once
/// you've discarded this turn. ETB: fetch The Underworld Cookbook. Sac two
/// Foods: a creature deals 6 damage to itself.
pub fn asmoranomardicadaistinaculdacar() -> CardDefinition {
    CardDefinition {
        name: "Asmoranomardicadaistinaculdacar",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[hybrid(Color::Black, Color::Red)]),
            condition: Some(Predicate::DiscardedThisTurn { who: PlayerRef::You }),
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasName("The Underworld Cookbook".into()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(crate::card::ArtifactSubtype::Food),
                2,
            )),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(6),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Retract — {U} Instant. Bounce all your artifacts.
pub fn retract() -> CardDefinition {
    CardDefinition {
        name: "Retract",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::EachPermanent(
                SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Jeskai Ascendancy — {U}{R}{W}. Noncreature cast: team +1/+1 EOT + untap;
/// and may loot.
pub fn jeskai_ascendancy() -> CardDefinition {
    let noncreature_cast = || {
        EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Noncreature))
    };
    CardDefinition {
        name: "Jeskai Ascendancy",
        cost: cost(&[u(), r(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: noncreature_cast(),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        up_to: None,
                    },
                ]),
            },
            TriggeredAbility {
                event: noncreature_cast(),
                effect: Effect::MayDo {
                    description: "Draw a card, then discard a card?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Fatestitcher — {3}{U} 1/2. {T}: tap or untap another target permanent.
/// Unearth {U}.
pub fn fatestitcher() -> CardDefinition {
    let another_permanent =
        || target_filtered(SelectionRequirement::Permanent.and(SelectionRequirement::OtherThanSource));
    CardDefinition {
        name: "Fatestitcher",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::ChooseMode(vec![
                    Effect::Tap { what: another_permanent() },
                    Effect::Untap { what: another_permanent(), up_to: None },
                ]),
                ..Default::default()
            },
            unearth(cost(&[u()])),
        ],
        ..Default::default()
    }
}

/// Urza, Lord High Artificer — {2}{U}{U} 1/4. ETB Karnstruct; tap an untapped
/// artifact: add {U}; {5}: shuffle, exile top, may play it this turn.
pub fn urza_lord_high_artificer() -> CardDefinition {
    let construct = TokenDefinition {
        name: "Construct".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::Artifact,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Urza, Lord High Artificer",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: construct,
        })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_other_filter: Some(SelectionRequirement::Artifact),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                effect: Effect::Seq(vec![
                    Effect::ShuffleLibrary { who: PlayerRef::You },
                    Effect::ExileTopAndGrantMayPlay {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        duration: crate::card::MayPlayDuration::EndOfThisTurn,
                        pay_any_color: false, pay_own_cost: false,
                        uncast_penalty: None,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tezzeret, Agent of Bolas — {2}{U}{B} walker. +1 dig 5 for an artifact;
/// -1 an artifact becomes 5/5; -4 drain twice your artifacts.
pub fn tezzeret_agent_of_bolas() -> CardDefinition {
    CardDefinition {
        name: "Tezzeret, Agent of Bolas",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Tezzeret],
            ..Default::default()
        },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(5),
                    rest_to_graveyard: false,
                    pick_filter: Some(SelectionRequirement::Artifact),
                    take: None,
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: false,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::BecomeCreature {
                    what: target_filtered(SelectionRequirement::Artifact),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::Drain {
                    from: target_filtered(SelectionRequirement::Player),
                    to: Selector::You,
                    amount: Value::Times(
                        Box::new(Value::CountOf(Box::new(Selector::EachPermanent(
                            SelectionRequirement::Artifact
                                .and(SelectionRequirement::ControlledByYou),
                        )))),
                        Box::new(Value::Const(2)),
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Second Sunrise — {1}{W}{W} Instant. Everyone rebuilds what died this turn.
pub fn second_sunrise() -> CardDefinition {
    CardDefinition {
        name: "Second Sunrise",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SecondSunrise,
        ..Default::default()
    }
}
