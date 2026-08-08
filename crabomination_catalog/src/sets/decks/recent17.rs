//! A seventeenth wave — a Foundations (FDN) value batch: ETB payoffs, tribal
//! lords, life-matters, and a pair of Auras/Equipment. Tests in
//! `crabomination/src/tests/recent17.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_noncreature, etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Burglar Rat — {1}{B} Rat 1/1. When it enters, each opponent discards a card.
pub fn burglar_rat() -> CardDefinition {
    CardDefinition {
        name: "Burglar Rat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        })],
        ..Default::default()
    }
}

/// Corsair Captain — {2}{U} Human Pirate 2/2. ETB: create a Treasure. Other
/// Pirates you control get +1/+1.
pub fn corsair_captain() -> CardDefinition {
    CardDefinition {
        name: "Corsair Captain",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Box::new(crate::game::effects::treasure_token()),
        })],
        static_abilities: vec![StaticAbility {
            description: "Other Pirates you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Pirate)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Crow of Dark Tidings — {2}{B} Zombie Bird 2/1 with flying. When it enters or
/// dies, mill two cards.
pub fn crow_of_dark_tidings() -> CardDefinition {
    let mill = || Effect::Mill {
        who: Selector::You,
        amount: Value::Const(2),
    };
    CardDefinition {
        name: "Crow of Dark Tidings",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(mill()), on_dies(mill())],
        ..Default::default()
    }
}

/// Crusader of Odric — {2}{W} Human Soldier. Its power and toughness are each
/// equal to the number of creatures you control.
pub fn crusader_of_odric() -> CardDefinition {
    CardDefinition {
        name: "Crusader of Odric",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        dynamic_pt: Some(DynamicPt::CreaturesControlled { base: 0 }),
        ..Default::default()
    }
}

/// Angel of Finality — {3}{W} Angel 3/4 with flying. ETB: exile a graveyard.
/// (Printed "target player's graveyard" is modeled as each opponent's, 1v1.)
pub fn angel_of_finality() -> CardDefinition {
    CardDefinition {
        name: "Angel of Finality",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ExilePlayerGraveyard {
            who: PlayerRef::EachOpponent,
            filter: None,
        })],
        ..Default::default()
    }
}

/// Bishop's Soldier — {1}{W} Vampire Soldier 2/2 with lifelink.
pub fn bishops_soldier() -> CardDefinition {
    CardDefinition {
        name: "Bishop's Soldier",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    }
}

/// Affectionate Indrik — {5}{G} Beast 4/4. ETB: it fights target creature you
/// don't control. (Printed "you may" is auto-resolved to fight when a legal
/// target exists.)
pub fn affectionate_indrik() -> CardDefinition {
    CardDefinition {
        name: "Affectionate Indrik",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::Fight {
            attacker: Selector::This,
            defender: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
    }
}

/// Angelic Edict — {4}{W} Sorcery. Exile target creature or enchantment.
pub fn angelic_edict() -> CardDefinition {
    CardDefinition {
        name: "Angelic Edict",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
            ),
        },
        ..Default::default()
    }
}

/// Broken Wings — {2}{G} Instant. Destroy target artifact, enchantment, or
/// creature with flying.
pub fn broken_wings() -> CardDefinition {
    CardDefinition {
        name: "Broken Wings",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying))),
            ),
        },
        ..Default::default()
    }
}

/// Ambush Wolf — {2}{G} Wolf 4/2 with flash. ETB: exile up to one target card
/// from a graveyard.
pub fn ambush_wolf() -> CardDefinition {
    CardDefinition {
        name: "Ambush Wolf",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wolf],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Any),
            to: ZoneDest::Exile,
        })],
        ..Default::default()
    }
}

/// Crackling Cyclops — {2}{R} Cyclops Wizard 0/4. Whenever you cast a
/// noncreature spell, it gets +3/+0 until end of turn.
pub fn crackling_cyclops() -> CardDefinition {
    CardDefinition {
        name: "Crackling Cyclops",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cyclops, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Angel of Vitality — {2}{W} Angel 2/2 with flying. Life gained is +1.
/// Gets +2/+2 while you have 25 or more life.
pub fn angel_of_vitality() -> CardDefinition {
    CardDefinition {
        name: "Angel of Vitality",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "If you would gain life, you gain that much plus 1 instead.",
                effect: StaticEffect::LifeGainBonus {
                    target: PlayerStaticTarget::Controller,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Gets +2/+2 as long as you have 25 or more life.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::PlayerLifeAtLeast {
                        who: PlayerRef::You,
                        life: 25,
                    },
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
        ],
        ..Default::default()
    }
}

/// Basilisk Collar — {1} Equipment. Equipped creature has deathtouch and
/// lifelink. Equip {2}.
pub fn basilisk_collar() -> CardDefinition {
    CardDefinition {
        name: "Basilisk Collar",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Archway Angel — {5}{W} Angel 3/4 with flying. ETB: gain 2 life for each
/// Gate you control.
pub fn archway_angel() -> CardDefinition {
    CardDefinition {
        name: "Archway Angel",
        cost: cost(&[generic(5), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::Const(2)),
                Box::new(Value::count(Selector::EachPermanent(
                    SelectionRequirement::HasLandType(LandType::Gate)
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            ),
        })],
        ..Default::default()
    }
}

/// Cemetery Recruitment — {1}{B} Sorcery. Return target creature card from your
/// graveyard to your hand. If it's a Zombie card, draw a card.
pub fn cemetery_recruitment() -> CardDefinition {
    CardDefinition {
        name: "Cemetery Recruitment",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Read the Zombie rider while the target is still in the graveyard
            // (after the Move, Target(0) points at a card that's left the zone).
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Zombie),
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

/// Seasoned Hallowblade — {1}{W} Human Warrior 3/1. Discard a card: Tap it. It
/// gains indestructible until end of turn.
pub fn seasoned_hallowblade() -> CardDefinition {
    CardDefinition {
        name: "Seasoned Hallowblade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((SelectionRequirement::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::This,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dryad Greenseeker — {1}{G} Dryad 1/3. {T}: Look at the top card of your
/// library. If it's a land card, you may reveal it and put it into your hand.
/// (Modeled via reveal-and-draw — the put-into-hand half is approximated.)
pub fn dryad_greenseeker() -> CardDefinition {
    CardDefinition {
        name: "Dryad Greenseeker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::RevealTopAndDrawIf {
                who: PlayerRef::You,
                reveal_filter: SelectionRequirement::Land,
                may_graveyard_miss: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aggressive Mammoth — {3}{G}{G}{G} Elephant 8/8 with trample. Other creatures
/// you control have trample.
pub fn aggressive_mammoth() -> CardDefinition {
    CardDefinition {
        name: "Aggressive Mammoth",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Dwarven Forge-Chanter — {1}{R} Dwarf Wizard 1/3 with prowess. Ward — Pay 2
/// life.
pub fn dwarven_forge_chanter() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Dwarven Forge-Chanter",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Prowess, Keyword::Ward(WardCost::Life(2))],
        triggered_abilities: vec![crate::effect::shortcut::prowess()],
        ..Default::default()
    }
}

/// Staunch Shieldmate — {W} Dwarf Soldier 1/3 vanilla.
pub fn staunch_shieldmate() -> CardDefinition {
    CardDefinition {
        name: "Staunch Shieldmate",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        ..Default::default()
    }
}

/// Amplify N (CR 702.38): "As this creature enters, put N +1/+1 counters on it
/// for each [type] card you reveal in your hand." Modeled by entering with
/// `n × (matching hand cards)` counters — all matching cards are auto-revealed.
fn amplify(n: i32, ct: CreatureType) -> Option<(CounterType, Value)> {
    Some((
        CounterType::PlusOnePlusOne,
        Value::Times(
            Box::new(Value::Const(n)),
            Box::new(Value::CardsInHandMatching {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCreatureType(ct),
            }),
        ),
    ))
}

/// Canopy Crawler — {3}{G} Beast 2/2 with Amplify 1 (Beast).
pub fn canopy_crawler() -> CardDefinition {
    CardDefinition {
        name: "Canopy Crawler",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: amplify(1, CreatureType::Beast),
        ..Default::default()
    }
}

/// Feral Throwback — {4}{G}{G} Beast 3/3 with trample and Amplify 2 (Beast).
pub fn feral_throwback() -> CardDefinition {
    CardDefinition {
        name: "Feral Throwback",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        enters_with_counters: amplify(2, CreatureType::Beast),
        ..Default::default()
    }
}

/// Kilnmouth Dragon — {5}{R}{R} Dragon 5/5 with flying and Amplify 3 (Dragon).
/// {T}: deals damage equal to its +1/+1 counters to any target.
pub fn kilnmouth_dragon() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    CardDefinition {
        name: "Kilnmouth Dragon",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        enters_with_counters: amplify(3, CreatureType::Dragon),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scrabbling Claws — {1} Artifact. {T}: Exile a card from a graveyard. {1},
/// Sacrifice this artifact: Exile target card from a graveyard. Draw a card.
/// (The first ability's "target player exiles" choice is auto-resolved.)
pub fn scrabbling_claws() -> CardDefinition {
    CardDefinition {
        name: "Scrabbling Claws",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Any),
                        to: ZoneDest::Exile,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
