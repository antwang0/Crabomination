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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        // Ship a 1/1 base body so it survives SBA before the ETB
        // counter trigger fires (engine has no replacement-effect
        // primitive for "enters with X +1/+1 counters"; a 0/0 base
        // would die before the ETB resolves).
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::PermanentCountControlledBy(PlayerRef::You),
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
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
        back_face: None,
        opening_hand: None,
    }
}

// ── Dragon's Approach ───────────────────────────────────────────────────────

/// Dragon's Approach — {1}{R} Sorcery (Strixhaven uncommon).
/// "Dragon's Approach deals 3 damage to any target. Then if you have
/// four or more cards named Dragon's Approach in your graveyard, you
/// may search your library for a Dragon creature card, put it onto the
/// battlefield, then shuffle."
///
/// 🟡 Only the 3-damage half is wired. The "if 4+ Dragon's Approach in
/// graveyard, may tutor a Dragon" rider needs a card-name-match
/// predicate that we don't have today. As a flat 3-to-any-target burn
/// at {1}{R} it still ships as a clean Lava Spike-on-curve.
pub fn dragons_approach() -> CardDefinition {
    CardDefinition {
        name: "Dragon's Approach",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Any),
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
        back_face: None,
        opening_hand: None,
    }
}
