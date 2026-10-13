//! Commander: three-card gaps for Belbe, Zurzoth, Vannifar, River Song and
//! The Cabbage Merchant (`cmdr_edhrec30` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec31.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, SpellSubtype, StaticAbility, StaticEffect,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{dash, deal, etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn on_attacks(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect }
}

fn each_opponent_loses(n: i32) -> Effect {
    Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(n) }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn food() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Food)
}

fn make_food(count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(crabomination_base::tokens::food_token()) }
}

fn blink(what: Selector) -> Effect {
    Effect::ExileAndReturnToOwner { what }
}

/// "Destroy target land. Its controller may search their library for a basic
/// land card, put it onto the battlefield (tapped), then shuffle."
fn raze_and_replace(tapped: bool) -> Effect {
    Effect::Seq(vec![
        Effect::Destroy { what: target_filtered(R::Land) },
        Effect::Search {
            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped },
        },
    ])
}

// ── Belbe, Corrupted Observer ────────────────────────────────────────────────

/// Mardu Shadowspear — {B} 1/1 Human Warrior. Attacking drains each opponent
/// for 1. Dash {1}{B}.
pub fn mardu_shadowspear() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(dash(cost(&[generic(1), b()]))),
        triggered_abilities: vec![on_attacks(each_opponent_loses(1))],
        ..creature("Mardu Shadowspear", cost(&[b()]), vec![CreatureType::Human, CreatureType::Warrior], 1, 1)
    }
}

/// Night Market Lookout — {B} 1/1 Human Rogue. Becoming tapped, each opponent
/// loses 1 and you gain 1.
pub fn night_market_lookout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                each_opponent_loses(1),
                Effect::GainLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature("Night Market Lookout", cost(&[b()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Thornbow Archer — {B} 1/2 Elf Archer. Attacking, each opponent who doesn't
/// control an Elf loses 1.
pub fn thornbow_archer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attacks(Effect::EachPlayerDoes {
            who: PlayerRef::EachOpponent,
            body: Box::new(Effect::If {
                cond: Predicate::SelectorExists(yours(R::HasCreatureType(CreatureType::Elf))),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::LoseLife { who: Selector::You, amount: Value::ONE }),
            }),
        })],
        ..creature("Thornbow Archer", cost(&[b()]), vec![CreatureType::Elf, CreatureType::Archer], 1, 2)
    }
}

// ── Zurzoth, Chaos Rider ─────────────────────────────────────────────────────

/// Gibbering Fiend — {1}{R} 2/1 Devil. ETB: 1 damage to each opponent.
/// Delirium — each opponent's upkeep, 1 damage to that player.
pub fn gibbering_fiend() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(deal(1, Selector::Player(PlayerRef::EachOpponent))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl)
                    .with_filter(Predicate::DeliriumActive { who: PlayerRef::You }),
                effect: deal(1, Selector::Player(PlayerRef::ActivePlayer)),
            },
        ],
        ..creature("Gibbering Fiend", cost(&[generic(1), r()]), vec![CreatureType::Devil], 2, 1)
    }
}

/// Hobblefiend — {1}{R} 2/1 Devil, trample. {1}, sacrifice another creature:
/// a +1/+1 counter.
pub fn hobblefiend() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature.and(R::ControlledByYou), 1)),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Hobblefiend", cost(&[generic(1), r()]), vec![CreatureType::Devil], 2, 1)
    }
}

/// Spiteful Prankster — {2}{R} 3/2 Devil, first strike on your turn. Another
/// creature dying pings a player or planeswalker for 1.
pub fn spiteful_prankster() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, this creature has first strike.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::FirstStrike,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource }),
            effect: Effect::DealDamage { to: target_filtered(R::Player.or(R::Planeswalker)), amount: Value::ONE },
        }],
        ..creature("Spiteful Prankster", cost(&[generic(2), r()]), vec![CreatureType::Devil], 3, 2)
    }
}

// ── Vannifar, Evolved Enigma ─────────────────────────────────────────────────

/// Meneldor, Swift Savior — {3}{U} 3/3 legendary Bird Soldier, flying.
/// Connecting blinks up to one creature you own, back under your control.
pub fn meneldor_swift_savior() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Creature.and(R::OwnedByYou),
                effect: Box::new(blink(Selector::Target(0))),
            },
        }],
        ..creature("Meneldor, Swift Savior", cost(&[generic(3), u()]), vec![CreatureType::Bird, CreatureType::Soldier], 3, 3)
    }
}

/// Illusionist's Stratagem — {3}{U} instant. Blink up to two creatures you
/// control; draw a card.
pub fn illusionists_stratagem() -> CardDefinition {
    spell(
        "Illusionist's Stratagem",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(blink(Selector::Target(0))),
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Siren's Ruse — {1}{U} instant. Blink a creature you control; draw if it
/// was a Pirate.
pub fn sirens_ruse() -> CardDefinition {
    let target = || target_filtered(R::Creature.and(R::ControlledByYou));
    spell(
        "Siren's Ruse",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::EntityMatches { what: target(), filter: R::HasCreatureType(CreatureType::Pirate) },
            then: Box::new(Effect::Seq(vec![blink(target()), Effect::Draw { who: Selector::You, amount: Value::ONE }])),
            else_: Box::new(blink(target())),
        },
    )
}

// ── River Song ───────────────────────────────────────────────────────────────

/// Eager Construct — {2} 2/2 artifact Construct. ETB: each player may scry 1.
pub fn eager_construct() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::EachPlayerDoes {
            who: PlayerRef::EachPlayer,
            body: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::ONE }),
        })],
        ..creature("Eager Construct", cost(&[generic(2)]), vec![CreatureType::Construct], 2, 2)
    }
}

/// Avengers Disassembled — {1}{R}{R} sorcery. Choose one or both: 3 damage to
/// each creature; destroy a land, whose controller may fetch a basic tapped.
pub fn avengers_disassembled() -> CardDefinition {
    spell(
        "Avengers Disassembled",
        cost(&[generic(1), r(), r()]),
        CardType::Sorcery,
        Effect::ChooseModesCast {
            modes: vec![deal(3, Selector::EachPermanent(R::Creature)), raze_and_replace(true)],
            min: 1,
            max: 2,
            allow_repeats: false,
        },
    )
}

/// Geomancer's Gambit — {2}{R} sorcery. Destroy a land; its controller may
/// fetch a basic untapped. Draw a card.
pub fn geomancers_gambit() -> CardDefinition {
    spell(
        "Geomancer's Gambit",
        cost(&[generic(2), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![raze_and_replace(false), Effect::Draw { who: Selector::You, amount: Value::ONE }]),
    )
}

// ── The Cabbage Merchant ─────────────────────────────────────────────────────

/// Bosco, Just a Bear — {4}{G} 4/4 legendary Bear. ETB: a Food per legendary
/// creature you control. {2}{G}, sacrifice a Food: two +1/+1 counters and
/// trample until end of turn.
pub fn bosco_just_a_bear() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(make_food(Value::count(yours(
            R::Creature.and(R::HasSupertype(Supertype::Legendary)),
        ))))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sac_other_filter: Some((food().and(R::ControlledByYou), 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Bosco, Just a Bear", cost(&[generic(4), g()]), vec![CreatureType::Bear], 4, 4)
    }
}

/// Teething Wurmlet — {G} 1/1 Wurm, deathtouch with three or more artifacts.
/// An artifact of yours entering gains you 1 life, and the first resolution
/// each turn also grows it.
pub fn teething_wurmlet() -> CardDefinition {
    let gain = || Effect::GainLife { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has deathtouch as long as you control three or more artifacts.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Deathtouch,
                condition: Predicate::ValueAtLeast(Value::count(yours(R::Artifact)), Value::Const(3)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::EscalatingThisTurn {
                modes: vec![
                    Effect::Seq(vec![
                        gain(),
                        Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                    ]),
                    gain(),
                ],
            },
        }],
        ..creature("Teething Wurmlet", cost(&[g()]), vec![CreatureType::Wurm], 1, 1)
    }
}

/// Bumi's Feast Lecture — {1}{G} sorcery — Lesson. A Food, then earthbend
/// twice the Foods you control.
pub fn bumis_feast_lecture() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        ..spell(
            "Bumi's Feast Lecture",
            cost(&[generic(1), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                make_food(Value::ONE),
                Effect::Earthbend { n: Value::Times(Box::new(Value::Const(2)), Box::new(Value::count(yours(food())))) },
            ]),
        )
    }
}
