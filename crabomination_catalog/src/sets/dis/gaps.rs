//! Dissension (DIS) gap cards beyond the creature batch — Auras, enchantments,
//! and artifacts filling the `set_gaps.py dis` remainder.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Selector, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, StaticEffect, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{b, cost, g, generic, r, u, w};

/// Nettling Curse — {2}{B} Aura. Enchant creature. Whenever enchanted creature
/// attacks or blocks, its controller loses 3 life. `{1}{R}: Enchanted creature
/// attacks this turn if able.`
pub fn nettling_curse() -> CardDefinition {
    let lose3 = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
    };
    CardDefinition {
        name: "Nettling Curse",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![lose3(EventKind::Attacks), lose3(EventKind::Blocks)],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::GrantKeyword {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::MustAttack,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Riot Spikes — {B/R} Aura. Enchant creature. Enchanted creature gets +2/-1.
pub fn riot_spikes() -> CardDefinition {
    CardDefinition {
        name: "Riot Spikes",
        cost: cost(&[b()]), // {B/R} — modeled with the black pip
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, toughness: -1, ..Default::default() }),
        ..Default::default()
    }
}

/// Street Savvy — {G} Aura. Enchant creature. Enchanted creature gets +0/+2.
/// (The "can block landwalkers as though they lacked landwalk" rider is a niche
/// combat-evasion bypass tracked in TODO.md.)
pub fn street_savvy() -> CardDefinition {
    CardDefinition {
        name: "Street Savvy",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { toughness: 2, ..Default::default() }),
        ..Default::default()
    }
}

/// Proper Burial — {3}{W} Enchantment. Whenever a creature you control dies, you
/// gain life equal to that creature's toughness.
pub fn proper_burial() -> CardDefinition {
    CardDefinition {
        name: "Proper Burial",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        ..Default::default()
    }
}

/// Rain of Gore — {B}{R} Enchantment. If a spell or ability would cause its
/// controller to gain life, that player loses that much life instead.
pub fn rain_of_gore() -> CardDefinition {
    CardDefinition {
        name: "Rain of Gore",
        cost: cost(&[b(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If a spell or ability would cause its controller to gain life, that player loses that much life instead.",
            effect: StaticEffect::LifeGainBecomesLoss { target: PlayerStaticTarget::EachPlayer },
        }],
        ..Default::default()
    }
}

/// Skullmead Cauldron — {4} Artifact. `{T}: You gain 1 life.` and
/// `{T}, Discard a card: You gain 3 life.`
pub fn skullmead_cauldron() -> CardDefinition {
    CardDefinition {
        name: "Skullmead Cauldron",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Celestial Ancient — {3}{W}{W} 3/3 Elemental with flying. Whenever you cast an
/// enchantment spell, put a +1/+1 counter on each creature you control.
pub fn celestial_ancient() -> CardDefinition {
    CardDefinition {
        name: "Celestial Ancient",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasCardType(CardType::Enchantment))),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Nihilistic Glee — {2}{B}{B} Enchantment. `{2}{B}, Discard a card: Target
/// opponent loses 1 life and you gain 1 life.` and Hellbent —
/// `{1}, Pay 2 life: Draw a card. Activate only if you have no cards in hand.`
pub fn nihilistic_glee() -> CardDefinition {
    CardDefinition {
        name: "Nihilistic Glee",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), b()]),
                discard_cost: Some((R::Any, 1)),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::ONE,
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                life_cost: 2,
                condition: Some(Predicate::HellbentActive { who: PlayerRef::You }),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Slithering Shade — {B} 0/1 Shade with Defender. `{B}: +1/+1 until end of
/// turn.` Hellbent — can attack as though it lacked Defender while you have no
/// cards in hand.
pub fn slithering_shade() -> CardDefinition {
    CardDefinition {
        name: "Slithering Shade",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shade], ..Default::default() },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        static_abilities: vec![StaticAbility {
            description: "Can attack as though it didn't have defender while you have no cards in hand.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile {
                condition: Predicate::HellbentActive { who: PlayerRef::You },
            },
        }],
        ..Default::default()
    }
}

/// Ocular Halo — {3}{U} Aura. Enchanted creature has `{T}: Draw a card.` and the
/// aura grants it vigilance: `{W}: Enchanted creature gains vigilance until end
/// of turn.`
pub fn ocular_halo() -> CardDefinition {
    CardDefinition {
        name: "Ocular Halo",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            }],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantKeyword {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sprouting Phytohydra — {4}{G} 0/2 Plant Hydra with Defender. Whenever it's
/// dealt damage, you may create a token that's a copy of it.
pub fn sprouting_phytohydra() -> CardDefinition {
    CardDefinition {
        name: "Sprouting Phytohydra",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Hydra],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "create a token that's a copy of this creature".to_string(),
                body: Box::new(Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::This,
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
                }),
            },
        }],
        ..Default::default()
    }
}

/// Ratcatcher — {4}{B}{B} 4/4 Ogre Rogue with Fear. At the beginning of your
/// upkeep, you may search your library for a Rat card and put it into your hand.
pub fn ratcatcher() -> CardDefinition {
    CardDefinition {
        name: "Ratcatcher",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasCreatureType(CreatureType::Rat),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Cytospawn Shambler — {6}{G} 0/0 Elemental Mutant with Graft 6.
/// `{G}: Target creature with a +1/+1 counter on it gains trample until end of
/// turn.`
pub fn cytospawn_shambler() -> CardDefinition {
    CardDefinition {
        name: "Cytospawn Shambler",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(6))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cytoplast Manipulator — {2}{U}{U} 0/0 Human Wizard Mutant with Graft 2.
/// `{U}, {T}: Gain control of target creature with a +1/+1 counter on it for as
/// long as this creature remains on the battlefield.`
pub fn cytoplast_manipulator() -> CardDefinition {
    CardDefinition {
        name: "Cytoplast Manipulator",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wizard, CreatureType::Mutant],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::GainControlWhileSourceRemains {
                what: target_filtered(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Paladin of Prahv — {4}{W}{W} 3/4 Human Knight. "Whenever this creature deals
/// damage, you gain that much life" is modeled as Lifelink (CR 702.15 — the
/// controller gains that much on any damage the source deals). The Forecast
/// grant-lifelink-to-a-target rider is deferred (TODO.md).
pub fn paladin_of_prahv() -> CardDefinition {
    CardDefinition {
        name: "Paladin of Prahv",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    }
}

/// Wit's End — {5}{B}{B} Sorcery. Target player discards their hand.
pub fn wits_end() -> CardDefinition {
    CardDefinition {
        name: "Wit's End",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(100), // capped at hand size — "their hand"
            random: false,
        },
        ..Default::default()
    }
}

/// Weight of Spires — {R} Instant. Deals damage to target creature equal to the
/// number of nonbasic lands its controller controls.
pub fn weight_of_spires() -> CardDefinition {
    CardDefinition {
        name: "Weight of Spires",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::NonbasicLandCountControlledBy(PlayerRef::ControllerOf(Box::new(
                Selector::Target(0),
            ))),
        },
        ..Default::default()
    }
}

/// Tidespout Tyrant — {5}{U}{U}{U} 5/5 Djinn with flying. Whenever you cast a
/// spell, return target permanent to its owner's hand.
pub fn tidespout_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Tidespout Tyrant",
        cost: cost(&[generic(5), u(), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Djinn], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Taste for Mayhem — {R} Aura. Enchanted creature gets +2/+0. (The Hellbent
/// rider — an additional +2/+0 while you have no cards in hand — needs a
/// condition-gated attached-creature pump static; deferred, tracked in TODO.md.)
pub fn taste_for_mayhem() -> CardDefinition {
    CardDefinition {
        name: "Taste for Mayhem",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: 2, ..Default::default() }),
        ..Default::default()
    }
}
