//! Commander: the cards the **Ruthless Regiment** precon (C20, Jirina Kudro)
//! needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_jirina.rs`.
//!
//! Residuals (each also on its card):
//! - **Sanctuary Blade** — the colour is chosen by a trigger as the Blade
//!   becomes attached, not as a replacement, so the protection starts once
//!   that trigger resolves.
//! - **Odric, Master Tactician** — the block choice holds for the rest of the
//!   turn, not only this combat (it matters only with an extra combat).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, partner_with_search, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, RevealMissDest, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};

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

fn human() -> R {
    R::HasCreatureType(CreatureType::Human)
}

fn human_soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Human Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn make(count: Value, definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: definition.into() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

/// Jirina Kudro — a Human Soldier per commander cast from the command zone
/// (the cast that put a commander Jirina here counts); other Humans +2/+0.
pub fn jirina_kudro() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(
            Value::CommanderCastsFromCommandZone(PlayerRef::You),
            human_soldier(),
        ))],
        static_abilities: vec![static_ab(
            "Other Humans you control get +2/+0.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(human().and(R::ControlledByYou).and(R::OtherThanSource)),
                power: 2,
                toughness: 0,
            },
        )],
        ..legend(
            "Jirina Kudro",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Bounty Agent — {T}, sacrifice: destroy a legendary artifact, creature or
/// enchantment.
pub fn bounty_agent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::HasSupertype(Supertype::Legendary).and(R::Artifact.or(R::Creature).or(R::Enchantment)),
                ),
            },
            ..Default::default()
        }],
        ..creature("Bounty Agent", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Captivating Crew — sorcery-speed threaten on an opponent's creature.
pub fn captivating_crew() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    to: None,
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Captivating Crew", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Pirate], 4, 3)
    }
}

/// Dearly Departed — from your graveyard, each Human creature you control
/// enters with an additional +1/+1 counter (one per copy there).
pub fn dearly_departed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![static_ab(
            "As long as this creature is in your graveyard, each Human creature you control enters with an additional +1/+1 counter on it.",
            StaticEffect::GraveyardMatchingEntersWithExtraCounters {
                filter: R::Creature.and(human()),
                kind: CounterType::PlusOnePlusOne,
            },
        )],
        ..creature("Dearly Departed", cost(&[generic(4), w(), w()]), vec![CreatureType::Spirit], 5, 5)
    }
}

/// Devout Chaplain — {T}, tap two untapped Humans: exile an artifact or
/// enchantment.
pub fn devout_chaplain() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            tap_others_cost: Some((human(), 2)),
            effect: Effect::Exile { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..creature("Devout Chaplain", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Fireflux Squad — on attack, may exile another attacker of yours to reveal
/// until a creature, which enters tapped and attacking (never declared, so
/// no attack triggers — CR 508.4); the rest go to the bottom at random.
pub fn fireflux_squad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Exile another attacking creature you control?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(
                        R::Creature.and(R::IsAttacking).and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                },
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Creature,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    cap: Value::Const(1000),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
                Effect::JoinCombatAttacking { what: Selector::LastMoved },
            ])),
        })],
        ..creature("Fireflux Squad", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Soldier], 4, 3)
    }
}

/// General's Enforcer — legendary Humans you control are indestructible;
/// graveyard hate that makes a Human Soldier off a creature card.
pub fn generals_enforcer() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Legendary Humans you control have indestructible.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    human().and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                ),
                keyword: Keyword::Indestructible,
            },
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), b()]),
            effect: Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Any.from_any_graveyard()), to: ZoneDest::Exile },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                    then: Box::new(make(Value::ONE, human_soldier())),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..creature("General's Enforcer", cost(&[w(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Kelsien, the Plague — grows with experience; pings a creature you don't
/// control and earns experience when it dies this turn.
pub fn kelsien_the_plague() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        dynamic_pt: Some(DynamicPt::ControllerExperience { base_p: 2, base_t: 2 }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            // The death-watch goes first so it is live if the ping kills.
            effect: Effect::Seq(vec![
                Effect::WhenTargetDiesThisTurn {
                    filter: None,
                    body: Box::new(Effect::AddExperience(Value::ONE)),
                    slot: 0,
                },
                Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Kelsien, the Plague",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            2,
            2,
        )
    }
}

/// Magus of the Disk — enters tapped; {1}, {T}: destroy every artifact,
/// creature and enchantment at once.
pub fn magus_of_the_disk() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: Selector::EachPermanent(R::Artifact.or(R::Creature).or(R::Enchantment)),
            },
            ..Default::default()
        }],
        ..creature("Magus of the Disk", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 4)
    }
}

/// Odric, Master Tactician — when it and at least three others attack, you
/// choose the blocks. Residual: the choice holds for the rest of the turn.
pub fn odric_master_tactician() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::AttackingWithAtLeast(4)),
            effect: Effect::ChooseBlocksThisTurn,
        }],
        ..legend(
            "Odric, Master Tactician",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Riders of Gavony — name a creature type; your Humans have protection from
/// creatures of it.
pub fn riders_of_gavony() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        static_abilities: vec![static_ab(
            "Human creatures you control have protection from creatures of the chosen type.",
            StaticEffect::GrantProtectionFromChosenCreatureType {
                applies_to: Selector::EachPermanent(R::Creature.and(human()).and(R::ControlledByYou)),
            },
        )],
        ..creature("Riders of Gavony", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Knight], 3, 3)
    }
}

/// Sanctuary Blade — +2/+0 and protection from the colour last chosen as it
/// became attached. Residual: the choice is a trigger, not a replacement.
pub fn sanctuary_blade() -> CardDefinition {
    CardDefinition {
        name: "Sanctuary Blade",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameAttached, EventScope::SelfSource),
            effect: Effect::ChooseColorForSelf,
        }],
        static_abilities: vec![static_ab(
            "Equipped creature has protection from the last chosen color.",
            StaticEffect::GrantProtectionFromChosenColor {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        )],
        ..Default::default()
    }
}

/// Silvar, Devourer of the Free — partner with Trynn; sacrifice a Human for a
/// +1/+1 counter and indestructible until end of turn.
pub fn silvar_devourer_of_the_free() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Trynn, Champion of Freedom".into()), Keyword::Menace],
        triggered_abilities: vec![partner_with_search("Trynn, Champion of Freedom")],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((human(), 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Silvar, Devourer of the Free",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Cat, CreatureType::Nightmare],
            4,
            2,
        )
    }
}

/// Trynn, Champion of Freedom — partner with Silvar; a Human Soldier at your
/// end step if you attacked this turn.
pub fn trynn_champion_of_freedom() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::PartnerWith("Silvar, Devourer of the Free".into())],
        triggered_abilities: vec![
            partner_with_search("Silvar, Devourer of the Free"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
                effect: make(Value::ONE, human_soldier()),
            },
        ],
        ..legend(
            "Trynn, Champion of Freedom",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Species Specialist — name a creature type; may draw whenever a creature of
/// it dies.
pub fn species_specialist() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::NameCreatureType { what: Selector::This }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::TriggerObjectIsChosenType),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..creature("Species Specialist", cost(&[generic(2), b(), b()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 3)
    }
}

/// Thraben Doomsayer — {T}: a 1/1 Human; fateful hour pumps your other
/// creatures +2/+2 at 5 or less life.
pub fn thraben_doomsayer() -> CardDefinition {
    let human_token = TokenDefinition {
        name: "Human".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: make(Value::ONE, human_token),
            ..Default::default()
        }],
        static_abilities: vec![static_ab(
            "Fateful hour — As long as you have 5 or less life, other creatures you control get +2/+2.",
            StaticEffect::WhileCondition {
                condition: Predicate::PlayerLifeAtMost { who: PlayerRef::You, life: 5 },
                inner: Box::new(StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                    power: 2,
                    toughness: 2,
                }),
            },
        )],
        ..creature("Thraben Doomsayer", cost(&[generic(1), w(), w()]), vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
    }
}

/// Verge Rangers — look at your library's top any time; play lands from it
/// while an opponent controls more lands than you.
pub fn verge_rangers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![
            static_ab("You may look at the top card of your library any time.", StaticEffect::MayLookAtOwnLibraryTop),
            static_ab(
                "As long as an opponent controls more lands than you, you may play lands from the top of your library.",
                StaticEffect::WhileCondition {
                    condition: Predicate::OpponentControlsMoreLandsThanYou,
                    inner: Box::new(StaticEffect::PlayFromLibraryTop { filter: R::Land }),
                },
            ),
        ],
        ..creature(
            "Verge Rangers",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Scout, CreatureType::Ranger],
            3,
            3,
        )
    }
}

/// Vigilante Justice — 1 damage to any target whenever a Human you control
/// enters.
pub fn vigilante_justice() -> CardDefinition {
    CardDefinition {
        name: "Vigilante Justice",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(human()) }),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Xathrid Necromancer — a tapped 2/2 Zombie whenever it or another Human
/// creature you control dies (it is a Human itself).
pub fn xathrid_necromancer() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: human() }),
            effect: make(Value::ONE, zombie),
        }],
        ..creature("Xathrid Necromancer", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 2)
    }
}
