//! Odyssey (ODY) gap-closing wave 4: the land-sacrifice red shell and the last
//! blue utility. Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
    Zone,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, target_any, target_filtered},
};
use crate::mana::{ManaCost, cost, generic, r, u};

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

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Kamahl, Pit Fighter — {4}{R}{R} 6/1 hasty machine gun.
pub fn kamahl_pit_fighter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature(
            "Kamahl, Pit Fighter",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            6,
            1,
        )
    }
}

/// Dwarven Strike Force — {4}{R} 4/3 that pitches at random for a fast swing.
pub fn dwarven_strike_force() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Dwarven Strike Force",
            cost(&[generic(4), r()]),
            vec![CreatureType::Dwarf, CreatureType::Berserker],
            4,
            3,
        )
    }
}

/// Frenetic Ogre — {4}{R} 2/3 that pitches at random for power.
pub fn frenetic_ogre() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Frenetic Ogre", cost(&[generic(4), r()]), vec![CreatureType::Ogre], 2, 3)
    }
}

/// Magma Vein — {2}{R} turns spare lands into a ground sweep.
pub fn magma_vein() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Magma Vein", cost(&[generic(2), r()]))
    }
}

/// Need for Speed — {R} turns spare lands into haste.
pub fn need_for_speed() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Need for Speed", cost(&[r()]))
    }
}

/// Battle Strain — {1}{R} pings the defender for every block.
pub fn battle_strain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..enchantment("Battle Strain", cost(&[generic(1), r()]))
    }
}

/// Burning Sands — {3}{R}{R} taxes every death a land.
pub fn burning_sands() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                count: Value::ONE,
                filter: R::Land,
            },
        }],
        ..enchantment("Burning Sands", cost(&[generic(3), r(), r()]))
    }
}

/// Epicenter — {4}{R} one land, or every land past Threshold.
pub fn epicenter() -> CardDefinition {
    sorcery(
        "Epicenter",
        cost(&[generic(4), r()]),
        Effect::If {
            cond: Predicate::ThresholdActive { who: PlayerRef::You },
            then: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(99),
                filter: R::Land,
            }),
            else_: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Land,
            }),
        },
    )
}

/// Mudhole — {2}{R} strips a graveyard of its lands.
pub fn mudhole() -> CardDefinition {
    instant(
        "Mudhole",
        cost(&[generic(2), r()]),
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::Target(0),
                zone: Zone::Graveyard,
                filter: R::Land,
            },
            to: ZoneDest::Exile,
        },
    )
}

/// Volley of Boulders — {8}{R} six damage spread as you like, twice.
pub fn volley_of_boulders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[r(), r(), r(), r(), r(), r()]))],
        ..sorcery(
            "Volley of Boulders",
            cost(&[generic(8), r()]),
            Effect::DealDamageDivided {
                total: Value::Const(6),
                filter: R::Creature.or(R::Player).or(R::Planeswalker),
                max_targets: 6,
                retaliate_to_source: false,
            },
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Touch of Invisibility — {3}{U} an unblockable turn plus a card.
pub fn touch_of_invisibility() -> CardDefinition {
    sorcery(
        "Touch of Invisibility",
        cost(&[generic(3), u()]),
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Dematerialize — {3}{U} bounce anything, twice.
pub fn dematerialize() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), u(), u()]))],
        ..sorcery(
            "Dematerialize",
            cost(&[generic(3), u()]),
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )
    }
}

/// Laquatus's Creativity — {4}{U} a full hand swap.
pub fn laquatuss_creativity() -> CardDefinition {
    sorcery(
        "Laquatus's Creativity",
        cost(&[generic(4), u()]),
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::CardsInHandMatching {
                    who: PlayerRef::Target(0),
                    filter: R::Any,
                },
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::CardsDrawnThisEffect,
                random: false,
            },
        ]),
    )
}
