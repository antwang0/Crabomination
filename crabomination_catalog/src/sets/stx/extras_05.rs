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
use crate::mana::{Color, b, colorless, cost, g, generic, hybrid, mono_hybrid, phyrexian, r, u, w, x, ManaCost};

// ── Bookwurm ────────────────────────────────────────────────────────────────

// Bookwurm — {5}{G}{G}, 5/5 Wurm. "Trample / When this creature enters,
// you gain 4 life and draw a card."
//
// ✅ ETB body is a simple `Seq(GainLife(4), Draw(1))`. The 5/5 trample
// body is a fine top-end finisher in any green deck.

// ── Pestilent Inkmage ──────────────────────────────────────────────────────

/// Pestilent Inkmage — {2}{W}{B}, 2/4 Human Wizard with Lifelink
/// (synthesised STX Silverquill flavor). "Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, this creature gets
/// +2/+0 until end of turn."
///
/// A Magecraft pump-and-Lifelink finisher — every IS spell turns the
/// Inkmage into a 4/4 Lifelinker. Pairs naturally with cheap cantrips
/// (Make Your Mark, Containment Breach). Tests:
/// `pestilent_inkmage_magecraft_pumps_self_two_zero`,
/// `pestilent_inkmage_does_not_trigger_on_creature_cast`.
pub fn pestilent_inkmage() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Inkmage",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(2, 0)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkling Reaver ─────────────────────────────────────────────────────────

/// Inkling Reaver — {3}{B}, 3/3 Inkling Warrior with Menace
/// (synthesised STX Silverquill flavor).
///
/// A 4-mana Inkling Warrior with Menace — a hard-to-block midrange
/// threat that ramps into the Tenured Inkcaster anthem (5/5 Menace).
/// Test: `inkling_reaver_is_a_four_mana_three_three_menace_inkling_warrior`.
pub fn inkling_reaver() -> CardDefinition {
    CardDefinition {
        name: "Inkling Reaver",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quintessential Inkling ─────────────────────────────────────────────────

/// Quintessential Inkling — {1}{W}{B}, 2/2 Inkling Spirit with Flying
/// and Lifelink (synthesised STX Silverquill flavor).
///
/// A 3-mana 2/2 Flying/Lifelink Inkling — the curve-fitter between
/// Inkling Mascot ({W}{B} 2/2 Repartee) and Tenured Inkcaster
/// ({2}{W}{B} 3/2 lord). With Tenured Inkcaster on the battlefield,
/// becomes a 4/4 Flying/Lifelink racer. Test:
/// `quintessential_inkling_is_a_three_mana_two_two_flying_lifelink_inkling_spirit`.
pub fn quintessential_inkling() -> CardDefinition {
    CardDefinition {
        name: "Quintessential Inkling",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Spirit],
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quill Witch ────────────────────────────────────────────────────────────

/// Quill Witch — {1}{B}{B}, 2/2 Human Warlock with Flying
/// (synthesised STX Silverquill flavor). "Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, target opponent loses
/// 1 life and you gain 1 life."
///
/// A black drain Magecraft body — converts every cantrip into 2 life
/// of swing while pressuring through the air. Same magecraft template
/// as Promising Duskmage (Inkling) but on a sturdier 3-mana body.
/// Tests: `quill_witch_magecraft_drains_one_on_instant_cast`,
/// `quill_witch_is_a_three_mana_two_two_flying_warlock`.
pub fn quill_witch() -> CardDefinition {
    CardDefinition {
        name: "Quill Witch",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lesson in Honor ────────────────────────────────────────────────────────

/// Lesson in Honor — {1}{W} Sorcery — Lesson (synthesised STX
/// Silverquill flavor). "Target creature gets +2/+2 until end of
/// turn. Learn."
///
/// A combat trick Lesson — pumps a friendly +2/+2 EOT and Learns via
/// `Effect::Learn`. Mirror to Fortifying Draught
/// (which gives +1/+4) and Guiding Voice (+1/+1 counter + Learn);
/// Lesson in Honor goes wider on the offensive curve. Tests:
/// `lesson_in_honor_pumps_two_two_and_cantrips`,
/// `lesson_in_honor_is_a_two_mana_white_sorcery`.
pub fn lesson_in_honor() -> CardDefinition {
    CardDefinition {
        name: "Lesson in Honor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
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
            // Learn (CR 701.45) — reveal a Lesson into hand or discard-to-draw.
            Effect::Learn { who: crate::effect::PlayerRef::You },
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkling Squad ──────────────────────────────────────────────────────────

/// Inkling Squad — {3}{W}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Create three 1/1 white and black Inkling creature tokens
/// with flying."
///
/// 5-mana go-wide. Mirror to Defend the Campus ({3}{W}{W} for three
/// Inklings); Inkling Squad is the Silverquill (W/B) color-pip
/// equivalent. Excellent late-game flood payoff and Felisa engine
/// enabler (each Inkling that dies after picking up a +1/+1 counter
/// mints another Inkling). Tests:
/// `inkling_squad_creates_three_inkling_tokens`,
/// `inkling_squad_is_a_five_mana_wb_sorcery`.
pub fn inkling_squad() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Squad",
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkling Drillmaster ────────────────────────────────────────────────────

/// Inkling Drillmaster — {1}{W}, 1/2 Inkling Soldier with Flying
/// (synthesised STX Silverquill flavor). "When this creature enters,
/// put a +1/+1 counter on another target Inkling creature you
/// control."
///
/// An Inkling-tribal ETB anthem-of-one. ETB:
/// `AddCounter +1/+1 → target Inkling & ControlledByYou & OtherThanSource`.
/// Pairs naturally with Inkling Squire ({W} 1/1 Flying) — turns turn
/// 2 into a 2/2 Flier alongside the Drillmaster's own 1/2 Flier.
/// Tests: `inkling_drillmaster_etb_pumps_other_inkling`,
/// `inkling_drillmaster_etb_does_not_target_non_inkling`.
pub fn inkling_drillmaster() -> CardDefinition {
    CardDefinition {
        name: "Inkling Drillmaster",
        cost: cost(&[generic(1), w()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Sealing Verse ──────────────────────────────────────────────────────────

/// Sealing Verse — {W}{B} Instant (synthesised STX Silverquill
/// flavor). "Exile target creature with mana value 3 or less."
///
/// Two-mana exile removal capped at MV ≤ 3 — answers most early
/// pressure pieces (Star Pupil, Eager First-Year, Witherbloom
/// Apprentice, Eyetwitch, Silverquill Pledgemage). Exile (not
/// destroy) sidesteps "when this dies" triggers like Eyetwitch's
/// Learn and Star Pupil's counter transfer. Tests:
/// `sealing_verse_exiles_low_mv_creature`,
/// `sealing_verse_rejects_high_mv_target`.
pub fn sealing_verse() -> CardDefinition {
    CardDefinition {
        name: "Sealing Verse",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Strict Tutelage ────────────────────────────────────────────────────────

/// Strict Tutelage — {1}{W}{B} Enchantment (synthesised STX
/// Silverquill flavor). "Whenever an opponent draws a card, that
/// player loses 1 life."
///
/// An Underworld Dreams-style passive drain on opp card-draw —
/// payoff for forcing opp draws (Tezzeret's Gambit, Tendrils of
/// Agony's storm spam) and steady-pressure against control mirrors
/// where opp uses cantrips to dig. Wired via `CardDrawn /
/// OpponentControl → LoseLife 1` against the firing player. Tests:
/// `strict_tutelage_drains_opp_on_each_draw`,
/// `strict_tutelage_does_not_drain_you_on_your_draw`.
pub fn strict_tutelage() -> CardDefinition {
    CardDefinition {
        name: "Strict Tutelage",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Triggerer),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkrise Vampire ────────────────────────────────────────────────────────

/// Inkrise Vampire — {2}{B}, 2/3 Vampire Warlock with Lifelink
/// (synthesised STX Silverquill flavor).
///
/// A 3-mana 2/3 Lifelinker — body upgrade to Codespell Cleric
/// (1-mana 1/1 Lifelink) for the midrange curve. Synergises with
/// Stridehollow Vampire (Vampire tribal) and Pestilent Acolyte's
/// (Human/Warlock) ETB -1/-1 effects. Test:
/// `inkrise_vampire_is_a_three_mana_two_three_lifelink_vampire_warlock`.
pub fn inkrise_vampire() -> CardDefinition {
    CardDefinition {
        name: "Inkrise Vampire",
        cost: cost(&[generic(2), b()]),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Silverquill Sting ──────────────────────────────────────────────────────

/// Silverquill Sting — {W}{B} Instant (synthesised STX Silverquill
/// flavor). "Target opponent loses 2 life. You gain 2 life."
///
/// Two-mana cheap drain instant — same drain template as Tribute to
/// Hunger but without the sacrifice rider. Useful as a finisher in
/// Silverquill burn/drain shells. Wired via `Effect::Drain { from:
/// Target(0), to: You, amount: 2 }`. Tests:
/// `silverquill_sting_drains_opp_by_two`,
/// `silverquill_sting_is_a_two_mana_wb_instant`.
pub fn silverquill_sting() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Sting",
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Blade Historian ────────────────────────────────────────────────────────

/// Blade Historian — {R/W}{R/W}{R/W}{R/W}, 2/3 Human Cleric (STX Lorehold rare).
/// "Attacking creatures you control have double strike." Wired via the
/// `GrantKeywordToAttackers` static (resolved against the live attacking
/// set at `compute_battlefield` time).
pub fn blade_historian() -> CardDefinition {
    CardDefinition {
        name: "Blade Historian",
        cost: cost(&[hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White), hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control have double strike.",
            effect: StaticEffect::GrantKeywordToAttackers {
                keyword: Keyword::DoubleStrike,
            },
        }],
        ..Default::default()
    }
}

// ── Carving Cherub ─────────────────────────────────────────────────────────

/// Carving Cherub — {W}, 1/1 Spirit (printed STX Silverquill flavor).
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target creature gets +1/+1 until end of turn."
///
/// Same magecraft template as Eager First-Year ({W} 2/1 with the same
/// magecraft) but on a 1/1 Spirit body — slots into Silverquill /
/// Spirit tribal decks (Hofri, Quintorius). Test:
/// `carving_cherub_is_a_one_mana_one_one_spirit_with_magecraft`.
pub fn carving_cherub() -> CardDefinition {
    CardDefinition {
        name: "Carving Cherub",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkrider Witch ─────────────────────────────────────────────────────────

/// Inkrider Witch — {1}{B}, 2/2 Human Rogue with Menace (synthesised
/// STX Silverquill flavor).
///
/// A 2-mana 2/2 Menace body — early Black aggressive Rogue/Warlock
/// tribal that pressures opp's life total. Test:
/// `inkrider_witch_is_a_two_mana_two_two_menace_human_rogue`.
pub fn inkrider_witch() -> CardDefinition {
    CardDefinition {
        name: "Inkrider Witch",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Roving Scholar ─────────────────────────────────────────────────────────

/// Roving Scholar — {3}{U}, 2/3 Human Wizard (synthesised STX
/// Quandrix-adjacent flavor). "When this creature enters, each
/// player draws two cards."
///
/// A symmetrical 4-mana 2/3 with Howling Mine-style ETB draw. Both
/// players draw 2 — net card velocity for the caster in a deck that
/// can leverage the cards faster (Wheel of Fortune-style template).
/// Tests: `roving_scholar_etb_each_player_draws_two`,
/// `roving_scholar_is_a_four_mana_two_three_human_wizard`.
pub fn roving_scholar() -> CardDefinition {
    CardDefinition {
        name: "Roving Scholar",
        cost: cost(&[generic(3), u()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Forceful Mirror ────────────────────────────────────────────────────────

/// Forceful Mirror — {2}{U} Sorcery (synthesised STX Quandrix flavor).
/// "Copy target instant or sorcery spell you control. You may choose
/// new targets for the copy."
///
/// Counter-Twincast at 3 mana — the budget Quandrix copy spell. The
/// "you may choose new targets" rider collapses to "copy inherits
/// targets" (engine-wide CopySpell gap). Tests:
/// `forceful_mirror_copies_target_instant`,
/// `forceful_mirror_is_a_three_mana_blue_sorcery`.
pub fn forceful_mirror() -> CardDefinition {
    CardDefinition {
        name: "Forceful Mirror",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::HasCardType(CardType::Instant).or(
                        SelectionRequirement::HasCardType(CardType::Sorcery),
                    )),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Fractalic Discovery ────────────────────────────────────────────────────

/// Fractalic Discovery — {2}{G}{U} Sorcery (synthesised STX Quandrix
/// flavor). "Draw three cards, then put two cards from your hand on
/// top of your library."
///
/// Inspired Idea reprint at Quandrix mana. Pure card-velocity dig +
/// stack. Wired as `Seq(Draw 3, PutOnLibraryFromHand 2)`. Tests:
/// `fractalic_discovery_draws_three_then_stacks_two`,
/// `fractalic_discovery_is_a_four_mana_gu_sorcery`.
pub fn fractalic_discovery() -> CardDefinition {
    CardDefinition {
        name: "Fractalic Discovery",
        cost: cost(&[generic(2), g(), u()]),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Lookback ──────────────────────────────────────────────────────

/// Lorehold Lookback — {2}{R}{W} Sorcery (synthesised STX Lorehold
/// flavor). "Return target creature or artifact card from your
/// graveyard to your hand. Mints a 2/2 R/W Spirit token with flying."
///
/// Reanimation + body — combines Pillardrop Rescuer's gy-to-hand
/// recursion with Sparring Regimen's 2/2 R/W Spirit-token mint.
/// Tests: `lorehold_lookback_returns_creature_from_gy_and_creates_spirit`,
/// `lorehold_lookback_is_a_four_mana_rw_sorcery`.
pub fn lorehold_lookback() -> CardDefinition {
    use crate::catalog::sets::stx::lorehold_spirit_token;
    CardDefinition {
        name: "Lorehold Lookback",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::HasCardType(CardType::Artifact)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: lorehold_spirit_token(),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Reaper Spirit ──────────────────────────────────────────────

/// Witherbloom Reaper Spirit — {2}{B}{G}, 4/3 Plant Spirit with
/// Deathtouch (synthesised STX Witherbloom flavor).
///
/// A 4-mana 4/3 deathtoucher — same body template as Witherbloom
/// Reaper but without the ETB edict (which Reaper has). Pure combat
/// presence for Witherbloom midrange. Test:
/// `witherbloom_reaper_spirit_is_a_four_mana_four_three_deathtouch_plant_spirit`.
pub fn witherbloom_reaper_spirit() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaper Spirit",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Lifedrinker ────────────────────────────────────────────────

/// Witherbloom Lifedrinker — {1}{B}, 1/3 Plant Warlock with Lifelink
/// (synthesised STX Witherbloom flavor). "Whenever you gain life,
/// put a +1/+1 counter on this creature."
///
/// A 2-mana 1/3 Lifelink Pest-style payoff — every Lifelink swing
/// or drain effect (Witherbloom Apprentice, Promising Duskmage,
/// Beledros) pumps the body. Wired via `LifeGained / YourControl
/// → AddCounter(+1/+1)` on `Selector::This`. Tests:
/// `witherbloom_lifedrinker_is_a_two_mana_one_three_lifelink_plant_warlock`,
/// `witherbloom_lifedrinker_grows_on_lifegain`.
pub fn witherbloom_lifedrinker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Lifedrinker",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Battlemaster ──────────────────────────────────────────────────

/// Lorehold Battlemaster — {2}{R}{W}, 3/3 Spirit Cleric with Haste +
/// First Strike (synthesised STX Lorehold flavor).
///
/// A 4-mana 3/3 Haste + First Strike Spirit — a more aggressive body
/// alternative to the existing 2/4 Lorehold Battle-Priest. Slots
/// into Hofri/Quintorius Spirit tribal as a tempo finisher. Test:
/// `lorehold_battlemaster_is_a_four_mana_three_three_haste_first_strike_spirit_cleric`.
pub fn lorehold_battlemaster() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battlemaster",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste, Keyword::FirstStrike],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Prismari Spellfire ─────────────────────────────────────────────────────

/// Prismari Spellfire — {3}{U}{R} Sorcery (synthesised STX Prismari
/// flavor). "Prismari Spellfire deals 5 damage to target creature or
/// planeswalker. Draw a card."
///
/// 5-mana 5-damage burn + cantrip — Prismari's headline removal/
/// finisher hybrid. Mirror to Pyromancer's Bolt (3 dmg, {1}{R}) but
/// scaled up to 5 dmg + cantrip. Tests:
/// `prismari_spellfire_burns_for_five_and_cantrips`,
/// `prismari_spellfire_is_a_five_mana_ur_sorcery`.
pub fn prismari_spellfire() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spellfire",
        cost: cost(&[generic(3), u(), r()]),
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
                        .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                ),
                amount: Value::Const(5),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Recalibrator ──────────────────────────────────────────────────

/// Quandrix Recalibrator — {1}{G}{U}, 2/2 Elf Wizard (synthesised STX
/// Quandrix flavor). "When this creature enters, put a +1/+1 counter
/// on each creature you control."
///
/// A 3-mana 2/2 fan-out anthem ETB — every friendly creature picks
/// up a +1/+1 counter on resolution. Pairs naturally with Tanazir
/// Quandrix's counter-doubling and Practical Research's "double the
/// counters" payoff. Tests:
/// `quandrix_recalibrator_etb_fans_counters`,
/// `quandrix_recalibrator_is_a_three_mana_two_two_elf_wizard`.
pub fn quandrix_recalibrator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Recalibrator",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Crackleburr Initiate ───────────────────────────────────────────────────

/// Crackleburr Initiate — {U}{R}, 2/1 Human Wizard with Flash
/// (synthesised STX Prismari flavor). "Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature gets +1/+0
/// until end of turn."
///
/// Symmetry Sage ({U} 1/2) at a wider P/T budget — 2-mana 2/1 with
/// Flash and Magecraft self-pump. Useful as a flash threat that
/// scales with Prismari's spell-heavy game plan. Tests:
/// `crackleburr_initiate_is_a_two_mana_two_one_flash_human_wizard`,
/// `crackleburr_initiate_magecraft_pumps_self_one_zero`.
pub fn crackleburr_initiate() -> CardDefinition {
    CardDefinition {
        name: "Crackleburr Initiate",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Spellseeker's Insight ──────────────────────────────────────────────────

/// Spellseeker's Insight — {1}{U} Instant (synthesised STX Prismari
/// flavor). "Search your library for an instant or sorcery card with
/// mana value 3 or less, reveal it, put it into your hand, then
/// shuffle."
///
/// 2-mana tutor for cheap removal / cantrip / counter. Mirror to
/// Mystical Inquiry (open-ended IS tutor at {2}{U}) — Spellseeker's
/// Insight caps at MV ≤ 3 but ships at the rate-efficient 2-mana
/// slot. Tests:
/// `spellseekers_insight_is_a_two_mana_blue_instant`,
/// `spellseekers_insight_tutors_a_low_mv_instant`.
pub fn spellseekers_insight() -> CardDefinition {
    CardDefinition {
        name: "Spellseeker's Insight",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: (SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)))
            .and(SelectionRequirement::ManaValueAtMost(3)),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Burrog Snapper ─────────────────────────────────────────────────────────

/// Burrog Snapper — {1}{U}, 2/2 Frog Wizard with Flash (synthesised
/// STX Prismari-adjacent flavor). "When this creature enters,
/// target creature gets -2/-0 until end of turn."
///
/// Same ETB combat trick as Burrog Befuddler but lands a 2/2 (vs.
/// 2/1) body. Tests:
/// `burrog_snapper_etb_minus_two_zero`,
/// `burrog_snapper_is_a_two_mana_two_two_frog_wizard_with_flash`.
pub fn burrog_snapper() -> CardDefinition {
    CardDefinition {
        name: "Burrog Snapper",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Galvanic Ribbons ───────────────────────────────────────────────────────

/// Galvanic Ribbons — {1}{R} Instant (synthesised STX Prismari
/// flavor). "Galvanic Ribbons deals 2 damage to any target. Draw a
/// card if you control an artifact."
///
/// 2-mana burn + conditional cantrip — pairs with Treasure tokens
/// from Storm-Kiln Artist / Prismari Command / Galazeth Prismari.
/// Wired as `Seq(DealDamage 2 → creature/PW/player, If(SelectorExists
/// EachPermanent(Artifact & ControlledByYou)) → Draw 1)`. Tests:
/// `galvanic_ribbons_burns_for_two`,
/// `galvanic_ribbons_cantrips_with_artifact_in_play`.
pub fn galvanic_ribbons() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Ribbons",
        cost: cost(&[generic(1), r()]),
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
                        .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
                ),
                amount: Value::Const(2),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .and(SelectionRequirement::ControlledByYou),
                )),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Plant Mascot ───────────────────────────────────────────────────────────

/// Plant Mascot — {1}{G}, 2/2 Plant (synthesised STX Witherbloom
/// flavor). "When this creature enters, target creature you control
/// gets +1/+1 until end of turn."
///
/// 2-mana 2/2 with a one-shot ETB pump — useful as a tempo enabler
/// for Witherbloom decks that need a fast +1/+1 push. Tests:
/// `plant_mascot_etb_pumps_friendly_creature`,
/// `plant_mascot_is_a_two_mana_two_two_plant`.
pub fn plant_mascot() -> CardDefinition {
    CardDefinition {
        name: "Plant Mascot",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Wavebender ────────────────────────────────────────────────────

/// Quandrix Wavebender — {1}{G}{U}, 2/3 Elf Druid (synthesised STX
/// Quandrix flavor). "Whenever you cast a spell with {X} in its
/// mana cost, put X +1/+1 counters on this creature."
///
/// A 3-mana 2/3 Elf Druid that scales with X-cost spells. Pairs
/// naturally with Geometer's Arthropod / Paradox Surveyor (both
/// already wired). Wired via the `Predicate::CastSpellHasX` filter
/// plus `Value::XFromCost` (read from `EffectContext.x_value` of
/// the resolving spell — threaded by the dispatcher into
/// `ctx.mana_spent` for spell-cast triggers). Tests live keyed by
/// `quandrix_wavebender_is_a_three_mana_two_three_elf_druid`.
pub fn quandrix_wavebender() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavebender",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellHasX),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::XFromCost,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Tezzeret's Inkling Forge ───────────────────────────────────────────────

/// Tezzeret's Inkling Forge — {1}{W}{B} Enchantment (synthesised STX
/// Silverquill flavor). "At the beginning of your end step, create a
/// 1/1 white and black Inkling creature token with flying."
///
/// Per-turn Inkling token generator. Wired via the `StepBegins
/// (EndStep)/ActivePlayer` trigger (so it only fires on your own
/// end step). Mints one Inkling per turn — slow but inevitable
/// go-wide finisher. Tests:
/// `tezzerets_inkling_forge_is_a_three_mana_wb_enchantment`.
pub fn tezzerets_inkling_forge() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Tezzeret's Inkling Forge",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            // Only fires on the controller's own end step. The
            // ActivePlayer scope already gates this — on opp's end step,
            // active = opp ≠ controller of Forge, so the trigger
            // wouldn't fire.
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Snake-Charmer ─────────────────────────────────────────────────

/// Quandrix Snake-Charmer — {2}{G}, 3/3 Snake Druid (synthesised STX
/// Quandrix flavor). "When this creature enters, draw a card."
///
/// 3-mana 3/3 Elvish Visionary upgrade — efficient body + cantrip
/// in green. Slots into any Quandrix midrange shell. Tests:
/// `quandrix_snake_charmer_is_a_three_mana_three_three_snake_druid`,
/// `quandrix_snake_charmer_etb_cantrips`.
pub fn quandrix_snake_charmer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Snake-Charmer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Necrotouch ─────────────────────────────────────────────────

/// Witherbloom Necrotouch — {2}{B}{G} Instant (synthesised STX
/// Witherbloom flavor). "Destroy target creature. You gain 2 life."
///
/// 4-mana premium removal + life buffer. Mirror to Grapple with
/// Death (already exists as {1}{B}{G}, +1 life) but trades flexibility
/// (no artifact mode) for life-gain depth (2 life instead of 1).
/// Tests: `witherbloom_necrotouch_destroys_creature_and_gains_two`,
/// `witherbloom_necrotouch_is_a_four_mana_bg_instant`.
pub fn witherbloom_necrotouch() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necrotouch",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Silverquill Apprentice ─────────────────────────────────────────────────

// ── Pestilent Lecturer ─────────────────────────────────────────────────────

/// Pestilent Lecturer — {1}{W}{B}, 2/3 Inkling Cleric with Flying.
/// "When this creature enters, each opponent loses 1 life and you
/// gain 1 life."
///
/// Inkling tribal payoff with a drain ETB. Buffs to a 4/5 under
/// Tenured Inkcaster's +2/+2 anthem.
pub fn pestilent_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Lecturer",
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
        triggered_abilities: vec![etb_drain(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Shadow-Mage Hopeful ────────────────────────────────────────────────────

/// Shadow-Mage Hopeful — {1}{W}{B}, 2/2 Human Wizard with Lifelink.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// each opponent loses 1 life and you gain 1 life."
///
/// Lifelink magecraft drain — at 1 lifelink + 1 drain per spell, the
/// life swap quickly snowballs in a control deck running cheap cantrips.
pub fn shadow_mage_hopeful() -> CardDefinition {
    CardDefinition {
        name: "Shadow-Mage Hopeful",
        cost: cost(&[generic(1), w(), b()]),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quill Page ─────────────────────────────────────────────────────────────

/// Quill Page — {W}, 1/1 Human Cleric. "Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, scry 1."
///
/// Library-velocity Magecraft — keeps the deck flowing in a
/// spells-matter shell. Bare 1/1 body for one mana means it just sits
/// behind the scry engine.
pub fn quill_page() -> CardDefinition {
    CardDefinition {
        name: "Quill Page",
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkbond Cleric ─────────────────────────────────────────────────────────

/// Inkbond Cleric — {2}{W}, 2/3 Human Cleric. "When this creature
/// enters, surveil 1 and put a +1/+1 counter on another target Inkling
/// you control."
///
/// Surveil + Inkling counter ETB — a tribal payoff for Silverquill's
/// Inkling theme. Slots into any deck running Inkling Drillmaster,
/// Pestilent Lecturer, etc.
pub fn inkbond_cleric() -> CardDefinition {
    CardDefinition {
        name: "Inkbond Cleric",
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
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasCreatureType(CreatureType::Inkling))
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    kind: CounterType::PlusOnePlusOne,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quill Inscriber ────────────────────────────────────────────────────────

/// Quill Inscriber — {1}{B}, 2/2 Human Warlock. "Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, this creature gets
/// +1/+0 until end of turn."
///
/// Wraps `magecraft_self_pump(1, 0)` — same template as Symmetry Sage
/// in Prismari but in Silverquill colors. A common Magecraft creature
/// shape across all colleges.
pub fn quill_inscriber() -> CardDefinition {
    CardDefinition {
        name: "Quill Inscriber",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Pestilent Squire ───────────────────────────────────────────────────────

/// Pestilent Squire — {1}{B}, 2/1 Pest Warrior with Lifelink.
///
/// Aggressive Pest with the canonical Witherbloom lifelink body —
/// turns one of your own Magecraft drains into life advantage on the
/// attack. Synergises with any Pest-tribal payoff (Eyetwitch Brood).
pub fn pestilent_squire() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Squire",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Silverquill Mediator ───────────────────────────────────────────────────

/// Silverquill Mediator — {3}{W}{B}, 3/4 Inkling Cleric with Flying
/// and Lifelink. "When this creature enters, each opponent loses 2
/// life and you gain 2 life."
///
/// Top-of-curve Silverquill drain finisher. Flying + Lifelink + on-ETB
/// drain shifts 4 life on a single attack swing (drain 2 + lifelink 3
/// on the attacker).
pub fn silverquill_mediator() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Mediator",
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Dissident Lecturer ─────────────────────────────────────────────────────

/// Dissident Lecturer — {2}{B}, 2/3 Human Warlock. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, each opponent
/// loses 1 life."
///
/// Pure burn Magecraft (no lifegain rider, so it's strictly different
/// from Witherbloom Apprentice). Stacks with Promising Duskmage in any
/// Silverquill drain deck.
pub fn dissident_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Dissident Lecturer",
        cost: cost(&[generic(2), b()]),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Silverquill Persuader ──────────────────────────────────────────────────

/// Silverquill Persuader — {2}{W}{B}, 2/3 Inkling Wizard with Flying.
/// "Other Cleric creatures you control get +1/+1."
///
/// Tribal anthem for Clerics (Silverquill's secondary tribe alongside
/// Inklings). The flying body slots into the air force; the anthem
/// rewards stacking Pestilent Lecturer, Inkbond Cleric, Pestilent
/// Acolyte, Silverquill Mediator, etc.
pub fn silverquill_persuader() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Persuader",
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
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other Cleric creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Cleric))
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Pestilent Imp ──────────────────────────────────────────────────────────

/// Pestilent Imp — {B}, 1/1 Imp Pest with Flying.
///
/// One-mana flying Pest. Slots into Inkling/Pest go-wide shells with
/// evasion. Synergises with Eyetwitch Brood's "another Pest dies"
/// counter trigger and any Pest tribal anthem.
pub fn pestilent_imp() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Imp",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp, CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Tincture-Maker ─────────────────────────────────────────────

/// Witherbloom Tincture-Maker — {1}{B}{G}, 2/3 Human Druid.
/// "Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, you gain 1 life."
///
/// Pure lifegain Magecraft — slots into Witherbloom lifegain shells
/// (Honor Troll, Witherbloom Lifedrinker payoff) without contributing
/// to opponent damage.
pub fn witherbloom_tincture_maker() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Tincture-Maker",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Crusader ──────────────────────────────────────────────────────

/// Lorehold Crusader — {2}{R}{W}, 3/3 Spirit Soldier with First
/// Strike + Vigilance.
///
/// Aggressive mid-curve Spirit slotting into Quintorius's Spirit
/// tribal anthem. First Strike + Vigilance makes it a strong attacker
/// and a fearless blocker.
pub fn lorehold_crusader() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Crusader",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::FirstStrike, Keyword::Vigilance],
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Initiate ──────────────────────────────────────────────────────

/// Quandrix Initiate — {G}{U}, 1/2 Elf Druid. "Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, put a +1/+1 counter
/// on this creature."
///
/// Self-scaling Magecraft body. Same shape as Cuboid Colony's
/// Increment payoff, but on the magecraft event with no mana-spent
/// gate. Grows quickly in a spell-heavy Quandrix shell.
pub fn quandrix_initiate() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Initiate",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Wand ──────────────────────────────────────────────────────────

/// Lorehold Wand — {2} Artifact. "{2}{R}, {T}: This deals 2 damage
/// to any target."
///
/// Repeatable burn artifact. Tap-gated 4-mana 2-damage ping — not
/// efficient at face value but the artifact body slots into
/// Construct / Karn payoffs in any artifact-heavy shell.
pub fn lorehold_wand() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Wand",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
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
            mana_cost: cost(&[generic(2), r()]),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Bramble ────────────────────────────────────────────────────

/// Witherbloom Bramble — {1}{B}{G} Sorcery. "Create a 1/1 black and
/// green Pest creature token with 'When this creature dies, you gain
/// 1 life.' Then put a +1/+1 counter on each creature you control."
///
/// Wired via `Seq(CreateToken(Pest), ForEach(Creature & ControlledByYou)
/// → AddCounter)`. The Pest mints with its native lifegain death
/// trigger via the `TokenDefinition.triggered_abilities` slot
/// (SOS-VI), and then every creature (including the just-minted Pest)
/// gets a +1/+1 counter — so the Pest enters as a 2/2 lifegain-on-die.
pub fn witherbloom_bramble() -> CardDefinition {
    let pest = super::shared::stx_pest_token();
    CardDefinition {
        name: "Witherbloom Bramble",
        cost: cost(&[generic(1), b(), g()]),
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
                definition: pest,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Prismari Spark ─────────────────────────────────────────────────────────

/// Prismari Spark — {U}{R} Instant. "Prismari Spark deals 2 damage
/// to target creature. Draw a card."
///
/// Standard Prismari cantrip-burn. Same shape as Galvanic Bombardment
/// but at instant speed and gated on creatures only (no PW/player).
pub fn prismari_spark() -> CardDefinition {
    CardDefinition {
        name: "Prismari Spark",
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Trickster ─────────────────────────────────────────────────────

/// Quandrix Trickster — {1}{U}, 2/1 Merfolk Wizard with Flash.
/// "When this creature enters, target creature gets -2/-0 until end
/// of turn."
///
/// Flash combat trick body — same template as Burrog Befuddler in
/// the Quandrix half. Functionally identical, different flavor.
pub fn quandrix_trickster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Trickster",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Memorialist ───────────────────────────────────────────────────

/// Lorehold Memorialist — {R}{W} Sorcery. "Return target creature
/// card from your graveyard to your hand."
///
/// Cheap creature-only reanimation. Slots into Lorehold reanimator
/// shells alongside Brilliant Restoration ({3}{W}{W}, +2 life, faster).
pub fn lorehold_memorialist() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Memorialist",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::HasCardType(CardType::Creature)),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Researcher ─────────────────────────────────────────────────

/// Witherbloom Researcher — {2}{B}{G}, 3/3 Human Druid. "When this
/// creature enters, you gain 2 life and draw a card."
///
/// Classic Witherbloom value — a 3/3 + 2 life + cantrip for 4 mana.
/// Slots into any lifegain shell where the 2 incidental life feeds
/// Honor Troll's flip, Pursuit of Knowledge, etc.
pub fn witherbloom_researcher() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Researcher",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
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
                Effect::GainLife {
                    who: Selector::You,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Catalyst ──────────────────────────────────────────────────────

/// Quandrix Catalyst — {1}{G}{U} Sorcery. "Put two +1/+1 counters on
/// target creature you control, then double the number of +1/+1
/// counters on that creature."
///
/// Wired as `Seq(AddCounter +2, AddCounter CountersOn(target, +1/+1))`
/// — same doubling pattern as Growth Curve ({G}{U}, +1 instead of +2)
/// at one more mana but +2 counters before the doubling, so it nets
/// 4 counters on a vanilla creature instead of 2.
pub fn quandrix_catalyst() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Catalyst",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Vanguard ──────────────────────────────────────────────────────

/// Lorehold Vanguard — {R}{W}, 2/2 Spirit Soldier with Haste.
///
/// Aggressive Lorehold body. Haste means it can immediately deploy
/// for Quintorius's "another Spirit attacks" anthem and Sparring
/// Regimen's per-attacker counter trigger.
pub fn lorehold_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Vanguard",
        cost: cost(&[r(), w()]),
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Inkling Sentinel ───────────────────────────────────────────────────────

/// Inkling Sentinel — {1}{W}, 1/3 Inkling Soldier with Flying and
/// Vigilance.
///
/// Defensive Inkling body — Vigilance + Flying makes it a great
/// blocker that also turns sideways without committing a tap.
/// Tribal synergy with Tenured Inkcaster (3/5 under the +2/+2 anthem).
pub fn inkling_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Inkling Sentinel",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Ritualist ──────────────────────────────────────────────────

/// Witherbloom Ritualist — {2}{B}{G}, 3/3 Human Druid. "{1}{B}{G}:
/// Target creature gets +1/+1 until end of turn. You gain 1 life."
///
/// Repeatable pump activation with a lifegain tail. Slots into
/// Witherbloom lifegain decks (Honor Troll's flip threshold, Pursuit
/// of Knowledge's study counters), and gives Bartlett's Witherbloom
/// Apprentice's drain magecraft a turn-after-turn target.
pub fn witherbloom_ritualist() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Ritualist",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(1), b(), g()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
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
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Quandrix Theorem ───────────────────────────────────────────────────────

/// Quandrix Theorem — {2}{G}{U} Sorcery. "Put a +1/+1 counter on each
/// creature you control."
///
/// Mass +1/+1 counter for the Quandrix board. Wired via
/// `ForEach(EachPermanent(Creature & ControlledByYou)) → AddCounter
/// (TriggerSource, +1/+1)`. Same template as Practical Research's
/// counter fan-out but applied to every creature, not just one
/// target.
pub fn quandrix_theorem() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Theorem",
        cost: cost(&[generic(2), g(), u()]),
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
                kind: CounterType::PlusOnePlusOne,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Prismari Surge ─────────────────────────────────────────────────────────

/// Prismari Surge — {1}{U}{R} Sorcery. "Draw a card. Prismari Surge
/// deals 3 damage to any target."
///
/// Cantrip + 3 damage at sorcery speed. Same shape as the cube
/// classic Frostburn Weird's "draw + burn" but as a single-shot
/// instant.
pub fn prismari_surge() -> CardDefinition {
    CardDefinition {
        name: "Prismari Surge",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        enters_as_copy: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Lorehold Conservator ───────────────────────────────────────────────────

/// Lorehold Conservator — {2}{R}{W}, 3/3 Spirit Cleric with Vigilance.
/// "When this creature enters, exile target card from a graveyard."
///
/// Graveyard hate body with vigilance. Slots into Lorehold reanimator
/// shells as a graveyard-management option in matchups vs Witherbloom
/// reanimator / Prismari spellslinger gy decks.
pub fn lorehold_conservator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Conservator",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Silverquill Initiate ───────────────────────────────────────────────────

/// Silverquill Initiate — {W}, 1/2 Human Cleric. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, you gain
/// 1 life."
///
/// Lifegain Magecraft 1-drop in white. Slots into any lifegain
/// shell — fuels Honor Troll's flip, Light of Promise's "that many"
/// counter rider, etc.
pub fn silverquill_initiate() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Initiate",
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}

// ── Witherbloom Channeler ──────────────────────────────────────────────────

/// Witherbloom Channeler — {2}{B}, 2/3 Human Druid. "{T}: Add {B}
/// or {G}. / {1}, {T}: Each opponent loses 1 life and you gain 1
/// life."
///
/// Dual-ability Witherbloom utility creature — mana ramp + drain
/// activation. The mana ability uses `ManaPayload::AnyOneColor` to
/// approximate "B or G". The drain activation is the standard
/// Witherbloom Apprentice payoff but on a body.
pub fn witherbloom_channeler() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Channeler",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
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
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
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
                ..Default::default()
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
        affinity_graveyard_filter: None,
        equipped_bonus: None,
        soulbond_bonus: None,
        additional_cast_cost: vec![],
        bestow: None,
        foretell_cost: None,
        adventure: None,
        plot_cost: None,
        split: None,
        saga_chapters: vec![],
        miracle: None,
    }
}
