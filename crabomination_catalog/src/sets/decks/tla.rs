//! The Last Airbender (TLA) staples on existing primitives — Allies, hybrid
//! costs, attack/ETB triggers, and a defensive anthem. Tests in
//! `crabomination/src/tests/tla.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, Effect, EnchantmentSubtype, EventKind, EventScope, EventSpec, ExileReturnZone,
    Keyword, LandType, Predicate, SelectionRequirement, Selector, SpellSubtype, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crabomination_base::tokens::{clue_token, food_token};
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

// ── Batch 9: dig-for-Lesson / Prowess attack tempo ─────────────────────────

/// Guru Pathik — {2}{G/U}{G/U} 2/4 legendary Human Monk Ally. When it enters,
/// look at the top five cards; you may put a Lesson, Saga, or Shrine from among
/// them into your hand, the rest to the bottom.
pub fn guru_pathik() -> CardDefinition {
    CardDefinition {
        name: "Guru Pathik",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::Blue), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Monk, CreatureType::Ally]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(
                SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson)
                    .or(SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Saga))
                    .or(SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Shrine)),
            ),
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Ty Lee, Artful Acrobat — {2}{R} 3/2 legendary Human Monk. Prowess. Whenever
/// she attacks, you may pay {1}; if you do, target creature can't block this
/// turn.
pub fn ty_lee_artful_acrobat() -> CardDefinition {
    CardDefinition {
        name: "Ty Lee, Artful Acrobat",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![on_attack(Effect::MayPay {
            description: "Pay {1} to make a creature unable to block?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            }),
            else_: None,
        })],
        ..Default::default()
    }
}

// ── Batch 10: firebending bodies ───────────────────────────────────────────

/// Uncle Iroh — {1}{R/G}{R/G} 4/2 legendary Human Noble Ally. Firebending 1.
/// Lesson spells you cast cost {1} less.
pub fn uncle_iroh() -> CardDefinition {
    CardDefinition {
        name: "Uncle Iroh",
        cost: cost(&[generic(1), hybrid(Color::Red, Color::Green), hybrid(Color::Red, Color::Green)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: ally(&[CreatureType::Human, CreatureType::Noble, CreatureType::Ally]),
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Firebending(1)],
        static_abilities: vec![StaticAbility {
            description: "Lesson spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Vindictive Warden — {2}{B/R} 2/3 Human Soldier. Menace, firebending 1. {3}:
/// deals 1 damage to each opponent.
pub fn vindictive_warden() -> CardDefinition {
    CardDefinition {
        name: "Vindictive Warden",
        cost: cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace, Keyword::Firebending(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Batch 11: enchantment value, sacrifice/death payoffs, firebending ───────

/// Air Nomad Legacy — {W}{U} Enchantment. ETB: create a Clue. Creatures you
/// control with flying get +1/+1.
pub fn air_nomad_legacy() -> CardDefinition {
    CardDefinition {
        name: "Air Nomad Legacy",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(investigate(1))],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with flying get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasKeyword(Keyword::Flying)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// True Ancestry — {1}{G} Sorcery (Lesson). Return up to one target permanent
/// card from your graveyard to your hand. Create a Clue.
pub fn true_ancestry() -> CardDefinition {
    CardDefinition {
        name: "True Ancestry",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Permanent),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Tolls of War — {W}{B} Enchantment. ETB: create a Clue. Whenever you
/// sacrifice a permanent during your turn, create a 1/1 white Ally (once/turn).
pub fn tolls_of_war() -> CardDefinition {
    CardDefinition {
        name: "Tolls of War",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(investigate(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You))
                    .once_per_turn(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: ally_token(),
                },
            },
        ],
        ..Default::default()
    }
}

/// Long Feng, Grand Secretariat — {1}{B/G}{B/G} 2/3 legendary Human Advisor.
/// Whenever another creature you control dies, put a +1/+1 counter on target
/// creature you control. (The "or a land" clause is approximated to creatures.)
pub fn long_feng_grand_secretariat() -> CardDefinition {
    CardDefinition {
        name: "Long Feng, Grand Secretariat",
        cost: cost(&[
            generic(1),
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
        ]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Zhao, Ruthless Admiral — {2}{B/R}{B/R} 3/4 legendary Human Soldier.
/// Firebending 2. Whenever you sacrifice another permanent, creatures you
/// control get +1/+0 until end of turn.
pub fn zhao_ruthless_admiral() -> CardDefinition {
    CardDefinition {
        name: "Zhao, Ruthless Admiral",
        cost: cost(&[
            generic(2),
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Firebending(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::OtherThanSource,
                }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Zuko, Exiled Prince — {3}{R} 4/3 legendary Human Noble. Firebending 3.
/// {3}: exile the top card of your library; you may play it this turn.
pub fn zuko_exiled_prince() -> CardDefinition {
    CardDefinition {
        name: "Zuko, Exiled Prince",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Firebending(3)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Beifong's Bounty Hunters — {2}{B}{G} 4/4 Human Mercenary. Whenever a nonland
/// creature you control dies, earthbend X, where X is that creature's power.
pub fn beifongs_bounty_hunters() -> CardDefinition {
    CardDefinition {
        name: "Beifong's Bounty Hunters",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Nonland,
                },
            ),
            effect: Effect::Earthbend { n: Value::PowerOf(Box::new(Selector::TriggerSource)) },
        }],
        ..Default::default()
    }
}

// ── Batch 12: vehicles, equipment, Ally lords, combat payoffs ───────────────

/// Tundra Tank — {2}{B} Vehicle 4/4. Firebending 1. ETB: target creature you
/// control gains indestructible until end of turn. Crew 1.
pub fn tundra_tank() -> CardDefinition {
    CardDefinition {
        name: "Tundra Tank",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Firebending(1), Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Indestructible,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Twin Blades — {2}{R} Equipment. Flash. ETB: attach to target creature you
/// control; it gains double strike until end of turn. Equipped +1/+1. Equip {2}.
pub fn twin_blades() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Twin Blades",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Vengeful Villagers — {3}{W} 3/3 Human Citizen. On attack: tap target creature
/// an opponent controls, then you may sacrifice an artifact or creature; if you
/// do, put a stun counter on it.
pub fn vengeful_villagers() -> CardDefinition {
    CardDefinition {
        name: "Vengeful Villagers",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Citizen],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent)) },
            Effect::MaySacrifice {
                description: "Sacrifice an artifact or creature to stun the tapped creature?".into(),
                filter: (SelectionRequirement::Artifact.or(SelectionRequirement::Creature))
                    .and(SelectionRequirement::ControlledByYou),
                count: Value::ONE,
                then: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::ONE,
                }),
                else_: None,
            },
        ]))],
        ..Default::default()
    }
}

/// Invasion Tactics — {4}{G} Enchantment. ETB: creatures you control get +2/+2
/// until end of turn. Whenever one or more Allies you control deal combat
/// damage to a player, draw a card.
pub fn invasion_tactics() -> CardDefinition {
    CardDefinition {
        name: "Invasion Tactics",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Ally),
                    })
                    .once_per_turn(),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Jet, Freedom Fighter — {2}{R/W}{R/W}{R/W} 3/1 legendary Human Rebel Ally.
/// ETB: deal damage equal to the number of creatures you control to target
/// creature an opponent controls. Dies: +1/+1 counter on each of up to two
/// target creatures.
pub fn jet_freedom_fighter() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        name: "Jet, Freedom Fighter",
        cost: cost(&[generic(2), rw(), rw(), rw()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::CreatureCountControlledBy(PlayerRef::You),
            }),
            on_dies(Effect::ApplyToTargets {
                max_targets: 2,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            }),
        ],
        ..Default::default()
    }
}

/// Sold Out — {3}{B} Instant. Exile target creature. If it was dealt damage
/// this turn, create a Clue.
pub fn sold_out() -> CardDefinition {
    CardDefinition {
        name: "Sold Out",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::DealtDamageThisTurn,
                },
                then: Box::new(investigate(1)),
                else_: Box::new(Effect::Noop),
            },
            Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ]),
        ..Default::default()
    }
}

/// Sokka, Tenacious Tactician — {1}{U}{R}{W} 3/3 legendary Human Warrior Ally.
/// Menace, prowess. Other Allies you control have menace and prowess. Whenever
/// you cast a noncreature spell, create a 1/1 white Ally.
pub fn sokka_tenacious_tactician() -> CardDefinition {
    let other_allies = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Ally)
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        )
    };
    CardDefinition {
        name: "Sokka, Tenacious Tactician",
        cost: cost(&[generic(1), u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace, Keyword::Prowess],
        static_abilities: vec![
            StaticAbility {
                description: "Other Allies you control have menace.",
                effect: StaticEffect::GrantKeyword { applies_to: other_allies(), keyword: Keyword::Menace },
            },
            StaticAbility {
                description: "Other Allies you control have prowess.",
                effect: StaticEffect::GrantKeyword { applies_to: other_allies(), keyword: Keyword::Prowess },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                },
            ),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: ally_token() },
        }],
        ..Default::default()
    }
}

/// Team Avatar — {2}{W} Enchantment. Whenever a creature you control attacks
/// alone, it gets +X/+X where X is the number of creatures you control.
/// {2}{W}, Discard this card: deal damage equal to creatures you control to
/// target creature.
pub fn team_avatar() -> CardDefinition {
    CardDefinition {
        name: "Team Avatar",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::AttackingAlone),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::CreatureCountControlledBy(PlayerRef::You),
                toughness: Value::CreatureCountControlledBy(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            discard_self_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::CreatureCountControlledBy(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Batch 13: modal Bison, copy-while-attacking, X-earthbend, token flashback ─

/// Appa, Loyal Sky Bison — {4}{W}{W} 4/4 legendary Bison Ally. Flying. When
/// Appa enters or attacks, choose one — target creature you control gains
/// flying; or airbend another target nonland permanent you control.
pub fn appa_loyal_sky_bison() -> CardDefinition {
    let modal = || Effect::ChooseN {
        picks: vec![0],
        modes: vec![
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::Airbend {
                what: target_filtered(
                    SelectionRequirement::Nonland
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
            },
        ],
    };
    CardDefinition {
        name: "Appa, Loyal Sky Bison",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bison, CreatureType::Ally],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(modal()), on_attack(modal())],
        ..Default::default()
    }
}

/// Fire Lord Azula — {1}{U}{B}{R} 4/4 legendary Human Noble. Firebending 2.
/// Whenever you cast a spell while Azula is attacking, copy that spell (you may
/// choose new targets). (Modeled as "while Azula attacked this turn".)
pub fn fire_lord_azula() -> CardDefinition {
    CardDefinition {
        name: "Fire Lord Azula",
        cost: cost(&[generic(1), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Firebending(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::SourceAttackedThisTurn),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::TriggerSource,
                count: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Rockalanche — {2}{G} Sorcery (Lesson). Earthbend X, where X is the number of
/// Forests you control. Flashback {5}{G}.
pub fn rockalanche() -> CardDefinition {
    CardDefinition {
        name: "Rockalanche",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        keywords: vec![Keyword::Flashback(cost(&[generic(5), g()]))],
        effect: Effect::Earthbend {
            n: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasLandType(LandType::Forest),
            },
        },
        ..Default::default()
    }
}

/// Fire Nation Attacks — {4}{R} Instant. Create two 2/2 red Soldier tokens with
/// firebending 1. Flashback {8}{R}.
pub fn fire_nation_attacks() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Attacks",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(8), r()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: firebending_soldier_token(),
        },
        ..Default::default()
    }
}

// ── Batch 14: multi-target Lessons, player-scoped pump ──────────────────────

/// How to Start a Riot — {2}{R} Instant (Lesson). Target creature gains menace
/// until end of turn. Creatures target player controls get +2/+0 until EOT.
pub fn how_to_start_a_riot() -> CardDefinition {
    CardDefinition {
        name: "How to Start a Riot",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::ControlledBy {
                    who: PlayerRef::Target(1),
                    filter: SelectionRequirement::Creature,
                },
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Lost Days — {4}{U} Instant (Lesson). The owner of target creature or
/// enchantment puts it into their library second from the top. Create a Clue.
/// (The printed "or on the bottom" owner's-choice rider is fixed to 2nd-from-top.)
pub fn lost_days() -> CardDefinition {
    CardDefinition {
        name: "Lost Days",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
                ),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: crate::effect::LibraryPosition::FromTop(1),
                },
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Sokka's Haiku — {3}{U}{U} Instant (Lesson). Counter target spell. Draw a
/// card, then mill three. Untap target land.
pub fn sokkas_haiku() -> CardDefinition {
    CardDefinition {
        name: "Sokka's Haiku",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Any },
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::Untap {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::HasCardType(CardType::Land),
                },
                up_to: None,
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 15: Shrine cycle + mana rock ──────────────────────────────────────

/// A 1/1 red Monk creature token with prowess (Crescent Island Temple).
fn monk_prowess_token() -> TokenDefinition {
    TokenDefinition {
        name: "Monk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Monk], ..Default::default() },
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    }
}

/// Number of Shrines you control (counts the source Shrine on ETB).
fn shrines_you_control() -> Value {
    Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Shrine),
    }
}

/// "Whenever another Shrine you control enters, `effect`."
fn on_another_shrine_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::HasEnchantmentSubtype(EnchantmentSubtype::Shrine),
            }),
        effect,
    }
}

/// Crescent Island Temple — {3}{R} Legendary Enchantment — Shrine. ETB: for
/// each Shrine you control, make a 1/1 red Monk with prowess. Another Shrine
/// enters → make one.
pub fn crescent_island_temple() -> CardDefinition {
    CardDefinition {
        name: "Crescent Island Temple",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: shrines_you_control(),
                definition: monk_prowess_token(),
            }),
            on_another_shrine_enters(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: monk_prowess_token(),
            }),
        ],
        ..Default::default()
    }
}

/// Southern Air Temple — {3}{W} Legendary Enchantment — Shrine. ETB: put X
/// +1/+1 counters on each creature you control (X = Shrines). Another Shrine
/// enters → +1/+1 on each.
pub fn southern_air_temple() -> CardDefinition {
    let each_creature = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Southern Air Temple",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::AddCounter {
                what: each_creature(),
                kind: CounterType::PlusOnePlusOne,
                amount: shrines_you_control(),
            }),
            on_another_shrine_enters(Effect::AddCounter {
                what: each_creature(),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        ],
        ..Default::default()
    }
}

/// Waterbending Scroll — {1}{U} Artifact. {6}, {T}: draw a card. This ability
/// costs {1} less to activate for each Island you control.
pub fn waterbending_scroll() -> CardDefinition {
    CardDefinition {
        name: "Waterbending Scroll",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            cost_reduction_per: Some(SelectionRequirement::HasLandType(LandType::Island)),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Batch 16: Shrine cycle finish, X-creature ──────────────────────────────

/// Kyoshi Island Plaza — {3}{G} Legendary Enchantment — Shrine. ETB: search
/// for up to X basic lands (X = Shrines you control) onto the battlefield
/// tapped. Another Shrine enters → search a basic onto the battlefield tapped.
pub fn kyoshi_island_plaza() -> CardDefinition {
    CardDefinition {
        name: "Kyoshi Island Plaza",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Shrine],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: shrines_you_control(),
            }),
            on_another_shrine_enters(Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
        ],
        ..Default::default()
    }
}

/// Wan Shi Tong, Librarian — {X}{U}{U} 1/1 legendary Bird Spirit. Flash. Flying,
/// vigilance. ETB: put X +1/+1 counters on him, then draw half X (rounded down).
/// (The "opponent searches → grow + draw" rider is dropped — no search trigger.)
pub fn wan_shi_tong_librarian() -> CardDefinition {
    CardDefinition {
        name: "Wan Shi Tong, Librarian",
        cost: cost(&[x(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::HalfDown(Box::new(Value::XFromCost)),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 17: artifacts / vehicles ──────────────────────────────────────────

/// Bender's Waterskin — {3} Artifact. Untap it during each other player's untap
/// step. {T}: Add one mana of any color.
pub fn benders_waterskin() -> CardDefinition {
    CardDefinition {
        name: "Bender's Waterskin",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Untap this artifact during each other player's untap step.",
            effect: StaticEffect::UntapSelfEachUntapStep,
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Fire Nation Drill — {2}{B}{B} Legendary Artifact — Vehicle 6/3. Trample.
/// ETB: you may tap it; if you do, destroy target creature with power 4 or less.
/// {1}: permanents your opponents control lose hexproof and indestructible
/// until end of turn. Crew 2.
pub fn the_fire_nation_drill() -> CardDefinition {
    let opp_permanents =
        || Selector::ControlledBy { who: PlayerRef::EachOpponent, filter: SelectionRequirement::Any };
    CardDefinition {
        name: "The Fire Nation Drill",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 6,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Crew(2)],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Tap The Fire Nation Drill to destroy a creature with power 4 or less?"
                .to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Tap { what: Selector::This },
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(4)),
                    ),
                },
            ])),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Seq(vec![
                Effect::LoseKeywordThisTurn { what: opp_permanents(), keyword: Keyword::Hexproof },
                Effect::LoseKeywordThisTurn {
                    what: opp_permanents(),
                    keyword: Keyword::Indestructible,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Iroh, Grand Lotus — {3}{G}{U}{R} 5/5 legendary Human Noble Ally. Firebending
/// 2. Each instant/sorcery card in your graveyard has flashback for its mana
/// cost. (The "during your turn" gate and the cheaper Lesson flashback {1} are
/// approximated to Lier's always-on grant.)
pub fn iroh_grand_lotus() -> CardDefinition {
    CardDefinition {
        name: "Iroh, Grand Lotus",
        cost: cost(&[generic(3), g(), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble, CreatureType::Ally],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Firebending(2)],
        static_abilities: vec![StaticAbility {
            description: "Each instant and sorcery card in your graveyard has flashback equal to its mana cost.",
            effect: StaticEffect::GraveyardInstantsSorceriesHaveFlashback,
        }],
        ..Default::default()
    }
}

/// Suki, Kyoshi Warrior — {2}{G/W}{G/W} */4 Human Warrior Ally. Power = the
/// number of creatures you control; attacks → create a tapped, attacking 1/1
/// white Ally.
pub fn suki_kyoshi_warrior() -> CardDefinition {
    CardDefinition {
        name: "Suki, Kyoshi Warrior",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::White), hybrid(Color::Green, Color::White)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        dynamic_pt: Some(DynamicPt::CreaturesControlledPower { base_p: 0, base_t: 4 }),
        triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: ally_token(),
            cleanup: Default::default(),
        })],
        ..Default::default()
    }
}

/// Toph, the Blind Bandit — {2}{G} */3 Human Warrior Ally. ETB earthbend 2;
/// power = the +1/+1 counters on lands you control.
pub fn toph_the_blind_bandit() -> CardDefinition {
    CardDefinition {
        name: "Toph, the Blind Bandit",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        dynamic_pt: Some(DynamicPt::PlusCountersOnLandsControlledPower { base_p: 0, base_t: 3 }),
        triggered_abilities: vec![etb(Effect::Earthbend { n: Value::Const(2) })],
        ..Default::default()
    }
}

/// Cycle of Renewal — {2}{G} Instant — Lesson. Sacrifice a land, then search
/// for up to two basic lands onto the battlefield tapped.
pub fn cycle_of_renewal() -> CardDefinition {
    CardDefinition {
        name: "Cycle of Renewal",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: SelectionRequirement::Land },
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Zuko's Exile — {5} Instant — Lesson. Exile target artifact, creature, or
/// enchantment; its controller creates a Clue token.
pub fn zukos_exile() -> CardDefinition {
    CardDefinition {
        name: "Zuko's Exile",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::ONE,
                definition: clue_token(),
            },
            Effect::Exile {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Enchantment),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Sokka, Bold Boomeranger — {U}{R} 1/1 Human Warrior Ally. ETB loots up to
/// two; gains a +1/+1 counter whenever you cast an artifact or Lesson spell.
pub fn sokka_bold_boomeranger() -> CardDefinition {
    CardDefinition {
        name: "Sokka, Bold Boomeranger",
        cost: cost(&[u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            // "Discard up to two" approximated as discard any number, draw that many.
            etb(Effect::Seq(vec![
                Effect::DiscardAnyNumber { who: Selector::You },
                Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Artifact
                            .or(SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson)),
                    },
                ),
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

/// Sun Warriors — {2}{R}{W} 3/5 Human Warrior Ally. Firebending X (X = the
/// number of creatures you control); {5}: create a 1/1 white Ally token.
pub fn sun_warriors() -> CardDefinition {
    CardDefinition {
        name: "Sun Warriors",
        cost: cost(&[generic(2), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::FirebendingCreaturesYouControl],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: ally_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Razor Rings — {1}{W} Instant. Deals 4 damage to target attacking or
/// blocking creature; you gain life equal to the excess damage dealt this way.
pub fn razor_rings() -> CardDefinition {
    CardDefinition {
        name: "Razor Rings",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking)),
                ),
                amount: Value::Const(4),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ExcessDamageDealtThisResolution,
            },
        ]),
        ..Default::default()
    }
}

/// The Last Agni Kai — {1}{R} Instant. Target creature you control fights
/// target creature an opponent controls; add {R} equal to the excess damage
/// dealt this way. (The "don't lose unspent red mana" rider is omitted.)
pub fn the_last_agni_kai() -> CardDefinition {
    CardDefinition {
        name: "The Last Agni Kai",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::ExcessDamageDealtThisResolution),
            },
        ]),
        ..Default::default()
    }
}

/// Hei Bai, Spirit of Balance — {2}{W/B}{W/B} 3/3 Legendary Bear Spirit.
/// Enters or attacks: may sacrifice another creature/artifact for two +1/+1
/// counters. On leaving the battlefield, moves its counters to target creature
/// you control.
pub fn hei_bai_spirit_of_balance() -> CardDefinition {
    let sac_for_counters = || {
        Effect::MaySacrifice {
            description: "Sacrifice another creature or artifact: two +1/+1 counters on Hei Bai"
                .into(),
            filter: (SelectionRequirement::Creature.or(SelectionRequirement::Artifact))
                .and(SelectionRequirement::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
            else_: None,
        }
    };
    CardDefinition {
        name: "Hei Bai, Spirit of Balance",
        cost: cost(&[generic(2), hybrid(Color::White, Color::Black), hybrid(Color::White, Color::Black)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(sac_for_counters()),
            on_attack(sac_for_counters()),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::MoveAllCounters {
                    from: Selector::This,
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                },
            },
        ],
        ..Default::default()
    }
}

/// Zuko's Conviction — {B} Instant, Kicker {4}. Return target creature card
/// from your graveyard to your hand; if kicked, put it onto the battlefield
/// tapped instead.
pub fn zukos_conviction() -> CardDefinition {
    CardDefinition {
        name: "Zuko's Conviction",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Kicker(cost(&[generic(4)]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            else_: Box::new(Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Barrels of Blasting Jelly — {1} Artifact. {1}: Add one mana of any color,
/// once each turn. {5}, {T}, Sacrifice: deal 5 damage to target creature.
pub fn barrels_of_blasting_jelly() -> CardDefinition {
    CardDefinition {
        name: "Barrels of Blasting Jelly",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                once_per_turn: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(5)]),
                effect: Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Creature),
                    amount: Value::Const(5),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Accumulate Wisdom — {1}{U} Instant — Lesson. Look at the top three; put one
/// into your hand (rest to bottom), or all three if you have 3+ Lessons in gy.
pub fn accumulate_wisdom() -> CardDefinition {
    let dig = |take: Value| Effect::LookPickToHand {
        who: PlayerRef::You,
        count: Value::Const(3),
        rest_to_graveyard: false,
        pick_filter: None,
        take: Some(take),
        to_battlefield: false,
    };
    CardDefinition {
        name: "Accumulate Wisdom",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: lesson(),
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
                },
                Value::Const(3),
            ),
            then: Box::new(dig(Value::Const(3))),
            else_: Box::new(dig(Value::ONE)),
        },
        ..Default::default()
    }
}

/// Dragonfly Swarm — {1}{U}{R} */3 Dragon Insect. Flying, ward {1}. Power =
/// noncreature, nonland cards in your graveyard. Dies → draw if a Lesson is
/// in your graveyard.
pub fn dragonfly_swarm() -> CardDefinition {
    CardDefinition {
        name: "Dragonfly Swarm",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Insect],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        dynamic_pt: Some(DynamicPt::NoncreatureNonlandCardsInControllerGraveyard { base_t: 3 }),
        triggered_abilities: vec![on_dies(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson),
                },
                Value::ONE,
            ),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Enters tapped unless you control a basic land (the TLA mono-land cycle).
fn tapped_unless_basic() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                SelectionRequirement::IsBasicLand.and(SelectionRequirement::ControlledByYou),
            )),
            then: Box::new(Effect::Noop),
            else_: Box::new(Effect::Tap { what: Selector::This }),
        },
    }
}

fn tap_for(color: Color) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![color]) },
        ..Default::default()
    }
}

/// Abandoned Air Temple — Land. Enters tapped unless you control a basic land.
/// {T}: Add {W}. {3}{W}, {T}: Put a +1/+1 counter on each creature you control.
pub fn abandoned_air_temple() -> CardDefinition {
    CardDefinition {
        name: "Abandoned Air Temple",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![tapped_unless_basic()],
        activated_abilities: vec![
            tap_for(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), w()]),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Agna Qel'a — Land. Enters tapped unless you control a basic land.
/// {T}: Add {U}. {2}{U}, {T}: Draw a card, then discard a card.
pub fn agna_qela() -> CardDefinition {
    CardDefinition {
        name: "Agna Qel'a",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![tapped_unless_basic()],
        activated_abilities: vec![
            tap_for(Color::Blue),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ba Sing Se — Land. Enters tapped unless you control a basic land.
/// {T}: Add {G}. {2}{G}, {T}: Earthbend 2. Activate only as a sorcery.
pub fn ba_sing_se() -> CardDefinition {
    CardDefinition {
        name: "Ba Sing Se",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![tapped_unless_basic()],
        activated_abilities: vec![
            tap_for(Color::Green),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), g()]),
                sorcery_speed: true,
                effect: Effect::Earthbend { n: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Fire Nation Palace — Land. Enters tapped unless you control a basic land.
/// {T}: Add {R}. {1}{R}, {T}: Target creature you control gains firebending 4
/// until end of turn.
pub fn fire_nation_palace() -> CardDefinition {
    CardDefinition {
        name: "Fire Nation Palace",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![tapped_unless_basic()],
        activated_abilities: vec![
            tap_for(Color::Red),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), r()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Firebending(4),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Price of Freedom — {1}{R} Sorcery — Lesson. Destroy target artifact or land
/// an opponent controls; its controller may fetch a basic land tapped. Draw.
pub fn price_of_freedom() -> CardDefinition {
    CardDefinition {
        name: "Price of Freedom",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: lesson(),
        effect: Effect::Seq(vec![
            // Search the target's controller's library first, while the target
            // (and thus its controller) is still on the battlefield.
            Effect::SearchUpToN {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    tapped: true,
                },
                count: Value::ONE,
            },
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: (SelectionRequirement::Artifact.or(SelectionRequirement::Land))
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

fn realm_of_koh_spirit() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        // "Can't be blocked by non-Spirit creatures"; the symmetric "can't block
        // non-Spirit creatures" half is dropped (no can-block-only filter yet).
        keywords: vec![Keyword::CantBeBlockedExceptBy(Box::new(
            SelectionRequirement::HasCreatureType(CreatureType::Spirit),
        ))],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

/// Realm of Koh — Land. Enters tapped unless you control a basic land.
/// {T}: Add {B}. {3}{B}, {T}: Create a 1/1 colorless Spirit token.
pub fn realm_of_koh() -> CardDefinition {
    CardDefinition {
        name: "Realm of Koh",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![tapped_unless_basic()],
        activated_abilities: vec![
            tap_for(Color::Black),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), b()]),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: realm_of_koh_spirit(),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Earthen Ally — {G} */2 Human Soldier Ally. Gets +1/+0 for each color among
/// Allies you control. {W}{U}{B}{R}{G}: Earthbend 5.
pub fn earthen_ally() -> CardDefinition {
    CardDefinition {
        name: "Earthen Ally",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Ally],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        dynamic_pt: Some(DynamicPt::ColorsAmongAlliesControlledPower { base_p: 0, base_t: 2 }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u(), b(), r(), g()]),
            effect: Effect::Earthbend { n: Value::Const(5) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leaves from the Vine — {1}{G} Enchantment — Saga. I: mill 3, create a Food.
/// II: +1/+1 on up to two target creatures you control. III: draw if a creature
/// or Lesson card is in your graveyard.
pub fn leaves_from_the_vine() -> CardDefinition {
    CardDefinition {
        name: "Leaves from the Vine",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: vec![
            (1, Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(3) },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: food_token() },
            ])),
            (2, Effect::SupportCounters {
                max_targets: 2,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            }),
            (3, Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: SelectionRequirement::Creature
                            .or(SelectionRequirement::HasSpellSubtype(SpellSubtype::Lesson)),
                    },
                    Value::ONE,
                ),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            }),
        ],
        ..Default::default()
    }
}

/// Rumble Arena — Land. When it enters, scry 1. {T}: Add {C}. {1}, {T}: Add one
/// mana of any color.
pub fn rumble_arena() -> CardDefinition {
    CardDefinition {
        name: "Rumble Arena",
        card_types: vec![CardType::Land],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hakoda, Selfless Commander — {3}{W} 3/5 Human Warrior Ally. Vigilance; play
/// with the top card revealed and cast Ally spells from it; Sacrifice Hakoda:
/// creatures you control get +0/+5 and gain indestructible until end of turn.
pub fn hakoda_selfless_commander() -> CardDefinition {
    let team = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Hakoda, Selfless Commander",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may cast Ally spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop {
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Ally),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: team(),
                    power: Value::Const(0),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: team(),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Obsessive Pursuit — {1}{B} Enchantment. ETB and at each upkeep: lose 1 life,
/// make a Clue. Whenever you attack, put X +1/+1 counters on target attacking
/// creature (X = permanents you've sacrificed this turn); at X≥3 it gains
/// lifelink.
pub fn obsessive_pursuit() -> CardDefinition {
    let drain_clue = || Effect::Seq(vec![
        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
        investigate(1),
    ]);
    CardDefinition {
        name: "Obsessive Pursuit",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(drain_clue()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: drain_clue(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::IsAttacking),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::PermanentsSacrificedThisTurn(PlayerRef::You),
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::PermanentsSacrificedThisTurn(PlayerRef::You),
                            Value::Const(3),
                        ),
                        then: Box::new(Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Lifelink,
                            duration: Duration::EndOfTurn,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Combustion Man — {3}{R}{R} 4/6 Legendary Human Assassin. Whenever he
/// attacks, destroy target permanent unless its controller takes damage equal
/// to his power. (Punisher cost modeled as life loss = his power.)
pub fn combustion_man() -> CardDefinition {
    CardDefinition {
        name: "Combustion Man",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        triggered_abilities: vec![on_attack(Effect::UnlessPlayerPays {
            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            cost: crate::card::WardCost::LifeSourcePower,
            then: Box::new(Effect::Destroy {
                what: target_filtered(SelectionRequirement::Permanent),
            }),
        })],
        ..Default::default()
    }
}

/// Teo, Spirited Glider — {3}{U} 1/4 Legendary Human Pilot Ally. Flying.
/// Whenever one or more creatures you control with flying attack, loot 1; if a
/// nonland card was discarded that way, put a +1/+1 counter on target creature
/// you control.
pub fn teo_spirited_glider() -> CardDefinition {
    CardDefinition {
        name: "Teo, Spirited Glider",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pilot, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::HasKeyword(Keyword::Flying),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
                    then: Box::new(Effect::AddCounter {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Bitter Work — {1}{R}{G} Enchantment. Whenever you attack with one or more
/// creatures of power 4+, draw a card. Exhaust — {4}: Earthbend 4 (your turn).
pub fn bitter_work() -> CardDefinition {
    CardDefinition {
        name: "Bitter Work",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::AttackedWithCreatureMatching {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::PowerAtLeast(4),
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            exhaust: true,
            sorcery_speed: true,
            effect: Effect::Earthbend { n: Value::Const(4) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Momo, Friendly Flier — {W} 1/1 Legendary Lemur Bat Ally. Flying; whenever
/// another creature you control with flying enters, Momo gets +1/+1 until end
/// of turn. (The "first flyer each turn costs {1} less" rider is omitted —
/// no first-spell-of-type cost reduction yet.)
pub fn momo_friendly_flier() -> CardDefinition {
    CardDefinition {
        name: "Momo, Friendly Flier",
        cost: cost(&[w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lemur, CreatureType::Bat, CreatureType::Ally],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
