//! Commander: the cards the **Power Hungry** precon (C13, Prossh, Skyraider
//! of Kher) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_prossh.rs`.
//!
//! Residuals (each also on its card):
//! - **Sudden Demise** — the color is the engine's pick.
//! - **Night Soil** — the two creature cards come from your own graveyard.
//! - **Widespread Panic** — any shuffle a spell or ability makes counts, not
//!   only one its controller made of their own library.
//! - **Capricious Efreet** — the up-to-two opponents' targets are the
//!   auto-picker's.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Selector, StateTriggeredAbility, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{evoke, on_cast, on_dies, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, x};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn token(name: &str, colors: Vec<Color>, p: i32, t: i32, kinds: Vec<CreatureType>, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: kinds, ..Default::default() },
        ..Default::default()
    })
}

fn make(count: Value, definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition }
}

fn saproling() -> Arc<TokenDefinition> {
    token("Saproling", vec![Color::Green], 1, 1, vec![CreatureType::Saproling], vec![])
}

fn jund() -> ManaCost {
    cost(&[b(), r(), g()])
}

/// Prossh, Skyraider of Kher — casting it makes a 0/1 Kobolds of Kher Keep
/// per mana spent; flying; sacrifice another creature: +1/+0.
pub fn prossh_skyraider_of_kher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_cast(make(
            Value::CastSpellManaSpent,
            token("Kobolds of Kher Keep", vec![Color::Red], 0, 1, vec![CreatureType::Kobold], vec![]),
        ))],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Prossh, Skyraider of Kher",
            cost(&[generic(3), b(), r(), g()]),
            vec![CreatureType::Dragon],
            5,
            5,
        ))
    }
}

/// Brooding Saurian — at the beginning of each end step, each player gains
/// control of all nontoken permanents they own.
pub fn brooding_saurian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::OwnersGainControlOfNontokens,
        }],
        ..creature("Brooding Saurian", cost(&[generic(2), g(), g()]), vec![CreatureType::Lizard], 4, 4)
    }
}

/// Capricious Efreet — your upkeep: target nonland permanent you control and
/// up to two you don't; destroy one of them at random.
pub fn capricious_efreet() -> CardDefinition {
    let yours = R::Nonland.and(R::Permanent).and(R::ControlledByYou);
    let theirs = || R::Nonland.and(R::Permanent).and(R::Not(Box::new(R::ControlledByYou)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::OptionalTargets {
                min: 1,
                body: Box::new(Effect::Destroy {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::Both(
                            Box::new(Selector::TargetFiltered { slot: 0, filter: yours }),
                            Box::new(Selector::Both(
                                Box::new(Selector::TargetFiltered { slot: 1, filter: theirs() }),
                                Box::new(Selector::TargetFiltered { slot: 2, filter: theirs() }),
                            )),
                        )),
                        count: Box::new(Value::ONE),
                    },
                }),
            },
        }],
        ..creature("Capricious Efreet", cost(&[generic(4), r(), r()]), vec![CreatureType::Efreet], 6, 4)
    }
}

/// Deathbringer Thoctar — another creature dying may add a +1/+1 counter;
/// remove one: 1 damage to any target.
pub fn deathbringer_thoctar() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Deathbringer Thoctar?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Deathbringer Thoctar",
            cost(&[generic(4), b(), r()]),
            vec![CreatureType::Zombie, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Deepfire Elemental — {X}{X}{1}: destroy target artifact or creature with
/// mana value X.
pub fn deepfire_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x(), generic(1)]),
            effect: Effect::Destroy {
                what: target_filtered((R::Artifact.or(R::Creature)).and(R::ManaValueExactlyXFromCost)),
            },
            ..Default::default()
        }],
        ..creature("Deepfire Elemental", cost(&[generic(4), b(), r()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Endrek Sahr, Master Breeder — each creature spell you cast makes a 1/1
/// Thrull per point of its mana value; seven or more Thrulls sacrifice it.
pub fn endrek_sahr_master_breeder() -> CardDefinition {
    let thrulls = || R::Creature.and(R::HasCreatureType(CreatureType::Thrull)).and(R::ControlledByYou);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: make(
                Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                token("Thrull", vec![Color::Black], 1, 1, vec![CreatureType::Thrull], vec![]),
            ),
        }],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(thrulls()), n: Value::Const(7) },
            effect: Effect::SacrificeSource,
        }),
        ..legendary(creature(
            "Endrek Sahr, Master Breeder",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        ))
    }
}

/// Fell Shepherd — combat damage to a player may return every creature card
/// put into your graveyard from the battlefield this turn; {B}, sacrifice
/// another creature: target creature gets -2/-2 until end of turn.
pub fn fell_shepherd() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return the creature cards that died this turn?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Fell Shepherd", cost(&[generic(5), b(), b()]), vec![CreatureType::Avatar], 8, 6)
    }
}

/// Furnace Celebration — whenever you sacrifice another permanent, you may
/// pay {2} for 2 damage to any target.
pub fn furnace_celebration() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::MayPay {
                description: "Pay {2} for 2 damage to any target?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(2) }),
                else_: None,
            },
        }],
        ..enchantment("Furnace Celebration", cost(&[generic(1), r(), r()]))
    }
}

/// Hua Tuo, Honored Physician — {T}: target creature card from your graveyard
/// on top of your library, only during your turn before attackers.
pub fn hua_tuo_honored_physician() -> CardDefinition {
    use crate::effect::Predicate as P;
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(P::All(vec![
                P::IsTurnOf(PlayerRef::You),
                P::Any(vec![
                    P::CurrentStepIs(TurnStep::Upkeep),
                    P::CurrentStepIs(TurnStep::Draw),
                    P::CurrentStepIs(TurnStep::PreCombatMain),
                    P::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
            ..Default::default()
        }],
        ..legendary(creature("Hua Tuo, Honored Physician", cost(&[generic(1), g(), g()]), vec![CreatureType::Human], 1, 2))
    }
}

/// Jar of Eyeballs — two eyeball counters per creature of yours dying; {3},
/// {T}, remove them all: look at that many, take one, the rest to the bottom.
pub fn jar_of_eyeballs() -> CardDefinition {
    CardDefinition {
        name: "Jar of Eyeballs",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Eyeball, amount: Value::Const(2) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            remove_all_counters_cost: Some(CounterType::Eyeball),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                count: Value::CountersRemovedAsCost,
                ..Default::default()
            })),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Night Soil — {1}, exile two creature cards from a graveyard: a Saproling.
/// Residual: your own graveyard.
pub fn night_soil() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            exile_other_filter: Some((R::Creature, 2)),
            effect: make(Value::ONE, saproling()),
            ..Default::default()
        }],
        ..enchantment("Night Soil", cost(&[g(), g()]))
    }
}

/// Obelisk of Jund — {T}: {B}, {R} or {G}.
pub fn obelisk_of_jund() -> CardDefinition {
    CardDefinition {
        name: "Obelisk of Jund",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Black, Color::Red, Color::Green], Value::ONE),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Primal Vigor — every player's tokens are doubled, and so are +1/+1
/// counters put on any creature.
pub fn primal_vigor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "If one or more tokens would be created, twice that many are created instead.",
                effect: StaticEffect::DoubleTokensEveryone,
            },
            StaticAbility {
                description: "If one or more +1/+1 counters would be put on a creature, twice that many are.",
                effect: StaticEffect::DoublePlusOneCountersEveryone,
            },
        ],
        ..enchantment("Primal Vigor", cost(&[generic(4), g()]))
    }
}

/// Scarland Thrinax — sacrifice a creature: a +1/+1 counter on it.
pub fn scarland_thrinax() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Scarland Thrinax", jund(), vec![CreatureType::Lizard], 2, 2)
    }
}

/// Sek'Kuar, Deathkeeper — another nontoken creature of yours dying makes a
/// 3/1 black and red Graveborn with haste.
pub fn sekkuar_deathkeeper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken.and(R::OtherThanSource) },
            ),
            effect: make(
                Value::ONE,
                token(
                    "Graveborn",
                    vec![Color::Black, Color::Red],
                    3,
                    1,
                    vec![CreatureType::Graveborn],
                    vec![Keyword::Haste],
                ),
            ),
        }],
        ..legendary(creature(
            "Sek'Kuar, Deathkeeper",
            cost(&[generic(2), b(), r(), g()]),
            vec![CreatureType::Orc, CreatureType::Shaman],
            4,
            3,
        ))
    }
}

/// Shattergang Brothers — {2}{B}/{2}{R}/{2}{G}, sacrifice a creature /
/// artifact / enchantment: each other player sacrifices one of that type.
pub fn shattergang_brothers() -> CardDefinition {
    let edict = |mana: ManaCost, kind: R| ActivatedAbility {
        mana_cost: mana,
        sac_other_filter: Some((kind.clone(), 1)),
        sac_other_may_be_source: true,
        effect: Effect::Sacrifice { who: Selector::Player(PlayerRef::EachOpponent), count: Value::ONE, filter: kind },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            edict(cost(&[generic(2), b()]), R::Creature),
            edict(cost(&[generic(2), r()]), R::Artifact),
            edict(cost(&[generic(2), g()]), R::Enchantment),
        ],
        ..legendary(creature(
            "Shattergang Brothers",
            cost(&[generic(1), b(), r(), g()]),
            vec![CreatureType::Goblin, CreatureType::Artificer],
            3,
            3,
        ))
    }
}

/// Spoils of Victory — a Plains, Island, Swamp, Mountain or Forest card onto
/// the battlefield (untapped).
pub fn spoils_of_victory() -> CardDefinition {
    let basic_typed = [LandType::Plains, LandType::Island, LandType::Swamp, LandType::Mountain, LandType::Forest]
        .into_iter()
        .map(R::HasLandType)
        .reduce(R::or)
        .expect("five types");
    CardDefinition {
        name: "Spoils of Victory",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::Land.and(basic_typed),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Sprouting Thrinax — dies: three 1/1 Saprolings.
pub fn sprouting_thrinax() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(make(Value::Const(3), saproling()))],
        ..creature("Sprouting Thrinax", jund(), vec![CreatureType::Lizard], 3, 3)
    }
}

/// Sudden Demise — choose a color; X damage to each creature of that color.
/// Residual: the engine picks the color.
pub fn sudden_demise() -> CardDefinition {
    CardDefinition {
        name: "Sudden Demise",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DamageEachCreatureOfChosenColor { amount: Value::XFromCost },
        ..Default::default()
    }
}

/// Walker of the Grove — leaving the battlefield makes a 4/4 Elemental;
/// evoke {4}{G}.
pub fn walker_of_the_grove() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(evoke(cost(&[generic(4), g()]))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: make(
                Value::ONE,
                token("Elemental", vec![Color::Green], 4, 4, vec![CreatureType::Elemental], vec![]),
            ),
        }],
        ..creature("Walker of the Grove", cost(&[generic(6), g(), g()]), vec![CreatureType::Elemental], 7, 7)
    }
}

/// Widespread Panic — whenever a spell or ability makes a player shuffle,
/// they put a card from their hand on top of their library. Residual: any
/// such shuffle counts.
pub fn widespread_panic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LibraryShuffled, EventScope::AnyPlayer),
            effect: Effect::PutCardFromHandOnTopOfLibrary {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
            },
        }],
        ..enchantment("Widespread Panic", cost(&[generic(2), r()]))
    }
}
