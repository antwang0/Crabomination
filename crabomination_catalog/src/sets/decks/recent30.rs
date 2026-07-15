//! A thirtieth wave — Aetherdrift (DFT) staples on existing primitives:
//! Vehicles + Crew, Mounts + Saddle, Exhaust abilities, Start your engines! /
//! max-speed gates, reanimation, modal removal, and graveyard value. Tests in
//! `crabomination/src/tests/recent30.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    Effect, EventKind, EventScope, EventSpec, ExileReturnZone, Keyword, Predicate,
    SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{attacks_while_saddled, deal, etb, target_any, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn cycling(c: crate::mana::ManaCost) -> Keyword {
    Keyword::Cycling(c)
}

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

fn vehicle(subs: Vec<ArtifactSubtype>) -> Subtypes {
    Subtypes { artifact_subtypes: subs, ..Default::default() }
}

/// Burner Rocket — {1}{R} 3/1 Vehicle with flash. ETB: target creature you
/// control gets +2/+0 and gains trample until end of turn. Crew 1.
pub fn burner_rocket() -> CardDefinition {
    CardDefinition {
        name: "Burner Rocket",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(vec![ArtifactSubtype::Vehicle]),
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Clamorous Ironclad — {3}{R} 6/3 Vehicle with menace. Crew 3. Cycling {R}.
pub fn clamorous_ironclad() -> CardDefinition {
    CardDefinition {
        name: "Clamorous Ironclad",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(vec![ArtifactSubtype::Vehicle]),
        power: 6,
        toughness: 3,
        keywords: vec![Keyword::Menace, Keyword::Crew(3), cycling(cost(&[r()]))],
        ..Default::default()
    }
}

/// Broadcast Rambler — {4}{W} 5/4 Vehicle. ETB: create a 1/1 flying Thopter.
/// Crew 1.
pub fn broadcast_rambler() -> CardDefinition {
    CardDefinition {
        name: "Broadcast Rambler",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(vec![ArtifactSubtype::Vehicle]),
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: thopter_token(),
        })],
        ..Default::default()
    }
}

/// Carrion Cruiser — {2}{B} 3/2 Vehicle. ETB: mill two, then return a creature
/// or Vehicle card from your graveyard to your hand. Crew 1.
pub fn carrion_cruiser() -> CardDefinition {
    CardDefinition {
        name: "Carrion Cruiser",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(vec![ArtifactSubtype::Vehicle]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Crew(1)],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::ReturnGraveyardCardsToHand {
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                max: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Alacrian Jaguar — {4}{G} 4/4 Cat Mount with vigilance. Whenever it attacks
/// while saddled, it gets +2/+2 until end of turn. Saddle 1.
pub fn alacrian_jaguar() -> CardDefinition {
    CardDefinition {
        name: "Alacrian Jaguar",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Mount],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Saddle(1)],
        triggered_abilities: vec![attacks_while_saddled(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Brightfield Glider — {W} 1/1 Possum Mount with vigilance. Whenever it
/// attacks while saddled, it gets +1/+2 and gains flying. Saddle 3.
pub fn brightfield_glider() -> CardDefinition {
    CardDefinition {
        name: "Brightfield Glider",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Possum, CreatureType::Mount],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance, Keyword::Saddle(3)],
        triggered_abilities: vec![attacks_while_saddled(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// District Mascot — {G} 0/0 Dog Mount. Enters with a +1/+1 counter. Whenever
/// it attacks while saddled, put a +1/+1 counter on it. {1}{G}, Remove two
/// +1/+1 counters: Destroy target artifact. Saddle 1.
pub fn district_mascot() -> CardDefinition {
    CardDefinition {
        name: "District Mascot",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Mount],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Saddle(1)],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![attacks_while_saddled(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 2)),
            effect: Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bulwark Ox — {1}{W} 2/2 Ox Mount. Whenever it attacks while saddled, put a
/// +1/+1 counter on target creature. Sacrifice it: creatures you control with
/// counters gain hexproof and indestructible until end of turn. Saddle 1.
pub fn bulwark_ox() -> CardDefinition {
    let counter_creatures = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::WithAnyCounter),
        )
    };
    CardDefinition {
        name: "Bulwark Ox",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ox, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Saddle(1)],
        triggered_abilities: vec![attacks_while_saddled(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: counter_creatures(),
                    keyword: Keyword::Hexproof,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: counter_creatures(),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Autarch Mammoth — {4}{G}{G} 5/5 Elephant Mount. When it enters and whenever
/// it attacks while saddled, create a 3/3 green Elephant. Saddle 5.
pub fn autarch_mammoth() -> CardDefinition {
    let elephant = || TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elephant], ..Default::default() },
        ..Default::default()
    };
    let make = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: elephant(),
    };
    CardDefinition {
        name: "Autarch Mammoth",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Mount],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Saddle(5)],
        triggered_abilities: vec![etb(make()), attacks_while_saddled(make())],
        ..Default::default()
    }
}

/// Earthrumbler — {4}{G} 7/6 Vehicle with vigilance and trample. Crew 3.
/// (The exile-from-graveyard self-crew alternative is dropped.)
pub fn earthrumbler() -> CardDefinition {
    CardDefinition {
        name: "Earthrumbler",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(vec![ArtifactSubtype::Vehicle]),
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Crew(3)],
        ..Default::default()
    }
}

/// Elvish Refueler — {2}{G} 2/3 Elf Druid. Exhaust — {1}{G}: Put a +1/+1
/// counter on it. (The "activate exhaust abilities twice" rider is dropped.)
pub fn elvish_refueler() -> CardDefinition {
    CardDefinition {
        name: "Elvish Refueler",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            exhaust: true,
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

/// Endrider Catalyzer — {1}{R} 3/1 Human Warrior. Start your engines! Max speed
/// — {T}: Add {R}{R}.
pub fn endrider_catalyzer() -> CardDefinition {
    CardDefinition {
        name: "Endrider Catalyzer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 }),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Red]),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glitch Ghost Surveyor — {2}{U} 2/2 Spirit Scout with flying. Start your
/// engines! (The max-speed graveyard draw is dropped.)
pub fn glitch_ghost_surveyor() -> CardDefinition {
    CardDefinition {
        name: "Glitch Ghost Surveyor",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::StartYourEngines],
        ..Default::default()
    }
}

/// Collision Course — {1}{W} Sorcery. Choose one — deal X damage to target
/// creature, where X is the number of creatures and Vehicles you control; or
/// destroy target artifact.
pub fn collision_course() -> CardDefinition {
    CardDefinition {
        name: "Collision Course",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            deal_value(
                Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle))
                        .and(SelectionRequirement::ControlledByYou),
                ))),
                target_filtered(SelectionRequirement::Creature),
            ),
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
        ]),
        ..Default::default()
    }
}

/// Back on Track — {4}{B} Sorcery. Return target creature or Vehicle card from
/// your graveyard to the battlefield, then create a 1/1 colorless Pilot.
/// (The Pilot's enhanced crew/saddle rider is dropped.)
pub fn back_on_track() -> CardDefinition {
    let pilot = TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pilot], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Back on Track",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: pilot },
        ]),
        ..Default::default()
    }
}

/// Dredger's Insight — {1}{G} Enchantment. ETB: mill four, you may take an
/// artifact/creature/land card. Whenever one or more artifact/creature cards
/// leave your graveyard, gain 1 life.
pub fn dredgers_insight() -> CardDefinition {
    CardDefinition {
        name: "Dredger's Insight",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: true,
                pick_filter: Some(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .or(SelectionRequirement::Land),
                ),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Artifact
                            .or(SelectionRequirement::Creature),
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// "Deal `amount` damage to `target`" with a dynamic amount.
fn deal_value(amount: Value, target: Selector) -> Effect {
    Effect::DealDamage { to: target, amount }
}

/// Dracosaur Auxiliary — {4}{R}{R} 4/4 Dinosaur Dragon Mount with flying and
/// haste. Whenever it attacks while saddled, it deals 2 damage to any target.
/// Saddle 3.
pub fn dracosaur_auxiliary() -> CardDefinition {
    CardDefinition {
        name: "Dracosaur Auxiliary",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Dragon, CreatureType::Mount],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste, Keyword::Saddle(3)],
        triggered_abilities: vec![attacks_while_saddled(deal(2, target_any()))],
        ..Default::default()
    }
}

/// Detention Chariot — {4}{W}{W} 6/6 Vehicle. ETB: exile target artifact or
/// creature an opponent controls until this leaves. Crew 3. Cycling {W}.
pub fn detention_chariot() -> CardDefinition {
    CardDefinition {
        name: "Detention Chariot",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: vehicle(vec![ArtifactSubtype::Vehicle]),
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Crew(3), cycling(cost(&[w()]))],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Creature)
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Endrider Spikespitter — {3}{R} 3/4 Human Mercenary with reach. Start your
/// engines! Max speed — at the beginning of your upkeep, exile the top card and
/// you may play it this turn.
pub fn endrider_spikespitter() -> CardDefinition {
    CardDefinition {
        name: "Endrider Spikespitter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach, Keyword::StartYourEngines],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: Predicate::SpeedAtLeast { who: PlayerRef::You, speed: 4 },
                then: Box::new(Effect::LookTopExileOneMayPlay { count: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Aether Syphon — {1}{U}{U} Artifact. Start your engines! {2}, {T}: Draw a
/// card. (The max-speed draw-mill rider is dropped.)
pub fn aether_syphon() -> CardDefinition {
    CardDefinition {
        name: "Aether Syphon",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::StartYourEngines],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Alacrian Armory — {3}{W} Artifact. Creatures you control get +0/+1 and have
/// vigilance. (The begin-combat saddle/crew rider is dropped.)
pub fn alacrian_armory() -> CardDefinition {
    CardDefinition {
        name: "Alacrian Armory",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +0/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: 0,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Vigilance,
                },
            },
        ],
        ..Default::default()
    }
}
