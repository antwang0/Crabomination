//! Ravnica batch 2: a spread of guild commons/uncommons that reuse existing
//! primitives — Convoke, Landfall (`EventKind::LandPlayed`),
//! `EventKind::AuraAttached`, protection from monocolored (new
//! `Keyword::ProtectionFromMonocolored`), `Effect::Regenerate`, and the
//! `BasePlusPerAttachedAura` dynamic P/T. Tests in `recent_b/recent_292`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, DynamicPt, Effect,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R,
    Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// A vanilla 1/1 green Saproling token.
fn saproling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Saproling".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Saproling],
            ..Default::default()
        },
        ..Default::default()
    }
}

// ── Guild creatures & spells ────────────────────────────────────────────────

/// Siege Wurm — {5}{G}{G} 5/5 Wurm with Convoke and trample.
pub fn siege_wurm() -> CardDefinition {
    CardDefinition {
        name: "Siege Wurm",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Convoke, Keyword::Trample],
        ..Default::default()
    }
}

/// Nightguard Patrol — {2}{W} 2/1 Human Soldier with first strike and vigilance.
pub fn nightguard_patrol() -> CardDefinition {
    CardDefinition {
        name: "Nightguard Patrol",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        ..Default::default()
    }
}

/// Guardian of the Guildpact — {3}{W} 2/3 Spirit with protection from
/// monocolored (`Keyword::ProtectionFromMonocolored`).
pub fn guardian_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Guardian of the Guildpact",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::ProtectionFromMonocolored],
        ..Default::default()
    }
}

/// Ghost Warden — {1}{W} 1/1 Spirit. {T}: target creature gets +1/+1 until EOT.
pub fn ghost_warden() -> CardDefinition {
    CardDefinition {
        name: "Ghost Warden",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
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

/// Selesnya Evangel — {G}{W} 1/2 Elf Shaman. {1}, {T}, Tap an untapped creature
/// you control: create a 1/1 Saproling.
pub fn selesnya_evangel() -> CardDefinition {
    CardDefinition {
        name: "Selesnya Evangel",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            tap_other_filter: Some(R::Creature),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: saproling_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fists of the Anvil — {1}{R} Instant. Target creature gets +4/+0 until EOT.
pub fn fists_of_the_anvil() -> CardDefinition {
    CardDefinition {
        name: "Fists of the Anvil",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(4),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Fiery Conclusion — {1}{R} Instant. Sacrifice a creature as an additional
/// cost (modeled at resolution, per the Fling idiom); deal 5 to target creature.
pub fn fiery_conclusion() -> CardDefinition {
    CardDefinition {
        name: "Fiery Conclusion",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: R::Creature,
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::Const(5),
            },
        ]),
        ..Default::default()
    }
}

/// Gatherer of Graces — {1}{G} 1/2 Human Druid. +1/+1 for each Aura attached to
/// it; Sacrifice an Aura: regenerate it.
pub fn gatherer_of_graces() -> CardDefinition {
    CardDefinition {
        name: "Gatherer of Graces",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        dynamic_pt: Some(DynamicPt::BasePlusPerAttachedAura {
            base_p: 1,
            base_t: 2,
            per: 1,
        }),
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasEnchantmentSubtype(EnchantmentSubtype::Aura), 1)),
            effect: Effect::Regenerate {
                what: Selector::This,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bramble Elemental — {3}{G}{G} 4/4 Elemental. Whenever an Aura becomes
/// attached to it, create two 1/1 Saprolings (`EventKind::AuraAttached`).
pub fn bramble_elemental() -> CardDefinition {
    CardDefinition {
        name: "Bramble Elemental",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AuraAttached, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: saproling_token(),
            },
        }],
        ..Default::default()
    }
}

/// Skarrgan Pit-Skulk — {G} 1/1 Human Warrior with Bloodthirst 1 that can't be
/// blocked by creatures with less power.
pub fn skarrgan_pit_skulk() -> CardDefinition {
    CardDefinition {
        name: "Skarrgan Pit-Skulk",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Bloodthirst(1), Keyword::CantBeBlockedByPowerLess],
        triggered_abilities: vec![crate::effect::shortcut::bloodthirst(1)],
        ..Default::default()
    }
}

/// Gruul Nodorog — {4}{G}{G} 4/4 Beast. {R}: gains menace until end of turn.
pub fn gruul_nodorog() -> CardDefinition {
    CardDefinition {
        name: "Gruul Nodorog",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ostiary Thrull — {3}{B} 2/2 Thrull. {W}, {T}: tap target creature.
pub fn ostiary_thrull() -> CardDefinition {
    CardDefinition {
        name: "Ostiary Thrull",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rakdos Ickspitter — {1}{B}{R} 1/1 Thrull. {T}: deal 1 to target creature and
/// its controller loses 1 life.
pub fn rakdos_ickspitter() -> CardDefinition {
    CardDefinition {
        name: "Rakdos Ickspitter",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(R::Creature),
                    amount: Value::ONE,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Galvanic Arc — {2}{R} Aura. Enchant creature; ETB deals 3 to any target; the
/// enchanted creature has first strike.
pub fn galvanic_arc() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Galvanic Arc",
        cost: cost(&[generic(2), r()]),
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
            keywords: vec![Keyword::FirstStrike],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::Const(3),
        })],
        ..Default::default()
    }
}

/// Ghor-Clan Bloodscale — {3}{R} 2/1 Lizard Warrior with first strike.
/// {3}{G}: +2/+2 until end of turn, once each turn.
pub fn ghor_clan_bloodscale() -> CardDefinition {
    CardDefinition {
        name: "Ghor-Clan Bloodscale",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sandsower — {3}{W} 1/3 Spirit. Tap three untapped creatures you control:
/// tap target creature.
pub fn sandsower() -> CardDefinition {
    CardDefinition {
        name: "Sandsower",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_n_filter: Some((R::Creature.and(R::ControlledByYou), 3)),
            effect: Effect::Tap {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gruul Scrapper — {3}{G} 3/2 Human Berserker. When it enters, if {R} was
/// spent to cast it, it gains haste until end of turn
/// (`Predicate::SourceCastWithColorSpent`).
pub fn gruul_scrapper() -> CardDefinition {
    CardDefinition {
        name: "Gruul Scrapper",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceCastWithColorSpent {
                    color: Color::Red,
                    at_least: 1,
                }),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Steamcore Weird — {3}{U} 1/3 Weird. When it enters, if {R} was spent to cast
/// it, it deals 2 damage to any target.
pub fn steamcore_weird() -> CardDefinition {
    CardDefinition {
        name: "Steamcore Weird",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceCastWithColorSpent {
                    color: Color::Red,
                    at_least: 1,
                }),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Torch Drake — {3}{U} 2/2 Drake with flying. {1}{R}: +1/+0 until end of turn.
pub fn torch_drake() -> CardDefinition {
    CardDefinition {
        name: "Torch Drake",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
