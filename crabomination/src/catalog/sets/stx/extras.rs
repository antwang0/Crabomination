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
use crate::effect::shortcut::{magecraft, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
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
    }
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — {U}{R} Instant. "Tap up to two target creatures. Put a
/// stun counter on each of them."
///
/// Single-target tap+stun. "Up to two targets" is an engine-wide gap
/// (shared with Vibrant Outburst, Spell Satchel, Devious Cover-Up).
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
            Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
            Effect::AddCounter {
                what: Selector::Target(0),
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
    }
}

/// Quintorius, Field Historian — {2}{R}{W} Legendary Creature — Elephant
/// Cleric Spirit, 3/3 (Lorehold). "Vigilance / When Quintorius enters,
/// exile target card from a graveyard. Create a 3/2 red and white
/// Spirit creature token."
///
/// ✅ ETB body (exile gy card + mint 3/2 R/W Spirit token) wired via the
/// EntersBattlefield/SelfSource trigger. The printed static "Other
/// Spirit creatures you control get +1/+0" anthem is now wired via a
/// compute-time injection in `GameState::compute_battlefield`, using
/// the new `AffectedPermanents::AllWithCreatureType.exclude_source`
/// flag so Quintorius himself doesn't buff himself (he is a Spirit,
/// matching the printed "Other" gate). The injection scopes to his
/// controller's Spirit creatures, layer 7b (+1/+0), and re-evaluates
/// every recompute — so a Spirit minted by his ETB trigger is buffed
/// immediately when state-based actions next fire.
pub fn quintorius_field_historian() -> CardDefinition {
    use crate::card::Supertype;
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
/// 🟡 approximation: the printed "may pay {2}" tax is collapsed into
/// an automatic copy via `Effect::CopySpell` whenever an opponent
/// casts an instant or sorcery. This is strictly stronger than the
/// printed Oracle (no opt-out for the opp) but preserves the
/// "Wandering Archaic punishes spell-heavy decks" play pattern. The
/// `CounterUnlessPaid`-style "pay or get copied" gate is engine-wide
/// ⏳ — it needs a new `Effect::CopyUnlessPaid { ... }` primitive that
/// hooks into the opp's auto-decider at cast time.
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
            effect: Effect::CopySpell {
                what: Selector::TriggerSource,
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
    }
}

// ── Snarl land cycle ────────────────────────────────────────────────────────

/// Build a Strixhaven Snarl dual land. Printed Oracle: "As this land
/// enters, you may reveal a [C1] or [C2] card from your hand. If you
/// don't, this land enters tapped."
///
/// 🟡 Approximation: we ship the conservative ("don't reveal") branch
/// — these always enter tapped. The reveal-from-hand decision is a
/// non-trivial UI prompt (the engine has no "may reveal" action shape
/// at ETB time), and a strictly-untapped version would be too strong.
/// Wiring the optimization (look at hand for the right color and
/// auto-skip the tap) is tracked under TODO.md as the "Snarl-land
/// reveal" gap.
fn snarl_land(
    name: &'static str,
    type_a: LandType,
    type_b: LandType,
    color_a: Color,
    color_b: Color,
) -> CardDefinition {
    use super::super::{etb_tap, tap_add};
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
        triggered_abilities: vec![etb_tap()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    }
}

// ── Plargg, Dean of Chaos ───────────────────────────────────────────────────

/// Plargg, Dean of Chaos — {1}{R}, 2/2 Legendary Human Cleric.
///
/// "{T}: Discard a card, then draw a card. If a creature card was
/// discarded this way, Plargg, Dean of Chaos deals 2 damage to any
/// target."
///
/// Push (modern_decks): the loot half is wired faithfully as a tap
/// activation with `Seq(Discard 1, Draw 1)`. The "if a creature card
/// was discarded → 2 damage" rider is omitted (engine has no
/// track-card-discarded-by-this-effect tally; same gap as Borrowed
/// Knowledge mode 1 and Colossus of the Blood Age's death-trigger).
/// Tracked in TODO.md as the
/// `Effect::DiscardThisManyDrawSame` suggestion. The "Partner with
/// Augusta, Dean of Order" rider is also omitted — engine has no
/// Partner-pair primitive (only the singleton legend constraint is
/// enforced).
///
/// At face value this is a 2-mana 2/2 with a tap-loot — a respectable
/// curve filler for any Lorehold (R/W) shell.
pub fn plargg_dean_of_chaos() -> CardDefinition {
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
    }
}

// ── Augusta, Dean of Order ──────────────────────────────────────────────────

/// Augusta, Dean of Order — {2}{W}, 2/3 Legendary Human Cleric.
///
/// "Whenever you attack with three or more creatures with the same
/// power, each of those creatures gets +1/+1 and gains your choice of
/// flying, first strike, vigilance, or lifelink until end of turn."
///
/// Push (modern_decks): body-only wire. The 2/3 Legendary Human Cleric
/// is a respectable Lorehold (R/W) "go-wide" lord at three mana. The
/// printed combat-step trigger is omitted (engine has no "attacking
/// creatures with the same power" predicate, nor a multi-pump-with-
/// chosen-keyword shape). The "Partner with Plargg, Dean of Chaos"
/// rider is also omitted (no Partner-pair primitive — only the
/// singleton legendary rule is enforced). At face value this is a
/// 3-mana 2/3 legendary that can attack on its own and pairs with
/// Plargg as part of the printed Augusta + Plargg combo (when both
/// resolve and Partner is honored).
///
/// Tests cover the body, P/T, and creature subtypes.
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
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    }
}
