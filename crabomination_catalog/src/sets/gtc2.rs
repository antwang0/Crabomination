//! Gatecrash (GTC) wave 2: the Battalion ability word, Bloodrush, an Equipment,
//! and a spread of guild uncommons. Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, Subtypes,
};
use crate::effect::shortcut::{on_attack, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, Value as V};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}

/// Battalion (CR ability word) — a SelfSource "attacks" trigger gated on three
/// or more attackers this combat.
fn battalion(effect: Effect) -> crate::card::TriggeredAbility {
    crate::card::TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
            .with_filter(Predicate::AttackingWithAtLeast(3)),
        effect,
    }
}

/// Bomber Corps — {1}{R} 1/2 Human Soldier. Battalion: deal 1 damage to any target.
pub fn bomber_corps() -> CardDefinition {
    CardDefinition {
        name: "Bomber Corps",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 1,
        toughness: 2,
        triggered_abilities: vec![battalion(Effect::DealDamage { to: target_any(), amount: V::ONE })],
        ..Default::default()
    }
}

/// Warmind Infantry — {2}{R} 2/3 Elemental Soldier. Battalion: +2/+0 EOT.
pub fn warmind_infantry() -> CardDefinition {
    CardDefinition {
        name: "Warmind Infantry",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental, CreatureType::Soldier]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![battalion(Effect::PumpPT {
            what: Selector::This,
            power: V::Const(2),
            toughness: V::ZERO,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Daring Skyjek — {1}{W} 3/1 Human Knight. Battalion: gains flying EOT.
pub fn daring_skyjek() -> CardDefinition {
    CardDefinition {
        name: "Daring Skyjek",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Knight]),
        power: 3,
        toughness: 1,
        triggered_abilities: vec![battalion(Effect::GrantKeyword {
            what: Selector::This,
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Nav Squad Commandos — {4}{W} 3/5 Human Soldier. Battalion: +1/+1 EOT, untap it.
pub fn nav_squad_commandos() -> CardDefinition {
    CardDefinition {
        name: "Nav Squad Commandos",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Soldier]),
        power: 3,
        toughness: 5,
        triggered_abilities: vec![battalion(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: V::ONE,
                toughness: V::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::This, up_to: None },
        ]))],
        ..Default::default()
    }
}

/// Keymaster Rogue — {3}{U} 3/2 Human Rogue. Unblockable. ETB: return a
/// creature you control to its owner's hand.
pub fn keymaster_rogue() -> CardDefinition {
    CardDefinition {
        name: "Keymaster Rogue",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: Selector::Take {
                inner: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                count: Box::new(V::ONE),
            },
            to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}

/// Truefire Paladin — {R}{W} 2/2 Human Knight with vigilance. {R}{W}: +2/+0 EOT;
/// {R}{W}: first strike EOT.
pub fn truefire_paladin() -> CardDefinition {
    use crate::mana::colored;
    CardDefinition {
        name: "Truefire Paladin",
        cost: cost(&[colored(Color::Red), colored(Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Knight]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r(), w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: V::Const(2),
                    toughness: V::ZERO,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r(), w()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Horror of the Dim — {4}{B} 3/4 Horror. {U}: hexproof EOT.
pub fn horror_of_the_dim() -> CardDefinition {
    CardDefinition {
        name: "Horror of the Dim",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Horror]),
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gateway Shade — {2}{B} 1/1 Shade. {B}: +1/+1 EOT. Tap an untapped Gate you
/// control: +2/+2 EOT.
pub fn gateway_shade() -> CardDefinition {
    CardDefinition {
        name: "Gateway Shade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Shade]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: V::ONE,
                    toughness: V::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_other_filter: Some(R::HasLandType(LandType::Gate)),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: V::Const(2),
                    toughness: V::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rubblebelt Raiders — {1}{R/G}{R/G}{R/G} 3/3 Human Warrior. Attacks: put a
/// +1/+1 counter on it for each attacking creature you control.
pub fn rubblebelt_raiders() -> CardDefinition {
    use crate::mana::hybrid;
    let rg = || hybrid(Color::Red, Color::Green);
    CardDefinition {
        name: "Rubblebelt Raiders",
        cost: cost(&[generic(1), rg(), rg(), rg()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Warrior]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: V::count(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
            )),
        })],
        ..Default::default()
    }
}

/// Riot Gear — {2} Equipment. Equipped creature gets +1/+2. Equip {2}.
pub fn riot_gear() -> CardDefinition {
    CardDefinition {
        name: "Riot Gear",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        ..Default::default()
    }
}

/// Skarrg Goliath — {6}{G}{G} 9/9 Beast with trample. Bloodrush — {5}{G}{G},
/// Discard this card: target attacking creature gets +9/+9 and gains trample EOT.
pub fn skarrg_goliath() -> CardDefinition {
    CardDefinition {
        name: "Skarrg Goliath",
        cost: cost(&[generic(6), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Beast]),
        power: 9,
        toughness: 9,
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::IsAttacking)),
                    power: V::Const(9),
                    toughness: V::Const(9),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Predator's Rapport — {2}{G} Instant. You gain life equal to target creature
/// you control's power plus its toughness.
pub fn predators_rapport() -> CardDefinition {
    CardDefinition {
        name: "Predator's Rapport",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: V::Sum(vec![
                // The filtered slot-0 selector declares the target (a creature
                // you control); the toughness half reuses the resolved target.
                V::PowerOf(Box::new(target_filtered(R::Creature.and(R::ControlledByYou)))),
                V::ToughnessOf(Box::new(Selector::Target(0))),
            ]),
        },
        ..Default::default()
    }
}
