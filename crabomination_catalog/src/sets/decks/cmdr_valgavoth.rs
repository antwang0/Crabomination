//! Commander: the cards the **Endless Punishment** precon (DSC, Valgavoth,
//! Harrower of Souls) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_valgavoth.rs`.
//!
//! Residuals (each also on its card):
//! - **Barbflare Gremlin** and **Enchanter's Bane** — the damage comes from
//!   the Gremlin / the Bane, not the land / the enchantment.
//! - **Star Athlete** — "up to one target" always takes a target.
//! - **Torture Pit** — its +2 also reaches permanents opponents control
//!   (the shared `NoncombatDamageToOpponentsBonus`).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, RoomDoor, RoomDoors, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::{target_filtered, blitz};
use crate::effect::{Effect, ManaPayload, PlayerRef, PlayerStaticTarget, Predicate};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, Color, ManaCost, SpendRestriction};
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

/// "[Its controller] may sacrifice [it]. If they don't, [damage] to them."
fn sacrifice_or_take(what: Selector, damage: Value) -> Effect {
    let controller = || PlayerRef::ControllerOf(Box::new(what.clone()));
    Effect::PlayersMayAccept {
        who: controller(),
        description: "Sacrifice it rather than take the damage?".into(),
        on_accept: Box::new(Effect::SacrificePermanent { what: what.clone() }),
        if_any: Box::new(Effect::Noop),
        otherwise: Box::new(Effect::DealDamage { to: Selector::Player(controller()), amount: damage }),
    }
}

fn opponents_land(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
        effect,
    }
}

/// Valgavoth, Harrower of Souls — flying, ward—pay 2 life; an opponent
/// losing life for the first time on their turn grows it and draws you a card.
pub fn valgavoth_harrower_of_souls() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Life(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec {
                once_per_turn: true,
                ..EventSpec::new(EventKind::LifeLost, EventScope::OpponentControl)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::Triggerer))
            },
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature(
            "Valgavoth, Harrower of Souls",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Elder, CreatureType::Demon],
            4,
            4,
        )
    }
}

/// Barbflare Gremlin — first strike, haste; while it's tapped, a land tapped
/// for mana adds one more mana of a type it made, and its controller takes 1.
pub fn barbflare_gremlin() -> CardDefinition {
    let tapper = || PlayerRef::ControllerOf(Box::new(Selector::TriggerSource));
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                Predicate::EntityMatches { what: Selector::This, filter: R::Tapped },
            ])),
            effect: Effect::Seq(vec![
                Effect::AddMana { who: tapper(), pool: ManaPayload::AnyTypeTriggerSourceProduces },
                Effect::DealDamage { to: Selector::Player(tapper()), amount: Value::ONE },
            ]),
        }],
        ..creature("Barbflare Gremlin", cost(&[generic(3), r()]), vec![CreatureType::Gremlin], 3, 2)
    }
}

/// Enchanter's Bane — at your end step, target enchantment's controller
/// sacrifices it or takes its mana value in damage.
pub fn enchanters_bane() -> CardDefinition {
    CardDefinition {
        name: "Enchanter's Bane",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: sacrifice_or_take(
                target_filtered(R::Enchantment),
                Value::ManaValueOf(Box::new(Selector::Target(0))),
            ),
        }],
        ..Default::default()
    }
}

/// Geothermal Bog — a Swamp Mountain that enters tapped.
pub fn geothermal_bog() -> CardDefinition {
    let mut d = crate::sets::dual_land_with(
        "Geothermal Bog",
        LandType::Swamp,
        LandType::Mountain,
        Color::Black,
        Color::Red,
        vec![],
    );
    d.static_abilities.push(crate::sets::enters_tapped());
    d
}

/// Gleeful Arsonist — an opponent's noncreature spell burns them for its
/// power; undying.
pub fn gleeful_arsonist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Undying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
        }],
        ..creature("Gleeful Arsonist", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 2)
    }
}

/// Kederekt Parasite — an opponent drawing a card may take 1, if you control
/// a red permanent.
pub fn kederekt_parasite() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(
                Predicate::SelectorExists(Selector::EachPermanent(R::HasColor(Color::Red).and(R::ControlledByYou))),
            ),
            effect: Effect::MayDo {
                description: "Deal 1 damage to that player?".into(),
                body: Box::new(Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::ONE }),
            },
        }],
        ..creature("Kederekt Parasite", cost(&[b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Leechridden Swamp — a Swamp that enters tapped; {B}, {T}: each opponent
/// loses 1, with two or more black permanents.
pub fn leechridden_swamp() -> CardDefinition {
    CardDefinition {
        name: "Leechridden Swamp",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Swamp], ..Default::default() },
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Black),
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::HasColor(Color::Black).and(R::ControlledByYou)),
                    n: Value::Const(2),
                }),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nightshade Harvester — an opponent's land entering costs them 1 life and
/// grows it.
pub fn nightshade_harvester() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![opponents_land(Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ]))],
        ..creature("Nightshade Harvester", cost(&[generic(3), b()]), vec![CreatureType::Elf, CreatureType::Shaman], 2, 2)
    }
}

/// Persistent Constrictor — each opponent's upkeep costs them 1 life and a
/// -1/-1 counter on one of their creatures; persist.
pub fn persistent_constrictor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Persist],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByActivePlayer)),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..creature(
            "Persistent Constrictor",
            cost(&[generic(4), b()]),
            vec![CreatureType::Zombie, CreatureType::Snake],
            5,
            3,
        )
    }
}

/// Sadistic Shell Game — starting with the next opponent, each player picks
/// a creature you don't control; the picks die.
pub fn sadistic_shell_game() -> CardDefinition {
    CardDefinition {
        name: "Sadistic Shell Game",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerChoosesToDestroy { filter: R::Creature.and(R::ControlledByOpponent) },
        ..Default::default()
    }
}

/// Spiked Corridor // Torture Pit — unlocking the Corridor makes three
/// Devils that ping when they die; the Pit adds 2 to your noncombat damage
/// to opponents.
pub fn spiked_corridor_torture_pit() -> CardDefinition {
    let devil = TokenDefinition {
        name: "Devil".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::ONE },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Spiked Corridor // Torture Pit",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Room],
            ..Default::default()
        },
        room: Some(Box::new(RoomDoors {
            left: RoomDoor {
                name: "Spiked Corridor".into(),
                cost: cost(&[generic(3), r()]),
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DoorUnlocked, EventScope::SelfSource),
                    effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(3), definition: Arc::new(devil) },
                }],
                ..Default::default()
            },
            right: RoomDoor {
                name: "Torture Pit".into(),
                cost: cost(&[generic(3), r()]),
                static_abilities: vec![StaticAbility {
                    description: "If a source you control would deal noncombat damage to an opponent, it deals that \
                                  much damage plus 2 instead.",
                    effect: StaticEffect::NoncombatDamageToOpponentsBonus { amount: 2, while_revolt: false },
                }],
                ..Default::default()
            },
        })),
        ..Default::default()
    }
}

/// Star Athlete — menace; attacking, a nonland permanent's controller
/// sacrifices it or takes 5; blitz {3}{R}.
pub fn star_athlete() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        alternative_cost: Some(blitz(cost(&[generic(3), r()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: sacrifice_or_take(target_filtered(R::Nonland.and(R::Permanent)), Value::Const(5)),
        }],
        ..creature("Star Athlete", cost(&[generic(1), r(), r()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 2)
    }
}

/// Suspended Sentence — destroy an opponent's creature, they lose 3, and it
/// suspends itself again; suspend 3—{1}{B}.
pub fn suspended_sentence() -> CardDefinition {
    CardDefinition {
        name: "Suspended Sentence",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), b()]))],
        exile_on_resolve_time_counters: 3,
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(3),
            },
            Effect::Destroy { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        ]),
        ..Default::default()
    }
}

/// Séance Board — morbid: a soul counter each end step; {T}: that much mana
/// of one color for instants, sorceries, Demons and Spirits.
pub fn seance_board() -> CardDefinition {
    CardDefinition {
        name: "Séance Board",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Soul, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Soul,
                    })),
                    SpendRestriction::InstantSorceryOrTypes([CreatureType::Demon, CreatureType::Spirit]),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Lord of Pain — menace; your opponents can't gain life; each player's
/// first spell each turn deals its mana value to another target player.
pub fn the_lord_of_pain() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::EachOpponent },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::ONE },
            ),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.and(R::Not(Box::new(R::ControlledByTriggerPlayer)))),
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..creature(
            "The Lord of Pain",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            5,
            5,
        )
    }
}
