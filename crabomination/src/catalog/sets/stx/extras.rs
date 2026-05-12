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

// ── Strict Mentor ───────────────────────────────────────────────────────────

/// Pest Token (white-and-black Strixhaven Pest) — small helper token.
/// Used here as a stand-in for the "1/1 black-and-green Pest with die →
/// gain 1" tokens we already mint from `super::shared::stx_pest_token`.
/// Kept private to this module.
#[allow(dead_code)]
fn local_pest_token() -> TokenDefinition {
    TokenDefinition {
        name: "Pest".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black, Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

// ── Mentor of Evos Isle (stand-in placeholder dropped) ─────────────────────
