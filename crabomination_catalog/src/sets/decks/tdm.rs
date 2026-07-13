//! Tarkir: Dragonstorm (TDM) gap batch — simple commons/uncommons on existing
//! primitives: a deathtouch+indestructible combat trick (Alesha's Legacy), a
//! flash pump Aura granting first strike (Fire-Rim Form), a graveyard-hate
//! bottoming Construct (Jade-Cast Sentinel), and a dig-3-keep-top-rest-mill
//! body (Gurmag Nightwatch). Tests in `crabomination/src/tests/tdm.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, mono_hybrid, r, u, w, Color};

/// Alesha's Legacy — {1}{B} Instant. Target creature you control gains
/// deathtouch and indestructible until end of turn.
pub fn aleshas_legacy() -> CardDefinition {
    CardDefinition {
        name: "Alesha's Legacy",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Fire-Rim Form — {1}{R} Aura, Flash. Enchant creature. When it enters,
/// enchanted creature gains first strike until end of turn. Enchanted creature
/// gets +2/+0.
pub fn fire_rim_form() -> CardDefinition {
    CardDefinition {
        name: "Fire-Rim Form",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(crate::card::EquipBonus { power: 2, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            keyword: Keyword::FirstStrike,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Jade-Cast Sentinel — {4} Artifact Creature — Ape Snake 1/5, Reach. {2}, {T}:
/// Put target card from a graveyard on the bottom of its owner's library.
pub fn jade_cast_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Jade-Cast Sentinel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ape, CreatureType::Snake],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Move {
                what: target_filtered(R::InGraveyard),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::Bottom,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gurmag Nightwatch — {2/B}{2/G}{2/U} 3/3 Human Ranger. When it enters, look at
/// the top three cards of your library, put one back on top, the rest into your
/// graveyard.
pub fn gurmag_nightwatch() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Nightwatch",
        cost: cost(&[
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
            mono_hybrid(2, Color::Blue),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ranger],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::LookTopKeepOneRestToGraveyard {
            count: Value::Const(3),
            who: None,
            exile_rest: false,
        })],
        ..Default::default()
    }
}

/// Kin-Tree Severance — {2/W}{2/B}{2/G} Instant. Exile target permanent with
/// mana value 3 or greater.
pub fn kin_tree_severance() -> CardDefinition {
    CardDefinition {
        name: "Kin-Tree Severance",
        cost: cost(&[
            mono_hybrid(2, Color::White),
            mono_hybrid(2, Color::Black),
            mono_hybrid(2, Color::Green),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: target_filtered(R::Permanent.and(R::ManaValueAtLeast(3))),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

/// Armament Dragon — {3}{W}{B}{G} 3/4 Dragon, Flying. When it enters, distribute
/// three +1/+1 counters among one, two, or three target creatures you control.
pub fn armament_dragon() -> CardDefinition {
    CardDefinition {
        name: "Armament Dragon",
        cost: cost(&[generic(3), w(), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::DistributeCounters {
            total: Value::Const(3),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature.and(R::ControlledByYou),
            max_targets: 3,
        })],
        ..Default::default()
    }
}

/// Fresh Start — {1}{U} Aura, Flash. Enchant creature. Enchanted creature gets
/// −5/−0 and loses all abilities.
pub fn fresh_start() -> CardDefinition {
    CardDefinition {
        name: "Fresh Start",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: -5,
            remove_abilities: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Lie in Wait — {B}{G}{U} Sorcery. Return target creature card from your
/// graveyard to your hand. It deals damage equal to that card's power to target
/// creature.
pub fn lie_in_wait() -> CardDefinition {
    CardDefinition {
        name: "Lie in Wait",
        cost: cost(&[b(), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Dragonstorm Globe — {3} Artifact. Each Dragon you control enters with an
/// additional +1/+1 counter on it. {T}: Add one mana of any color.
pub fn dragonstorm_globe() -> CardDefinition {
    CardDefinition {
        name: "Dragonstorm Globe",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each Dragon you control enters with an additional +1/+1 counter on it.",
            effect: StaticEffect::TypeEntersWithCounter {
                creature_type: CreatureType::Dragon,
                kind: CounterType::PlusOnePlusOne,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyColors(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wingspan Stride — {U} Aura. Enchant creature. Enchanted creature gets +1/+1
/// and has flying. {2}{U}: Return this Aura to its owner's hand.
pub fn wingspan_stride() -> CardDefinition {
    CardDefinition {
        name: "Wingspan Stride",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Riverwalk Technique — {3}{U} Instant. Choose one — the owner of target nonland
/// permanent puts it on their choice of the top or bottom of their library; or
/// counter target noncreature spell.
pub fn riverwalk_technique() -> CardDefinition {
    CardDefinition {
        name: "Riverwalk Technique",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: R::Nonland },
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: LibraryPosition::OwnerChoice,
                },
            },
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.negate())),
            },
        ]),
        ..Default::default()
    }
}
