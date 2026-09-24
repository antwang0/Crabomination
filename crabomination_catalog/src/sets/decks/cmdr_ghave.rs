//! Commander: the cards the **Counterpunch** precon (CMD, Ghave, Guru of
//! Spores) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_ghave.rs`.
//!
//! Residuals (each also on its card):
//! - **Footbottom Feast** — the cards are chosen as it resolves, not targeted
//!   on cast, and go on top in mana-value order (the greatest on top).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered, token_copy_of};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, w};
use crate::sets::enters_tapped;
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

fn token(name: &str, color: Color, t: CreatureType) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![t], ..Default::default() },
        ..Default::default()
    })
}

fn saproling() -> Arc<TokenDefinition> {
    token("Saproling", Color::Green, CreatureType::Saproling)
}

/// Acorn Catapult — {1}, {T}: 1 damage to any target; that permanent's
/// controller or that player creates a 1/1 Squirrel.
pub fn acorn_catapult() -> CardDefinition {
    CardDefinition {
        name: "Acorn Catapult",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                    amount: Value::Const(1),
                },
                Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    count: Value::Const(1),
                    definition: token("Squirrel", Color::Green, CreatureType::Squirrel),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Alliance of Arms — join forces: each player creates X 1/1 Soldiers, X the
/// total mana paid.
pub fn alliance_of_arms() -> CardDefinition {
    CardDefinition {
        name: "Alliance of Arms",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::JoinForces {
            description: "Join forces — pay any amount of mana; each player creates that many Soldiers"
                .into(),
            body: Box::new(Effect::CreateToken {
                who: PlayerRef::EachPlayer,
                count: Value::TriggerEventAmount,
                definition: token("Soldier", Color::White, CreatureType::Soldier),
            }),
        },
        ..Default::default()
    }
}

/// Awakening Zone — at the beginning of your upkeep, you may create a 0/1
/// Eldrazi Spawn.
pub fn awakening_zone() -> CardDefinition {
    CardDefinition {
        name: "Awakening Zone",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Create a 0/1 Eldrazi Spawn?".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: Arc::new(crabomination_base::tokens::eldrazi_spawn_token()),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Celestial Force — at the beginning of each upkeep, you gain 3 life.
pub fn celestial_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
        }],
        ..creature(
            "Celestial Force",
            cost(&[generic(5), w(), w(), w()]),
            vec![CreatureType::Elemental],
            7,
            7,
        )
    }
}

/// Footbottom Feast — put any number of creature cards from your graveyard
/// on top of your library, then draw a card.
pub fn footbottom_feast() -> CardDefinition {
    CardDefinition {
        name: "Footbottom Feast",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PutAnyNumberFromGraveyardOnTop { filter: R::Creature },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Ghave, Guru of Spores — enters with five +1/+1 counters; {1}, remove a
/// +1/+1 counter from a creature you control: a Saproling; {1}, sacrifice a
/// creature: a +1/+1 counter on target creature.
pub fn ghave_guru_of_spores() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_among_filter: Some((Some(CounterType::PlusOnePlusOne), 1, R::Creature)),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: saproling() },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_may_be_source: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Ghave, Guru of Spores",
            cost(&[generic(2), w(), b(), g()]),
            vec![CreatureType::Fungus, CreatureType::Shaman],
            0,
            0,
        )
    }
}

/// Karador, Ghost Chieftain — costs {1} less per creature card in your
/// graveyard; once during each of your turns, cast a creature spell from
/// your graveyard.
pub fn karador_ghost_chieftain() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "This spell costs {1} less to cast for each creature card in your graveyard.",
                effect: StaticEffect::SelfCostReducedPerCreatureInGraveyard,
            },
            StaticAbility {
                description: "Once during each of your turns, you may cast a creature spell from your graveyard.",
                effect: StaticEffect::GraveyardCastOncePerTurn { filter: R::Creature, exile_after: false },
            },
        ],
        ..creature(
            "Karador, Ghost Chieftain",
            cost(&[generic(5), w(), b(), g()]),
            vec![CreatureType::Centaur, CreatureType::Spirit],
            3,
            4,
        )
    }
}

/// Necrogenesis — {2}: exile target creature card from a graveyard; create
/// a 1/1 Saproling.
pub fn necrogenesis() -> CardDefinition {
    CardDefinition {
        name: "Necrogenesis",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::Move { what: target_filtered(R::Creature.from_any_graveyard()), to: ZoneDest::Exile },
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: saproling() },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rupture Spire — enters tapped; sacrifice it unless you pay {1}; {T}: one
/// mana of any color.
pub fn rupture_spire() -> CardDefinition {
    CardDefinition {
        name: "Rupture Spire",
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(1)]) },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::Const(1)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// A 1/1: the intervening "if that creature is 1/1" of Sigil Captain.
fn one_one() -> R {
    R::PowerAtLeast(1).and(R::PowerAtMost(1)).and(R::ToughnessAtLeast(1)).and(R::ToughnessAtMost(1))
}

/// Sigil Captain — whenever a creature you control enters, if it is 1/1,
/// two +1/+1 counters on it.
pub fn sigil_captain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(one_one()) },
            ),
            // CR 603.4 — checked again as it resolves.
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: Selector::TriggerSource, filter: one_one() },
                then: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature(
            "Sigil Captain",
            cost(&[generic(1), g(), w(), w()]),
            vec![CreatureType::Rhino, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Spawnwrithe — trample; connecting, create a token copy of it.
pub fn spawnwrithe() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: token_copy_of(PlayerRef::You, Value::Const(1), Selector::This),
        }],
        ..creature("Spawnwrithe", cost(&[generic(2), g()]), vec![CreatureType::Elemental], 2, 2)
    }
}

/// Teneb, the Harvester — flying; connecting, you may pay {2}{B} to put
/// target creature card from a graveyard onto the battlefield under your
/// control.
pub fn teneb_the_harvester() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2}{B} to reanimate a creature card from a graveyard?".into(),
                mana_cost: cost(&[generic(2), b()]),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.from_any_graveyard()),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: None,
            },
        }],
        ..creature(
            "Teneb, the Harvester",
            cost(&[generic(3), w(), b(), g()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Vish Kal, Blood Arbiter — flying, lifelink; sacrifice a creature: X +1/+1
/// counters, X its power; remove all +1/+1 counters: target creature gets
/// -1/-1 per counter removed.
pub fn vish_kal_blood_arbiter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_may_be_source: true,
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::SacrificedPower,
                },
                ..Default::default()
            },
            ActivatedAbility {
                remove_all_counters_cost: Some(CounterType::PlusOnePlusOne),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Negate(Box::new(Value::CountersRemovedAsCost)),
                    toughness: Value::Negate(Box::new(Value::CountersRemovedAsCost)),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Vish Kal, Blood Arbiter",
            cost(&[generic(4), w(), b(), b()]),
            vec![CreatureType::Vampire],
            5,
            5,
        )
    }
}

/// Vow of Wildness — +3/+3, trample, can't attack you or your planeswalkers.
pub fn vow_of_wildness() -> CardDefinition {
    CardDefinition {
        name: "Vow of Wildness",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Trample, Keyword::CantAttackAuraController],
            ..Default::default()
        }),
        ..Default::default()
    }
}
