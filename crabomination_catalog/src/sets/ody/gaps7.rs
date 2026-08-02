//! Odyssey (ODY) gap-closing wave 7: the Egg and Threshold-land cycles, the
//! artifact utility and the legends. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, target_filtered},
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn land(name: &'static str) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], ..Default::default() }
}

fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

// ── The Egg cycle ───────────────────────────────────────────────────────────

/// `{2}, {T}, Sacrifice this: Add two coloured mana. Draw a card.`
fn egg(name: &'static str, colors: [Color; 2]) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![colors[0]], Value::ONE),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![colors[1]], Value::ONE),
                },
                draw(1),
            ]),
            ..Default::default()
        }],
        ..artifact(name, cost(&[generic(1)]))
    }
}

pub fn skycloud_egg() -> CardDefinition {
    egg("Skycloud Egg", [Color::White, Color::Blue])
}
pub fn darkwater_egg() -> CardDefinition {
    egg("Darkwater Egg", [Color::Blue, Color::Black])
}
pub fn shadowblood_egg() -> CardDefinition {
    egg("Shadowblood Egg", [Color::Black, Color::Red])
}
pub fn mossfire_egg() -> CardDefinition {
    egg("Mossfire Egg", [Color::Red, Color::Green])
}

// ── The Threshold pain-land cycle ───────────────────────────────────────────

/// `{T}: Add [c]. This land deals 1 damage to you.` plus a Threshold sac
/// ability.
fn threshold_land(name: &'static str, color: Color, sac: ActivatedAbility) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::OfColors(vec![color], Value::ONE),
                    },
                    Effect::DealDamage { to: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            sac,
        ],
        ..land(name)
    }
}

pub fn nomad_stadium() -> CardDefinition {
    threshold_land(
        "Nomad Stadium",
        Color::White,
        ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            sac_cost: true,
            condition: Some(threshold()),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ..Default::default()
        },
    )
}

pub fn cabal_pit() -> CardDefinition {
    threshold_land(
        "Cabal Pit",
        Color::Black,
        ActivatedAbility {
            mana_cost: cost(&[b()]),
            tap_cost: true,
            sac_cost: true,
            condition: Some(threshold()),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        },
    )
}

pub fn centaur_garden() -> CardDefinition {
    threshold_land(
        "Centaur Garden",
        Color::Green,
        ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_cost: true,
            condition: Some(threshold()),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        },
    )
}

// ── Other lands ─────────────────────────────────────────────────────────────

/// The Odyssey filter-land cycle: `{1}, {T}: Add two coloured mana.`
fn filter_land(name: &'static str, colors: [Color; 2]) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![colors[0]], Value::ONE),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![colors[1]], Value::ONE),
                },
            ]),
            ..Default::default()
        }],
        ..land(name)
    }
}

pub fn sungrass_prairie() -> CardDefinition {
    filter_land("Sungrass Prairie", [Color::Green, Color::White])
}
pub fn skycloud_expanse() -> CardDefinition {
    filter_land("Skycloud Expanse", [Color::White, Color::Blue])
}
pub fn darkwater_catacombs() -> CardDefinition {
    filter_land("Darkwater Catacombs", [Color::Blue, Color::Black])
}
pub fn shadowblood_ridge() -> CardDefinition {
    filter_land("Shadowblood Ridge", [Color::Black, Color::Red])
}
pub fn mossfire_valley() -> CardDefinition {
    filter_land("Mossfire Valley", [Color::Red, Color::Green])
}

/// Crystal Quarry — {T} for {C}, or {5} for all five colours.
pub fn crystal_quarry() -> CardDefinition {
    CardDefinition {
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
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![
                        Color::White,
                        Color::Blue,
                        Color::Black,
                        Color::Red,
                        Color::Green,
                    ]),
                },
                ..Default::default()
            },
        ],
        ..land("Crystal Quarry")
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Steamclaw — {2} graveyard hate that can cash itself in.
pub fn steamclaw() -> CardDefinition {
    let exile = || Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::Exile };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: exile(),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                effect: exile(),
                ..Default::default()
            },
        ],
        ..artifact("Steamclaw", cost(&[generic(2)]))
    }
}

/// Sandstone Deadfall — {3} that eats two lands and an attacker.
pub fn sandstone_deadfall() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            sac_other_filter: Some((R::Land, 2)),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
            },
            ..Default::default()
        }],
        ..artifact("Sandstone Deadfall", cost(&[generic(3)]))
    }
}

/// Otarian Juggernaut — {4} 2/3 that Walls can't stop and that goes berserk
/// past Threshold.
pub fn otarian_juggernaut() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBeBlockedByCreatureType(CreatureType::Wall)],
        static_abilities: vec![StaticAbility {
            description: "Threshold — +3/+0 and attacks each combat if able.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 3,
                toughness: 0,
                keywords: vec![Keyword::MustAttack],
            },
        }],
        ..creature(
            "Otarian Juggernaut",
            cost(&[generic(4)]),
            vec![CreatureType::Juggernaut],
            2,
            3,
        )
    }
}

// ── Legends ─────────────────────────────────────────────────────────────────

/// Braids, Cabal Minion — {2}{B}{B} 2/2 that taxes every upkeep.
pub fn braids_cabal_minion() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::Artifact.or(R::Creature).or(R::Land),
            },
        }],
        ..creature(
            "Braids, Cabal Minion",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// Lieutenant Kirtar — {1}{W}{W} 2/2 flier that exiles an attacker.
pub fn lieutenant_kirtar() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            sac_cost: true,
            effect: Effect::Exile {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
            },
            ..Default::default()
        }],
        ..creature(
            "Lieutenant Kirtar",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Pianna, Nomad Captain — {1}{W}{W} 2/2 that pumps the whole swing.
pub fn pianna_nomad_captain() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Pianna, Nomad Captain",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            2,
            2,
        )
    }
}

/// Seton, Krosan Protector — {G}{G}{G} 2/2 that taps Druids for mana.
pub fn seton_krosan_protector() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature.and(R::HasCreatureType(CreatureType::Druid))),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Green], Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Seton, Krosan Protector",
            cost(&[g(), g(), g()]),
            vec![CreatureType::Centaur, CreatureType::Druid],
            2,
            2,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Beast Attack — {2}{G}{G}{G} an instant-speed 4/4, twice.
pub fn beast_attack() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(2), g(), g(), g()]))],
        ..instant(
            "Beast Attack",
            cost(&[generic(2), g(), g(), g()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Beast".into(),
                    power: 4,
                    toughness: 4,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Beast],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        )
    }
}

/// Bash to Bits — {3}{R} artifact removal with flashback.
pub fn bash_to_bits() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r(), r()]))],
        ..instant(
            "Bash to Bits",
            cost(&[generic(3), r()]),
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Divert — {U} repoints a single-target spell unless its caster pays {2}.
pub fn divert() -> CardDefinition {
    instant(
        "Divert",
        cost(&[u()]),
        Effect::UnlessPlayerPays {
            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
            cost: crate::card::WardCost::Mana(cost(&[generic(2)])),
            then: Box::new(Effect::ChooseNewTargetsForSpell { what: Selector::Target(0) }),
            if_paid: None,
        },
    )
}
