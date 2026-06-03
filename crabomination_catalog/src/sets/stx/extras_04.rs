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

// ── Sigardian Savior (synthesised STX 2021 white finisher) ────────────────

/// Sigardian Savior — {3}{W}{W}, 4/4 Angel (synthesised STX
/// white-tribal flavor). "Flying. When this creature enters, return
/// up to two target creature cards with mana value 3 or less from
/// your graveyard to the battlefield."
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana flying body
/// with a 2-for-1 reanimation rider. The "up to two" multi-target is
/// approximated as a single target return (engine-wide multi-target
/// gap). Wired via ETB `Effect::Move` against a creature card in
/// your graveyard with `ManaValueAtMost(3)`. Tests:
/// `sigardian_savior_is_a_five_mana_four_four_flying_angel`,
/// `sigardian_savior_etb_returns_low_mv_creature_card`.
pub fn sigardian_savior() -> CardDefinition {
    CardDefinition {
        name: "Sigardian Savior",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Sneaky Snacker (synthesised STX Witherbloom common) ───────────────────

/// Sneaky Snacker — {B}, 1/1 Rat Rogue (synthesised STX Witherbloom
/// flavor). "Menace. {2}{B}: Return Sneaky Snacker from your
/// graveyard to your hand. Activate only as a sorcery."
///
/// Push (modern_decks, NEW, `stx::extras`): One-mana evasive body
/// with built-in graveyard recursion. Wired via a `from_graveyard:
/// true` activated ability (engine primitive added in push XVII for
/// Summoned Dromedary) with `sorcery_speed: true`. Tests:
/// `sneaky_snacker_is_a_one_mana_rat_with_menace`,
/// `sneaky_snacker_recurs_from_graveyard_to_hand`.
pub fn sneaky_snacker() -> CardDefinition {
    CardDefinition {
        name: "Sneaky Snacker",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Soulknife Spy (synthesised STX Quandrix/blue Rogue) ───────────────────

/// Soulknife Spy — {1}{U}, 1/3 Human Rogue (synthesised STX flavor).
/// "Whenever Soulknife Spy deals combat damage to a player, you may
/// pay {U}. If you do, draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Card-velocity attacker
/// that turns each combat hit into a {U} → draw exchange. Wired via
/// `DealsCombatDamageToPlayer/SelfSource` trigger + `Effect::MayPay`
/// for the optional card draw. Tests:
/// `soulknife_spy_is_a_two_mana_one_three_rogue`,
/// `soulknife_spy_combat_damage_can_draw_a_card_via_scripted_decider`,
/// `soulknife_spy_combat_damage_no_pay_skips_draw`.
pub fn soulknife_spy() -> CardDefinition {
    CardDefinition {
        name: "Soulknife Spy",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::MayPay {
                description: "Pay {U} to draw a card.".into(),
                mana_cost: cost(&[u()]),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix Cryptomancer (synthesised STX Quandrix Rogue) ────────────────

/// Quandrix Cryptomancer — {2}{U}, 2/2 Vedalken Wizard (synthesised STX
/// flavor). "Whenever this attacks, connive." (CR 702.158 — draw a card,
/// discard a card; a +1/+1 counter for each nonland pitched.)
///
/// Showcases the new `shortcut::connive` action word built on
/// Draw + Discard + `DiscardedThisResolution`-counted +1/+1 counters.
pub fn quandrix_cryptomancer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cryptomancer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: crate::effect::shortcut::connive(1),
        }],
        ..Default::default()
    }
}

// ── Daring Diversion (synthesised STX 2021 Lorehold Sorcery) ──────────────

/// Daring Diversion — {3}{R} Sorcery (synthesised STX Lorehold flavor).
/// "Daring Diversion deals 2 damage to each of two target creatures."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-damage divided burn
/// spell that hits two creatures. The "divided as you choose" rider
/// is collapsed to "2 damage to each" (slot 0 + slot 1 multi-target,
/// each getting 2 damage). The full "divided" semantics need the
/// engine-wide DealDamageDivided primitive. Tests:
/// `daring_diversion_burns_one_creature`,
/// `daring_diversion_burns_two_creatures_via_multi_target`,
/// `daring_diversion_is_a_four_mana_red_sorcery`.
pub fn daring_diversion() -> CardDefinition {
    CardDefinition {
        name: "Daring Diversion",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
                amount: Value::Const(2),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Possibility Storm (synthesised STX Prismari reprint flavor) ───────────

/// Possibility Storm — {2}{R} Enchantment (synthesised STX Prismari
/// reprint flavor of Lorwyn). "Whenever a player casts a spell from
/// their hand, that player exiles it, then exiles cards from the top
/// of their library until they exile a card that shares a card type
/// with it. That player may cast that card without paying its mana
/// cost. Then they put all cards exiled with this enchantment on the
/// bottom of their library in a random order."
///
/// Push (modern_decks, NEW, `stx::extras`): The full Oracle requires
/// a complex deferred-cast-from-exile primitive. We ship the body
/// (enchantment + no triggered ability) as a chaos-engine placeholder
/// that toggles a marker on the battlefield for future engine work.
/// Currently a vanilla enchantment frame; will get its trigger when
/// the cast-from-exile pipeline lands. Tests:
/// `possibility_storm_is_a_three_mana_red_enchantment`.
pub fn possibility_storm() -> CardDefinition {
    CardDefinition {
        name: "Possibility Storm",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pilgrim of the Ages (synthesised STX-flavor archetype anchor) ─────────

/// Pilgrim of the Ages — {3}, 1/1 Spirit (synthesised STX colorless
/// utility creature). "{2}, Sacrifice this creature: Search your
/// library for a basic land card, reveal it, put it into your hand,
/// then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): Colorless ramp on a body —
/// useful in any deck that wants color-fixing without committing to
/// a color. Wired via `sac_cost: true` activated ability + `Effect::
/// Search` for a basic land. Tests:
/// `pilgrim_of_the_ages_is_a_three_mana_one_one_spirit`,
/// `pilgrim_of_the_ages_sac_searches_for_basic_land`.
pub fn pilgrim_of_the_ages() -> CardDefinition {
    CardDefinition {
        name: "Pilgrim of the Ages",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Strixhaven Spawner (synthesised STX-flavor token doubler aid) ─────────

/// Strixhaven Spawner — {3}{G}{U} Sorcery (synthesised STX Quandrix
/// flavor). "Create three 0/0 green and blue Fractal creature tokens.
/// Put two +1/+1 counters on each of them."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix mass token-mint +
/// counter-pump. Each Fractal lands as a 2/2 (printed 0/0 + 2 counters).
/// With Adrix and Nev on the battlefield, the count doubles to six 2/2s.
/// Tests: `strixhaven_spawner_creates_three_fractal_tokens`,
/// `strixhaven_spawner_fractals_have_two_plus_one_counters`,
/// `strixhaven_spawner_is_a_five_mana_gu_sorcery`.
pub fn strixhaven_spawner() -> CardDefinition {
    let fractal_def = TokenDefinition {
        name: "Fractal".to_string(),
        colors: vec![Color::Green, Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        supertypes: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Strixhaven Spawner",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: fractal_def,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Mage Hunter Defender (synthesised STX-flavor wall) ────────────────────

/// Mage Hunter Defender — {2}{B}, 2/3 Human Wizard with Defender
/// (synthesised STX Witherbloom flavor). "Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, each opponent loses 1
/// life."
///
/// Push (modern_decks, NEW, `stx::extras`): A Defender Magecraft
/// drainer — sits behind the lines and pings as you cast spells.
/// Wired via the `magecraft_drain_each_opp(1)` shortcut. Tests:
/// `mage_hunter_defender_drains_on_instant_cast`,
/// `mage_hunter_defender_is_a_three_mana_defender_wizard`.
pub fn mage_hunter_defender() -> CardDefinition {
    CardDefinition {
        name: "Mage Hunter Defender",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Detention Sphere (synthesised STX-flavor enchantment) ─────────────────

/// Detention Sphere — {1}{W}{U} Enchantment (synthesised STX
/// Silverquill-aligned hybrid removal). "When this enchantment
/// enters, exile target nonland permanent."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana hybrid exile-on-
/// ETB. The printed "until this leaves" return rider is omitted (no
/// exile-until-leaves replacement primitive — same gap as Banisher
/// Priest / Tidehollow Sculler). Tests:
/// `detention_sphere_exiles_target_nonland_permanent`,
/// `detention_sphere_is_a_three_mana_white_blue_enchantment`.
pub fn detention_sphere() -> CardDefinition {
    CardDefinition {
        name: "Detention Sphere",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Mascot Trainer (synthesised STX-flavor anthem) ────────────────────────

/// Mascot Trainer — {2}{G}, 2/2 Human Druid (synthesised STX
/// Witherbloom/Quandrix tribal anthem). "Other Mascot tokens you
/// control get +1/+1."
///
/// Push (modern_decks, NEW, `stx::extras`): The Mascot Exhibition
/// tokens (Elephant, Cat, Bird, plus Strixhaven Stadium's Inkling)
/// don't have a unified subtype, so this card uses the same
/// `OtherThanSource` anthem pattern as Tenured Inkcaster but applies
/// to all friendly tokens. Wired via `PumpPT` static against
/// `EachPermanent(Creature ∧ ControlledByYou ∧ IsToken ∧
/// OtherThanSource)`. Tests:
/// `mascot_trainer_buffs_friendly_tokens`,
/// `mascot_trainer_does_not_buff_non_tokens`,
/// `mascot_trainer_is_a_three_mana_two_two_druid`.
pub fn mascot_trainer() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Mascot Trainer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other tokens you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsToken)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix Cryptidkeeper (synthesised STX-flavor) ───────────────────────

/// Quandrix Cryptidkeeper — {2}{G}{U}, 3/3 Elf Druid (synthesised STX
/// Quandrix flavor). "When this creature enters, put two +1/+1
/// counters on another target creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Mid-curve counter-anchor
/// body — pairs with Quandrix counter doublers and Practical Research.
/// Wired via ETB `Effect::AddCounter(+1/+1, ×2)` on a `Creature ∧
/// ControlledByYou ∧ OtherThanSource` target. Tests:
/// `quandrix_cryptidkeeper_etb_pumps_friendly`,
/// `quandrix_cryptidkeeper_is_a_four_mana_three_three_elf_druid`.
pub fn quandrix_cryptidkeeper() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cryptidkeeper",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Ember Anvil (synthesised STX-flavor Lorehold mana rock) ──────────────

/// Ember Anvil — {3} Artifact (synthesised STX Lorehold-flavor mana
/// rock). "{T}: Add {R} or {W}. / {3}, {T}, Sacrifice this artifact:
/// Search your library for a Spirit card, reveal it, put it into
/// your hand, then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): A Spirit tribal tutor on
/// a mana-rock body. Wired with two `tap_add` activations (R + W) and
/// a `sac_cost: true` Search activation. Tests:
/// `ember_anvil_taps_for_white_or_red`,
/// `ember_anvil_sac_tutors_a_spirit_creature_into_hand`,
/// `ember_anvil_is_a_three_mana_artifact`.
pub fn ember_anvil() -> CardDefinition {
    CardDefinition {
        name: "Ember Anvil",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            super::super::tap_add(Color::Red),
            super::super::tap_add(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: true,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Witherbloom Strangler (synthesised STX-flavor Witherbloom) ────────────

/// Witherbloom Strangler — {1}{B}{G}, 2/2 Plant Warlock (synthesised
/// STX Witherbloom flavor). "When this creature enters, target
/// creature an opponent controls gets -2/-2 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana removal-on-a-body
/// — kills 2-toughness creatures on ETB. Wired via ETB `Effect::PumpPT
/// {-2, -2, EOT}` on a `Creature ∧ ControlledByOpponent` target.
/// Tests: `witherbloom_strangler_etb_minus_two_minus_two`,
/// `witherbloom_strangler_kills_two_two_creature`,
/// `witherbloom_strangler_is_a_three_mana_two_two_plant_warlock`.
pub fn witherbloom_strangler() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Strangler",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Glasspool Embellisher (synthesised STX-flavor Quandrix utility) ──────

/// Glasspool Embellisher — {U} Instant (synthesised STX Quandrix
/// flavor). "Draw a card, then discard a card. Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): A {U} looter-cantrip with
/// no Magecraft body (it's an instant, not a creature). Just a card
/// filtering spell. Wired as `Seq(Draw 1, Discard 1)`. Tests:
/// `glasspool_embellisher_loots_one`,
/// `glasspool_embellisher_is_a_one_mana_blue_instant`.
pub fn glasspool_embellisher() -> CardDefinition {
    CardDefinition {
        name: "Glasspool Embellisher",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Lorehold Reanimator (synthesised STX-flavor Lorehold archetype) ──────

/// Lorehold Reanimator — {2}{R}{W}, 3/3 Spirit Cleric (synthesised
/// STX Lorehold flavor). "When this creature enters, you may return
/// target creature card with mana value 2 or less from your graveyard
/// to the battlefield."
///
/// Push (modern_decks, NEW, `stx::extras`): A small reanimator body
/// that brings back a 1- or 2-drop on ETB. Wired via ETB `Effect::
/// MayDo` wrapping a `Move(creature in gy with MV≤2 → battlefield)`.
/// Tests: `lorehold_reanimator_etb_optionally_returns_low_mv_creature`,
/// `lorehold_reanimator_is_a_four_mana_three_three_spirit_cleric`.
pub fn lorehold_reanimator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reanimator",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return a creature card with mana value 2 or less from your graveyard to the battlefield."
                    .into(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Prismari Eruption (synthesised STX-flavor mass burn) ──────────────────

/// Prismari Eruption — {3}{U}{R} Sorcery (synthesised STX Prismari
/// flavor). "Prismari Eruption deals 2 damage to each creature without
/// flying. Scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): A small Prismari sweep
/// that filters lots of x/2 attackers while leaving fliers alive.
/// Wired via `ForEach(EachPermanent(Creature ∧ ¬HasKeyword(Flying)))
/// → DealDamage 2` followed by Scry 1. Tests:
/// `prismari_eruption_burns_grounded_creatures`,
/// `prismari_eruption_spares_flyers`,
/// `prismari_eruption_is_a_five_mana_ur_sorcery`.
pub fn prismari_eruption() -> CardDefinition {
    CardDefinition {
        name: "Prismari Eruption",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature.and(
                    SelectionRequirement::Not(Box::new(SelectionRequirement::HasKeyword(
                        Keyword::Flying,
                    ))),
                )),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(2),
                }),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Silverquill Inquisitor (synthesised STX-flavor) ───────────────────────

/// Silverquill Inquisitor — {1}{W}{B}, 2/2 Human Cleric (synthesised
/// STX Silverquill flavor). "When this creature enters, target
/// opponent reveals their hand. You choose a noncreature, nonland
/// card from it. That player discards that card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana hand-disruption
/// body — approximates Thoughtseize on a body. The "look at hand and
/// choose" is engine-wide ⏳ (no opp-hand-reveal-and-pick primitive);
/// we approximate with `Effect::Discard { random: true }` against the
/// chosen opp. The auto-decider picks the first opp; a true
/// implementation would surface a `Decision::ChooseFromHand` modal.
/// Tests: `silverquill_inquisitor_etb_discards_from_opp_hand`,
/// `silverquill_inquisitor_is_a_three_mana_two_two_cleric`.
pub fn silverquill_inquisitor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inquisitor",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: true,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Lorehold Spectral Lecturer (synthesised STX-flavor Lorehold) ─────────

/// Lorehold Spectral Lecturer — {3}{R}{W}, 4/3 Spirit Cleric Wizard
/// (synthesised STX Lorehold flavor). "Vigilance. / Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, this
/// creature gets +1/+0 and gains lifelink until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana Lorehold
/// Magecraft body — gets bigger and gains lifelink each cast. Wired
/// via `magecraft(Seq(PumpPT(+1/+0 EOT), GrantKeyword(Lifelink EOT)))`
/// on `Selector::This`. Tests:
/// `lorehold_spectral_lecturer_magecraft_pumps_and_lifelinks`,
/// `lorehold_spectral_lecturer_is_a_five_mana_four_three_spirit_cleric_wizard`.
pub fn lorehold_spectral_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectral Lecturer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spirit,
                CreatureType::Cleric,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pop Quiz Recital (synthesised STX-flavor Lesson) ──────────────────────

/// Pop Quiz Recital — {2}{W} Sorcery — Lesson (synthesised STX
/// Lesson flavor). "Choose one — / • Target creature gets +2/+2 and
/// gains flying until end of turn. / • Target creature gets +0/+3
/// and gains vigilance until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A flexible combat-trick
/// Lesson via `Effect::ChooseMode`. Mode 0 (pump+flying) wins in air;
/// mode 1 (toughness+vigilance) wins on the ground. Tests:
/// `pop_quiz_recital_mode_zero_pumps_and_grants_flying`,
/// `pop_quiz_recital_is_a_three_mana_white_lesson`.
pub fn pop_quiz_recital() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz Recital",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(0),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Diviner's Wand (synthesised STX-flavor Equipment) ─────────────────────

/// Diviner's Wand — {4} Artifact — Equipment (synthesised STX-flavor).
/// Equipped creature gets +2/+1 and has flying. Equip {3}.
pub fn diviners_wand() -> CardDefinition {
    CardDefinition {
        name: "Diviner's Wand",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(crate::card::EquipBonus {
            power: 2,
            toughness: 1,
            keywords: vec![Keyword::Flying],
        }),
        ..Default::default()
    }
}

// ── Stonebound Mentor (already exists) – stub note removed ────────────────

// ── Fascinating Lecture (synthesised STX-flavor Lesson) ───────────────────

/// Fascinating Lecture — {1}{U} Sorcery — Lesson (synthesised STX
/// Lesson flavor). "Draw two cards, then discard a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana looter Lesson —
/// strong card velocity tutored from the Lessons sideboard once that
/// primitive lands. For now slots into any blue deck as a +1 net card
/// hand-filter. Wired as `Seq(Draw 2, Discard 1)`. Tests:
/// `fascinating_lecture_draws_two_discards_one`,
/// `fascinating_lecture_is_a_two_mana_blue_lesson`.
pub fn fascinating_lecture() -> CardDefinition {
    CardDefinition {
        name: "Fascinating Lecture",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix Sphinx (synthesised STX-flavor Quandrix flyer) ───────────────

/// Quandrix Sphinx — {3}{G}{U}, 3/4 Sphinx Druid (synthesised STX
/// Quandrix flavor). "Flying. When this creature enters, put a +1/+1
/// counter on each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana flying mass-pump
/// body — Quandrix's signature counter shape on a relevant attacker.
/// Wired via ETB `ForEach(Creature & ControlledByYou) → AddCounter`.
/// Tests: `quandrix_sphinx_etb_counters_each_friendly_creature`,
/// `quandrix_sphinx_is_a_five_mana_three_four_flying_sphinx_druid`.
pub fn quandrix_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sphinx",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Lorehold Excavator (synthesised STX-flavor) ───────────────────────────

/// Lorehold Excavator — {1}{R}{W}, 2/2 Spirit Cleric (synthesised STX
/// Lorehold flavor). "When this creature enters, exile target card
/// from a graveyard. / {2}{R}{W}, {T}: This creature deals 1 damage
/// to any target."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana gy-hate body
/// with an activated ping. Wired with an ETB exile-target-gy-card
/// trigger + tap activation. Tests:
/// `lorehold_excavator_etb_exiles_target_gy_card`,
/// `lorehold_excavator_is_a_three_mana_two_two_spirit_cleric`.
pub fn lorehold_excavator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Excavator",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                }),
                to: ZoneDest::Exile,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Stridehollow Vampire (synthesised STX Silverquill modal pump) ─────────

/// Stridehollow Vampire — {1}{W}{B}, 2/2 Vampire Soldier (synthesised
/// STX Silverquill flavor). "Flying. / When this creature enters,
/// choose one — / • Draw a card. / • Target creature gets +1/+1
/// until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana evasive modal
/// flier. ETB ChooseMode picks draw or pump. AutoDecider picks mode
/// 0 (draw). Tests:
/// `stridehollow_vampire_etb_default_draws`,
/// `stridehollow_vampire_is_a_three_mana_two_two_vampire`.
pub fn stridehollow_vampire() -> CardDefinition {
    CardDefinition {
        name: "Stridehollow Vampire",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Witherbloom Necrotutor (synthesised STX-flavor Witherbloom) ───────────

/// Witherbloom Necrotutor — {2}{B}{B}, 3/2 Human Warlock (synthesised
/// STX Witherbloom flavor). "When this creature enters, return target
/// creature card from your graveyard to your hand. You lose 2 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Pay-life Raise Dead on a
/// body. Wired as `Seq(Move(creature in gy → hand), LoseLife 2)`.
/// Tests: `witherbloom_necrotutor_etb_returns_creature_card_and_loses_two_life`,
/// `witherbloom_necrotutor_is_a_four_mana_three_two_warlock`.
pub fn witherbloom_necrotutor() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necrotutor",
        cost: cost(&[generic(2), b(), b()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(2),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Silverquill Pledge ──────────────────────────────────────────────────────

/// Silverquill Pledge — {1}{W}{B} Instant (synthesised STX Silverquill
/// flavor based on the printed Silverquill Cleric tradition).
///
/// "Target creature gets +3/+1 until end of turn." Pure combat trick on
/// the Silverquill color identity — Inkrise Infiltrator's swing
/// becomes a 5/2 with menace; Eager First-Year goes from a 2/1 to a
/// 5/2 trader. Wired as `Effect::PumpPT { +3/+1, EOT }`. Tests:
/// `silverquill_pledge_pumps_target_three_one`,
/// `silverquill_pledge_is_a_three_mana_silverquill_instant`.
pub fn silverquill_pledge() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pledge",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(3),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkwell Strider ─────────────────────────────────────────────────────────

/// Inkwell Strider — {2}{W}{B} 2/3 Inkling Soldier with Flying + Lifelink
/// (synthesised STX Silverquill flavor — combines the school's two
/// signature keywords on a fair body).
///
/// Provides reach + life-gain for Silverquill payoffs (Promising
/// Duskmage's Magecraft, Light of Promise's counter rain, Tenured
/// Inkcaster's anthem). Tests:
/// `inkwell_strider_is_a_four_mana_two_three_flying_inkling`,
/// `inkwell_strider_combat_grants_life_via_lifelink`.
pub fn inkwell_strider() -> CardDefinition {
    CardDefinition {
        name: "Inkwell Strider",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Scolding Detention ──────────────────────────────────────────────────────

/// Scolding Detention — {2}{W} Sorcery (synthesised STX Silverquill
/// flavor). "Tap target creature an opponent controls. Put two stun
/// counters on it."
///
/// A heavier-handed Frost Trickster — clears two combat steps. The
/// stun-counter SBA gates removal of one counter per failed untap, so
/// the target stays tapped for two of the controller's untaps. Wired
/// as `Seq(Tap target, AddCounter Stun × 2)`. Tests:
/// `scolding_detention_taps_and_stuns_twice`.
pub fn scolding_detention() -> CardDefinition {
    CardDefinition {
        name: "Scolding Detention",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(2),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Lesson Recall ───────────────────────────────────────────────────────────

/// Lesson Recall — {1}{U} Instant (synthesised STX Strixhaven flavor).
/// "Return target instant or sorcery card from your graveyard to your
/// hand. Draw a card."
///
/// Card-advantage cantrip that recurs spells. Wired as
/// `Seq(Move(target IS in your gy → hand), Draw 1)`. Tests:
/// `lesson_recall_returns_instant_and_cantrips`,
/// `lesson_recall_is_a_two_mana_blue_instant`.
pub fn lesson_recall() -> CardDefinition {
    CardDefinition {
        name: "Lesson Recall",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pestilent Acolyte ───────────────────────────────────────────────────────

/// Pestilent Acolyte — {2}{B} 2/3 Human Warlock (synthesised STX
/// Witherbloom flavor). "When this creature enters, target creature
/// gets -1/-1 until end of turn."
///
/// ETB removal-on-a-stick — kills X/1 creatures and softens 2/3s for
/// trading. Wired as `Effect::PumpPT { -1/-1, EOT }` on a creature
/// target. Tests:
/// `pestilent_acolyte_etb_kills_one_toughness_creature`,
/// `pestilent_acolyte_is_a_three_mana_two_three_warlock`.
pub fn pestilent_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Acolyte",
        cost: cost(&[generic(2), b()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Stoneglare Lecturer ─────────────────────────────────────────────────────

/// Stoneglare Lecturer — {3}{W} 3/3 Cat Cleric (synthesised STX
/// Silverquill flavor). "When this creature enters, you gain 2 life
/// and draw a card."
///
/// A four-mana Bookwurm: 3/3 + 2 life + cantrip. Solid card-advantage
/// body for white control / midrange decks. Wired as ETB
/// `Seq(GainLife 2, Draw 1)`. Tests:
/// `stoneglare_lecturer_etb_gains_life_and_draws`,
/// `stoneglare_lecturer_is_a_four_mana_three_three_cat_cleric`.
pub fn stoneglare_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Stoneglare Lecturer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
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
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pop Quiz Sorcery (cantrip variant) ──────────────────────────────────────

/// Critical Critique — {1}{B} Instant (synthesised STX Silverquill flavor).
/// "Target creature gets -2/-2 until end of turn. Scry 1."
///
/// Two-mana spot removal with a Scry rider — kills 2/2 creatures and
/// digs for the next play. Wired as `Seq(PumpPT -2/-2 EOT, Scry 1)`.
/// Tests: `critical_critique_kills_two_two_and_scrys`,
/// `critical_critique_is_a_two_mana_black_instant`.
pub fn critical_critique() -> CardDefinition {
    CardDefinition {
        name: "Critical Critique",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix Manipulator ────────────────────────────────────────────────────

/// Quandrix Manipulator — {2}{G}{U} 3/3 Elf Druid (synthesised STX
/// Quandrix flavor). "When this creature enters, double the number of
/// +1/+1 counters on target creature."
///
/// Tanazir-on-a-budget — a 3/3 body that fans counters to a friendly
/// +1/+1-bearer. Wired via `Effect::AddCounter { amount:
/// CountersOn(target, +1/+1) }` — adds N more, doubling from N to 2N.
/// Tests: `quandrix_manipulator_doubles_counters_on_target_creature`,
/// `quandrix_manipulator_is_a_four_mana_three_three_elf_druid`.
pub fn quandrix_manipulator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Manipulator",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Prismari Iteration ──────────────────────────────────────────────────────

/// Prismari Iteration — {2}{U}{R} Sorcery (synthesised STX Prismari
/// flavor). "Discard a card, then draw two cards."
///
/// 4-mana looter — trades one off-color card for two fresh ones. Wired
/// as `Seq(Discard 1, Draw 2)`. Net card advantage: -1 + 2 = +1.
/// Tests: `prismari_iteration_loots_two_for_one`,
/// `prismari_iteration_is_a_four_mana_ur_sorcery`.
pub fn prismari_iteration() -> CardDefinition {
    CardDefinition {
        name: "Prismari Iteration",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Lorehold Battle-Priest ──────────────────────────────────────────────────

/// Lorehold Battle-Priest — {2}{R}{W} 2/4 Spirit Cleric with First
/// Strike + Vigilance (synthesised STX Lorehold flavor — exactly the
/// printed shape of Pillardrop Rescuer + first-strike instead of flying).
///
/// A defensive Lorehold body that trades up in combat and stays
/// available for blocking. Tests:
/// `lorehold_battle_priest_is_a_four_mana_two_four_spirit_cleric_with_first_strike_vigilance`.
pub fn lorehold_battle_priest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle-Priest",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Witherbloom Strangler (already exists; this is Witherbloom Reaper) ─────

/// Witherbloom Reaper — {3}{B}{G} 4/3 Plant Warlock with Deathtouch
/// (synthesised STX Witherbloom flavor). "When this creature enters,
/// each opponent sacrifices a creature."
///
/// Edict-on-a-body — a Cruel Edict stapled to a 4/3 Deathtouch. Wired
/// as ETB `ForEach(EachOpponent) → Sacrifice { who: Triggerer,
/// filter: Creature, count: 1 }`. Tests:
/// `witherbloom_reaper_etb_edicts_each_opp`,
/// `witherbloom_reaper_is_a_five_mana_four_three_deathtouch_plant_warlock`.
pub fn witherbloom_reaper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaper",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    filter: SelectionRequirement::Creature,
                    count: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pyromancer's Bolt ──────────────────────────────────────────────────────

/// Pyromancer's Bolt — {1}{R} Instant (synthesised STX Prismari flavor).
/// "This deals 3 damage to target creature or planeswalker."
///
/// Strict-upgrade Lightning Bolt on creatures/PW only — a clean
/// 2-mana removal spell that doesn't trade card advantage for face
/// damage. Wired with `target_filtered(Creature ∨ Planeswalker)`.
/// Tests: `pyromancers_bolt_kills_three_toughness_creature`,
/// `pyromancers_bolt_is_a_two_mana_red_instant`.
pub fn pyromancers_bolt() -> CardDefinition {
    CardDefinition {
        name: "Pyromancer's Bolt",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
            ),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Symmetry Lecturer ──────────────────────────────────────────────────────

/// Symmetry Lecturer — {1}{G}{U} 2/2 Elf Wizard with Flash (synthesised
/// STX Quandrix flavor). "Flash / When this creature enters, put a
/// +1/+1 counter on another target creature you control."
///
/// A combat-trick Quandrix body — flashes in, lands a counter on a
/// blocker mid-combat, then trades up. Wired with `Keyword::Flash` +
/// ETB `AddCounter(+1/+1) → target friendly other`. Tests:
/// `symmetry_lecturer_etb_pumps_friendly_other`,
/// `symmetry_lecturer_has_flash`.
pub fn symmetry_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Symmetry Lecturer",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Wisdom of the Ancients ──────────────────────────────────────────────────

/// Wisdom of the Ancients — {3}{U} Sorcery (synthesised STX Quandrix
/// flavor). "Draw three cards."
///
/// Pure card-velocity blue sorcery — Concentrate's strict-equivalent
/// at the standard 4-mana three-draw template. Useful test exerciser
/// for engine's `Effect::Draw` count-N path. Tests:
/// `wisdom_of_the_ancients_draws_three`,
/// `wisdom_of_the_ancients_is_a_four_mana_blue_sorcery`.
pub fn wisdom_of_the_ancients() -> CardDefinition {
    CardDefinition {
        name: "Wisdom of the Ancients",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Mob Mentality ──────────────────────────────────────────────────────────

/// Mob Mentality — {1}{R}{W} Instant (synthesised STX Lorehold flavor).
/// "Creatures you control get +1/+1 until end of turn. If you've cast
/// another spell this turn, they also gain first strike until end of
/// turn."
///
/// A combat-step finisher — pump everyone, then upgrade with first
/// strike if you cast a setup spell first. Wired as
/// `Seq(ForEach(Creature & ControlledByYou) → PumpPT(+1/+1 EOT),
///      If(SpellsCastThisTurnAtLeast(2)) → ForEach(...) → GrantKeyword(FirstStrike EOT))`.
/// Tests: `mob_mentality_pumps_each_friendly_creature`,
/// `mob_mentality_grants_first_strike_after_second_spell`.
pub fn mob_mentality() -> CardDefinition {
    CardDefinition {
        name: "Mob Mentality",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    }),
                }),
                else_: Box::new(Effect::Noop),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Witherbloom Drain Ritual ───────────────────────────────────────────────

/// Witherbloom Drain Ritual — {2}{B}{G} Sorcery (synthesised STX
/// Witherbloom flavor). "Each opponent loses 3 life. You gain life
/// equal to the life lost this way."
///
/// A Drain Life-style finisher on the Witherbloom color identity.
/// Wired via `Effect::Drain { from: EachOpponent, to: You, amount: 3 }`
/// — symmetric on 1v1 (you gain exactly 3 life). Tests:
/// `witherbloom_drain_ritual_drains_three_each_opp`,
/// `witherbloom_drain_ritual_is_a_four_mana_bg_sorcery`.
pub fn witherbloom_drain_ritual() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drain Ritual",
        cost: cost(&[generic(2), b(), g()]),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Spell Tutor ────────────────────────────────────────────────────────────

/// Mystical Inquiry — {2}{U} Sorcery (synthesised STX Quandrix flavor —
/// a sorcery tutor for the Prismari/Quandrix shells).
/// "Search your library for an instant or sorcery card, reveal it,
/// put it into your hand, then shuffle."
///
/// Standard {2}{U} tutor on a printed catalog template (same shape as
/// `solve_the_equation` but at sorcery speed without an MV cap).
/// Tests: `mystical_inquiry_tutors_an_instant_or_sorcery`,
/// `mystical_inquiry_is_a_three_mana_blue_sorcery`.
pub fn mystical_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Mystical Inquiry",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            to: ZoneDest::Hand(PlayerRef::You),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Conjurer's Bauble (STA reprint, originally Modern Horizons) ────────────

/// Conjurer's Bauble — {0} Artifact (STA reprint, originally Modern
/// Horizons). "{1}, Sacrifice this artifact: Put a card from your
/// graveyard on the bottom of your library. Draw a card."
///
/// A zero-mana artifact that cycles graveyard contents for a fresh
/// draw — useful for both blue control and graveyard-based decks
/// (snake the right card back into the library for a future tutor).
/// Wired with `cost: 0`, `{1}` mana cost + `sac_cost: true` on the
/// activation. The "put gy card on bottom" approximation moves a
/// chosen creature card from gy → library bottom; if no creature
/// is available, the activation still resolves with just the draw.
/// Tests: `conjurers_bauble_zero_mana_artifact`,
/// `conjurers_bauble_sac_activation_cantrips`.
pub fn conjurers_bauble() -> CardDefinition {
    CardDefinition {
        name: "Conjurer's Bauble",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: false,
            sac_cost: true,
            life_cost: 0,
            sorcery_speed: false,
            condition: None,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            once_per_turn: false,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quartzwood Inkling ──────────────────────────────────────────────────────

/// Quartzwood Inkling — {2}{B} 3/2 Inkling Soldier with Menace
/// (synthesised STX Silverquill flavor).
///
/// An efficient Menace beater for the Silverquill aggro shell — scales
/// with Tenured Inkcaster's +2/+2 anthem to a 5/4 trampler. Tests:
/// `quartzwood_inkling_is_a_three_mana_three_two_menace_inkling`,
/// `quartzwood_inkling_buffs_under_tenured_inkcaster`.
pub fn quartzwood_inkling() -> CardDefinition {
    CardDefinition {
        name: "Quartzwood Inkling",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pop Quiz Lecturer ──────────────────────────────────────────────────────

/// Pop Quiz Lecturer — {2}{W} 2/3 Human Cleric with Vigilance
/// (synthesised STX Silverquill flavor). "When this creature enters,
/// scry 2."
///
/// A defensive tap-out creature with a sticky 2/3 + Vigilance body
/// and an ETB scry. Wired with `Effect::Scry { who: You, amount: 2 }`.
/// Tests: `pop_quiz_lecturer_etb_scries_two`,
/// `pop_quiz_lecturer_is_a_three_mana_two_three_vigilance_cleric`.
pub fn pop_quiz_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz Lecturer",
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Brilliant Restoration ──────────────────────────────────────────────────

/// Brilliant Restoration — {3}{W}{W} Sorcery (synthesised STX
/// Silverquill flavor). "Return target creature card from your
/// graveyard to the battlefield. You gain 2 life."
///
/// 5-mana reanimation with a lifegain rider — fits Silverquill's
/// lifegain payoffs (Light of Promise, Promising Duskmage). Wired
/// as `Seq(Move(target creature in gy → bf), GainLife 2)`.
/// Tests: `brilliant_restoration_returns_creature_card_and_gains_life`,
/// `brilliant_restoration_is_a_five_mana_white_sorcery`.
pub fn brilliant_restoration() -> CardDefinition {
    CardDefinition {
        name: "Brilliant Restoration",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkling Studies ────────────────────────────────────────────────────────

/// Inkling Studies — {2}{W}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Create two 2/1 white-and-black Inkling creature tokens
/// with flying."
///
/// Token-mint payoff for Silverquill — slots into Tenured Inkcaster +
/// Felisa Fang lifelink shells. Wired with `Effect::CreateToken {
/// count: 2, definition: inkling_token() }`. Tests:
/// `inkling_studies_creates_two_inkling_tokens`,
/// `inkling_studies_is_a_four_mana_wb_sorcery`.
pub fn inkling_studies() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Studies",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: inkling_token(),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Spirit Banner ──────────────────────────────────────────────────────────

/// Spirit Banner — {3} Artifact (synthesised STX Lorehold flavor).
/// "Spirit creatures you control get +1/+1."
///
/// A tribal anthem for Lorehold's Spirit-focused shells (Sparring
/// Regimen, Quintorius's tokens, Lorehold Excavation's tokens).
/// Wired via `StaticEffect::PumpPT { applies_to:
/// EachPermanent(Creature & ControlledByYou & HasCreatureType(Spirit)),
/// power: +1, toughness: +1 }`. Tests:
/// `spirit_banner_pumps_spirits_by_one_one`,
/// `spirit_banner_does_not_pump_non_spirits`.
pub fn spirit_banner() -> CardDefinition {
    CardDefinition {
        name: "Spirit Banner",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Spirit creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
                ),
                power: 1,
                toughness: 1,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Spectral Adjudicator ──────────────────────────────────────────────────

/// Spectral Adjudicator — {3}{W} 2/3 Spirit Cleric with Flying +
/// Lifelink (synthesised STX Silverquill flavor).
///
/// A defensive Spirit flyer that doubles as a 2/3 lifelink racer.
/// Tests: `spectral_adjudicator_is_a_four_mana_two_three_lifelink_spirit_cleric_with_flying`.
pub fn spectral_adjudicator() -> CardDefinition {
    CardDefinition {
        name: "Spectral Adjudicator",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Quandrix Doubling Tutor ────────────────────────────────────────────────

/// Quandrix Doubling Tutor — {2}{G}{U} Sorcery (synthesised STX
/// Quandrix flavor). "Create two 0/0 green and blue Fractal creature
/// tokens, then put a +1/+1 counter on each Fractal you control."
///
/// Wired as `Seq(CreateToken(2 Fractals), ForEach(Fractal &
/// ControlledByYou) → AddCounter(+1/+1, 1))`. The result is two 1/1
/// Fractals — but with Tanazir Quandrix in play, the counter rain
/// doubles. Tests:
/// `quandrix_doubling_tutor_creates_two_fractals_with_counters`,
/// `quandrix_doubling_tutor_is_a_four_mana_gu_sorcery`.
pub fn quandrix_doubling_tutor() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Doubling Tutor",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: fractal_token(),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkling Scholar ────────────────────────────────────────────────────────

/// Inkling Scholar — {2}{W}{B}, 3/3 Inkling Cleric with Flying +
/// Lifelink (synthesised STX Silverquill flavor).
///
/// A clean four-mana evasive Lifelink threat that doubles as a tribal
/// target for Tenured Inkcaster's +2/+2 anthem (jumping a base 3/3 to
/// 5/5 Flying/Lifelink). Tests:
/// `inkling_scholar_is_a_four_mana_three_three_lifelink_inkling_with_flying`.
pub fn inkling_scholar() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scholar",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkling Squire ─────────────────────────────────────────────────────────

/// Inkling Squire — {W}, 1/1 Inkling Soldier with Flying (synthesised
/// STX Silverquill flavor).
///
/// A 1-mana Inkling flyer for go-wide Silverquill aggro shells.
/// Synergises with Tenured Inkcaster (becomes 3/3 Flying), Felisa
/// (mints another Inkling on death), and Stirring Hopesinger's
/// Repartee counter rain. Test:
/// `inkling_squire_is_a_one_mana_inkling_flier`.
pub fn inkling_squire() -> CardDefinition {
    CardDefinition {
        name: "Inkling Squire",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Silverquill Scholar ────────────────────────────────────────────────────

/// Silverquill Scholar — {W}{B}, 2/1 Human Wizard (synthesised STX
/// Silverquill flavor). "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, draw a card and lose 1 life."
///
/// A Silverquill twist on Archmage Emeritus (which is mono-blue): card
/// advantage with a 1-life tax. Wired via the `magecraft()` shortcut
/// with `Seq(Draw 1, LoseLife 1)`. Tests:
/// `silverquill_scholar_magecraft_draws_and_loses_life`,
/// `silverquill_scholar_does_not_trigger_on_creature_cast`.
pub fn silverquill_scholar() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scholar",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkling Reinforcement ──────────────────────────────────────────────────

/// Inkling Reinforcement — {W}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Create two 1/1 white and black Inkling creature tokens
/// with flying."
///
/// Two-mana go-wide for the Inkling tribe. Cheaper than Inkling
/// Studies (4 mana) for the same effect, trading speed for color
/// pip density. Tests:
/// `inkling_reinforcement_creates_two_inkling_tokens`,
/// `inkling_reinforcement_is_a_two_mana_wb_sorcery`.
pub fn inkling_reinforcement() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Reinforcement",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: inkling_token(),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Pestilent Verse ────────────────────────────────────────────────────────

/// Pestilent Verse — {1}{B}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Destroy target creature. You lose 1 life."
///
/// Three-mana unconditional creature removal at a 1-life tax. Fills
/// the Doom Blade slot in Silverquill / mono-Black shells without
/// the printed "nonblack" restriction. Wired via
/// `Seq(Destroy → target Creature, LoseLife 1)`. Tests:
/// `pestilent_verse_destroys_creature_and_costs_one_life`,
/// `pestilent_verse_is_a_three_mana_black_sorcery`.
pub fn pestilent_verse() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Verse",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkling Ambusher ───────────────────────────────────────────────────────

/// Inkling Ambusher — {2}{B}, 2/2 Inkling Rogue with Flash + Flying
/// (synthesised STX Silverquill flavor).
///
/// A Flash flyer for surprise blocks or end-of-turn pressure.
/// Three-mana 2/2 evasive Inkling that doesn't compete for the
/// double-color pip slot. Test:
/// `inkling_ambusher_is_a_three_mana_inkling_with_flash_and_flying`.
pub fn inkling_ambusher() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ambusher",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Silver-Quill Scholarship ───────────────────────────────────────────────

/// Silver-Quill Scholarship — {2}{W} Sorcery (synthesised STX
/// Silverquill flavor). "Put a +1/+1 counter on target creature. Draw
/// a card."
///
/// Three-mana counter + cantrip; works as a synergy enabler for
/// Tenured Inkcaster (anthem on the bigger counter-target), Felisa
/// (Inkling on counter-bearing-creature-death), Hardened Academic
/// (combat-damage cantrip), or Scolding Administrator (Repartee
/// snowball). Tests: `silver_quill_scholarship_counters_target_and_draws`,
/// `silver_quill_scholarship_is_a_three_mana_white_sorcery`.
pub fn silver_quill_scholarship() -> CardDefinition {
    CardDefinition {
        name: "Silver-Quill Scholarship",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Silvercrown Lecturer ───────────────────────────────────────────────────

/// Silvercrown Lecturer — {3}{W} 2/4 Human Cleric (synthesised STX
/// Silverquill flavor). "When this creature enters, put a +1/+1
/// counter on target creature you control."
///
/// A defensive 4-mana 2/4 body that snowballs the controller's
/// strongest creature. ETB:
/// `AddCounter +1/+1 → target Creature & ControlledByYou`. Tests:
/// `silvercrown_lecturer_etb_lands_counter_on_friendly`,
/// `silvercrown_lecturer_is_a_four_mana_two_four_human_cleric`.
pub fn silvercrown_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Silvercrown Lecturer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Demolishing Lecture ────────────────────────────────────────────────────

/// Demolishing Lecture — {2}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Destroy target creature with toughness 2 or less."
///
/// A focused 3-mana removal spell targeted at small creatures.
/// Cheap maindeck answer to early Silverquill/Witherbloom 2-toughness
/// creatures (Eyetwitch, Star Pupil, Pest Mascot). Tests:
/// `demolishing_lecture_destroys_two_toughness_creature`,
/// `demolishing_lecture_is_a_three_mana_black_sorcery`.
pub fn demolishing_lecture() -> CardDefinition {
    CardDefinition {
        name: "Demolishing Lecture",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtMost(2)),
            ),
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}

// ── Inkling Mentor ─────────────────────────────────────────────────────────

/// Inkling Mentor — {3}{W}{B}, 3/4 Human Cleric (synthesised STX
/// Silverquill flavor). "Other Inkling creatures you control get
/// +1/+1."
///
/// A second Inkling tribal lord (alongside Tenured Inkcaster's
/// +2/+2). Stacks multiplicatively: 1-mana Inkling Squire becomes
/// 4/4 Flying with both lords. Wired via the
/// `tribal_anthem_for_name` compute-time injection in
/// `GameState::compute_battlefield`. Tests:
/// `inkling_mentor_pumps_other_inklings`,
/// `inkling_mentor_does_not_pump_non_inklings`,
/// `inkling_mentor_does_not_pump_self`.
pub fn inkling_mentor() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mentor",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        // Tribal anthem on the Inkling subtype, same shape as
        // Tenured Inkcaster's `StaticEffect::PumpPT` — exclude the
        // source (a Human Cleric, technically already excluded by
        // the creature-type filter) for faithful "Other ..." wording.
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
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
        additional_cast_cost: vec![],
        bestow: None,
    }
}
