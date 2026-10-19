//! Commander: 2026-10-19 (`01DJHkWk`) one-gap closers for the Marvel and
//! Spider-Man Villain decks the Masters of Evil cluster left one card short.
//! Tests in `tests/recent_b/cmdr_01dj_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
};
use crate::effect::shortcut::{connive, etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::mana::{ManaCost, b, cost, generic, r, u};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Armed Assailant — {2}{B} 1/3 deathtouch Human Villain; +2/+0 and menace
/// while equipped.
pub fn armed_assailant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it gets +2/+0 and has menace.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::Menace],
            },
        }],
        ..creature(
            "Armed Assailant",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Villain],
            1,
            3,
        )
    }
}

/// A.I.M. Bot — {2}{U} 2/3 flying Robot Villain artifact creature.
pub fn a_i_m_bot() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        ..creature(
            "A.I.M. Bot",
            cost(&[generic(2), u()]),
            vec![CreatureType::Robot, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Steel Wrecking Ball — {5} artifact. Enters: 5 damage to target creature.
/// {1}{R}, discard it: destroy target artifact.
pub fn steel_wrecking_ball() -> CardDefinition {
    CardDefinition {
        name: "Steel Wrecking Ball",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::Const(5),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Revoke Demise — {4}{B} sorcery, {2} less aimed at a creature card with
/// mana value 3 or less. Return target creature card from your graveyard to
/// the battlefield; gain 2 life.
pub fn revoke_demise() -> CardDefinition {
    let card = || R::Creature.and(R::InYourGraveyard);
    CardDefinition {
        name: "Revoke Demise",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(card()),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        self_cost_reduction_if_target: Some((card().and(R::ManaValueAtMost(3)), 2)),
        ..Default::default()
    }
}

/// Revel Ruiner — {3}{B} 3/1 menace Octopus Rogue; connives as it enters.
pub fn revel_ruiner() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(connive(1))],
        ..creature(
            "Revel Ruiner",
            cost(&[generic(3), b()]),
            vec![CreatureType::Octopus, CreatureType::Rogue],
            3,
            1,
        )
    }
}

/// Unliving Legionnaire — {3}{B} 3/2 flying Vampire Villain. Power-up
/// {5}{B}{B}: return up to one target creature card from your graveyard to
/// your hand; two +1/+1 counters on this creature.
pub fn unliving_legionnaire() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b(), b()]),
            power_up: true,
            effect: Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                ])),
            },
            ..Default::default()
        }],
        ..creature(
            "Unliving Legionnaire",
            cost(&[generic(3), b()]),
            vec![CreatureType::Vampire, CreatureType::Villain],
            3,
            2,
        )
    }
}

/// Titania, Rugged Rumbler — {2}{B/G} legendary 5/5 Human Villain. As an
/// additional cost, discard a card or pay {2}. Ward—discard a card or pay {2}.
pub fn titania_rugged_rumbler() -> CardDefinition {
    use crate::card::{AdditionalCastCost, Supertype, WardCost};
    use crate::mana::{Color, hybrid};
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        additional_cast_cost: vec![AdditionalCastCost::DiscardOrPay { pay: 2 }],
        keywords: vec![Keyword::Ward(WardCost::ManaOrDiscard(
            cost(&[generic(2)]),
            1,
        ))],
        ..creature(
            "Titania, Rugged Rumbler",
            cost(&[generic(2), hybrid(Color::Black, Color::Green)]),
            vec![CreatureType::Human, CreatureType::Villain],
            5,
            5,
        )
    }
}
