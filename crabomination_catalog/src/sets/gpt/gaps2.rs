//! Guildpact (GPT) second gap wave: the Magemark aura anthems, a pair of
//! enters-if-color-spent creatures, a bloodthirst phoenix, and simple
//! spells/creatures. Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bloodthirst, on_dies, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

// ── Magemark auras ──────────────────────────────────────────────────────────
// Each enchants a creature and anthems *every* enchanted creature you control;
// the +1/+1 covers the enchanted host too (it matches `IsEnchanted`).

/// Fencer's Magemark — {2}{R} Aura. Enchant creature. Your enchanted creatures
/// get +1/+1 and have first strike.
pub fn fencers_magemark() -> CardDefinition {
    CardDefinition {
        name: "Fencer's Magemark",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that are enchanted get +1/+1 and have first strike.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsEnchanted),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::FirstStrike],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Guardian's Magemark — {2}{W} Aura with flash. Enchant creature. Your
/// enchanted creatures get +1/+1.
pub fn guardians_magemark() -> CardDefinition {
    CardDefinition {
        name: "Guardian's Magemark",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Creatures you control that are enchanted get +1/+1.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::IsEnchanted),
                power: 1,
                toughness: 1,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Skyrider Trainee — {4}{W} 3/3 Human Soldier. Has flying as long as it's
/// enchanted.
pub fn skyrider_trainee() -> CardDefinition {
    CardDefinition {
        name: "Skyrider Trainee",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Skyrider Trainee has flying as long as it's enchanted.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Flying,
                condition: R::IsEnchanted,
            },
        }],
        ..Default::default()
    }
}

/// Lionheart Maverick — {W} 1/1 Human Knight with vigilance. {4}{W}: gets
/// +1/+2 until end of turn.
pub fn lionheart_maverick() -> CardDefinition {
    CardDefinition {
        name: "Lionheart Maverick",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Order of the Stars — {W} 0/1 Human Cleric with defender. As it enters,
/// choose a color; it has protection from the chosen color.
pub fn order_of_the_stars() -> CardDefinition {
    CardDefinition {
        name: "Order of the Stars",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Cleric], ..Default::default() },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseColorForSelf,
        }],
        static_abilities: vec![StaticAbility {
            description: "This creature has protection from the chosen color.",
            effect: StaticEffect::GrantProtectionFromChosenColor { applies_to: Selector::This },
        }],
        ..Default::default()
    }
}

/// Ogre Savant — {4}{R} 3/2 Ogre Wizard. When it enters, if {U} was spent to
/// cast it, return target creature to its owner's hand.
pub fn ogre_savant() -> CardDefinition {
    CardDefinition {
        name: "Ogre Savant",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ogre, CreatureType::Wizard], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SourceCastWithColorSpent { color: Color::Blue, at_least: 1 },
                then: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Revenant Patriarch — {4}{B} 4/3 Spirit that can't block. When it enters, if
/// {W} was spent to cast it, target player skips their next combat phase.
pub fn revenant_patriarch() -> CardDefinition {
    CardDefinition {
        name: "Revenant Patriarch",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::SourceCastWithColorSpent { color: Color::White, at_least: 1 },
                then: Box::new(Effect::SkipNextCombatPhase { who: PlayerRef::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Restless Bones — {2}{B} 1/1 Skeleton. {3}{B}, {T}: target creature gains
/// swampwalk until end of turn. {1}{B}: Regenerate this creature.
pub fn restless_bones() -> CardDefinition {
    CardDefinition {
        name: "Restless Bones",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), b()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Landwalk(LandType::Swamp),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Smogsteed Rider — {2}{B}{B} 2/2 Human Wizard. Whenever it attacks, each
/// other attacking creature gains fear until end of turn.
pub fn smogsteed_rider() -> CardDefinition {
    CardDefinition {
        name: "Smogsteed Rider",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                keyword: Keyword::Fear,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Martyred Rusalka — {W} 1/1 Spirit. {W}, Sacrifice a creature: Target
/// creature can't attack this turn.
pub fn martyred_rusalka() -> CardDefinition {
    CardDefinition {
        name: "Martyred Rusalka",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantAttack,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skarrgan Firebird — {4}{R}{R} 3/3 Phoenix with bloodthirst 3 and flying.
/// {R}{R}{R}: Return this from your graveyard to your hand. Activate only if an
/// opponent was dealt damage this turn.
pub fn skarrgan_firebird() -> CardDefinition {
    CardDefinition {
        name: "Skarrgan Firebird",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phoenix], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![bloodthirst(3)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r(), r(), r()]),
            from_graveyard: true,
            condition: Some(Predicate::PlayerDamagedThisTurn { who: PlayerRef::EachOpponent }),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spells & noncreature ────────────────────────────────────────────────────

/// Runeboggle — {2}{U} Instant. Counter target spell unless its controller pays
/// {1}. Draw a card.
pub fn runeboggle() -> CardDefinition {
    CardDefinition {
        name: "Runeboggle",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(1)]),
                exile: false,
                extra_generic: None,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Primeval Light — {3}{G} Sorcery. Destroy all enchantments target player
/// controls.
pub fn primeval_light() -> CardDefinition {
    CardDefinition {
        name: "Primeval Light",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Enchantment },
            body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
        },
        ..Default::default()
    }
}

/// Hatching Plans — {1}{U} Enchantment. When it's put into a graveyard from the
/// battlefield, draw three cards.
pub fn hatching_plans() -> CardDefinition {
    CardDefinition {
        name: "Hatching Plans",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        }],
        ..Default::default()
    }
}

/// Gruul War Plow — {4} Artifact. Creatures you control have trample. {1}{R}{G}:
/// This artifact becomes a 4/4 Juggernaut artifact creature until end of turn.
pub fn gruul_war_plow() -> CardDefinition {
    CardDefinition {
        name: "Gruul War Plow",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have trample.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), g()]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![CreatureType::Juggernaut],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sinstriker's Will — {3}{W} Aura. Enchant creature. Enchanted creature has
/// "{T}: This creature deals damage equal to its power to target attacking or
/// blocking creature."
pub fn sinstrikers_will() -> CardDefinition {
    CardDefinition {
        name: "Sinstriker's Will",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::This,
                    target: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Cryptwailing — {3}{B} Enchantment. {1}, Exile two creature cards from your
/// graveyard: Target player discards a card. Activate only as a sorcery.
pub fn cryptwailing() -> CardDefinition {
    CardDefinition {
        name: "Cryptwailing",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sorcery_speed: true,
            exile_other_filter: Some((R::Creature, 2)),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nullstone Gargoyle — {9} Artifact Creature — Gargoyle 4/5 with flying.
/// Whenever the first noncreature spell of a turn is cast, counter that spell.
pub fn nullstone_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Nullstone Gargoyle",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature },
                    Predicate::FirstNoncreatureSpellThisTurn,
                ]),
            ),
            effect: Effect::CounterSpell { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Angel of Despair — {3}{W}{W}{B}{B} 5/5 Angel with flying. When it enters,
/// destroy target permanent.
pub fn angel_of_despair() -> CardDefinition {
    CardDefinition {
        name: "Angel of Despair",
        cost: cost(&[generic(3), w(), w(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy { what: target_filtered(R::Permanent) },
        }],
        ..Default::default()
    }
}

/// Debtors' Knell — {4}{W/B}{W/B}{W/B} Enchantment. At the beginning of your
/// upkeep, put target creature card from a graveyard onto the battlefield under
/// your control.
pub fn debtors_knell() -> CardDefinition {
    CardDefinition {
        name: "Debtors' Knell",
        cost: cost(&[
            generic(4),
            crate::mana::hybrid(Color::White, Color::Black),
            crate::mana::hybrid(Color::White, Color::Black),
            crate::mana::hybrid(Color::White, Color::Black),
        ]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..Default::default()
    }
}

/// Hypervolt Grasp — {2}{R} Aura. Enchant creature. Enchanted creature has
/// "{T}: This creature deals 1 damage to any target." {1}{U}: Return this Aura
/// to its owner's hand.
pub fn hypervolt_grasp() -> CardDefinition {
    CardDefinition {
        name: "Hypervolt Grasp",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Invoke the Firemind — {X}{U}{U}{R} Sorcery. Choose one — draw X cards; or
/// deal X damage to any target.
pub fn invoke_the_firemind() -> CardDefinition {
    CardDefinition {
        name: "Invoke the Firemind",
        cost: cost(&[crate::mana::x(), u(), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseModesCast {
            modes: vec![
                Effect::Draw { who: Selector::You, amount: Value::XFromCost },
                Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
            ],
            min: 1,
            max: 1,
            allow_repeats: false,
        },
        ..Default::default()
    }
}

/// Orzhov Euthanist — {2}{B} 2/2 Human Assassin with haunt. When it enters or
/// the creature it haunts dies, destroy target creature that was dealt damage
/// this turn.
pub fn orzhov_euthanist() -> CardDefinition {
    let destroy = Effect::Destroy {
        what: target_filtered(R::Creature.and(R::DealtDamageThisTurn)),
    };
    CardDefinition {
        name: "Orzhov Euthanist",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Assassin], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: destroy.clone(),
            },
            on_dies(Effect::HauntCreature { body: Box::new(destroy) }),
        ],
        ..Default::default()
    }
}
