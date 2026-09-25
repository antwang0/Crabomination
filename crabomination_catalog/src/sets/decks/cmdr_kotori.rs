//! Commander: the cards the **Buckle Up** precon (NEC, Kotori, Pilot Prodigy)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_kotori.rs`.
//!
//! Residuals (each also on its card):
//! - **Armed and Armored** — "choose a Dwarf" takes the engine's pick and
//!   attaches every Equipment you control.
//! - **Katsumasa, the Animator** / **Dance of the Manse** — "up to" target
//!   counts take what the targeter picks.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{counter_target_spell, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, hybrid, u, w, x, Color, ManaCost};

fn vehicle() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Vehicle)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn artifact() -> R {
    R::HasCardType(CardType::Artifact)
}

/// Historic (CR 700.6): artifacts, legendaries and Sagas.
fn historic() -> R {
    artifact()
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga))
}

fn vehicle_card(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..Default::default()
    }
}

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

fn on_artifact_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(artifact())),
        effect,
    }
}

fn animate_until_eot(what: Selector) -> Effect {
    Effect::AnimateAsCreature { what, duration: Duration::EndOfTurn }
}

/// Kotori, Pilot Prodigy — your Vehicles have crew 2; at the beginning of
/// combat on your turn, target artifact creature you control gains lifelink
/// and vigilance.
pub fn kotori_pilot_prodigy() -> CardDefinition {
    let target = || target_filtered(R::Creature.and(artifact()).and(R::ControlledByYou));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "Vehicles you control have crew 2.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(vehicle()), keyword: Keyword::Crew(2) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: target(), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: target(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature(
            "Kotori, Pilot Prodigy",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Pilot],
            2,
            4,
        )
    }
}

/// Access Denied — counter target spell; X 1/1 flying Thopters, X its mana
/// value.
pub fn access_denied() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Access Denied",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            counter_target_spell(),
            Effect::CreateToken { who: PlayerRef::You, count: Value::CounteredSpellManaValue, definition: Arc::new(thopter) },
        ]),
        ..Default::default()
    }
}

/// Aerial Surveyor — flying Vehicle; attacking a player with more lands than
/// you fetches a basic Plains tapped. Crew 2.
pub fn aerial_surveyor() -> CardDefinition {
    let lands = |who| Value::PermanentCountControlledByMatching(who, R::Land);
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::ValueAtLeast(
                lands(PlayerRef::DefendingPlayer),
                Value::Sum(vec![lands(PlayerRef::You), Value::ONE]),
            )),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasSupertype(Supertype::Basic).and(R::HasLandType(crate::card::LandType::Plains)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..vehicle_card("Aerial Surveyor", cost(&[generic(2), w()]), 3, 4, 2)
    }
}

/// Aeronaut Admiral — flying; your Vehicles have flying.
pub fn aeronaut_admiral() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Vehicles you control have flying.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(vehicle()), keyword: Keyword::Flying },
        }],
        ..creature("Aeronaut Admiral", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Pilot], 3, 1)
    }
}

/// Arcanist's Owl — flying; entering digs four for an artifact or enchantment.
pub fn arcanists_owl() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            rest_bottom_random: true,
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(artifact().or(R::HasCardType(CardType::Enchantment))),
            optional: true,
            ..Default::default()
        })))],
        ..creature("Arcanist's Owl", cost(&[wu(), wu(), wu(), wu()]), vec![CreatureType::Bird], 3, 3)
    }
}

/// Armed and Armored — your Vehicles become artifact creatures until end of
/// turn; attach your Equipment to a Dwarf you control. ⚠ The engine picks the
/// Dwarf and attaches every Equipment.
pub fn armed_and_armored() -> CardDefinition {
    CardDefinition {
        name: "Armed and Armored",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            animate_until_eot(yours(vehicle())),
            Effect::AttachAnyNumberTo {
                what: yours(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                to: Selector::one_of(yours(R::Creature.and(R::HasCreatureType(CreatureType::Dwarf)))),
            },
        ]),
        ..Default::default()
    }
}

/// Colossal Plow — attacking adds {W}{W}{W} that lasts the turn and gains 3
/// life. Crew 6.
pub fn colossal_plow() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::AddManaKeptThisTurn { who: PlayerRef::You, colors: vec![Color::White, Color::White, Color::White] },
            Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..vehicle_card("Colossal Plow", cost(&[generic(2)]), 6, 3, 6)
    }
}

/// Dance of the Manse — up to X artifact and/or non-Aura enchantment cards of
/// mana value X or less return from your graveyard; at X 6+ they're 4/4
/// creatures too.
pub fn dance_of_the_manse() -> CardDefinition {
    let filter = artifact()
        .or(R::HasCardType(CardType::Enchantment).and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)))))
        .and(R::ManaValueAtMostXFromCost)
        .from_your_graveyard();
    CardDefinition {
        name: "Dance of the Manse",
        cost: cost(&[x(), w(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CapTargetsAt {
            amount: Value::XFromCost,
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 10,
                min_targets: 0,
                filter,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(6)),
                        then: Box::new(Effect::BecomeCreature {
                            what: Selector::LastMoved,
                            power: Value::Const(4),
                            toughness: Value::Const(4),
                            creature_types: vec![],
                            keywords: vec![],
                            duration: Duration::Permanent,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            }),
        },
        ..Default::default()
    }
}

/// Imperial Recovery Unit — attacking returns a creature or Vehicle card of
/// mana value 2 or less from your graveyard to hand. Crew 2.
pub fn imperial_recovery_unit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::Move {
            what: target_filtered(R::Creature.or(vehicle()).and(R::ManaValueAtMost(2)).from_your_graveyard()),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..vehicle_card("Imperial Recovery Unit", cost(&[generic(2), w()]), 3, 4, 2)
    }
}

/// Imposter Mech — may enter as a copy of an opponent's creature, except it's
/// a Vehicle artifact with crew 3 and no other type. Crew 3.
pub fn imposter_mech() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByOpponent),
            as_vehicle_crew: Some(3),
            ..Default::default()
        }),
        ..vehicle_card("Imposter Mech", cost(&[generic(1), u()]), 3, 1, 3)
    }
}

/// Ironsoul Enforcer — whenever it or a commander you control attacks alone,
/// return target artifact card from your graveyard to the battlefield.
pub fn ironsoul_enforcer() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::AttackingAlone,
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource.or(R::IsCommander) },
            ])),
            effect: Effect::Move {
                what: target_filtered(artifact().from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature("Ironsoul Enforcer", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Samurai], 4, 4)
    }
}

/// Katsumasa, the Animator — flying; {2}{U}: target noncreature artifact of
/// yours flies as an artifact creature (1/1 unless a Vehicle) this turn; at
/// your upkeep, a +1/+1 counter on each of up to three target noncreature
/// artifacts.
pub fn katsumasa_the_animator() -> CardDefinition {
    let noncreature_artifact = || artifact().and(R::Not(Box::new(R::Creature)));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: vehicle() },
                    then: Box::new(animate_until_eot(target_filtered(noncreature_artifact().and(R::ControlledByYou)))),
                    else_: Box::new(Effect::BecomeCreature {
                        what: target_filtered(noncreature_artifact().and(R::ControlledByYou)),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        creature_types: vec![],
                        keywords: vec![],
                        duration: Duration::EndOfTurn,
                    }),
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: noncreature_artifact(),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature(
            "Katsumasa, the Animator",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Moonfolk, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Mobilizer Mech — flying; becoming crewed animates up to one other Vehicle
/// of yours. Crew 3.
pub fn mobilizer_mech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        triggered_abilities: vec![TriggeredAbility {
            // The crew event's subject is the Vehicle: "this Vehicle becomes crewed".
            event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsSource }),
            effect: animate_until_eot(target_filtered(vehicle().and(R::ControlledByYou).and(R::OtherThanSource))),
        }],
        ..vehicle_card("Mobilizer Mech", cost(&[generic(1), u()]), 3, 4, 3)
    }
}

/// Myrsmith — an artifact spell may pay {1} for a 1/1 Myr.
pub fn myrsmith() -> CardDefinition {
    let myr = TokenDefinition {
        name: "Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![on_artifact_cast(Effect::MayPay {
            description: "Pay {1} to create a 1/1 Myr?".into(),
            mana_cost: cost(&[generic(1)]),
            body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(myr) }),
            else_: None,
        })],
        ..creature("Myrsmith", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Peacewalker Colossus — {1}{W}: another Vehicle of yours becomes an artifact
/// creature until end of turn. Crew 4.
pub fn peacewalker_colossus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: animate_until_eot(target_filtered(vehicle().and(R::ControlledByYou).and(R::OtherThanSource))),
            ..Default::default()
        }],
        ..vehicle_card("Peacewalker Colossus", cost(&[generic(3)]), 6, 6, 4)
    }
}

/// Prodigy's Prototype — Vehicles of yours attacking make a Pilot that crews
/// as though its power were 2 greater. Crew 2.
pub fn prodigys_prototype() -> CardDefinition {
    let pilot = TokenDefinition {
        name: "Pilot".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pilot], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token crews Vehicles as though its power were 2 greater.",
            effect: StaticEffect::CrewSaddlePowerBonus { applies_to: Selector::This, amount: 2 },
        }],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: vehicle() })
                .once_per_batch(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(pilot) },
        }],
        ..vehicle_card("Prodigy's Prototype", cost(&[generic(1), w(), u()]), 3, 4, 2)
    }
}

/// Raff Capashen, Ship's Mage — flash, flying; historic spells have flash.
pub fn raff_capashen_ships_mage() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may cast historic spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: historic() },
        }],
        ..creature(
            "Raff Capashen, Ship's Mage",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Raiders' Karve — attacking, a land on top of your library may enter
/// tapped. Crew 3.
pub fn raiders_karve() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::If {
            cond: Predicate::EntityMatchesAny { what: top(), filter: R::Land },
            then: Box::new(Effect::MayDo {
                description: "Put the land on top of your library onto the battlefield tapped?".into(),
                body: Box::new(Effect::Move {
                    what: top(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..vehicle_card("Raiders' Karve", cost(&[generic(3)]), 4, 4, 3)
    }
}

/// Release to Memory — exile target opponent's graveyard; a 1/1 Spirit per
/// creature card exiled.
pub fn release_to_memory() -> CardDefinition {
    let spirit = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Release to Memory",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::TargetPlayerThen {
            filter: R::OpponentPlayer,
            then: Box::new(Effect::Seq(vec![
                Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountOf(Box::new(Selector::ExiledThisResolution { filter: R::Creature })),
                    definition: Arc::new(spirit),
                },
            ])),
        },
        ..Default::default()
    }
}

/// Riddlesmith — an artifact spell lets you loot.
pub fn riddlesmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_artifact_cast(Effect::MayDo {
            description: "Draw a card, then discard a card?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ])),
        })],
        ..creature("Riddlesmith", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Surgehacker Mech — menace; entering deals twice your Vehicle count to an
/// opponent's creature or planeswalker. Crew 4.
pub fn surgehacker_mech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Crew(4)],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent)),
            amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::CountOf(Box::new(yours(vehicle()))))),
        })],
        ..vehicle_card("Surgehacker Mech", cost(&[generic(4)]), 5, 5, 4)
    }
}

/// Swift Reconfiguration — flash Aura; the enchanted creature or Vehicle is a
/// Vehicle artifact with crew 5 and no other card type.
pub fn swift_reconfiguration() -> CardDefinition {
    CardDefinition {
        name: "Swift Reconfiguration",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature.or(vehicle()) },
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Crew(5)],
            set_card_types: Some(vec![CardType::Artifact]),
            set_artifact_types: Some(vec![ArtifactSubtype::Vehicle]),
            set_creature_types: Some(vec![]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Weatherlight — flying Vehicle; hitting a player digs five for a historic
/// card. Crew 3.
pub fn weatherlight() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Crew(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                rest_bottom_random: true,
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(historic()),
                optional: true,
                ..Default::default()
            })),
        }],
        ..vehicle_card("Weatherlight", cost(&[generic(4)]), 4, 5, 3)
    }
}
