//! Commander: three-card gaps for Cynette, Elda and Ekthi (`cmdr_edhrec36`
//! is the previous batch of this run; Meneldor is `cmdr_edhrec31`'s). Tests in
//! `tests/recent_b/cmdr_edhrec37.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EquipScale, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies};
use crate::effect::{Effect, PlayerRef};
use crate::mana::{cost, g, generic, u, w, Color, ManaCost};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn equipment(name: &'static str, mana: ManaCost, equip: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(equip)]))],
        ..Default::default()
    }
}

fn equipment_filter() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Equipment)
}

// ── Cynette, Jelly Drover ────────────────────────────────────────────────────

/// Cynette, Jelly Drover — {3}{U} legendary 2/2 Human Wizard. Enters or dies:
/// a 1/1 blue flying Jellyfish. Your fliers get +1/+1.
pub fn cynette_jelly_drover() -> CardDefinition {
    let jelly = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Jellyfish".into(),
            power: 1,
            toughness: 1,
            colors: vec![Color::Blue],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Jellyfish], ..Default::default() },
            keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
    };
    CardDefinition {
        triggered_abilities: vec![etb(jelly()), on_dies(jelly())],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with flying get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..legend(
            "Cynette, Jelly Drover",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Naban, Dean of Iteration — {1}{U} legendary 2/1 Human Wizard. A Wizard
/// you control entering triggers your permanents' abilities an additional
/// time.
pub fn naban_dean_of_iteration() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a Wizard you control entering causes a triggered ability of a permanent you control \
                          to trigger, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerEtbTriggers {
                entering: R::HasCreatureType(CreatureType::Wizard).and(R::ControlledByYou),
            },
        }],
        ..legend(
            "Naban, Dean of Iteration",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            1,
        )
    }
}

// ── Elda, Conjurer of Spectacle ──────────────────────────────────────────────

/// Elda, Conjurer of Spectacle — {2}{G}{W} legendary 2/2 Human Wizard. The
/// first nonlegendary creature spell you cast each turn has demonstrate.
pub fn elda_conjurer_of_spectacle() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellFirstMatchingThisTurn(
                    R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
                )),
            effect: Effect::Demonstrate,
        }],
        ..legend(
            "Elda, Conjurer of Spectacle",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Fungus Sliver — {3}{G} 2/2 Fungus Sliver. All Slivers have "whenever this
/// creature is dealt damage, put a +1/+1 counter on it."
pub fn fungus_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Sliver creatures have \"Whenever this creature is dealt damage, put a +1/+1 counter \
                          on it.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Sliver)),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
                    effect: Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                }),
            },
        }],
        ..creature(
            "Fungus Sliver",
            cost(&[generic(3), g()]),
            vec![CreatureType::Fungus, CreatureType::Sliver],
            2,
            2,
        )
    }
}

/// Reflex Sliver — {3}{G} 2/2 Sliver. All Slivers have haste.
pub fn reflex_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Sliver creatures have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Sliver))),
                keyword: Keyword::Haste,
            },
        }],
        ..creature("Reflex Sliver", cost(&[generic(3), g()]), vec![CreatureType::Sliver], 2, 2)
    }
}

// ── Ekthi, Contaminator Priest ───────────────────────────────────────────────

/// Ekthi, Contaminator Priest — {3}{W} legendary 3/3 Phyrexian Cleric. Other
/// Phyrexians you control get +1/+1; each Equipment you control has living
/// weapon (CR 702.92).
pub fn ekthi_contaminator_priest() -> CardDefinition {
    let germ = TokenDefinition {
        name: "Phyrexian Germ".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    };
    let living_weapon = TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(germ) },
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]),
    };
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Phyrexians you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::Creature
                            .and(R::HasCreatureType(CreatureType::Phyrexian))
                            .and(R::ControlledByYou)
                            .and(R::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Each Equipment you control has living weapon.",
                effect: StaticEffect::GrantTriggeredAbility {
                    filter: equipment_filter().and(R::ControlledByYou),
                    ability: Box::new(living_weapon),
                },
            },
        ],
        ..legend(
            "Ekthi, Contaminator Priest",
            cost(&[generic(3), w()]),
            vec![CreatureType::Phyrexian, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Long-Lost Lances — {2} Equipment. Equipped creature gets +2/+0. During
/// your turn, your equipped creatures have first strike and vigilance. Equip
/// {2}.
pub fn long_lost_lances() -> CardDefinition {
    let during_your_turn = |keyword| StaticAbility {
        description: "During your turn, creatures you control that are equipped have first strike and vigilance.",
        effect: StaticEffect::WhileYourTurn {
            inner: Box::new(StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::IsEquipped).and(R::ControlledByYou)),
                keyword,
            }),
        },
    };
    CardDefinition {
        equipped_bonus: Some(EquipBonus { power: 2, ..Default::default() }),
        static_abilities: vec![during_your_turn(Keyword::FirstStrike), during_your_turn(Keyword::Vigilance)],
        ..equipment("Long-Lost Lances", cost(&[generic(2)]), 2)
    }
}

/// Swordsman's Steel — {4} Equipment. Enters: draw a card per Equipment you
/// control. Equipped creature gets +2/+2 per Equipment you control. Equip {3}.
pub fn swordsmans_steel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::count(Selector::ControlledBy { who: PlayerRef::You, filter: equipment_filter() }),
        })],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale { filter: equipment_filter(), per_power: 2, per_toughness: 2, ..Default::default() }),
            ..Default::default()
        }),
        ..equipment("Swordsman's Steel", cost(&[generic(4)]), 3)
    }
}
