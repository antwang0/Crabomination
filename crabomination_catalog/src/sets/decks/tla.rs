//! The Last Airbender (TLA) staples on existing primitives — Allies, hybrid
//! costs, attack/ETB triggers, and a defensive anthem. Tests in
//! `crabomination/src/tests/tla.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, ExileReturnZone, Keyword, LandType, Predicate, SelectionRequirement,
    Selector, SpellSubtype, StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_dies, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost, ManaSymbol};

/// A Lesson-subtyped spell shell (instant/sorcery).
fn lesson() -> Subtypes {
    Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() }
}

/// A 1/1 white Ally creature token (Kyoshi Warriors).
fn ally_token() -> TokenDefinition {
    TokenDefinition {
        name: "Ally".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ally], ..Default::default() },
        ..Default::default()
    }
}

/// Cat-Gator — {6}{B} 3/2 Fish Crocodile. Lifelink; ETB deals damage equal to
/// the number of Swamps you control to any target.
pub fn cat_gator() -> CardDefinition {
    CardDefinition {
        name: "Cat-Gator",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish, CreatureType::Crocodile],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasLandType(LandType::Swamp),
            },
        })],
        ..Default::default()
    }
}

/// Cat-Owl — {3}{W/U} 3/3 Cat Bird. Flying; on attack, untap target artifact or
/// creature.
pub fn cat_owl() -> CardDefinition {
    CardDefinition {
        name: "Cat-Owl",
        cost: cost(&[generic(3), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Bird],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::Untap {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Creature),
            ),
            up_to: None,
        })],
        ..Default::default()
    }
}

/// Kyoshi Warriors — {3}{W} 3/3 Human Warrior Ally. ETB: make a 1/1 white Ally.
pub fn kyoshi_warriors() -> CardDefinition {
    CardDefinition {
        name: "Kyoshi Warriors",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: ally_token(),
        })],
        ..Default::default()
    }
}

/// The Walls of Ba Sing Se — {8} 0/30 legendary Wall. Defender; other permanents
/// you control have indestructible.
pub fn walls_of_ba_sing_se() -> CardDefinition {
    CardDefinition {
        name: "The Walls of Ba Sing Se",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 30,
        keywords: vec![Keyword::Defender],
        static_abilities: vec![StaticAbility {
            description: "Other permanents you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::ControlledByYou
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Indestructible,
            },
        }],
        ..Default::default()
    }
}

/// Wandering Musicians — {3}{R/W} 2/5 Human Bard Ally. Whenever it attacks,
/// creatures you control get +1/+0 until end of turn.
pub fn wandering_musicians() -> CardDefinition {
    CardDefinition {
        name: "Wandering Musicians",
        cost: cost(&[generic(3), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Bard, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// It'll Quench Ya! — {1}{U} Instant — Lesson. Counter target spell unless its
/// controller pays {2}.
pub fn itll_quench_ya() -> CardDefinition {
    CardDefinition {
        name: "It'll Quench Ya!",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Ozai's Cruelty — {2}{B} Sorcery — Lesson. Deals 2 damage to target player,
/// who then discards two cards.
pub fn ozais_cruelty() -> CardDefinition {
    CardDefinition {
        name: "Ozai's Cruelty",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            Effect::Discard { who: Selector::Target(0), amount: Value::Const(2), random: false },
        ]),
        ..Default::default()
    }
}

/// Pillar Launch — {G} Instant. Target creature gets +2/+2, gains reach, and
/// untaps.
pub fn pillar_launch() -> CardDefinition {
    CardDefinition {
        name: "Pillar Launch",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Rocky Rebuke — {1}{G} Instant. Target creature you control deals damage equal
/// to its power to target creature an opponent controls.
pub fn rocky_rebuke() -> CardDefinition {
    CardDefinition {
        name: "Rocky Rebuke",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamageEqualToPower {
            source: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            target: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
        },
        ..Default::default()
    }
}

/// Shared Roots — {1}{G} Sorcery — Lesson. Search your library for a basic land
/// and put it onto the battlefield tapped.
pub fn shared_roots() -> CardDefinition {
    CardDefinition {
        name: "Shared Roots",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// United Front — {X}{W}{W} Sorcery. Create X 1/1 white Allies, then put a +1/+1
/// counter on each creature you control.
pub fn united_front() -> CardDefinition {
    CardDefinition {
        name: "United Front",
        cost: cost(&[x(), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: ally_token(),
            },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── Second wave: Ally / Lesson / Raid / Clue commons ────────────────────────

fn ally(types: &[CreatureType]) -> Subtypes {
    Subtypes { creature_types: types.to_vec(), ..Default::default() }
}

/// "Another Ally you control enters" trigger.
fn another_ally_enters() -> EventSpec {
    EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
        Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: SelectionRequirement::HasCreatureType(CreatureType::Ally),
        },
    )
}

/// A Lesson card sits in your graveyard.
fn lesson_in_graveyard() -> Predicate {
    Predicate::SelectorCountAtLeast {
        sel: Selector::CardsInZone {
            who: PlayerRef::You,
            zone: Zone::Graveyard,
            filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
        },
        n: Value::ONE,
    }
}

/// Water Tribe Captain — {2}{W} 3/3 Human Soldier. {5}: Creatures you control
/// get +1/+1 until end of turn.
pub fn water_tribe_captain() -> CardDefinition {
    CardDefinition {
        name: "Water Tribe Captain",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier]),
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![ManaSymbol::Generic(5)]),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fancy Footwork — {2}{W} Instant (Lesson). Untap one or two target creatures;
/// each gets +2/+2 until end of turn.
pub fn fancy_footwork() -> CardDefinition {
    CardDefinition {
        name: "Fancy Footwork",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::ApplyToTargets {
            filter: SelectionRequirement::Creature,
            max_targets: 2,
            effect: Box::new(Effect::Seq(vec![
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
        ..Default::default()
    }
}

/// Earth Kingdom Protectors — {W} 1/1 Human Soldier Ally. Vigilance. Sacrifice
/// this creature: Another target Ally you control gains indestructible until end
/// of turn.
pub fn earth_kingdom_protectors() -> CardDefinition {
    CardDefinition {
        name: "Earth Kingdom Protectors",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier, CreatureType::Ally]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Ally)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Yip Yip! — {W} Instant (Lesson). Target creature you control gets +2/+2; if
/// that creature is an Ally, it also gains flying until end of turn.
pub fn yip_yip() -> CardDefinition {
    CardDefinition {
        name: "Yip Yip!",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Ally),
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Earth Kingdom Jailer — {2}{W} 3/3 Human Soldier. When this enters, exile up
/// to one target artifact, creature, or enchantment an opponent controls with
/// mana value 3 or greater until this creature leaves the battlefield.
pub fn earth_kingdom_jailer() -> CardDefinition {
    CardDefinition {
        name: "Earth Kingdom Jailer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::ControlledByOpponent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::ManaValueAtLeast(3)),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Octopus Form — {U} Instant (Lesson). Target creature you control gets +1/+1
/// and gains hexproof until end of turn. Untap it.
pub fn octopus_form() -> CardDefinition {
    CardDefinition {
        name: "Octopus Form",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// First-Time Flyer — {1}{U} 1/2 Human Pilot Ally. Flying. Gets +1/+1 as long as
/// there's a Lesson card in your graveyard.
pub fn first_time_flyer() -> CardDefinition {
    CardDefinition {
        name: "First-Time Flyer",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Pilot, CreatureType::Ally]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Gets +1/+1 while a Lesson card is in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: lesson_in_graveyard(),
                power: 1,
                toughness: 1,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Fire Nation Engineer — {2}{B} 2/3 Human Artificer. Raid — At the beginning of
/// your end step, if you attacked this turn, put a +1/+1 counter on another
/// target creature you control.
pub fn fire_nation_engineer() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Engineer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Artificer]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Cunning Maneuver — {1}{R} Instant. Target creature gets +3/+1 until end of
/// turn. Create a Clue token.
pub fn cunning_maneuver() -> CardDefinition {
    CardDefinition {
        name: "Cunning Maneuver",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Fire Nation Raider — {3}{R} 4/2 Human Soldier. Raid — When this creature
/// enters, if you attacked this turn, create a Clue token.
pub fn fire_nation_raider() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Raider",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier]),
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
            then: Box::new(investigate(1)),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Wartime Protestors — {3}{R} 4/4 Human Rebel Ally. Haste. Whenever another Ally
/// you control enters, put a +1/+1 counter on that creature and it gains haste
/// until end of turn.
pub fn wartime_protestors() -> CardDefinition {
    CardDefinition {
        name: "Wartime Protestors",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Rebel, CreatureType::Ally]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: another_ally_enters(),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Saber-Tooth Moose-Lion — {4}{G}{G} 7/7 Elk Cat. Reach. Forestcycling {2}.
pub fn saber_tooth_moose_lion() -> CardDefinition {
    CardDefinition {
        name: "Saber-Tooth Moose-Lion",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Elk, CreatureType::Cat]),
        power: 7,
        toughness: 7,
        keywords: vec![
            Keyword::Reach,
            Keyword::Landcycling(ManaCost::new(vec![ManaSymbol::Generic(2)]), LandType::Forest),
        ],
        ..Default::default()
    }
}

/// Walltop Sentries — {2}{G} 2/3 Human Soldier. Reach, deathtouch. When this
/// creature dies, if there's a Lesson card in your graveyard, you gain 2 life.
pub fn walltop_sentries() -> CardDefinition {
    CardDefinition {
        name: "Walltop Sentries",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        triggered_abilities: vec![on_dies(Effect::If {
            cond: lesson_in_graveyard(),
            then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Earth Kingdom Soldier — {4}{G/W} 3/4 Human Soldier. Vigilance. When this
/// enters, put a +1/+1 counter on each of up to two target creatures you
/// control.
pub fn earth_kingdom_soldier() -> CardDefinition {
    CardDefinition {
        name: "Earth Kingdom Soldier",
        cost: cost(&[generic(4), hybrid(Color::Green, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            max_targets: 2,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        })],
        ..Default::default()
    }
}

/// White Lotus Reinforcements — {1}{G}{W} 2/3 Human Soldier Ally. Vigilance.
/// Other Allies you control get +1/+1.
pub fn white_lotus_reinforcements() -> CardDefinition {
    CardDefinition {
        name: "White Lotus Reinforcements",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier, CreatureType::Ally]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Other Allies you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Ally)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Combustion Technique — {1}{R} Instant (Lesson). Deals damage equal to 2 plus
/// the number of Lesson cards in your graveyard to target creature.
pub fn combustion_technique() -> CardDefinition {
    CardDefinition {
        name: "Combustion Technique",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Sum(vec![
                Value::Const(2),
                Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
                },
            ]),
        },
        ..Default::default()
    }
}
