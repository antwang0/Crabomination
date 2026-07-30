//! Guildpact (GPT) gap wave 5: two guildhall utility lands, a pair of red
//! spells (uncounterable Naya destruction, spell-MV burn), a green fatty Aura,
//! and Quicken's sorcery-flash cantrip. Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    Keyword, SelectionRequirement as R, Subtypes, TokenDefinition, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

fn wurm_token() -> TokenDefinition {
    TokenDefinition {
        name: "Wurm".into(),
        power: 6,
        toughness: 6,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Skarrg, the Rage Pits — Land. {T}: Add {C}. {R}{G}, {T}: Target creature gets
/// +1/+1 and gains trample until end of turn.
pub fn skarrg_the_rage_pits() -> CardDefinition {
    CardDefinition {
        name: "Skarrg, the Rage Pits",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[r(), g()]),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(R::Creature),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Orzhova, the Church of Deals — Land. {T}: Add {C}. {3}{W}{B}, {T}: Target
/// player loses 1 life and you gain 1 life.
pub fn orzhova_the_church_of_deals() -> CardDefinition {
    CardDefinition {
        name: "Orzhova, the Church of Deals",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), w(), b()]),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::Target(0)),
                    to: Selector::Player(PlayerRef::You),
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wreak Havoc — {2}{R}{G} Sorcery. This spell can't be countered. Destroy
/// target artifact or land.
pub fn wreak_havoc() -> CardDefinition {
    CardDefinition {
        name: "Wreak Havoc",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Destroy {
            what: target_filtered(R::Artifact.or(R::Land)),
        },
        ..Default::default()
    }
}

/// Parallectric Feedback — {3}{R} Instant. Deals damage to target spell's
/// controller equal to that spell's mana value.
pub fn parallectric_feedback() -> CardDefinition {
    CardDefinition {
        name: "Parallectric Feedback",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
            amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
        },
        ..Default::default()
    }
}

/// Quicken — {U} Instant. The next sorcery spell you cast this turn can be cast
/// as though it had flash. Draw a card. (Approximated as all your sorceries
/// gaining flash until end of turn.)
pub fn quicken() -> CardDefinition {
    CardDefinition {
        name: "Quicken",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantSorceriesAsFlash {
                who: PlayerRef::You,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Wurmweaver Coil — {4}{G}{G} Aura. Enchant green creature. Enchanted creature
/// gets +6/+6. {G}{G}{G}, Sacrifice this Aura: Create a 6/6 green Wurm.
pub fn wurmweaver_coil() -> CardDefinition {
    CardDefinition {
        name: "Wurmweaver Coil",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::HasColor(Color::Green))),
        },
        equipped_bonus: Some(EquipBonus {
            power: 6,
            toughness: 6,
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            mana_cost: cost(&[g(), g(), g()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: wurm_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
