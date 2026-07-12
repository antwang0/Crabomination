//! Aetherdrift (DFT) gap batch: a max-speed "each player without max speed"
//! damage enchantment and an attacked-by trigger that debuffs the attackers.
//! Tests in `crabomination/src/tests/recent175.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::etb;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{cost, generic, r, u};

/// Magmakin Artillerist — {2}{R} 1/4 Elemental Pirate. Whenever you discard one
/// or more cards, deal that much damage to each opponent. Cycling {1}{R}. When
/// you cycle this card, it deals 1 damage to each opponent.
pub fn magmakin_artillerist() -> CardDefinition {
    let bolt_opponents = |amount: Value| Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount,
    };
    CardDefinition {
        name: "Magmakin Artillerist",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[generic(1), r()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DiscardedOneOrMore, EventScope::YourControl),
                effect: bolt_opponents(Value::TriggerEventAmount),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: bolt_opponents(Value::ONE),
            },
        ],
        ..Default::default()
    }
}

/// Outpace Oblivion — {2}{R} Enchantment. Start your engines! ETB: deal 5 damage
/// to up to one target creature or planeswalker. {2}, Sacrifice this: deal 2
/// damage to each player who doesn't have max speed.
pub fn outpace_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Outpace Oblivion",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            filter: R::Creature.or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayerWithoutMaxSpeed),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sabotage Strategist — {2}{U}{U} 2/2 Vedalken Ranger. Flying, vigilance.
/// Whenever one or more creatures attack you, those creatures get -1/-0 until
/// end of turn. Exhaust — {5}{U}{U}: put three +1/+1 counters on this.
/// (Attacks on a planeswalker you control also fire it — a slight over-fire.)
pub fn sabotage_strategist() -> CardDefinition {
    CardDefinition {
        name: "Sabotage Strategist",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u(), u()]),
            exhaust: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
