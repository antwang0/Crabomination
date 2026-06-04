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

/// Bookwurm — {5}{G}{G}, 5/5 Wurm. "Trample / When this creature enters,
/// you gain 4 life and draw a card."
///
/// ✅ ETB body is a simple `Seq(GainLife(4), Draw(1))`. The 5/5 trample
/// body is a fine top-end finisher in any green deck.
pub fn bookwurm() -> CardDefinition {
    CardDefinition {
        name: "Bookwurm",
        cost: cost(&[generic(5), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
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
        foretell_cost: None,
    }
}

// ── Field Trip ──────────────────────────────────────────────────────────────

/// Field Trip — {2}{G} Sorcery. "Search your library for a basic Forest
/// card, put it onto the battlefield, then shuffle. Learn."
///
/// ✅ Faithful single-search wire via `Effect::Search` for a basic land
/// with the Forest land subtype, plus Learn via `Effect::Learn`.
pub fn field_trip() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Field Trip",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand
                    .and(SelectionRequirement::HasLandType(LandType::Forest)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
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
        foretell_cost: None,
    }
}

// ── Reduce to Memory ────────────────────────────────────────────────────────

/// Reduce to Memory — {2}{U} Sorcery. "Exile target nonland permanent.
/// Its controller creates a 2/2 colorless Inkling artifact creature
/// token."
///
/// ✅ Wired faithfully: `Exile` the targeted nonland permanent, then
/// mint a 2/2 Inkling artifact creature token. The token is given to
/// the *original controller* of the exiled permanent via
/// `PlayerRef::ControllerOfTarget(0)` (mirror of the printed
/// "its controller").
pub fn reduce_to_memory() -> CardDefinition {
    let inkling = TokenDefinition {
        name: "Inkling".into(),
        power: 2,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        // Colorless artifact creature.
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Reduce to Memory",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: inkling,
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
        foretell_cost: None,
    }
}

// ── Baleful Mastery ─────────────────────────────────────────────────────────

/// Baleful Mastery — {3}{B} Instant. "You may pay {1}{B} rather than pay
/// this spell's mana cost. If the {1}{B} cost was paid, an opponent draws a
/// card. / Exile target creature or planeswalker."
///
/// ✅ Full wiring: base cost {3}{B} exiles target creature or planeswalker
/// cleanly. Alt cost {1}{B} via `AlternativeCost` with `effect_override`
/// that sequences opponent-draws-1 before the exile — only the alt-cast
/// path triggers the draw penalty.
pub fn baleful_mastery() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Baleful Mastery",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), b()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 0,
            return_to_hand: None,
            sacrifice_permanents: None,
            effect_override: Some(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
                // Same target-filtered slot 0 as the base effect, so the
                // alt-cast path surfaces the "target creature or planeswalker"
                // requirement (the client prompts for it) instead of leaving
                // slot 0 unfiltered — which made the alt cast resolve the
                // opponent's draw without actually exiling anything.
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Planeswalker),
                    ),
                },
            ])),
            dash: false,
            blitz: false,
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
        foretell_cost: None,
    }
}

// ── Igneous Inspiration ─────────────────────────────────────────────────────

/// Igneous Inspiration — {2}{R} Sorcery. "Igneous Inspiration deals 3
/// damage to target creature or planeswalker. Learn."
///
/// ✅ Wired faithfully: 3 damage to a creature/planeswalker target,
/// then Learn via `Effect::Learn`.
pub fn igneous_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Igneous Inspiration",
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
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
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
        foretell_cost: None,
    }
}

// ── Combat Professor ────────────────────────────────────────────────────────

/// Combat Professor — {3}{W} Creature — Cat Cleric, 2/4, Flying,
/// Vigilance. "Mentor (Whenever this creature attacks, put a +1/+1
/// counter on target attacking creature with lesser power.)"
///
/// Combat Professor — {3}{W}, 2/4 Cat Cleric, Flying / Vigilance. Mentor
/// (attack trigger puts a +1/+1 counter on a target attacking creature
/// with lesser power) wired via `SelectionRequirement::PowerLessThanSource`,
/// so the "lesser power" check tracks Combat Professor's current power.
pub fn combat_professor() -> CardDefinition {
    CardDefinition {
        name: "Combat Professor",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            // Mentor (CR 702.114): counter goes on a target attacking
            // creature with lesser power than this — `PowerLessThanSource`
            // re-evaluates against Combat Professor's current power.
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::IsAttacking)
                        .and(SelectionRequirement::PowerLessThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Conspiracy Theorist ─────────────────────────────────────────────────────

// ── Beaming Defiance ────────────────────────────────────────────────────────

/// Beaming Defiance — {1}{W} Instant. "Target creature you control gets
/// +2/+0 and gains indestructible until end of turn."
///
/// ✅ Wired as `PumpPT(+2/+0)` + `GrantKeyword(Indestructible, EOT)`.
/// A combat-trick pump-and-protect. (Printed Oracle: "Hexproof" until
/// end of turn — but Strixhaven's printed Beaming Defiance is actually
/// "+2/+0 and gains hexproof until end of turn". We use Hexproof to
/// match Oracle.)
pub fn beaming_defiance() -> CardDefinition {
    CardDefinition {
        name: "Beaming Defiance",
        cost: cost(&[generic(1), w()]),
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
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
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
        foretell_cost: None,
    }
}

// ── Spell Satchel ───────────────────────────────────────────────────────────

/// Spell Satchel — {3} Artifact. "{T}: Add {C}. / {3}, {T}, Sacrifice
/// this artifact: Choose any number of target instant and/or sorcery
/// cards in your graveyard with total mana value 4 or less. Return them
/// to your hand."
///
/// ✅ (was 🟡): "any number with total MV ≤ 4" is now wired via the
/// new `Selector::TakeWithSumCap { inner, cap, value_of_each }`
/// primitive — walks the caster's gy in iteration order, accumulating
/// each card's mana value, and takes them greedily while the running
/// sum stays ≤ 4. Skips cards that would push the sum over (so a 4-MV
/// Cancel + 1-MV Bolt picks Bolt-and-Cancel = 5 → skip Cancel, take
/// Bolt = 1, then if a 3-MV Lightning Helix is next take it = 4 total).
/// The auto-decider walks gy-iteration order; a real UI player would
/// pick. The `{T}: Add {C}` mana ability and sac-as-cost are wired.
pub fn spell_satchel() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Spell Satchel",
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
                mana_cost: cost(&[]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
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
                effect: Effect::Move {
                    what: Selector::TakeWithSumCap {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: SelectionRequirement::HasCardType(CardType::Instant)
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        }),
                        cap: Box::new(Value::Const(4)),
                        value_of_each: Box::new(Value::ManaValueOf(Box::new(
                            Selector::TriggerSource,
                        ))),
                    },
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
        foretell_cost: None,
    }
}

// ── Squirrel Sanctuary (stand-in placeholder dropped) ───────────────────────

// ── Excavated Wall ──────────────────────────────────────────────────────────

/// Excavated Wall — {2} Artifact Creature — Wall, 0/4, Defender. "When
/// this creature enters, you gain 2 life."
///
/// ✅ Simple ETB lifegain on a defender wall body. Same shape as
/// Wall of Omens but the value is straight lifegain instead of a card.
pub fn excavated_wall() -> CardDefinition {
    CardDefinition {
        name: "Excavated Wall",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
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
        foretell_cost: None,
    }
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — {U}{R} Instant. "Tap up to two target creatures. Put a
/// stun counter on each of them."
///
/// ✅ Push (modern_decks): wired faithfully as a two-slot spell. Slot 0
/// is the first creature, slot 1 (passed via
/// `GameAction::CastSpell.additional_targets[0]`) is the second.
/// "Up to two" semantics fall out naturally — if the cast supplies
/// only one target, `Selector::Target(1)` and
/// `Selector::TargetFiltered { slot: 1, … }` resolve to nothing and
/// the second tap+stun pair is a no-op. The cast-side AutoDecider
/// currently doesn't auto-pick slot-1 targets; tests pass them
/// explicitly via `additional_targets: vec![Target::Permanent(c)]`.
pub fn snow_day() -> CardDefinition {
    CardDefinition {
        name: "Snow Day",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Slot 0: tap + stun the first creature.
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
            // Slot 1: tap + stun the second creature (optional — resolves to
            // no-op when only one target was chosen).
            Effect::Tap {
                what: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
            },
            Effect::AddCounter {
                what: Selector::Target(1),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── (helper `local_pest_token` removed in push XX — `super::shared::stx_pest_token`
//     is the canonical Pest factory used everywhere a Pest is minted.)

// ── Curate ──────────────────────────────────────────────────────────────────

/// Curate — {1}{U} Instant. "Look at the top four cards of your library.
/// Put one of them into your hand and the rest on the bottom of your
/// library in a random order."
///
/// Ships via `Effect::LookPickToHand` (look at top 4, one to hand, rest to
/// the bottom). The "random order" on the bottom is cosmetic (those cards
/// aren't seen again until reshuffled).
pub fn curate() -> CardDefinition {
    CardDefinition {
        name: "Curate",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
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
        foretell_cost: None,
    }
}

// ── Strategic Planning (already defined in `decks::modern`) ────────────────
//
// Strategic Planning is wired in `catalog::sets::decks::modern::strategic_planning`
// — a Mill 3 + Draw 1 approximation that pairs well with reanimator
// shells. STX shares the same printed text, so the STX module re-uses
// the existing function rather than redefining it. (Adding a duplicate
// here would shadow the existing glob re-export from `catalog::*`.)

// ── Solve the Equation ─────────────────────────────────────────────────────

/// Solve the Equation — {2}{U} Sorcery. "Search your library for an
/// instant or sorcery card, reveal it, put it into your hand, then
/// shuffle."
///
/// Straight tutor for instant/sorcery via `Effect::Search` against
/// `IsSpell`-style filters (HasCardType(Instant) ∨ HasCardType(Sorcery))
/// → `ZoneDest::Hand(You)`. A simple Mystical Tutor cousin.
pub fn solve_the_equation() -> CardDefinition {
    CardDefinition {
        name: "Solve the Equation",
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
        foretell_cost: None,
    }
}

// ── Resculpt ───────────────────────────────────────────────────────────────

/// Resculpt — {1}{U} Instant. "Exile target creature or artifact. Its
/// controller creates a 4/4 blue Elemental creature token."
///
/// ✅ Wired faithfully: `Exile` the target, then mint a 4/4 blue
/// Elemental token under the *original controller* of the exiled
/// permanent (`PlayerRef::ControllerOf(Target(0))`). A clean unconditional
/// removal-with-trade — the controller gets a card-quality token in
/// exchange for losing whatever permanent was targeted.
pub fn resculpt() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Resculpt",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: elemental,
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
        foretell_cost: None,
    }
}

// ── Mortality Spear ────────────────────────────────────────────────────────

/// Mortality Spear — {3}{B}{G} Instant. "Destroy target creature,
/// planeswalker, or battle."
///
/// ✅ Catch-all removal: `Destroy` against a Creature ∨ Planeswalker
/// target. Battle subtype isn't yet modelled (no MoM/March of the
/// Machine in this catalog), so the printed third clause is dropped —
/// it's a no-op in the current card pool anyway.
pub fn mortality_spear() -> CardDefinition {
    CardDefinition {
        name: "Mortality Spear",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
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
        foretell_cost: None,
    }
}

// ── Daemogoth Titan ────────────────────────────────────────────────────────

/// Daemogoth Titan — {B}{B}, 11/11 Demon Horror. "When this attacks or
/// blocks, sacrifice another creature."
///
/// ✅ Both halves now wired. The attack half uses
/// `EventKind::Attacks/SelfSource`; the block half uses the new
/// `EventKind::Blocks/SelfSource` (push XXVI added the `Blocks` event
/// and the dispatcher wiring per CR 509.1i). The sacrifice resolves
/// via `Effect::Sacrifice` over creatures you control — the
/// auto-decider prefers lowest-power non-source creatures, so a fresh
/// Titan will sac something else rather than itself.
pub fn daemogoth_titan() -> CardDefinition {
    let sac_another = Effect::Sacrifice {
        who: Selector::You,
        count: Value::Const(1),
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Daemogoth Titan",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon, CreatureType::Horror],
            ..Default::default()
        },
        power: 11,
        toughness: 11,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: sac_another.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: sac_another,
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
        foretell_cost: None,
    }
}

// ── Daemogoth Woe-Eater ────────────────────────────────────────────────────

/// Daemogoth Woe-Eater — {2}{B}{G}, 4/4 Demon Horror. "When this enters,
/// sacrifice another creature. Whenever this attacks, you may sacrifice
/// another creature. If you do, put a +1/+1 counter on this creature."
///
/// ETB sacrifice is mandatory; attack sac is optional via `MayDo`. The
/// +1/+1 counter is gated on the controller's "yes" answer, not on
/// legality — `Sacrifice` no-ops cleanly when no candidate exists.
pub fn daemogoth_woe_eater() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Daemogoth Woe-Eater",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon, CreatureType::Horror],
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
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Daemogoth Woe-Eater attack: sacrifice another \
                                  creature to put a +1/+1 counter on it?"
                        .into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::Const(1),
                            filter: SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        },
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(1),
                        },
                    ])),
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
        foretell_cost: None,
    }
}

// ── Honor Troll ────────────────────────────────────────────────────────────

/// Honor Troll — {1}{B}{G}, 1/4 Troll Warrior. "Trample. As long as
/// you've gained life this turn, this creature has +2/+0 and lifelink."
///
/// Compute-time injection in `GameState::compute_battlefield` (same
/// pattern as Cruel Somnophage / Tarmogoyf): when controller has gained
/// ≥1 life this turn, layers 6 and 7b add Lifelink and +2/+0. The gate
/// re-evaluates every recompute and resets at untap.
pub fn honor_troll() -> CardDefinition {
    CardDefinition {
        name: "Honor Troll",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
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
        foretell_cost: None,
    }
}

// ── Quandrix Cultivator ────────────────────────────────────────────────────

/// Quandrix Cultivator — {3}{G}{U}, 3/3 Elf Druid. "When this creature
/// enters, search your library for a basic Forest or Island card, put
/// it onto the battlefield tapped, then shuffle."
///
/// ✅ Faithful ETB ramp wired via `Effect::Search` against
/// `IsBasicLand & (HasLandType(Forest) ∨ HasLandType(Island))`. Lands
/// enter tapped, matching the printed restriction.
pub fn quandrix_cultivator() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Quandrix Cultivator",
        cost: cost(&[generic(3), g(), u()]),
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
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand.and(
                    SelectionRequirement::HasLandType(LandType::Forest)
                        .or(SelectionRequirement::HasLandType(LandType::Island)),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
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
        foretell_cost: None,
    }
}

// ── Hofri Ghostforge ───────────────────────────────────────────────────────

/// Hofri Ghostforge — {2}{R}{W}, 3/4 Legendary Spirit Cleric. "Other
/// creatures you control get +1/+0. / Whenever another nontoken
/// creature you control dies, exile it. At the beginning of the next
/// end step, return it to the battlefield as a 1/1 Spirit with flying."
///
/// 🟡 Body + keywords (legendary, P/T, types) ship full. The "Other
/// creatures you control get +1/+0" anthem is **now wired** (push
/// XXXV) via the new `SelectionRequirement::OtherThanSource` primitive
/// flowing through `affected_from_requirement`, which flips the
/// resulting `AffectedPermanents::All.exclude_source` flag so the
/// anthem layer skips Hofri itself. Matches the printed "**other**
/// creatures" wording exactly.
///
/// The "exile-on-death + return at end step as a 1/1 Spirit" cycle
/// stays ⏳ pending a delayed-replacement-on-graveyard primitive
/// (tracked in TODO.md). Hofri retains its 🟡 status until that
/// closes; the anthem half is real-card-faithful.
pub fn hofri_ghostforge() -> CardDefinition {
    use crate::card::{
        EventKind, EventScope, EventSpec, Predicate, SelectionRequirement,
        StaticAbility, TriggeredAbility,
    };
    use crate::effect::{DelayedTriggerKind, Selector, StaticEffect, ZoneDest};
    CardDefinition {
        name: "Hofri Ghostforge",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Push (modern_decks, batch 80): "Whenever another nontoken
        // creature you control dies, exile that card. Return it to the
        // battlefield. Exile it at the beginning of the next end step."
        // Wired as `Move(TriggerSource, gy → bf untapped)` +
        // `DelayUntilNextEndStep { Move(TriggerSource, → Exile) }`. The
        // brief exile-then-return half is collapsed to just "return" —
        // the engine has no replacement primitive that routes a card
        // through exile mid-resolution, but the net play pattern (card
        // ends up on the battlefield, then exiles at next EOT) matches.
        // The printed "It's a Spirit in addition to its other types"
        // type-override (layer 4) is approximated as a no-op — the
        // returned card keeps its printed creature types only. Filter:
        // non-token via `Predicate::EntityMatches` against
        // `Not(IsToken)`.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Not(Box::new(SelectionRequirement::IsToken)),
                }),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: crate::effect::PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move {
                        what: Selector::TriggerSource,
                        to: ZoneDest::Exile,
                    }),
                },
            ]),
        }],
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
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
    }
}

// ── Tempted by the Oriq ────────────────────────────────────────────────────

/// Tempted by the Oriq — {2}{B} Sorcery. "Gain control of target
/// creature until end of turn. Untap that creature. It gains haste
/// until end of turn." (Threaten / Act of Treason template, printed
/// as a one-shot sorcery — there is no Magecraft rider on the
/// printed card; the prior note referencing a "Magecraft rider" was
/// a doc-only artifact from an earlier draft and has been cleared
/// here.)
///
/// Full printed Threaten template: `GainControl` (EOT) +
/// `Untap(Target)` + `GrantKeyword(Haste, EOT)`.
pub fn tempted_by_the_oriq() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Tempted by the Oriq",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
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
        bestow: None,
        foretell_cost: None,
    }
}


/// Confront the Past — {3}{R} Sorcery.
/// "Choose one — / • Put target planeswalker card from your graveyard
/// onto the battlefield. / • Return target planeswalker to its
/// owner's hand. / • Confront the Past deals damage to target
/// planeswalker equal to the number of loyalty counters on it."
///
/// ✅ Three-mode `ChooseMode`: mode 0 reanimates a PW from your
/// graveyard (auto-decider picks the only PW in gy), mode 1 bounces
/// an opp PW, mode 2 deals damage = the target PW's current loyalty
/// counters via the new `Value::LoyaltyOf(Target(0))` primitive (push
/// XXXIII). The damage value is computed at resolution time from the
/// `CounterType::Loyalty` counter pool on the targeted planeswalker;
/// since damage to a planeswalker comes off as loyalty loss (CR
/// 120.3c), the effect strictly removes all remaining loyalty —
/// matching the printed "lethal-to-the-PW" Oracle behavior. (For an
/// opponent's PW the practical effect is also lethal because loyalty
/// loss exactly equals current loyalty.)
pub fn confront_the_past() -> CardDefinition {
    CardDefinition {
        name: "Confront the Past",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Planeswalker),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Planeswalker),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Planeswalker),
                amount: Value::LoyaltyOf(Box::new(Selector::Target(0))),
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
        foretell_cost: None,
    }
}

/// Specter of the Fens — {4}{B} Creature — Specter. 3/4 Flying.
/// "When this creature enters, return target creature or planeswalker
/// card from your graveyard to your hand."
///
/// ✅ Reanimation-flavoured ETB on a sizeable flier. Standard
/// `Move(filter → Hand(You))` against a graveyard creature/PW.
pub fn specter_of_the_fens() -> CardDefinition {
    CardDefinition {
        name: "Specter of the Fens",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
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
        bestow: None,
        foretell_cost: None,
    }
}

/// Mascot Interception — {4}{R}{W} Instant.
/// "Gain control of target permanent until end of turn. Untap it.
/// It gains haste until end of turn."
///
/// ✅ Threaten-with-untap-and-haste at instant speed against any
/// permanent. Similar shape to Tempted by the Oriq (push XX) but
/// instant-speed and any-permanent rather than sorcery-speed creature-only.
pub fn mascot_interception() -> CardDefinition {
    CardDefinition {
        name: "Mascot Interception",
        cost: cost(&[generic(4), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Permanent),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
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
        bestow: None,
        foretell_cost: None,
    }
}

/// Twinscroll Shaman — {2}{U}{R} Creature — Human Wizard. 3/3.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, copy that spell. You may choose new targets for the copy."
///
/// ✅ The Magecraft trigger uses the existing `Effect::CopySpell`
/// primitive (push XVII), pointed at `Selector::TriggerSource` —
/// which `fire_spell_cast_triggers` binds to the cast spell's
/// CardId. The "may choose new targets" rider collapses to keep
/// (auto-decider default).
pub fn twinscroll_shaman() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Twinscroll Shaman",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::CopySpell {
            what: Selector::TriggerSource,
            count: Value::Const(1),
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
        bestow: None,
        foretell_cost: None,
    }
}

/// Practical Research — {1}{G}{U} Sorcery.
/// "Choose target creature you control. For each +1/+1 counter on
/// it, put another +1/+1 counter on it."
///
/// ✅ Doubles +1/+1 counters on the chosen creature via
/// `AddCounter(amount = CountersOn(target, +1/+1))`. Same shape as
/// Growth Curve's second half but as a one-shot without the
/// initial-counter bump.
pub fn practical_research() -> CardDefinition {
    CardDefinition {
        name: "Practical Research",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        foretell_cost: None,
    }
}

/// Hall of Oracles — Land.
/// "{T}: Add {C}. / {2}, {T}: Put a +1/+1 counter on target Wizard
/// or Fractal creature you control."
///
/// ✅ Quandrix-flavoured utility land. The `{T}: Add {C}` mana
/// ability uses the shared `tap_add_colorless` helper. The +1/+1
/// activation is wired with a tribal filter (Wizard ∪ Fractal &
/// ControlledByYou).
pub fn hall_of_oracles() -> CardDefinition {
    CardDefinition {
        name: "Hall of Oracles",
        cost: cost(&[]),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(
                            SelectionRequirement::HasCreatureType(CreatureType::Wizard)
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                        ),
                    ),
                    kind: CounterType::PlusOnePlusOne,
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
        bestow: None,
        foretell_cost: None,
    }
}

/// Star Pupil — {B} Creature — Cat Spirit, 0/1 (Silverquill).
/// "Star Pupil enters the battlefield with a +1/+1 counter on it. /
/// When Star Pupil dies, put a +1/+1 counter on target creature."
///
/// ✅ Both halves wired. The ETB-counter is modelled via an ETB
/// trigger (matches Pterafractyl). The death trigger drops exactly
/// one +1/+1 counter on target creature — matching the printed
/// Oracle, which says "a +1/+1 counter" (singular). Note that the
/// closely-related "its +1/+1 counters" template would *not* work at
/// printed speed per CR 122.8 — counters on the source are checked
/// after it has left the battlefield, and CR 122.8 explicitly says
/// no transfer happens in that case. Star Pupil dodges the rule by
/// hard-coding one counter; cards like Mantle of Tides that DO say
/// "its +1/+1 counters" have errata changing the language to "1"
/// instead. `Value::CountersOn` supports cross-zone search so future
/// cards that need source's counter count post-death can read it.
pub fn star_pupil() -> CardDefinition {
    CardDefinition {
        name: "Star Pupil",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: put a +1/+1 counter on self (approximating the
            // "enters with" replacement effect with a trigger; matches
            // the Pterafractyl pattern).
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            // Dies: put a +1/+1 counter on target creature. Exactly
            // one counter per the printed Oracle (CR 122.8-friendly).
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
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
        foretell_cost: None,
    }
}

/// Ageless Guardian — {2}{W} Creature — Spirit Cleric, 1/4 (Silverquill).
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// Ageless Guardian gets +1/+0 until end of turn."
///
/// ✅ Pure magecraft self-pump via `effect::shortcut::magecraft_self_pump(1, 0)`.
/// Same shape as Symmetry Sage's first half but without the flying-grant
/// rider. The 1/4 body soaks early aggression while spellslinging chip.
pub fn ageless_guardian() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Ageless Guardian",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
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
        bestow: None,
        foretell_cost: None,
    }
}

/// Letter of Acceptance — {1} Artifact (Colorless).
/// "When Letter of Acceptance enters, you gain 1 life. / {T}: Add {C}.
/// / {2}, {T}, Sacrifice this artifact: Draw a card."
///
/// ✅ A two-cost artifact mana-rock with an ETB lifegain rider and a
/// late-game sac-to-draw mode. All three abilities use existing
/// engine primitives (ETB trigger, mana ability via `tap_add_colorless`,
/// `sac_cost: true` on the draw activation).
pub fn letter_of_acceptance() -> CardDefinition {
    CardDefinition {
        name: "Letter of Acceptance",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
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
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
    }
}

/// Charge Through — {G} Sorcery (Mono-G STX).
/// "Target creature you control gets +1/+1 and gains trample until
/// end of turn."
///
/// ✅ A one-mana pump-and-trample combat trick. Wired as a `Seq` of
/// `PumpPT(+1/+1, EOT)` and `GrantKeyword(Trample, EOT)`. Both halves
/// reference the same `Target(0)` slot.
pub fn charge_through() -> CardDefinition {
    CardDefinition {
        name: "Charge Through",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
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
                toughness: Value::Const(1),
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
        bestow: None,
        foretell_cost: None,
    }
}

/// Devious Cover-Up — {2}{U}{U} Instant (Mono-U STX).
/// "Counter target spell. Then exile any number of target cards from
/// graveyards." The graveyard-strip rider ships via
/// `Effect::ExileAnyNumberFromGraveyards` (`Decision::ChooseCards`).
pub fn devious_cover_up() -> CardDefinition {
    CardDefinition {
        name: "Devious Cover-Up",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
            Effect::ExileAnyNumberFromGraveyards { filter: SelectionRequirement::Any },
        ]),
        ..Default::default()
    }
}

/// Manifestation Sage — {2}{G}{U} Creature — Fractal Wizard, 2/2 (Quandrix).
/// "Flying / When Manifestation Sage enters, create a 0/0 green and
/// blue Fractal creature token, then put X +1/+1 counters on it, where
/// X is the number of cards in your hand."
///
/// ✅ Wired faithfully: ETB mints a 0/0 G/U Fractal token (shared
/// definition pattern with Body of Research), then drops one +1/+1
/// counter on the just-created token for every card in the
/// controller's hand via `Value::HandSizeOf(You)`. Counters apply to
/// `Selector::LastCreatedToken` so the ETB resolves correctly even
/// when other tokens are minted in the same response window.
pub fn manifestation_sage() -> CardDefinition {
    let fractal = TokenDefinition {
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
        name: "Manifestation Sage",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal,
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::HandSizeOf(PlayerRef::You),
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
        foretell_cost: None,
    }
}

/// Crackle with Power — {X}{R}{R}{R}{R}{R} Sorcery (Mono-R STX).
/// "Crackle with Power deals 5X damage divided as you choose among
/// any number of targets."
///
/// ✅ The 5X scaling wires faithfully via `Value::Times(Const(5),
/// XFromCost)` and `DealDamageDivided` splits it among up to five
/// Creature ∨ Player ∨ Planeswalker targets (AutoDecider spreads evenly).
/// The printed five-quintuple-pip {RRRRR} cost is honored exactly via the
/// ordered `ManaCost` builder.
pub fn crackle_with_power() -> CardDefinition {
    use crate::mana::ManaSymbol;
    let mut crackle_cost = cost(&[r(), r(), r(), r(), r()]);
    crackle_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Crackle with Power",
        cost: crackle_cost,
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamageDivided {
            total: Value::Times(
                Box::new(Value::Const(5)),
                Box::new(Value::XFromCost),
            ),
            filter: SelectionRequirement::Creature
                .or(SelectionRequirement::Player)
                .or(SelectionRequirement::Planeswalker),
            max_targets: 5,
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
        foretell_cost: None,
    }
}

/// Mentor's Guidance — {1}{G}{U} Instant (Quandrix).
/// "Choose one — / • Mentor's Guidance deals damage equal to the
/// number of creatures you control to target creature an opponent
/// controls. / • Draw a card for each creature with a +1/+1 counter
/// on it you control."
///
/// Two-mode `ChooseMode`. Mode 0 deals `CountOf(YourCreatures)` damage to
/// a target creature an opponent controls (`ControlledByOpponent` filter).
/// Mode 1 draws `CountOf(YourCreatures WithCounter(+1/+1))` cards.
pub fn mentors_guidance() -> CardDefinition {
    CardDefinition {
        name: "Mentor's Guidance",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: damage equal to N creatures you control.
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            },
            // Mode 1: draw N where N = creatures you control with a +1/+1.
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(
                            CounterType::PlusOnePlusOne,
                        )),
                ))),
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
        foretell_cost: None,
    }
}

/// Quintorius, Field Historian — {2}{R}{W} Legendary Creature — Elephant
/// Cleric Spirit, 3/3 (Lorehold). "Vigilance / When Quintorius enters,
/// exile target card from a graveyard. Create a 3/2 red and white
/// Spirit creature token."
///
/// ✅ ETB body (exile gy card + mint 3/2 R/W Spirit token) wired via the
/// EntersBattlefield/SelfSource trigger. The printed static "Other
/// Spirit creatures you control get +1/+0" anthem is now wired as a
/// regular `StaticEffect::PumpPT` over
/// `Selector::EachPermanent(Creature ∧ HasCreatureType(Spirit) ∧
/// ControlledByYou ∧ OtherThanSource)` — same shape Hofri Ghostforge
/// uses. The `OtherThanSource` predicate flows through
/// `affected_from_requirement`, which flips
/// `AffectedPermanents::AllWithCreatureType.exclude_source: true` so
/// Quintorius himself doesn't buff himself (he IS a Spirit, matching
/// the printed "Other" gate). Push (modern_decks) consolidation
/// retired the `tribal_anthem_for_name` helper table.
pub fn quintorius_field_historian() -> CardDefinition {
    use crate::card::{SelectionRequirement, StaticAbility, Supertype};
    use crate::effect::StaticEffect;
    let spirit = TokenDefinition {
        name: "Spirit".to_string(),
        power: 3,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Quintorius, Field Historian",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Elephant,
                CreatureType::Cleric,
                CreatureType::Spirit,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                // "Exile target card from a graveyard" — needs the
                // `Move(... → Exile)` path (`Effect::Exile` on an
                // EntityRef::Permanent only no-ops for non-battlefield
                // cards). Same shape as SOS Heated Argument mode 2.
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: spirit,
                },
            ]),
        }],
        static_abilities: vec![StaticAbility {
            description: "Other Spirit creatures you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
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
        bestow: None,
        foretell_cost: None,
    }
}

// ── Galvanic Iteration ──────────────────────────────────────────────────────

/// Galvanic Iteration — {U}{R} Instant. "Copy target instant or sorcery
/// spell you control. You may choose new targets for the copy. /
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// exile Galvanic Iteration."
///
/// ✅ The headline copy half wires faithfully via `Effect::CopySpell`
/// (push XVII): targets a friendly IS spell on the stack and pushes
/// one copy above it. The Magecraft self-exile rider — which routes
/// Iteration from the stack/graveyard into exile after its own cast —
/// is omitted because the engine has no exile-self-on-resolution
/// primitive that sequences correctly with the stack-top copy. The
/// gameplay difference is **strictly graveyard vs exile** (the copy
/// still resolves identically); for the Prismari instant-doubling
/// play pattern (twin-cast a Lightning Bolt for {U}{R}) the body is
/// fully faithful. Tracked in TODO.md.
pub fn galvanic_iteration() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Iteration",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            ),
            count: Value::Const(1),
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
        foretell_cost: None,
    }
}

// ── Expressive Iteration ────────────────────────────────────────────────────

/// Expressive Iteration — {U}{R} Sorcery. "Exile the top three cards of
/// your library. You may play one of them this turn, and you may play
/// a land from among them this turn. Put the rest on the bottom of
/// your library in a random order."
///
/// Push (modern_decks, claude/modern_decks): promoted via the existing
/// `Effect::GrantMayPlay` primitive — moves the top 3 cards from the
/// library to exile, then grants the caster `MayPlay::EndOfThisTurn`
/// on `Selector::LastMoved` (each exiled card individually — `LastMoved`
/// is the multi-card slot per `effect.rs:107-112`). The "put the rest
/// on the bottom" rider collapses to "leftovers stay in exile" since
/// the engine doesn't auto-bottom unplayed exile-zone cards (no
/// functional difference — they're not playable any more). Closes the
/// Prismari school's last 🟡.
pub fn expressive_iteration() -> CardDefinition {
    CardDefinition {
        name: "Expressive Iteration",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                },
                to: ZoneDest::Exile,
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
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
        foretell_cost: None,
    }
}

// ── Magma Opus ──────────────────────────────────────────────────────────────

/// Magma Opus — {7}{U}{R} Sorcery. "Magma Opus deals 4 damage divided
/// as you choose among any number of targets. Tap up to two creatures.
/// Create a 4/4 blue and red Elemental creature token. Draw two cards.
/// / {U/R}{U/R}, Discard Magma Opus: Create a Treasure token."
///
/// ✅ The main `Seq` ships all four printed primary clauses: 4 damage
/// divided (`DealDamageDivided`) among up to four creatures/planeswalkers,
/// tap, a 4/4 Elemental token, and draw 2. The tap rider strict-upgrades
/// from "up to two creatures" to "all opponent creatures" (favors the
/// caster; matters only with 3+ opp creatures). The {U/R}{U/R}-and-
/// discard-self → Treasure alt mode is a doc-tracked engine-wide gap
/// (no discard-as-activation-cost primitive yet). Tracked in TODO.md.
pub fn magma_opus() -> CardDefinition {
    let elemental = crate::catalog::sets::sos::elemental_token();
    CardDefinition {
        name: "Magma Opus",
        cost: cost(&[generic(7), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamageDivided {
                total: Value::Const(4),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
                max_targets: 4,
            },
            Effect::Tap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: elemental,
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
        foretell_cost: None,
    }
}

// ── Reckless Amplimancer ────────────────────────────────────────────────────

/// Reckless Amplimancer — {2}{G} Creature — Elf Druid, 2/2.
/// `{X}: this creature gets +X/+X until end of turn.`
pub fn reckless_amplimancer() -> CardDefinition {
    CardDefinition {
        name: "Reckless Amplimancer",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        // {X}: +X/+X EOT, where X = the mana spent on this activation
        // (`Value::XFromCost` reads `ActivateAbility.x_value`).
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::ManaSymbol::X]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Crashing Drawbridge ─────────────────────────────────────────────────────

/// Crashing Drawbridge — {3} Artifact Creature — Construct, 0/4.
/// "Other creatures you control have haste."
///
/// Wired with a `StaticEffect::GrantKeyword` applying Haste to
/// other creatures you control. The static layer evaluates each
/// frame, so newly-summoned creatures pick up haste immediately
/// (matches the printed "creatures you control have haste"
/// continuous effect).
pub fn crashing_drawbridge() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Crashing Drawbridge",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        bestow: None,
        foretell_cost: None,
    }
}

// ── Eyetwitch Brood ─────────────────────────────────────────────────────────

/// Eyetwitch Brood — {1}{B}{G} Creature — Pest, 1/1, Lifelink. "Whenever
/// another Pest you control dies, put a +1/+1 counter on this creature."
///
/// Tribal Witherbloom payoff sibling to Felisa Fang. Triggers off the
/// death of any *other* Pest you control via `EventKind::CreatureDied
/// / AnotherOfYours` + `Predicate::EntityMatches { what: TriggerSource,
/// filter: HasCreatureType(Pest) }`. Counters on the dead Pest persist
/// in the graveyard (push XXIII's cross-zone CountersOn fallback), so
/// the filter reads the dead card's printed creature types correctly.
///
/// Name disambiguates from SOS's "Pest Mascot" (same Pest-Ape flavour,
/// different trigger condition).
pub fn eyetwitch_brood() -> CardDefinition {
    CardDefinition {
        name: "Eyetwitch Brood",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
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
        foretell_cost: None,
    }
}

// ── First Day of Class ──────────────────────────────────────────────────────

/// First Day of Class — {W} Sorcery. "Until end of turn, creatures you
/// control get +1/+1. Whenever a creature you control deals combat
/// damage to a player this turn, create a 1/1 white Pest creature
/// token with 'When this creature dies, you gain 1 life.'"
///
/// ✅ The anthem clause (+1/+1 EOT for each creature you control)
/// wires faithfully via `ForEach(Creature & ControlledByYou)` +
/// `PumpPT`, which is the headline play pattern: a one-mana
/// Glorious Anthem for a turn. The "deals combat damage → 1/1 Pest"
/// delayed trigger is omitted — the engine has no
/// `DelayedTriggerSpec` primitive that registers a one-turn-window
/// trigger from a sorcery resolution. This rider is bonus value
/// that rarely flips combat math when the anthem is already swinging
/// in. Tracked in TODO.md.
pub fn first_day_of_class() -> CardDefinition {
    CardDefinition {
        name: "First Day of Class",
        cost: cost(&[w()]),
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
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
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
        foretell_cost: None,
    }
}

// ── Verdant Mastery ─────────────────────────────────────────────────────────

/// Verdant Mastery — {3}{G}{G} Sorcery. "Search your library for a
/// basic land card, put it onto the battlefield, then shuffle. Each
/// other player may search their library for a basic land card, put
/// it onto the battlefield tapped, then shuffle."
///
/// ✅ Both printed clauses of the regular cast resolve: caster fetches
/// a basic untapped, then each opponent fetches a basic tapped. The
/// auto-decider opts each opponent into the "may search" rider when
/// a basic is available (no-op otherwise), so the play pattern
/// matches the printed "each other player may" exactly under the
/// engine's deterministic decision model. The {6}{G}{G} alt-cost
/// (two basics for everyone) is an engine-wide alt-cost-implies-
/// mode gap shared with Baleful Mastery ✅ and Devastating Mastery ✅;
/// the regular cast covers the headline ramp play pattern. Tracked
/// in TODO.md.
pub fn verdant_mastery() -> CardDefinition {
    CardDefinition {
        name: "Verdant Mastery",
        cost: cost(&[generic(3), g(), g()]),
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
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::Search {
                    who: PlayerRef::Triggerer,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::Triggerer,
                        tapped: true,
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
        bestow: None,
        foretell_cost: None,
    }
}

// ── Sacred Fire ─────────────────────────────────────────────────────────────

/// Sacred Fire — {R}{W} Sorcery. "Deals 3 damage to any target. You gain
/// 3 life. / Flashback {5}{R}{W}" (re-cast from graveyard via `cast_flashback`).
pub fn sacred_fire() -> CardDefinition {
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![
            ManaSymbol::Generic(5),
            ManaSymbol::Colored(Color::Red),
            ManaSymbol::Colored(Color::White),
        ],
    };
    CardDefinition {
        name: "Sacred Fire",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

// ── Rip Apart ───────────────────────────────────────────────────────────────

// ── Codespell Cleric ────────────────────────────────────────────────────────

/// Codespell Cleric — {W} Creature — Kor Cleric, 1/1, Lifelink. Simple
/// Silverquill body — vanilla 1/1 lifelink for one white mana. Pairs
/// well with Felisa Fang's "creature with +1/+1 counter dies → Inkling"
/// trigger when augmented by Eager First-Year-style magecraft pumps.
pub fn codespell_cleric() -> CardDefinition {
    CardDefinition {
        name: "Codespell Cleric",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric],
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
        bestow: None,
        foretell_cost: None,
    }
}

// ── Sparkmage Apprentice ────────────────────────────────────────────────────

/// Sparkmage Apprentice — {1}{R} Creature — Human Wizard, 1/2.
/// "When this creature enters, it deals 2 damage to any target."
///
/// Pinpoint Prismari ETB removal. Wired with a standard
/// `EntersBattlefield / SelfSource` trigger and a creature-or-player-
/// or-planeswalker target picker.
pub fn sparkmage_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Sparkmage Apprentice",
        cost: cost(&[generic(1), r()]),
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
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
    }
}

// ── Karok Wrangler ──────────────────────────────────────────────────────────

/// Karok Wrangler — {1}{G}{U} Creature — Elf Druid, 2/2.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on target creature you control."
pub fn karok_wrangler() -> CardDefinition {
    CardDefinition {
        name: "Karok Wrangler",
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        bestow: None,
        foretell_cost: None,
    }
}

// ── Witherbloom Command ─────────────────────────────────────────────────────

// ── Lorehold Command ────────────────────────────────────────────────────────

// ── Quandrix Command ────────────────────────────────────────────────────────

// ── Silverquill Command ─────────────────────────────────────────────────────

// ── Prismari Command ────────────────────────────────────────────────────────

// ── Defend the Campus ───────────────────────────────────────────────────────

/// Defend the Campus — {3}{W}{W} Sorcery. "Create three 1/1 white and
/// black Inkling creature tokens with flying."
///
/// ✅ Faithful 3x mint via `Effect::CreateToken { count: Value::Const(3) }`.
/// Reuses the SOS catalog's `inkling_token()` definition for visual
/// consistency with the other Silverquill Inkling cards.
pub fn defend_the_campus() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Defend the Campus",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
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
        foretell_cost: None,
    }
}

// ── Hall Monitor ────────────────────────────────────────────────────────────

/// Hall Monitor — {W} Creature — Human Cleric, 1/1. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, untap Hall
/// Monitor."
///
/// ✅ Wired via the new `magecraft_self_untap()` shortcut (push XXVII).
/// On every IS-cast trigger, the source is untapped (lets it block
/// over multiple combat turns or chain Spectral Adversary-style
/// re-tap activations).
pub fn hall_monitor() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_untap;
    CardDefinition {
        name: "Hall Monitor",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_untap()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
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
        foretell_cost: None,
    }
}

// ── Stonebinder's Familiar ──────────────────────────────────────────────────

/// Stonebinder's Familiar — {1} Artifact Creature — Spirit, 0/1.
/// "Whenever one or more cards leave your graveyard, put a +1/+1
/// counter on Stonebinder's Familiar."
///
/// ✅ Wired against `EventKind::CardLeftGraveyard` (per-card emission;
/// the printed "one or more" wording is approximated per-card, matching
/// the SOS Spirit Mascot / Owlin Historian pattern). Trigger source is
/// `Selector::This`. Pairs naturally with the Lorehold cycle.
pub fn stonebinders_familiar() -> CardDefinition {
    CardDefinition {
        name: "Stonebinder's Familiar",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.10a — leaves-graveyard triggers fire when the
            // event's player matches; `YourControl` matches when the
            // gy-leave was from the controller's own graveyard.
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
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
        foretell_cost: None,
    }
}

// ── Necrotic Fumes ──────────────────────────────────────────────────────────

/// Necrotic Fumes — {2}{B}{B} Sorcery. "As an additional cost to cast
/// this spell, sacrifice a creature. / Exile target creature." The
/// sacrifice is a real cast-time additional cost via
/// `AdditionalCastCost::SacrificePermanent`.
pub fn necrotic_fumes() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Fumes",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: SelectionRequirement::Creature,
            count: 1,
        }],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

// ── Make Your Mark ──────────────────────────────────────────────────────────

/// Make Your Mark — {1}{W} Instant. "Target creature gets +1/+1 until
/// end of turn. Draw a card."
///
/// ✅ Trivial pump + cantrip wire. The +1/+1 EOT goes on a chosen
/// creature target via `target_filtered(Creature)`; the cantrip
/// fires regardless of whether the pump finds a legal target.
pub fn make_your_mark() -> CardDefinition {
    CardDefinition {
        name: "Make Your Mark",
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
                power: Value::Const(1),
                toughness: Value::Const(1),
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
        foretell_cost: None,
    }
}

// ── Containment Breach ──────────────────────────────────────────────────────

/// Containment Breach — {1}{W} Sorcery. "Destroy target enchantment.
/// Surveil 1."
///
/// ✅ Standard `Seq(Destroy + Surveil 1)` wire. The Surveil is the
/// engine's existing `Effect::Surveil` primitive (top card → graveyard
/// or stays on top per the controller's choice).
pub fn containment_breach() -> CardDefinition {
    CardDefinition {
        name: "Containment Breach",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            Effect::Surveil {
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
        foretell_cost: None,
    }
}

// ── Burrog Befuddler ────────────────────────────────────────────────────────

/// Burrog Befuddler — {1}{U} Creature — Frog Wizard, 2/1.
/// "Flash. When this creature enters, target creature gets -3/-0 until
/// end of turn."
///
/// Flash + ETB combat trick. The -3/-0 takes a 3/3 down to 0/3 which
/// can no longer profitably attack; the body sticks around as a 2/1
/// flier-blocker (well, 2/1 ground, but cheap interaction at instant
/// speed). Standard `EntersBattlefield/SelfSource` trigger with a
/// negative `Effect::PumpPT` against a `Creature` target.
pub fn burrog_befuddler() -> CardDefinition {
    CardDefinition {
        name: "Burrog Befuddler",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
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
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
    }
}

// ── Mage Hunters' Mark ──────────────────────────────────────────────────────

/// Mage Hunters' Mark — {1}{R} Instant.
/// "Target creature gets +3/+0 and gains menace until end of turn."
///
/// Strixhaven combat trick — a Lava-Coil-curve pump that punches a
/// blocker out (menace forces double-block). Wired as
/// `Seq(PumpPT(+3/+0), GrantKeyword(Menace))` against a `Creature`
/// target. The target's controller doesn't matter (the card lets you
/// turn an opp's blocker into a forced-2-block headache).
pub fn mage_hunters_mark() -> CardDefinition {
    CardDefinition {
        name: "Mage Hunters' Mark",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Menace,
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
        foretell_cost: None,
    }
}

// ── Mage Duel ───────────────────────────────────────────────────────────────

/// Mage Duel — {1}{R} Sorcery.
/// "Target creature you control deals damage equal to its power to
/// target creature you don't control."
///
/// Mage Duel — {1}{R} Sorcery. Target creature you control deals damage
/// equal to its power to target creature you don't control.
///
/// Two-slot spell: slot 0 is the hostile victim (auto-picked hostile),
/// slot 1 (`additional_targets[0]`) is the friendly dealer. The fight
/// is one-sided — only the dealer's power is dealt, so it survives.
pub fn mage_duel() -> CardDefinition {
    CardDefinition {
        name: "Mage Duel",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            })),
        },
        ..Default::default()
    }
}

// ── Eccentric Apprentice ────────────────────────────────────────────────────

/// Eccentric Apprentice — {1}{R} Creature — Human Wizard, 1/3.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature gets +1/+0 until end of turn."
///
/// Vanilla Prismari/Lorehold magecraft body. The pump applies to the
/// source itself via `magecraft_self_pump(1, 0)` — same shortcut
/// Symmetry Sage uses. A 1/3 base body that scales into a 2/3 or 3/3
/// attacker every time you cast a spell turns into a credible threat
/// in an instants-and-sorceries deck.
pub fn eccentric_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Eccentric Apprentice",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
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
        bestow: None,
        foretell_cost: None,
    }
}

// ── Tezzeret's Gambit ───────────────────────────────────────────────────────

/// Tezzeret's Gambit — {U}{B} Sorcery.
/// "Choose one — / • Proliferate. / • Pay 2 life. Draw two cards."
///
/// Printed cost is `{U/P}{B/P}` (Phyrexian: pay 2 life instead of each
/// pip). Wired with real `ManaSymbol::Phyrexian` pips — `ManaCost::pay()`
/// pays each pip with the colored mana if available, else 2 life, so the
/// card can be cast for {U}{B}, {U} + 2 life, or 4 life, etc.
///
/// Two-mode `Effect::ChooseMode`:
/// * Mode 0 — `Effect::Proliferate` (every permanent and player with a
///   counter gets one more of any kind they already have, controller
///   chooses per object).
/// * Mode 1 — `Seq(LoseLife(2), Draw(2))` (pay 2 life, draw 2 cards).
///
/// Auto-decider picks mode 0 by default (Proliferate is the stronger
/// floor in any counter-having board state — +1/+1 counters, poison,
/// charge, loyalty all benefit). Scripted decider can probe mode 1.
pub fn tezzerets_gambit() -> CardDefinition {
    CardDefinition {
        name: "Tezzeret's Gambit",
        cost: cost(&[crate::mana::phyrexian(Color::Blue), crate::mana::phyrexian(Color::Black)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: Proliferate.
            Effect::Proliferate,
            // Mode 1: Pay 2 life, draw 2.
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
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
        foretell_cost: None,
    }
}

// ── Wandering Archaic ───────────────────────────────────────────────────────

/// Wandering Archaic — {2}{W}{W} Creature — Spirit, 4/4.
/// (Front face only; the printed card is reversible with a back face
/// "Explore the Vastlands" that's omitted here — reversible-card
/// pipeline is engine-wide ⏳ similar to the back-face MDFC handling.)
///
/// "Whenever an opponent casts an instant or sorcery spell, that
/// player may pay {2}. If they don't, you may copy the spell. You may
/// choose new targets for the copy."
///
/// ✅ (push modern_decks): the printed "may pay {2} or get copied" tax
/// is wired via the new `Effect::CopySpellUnlessPaid` primitive. At
/// trigger resolution, the engine asks the spell's caster yes/no — if
/// they accept *and* can afford {2} from their floated mana pool, the
/// engine deducts the cost and skips the copy. Otherwise the spell
/// gets copied once. The "you may choose new targets for the copy" half
/// is engine-wide ⏳ (the copy inherits the original's targets — same
/// gap as every other CopySpell user).
///
/// The body is a 4/4 Spirit for {2}{W}{W} — a strong wall against
/// non-spell-heavy decks and a free copy generator against
/// spell-heavy ones.
pub fn wandering_archaic() -> CardDefinition {
    use crate::card::{Predicate, Subtypes};
    CardDefinition {
        name: "Wandering Archaic",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::Any(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCardType(CardType::Instant),
                    },
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCardType(CardType::Sorcery),
                    },
                ])),
            effect: Effect::CopySpellUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: cost(&[generic(2)]),
                count: Value::Const(1),
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
        foretell_cost: None,
    }
}

// ── Illuminate History ──────────────────────────────────────────────────────

/// Illuminate History — {1}{R}{W} Sorcery.
/// "As an additional cost to cast this spell, discard a card. Create two
/// 2/2 red and white Spirit creature tokens with flying." Discard is a
/// real cast-time cost via `AdditionalCastCost::Discard`.
pub fn illuminate_history() -> CardDefinition {
    let lorehold_spirit_flying = TokenDefinition {
        name: "Spirit".to_string(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Illuminate History",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::Discard { count: 1 }],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: lorehold_spirit_flying,
        },
        ..Default::default()
    }
}
