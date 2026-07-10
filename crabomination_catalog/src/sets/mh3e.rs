//! Modern Horizons 3 (MH3), batch 5. Energy payoffs (variable-{E} costs), an
//! attack-count observer, a graveyard/burn spell, a control-exchange Drake and
//! a ramp dig. Tests in `tests/mh3e.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, Subtypes,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, generic, r, u, w};

/// Izzet Generatorium — {U}{R} artifact. If you would get one or more {E}, you
/// get that many plus one instead. {T}: Draw a card. Activate only if you've
/// paid or lost four or more {E} this turn.
pub fn izzet_generatorium() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Izzet Generatorium",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "If you would get one or more {E}, you get that many plus one instead.",
            effect: StaticEffect::EnergyGainBonus { amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::EnergyPaidThisTurnAtLeast { who: PlayerRef::You, n: 4 }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unstable Amulet — {1}{R} artifact. ETB: get {E}{E}. Whenever you cast a
/// spell from anywhere other than your hand, deal 1 damage to each opponent.
/// {T}, Pay {E}{E}: exile the top card of your library; you may play it this
/// turn. (The "until you exile another card with this" window is approximated
/// as end-of-turn.)
pub fn unstable_amulet() -> CardDefinition {
    CardDefinition {
        name: "Unstable Amulet",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::AddEnergy(Value::Const(2))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::SpellNotCastFromHand,
                    },
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            energy_cost: 2,
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                uncast_penalty: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Planar Genesis — {G}{U} instant. Look at the top four cards; you may put a
/// land onto the battlefield tapped, otherwise put a card into your hand; put
/// the rest on the bottom in a random order.
pub fn planar_genesis() -> CardDefinition {
    CardDefinition {
        name: "Planar Genesis",
        cost: cost(&[crate::mana::g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::LookTopDeployLandOrHand { count: Value::Const(4) },
        ..Default::default()
    }
}

/// Reiterating Bolt — {1}{R} sorcery. Deals 3 damage to target creature or
/// planeswalker. (Replicate—Pay {E}{E}{E} is dropped: the engine models
/// Replicate as a mana cost only, not an energy cost.)
pub fn reiterating_bolt() -> CardDefinition {
    CardDefinition {
        name: "Reiterating Bolt",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Planeswalker)),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Vega, the Watcher — {1}{W}{U} 2/2 Bird Spirit with flying. Whenever you cast
/// a spell from anywhere other than your hand, draw a card.
pub fn vega_the_watcher() -> CardDefinition {
    CardDefinition {
        name: "Vega, the Watcher",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::SpellNotCastFromHand,
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Volatile Stormdrake — {1}{U} 3/2 Drake with flying. ETB: exchange control of
/// it and target creature an opponent controls; if you do, get {E}{E}{E}{E},
/// then sacrifice that creature unless you pay {E} equal to its mana value.
/// (The "hexproof from activated and triggered abilities" rider is dropped.)
pub fn volatile_stormdrake() -> CardDefinition {
    CardDefinition {
        name: "Volatile Stormdrake",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Drake], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExchangeControl {
                a: Selector::This,
                b: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByOpponent) },
            },
            Effect::AddEnergy(Value::Const(4)),
            Effect::PayEnergyOrElseValue {
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                otherwise: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
            },
        ]))],
        ..Default::default()
    }
}

/// Jolted Awake — {W} sorcery. Target artifact or creature card in your
/// graveyard; get {E}{E}, then you may pay {E} equal to its mana value to
/// return it to the battlefield. Cycling {2}. (Printed "up to one" target
/// modeled as a required target.)
pub fn jolted_awake() -> CardDefinition {
    CardDefinition {
        name: "Jolted Awake",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Seq(vec![
            Effect::AddEnergy(Value::Const(2)),
            Effect::PayEnergyValue {
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
                then: Box::new(Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::InYourGraveyard.and(R::Artifact.or(R::Creature)),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Lethal Throwdown — {B} sorcery. Additional cost: sacrifice a creature (or a
/// modified creature). Destroy target creature or planeswalker; if the modified
/// creature was sacrificed, draw a card. (The two additional-cost options are
/// folded into a cast-time `ChooseMode`.)
pub fn lethal_throwdown() -> CardDefinition {
    CardDefinition {
        name: "Lethal Throwdown",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ChooseMode(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature },
                Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::Creature.and(R::IsModified),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
            ]),
            Effect::Destroy { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        ]),
        ..Default::default()
    }
}

/// Pyretic Rebirth — {2}{B}{R} instant. Return target artifact or creature card
/// from your graveyard to hand; deal damage equal to its mana value to a
/// creature or planeswalker. (Printed as "up to one" damage target; modeled as
/// a required second target.)
pub fn pyretic_rebirth() -> CardDefinition {
    CardDefinition {
        name: "Pyretic Rebirth",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(R::Artifact.or(R::Creature)),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::Planeswalker) },
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Argent Dais — {1}{W} artifact. Enters with two oil counters. Whenever two
/// or more creatures attack, put an oil counter on it. {2}, {T}, remove two
/// oil: exile another target nonland permanent; its controller draws two.
pub fn argent_dais() -> CardDefinition {
    CardDefinition {
        name: "Argent Dais",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Oil, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedWithCountAtLeast { who: PlayerRef::ActivePlayer, at_least: 2 },
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Oil, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Oil, 2)),
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(R::Permanent.and(R::Nonland).and(R::OtherThanSource)) },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Glimpse the Impossible — {2}{R} sorcery. Exile the top three cards; you may
/// play them this turn. At the next end step, each still-exiled card is put
/// into your graveyard and makes a 0/1 Eldrazi Spawn.
pub fn glimpse_the_impossible() -> CardDefinition {
    CardDefinition {
        name: "Glimpse the Impossible",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(3),
            duration: MayPlayDuration::EndOfThisTurn,
            pay_any_color: false,
            uncast_penalty: Some(Box::new(Effect::Seq(vec![
                Effect::Move { what: Selector::Target(0), to: ZoneDest::Graveyard },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::eldrazi_spawn_token(),
                },
            ]))),
        },
        ..Default::default()
    }
}

/// Chthonian Nightmare — {1}{B} enchantment. ETB: get {E}{E}{E}. Pay X {E},
/// sacrifice a creature, return this to hand: reanimate a creature card with
/// mana value X from your graveyard. Sorcery-speed.
pub fn chthonian_nightmare() -> CardDefinition {
    CardDefinition {
        name: "Chthonian Nightmare",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::AddEnergy(Value::Const(3)))],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            energy_x_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            return_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueExactlyXFromCost)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
