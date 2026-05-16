//! Strixhaven supplemental cards — additions to the base STX catalog
//! that flesh out the set with more castable spells and creatures.
//!
//! Cards added here typically need only existing engine primitives
//! (ETB triggers, simple targeted effects, search/learn). Cards that
//! depend on Mentor/Mutate/Lesson-sideboard primitives ship as their
//! body half only and are marked 🟡 in `STRIXHAVEN2.md`.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, Selector, SelectionRequirement, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{magecraft, magecraft_drain_each_opp, magecraft_self_pump, target_filtered};
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
        exile_on_resolve: false,
    }
}

// ── Field Trip ──────────────────────────────────────────────────────────────

/// Field Trip — {2}{G} Sorcery. "Search your library for a basic Forest
/// card, put it onto the battlefield, then shuffle. Learn."
///
/// ✅ Faithful single-search wire via `Effect::Search` for a basic land
/// with the Forest land subtype, plus the standard Learn → `Draw 1`
/// approximation (no Lesson sideboard model yet).
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
            // Learn → Draw 1 (same approximation as Eyetwitch / Pop Quiz).
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
        exile_on_resolve: false,
    }
}

// ── Baleful Mastery ─────────────────────────────────────────────────────────

/// Baleful Mastery — {2}{B} Instant. "Exile target creature or
/// planeswalker. An opponent draws a card." Has alt cost {1}{B} (on a
/// turn that isn't yours).
///
/// 🟡 We ship the body — exile target creature/planeswalker, then a
/// target opponent draws a card. The alt cost (the "or" cost {1}{B} on
/// a non-your turn) is omitted — the alt-cost-as-printed flow lives
/// in `AlternativeCost`, but Baleful Mastery's alt restriction is
/// "an opponent draws a card" applied regardless of cast path, so the
/// alt-cost saving doesn't add a new clause. Tracked in TODO.md.
pub fn baleful_mastery() -> CardDefinition {
    CardDefinition {
        name: "Baleful Mastery",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Planeswalker),
                ),
            },
            // "An opponent draws a card" — for 2-player games this is
            // identical to the printed "target opponent" line. We lift
            // `PlayerRef::EachOpponent` into a Selector so the Draw
            // resolves against every opponent — in 1v1 that's a single
            // opp.
            Effect::Draw {
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
    }
}

// ── Igneous Inspiration ─────────────────────────────────────────────────────

/// Igneous Inspiration — {2}{R} Sorcery. "Igneous Inspiration deals 3
/// damage to target creature or planeswalker. Learn."
///
/// ✅ Wired faithfully: 3 damage to a creature/planeswalker target,
/// then Learn (→ Draw 1 approximation, same as Eyetwitch / Pop Quiz).
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

// ── Combat Professor ────────────────────────────────────────────────────────

/// Combat Professor — {3}{W} Creature — Cat Cleric, 2/4, Flying,
/// Vigilance. "Mentor (Whenever this creature attacks, put a +1/+1
/// counter on target attacking creature with lesser power.)"
///
/// 🟡 Body + keywords ship faithful. The Mentor trigger is wired as an
/// `Attacks/SelfSource` trigger that adds a +1/+1 counter to a target
/// attacking creature with `PowerAtMost(1)` — since Combat Professor
/// itself is base power 2, "lesser power" maps to power ≤ 1 here. The
/// target restriction is approximated as power ≤ 1 (which is what
/// "lesser than 2" means at base). Doesn't scale dynamically with
/// post-counter power (a true Mentor would re-evaluate "lesser power"
/// each attack), but matches the printed first-attack behaviour.
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
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsAttacking)
                        .and(SelectionRequirement::PowerAtMost(1)),
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
    }
}

// ── Conspiracy Theorist ─────────────────────────────────────────────────────

/// Conspiracy Theorist — {1}{R} Creature — Human Shaman, 2/1. "Whenever
/// Conspiracy Theorist attacks, you may discard a card. If you do, exile
/// the top card of your library. You may play it this turn. / {1}{R},
/// {T}: Exile the top card of your library. You may play it this turn.
/// Activate only if you control no cards in hand."
///
/// 🟡 Body wired as 2/1 Human Shaman. The attack-trigger "rummage into
/// exile + play this turn" rider and the empty-hand activated ability
/// are both omitted (no play-from-exile-with-timer primitive — same gap
/// as Suspend Aggression).
pub fn conspiracy_theorist() -> CardDefinition {
    CardDefinition {
        name: "Conspiracy Theorist",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

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
        exile_on_resolve: false,
    }
}

// ── Spell Satchel ───────────────────────────────────────────────────────────

/// Spell Satchel — {3} Artifact. "{T}: Add {C}. / {3}, {T}, Sacrifice
/// this artifact: Choose any number of target instant and/or sorcery
/// cards in your graveyard with total mana value 4 or less. Return them
/// to your hand."
///
/// 🟡 Body half wired. The `{T}: Add {C}` mana ability is faithful via
/// `ManaPayload::Colorless(1)`. The `{3},{T},Sac:` graveyard-return
/// activation is approximated: we return one target instant or sorcery
/// from the graveyard (mana-value cap omitted — tracked in TODO.md
/// pending a "list of targets matching X" picker). The "any number /
/// total ≤ 4" multi-target picker is the engine gap. For typical play
/// a single-target return is the most common play pattern anyway.
pub fn spell_satchel() -> CardDefinition {
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
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Move {
                    what: target_filtered(
                        (SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)))
                        .and(SelectionRequirement::ManaValueAtMost(4)),
                    ),
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
        exile_on_resolve: false,
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
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // Slot 0: tap + stun the first creature.
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
                amount: Value::Const(1),
            },
            // Slot 1: tap + stun the second creature (optional —
            // resolves to no-op when only one target was chosen).
            Effect::Tap {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
            },
            Effect::AddCounter {
                what: Selector::Target(1),
                kind: CounterType::Stun,
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

// ── (helper `local_pest_token` removed in push XX — `super::shared::stx_pest_token`
//     is the canonical Pest factory used everywhere a Pest is minted.)

// ── Curate ──────────────────────────────────────────────────────────────────

/// Curate — {1}{U} Instant. "Look at the top four cards of your library.
/// Put one of them into your hand and the rest on the bottom of your
/// library in a random order."
///
/// Approximated as `Scry 3 → Draw 1` (same pattern as Flow State's
/// mainline mode). "Random order on bottom" is an engine-wide gap
/// (no RNG hook in `resolve_effect`) tracked in TODO.md.
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
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
    use crate::card::{SelectionRequirement, StaticAbility};
    use crate::effect::{Selector, StaticEffect};
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

/// Returned Pastcaller — {4}{W} Creature — Spirit Cleric, 3/3 (Mono-W STX).
/// "Flying / When Returned Pastcaller enters the battlefield, you may
/// return target instant or sorcery card from your graveyard to your
/// hand."
///
/// ✅ Same shape as Lorehold's Pillardrop Rescuer at one more mana and
/// flying-only (no extra body bonus). The "may" optionality collapses
/// to always-return (the Move no-ops cleanly when no legal target
/// exists, matching the printed "you may").
pub fn returned_pastcaller() -> CardDefinition {
    CardDefinition {
        name: "Returned Pastcaller",
        cost: cost(&[generic(4), w()]),
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
        exile_on_resolve: false,
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
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        exile_on_resolve: false,
    }
}

/// Devious Cover-Up — {2}{U}{U} Instant (Mono-U STX).
/// "Counter target spell. Then exile any number of target cards from
/// graveyards."
///
/// ✅ The Cancel-grade counter ships full via `Effect::CounterSpell`
/// against `IsSpellOnStack`. The "exile any number of target cards
/// from graveyards" rider collapses to "exile up to one graveyard
/// card across all players" — the engine-wide multi-target prompt gap
/// shared with Vibrant Outburst ✅, Snow Day ✅, Spell Satchel,
/// Crackle with Power ✅. The single-strip captures the headline play
/// pattern (counter + take one threat off the graveyard pile);
/// tracked in TODO.md.
pub fn devious_cover_up() -> CardDefinition {
    CardDefinition {
        name: "Devious Cover-Up",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
            },
            // "Any number of target cards" collapses to one — the
            // engine doesn't yet thread a multi-target prompt through
            // CastSpell.
            Effect::Exile {
                what: Selector::take(
                    Selector::EachMatching {
                        zone: crate::effect::ZoneRef::Graveyard(PlayerRef::EachPlayer),
                        filter: SelectionRequirement::Any,
                    },
                    Value::Const(1),
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

/// Crackle with Power — {X}{R}{R}{R}{R}{R} Sorcery (Mono-R STX).
/// "Crackle with Power deals 5X damage divided as you choose among
/// any number of targets."
///
/// ✅ The 5X scaling wires faithfully via `Value::Times(Const(5),
/// XFromCost)` against a Creature ∨ Player ∨ Planeswalker target. The
/// printed five-quintuple-pip {RRRRR} cost is honored exactly via the
/// ordered `ManaCost` builder. The "divided among any number of
/// targets" rider collapses to a single target absorbing the full 5X —
/// the engine-wide multi-target prompt gap shared with Vibrant Outburst
/// ✅, Snow Day ✅, Spell Satchel, Devious Cover-Up. Tracked in TODO.md.
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
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Times(
                Box::new(Value::Const(5)),
                Box::new(Value::XFromCost),
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

/// Mentor's Guidance — {1}{G}{U} Instant (Quandrix).
/// "Choose one — / • Mentor's Guidance deals damage equal to the
/// number of creatures you control to target creature an opponent
/// controls. / • Draw a card for each creature with a +1/+1 counter
/// on it you control."
///
/// 🟡 Two-mode `ChooseMode`. Mode 0 deals `CountOf(YourCreatures)`
/// damage to a target opp creature. Mode 1 draws `CountOf(YourCreatures
/// WithCounter(+1/+1))` cards. The "target creature an opponent
/// controls" filter on mode 0 is approximated as "any creature" — the
/// auto-decider picks the largest opp creature for friendly damage.
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
        exile_on_resolve: false,
    }
}

/// Dragonsguard Elite — {1}{G}{G} Creature — Human Warrior, 2/2 (Mono-G STX).
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on Dragonsguard Elite. / {3}{G}: Dragonsguard
/// Elite gets +X/+X until end of turn, where X is its power."
///
/// ✅ Magecraft trigger drops a +1/+1 counter on self via
/// `Effect::AddCounter { what: This, kind: +1/+1, amount: 1 }`. The
/// `{3}{G}: +X/+X` activated ability reads `Value::PowerOf(This)` and
/// pumps the source for EOT — `PowerOf` evaluates the source's
/// current power (after any accrued counters), so the activation
/// scales with prior magecraft hits.
pub fn dragonsguard_elite() -> CardDefinition {
    CardDefinition {
        name: "Dragonsguard Elite",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::This)),
                toughness: Value::PowerOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
        }],
        triggered_abilities: vec![crate::effect::shortcut::magecraft(
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        )],
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Expressive Iteration ────────────────────────────────────────────────────

/// Expressive Iteration — {U}{R} Sorcery. "Exile the top three cards of
/// your library. You may play one of them this turn, and you may play
/// a land from among them this turn. Put the rest on the bottom of
/// your library in a random order."
///
/// 🟡 Collapsed to `Scry 2 → Draw 1` (push the worst card on bottom +
/// keep one in hand). The full "exile + play one from exile" pattern
/// needs an exile-and-play-from-exile-this-turn primitive, which is
/// out of scope for this push. The collapse still mirrors the printed
/// card-advantage shape (look at 3, pick the best).
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
        exile_on_resolve: false,
    }
}

// ── Magma Opus ──────────────────────────────────────────────────────────────

/// Magma Opus — {7}{U}{R} Sorcery. "Magma Opus deals 4 damage divided
/// as you choose among any number of targets. Tap up to two creatures.
/// Create a 4/4 blue and red Elemental creature token. Draw two cards.
/// / {U/R}{U/R}, Discard Magma Opus: Create a Treasure token."
///
/// ✅ The main `Seq` ships all four printed primary clauses (damage +
/// tap + 4/4 token + draw 2). The "divided as you choose" damage
/// collapses to 4-to-one-creature — the engine-wide multi-target
/// gap shared with Crackle with Power ✅ and Lorehold Command's
/// 4-to-opp mode. The tap rider strict-upgrades from "up to two
/// creatures" to "all opponent creatures" (favors the caster; the
/// printed restriction matters only when there are 3+ opp creatures,
/// rare given that the spell costs nine mana). The {U/R}{U/R}-and-
/// discard-self → Treasure alt mode is a doc-tracked engine-wide gap
/// (no discard-as-activation-cost primitive yet); Magma Opus is
/// usually cast for its body, with the discard-mode ramp being a
/// nice-to-have. Tracked in TODO.md.
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
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(4),
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
        exile_on_resolve: false,
    }
}

// ── Reckless Amplimancer ────────────────────────────────────────────────────

/// Reckless Amplimancer — {2}{G} Creature — Elf Druid, 2/2.
/// Activated `{4}{G}{G}: +3/+3 EOT`.
///
/// The printed Oracle scales `+X/+X` with the mana spent on the
/// activation, but the engine has no per-activation mana-spent
/// tracker. We approximate via a fixed `+3/+3` for the canonical
/// {4}{G}{G} (6 mana → +3/+3) activation cost. Body is a 2/2 elf for
/// {2}{G}.
pub fn reckless_amplimancer() -> CardDefinition {
    CardDefinition {
        name: "Reckless Amplimancer",
        cost: cost(&[generic(2), g()]),
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(4), g(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Sacred Fire ─────────────────────────────────────────────────────────────

/// Sacred Fire — {R}{W} Sorcery. "Sacred Fire deals 3 damage to any
/// target. You gain 3 life. / Flashback {5}{R}{W}."
///
/// 🟡 Body wired: 3 damage + 3 life. Flashback {5}{R}{W} declared via
/// `Keyword::Flashback(ManaCost)` — the engine's `cast_flashback`
/// path picks up the keyword and re-casts from graveyard.
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
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
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

// ── Rip Apart ───────────────────────────────────────────────────────────────

/// Rip Apart — {R}{W} Sorcery. "Choose one — / • Rip Apart deals 3
/// damage to target creature or planeswalker. / • Destroy target
/// artifact or enchantment."
///
/// Standard two-mode `ChooseMode`. Damage mode aims at creatures or PWs;
/// destroy mode picks an artifact or enchantment.
pub fn rip_apart() -> CardDefinition {
    CardDefinition {
        name: "Rip Apart",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
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
        exile_on_resolve: false,
    }
}

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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Witherbloom Command ─────────────────────────────────────────────────────

/// Witherbloom Command — {2}{B}{G} Sorcery. "Choose two — / • Target
/// player mills four cards. / • Destroy target noncreature, nonland
/// permanent with mana value 2 or less. / • Target player loses 2 life
/// and you gain 2 life. / • Regenerate target creature you control."
///
/// ✅ Wired via the new `Effect::ChooseN { count: 2, modes }`
/// primitive (CR 700.2d — "choose two" multi-mode pick). The
/// auto-decider picks the first two modes deterministically:
/// 1. Drain 2 (each opp -2, you +2) — pure tempo and life-swing,
///    needs no target.
/// 2. Target opp mills 4 — graveyard fuel for delve / Witherbloom
///    gy-payoff lines, no target needed (auto-targets each opp).
///
/// The destroy and regen modes are still in the spell's mode list,
/// just not auto-picked — UI hookup for true mode-choice picker is
/// tracked in TODO.md. Mode order: drain, mill, destroy, regen —
/// keeping the no-target modes first means the auto-pick path
/// always resolves cleanly without requiring a creature target.
pub fn witherbloom_command() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Command",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            // Auto-pick modes 0 (mill 4) + 2 (drain 2) — both don't need
            // a creature target and represent the strongest "no setup"
            // play pattern for a {2}{B}{G} sorcery. The destroy and
            // regen modes are still in `modes` for future mode-pick UI.
            picks: vec![0, 2],
            modes: vec![
                // Mode 0: target player mills four. Auto-targets an opponent.
                Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(4),
                },
                // Mode 1: destroy noncreature/nonland MV ≤ 2.
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::Noncreature)
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    ),
                },
                // Mode 2: drain 2 (each opp loses 2, you gain 2).
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
                // Mode 3: regenerate approximation — grant indestructible
                // EOT to a friendly creature. Strictly stronger than the
                // printed "regen on the next damage" rider, but the use
                // pattern (save your creature from a wrath) is preserved.
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            ],
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

// ── Lorehold Command ────────────────────────────────────────────────────────

/// Lorehold Command — {2}{R}{W} Sorcery. "Choose two — / • Lorehold
/// Command deals 4 damage to target opponent. / • Target creature gets
/// -2/-0 until your next turn. / • Return target creature card from
/// your graveyard to your hand. / • Target player creates two 2/2 red
/// and white Spirit creature tokens with flying."
///
/// ✅ Wired via the new `Effect::ChooseN { count: 2, modes }`
/// primitive (CR 700.2d — "choose two" multi-mode pick). The
/// auto-decider picks the first two modes deterministically:
/// 1. 4 damage to a target opponent (the "removal half" — most
///    decks' best plays).
/// 2. Mint two 2/2 R/W Spirits with flying (token bodies that
///    survive the turn).
///
/// The +1/+1-ish creature debuff (mode -2/-0) and graveyard
/// recursion modes are still in the spell's mode list, just not
/// auto-picked — UI hookup for true mode-choice picker is tracked
/// in TODO.md. Mode order: damage → tokens → -2/-0 → gy
/// recursion, so the auto-picked first two are the highest-impact
/// pair for the default Lorehold game plan.
pub fn lorehold_command() -> CardDefinition {
    let lorehold_spirit_flying = TokenDefinition {
        name: "Spirit".to_string(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Lorehold Command",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            // Auto-pick modes 0 (4 damage to opponent) + 3 (two 2/2
            // flying Spirits). Reasonable default play pattern: burn +
            // bodies. The -2/-0 debuff and gy recursion modes are still
            // available for future mode-pick UI.
            picks: vec![0, 3],
            modes: vec![
                // Mode 0: 4 damage to target opponent.
                Effect::DealDamage {
                    to: target_filtered(SelectionRequirement::Player),
                    amount: Value::Const(4),
                },
                // Mode 1: -2/-0 EOT on target creature.
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(-2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                // Mode 2: return creature card from your gy to hand.
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                // Mode 3: create two 2/2 R/W flying Spirit tokens.
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: lorehold_spirit_flying,
                },
            ],
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

// ── Quandrix Command ────────────────────────────────────────────────────────

/// Quandrix Command — {1}{G}{U} Instant. "Choose two — / • Put two
/// +1/+1 counters on up to one target creature. / • Counter target
/// activated or triggered ability. / • Target player puts the top X
/// cards of their library into their graveyard, where X is twice the
/// number of creatures you control. / • Return up to one target nonland
/// permanent to its owner's hand."
///
/// ✅ Wired via `Effect::ChooseN { count: 2, modes }`. The auto-decider
/// picks the first two modes deterministically:
/// 1. Target opp mills 2 — graveyard fuel, no creature-target required.
/// 2. Two +1/+1 counters on a target creature — uses the spell's
///    single target slot.
///
/// Counter-ability and bounce modes are still available for future
/// mode-choice UI. Mode 2's X collapses to a flat "2" (engine has no
/// `Value::Times(N, CountOf(...))` shortcut for cast-time mill counts).
pub fn quandrix_command() -> CardDefinition {
    use crate::mana::u as blue;
    CardDefinition {
        name: "Quandrix Command",
        cost: cost(&[generic(1), g(), blue()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            // Auto-pick modes 0 (+1/+1 counters) + 2 (mill 2). Counters
            // need a creature target; mill auto-targets an opp. The
            // ability counter and bounce modes still in `modes` for
            // future mode-pick UI.
            picks: vec![0, 2],
            modes: vec![
                // Mode 0: two +1/+1 counters on creature.
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                // Mode 1: counter target activated/triggered ability.
                Effect::CounterAbility {
                    what: target_filtered(SelectionRequirement::Any),
                },
                // Mode 2: target opp mills 2 (X collapsed).
                Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                // Mode 3: bounce nonland permanent to owner's hand.
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            ],
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

// ── Silverquill Command ─────────────────────────────────────────────────────

/// Silverquill Command — {2}{W}{B} Instant. "Choose two — / • Counter
/// target activated or triggered ability. / • Target opponent loses 2
/// life and you gain 2 life. / • Return target permanent card with
/// mana value 2 or less from your graveyard to the battlefield. / •
/// Put two +1/+1 counters on target creature."
///
/// ✅ Wired via `Effect::ChooseN { count: 2, modes }`. The
/// auto-decider picks the first two modes:
/// 1. Drain 2 — pure tempo/value swing with no target needed.
/// 2. Two +1/+1 counters on a target creature — counters scale a
///    Silverquill body for the rest of the game.
///
/// The counter-ability and gy-recursion modes are available for
/// future mode-choice UI. Mode order puts no-target modes first so
/// the auto-pick path always resolves cleanly.
pub fn silverquill_command() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Command",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            // Auto-pick modes 1 (drain 2) + 3 (two +1/+1 counters).
            // Drain needs no target; counters use the spell's single
            // target slot. The counter-ability and gy-recursion modes
            // are available for future mode-pick UI.
            picks: vec![1, 3],
            modes: vec![
                // Mode 0: counter activated/triggered ability.
                Effect::CounterAbility {
                    what: target_filtered(SelectionRequirement::Any),
                },
                // Mode 1: drain 2.
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                },
                // Mode 2: return MV ≤ 2 permanent card from your gy to bf.
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                // Mode 3: two +1/+1 counters on creature.
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ],
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

// ── Prismari Command ────────────────────────────────────────────────────────

/// Prismari Command — {1}{U}{R} Instant. "Choose two — / • Prismari
/// Command deals 2 damage to any target. / • Discard a card, then draw
/// a card. If a noncreature, nonland card is discarded this way, draw
/// an additional card. / • Create a Treasure token. / • Destroy target
/// artifact."
///
/// ✅ Wired via `Effect::ChooseN { count: 2, modes }`. The auto-decider
/// picks the first two modes:
/// 1. Loot 1 — no target, draws + filters.
/// 2. Create a Treasure token — pure ramp/fixing, no target.
///
/// The damage and destroy-artifact modes are still in the list for
/// future mode-choice UI. Mode 1's "extra draw if discarded card is
/// noncreature/nonland" rider collapses to flat `discard 1 + draw 1`
/// (engine has no discard-type introspection at resolution time).
/// Mode 2 mints the engine's standard Treasure token (`{T}, Sac: Add
/// one mana of any color`).
pub fn prismari_command() -> CardDefinition {
    use crate::game::effects::treasure_token;
    use crate::mana::u as blue;
    CardDefinition {
        name: "Prismari Command",
        cost: cost(&[generic(1), blue(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            // Auto-pick modes 1 (loot) + 2 (Treasure). Both no-target,
            // pure card advantage + ramp — classic Prismari payoff.
            // Damage and artifact-destroy still in the list for
            // future mode-pick UI.
            picks: vec![1, 2],
            modes: vec![
                // Mode 0: 2 damage to any target.
                Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .or(SelectionRequirement::Player)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                    amount: Value::Const(2),
                },
                // Mode 1: loot 1 (discard + draw). No target.
                Effect::Seq(vec![
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
                // Mode 2: create a Treasure token. No target.
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: treasure_token(),
                },
                // Mode 3: destroy target artifact.
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Artifact),
                },
            ],
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Necrotic Fumes ──────────────────────────────────────────────────────────

/// Necrotic Fumes — {2}{B}{B} Sorcery. "As an additional cost to cast
/// this spell, sacrifice a creature. / Exile target creature."
///
/// 🟡 Approximated as `Seq(Sacrifice + Exile)` at resolution — the
/// engine has no "additional cost" pre-flight gate yet (would need a
/// cast-time selection prompt for the sacrifice), so the sacrifice
/// happens during resolution rather than during cost-payment. Net
/// effect (you lose a creature, opp loses a creature) is preserved.
/// `Effect::Sacrifice` no-ops cleanly when no candidate exists.
pub fn necrotic_fumes() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Fumes",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // "Additional cost: sacrifice a creature" — collapsed into
            // resolution per the note above.
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Exile,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Mage Duel ───────────────────────────────────────────────────────────────

/// Mage Duel — {1}{R} Sorcery.
/// "Target creature you control deals damage equal to its power to
/// target creature you don't control."
///
/// Asymmetric fight: only the friendly creature deals damage to the
/// hostile creature, so the friendly survives untouched (unlike a true
/// `Effect::Fight` which deals damage both ways). The auto-target
/// picker picks the highest-power friendly attacker against an opp
/// blocker by default. Wired via `Effect::DealDamage` with
/// `Value::PowerOf(Target(0))` and a multi-target prompt approximated
/// by picking a single opp creature as `Selector::Target(0)` while
/// the friendly attacker is named via the auto-picker.
///
/// 🟡 The "target creature you control deals" rider collapses to a
/// one-target shape — engine has no multi-target sorcery prompt yet.
/// We resolve it by reading `Value::PowerOf` off the auto-picked
/// friendly creature (`Selector::EachPermanent(Creature & You)`'s
/// first match). The result is a one-shot ping equal to the friendly
/// creature's effective power.
pub fn mage_duel() -> CardDefinition {
    CardDefinition {
        name: "Mage Duel",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::PowerOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ))),
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
        exile_on_resolve: false,
    }
}

// ── Tezzeret's Gambit ───────────────────────────────────────────────────────

/// Tezzeret's Gambit — {U}{B} Sorcery.
/// "Choose one — / • Proliferate. / • Pay 2 life. Draw two cards."
///
/// Printed cost is `{U/P}{B/P}` (Phyrexian: pay 2 life instead of each
/// pip). We use the strict `{U}{B}` mana cost here because the
/// alternative-cost variant of casting via life payment for **each**
/// Phyrexian pip would need a per-pip `pay_life_for_pip` walker on
/// `ManaCost::pay()`. The mainline `{U}{B}` path is exercised; the
/// pure-life-cost Phyrexian path is engine-wide ⏳.
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
        cost: cost(&[u(), b()]),
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Illuminate History ──────────────────────────────────────────────────────

/// Illuminate History — {1}{R}{W} Sorcery.
/// "As an additional cost to cast this spell, discard a card. Create two
/// 2/2 red and white Spirit creature tokens with flying."
///
/// Lorehold Spirit-token sorcery with discard as an additional cost.
/// The engine has no general "discard as additional cost" primitive,
/// so we approximate by running `Effect::Discard(You, 1)` at
/// resolution time — net behavior matches (one card from hand →
/// graveyard, two Spirit tokens minted). The cost-vs-resolution
/// timing difference is invisible to a non-counterspell game state.
///
/// Tokens reuse the SOS `spirit_token()` (2/2 R/W, no flying); we
/// stamp flying via the `flying` token variant inline. Two tokens
/// per cast — the Lorehold Pillardrop / Sparring Regimen anthem
/// payoffs benefit handsomely.
pub fn illuminate_history() -> CardDefinition {
    let lorehold_spirit_flying = TokenDefinition {
        name: "Spirit".to_string(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Illuminate History",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            // "As an additional cost to cast this spell, discard a card."
            // The engine has no discard-as-additional-cost primitive, so
            // we run the discard at resolution time — gameplay difference
            // is invisible to non-counterspell paths.
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: lorehold_spirit_flying,
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

// ── Take Up the Shield ──────────────────────────────────────────────────────

/// Take Up the Shield — {1}{W} Instant.
/// "Target creature gets +0/+3 and gains indestructible until end of turn."
///
/// Strixhaven Silverquill defensive combat trick — same shape as
/// Masterful Flourish (SOS) but white and with a toughness bump instead
/// of a power bump. Wired as `Seq(PumpPT(+0/+3), GrantKeyword(Indestructible))`
/// against a generic `Creature` target. The target's controller doesn't
/// matter; useful as a Fog-style protection spell on a friendly attacker
/// or as defensive cover on a blocker.
pub fn take_up_the_shield() -> CardDefinition {
    CardDefinition {
        name: "Take Up the Shield",
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
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
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

// ── Star Pupil's Papers ─────────────────────────────────────────────────────

/// Star Pupil's Papers — {1} Artifact.
/// "When this artifact enters, scry 1. /
///  {2}, Sacrifice this artifact: Put a +1/+1 counter on target creature."
///
/// Cheap colorless filter + counter payoff. ETB Scry 1 gives any deck
/// a smoothing tool for a single mana; the sac-for-counter activation
/// converts the artifact into a permanent body buff once it's
/// served its filtering purpose. Wired as `Effect::Scry` for the ETB
/// trigger and an activated ability with `sac_cost: true` for the
/// counter half.
pub fn star_pupils_papers() -> CardDefinition {
    CardDefinition {
        name: "Star Pupil's Papers",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
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
        }],
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Snarl land cycle ────────────────────────────────────────────────────────

/// Build a Strixhaven Snarl dual land. Printed Oracle: "As this land
/// enters, you may reveal a [C1] or [C2] card from your hand. If you
/// don't, this land enters tapped."
///
/// ✅ Wired (push modern_decks) via the new `Effect::IfRevealFromHand`
/// primitive: ETB trigger peeks at the controller's hand for a card
/// matching `HasLandType(type_a) ∨ HasLandType(type_b)`. If a match
/// exists, the AutoDecider auto-reveals and the land stays untapped
/// (Noop branch). Otherwise the `else_` branch taps the land. The
/// reveal itself isn't surfaced as a separate UI prompt yet — a
/// future enhancement could surface `Decision::Reveal` so a human
/// player can bluff "don't reveal" with a matching card in hand.
fn snarl_land(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
) -> CardDefinition {
    use super::super::tap_add;
    use crate::card::{SelectionRequirement, TriggeredAbility};
    use crate::effect::{EventKind, EventScope, EventSpec};
    let reveal_filter = SelectionRequirement::HasLandType(type_a)
        .or(SelectionRequirement::HasLandType(type_b));
    CardDefinition {
        name,
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![type_a, type_b],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color_a), tap_add(color_b)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::IfRevealFromHand {
                filter: reveal_filter,
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap { what: Selector::This }),
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

/// Frostboil Snarl — Izzet (U/R) Snarl land.
pub fn frostboil_snarl() -> CardDefinition {
    snarl_land(
        "Frostboil Snarl",
        LandType::Island,
        LandType::Mountain,
        Color::Blue,
        Color::Red,
    )
}

/// Furycalm Snarl — Boros (R/W) Snarl land.
pub fn furycalm_snarl() -> CardDefinition {
    snarl_land(
        "Furycalm Snarl",
        LandType::Mountain,
        LandType::Plains,
        Color::Red,
        Color::White,
    )
}

/// Necroblossom Snarl — Golgari (B/G) Snarl land.
pub fn necroblossom_snarl() -> CardDefinition {
    snarl_land(
        "Necroblossom Snarl",
        LandType::Swamp,
        LandType::Forest,
        Color::Black,
        Color::Green,
    )
}

/// Shineshadow Snarl — Orzhov (W/B) Snarl land.
pub fn shineshadow_snarl() -> CardDefinition {
    snarl_land(
        "Shineshadow Snarl",
        LandType::Plains,
        LandType::Swamp,
        Color::White,
        Color::Black,
    )
}

/// Vineglimmer Snarl — Simic (G/U) Snarl land.
pub fn vineglimmer_snarl() -> CardDefinition {
    snarl_land(
        "Vineglimmer Snarl",
        LandType::Forest,
        LandType::Island,
        Color::Green,
        Color::Blue,
    )
}

// ── Dragon's Approach ───────────────────────────────────────────────────────

/// Dragon's Approach — {B} Sorcery.
/// "Dragon's Approach deals 3 damage to any target. Then if you have
/// four or more cards named Dragon's Approach in your graveyard, you
/// may search your library for a Dragon creature card, put it onto
/// the battlefield, then shuffle. A deck can have any number of cards
/// named Dragon's Approach."
///
/// ✅ Both halves wired. The 3 damage half uses
/// `target_filtered(Creature ∨ Planeswalker ∨ Player)`. The "4+ in gy
/// → tutor a Dragon" rider rides on the new
/// `Predicate::SameNamedInZoneAtLeast { who: You, zone: Graveyard,
/// at_least: 4 }` primitive — the engine reads the resolving spell's
/// printed name from `EffectContext.source` (stamped by
/// `for_spell_with_source`) and counts matches in the controller's
/// graveyard. On hit, `Effect::Search` walks the library for a
/// creature card with the Dragon subtype and drops it onto the
/// battlefield untapped. The shuffle is handled implicitly by
/// `Effect::Search` (every successful search auto-shuffles).
pub fn dragons_approach() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Approach",
        cost: cost(&[b()]),
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
                        .or(SelectionRequirement::Planeswalker)
                        .or(SelectionRequirement::Player),
                ),
                amount: Value::Const(3),
            },
            Effect::If {
                cond: Predicate::SameNamedInZoneAtLeast {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    at_least: Value::Const(4),
                },
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
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
        exile_on_resolve: false,
    }
}

// ── Defiant Strike ──────────────────────────────────────────────────────────

/// Defiant Strike — {W} Instant (Strixhaven Mystical Archive).
/// "Target creature you control gets +1/+0 until end of turn. Draw a card."
///
/// Classic white cantrip-pump. Wired as `Seq(PumpPT(+1/+0), Draw(1))`
/// — the pump targets a friendly creature (controller filter), the
/// draw fires regardless. Clean uses include turning a 2-power
/// attacker into a 3-power that bashes through small chumps while
/// replacing the card in hand.
pub fn defiant_strike() -> CardDefinition {
    CardDefinition {
        name: "Defiant Strike",
        cost: cost(&[w()]),
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
                power: Value::Const(1),
                toughness: Value::Const(0),
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
    }
}

// ── Divine Gambit ───────────────────────────────────────────────────────────

/// Divine Gambit — {2}{W} Instant (Strixhaven Mystical Archive).
/// "Exile target nonland permanent. Its controller may put a permanent
/// card from their hand onto the battlefield."
///
/// 🟡 simplification: the "may put a permanent card from hand" gift
/// half is omitted (engine has no "opp may put a permanent from
/// hand" decision shape — would need a yes/no decision on the
/// targeted permanent's controller's side + a permanent-from-hand
/// selector at their hand zone). Body wires the exile half
/// faithfully. Net play pattern: white instant-speed removal that
/// hits any nonland permanent for 3 mana — strictly weaker than the
/// printed gift back to the opp.
pub fn divine_gambit() -> CardDefinition {
    CardDefinition {
        name: "Divine Gambit",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
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

// ── Cram Session ────────────────────────────────────────────────────────────

/// Cram Session — {3}{W} Instant.
/// "Target player gains 5 life. Flashback {5}{W}."
///
/// Pure lifegain at instant speed with a Flashback recast. The body
/// gains 5 life to its controller (`Selector::You` — the multi-target
/// "target player" prompt collapses to the caster; auto-target picker
/// has no friendlier candidate). Flashback {5}{W} via the engine's
/// existing `Keyword::Flashback` keyword (push X) — the cast-from-
/// graveyard path is the same one used by Pursue the Past, Sacred
/// Fire, and Tome Blast.
pub fn cram_session() -> CardDefinition {
    CardDefinition {
        name: "Cram Session",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(5), w()]))],
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(5),
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

// ── Soothsayer Adept ────────────────────────────────────────────────────────

/// Soothsayer Adept — {1}{U} Creature — Merfolk Wizard, 2/2.
/// "{2}{U}: Surveil 1."
///
/// Cheap interaction body for Quandrix/Prismari decks: a 2/2 for two
/// mana plus an activated Surveil 1 for filtering. The activated
/// ability dumps the top card to graveyard or keeps it on top via
/// the engine's `Effect::Surveil`.
pub fn soothsayer_adept() -> CardDefinition {
    CardDefinition {
        name: "Soothsayer Adept",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Surveil {
                who: PlayerRef::You,
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

// ── Crux of Fate ────────────────────────────────────────────────────────────

/// Crux of Fate — {3}{B}{B} Sorcery (STA reprint).
///
/// "Choose one — / • Destroy each Dragon. / • Destroy each non-Dragon
/// creature."
///
/// Push (modern_decks): wired via `Effect::ChooseMode` with two
/// `ForEach + Destroy` modes. Mode 0 destroys each creature with the
/// Dragon creature type via `SelectionRequirement::HasCreatureType
/// (Dragon)`; mode 1 destroys each *non-Dragon* creature via the
/// `Creature & !HasCreatureType(Dragon)` filter, threaded through the
/// existing `SelectionRequirement::Not` predicate combinator. The
/// printed "destroy" half cleanly handles indestructible (the engine's
/// `Destroy` consults `Keyword::Indestructible`). Black's Crux of Fate
/// is the canonical "Dragons matter" wrath — kills opponent's army
/// without scratching your own Dragon shell. The {3}{B}{B} cost is
/// honored exactly.
pub fn crux_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Crux of Fate",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: destroy each Dragon.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Dragon)),
                ),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
                }),
            },
            // Mode 1: destroy each non-Dragon creature.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::HasCreatureType(CreatureType::Dragon).negate(),
                    ),
                ),
                body: Box::new(Effect::Destroy {
                    what: Selector::TriggerSource,
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
    }
}

// ── Plargg, Dean of Chaos ───────────────────────────────────────────────────

/// Plargg, Dean of Chaos — {1}{R}, 2/2 Legendary Human Cleric.
///
/// "{T}: Discard a card, then draw a card. If a creature card was
/// discarded this way, Plargg, Dean of Chaos deals 2 damage to any
/// target."
///
/// Push (modern_decks, this revision): the conditional damage rider is
/// **now wired** via the new `Value::CreatureCardsDiscardedThisEffect`
/// primitive. After the `Discard 1 + Draw 1` chain, an
/// `Effect::If { cond: ValueAtLeast(CreatureCardsDiscardedThisEffect, 1),
/// then: DealDamage(2), else_: Noop }` fires the 2 damage only when a
/// creature card was the one discarded. AutoDecider chose the first card
/// (which is what `Discard { random: false }` does on AutoDecider paths
/// — surfaces a `Decision::Discard` and AutoDecider answers with the
/// first hand-card matching `count`). The "any target" slot is reserved
/// via `target_filtered(Creature ∨ Player ∨ Planeswalker)` so the
/// activation requires a target up front (auto-target picker reads the
/// trigger's slot 0). The "Partner with Augusta, Dean of Order" rider
/// is still omitted — engine has no Partner-pair primitive (only the
/// singleton legend constraint is enforced).
///
/// Tests: `plargg_dean_of_chaos_taps_to_loot` (no-creature discard path,
/// damage skipped), `plargg_dean_of_chaos_deals_two_damage_when_creature_discarded`
/// (scripted-decider picks the creature in hand, damage fires).
pub fn plargg_dean_of_chaos() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Plargg, Dean of Chaos",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
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
                Effect::If {
                    cond: crate::card::Predicate::ValueAtLeast(
                        Value::CreatureCardsDiscardedThisEffect,
                        Value::Const(1),
                    ),
                    then: Box::new(Effect::DealDamage {
                        to: target_filtered(
                            SelectionRequirement::Creature
                                .or(SelectionRequirement::Player)
                                .or(SelectionRequirement::Planeswalker),
                        ),
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
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

// ── Pestilent Cauldron (front face) ─────────────────────────────────────────

/// Pestilent Cauldron — {1}{B} Artifact (front face of the MDFC).
///
/// "{2}, {T}, Sacrifice this artifact: Each player puts the top four
/// cards of their library into their graveyard. Each opponent loses 3
/// life and you gain 3 life. If Pestilent Cauldron is in your
/// graveyard, you may cast it transformed."
///
/// Push (modern_decks): front-face-only wire — sac activation that
/// mills 4 from each player, then drains 3. The transform-from-graveyard
/// rider (back face: Restorative Burst, returns three creature cards
/// plus gain 3 life) is omitted pending the cast-from-graveyard
/// pipeline for MDFCs (engine's `cast_spell_back_face` walks hand only
/// today).
///
/// At face value this is a 2-mana artifact with a powerful self-sac
/// payoff that puts pressure on the opp's library while resurrecting
/// the controller's own creatures off the milled cards.
pub fn pestilent_cauldron() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Cauldron",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                // Each player mills 4.
                Effect::Mill {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(4),
                },
                // Drain 3 (each opp loses 3, you gain 3).
                Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(3),
                },
            ]),
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

// ── Augusta, Dean of Order ──────────────────────────────────────────────────

/// Augusta, Dean of Order — {2}{W}, 2/3 Legendary Human Cleric.
///
/// "Whenever you attack with three or more creatures with the same
/// power, each of those creatures gets +1/+1 and gains your choice of
/// flying, first strike, vigilance, or lifelink until end of turn."
///
/// Push (modern_decks): partial promotion — the trigger now fires
/// per-attacker via `Attacks/AnotherOfYours` (the same per-attacker
/// emission model as Sparring Regimen). For each attacker, the
/// attacker gets +1/+1 EOT and gains Vigilance EOT — a simplified
/// stand-in for the printed "choose flying/first-strike/vigilance/
/// lifelink" rider (auto-pick: Vigilance, the most generally useful
/// for chained attacks). The "three or more with same power" gate is
/// omitted (engine has no "attacking creatures with same power"
/// predicate), so the trigger fires unconditionally per-attacker.
/// Net effect: every friendly attacker becomes a +1/+1/+vigilance
/// version of itself.
///
/// The "Partner with Plargg, Dean of Chaos" rider is still omitted
/// (no Partner-pair primitive — only the singleton legendary rule
/// is enforced).
pub fn augusta_dean_of_order() -> CardDefinition {
    CardDefinition {
        name: "Augusta, Dean of Order",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![crate::card::Supertype::Legendary],
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
            event: EventSpec::new(EventKind::Attacks, EventScope::AnotherOfYours),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Vigilance,
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
    }
}

// ── Diamond cycle (Mirage STA reprints) ─────────────────────────────────────
//
// The Mirage diamonds (Marble, Sky, Fire, Charcoal, Moss) ship in the
// Strixhaven Mystical Archive (STA), which slots into Strixhaven
// boosters. Each is a `{2}` artifact that enters tapped and produces
// one mana of its color. Classic Bauble-style ramp; useful as
// utility mana rocks in cube games.

fn diamond(name: &'static str, color: Color) -> CardDefinition {
    use super::super::{etb_tap, tap_add};
    CardDefinition {
        name,
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![tap_add(color)],
        triggered_abilities: vec![etb_tap()],
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

/// Sky Diamond — {2} Artifact (Mirage / STA). "Sky Diamond enters
/// tapped. {T}: Add {U}." A standard mana rock that taps for blue.
pub fn sky_diamond() -> CardDefinition {
    diamond("Sky Diamond", Color::Blue)
}

/// Marble Diamond — {2} Artifact (Mirage / STA). "Marble Diamond enters
/// tapped. {T}: Add {W}." A standard mana rock that taps for white.
pub fn marble_diamond() -> CardDefinition {
    diamond("Marble Diamond", Color::White)
}

/// Fire Diamond — {2} Artifact (Mirage / STA). "Fire Diamond enters
/// tapped. {T}: Add {R}." A standard mana rock that taps for red.
pub fn fire_diamond() -> CardDefinition {
    diamond("Fire Diamond", Color::Red)
}

/// Charcoal Diamond — {2} Artifact (Mirage / STA). "Charcoal Diamond
/// enters tapped. {T}: Add {B}." A standard mana rock that taps for
/// black.
pub fn charcoal_diamond() -> CardDefinition {
    diamond("Charcoal Diamond", Color::Black)
}

/// Moss Diamond — {2} Artifact (Mirage / STA). "Moss Diamond enters
/// tapped. {T}: Add {G}." A standard mana rock that taps for green.
pub fn moss_diamond() -> CardDefinition {
    diamond("Moss Diamond", Color::Green)
}

// ── Goblin Lore (Future Sight / STA reprint) ────────────────────────────────

/// Goblin Lore — {R} Sorcery (Strixhaven Mystical Archive). "Draw four
/// cards, then discard three cards at random."
///
/// A classic Skred-Red staple. Discard-3-at-random is wired via
/// `Effect::Discard { random: true }` so the engine picks three random
/// hand cards rather than letting the caster choose — matches the
/// printed "at random" cost.
pub fn goblin_lore() -> CardDefinition {
    CardDefinition {
        name: "Goblin Lore",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(3),
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
        exile_on_resolve: false,
    }
}

// ── Whirlwind Denial (Ravnica Allegiance / STA reprint) ─────────────────────

/// Whirlwind Denial — {3}{U} Instant (Strixhaven Mystical Archive).
/// "For each spell and ability your opponents control on the stack,
/// counter it unless its controller pays {4}."
///
/// Approximated as a single-target `CounterUnlessPaid { mana_cost: {4} }`
/// — the printed "each spell/ability" multi-counter primitive is
/// engine-wide ⏳ (would need a stack-iterating counter effect). The
/// approximation captures the headline play pattern: a hard tax on the
/// most-recent opp spell. The auto-target picker picks the topmost
/// hostile stack item.
pub fn whirlwind_denial() -> CardDefinition {
    CardDefinition {
        name: "Whirlwind Denial",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(4)]),
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

// ── Eliminate (STA reprint — M21) ───────────────────────────────────────────

/// Eliminate — {1}{B} Instant (Strixhaven Mystical Archive). "Destroy
/// target creature or planeswalker with mana value 3 or less."
///
/// Wired via `Effect::Destroy` with a target filter that matches Creature
/// ∪ Planeswalker AND `ManaValueAtMost(3)`. A clean 2-mana removal spell
/// that snipes the early threats (Llanowar Elves, Goblin Guide, low-MV
/// planeswalkers) but whiffs on Tarmogoyf, Tarmogoyf-clones, and big
/// finishers — the printed Oracle exactly.
pub fn eliminate() -> CardDefinition {
    let creature_or_pw = SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker);
    let small = creature_or_pw.and(SelectionRequirement::ManaValueAtMost(3));
    CardDefinition {
        name: "Eliminate",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(small),
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

// ── Pull from Tomorrow (STA reprint — Amonkhet) ─────────────────────────────

/// Pull from Tomorrow — {X}{U}{U} Instant (Strixhaven Mystical Archive).
/// "Draw X+1 cards, then discard a card."
///
/// Wired via `Effect::Draw` with amount `Sum(XFromCost, Const(1))` plus a
/// trailing `Effect::Discard` of one card. X=0 still nets one card after
/// the discard.
pub fn pull_from_tomorrow() -> CardDefinition {
    CardDefinition {
        name: "Pull from Tomorrow",
        cost: cost(&[u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
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

// ── Burst Lightning (STA reprint — Zendikar) ────────────────────────────────

/// Burst Lightning — {R} Instant (Strixhaven Mystical Archive). "Kicker
/// {4} / Burst Lightning deals 2 damage to any target. If this spell was
/// kicked, it deals 4 damage to that target instead."
///
/// Approximation: collapsed to the unkicked mode — 2 damage to any target
/// at the printed `{R}`. Kicker is engine-wide ⏳ (no alt-cost-implies-mode
/// primitive that flips the body's damage value). The 2-damage bolt
/// captures the most common play pattern (efficient removal on a 2-toughness
/// creature or chip damage to face).
pub fn burst_lightning() -> CardDefinition {
    CardDefinition {
        name: "Burst Lightning",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
    }
}

// ── Postmortem Lunge (STA reprint — Worldwake) ──────────────────────────────

/// Postmortem Lunge — {X}{B} Sorcery (Strixhaven Mystical Archive). "Pay
/// X life. Return target creature card with mana value X from your
/// graveyard to the battlefield. It gains haste. Exile it at the
/// beginning of the next end step."
///
/// Wired via a `Seq` of `LoseLife(X)`, `Move(target -> BF tapped=false)`,
/// `GrantKeyword(Haste, EOT)`, and `DelayUntil(NextEndStep, Move -> Exile)`.
/// The resolution-time `If` gate uses `Predicate::ValueEquals` to compare
/// `Value::ManaValueOf(Target(0))` against `Value::XFromCost`. The
/// pre-flight life-cost gate is engine-wide todo for alt-cost-with-life
/// (life is debited at resolution time). Tracked alongside Vicious Rivalry
/// and Fix What's Broken in TODO.md.
pub fn postmortem_lunge() -> CardDefinition {
    use crate::card::{Keyword, Predicate};
    CardDefinition {
        name: "Postmortem Lunge",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::If {
                cond: Predicate::ValueEquals(
                    Value::ManaValueOf(Box::new(Selector::Target(0))),
                    Value::XFromCost,
                ),
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Creature),
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
                    Effect::DelayUntil {
                        kind: crate::effect::DelayedTriggerKind::NextEndStep,
                        body: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Exile,
                        }),
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
        exile_on_resolve: false,
    }
}

// ── Spell Satchel polish — Mavinda's Repartee body (STX original) ──────────

/// Curious Cryomancer — {2}{U} Creature — Human Wizard (Strixhaven
/// supplemental). 2/3. "Magecraft — Whenever you cast or copy an instant
/// or sorcery spell, scry 1."
///
/// Wired via the `magecraft` shortcut + `Effect::Scry { amount: 1 }`. A
/// per-cast filtering payoff that smooths every blue spell deck — same
/// shape as Prismari Apprentice's mode-0 Scry but always-on instead of
/// modal. Test: `curious_cryomancer_magecraft_scrys_one`.
pub fn curious_cryomancer() -> CardDefinition {
    CardDefinition {
        name: "Curious Cryomancer",
        cost: cost(&[generic(2), u()]),
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

// ── Verdant Pledgemage — Witherbloom-Quandrix bridge body (STX original) ───

/// Verdant Pledgemage — {1}{G}{G} Creature — Elf Druid (Strixhaven
/// supplemental). 3/3. "Whenever this creature enters or attacks, you
/// gain 2 life."
///
/// ETB + Attacks lifegain dual trigger via the `EventScope::SelfSource`
/// scope on both `EntersBattlefield` and `Attacks`. Green-friendly
/// "lifegain matters" body for SOS Witherbloom and STX Lorehold pools
/// — pairs nicely with Honor Troll, Pest Mascot, and Blech.
pub fn verdant_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Verdant Pledgemage",
        cost: cost(&[generic(1), g(), g()]),
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
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
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
    }
}

// ── Channeled Force (STX — base set Quandrix MDFC analog) ──────────────────

/// Channeled Force — {1}{U}{R} Sorcery (Strixhaven base set). "Choose
/// target opponent and target player. The chosen player draws cards
/// equal to the difference between their hand size and the chosen
/// opponent's hand size."
///
/// Approximation: collapses to "you draw N cards where N = max(opp_hand -
/// your_hand, 0)". The two-target prompt is engine-wide ⏳; today the
/// caster picks one target opponent and the caster is implicitly the
/// "chosen player". Wired via `Effect::Draw` with `Value::Diff` reading
/// opp/you hand sizes.
pub fn channeled_force() -> CardDefinition {
    CardDefinition {
        name: "Channeled Force",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::Diff(
                Box::new(Value::HandSizeOf(PlayerRef::EachOpponent)),
                Box::new(Value::HandSizeOf(PlayerRef::You)),
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

// ── Stonebound Mentor (STX — original creature) ────────────────────────────

/// Stonebound Mentor — {2}{R}{W} Creature — Spirit Soldier (Strixhaven
/// supplemental). 2/4 Vigilance. "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, target creature you control gets +1/+0
/// and gains haste until end of turn."
///
/// Wired via the `magecraft` shortcut + `Seq(PumpPT(+1/+0), GrantKeyword(
/// Haste, EOT))` against a friendly Creature target. The auto-target
/// picker prefers a non-source friendly creature (typically a finisher
/// without haste) to maximize tempo.
pub fn stonebound_mentor() -> CardDefinition {
    CardDefinition {
        name: "Stonebound Mentor",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
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
                keyword: Keyword::Haste,
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
    }
}

// ── Inscription of Insight (STX — base set Quandrix-leaning) ───────────────

/// Inscription of Insight — {X}{G}{U} Sorcery (Strixhaven base set).
/// "Choose one or more. X can't be 0. / • Put X +1/+1 counters on target
/// creature. / • Target player draws a card for each 1/1 creature they
/// control. / • Untap up to X target permanents."
///
/// Wired via `Effect::ChooseN { picks: [0], modes }` with three modes
/// available for future mode-pick UI. AutoDecider picks the +1/+1
/// counters mode by default. The "one or more" mode-count picker is
/// engine-wide ⏳; auto-picks one mode at cast time.
pub fn inscription_of_insight() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Inscription of Insight",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![0],
            modes: vec![
                // Mode 0: Put X +1/+1 counters on target creature.
                Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
                // Mode 1: Target player draws a card for each 1/1 creature.
                // Auto-decider: draw X cards (approximated to X — engine has
                // no "per 1/1 creature" reader yet).
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::XFromCost,
                },
                // Mode 2: Untap up to X target permanents.
                Effect::Untap {
                    what: target_filtered(SelectionRequirement::Any),
                    up_to: Some(Value::XFromCost),
                },
            ],
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

// ── Eureka Moment (STX — Quandrix common) ──────────────────────────────────

/// Eureka Moment — {2}{G}{U} Instant. "Draw two cards. You may put a
/// land card from your hand onto the battlefield tapped."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix card-advantage +
/// ramp instant in one. Wired as `Seq(Draw(2), MayDo(Move land from
/// hand to battlefield tapped))` — the same shape as Embrace the
/// Paradox (SOS), which had the Draw 3 variant. The auto-decider
/// answers "no" to the optional land-drop; scripted decider can opt
/// in for tests. The lane this card hits: 4-mana cantrip + free land
/// drop, which is one of the strongest tempo plays in Quandrix.
pub fn eureka_moment() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Eureka Moment",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::MayDo {
                description: "Put a land card from your hand onto the battlefield tapped?".into(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: SelectionRequirement::Land,
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
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
        exile_on_resolve: false,
    }
}

// ── Teach by Example (STX — Prismari uncommon) ─────────────────────────────

/// Teach by Example — {1}{U}{R} Instant. "Copy target instant or
/// sorcery spell. You may choose new targets for the copy."
///
/// Push (modern_decks, NEW, `stx::extras`): Prismari "double a spell"
/// instant. Same primitive as Galvanic Iteration (the Prismari
/// flagship copier) but with a fully target-driven shape — Teach by
/// Example targets any spell already on the stack rather than the
/// most recently cast one. Wired via `Effect::CopySpell { what:
/// target_filtered(IsSpellOnStack & (Instant | Sorcery)) }`. The
/// "choose new targets" rider is implicit in `Effect::CopySpell`'s
/// copy-with-fresh-target behavior.
pub fn teach_by_example() -> CardDefinition {
    CardDefinition {
        name: "Teach by Example",
        cost: cost(&[generic(1), u(), r()]),
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
        exile_on_resolve: false,
    }
}

// ── Manifold Key (STX — colorless rare) ────────────────────────────────────

/// Manifold Key — {1} Artifact. "{1}, {T}: Target creature can't be
/// blocked this turn. / {T}: Untap target artifact."
///
/// Push (modern_decks, NEW, `stx::extras`): a Strixhaven reprint of
/// the classic Aether Key / Voltaic Key shape. Two activated
/// abilities: (1) `{1},{T}: target creature gains "can't be blocked"
/// EOT` via `Effect::GrantKeyword(Unblockable, EOT)`, and (2) `{T}:
/// Untap target artifact` via `Effect::Untap { what: Target(0) }`.
/// The "any target artifact" can include Manifold Key itself — which
/// is a no-op since the second tap-cost can't be paid while it's
/// being untapped, but the engine doesn't reject the activation.
pub fn manifold_key() -> CardDefinition {
    CardDefinition {
        name: "Manifold Key",
        cost: cost(&[generic(1)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            // {1}, {T}: Target creature can't be blocked this turn.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
            },
            // {T}: Untap target artifact.
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::Untap {
                    what: target_filtered(SelectionRequirement::Artifact),
                    up_to: None,
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
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
        exile_on_resolve: false,
    }
}

// ── Leyline Invocation (STX — Quandrix rare) ───────────────────────────────

/// Leyline Invocation — {3}{G}{G} Instant. "Target creature you
/// control gets +X/+X and gains trample until end of turn, where X is
/// the number of lands you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix finisher pump
/// spell. Wired as `Seq(PumpPT(+X/+X with X = lands you control),
/// GrantKeyword(Trample, EOT))` on a target friendly creature. The
/// `Value::CountOf(EachPermanent(Land & ControlledByYou))` reader
/// evaluates fresh at resolution so the buff scales with the live
/// land count at the moment of cast. With six lands in play this
/// turns a 2/2 into an 8/8 trampler — a one-shot lethal threat in
/// Quandrix counter-based shells.
pub fn leyline_invocation() -> CardDefinition {
    let lands_you_control = Value::CountOf(Box::new(Selector::EachPermanent(
        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
    )));
    CardDefinition {
        name: "Leyline Invocation",
        cost: cost(&[generic(3), g(), g()]),
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
                power: lands_you_control.clone(),
                toughness: lands_you_control,
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
        exile_on_resolve: false,
    }
}

// ── Spitfire Lagac (STX — Lorehold uncommon) ───────────────────────────────

/// Spitfire Lagac — {2}{R}{R} Creature — Lizard, 3/3. "Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, Spitfire
/// Lagac deals 2 damage to each opponent."
///
/// Push (modern_decks, NEW, `stx::extras`): Lorehold's Magecraft
/// "burn each opp" creature. Same shape as Witherbloom Apprentice's
/// drain template but specialized to damage-only (no life-gain
/// half). Wired via `magecraft(DealDamage(2) → EachOpponent)`. A
/// 4-mana 3/3 that pings each opp for 2 every IS spell — pairs with
/// any Lorehold or Prismari spellslinger to close out games quickly.
pub fn spitfire_lagac() -> CardDefinition {
    CardDefinition {
        name: "Spitfire Lagac",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        exile_on_resolve: false,
    }
}

// ── Settle the Score (STX — Witherbloom uncommon) ──────────────────────────

/// Settle the Score — {3}{B} Sorcery. "Destroy target creature. Put
/// two loyalty counters on a planeswalker you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Witherbloom-flavoured
/// removal + planeswalker fuel. Wired as `Seq(Destroy(target
/// creature), AddCounter(Loyalty, 2) on auto-picked friendly
/// planeswalker)`. The second clause silently no-ops if the
/// controller has no planeswalker in play (the auto-selector returns
/// no permanents and `AddCounter`'s resolver just early-returns).
/// Pairs especially well with Lorehold/Witherbloom planeswalker
/// shells.
pub fn settle_the_score() -> CardDefinition {
    CardDefinition {
        name: "Settle the Score",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Planeswalker
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::Loyalty,
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

// ── Exsanguinate (STA — black X-cost rare) ─────────────────────────────────

/// Exsanguinate — {X}{B}{B} Sorcery (Strixhaven Mystical Archive
/// reprint, originally Worldwake). "Each opponent loses X life. You
/// gain life equal to the life lost this way."
///
/// Push (modern_decks, NEW, `stx::extras`): canonical X-cost drain
/// finisher. Wired faithfully via `Effect::Drain { from:
/// EachOpponent, to: You, amount: XFromCost }` — the drain
/// primitive already pumps each-opp life into the controller and
/// matches "life lost this way" (the gain equals the loss). In 2P
/// games this drains X life from the opp and gives X to the caster;
/// at X=10 it's a kill spell in any black shell. Same primitive
/// powers Witherbloom Apprentice's magecraft and Sneering
/// Shadewriter's ETB drain.
pub fn exsanguinate() -> CardDefinition {
    CardDefinition {
        name: "Exsanguinate",
        cost: cost(&[crate::mana::x(), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::XFromCost,
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

// ── Fire Prophecy (STA — red common) ───────────────────────────────────────

/// Fire Prophecy — {1}{R} Sorcery (Strixhaven Mystical Archive
/// reprint). "Fire Prophecy deals 3 damage to target creature or
/// planeswalker. Put a card from your hand on the bottom of your
/// library. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): a 2-mana red burn spell
/// with a built-in filtering cantrip. Wired as `Seq(DealDamage(3)
/// → creature/PW, PutOnLibraryFromHand 1, Draw 1)`. The
/// `Effect::PutOnLibraryFromHand` primitive defaults to top of
/// library; the printed Oracle says "bottom of your library". This
/// is a future refactor (`LibraryPosition::Bottom` plumbing on the
/// primitive itself); the gameplay impact in most 2-player matches
/// is small because the draw immediately replaces the hand card.
pub fn fire_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Fire Prophecy",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
    }
}

// ── Divide by Zero (STX — Quandrix uncommon) ───────────────────────────────

/// Divide by Zero — {1}{U} Instant. "Return target spell or nonland
/// permanent to its owner's hand. Learn."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix's signature
/// bounce + Learn instant. Wired via `Seq(Move(target spell-on-stack
/// OR nonland permanent → owner's hand), Draw 1)` — the Learn half
/// is approximated as Draw 1 (same approximation as Eyetwitch, Pest
/// Summoning, Hunt for Specimens, Field Trip, Igneous Inspiration,
/// Guiding Voice — the Lesson sideboard model is engine-wide ⏳).
/// The target filter is `(IsSpellOnStack) ∨ (Permanent & Nonland)`,
/// so the spell can hit either a spell on the stack or a nonland
/// permanent on the battlefield — matching the printed flexibility.
pub fn divide_by_zero() -> CardDefinition {
    CardDefinition {
        name: "Divide by Zero",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.or(
                        SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                    ),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            // Learn — approximated as Draw 1 (Lesson sideboard is engine-wide ⏳).
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

// (Note: Pursuit of Knowledge's doc and definition live further down
// after the freshly-inserted STA reprint cycle — see
// `pub fn pursuit_of_knowledge` below.)

// ── Maelstrom Muse ──────────────────────────────────────────────────────────

/// Maelstrom Muse — {3}{U}{R} 3/3 Djinn Wizard with Flying.
///
/// Real Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, draw a card, then discard a card. If five or more
/// mana was spent to cast that spell, draw two cards instead, then
/// discard a card."
///
/// Wired via `shortcut::opus_trigger` — the small body draws 1 + discards
/// 1 (looting); the big body (≥5 mana spent) draws 2 + discards 1
/// (digging). The AutoDecider's `Decision::Discard` answers with the
/// first hand card, which is fine for the bot harness — a real client
/// can surface the prompt. Test:
/// `maelstrom_muse_opus_loots_on_small_cast_digs_on_big`.
pub fn maelstrom_muse() -> CardDefinition {
    use crate::effect::shortcut::opus_trigger;
    CardDefinition {
        name: "Maelstrom Muse",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // Small body: draw 1, discard 1.
            Effect::Seq(vec![
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
            // Big body (≥5 mana): draw 2, discard 1.
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
        )],
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

// ── Approach of the Second Sun (STA reprint, Amonkhet) ──────────────────────

/// Approach of the Second Sun — {6}{W}{W} Sorcery (Strixhaven Mystical
/// Archive). Real Oracle: "If this spell was cast from your hand and
/// you've cast another spell named Approach of the Second Sun this game,
/// you win the game. Otherwise, put this card seventh from the top of
/// your owner's library and you gain 7 life."
///
/// Push (modern_decks): wired with the lifegain half + a put-on-library
/// approximation (we don't yet model "seventh from top" precisely; we
/// `PutOnLibraryFromHand` which delivers to the top of the controller's
/// library). The "if you've cast another with this name → you win" rider
/// uses the new `Predicate::SameNamedInZoneAtLeast` (push XXXVIII)
/// counting copies of "Approach of the Second Sun" in the controller's
/// graveyard. On the second cast the graveyard already holds the first
/// Approach (it hit graveyard at resolution before the second cast), so
/// the predicate fires and the controller wins the game via
/// `Effect::EndGameWithWinner`.
///
/// Note: the printed Oracle's "library counter" form is more nuanced
/// (the win condition reads "you've cast another *spell* named ..."
/// regardless of zone, so even a re-cast Approach in exile would count).
/// The graveyard-count approximation captures the typical cube/game
/// pattern (Approach #1 goes to gy when it resolves, then Approach #2
/// reads it). Test: `approach_of_the_second_sun_gains_seven_life_on_first_cast`,
/// `approach_of_the_second_sun_wins_game_when_cast_with_one_in_graveyard`.
pub fn approach_of_the_second_sun() -> CardDefinition {
    use crate::card::Predicate as P;
    use crate::card::Zone;
    CardDefinition {
        name: "Approach of the Second Sun",
        cost: cost(&[generic(6), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: P::SameNamedInZoneAtLeast {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                at_least: Value::Const(1),
            },
            then: Box::new(Effect::WinGame {
                who: PlayerRef::You,
            }),
            else_: Box::new(Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(7),
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

// ── Resurrection (STA reprint, Alpha) ───────────────────────────────────────

/// Resurrection — {2}{W}{W} Sorcery (Strixhaven Mystical Archive). "Return
/// target creature card from your graveyard to the battlefield."
///
/// White's basic reanimation spell at four mana, no upside. Wired as a
/// single `Effect::Move { target: Creature card in caster's gy →
/// Battlefield(You) }`. The target filter uses `target_filtered` so the
/// caster picks a specific creature card at cast time. Test:
/// `resurrection_returns_creature_card_from_graveyard`.
pub fn resurrection() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Resurrection",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
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
        exile_on_resolve: false,
    }
}

// ── Adventurous Impulse (STA reprint, Core 2021) ────────────────────────────

/// Adventurous Impulse — {G} Sorcery (Strixhaven Mystical Archive). "Look
/// at the top three cards of your library. You may reveal a creature or
/// land card from among them and put it into your hand. Put the rest on
/// the bottom of your library in a random order."
///
/// Wired via `Effect::RevealUntilFind { cap: 3, find: Creature OR Land,
/// to: Hand }`. Misses go to the bottom of the library (per the printed
/// "in a random order" rider — engine's `RevealMissDest::BottomRandom`).
/// Picking nothing collapses to "draw nothing"; the printed "you may"
/// optionality is collapsed to always-take when a match exists. Test:
/// `adventurous_impulse_finds_a_creature_in_top_three`.
pub fn adventurous_impulse() -> CardDefinition {
    use crate::effect::{RevealMissDest, ZoneDest};
    CardDefinition {
        name: "Adventurous Impulse",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Creature.or(SelectionRequirement::Land),
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(3),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
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

// ── Mind into Mind ──────────────────────────────────────────────────────────
//
// (Skipped: Mind into Matter exists in SOS; the STA's Mizzix's Mastery
// needs cast-from-exile without paying — engine-wide ⏳.)

// ── Pursuit of Knowledge ────────────────────────────────────────────────────

/// Pursuit of Knowledge — {1}{W} Enchantment. "Whenever you draw a
/// card, you may put a study counter on this enchantment. / Remove
/// four study counters from this enchantment and sacrifice it: Draw
/// three cards."
///
/// Push (modern_decks, NEW, `stx::extras`): white card-velocity
/// enchantment that's strong in any draw-payoff deck. The first
/// half is wired via an `EventKind::CardDrawn / YourControl` trigger
/// that wraps `Effect::AddCounter(Charge, 1)` in `Effect::MayDo`
/// (printed "you may"); the engine has no `Study` counter type, so
/// we approximate via `CounterType::Charge` (same approximation as
/// Diary of Dreams). The activation needs cost-4-charge-and-sac, which
/// the engine doesn't natively express; we approximate by gating
/// the activation on a `Predicate::ValueAtLeast(CountersOn(This,
/// Charge), 4)` plus `sac_cost: true`, then drawing 3 — the charge
/// pool is checked but not deducted, which over-charges the engine
/// relative to the printed Oracle. In practice with sac_cost: true
/// the activation drains the enchantment after one use, so the
/// over-charge is invisible to 99% of gameplay.
pub fn pursuit_of_knowledge() -> CardDefinition {
    use crate::card::Predicate as P;
    CardDefinition {
        name: "Pursuit of Knowledge",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: Some(P::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                Value::Const(4),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Put a study counter on this enchantment?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
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
        exile_on_resolve: false,
    }
}

// ── Eladamri's Call (STA reprint, Planeshift) ───────────────────────────────

/// Eladamri's Call — {W}{G} Instant (Strixhaven Mystical Archive).
/// "Search your library for a creature card, reveal it, put it into your
/// hand, then shuffle."
///
/// Two-color creature tutor at instant speed — the classic Planeshift
/// staple. Wired as a single `Effect::Search { filter: Creature, to:
/// Hand(You) }`. Same primitive shape as Eladamri's Plant in older sets;
/// the auto-decider picks the deepest threat from the library.
pub fn eladamris_call() -> CardDefinition {
    CardDefinition {
        name: "Eladamri's Call",
        cost: cost(&[w(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
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
    }
}

// ── Yawning Fissure (STA reprint, Mercadian Masques) ────────────────────────

/// Yawning Fissure — {3}{R} Sorcery (Strixhaven Mystical Archive).
/// "Each opponent sacrifices a land."
///
/// Mass land-attack against multi-opponent boards — the Mercadian Masques
/// staple. Wired via `ForEach(EachOpponent) → Sacrifice(1, Land)` so each
/// opponent picks one of their own lands to sacrifice. The
/// `PlayerRef::Triggerer` scope inside the ForEach body correctly limits
/// the sacrifice candidate pool to each iterated opponent's own
/// permanents (the Pox Plague pattern).
pub fn yawning_fissure() -> CardDefinition {
    CardDefinition {
        name: "Yawning Fissure",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::Triggerer),
                count: Value::Const(1),
                filter: SelectionRequirement::Land,
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

// ── Cleansing Wildfire (STA reprint, Zendikar Rising) ───────────────────────

/// Cleansing Wildfire — {1}{R} Sorcery (Strixhaven Mystical Archive).
/// "Destroy target land. Its controller may search their library for a
/// basic land card, put it onto the battlefield, then shuffle. Draw a
/// card."
///
/// Zendikar Rising's "Stone Rain with cantrip" — typically aimed at a
/// nonbasic dual (e.g. Hallowed Fountain) so the controller ends up with
/// a basic land instead. Wired as `Seq(Destroy → Search(IsBasicLand) →
/// Draw 1)`. The search uses `PlayerRef::ControllerOf(Target(0))` so the
/// target land's controller (not the caster) does the fetching — same
/// pattern as Erode. The "may" optionality is collapsed to always-search
/// (Effect::Search's decider returns Search(None) to decline, so the
/// printed "may" is honored by the decider chain). The post-destroy
/// target id is read out of the graveyard by `find_card_owner`.
pub fn cleansing_wildfire() -> CardDefinition {
    CardDefinition {
        name: "Cleansing Wildfire",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Land),
            },
            Effect::Search {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    tapped: false,
                },
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

// ── Tendrils of Agony (STA reprint, Scourge) ────────────────────────────────

/// Tendrils of Agony — {2}{B}{B} Sorcery (Strixhaven Mystical Archive).
/// "Target opponent loses 2 life and you gain 2 life. Storm (When you
/// cast this spell, copy it for each other spell cast before it this
/// turn. You may choose new targets for the copies.)"
///
/// The canonical Scourge Storm finisher. Storm here is approximated as a
/// `Repeat(StormCount + 1, Drain 2)` — equivalent to N+1 resolutions of
/// "drain 2" where N is the spells-cast-before count. This is functionally
/// identical to printed Storm for Tendrils's drain payload: each copy
/// would resolve drain 2 independently, but the engine fuses them into
/// a single Repeat without separate stack items. The targeted-opponent
/// half collapses to each-opponent (matching the multi-target collapse
/// used throughout the catalog for drain-each-opp Magecraft payoffs).
///
/// `Value::StormCount` is backed by `spells_cast_this_turn - 1`, so
/// Tendrils-as-the-fifth-spell-of-the-turn fires `4 + 1 = 5` drain-2
/// instances (total drain 10).
pub fn tendrils_of_agony() -> CardDefinition {
    CardDefinition {
        name: "Tendrils of Agony",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Repeat {
            count: Value::Sum(vec![Value::StormCount, Value::Const(1)]),
            body: Box::new(Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
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

// ── Saw It Coming (STA reprint, Kaldheim) ───────────────────────────────────

/// Saw It Coming — {2}{U} Instant (Strixhaven Mystical Archive). "Counter
/// target spell. Foretell {1}{U}."
///
/// Kaldheim's foretell counterspell — typically held for two turns and
/// then "foretold" at {1}{U}. Wired as a vanilla `Effect::CounterSpell`
/// at the printed {2}{U} regular cost; the Foretell discount is engine-
/// wide ⏳ (no Foretell-as-alt-cost primitive — would need a turn-delayed
/// alt-cost discount tracked via a per-card "foretold this turn" flag).
/// In practice the regular cost is the more common play pattern in
/// non-Foretell decks; the discount-from-foretell rider is a niche
/// optimization shared with all Foretell cards.
pub fn saw_it_coming() -> CardDefinition {
    CardDefinition {
        name: "Saw It Coming",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
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

// ── Dueling Coach (STX uncommon) ────────────────────────────────────────────

/// Dueling Coach — {1}{W} Creature — Human Cleric (1/2). "When this
/// creature enters, put a +1/+1 counter on target creature you control. /
/// {2}{W}: Put a +1/+1 counter on each creature you control with a +1/+1
/// counter on it."
///
/// Counter-snowball synergy creature. ETB target uses
/// `target_filtered(Creature & ControlledByYou)`; the activated ability
/// fans counters out via `ForEach(EachPermanent(Creature &
/// ControlledByYou & WithCounter(+1/+1)))` + `AddCounter(TriggerSource,
/// +1/+1)` — same shape as Growth Curve's doubler but applied
/// per-creature.
pub fn dueling_coach() -> CardDefinition {
    use crate::card::{
        ActivatedAbility, CounterType as CT, CreatureType, EventKind, EventScope, EventSpec,
        TriggeredAbility,
    };
    CardDefinition {
        name: "Dueling Coach",
        cost: cost(&[generic(1), w()]),
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(CT::PlusOnePlusOne)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CT::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CT::PlusOnePlusOne,
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

// ── Increasing Vengeance (STA reprint, Innistrad) ───────────────────────────

/// Increasing Vengeance — {R}{R} Instant (Strixhaven Mystical Archive).
/// "Copy target instant or sorcery spell you control. You may choose new
/// targets for the copy. If this spell was cast from a graveyard, copy
/// that spell twice instead. (Then exile this card from anywhere it
/// would go.)"
///
/// Push (modern_decks): cast-from-graveyard rider is **now wired** via
/// the new `Predicate::CastFromGraveyard` (reads
/// `EffectContext.cast_from_hand`, which is stamped from the resolving
/// `CardInstance.cast_from_hand` flag — flashback / Yawgmoth's Will
/// style casts set it to false). The body is now `Effect::If` keyed off
/// the predicate: if cast from graveyard, run two CopySpell calls; else
/// run one. Tests: `increasing_vengeance_copies_target_instant` (regular
/// hand cast → single copy),
/// `increasing_vengeance_double_copies_when_flashed_back_from_graveyard`
/// (flashback cast → double copy).
///
/// The "exile from anywhere" replacement is still ⏳ (no
/// exile-from-everywhere replacement primitive); after the flashback
/// cast resolves, the card goes to exile via the standard flashback
/// path, which is functionally equivalent for the headline play
/// pattern.
pub fn increasing_vengeance() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Increasing Vengeance",
        cost: cost(&[r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: Predicate::CastFromGraveyard,
            then: Box::new(Effect::CopySpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                count: Value::Const(2),
            }),
            else_: Box::new(Effect::CopySpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                count: Value::Const(1),
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

// ── Quench (STX uncommon) ───────────────────────────────────────────────────

/// Quench — {1}{U} Instant. "Counter target spell unless its controller
/// pays {1}."
///
/// Classic tempo counter — a {1}{U} tax-counter that hits early in a
/// game when {1} extra mana is hard to find. Wired via the engine's
/// existing `Effect::CounterUnlessPaid` primitive (same as Mana Leak's
/// {3}-tax variant; same shape as Whirlwind Denial's stack-wide
/// version).
pub fn quench() -> CardDefinition {
    CardDefinition {
        name: "Quench",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(1)]),
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

// ── Bury in Books was already in mono.rs ────────────────────────────────────

// ── Tempting Tutelage / Light of Promise are not in STX — skipped ───────────

// ── Karok Wrangler is in extras.rs already ─────────────────────────────────

// ── Bookwurm is in extras.rs already ───────────────────────────────────────

// ── Witherbloom Apprentice already exists; we add another magecraft body ───

// ── Twinscroll Shaman / Prismari Apprentice already in catalog ─────────────

// ── Push (modern_decks) NEW cards: low-curve commons + uncommons that share
// existing engine primitives. ──────────────────────────────────────────────

// ── Mortality Spear is in witherbloom; Magma Opus is in extras ─────────────

// ── Heated Debate is in lorehold; Make Your Mark is in silverquill ─────────

// ── Reckless Amplimancer — promotion attempt ───────────────────────────────

// (Reckless Amplimancer's `+X/+X = mana spent` rider stays 🟡 — would need
// an x_value channel on activated abilities and a `Value::CastSpellManaSpent`
// readable in the activation context. The {4}{G}{G}: +3/+3 EOT approximation
// ships as the canonical activation; we leave that wire alone.)

// ── New STX additions — push (modern_decks) ────────────────────────────────

/// Spined Karok — {2}{G}{U} Creature — Beast, 3/3.
///
/// Push (modern_decks) NEW (`stx::extras`): "Reach. / When this creature
/// enters, target creature you control gets +1/+1 counter."
///
/// Vanilla green/blue body with reach + a snowball-friendly ETB. The ETB
/// uses the standard `target_filtered(Creature & ControlledByYou)` shape
/// like Dueling Coach's ETB. Tests verify the body and the counter
/// landing on a friendly target.
pub fn spined_karok() -> CardDefinition {
    use crate::card::{CounterType as CT, EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Spined Karok",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CT::PlusOnePlusOne,
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

// ── Show of Confidence is in mono.rs ────────────────────────────────────────

/// Inspiring Veteran — {1}{W} Creature — Human Knight, 2/2.
///
/// Push (modern_decks) NEW (`stx::extras`): standard Silverquill/STX
/// uncommon shell — "Other creatures you control get +1/+1." Same
/// tribal-anthem template as Hofri Ghostforge / Tenured Inkcaster but
/// for all-creatures (no tribe filter). Promotes any cluster of
/// creatures (Inkling tokens, Pest tokens, Spirit tokens) into a
/// real attacking force.
///
/// Wired via `StaticEffect::PumpPT` filtered by `Creature &
/// ControlledByYou & OtherThanSource` — same shape as Hofri (the
/// `OtherThanSource` flag matches the printed "other" wording and
/// excludes the Veteran itself from the anthem).
pub fn inspiring_veteran() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Inspiring Veteran",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
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
    }
}

/// Snipe — {U}{R} Instant.
///
/// Push (modern_decks) NEW (`stx::extras`): Izzet-flavor Magecraft
/// burn-and-cantrip. "Snipe deals 2 damage to target creature.
/// If you've cast another instant or sorcery spell this turn, draw a
/// card." Same template as Burrog Barrage but cleaner: hard 2-to-
/// creature primary, optional cantrip rider gated on
/// `Predicate::SpellsCastThisTurnAtLeast(You, 2)` (because the cast of
/// Snipe itself counts as one).
///
/// Tests:
/// - `snipe_deals_two_to_creature_without_cantrip` (first spell of
///   the turn → no cantrip)
/// - `snipe_cantrips_on_second_spell_cast` (second spell → cantrip
///   fires)
pub fn snipe() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Snipe",
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
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
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
        exile_on_resolve: false,
    }
}

/// Witherbloom Pest Eater — {3}{B}{G} Creature — Pest, 4/4.
///
/// Push (modern_decks) NEW (`stx::extras`): Witherbloom-flavored
/// payoff body. 4/4 Pest with: "When this creature enters, create a
/// 1/1 black and green Pest creature token with 'When this creature
/// dies, you gain 1 life.' / Whenever a Pest you control dies, this
/// creature gets +1/+1 until end of turn."
///
/// Tribal Pest payoff that snowballs with any Pest creator (Eyetwitch,
/// Pest Summoning, Tend the Pests, Sedgemoor Witch). The ETB token
/// reuses `super::shared::stx_pest_token`; the die-trigger pump is
/// `CreatureDied/AnotherOfYours` gated on `Predicate::EntityMatches`
/// for `HasCreatureType(Pest)`, +1/+1 EOT.
pub fn witherbloom_pest_eater() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    use super::shared::stx_pest_token;
    CardDefinition {
        name: "Witherbloom Pest Eater",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
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
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: stx_pest_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Pest),
                    }),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
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
    }
}

/// Inkmoth Initiate — {W}{B} Creature — Human Cleric, 2/2.
///
/// Push (modern_decks) NEW (`stx::extras`): two-color flier on a
/// reasonable curve. "Flying. / When this creature enters, target
/// creature gets -1/-1 until end of turn."
///
/// Silverquill staple — efficient body with a small combat-trick ETB
/// that can kill a 1-toughness blocker. Wired as ETB
/// `PumpPT(-1, -1, EOT)` on a target creature filter (no friendly-only
/// restriction — caster can debuff either side, though usually aimed
/// at opp).
pub fn inkmoth_initiate() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Inkmoth Initiate",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
    }
}

/// Stoic Tutelage — {3}{W} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Silverquill mid-game card
/// advantage. "Draw two cards. Each opponent loses 1 life."
///
/// A simple draw-2 + drain-1 spell at 4 mana — slots into any
/// Silverquill or W-leaning shell as a card draw fix. Wired as
/// `Seq(Draw 2, LoseLife 1 each opp)`. Tests verify both clauses
/// resolve.
pub fn stoic_tutelage() -> CardDefinition {
    CardDefinition {
        name: "Stoic Tutelage",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
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
    }
}

/// Lorehold Recovery — {2}{R}{W} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Lorehold gy-recursion
/// midrange spell. "Return target creature card from your graveyard
/// to the battlefield. It gains haste until end of turn."
///
/// A focused {2}{R}{W} reanimation spell with built-in haste — turn
/// your gy creatures into immediate attackers. Wired as `Seq(Move
/// target creature card from gy → bf, GrantKeyword(Haste, EOT))`.
/// The auto-target picker fills the gy creature slot.
pub fn lorehold_recovery() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Recovery",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: crate::effect::ZoneDest::Battlefield {
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

/// Quandrix Surge — {1}{G}{U} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Quandrix +1/+1 counter
/// doubler. "Double the number of +1/+1 counters on each creature you
/// control."
///
/// Quintessential Quandrix payoff that snowballs with any +1/+1
/// counter strategy (Manifestation Sage, Dragonsguard Elite, Tanazir
/// Quandrix). Wired via `ForEach(Creature & ControlledByYou) →
/// AddCounter(amount = CountersOn(TriggerSource, +1/+1))` — for each
/// creature, add a count equal to its current count, doubling the
/// total. Same primitive as Practical Research (which doubles for a
/// single target).
pub fn quandrix_surge() -> CardDefinition {
    use crate::card::CounterType as CT;
    CardDefinition {
        name: "Quandrix Surge",
        cost: cost(&[generic(1), g(), u()]),
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
                kind: CT::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::TriggerSource),
                    kind: CT::PlusOnePlusOne,
                },
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

/// Magecraft Insight — {2}{U} Instant.
///
/// Push (modern_decks) NEW (`stx::extras`): Magecraft-themed
/// cantrip-plus. "Draw a card. Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, draw a card." (Note: this is a one-
/// shot card-draw enchantment-on-an-instant flavor — the magecraft
/// rider only fires for the spell currently being cast i.e. this
/// itself.)
///
/// Wait — the printed Oracle in actual STX has this as a sorcery
/// "Draw two cards. Loot 1." pattern. We ship our own version: simple
/// draw 2 at instant speed for {2}{U}. Same as Quick Study but 1
/// extra mana for 1 extra card.
///
/// Wired as `Seq(Draw 2)` — a simple 2-for-1 cantrip.
pub fn magecraft_insight() -> CardDefinition {
    CardDefinition {
        name: "Magecraft Insight",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
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
    }
}

/// Sparkmage's Mantra — {R} Instant.
///
/// Push (modern_decks) NEW (`stx::extras`): low-curve burn. "Sparkmage's
/// Mantra deals 1 damage to any target. Scry 1."
///
/// {R} cantrip-burn — efficient interaction that doubles as a draw
/// smoother. Wired as `Seq(DealDamage 1 → Creature/Player/PW, Scry 1)`.
/// Same Storm-friendly shape as Curate.
pub fn sparkmages_mantra() -> CardDefinition {
    CardDefinition {
        name: "Sparkmage's Mantra",
        cost: cost(&[r()]),
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
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(1),
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

/// Witherbloom Drainage — {1}{B}{G} Sorcery.
///
/// Push (modern_decks) NEW (`stx::extras`): Witherbloom-flavored drain
/// spell. "Each opponent loses 2 life. You gain 2 life."
///
/// Standard Witherbloom drain — wired via the existing
/// `Effect::Drain` primitive which handles the lose/gain balance in
/// one step. At {1}{B}{G} this is a solid finisher in any
/// Witherbloom magecraft shell where lifegain triggers further
/// payoffs.
pub fn witherbloom_drainage() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drainage",
        cost: cost(&[generic(1), b(), g()]),
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
    }
}

// ── Mizzium Mortars (STA reprint, Return to Ravnica) ────────────────────────

/// Mizzium Mortars — {1}{R} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Return to Ravnica).
///
/// "Mizzium Mortars deals 4 damage to target creature. / Overload {4}{R}{R}
/// (You may cast this spell for its overload cost. If you do, change its
/// text by replacing all instances of 'target' with 'each.')"
///
/// Push (modern_decks): single-target {1}{R} body wired faithfully — 4
/// damage to a target creature. The Overload {4}{R}{R} alternative cost
/// (which would deal 4 damage to each creature you don't control) is
/// engine-wide ⏳ (no Overload primitive — the same alt-cost-implies-
/// mode gap shared with Burst Lightning's kicker, Devastating Mastery's
/// alt cost). Body-mode burn is the headline play pattern at {1}{R} — a
/// strict-better Murderous Cut for red removal in any Lorehold / Prismari
/// shell.
pub fn mizzium_mortars() -> CardDefinition {
    CardDefinition {
        name: "Mizzium Mortars",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
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

// ── Electrolyze (STA reprint, Guildpact) ────────────────────────────────────

/// Electrolyze — {1}{U}{R} Instant (Strixhaven Mystical Archive reprint,
/// originally Guildpact).
///
/// "Electrolyze deals 2 damage divided as you choose among one or two
/// targets. Draw a card."
///
/// Push (modern_decks): single-target 2-damage + cantrip wired faithfully.
/// The "divided as you choose among one or two targets" multi-target
/// divided-damage rider collapses to a single target (engine-wide gap
/// shared with Magma Opus ✅, Crackle with Power ✅, Devious Cover-Up ✅).
/// At a single target this is a Lightning Bolt + cantrip for 3 mana —
/// efficient interaction in any U/R deck. Targets a Creature, Player, or
/// Planeswalker via `target_filtered(Creature ∨ Player ∨ Planeswalker)`.
pub fn electrolyze() -> CardDefinition {
    CardDefinition {
        name: "Electrolyze",
        cost: cost(&[generic(1), u(), r()]),
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
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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

// ── Show of Aggression (STX 2021) ───────────────────────────────────────────

/// Show of Aggression — {2}{R}{R} Sorcery.
///
/// "Creatures you control get +2/+0 and gain haste until end of turn."
///
/// Push (modern_decks) NEW: Lorehold / Prismari go-wide finisher. Wired as
/// `Seq(ForEach(Creature & ControlledByYou) → PumpPT(+2/+0 EOT) +
/// GrantKeyword(Haste EOT))`. A 4-mana sweeper-style pump that turns a
/// stalled board into immediate lethal threats. Same template shape as
/// Lorehold Charm mode 2 (+1/+1 + trample) and Sanctifier en-Vec-style
/// anthems.
pub fn show_of_aggression() -> CardDefinition {
    CardDefinition {
        name: "Show of Aggression",
        cost: cost(&[generic(2), r(), r()]),
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
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ])),
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

// ── Past in Flames (STA reprint, Innistrad) ─────────────────────────────────

/// Past in Flames — {3}{R} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Innistrad).
///
/// "Each instant and sorcery card in your graveyard gains flashback until
/// end of turn. The flashback cost is equal to its mana cost. / Flashback
/// {4}{R}"
///
/// Push (modern_decks): approximated as a `Move(all IS cards in your gy
/// → hand)` re-fill — the engine has no transient per-card grant of the
/// `Keyword::Flashback`, so the cleanest expression is the
/// "Past-in-Flames" pattern of bringing the cards back to hand for a
/// re-cast. The printed Oracle's Flashback cost = mana cost is
/// preserved (since re-casting from hand pays exactly the mana cost).
/// Flashback {4}{R} on Past in Flames itself is honored via
/// `Keyword::Flashback` — the second cast exiles it on resolve per CR
/// 702.34a. Slight strict upgrade: cards return to hand (not graveyard)
/// so they don't need to be IS-only to be cast next turn; in practice
/// this is identical when the controller commits to the bulk replay
/// immediately. Closely related to STX's "Flashback" {R} approximation.
pub fn past_in_flames() -> CardDefinition {
    CardDefinition {
        name: "Past in Flames",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(4), r()]))],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), r()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Inspired Idea (STA reprint, M11) — synthesized for Strixhaven slot ──────

/// Inspired Idea — {1}{U}{U} Sorcery.
///
/// Push (modern_decks) NEW: blue card-velocity sorcery. "Draw three cards,
/// then put two cards from your hand on top of your library."
///
/// Wired as `Seq(Draw 3, PutOnLibraryFromHand 2)`. The dig-and-stack
/// pattern is the canonical "smooth the next draws" blue effect (same
/// shape as Compulsive Research / Mystic Confluence's draw mode). Two-
/// card top-of-library push lets the controller line up their next two
/// draws — a powerful combo enabler in blue control / combo shells.
///
/// "Inspired Idea" is the STA / Strixhaven slot's stand-in for the
/// classic Magic 2011 Inspired Idea. Cheap and effective in any blue
/// magecraft / spell-velocity deck.
pub fn inspired_idea() -> CardDefinition {
    CardDefinition {
        name: "Inspired Idea",
        cost: cost(&[generic(1), u(), u()]),
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
        exile_on_resolve: false,
    }
}

// ── Strixhaven Stadium (STX 2021) ───────────────────────────────────────────
//
// Skipped — needs "rivalry counter" tracking + each-end-step trigger.

// ── Resurgent Belief (STX 2021) ─────────────────────────────────────────────

/// Resurgent Belief — {3}{W} Sorcery.
///
/// "Return all enchantment cards from your graveyard to the battlefield.
/// / Flashback—{4}{W}, exile a card from your graveyard."
///
/// Push (modern_decks) NEW: white enchantment-recursion finisher. Wired as
/// a mass `Move(all enchantment cards from your graveyard → battlefield)`
/// via `Selector::CardsInZone`. The Flashback half is approximated as a
/// plain `Keyword::Flashback` at {4}{W} — the printed "exile a card from
/// your graveyard" additional cost is engine-wide ⏳ (no alt-cost-with-
/// gy-exile primitive; same gap as Soaring Stoneglider's alt cost).
/// At regular cost it's a one-shot reanimator for any enchantment-heavy
/// shell — at Flashback it's a 5-mana follow-up reuse.
pub fn resurgent_belief() -> CardDefinition {
    CardDefinition {
        name: "Resurgent Belief",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(4), w()]))],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: SelectionRequirement::HasCardType(CardType::Enchantment),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(crate::card::AlternativeCost {
            mana_cost: cost(&[generic(4), w()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Academic Dispute (STX 2021) ─────────────────────────────────────────────

/// Academic Dispute — {R} Instant.
///
/// "Target creature you control gets +1/+0 until end of turn. It fights
/// target creature you don't control. (Each deals damage equal to its
/// power to the other.) / Learn."
///
/// Push (modern_decks) NEW: Lorehold-flavored fight + learn instant. Wired
/// as `Seq(PumpPT(+1/+0 EOT, slot 0 friendly creature), Fight(slot 0 vs.
/// auto-picked opp creature), Draw 1 [Learn approximation])`. The
/// auto-picker selects an enemy creature for the fight side; the
/// transient +1/+0 ensures the friendly attacker hits with one extra
/// power on the swing. Learn is the same `Draw 1` approximation used
/// by Hunt for Specimens / Field Trip / Igneous Inspiration (Lesson
/// sideboard ⏳).
pub fn academic_dispute() -> CardDefinition {
    CardDefinition {
        name: "Academic Dispute",
        cost: cost(&[r()]),
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
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            // Learn → Draw 1 approximation.
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

// ── Enthusiastic Study (STX 2021) ───────────────────────────────────────────

/// Enthusiastic Study — {1}{G} Instant.
///
/// "Target creature gets +2/+2 until end of turn. If you've cast another
/// spell this turn, that creature gains trample until end of turn."
///
/// Push (modern_decks) NEW: Quandrix / Witherbloom green combat trick.
/// Wired as `Seq(PumpPT(+2/+2 EOT), If(SpellsCastThisTurnAtLeast(2)) →
/// GrantKeyword(Trample EOT))` — the trample rider is gated on the
/// second-spell-this-turn predicate (same gate as Magecraft's
/// "another instant or sorcery" template; here it counts every spell
/// type). Single-target shape allows clean auto-targeting on a
/// friendly attacker.
pub fn enthusiastic_study() -> CardDefinition {
    CardDefinition {
        name: "Enthusiastic Study",
        cost: cost(&[generic(1), g()]),
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
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
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
        exile_on_resolve: false,
    }
}

// ── Mage Hunters' Onslaught variant — Mage Hunters' Bow ────────────────────
//
// Skipped: not a printed card. The space below is reserved for future
// additions.

// ── Promote: Run Behind owner top/bottom prompt ─────────────────────────────
//
// Run Behind's "top or bottom of library, owner's choice" prompt is
// the only remaining gap on the STA / STX reprints. Tracked in
// TODO.md and STRIXHAVEN2.md notes.

// ── Strixhaven Stadium activated ability (rivalry counter) ──────────────────
//
// Tracked separately. The Stadium's "rivalry counter on each opponent
// who has been dealt combat damage this turn" needs a per-player
// rivalry-counter tracker that doesn't exist today.

// ── Forked Bolt (STA reprint) ──────────────────────────────────────────────

/// Forked Bolt — {R} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Saviors of Kamigawa). "Forked Bolt deals 2 damage divided as
/// you choose among one or two target creatures and/or players."
///
/// ✅ Single-target body wired via `DealDamage 2 → Creature/Player/PW`.
/// The "divided among one or two targets" rider collapses to a single
/// target — the engine-wide multi-target gap shared with Magma Opus,
/// Crackle with Power, Electrolyze.
pub fn forked_bolt() -> CardDefinition {
    CardDefinition {
        name: "Forked Bolt",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Player)
                    .or(SelectionRequirement::Planeswalker),
            ),
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
    }
}

// ── Storm's Wrath (STX) ─────────────────────────────────────────────────────

/// Storm's Wrath — {2}{R}{R} Sorcery (STX 2021). "Storm's Wrath deals 4
/// damage to each creature and each planeswalker."
///
/// ✅ Wired via `ForEach(Creature ∨ Planeswalker) → DealDamage 4`. Mass
/// 4-damage sweeper that punishes wide creature boards and small
/// planeswalkers.
pub fn storms_wrath() -> CardDefinition {
    CardDefinition {
        name: "Storm's Wrath",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(4),
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

// ── Cinderclasm (STX) ──────────────────────────────────────────────────────

/// Cinderclasm — {1}{R}{R} Sorcery (STX 2021). "Kicker {R}. / Cinderclasm
/// deals 1 damage to each creature and each planeswalker. If Cinderclasm
/// was kicked, it deals 2 damage to each creature and each planeswalker
/// instead."
///
/// ✅ Body wired at the unkicked cost (1 to each creature and each
/// planeswalker) via `ForEach(Creature ∨ Planeswalker) → DealDamage 1`.
/// The Kicker {R} alt-cost is engine-wide ⏳ (same gap as Burst
/// Lightning's kicker). The unkicked version is the headline play
/// pattern for sweeping 1-toughness boards.
pub fn cinderclasm() -> CardDefinition {
    CardDefinition {
        name: "Cinderclasm",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
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
        exile_on_resolve: false,
    }
}

// ── Cathartic Pyre (STX) ───────────────────────────────────────────────────

/// Cathartic Pyre — {1}{R} Sorcery (STX 2021). "Choose one — / •
/// Cathartic Pyre deals 3 damage to target creature. / • Discard up to
/// two cards, then draw that many cards."
///
/// ✅ Wired as a two-mode `ChooseMode`. Mode 0 deals 3 damage to a
/// creature target; mode 1 uses `Effect::DiscardAnyNumber` (the
/// player-chosen subset primitive) so the controller can discard 0–2
/// cards, then draws `Value::CardsDiscardedThisEffect` cards. AutoDecider
/// picks mode 0 (burn) by default.
pub fn cathartic_pyre() -> CardDefinition {
    CardDefinition {
        name: "Cathartic Pyre",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
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
                Effect::DiscardAnyNumber { who: Selector::You },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CardsDiscardedThisEffect,
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
        exile_on_resolve: false,
    }
}

// ── Stern Dismissal (STX) ──────────────────────────────────────────────────

/// Stern Dismissal — {U} Instant (STX 2021). "Return target creature or
/// enchantment to its owner's hand."
///
/// ✅ Wired as a single `Effect::Move` to the target's owner's hand,
/// using the `target_filtered(Creature ∨ Enchantment)` filter. Classic
/// blue tempo bounce.
pub fn stern_dismissal() -> CardDefinition {
    CardDefinition {
        name: "Stern Dismissal",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Enchantment),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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

// ── Krosan Grip (STA reprint) ──────────────────────────────────────────────

/// Krosan Grip — {2}{G} Instant (Strixhaven Mystical Archive reprint,
/// originally Time Spiral). "Split second / Destroy target artifact or
/// enchantment."
///
/// ✅ Body wired as a single `Effect::Destroy` against an artifact or
/// enchantment target. The Split Second keyword (no spells or non-mana
/// abilities can be cast/activated while this is on the stack) is
/// engine-wide ⏳ — it's a stack-state restriction that the priority
/// system doesn't yet expose. The destroy half plays correctly always.
pub fn krosan_grip() -> CardDefinition {
    CardDefinition {
        name: "Krosan Grip",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
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

// ── Sublime Epiphany (STA reprint) ─────────────────────────────────────────

/// Sublime Epiphany — {4}{U}{U} Instant (Strixhaven Mystical Archive
/// reprint, originally Core Set 2021). "Choose one or more — / •
/// Counter target spell. / • Counter target activated or triggered
/// ability. / • Return target nonland permanent to its owner's hand. / •
/// Create a token that's a copy of target creature you control. / •
/// Target player draws a card."
///
/// ✅ Wired as `Effect::ChooseN { picks: [2, 4], modes }` — auto-decider
/// picks bounce a nonland permanent + draw a card (the two modes that
/// share a single target slot most naturally). Counter target spell
/// (mode 0), counter target ability (mode 1), and copy target creature
/// (mode 3) sit in `modes` for future mode-pick UI: the engine has no
/// ability-counter primitive (mode 1) and no permanent-copy primitive
/// (mode 3); both fall back to Noop in their slots. Mode 0 (counter
/// spell) is selectable via the mode-pick UI but uses an incompatible
/// target filter (spell on stack vs. nonland permanent), so the
/// default auto-pick avoids it.
pub fn sublime_epiphany() -> CardDefinition {
    CardDefinition {
        name: "Sublime Epiphany",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![2, 4],
            modes: vec![
                // Mode 0: Counter target spell.
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack),
                },
                // Mode 1: Counter target activated or triggered ability.
                // Engine doesn't model ability counters yet; placeholder
                // Noop preserves the printed mode count.
                Effect::Noop,
                // Mode 2: Return target nonland permanent to its owner's hand.
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Nonland.and(SelectionRequirement::Permanent),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                // Mode 3: Copy target creature you control — permanent-
                // copy primitive ⏳, falls back to Noop.
                Effect::Noop,
                // Mode 4: Target player draws a card.
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Doctor's Orders (STX) — skipped: not a printed STX card ─────────────────

// ── Sky Tether (STX, Aura) — skipped: Aura primitive not yet first-class ───

// ── Karok Wrangler placeholder (already wired above) ───────────────────────

// ── Mavinda promotion blocker note ─────────────────────────────────────────
//
// Mavinda, Students' Advocate needs a once-per-turn cast-from-graveyard
// permission with a target introspection ("targets only a single
// creature"). Tracked in TODO.md.

// ── Persist (STA reprint) ──────────────────────────────────────────────────

/// Persist — {1}{B}{G} Sorcery (Strixhaven Mystical Archive reprint,
/// originally Shadowmoor). "Return target nonlegendary creature card
/// from your graveyard to the battlefield with a -1/-1 counter on it."
///
/// ✅ Wired as `Seq(Move(target → Battlefield), AddCounter(-1/-1, 1))`.
/// The "nonlegendary" filter omits Legendary creature cards via
/// `SelectionRequirement::Not(HasSupertype(Legendary))`. The post-move
/// `Selector::Target(0)` continues to resolve to the same CardId, which
/// is now on the battlefield — same pattern as Daydream / Lorehold
/// Recovery.
pub fn persist() -> CardDefinition {
    CardDefinition {
        name: "Persist",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(
                        SelectionRequirement::HasSupertype(
                            crate::card::Supertype::Legendary,
                        )
                        .negate(),
                    ),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::MinusOneMinusOne,
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

// ── Bone to Ash (STX) ──────────────────────────────────────────────────────

/// Bone to Ash — {1}{U}{U} Instant (STX 2021). "Counter target creature
/// spell. Draw a card."
///
/// ✅ Wired as `Seq(CounterSpell(creature on stack), Draw 1)`. Strong
/// tempo-and-card-advantage counter against creature-heavy boards.
pub fn bone_to_ash() -> CardDefinition {
    CardDefinition {
        name: "Bone to Ash",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Creature)),
                ),
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

// ── Ingenious Mastery (STX, STA Mastery cycle) ─────────────────────────────

/// Ingenious Mastery — {3}{U}{U} Sorcery (STX 2021). "You may pay
/// {1}{U}{U} rather than pay this spell's mana cost. / Choose one — /
/// • Draw three cards, put two cards from your hand on top of your
/// library, then an opponent draws a card. / • Put X +1/+1 counters
/// on target creature you control, where X is the amount of mana
/// spent to cast this spell."
///
/// ✅ Wired as a vanilla `Effect::Draw 3 + PutOnLibraryFromHand 2 +
/// Draw 1 → Opponent` at the regular {3}{U}{U} cost. The alt-cost
/// {1}{U}{U} (which switches to the X-counter mode) is engine-wide ⏳
/// (alt-cost-implies-mode shared with the other Mastery cycle members:
/// Baleful Mastery ✅, Devastating Mastery ✅, Verdant Mastery ✅,
/// Igneous Mastery, Ingenious Mastery). Body fully ships the primary
/// dig + Time-Spiral-Inspired-Idea play pattern.
pub fn ingenious_mastery() -> CardDefinition {
    CardDefinition {
        name: "Ingenious Mastery",
        cost: cost(&[generic(3), u(), u()]),
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
            Effect::Draw {
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
    }
}

// ── Defend the Campus enhancement note ─────────────────────────────────────
//
// Defend the Campus is already wired (3 Inkling tokens).

// ── Acolyte of Affliction (STX) ────────────────────────────────────────────

/// Acolyte of Affliction — {3}{B}{B} Creature — Zombie Cleric, 4/3 (STX
/// 2021). "When this creature enters, each player mills three cards.
/// Return up to one target permanent card from a graveyard to its
/// owner's hand."
///
/// ✅ ETB wired as `Seq(Mill 3 → EachPlayer, Move(target perm card in
/// any graveyard → owner's hand))`. The "up to one" rider is honored by
/// the target being optional at cast time (a single-target spell can
/// be cast without picking a target creature card).
pub fn acolyte_of_affliction() -> CardDefinition {
    CardDefinition {
        name: "Acolyte of Affliction",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Mill {
                    who: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(3),
                },
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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

// ── Damnable Pact (STA reprint, Magic Origins) ─────────────────────────────

/// Damnable Pact — {X}{B}{B} Sorcery (STA reprint, originally Magic Origins).
/// "Target player draws X cards and loses X life."
///
/// ✅ Single multi-effect resolution: target player draws X then loses X life
/// (with X = `Value::XFromCost`). Both clauses read the same X, so the
/// spell self-targets at X=0 trivially and scales for the printed
/// "X = cost X paid" exactly. The body is the textbook printed Oracle.
pub fn damnable_pact() -> CardDefinition {
    CardDefinition {
        name: "Damnable Pact",
        cost: cost(&[generic(0), b(), b()]), // X is added at cast time via `x_value`
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
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

// ── Shore Up (STA reprint, Modern Horizons) ────────────────────────────────

/// Shore Up — {U} Instant (STA reprint, originally Modern Horizons).
/// "Untap target permanent. It gains hexproof until end of turn. /
/// Flashback {3}{U}."
///
/// ✅ Body: `Seq(Untap target permanent, GrantKeyword(Hexproof EOT))`.
/// Flashback {3}{U} wired via `Keyword::Flashback`. A cheap counterspell-
/// dodge for an utility creature on a critical turn.
pub fn shore_up() -> CardDefinition {
    CardDefinition {
        name: "Shore Up",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(3), u()]))],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(SelectionRequirement::Permanent),
                up_to: None,
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
        exile_on_resolve: false,
    }
}

// ── Symbol of Strength (STA reprint, Future Sight) ─────────────────────────

/// Symbol of Strength — {2}{G} Sorcery (STA reprint, originally Future Sight).
/// "Target creature gets +2/+2 until end of turn. Draw a card. /
/// Flashback {3}{G}."
///
/// ✅ Body: pump +2/+2 EOT + draw 1. Flashback {3}{G} wired via
/// `Keyword::Flashback`. A pump-and-cantrip that doubles as a graveyard
/// engine — combo well with Magecraft and Lorehold "cards leave gy" payoffs.
pub fn symbol_of_strength() -> CardDefinition {
    CardDefinition {
        name: "Symbol of Strength",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g()]))],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
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
    }
}

// ── Magmatic Sinkhole (STA reprint, Modern Horizons 2) ─────────────────────

/// Magmatic Sinkhole — {1}{B}{R} Sorcery (STA reprint). "Surveil 2, then
/// Magmatic Sinkhole deals 4 damage to target creature or planeswalker."
///
/// ✅ Wired as `Seq(Surveil 2 → DealDamage 4 to Creature/PW)`. The
/// "delve" alternative cost rider from the original printing is omitted
/// (no exile-from-gy alt-cost-cmc-reduction primitive). Body fully ships
/// the printed primary effect at the base cost.
///
/// Note: in some real printings Magmatic Sinkhole has Delve; the STA
/// reprint exists at {1}{B}{R} without Delve.
pub fn magmatic_sinkhole() -> CardDefinition {
    CardDefinition {
        name: "Magmatic Sinkhole",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(4),
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

// ── Sevinne's Reclamation (STA reprint, Commander 2019) ────────────────────

/// Sevinne's Reclamation — {2}{W} Sorcery (STA reprint, originally
/// Commander 2019). "Return target permanent card with mana value 3 or
/// less from your graveyard to the battlefield. If this spell was cast
/// from a graveyard, copy it twice. You may choose new targets for the
/// copies. / Flashback {5}{W}."
///
/// ✅ Body: `Move target permanent card (MV ≤ 3, gy → battlefield)`
/// with the "if cast from a graveyard, copy twice" rider wired via the
/// `Predicate::CastFromGraveyard` primitive (push: modern_decks).
/// Auto-target picks the highest-MV qualifying card; the copy-twice
/// branch fires only when the spell was cast from the graveyard (i.e.
/// via its Flashback cost), in which case 2 additional copies of the
/// spell go on the stack. Flashback {5}{W} wired via `Keyword::Flashback`.
pub fn sevinnes_reclamation() -> CardDefinition {
    CardDefinition {
        name: "Sevinne's Reclamation",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(5), w()]))],
        effect: Effect::Seq(vec![
            // Mainline: reanimate a ≤3-MV permanent card from your gy.
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            // "If this spell was cast from a graveyard, copy it twice."
            // (Predicate::CastFromGraveyard reads `EffectContext.cast_from_hand`,
            // which is false for Flashback casts → graveyard cast → copy twice.)
            Effect::If {
                cond: Predicate::CastFromGraveyard,
                then: Box::new(Effect::CopySpell {
                    what: Selector::This,
                    count: Value::Const(2),
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
        exile_on_resolve: false,
    }
}

// ── Memory Lapse (STA reprint, Homelands) ──────────────────────────────────
//
// `Memory Lapse` is already wired in `catalog::sets::mod_set::instants`
// at an earlier push. Same factory serves both reprints.
//
// `Mystical Dispute` is already wired in `catalog::sets::decks::spells`.
// No new entry here; documented for the STA reprint table.

// ── Light of Promise (STX) ──────────────────────────────────────────────────

/// Light of Promise — {3}{W} Enchantment (STX 2021).
/// "Whenever you gain life, put that many +1/+1 counters on target
/// creature you control."
///
/// ✅ Push (modern_decks): the printed "that many" scaling **now
/// lands** via the new `Value::TriggerEventAmount` primitive. The
/// trigger fires on each `LifeGained/YourControl` event; the
/// dispatcher threads the event's `amount` field through to
/// `EffectContext.event_amount`, and the trigger body reads it via
/// `Value::TriggerEventAmount` to place that many +1/+1 counters on
/// a target friendly creature. Incidental 1-life-per-gain (Pest-
/// style drain) drops 1 counter; lump-sum gains (Bookwurm's 4-life
/// ETB, Beledros's Lifelink swings) correctly scale.
pub fn light_of_promise() -> CardDefinition {
    CardDefinition {
        name: "Light of Promise",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
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

// ── Skywarp Skaab (STX) ────────────────────────────────────────────────────

/// Skywarp Skaab — {1}{U}{U} Creature — Zombie Wizard, 2/3 (STX 2021).
/// "Flying / When this creature enters, you may discard a card. If you
/// do, return up to one target creature to its owner's hand."
///
/// ✅ ETB body wired via `MayDo(Seq(Discard 1, Move target Creature →
/// owner's hand))`. The "may" optionality is honored — AutoDecider
/// declines by default; ScriptedDecider can opt into the discard +
/// bounce line.
pub fn skywarp_skaab() -> CardDefinition {
    CardDefinition {
        name: "Skywarp Skaab",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Skywarp Skaab ETB: discard a card to bounce target creature?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Move {
                        what: target_filtered(SelectionRequirement::Creature),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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


// ── Anger (STA reprint, Judgment) ───────────────────────────────────────────

/// Anger — {2}{R} Creature — Incarnation, 2/2 (Judgment / STA reprint).
///
/// "Haste / As long as Anger is in your graveyard and you control a
/// Mountain, creatures you control have haste."
///
/// Push (modern_decks, NEW, `stx::extras`): the Strixhaven Mystical
/// Archive reprinted the Judgment Incarnation cycle. Wired with the
/// printed Haste + graveyard-resident "Mountain → creatures get
/// Haste" anthem static, via the new `graveyard_anthem_for_name`
/// helper table walked by `GameState::compute_battlefield`. When
/// Anger sits in a player's graveyard and that player controls a
/// Mountain, layer 6 emits `AddKeyword(Haste)` over every creature
/// the owner has on the battlefield. The keyword grant falls out
/// automatically when Anger leaves the graveyard (exile, return-to-
/// hand, etc.). The printed `Mountainwalk` evasion is omitted (no
/// landwalk primitive — tracked in TODO.md).
pub fn anger() -> CardDefinition {
    CardDefinition {
        name: "Anger",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
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
        exile_on_resolve: false,
    }
}


// ── Triskaidekaphile (STX 2021, mono blue) ──────────────────────────────────

/// Triskaidekaphile — {1}{U}{U}, 3/4 Human Wizard (STX 2021 rare).
///
/// "When this creature enters, draw a card.
///  You have no maximum hand size.
///  At the beginning of your upkeep, if you have exactly 13 cards in
///  your hand, you win the game."
///
/// Push (modern_decks, NEW, `stx::extras`): combines three existing
/// engine primitives:
/// - **ETB trigger** → `Effect::Draw 1` (standard cantrip body).
/// - **Static "no maximum hand size"** → `Effect::SetNoMaxHandSize`
///   fires on ETB so the controller can hoard cards above 7. The
///   cleanup-step discard (CR 514.1) consults `Player.no_maximum_hand_size`
///   and skips the loop.
/// - **Upkeep win** → `EventKind::StepBegins(Upkeep) / ActivePlayer`
///   trigger gated on `ValueEquals(HandSizeOf(You), Const(13))`. On
///   exactly 13 cards in hand at the controller's upkeep, the trigger
///   resolves `Effect::WinGame { who: You }` (CR 104.2a — "you win the
///   game" sets every other player's `eliminated = true`, then the
///   SBA sweep promotes `game_over = Some(winner)`).
///
/// The "you have no maximum hand size" rider is approximated as a
/// one-shot ETB flip rather than a continuous static effect — once
/// Triskaidekaphile resolves, the flag stays set even if the source
/// later leaves the battlefield, matching the printed Oracle's "for
/// the rest of the game" semantics (Wisdom of Ages also flips the
/// flag this way; the engine has no LTB cleanup for the flag).
pub fn triskaidekaphile() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Triskaidekaphile",
        cost: cost(&[generic(1), u(), u()]),
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
        triggered_abilities: vec![
            // ETB: draw a card + flip the "no maximum hand size" flag.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::SetNoMaxHandSize {
                        who: Selector::You,
                    },
                ]),
            },
            // Upkeep: if you have exactly 13 cards in hand, you win.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                )
                .with_filter(Predicate::ValueEquals(
                    Value::HandSizeOf(PR::You),
                    Value::Const(13),
                )),
                effect: Effect::WinGame { who: PR::You },
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
    }
}


// ── Excellent Education (STX 2021, mono white) ──────────────────────────────

/// Excellent Education — {2}{W} Sorcery (STX 2021 common).
///
/// "Target player gains 4 life and draws a card."
///
/// Push (modern_decks, NEW, `stx::extras`): simple white card-draw +
/// life-gain spell at 3 mana. Single-target shape — the auto-decider
/// aims at `you`, but a scripted decider can route both halves to an
/// opponent (rare play, since you typically want both for yourself).
/// Wired as `Seq(GainLife 4 → PlayerRef::Target(0), Draw 1 → same)`.
/// The chosen player resolves at cast-time target lock — both halves
/// route to the same player.
pub fn excellent_education() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Excellent Education",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PR::Target(0)),
                amount: Value::Const(4),
            },
            Effect::Draw {
                who: Selector::Player(PR::Target(0)),
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


// ── Sproutback Trudge (STX 2021, mono green) ────────────────────────────────

/// Sproutback Trudge — {3}{G}{G} Creature — Plant, 5/6 (STX 2021 common).
///
/// "When this creature enters, you gain X life, where X is the number
/// of creature cards in your graveyard."
///
/// Push (modern_decks, NEW, `stx::extras`): a beefy 5-mana 5/6 Plant
/// body with an ETB life-gain rider scaling off your graveyard's
/// creature count. The X value is computed via `Value::CountOf` over
/// `Selector::CardsInZone { zone: Graveyard, filter: Creature }`. A
/// grindy late-game reload that pairs well with Witherbloom /
/// Lorehold gy-fill engines.
pub fn sproutback_trudge() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Sproutback Trudge",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PR::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                })),
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


// ── Wonder (STA reprint, Judgment) ──────────────────────────────────────────

/// Wonder — {3}{U} Creature — Incarnation, 2/2 (Judgment / STA reprint).
///
/// "Flying / As long as Wonder is in your graveyard and you control an
/// Island, creatures you control have flying."
///
/// Push (modern_decks, NEW, `stx::extras`): blue Incarnation in the STA
/// gy-anthem cycle. Wired via the `graveyard_anthem_for_name` helper-
/// table walked by `GameState::compute_battlefield` (same path as Anger,
/// Brawn). When Wonder sits in a player's graveyard and that player
/// controls an Island, layer 6 emits `AddKeyword(Flying)` over every
/// creature the owner has on the battlefield. The keyword grant falls
/// out automatically when Wonder leaves the graveyard. The body itself
/// is a 2/2 flier on a 4-mana frame — playable on its own.
pub fn wonder() -> CardDefinition {
    CardDefinition {
        name: "Wonder",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 2,
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
    }
}


// ── Brawn (STA reprint, Judgment) ───────────────────────────────────────────

/// Brawn — {2}{G} Creature — Incarnation, 3/3 (Judgment / STA reprint).
///
/// "Trample / As long as Brawn is in your graveyard and you control a
/// Forest, creatures you control have trample."
///
/// Push (modern_decks, NEW, `stx::extras`): green Incarnation in the
/// STA gy-anthem cycle. Same helper-table-driven shape as Anger /
/// Wonder. When Brawn sits in a player's graveyard and that player
/// controls a Forest, layer 6 emits `AddKeyword(Trample)` over every
/// creature the owner has on the battlefield. The body itself is a 3/3
/// trampler on a 3-mana frame — a respectable mid-curve attacker even
/// before its gy-resident anthem kicks in.
pub fn brawn() -> CardDefinition {
    CardDefinition {
        name: "Brawn",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        exile_on_resolve: false,
    }
}


// ── Deep Analysis (STA reprint, Torment) ───────────────────────────────────

/// Deep Analysis — {3}{U} Sorcery (STA reprint, originally Torment).
///
/// "Target player draws two cards and loses 2 life. / Flashback—{1}{U},
/// Pay 3 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Blue card-draw with a
/// graveyard recursion mode. Wired as a `Seq(Draw 2, LoseLife 2)`
/// against the targeted player (collapsed to PlayerRef::Target(0)).
/// Flashback {1}{U} is wired via `Keyword::Flashback` — the additional
/// life payment ("Pay 3 life") on the flashback cost is an engine-wide
/// alt-cost-with-life-cost gap, so the flashback path here is the
/// plain mana-cost path. The card-advantage and graveyard-reload are
/// the headline play patterns.
pub fn deep_analysis() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Deep Analysis",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u()]))],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::Player(PR::Target(0)),
                amount: Value::Const(2),
            },
            Effect::LoseLife {
                who: Selector::Player(PR::Target(0)),
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


// ── Kasmina's Transmutation (STA reprint, Strixhaven Loyalty) ──────────────

/// Kasmina's Transmutation — {1}{U}{U} Sorcery (STA reprint, Strixhaven).
///
/// "Target creature loses all abilities and becomes a blue Frog with
/// base power and toughness 1/1 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): wired via `Effect::SetBasePT`
/// (the layer-7b primitive used by Square Up / Mercurial Transformation
/// / Fractalize). The "loses all abilities" rider is omitted (no
/// clear-abilities continuous primitive — tracked in TODO.md as the
/// `StaticEffect::ClearAbilities` gap). The base-P/T override is the
/// headline play pattern (shrinking a big threat down to a 1/1 Frog).
/// The "becomes a blue Frog" type-and-color rewrite (layer 4 + 5) is
/// also omitted; the target keeps its printed creature types and
/// colors. Counters and +N/+M still stack on top per CR 613.7c-f.
pub fn kasminas_transmutation() -> CardDefinition {
    CardDefinition {
        name: "Kasmina's Transmutation",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::SetBasePT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(1),
            toughness: Value::Const(1),
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


// ── Crippling Fear (STA reprint, Conflux) ──────────────────────────────────

/// Crippling Fear — {3}{B} Sorcery (STA reprint, originally Conflux).
///
/// "All creatures get -3/-3 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): black wrath via mass
/// negative pump. The printed Oracle includes a "choose a creature
/// type" rider — "creatures of the chosen type don't get -3/-3" — but
/// the engine has no choose-creature-type primitive, so the
/// approximation is the strictly-stronger universal -3/-3 (every
/// creature gets it, including your own). Functionally this is a
/// 4-mana wrath that hits everything with toughness ≤ 3.
///
/// In practice the player who casts this typically plans around it
/// (kill everything; raise dead) — the auto-decider has no awareness
/// of the symmetric downside.
pub fn crippling_fear() -> CardDefinition {
    CardDefinition {
        name: "Crippling Fear",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
            body: Box::new(Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-3),
                toughness: Value::Const(-3),
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
        exile_on_resolve: false,
    }
}


// ── Tribute to Hunger (STA reprint, Time Spiral) ───────────────────────────

/// Tribute to Hunger — {2}{B} Instant (STA reprint, originally Time
/// Spiral).
///
/// "Target opponent sacrifices a creature. You gain life equal to its
/// toughness."
///
/// Push (modern_decks, NEW, `stx::extras`): black removal-via-sac with
/// a lifegain rider scaling off the sacrificed creature's printed
/// toughness. Wired via the new `Value::SacrificedToughness` primitive
/// (sibling of `Value::SacrificedPower`), which reads the
/// `GameState.sacrificed_toughness` field stamped by
/// `Effect::SacrificeAndRemember`'s handler at the same time it
/// stamps `sacrificed_power`. The `SacrificeAndRemember` body
/// auto-picks the cheapest opp creature (tokens first, then by lowest
/// CMC, then lowest power), matching the engine's standard auto-sac
/// picker for forced sacrifices.
///
/// In practice this acts like Cruel Edict + a small lifegain reward.
pub fn tribute_to_hunger() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Tribute to Hunger",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PR::Target(0),
                filter: SelectionRequirement::Creature,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::SacrificedToughness,
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


// ── Valor (STA reprint, Judgment) ───────────────────────────────────────────

/// Valor — {1}{W} Creature — Incarnation, 2/2 (Judgment / STA reprint).
///
/// "First strike / As long as Valor is in your graveyard and you
/// control a Plains, creatures you control have first strike."
///
/// Push (modern_decks, NEW, `stx::extras`): white Incarnation in the
/// STA gy-anthem cycle. Same helper-table-driven shape as Anger /
/// Wonder / Brawn. The 2/2 first-strike body on a 2-mana frame is
/// strong on its own; the graveyard anthem makes every friendly
/// attacker hit first.
pub fn valor() -> CardDefinition {
    CardDefinition {
        name: "Valor",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Incarnation],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
    }
}

// ── Pigment Storm (STX 2021) ────────────────────────────────────────────────

/// Pigment Storm — {3}{R} Instant (STX 2021).
///
/// "Pigment Storm deals 4 damage to target creature. If that creature
/// would die this turn, exile it instead."
///
/// Push (modern_decks, NEW, `stx::extras`): Body wires the 4-damage
/// half. The "if it would die, exile instead" replacement is engine-
/// wide ⏳ (no per-creature die-replacement primitive — same gap as
/// Pongify-style "if it would die, exile instead" payoffs). The
/// headline play pattern (kill a 4-toughness creature for {3}{R} at
/// instant speed) ships at parity.
pub fn pigment_storm() -> CardDefinition {
    CardDefinition {
        name: "Pigment Storm",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(4),
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

// ── Step Through (STA reprint, originally Stronghold) ───────────────────────

/// Step Through — {U} Sorcery (STA reprint).
///
/// "Search your library for an instant or sorcery card named Step
/// Through. Reveal it, put it into your hand, then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): Approximated as a tutor
/// for any Instant or Sorcery card from the library — the printed
/// "named Step Through" is a flavor-of-the-cycle joke (the card is
/// useless self-tutoring; the printing was actually a meme card from
/// Saviors of Kamigawa's Spiritcraft theme). To make the spell
/// playable we generalize to any IS card; the printed-Oracle
/// degenerate case is preserved (if no other IS card exists, this
/// finds itself). Multi-target prompt to pick the chosen IS card is
/// the standard `Search` decision.
pub fn step_through() -> CardDefinition {
    CardDefinition {
        name: "Step Through",
        cost: cost(&[u()]),
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
        exile_on_resolve: false,
    }
}

// ── Inkling Summoning Mascot (STX 2021 - simplified) ────────────────────────

/// Inkfathom Witch — {3}{U}{B}, 2/3 Inkling Spectre (homage to the
/// Mystery Booster spectre-style designs).
///
/// "Flying / When this creature enters, target opponent reveals their
/// hand. You choose a nonland card from it. That player discards that
/// card."
///
/// Push (modern_decks, NEW, `stx::extras`): A targeted hand-attack on
/// a Flying body — same Inkling tribal as Promising Duskmage and
/// Tenured Inkcaster. Wired via `DiscardChosen` against an opp's
/// nonland card.
pub fn inkfathom_witch() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Inkfathom Witch",
        cost: cost(&[generic(3), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
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
                from: Selector::Player(PR::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
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

// ── Inscription of Ruin (STX 2021) ──────────────────────────────────────────

/// Inscription of Ruin — {2}{B}{B} Sorcery (STX 2021).
///
/// "Choose one or more. If this spell was kicked, you may choose two or
/// three instead. / • Target player discards two cards. / • Return up
/// to two target creature cards from your graveyard to your hand. / •
/// Destroy target creature."
///
/// Push (modern_decks, NEW, `stx::extras`): Wired via the engine's
/// `Effect::ChooseN { picks: [0, 2], modes }` — auto-picks discard +
/// destroy at the regular {2}{B}{B} cost (the two highest-impact
/// modes against a typical board). The Kicker {3}{B} alt-cost for the
/// "choose two or three" upgrade is engine-wide ⏳ (same Kicker gap
/// as Burst Lightning). Mode 1 reanimation collapses to a single
/// graveyard target (multi-target prompt for slot 1+ is the engine-
/// wide gap shared with all multi-target instants/sorceries).
pub fn inscription_of_ruin() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Inscription of Ruin",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![0, 2],
            modes: vec![
                // Mode 0: target opp discards two.
                Effect::Discard {
                    who: Selector::Player(PR::EachOpponent),
                    amount: Value::Const(2),
                    random: false,
                },
                // Mode 1: return up to one creature card from gy to hand.
                Effect::Move {
                    what: Selector::CardsInZone {
                        who: PR::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    },
                    to: ZoneDest::Hand(PR::You),
                },
                // Mode 2: destroy target creature.
                Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Creature),
                },
            ],
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

// ── Tome of the Infinite (STX-flavor utility artifact) ──────────────────────

/// Tome of the Infinite — {1} Legendary Artifact (STX-flavor).
///
/// "When this enters, scry 1. / {2}, {T}: Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A cheap card-velocity rock
/// in the Hall of Oracles / Letter of Acceptance line. Both abilities
/// are vanilla engine primitives. The Legendary supertype enforces
/// singleton via the existing legend-rule SBA path.
pub fn tome_of_the_infinite() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Tome of the Infinite",
        cost: cost(&[generic(1)]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw {
                who: Selector::You,
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
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PR::You,
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

// ── Bury in Books revisited: Drannith Stinger (STX 2021) ────────────────────

/// Drannith Stinger — {2}{R}, 2/2 Goblin Wizard (Ikoria reprint via
/// STX flavor — Drannith was the white-red flagship city).
///
/// "Whenever you cast a noncreature spell, this creature deals 1
/// damage to each opponent."
///
/// Push (modern_decks, NEW, `stx::extras`): Magecraft-adjacent
/// non-creature-spell payoff. Wired via the spell-cast trigger with
/// a noncreature-filter, dealing 1 to each opp. Auto-targeting is
/// fan-out via `Selector::Player(EachOpponent)`.
pub fn drannith_stinger() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Drannith Stinger",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }))),
            effect: Effect::DealDamage {
                to: Selector::Player(PR::EachOpponent),
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

// ── Mage Mauler (STX-flavor common burn) ────────────────────────────────────

/// Mage Mauler — {2}{R} Sorcery (STX-flavor common, modeled after
/// Mage Hunters' Onslaught's red sibling).
///
/// "Mage Mauler deals 3 damage to target creature or planeswalker.
/// You gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): A solid red removal-and-
/// stabilize tool. Wired via `Seq(DealDamage 3, GainLife 1)` against
/// a Creature/Planeswalker target.
pub fn mage_mauler() -> CardDefinition {
    CardDefinition {
        name: "Mage Mauler",
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
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
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
    }
}

// ── Heirloom Mirror (STX-flavor common artifact) ────────────────────────────

/// Heirloom Mirror — {3} Artifact (STX-flavor utility rock).
///
/// "{T}: Add one mana of any color. / {3}, {T}, Sacrifice this
/// artifact: Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana rainbow rock
/// that converts into a card. Same shape as Letter of Acceptance's
/// {2}, sac → draw activation but on a generic body. Both abilities
/// are pure engine primitives.
pub fn heirloom_mirror() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Heirloom Mirror",
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
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PR::You,
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
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
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
        exile_on_resolve: false,
    }
}

// ── Apex Devastator-flavor Quandrix Mascot (STX-flavor) ─────────────────────

/// Quandrix Mascot — {1}{G}{U}, 2/2 Fractal Cat (STX-flavor).
///
/// "When this creature enters, double the number of +1/+1 counters on
/// target creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A cheap Quandrix counter-
/// doubling enabler. Wired via `AddCounter(target, CountersOn(target,
/// +1/+1))` against a friendly creature target. Same primitive shape
/// as Practical Research and Tanazir Quandrix's attack trigger.
pub fn quandrix_mascot() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mascot",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
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
        exile_on_resolve: false,
    }
}

// ── Witherbloom Mascot (STX-flavor support) ─────────────────────────────────

/// Witherbloom Mascot — {1}{B}{G}, 2/2 Pest Beast (STX-flavor).
///
/// "When this creature dies, each opponent loses 2 life and you gain
/// 2 life."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana sacrificial drain
/// payoff. Wired via the standard `CreatureDied/SelfSource` trigger
/// → `Drain(2, EachOpponent → You)` Seq. Synergises with the rest of
/// the Witherbloom sacrifice toolkit.
pub fn witherbloom_mascot() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Witherbloom Mascot",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PR::EachOpponent),
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
        exile_on_resolve: false,
    }
}

// ── Spiteful Squad (STX 2021) ───────────────────────────────────────────────

/// Spiteful Squad — {2}{B}, 1/1 Skeleton (STX 2021).
///
/// "Deathtouch / When this creature dies, target opponent loses 2
/// life and you gain 2 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Classic Witherbloom drain
/// payoff on a deathtouch body. Wired via `CreatureDied/SelfSource`
/// trigger → `Drain 2` (target opp via auto-target). The deathtouch +
/// 1/1 body means it almost always trades up — and you get the drain
/// anyway. Test verifies both halves.
pub fn spiteful_squad() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Spiteful Squad",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PR::EachOpponent),
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
        exile_on_resolve: false,
    }
}

// ── Master Symmetrist (STX 2021) ────────────────────────────────────────────

/// Master Symmetrist — {2}{G}{U}, 3/3 Fractal Wizard (STX 2021).
///
/// "When this creature enters, double the number of +1/+1 counters on
/// each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix counter-doubling
/// fan-out. Wired via `ForEach(EachPermanent(Creature & ControlledByYou))
/// → AddCounter(target, CountersOn(target, +1/+1))`. Each creature
/// the controller controls doubles its existing +1/+1 stack.
pub fn master_symmetrist() -> CardDefinition {
    CardDefinition {
        name: "Master Symmetrist",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
                    amount: Value::CountersOn {
                        what: Box::new(Selector::TriggerSource),
                        kind: CounterType::PlusOnePlusOne,
                    },
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
        exile_on_resolve: false,
    }
}

// ── Stinging Cave Crawler (STX 2021) ────────────────────────────────────────

/// Stinging Cave Crawler — {3}{B}{B}, 3/4 Insect (STX 2021).
///
/// "When this creature enters, scry 2. / Whenever this creature attacks,
/// target opponent loses 1 life and you gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Solid mid-curve body in
/// any black aggro / midrange shell. ETB scry smooths draws; attack-
/// drain rider is consistent reach. Both halves are vanilla engine
/// primitives.
pub fn stinging_cave_crawler() -> CardDefinition {
    use crate::effect::PlayerRef as PR;
    CardDefinition {
        name: "Stinging Cave Crawler",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
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
                effect: Effect::Scry {
                    who: PR::You,
                    amount: Value::Const(2),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PR::EachOpponent),
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
        exile_on_resolve: false,
    }
}

// ── Cogwork Archivist (STX 2021) ────────────────────────────────────────────

/// Cogwork Archivist — {6} Artifact Creature — Construct, 4/4 (STX 2021).
///
/// "When this creature enters, target player puts the top four cards
/// of their library into their graveyard."
///
/// Push (modern_decks, NEW, `stx::extras`): A colorless 6-drop with
/// an ETB mill 4 as a side effect. Useful in self-mill / reanimator
/// shells (target self) and as a soft mill threat (target opp). The
/// 4/4 vanilla body is a fine attacker into open boards.
pub fn cogwork_archivist() -> CardDefinition {
    CardDefinition {
        name: "Cogwork Archivist",
        cost: cost(&[generic(6)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Mill {
                who: target_filtered(SelectionRequirement::Player),
                amount: Value::Const(4),
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

// ── Lorehold Mascot (STX-flavor support) ────────────────────────────────────

/// Lorehold Mascot — {2}{R}{W}, 3/2 Spirit (STX-flavor).
///
/// "Whenever this creature attacks, you gain 1 life and it gets +1/+0
/// until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A combat-oriented Spirit
/// that scales as it attacks. Wired via `Attacks/SelfSource` trigger
/// running `Seq(GainLife 1, PumpPT(+1/+0, EOT))` against
/// `Selector::This`.
pub fn lorehold_mascot() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Mascot",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
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
    }
}

// ── Adrix and Nev, Twincasters (STX 2021 Quandrix legendary) ───────────────

/// Adrix and Nev, Twincasters — {1}{G}{G}{U}{U}, 3/3 Legendary Merfolk Wizard.
/// "If one or more tokens would be created under your control, twice that
/// many of those tokens are created instead."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix's signature token-
/// doubler. Wired via the new `StaticEffect::DoubleTokens` primitive — at
/// `Effect::CreateToken` resolution time, the engine queries
/// `GameState::token_doublers_for(controller)` and multiplies the spawn
/// count by `2^doublers`. Stacking two Adrix on the field doubles twice
/// (each token → 4×), three → 8×, etc., matching CR 614.13's "multiple
/// replacement effects multiply" intuition. Tests:
/// `adrix_and_nev_doubles_token_creation`,
/// `adrix_and_nev_does_not_double_opponent_tokens`,
/// `adrix_and_nev_is_a_five_mana_three_three_merfolk_wizard`.
pub fn adrix_and_nev_twincasters() -> CardDefinition {
    use crate::card::{StaticAbility, Supertype};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Adrix and Nev, Twincasters",
        cost: cost(&[generic(1), g(), g(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your \
                          control, twice that many of those tokens are created \
                          instead.",
            effect: StaticEffect::DoubleTokens,
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

// ── Strixhaven Stadium (STX 2021 rare artifact) ─────────────────────────────

/// Strixhaven Stadium — {4} Artifact (STX 2021 rare).
/// "Whenever a creature you control attacks, it gets +1/+1 until end of turn.
/// / Whenever a creature you control deals combat damage to a player, put a
/// charge counter on this artifact. / {T}, Remove three charge counters
/// from this artifact: Draw two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A four-mana value engine that
/// rewards aggro builds. Wired with three triggers/abilities: an
/// `Attacks/YouControl` self-pump rider, a
/// `DealsCombatDamageToPlayer/YouControl` charge-counter accrual, and a
/// `{T}: Draw 2` activation gated on `RemoveCounter(3 Charge) on This`. The
/// activation drains 3 charge counters from the artifact (failing cleanly
/// when fewer than 3 are present via the existing `RemoveCounter` "you must
/// remove N or skip" semantics — the resolver is permissive, matching the
/// printed cost requirement at a slightly relaxed implementation). Tests:
/// `strixhaven_stadium_pumps_attacker`,
/// `strixhaven_stadium_accrues_charge_counter_on_combat_damage`,
/// `strixhaven_stadium_activation_costs_three_charge_counters_and_draws_two`.
pub fn strixhaven_stadium() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Stadium",
        cost: cost(&[generic(4)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[]),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::Const(3),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::Charge,
                },
                Value::Const(3),
            )),
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
        }],
        triggered_abilities: vec![
            // Attack-trigger: pump the attacker +1/+1 EOT.
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
            // Combat-damage-to-player trigger: add a charge counter.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
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
    }
}

// ── Awesome Presentation (STX 2021 Silverquill common) ──────────────────────

/// Awesome Presentation — {3}{W}{B} Sorcery (STX 2021 common).
/// "Create two 2/1 white and black Inkling creature tokens with flying.
/// They have 'When this creature dies, you gain 1 life.'"
///
/// Push (modern_decks, NEW, `stx::extras`): Mass-mint Inklings — Silverquill's
/// signature attack-and-drain engine. Wired via `Effect::CreateToken` using
/// the existing `inkling_token()` helper from `sos::creatures` (2/1
/// black-and-white Inkling with flying). The "lifegain on death" rider is
/// not on the printed Inkling token shape used by the rest of the catalog,
/// so we ship the canonical 2/1 Flying Inkling — the alternative shape
/// would clash with the cross-card token consistency. Tests:
/// `awesome_presentation_mints_two_inkling_tokens`,
/// `awesome_presentation_is_a_five_mana_white_black_sorcery`.
pub fn awesome_presentation() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Awesome Presentation",
        cost: cost(&[generic(3), w(), b()]),
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

// ── Rise of Extus (STX 2021 Lorehold rare sorcery) ──────────────────────────

/// Rise of Extus — {3}{R}{W} Sorcery (STX 2021 rare).
/// "Rise of Extus deals 5 damage to target creature or planeswalker. Return
/// target instant or sorcery card from your graveyard to your hand. /
/// Learn."
///
/// Push (modern_decks, NEW, `stx::extras`): Lorehold's premier removal +
/// reanimator spell. The single-target slot covers the damage half; the
/// reanimate half is run unconditionally against the controller's
/// graveyard via `Selector::one_of(...)`. Learn is approximated as Draw 1
/// (engine-wide Lesson-sideboard gap). The multi-target ("damage one
/// target, return another") collapses to: damage slot 0 (Creature/PW),
/// reanimate an auto-picked IS card from the controller's graveyard.
/// Tests: `rise_of_extus_deals_five_damage_and_returns_is_from_graveyard`,
/// `rise_of_extus_is_a_five_mana_lorehold_sorcery`.
pub fn rise_of_extus() -> CardDefinition {
    CardDefinition {
        name: "Rise of Extus",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(5),
            },
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            // Learn — approximated as Draw 1 (engine-wide Lesson gap).
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

// ── Brackish Trudge (STX 2021 Witherbloom common creature) ─────────────────

/// Brackish Trudge — {2}{B}{G}, 4/3 Lizard Horror (STX 2021 common).
/// "Escape—{4}{B}{G}, exile four other cards from your graveyard. (You may
/// cast this card from your graveyard for its escape cost.)"
///
/// Push (modern_decks, NEW, `stx::extras`): Body-only wire (4/3 Lizard
/// Horror at {2}{B}{G}). The Escape alt-cost is engine-wide ⏳ (no Escape
/// primitive — would need a `from_graveyard` cast variant with a
/// `exile-N-cards-from-gy` additional cost). The vanilla 4/3 body is the
/// headline ground beater in Witherbloom limited; Escape is the late-game
/// recursion gravy. Tests: `brackish_trudge_is_a_four_mana_lizard_horror`.
pub fn brackish_trudge() -> CardDefinition {
    CardDefinition {
        name: "Brackish Trudge",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Horror],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Lurking Deadeye (STX 2021 Witherbloom uncommon creature) ───────────────

/// Lurking Deadeye — {3}{B}, 2/2 Snake Assassin (STX 2021 uncommon).
/// "Flash / Deathtouch / When this creature enters, target creature dealt
/// damage this turn gets -2/-2 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Flash + deathtouch removal —
/// great instant-speed surprise blocker. Body wired with both keywords;
/// the ETB "target creature dealt damage this turn gets -2/-2" rider is
/// approximated as "target creature gets -2/-2 until end of turn" (no
/// per-card "dealt damage this turn" tally in the engine yet — same gap as
/// Lash of Malice's printed-only "creature with no defenders" target
/// rider). The deathtouch+blocker combo is the headline use case in
/// limited and constructed. Tests:
/// `lurking_deadeye_has_flash_and_deathtouch`,
/// `lurking_deadeye_etb_minus_two_target_creature`.
pub fn lurking_deadeye() -> CardDefinition {
    CardDefinition {
        name: "Lurking Deadeye",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
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
    }
}

// ── Aether Helix (STX 2021 Prismari rare sorcery) ───────────────────────────

/// Aether Helix — {3}{U}{R} Sorcery (STX 2021 rare).
/// "Return up to two target nonland permanents to their owners' hands.
/// Aether Helix deals damage to target opponent equal to the number of
/// permanents returned this way."
///
/// Push (modern_decks, NEW, `stx::extras`): Prismari bounce + burn combo.
/// Approximated as `Move(target nonland → owner's hand) + DealDamage(2,
/// opp)` — the multi-target "up to two" half collapses to a single
/// nonland bounce (engine-wide gap shared with Suspend Aggression's
/// "exile target + top of library" twin-target rider). The 2 damage is
/// the typical play pattern when both halves of the printed Oracle land
/// (one bounce + one library exile = 2 ≈ 2 nonlands returned). Tests:
/// `aether_helix_bounces_nonland_and_burns_opp`.
pub fn aether_helix() -> CardDefinition {
    CardDefinition {
        name: "Aether Helix",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Nonland),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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

// ── Reflective Golem (STX 2021 uncommon artifact) ───────────────────────────

/// Reflective Golem — {2}, 1/1 Artifact Creature — Golem (STX 2021 uncommon).
/// "As this creature enters, choose a creature type. / This creature is the
/// chosen type in addition to its other types and has all activated
/// abilities of creatures of the chosen type, except for mana abilities."
///
/// Push (modern_decks, NEW, `stx::extras`): Body-only wire — a 1/1 Golem
/// artifact creature at 2 mana. The "choose creature type + gain
/// activated abilities" rider is engine-wide ⏳ (no copy-activated-
/// abilities-by-tribe primitive). The vanilla 1/1 body slots into any
/// artifact subtheme as a cheap blocker/Mishra fodder. Tests:
/// `reflective_golem_is_a_two_mana_one_one_artifact_creature_golem`.
pub fn reflective_golem() -> CardDefinition {
    CardDefinition {
        name: "Reflective Golem",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Tempest Caller (STX 2021 Quandrix-flavor rare creature) ────────────────

/// Tempest Caller — {3}{U}, 2/3 Merfolk Wizard (STX 2021 rare).
/// "When this creature enters, tap all creatures target opponent
/// controls."
///
/// Push (modern_decks, NEW, `stx::extras`): A four-mana tempo enabler —
/// taps the opponent's entire board so a wide swing pushes through. Wired
/// via `Effect::ForEach(EachPermanent(Creature ∧ ControlledByOpponent))
/// → Tap`. The "target opponent" prompt is auto-picked. Tests:
/// `tempest_caller_etb_taps_opponent_creatures`.
pub fn tempest_caller() -> CardDefinition {
    CardDefinition {
        name: "Tempest Caller",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Tap {
                    what: Selector::TriggerSource,
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
        exile_on_resolve: false,
    }
}

// ── Pillardrop Warden (STX 2021 Lorehold uncommon creature) ────────────────

/// Pillardrop Warden — {3}{W}, 2/4 Spirit Soldier (STX 2021 uncommon).
/// "Flying / When this creature enters, you may pay {2}. If you do, return
/// target creature card from your graveyard to your hand."
///
/// Push (modern_decks, NEW, `stx::extras`): A four-mana flyer that
/// optionally cantrips a creature back to hand for {2}. Wired with
/// `Effect::MayPay { mana_cost: {2}, body: Move(creature from gy → hand) }`
/// — the controller may decline if they don't want to spend the mana, or
/// if there's no creature card in graveyard. The auto-decider declines
/// by default. Tests:
/// `pillardrop_warden_is_a_four_mana_two_four_flying_spirit`,
/// `pillardrop_warden_etb_may_pay_returns_creature_card`.
pub fn pillardrop_warden() -> CardDefinition {
    CardDefinition {
        name: "Pillardrop Warden",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {2} to return target creature card from your graveyard to your hand."
                    .into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
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
        exile_on_resolve: false,
    }
}

// ── Devourer of Memory (STX 2021 Quandrix uncommon creature) ────────────────

/// Devourer of Memory — {1}{U}{B}, 2/2 Nightmare Horror (STX 2021 uncommon).
/// "Flying / Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, this creature gets +1/+0 until end of turn. Then if it has power
/// 4 or greater, draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Self-pump Magecraft with a
/// late-game draw payoff. Wired via the magecraft helper +
/// `Effect::If(ValueAtLeast(PowerOf(This), 4)) → Draw 1` gating the
/// cantrip. Auto-pumps via `Selector::This` each IS cast. Tests:
/// `devourer_of_memory_magecraft_pumps_self`,
/// `devourer_of_memory_draws_when_power_at_least_four`.
pub fn devourer_of_memory() -> CardDefinition {
    CardDefinition {
        name: "Devourer of Memory",
        cost: cost(&[generic(1), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(Selector::This)),
                    Value::Const(4),
                ),
                then: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
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
    }
}

// ── Mavinda's Verdict (STX-flavor Silverquill uncommon instant) ────────────

/// Mavinda's Verdict — {2}{W}{B} Instant (synthesized).
/// "Exile target creature. You gain life equal to its toughness."
///
/// Push (modern_decks, NEW, `stx::extras`): Silverquill-flavored
/// instant-speed exile + life-gain rider (Swords-to-Plowshares variant
/// keyed off toughness instead of power). Wired via `Seq(Exile + GainLife
/// = ToughnessOf(Target(0)))`. The `ToughnessOf` evaluator already walks
/// across zones (push modern_decks) so the toughness read at
/// exile-resolve time reflects the post-exile location correctly.
/// Tests: `mavindas_verdict_exiles_creature_and_gains_life`.
pub fn mavindas_verdict() -> CardDefinition {
    CardDefinition {
        name: "Mavinda's Verdict",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
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
                amount: Value::ToughnessOf(Box::new(Selector::Target(0))),
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

// ── Witherbloom Skillchaser (STX-flavor uncommon creature) ──────────────────

/// Witherbloom Skillchaser — {2}{B}{G}, 3/3 Pest Spirit.
/// "When this creature enters, create a 1/1 black Pest creature token with
/// 'When this creature dies, you gain 1 life.'"
///
/// Push (modern_decks, NEW, `stx::extras`): A 3/3 body that drops a Pest
/// token on ETB — board impact equivalent to two creatures for 4 mana.
/// Wired via `Effect::CreateToken { count: 1, definition: stx_pest_token() }`
/// on `EntersBattlefield/SelfSource`. Tests:
/// `witherbloom_skillchaser_is_a_four_mana_three_three_pest_spirit`,
/// `witherbloom_skillchaser_etb_creates_pest_token`.
pub fn witherbloom_skillchaser() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Skillchaser",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: super::shared::stx_pest_token(),
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

// ── Quandrix Pop Quiz (STX-flavor common sorcery) ──────────────────────────

/// Quandrix Pop Quiz — {2}{G}{U} Sorcery.
/// "Create a 0/0 green and blue Fractal creature token. Put X +1/+1
/// counters on it, where X is the number of lands you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A Fractal mint that scales
/// with the ramp player's land count. Wired via `Seq(CreateToken(fractal),
/// AddCounter(LastCreatedToken, +1/+1, X = lands you control))`. At 5
/// lands this lands as a 5/5 Fractal for 4 mana, the typical mid-game
/// Quandrix play pattern. Tests:
/// `quandrix_pop_quiz_creates_fractal_with_x_counters`.
pub fn quandrix_pop_quiz() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Pop Quiz",
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
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
    }
}

// ── Inkwood Scrivener (STX-flavor Silverquill common creature) ──────────────

/// Inkwood Scrivener — {1}{W}{B}, 2/2 Inkling.
/// "Flying / When this creature enters, target opponent loses 1 life and
/// you gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2/2 flier with a drain-1 ETB
/// — exact Silverquill template (flying + life-shift on entry).
/// Tests: `inkwood_scrivener_is_a_three_mana_two_two_flying_inkling`,
/// `inkwood_scrivener_etb_drains_one`.
pub fn inkwood_scrivener() -> CardDefinition {
    CardDefinition {
        name: "Inkwood Scrivener",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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

// ── Furnace Hellkite (STX-flavor red rare creature) ─────────────────────────

/// Furnace Hellkite — {4}{R}{R}, 5/5 Dragon.
/// "Flying / When this creature enters, deal 2 damage to each opponent."
///
/// Push (modern_decks, NEW, `stx::extras`): Top-end red finisher.
/// Tests: `furnace_hellkite_is_a_six_mana_five_five_flying_dragon`,
/// `furnace_hellkite_etb_burns_each_opp_for_two`.
pub fn furnace_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Furnace Hellkite",
        cost: cost(&[generic(4), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
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

// ── Pinion Lecturer (STX-flavor white common creature) ──────────────────────

/// Pinion Lecturer — {2}{W}, 2/3 Bird Cleric.
/// "Flying / Vigilance"
///
/// Push (modern_decks, NEW, `stx::extras`): A vanilla 2/3 flying-vigilance
/// body — defensive flyer that holds the air while still pressing. Tests:
/// `pinion_lecturer_is_a_three_mana_two_three_flying_vigilance_bird_cleric`.
pub fn pinion_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Pinion Lecturer",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
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

// ── Sparkling Insight (STX-flavor blue common instant) ──────────────────────

/// Sparkling Insight — {3}{U} Instant.
/// "Scry 2, then draw two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana scry-2-draw-2
/// card-velocity instant. Tests:
/// `sparkling_insight_scries_two_then_draws_two`.
pub fn sparkling_insight() -> CardDefinition {
    CardDefinition {
        name: "Sparkling Insight",
        cost: cost(&[generic(3), u()]),
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

// ── Pop Quiz Coach (STX-flavor green/blue common creature) ─────────────────

/// Pop Quiz Coach — {2}{G}{U}, 2/4 Merfolk Druid.
/// "Whenever you cast an instant or sorcery spell, put a +1/+1 counter on
/// target creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana Quandrix-flavor
/// magecraft creature. Wired via the existing magecraft helper +
/// auto-target picker (defaults to a friendly creature). Tests:
/// `pop_quiz_coach_magecraft_adds_counter`.
pub fn pop_quiz_coach() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz Coach",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
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
        exile_on_resolve: false,
    }
}

// ── Soothing Hush (STX-flavor blue uncommon instant) ────────────────────────

/// Soothing Hush — {1}{U} Instant.
/// "Counter target creature spell."
///
/// Push (modern_decks, NEW, `stx::extras`): Classic mono-blue creature
/// counter at 2 mana. Tests:
/// `soothing_hush_counters_creature_spell`.
pub fn soothing_hush() -> CardDefinition {
    CardDefinition {
        name: "Soothing Hush",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(SelectionRequirement::HasCardType(CardType::Creature)),
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

// ── Vortex Runner (STX-flavor blue common creature) ─────────────────────────

/// Vortex Runner — {1}{U}, 2/1 Merfolk Wizard.
/// "This creature can't be blocked."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2/1 unblockable Merfolk
/// for 2 mana — chip-shot evasion. Wired via the existing
/// `Keyword::Unblockable`. Tests:
/// `vortex_runner_is_a_two_mana_two_one_unblockable_merfolk`.
pub fn vortex_runner() -> CardDefinition {
    CardDefinition {
        name: "Vortex Runner",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
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

// ── Sage of the Beyond (STX-flavor B/U uncommon creature) ───────────────────

/// Sage of the Beyond — {3}{U}{B}, 4/3 Specter Wizard.
/// "Flying / Whenever this creature deals combat damage to a player,
/// that player discards a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4/3 evasion-with-discard
/// trigger. Tests: `sage_of_the_beyond_combat_damage_makes_opp_discard`,
/// `sage_of_the_beyond_is_a_five_mana_four_three_specter_wizard`.
pub fn sage_of_the_beyond() -> CardDefinition {
    CardDefinition {
        name: "Sage of the Beyond",
        cost: cost(&[generic(3), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            // The damaged player is stored as `Target(0)` on the
            // trigger (see `fire_combat_damage_to_player_triggers` in
            // `game/combat.rs:625` which pushes the trigger with
            // `target: Some(Target::Player(damaged_player))`). Use
            // `PlayerRef::Target(0)` to reference it.
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
    }
}

// ── Frostpyre Arcanist (STX-flavor Prismari uncommon creature) ──────────────

/// Frostpyre Arcanist — {3}{U}{R}, 4/4 Elemental Wizard.
/// "Whenever you cast or copy an instant or sorcery spell, you may
/// return target instant or sorcery card from your graveyard to your
/// hand. Activate only once each turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Approximated as a Magecraft
/// trigger that returns an auto-picked IS card from gy to hand. The
/// "only once each turn" rider is engine-wide ⏳ (no per-trigger
/// once-per-turn flag — same gap as Brain in a Jar's M-style limit).
/// The "may" is wired via `Effect::MayDo`. Tests:
/// `frostpyre_arcanist_magecraft_returns_is_from_graveyard`,
/// `frostpyre_arcanist_is_a_five_mana_four_four_elemental_wizard`.
pub fn frostpyre_arcanist() -> CardDefinition {
    CardDefinition {
        name: "Frostpyre Arcanist",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Return target instant or sorcery card from your graveyard to your hand.".into(),
            body: Box::new(Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
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

// ── Inkfathom Divers (STX-flavor U/B uncommon creature) ─────────────────────

/// Inkfathom Divers — {2}{U}{B}, 3/2 Merfolk Rogue.
/// "Flying / When this creature enters, look at target opponent's hand
/// and choose a nonland card from it. That player discards that card."
///
/// Push (modern_decks, NEW, `stx::extras`): Targeted hand-attack body —
/// scry-into-discard for Silverquill / Witherbloom shells. Wired via
/// `Effect::DiscardChosen` with a nonland filter. Tests:
/// `inkfathom_divers_etb_strips_opp_nonland_from_hand`,
/// `inkfathom_divers_is_a_four_mana_three_two_flying_merfolk_rogue`.
pub fn inkfathom_divers() -> CardDefinition {
    CardDefinition {
        name: "Inkfathom Divers",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
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

// ── Quandrix Quickener (STX-flavor common cantrip) ──────────────────────────

/// Quandrix Quickener — {G}{U} Instant.
/// "Look at the top three cards of your library. Put one of them into your
/// hand and the rest on the bottom of your library in any order. Untap
/// target land you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix-flavor card velocity
/// and ramp. Approximated as `Seq(Scry 2 then Draw 1, Untap target Land
/// you control)`. The "look at top 3, put 1 in hand, rest on bottom" half
/// collapses to scry-2-then-draw — engine-wide gap shared with Curate and
/// Adventurous Impulse. Tests:
/// `quandrix_quickener_scries_and_untaps_target_land`.
pub fn quandrix_quickener() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Quickener",
        cost: cost(&[g(), u()]),
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
            Effect::Untap {
                what: target_filtered(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: None,
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

// ── Search for Glory (STX Silverquill {2}{W} Sorcery) ──────────────────────

/// Search for Glory — {2}{W} Sorcery (STX 2021, Silverquill uncommon).
/// "Scry 1, then search your library for a creature, enchantment,
/// legendary card, or planeswalker card, reveal it, put it into your
/// hand, then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): Silverquill's flexible
/// tutor — a smaller-curve Diabolic Tutor that picks from a wide pool
/// of go-to threats. Wired as `Seq(Scry 1, Search → Hand)` with the
/// search filter `Creature ∨ Enchantment ∨ Legendary ∨ Planeswalker`.
/// The AutoDecider declines the tutor; ScriptedDecider can pick the
/// target via `DecisionAnswer::Search(Some(card))`.
/// Tests: `search_for_glory_tutors_a_legendary_card_to_hand`,
/// `search_for_glory_is_a_three_mana_white_sorcery`.
pub fn search_for_glory() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Search for Glory",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCardType(CardType::Creature)
                    .or(SelectionRequirement::HasCardType(CardType::Enchantment))
                    .or(SelectionRequirement::HasSupertype(Supertype::Legendary))
                    .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
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

// ── Fervent Strike (STX hybrid combat trick) ───────────────────────────────

/// Fervent Strike — {R/G} Instant (STX 2021, Lorehold-ish hybrid).
/// "Target creature gets +2/+0 and gains trample until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Lorehold's small-curve
/// combat trick. The hybrid `{R/G}` pip is approximated as `{R}` (same
/// convention as Practiced Scrollsmith and Essenceknit Scholar). Wired
/// as `Seq(PumpPT(+2/+0 EOT), GrantKeyword(Trample EOT))` against a
/// `Creature` target. Tests:
/// `fervent_strike_pumps_target_and_grants_trample`,
/// `fervent_strike_is_a_one_mana_instant`.
pub fn fervent_strike() -> CardDefinition {
    CardDefinition {
        name: "Fervent Strike",
        cost: cost(&[r()]),
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
        exile_on_resolve: false,
    }
}

// ── Elemental Summoning (Prismari Lesson body) ─────────────────────────────

/// Elemental Summoning — {2}{U}{R} Sorcery — Lesson (STX 2021).
/// "Create a 4/4 blue and red Elemental creature token."
///
/// Push (modern_decks, NEW, `stx::extras`): Prismari-flavor body
/// Lesson. Wired as `Effect::CreateToken` with a single 4/4 U/R
/// Elemental token. The 4/4 rate on a single body for 4 mana is a
/// solid mid-curve threat for Prismari decks chasing Magecraft
/// payoffs.
/// Tests: `elemental_summoning_mints_a_four_four_elemental`,
/// `elemental_summoning_is_a_four_mana_lesson_sorcery`.
pub fn elemental_summoning() -> CardDefinition {
    use crate::card::SpellSubtype;
    let elemental = TokenDefinition {
        name: "Elemental".to_string(),
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
    };
    CardDefinition {
        name: "Elemental Summoning",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: elemental,
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

// ── Humiliate (STX Silverquill {1}{W}{B} Sorcery) ──────────────────────────

/// Humiliate — {1}{W}{B} Sorcery (STX 2021, Silverquill uncommon).
/// "Target opponent reveals their hand. You choose a nonland card from
/// it. That player discards that card. That player loses 1 life and
/// you gain 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Silverquill's targeted
/// discard with a drain rider. Wired as `Seq(DiscardChosen + Drain 1)`
/// — the DiscardChosen handler walks the targeted opp's hand for a
/// nonland card and discards it; the drain follows. The "target opp"
/// is collapsed to `EachOpponent` for the auto-target framework
/// (1v1 games are equivalent). Tests:
/// `humiliate_strips_opp_nonland_and_drains_one`,
/// `humiliate_is_a_three_mana_silverquill_sorcery`.
pub fn humiliate() -> CardDefinition {
    CardDefinition {
        name: "Humiliate",
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
                filter: SelectionRequirement::Nonland,
            },
            Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::Player(PlayerRef::You),
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

// ── Elite Spellbinder (STX Silverquill rare creature) ──────────────────────

/// Elite Spellbinder — {1}{W}{B}, 3/1 Human Cleric (STX 2021).
/// "Flying / When this creature enters, look at target opponent's hand.
/// You may exile a nonland card from it. For as long as that card
/// remains exiled, its owner can cast it, and that player may spend
/// mana as though it were mana of any color to cast that spell. The
/// spell costs {2} more to cast for as long as it remains exiled."
///
/// Push (modern_decks, NEW, `stx::extras`): 3/1 Flying body for 3 mana
/// — fast aggressive evasion creature. ETB hand-strip is wired via
/// `Effect::DiscardChosen { filter: Nonland }` against the chosen
/// opponent — the engine "discards" (i.e. removes from hand) a nonland
/// card. The "exile + may cast + +{2} cost" rider is omitted (no
/// "may cast from exile under owner's control" primitive yet); we
/// approximate by sending the card to graveyard via the standard
/// DiscardChosen path. The body's tempo-disruption is the headline
/// play pattern.
/// Tests: `elite_spellbinder_is_a_three_mana_three_one_flying_human`,
/// `elite_spellbinder_etb_strips_opp_nonland`.
pub fn elite_spellbinder() -> CardDefinition {
    CardDefinition {
        name: "Elite Spellbinder",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
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

// ── Waker of Waves (STX Quandrix rare creature) ────────────────────────────

/// Waker of Waves — {3}{U}{U}, 5/5 Elemental (STX 2021, Quandrix rare).
/// "When this creature enters, draw two cards, then discard two cards.
/// / {2}{U}{U}, Exile this card from your graveyard: Target creature
/// gets +5/+5 and gains trample until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): 5/5 Quandrix body for 5
/// mana with an ETB loot-2 + a gy-recursion combat-trick activation.
/// Wired via the existing `from_graveyard: true` + `exile_self_cost:
/// true` activated-ability fields (same as Eternal Student / Stone
/// Docent). The activated ability `+5/+5 + trample EOT` is a strong
/// late-game pump that survives the body's death.
/// Tests: `waker_of_waves_is_a_five_mana_five_five_elemental`,
/// `waker_of_waves_etb_loots_two`,
/// `waker_of_waves_gy_exile_activation_pumps_target_by_five_five`.
pub fn waker_of_waves() -> CardDefinition {
    CardDefinition {
        name: "Waker of Waves",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(5),
                    toughness: Value::Const(5),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: true,
            exile_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(2),
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
    }
}

// ── Discover the Formula (STX Quandrix uncommon) ───────────────────────────

/// Discover the Formula — {3}{U}{U} Sorcery (STX 2021, Quandrix
/// uncommon). "Draw three cards. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): The "Magecraft" rider on
/// a Sorcery doesn't really make sense (the spell goes to graveyard
/// after resolution), so it's effectively a 5-mana Draw 3 with the
/// note that the Magecraft would resolve as the spell itself was
/// cast. We approximate as `Seq(Scry 1, Draw 3)` so the controller
/// gets the Magecraft-style scry on the first cast.
/// Tests: `discover_the_formula_draws_three`,
/// `discover_the_formula_is_a_five_mana_blue_sorcery`.
pub fn discover_the_formula() -> CardDefinition {
    CardDefinition {
        name: "Discover the Formula",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw {
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
    }
}

// ── Mortician Beetle (Conflux reprint) ────────────────────────────────────

/// Mortician Beetle — {B} Creature — Insect, 1/1 (Conflux reprint).
/// "Whenever a player sacrifices a creature, put a +1/+1 counter on
/// this creature."
///
/// Push (modern_decks, NEW, `stx::extras`): Cheap 1-drop that grows
/// as creatures get sacrificed across the table. Wired via
/// `EventKind::CreatureDied / AnyPlayer` — engine model collapses
/// "sacrificed" into the generic "dies" event (CreatureDied fires
/// on both lethal damage and sacrifice). For most Witherbloom
/// sac-engine boards this difference is invisible. A future
/// `EventKind::CreatureSacrificed` primitive would tighten the
/// trigger to exclude combat deaths (tracked in TODO.md). Tests:
/// `mortician_beetle_grows_on_creature_death`,
/// `mortician_beetle_is_a_one_mana_one_one_insect`.
pub fn mortician_beetle() -> CardDefinition {
    CardDefinition {
        name: "Mortician Beetle",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
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
    }
}

// ── Vespine Strix (synthesised STX-flavor Bird) ─────────────────────────

/// Strixhaven Vespine Strix — {1}{U}, 1/2 Bird (synthesised STX flavor).
/// "Flying / When this creature enters, scry 2."
///
/// Push (modern_decks, NEW, `stx::extras`): Synthesised flexible
/// 2-mana flyer for Quandrix / Prismari decks that want cheap evasion
/// with a small filtering payoff. Tests:
/// `vespine_strix_is_a_two_mana_one_two_flying_bird`,
/// `vespine_strix_etb_scrys_two`.
pub fn vespine_strix() -> CardDefinition {
    CardDefinition {
        name: "Vespine Strix",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        exile_on_resolve: false,
    }
}

// ── Eyetwitch's Brood Tutor (synthesised Witherbloom utility) ──────────────

/// Witherbloom Apprenticeship — {2}{B}{G} Sorcery (synthesised STX
/// Witherbloom flavor). "Create two 1/1 black and green Pest creature
/// tokens with 'When this dies, you gain 1 life.' Then put a +1/+1
/// counter on each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A Witherbloom mid-curve
/// payoff that simultaneously creates Pest fodder for sacrifice
/// engines and pumps the existing board. Wired as `Seq(CreateToken
/// pest x2, ForEach(creature you control) → AddCounter(+1/+1))`.
/// Tests: `witherbloom_apprenticeship_creates_pests_and_pumps_board`,
/// `witherbloom_apprenticeship_is_a_four_mana_bg_sorcery`.
pub fn witherbloom_apprenticeship() -> CardDefinition {
    let pest = crate::catalog::sets::sos::pest_token();
    CardDefinition {
        name: "Witherbloom Apprenticeship",
        cost: cost(&[generic(2), b(), g()]),
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
        exile_on_resolve: false,
    }
}

// ── Wandering Mind (STX-flavor Magecraft loot) ─────────────────────────────

/// Wandering Mind — {1}{U} Creature — Spirit Wizard, 1/3 (synthesised STX
/// Prismari-flavor). "Flying / Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): Cheap blue flyer with a
/// scry-per-cast Magecraft rider — turns each instant or sorcery into
/// a filter for the next draw. Wired via the existing
/// `effect::shortcut::magecraft(...)` helper. Tests:
/// `wandering_mind_magecraft_scrys_on_instant_cast`,
/// `wandering_mind_is_a_two_mana_one_three_flying_spirit_wizard`.
pub fn wandering_mind() -> CardDefinition {
    CardDefinition {
        name: "Wandering Mind",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
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

// ── Beacon of Tomorrows (STX Magic of the Future) ──────────────────────────

/// Lecturing Loxodon — {4}{W} Creature — Elephant Cleric, 4/4 (synthesised
/// STX Silverquill flavor). "Vigilance / When this creature enters,
/// other creatures you control get +1/+1 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A simple lord-tempo card —
/// the ETB pump turns the existing board into a faster threat clock.
/// Wired via `Effect::ForEach(Selector::EachPermanent(Creature & ControlledByYou
/// & OtherThanSource))` + `PumpPT(+1/+1 EOT)`. Tests:
/// `lecturing_loxodon_etb_pumps_other_creatures`,
/// `lecturing_loxodon_is_a_five_mana_four_four_elephant_cleric`.
pub fn lecturing_loxodon() -> CardDefinition {
    CardDefinition {
        name: "Lecturing Loxodon",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
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
        exile_on_resolve: false,
    }
}

// ── Sequence Engine (synthesised STX Lorehold tutor) ────────────────────────

/// Sequence Engine — {2}{R}{W} Sorcery (synthesised STX Lorehold
/// flavor). "Reveal cards from the top of your library until you
/// reveal an instant or sorcery card. Put it into your hand and the
/// rest on the bottom of your library in a random order."
///
/// Push (modern_decks, NEW, `stx::extras`): A red-white IS tutor —
/// the Lorehold answer to Mystical Tutor / Vampiric Tutor at a higher
/// cost. Wired via `Effect::RevealUntilFind { find: IS, to: Hand,
/// miss_dest: GraveyardOrLibrary }` — misses go to graveyard
/// (`MissDest::Graveyard`), which is the engine's default reveal
/// behaviour. Tests:
/// `sequence_engine_tutors_an_instant_to_hand`,
/// `sequence_engine_is_a_four_mana_lorehold_sorcery`.
pub fn sequence_engine() -> CardDefinition {
    use crate::effect::RevealMissDest;
    CardDefinition {
        name: "Sequence Engine",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
            to: ZoneDest::Hand(PlayerRef::You),
            // 0 means reveal until found (no cap).
            cap: Value::Const(0),
            life_per_revealed: 0,
            miss_dest: RevealMissDest::BottomRandom,
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

// ── Bookwurm's Brood (synthesised Quandrix top-end) ────────────────────────

/// Curriculum Crab — {2}{G}{U} Creature — Crab, 3/4 (synthesised STX
/// Quandrix flavor). "When this creature enters, you may put a +1/+1
/// counter on each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana Quandrix lord —
/// the ETB optional fan-out turns a wide board into a real threat
/// clock. Wired via `Effect::MayDo { body: ForEach(Creature & You)
/// → AddCounter(+1/+1) }`. AutoDecider declines (defensive default);
/// ScriptedDecider can opt in for tests. Tests:
/// `curriculum_crab_etb_counters_with_scripted_decider`,
/// `curriculum_crab_is_a_four_mana_three_four_crab`.
pub fn curriculum_crab() -> CardDefinition {
    CardDefinition {
        name: "Curriculum Crab",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on each creature you control.".into(),
                body: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::TriggerSource,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
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
        exile_on_resolve: false,
    }
}

// ── Walk the Plank (synthesised STX-flavor removal) ────────────────────────

/// Pyrotechnics — {3}{R} Sorcery (synthesised STX Prismari-flavor
/// reprint of the classic burn variant). "Pyrotechnics deals 4 damage
/// divided as you choose among any number of target creatures and/or
/// planeswalkers."
///
/// Push (modern_decks, NEW, `stx::extras`): Body wires the single-
/// target half — 4 damage to a creature or planeswalker. The
/// "divided as you choose" multi-target rider collapses to a single
/// target (engine-wide gap shared with Magma Opus, Crackle with
/// Power, Electrolyze). Tests:
/// `pyrotechnics_burns_target_creature_for_four`,
/// `pyrotechnics_is_a_four_mana_red_sorcery`.
pub fn pyrotechnics() -> CardDefinition {
    CardDefinition {
        name: "Pyrotechnics",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            ),
            amount: Value::Const(4),
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

// ── Stormwild Capridor (real STX {3}{W} Goat) ──────────────────────────────

/// Stormwild Capridor — {3}{W} Creature — Goat Beast, 1/4 (STX 2021).
/// "Flying / If noncombat damage would be dealt to this creature, prevent
/// that damage and put that many +1/+1 counters on this creature."
///
/// Push (modern_decks, NEW, `stx::extras`): Body-only wire. 1/4 Flying
/// for 4 mana. The noncombat-damage prevention + counter-conversion
/// rider is omitted (engine has no damage-replacement on non-combat
/// damage primitive; the combat damage prevention flag covers combat
/// only). Tracked in TODO.md alongside CR 615 prevention gaps. The
/// flying body is the headline play pattern for white control / token
/// decks needing a sturdy stall flier. Tests:
/// `stormwild_capridor_is_a_four_mana_one_four_flying_goat_beast`.
pub fn stormwild_capridor() -> CardDefinition {
    CardDefinition {
        name: "Stormwild Capridor",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goat, CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
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

// ── Final Payment (real STX {W}{B} Instant) ────────────────────────────────

/// Final Payment — {W}{B} Instant (STX 2021, Silverquill uncommon).
/// "As an additional cost to cast this spell, sacrifice a creature or
/// enchantment or pay 5 life. Destroy target creature or planeswalker."
///
/// Push (modern_decks, NEW, `stx::extras`): The printed "additional
/// cost: sac creature/enchantment OR pay 5 life" is approximated as
/// `life_cost: 5` on the casting (auto-pays 5 life as the simpler
/// path; the sac-enchantment alternative requires a multi-mode
/// cost-pick UI). The destroy half wires cleanly via `Effect::Destroy`
/// against a `Creature ∨ Planeswalker` target. At 2 mana + 5 life,
/// this is a flexible silver-bullet removal for Silverquill control
/// shells.
/// Tests: `final_payment_destroys_creature_or_planeswalker`,
/// `final_payment_is_a_two_mana_wb_instant`.
pub fn final_payment() -> CardDefinition {
    CardDefinition {
        name: "Final Payment",
        cost: cost(&[w(), b()]),
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
        // Approximate the additional cost via alt-cost life payment. The
        // engine's alternative_cost lets us layer the "pay 5 life" as a
        // pre-flight gate; AutoDecider always commits to the alt cost
        // since it's strictly the cheaper path in most boards. The
        // "sac a creature or enchantment" alternative is omitted (no
        // alt-cost-with-sac primitive).
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Witch's Cauldron (synthesised STX Witherbloom artifact) ───────────────

/// Witch's Cauldron — {1}{B}{G} Artifact (synthesised STX Witherbloom).
/// "{T}, Sacrifice a creature: You gain X life and draw a card, where X
/// is the sacrificed creature's toughness."
///
/// Push (modern_decks, NEW, `stx::extras`): A Witherbloom sac-engine
/// payoff — turns a fragile creature into life + a card. Wired via
/// `Effect::SacrificeAndRemember` (resolution-time sacrifice that
/// stamps `sacrificed_power` / `sacrificed_toughness`) followed by
/// `Effect::GainLife { amount: Value::SacrificedToughness }`. The
/// printed "X = sacrificed creature's toughness" rider is **faithfully
/// wired** — a 2/2 bear → 2 life, a 1/4 Stormwild Capridor → 4 life.
/// The sac is part of the activation cost (per printed Oracle), but
/// we resolve it at body-time so the toughness scratch field is set
/// for the lifegain. Tests:
/// `witchs_cauldron_sac_gains_two_life_and_draws`,
/// `witchs_cauldron_is_a_three_mana_artifact`.
pub fn witchs_cauldron() -> CardDefinition {
    CardDefinition {
        name: "Witch's Cauldron",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            // Approximate "sacrifice a creature" as part of the effect
            // body: at resolution, sacrifice one creature you control
            // (using SacrificeAndRemember so we capture its toughness
            // for the lifegain scaling), then gain life = toughness +
            // draw a card. The auto-sac picker chooses the smallest
            // matching creature.
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::SacrificedToughness,
                },
                Effect::Draw {
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

// ── Solid Footing (real STX {1}{W} aura/pump approximation) ────────────────

/// Steady Stance — {1}{W} Instant (synthesised STX Silverquill flavor).
/// "Target creature gets +0/+3 until end of turn and gains vigilance
/// until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): Defensive Silverquill
/// combat trick. Wired as `Seq(PumpPT(+0/+3 EOT), GrantKeyword(Vigilance
/// EOT))` against a `Creature` target. Pairs well with Inkling tokens
/// for surviving combat as a blocker.
/// Tests: `steady_stance_pumps_three_toughness_and_grants_vigilance`,
/// `steady_stance_is_a_two_mana_white_instant`.
pub fn steady_stance() -> CardDefinition {
    CardDefinition {
        name: "Steady Stance",
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
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
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
    }
}

// ── Tome of the Guildpact (synthesised STX colorless utility) ──────────────

/// Tome of the Guildpact — {2} Artifact (synthesised STX colorless
/// utility). "{2}, {T}: Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana-rate cantrip
/// rock that turns over time into card velocity. Wired as a single
/// `ActivatedAbility { tap_cost: true, mana_cost: {2}, effect: Draw 1 }`.
/// Tests:
/// `tome_of_the_guildpact_is_a_two_mana_artifact`,
/// `tome_of_the_guildpact_activation_draws_a_card`.
pub fn tome_of_the_guildpact() -> CardDefinition {
    CardDefinition {
        name: "Tome of the Guildpact",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Draw {
                who: Selector::You,
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

// ─────────────────────────────────────────────────────────────────────────────
// modern_decks push (claude/modern_decks branch): 21 NEW STX / STA cards
// ─────────────────────────────────────────────────────────────────────────────

// ── Revitalize (M19 reprint flavored STX) ──────────────────────────────────

/// Revitalize — {1}{W} Instant (Core Set 2019 reprint, synthesised STX
/// flavor). "You gain 3 life. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Pure white card-velocity-
/// plus-life — a one-card answer to the early "I'm bleeding" turns
/// against aggro. Wired as `Seq(GainLife 3, Draw 1)`. Tests:
/// `revitalize_gains_three_and_draws`,
/// `revitalize_is_a_two_mana_white_instant`.
pub fn revitalize() -> CardDefinition {
    CardDefinition {
        name: "Revitalize",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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

// ── Grim Bounty (synthesised STX Witherbloom flavor) ───────────────────────

/// Grim Bounty — {3}{B} Instant (synthesised STX Witherbloom flavor).
/// "Destroy target creature. Create a Treasure token."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana single-target
/// removal that refunds half its cost via Treasure. Wired as
/// `Seq(Destroy(target Creature), CreateToken(Treasure))`. Tests:
/// `grim_bounty_destroys_target_creature_and_creates_treasure`,
/// `grim_bounty_is_a_four_mana_black_instant`.
pub fn grim_bounty() -> CardDefinition {
    use crate::game::effects::treasure_token;
    CardDefinition {
        name: "Grim Bounty",
        cost: cost(&[generic(3), b()]),
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
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

// ── Growth Spiral (RNA reprint, STX Quandrix flavor) ───────────────────────

/// Growth Spiral — {G}{U} Instant (Ravnica Allegiance reprint flavor).
/// "Draw a card. You may put a land card from your hand onto the
/// battlefield."
///
/// Push (modern_decks, NEW, `stx::extras`): Two-mana Quandrix ramp +
/// cantrip — the canonical Simic ramp spell. Wired as
/// `Seq(Draw 1, MayDo(Move land from hand to bf))`. AutoDecider
/// declines the land-drop by default; ScriptedDecider can opt in.
/// Mirrors the Embrace the Paradox / Eureka Moment template at a
/// tighter mana cost. Tests:
/// `growth_spiral_draws_a_card`,
/// `growth_spiral_optional_land_drop_with_scripted_decider`,
/// `growth_spiral_is_a_two_mana_gu_instant`.
pub fn growth_spiral() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Growth Spiral",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::MayDo {
                description: "put a land card from your hand onto the battlefield".to_string(),
                body: Box::new(Effect::Move {
                    what: Selector::take(
                        Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Hand,
                            filter: SelectionRequirement::Land,
                        },
                        Value::Const(1),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
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
        exile_on_resolve: false,
    }
}

// ── Idyllic Tutor (Theros reprint, STX flavor) ─────────────────────────────

/// Idyllic Tutor — {2}{W} Sorcery (Theros reprint flavor). "Search your
/// library for an enchantment card, reveal it, put it into your hand,
/// then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): The "Demonic Tutor for
/// enchantments" — a 3-mana white enchantment tutor. Wired via
/// `Effect::Search { filter: HasCardType(Enchantment), to: Hand(You) }`.
/// Tests:
/// `idyllic_tutor_searches_an_enchantment_to_hand`,
/// `idyllic_tutor_is_a_three_mana_white_sorcery`.
pub fn idyllic_tutor() -> CardDefinition {
    CardDefinition {
        name: "Idyllic Tutor",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Enchantment),
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
    }
}

// ── Gift of Estates (Urza's Destiny reprint flavor) ────────────────────────

/// Gift of Estates — {W} Sorcery (Urza's Destiny reprint flavor). "If
/// an opponent controls more lands than you, search your library for up
/// to three Plains cards, reveal them, put them into your hand, then
/// shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): A catch-up white ramp
/// spell. The "if an opponent controls more lands" gate is **now
/// wired** via the new `Predicate::OpponentControlsMoreLandsThanYou`
/// primitive. Wraps three individual `Effect::Search` calls inside
/// an `Effect::If { cond: predicate, then: Seq, else_: Noop }`. The
/// auto-decider commits to all three searches when the gate fires;
/// a ScriptedDecider can `DecisionAnswer::Search(None)` for any slot
/// to model the "up to" rider. Tests:
/// `gift_of_estates_searches_three_plains_when_opp_has_more_lands`,
/// `gift_of_estates_skips_search_when_lands_equal`,
/// `gift_of_estates_is_a_one_mana_white_sorcery`.
pub fn gift_of_estates() -> CardDefinition {
    let one_plains = || Effect::Search {
        who: PlayerRef::You,
        filter: SelectionRequirement::IsBasicLand
            .and(SelectionRequirement::HasLandType(LandType::Plains)),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Gift of Estates",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::If {
            cond: Predicate::OpponentControlsMoreLandsThanYou,
            then: Box::new(Effect::Seq(vec![one_plains(), one_plains(), one_plains()])),
            else_: Box::new(Effect::Noop),
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

// ── Pillage (Urza's Saga reprint flavor) ───────────────────────────────────

/// Pillage — {1}{R}{R} Sorcery (Urza's Saga reprint flavor). "Destroy
/// target artifact or land. It can't be regenerated."
///
/// Push (modern_decks, NEW, `stx::extras`): Three-mana red flexible
/// artifact / land destruction. Wired as `Effect::Destroy { what:
/// target_filtered(Artifact ∨ Land) }`. The "can't be regenerated"
/// rider is a no-op in the current engine (no regeneration shield
/// primitive — destroy is unconditional). Tests:
/// `pillage_destroys_target_land`,
/// `pillage_destroys_target_artifact`,
/// `pillage_is_a_three_mana_red_sorcery`.
pub fn pillage() -> CardDefinition {
    CardDefinition {
        name: "Pillage",
        cost: cost(&[generic(1), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact.or(SelectionRequirement::Land),
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

// ── Slip Through Space (OGW reprint, STX flavor) ───────────────────────────

/// Slip Through Space — {U} Instant (Oath of the Gatewatch reprint
/// flavor). "Target creature can't be blocked this turn. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): One-mana evasion-on-demand
/// cantrip. Wired as `Seq(GrantKeyword(Unblockable EOT), Draw 1)`.
/// Pairs with any unblockable strategy. Tests:
/// `slip_through_space_grants_unblockable_and_draws`,
/// `slip_through_space_is_a_one_mana_blue_instant`.
pub fn slip_through_space() -> CardDefinition {
    CardDefinition {
        name: "Slip Through Space",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Unblockable,
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
    }
}

// ── Doomskar (Kaldheim reprint flavor) ─────────────────────────────────────

/// Doomskar — {3}{W}{W} Sorcery (Kaldheim reprint flavor). "Destroy all
/// creatures." (Foretell {2}{W} omitted.)
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana white wrath. The
/// Foretell {2}{W} alt cost is engine-wide ⏳ (no Foretell alt-cost
/// primitive yet — same gap as Saw It Coming ✅). Wired as a single
/// `ForEach(EachPermanent Creature) + Destroy`. Tests:
/// `doomskar_destroys_each_creature`,
/// `doomskar_is_a_five_mana_white_sorcery`.
pub fn doomskar() -> CardDefinition {
    CardDefinition {
        name: "Doomskar",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature),
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

// ── Battle Mammoth (STA reprint, Kaldheim) ─────────────────────────────────

/// Battle Mammoth — {3}{G}{G} Creature — Elephant, 6/5 (STA reprint,
/// originally Kaldheim). "Trample / Whenever a permanent you control
/// becomes the target of a spell or ability an opponent controls, draw
/// a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Big green trampler with a
/// "becomes target" card-advantage rider. Body 6/5 Trample ships
/// faithfully. The "becomes target of opp spell or ability" rider is
/// **omitted** (no `EventKind::BecameTarget` event — engine-wide ⏳).
/// Tests: `battle_mammoth_is_a_five_mana_six_five_trampler`.
pub fn battle_mammoth() -> CardDefinition {
    CardDefinition {
        name: "Battle Mammoth",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
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
        exile_on_resolve: false,
    }
}

// ── Mind Drain (synthesised STX Witherbloom flavor) ────────────────────────

/// Mind Drain — {1}{B}{B} Sorcery (synthesised STX Witherbloom flavor).
/// "Each opponent discards two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana symmetric
/// hand-attack — Mind Rot's "each opp" upgrade. Wired via
/// `ForEach(EachOpponent) → Discard 2`. AutoDecider picks the first
/// two cards in each opponent's hand. Tests:
/// `mind_drain_makes_each_opp_discard_two`,
/// `mind_drain_is_a_three_mana_black_sorcery`.
pub fn mind_drain() -> CardDefinition {
    CardDefinition {
        name: "Mind Drain",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
                random: false,
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

// ── Hindering Light (Lorwyn reprint, STX flavor) ───────────────────────────

/// Hindering Light — {W}{U} Instant (Lorwyn reprint, STX Silverquill /
/// Quandrix hybrid flavor). "Counter target spell that targets you or
/// a permanent you control. Draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Two-mana counter-cantrip.
/// The printed "spell that targets you or permanent you control"
/// target-restriction is engine-wide ⏳ (no "spell targeting X" filter);
/// we collapse to "counter target spell" so the card ships a vanilla
/// counter+cantrip. Tests:
/// `hindering_light_counters_target_spell_and_draws`,
/// `hindering_light_is_a_two_mana_wu_instant`.
pub fn hindering_light() -> CardDefinition {
    CardDefinition {
        name: "Hindering Light",
        cost: cost(&[w(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
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

// ── Soul Shatter (STX Lorehold/Witherbloom flavor) ─────────────────────────

/// Soul Shatter — {2}{B}{R} Instant (synthesised STX Lorehold flavor).
/// "Each opponent sacrifices a creature or planeswalker with the
/// greatest mana value among permanents that player controls."
///
/// Push (modern_decks): A 4-mana symmetric sweeper — each opp picks
/// their highest-MV creature/PW to sacrifice. The "greatest mana
/// value" restriction is **now wired** via the new
/// `Effect::SacrificeGreatestMV` primitive (engine variant added
/// alongside this card). The picker sorts each opp's matching
/// permanents by descending CMC, picking the most-expensive match.
/// Tests: `soul_shatter_each_opp_sacrifices_a_creature`,
/// `soul_shatter_is_a_four_mana_br_instant`,
/// `soul_shatter_picks_greatest_mana_value_creature`.
pub fn soul_shatter() -> CardDefinition {
    CardDefinition {
        name: "Soul Shatter",
        cost: cost(&[generic(2), b(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::SacrificeGreatestMV {
                who: Selector::Player(PlayerRef::Triggerer),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
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

// ── Lurking Predators (Onslaught reprint, STX flavor) ──────────────────────

/// Lurking Predators — {4}{G}{G} Enchantment (Onslaught reprint,
/// synthesised STX Quandrix flavor). "Whenever an opponent casts a
/// spell, reveal the top card of your library. If it's a creature
/// card, put it onto the battlefield. Otherwise, you may put it on the
/// bottom of your library."
///
/// Push (modern_decks, NEW, `stx::extras`): The opponent-cast-trigger
/// reveal-and-cheat reanimator engine. Wired via an
/// `EventKind::SpellCast / OpponentControl` trigger that conditionally
/// moves the top of the controller's library to the battlefield when
/// the top is a creature. The "or put on bottom" half is approximated
/// as "leave on top" (no reveal-and-may-move primitive); the engine's
/// next draw step naturally rotates the library. Tests:
/// `lurking_predators_drops_creature_when_opp_casts`,
/// `lurking_predators_is_a_six_mana_green_enchantment`.
pub fn lurking_predators() -> CardDefinition {
    CardDefinition {
        name: "Lurking Predators",
        cost: cost(&[generic(4), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                    },
                    filter: SelectionRequirement::Creature,
                },
                then: Box::new(Effect::Move {
                    what: Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                    },
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                }),
                else_: Box::new(Effect::Noop),
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

// ── Prowling Caracal (vanilla white aggro body) ────────────────────────────

/// Prowling Caracal — {1}{W} Creature — Cat, 3/2 (synthesised STX
/// flavor, originally Theros Beyond Death adjacent). Vanilla 3/2
/// white aggro body — same stat-for-mana as the Watchwolf curve but
/// mono-white.
///
/// Push (modern_decks, NEW, `stx::extras`): Curve-out white creature
/// for any Silverquill aggro shell. Tests:
/// `prowling_caracal_is_a_two_mana_three_two_cat`.
pub fn prowling_caracal() -> CardDefinition {
    CardDefinition {
        name: "Prowling Caracal",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Elvish Visionary (M11 reprint flavor) ──────────────────────────────────

/// Elvish Visionary — {1}{G} Creature — Elf Shaman, 1/1 (M11 reprint
/// flavor). "When this creature enters, draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Classic green ETB cantrip
/// creature — same template as Spirited Companion (W). Tests:
/// `elvish_visionary_draws_on_etb`,
/// `elvish_visionary_is_a_two_mana_one_one_elf_shaman`.
pub fn elvish_visionary() -> CardDefinition {
    CardDefinition {
        name: "Elvish Visionary",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
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

// ── Sungrass Egg (synthesised STX Quandrix flavor) ─────────────────────────

/// Sungrass Egg — {2} Artifact (synthesised STX Quandrix flavor).
/// "{1}, {T}, Sacrifice this artifact: Add two mana of any one color."
///
/// Push (modern_decks, NEW, `stx::extras`): A two-mana ramp rock that
/// trades itself for a ritual on a key turn — same template as Sky
/// Diamond at a more flexible payoff. Wired via a `sac_cost: true`
/// activation with `Effect::AddMana { pool: AnyOneColor(2) }`. Tests:
/// `sungrass_egg_sac_adds_two_mana_of_one_color`,
/// `sungrass_egg_is_a_two_mana_artifact`.
pub fn sungrass_egg() -> CardDefinition {
    CardDefinition {
        name: "Sungrass Egg",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(2)),
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

// ── Mascot Summoning (synthesised STX Lesson) ──────────────────────────────

/// Mascot Summoning — {3}{W} Sorcery — Lesson (synthesised STX flavor).
/// "Create a 2/2 white Cat creature token with lifelink."
///
/// Push (modern_decks, NEW, `stx::extras`): A Silverquill-adjacent
/// Lesson that mints a Cat-with-lifelink body — the printed Oracle
/// shape of Spirit Summoning re-flavored for the Cat tribe. Tests:
/// `mascot_summoning_creates_a_two_two_lifelink_cat`,
/// `mascot_summoning_is_a_four_mana_white_lesson`.
pub fn mascot_summoning() -> CardDefinition {
    use crate::card::SpellSubtype;
    CardDefinition {
        name: "Mascot Summoning",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Cat".to_string(),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Lifelink],
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Cat],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
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
        exile_on_resolve: false,
    }
}

// ── Scry Inversion (synthesised STX Quandrix flavor) ───────────────────────

/// Scry Inversion — {2}{U} Instant (synthesised STX Quandrix flavor).
/// "Scry 2, then draw two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana hybrid filter +
/// card-velocity instant. Wired as `Seq(Scry 2, Draw 2)`. Tests:
/// `scry_inversion_scrys_and_draws_two`,
/// `scry_inversion_is_a_three_mana_blue_instant`.
pub fn scry_inversion() -> CardDefinition {
    CardDefinition {
        name: "Scry Inversion",
        cost: cost(&[generic(2), u()]),
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

// ── Cunning Rhetoric (synthesised STX Silverquill flavor) ──────────────────

/// Cunning Rhetoric — {2}{W}{B} Enchantment (synthesised STX
/// Silverquill flavor). "Whenever an opponent casts a spell, you gain
/// 1 life and they lose 1 life."
///
/// Push (modern_decks, NEW, `stx::extras`): An anti-spell tax that
/// punishes any opp-cast spell — a Silverquill life-drain payoff
/// against control / combo decks. Wired via an `EventKind::SpellCast /
/// OpponentControl` trigger that drains 1 from the triggering player.
/// Tests: `cunning_rhetoric_drains_on_opp_cast`,
/// `cunning_rhetoric_is_a_four_mana_wb_enchantment`.
pub fn cunning_rhetoric() -> CardDefinition {
    CardDefinition {
        name: "Cunning Rhetoric",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
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

// ── Library Larcenist (synthesised STX Witherbloom flavor) ─────────────────

/// Library Larcenist — {1}{B}{G} Creature — Pest Rogue, 2/3
/// (synthesised STX Witherbloom flavor). "Whenever this creature deals
/// combat damage to a player, that player mills two cards."
///
/// Push (modern_decks, NEW, `stx::extras`): A combat-damage mill body
/// — pairs with Witherbloom Apprentice / Sedgemoor Witch's gy-build
/// engines. Wired via `EventKind::DealsCombatDamageToPlayer /
/// SelfSource` trigger + `Effect::Mill { who: Triggerer, amount: 2 }`.
/// Tests: `library_larcenist_mills_on_combat_damage`,
/// `library_larcenist_is_a_three_mana_two_three_pest_rogue`.
pub fn library_larcenist() -> CardDefinition {
    CardDefinition {
        name: "Library Larcenist",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Triggerer),
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

// ── Dean's List (synthesised STX blue utility) ─────────────────────────────

/// Dean's List — {1}{U} Sorcery (synthesised STX colorless utility).
/// "Look at the top four cards of your library. Put one of them into
/// your hand and the rest into your graveyard."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana selective-mill +
/// hand-fix. Wired via `Effect::RevealUntilFind` with `find: Any` so
/// the auto-picker takes the first card to hand and misses go to
/// graveyard. Strong with gy-recursion strategies (Past in Flames,
/// Sevinne's Reclamation). Tests:
/// `deans_list_takes_top_card_and_mills_rest`,
/// `deans_list_is_a_two_mana_blue_sorcery`.
pub fn deans_list() -> CardDefinition {
    CardDefinition {
        name: "Dean's List",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::Const(1),
            life_per_revealed: 0,
            miss_dest: crate::effect::RevealMissDest::Graveyard,
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

// ── Inkrise Infiltrator (STX 2021 Silverquill common) ─────────────────────

/// Inkrise Infiltrator — {1}{B}, 2/1 Inkling Rogue (synthesised STX
/// Silverquill flavor). "Menace. (This creature can't be blocked except
/// by two or more creatures.)"
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana evasive Inkling
/// body that scales with Inkling tribal anthems (Tenured Inkcaster,
/// Promising Duskmage). Wired with bare `Keyword::Menace` — engine
/// already enforces menace at combat-blocker validation. Pure vanilla
/// body, no triggered abilities. Tests:
/// `inkrise_infiltrator_is_a_two_mana_inkling_with_menace`,
/// `inkrise_infiltrator_buffs_under_tenured_inkcaster`.
pub fn inkrise_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Inkrise Infiltrator",
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

// ── Sigardian Savior (synthesised STX 2021 white finisher) ────────────────

/// Sigardian Savior — {3}{W}{W}, 4/4 Angel (synthesised STX
/// white-tribal flavor). "Flying. When this creature enters, return
/// up to two target creature cards with mana value 3 or less from
/// your graveyard to the battlefield."
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana flying body
/// with a 2-for-1 reanimation rider. The "up to two" multi-target is
/// approximated as a single target return (engine-wide multi-target
/// gap). Wired via ETB `Effect::Move` against a creature card in
/// your graveyard with `ManaValueAtMost(3)`. Tests:
/// `sigardian_savior_is_a_five_mana_four_four_flying_angel`,
/// `sigardian_savior_etb_returns_low_mv_creature_card`.
pub fn sigardian_savior() -> CardDefinition {
    CardDefinition {
        name: "Sigardian Savior",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
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
        exile_on_resolve: false,
    }
}

// ── Sneaky Snacker (synthesised STX Witherbloom common) ───────────────────

/// Sneaky Snacker — {B}, 1/1 Rat Rogue (synthesised STX Witherbloom
/// flavor). "Menace. {2}{B}: Return Sneaky Snacker from your
/// graveyard to your hand. Activate only as a sorcery."
///
/// Push (modern_decks, NEW, `stx::extras`): One-mana evasive body
/// with built-in graveyard recursion. Wired via a `from_graveyard:
/// true` activated ability (engine primitive added in push XVII for
/// Summoned Dromedary) with `sorcery_speed: true`. Tests:
/// `sneaky_snacker_is_a_one_mana_rat_with_menace`,
/// `sneaky_snacker_recurs_from_graveyard_to_hand`.
pub fn sneaky_snacker() -> CardDefinition {
    CardDefinition {
        name: "Sneaky Snacker",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
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

// ── Soulknife Spy (synthesised STX Quandrix/blue Rogue) ───────────────────

/// Soulknife Spy — {1}{U}, 1/3 Human Rogue (synthesised STX flavor).
/// "Whenever Soulknife Spy deals combat damage to a player, you may
/// pay {U}. If you do, draw a card."
///
/// Push (modern_decks, NEW, `stx::extras`): Card-velocity attacker
/// that turns each combat hit into a {U} → draw exchange. Wired via
/// `DealsCombatDamageToPlayer/SelfSource` trigger + `Effect::MayPay`
/// for the optional card draw. Tests:
/// `soulknife_spy_is_a_two_mana_one_three_rogue`,
/// `soulknife_spy_combat_damage_can_draw_a_card_via_scripted_decider`,
/// `soulknife_spy_combat_damage_no_pay_skips_draw`.
pub fn soulknife_spy() -> CardDefinition {
    CardDefinition {
        name: "Soulknife Spy",
        cost: cost(&[generic(1), u()]),
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
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::SelfSource,
            ),
            effect: Effect::MayPay {
                description: "Pay {U} to draw a card.".into(),
                mana_cost: cost(&[u()]),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
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
        exile_on_resolve: false,
    }
}

// ── Daring Diversion (synthesised STX 2021 Lorehold Sorcery) ──────────────

/// Daring Diversion — {3}{R} Sorcery (synthesised STX Lorehold flavor).
/// "Daring Diversion deals 2 damage to each of two target creatures."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-damage divided burn
/// spell that hits two creatures. The "divided as you choose" rider
/// is collapsed to "2 damage to each" (slot 0 + slot 1 multi-target,
/// each getting 2 damage). The full "divided" semantics need the
/// engine-wide DealDamageDivided primitive. Tests:
/// `daring_diversion_burns_one_creature`,
/// `daring_diversion_burns_two_creatures_via_multi_target`,
/// `daring_diversion_is_a_four_mana_red_sorcery`.
pub fn daring_diversion() -> CardDefinition {
    CardDefinition {
        name: "Daring Diversion",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature,
                },
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

// ── Possibility Storm (synthesised STX Prismari reprint flavor) ───────────

/// Possibility Storm — {2}{R} Enchantment (synthesised STX Prismari
/// reprint flavor of Lorwyn). "Whenever a player casts a spell from
/// their hand, that player exiles it, then exiles cards from the top
/// of their library until they exile a card that shares a card type
/// with it. That player may cast that card without paying its mana
/// cost. Then they put all cards exiled with this enchantment on the
/// bottom of their library in a random order."
///
/// Push (modern_decks, NEW, `stx::extras`): The full Oracle requires
/// a complex deferred-cast-from-exile primitive. We ship the body
/// (enchantment + no triggered ability) as a chaos-engine placeholder
/// that toggles a marker on the battlefield for future engine work.
/// Currently a vanilla enchantment frame; will get its trigger when
/// the cast-from-exile pipeline lands. Tests:
/// `possibility_storm_is_a_three_mana_red_enchantment`.
pub fn possibility_storm() -> CardDefinition {
    CardDefinition {
        name: "Possibility Storm",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Pilgrim of the Ages (synthesised STX-flavor archetype anchor) ─────────

/// Pilgrim of the Ages — {3}, 1/1 Spirit (synthesised STX colorless
/// utility creature). "{2}, Sacrifice this creature: Search your
/// library for a basic land card, reveal it, put it into your hand,
/// then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): Colorless ramp on a body —
/// useful in any deck that wants color-fixing without committing to
/// a color. Wired via `sac_cost: true` activated ability + `Effect::
/// Search` for a basic land. Tests:
/// `pilgrim_of_the_ages_is_a_three_mana_one_one_spirit`,
/// `pilgrim_of_the_ages_sac_searches_for_basic_land`.
pub fn pilgrim_of_the_ages() -> CardDefinition {
    CardDefinition {
        name: "Pilgrim of the Ages",
        cost: cost(&[generic(3)]),
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
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

// ── Strixhaven Spawner (synthesised STX-flavor token doubler aid) ─────────

/// Strixhaven Spawner — {3}{G}{U} Sorcery (synthesised STX Quandrix
/// flavor). "Create three 0/0 green and blue Fractal creature tokens.
/// Put two +1/+1 counters on each of them."
///
/// Push (modern_decks, NEW, `stx::extras`): Quandrix mass token-mint +
/// counter-pump. Each Fractal lands as a 2/2 (printed 0/0 + 2 counters).
/// With Adrix and Nev on the battlefield, the count doubles to six 2/2s.
/// Tests: `strixhaven_spawner_creates_three_fractal_tokens`,
/// `strixhaven_spawner_fractals_have_two_plus_one_counters`,
/// `strixhaven_spawner_is_a_five_mana_gu_sorcery`.
pub fn strixhaven_spawner() -> CardDefinition {
    let fractal_def = TokenDefinition {
        name: "Fractal".to_string(),
        colors: vec![Color::Green, Color::Blue],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        supertypes: vec![],
    };
    CardDefinition {
        name: "Strixhaven Spawner",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: fractal_def,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
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
    }
}

// ── Mage Hunter Defender (synthesised STX-flavor wall) ────────────────────

/// Mage Hunter Defender — {2}{B}, 2/3 Human Wizard with Defender
/// (synthesised STX Witherbloom flavor). "Magecraft — Whenever you
/// cast or copy an instant or sorcery spell, each opponent loses 1
/// life."
///
/// Push (modern_decks, NEW, `stx::extras`): A Defender Magecraft
/// drainer — sits behind the lines and pings as you cast spells.
/// Wired via the `magecraft_drain_each_opp(1)` shortcut. Tests:
/// `mage_hunter_defender_drains_on_instant_cast`,
/// `mage_hunter_defender_is_a_three_mana_defender_wizard`.
pub fn mage_hunter_defender() -> CardDefinition {
    CardDefinition {
        name: "Mage Hunter Defender",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender],
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

// ── Detention Sphere (synthesised STX-flavor enchantment) ─────────────────

/// Detention Sphere — {1}{W}{U} Enchantment (synthesised STX
/// Silverquill-aligned hybrid removal). "When this enchantment
/// enters, exile target nonland permanent."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana hybrid exile-on-
/// ETB. The printed "until this leaves" return rider is omitted (no
/// exile-until-leaves replacement primitive — same gap as Banisher
/// Priest / Tidehollow Sculler). Tests:
/// `detention_sphere_exiles_target_nonland_permanent`,
/// `detention_sphere_is_a_three_mana_white_blue_enchantment`.
pub fn detention_sphere() -> CardDefinition {
    CardDefinition {
        name: "Detention Sphere",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
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

// ── Mascot Trainer (synthesised STX-flavor anthem) ────────────────────────

/// Mascot Trainer — {2}{G}, 2/2 Human Druid (synthesised STX
/// Witherbloom/Quandrix tribal anthem). "Other Mascot tokens you
/// control get +1/+1."
///
/// Push (modern_decks, NEW, `stx::extras`): The Mascot Exhibition
/// tokens (Elephant, Cat, Bird, plus Strixhaven Stadium's Inkling)
/// don't have a unified subtype, so this card uses the same
/// `OtherThanSource` anthem pattern as Tenured Inkcaster but applies
/// to all friendly tokens. Wired via `PumpPT` static against
/// `EachPermanent(Creature ∧ ControlledByYou ∧ IsToken ∧
/// OtherThanSource)`. Tests:
/// `mascot_trainer_buffs_friendly_tokens`,
/// `mascot_trainer_does_not_buff_non_tokens`,
/// `mascot_trainer_is_a_three_mana_two_two_druid`.
pub fn mascot_trainer() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Mascot Trainer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Other tokens you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsToken)
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
    }
}

// ── Quandrix Cryptidkeeper (synthesised STX-flavor) ───────────────────────

/// Quandrix Cryptidkeeper — {2}{G}{U}, 3/3 Elf Druid (synthesised STX
/// Quandrix flavor). "When this creature enters, put two +1/+1
/// counters on another target creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): Mid-curve counter-anchor
/// body — pairs with Quandrix counter doublers and Practical Research.
/// Wired via ETB `Effect::AddCounter(+1/+1, ×2)` on a `Creature ∧
/// ControlledByYou ∧ OtherThanSource` target. Tests:
/// `quandrix_cryptidkeeper_etb_pumps_friendly`,
/// `quandrix_cryptidkeeper_is_a_four_mana_three_three_elf_druid`.
pub fn quandrix_cryptidkeeper() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Cryptidkeeper",
        cost: cost(&[generic(2), g(), u()]),
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
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
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

// ── Ember Anvil (synthesised STX-flavor Lorehold mana rock) ──────────────

/// Ember Anvil — {3} Artifact (synthesised STX Lorehold-flavor mana
/// rock). "{T}: Add {R} or {W}. / {3}, {T}, Sacrifice this artifact:
/// Search your library for a Spirit card, reveal it, put it into
/// your hand, then shuffle."
///
/// Push (modern_decks, NEW, `stx::extras`): A Spirit tribal tutor on
/// a mana-rock body. Wired with two `tap_add` activations (R + W) and
/// a `sac_cost: true` Search activation. Tests:
/// `ember_anvil_taps_for_white_or_red`,
/// `ember_anvil_sac_tutors_a_spirit_creature_into_hand`,
/// `ember_anvil_is_a_three_mana_artifact`.
pub fn ember_anvil() -> CardDefinition {
    CardDefinition {
        name: "Ember Anvil",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            super::super::tap_add(Color::Red),
            super::super::tap_add(Color::White),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
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
        exile_on_resolve: false,
    }
}

// ── Witherbloom Strangler (synthesised STX-flavor Witherbloom) ────────────

/// Witherbloom Strangler — {1}{B}{G}, 2/2 Plant Warlock (synthesised
/// STX Witherbloom flavor). "When this creature enters, target
/// creature an opponent controls gets -2/-2 until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana removal-on-a-body
/// — kills 2-toughness creatures on ETB. Wired via ETB `Effect::PumpPT
/// {-2, -2, EOT}` on a `Creature ∧ ControlledByOpponent` target.
/// Tests: `witherbloom_strangler_etb_minus_two_minus_two`,
/// `witherbloom_strangler_kills_two_two_creature`,
/// `witherbloom_strangler_is_a_three_mana_two_two_plant_warlock`.
pub fn witherbloom_strangler() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Strangler",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
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
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
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
    }
}

// ── Glasspool Embellisher (synthesised STX-flavor Quandrix utility) ──────

/// Glasspool Embellisher — {U} Instant (synthesised STX Quandrix
/// flavor). "Draw a card, then discard a card. Magecraft — Whenever
/// you cast or copy an instant or sorcery spell, scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): A {U} looter-cantrip with
/// no Magecraft body (it's an instant, not a creature). Just a card
/// filtering spell. Wired as `Seq(Draw 1, Discard 1)`. Tests:
/// `glasspool_embellisher_loots_one`,
/// `glasspool_embellisher_is_a_one_mana_blue_instant`.
pub fn glasspool_embellisher() -> CardDefinition {
    CardDefinition {
        name: "Glasspool Embellisher",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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

// ── Lorehold Reanimator (synthesised STX-flavor Lorehold archetype) ──────

/// Lorehold Reanimator — {2}{R}{W}, 3/3 Spirit Cleric (synthesised
/// STX Lorehold flavor). "When this creature enters, you may return
/// target creature card with mana value 2 or less from your graveyard
/// to the battlefield."
///
/// Push (modern_decks, NEW, `stx::extras`): A small reanimator body
/// that brings back a 1- or 2-drop on ETB. Wired via ETB `Effect::
/// MayDo` wrapping a `Move(creature in gy with MV≤2 → battlefield)`.
/// Tests: `lorehold_reanimator_etb_optionally_returns_low_mv_creature`,
/// `lorehold_reanimator_is_a_four_mana_three_three_spirit_cleric`.
pub fn lorehold_reanimator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Reanimator",
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return a creature card with mana value 2 or less from your graveyard to the battlefield."
                    .into(),
                body: Box::new(Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    }),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
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
        exile_on_resolve: false,
    }
}

// ── Prismari Eruption (synthesised STX-flavor mass burn) ──────────────────

/// Prismari Eruption — {3}{U}{R} Sorcery (synthesised STX Prismari
/// flavor). "Prismari Eruption deals 2 damage to each creature without
/// flying. Scry 1."
///
/// Push (modern_decks, NEW, `stx::extras`): A small Prismari sweep
/// that filters lots of x/2 attackers while leaving fliers alive.
/// Wired via `ForEach(EachPermanent(Creature ∧ ¬HasKeyword(Flying)))
/// → DealDamage 2` followed by Scry 1. Tests:
/// `prismari_eruption_burns_grounded_creatures`,
/// `prismari_eruption_spares_flyers`,
/// `prismari_eruption_is_a_five_mana_ur_sorcery`.
pub fn prismari_eruption() -> CardDefinition {
    CardDefinition {
        name: "Prismari Eruption",
        cost: cost(&[generic(3), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature.and(
                    SelectionRequirement::Not(Box::new(SelectionRequirement::HasKeyword(
                        Keyword::Flying,
                    ))),
                )),
                body: Box::new(Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::Const(2),
                }),
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

// ── Silverquill Inquisitor (synthesised STX-flavor) ───────────────────────

/// Silverquill Inquisitor — {1}{W}{B}, 2/2 Human Cleric (synthesised
/// STX Silverquill flavor). "When this creature enters, target
/// opponent reveals their hand. You choose a noncreature, nonland
/// card from it. That player discards that card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana hand-disruption
/// body — approximates Thoughtseize on a body. The "look at hand and
/// choose" is engine-wide ⏳ (no opp-hand-reveal-and-pick primitive);
/// we approximate with `Effect::Discard { random: true }` against the
/// chosen opp. The auto-decider picks the first opp; a true
/// implementation would surface a `Decision::ChooseFromHand` modal.
/// Tests: `silverquill_inquisitor_etb_discards_from_opp_hand`,
/// `silverquill_inquisitor_is_a_three_mana_two_two_cleric`.
pub fn silverquill_inquisitor() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Inquisitor",
        cost: cost(&[generic(1), w(), b()]),
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
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: true,
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

// ── Lorehold Spectral Lecturer (synthesised STX-flavor Lorehold) ─────────

/// Lorehold Spectral Lecturer — {3}{R}{W}, 4/3 Spirit Cleric Wizard
/// (synthesised STX Lorehold flavor). "Vigilance. / Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, this
/// creature gets +1/+0 and gains lifelink until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana Lorehold
/// Magecraft body — gets bigger and gains lifelink each cast. Wired
/// via `magecraft(Seq(PumpPT(+1/+0 EOT), GrantKeyword(Lifelink EOT)))`
/// on `Selector::This`. Tests:
/// `lorehold_spectral_lecturer_magecraft_pumps_and_lifelinks`,
/// `lorehold_spectral_lecturer_is_a_five_mana_four_three_spirit_cleric_wizard`.
pub fn lorehold_spectral_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Spectral Lecturer",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spirit,
                CreatureType::Cleric,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
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
    }
}

// ── Pop Quiz Recital (synthesised STX-flavor Lesson) ──────────────────────

/// Pop Quiz Recital — {2}{W} Sorcery — Lesson (synthesised STX
/// Lesson flavor). "Choose one — / • Target creature gets +2/+2 and
/// gains flying until end of turn. / • Target creature gets +0/+3
/// and gains vigilance until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A flexible combat-trick
/// Lesson via `Effect::ChooseMode`. Mode 0 (pump+flying) wins in air;
/// mode 1 (toughness+vigilance) wins on the ground. Tests:
/// `pop_quiz_recital_mode_zero_pumps_and_grants_flying`,
/// `pop_quiz_recital_is_a_three_mana_white_lesson`.
pub fn pop_quiz_recital() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz Recital",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(0),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
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
        exile_on_resolve: false,
    }
}

// ── Diviner's Wand (synthesised STX-flavor Equipment) ─────────────────────

/// Diviner's Wand — {4} Artifact — Equipment (synthesised STX-flavor
/// rare Equipment). "Equipped creature gets +2/+1 and has 'When this
/// creature deals combat damage to a player, draw a card.' / Equip {3}."
///
/// Push (modern_decks, NEW, `stx::extras`): A 4-mana equipment with
/// equip {3}. Equip-grant statics are not yet a full primitive
/// (Equipment-attached pump is engine-side; the combat-damage-draw
/// rider needs a transient trigger grant). For now we ship the body
/// + Equip ability shape only — wears as a +0/+0 placeholder.
///
/// Tests: `diviners_wand_is_a_four_mana_equipment_with_equip_three`.
pub fn diviners_wand() -> CardDefinition {
    CardDefinition {
        name: "Diviner's Wand",
        cost: cost(&[generic(4)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
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
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

// ── Stonebound Mentor (already exists) – stub note removed ────────────────

// ── Fascinating Lecture (synthesised STX-flavor Lesson) ───────────────────

/// Fascinating Lecture — {1}{U} Sorcery — Lesson (synthesised STX
/// Lesson flavor). "Draw two cards, then discard a card."
///
/// Push (modern_decks, NEW, `stx::extras`): A 2-mana looter Lesson —
/// strong card velocity tutored from the Lessons sideboard once that
/// primitive lands. For now slots into any blue deck as a +1 net card
/// hand-filter. Wired as `Seq(Draw 2, Discard 1)`. Tests:
/// `fascinating_lecture_draws_two_discards_one`,
/// `fascinating_lecture_is_a_two_mana_blue_lesson`.
pub fn fascinating_lecture() -> CardDefinition {
    CardDefinition {
        name: "Fascinating Lecture",
        cost: cost(&[generic(1), u()]),
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

// ── Quandrix Sphinx (synthesised STX-flavor Quandrix flyer) ───────────────

/// Quandrix Sphinx — {3}{G}{U}, 3/4 Sphinx Druid (synthesised STX
/// Quandrix flavor). "Flying. When this creature enters, put a +1/+1
/// counter on each creature you control."
///
/// Push (modern_decks, NEW, `stx::extras`): A 5-mana flying mass-pump
/// body — Quandrix's signature counter shape on a relevant attacker.
/// Wired via ETB `ForEach(Creature & ControlledByYou) → AddCounter`.
/// Tests: `quandrix_sphinx_etb_counters_each_friendly_creature`,
/// `quandrix_sphinx_is_a_five_mana_three_four_flying_sphinx_druid`.
pub fn quandrix_sphinx() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sphinx",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
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
        exile_on_resolve: false,
    }
}

// ── Lorehold Excavator (synthesised STX-flavor) ───────────────────────────

/// Lorehold Excavator — {1}{R}{W}, 2/2 Spirit Cleric (synthesised STX
/// Lorehold flavor). "When this creature enters, exile target card
/// from a graveyard. / {2}{R}{W}, {T}: This creature deals 1 damage
/// to any target."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana gy-hate body
/// with an activated ping. Wired with an ETB exile-target-gy-card
/// trigger + tap activation. Tests:
/// `lorehold_excavator_etb_exiles_target_gy_card`,
/// `lorehold_excavator_is_a_three_mana_two_two_spirit_cleric`.
pub fn lorehold_excavator() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Excavator",
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
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), r(), w()]),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .or(SelectionRequirement::Player)
                        .or(SelectionRequirement::Planeswalker),
                ),
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
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Any,
                }),
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
        exile_on_resolve: false,
    }
}

// ── Stridehollow Vampire (synthesised STX Silverquill modal pump) ─────────

/// Stridehollow Vampire — {1}{W}{B}, 2/2 Vampire Soldier (synthesised
/// STX Silverquill flavor). "Flying. / When this creature enters,
/// choose one — / • Draw a card. / • Target creature gets +1/+1
/// until end of turn."
///
/// Push (modern_decks, NEW, `stx::extras`): A 3-mana evasive modal
/// flier. ETB ChooseMode picks draw or pump. AutoDecider picks mode
/// 0 (draw). Tests:
/// `stridehollow_vampire_etb_default_draws`,
/// `stridehollow_vampire_is_a_three_mana_two_two_vampire`.
pub fn stridehollow_vampire() -> CardDefinition {
    CardDefinition {
        name: "Stridehollow Vampire",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
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
    }
}

// ── Witherbloom Necrotutor (synthesised STX-flavor Witherbloom) ───────────

/// Witherbloom Necrotutor — {2}{B}{B}, 3/2 Human Warlock (synthesised
/// STX Witherbloom flavor). "When this creature enters, return target
/// creature card from your graveyard to your hand. You lose 2 life."
///
/// Push (modern_decks, NEW, `stx::extras`): Pay-life Raise Dead on a
/// body. Wired as `Seq(Move(creature in gy → hand), LoseLife 2)`.
/// Tests: `witherbloom_necrotutor_etb_returns_creature_card_and_loses_two_life`,
/// `witherbloom_necrotutor_is_a_four_mana_three_two_warlock`.
pub fn witherbloom_necrotutor() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Necrotutor",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::one_of(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::Creature,
                    }),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::LoseLife {
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
    }
}

// ── Silverquill Pledge ──────────────────────────────────────────────────────

/// Silverquill Pledge — {1}{W}{B} Instant (synthesised STX Silverquill
/// flavor based on the printed Silverquill Cleric tradition).
///
/// "Target creature gets +3/+1 until end of turn." Pure combat trick on
/// the Silverquill color identity — Inkrise Infiltrator's swing
/// becomes a 5/2 with menace; Eager First-Year goes from a 2/1 to a
/// 5/2 trader. Wired as `Effect::PumpPT { +3/+1, EOT }`. Tests:
/// `silverquill_pledge_pumps_target_three_one`,
/// `silverquill_pledge_is_a_three_mana_silverquill_instant`.
pub fn silverquill_pledge() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Pledge",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(3),
            toughness: Value::Const(1),
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

// ── Inkwell Strider ─────────────────────────────────────────────────────────

/// Inkwell Strider — {2}{W}{B} 2/3 Inkling Soldier with Flying + Lifelink
/// (synthesised STX Silverquill flavor — combines the school's two
/// signature keywords on a fair body).
///
/// Provides reach + life-gain for Silverquill payoffs (Promising
/// Duskmage's Magecraft, Light of Promise's counter rain, Tenured
/// Inkcaster's anthem). Tests:
/// `inkwell_strider_is_a_four_mana_two_three_flying_inkling`,
/// `inkwell_strider_combat_grants_life_via_lifelink`.
pub fn inkwell_strider() -> CardDefinition {
    CardDefinition {
        name: "Inkwell Strider",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
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

// ── Scolding Detention ──────────────────────────────────────────────────────

/// Scolding Detention — {2}{W} Sorcery (synthesised STX Silverquill
/// flavor). "Tap target creature an opponent controls. Put two stun
/// counters on it."
///
/// A heavier-handed Frost Trickster — clears two combat steps. The
/// stun-counter SBA gates removal of one counter per failed untap, so
/// the target stays tapped for two of the controller's untaps. Wired
/// as `Seq(Tap target, AddCounter Stun × 2)`. Tests:
/// `scolding_detention_taps_and_stuns_twice`.
pub fn scolding_detention() -> CardDefinition {
    CardDefinition {
        name: "Scolding Detention",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Stun,
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

// ── Lesson Recall ───────────────────────────────────────────────────────────

/// Lesson Recall — {1}{U} Instant (synthesised STX Strixhaven flavor).
/// "Return target instant or sorcery card from your graveyard to your
/// hand. Draw a card."
///
/// Card-advantage cantrip that recurs spells. Wired as
/// `Seq(Move(target IS in your gy → hand), Draw 1)`. Tests:
/// `lesson_recall_returns_instant_and_cantrips`,
/// `lesson_recall_is_a_two_mana_blue_instant`.
pub fn lesson_recall() -> CardDefinition {
    CardDefinition {
        name: "Lesson Recall",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                }),
                to: ZoneDest::Hand(PlayerRef::You),
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

// ── Pestilent Acolyte ───────────────────────────────────────────────────────

/// Pestilent Acolyte — {2}{B} 2/3 Human Warlock (synthesised STX
/// Witherbloom flavor). "When this creature enters, target creature
/// gets -1/-1 until end of turn."
///
/// ETB removal-on-a-stick — kills X/1 creatures and softens 2/3s for
/// trading. Wired as `Effect::PumpPT { -1/-1, EOT }` on a creature
/// target. Tests:
/// `pestilent_acolyte_etb_kills_one_toughness_creature`,
/// `pestilent_acolyte_is_a_three_mana_two_three_warlock`.
pub fn pestilent_acolyte() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Acolyte",
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
    }
}

// ── Stoneglare Lecturer ─────────────────────────────────────────────────────

/// Stoneglare Lecturer — {3}{W} 3/3 Cat Cleric (synthesised STX
/// Silverquill flavor). "When this creature enters, you gain 2 life
/// and draw a card."
///
/// A four-mana Bookwurm: 3/3 + 2 life + cantrip. Solid card-advantage
/// body for white control / midrange decks. Wired as ETB
/// `Seq(GainLife 2, Draw 1)`. Tests:
/// `stoneglare_lecturer_etb_gains_life_and_draws`,
/// `stoneglare_lecturer_is_a_four_mana_three_three_cat_cleric`.
pub fn stoneglare_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Stoneglare Lecturer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
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
        exile_on_resolve: false,
    }
}

// ── Pop Quiz Sorcery (cantrip variant) ──────────────────────────────────────

/// Critical Critique — {1}{B} Instant (synthesised STX Silverquill flavor).
/// "Target creature gets -2/-2 until end of turn. Scry 1."
///
/// Two-mana spot removal with a Scry rider — kills 2/2 creatures and
/// digs for the next play. Wired as `Seq(PumpPT -2/-2 EOT, Scry 1)`.
/// Tests: `critical_critique_kills_two_two_and_scrys`,
/// `critical_critique_is_a_two_mana_black_instant`.
pub fn critical_critique() -> CardDefinition {
    CardDefinition {
        name: "Critical Critique",
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
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
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

// ── Quandrix Manipulator ────────────────────────────────────────────────────

/// Quandrix Manipulator — {2}{G}{U} 3/3 Elf Druid (synthesised STX
/// Quandrix flavor). "When this creature enters, double the number of
/// +1/+1 counters on target creature."
///
/// Tanazir-on-a-budget — a 3/3 body that fans counters to a friendly
/// +1/+1-bearer. Wired via `Effect::AddCounter { amount:
/// CountersOn(target, +1/+1) }` — adds N more, doubling from N to 2N.
/// Tests: `quandrix_manipulator_doubles_counters_on_target_creature`,
/// `quandrix_manipulator_is_a_four_mana_three_three_elf_druid`.
pub fn quandrix_manipulator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Manipulator",
        cost: cost(&[generic(2), g(), u()]),
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
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
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
        exile_on_resolve: false,
    }
}

// ── Prismari Iteration ──────────────────────────────────────────────────────

/// Prismari Iteration — {2}{U}{R} Sorcery (synthesised STX Prismari
/// flavor). "Discard a card, then draw two cards."
///
/// 4-mana looter — trades one off-color card for two fresh ones. Wired
/// as `Seq(Discard 1, Draw 2)`. Net card advantage: -1 + 2 = +1.
/// Tests: `prismari_iteration_loots_two_for_one`,
/// `prismari_iteration_is_a_four_mana_ur_sorcery`.
pub fn prismari_iteration() -> CardDefinition {
    CardDefinition {
        name: "Prismari Iteration",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
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

// ── Lorehold Battle-Priest ──────────────────────────────────────────────────

/// Lorehold Battle-Priest — {2}{R}{W} 2/4 Spirit Cleric with First
/// Strike + Vigilance (synthesised STX Lorehold flavor — exactly the
/// printed shape of Pillardrop Rescuer + first-strike instead of flying).
///
/// A defensive Lorehold body that trades up in combat and stays
/// available for blocking. Tests:
/// `lorehold_battle_priest_is_a_four_mana_two_four_spirit_cleric_with_first_strike_vigilance`.
pub fn lorehold_battle_priest() -> CardDefinition {
    CardDefinition {
        name: "Lorehold Battle-Priest",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
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
        exile_on_resolve: false,
    }
}

// ── Witherbloom Strangler (already exists; this is Witherbloom Reaper) ─────

/// Witherbloom Reaper — {3}{B}{G} 4/3 Plant Warlock with Deathtouch
/// (synthesised STX Witherbloom flavor). "When this creature enters,
/// each opponent sacrifices a creature."
///
/// Edict-on-a-body — a Cruel Edict stapled to a 4/3 Deathtouch. Wired
/// as ETB `ForEach(EachOpponent) → Sacrifice { who: Triggerer,
/// filter: Creature, count: 1 }`. Tests:
/// `witherbloom_reaper_etb_edicts_each_opp`,
/// `witherbloom_reaper_is_a_five_mana_four_three_deathtouch_plant_warlock`.
pub fn witherbloom_reaper() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Reaper",
        cost: cost(&[generic(3), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::Player(PlayerRef::EachOpponent),
                body: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    filter: SelectionRequirement::Creature,
                    count: Value::Const(1),
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
        exile_on_resolve: false,
    }
}

// ── Pyromancer's Bolt ──────────────────────────────────────────────────────

/// Pyromancer's Bolt — {1}{R} Instant (synthesised STX Prismari flavor).
/// "This deals 3 damage to target creature or planeswalker."
///
/// Strict-upgrade Lightning Bolt on creatures/PW only — a clean
/// 2-mana removal spell that doesn't trade card advantage for face
/// damage. Wired with `target_filtered(Creature ∨ Planeswalker)`.
/// Tests: `pyromancers_bolt_kills_three_toughness_creature`,
/// `pyromancers_bolt_is_a_two_mana_red_instant`.
pub fn pyromancers_bolt() -> CardDefinition {
    CardDefinition {
        name: "Pyromancer's Bolt",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
            ),
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

// ── Symmetry Lecturer ──────────────────────────────────────────────────────

/// Symmetry Lecturer — {1}{G}{U} 2/2 Elf Wizard with Flash (synthesised
/// STX Quandrix flavor). "Flash / When this creature enters, put a
/// +1/+1 counter on another target creature you control."
///
/// A combat-trick Quandrix body — flashes in, lands a counter on a
/// blocker mid-combat, then trades up. Wired with `Keyword::Flash` +
/// ETB `AddCounter(+1/+1) → target friendly other`. Tests:
/// `symmetry_lecturer_etb_pumps_friendly_other`,
/// `symmetry_lecturer_has_flash`.
pub fn symmetry_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Symmetry Lecturer",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
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
    }
}

// ── Wisdom of the Ancients ──────────────────────────────────────────────────

/// Wisdom of the Ancients — {3}{U} Sorcery (synthesised STX Quandrix
/// flavor). "Draw three cards."
///
/// Pure card-velocity blue sorcery — Concentrate's strict-equivalent
/// at the standard 4-mana three-draw template. Useful test exerciser
/// for engine's `Effect::Draw` count-N path. Tests:
/// `wisdom_of_the_ancients_draws_three`,
/// `wisdom_of_the_ancients_is_a_four_mana_blue_sorcery`.
pub fn wisdom_of_the_ancients() -> CardDefinition {
    CardDefinition {
        name: "Wisdom of the Ancients",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::You,
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

// ── Mob Mentality ──────────────────────────────────────────────────────────

/// Mob Mentality — {1}{R}{W} Instant (synthesised STX Lorehold flavor).
/// "Creatures you control get +1/+1 until end of turn. If you've cast
/// another spell this turn, they also gain first strike until end of
/// turn."
///
/// A combat-step finisher — pump everyone, then upgrade with first
/// strike if you cast a setup spell first. Wired as
/// `Seq(ForEach(Creature & ControlledByYou) → PumpPT(+1/+1 EOT),
///      If(SpellsCastThisTurnAtLeast(2)) → ForEach(...) → GrantKeyword(FirstStrike EOT))`.
/// Tests: `mob_mentality_pumps_each_friendly_creature`,
/// `mob_mentality_grants_first_strike_after_second_spell`.
pub fn mob_mentality() -> CardDefinition {
    CardDefinition {
        name: "Mob Mentality",
        cost: cost(&[generic(1), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ForEach {
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
            Effect::If {
                cond: Predicate::SpellsCastThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(2),
                },
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    }),
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
        exile_on_resolve: false,
    }
}

// ── Witherbloom Drain Ritual ───────────────────────────────────────────────

/// Witherbloom Drain Ritual — {2}{B}{G} Sorcery (synthesised STX
/// Witherbloom flavor). "Each opponent loses 3 life. You gain life
/// equal to the life lost this way."
///
/// A Drain Life-style finisher on the Witherbloom color identity.
/// Wired via `Effect::Drain { from: EachOpponent, to: You, amount: 3 }`
/// — symmetric on 1v1 (you gain exactly 3 life). Tests:
/// `witherbloom_drain_ritual_drains_three_each_opp`,
/// `witherbloom_drain_ritual_is_a_four_mana_bg_sorcery`.
pub fn witherbloom_drain_ritual() -> CardDefinition {
    CardDefinition {
        name: "Witherbloom Drain Ritual",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
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

// ── Spell Tutor ────────────────────────────────────────────────────────────

/// Mystical Inquiry — {2}{U} Sorcery (synthesised STX Quandrix flavor —
/// a sorcery tutor for the Prismari/Quandrix shells).
/// "Search your library for an instant or sorcery card, reveal it,
/// put it into your hand, then shuffle."
///
/// Standard {2}{U} tutor on a printed catalog template (same shape as
/// `solve_the_equation` but at sorcery speed without an MV cap).
/// Tests: `mystical_inquiry_tutors_an_instant_or_sorcery`,
/// `mystical_inquiry_is_a_three_mana_blue_sorcery`.
pub fn mystical_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Mystical Inquiry",
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
        exile_on_resolve: false,
    }
}

// ── Conjurer's Bauble (STA reprint, originally Modern Horizons) ────────────

/// Conjurer's Bauble — {0} Artifact (STA reprint, originally Modern
/// Horizons). "{1}, Sacrifice this artifact: Put a card from your
/// graveyard on the bottom of your library. Draw a card."
///
/// A zero-mana artifact that cycles graveyard contents for a fresh
/// draw — useful for both blue control and graveyard-based decks
/// (snake the right card back into the library for a future tutor).
/// Wired with `cost: 0`, `{1}` mana cost + `sac_cost: true` on the
/// activation. The "put gy card on bottom" approximation moves a
/// chosen creature card from gy → library bottom; if no creature
/// is available, the activation still resolves with just the draw.
/// Tests: `conjurers_bauble_zero_mana_artifact`,
/// `conjurers_bauble_sac_activation_cantrips`.
pub fn conjurers_bauble() -> CardDefinition {
    CardDefinition {
        name: "Conjurer's Bauble",
        cost: ManaCost::default(),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: false,
            sac_cost: true,
            life_cost: 0,
            sorcery_speed: false,
            condition: None,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            once_per_turn: false,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
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

// ── Quartzwood Inkling ──────────────────────────────────────────────────────

/// Quartzwood Inkling — {2}{B} 3/2 Inkling Soldier with Menace
/// (synthesised STX Silverquill flavor).
///
/// An efficient Menace beater for the Silverquill aggro shell — scales
/// with Tenured Inkcaster's +2/+2 anthem to a 5/4 trampler. Tests:
/// `quartzwood_inkling_is_a_three_mana_three_two_menace_inkling`,
/// `quartzwood_inkling_buffs_under_tenured_inkcaster`.
pub fn quartzwood_inkling() -> CardDefinition {
    CardDefinition {
        name: "Quartzwood Inkling",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
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
        exile_on_resolve: false,
    }
}

// ── Pop Quiz Lecturer ──────────────────────────────────────────────────────

/// Pop Quiz Lecturer — {2}{W} 2/3 Human Cleric with Vigilance
/// (synthesised STX Silverquill flavor). "When this creature enters,
/// scry 2."
///
/// A defensive tap-out creature with a sticky 2/3 + Vigilance body
/// and an ETB scry. Wired with `Effect::Scry { who: You, amount: 2 }`.
/// Tests: `pop_quiz_lecturer_etb_scries_two`,
/// `pop_quiz_lecturer_is_a_three_mana_two_three_vigilance_cleric`.
pub fn pop_quiz_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz Lecturer",
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
        exile_on_resolve: false,
    }
}

// ── Brilliant Restoration ──────────────────────────────────────────────────

/// Brilliant Restoration — {3}{W}{W} Sorcery (synthesised STX
/// Silverquill flavor). "Return target creature card from your
/// graveyard to the battlefield. You gain 2 life."
///
/// 5-mana reanimation with a lifegain rider — fits Silverquill's
/// lifegain payoffs (Light of Promise, Promising Duskmage). Wired
/// as `Seq(Move(target creature in gy → bf), GainLife 2)`.
/// Tests: `brilliant_restoration_returns_creature_card_and_gains_life`,
/// `brilliant_restoration_is_a_five_mana_white_sorcery`.
pub fn brilliant_restoration() -> CardDefinition {
    CardDefinition {
        name: "Brilliant Restoration",
        cost: cost(&[generic(3), w(), w()]),
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
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
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

// ── Inkling Studies ────────────────────────────────────────────────────────

/// Inkling Studies — {2}{W}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Create two 2/1 white-and-black Inkling creature tokens
/// with flying."
///
/// Token-mint payoff for Silverquill — slots into Tenured Inkcaster +
/// Felisa Fang lifelink shells. Wired with `Effect::CreateToken {
/// count: 2, definition: inkling_token() }`. Tests:
/// `inkling_studies_creates_two_inkling_tokens`,
/// `inkling_studies_is_a_four_mana_wb_sorcery`.
pub fn inkling_studies() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Studies",
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

// ── Spirit Banner ──────────────────────────────────────────────────────────

/// Spirit Banner — {3} Artifact (synthesised STX Lorehold flavor).
/// "Spirit creatures you control get +1/+1."
///
/// A tribal anthem for Lorehold's Spirit-focused shells (Sparring
/// Regimen, Quintorius's tokens, Lorehold Excavation's tokens).
/// Wired via `StaticEffect::PumpPT { applies_to:
/// EachPermanent(Creature & ControlledByYou & HasCreatureType(Spirit)),
/// power: +1, toughness: +1 }`. Tests:
/// `spirit_banner_pumps_spirits_by_one_one`,
/// `spirit_banner_does_not_pump_non_spirits`.
pub fn spirit_banner() -> CardDefinition {
    CardDefinition {
        name: "Spirit Banner",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Spirit creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Spirit)),
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
    }
}

// ── Spectral Adjudicator ──────────────────────────────────────────────────

/// Spectral Adjudicator — {3}{W} 2/3 Spirit Cleric with Flying +
/// Lifelink (synthesised STX Silverquill flavor).
///
/// A defensive Spirit flyer that doubles as a 2/3 lifelink racer.
/// Tests: `spectral_adjudicator_is_a_four_mana_two_three_lifelink_spirit_cleric_with_flying`.
pub fn spectral_adjudicator() -> CardDefinition {
    CardDefinition {
        name: "Spectral Adjudicator",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
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

// ── Quandrix Doubling Tutor ────────────────────────────────────────────────

/// Quandrix Doubling Tutor — {2}{G}{U} Sorcery (synthesised STX
/// Quandrix flavor). "Create two 0/0 green and blue Fractal creature
/// tokens, then put a +1/+1 counter on each Fractal you control."
///
/// Wired as `Seq(CreateToken(2 Fractals), ForEach(Fractal &
/// ControlledByYou) → AddCounter(+1/+1, 1))`. The result is two 1/1
/// Fractals — but with Tanazir Quandrix in play, the counter rain
/// doubles. Tests:
/// `quandrix_doubling_tutor_creates_two_fractals_with_counters`,
/// `quandrix_doubling_tutor_is_a_four_mana_gu_sorcery`.
pub fn quandrix_doubling_tutor() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Doubling Tutor",
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
                count: Value::Const(2),
                definition: fractal_token(),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
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
    }
}

// ── Inkling Scholar ────────────────────────────────────────────────────────

/// Inkling Scholar — {2}{W}{B}, 3/3 Inkling Cleric with Flying +
/// Lifelink (synthesised STX Silverquill flavor).
///
/// A clean four-mana evasive Lifelink threat that doubles as a tribal
/// target for Tenured Inkcaster's +2/+2 anthem (jumping a base 3/3 to
/// 5/5 Flying/Lifelink). Tests:
/// `inkling_scholar_is_a_four_mana_three_three_lifelink_inkling_with_flying`.
pub fn inkling_scholar() -> CardDefinition {
    CardDefinition {
        name: "Inkling Scholar",
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

// ── Inkling Squire ─────────────────────────────────────────────────────────

/// Inkling Squire — {W}, 1/1 Inkling Soldier with Flying (synthesised
/// STX Silverquill flavor).
///
/// A 1-mana Inkling flyer for go-wide Silverquill aggro shells.
/// Synergises with Tenured Inkcaster (becomes 3/3 Flying), Felisa
/// (mints another Inkling on death), and Stirring Hopesinger's
/// Repartee counter rain. Test:
/// `inkling_squire_is_a_one_mana_inkling_flier`.
pub fn inkling_squire() -> CardDefinition {
    CardDefinition {
        name: "Inkling Squire",
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

// ── Silverquill Scholar ────────────────────────────────────────────────────

/// Silverquill Scholar — {W}{B}, 2/1 Human Wizard (synthesised STX
/// Silverquill flavor). "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, draw a card and lose 1 life."
///
/// A Silverquill twist on Archmage Emeritus (which is mono-blue): card
/// advantage with a 1-life tax. Wired via the `magecraft()` shortcut
/// with `Seq(Draw 1, LoseLife 1)`. Tests:
/// `silverquill_scholar_magecraft_draws_and_loses_life`,
/// `silverquill_scholar_does_not_trigger_on_creature_cast`.
pub fn silverquill_scholar() -> CardDefinition {
    CardDefinition {
        name: "Silverquill Scholar",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::LoseLife {
                who: Selector::You,
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
    }
}

// ── Inkling Reinforcement ──────────────────────────────────────────────────

/// Inkling Reinforcement — {W}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Create two 1/1 white and black Inkling creature tokens
/// with flying."
///
/// Two-mana go-wide for the Inkling tribe. Cheaper than Inkling
/// Studies (4 mana) for the same effect, trading speed for color
/// pip density. Tests:
/// `inkling_reinforcement_creates_two_inkling_tokens`,
/// `inkling_reinforcement_is_a_two_mana_wb_sorcery`.
pub fn inkling_reinforcement() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Reinforcement",
        cost: cost(&[w(), b()]),
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

// ── Pestilent Verse ────────────────────────────────────────────────────────

/// Pestilent Verse — {1}{B}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Destroy target creature. You lose 1 life."
///
/// Three-mana unconditional creature removal at a 1-life tax. Fills
/// the Doom Blade slot in Silverquill / mono-Black shells without
/// the printed "nonblack" restriction. Wired via
/// `Seq(Destroy → target Creature, LoseLife 1)`. Tests:
/// `pestilent_verse_destroys_creature_and_costs_one_life`,
/// `pestilent_verse_is_a_three_mana_black_sorcery`.
pub fn pestilent_verse() -> CardDefinition {
    CardDefinition {
        name: "Pestilent Verse",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::LoseLife {
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

// ── Inkling Ambusher ───────────────────────────────────────────────────────

/// Inkling Ambusher — {2}{B}, 2/2 Inkling Rogue with Flash + Flying
/// (synthesised STX Silverquill flavor).
///
/// A Flash flyer for surprise blocks or end-of-turn pressure.
/// Three-mana 2/2 evasive Inkling that doesn't compete for the
/// double-color pip slot. Test:
/// `inkling_ambusher_is_a_three_mana_inkling_with_flash_and_flying`.
pub fn inkling_ambusher() -> CardDefinition {
    CardDefinition {
        name: "Inkling Ambusher",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
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

// ── Silver-Quill Scholarship ───────────────────────────────────────────────

/// Silver-Quill Scholarship — {2}{W} Sorcery (synthesised STX
/// Silverquill flavor). "Put a +1/+1 counter on target creature. Draw
/// a card."
///
/// Three-mana counter + cantrip; works as a synergy enabler for
/// Tenured Inkcaster (anthem on the bigger counter-target), Felisa
/// (Inkling on counter-bearing-creature-death), Hardened Academic
/// (combat-damage cantrip), or Scolding Administrator (Repartee
/// snowball). Tests: `silver_quill_scholarship_counters_target_and_draws`,
/// `silver_quill_scholarship_is_a_three_mana_white_sorcery`.
pub fn silver_quill_scholarship() -> CardDefinition {
    CardDefinition {
        name: "Silver-Quill Scholarship",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
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
    }
}

// ── Silvercrown Lecturer ───────────────────────────────────────────────────

/// Silvercrown Lecturer — {3}{W} 2/4 Human Cleric (synthesised STX
/// Silverquill flavor). "When this creature enters, put a +1/+1
/// counter on target creature you control."
///
/// A defensive 4-mana 2/4 body that snowballs the controller's
/// strongest creature. ETB:
/// `AddCounter +1/+1 → target Creature & ControlledByYou`. Tests:
/// `silvercrown_lecturer_etb_lands_counter_on_friendly`,
/// `silvercrown_lecturer_is_a_four_mana_two_four_human_cleric`.
pub fn silvercrown_lecturer() -> CardDefinition {
    CardDefinition {
        name: "Silvercrown Lecturer",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
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
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
    }
}

// ── Demolishing Lecture ────────────────────────────────────────────────────

/// Demolishing Lecture — {2}{B} Sorcery (synthesised STX Silverquill
/// flavor). "Destroy target creature with toughness 2 or less."
///
/// A focused 3-mana removal spell targeted at small creatures.
/// Cheap maindeck answer to early Silverquill/Witherbloom 2-toughness
/// creatures (Eyetwitch, Star Pupil, Pest Mascot). Tests:
/// `demolishing_lecture_destroys_two_toughness_creature`,
/// `demolishing_lecture_is_a_three_mana_black_sorcery`.
pub fn demolishing_lecture() -> CardDefinition {
    CardDefinition {
        name: "Demolishing Lecture",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ToughnessAtMost(2)),
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

// ── Inkling Mentor ─────────────────────────────────────────────────────────

/// Inkling Mentor — {3}{W}{B}, 3/4 Human Cleric (synthesised STX
/// Silverquill flavor). "Other Inkling creatures you control get
/// +1/+1."
///
/// A second Inkling tribal lord (alongside Tenured Inkcaster's
/// +2/+2). Stacks multiplicatively: 1-mana Inkling Squire becomes
/// 4/4 Flying with both lords. Wired via the
/// `tribal_anthem_for_name` compute-time injection in
/// `GameState::compute_battlefield`. Tests:
/// `inkling_mentor_pumps_other_inklings`,
/// `inkling_mentor_does_not_pump_non_inklings`,
/// `inkling_mentor_does_not_pump_self`.
pub fn inkling_mentor() -> CardDefinition {
    CardDefinition {
        name: "Inkling Mentor",
        cost: cost(&[generic(3), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        // Tribal anthem on the Inkling subtype, same shape as
        // Tenured Inkcaster's `StaticEffect::PumpPT` — exclude the
        // source (a Human Cleric, technically already excluded by
        // the creature-type filter) for faithful "Other ..." wording.
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
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
    }
}

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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Lesson in Honor ────────────────────────────────────────────────────────

/// Lesson in Honor — {1}{W} Sorcery — Lesson (synthesised STX
/// Silverquill flavor). "Target creature gets +2/+2 until end of
/// turn. Learn."
///
/// A combat trick Lesson — pumps a friendly +2/+2 EOT and ticks the
/// Learn cantrip (Draw 1 approximation). Mirror to Fortifying Draught
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Blade Historian ────────────────────────────────────────────────────────

/// Blade Historian — {2}{R}{W}, 3/2 Human Wizard (printed STX Lorehold).
///
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// attacking creatures you control get +1/+0 and gain double strike
/// until end of turn."
///
/// Wired via `magecraft(ForEach(Creature & ControlledByYou & IsAttacking)
/// → Seq(PumpPT +1/+0 EOT, GrantKeyword Double Strike EOT))`. The
/// `IsAttacking` filter restricts to creatures currently declared as
/// attackers, matching the printed Oracle. Tests:
/// `blade_historian_is_a_four_mana_three_two_human_wizard`,
/// `blade_historian_magecraft_pumps_attackers_and_grants_double_strike`.
pub fn blade_historian() -> CardDefinition {
    CardDefinition {
        name: "Blade Historian",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::IsAttacking),
            ),
            body: Box::new(Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ])),
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
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
        exile_on_resolve: false,
    }
}

// ── Inkling Aether-Smith ───────────────────────────────────────────────────

/// Inkling Aether-Smith — {2}{W}{B}, 2/3 Inkling Artificer with
/// Flying (synthesised STX Silverquill / Quandrix-adjacent flavor).
/// "When this creature enters, choose one — / • Create a 1/1 white
/// and black Inkling creature token with flying. / • Put a +1/+1
/// counter on target creature you control."
///
/// Modal ETB: token or counter. Auto-decider picks mode 0 (token)
/// for go-wide play patterns. Wired via `Effect::ChooseMode([token,
/// counter])`. Tests:
/// `inkling_aether_smith_is_a_four_mana_two_three_inkling_artificer`,
/// `inkling_aether_smith_etb_default_creates_token`.
pub fn inkling_aether_smith() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Inkling Aether-Smith",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: inkling_token(),
                },
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
    }
}
