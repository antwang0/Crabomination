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
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, Selector, SelectionRequirement, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{magecraft, target_filtered};
use crate::effect::Duration;
use crate::mana::{cost, generic, w, b, x};

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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Vanishing Verse ─────────────────────────────────────────────────────────

/// Vanishing Verse — {W}{B} Instant. "Exile target nonland, monocolored
/// permanent."
///
/// Now wired (push XX) via the new `SelectionRequirement::Monocolored`
/// predicate (push XX). The target filter is `Permanent ∧ Nonland ∧
/// Monocolored`, so two-color and colorless permanents reject as
/// invalid targets at cast time. (Monocolored is `distinct_colors == 1`;
/// hybrid pips count both halves, Phyrexian counts the colored side.)
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Killian, Ink Duelist ────────────────────────────────────────────────────

/// Killian, Ink Duelist — {W}{B}, 2/3 Legendary Human Warrior with Lifelink.
///
/// ✅ Push XXXVIII: 🟡 → ✅. The static "spells you cast that target a
/// creature cost {2} less" is now wired via the new
/// `StaticEffect::CostReductionTargeting { spell_filter: Any,
/// target_filter: Creature, amount: 2 }`. The discount is applied to
/// generic mana only at cast time by `cost_reduction_for_spell` in
/// `game/actions.rs` (cannot reduce colored requirements). All three
/// cast paths (regular hand cast, alt cost, flashback) consult the
/// reduction.
///
/// Two Killians stack: a {3}{B} creature-targeting spell becomes free
/// (3 - 2 - 2 = saturated to 0 generic, plus the {B}). Killian himself
/// (cost {W}{B}) is fully colored so the discount no-ops on him.
pub fn killian_ink_duelist() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Killian, Ink Duelist",
        cost: cost(&[w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast that target a creature cost {2} less to cast",
            effect: StaticEffect::CostReductionTargeting {
                spell_filter: SelectionRequirement::Any,
                target_filter: SelectionRequirement::Creature,
                amount: 2,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Devastating Mastery ─────────────────────────────────────────────────────

/// Devastating Mastery — {4}{W}{W} Sorcery. Printed Oracle:
/// "Destroy all nonland permanents.
///  Mastery — {7}{W}{W}: ...and return up to two target nonland
///  permanent cards from your graveyard to the battlefield under your
///  control."
///
/// ✅ Push XXXVIII: 🟡 → ✅. Wired as a 2-mode `Effect::ChooseMode`
/// where mode 0 is the printed Wrath and mode 1 is Wrath + reanimate.
/// The Mastery alt cost ({7}{W}{W}) wires via the new
/// `AlternativeCost.mode_on_alt: Some(1)` field — paying the alt cost
/// auto-selects mode 1 at cast time, so the user can't pay alt cost
/// then "skip" the reanimate. Regular cast at {4}{W}{W} resolves
/// mode 0. The reanimate target slot picks the oldest nonland card
/// in the caster's graveyard (auto-decider chooses index 0); a future
/// multi-target prompt would let the user pick exactly two.
pub fn devastating_mastery() -> CardDefinition {
    use crate::card::AlternativeCost;
    use crate::effect::{PlayerRef, ZoneDest};
    use crate::mana::ManaCost;
    let wrath = Effect::ForEach {
        selector: Selector::EachPermanent(SelectionRequirement::Nonland),
        body: Box::new(Effect::Destroy {
            what: Selector::TriggerSource,
        }),
    };
    // Mastery rider: return one nonland permanent card from your
    // graveyard. Single-pick today (multi-target prompt gap); future
    // engine work would extend this to "up to two."
    let reanimate = Effect::Move {
        what: Selector::take(
            Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::Nonland.and(SelectionRequirement::Permanent),
            },
            crate::effect::Value::Const(1),
        ),
        to: ZoneDest::Battlefield {
            controller: PlayerRef::You,
            tapped: false,
        },
    };
    CardDefinition {
        name: "Devastating Mastery",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            wrath.clone(),
            Effect::Seq(vec![wrath, reanimate]),
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::new(vec![
                crate::mana::generic(7),
                crate::mana::w(),
                crate::mana::w(),
            ]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            mode_on_alt: Some(1),
        }),
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Hunt for Specimens ──────────────────────────────────────────────────────

/// Hunt for Specimens — {3}{B} Sorcery. "Create a 1/1 black Pest creature
/// token with 'When this creature dies, you gain 1 life.' Then learn."
///
/// Push XXIV: promoted ✅. The spawned Pest token carries the printed
/// "When this creature dies, you gain 1 life" trigger via
/// `TokenDefinition.triggered_abilities` (SOS push VI). Learn collapses
/// to `Draw 1` — same approximation as Eyetwitch / Igneous Inspiration
/// / Enthusiastic Study; the Lesson sideboard model is tracked as a
/// future engine feature in TODO.md but doesn't gate the card's primary
/// play pattern (Pest body + cantrip).
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Star Pupil ──────────────────────────────────────────────────────────────

/// Star Pupil — {B}, 0/0 Spirit. Printed Oracle:
/// "Star Pupil enters the battlefield with two +1/+1 counters on it.
///  When Star Pupil dies, put a +1/+1 counter on target creature."
///
/// Push XL: ✅ promoted via the new `enters_with_counters`
/// replacement field. Base body is now the printed 0/0 (no
/// over-statement); the two +1/+1 counters are added at bf entry
/// time *before* SBAs run, so the 0-toughness body never sees the
/// graveyard. Felisa, Fang of Silverquill's "creature you control
/// with a counter on it dies" trigger keeps firing correctly
/// because the counters are real `CounterType::PlusOnePlusOne`. The
/// dies trigger is faithful — `EventKind::CreatureDied/SelfSource` →
/// `Effect::AddCounter` on a targeted creature.
pub fn star_pupil() -> CardDefinition {
    CardDefinition {
        name: "Star Pupil",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        // Printed 0/0 — the two +1/+1 counters from
        // `enters_with_counters` keep it alive at 2/2.
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
    }
}

// ── Codespell Cleric ────────────────────────────────────────────────────────

/// Codespell Cleric — {W}, 1/1 Human Cleric. Printed Oracle:
/// "Lifelink. When Codespell Cleric enters the battlefield, scry 1."
///
/// Vanilla one-mana Cleric body with Lifelink + ETB scry. Both pieces
/// are first-class engine primitives — `Keyword::Lifelink` on the body
/// and `Effect::Scry` on the ETB trigger.
pub fn codespell_cleric() -> CardDefinition {
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Codespell Cleric",
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Combat Professor ────────────────────────────────────────────────────────

/// Combat Professor — {3}{W}, 2/3 Cat Cleric, Flying. Printed Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
///  target creature gets +1/+1 until end of turn."
///
/// Same shape as Eager First-Year (the Silverquill apprentice) but
/// gated on a 2/3 flier body for {3}{W}. Wired via the `magecraft()`
/// helper plus the standard `target_filtered(Creature)` pump body —
/// auto-target framework picks the highest-power friendly creature.
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
        toughness: 3,
        keywords: vec![Keyword::Flying],
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Silverquill Command ─────────────────────────────────────────────────────

/// Silverquill Command — {2}{W}{B} Instant.
/// "Choose two —
/// • Counter target activated or triggered ability.
/// • Target creature gets -3/-3 until end of turn.
/// • Target player loses 3 life and you gain 3 life.
/// • Draw a card."
///
/// Push XXXVI: ✅ — "choose two" now wires faithfully via the new
/// `Effect::ChooseModes { count: 2, up_to: false, allow_duplicates:
/// false }` primitive. Auto-decider picks modes 0+1 (counter activated/
/// triggered ability + -3/-3 EOT on a creature) — both modes read
/// `Target(0)` so the same Permanent target is shared. For pure-value
/// pair (drain 3 + draw 1, modes 2+3), use `ScriptedDecider::new([
/// DecisionAnswer::Modes(vec![2, 3])])`. The multi-target uniqueness
/// caveat (modes 0 and 1 both reading Target(0)) is the same engine
/// gap noted on Moment of Reckoning / Together as One.
pub fn silverquill_command() -> CardDefinition {
    use crate::effect::{Duration, PlayerRef};
    CardDefinition {
        name: "Silverquill Command",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseModes {
            count: 2,
            up_to: false,
            allow_duplicates: false,
            modes: vec![
                // Mode 0: counter target activated/triggered ability (same
                // primitive as Stifle / Quandrix Command mode 0).
                Effect::CounterAbility {
                    what: target_filtered(SelectionRequirement::Permanent),
                },
                // Mode 1: -3/-3 EOT.
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-3),
                    toughness: Value::Const(-3),
                    duration: Duration::EndOfTurn,
                },
                // Mode 2: drain 3 (each-opp collapse, same as Witherbloom
                // Command mode 0).
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(3),
                },
                // Mode 3: draw a card.
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
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Dueling Coach ───────────────────────────────────────────────────────────

/// Dueling Coach — {2}{W}, 3/3 Human Cleric. Printed Oracle:
/// "Vigilance.
///  Magecraft — Whenever you cast or copy an instant or sorcery spell,
///  put a +1/+1 counter on target creature."
///
/// Push XXX: ✅. Silverquill counter-payoff body — same magecraft rider as
/// Lecturing Scornmage / Stonebinder's Familiar but with a much bigger
/// 3/3 Vigilance frame and a `Creature` (any-side) target. Wired via the
/// `magecraft()` shortcut + `Effect::AddCounter` on a target_filtered
/// creature. Auto-decider picks the highest-power friendly creature.
pub fn dueling_coach() -> CardDefinition {
    CardDefinition {
        name: "Dueling Coach",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Hall Monitor ────────────────────────────────────────────────────────────

/// Hall Monitor — {W}, 1/1 Human Wizard. Printed Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
///  target creature can't block this turn."
///
/// Push XXX: ✅. One-mana Silverquill defensive magecraft creature —
/// the printed "can't block this turn" rider is wired via
/// `Effect::GrantKeyword { keyword: Keyword::CantBlock, duration:
/// EndOfTurn }` against a target creature (any side — auto-decider picks
/// the largest opposing blocker), reusing the same primitive Duel
/// Tactics uses for its CantBlock EOT clause. Auto-target framework
/// chooses the most threatening opposing creature.
pub fn hall_monitor() -> CardDefinition {
    CardDefinition {
        name: "Hall Monitor",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Clever Lumimancer ───────────────────────────────────────────────────────

/// Clever Lumimancer — {W}, 1/1 Human Wizard. Printed Oracle:
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
///  Clever Lumimancer gets +2/+2 until end of turn."
///
/// Push XXX: ✅. Aggressive Silverquill magecraft creature — bigger
/// per-magecraft pump than Symmetry Sage's +1/+0 (just stat-line, no
/// flying) on the same {W} 1/1 frame. Wired via the `magecraft_self_
/// pump(2, 2)` shortcut: the self-source pump fires on every IS cast
/// for +2/+2 EOT.
pub fn clever_lumimancer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Clever Lumimancer",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(2, 2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Inkling Summoning siblings: Karok Wrangler ──────────────────────────────

/// Karok Wrangler — {2}{W}, 3/3 Human Wizard. Printed Oracle:
/// "When this creature enters, tap target creature an opponent controls,
///  then put a stun counter on it. If you control two or more Wizards,
///  put an additional stun counter on it instead."
///
/// Push XXXI: ✅. The Wizard-count rider now wires via `Effect::If`
/// gated on `Predicate::ValueAtLeast(CountOf(EachPermanent(Creature ∧
/// Wizard ∧ ControlledByYou)), 2)`. The "instead" wording is
/// approximated as additive (1 base stun counter → 2 stun counters
/// when ≥2 Wizards) rather than a strict swap, but stun counters
/// stack 1-for-1 against future untap steps so combat-correct against
/// the printed semantic. Karok itself counts toward the Wizard
/// threshold, so a solo Karok lands 1 stun, but Karok next to any
/// other Wizard (e.g., Hall Monitor, Combat Professor, Codespell
/// Cleric) lands 2.
pub fn karok_wrangler() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Karok Wrangler",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
                // Wizard-count rider: +1 additional stun counter when
                // you control ≥2 Wizards (Karok itself counts).
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountOf(Box::new(Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::HasCreatureType(
                                    CreatureType::Wizard,
                                ))
                                .and(SelectionRequirement::ControlledByYou),
                        ))),
                        Value::Const(2),
                    ),
                    then: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        additional_discard_cost: None,
        additional_life_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

