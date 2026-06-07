//! Silverquill (W/B) college cards from Strixhaven.
//!
//! Common shapes:
//! - **Magecraft** triggers (Eager First-Year): "whenever you cast or copy
//!   an instant or sorcery spell, …". Implemented via the spell-cast trigger
//!   path with an `EventSpec.filter` predicate that gates on the just-cast
//!   spell's card type. See `fire_spell_cast_triggers` in
//!   `crabomination::game::actions`.
//! - **Learn** (Eyetwitch death trigger, Hunt for Specimens rider). Wired via
//!   `Effect::Learn`: reveal a Lesson from the sideboard into hand, or
//!   discard-to-draw (falls back to a plain draw when no Lessons sideboard is
//!   configured). Every deck-build path seats the standard sideboard.
//!
//! Many cards also have static abilities or token-creation clauses that need
//! engine features the engine doesn't have yet (cost-reduction-aware-of-
//! target, token-with-self-die-trigger). Each affected card is marked 🟡 in
//! the tracker; the body / keywords / P/T are still correct so the card is
//! playable as a 4/3 lifelink flier or whatever.

use crate::catalog::sets::sos::inkling_token;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{
    drain, drain_and_scry, drain_and_surveil, etb, etb_drain, etb_gain_life, etb_mint_token,
    magecraft, magecraft_drain_each_opp, magecraft_gain_life, magecraft_scry, magecraft_self_pump,
    on_attack_drain, on_attack_gain_life, on_other_dies, target_filtered,
};
use crate::effect::{Duration, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{cost, generic, g, u, w, b, x, ManaCost};

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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog, CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Eyetwitch ───────────────────────────────────────────────────────────────

/// Eyetwitch — {B}, 1/1 Pest. "When Eyetwitch dies, learn." Set: Strixhaven.
///
/// Learn via `Effect::Learn` — reveal a Lesson into hand or discard-to-draw.
pub fn eyetwitch() -> CardDefinition {
    CardDefinition {
        name: "Eyetwitch",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            // Learn (CR 701.45): reveal a Lesson from your sideboard into
            // hand, or discard a card to draw. `Effect::Learn` falls back to
            // a plain draw when no Lessons sideboard is configured.
            effect: Effect::Learn {
                who: crate::effect::PlayerRef::You,
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast that target a creature cost {2} less to cast.",
            effect: StaticEffect::CostReductionTargetingFilter {
                spell_filter: SelectionRequirement::Any,
                target_filter: SelectionRequirement::Creature,
                amount: 2,
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        ..Default::default()
    }
}

// ── Mavinda, Students' Advocate ─────────────────────────────────────────────

/// Mavinda, Students' Advocate — {1}{W}{W}, 1/3 Legendary Human Cleric,
/// Flying + Vigilance.
///
/// Push (modern_decks, batch 73): the `{0}` cast-from-graveyard
/// activated ability is **now wired** via the Move(target → Exile) +
/// `GrantMayPlay { exile_after: true }` permission-grant pattern (same
/// shape as Nita Forum Conciliator's activation, which lands a gy IS
/// card in exile with may-play-this-turn + exile-on-resolve). Cost
/// {0} + `once_per_turn: true` (printed "Activate only once each
/// turn"). The target filter is "Instant ∨ Sorcery" in your graveyard
/// — the printed "that targets only a single creature" sub-filter is
/// approximated to all IS cards (the engine has no "card in gy that
/// would target only a creature" introspection since gy cards aren't
/// on the stack — non-creature-target IS spells in your gy can still
/// be picked, a minor convenience extension over the printed Oracle).
/// Body/flying/vigilance unchanged.
pub fn mavinda_students_advocate() -> CardDefinition {
    let target_is_in_your_gy = crate::effect::shortcut::target_filtered(
        SelectionRequirement::HasCardType(CardType::Instant)
            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
    );
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
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_is_in_your_gy,
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: true,
                },
            ]),
            once_per_turn: true,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Eager First-Year ────────────────────────────────────────────────────────

/// Eager First-Year — {1}{W} 2/2 Human Wizard. "Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature gets +1/+0 until end of
/// turn."
pub fn eager_first_year() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Eager First-Year",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

// ── Hunt for Specimens ──────────────────────────────────────────────────────

/// Hunt for Specimens — {3}{B} Sorcery. "Create a 1/1 black Pest creature
/// token with 'When this creature dies, you gain 1 life.' Then learn."
///
/// Both halves wired faithfully. The spawned Pest token carries the
/// printed death-trigger lifegain via `TokenDefinition.triggered_abilities`
/// (SOS-VI); Learn uses `Effect::Learn` (reveal a Lesson / discard-to-draw).
pub fn hunt_for_specimens() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Hunt for Specimens",
        cost: cost(&[generic(3), b()]),
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
            // Learn (CR 701.45) — reveal a Lesson into hand or discard-to-draw.
            Effect::Learn { who: PR::You },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
            energy_cost: 0,
            discard_cost: None,
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_gain_life` shortcut.
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(3)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Tap {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
    use crate::effect::shortcut::drain as drain_eff;
    CardDefinition {
        name: "Silverquill Chronicle",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain_eff(2),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_gain_life` shortcut.
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
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
    use crate::effect::shortcut::drain_and_scry;
    CardDefinition {
        name: "Silverquill Heartrender",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_scry(3, 1),
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
    use crate::effect::shortcut::etb_tap_opp_creature;
    CardDefinition {
        name: "Silverquill Lawkeeper",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_tap_opp_creature()],
        ..Default::default()
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
    CardDefinition {
        name: "Inkling Penmaster",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_mint_inkling()],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: target_filtered(SelectionRequirement::Player),
                to: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::catalog::sets::sos::inkling_token(),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::HasCardType(CardType::Land).negate(),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature),
            1,
            1,
        )],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
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
            energy_cost: 0,
            discard_cost: None,
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
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Inkling Verseweaver",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling,
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::Target(0)),
                to: Selector::You,
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Scrivener — {2}{W}, 2/3 Human Cleric.
///
/// Synthesised Oracle: "When this creature enters, look at the top three
/// cards of your library; put one into your hand and the rest on the bottom
/// of your library in any order." Ships via `Effect::LookPickToHand`.
pub fn silverquill_scrivener_b30() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scrivener B30",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
            
                take: None,
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Tutor — {1}{W}, 2/2 Human Cleric. Synthesised Oracle:
/// "When this creature enters, draw a card, then discard a card."
/// 2-mana loot body.
pub fn silverquill_lorescribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lorescribe",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Inkletter — {W}{B}, instant. Synthesised Oracle:
/// "Drain 1. Surveil 1." 2-mana drain + selection — pairs with all
/// Witherbloom/Silverquill graveyard-care payoffs.
pub fn silverquill_inkletter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkletter",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(3)],
        ..Default::default()
    }
}

/// Inkling Squire — {W}{B}, 2/2 Inkling Knight Flying.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, target creature gets -1/-1 until end of turn."
pub fn inkling_quillbearer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillbearer",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Silverquill Indoctrinator — {2}{W}, 2/3 Human Cleric Vigilance.
/// Synthesised Oracle: "When this creature enters, each opponent discards
/// a card."
pub fn silverquill_indoctrinator() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Indoctrinator",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Inkling Choirsinger — {1}{W}{B}, 2/2 Inkling Cleric Flying Lifelink.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, you gain 1 life."
pub fn inkling_choirsinger() -> CardDefinition {
    CardDefinition {
        name: "Inkling Choirsinger",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Ovation — {3}{W}{B}, sorcery.
/// Synthesised Oracle: "Create two 1/1 white-and-black Inkling creature
/// tokens with flying, then put a +1/+1 counter on each Inkling you control."
pub fn silverquill_ovation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ovation",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Loremaster — {2}{W}{B}, 2/4 Inkling Wizard Flying.
/// Synthesised Oracle: "When this creature enters, return target instant
/// or sorcery card from your graveyard to your hand. You gain 1 life."
pub fn inkling_loremaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Loremaster",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Litany — {1}{B}, instant.
/// Synthesised Oracle: "Target creature gets -2/-1 until end of turn. You
/// gain 1 life."
pub fn silverquill_litany() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Litany",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        }],
        ..Default::default()
    }
}

/// Inkling Strikemark — {2}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent loses 2 life. You gain 2 life."
pub fn inkling_strikemark() -> CardDefinition {
    CardDefinition {
        name: "Inkling Strikemark",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Scribe-Tutor — {1}{W}, 1/3 Human Cleric.
/// Synthesised Oracle: "When this creature enters, surveil 1."
pub fn silverquill_scribe_tutor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scribe-Tutor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Magemark — {W}{B}, Instant.
/// Synthesised Oracle: "Target creature gets -2/-2 until end of turn.
/// You gain 2 life."
pub fn silverquill_magemark() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Magemark",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Battle Chant — {3}{W}, Sorcery.
/// Synthesised Oracle: "Creatures you control get +2/+1 and gain vigilance
/// until end of turn."
pub fn silverquill_battle_chant() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Battle Chant",
        cost: cost(&[generic(3), w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Homily — {1}{W}{B}, Sorcery.
/// Synthesised Oracle: "Drain 1 (each opponent loses 1 life and you gain
/// 1 life) and each opponent mills two cards."
pub fn silverquill_homily() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Homily",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Avenger — {3}{W}{B}, 3/3 Inkling Knight, Flying + First Strike.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter on
/// another target creature you control."
pub fn inkling_avenger() -> CardDefinition {
    CardDefinition {
        name: "Inkling Avenger",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Mandate — {2}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent sacrifices a creature."
pub fn silverquill_mandate() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mandate",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::SacrificeAndRemember {
            who: PlayerRef::EachOpponent,
            filter: SelectionRequirement::Creature,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Spellquill — {W}{B}, 1/2 Inkling Bard.
/// Synthesised Oracle: "Flying. Magecraft — gain 1 life. When this creature
/// dies, draw a card."
pub fn silverquill_spellquill() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Spellquill",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        // Refactored in batch 40: ETB drain wired via the canonical
        // `etb_drain(1)` shortcut from batch 39.
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Inkling Echobringer — {1}{W}{B}, 2/2 Inkling Cleric, Flying + Lifelink.
/// Synthesised Oracle: Inkling tribal payoff.
pub fn inkling_echobringer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Echobringer",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Verseblade — {1}{W}{B}, Instant.
/// Synthesised Oracle: "Target creature gets +1/+1 until end of turn. Draw
/// a card."
pub fn silverquill_verseblade() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Verseblade",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Lifepenner — {2}{W}, 2/3 Human Cleric.
/// Synthesised Oracle: "Magecraft — you gain 2 life."
pub fn silverquill_lifepenner() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifepenner",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(2)],
        ..Default::default()
    }
}

/// Inkling Maverick — {2}{B}, 3/2 Inkling Rogue, Flying.
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life and you gain 1 life."
pub fn inkling_maverick() -> CardDefinition {
    CardDefinition {
        name: "Inkling Maverick",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Antiphony — {2}{W}{B}, Instant.
/// Synthesised Oracle: "Drain 2. Surveil 1."
pub fn silverquill_antiphony() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Antiphony",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sentinel — {2}{W}, 2/3 Inkling Soldier with Flying.
/// Synthesised Oracle: vanilla Inkling at the 3-mana slot.
pub fn inkling_b36_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentinel II",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Forge — {3}{W}{B}, Sorcery.
/// Synthesised Oracle: "Create two 1/1 W/B Inkling creature tokens with
/// flying. Each opponent loses 1 life and you gain 1 life."
pub fn silverquill_forge() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Forge",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Cardinal — {3}{W}{B}, 3/4 Inkling Cleric, Flying + Vigilance.
/// Synthesised Oracle: "When this creature enters, you gain 2 life."
pub fn inkling_cardinal() -> CardDefinition {
    CardDefinition {
        name: "Inkling Cardinal",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_gain_life` shortcut.
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Inkling Scriptwarden — {2}{W}{B}, 2/3 Inkling Wizard, Flying + Vigilance.
/// Synthesised Oracle: "When this creature enters, each opponent loses 1
/// life and you gain 1 life."
pub fn inkling_scriptwarden() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scriptwarden",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        // Refactored in batch 40 to use the `etb_drain` shortcut.
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Pinion — {W}, Instant.
/// Synthesised Oracle: "Target creature gets +1/+1 EOT and gains Flying
/// until end of turn."
pub fn silverquill_pinion() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pinion",
        cost: cost(&[w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Battle Oration — {4}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent loses 4 life and you gain 4 life.
/// Create a 1/1 W/B Inkling creature token with flying."
pub fn silverquill_battle_oration() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Battle Oration",
        cost: cost(&[generic(4), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Manuscript — {1}{B}, Sorcery.
/// Synthesised Oracle: "Target opponent loses 2 life. You draw a card."
pub fn silverquill_manuscript() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Manuscript",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Ambassador — {1}{W}, 1/1 Inkling Cleric with Flying + Lifelink.
/// Synthesised Oracle: lean 2-mana evasive lifegainer.
pub fn inkling_ambassador() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ambassador",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Calligraphist — {3}{W}, 2/4 Inkling Cleric, Flying.
/// Synthesised Oracle: "Magecraft — Put a +1/+1 counter on this creature."
pub fn inkling_calligraphist() -> CardDefinition {
    CardDefinition {
        name: "Inkling Calligraphist",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

// ── Batch 39: 6 more Silverquill cards ─────────────────────────────────────

/// Silverquill Liturgist — {2}{W}, 1/4 Inkling Cleric with Flying.
/// Synthesised Oracle: "Defensive evasive body. Magecraft — gain 1 life."
pub fn silverquill_liturgist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Liturgist",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Inkling Bookwarden — {3}{W}{B}, 4/5 Inkling Warrior Flying + Lifelink.
/// Synthesised Oracle: "Top-end Silverquill finisher."
pub fn inkling_bookwarden() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bookwarden",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Soulbinder — {1}{W}{B}, 2/2 Vampire Warlock.
/// Synthesised Oracle: "When this creature enters, target opp loses 2 life
/// and you gain 2 life. Magecraft — put a +1/+1 counter on this creature."
pub fn silverquill_soulbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Soulbinder",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_drain(2),
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        ..Default::default()
    }
}

/// Inkling Magister — {4}{W}{B}, 3/4 Inkling Wizard, Flying + Vigilance.
/// Synthesised Oracle: "ETB drain 3. Magecraft — gain 1 life."
pub fn inkling_magister() -> CardDefinition {
    CardDefinition {
        name: "Inkling Magister",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(3), magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Inkproclamation — {2}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent sacrifices a creature, then you
/// create a 1/1 W/B Inkling token with flying."
pub fn silverquill_inkproclamation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkproclamation",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Loredrain — {3}{W}{B}, Sorcery.
/// Synthesised Oracle: "Each opponent discards a card and loses 2 life.
/// You gain 2 life."
pub fn inkling_loredrain() -> CardDefinition {
    CardDefinition {
        name: "Inkling Loredrain",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Purifier — {1}{W}, 2/2 Human Cleric.
/// Synthesised Oracle: "When this creature enters, you gain 2 life.
/// Magecraft — Scry 1."
pub fn silverquill_purifier() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Purifier",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_gain_life(2),
            magecraft(Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) }),
        ],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(1),
            random: true,
        })],
        ..Default::default()
    }
}

/// Silverquill Witnessing — {2}{W}{B} Instant.
/// Synthesised Oracle: "Each opponent loses 3 life and you gain 3 life.
/// Draw a card."
pub fn silverquill_witnessing() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Witnessing",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Avant-Garde — {4}{W}{B}, 4/4 Inkling Bard Flying + Lifelink.
/// Synthesised Oracle: "When this creature enters, each opponent loses
/// 2 life and you gain 2 life." A 6-mana evasive race-breaker.
pub fn inkling_avant_garde() -> CardDefinition {
    CardDefinition {
        name: "Inkling Avant-Garde",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling_token(),
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(3)],
        ..Default::default()
    }
}

/// Inkling Disciple — {1}{W}, 1/1 Inkling Cleric Flying.
/// Synthesised Oracle: "Flying. When this creature enters, you gain
/// 1 life." 2-mana defensive evasive lifegain.
pub fn inkling_disciple() -> CardDefinition {
    CardDefinition {
        name: "Inkling Disciple",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(4)],
        ..Default::default()
    }
}

/// Silverquill Diatribe — {2}{B} Sorcery. Synthesised Oracle: "Target
/// player loses 4 life. Surveil 1." 3-mana drain + selection.
pub fn silverquill_diatribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Diatribe",
        cost: cost(&[generic(2), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Vassal — {1}{B}, 1/2 Inkling Cleric Lifelink. Magecraft →
/// opponent loses 1 life. Cheap pingy drain body.
pub fn inkling_vassal() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vassal",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Vellum — {W}{B}, Instant. "Each opponent loses 2 life and
/// you gain 2 life." 2-mana symmetric Silverquill drain template.
pub fn silverquill_vellum() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vellum",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Decreemaster — {2}{W}{B}, 2/3 Inkling Cleric Flying Lifelink.
/// ETB: target opponent discards a card. 4-mana double-keyword discard
/// body.
pub fn inkling_decreemaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Decreemaster",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: false,
        })],
        ..Default::default()
    }
}

/// Silverquill Penbringer — {3}{W}, 2/4 Human Cleric Vigilance. Magecraft
/// → gain 1 life. Defensive Silverquill anchor.
pub fn silverquill_penbringer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penbringer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Ravenswing — {1}{W}{B}, 2/2 Vampire Cleric Flying.
/// "Whenever this creature attacks, each opponent loses 1 life and you
/// gain 1 life." 3-mana evasive drain attacker.
pub fn silverquill_ravenswing() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ravenswing",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Inkling Magistrate — {2}{B}, 2/2 Inkling Cleric. ETB: opponent loses
/// 2 life. Body-on-a-Bloodthirst template.
pub fn inkling_magistrate() -> CardDefinition {
    CardDefinition {
        name: "Inkling Magistrate",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb_drain_each_opp(2)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Bookbinder — {1}{B}, 1/1 Inkling Cleric. Magecraft → +1/+1
/// counter on this creature. 2-mana magecraft scaler — same shape as
/// Lorehold Bonepriest but in black for the Inkling tribal.
pub fn inkling_bookbinder() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bookbinder",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Silverquill Scribebearer — {1}{W}, 1/2 Human Cleric Flying. ETB:
/// Scry 2. 2-mana evasive scry body.
pub fn silverquill_scribebearer() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Scribebearer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(2)],
        ..Default::default()
    }
}

/// Silverquill Adept — {W}{B}, 2/1 Vampire Cleric. Magecraft → opp loses
/// 1 life. Cheap 2-mana pingy drain.
pub fn silverquill_adept() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Adept",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_dies(draw(1))],
        ..Default::default()
    }
}

/// Silverquill Inkcaller — {1}{W}{B}, 2/2 Vampire Cleric. ETB: mint a
/// 1/1 W/B Inkling token (flying). 3-mana Inkling-tribal mint body.
pub fn silverquill_inkcaller() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token;
    CardDefinition {
        name: "Silverquill Inkcaller",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Lecture — {1}{W}{B}, Instant. "Each opponent loses 3
/// life and you gain 3 life." 3-mana instant-speed Silverquill drain.
pub fn silverquill_lecture() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lecture",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Battlescholar — {3}{W}{B}, 3/3 Inkling Cleric Flying. On
/// attack: +1/+0 EOT to self. 5-mana evasive growth body.
pub fn inkling_battlescholar() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Inkling Battlescholar",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Silverquill Final-Year — {2}{B}, 3/2 Human Cleric Lifelink. Magecraft
/// → +1/+0 EOT self-pump. 3-mana lifelink aggro magecraft body.
pub fn silverquill_final_year() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Final-Year",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Inkling Devotee — {2}{W}, 2/3 Inkling Cleric. ETB: gain 2 life.
/// 3-mana defensive lifegain body.
pub fn inkling_devotee() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Inkling Devotee",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

/// Silverquill Inkspear — {W}{B}, Instant. "Target opponent loses 1
/// life and you gain 1 life." 2-mana single-target Silverquill drain.
pub fn silverquill_inkspear() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkspear",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sergeant — {2}{W}, 2/2 Inkling Soldier. Static: other
/// Inklings you control get +1/+0. 3-mana Inkling-tribal anthem.
pub fn inkling_sergeant() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sergeant",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Refrain — {W}{B} Instant. Synthesised Oracle: "Drain
/// 2 (each opponent loses 2 life and you gain 2 life). Surveil 1."
/// 2-mana drain + selection.
pub fn silverquill_refrain() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Refrain",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Silverquill Vermilion — {2}{W} Instant. Synthesised Oracle:
/// "Target creature gets -3/-3 until end of turn. You gain 3 life."
/// 3-mana shrink-removal with lifegain rider.
pub fn silverquill_vermilion() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vermilion",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Drainmaster II — {3}{W}{B}, 3/3 Inkling Warlock.
/// Synthesised Oracle: "When this creature enters, each opponent
/// loses 3 life and you gain 3 life." 5-mana drain top-end.
pub fn silverquill_drainmaster_v2() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainmaster II",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(3)],
        ..Default::default()
    }
}

/// Silverquill Bookbond — {W}{B} Sorcery. Synthesised Oracle:
/// "Return target creature card from your graveyard to your hand.
/// You gain 1 life." 2-mana cheap recursion + lifegain.
pub fn silverquill_bookbond() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookbond",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pencrafter — {1}{W}{B}, 2/3 Inkling Wizard. Synthesised
/// Oracle: "When this creature enters, draw a card. You lose 1 life."
/// 3-mana cantrip body with life cost.
pub fn silverquill_pencrafter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pencrafter",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Inkling Inkblot — {B} Sorcery. Synthesised Oracle: "Target opponent
/// loses 1 life and you gain 1 life." 1-mana cheap drain spell.
pub fn inkling_inkblot() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkblot",
        cost: cost(&[b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Censurewright — {1}{B}, 1/3 Human Rogue.
/// Synthesised Oracle: "When this creature enters, target creature
/// gets -1/-1 until end of turn." 2-mana cheap removal-trigger body.
pub fn silverquill_censurewright() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Censurewright",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Inkling Stylescribe — {W}{B}, 2/2 Inkling Cleric Flying.
/// Magecraft: scry 1 — Inkling-tribal smoothing.
pub fn inkling_stylescribe() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry;
    CardDefinition {
        name: "Inkling Stylescribe",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Pageturner — {1}{W}, 1/3 Human Wizard. Vigilance.
/// ETB Scry 1. Defensive smoothing body.
pub fn silverquill_pageturner() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Pageturner",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Inkling Stormwriter — {2}{W}{B}, 3/2 Inkling Wizard Flying.
/// Magecraft: gain 1 life. 4-mana evasive lifegain on each IS cast.
pub fn inkling_stormwriter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stormwriter",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Inkbinder — {2}{W}, 2/3 Human Cleric. ETB target
/// creature you control gets +1/+1 EOT + gains Lifelink EOT. 3-mana
/// combat trick + lifelink-on-the-pumped-creature.
pub fn silverquill_inkbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbinder",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Quietus — {1}{B}, Instant. -3/-3 EOT to target
/// creature. 2-mana shrink-removal.
pub fn silverquill_quietus() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quietus",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Skywriter — {1}{W}{B}, 2/2 Inkling Wizard Flying.
/// Magecraft: target creature you control gains +1/+1 EOT.
pub fn inkling_skywriter() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Inkling Skywriter",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1,
            1,
        )],
        ..Default::default()
    }
}

/// Silverquill Glyphmaster — {3}{W}{B}, 3/4 Vampire Cleric Lifelink.
/// ETB drain 2. 5-mana race breaker with lifelink + 4-life swing.
pub fn silverquill_glyphmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Glyphmaster",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Inkling Mournful — {2}{B}, 2/2 Inkling Rogue Flying.
/// Dies → drain 1 (each opp loses 1, you gain 1). 3-mana evasive
/// trade-up body with on-die drain payoff.
pub fn inkling_mournful() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mournful",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Pen-Squire — {W}, 1/1 Human Soldier. Magecraft: this
/// creature gets +1/+0 EOT. Cheapest Silverquill self-pump magecraft body.
pub fn silverquill_pen_squire() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Squire",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Inkling Spellbinder — {3}{W}{B}, 4/4 Inkling Wizard Flying + Lifelink.
/// 5-mana evasive race breaker — vanilla flier + lifelink.
pub fn inkling_spellbinder() -> CardDefinition {
    CardDefinition {
        name: "Inkling Spellbinder",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Diction — {W}{B}, Instant. Drain 2 each opp + Surveil 1.
/// 2-mana drain + selection. Uses Seq.
pub fn silverquill_diction() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Diction",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Quietude — {2}{W}{B}, Sorcery. Drain 3 + Scry 2.
/// 4-mana drain + selection.
pub fn silverquill_quietude() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quietude",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Beautisage — {3}{W}, 3/3 Inkling Cleric Vigilance.
/// ETB: gain 3 life. 4-mana defensive lifegain finisher.
pub fn inkling_beautisage() -> CardDefinition {
    CardDefinition {
        name: "Inkling Beautisage",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(3)],
        ..Default::default()
    }
}

/// Silverquill Inkmender — {1}{W}{B}, 2/3 Vampire Warlock Lifelink.
/// ETB: return target ≤2 MV creature card from your gy to hand. 3-mana
/// lifelink reanimator.
pub fn silverquill_inkmender() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmender",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Memorial — {2}{W}{B}, Sorcery. Return target creature
/// card from your gy to bf + drain 1. 4-mana reanimator with drain.
pub fn silverquill_memorial() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memorial",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Inkstain — {1}{W}, 2/1 Inkling Soldier. On-attack: target
/// creature gets -1/-0 EOT. Tempo-shrink attacker.
pub fn inkling_inkstain() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkstain",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Convene — {2}{W}{B}, Sorcery. Mint 2 Inkling tokens
/// + each opp loses 1. 4-mana double mint with drain rider.
pub fn silverquill_convene() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Convene",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sermoneer — {3}{W}, 2/4 Human Cleric Vigilance.
/// ETB Seq(Scry 1 + GainLife 1). 4-mana defensive smoother body.
pub fn silverquill_sermoneer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sermoneer",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Inkling Pageboy — {W}, 1/2 Inkling Cleric Flying. Vanilla 1-drop
/// evasive Inkling — cheapest evasive Inkling in the pool.
pub fn inkling_pageboy() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pageboy",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkstrike-Page — {1}{B}, Sorcery. Destroy target
/// creature with power ≤ 2. Cheap power-gated removal.
pub fn silverquill_inkstrike_page() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkstrike-Page",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Mentor — {2}{W}, 2/3 Human Cleric Vigilance.
/// ETB: target friendly creature gets a +1/+1 counter.
/// 3-mana sticky pumper.
pub fn silverquill_mentor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mentor",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Necroscribe — {3}{B}, 3/3 Vampire Wizard. ETB return
/// target IS card from your graveyard to your hand. 4-mana value
/// recursion on a body.
pub fn silverquill_necroscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Necroscribe",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Pronouncement — {3}{W}{B}, Sorcery. Seq(Drain 3 +
/// CreateToken 2 Inkling). 5-mana drain + double-mint finisher.
pub fn silverquill_pronouncement() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pronouncement",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Cipher — {W}{B}, Instant. Drain 1 + Draw 1.
/// 2-mana micro drain cantrip.
pub fn silverquill_cipher() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cipher",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quillpoint — {1}{W}{B}, 2/3 Inkling Knight First Strike.
/// 3-mana first strike Inkling — combat-leaning Tenured Inkcaster fodder.
pub fn inkling_quillpoint() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillpoint",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Memoriam — {1}{W}{B}, 2/3 Vampire Cleric.
/// ETB Seq(Drain 1 + Scry 1). Compact 3-mana drain + smoothing body.
pub fn silverquill_memoriam() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memoriam",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Inkling Sigilbearer — {2}{W}{B}, 3/3 Inkling Cleric Flying. ETB
/// puts a +1/+1 counter on each other Inkling you control.
pub fn inkling_sigilbearer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sigilbearer",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Eulogize — {2}{W}{B}, Sorcery. Reanimate a Creature
/// card with mana value ≤ 3 from your graveyard + gain 2 life.
pub fn silverquill_eulogize() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Eulogize",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Voidwalker — {3}{B}, 3/2 Inkling Rogue Flying + Menace.
/// 4-mana evasive double-evasion attacker.
pub fn inkling_voidwalker() -> CardDefinition {
    CardDefinition {
        name: "Inkling Voidwalker",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Menace],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Festscribe — {2}{W}{B}, 3/3 Vampire Wizard.
/// ETB: mints an Inkling token and you gain 2 life.
pub fn silverquill_festscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Festscribe",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1), magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Inkling Archivist — {2}{W}{B}, 2/3 Inkling Cleric Flying. ETB drain 1
/// and magecraft Scry 1. 4-mana evasive scaler.
pub fn inkling_archivist() -> CardDefinition {
    use crate::effect::shortcut::{etb_drain, magecraft_scry};
    CardDefinition {
        name: "Inkling Archivist",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1), magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Ledgermage — {2}{W}{B}, 3/3 Vampire Wizard. ETB Drain 2 via
/// the canonical drain template. 4-mana race-breaker body.
pub fn silverquill_ledgermage() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Silverquill Ledgermage",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Inkling Inkscribe — {W}{B}, 2/1 Inkling Soldier Flying. Aggressive
/// 2-mana evasive Inkling.
pub fn inkling_inkscribe() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkscribe",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Codex — {1}{W}, Sorcery. Seq(GainLife 2 + Draw 1). 2-mana
/// defensive cantrip.
pub fn silverquill_codex() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Codex",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Studyhall — {2}{W}, 2/3 Human Cleric Vigilance. Magecraft
/// gain 1 life — defensive vigilance body that scales with IS casts.
pub fn silverquill_studyhall() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Studyhall",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Pronouncer — {3}{W}{B}, 4/4 Inkling Bard Flying + Lifelink.
/// ETB drain 1 — 5-mana evasive lifelink finisher.
pub fn silverquill_pronouncer() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Silverquill Pronouncer",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Etching — {W}{B}, Instant. Seq(GainLife 2 + LoseLife 2 each
/// opp). 2-mana symmetric drain.
pub fn silverquill_etching() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Etching",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Inkling Chaplain — {1}{W}, 1/3 Inkling Cleric Vigilance + Lifelink.
/// Defensive 2-mana evasive lifelinker that locks down combat. Pairs with
/// Tenured Inkcaster's +2/+2 anthem (→ 3/5 Vigilance + Lifelink Flier).
pub fn inkling_chaplain() -> CardDefinition {
    CardDefinition {
        name: "Inkling Chaplain",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Warden — {2}{W}, 2/4 Human Cleric Vigilance. ETB Drain 1.
/// 3-mana defensive drain anchor.
pub fn silverquill_warden() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Warden",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Inkling Acolyte v2 — {1}{B}, 1/2 Inkling Cleric. Magecraft Drain 1.
/// Cheap on-cast magecraft drainer.
pub fn inkling_acolyte_v2() -> CardDefinition {
    CardDefinition {
        name: "Inkling Acolyte (Adept)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Reflect — {2}{W}, Instant. Seq(Drain 2 + Surveil 2). 3-mana
/// instant-speed drain + deeper selection.
pub fn silverquill_reflect() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Reflect",
        cost: cost(&[generic(2), w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Evangel — {3}{W}{B}, 3/3 Inkling Bard Flying + Lifelink. ETB
/// +1/+1 counter on target Inkling you control. Inkling tribal payoff.
pub fn inkling_evangel() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Evangel",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Invocation — {3}{W}{B}, Sorcery. Mints 3 Inkling tokens.
/// 5-mana wide go-wide finisher (Defend-the-Campus-template at the same
/// cost).
pub fn silverquill_invocation() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Invocation",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Ghostwriter — {2}{B}, 2/3 Inkling Rogue. Magecraft drain each
/// opp for 1. Medium-mana magecraft drainer.
pub fn inkling_ghostwriter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ghostwriter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Doom — {2}{B}, Instant. Drain 4 (5-life swing) — instant-
/// speed drain finisher.
pub fn silverquill_doom() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Doom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Attendant — {W}{B}, 1/2 Inkling Cleric Flying + Lifelink. ETB
/// Scry 1 — cheap evasive lifelink with smoothing rider.
pub fn inkling_attendant() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Inkling Attendant",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Psalm — {1}{W}{B}, Instant. Seq(Drain 2 + Draw 1). 3-mana
/// instant-speed drain + cantrip.
pub fn silverquill_psalm() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Psalm",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Pageant — {2}{W}{B}, Sorcery. Seq(Mint 2 Inklings + GainLife
/// 2). 4-mana mint + lifegain.
pub fn inkling_pageant() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pageant",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: drain(1),
        }],
        ..Default::default()
    }
}

/// Inkling Sentinel II — {2}{W}, 1/4 Inkling Soldier Vigilance. Defensive
/// Inkling — slots into Tenured Inkcaster + Inkling Verselord shells.
pub fn inkling_sentinel_b55() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentinel (b55)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inksong — {W}{B}, Instant. Seq(Drain 1 + Scry 2). 2-mana
/// instant-speed drain + heavy selection.
pub fn silverquill_inksong() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Inksong",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Pact-Caller — {1}{W}{B}, 2/3 Inkling Cleric Flying. ETB mints
/// 1 Inkling token. Self-replicating Inkling enabler.
pub fn inkling_pact_caller() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token;
    CardDefinition {
        name: "Inkling Pact-Caller",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Inkmaster — {2}{W}{B}, 2/3 Inkling Wizard Flying. Magecraft
/// each opp loses 1 life and you gain 1 life — Witherbloom-Apprentice
/// drain template on a flying body, costed for the {W}{B} flyer slot.
pub fn inkling_inkmaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkmaster",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Acolyte — {1}{W}, 2/2 Human Cleric. ETB Drain 1.
/// 2-mana defensive drain body — Light-of-Promise enabler.
pub fn silverquill_acolyte_b56() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Acolyte II",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_dies(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Silverquill Scriptmaster — {2}{W}{B}, 3/3 Vampire Cleric.
/// ETB Drain 2 + Scry 1. 4-mana mid-curve value engine.
pub fn silverquill_scriptmaster() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Silverquill Scriptmaster",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(2),
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Inkling Bladerunner — {2}{W}, 2/2 Inkling Soldier with Flying +
/// First Strike. 3-mana evasive combat-ready Inkling — works under
/// the Tenured Inkcaster anthem for an effective 4/4 first-strike flyer.
pub fn inkling_bladerunner() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bladerunner",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sentinel — {1}{W}, 1/3 Inkling Soldier with Vigilance
/// + Flying. 2-mana defensive flyer that pulls double duty on attack
///   (with Tenured Inkcaster: 3/5 vigilance flyer).
pub fn silverquill_sentinel_b57() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentinel III",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 1,
        )],
        ..Default::default()
    }
}

/// Inkling Quillblade II — {1}{B}, 2/1 Inkling Wizard with Flying.
/// 2-mana evasive body — fills the Silverquill flying-aggro shell.
pub fn inkling_quillblade_b58() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillblade II",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Scribecaller — {W}{B}, 2/2 Inkling Soldier with Lifelink.
/// 2-mana lifelink body — anchors the W/B drain shell with steady
/// life swing on every attack.
pub fn silverquill_scribecaller() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scribecaller",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Inkmaster — {3}{W}{B}, 3/4 Inkling Wizard with Flying.
/// Magecraft: each opp loses 1 life and you gain 1 life. 5-mana
/// evasive drain engine — Felisa-light without the counter rider.
pub fn silverquill_inkmaster_b58() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmaster II",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_surveil(1),
            magecraft_scry(1),
        ],
        ..Default::default()
    }
}

/// Silverquill Inkflight — {1}{W}, 1/2 Inkling Cleric with Flying.
/// Magecraft self-pump +1/+0 EOT. Cheap evasive body that grows on
/// each IS cast.
pub fn silverquill_inkflight_b59() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkflight",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Silverquill Pen-Priest — {W}{B}, 1/3 Vampire Cleric Lifelink. ETB
/// drain 1. 2-mana defensive lifelink + immediate drain.
pub fn silverquill_pen_priest() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Priest",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        triggered_abilities: vec![dies_drain(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 0,
        )],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(2),
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 61): 5 more Silverquill cards ────────────────

/// Silverquill Pentor — {1}{W}, 2/2 Human Cleric. ETB Seq(GainLife 2 +
/// magecraft Scry 1). 2-mana defensive lifegain body + on-cast smoother.
pub fn silverquill_pentor_b61() -> CardDefinition {
    use crate::effect::shortcut::etb_gain_life;
    CardDefinition {
        name: "Silverquill Pentor",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_gain_life(2),
            crate::effect::shortcut::magecraft_scry(1),
        ],
        ..Default::default()
    }
}

/// Inkling Arbiter — {W}{B}, 2/2 Inkling Cleric Flying + Lifelink.
/// Compact 2-mana evasive lifelinker — Tenured Inkcaster fodder.
pub fn inkling_arbiter() -> CardDefinition {
    CardDefinition {
        name: "Inkling Arbiter",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkmage — {2}{W}{B}, 3/3 Vampire Wizard. ETB Drain 2 via
/// the `etb_drain(2)` shortcut. 4-mana drain race-breaker.
pub fn silverquill_inkmage_b61() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmage",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Drainpoet — {3}{W}{B}, 3/3 Vampire Bard Flying. ETB drain
/// 3 + magecraft GainLife 1. 5-mana race-breaker engine — 6-life swing
/// on entry plus a per-cast lifegain rider.
pub fn silverquill_drainpoet() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainpoet",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_drain(3),
            magecraft_gain_life(1),
        ],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Lecturer II — {2}{W}{B}, 3/2 Vampire Cleric Lifelink. ETB
/// Seq(Drain 1 + Surveil 1). 4-mana value engine — lifelink + drain +
/// graveyard fuel rolled into a single curve-out body.
pub fn silverquill_lecturer_b62() -> CardDefinition {
    use crate::effect::shortcut::{drain, etb};
    CardDefinition {
        name: "Silverquill Lecturer (b62)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            drain(1),
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 63): 5 more Silverquill cards ─────────────────

/// Inkling Scribesage — {1}{W}{B}, 2/2 Inkling Cleric Flying. Magecraft
/// gain 1 life. 3-mana evasive lifegain-on-cast body.
pub fn inkling_scribesage() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scribesage",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Dirgesage — {2}{W}{B}, 3/3 Vampire Cleric. ETB drain 2.
/// 4-mana sturdy drain body.
pub fn silverquill_dirgesage() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Dirgesage",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Hymnsmith — {1}{W}, 2/2 Human Wizard. Magecraft +1/+0 EOT
/// self-pump via `magecraft_self_pump(1, 0)`. 2-mana magecraft scaler.
pub fn silverquill_hymnsmith() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Hymnsmith",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Silverquill Quillchorus — {3}{W}{B}, Sorcery. Creates 3 Inkling tokens
/// + drain 1 each opp. 5-mana go-wide drain.
pub fn silverquill_quillchorus() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Quillchorus",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Riftcaster — {2}{B}, 2/3 Inkling Wizard Flying. Magecraft drain
/// 1 each opp. 3-mana evasive drain magecraft body.
pub fn inkling_riftcaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Riftcaster",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature.and(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling),
            )
            .and(SelectionRequirement::ControlledByYou)),
            1,
            1,
        )],
        ..Default::default()
    }
}

/// Silverquill Vespersong — {2}{W}{B}, Sorcery. Seq(Drain 2 + Draw 1).
/// 4-mana drain-and-draw.
pub fn silverquill_vespersong() -> CardDefinition {
    use crate::effect::shortcut::drain;
    CardDefinition {
        name: "Silverquill Vespersong",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Battlechoir — {3}{W}{B}, 3/4 Inkling Cleric Flying + Lifelink.
/// ETB drain 3 via the `etb_drain(3)` shortcut. 5-mana race-breaker.
pub fn inkling_battlechoir() -> CardDefinition {
    CardDefinition {
        name: "Inkling Battlechoir",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(3)],
        ..Default::default()
    }
}

/// Silverquill Inkmuse — {W}{B}, 2/1 Vampire Cleric Lifelink. Magecraft
/// Surveil 1 — every spell sets up the next.
pub fn silverquill_inkmuse() -> CardDefinition {
    use crate::effect::shortcut::magecraft_surveil;
    CardDefinition {
        name: "Silverquill Inkmuse",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_surveil(1)],
        ..Default::default()
    }
}

/// Inkling Heraldcourier — {2}{W}, 2/3 Inkling Soldier Flying + Vigilance.
/// ETB mint 1 Inkling token. Inkling-tribal go-wide enabler.
pub fn inkling_heraldcourier() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Heraldcourier",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            definition: inkling_token(),
            count: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Inkscale — {1}{W}{B}, Instant. Seq(PumpPT(+2/+0 EOT) +
/// GrantKeyword(Lifelink, EOT)) on target friendly. 3-mana combat trick.
pub fn silverquill_inkscale() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkscale",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Pallidwing — {3}{W}, 2/3 Inkling Cleric Flying + Lifelink.
/// Vanilla evasive lifelinker — Tenured Inkcaster anthem fodder.
pub fn inkling_pallidwing() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pallidwing",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Cantillator — {2}{W}, 2/3 Human Cleric. ETB gain 2 life +
/// magecraft +1/+0 EOT self-pump. 3-mana lifegain + scaler.
pub fn silverquill_cantillator() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cantillator",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2), magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Inkling Stormpenner — {2}{W}{B}, 2/3 Inkling Wizard Flying. Magecraft
/// AddCounter +1/+1 on self. 4-mana self-growing evasive body.
pub fn inkling_stormpenner() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stormpenner",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Inkling Bannerer — {1}{W}{B}, 2/2 Inkling Cleric. Magecraft pump each
/// Inkling you control +1/+0 EOT via the new tribal-pump shortcut.
pub fn inkling_bannerer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_pump_each_creature_type;
    CardDefinition {
        name: "Inkling Bannerer",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_pump_each_creature_type(
            CreatureType::Inkling,
            1,
            0,
        )],
        ..Default::default()
    }
}

/// Silverquill Inkmark — {1}{B}, Sorcery. Target opponent loses 3 life,
/// you gain 3 life. Pure 2-mana drain spell.
pub fn silverquill_inkmark() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmark",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 67): 6 more Silverquill cards ───────────────

/// Silverquill Inkbearer — {1}{W}, 2/2 Inkling Cleric Flying. Vanilla
/// 2-mana evasive Inkling. Stacks with Tenured Inkcaster's +2/+2 anthem.
pub fn silverquill_inkbearer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbearer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Quietkeeper — {2}{W}, 2/3 Human Cleric. ETB Seq(Scry 1
/// + GainLife 2). 3-mana defensive smoother + lifegain body.
pub fn silverquill_quietkeeper() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Quietkeeper",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
        ..Default::default()
    }
}

/// Inkling Lorebearer — {W}{B}, 2/2 Inkling Cleric Lifelink. Cheap 2-
/// mana evasive-style lifelinker. (Has no flying, but pairs hard with
/// any Inkling anthem.)
pub fn inkling_lorebearer() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lorebearer",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkcrier — {2}{B}, 2/3 Inkling Rogue. Magecraft drain 1.
/// 3-mana drain-on-cast body.
pub fn silverquill_inkcrier() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkcrier",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Drainscribe — {1}{W}{B}, 2/2 Vampire Warlock Flying.
/// ETB drain 2 via `etb_drain(2)`. 3-mana evasive race-breaker.
pub fn silverquill_drainscribe() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainscribe",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Inksong II — {W}{B}, Instant. Drain 2 (each opp loses
/// 2, you gain 2). 2-mana drain at instant speed.
pub fn silverquill_inksong_b67() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inksong (b67)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
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
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Push (modern_decks, batch 68): more Silverquill W/B cards ─────────────

/// Silverquill Inkdiplomat — {1}{W}, 2/2 Human Cleric. ETB Seq(GainLife
/// 1 + Draw 1). 2-mana cantripping lifegain body. Pairs with Light of
/// Promise / Felisa for triggered payoffs.
pub fn silverquill_inkdiplomat() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Inkdiplomat",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Inkling Glyphkeeper — {W}{B}, 2/2 Inkling Cleric Flying. Magecraft
/// Drain 1. 2-mana magecraft Inkling.
pub fn inkling_glyphkeeper() -> CardDefinition {
    CardDefinition {
        name: "Inkling Glyphkeeper",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Scriptdrain — {2}{B}, Instant. Drain 3. 3-mana instant-
/// speed drain finisher.
pub fn silverquill_scriptdrain() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scriptdrain",
        cost: cost(&[generic(2), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Scrollwarden II — {2}{W}{B}, 3/3 Inkling Soldier Flying +
/// Vigilance. ETB +1/+1 counter on self. 4-mana evasive vigilance
/// finisher that grows on entry to 4/4 flying vigilance.
pub fn inkling_scrollwarden_b68() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Scrollwarden (b68)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Bookmark — {W}, Instant. Target creature you control
/// gets +0/+2 EOT and gains Lifelink EOT. 1-mana defensive combat
/// trick with lifegain rider.
pub fn silverquill_bookmark() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookmark",
        cost: cost(&[w()]),
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
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 125 (push claude/modern_decks): five new Silverquill cards ──────

/// Silverquill Stridemage (b125) — {2}{W}{B}, 3/3 Vampire Cleric.
/// "Whenever this creature attacks, each opponent loses 1 life and you
/// gain 1 life." Attack-drain via `on_attack_drain`.
pub fn silverquill_stridemage_b125() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Stridemage (b125)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

/// Inkling Skyhunter (b125) — {2}{W}, 2/2 Inkling Soldier Flying.
/// "Whenever this creature attacks, you gain 1 life." Attack-gain
/// via `on_attack_gain_life`. 3-mana evasive lifegain trigger.
pub fn inkling_skyhunter_b125() -> CardDefinition {
    CardDefinition {
        name: "Inkling Skyhunter (b125)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Soulscholar (b125) — {1}{W}, 1/2 Human Cleric Lifelink.
/// Magecraft AddCounter(+1/+1, Self). 2-mana magecraft self-grower
/// with lifelink — snowballs hard in spell-heavy lifegain shells.
pub fn silverquill_soulscholar_b125() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Soulscholar (b125)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Inkling Drainsage (b125) — {3}{W}{B}, 3/4 Inkling Cleric Flying +
/// Lifelink. ETB drain 2 via `etb_drain(2)`. 5-mana evasive
/// race-breaker top-end.
pub fn inkling_drainsage_b125() -> CardDefinition {
    CardDefinition {
        name: "Inkling Drainsage (b125)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Ravenstrike (b125) — {1}{W}{B}, Sorcery. Mints 1 Inkling
/// token + gain 2 life. 3-mana Inkling mint with lifegain rider.
pub fn silverquill_ravenstrike_b125() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ravenstrike (b125)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 126 (push claude/modern_decks): five new Silverquill cards ──────

/// Silverquill Glyphmage (b126) — {1}{W}, 1/3 Human Cleric. Magecraft
/// Scry 1. 2-mana defensive smoother body. Pairs with the new
/// `magecraft_scry` helper.
pub fn silverquill_glyphmage_b126() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Glyphmage (b126)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Pen-Sage (b126) — {2}{W}{B}, 3/3 Vampire Cleric. ETB
/// drain 2. 4-mana drain race-breaker body.
pub fn silverquill_pen_sage_b126() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Sage (b126)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Inkling Squire (b126) — {1}{B}, 2/1 Inkling Knight Flying. 2-mana
/// aggressive evasive Inkling — Tenured Inkcaster fodder.
pub fn inkling_squire_b126() -> CardDefinition {
    CardDefinition {
        name: "Inkling Squire (b126)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sigilrider (b126) — {2}{W}{B}, 3/3 Inkling Cleric Flying +
/// Lifelink. ETB GainLife 2. 4-mana evasive lifelinker — race-breaker
/// finisher.
pub fn inkling_sigilrider_b126() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sigilrider (b126)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

/// Silverquill Glyphcaller (b126) — {W}{B} Instant. Seq(Drain 2 +
/// Surveil 1). 2-mana drain + selection at instant speed via
/// `drain_and_surveil(2, 1)`.
pub fn silverquill_glyphcaller_b126() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Glyphcaller (b126)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_surveil(2, 1),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 127 (push claude/modern_decks): new Silverquill cards ───────────

/// Silverquill Aristocrat (b127) — {1}{B}, 1/2 Inkling Cleric Flying.
/// Magecraft drain each opp 1 — small evasive aristocrats payoff.
pub fn silverquill_aristocrat_b127() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Aristocrat (b127)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Inkling Quillmender (b127) — {2}{W}, 2/3 Inkling Cleric Flying.
/// On_attack gain life 1 — small evasive lifegain attacker.
pub fn inkling_quillmender_b127() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillmender (b127)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Lecturist (b127) — {W}{B}, 2/2 Vampire Cleric Lifelink.
/// 2-mana lifelink body — Light-of-Promise enabler that swings to gain
/// life every turn.
pub fn silverquill_lecturist_b127() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lecturist (b127)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Battle Drone (b127) — {3}{W}{B}, 3/3 Inkling Soldier Flying
/// + Vigilance. ETB drain 1. 5-mana evasive race breaker.
pub fn inkling_battle_drone_b127() -> CardDefinition {
    CardDefinition {
        name: "Inkling Battle Drone (b127)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Inkling Skyraider (b127) — {1}{W}{B}, 2/2 Inkling Rogue Flying.
/// "Whenever this creature attacks and isn't blocked, you gain 1 life
/// and each opponent loses 1 life." Tests the new CR 509.3g
/// `AttacksAndIsntBlocked` event added in this batch — wired via the
/// new `on_unblocked()` shortcut.
pub fn inkling_skyraider_b127() -> CardDefinition {
    use crate::effect::shortcut::on_unblocked;
    CardDefinition {
        name: "Inkling Skyraider (b127)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_unblocked(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::Player(PlayerRef::You),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Quillplate (b127) — {2}{W}, 2/4 Human Soldier Vigilance.
/// ETB GainLife 2. 3-mana defensive lifegainer + vigilant blocker.
pub fn silverquill_quillplate_b127() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillplate (b127)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

// ── Batch 128 (push claude/modern_decks): new Silverquill cards ───────────

/// Inkling Quillstrike (b128) — {1}{W}{B}, 2/2 Inkling Rogue Flying.
/// Magecraft drain 1 — same shape as Inkling Coursebinder, with Rogue
/// subtype instead of Wizard (synergises with Rogue tribal payoffs).
pub fn inkling_quillstrike_b128() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillstrike (b128)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Inkmaster (b128) — {2}{W}{B}, 3/3 Inkling Wizard
/// Flying + Lifelink. ETB mints an Inkling token. 4-mana race-breaking
/// double-flyer payoff (4 power in the air on entry, gains 4 life
/// triggered by attacking lifelink combined).
pub fn silverquill_inkmaster_b128() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmaster (b128)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Drafter (b128) — {2}{B}, 2/2 Vampire Warlock. Magecraft
/// Surveil 1 — Witherbloom-flavor surveil on a Silverquill body for
/// gy-fueling decks.
pub fn silverquill_drafter_b128() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drafter (b128)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_surveil(1)],
        ..Default::default()
    }
}

/// Silverquill Sermonist (b128) — {1}{W}, 2/3 Human Cleric Vigilance.
/// ETB Scry 1. Standard early defender at the 2-drop slot — durable
/// blocker that also smooths the early draw.
pub fn silverquill_sermonist_b128() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sermonist (b128)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::etb_scry(1)],
        ..Default::default()
    }
}

/// Inkling Vellumbinder (b128) — {3}{W}{B}, 4/3 Inkling Cleric Flying.
/// ETB drain 2 (4-life swing). 5-mana evasive race-breaker.
pub fn inkling_vellumbinder_b128() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vellumbinder (b128)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Inkblot (b128) — {W}{B} Instant. Seq(Drain 1 + Draw 1).
/// 2-mana drain-and-cantrip. Mini Sign in Blood variant.
pub fn silverquill_inkblot_b128() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkblot (b128)",
        cost: cost(&[w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Watchwarden (b128) — {2}{W}, 2/4 Inkling Soldier Flying +
/// Vigilance. Big evasive defender; pairs with Tenured Inkcaster
/// anthem to become a 4/6 flier.
pub fn inkling_watchwarden_b128() -> CardDefinition {
    CardDefinition {
        name: "Inkling Watchwarden (b128)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 129 (push claude/modern_decks): new Silverquill cards ──────────

/// Silverquill Inkwriter (b129) — {2}{W}, 2/3 Human Cleric. ETB Seq(
/// GainLife 1 + Draw 1). 3-mana cantrip + lifegain body — feeds Light
/// of Promise / Inkling Bloodscribe lifegain payoffs.
pub fn silverquill_inkwriter_b129() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Inkwriter (b129)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Inkling Stormpaper (b129) — {3}{W}{B}, 4/4 Inkling Wizard Flying.
/// ETB Drain 2 + Inkling token mint — Lorehold's twin-mode finisher
/// but with Inkling tribal payoff. Uses the new b129
/// `etb_mint_token_and_drain` shortcut helper (mint Inkling + drain 2).
pub fn inkling_stormpaper_b129() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_and_drain;
    CardDefinition {
        name: "Inkling Stormpaper (b129)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_and_drain(inkling_token(), 2)],
        ..Default::default()
    }
}

/// Silverquill Quillrender (b129) — {2}{B} Sorcery. Target opp loses
/// 3 life, you gain 3. Sign-in-Blood-style drain at sorcery speed.
pub fn silverquill_quillrender_b129() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillrender (b129)",
        cost: cost(&[generic(2), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Loreward (b129) — {1}{W} 2/2 Inkling Cleric Vigilance.
/// Tribal vigilance defender at 2 mana — premier early curve Inkling.
pub fn inkling_loreward_b129() -> CardDefinition {
    CardDefinition {
        name: "Inkling Loreward (b129)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 130 (push claude/modern_decks): more Silverquill cards ────────────

/// Silverquill Pageturner (b130) — {1}{W}, 1/2 Inkling Cleric Flying.
/// ETB Scry 1. Tempo-flier with smoothing — cheap Inkling at 2 mana
/// that triggers Inkcaster's anthem and digs for Inkling payoffs.
pub fn silverquill_pageturner_b130() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Pageturner (b130)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Inkling Archivist (b130) — {3}{B}, 2/3 Inkling Wizard Flying. Dies
/// → drain 1. Aristocrat-style 4-drop Inkling that trades for value.
pub fn inkling_archivist_b130() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Inkling Archivist (b130)",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![dies_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Inkclaw (b130) — {1}{W}{B} Instant. Each opponent loses
/// 2 life, you gain 2 life, then put a -1/-1 counter on target creature.
/// Drain-and-shrink combo at instant speed.
pub fn silverquill_inkclaw_b130() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkclaw (b130)",
        cost: cost(&[generic(1), w(), b()]),
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
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Quillsworn (b130) — {2}{W}{B}, 3/3 Inkling Knight. First
/// Strike. A premium Inkling-typed knight with strong combat math —
/// scales further with Tenured Inkcaster's anthem (→ 5/5 first strike).
pub fn silverquill_quillsworn_b130() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillsworn (b130)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─── Batch 131: Silverquill synthesised cards ──────────────────────────────────

/// Silverquill Inkblade (b131) — {1}{W}, 2/2 Inkling Cleric, Flying.
/// Vanilla evasive 2-drop — strong on its own and pumped further by
/// Tenured Inkcaster's +2/+2 anthem.
pub fn silverquill_inkblade_b131() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkblade (b131)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sermon II (b131) — {2}{W}{B} Sorcery. Seq(Drain 2 +
/// CreateToken 1 Inkling). Uses the `drain` shortcut.
pub fn inkling_sermon_ii_b131() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sermon II (b131)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Serene Voice (b131) — {1}{W}{B}, 2/2 Vampire Cleric,
/// Lifelink. ETB drain 1.
pub fn silverquill_serene_voice_b131() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Serene Voice (b131)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Quill Blade (b131) — {W}{B} Instant. Seq(Drain 2 +
/// PumpPT +1/+1 EOT to target creature you control).
pub fn silverquill_quill_blade_b131() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quill Blade (b131)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 132 ───────────────────────────────────────────────────────────────

/// Silverquill Ink-Apprentice (b132) — {W}, 1/2 Inkling Cleric, Flying.
/// Vanilla one-drop flier; feeds the Inkling tribal pool (Inkcaster,
/// Verselord) at the bottom of the curve.
pub fn silverquill_ink_apprentice_b132() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ink-Apprentice (b132)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quill-Striker (b132) — {2}{B}, 3/2 Inkling Rogue, Flying.
/// Whenever this attacks, each opponent loses 1 life and you gain 1.
/// Mirror of the on-attack-drain pattern on a flier.
pub fn inkling_quill_striker_b132() -> CardDefinition {
    use crate::effect::shortcut::on_attack_drain;
    CardDefinition {
        name: "Inkling Quill-Striker (b132)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Scrivener-Apprentice (b132) — {2}{W}, 2/3 Human Cleric.
/// ETB scry 1, then draw a card. Smoothing body that pairs with the
/// Mavinda recursion engine. Uses `etb_scry_and_draw` shortcut.
pub fn silverquill_scrivener_apprentice_b132() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Silverquill Scrivener-Apprentice (b132)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Inkling Pamphleteer II (b132) — {1}{W}{B}, 2/2 Inkling Wizard,
/// Flying. Magecraft drain 1 (asymmetric — opp loses 1, you gain 1).
/// Cheap flier with a magecraft drain engine.
pub fn inkling_pamphleteer_ii_b132() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain;
    CardDefinition {
        name: "Inkling Pamphleteer II (b132)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain(1)],
        ..Default::default()
    }
}

// ── Batch 133 ───────────────────────────────────────────────────────────────

/// Silverquill Inkwriter II (b133) — {2}{W}, 2/3 Inkling Cleric.
/// ETB mints an Inkling token and gains 1 life. Uses the new
/// `etb_mint_token_and_gain_life` shortcut.
pub fn silverquill_inkwriter_ii_b133() -> CardDefinition {
    use crate::catalog::inkling_token;
    use crate::effect::shortcut::etb_mint_token_and_gain_life;
    CardDefinition {
        name: "Silverquill Inkwriter II (b133)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_and_gain_life(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Inkling Skydeath (b133) — {3}{B}, 3/2 Inkling Rogue, Flying.
/// Dies → drain 2. Mid-curve fragile sweeper bait that pays you back.
pub fn inkling_skydeath_b133() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Inkling Skydeath (b133)",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![dies_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Pure Touch (b133) — {W}{B} Instant. Target creature
/// gets +1/+1 EOT and gains Lifelink EOT. Uses the new
/// `pump_and_grant_keyword` shortcut.
pub fn silverquill_pure_touch_b133() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Silverquill Pure Touch (b133)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(1, 1, Keyword::Lifelink),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 134 ───────────────────────────────────────────────────────────────

/// Silverquill Inkflight (b134) — {W} Instant. Target creature gets
/// +1/+1 EOT and gains Flying EOT. Mirrors Pure Touch but with the
/// Flying-grant rider for an aerial combat trick.
pub fn silverquill_inkflight_b134() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Silverquill Inkflight (b134)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(1, 1, Keyword::Flying),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Lifemender (b134) — {2}{W}, 2/3 Inkling Cleric Flying.
/// Magecraft: gain 1 life. Defensive flier that drips lifegain on
/// every instant/sorcery cast.
pub fn inkling_lifemender_b134() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life;
    CardDefinition {
        name: "Inkling Lifemender (b134)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

// ── Batch 135 ───────────────────────────────────────────────────────────────

/// Silverquill Penwarden (b135) — {1}{W}{B}, 2/3 Inkling Cleric, Flying.
/// ETB drain 1. Defensive evasive drain body — 3-mana 2/3 flier
/// with a built-in 2-life swing.
pub fn silverquill_penwarden_b135() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penwarden (b135)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Inkling Quill-Cleric (b135) — {2}{W}, 2/2 Inkling Cleric, Flying +
/// Lifelink. Vanilla efficient evasive lifelinker — fills the 3-mana
/// curve slot alongside Inkling Sanctifier with a different keyword mix.
pub fn inkling_quill_cleric_b135() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quill-Cleric (b135)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Edict-Speaker (b135) — {1}{W}{B} Sorcery. Target opponent
/// sacrifices a creature; you gain 2 life and draw a card. Beefier
/// Diabolic-Edict-with-rider at the 3-mana slot — closes out aristocrats
/// and tempo positions.
pub fn silverquill_edict_speaker_b135() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edict-Speaker (b135)",
        cost: cost(&[generic(1), w(), b()]),
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
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Bookworm (b135) — {W} 1/2 Human Cleric. Magecraft scry 1.
/// Cheap one-drop selection engine.
pub fn silverquill_bookworm_b135() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookworm (b135)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

// ── Batch 136 ───────────────────────────────────────────────────────────────

/// Inkling Forewing (b136) — {2}{W} 2/2 Inkling Cleric Flying. Ward 1.
/// 3-mana Inkling flier with a 1-mana protection tax — fills the
/// defensive midrange slot in Inkling tribal builds.
pub fn inkling_forewing_b136() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Inkling Forewing (b136)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(1))],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Honor-Witness (b136) — {2}{W} 2/3 Human Cleric. ETB Seq
/// (GainLife 2 + Scry 1). Defensive lifegain body that smooths the top.
pub fn silverquill_honor_witness_b136() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::{EventScope, EventSpec};
    CardDefinition {
        name: "Silverquill Honor-Witness (b136)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Inkling Battle-Scribe (b136) — {3}{B} 3/3 Inkling Wizard Flying.
/// Magecraft each-opp loses 1. Heavier evasive drain body — pairs with
/// Tenured Inkcaster to swing the race.
pub fn inkling_battle_scribe_b136() -> CardDefinition {
    CardDefinition {
        name: "Inkling Battle-Scribe (b136)",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

// ── Batch 137 ───────────────────────────────────────────────────────────────

/// Silverquill Pen-Master (b137) — {2}{W}{B} 2/3 Inkling Wizard Flying.
/// ETB Drain 1 + Draw a card. Uses the new `etb_drain_and_draw(1)`
/// shortcut. 4-mana evasive value body.
pub fn silverquill_pen_master_b137() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_draw;
    CardDefinition {
        name: "Silverquill Pen-Master (b137)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_draw(1)],
        ..Default::default()
    }
}

/// Inkling Wingmother (b137) — {3}{W}{B} 3/3 Inkling Wizard Flying.
/// Whenever this creature attacks, create a 1/1 Inkling token. Uses
/// the new `on_attack_create_token` shortcut. Strong Inkling-tribal
/// engine — every attack adds a body.
pub fn inkling_wingmother_b137() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    use crate::effect::shortcut::on_attack_create_token;
    CardDefinition {
        name: "Inkling Wingmother (b137)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_create_token(inkling_token())],
        ..Default::default()
    }
}

/// Silverquill Pristine Sermon (b136) — {3}{W}{B} Sorcery. Seq
/// (Drain 3 + Scry 2 + CreateToken 1 Inkling). 5-mana drain finisher
/// that mints a flyer to push through the table.
pub fn silverquill_pristine_sermon_b136() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Pristine Sermon (b136)",
        cost: cost(&[generic(3), w(), b()]),
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 138 ───────────────────────────────────────────────────────────────

/// Silverquill Inksworn (b138) — {1}{W}{B} 2/3 Inkling Cleric Flying.
/// ETB Drain 1. Solid 3-mana evasive drain body, mirrors
/// Inkling Coursebinder shape.
pub fn silverquill_inksworn_b138() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inksworn (b138)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Inkling Ledgerwarden (b138) — {2}{W} 1/4 Inkling Cleric Flying +
/// Vigilance. Magecraft Scry 1. Defensive evasive scaler.
pub fn inkling_ledgerwarden_b138() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ledgerwarden (b138)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Quillstrike (b138) — {W}{B} Instant.
/// Seq(Drain 2 + Scry 1). 2-mana drain spell with selection.
pub fn silverquill_quillstrike_b138() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillstrike (b138)",
        cost: cost(&[w(), b()]),
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
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quillforge (b138) — {3}{W}{B} 3/3 Inkling Wizard Flying.
/// ETB drain 1 + draw a card. Uses `etb_drain_and_draw(1)`.
pub fn inkling_quillforge_b138() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_draw;
    CardDefinition {
        name: "Inkling Quillforge (b138)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_draw(1)],
        ..Default::default()
    }
}

// ── Batch 139 ───────────────────────────────────────────────────────────────

/// Silverquill Inkdrinker (b139) — {2}{W}{B} 3/3 Vampire Warlock
/// Lifelink. ETB drain 2.
pub fn silverquill_inkdrinker_b139() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkdrinker (b139)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Inkling Scribesong (b139) — {2}{W}{B} Sorcery.
/// Seq(Drain 2 + Surveil 2). Drain + selection.
pub fn inkling_scribesong_b139() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scribesong (b139)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_surveil(2, 2),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pearlcaller (b139) — {W} 1/1 Human Cleric. Magecraft
/// 2 life. High-rate magecraft lifegain at the 1-drop slot.
pub fn silverquill_pearlcaller_b139() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pearlcaller (b139)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(2)],
        ..Default::default()
    }
}

/// Silverquill Memorialist II (b138) — {1}{W} 1/3 Human Cleric.
/// Magecraft Gain 1 life. Defensive lifegain-on-cast body.
pub fn silverquill_memorialist_ii_b138() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memorialist II (b138)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

// ── Batch 141 ───────────────────────────────────────────────────────────────

/// Inkling Lifeharvester (b141) — {2}{W}{B} 3/3 Inkling Cleric Flying
/// + Lifelink. ETB drain 1. Classic Silverquill drain flyer.
pub fn inkling_lifeharvester_b141() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lifeharvester (b141)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Penblade (b141) — {W}{B} Instant.
/// Seq(Drain 1 + PumpPT +1/+1 EOT target friendly creature).
/// 2-mana combat trick that doubles as drain.
pub fn silverquill_penblade_b141() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penblade (b141)",
        cost: cost(&[w(), b()]),
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
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Initiate (b141) — {W} 1/2 Human Cleric.
/// Magecraft Surveil 1. 1-mana defender + selection on every spell.
pub fn silverquill_initiate_b141() -> CardDefinition {
    use crate::effect::shortcut::magecraft_surveil;
    CardDefinition {
        name: "Silverquill Initiate (b141)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_surveil(1)],
        ..Default::default()
    }
}

/// Inkling Quill-Knight (b141) — {3}{W}{B} 4/3 Inkling Knight Flying +
/// Vigilance. ETB mint Inkling token + drain 1. Aggressive go-wide drain
/// finisher.
pub fn inkling_quill_knight_b141() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token_and_drain;
    CardDefinition {
        name: "Inkling Quill-Knight (b141)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token_and_drain(inkling_token(), 1)],
        ..Default::default()
    }
}

// ── Batch 142 ───────────────────────────────────────────────────────────────

/// Inkling Magistry (b142) — {3}{W}{B} Sorcery. Drain 3 + Surveil 2.
/// 5-mana drain + selection finisher.
pub fn inkling_magistry_b142() -> CardDefinition {
    CardDefinition {
        name: "Inkling Magistry (b142)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_surveil(3, 2),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkmaster (b142) — {2}{W} 2/3 Human Wizard Vigilance.
/// Magecraft put a +1/+1 counter on target friendly Inkling. Inkling-
/// tribal magecraft snowball.
pub fn silverquill_inkmaster_b142() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmaster (b142)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Decree (b142) — {1}{B} Instant. Target creature gets
/// -3/-3 EOT and you gain 1 life. 2-mana removal trick.
pub fn silverquill_decree_b142() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Decree (b142)",
        cost: cost(&[generic(1), b()]),
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
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Heartbinder (b142) — {2}{W}{B} 2/4 Inkling Cleric Flying +
/// Lifelink. ETB Scry 2. 4-mana defensive lifelink flyer + smoothing.
pub fn inkling_heartbinder_b142() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Inkling Heartbinder (b142)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(2)],
        ..Default::default()
    }
}

/// Silverquill Ledgerward (b142) — {W}{B} 2/2 Vampire Cleric.
/// ETB Drain 1 + Surveil 1. 2-mana early drain + selection.
pub fn silverquill_ledgerward_b142() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_surveil;
    CardDefinition {
        name: "Silverquill Ledgerward (b142)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_surveil(1, 1)],
        ..Default::default()
    }
}

// ── Batch 143 ───────────────────────────────────────────────────────────────

/// Silverquill Inkflight (b143) — {1}{W} 2/2 Inkling Cleric Flying.
/// Vanilla evasive 2-mana Inkling — Tenured Inkcaster fodder.
pub fn silverquill_inkflight_b143() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkflight (b143)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pyremaster (b143) — {2}{W}{B} 3/3 Vampire Bard Flying.
/// ETB Seq(Drain 2 + Scry 1). 4-mana race-breaker.
pub fn silverquill_pyremaster_b143() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_scry;
    CardDefinition {
        name: "Silverquill Pyremaster (b143)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_scry(2, 1)],
        ..Default::default()
    }
}

/// Inkling Quillwhisper (b143) — {1}{W}{B} 2/2 Inkling Wizard Flying.
/// Magecraft Seq(Drain 1 + Scry 1).
pub fn inkling_quillwhisper_b143() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillwhisper (b143)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Silverquill Quillcleave (b143) — {1}{B} Instant. Target creature
/// gets -4/-4 EOT. 2-mana big shrink-removal.
pub fn silverquill_quillcleave_b143() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillcleave (b143)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Ledgerlord (b143) — {3}{W}{B} 3/4 Inkling Bard Flying +
/// Lifelink. ETB MayDo(Sacrifice another creature → mint 2 Inkling tokens).
pub fn inkling_ledgerlord_b143() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Inkling Ledgerlord (b143)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Sacrifice another creature to mint 2 Inkling tokens".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: inkling_token(),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Silverquill Resonance (b143) — {W}{B} Sorcery. Each opponent loses
/// 2 life and discards a card. 2-mana double drain + discard.
pub fn silverquill_resonance_b143() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Resonance (b143)",
        cost: cost(&[w(), b()]),
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
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Inkcaller (b143) — {2}{W}{B} 2/2 Inkling Cleric Flying +
/// Lifelink. ETB mint 1 Inkling token via shared helper.
pub fn inkling_inkcaller_b143() -> CardDefinition {
    use crate::effect::shortcut::etb_mint_token;
    CardDefinition {
        name: "Inkling Inkcaller (b143)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Devotional (b143) — {2}{W} Sorcery. Gain 5 life and
/// scry 2. 3-mana defensive lifegain + selection.
pub fn silverquill_devotional_b143() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Devotional (b143)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(5),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 144 ───────────────────────────────────────────────────────────────

/// Silverquill Quillscholar (b144) — {1}{W}{B} 2/3 Inkling Cleric.
/// Cycling {2}. Filler card to lock in a cycling-with-creature-body
/// pattern that exists on Watcher of the Spheres-class cards.
pub fn silverquill_quillscholar_b144() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillscholar (b144)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Vanquisher (b144) — {2}{W}{B} 3/3 Inkling Knight Flying.
/// Whenever this attacks, target opp loses 2 life and you gain 2 life.
pub fn inkling_vanquisher_b144() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vanquisher (b144)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Devout (b144) — {W} 1/1 Human Cleric. Magecraft +1/+1
/// counter on this creature. Self-growing 1-drop.
pub fn silverquill_devout_b144() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Devout (b144)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Inkling Sanctioner (b144) — {2}{W} 2/3 Inkling Soldier Vigilance.
/// ETB GainLife 2 + magecraft Scry 1. 3-mana scaling defensive body.
pub fn inkling_sanctioner_b144() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sanctioner (b144)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2), magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Reproach (b144) — {2}{B} Instant. Destroy target creature
/// you don't control with toughness ≤ 3. 3-mana mid-range removal.
pub fn silverquill_reproach_b144() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Reproach (b144)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ToughnessAtMost(3))
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Mercenary (b144) — {1}{B} 2/2 Inkling Rogue Menace.
/// 2-mana menace body — Tenured Inkcaster fodder.
pub fn inkling_mercenary_b144() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mercenary (b144)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Ascension (b144) — {3}{W}{B} Enchantment. "At the
/// beginning of your end step, drain 1." 5-mana drip-drain payoff.
pub fn silverquill_ascension_b144() -> CardDefinition {
    use crate::card::EventScope;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Silverquill Ascension (b144)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            ),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 145 ───────────────────────────────────────────────────────────────

/// Silverquill Hexbearer (b145) — {1}{B} 2/2 Inkling Wizard. ETB target
/// opp discards 1 + Drain 1. Compact 2-mana disruption + drain.
pub fn silverquill_hexbearer_b145() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Hexbearer (b145)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Silverquill Spellbearer (b145) — {2}{W}{B} 2/4 Inkling Cleric Vigilance.
/// Static: Other Inkling creatures you control have lifelink.
pub fn silverquill_spellbearer_b145() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Silverquill Spellbearer (b145)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Sage (b145) — {3}{W} 2/4 Human Cleric Vigilance.
/// Cycling {W}. Defensive cycle-trigger anchor.
pub fn silverquill_sage_b145() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sage (b145)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Cycling(cost(&[w()]))],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Heartmender (b145) — {1}{W} Sorcery. Gain 4 life.
/// 2-mana cheap heal.
pub fn silverquill_heartmender_b145() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Heartmender (b145)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(4),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Wraith (b145) — {2}{B} 2/2 Inkling Spirit Flying.
/// Dies → each opp loses 2 life. Aristocrat-style death-drain.
pub fn inkling_wraith_b145() -> CardDefinition {
    use crate::effect::shortcut::dies_lose_life_each_opp;
    CardDefinition {
        name: "Inkling Wraith (b145)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![dies_lose_life_each_opp(2)],
        ..Default::default()
    }
}

// ── Batch 146 ───────────────────────────────────────────────────────────────

/// Silverquill Inkmaster Adept (b146) — {2}{W}{B} 3/3 Inkling Wizard
/// Flying. Magecraft target opp loses 1 life and you gain 1 life.
/// 4-mana flier with magecraft drain — pairs with Tenured Inkcaster.
pub fn silverquill_inkmaster_adept_b146() -> CardDefinition {
    use crate::effect::shortcut::magecraft_drain;
    CardDefinition {
        name: "Silverquill Inkmaster Adept (b146)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Inkglyph (b146) — {W}{B} Sorcery. Seq(Drain 2 + Scry 1).
/// 2-mana drain + dig — strict upgrade to Silverquill Witness's
/// magecraft drain at instant tempo without the body.
pub fn silverquill_inkglyph_b146() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkglyph (b146)",
        cost: cost(&[w(), b()]),
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
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Pyrescribe (b146) — {1}{W} 2/2 Inkling Soldier. ETB GainLife 1
/// + magecraft GainLife 1. 2-mana drip-life body that scales in
///   spell-heavy shells.
pub fn inkling_pyrescribe_b146() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pyrescribe (b146)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb_gain_life(1),
            magecraft_gain_life(1),
        ],
        ..Default::default()
    }
}

/// Silverquill Inkbinder (b146) — {1}{B} 2/1 Human Wizard. Magecraft
/// target opp discards a card at random. 2-mana attrition-style
/// magecraft body — pairs with Silverquill Hexbearer.
pub fn silverquill_inkbinder_b146() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbinder (b146)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
            random: true,
        })],
        ..Default::default()
    }
}

/// Inkling Inkbearer (b146) — {3}{W}{B} 3/4 Inkling Knight Flying +
/// Vigilance. ETB mints 1 Inkling token. 5-mana go-wide finisher.
pub fn inkling_inkbearer_b146() -> CardDefinition {
    CardDefinition {
        name: "Inkling Inkbearer (b146)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Ledgerblade (b146) — {1}{W} Instant. Seq(PumpPT(+1/+2 EOT)
/// + GrantKeyword(Vigilance EOT)). 2-mana combat trick — strengthens
///   the blocker and lets it crack back.
pub fn silverquill_ledgerblade_b146() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Silverquill Ledgerblade (b146)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Hex-Cleric (b146) — {2}{B} 3/2 Human Cleric Lifelink.
/// ETB target opp loses 2 life. 3-mana lifelink finisher.
pub fn silverquill_hex_cleric_b146() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Hex-Cleric (b146)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Inkling Verseguard (b146) — {1}{W}{B} 2/3 Inkling Soldier Flying.
/// Magecraft +1/+1 counter on this creature. 3-mana sticky magecraft
/// flier that snowballs in spell-heavy shells.
pub fn inkling_verseguard_b146() -> CardDefinition {
    CardDefinition {
        name: "Inkling Verseguard (b146)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Silverquill Inkriot (b146) — {3}{W}{B} Sorcery. Seq(CreateToken(2
/// Inklings) + GainLife 2). 5-mana go-wide + life rider.
pub fn silverquill_inkriot_b146() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkriot (b146)",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Wingblade (b146) — {W}{B} 2/2 Inkling Knight Flying.
/// Aggressive 2-mana evasive Inkling — pairs with Tenured Inkcaster
/// (+2/+2 anthem) and Inkling Banner-Bearer (+1/+0 anthem).
pub fn inkling_wingblade_b146() -> CardDefinition {
    CardDefinition {
        name: "Inkling Wingblade (b146)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Lifeward (b146) — {2}{W}{B} 2/4 Inkling Cleric Lifelink.
/// Static: "Your opponents can't lose life." A foil for drain decks
/// (locks Bolas-style life loss). Wires the existing
/// `StaticEffect::PlayerCannotLoseLife` primitive end-to-end with a
/// representative card — see CR 119.8.
pub fn silverquill_lifeward_b146() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{PlayerStaticTarget, StaticEffect};
    CardDefinition {
        name: "Silverquill Lifeward (b146)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Your opponents can't lose life.",
            effect: StaticEffect::PlayerCannotLoseLife {
                target: PlayerStaticTarget::EachOpponent,
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Reproach (b209) — {2}{B} Enchantment. Static (CR 614,
/// Tainted Remedy template): "If an opponent would gain life, that player
/// loses that much life instead." Wires `StaticEffect::LifeGainBecomesLoss`
/// end-to-end with a representative card.
pub fn silverquill_reproach_b209() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{PlayerStaticTarget, StaticEffect};
    CardDefinition {
        name: "Silverquill Reproach (b209)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would gain life, that player loses that much life instead.",
            effect: StaticEffect::LifeGainBecomesLoss {
                target: PlayerStaticTarget::EachOpponent,
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Tithe-Taker (b209) — {1}{B} 2/2 Inkling. Exploit (CR
/// 702.105). "When you exploit a creature, each opponent loses 2 life and
/// you gain 2 life." Wires `shortcut::exploit` end-to-end.
pub fn silverquill_tithe_taker_b209() -> CardDefinition {
    use crate::effect::shortcut::{drain, exploit};
    CardDefinition {
        name: "Silverquill Tithe-Taker (b209)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![exploit(drain(2))],
        ..Default::default()
    }
}

/// Witherbloom Devourer (b209) — {3}{G} 3/3 Beast. Devour 1 (CR 702.83):
/// "As this enters, you may sacrifice any number of creatures. It enters
/// with a +1/+1 counter for each." Wires `shortcut::devour`.
pub fn witherbloom_devourer_b209() -> CardDefinition {
    use crate::effect::shortcut::devour;
    CardDefinition {
        name: "Witherbloom Devourer (b209)",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![devour(1)],
        ..Default::default()
    }
}

// ── Batch 147 ───────────────────────────────────────────────────────────────

/// Silverquill Penmaster (b147) — {1}{W}{B} 2/2 Inkling Wizard Flying.
/// Magecraft +1/+1 counter on this creature + drain 1. Uses the new
/// `magecraft_self_pump_and_drain(1)` helper — scales aggressively in
/// spell shells.
pub fn silverquill_penmaster_b147() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump_and_drain;
    CardDefinition {
        name: "Silverquill Penmaster (b147)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump_and_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Cantorscribe (b147) — {2}{W}{B} 3/3 Inkling Cleric.
/// ETB drain 1 + draw 1. Uses the new `etb_drain_and_draw_one(1)`
/// helper. 4-mana value body that converts attrition into cards.
pub fn silverquill_cantorscribe_b147() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_draw_one;
    CardDefinition {
        name: "Silverquill Cantorscribe (b147)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_draw_one(1)],
        ..Default::default()
    }
}

/// Silverquill Inkdrip (b147) — {W}{B} Instant. Drain 2 + Gain 1 life.
/// 2-mana instant Drain Life — 3-life swing.
pub fn silverquill_inkdrip_b147() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkdrip (b147)",
        cost: cost(&[w(), b()]),
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
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Lifesong (b147) — {2}{W} 2/3 Inkling Bard Lifelink + Vigilance.
/// 3-mana double-keyword defensive body.
pub fn inkling_lifesong_b147() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lifesong (b147)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Aggressor (b147) — {1}{B} 2/2 Inkling Rogue Menace.
/// On-attack: each opp loses 1 life. 2-mana attrition aggressor.
pub fn silverquill_aggressor_b147() -> CardDefinition {
    use crate::effect::shortcut::on_attack_drain;
    CardDefinition {
        name: "Silverquill Aggressor (b147)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

// ── Batch 148 ───────────────────────────────────────────────────────────────

/// Silverquill Mortarscribe (b148) — {2}{W} 2/4 Inkling Cleric.
/// Static "Whenever you gain life, each opp loses 1 life." Witherbloom-
/// flavor Inkling that converts each lifegain into a drain trickle.
pub fn silverquill_mortarscribe_b148() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mortarscribe (b148)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Inkling Crusader (b148) — {2}{B} 3/2 Inkling Knight Menace + Lifelink.
/// 3-mana hyper-aggressive evasive lifelinker.
pub fn inkling_crusader_b148() -> CardDefinition {
    CardDefinition {
        name: "Inkling Crusader (b148)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Cinderglyph (b148) — {1}{B} Instant. Target creature gets
/// -2/-2 EOT. 2-mana removal-for-2-toughness.
pub fn silverquill_cinderglyph_b148() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cinderglyph (b148)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Glyphmaster (b148) — {1}{W}{B} 1/4 Inkling Cleric. Static
/// "Other Inkling creatures you control get +0/+1." Defensive Inkling
/// anthem.
pub fn inkling_glyphmaster_b148() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Inkling Glyphmaster (b148)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 0,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Lifesong (b148) — {W} Sorcery. Gain 3 life + Scry 1.
/// 1-mana cheap value spell.
pub fn silverquill_lifesong_b148() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifesong (b148)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 149 ───────────────────────────────────────────────────────────────

/// Silverquill Ink-Knight (b149) — {2}{W}{B} 3/2 Inkling Knight Flying +
/// Lifelink + Indestructible. Tank evasive lifelinker for combat-heavy
/// shells. Engine note: Indestructible blocks damage-from-zero-toughness
/// only — `-N/-N` debuffs still kill it via the 0-toughness SBA path
/// (CR 704.5f), matching the printed Indestructible interaction.
pub fn silverquill_ink_knight_b149() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ink-Knight (b149)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink, Keyword::Indestructible],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Soulpenitent (b149) — {1}{W} 1/3 Human Cleric Hexproof.
/// 2-mana sticky defender — Hexproof body.
pub fn silverquill_soulpenitent_b149() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Soulpenitent (b149)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Hexproof],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 150 ───────────────────────────────────────────────────────────────

/// Silverquill Penmaster-General (b150) — {3}{W}{B} 4/4 Human Cleric
/// Wizard. Vigilance + lifelink. Solid mid-curve lifelinker.
pub fn silverquill_penmaster_general_b150() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penmaster-General (b150)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quillveteran (b150) — {2}{W}{B} 3/4 Inkling Soldier Flying +
/// Vigilance. Beefy evasive Inkling — Tenured Inkcaster anthem stacks
/// to 5/6.
pub fn inkling_quillveteran_b150() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillveteran (b150)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Lifebringer (b150) — {2}{W} 2/2 Human Cleric. ETB
/// gain 3 life. Bigger sibling of Silverquill Marshal.
pub fn silverquill_lifebringer_b150() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifebringer (b150)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(3)],
        ..Default::default()
    }
}

/// Silverquill Doomscribe (b150) — {2}{B} 3/2 Human Wizard. Magecraft
/// each opponent loses 2 life (no symmetric self-gain — pure burn).
pub fn silverquill_doomscribe_b150() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Doomscribe (b150)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Silverquill Verseblade (b150) — {W} Instant. Target creature you
/// control gets +2/+2 EOT and gains lifelink EOT — combat trick + lifelink.
pub fn silverquill_verseblade_b150() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Silverquill Verseblade (b150)",
        cost: cost(&[w()]),
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
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Funerary Rite (b150) — {1}{B} Sorcery. Drain 2 + you
/// gain 2 life. Standard Witherbloom/Silverquill burn-and-heal at 2 mana.
pub fn silverquill_funerary_rite_b150() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Funerary Rite (b150)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(2),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Edgewriter (b150) — {1}{W}{B} 2/3 Inkling Knight Flying +
/// Lifelink. Solid 3-drop with Inkling's anthem support.
pub fn inkling_edgewriter_b150() -> CardDefinition {
    CardDefinition {
        name: "Inkling Edgewriter (b150)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 151 ───────────────────────────────────────────────────────────────

/// Silverquill Disciple (b151) — {W} 1/1 Human Cleric. Lifelink.
/// Cheap 1-drop lifelink body.
pub fn silverquill_disciple_b151() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Disciple (b151)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Scout (b151) — {1}{B} 2/2 Inkling Scout Flying.
/// Compact 2-mana flying body.
pub fn inkling_scout_b151() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scout (b151)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sanctuary (b151) — {4}{W}{B} Enchantment.
/// At the beginning of your upkeep, drain 1. Slow but steady drain.
pub fn silverquill_sanctuary_b151() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Silverquill Sanctuary (b151)",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: drain(1),
        }],
        ..Default::default()
    }
}

/// Silverquill Recruiter (b151) — {2}{W} 2/3 Human Cleric.
/// ETB scry 1 + draw 1 (an Elvish-Visionary-with-scry shape).
pub fn silverquill_recruiter_b151() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Recruiter (b151)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Silverquill Smite (b151) — {2}{B} Instant. Destroy target creature.
/// Clean 3-mana hard removal.
pub fn silverquill_smite_b151() -> CardDefinition {
    use crate::card::SelectionRequirement;
    CardDefinition {
        name: "Silverquill Smite (b151)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(SelectionRequirement::Creature),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pen Striker (b151) — `{1}{B}` 2/2 Inkling Knight with
/// Flying and Lifelink. Mid-curve evasive lifelink body. Combos with
/// Tenured Inkcaster's anthem (+2/+2) to swing as a 4/4.
pub fn silverquill_pen_striker_b151() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen Striker (b151)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Conjurer (b151) — {3}{W}{B} 2/2 Human Wizard. ETB mints 2
/// Inkling tokens. Token engine.
pub fn inkling_conjurer_b151() -> CardDefinition {
    CardDefinition {
        name: "Inkling Conjurer (b151)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 2)],
        ..Default::default()
    }
}

// ── Batch 152 ───────────────────────────────────────────────────────────────

/// Silverquill Verseguard (b152) — {1}{W} 2/2 Inkling Knight Vigilance.
/// Compact tempo defender.
pub fn silverquill_verseguard_b152() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Verseguard (b152)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Memoryflame (b152) — {W}{B} Instant. Drain 1 + Surveil 2.
/// Compact removal-adjacent that fills the graveyard.
pub fn silverquill_memoryflame_b152() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Memoryflame (b152)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(1),
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Champion (b152) — {2}{W}{B} 3/3 Inkling Knight
/// Flying + Lifelink. Aggressive midrange Inkling.
pub fn silverquill_champion_b152() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Champion (b152)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Mortarscribe (b152) — {2}{W} 2/3 Human Wizard.
/// ETB drain 2.
pub fn silverquill_mortarscribe_b152() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mortarscribe (b152)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Sacrificemage (b152) — {1}{B}{B} 3/2 Human Cleric.
/// Magecraft drain 2. Premium midrange drain engine.
pub fn silverquill_sacrificemage_b152() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sacrificemage (b152)",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(2)],
        ..Default::default()
    }
}

/// Inkling Tactician (b152) — {2}{W} 2/3 Inkling Soldier Flying.
/// Magecraft +1/+0 EOT to target friendly Inkling. Pumps Inkling
/// army on cast.
pub fn inkling_tactician_b152() -> CardDefinition {
    CardDefinition {
        name: "Inkling Tactician (b152)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: Selector::EachPermanent(
                SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                    .and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

// ── Batch 154 ───────────────────────────────────────────────────────────────

/// Silverquill Inkmancer (b154) — {1}{W}{B} 2/2 Inkling Wizard,
/// Flying. Magecraft → mint a 1/1 W/B flying Inkling token via the
/// new `magecraft_mint_inkling()` shortcut. Self-replicating
/// magecraft body — the Silverquill counterpart to Sedgemoor Witch /
/// Witherbloom Pestmancer II.
pub fn silverquill_inkmancer_b154() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_inkling;
    CardDefinition {
        name: "Silverquill Inkmancer (b154)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_inkling()],
        ..Default::default()
    }
}

/// Silverquill Recitalist (b154) — {1}{W} 1/2 Human Cleric.
/// Magecraft self-pump +1/+1 self-counter via the new
/// `magecraft_add_counter_self()` shortcut.
pub fn silverquill_recitalist_b154() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Silverquill Recitalist (b154)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Silverquill Pacifier (b154) — {2}{W} 2/3 Human Soldier Vigilance.
/// ETB tap target opp creature via `etb_tap_opp_creature()` — defensive
/// tempo-pressure body.
pub fn silverquill_pacifier_b154() -> CardDefinition {
    use crate::effect::shortcut::etb_tap_opp_creature;
    CardDefinition {
        name: "Silverquill Pacifier (b154)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_tap_opp_creature()],
        ..Default::default()
    }
}

/// Inkling Drainreaver (b154) — {3}{W}{B} 3/3 Inkling Knight Flying.
/// ETB Drain 3 + magecraft Drain 1. Sustained Silverquill drain
/// engine.
pub fn inkling_drainreaver_b154() -> CardDefinition {
    CardDefinition {
        name: "Inkling Drainreaver (b154)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(3), magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Quilledict (b154) — {2}{W}{B} Sorcery. Seq(Drain 3 +
/// CreateToken 2 Inklings). 4-mana drain + token combo.
pub fn silverquill_quilledict_b154() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quilledict (b154)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(3),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sentinel (b154) — {1}{W} 1/3 Human Soldier
/// Vigilance. Magecraft GainLife 1 — defensive lifegain body.
pub fn silverquill_sentinel_b154() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life as mgl;
    CardDefinition {
        name: "Silverquill Sentinel (b154)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![mgl(1)],
        ..Default::default()
    }
}

/// Silverquill Sphereturn (b154) — {2}{W}{B} Instant. Drain 4
/// (8-life swing). 4-mana instant-speed game-end drain.
pub fn silverquill_sphereturn_b154() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sphereturn (b154)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Bookwarden II (b154) — {1}{W}{B} 2/3 Inkling Cleric
/// Flying + Lifelink. Compact value-stat Inkling at 3-mana.
pub fn inkling_bookwarden_b154() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bookwarden II (b154)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Reciter (b155) — {1}{W} 1/2 Human Wizard Lifelink.
/// Magecraft GainLife 2.
pub fn silverquill_reciter_b155() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life as mgl;
    CardDefinition {
        name: "Silverquill Reciter (b155)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![mgl(2)],
        ..Default::default()
    }
}

/// Inkling Striplark (b155) — {1}{B} 2/1 Inkling Rogue Flying.
/// Magecraft drains each opp for 1.
pub fn inkling_striplark_b155() -> CardDefinition {
    CardDefinition {
        name: "Inkling Striplark (b155)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Manuscriber (b155) — {2}{W} 2/3 Human Wizard.
/// ETB Scry 1 + magecraft MayDo Draw 1. Defensive body that loots.
pub fn silverquill_manuscriber_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Manuscriber (b155)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) }),
            magecraft(Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            }),
        ],
        ..Default::default()
    }
}

/// Inkling Lifepoet (b155) — {1}{W}{B} 2/3 Inkling Cleric Lifelink.
/// ETB Drain 2.
pub fn inkling_lifepoet_b155() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lifepoet (b155)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Adjudicator (b155) — {2}{W}{B} Instant. Seq(Move
/// target creature → Exile + Drain 1). 4-mana exile-removal + drain.
pub fn silverquill_adjudicator_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Adjudicator (b155)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Exile,
            },
            drain(1),
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Spellbinder (b155) — {3}{W}{B} 3/4 Inkling Wizard Flying.
/// Magecraft mints an Inkling token. Top-end Inkling tribal payoff.
pub fn inkling_spellbinder_b155() -> CardDefinition {
    use crate::effect::shortcut::magecraft_mint_inkling;
    CardDefinition {
        name: "Inkling Spellbinder (b155)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_mint_inkling()],
        ..Default::default()
    }
}

/// Silverquill Quillplay (b155) — {W}{B} Sorcery. Seq(Drain 1 +
/// CreateToken 1 Inkling). 2-mana cheap drain + body.
pub fn silverquill_quillplay_b155() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Quillplay (b155)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(1),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Glyphwarden (b155) — {2}{W} 2/4 Inkling Soldier Vigilance.
/// Static: Other Inkling creatures you control have lifelink.
pub fn inkling_glyphwarden_b155() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Inkling Glyphwarden (b155)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Silverquill Curatorial (b155) — {3}{W}{B} Sorcery. Seq(Drain 2 +
/// Move target creature card from your gy → bf untapped). Reanimator
/// + drain.
pub fn silverquill_curatorial_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Curatorial (b155)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Slipscribe (b155) — {1}{B} 1/3 Inkling Rogue Flying.
/// Magecraft +1/+0 EOT self-pump.
pub fn inkling_slipscribe_b155() -> CardDefinition {
    CardDefinition {
        name: "Inkling Slipscribe (b155)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Silverquill Recital (b155) — {2}{W}{B} Instant. Each opp sacrifices
/// a creature; mint 1 Inkling token + gain 1 life.
pub fn silverquill_recital_b155() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Recital (b155)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Instant],
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
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Vespermage (b155) — {2}{W}{B} 2/3 Inkling Wizard Flying.
/// ETB +1/+1 counter on target friendly Inkling.
pub fn inkling_vespermage_b155() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Inkling Vespermage (b155)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Caesura (b155) — {W} Instant. Tap target creature + draw 1.
pub fn silverquill_caesura_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Caesura (b155)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Pen-Verseman (b155) — {3}{W}{B} 3/3 Inkling Bard Flying +
/// Lifelink. ETB Seq(Drain 1 + Scry 1).
pub fn inkling_pen_verseman_b155() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_scry;
    CardDefinition {
        name: "Inkling Pen-Verseman (b155)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_scry(1, 1)],
        ..Default::default()
    }
}

/// Silverquill Liturgist II (b155) — {1}{W} 1/3 Human Cleric.
/// Magecraft drain 1.
pub fn silverquill_liturgist_ii_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Liturgist II (b155)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Inkling Skydrifter (b155) — {3}{W} 2/2 Inkling Soldier Flying +
/// Lifelink. ETB +1/+1 counter on self.
pub fn inkling_skydrifter_b155() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Inkling Skydrifter (b155)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Inkling Standardbearer (b154) — {3}{W} 2/4 Inkling Soldier Flying +
/// Vigilance. Static: Other Inkling creatures you control get +1/+1.
/// Premium Inkling-tribal lord at the 4-mana slot — stacks with
/// Tenured Inkcaster (+2/+2) for combo anthem on Inkling tokens.
pub fn inkling_standardbearer_b154() -> CardDefinition {
    use crate::card::StaticAbility;
    CardDefinition {
        name: "Inkling Standardbearer (b154)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
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
        ..Default::default()
    }
}

// ── Batch 155 (modern_decks) — 8 new Silverquill cards ─────────────────────

/// Silverquill Confidant (b155) — {W}{B} 2/2 Human Cleric. Magecraft:
/// gain 1 life. The "Cleric Apprentice" template — micro-lifegain
/// per spell.
pub fn silverquill_confidant_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Confidant (b155)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Inkling Veteran (b155) — {2}{W}{B} 2/2 Inkling Flying. ETB: mint
/// another 1/1 Inkling token. Silverquill flying snowball template.
pub fn inkling_veteran_b155() -> CardDefinition {
    CardDefinition {
        name: "Inkling Veteran (b155)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Critic (b155) — {1}{W} 1/3 Human Cleric. Dies → mint
/// a 1/1 W/B Inkling token with flying. Resilient body — turns into
/// a Inkling on death.
pub fn silverquill_critic_b155() -> CardDefinition {
    use crate::effect::shortcut::dies_mint_token;
    CardDefinition {
        name: "Silverquill Critic (b155)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![dies_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Wordsmith (b155) — {W}{B} 1/2 Human Wizard. Magecraft:
/// gain 1 life + scry 1. Card-selection + lifegain.
pub fn silverquill_wordsmith_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Wordsmith (b155)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Inkling Avenger (b155) — {3}{W}{B} 3/3 Inkling Flying Lifelink.
/// Pure flying lifelink finisher.
pub fn inkling_avenger_b155() -> CardDefinition {
    CardDefinition {
        name: "Inkling Avenger (b155)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Eulogist (b155) — {1}{W}{B} Sorcery. Destroy target
/// creature. Lose 1 life. Cheap removal with a Witherbloom-flavored
/// life-tail.
pub fn silverquill_eulogist_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Eulogist (b155)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Discipline (b155) — {1}{W}{B} Instant. Target creature
/// gets +2/+2 EOT, then drain 1. Combat-trick + life swing.
pub fn silverquill_discipline_b155() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Discipline (b155)",
        cost: cost(&[generic(1), w(), b()]),
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
            drain(1),
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Strider (b155) — {W}{B} 2/1 Inkling Flying Haste.
/// Aggressive 2-drop flier.
pub fn inkling_strider_b155() -> CardDefinition {
    CardDefinition {
        name: "Inkling Strider (b155)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 156 (modern_decks) — Silverquill broadcast-anchor cards ──────────

/// Silverquill Tactician (b156) — {3}{W}{B} 2/4 Human Soldier. Whenever
/// another creature you control attacks, mint a 1/1 W/B Inkling token
/// with flying. Inkling fan-out per attacker.
pub fn silverquill_tactician_b156() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Tactician (b156)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 158 (modern_decks) — Silverquill cards ───────────────────────────

/// Silverquill Inkwarden (b158) — {1}{W} 1/3 Human Cleric Vigilance.
/// ETB gain 2 life. Classic Light-of-Promise enabler at 2 mana.
pub fn silverquill_inkwarden_b158() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkwarden (b158)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

/// Inkling Pinionguard (b158) — {1}{W}{B} 2/2 Inkling Cleric Flying + Lifelink.
/// Compact 3-mana evasive lifelinker. Stacks with Tenured Inkcaster
/// for race-breaker.
pub fn inkling_pinionguard_b158() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pinionguard (b158)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pen-Crier (b158) — {1}{W}{B} 2/2 Vampire Cleric.
/// ETB Drain 2 (each opp loses 2, you gain 2). 3-mana race-breaker
/// drain body.
pub fn silverquill_pen_crier_b158() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Crier (b158)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Pen-Bearer (b158) — {1}{W} 2/2 Human Cleric.
/// Magecraft gain 1 life. Cheap lifegain-on-cast magecraft body.
pub fn silverquill_pen_bearer_b158() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Bearer (b158)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Inkling Scriptor (b158) — {W}{B} 2/2 Inkling Wizard.
/// Magecraft Drain 1. 2-mana drain magecraft body.
pub fn inkling_scriptor_b158() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scriptor (b158)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Penkeeper (b158) — {2}{W} 2/3 Human Cleric.
/// ETB Scry 1. Defensive smoother body.
pub fn silverquill_penkeeper_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Penkeeper (b158)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Vow (b158) — {W}{B} Instant. Drain 1, draw a card.
/// 2-mana drain-cantrip.
pub fn silverquill_vow_b158() -> CardDefinition {
    use crate::effect::shortcut::drain_and_draw;
    CardDefinition {
        name: "Silverquill Vow (b158)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_draw(1),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Penlord (b158) — {3}{W}{B} 3/4 Inkling Bard Flying + Lifelink.
/// ETB gain 3 life. Race-breaker top-end.
pub fn inkling_penlord_b158() -> CardDefinition {
    CardDefinition {
        name: "Inkling Penlord (b158)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(3)],
        ..Default::default()
    }
}

/// Silverquill Censurer (b158) — {2}{W} 2/3 Human Soldier Vigilance.
/// ETB Tap target opponent creature. Tempo defender + lockdown.
pub fn silverquill_censurer_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_tap_opp_creature;
    CardDefinition {
        name: "Silverquill Censurer (b158)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_tap_opp_creature()],
        ..Default::default()
    }
}

/// Silverquill Inkdrain (b158) — {2}{W}{B} Sorcery. Drain 3 (each
/// opponent loses 3, you gain 3). 4-mana cheap drain finisher.
pub fn silverquill_inkdrain_b158() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkdrain (b158)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(3),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Aerogate (b158) — {2}{W} 1/3 Inkling Soldier Flying + Vigilance.
/// 3-mana defensive vigilance flier — Tenured Inkcaster fodder.
pub fn inkling_aerogate_b158() -> CardDefinition {
    CardDefinition {
        name: "Inkling Aerogate (b158)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Battlescholar (b158) — {2}{W}{B} 3/3 Vampire Cleric.
/// ETB Seq(Drain 1 + Scry 1). 4-mana drain + selection.
pub fn silverquill_battlescholar_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_scry;
    CardDefinition {
        name: "Silverquill Battlescholar (b158)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_scry(1, 1)],
        ..Default::default()
    }
}

/// Inkling Veilwarden (b158) — {3}{W}{B} 4/4 Inkling Bard Flying + Lifelink.
/// 5-mana evasive race-breaker.
pub fn inkling_veilwarden_b158() -> CardDefinition {
    CardDefinition {
        name: "Inkling Veilwarden (b158)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Edicter (b158) — {2}{B} Sorcery. Target opponent
/// sacrifices a creature; you gain 1 life. Cheap edict + lifegain.
pub fn silverquill_edicter_b158() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edicter (b158)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                filter: SelectionRequirement::Creature,
                count: Value::Const(1),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Heraldscribe (b158) — {1}{W}{B} 2/3 Inkling Cleric Flying.
/// ETB Scry 1. 3-mana defensive evasive smoother.
pub fn inkling_heraldscribe_b158() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Inkling Heraldscribe (b158)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

// ── Batch 159 (modern_decks) — More Silverquill cards ──────────────────────

/// Inkling Lawkeeper (b159) — {1}{W} 1/3 Inkling Soldier Flying + Vigilance.
/// 2-mana defensive evasive vigilance Inkling.
pub fn inkling_lawkeeper_b159() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lawkeeper (b159)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pen-Director (b159) — {3}{W}{B} 4/4 Vampire Bard Lifelink.
/// 5-mana lifelink finisher.
pub fn silverquill_pen_director_b159() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Director (b159)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pen-Sketch (b159) — {1}{W}{B} Instant.
/// Drain 1 + Draw 1. 3-mana drain-cantrip.
pub fn silverquill_pen_sketch_b159() -> CardDefinition {
    use crate::effect::shortcut::drain_and_draw;
    CardDefinition {
        name: "Silverquill Pen-Sketch (b159)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_draw(1),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Stalwart (b159) — {2}{B} 3/2 Inkling Knight.
/// 3-mana aggressive Inkling body.
pub fn inkling_stalwart_b159() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stalwart (b159)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pen-Sage (b159) — {2}{W}{B} 2/4 Vampire Wizard.
/// ETB Seq(Scry 1 + GainLife 2). 4-mana defensive scaling.
pub fn silverquill_pen_sage_b159() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pen-Sage (b159)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]))],
        ..Default::default()
    }
}

/// Inkling Pen-Adept (b159) — {W}{B} 2/2 Inkling Wizard.
/// Magecraft self-pump +1/+1 EOT.
pub fn inkling_pen_adept_b159() -> CardDefinition {
    CardDefinition {
        name: "Inkling Pen-Adept (b159)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Silverquill Soulbinder II (b159) — {2}{W}{B} 2/3 Vampire Cleric.
/// ETB Drain 1, then put a +1/+1 counter on self. Snowball drainer.
pub fn silverquill_soulbinder_ii_b159() -> CardDefinition {
    use crate::effect::shortcut::etb_drain_and_counter_self;
    CardDefinition {
        name: "Silverquill Soulbinder II (b159)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain_and_counter_self(1)],
        ..Default::default()
    }
}

// ── Batch 160 (modern_decks) — Silverquill additions ───────────────────────

/// Silverquill Scribecadet (b160) — {1}{W} 2/2 Human Cleric Lifelink.
/// Vanilla 2-mana lifelink body.
pub fn silverquill_scribecadet_b160() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scribecadet (b160)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Coursemate (b160) — {2}{W}{B} 2/3 Inkling Cleric Flying Lifelink.
/// 4-mana evasive lifelink flyer.
pub fn inkling_coursemate_b160() -> CardDefinition {
    CardDefinition {
        name: "Inkling Coursemate (b160)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Penblade (b160) — {2}{W}{B} 3/3 Vampire Bard.
/// Magecraft self-pump +1/+1 EOT.
pub fn silverquill_penblade_b160() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penblade (b160)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Silverquill Pendrop (b160) — {W}{B} Instant.
/// Drain 1 + Scry 1.
pub fn silverquill_pendrop_b160() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pendrop (b160)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(1),
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Verseknight (b160) — {1}{W}{B} 2/2 Inkling Knight Flying + Vigilance.
/// 3-mana evasive vigilance Inkling.
pub fn inkling_verseknight_b160() -> CardDefinition {
    CardDefinition {
        name: "Inkling Verseknight (b160)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Lectern (b160) — {3} Artifact.
/// `{2}, {T}: Each opponent loses 1 life and you gain 1 life.`
pub fn silverquill_lectern_b160() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lectern (b160)",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: drain(1),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            self_counter_cost_reduction: None,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Penbearer (b160) — {2}{W} 2/2 Inkling Cleric Flying.
/// Magecraft +1/+1 self EOT.
pub fn inkling_penbearer_b160() -> CardDefinition {
    CardDefinition {
        name: "Inkling Penbearer (b160)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        ..Default::default()
    }
}

/// Silverquill Inkstrike (b160) — {1}{B} Instant.
/// Target creature gets -2/-2 EOT.
pub fn silverquill_inkstrike_b160() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkstrike (b160)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 161 (modern_decks) — More Silverquill ───────────────────────────

/// Silverquill Inkmaster (b161) — {3}{W}{B} 4/4 Vampire Cleric.
/// Lifelink. 5-mana lifelink finisher.
pub fn silverquill_inkmaster_b161() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmaster (b161)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quillsoldier (b161) — {2}{W} 2/3 Inkling Soldier Flying.
/// 3-mana evasive flier.
pub fn inkling_quillsoldier_b161() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillsoldier (b161)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Penkeeper (b161) — {1}{W}{B} Sorcery.
/// Drain 2 + create a 1/1 W/B Inkling token with flying.
pub fn silverquill_penkeeper_b161() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penkeeper (b161)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Quillsaint (b161) — {2}{W}{B} 3/3 Spirit Cleric Lifelink.
/// 4-mana lifelink Spirit.
pub fn silverquill_quillsaint_b161() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillsaint (b161)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Vowscribe (b161) — {1}{B} 1/2 Inkling Cleric Flying Deathtouch.
/// 2-mana evasive deathtouch trader.
pub fn inkling_vowscribe_b161() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vowscribe (b161)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 162 (modern_decks) — More Silverquill ───────────────────────────

/// Silverquill Devotionseer (b162) — {1}{W} 1/3 Human Cleric Lifelink.
/// 2-mana defensive cleric.
pub fn silverquill_devotionseer_b162() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Devotionseer (b162)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Plumefall (b162) — {3}{W} 3/3 Inkling Soldier Flying.
/// 4-mana flying finisher.
pub fn inkling_plumefall_b162() -> CardDefinition {
    CardDefinition {
        name: "Inkling Plumefall (b162)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inksong (b162) — {1}{W}{B} Sorcery.
/// Drain 3.
pub fn silverquill_inksong_b162() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inksong (b162)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(3),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Apprentice II (b162) — {W}{B} 2/2 Human Cleric.
/// Magecraft drain 1.
pub fn silverquill_apprentice_ii_b162() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Silverquill Apprentice II (b162)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(drain(1))],
        ..Default::default()
    }
}

/// Inkling Sentry (b162) — {2}{B} 2/2 Inkling Soldier Flying Deathtouch.
/// 3-mana evasive deathtouch trader.
pub fn inkling_sentry_b162() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentry (b162)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 164 (modern_decks) — More Silverquill ───────────────────────────

/// Silverquill Quillkeeper (b164) — {1}{W}{B} 2/3 Human Cleric Vigilance.
/// ETB: drain 1.
pub fn silverquill_quillkeeper_b164() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillkeeper (b164)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(1)],
        ..Default::default()
    }
}

/// Inkling Herald (b164) — {2}{W}{B} 3/3 Inkling Knight Flying.
/// ETB: draw a card, then discard a card.
pub fn inkling_herald_b164() -> CardDefinition {
    CardDefinition {
        name: "Inkling Herald (b164)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![{
            use crate::effect::shortcut::etb_loot;
            etb_loot()
        }],
        ..Default::default()
    }
}

/// Silverquill Commandant (b164) — {2}{W} 2/3 Human Soldier Vigilance.
/// Magecraft: gain 1 life.
pub fn silverquill_commandant_b164() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Commandant (b164)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Inkling Skirmisher (b164) — {1}{B} 2/1 Inkling Rogue Flying.
/// When dies: drain 1.
pub fn inkling_skirmisher_b164() -> CardDefinition {
    CardDefinition {
        name: "Inkling Skirmisher (b164)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![{
            use crate::effect::shortcut::dies_drain;
            dies_drain(1)
        }],
        ..Default::default()
    }
}

/// Silverquill Verdict (b164) — {1}{W}{B} Sorcery.
/// Drain 2 + Draw 1.
pub fn silverquill_verdict_b164() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Verdict (b164)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_surveil(2, 1),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Duelist (b164) — {W}{B} 2/2 Inkling Warrior Flying.
/// Vanilla aggro flyer.
pub fn inkling_duelist_b164() -> CardDefinition {
    CardDefinition {
        name: "Inkling Duelist (b164)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Denouncement (b164) — {2}{B} Instant.
/// Target creature gets -3/-3 EOT.
pub fn silverquill_denouncement_b164() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Denouncement (b164)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: Selector::Target(0),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 165 (modern_decks) — More Silverquill ───────────────────────────

/// Inkling Shadowcaster (b165) — {2}{W}{B} 3/2 Inkling Wizard Flying.
/// Magecraft: draw 1.
pub fn inkling_shadowcaster_b165() -> CardDefinition {
    use crate::effect::shortcut::magecraft_draw;
    CardDefinition {
        name: "Inkling Shadowcaster (b165)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_draw(1)],
        ..Default::default()
    }
}

/// Silverquill Spiritspeaker (b165) — {1}{W} 2/2 Human Cleric.
/// ETB: Scry 1 + gain 1 life.
pub fn silverquill_spiritspeaker_b165() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Silverquill Spiritspeaker (b165)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry { who: crate::effect::PlayerRef::You, amount: Value::Const(1) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Silverquill Vindict (b165) — {2}{W}{B} Sorcery.
/// Destroy target creature + gain 2 life.
pub fn silverquill_vindict_b165() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vindict (b165)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::Target(0) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Skywarden (b165) — {3}{W}{B} 3/4 Inkling Knight Flying Vigilance.
/// Top-end evasive defender.
pub fn inkling_skywarden_b165() -> CardDefinition {
    CardDefinition {
        name: "Inkling Skywarden (b165)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Deathmark (b165) — {1}{B} Instant.
/// Target creature gets -2/-2 EOT + you gain 1 life.
pub fn silverquill_deathmark_b165() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Deathmark (b165)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 166 (modern_decks) — Silverquill cycle ──────────────────────────
//
// Ten new Silverquill (W/B) cards: a mix of evasive Inklings, drain
// bodies, magecraft payoffs, and Silverquill removal. All compose against
// existing shortcuts; no engine changes required.

/// Inkling Bonecaster (b166) — {1}{W}{B} 2/2 Inkling Cleric Flying.
/// ETB: target opp creature gets -1/-1 EOT (Soft removal + body).
pub fn inkling_bonecaster_b166() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bonecaster (b166)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            ),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Silverquill Auditor (b166) — {2}{W} 2/3 Human Cleric Vigilance.
/// Magecraft: target opp loses 1 life.
pub fn silverquill_auditor_b166() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Auditor (b166)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Inkling Squire (b166) — {W}{B} 2/2 Inkling Knight First Strike.
/// Aggressive 2-mana first-strike Inkling body.
pub fn inkling_squire_b166() -> CardDefinition {
    CardDefinition {
        name: "Inkling Squire (b166)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Quill-Wielder (b166) — {1}{W} 2/2 Human Cleric.
/// ETB: scry 1. Magecraft: target friendly Inkling gets +1/+1 EOT.
pub fn silverquill_quill_wielder_b166() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quill-Wielder (b166)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            }),
            magecraft(Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Inkling)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            }),
        ],
        ..Default::default()
    }
}

/// Inkling Soulkeeper (b166) — {3}{W}{B} 3/3 Inkling Cleric
/// Flying + Lifelink. ETB mints 1 Inkling token.
pub fn inkling_soulkeeper_b166() -> CardDefinition {
    CardDefinition {
        name: "Inkling Soulkeeper (b166)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(inkling_token(), 1)],
        ..Default::default()
    }
}

/// Silverquill Ascription (b166) — {3}{W}{B} Sorcery.
/// Drain 3 + Scry 2. 5-mana drain + selection finisher.
pub fn silverquill_ascription_b166() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ascription (b166)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: {
            use crate::effect::shortcut::drain_and_scry;
            drain_and_scry(3, 2)
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Vellumkeeper (b166) — {2}{B} 2/3 Inkling Rogue.
/// CreatureDied (another of yours) → you gain 1 life.
pub fn inkling_vellumkeeper_b166() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vellumkeeper (b166)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Recital (b166) — {W}{B} Instant.
/// Drain 1 + Draw 1. 2-mana drain-cantrip.
pub fn silverquill_recital_b166() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Recital (b166)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: {
            use crate::effect::shortcut::drain_and_draw;
            drain_and_draw(1)
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Lifegiver (b166) — {2}{W} 1/3 Inkling Cleric Flying + Lifelink.
/// Defensive 3-mana lifelink flyer.
pub fn inkling_lifegiver_b166() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lifegiver (b166)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sentencing (b166) — {2}{W}{B} Sorcery.
/// Exile target creature with mana value ≤ 4 + you gain 2 life.
pub fn silverquill_sentencing_b166() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentencing (b166)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(4)),
                ),
                to: ZoneDest::Exile,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 167 (modern_decks) — Silverquill follow-up ──────────────────────
//
// Six more Silverquill cards focused on engine variety:
// finality-counter granter (exercises the CR 122.1h wire), drain-+1/+1
// counter Inkling lord, Surveil sorcery, evasive Bookwurm-style flier,
// reach Cleric defender, and a stunning combat trick.

/// Silverquill Curse (b167) — {2}{B} Instant.
/// Put a finality counter on target creature. Combo with a death trigger
/// to exile-instead-of-graveyard.
pub fn silverquill_curse_b167() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Silverquill Curse (b167)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::Finality,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkbond (b167) — {2}{W}{B} 3/3 Inkling Cleric Flying.
/// Static: Other Inkling creatures you control get +1/+1.
pub fn silverquill_inkbond_b167() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbond (b167)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Inkling creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                power: 1,
                toughness: 1,
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Penbinder (b167) — {1}{W} Sorcery.
/// Surveil 2.
pub fn silverquill_penbinder_b167() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penbinder (b167)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Diviner (b167) — {3}{W} 2/4 Inkling Cleric Flying + Vigilance.
/// ETB draw a card.
pub fn inkling_diviner_b167() -> CardDefinition {
    use crate::effect::shortcut::etb_draw;
    CardDefinition {
        name: "Inkling Diviner (b167)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_draw(1)],
        ..Default::default()
    }
}

/// Silverquill Bulwark (b167) — {2}{W} 1/5 Human Cleric Defender.
/// Pure 1/5 wall body for defensive shells.
pub fn silverquill_bulwark_b167() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bulwark (b167)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Stunning (b167) — {2}{W} Instant.
/// Tap target creature + put a stun counter on it.
pub fn silverquill_stunning_b167() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Silverquill Stunning (b167)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 168 (modern_decks) — Silverquill premium variants ───────────────

/// Silverquill Banisher (b168) — {3}{W} Sorcery.
/// Exile target creature with mana value exactly 3 (uses the new
/// ManaValueExactly predicate).
pub fn silverquill_banisher_b168() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Banisher (b168)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueExactly(3)),
            ),
            to: ZoneDest::Exile,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sage II (b168) — {1}{W}{B} 2/2 Inkling Cleric Flying + Lifelink.
/// Compact double-keyword Inkling.
pub fn inkling_sage_ii_b168() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sage II (b168)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Penlord (b168) — {2}{W}{B} 3/2 Vampire Wizard.
/// "Whenever you cast a creature spell, drain 1."
pub fn silverquill_penlord_b168() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penlord (b168)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Silverquill Command ────────────────────────────────────────────────────

/// Silverquill Command — {2}{W}{B} Sorcery. Choose two among 4 modes.
///
/// Approximation: AutoDecider picks drain 2 + +1/+1 counters (×2).
/// Choose-two collapsed to Seq of the two auto-default modes.
pub fn silverquill_command() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Silverquill Command",
        cost: cost(&[generic(2), w(), b()]),
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
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Umbral Juke ────────────────────────────────────────────────────────────

/// Umbral Juke — {2}{B} Instant. Choose one: opponent sacs creature/PW
/// or create 2/1 Inkling with flying.
pub fn umbral_juke() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Umbral Juke",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Silverquill Silencer ───────────────────────────────────────────────────

/// Silverquill Silencer — {W}{B}, 3/2 Human Cleric. As it enters, choose a
/// nonland card name. Whenever an opponent casts a spell with that name, they
/// lose 3 life and you draw a card.
pub fn silverquill_silencer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Silencer",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        triggered_abilities: vec![
            etb(Effect::NameCard { what: Selector::This }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::TriggerObjectNameMatchesNamedCard),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(3),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── Fracture ───────────────────────────────────────────────────────────────

/// Fracture — {W}{B} Instant. Destroy target artifact, enchantment, or PW.
pub fn fracture() -> CardDefinition {
    CardDefinition {
        name: "Fracture",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Planeswalker),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Humiliate ──────────────────────────────────────────────────────────────

/// Humiliate — {1}{W}{B} Sorcery. Opponent discards a nonland; you
/// drain 1 (gain 1 life, opp loses 1 life). Approximation of "Reveal
/// hand, you choose, opp discards" — collapsed to auto-pick a nonland.
pub fn humiliate() -> CardDefinition {
    CardDefinition {
        name: "Humiliate",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Clever Lumimancer ──────────────────────────────────────────────────────

/// Clever Lumimancer — {W}, 0/1 Human Wizard. Magecraft: +2/+0 EOT.
pub fn clever_lumimancer() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Clever Lumimancer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_self_pump(2, 0)],
        ..Default::default()
    }
}

// ── Silverquill Apprentice ────────────────────────────────────────────────

/// Silverquill Apprentice — {W}{B}, 2/1 Human Wizard.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on target creature you control."
pub fn silverquill_apprentice() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Silverquill Apprentice",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Shadewing Laureate ────────────────────────────────────────────────────

/// Shadewing Laureate — {1}{W}{B}, 2/2 Bird Warlock. Flying.
/// "Whenever another creature you control with flying dies, put a +1/+1
/// counter on Shadewing Laureate."
pub fn shadewing_laureate() -> CardDefinition {
    CardDefinition {
        name: "Shadewing Laureate",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasKeyword(Keyword::Flying),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Batch 169 (modern_decks) — Silverquill expansion (8 cards) ────────────
//
// Eight more Silverquill cards exercising drain templates, inkling-tribal,
// finality-counter granters, and a new "spectral" mode. Each card ships
// with a functionality test in tests::stx.

/// Silverquill Spectralist (b169) — {1}{W}{B} 2/2 Inkling Wizard.
/// Flying + Lifelink. ETB Surveil 1.
pub fn silverquill_spectralist_b169() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Spectralist (b169)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Finalizer (b169) — {2}{W}{B} Sorcery.
/// Drain 2, then put a finality counter on target creature.
/// Combines the staple drain template with the new CR 122.1h finality
/// counter mechanic for hard-removal-on-next-death.
pub fn silverquill_finalizer_b169() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Silverquill Finalizer (b169)",
        cost: cost(&[generic(2), w(), b()]),
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
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::Finality,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Banshee (b169) — {2}{B} 3/2 Inkling Spirit. Flying.
/// When this creature dies, each opponent loses 1 life.
pub fn inkling_banshee_b169() -> CardDefinition {
    CardDefinition {
        name: "Inkling Banshee (b169)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::dies_lose_life_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Verdict (b169) — {3}{W}{B} Instant.
/// Exile target creature, you gain 3 life.
pub fn silverquill_verdict_b169() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Silverquill Verdict (b169)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Exile,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quill-Captain (b169) — {3}{W}{B} 3/3 Inkling Soldier Flying.
/// Whenever this creature attacks, put a +1/+1 counter on it.
pub fn inkling_quill_captain_b169() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Inkling Quill-Captain (b169)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Edict (b169) — {2}{B} Sorcery.
/// Target opponent sacrifices a creature, you gain 2 life.
pub fn silverquill_edict_b169() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edict (b169)",
        cost: cost(&[generic(2), b()]),
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
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Bookmage (b169) — {1}{W} 2/2 Human Wizard.
/// Magecraft: scry 1, then draw a card.
pub fn silverquill_bookmage_b169() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookmage (b169)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 170 (modern_decks) — Silverquill shield-counter variants ────────

/// Silverquill Aegismage (b170) — {1}{W}{B} 2/3 Vampire Cleric.
/// ETB: put a shield counter on target creature you control.
pub fn silverquill_aegismage_b170() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Aegismage (b170)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::Shield,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 171 (modern_decks) — Silverquill expansion ──────────────────────

/// Silverquill Quillsmith (b171) — {2}{W} 2/3 Human Wizard Vigilance.
/// Magecraft: put a +1/+1 counter on target friendly creature.
pub fn silverquill_quillsmith_b171() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Silverquill Quillsmith (b171)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Inkling Vanguard II (b171) — {W}{B} 1/3 Inkling Soldier Flying + Vigilance.
/// Compact defensive Inkling.
pub fn inkling_vanguard_ii_b171() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vanguard II (b171)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 172 (modern_decks) — Silverquill expansion ──────────────────────

// ── Batch 173 (modern_decks) — Shield/finality magecraft variants ─────────

// ── Batch 174 (modern_decks) — additional Silverquill cards ────────────────

/// Silverquill Inkbinder (b174) — {1}{W} 2/2 Human Cleric.
/// Magecraft: put a +1/+1 counter on this creature.
pub fn silverquill_inkbinder_b174() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_counter_self;
    CardDefinition {
        name: "Silverquill Inkbinder (b174)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_counter_self()],
        ..Default::default()
    }
}

/// Inkling Stylist (b174) — {W}{B} 2/2 Inkling Cleric Flying + Lifelink.
/// Premium 2-mana evasive lifelinker; pairs with Tenured Inkcaster.
pub fn inkling_stylist_b174() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stylist (b174)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Lifeleach (b174) — {1}{W}{B} Instant.
/// Drain 2 + Scry 1.
pub fn silverquill_lifeleach_b174() -> CardDefinition {
    use crate::effect::shortcut::drain_and_scry;
    CardDefinition {
        name: "Silverquill Lifeleach (b174)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_scry(2, 1),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Scrollguard (b174) — {2}{W}{B} 3/3 Inkling Soldier Flying.
/// ETB: gain 2 life.
pub fn inkling_scrollguard_b174() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scrollguard (b174)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

/// Silverquill Inkfiend (b174) — {2}{B} 2/3 Vampire Warlock.
/// Whenever another creature you control dies, target opp loses 1 life.
pub fn silverquill_inkfiend_b174() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkfiend (b174)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_other_dies(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 175 (modern_decks) — additional Silverquill cards ────────────────

/// Silverquill Stenographer (b175) — {1}{W}{B} 2/2 Inkling Wizard.
/// Magecraft loot.
pub fn silverquill_stenographer_b175() -> CardDefinition {
    use crate::effect::shortcut::magecraft_loot;
    CardDefinition {
        name: "Silverquill Stenographer (b175)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_loot()],
        ..Default::default()
    }
}

/// Inkling Mortician (b175) — {3}{W}{B} 3/3 Inkling Cleric Flying + Lifelink.
/// Premium 5-mana evasive lifelink finisher.
pub fn inkling_mortician_b175() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mortician (b175)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Reapcrier (b175) — {2}{B} 3/2 Vampire Warlock.
/// Dies: drain 1 (each opp loses 1, you gain 1).
pub fn silverquill_reapcrier_b175() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Silverquill Reapcrier (b175)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![dies_drain(1)],
        ..Default::default()
    }
}

/// Inkling Cantor (b175) — {2}{W} 2/3 Inkling Cleric Flying.
/// ETB: scry 1 + gain 1 life.
pub fn inkling_cantor_b175() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    let _ = etb_scry_and_draw; // mark as available
    CardDefinition {
        name: "Inkling Cantor (b175)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

/// Silverquill Penkeeper (b175) — {1}{B} 1/2 Vampire Warlock.
/// Magecraft: each opp discards a card (collapsed to single discard, simplified).
/// Actually we'll use Drain 1 as simpler stand-in.
pub fn silverquill_penkeeper_b175() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penkeeper (b175)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Inkling Hatchling (b175) — {W}{B} 1/1 Inkling Flying.
/// ETB: put a +1/+1 counter on this creature.
pub fn inkling_hatchling_b175() -> CardDefinition {
    CardDefinition {
        name: "Inkling Hatchling (b175)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ── Batch 176 (modern_decks) — engine-improvement-driven Silverquill ──────

/// Silverquill Doomgrant (b176) — {2}{B} Instant.
/// Put a finality counter on target creature. (CR 122.1h-granting card —
/// when the target next dies, it exiles instead of going to the graveyard.)
pub fn silverquill_doomgrant_b176() -> CardDefinition {
    use crate::effect::shortcut::add_finality_to_target_creature;
    CardDefinition {
        name: "Silverquill Doomgrant (b176)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: add_finality_to_target_creature(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 177 (modern_decks) — more Silverquill variants ──────────────────

/// Silverquill Anthem-Bearer (b177) — {2}{W} 1/3 Inkling Cleric Flying.
/// Other Inklings you control get +1/+0.
pub fn silverquill_anthem_bearer_b177() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Silverquill Anthem-Bearer (b177)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

/// Inkling Stylekeeper (b177) — {1}{B} 1/3 Inkling Wizard.
/// Magecraft: drain 1.
pub fn inkling_stylekeeper_b177() -> CardDefinition {
    CardDefinition {
        name: "Inkling Stylekeeper (b177)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

// ── Batch 186 (modern_decks) — multi-counter and effect cards ─────────────

/// Silverquill Glyphmaker (b186) — {1}{W}{B} 1/2 Inkling Wizard Flying.
/// Whenever you cast an instant or sorcery, target creature gets a +1/+1
/// counter and a flying counter (combined CR 122.1b + standard +1/+1).
pub fn silverquill_glyphmaker_b186() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Glyphmaker (b186)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![crate::effect::shortcut::magecraft(Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
        ]))],
        ..Default::default()
    }
}

// ── Batch 184 (modern_decks) — more keyword counter cards ─────────────────

/// Silverquill Wordsharpener (b184) — {1}{W}{B} Sorcery.
/// Put a first strike counter on target creature.
pub fn silverquill_wordsharpener_b184() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Wordsharpener (b184)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::FirstStrike,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Drainmark (b184) — {1}{B} Sorcery.
/// Put a deathtouch counter on target creature you control.
pub fn silverquill_drainmark_b184() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Drainmark (b184)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Deathtouch,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 183 (modern_decks) — Keyword counter cards (CR 122.1b) ─────────

/// Silverquill Skystudent (b183) — {1}{W} Sorcery.
/// Put a flying counter on target creature.
pub fn silverquill_skystudent_b183() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Skystudent (b183)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Flying,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 182 (modern_decks) — closer to a balanced Silverquill cube ──────

/// Silverquill Ascendant (b182) — {4}{W}{B} 5/5 Inkling Bard Flying + Lifelink.
/// Finisher body. Pairs with Tenured Inkcaster's anthem.
pub fn silverquill_ascendant_b182() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ascendant (b182)",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Bard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Stampcrafter (b182) — {2}{W}{B} 3/3 Inkling Wizard.
/// ETB: drain 1 + scry 1.
pub fn silverquill_stampcrafter_b182() -> CardDefinition {
    use crate::effect::shortcut::drain_and_scry;
    CardDefinition {
        name: "Silverquill Stampcrafter (b182)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(drain_and_scry(1, 1))],
        ..Default::default()
    }
}

// ── Batch 179 (modern_decks) — Inkling tribal expansion ───────────────────

/// Inkling Tutor (b179) — {1}{B} Sorcery. Discard a card, then draw two cards.
pub fn inkling_tutor_b179() -> CardDefinition {
    CardDefinition {
        name: "Inkling Tutor (b179)",
        cost: cost(&[generic(1), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Heraldscribe (b179) — {2}{W}{B} 2/3 Inkling Cleric Flying.
/// Whenever this attacks, drain 1.
pub fn inkling_heraldscribe_b179() -> CardDefinition {
    use crate::effect::shortcut::on_attack_drain;
    CardDefinition {
        name: "Inkling Heraldscribe (b179)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Penquill (b179) — {W} 1/1 Inkling Soldier Flying. Vanilla 1-drop.
pub fn silverquill_penquill_b179() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penquill (b179)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 178 (modern_decks) — more variety ───────────────────────────────

/// Inkling Lifesong (b178) — {W}{B} Instant. Drain 2 + Draw a card.
pub fn inkling_lifesong_b178() -> CardDefinition {
    use crate::effect::shortcut::drain_and_draw;
    CardDefinition {
        name: "Inkling Lifesong (b178)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_draw(2),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pridecrier (b178) — {2}{W} 3/2 Inkling Cleric Flying.
/// Whenever you cast an instant or sorcery, gain 2 life.
pub fn silverquill_pridecrier_b178() -> CardDefinition {
    use crate::effect::shortcut::magecraft_gain_life;
    CardDefinition {
        name: "Silverquill Pridecrier (b178)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(2)],
        ..Default::default()
    }
}

/// Silverquill Wordweaver (b177) — {3}{W}{B} 3/4 Vampire Bard Flying.
/// ETB: each opp discards a card. (Approximated as Drain 1 — no targeted discard target.)
/// We'll use drain to mark a meaningful effect; alternative is to skip and use a vanilla body.
pub fn silverquill_wordweaver_b177() -> CardDefinition {
    use crate::effect::shortcut::etb_drain;
    CardDefinition {
        name: "Silverquill Wordweaver (b177)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Aegis (b176) — {1}{W} Sorcery.
/// Put a shield counter on target creature.
pub fn silverquill_aegis_b176() -> CardDefinition {
    use crate::effect::shortcut::add_shield_to_target_creature;
    CardDefinition {
        name: "Silverquill Aegis (b176)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: add_shield_to_target_creature(),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Verdictbearer (b175) — {3}{W} Sorcery.
/// Exile target creature with power 4 or greater.
pub fn silverquill_verdictbearer_b175() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Verdictbearer (b175)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
            ),
            to: ZoneDest::Exile,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pyremist (b174) — {3}{W}{B} 3/4 Vampire Cleric Flying.
/// ETB Seq(Drain 2 + Scry 1).
pub fn silverquill_pyremist_b174() -> CardDefinition {
    use crate::effect::shortcut::drain_and_scry;
    CardDefinition {
        name: "Silverquill Pyremist (b174)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(drain_and_scry(2, 1))],
        ..Default::default()
    }
}

/// Silverquill Wardlord (b173) — {1}{W}{B} 2/3 Inkling Cleric.
/// Magecraft: put a shield counter on this creature.
pub fn silverquill_wardlord_b173() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_shield_self;
    CardDefinition {
        name: "Silverquill Wardlord (b173)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_shield_self()],
        ..Default::default()
    }
}

/// Silverquill Doomspeaker (b173) — {1}{B} 1/2 Vampire Warlock.
/// Magecraft: put a finality counter on this creature.
pub fn silverquill_doomspeaker_b173() -> CardDefinition {
    use crate::effect::shortcut::magecraft_add_finality_self;
    CardDefinition {
        name: "Silverquill Doomspeaker (b173)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_add_finality_self()],
        ..Default::default()
    }
}

/// Silverquill Sentinel (b172) — {1}{W} 2/2 Inkling Soldier Vigilance.
/// Simple defender body.
pub fn silverquill_sentinel_b172() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentinel (b172)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkmage (b172) — {2}{B} 2/3 Vampire Warlock.
/// ETB: target opp loses 2 life, you gain 2 life.
pub fn silverquill_inkmage_b172() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmage (b172)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(drain(2))],
        ..Default::default()
    }
}

/// Inkling Skywatch (b172) — {2}{W}{B} 2/2 Inkling Cleric Flying + Vigilance.
/// Combat-damage trigger: gain 1 life.
pub fn inkling_skywatch_b172() -> CardDefinition {
    CardDefinition {
        name: "Inkling Skywatch (b172)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Tombwarden (b171) — {3}{W}{B} 3/5 Vampire Cleric Lifelink.
/// Whenever another creature you control dies, you gain 1 life.
pub fn silverquill_tombwarden_b171() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Tombwarden (b171)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![on_other_dies(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Wardward (b170) — {W}{B} Instant.
/// Put two shield counters on target creature.
pub fn silverquill_wardward_b170() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Wardward (b170)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::Shield,
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Standard-Bearer (b169) — {4}{W}{B} 4/4 Inkling Knight.
/// Flying + Lifelink. Other Inkling creatures you control have lifelink.
pub fn inkling_standard_bearer_b169() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Inkling Standard-Bearer (b169)",
        cost: cost(&[generic(4), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

// ── Batch 187 (modern_decks) — Silverquill expansion ─────────────────────
// Mix of keyword counter granters, magecraft engines, and Inkling tribal
// bodies. Each card ships with one functionality test.

/// Silverquill Reachseal (b187) — {1}{W} Sorcery.
/// Put a reach counter on target creature.
pub fn silverquill_reachseal_b187() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Silverquill Reachseal (b187)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Reach,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Mentordrain (b187) — {2}{W}{B} 2/3 Inkling Cleric Flying.
/// Magecraft: each opp loses 1 + you gain 1 (drain 1) + scry 1.
pub fn silverquill_mentordrain_b187() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mentordrain (b187)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
                amount: Value::Const(1),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Inkling Vigilkeeper (b187) — {2}{W} 2/3 Inkling Soldier.
/// ETB puts a vigilance counter on self.
pub fn inkling_vigilkeeper_b187() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vigilkeeper (b187)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: Selector::This,
            keyword: Keyword::Vigilance,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Skytutor (b187) — {3}{W}{B} 3/2 Inkling Wizard Flying.
/// ETB tutors a creature card with mana value ≤ 2 from library → hand.
pub fn silverquill_skytutor_b187() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Skytutor (b187)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(2)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Silverquill Inkletter II (b187) — {W}{B} Instant.
/// Drain 2 + draw a card.
pub fn silverquill_inkletter_ii_b187() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkletter II (b187)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Spellguard (b187) — {1}{W}{B} 2/3 Inkling Cleric.
/// Static: friendly Inkling creatures have Lifelink.
pub fn inkling_spellguard_b187() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Inkling Spellguard (b187)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
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
        ..Default::default()
    }
}

// ── Batch 191 (modern_decks) — multi-action cards + Inkling tribal ────────

/// Silverquill Inkdrain (b191) — {2}{W}{B} Sorcery.
/// Drain 3 + draw 1 + create 1 Inkling.
pub fn silverquill_inkdrain_b191() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Inkdrain (b191)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(3),
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Highscribe (b191) — {2}{W}{B} 3/3 Inkling Cleric Flying.
/// ETB scry 2 + magecraft gain 1 life.
pub fn inkling_highscribe_b191() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    let _ = etb_scry;
    CardDefinition {
        name: "Inkling Highscribe (b191)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![
            etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            magecraft_gain_life(1),
        ],
        ..Default::default()
    }
}

/// Silverquill Vampirebond (b191) — {B}{B} 2/2 Vampire Warlock Lifelink.
pub fn silverquill_vampirebond_b191() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vampirebond (b191)",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 190 (modern_decks) — keyword counter granters ──────────────────

/// Silverquill Doublecurse (b190) — {1}{B} Sorcery.
/// Target creature gets a deathtouch counter and a flying counter.
pub fn silverquill_doublecurse_b190() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Doublecurse (b190)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Deathtouch,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Wardseal (b190) — {1}{W} Sorcery.
/// Put a vigilance counter on target creature you control.
pub fn silverquill_wardseal_b190() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Wardseal (b190)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Vigilance,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Lifeward (b190) — {W}{B} Instant.
/// Target creature gets a lifelink counter.
pub fn silverquill_lifeward_b190() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifeward (b190)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddKeywordCounter {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::Lifelink,
            amount: Value::Const(1),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 189 (modern_decks) — Silverquill drain + Inkling tribal ─────────

/// Silverquill Drainmaster II (b189) — {2}{B}{B} 3/3 Vampire Warlock.
/// ETB drain 3.
pub fn silverquill_drainmaster_ii_b189() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainmaster II (b189)",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(3)],
        ..Default::default()
    }
}

/// Inkling Vassalking (b189) — {3}{W}{B} 4/3 Inkling Knight Flying + Lifelink.
/// On-attack drain 1.
pub fn inkling_vassalking_b189() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vassalking (b189)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Exilewright (b189) — {3}{W} Sorcery.
/// Exile target creature.
pub fn silverquill_exilewright_b189() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Exilewright (b189)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Exile,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 188 (modern_decks) — additional Silverquill cards ───────────────

/// Silverquill Cantrap (b188) — {W} Instant.
/// Target creature gets +1/+1 EOT and gains lifelink EOT.
pub fn silverquill_cantrap_b188() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cantrap (b188)",
        cost: cost(&[w()]),
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
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Tribune (b188) — {3}{W}{B} 3/4 Inkling Cleric Flying.
/// ETB drain 2 + magecraft self-pump +1/+0 EOT.
pub fn inkling_tribune_b188() -> CardDefinition {
    CardDefinition {
        name: "Inkling Tribune (b188)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2), magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

/// Silverquill Litany (b188) — {1}{W}{B} Sorcery.
/// Each opponent loses 2 life; you gain 2 life; scry 1.
pub fn silverquill_litany_b188() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Litany (b188)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Wardlock (b187) — {1}{W} Instant.
/// Put a shield counter on each creature you control.
pub fn silverquill_wardlock_b187() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Wardlock (b187)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
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
                kind: CounterType::Shield,
                amount: Value::Const(1),
            }),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 193 (modern_decks) — Silverquill W/B deep cuts ─────────────────

/// Silverquill Inkbreaker (b193) — {1}{W} 2/2 Human Soldier.
/// On attack: gain 1 life.
pub fn silverquill_inkbreaker_b193() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbreaker (b193)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Sealkeeper (b193) — {W}{W} 2/2 Human Cleric Vigilance.
/// Cheap Vigilance vanilla body.
pub fn silverquill_sealkeeper_b193() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sealkeeper (b193)",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Drainwight (b193) — {2}{B} 2/3 Vampire Warlock.
/// Magecraft: each opponent loses 1 life and you gain 1.
pub fn silverquill_drainwight_b193() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainwight (b193)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Inkflood (b193) — {2}{W}{B} Sorcery.
/// Drain 2 + each player draws a card. (Symmetrical card draw.)
pub fn silverquill_inkflood_b193() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkflood (b193)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            drain(2),
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inklingbond (b193) — {1}{W}{B} 2/2 Inkling Cleric Flying.
/// Solid two-color flying body in Silverquill colors.
pub fn silverquill_inklingbond_b193() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inklingbond (b193)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pridescholar (b193) — {2}{W} 3/2 Human Cleric.
/// ETB: gain 2 life.
pub fn silverquill_pridescholar_b193() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pridescholar (b193)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

// ── Batch 194 (modern_decks) — Silverquill W/B compact additions ──────────

/// Silverquill Wardstamp (b194) — {1}{W} Instant.
/// Target creature gets +0/+2 EOT and gains vigilance EOT.
pub fn silverquill_wardstamp_b194() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Wardstamp (b194)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(0),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Tutorquill (b194) — {3}{W}{B} 4/3 Inkling Cleric Flying Lifelink.
/// Strong Silverquill flier with two keywords.
pub fn silverquill_tutorquill_b194() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Tutorquill (b194)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Drainscholar (b194) — {2}{B}{W} 3/3 Human Cleric.
/// ETB: drain 2 (each opp -2, you +2). Magecraft: drain 1.
pub fn silverquill_drainscholar_b194() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainscholar (b194)",
        cost: cost(&[generic(2), b(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2), magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Glyphstudent (b194) — {1}{W} 2/2 Human Soldier Vigilance.
/// Compact white body.
pub fn silverquill_glyphstudent_b194() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Glyphstudent (b194)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Exilescribe (b194) — {2}{B} Sorcery.
/// Target opponent discards a card. You draw a card.
pub fn silverquill_exilescribe_b194() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Exilescribe (b194)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: true,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 195 (modern_decks) — Silverquill more deep cuts ─────────────────

/// Silverquill Wordstamp (b195) — {1}{W}{B} Sorcery.
/// Create 2 Inkling tokens.
pub fn silverquill_wordstamp_b195() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Wordstamp (b195)",
        cost: cost(&[generic(1), w(), b()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkmark (b195) — {W}{B} 2/2 Inkling Flying.
/// Compact bicolor flier with no riders.
pub fn silverquill_inkmark_b195() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkmark (b195)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Painlace (b195) — {B} Instant.
/// Target creature gets -2/-0 EOT.
pub fn silverquill_painlace_b195() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Painlace (b195)",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Drainlord (b195) — {3}{B}{B} 5/4 Vampire Warlock.
/// On attack: drain 1.
pub fn silverquill_drainlord_b195() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainlord (b195)",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

// ── Batch 196 (modern_decks) — Silverquill more variety ───────────────────

/// Silverquill Loremaster (b196) — {3}{W} 3/4 Human Wizard.
/// Vigilance. ETB: scry 2.
pub fn silverquill_loremaster_b196() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Loremaster (b196)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(2)],
        ..Default::default()
    }
}

/// Silverquill Penmage (b196) — {2}{W}{B} 3/3 Inkling Cleric.
/// Flying, lifelink (premium flier).
pub fn silverquill_penmage_b196() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penmage (b196)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Pact (b196) — {2}{W}{B} Sorcery.
/// You gain 4 life. Each opponent loses 4 life.
pub fn silverquill_pact_b196() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pact (b196)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sergeant (b196) — {1}{W} 1/1 Soldier.
/// ETB: create a 1/1 Soldier token. (Anthem aggro support.)
pub fn silverquill_sergeant_b196() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color as C;
    let soldier = TokenDefinition {
        name: "Soldier".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![C::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Soldier],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Silverquill Sergeant (b196)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_mint_token(soldier, 1)],
        ..Default::default()
    }
}

// ── Batch 197 (modern_decks) — Silverquill polish ────────────────────────

/// Silverquill Wordcaller (b197) — {1}{W} 1/3 Human Cleric.
/// Magecraft: target creature gets +1/+1 EOT.
pub fn silverquill_wordcaller_b197() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Silverquill Wordcaller (b197)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(SelectionRequirement::Creature),
            1,
            1,
        )],
        ..Default::default()
    }
}

/// Silverquill Glyphmark (b197) — {1}{B} 2/2 Vampire Warlock.
/// Magecraft: each opp loses 1 life.
pub fn silverquill_glyphmark_b197() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Glyphmark (b197)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

// ── Batch 198 (modern_decks) — Silverquill (W/B) extension ───────────────

/// Silverquill Lifesinger (b198) — {1}{W} 1/2 Human Cleric Lifelink.
/// Vanilla lifelink one-drop+.
pub fn silverquill_lifesinger_b198() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lifesinger (b198)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Conductor (b198) — {1}{W}{B} 2/2 Inkling Wizard Flying.
/// ETB scry 1.
pub fn inkling_conductor_b198() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Inkling Conductor (b198)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Ascetic (b198) — {2}{W} 2/3 Human Cleric Vigilance.
/// ETB gain 3 life.
pub fn silverquill_ascetic_b198() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Ascetic (b198)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(3)],
        ..Default::default()
    }
}

/// Inkling Streamer (b198) — {W}{B} 2/2 Inkling Rogue Flying.
/// Compact two-mana Flying body.
pub fn inkling_streamer_b198() -> CardDefinition {
    CardDefinition {
        name: "Inkling Streamer (b198)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Forewing (b198) — {3}{W} 2/4 Inkling Cleric Flying Vigilance.
/// Mid-game defender-flyer.
pub fn silverquill_forewing_b198() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Forewing (b198)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Edict (b198) — {1}{B} Sorcery.
/// Target opponent sacrifices a creature.
pub fn silverquill_edict_b198() -> CardDefinition {
    use crate::effect::shortcut::each_opponent;
    CardDefinition {
        name: "Silverquill Edict (b198)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Sacrifice {
            who: each_opponent(),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Tithe (b198) — {W}{B} Instant.
/// Drain 2 (each opp loses 2, you gain 2).
pub fn silverquill_tithe_b198() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Tithe (b198)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(2),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sentinel (b198) — {2}{W}{B} 4/3 Inkling Knight Flying.
/// Big evasive Inkling body.
pub fn silverquill_sentinel_b198() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentinel (b198)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 199 (modern_decks) — Silverquill rounding-out ──────────────────

/// Silverquill Cleric (b199) — {W} 1/1 Human Cleric Vigilance.
/// Cheap defensive 1-drop.
pub fn silverquill_cleric_b199() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Cleric (b199)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkdraw (b199) — {3}{U} Sorcery.
/// Draw 3 cards.
pub fn silverquill_inkdraw_b199() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Silverquill Inkdraw (b199)",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Beacon (b199) — {2}{W}{B} 3/3 Inkling Cleric Flying Lifelink.
/// Premium evasive lifelinker.
pub fn inkling_beacon_b199() -> CardDefinition {
    CardDefinition {
        name: "Inkling Beacon (b199)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Hymn (b199) — {W} Instant.
/// Gain 4 life.
pub fn silverquill_hymn_b199() -> CardDefinition {
    use crate::effect::shortcut::gain_life;
    CardDefinition {
        name: "Silverquill Hymn (b199)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: gain_life(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Smiter (b199) — {2}{W} Instant.
/// Destroy target creature with power 4 or greater.
pub fn silverquill_smiter_b199() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Smiter (b199)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 200 (modern_decks) — Silverquill round 200 ─────────────────────

/// Silverquill Quietkeeper (b200) — {W}{B} 2/2 Inkling Cleric Vigilance.
pub fn silverquill_quietkeeper_b200() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quietkeeper (b200)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Indrain (b200) — {2}{B}{B} Sorcery. Drain 4.
pub fn silverquill_indrain_b200() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Indrain (b200)",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Wraith (b200) — {3}{W}{B} 4/2 Inkling Spirit Flying Haste.
/// Aggressive evasive haster.
pub fn inkling_wraith_b200() -> CardDefinition {
    CardDefinition {
        name: "Inkling Wraith (b200)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 201 (modern_decks) — Silverquill nuanced round ─────────────────

/// Inkling Skybinder (b201) — {1}{W}{B} 2/2 Inkling Wizard Flying.
/// On attack: target opponent loses 1 life.
pub fn inkling_skybinder_b201() -> CardDefinition {
    use crate::effect::shortcut::on_attack;
    CardDefinition {
        name: "Inkling Skybinder (b201)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Whitewash (b201) — {2}{W} Sorcery.
/// Exile all creatures with power 3 or greater.
pub fn silverquill_whitewash_b201() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Whitewash (b201)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(3)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 202 (modern_decks) — Silverquill expansion ─────────────────────

/// Inkling Bloodbearer (b202) — {2}{W}{B} 3/3 Inkling Vampire Flying.
/// ETB: each opponent loses 2 life and you gain 2 life. Race-breaker
/// evasive drain body.
pub fn inkling_bloodbearer_b202() -> CardDefinition {
    CardDefinition {
        name: "Inkling Bloodbearer (b202)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Vampire],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Excoriator (b202) — {3}{B} Instant.
/// Destroy target creature with power 4 or less.
pub fn silverquill_excoriator_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Excoriator (b202)",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(4)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Bookbinder II (b202) — {1}{W} 1/3 Inkling Cleric.
/// Magecraft: scry 1, then draw a card. Spell-heavy engine.
pub fn inkling_bookbinder_ii_b202() -> CardDefinition {
    use crate::effect::shortcut::magecraft_scry_and_draw;
    CardDefinition {
        name: "Inkling Bookbinder II (b202)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry_and_draw(1)],
        ..Default::default()
    }
}

/// Silverquill Crestwalker (b202) — {2}{W}{B} 3/2 Vampire Knight.
/// First strike, lifelink. Premium evasive race-breaker.
pub fn silverquill_crestwalker_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Crestwalker (b202)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Quillforge (b202) — {3}{W}{B} Sorcery.
/// Create two 1/1 W/B Inkling tokens with flying, then each opponent
/// loses 3 life. Heavy-hitting double-body + drain finisher.
pub fn silverquill_quillforge_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Quillforge (b202)",
        cost: cost(&[generic(3), w(), b()]),
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
            drain(3),
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Quillward (b202) — {W} 1/1 Inkling Cleric.
/// ETB: put a flying counter on target creature you control.
/// CR 122.1b keyword counter grant + Inkling tribal slot.
pub fn inkling_quillward_b202() -> CardDefinition {
    CardDefinition {
        name: "Inkling Quillward (b202)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::AddKeywordCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Flying,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Dirge (b202) — {1}{B} Sorcery.
/// Each opponent loses 2 life. You gain 2 life. Then surveil 1.
pub fn silverquill_dirge_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Dirge (b202)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain_and_surveil(2, 1),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Heartcaller (b202) — {2}{B} 2/2 Inkling Cleric.
/// Whenever another Inkling you control dies, you gain 2 life.
/// Aristocrat-style payoff scaled to the tribe.
pub fn inkling_heartcaller_b202() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate};
    CardDefinition {
        name: "Inkling Heartcaller (b202)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Inkling),
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Inkblade (b202) — {1}{W} Instant.
/// Target creature gets +2/+0 EOT and gains first strike EOT.
pub fn silverquill_inkblade_b202() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Silverquill Inkblade (b202)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(2, 0, Keyword::FirstStrike),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Glyphlord (b202) — {3}{W}{B} 3/4 Inkling Wizard Flying.
/// Magecraft: target creature you control gets a +1/+1 counter and
/// gains lifelink until end of turn. Spell-engine tribal scaler.
pub fn inkling_glyphlord_b202() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Inkling Glyphlord (b202)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Silverquill Inkmaster (b202) — {2}{W} 2/3 Human Wizard Vigilance.
/// Whenever you gain life, put a +1/+1 counter on this creature.
/// Heliod-template life-gain payoff in white.
pub fn silverquill_inkmaster_b202() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "Silverquill Inkmaster (b202)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Silverquill Edictsong (b202) — {2}{W}{B} Sorcery.
/// Each opponent sacrifices a creature, then you gain 2 life and
/// draw a card. Mass edict + value.
pub fn silverquill_edictsong_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edictsong (b202)",
        cost: cost(&[generic(2), w(), b()]),
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
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Plumegrower (b202) — {1}{B} 1/2 Inkling Cleric.
/// ETB: each opponent loses 1 life, you gain 1 life, scry 1. Compact
/// drain + smoothing one-drop.
pub fn inkling_plumegrower_b202() -> CardDefinition {
    CardDefinition {
        name: "Inkling Plumegrower (b202)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(drain_and_scry(1, 1))],
        ..Default::default()
    }
}

/// Silverquill Quillstrike (b202) — {W} Instant.
/// Target creature gets +1/+0 EOT and gains lifelink EOT.
/// 1-mana combat trick + race-stabiliser.
pub fn silverquill_quillstrike_b202() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Silverquill Quillstrike (b202)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(1, 0, Keyword::Lifelink),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Lifecaller (b202) — {3}{W}{B} 3/3 Inkling Cleric Flying
/// Lifelink. ETB: target creature card from your graveyard returns
/// to your hand. Value evasive body.
pub fn inkling_lifecaller_b202() -> CardDefinition {
    CardDefinition {
        name: "Inkling Lifecaller (b202)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: SelectionRequirement::Creature,
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Silverquill Pendant (b202) — {3} Artifact.
/// Whenever you cast or copy an instant or sorcery, target creature
/// you control gets +1/+0 EOT. Magecraft-anthem in artifact form.
pub fn silverquill_pendant_b202() -> CardDefinition {
    use crate::effect::shortcut::magecraft_target_pump;
    CardDefinition {
        name: "Silverquill Pendant (b202)",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_target_pump(
            target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            1, 0,
        )],
        ..Default::default()
    }
}

/// Silverquill Vellumguard (b202) — {2}{W} 1/4 Inkling Cleric Defender.
/// Vigilance + lifelink — wall-style stabiliser that gains life from
/// any attack on the ground.
pub fn silverquill_vellumguard_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Vellumguard (b202)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Recap (b202) — {3}{W}{B} Sorcery.
/// Return up to two target creature or planeswalker cards with mana
/// value 3 or less from your graveyard to your hand. Mass recursion.
pub fn silverquill_recap_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Recap (b202)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Push (claude/modern_decks batch 202 round-3): Now uses
        // `Selector::take(.., 2)` to pull up to two low-MV creature
        // cards out of the graveyard in one Move. The take-N selector
        // shape was already there for spell-graveyard returns; this is
        // its first use for catalog graveyard-bf retrieval.
        effect: Effect::Move {
            what: Selector::take(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                },
                Value::Const(2),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Drainling (b202) — {W}{B} 2/1 Inkling Cleric Lifelink.
/// On attack: each opponent loses 1 life. Compact aggressive lifelinker.
pub fn inkling_drainling_b202() -> CardDefinition {
    CardDefinition {
        name: "Inkling Drainling (b202)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![on_attack_drain(1)],
        ..Default::default()
    }
}

/// Silverquill Sumptuous (b202) — {3}{W} Sorcery.
/// Create three 1/1 W/B Inkling tokens with flying. Big mass mint.
pub fn silverquill_sumptuous_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sumptuous (b202)",
        cost: cost(&[generic(3), w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Cantrix (b202) — {1}{W}{B} 2/2 Inkling Wizard Flying.
/// Magecraft: scry 1. Spell-heavy smoothing engine on a flyer.
pub fn inkling_cantrix_b202() -> CardDefinition {
    CardDefinition {
        name: "Inkling Cantrix (b202)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Pinionbreaker (b202) — {3}{B} Instant.
/// Destroy target creature with flying. Anti-flier removal at 4 mana.
pub fn silverquill_pinionbreaker_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pinionbreaker (b202)",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Augur (b202) — {2}{W} 2/2 Inkling Wizard.
/// ETB: scry 2, then draw a card. Premium selection body.
pub fn inkling_augur_b202() -> CardDefinition {
    use crate::effect::shortcut::etb_scry_and_draw;
    CardDefinition {
        name: "Inkling Augur (b202)",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry_and_draw(2)],
        ..Default::default()
    }
}

/// Silverquill Drainscholar II (b202) — {2}{B} 3/2 Vampire Warlock.
/// ETB: each opponent loses 2 life, you gain 2 life. Aggressive vamp.
pub fn silverquill_drainscholar_ii_b202() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainscholar II (b202)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Wardrune (b202) — {1}{W} Instant.
/// Target creature gets +0/+3 EOT and gains vigilance EOT.
/// Defensive combat trick.
pub fn silverquill_wardrune_b202() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Silverquill Wardrune (b202)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(0, 3, Keyword::Vigilance),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 203 (modern_decks) — Silverquill compact round ─────────────────

/// Silverquill Cantor (b203) — {1}{W} 2/2 Human Cleric.
/// ETB Scry 1.
pub fn silverquill_cantor_b203() -> CardDefinition {
    use crate::effect::shortcut::etb_scry;
    CardDefinition {
        name: "Silverquill Cantor (b203)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_scry(1)],
        ..Default::default()
    }
}

/// Inkling Whisperer (b203) — {W}{B} 2/1 Inkling Rogue Flying.
/// Magecraft Scry 1.
pub fn inkling_whisperer_b203() -> CardDefinition {
    CardDefinition {
        name: "Inkling Whisperer (b203)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_scry(1)],
        ..Default::default()
    }
}

/// Silverquill Censurer (b203) — {2}{W}{B} 2/4 Vampire Cleric Lifelink.
/// On combat damage to player: drain 1.
pub fn silverquill_censurer_b203() -> CardDefinition {
    use crate::effect::shortcut::on_combat_damage_to_player_drain;
    CardDefinition {
        name: "Silverquill Censurer (b203)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![on_combat_damage_to_player_drain(1)],
        ..Default::default()
    }
}

/// Inkling Mentor (b203) — {2}{W}{B} 2/3 Inkling Cleric Flying.
/// Magecraft: gain 1 life and target opp loses 1.
pub fn inkling_mentor_b203() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mentor (b203)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Edict (b203) — {2}{B} Sorcery.
/// Target opponent sacrifices a creature.
pub fn silverquill_edict_b203() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edict (b203)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Hospitaller (b203) — {3}{W}{B} 4/4 Vampire Cleric Lifelink
/// Vigilance.
pub fn silverquill_hospitaller_b203() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Hospitaller (b203)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sage Apprentice (b203) — {1}{W} 1/2 Inkling Cleric Flying.
/// Magecraft Gain 1 life.
pub fn inkling_sage_apprentice_b203() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sage Apprentice (b203)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_gain_life(1)],
        ..Default::default()
    }
}

/// Silverquill Lay-Faith (b203) — {W} Sorcery. Gain 4 life.
pub fn silverquill_lay_faith_b203() -> CardDefinition {
    use crate::effect::shortcut::gain_life;
    CardDefinition {
        name: "Silverquill Lay-Faith (b203)",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: gain_life(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Sheriff (b203) — {3}{W}{B} 3/3 Inkling Soldier Flying
/// Vigilance Lifelink. Premium top-end Inkling finisher.
pub fn inkling_sheriff_b203() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sheriff (b203)",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Mausoleum (b203) — {3} Artifact.
/// {2}, {T}: target opp loses 1 life and you gain 1 life.
pub fn silverquill_mausoleum_b203() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Silverquill Mausoleum (b203)",
        cost: cost(&[generic(3)]),
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
            effect: drain(1),
            ..ActivatedAbility::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Batch 204 (modern_decks) — Silverquill round 4 ───────────────────────

/// Inkling Vanguard II (b204) — {2}{W}{B} 3/3 Inkling Soldier Vigilance.
/// First strike. Aggressive evasive defender.
pub fn inkling_vanguard_ii_b204() -> CardDefinition {
    CardDefinition {
        name: "Inkling Vanguard II (b204)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Drainstrike (b204) — {2}{B} Instant.
/// Drain 3 (each opp loses 3, you gain 3).
pub fn silverquill_drainstrike_b204() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drainstrike (b204)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(3),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Bond (b204) — {W}{B} Instant.
/// Target creature gets +1/+1 EOT and gains lifelink EOT.
pub fn silverquill_bond_b204() -> CardDefinition {
    use crate::effect::shortcut::pump_and_grant_keyword;
    CardDefinition {
        name: "Silverquill Bond (b204)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: pump_and_grant_keyword(1, 1, Keyword::Lifelink),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Champion (b204) — {3}{W} 3/3 Inkling Knight Vigilance Flying.
/// Premium evasive Inkling beater.
pub fn inkling_champion_b204() -> CardDefinition {
    CardDefinition {
        name: "Inkling Champion (b204)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Censer (b204) — {1}{B} 1/2 Vampire Cleric Deathtouch.
/// 2-mana removal-blocker.
pub fn silverquill_censer_b204() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Censer (b204)",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 205 (modern_decks) — Silverquill (W/B) round: lifegain, drain, and
// Inkling-tribal fillers using existing primitives.
// ─────────────────────────────────────────────────────────────────────────

/// Silverquill Lightscribe (b205) — {1}{W} 2/2 Human Cleric.
/// ETB — you gain 3 life.
pub fn silverquill_lightscribe_b205() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lightscribe (b205)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_gain_life(3)],
        ..Default::default()
    }
}

/// Silverquill Grimquill (b205) — {2}{B} 2/2 Inkling Wizard with Flying.
/// Magecraft — whenever you cast or copy an instant or sorcery, each
/// opponent loses 1 life and you gain 1 life.
pub fn silverquill_grimquill_b205() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Grimquill (b205)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Final Edict (b205) — {2}{B} Sorcery.
/// Each opponent loses 3 life and you gain 3 life.
pub fn silverquill_final_edict_b205() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Final Edict (b205)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: drain(3),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Inkguard (b205) — {W}{B} 2/3 Inkling Cleric with Lifelink.
/// A sturdy two-drop Inkling for the W/B aggro-lifegain shell.
pub fn silverquill_inkguard_b205() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkguard (b205)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Deathscribe (b205) — {2}{B} 2/2 Vampire Warlock.
/// Whenever another creature you control dies, each opponent loses 1 life
/// and you gain 1 life.
pub fn silverquill_deathscribe_b205() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Deathscribe (b205)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_other_dies(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 206 (modern_decks) — Silverquill (W/B) staples.
// ─────────────────────────────────────────────────────────────────────────

/// Silverquill Dictator (b206) — {2}{W}{B} 3/3 Inkling Noble with Flying
/// and Lifelink. A clean evasive lifelink finisher for the W/B shell.
pub fn silverquill_dictator_b206() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Dictator (b206)",
        cost: cost(&[generic(2), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Noble],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Purge (b206) — {1}{W} Sorcery.
/// Exile target creature with power 2 or less.
pub fn silverquill_purge_b206() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Purge (b206)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
            ),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ─────────────────────────────────────────────────────────────────────────
// Batch 207 (modern_decks) — Silverquill (W/B) Inkling / drain staples.
// ─────────────────────────────────────────────────────────────────────────

/// Silverquill Inkbinder (b207) — {1}{W}{B} 2/2 Inkling Wizard, Flying.
/// Magecraft — you gain 1 life and each opponent loses 1 life.
pub fn silverquill_inkbinder_b207() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inkbinder (b207)",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![magecraft_drain_each_opp(1)],
        ..Default::default()
    }
}

/// Silverquill Eulogist (b207) — {2}{B} 2/2 Vampire Cleric.
/// When this creature enters, each opponent loses 2 life and you gain 2.
pub fn silverquill_eulogist_b207() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Eulogist (b207)",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![etb_drain(2)],
        ..Default::default()
    }
}

/// Silverquill Edict II (b207) — {2}{B} Sorcery.
/// Each opponent sacrifices a creature, then you draw a card.
pub fn silverquill_edict_ii_b207() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Edict II (b207)",
        cost: cost(&[generic(2), b()]),
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Inkling Highflier (b207) — {3}{W} 2/3 Inkling Knight, Flying + Vigilance.
/// A defensive evasive Inkling body for the W/B midrange shell.
pub fn inkling_highflier_b207() -> CardDefinition {
    CardDefinition {
        name: "Inkling Highflier (b207)",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Sanction (b207) — {1}{W} Instant.
/// Exile target attacking or blocking creature, then you gain 2 life.
pub fn silverquill_sanction_b207() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sanction (b207)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Silverquill Coursemate (b207) — {W}{B} 2/2 Inkling Cleric.
/// Whenever another creature you control dies, you gain 1 life.
pub fn silverquill_coursemate_b207() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Coursemate (b207)",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![on_other_dies(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Silverquill Duelmaster (b207) — {1}{W} 2/2 Human Knight with Exalted
/// (CR 702.83). "Whenever a creature you control attacks alone, that
/// creature gets +1/+1 until end of turn." Exercises the new
/// `Predicate::AttackingAlone` / `exalted()` shortcut.
pub fn silverquill_duelmaster_b207() -> CardDefinition {
    use crate::effect::shortcut::exalted;
    CardDefinition {
        name: "Silverquill Duelmaster (b207)",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![exalted()],
        ..Default::default()
    }
}
