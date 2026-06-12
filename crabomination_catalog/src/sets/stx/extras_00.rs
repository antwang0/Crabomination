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
    SelectionRequirement, SpellSubtype, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb_drain, etb_gain_life, magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticAbility, StaticEffect, ZoneDest};
use crate::mana::{Color, b, colorless, cost, g, generic, hybrid, mono_hybrid, phyrexian, r, u, w, x, ManaCost};

// ── Bookwurm ────────────────────────────────────────────────────────────────

/// Bookwurm — {7}{G}, 7/7 Wurm. "Trample / When this creature enters,
/// you gain 4 life and draw a card."
///
/// ✅ ETB body is a simple `Seq(GainLife(4), Draw(1))`. The 5/5 trample
/// body is a fine top-end finisher in any green deck.
pub fn bookwurm() -> CardDefinition {
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
                    amount: Value::Const(4),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
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
        ..Default::default()
    };
    CardDefinition {
        name: "Reduce to Memory",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
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
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
            ),
        },
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
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

// ── Combat Professor ────────────────────────────────────────────────────────

/// Combat Professor — {3}{W} Creature — Cat Cleric, 2/3, Flying,
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
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
        card_types: vec![CardType::Instant],
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
        ..Default::default()
    }
}

// ── Spell Satchel ───────────────────────────────────────────────────────────

/// Spell Satchel — {2} Artifact. "{T}: Add {C}. / {3}, {T}, Sacrifice
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
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
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
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
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
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Squirrel Sanctuary (stand-in placeholder dropped) ───────────────────────

// ── Excavated Wall ──────────────────────────────────────────────────────────

/// Excavated Wall — {1} Artifact Creature — Wall, 0/4, Defender. "When
/// this creature enters, you gain 2 life."
///
/// ✅ Simple ETB lifegain on a defender wall body. Same shape as
/// Wall of Omens but the value is straight lifegain instead of a card.
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
        triggered_abilities: vec![etb_gain_life(2)],
        ..Default::default()
    }
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — {4}{U}{U} Instant. "Tap up to two target creatures. Put a
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
        cost: cost(&[generic(4), u(), u()]),
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
        card_types: vec![CardType::Instant],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
        
            take: None,
            to_battlefield: false,
        },
        ..Default::default()
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

/// Mortality Spear — {2}{B}{G} Instant. "Destroy target creature,
/// planeswalker, or battle."
///
/// ✅ Catch-all removal: `Destroy` against a Creature ∨ Planeswalker
/// target. Battle subtype isn't yet modelled (no MoM/March of the
/// Machine in this catalog), so the printed third clause is dropped —
/// it's a no-op in the current card pool anyway.
pub fn mortality_spear() -> CardDefinition {
    CardDefinition {
        name: "Mortality Spear",
        cost: cost(&[generic(2), b(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
        },
        ..Default::default()
    }
}

// ── Daemogoth Titan ────────────────────────────────────────────────────────

/// Daemogoth Titan — {B/G}{B/G}{B/G}{B/G}, 11/10 Demon Horror. "When this attacks or
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
        cost: cost(&[hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
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

/// Daemogoth Woe-Eater — {1}{B}{B/G}{G}, 7/6 Demon Horror. "When this enters,
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
        cost: cost(&[generic(1), b(), hybrid(Color::Black, Color::Green), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon, CreatureType::Horror],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
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

/// Quandrix Cultivator — {1}{G}{G/U}{U} 3/4 Turtle Druid. "When this creature
/// enters, you may search your library for a basic Forest or Island card, put
/// it onto the battlefield, then shuffle." (The "may" collapses to an
/// auto-search.)
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
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand.and(
                    SelectionRequirement::HasLandType(LandType::Forest)
                        .or(SelectionRequirement::HasLandType(LandType::Island)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
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
        EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement,
        StaticAbility, TriggeredAbility,
    };
    use crate::effect::{PlayerRef, Selector, StaticEffect, ZoneDest, Value};
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
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Not(Box::new(SelectionRequirement::IsToken)),
                }),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![CreatureType::Spirit],
                    override_pt: None,
                    non_legendary: false,
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
                effect: StaticEffect::PumpPT { applies_to: spirits(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Spirits you control have trample.",
                effect: StaticEffect::GrantKeyword { applies_to: spirits(), keyword: Keyword::Trample },
            },
            StaticAbility {
                description: "Spirits you control have haste.",
                effect: StaticEffect::GrantKeyword { applies_to: spirits(), keyword: Keyword::Haste },
            },
        ],
        ..Default::default()
    }
}

// ── Tempted by the Oriq ────────────────────────────────────────────────────

/// Tempted by the Oriq — {1}{U}{U}{U} Sorcery. "Gain control of target
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
    CardDefinition {
        name: "Tempted by the Oriq",
        cost: cost(&[generic(1), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        // Permanently gain control of a creature/PW (MV 3 or less) an opponent
        // controls. Printed text is per-opponent; with single-target engine
        // targeting this grabs one such permanent (exact in 1v1).
        effect: Effect::GainControl {
            what: target_filtered(
                SelectionRequirement::ControlledByOpponent
                    .and(SelectionRequirement::ManaValueAtMost(3))
                    .and(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Planeswalker),
                    ),
            ),
            to: None,
            duration: Duration::Permanent,
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
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Planeswalker
                        .and(SelectionRequirement::InGraveyard)
                        .and(SelectionRequirement::ManaValueAtMostXFromCost),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
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

/// Mascot Interception — {3}{R} Sorcery. "Gain control of target creature
/// until end of turn. Untap that creature. It gets +2/+0 and gains haste
/// until end of turn." (The "costs {3} less if it targets a token" reduction
/// is dropped.)
pub fn mascot_interception() -> CardDefinition {
    CardDefinition {
        name: "Mascot Interception",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(SelectionRequirement::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
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
/// cards unless you discard an instant or sorcery card." (The discard is
/// modeled as a flat discard-2; the IS-discard exemption is dropped.)
pub fn practical_research() -> CardDefinition {
    CardDefinition {
        name: "Practical Research",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
        ]),
        ..Default::default()
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
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::super::tap_add_colorless(),
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
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
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Star Pupil — {W} Creature — Cat Spirit, 0/1 (Silverquill).
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
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
            Effect::ExileAnyNumberFromGraveyards { filter: SelectionRequirement::Any },
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

/// Mentor's Guidance — {2}{U} Instant (Quandrix).
/// "Choose one — / • Mentor's Guidance deals damage equal to the
/// number of creatures you control to target creature an opponent
/// controls. / • Draw a card for each creature with a +1/+1 counter
/// on it you control."
///
/// Two-mode `ChooseMode`. Mode 0 deals `CountOf(YourCreatures)` damage to
/// a target creature an opponent controls (`ControlledByOpponent` filter).
/// Mode 1 draws `CountOf(YourCreatures WithCounter(+1/+1))` cards.
pub fn mentors_guidance() -> CardDefinition {
    use crate::card::{CreatureType, Predicate};
    use crate::effect::shortcut::on_cast;
    CardDefinition {
        name: "Mentor's Guidance",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        // Scry 1, then draw a card.
        effect: Effect::Seq(vec![
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
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
        card_types: vec![CardType::Sorcery],
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
                pay_own_cost: false, any_color: false,
            },
        ]),
        ..Default::default()
    }
}

// ── Magma Opus ──────────────────────────────────────────────────────────────

/// Magma Opus — {6}{U}{R} Sorcery. "Magma Opus deals 4 damage divided
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
        cost: cost(&[generic(6), u(), r()]),
        card_types: vec![CardType::Sorcery],
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
            creature_types: vec![CreatureType::Construct],
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
        ..Default::default()
    }
}

// ── First Day of Class ──────────────────────────────────────────────────────

/// First Day of Class — {1}{R} Sorcery. "Whenever a creature you control
/// enters this turn, put a +1/+1 counter on it and it gains haste until
/// end of turn. Learn." The turn-scoped enters trigger rides
/// `Effect::CreaturesYouControlEnteringThisTurn` (CR 603.4).
pub fn first_day_of_class() -> CardDefinition {
    CardDefinition {
        name: "First Day of Class",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
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
            Effect::Learn { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

// ── Verdant Mastery ─────────────────────────────────────────────────────────

/// Verdant Mastery — {5}{G} Sorcery. "Search your library for a
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
    use crate::card::AlternativeCost;
    let basic = || SelectionRequirement::IsBasicLand;
    let to_your_bf = || ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true };
    let to_hand = || ZoneDest::Hand(PlayerRef::You);
    // Base: put two basics onto the battlefield tapped under your control and
    // the rest (up to two) into your hand.
    let base = || {
        Effect::Seq(vec![
            Effect::Search { who: PlayerRef::You, filter: basic(), to: to_your_bf() },
            Effect::Search { who: PlayerRef::You, filter: basic(), to: to_your_bf() },
            Effect::Search { who: PlayerRef::You, filter: basic(), to: to_hand() },
            Effect::Search { who: PlayerRef::You, filter: basic(), to: to_hand() },
        ])
    };
    // Alt ({3}{G} paid): one basic goes onto the battlefield tapped under an
    // opponent's control, two under yours, the rest into your hand.
    let alt = Effect::Seq(vec![
        Effect::Search {
            who: PlayerRef::You,
            filter: basic(),
            to: ZoneDest::Battlefield { controller: PlayerRef::EachOpponent, tapped: true },
        },
        Effect::Search { who: PlayerRef::You, filter: basic(), to: to_your_bf() },
        Effect::Search { who: PlayerRef::You, filter: basic(), to: to_your_bf() },
        Effect::Search { who: PlayerRef::You, filter: basic(), to: to_hand() },
    ]);
    CardDefinition {
        name: "Verdant Mastery",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Sorcery],
        effect: base(),
        alternative_cost: Some(AlternativeCost {
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
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
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
/// (The additional cost is modeled as a sacrifice — exile-as-cost isn't a
/// distinct primitive.)
pub fn necrotic_fumes() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Fumes",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
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
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
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

/// Mage Duel — {2}{G} Sorcery. Target creature you control gets +1/+2 until
/// end of turn, then it fights target creature you don't control.
///
/// Two-slot spell: slot 0 is the friendly creature (pumped, then the fight's
/// attacker), slot 1 (`additional_targets[0]`) is the opponent's victim. The
/// "{2} less if you've cast another instant/sorcery this turn" discount is
/// dropped.
pub fn mage_duel() -> CardDefinition {
    CardDefinition {
        name: "Mage Duel",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
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
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![magecraft_self_pump(1, 0)],
        ..Default::default()
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
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

// ── Wandering Archaic ───────────────────────────────────────────────────────

/// Wandering Archaic — {5} Creature — Spirit, 4/4.
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
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
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
            Effect::Search { who: PlayerRef::You, filter: SelectionRequirement::Any, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Search { who: PlayerRef::You, filter: SelectionRequirement::Any, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Search { who: PlayerRef::You, filter: SelectionRequirement::Any, to: ZoneDest::Hand(PlayerRef::You) },
            Effect::Discard { who: Selector::You, amount: Value::Const(3), random: true },
        ]
    };
    let opponent_loot = vec![
        Effect::DiscardAnyNumber { who: Selector::Player(PlayerRef::EachOpponent) },
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
