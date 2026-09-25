//! Commander: the cards the **Bedecked Brokers** precon (NCC, Perrie, the
//! Pulverizer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_perrie.rs`.
//!
//! Residuals (each also on its card):
//! - **Kros, Defense Contractor** — only the counters its own upkeep
//!   trigger puts goad; counters other sources put on an opposing creature
//!   don't.
//! - **Aven Mimeomancer** — the 3/1 flier lasts while Mimeomancer is on the
//!   battlefield (a static over every creature with a feather counter), not
//!   for as long as the counter stays.
//! - **Agent's Toolkit** — its counters arrive by an entry trigger, and the
//!   counter it moves is the engine's pick (+1/+1 first).
//! - **Littjara Mirrorlake** — the extra +1/+1 counter is put on the copy
//!   after it enters.
//! - **Skyship Plunderer** — a player target gets one more energy,
//!   experience or poison counter for each they have; other player
//!   counters aren't read.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{dethrone, etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w};
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

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

fn gwu() -> ManaCost {
    cost(&[generic(1), g(), w(), u()])
}

fn on_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl), effect }
}

fn counter(what: Selector, kind: CounterType) -> Effect {
    Effect::AddCounter { what, kind, amount: Value::ONE }
}

fn keyword_counter(what: Selector, keyword: Keyword) -> Effect {
    Effect::AddKeywordCounter { what, keyword, amount: Value::ONE }
}

/// "The number of different kinds of counters among permanents you control."
fn kinds_you_control() -> Value {
    Value::CounterKindsAmong { who: PlayerRef::You, filter: R::Any }
}

/// Perrie, the Pulverizer — a shield counter on entry; each attack pumps a
/// creature of yours by the kinds of counters you have, with trample.
pub fn perrie_the_pulverizer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(counter(target_filtered(R::Creature), CounterType::Shield)),
            on_attack(Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: kinds_you_control(),
                    toughness: kinds_you_control(),
                    duration: Duration::EndOfTurn,
                },
            ])),
        ],
        ..legendary(creature(
            "Perrie, the Pulverizer",
            gwu(),
            vec![CreatureType::Rhino, CreatureType::Soldier],
            3,
            3,
        ))
    }
}

/// Kros, Defense Contractor — each upkeep a shield counter on an opposing
/// creature, which is tapped, goaded and given trample until your next turn.
/// Residual: only its own counters goad.
pub fn kros_defense_contractor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_upkeep(Effect::Seq(vec![
            counter(target_filtered(R::Creature.and(R::ControlledByOpponent)), CounterType::Shield),
            Effect::Tap { what: Selector::Target(0) },
            Effect::Goad { what: Selector::Target(0) },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::UntilNextTurn },
        ]))],
        ..legendary(creature(
            "Kros, Defense Contractor",
            gwu(),
            vec![CreatureType::Cat, CreatureType::Advisor],
            2,
            4,
        ))
    }
}

/// Agent's Toolkit — a Clue carrying +1/+1, flying, deathtouch and shield
/// counters that it hands to your creatures as they enter.
/// Residual: the counters arrive by an entry trigger; the moved counter is
/// the engine's pick.
pub fn agents_toolkit() -> CardDefinition {
    CardDefinition {
        name: "Agent's Toolkit",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Clue], ..Default::default() },
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                counter(Selector::This, CounterType::PlusOnePlusOne),
                keyword_counter(Selector::This, Keyword::Flying),
                keyword_counter(Selector::This, Keyword::Deathtouch),
                counter(Selector::This, CounterType::Shield),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
                effect: Effect::MayDo {
                    description: "Move a counter from Agent's Toolkit onto that creature?".into(),
                    body: Box::new(Effect::MoveOneCounter { from: Selector::This, to: Selector::TriggerSource }),
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aven Courier — attacking, it copies a kind of counter you have onto a
/// permanent of yours that lacks it.
pub fn aven_courier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::AddMissingCounterKindFromYours {
            what: target_filtered(R::Permanent.and(R::ControlledByYou)),
        })],
        ..creature("Aven Courier", cost(&[generic(1), u()]), vec![CreatureType::Bird, CreatureType::Advisor], 1, 1)
    }
}

/// Aven Mimeomancer — each upkeep it may make a creature a 3/1 flier with a
/// feather counter. Residual: the effect lasts while Mimeomancer does.
pub fn aven_mimeomancer() -> CardDefinition {
    let feathered = || Selector::EachPermanent(R::Creature.and(R::WithCounter(CounterType::Feather)));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_upkeep(Effect::MayDo {
            description: "Put a feather counter on target creature?".into(),
            body: Box::new(counter(target_filtered(R::Creature), CounterType::Feather)),
        })],
        static_abilities: vec![
            StaticAbility {
                description: "A creature with a feather counter has base power and toughness 3/1.",
                effect: StaticEffect::SetBasePtForFilter { applies_to: feathered(), power: 3, toughness: 1 },
            },
            StaticAbility {
                description: "A creature with a feather counter has flying.",
                effect: StaticEffect::GrantKeyword { applies_to: feathered(), keyword: Keyword::Flying },
            },
        ],
        ..creature(
            "Aven Mimeomancer",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            3,
            1,
        )
    }
}

/// Bribe Taker — enters with a counter per kind you have (a +1/+1 counter in
/// place of any kind but shield and keyword counters).
pub fn bribe_taker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::AddCounterOfEachKindAmong {
            onto: Selector::This,
            who: PlayerRef::You,
            filter: R::Any,
            or_plus_one: true,
        })],
        ..creature("Bribe Taker", cost(&[generic(5), g()]), vec![CreatureType::Rhino, CreatureType::Warrior], 6, 6)
    }
}

/// Brokers Charm — bite, destroy an enchantment, or draw two.
pub fn brokers_charm() -> CardDefinition {
    spell(
        "Brokers Charm",
        cost(&[g(), w(), u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamageEqualToPower {
                    source: Selector::Target(0),
                    target: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                    },
                },
            ]),
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
    )
}

/// Brokers Confluence — choose three (repeats allowed): proliferate, phase
/// out a creature, counter an activated or triggered ability.
pub fn brokers_confluence() -> CardDefinition {
    spell(
        "Brokers Confluence",
        cost(&[generic(2), g(), w(), u()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Proliferate,
                Effect::PhaseOut { what: target_filtered(R::Creature), until_source_leaves: false },
                Effect::CounterAbility { what: target_filtered(R::HasAbilityOnStack) },
            ],
            min: 3,
            max: 3,
            allow_repeats: true,
        },
    )
}

/// Contractual Safeguard — addendum: a shield counter first; then a kind of
/// counter on one of your creatures goes on each of the others.
pub fn contractual_safeguard() -> CardDefinition {
    spell(
        "Contractual Safeguard",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::YourMainPhase,
            then: Box::new(Effect::SpreadCounterKindToOthers { shield_first: true }),
            else_: Box::new(Effect::SpreadCounterKindToOthers { shield_first: false }),
        },
    )
}

/// Damning Verdict — destroy all creatures with no counters on them.
pub fn damning_verdict() -> CardDefinition {
    spell(
        "Damning Verdict",
        cost(&[generic(3), w(), w()]),
        CardType::Sorcery,
        Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::HasNoCounters)) },
    )
}

/// Declaration in Stone — exile a creature and its namesakes under the same
/// controller; they investigate per nontoken creature exiled.
pub fn declaration_in_stone() -> CardDefinition {
    spell(
        "Declaration in Stone",
        cost(&[generic(1), w()]),
        CardType::Sorcery,
        Effect::ExileSameNameInvestigate { what: target_filtered(R::Creature) },
    )
}

/// Denry Klin, Editor in Chief — enters with a +1/+1, first strike or
/// vigilance counter; each other nontoken creature you control copies its
/// counters. Its own entry triggers it too, so the chosen counter doubles.
pub fn denry_klin_editor_in_chief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::ChooseMode(vec![
                    counter(Selector::This, CounterType::PlusOnePlusOne),
                    keyword_counter(Selector::This, Keyword::FirstStrike),
                    keyword_counter(Selector::This, Keyword::Vigilance),
                ]),
                Effect::CopyCountersOnto { from: Selector::This, to: Selector::This },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::Creature.and(R::NotToken).and(R::OtherThanSource),
                        },
                        Predicate::EntityMatches { what: Selector::This, filter: R::WithAnyCounter },
                    ]),
                ),
                effect: Effect::CopyCountersOnto { from: Selector::This, to: Selector::TriggerSource },
            },
        ],
        ..legendary(creature(
            "Denry Klin, Editor in Chief",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Cat, CreatureType::Advisor],
            2,
            2,
        ))
    }
}

fn fish() -> TokenDefinition {
    TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Blue],
        keywords: vec![Keyword::Unblockable],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        ..Default::default()
    }
}

/// Exotic Pets — two unblockable Fish, then a counter of each kind among
/// your creatures on one of them.
pub fn exotic_pets() -> CardDefinition {
    spell(
        "Exotic Pets",
        cost(&[generic(1), w(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: Arc::new(fish()) },
            Effect::AddCounterOfEachKindAmong {
                onto: Selector::LastCreatedTokens,
                who: PlayerRef::You,
                filter: R::Creature,
                or_plus_one: false,
            },
        ]),
    )
}

/// Family's Favor — whenever you attack, a shield counter on an attacker,
/// which it can cash in for a card when it connects.
pub fn familys_favor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::Seq(vec![
                counter(target_filtered(R::Creature.and(R::IsAttacking)), CounterType::Shield),
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                        effect: Effect::If {
                            cond: Predicate::EntityMatches {
                                what: Selector::This,
                                filter: R::WithCounter(CounterType::Shield),
                            },
                            then: Box::new(Effect::Seq(vec![
                                Effect::RemoveCounter {
                                    what: Selector::This,
                                    kind: CounterType::Shield,
                                    amount: Value::ONE,
                                },
                                Effect::Draw { who: Selector::You, amount: Value::ONE },
                            ])),
                            else_: Box::new(Effect::Noop),
                        },
                    }),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..spell("Family's Favor", cost(&[generic(2), g()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Jenara, Asura of War — a flier that grows for {1}{W}.
pub fn jenara_asura_of_war() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: counter(Selector::This, CounterType::PlusOnePlusOne),
            ..Default::default()
        }],
        ..legendary(creature("Jenara, Asura of War", cost(&[g(), w(), u()]), vec![CreatureType::Angel], 3, 3))
    }
}

/// Littjara Mirrorlake — a tapped {U} land that sacrifices into a copy of
/// your creature with an extra +1/+1 counter. Residual: the counter is put
/// on after the copy enters.
pub fn littjara_mirrorlake() -> CardDefinition {
    CardDefinition {
        name: "Littjara Mirrorlake",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Blue]) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g(), g(), u()]),
                tap_cost: true,
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::CreateTokenCopyOf {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: target_filtered(R::Creature.and(R::ControlledByYou)),
                        extra_creature_types: vec![],
                        extra_card_types: vec![],
                        override_pt: None,
                        override_colors: None,
                        enters_tapped: false,
                        non_legendary: false,
                        legendary: false,
                        extra_keywords: vec![],
                    },
                    counter(Selector::LastCreatedToken, CounterType::PlusOnePlusOne),
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oracle's Vault — pay {2} to play the top card this turn and add a brick;
/// at three bricks, the top card plays for free.
pub fn oracles_vault() -> CardDefinition {
    let impulse = |pay_own_cost| Effect::ExileTopAndGrantMayPlay {
        who: PlayerRef::You,
        count: Value::ONE,
        duration: crate::card::MayPlayDuration::EndOfThisTurn,
        pay_any_color: false,
        max_mana_value: None,
        pay_own_cost,
        uncast_penalty: None,
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![impulse(true), counter(Selector::This, CounterType::Brick)]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SourceHasCountersAtLeast { counter: CounterType::Brick, n: 3 }),
                effect: impulse(false),
                ..Default::default()
            },
        ],
        ..spell("Oracle's Vault", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Park Heights Maverick — dethrone; can't be blocked by power 2 or less;
/// proliferates on combat damage to a player or when it dies.
pub fn park_heights_maverick() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByPowerAtMost(2)],
        triggered_abilities: vec![
            dethrone(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Proliferate,
            },
            on_dies(Effect::Proliferate),
        ],
        ..creature(
            "Park Heights Maverick",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Skyboon Evangelist — support 6; your creatures with counters fly when they
/// attack an opponent.
pub fn skyboon_evangelist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::SupportCounters { max_targets: 6, filter: R::Creature.and(R::OtherThanSource) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::WithAnyCounter.and(R::IsAttackingAnOpponent),
                    },
                ),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..creature(
            "Skyboon Evangelist",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Advisor],
            3,
            3,
        )
    }
}

/// Skyship Plunderer — combat damage to a player gives a permanent or player
/// one more counter of each kind it has. Residual: a player's energy,
/// experience and poison only.
pub fn skyship_plunderer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounterOfEachKindOn { what: target_filtered(R::Permanent.or(R::Player)) },
        }],
        ..creature(
            "Skyship Plunderer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            1,
        )
    }
}

/// Storm of Forms — bounce a nonland permanent, copied once per kind of
/// counter among your permanents.
pub fn storm_of_forms() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::This, count: kinds_you_control() },
        }],
        ..spell(
            "Storm of Forms",
            cost(&[generic(3), u()]),
            CardType::Instant,
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )
    }
}
