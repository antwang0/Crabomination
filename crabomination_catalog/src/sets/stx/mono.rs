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
use crate::mana::{Color, b, cost, g, generic, r, u, w};

// ── Pop Quiz ────────────────────────────────────────────────────────────────

/// Pop Quiz — {2}{U} Instant. "Draw a card. Learn."
pub fn pop_quiz() -> CardDefinition {
    CardDefinition {
        name: "Pop Quiz",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Learn { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

// ── Mascot Exhibition ───────────────────────────────────────────────────────

/// Mascot Exhibition — {7} Sorcery — Lesson. Create a 2/1 white-and-black
/// Inkling with flying, a 3/2 red-and-white Spirit, and a 4/4 blue-and-red
/// Elemental.
pub fn mascot_exhibition() -> CardDefinition {
    let token = |name: &str, power, toughness, colors, ctype, keywords| TokenDefinition {
        name: name.to_string(),
        power,
        toughness,
        keywords,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: vec![ctype], ..Default::default() },
        ..Default::default()
    };
    let mint = |t: TokenDefinition| Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: t,
    };
    CardDefinition {
        name: "Mascot Exhibition",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            mint(token("Inkling", 2, 1, vec![Color::White, Color::Black], CreatureType::Inkling, vec![Keyword::Flying])),
            mint(token("Spirit", 3, 2, vec![Color::Red, Color::White], CreatureType::Spirit, vec![])),
            mint(token("Elemental", 4, 4, vec![Color::Blue, Color::Red], CreatureType::Elemental, vec![])),
        ]),
        ..Default::default()
    }
}

// ── Plumb the Forbidden ─────────────────────────────────────────────────────

/// Plumb the Forbidden — {1}{B} Instant. "As an additional cost to cast
/// this spell, you may sacrifice one or more creatures. When you
/// sacrifice a creature this way, copy this spell. / You draw a card and
/// you lose 1 life."
///
/// Approximation: modeled as "sacrifice X of your creatures at
/// resolution, then draw X + 1 cards and lose X + 1 life" (X = the
/// cast-time `x_value`, read via `Value::XFromCost`; the `+ 1` is the
/// original spell's own draw/life alongside its X copies). Still missing
/// vs. the printed card: the sacrifice is a resolution-time effect, not
/// a cast-time additional cost (the creatures are still on the
/// battlefield while the spell is on the stack, and removal can't fizzle
/// the copies), and the copies are not real spell objects — no per-copy
/// magecraft triggers and no per-copy responses. Faithful support needs
/// a "sacrifice-as-additional-cost → copy this spell per sacrifice"
/// primitive.
pub fn plumb_the_forbidden() -> CardDefinition {
    CardDefinition {
        name: "Plumb the Forbidden",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::XFromCost,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
            // X copies + the original each draw 1 / lose 1 → X + 1 total.
            Effect::Draw {
                who: Selector::You,
                amount: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Sum(vec![Value::XFromCost, Value::Const(1)]),
            },
        ]),
        ..Default::default()
    }
}

// ── Owlin Shieldmage ────────────────────────────────────────────────────────

/// Owlin Shieldmage — {3}{W}{B} 3/3 Bird Warlock with flying and Ward—Pay 3
/// life.
pub fn owlin_shieldmage() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Owlin Shieldmage",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Life(3))],
        ..Default::default()
    }
}

// ── Frost Trickster ─────────────────────────────────────────────────────────

/// Frost Trickster — {2}{U} 2/2 Bird Wizard with flying. "When this creature
/// enters, tap target creature an opponent controls. That creature doesn't
/// untap during its controller's next untap step." (Modeled as tap + a stun
/// counter, which prevents the next untap.)
pub fn frost_trickster() -> CardDefinition {
    CardDefinition {
        name: "Frost Trickster",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
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
        ..Default::default()
    };
    CardDefinition {
        name: "Body of Research",
        cost: cost(&[generic(4), g(), u()]),
        card_types: vec![CardType::Sorcery],
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
        ..Default::default()
    }
}

// ── Show of Confidence ──────────────────────────────────────────────────────

/// Show of Confidence — {1}{W} Instant. "When you cast this spell, copy
/// it for each other instant or sorcery spell you've cast this turn. You
/// may choose new targets for the copies. / Put a +1/+1 counter on
/// target creature. It gains vigilance until end of turn."
///
/// Real cast-time stack copies via `Keyword::SpellStorm` — one per
/// other instant/sorcery cast this turn, each resolving its own
/// counter + vigilance (Storm machinery: copies sit above the original
/// and can be aimed separately by the auto-retargeter).
pub fn show_of_confidence() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Show of Confidence",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        // Real cast-time copies via `Keyword::SpellStorm` — one copy per
        // other instant/sorcery cast this turn, each resolving its own
        // counter + vigilance (auto-retarget picks per copy).
        keywords: vec![Keyword::SpellStorm],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Bury in Books ───────────────────────────────────────────────────────────

/// Bury in Books — {4}{U} Instant. "This spell costs {2} less to cast if
/// it targets an attacking creature. / Put target creature into its
/// owner's library second from the top."
///
/// The attack discount rides `self_cost_reduction_if_target` (generic-only
/// reduction, evaluated against the chosen target at cast time); the
/// placement is `LibraryPosition::FromTop(1)` — second from the top, with
/// the CR 401.7 "fewer than N cards → bottom" fallback handled by the
/// engine.
pub fn bury_in_books() -> CardDefinition {
    CardDefinition {
        name: "Bury in Books",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::IsAttacking, 2)),
        effect: Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::FromTop(1),
            },
        },
        ..Default::default()
    }
}

// ── Test of Talents ─────────────────────────────────────────────────────────

/// Test of Talents — {1}{U} Instant. "Counter target instant or sorcery
/// spell. Search its controller's graveyard, hand, and library for any
/// number of cards with the same name as that spell and exile them.
/// That player shuffles, then draws a card for each card exiled from
/// their hand this way."
///
/// ✅ Fully wired via `Effect::CounterSpellExileSameNamed`: hard-counter
/// the IS spell, exile every same-named card from its owner's
/// graveyard/hand/library (including the countered copy, which hits the
/// graveyard first), shuffle, and that player draws per hand-exile.
pub fn test_of_talents() -> CardDefinition {
    CardDefinition {
        name: "Test of Talents",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterSpellExileSameNamed {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack
                    .and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
            ),
        },
        ..Default::default()
    }
}

// ── Multiple Choice ─────────────────────────────────────────────────────────

/// Multiple Choice — {X}{U} Sorcery.
/// "If X is 1, scry 1, then draw a card.
/// If X is 2, you may choose a player. They return a creature they
/// control to its owner's hand.
/// If X is 3, create a 4/4 blue and red Elemental creature token.
/// If X is 4 or more, do all of the above."
///
/// Each branch gates on the cast-time X (`Value::XFromCost`): the
/// "X is N" checks are `ValueEquals(X, N)`, and "X is 4 or more"
/// re-enables every branch via `Any([.., ValueAtLeast(X, 4)])`.
/// Approximation in the X=2 branch: the printed text lets the caster
/// choose any player, who then picks their own creature to bounce.
/// There is no "chosen player picks" decision primitive, so the caster
/// directly picks the bounced creature (any creature on the
/// battlefield) inside a MayDo — same set of reachable outcomes minus
/// the bounced creature's controller making the pick.
pub fn multiple_choice() -> CardDefinition {
    use crate::effect::Predicate;
    let x_is = |n: i32| Predicate::Any(vec![
        Predicate::ValueEquals(Value::XFromCost, Value::Const(n)),
        Predicate::ValueAtLeast(Value::XFromCost, Value::Const(4)),
    ]);
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
        static_abilities: vec![],
        ..Default::default()
    };
    CardDefinition {
        name: "Multiple Choice",
        cost: cost(&[crate::mana::x(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // "If X is 1, scry 1, then draw a card."
            Effect::If {
                cond: x_is(1),
                then: Box::new(Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ])),
                else_: Box::new(Effect::Noop),
            },
            // "If X is 2, you may choose a player. They return a creature
            // they control to its owner's hand." The chosen player picks
            // their own creature (`PlayerReturnsPermanentsToHand`);
            // approximation: the choose-a-player step defaults to the
            // opponent (the overwhelmingly common pick in 1v1).
            Effect::If {
                cond: x_is(2),
                then: Box::new(Effect::MayDo {
                    description: "Have the opponent return a creature they control to its owner's hand?".into(),
                    body: Box::new(Effect::PlayerReturnsPermanentsToHand {
                        who: PlayerRef::EachOpponent,
                        count: Value::Const(1),
                        filter: SelectionRequirement::Creature,
                        up_to: false,
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
            // "If X is 3, create a 4/4 blue and red Elemental creature token."
            Effect::If {
                cond: x_is(3),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: elemental,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Quick Study ─────────────────────────────────────────────────────────────

/// Quick Study — {2}{U} Instant. "Target player draws two cards."
///
/// ✅ Simple targeted card-draw instant. The auto-decider aims at the
/// caster by default (Draw effects bind to the caster when no target
/// is specified). Mirrors Tidings' shape at instant speed for two
/// fewer mana.
pub fn quick_study() -> CardDefinition {
    CardDefinition {
        name: "Quick Study",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::You),
            amount: Value::Const(2),
        },
        ..Default::default()
    }
}

// ── Lash of Malice ──────────────────────────────────────────────────────────

/// Lash of Malice — {B} Instant. "Target creature gets +2/-2 until end
/// of turn."
///
/// One pip, one clause: a +2/-2 `Effect::PumpPT` on a creature target.
/// (An earlier revision shipped a synthesized -2/-2 body with a
/// Flashback {3}{B} rider the printed card never had; both are gone.)
pub fn lash_of_malice() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Lash of Malice",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Big Play ────────────────────────────────────────────────────────────────

/// Big Play — {1}{G} Instant. "Target creature gets +2/+2 and gains reach until
/// end of turn. Put a +1/+1 counter on it."
///
/// Fully faithful: pump + reach EOT + a permanent +1/+1 counter, all on
/// the single target. (An earlier stale doc block describing a modal
/// "choose one" spell belonged to a different design and was removed.)
pub fn big_play() -> CardDefinition {
    use crate::card::{CounterType, Keyword};
    use crate::effect::Duration;
    CardDefinition {
        name: "Big Play",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Reach,
                duration: Duration::EndOfTurn,
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
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
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Learn { who: crate::effect::PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// Academic Probation — {1}{W} Sorcery — Lesson. "Choose one —
/// • Choose a nonland card name. Opponents can't cast spells with the
///   chosen name until your next turn. (`NameOpponentCastLock`.)
/// • Choose target nonland permanent. Until your next turn, it can't
///   attack or block, and its activated abilities can't be activated.
///   (Grant CantAttack + CantBlock + CantActivateAbilities for
///   `UntilYourNextUntap`.)"
pub fn academic_probation() -> CardDefinition {
    use crate::card::Keyword;
    use crate::effect::Duration;
    CardDefinition {
        name: "Academic Probation",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::ChooseMode(vec![
            Effect::NameOpponentCastLock,
            Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Nonland),
                    keyword: Keyword::CantAttack,
                    duration: Duration::UntilYourNextUntap,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::UntilYourNextUntap,
                },
                // "…and its activated abilities can't be activated."
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantActivateAbilities,
                    duration: Duration::UntilYourNextUntap,
                },
            ]),
        ]),
        ..Default::default()
    }
}

/// Elemental Expressionism — {3}{U}{R} Sorcery.
/// "Return up to two target creatures to their owners' hands. Create
/// two 4/4 blue and red Elemental creature tokens."
pub fn elemental_expressionism() -> CardDefinition {
    CardDefinition {
        name: "Elemental Expressionism",
        cost: cost(&[generic(3), u(), crate::mana::r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: SelectionRequirement::Creature,
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
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
                    ..Default::default()
                },
            },
        ]),
        ..Default::default()
    }
}

/// Rush of Knowledge — {4}{U} Sorcery.
/// "Draw cards equal to the highest mana value among permanents you control."
pub fn rush_of_knowledge() -> CardDefinition {
    CardDefinition {
        name: "Rush of Knowledge",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::HighestManaValueAmong(Box::new(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: SelectionRequirement::Permanent,
            })),
        },
        ..Default::default()
    }
}

/// Unwilling Ingredient — {B} Creature — Frog. 1/1. "Menace / {2}{B},
/// Exile this card from your graveyard: You draw a card and you lose
/// 1 life."
///
/// The graveyard ability is an `ActivatedAbility` with `from_graveyard`
/// + `exile_self_cost` — activatable only while the card sits in its
/// owner's graveyard, exiling it as part of the cost, then draw 1 /
/// lose 1. (An earlier revision shipped a synthesized "dies → may pay
/// {2}{B} to draw" trigger the printed card never had.)
pub fn unwilling_ingredient() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Unwilling Ingredient",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
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
        effect: Effect::ChooseMode(vec![
            deal(5, target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
            )),
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact),
            },
        ]),
        ..Default::default()
    }
}

// ── Introduction to Prophecy ───────────────────────────────────────────────

/// Introduction to Prophecy — {3} Sorcery — Lesson. "Scry 2, then draw
/// a card."
///
/// Straightforward scry-then-draw Lesson (learnable via `Effect::Learn`).
pub fn introduction_to_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Prophecy",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
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
        ..Default::default()
    }
}

// ── Introduction to Annihilation ───────────────────────────────────────────

/// Introduction to Annihilation — {5} Sorcery — Lesson. "Exile target
/// nonland permanent. Its controller draws a card."
///
/// Colorless Lesson removal spell. The compensation draw resolves after
/// the exile, reading the exiled permanent's controller off the
/// target-slot LKI (same `ControllerOf(Target(0))` pattern as
/// Transforming Flourish's rider).
pub fn introduction_to_annihilation() -> CardDefinition {
    CardDefinition {
        name: "Introduction to Annihilation",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Lesson],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            Effect::Exile {
                what: target_filtered(SelectionRequirement::Nonland),
            },
            // "Its controller draws a card."
            Effect::Draw {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                    Selector::Target(0),
                ))),
                amount: Value::Const(1),
            },
        ]),
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
        ..Default::default()
    }
}


// ── Demonstrate cycle (the STX "Technique" sorceries, CR 702.150) ────────────
// Each fires `shortcut::demonstrate()` — a SpellCast/SelfSource trigger running
// `Effect::Demonstrate`, which copies the spell for its caster and an opponent
// (both copies may choose new targets).

/// Excavation Technique — {3}{W} Sorcery. Demonstrate. Destroy target nonland
/// permanent; its controller creates two Treasure tokens.
pub fn excavation_technique() -> CardDefinition {
    CardDefinition {
        name: "Excavation Technique",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(2),
                definition: crabomination_base::tokens::treasure_token(),
            },
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
        ]),
        triggered_abilities: vec![crate::effect::shortcut::demonstrate()],
        ..Default::default()
    }
}

/// Healing Technique — {3}{G} Sorcery. Demonstrate. Return target card from
/// your graveyard to your hand; gain life equal to its mana value; exile self.
pub fn healing_technique() -> CardDefinition {
    CardDefinition {
        name: "Healing Technique",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ManaValueOf(Box::new(Selector::Target(0))),
            },
            Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        triggered_abilities: vec![crate::effect::shortcut::demonstrate()],
        ..Default::default()
    }
}

/// Replication Technique — {4}{U} Sorcery. Demonstrate. Create a token that's a
/// copy of target permanent you control.
pub fn replication_technique() -> CardDefinition {
    CardDefinition {
        name: "Replication Technique",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateTokenCopyOf {
            extra_keywords: vec![],
            who: PlayerRef::You,
            count: Value::Const(1),
            source: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::ControlledByYou),
            ),
            extra_creature_types: vec![],
            extra_card_types: vec![],
            override_pt: None,
            override_colors: None,
            enters_tapped: false,
            non_legendary: false,
            legendary: false,
        },
        triggered_abilities: vec![crate::effect::shortcut::demonstrate()],
        ..Default::default()
    }
}

/// Incarnation Technique — {4}{B} Sorcery. Demonstrate. Mill five, then return
/// a creature card from your graveyard to the battlefield.
pub fn incarnation_technique() -> CardDefinition {
    CardDefinition {
        name: "Incarnation Technique",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(5) },
            Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::Creature,
                }),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]),
        triggered_abilities: vec![crate::effect::shortcut::demonstrate()],
        ..Default::default()
    }
}

/// Creative Technique — {4}{R} Sorcery. Demonstrate. Shuffle your library, then
/// exile cards from the top until a nonland card; you may cast it for free, the
/// rest go to the bottom. (The reveal-until-nonland + free-cast rides
/// `Effect::Cascade` with no real MV gate, after the shuffle.)
pub fn creative_technique() -> CardDefinition {
    CardDefinition {
        name: "Creative Technique",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ShuffleLibrary { who: PlayerRef::You },
            Effect::Cascade { max_mv: Value::Const(99) },
        ]),
        triggered_abilities: vec![crate::effect::shortcut::demonstrate()],
        ..Default::default()
    }
}

/// Transforming Flourish — {2}{R} Instant. Demonstrate. Destroy target artifact
/// or creature you don't control. "Its controller exiles cards from the top of
/// their library until they exile a nonland card, then may cast it without
/// paying its mana cost."
///
/// The impulse rider is wired: `ExileTopUntilNonlandMayPlay` exiles from
/// the destroyed permanent's controller's library
/// (`PlayerRef::ControllerOf(Target(0))` — resolved off death-time LKI)
/// and `grant_to_exiling_player: true` gives the free cast to THAT
/// player, exactly the printed "its controller ... then may cast it".
pub fn transforming_flourish() -> CardDefinition {
    CardDefinition {
        name: "Transforming Flourish",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
            Effect::ExileTopUntilNonlandMayPlay {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                free: true,
                hand_unless_mv_below: None,
                grant_to_exiling_player: true,
            },
        ]),
        triggered_abilities: vec![crate::effect::shortcut::demonstrate()],
        ..Default::default()
    }
}
