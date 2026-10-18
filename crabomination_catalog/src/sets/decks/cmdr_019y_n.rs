//! Commander: 2026-10-18 (`019Y6J4R`) closers, fourteenth batch — each the
//! one missing card of an otherwise-buildable EDHREC average deck. Tests in
//! `tests/recent_b/cmdr_019y_n.rs`.

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, ModesWiden, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u, w};
use std::sync::Arc;

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn shrink(n: i32) -> Effect {
    Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(-n),
        toughness: Value::Const(-n),
        duration: Duration::EndOfTurn,
    }
}

/// Magnificent End — {4}{W} instant. Costs {3} less if it targets a tapped
/// creature (CR 601.2f). 5 damage to target creature.
pub fn magnificent_end() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::Creature.and(R::Tapped), 3)),
        ..spell(
            "Magnificent End",
            cost(&[generic(4), w()]),
            CardType::Instant,
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) },
        )
    }
}

/// Depower — {2}{U} instant. Costs {2} less if it targets an attacking
/// creature. Target creature gets -4/-0 until end of turn; draw a card.
pub fn depower() -> CardDefinition {
    CardDefinition {
        self_cost_reduction_if_target: Some((R::Creature.and(R::IsAttacking), 2)),
        ..spell(
            "Depower",
            cost(&[generic(2), u()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-4),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        )
    }
}

/// Draconic Roar — {1}{R} instant. As an additional cost, you may reveal a
/// Dragon card from your hand. 3 damage to target creature; if you revealed a
/// Dragon or controlled one as you cast it, 3 damage to that creature's
/// controller. (A Dragon in hand as it's cast is the reveal — revealing is
/// free and only ever helps.)
pub fn draconic_roar() -> CardDefinition {
    let dragon = || R::HasCreatureType(CreatureType::Dragon);
    CardDefinition {
        as_cast_condition: Some(Predicate::Any(vec![
            Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: dragon() }),
            Predicate::SelectorExists(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: dragon() }),
        ])),
        ..spell(
            "Draconic Roar",
            cost(&[generic(1), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(3) },
                Effect::If {
                    cond: Predicate::CastConditionHeld,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::Const(3),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Stomped by the Foot — {1}{B} instant. Kicker—sacrifice an artifact or
/// creature. Target creature gets -2/-2 until end of turn, or -5/-5 kicked.
pub fn stomped_by_the_foot() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::SacrificePermanent { filter: R::Artifact.or(R::Creature), count: 1 }),
        ..spell(
            "Stomped by the Foot",
            cost(&[generic(1), b()]),
            CardType::Instant,
            Effect::If { cond: Predicate::SpellWasKicked, then: Box::new(shrink(5)), else_: Box::new(shrink(2)) },
        )
    }
}

/// Pyrrhic Strike — {2}{W} instant. As an additional cost, you may blight 2.
/// Choose one — both if that cost was paid: destroy target artifact or
/// enchantment; destroy target creature with mana value 3 or greater.
pub fn pyrrhic_strike() -> CardDefinition {
    CardDefinition {
        kicker_action_cost: Some(AdditionalCastCost::Blight { n: 2 }),
        modes_widen: Some(ModesWiden::Kicked),
        ..spell(
            "Pyrrhic Strike",
            cost(&[generic(2), w()]),
            CardType::Instant,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                    Effect::Destroy { what: target_filtered(R::Creature.and(R::ManaValueAtLeast(3))) },
                ],
                min: 1,
                max: 1,
                allow_repeats: false,
            },
        )
    }
}

/// Valkyrie's Sword — {1}{W} Equipment. ETB: you may pay {4}{W}; if you do,
/// create a 4/4 white Angel Warrior token with flying and vigilance and attach
/// this to it. Equipped creature gets +2/+1. Equip {3}.
pub fn valkyries_sword() -> CardDefinition {
    let angel = TokenDefinition {
        name: "Angel Warrior".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Valkyrie's Sword",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {4}{W} for a 4/4 Angel Warrior wielding the Sword?".into(),
            mana_cost: cost(&[generic(4), w()]),
            body: Box::new(Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(angel) },
                Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Bifur, Melodic Rider — {4}{R/W}{R/W} legendary 4/5 Dwarf Bard. Storied.
/// Whenever Bifur enters or attacks, a +1/+1 counter on target creature. As
/// long as you have an enduring story, a Dwarf you control's triggered
/// abilities trigger an additional time.
pub fn bifur_melodic_rider() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    let counter = || Effect::AddCounter {
        what: target_filtered(R::Creature),
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    };
    CardDefinition {
        name: "Bifur, Melodic Rider",
        cost: cost(&[generic(4), rw(), rw()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Bard], ..Default::default() },
        power: 4,
        toughness: 5,
        static_abilities: vec![
            StaticAbility { description: "Storied", effect: StaticEffect::Storied },
            StaticAbility {
                description: "As long as you have an enduring story, if a triggered ability of a Dwarf you control triggers, that ability triggers an additional time.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasEnduringStory { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::DoubleControllerTriggersOfType {
                        types: vec![CreatureType::Dwarf],
                        exclude_source: false,
                    }),
                },
            },
        ],
        triggered_abilities: vec![
            etb(counter()),
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: counter() },
        ],
        ..Default::default()
    }
}
