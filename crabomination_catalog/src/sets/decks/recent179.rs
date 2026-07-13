//! FDN (Foundations) gap batch — commons/uncommons/rares on existing
//! primitives: Twinblade Blessing (flash double-strike Aura), Tragic Banshee
//! (Morbid -1/-1 → -13/-13), Midnight Snack (Raid Food + life-gained drain),
//! Uncharted Voyage (owner-choice tuck + surveil), Raise the Past (mass
//! reanimate MV≤2), Sylvan Scavenging (end-step modal), Ravenous Amulet
//! (sac-to-draw charge storage), and Zul Ashur (Ward + graveyard Zombie
//! caster). Tests in `crabomination/src/tests/recent179.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
    WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, ZoneDest, ZoneRef,
};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, u, w};

/// Twinblade Blessing — {1}{W}{W} Aura with flash. Enchanted creature has
/// double strike.
pub fn twinblade_blessing() -> CardDefinition {
    CardDefinition {
        name: "Twinblade Blessing",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Tragic Banshee — {4}{B} 5/3 Spirit. Morbid ETB: target creature an opponent
/// controls gets -1/-1 until end of turn, or -13/-13 instead if a creature died
/// this turn.
pub fn tragic_banshee() -> CardDefinition {
    CardDefinition {
        name: "Tragic Banshee",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 5,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE },
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-13),
                toughness: Value::Const(-13),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            }),
        })],
        ..Default::default()
    }
}

/// Midnight Snack — {2}{B} Enchantment. Raid: at your end step, if you attacked
/// this turn, create a Food. {2}{B}, Sacrifice this: target opponent loses X
/// life, where X is the life you gained this turn.
pub fn midnight_snack() -> CardDefinition {
    CardDefinition {
        name: "Midnight Snack",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::If {
                cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: crabomination_base::tokens::food_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::LoseLife {
                who: target_filtered(R::OpponentPlayer),
                amount: Value::LifeGainedThisTurn(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Uncharted Voyage — {3}{U} Instant. Target creature's owner puts it on the
/// top or bottom of their library (their choice). Surveil 1.
pub fn uncharted_voyage() -> CardDefinition {
    CardDefinition {
        name: "Uncharted Voyage",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::OwnerChoice,
                },
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Raise the Past — {2}{W}{W} Sorcery. Return all creature cards with mana
/// value 2 or less from your graveyard to the battlefield.
pub fn raise_the_past() -> CardDefinition {
    CardDefinition {
        name: "Raise the Past",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature.and(R::ManaValueAtMost(2)),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Sylvan Scavenging — {1}{G}{G} Enchantment. At your end step, choose one —
/// put a +1/+1 counter on target creature you control; or create a 3/3 green
/// Raccoon if you control a creature with power 4 or greater.
pub fn sylvan_scavenging() -> CardDefinition {
    let raccoon = crate::card::TokenDefinition {
        name: "Raccoon".to_string(),
        colors: vec![crate::mana::Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Raccoon], ..Default::default() },
        power: 3,
        toughness: 3,
        ..Default::default()
    };
    CardDefinition {
        name: "Sylvan Scavenging",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4)),
                    )),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: raccoon.clone(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Ravenous Amulet — {2} Artifact. {1}, {T}, Sacrifice a creature: Draw a card
/// and put a soul counter on this. Activate only as a sorcery. {4}, {T},
/// Sacrifice this: each opponent loses life equal to the soul counters on it.
pub fn ravenous_amulet() -> CardDefinition {
    CardDefinition {
        name: "Ravenous Amulet",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_other_filter: Some((R::Creature, 1)),
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Charge,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Zul Ashur, Lich Lord — {1}{B} 2/2 legendary Zombie Warlock. Ward—Pay 2 life.
/// {T}: You may cast target Zombie creature card from your graveyard this turn.
pub fn zul_ashur_lich_lord() -> CardDefinition {
    CardDefinition {
        name: "Zul Ashur, Lich Lord",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ward(WardCost::Life(2))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantMayPlay {
                what: target_filtered(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Zombie))
                        .and(R::InYourGraveyard),
                ),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
