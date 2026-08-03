//! Ravnica (RAV) gap wave 7: a spread of simple activated/triggered creatures
//! and spells that reuse existing primitives. Tests in `classic_sets/rav`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

/// Votary of the Conclave — {W} 1/1 Human Soldier. {2}{G}: Regenerate this.
pub fn votary_of_the_conclave() -> CardDefinition {
    CardDefinition {
        name: "Votary of the Conclave",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Torpid Moloch — {R} 3/2 Lizard with defender. Sacrifice three lands: it loses
/// defender until end of turn.
pub fn torpid_moloch() -> CardDefinition {
    CardDefinition {
        name: "Torpid Moloch",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land, 3)),
            effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                what: Selector::This,
                keyword: Keyword::Defender,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Psychic Drain — {X}{U}{B} Sorcery. Target player mills X cards and you gain X
/// life.
pub fn psychic_drain() -> CardDefinition {
    CardDefinition {
        name: "Psychic Drain",
        cost: cost(&[x(), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}

/// Rolling Spoil — {2}{G}{G} Sorcery. Destroy target land. If {B} was spent to
/// cast this, all creatures get -1/-1 until end of turn.
pub fn rolling_spoil() -> CardDefinition {
    CardDefinition {
        name: "Rolling Spoil",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Land),
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast {
                    color: Color::Black,
                    at_least: 1,
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Quickchange — {1}{U} Instant. Target creature becomes the color or colors of
/// your choice until end of turn. Draw a card.
pub fn quickchange() -> CardDefinition {
    CardDefinition {
        name: "Quickchange",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeChosenColor {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Ursapine — {3}{G}{G} 3/3 Beast. {G}: target creature gets +1/+1 until end of
/// turn.
pub fn ursapine() -> CardDefinition {
    CardDefinition {
        name: "Ursapine",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tidewater Minion — {3}{U}{U} 4/4 Elemental Minion with defender. {4}: it
/// loses defender until end of turn. {T}: untap target permanent.
pub fn tidewater_minion() -> CardDefinition {
    CardDefinition {
        name: "Tidewater Minion",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Minion],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::LoseKeyword { duration: Duration::EndOfTurn,
                    what: Selector::This,
                    keyword: Keyword::Defender,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap {
                    what: target_filtered(R::Permanent),
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Twisted Justice — {4}{U}{B} Sorcery. Target player sacrifices a creature of
/// their choice. You draw cards equal to that creature's power.
pub fn twisted_justice() -> CardDefinition {
    CardDefinition {
        name: "Twisted Justice",
        cost: cost(&[generic(4), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::Target(0),
                filter: R::Creature,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::SacrificedPower,
            },
        ]),
        ..Default::default()
    }
}

/// Strands of Undeath — {3}{B} Aura. Enchant creature. When it enters, target
/// player discards two cards. {B}: Regenerate enchanted creature.
pub fn strands_of_undeath() -> CardDefinition {
    CardDefinition {
        name: "Strands of Undeath",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        triggered_abilities: vec![etb(Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
            random: false,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wizened Snitches — {3}{U} 1/3 Faerie Rogue with flying. Players play with the
/// top card of their libraries revealed.
pub fn wizened_snitches() -> CardDefinition {
    CardDefinition {
        name: "Wizened Snitches",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Players play with the top card of their libraries revealed.",
            effect: StaticEffect::AllLibraryTopsRevealed,
        }],
        ..Default::default()
    }
}
