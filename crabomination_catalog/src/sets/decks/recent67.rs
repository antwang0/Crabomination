//! OTJ staples wave 2 (Treasures, Mercenaries, crime/second-spell payoffs). All
//! ride existing engine primitives. Tests in `tests/recent67.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Keyword, Predicate,
    Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value,
};
use crate::mana::{Color, b, cost, g, generic, r, w};
use crabomination_base::tokens;

/// The 1/1 red Mercenary token (OTJ) — "{T}: Target creature you control gets
/// +1/+0 until end of turn. Activate only as a sorcery."
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mercenary],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn treasures_you_control() -> Value {
    Value::count(Selector::EachPermanent(
        R::HasArtifactSubtype(ArtifactSubtype::Treasure).and(R::ControlledByYou),
    ))
}

/// Nezumi Linkbreaker — {B} 1/1 Rat Warlock. When it dies, create a 1/1 red
/// Mercenary.
pub fn nezumi_linkbreaker() -> CardDefinition {
    CardDefinition {
        name: "Nezumi Linkbreaker",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(mercenary_token()),
            },
        }],
        ..Default::default()
    }
}

/// Gold Rush — {1}{G} Instant. Create a Treasure token. Until end of turn, up to
/// one target creature gets +2/+2 for each Treasure you control.
pub fn gold_rush() -> CardDefinition {
    CardDefinition {
        name: "Gold Rush",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(tokens::treasure_token()),
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Times(Box::new(treasures_you_control()), Box::new(Value::Const(2))),
                toughness: Value::Times(
                    Box::new(treasures_you_control()),
                    Box::new(Value::Const(2)),
                ),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Prosperity Tycoon — {3}{W} 4/2 Human Noble. ETB create a 1/1 Mercenary. {2},
/// Sacrifice a token: this creature gains indestructible until end of turn; tap it.
pub fn prosperity_tycoon() -> CardDefinition {
    CardDefinition {
        name: "Prosperity Tycoon",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(mercenary_token()),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::IsToken, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
                Effect::Tap {
                    what: Selector::This,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ambuscade — {2}{G} Instant. Target creature you control gets +1/+0 until end
/// of turn, then deals damage equal to its power to target creature an opponent
/// controls.
pub fn ambuscade() -> CardDefinition {
    CardDefinition {
        name: "Ambuscade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamageEqualToPower {
                source: target_filtered(R::Creature.and(R::ControlledByYou)),
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Nyxborn Unicorn — {1}{W} 2/2 Enchantment Creature. Bestow {3}{W}, Mentor;
/// the enchanted creature gets +2/+2 and has mentor.
pub fn nyxborn_unicorn() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Nyxborn Unicorn",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Unicorn],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::mentor()],
        bestow: Some(cost(&[generic(3), w()])),
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![crate::effect::shortcut::mentor()],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Iron-Fist Pulverizer — {4}{R} 4/5 Giant Warrior. Reach. Whenever you cast
/// your second spell each turn, deal 2 damage to target opponent and scry 1.
pub fn iron_fist_pulverizer() -> CardDefinition {
    CardDefinition {
        name: "Iron-Fist Pulverizer",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::OpponentPlayer),
                    amount: Value::Const(2),
                },
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}
