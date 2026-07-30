//! Modern Horizons 2 sweep, batch 9 — chosen-card-type protection
//! (CR 702.16j), loyalty-cost taxes, modular bonuses, granted outlast,
//! remove-X-counter costs, delirium cascade. Tests in `tests/mh2h.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, investigate, modular_dies, outlast, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, StaticEffect, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

use SelectionRequirement as R;

/// Arcbound Javelineer — {W} 0/1 Soldier, modular 1. {T}, Remove X +1/+1
/// counters: deal X damage to target attacking or blocking creature.
pub fn arcbound_javelineer() -> CardDefinition {
    CardDefinition {
        name: "Arcbound Javelineer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        keywords: vec![Keyword::Modular(1)],
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![modular_dies()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_x: Some(CounterType::PlusOnePlusOne),
            effect: Effect::DealDamage {
                amount: Value::XFromCost,
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Arcus Acolyte — {G}{W} 2/2 reach lifelink. Outlast {G/W}; each other
/// creature you control without a +1/+1 counter has outlast {G/W}.
pub fn arcus_acolyte() -> CardDefinition {
    let gw = || cost(&[hybrid(Color::Green, Color::White)]);
    CardDefinition {
        name: "Arcus Acolyte",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Cleric,
                CreatureType::Archer,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach, Keyword::Lifelink],
        activated_abilities: vec![outlast(gw())],
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control without a +1/+1 counter has outlast {G/W}.",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource)
                        .and(R::Not(Box::new(R::WithCounter(
                            CounterType::PlusOnePlusOne,
                        )))),
                ),
                ability: outlast(gw()),
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Serra's Emissary — {4}{W}{W}{W} 7/7 flying Angel. As it enters, choose a
/// card type; you and creatures you control have protection from it.
pub fn serras_emissary() -> CardDefinition {
    CardDefinition {
        name: "Serra's Emissary",
        cost: cost(&[generic(4), w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ChooseCardTypeForSource)],
        static_abilities: vec![StaticAbility {
            description: "You and creatures you control have protection from the chosen card type.",
            effect: StaticEffect::YouAndCreaturesProtectionFromChosenCardType,
        }],
        ..Default::default()
    }
}

/// Shattered Ego — {U} Aura. Enchanted creature gets -3/-0; {3}{U}{U}: put
/// enchanted creature into its owner's library third from the top.
pub fn shattered_ego() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Shattered Ego",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: -3,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::FromTop(2),
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn tapped_squirrel() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        colors: vec![Color::Green],
        tapped: true,
        ..Default::default()
    }
}

/// Verdant Command — {1}{G} instant. Choose two: two tapped Squirrels;
/// counter target loyalty ability; exile a graveyard card; 3 life.
pub fn verdant_command() -> CardDefinition {
    CardDefinition {
        name: "Verdant Command",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![2],
            modes: vec![
                Effect::CreateToken {
                    who: PlayerRef::Target(0),
                    count: Value::Const(2),
                    definition: tapped_squirrel(),
                },
                Effect::CounterAbility {
                    what: target_filtered(R::Planeswalker),
                },
                Effect::Move {
                    what: target_filtered(R::InGraveyard),
                    to: ZoneDest::Exile,
                },
                Effect::GainLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(3),
                },
            ],
        },
        ..Default::default()
    }
}

/// Zabaz, the Glimmerwasp — {1} 0/0 legendary Insect, modular 1. Modular
/// triggers you control add an extra counter; {R}: destroy target artifact
/// you control; {W}: Zabaz gains flying until end of turn.
pub fn zabaz_the_glimmerwasp() -> CardDefinition {
    CardDefinition {
        name: "Zabaz, the Glimmerwasp",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        keywords: vec![Keyword::Modular(1)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![modular_dies()],
        static_abilities: vec![StaticAbility {
            description: "Modular abilities put that many counters plus one instead.",
            effect: StaticEffect::ModularBonusCounters(1),
        }],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::Destroy {
                    what: target_filtered(R::Artifact.and(R::ControlledByYou)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lonis, Cryptozoologist — {G}{U} 1/2. Investigate on another nontoken
/// creature ETB; {T}, Sacrifice X Clues: steal a nonland permanent with
/// MV ≤ X from target opponent's top X cards.
pub fn lonis_cryptozoologist() -> CardDefinition {
    CardDefinition {
        name: "Lonis, Cryptozoologist",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::OtherThanSource),
                }),
            effect: investigate(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::HasArtifactSubtype(crate::card::ArtifactSubtype::Clue), 1)),
            sac_other_x: true,
            effect: Effect::OpponentRevealsPickToBattlefield {
                count: Value::XFromCost,
                max_mv: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Carth the Lion — {2}{B}{G} 3/5. ETB / your planeswalker dies: dig 7 for a
/// planeswalker; your loyalty abilities cost an additional [+1].
pub fn carth_the_lion() -> CardDefinition {
    let dig = Effect::LookPickToHand {
        who: PlayerRef::You,
        count: Value::Const(7),
        rest_to_graveyard: false,
        pick_filter: Some(R::Planeswalker),
        take: None,
        to_battlefield: false,
        gain_life_if_pick: None,
        gain_life_greatest_power_rest: false,
        optional: false,
        picked_lands_to_battlefield: false,
        rest_bottom_random: false,
    };
    CardDefinition {
        name: "Carth the Lion",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![
            etb(dig.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Planeswalker,
                    }),
                effect: dig,
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Planeswalkers' loyalty abilities you activate cost an additional [+1].",
            effect: StaticEffect::LoyaltyAbilitiesCostExtra(1),
        }],
        ..Default::default()
    }
}

/// Bloodbraid Marauder — {1}{R} 3/1, can't block. Delirium — cascade while
/// four or more card types are among cards in your graveyard.
pub fn bloodbraid_marauder() -> CardDefinition {
    CardDefinition {
        name: "Bloodbraid Marauder",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource).with_filter(
                Predicate::DeliriumActive {
                    who: PlayerRef::You,
                },
            ),
            effect: Effect::Cascade {
                max_mv: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Rise and Shine — {1}{U} sorcery. Target noncreature artifact you control
/// becomes a 0/0 creature with four +1/+1 counters. Overload {4}{U}{U}.
pub fn rise_and_shine() -> CardDefinition {
    use crate::card::AlternativeCost;
    let animate = |what: Selector| {
        Effect::Seq(vec![
            Effect::BecomeCreature {
                what: what.clone(),
                power: Value::Const(0),
                toughness: Value::Const(0),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            Effect::AddCounter {
                what,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            },
        ])
    };
    let filter = R::Artifact.and(R::Noncreature).and(R::ControlledByYou);
    CardDefinition {
        name: "Rise and Shine",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: animate(target_filtered(filter.clone())),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(4), u(), u()]),
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(filter),
                body: Box::new(animate(Selector::TriggerSource)),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}
