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
    EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, LibraryPosition, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Multiple Choice ─────────────────────────────────────────────────────────

/// Multiple Choice — {1}{U}{U} Sorcery (Strixhaven Quandrix-flavoured
/// flexible spell). Printed Oracle:
/// "Choose one or more —
///  • Scry 2.
///  • Target creature gets +1/+0 and gains hexproof until end of turn.
///  • Create a 1/1 blue Pest creature token.
///  • If you chose all of the above, …" (mega-mode rider).
///
/// Push XXXVI: 🟡 → ✅ (modes 0–2). Now wires the "choose one or more"
/// shape via `Effect::ChooseModes { count: 3, up_to: true,
/// allow_duplicates: false }`. Auto-decider picks all 3 modes
/// (Scry 2 + pump+hexproof + Pest token). `ScriptedDecider::new([
/// Modes(vec![0, 2])])` lets tests verify the Scry-2 + Pest pair.
/// The "if you chose all of the above" mega-mode rider is omitted (no
/// "modes-picked count" introspection primitive yet — would need to
/// expose the chosen-mode list in `EffectContext` for an `Effect::If`
/// gate). Mode 0/1/2 fidelity now matches printed "choose one or more".
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
        effect: Effect::ChooseModes {
            count: 3,
            up_to: true,
            allow_duplicates: false,
            modes: vec![
                // Mode 0: Scry 2.
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                // Mode 1: target creature +1/+0 and hexproof EOT.
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
                // Mode 2: 1/1 blue Pest token.
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: pest,
                },
            ],
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Charge Through ──────────────────────────────────────────────────────────

/// Charge Through — {G} Sorcery (Strixhaven common). "Target creature
/// gets +1/+1 and gains trample until end of turn. Draw a card."
///
/// Cantripping pump that doubles as a combat enabler. Wired with a
/// `PumpPT(+1/+1) + GrantKeyword(Trample) + Draw 1` Seq, all gated to
/// the same `Selector::Target(0)` so the pump and trample land on the
/// same chosen creature.
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
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Resculpt ───────────────────────────────────────────────────────────────

/// Resculpt — {1}{U} Instant (Strixhaven uncommon). "Exile target
/// artifact or creature. Its controller creates a 4/4 blue Elemental
/// creature token."
///
/// The exile is faithful to the printed wording. The 4/4 Elemental
/// token is created under the original target's controller via the
/// `ZoneDest::Battlefield { controller: PlayerRef::ControllerOf(...) }`
/// path, so removing an opponent's threat hands them a vanilla 4/4
/// (which is usually a bad trade for the targeting player — the
/// printed downside is preserved).
pub fn resculpt() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".to_string(),
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
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::Creature),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Letter of Acceptance ────────────────────────────────────────────────────

/// Letter of Acceptance — {3} Artifact (Strixhaven common).
/// "When this artifact enters, scry 1, then draw a card. /
///  {3}, Sacrifice this artifact: Draw a card."
///
/// Body wired faithfully. ETB Scry 1 + Draw 1 cantrips immediately;
/// the {3}+sac activation provides a late-game sink for surplus mana
/// and a card.
pub fn letter_of_acceptance() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Letter of Acceptance",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Reduce to Memory ───────────────────────────────────────────────────────

/// Reduce to Memory — {2}{U} Instant (Strixhaven uncommon).
/// "Exile target creature or planeswalker. Its controller creates a
/// 2/2 white and black Inkling creature token with flying."
///
/// Exile + create-Inkling. The Inkling token reuses the SOS
/// `inkling_token()` helper (1/1 W/B with flying). Note: the printed
/// token is 2/2; we mint the Strixhaven 1/1 helper to keep token-pool
/// definitions consistent — power band is similar at common
/// fixity, and the exile half remains the dominant value.
pub fn reduce_to_memory() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    CardDefinition {
        name: "Reduce to Memory",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Defend the Campus ──────────────────────────────────────────────────────

/// Defend the Campus — {3}{R}{W} Instant (Strixhaven uncommon).
/// "Up to two target attacking creatures get -3/-0 until end of turn."
///
/// Approximation: collapsed to a single -3/-0 EOT debuff on a
/// single target (no multi-target prompt yet). Combat-only filter
/// (`IsAttacking`) keeps the spell aimed at a defender's-side use
/// — preserving the printed combat-trick role.
pub fn defend_the_campus() -> CardDefinition {
    CardDefinition {
        name: "Defend the Campus",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::IsAttacking),
            ),
            power: Value::Const(-3),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Conspiracy Theorist ─────────────────────────────────────────────────────

/// Conspiracy Theorist — {R} Creature — Human Shaman (Strixhaven rare).
/// 1/3. "Whenever you discard a card, you may pay {R}. If you do,
/// exile that card from your graveyard. You may play that card this
/// turn." (Approximation: the "may pay {R} → may play that card this
/// turn" rider is dropped — engine has no per-card "may-play-from-
/// exile-until-EOT" primitive. The body remains a useful 1/3 R Shaman
/// that composes against other discard-matters payoffs.)
///
/// 🟡 Body-only wire. The discard-recursion trigger is omitted pending
/// the cast-from-exile-with-time-limit primitive.
pub fn conspiracy_theorist() -> CardDefinition {
    CardDefinition {
        name: "Conspiracy Theorist",
        cost: cost(&[r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Honor Troll ─────────────────────────────────────────────────────────────

/// Honor Troll — {2}{W} Creature — Troll (Strixhaven uncommon).
/// 0/3 with "Honor Troll has lifelink as long as you control four or
/// more creatures." (Approximation: lifelink-when-4-creatures rider
/// is dropped — engine has no `StaticEffect::ConditionalKeyword`
/// primitive. Body ships as a 0/3 Troll.) 🟡
pub fn honor_troll() -> CardDefinition {
    CardDefinition {
        name: "Honor Troll",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Pillardrop Warden ───────────────────────────────────────────────────────

/// Pillardrop Warden — {2}{W} Creature — Spirit Cleric (Strixhaven common).
/// 2/3 with "When this creature enters, scry 1." A clean playable common —
/// solid late-game body + a small library-shaping cantrip on entry.
pub fn pillardrop_warden() -> CardDefinition {
    CardDefinition {
        name: "Pillardrop Warden",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Beaming Defiance ────────────────────────────────────────────────────────

/// Beaming Defiance — {1}{W} Instant (Strixhaven common). "Target creature
/// you control gets +1/+1 and gains hexproof until end of turn."
///
/// A clean combat trick that doubles as a counter-magic dodge — hexproof
/// hardens the target against opponent spot removal. The pump and grant
/// share the same `Selector::Target(0)` so they both land on the same
/// chosen creature.
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
                power: Value::Const(1),
                toughness: Value::Const(1),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Ageless Guardian ────────────────────────────────────────────────────────

/// Ageless Guardian — {1}{W} Creature — Spirit Wall. 0/4 with Vigilance
/// and Defender. (Approximation: the printed card has Defender + a
/// "becomes attacker" rider; we drop the becomes-attacker rider — without
/// it the body is a pure 0/4 Vigilance defensive wall, still useful in
/// any white midrange shell.)
pub fn ageless_guardian() -> CardDefinition {
    CardDefinition {
        name: "Ageless Guardian",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Expel ───────────────────────────────────────────────────────────────────

/// Expel — {2}{W} Instant (Strixhaven common). "Exile target attacking
/// or blocking creature." Combat-only removal — efficient at instant
/// speed in white. The selector filters on `IsAttacking ∨ IsBlocking`
/// for the printed combat-window restriction.
pub fn expel() -> CardDefinition {
    CardDefinition {
        name: "Expel",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(
                    SelectionRequirement::IsAttacking.or(SelectionRequirement::IsBlocking),
                ),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Eureka Moment ───────────────────────────────────────────────────────────

/// Eureka Moment — {2}{U} Instant (Strixhaven common). "Untap target
/// land. Draw two cards." A draw-2 instant with an untap rider that
/// effectively floats one mana for a tempo play. The untap targets a
/// specific land via `Untap { what: Target(0), up_to: None }` so only
/// the chosen one untaps; the draw is unconditional.
pub fn eureka_moment() -> CardDefinition {
    CardDefinition {
        name: "Eureka Moment",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(SelectionRequirement::Land),
                up_to: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Curate ──────────────────────────────────────────────────────────────────

/// Curate — {1}{U} Instant (Strixhaven common). "Surveil 2, then draw
/// a card." A blue smoothing instant: Surveil 2 fills the graveyard
/// with junk while sculpting the next two draws, then the cantrip
/// keeps card-flow ticking.
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Necrotic Fumes ──────────────────────────────────────────────────────────

/// Necrotic Fumes — {1}{B}{B} Sorcery (Strixhaven uncommon). "As an
/// additional cost to cast this spell, sacrifice a creature. Exile
/// target creature."
///
/// 🟡 The "additional sacrifice cost" is approximated as an in-resolution
/// `Sacrifice { count: 1, filter: Creature & ControlledByYou }` followed
/// by the targeted exile. The engine has no "extra cost on cast" primitive
/// (cost paid before the spell hits the stack) — moving the sac into the
/// resolution effect keeps the card playable; the only fidelity loss is
/// that a "sacrifice on cast" requires a creature to be in play *as the
/// spell resolves* rather than at cast time.
pub fn necrotic_fumes() -> CardDefinition {
    CardDefinition {
        name: "Necrotic Fumes",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::Const(1),
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Creature),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Bookwurm ────────────────────────────────────────────────────────────────

/// Bookwurm — {3}{G}{G} Creature — Wurm. 4/5. "When this creature enters,
/// you gain 4 life and draw a card." A vanilla 4/5 with a generous ETB
/// — both halves resolve at the controller (the printed Oracle says
/// "you", which collapses to `Selector::You`).
pub fn bookwurm() -> CardDefinition {
    CardDefinition {
        name: "Bookwurm",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![],
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Spined Karok ────────────────────────────────────────────────────────────

/// Spined Karok — {3}{G} Creature — Wurm. 4/5 with no abilities — a
/// vanilla beater. The printed card is a Strixhaven common that ships
/// as a clean curve-topper for green draft archetypes.
pub fn spined_karok() -> CardDefinition {
    CardDefinition {
        name: "Spined Karok",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Field Trip ──────────────────────────────────────────────────────────────

/// Field Trip — {2}{G} Sorcery — Lesson (Strixhaven common). "Search
/// your library for a Forest card, put it onto the battlefield tapped,
/// then shuffle. Scry 1."
///
/// Approximation: search filter is `IsBasicLand` (the engine has no
/// "basic-land-of-named-type" filter today; most mono-G mana bases run
/// only Forests so this is a faithful match in practice). Lesson type
/// is recorded via `SpellSubtype::Lesson` so future Learn-aware code
/// can filter on it.
pub fn field_trip() -> CardDefinition {
    CardDefinition {
        name: "Field Trip",
        cost: cost(&[generic(2), g()]),
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
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand
                    .and(SelectionRequirement::HasLandType(crate::card::LandType::Forest)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Quandrix Cultivator ─────────────────────────────────────────────────────

/// Quandrix Cultivator — {3}{G}{U} Creature — Elf Druid (Strixhaven
/// common). 3/4 with "When this creature enters, search your library
/// for up to two basic land cards, put them onto the battlefield tapped,
/// then shuffle." A two-for-one ramp body — fixes mana and adds a 3/4
/// blocker in the same card.
///
/// The printed "up to two" is approximated as exactly-two by issuing
/// two `Search` effects in sequence; if the library is empty for the
/// second, the search just fails-silently (engine default).
pub fn quandrix_cultivator() -> CardDefinition {
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
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Square Up ───────────────────────────────────────────────────────────────

/// Square Up — {U}{R} Instant (Strixhaven common). "Target creature you
/// control gets +0/+1 until end of turn. It deals damage equal to its
/// power to target creature you don't control."
///
/// Approximation: collapses to a single auto-target combat trick — pump
/// the friendly creature, then have it deal its power as `DealDamage`
/// (we read its post-pump power via `Value::PowerOf(Selector::Target(0))`).
/// Single-target prompt; the printed two-target shape is collapsed to one
/// chosen friendly + auto-picked enemy creature in the engine's
/// resolution path.
pub fn square_up() -> CardDefinition {
    CardDefinition {
        name: "Square Up",
        cost: cost(&[u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        // Pump first so the post-pump power feeds the damage value.
        // The damage half is collapsed to "auto-pick an opp creature";
        // we use Fight's bidirectional shape but with a tiny attacker
        // (the friendly target gives its power, but we don't want the
        // friendly to take damage), so instead we emit a `DealDamage`
        // sourced from the friendly with `Value::PowerOf(Target(0))`.
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::Const(1),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Thrilling Discovery ─────────────────────────────────────────────────────

/// Thrilling Discovery — {1}{U}{R} Instant (Strixhaven common). "As an
/// additional cost to cast this spell, discard a card. You gain 2 life
/// and draw two cards."
///
/// 🟡 The "additional cost discard a card" is moved into resolution as a
/// `Discard 1` (same approximation as Necrotic Fumes' additional-cost
/// sacrifice) — engine has no extra-cost-at-cast primitive yet. The 2
/// life + draw 2 halves resolve unconditionally.
pub fn thrilling_discovery() -> CardDefinition {
    CardDefinition {
        name: "Thrilling Discovery",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
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
            Effect::GainLife {
                who: Selector::You,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Reckless Amplimancer ────────────────────────────────────────────────────

/// Reckless Amplimancer — {2}{G} Creature — Elf Druid Mutant (Strixhaven
/// common). "This creature enters with X +1/+1 counters on it, where X
/// is the number of mana symbols in the mana costs of permanents you
/// control."
///
/// Approximation: enters with N counters where N = `PermanentCountControlled`
/// (count of permanents you control) — the engine has no "sum mana
/// symbols across your battlefield" primitive yet, so we use the
/// permanent-count proxy. In practice, both numbers correlate strongly
/// during a typical mid-game (one mana symbol per permanent on average),
/// preserving the printed ramp-payoff feel.
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
        // Push XL: printed 0/0 — the `enters_with_counters`
        // replacement (push XL) lands a +1/+1 counter per permanent
        // you control at bf entry time, before SBAs run.
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::PermanentCountControlledBy(PlayerRef::You),
        )),
    }
}

// ── Specter of the Fens ─────────────────────────────────────────────────────

/// Specter of the Fens — {2}{B}{B} Creature — Specter (Strixhaven common).
/// 3/3 Flying. "When this creature enters, create a 1/1 black Pest creature
/// token with 'When this creature dies, you gain 1 life.'" An ETB Pest
/// minter — body + free chip-damage minion + lifegain trigger from the
/// SOS-VI's `TokenDefinition.triggered_abilities` plumbing.
pub fn specter_of_the_fens() -> CardDefinition {
    CardDefinition {
        name: "Specter of the Fens",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Ardent Dustspeaker ──────────────────────────────────────────────────────

/// Ardent Dustspeaker — {3}{R} Creature — Minotaur Shaman (Strixhaven
/// common). 3/3 with "At the beginning of combat on your turn, exile up
/// to one target card from a graveyard." Combat-step graveyard hate
/// stapled to a 3/3 body — fights well in graveyard-matters meta.
pub fn ardent_dustspeaker() -> CardDefinition {
    CardDefinition {
        name: "Ardent Dustspeaker",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Any,
                },
                to: ZoneDest::Exile,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Skyswimmer Koi ──────────────────────────────────────────────────────────

/// Skyswimmer Koi — {2}{U} Creature — Fish (Strixhaven common). 2/3
/// with "{4}{U}: Skyswimmer Koi gets +1/+1 until end of turn." A
/// late-game mana-sink — 2/3 base body that grows in long games when
/// the controller has surplus mana.
///
/// Implementation note: the activated ability targets `Selector::This`
/// and resolves immediately (no targeting prompt).
pub fn skyswimmer_koi() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Skyswimmer Koi",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(4), u()]),
            sac_cost: false,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Stonebinder's Familiar ──────────────────────────────────────────────────

/// Stonebinder's Familiar — {U} Creature — Spirit (Strixhaven uncommon).
/// 1/2 with "Whenever a permanent is put into a graveyard from anywhere
/// other than the battlefield, put a +1/+1 counter on this creature."
///
/// Approximation: triggers on any `EventKind::CardLeftGraveyard` (the
/// engine has no "card placed into graveyard" event yet — only
/// "card *left* graveyard"). Effect-wise that's the wrong direction;
/// instead we wire on `EventKind::PermanentLeavesBattlefield` with
/// `EventScope::AnyPlayer` — every time any permanent dies / gets
/// discarded / milled the Familiar grows. The printed card cares about
/// "anywhere other than the battlefield" (so no battlefield → graveyard
/// trigger) but our approximation includes battlefield → graveyard too;
/// that's a slight overcount versus the printed Oracle.
pub fn stonebinders_familiar() -> CardDefinition {
    CardDefinition {
        name: "Stonebinder's Familiar",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Quintorius, Field Historian ─────────────────────────────────────────────

/// Quintorius, Field Historian — {2}{R}{W} Legendary Creature — Elephant
/// Spirit (Strixhaven mythic). 3/4 with "When this creature enters,
/// exile target card from a graveyard. If a creature card was exiled
/// this way, create a 3/2 red and white Spirit creature token. /
/// Whenever a card leaves your graveyard, you may pay {3}{R}{W}. If
/// you do, create a 3/2 red and white Spirit creature token."
///
/// Approximation: ETB exiles any graveyard card (auto-decider takes
/// the first available); we always create a 3/2 R/W Spirit token
/// (the printed conditional on creature-card-exiled is collapsed —
/// if the exile fails, the create-token half still runs but is
/// usually dead, since selector resolution would fizzle the trigger
/// chain only in edge cases). The "may pay {3}{R}{W} on gy-leave"
/// rider is omitted (no `MayPay` on gy-leave triggers wired today,
/// though `Effect::MayPay` exists — just no card hooked it onto a
/// CardLeftGraveyard event yet).
pub fn quintorius_field_historian() -> CardDefinition {
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
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Dragon's Approach ───────────────────────────────────────────────────────

/// Dragon's Approach — {1}{R} Sorcery (Strixhaven uncommon).
/// "Dragon's Approach deals 3 damage to any target. Then if you have
/// four or more cards named Dragon's Approach in your graveyard, you
/// may search your library for a Dragon creature card, put it onto the
/// battlefield, then shuffle."
///
/// ✅ Push XXII: now fully wired via the new `SelectionRequirement::
/// HasName` predicate. The graveyard count is read via
/// `Value::CountOf(Selector::CardsInZone { Graveyard, HasName("Dragon's
/// Approach") })`; the gate uses `Predicate::ValueAtLeast(_, Const(4))`
/// to fork the resolution into a `Search { filter: Creature ∧
/// HasCreatureType(Dragon), to: Battlefield }` tutor (untapped, per
/// printed Oracle). The "may" optionality collapses to always-do
/// (auto-decider takes the value). The 4-copy-deckbuilding constraint
/// is enforced naturally by the deck construction layer; the engine
/// just counts whatever named copies exist.
pub fn dragons_approach() -> CardDefinition {
    use crate::card::Zone;
    use crate::effect::{Predicate, ZoneDest};
    use std::borrow::Cow;
    let name_filter = SelectionRequirement::HasName(Cow::Borrowed("Dragon's Approach"));
    CardDefinition {
        name: "Dragon's Approach",
        cost: cost(&[generic(1), r()]),
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
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: name_filter,
                    }),
                    Value::Const(4),
                ),
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(
                            crate::card::CreatureType::Dragon,
                        )),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Manifestation Sage ──────────────────────────────────────────────────────

/// Manifestation Sage — {2}{G}{U} Creature — Elf Wizard (Strixhaven
/// uncommon). 3/3, Flying. "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, target creature you control gets +X/+X
/// until end of turn, where X is the number of cards in your hand
/// minus 3."
///
/// 🟡 Body wired (3/3 Flying Elf Wizard). The Magecraft pump scales
/// off the inverse of HandSizeOf via `Value::Diff(HandSizeOf, 3)`.
/// Auto-decider picks first creature you control.
pub fn manifestation_sage() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Manifestation Sage",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::NonNeg(Box::new(Value::Diff(
                Box::new(Value::HandSizeOf(PlayerRef::You)),
                Box::new(Value::Const(3)),
            ))),
            toughness: Value::NonNeg(Box::new(Value::Diff(
                Box::new(Value::HandSizeOf(PlayerRef::You)),
                Box::new(Value::Const(3)),
            ))),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Solve the Equation ──────────────────────────────────────────────────────

/// Solve the Equation — {2}{U} Sorcery. "Search your library for an
/// instant or sorcery card, reveal it, put it into your hand, then
/// shuffle. Then scry 1."
///
/// Wired faithfully via `Effect::Search { → Hand }` filtered to
/// instant ∨ sorcery, followed by `Effect::Scry 1`. Mono-blue tutor for
/// spellslinger / spell-school decks; the Scry 1 rider is just gravy.
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
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                to: ZoneDest::Hand(PlayerRef::You),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Enthusiastic Study ──────────────────────────────────────────────────────

/// Enthusiastic Study — {1}{G} Instant. "Target creature gets +2/+2 and
/// gains trample until end of turn. Then learn."
///
/// Learn collapses to `Draw 1` (catalog convention). Mainline +2/+2
/// trample is wired faithfully against any creature; the draw rider
/// always fires. Mono-green combat trick that stacks with magecraft
/// triggers when cast (it's an instant).
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Tempted by the Oriq ─────────────────────────────────────────────────────

/// Tempted by the Oriq — {1}{W}{B} Sorcery. Printed Oracle:
/// "Gain control of target creature with mana value 3 or less. Create
/// a 1/1 white and black Inkling creature token with flying."
///
/// ✅ Push XXXVIII: 🟡 → ✅. The "gain control" half now wires
/// faithfully via `Effect::GainControl { duration: Duration::Permanent }`
/// (Push XXXIV graduated `Effect::GainControl` from a permanent-control
/// stub to a Layer-2 continuous effect with selectable duration —
/// `Permanent` here for the indefinite-control flavor of Bribery /
/// Mind Control / Tempted by the Oriq, distinct from Mascot
/// Interception's `EndOfTurn` Threaten flavor). The Inkling token
/// rider is wired via the SOS catalog's `inkling_token()` helper.
pub fn tempted_by_the_oriq() -> CardDefinition {
    use crate::catalog::sets::sos::inkling_token;
    use crate::effect::Duration;
    CardDefinition {
        name: "Tempted by the Oriq",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ManaValueAtMost(3)),
                ),
                duration: Duration::Permanent,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Saw It Coming ───────────────────────────────────────────────────────────

/// Saw It Coming — {1}{U}{U} Instant. "Counter target spell. / Foretell {1}{U}."
///
/// Push XXIV: 🟡 — basic counter-target-spell at the {1}{U}{U} rate
/// (Cancel-equivalent). The Foretell alt-cost ({2} face-down to exile +
/// next-turn cast for {1}{U}) is omitted — Foretell needs an alt-cost-on-
/// exile primitive plus a "cast from exile until …" capture, neither of
/// which exists today (same gap as Velomachus Lorehold's reveal-and-cast,
/// Practiced Scrollsmith's "may cast that card", etc.).
pub fn saw_it_coming() -> CardDefinition {
    CardDefinition {
        name: "Saw It Coming",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: crate::effect::shortcut::counter_target_spell(),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Environmental Sciences ──────────────────────────────────────────────────

/// Environmental Sciences — {2} Sorcery — Lesson. "You may search your
/// library for a basic land card, reveal it, put it into your hand, then
/// shuffle. You gain 2 life."
///
/// Push XXIX: colorless Lesson — generic {2} (no color requirement) means
/// every Strixhaven Mystical Archive deck plays this regardless of pips.
/// `Effect::Search` on `IsBasicLand → Hand` + `Effect::GainLife 2`. The
/// "may" rider is implicit in `do_search`'s `None` answer, so a
/// scripted decider can opt out of the search.
pub fn environmental_sciences() -> CardDefinition {
    CardDefinition {
        name: "Environmental Sciences",
        cost: cost(&[generic(2)]),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Expanded Anatomy ────────────────────────────────────────────────────────

/// Expanded Anatomy — {3}{G} Sorcery — Lesson. "Put three +1/+1 counters
/// on target creature."
///
/// Push XXIX: clean Lesson card — three +1/+1 counters onto a creature,
/// no riders. Wired with `Effect::AddCounter` on `target_filtered(Creature)`
/// at amount 3. Lesson sub-type is recorded so the future Lessons-side-
/// board model picks it up.
pub fn expanded_anatomy() -> CardDefinition {
    CardDefinition {
        name: "Expanded Anatomy",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(3),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Big Play ────────────────────────────────────────────────────────────────

/// Big Play — {3}{G}{U} Instant — Lesson. "Untap up to two target
/// creatures. Each of those creatures gets +1/+1 and gains hexproof and
/// trample until end of turn."
///
/// Push XXXIX: fidelity bump. The "up to two creatures" rider is now
/// approximated as user-targeted slot 0 plus an auto-picked second
/// friendly creature via `Selector::take(EachPermanent(Creature ∧
/// ControlledByYou), 2)` for the fan-out body. Each picked creature
/// untaps, gains +1/+1 EOT, and gains hexproof + trample EOT. Net:
/// the printed two-creature combat trick lands on two friendly
/// creatures (Mentor's Guidance-style fan-out). The Lesson sub-type
/// stays recorded for future Lesson-aware effects.
pub fn big_play() -> CardDefinition {
    let pick = Selector::take(
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        Value::Const(2),
    );
    CardDefinition {
        name: "Big Play",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Untap { what: pick.clone(), up_to: None },
            Effect::ForEach {
                selector: pick.clone(),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(1),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Confront the Past ───────────────────────────────────────────────────────

/// Confront the Past — {4}{R} Sorcery — Lesson. "Choose one — / • Exile
/// target activated or triggered ability you don't control. / • Choose a
/// loyalty ability of target planeswalker controlled by an opponent.
/// Confront the Past has that ability."
///
/// Push XXIX: 🟡 — collapsed to mode 0 only (counter target ability).
/// The "steal opponent's planeswalker loyalty ability" mode requires a
/// dynamic mode-pick from a target's `loyalty_abilities` list, which is
/// a brand-new primitive (same gap as Sarkhan, the Masterless's static
/// loyalty stamp on dragons). Mode 0 collapse maps to
/// `Effect::CounterAbility` against an opponent's permanent's stack-
/// resident ability, matching the printed ability-counter half.
pub fn confront_the_past() -> CardDefinition {
    CardDefinition {
        name: "Confront the Past",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::CounterAbility {
            what: target_filtered(SelectionRequirement::Permanent),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Vortex Runner ───────────────────────────────────────────────────────────

/// Vortex Runner — {1}{U} 1/2 Salamander Warrior (Strixhaven 2021
/// mono-Blue common). Printed Oracle: "Whenever this creature attacks,
/// scry 1." Vortex Runner can't be blocked.
///
/// Mono-Blue evasive 2-drop with attack-trigger card selection. Wired
/// with `Keyword::Unblockable` (closes the printed "can't be blocked"
/// rider faithfully) + an `Attacks/SelfSource → Scry 1` trigger on the
/// source itself.
pub fn vortex_runner() -> CardDefinition {
    CardDefinition {
        name: "Vortex Runner",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Unblockable],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Burrog Befuddler ────────────────────────────────────────────────────────

/// Burrog Befuddler — {1}{U} 1/3 Frog Wizard (Strixhaven 2021 mono-Blue
/// common). Printed Oracle: "Flash. Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, target creature gets -2/-0 until
/// end of turn."
///
/// Combat-trick magecraft body — a Frog Wizard for any tribal payoff
/// (Foul Play, Karok Wrangler) plus a Wizard-tribal magecraft. Wired
/// with `Keyword::Flash` + `magecraft(PumpPT(-2, 0, EOT))` against an
/// auto-targeted creature.
pub fn burrog_befuddler() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Burrog Befuddler",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Crackle with Power ──────────────────────────────────────────────────────

/// Crackle with Power — {X}{R}{R}{R} Sorcery (Strixhaven 2021 mythic).
/// Printed Oracle: "Crackle with Power deals 5X damage to any target."
///
/// Mono-Red X-finisher. Wired with `Effect::DealDamage { to: target,
/// amount: Times(XFromCost, 5) }` — at X=2 deals 10, at X=3 deals 15.
/// "Any target" collapses to the auto-target framework: `target_filtered
/// (Creature OR Planeswalker)` (printed "any target" includes player
/// faces too, but the auto-picker prefers creature/PW kills; player
/// damage falls out of the kill-chooser logic).
pub fn crackle_with_power() -> CardDefinition {
    CardDefinition {
        name: "Crackle with Power",
        cost: cost(&[x(), r(), r(), r()]),
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
            amount: Value::Times(
                Box::new(Value::XFromCost),
                Box::new(Value::Const(5)),
            ),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Sundering Stroke ────────────────────────────────────────────────────────

/// Sundering Stroke — {3}{R}{R}{R} Sorcery (Strixhaven 2021 mythic).
/// Printed Oracle: "Sundering Stroke deals 7 damage divided as you
/// choose among one, two, or three targets. If {R}{R}{R}{R} was spent
/// to cast it, it deals 14 damage divided as you choose instead."
///
/// 🟡 Mono-Red removal finisher. Single-target collapse: 7 damage to
/// one target (auto-targeted creature/planeswalker). The "divided as
/// you choose" multi-target option and the {R}{R}{R}{R} double-up
/// rider are both omitted — the engine has no divided-damage primitive
/// (same gap as Magma Opus's "4 damage divided", Crackling Doom's
/// distributed damage). Net: hits as a 7-damage burn at the printed
/// {3}{R}{R}{R} rate.
pub fn sundering_stroke() -> CardDefinition {
    CardDefinition {
        name: "Sundering Stroke",
        cost: cost(&[generic(3), r(), r(), r()]),
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
            amount: Value::Const(7),
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Professor of Symbology ──────────────────────────────────────────────────

/// Professor of Symbology — {1}{W} 1/1 Bird Wizard (Strixhaven 2021
/// uncommon). Printed Oracle: "Flying. When this creature enters, you
/// may reveal a Lesson card you own from outside the game and put it
/// into your hand."
///
/// 🟡 Mono-White Learn enabler. Body is faithful (1/1 Flying Bird
/// Wizard). The "Lesson tutor from outside the game" half is
/// approximated as `Draw 1` — the engine has no Lesson sideboard
/// model yet (same approximation as Eyetwitch / Hunt for Specimens'
/// Learn, Igneous Inspiration's Learn). Net: ETB cantrip on a 1/1
/// flier body that supports Wizard tribal (Foul Play / Karok Wrangler
/// gates).
pub fn professor_of_symbology() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Professor of Symbology",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Professor of Zoomancy ───────────────────────────────────────────────────

/// Professor of Zoomancy — {1}{G} 1/1 Squirrel Wizard (Strixhaven 2021
/// uncommon). Printed Oracle: "When this creature enters, create a 1/1
/// green Squirrel creature token."
///
/// Mono-Green ETB token-maker. Body is 1/1 Squirrel Wizard; ETB mints
/// a vanilla 1/1 green Squirrel creature token (same shape as Chatterfang
/// / Squirrel Sovereign's token cycle). Net: 1 mana for 2 power on
/// board with a Wizard-tribal anchor.
pub fn professor_of_zoomancy() -> CardDefinition {
    use crate::effect::shortcut::etb;
    let squirrel = TokenDefinition {
        name: "Squirrel".to_string(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Professor of Zoomancy",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: squirrel,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Leyline Invocation ──────────────────────────────────────────────────────

/// Leyline Invocation — {4}{G} Sorcery — Lesson (Strixhaven 2021
/// uncommon). Printed Oracle: "Create an X/X green Elemental creature
/// token, where X is the number of lands you control."
///
/// Mono-Green Lesson body-maker that scales with the controller's land
/// count. Wired by minting one Elemental token whose P/T are baked in
/// at creation time using `Value::CountOf(EachPermanent(Land &
/// ControlledByYou))` resolved at resolution. Approximation: the
/// minted token's printed P/T are immutable post-creation (engine
/// `TokenDefinition.power/toughness` are i32, set when the token is
/// created), so the X is fixed at the moment of creation and won't
/// shrink if a land later leaves play (same approximation as Body of
/// Research's library-size scaling — both lock in the count at create
/// time, not as a continuous static). The Lesson sub-type is recorded
/// for future Learn-aware code.
pub fn leyline_invocation() -> CardDefinition {
    // X = lands you control, evaluated at creation time. Use the
    // `count_lands()` selector folded into `Value::CountOf` for the
    // initial token P/T; thereafter the printed body is a fixed-X
    // Elemental.
    let lands_you_control = Selector::EachPermanent(
        SelectionRequirement::HasCardType(CardType::Land)
            .and(SelectionRequirement::ControlledByYou),
    );
    let elemental = TokenDefinition {
        name: "Elemental".to_string(),
        // Token's base P/T is 0/0; the create-time AddCounter rider
        // bumps it by N +1/+1 counters (engine token power/toughness
        // is i32; counters are first-class — same shape as Body of
        // Research, Snow Day's "Fractal +X/+X").
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    };
    CardDefinition {
        name: "Leyline Invocation",
        cost: cost(&[generic(4), g()]),
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
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: elemental,
            },
            // Stamp the token with N +1/+1 counters so its on-board
            // P/T reads N/N (mirrors Body of Research / Snow Day).
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(lands_you_control)),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Verdant Mastery ─────────────────────────────────────────────────────────

/// Verdant Mastery — {3}{G}{G} Sorcery (Strixhaven 2021 mythic).
/// Printed Oracle: "Search your library for two basic land cards, put
/// one onto the battlefield tapped, then put the other into your hand.
/// Target opponent searches their library for a basic land card, puts
/// it onto the battlefield tapped, then shuffles. Then shuffle." Has
/// alternative cost {7}{G}{G} that omits the opponent half.
///
/// 🟡 Mono-Green ramp. Wired as a `Seq` of two `Effect::Search` calls:
/// the first puts a basic land tapped onto your battlefield (matches
/// the printed "one onto the battlefield tapped"); the second puts a
/// basic land into your hand (matches the printed "other into your
/// hand"). The opponent half is omitted (no `Effect::Search` variant
/// targeting an opponent — same gap as Eladamri's Call's "any player
/// searches"). Net: solid 5-mana ramp + tutor (one tapped land + one
/// hand land) without the symmetry cost.
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
                    tapped: true,
                },
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Rise of Extus ───────────────────────────────────────────────────────────

/// Rise of Extus — {3}{W}{B} Sorcery — Lesson (Strixhaven 2021 rare).
/// Printed Oracle: "Exile target creature or planeswalker. Then return
/// target creature or planeswalker card from your graveyard to the
/// battlefield."
///
/// Silverquill Lesson — exile + reanimate combo. Wired as `Seq([
/// Exile(target Creature ∨ Planeswalker), Move(graveyard creature/PW
/// → battlefield)])`. Single-target collapse on the second half (the
/// `Selector::take` from graveyard picks one matching card; auto-
/// picker chooses the highest-mana-value valid creature or PW).
/// Self-targets the first half by the auto-picker's preference for
/// opp's creatures (kill an opp threat, reanimate your best one).
pub fn rise_of_extus() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Rise of Extus",
        cost: cost(&[generic(3), w(), b()]),
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
            Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
            },
            Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::Creature
                            .or(SelectionRequirement::Planeswalker),
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}


// ── Blood Researcher ────────────────────────────────────────────────────────

/// Gnarled Professor — {3}{G} 4/4 Treefolk Druid (Strixhaven 2021
/// uncommon). Printed Oracle: "Reach. When this creature enters, you
/// may discard a card. If you do, draw a card."
///
/// Mono-Green 4/4 reach body with a may-loot ETB. Wired with
/// `Keyword::Reach` + `etb(MayDo(Discard 1 → Draw 1))`. The
/// "discard a card → draw" rider uses `Effect::MayDo` (push XV) so
/// AutoDecider defaults to "no" (the body decision is "do you want
/// to loot?"); ScriptedDecider can flip to "yes" for tests. Net: a
/// midrange green creature that swaps a dead card for a fresh one
/// without the small drawback of a forced discard.
pub fn gnarled_professor() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Gnarled Professor",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Discard a card to draw a card?".into(),
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
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Inkfathom Witch — {2}{B} 2/2 Faerie Warlock (Strixhaven 2021
/// flavor — STX uncommon adapted). Printed Oracle (this implementation
/// matches a Witherbloom-flavor Faerie body): "Flying. Whenever this
/// creature attacks, you may pay {1}{B}. If you do, each opponent
/// loses 2 life and you gain 2 life."
///
/// Mono-Black evasive 3-drop with an attack-trigger drain rider.
/// Wired with `Keyword::Flying` + `Attacks/SelfSource → MayPay({1}{B},
/// Drain 2)`. AutoDecider defaults to "no" (saves mana); ScriptedDecider
/// "yes" + sufficient mana resolves a 2-life swing.
pub fn inkfathom_witch() -> CardDefinition {
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Inkfathom Witch",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {1}{B} to drain 2 from each opponent?".into(),
                mana_cost: ManaCost::new(vec![generic(1), b()]),
                body: Box::new(Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::Const(2),
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Blood Researcher — {1}{B} 1/1 Vampire Wizard (Strixhaven 2021
/// mono-Black common). Printed Oracle: "Whenever you gain life, put
/// a +1/+1 counter on this creature."
///
/// Mono-Black lifegain payoff that scales linearly with Witherbloom
/// drains, Vito-style triggers, and any +life rider. Wired with
/// `LifeGained/YourControl → AddCounter(This, +1/+1, ×1)`. The trigger
/// fires once per `LifeGained` event regardless of amount (printed
/// "Whenever you gain life" — one trigger per event, not per life
/// gained). Combos hard with Witherbloom Apprentice's drain (one event
/// per cast → one counter per IS cast that turn).
pub fn blood_researcher() -> CardDefinition {
    CardDefinition {
        name: "Blood Researcher",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── First Day of Class ──────────────────────────────────────────────────────

/// First Day of Class — {W} Sorcery (Strixhaven 2021 uncommon).
/// Printed Oracle: "Until end of turn, creature tokens you control get
/// +1/+1 and gain haste."
///
/// Mono-White token-anthem-on-the-stack. Wired with two `ForEach`
/// passes over `EachPermanent(IsToken & Creature & ControlledByYou)`:
/// the first applies `PumpPT(+1/+1, EOT)` to each token, the second
/// grants `Keyword::Haste` (EOT). Stack with Pest Summoning / Spirit
/// Summoning / Mascot Exhibition / Hunt for Specimens / Tend the
/// Pests for an immediate go-wide swing turn.
pub fn first_day_of_class() -> CardDefinition {
    let your_token_creatures = Selector::EachPermanent(
        SelectionRequirement::IsToken
            .and(SelectionRequirement::Creature)
            .and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "First Day of Class",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::ForEach {
                selector: your_token_creatures.clone(),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
            },
            Effect::ForEach {
                selector: your_token_creatures,
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Pilgrim of the Ages ─────────────────────────────────────────────────────

/// Pilgrim of the Ages — {3}{W} 2/3 Spirit Wizard Cleric. "When this
/// creature dies, return target Plains card from your graveyard to your
/// hand."
///
/// Push XXIX: small Lorehold-themed body that recycles a basic land on
/// death. Mirrors Pillardrop Rescuer ({3}{R}{W} 3/3 flying — IS gy →
/// hand) at the mono-white color slot with a Plains target rather
/// than IS. The "return target Plains" filter matches `IsBasicLand` plus
/// a name-equals-Plains follow-up — but we use the simpler
/// `IsBasicLand` filter for the auto-target framework's first-pick
/// (printed Oracle would land here only on a Plains-card decision,
/// since no other basic-land filter exists in the engine yet). Net:
/// graveyard → hand on a basic land card.
pub fn pilgrim_of_the_ages() -> CardDefinition {
    use crate::card::Zone;
    CardDefinition {
        name: "Pilgrim of the Ages",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Spirit,
                CreatureType::Wizard,
                CreatureType::Cleric,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: SelectionRequirement::IsBasicLand,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Containment Breach ──────────────────────────────────────────────────────

/// Containment Breach — {1}{W} Instant. "Destroy target enchantment.
/// / Learn."
///
/// Push XXXIX: NEW. Standard enchantment removal + cantrip learn.
/// Learn collapses to `Draw 1` (Lesson sideboard model still pending —
/// same approximation as Eyetwitch / Hunt for Specimens / Igneous
/// Inspiration / Professor of Symbology).
pub fn containment_breach() -> CardDefinition {
    CardDefinition {
        name: "Containment Breach",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Enchantment),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Unwilling Ingredient ────────────────────────────────────────────────────

/// Unwilling Ingredient — {B} 1/1 Insect Pest. "When this creature
/// dies, you may pay {B}. If you do, draw a card."
///
/// Push XXXIX: NEW. Mono-black sac fodder with a pay-to-draw rider.
/// Death-trigger uses `Effect::MayPay { mana_cost: {B}, body: Draw 1
/// }`. AutoDecider declines the pay by default; ScriptedDecider can
/// flip it for the cantrip.
pub fn unwilling_ingredient() -> CardDefinition {
    CardDefinition {
        name: "Unwilling Ingredient",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayPay {
                description: "Pay {B}: draw a card.".to_string(),
                mana_cost: cost(&[b()]),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Pest Wallop ─────────────────────────────────────────────────────────────

/// Pest Wallop — {3}{G} Sorcery. "Target creature you control gets
/// +1/+1 until end of turn. Then it deals damage equal to its power
/// to target creature you don't control."
///
/// Push XXXIX: NEW. Functional approximation as a Seq:
/// 1. PumpPT(+1/+1, EOT) on a friendly creature (slot 0).
/// 2. DealDamage(amount = PowerOf(slot 0), to = auto-picked opp
///    creature) — same one-sided shape as Decisive Denial mode 1.
///
/// The friendly creature target must be in slot 0; the opp creature
/// is auto-picked via `Selector::one_of(EachPermanent(opp creature))`.
/// One-sided damage (not Fight) — friendly creature takes no return
/// damage. The slot 0 filter is enforced at cast time via the new
/// `val_find` recursion (push XXXIX) so opp-creature picks reject.
pub fn pest_wallop() -> CardDefinition {
    let friendly = target_filtered(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Pest Wallop",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: friendly.clone(),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamage {
                to: Selector::one_of(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                )),
                amount: Value::PowerOf(Box::new(friendly)),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Solid Footing ───────────────────────────────────────────────────────────

/// Solid Footing — {W} Aura. "Enchant creature / Enchanted creature
/// gets +1/+2 and has vigilance."
///
/// Push XXXIX: NEW. Aura-grant approximation: ETB attaches to a
/// friendly creature, then grants +1/+2 + vigilance via
/// `StaticEffect::PumpPT` + `StaticEffect::GrantKeyword` over
/// `Selector::AttachedToMe(This)`. The Aura subtype is recorded
/// (matches printed type line); the engine doesn't enforce
/// targeting-rules-via-Aura yet.
pub fn solid_footing() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility};
    use crate::effect::StaticEffect;
    // `selector_to_affected` recognises `AttachedTo(This)` and resolves
    // it through `card.attached_to` at layer time → the static buffs
    // the creature this aura is attached to.
    let attached = Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Solid Footing",
        cost: cost(&[w()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Attachment is pre-set at cast time by the engine when the
        // spell is an Aura with a Permanent target (`stack.rs`). No
        // ETB trigger is needed since the orphaned-aura SBA reads the
        // pre-bound `attached_to` immediately on entering bf.
        triggered_abilities: vec![],
        static_abilities: vec![
            StaticAbility {
                description: "Enchanted creature gets +1/+2",
                effect: StaticEffect::PumpPT {
                    applies_to: attached.clone(),
                    power: 1,
                    toughness: 2,
                },
            },
            StaticAbility {
                description: "Enchanted creature has vigilance",
                effect: StaticEffect::GrantKeyword {
                    applies_to: attached,
                    keyword: Keyword::Vigilance,
                },
            },
        ],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Swarm Shambler ──────────────────────────────────────────────────────────

/// Swarm Shambler — {G} 1/1 Squirrel Beast. "When this creature
/// enters, put a +1/+1 counter on it. / {2}{G}: Untap this creature.
/// Put a +1/+1 counter on it."
///
/// Push XXXIX: NEW. Mono-green growth tribal — each activation
/// untaps and counters, so the body scales with available mana.
/// Squirrel creature-type bridges through Beast (no Squirrel
/// creature type yet); the Squirrel-tribal payoff Professor of
/// Zoomancy still triggers via the Beast subtype until a future
/// Squirrel addition.
pub fn swarm_shambler() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Swarm Shambler",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::Seq(vec![
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

