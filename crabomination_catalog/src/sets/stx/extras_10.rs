#![allow(unused_imports)]
//! Strixhaven supplemental cards — additions to the base STX catalog
//! that flesh out the set with more castable spells and creatures.
//!
//! Cards added here typically need only existing engine primitives
//! (ETB triggers, simple targeted effects, search/learn). Cards that
//! depend on Mentor/Mutate/Lesson-sideboard primitives ship as their
//! body half only and are marked 🟡 in `STRIXHAVEN2.md`.

use super::super::no_abilities;
use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Effect, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, Selector,
    SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb_drain, etb_gain_life, magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w, ManaCost};

// ── Bookwurm ────────────────────────────────────────────────────────────────

// Bookwurm — {5}{G}{G}, 5/5 Wurm. "Trample / When this creature enters,
// you gain 4 life and draw a card."
//
// ✅ ETB body is a simple `Seq(GainLife(4), Draw(1))`. The 5/5 trample
// body is a fine top-end finisher in any green deck.

/// Quandrix Geometer — {1}{G}{U}, 2/3 Fractal Wizard.
///
/// Synthesised: "When this creature enters, put a +1/+1 counter on
/// target creature you control. Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature gets +1/+1 until end of
/// turn."
pub fn quandrix_geometer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geometer",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            crate::effect::shortcut::magecraft_self_pump(1, 1),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Silverquill Sentinel-Cleric — {2}{W}{B}, 3/3 Inkling Cleric. Flying, Vigilance.
///
/// Synthesised top-end Inkling that closes games defensively. Stacks
/// with Tenured Inkcaster's anthem (3/3 → 5/5 Flying+Vigilance).
pub fn silverquill_sentinel_cleric() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentinel-Cleric",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Soilshaper — {2}{B}{G}, 3/3 Plant Druid.
///
/// Synthesised: "When this creature enters, mill 2 cards. Then put a
/// +1/+1 counter on this creature for each creature card in your
/// graveyard." Gy-self-fill + body that scales with the gy.
pub fn witherbloom_soilshaper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Soilshaper",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Prismari Fireshaper — {2}{U}{R}, 2/3 Elemental Wizard.
///
/// Synthesised: "When this creature enters, create a Treasure token.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature deals 1 damage to any target." A Treasure-mint
/// ramp creature with built-in ping engine.
pub fn prismari_fireshaper() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Fireshaper",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
                },
            },
            magecraft(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Scry-Wizard — {2}{U}, 2/2 Human Wizard.
///
/// Synthesised: "When this creature enters, scry 2. Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, scry 1."
pub fn strixhaven_scry_wizard() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Scry-Wizard",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            },
            magecraft(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Bookbinder — {3}{R}{W}, 4/4 Spirit Cleric.
///
/// Synthesised: "When this creature enters, return target instant or
/// sorcery card from your graveyard to your hand. Each creature you
/// control gains haste until end of turn."
pub fn lorehold_bookbinder() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    CardDefinition {
        name: "Lorehold Bookbinder",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::GrantKeyword {
                    what: each_your_creature(),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Quandrix Wavecaster — {1}{G}{U}, 1/3 Merfolk Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you may put a +1/+1 counter on target creature you
/// control. If you control three or more creatures with +1/+1
/// counters, draw a card."
///
/// The conditional draw rider is omitted as we use the simpler
/// per-cast counter shape (the printed-form conditional needs more
/// expression).
pub fn quandrix_wavecaster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavecaster",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Silverquill Embodiment — {2}{W}{B}, 3/3 Inkling Bard. Flying.
///
/// Synthesised: "When this creature enters, drain 2 (each opp loses
/// 2, you gain 2). Whenever another creature you control dies, you
/// gain 1 life."
pub fn silverquill_embodiment() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Embodiment",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(2),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Plagueweaver — {1}{B}{G}, 2/2 Plant Warlock.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, target creature gets -1/-1 until end of turn."
pub fn witherbloom_plagueweaver() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Plagueweaver",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Mage-Hunter — {2}{B}, 2/3 Human Assassin. Deathtouch.
///
/// Synthesised: "{T}: Target opponent reveals their hand. You choose a
/// nonland card from it. That player discards that card." A repeatable
/// hand-attacker on a deathtouch body.
pub fn strixhaven_mage_hunter() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Strixhaven Mage-Hunter",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: ManaCost::default(),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            sorcery_speed: false,
            condition: None,
            once_per_turn: false,
            from_graveyard: false,
            exile_self_cost: false,
            effect: Effect::DiscardChosen {
                from: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::HasCardType(CardType::Land).negate(),
            },
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Pyresmith — {1}{R}, 2/1 Spirit Warrior. First strike.
///
/// Synthesised: "When this creature enters, deal 1 damage to any
/// target." A first-strike 2/1 with built-in shock.
pub fn lorehold_pyresmith() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Pyresmith",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Prismari Sparkbender — {U}{R}, 2/2 Human Wizard.
///
/// Synthesised: "When this creature enters, loot 1 (Draw 1, discard
/// 1)."
pub fn prismari_sparkbender() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkbender",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Quandrix Mathmage — {2}{G}{U}, 3/3 Elf Wizard.
///
/// Synthesised: "When this creature enters, look at the top 4 cards
/// of your library. You may reveal a creature or land card and put
/// it into your hand. Put the rest on the bottom of your library."
pub fn quandrix_mathmage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mathmage",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Creature
                    .or(SelectionRequirement::HasCardType(CardType::Land)),
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(4),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Silverquill Adjudicator — {3}{W}, 2/4 Human Cleric. Vigilance.
///
/// Synthesised: "When this creature enters, target opponent's creature
/// gets -3/-0 until end of turn."
pub fn silverquill_adjudicator() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Adjudicator",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Drain-Mage — {2}{B}, 2/2 Human Warlock.
///
/// Synthesised: "When this creature enters, target opponent loses 3
/// life and you gain 3 life."
pub fn witherbloom_drain_mage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drain-Mage",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(3)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Pop-Quiz Sage — {2}{W}, 2/3 Human Wizard.
///
/// Synthesised: "When this creature enters, draw two cards, then put
/// a card from your hand on top of your library." Pop-Quiz on a body.
pub fn strixhaven_pop_quiz_sage() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Pop-Quiz Sage",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::PutOnLibraryFromHand {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Spirit-Champion — {3}{R}{W}, 4/3 Spirit Knight. First
/// strike, haste.
///
/// Synthesised: "Other Spirits you control have first strike." Spirit
/// tribal anthem on an attacker body.
pub fn lorehold_spirit_champion() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spirit-Champion",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirits you control have first strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::FirstStrike,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Pest-Spawner — {2}{B}{G}, 1/3 Plant Druid.
///
/// Synthesised: "When this creature enters, create two 1/1 Pest
/// tokens with 'When this creature dies, you gain 1 life.' / Whenever
/// another creature you control dies, you gain 1 life." Pest engine
/// + drain payoff in one card.
pub fn witherbloom_pest_spawner() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pest-Spawner",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: super::shared::stx_pest_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Prismari Wave-Mage — {1}{U}{R}, 2/2 Elemental Wizard.
///
/// Synthesised: "Whenever you cast or copy an instant or sorcery spell,
/// scry 1 and this creature deals 1 damage to any target. / Treasure
/// token ETB."
pub fn prismari_wave_mage() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Wave-Mage",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
                },
            },
            magecraft(Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(1),
                },
            ])),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Quandrix Counterstudent — {1}{U}, 1/2 Elf Wizard.
///
/// Synthesised: "{1}{G}{U}, {T}: Counter target activated ability."
/// Stifle on a Quandrix body — a small repeatable counter for
/// non-mana activations.
pub fn quandrix_counterstudent() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Quandrix Counterstudent",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), g(), u()]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            sorcery_speed: false,
            condition: None,
            once_per_turn: false,
            from_graveyard: false,
            exile_self_cost: false,
            effect: Effect::CounterAbility {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Silverquill Drain-Lord — {2}{W}{B}, 3/3 Inkling Vampire. Lifelink,
/// Flying.
///
/// Synthesised: "Whenever you gain life, target opponent loses 1
/// life." Lifelink-into-drain feedback loop. Pairs with Light of
/// Promise / Honor Troll / Pest tokens for runaway drain.
pub fn silverquill_drain_lord() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drain-Lord",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Push (modern_decks) batch 28: 5 more shared/multi-college cards ────────
//
// Cross-school cards using existing primitives. No new engine features
// required.

/// Strixhaven Battle-Cleric — {W}, 2/1 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, you gain 1
/// life."
///
/// 1-mana white aggressive body + lifegain rider. Slots into Light of
/// Promise / Inkling Bloodscribe shells.
pub fn strixhaven_battle_cleric() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Battle-Cleric",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Researcher — {2}{U}, 2/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2."
///
/// 3-mana sticky body + smoothing. Slots into any blue mid-curve shell.
pub fn strixhaven_researcher() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Researcher",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Combatant — {1}{R}, 2/2 Human Warrior.
///
/// Printed Oracle (synthesised): "Haste. When this creature attacks, it
/// gets +1/+0 until end of turn."
///
/// 2-mana hasty self-pumper. Each combat keeps it at 3/2 while connecting.
pub fn strixhaven_combatant() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Combatant",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Druid — {1}{G}, 2/2 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, search your
/// library for a basic land card, reveal it, put it into your hand, then
/// shuffle."
///
/// 2-mana ramp + body. Functionally a green Sakura-Tribe-Elder on landing.
pub fn strixhaven_druid() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Druid",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Drainsong — {1}{B}, instant.
///
/// Printed Oracle (synthesised): "Target opponent loses 2 life and you
/// gain 2 life."
///
/// 2-mana straight drain. Witherbloom's "Drain Life" template — slots
/// alongside Drain 2 / Drain 3 effects.
pub fn strixhaven_drainsong() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Drainsong",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::Target(0)),
            to: Selector::You,
            amount: Value::Const(2),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Batch 32 — Additional STX extras ────────────────────────────────────────

/// Strixhaven Honor Guard — {1}{W}, 2/2 Human Soldier Vigilance.
///
/// Synthesised Oracle: "When this creature enters, gain 1 life. Whenever
/// you gain life, this creature gets +0/+1 until end of turn."
///
/// 2-mana sticky lifegain enabler. Pairs with Light of Promise / Felisa for
/// chained scaling.
pub fn strixhaven_honor_guard() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Honor Guard",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_gain_life(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(0),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Sapper — {1}{B}, 1/2 Human Rogue Menace.
///
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life."
pub fn strixhaven_sapper() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Sapper",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Cartographer — {1}{G}, 1/2 Elf Druid.
///
/// Synthesised Oracle: "When this creature enters, look at the top three
/// cards of your library. You may reveal a land card from among them and
/// put it into your hand. Put the rest on the bottom of your library in a
/// random order."
pub fn strixhaven_cartographer_b32() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Cartographer",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::HasCardType(CardType::Land),
                cap: Value::Const(3),
                to: ZoneDest::Hand(PlayerRef::You),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Glyphmage — {2}{U}, 2/3 Human Wizard.
///
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, scry 1."
pub fn strixhaven_glyphmage() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Glyphmage",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Batch 33: 5 cross-school extras ────────────────────────────────────

/// Strixhaven Mentor — {2}{W}, 2/3 Human Cleric Vigilance.
/// Synthesised Oracle: "Vigilance / When this creature enters, another
/// target creature you control gets a +1/+1 counter."
pub fn strixhaven_mentor() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Mentor",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Banner — {3}, Artifact.
/// Synthesised Oracle: "{T}: Add one mana of any color. /
/// {2}, {T}, Sacrifice this artifact: Draw a card."
pub fn strixhaven_banner() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Banner",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {T}: Add one mana of any color.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                mana_cost: ManaCost::default(),
                tap_cost: true,
                sac_cost: false,
                life_cost: 0,
                exile_other_filter: None,
                condition: None,
                exile_self_cost: false,
                from_graveyard: false,
                sorcery_speed: false,
                once_per_turn: false,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                self_counter_cost_reduction: None, sac_other_filter: None,
                tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
            // {2}, {T}, Sacrifice this artifact: Draw a card.
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                life_cost: 0,
                exile_other_filter: None,
                condition: None,
                exile_self_cost: false,
                from_graveyard: false,
                sorcery_speed: false,
                once_per_turn: false,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                self_counter_cost_reduction: None, sac_other_filter: None,
                tap_other_filter: None, from_hand: false,
                ..Default::default()
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Apprentice — {1}{U}, 1/2 Human Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn strixhaven_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Apprentice",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Sorcerer — {3}{R}, 3/3 Human Wizard Haste.
/// Synthesised Oracle: "Haste / When this creature enters, it deals 2
/// damage to any target."
pub fn strixhaven_sorcerer() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Sorcerer",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Pupil — {2}, 1/1 Human Wizard Artifact Creature.
/// Synthesised Oracle: "{2}, {T}: Scry 1, then draw a card."
pub fn strixhaven_pupil() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Pupil",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Field Researcher — {2}{G}, 2/3 Human Druid.
/// Original Oracle: "When this creature enters, put a +1/+1 counter on
/// each creature you control."
pub fn strixhaven_field_researcher() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Field Researcher",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Batch 47 (modern_decks) — Strixhaven extras ─────────────────────────────

/// Strixhaven Quartermaster — {1}{W}, 2/2 Human Soldier.
/// Synthesised Oracle: "Vigilance. When this creature enters, you
/// gain 2 life."
pub fn strixhaven_quartermaster() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Quartermaster",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Library Mage — {2}{U}, 2/3 Human Wizard.
/// Synthesised Oracle: "When this creature enters, scry 2."
pub fn strixhaven_library_mage() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Library Mage",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_scry(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Demonstrator — {2}{B}, 3/2 Human Warlock.
/// Synthesised Oracle: "When this creature enters, each opponent loses
/// 2 life and you gain 2 life."
pub fn strixhaven_demonstrator() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Demonstrator",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb_drain(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Crucible — {3}, Artifact. Synthesised Oracle:
/// "{2}, {T}: Target player loses 1 life and you gain 1 life."
/// A slow-burn drain engine artifact.
pub fn strixhaven_crucible() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Crucible",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
                amount: Value::Const(1),
            },
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Anthemcaster — {3}{W}, 2/3 Human Soldier.
/// Synthesised Oracle: "Other creatures you control get +1/+0."
/// Anthem of Order — a 4-mana lord whose static pump fires on every
/// other friendly creature, including future ETBs.
pub fn strixhaven_anthemcaster() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Anthemcaster",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Stormsage — {2}{U}, 2/2 Human Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card."
/// Cantrip body. 3-mana hand-refilling Wizard.
pub fn strixhaven_stormsage() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Stormsage",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Strixhaven Skylancer — {3}{W}, 3/3 Human Knight Flying + Vigilance.
/// Synthesised Oracle: "Flying, vigilance."
pub fn strixhaven_skylancer() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Skylancer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── modern_decks batch 103: 25 new Strixhaven synthesised cards ─────────────
//
// A batch of fresh Strixhaven-themed synthesised cards across all five
// colleges, expanding the existing per-college pools with new common /
// uncommon shapes. Each card is built on the existing engine
// primitives (magecraft, etb, drain, treasure mint, pest mint, spirit
// mint, fractal mint) — no new engine work required.
//
// The batch breaks down 5 cards per college: Silverquill (W/B),
// Witherbloom (B/G), Lorehold (R/W), Prismari (U/R), Quandrix (G/U).

// ── Silverquill (W/B) — 5 new cards ─────────────────────────────────────────

/// Silverquill Ledgerkeeper — {2}{W}{B}, 2/3 Inkling Cleric Flying.
///
/// Synthesised: "Flying. When this creature enters, each opponent
/// loses 2 life and you gain 2 life."
pub fn silverquill_ledgerkeeper() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ledgerkeeper",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Inkling Aerospread — {3}{W}{B}, 3/3 Inkling Soldier Flying.
///
/// Synthesised: "Flying. When this creature enters, create a 1/1
/// white-and-black Inkling creature token with flying."
pub fn inkling_aerospread() -> CardDefinition {
    use crate::effect::shortcut::{etb, mint_inklings};
    CardDefinition {
        name: "Inkling Aerospread",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(mint_inklings(1))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Silverquill Brushmage — {1}{W}, 2/1 Human Cleric.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+1 until end of turn."
pub fn silverquill_brushmage() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Brushmage",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Inkling Glaivemaster — {2}{B}, 2/2 Inkling Wizard.
///
/// Synthesised: "Whenever this creature attacks, each opponent loses
/// 1 life and you gain 1 life."
pub fn inkling_glaivemaster() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Inkling Glaivemaster",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Silverquill Lecturer (batch 103) — {1}{W} Sorcery.
///
/// Synthesised: "Each opponent loses 2 life and you gain 2 life."
/// Simple drain spell built on the existing `Effect::Drain` primitive.
pub fn silverquill_lecturer_b103() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lecturer (Batch 103)",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(2),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Witherbloom (B/G) — 5 new cards ─────────────────────────────────────────

/// Witherbloom Necromage — {2}{B}{G}, 2/3 Human Warlock.
///
/// Synthesised: "When this creature enters, create a 1/1 black-and-
/// green Pest creature token with 'When this creature dies, you gain
/// 1 life.' When this creature dies, each opponent loses 2 life and
/// you gain 2 life."
pub fn witherbloom_necromage() -> CardDefinition {
    use super::shared::stx_pest_token;
    use crate::effect::shortcut::{dies_drain, etb};
    CardDefinition {
        name: "Witherbloom Necromage",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
            }),
            dies_drain(2),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Toxinsage — {1}{B}{G}, 2/2 Plant Warlock.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, each opponent loses 1 life and you gain 1 life."
pub fn witherbloom_toxinsage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Toxinsage",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Wildmage — {3}{B}{G}, 3/4 Human Warlock.
///
/// Synthesised: "When this creature enters, you gain 3 life.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// scry 1."
pub fn witherbloom_wildmage() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Witherbloom Wildmage",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(3), magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Pestcaller (batch 103) — {2}{B}, 2/2 Human Warlock.
///
/// Synthesised: "When this creature enters, create a 1/1 black-and-
/// green Pest creature token with 'When this creature dies, you gain
/// 1 life.'"
pub fn witherbloom_pestcaller_b103() -> CardDefinition {
    use super::shared::stx_pest_token;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Witherbloom Pestcaller (Batch 103)",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: stx_pest_token(),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Witherbloom Lecturer — {1}{B}{G} Sorcery.
///
/// Synthesised: "Each opponent loses 3 life and you gain 3 life."
pub fn witherbloom_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lecturer",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Lorehold (R/W) — 5 new cards ────────────────────────────────────────────

/// Lorehold Battlemage (batch 103) — {2}{R}{W}, 2/3 Human Wizard.
///
/// Synthesised: "When this creature enters, it deals 2 damage to any
/// target. Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, create a 2/2 red-and-white Spirit creature token."
pub fn lorehold_battlemage_b103() -> CardDefinition {
    use super::lorehold::lorehold_spirit_token;
    use crate::effect::shortcut::{etb, magecraft};
    CardDefinition {
        name: "Lorehold Battlemage (Batch 103)",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            }),
            magecraft(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Embertusk — {3}{R}{W}, 3/3 Spirit Beast.
///
/// Synthesised: "When this creature enters, it deals 1 damage to each
/// opponent and you create a 2/2 red-and-white Spirit creature token."
pub fn lorehold_embertusk() -> CardDefinition {
    use super::lorehold::lorehold_spirit_token;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Embertusk",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(1),
                }),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Pyrescholar (batch 103) — {2}{R}, 2/2 Human Wizard.
///
/// Synthesised: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature deals 1 damage to any target."
pub fn lorehold_pyrescholar_b103() -> CardDefinition {
    use crate::effect::shortcut::magecraft_ping_any;
    CardDefinition {
        name: "Lorehold Pyrescholar (Batch 103)",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_ping_any(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Spirit Reclaimer — {3}{W}, 2/3 Spirit Cleric.
///
/// Synthesised: "When this creature enters, return target creature
/// card from your graveyard to your hand."
pub fn lorehold_spirit_reclaimer() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Lorehold Spirit Reclaimer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                },
                Value::Const(1),
            ),
            to: crate::effect::ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Lorehold Lecturer — {1}{R}{W} Sorcery.
///
/// Synthesised: "Deal 2 damage to any target. Create a 2/2 red-and-
/// white Spirit creature token."
pub fn lorehold_lecturer() -> CardDefinition {
    use super::lorehold::lorehold_spirit_token;
    CardDefinition {
        name: "Lorehold Lecturer",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

// ── Prismari (U/R) — 5 new cards ────────────────────────────────────────────

/// Prismari Sparkblade — {1}{U}{R}, 2/2 Elemental Wizard.
///
/// Synthesised: "When this creature enters, create a Treasure token.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// scry 1."
pub fn prismari_sparkblade() -> CardDefinition {
    use crate::effect::shortcut::{etb, magecraft_scry};
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Prismari Sparkblade",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            }),
            magecraft_scry(1),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}

/// Prismari Aetherweaver — {3}{U}{R}, 3/3 Elemental Wizard.
///
/// Synthesised: "When this creature enters, draw a card, then discard
/// a card. Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature deals 1 damage to any target."
pub fn prismari_aetherweaver() -> CardDefinition {
    use crate::effect::shortcut::{etb, magecraft_ping_any};
    CardDefinition {
        name: "Prismari Aetherweaver",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
            ])),
            magecraft_ping_any(1),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
    }
}
