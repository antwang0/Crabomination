//! Aristocrats + enchantress batch. The AKH Monument cycle (color-gated
//! creature-cost reduction + a per-cast rider), sacrifice-outlet payoffs,
//! the Curiosity aura family (combat-damage → draw), constellation life
//! swings, and lifegain-matters counters. Tests in `tests/recent81.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{draw, gain_life, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w};

// ── Sacrifice outlets ────────────────────────────────────────────────────────

/// Vampiric Rites — {B} Enchantment. {1}{B}, Sacrifice a creature: You gain 1
/// life and draw a card.
pub fn vampiric_rites() -> CardDefinition {
    CardDefinition {
        name: "Vampiric Rites",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![gain_life(1), draw(1)]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Blasting Station — {3} Artifact. {T}, Sacrifice a creature: This deals 1
/// damage to any target. Whenever a creature enters, you may untap this.
pub fn blasting_station() -> CardDefinition {
    CardDefinition {
        name: "Blasting Station",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::DealDamage {
                to: crate::effect::shortcut::target_any(),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                },
            ),
            effect: Effect::MayDo {
                description: "untap Blasting Station".into(),
                body: Box::new(Effect::Untap {
                    what: Selector::This,
                    up_to: None,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Additional-cost draw ───────────────────────────────────────────────────────

/// Seize the Spoils — {2}{R} Sorcery. Additional cost: discard a card. Draw two
/// cards and create a Treasure token.
pub fn seize_the_spoils() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Seize the Spoils",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::Discard {
            count: 1,
            filter: None,
        }],
        effect: Effect::Seq(vec![
            draw(2),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(crate::game::effects::treasure_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Blood Divination — {3}{B} Sorcery. Additional cost: sacrifice a creature.
/// Draw three cards.
pub fn blood_divination() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Blood Divination",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        effect: draw(3),
        ..Default::default()
    }
}

// ── Curiosity aura family ──────────────────────────────────────────────────────

/// "Whenever enchanted creature deals combat damage to a player, you may draw a
/// card." (The printed "deals damage to an opponent" on Curiosity/Ophidian
/// Eye/Keen Sense is modeled as combat damage — the common case.)
fn combat_damage_draw() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: draw(1),
    }
}

fn draw_aura(
    name: &'static str,
    mana: &[crate::mana::ManaSymbol],
    pt: i32,
    extra: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: extra,
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature,
            },
        },
        equipped_bonus: Some(EquipBonus {
            power: pt,
            toughness: pt,
            triggered_abilities: vec![combat_damage_draw()],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Snake Umbra — {2}{G} Aura. Enchanted creature gets +1/+1 and has "Whenever
/// this creature deals damage to an opponent, you may draw a card." Umbra armor.
pub fn snake_umbra() -> CardDefinition {
    let mut c = draw_aura(
        "Snake Umbra",
        &[generic(2), g()],
        1,
        vec![Keyword::UmbraArmor],
    );
    // Umbra armor is a keyword on the Aura itself, not a granted keyword.
    if let Some(b) = c.equipped_bonus.as_mut() {
        b.power = 1;
        b.toughness = 1;
    }
    c
}

/// Curious Obsession — {U} Aura. Enchanted creature gets +1/+1 and has
/// "Whenever this creature deals combat damage to a player, you may draw a
/// card." At your end step, if you didn't attack, sacrifice this Aura.
pub fn curious_obsession() -> CardDefinition {
    let mut c = draw_aura("Curious Obsession", &[u()], 1, vec![]);
    c.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(TurnStep::End),
            EventScope::ActivePlayer,
        ),
        effect: Effect::If {
            cond: Predicate::Not(Box::new(Predicate::PlayerAttackedThisTurn {
                who: PlayerRef::You,
            })),
            then: Box::new(Effect::SacrificePermanent {
                what: Selector::This,
            }),
            else_: Box::new(Effect::Noop),
        },
    }];
    c
}

// ── Lifegain matters ───────────────────────────────────────────────────────────

/// Ageless Entity — {3}{G}{G} 4/4 Elemental. Whenever you gain life, put that
/// many +1/+1 counters on this creature.
pub fn ageless_entity() -> CardDefinition {
    CardDefinition {
        name: "Ageless Entity",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Sunbond — {3}{W} Aura. Enchanted creature has "Whenever you gain life, put
/// that many +1/+1 counters on this creature."
pub fn sunbond() -> CardDefinition {
    CardDefinition {
        name: "Sunbond",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature,
            },
        },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::TriggerEventAmount,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Nyx-Fleece Ram — {1}{W} 0/5 Enchantment Creature — Sheep. At your upkeep,
/// gain 1 life.
pub fn nyx_fleece_ram() -> CardDefinition {
    CardDefinition {
        name: "Nyx-Fleece Ram",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sheep],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: gain_life(1),
        }],
        ..Default::default()
    }
}

/// Wall of Reverence — {3}{W} 1/6 Spirit Wall. Defender, flying. At your end
/// step, you may gain life equal to the power of target creature you control.
/// (The "target" is modeled as the greatest-power creature you control — the
/// value-maximizing pick.)
pub fn wall_of_reverence() -> CardDefinition {
    CardDefinition {
        name: "Wall of Reverence",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wall],
            ..Default::default()
        },
        power: 1,
        toughness: 6,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::GreatestPowerControlledMatching(
                    R::Creature.and(R::ControlledByYou),
                ))),
            },
        }],
        ..Default::default()
    }
}

// ── Constellation ──────────────────────────────────────────────────────────────

/// Constellation trigger: whenever this or another enchantment you control
/// enters, run `body`.
fn constellation(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Enchantment,
            },
        ),
        effect: body,
    }
}

/// Grim Guardian — {2}{B} 1/4 Enchantment Creature — Zombie. Constellation —
/// whenever this or another enchantment you control enters, each opponent loses
/// 1 life.
pub fn grim_guardian() -> CardDefinition {
    CardDefinition {
        name: "Grim Guardian",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![constellation(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Underworld Coinsmith — {W}{B} 2/2 Human Cleric Enchantment Creature.
/// Constellation — gain 1 life. {W}{B}, Pay 1 life: Each opponent loses 1 life.
pub fn underworld_coinsmith() -> CardDefinition {
    CardDefinition {
        name: "Underworld Coinsmith",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![constellation(gain_life(1))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), b()]),
            life_cost: 1,
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Misc ───────────────────────────────────────────────────────────────────────

/// Fecundity — {2}{G} Enchantment. Whenever a creature dies, that creature's
/// controller may draw a card. (Modeled as: you draw when a creature you
/// control dies — the "each controller" clause is approximated to the
/// Fecundity controller.)
pub fn fecundity() -> CardDefinition {
    CardDefinition {
        name: "Fecundity",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Sanctuary Cat — {W} 1/2 Cat. Vanilla.
pub fn sanctuary_cat() -> CardDefinition {
    CardDefinition {
        name: "Sanctuary Cat",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        ..Default::default()
    }
}

/// Chaplain's Blessing — {W} Sorcery. You gain 5 life.
pub fn chaplains_blessing() -> CardDefinition {
    CardDefinition {
        name: "Chaplain's Blessing",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: gain_life(5),
        ..Default::default()
    }
}

/// Vicious Hunger — {B}{B} Sorcery. Deals 2 damage to target creature and you
/// gain 2 life.
pub fn vicious_hunger() -> CardDefinition {
    CardDefinition {
        name: "Vicious Hunger",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(2),
            },
            gain_life(2),
        ]),
        ..Default::default()
    }
}

/// Life Goes On — {G} Instant. You gain 4 life; 8 instead if a creature died
/// this turn.
pub fn life_goes_on() -> CardDefinition {
    CardDefinition {
        name: "Life Goes On",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast {
                at_least: Value::Const(1),
            },
            then: Box::new(gain_life(8)),
            else_: Box::new(gain_life(4)),
        },
        ..Default::default()
    }
}

/// Feed the Clan — {1}{G} Instant. You gain 5 life; 10 instead if you control a
/// creature with power 4 or greater (Ferocious).
pub fn feed_the_clan() -> CardDefinition {
    CardDefinition {
        name: "Feed the Clan",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::If {
            cond: Predicate::FerociousActive {
                who: PlayerRef::You,
            },
            then: Box::new(gain_life(10)),
            else_: Box::new(gain_life(5)),
        },
        ..Default::default()
    }
}

/// Silverflame Ritual — {3}{W} Sorcery. Put a +1/+1 counter on each creature
/// you control. (Printed Adamant vigilance rider omitted.)
pub fn silverflame_ritual() -> CardDefinition {
    CardDefinition {
        name: "Silverflame Ritual",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::AddCounter {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        },
        ..Default::default()
    }
}

/// Renewed Faith — {2}{W} Instant. You gain 6 life. Cycling {1}{W}; when you
/// cycle this card, you may gain 2 life.
pub fn renewed_faith() -> CardDefinition {
    CardDefinition {
        name: "Renewed Faith",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Cycling(cost(&[generic(1), w()]))],
        effect: gain_life(6),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: gain_life(2),
        }],
        ..Default::default()
    }
}

/// Mask of Griselbrand — {1}{B}{B} Legendary Equipment. Equipped creature has
/// flying and lifelink. When equipped creature dies, you may pay X life (X =
/// its power); if you do, draw X cards. Equip {3}. (The pay-X-life gate is
/// approximated as an unconditional draw of the dying creature's power.)
pub fn mask_of_griselbrand() -> CardDefinition {
    CardDefinition {
        name: "Mask of Griselbrand",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Flying, Keyword::Lifelink],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
            }],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}
