//! Odyssey (ODY) gap-closing wave 5: the black graveyard engines and the green
//! Squirrel/Lhurgoyf shell. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::target_filtered,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, x};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

fn squirrel() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        ..Default::default()
    }
}

fn upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::Upkeep), EventScope::YourControl)
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Cabal Patriarch — {3}{B}{B}{B} 5/5 that shrinks a creature two ways.
pub fn cabal_patriarch() -> CardDefinition {
    let shrink = Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(-2),
        toughness: Value::Const(-2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                sac_other_filter: Some((R::Creature, 1)),
                effect: shrink.clone(),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                exile_other_filter: Some((R::Creature.and(R::InYourGraveyard), 1)),
                effect: shrink,
                ..Default::default()
            },
        ],
        ..creature(
            "Cabal Patriarch",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            5,
            5,
        )
    }
}

/// Screams of the Damned — {3}{B}{B} turns your graveyard into a sweeper.
pub fn screams_of_the_damned() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            exile_other_filter: Some((R::InYourGraveyard, 1)),
            effect: Effect::DealDamage {
                to: Selector::Both(
                    Box::new(Selector::EachPermanent(R::Creature)),
                    Box::new(Selector::Player(PlayerRef::EachPlayer)),
                ),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Screams of the Damned", cost(&[generic(3), b(), b()]))
    }
}

/// Malevolent Awakening — {1}{B}{B} recycles a body into a graveyard creature.
pub fn malevolent_awakening() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..enchantment("Malevolent Awakening", cost(&[generic(1), b(), b()]))
    }
}

/// Stalking Bloodsucker — {4}{B}{B} 4/4 flier that pitches for size.
pub fn stalking_bloodsucker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Stalking Bloodsucker",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Vampire],
            4,
            4,
        )
    }
}

/// Vampiric Dragon — {6}{B}{R} 5/5 flier that feeds on what it kills.
pub fn vampiric_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Vampiric Dragon",
            cost(&[generic(6), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Dragon],
            5,
            5,
        )
    }
}

/// Skeletal Scrying — {X}{B} draws X off your graveyard, at X life.
pub fn skeletal_scrying() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::ExileFromGraveyard {
            filter: R::Any,
            count: 1,
        }],
        ..instant(
            "Skeletal Scrying",
            cost(&[x(), b()]),
            Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                Effect::LoseLife { who: Selector::You, amount: Value::XFromCost },
            ]),
        )
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Terravore — {1}{G}{G} trampler the size of every graveyard's lands.
pub fn terravore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::BasePlusLandsInAllGraveyards { base_p: 0, base_t: 0 }),
        ..creature("Terravore", cost(&[generic(1), g(), g()]), vec![CreatureType::Lhurgoyf], 0, 0)
    }
}

/// Squirrel Mob — {1}{G}{G} 2/2 that scales with its friends.
pub fn squirrel_mob() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each other Squirrel on the battlefield.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Squirrel))
                        .and(R::OtherThanSource),
                ))),
                toughness: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Squirrel))
                        .and(R::OtherThanSource),
                ))),
            },
        }],
        ..creature("Squirrel Mob", cost(&[generic(1), g(), g()]), vec![CreatureType::Squirrel], 2, 2)
    }
}

/// Nut Collector — {5}{G} 1/1 that mints Squirrels and lords them past
/// Threshold.
pub fn nut_collector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: upkeep(),
            effect: Effect::MayDo {
                description: "Create a 1/1 Squirrel?".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: squirrel(),
                }),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Threshold — all Squirrels get +2/+2.",
            effect: StaticEffect::PumpTeamIf {
                condition: threshold(),
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Squirrel)),
                ),
                power: 2,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..creature(
            "Nut Collector",
            cost(&[generic(5), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Spellbane Centaur — {2}{G} 3/2 that walls off blue targeting.
pub fn spellbane_centaur() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control can't be the targets of blue spells or abilities.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Protection(Color::Blue),
            },
        }],
        ..creature("Spellbane Centaur", cost(&[generic(2), g()]), vec![CreatureType::Centaur], 3, 2)
    }
}

/// Zoologist — {3}{G} 1/2 that flips the top card for a free creature.
pub fn zoologist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            tap_cost: true,
            effect: Effect::LookTopMayRevealMatchToHandElseBottom { filter: R::Creature },
            ..Default::default()
        }],
        ..creature(
            "Zoologist",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            2,
        )
    }
}

/// Howling Gale — {1}{G} a flier sweeper that comes back once.
pub fn howling_gale() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), g()]))],
        ..instant(
            "Howling Gale",
            cost(&[generic(1), g()]),
            Effect::DealDamage {
                to: Selector::Both(
                    Box::new(Selector::EachPermanent(
                        R::Creature.and(R::HasKeyword(Keyword::Flying)),
                    )),
                    Box::new(Selector::Player(PlayerRef::EachPlayer)),
                ),
                amount: Value::ONE,
            },
        )
    }
}

/// New Frontiers — {X}{G} every player ramps X basics.
pub fn new_frontiers() -> CardDefinition {
    sorcery(
        "New Frontiers",
        cost(&[x(), g()]),
        Effect::Repeat {
            count: Value::XFromCost,
            body: Box::new(Effect::Search {
                who: PlayerRef::EachPlayer,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: true },
            }),
        },
    )
}

/// Hint of Insanity — {2}{B} strips every duplicate from a hand.
pub fn hint_of_insanity() -> CardDefinition {
    sorcery(
        "Hint of Insanity",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::RevealHand { who: PlayerRef::Target(0) },
            Effect::RevealHandDiscardAllMatching {
                who: PlayerRef::Target(0),
                filter: R::Nonland.and(R::SharesNameWithAnotherPermanent),
            },
        ]),
    )
}

/// Still Life — {1}{G}{G} animates into a 4/3 Centaur.
pub fn still_life() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g()]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(3),
                creature_types: vec![CreatureType::Centaur],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Still Life", cost(&[generic(1), g(), g()]))
    }
}

/// Dreamwinder — {3}{U} 4/3 Serpent that needs an Island on the table.
/// (The gate is any Island rather than the defender's, and the `{U}`,
/// sacrifice-an-Island land-animation half is dropped.)
pub fn dreamwinder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessLandTypeOnBattlefield(
            crate::card::LandType::Island,
        )],
        ..creature("Dreamwinder", cost(&[generic(3), u()]), vec![CreatureType::Serpent], 4, 3)
    }
}
