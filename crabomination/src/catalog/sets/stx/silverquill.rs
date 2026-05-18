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
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, Supertype, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
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
    use crate::catalog::sets::sos::inkling_token;
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
    use crate::catalog::sets::sos::inkling_token;
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
    }
}
