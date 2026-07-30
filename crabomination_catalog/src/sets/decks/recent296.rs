//! Ravnica batch 6: Radiance (Rally the Righteous via the new
//! `Selector::RadianceGroup`), a Blocks-trigger tapper, and a spread of guild
//! auras/utility. Tests in `recent_b/recent_296`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

// ── Boros Radiance ──────────────────────────────────────────────────────────

/// Rally the Righteous — {1}{R}{W} Instant. Radiance — untap target creature
/// and each other creature that shares a color with it; those creatures get
/// +2/+0 until end of turn.
pub fn rally_the_righteous() -> CardDefinition {
    let group = || Selector::RadianceGroup {
        subject: Box::new(target_filtered(R::Creature)),
    };
    CardDefinition {
        name: "Rally the Righteous",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: group(),
                up_to: None,
            },
            Effect::PumpPT {
                what: group(),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Dimir ───────────────────────────────────────────────────────────────────

/// Vertigo Spawn — {1}{U} 0/3 Illusion with Defender. Whenever it blocks a
/// creature, tap that creature; it doesn't untap during its controller's next
/// untap step.
pub fn vertigo_spawn() -> CardDefinition {
    CardDefinition {
        name: "Vertigo Spawn",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::BlockedAttacker,
                },
                Effect::SkipNextUntap {
                    what: Selector::BlockedAttacker,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Gruul / Izzet ───────────────────────────────────────────────────────────

/// Tin Street Hooligan — {1}{R} 2/1 Goblin Rogue. When it enters, if {G} was
/// spent to cast it, destroy target artifact.
pub fn tin_street_hooligan() -> CardDefinition {
    CardDefinition {
        name: "Tin Street Hooligan",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceCastWithColorSpent {
                    color: Color::Green,
                    at_least: 1,
                }),
            effect: Effect::Destroy {
                what: target_filtered(R::Artifact),
            },
        }],
        ..Default::default()
    }
}

/// Petrahydrox — {3}{U/R} 3/3 Weird. When it becomes the target of a spell or
/// ability, return it to its owner's hand.
pub fn petrahydrox() -> CardDefinition {
    CardDefinition {
        name: "Petrahydrox",
        cost: cost(&[generic(3), hybrid(Color::Blue, Color::Red)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Weird],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
        }],
        ..Default::default()
    }
}

// ── Orzhov ──────────────────────────────────────────────────────────────────

/// Souls of the Faultless — {W}{B}{B} 0/4 Spirit with Defender. Whenever it's
/// dealt combat damage, you gain that much life and the attacking player loses
/// that much life.
pub fn souls_of_the_faultless() -> CardDefinition {
    CardDefinition {
        name: "Souls of the Faultless",
        cost: cost(&[w(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtCombatDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::TriggerEventAmount,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::CombatDamagerController(Box::new(
                        Selector::This,
                    ))),
                    amount: Value::TriggerEventAmount,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Shadow Lance — {W} Aura. Enchant creature. Enchanted creature has first
/// strike. {1}{B}: Enchanted creature gets +2/+2 until end of turn.
pub fn shadow_lance() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Shadow Lance",
        cost: cost(&[w()]),
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
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::PumpPT {
                    what: Selector::AttachedToMe(Box::new(Selector::This)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Shielding Plax — {2}{G/U} Aura. Enchant creature. When it enters, draw a
/// card. Enchanted creature can't be the target of opponents' spells/abilities.
pub fn shielding_plax() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Shielding Plax",
        cost: cost(&[generic(2), hybrid(Color::Green, Color::Blue)]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Hexproof],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Utility ─────────────────────────────────────────────────────────────────

/// Dowsing Shaman — {4}{G} 3/4 Centaur Shaman. {2}{G}, {T}: Return target
/// enchantment card from your graveyard to your hand.
pub fn dowsing_shaman() -> CardDefinition {
    CardDefinition {
        name: "Dowsing Shaman",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            tap_cost: true,
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(R::Enchantment),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Poison the Well — {2}{B/R}{B/R} Sorcery. Destroy target land and deal 2
/// damage to that land's controller.
pub fn poison_the_well() -> CardDefinition {
    CardDefinition {
        name: "Poison the Well",
        cost: cost(&[
            generic(2),
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(target_filtered(R::Land)))),
                amount: Value::Const(2),
            },
            Effect::Destroy {
                what: target_filtered(R::Land),
            },
        ]),
        ..Default::default()
    }
}

/// Congregation at Dawn — {G}{G}{W} Instant. Search your library for up to
/// three creature cards, reveal them, shuffle, then put them on top.
pub fn congregation_at_dawn() -> CardDefinition {
    CardDefinition {
        name: "Congregation at Dawn",
        cost: cost(&[g(), g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Creature,
            to: ZoneDest::Library {
                who: PlayerRef::You,
                pos: LibraryPosition::Top,
            },
            count: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Peregrine Mask — {1} Equipment. Equipped creature has defender, flying, and
/// first strike. Equip {2}.
pub fn peregrine_mask() -> CardDefinition {
    use crate::card::EquipBonus;
    CardDefinition {
        name: "Peregrine Mask",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Defender, Keyword::Flying, Keyword::FirstStrike],
            ..Default::default()
        }),
        ..Default::default()
    }
}
