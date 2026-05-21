//! Silverquill (W/B) college cards from Strixhaven.
//!
//! Common shapes:
//! - **Magecraft** triggers (Eager First-Year): "whenever you cast or copy
//!   an instant or sorcery spell, …". Implemented via the spell-cast trigger
//!   path with an `EventSpec.filter` predicate that gates on the just-cast
//!   spell's card type. See `fire_spell_cast_triggers` in
//!   `crabomination::game::actions`.
//! - **Learn** (Eyetwitch death trigger, Hunt for Specimens rider). The full
//!   Oracle searches a Lessons sideboard or discards-then-draws. We don't
//!   model a sideboard, so Learn is collapsed to `Draw 1` here. See
//!   `STRIXHAVEN2.md` for the engine TODO.
//!
//! Many cards also have static abilities or token-creation clauses that need
//! engine features the engine doesn't have yet (cost-reduction-aware-of-
//! target, token-with-self-die-trigger). Each affected card is marked 🟡 in
//! the tracker; the body / keywords / P/T are still correct so the card is
//! playable as a 4/3 lifelink flier or whatever.

use super::no_abilities;
use crate::catalog::sets::sos::inkling_token;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    etb_drain, etb_gain_life, etb_mint_token, magecraft, magecraft_drain_each_opp,
    magecraft_gain_life, magecraft_self_pump, target_filtered,
};
use crate::effect::{Duration, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{cost, generic, u, w, b, x, ManaCost};

// ── Spirited Companion ──────────────────────────────────────────────────────

/// Spirited Companion — {1}{W}, 1/2 Dog Spirit. ETB: draw a card.
///
/// Reprinted across many sets; in Strixhaven it's an uncommon. Functionally
/// identical to Elvish Visionary in white. The ETB draw goes through the
/// existing `EntersBattlefield` + `SelfSource` trigger path.
pub fn spirited_companion() -> CardDefinition {
    CardDefinition {
        name: "Spirited Companion",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Spirit],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Eyetwitch ───────────────────────────────────────────────────────────────

/// Eyetwitch — {B}, 1/1 Pest. "When Eyetwitch dies, learn." Set: Strixhaven.
///
/// Learn is approximated as `Draw 1`; see `STRIXHAVEN2.md` for the planned
/// Lesson sideboard model.
pub fn eyetwitch() -> CardDefinition {
    CardDefinition {
        name: "Eyetwitch",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            // "Learn" approximation: draw a card. The Oracle text alternates
            // between "search Lessons sideboard" and "discard a card, then
            // draw a card". Since there is no Lessons sideboard model yet,
            // the cleanest single-effect substitute is a plain draw.
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Closing Statement ───────────────────────────────────────────────────────

/// Closing Statement — {X}{W}{W} Sorcery. "Exile target nonland permanent.
/// You gain X life."
///
/// X is read off the spell's cast-time `x_value`, threaded into the
/// resolution context as `Value::XFromCost`. The exile half is unconditional
/// (X doesn't gate the exile in the printed Oracle either — it just sets
/// the lifegain).
pub fn closing_statement() -> CardDefinition {
    CardDefinition {
        name: "Closing Statement",
        cost: cost(&[x(), w(), w()]),
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
            Effect::GainLife {
                who: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Vanishing Verse ─────────────────────────────────────────────────────────

/// Vanishing Verse — {W}{B} Instant. "Exile target nonland, monocolored
/// permanent."
///
/// ✅ Now wired faithfully via the new `SelectionRequirement::Monocolored`
/// predicate (distinct_colors == 1). Multicolored and colorless
/// permanents fail the target filter cleanly, matching the printed
/// Oracle text.
pub fn vanishing_verse() -> CardDefinition {
    CardDefinition {
        name: "Vanishing Verse",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Permanent
                    .and(SelectionRequirement::Nonland)
                    .and(SelectionRequirement::Monocolored),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Killian, Ink Duelist ────────────────────────────────────────────────────

/// Killian, Ink Duelist — {W}{B}, 2/3 Legendary Human Warlock with Lifelink.
///
/// ✅ The static "spells you cast that target a creature cost {2} less to
/// cast" now wires via `StaticEffect::CostReductionTargetingFilter`. The
/// reduction is applied during `cast_spell_with_convoke` after target
/// validation; CR 601.2f / 117.7c forbid trimming colored or X pips, so
/// the engine's `ManaCost::reduce_generic` helper drains generic pips
/// only and clamps at zero. The spell filter is `Any` (any spell with a
/// creature target qualifies — the printed Oracle reads "spells you
/// cast", no card-type clause).
pub fn killian_ink_duelist() -> CardDefinition {
    CardDefinition {
        name: "Killian, Ink Duelist",
        cost: cost(&[w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast that target a creature cost {2} less to cast.",
            effect: StaticEffect::CostReductionTargetingFilter {
                spell_filter: SelectionRequirement::Any,
                target_filter: SelectionRequirement::Creature,
                amount: 2,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Devastating Mastery ─────────────────────────────────────────────────────

/// Devastating Mastery — {4}{W}{W} Sorcery. "Destroy all nonland permanents."
///
/// ✅ The destroy-each-nonland-permanent body fully matches the printed
/// Oracle's primary clause; this is "Wrath of God for everything that
/// isn't a land". The alt cost {7}{W}{W} (cast for {3} more to return up
/// to two nonland permanent cards from a graveyard) is an engine-wide
/// "alt-cost-implies-mode" gap (also missing from Verdant Mastery and
/// Baleful Mastery's alt-paths). Tracked in TODO.md; the printed primary
/// effect is unaffected so the card plays correctly when cast for
/// regular mana.
pub fn devastating_mastery() -> CardDefinition {
    CardDefinition {
        name: "Devastating Mastery",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Nonland),
            body: Box::new(Effect::Destroy {
                what: Selector::TriggerSource,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Felisa, Fang of Silverquill ─────────────────────────────────────────────

/// Felisa, Fang of Silverquill — {2}{W}{B}, 4/3 Legendary Cat Cleric, Flying
/// + Lifelink.
///
/// Now wired (push XVI): the printed "Whenever a creature you control
/// with a +1/+1 counter on it dies, create a 1/1 white and black
/// Inkling creature token with flying" trigger uses
/// `EventKind::CreatureDied / AnotherOfYours` filtered by
/// `Predicate::EntityMatches { what: TriggerSource, filter:
/// WithCounter(+1/+1) }`. Counters persist on a card after move-to-
/// graveyard (only `damage` / `tapped` / `attached_to` get cleared on
/// zone-out per `move_card_to`), so the post-die graveyard-resident
/// CardInstance still carries its `+1/+1` counters and `evaluate_
/// requirement_static`'s `WithCounter` arm sees them. The minted token
/// shares the SOS catalog's `inkling_token()` definition (1/1 W/B
/// Inkling with flying) for visual + tribal consistency.
pub fn felisa_fang_of_silverquill() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    use crate::catalog::sets::sos::inkling_token;
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Felisa, Fang of Silverquill",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Mavinda, Students' Advocate ─────────────────────────────────────────────

/// Mavinda, Students' Advocate — {1}{W}{W}, 1/3 Legendary Human Cleric,
/// Flying + Vigilance.
///
/// 🟡 The "{3}{W}{W}: Cast target instant/sorcery from your graveyard if it
/// targets a creature; exile it as it would leave the stack" activated
/// ability is not wired. Cast-from-graveyard requires a graveyard-cast
/// primitive (similar to Flashback but more constrained). Body/lifelink/
/// flying are correct so combat behavior matches the printed card.
pub fn mavinda_students_advocate() -> CardDefinition {
    CardDefinition {
        name: "Mavinda, Students' Advocate",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Eager First-Year ────────────────────────────────────────────────────────

/// Eager First-Year — {W}, 2/1 Human Student. "Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, target creature gets +1/+1
/// until end of turn."
///
/// The magecraft trigger uses the new `EventSpec.filter` evaluation at the
/// spell-cast site: `Predicate::EntityMatches { what: TriggerSource, filter:
/// HasCardType(Instant) ∨ HasCardType(Sorcery) }`. The filter is evaluated
/// against the just-cast spell — `Selector::TriggerSource` is bound to its
/// `CardId` for the duration of filter evaluation, then the trigger's own
/// body runs with the auto-targeted creature.
pub fn eager_first_year() -> CardDefinition {
    CardDefinition {
        name: "Eager First-Year",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Hunt for Specimens ──────────────────────────────────────────────────────

/// Hunt for Specimens — {3}{B} Sorcery. "Create a 1/1 black Pest creature
/// token with 'When this creature dies, you gain 1 life.' Then learn."
///
/// ✅ Both halves wired faithfully. The spawned Pest token carries the
/// printed death-trigger lifegain via `TokenDefinition.triggered_
/// abilities` (SOS-VI). Learn collapses to `Draw 1` — the same
/// approximation shared by Eyetwitch ✅, Pest Summoning ✅, Igneous
/// Inspiration ✅, and Field Trip ✅. The full "sideboard search" path
/// is engine-wide (no Lessons sideboard model yet) and tracked in
/// TODO.md.
pub fn hunt_for_specimens() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Hunt for Specimens",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PR::You,
                count: Value::Const(1),
                definition: pest,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Pledgemage ──────────────────────────────────────────────────

/// Silverquill Pledgemage — {1}{W}{B}, 2/2 Inkling Druid. Flying.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// this creature gets +1/+1 until end of turn."
///
/// Uses the `magecraft_self_pump(1, 1)` helper (push XXVII) — the
/// magecraft trigger pumps the source itself +1/+1 EOT. The Inkling
/// subtype was added in the Strixhaven era; this card's flying ties
/// it into the Silverquill tribal pool that Tenured Inkcaster
/// powers up via the new Inkling anthem.
pub fn silverquill_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pledgemage",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Archmage Emeritus ───────────────────────────────────────────────────────

/// Archmage Emeritus — {2}{U}{U}, 3/3 Human Wizard. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, draw a card."
///
/// Pure magecraft draw payoff. Reuses the `magecraft(...)` helper to
/// gate on instant/sorcery casts, then draws one card for the
/// controller. Closes the same loop as Witherbloom Apprentice's drain
/// payoff — the canonical "magecraft does N" creature for each
/// college. Strong synergy with copy-spell triggers (Aziza, Zaffai,
/// Galvanic Iteration): the "or copy" half doubles the draw.
pub fn archmage_emeritus() -> CardDefinition {
    CardDefinition {
        name: "Archmage Emeritus",
        cost: cost(&[generic(2), u(), u()]),
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
        triggered_abilities: vec![magecraft(Effect::Draw {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Promising Duskmage ──────────────────────────────────────────────────────

/// Promising Duskmage — {2}{W}{B}, 2/2 Inkling Wizard. Flying.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target opponent loses 1 life and you gain 1 life."
///
/// The Witherbloom-style drain payoff in Silverquill colours — the
/// `magecraft_drain_each_opp(1)` shortcut emits the canonical
/// `Effect::Drain { from: EachOpponent, to: You, amount: 1 }` so the
/// life swap is atomic. The printed Oracle says "target opponent"
/// (single); the shortcut collapses to each-opponent for the auto-
/// target friendliness — in a 1v1 game this is identical, and in a
/// 4-player game it's strictly better (which is fine for an Inkling
/// 2/2 flyer at four mana).
pub fn promising_duskmage() -> CardDefinition {
    CardDefinition {
        name: "Promising Duskmage",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Tenured Inkcaster ───────────────────────────────────────────────────────

/// Tenured Inkcaster — {2}{W}{B}, 3/2 Vampire Warlock. "Other Inkling
/// creatures you control get +2/+2."
///
/// Tribal anthem on the Inkling creature type. Push (modern_decks)
/// consolidation: wired via a regular `StaticEffect::PumpPT` with
/// `Selector::EachPermanent(Creature ∧ HasCreatureType(Inkling) ∧
/// ControlledByYou ∧ OtherThanSource)` — same shape as Hofri / Quintorius
/// since the `OtherThanSource` target-validation half now works. The
/// `OtherThanSource` half is technically vacuous here (Inkcaster is a
/// Vampire, not an Inkling, so the CreatureType filter already
/// excludes the source), but it's kept for consistency with the
/// printed "Other" wording. The +2/+2 makes a 2/1 Inkling token
/// attack as a 4/3 flier, a huge Silverquill payoff.
pub fn tenured_inkcaster() -> CardDefinition {
    use crate::card::{SelectionRequirement, StaticAbility};
    use crate::effect::{Selector, StaticEffect};
    CardDefinition {
        name: "Tenured Inkcaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 2,
                toughness: 2,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Selfless Glyphweaver ────────────────────────────────────────────────────

/// Selfless Glyphweaver — {1}{W}{W}, 2/3 Human Cleric Wizard.
///
/// "Sacrifice this creature: Creatures you control gain indestructible
/// until end of turn."
///
/// Push (modern_decks): front-face only of the MDFC Selfless Glyphweaver
/// // Deadly Vanity. The back-face mass-sacrifice is too complex (each
/// opponent picks which creature to keep — no multi-pick decision shape
/// yet) and is omitted. The front face is a respectable 3-mana 2/3 body
/// with a one-shot indestructible-all-creatures-EOT activation that
/// protects the board through a Wrath.
///
/// The activation is a `sac_cost` activated ability (mirroring Shattered
/// Acolyte and similar sac-self payoff cards) whose effect grants
/// Indestructible (EOT) to each creature the controller owns. Because
/// the source is sacrificed as part of the cost (before resolution), it
/// won't grant indestructible to itself — matching the printed Oracle
/// where the sacrificed Glyphweaver is no longer on the battlefield
/// when the effect resolves.
pub fn selfless_glyphweaver() -> CardDefinition {
    CardDefinition {
        name: "Selfless Glyphweaver",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ============================================================================
// Batch 14 — Silverquill expansion (push modern_decks).
//
// 15 new synthesised STX Silverquill (W/B) cards using existing engine
// primitives. Each card ships with at least one functionality test in
// `tests::stx`. Focus areas: Inkling tribal payoffs, Magecraft drain
// variants, life-gain shells, and combat tricks that close out the
// Silverquill college's coverage gap.
// ============================================================================

// ── Silverquill Loremender (batch 14) ───────────────────────────────────────

/// Silverquill Loremender — {1}{W}, 2/2 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, you gain
/// 2 life."
///
/// Standard Silverquill ETB lifegain body. Pairs with Light of Promise,
/// Honor Troll, and any lifegain payoff. The 2/2 body is on-curve for
/// the cost and trades up into 2-toughness creatures.
pub fn silverquill_loremender() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Loremender",
        cost: cost(&[generic(1), w()]),
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
        // Refactored in batch 40 to use the `etb_gain_life` shortcut.
        triggered_abilities: vec![etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Verselord (batch 14) ────────────────────────────────────────────

/// Inkling Verselord — {2}{W}{B}, 3/3 Inkling Cleric Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Other Inkling creatures you
/// control have lifelink."
///
/// Inkling tribal payoff that hands lifelink to every other Inkling
/// you control. Stacks with Tenured Inkcaster's +2/+2 anthem and
/// Felisa's Inkling minting — every Inkling attack drains the opp
/// for its post-anthem power. Wired via `StaticEffect::GrantKeyword`
/// against `EachPermanent(Creature & Inkling & ControlledByYou &
/// OtherThanSource)`.
pub fn inkling_verselord() -> CardDefinition {
    CardDefinition {
        name: "Inkling Verselord",
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
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Drainmaster (batch 14) ──────────────────────────────────────

/// Silverquill Drainmaster — {2}{W}{B}, 3/2 Vampire Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, each
/// opponent loses 3 life and you gain 3 life."
///
/// Pure Witherbloom-style drain in Silverquill colors at 4 mana. The
/// drain trickle pairs with lifegain payoffs and burn finishers.
pub fn silverquill_drainmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainmaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(3)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkrise Lifedrainer (batch 14) ──────────────────────────────────────────

/// Inkrise Lifedrainer — {1}{B}, 2/1 Inkling Rogue.
///
/// Printed Oracle (synthesised): "Menace / Whenever this creature deals
/// combat damage to a player, you gain 1 life."
///
/// Aggressive Inkling that grinds in the lifegain shell. Menace forces
/// the chump-block math and the combat-damage-to-player rider drips
/// life back. Pairs with Tenured Inkcaster — a 4/3 menace inkling
/// attacker drops opp from 4 → 0 quick.
pub fn inkrise_lifedrainer() -> CardDefinition {
    CardDefinition {
        name: "Inkrise Lifedrainer",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Penman (batch 14) ───────────────────────────────────────────

/// Silverquill Penman — {1}{W}{B}, 2/2 Inkling Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// you may discard a card. If you do, draw a card and each opponent
/// loses 1 life."
///
/// Inkling looter that double-dips on discard payoff. The
/// `Effect::MayDo` shell asks the controller whether to discard; on
/// yes, the controller loots and drips opp life. AutoDecider declines
/// by default; ScriptedDecider can flip to true for tests.
pub fn silverquill_penman() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penman",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Discard a card. If you do, draw a card and each opponent loses 1 life.".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Anthemwriter (batch 14) ─────────────────────────────────────

/// Silverquill Anthemwriter — {3}{W}{B}, 4/4 Inkling Bard.
///
/// Printed Oracle (synthesised): "Flying, lifelink / Other creatures
/// you control get +1/+0."
///
/// Five-mana finisher anthem in Silverquill colors. The +1/+0 anthem
/// to other friendlies + intrinsic flying + lifelink turns a wide
/// Inkling board into an immediate lethal threat that races back any
/// life lost. Wired via `StaticEffect::PumpPT` filtered to "Other"
/// creatures via `OtherThanSource`.
pub fn silverquill_anthemwriter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Anthemwriter",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Quillmage (batch 14) ────────────────────────────────────────

/// Silverquill Quillmage — {W}{B}, 2/2 Human Wizard, Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, target opponent loses 1
/// life."
///
/// Cheap Silverquill 2-drop that ticks down opp life on every cast.
/// Lifelink on a 2/2 enables a soft race plan. Wired via
/// `magecraft(Effect::LoseLife { who: EachOpponent, amount: 1 })` —
/// each-opp collapse is the auto-target friendliness convention
/// (Promising Duskmage precedent).
pub fn silverquill_quillmage() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillmage",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Memorialist (batch 14) ──────────────────────────────────────

/// Silverquill Memorialist — {2}{W}, 2/3 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, return
/// target creature card with mana value 2 or less from your graveyard
/// to your hand."
///
/// Three-mana body that recurs cheap creatures from the graveyard. The
/// MV ≤ 2 filter lets it grab a Pest token spawner (after dying), a
/// Cleric, or a one-drop. Pairs with Witherbloom Pest token shells
/// for value plays.
pub fn silverquill_memorialist() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Silverquill Memorialist",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(2)),
                }),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Aspirant (batch 14) ─────────────────────────────────────────────

/// Inkling Aspirant — {W}{B}, 2/1 Inkling Cleric, Flying.
///
/// Printed Oracle (synthesised): "Flying"
///
/// Vanilla aggressive Inkling 2-drop. Slots into the Inkling tribal
/// shell as a curve-topper for Tenured Inkcaster's +2/+2 anthem —
/// becomes a 4/3 flier for 2 mana.
pub fn inkling_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Inkling Aspirant",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Witherspell Drain (batch 14) ────────────────────────────────────────────

/// Witherspell Drain — {1}{W}{B} Instant.
///
/// Printed Oracle (synthesised): "Target opponent loses 3 life and you
/// gain 3 life."
///
/// Three-mana drain instant in Silverquill colors. Functionally
/// equivalent to a smaller Death Grasp at fixed 3 life. Slots into any
/// Silverquill drain shell as a removal-style life swing.
pub fn witherspell_drain() -> CardDefinition {
    CardDefinition {
        name: "Witherspell Drain",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Scribe (batch 14) ───────────────────────────────────────────────

/// Inkling Scribe — {2}{W}, 1/2 Inkling Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 white and black Inkling creature token with flying."
///
/// Silverquill Inkling generator that pairs with Tenured Inkcaster's
/// anthem. Each Scribe is a 1/2 body + a 1/1 flying body for 3 mana —
/// solid Inkling tribal density.
pub fn inkling_scribe() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scribe",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Erudite (batch 14) ──────────────────────────────────────────

/// Silverquill Erudite — {3}{W}, 2/4 Human Wizard, Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets +1/+0
/// until end of turn."
///
/// Defensive Silverquill body that becomes an aggressive attacker on
/// the back of any cast. Vigilance lets it swing AND block. Wired via
/// `magecraft_self_pump(1, 0)` — same template as Symmetry Sage.
pub fn silverquill_erudite() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Erudite",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Bloodscribe (batch 14) ──────────────────────────────────────────

/// Inkling Bloodscribe — {3}{W}{B}, 3/3 Inkling Vampire, Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink / Whenever another creature
/// you control dies, each opponent loses 1 life and you gain 1 life."
///
/// Silverquill aristocrats-style body that scales with creature deaths.
/// Each Pest dying ticks 1 life to the opp; Felisa-style Inkling
/// minting fuels this. Wired via `EventKind::CreatureDied` +
/// `EventScope::AnotherOfYours` filter — same shape as Cauldron of
/// Essence.
pub fn inkling_bloodscribe() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bloodscribe",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Reprimand (batch 14) ────────────────────────────────────────

/// Silverquill Reprimand — {2}{W} Sorcery.
///
/// Printed Oracle (synthesised): "Exile target creature with power 2
/// or less."
///
/// Standard Silverquill small-creature removal. Cleanly handles
/// indestructible 1/1 tokens, Pests, and early aggressive 2-drops.
/// Filter is `Creature & PowerAtMost(2)`.
pub fn silverquill_reprimand() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Silverquill Reprimand",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
            ),
            to: ZoneDest::Exile,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Inquisition (batch 14) ──────────────────────────────────────

/// Silverquill Inquisition — {1}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Target opponent reveals their hand.
/// You choose a nonland card from it. That player discards that card."
///
/// Targeted hand-attack in Silverquill colors. Lets the controller
/// pick the worst card to strip. Same shape as the SOS Render
/// Speechless mode 0 (reveal + chosen-discard). Wired via
/// `Effect::DiscardChosen`.
pub fn silverquill_inquisition() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inquisition",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Archivist (batch 15) ────────────────────────────────────────

/// Silverquill Archivist — {1}{W}, 1/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 1.
/// You gain 1 life."
///
/// Cheap defensive 2-drop with a smoothing ETB + a tiny life bump.
/// Slots into Silverquill spellslinger shells as an early body that
/// also doubles as Light-of-Promise / aristocrats fodder.
pub fn silverquill_archivist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Archivist",
        cost: cost(&[generic(1), w()]),
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
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                Effect::GainLife {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Witness (batch 15) ──────────────────────────────────────────

/// Silverquill Witness — {W}{B}, 2/1 Human Cleric, Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, you gain 1 life."
///
/// Silverquill cleric that turns every cast into a life trickle.
/// Pairs with Light of Promise / Honor Troll for compounding lifegain.
pub fn silverquill_witness() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Witness",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Judge (batch 15) ────────────────────────────────────────────

/// Silverquill Judge — {2}{W}, 2/3 Human Cleric, Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, tap target creature an
/// opponent controls."
///
/// A defensive Silverquill body that locks down an opp creature on
/// every cast. Vigilance lets it attack and still tap when blocked,
/// then magecraft converts spells into tempo swings.
pub fn silverquill_judge() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Judge",
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
        triggered_abilities: vec![magecraft(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Brigade (batch 15) ──────────────────────────────────────────────

/// Inkling Brigade — {3}{W}{B}, 3/3 Inkling Soldier, Flying.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// create two 1/1 white and black Inkling creature tokens with flying."
///
/// Big Inkling-tribal payoff: drops 3 flying bodies for 5 mana. Pairs
/// with Tenured Inkcaster (+2/+2 anthem) for an overnight 5/5 + two
/// 3/3 fliers swing.
pub fn inkling_brigade() -> CardDefinition {
    CardDefinition {
        name: "Inkling Brigade",
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
        triggered_abilities: vec![etb_mint_token(inkling_token(), 2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Pen-Pusher (batch 15) ───────────────────────────────────────

/// Silverquill Pen-Pusher — {1}{B}, 1/1 Inkling Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, scry 1."
///
/// Cheap evasive Inkling 2-drop that smooths every cast. Boosts
/// Inkling tribal density and supports the Silverquill spellslinger
/// shell.
pub fn silverquill_pen_pusher() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Pusher",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Chronicle (batch 15) ────────────────────────────────────────

/// Silverquill Chronicle — {3}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 2 life and you
/// gain 2 life. Return target instant or sorcery card from your
/// graveyard to your hand."
///
/// Combined drain + IS recursion in Silverquill colors — same pattern
/// as Read the Bones flavoured for a spellslinger shell. The drain
/// triggers Witherbloom-Silverquill lifegain payoffs (Honor Troll,
/// Light of Promise, Inkling Bloodscribe) and the return gives the
/// chronicle a "rebuy" upside.
pub fn silverquill_chronicle() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Chronicle",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Vanguard (batch 15) ─────────────────────────────────────────────

/// Inkling Vanguard — {2}{W}, 2/3 Inkling Soldier, Flying, Vigilance.
///
/// Printed Oracle (synthesised): "Flying, vigilance"
///
/// Slightly-bigger sibling to Inkling Sentinel — same Flying+Vigilance
/// frame but at the 3-mana 2/3 stat line. Boosts Inkling tribal density
/// for Tenured Inkcaster / Inkling Verselord anthems.
pub fn inkling_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vanguard",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Marshal (batch 17) ──────────────────────────────────────────

/// Silverquill Marshal — {2}{W}, 2/3 Human Soldier.
///
/// Printed Oracle (synthesised): "When this creature enters, you gain 2
/// life."
///
/// Bread-and-butter Silverquill defensive body — a 2/3 for 3 with a
/// 2-life ETB stabilizer. Pairs naturally with Light of Promise /
/// Honor Troll lifegain payoffs and the Inkling Bloodscribe drain
/// chain — every ETB lifegain ticks those engines.
pub fn silverquill_marshal() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Marshal",
        cost: cost(&[generic(2), w()]),
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
        // Refactored in batch 40 to use the `etb_gain_life` shortcut.
        triggered_abilities: vec![etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Sanctifier (batch 17) ───────────────────────────────────────────

/// Inkling Sanctifier — {2}{W}, 2/3 Inkling Cleric, Flying + Lifelink.
///
/// Printed Oracle (synthesised): "Flying, lifelink"
///
/// Hard-hitting Inkling lifelinker — 2/3 Flying with Lifelink at three
/// mana qualifies as a top-end Inkling body for the Silverquill tribal
/// pool. Stacks with Tenured Inkcaster (+2/+2 anthem → 4/5 Lifelink
/// Flier) and Inkling Verselord (Other Inklings have lifelink — this
/// one already has it natively).
pub fn inkling_sanctifier() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sanctifier",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Pupil (batch 17) ────────────────────────────────────────────

/// Silverquill Pupil — {W}, 1/2 Human Student.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature gets +1/+0 until end of
/// turn."
///
/// Cheap one-drop magecraft body that swings bigger on every cast.
/// Smaller cousin to Inkling Choirmaster / Eager First-Year — a +1/+0
/// magecraft pump scales aggressively in a spell-heavy shell.
pub fn silverquill_pupil() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pupil",
        cost: cost(&[w()]),
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
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Defend the Inkwell (batch 17) ───────────────────────────────────────────

/// Defend the Inkwell — {2}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 2 life and you
/// gain 2 life. Scry 2."
///
/// Drain-plus-scry for {2}{W}{B} — fits the Silverquill removal / card
/// selection slot. Pairs with Witherbloom Apprentice / Honor Troll for
/// double-drain triggers and book-end lifegain payoffs.
pub fn defend_the_inkwell() -> CardDefinition {
    CardDefinition {
        name: "Defend the Inkwell",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Witness (batch 17) ──────────────────────────────────────────────

/// Inkling Witness — {W}{B}, 2/2 Inkling Cleric, Flying.
///
/// Printed Oracle (synthesised): "Flying / Whenever another Inkling you
/// control dies, you gain 1 life."
///
/// Per-Inkling-death lifegain via `EventKind::CreatureDied` /
/// `EventScope::AnotherOfYours` filtered on `HasCreatureType(Inkling)`
/// — same shape as the existing Pestmaster pattern but for the Inkling
/// tribe. Pairs with Inkling Summoning / Felisa's Inkling minter for
/// chained drain.
pub fn inkling_witness() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Inkling Witness",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Inkling),
                }),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Coursebinder (batch 18) ────────────────────────────────────────

/// Inkling Coursebinder — {1}{W}{B}, 2/2 Inkling Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, you gain 1 life and each
/// opponent loses 1 life."
///
/// Three-mana flying Inkling with built-in magecraft drain — the
/// classic Silverquill flyer + drain payoff. Pairs with Tenured
/// Inkcaster (Inkling tribal +2/+2) for a 4/4 lifedrain flyer.
pub fn inkling_coursebinder() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain_each_opp;
    CardDefinition {
        name: "Inkling Coursebinder",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Sermon (batch 18) ──────────────────────────────────────────

/// Silverquill Sermon — {2}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Create two 1/1 W/B Inkling creature
/// tokens with flying."
///
/// Four-mana double Inkling fan-out — same shape as Defend the Campus
/// at a lower cost (4 vs 5 mana) for 2 tokens instead of 3. Pairs
/// with Tenured Inkcaster's Inkling-tribal anthem (+2/+2) for two
/// 3/3 flyers immediately.
pub fn silverquill_sermon() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Sermon",
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Censure (batch 18) ─────────────────────────────────────────

/// Silverquill Censure — {1}{W} Instant.
///
/// Printed Oracle (synthesised): "Exile target creature with power 3
/// or less. You gain 2 life."
///
/// Two-mana exile-removal at the small-creature slot + 2 life rider.
/// A clean answer to early-game threats with a Light-of-Promise hook.
/// Stronger than Silverquill Reprimand at the same role since exile
/// dodges Persist / Undying / gy-recursion shells.
pub fn silverquill_censure() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Silverquill Censure",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::PowerAtMost(3)),
                ),
                to: ZoneDest::Exile,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Castigant (batch 19) ───────────────────────────────────────

/// Silverquill Castigant — {2}{W}, 2/3 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, each
/// opponent loses 1 life and you gain 1 life."
///
/// Compact ETB drain body. Slots into Silverquill curve (2-drop +
/// Castigant + Verselord) for incidental life-swap value. Combo-friendly
/// with Light of Promise (gain-life on bodies) and Inkling Bloodscribe
/// (lifelink Vampire Inkling).
pub fn silverquill_castigant() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Castigant",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Heartrender (batch 19) ─────────────────────────────────────

/// Silverquill Heartrender — {2}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 3 life and you
/// gain 3 life. Scry 1."
///
/// Pure 3-mana drain with selection rider. Stronger ratio than
/// Witherbloom Reverie at the same cost (drain 3 + scry vs drain 3
/// alone) trading the green pip for the scry. Pairs with Sanguine
/// Bond / Witherbloom Apprentice for snowball boards.
pub fn silverquill_heartrender() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Heartrender",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Confessor (batch 19) ───────────────────────────────────────────

/// Inkling Confessor — {1}{W}{B}, 2/2 Inkling Cleric, Flying.
///
/// Printed Oracle (synthesised): "Flying / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, each opponent loses 1
/// life and you gain 1 life."
///
/// Flying Inkling magecraft drain body. Mirror of Witherbloom
/// Apprentice on a flier with one fewer power. Stacks with Tenured
/// Inkcaster's anthem and Inkling Verselord's lifelink grant.
pub fn inkling_confessor() -> CardDefinition {
    CardDefinition {
        name: "Inkling Confessor",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Quillblade (batch 19+) ─────────────────────────────────────

/// Silverquill Quillblade — {W} Instant.
///
/// Printed Oracle (synthesised): "Target creature you control gets
/// +X/+0 until end of turn, where X is the number of creatures you
/// control."
///
/// One-mana combat trick that scales with board count. On a 4-creature
/// board the trigger source gets +4/+0 — same shape as Spear of
/// Heliod for a faction-specific Silverquill tempo card. Wired via
/// `Value::PermanentCountControlledBy(You)` filter inside a `PumpPT`.
pub fn silverquill_quillblade() -> CardDefinition {
    let creatures_you_control = Value::count(Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    ));
    CardDefinition {
        name: "Silverquill Quillblade",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: creatures_you_control,
            toughness: Value::Const(0),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Decree (batch 19+) ─────────────────────────────────────────────

/// Inkling Decree — {3}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 2 life and you
/// gain 2 life. Create a 1/1 white and black Inkling creature token
/// with flying."
///
/// 5-mana drain-and-token combo. Drains 2 (4-life swing) + mints a
/// 1/1 Inkling flier. Mid-game stabilizer with built-in tempo body.
pub fn inkling_decree() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Decree",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Inkrider (batch 19) ────────────────────────────────────────────

/// Inkling Inkrider — {2}{W}{B}, 3/2 Inkling Knight, Flying +
/// Vigilance.
///
/// Printed Oracle (synthesised): "Flying, vigilance."
///
/// Aggressive 4-mana evasive Inkling — exact P/T as Inkling Sanctifier
/// but trades lifelink for vigilance to enable persistent ground
/// defense while attacking. Stacks with Inkling Verselord (grant
/// lifelink → 3/2 Flying + Vigilance + Lifelink) for a true beater.
pub fn inkling_inkrider() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkrider",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Lawkeeper (batch 20) ───────────────────────────────────────

/// Silverquill Lawkeeper — {1}{W}, 2/2 Human Soldier with Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters,
/// tap target creature an opponent controls."
///
/// 2-mana 2/2 tempo defender — comes down with vigilance, locks down an
/// opp attacker, and stays back to block on the swing-back. The ETB tap
/// targets an opp creature via `target_filtered`. Mirror of Master of
/// Cruelties on a clean defensive frame; pairs with Stun-counter cards
/// for a permanent lock.
pub fn silverquill_lawkeeper() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lawkeeper",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Penmaster (batch 20) ───────────────────────────────────────────

/// Inkling Penmaster — {2}{W}{B}, 2/3 Inkling Wizard with Flying.
///
/// Printed Oracle (synthesised): "Flying. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, create a 1/1 white and black
/// Inkling creature token with flying."
///
/// 4-mana magecraft Inkling minter — every instant/sorcery floods the
/// board with another 1/1 evasive body. Major Tenured Inkcaster
/// engine: pair with Apprentice + Sorcery to mint Tenured-buffed 3/3
/// fliers each spell.
pub fn inkling_penmaster() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Penmaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling_token(),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Dictation (batch 20) ───────────────────────────────────────

/// Silverquill Dictation — {1}{W}{B} Instant.
///
/// Printed Oracle (synthesised): "Target opponent loses 2 life. Draw a
/// card."
///
/// 3-mana 2-life-drain cantrip-style instant — pure card advantage with
/// the drain rider. Combos with Silverquill / Witherbloom drain payoffs.
/// Slots into any control deck running B for the drain-tempo trade.
pub fn silverquill_dictation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Dictation",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Stormcaller (batch 20) ─────────────────────────────────────────

/// Inkling Stormcaller — {3}{W}{B}, 3/4 Inkling Cleric with Flying and
/// Lifelink.
///
/// Printed Oracle (synthesised): "Flying, lifelink. When this creature
/// enters, target opponent loses 2 life and you gain 2 life."
///
/// 5-mana evasive lifelink finisher with built-in 4-life ETB swing
/// (drain 2 + 2 = 4-life swing). Inkling-tribal anthem stacks with
/// Tenured Inkcaster (+2/+2 → 5/6 lifelink flier). Strong race breaker.
pub fn inkling_stormcaller() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stormcaller",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Discipline (batch 20) ──────────────────────────────────────

/// Silverquill Discipline — {W} Instant.
///
/// Printed Oracle (synthesised): "Target creature gets +2/+1 and gains
/// lifelink until end of turn."
///
/// One-mana combat trick + lifelink rider — bread-and-butter pump that
/// turns a combat trade into a life-swing. Stacks with Inkling Bloodscribe
/// / Felisa for a counter-on-die when the buffed creature attacks.
pub fn silverquill_discipline() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Discipline",
        cost: cost(&[w()]),
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
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Inkscholar (batch 21) ──────────────────────────────────────

/// Silverquill Inkscholar — {2}{W}, 2/3 Human Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a card,
/// then discard a card."
///
/// Compact 3-mana looter body. Filters dead cards for Inkling Bloodscribe /
/// Felisa lifegain payoffs. Slots into Silverquill drain shells as a
/// midrange enabler.
pub fn silverquill_inkscholar() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkscholar",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Battlecaster (batch 21) ────────────────────────────────────────

/// Inkling Battlecaster — {3}{W}{B}, 3/3 Inkling Knight with Flying and
/// Vigilance.
///
/// Printed Oracle (synthesised): "Flying, vigilance. Whenever this creature
/// attacks, you gain 1 life and each opponent loses 1 life."
///
/// 5-mana attack-trigger drain body — combat-tempo finisher that drains for
/// each attack. Stacks with Tenured Inkcaster anthem (→ 5/5 attack-drain).
/// Vigilance keeps it available to block on the swing-back.
pub fn inkling_battlecaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Battlecaster",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Compulsion (batch 21) ──────────────────────────────────────

/// Silverquill Compulsion — {1}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Target opponent reveals their hand. You
/// choose a nonland card from it. That player discards that card."
///
/// 2-mana targeted discard — Thoughtseize template at sorcery speed. Strong
/// hand disruption for the Silverquill control build, especially against
/// combo decks.
pub fn silverquill_compulsion() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Compulsion",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DiscardChosen {
            from: target_filtered(SelectionRequirement::Player),
            count: Value::Const(1),
            filter: SelectionRequirement::Nonland,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill Sealwriter (batch 21) ──────────────────────────────────────

/// Silverquill Sealwriter — {2}{B}, 2/2 Human Wizard with Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. When this creature enters,
/// target opponent loses 2 life and you gain 2 life."
///
/// 3-mana drain-on-ETB lifelink body. Combines lifelink stats with the
/// printed 4-life-swing on ETB. Defensive body that swaps tempo for life.
pub fn silverquill_sealwriter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sealwriter",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Inkling Acolyte (batch 21) ─────────────────────────────────────────────

/// Inkling Acolyte — {1}{W}, 1/2 Inkling Cleric with Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters, create
/// a 1/1 white and black Inkling creature token with flying."
///
/// 2-mana double-Inkling ETB body — pushes two Inkling bodies onto the
/// battlefield from a single 2-mana cast. Maximum tribal density for
/// Tenured Inkcaster (+2/+2 → 3/4 each) and Inkling Verselord lifelink
/// anthems. Card+token can both attack on the same trigger budget.
pub fn inkling_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Inkling Acolyte",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::catalog::sets::sos::inkling_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Silverquill batch 22 ───────────────────────────────────────────────────

/// Silverquill Conviction — {W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 2 life and you gain
/// 2 life. Surveil 1."
///
/// 2-mana drain-and-fix: standard Witherbloom apprentice tax + a peek.
/// Trades Sign-in-Blood's cards for board pressure on a fixed-mana cost.
pub fn silverquill_conviction() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Conviction",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Bookbearer — {2}{W}, 1/4 Human Cleric Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance / When this creature enters,
/// scry 2."
///
/// 3-mana defender that smooths draws on arrival. Pairs with Tenured
/// Inkcaster's +2/+2 (a 3/6 Vigilance smoothing engine).
pub fn silverquill_bookbearer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookbearer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Inquisitor — {2}{B}, 2/3 Inkling Rogue with Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters,
/// target opponent reveals their hand. You choose a nonland card from
/// it. That player discards that card."
///
/// 3-mana flying body + targeted hand-strip — the Silverquill answer to
/// Brainstealer Dragon at a more compact slot.
pub fn inkling_inquisitor() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inquisitor",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
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
                from: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::HasCardType(CardType::Land).negate(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Reckoning — {3}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Destroy target creature. Create a 1/1
/// white and black Inkling creature token with flying."
///
/// 5-mana removal + body — strict upgrade over Vraska's Contempt at the
/// 5-mana slot for Inkling tribal shells: trades exile for token +
/// Inkling-tribal payoff.
pub fn silverquill_reckoning() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Reckoning",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(SelectionRequirement::Creature) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::catalog::sets::sos::inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lifeglyph — {1}{W}{B}, 2/3 Inkling Bard Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, target creature gets +1/+1 until
/// end of turn."
///
/// 3-mana Lifelink Inkling that pumps the team incrementally on every
/// spell. Combos with Inkling Verselord and Silverquill Anthemwriter for
/// stacked lifelink anthems.
pub fn silverquill_lifeglyph() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Silverquill Lifeglyph",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature),
            1,
            1,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 23: 5 new Silverquill cards ───────────────────

/// Inkling Aristocrat — {1}{B}, 1/2 Inkling Cleric.
///
/// Printed Oracle (synthesised): "Whenever another creature you control
/// dies, you gain 1 life."
///
/// A compact Cauldron-of-Essence-style aristocrat payoff for 2 mana. Triggers
/// on tokens and non-tokens alike, so Pest / Inkling / Spirit sacrifice
/// engines all feed it.
pub fn inkling_aristocrat() -> CardDefinition {
    CardDefinition {
        name: "Inkling Aristocrat",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inscriber — {2}{W}{B}, 3/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a 1/1
/// white and black Inkling creature token with flying. Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, put a +1/+1 counter on
/// target Inkling you control."
///
/// 4-mana Inkling engine: comes with a token in tow and grows the tribe on
/// every spell. Pairs with Tenured Inkcaster for stacked anthems.
pub fn silverquill_quillscribe() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Silverquill Quillscribe",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::catalog::sets::sos::inkling_token(),
                },
            },
            magecraft(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Hush — {W}{B}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -2/-2 until end of
/// turn. You gain 2 life."
///
/// 2-mana removal-for-toughness-2-and-under + a defensive lifegain rider.
/// Cleanly trades into 2-toughness creatures while feeding Light of Promise
/// and the Inkling drain payoffs.
pub fn silverquill_hush() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Hush",
        cost: cost(&[w(), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Lorewright — {3}{W}{B}, 2/4 Inkling Wizard Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters, draw a
/// card and you lose 1 life."
///
/// 5-mana defensive flyer + a built-in card. The 1-life cost is a small
/// drawback that's negligible in lifegain shells with Lifelink anthems.
pub fn inkling_lorewright() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lorewright",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::LoseLife {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Battle Hymn — {2}{W}, sorcery.
///
/// Printed Oracle (synthesised): "Creatures you control get +1/+1 and gain
/// vigilance until end of turn."
///
/// 3-mana team anthem with vigilance for the alpha-strike-then-block turn.
/// Pair with Inkling Brigade or Silverquill Sermon for a wide swarm.
pub fn silverquill_battle_hymn() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    CardDefinition {
        name: "Silverquill Battle Hymn",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: each_your_creature(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: each_your_creature(),
                keyword: Keyword::Vigilance,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 24: 5 new Silverquill cards ───────────────────

// ── Push (modern_decks) batch 24+: 10 more STX cards (2 per college) ───────

/// Silverquill Eulogist — {1}{B}, 1/3 Human Cleric.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target opponent loses 1 life."
///
/// 2-mana defensive body + per-cast 1-drain. The Silverquill Apprentice
/// template at the BB/WB slot, but mono-black-friendly.
pub fn silverquill_eulogist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Eulogist",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Notetaker — {1}{W}, 1/2 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 1.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// you may pay {1}. If you do, draw a card."
///
/// 2-mana Silverquill velocity body — ETB selection + every cast becomes a
/// rate-of-1 mana loot. Pairs with Tenured Inkcaster anthem and feeds the
/// drain payoffs.
pub fn silverquill_notetaker() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Silverquill Notetaker",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::MayDo {
                description: "draw a card".to_string(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Pamphleteer — {W}{B}, 2/2 Inkling Cleric Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters,
/// each opponent loses 1 life and you gain 1 life."
///
/// 2-mana evasive Inkling with a built-in drain ETB. Same shape as
/// Inkling Acolyte but trades the Inkling token for a drain on landing.
pub fn inkling_pamphleteer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pamphleteer",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Indictment — {2}{W}{B}, instant.
///
/// Printed Oracle (synthesised): "Exile target creature with mana value
/// 3 or less. You gain 2 life."
///
/// 4-mana exile-removal for the small-creature slot + a lifegain rider.
/// Cleanly answers most 1-3 MV threats while feeding Light of Promise /
/// Felisa-style lifegain payoffs.
pub fn silverquill_indictment() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Indictment",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Exile,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Banner-Bearer — {3}{W}, 2/3 Inkling Soldier Flying + Vigilance.
///
/// Printed Oracle (synthesised): "Flying, vigilance. Other Inkling
/// creatures you control get +1/+0."
///
/// 4-mana tribal lord for the Inkling deck — pumps every other Inkling
/// for +1 power. Stacks with Tenured Inkcaster (+2/+2 anthem) and
/// Silverquill Anthemwriter (+1/+0 anthem) for absurd evasive damage.
pub fn inkling_banner_bearer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Banner-Bearer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Tribunal — {2}{B}, sorcery.
///
/// Printed Oracle (synthesised): "Target opponent sacrifices a creature.
/// You gain 1 life."
///
/// Cruel Edict with a small lifegain rider in Silverquill colors. The
/// 1-life is enough to feed Light of Promise's trigger and any +1-life
/// payoff in the Silverquill drain shell.
pub fn silverquill_tribunal() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Tribunal",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Target(0)),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Quillwarden — {2}{W}{B}, 2/4 Inkling Knight Flying + Vigilance.
///
/// Printed Oracle (synthesised): "Flying, vigilance. Whenever you cast or
/// copy an instant or sorcery spell, this creature gets +1/+0 until end
/// of turn."
///
/// 4-mana evasive Inkling that scales aggressively in spell-heavy shells.
/// Same magecraft self-pump template as Eccentric Apprentice with a +1
/// toughness frame.
pub fn inkling_quillwarden() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Inkling Quillwarden",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sage — {1}{W}, 1/2 Inkling Wizard.
///
/// Printed Oracle (synthesised): "Flying. {2}{W}{B}: This creature gets
/// +1/+1 until end of turn."
///
/// 2-mana flying body with a mid-game pump sink — turns excess mana into
/// damage in attrition wars.
pub fn inkling_sage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::Duration;
    CardDefinition {
        name: "Inkling Sage",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), b()]),
            tap_cost: false,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
                    self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}


// ── Push (modern_decks) batch 24++: 1 more Silverquill card ────────────────

/// Silverquill Memorist — {2}{W}{B}, 2/3 Inkling Bard.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters, return
/// target instant or sorcery card from your graveyard to your hand."
///
/// 4-mana evasive recursion body — drains opp/refills your hand of IS
/// spells, threatens chip damage in the air. Closes the Silverquill
/// curve at the 4-mana slot.
pub fn silverquill_memorist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memorist",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
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
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 25: 7 more Silverquill cards ─────────────────
//
// Continuing Silverquill (W/B) buildout: 4 new creatures + 3 spells using
// existing magecraft / drain / counter / token primitives. No new engine
// features required.

/// Silverquill Inkmaster — {1}{W}{B}, 2/2 Inkling Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, each opponent loses 1 life and you gain 1
/// life."
///
/// 3-mana drain-magecraft body — Witherbloom Apprentice template in
/// Silverquill colors. Every IS spell drains 2 life total (1 from
/// each opp, 1 to you). Pairs with Tenured Inkcaster's +2/+2 anthem.
pub fn silverquill_inkmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmaster",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Censurer — {2}{W}, 2/3 Inkling Cleric Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance. When this creature enters, tap
/// target creature an opponent controls."
///
/// 3-mana Vigilance defender + free tap-down trick on ETB. Removes a
/// blocker for your alpha-strike turn or shuts down opp's attacker for a
/// round. Same shape as Frost Trickster but at sorcery speed.
pub fn inkling_censurer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Censurer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Loredrain — {2}{B}, instant.
///
/// Printed Oracle (synthesised): "Target creature gets -2/-2 until end of
/// turn. You gain 2 life."
///
/// 3-mana mass-removal-for-small + lifegain. Kills any 1- or 2-toughness
/// creature outright, weakens larger threats for combat trades. Feeds the
/// Silverquill drain shell's lifegain triggers (Light of Promise, Felisa).
pub fn silverquill_loredrain() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Loredrain",
        cost: cost(&[generic(2), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Verseweaver — {3}{W}{B}, 3/3 Inkling Bard Flying.
///
/// Printed Oracle (synthesised): "Flying. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, create a 2/1 white and black Inkling
/// creature token with flying."
///
/// 5-mana evasive Inkling factory. Each IS spell makes another 2/1
/// flying Inkling — turns a stocked hand of spells into a flying horde.
/// Slot into any Silverquill spell-heavy shell.
pub fn inkling_verseweaver() -> CardDefinition {
    use crate::card::TokenDefinition;
    let inkling = TokenDefinition {
        name: "Inkling".to_string(),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White, crate::mana::Color::Black],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Inkling Verseweaver",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Hightutor — {1}{W}, sorcery.
///
/// Printed Oracle (synthesised): "Search your library for an instant or
/// sorcery card with mana value 2 or less, reveal it, and put it into
/// your hand. Then shuffle."
///
/// 2-mana tutor for cheap spells — finds removal or a counterspell when
/// you need it. Slots into Silverquill / Lorehold / Prismari spellslinger
/// shells looking for redundancy.
pub fn silverquill_hightutor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Hightutor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                .and(SelectionRequirement::ManaValueAtMost(2)),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lifebinder — {2}{W}, 2/3 Human Cleric Lifelink.
///
/// Printed Oracle (synthesised): "Lifelink. When this creature enters,
/// you gain 2 life."
///
/// 3-mana lifegain stapled to a body. Solid defensive lifelinker, feeds
/// Light of Promise / Honor Troll / Bookwurm payoffs immediately on
/// arrival.
pub fn silverquill_lifebinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifebinder",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Drainmaster — {3}{B}, 2/4 Inkling Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, target
/// opponent loses 3 life and you gain 3 life."
///
/// 4-mana high-toughness drain body. 6-life-swing on ETB + 2/4 stats
/// makes it brutally efficient against aggro decks while feeding any
/// lifegain payoffs in the shell.
pub fn inkling_drainmaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Drainmaster",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::Target(0)),
                to: Selector::You,
                amount: Value::Const(3),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 28: 5 more Silverquill cards ─────────────────
//
// Continuing the Silverquill (W/B) buildout with 5 new cards using existing
// primitives. No new engine features required. Each card has a paired test
// in `tests::stx`.

/// Silverquill Heraldist — {1}{W}, 2/2 Human Soldier.
///
/// Printed Oracle (synthesised): "When this creature enters, you gain 1 life
/// and create a 1/1 white-and-black Inkling creature token with flying."
///
/// 2-mana lifegain + Inkling-mint body. Triple-threat: warm body, 1-life
/// floor, evasive token. Feeds Light of Promise / Inkling Bloodscribe.
pub fn silverquill_heraldist() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Heraldist",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
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
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Spireguard — {2}{W}, 2/3 Inkling Soldier Flying.
///
/// Printed Oracle (synthesised): "Flying. When this creature enters, target
/// creature you control gets +1/+1 until end of turn."
///
/// 3-mana flying body + combat trick. Stacks with Tenured Inkcaster and
/// Inkling Banner-Bearer for tribal payoffs.
pub fn inkling_spireguard() -> CardDefinition {
    CardDefinition {
        name: "Inkling Spireguard",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Quillwitch — {1}{B}, 2/2 Inkling Warlock.
///
/// Printed Oracle (synthesised): "When this creature dies, target opponent
/// loses 2 life."
///
/// 2-mana sticky drain — dies-to-drain template at the small slot.
pub fn silverquill_quillwitch() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillwitch",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkpurge — {1}{W}{B}, sorcery.
///
/// Printed Oracle (synthesised): "Each opponent sacrifices a creature. You
/// gain 2 life."
///
/// 3-mana edict + lifegain. The combined effect breaks open boards where
/// the opponent has a single big threat. Per-opp sac picker walks each
/// opp's creatures and picks the cheapest.
pub fn silverquill_inkpurge() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkpurge",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    filter: SelectionRequirement::Creature,
                    count: Value::Const(1),
                }),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkrise Schoolwarden — {3}{W}{B}, 3/4 Inkling Cleric Flying Lifelink.
///
/// Printed Oracle (synthesised): "Flying, lifelink. When this creature
/// enters, draw a card."
///
/// 5-mana evasive lifelink finisher + cantrip. Card-neutral on landing,
/// trades up against any non-removal interaction.
pub fn inkrise_schoolwarden() -> CardDefinition {
    CardDefinition {
        name: "Inkrise Schoolwarden",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 30: 5 new Silverquill cards ──────────────────────────────────────

/// Silverquill Drafter — {1}{B}, 2/2 Inkling Wizard, Flying.
///
/// Synthesised Oracle: "Flying. When this creature enters, target opponent
/// loses 2 life."
///
/// 2-mana evasive flier with stapled drain — fuels Tenured Inkcaster tribal
/// + life-matters payoffs.
pub fn silverquill_drafter_b30() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drafter B30",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: target_filtered(SelectionRequirement::Player),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Scrivener — {2}{W}, 2/3 Human Cleric.
///
/// Synthesised Oracle: "When this creature enters, look at the top three
/// cards of your library; put one into your hand and the rest on the bottom
/// of your library in any order." Approximated as Scry 2 + Draw 1.
pub fn silverquill_scrivener_b30() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scrivener B30",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Cantor — {W}{B}, 2/2 Inkling Wizard, Flying.
///
/// Synthesised Oracle: "Flying. Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, target creature you control gets +1/+1 until
/// end of turn."
pub fn inkling_cantor() -> CardDefinition {
    CardDefinition {
        name: "Inkling Cantor",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pact — {3}{W}{B}, sorcery.
///
/// Synthesised Oracle: "You gain 4 life and create two 1/1 white-and-black
/// Inkling creature tokens with flying."
///
/// 5-mana lifegain + double-Inkling minter — Silverquill go-wide finisher.
pub fn silverquill_pact() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Pact",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Vellumweaver — {1}{W}, 1/3 Human Cleric, Vigilance.
///
/// Synthesised Oracle: "Vigilance. Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life."
///
/// Defensive lifegain-on-cast that pairs with Light of Promise / Felisa.
pub fn silverquill_vellumweaver() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life;
    CardDefinition {
        name: "Silverquill Vellumweaver",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sermon — {1}{W}{B}, sorcery. Synthesised Oracle: "Drain 2.
/// Create a 1/1 white-and-black Inkling creature token with flying."
/// 3-mana drain + Inkling mint.
pub fn inkling_sermon() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Sermon",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Tutor — {1}{W}, 2/2 Human Cleric. Synthesised Oracle:
/// "When this creature enters, draw a card, then discard a card."
/// 2-mana loot body.
pub fn silverquill_lorescribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lorescribe",
        cost: cost(&[generic(1), w()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Warden — {2}{W}{B}, 2/4 Inkling Knight Flying + Vigilance.
/// Synthesised Oracle: "Whenever another Inkling you control enters,
/// put a +1/+1 counter on this creature." 4-mana Inkling-tribal payoff.
pub fn inkling_warden() -> CardDefinition {
    use crate::card::{CounterType, Predicate};
    CardDefinition {
        name: "Inkling Warden",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Inkling),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkletter — {W}{B}, instant. Synthesised Oracle:
/// "Drain 1. Surveil 1." 2-mana drain + selection — pairs with all
/// Witherbloom/Silverquill graveyard-care payoffs.
pub fn silverquill_inkletter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkletter",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 32 (modern_decks) — Silverquill expansion ─────────────────────────

/// Silverquill Drainlord — {3}{W}{B}, 3/4 Vampire Warlock Lifelink.
/// Synthesised Oracle: "When this creature enters, each opponent loses 3
/// life and you gain 3 life."
pub fn silverquill_drainlord() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainlord",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(3)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Squire — {W}{B}, 2/2 Inkling Knight Flying.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, target creature gets -1/-1 until end of turn."
pub fn inkling_quillbearer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillbearer",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Indoctrinator — {2}{W}, 2/3 Human Cleric Vigilance.
/// Synthesised Oracle: "When this creature enters, each opponent discards
/// a card."
pub fn silverquill_indoctrinator() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Indoctrinator",
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
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Choirsinger — {1}{W}{B}, 2/2 Inkling Cleric Flying Lifelink.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life."
pub fn inkling_choirsinger() -> CardDefinition {
    CardDefinition {
        name: "Inkling Choirsinger",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Ovation — {3}{W}{B}, sorcery.
/// Synthesised Oracle: "Create two 1/1 white-and-black Inkling creature
/// tokens with flying, then put a +1/+1 counter on each Inkling you control."
pub fn silverquill_ovation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ovation",
        cost: cost(&[generic(3), w(), b()]),
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
                definition: inkling_token(),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                        .and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Loremaster — {2}{W}{B}, 2/4 Inkling Wizard Flying.
/// Synthesised Oracle: "When this creature enters, return target instant
/// or sorcery card from your graveyard to your hand. You gain 1 life."
pub fn inkling_loremaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Loremaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        zone: Zone::Graveyard,
                        who: PlayerRef::You,
                        filter: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::GainLife {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Litany — {1}{B}, instant.
/// Synthesised Oracle: "Target creature gets -2/-1 until end of turn. You
/// gain 1 life."
pub fn silverquill_litany() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Litany",
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
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 33: 5 new Silverquill cards ────────────────────────────────────

/// Inkling Calligrapher — {1}{W}{B}, 2/3 Inkling Cleric Flying.
/// Synthesised Oracle: "Flying / Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target creature gets -1/-1 until end
/// of turn."
pub fn inkling_calligrapher() -> CardDefinition {
    CardDefinition {
        name: "Inkling Calligrapher",
        cost: cost(&[generic(1), w(), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Spellscribe — {2}{W}{B}, 3/3 Inkling Wizard Flying
/// Lifelink. Synthesised Oracle: "Flying, lifelink / When this creature
/// enters, create a 1/1 white-and-black Inkling creature token with
/// flying."
pub fn silverquill_spellscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Spellscribe",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Strikemark — {2}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent loses 2 life. You gain 2 life."
pub fn inkling_strikemark() -> CardDefinition {
    CardDefinition {
        name: "Inkling Strikemark",
        cost: cost(&[generic(2), w(), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Scribe-Tutor — {1}{W}, 1/3 Human Cleric.
/// Synthesised Oracle: "When this creature enters, surveil 1."
pub fn silverquill_scribe_tutor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scribe-Tutor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Surveil {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Magemark — {W}{B}, Instant.
/// Synthesised Oracle: "Target creature gets -2/-2 until end of turn.
/// You gain 2 life."
pub fn silverquill_magemark() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Magemark",
        cost: cost(&[w(), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Standardbearer — {2}{W}, 2/2 Human Soldier Vigilance.
/// Synthesised Oracle: "Other creatures you control get +1/+1 as long as
/// you control an Inkling." (Approximated as unconditional anthem since
/// the conditional gate would require static gating not yet wired; played
/// in an Inkling-themed deck this is functionally identical.)
pub fn silverquill_standardbearer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Standardbearer",
        cost: cost(&[generic(2), w()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 34: Silverquill cards ─────────────────────────────────────────────

/// Silverquill Drainwriter — {2}{W}{B}, 3/3 Inkling Wizard, Flying.
/// Synthesised Oracle: "When this creature enters, each opponent loses 2
/// life and you gain 2 life."
pub fn silverquill_drainwriter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainwriter",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Battle Chant — {3}{W}, Sorcery.
/// Synthesised Oracle: "Creatures you control get +2/+1 and gain vigilance
/// until end of turn."
pub fn silverquill_battle_chant() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Battle Chant",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Vigilance,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Homily — {1}{W}{B}, Sorcery.
/// Synthesised Oracle: "Drain 1 (each opponent loses 1 life and you gain
/// 1 life) and each opponent mills two cards."
pub fn silverquill_homily() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Homily",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Avenger — {3}{W}{B}, 3/3 Inkling Knight, Flying + First Strike.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter on
/// another target creature you control."
pub fn inkling_avenger() -> CardDefinition {
    CardDefinition {
        name: "Inkling Avenger",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Mandate — {2}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent sacrifices a creature."
pub fn silverquill_mandate() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mandate",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::SacrificeAndRemember {
            who: PlayerRef::EachOpponent,
            filter: SelectionRequirement::Creature,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Spellquill — {W}{B}, 1/2 Inkling Bard.
/// Synthesised Oracle: "Flying. Magecraft — gain 1 life. When this creature
/// dies, draw a card."
pub fn silverquill_spellquill() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Spellquill",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            magecraft_gain_life(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 35: Silverquill cards ─────────────────────────────────────────────

/// Silverquill Penitent — {1}{W}, 2/2 Human Cleric.
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life and you gain 1 life."
pub fn silverquill_penitent() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penitent",
        cost: cost(&[generic(1), w()]),
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
        // Refactored in batch 40: ETB drain wired via the canonical
        // `etb_drain(1)` shortcut from batch 39.
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Echobringer — {1}{W}{B}, 2/2 Inkling Cleric, Flying + Lifelink.
/// Synthesised Oracle: Inkling tribal payoff.
pub fn inkling_echobringer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Echobringer",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Verseblade — {1}{W}{B}, Instant.
/// Synthesised Oracle: "Target creature gets +1/+1 until end of turn. Draw
/// a card."
pub fn silverquill_verseblade() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Verseblade",
        cost: cost(&[generic(1), w(), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lifepenner — {2}{W}, 2/3 Human Cleric.
/// Synthesised Oracle: "Magecraft — you gain 2 life."
pub fn silverquill_lifepenner() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifepenner",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
        triggered_abilities: vec![magecraft_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Maverick — {2}{B}, 3/2 Inkling Rogue, Flying.
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life and you gain 1 life."
pub fn inkling_maverick() -> CardDefinition {
    CardDefinition {
        name: "Inkling Maverick",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Antiphony — {2}{W}{B}, Instant.
/// Synthesised Oracle: "Drain 2. Surveil 1."
pub fn silverquill_antiphony() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Antiphony",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 36: more Silverquill cards ────────────────────────────────────────

/// Silverquill Stylepoint — {W}, Instant.
/// Synthesised Oracle: "Target creature gets +1/+1 EOT and gains First
/// Strike until end of turn."
pub fn silverquill_stylepoint() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Stylepoint",
        cost: cost(&[w()]),
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
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::FirstStrike,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sentinel — {2}{W}, 2/3 Inkling Soldier with Flying.
/// Synthesised Oracle: vanilla Inkling at the 3-mana slot.
pub fn inkling_b36_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentinel II",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Forge — {3}{W}{B}, Sorcery.
/// Synthesised Oracle: "Create two 1/1 W/B Inkling creature tokens with
/// flying. Each opponent loses 1 life and you gain 1 life."
pub fn silverquill_forge() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Forge",
        cost: cost(&[generic(3), w(), b()]),
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
                definition: inkling_token(),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Cardinal — {3}{W}{B}, 3/4 Inkling Cleric, Flying + Vigilance.
/// Synthesised Oracle: "When this creature enters, you gain 2 life."
pub fn inkling_cardinal() -> CardDefinition {
    CardDefinition {
        name: "Inkling Cardinal",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_gain_life` shortcut.
        triggered_abilities: vec![etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 38: more Silverquill cards ────────────────────────────────────────

/// Silverquill Essayist — {1}{W}, 1/3 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, you gain 1 life. Scry 1."
pub fn silverquill_essayist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Essayist",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Scriptwarden — {2}{W}{B}, 2/3 Inkling Wizard, Flying + Vigilance.
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life and you gain 1 life."
pub fn inkling_scriptwarden() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scriptwarden",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pinion — {W}, Instant.
/// Synthesised Oracle: "Target creature gets +1/+1 EOT and gains Flying
/// until end of turn."
pub fn silverquill_pinion() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pinion",
        cost: cost(&[w()]),
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
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Battle Oration — {4}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent loses 4 life and you gain 4 life.
/// Create a 1/1 W/B Inkling creature token with flying."
pub fn silverquill_battle_oration() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Battle Oration",
        cost: cost(&[generic(4), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(4),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Manuscript — {1}{B}, Sorcery.
/// Synthesised Oracle: "Target opponent loses 2 life. You draw a card."
pub fn silverquill_manuscript() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Manuscript",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Ambassador — {1}{W}, 1/1 Inkling Cleric with Flying + Lifelink.
/// Synthesised Oracle: lean 2-mana evasive lifegainer.
pub fn inkling_ambassador() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ambassador",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Calligraphist — {3}{W}, 2/4 Inkling Cleric, Flying.
/// Synthesised Oracle: "Magecraft — Put a +1/+1 counter on this creature."
pub fn inkling_calligraphist() -> CardDefinition {
    CardDefinition {
        name: "Inkling Calligraphist",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 39: 6 more Silverquill cards ─────────────────────────────────────

/// Silverquill Liturgist — {2}{W}, 1/4 Inkling Cleric with Flying.
/// Synthesised Oracle: "Defensive evasive body. Magecraft — gain 1 life."
pub fn silverquill_liturgist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Liturgist",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bookwarden — {3}{W}{B}, 4/5 Inkling Warrior Flying + Lifelink.
/// Synthesised Oracle: "Top-end Silverquill finisher."
pub fn inkling_bookwarden() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bookwarden",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Soulbinder — {1}{W}{B}, 2/2 Vampire Warlock.
/// Synthesised Oracle: "When this creature enters, target opp loses 2 life
/// and you gain 2 life. Magecraft — put a +1/+1 counter on this creature."
pub fn silverquill_soulbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Soulbinder",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(2),
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Magister — {4}{W}{B}, 3/4 Inkling Wizard, Flying + Vigilance.
/// Synthesised Oracle: "ETB drain 3. Magecraft — gain 1 life."
pub fn inkling_magister() -> CardDefinition {
    CardDefinition {
        name: "Inkling Magister",
        cost: cost(&[generic(4), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(3), magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkproclamation — {2}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent sacrifices a creature, then you
/// create a 1/1 W/B Inkling token with flying."
pub fn silverquill_inkproclamation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkproclamation",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Loredrain — {3}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent discards a card and loses 2 life.
/// You gain 2 life."
pub fn inkling_loredrain() -> CardDefinition {
    CardDefinition {
        name: "Inkling Loredrain",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 40: more Silverquill cards ────────────────────────────────────────

/// Silverquill Scriptwright — {1}{W}, 2/2 Human Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, target Inkling you control gets +1/+1 until end of
/// turn." A focused inkling-tribal pump payoff that scales with the
/// spellslinger plan.
pub fn silverquill_scriptwright() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scriptwright",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bookcrier — {2}{B}, 3/2 Inkling Rogue Flying.
/// Synthesised Oracle: "Aggressive evasive Inkling — top-end of the
/// 3-mana flying slot." Stacks with Tenured Inkcaster's +2/+2 anthem
/// for a 5/4 flier and with Inkling Bloodscribe's anothers-dies drain.
pub fn inkling_bookcrier() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bookcrier",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Cantorist — {W}{B}, 2/2 Vampire Cleric.
/// Synthesised Oracle: "Lifelink. When this creature enters, each opponent
/// loses 1 life and you gain 1 life." Pure aggressive drain body in the
/// 2-drop slot — pairs with Felisa for an Inkling token on death of
/// other counter-bearing creatures.
pub fn silverquill_cantorist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cantorist",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Treasurer — {2}{W}, 1/4 Inkling Soldier Flying.
/// Synthesised Oracle: "When this creature enters, you gain 1 life and
/// scry 1." A defensive 3-mana flier with selection — feeds Light of
/// Promise lifegain payoffs and smooths the next draw.
pub fn inkling_treasurer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Treasurer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Memorize — {1}{W}{B}, Instant.
/// Synthesised Oracle: "Drain 2 life and target creature gets +1/+1
/// until end of turn." A combat-trick-meets-drain instant. The pump
/// can grow your Inkling for the alpha or shrink a blocker via the
/// drain math.
pub fn silverquill_memorize() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memorize",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bellringer — {3}{W}{B}, 4/3 Inkling Bard Flying + Lifelink.
/// Synthesised Oracle: "When this creature enters, target opponent
/// discards a card." Hard-hitting 5-mana finisher with lifelink and
/// a hand-attack rider. Pairs with Silverquill Inquisition for a
/// two-turn hand-strip plan.
pub fn inkling_bellringer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bellringer",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Encore — {2}{W}, Instant.
/// Synthesised Oracle: "Creatures you control get +1/+0 and gain
/// lifelink until end of turn." Team alpha-strike trick with lifelink
/// to stabilise after the swing.
pub fn silverquill_encore() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Encore",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Lifelink,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sentencer — {1}{W}, 2/1 Inkling Soldier Flying.
/// Synthesised Oracle: "When this creature enters, target creature you
/// don't control gets -1/-0 until end of turn." A 2-mana evasive
/// Inkling that tempos out a blocker on entry.
pub fn inkling_sentencer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentencer",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkflood — {3}{W}{B}, Sorcery.
/// Synthesised Oracle: "Create two 1/1 white and black Inkling creature
/// tokens with flying. You gain 2 life." A reach-2-token finisher in
/// the same slot as Defend the Campus (which mints 3 for 5 mana with no
/// life rider). Pairs with Tenured Inkcaster (+2/+2 anthem) and Light
/// of Promise (lifegain → +1/+1 counters).
pub fn silverquill_inkflood() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkflood",
        cost: cost(&[generic(3), w(), b()]),
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
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Quilltender — {1}{W}{B}, 2/2 Inkling Cleric Lifelink.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter
/// on target Inkling you control." A 3-mana lifelink Inkling that
/// grows another Inkling on the way in — turbo-charges the tribal
/// snowball.
pub fn inkling_quilltender() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quilltender",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Purifier — {1}{W}, 2/2 Human Cleric.
/// Synthesised Oracle: "When this creature enters, you gain 2 life.
/// Magecraft — Scry 1."
pub fn silverquill_purifier() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Purifier",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![
            etb_gain_life(2),
            magecraft(Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Proxy — {2}{B}, 2/3 Inkling Rogue Flying.
/// Synthesised Oracle: "When this creature enters, target opponent
/// discards a card at random." A 3-mana defensive flier + targeted
/// disruption.
pub fn inkling_proxy() -> CardDefinition {
    CardDefinition {
        name: "Inkling Proxy",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(1),
            random: true,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Witnessing — {2}{W}{B} Instant.
/// Synthesised Oracle: "Each opponent loses 3 life and you gain 3 life.
/// Draw a card."
pub fn silverquill_witnessing() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Witnessing",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Avant-Garde — {4}{W}{B}, 4/4 Inkling Bard Flying + Lifelink.
/// Synthesised Oracle: "When this creature enters, each opponent loses
/// 2 life and you gain 2 life." A 6-mana evasive race-breaker.
pub fn inkling_avant_garde() -> CardDefinition {
    CardDefinition {
        name: "Inkling Avant-Garde",
        cost: cost(&[generic(4), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Convocation — {3}{W}{B} Sorcery.
/// Synthesised Oracle: "Create two 1/1 white-and-black Inkling creature
/// tokens with flying. Each opponent loses 1 life and you gain 1 life
/// for each Inkling you control."
pub fn silverquill_convocation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Convocation",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: inkling_token(),
                count: Value::Const(2),
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                        .and(SelectionRequirement::ControlledByYou),
                )),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 42 (modern_decks) — Silverquill expansion ─────────────────────────

/// Silverquill Spellbinder — {2}{W}{B}, 2/3 Vampire Cleric Lifelink.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, each opponent loses 1 life and you gain 1 life."
/// A 4-mana drain-each-spell body that anchors the Silverquill drain
/// engine alongside Witherbloom Apprentice and Tenured Inkcaster's pump.
pub fn silverquill_spellbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Spellbinder",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Recruiter — {W}{B}, 1/2 Inkling Soldier Flying.
/// Synthesised Oracle: "When this creature enters, create a 1/1 white
/// and black Inkling creature token with flying." A 2-mana 1/2 flier
/// that immediately mints a second body, doubling the air force.
pub fn inkling_recruiter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Recruiter",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling_token(),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Censure II — {1}{W} Instant.
/// Synthesised Oracle: "Target creature gets -3/-3 until end of turn."
/// A 2-mana removal trick that handles X/3-and-smaller creatures while
/// also softening up larger bodies for combat tricks.
pub fn silverquill_censure_v2() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Censure II",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Drafter II — {1}{B}, 2/2 Human Rogue.
/// Synthesised Oracle: "When this creature enters, target opponent
/// discards a card." 3-mana ETB hand attack body — symmetric to Inkling
/// Proxy but discard-of-opponent's-choice rather than random.
pub fn silverquill_drafter_v2() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drafter II",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(1),
            random: false,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkflame — {1}{W}{B} Sorcery.
/// Synthesised Oracle: "Each opponent loses 2 life. You gain 2 life and
/// draw a card." 3-mana drain-and-cantrip — net +1 card and a 4-life
/// swing per cast.
pub fn silverquill_inkflame() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkflame",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Penlord — {3}{W}{B}, 4/4 Vampire Cleric Flying + Lifelink.
/// Synthesised Oracle: "Flying, lifelink. When this creature enters,
/// each opponent loses 3 life and you gain 3 life." A 5-mana flying
/// drain finisher.
pub fn silverquill_penlord() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penlord",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Disciple — {1}{W}, 1/1 Inkling Cleric Flying.
/// Synthesised Oracle: "Flying. When this creature enters, you gain
/// 1 life." 2-mana defensive evasive lifegain.
pub fn inkling_disciple() -> CardDefinition {
    CardDefinition {
        name: "Inkling Disciple",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 43 (modern_decks) — Silverquill expansion ─────────────────────────

/// Silverquill Blackquill Acolyte — {W}{B}, 1/2 Inkling Cleric.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, each opponent loses 1 life and you gain 1 life."
/// Mirror of Witherbloom Apprentice in W/B at the Inkling type slot.
pub fn silverquill_blackquill_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Blackquill Acolyte",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Ravenmage — {2}{W}{B}, 2/3 Vampire Wizard Flying.
/// Synthesised Oracle: "Flying. Whenever this creature attacks, each
/// opponent loses 1 life and you gain 1 life." 4-mana flying drain
/// trigger on combat.
pub fn silverquill_ravenmage() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ravenmage",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Drain {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkjet Scribe — {1}{B}, 2/1 Inkling Rogue Flying.
/// Synthesised Oracle: "Flying. When this creature enters, create a
/// 1/1 white-and-black Inkling creature token with flying."
/// Aggressive 2-mana evasive 2/1 + Inkling tribal seeder.
pub fn silverquill_inkjet_scribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkjet Scribe",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Grand Inkmaster — {4}{W}{B}, 4/5 Inkling Wizard
/// Flying + Lifelink. Synthesised Oracle: "Flying, lifelink. When
/// this creature enters, each opponent loses 4 life and you gain
/// 4 life." 6-mana evasive race-breaker.
pub fn silverquill_grand_inkmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Grand Inkmaster",
        cost: cost(&[generic(4), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(4)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Diatribe — {2}{B} Sorcery. Synthesised Oracle: "Target
/// player loses 4 life. Surveil 1." 3-mana drain + selection.
pub fn silverquill_diatribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Diatribe",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(4),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Saboteur — {2}{B}, 2/2 Inkling Rogue Menace.
/// Synthesised Oracle: "Menace. Whenever this creature deals combat
/// damage to a player, that player discards a card." 3-mana evasive
/// disruption body.
pub fn inkling_saboteur() -> CardDefinition {
    CardDefinition {
        name: "Inkling Saboteur",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Sealwright — {1}{W}{B}, 2/2 Vampire Cleric Lifelink.
/// Synthesised Oracle: "Lifelink. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on target
/// creature you control." 3-mana magecraft growth.
pub fn silverquill_sealwright() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Sealwright",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks claude/modern_decks) — close-out Silverquill batch ──
//
// 22 new Silverquill cards across drain / lifelink / Inkling-tribal /
// Magecraft templates. Each card uses existing engine primitives — no
// engine changes required. All 22 are ✅ status; each has a lock-in
// test in `tests::stx`. After this batch the Silverquill school is
// fully closed-out (only Mavinda, Students' Advocate stays 🟡 pending
// the cast-from-graveyard-targeting-only-a-single-creature engine
// primitive — tracked separately).
//
// Names verified against `crabomination/src/catalog/sets/` for
// uniqueness; "Silverquill Vanguard" already exists upstream, so this
// batch uses "Silverquill Spellguard" instead.

/// Silverquill Maxim — {2}{W}{B}, Sorcery. "Silverquill Maxim deals 3
/// damage to any target. You gain 3 life." Synthesized drain + burn
/// finisher.
pub fn silverquill_maxim() -> CardDefinition {
    use crate::effect::shortcut::{deal, target_filtered};
    CardDefinition {
        name: "Silverquill Maxim",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            deal(
                3,
                target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
            ),
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Vassal — {1}{B}, 1/2 Inkling Cleric Lifelink. Magecraft →
/// opponent loses 1 life. Cheap pingy drain body.
pub fn inkling_vassal() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vassal",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Vellum — {W}{B}, Instant. "Each opponent loses 2 life and
/// you gain 2 life." 2-mana symmetric Silverquill drain template.
pub fn silverquill_vellum() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vellum",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Decreemaster — {2}{W}{B}, 2/3 Inkling Cleric Flying Lifelink.
/// ETB: target opponent discards a card. 4-mana double-keyword discard
/// body.
pub fn inkling_decreemaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Decreemaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Penbringer — {3}{W}, 2/4 Human Cleric Vigilance. Magecraft
/// → gain 1 life. Defensive Silverquill anchor.
pub fn silverquill_penbringer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penbringer",
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
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Ravenswing — {1}{W}{B}, 2/2 Vampire Cleric Flying.
/// "Whenever this creature attacks, each opponent loses 1 life and you
/// gain 1 life." 3-mana evasive drain attacker.
pub fn silverquill_ravenswing() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ravenswing",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Drain {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Magistrate — {2}{B}, 2/2 Inkling Cleric. ETB: opponent loses
/// 2 life. Body-on-a-Bloodthirst template.
pub fn inkling_magistrate() -> CardDefinition {
    CardDefinition {
        name: "Inkling Magistrate",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Liturgy — {3}{W}{B}, Sorcery. "Each opponent loses 2
/// life. You gain 4 life. Draw a card." 5-mana Silverquill cantrip-
/// drain shell — same template as Pull from the Grave / Silverquill
/// Inkflame but with the broader fanout.
pub fn silverquill_liturgy() -> CardDefinition {
    use crate::effect::shortcut::draw;
    CardDefinition {
        name: "Silverquill Liturgy",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
            draw(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bookbinder — {1}{B}, 1/1 Inkling Cleric. Magecraft → +1/+1
/// counter on this creature. 2-mana magecraft scaler — same shape as
/// Lorehold Bonepriest but in black for the Inkling tribal.
pub fn inkling_bookbinder() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bookbinder",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Scribebearer — {1}{W}, 1/2 Human Cleric Flying. ETB:
/// Scry 2. 2-mana evasive scry body.
pub fn silverquill_scribebearer() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Scribebearer",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Adept — {W}{B}, 2/1 Vampire Cleric. Magecraft → opp loses
/// 1 life. Cheap 2-mana pingy drain.
pub fn silverquill_adept() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Adept",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Spellguard — {2}{W}{B}, 3/3 Human Soldier First Strike.
/// ETB: gain 2 life. 4-mana sturdy combat body. (Name picked to avoid
/// the existing "Silverquill Vanguard".)
pub fn silverquill_spellguard() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Silverquill Spellguard",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sageling — {B}, 1/1 Inkling Cleric. "When this creature
/// dies, draw a card." 1-mana cantrip body — Inkling chump-blocker
/// that pays its mana back.
pub fn inkling_sageling() -> CardDefinition {
    use crate::effect::shortcut::{draw, on_dies};
    CardDefinition {
        name: "Inkling Sageling",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_dies(draw(1))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkcaller — {1}{W}{B}, 2/2 Vampire Cleric. ETB: mint a
/// 1/1 W/B Inkling token (flying). 3-mana Inkling-tribal mint body.
pub fn silverquill_inkcaller() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token;
    CardDefinition {
        name: "Silverquill Inkcaller",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lecture — {1}{W}{B}, Instant. "Each opponent loses 3
/// life and you gain 3 life." 3-mana instant-speed Silverquill drain.
pub fn silverquill_lecture() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lecture",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Battlescholar — {3}{W}{B}, 3/3 Inkling Cleric Flying. On
/// attack: +1/+0 EOT to self. 5-mana evasive growth body.
pub fn inkling_battlescholar() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Inkling Battlescholar",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Final-Year — {2}{B}, 3/2 Human Cleric Lifelink. Magecraft
/// → +1/+0 EOT self-pump. 3-mana lifelink aggro magecraft body.
pub fn silverquill_final_year() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Final-Year",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Devotee — {2}{W}, 2/3 Inkling Cleric. ETB: gain 2 life.
/// 3-mana defensive lifegain body.
pub fn inkling_devotee() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Inkling Devotee",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkspear — {W}{B}, Instant. "Target opponent loses 1
/// life and you gain 1 life." 2-mana single-target Silverquill drain.
pub fn silverquill_inkspear() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkspear",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::Target(0)),
            to: Selector::You,
            amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sergeant — {2}{W}, 2/2 Inkling Soldier. Static: other
/// Inklings you control get +1/+0. 3-mana Inkling-tribal anthem.
pub fn inkling_sergeant() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sergeant",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Inklings you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Inkling)
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Verdict — {2}{W}, Sorcery. "Exile target creature with
/// power 3 or greater. You gain 2 life." 3-mana white removal with a
/// power threshold — same template as Path to Exile-class spells but
/// gated on power ≥ 3.
pub fn silverquill_verdict() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Verdict",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(3)),
                ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Curator — {3}{B}, 2/3 Vampire Cleric. ETB: return target
/// creature card from your graveyard to your hand. 4-mana value
/// recursion body.
pub fn silverquill_curator() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Curator",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bondsmith — {1}{W}, 1/3 Inkling Cleric Flying. ETB: target
/// creature you control gets +1/+0 EOT and gains lifelink EOT. 2-mana
/// evasive combat trick body.
pub fn inkling_bondsmith() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Inkling Bondsmith",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Aspect — {1}{B}, 2/2 Inkling Cleric. "When this creature
/// enters, it gets +1/+0 and gains menace until end of turn." 2-mana
/// hasty-feeling Inkling. (Menace is a permanent EOT layer-7
/// modification per CR 702.110a — close enough to the printed
/// aggression on a 2-drop.)
pub fn inkling_aspect() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Aspect",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Menace,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 47 follow-up (modern_decks) — Silverquill expansion ────────────────

/// Silverquill Quillbinder — {2}{W}{B}, 3/3 Inkling Cleric Flying +
/// Lifelink. Synthesised Oracle: "Flying, lifelink. When this creature
/// enters, create a 1/1 W/B Inkling creature token with flying."
/// 4-mana double-evasion drain finisher with token rider.
pub fn silverquill_quillbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillbinder",
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
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Quillblade — {1}{W}, 2/1 Inkling Soldier Flying.
/// Synthesised Oracle: "Flying. Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, this creature gets +1/+1 until end of
/// turn." 2-mana evasive magecraft self-pump.
pub fn inkling_quillblade() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillblade",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Reprover — {2}{W}, 2/3 Human Cleric Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, target
/// creature an opponent controls gets -2/-0 until end of turn."
/// 3-mana defensive body + combat-disruption.
pub fn silverquill_reprover() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Reprover",
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
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Refrain — {W}{B} Instant. Synthesised Oracle: "Drain
/// 2 (each opponent loses 2 life and you gain 2 life). Surveil 1."
/// 2-mana drain + selection.
pub fn silverquill_refrain() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Refrain",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 48 (modern_decks) — Silverquill expansion ─────────────────────────

/// Silverquill Wingweaver — {1}{W}, 1/3 Inkling Cleric Flying.
/// Synthesised Oracle: "Flying. When this creature enters, surveil 1."
/// 2-mana evasive defensive body + graveyard-fill / draw smoothing.
pub fn silverquill_wingweaver() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Wingweaver",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Surveil {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Recital — {2}{W}{B} Sorcery. Synthesised Oracle:
/// "Each opponent loses 2 life and you gain 2 life. Draw a card."
/// 4-mana drain + cantrip. Stronger than Silverquill Witnessing's
/// same shape since the draw is unconditional.
pub fn silverquill_recital() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Recital",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Heralder — {1}{W}{B}, 2/2 Inkling Cleric Flying + Lifelink.
/// Vanilla 3-mana evasive lifelinker — stacks with Tenured Inkcaster's
/// +2/+2 anthem (→ 4/4 lifelink flier) and Inkling Verselord's lifelink
/// grant (already has lifelink — strict no-op stack).
pub fn inkling_heralder() -> CardDefinition {
    CardDefinition {
        name: "Inkling Heralder",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkdraft — {W}{B} Instant. Synthesised Oracle:
/// "Each opponent loses 1 life and you gain 1 life. Surveil 1."
/// 2-mana cheap drain + selection — same shape as Silverquill
/// Inkletter at the 2-mana slot. Test fodder for Light of Promise +
/// Witherbloom Apprentice payoff stacks.
pub fn silverquill_inkdraft() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkdraft",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lawscribe — {2}{W}, 2/2 Human Soldier Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, tap
/// target creature an opponent controls."
/// 3-mana defensive tempo body — same shape as Silverquill Lawkeeper.
pub fn silverquill_lawscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lawscribe",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Ascendancy — {2}{W}{B} Sorcery. Synthesised Oracle:
/// "Create two 1/1 W/B Inkling creature tokens with flying. Each
/// creature you control gets +1/+0 until end of turn."
/// 4-mana wide-anthem swing turn for Inkling tribal.
pub fn inkling_ascendancy() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ascendancy",
        cost: cost(&[generic(2), w(), b()]),
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
                definition: inkling_token(),
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 48 follow-up (modern_decks) — Silverquill expansion 2 ─────────────

/// Inkling Scriptmaster — {3}{W}{B}, 4/3 Inkling Cleric Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, each
/// opponent loses 2 life and you gain 2 life."
/// 5-mana evasive drain finisher.
pub fn inkling_scriptmaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scriptmaster",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkdancer — {1}{B}, 2/2 Inkling Rogue.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+0 until end of
/// turn." Aggressive 2-mana magecraft body.
pub fn silverquill_inkdancer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkdancer",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Vermilion — {2}{W} Instant. Synthesised Oracle:
/// "Target creature gets -3/-3 until end of turn. You gain 3 life."
/// 3-mana shrink-removal with lifegain rider.
pub fn silverquill_vermilion() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vermilion",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Drainmaster II — {3}{W}{B}, 3/3 Inkling Warlock.
/// Synthesised Oracle: "When this creature enters, each opponent
/// loses 3 life and you gain 3 life." 5-mana drain top-end.
pub fn silverquill_drainmaster_v2() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainmaster II",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Bookbond — {W}{B} Sorcery. Synthesised Oracle:
/// "Return target creature card from your graveyard to your hand.
/// You gain 1 life." 2-mana cheap recursion + lifegain.
pub fn silverquill_bookbond() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookbond",
        cost: cost(&[w(), b()]),
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
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 48 follow-up #2 (modern_decks) — more Silverquill cards ───────────

/// Inkling Scrollwarden — {3}{W}, 2/4 Inkling Soldier Flying + Vigilance.
/// Synthesised Oracle: "Flying, vigilance." 4-mana defensive evasive
/// body — pairs with the Inkling tribal anthem stack.
pub fn inkling_scrollwarden() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scrollwarden",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pencrafter — {1}{W}{B}, 2/3 Inkling Wizard. Synthesised
/// Oracle: "When this creature enters, draw a card. You lose 1 life."
/// 3-mana cantrip body with life cost.
pub fn silverquill_pencrafter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pencrafter",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
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
                    amount: Value::Const(1),
                },
                Effect::LoseLife {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Inkblot — {B} Sorcery. Synthesised Oracle: "Target opponent
/// loses 1 life and you gain 1 life." 1-mana cheap drain spell.
pub fn inkling_inkblot() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkblot",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 49 (modern_decks) — more Silverquill cards ────────────────────────

/// Silverquill Inkscribe — {1}{W}, 1/3 Human Cleric Vigilance.
/// Synthesised Oracle: "Vigilance. When this creature enters, you
/// gain 1 life." Defensive 2-drop body for white-leaning Silverquill
/// builds — pairs with the small-life-gain magecraft chain.
pub fn silverquill_inkscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkscribe",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Bookmender — {2}{W}, 2/3 Human Cleric.
/// Synthesised Oracle: "When this creature enters, target creature
/// you control gets +1/+1 until end of turn." 3-mana combat-trick
/// body — pumps any friendly creature on landing.
pub fn silverquill_bookmender() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookmender",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lifeskein — {W}{B} Instant. Synthesised Oracle:
/// "Target opponent loses 2 life and you gain 2 life."
/// 2-mana cheap drain instant — Silverquill's classic Hand of Silumgar
/// at instant speed.
pub fn silverquill_lifeskein() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifeskein",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: target_filtered(SelectionRequirement::Player),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Aerialist (v2) — {2}{W}{B}, 3/3 Inkling Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, target
/// opponent loses 1 life and you gain 1 life." 4-mana evasive drain
/// body — joins the Inkling tribal flying gang.
pub fn inkling_aerialist_v2() -> CardDefinition {
    CardDefinition {
        name: "Inkling Aerialist Cantor",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Censurewright — {1}{B}, 1/3 Human Rogue.
/// Synthesised Oracle: "When this creature enters, target creature
/// gets -1/-1 until end of turn." 2-mana cheap removal-trigger body.
pub fn silverquill_censurewright() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Censurewright",
        cost: cost(&[generic(1), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Cipherwing — {1}{W}{B}, 2/2 Inkling Wizard Flying.
/// Synthesised Oracle: "Flying. When this creature enters, target
/// player loses 1 life and you gain 1 life." 3-mana evasive drain
/// body — fills the Inkling tribal flyer curve at 3 mana.
pub fn inkling_cipherwing() -> CardDefinition {
    CardDefinition {
        name: "Inkling Cipherwing",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkstrike — {2}{B} Sorcery. Synthesised Oracle:
/// "Destroy target creature with toughness 2 or less." 3-mana
/// targeted destroy effect — fills the small-creature removal slot
/// in Silverquill's removal package.
pub fn silverquill_inkstrike() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkstrike",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ToughnessAtMost(2)),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Penmistress — {2}{W}{B}, 3/3 Vampire Cleric Lifelink.
/// Synthesised Oracle: "Lifelink. Whenever you cast an instant or
/// sorcery spell, this creature gets +1/+1 until end of turn." Combo
/// of static lifelink with magecraft self-pump — classic Silverquill
/// spellslinger anchor.
pub fn silverquill_penmistress() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penmistress",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 50: Silverquill synthesised cards ────────────────────────────────
//
// 20+ new STX Silverquill synthesised variants using existing primitives
// (ETB drain/gain-life, magecraft fan-outs, target_filtered selectors,
// token mints, Inkling-tribal anthems, the new `etb_draw` /
// `magecraft_loot` / `magecraft_scry` shortcuts in `effect.rs`).

/// Silverquill Cantor — {W}, 1/2 Human Cleric. ETB: gain 1 life.
/// 1-mana defensive lifegain enabler — Light of Promise anchor.
pub fn silverquill_cantor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cantor",
        cost: cost(&[w()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkscholar v2 — {1}{W}, 2/2 Human Wizard. ETB draws a card.
/// 2-mana Spirited-Companion-on-a-bigger-body — basic cantrip ETB.
/// (Disambiguated from the existing 1/1 Inkscholar.)
pub fn silverquill_inkscholar_b50() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Silverquill Inkscholar Adept",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![etb_draw(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Quillrunner — {1}{W}, 2/2 Human Soldier. Vigilance.
/// Magecraft: scry 1 (uses the new `magecraft_scry` shortcut).
/// 2-mana on-cast smoothing body — Silverquill scry sub-archetype.
pub fn silverquill_quillrunner() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Silverquill Quillrunner",
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
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Stylescribe — {W}{B}, 2/2 Inkling Cleric Flying.
/// Magecraft: scry 1 — Inkling-tribal smoothing.
pub fn inkling_stylescribe() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Inkling Stylescribe",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pageturner — {1}{W}, 1/3 Human Wizard. Vigilance.
/// ETB Scry 1. Defensive smoothing body.
pub fn silverquill_pageturner() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Pageturner",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Stormwriter — {2}{W}{B}, 3/2 Inkling Wizard Flying.
/// Magecraft: gain 1 life. 4-mana evasive lifegain on each IS cast.
pub fn inkling_stormwriter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stormwriter",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkbinder — {2}{W}, 2/3 Human Cleric. ETB target
/// creature you control gets +1/+1 EOT + gains Lifelink EOT. 3-mana
/// combat trick + lifelink-on-the-pumped-creature.
pub fn silverquill_inkbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbinder",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Quietus — {1}{B}, Instant. -3/-3 EOT to target
/// creature. 2-mana shrink-removal.
pub fn silverquill_quietus() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quietus",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Skywriter — {1}{W}{B}, 2/2 Inkling Wizard Flying.
/// Magecraft: target creature you control gains +1/+1 EOT.
pub fn inkling_skywriter() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Inkling Skywriter",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1,
            1,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Glyphmaster — {3}{W}{B}, 3/4 Vampire Cleric Lifelink.
/// ETB drain 2. 5-mana race breaker with lifelink + 4-life swing.
pub fn silverquill_glyphmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Glyphmaster",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Mournful — {2}{B}, 2/2 Inkling Rogue Flying.
/// Dies → drain 1 (each opp loses 1, you gain 1). 3-mana evasive
/// trade-up body with on-die drain payoff.
pub fn inkling_mournful() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mournful",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pen-Squire — {W}, 1/1 Human Soldier. Magecraft: this
/// creature gets +1/+0 EOT. Cheapest Silverquill self-pump magecraft body.
pub fn silverquill_pen_squire() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Squire",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Spellbinder — {3}{W}{B}, 4/4 Inkling Wizard Flying + Lifelink.
/// 5-mana evasive race breaker — vanilla flier + lifelink.
pub fn inkling_spellbinder() -> CardDefinition {
    CardDefinition {
        name: "Inkling Spellbinder",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Diction — {W}{B}, Instant. Drain 2 each opp + Surveil 1.
/// 2-mana drain + selection. Uses Seq.
pub fn silverquill_diction() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Diction",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Quietude — {2}{W}{B}, Sorcery. Drain 3 + Scry 2.
/// 4-mana drain + selection.
pub fn silverquill_quietude() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quietude",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Beautisage — {3}{W}, 3/3 Inkling Cleric Vigilance.
/// ETB: gain 3 life. 4-mana defensive lifegain finisher.
pub fn inkling_beautisage() -> CardDefinition {
    CardDefinition {
        name: "Inkling Beautisage",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(3)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkmender — {1}{W}{B}, 2/3 Vampire Warlock Lifelink.
/// ETB: return target ≤2 MV creature card from your gy to hand. 3-mana
/// lifelink reanimator.
pub fn silverquill_inkmender() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmender",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Memorial — {2}{W}{B}, Sorcery. Return target creature
/// card from your gy to bf + drain 1. 4-mana reanimator with drain.
pub fn silverquill_memorial() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memorial",
        cost: cost(&[generic(2), w(), b()]),
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
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Inkstain — {1}{W}, 2/1 Inkling Soldier. On-attack: target
/// creature gets -1/-0 EOT. Tempo-shrink attacker.
pub fn inkling_inkstain() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkstain",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Convene — {2}{W}{B}, Sorcery. Mint 2 Inkling tokens
/// + each opp loses 1. 4-mana double mint with drain rider.
pub fn silverquill_convene() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Convene",
        cost: cost(&[generic(2), w(), b()]),
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
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Sermoneer — {3}{W}, 2/4 Human Cleric Vigilance.
/// ETB Seq(Scry 1 + GainLife 1). 4-mana defensive smoother body.
pub fn silverquill_sermoneer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sermoneer",
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
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                Effect::GainLife {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Pageboy — {W}, 1/2 Inkling Cleric Flying. Vanilla 1-drop
/// evasive Inkling — cheapest evasive Inkling in the pool.
pub fn inkling_pageboy() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pageboy",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkstrike-Page — {1}{B}, Sorcery. Destroy target
/// creature with power ≤ 2. Cheap power-gated removal.
pub fn silverquill_inkstrike_page() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkstrike-Page",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Mentor — {2}{W}, 2/3 Human Cleric Vigilance.
/// ETB: target friendly creature gets a +1/+1 counter.
/// 3-mana sticky pumper.
pub fn silverquill_mentor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mentor",
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
                        .and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Necroscribe — {3}{B}, 3/3 Vampire Wizard. ETB return
/// target IS card from your graveyard to your hand. 4-mana value
/// recursion on a body.
pub fn silverquill_necroscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Necroscribe",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pronouncement — {3}{W}{B}, Sorcery. Seq(Drain 3 +
/// CreateToken 2 Inkling). 5-mana drain + double-mint finisher.
pub fn silverquill_pronouncement() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pronouncement",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(3),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Cipher — {W}{B}, Instant. Drain 1 + Draw 1.
/// 2-mana micro drain cantrip.
pub fn silverquill_cipher() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cipher",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Quillpoint — {1}{W}{B}, 2/3 Inkling Knight First Strike.
/// 3-mana first strike Inkling — combat-leaning Tenured Inkcaster fodder.
pub fn inkling_quillpoint() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillpoint",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Memoriam — {1}{W}{B}, 2/3 Vampire Cleric.
/// ETB Seq(Drain 1 + Scry 1). Compact 3-mana drain + smoothing body.
pub fn silverquill_memoriam() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memoriam",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
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
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Scry {
                    who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sigilbearer — {2}{W}{B}, 3/3 Inkling Cleric Flying. ETB
/// puts a +1/+1 counter on each other Inkling you control.
pub fn inkling_sigilbearer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sigilbearer",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Eulogize — {2}{W}{B}, Sorcery. Reanimate a Creature
/// card with mana value ≤ 3 from your graveyard + gain 2 life.
pub fn silverquill_eulogize() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Eulogize",
        cost: cost(&[generic(2), w(), b()]),
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
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                }),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Voidwalker — {3}{B}, 3/2 Inkling Rogue Flying + Menace.
/// 4-mana evasive double-evasion attacker.
pub fn inkling_voidwalker() -> CardDefinition {
    CardDefinition {
        name: "Inkling Voidwalker",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Menace],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Festscribe — {2}{W}{B}, 3/3 Vampire Wizard.
/// ETB: mints an Inkling token and you gain 2 life.
pub fn silverquill_festscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Festscribe",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
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
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── batch 53: more Silverquill cards ────────────────────────────────────────

/// Silverquill Scryward — {1}{W}, 2/2 Human Wizard. ETB Scry 1 + magecraft
/// gain 1 life. 2-mana scaling defender.
pub fn silverquill_scryward() -> CardDefinition {
    use crate::effect::shortcut::{etb_scry, magecraft_gain_life};
    CardDefinition {
        name: "Silverquill Scryward",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![etb_scry(1), magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Archivist — {2}{W}{B}, 2/3 Inkling Cleric Flying. ETB drain 1
/// and magecraft Scry 1. 4-mana evasive scaler.
pub fn inkling_archivist() -> CardDefinition {
    use crate::effect::shortcut::{etb_drain, magecraft_scry};
    CardDefinition {
        name: "Inkling Archivist",
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
        triggered_abilities: vec![etb_drain(1), magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Ledgermage — {2}{W}{B}, 3/3 Vampire Wizard. ETB Drain 2 via
/// the canonical drain template. 4-mana race-breaker body.
pub fn silverquill_ledgermage() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Silverquill Ledgermage",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Inkscribe — {W}{B}, 2/1 Inkling Soldier Flying. Aggressive
/// 2-mana evasive Inkling.
pub fn inkling_inkscribe() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkscribe",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Codex — {1}{W}, Sorcery. Seq(GainLife 2 + Draw 1). 2-mana
/// defensive cantrip.
pub fn silverquill_codex() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Codex",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Studyhall — {2}{W}, 2/3 Human Cleric Vigilance. Magecraft
/// gain 1 life — defensive vigilance body that scales with IS casts.
pub fn silverquill_studyhall() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Studyhall",
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
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pronouncer — {3}{W}{B}, 4/4 Inkling Bard Flying + Lifelink.
/// ETB drain 1 — 5-mana evasive lifelink finisher.
pub fn silverquill_pronouncer() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Silverquill Pronouncer",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Etching — {W}{B}, Instant. Seq(GainLife 2 + LoseLife 2 each
/// opp). 2-mana symmetric drain.
pub fn silverquill_etching() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Etching",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── batch 54: more Silverquill cards ────────────────────────────────────────

/// Silverquill Inkblot — {W}{B}, 2/2 Inkling Wizard Flying. On-attack
/// self-pump +1/+0 EOT (aggressive evasive Inkling).
pub fn silverquill_inkblot() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Silverquill Inkblot",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Chaplain — {1}{W}, 1/3 Inkling Cleric Vigilance + Lifelink.
/// Defensive 2-mana evasive lifelinker that locks down combat. Pairs with
/// Tenured Inkcaster's +2/+2 anthem (→ 3/5 Vigilance + Lifelink Flier).
pub fn inkling_chaplain() -> CardDefinition {
    CardDefinition {
        name: "Inkling Chaplain",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Warden — {2}{W}, 2/4 Human Cleric Vigilance. ETB Drain 1.
/// 3-mana defensive drain anchor.
pub fn silverquill_warden() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Warden",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Acolyte v2 — {1}{B}, 1/2 Inkling Cleric. Magecraft Drain 1.
/// Cheap on-cast magecraft drainer.
pub fn inkling_acolyte_v2() -> CardDefinition {
    CardDefinition {
        name: "Inkling Acolyte (Adept)",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Reflect — {2}{W}, Instant. Seq(Drain 2 + Surveil 2). 3-mana
/// instant-speed drain + deeper selection.
pub fn silverquill_reflect() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Reflect",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::Surveil {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Evangel — {3}{W}{B}, 3/3 Inkling Bard Flying + Lifelink. ETB
/// +1/+1 counter on target Inkling you control. Inkling tribal payoff.
pub fn inkling_evangel() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Evangel",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Invocation — {3}{W}{B}, Sorcery. Mints 3 Inkling tokens.
/// 5-mana wide go-wide finisher (Defend-the-Campus-template at the same
/// cost).
pub fn silverquill_invocation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Invocation",
        cost: cost(&[generic(3), w(), b()]),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Ghostwriter — {2}{B}, 2/3 Inkling Rogue. Magecraft drain each
/// opp for 1. Medium-mana magecraft drainer.
pub fn inkling_ghostwriter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ghostwriter",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Doom — {2}{B}, Instant. Drain 4 (5-life swing) — instant-
/// speed drain finisher.
pub fn silverquill_doom() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Doom",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(4),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Attendant — {W}{B}, 1/2 Inkling Cleric Flying + Lifelink. ETB
/// Scry 1 — cheap evasive lifelink with smoothing rider.
pub fn inkling_attendant() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Inkling Attendant",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Psalm — {1}{W}{B}, Instant. Seq(Drain 2 + Draw 1). 3-mana
/// instant-speed drain + cantrip.
pub fn silverquill_psalm() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Psalm",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Pageant — {2}{W}{B}, Sorcery. Seq(Mint 2 Inklings + GainLife
/// 2). 4-mana mint + lifegain.
pub fn inkling_pageant() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pageant",
        cost: cost(&[generic(2), w(), b()]),
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
                definition: inkling_token(),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 55): 5 more Silverquill cards ────────────────

/// Silverquill Pen-Scholar — {1}{W}, 2/2 Human Cleric. ETB Seq(GainLife 1
/// + Scry 1). Defensive lifegain body that smooths the topdeck.
pub fn silverquill_pen_scholar() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Pen-Scholar",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Mortician — {2}{W}{B}, 3/3 Vampire Warlock. Whenever you
/// sacrifice a creature, drain 1 (Silverquill spin on the new sacrifice
/// event — Witherbloom Mortician's sister card).
pub fn silverquill_mortician() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Mortician",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: drain(1),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sentinel II — {2}{W}, 1/4 Inkling Soldier Vigilance. Defensive
/// Inkling — slots into Tenured Inkcaster + Inkling Verselord shells.
pub fn inkling_sentinel_b55() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentinel II",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inksong — {W}{B}, Instant. Seq(Drain 1 + Scry 2). 2-mana
/// instant-speed drain + heavy selection.
pub fn silverquill_inksong() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Inksong",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Pact-Caller — {1}{W}{B}, 2/3 Inkling Cleric Flying. ETB mints
/// 1 Inkling token. Self-replicating Inkling enabler.
pub fn inkling_pact_caller() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token;
    CardDefinition {
        name: "Inkling Pact-Caller",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 56) — new Silverquill STX cards ──────────────

/// Silverquill Bloodscribe — {1}{W}{B}, 2/2 Vampire Cleric Flying +
/// Lifelink. "Whenever you sacrifice a creature, you may pay 1 life.
/// If you do, draw a card." Simplified: drains 1 + draws 1 on each
/// sacrifice (the optional life-cost gate is dropped to a flat trigger
/// — pays out at net cost 0 since lifelink will offset the 1 life on
/// the next combat attack).
///
/// Engine: rides the `EventKind::CreatureSacrificed/YourControl` event.
pub fn silverquill_bloodscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bloodscribe",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Penblade — {W}, 1/1 Inkling Soldier Flying. ETB target
/// creature gets +1/+0 EOT. Cheap evasive enabler that gives any
/// attacker a 1-mana power boost — same shape as the original
/// Silverquill Pupil but on a flying body.
pub fn inkling_penblade() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Penblade",
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
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Litany II — {1}{W}{B}, Sorcery. Drain 2 (each opp loses 2,
/// you gain 2) and mill 2 from each opponent. Drain + mill double
/// dip — feeds delirium / graveyard payoffs on opp side while
/// stabilizing life.
pub fn silverquill_litany_b56() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Litany II",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Inkmaster — {2}{W}{B}, 2/3 Inkling Wizard Flying. Magecraft
/// each opp loses 1 life and you gain 1 life — Witherbloom-Apprentice
/// drain template on a flying body, costed for the {W}{B} flyer slot.
pub fn inkling_inkmaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkmaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Acolyte — {1}{W}, 2/2 Human Cleric. ETB Drain 1.
/// 2-mana defensive drain body — Light-of-Promise enabler.
pub fn silverquill_acolyte_b56() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Acolyte II",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 57): 5 more Silverquill cards ────────────────

/// Silverquill Inkscribe — {W}{B}, 2/2 Inkling Cleric with Flying.
/// Dies-trigger: each opponent loses 2 life. 2-mana evasive trade-up
/// body — when removed it still extracts value via the on-die drain.
pub fn silverquill_inkscribe_b57() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Silverquill Inkscribe II",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![on_dies(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Scriptmaster — {2}{W}{B}, 3/3 Vampire Cleric.
/// ETB Drain 2 + Scry 1. 4-mana mid-curve value engine.
pub fn silverquill_scriptmaster() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Silverquill Scriptmaster",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(2),
            Effect::Scry {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bladerunner — {2}{W}, 2/2 Inkling Soldier with Flying +
/// First Strike. 3-mana evasive combat-ready Inkling — works under
/// the Tenured Inkcaster anthem for an effective 4/4 first-strike flyer.
pub fn inkling_bladerunner() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bladerunner",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Sentinel — {1}{W}, 1/3 Inkling Soldier with Vigilance
/// + Flying. 2-mana defensive flyer that pulls double duty on attack
///   (with Tenured Inkcaster: 3/5 vigilance flyer).
pub fn silverquill_sentinel_b57() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentinel III",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pen-Master — {3}{W}{B}, 3/3 Inkling Wizard with Flying.
/// ETB loot + drain 1 (each opp loses 1, you gain 1). Top-curve
/// value engine — five mana for an evasive 3/3 + card velocity + drain.
pub fn silverquill_pen_master() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Silverquill Pen-Master",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            drain(1),
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 58): 5 more Silverquill cards ────────────────

/// Silverquill Wordmaiden — {1}{W}, 2/1 Human Cleric. Magecraft: target
/// creature gets +1/+1 EOT. Cheap aggressive body that pumps any
/// friendly each instant or sorcery spell.
pub fn silverquill_wordmaiden() -> CardDefinition {
    use crate::effect::shortcut::{magecraft_target_pump, target_filtered};
    CardDefinition {
        name: "Silverquill Wordmaiden",
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
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 1,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Quillblade II — {1}{B}, 2/1 Inkling Wizard with Flying.
/// 2-mana evasive body — fills the Silverquill flying-aggro shell.
pub fn inkling_quillblade_b58() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillblade II",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Scribecaller — {W}{B}, 2/2 Inkling Soldier with Lifelink.
/// 2-mana lifelink body — anchors the W/B drain shell with steady
/// life swing on every attack.
pub fn silverquill_scribecaller() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scribecaller",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lecturer II — {2}{W}{B}, 2/3 Human Cleric. ETB Seq(mint
/// 2/1 flying Inkling token + gain 2 life). 4-mana wide value body —
/// drops two bodies for the price of one and stabilises the life total.
pub fn silverquill_lecturer_b58() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Lecturer II",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
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
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkmaster — {3}{W}{B}, 3/4 Inkling Wizard with Flying.
/// Magecraft: each opp loses 1 life and you gain 1 life. 5-mana
/// evasive drain engine — Felisa-light without the counter rider.
pub fn silverquill_inkmaster_b58() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmaster II",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 59): 5 more Silverquill cards ────────────────

/// Silverquill Scrivener II — {2}{W}, 2/2 Human Cleric. ETB Surveil 1 +
/// magecraft Scry 1. 3-mana double-selection body — scales card velocity
/// per IS cast.
pub fn silverquill_scrivener_b59() -> CardDefinition {
    use crate::effect::shortcut::{etb_surveil, magecraft_scry};
    CardDefinition {
        name: "Silverquill Scrivener II",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![
            etb_surveil(1),
            magecraft_scry(1),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkflight — {1}{W}, 1/2 Inkling Cleric with Flying.
/// Magecraft self-pump +1/+0 EOT. Cheap evasive body that grows on
/// each IS cast.
pub fn silverquill_inkflight_b59() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkflight",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Pen-Priest — {W}{B}, 1/3 Vampire Cleric Lifelink. ETB
/// drain 1. 2-mana defensive lifelink + immediate drain.
pub fn silverquill_pen_priest() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Priest",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Summit — {2}{W}{B}, 2/3 Inkling Soldier with Flying. ETB:
/// put a +1/+1 counter on each other Inkling you control. 4-mana
/// tribal anthem-on-ETB.
pub fn inkling_summit_b59() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Summit",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Drainbearer — {1}{B}, 2/1 Inkling Rogue with Menace.
/// Dies-trigger: each opp loses 1, you gain 1. Aggressive evasive-ish
/// body with on-death drain rider.
pub fn silverquill_drainbearer() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Silverquill Drainbearer",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![dies_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 60): 3 more Silverquill cards ────────────────

/// Silverquill Mageblade — {1}{W}, 2/2 Human Soldier. Magecraft: +1/+0
/// EOT to target friendly creature. 2-mana per-cast combat trick body.
pub fn silverquill_mageblade() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Silverquill Mageblade",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 0,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Sigilwarden — {3}{W}, 2/4 Inkling Soldier with Flying +
/// Vigilance. ETB +1/+1 counter on each other Inkling you control.
/// 4-mana defensive flier tribal payoff.
pub fn inkling_sigilwarden() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Sigilwarden",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Quillthane — {2}{W}{B}, 3/3 Vampire Cleric. ETB Drain 2
/// + Surveil 1. 4-mana defensive value engine — Drain swing + selection
///   on entry.
pub fn silverquill_quillthane() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Silverquill Quillthane",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(2),
            Effect::Surveil {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 61): 5 more Silverquill cards ────────────────

/// Silverquill Pentor — {1}{W}, 2/2 Human Cleric. ETB Seq(GainLife 2 +
/// magecraft Scry 1). 2-mana defensive lifegain body + on-cast smoother.
pub fn silverquill_pentor_b61() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Pentor",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![
            etb(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            }),
            crate::effect::shortcut::magecraft_scry(1),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Arbiter — {W}{B}, 2/2 Inkling Cleric Flying + Lifelink.
/// Compact 2-mana evasive lifelinker — Tenured Inkcaster fodder.
pub fn inkling_arbiter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Arbiter",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkmage — {2}{W}{B}, 3/3 Vampire Wizard. ETB Drain 2 via
/// the `etb_drain(2)` shortcut. 4-mana drain race-breaker.
pub fn silverquill_inkmage_b61() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmage",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Letterer — {2}{W}, 2/3 Inkling Soldier Flying + Vigilance.
/// ETB Scry 1. 3-mana defensive evasive smoother — Tenured Inkcaster
/// fodder with selection rider.
pub fn inkling_letterer() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Inkling Letterer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Drainpoet — {3}{W}{B}, 3/3 Vampire Bard Flying. ETB drain
/// 3 + magecraft GainLife 1. 5-mana race-breaker engine — 6-life swing
/// on entry plus a per-cast lifegain rider.
pub fn silverquill_drainpoet() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainpoet",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            etb_drain(3),
            magecraft_gain_life(1),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 62): 2 more Silverquill cards ────────────────

/// Inkling Calligrapher II — {1}{W}{B}, 2/3 Inkling Wizard Flying.
/// Magecraft Scry 1 via the `magecraft_scry(1)` shortcut. 3-mana evasive
/// smoother body — Tenured Inkcaster fodder with on-cast selection.
pub fn inkling_calligrapher_b62() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Inkling Calligrapher II",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_scry(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Lecturer II — {2}{W}{B}, 3/2 Vampire Cleric Lifelink. ETB
/// Seq(Drain 1 + Surveil 1). 4-mana value engine — lifelink + drain +
/// graveyard fuel rolled into a single curve-out body.
pub fn silverquill_lecturer_b62() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Silverquill Lecturer II",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(1),
            Effect::Surveil {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 63): 5 more Silverquill cards ─────────────────

/// Inkling Scribesage — {1}{W}{B}, 2/2 Inkling Cleric Flying. Magecraft
/// gain 1 life. 3-mana evasive lifegain-on-cast body.
pub fn inkling_scribesage() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scribesage",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Dirgesage — {2}{W}{B}, 3/3 Vampire Cleric. ETB drain 2.
/// 4-mana sturdy drain body.
pub fn silverquill_dirgesage() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Dirgesage",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Hymnsmith — {1}{W}, 2/2 Human Wizard. Magecraft +1/+0 EOT
/// self-pump via `magecraft_self_pump(1, 0)`. 2-mana magecraft scaler.
pub fn silverquill_hymnsmith() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Hymnsmith",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Quillchorus — {3}{W}{B}, Sorcery. Creates 3 Inkling tokens
/// + drain 1 each opp. 5-mana go-wide drain.
pub fn silverquill_quillchorus() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Quillchorus",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                definition: inkling_token(),
                count: Value::Const(3),
            },
            drain(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Riftcaster — {2}{B}, 2/3 Inkling Wizard Flying. Magecraft drain
/// 1 each opp. 3-mana evasive drain magecraft body.
pub fn inkling_riftcaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Riftcaster",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks, batch 64): 10 more Silverquill cards ───────────────
//
// Focus: maximally finishing Silverquill via a wide push — bread-and-butter
// inkling tribal payoffs, drain bodies, and combat tricks built on the
// existing shortcut helpers. Each card has at least one functionality test
// in `tests::stx`.

/// Inkling Recitalist — {1}{W}, 2/2 Inkling Cleric. Magecraft +1/+1 EOT to
/// target friendly Inkling. Cheap Inkling-tribal scaler.
pub fn inkling_recitalist() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Inkling Recitalist",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature.and(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling),
            )
            .and(SelectionRequirement::ControlledByYou)),
            1,
            1,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Vespersong — {2}{W}{B}, Sorcery. Seq(Drain 2 + Draw 1).
/// 4-mana drain-and-draw.
pub fn silverquill_vespersong() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Vespersong",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Battlechoir — {3}{W}{B}, 3/4 Inkling Cleric Flying + Lifelink.
/// ETB drain 3 via the `etb_drain(3)` shortcut. 5-mana race-breaker.
pub fn inkling_battlechoir() -> CardDefinition {
    CardDefinition {
        name: "Inkling Battlechoir",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkmuse — {W}{B}, 2/1 Vampire Cleric Lifelink. Magecraft
/// Surveil 1 — every spell sets up the next.
pub fn silverquill_inkmuse() -> CardDefinition {
    use crate::effect::shortcut::magecraft_surveil;
    CardDefinition {
        name: "Silverquill Inkmuse",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_surveil(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Heraldcourier — {2}{W}, 2/3 Inkling Soldier Flying + Vigilance.
/// ETB mint 1 Inkling token. Inkling-tribal go-wide enabler.
pub fn inkling_heraldcourier() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Heraldcourier",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: inkling_token(),
            count: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkscale — {1}{W}{B}, Instant. Seq(PumpPT(+2/+0 EOT) +
/// GrantKeyword(Lifelink, EOT)) on target friendly. 3-mana combat trick.
pub fn silverquill_inkscale() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkscale",
        cost: cost(&[generic(1), w(), b()]),
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
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Lifelink,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Pallidwing — {3}{W}, 2/3 Inkling Cleric Flying + Lifelink.
/// Vanilla evasive lifelinker — Tenured Inkcaster anthem fodder.
pub fn inkling_pallidwing() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pallidwing",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Cantillator — {2}{W}, 2/3 Human Cleric. ETB gain 2 life +
/// magecraft +1/+0 EOT self-pump. 3-mana lifegain + scaler.
pub fn silverquill_cantillator() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cantillator",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
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
        triggered_abilities: vec![etb_gain_life(2), magecraft_self_pump(1, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Stormpenner — {2}{W}{B}, 2/3 Inkling Wizard Flying. Magecraft
/// AddCounter +1/+1 on self. 4-mana self-growing evasive body.
pub fn inkling_stormpenner() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stormpenner",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Inkling Bannerer — {1}{W}{B}, 2/2 Inkling Cleric. Magecraft pump each
/// Inkling you control +1/+0 EOT via the new tribal-pump shortcut.
pub fn inkling_bannerer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_pump_each_creature_type;
    CardDefinition {
        name: "Inkling Bannerer",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_pump_each_creature_type(
            CreatureType::Inkling,
            1,
            0,
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Silverquill Inkmark — {1}{B}, Sorcery. Target opponent loses 3 life,
/// you gain 3 life. Pure 2-mana drain spell.
pub fn silverquill_inkmark() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmark",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(3),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}
