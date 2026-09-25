//! Commander: the cards the **Coven Counters** precon (MIC, Leinore, Autumn
//! Sovereign) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_leinore.rs`.
//!
//! Residuals (each also on its card):
//! - **Curse of Conformity** — a changeling keeps every creature type (the
//!   engine reads Changeling as a type wildcard, not a layer-4 CDA).
//! - **Celestial Judgment / Sigardian Zealot** — the per-power pick is the
//!   engine's (`Selector::OnePerDistinctPower`), not a prompt.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, w, x};
use crabomination_base::tokens::eldrazi_spawn_token;
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn human() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Human))
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn coven() -> Predicate {
    Predicate::CovenActive { who: PlayerRef::You }
}

/// "At the beginning of [step] on your turn, if you control three or more
/// creatures with different powers, [body]" — CR 603.4: checked as it
/// triggers and again as it resolves.
fn coven_step(step: TurnStep, body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl).with_filter(coven()),
        effect: Effect::If { cond: coven(), then: Box::new(body), else_: Box::new(Effect::Noop) },
    }
}

/// "Whenever another Human you control enters, put a +1/+1 counter on this."
fn another_human_enters_counter() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: human().and(R::OtherThanSource) },
        ),
        effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
    }
}

fn token(name: &str, color: Color, p: i32, t: i32, kinds: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: kinds, ..Default::default() },
        ..Default::default()
    }
}

fn make(definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(definition) }
}

fn player_curse(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        ..Default::default()
    }
}

/// Leinore, Autumn Sovereign — coven: at the beginning of combat on your
/// turn, a +1/+1 counter on up to one target creature you control, then draw
/// if you control three or more creatures with different powers.
pub fn leinore_autumn_sovereign() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::AddCounter {
                        what: target_filtered(R::Creature.and(R::ControlledByYou)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
                Effect::If {
                    cond: coven(),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legendary(creature(
            "Leinore, Autumn Sovereign",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            0,
            4,
        ))
    }
}

/// Angel of Glory's Rise — flying; enters: exile all Zombies, then return
/// every Human creature card from your graveyard to the battlefield.
pub fn angel_of_glorys_rise() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Zombie)),
                to: ZoneDest::Exile,
            },
            Effect::Move {
                what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: human() },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]))],
        ..creature("Angel of Glory's Rise", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 4, 6)
    }
}

/// Celebrate the Harvest — up to X basic lands onto the battlefield tapped, X
/// the number of different powers among creatures you control (read as it
/// resolves).
pub fn celebrate_the_harvest() -> CardDefinition {
    spell(
        "Celebrate the Harvest",
        cost(&[generic(3), g()]),
        CardType::Sorcery,
        Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::DistinctPowersAmongCreaturesControlled(PlayerRef::You),
        },
    )
}

/// Celestial Judgment — for each different power among creatures, one with
/// that power is chosen; every other creature is destroyed at once. Residual:
/// the engine picks (the caster's best, else an opponent's weakest).
pub fn celestial_judgment() -> CardDefinition {
    spell(
        "Celestial Judgment",
        cost(&[generic(4), w(), w()]),
        CardType::Sorcery,
        Effect::Destroy { what: Selector::OnePerDistinctPower { filter: R::Creature, rest: true } },
    )
}

/// Curse of Clinging Webs — enchant player; whenever a nontoken creature they
/// control dies, exile it (if it is still there) and you make a 1/2 Spider
/// with reach.
pub fn curse_of_clinging_webs() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken },
                Predicate::SamePlayer(
                    PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    PlayerRef::EnchantedPlayer,
                ),
            ])),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::MatchingAmong { inner: Box::new(Selector::TriggerSource), filter: R::InGraveyard },
                    to: ZoneDest::Exile,
                },
                make(token("Spider", Color::Green, 1, 2, vec![CreatureType::Spider], vec![Keyword::Reach])),
            ]),
        }],
        ..player_curse("Curse of Clinging Webs", cost(&[generic(2), g()]))
    }
}

/// Curse of Conformity — enchant player; their nonlegendary creatures have
/// base P/T 3/3 and lose all creature types. Residual: a changeling keeps its
/// types.
pub fn curse_of_conformity() -> CardDefinition {
    let theirs = || Selector::ControlledBy {
        who: PlayerRef::EnchantedPlayer,
        filter: R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Nonlegendary creatures enchanted player controls have base power and toughness 3/3.",
                effect: StaticEffect::SetBasePtForFilter { applies_to: theirs(), power: 3, toughness: 3 },
            },
            StaticAbility {
                description: "Nonlegendary creatures enchanted player controls lose all creature types.",
                effect: StaticEffect::MatchingLoseAllCreatureTypes { applies_to: theirs() },
            },
        ],
        ..player_curse("Curse of Conformity", cost(&[generic(4), w()]))
    }
}

/// Growth Spasm — a basic land onto the battlefield tapped, and a 0/1 Eldrazi
/// Spawn either way.
pub fn growth_spasm() -> CardDefinition {
    spell(
        "Growth Spasm",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            make(eldrazi_spawn_token()),
        ]),
    )
}

/// Heron's Grace Champion — flash, lifelink; enters: other Humans you control
/// get +1/+1 and gain lifelink until end of turn.
pub fn herons_grace_champion() -> CardDefinition {
    let others = || yours(human().and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT { what: others(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: others(), keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
        ]))],
        ..creature(
            "Heron's Grace Champion",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Heronblade Elite — vigilance; grows with each other Human you control
/// entering; {T}: X mana of any one color, X its power.
pub fn heronblade_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![another_human_enters_counter()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::PowerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..creature(
            "Heronblade Elite",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Kurbis, Harvest Celebrant — enters with a +1/+1 counter per mana spent to
/// cast it; remove one: prevent all damage to another target creature with a
/// +1/+1 counter this turn.
pub fn kurbis_harvest_celebrant() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CastSpellManaSpent)),
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: Effect::PreventAllDamageThisTurn {
                target: target_filtered(
                    R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne)).and(R::OtherThanSource),
                ),
                redirect_to: None,
            },
            ..Default::default()
        }],
        ..legendary(creature("Kurbis, Harvest Celebrant", cost(&[x(), g(), g()]), vec![CreatureType::Treefolk], 0, 0))
    }
}

/// Kyler, Sigardian Emissary — grows with each other Human you control
/// entering; other Humans you control get +1/+1 per counter (of any kind) on
/// Kyler.
pub fn kyler_sigardian_emissary() -> CardDefinition {
    let per = || Value::TotalCountersOn { what: Box::new(Selector::This) };
    CardDefinition {
        triggered_abilities: vec![another_human_enters_counter()],
        static_abilities: vec![StaticAbility {
            description: "Other Humans you control get +1/+1 for each counter on Kyler.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: yours(human().and(R::OtherThanSource)),
                power: per(),
                toughness: per(),
            },
        }],
        ..legendary(creature(
            "Kyler, Sigardian Emissary",
            cost(&[generic(3), g(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        ))
    }
}

/// Moorland Rescuer — dies: return any number of other creature cards with
/// total power X or less from your graveyard (X its last-known power), then
/// exile it.
pub fn moorland_rescuer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Move {
                what: Selector::TakeWithSumCap {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: R::Creature.and(R::OtherThanSource),
                    }),
                    cap: Box::new(Value::PowerOf(Box::new(Selector::This))),
                    value_of_each: Box::new(Value::PowerOf(Box::new(Selector::TriggerSource))),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::Move {
                what: Selector::MatchingAmong { inner: Box::new(Selector::This), filter: R::InGraveyard },
                to: ZoneDest::Exile,
            },
        ]))],
        ..creature(
            "Moorland Rescuer",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Ruinous Intrusion — exile target artifact or enchantment; X +1/+1 counters
/// on target creature you control, X the exiled permanent's mana value.
pub fn ruinous_intrusion() -> CardDefinition {
    spell(
        "Ruinous Intrusion",
        cost(&[generic(3), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: R::Artifact.or(R::Enchantment) },
                to: ZoneDest::Exile,
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
    )
}

/// Sigarda, Heron's Grace — flying; you and Humans you control have hexproof;
/// {2}, exile a card from your graveyard: a 1/1 Human Soldier.
pub fn sigarda_herons_grace() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility { description: "You have hexproof.", effect: StaticEffect::ControllerHasHexproof },
            StaticAbility {
                description: "Humans you control have hexproof.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(human()), keyword: Keyword::Hexproof },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            exile_other_filter: Some((R::Any, 1)),
            effect: make(token(
                "Human Soldier",
                Color::White,
                1,
                1,
                vec![CreatureType::Human, CreatureType::Soldier],
                vec![],
            )),
            ..Default::default()
        }],
        ..legendary(creature("Sigarda, Heron's Grace", cost(&[generic(3), g(), w()]), vec![CreatureType::Angel], 4, 5))
    }
}

/// Sigardian Zealot — at the beginning of combat on your turn, creatures with
/// different powers (one per power among yours — the engine's pick) get +X/+X
/// and vigilance until end of turn, X its power.
pub fn sigardian_zealot() -> CardDefinition {
    // Vigilance first: the pump moves powers, and the pick is re-read.
    let chosen = || Selector::OnePerDistinctPower { filter: R::Creature.and(R::ControlledByYou), rest: false };
    let x = || Value::PowerOf(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: chosen(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                Effect::PumpPT { what: chosen(), power: x(), toughness: x(), duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature(
            "Sigardian Zealot",
            cost(&[generic(4), g()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Somberwald Beastmaster — enters: a 2/2 Wolf, a 3/3 Beast and a 4/4 Beast;
/// creature tokens you control have deathtouch.
pub fn somberwald_beastmaster() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(token("Wolf", Color::Green, 2, 2, vec![CreatureType::Wolf], vec![])),
            make(token("Beast", Color::Green, 3, 3, vec![CreatureType::Beast], vec![])),
            make(token("Beast", Color::Green, 4, 4, vec![CreatureType::Beast], vec![])),
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::IsToken)),
                keyword: Keyword::Deathtouch,
            },
        }],
        ..creature(
            "Somberwald Beastmaster",
            cost(&[generic(6), g()]),
            vec![CreatureType::Human, CreatureType::Ranger],
            1,
            1,
        )
    }
}

/// Stalwart Pathlighter — vigilance; coven: at the beginning of combat on
/// your turn, creatures you control gain indestructible until end of turn.
pub fn stalwart_pathlighter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![coven_step(
            TurnStep::BeginCombat,
            Effect::GrantKeyword {
                what: yours(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Stalwart Pathlighter",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            1,
        )
    }
}

/// Wall of Mourning — defender; enters: exile the top card of your library
/// face down per opponent; coven, at your end step: one of them (at random)
/// to its owner's hand.
pub fn wall_of_mourning() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![
            etb(Effect::ExileTopOfLibrary {
                who: Selector::You,
                amount: Value::OpponentCount,
                link_to_source: true,
                face_down: true,
            }),
            coven_step(
                TurnStep::End,
                Effect::Move {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::CardExiledWithSource),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ),
        ],
        ..creature("Wall of Mourning", cost(&[generic(1), w()]), vec![CreatureType::Wall], 0, 4)
    }
}
