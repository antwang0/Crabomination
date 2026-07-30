//! **Mayhem** cards (CR 702.187). A static ability in the graveyard: "As long
//! as you discarded this card this turn, you may cast it from your graveyard by
//! paying its mayhem cost rather than its mana cost." Wired through the
//! flashback machinery (`GameAction::CastMayhem` → `cast_flashback`), gated on
//! `Player.discarded_this_turn`; the spell is exiled if it would leave the
//! stack. Tracked in `DECK_FEATURES.md`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword, Predicate,
    SelectionRequirement, Selector, Subtypes, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, generic, r};

/// Electro's Bolt — {2}{R} Sorcery. Deal 4 damage to target creature.
/// Mayhem {1}{R}.
pub fn electros_bolt() -> CardDefinition {
    CardDefinition {
        name: "Electro's Bolt",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Mayhem(cost(&[generic(1), r()]))],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Sadistic Slash — {3}{B} Instant. Target creature gets -5/-5 until end of
/// turn. Mayhem {1}{B}.
pub fn sadistic_slash() -> CardDefinition {
    CardDefinition {
        name: "Sadistic Slash",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Mayhem(cost(&[generic(1), b()]))],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-5),
            toughness: Value::Const(-5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Raging Goblinoids — {4}{R} 5/4 Goblin Berserker with Haste. Mayhem {2}{R}.
pub fn raging_goblinoids() -> CardDefinition {
    CardDefinition {
        name: "Raging Goblinoids",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Haste, Keyword::Mayhem(cost(&[generic(2), r()]))],
        ..Default::default()
    }
}

/// Spider-Islanders — {3}{R} 4/3 Spider Horror Citizen. Mayhem {1}{R}.
pub fn spider_islanders() -> CardDefinition {
    CardDefinition {
        name: "Spider-Islanders",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spider,
                CreatureType::Horror,
                CreatureType::Citizen,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Mayhem(cost(&[generic(1), r()]))],
        ..Default::default()
    }
}

/// Prison Break — {4}{B} Sorcery. Return target creature card from your
/// graveyard to the battlefield with an additional +1/+1 counter on it.
/// Mayhem {3}{B}.
pub fn prison_break() -> CardDefinition {
    CardDefinition {
        name: "Prison Break",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Mayhem(cost(&[generic(3), b()]))],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InGraveyard),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Sandman's Quicksand — {1}{B}{B} Sorcery. All creatures get -2/-2 until end
/// of turn. If this spell's mayhem cost was paid, only creatures your opponents
/// control get -2/-2 instead. Mayhem {3}{B}.
pub fn sandmans_quicksand() -> CardDefinition {
    let all = Selector::EachPermanent(SelectionRequirement::Creature);
    let opponents = Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
    );
    let minus_two = |what: Selector| Effect::PumpPT {
        what,
        power: Value::Const(-2),
        toughness: Value::Const(-2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Sandman's Quicksand",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Mayhem(cost(&[generic(3), b()]))],
        effect: Effect::If {
            cond: Predicate::SpellWasMayhem,
            then: Box::new(minus_two(opponents)),
            else_: Box::new(minus_two(all)),
        },
        ..Default::default()
    }
}
