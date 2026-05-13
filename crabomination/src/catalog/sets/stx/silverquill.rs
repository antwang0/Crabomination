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
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    Selector, SelectionRequirement, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, StaticAbility, StaticEffect};
use crate::mana::{cost, generic, u, w, b, x};

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
    }
}

// ── Tenured Inkcaster ───────────────────────────────────────────────────────

/// Tenured Inkcaster — {2}{W}{B}, 3/2 Vampire Warlock. "Other Inkling
/// creatures you control get +2/+2."
///
/// Tribal anthem on the Inkling creature type. The "Other" gate is
/// wired via the engine's `AffectedPermanents::AllWithCreatureType
/// .exclude_source: true` flag (push XXX, Quintorius pattern). The
/// anthem is layered in via a compute-time injection in
/// `GameState::compute_battlefield`, so all of the controller's
/// Inkling creatures (including Inkling tokens from Inkling Summoning,
/// Defend the Campus) get +2/+2 while Inkcaster is on the battlefield
/// — Inkcaster himself stays a 3/2 Vampire (he is not an Inkling, so
/// the exclude-source clause is technically vacuous, but the
/// CreatureType filter on the layer already excludes non-Inklings
/// anyway). The +2/+2 makes a 2/1 Inkling token attack as a 4/3
/// flier, which is a huge Silverquill payoff.
pub fn tenured_inkcaster() -> CardDefinition {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
