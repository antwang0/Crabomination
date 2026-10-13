//! Commander: the 2026-10-13 rescan's gaps for Azami, Lady of Scrolls; Kolodin,
//! Triumph Caster; and Sokka and Suki. Tests in
//! `tests/recent_b/cmdr_edhrec45.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, u, w, Color, ManaCost};
use crabomination_base::turn_step::TurnStep;
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn equipment() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Equipment)
}

fn vehicle() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
}

fn wizard() -> R {
    R::HasCreatureType(CreatureType::Wizard)
}

// ── Azami, Lady of Scrolls ───────────────────────────────────────────────────

/// Cyclone Summoner — {5}{U}{U} 7/7 Giant Wizard. Entering after a cast from
/// your hand: every permanent but Giants, Wizards and lands to its owner's hand.
pub fn cyclone_summoner() -> CardDefinition {
    let spared = R::HasCreatureType(CreatureType::Giant).or(wizard()).or(R::Land);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceCastFromOwnersHand),
            effect: Effect::Move {
                what: Selector::EachPermanent(R::Permanent.and(R::Not(Box::new(spared)))),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature(
            "Cyclone Summoner",
            cost(&[generic(5), u(), u()]),
            vec![CreatureType::Giant, CreatureType::Wizard],
            7,
            7,
        )
    }
}

/// Sigil Tracer — {1}{U}{U} 2/2 Merfolk Wizard. {1}{U}, tap two untapped
/// Wizards you control: copy target instant or sorcery spell; new targets
/// allowed.
pub fn sigil_tracer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_others_cost: Some((wizard().and(R::ControlledByYou), 2)),
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
                count: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Sigil Tracer", cost(&[generic(1), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 2, 2)
    }
}

/// Vedalken Aethermage — {1}{U} 1/2 Vedalken Wizard, flash. Enters: return
/// target Sliver to its owner's hand. Wizardcycling {3}.
pub fn vedalken_aethermage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Typecycling(Box::new((cost(&[generic(3)]), wizard())))],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::HasCreatureType(CreatureType::Sliver)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..creature("Vedalken Aethermage", cost(&[generic(1), u()]), vec![CreatureType::Vedalken, CreatureType::Wizard], 1, 2)
    }
}

// ── Kolodin, Triumph Caster ──────────────────────────────────────────────────

/// Consulate Dreadnought — {1} artifact Vehicle 7/11, crew 6.
pub fn consulate_dreadnought() -> CardDefinition {
    CardDefinition {
        name: "Consulate Dreadnought",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 7,
        toughness: 11,
        keywords: vec![Keyword::Crew(6)],
        ..Default::default()
    }
}

/// Depala, Pilot Exemplar — {1}{R}{W} legendary 3/3 Dwarf Pilot. Other Dwarves
/// you control get +1/+1; your Vehicles get +1/+1 while creatures. Becoming
/// tapped: you may pay {X} to reveal the top X and keep the Dwarves and
/// Vehicles, the rest to the bottom at random.
pub fn depala_pilot_exemplar() -> CardDefinition {
    let dwarf = || R::HasCreatureType(CreatureType::Dwarf);
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Dwarves you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(dwarf().and(R::ControlledByYou).and(R::OtherThanSource)),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Each Vehicle you control gets +1/+1 as long as it's a creature.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(vehicle().and(R::Creature).and(R::ControlledByYou)),
                    power: 1,
                    toughness: 1,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayPayX {
                description: "Pay {X} to reveal the top X for Dwarves and Vehicles?".into(),
                body: Box::new(Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::XFromCost,
                    filter: dwarf().or(vehicle()),
                    distinct_powers: false,
                }),
            },
        }],
        ..legend(
            "Depala, Pilot Exemplar",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Dwarf, CreatureType::Pilot],
            3,
            3,
        )
    }
}

/// Siege Modification — {1}{R}{R} Aura, enchant creature or Vehicle. A Vehicle
/// it enchants is a creature; the enchanted creature gets +3/+0 and first
/// strike.
pub fn siege_modification() -> CardDefinition {
    CardDefinition {
        name: "Siege Modification",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.or(vehicle())) },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            keywords: vec![Keyword::FirstStrike],
            add_card_types: vec![CardType::Creature],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Sokka and Suki ───────────────────────────────────────────────────────────

/// Sokka and Suki — {U}{R}{W} legendary 3/3 Human Warrior Ally. It or another
/// Ally of yours entering: attach an Equipment of yours to that creature. An
/// Equipment of yours entering: a 1/1 white Ally.
pub fn sokka_and_suki() -> CardDefinition {
    let ally = TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Ally),
                    },
                ),
                effect: Effect::Attach {
                    what: target_filtered(equipment().and(R::ControlledByYou)),
                    to: Selector::TriggerSource,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: equipment() }),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(ally) },
            },
        ],
        ..legend(
            "Sokka and Suki",
            cost(&[u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Trusty Boomerang — {1} Equipment, equip {1}. Equipped creature has "{1},
/// {T}: tap target creature; return Trusty Boomerang to its owner's hand."
pub fn trusty_boomerang() -> CardDefinition {
    CardDefinition {
        name: "Trusty Boomerang",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Tap { what: target_filtered(R::Creature) },
                    Effect::Move { what: Selector::AttachmentGranting, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Sokka, Swordmaster — {2}{W} legendary 3/3 Human Warrior Ally, vigilance.
/// Your Equipment spells cost {1} less per Ally you control. Your beginning
/// of combat: attach an Equipment of yours to Sokka.
pub fn sokka_swordmaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Equipment spells you cast cost {1} less to cast for each Ally you control.",
            effect: StaticEffect::CostReductionByValue {
                filter: equipment(),
                amount: Value::count(Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Ally).and(R::ControlledByYou),
                )),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Attach { what: target_filtered(equipment().and(R::ControlledByYou)), to: Selector::This },
        }],
        ..legend(
            "Sokka, Swordmaster",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            3,
            3,
        )
    }
}
