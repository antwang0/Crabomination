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

// ── Witherbloom Soothsayer (batch 11) ───────────────────────────────────────

/// Witherbloom Soothsayer — {2}{B}{G}, 2/3 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, surveil 2.
/// Each opponent loses 1 life and you gain 1 life."
///
/// Witherbloom's signature setup-and-drain ETB: bin a target while
/// shaving 1 life off each opp. Pairs with the cheap reanimation
/// shells (Witherbloom Necrogale, Lorehold Memorial, Cauldron of
/// Essence).
pub fn witherbloom_soothsayer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Soothsayer",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
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
    }
}

// ── Lorehold Vanquisher (batch 11) ──────────────────────────────────────────

/// Lorehold Vanquisher — {2}{R}{W}, 3/3 Knight Spirit.
///
/// Printed Oracle (synthesised): "First strike / Whenever this creature
/// attacks, you gain 1 life."
///
/// Lorehold attack-trigger lifegain feeding the Hofri / Spirit-tribal
/// shell. First strike makes the body durable in a stalled board, and
/// the lifegain per swing feeds Witherbloom-cross payoffs in mixed pools.
pub fn lorehold_vanquisher() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanquisher",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Knight, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GainLife {
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Burnscholar (batch 11) ─────────────────────────────────────────

/// Lorehold Burnscholar — {R}{W}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature deals 1 damage to any
/// target. You gain 1 life."
///
/// Lorehold's signature drain-on-cast at the cheap two-drop slot. Two
/// spells in a turn slings 2 damage at face + 2 life gained — a tempo
/// engine the IS-heavy Lorehold pool wants to chain off of.
pub fn lorehold_burnscholar() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Burnscholar",
        cost: cost(&[r(), w()]),
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                ),
                amount: Value::Const(1),
            },
            Effect::GainLife {
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
    }
}

// ── Pillardrop Cultivator (batch 11) ────────────────────────────────────────

/// Pillardrop Cultivator — {3}{R}{W}, 2/3 Spirit Bird, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// return target creature card with mana value 2 or less from your
/// graveyard to the battlefield."
///
/// Lorehold's ETB-reanimation shell, MV-capped to 2 to keep it from
/// pulling oversized targets. Pairs with Daemogoth Woe-Eater and the
/// Witherbloom Pest chain — Pillardrop minds a 1- or 2-mana creature
/// at +flying.
pub fn pillardrop_cultivator() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Pillardrop Cultivator",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Bird],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(2)),
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
    }
}

// ── Prismari Skywatcher (batch 11) ──────────────────────────────────────────

/// Prismari Skywatcher — {U}{R}, 1/2 Merfolk Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature gets +1/+0 until
/// end of turn."
///
/// Symmetry-Sage-styled cheap evasive Magecraft body. Two casts in a
/// turn → 3/2 flier on the swing.
pub fn prismari_skywatcher() -> CardDefinition {
    CardDefinition {
        name: "Prismari Skywatcher",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
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
    }
}

// ── Brewmaster Pyrologist (batch 11) ────────────────────────────────────────

/// Brewmaster Pyrologist — {3}{U}{R}, 4/3 Elemental, Trample.
///
/// Printed Oracle (synthesised): "Trample / When this creature enters,
/// it deals 2 damage to target opponent and you draw a card."
///
/// Pyromentor-style ETB ping + cantrip. Big trampler body lets the
/// damage stick after the +1 card.
pub fn brewmaster_pyrologist() -> CardDefinition {
    CardDefinition {
        name: "Brewmaster Pyrologist",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Player),
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
    }
}

// ── Prismari Spell Smith (batch 11) ─────────────────────────────────────────

/// Prismari Spell Smith — {1}{U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, add one mana of any color to your mana
/// pool."
///
/// Prismari ramp-on-cast — every instant or sorcery refunds itself with
/// the next pip. Curves into chain-cast Magecraft turns.
pub fn prismari_spell_smith() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spell Smith",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::AnyOneColor(Value::Const(1)),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Botanist (batch 11) ────────────────────────────────────────────

/// Quandrix Botanist — {G}{U}, 2/2 Elf Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, put a +1/+1 counter on target Fractal
/// you control."
///
/// Tightly tribal Quandrix payoff — feeds the Fractal counter-stacking
/// shell (Manifestation Sage, Fractal Mascot, Body of Research) every
/// time you Magecraft. Interacts with the new Witherbloom Pestseed
/// (DoubleCounters) for 2× per Magecraft.
pub fn quandrix_botanist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Botanist",
        cost: cost(&[g(), u()]),
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
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Fractal)
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Augur (batch 11) ───────────────────────────────────────────────

/// Quandrix Augur — {2}{G}{U}, 2/3 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2,
/// then draw a card."
///
/// Cheap card-velocity stapled to a 2/3 Fractal body — fits the
/// Quandrix Symmathematics / Manifestation Sage curve and bumps the
/// Tenured Inkcaster-cross Fractal lord with one more dies-trigger
/// fodder.
pub fn quandrix_augur() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Augur",
        cost: cost(&[generic(2), g(), u()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
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
    }
}

// ── Fractal Trefoil (batch 11) ──────────────────────────────────────────────

/// Fractal Trefoil — {1}{G}{U}, 0/0 Fractal.
///
/// Printed Oracle (synthesised): "Trample / This creature enters with
/// X +1/+1 counters on it, where X is the number of lands you control."
///
/// Quandrix's lands-scaling Fractal at the early curve: turn-3 on the
/// play, lands == 3 → 3/3 trampler. Composes cleanly with the
/// `enters_with_counters` field (CR 614.12) and Pestseed
/// (`DoubleCounters`) for compounding 2× scaling.
pub fn fractal_trefoil() -> CardDefinition {
    CardDefinition {
        name: "Fractal Trefoil",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        // CR 614.12 — counters land before SBA so the 0/0 base survives ETB.
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ))),
        )),
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Equationist (batch 11) ─────────────────────────────────────────

/// Quandrix Equationist — {3}{G}{U}, 3/3 Fractal Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Whenever one or more +1/+1
/// counters are put on a creature you control, draw a card."
///
/// Counter-payoff Quandrix flyer. Pairs with Pestseed (doubled
/// counters fire once per AddCounter resolution; per-card emission
/// matches the printed "one or more" wording). Sustains card velocity
/// alongside Manifestation Sage / Quandrix Pledgemage / Botanist /
/// Symmathematics shells.
pub fn quandrix_equationist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equationist",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature,
            }),
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
        additional_cast_cost: vec![],
    }
}

// ── Pyrokinetic Insight (batch 11) ──────────────────────────────────────────

/// Pyrokinetic Insight — {1}{U}{R} Sorcery.
///
/// Printed Oracle (synthesised): "Choose one — / • Pyrokinetic Insight
/// deals 3 damage to any target. / • Draw two cards, then discard a card."
///
/// Prismari "burn or loot" charm shape. Mode 0 closes a game; mode 1
/// digs for the next gas. Auto-decider picks mode 0 unless a scripted
/// override selects mode 1.
pub fn pyrokinetic_insight() -> CardDefinition {
    CardDefinition {
        name: "Pyrokinetic Insight",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: 3 damage to any target.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                ),
                amount: Value::Const(3),
            },
            // Mode 1: draw 2, discard 1 (rummage).
            Effect::Seq(vec![
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
    }
}

// ── Lorehold Spirit Tutor (batch 11) ────────────────────────────────────────

/// Lorehold Spirit Tutor — {1}{R}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Reveal cards from the top of your
/// library until you reveal a Spirit creature card. Put that card into
/// your hand and the rest into your graveyard."
///
/// Spirit-tribal tutor — feeds the Spirit Banner / Hofri / Quintorius
/// chain. The reveal-misses go to graveyard, which is a Lorehold-
/// favorable side effect (Pillardrop Rescuer + Lorehold Memorial fuel).
pub fn lorehold_spirit_tutor() -> CardDefinition {
    use crate::effect::RevealMissDest;
    CardDefinition {
        name: "Lorehold Spirit Tutor",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Creature
                .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(20),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::Graveyard,
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
    }
}

// ── Strixhaven Sanctum (batch 11) ───────────────────────────────────────────

/// Strixhaven Sanctum — Land (no cost).
///
/// Printed Oracle (synthesised): "{T}: Add {C}. / {2}, {T}: Surveil 1.
/// (Look at the top card of your library. You may put it into your
/// graveyard.)"
///
/// Colorless utility land — like the SOS school lands' surveil ability,
/// but in mono-{C}. Fixes any deck wanting an extra Surveil source for
/// graveyard setup (Witherbloom reanimators, Lorehold gy recursion).
pub fn strixhaven_sanctum() -> CardDefinition {
    use super::super::tap_add_colorless;
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Strixhaven Sanctum",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Surveil {
                    who: PlayerRef::You,
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
    }
}

// ── Strixhaven Bloomstadium (batch 11) — Doubling-Season-style enchant ─────

/// Strixhaven Bloomstadium — {3}{G}{G} Enchantment (synthesised).
///
/// Printed Oracle (synthesised, Doubling-Season template): "If one or
/// more tokens would be created under your control, twice that many of
/// those tokens are created instead. / If one or more counters would
/// be put on a permanent you control, twice that many of those
/// counters are put on that permanent instead."
///
/// Pairs the new `StaticEffect::DoubleCounters` with the existing
/// `StaticEffect::DoubleTokens` to ship the canonical "both halves of
/// CR 614.16" card. Composes multiplicatively with Pestseed
/// (DoubleCounters) and Adrix and Nev (DoubleTokens): one of each
/// + Bloomstadium → 4× counters, 4× tokens.
pub fn strixhaven_bloomstadium() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Strixhaven Bloomstadium",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![
            StaticAbility {
                description: "If one or more tokens would be created under your \
                              control, twice that many of those tokens are created \
                              instead.",
                effect: StaticEffect::DoubleTokens,
            },
            StaticAbility {
                description: "If one or more counters would be put on a permanent \
                              you control, twice that many of those counters are \
                              put on that permanent instead.",
                effect: StaticEffect::DoubleCounters,
            },
        ],
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
    }
}

// ── Mystic Slate (batch 11) ─────────────────────────────────────────────────

/// Mystic Slate — {2} Artifact.
///
/// Printed Oracle (synthesised): "{T}: Scry 1. / {2}, {T}: Draw a card.
/// Activate only as a sorcery."
///
/// Library-smoothing colorless artifact at the cheap slot. Pairs with
/// any IS-heavy shell that wants library quality control without
/// burning a card slot — feeds Magecraft / Repartee / cast-trigger
/// chains.
pub fn mystic_slate() -> CardDefinition {
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Mystic Slate",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {T}: Scry 1.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Scry {
                    who: PlayerRef::You,
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
            },
            // {2}, {T}: Draw a card. Sorcery speed.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                once_per_turn: false,
                sorcery_speed: true,
                sac_cost: false,
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
    }
}

// ============================================================================
// Batch 12 — 21 new synthesised STX cards across all five colleges +
// colorless/mono splash. Cards exercise existing engine primitives
// (Magecraft, drain templates, counter-doublers, gy-recursion bodies,
// pump-and-fight combat tricks). Tests in `tests::stx` lock in primary
// play patterns end-to-end.
// ============================================================================

// ── Silverquill Verseweaver (batch 12) ─────────────────────────────────────

/// Silverquill Verseweaver — {2}{W}{B}, 3/3 Inkling Cleric Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// each opponent loses 2 life and you gain 2 life."
///
/// 4-mana evasive lifeswing body — a Silverquill drain on top of a 3/3
/// flyer. The drain feeds Light of Promise / Bookwurm-style lifegain
/// triggers and finishes a low-life opponent.
pub fn silverquill_verseweaver() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Verseweaver",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
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
        additional_cast_cost: vec![],
    }
}

// ── Inkling Choirmaster (batch 12) ─────────────────────────────────────────

/// Inkling Choirmaster — {1}{W}{B}, 1/3 Inkling Cleric.
///
/// Printed Oracle (synthesised): "Whenever you gain life, put a +1/+1
/// counter on this creature. / Other Inkling creatures you control get
/// +1/+0."
///
/// Inkling tribal lord that grows itself off the Silverquill drain plan.
/// Pairs with Inkstrike Bolt, Silverquill Verseweaver, and every
/// magecraft-drain body. The +1/+0 anthem is layered as a static.
pub fn inkling_choirmaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Choirmaster",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // On-lifegain self-grow.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
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
        additional_cast_cost: vec![],
    }
}

// ── Bramble Brewer (batch 12) ──────────────────────────────────────────────

/// Bramble Brewer — {1}{B}{G}, 2/3 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.' / {3}{B}{G}, {T}, Sacrifice another creature:
/// Draw a card and you gain 1 life."
///
/// Witherbloom value engine — gets a Pest on ETB then turns later board
/// presence into card+life. Pairs with Pest Summoning / Tend the Pests
/// to feed the activated ability with disposable fodder.
pub fn bramble_brewer() -> CardDefinition {
    use super::shared::stx_pest_token;
    CardDefinition {
        name: "Bramble Brewer",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3), b(), g()]),
            // Sacrifice another creature as a proper activation cost.
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: Some((
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
                1,
            )),
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: stx_pest_token(),
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
    }
}

// ── Witherbloom Decanter (batch 12) ────────────────────────────────────────

/// Witherbloom Decanter — {B}{G} Instant.
///
/// Printed Oracle (synthesised): "Target creature gets -2/-2 until end
/// of turn. You gain 2 life."
///
/// Witherbloom version of Cast Down with built-in lifegain. Kills any
/// 2/2 or smaller while netting a 2-life swing; pairs with Honor Troll
/// / Light of Promise / Daemogoth Titan for lifegain-payoff stacks.
pub fn witherbloom_decanter() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decanter",
        cost: cost(&[b(), g()]),
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
    }
}

// ── Pestbrood Grovecaller (batch 12) ───────────────────────────────────────

/// Pestbrood Grovecaller — {3}{B}{G}, 3/4 Plant Beast.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.' / Whenever another Pest creature you control
/// dies, you gain 1 life and draw a card."
///
/// Pest tribal payoff that turns each dying Pest into card-and-life.
/// Pairs with Witherbloom Pestmaster (counter-on-Pest-death) for double
/// payoff stacks.
pub fn pestbrood_grovecaller() -> CardDefinition {
    use super::shared::stx_pest_token;
    use crate::card::Predicate;
    CardDefinition {
        name: "Pestbrood Grovecaller",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB Pest.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
                },
            },
            // Whenever another Pest you control dies → gain 1 + draw 1.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                    }),
                effect: Effect::Seq(vec![
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Cathedral (batch 12) ──────────────────────────────────────────

/// Lorehold Cathedral — Land (no cost).
///
/// Printed Oracle (synthesised): "{T}: Add {R} or {W}. / {3}{R}{W},
/// {T}, Sacrifice this land: Return target creature card from your
/// graveyard to the battlefield."
///
/// Lorehold gy-recursion land — taps for either color, with a late-game
/// reanimate sink that survives a clean board state. Sacrificing the
/// land removes the dual-color source but plays an immediate-impact
/// creature.
pub fn lorehold_cathedral() -> CardDefinition {
    use super::super::tap_add;
    CardDefinition {
        name: "Lorehold Cathedral",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            tap_add(Color::Red),
            tap_add(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), r(), w()]),
                // Cast-time targeter sees graveyard cards because
                // `evaluate_requirement_static` walks all zones to find the
                // candidate, then applies the Creature filter (off-battlefield
                // reads consult `CardDefinition.is_creature()`).
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
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
    }
}

// ── Lorehold Bannerbearer (batch 12) ───────────────────────────────────────

/// Lorehold Bannerbearer — {2}{R}{W}, 3/3 Spirit Soldier, First Strike.
///
/// Printed Oracle (synthesised): "First strike / Other Spirit creatures
/// you control get +1/+1."
///
/// Spirit-tribal anthem in a 4-mana first-strike body. Pairs with
/// Quintorius's +1/+0 + Hofri's +1/+0 to push a Spirit-flood Lorehold
/// shell into lethal swings.
pub fn lorehold_bannerbearer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bannerbearer",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
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
    }
}

// ── Lorehold Pyromage (batch 12) ───────────────────────────────────────────

/// Lorehold Pyromage — {3}{R}{W}, 3/4 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, it deals
/// 3 damage to any target. / Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature deals 1 damage to any
/// target."
///
/// Three-mana payoff with on-ETB Searing Spear and per-cast 1-damage
/// rider. The repeatable 1-damage spreads across turns into significant
/// pings — a hidden finisher in spellslinger Lorehold shells.
pub fn lorehold_pyromage() -> CardDefinition {
    use crate::card::Predicate;
    let _ = Predicate::EntityMatches {
        what: Selector::This,
        filter: SelectionRequirement::Creature,
    };
    CardDefinition {
        name: "Lorehold Pyromage",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                    ),
                    amount: Value::Const(3),
                },
            },
            magecraft(Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Geomancer (batch 12) ──────────────────────────────────────────

/// Quandrix Geomancer — {2}{G}{U}, 2/3 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create
/// X 1/1 green and blue Fractal creature tokens, where X is the number
/// of lands you control."
///
/// Fractal token engine — scales with the ramp game plan. With four
/// lands at curve, mints four 1/1s; with seven lands, mints seven.
/// Pairs with Quandrix Conjurer's mass counter-spread.
pub fn quandrix_geomancer() -> CardDefinition {
    let one_one_fractal = TokenDefinition {
        name: "Fractal".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Quandrix Geomancer",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ))),
                definition: one_one_fractal,
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
    }
}

// ── Quandrix Fractalist (batch 12) ─────────────────────────────────────────

/// Quandrix Fractalist — {3}{G}{U}, 3/3 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, put X
/// +1/+1 counters on this creature, where X is the number of cards in
/// your hand. / Trample"
///
/// Scales with hand size — a fresh hand of 5 = 8/8 trample, a topdeck
/// of 1 = 4/4 trample. Pairs with Triskaidekaphile-style high-hand
/// engines and "no maximum hand size" effects.
pub fn quandrix_fractalist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalist",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
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
    }
}

// ── Quandrix Skybinder (batch 12) ──────────────────────────────────────────

/// Quandrix Skybinder — {1}{G}{U}, 2/3 Elf Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Whenever this creature
/// attacks, put a +1/+1 counter on target creature you control."
///
/// Attack-trigger counter feed for friendly bodies. A Skybinder swing
/// + Tanazir Quandrix on the same turn snowballs into a counter
///   avalanche. Pairs with Sparring Regimen for layered counter rain.
pub fn quandrix_skybinder() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Skybinder",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
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
    }
}

// ── Prismari Mistcaller (batch 12) ─────────────────────────────────────────

/// Prismari Mistcaller — {1}{U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2,
/// then draw a card. / Prowess"
///
/// Cantrip + smoothing + prowess body — pure card-velocity Prismari
/// payoff at 3 mana. The ETB Scry+Draw nets +1 card and library
/// quality, while the prowess-ish push makes the body more relevant
/// across multi-spell turns.
pub fn prismari_mistcaller() -> CardDefinition {
    use crate::effect::shortcut::prowess;
    CardDefinition {
        name: "Prismari Mistcaller",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            prowess(),
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
        additional_cast_cost: vec![],
    }
}

// ── Prismari Conflagration (batch 12) ──────────────────────────────────────

/// Prismari Conflagration — {3}{U}{R} Instant.
///
/// Printed Oracle (synthesised): "Choose one — / • Prismari Inferno
/// deals 4 damage to target creature. / • Counter target spell unless
/// its controller pays {3}."
///
/// Prismari modal: removal mode or tempo counter mode. AutoDecider
/// picks the removal mode by default since most casts are at instant
/// speed in response. The {3} tax counter mode is for the bigger
/// counter-magic plan.
pub fn prismari_conflagration() -> CardDefinition {
    CardDefinition {
        name: "Prismari Conflagration",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: 4 damage to target creature.
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
            },
            // Mode 1: Counter target spell unless controller pays {3}.
            Effect::CounterUnlessPaid {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                mana_cost: cost(&[generic(3)]),
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
    }
}

// ── Prismari Treasurewright (batch 12) ─────────────────────────────────────

/// Prismari Treasurewright — {2}{U}{R}, 2/3 Human Artificer.
///
/// Printed Oracle (synthesised): "When this creature enters, create
/// two Treasure tokens. / Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1."
///
/// Treasure ramp for Magma Opus + spellslinger Magecraft engine.
/// Cheap, effective fuel for any U/R deck looking to chain Magma Opus
/// or Crackle with Power on turn 5+.
pub fn prismari_treasurewright() -> CardDefinition {
    CardDefinition {
        name: "Prismari Treasurewright",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
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
                    count: Value::Const(2),
                    definition: crate::game::effects::treasure_token(),
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
        additional_cast_cost: vec![],
    }
}

// ── Silverquill Auctioneer (batch 12) ──────────────────────────────────────

/// Silverquill Auctioneer — {2}{W}{B}, 3/2 Inkling Wizard, Flying +
/// Lifelink.
///
/// Printed Oracle (synthesised): "Flying, lifelink / Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, put a +1/+1
/// counter on this creature."
///
/// Self-growing flying lifelinker — every cast feeds the counter feed,
/// which the lifelink then converts to lifegain on the swing. Pairs
/// with Light of Promise for double counters per cast.
pub fn silverquill_auctioneer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Auctioneer",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
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
    }
}

// ── Witherbloom Reanimist (batch 12) ───────────────────────────────────────

/// Witherbloom Reanimist — {2}{B}{G}, 3/2 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, return
/// target creature card with mana value 2 or less from your graveyard
/// to your hand. / {2}{B}{G}, Pay 2 life: Return target creature card
/// from your graveyard to your hand."
///
/// Witherbloom value engine — repeatable graveyard recursion at a
/// per-activation life cost. Pairs with Pest-die-to-life Witherbloom
/// drain math (net 1 life lost per activation).
pub fn witherbloom_reanimist() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reanimist",
        cost: cost(&[generic(2), b(), g()]),
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), b(), g()]),
            // Target-filter walks all zones for the candidate; the gy
            // card matches `Creature` via off-bf is_creature() read.
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 2,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(2)),
                ),
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
        additional_cast_cost: vec![],
    }
}

// ── Lorehold Skirmisher (batch 12) ─────────────────────────────────────────

/// Lorehold Skirmisher — {1}{R}{W}, 2/2 Spirit Soldier, Haste.
///
/// Printed Oracle (synthesised): "Haste / Whenever this creature
/// attacks, you may pay {R}. If you do, it gets +1/+0 until end of
/// turn."
///
/// Cheap haste body with an optional Lava-Spike-style attack pump.
/// AutoDecider auto-declines the {R} payment to preserve mana; a
/// ScriptedDecider can pay for the +1/+0 swing.
pub fn lorehold_skirmisher() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Skirmisher",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {R} to pump this creature +1/+0 until end of turn?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
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
    }
}

// ── Quandrix Landmapper (batch 12) ─────────────────────────────────────────

/// Quandrix Landmapper — {2}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Search your library for a basic land
/// card, put it onto the battlefield, then shuffle. Scry 2."
///
/// Cultivate-style ramp + Scry 2 smoothing in Quandrix colors. Three
/// mana to net a land drop and dig two cards deep.
pub fn quandrix_landmapper() -> CardDefinition {
    use crate::card::LandType;
    let _ = LandType::Forest;
    CardDefinition {
        name: "Quandrix Landmapper",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Scry {
                who: PlayerRef::You,
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
    }
}

// ── Prismari Spellsong (batch 12) ──────────────────────────────────────────

/// Prismari Spellsong — {U}{R} Instant.
///
/// Printed Oracle (synthesised): "Draw a card, then discard a card. If
/// a noncreature card was discarded, this deals 2 damage to any
/// target."
///
/// Pure card-velocity instant with conditional burn rider. The
/// noncreature-discard check uses the existing
/// `creature_cards_discarded_this_resolution` counter (via the inverse
/// — if zero, then a noncreature card was discarded; if non-zero, then
/// the discarded card was a creature). The body is wired as a fixed
/// "draw+discard+conditional ping" sequence.
pub fn prismari_spellsong() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Prismari Spellsong",
        cost: cost(&[u(), r()]),
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
            // If no creature card was discarded → 2 damage. The predicate
            // is `creatures_discarded_this_resolution == 0`. We approximate
            // via `ValueAtLeast` inverted: gate fires when the count is 0.
            Effect::If {
                cond: Predicate::ValueEquals(
                    Value::CreatureCardsDiscardedThisEffect,
                    Value::Const(0),
                ),
                then: Box::new(Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                    ),
                    amount: Value::Const(2),
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
    }
}

// ── Silverquill Reaper (batch 12) ──────────────────────────────────────────

/// Silverquill Reaper — {3}{W}{B}, 4/3 Inkling Warlock, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// destroy target creature with toughness 2 or less."
///
/// Evasive ETB removal body — drops on turn 5 with a 2-toughness-or-
/// less kill stapled on (sweeps Inklings, Pests, mana dorks, and most
/// chump blockers).
pub fn silverquill_reaper() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Reaper",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtMost(2)),
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
    }
}

// ── Strixhaven Reservoir (batch 12) ────────────────────────────────────────

/// Strixhaven Reservoir — {3} Artifact.
///
/// Printed Oracle (synthesised): "{T}: Add one mana of any color. /
/// {3}, {T}: Draw a card."
///
/// Five-color rock at three mana with a built-in draw outlet. The
/// mana-of-any-color floods polychromatic shells; the {3}{T}: Draw
/// turns excess mana on stalled turns into card velocity.
pub fn strixhaven_reservoir() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Reservoir",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
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
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Draw {
                    who: Selector::You,
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
    }
}

// ── Lone Rider (batch 12, CR 506.5 exerciser) ──────────────────────────────

/// Lone Rider — {1}{R}, 2/2 Human Knight, Haste.
///
/// Printed Oracle (synthesised): "Haste / Whenever this creature attacks
/// alone, it gets +2/+0 and gains trample until end of turn."
///
/// First card exercising the new `SelectionRequirement::IsAttackingAlone`
/// predicate (CR 506.5). The trigger fires on every Attacks event but
/// is gated by an intervening-if predicate: the trigger is pushed onto
/// the stack only when this creature is the only declared attacker.
/// Tests: `lone_rider_pumps_when_attacking_alone`,
/// `lone_rider_does_not_pump_with_other_attackers`.
pub fn lone_rider() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Lone Rider",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsAttackingAlone,
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Trample,
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
    }
}

// ── Spelltongue Statute (batch 12) ─────────────────────────────────────────

/// Spelltongue Statute — {2}{W} Enchantment.
///
/// Printed Oracle (synthesised): "Whenever you cast or copy an instant
/// or sorcery spell, you gain 1 life."
///
/// Pure spellslinger lifegain payoff — every cast nets a life,
/// enabling Light of Promise / Honor Troll / Heliod-style lifegain
/// chains. Pairs with Wandering Archaic copy triggers for double-tap.
pub fn spelltongue_statute() -> CardDefinition {
    CardDefinition {
        name: "Spelltongue Statute",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::GainLife {
            who: Selector::You,
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
        additional_cast_cost: vec![],
    }
}

// ============================================================================
// Batch 13 — 5 more synthesised STX cards (additional Lone-Rider-style
// IsAttackingAlone payoffs, Lorehold reanimator combo, Quandrix card
// velocity + a finisher).
// ============================================================================

// ── Solo Striker (batch 13, CR 506.5 exerciser) ────────────────────────────

/// Solo Striker — {2}{W}, 3/2 Human Soldier, Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance / Whenever this creature
/// attacks alone, this creature gets +1/+2 and gains lifelink until end
/// of turn."
///
/// Second card exercising `SelectionRequirement::IsAttackingAlone`.
/// Pairs with Lone Rider — a White Knight's-tale combat trick: 4/4
/// Lifelink + Vigilance is a swift, recoverable swing.
pub fn solo_striker() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Solo Striker",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: SelectionRequirement::IsAttackingAlone,
                }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Lifelink,
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
    }
}

// ── Lorehold Tomb Robber (batch 13) ────────────────────────────────────────

/// Lorehold Tomb Robber — {2}{R}{W}, 3/3 Spirit Rogue.
///
/// Printed Oracle (synthesised): "When this creature enters, exile
/// target creature card from your graveyard. Create a token that's
/// a copy of that card except it has haste. Exile that token at the
/// beginning of the next end step."
///
/// Approximation: since the engine has no permanent-copy primitive
/// (Effect::CreateCopyToken is still ⏳), this ships the simpler
/// Move(target gy creature card → battlefield, tapped) + grant haste +
/// delayed Exile-at-end-step. That's a one-turn-rental reanimation
/// pattern equivalent to printed Oracle for combat math.
pub fn lorehold_tomb_robber() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Tomb Robber",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Rogue],
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
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::DelayUntil {
                    kind: crate::effect::DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Exile {
                        what: Selector::Target(0),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Loremind (batch 13) ───────────────────────────────────────────

/// Quandrix Loremind — {1}{G}{U}, 1/3 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a
/// card. / {3}{G}{U}, Sacrifice this creature: Draw two cards."
///
/// Card-velocity body with a sac-for-draw outlet. The cheap activate
/// makes the Loremind a flexible mid-game card source — sacs into 2
/// cards when the board is settled, or fuels its own value while
/// holding open mana for instants.
pub fn quandrix_loremind() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Loremind",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(3), g(), u()]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
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
        additional_cast_cost: vec![],
    }
}

// ── Prismari Sparkbinder (batch 13) ────────────────────────────────────────

/// Prismari Sparkbinder — {2}{U}{R}, 3/3 Elemental Wizard.
///
/// Printed Oracle (synthesised): "Whenever you cast or copy an instant
/// or sorcery spell, this creature deals 1 damage to each opponent and
/// you create a Treasure token."
///
/// Spellslinger payoff that doubles as a Treasure ramp engine. Pairs
/// with Magma Opus / Crackle with Power to close games — each cast
/// pings opp AND nets a Treasure. Combos with Wandering Archaic's
/// copy trigger for triple value.
pub fn prismari_sparkbinder() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkbinder",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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
    }
}

// ── Witherbloom Hexweaver (batch 13) ───────────────────────────────────────
// (Last batch-13 card — batch 14 additions begin after the closing brace.)

/// Witherbloom Hexweaver — {3}{B}{G}, 3/4 Human Warlock, Deathtouch.
///
/// Printed Oracle (synthesised): "Deathtouch / When this creature
/// enters, target opponent loses 2 life and you gain 2 life. / Whenever
/// you gain life, target creature an opponent controls gets -1/-1
/// until end of turn."
///
/// Witherbloom drain + lifegain-payoff combo. The ETB drain triggers
/// the lifegain rider on itself (immediate -1/-1), and any subsequent
/// lifegain (Pest dies, Honor Troll trigger) keeps shrinking opp's
/// board. A long-game grind engine.
pub fn witherbloom_hexweaver() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Hexweaver",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB drain 2 life.
            etb_drain(2),
            // Whenever you gain life, target opp creature gets -1/-1 EOT.
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
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
        additional_cast_cost: vec![],
    }
}


// ============================================================================
// Batch 14 — Cross-college expansion (push modern_decks).
//
// 10 new synthesised STX cards across multiple colleges + colorless +
// shared slots. Each card ships with at least one functionality test
// in `tests::stx`. Cards target gaps in the catalog around tribal
// payoffs (Lorehold Spirit, Quandrix Fractal), Magecraft variants,
// reanimation chains, and combat-step interactions.
// ============================================================================

// ── Lorehold Phantasmist (batch 14) ─────────────────────────────────────────

/// Lorehold Phantasmist — {2}{R}{W}, 3/2 Spirit Wizard.
///
/// Printed Oracle (synthesised): "Other Spirit creatures you control
/// have haste."
///
/// Spirit-tribal payoff that pairs with Quintorius / Sparring Regimen
/// minted Spirits. Haste on a wide Spirit board turns every freshly
/// minted token into an immediate attacker. Wired via
/// `StaticEffect::GrantKeyword` filtered to Other Spirits — same shape
/// as Inkling Verselord's lifelink anthem.
pub fn lorehold_phantasmist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Phantasmist",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Haste,
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
    }
}

// ── Lorehold Bookburner (batch 14) ──────────────────────────────────────────

/// Lorehold Bookburner — {1}{R}{W}, 2/2 Dwarf Shaman.
///
/// Printed Oracle (synthesised): "{R}{W}, Sacrifice this creature:
/// This creature deals 2 damage to any target."
///
/// A 3-mana 2/2 with a built-in burn outlet. The activation puts 2
/// damage on a creature, player, or planeswalker — a flexible "Voltaic
/// Bolt" attached to a body. Wired via `sac_cost: true` activated
/// ability with `target_filtered(Creature ∨ Player ∨ Planeswalker)`.
pub fn lorehold_bookburner() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Bookburner",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[r(), w()]),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(2),
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
    }
}

// ── Prismari Lightcaster (batch 14) ─────────────────────────────────────────

/// Prismari Lightcaster — {U}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2."
///
/// Cheap blue-red 2-drop with a scry-2 ETB — fixes the next two draws
/// while attacking on a clock. Slots into the Prismari spellslinger
/// shell as a curve-topper for Magecraft + smoothing.
pub fn prismari_lightcaster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Lightcaster",
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
    }
}

// ── Prismari Stormbringer (batch 14) ────────────────────────────────────────

/// Prismari Stormbringer — {3}{U}{R}, 4/4 Elemental Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature deals 2
/// damage to each opponent."
///
/// Five-mana finisher that scales hard with spellslinger payoffs.
/// Stacks with Sparkbinder's Treasure + ping body for cumulative
/// damage. Wired via the existing `magecraft` helper.
pub fn prismari_stormbringer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Stormbringer",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
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
        additional_cast_cost: vec![],
    }
}

// ── Quandrix Counterspeaker (batch 14) ──────────────────────────────────────

/// Quandrix Counterspeaker — {2}{G}{U}, 3/3 Frog Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, put a +1/+1 counter on this creature."
///
/// Pure Quandrix self-pump on cast. Scales linearly with cast count
/// — a 5-mana 4/4 after one cast, 5/5 after two, etc. Pairs with
/// Tanazir Quandrix's counter-doubling and Symmathematics' magecraft
/// counter doubler.
pub fn quandrix_counterspeaker() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Counterspeaker",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
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
    }
}

// ── Quandrix Tessellator (batch 14) ─────────────────────────────────────────

/// Quandrix Tessellator — {1}{G}{U}, 2/2 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 1.
/// / {3}{G}{U}: Create a 0/0 green and blue Fractal creature token,
/// then put two +1/+1 counters on it."
///
/// Quandrix 3-drop with both a smoothing ETB AND a mid-game Fractal
/// minting outlet. Each activation drops a 2/2 Fractal — solid
/// value-engine body. Wired via:
/// - ETB Scry 1 (smoothing)
/// - Activated `{3}{G}{U}`: `Seq(CreateToken 0/0 Fractal, AddCounter
///   +1/+1 × 2 on Selector::LastCreatedToken)`
pub fn quandrix_tessellator() -> CardDefinition {
    let fractal = crate::card::TokenDefinition {
        name: "Fractal".to_string(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Quandrix Tessellator",
        cost: cost(&[generic(1), g(), u()]),
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(3), g(), u()]),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal,
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
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
            effect: Effect::Scry {
                who: PlayerRef::You,
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
    }
}

// ── Witherbloom Wanderer (batch 14) ─────────────────────────────────────────

/// Witherbloom Wanderer — {2}{B}{G}, 3/2 Plant Warrior.
///
/// Printed Oracle (synthesised): "When this creature enters, you may
/// pay 2 life. If you do, return target creature card from your
/// graveyard to your hand."
///
/// Witherbloom-flavored gravedigger: pay 2 life as part of the ETB
/// resolution to recur a creature. The MayDo body sequences a
/// LoseLife with a graveyard-to-hand Move. AutoDecider declines by
/// default; ScriptedDecider can flip to true for tests.
pub fn witherbloom_wanderer() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Wanderer",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Pay 2 life: Return target creature card from your graveyard to your hand.".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                    Effect::Move {
                        what: Selector::one_of(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: crate::card::Zone::Graveyard,
                            filter: SelectionRequirement::Creature,
                        }),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
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
    }
}

// ── Witherbloom Pestbinder (batch 14) ───────────────────────────────────────

/// Witherbloom Pestbinder — {1}{B}{G}, 1/1 Pest Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create
/// a 1/1 black and green Pest creature token with 'When this dies,
/// you gain 1 life.' / Whenever a Pest you control dies, draw a card."
///
/// Witherbloom Pest-tribal value engine. Each Pest dying (including
/// the Pestbinder itself if blocked) draws a card AND gains a life
/// via the Pest token's death trigger. Stacks with Pest Summoning,
/// Tend the Pests, and Felisa's Pest minting for an endless value
/// chain.
pub fn witherbloom_pestbinder() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Pestbinder",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: super::shared::stx_pest_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                    }),
                effect: Effect::Draw {
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
        additional_cast_cost: vec![],
    }
}

// ── Strixhaven Vault (batch 14) ─────────────────────────────────────────────

/// Strixhaven Vault — {3} Artifact.
///
/// Printed Oracle (synthesised): "When this artifact enters, scry 2.
/// / {1}, {T}, Sacrifice this artifact: Draw a card."
///
/// Colorless utility artifact for any deck — smoothing on ETB + an
/// outlet to convert into a fresh card later. Sized at 3 mana for
/// the scry-2 ETB; sized at {1},{T},Sac for the cantrip-on-demand.
pub fn strixhaven_vault() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Vault",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
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
    }
}

// ── Strixhaven Acolyte (batch 14) ───────────────────────────────────────────

/// Strixhaven Acolyte — {W}, 1/1 Human Cleric, Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink"
///
/// Vanilla white 1-drop with lifelink. Slots into any white aggressive
/// deck as an early racer + Light of Promise enabler. Pairs with
/// magecraft / spellslinger payoffs as a cheap body that doesn't
/// dilute the spell density.
pub fn strixhaven_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Acolyte",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
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
    }
}

// ── Strixhaven Scholar (batch 20) ──────────────────────────────────────────

/// Strixhaven Scholar — {1}{U}, 2/1 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, scry 1."
///
/// 2-mana magecraft scry body — slow-game card-selection engine. Pairs
/// with Symmetrist / Wavewright for scry-into-draw stacks.
pub fn strixhaven_scholar() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Scholar",
        cost: cost(&[generic(1), u()]),
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
        additional_cast_cost: vec![],
    }
}

// ── Strixhaven Quill-Mage (batch 20) ───────────────────────────────────────

/// Strixhaven Quill-Mage — {2}{R}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature deals 1 damage to target
/// opponent."
///
/// 3-mana magecraft direct-damage body. Strictly worse than Mascot
/// Exhibition's general "any target" version but with no creature
/// targeting constraint — pure player-burn engine.
pub fn strixhaven_quill_mage() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Quill-Mage",
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
        triggered_abilities: vec![magecraft(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Player),
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
        additional_cast_cost: vec![],
    }
}

// ── Strixhaven Initiate (batch 20) ─────────────────────────────────────────

/// Strixhaven Initiate — {G}, 1/2 Human Druid.
///
/// Printed Oracle (synthesised): "Reach. {T}: Add {G}."
///
/// 1-mana reach defender + green mana ramp. Plays into Quandrix
/// counter-grow shells (mana for the +1/+1-counter pump) and Witherbloom
/// drain shells (Pest payoffs need {B}{G}).
pub fn strixhaven_initiate() -> CardDefinition {
    use super::super::tap_add;
    CardDefinition {
        name: "Strixhaven Initiate",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(Color::Green)],
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
    }
}

// ── Strixhaven Burnscholar (batch 20) ──────────────────────────────────────

/// Strixhaven Burnscholar — {R}, 1/1 Human Wizard with Haste.
///
/// Printed Oracle (synthesised): "Haste. When this creature enters, it
/// deals 1 damage to target opponent."
///
/// 1-mana haste 1/1 with ETB-ping rider. Combines tempo (haste swing
/// for 1) with reach (ETB-1) for 2 effective damage per cast. Reachy
/// burn fodder for the early Lorehold / Prismari curve.
pub fn strixhaven_burnscholar() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Burnscholar",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Player),
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
    }
}

// ── Strixhaven Necropact (batch 20) ────────────────────────────────────────

/// Strixhaven Necropact — {2}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Target player draws two cards and
/// loses 2 life."
///
/// 3-mana Sign-in-Blood that can target either side. Classic black
/// card-draw at a life cost — strictly worse than Sign in Blood when
/// targeting self but with the flexibility to target opp as a drain.
pub fn strixhaven_necropact() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Necropact",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Target(0),
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
    }
}

// ── Push (modern_decks) batch 27: 22 new STX cards ──────────────────────────
//
// New batch adds cards across all five colleges plus shared/mono cards.
// All use existing primitives. Each card has at least one functionality
// test in `tests/stx.rs`.

/// Lorehold Stonebrand — {2}{R}{W}, 3/3 Spirit Soldier.
///
/// Synthesised real STX-style design: "When this creature enters,
/// you may exile target creature card from a graveyard. If you do,
/// create a 2/2 R/W Spirit token."
///
/// Conditional Spirit minter — turns gy fodder into pressure. Pairs
/// with Lorehold Excavation chains.
pub fn lorehold_stonebrand() -> CardDefinition {
    use super::lorehold::lorehold_spirit_token;
    CardDefinition {
        name: "Lorehold Stonebrand",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
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
                description: "Exile a creature card from a graveyard; create a 2/2 R/W Spirit token".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::one_of(Selector::CardsInZone {
                            who: PlayerRef::EachPlayer,
                            zone: crate::card::Zone::Graveyard,
                            filter: SelectionRequirement::Creature,
                        }),
                        to: ZoneDest::Exile,
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: lorehold_spirit_token(),
                    },
                ])),
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
    }
}
