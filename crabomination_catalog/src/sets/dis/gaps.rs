//! Dissension (DIS) gap cards beyond the creature batch — Auras, enchantments,
//! and artifacts filling the `set_gaps.py dis` remainder.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, forecast, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, PlayerStaticTarget, StaticEffect, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, SpendRestriction, b, cost, g, generic, r, u, w, x};

use super::super::tap_add_colorless;

/// Nettling Curse — {2}{B} Aura. Enchant creature. Whenever enchanted creature
/// attacks or blocks, its controller loses 3 life. `{1}{R}: Enchanted creature
/// attacks this turn if able.`
pub fn nettling_curse() -> CardDefinition {
    let lose3 = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::LoseLife {
            who: Selector::You,
            amount: Value::Const(3),
        },
    };
    CardDefinition {
        name: "Nettling Curse",
        cost: cost(&[generic(2), b()]),
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

/// Blessing of the Nephilim — {W} Aura. Enchant creature. Enchanted creature
/// gets +1/+1 for each of its colors.
pub fn blessing_of_the_nephilim() -> CardDefinition {
    use crate::card::EquipScale;
    CardDefinition {
        name: "Blessing of the Nephilim",
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
            scale: Some(EquipScale {
                filter: R::Any,
                per_power: 1,
                per_toughness: 1,
                count_host_colors: true,
                ..Default::default()
            }),
            ..Default::default()
        }),
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
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: -1,
            ..Default::default()
        }),
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
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            toughness: 2,
            ..Default::default()
        }),
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
            effect: StaticEffect::LifeGainBecomesLoss {
                target: PlayerStaticTarget::EachPlayer,
            },
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
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                discard_cost: Some((R::Any, 1)),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
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
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(R::HasCardType(CardType::Enchantment)),
            ),
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
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                life_cost: 2,
                condition: Some(Predicate::HellbentActive {
                    who: PlayerRef::You,
                }),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
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
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shade],
            ..Default::default()
        },
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
                condition: Predicate::HellbentActive {
                    who: PlayerRef::You,
                },
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
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
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
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
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
/// damage, you gain that much life" is modeled as Lifelink (CR 702.15). Forecast
/// — {1}{W}: Whenever target creature deals damage this turn, you gain that much
/// life.
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
        activated_abilities: vec![forecast(
            cost(&[generic(1), w()]),
            Effect::GainLifeWhenTargetDealsDamageThisTurn { slot: 0 },
        )],
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
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn],
            ..Default::default()
        },
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

/// Taste for Mayhem — {R} Aura. Enchanted creature gets +2/+0, and an additional
/// +2/+0 while you have no cards in hand (Hellbent) — a condition-gated
/// attached-creature pump that tracks the empty-hand state live.
pub fn taste_for_mayhem() -> CardDefinition {
    CardDefinition {
        name: "Taste for Mayhem",
        cost: cost(&[r()]),
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
            power: 2,
            conditional_pt: Some((
                2,
                0,
                Predicate::HellbentActive {
                    who: PlayerRef::You,
                },
            )),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Windreaver — {3}{W}{U} 1/3 Elemental with flying and four pump/evade
/// activations: `{W}` gain vigilance, `{W}` +0/+1, `{U}` switch P/T, and
/// `{U}` return this creature to its owner's hand.
pub fn windreaver() -> CardDefinition {
    CardDefinition {
        name: "Windreaver",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(0),
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::SwitchPT {
                    what: Selector::This,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Walking Archive — {3} 1/1 Golem with Defender. Enters with a +1/+1 counter.
/// At the beginning of each player's upkeep, that player draws a card for each
/// +1/+1 counter on it. `{2}{W}{U}: Put a +1/+1 counter on this creature.`
pub fn walking_archive() -> CardDefinition {
    CardDefinition {
        name: "Walking Archive",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Defender],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), u()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rix Maadi, Dungeon Palace — Land. `{T}: Add {C}.` `{1}{B}{R}, {T}: Each
/// player discards a card. Activate only as a sorcery.`
pub fn rix_maadi_dungeon_palace() -> CardDefinition {
    CardDefinition {
        name: "Rix Maadi, Dungeon Palace",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), b(), r()]),
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::ONE,
                    random: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Novijen, Heart of Progress — Land. `{T}: Add {C}.` `{G}{U}, {T}: Put a +1/+1
/// counter on each creature that entered the battlefield this turn.`
pub fn novijen_heart_of_progress() -> CardDefinition {
    CardDefinition {
        name: "Novijen, Heart of Progress",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[g(), u()]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(R::Creature.and(R::EnteredThisTurn)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Pillar of the Paruns — Land. `{T}: Add one mana of any color. Spend this mana
/// only to cast a multicolored spell.`
pub fn pillar_of_the_paruns() -> CardDefinition {
    CardDefinition {
        name: "Pillar of the Paruns",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                    SpendRestriction::MulticoloredSpell,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nightcreep — {B}{B} Instant. Until end of turn, all creatures become black
/// and all lands become Swamps.
pub fn nightcreep() -> CardDefinition {
    CardDefinition {
        name: "Nightcreep",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeColor {
                what: Selector::EachPermanent(R::Creature),
                colors: vec![Color::Black],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::BecomeBasicLand {
                what: Selector::EachPermanent(R::Land),
                land_type: LandType::Swamp,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Demonfire — {X}{R} Sorcery. Deals X damage to any target; a creature dealt
/// damage this way is exiled instead of dying. Hellbent — with no cards in hand
/// the damage can't be prevented (the can't-be-countered rider is cast-time).
pub fn demonfire() -> CardDefinition {
    CardDefinition {
        name: "Demonfire",
        cost: cost(&[x(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::HellbentActive {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::DamageCantBePreventedThisTurn),
                else_: Box::new(Effect::Noop),
            },
            // Install the exile-instead-of-die redirect before the damage lands.
            Effect::ExileIfWouldDieThisTurn {
                what: Selector::Target(0),
            },
            Effect::DealDamage {
                to: target_any(),
                amount: Value::XFromCost,
            },
        ]),
        ..Default::default()
    }
}

/// Grand Arbiter Augustin IV — {2}{W}{U} 2/3 Legendary Human Advisor. Your
/// white and blue spells cost {1} less; opponents' spells cost {1} more.
pub fn grand_arbiter_augustin_iv() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Grand Arbiter Augustin IV",
        cost: cost(&[generic(2), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![
            StaticAbility {
                description: "White spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: R::HasColor(Color::White),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Blue spells you cast cost {1} less to cast.",
                effect: StaticEffect::CostReduction {
                    filter: R::HasColor(Color::Blue),
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Spells your opponents cast cost {1} more to cast.",
                effect: StaticEffect::OpponentSpellsCostMore {
                    filter: R::Any,
                    amount: 1,
                },
            },
        ],
        ..Default::default()
    }
}

/// Magewright's Stone — {2} Artifact. `{1}, {T}: Untap target creature that has
/// an activated ability with {T} in its cost.` (The tap-ability restriction on
/// the target is approximated as any creature.)
pub fn magewrights_stone() -> CardDefinition {
    CardDefinition {
        name: "Magewright's Stone",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Untap {
                what: target_filtered(R::Creature),
                up_to: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hellhole Rats — {2}{B}{R} 2/2 Rat with Haste. When it enters, target player
/// discards a card; deal damage to that player equal to that card's mana value.
pub fn hellhole_rats() -> CardDefinition {
    CardDefinition {
        name: "Hellhole Rats",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Player,
                    },
                    amount: Value::ONE,
                    random: false,
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Player,
                    },
                    amount: Value::GreatestDiscardedManaValueThisEffect,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Freewind Equenaut — {2}{W} 2/2 Human Archer with flying. As long as it's
/// enchanted, it has "{T}: deal 2 damage to target attacking or blocking
/// creature."
pub fn freewind_equenaut() -> CardDefinition {
    CardDefinition {
        name: "Freewind Equenaut",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Archer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as this is enchanted, it has \"{T}: deal 2 damage to target attacking or blocking creature.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::This,
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::DealDamage {
                        to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                        amount: Value::Const(2),
                    },
                    ..Default::default()
                },
                condition: Some(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsEnchanted,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Govern the Guildless — {5}{U} Sorcery. Gain control of target monocolored
/// creature. Forecast — {1}{U}: Target creature becomes the color or colors of
/// your choice until end of turn.
pub fn govern_the_guildless() -> CardDefinition {
    CardDefinition {
        name: "Govern the Guildless",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GainControl {
            what: target_filtered(R::Creature.and(R::Monocolored)),
            to: Some(PlayerRef::You),
            duration: Duration::Permanent,
        },
        activated_abilities: vec![forecast(
            cost(&[generic(1), u()]),
            Effect::BecomeChosenColor {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
        )],
        ..Default::default()
    }
}

/// Anthem of Rakdos — {2}{B}{R}{R} Enchantment. Whenever a creature you control
/// attacks, it gets +2/+0 until end of turn and this deals 1 damage to you.
/// Hellbent — while your hand is empty, your sources deal double damage.
pub fn anthem_of_rakdos() -> CardDefinition {
    CardDefinition {
        name: "Anthem of Rakdos",
        cost: cost(&[generic(2), b(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::DealDamage {
                    to: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Hellbent — while you have no cards in hand, sources you control deal double damage.",
            effect: StaticEffect::DoubleYourSourcesDamageWhileHellbent,
        }],
        ..Default::default()
    }
}

/// Plumes of Peace — {1}{W}{U} Aura. Enchant creature. Enchanted creature
/// doesn't untap during its controller's untap step. Forecast — {W}{U}: Tap
/// target creature.
pub fn plumes_of_peace() -> CardDefinition {
    CardDefinition {
        name: "Plumes of Peace",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![forecast(
            cost(&[w(), u()]),
            Effect::Tap {
                what: target_filtered(R::Creature),
            },
        )],
        ..Default::default()
    }
}

/// Avatar of Discord — {B/R}{B/R}{B/R} 5/3 Avatar with flying. When it enters,
/// sacrifice it unless you discard two cards.
pub fn avatar_of_discord() -> CardDefinition {
    CardDefinition {
        name: "Avatar of Discord",
        cost: cost(&[b(), b(), b()]), // {B/R}×3 — modeled with black pips
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDiscard {
                description: "Discard two cards to keep Avatar of Discord?".into(),
                count: Value::Const(2),
                then: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::SacrificeSource)),
            },
        }],
        ..Default::default()
    }
}

/// Omnibian — {1}{G}{G}{U} 3/3 Frog. `{T}: Target creature becomes a Frog with
/// base power and toughness 3/3 until end of turn.`
pub fn omnibian() -> CardDefinition {
    CardDefinition {
        name: "Omnibian",
        cost: cost(&[generic(1), g(), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::BecomeCreatureType {
                    what: Selector::Target(0),
                    creature_types: vec![CreatureType::Frog],
                    duration: Duration::EndOfTurn,
                },
                Effect::SetBasePT {
                    what: Selector::Target(0),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unliving Psychopath — {2}{B}{B} 0/4 Zombie Assassin. `{B}: +1/-1 until end of
/// turn.` `{B}, {T}: Destroy target creature with power less than this
/// creature's power.`
pub fn unliving_psychopath() -> CardDefinition {
    CardDefinition {
        name: "Unliving Psychopath",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Assassin],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                tap_cost: true,
                effect: Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::PowerLessThanSource)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Cytoshape — {1}{G}{U} Instant. Target creature becomes a copy of another
/// creature until end of turn. (The "nonlegendary" restriction on the copied
/// creature is approximated as any creature.)
pub fn cytoshape() -> CardDefinition {
    CardDefinition {
        name: "Cytoshape",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::BecomeCopyOfFor {
            what: Selector::Target(0),
            source: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature,
            },
            duration: Duration::EndOfTurn,
            non_legendary: false,
        },
        ..Default::default()
    }
}

/// Rakdos the Defiler — {2}{B}{B}{R}{R} 7/6 Legendary Demon with flying and
/// trample. Whenever it attacks, sacrifice half the non-Demon permanents you
/// control, rounded up. Whenever it deals combat damage to a player, that
/// player sacrifices half their non-Demon permanents, rounded up.
pub fn rakdos_the_defiler() -> CardDefinition {
    use crate::card::Supertype;
    let non_demon = || R::Permanent.and(R::HasCreatureType(CreatureType::Demon).negate());
    CardDefinition {
        name: "Rakdos the Defiler",
        cost: cost(&[generic(2), b(), b(), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::SacrificeHalf {
                    who: Selector::You,
                    filter: non_demon(),
                    rounded_up: true,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::SacrificeHalf {
                    who: Selector::Player(PlayerRef::Target(0)),
                    filter: non_demon(),
                    rounded_up: true,
                },
            },
        ],
        ..Default::default()
    }
}

/// Dread Slag — {3}{B}{R} 9/9 Horror with trample. Gets −4/−4 for each card in
/// your hand.
pub fn dread_slag() -> CardDefinition {
    use crate::card::DynamicPt;
    CardDefinition {
        name: "Dread Slag",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::BaseMinusPerCardInHand {
            base_p: 9,
            base_t: 9,
            per: 4,
        }),
        ..Default::default()
    }
}

/// Voidslime — {G}{U}{U} Instant. Counter target spell, activated ability, or
/// triggered ability.
pub fn voidslime() -> CardDefinition {
    CardDefinition {
        name: "Voidslime",
        cost: cost(&[g(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpellOrAbility {
            what: target_filtered(R::IsSpellOnStack.or(R::HasAbilityOnStack)),
        },
        ..Default::default()
    }
}

/// Brain Pry — {1}{B} Sorcery. Choose a nonland card name. Target player
/// reveals their hand and discards a card with that name. If they can't, you
/// draw a card.
pub fn brain_pry() -> CardDefinition {
    CardDefinition {
        name: "Brain Pry",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::NameCardTargetDiscardsOneOrYouDraw,
        ..Default::default()
    }
}

/// Biomantic Mastery — {4}{G/U}{G/U}{G/U} Sorcery. Draw a card for each creature
/// target player controls, then a card for each creature another target player
/// controls.
pub fn biomantic_mastery() -> CardDefinition {
    CardDefinition {
        name: "Biomantic Mastery",
        cost: cost(&[generic(4), g(), g(), g()]), // {G/U}{G/U}{G/U} — green pips
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Sum(vec![
                Value::CreatureCountControlledBy(PlayerRef::Target(0)),
                Value::CreatureCountControlledBy(PlayerRef::Target(1)),
            ]),
        },
        ..Default::default()
    }
}

/// Leafdrake Roost — {3}{G}{U} Aura. Enchant land. Enchanted land has
/// "{G}{U}, {T}: Create a 2/2 green and blue Drake creature token with flying."
pub fn leafdrake_roost() -> CardDefinition {
    let drake = TokenDefinition {
        name: "Drake".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Leafdrake Roost",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Land),
        },
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[g(), u()]),
                tap_cost: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: drake,
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// A Karoo bounce-land (CR — Ravnica): enters tapped, returns a land you
/// control to hand on entry, and taps for two guild colors at once.
fn karoo(name: &'static str, a: Color, b: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![a, b]),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![
            super::super::etb_tap(),
            etb(Effect::Move {
                what: target_filtered(R::Land.and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        ],
        ..Default::default()
    }
}

pub fn azorius_chancery() -> CardDefinition {
    karoo("Azorius Chancery", Color::White, Color::Blue)
}
pub fn rakdos_carnarium() -> CardDefinition {
    karoo("Rakdos Carnarium", Color::Black, Color::Red)
}
pub fn simic_growth_chamber() -> CardDefinition {
    karoo("Simic Growth Chamber", Color::Green, Color::Blue)
}

/// Writ of Passage — {U} Aura. Enchant creature. Whenever enchanted creature
/// attacks, if its power is 2 or less, it can't be blocked this turn.
/// Forecast — {1}{U}: Target creature with power 2 or less can't be blocked
/// this turn.
pub fn writ_of_passage() -> CardDefinition {
    CardDefinition {
        name: "Writ of Passage",
        cost: cost(&[u()]),
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
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::PowerAtMost(2),
                    },
                ),
                effect: Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
            }],
            ..Default::default()
        }),
        activated_abilities: vec![forecast(
            cost(&[generic(1), u()]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        )],
        ..Default::default()
    }
}

/// Prahv, Spires of Order — Land. `{T}: Add {C}.` and `{4}{W}{U}, {T}: Prevent
/// all damage a source of your choice would deal this turn.`
pub fn prahv_spires_of_order() -> CardDefinition {
    CardDefinition {
        name: "Prahv, Spires of Order",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(4), w(), u()]),
                tap_cost: true,
                effect: Effect::PreventAllDamageFromChosenSourceThisTurn { filter: R::Any },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Palliation Accord — {3}{W}{U} Enchantment. Whenever a creature an opponent
/// controls becomes tapped, put a palliation counter on this. Remove a
/// palliation counter: Prevent the next 1 damage that would be dealt to you
/// this turn.
pub fn palliation_accord() -> CardDefinition {
    CardDefinition {
        name: "Palliation Accord",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Palliation,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Palliation, 1)),
            effect: Effect::PreventNextDamage {
                target: Selector::You,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pain Magnification — {1}{B}{R} Enchantment. Whenever an opponent is dealt 3
/// or more damage by a single source, that player discards a card.
pub fn pain_magnification() -> CardDefinition {
    CardDefinition {
        name: "Pain Magnification",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentControl)
                .with_filter(Predicate::ValueAtLeast(
                    Value::TriggerEventAmount,
                    Value::Const(3),
                )),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..Default::default()
    }
}
