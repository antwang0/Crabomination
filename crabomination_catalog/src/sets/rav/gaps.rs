//! Ravnica block (RAV/GPT) gap cards: the guild bounce-land cycle plus simple
//! creatures and spells filling the `set_gaps.py` remainder.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EquipScale, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest, ZoneRef};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

use super::super::etb_tap;

/// A Karoo bounce-land (CR — Ravnica block): enters tapped, returns a land you
/// control to hand on entry, and taps for two guild colors at once.
fn bounce_land(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![a, b]),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![
            etb_tap(),
            etb(Effect::Move {
                what: target_filtered(R::Land.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        ],
        ..Default::default()
    }
}

pub fn dimir_aqueduct() -> CardDefinition {
    bounce_land("Dimir Aqueduct", Color::Blue, Color::Black)
}
pub fn golgari_rot_farm() -> CardDefinition {
    bounce_land("Golgari Rot Farm", Color::Black, Color::Green)
}
pub fn selesnya_sanctuary() -> CardDefinition {
    bounce_land("Selesnya Sanctuary", Color::Green, Color::White)
}
pub fn boros_garrison() -> CardDefinition {
    bounce_land("Boros Garrison", Color::Red, Color::White)
}
pub fn gruul_turf() -> CardDefinition {
    bounce_land("Gruul Turf", Color::Red, Color::Green)
}
pub fn izzet_boilerworks() -> CardDefinition {
    bounce_land("Izzet Boilerworks", Color::Blue, Color::Red)
}
pub fn orzhov_basilica() -> CardDefinition {
    bounce_land("Orzhov Basilica", Color::White, Color::Black)
}

/// Benevolent Ancestor — {2}{W} 0/4 Spirit with Defender. `{T}: Prevent the
/// next 1 damage that would be dealt to any target this turn.`
pub fn benevolent_ancestor() -> CardDefinition {
    CardDefinition {
        name: "Benevolent Ancestor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage {
                target: target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Carrion Howler — {3}{B} 2/2 Zombie Wolf. `Pay 1 life: This creature gets
/// +2/-1 until end of turn.`
pub fn carrion_howler() -> CardDefinition {
    CardDefinition {
        name: "Carrion Howler",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wolf],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Conclave Phalanx — {4}{W} 2/4 Human Soldier with Convoke. When it enters,
/// you gain 1 life for each creature you control.
pub fn conclave_phalanx() -> CardDefinition {
    CardDefinition {
        name: "Conclave Phalanx",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::CreatureCountControlledBy(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Dogpile — {3}{R} Instant. Deals damage to any target equal to the number of
/// attacking creatures you control.
pub fn dogpile() -> CardDefinition {
    CardDefinition {
        name: "Dogpile",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_any(),
            amount: Value::count(Selector::EachPermanent(
                R::IsAttacking.and(R::ControlledByYou),
            )),
        },
        ..Default::default()
    }
}

/// Dimir Cutpurse — {1}{U}{B} 2/2 Spirit. Whenever it deals combat damage to a
/// player, that player discards a card and you draw a card.
pub fn dimir_cutpurse() -> CardDefinition {
    CardDefinition {
        name: "Dimir Cutpurse",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Clinging Darkness — {1}{B} Aura. Enchant creature. Enchanted creature gets
/// -4/-1.
pub fn clinging_darkness() -> CardDefinition {
    CardDefinition {
        name: "Clinging Darkness",
        cost: cost(&[generic(1), b()]),
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
            power: -4,
            toughness: -1,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Consult the Necrosages — {1}{U}{B} Sorcery. Choose one — target player draws
/// two cards; or target player discards two cards.
pub fn consult_the_necrosages() -> CardDefinition {
    CardDefinition {
        name: "Consult the Necrosages",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::Const(2),
                    random: false,
                },
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Caregiver — {W} 1/1 Human Cleric. `{W}, Sacrifice a creature: Prevent the
/// next 1 damage that would be dealt to any target this turn.`
pub fn caregiver() -> CardDefinition {
    CardDefinition {
        name: "Caregiver",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::PreventNextDamage {
                target: target_any(),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cerulean Sphinx — {4}{U}{U} 5/5 Sphinx with flying. `{U}: This creature's
/// owner shuffles it into their library.`
pub fn cerulean_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Cerulean Sphinx",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                    pos: crate::effect::LibraryPosition::Shuffled,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drooling Groodion — {3}{B}{B}{G} 4/3 Beast. `{2}{B}{G}, Sacrifice a creature:
/// Target creature gets +2/+2 until end of turn. Another target creature gets
/// -2/-2 until end of turn.`
pub fn drooling_groodion() -> CardDefinition {
    CardDefinition {
        name: "Drooling Groodion",
        cost: cost(&[generic(3), b(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), g()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::TargetFiltered {
                        slot: 1,
                        filter: R::Creature,
                    },
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dryad's Caress — {4}{G}{G} Instant. You gain 1 life for each creature on the
/// battlefield. If {W} was spent to cast this spell, untap all creatures you
/// control.
pub fn dryads_caress() -> CardDefinition {
    CardDefinition {
        name: "Dryad's Caress",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(R::Creature)),
            },
            Effect::If {
                cond: Predicate::ManaSpentOfColorAtLeast {
                    color: Color::White,
                    at_least: 1,
                },
                then: Box::new(Effect::Untap {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    up_to: None,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Empty the Catacombs — {3}{B} Sorcery. Each player returns all creature cards
/// from their graveyard to their hand.
pub fn empty_the_catacombs() -> CardDefinition {
    CardDefinition {
        name: "Empty the Catacombs",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachPlayer),
            body: Box::new(Effect::Move {
                what: Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::Triggerer),
                    filter: R::Creature,
                },
                to: ZoneDest::Hand(PlayerRef::Triggerer),
            }),
        },
        ..Default::default()
    }
}

/// Conclave's Blessing — {3}{W} Aura with Convoke. Enchant creature. Enchanted
/// creature gets +0/+2 for each other creature you control. (The self-exclusion
/// is approximated as all creatures you control.)
pub fn conclaves_blessing() -> CardDefinition {
    CardDefinition {
        name: "Conclave's Blessing",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Convoke],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::Creature.and(R::ControlledByYou),
                per_power: 0,
                per_toughness: 2,
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Autochthon Wurm — {10}{G}{G}{G}{W}{W} 9/14 Wurm with Convoke and Trample.
pub fn autochthon_wurm() -> CardDefinition {
    CardDefinition {
        name: "Autochthon Wurm",
        cost: cost(&[generic(10), g(), g(), g(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 9,
        toughness: 14,
        keywords: vec![Keyword::Convoke, Keyword::Trample],
        ..Default::default()
    }
}

/// Cackling Imp — {2}{B}{B} 2/2 Imp with flying. `{T}: Target player loses 1
/// life.`
pub fn cackling_imp() -> CardDefinition {
    CardDefinition {
        name: "Cackling Imp",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
