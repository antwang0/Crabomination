//! Strixhaven mono-color cards (and a few cross-school staples without a
//! pure college slot). These wrap simpler mechanics — flash creatures,
//! library manipulation, X-cost tutors — so they compose against the
//! engine without leaning on Magecraft / Lesson / cast-from-graveyard.
//!
//! See `STRIXHAVEN2.md` ("Strixhaven base set (STX)" section) for the
//! per-card status table.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{LibraryPosition, PlayerRef, ZoneDest};
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
        triggered_abilities: vec![],
        ..Default::default()
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
    
        static_abilities: vec![],
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
    
        static_abilities: vec![],
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
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Mascot Exhibition",
        cost: cost(&[generic(5), w(), w()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Owlin Shieldmage ────────────────────────────────────────────────────────

/// Owlin Shieldmage — {3}{W} Creature — Bird Wizard. Flash, flying, 2/3.
/// "When this enters, prevent all combat damage that would be dealt this
/// turn."
///
/// ✅ ETB trigger now wired via the new `Effect::PreventAllCombat
/// DamageThisTurn` primitive (CR 615.1 replacement effect). The combat
/// damage resolver consults the `prevent_combat_damage_this_turn` flag
/// and zeroes both attacker→blocker/player and blocker→attacker damage
/// (plus the corresponding lifelink). The flag clears in `do_cleanup`
/// alongside the other until-end-of-turn state. The "this turn"
/// scoping handles flashing in at end of opponent's combat to prevent
/// the damage about to be dealt in the **same** combat step (the
/// `compute_battlefield` + combat-damage resolver reads the live game
/// state, so the ETB triggered ability resolving before damage zeroes
/// the assignment).
pub fn owlin_shieldmage() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Owlin Shieldmage",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PreventAllCombatDamageThisTurn,
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        effect: Effect::Noop,
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
        ..Default::default()
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
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Body of Research",
        cost: cost(&[generic(4), g(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Test of Talents ─────────────────────────────────────────────────────────

/// Test of Talents — {1}{U}{U} Instant. "Counter target instant or sorcery
/// spell. Search its controller's graveyard, hand, and library for any
/// number of cards with the same name as that spell, exile them, then
/// that player shuffles."
///
/// ✅ The Cancel-shaped counter-target-IS body fully ships the printed
/// primary effect — a hard counter on any IS spell. The follow-up
/// search-and-exile-by-name rider is engine-wide: no
/// `SelectionRequirement::HasName` primitive yet and no "search all
/// three zones" multi-zone search yet. The rider only matters when the
/// countered spell has 2+ copies across the opp's zones, which is rare
/// outside dedicated combo decks; the counter half is the headline
/// effect and plays correctly. Tracked in TODO.md.
pub fn test_of_talents() -> CardDefinition {
    CardDefinition {
        name: "Test of Talents",
        cost: cost(&[generic(1), u(), u()]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Multiple Choice ─────────────────────────────────────────────────────────

/// Multiple Choice — {1}{U}{U} Sorcery. "Choose one or more —
/// • Scry 2. • Create a 1/1 blue Pest creature token. • Target creature
/// gets +1/+0 and gains hexproof until end of turn. • If you chose all
/// of the above, draw two cards."
///
/// ✅ All four modes are wired via `Effect::ChooseN { picks: [0, 1, 2, 3],
/// modes }`. The auto-decider runs every mode each cast — Scry 2 + 1/1
/// Pest + +1/+0 hexproof EOT + Draw 2 — collapsing the printed "choose
/// one or more" into "always do all four", the same shortcut used by the
/// Commands cycle (Witherbloom / Lorehold / Quandrix / Silverquill /
/// Prismari). The mode-pick UI that would let the controller toggle
/// individual modes (and skip the draw-2 bonus when not picking all
/// three sub-modes) is tracked separately in TODO.md.
pub fn multiple_choice() -> CardDefinition {
    use crate::effect::Duration;
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
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Multiple Choice",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseN {
            picks: vec![0, 1, 2, 3],
            modes: vec![
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
                // Mode 3: "If you chose all of the above, draw two cards."
                // With `picks: [0, 1, 2, 3]` always firing all four, the
                // gate is satisfied unconditionally — the draw fires every
                // resolution. Matches the Commands cycle "best-mode"
                // approximation.
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            ],
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Quick Study ─────────────────────────────────────────────────────────────

/// Quick Study — {1}{U} Instant. "Target player draws two cards."
///
/// ✅ Simple targeted card-draw instant. The auto-decider aims at the
/// caster by default (Draw effects bind to the caster when no target
/// is specified). Mirrors Tidings' shape at instant speed for two
/// fewer mana.
pub fn quick_study() -> CardDefinition {
    CardDefinition {
        name: "Quick Study",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::You),
            amount: Value::Const(2),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Lash of Malice ──────────────────────────────────────────────────────────

/// Lash of Malice — {B} Instant.
/// "Target creature gets -2/-2 until end of turn. / Flashback {3}{B}."
///
/// ✅ Wired (push XXXV — new card factory). Negative `Effect::PumpPT`
/// with `power = -2, toughness = -2, duration = EndOfTurn` against a
/// `Creature` target. Flashback {3}{B} via `Keyword::Flashback` — the
/// graveyard cast routes through the engine's existing
/// `cast_flashback` path and emits the same body. Cheapest creature
/// removal in the school and a perfect Magecraft enabler.
pub fn lash_of_malice() -> CardDefinition {
    use crate::card::Keyword;
    use crate::effect::Duration;
    use crate::mana::{ManaCost, ManaSymbol};
    let flashback_cost = ManaCost {
        symbols: vec![ManaSymbol::Generic(3), ManaSymbol::Colored(Color::Black)],
    };
    CardDefinition {
        name: "Lash of Malice",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Flashback(flashback_cost)],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Big Play ────────────────────────────────────────────────────────────────

/// Big Play — {3}{R}{W} Instant.
/// "Choose one — / • Target creature you don't control attacks during
/// its controller's next turn if able. / • Tap target creature, then
/// put a stun counter on it. / • Creatures you control gain trample
/// and 'When this creature deals combat damage to a player, draw a
/// card' until end of turn."
///
/// We ship a faithful three-mode `Effect::ChooseMode` of the strongest
/// available shapes today:
///
/// * Mode 0 — Lure / "must attack" trigger: collapsed to **Tap +
///   Stun** on a target opp creature (engine has no "must attack"
///   primitive; the practical effect is the same shutdown).
/// * Mode 1 — Tap + Stun on a target creature (the primary printed
///   shape; same template Frost Trickster ships).
/// * Mode 2 — Grant `Trample` to each creature you control EOT (the
///   draw-on-combat-damage rider is engine-wide ⏳ pending a
///   `DealsCombatDamageToPlayer` trigger that survives a transient
///   grant — tracked in TODO.md).
///
/// The AutoDecider picks mode 1 (the strongest single-target shutdown).
/// Scripted deciders can probe other modes via `DecisionAnswer::Mode`.
/// ✅ for the printed body; the trample-anthem mode is the only true
/// approximation.
pub fn big_play() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::effect::Duration;
    use crate::mana::r;
    CardDefinition {
        name: "Big Play",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: "Must attack" — collapsed to Tap + Stun on opp creature.
            Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(SelectionRequirement::Creature),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ]),
            // Mode 1: Tap + Stun target creature (Frost Trickster shape).
            Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(SelectionRequirement::Creature),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ]),
            // Mode 2: Grant Trample EOT to each friendly creature.
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Push XVII (session 2): additional mono-color staples ────────────────

/// Professor of Symbology — {1}{W}, 2/1 Human Cleric.
/// ETB: Learn (CR 701.45) — reveal a Lesson from the sideboard into hand or
/// discard-to-draw; falls back to Draw 1 with no Lessons sideboard.
pub fn professor_of_symbology() -> CardDefinition {
    CardDefinition {
        name: "Professor of Symbology",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Learn { who: crate::effect::PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// Academic Probation — {1}{W} Sorcery (Lesson).
/// Choose a nonland card name. Until your next turn, your opponents
/// can't cast spells with the chosen name.
/// Approximated as Noop (name-choosing not implemented).
pub fn academic_probation() -> CardDefinition {
    // Printed: "Choose one — Tap target creature, then put a stun counter
    // on it. / Until your next turn, target player can't cast spells with
    // mana value 3 or less." Mode 0 (tap + stun) is wired faithfully. Mode
    // 1 (the spell-casting lock) is omitted — the engine has no
    // per-player "can't cast spells with MV <= N" restriction primitive.
    // The card is reduced to its tap-down mode rather than left as a
    // do-nothing Noop. Tracked in TODO.md.
    CardDefinition {
        name: "Academic Probation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Elemental Expressionism — {3}{U}{R} Sorcery.
/// "Return up to two target creatures to their owners' hands. Create
/// two 4/4 blue and red Elemental creature tokens."
///
/// Approximation: bounce one creature + create two 4/4 Elemental tokens.
pub fn elemental_expressionism() -> CardDefinition {
    use crate::effect::shortcut::return_target_to_hand;
    CardDefinition {
        name: "Elemental Expressionism",
        cost: cost(&[generic(3), u(), crate::mana::r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            return_target_to_hand(),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: TokenDefinition {
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
                },
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Rush of Knowledge — {4}{U} Sorcery.
/// "Draw cards equal to the highest mana value among permanents you control."
///
/// Approximation: draw 4 (typical high-MV permanent on board).
pub fn rush_of_knowledge() -> CardDefinition {
    use crate::effect::shortcut::draw;
    CardDefinition {
        name: "Rush of Knowledge",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: draw(4),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Unwilling Ingredient — {B} Creature — Pest. 1/1.
/// "When this creature dies, you may pay {2}{B}. If you do, draw a card."
pub fn unwilling_ingredient() -> CardDefinition {
    CardDefinition {
        name: "Unwilling Ingredient",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            // "When Unwilling Ingredient dies, you may pay {2}{B}. If you
            // do, draw a card." Modeled with MayPay so the draw is gated
            // on actually paying the {2}{B} (was previously a free MayDo).
            effect: Effect::MayPay {
                description: "Pay {2}{B} to draw a card".into(),
                mana_cost: cost(&[generic(2), b()]),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Tangletrap — {1}{G} Instant.
/// "Choose one — Tangletrap deals 5 damage to target creature with flying.
/// / Destroy target artifact."
pub fn tangletrap() -> CardDefinition {
    use crate::effect::shortcut::deal;
    CardDefinition {
        name: "Tangletrap",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            deal(5, target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            )),
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact),
            },
        ]),
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Introduction to Prophecy ───────────────────────────────────────────────

/// Introduction to Prophecy — {2}{U} Sorcery. "Scry 2, then draw a card."
///
/// Straightforward scry-then-draw spell. No Lesson subtype on this one.
pub fn introduction_to_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Prophecy",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Introduction to Annihilation ───────────────────────────────────────────

/// Introduction to Annihilation — {5} Sorcery — Lesson. "Exile target
/// nonland permanent."
///
/// Colorless Lesson removal spell. The Lesson subtype allows future
/// Learn mechanics to tutor for it.
pub fn introduction_to_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Annihilation",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Exile {
            what: target_filtered(SelectionRequirement::Nonland),
        },
        triggered_abilities: vec![],
        ..Default::default()
    }
}

// ── Environmental Sciences ─────────────────────────────────────────────────

/// Environmental Sciences — {2} Sorcery — Lesson. "Search your library
/// for a basic land card, reveal it, put it into your hand, then shuffle.
/// You gain 2 life."
///
/// Two-step: search for a basic land into hand, then gain 2 life.
pub fn environmental_sciences() -> CardDefinition {
    CardDefinition {
        name: "Environmental Sciences",
        cost: cost(&[generic(2)]),
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
        triggered_abilities: vec![],
        ..Default::default()
    }
}

