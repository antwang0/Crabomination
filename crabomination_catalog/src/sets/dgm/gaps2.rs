//! Dragon's Maze (DGM) gap cards, wave 2 — Aetherling, Dragonshift (Overload),
//! Krasis Incubation, and the Fuse split cards. Tests in `classic_sets/dgm`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, Effect,
    EnchantmentSubtype, EquipBonus, Keyword, SelectionRequirement as R, SplitCard, SplitHalf,
    Subtypes, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes {
        creature_types: t,
        ..Default::default()
    }
}

/// Aetherling — {4}{U}{U} 4/5 Shapeshifter. {U}: blink until the next end step;
/// {U}: unblockable this turn; {1}: +1/-1; {1}: -1/+1 (all until end of turn).
pub fn aetherling() -> CardDefinition {
    let pump = |p: i32, t: i32| ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(p),
            toughness: Value::Const(t),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Aetherling",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Shapeshifter]),
        power: 4,
        toughness: 5,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::ExileReturnToOwnerNextEndStep {
                    what: Selector::This,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            pump(1, -1),
            pump(-1, 1),
        ],
        ..Default::default()
    }
}

/// Dragonshift — {1}{U}{R} Instant. Until end of turn, target creature you
/// control becomes a 4/4 blue-red Dragon with flying and loses all abilities.
/// Overload {3}{U}{U}{R}{R}.
pub fn dragonshift() -> CardDefinition {
    let animate = |what: Selector| {
        Effect::Seq(vec![
            Effect::LoseAllAbilities {
                what: what.clone(),
                duration: Duration::EndOfTurn,
            },
            Effect::BecomeColor {
                what: what.clone(),
                colors: vec![Color::Blue, Color::Red],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::BecomeCreatureType {
                what: what.clone(),
                creature_types: vec![CreatureType::Dragon],
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: what.clone(),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "Dragonshift",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: animate(target_filtered(R::Creature.and(R::ControlledByYou))),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u(), u(), r(), r()]),
            effect_override: Some(animate(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou),
            ))),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Krasis Incubation — {2}{G}{U} Aura. Enchanted creature can't attack or
/// block and its activated abilities can't be activated. {1}{G}{U}: return this
/// Aura to its owner's hand and put two +1/+1 counters on the creature.
pub fn krasis_incubation() -> CardDefinition {
    let enchanted = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Krasis Incubation",
        cost: cost(&[generic(2), g(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateAbilities,
            ],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), u()]),
            // Counters land while the Aura is still attached, then it bounces.
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: enchanted(),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Fuse split cards (CR 702.102) ───────────────────────────────────────────

/// Armed // Dangerous — {1}{R} // {3}{G} Sorcery // Sorcery, Fuse. Armed: target
/// creature gets +1/+1 and gains double strike. Dangerous: all creatures able
/// to block target creature do so.
pub fn armed_dangerous() -> CardDefinition {
    CardDefinition {
        name: "Armed // Dangerous",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::AllMustBlock,
                    duration: Duration::EndOfTurn,
                },
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Protect // Serve — {2}{W} // {1}{U} Instant // Instant, Fuse. Protect: target
/// creature gets +2/+4. Serve: target creature gets -6/-0.
pub fn protect_serve() -> CardDefinition {
    CardDefinition {
        name: "Protect // Serve",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(2),
            toughness: Value::Const(4),
            duration: Duration::EndOfTurn,
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(1), u()]),
                card_types: vec![CardType::Instant],
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(-6),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Down // Dirty — {3}{B} // {2}{G} Sorcery // Sorcery, Fuse. Down: target player
/// discards two cards. Dirty: return target card from your graveyard to hand.
pub fn down_dirty() -> CardDefinition {
    CardDefinition {
        name: "Down // Dirty",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
            random: false,
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(2), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Move {
                    what: target_filtered(R::InYourGraveyard),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            fuse: true,
            aftermath: false,
        })),
        ..Default::default()
    }
}
