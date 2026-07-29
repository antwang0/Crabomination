//! Recover (CR 702.59) completion plus draw-lock and blocking-creature
//! payoffs. Tests in `recent_b/recent_307`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    counter_target_spell, draw, recover, recover_paying_half_life, target_filtered,
};
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, StaticAbility, StaticEffect};
use crate::mana::{b, cost, g, generic, u, w};

// ── Recover (CR 702.59) ──────────────────────────────────────────────────────

/// Krovikan Rot — {2}{B} Instant. Destroy target creature with power 2 or
/// less. Recover {1}{B}{B}.
pub fn krovikan_rot() -> CardDefinition {
    CardDefinition {
        name: "Krovikan Rot",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
        },
        triggered_abilities: vec![recover(cost(&[generic(1), b(), b()]))],
        ..Default::default()
    }
}

/// Controvert — {2}{U}{U} Instant. Counter target spell. Recover {2}{U}{U}.
pub fn controvert() -> CardDefinition {
    CardDefinition {
        name: "Controvert",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: counter_target_spell(),
        triggered_abilities: vec![recover(cost(&[generic(2), u(), u()]))],
        ..Default::default()
    }
}

/// Garza's Assassin — {B}{B}{B} 2/2 Human Assassin. Sacrifice: destroy target
/// nonblack creature. Recover—Pay half your life, rounded up.
pub fn garzas_assassin() -> CardDefinition {
    CardDefinition {
        name: "Garza's Assassin",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasColor(crate::mana::Color::Black).negate())),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![recover_paying_half_life()],
        ..Default::default()
    }
}

// ── Draw locks (CR 121.2b) ───────────────────────────────────────────────────

/// Leovold, Emissary of Trest — {B}{G}{U} 3/3 Elf Advisor. Each opponent can't
/// draw more than one card each turn; whenever you or a permanent you control
/// becomes the target of an opponent's spell or ability, you may draw a card.
pub fn leovold_emissary_of_trest() -> CardDefinition {
    CardDefinition {
        name: "Leovold, Emissary of Trest",
        cost: cost(&[b(), g(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Advisor],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Each opponent can't draw more than one card each turn.",
            effect: StaticEffect::CapDrawsPerTurn {
                target: PlayerStaticTarget::EachOpponent,
                max: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(draw(1)),
            },
        }],
        ..Default::default()
    }
}

// ── Blocking-creature payoffs ────────────────────────────────────────────────

/// Captain's Defense — {W} Instant. Target blocking creature gets +2/+2 until
/// end of turn. Draw a card.
pub fn captains_defense() -> CardDefinition {
    CardDefinition {
        name: "Captain's Defense",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsBlocking)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Aang's Defense — {W} Instant. Target blocking creature you control gets
/// +2/+2 until end of turn. Draw a card.
pub fn aangs_defense() -> CardDefinition {
    CardDefinition {
        name: "Aang's Defense",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsBlocking).and(R::ControlledByYou)),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Outflank — {W} Instant. Deals damage to target attacking or blocking
/// creature equal to the number of creatures you control.
pub fn outflank() -> CardDefinition {
    CardDefinition {
        name: "Outflank",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
            amount: Value::CreatureCountControlledBy(PlayerRef::You),
        },
        ..Default::default()
    }
}
