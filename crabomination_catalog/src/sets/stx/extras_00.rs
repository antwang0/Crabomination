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
    Effect, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement,
    Selector, SpellSubtype, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    etb_drain, etb_gain_life, magecraft, magecraft_drain_each_opp, magecraft_self_pump,
    target_filtered,
};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{
    Color, ManaCost, b, colorless, cost, g, generic, hybrid, mono_hybrid, phyrexian, r, u, w, x,
};

// ── Bookwurm ────────────────────────────────────────────────────────────────

/// Bookwurm — {7}{G}, 7/7 Wurm. Real oracle: "Trample / When this
/// creature enters, you gain 3 life and draw a card. / {2}{G}: Put this
/// card from your graveyard into your library third from the top."
///
/// ✅ ETB body is `Seq(GainLife(3), Draw(1))`; the graveyard recursion
/// activation is a `from_graveyard` ability moving the card to
/// `LibraryPosition::FromTop(2)` (third from the top).
pub fn bookwurm() -> CardDefinition {
    use crate::effect::LibraryPosition;
    CardDefinition {
        name: "Bookwurm",
        cost: cost(&[generic(7), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            from_graveyard: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::FromTop(2),
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Field Trip ──────────────────────────────────────────────────────────────

/// Field Trip — {2}{G} Sorcery. "Search your library for a basic Forest
/// card, put that card onto the battlefield tapped, then shuffle. /
/// Learn."
///
/// ✅ Faithful single-search wire via `Effect::Search` for a basic land
/// with the Forest land subtype (entering TAPPED per the printed text),
/// plus Learn via `Effect::Learn`.
pub fn field_trip() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Field Trip",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand
                    .and(SelectionRequirement::HasLandType(LandType::Forest)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            // Learn (CR 701.45) — reveal a Lesson into hand or discard-to-draw.
            Effect::Learn {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

// ── Reduce to Memory ────────────────────────────────────────────────────────

/// Reduce to Memory — Sorcery — Lesson. Real oracle: "Exile target
/// nonland permanent. Its controller creates a 3/2 red and white Spirit
/// creature token."
///
/// ✅ Wired faithfully: `Exile` the targeted nonland permanent, then
/// mint a 3/2 red-and-white Lorehold Spirit token for the *original
/// controller* of the exiled permanent via
/// `PlayerRef::ControllerOf(Target(0))` (mirror of the printed "its
/// controller").
pub fn reduce_to_memory() -> CardDefinition {
    CardDefinition {
        name: "Reduce to Memory",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: crabomination_base::tokens::lorehold_spirit_3_2_token(),
            },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
        },
        alternative_cost: Some(AlternativeCost {
            awaken: false,
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
                        SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                    ),
                },
            ])),
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
            offering: None,
            warp: false,
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Igneous Inspiration ─────────────────────────────────────────────────────

/// Igneous Inspiration — {2}{R} Sorcery. "Igneous Inspiration deals 3
/// damage to any target. / Learn."
///
/// ✅ Wired faithfully: 3 damage to any target (creature, player, or
/// planeswalker), then Learn via `Effect::Learn`.
pub fn igneous_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Igneous Inspiration",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            // Learn (CR 701.45) — reveal a Lesson into hand or discard-to-draw.
            Effect::Learn {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

// ── Combat Professor ────────────────────────────────────────────────────────

/// Combat Professor — {3}{W} Creature — Bird Cleric, 2/3. Real oracle:
/// "Flying / At the beginning of combat on your turn, target creature
/// you control gets +1/+0 and gains vigilance until end of turn."
///
/// ✅ Wired via `StepBegins(BeginCombat) / YourControl` (fires only on
/// the controller's own combat), pumping a target creature you control
/// +1/+0 and granting vigilance until end of turn.
pub fn combat_professor() -> CardDefinition {
    CardDefinition {
        name: "Combat Professor",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
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
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Conspiracy Theorist ─────────────────────────────────────────────────────

// ── Beaming Defiance ────────────────────────────────────────────────────────

/// Beaming Defiance — {1}{W} Instant. Real oracle: "Target creature you
/// control gets +2/+2 and gains hexproof until end of turn."
///
/// ✅ Wired as `PumpPT(+2/+2)` + `GrantKeyword(Hexproof, EOT)`.
pub fn beaming_defiance() -> CardDefinition {
    CardDefinition {
        name: "Beaming Defiance",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
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
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Spell Satchel ───────────────────────────────────────────────────────────

/// Spell Satchel — Artifact. Real oracle: "Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, put a book counter on this
/// artifact. / {T}, Remove a book counter from this artifact: Add {C}. /
/// {3}, {T}, Remove three book counters from this artifact: Draw a card."
///
/// ✅ Magecraft rides the shared `magecraft` shortcut, adding a
/// `CounterType::Book` counter to the source. Both activations pay
/// their book counters as a real cost via `remove_counter_cost`.
pub fn spell_satchel() -> CardDefinition {
    CardDefinition {
        name: "Spell Satchel",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Book,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Book, 1)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                remove_counter_cost: Some((CounterType::Book, 3)),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Squirrel Sanctuary (stand-in placeholder dropped) ───────────────────────

// ── Excavated Wall ──────────────────────────────────────────────────────────

/// Excavated Wall — Artifact Creature — Wall, 0/4. Real oracle:
/// "Defender / {1}, {T}: Mill a card."
///
/// ✅ Defender body with a `{1}, {T}` self-mill activation via
/// `Effect::Mill`.
pub fn excavated_wall() -> CardDefinition {
    CardDefinition {
        name: "Excavated Wall",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Mill {
                who: Selector::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — Instant. Real oracle: "Tap up to two target creatures.
/// Those creatures don't untap during their controller's next untap
/// step. / Draw two cards, then discard a card."
///
/// ✅ Two-slot spell: slot 0 is the first creature, slot 1 (passed via
/// `GameAction::CastSpell.additional_targets[0]`) is the second. "Up to
/// two" semantics fall out naturally — with only one target the slot-1
/// tap/skip pair no-ops. The freeze rider rides `Effect::SkipNextUntap`
/// (the real "doesn't untap during its controller's next untap step"
/// flag, not a stun counter), then the caster draws two and discards one.
pub fn snow_day() -> CardDefinition {
    CardDefinition {
        name: "Snow Day",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            // Slot 0: tap + freeze the first creature.
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::SkipNextUntap {
                what: Selector::Target(0),
            },
            // Slot 1: tap + freeze the second creature (optional — resolves
            // to no-op when only one target was chosen).
            Effect::Tap {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
            },
            Effect::SkipNextUntap {
                what: Selector::Target(1),
            },
            // Draw two cards, then discard a card.
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
        ..Default::default()
    }
}

// ── (helper `local_pest_token` removed in push XX — `super::shared::stx_pest_token`
//     is the canonical Pest factory used everywhere a Pest is minted.)

// ── Curate ──────────────────────────────────────────────────────────────────

/// Curate — {1}{U} Instant. Real oracle: "Surveil 2. / Draw a card."
///
/// ✅ Straight `Effect::Surveil(2)` followed by a draw.
pub fn curate() -> CardDefinition {
    CardDefinition {
        name: "Curate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

// ── Strategic Planning (already defined in `decks::modern`) ────────────────
//
// Strategic Planning is wired in
// `catalog::sets::decks::modern::strategic_planning` as the faithful
// `Effect::LookPickToHand { count: 3, rest_to_graveyard: true, rest_to_exile: false }` — look at
// the top three, put your pick into your hand, rest to the graveyard.

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
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}

// ── Resculpt ───────────────────────────────────────────────────────────────

/// Resculpt — {1}{U} Instant. Real oracle: "Exile target artifact or
/// creature. Its controller creates a 4/4 blue and red Elemental
/// creature token."
///
/// ✅ Wired faithfully: `Exile` the target, then mint a 4/4 blue-and-red
/// Elemental token under the *original controller* of the exiled
/// permanent (`PlayerRef::ControllerOf(Target(0))`).
pub fn resculpt() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],

        static_abilities: vec![],
        ..Default::default()
    };
    CardDefinition {
        name: "Resculpt",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

// ── Mortality Spear ────────────────────────────────────────────────────────

/// Mortality Spear — Instant. Real oracle: "This spell costs {2} less to
/// cast if you gained life this turn. / Destroy target nonland
/// permanent."
///
/// ✅ The lifegain discount is a `SelfCostReducedIf` static gated on
/// `Predicate::PlayerGainedLifeThisTurn`; the removal hits any nonland
/// permanent.
pub fn mortality_spear() -> CardDefinition {
    CardDefinition {
        name: "Mortality Spear",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if you gained life this turn.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::PlayerGainedLifeThisTurn {
                    who: PlayerRef::You,
                },
                amount: 2,
            },
        }],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
            ),
        },
        ..Default::default()
    }
}

// ── Daemogoth Titan ────────────────────────────────────────────────────────

/// Daemogoth Titan — 11/10 Demon. Real oracle: "Whenever this creature
/// attacks or blocks, sacrifice a creature."
///
/// ✅ Both halves wired: `EventKind::Attacks/SelfSource` and
/// `EventKind::Blocks/SelfSource` (CR 509.1i). The sacrifice is "a
/// creature" (any creature you control, the Titan itself included) via
/// `Effect::Sacrifice`; the auto-decider prefers lowest-power non-source
/// creatures, so a fresh Titan will sac something else when possible.
pub fn daemogoth_titan() -> CardDefinition {
    let sac_another = Effect::Sacrifice {
        who: Selector::You,
        count: Value::Const(1),
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Daemogoth Titan",
        cost: cost(&[
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
            hybrid(Color::Black, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 11,
        toughness: 10,
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
        ..Default::default()
    }
}

// ── Daemogoth Woe-Eater ────────────────────────────────────────────────────

/// Daemogoth Woe-Eater — 7/6 Demon. Real oracle: "At the beginning of
/// your upkeep, sacrifice a creature. / When you sacrifice this
/// creature, each opponent discards a card, you draw a card, and you
/// gain 2 life."
///
/// ✅ The upkeep tithe rides `StepBegins(Upkeep)/YourControl`; the
/// sacrifice payoff rides `EventKind::CreatureSacrificed/SelfSource`
/// (CR 701.16 — sacrifice is its own event, so death by combat or
/// removal does NOT fire the payoff).
pub fn daemogoth_woe_eater() -> CardDefinition {
    CardDefinition {
        name: "Daemogoth Woe-Eater",
        cost: cost(&[generic(1), b(), hybrid(Color::Black, Color::Green), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(2),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

// ── Honor Troll ────────────────────────────────────────────────────────────

/// Honor Troll — {2}{G} 2/3 Troll Druid with vigilance. "If you would gain
/// life, you gain that much life plus 1 instead. This creature gets +2/+1 as
/// long as you have 25 or more life."
pub fn honor_troll() -> CardDefinition {
    use crate::effect::PlayerStaticTarget;
    CardDefinition {
        name: "Honor Troll",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "If you would gain life, you gain that much plus 1 instead.",
                effect: StaticEffect::LifeGainBonus {
                    target: PlayerStaticTarget::Controller,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "Gets +2/+1 as long as you have 25 or more life.",
                effect: StaticEffect::PumpSelfIf {
                    condition: Predicate::ValueAtLeast(
                        Value::LifeOf(PlayerRef::You),
                        Value::Const(25),
                    ),
                    power: 2,
                    toughness: 1,
                    keywords: vec![],
                },
            },
        ],
        ..Default::default()
    }
}

// ── Quandrix Cultivator ────────────────────────────────────────────────────

/// Quandrix Cultivator — 3/4 Turtle Druid. Real oracle: "When this
/// creature enters, you may search your library for a basic Forest or
/// Island card, put it onto the battlefield, then shuffle."
///
/// ✅ The printed "may" is a real `MayDo` wrap around the search (the
/// controller can decline); the fetched basic enters untapped as
/// printed.
pub fn quandrix_cultivator() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Quandrix Cultivator",
        cost: cost(&[generic(1), g(), hybrid(Color::Green, Color::Blue), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Quandrix Cultivator: search your library for a basic \
                              Forest or Island card and put it onto the battlefield?"
                    .into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand.and(
                        SelectionRequirement::HasLandType(LandType::Forest)
                            .or(SelectionRequirement::HasLandType(LandType::Island)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Hofri Ghostforge ───────────────────────────────────────────────────────

/// Hofri Ghostforge — {3}{R}{W}, 4/5 Legendary Dwarf Cleric. "Spirits you
/// control get +1/+1 and have trample and haste. / Whenever another nontoken
/// creature you control dies, exile it. If you do, create a token that's a
/// copy of that creature, except it's a Spirit in addition to its other
/// types."
pub fn hofri_ghostforge() -> CardDefinition {
    use crate::card::{
        EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement, StaticAbility,
        TriggeredAbility,
    };
    use crate::effect::{PlayerRef, Selector, StaticEffect, Value, ZoneDest};
    let spirits = || {
        Selector::EachPermanent(
            SelectionRequirement::HasCreatureType(CreatureType::Spirit)
                .and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Hofri Ghostforge",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        // Exile the dying creature, then mint a Spirit-typed token copy of it.
        // `CreateTokenCopyOf` resolves the source from exile, so it sees the
        // just-exiled card.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Not(Box::new(SelectionRequirement::IsToken)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Exile,
                },
                Effect::CreateTokenCopyOf {
                    extra_keywords: vec![],
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![CreatureType::Spirit],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                },
                // "When that token leaves the battlefield, return the exiled
                // card to its owner's graveyard."
                Effect::WhenLastCreatedTokenLeaves {
                    body: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Graveyard,
                    }),
                },
            ]),
        }],
        static_abilities: vec![
            StaticAbility {
                description: "Spirits you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: spirits(),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Spirits you control have trample.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: spirits(),
                    keyword: Keyword::Trample,
                },
            },
            StaticAbility {
                description: "Spirits you control have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: spirits(),
                    keyword: Keyword::Haste,
                },
            },
        ],
        ..Default::default()
    }
}

// ── Tempted by the Oriq ────────────────────────────────────────────────────

/// Tempted by the Oriq — Sorcery. Real oracle: "For each opponent, gain
/// control of up to one target creature or planeswalker that player
/// controls with mana value 3 or less."
///
/// ✅ Permanent `GainControl` of *up to one* opponent-controlled
/// creature or planeswalker with MV ≤ 3, via `ApplyToTargets {
/// max_targets: 1, min_targets: 0 }` (the printed "up to one" — the
/// caster may cast it targetless). The printed text is per-opponent;
/// with the engine's targeting this grabs one such permanent, which is
/// exact in 1v1.
pub fn tempted_by_the_oriq() -> CardDefinition {
    CardDefinition {
        name: "Tempted by the Oriq",
        cost: cost(&[generic(1), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        // Permanently gain control of up to one creature/PW (MV 3 or less)
        // an opponent controls. Printed text is per-opponent; with the
        // engine's targeting this grabs one such permanent (exact in 1v1).
        effect: Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: SelectionRequirement::ControlledByOpponent
                .and(SelectionRequirement::ManaValueAtMost(3))
                .and(SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker)),
            effect: Box::new(Effect::GainControl {
                what: Selector::Target(0),
                to: None,
                duration: Duration::Permanent,
            }),
        },
        ..Default::default()
    }
}

/// Confront the Past — {X}{B} Sorcery — Lesson. Choose one — return target
/// planeswalker card with mana value X or less from your graveyard to the
/// battlefield; or remove twice X loyalty counters from target planeswalker
/// an opponent controls.
pub fn confront_the_past() -> CardDefinition {
    CardDefinition {
        name: "Confront the Past",
        cost: cost(&[x(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Planeswalker
                        .and(SelectionRequirement::InGraveyard)
                        .and(SelectionRequirement::ManaValueAtMostXFromCost),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::RemoveCounter {
                what: target_filtered(
                    SelectionRequirement::Planeswalker
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                kind: CounterType::Loyalty,
                amount: Value::Times(Box::new(Value::Const(2)), Box::new(Value::XFromCost)),
            },
        ]),
        ..Default::default()
    }
}

/// Specter of the Fens — {3}{B} 2/3 Specter with flying. `{5}{B}: Target
/// opponent loses 2 life and you gain 2 life.`
pub fn specter_of_the_fens() -> CardDefinition {
    CardDefinition {
        name: "Specter of the Fens",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            effect: crate::effect::shortcut::drain(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mascot Interception — {3}{R} Sorcery. Real oracle: "This spell costs
/// {3} less to cast if it targets a creature token. / Gain control of
/// target creature until end of turn. Untap that creature. It gets
/// +2/+0 and gains haste until end of turn."
///
/// ✅ Fully faithful: the creature-token discount is a mandatory CR
/// 601.2f generic reduction via `self_cost_reduction_if_target:
/// (IsToken ∧ Creature, 3)` (same primitive as Ride's End), evaluated
/// against the chosen target at cast time.
pub fn mascot_interception() -> CardDefinition {
    CardDefinition {
        name: "Mascot Interception",
        cost: cost(&[generic(3), r()]),
        self_cost_reduction_if_target: Some((
            SelectionRequirement::IsToken.and(SelectionRequirement::Creature),
            3,
        )),
        card_types: vec![CardType::Sorcery],
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
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Twinscroll Shaman — {2}{R} 1/2 Dwarf Shaman with double strike.
pub fn twinscroll_shaman() -> CardDefinition {
    CardDefinition {
        name: "Twinscroll Shaman",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::DoubleStrike],
        ..Default::default()
    }
}

/// Practical Research — {3}{U}{R} Instant. "Draw four cards. Then discard two
/// cards unless you discard an instant or sorcery card."
///
/// ✅ Fully faithful: the discard rider uses `Effect::DiscardUnlessKind`
/// (the Wrench Mind primitive) — with an instant or sorcery card in hand
/// the discarder pitches that single card (lowest-MV match) instead of
/// two; otherwise the full two-card discard applies.
pub fn practical_research() -> CardDefinition {
    CardDefinition {
        name: "Practical Research",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::DiscardUnlessKind {
                who: PlayerRef::You,
                count: Value::Const(2),
                instead: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
        ]),
        ..Default::default()
    }
}

/// Hall of Oracles — Land. Real oracle: "{T}: Add {C}. / {1}, {T}: Add
/// one mana of any color. / {T}: Put a +1/+1 counter on target creature.
/// Activate only as a sorcery and only if you've cast an instant or
/// sorcery spell this turn."
///
/// ✅ Three activations: the shared `tap_add_colorless` helper, a
/// `{1}, {T}` any-color filter ability, and a sorcery-speed `{T}`
/// counter ability gated on
/// `Predicate::InstantsOrSorceriesCastThisTurnAtLeast(You, 1)`.
pub fn hall_of_oracles() -> CardDefinition {
    CardDefinition {
        name: "Hall of Oracles",
        cost: cost(&[]),
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                condition: Some(Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                }),
                effect: Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Star Pupil — {W} Creature — Human Wizard, 0/0. Real oracle: "This
/// creature enters with a +1/+1 counter on it. / When this creature
/// dies, put its counters on target creature you control."
///
/// ✅ Both halves wired: `enters_with_counters` for the ETB counter,
/// and a death trigger that moves the source's +1/+1 counter total onto
/// a target creature you control via cross-zone `Value::CountersOn`
/// (counters persist on the card in the graveyard, so the count reads
/// correctly post-death).
pub fn star_pupil() -> CardDefinition {
    CardDefinition {
        name: "Star Pupil",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        // Enters with a +1/+1 counter (→ 1/1).
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(1))),
        // Dies: put its +1/+1 counters on target creature you control.
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        }],
        ..Default::default()
    }
}

/// Ageless Guardian — {1}{W} 1/4 Spirit Soldier (vanilla).
pub fn ageless_guardian() -> CardDefinition {
    CardDefinition {
        name: "Ageless Guardian",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        ..Default::default()
    }
}

/// Letter of Acceptance — {3} Artifact. `{T}: Add one mana of any color.`
/// `{2}, {T}, Sacrifice this artifact: Draw a card.`
pub fn letter_of_acceptance() -> CardDefinition {
    CardDefinition {
        name: "Letter of Acceptance",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Charge Through — {G} Instant. "Target creature gains trample until end of
/// turn. Draw a card."
pub fn charge_through() -> CardDefinition {
    CardDefinition {
        name: "Charge Through",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
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
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            Effect::ExileAnyNumberFromGraveyards {
                filter: SelectionRequirement::Any,
            },
        ]),
        ..Default::default()
    }
}

/// Manifestation Sage — {G/U}{G/U}{G/U}{G/U} 2/2 Human Wizard. "When this
/// creature enters, create a 0/0 green and blue Fractal creature token. Put X
/// +1/+1 counters on it, where X is the number of cards in your hand."
pub fn manifestation_sage() -> CardDefinition {
    let gu = || hybrid(Color::Green, Color::Blue);
    CardDefinition {
        name: "Manifestation Sage",
        cost: cost(&[gu(), gu(), gu(), gu()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::catalog::sets::sos::fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::HandSizeOf(PlayerRef::You),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Crackle with Power — {X}{R}{R}{R}{R}{R} Sorcery. Real oracle:
/// "Crackle with Power deals five times X damage to each of up to X
/// targets."
///
/// ✅ Each supplied target takes the FULL 5X (not divided): wired via
/// `ApplyToTargets` running `DealDamage(5·X)` once per target. Residue:
/// "Up to X targets" is exact via `CapTargetsAtX` (resolution drops
/// slots beyond the paid X); the inner `max_targets: 5` is only the
/// static slot ceiling for cast-time enumeration.
pub fn crackle_with_power() -> CardDefinition {
    CardDefinition {
        name: "Crackle with Power",
        cost: cost(&[x(), x(), x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        // "UP TO X targets": CapTargetsAtX makes X the true cap; the
        // inner max_targets 5 is only the static slot ceiling.
        effect: Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 5,
                min_targets: 0,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
                effect: Box::new(Effect::DealDamage {
                    to: Selector::Target(0),
                    amount: Value::Times(Box::new(Value::Const(5)), Box::new(Value::XFromCost)),
                }),
            }),
        },
        ..Default::default()
    }
}

/// Mentor's Guidance — {2}{U} Sorcery. Real oracle: "When you cast this
/// spell, copy it if you control a planeswalker, Cleric, Druid, Shaman,
/// Warlock, or Wizard. / Scry 1, then draw a card."
///
/// ✅ Body is `Scry 1` + `Draw 1`; the cast trigger copies the spell
/// when the class-tribal condition holds (`on_cast` + `CopySpell`).
pub fn mentors_guidance() -> CardDefinition {
    use crate::card::{CreatureType, Predicate};
    use crate::effect::shortcut::on_cast;
    CardDefinition {
        name: "Mentor's Guidance",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        // Scry 1, then draw a card.
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        // "When you cast this spell, copy it if you control a planeswalker,
        // Cleric, Druid, Shaman, Warlock, or Wizard."
        triggered_abilities: vec![on_cast(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                SelectionRequirement::ControlledByYou.and(
                    SelectionRequirement::Planeswalker
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Cleric))
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Druid))
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Shaman))
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Warlock))
                        .or(SelectionRequirement::HasCreatureType(CreatureType::Wizard)),
                ),
            )),
            then: Box::new(Effect::CopySpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                count: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Quintorius, Field Historian — {3}{R}{W} Legendary 2/4 Elephant Cleric.
/// "Spirits you control get +1/+0. Whenever one or more cards leave your
/// graveyard, create a 3/2 red and white Spirit creature token." (The
/// "one or more" batch collapses to a per-card trigger.)
pub fn quintorius_field_historian() -> CardDefinition {
    use crate::card::{StaticAbility, Supertype};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Quintorius, Field Historian",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::lorehold_spirit_3_2_token(),
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Spirits you control get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
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
/// one copy above it. The Magecraft self-exile rider is wired via
/// `exile_on_resolve: true` — casting Galvanic Iteration is itself
/// casting an instant, so its own Magecraft always fires and the card
/// routes to exile instead of the graveyard when it resolves,
/// sequencing after the stack-top copy exactly like the printed rider.
/// Sole residue: if Iteration is countered it lands in the graveyard,
/// and a later IS cast won't exile it from there (the engine has no
/// graveyard-watching self-exile trigger) — a corner with no gameplay
/// read in this catalog.
pub fn galvanic_iteration() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Iteration",
        cost: cost(&[u(), r()]),
        exile_on_resolve: true,
        card_types: vec![CardType::Instant],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            ),
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

// ── Expressive Iteration ────────────────────────────────────────────────────

/// Expressive Iteration — {U}{R} Sorcery. Real oracle: "Look at the top
/// three cards of your library. Put one of them into your hand, put one
/// of them on the bottom of your library, and exile one of them. You
/// may play the exiled card this turn."
///
/// 🟡 APPROXIMATION — the engine has no "look at top N and distribute
/// one to hand / one to bottom / one to exile-with-may-play" primitive
/// (a three-way `LookTopDistribute` is the exact missing piece;
/// `LookPickToHand` can do hand+bottom but can't route a second pick to
/// exile-with-play-permission from the same looked-at set). Current
/// wiring: `LookPickToHand(3)` — pick one of the top three to hand,
/// bottom the rest — then `ExileTopAndGrantMayPlay(1, pay_own_cost)`
/// exiles the (new) top card playable this turn. Card economy matches
/// the printed line exactly (+1 hand, +1 playable exile, rest
/// bottomed); the only drift is that the exiled card comes from the
/// post-bottoming top instead of the original three.
pub fn expressive_iteration() -> CardDefinition {
    CardDefinition {
        name: "Expressive Iteration",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::LookPickToHand {
                then_if_picked: None,
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
            Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::Const(1),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
            // "...and the rest on the bottom of your library" — bottom the
            // last leftover instead of leaving it on top.
            Effect::Move {
                what: Selector::TopOfLibrary {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                },
                to: crate::effect::ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Bottom,
                },
            },
        ]),
        ..Default::default()
    }
}

// ── Magma Opus ──────────────────────────────────────────────────────────────

/// Magma Opus — {6}{U}{R} Instant. Real oracle: "Magma Opus deals 4
/// damage divided as you choose among any number of targets. Tap two
/// target permanents. Create a 4/4 blue and red Elemental creature
/// token. Draw two cards. / {U/R}{U/R}, Discard this card: Create a
/// Treasure token."
///
/// ✅ The main `Seq` ships all four printed primary clauses: 4 damage
/// divided (`DealDamageDivided`, "any target" so players are legal)
/// among up to four targets, tap two permanents, a 4/4 blue-and-red
/// Elemental token, and draw 2. Residue: "Tap two target permanents"
/// rides `TapUpToValue { exact: true }` — the two permanents are chosen
/// at resolution (`Decision::ChooseCards`) rather than declared as cast
/// targets, because `DealDamageDivided` owns the leading target-slot
/// range and the engine's positional slot model can't append fixed
/// slots after a variable-count divided block. The {U/R}{U/R}, Discard
/// → Treasure mode ships via `discard_activated`.
pub fn magma_opus() -> CardDefinition {
    use crate::mana::{Color, hybrid};
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        ..Default::default()
    };
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        name: "Magma Opus",
        cost: cost(&[generic(6), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamageDivided {
                retaliate_to_source: false,
                total: Value::Const(4),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
                max_targets: 4,
            },
            Effect::TapUpToValue {
                count: Value::Const(2),
                filter: SelectionRequirement::Permanent,
                skip_untap: false,
                exact: true,
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
        discard_activated: Some(Box::new(crate::card::DiscardActivated {
            cost: cost(&[ur(), ur()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        })),
        ..Default::default()
    }
}

// ── Reckless Amplimancer ────────────────────────────────────────────────────

/// Reckless Amplimancer — {1}{G} 2/2 Elf Druid. `{4}{G}: Double this
/// creature's power and toughness until end of turn.`
pub fn reckless_amplimancer() -> CardDefinition {
    CardDefinition {
        name: "Reckless Amplimancer",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        // Doubling = add the creature's current P/T as an EOT pump.
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::ToughnessOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Crashing Drawbridge ─────────────────────────────────────────────────────

/// Crashing Drawbridge — {2} Artifact Creature — Construct, 0/4.
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
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── First Day of Class ──────────────────────────────────────────────────────

/// First Day of Class — {1}{R} Instant. "Whenever a creature you control
/// enters this turn, put a +1/+1 counter on it and it gains haste until
/// end of turn. Learn." The turn-scoped enters trigger rides
/// `Effect::CreaturesYouControlEnteringThisTurn` (CR 603.4).
pub fn first_day_of_class() -> CardDefinition {
    CardDefinition {
        name: "First Day of Class",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CreaturesYouControlEnteringThisTurn {
                body: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
            Effect::Learn {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

// ── Verdant Mastery ─────────────────────────────────────────────────────────

/// Verdant Mastery — {5}{G} Sorcery. Real oracle: "You may pay {3}{G}
/// rather than pay this spell's mana cost. / Search your library for up
/// to four basic land cards and reveal them. Put one of them onto the
/// battlefield tapped under an opponent's control if the {3}{G} cost
/// was paid. Put two of them onto the battlefield tapped under your
/// control and the rest into your hand. Then shuffle."
///
/// ✅ Base cast: four sequential basic-land searches — two to your
/// battlefield tapped, two to hand (each search individually
/// declinable, giving the printed "up to four"). The {3}{G} alt cost
/// rides `AlternativeCost.effect_override`: one basic goes onto the
/// battlefield tapped under an opponent's control first, then two
/// tapped under yours and the rest (one) to hand.
pub fn verdant_mastery() -> CardDefinition {
    use crate::card::AlternativeCost;
    let basic = || SelectionRequirement::IsBasicLand;
    let to_your_bf = || ZoneDest::Battlefield {
        controller: PlayerRef::You,
        tapped: true,
    };
    let to_hand = || ZoneDest::Hand(PlayerRef::You);
    // Base: put two basics onto the battlefield tapped under your control and
    // the rest (up to two) into your hand.
    let base = || {
        Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: basic(),
                to: to_your_bf(),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: basic(),
                to: to_your_bf(),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: basic(),
                to: to_hand(),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: basic(),
                to: to_hand(),
            },
        ])
    };
    // Alt ({3}{G} paid): one basic goes onto the battlefield tapped under an
    // opponent's control, two under yours, the rest into your hand.
    let alt = Effect::Seq(vec![
        Effect::Search {
            who: PlayerRef::You,
            filter: basic(),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::EachOpponent,
                tapped: true,
            },
        },
        Effect::Search {
            who: PlayerRef::You,
            filter: basic(),
            to: to_your_bf(),
        },
        Effect::Search {
            who: PlayerRef::You,
            filter: basic(),
            to: to_your_bf(),
        },
        Effect::Search {
            who: PlayerRef::You,
            filter: basic(),
            to: to_hand(),
        },
    ]);
    CardDefinition {
        name: "Verdant Mastery",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: base(),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(3), g()]),
            effect_override: Some(alt),
            ..Default::default()
        }),
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

// ── Sparkmage Apprentice ────────────────────────────────────────────────────

/// Sparkmage Apprentice — {1}{R} Creature — Human Wizard, 1/1.
/// "When this creature enters, it deals 2 damage to any target."
///
/// Pinpoint Prismari ETB removal. Wired with a standard
/// `EntersBattlefield / SelfSource` trigger and a creature-or-player-
/// or-planeswalker target picker.
pub fn sparkmage_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Sparkmage Apprentice",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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
        ..Default::default()
    }
}

// ── Karok Wrangler ──────────────────────────────────────────────────────────

/// Karok Wrangler — {4}{G} 3/3 Elf Druid. "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on target creature
/// you control."
pub fn karok_wrangler() -> CardDefinition {
    CardDefinition {
        name: "Karok Wrangler",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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

// ── Witherbloom Command ─────────────────────────────────────────────────────

// ── Lorehold Command ────────────────────────────────────────────────────────

// ── Quandrix Command ────────────────────────────────────────────────────────

// ── Silverquill Command ─────────────────────────────────────────────────────

// ── Prismari Command ────────────────────────────────────────────────────────

// ── Defend the Campus ───────────────────────────────────────────────────────

/// Defend the Campus — {3}{W} Instant. "Choose one — Creatures you control get
/// +1/+1 until end of turn; or Destroy target creature with power 4 or
/// greater." (AutoDecider keeps mode 0.)
pub fn defend_the_campus() -> CardDefinition {
    CardDefinition {
        name: "Defend the Campus",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                Effect::PumpPT {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4)),
                    ),
                },
            ],
        },
        ..Default::default()
    }
}

// ── Hall Monitor ────────────────────────────────────────────────────────────

/// Hall Monitor — {R} 1/1 Lizard Shaman with haste. `{1}{R}, {T}: Target
/// creature can't block this turn.`
pub fn hall_monitor() -> CardDefinition {
    CardDefinition {
        name: "Hall Monitor",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Stonebinder's Familiar ──────────────────────────────────────────────────

/// Stonebinder's Familiar — {W} 1/1 Spirit Dog. Whenever one or more cards are
/// put into exile during your turn, put a +1/+1 counter on this creature. This
/// ability triggers only once each turn.
pub fn stonebinders_familiar() -> CardDefinition {
    CardDefinition {
        name: "Stonebinder's Familiar",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dog],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardExiled, EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::You))
                .once_per_turn(),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

// ── Necrotic Fumes ──────────────────────────────────────────────────────────

/// Necrotic Fumes — {1}{B}{B} Sorcery — Lesson. "As an additional cost,
/// exile a creature you control. Exile target creature or planeswalker."
pub fn necrotic_fumes() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Fumes",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        additional_cast_cost: vec![AdditionalCastCost::ExilePermanent {
            filter: SelectionRequirement::Creature,
            count: 1,
        }],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            to: ZoneDest::Exile,
        },
        ..Default::default()
    }
}

// ── Make Your Mark ──────────────────────────────────────────────────────────

/// Make Your Mark — {R/W} Instant. "Target creature gets +1/+0 until end of
/// turn. When that creature dies this turn, create a 3/2 red and white Spirit
/// creature token."
pub fn make_your_mark() -> CardDefinition {
    CardDefinition {
        name: "Make Your Mark",
        cost: cost(&[hybrid(Color::Red, Color::White)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                filter: None,
                slot: 0,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::lorehold_spirit_3_2_token(),
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Containment Breach ──────────────────────────────────────────────────────

/// Containment Breach — {2}{G} Sorcery — Lesson. "Destroy target artifact or
/// enchantment. If its mana value is 2 or less, create a 1/1 black and green
/// Pest token with 'When this token dies, you gain 1 life.'"
pub fn containment_breach() -> CardDefinition {
    CardDefinition {
        name: "Containment Breach",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::If {
            // Check the target's mana value before it's destroyed.
            cond: Predicate::ValueAtLeast(
                Value::Const(2),
                Value::ManaValueOf(Box::new(Selector::Target(0))),
            ),
            then: Box::new(Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                    ),
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crabomination_base::tokens::stx_pest_token(),
                },
            ])),
            else_: Box::new(Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            }),
        },
        ..Default::default()
    }
}

// ── Burrog Befuddler ────────────────────────────────────────────────────────

/// Burrog Befuddler — {1}{U} 2/1 Frog Wizard with flash. "When this creature
/// enters, target creature an opponent controls gets -1/-0 until end of turn."
pub fn burrog_befuddler() -> CardDefinition {
    CardDefinition {
        name: "Burrog Befuddler",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

// ── Mage Duel ───────────────────────────────────────────────────────────────

/// Mage Duel — {2}{G} Sorcery. "This spell costs {2} less to cast if
/// you've cast another instant or sorcery spell this turn. / Target
/// creature you control gets +1/+2 until end of turn, then it fights
/// target creature you don't control."
///
/// ✅ Two-slot spell: slot 0 is the friendly creature (pumped, then the
/// fight's attacker), slot 1 (`additional_targets[0]`) is the opponent's
/// victim. The "another instant/sorcery this turn" discount is wired via
/// `StaticEffect::SelfCostReducedIf` gated on
/// `Predicate::InstantsOrSorceriesCastThisTurnAtLeast(You, 1)` — at cost
/// determination (CR 601.2f) Mage Duel itself hasn't been counted yet,
/// so "at least 1" is exactly the printed "another".
pub fn mage_duel() -> CardDefinition {
    CardDefinition {
        name: "Mage Duel",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {2} less to cast if you've cast \
                          another instant or sorcery spell this turn.",
            effect: StaticEffect::SelfCostReducedIf {
                condition: Predicate::InstantsOrSorceriesCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                amount: 2,
            },
        }],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

// ── Eccentric Apprentice ────────────────────────────────────────────────────

/// Eccentric Apprentice — {2}{U} Creature — Human Wizard, 2/2.
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
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Tiefling, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
    }
}

// ── Tezzeret's Gambit ───────────────────────────────────────────────────────

/// Tezzeret's Gambit — {3}{U/P} Sorcery.
/// "Draw two cards, then proliferate."
///
/// The single `{U/P}` Phyrexian pip is a real `ManaSymbol::Phyrexian` —
/// `ManaCost::pay()` pays it with blue mana if available, else 2 life,
/// so the card can be cast for {3}{U} or {3} + 2 life.
///
/// Non-modal: draw 2, then `Effect::Proliferate` (every permanent and
/// player with a counter gets one more of any kind they already have,
/// controller chooses per object).
pub fn tezzerets_gambit() -> CardDefinition {
    CardDefinition {
        name: "Tezzeret's Gambit",
        cost: cost(&[generic(3), crate::mana::phyrexian(Color::Blue)]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::Proliferate,
        ]),
        ..Default::default()
    }
}

// ── Wandering Archaic ───────────────────────────────────────────────────────

/// Wandering Archaic // Explore the Vastlands — {5} Creature — Spirit, 4/4.
/// Modal double-faced; the back face `Explore the Vastlands` ({4} Sorcery —
/// add six colorless mana, gain 3 life) is now wired via `back_face` and is
/// castable from hand through `GameAction::CastSpellBack`.
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
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::Any(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCardType(CardType::Instant),
                    },
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCardType(CardType::Sorcery),
                    },
                ]),
            ),
            effect: Effect::CopySpellUnlessPaid {
                what: Selector::TriggerSource,
                mana_cost: cost(&[generic(2)]),
                count: Value::Const(1),
            },
        }],
        // Back face: Explore the Vastlands — {3} Sorcery. "Each player looks
        // at the top five cards of their library and may reveal a land card
        // and/or an instant or sorcery card from among them. Each player puts
        // the revealed cards into their hand and the rest on the bottom of
        // their library in a random order. Each player gains 3 life."
        // Per-player `ForEach` + `LookPickToHand` (take up to 2, filtered to
        // land/instant/sorcery, rest to bottom random). Approximation: the
        // "one land AND/OR one instant/sorcery" category split is collapsed
        // to "up to two matching cards" — a player could take two lands.
        back_face: Some(Box::new(CardDefinition {
            name: "Explore the Vastlands",
            cost: cost(&[generic(3)]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachPlayer),
                body: Box::new(Effect::Seq(vec![
                    Effect::LookPickToHand {
                        then_if_picked: None,
                        who: PlayerRef::Triggerer,
                        count: Value::Const(5),
                        rest_to_graveyard: false,
                        pick_filter: Some(
                            SelectionRequirement::HasCardType(CardType::Land)
                                .or(SelectionRequirement::HasCardType(CardType::Instant))
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        ),
                        take: Some(Value::Const(2)),
                        to_battlefield: false,
                        gain_life_if_pick: None,
                        gain_life_greatest_power_rest: false,
                        optional: true,
                        picked_lands_to_battlefield: false,
                        rest_bottom_random: true,
                        rest_to_exile: false,
                    },
                    Effect::GainLife {
                        who: Selector::Player(PlayerRef::Triggerer),
                        amount: Value::Const(3),
                    },
                ])),
            },
            ..Default::default()
        })),
        ..Default::default()
    }
}

// ── Draconic Intervention ───────────────────────────────────────────────────

/// Draconic Intervention — {2}{R}{R} Sorcery. "As an additional cost to cast
/// this spell, exile an instant or sorcery card from your graveyard. Draconic
/// Intervention deals X damage to each non-Dragon creature, where X is the
/// exiled card's mana value. If a creature dealt damage this way would die
/// this turn, exile it instead. Exile Draconic Intervention." The exile-instead
/// rider rides `ExileIfWouldDieThisTurn` (installed before the damage so a
/// lethal hit is redirected), X = the exiled card's MV via
/// `AdditionalCastCost::ExileFromGraveyard`.
pub fn draconic_intervention() -> CardDefinition {
    let non_dragon = || {
        SelectionRequirement::Creature
            .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon).negate())
    };
    CardDefinition {
        name: "Draconic Intervention",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::ExileFromGraveyard {
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn {
                what: Selector::EachPermanent(non_dragon()),
            },
            Effect::DealDamage {
                to: Selector::EachPermanent(non_dragon()),
                amount: Value::XFromCost,
            },
        ]),
        exile_on_resolve: true,
        ..Default::default()
    }
}

// ── Fervent Mastery ─────────────────────────────────────────────────────────

/// Fervent Mastery — {3}{R}{R} Sorcery. "You may pay {2}{R}{R} rather than
/// pay this spell's mana cost. If the {2}{R}{R} cost was paid, an opponent
/// discards any number of cards, then draws that many cards. Search your
/// library for up to three cards, put them into your hand, shuffle, then
/// discard three cards at random." The alt-cost rider rides
/// `AlternativeCost.effect_override` (the cheaper cast runs the extra opponent
/// loot first). "Up to three" is three sequential library searches.
pub fn fervent_mastery() -> CardDefinition {
    use crate::card::AlternativeCost;
    let base = || {
        vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(3),
                random: true,
            },
        ]
    };
    let opponent_loot = vec![
        Effect::DiscardAnyNumber {
            who: Selector::Player(PlayerRef::EachOpponent),
        },
        Effect::Draw {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::CountOf(Box::new(Selector::DiscardedThisResolution {
                filter: SelectionRequirement::Any,
            })),
        },
    ];
    let alt_effect: Vec<Effect> = opponent_loot.into_iter().chain(base()).collect();
    CardDefinition {
        name: "Fervent Mastery",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(base()),
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[generic(2), r(), r()]),
            effect_override: Some(Effect::Seq(alt_effect)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Illuminate History ──────────────────────────────────────────────────────

/// Illuminate History — {2}{R}{R} Sorcery — Lesson. "Discard any number of
/// cards, then draw that many cards. Then if there are seven or more cards in
/// your graveyard, create a 3/2 red and white Spirit creature token."
pub fn illuminate_history() -> CardDefinition {
    use crabomination_base::tokens::lorehold_spirit_3_2_token;
    CardDefinition {
        name: "Illuminate History",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Sorcery],
        // Discard any number, then draw that many. Then if 7+ cards in your
        // graveyard, create a 3/2 red-and-white Spirit.
        effect: Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::DiscardedThisResolution {
                    filter: SelectionRequirement::Any,
                })),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::GraveyardSizeOf(PlayerRef::You),
                    Value::Const(7),
                ),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: lorehold_spirit_3_2_token(),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}
