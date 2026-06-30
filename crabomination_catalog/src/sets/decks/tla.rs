//! The Last Airbender (TLA) staples on existing primitives — Allies, hybrid
//! costs, attack/ETB triggers, and a defensive anthem. Tests in
//! `crabomination/src/tests/tla.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, LandType,
    Predicate, SelectionRequirement, Selector, SpellSubtype, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crabomination_base::tokens::food_token;
use crate::effect::shortcut::{etb, investigate, on_attack, on_dies, raid_etb, target_any, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color, ManaCost, ManaSymbol, SpendRestriction};

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

/// Iroh's Demonstration — {1}{R} Sorcery (Lesson). Choose one — 1 damage to
/// each creature your opponents control; or 4 damage to target creature.
pub fn irohs_demonstration() -> CardDefinition {
    CardDefinition {
        name: "Iroh's Demonstration",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Creature),
                    amount: Value::Const(4),
                },
            ],
        },
        ..Default::default()
    }
}

/// Azula Always Lies — {1}{B} Instant (Lesson). Choose one or both — target
/// creature gets -1/-1 until end of turn; and/or put a +1/+1 counter on target
/// creature.
pub fn azula_always_lies() -> CardDefinition {
    CardDefinition {
        name: "Azula Always Lies",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ],
        },
        ..Default::default()
    }
}

/// Tiger-Dillo — {1}{R} 4/3 Cat Armadillo. Can't attack or block unless you
/// control another creature with power 4 or greater.
pub fn tiger_dillo() -> CardDefinition {
    CardDefinition {
        name: "Tiger-Dillo",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Armadillo],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantAttackOrBlockUnlessYouControlCount {
            filter: Box::new(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
            ),
            min: 1,
            attack_only: false,
            block_only: false,
            exclude_self: true,
        }],
        ..Default::default()
    }
}

/// Raucous Audience — {1}{G} 2/1 Human Citizen. {T}: Add {G}; add {G}{G}
/// instead if you control a creature with power 4 or greater.
pub fn raucous_audience() -> CardDefinition {
    CardDefinition {
        name: "Raucous Audience",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Green, Value::Const(2)),
                }),
                else_: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(Color::Green, Value::ONE),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Great Divide Guide — {1}{G} 2/3 Human Scout Ally. Each land and Ally you
/// control has "{T}: Add one mana of any color."
pub fn great_divide_guide() -> CardDefinition {
    CardDefinition {
        name: "Great Divide Guide",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Scout]),
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Each land and Ally you control has \"{T}: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Ally))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                ability: super::super::tap_add_any_color(),
            },
        }],
        ..Default::default()
    }
}

// ── Avatar (TLA) wave — Allies, Shrines, Firebending, draw-matters ──────────

/// Gather the White Lotus — {4}{W} Sorcery. Make a 1/1 white Ally for each
/// Plains you control, then scry 2.
pub fn gather_the_white_lotus() -> CardDefinition {
    CardDefinition {
        name: "Gather the White Lotus",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                    filter: SelectionRequirement::HasLandType(LandType::Plains),
                },
                definition: ally_token(),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Momo, Playful Pet — {W} 1/1 legendary Lemur Bat Ally. Flying, vigilance.
/// When it leaves the battlefield, choose one — make a Food, or put a +1/+1
/// counter on target creature.
pub fn momo_playful_pet() -> CardDefinition {
    CardDefinition {
        name: "Momo, Playful Pet",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Lemur, CreatureType::Bat, CreatureType::Ally]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseN {
                picks: vec![0],
                modes: vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: food_token(),
                    },
                    Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ],
            },
        }],
        ..Default::default()
    }
}

/// Rabaroo Troop — {3}{W}{W} 3/5 Rabbit Kangaroo. Landfall: gains flying until
/// end of turn and you gain 1 life. Plainscycling {2}.
pub fn rabaroo_troop() -> CardDefinition {
    CardDefinition {
        name: "Rabaroo Troop",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Kangaroo],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(
            ManaCost::new(vec![ManaSymbol::Generic(2)]),
            LandType::Plains,
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Tiger-Seal — {U} 3/3 Cat Seal. Vigilance. Taps itself each upkeep; untaps
/// when you draw your second card each turn.
pub fn tiger_seal() -> CardDefinition {
    CardDefinition {
        name: "Tiger-Seal",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Seal],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Tap { what: Selector::This },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                    .once_per_turn(),
                effect: Effect::Untap { what: Selector::This, up_to: None },
            },
        ],
        ..Default::default()
    }
}

/// The Spirit Oasis — {2}{U} legendary Shrine. ETB: draw a card per Shrine you
/// control. Whenever another Shrine you control enters, draw a card.
pub fn the_spirit_oasis() -> CardDefinition {
    CardDefinition {
        name: "The Spirit Oasis",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::Draw {
                who: Selector::You,
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                    filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Shrine),
                },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasEnchantmentSubtype(
                            EnchantmentSubtype::Shrine,
                        ),
                    }),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Northern Air Temple — {B} legendary Shrine. ETB: each opponent loses X and
/// you gain X, X = Shrines you control. Whenever another Shrine enters, drain 1.
pub fn northern_air_temple() -> CardDefinition {
    let shrines = || Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Shrine),
    };
    CardDefinition {
        name: "Northern Air Temple",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: shrines(),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasEnchantmentSubtype(
                            EnchantmentSubtype::Shrine,
                        ),
                    }),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Epic Downfall — {1}{B} Sorcery. Exile target creature with mana value 3 or
/// greater.
pub fn epic_downfall() -> CardDefinition {
    CardDefinition {
        name: "Epic Downfall",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtLeast(3)),
            ),
        },
        ..Default::default()
    }
}

/// Callous Inspector — {B} 1/1 Human Soldier. Menace. When it dies, it deals 1
/// damage to you and you investigate.
pub fn callous_inspector() -> CardDefinition {
    CardDefinition {
        name: "Callous Inspector",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::You, amount: Value::ONE },
            investigate(1),
        ]))],
        ..Default::default()
    }
}

/// Canyon Crawler — {4}{B}{B} 6/6 Spider Beast. Deathtouch. ETB: make a Food.
/// Swampcycling {2}.
pub fn canyon_crawler() -> CardDefinition {
    CardDefinition {
        name: "Canyon Crawler",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![
            Keyword::Deathtouch,
            Keyword::Landcycling(ManaCost::new(vec![ManaSymbol::Generic(2)]), LandType::Swamp),
        ],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: food_token(),
        })],
        ..Default::default()
    }
}

/// Foggy Swamp Hunters — {3}{B} 3/4 Human Ranger Ally. Has lifelink and menace
/// as long as you've drawn two or more cards this turn.
pub fn foggy_swamp_hunters() -> CardDefinition {
    let while_drew2 = |kw: Keyword| StaticAbility {
        description: "Lifelink and menace while you've drawn two or more cards this turn.",
        effect: StaticEffect::SelfHasKeywordWhile {
            keyword: kw,
            condition: SelectionRequirement::ControllerDrewAtLeastThisTurn(2),
        },
    };
    CardDefinition {
        name: "Foggy Swamp Hunters",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Ranger]),
        power: 3,
        toughness: 4,
        static_abilities: vec![while_drew2(Keyword::Lifelink), while_drew2(Keyword::Menace)],
        ..Default::default()
    }
}

/// June, Bounty Hunter — {1}{B} 2/2 legendary Human Mercenary. Can't be blocked
/// while you've drawn two or more cards this turn. {1}, Sacrifice another
/// creature: investigate. (Activate as a sorcery — approximates "during your
/// turn.")
pub fn june_bounty_hunter() -> CardDefinition {
    CardDefinition {
        name: "June, Bounty Hunter",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Can't be blocked while you've drawn two or more cards this turn.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Unblockable,
                condition: SelectionRequirement::ControllerDrewAtLeastThisTurn(2),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: investigate(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fire Sages — {1}{R} 2/2 Human Cleric. Firebending 1. {1}{R}{R}: put a +1/+1
/// counter on this creature.
pub fn fire_sages() -> CardDefinition {
    CardDefinition {
        name: "Fire Sages",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Firebending(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Azula, On the Hunt — {3}{B} 4/3 legendary Human Noble. Firebending 2. When
/// it attacks, you lose 1 life and investigate.
pub fn azula_on_the_hunt() -> CardDefinition {
    CardDefinition {
        name: "Azula, On the Hunt",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Firebending(2)],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            investigate(1),
        ]))],
        ..Default::default()
    }
}

/// Earth King's Lieutenant — {G}{W} 1/1 Human Soldier Ally. Trample. ETB: put a
/// +1/+1 counter on each other Ally you control. Whenever another Ally enters,
/// put a +1/+1 counter on this creature.
pub fn earth_kings_lieutenant() -> CardDefinition {
    let other_ally = || {
        SelectionRequirement::HasCreatureType(CreatureType::Ally)
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::OtherThanSource)
    };
    CardDefinition {
        name: "Earth King's Lieutenant",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: Selector::EachPermanent(other_ally()),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: other_ally(),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Sandbenders' Storm — {3}{W} Instant. Choose one — destroy target creature
/// with power 4 or greater; or earthbend 3.
pub fn sandbenders_storm() -> CardDefinition {
    CardDefinition {
        name: "Sandbenders' Storm",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                },
                Effect::Earthbend { n: Value::Const(3) },
            ],
        },
        ..Default::default()
    }
}

/// Airbender's Reversal — {1}{W} Instant (Lesson). Choose one — destroy target
/// attacking creature; or airbend target creature you control.
pub fn airbenders_reversal() -> CardDefinition {
    CardDefinition {
        name: "Airbender's Reversal",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
                    ),
                },
                Effect::Airbend {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                },
            ],
        },
        ..Default::default()
    }
}

/// Day of Black Sun — {X}{B}{B} Sorcery. Each creature with mana value X or
/// less loses all abilities until end of turn, then is destroyed.
pub fn day_of_black_sun() -> CardDefinition {
    let small = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ManaValueAtMostXFromCost),
        )
    };
    CardDefinition {
        name: "Day of Black Sun",
        cost: cost(&[x(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LoseAllAbilities { what: small(), duration: Duration::EndOfTurn },
            Effect::Destroy { what: small() },
        ]),
        ..Default::default()
    }
}

/// Master Piandao — {4}{W} 4/4 legendary Human Warrior Ally. First strike. On
/// attack, look at the top four and may put an Ally/Equipment/Lesson into hand.
pub fn master_piandao() -> CardDefinition {
    CardDefinition {
        name: "Master Piandao",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Warrior, CreatureType::Ally]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![on_attack(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::HasCreatureType(CreatureType::Ally)
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Equipment))
                    .or(SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson)),
            ),
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Beetle-Headed Merchants — {4}{B} 5/4 Human Citizen. On attack, you may
/// sacrifice another creature or artifact to draw a card and grow this.
pub fn beetle_headed_merchants() -> CardDefinition {
    CardDefinition {
        name: "Beetle-Headed Merchants",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature or artifact?".into(),
            filter: (SelectionRequirement::Creature.or(SelectionRequirement::Artifact))
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Lo and Li, Twin Tutors — {4}{B} 2/2 legendary Human Advisor. ETB: search for
/// a Lesson or Noble card to hand. Your Noble creatures have lifelink.
pub fn lo_and_li_twin_tutors() -> CardDefinition {
    CardDefinition {
        name: "Lo and Li, Twin Tutors",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson)
                .or(SelectionRequirement::HasCreatureType(CreatureType::Noble)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![StaticAbility {
            description: "Noble creatures you control have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Noble)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        ..Default::default()
    }
}

/// Fire Navy Trebuchet — {2}{B} 0/4 artifact Wall. Defender, reach. Whenever
/// you attack, make a tapped-and-attacking 2/1 flying Construct (Ballistic
/// Boulder).
pub fn fire_navy_trebuchet() -> CardDefinition {
    CardDefinition {
        name: "Fire Navy Trebuchet",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Ballistic Boulder".into(),
                    power: 2,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Construct],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
                cleanup: Default::default(),
            },
        }],
        ..Default::default()
    }
}

/// Hog-Monkey — {2}{B} 3/2 Boar Monkey. At the beginning of combat on your
/// turn, a target creature you control with a +1/+1 counter gains menace.
pub fn hog_monkey() -> CardDefinition {
    CardDefinition {
        name: "Hog-Monkey",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar, CreatureType::Monkey],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithAnyCounter),
                ),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Energybending — {2} Instant (Lesson). Lands you control gain all basic land
/// types until end of turn, then draw a card.
pub fn energybending() -> CardDefinition {
    CardDefinition {
        name: "Energybending",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::GainAllBasicLandTypes {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Swampsnare Trap — {2}{B} Aura. Enchanted creature gets -5/-3. (The cost
/// reduction vs. fliers is dropped.)
pub fn swampsnare_trap() -> CardDefinition {
    CardDefinition {
        name: "Swampsnare Trap",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: -5,
            toughness: -3,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Flopsie, Bumi's Buddy — {4}{G}{G} 4/4 legendary Ape Goat. ETB: +1/+1 counter
/// on each creature you control. Your power-4+ creatures can't be blocked by
/// more than one creature.
pub fn flopsie_bumis_buddy() -> CardDefinition {
    CardDefinition {
        name: "Flopsie, Bumi's Buddy",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Goat],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        static_abilities: vec![StaticAbility {
            description: "Your power-4+ creatures can't be blocked by more than one creature.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(4)),
                ),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
            },
        }],
        ..Default::default()
    }
}

/// Professor Zei, Anthropologist — {U/R}{U/R} 0/3 legendary Human Advisor Ally.
/// {T}, Discard a card: Draw a card. {1}, {T}, Sacrifice him: return an instant
/// or sorcery from your graveyard to hand (sorcery speed).
pub fn professor_zei_anthropologist() -> CardDefinition {
    CardDefinition {
        name: "Professor Zei, Anthropologist",
        cost: cost(&[hybrid(Color::Blue, Color::Red), hybrid(Color::Blue, Color::Red)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Advisor]),
        power: 0,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((SelectionRequirement::Any, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                            .and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Foggy Swamp Spirit Keeper — {1}{U}{B} 2/4 Human Druid Ally. Lifelink. On
/// your second draw each turn, make a 1/1 Spirit token. (The token's
/// Spirit-only block restriction is approximated as a vanilla 1/1.)
pub fn foggy_swamp_spirit_keeper() -> CardDefinition {
    CardDefinition {
        name: "Foggy Swamp Spirit Keeper",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Druid]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Spirit".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Spirit],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

// ── TLA "refuge" dual lands (enters tapped; {T}: Add A or B; {4},{T},Sac: draw) ─

/// One TLA two-color sac-land: enters tapped, taps for either color, and can be
/// sacrificed for a card. Untyped (no basic land subtypes).
fn tla_sac_land(name: &'static str, color_a: Color, color_b: Color) -> CardDefinition {
    use super::super::{etb_tap, tap_add};
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add(color_a),
            tap_add(color_b),
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![etb_tap()],
        ..Default::default()
    }
}

pub fn north_pole_gates() -> CardDefinition { tla_sac_land("North Pole Gates", Color::White, Color::Blue) }
pub fn serpents_pass() -> CardDefinition { tla_sac_land("Serpent's Pass", Color::Blue, Color::Black) }
pub fn boiling_rock_prison() -> CardDefinition { tla_sac_land("Boiling Rock Prison", Color::Black, Color::Red) }
pub fn omashu_city() -> CardDefinition { tla_sac_land("Omashu City", Color::Red, Color::Green) }
pub fn kyoshi_village() -> CardDefinition { tla_sac_land("Kyoshi Village", Color::Green, Color::White) }
pub fn misty_palms_oasis() -> CardDefinition { tla_sac_land("Misty Palms Oasis", Color::White, Color::Black) }
pub fn airship_engine_room() -> CardDefinition { tla_sac_land("Airship Engine Room", Color::Blue, Color::Red) }
pub fn foggy_bottom_swamp() -> CardDefinition { tla_sac_land("Foggy Bottom Swamp", Color::Black, Color::Green) }
pub fn sun_blessed_peak() -> CardDefinition { tla_sac_land("Sun-Blessed Peak", Color::Red, Color::White) }
pub fn meditation_pools() -> CardDefinition { tla_sac_land("Meditation Pools", Color::Green, Color::Blue) }

/// Fire Nation Cadets — {R} 1/2 Human Soldier. Has firebending 2 while a Lesson
/// is in your graveyard; {2}: +1/+0 until end of turn.
pub fn fire_nation_cadets() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Cadets",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Has firebending 2 while a Lesson card is in your graveyard.",
            effect: StaticEffect::PumpSelfIf {
                condition: lesson_in_graveyard(),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Firebending(2)],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Firebending Lesson — {R} Instant — Lesson. Kicker {4}. Deals 2 damage to
/// target creature, or 5 if it was kicked.
pub fn firebending_lesson() -> CardDefinition {
    CardDefinition {
        name: "Firebending Lesson",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(5),
            }),
            else_: Box::new(Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}

/// Mongoose Lizard — {4}{R}{R} 5/6 Menace. ETB deals 1 damage to any target.
/// Mountaincycling {2}.
pub fn mongoose_lizard() -> CardDefinition {
    CardDefinition {
        name: "Mongoose Lizard",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mongoose, CreatureType::Lizard],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![
            Keyword::Menace,
            Keyword::Landcycling(cost(&[generic(2)]), LandType::Mountain),
        ],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Seismic Sense — {G} Sorcery — Lesson. Look at the top X cards (X = lands you
/// control); you may reveal a creature or land from among them to your hand,
/// the rest to the bottom.
pub fn seismic_sense() -> CardDefinition {
    CardDefinition {
        name: "Seismic Sense",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::count(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            )),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::Creature.or(SelectionRequirement::Land),
            ),
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Origin of Metalbending — {1}{G} Instant — Lesson. Choose one — destroy target
/// artifact or enchantment; or put a +1/+1 counter on target creature you
/// control and it gains indestructible until end of turn.
pub fn origin_of_metalbending() -> CardDefinition {
    CardDefinition {
        name: "Origin of Metalbending",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
                ),
            },
            Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Deadly Precision — {B} Sorcery. Additional cost: pay {4} or sacrifice an
/// artifact or creature. Destroy target creature.
pub fn deadly_precision() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Deadly Precision",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay {
            filter: SelectionRequirement::HasCardType(CardType::Artifact)
                .or(SelectionRequirement::Creature),
            pay: 4,
        }],
        effect: Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Enter the Avatar State — {W} Instant — Lesson. Until end of turn, target
/// creature you control gains flying, first strike, lifelink, and hexproof.
/// (The "becomes an Avatar" type-add is dropped — no additive type primitive.)
pub fn enter_the_avatar_state() -> CardDefinition {
    let tgt = || target_filtered(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Enter the Avatar State",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::GrantKeyword { what: tgt(), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Gran-Gran — {U} 1/2 legendary Human Peasant Ally. When it becomes tapped,
/// draw then discard. Noncreature spells you cast cost {1} less while 3+ Lesson
/// cards are in your graveyard (`StaticEffect::CostReductionWhile`).
pub fn gran_gran() -> CardDefinition {
    CardDefinition {
        name: "Gran-Gran",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Peasant, CreatureType::Ally]),
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast cost {1} less as long as there are three or more Lesson cards in your graveyard.",
            effect: StaticEffect::CostReductionWhile {
                filter: SelectionRequirement::Noncreature,
                amount: 1,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
                    },
                    n: Value::Const(3),
                },
            },
        }],
        ..Default::default()
    }
}

/// South Pole Voyager — {1}{W} 2/2 Human Scout Ally. Whenever this or another
/// Ally you control enters, gain 1 life; if it's the second time this ability
/// resolved this turn, draw a card. The "only the 2nd" is modeled with an
/// `EscalatingThisTurn` whose 1st/3rd+ branches are no-ops.
pub fn south_pole_voyager() -> CardDefinition {
    CardDefinition {
        name: "South Pole Voyager",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Scout, CreatureType::Ally]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Ally),
                }),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
                Effect::EscalatingThisTurn {
                    modes: vec![
                        Effect::Noop,
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::Noop,
                    ],
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Hermitic Herbalist — {G}{U} 2/3 Human Druid Ally. `{T}: Add one mana of any
/// color.` and `{T}: Add two mana in any combination of colors. Spend this mana
/// only to cast Lesson spells.`
pub fn hermitic_herbalist() -> CardDefinition {
    CardDefinition {
        name: "Hermitic Herbalist",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid, CreatureType::Ally],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyColors(Value::Const(2))),
                        SpendRestriction::LessonSpellsOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Firebending X / Lesson bounce ──────────────────────────────────────────

/// Firebending Student — {1}{R} 1/2 Human Monk. Prowess; firebending X, where X
/// is this creature's power (`Keyword::FirebendingPower`).
pub fn firebending_student() -> CardDefinition {
    CardDefinition {
        name: "Firebending Student",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Prowess, Keyword::FirebendingPower],
        ..Default::default()
    }
}

/// Boomerang Basics — {U} Sorcery — Lesson. Return target nonland permanent to
/// its owner's hand; if you controlled it, draw a card. The controller check
/// runs before the bounce so it reads the still-on-battlefield permanent.
pub fn boomerang_basics() -> CardDefinition {
    CardDefinition {
        name: "Boomerang Basics",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::ControlledByYou,
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 2: earthbend / Raid / mill / kicker control ──────────────────────

/// A 2/2 red Soldier token with firebending 1 (Cruel Administrator).
fn firebending_soldier_token() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        keywords: vec![Keyword::Firebending(1)],
        ..Default::default()
    }
}

/// Earth Kingdom General — {3}{G} 2/2 Human Soldier Ally. When it enters,
/// earthbend 2.
pub fn earth_kingdom_general() -> CardDefinition {
    CardDefinition {
        name: "Earth Kingdom General",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Soldier, CreatureType::Ally]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::Const(2) })],
        ..Default::default()
    }
}

/// Cruel Administrator — {3}{B}{R} 5/4 Human Soldier. Raid — enters with a
/// +1/+1 counter if you attacked this turn. Whenever it attacks, create a 2/2
/// red Soldier with firebending 1.
pub fn cruel_administrator() -> CardDefinition {
    CardDefinition {
        name: "Cruel Administrator",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        triggered_abilities: vec![
            raid_etb(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            on_attack(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: firebending_soldier_token(),
            }),
        ],
        ..Default::default()
    }
}

/// Sparring Dummy — {1}{G} 1/3 Artifact Creature — Scarecrow. Defender. {T}:
/// Mill a card; you may put a land milled this way into your hand. (The "gain 2
/// life if a Lesson is milled" rider is dropped.)
pub fn sparring_dummy() -> CardDefinition {
    CardDefinition {
        name: "Sparring Dummy",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scarecrow],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::MillThenToHand {
                amount: Value::ONE,
                filter: SelectionRequirement::Land,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Buzzard-Wasp Colony — {3}{B} 2/2 Bird Insect. Flying. When it enters, you
/// may sacrifice an artifact or creature; if you do, draw a card. (The
/// "another creature you control dies → move its counters onto this" rider is
/// dropped — observer death-triggers can't read the dead creature's LKI
/// counters yet; tracked in TODO.md.)
pub fn buzzard_wasp_colony() -> CardDefinition {
    CardDefinition {
        name: "Buzzard-Wasp Colony",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Insect],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice an artifact or creature?".into(),
            filter: SelectionRequirement::HasCardType(CardType::Artifact)
                .or(SelectionRequirement::Creature),
            count: Value::ONE,
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Jet's Brainwashing — {R} Sorcery. Kicker {3}. Target creature can't block
/// this turn; if kicked, also gain control of it until end of turn, untap it,
/// and it gains haste.
pub fn jets_brainwashing() -> CardDefinition {
    CardDefinition {
        name: "Jet's Brainwashing",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::Target(0),
                        to: Some(PlayerRef::You),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 3: equipment + modal burn ────────────────────────────────────────

/// Meteor Sword — {7} Equipment. When it enters, destroy target permanent.
/// Equipped creature gets +3/+3. Equip {3}.
pub fn meteor_sword() -> CardDefinition {
    CardDefinition {
        name: "Meteor Sword",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 3, toughness: 3, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(SelectionRequirement::Permanent),
        })],
        ..Default::default()
    }
}

/// Kyoshi Battle Fan — {2} Equipment. When it enters, create a 1/1 white Ally
/// and attach this to it (living-weapon shape). Equipped creature gets +1/+0.
/// Equip {2}.
pub fn kyoshi_battle_fan() -> CardDefinition {
    CardDefinition {
        name: "Kyoshi Battle Fan",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 0, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: ally_token() },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        ..Default::default()
    }
}

/// Bumi Bash — {3}{R} Sorcery. Choose one — deal damage equal to the lands you
/// control to target creature; or destroy target land creature or nonbasic land.
pub fn bumi_bash() -> CardDefinition {
    CardDefinition {
        name: "Bumi Bash",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                )),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Land.and(SelectionRequirement::Creature)
                        .or(SelectionRequirement::IsNonbasicLand),
                ),
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 4: Exhaust (CR 702.177) ──────────────────────────────────────────

/// Rebellious Captives — {1}{G} 2/2 Human Peasant Ally. Exhaust — {6}: put two
/// +1/+1 counters on this creature, then earthbend 2.
pub fn rebellious_captives() -> CardDefinition {
    CardDefinition {
        name: "Rebellious Captives",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: ally(&[CreatureType::Human, CreatureType::Peasant, CreatureType::Ally]),
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::Earthbend { n: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rough Rhino Cavalry — {4}{R} 5/5 Human Mercenary. Firebending 2. Exhaust —
/// {8}: put two +1/+1 counters on this creature; it gains trample.
pub fn rough_rhino_cavalry() -> CardDefinition {
    CardDefinition {
        name: "Rough Rhino Cavalry",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Firebending(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            exhaust: true,
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// Mai, Jaded Edge already ships in `sets::eoe` (mis-filed there); reuse it.

// ── Batch 5: Aura / earthbend / Vehicle / conditional anthem ───────────────

/// Path to Redemption — {1}{W} Aura. Enchanted creature can't attack or block.
/// {5}, Sacrifice this Aura: exile the enchanted creature, create a 1/1 white
/// Ally. (Activate only as a sorcery.)
pub fn path_to_redemption() -> CardDefinition {
    CardDefinition {
        name: "Path to Redemption",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
            ..Default::default()
        }),
        // "{5}, Sacrifice this Aura: exile enchanted creature, create an Ally."
        // Modeled as exile-the-host first (the now-hostless Aura is put into the
        // graveyard by the illegally-attached SBA, CR 704.5n) so the host is
        // still readable when the effect resolves.
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: Selector::AttachedTo(Box::new(Selector::This)) },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: ally_token() },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dai Li Agents — {3}{B}{G} 3/4 Human Soldier. When it enters, earthbend 1
/// twice.
pub fn dai_li_agents() -> CardDefinition {
    CardDefinition {
        name: "Dai Li Agents",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Earthbend { n: Value::ONE },
            Effect::Earthbend { n: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Fire Nation Warship — {3} Artifact Vehicle 4/4. Reach. When it dies, create
/// a Clue. Crew 2.
pub fn fire_nation_warship() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Warship",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::Crew(2)],
        // A Vehicle "dies" → leaves the battlefield for the graveyard (fires
        // whether or not it was crewed when destroyed).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: investigate(1),
        }],
        ..Default::default()
    }
}

/// Earth Rumble Wrestlers — {3}{R/G} 3/4 Human Warrior Performer. Reach. Gets
/// +1/+0 and trample while you control a land creature or a land entered this
/// turn (the latter approximated by a land *played* this turn).
pub fn earth_rumble_wrestlers() -> CardDefinition {
    CardDefinition {
        name: "Earth Rumble Wrestlers",
        cost: cost(&[generic(3), hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        static_abilities: vec![StaticAbility {
            description: "+1/+0 and trample while you control a land creature or a land entered this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::Any(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::Land
                            .and(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                    Predicate::ValueAtLeast(
                        Value::LandsPlayedThisTurn(PlayerRef::You),
                        Value::ONE,
                    ),
                ]),
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

// ── Batch 6: loot Lesson + attack-with-others draw ─────────────────────────

/// Abandon Attachments — {1}{U/R} Instant — Lesson. You may discard a card; if
/// you do, draw two.
pub fn abandon_attachments() -> CardDefinition {
    CardDefinition {
        name: "Abandon Attachments",
        cost: cost(&[generic(1), hybrid(Color::Blue, Color::Red)]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::MayDo {
            description: "Discard a card to draw two?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ])),
        },
        ..Default::default()
    }
}

/// Sokka, Lateral Strategist — {1}{W/U}{W/U} 2/4 legendary Human Warrior Ally.
/// Vigilance. Whenever Sokka and at least one other creature attack, draw a card.
pub fn sokka_lateral_strategist() -> CardDefinition {
    CardDefinition {
        name: "Sokka, Lateral Strategist",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Blue), hybrid(Color::White, Color::Blue)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Warrior, CreatureType::Ally]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                Predicate::ValueAtLeast(
                    Value::CreaturesAttackedWithThisTurn(PlayerRef::You),
                    Value::Const(2),
                ),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

// ── Batch 7: magecraft Clue / ETB Food / Flash tapper / attack earthbend X ──

/// The Mechanist, Aerial Artisan — {2}{U} 1/3 legendary Human Artificer Ally.
/// Whenever you cast a noncreature spell, create a Clue. (The {T} token-animate
/// ability is dropped.)
pub fn the_mechanist_aerial_artisan() -> CardDefinition {
    CardDefinition {
        name: "The Mechanist, Aerial Artisan",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Artificer, CreatureType::Ally]),
        power: 1,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::magecraft(investigate(1))],
        ..Default::default()
    }
}

/// Iroh, Tea Master — {1}{R}{W} 2/2 legendary Human Citizen Ally. When it
/// enters, create a Food token. (The combat donate ability is dropped.)
pub fn iroh_tea_master() -> CardDefinition {
    CardDefinition {
        name: "Iroh, Tea Master",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Citizen, CreatureType::Ally]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: food_token(),
        })],
        ..Default::default()
    }
}

/// Ty Lee, Chi Blocker — {2}{U} 2/1 legendary Human Monk Ally. Flash. Prowess.
/// When it enters, tap up to one target creature; it doesn't untap during its
/// controller's next untap step.
pub fn ty_lee_chi_blocker() -> CardDefinition {
    CardDefinition {
        name: "Ty Lee, Chi Blocker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Monk, CreatureType::Ally]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::Target(0) },
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ])),
        })],
        ..Default::default()
    }
}

/// The Boulder, Ready to Rumble — {3}{G} 4/4 legendary Human Warrior. Whenever
/// it attacks, earthbend X, where X is the number of creatures you control with
/// power 4 or greater.
pub fn the_boulder_ready_to_rumble() -> CardDefinition {
    CardDefinition {
        name: "The Boulder, Ready to Rumble",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::Earthbend {
            n: Value::count(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::PowerAtLeast(4)),
            )),
        })],
        ..Default::default()
    }
}

// ── Batch 8: token-maker / lifegain manadork / anthem + token ──────────────

/// The Earth King — {3}{G} 2/2 legendary Human Noble Ally. When it enters,
/// create a 4/4 green Bear. (The attack-with-power-4+ land tutor is dropped.)
pub fn the_earth_king() -> CardDefinition {
    let bear = TokenDefinition {
        name: "Bear".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bear], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Earth King",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Noble, CreatureType::Ally]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: bear,
        })],
        ..Default::default()
    }
}

/// The Lion-Turtle — {1}{G}{U} 3/6 legendary Cat Turtle. Vigilance, reach. When
/// it enters, gain 3 life. {T}: Add one mana of any color. (The Lesson-gated
/// attack/block restriction is dropped.)
pub fn the_lion_turtle() -> CardDefinition {
    CardDefinition {
        name: "The Lion-Turtle",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Turtle],
            ..Default::default()
        },
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Suki, Courageous Rescuer — {1}{W}{W} 2/4 legendary Human Warrior Ally. Other
/// creatures you control get +1/+0. Whenever another permanent you control
/// leaves the battlefield during your turn, create a 1/1 white Ally (once each
/// turn).
pub fn suki_courageous_rescuer() -> CardDefinition {
    CardDefinition {
        name: "Suki, Courageous Rescuer",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Warrior, CreatureType::Ally]),
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        // The "another permanent you control leaves during your turn → Ally
        // token (once/turn)" rider is dropped — observer leaves-battlefield
        // triggers can't read the departed permanent's LKI yet (TODO.md).
        ..Default::default()
    }
}
