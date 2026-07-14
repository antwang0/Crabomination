//! Foundations (FDN) gap batch 6 — commons/uncommons on existing primitives:
//! token makers, ETB pings/bounce, a tapped-creature burn, a Cat lord, an
//! attacking anthem, a donate, and french-vanilla bodies. Tests in
//! `tests/recent207.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword, Selector,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{deal, drain, etb, target_filtered};
use crate::effect::{Duration, EventKind, EventScope, EventSpec, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn dog_token() -> TokenDefinition {
    TokenDefinition {
        name: "Dog".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        ..Default::default()
    }
}

fn zombie_token() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    }
}

fn cat_lifelink_token() -> TokenDefinition {
    TokenDefinition {
        name: "Cat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Lifelink],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        ..Default::default()
    }
}

/// Release the Dogs — {3}{W} Sorcery. Create four 1/1 white Dog tokens.
pub fn release_the_dogs() -> CardDefinition {
    CardDefinition {
        name: "Release the Dogs",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(4), definition: dog_token() },
        ..Default::default()
    }
}

/// Moment of Triumph — {W} Instant. Target creature gets +2/+2; you gain 2 life.
pub fn moment_of_triumph() -> CardDefinition {
    CardDefinition {
        name: "Moment of Triumph",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Deadly Riposte — {1}{W} Instant. Deals 3 damage to target tapped creature;
/// you gain 2 life.
pub fn deadly_riposte() -> CardDefinition {
    CardDefinition {
        name: "Deadly Riposte",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            deal(3, target_filtered(R::Creature.and(R::Tapped))),
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Skeleton Archer — {3}{B} 3/3. When it enters, it deals 1 damage to any target.
pub fn skeleton_archer() -> CardDefinition {
    CardDefinition {
        name: "Skeleton Archer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Archer],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(deal(1, Selector::Target(0)))],
        ..Default::default()
    }
}

/// Maalfeld Twins — {5}{B} 4/4. When it dies, create two 2/2 black Zombies.
pub fn maalfeld_twins() -> CardDefinition {
    CardDefinition {
        name: "Maalfeld Twins",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: zombie_token() },
        }],
        ..Default::default()
    }
}

/// Rapacious Dragon — {4}{R} 3/3. Flying; when it enters, create two Treasures.
pub fn rapacious_dragon() -> CardDefinition {
    CardDefinition {
        name: "Rapacious Dragon",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crabomination_base::tokens::treasure_token(),
        })],
        ..Default::default()
    }
}

/// Exclusion Mage — {2}{U} 2/2. When it enters, return target creature an
/// opponent controls to its owner's hand.
pub fn exclusion_mage() -> CardDefinition {
    CardDefinition {
        name: "Exclusion Mage",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
        })],
        ..Default::default()
    }
}

/// Mystic Archaeologist — {1}{U} 2/1. {3}{U}{U}: Draw two cards.
pub fn mystic_archaeologist() -> CardDefinition {
    CardDefinition {
        name: "Mystic Archaeologist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deathmark — {B} Sorcery. Destroy target green or white creature.
pub fn deathmark() -> CardDefinition {
    CardDefinition {
        name: "Deathmark",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.and(R::HasColor(Color::Green).or(R::HasColor(Color::White)))),
        },
        ..Default::default()
    }
}

/// Magnigoth Sentry — {3}{G} 4/4. Reach.
pub fn magnigoth_sentry() -> CardDefinition {
    CardDefinition {
        name: "Magnigoth Sentry",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Raging Redcap — {2}{R} 1/2. Double strike.
pub fn raging_redcap() -> CardDefinition {
    CardDefinition {
        name: "Raging Redcap",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Goblin Oriflamme — {1}{R} Enchantment. Attacking creatures you control get
/// +1/+0.
pub fn goblin_oriflamme() -> CardDefinition {
    CardDefinition {
        name: "Goblin Oriflamme",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::IsAttacking)),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Vampire Neonate — {B} 0/3. {2}, {T}: Each opponent loses 1 life and you gain
/// 1 life.
pub fn vampire_neonate() -> CardDefinition {
    CardDefinition {
        name: "Vampire Neonate",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 0,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: drain(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Volley Veteran — {3}{R} 4/2. When it enters, it deals damage to target
/// creature an opponent controls equal to the number of Goblins you control.
pub fn volley_veteran() -> CardDefinition {
    CardDefinition {
        name: "Volley Veteran",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            amount: Value::CountMatching {
                sel: Box::new(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Goblin),
                }),
                filter: R::HasCreatureType(CreatureType::Goblin),
            },
        })],
        ..Default::default()
    }
}

/// Regal Caracal — {3}{W}{W} 3/3. Other Cats you control get +1/+1 and have
/// lifelink. When it enters, create two 1/1 white Cat tokens with lifelink.
pub fn regal_caracal() -> CardDefinition {
    let other_cats = || {
        Selector::EachPermanent(
            R::HasCreatureType(CreatureType::Cat)
                .and(R::ControlledByYou)
                .and(R::OtherThanSource),
        )
    };
    CardDefinition {
        name: "Regal Caracal",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 3,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "Other Cats you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: other_cats(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Other Cats you control have lifelink.",
                effect: StaticEffect::GrantKeyword { applies_to: other_cats(), keyword: Keyword::Lifelink },
            },
        ],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: cat_lifelink_token(),
        })],
        ..Default::default()
    }
}

/// Harmless Offering — {2}{R} Sorcery. Target opponent gains control of target
/// permanent you control.
pub fn harmless_offering() -> CardDefinition {
    CardDefinition {
        name: "Harmless Offering",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainControl {
            what: target_filtered(R::Permanent.and(R::ControlledByYou)),
            to: Some(PlayerRef::EachOpponent),
            duration: Duration::Permanent,
        },
        ..Default::default()
    }
}

/// Dive Down — {U} Instant. Target creature you control gets +0/+3 and gains
/// hexproof until end of turn.
pub fn dive_down() -> CardDefinition {
    CardDefinition {
        name: "Dive Down",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Syr Alin, the Lion's Claw — {3}{W}{W} 4/4 Legendary. First strike; whenever
/// it attacks, other creatures you control get +1/+1 until end of turn.
pub fn syr_alin_the_lions_claw() -> CardDefinition {
    CardDefinition {
        name: "Syr Alin, the Lion's Claw",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
