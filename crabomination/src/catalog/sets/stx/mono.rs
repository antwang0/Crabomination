//! Strixhaven mono-color cards (and a few cross-school staples without a
//! pure college slot). These wrap simpler mechanics — flash creatures,
//! library manipulation, X-cost tutors — so they compose against the
//! engine without leaning on Magecraft / Lesson / cast-from-graveyard.
//!
//! See `STRIXHAVEN2.md` ("Strixhaven base set (STX)" section) for the
//! per-card status table.

use super::no_abilities;
use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Selector, SelectionRequirement, SpellSubtype, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, u, w, x};

// ── Pop Quiz ────────────────────────────────────────────────────────────────

/// Pop Quiz — {1}{W} Sorcery — Lesson. "Draw two cards, then put a card
/// from your hand on top of your library."
///
/// Two-step: `Draw 2` then `PutOnLibraryFromHand 1`. The Lesson sub-type is
/// recorded so future Lesson-aware effects (Mascot Exhibition's "search
/// your sideboard") can filter on it; today Lesson cards resolve from hand
/// like any other sorcery.
pub fn pop_quiz() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz",
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
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::PutOnLibraryFromHand {
                who: PlayerRef::You,
                count: Value::Const(1),
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

// ── Mascot Exhibition ───────────────────────────────────────────────────────

/// Mascot Exhibition — {5}{W}{W} Sorcery. "Create a 3/3 white Elephant
/// creature token, a 2/2 white Cat creature token with lifelink, and a
/// 1/1 white Bird creature token with flying."
pub fn mascot_exhibition() -> CardDefinition {
    let elephant = TokenDefinition {
        name: "Elephant".to_string(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    let cat = TokenDefinition {
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
    };
    let bird = TokenDefinition {
        name: "Bird".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Mascot Exhibition",
        cost: cost(&[generic(5), w(), w()]),
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
                definition: elephant,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: cat,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: bird,
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

// ── Plumb the Forbidden ─────────────────────────────────────────────────────

/// Plumb the Forbidden — {X}{B}{B} Instant. "Sacrifice X creatures. Each
/// player who controlled a sacrificed creature draws X cards and loses X
/// life."
///
/// Approximation: caster sacrifices X of their own creatures, draws X
/// cards, and loses X life. Multi-controller mode (when a creature was
/// stolen from another player) collapses to "you" — keeps the card
/// playable as the standard self-sac engine. The X is read off
/// `Value::XFromCost` via the cast-time `x_value`.
pub fn plumb_the_forbidden() -> CardDefinition {
    CardDefinition {
        name: "Plumb the Forbidden",
        cost: cost(&[x(), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::XFromCost,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::LoseLife {
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
        back_face: None,
        opening_hand: None,
    }
}

// ── Owlin Shieldmage ────────────────────────────────────────────────────────

/// Owlin Shieldmage — {3}{W} Creature — Bird Wizard. Flash, flying, 2/3.
/// "When this enters, prevent all combat damage that would be dealt this
/// turn." We ship the flash flyer body; damage prevention requires a
/// replacement primitive the engine doesn't yet have.
pub fn owlin_shieldmage() -> CardDefinition {
    CardDefinition {
        name: "Owlin Shieldmage",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
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
    }
}

// ── Frost Trickster ─────────────────────────────────────────────────────────

/// Frost Trickster — {1}{U} Creature — Spirit Wizard. Flash, flying, 2/2.
/// "When this creature enters, tap target creature an opponent controls.
/// That creature doesn't untap during its controller's next untap step."
///
/// Modeled as "When this enters, tap target creature an opponent controls
/// and put a stun counter on it" — close enough for the demo (a stun
/// counter prevents the next untap, matching the printed line).
pub fn frost_trickster() -> CardDefinition {
    CardDefinition {
        name: "Frost Trickster",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
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

// ── Body of Research ────────────────────────────────────────────────────────

/// Body of Research — {4}{G}{U} Sorcery. "Create a 0/0 green and blue
/// Fractal creature token. Put a +1/+1 counter on it for each card in your
/// library."
///
/// Now wired (push XVI) via the new `Value::LibrarySizeOf` primitive —
/// the Fractal enters with one +1/+1 counter per library card, matching
/// the printed Oracle exactly. Earlier revisions approximated this as
/// `GraveyardSizeOf` because `LibrarySize` wasn't a primitive.
pub fn body_of_research() -> CardDefinition {
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
        name: "Body of Research",
        cost: cost(&[generic(4), g(), u()]),
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
                amount: Value::LibrarySizeOf(PlayerRef::You),
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

// ── Show of Confidence ──────────────────────────────────────────────────────

/// Show of Confidence — {1}{W} Instant. "Put N +1/+1 counters on target
/// creature, where N is the number of times you've cast Show of Confidence
/// this game, plus one." We ship the counter-by-storm-count approximation:
/// N = `StormCount + 1` (i.e. one counter for the spell itself plus one
/// for every other spell cast this turn). Close to the printed card's
/// late-game scaling without per-cast-name tracking.
pub fn show_of_confidence() -> CardDefinition {
    CardDefinition {
        name: "Show of Confidence",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Sum(vec![Value::StormCount, Value::Const(1)]),
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

// ── Bury in Books ───────────────────────────────────────────────────────────

/// Bury in Books — {3}{U} Sorcery. "Put target creature on top of its
/// owner's library." A clean library-position bounce — same shape as
/// Hinder/Spell Crumple but for permanents.
pub fn bury_in_books() -> CardDefinition {
    CardDefinition {
        name: "Bury in Books",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
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

// ── Test of Talents ─────────────────────────────────────────────────────────

/// Test of Talents — {1}{U}{U} Instant. "Counter target instant or sorcery
/// spell. Search its controller's graveyard, hand, and library for any
/// number of cards with the same name as that spell, exile them, then
/// that player shuffles."
///
/// 🟡 We ship just the counter-target-IS-spell half. The follow-up
/// search-and-exile by name needs a name primitive (the engine has no
/// `SelectionRequirement::HasName` yet). Tracked in TODO.md.
pub fn test_of_talents() -> CardDefinition {
    CardDefinition {
        name: "Test of Talents",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterSpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
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

// ── Multiple Choice ─────────────────────────────────────────────────────────

/// Multiple Choice — {1}{U}{U} Sorcery. "Choose one or more — • Scry 2.
/// • Create a 1/1 blue Pest creature token. (We use a Bird with flying
/// since the printed card is a 'Pest'? No — Multiple Choice creates a
/// 1/1 blue Pest. We use a generic Pest token.) • Target creature gets
/// +1/+0 and gains hexproof until end of turn. • If you chose all of
/// the above, ..."
///
/// 🟡 Single-mode `ChooseMode` instead of Magic's "choose one or more" —
/// we surface only the first three modes (mode 0/1/2). Mode 3 (all four
/// at once) needs a multi-mode primitive.
// ── Environmental Sciences ─────────────────────────────────────────────────

/// Environmental Sciences — {2} Sorcery — Lesson. "Search your library
/// for a basic land card, reveal it, put it into your hand, then shuffle.
/// You gain 2 life."
pub fn environmental_sciences() -> CardDefinition {
    CardDefinition {
        name: "Environmental Sciences",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
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
    }
}

// ── Introduction to Annihilation ───────────────────────────────────────────

/// Introduction to Annihilation — {5} Sorcery — Lesson. "Exile target
/// nonland permanent. Its controller draws a card."
pub fn introduction_to_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Annihilation",
        cost: cost(&[generic(5)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
            Effect::Draw {
                who: Selector::Player(
                    PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                ),
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

// ── Introduction to Prophecy ───────────────────────────────────────────────

/// Introduction to Prophecy — {3} Sorcery — Lesson. "Scry 2, then draw
/// a card."
pub fn introduction_to_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Prophecy",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
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
    }
}

// ── Expanded Anatomy ───────────────────────────────────────────────────────

/// Expanded Anatomy — {3} Sorcery — Lesson. "Put two +1/+1 counters on
/// target creature. It gains vigilance until end of turn."
pub fn expanded_anatomy() -> CardDefinition {
    CardDefinition {
        name: "Expanded Anatomy",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
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
    }
}

// ── Cram Session ───────────────────────────────────────────────────────────

/// Cram Session — {1}{W/B} Sorcery. "You gain 4 life. Learn."
/// Hybrid approximated as {1}{W}. Learn → Draw 1.
pub fn cram_session() -> CardDefinition {
    CardDefinition {
        name: "Cram Session",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
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

// ── Professor of Symbology ─────────────────────────────────────────────────

/// Professor of Symbology — {1}{W}, 2/1 Human Cleric. "When this enters,
/// you may discard a card. If you do, draw a card." (Learn approximation.)
pub fn professor_of_symbology() -> CardDefinition {
    CardDefinition {
        name: "Professor of Symbology",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::MayDo {
                description: "Learn (discard a card, then draw a card)?".into(),
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
                ])),
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

// ── Guiding Voice ──────────────────────────────────────────────────────────

/// Guiding Voice — {W} Sorcery. "Put a +1/+1 counter on target creature.
/// Learn." (Learn → Draw 1)
pub fn guiding_voice() -> CardDefinition {
    CardDefinition {
        name: "Guiding Voice",
        cost: cost(&[w()]),
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
    }
}

// ── Divide by Zero ─────────────────────────────────────────────────────────

/// Divide by Zero — {2}{U} Instant. "Return target spell or permanent
/// with MV ≤ 4 to its owner's hand. Learn." (Learn → Draw 1)
///
/// 🟡 Collapsed to bounce nonland permanent (spell-on-stack bounce
/// collapsed).
pub fn divide_by_zero() -> CardDefinition {
    CardDefinition {
        name: "Divide by Zero",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ManaValueAtMost(4)),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
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

// ── Flunk ──────────────────────────────────────────────────────────────────

/// Flunk — {1}{B} Instant. "Target creature gets -X/-X until end of turn,
/// where X is 7 minus the number of cards in its controller's hand."
///
/// 🟡 Approximated as flat -4/-4 (typical hand of 3 → -4/-4).
pub fn flunk() -> CardDefinition {
    CardDefinition {
        name: "Flunk",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
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

// ── Curate ─────────────────────────────────────────────────────────────────

/// Curate — {1}{U} Instant. "Surveil 2. Draw a card."
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
            Effect::Surveil {
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
    }
}

// ── Igneous Inspiration ────────────────────────────────────────────────────

/// Igneous Inspiration — {2}{R} Sorcery. "Igneous Inspiration deals 3
/// damage to any target. Learn." (Learn → Draw 1)
pub fn igneous_inspiration() -> CardDefinition {
    CardDefinition {
        name: "Igneous Inspiration",
        cost: cost(&[generic(2), crate::mana::r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
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

// ── Charge Through ─────────────────────────────────────────────────────────

/// Charge Through — {G} Instant. "Target creature gains trample until
/// end of turn. Draw a card."
pub fn charge_through() -> CardDefinition {
    CardDefinition {
        name: "Charge Through",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
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

pub fn multiple_choice() -> CardDefinition {
    let pest = TokenDefinition {
        name: "Pest".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Multiple Choice",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: Scry 2.
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            // Mode 1: 1/1 blue Pest token.
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pest,
            },
            // Mode 2: target creature +1/+0 and hexproof EOT.
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(SelectionRequirement::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Hexproof,
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
    }
}
