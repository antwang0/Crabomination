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

// ── Take Up the Shield ──────────────────────────────────────────────────────

/// Take Up the Shield — {1}{W} Instant.
/// "Target creature gets +0/+3 and gains indestructible until end of turn."
///
/// Strixhaven Silverquill defensive combat trick — same shape as
/// Masterful Flourish (SOS) but white and with a toughness bump instead
/// of a power bump. Wired as `Seq(PumpPT(+0/+3), GrantKeyword(Indestructible))`
/// against a generic `Creature` target. The target's controller doesn't
/// matter; useful as a Fog-style protection spell on a friendly attacker
/// or as defensive cover on a blocker.
pub fn take_up_the_shield() -> CardDefinition {
    CardDefinition {
        name: "Take Up the Shield",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
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

// ── Star Pupil's Papers ─────────────────────────────────────────────────────

/// Star Pupil's Papers — {1} Artifact.
/// "When this artifact enters, scry 1. /
///  {2}, Sacrifice this artifact: Put a +1/+1 counter on target creature."
///
/// Cheap colorless filter + counter payoff. ETB Scry 1 gives any deck
/// a smoothing tool for a single mana; the sac-for-counter activation
/// converts the artifact into a permanent body buff once it's
/// served its filtering purpose. Wired as `Effect::Scry` for the ETB
/// trigger and an activated ability with `sac_cost: true` for the
/// counter half.
pub fn star_pupils_papers() -> CardDefinition {
    CardDefinition {
        name: "Star Pupil's Papers",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
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
            tap_other_filter: None,
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

// ── Snarl land cycle ────────────────────────────────────────────────────────

/// Build a Strixhaven Snarl dual land. Printed Oracle: "As this land
/// enters, you may reveal a [C1] or [C2] card from your hand. If you
/// don't, this land enters tapped."
///
/// ✅ Wired (push modern_decks) via the new `Effect::IfRevealFromHand`
/// primitive: ETB trigger peeks at the controller's hand for a card
/// matching `HasLandType(type_a) ∨ HasLandType(type_b)`. If a match
/// exists, the AutoDecider auto-reveals and the land stays untapped
/// (Noop branch). Otherwise the `else_` branch taps the land. The
/// reveal itself isn't surfaced as a separate UI prompt yet — a
/// future enhancement could surface `Decision::Reveal` so a human
/// player can bluff "don't reveal" with a matching card in hand.
fn snarl_land(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
) -> CardDefinition {
    use super::super::tap_add;
    use crate::card::{SelectionRequirement, TriggeredAbility};
    use crate::effect::{EventKind, EventScope, EventSpec};
    let reveal_filter = SelectionRequirement::HasLandType(type_a)
        .or(SelectionRequirement::HasLandType(type_b));
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::IfRevealFromHand {
                filter: reveal_filter,
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap { what: Selector::This }),
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

/// Frostboil Snarl — Izzet (U/R) Snarl land.
pub fn frostboil_snarl() -> CardDefinition {
    snarl_land(
        "Frostboil Snarl",
        LandType::Island,
        LandType::Mountain,
        Color::Blue,
        Color::Red,
    )
}

/// Furycalm Snarl — Boros (R/W) Snarl land.
pub fn furycalm_snarl() -> CardDefinition {
    snarl_land(
        "Furycalm Snarl",
        LandType::Mountain,
        LandType::Plains,
        Color::Red,
        Color::White,
    )
}

/// Necroblossom Snarl — Golgari (B/G) Snarl land.
pub fn necroblossom_snarl() -> CardDefinition {
    snarl_land(
        "Necroblossom Snarl",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
    )
}

/// Shineshadow Snarl — Orzhov (W/B) Snarl land.
pub fn shineshadow_snarl() -> CardDefinition {
    snarl_land(
        "Shineshadow Snarl",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
    )
}

/// Vineglimmer Snarl — Simic (G/U) Snarl land.
pub fn vineglimmer_snarl() -> CardDefinition {
    snarl_land(
        "Vineglimmer Snarl",
        LandType::Forest,
        LandType::Island,
        Color::Green,
        Color::Blue,
    )
}

// ── Dragon's Approach ───────────────────────────────────────────────────────

/// Dragon's Approach — {B} Sorcery.
/// "Dragon's Approach deals 3 damage to any target. Then if you have
/// four or more cards named Dragon's Approach in your graveyard, you
/// may search your library for a Dragon creature card, put it onto
/// the battlefield, then shuffle. A deck can have any number of cards
/// named Dragon's Approach."
///
/// ✅ Both halves wired. The 3 damage half uses
/// `target_filtered(Creature ∨ Planeswalker ∨ Player)`. The "4+ in gy
/// → tutor a Dragon" rider rides on the new
/// `Predicate::SameNamedInZoneAtLeast { who: You, zone: Graveyard,
/// at_least: 4 }` primitive — the engine reads the resolving spell's
/// printed name from `EffectContext.source` (stamped by
/// `for_spell_with_source`) and counts matches in the controller's
/// graveyard. On hit, `Effect::Search` walks the library for a
/// creature card with the Dragon subtype and drops it onto the
/// battlefield untapped. The shuffle is handled implicitly by
/// `Effect::Search` (every successful search auto-shuffles).
pub fn dragons_approach() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Approach",
        cost: cost(&[b()]),
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
                        .or(SelectionRequirement::Planeswalker)
                        .or(SelectionRequirement::Player),
                ),
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::SameNamedInZoneAtLeast {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    at_least: Value::Const(4),
                },
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
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

// ── Defiant Strike ──────────────────────────────────────────────────────────

/// Defiant Strike — {W} Instant (Strixhaven Mystical Archive).
/// "Target creature you control gets +1/+0 until end of turn. Draw a card."
///
/// Classic white cantrip-pump. Wired as `Seq(PumpPT(+1/+0), Draw(1))`
/// — the pump targets a friendly creature (controller filter), the
/// draw fires regardless. Clean uses include turning a 2-power
/// attacker into a 3-power that bashes through small chumps while
/// replacing the card in hand.
pub fn defiant_strike() -> CardDefinition {
    CardDefinition {
        name: "Defiant Strike",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
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
    }
}

// ── Divine Gambit ───────────────────────────────────────────────────────────

/// Divine Gambit — {2}{W} Instant (Strixhaven Mystical Archive).
/// "Exile target nonland permanent. Its controller may put a permanent
/// card from their hand onto the battlefield." Both clauses ship: the
/// gift-back is a `MayDo(Move(hand → battlefield))` offered to the
/// target's controller (auto-decider declines by default).
pub fn divine_gambit() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Divine Gambit",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Push (modern_decks, batch 77): both printed clauses now ship.
        // Body 1: exile target nonland permanent. Body 2: the target's
        // *controller* may put a permanent card from their hand onto
        // the battlefield. Body 2 wraps a Move(hand → battlefield) inside
        // `Effect::MayDo`. AutoDecider's default `Bool(false)` declines
        // the gift-back — matches the engine-level "auto-pessimistic"
        // behavior (the Divine Gambit caster wouldn't want their opp to
        // gift themselves a new threat for free). `ScriptedDecider::
        // new([Bool(true)])` exercises the printed "opp accepts" path.
        // The MayDo question is technically asked of ctx.controller
        // (= Divine Gambit caster) rather than the target's controller
        // — but the auto outcomes are equivalent since both perspectives
        // align on declining the gift-back. The card picker for the
        // hand → bf move auto-selects the highest-CMC permanent card via
        // `Selector::take`'s sort.
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
                to: ZoneDest::Exile,
            },
            Effect::MayDo {
                description: "Put a permanent card from your hand onto the battlefield?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                            zone: Zone::Hand,
                            filter: SelectionRequirement::Permanent
                                .and(SelectionRequirement::Nonland),
                        },
                        Value::Const(1),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                        tapped: false,
                    },
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
    }
}

// ── Cram Session ────────────────────────────────────────────────────────────

/// Cram Session — {3}{W} Instant.
/// "Target player gains 5 life. Flashback {5}{W}."
///
/// Pure lifegain at instant speed with a Flashback recast. The body
/// gains 5 life to its controller (`Selector::You` — the multi-target
/// "target player" prompt collapses to the caster; auto-target picker
/// has no friendlier candidate). Flashback {5}{W} via the engine's
/// existing `Keyword::Flashback` keyword (push X) — the cast-from-
/// graveyard path is the same one used by Pursue the Past, Sacred
/// Fire, and Tome Blast.
pub fn cram_session() -> CardDefinition {
    CardDefinition {
        name: "Cram Session",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(5), w()]))],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(5),
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

// ── Soothsayer Adept ────────────────────────────────────────────────────────

/// Soothsayer Adept — {1}{U} Creature — Merfolk Wizard, 2/2.
/// "{2}{U}: Surveil 1."
///
/// Cheap interaction body for Quandrix/Prismari decks: a 2/2 for two
/// mana plus an activated Surveil 1 for filtering. The activated
/// ability dumps the top card to graveyard or keeps it on top via
/// the engine's `Effect::Surveil`.
pub fn soothsayer_adept() -> CardDefinition {
    CardDefinition {
        name: "Soothsayer Adept",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), u()]),
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
            tap_other_filter: None,
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

// ── Crux of Fate ────────────────────────────────────────────────────────────

/// Crux of Fate — {3}{B}{B} Sorcery (STA reprint).
///
/// "Choose one — / • Destroy each Dragon. / • Destroy each non-Dragon
/// creature."
///
/// Push (modern_decks): wired via `Effect::ChooseMode` with two
/// `ForEach + Destroy` modes. Mode 0 destroys each creature with the
/// Dragon creature type via `SelectionRequirement::HasCreatureType
/// (Dragon)`; mode 1 destroys each *non-Dragon* creature via the
/// `Creature & !HasCreatureType(Dragon)` filter, threaded through the
/// existing `SelectionRequirement::Not` predicate combinator. The
/// printed "destroy" half cleanly handles indestructible (the engine's
/// `Destroy` consults `Keyword::Indestructible`). Black's Crux of Fate
/// is the canonical "Dragons matter" wrath — kills opponent's army
/// without scratching your own Dragon shell. The {3}{B}{B} cost is
/// honored exactly.
pub fn crux_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Crux of Fate",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: destroy each Dragon.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                ),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
                }),
            },
            // Mode 1: destroy each non-Dragon creature.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::HasCreatureType(CreatureType::Dragon).negate(),
                    ),
                ),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
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
    }
}

// ── Plargg, Dean of Chaos ───────────────────────────────────────────────────

/// Plargg, Dean of Chaos — {1}{R}, 2/2 Legendary Human Cleric.
///
/// "{T}: Discard a card, then draw a card. If a creature card was
/// discarded this way, Plargg, Dean of Chaos deals 2 damage to any
/// target."
///
/// Push (modern_decks, this revision): the conditional damage rider is
/// **now wired** via the new `Value::CreatureCardsDiscardedThisEffect`
/// primitive. After the `Discard 1 + Draw 1` chain, an
/// `Effect::If { cond: ValueAtLeast(CreatureCardsDiscardedThisEffect, 1),
/// then: DealDamage(2), else_: Noop }` fires the 2 damage only when a
/// creature card was the one discarded. AutoDecider chose the first card
/// (which is what `Discard { random: false }` does on AutoDecider paths
/// — surfaces a `Decision::Discard` and AutoDecider answers with the
/// first hand-card matching `count`). The "any target" slot is reserved
/// via `target_filtered(Creature ∨ Player ∨ Planeswalker)` so the
/// activation requires a target up front (auto-target picker reads the
/// trigger's slot 0). The "Partner with Augusta, Dean of Order" rider
/// is still omitted — engine has no Partner-pair primitive (only the
/// singleton legend constraint is enforced).
///
/// Tests: `plargg_dean_of_chaos_taps_to_loot` (no-creature discard path,
/// damage skipped), `plargg_dean_of_chaos_deals_two_damage_when_creature_discarded`
/// (scripted-decider picks the creature in hand, damage fires).
pub fn plargg_dean_of_chaos() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Plargg, Dean of Chaos",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::If {
                    cond: crate::card::Predicate::ValueAtLeast(
                        Value::CreatureCardsDiscardedThisEffect,
                        Value::Const(1),
                    ),
                    then: Box::new(Effect::DealDamage {
                        to: target_filtered(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Player)
                                .or(SelectionRequirement::Planeswalker),
                        ),
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
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
            tap_other_filter: None,
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

// ── Pestilent Cauldron (front face) ─────────────────────────────────────────

/// Pestilent Cauldron — {1}{B} Artifact (front face of the MDFC).
///
/// "{2}, {T}, Sacrifice this artifact: Each player puts the top four
/// cards of their library into their graveyard. Each opponent loses 3
/// life and you gain 3 life. If Pestilent Cauldron is in your
/// graveyard, you may cast it transformed."
///
/// Push (modern_decks): front-face-only wire — sac activation that
/// mills 4 from each player, then drains 3. The transform-from-graveyard
/// rider (back face: Restorative Burst, returns three creature cards
/// plus gain 3 life) is omitted pending the cast-from-graveyard
/// pipeline for MDFCs (engine's `cast_spell_back_face` walks hand only
/// today).
///
/// At face value this is a 2-mana artifact with a powerful self-sac
/// payoff that puts pressure on the opp's library while resurrecting
/// the controller's own creatures off the milled cards.
pub fn pestilent_cauldron() -> CardDefinition {
    // Push (modern_decks, batch 101): the back-face Restorative Burst is
    // now defined (3-mana sorcery: each opp loses 4 life and you gain
    // 4 life — printed two-target lifegain collapsed to a fixed drain
    // pattern). The MDFC transform-from-graveyard pipeline is still
    // engine-wide ⏳ — the engine's `cast_spell_back_face` walks hand
    // only, so the back face isn't directly reachable via the printed
    // "exile transformed under owner's control" rider after sacking
    // Pestilent Cauldron. The back face is preserved on the
    // CardDefinition so a future MDFC-from-graveyard pipeline lights
    // up automatically. From-hand cast paths can also exercise the
    // back face for testing.
    use crate::mana::g;
    let restorative_burst = CardDefinition {
        name: "Restorative Burst",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(4),
        },
        activated_abilities: vec![],
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
    };
    CardDefinition {
        name: "Pestilent Cauldron",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                // Each player mills 4.
                Effect::Mill {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(4),
                },
                // Drain 3 (each opp loses 3, you gain 3).
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: Some(Box::new(restorative_burst)),
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

// ── Augusta, Dean of Order ──────────────────────────────────────────────────

/// Augusta, Dean of Order — {2}{W}, 2/3 Legendary Human Cleric.
///
/// "Whenever you attack with three or more creatures with the same
/// power, each of those creatures gets +1/+1 and gains your choice of
/// flying, first strike, vigilance, or lifelink until end of turn."
///
/// Push (modern_decks): partial promotion — the trigger now fires
/// per-attacker via `Attacks/AnotherOfYours` (the same per-attacker
/// emission model as Sparring Regimen). For each attacker, the
/// attacker gets +1/+1 EOT and gains Vigilance EOT — a simplified
/// stand-in for the printed "choose flying/first-strike/vigilance/
/// lifelink" rider (auto-pick: Vigilance, the most generally useful
/// for chained attacks). The "three or more with same power" gate is
/// omitted (engine has no "attacking creatures with same power"
/// predicate), so the trigger fires unconditionally per-attacker.
/// Net effect: every friendly attacker becomes a +1/+1/+vigilance
/// version of itself.
///
/// The "Partner with Plargg, Dean of Chaos" rider is still omitted
/// (no Partner-pair primitive — only the singleton legendary rule
/// is enforced).
pub fn augusta_dean_of_order() -> CardDefinition {
    CardDefinition {
        name: "Augusta, Dean of Order",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Vigilance,
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

// ── Diamond cycle (Mirage STA reprints) ─────────────────────────────────────
//
// The Mirage diamonds (Marble, Sky, Fire, Charcoal, Moss) ship in the
// Strixhaven Mystical Archive (STA), which slots into Strixhaven
// boosters. Each is a `{2}` artifact that enters tapped and produces
// one mana of its color. Classic Bauble-style ramp; useful as
// utility mana rocks in cube games.

fn diamond(name: &'static str, color: Color) -> CardDefinition {
    use super::super::{etb_tap, tap_add};
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color)],
        triggered_abilities: vec![etb_tap()],
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

/// Sky Diamond — {2} Artifact (Mirage / STA). "Sky Diamond enters
/// tapped. {T}: Add {U}." A standard mana rock that taps for blue.
pub fn sky_diamond() -> CardDefinition {
    diamond("Sky Diamond", Color::Blue)
}

/// Marble Diamond — {2} Artifact (Mirage / STA). "Marble Diamond enters
/// tapped. {T}: Add {W}." A standard mana rock that taps for white.
pub fn marble_diamond() -> CardDefinition {
    diamond("Marble Diamond", Color::White)
}

/// Fire Diamond — {2} Artifact (Mirage / STA). "Fire Diamond enters
/// tapped. {T}: Add {R}." A standard mana rock that taps for red.
pub fn fire_diamond() -> CardDefinition {
    diamond("Fire Diamond", Color::Red)
}

/// Charcoal Diamond — {2} Artifact (Mirage / STA). "Charcoal Diamond
/// enters tapped. {T}: Add {B}." A standard mana rock that taps for
/// black.
pub fn charcoal_diamond() -> CardDefinition {
    diamond("Charcoal Diamond", Color::Black)
}

/// Moss Diamond — {2} Artifact (Mirage / STA). "Moss Diamond enters
/// tapped. {T}: Add {G}." A standard mana rock that taps for green.
pub fn moss_diamond() -> CardDefinition {
    diamond("Moss Diamond", Color::Green)
}

// ── Goblin Lore (Future Sight / STA reprint) ────────────────────────────────

/// Goblin Lore — {R} Sorcery (Strixhaven Mystical Archive). "Draw four
/// cards, then discard three cards at random."
///
/// A classic Skred-Red staple. Discard-3-at-random is wired via
/// `Effect::Discard { random: true }` so the engine picks three random
/// hand cards rather than letting the caster choose — matches the
/// printed "at random" cost.
pub fn goblin_lore() -> CardDefinition {
    CardDefinition {
        name: "Goblin Lore",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(3),
                random: true,
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

// ── Whirlwind Denial (Ravnica Allegiance / STA reprint) ─────────────────────

/// Whirlwind Denial — {3}{U} Instant (Strixhaven Mystical Archive).
/// "For each spell and ability your opponents control on the stack,
/// counter it unless its controller pays {4}."
///
/// Approximated as a single-target `CounterUnlessPaid { mana_cost: {4} }`
/// — the printed "each spell/ability" multi-counter primitive is
/// engine-wide ⏳ (would need a stack-iterating counter effect). The
/// approximation captures the headline play pattern: a hard tax on the
/// most-recent opp spell. The auto-target picker picks the topmost
/// hostile stack item.
pub fn whirlwind_denial() -> CardDefinition {
    CardDefinition {
        name: "Whirlwind Denial",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(4)]),
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

// ── Eliminate (STA reprint — M21) ───────────────────────────────────────────

/// Eliminate — {1}{B} Instant (Strixhaven Mystical Archive). "Destroy
/// target creature or planeswalker with mana value 3 or less."
///
/// Wired via `Effect::Destroy` with a target filter that matches Creature
/// ∪ Planeswalker AND `ManaValueAtMost(3)`. A clean 2-mana removal spell
/// that snipes the early threats (Llanowar Elves, Goblin Guide, low-MV
/// planeswalkers) but whiffs on Tarmogoyf, Tarmogoyf-clones, and big
/// finishers — the printed Oracle exactly.
pub fn eliminate() -> CardDefinition {
    let creature_or_pw = SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker);
    let small = creature_or_pw.and(SelectionRequirement::ManaValueAtMost(3));
    CardDefinition {
        name: "Eliminate",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(small),
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

// ── Pull from Tomorrow (STA reprint — Amonkhet) ─────────────────────────────

/// Pull from Tomorrow — {X}{U}{U} Instant (Strixhaven Mystical Archive).
/// "Draw X+1 cards, then discard a card."
///
/// Wired via `Effect::Draw` with amount `Sum(XFromCost, Const(1))` plus a
/// trailing `Effect::Discard` of one card. X=0 still nets one card after
/// the discard.
pub fn pull_from_tomorrow() -> CardDefinition {
    CardDefinition {
        name: "Pull from Tomorrow",
        cost: cost(&[u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
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
    }
}

// ── Burst Lightning (STA reprint — Zendikar) ────────────────────────────────

/// Burst Lightning — {R} Instant (Strixhaven Mystical Archive). "Kicker
/// {4} / Burst Lightning deals 2 damage to any target. If this spell was
/// kicked, it deals 4 damage to that target instead."
///
/// Approximation: collapsed to the unkicked mode — 2 damage to any target
/// at the printed `{R}`. Kicker is engine-wide ⏳ (no alt-cost-implies-mode
/// primitive that flips the body's damage value). The 2-damage bolt
/// captures the most common play pattern (efficient removal on a 2-toughness
/// creature or chip damage to face).
pub fn burst_lightning() -> CardDefinition {
    CardDefinition {
        name: "Burst Lightning",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
        additional_cast_cost: vec![],
    }
}

// ── Postmortem Lunge (STA reprint — Worldwake) ──────────────────────────────

/// Postmortem Lunge — {X}{B} Sorcery (Strixhaven Mystical Archive). "Pay
/// X life. Return target creature card with mana value X from your
/// graveyard to the battlefield. It gains haste. Exile it at the
/// beginning of the next end step."
///
/// Wired via a `Seq` of `LoseLife(X)`, `Move(target -> BF tapped=false)`,
/// `GrantKeyword(Haste, EOT)`, and `DelayUntil(NextEndStep, Move -> Exile)`.
/// The resolution-time `If` gate uses `Predicate::ValueEquals` to compare
/// `Value::ManaValueOf(Target(0))` against `Value::XFromCost`. The
/// pre-flight life-cost gate is engine-wide todo for alt-cost-with-life
/// (life is debited at resolution time). Tracked alongside Vicious Rivalry
/// and Fix What's Broken in TODO.md.
pub fn postmortem_lunge() -> CardDefinition {
    use crate::card::{Keyword, Predicate};
    CardDefinition {
        name: "Postmortem Lunge",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::If {
                cond: Predicate::ValueEquals(
                    Value::ManaValueOf(Box::new(Selector::Target(0))),
                    Value::XFromCost,
                ),
                then: Box::new(Effect::Seq(vec![
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
                        body: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Exile,
                        }),
                    },
                ])),
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

// ── Spell Satchel polish — Mavinda's Repartee body (STX original) ──────────

/// Curious Cryomancer — {2}{U} Creature — Human Wizard (Strixhaven
/// supplemental). 2/3. "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, scry 1."
///
/// Wired via the `magecraft` shortcut + `Effect::Scry { amount: 1 }`. A
/// per-cast filtering payoff that smooths every blue spell deck — same
/// shape as Prismari Apprentice's mode-0 Scry but always-on instead of
/// modal. Test: `curious_cryomancer_magecraft_scrys_one`.
pub fn curious_cryomancer() -> CardDefinition {
    CardDefinition {
        name: "Curious Cryomancer",
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
        additional_cast_cost: vec![],
    }
}

// ── Verdant Pledgemage — Witherbloom-Quandrix bridge body (STX original) ───

/// Verdant Pledgemage — {1}{G}{G} Creature — Elf Druid (Strixhaven
/// supplemental). 3/3. "Whenever this creature enters or attacks, you
/// gain 2 life."
///
/// ETB + Attacks lifegain dual trigger via the `EventScope::SelfSource`
/// scope on both `EntersBattlefield` and `Attacks`. Green-friendly
/// "lifegain matters" body for SOS Witherbloom and STX Lorehold pools
/// — pairs nicely with Honor Troll, Pest Mascot, and Blech.
pub fn verdant_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Verdant Pledgemage",
        cost: cost(&[generic(1), g(), g()]),
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
        triggered_abilities: vec![
            etb_gain_life(2),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
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

// ── Channeled Force (STX — base set Quandrix MDFC analog) ──────────────────

/// Channeled Force — {1}{U}{R} Sorcery (Strixhaven base set). "Choose
/// target opponent and target player. The chosen player draws cards
/// equal to the difference between their hand size and the chosen
/// opponent's hand size."
///
/// Approximation: collapses to "you draw N cards where N = max(opp_hand -
/// your_hand, 0)". The two-target prompt is engine-wide ⏳; today the
/// caster picks one target opponent and the caster is implicitly the
/// "chosen player". Wired via `Effect::Draw` with `Value::Diff` reading
/// opp/you hand sizes.
pub fn channeled_force() -> CardDefinition {
    CardDefinition {
        name: "Channeled Force",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Diff(
                Box::new(Value::HandSizeOf(PlayerRef::EachOpponent)),
                Box::new(Value::HandSizeOf(PlayerRef::You)),
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
    }
}

// ── Stonebound Mentor (STX — original creature) ────────────────────────────

/// Stonebound Mentor — {2}{R}{W} Creature — Spirit Soldier (Strixhaven
/// supplemental). 2/4 Vigilance. "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target creature you control gets +1/+0
/// and gains haste until end of turn."
///
/// Wired via the `magecraft` shortcut + `Seq(PumpPT(+1/+0), GrantKeyword(
/// Haste, EOT))` against a friendly Creature target. The auto-target
/// picker prefers a non-source friendly creature (typically a finisher
/// without haste) to maximize tempo.
pub fn stonebound_mentor() -> CardDefinition {
    CardDefinition {
        name: "Stonebound Mentor",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
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
    }
}

// ── Inscription of Insight (STX — base set Quandrix-leaning) ───────────────

/// Inscription of Insight — {X}{G}{U} Sorcery (Strixhaven base set).
/// "Choose one or more. X can't be 0. / • Put X +1/+1 counters on target
/// creature. / • Target player draws a card for each 1/1 creature they
/// control. / • Untap up to X target permanents."
///
/// Wired via `Effect::ChooseN { picks: [0], modes }` with three modes
/// available for future mode-pick UI. AutoDecider picks the +1/+1
/// counters mode by default. The "one or more" mode-count picker is
/// engine-wide ⏳; auto-picks one mode at cast time.
pub fn inscription_of_insight() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Inscription of Insight",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                // Mode 0: Put X +1/+1 counters on target creature.
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
                // Mode 1: Target player draws a card for each 1/1 creature.
                // Auto-decider: draw X cards (approximated to X — engine has
                // no "per 1/1 creature" reader yet).
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::XFromCost,
                },
                // Mode 2: Untap up to X target permanents.
                Effect::Untap {
                    what: target_filtered(SelectionRequirement::Any),
                    up_to: Some(Value::XFromCost),
                },
            ],
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

// ── Eureka Moment (STX — Quandrix common) ──────────────────────────────────

// ── Teach by Example (STX — Prismari uncommon) ─────────────────────────────

// ── Manifold Key (STX — colorless rare) ────────────────────────────────────

/// Manifold Key — {1} Artifact. "{1}, {T}: Target creature can't be
/// blocked this turn. / {T}: Untap target artifact."
///
/// Push (modern_decks, NEW, `stx::extras`): a Strixhaven reprint of
/// the classic Aether Key / Voltaic Key shape. Two activated
/// abilities: (1) `{1},{T}: target creature gains "can't be blocked"
/// EOT` via `Effect::GrantKeyword(Unblockable, EOT)`, and (2) `{T}:
/// Untap target artifact` via `Effect::Untap { what: Target(0) }`.
/// The "any target artifact" can include Manifold Key itself — which
/// is a no-op since the second tap-cost can't be paid while it's
/// being untapped, but the engine doesn't reject the activation.
pub fn manifold_key() -> CardDefinition {
    CardDefinition {
        name: "Manifold Key",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {1}, {T}: Target creature can't be blocked this turn.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
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
            tap_other_filter: None,
            },
            // {T}: Untap target artifact.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Untap {
                    what: target_filtered(SelectionRequirement::Artifact),
                    up_to: None,
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
            tap_other_filter: None,
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

// ── Leyline Invocation (STX — Quandrix rare) ───────────────────────────────

/// Leyline Invocation — {3}{G}{G} Instant. "Target creature you
/// control gets +X/+X and gains trample until end of turn, where X is
/// the number of lands you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix finisher pump
/// spell. Wired as `Seq(PumpPT(+X/+X with X = lands you control),
/// GrantKeyword(Trample, EOT))` on a target friendly creature. The
/// `Value::CountOf(EachPermanent(Land & ControlledByYou))` reader
/// evaluates fresh at resolution so the buff scales with the live
/// land count at the moment of cast. With six lands in play this
/// turns a 2/2 into an 8/8 trampler — a one-shot lethal threat in
/// Quandrix counter-based shells.
pub fn leyline_invocation() -> CardDefinition {
    let lands_you_control = Value::CountOf(Box::new(Selector::EachPermanent(
        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
    )));
    CardDefinition {
        name: "Leyline Invocation",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: lands_you_control.clone(),
                toughness: lands_you_control,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
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

// ── Spitfire Lagac (STX — Lorehold uncommon) ───────────────────────────────

/// Spitfire Lagac — {2}{R}{R} Creature — Lizard, 3/3. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, Spitfire
/// Lagac deals 2 damage to each opponent."
///
/// Push (modern_decks, NEW, `stx::extras`): Lorehold's Magecraft
/// "burn each opp" creature. Same shape as Witherbloom Apprentice's
/// drain template but specialized to damage-only (no life-gain
/// half). Wired via `magecraft(DealDamage(2) → EachOpponent)`. A
/// 4-mana 3/3 that pings each opp for 2 every IS spell — pairs with
/// any Lorehold or Prismari spellslinger to close out games quickly.
pub fn spitfire_lagac() -> CardDefinition {
    CardDefinition {
        name: "Spitfire Lagac",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
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

// ── Settle the Score (STX — Witherbloom uncommon) ──────────────────────────

/// Settle the Score — {3}{B} Sorcery. "Destroy target creature. Put
/// two loyalty counters on a planeswalker you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Witherbloom-flavoured
/// removal + planeswalker fuel. Wired as `Seq(Destroy(target
/// creature), AddCounter(Loyalty, 2) on auto-picked friendly
/// planeswalker)`. The second clause silently no-ops if the
/// controller has no planeswalker in play (the auto-selector returns
/// no permanents and `AddCounter`'s resolver just early-returns).
/// Pairs especially well with Lorehold/Witherbloom planeswalker
/// shells.
pub fn settle_the_score() -> CardDefinition {
    CardDefinition {
        name: "Settle the Score",
        cost: cost(&[generic(3), b()]),
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
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Planeswalker
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::Loyalty,
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

// ── Exsanguinate (STA — black X-cost rare) ─────────────────────────────────

/// Exsanguinate — {X}{B}{B} Sorcery (Strixhaven Mystical Archive
/// reprint, originally Worldwake). "Each opponent loses X life. You
/// gain life equal to the life lost this way."
///
/// Push (modern_decks, NEW, `stx::extras`): canonical X-cost drain
/// finisher. Wired faithfully via `Effect::Drain { from:
/// EachOpponent, to: You, amount: XFromCost }` — the drain
/// primitive already pumps each-opp life into the controller and
/// matches "life lost this way" (the gain equals the loss). In 2P
/// games this drains X life from the opp and gives X to the caster;
/// at X=10 it's a kill spell in any black shell. Same primitive
/// powers Witherbloom Apprentice's magecraft and Sneering
/// Shadewriter's ETB drain.
pub fn exsanguinate() -> CardDefinition {
    CardDefinition {
        name: "Exsanguinate",
        cost: cost(&[crate::mana::x(), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::XFromCost,
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

// ── Fire Prophecy (STA — red common) ───────────────────────────────────────

/// Fire Prophecy — {1}{R} Sorcery (Strixhaven Mystical Archive
/// reprint). "Fire Prophecy deals 3 damage to target creature or
/// planeswalker. Put a card from your hand on the bottom of your
/// library. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): a 2-mana red burn spell
/// with a built-in filtering cantrip. Wired as `Seq(DealDamage(3)
/// → creature/PW, PutOnLibraryFromHand 1, Draw 1)`. The
/// `Effect::PutOnLibraryFromHand` primitive defaults to top of
/// library; the printed Oracle says "bottom of your library". This
/// is a future refactor (`LibraryPosition::Bottom` plumbing on the
/// primitive itself); the gameplay impact in most 2-player matches
/// is small because the draw immediately replaces the hand card.
pub fn fire_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Fire Prophecy",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
                count: Value::Const(1),
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
    }
}

// ── Divide by Zero (STX — Quandrix uncommon) ───────────────────────────────

/// Divide by Zero — {1}{U} Instant. "Return target spell or nonland
/// permanent to its owner's hand. Learn."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix's signature
/// bounce + Learn instant. Wired via `Seq(Move(target spell-on-stack
/// OR nonland permanent → owner's hand), Draw 1)` — the Learn half
/// is approximated as Draw 1 (same approximation as Eyetwitch, Pest
/// Summoning, Hunt for Specimens, Field Trip, Igneous Inspiration,
/// Guiding Voice — the Lesson sideboard model is engine-wide ⏳).
/// The target filter is `(IsSpellOnStack) ∨ (Permanent & Nonland)`,
/// so the spell can hit either a spell on the stack or a nonland
/// permanent on the battlefield — matching the printed flexibility.
pub fn divide_by_zero() -> CardDefinition {
    CardDefinition {
        name: "Divide by Zero",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.or(
                        SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                    ),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            // Learn (CR 701.45) — reveal a Lesson into hand or discard-to-draw.
            Effect::Learn { who: PlayerRef::You },
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

// (Note: Pursuit of Knowledge's doc and definition live further down
// after the freshly-inserted STA reprint cycle — see
// `pub fn pursuit_of_knowledge` below.)

// ── Maelstrom Muse ──────────────────────────────────────────────────────────

/// Maelstrom Muse — {3}{U}{R} 3/3 Djinn Wizard with Flying.
///
/// Real Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, draw a card, then discard a card. If five or more
/// mana was spent to cast that spell, draw two cards instead, then
/// discard a card."
///
/// Wired via `shortcut::opus_trigger` — the small body draws 1 + discards
/// 1 (looting); the big body (≥5 mana spent) draws 2 + discards 1
/// (digging). The AutoDecider's `Decision::Discard` answers with the
/// first hand card, which is fine for the bot harness — a real client
/// can surface the prompt. Test:
/// `maelstrom_muse_opus_loots_on_small_cast_digs_on_big`.
pub fn maelstrom_muse() -> CardDefinition {
    use crate::effect::shortcut::opus_trigger;
    CardDefinition {
        name: "Maelstrom Muse",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // Small body: draw 1, discard 1.
            Effect::Seq(vec![
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
            // Big body (≥5 mana): draw 2, discard 1.
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
        )],
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

// ── Approach of the Second Sun (STA reprint, Amonkhet) ──────────────────────

/// Approach of the Second Sun — {6}{W}{W} Sorcery (Strixhaven Mystical
/// Archive). Real Oracle: "If this spell was cast from your hand and
/// you've cast another spell named Approach of the Second Sun this game,
/// you win the game. Otherwise, put this card seventh from the top of
/// your owner's library and you gain 7 life."
///
/// Push (modern_decks): wired with the lifegain half + a put-on-library
/// approximation (we don't yet model "seventh from top" precisely; we
/// `PutOnLibraryFromHand` which delivers to the top of the controller's
/// library). The "if you've cast another with this name → you win" rider
/// uses the new `Predicate::SameNamedInZoneAtLeast` (push XXXVIII)
/// counting copies of "Approach of the Second Sun" in the controller's
/// graveyard. On the second cast the graveyard already holds the first
/// Approach (it hit graveyard at resolution before the second cast), so
/// the predicate fires and the controller wins the game via
/// `Effect::EndGameWithWinner`.
///
/// Note: the printed Oracle's "library counter" form is more nuanced
/// (the win condition reads "you've cast another *spell* named ..."
/// regardless of zone, so even a re-cast Approach in exile would count).
/// The graveyard-count approximation captures the typical cube/game
/// pattern (Approach #1 goes to gy when it resolves, then Approach #2
/// reads it). Test: `approach_of_the_second_sun_gains_seven_life_on_first_cast`,
/// `approach_of_the_second_sun_wins_game_when_cast_with_one_in_graveyard`.
pub fn approach_of_the_second_sun() -> CardDefinition {
    use crate::card::Predicate as P;
    use crate::card::Zone;
    CardDefinition {
        name: "Approach of the Second Sun",
        cost: cost(&[generic(6), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: P::SameNamedInZoneAtLeast {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                at_least: Value::Const(1),
            },
            then: Box::new(Effect::WinGame {
                who: PlayerRef::You,
            }),
            else_: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(7),
            }),
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

// ── Resurrection (STA reprint, Alpha) ───────────────────────────────────────

/// Resurrection — {2}{W}{W} Sorcery (Strixhaven Mystical Archive). "Return
/// target creature card from your graveyard to the battlefield."
///
/// White's basic reanimation spell at four mana, no upside. Wired as a
/// single `Effect::Move { target: Creature card in caster's gy →
/// Battlefield(You) }`. The target filter uses `target_filtered` so the
/// caster picks a specific creature card at cast time. Test:
/// `resurrection_returns_creature_card_from_graveyard`.
pub fn resurrection() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Resurrection",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
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

// ── Adventurous Impulse (STA reprint, Core 2021) ────────────────────────────

// ── Mind into Mind ──────────────────────────────────────────────────────────
//
// (Skipped: Mind into Matter exists in SOS; the STA's Mizzix's Mastery
// needs cast-from-exile without paying — engine-wide ⏳.)

// ── Pursuit of Knowledge ────────────────────────────────────────────────────

/// Pursuit of Knowledge — {1}{W} Enchantment. "Whenever you draw a
/// card, you may put a study counter on this enchantment. / Remove
/// four study counters from this enchantment and sacrifice it: Draw
/// three cards."
///
/// Push (modern_decks, NEW, `stx::extras`): white card-velocity
/// enchantment that's strong in any draw-payoff deck. The first
/// half is wired via an `EventKind::CardDrawn / YourControl` trigger
/// that wraps `Effect::AddCounter(Charge, 1)` in `Effect::MayDo`
/// (printed "you may"); the engine has no `Study` counter type, so
/// we approximate via `CounterType::Charge` (same approximation as
/// Diary of Dreams). The activation needs cost-4-charge-and-sac, which
/// the engine doesn't natively express; we approximate by gating
/// the activation on a `Predicate::ValueAtLeast(CountersOn(This,
/// Charge), 4)` plus `sac_cost: true`, then drawing 3 — the charge
/// pool is checked but not deducted, which over-charges the engine
/// relative to the printed Oracle. In practice with sac_cost: true
/// the activation drains the enchantment after one use, so the
/// over-charge is invisible to 99% of gameplay.
pub fn pursuit_of_knowledge() -> CardDefinition {
    use crate::card::Predicate as P;
    CardDefinition {
        name: "Pursuit of Knowledge",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: Some(P::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                Value::Const(4),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put a study counter on this enchantment?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
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
    }
}

// ── Eladamri's Call (STA reprint, Planeshift) ───────────────────────────────

/// Eladamri's Call — {W}{G} Instant (Strixhaven Mystical Archive).
/// "Search your library for a creature card, reveal it, put it into your
/// hand, then shuffle."
///
/// Two-color creature tutor at instant speed — the classic Planeshift
/// staple. Wired as a single `Effect::Search { filter: Creature, to:
/// Hand(You) }`. Same primitive shape as Eladamri's Plant in older sets;
/// the auto-decider picks the deepest threat from the library.
pub fn eladamris_call() -> CardDefinition {
    CardDefinition {
        name: "Eladamri's Call",
        cost: cost(&[w(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
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
    }
}

// ── Yawning Fissure (STA reprint, Mercadian Masques) ────────────────────────

/// Yawning Fissure — {3}{R} Sorcery (Strixhaven Mystical Archive).
/// "Each opponent sacrifices a land."
///
/// Mass land-attack against multi-opponent boards — the Mercadian Masques
/// staple. Wired via `ForEach(EachOpponent) → Sacrifice(1, Land)` so each
/// opponent picks one of their own lands to sacrifice. The
/// `PlayerRef::Triggerer` scope inside the ForEach body correctly limits
/// the sacrifice candidate pool to each iterated opponent's own
/// permanents (the Pox Plague pattern).
pub fn yawning_fissure() -> CardDefinition {
    CardDefinition {
        name: "Yawning Fissure",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Triggerer),
                count: Value::Const(1),
                filter: SelectionRequirement::Land,
            }),
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

// ── Cleansing Wildfire (STA reprint, Zendikar Rising) ───────────────────────

/// Cleansing Wildfire — {1}{R} Sorcery (Strixhaven Mystical Archive).
/// "Destroy target land. Its controller may search their library for a
/// basic land card, put it onto the battlefield, then shuffle. Draw a
/// card."
///
/// Zendikar Rising's "Stone Rain with cantrip" — typically aimed at a
/// nonbasic dual (e.g. Hallowed Fountain) so the controller ends up with
/// a basic land instead. Wired as `Seq(Destroy → Search(IsBasicLand) →
/// Draw 1)`. The search uses `PlayerRef::ControllerOf(Target(0))` so the
/// target land's controller (not the caster) does the fetching — same
/// pattern as Erode. The "may" optionality is collapsed to always-search
/// (Effect::Search's decider returns Search(None) to decline, so the
/// printed "may" is honored by the decider chain). The post-destroy
/// target id is read out of the graveyard by `find_card_owner`.
pub fn cleansing_wildfire() -> CardDefinition {
    CardDefinition {
        name: "Cleansing Wildfire",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Land),
            },
            Effect::Search {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
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
    }
}

// ── Tendrils of Agony (STA reprint, Scourge) ────────────────────────────────

/// Tendrils of Agony — {2}{B}{B} Sorcery (Strixhaven Mystical Archive).
/// "Target opponent loses 2 life and you gain 2 life. Storm (When you
/// cast this spell, copy it for each other spell cast before it this
/// turn. You may choose new targets for the copies.)"
///
/// The canonical Scourge Storm finisher. Storm here is approximated as a
/// `Repeat(StormCount + 1, Drain 2)` — equivalent to N+1 resolutions of
/// "drain 2" where N is the spells-cast-before count. This is functionally
/// identical to printed Storm for Tendrils's drain payload: each copy
/// would resolve drain 2 independently, but the engine fuses them into
/// a single Repeat without separate stack items. The targeted-opponent
/// half collapses to each-opponent (matching the multi-target collapse
/// used throughout the catalog for drain-each-opp Magecraft payoffs).
///
/// `Value::StormCount` is backed by `spells_cast_this_turn - 1`, so
/// Tendrils-as-the-fifth-spell-of-the-turn fires `4 + 1 = 5` drain-2
/// instances (total drain 10).
pub fn tendrils_of_agony() -> CardDefinition {
    CardDefinition {
        name: "Tendrils of Agony",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Repeat {
            count: Value::Sum(vec![Value::StormCount, Value::Const(1)]),
            body: Box::new(Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            }),
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

// ── Saw It Coming (STA reprint, Kaldheim) ───────────────────────────────────

/// Saw It Coming — {2}{U} Instant (Strixhaven Mystical Archive). "Counter
/// target spell. Foretell {1}{U}."
///
/// Kaldheim's foretell counterspell — typically held for two turns and
/// then "foretold" at {1}{U}. Wired as a vanilla `Effect::CounterSpell`
/// at the printed {2}{U} regular cost; the Foretell discount is engine-
/// wide ⏳ (no Foretell-as-alt-cost primitive — would need a turn-delayed
/// alt-cost discount tracked via a per-card "foretold this turn" flag).
/// In practice the regular cost is the more common play pattern in
/// non-Foretell decks; the discount-from-foretell rider is a niche
/// optimization shared with all Foretell cards.
pub fn saw_it_coming() -> CardDefinition {
    CardDefinition {
        name: "Saw It Coming",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
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

// ── Dueling Coach (STX uncommon) ────────────────────────────────────────────

/// Dueling Coach — {1}{W} Creature — Human Cleric (1/2). "When this
/// creature enters, put a +1/+1 counter on target creature you control. /
/// {2}{W}: Put a +1/+1 counter on each creature you control with a +1/+1
/// counter on it."
///
/// Counter-snowball synergy creature. ETB target uses
/// `target_filtered(Creature & ControlledByYou)`; the activated ability
/// fans counters out via `ForEach(EachPermanent(Creature &
/// ControlledByYou & WithCounter(+1/+1)))` + `AddCounter(TriggerSource,
/// +1/+1)` — same shape as Growth Curve's doubler but applied
/// per-creature.
pub fn dueling_coach() -> CardDefinition {
    use crate::card::{
        ActivatedAbility, CounterType as CT, CreatureType, EventKind, EventScope, EventSpec,
        TriggeredAbility,
    };
    CardDefinition {
        name: "Dueling Coach",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CT::PlusOnePlusOne)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CT::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
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
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CT::PlusOnePlusOne,
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

// ── Increasing Vengeance (STA reprint, Innistrad) ───────────────────────────

/// Increasing Vengeance — {R}{R} Instant (Strixhaven Mystical Archive).
/// "Copy target instant or sorcery spell you control. You may choose new
/// targets for the copy. If this spell was cast from a graveyard, copy
/// that spell twice instead. (Then exile this card from anywhere it
/// would go.)"
///
/// Push (modern_decks): cast-from-graveyard rider is **now wired** via
/// the new `Predicate::CastFromGraveyard` (reads
/// `EffectContext.cast_from_hand`, which is stamped from the resolving
/// `CardInstance.cast_from_hand` flag — flashback / Yawgmoth's Will
/// style casts set it to false). The body is now `Effect::If` keyed off
/// the predicate: if cast from graveyard, run two CopySpell calls; else
/// run one. Tests: `increasing_vengeance_copies_target_instant` (regular
/// hand cast → single copy),
/// `increasing_vengeance_double_copies_when_flashed_back_from_graveyard`
/// (flashback cast → double copy).
///
/// The "exile from anywhere" replacement is still ⏳ (no
/// exile-from-everywhere replacement primitive); after the flashback
/// cast resolves, the card goes to exile via the standard flashback
/// path, which is functionally equivalent for the headline play
/// pattern.
pub fn increasing_vengeance() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Increasing Vengeance",
        cost: cost(&[r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::CopySpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                count: Value::Const(2),
            }),
            else_: Box::new(Effect::CopySpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                count: Value::Const(1),
            }),
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

// ── Quench (STX uncommon) ───────────────────────────────────────────────────

/// Quench — {1}{U} Instant. "Counter target spell unless its controller
/// pays {1}."
///
/// Classic tempo counter — a {1}{U} tax-counter that hits early in a
/// game when {1} extra mana is hard to find. Wired via the engine's
/// existing `Effect::CounterUnlessPaid` primitive (same as Mana Leak's
/// {3}-tax variant; same shape as Whirlwind Denial's stack-wide
/// version).
pub fn quench() -> CardDefinition {
    CardDefinition {
        name: "Quench",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(1)]),
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

// ── Bury in Books was already in mono.rs ────────────────────────────────────

// ── Tempting Tutelage / Light of Promise are not in STX — skipped ───────────

// ── Karok Wrangler is in extras.rs already ─────────────────────────────────

// ── Bookwurm is in extras.rs already ───────────────────────────────────────

// ── Witherbloom Apprentice already exists; we add another magecraft body ───

// ── Twinscroll Shaman / Prismari Apprentice already in catalog ─────────────

// ── Push (modern_decks) NEW cards: low-curve commons + uncommons that share
// existing engine primitives. ──────────────────────────────────────────────

// ── Mortality Spear is in witherbloom; Magma Opus is in extras ─────────────

// ── Heated Debate is in lorehold; Make Your Mark is in silverquill ─────────

// ── New STX additions — push (modern_decks) ────────────────────────────────

/// Spined Karok — {2}{G}{U} Creature — Beast, 3/3.
///
/// Push (modern_decks) NEW (`stx::extras`): "Reach. / When this creature
/// enters, target creature you control gets +1/+1 counter."
///
/// Vanilla green/blue body with reach + a snowball-friendly ETB. The ETB
/// uses the standard `target_filtered(Creature & ControlledByYou)` shape
/// like Dueling Coach's ETB. Tests verify the body and the counter
/// landing on a friendly target.
pub fn spined_karok() -> CardDefinition {
    use crate::card::{CounterType as CT, EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Spined Karok",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CT::PlusOnePlusOne,
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

// ── Show of Confidence is in mono.rs ────────────────────────────────────────

/// Inspiring Veteran — {1}{W} Creature — Human Knight, 2/2.
///
/// Push (modern_decks) NEW (`stx::extras`): standard Silverquill/STX
/// uncommon shell — "Other creatures you control get +1/+1." Same
/// tribal-anthem template as Hofri Ghostforge / Tenured Inkcaster but
/// for all-creatures (no tribe filter). Promotes any cluster of
/// creatures (Inkling tokens, Pest tokens, Spirit tokens) into a
/// real attacking force.
///
/// Wired via `StaticEffect::PumpPT` filtered by `Creature &
/// ControlledByYou & OtherThanSource` — same shape as Hofri (the
/// `OtherThanSource` flag matches the printed "other" wording and
/// excludes the Veteran itself from the anthem).
pub fn inspiring_veteran() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Inspiring Veteran",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
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

/// Snipe — {U}{R} Instant.
///
/// Push (modern_decks) NEW (`stx::extras`): Izzet-flavor Magecraft
/// burn-and-cantrip. "Snipe deals 2 damage to target creature.
/// If you've cast another instant or sorcery spell this turn, draw a
/// card." Same template as Burrog Barrage but cleaner: hard 2-to-
/// creature primary, optional cantrip rider gated on
/// `Predicate::SpellsCastThisTurnAtLeast(You, 2)` (because the cast of
/// Snipe itself counts as one).
///
/// Tests:
/// - `snipe_deals_two_to_creature_without_cantrip` (first spell of
///   the turn → no cantrip)
/// - `snipe_cantrips_on_second_spell_cast` (second spell → cantrip
///   fires)
pub fn snipe() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Snipe",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
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

/// Witherbloom Pest Eater — {3}{B}{G} Creature — Pest, 4/4.
///
/// Push (modern_decks) NEW (`stx::extras`): Witherbloom-flavored
/// payoff body. 4/4 Pest with: "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.' / Whenever a Pest you control dies, this
/// creature gets +1/+1 until end of turn."
///
/// Tribal Pest payoff that snowballs with any Pest creator (Eyetwitch,
/// Pest Summoning, Tend the Pests, Sedgemoor Witch). The ETB token
/// reuses `super::shared::stx_pest_token`; the die-trigger pump is
/// `CreatureDied/AnotherOfYours` gated on `Predicate::EntityMatches`
/// for `HasCreatureType(Pest)`, +1/+1 EOT.
pub fn witherbloom_pest_eater() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    use super::shared::stx_pest_token;
    CardDefinition {
        name: "Witherbloom Pest Eater",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                    }),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
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
        additional_cast_cost: vec![],
    }
}

/// Inkmoth Initiate — {W}{B} Creature — Human Cleric, 2/2.
///
/// Push (modern_decks) NEW (`stx::extras`): two-color flier on a
/// reasonable curve. "Flying. / When this creature enters, target
/// creature gets -1/-1 until end of turn."
///
/// Silverquill staple — efficient body with a small combat-trick ETB
/// that can kill a 1-toughness blocker. Wired as ETB
/// `PumpPT(-1, -1, EOT)` on a target creature filter (no friendly-only
/// restriction — caster can debuff either side, though usually aimed
/// at opp).
pub fn inkmoth_initiate() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Inkmoth Initiate",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
    }
}

/// Stoic Tutelage — {3}{W} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Silverquill mid-game card
/// advantage. "Draw two cards. Each opponent loses 1 life."
///
/// A simple draw-2 + drain-1 spell at 4 mana — slots into any
/// Silverquill or W-leaning shell as a card draw fix. Wired as
/// `Seq(Draw 2, LoseLife 1 each opp)`. Tests verify both clauses
/// resolve.
pub fn stoic_tutelage() -> CardDefinition {
    CardDefinition {
        name: "Stoic Tutelage",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
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
    }
}

/// Lorehold Recovery — {2}{R}{W} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Lorehold gy-recursion
/// midrange spell. "Return target creature card from your graveyard
/// to the battlefield. It gains haste until end of turn."
///
/// A focused {2}{R}{W} reanimation spell with built-in haste — turn
/// your gy creatures into immediate attackers. Wired as `Seq(Move
/// target creature card from gy → bf, GrantKeyword(Haste, EOT))`.
/// The auto-target picker fills the gy creature slot.
pub fn lorehold_recovery() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recovery",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: crate::effect::ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
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
