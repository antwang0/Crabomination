//! Strixhaven base-set (STX) cards — second batch of missing printed
//! cards: Equipment, conditional-keyword and sacrifice-payoff creatures,
//! and a graveyard-fueled evasion granter. Wired against existing
//! primitives; each ships with a test in `crate::tests::stx`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, Effect,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, Selector,
    SelectionRequirement, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{add_mana, each_opponent, etb, target, target_filtered};
use crate::effect::{Duration, PlayerRef, StaticAbility, StaticEffect};
use crate::mana::{b, cost, generic, r, u, w, Color};

// ── Equipment ───────────────────────────────────────────────────────────────

/// Poet's Quill — {1}{B} Equipment. ETB: Learn. Equipped creature gets
/// +1/+1 and has lifelink. Equip {1}{B}.
pub fn poets_quill() -> CardDefinition {
    CardDefinition {
        name: "Poet's Quill",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1), b()]))],
        triggered_abilities: vec![etb(Effect::Learn { who: PlayerRef::You })],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Lifelink],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Team Pennant — {1} Equipment. Equipped creature gets +1/+1 and has
/// vigilance and trample. Equip {3}. (The cheaper "Equip creature token
/// {1}" clause collapses to the flat equip cost.)
pub fn team_pennant() -> CardDefinition {
    CardDefinition {
        name: "Team Pennant",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance, Keyword::Trample],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Zephyr Boots — {1} Equipment. Equipped creature has flying. Whenever it
/// deals combat damage to a player, draw a card, then discard a card.
/// Equip {2}.
pub fn zephyr_boots() -> CardDefinition {
    CardDefinition {
        name: "Zephyr Boots",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 0,
            keywords: vec![Keyword::Flying],
            scale: None,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                ]),
            }],
        }),
        ..Default::default()
    }
}

// ── Creatures ─────────────────────────────────────────────────────────────

/// Leech Fanatic — {1}{B} 2/2 Human Warlock. During your turn, it has
/// lifelink.
pub fn leech_fanatic() -> CardDefinition {
    CardDefinition {
        name: "Leech Fanatic",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "During your turn, Leech Fanatic has lifelink.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::IsTurnOf(PlayerRef::You),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Lifelink],
            },
        }],
        ..Default::default()
    }
}

/// Stonerise Spirit — {1}{W} 1/2 Spirit Bird with flying. `{4}, Exile a
/// card from your graveyard: Target creature gains flying until end of turn.`
pub fn stonerise_spirit() -> CardDefinition {
    CardDefinition {
        name: "Stonerise Spirit",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            exile_other_filter: Some((SelectionRequirement::Any, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Novice Dissector — {3}{B} 3/3 Troll Warlock. `{1}, Sacrifice another
/// creature: Put a +1/+1 counter on target creature. Activate only as a
/// sorcery.`
pub fn novice_dissector() -> CardDefinition {
    CardDefinition {
        name: "Novice Dissector",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            sac_other_filter: Some((
                SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                1,
            )),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blood Age General — {1}{R} 2/2 Spirit Warrior. `{T}: Attacking Spirits
/// get +1/+0 until end of turn.`
pub fn blood_age_general() -> CardDefinition {
    CardDefinition {
        name: "Blood Age General",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                        .and(SelectionRequirement::IsAttacking),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}


// ── Spells & more creatures ─────────────────────────────────────────────────

/// Go Blank — {2}{B} Sorcery. Target player discards two cards, then exile
/// that player's graveyard.
pub fn go_blank() -> CardDefinition {
    CardDefinition {
        name: "Go Blank",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
                random: false,
            },
            Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Secret Rendezvous — {1}{W}{W} Sorcery. You and target opponent each draw
/// three cards.
pub fn secret_rendezvous() -> CardDefinition {
    CardDefinition {
        name: "Secret Rendezvous",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Fuming Effigy — {3}{R} 4/3 Spirit. Whenever one or more cards leave your
/// graveyard, it deals 1 damage to each opponent.
pub fn fuming_effigy() -> CardDefinition {
    CardDefinition {
        name: "Fuming Effigy",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::DealDamage { to: each_opponent(), amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Kelpie Guide — {2}{U} 2/2 Beast. `{T}: Untap another target permanent you
/// control.` `{T}: Tap target permanent. Activate only if you control eight
/// or more lands.`
pub fn kelpie_guide() -> CardDefinition {
    CardDefinition {
        name: "Kelpie Guide",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap {
                    what: target_filtered(
                        SelectionRequirement::ControlledByYou
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    up_to: None,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    ),
                    n: Value::Const(8),
                }),
                effect: Effect::Tap { what: target() },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Explosive Welcome — {7}{R} Instant. Deals 5 damage to any target and 3
/// damage to any other target. Add {R}{R}{R}.
pub fn explosive_welcome() -> CardDefinition {
    CardDefinition {
        name: "Explosive Welcome",
        cost: cost(&[generic(7), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target(), amount: Value::Const(5) },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Any },
                amount: Value::Const(3),
            },
            add_mana(vec![Color::Red, Color::Red, Color::Red]),
        ]),
        ..Default::default()
    }
}
