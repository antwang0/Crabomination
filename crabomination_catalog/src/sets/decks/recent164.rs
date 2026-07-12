//! A Foundations wave built on new/underused primitives — incoming combat-damage
//! prevention, excess-damage token payoffs, and a second-draw self-copy. Tests
//! in `crabomination/src/tests/recent164.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, CounterType, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility,
    Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{cost, generic, r, u, w, x, Color};

/// Fleeting Flight — {W} Instant. Put a +1/+1 counter on target creature. It
/// gains flying until end of turn. Prevent all combat damage that would be dealt
/// to it this turn.
pub fn fleeting_flight() -> CardDefinition {
    let tgt = || Selector::TargetFiltered { slot: 0, filter: R::Creature };
    CardDefinition {
        name: "Fleeting Flight",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: tgt(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GrantKeyword { what: tgt(), keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            Effect::PreventCombatDamageToTargetThisTurn { target: tgt() },
        ]),
        ..Default::default()
    }
}

/// Goblin Negotiation — {X}{R}{R} Sorcery. Deal X damage to target creature.
/// Create a 1/1 red Goblin token for each point of excess damage dealt this way.
pub fn goblin_negotiation() -> CardDefinition {
    CardDefinition {
        name: "Goblin Negotiation",
        cost: cost(&[x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::XFromCost },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ExcessDamageDealtThisResolution,
                definition: goblin_token(),
            },
        ]),
        ..Default::default()
    }
}

fn goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    }
}

/// Homunculus Horde — {3}{U} 2/2 Homunculus. Whenever you draw your second card
/// each turn, create a token that's a copy of this creature.
pub fn homunculus_horde() -> CardDefinition {
    CardDefinition {
        name: "Homunculus Horde",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Homunculus], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::You, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        }],
        ..Default::default()
    }
}
