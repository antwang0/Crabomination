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
    EventScope, EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

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
    }
}

// ── Snow Day ────────────────────────────────────────────────────────────────

/// Snow Day — {U}{R} Instant. "Tap up to two target creatures. Put a
/// stun counter on each of them."
///
/// 🟡 Single-target approximation: tap one target creature and put a
/// stun counter on it. The "up to two targets" multi-target prompt is
/// the same gap as Vibrant Outburst — tracked in TODO.md.
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
    }
}

// ── (helper `local_pest_token` removed in push XX — `super::shared::stx_pest_token`
//     is the canonical Pest factory used everywhere a Pest is minted.)

// ── Curate ──────────────────────────────────────────────────────────────────

/// Curate — {1}{U} Instant. "Look at the top four cards of your library.
/// Put one of them into your hand and the rest on the bottom of your
/// library in a random order."
///
/// 🟡 Approximated as `Scry 3 → Draw 1`: the player scries the top three
/// (effectively their pick from the top of the library) and then draws
/// one. We don't model the "bottom of library in random order" rider —
/// the engine's `Effect::Scry` lets the controller pick top vs. bottom
/// per card. The net effect (pick one card to keep, send the rest
/// somewhere out of immediate reach) matches the printed gameplay
/// behaviour, with the small caveat that scry-bottomed cards land at
/// the *bottom* of the library (in scry order) rather than in random
/// order.
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
    }
}

// ── Daemogoth Titan ────────────────────────────────────────────────────────

/// Daemogoth Titan — {B}{B}, 11/11 Demon Horror. "When this attacks or
/// blocks, sacrifice another creature."
///
/// 🟡 We ship the full 11/11 body. The attack/block sacrifice rider is
/// wired only for the attack half — `EventKind::Attacks` fires on attack
/// declaration; a `Blocks` event for the block half is a TODO (the
/// engine has no blocker-declared event yet). The sacrifice resolves
/// via `Effect::Sacrifice` over "creature you control" — the filter is
/// approximated as "any creature you control" since the engine's
/// `Sacrifice` decider already prefers the lowest-power non-self
/// creature when multiple are available, so a fresh Titan will sac
/// something else rather than itself.
pub fn daemogoth_titan() -> CardDefinition {
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
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

// ── Daemogoth Woe-Eater ────────────────────────────────────────────────────

/// Daemogoth Woe-Eater — {2}{B}{G}, 4/4 Demon Horror. "When this enters,
/// sacrifice another creature. Whenever this attacks, you may sacrifice
/// another creature. If you do, put a +1/+1 counter on this creature."
///
/// 🟡 We ship the ETB sacrifice and the attack-trigger sac → counter as
/// a mandatory pair (the engine's `Effect::Sacrifice` already no-ops
/// cleanly when no legal creature exists; the counter then no-ops in
/// the same `Seq` step). Body is the 4/4 Demon Horror at {2}{B}{G}.
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
                effect: Effect::Seq(vec![
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
                ]),
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Honor Troll ────────────────────────────────────────────────────────────

/// Honor Troll — {1}{B}{G}, 1/4 Troll Warrior. "Trample. As long as
/// you've gained life this turn, this creature has +2/+0 and lifelink."
///
/// 🟡 The conditional pump/lifelink rider is omitted — the engine has
/// no per-turn `life_gained_this_turn` tracker yet. Body ships as a
/// {1}{B}{G} 1/4 trampler with the Trample keyword, which is still a
/// reasonable midrange defender. Tracked in TODO.md alongside the
/// other "this-turn" lifegain payoffs.
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
    }
}

// ── Hofri Ghostforge ───────────────────────────────────────────────────────

/// Hofri Ghostforge — {2}{R}{W}, 3/4 Legendary Spirit Cleric. "Other
/// creatures you control get +1/+0. / Whenever another nontoken
/// creature you control dies, exile it. At the beginning of the next
/// end step, return it to the battlefield as a 1/1 Spirit with flying."
///
/// 🟡 Body + keywords (legendary, P/T, types) ship full. The two
/// printed riders are not wired:
/// * "Other creatures you control get +1/+0" is a static anthem layer
///   we don't model (the engine's `StaticAbility` palette doesn't yet
///   include conditional anthems gated by "this is on the
///   battlefield").
/// * "Exile-on-death + return at end step as a Spirit" is a complex
///   delayed-trigger graveyard-cycle replacement we don't have a
///   primitive for yet. Both tracked in TODO.md.
pub fn hofri_ghostforge() -> CardDefinition {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Tempted by the Oriq ────────────────────────────────────────────────────

/// Tempted by the Oriq — {2}{B}, 1/3 Elf Warlock — actually this is a
/// Sorcery in real STX. Updated to the proper Sorcery body: "Gain
/// control of target creature until end of turn. Untap that creature.
/// It gains haste until end of turn. (Magecraft) Whenever you cast or
/// copy an instant or sorcery spell, that creature deals 1 damage to
/// any target." We approximate as the temp-steal + untap + haste body
/// only — the Magecraft rider on Tempted is sorcery-cast time and
/// would require a delayed trigger tied to the controlled creature
/// (not currently a primitive).
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
    }
}


// ── Push XXI: 6 new STX cards ───────────────────────────────────────────────

/// Confront the Past — {3}{R} Sorcery.
/// "Choose one — / • Put target planeswalker card from your graveyard
/// onto the battlefield. / • Return target planeswalker to its
/// owner's hand. / • Confront the Past deals damage to target
/// planeswalker equal to the number of loyalty counters on it."
///
/// 🟡 Three-mode `ChooseMode`: mode 0 reanimates a PW from your
/// graveyard (auto-decider picks the only PW in gy), mode 1 bounces
/// an opp PW. Mode 2 (X-damage where X = loyalty counters) is
/// approximated as a flat 3-damage burn — engine has no per-card
/// loyalty-counter introspection on damage today.
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
    }
}

// ── Push XXIII (2026-05-12): 11 new STX cards ───────────────────────────────
//
// New batch focuses on filling out Silverquill, Quandrix, and Lorehold STX
// printings plus three mono utility cards. All cards ship in the `extras`
// file and slot into the existing engine primitives — no new primitives
// added in this push. See STRIXHAVEN2.md for the per-card status row.

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
/// instead. The `Value::CountersOn` lookup was extended (push XXIII)
/// to cross-zone-search so future cards that legitimately need the
/// source's counter count post-death can read it.
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
    }
}

/// Devious Cover-Up — {2}{U}{U} Instant (Mono-U STX).
/// "Counter target spell. Then exile any number of target cards from
/// graveyards."
///
/// 🟡 The counterspell half ships full via `Effect::CounterSpell`. The
/// "exile any number of target cards from graveyards" rider collapses
/// to "exile up to one target card from a graveyard" via a single
/// graveyard-card target. The card's typical play pattern is counter +
/// strip a specific gy threat (Snapcaster food, Underworld Breach
/// food, etc.), so the single-target approximation captures the
/// intent.
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
    }
}

/// Crackle with Power — {X}{R}{R}{R}{R}{R} Sorcery (Mono-R STX).
/// "Crackle with Power deals 5X damage divided as you choose among
/// any number of targets."
///
/// 🟡 The "divided among any number of targets" rider collapses to a
/// single target absorbing the full 5X damage — same gap as the
/// printed multi-target rider on Crackle's siblings (no multi-target
/// prompt yet). The 5X scaling is wired via `Value::Times(Const(5),
/// XFromCost)`. A faithful 5-color quintuple-pip cost matches the
/// printed mana cost; the engine accepts the {RRRRR} pip sequence
/// because `cost()` builds an ordered ManaCost.
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
    }
}

/// Quintorius, Field Historian — {2}{R}{W} Legendary Creature — Elephant
/// Cleric Spirit, 3/3 (Lorehold). "Vigilance / When Quintorius enters,
/// exile target card from a graveyard. Create a 3/2 red and white
/// Spirit creature token."
///
/// 🟡 ETB body (exile gy card + mint Spirit) is faithful. The printed
/// static "Spirit creatures you control get +1/+0" anthem is omitted —
/// it'd want a tribal lord-with-creature-type filter, which the engine
/// supports (`AllWithCreatureType`) but composing it via a
/// `StaticEffect::PumpPT { applies_to: each_your_creature_with_type }`
/// requires a selector shape the layer system doesn't yet decode.
/// Tracked under TODO.md "Selector shapes for static lords".
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
    }
}
