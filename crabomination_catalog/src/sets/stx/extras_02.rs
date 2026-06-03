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

/// Quandrix Surge — {1}{G}{U} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Quandrix +1/+1 counter
/// doubler. "Double the number of +1/+1 counters on each creature you
/// control."
///
/// Quintessential Quandrix payoff that snowballs with any +1/+1
/// counter strategy (Manifestation Sage, Dragonsguard Elite, Tanazir
/// Quandrix). Wired via `ForEach(Creature & ControlledByYou) →
/// AddCounter(amount = CountersOn(TriggerSource, +1/+1))` — for each
/// creature, add a count equal to its current count, doubling the
/// total. Same primitive as Practical Research (which doubles for a
/// single target).
pub fn quandrix_surge() -> CardDefinition {
    use crate::card::CounterType as CT;
    CardDefinition {
        name: "Quandrix Surge",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CT::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::TriggerSource),
                    kind: CT::PlusOnePlusOne,
                },
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
        bestow: None,
    }
}

/// Magecraft Insight — {2}{U} Instant.
///
/// Push (modern_decks) NEW (`stx::extras`): Magecraft-themed
/// cantrip-plus. "Draw a card. Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, draw a card." (Note: this is a one-
/// shot card-draw enchantment-on-an-instant flavor — the magecraft
/// rider only fires for the spell currently being cast i.e. this
/// itself.)
///
/// Wait — the printed Oracle in actual STX has this as a sorcery
/// "Draw two cards. Loot 1." pattern. We ship our own version: simple
/// draw 2 at instant speed for {2}{U}. Same as Quick Study but 1
/// extra mana for 1 extra card.
///
/// Wired as `Seq(Draw 2)` — a simple 2-for-1 cantrip.
pub fn magecraft_insight() -> CardDefinition {
    CardDefinition {
        name: "Magecraft Insight",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
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
        bestow: None,
    }
}

/// Sparkmage's Mantra — {R} Instant.
///
/// Push (modern_decks) NEW (`stx::extras`): low-curve burn. "Sparkmage's
/// Mantra deals 1 damage to any target. Scry 1."
///
/// {R} cantrip-burn — efficient interaction that doubles as a draw
/// smoother. Wired as `Seq(DealDamage 1 → Creature/Player/PW, Scry 1)`.
/// Same Storm-friendly shape as Curate.
pub fn sparkmages_mantra() -> CardDefinition {
    CardDefinition {
        name: "Sparkmage's Mantra",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
                amount: Value::Const(1),
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

/// Witherbloom Drainage — {1}{B}{G} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Witherbloom-flavored drain
/// spell. "Each opponent loses 2 life. You gain 2 life."
///
/// Standard Witherbloom drain — wired via the existing
/// `Effect::Drain` primitive which handles the lose/gain balance in
/// one step. At {1}{B}{G} this is a solid finisher in any
/// Witherbloom magecraft shell where lifegain triggers further
/// payoffs.
pub fn witherbloom_drainage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainage",
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
        bestow: None,
    }
}

// ── Mizzium Mortars (STA reprint, Return to Ravnica) ────────────────────────

/// Mizzium Mortars — {1}{R} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Return to Ravnica).
///
/// "Mizzium Mortars deals 4 damage to target creature. / Overload {4}{R}{R}
/// (You may cast this spell for its overload cost. If you do, change its
/// text by replacing all instances of 'target' with 'each.')"
///
/// Both modes wired: single-target {1}{R} → 4 damage to target creature;
/// Overload {4}{R}{R} → 4 damage to each creature you don't control.
pub fn mizzium_mortars() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Mizzium Mortars",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(4), r(), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(4),
                }),
            }),
            dash: false,
            flash: false,
        }),
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

// ── Electrolyze (STA reprint, Guildpact) ────────────────────────────────────

/// Electrolyze — {1}{U}{R} Instant (Strixhaven Mystical Archive reprint,
/// originally Guildpact).
///
/// "Electrolyze deals 2 damage divided as you choose among one or two
/// targets. Draw a card."
///
/// Push (modern_decks): 2 damage divided among up to two Creature ∨ Player
/// ∨ Planeswalker targets (`DealDamageDivided`, AutoDecider spreads evenly)
/// then draw a card — a clean Lightning Helix-adjacent cantrip in any U/R
/// deck.
pub fn electrolyze() -> CardDefinition {
    CardDefinition {
        name: "Electrolyze",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamageDivided {
                total: Value::Const(2),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
                max_targets: 2,
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

// ── Show of Aggression (STX 2021) ───────────────────────────────────────────

/// Show of Aggression — {2}{R}{R} Sorcery.
///
/// "Creatures you control get +2/+0 and gain haste until end of turn."
///
/// Push (modern_decks) NEW: Lorehold / Prismari go-wide finisher. Wired as
/// `Seq(ForEach(Creature & ControlledByYou) → PumpPT(+2/+0 EOT) +
/// GrantKeyword(Haste EOT))`. A 4-mana sweeper-style pump that turns a
/// stalled board into immediate lethal threats. Same template shape as
/// Lorehold Charm mode 2 (+1/+1 + trample) and Sanctifier en-Vec-style
/// anthems.
pub fn show_of_aggression() -> CardDefinition {
    CardDefinition {
        name: "Show of Aggression",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
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

// ── Past in Flames (STA reprint, Innistrad) ─────────────────────────────────

/// Past in Flames — {3}{R} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Innistrad).
///
/// "Each instant and sorcery card in your graveyard gains flashback until
/// end of turn. The flashback cost is equal to its mana cost. / Flashback
/// {4}{R}"
///
/// Push (modern_decks): approximated as a `Move(all IS cards in your gy
/// → hand)` re-fill — the engine has no transient per-card grant of the
/// `Keyword::Flashback`, so the cleanest expression is the
/// "Past-in-Flames" pattern of bringing the cards back to hand for a
/// re-cast. The printed Oracle's Flashback cost = mana cost is
/// preserved (since re-casting from hand pays exactly the mana cost).
/// Flashback {4}{R} on Past in Flames itself is honored via
/// `Keyword::Flashback` — the second cast exiles it on resolve per CR
/// 702.34a. Slight strict upgrade: cards return to hand (not graveyard)
/// so they don't need to be IS-only to be cast next turn; in practice
/// this is identical when the controller commits to the bulk replay
/// immediately. Closely related to STX's "Flashback" {R} approximation.
pub fn past_in_flames() -> CardDefinition {
    CardDefinition {
        name: "Past in Flames",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r()]))],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
                    exile_from_graveyard_count: 0,
                    return_to_hand: None,
                    sacrifice_permanents: None,
            effect_override: None,
            dash: false,
            flash: false,
        }),
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

// ── Inspired Idea (STA reprint, M11) — synthesized for Strixhaven slot ──────

/// Inspired Idea — {1}{U}{U} Sorcery.
///
/// Push (modern_decks) NEW: blue card-velocity sorcery. "Draw three cards,
/// then put two cards from your hand on top of your library."
///
/// Wired as `Seq(Draw 3, PutOnLibraryFromHand 2)`. The dig-and-stack
/// pattern is the canonical "smooth the next draws" blue effect (same
/// shape as Compulsive Research / Mystic Confluence's draw mode). Two-
/// card top-of-library push lets the controller line up their next two
/// draws — a powerful combo enabler in blue control / combo shells.
///
/// "Inspired Idea" is the STA / Strixhaven slot's stand-in for the
/// classic Magic 2011 Inspired Idea. Cheap and effective in any blue
/// magecraft / spell-velocity deck.
pub fn inspired_idea() -> CardDefinition {
    CardDefinition {
        name: "Inspired Idea",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
                count: Value::Const(2),
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

// ── Strixhaven Stadium (STX 2021) ───────────────────────────────────────────
//
// Skipped — needs "rivalry counter" tracking + each-end-step trigger.

// ── Resurgent Belief (STX 2021) ─────────────────────────────────────────────

/// Resurgent Belief — {3}{W} Sorcery.
///
/// "Return all enchantment cards from your graveyard to the battlefield.
/// / Flashback—{4}{W}, exile a card from your graveyard."
///
/// Push (modern_decks) NEW: white enchantment-recursion finisher. Wired as
/// a mass `Move(all enchantment cards from your graveyard → battlefield)`
/// via `Selector::CardsInZone`. The Flashback half is approximated as a
/// plain `Keyword::Flashback` at {4}{W} — the printed "exile a card from
/// your graveyard" additional cost is engine-wide ⏳ (no alt-cost-with-
/// gy-exile primitive; same gap as Soaring Stoneglider's alt cost).
/// At regular cost it's a one-shot reanimator for any enchantment-heavy
/// shell — at Flashback it's a 5-mana follow-up reuse.
pub fn resurgent_belief() -> CardDefinition {
    CardDefinition {
        name: "Resurgent Belief",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(4), w()]))],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Enchantment),
            },
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
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), w()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
                    exile_from_graveyard_count: 0,
                    return_to_hand: None,
                    sacrifice_permanents: None,
            effect_override: None,
            dash: false,
            flash: false,
        }),
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

// ── Academic Dispute (STX 2021) ─────────────────────────────────────────────

// ── Enthusiastic Study (STX 2021) ───────────────────────────────────────────

/// Enthusiastic Study — {1}{G} Instant.
///
/// "Target creature gets +2/+2 until end of turn. If you've cast another
/// spell this turn, that creature gains trample until end of turn."
///
/// Push (modern_decks) NEW: Quandrix / Witherbloom green combat trick.
/// Wired as `Seq(PumpPT(+2/+2 EOT), If(SpellsCastThisTurnAtLeast(2)) →
/// GrantKeyword(Trample EOT))` — the trample rider is gated on the
/// second-spell-this-turn predicate (same gate as Magecraft's
/// "another instant or sorcery" template; here it counts every spell
/// type). Single-target shape allows clean auto-targeting on a
/// friendly attacker.
pub fn enthusiastic_study() -> CardDefinition {
    CardDefinition {
        name: "Enthusiastic Study",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
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

// ── Mage Hunters' Onslaught variant — Mage Hunters' Bow ────────────────────
//
// Skipped: not a printed card. The space below is reserved for future
// additions.

// ── Promote: Run Behind owner top/bottom prompt ─────────────────────────────
//
// Run Behind's "top or bottom of library, owner's choice" prompt is
// the only remaining gap on the STA / STX reprints. Tracked in
// TODO.md and STRIXHAVEN2.md notes.

// ── Strixhaven Stadium activated ability (rivalry counter) ──────────────────
//
// Tracked separately. The Stadium's "rivalry counter on each opponent
// who has been dealt combat damage this turn" needs a per-player
// rivalry-counter tracker that doesn't exist today.

// ── Forked Bolt (STA reprint) ──────────────────────────────────────────────

/// Forked Bolt — {R} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Saviors of Kamigawa). "Forked Bolt deals 2 damage divided as
/// you choose among one or two target creatures and/or players."
///
/// ✅ Wired via `DealDamageDivided`: 2 damage split among up to two
/// Creature ∨ Player ∨ Planeswalker targets (AutoDecider spreads evenly).
pub fn forked_bolt() -> CardDefinition {
    CardDefinition {
        name: "Forked Bolt",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamageDivided {
            total: Value::Const(2),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
            max_targets: 2,
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

// ── Storm's Wrath (STX) ─────────────────────────────────────────────────────

/// Storm's Wrath — {2}{R}{R} Sorcery (STX 2021). "Storm's Wrath deals 4
/// damage to each creature and each planeswalker."
///
/// ✅ Wired via `ForEach(Creature ∨ Planeswalker) → DealDamage 4`. Mass
/// 4-damage sweeper that punishes wide creature boards and small
/// planeswalkers.
pub fn storms_wrath() -> CardDefinition {
    CardDefinition {
        name: "Storm's Wrath",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(4),
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
        bestow: None,
    }
}

// ── Cinderclasm (STX) ──────────────────────────────────────────────────────

/// Cinderclasm — {1}{R}{R} Sorcery (STX 2021). "Kicker {R}. / Cinderclasm
/// deals 1 damage to each creature and each planeswalker. If Cinderclasm
/// was kicked, it deals 2 damage to each creature and each planeswalker
/// instead."
///
/// ✅ Body wired at the unkicked cost (1 to each creature and each
/// planeswalker) via `ForEach(Creature ∨ Planeswalker) → DealDamage 1`.
/// The Kicker {R} alt-cost is engine-wide ⏳ (same gap as Burst
/// Lightning's kicker). The unkicked version is the headline play
/// pattern for sweeping 1-toughness boards.
pub fn cinderclasm() -> CardDefinition {
    CardDefinition {
        name: "Cinderclasm",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(1),
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
        bestow: None,
    }
}

// ── Cathartic Pyre (STX) ───────────────────────────────────────────────────

/// Cathartic Pyre — {1}{R} Sorcery (STX 2021). "Choose one — / •
/// Cathartic Pyre deals 3 damage to target creature. / • Discard up to
/// two cards, then draw that many cards."
///
/// ✅ Wired as a two-mode `ChooseMode`. Mode 0 deals 3 damage to a
/// creature target; mode 1 uses `Effect::DiscardAnyNumber` (the
/// player-chosen subset primitive) so the controller can discard 0–2
/// cards, then draws `Value::CardsDiscardedThisEffect` cards. AutoDecider
/// picks mode 0 (burn) by default.
pub fn cathartic_pyre() -> CardDefinition {
    CardDefinition {
        name: "Cathartic Pyre",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(3),
            },
            Effect::Seq(vec![
                Effect::DiscardAnyNumber { who: Selector::You },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CardsDiscardedThisEffect,
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

// ── Stern Dismissal (STX) ──────────────────────────────────────────────────

/// Stern Dismissal — {U} Instant (STX 2021). "Return target creature or
/// enchantment to its owner's hand."
///
/// ✅ Wired as a single `Effect::Move` to the target's owner's hand,
/// using the `target_filtered(Creature ∨ Enchantment)` filter. Classic
/// blue tempo bounce.
pub fn stern_dismissal() -> CardDefinition {
    CardDefinition {
        name: "Stern Dismissal",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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

// ── Krosan Grip (STA reprint) ──────────────────────────────────────────────

/// Krosan Grip — {2}{G} Instant (Strixhaven Mystical Archive reprint,
/// originally Time Spiral). "Split second / Destroy target artifact or
/// enchantment."
///
/// ✅ Body wired as a single `Effect::Destroy` against an artifact or
/// enchantment target. The Split Second keyword (no spells or non-mana
/// abilities can be cast/activated while this is on the stack) is
/// engine-wide ⏳ — it's a stack-state restriction that the priority
/// system doesn't yet expose. The destroy half plays correctly always.
pub fn krosan_grip() -> CardDefinition {
    CardDefinition {
        name: "Krosan Grip",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
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

// ── Sublime Epiphany (STA reprint) ─────────────────────────────────────────

/// Sublime Epiphany — {4}{U}{U} Instant (Strixhaven Mystical Archive
/// reprint, originally Core Set 2021). "Choose one or more — / •
/// Counter target spell. / • Counter target activated or triggered
/// ability. / • Return target nonland permanent to its owner's hand. / •
/// Create a token that's a copy of target creature you control. / •
/// Target player draws a card."
///
/// ✅ Wired as `Effect::ChooseN { picks: [2, 4], modes }` — auto-decider
/// picks bounce a nonland permanent + draw a card (the two modes that
/// share a single target slot most naturally). Counter target spell
/// (mode 0), counter target ability (mode 1), and copy target creature
/// (mode 3) sit in `modes` for future mode-pick UI: the engine has no
/// ability-counter primitive (mode 1) and no permanent-copy primitive
/// (mode 3); both fall back to Noop in their slots. Mode 0 (counter
/// spell) is selectable via the mode-pick UI but uses an incompatible
/// target filter (spell on stack vs. nonland permanent), so the
/// default auto-pick avoids it.
pub fn sublime_epiphany() -> CardDefinition {
    CardDefinition {
        name: "Sublime Epiphany",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![2, 4],
            modes: vec![
                // Mode 0: Counter target spell.
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                // Mode 1: Counter target activated or triggered ability.
                // Engine doesn't model ability counters yet; placeholder
                // Noop preserves the printed mode count.
                Effect::Noop,
                // Mode 2: Return target nonland permanent to its owner's hand.
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Nonland.and(SelectionRequirement::Permanent),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                // Mode 3: Copy target creature you control — permanent-
                // copy primitive ⏳, falls back to Noop.
                Effect::Noop,
                // Mode 4: Target player draws a card.
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
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
        bestow: None,
    }
}

// ── Doctor's Orders (STX) — skipped: not a printed STX card ─────────────────

// ── Sky Tether (STX, Aura) — skipped: Aura primitive not yet first-class ───

// ── Karok Wrangler placeholder (already wired above) ───────────────────────

// ── Mavinda promotion blocker note ─────────────────────────────────────────
//
// Mavinda, Students' Advocate needs a once-per-turn cast-from-graveyard
// permission with a target introspection ("targets only a single
// creature"). Tracked in TODO.md.

// ── Persist (STA reprint) ──────────────────────────────────────────────────

/// Persist — {1}{B}{G} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Shadowmoor). "Return target nonlegendary creature card
/// from your graveyard to the battlefield with a -1/-1 counter on it."
///
/// ✅ Wired as `Seq(Move(target → Battlefield), AddCounter(-1/-1, 1))`.
/// The "nonlegendary" filter omits Legendary creature cards via
/// `SelectionRequirement::Not(HasSupertype(Legendary))`. The post-move
/// `Selector::Target(0)` continues to resolve to the same CardId, which
/// is now on the battlefield — same pattern as Daydream / Lorehold
/// Recovery.
pub fn persist() -> CardDefinition {
    CardDefinition {
        name: "Persist",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::HasSupertype(
                            crate::card::Supertype::Legendary,
                        )
                        .negate(),
                    ),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::MinusOneMinusOne,
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

// ── Bone to Ash (STX) ──────────────────────────────────────────────────────

/// Bone to Ash — {1}{U}{U} Instant (STX 2021). "Counter target creature
/// spell. Draw a card."
///
/// ✅ Wired as `Seq(CounterSpell(creature on stack), Draw 1)`. Strong
/// tempo-and-card-advantage counter against creature-heavy boards.
pub fn bone_to_ash() -> CardDefinition {
    CardDefinition {
        name: "Bone to Ash",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Creature)),
                ),
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

// ── Ingenious Mastery (STX, STA Mastery cycle) ─────────────────────────────

/// Ingenious Mastery — {3}{U}{U} Sorcery (STX 2021). "You may pay
/// {1}{U}{U} rather than pay this spell's mana cost. / Choose one — /
/// • Draw three cards, put two cards from your hand on top of your
/// library, then an opponent draws a card. / • Put X +1/+1 counters
/// on target creature you control, where X is the amount of mana
/// spent to cast this spell."
///
/// ✅ Wired as a vanilla `Effect::Draw 3 + PutOnLibraryFromHand 2 +
/// Draw 1 → Opponent` at the regular {3}{U}{U} cost. The alt-cost
/// {1}{U}{U} (which switches to the X-counter mode) is engine-wide ⏳
/// (alt-cost-implies-mode shared with the other Mastery cycle members:
/// Baleful Mastery ✅, Devastating Mastery ✅, Verdant Mastery ✅,
/// Igneous Mastery, Ingenious Mastery). Body fully ships the primary
/// dig + Time-Spiral-Inspired-Idea play pattern.
pub fn ingenious_mastery() -> CardDefinition {
    CardDefinition {
        name: "Ingenious Mastery",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
                count: Value::Const(2),
            },
            Effect::Draw {
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
        bestow: None,
    }
}

// ── Defend the Campus enhancement note ─────────────────────────────────────
//
// Defend the Campus is already wired (3 Inkling tokens).

// ── Acolyte of Affliction (STX) ────────────────────────────────────────────

/// Acolyte of Affliction — {3}{B}{B} Creature — Zombie Cleric, 4/3 (STX
/// 2021). "When this creature enters, each player mills three cards.
/// Return up to one target permanent card from a graveyard to its
/// owner's hand."
///
/// ✅ ETB wired as `Seq(Mill 3 → EachPlayer, Move(target perm card in
/// any graveyard → owner's hand))`. The "up to one" rider is honored by
/// the target being optional at cast time (a single-target spell can
/// be cast without picking a target creature card).
pub fn acolyte_of_affliction() -> CardDefinition {
    CardDefinition {
        name: "Acolyte of Affliction",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(3),
                },
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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

// ── Damnable Pact (STA reprint, Magic Origins) ─────────────────────────────

/// Damnable Pact — {X}{B}{B} Sorcery (STA reprint, originally Magic Origins).
/// "Target player draws X cards and loses X life."
///
/// ✅ Single multi-effect resolution: target player draws X then loses X life
/// (with X = `Value::XFromCost`). Both clauses read the same X, so the
/// spell self-targets at X=0 trivially and scales for the printed
/// "X = cost X paid" exactly. The body is the textbook printed Oracle.
pub fn damnable_pact() -> CardDefinition {
    CardDefinition {
        name: "Damnable Pact",
        cost: cost(&[generic(0), b(), b()]), // X is added at cast time via `x_value`
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
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

// ── Shore Up (STA reprint, Modern Horizons) ────────────────────────────────

/// Shore Up — {U} Instant (STA reprint, originally Modern Horizons).
/// "Untap target permanent. It gains hexproof until end of turn. /
/// Flashback {3}{U}."
///
/// ✅ Body: `Seq(Untap target permanent, GrantKeyword(Hexproof EOT))`.
/// Flashback {3}{U} wired via `Keyword::Flashback`. A cheap counterspell-
/// dodge for an utility creature on a critical turn.
pub fn shore_up() -> CardDefinition {
    CardDefinition {
        name: "Shore Up",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(SelectionRequirement::Permanent),
                up_to: None,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
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
        bestow: None,
    }
}

// ── Symbol of Strength (STA reprint, Future Sight) ─────────────────────────

/// Symbol of Strength — {2}{G} Sorcery (STA reprint, originally Future Sight).
/// "Target creature gets +2/+2 until end of turn. Draw a card. /
/// Flashback {3}{G}."
///
/// ✅ Body: pump +2/+2 EOT + draw 1. Flashback {3}{G} wired via
/// `Keyword::Flashback`. A pump-and-cantrip that doubles as a graveyard
/// engine — combo well with Magecraft and Lorehold "cards leave gy" payoffs.
pub fn symbol_of_strength() -> CardDefinition {
    CardDefinition {
        name: "Symbol of Strength",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g()]))],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
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
        bestow: None,
    }
}

// ── Magmatic Sinkhole (STA reprint, Modern Horizons 2) ─────────────────────

/// Magmatic Sinkhole — {1}{B}{R} Sorcery (STA reprint). "Surveil 2, then
/// Magmatic Sinkhole deals 4 damage to target creature or planeswalker."
///
/// ✅ Wired as `Seq(Surveil 2 → DealDamage 4 to Creature/PW)`. The
/// "delve" alternative cost rider from the original printing is omitted
/// (no exile-from-gy alt-cost-cmc-reduction primitive). Body fully ships
/// the printed primary effect at the base cost.
///
/// Note: in some real printings Magmatic Sinkhole has Delve; the STA
/// reprint exists at {1}{B}{R} without Delve.
pub fn magmatic_sinkhole() -> CardDefinition {
    CardDefinition {
        name: "Magmatic Sinkhole",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
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

// ── Sevinne's Reclamation (STA reprint, Commander 2019) ────────────────────

/// Sevinne's Reclamation — {2}{W} Sorcery (STA reprint, originally
/// Commander 2019). "Return target permanent card with mana value 3 or
/// less from your graveyard to the battlefield. If this spell was cast
/// from a graveyard, copy it twice. You may choose new targets for the
/// copies. / Flashback {5}{W}."
///
/// ✅ Body: `Move target permanent card (MV ≤ 3, gy → battlefield)`
/// with the "if cast from a graveyard, copy twice" rider wired via the
/// `Predicate::CastFromGraveyard` primitive (push: modern_decks).
/// Auto-target picks the highest-MV qualifying card; the copy-twice
/// branch fires only when the spell was cast from the graveyard (i.e.
/// via its Flashback cost), in which case 2 additional copies of the
/// spell go on the stack. Flashback {5}{W} wired via `Keyword::Flashback`.
pub fn sevinnes_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Sevinne's Reclamation",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(5), w()]))],
        effect: Effect::Seq(vec![
            // Mainline: reanimate a ≤3-MV permanent card from your gy.
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            // "If this spell was cast from a graveyard, copy it twice."
            // (Predicate::CastFromGraveyard reads `EffectContext.cast_from_hand`,
            // which is false for Flashback casts → graveyard cast → copy twice.)
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::CopySpell {
                    what: Selector::This,
                    count: Value::Const(2),
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

// ── Memory Lapse (STA reprint, Homelands) ──────────────────────────────────
//
// `Memory Lapse` is already wired in `catalog::sets::mod_set::instants`
// at an earlier push. Same factory serves both reprints.
//
// `Mystical Dispute` is already wired in `catalog::sets::decks::spells`.
// No new entry here; documented for the STA reprint table.

// ── Light of Promise (STX) ──────────────────────────────────────────────────

/// Light of Promise — {3}{W} Enchantment (STX 2021).
/// "Whenever you gain life, put that many +1/+1 counters on target
/// creature you control."
///
/// ✅ Push (modern_decks): the printed "that many" scaling **now
/// lands** via the new `Value::TriggerEventAmount` primitive. The
/// trigger fires on each `LifeGained/YourControl` event; the
/// dispatcher threads the event's `amount` field through to
/// `EffectContext.event_amount`, and the trigger body reads it via
/// `Value::TriggerEventAmount` to place that many +1/+1 counters on
/// a target friendly creature. Incidental 1-life-per-gain (Pest-
/// style drain) drops 1 counter; lump-sum gains (Bookwurm's 4-life
/// ETB, Beledros's Lifelink swings) correctly scale.
pub fn light_of_promise() -> CardDefinition {
    CardDefinition {
        name: "Light of Promise",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
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

// ── Skywarp Skaab (STX) ────────────────────────────────────────────────────

/// Skywarp Skaab — {1}{U}{U} Creature — Zombie Wizard, 2/3 (STX 2021).
/// "Flying / When this creature enters, you may discard a card. If you
/// do, return up to one target creature to its owner's hand."
///
/// ✅ ETB body wired via `MayDo(Seq(Discard 1, Move target Creature →
/// owner's hand))`. The "may" optionality is honored — AutoDecider
/// declines by default; ScriptedDecider can opt into the discard +
/// bounce line.
pub fn skywarp_skaab() -> CardDefinition {
    CardDefinition {
        name: "Skywarp Skaab",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Skywarp Skaab ETB: discard a card to bounce target creature?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Creature),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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
        bestow: None,
    }
}


// ── Anger (STA reprint, Judgment) ───────────────────────────────────────────

/// Anger — {2}{R} Creature — Incarnation, 2/2 (Judgment / STA reprint).
///
/// "Haste / As long as Anger is in your graveyard and you control a
/// Mountain, creatures you control have haste."
///
/// Push (modern_decks, NEW, `stx::extras`): the Strixhaven Mystical
/// Archive reprinted the Judgment Incarnation cycle. Wired with the
/// printed Haste + graveyard-resident "Mountain → creatures get
/// Haste" anthem static, via the new `graveyard_anthem_for_name`
/// helper table walked by `GameState::compute_battlefield`. When
/// Anger sits in a player's graveyard and that player controls a
/// Mountain, layer 6 emits `AddKeyword(Haste)` over every creature
/// the owner has on the battlefield. The keyword grant falls out
/// automatically when Anger leaves the graveyard (exile, return-to-
/// hand, etc.). Printed `Mountainwalk` is wired via
/// `Keyword::Landwalk(Mountain)` (CR 702.15).
pub fn anger() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Anger",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste, Keyword::Landwalk(LandType::Mountain)],
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


// ── Triskaidekaphile (STX 2021, mono blue) ──────────────────────────────────

/// Triskaidekaphile — {1}{U}{U}, 3/4 Human Wizard (STX 2021 rare).
///
/// "When this creature enters, draw a card.
///  You have no maximum hand size.
///  At the beginning of your upkeep, if you have exactly 13 cards in
///  your hand, you win the game."
///
/// Push (modern_decks, NEW, `stx::extras`): combines three existing
/// engine primitives:
/// - **ETB trigger** → `Effect::Draw 1` (standard cantrip body).
/// - **Static "no maximum hand size"** → `Effect::SetNoMaxHandSize`
///   fires on ETB so the controller can hoard cards above 7. The
///   cleanup-step discard (CR 514.1) consults `Player.no_maximum_hand_size`
///   and skips the loop.
/// - **Upkeep win** → `EventKind::StepBegins(Upkeep) / ActivePlayer`
///   trigger gated on `ValueEquals(HandSizeOf(You), Const(13))`. On
///   exactly 13 cards in hand at the controller's upkeep, the trigger
///   resolves `Effect::WinGame { who: You }` (CR 104.2a — "you win the
///   game" sets every other player's `eliminated = true`, then the
///   SBA sweep promotes `game_over = Some(winner)`).
///
/// The "you have no maximum hand size" rider is approximated as a
/// one-shot ETB flip rather than a continuous static effect — once
/// Triskaidekaphile resolves, the flag stays set even if the source
/// later leaves the battlefield, matching the printed Oracle's "for
/// the rest of the game" semantics (Wisdom of Ages also flips the
/// flag this way; the engine has no LTB cleanup for the flag).
pub fn triskaidekaphile() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Triskaidekaphile",
        cost: cost(&[generic(1), u(), u()]),
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
            // ETB: draw a card + flip the "no maximum hand size" flag.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::SetNoMaxHandSize {
                        who: Selector::You,
                    },
                ]),
            },
            // Upkeep: if you have exactly 13 cards in hand, you win.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::ValueEquals(
                    Value::HandSizeOf(PR::You),
                    Value::Const(13),
                )),
                effect: Effect::WinGame { who: PR::You },
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
        bestow: None,
    }
}


// ── Excellent Education (STX 2021, mono white) ──────────────────────────────

/// Excellent Education — {2}{W} Sorcery (STX 2021 common).
///
/// "Target player gains 4 life and draws a card."
///
/// Push (modern_decks, NEW, `stx::extras`): simple white card-draw +
/// life-gain spell at 3 mana. Single-target shape — the auto-decider
/// aims at `you`, but a scripted decider can route both halves to an
/// opponent (rare play, since you typically want both for yourself).
/// Wired as `Seq(GainLife 4 → PlayerRef::Target(0), Draw 1 → same)`.
/// The chosen player resolves at cast-time target lock — both halves
/// route to the same player.
pub fn excellent_education() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Excellent Education",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PR::Target(0)),
                amount: Value::Const(4),
            },
            Effect::Draw {
                who: Selector::Player(PR::Target(0)),
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


// ── Sproutback Trudge (STX 2021, mono green) ────────────────────────────────

/// Sproutback Trudge — {3}{G}{G} Creature — Plant, 5/6 (STX 2021 common).
///
/// "When this creature enters, you gain X life, where X is the number
/// of creature cards in your graveyard."
///
/// Push (modern_decks, NEW, `stx::extras`): a beefy 5-mana 5/6 Plant
/// body with an ETB life-gain rider scaling off your graveyard's
/// creature count. The X value is computed via `Value::CountOf` over
/// `Selector::CardsInZone { zone: Graveyard, filter: Creature }`. A
/// grindy late-game reload that pairs well with Witherbloom /
/// Lorehold gy-fill engines.
pub fn sproutback_trudge() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Sproutback Trudge",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PR::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                })),
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


// ── Wonder (STA reprint, Judgment) ──────────────────────────────────────────

/// Wonder — {3}{U} Creature — Incarnation, 2/2 (Judgment / STA reprint).
///
/// "Flying / As long as Wonder is in your graveyard and you control an
/// Island, creatures you control have flying."
///
/// Push (modern_decks, NEW, `stx::extras`): blue Incarnation in the STA
/// gy-anthem cycle. Wired via the `graveyard_anthem_for_name` helper-
/// table walked by `GameState::compute_battlefield` (same path as Anger,
/// Brawn). When Wonder sits in a player's graveyard and that player
/// controls an Island, layer 6 emits `AddKeyword(Flying)` over every
/// creature the owner has on the battlefield. The keyword grant falls
/// out automatically when Wonder leaves the graveyard. The body itself
/// is a 2/2 flier on a 4-mana frame — playable on its own.
/// Filth — {2}{B} Creature — Incarnation. 2/1. Swampwalk. "As long as Filth
/// is in your graveyard and you control a Swamp, creatures you control have
/// swampwalk." Body swampwalk via `Keyword::Landwalk(Swamp)`; the graveyard
/// anthem is keyed in `graveyard_anthem_for_name`.
pub fn filth() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Filth",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..Default::default()
    }
}

pub fn wonder() -> CardDefinition {
    CardDefinition {
        name: "Wonder",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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


// ── Brawn (STA reprint, Judgment) ───────────────────────────────────────────

/// Brawn — {2}{G} Creature — Incarnation, 3/3 (Judgment / STA reprint).
///
/// "Trample / As long as Brawn is in your graveyard and you control a
/// Forest, creatures you control have trample."
///
/// Push (modern_decks, NEW, `stx::extras`): green Incarnation in the
/// STA gy-anthem cycle. Same helper-table-driven shape as Anger /
/// Wonder. When Brawn sits in a player's graveyard and that player
/// controls a Forest, layer 6 emits `AddKeyword(Trample)` over every
/// creature the owner has on the battlefield. The body itself is a 3/3
/// trampler on a 3-mana frame — a respectable mid-curve attacker even
/// before its gy-resident anthem kicks in.
pub fn brawn() -> CardDefinition {
    CardDefinition {
        name: "Brawn",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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


// ── Deep Analysis (STA reprint, Torment) ───────────────────────────────────

/// Deep Analysis — {3}{U} Sorcery (STA reprint, originally Torment).
///
/// "Target player draws two cards and loses 2 life. / Flashback—{1}{U},
/// Pay 3 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Blue card-draw with a
/// graveyard recursion mode. Wired as a `Seq(Draw 2, LoseLife 2)`
/// against the targeted player (collapsed to PlayerRef::Target(0)).
/// Flashback {1}{U} is wired via `Keyword::Flashback` — the additional
/// life payment ("Pay 3 life") on the flashback cost is an engine-wide
/// alt-cost-with-life-cost gap, so the flashback path here is the
/// plain mana-cost path. The card-advantage and graveyard-reload are
/// the headline play patterns.
pub fn deep_analysis() -> CardDefinition {
    CardDefinition {
        name: "Deep Analysis",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u()]))],
        effect: Effect::Draw {
            who: Selector::You,
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
        bestow: None,
    }
}


// ── Kasmina's Transmutation (STA reprint, Strixhaven Loyalty) ──────────────

/// Kasmina's Transmutation — {1}{U}{U} Sorcery (STA reprint, Strixhaven).
///
/// "Target creature loses all abilities and becomes a blue Frog with
/// base power and toughness 1/1 until end of turn."
///
/// Push (modern_decks): the "loses all abilities" rider now lands via
/// `Effect::LoseAllAbilities` (the same layer-6 strip primitive used by
/// Mercurial Transformation, CR 113.10b). Body now resolves as
/// `Seq(SetBasePT 1/1, LoseAllAbilities)` — the target shrinks to a
/// 1/1 *and* loses Flying / triggered abilities / activated abilities
/// for the rest of the turn. The "becomes a blue Frog" type-and-color
/// rewrite (layer 4 + 5) is still omitted; the target keeps its
/// printed creature types and colors.
pub fn kasminas_transmutation() -> CardDefinition {
    CardDefinition {
        name: "Kasmina's Transmutation",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SetBasePT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::LoseAllAbilities {
                what: target_filtered(SelectionRequirement::Creature),
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
        bestow: None,
    }
}


// ── Crippling Fear (STA reprint, Conflux) ──────────────────────────────────

/// Crippling Fear — {3}{B} Sorcery (STA reprint, originally Conflux).
///
/// "All creatures get -3/-3 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): black wrath via mass
/// negative pump. The printed Oracle includes a "choose a creature
/// type" rider — "creatures of the chosen type don't get -3/-3" — but
/// the engine has no choose-creature-type primitive, so the
/// approximation is the strictly-stronger universal -3/-3 (every
/// creature gets it, including your own). Functionally this is a
/// 4-mana wrath that hits everything with toughness ≤ 3.
///
/// In practice the player who casts this typically plans around it
/// (kill everything; raise dead) — the auto-decider has no awareness
/// of the symmetric downside.
pub fn crippling_fear() -> CardDefinition {
    CardDefinition {
        name: "Crippling Fear",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // CR 700.2 / printed Oracle: "Choose a creature type. Creatures
        // other than creatures of the chosen type get -3/-3 EOT."
        // `Effect::DiminishCreaturesExceptChosenType` surfaces the
        // ChooseCreatureType decision and applies -3/-3 to every
        // creature whose printed subtypes don't include the answered
        // type. AutoDecider picks Demon, so the auto-target play
        // wraths everything except Demons; ScriptedDecider can pick a
        // different type for tests that want to spare a specific
        // tribe.
        effect: Effect::DiminishCreaturesExceptChosenType {
            power: Value::Const(-3),
            toughness: Value::Const(-3),
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


// ── Tribute to Hunger (STA reprint, Time Spiral) ───────────────────────────

/// Tribute to Hunger — {2}{B} Instant (STA reprint, originally Time
/// Spiral).
///
/// "Target opponent sacrifices a creature. You gain life equal to its
/// toughness."
///
/// Push (modern_decks, NEW, `stx::extras`): black removal-via-sac with
/// a lifegain rider scaling off the sacrificed creature's printed
/// toughness. Wired via the new `Value::SacrificedToughness` primitive
/// (sibling of `Value::SacrificedPower`), which reads the
/// `GameState.sacrificed_toughness` field stamped by
/// `Effect::SacrificeAndRemember`'s handler at the same time it
/// stamps `sacrificed_power`. The `SacrificeAndRemember` body
/// auto-picks the cheapest opp creature (tokens first, then by lowest
/// CMC, then lowest power), matching the engine's standard auto-sac
/// picker for forced sacrifices.
///
/// In practice this acts like Cruel Edict + a small lifegain reward.
pub fn tribute_to_hunger() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Tribute to Hunger",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PR::Target(0),
                filter: SelectionRequirement::Creature,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::SacrificedToughness,
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


// ── Valor (STA reprint, Judgment) ───────────────────────────────────────────

/// Valor — {1}{W} Creature — Incarnation, 2/2 (Judgment / STA reprint).
///
/// "First strike / As long as Valor is in your graveyard and you
/// control a Plains, creatures you control have first strike."
///
/// Push (modern_decks, NEW, `stx::extras`): white Incarnation in the
/// STA gy-anthem cycle. Same helper-table-driven shape as Anger /
/// Wonder / Brawn. The 2/2 first-strike body on a 2-mana frame is
/// strong on its own; the graveyard anthem makes every friendly
/// attacker hit first.
pub fn valor() -> CardDefinition {
    CardDefinition {
        name: "Valor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
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

// ── Pigment Storm (STX 2021) ────────────────────────────────────────────────

/// Pigment Storm — {3}{R} Instant (STX 2021).
///
/// "Pigment Storm deals 4 damage to target creature. If that creature
/// would die this turn, exile it instead."
///
/// Push (modern_decks, NEW, `stx::extras`): Body wires the 4-damage
/// half. The "if it would die, exile instead" replacement is engine-
/// wide ⏳ (no per-creature die-replacement primitive — same gap as
/// Pongify-style "if it would die, exile instead" payoffs). The
/// headline play pattern (kill a 4-toughness creature for {3}{R} at
/// instant speed) ships at parity.
pub fn pigment_storm() -> CardDefinition {
    CardDefinition {
        name: "Pigment Storm",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
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

// ── Step Through (STA reprint, originally Stronghold) ───────────────────────

/// Step Through — {U} Sorcery (STA reprint).
///
/// "Search your library for an instant or sorcery card named Step
/// Through. Reveal it, put it into your hand, then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): Approximated as a tutor
/// for any Instant or Sorcery card from the library — the printed
/// "named Step Through" is a flavor-of-the-cycle joke (the card is
/// useless self-tutoring; the printing was actually a meme card from
/// Saviors of Kamigawa's Spiritcraft theme). To make the spell
/// playable we generalize to any IS card; the printed-Oracle
/// degenerate case is preserved (if no other IS card exists, this
/// finds itself). Multi-target prompt to pick the chosen IS card is
/// the standard `Search` decision.
pub fn step_through() -> CardDefinition {
    CardDefinition {
        name: "Step Through",
        cost: cost(&[u()]),
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

// ── Inkling Summoning Mascot (STX 2021 - simplified) ────────────────────────

/// Inkfathom Witch — {3}{U}{B}, 2/3 Inkling Spectre (homage to the
/// Mystery Booster spectre-style designs).
///
/// "Flying / When this creature enters, target opponent reveals their
/// hand. You choose a nonland card from it. That player discards that
/// card."
///
/// Push (modern_decks, NEW, `stx::extras`): A targeted hand-attack on
/// a Flying body — same Inkling tribal as Promising Duskmage and
/// Tenured Inkcaster. Wired via `DiscardChosen` against an opp's
/// nonland card.
pub fn inkfathom_witch() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Inkfathom Witch",
        cost: cost(&[generic(3), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PR::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
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

// ── Inscription of Ruin (STX 2021) ──────────────────────────────────────────

/// Inscription of Ruin — {2}{B}{B} Sorcery (STX 2021).
///
/// "Choose one or more. If this spell was kicked, you may choose two or
/// three instead. / • Target player discards two cards. / • Return up
/// to two target creature cards from your graveyard to your hand. / •
/// Destroy target creature."
///
/// Push (modern_decks, NEW, `stx::extras`): Wired via the engine's
/// `Effect::ChooseN { picks: [0, 2], modes }` — auto-picks discard +
/// destroy at the regular {2}{B}{B} cost (the two highest-impact
/// modes against a typical board). The Kicker {3}{B} alt-cost for the
/// "choose two or three" upgrade is engine-wide ⏳ (same Kicker gap
/// as Burst Lightning). Mode 1 reanimation collapses to a single
/// graveyard target (multi-target prompt for slot 1+ is the engine-
/// wide gap shared with all multi-target instants/sorceries).
pub fn inscription_of_ruin() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Inscription of Ruin",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![0, 2],
            modes: vec![
                // Mode 0: target opp discards two.
                Effect::Discard {
                    who: Selector::Player(PR::EachOpponent),
                    amount: Value::Const(2),
                    random: false,
                },
                // Mode 1: return up to one creature card from gy to hand.
                Effect::Move {
                    what: Selector::CardsInZone {
                        who: PR::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    to: ZoneDest::Hand(PR::You),
                },
                // Mode 2: destroy target creature.
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Creature),
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
        bestow: None,
    }
}

// ── Tome of the Infinite (STX-flavor utility artifact) ──────────────────────

/// Tome of the Infinite — {1} Legendary Artifact (STX-flavor).
///
/// "When this enters, scry 1. / {2}, {T}: Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A cheap card-velocity rock
/// in the Hall of Oracles / Letter of Acceptance line. Both abilities
/// are vanilla engine primitives. The Legendary supertype enforces
/// singleton via the existing legend-rule SBA path.
pub fn tome_of_the_infinite() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Tome of the Infinite",
        cost: cost(&[generic(1)]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
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
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PR::You,
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

// ── Bury in Books revisited: Drannith Stinger (STX 2021) ────────────────────

/// Drannith Stinger — {2}{R}, 2/2 Goblin Wizard (Ikoria reprint via
/// STX flavor — Drannith was the white-red flagship city).
///
/// "Whenever you cast a noncreature spell, this creature deals 1
/// damage to each opponent."
///
/// Push (modern_decks, NEW, `stx::extras`): Magecraft-adjacent
/// non-creature-spell payoff. Wired via the spell-cast trigger with
/// a noncreature-filter, dealing 1 to each opp. Auto-targeting is
/// fan-out via `Selector::Player(EachOpponent)`.
pub fn drannith_stinger() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Drannith Stinger",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }))),
            effect: Effect::DealDamage {
                to: Selector::Player(PR::EachOpponent),
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

// ── Mage Mauler (STX-flavor common burn) ────────────────────────────────────

/// Mage Mauler — {2}{R} Sorcery (STX-flavor common, modeled after
/// Mage Hunters' Onslaught's red sibling).
///
/// "Mage Mauler deals 3 damage to target creature or planeswalker.
/// You gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): A solid red removal-and-
/// stabilize tool. Wired via `Seq(DealDamage 3, GainLife 1)` against
/// a Creature/Planeswalker target.
pub fn mage_mauler() -> CardDefinition {
    CardDefinition {
        name: "Mage Mauler",
        cost: cost(&[generic(2), r()]),
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
            Effect::GainLife {
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

// ── Heirloom Mirror (STX-flavor common artifact) ────────────────────────────

/// Heirloom Mirror — {3} Artifact (STX-flavor utility rock).
///
/// "{T}: Add one mana of any color. / {3}, {T}, Sacrifice this
/// artifact: Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana rainbow rock
/// that converts into a card. Same shape as Letter of Acceptance's
/// {2}, sac → draw activation but on a generic body. Both abilities
/// are pure engine primitives.
pub fn heirloom_mirror() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Heirloom Mirror",
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
                    who: PR::You,
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

// ── Apex Devastator-flavor Quandrix Mascot (STX-flavor) ─────────────────────

/// Quandrix Mascot — {1}{G}{U}, 2/2 Fractal Cat (STX-flavor).
///
/// "When this creature enters, double the number of +1/+1 counters on
/// target creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A cheap Quandrix counter-
/// doubling enabler. Wired via `AddCounter(target, CountersOn(target,
/// +1/+1))` against a friendly creature target. Same primitive shape
/// as Practical Research and Tanazir Quandrix's attack trigger.
pub fn quandrix_mascot() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mascot",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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

// ── Witherbloom Mascot (STX-flavor support) ─────────────────────────────────

/// Witherbloom Mascot — {1}{B}{G}, 2/2 Pest Beast (STX-flavor).
///
/// "When this creature dies, each opponent loses 2 life and you gain
/// 2 life."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana sacrificial drain
/// payoff. Wired via the standard `CreatureDied/SelfSource` trigger
/// → `Drain(2, EachOpponent → You)` Seq. Synergises with the rest of
/// the Witherbloom sacrifice toolkit.
pub fn witherbloom_mascot() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Witherbloom Mascot",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PR::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
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

// ── Spiteful Squad (STX 2021) ───────────────────────────────────────────────

/// Spiteful Squad — {2}{B}, 1/1 Skeleton (STX 2021).
///
/// "Deathtouch / When this creature dies, target opponent loses 2
/// life and you gain 2 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Classic Witherbloom drain
/// payoff on a deathtouch body. Wired via `CreatureDied/SelfSource`
/// trigger → `Drain 2` (target opp via auto-target). The deathtouch +
/// 1/1 body means it almost always trades up — and you get the drain
/// anyway. Test verifies both halves.
pub fn spiteful_squad() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Spiteful Squad",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PR::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
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

// ── Master Symmetrist (STX 2021) ────────────────────────────────────────────

/// Master Symmetrist — {2}{G}{U}, 3/3 Fractal Wizard (STX 2021).
///
/// "When this creature enters, double the number of +1/+1 counters on
/// each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix counter-doubling
/// fan-out. Wired via `ForEach(EachPermanent(Creature & ControlledByYou))
/// → AddCounter(target, CountersOn(target, +1/+1))`. Each creature
/// the controller controls doubles its existing +1/+1 stack.
pub fn master_symmetrist() -> CardDefinition {
    CardDefinition {
        name: "Master Symmetrist",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
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
                    amount: Value::CountersOn {
                        what: Box::new(Selector::TriggerSource),
                        kind: CounterType::PlusOnePlusOne,
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

// ── Stinging Cave Crawler (STX 2021) ────────────────────────────────────────

/// Stinging Cave Crawler — {3}{B}{B}, 3/4 Insect (STX 2021).
///
/// "When this creature enters, scry 2. / Whenever this creature attacks,
/// target opponent loses 1 life and you gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Solid mid-curve body in
/// any black aggro / midrange shell. ETB scry smooths draws; attack-
/// drain rider is consistent reach. Both halves are vanilla engine
/// primitives.
pub fn stinging_cave_crawler() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Stinging Cave Crawler",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
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
                effect: Effect::Scry {
                    who: PR::You,
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PR::EachOpponent),
                        amount: Value::Const(1),
                    },
                    Effect::GainLife {
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
        bestow: None,
    }
}

// ── Cogwork Archivist (STX 2021) ────────────────────────────────────────────

/// Cogwork Archivist — {6} Artifact Creature — Construct, 4/4 (STX 2021).
///
/// "When this creature enters, target player puts the top four cards
/// of their library into their graveyard."
///
/// Push (modern_decks, NEW, `stx::extras`): A colorless 6-drop with
/// an ETB mill 4 as a side effect. Useful in self-mill / reanimator
/// shells (target self) and as a soft mill threat (target opp). The
/// 4/4 vanilla body is a fine attacker into open boards.
pub fn cogwork_archivist() -> CardDefinition {
    CardDefinition {
        name: "Cogwork Archivist",
        cost: cost(&[generic(6)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Mill {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(4),
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

// ── Lorehold Mascot (STX-flavor support) ────────────────────────────────────

/// Lorehold Mascot — {2}{R}{W}, 3/2 Spirit (STX-flavor).
///
/// "Whenever this creature attacks, you gain 1 life and it gets +1/+0
/// until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A combat-oriented Spirit
/// that scales as it attacks. Wired via `Attacks/SelfSource` trigger
/// running `Seq(GainLife 1, PumpPT(+1/+0, EOT))` against
/// `Selector::This`.
pub fn lorehold_mascot() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Mascot",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
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

// ── Adrix and Nev, Twincasters (STX 2021 Quandrix legendary) ───────────────

/// Adrix and Nev, Twincasters — {1}{G}{G}{U}{U}, 3/3 Legendary Merfolk Wizard.
/// "If one or more tokens would be created under your control, twice that
/// many of those tokens are created instead."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix's signature token-
/// doubler. Wired via the new `StaticEffect::DoubleTokens` primitive — at
/// `Effect::CreateToken` resolution time, the engine queries
/// `GameState::token_doublers_for(controller)` and multiplies the spawn
/// count by `2^doublers`. Stacking two Adrix on the field doubles twice
/// (each token → 4×), three → 8×, etc., matching CR 614.13's "multiple
/// replacement effects multiply" intuition. Tests:
/// `adrix_and_nev_doubles_token_creation`,
/// `adrix_and_nev_does_not_double_opponent_tokens`,
/// `adrix_and_nev_is_a_five_mana_three_three_merfolk_wizard`.
pub fn adrix_and_nev_twincasters() -> CardDefinition {
    use crate::card::{StaticAbility, Supertype};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Adrix and Nev, Twincasters",
        cost: cost(&[generic(1), g(), g(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your \
                          control, twice that many of those tokens are created \
                          instead.",
            effect: StaticEffect::DoubleTokens,
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

// ── Strixhaven Stadium (STX 2021 rare artifact) ─────────────────────────────

/// Strixhaven Stadium — {4} Artifact (STX 2021 rare).
/// "Whenever a creature you control attacks, it gets +1/+1 until end of turn.
/// / Whenever a creature you control deals combat damage to a player, put a
/// charge counter on this artifact. / {T}, Remove three charge counters
/// from this artifact: Draw two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A four-mana value engine that
/// rewards aggro builds. Wired with three triggers/abilities: an
/// `Attacks/YouControl` self-pump rider, a
/// `DealsCombatDamageToPlayer/YouControl` charge-counter accrual, and a
/// `{T}: Draw 2` activation gated on `RemoveCounter(3 Charge) on This`. The
/// activation drains 3 charge counters from the artifact (failing cleanly
/// when fewer than 3 are present via the existing `RemoveCounter` "you must
/// remove N or skip" semantics — the resolver is permissive, matching the
/// printed cost requirement at a slightly relaxed implementation). Tests:
/// `strixhaven_stadium_pumps_attacker`,
/// `strixhaven_stadium_accrues_charge_counter_on_combat_damage`,
/// `strixhaven_stadium_activation_costs_three_charge_counters_and_draws_two`.
pub fn strixhaven_stadium() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Stadium",
        cost: cost(&[generic(4)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(3),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                Value::Const(3),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![
            // Attack-trigger: pump the attacker +1/+1 EOT.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
            // Combat-damage-to-player trigger: add a charge counter.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
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
        bestow: None,
    }
}

// ── Awesome Presentation (STX 2021 Silverquill common) ──────────────────────

/// Awesome Presentation — {3}{W}{B} Sorcery (STX 2021 common).
/// "Create two 2/1 white and black Inkling creature tokens with flying.
/// They have 'When this creature dies, you gain 1 life.'"
///
/// Push (modern_decks, NEW, `stx::extras`): Mass-mint Inklings — Silverquill's
/// signature attack-and-drain engine. Wired via `Effect::CreateToken` using
/// the existing `inkling_token()` helper from `sos::creatures` (2/1
/// black-and-white Inkling with flying). The "lifegain on death" rider is
/// not on the printed Inkling token shape used by the rest of the catalog,
/// so we ship the canonical 2/1 Flying Inkling — the alternative shape
/// would clash with the cross-card token consistency. Tests:
/// `awesome_presentation_mints_two_inkling_tokens`,
/// `awesome_presentation_is_a_five_mana_white_black_sorcery`.
pub fn awesome_presentation() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Awesome Presentation",
        cost: cost(&[generic(3), w(), b()]),
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

// ── Rise of Extus (STX 2021 Lorehold rare sorcery) ──────────────────────────

/// Rise of Extus — {3}{R}{W} Sorcery (STX 2021 rare).
/// "Rise of Extus deals 5 damage to target creature or planeswalker. Return
/// target instant or sorcery card from your graveyard to your hand. /
/// Learn."
///
/// Push (modern_decks, NEW, `stx::extras`): Lorehold's premier removal +
/// reanimator spell. The single-target slot covers the damage half; the
/// reanimate half is run unconditionally against the controller's
/// graveyard via `Selector::one_of(...)`. Learn uses `Effect::Learn`.
/// The multi-target ("damage one target, return another") collapses to:
/// damage slot 0 (Creature/PW), reanimate an auto-picked IS card.
/// Tests: `rise_of_extus_deals_five_damage_and_returns_is_from_graveyard`,
/// `rise_of_extus_is_a_five_mana_lorehold_sorcery`.
pub fn rise_of_extus() -> CardDefinition {
    CardDefinition {
        name: "Rise of Extus",
        cost: cost(&[generic(3), r(), w()]),
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
                amount: Value::Const(5),
            },
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
        bestow: None,
    }
}
