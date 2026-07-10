//! More BLB / DSK / FDN cards reusing existing primitives: control-gated cost
//! reduction, modal burn with exile-if-dies, Expend 4 payoffs, a may-pump +
//! delayed-sac attacker, and an additional-sacrifice-cost drain. Tests in
//! `tests/recent118.rs`.

use crate::card::{
    AdditionalCastCost, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, Value};
use crate::mana::{b, cost, g, generic, r, u};

/// Arcane Epiphany — {3}{U}{U} instant. Costs {1} less if you control a Wizard.
/// Draw three cards.
pub fn arcane_epiphany() -> CardDefinition {
    CardDefinition {
        name: "Arcane Epiphany",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_control: vec![(R::HasCreatureType(CreatureType::Wizard), 1)],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Agate Assault — {2}{R} sorcery. Choose one — deal 4 damage to target creature
/// (exile it if it would die this turn); or exile target artifact.
pub fn agate_assault() -> CardDefinition {
    CardDefinition {
        name: "Agate Assault",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                // Install the death replacement first, then deal the damage.
                Effect::ExileIfWouldDieThisTurn { what: target_filtered(R::Creature) },
                Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(4) },
            ]),
            Effect::Exile { what: target_filtered(R::Artifact) },
        ]),
        ..Default::default()
    }
}

/// Bark-Knuckle Boxer — {1}{G} 3/2 Raccoon Berserker. Whenever you expend 4, it
/// gains indestructible until end of turn.
pub fn bark_knuckle_boxer() -> CardDefinition {
    CardDefinition {
        name: "Bark-Knuckle Boxer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                .with_filter(Predicate::ExpendReached(4)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Brambleguard Veteran — {1}{G}{G} 3/4 Raccoon Warrior. Whenever you expend 4,
/// Raccoons you control get +1/+1 and gain vigilance until end of turn.
pub fn brambleguard_veteran() -> CardDefinition {
    let raccoons = || Selector::EachPermanent(R::HasCreatureType(CreatureType::Raccoon).and(R::ControlledByYou));
    CardDefinition {
        name: "Brambleguard Veteran",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Raccoon, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Expend, EventScope::YourControl)
                .with_filter(Predicate::ExpendReached(4)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: raccoons(),
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: raccoons(),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Attack-in-the-Box — {3} 2/4 Toy artifact creature. Whenever it attacks, you
/// may have it get +4/+0 until end of turn; if you do, sacrifice it at the
/// beginning of the next end step.
pub fn attack_in_the_box() -> CardDefinition {
    CardDefinition {
        name: "Attack-in-the-Box",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Toy], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "have it get +4/+0 until end of turn, then sacrifice it at the next end step".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(4),
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::AtNextEndStep { body: Box::new(Effect::SacrificeSource) },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Arbiter of Woe — {4}{B}{B} 5/4 Demon with flying. Additional cost: sacrifice
/// a creature. ETB: each opponent discards a card and loses 2 life; you draw a
/// card and gain 2 life.
pub fn arbiter_of_woe() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Arbiter of Woe",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false },
            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}
