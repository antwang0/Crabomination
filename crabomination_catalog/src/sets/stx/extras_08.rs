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

// ── Silverquill Lecturer (synthesised STX Silverquill) ──────────────────────

/// Silverquill Lecturer — {1}{W}{B}, 2/2 Human Cleric.
///
/// Printed Oracle (synthesised): "Lifelink / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, target creature gets
/// +1/+1 until end of turn."
///
/// A Silverquill lifelink Magecraft payoff — the body stays a small
/// 2/2 lifelink, but each spell pumps a chosen creature (often the
/// Lecturer itself for life cascades). Pairs with Inkling tokens and
/// Tenured Inkcaster's anthem for explosive lifegain swings.
pub fn silverquill_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Lecturer",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Conjurer (synthesised STX Quandrix) ────────────────────────────

/// Quandrix Conjurer — {2}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token, then put X +1/+1 counters on it, where X is the
/// number of creatures you control."
///
/// A Fractalize-flavor mass-counter spell — the Fractal arrives 0/0,
/// counters apply immediately via the same `enters_with_counters`
/// pipeline. With 3 creatures already on the battlefield, this lands
/// a 3/3 Fractal; with 5+ it's game-ending. Pairs with Tanazir's
/// counter doubling for runaway boards.
pub fn quandrix_conjurer() -> CardDefinition {
    let fractal_token = crate::catalog::sets::sos::fractal_token();
    CardDefinition {
        name: "Quandrix Conjurer",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Token + apply N counters where N = creatures you control.
        // We mint the 0/0 first, then use the `Selector::This` returned
        // by `Effect::CreateToken` indirectly. The simplest wire here
        // is `Seq(CreateToken, AddCounter on each newly-minted token)`
        // — the auto-target picks the freshest Fractal.
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token,
            },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Concoction (synthesised STX Witherbloom) ────────────────────

/// Witherbloom Concoction — {1}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Target creature gets -2/-2 until end
/// of turn. You gain 2 life and draw a card."
///
/// A Witherbloom 3-mana drain-and-cantrip — removal that swings the
/// life total +4 (kill a 2-toughness threat, gain 2 life) and replaces
/// itself with a draw. Pairs with Honor Troll's lifegain trigger and
/// the Witherbloom drain payoffs for tight tempo plays.
pub fn witherbloom_concoction() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Concoction",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Sparkmage (synthesised STX Prismari) ───────────────────────────

/// Prismari Sparkmage — {1}{U}{R}, 2/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, deal 2
/// damage to target creature or planeswalker. / Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, scry 1."
///
/// A Prismari ETB ping + Magecraft filtering creature. The ETB removes
/// a 2-toughness blocker; subsequent Magecraft scries smooth draws
/// across the spell chain. Pairs with Spell Satchel-style recursion
/// and Twinscroll Shaman copy-velocity.
pub fn prismari_sparkmage() -> CardDefinition {
    CardDefinition {
        name: "Prismari Sparkmage",
        cost: cost(&[generic(1), u(), r()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Ambassador (synthesised STX Silverquill) ────────────────────

/// Silverquill Ambassador — {2}{W}{B}, 3/3 Inkling Cleric.
///
/// Printed Oracle (synthesised): "Flying, lifelink / When this
/// creature enters, create a 1/1 white and black Inkling creature
/// token with flying."
///
/// A 5-mana 3/3 lifelink flier that mints a 1/1 Inkling flier on ETB —
/// effectively 4 power and 4 toughness in the air for 5 mana, plus
/// the Inkling tribal synergy with Tenured Inkcaster's +2/+2 anthem.
pub fn silverquill_ambassador() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Silverquill Ambassador",
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Battlemage (synthesised STX Lorehold) ──────────────────────────

/// Lorehold Battlemage — {2}{R}{W}, 3/3 Human Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, deal 1
/// damage to each opponent and gain 1 life. / {1}{R}{W}, {T}: Exile
/// target card from a graveyard. This creature deals 2 damage to any
/// target."
///
/// ETB drain (1 to each opp + gain 1) + tap-activate gy-exile + 2
/// damage to creature/player/PW. The activation reuses the
/// `exile_other_filter` field for "exile target card from a graveyard"
/// as cost-and-effect (the Move-to-exile rides on the activation's
/// effect, not as a payment).
pub fn lorehold_battlemage() -> CardDefinition {
    use crate::mana::cost as mc;
    CardDefinition {
        name: "Lorehold Battlemage",
        cost: cost(&[generic(2), r(), w()]),
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
        activated_abilities: vec![ActivatedAbility {
            mana_cost: mc(&[generic(1), r(), w()]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            condition: None,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
                Effect::DealDamage {
                    to: Selector::TargetFiltered {
                        slot: 1,
                        filter: SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    },
                    amount: Value::Const(2),
                },
            ]),
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Plaguemage (synthesised STX Witherbloom) ────────────────────

/// Witherbloom Plaguemage — {2}{B}{G}, 2/3 Human Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, each
/// opponent loses 2 life and you gain 2 life. / {1}{B}{G}, {T},
/// Sacrifice a creature: Each opponent loses 2 life and you gain 2
/// life."
///
/// ETB drain + repeatable tap-sacrifice drain — a Witherbloom drain
/// engine that scales with creature recursion (Pest tokens from Pest
/// Summoning / Tend the Pests / Eyetwitch). At 3 sacs per turn this is
/// a 6-life swing per upkeep.
pub fn witherbloom_plaguemage() -> CardDefinition {
    use crate::mana::cost as mc;
    CardDefinition {
        name: "Witherbloom Plaguemage",
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
        activated_abilities: vec![ActivatedAbility {
            mana_cost: mc(&[generic(1), b(), g()]),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            // "Sacrifice a creature" is now a proper pre-resolution
            // activation cost via sac_other_filter (rejects when there's
            // no creature to sacrifice), rather than a body Effect.
            exile_other_filter: None,
            condition: None,
            sorcery_speed: false,
            once_per_turn: false,
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
                    self_counter_cost_reduction: None,
                    sac_other_filter: Some((
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                        1,
                    )),
                    tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Skywriter (synthesised STX Silverquill) ─────────────────────

/// Silverquill Skywriter — {2}{W}{B}, 2/3 Inkling Wizard.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// draw a card. / Whenever you draw a card, each opponent loses 1
/// life and you gain 1 life."
///
/// A 2/3 flier with ETB cantrip + on-draw drain — every subsequent
/// draw (Pop Quiz, Curate, Quick Study, Triskaidekaphile's no-max-
/// hand-size payoff) ticks 1 life per opponent. Pairs with blue draw
/// engines for grindy lifegain control wins.
pub fn silverquill_skywriter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Skywriter",
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Tutor 2 (synthesised STX Quandrix) ─────────────────────────────

/// Quandrix Curriculum — {2}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Look at the top six cards of your
/// library. You may reveal a creature card and put it into your hand
/// and you may reveal a land card and put it into your hand. Put the
/// rest on the bottom of your library in a random order."
///
/// A Quandrix dig-and-tutor that's strictly better than Adventurous
/// Impulse — checks 6 cards (vs 3), grabs one creature AND one land
/// (not OR). Approximated as `Seq(RevealUntilFind Creature → Hand,
/// RevealUntilFind Land → Hand)`.
pub fn quandrix_curriculum() -> CardDefinition {
    use crate::effect::RevealMissDest;
    CardDefinition {
        name: "Quandrix Curriculum",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(6),
                miss_dest: RevealMissDest::BottomRandom,
                life_per_revealed: 0,
            },
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Land,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(6),
                miss_dest: RevealMissDest::BottomRandom,
                life_per_revealed: 0,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Researcher (synthesised STX Lorehold) ──────────────────────────

/// Lorehold Researcher — {R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "First strike / When this creature
/// dies, return target instant or sorcery card from your graveyard to
/// your hand."
///
/// A 2-mana 2/2 first-striker that recovers an IS spell on death — the
/// "trade-up" Lorehold body. Pairs with Lorehold Excavation's gy-
/// to-Spirit conversion for repeatable IS recursion.
pub fn lorehold_researcher() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Lorehold Researcher",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Magicraft (synthesised STX Prismari) ───────────────────────────

/// Prismari Magicraft — {3}{U}{R} Sorcery.
///
/// Printed Oracle (synthesised): "Copy target instant or sorcery spell
/// you control. You may choose new targets for the copy. Draw a card."
///
/// A Prismari double-spell-and-cantrip — a stronger Galvanic Iteration
/// at 2 extra mana plus a draw. The copy targeting follows the engine's
/// existing CopySpell path (copies inherit original targets); the
/// cantrip half guarantees raw card velocity.
pub fn prismari_magicraft() -> CardDefinition {
    CardDefinition {
        name: "Prismari Magicraft",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CopySpell {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                },
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Botanist (synthesised STX Witherbloom) ──────────────────────

/// Witherbloom Botanist — {1}{B}{G}, 2/2 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.' / {2}{B}{G}, Sacrifice this creature: Each
/// opponent loses 3 life and you gain 3 life."
///
/// A 3-mana 2/2 with Pest ETB + repeatable suicide drain. Pairs with
/// Tend the Pests / Pest Summoning for sacrifice fodder, and with the
/// Pest token's own die-to-gain-1 rider for cumulative life swings.
pub fn witherbloom_botanist() -> CardDefinition {
    use crate::mana::cost as mc;
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Witherbloom Botanist",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: mc(&[generic(2), b(), g()]),
            tap_cost: false,
            sac_cost: true,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            condition: None,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pest,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Drafter (synthesised STX Silverquill) ───────────────────────

/// Silverquill Drafter — {1}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Choose one — / • Target opponent
/// discards a card at random. / • Put a +1/+1 counter on each Inkling
/// you control. / • Each opponent loses 2 life and you gain 2 life."
///
/// A flexible Silverquill 3-mode utility spell. Pairs with the Inkling
/// tribal core (Defend the Campus, Inkling Summoning, Inkling Squad,
/// Tenured Inkcaster) for the +1/+1 mode payoff.
pub fn silverquill_drafter() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Drafter",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
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
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Schematist (synthesised STX Quandrix) ──────────────────────────

/// Quandrix Schematist — {G}{U}, 1/2 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2. /
/// {2}{G}{U}: Put a +1/+1 counter on target creature you control."
///
/// A 2-mana 1/2 with ETB filtering + repeatable counter activation —
/// the small body upgrades a single creature into a beater over the
/// course of the game, scaling with Tanazir Quandrix's doubling and
/// Symmathematics's counter-magic.
pub fn quandrix_schematist() -> CardDefinition {
    use crate::mana::cost as mc;
    CardDefinition {
        name: "Quandrix Schematist",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: mc(&[generic(2), g(), u()]),
            tap_cost: false,
            sac_cost: false,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            condition: None,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Resurrectionist (synthesised STX Lorehold) ─────────────────────

/// Lorehold Resurrectionist — {3}{R}{W}, 3/3 Spirit Cleric.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// return target creature card with mana value 3 or less from your
/// graveyard to the battlefield. It gains haste until end of turn."
///
/// A 5-mana 3/3 flier that reanimates a low-cost creature with haste —
/// the value version of Pillardrop Rescuer. Pairs with Lorehold's gy-
/// dump engines (Sparring Regimen, Hardened Academic) for tempo plays.
pub fn lorehold_resurrectionist() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Lorehold Resurrectionist",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    ),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Tinkerer (synthesised STX Prismari) ────────────────────────────

/// Prismari Tinkerer — {U}{R}, 2/1 Human Artificer.
///
/// Printed Oracle (synthesised): "Prowess (Whenever you cast a
/// noncreature spell, this creature gets +1/+1 until end of turn.) /
/// When this creature dies, create a Treasure token."
///
/// A 2-mana 2/1 Prowess body that leaves a Treasure on death — value
/// regardless of whether it trades or chump-blocks. Pairs with the
/// spell-velocity Prismari engines.
pub fn prismari_tinkerer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Tinkerer",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Forecaster (synthesised STX Quandrix, batch 9) ─────────────────

/// Quandrix Forecaster — {1}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Look at the top three cards of your
/// library. Put one into your hand and the rest into your graveyard.
/// Draw a card."
///
/// A Quandrix dig-and-cantrip — checks 3 cards, takes one, mills the
/// rest, then draws. Net: +1 card, +mill 2 for gy synergies. Pairs
/// with Lorehold gy recursion engines.
pub fn quandrix_forecaster() -> CardDefinition {
    use crate::effect::RevealMissDest;
    CardDefinition {
        name: "Quandrix Forecaster",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(3),
                miss_dest: RevealMissDest::Graveyard,
                life_per_revealed: 0,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Bookbinder (synthesised STX Silverquill, batch 9) ───────────

/// Silverquill Bookbinder — {2}{W}{B}, 2/4 Inkling Cleric.
///
/// Printed Oracle (synthesised): "Flying / When this creature enters,
/// you gain 3 life and each opponent loses 3 life."
///
/// A 4-mana 2/4 lifelink-flavor flier with built-in drain.
pub fn silverquill_bookbinder() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Bookbinder",
        cost: cost(&[generic(2), w(), b()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Crusader (synthesised STX Lorehold, batch 9) ───────────────────

/// Lorehold Crusader — {2}{R}{W}, 2/2 Spirit Knight.
///
/// Printed Oracle (synthesised): "First strike, lifelink / Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, this
/// creature gets +1/+1 until end of turn."
pub fn lorehold_crusader_knight() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Crusader Knight",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Conjurer (synthesised STX Witherbloom, batch 9) ─────────────

/// Witherbloom Conjurer — {3}{B}{G}, 3/4 Plant Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create
/// two 1/1 black and green Pest creature tokens with 'When this
/// creature dies, you gain 1 life.' / Whenever you gain life, put a
/// +1/+1 counter on this creature."
pub fn witherbloom_conjurer() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Witherbloom Conjurer",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: pest,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Conjurer (synthesised STX Prismari, batch 9) ───────────────────

/// Prismari Conjurer — {2}{U}{R}, 2/3 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 1 damage to
/// any target and you draw a card. Then discard a card."
pub fn prismari_conjurer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Conjurer",
        cost: cost(&[generic(2), u(), r()]),
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Calligrapher (synthesised STX Quandrix, batch 9) ───────────────

/// Quandrix Calligrapher — {3}{G}{U}, 4/4 Fractal Wizard.
///
/// Printed Oracle (synthesised): "This creature enters with three
/// +1/+1 counters on it. / {2}{G}{U}: Double the number of +1/+1
/// counters on this creature."
pub fn quandrix_calligrapher() -> CardDefinition {
    use crate::mana::cost as mc;
    CardDefinition {
        name: "Quandrix Calligrapher",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: mc(&[generic(2), g(), u()]),
            sac_cost: false,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            condition: None,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
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
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Penmaster (synthesised STX Silverquill, batch 9) ────────────

/// Silverquill Penmaster — {1}{W}{B} Instant.
///
/// Printed Oracle (synthesised): "Choose one — / • Destroy target
/// creature with power 4 or greater. / • Exile target creature with
/// power 2 or less."
pub fn silverquill_penmaster() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Penmaster",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
                ),
                to: ZoneDest::Exile,
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                ),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Smith (synthesised STX Lorehold, batch 9) ──────────────────────

/// Lorehold Smith — {1}{R}{W}, 2/3 Dwarf Artificer.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// Treasure token. / {1}, Sacrifice a Treasure: This creature gets
/// +1/+1 until end of turn."
pub fn lorehold_treasure_smith() -> CardDefinition {
    use crate::mana::cost as mc;
    CardDefinition {
        name: "Lorehold Treasure Smith",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: mc(&[generic(1)]),
            sac_cost: false,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            condition: None,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::HasArtifactSubtype(
                        crate::card::ArtifactSubtype::Treasure,
                    ),
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            ]),
                    self_counter_cost_reduction: None, sac_other_filter: None,
                    tap_other_filter: None, from_hand: false,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Tutor (synthesised STX Witherbloom, batch 9) ────────────────

/// Witherbloom Tutor — {1}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Search your library for a creature
/// card with mana value 3 or less, reveal it, put it into your hand,
/// then shuffle. You lose 2 life."
pub fn witherbloom_tutor() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Tutor",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::LoseLife {
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Cartographer (synthesised STX Prismari, batch 9) ───────────────

/// Prismari Cartographer — {U}{R} Instant.
///
/// Printed Oracle (synthesised): "Scry 2, then draw a card."
pub fn prismari_cartographer() -> CardDefinition {
    CardDefinition {
        name: "Prismari Cartographer",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Geologist (synthesised STX Quandrix, batch 9) ──────────────────

/// Quandrix Geologist — {G}{U}, 1/3 Elf Druid.
///
/// Printed Oracle (synthesised): "{T}: Add {G} or {U}. / {T}, Discard
/// a card: Draw a card."
pub fn quandrix_geologist() -> CardDefinition {
    use super::super::tap_add;
    use crate::mana::cost as mc;
    CardDefinition {
        name: "Quandrix Geologist",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            tap_add(Color::Green),
            tap_add(Color::Blue),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: mc(&[]),
                sac_cost: false,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
                condition: None,
                sorcery_speed: false,
                once_per_turn: false,
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
                ]),
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
        adventure: None,
        plot_cost: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Batch 10 (push: modern_decks) — 20+ new synthesised STX cards across all
// five colleges. Each card uses existing engine primitives and ships with
// at least one functionality test in `crate::tests::stx`.
// ═══════════════════════════════════════════════════════════════════════════

// ── Silverquill Chastiser (batch 10) ────────────────────────────────────────

/// Silverquill Chastiser — {1}{W}{B}, 3/2 Inkling Cleric, Flying.
///
/// Printed Oracle (synthesised): "Flying / Whenever another Inkling
/// you control enters, target opponent loses 1 life and you gain 1
/// life."
///
/// Inkling-tribal payoff that turns every token-mint (Inkling
/// Summoning, Defend the Campus, Silverquill Ambassador ETB) into a
/// drain trigger. Pairs with Tenured Inkcaster's +2/+2 anthem for a
/// Silverquill tribal shell.
pub fn silverquill_chastiser() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Chastiser",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Inkling),
                }),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Pestmaster (batch 10) ───────────────────────────────────────

/// Witherbloom Pestmaster — {2}{B}{G}, 2/3 Plant Warlock.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this token
/// dies, you gain 1 life.' / Whenever another Pest you control dies,
/// put a +1/+1 counter on this creature."
///
/// Pest-tribal payoff that snowballs counters off the engine's
/// Pest-die-to-gain-1 loop (Pest Summoning, Tend the Pests, Hunt for
/// Specimens). The body's 2/3 frame grows quickly with two or more
/// pest deaths a turn.
pub fn witherbloom_pestmaster() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Witherbloom Pestmaster",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
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
                    count: Value::Const(1),
                    definition: pest,
                },
            },
            TriggeredAbility {
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Chronicler (batch 10) ──────────────────────────────────────────

/// Lorehold Chronicler — {2}{R}{W}, 3/3 Spirit Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, return
/// target instant or sorcery card from your graveyard to your hand.
/// / Whenever this creature attacks, exile target card from a
/// graveyard."
///
/// Lorehold's classic ETB-recursion + on-attack graveyard hate
/// combo. ETB grabs an IS spell from graveyard; attack triggers
/// strip the opponent's gy engines (Lord of Extinction, Bloodghast,
/// Past in Flames, etc.).
pub fn lorehold_chronicler() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Chronicler",
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
        triggered_abilities: vec![
            TriggeredAbility {
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
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::EachOpponent,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Any,
                    }),
                    to: ZoneDest::Exile,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Pyromentor (batch 10) ──────────────────────────────────────────

/// Prismari Pyromentor — {3}{U}{R}, 3/4 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, this creature deals 2 damage to
/// target opponent."
///
/// A bigger Drannith Stinger sibling at the rare slot — every spell
/// in a Prismari deck becomes a 2-burn ping, scaling fast with
/// Magecraft + CopySpell triggers (Galvanic Iteration, Twinscroll
/// Shaman).
pub fn prismari_pyromentor() -> CardDefinition {
    CardDefinition {
        name: "Prismari Pyromentor",
        cost: cost(&[generic(3), u(), r()]),
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Equation (batch 10) ────────────────────────────────────────────

/// Quandrix Equation — {2}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is twice the
/// number of cards in your hand."
///
/// Hand-size scaling Fractal mint, exploiting Quandrix's draw-heavy
/// shells. Caster with 4 cards in hand → 8/8 Fractal for 4 mana.
/// Pairs with Manifestation Sage (X = HandSize) and Body of Research.
pub fn quandrix_equation() -> CardDefinition {
    let fractal = super::super::sos::fractal_token();
    CardDefinition {
        name: "Quandrix Equation",
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
                count: Value::Const(1),
                definition: fractal,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Inquisitor's Mark (batch 10) ────────────────────────────────

/// Silverquill Inquisitor's Mark — {1}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Target opponent reveals their hand
/// and discards a noncreature, nonland card of your choice. You gain
/// 2 life."
///
/// Targeted-discard + small lifegain — a Silverquill take on
/// Despise/Inquisition of Kozilek with a white life cushion. The
/// auto-decider picks the first non-creature non-land card from the
/// opp's hand.
pub fn silverquill_inquisitors_mark() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inquisitor's Mark",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Not(Box::new(
                    SelectionRequirement::Creature.or(SelectionRequirement::Land),
                )),
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Mire (batch 10) ─────────────────────────────────────────────

/// Witherbloom Mire — {2}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "Each opponent loses 3 life and you
/// gain 3 life. Surveil 2."
///
/// Witherbloom drain + graveyard-set-up. Pairs with `Effect::Surveil`
/// to bin reanimation targets while padding the life total.
pub fn witherbloom_mire() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Mire",
        cost: cost(&[generic(2), b(), g()]),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Memorial (batch 10) ────────────────────────────────────────────

/// Lorehold Memorial — {2}{R}{W} Enchantment.
///
/// Printed Oracle (synthesised): "When this enchantment enters,
/// return target creature card from your graveyard to your hand. /
/// At the beginning of your end step, if a creature died this turn,
/// you gain 1 life and create a 2/2 red and white Spirit creature
/// token."
///
/// A reanimator-flavored Lorehold enchantment: front-loaded gy →
/// hand recursion, then a per-turn Spirit-mint sustained by the
/// game's creature-death trigger.
pub fn lorehold_memorial() -> CardDefinition {
    use crate::card::Zone;
    let spirit = super::super::sos::spirit_token();
    CardDefinition {
        name: "Lorehold Memorial",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::CreaturesDiedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                }),
                effect: Effect::Seq(vec![
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: spirit,
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Ember-Trickster (batch 10) ─────────────────────────────────────

/// Prismari Ember-Trickster — {U}{R}, 1/3 Human Wizard, Prowess.
///
/// Printed Oracle (synthesised): "Prowess (Whenever you cast a
/// noncreature spell, this creature gets +1/+1 until end of turn.) /
/// When this creature enters, create a Treasure token."
///
/// 2-mana 1/3 prowess body that immediately replaces itself with a
/// Treasure — fixes a color while leaving a creature on board to
/// trigger off subsequent spells.
pub fn prismari_ember_trickster() -> CardDefinition {
    CardDefinition {
        name: "Prismari Ember-Trickster",
        cost: cost(&[u(), r()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            },
            crate::effect::shortcut::prowess(),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Aetherist (batch 10) ───────────────────────────────────────────

/// Quandrix Aetherist — {1}{G}{U}, 2/2 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, put a
/// +1/+1 counter on it for each card in your hand. / Whenever a
/// +1/+1 counter is put on this creature, you may draw a card."
///
/// Hand-size scaling ETB followed by a "draw on counter" engine —
/// hand → counter → draw → cast → counter creates loops with
/// proliferate / Hardened Scales-style payoffs.
pub fn quandrix_aetherist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Aetherist",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::HandSizeOf(PlayerRef::You),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::MayDo {
                    description: String::from("draw a card"),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Sentinel (batch 10) ─────────────────────────────────────────

/// Silverquill Sentinel — {W}{B}, 2/2 Inkling Knight, Flying.
///
/// Printed Oracle (synthesised): "Flying, lifelink / At the
/// beginning of combat on your turn, this creature gets +1/+0 until
/// end of turn."
///
/// A 2/2 flier that becomes a 3/2 every combat — converts steady
/// lifelink into both an attacker and a deterrent. Pairs with the
/// Inkling tribal engines.
pub fn silverquill_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sentinel",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Necrogale (batch 10) ────────────────────────────────────────

/// Witherbloom Necrogale — {3}{B}{G}, 4/4 Plant Zombie.
///
/// Printed Oracle (synthesised): "When this creature enters, return
/// target creature card with mana value 3 or less from your
/// graveyard to the battlefield. It gains haste until end of turn."
///
/// Witherbloom's mid-game reanimator finisher: 4/4 body + a
/// drop-and-swing reanimation. Pairs with the engine's `Predicate::
/// ManaValueAtMost(3)` cap on the gy filter.
pub fn witherbloom_necrogale() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Witherbloom Necrogale",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Zombie],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::GrantKeyword {
                    what: Selector::LastCreatedToken,
                    keyword: Keyword::Haste,
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Echo (batch 10) ────────────────────────────────────────────────

/// Lorehold Echo — {R}{W} Instant.
///
/// Printed Oracle (synthesised): "Target creature gets +2/+2 until
/// end of turn. If a creature died this turn, that creature also
/// gains first strike and lifelink until end of turn."
///
/// Lorehold's combat trick wrapped in a graveyard-conditional
/// upgrade. Pairs with token sacrifices to flip the rider on.
pub fn lorehold_echo() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Echo",
        cost: cost(&[r(), w()]),
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
                cond: Predicate::CreaturesDiedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Lifelink,
                        duration: Duration::EndOfTurn,
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Spellforger (batch 10) ─────────────────────────────────────────

/// Prismari Spellforger — {1}{U}{R}, 2/3 Human Artificer.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a
/// card, then discard a card. / Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, create a Treasure token."
///
/// Loot + Treasure on cast — every spell repairs your hand and
/// fixes mana for the next one. Pairs with Prismari Inventor (1/1
/// Treasure-on-cast) for the dedicated Treasure shell.
pub fn prismari_spellforger() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellforger",
        cost: cost(&[generic(1), u(), r()]),
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
            },
            crate::effect::shortcut::magecraft_treasure(),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Multiplier (batch 10) ──────────────────────────────────────────

/// Quandrix Multiplier — {2}{G}{U}, 3/3 Fractal Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, double
/// the number of +1/+1 counters on target creature you control."
///
/// Quandrix's signature "counter doubling" — same Tanazir Quandrix
/// shape but on a single targeted creature, at a 3/3 mid-curve
/// body. With Symmathematics (2/2 ETB with two counters): 2 → 4 → 8.
pub fn quandrix_multiplier() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Multiplier",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Scribefall (batch 10) ───────────────────────────────────────

/// Silverquill Scribefall — {3}{W}{B} Sorcery.
///
/// Printed Oracle (synthesised): "Create two 1/1 white and black
/// Inkling creature tokens with flying. Target opponent loses 2 life
/// and you gain 2 life."
///
/// Three-for-two threat play — two Inkling tokens to attack with
/// next turn, plus an immediate drain that beats the curve. Slotting
/// alongside Defend the Campus (3 Inklings for 5 mana) and Silverquill
/// Verse (modal pump+drain+Inkling).
pub fn silverquill_scribefall() -> CardDefinition {
    let inkling = super::super::sos::inkling_token();
    CardDefinition {
        name: "Silverquill Scribefall",
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
                definition: inkling,
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Wickering (batch 10) ────────────────────────────────────────

/// Witherbloom Wickering — {B}{G} Instant.
///
/// Printed Oracle (synthesised): "Sacrifice a creature: Target
/// creature gets -2/-2 until end of turn. If the sacrificed creature
/// had toughness 3 or greater, that creature gets -4/-4 until end of
/// turn instead."
///
/// Witherbloom sacrifice-driven removal that scales with the sacked
/// creature's printed toughness. The scaling branch (`-4/-4` vs.
/// `-2/-2`) is gated on `Predicate::ValueAtLeast(SacrificedToughness,
/// 3)` via the engine's existing `Effect::If` primitive — no `Value::
/// IfElse` is needed since the sub-effect-level branch suffices.
pub fn witherbloom_wickering() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Wickering",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::SacrificedToughness,
                    Value::Const(3),
                ),
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-4),
                    toughness: Value::Const(-4),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Historian (batch 10) ───────────────────────────────────────────

/// Lorehold Historian — {1}{R}{W}, 2/2 Spirit Cleric.
///
/// Printed Oracle (synthesised): "When this creature enters, you may
/// exile a card from your graveyard. If you do, this creature deals
/// 2 damage to any target."
///
/// Graveyard-exile-as-cost stapled to a 2-damage ping ETB.
/// Synergises with Lorehold's gy fuel — every exiled card both fires
/// the Bolt and feeds delirium / lieutenant-style payoffs.
pub fn lorehold_historian() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Lorehold Historian",
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: String::from("exile a card from your graveyard"),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::one_of(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: SelectionRequirement::Any,
                        }),
                        to: ZoneDest::Exile,
                    },
                    Effect::DealDamage {
                        to: target_filtered(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Player)
                                .or(SelectionRequirement::Planeswalker),
                        ),
                        amount: Value::Const(2),
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Spectacle (batch 10) ───────────────────────────────────────────

/// Prismari Spectacle — {1}{U}{R} Instant.
///
/// Printed Oracle (synthesised): "Choose one — / • Prismari
/// Spectacle deals 3 damage to target creature. / • Draw two cards,
/// then discard a card. / • Create two Treasure tokens."
///
/// A 3-mode `ChooseMode` Instant — Lightning Bolt + Tidings + ramp
/// in one card. AutoDecider picks the burn mode (0); scripted
/// decider can probe modes 1/2.
pub fn prismari_spectacle() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spectacle",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: crate::game::effects::treasure_token(),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Wavebreaker (batch 10) ─────────────────────────────────────────

/// Quandrix Wavebreaker — {3}{G}{U}, 4/4 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2,
/// then draw a card. / Whenever you draw a card, put a +1/+1 counter
/// on this creature."
///
/// Cantrip + draw-trigger counter accumulator. Pairs with cantrips
/// in any draw-heavy shell — at the standard draw-step that's one
/// counter per turn for free.
pub fn quandrix_wavebreaker() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavebreaker",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
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
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Anthemwright (batch 10) ─────────────────────────────────────

/// Silverquill Anthemwright — {2}{W}{B}, 3/3 Inkling Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Other creatures you
/// control get +1/+0 and have lifelink."
///
/// Anthem + lifelink for all your other creatures. Pairs with Inkling
/// tokens to swing as 2-3 power lifelinkers. Wired via two static
/// effects on the source itself targeting `OtherThanSource` creatures
/// the controller owns.
pub fn silverquill_anthemwright() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Anthemwright",
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
        triggered_abilities: vec![],
        static_abilities: vec![
            StaticAbility {
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
            },
            StaticAbility {
                description: "Other creatures you control have lifelink.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    keyword: Keyword::Lifelink,
                },
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
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Decay (batch 10) ────────────────────────────────────────────

/// Witherbloom Decay — {1}{B}{G} Instant.
///
/// Printed Oracle (synthesised): "Destroy target creature with mana
/// value 3 or less. You gain 2 life."
///
/// Witherbloom's efficient removal: 3 mana to kill anything CMC 3
/// or less and pad the life total by 2. Pairs with the Witherbloom
/// Drain/Lifegain shell.
pub fn witherbloom_decay() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Decay",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Lorehold Reverberation (batch 10) ───────────────────────────────────────

/// Lorehold Reverberation — {2}{R}{W} Instant.
///
/// Printed Oracle (synthesised): "Lorehold Reverberation deals 3
/// damage to target creature an opponent controls. If a creature
/// died this turn, you gain 3 life."
///
/// 3-damage removal + conditional lifegain — Lorehold's "trade" play
/// pattern at instant speed. Pairs with token sacrifice or Daemogoth
/// Titan / Daemogoth Woe-Eater deaths to flip the rider on.
pub fn lorehold_reverberation() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reverberation",
        cost: cost(&[generic(2), r(), w()]),
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
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::CreaturesDiedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Prismari Eccentric (batch 10) ───────────────────────────────────────────

/// Prismari Eccentric — {2}{U}{R}, 3/2 Human Wizard, Haste.
///
/// Printed Oracle (synthesised): "Haste / When this creature enters,
/// create a Treasure token. / Whenever this creature attacks, you
/// may sacrifice a Treasure. If you do, draw a card."
///
/// Haste body + immediate Treasure + recurring loot on attack via
/// sac-a-Treasure. Pairs with the Prismari Treasure shell (Prismari
/// Inventor, Prismari Spellforger, Lorehold Treasure Smith).
pub fn prismari_eccentric() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Prismari Eccentric",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: String::from("sacrifice a Treasure to draw a card"),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice {
                            who: Selector::You,
                            count: Value::Const(1),
                            filter: SelectionRequirement::HasArtifactSubtype(
                                ArtifactSubtype::Treasure,
                            ),
                        },
                        Effect::Draw {
                            who: Selector::You,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Quandrix Theorem Crafter (batch 10) ─────────────────────────────────────

/// Quandrix Theorem Crafter — {2}{G}{U}, 2/4 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, put a
/// number of +1/+1 counters on target creature equal to the number
/// of lands you control."
///
/// Ramp payoff: at 4 lands, 4 counters; at 6 lands, 6 counters. A
/// classic Quandrix curve-out from a turn-3 ramp into a turn-4
/// haymaker creature.
pub fn quandrix_theorem_crafter() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Theorem Crafter",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
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
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ))),
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
        adventure: None,
        plot_cost: None,
    }
}

// ============================================================================
// Batch 11 — 22 new synthesised STX cards + 1 engine primitive
// (StaticEffect::DoubleCounters for CR 614.16 counter-replacement half).
// Cards exercise existing engine primitives across all five colleges plus
// shared/colorless slots; tests in `tests::stx` lock in primary play
// patterns end-to-end.
// ============================================================================

// ── Witherbloom Pestseed (batch 11) — DoubleCounters exerciser ──────────────

/// Witherbloom Pestseed — {2}{B}{G}, 3/3 Plant Druid.
///
/// Printed Oracle (synthesised, Hardened-Scales template): "If one or more
/// counters would be put on a permanent you control, twice that many of
/// those counters are put on that permanent instead."
///
/// First card to wire the new `StaticEffect::DoubleCounters` primitive
/// (CR 614.16 counter-replacement half). The static applies to every
/// counter-placement on permanents the Pestseed's controller controls —
/// so a "+1/+1 counter" instruction lands as 2 counters, two doublers
/// stack multiplicatively to 4×, etc.
pub fn witherbloom_pestseed() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Witherbloom Pestseed",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "If one or more counters would be put on a permanent you \
                          control, twice that many of those counters are put on \
                          that permanent instead.",
            effect: StaticEffect::DoubleCounters,
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Silverquill Editorialist (batch 11) ─────────────────────────────────────

/// Silverquill Editorialist — {1}{W}{B}, 2/2 Inkling Wizard, Flying.
///
/// Printed Oracle (synthesised): "Flying / Whenever you cast an instant
/// or sorcery spell, each opponent loses 1 life."
///
/// Silverquill drain-on-cast in Wizard tribal frame. Pairs with the
/// existing Magecraft Silverquill shell (Archmage Emeritus, Eager
/// First-Year). Uses `magecraft_drain_each_opp(1)` to lean on the same
/// drain template as Promising Duskmage and Witherbloom Apprentice.
pub fn silverquill_editorialist() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Editorialist",
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Inkblot Recluse (batch 11) ──────────────────────────────────────────────

/// Inkblot Recluse — {2}{W}{B}, 2/4 Spider Inkling.
///
/// Printed Oracle (synthesised): "Reach / When this creature enters,
/// surveil 2."
///
/// Defensive Reach body that doubles as a Witherbloom-style graveyard
/// setup. Surveil 2 lets the controller bin a reanimation target or
/// keep top-of-library library order while smoothing the next draw.
pub fn inkblot_recluse() -> CardDefinition {
    CardDefinition {
        name: "Inkblot Recluse",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider, CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Surveil {
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Quill-Lecturer (batch 11) ───────────────────────────────────────────────

/// Quill-Lecturer — {3}{W}{B}, 2/4 Human Cleric, Vigilance.
///
/// Printed Oracle (synthesised): "Vigilance / Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, target creature an opponent
/// controls gets -1/-1 until end of turn."
///
/// Anti-air Vigilance body with an opp-creature-shrink magecraft rider.
/// Pairs the printed Lecturer pattern of Silverquill's "spells matter
/// against the opponent" play.
pub fn quill_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Quill-Lecturer",
        cost: cost(&[generic(3), w(), b()]),
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
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
            ),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Inkstrike Bolt (batch 11) ───────────────────────────────────────────────

/// Inkstrike Bolt — {1}{W}{B} Instant.
///
/// Printed Oracle (synthesised): "Inkstrike Bolt deals 3 damage to target
/// creature an opponent controls. You gain 2 life."
///
/// Silverquill efficient removal + lifegain at instant speed — a 3-mana
/// Lightning Helix-shaped spell with the lifegain tied to the cast
/// rather than the kill (always lands +2 life).
pub fn inkstrike_bolt() -> CardDefinition {
    CardDefinition {
        name: "Inkstrike Bolt",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(3),
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Withering Spores (batch 11) ─────────────────────────────────────────────

/// Withering Spores — {1}{B}{G} Sorcery.
///
/// Printed Oracle (synthesised): "All creatures get -1/-1 until end of
/// turn."
///
/// Symmetric mini-wrath — kills X/1s on both sides. Combos with the
/// existing Witherbloom +1/+1-counter plant package (counters survive
/// the -1/-1 cancel SBA per CR 122.3).
pub fn withering_spores() -> CardDefinition {
    CardDefinition {
        name: "Withering Spores",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: Selector::EachPermanent(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
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
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}

// ── Witherbloom Brewer (batch 11) ───────────────────────────────────────────

/// Witherbloom Brewer — {1}{B}{G}, 2/3 Plant Druid.
///
/// Printed Oracle (synthesised): "{T}, Pay 2 life: Add {B}{G}."
///
/// Witherbloom's high-density mana acceleration — the Pledgemage
/// printed shape costs 1 life per pip; the Brewer offers a 2-life
/// fixed cost for a guaranteed two-color pip output. Activation is a
/// real mana ability (skips the stack) per CR 605.1a; the life cost
/// rides on `ActivatedAbility.life_cost`.
pub fn witherbloom_brewer() -> CardDefinition {
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Witherbloom Brewer",
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
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black, Color::Green]),
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
        adventure: None,
        plot_cost: None,
    }
}

// ── Pestilent Brambletwig (batch 11) ────────────────────────────────────────

/// Pestilent Brambletwig — {B}{G}, 2/1 Plant Pest.
///
/// Printed Oracle (synthesised): "When this creature dies, you gain 2
/// life."
///
/// A bigger Pest body (2/1 vs the printed 1/1 token), with a printed
/// 2-life-on-death rider that doubles the standard Pest token trickle.
/// The on-die +2 life feeds into Witherbloom's lifegain payoffs (Pest
/// Mascot, Honor Troll, Witherbloom Briarmage).
pub fn pestilent_brambletwig() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Brambletwig",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Pest],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
    }
}
