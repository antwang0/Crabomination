//! More utility lands: charge-counter detonation (Blast Zone, reusing
//! `Effect::DestroyEachNonlandWithManaValue`), land destruction, graveyard
//! recursion, and devotion-style mana doublers. Tests in `tests/recent43.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, Effect, SelectionRequirement as R, Selector,
    Value, Zone,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{ActivatedAbility, LibraryPosition, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, cost, generic, u, x};

fn tap_colorless() -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(1)) },
        ..Default::default()
    }
}

/// Blast Zone — enters with a charge counter. `{T}: Add {C}.`
/// `{X}{X}, {T}: Put X charge counters.` `{3}, {T}, Sacrifice: Destroy each
/// nonland permanent with mana value equal to its charge counters.`
pub fn blast_zone() -> CardDefinition {
    let charges = Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge };
    CardDefinition {
        name: "Blast Zone",
        card_types: vec![CardType::Land],
        enters_with_counters: Some((CounterType::Charge, Value::Const(1))),
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[x(), x()]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::XFromCost,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sac_cost: true,
                effect: Effect::DestroyEachNonlandWithManaValue { value: charges },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Encroaching Wastes — `{T}: Add {C}.` `{4}, {T}, Sacrifice: Destroy target
/// nonbasic land.`
pub fn encroaching_wastes() -> CardDefinition {
    CardDefinition {
        name: "Encroaching Wastes",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(4)]),
                sac_cost: true,
                effect: Effect::Destroy {
                    what: target_filtered(R::Land.and(R::IsBasicLand.negate())),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Dust Bowl — `{T}: Add {C}.` `{3}, {T}, Sacrifice a land: Destroy target
/// nonbasic land.`
pub fn dust_bowl() -> CardDefinition {
    CardDefinition {
        name: "Dust Bowl",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sac_other_filter: Some((R::Land, 1)),
                effect: Effect::Destroy {
                    what: target_filtered(R::Land.and(R::IsBasicLand.negate())),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Tectonic Edge — `{T}: Add {C}.` `{1}, {T}, Sacrifice: Destroy target
/// nonbasic land. Activate only if an opponent controls four or more lands.`
pub fn tectonic_edge() -> CardDefinition {
    CardDefinition {
        name: "Tectonic Edge",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                sac_cost: true,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land.and(R::ControlledByOpponent)),
                    n: Value::Const(4),
                }),
                effect: Effect::Destroy {
                    what: target_filtered(R::Land.and(R::IsBasicLand.negate())),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Buried Ruin — `{T}: Add {C}.` `{2}, {T}, Sacrifice: Return target artifact
/// card from your graveyard to your hand.`
pub fn buried_ruin() -> CardDefinition {
    CardDefinition {
        name: "Buried Ruin",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::InGraveyard.and(R::Artifact)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Academy Ruins — `{T}: Add {C}.` `{1}{U}, {T}: Put target artifact card from
/// your graveyard on top of your library.`
pub fn academy_ruins() -> CardDefinition {
    CardDefinition {
        name: "Academy Ruins",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), u()]),
                effect: Effect::Move {
                    what: target_filtered(R::InGraveyard.and(R::Artifact)),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Petrified Field — `{T}: Add {C}.` `{T}, Sacrifice: Return target land card
/// from your graveyard to your hand.`
pub fn petrified_field() -> CardDefinition {
    CardDefinition {
        name: "Petrified Field",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Land },
                        Value::Const(1),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Serra's Sanctum — Legendary Land. `{T}: Add {W} for each enchantment you
/// control.`
pub fn serras_sanctum() -> CardDefinition {
    CardDefinition {
        name: "Serra's Sanctum",
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::White, Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
                    filter: R::Enchantment,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Tolarian Academy — Legendary Land. `{T}: Add {U} for each artifact you
/// control.`
pub fn tolarian_academy() -> CardDefinition {
    CardDefinition {
        name: "Tolarian Academy",
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Blue, Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
                    filter: R::Artifact,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
