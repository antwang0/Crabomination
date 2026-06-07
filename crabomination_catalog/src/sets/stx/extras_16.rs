//! Strixhaven base-set (STX) cards — third batch of missing printed cards:
//! the remaining Lessons, X-spells, the spell-copy / spell-counter package,
//! and a spread of payoff creatures. Wired against existing primitives plus
//! `ActivatedAbility.return_self_cost`, `Value::LifeGainedThisTurn`, and
//! `Value::DistinctPowerYouControl`. Each ships with a test in
//! `crate::tests::stx`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    Predicate, Selector, SelectionRequirement, SpellSubtype, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, gain_life, on_dies, target, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

// ── Lessons ──────────────────────────────────────────────────────────────────

/// Basic Conjuration — {1}{G}{G} Sorcery — Lesson. Look at the top six
/// cards of your library, put a creature card from among them into your
/// hand, the rest on the bottom in a random order, and gain 3 life.
pub fn basic_conjuration() -> CardDefinition {
    CardDefinition {
        name: "Basic Conjuration",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(6),
                rest_to_graveyard: false,
                pick_filter: Some(SelectionRequirement::Creature),
                take: None,
            },
            gain_life(3),
        ]),
        ..Default::default()
    }
}

/// Start from Scratch — {2}{R} Sorcery — Lesson. Choose one — 1 damage to
/// any target; or destroy target artifact.
pub fn start_from_scratch() -> CardDefinition {
    CardDefinition {
        name: "Start from Scratch",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage { to: target(), amount: Value::Const(1) },
            Effect::Destroy { what: target_filtered(SelectionRequirement::Artifact) },
        ]),
        ..Default::default()
    }
}

/// Teachings of the Archaics — {2}{U} Sorcery — Lesson. If an opponent has
/// more cards in hand than you, draw two cards. Draw three instead if an
/// opponent has at least four more cards in hand than you.
pub fn teachings_of_the_archaics() -> CardDefinition {
    let opp = || Value::HandSizeOf(PlayerRef::EachOpponent);
    let you = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        name: "Teachings of the Archaics",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        effect: Effect::If {
            // opp ≥ you + 4 → draw 3
            cond: Predicate::ValueAtLeast(opp(), Value::Sum(vec![you(), Value::Const(4)])),
            then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
            else_: Box::new(Effect::If {
                // opp ≥ you + 1 → draw 2
                cond: Predicate::ValueAtLeast(opp(), Value::Sum(vec![you(), Value::Const(1)])),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
                else_: Box::new(Effect::Noop),
            }),
        },
        ..Default::default()
    }
}

// ── X-spells ────────────────────────────────────────────────────────────────

/// 2/1 white-and-black Inkling token with flying (the Blot Out the Sky /
/// Mascot Exhibition variant — distinct from the 1/1 `inkling_token`).
fn inkling_2_1_token() -> TokenDefinition {
    TokenDefinition {
        name: "Inkling".into(),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Inkling], ..Default::default() },
        ..Default::default()
    }
}

/// Blot Out the Sky — {X}{W}{B} Sorcery. Create X tapped 2/1 white-and-black
/// Inkling tokens with flying. If X is 6 or more, destroy all noncreature,
/// nonland permanents.
pub fn blot_out_the_sky() -> CardDefinition {
    CardDefinition {
        name: "Blot Out the Sky",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: inkling_2_1_token(),
            },
            Effect::Tap { what: Selector::LastCreatedTokens },
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::XFromCost, Value::Const(6)),
                then: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Noncreature.and(SelectionRequirement::Nonland),
                    ),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Serpentine Curve — {3}{U} Sorcery. Create a 0/0 green-and-blue Fractal
/// creature token, then put X +1/+1 counters on it, where X is one plus the
/// number of instant and sorcery cards you own in exile and in your graveyard.
pub fn serpentine_curve() -> CardDefinition {
    let is_filter = || {
        SelectionRequirement::HasCardType(CardType::Instant)
            .or(SelectionRequirement::HasCardType(CardType::Sorcery))
    };
    CardDefinition {
        name: "Serpentine Curve",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::catalog::sets::sos::fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Sum(vec![
                    Value::Const(1),
                    Value::CountOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Graveyard,
                        filter: is_filter(),
                    })),
                    Value::CountOf(Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Exile,
                        filter: is_filter(),
                    })),
                ]),
            },
        ]),
        ..Default::default()
    }
}

// ── Spells ────────────────────────────────────────────────────────────────

/// Flunk — {1}{B} Instant. Target creature gets -X/-X until end of turn,
/// where X is 7 minus the number of cards in that creature's controller's
/// hand.
pub fn flunk() -> CardDefinition {
    let neg_x = || {
        // -X where X = 7 - hand → -X = hand - 7
        Value::Diff(
            Box::new(Value::HandSizeOf(PlayerRef::ControllerOf(Box::new(Selector::Target(0))))),
            Box::new(Value::Const(7)),
        )
    };
    CardDefinition {
        name: "Flunk",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: neg_x(),
            toughness: neg_x(),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Double Major — {G}{U} Instant. Copy target creature spell you control.
/// (A copy of a creature spell becomes a token.)
pub fn double_major() -> CardDefinition {
    CardDefinition {
        name: "Double Major",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CopySpell {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(SelectionRequirement::HasCardType(CardType::Creature)),
            ),
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

/// Reject — {1}{U} Instant. Counter target creature or planeswalker spell
/// unless its controller pays {3}. (The exile-instead-of-graveyard rider on
/// a successful counter is dropped — the countered spell goes to its owner's
/// graveyard.)
pub fn reject() -> CardDefinition {
    CardDefinition {
        name: "Reject",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack.and(
                SelectionRequirement::HasCardType(CardType::Creature)
                    .or(SelectionRequirement::HasCardType(CardType::Planeswalker)),
            )),
            mana_cost: cost(&[generic(3)]),
        },
        ..Default::default()
    }
}

/// Devouring Tendrils — {1}{G} Sorcery. Target creature you control deals
/// damage equal to its power to target creature or planeswalker you don't
/// control. (The "gain 2 when it dies this turn" rider is dropped.)
pub fn devouring_tendrils() -> CardDefinition {
    CardDefinition {
        name: "Devouring Tendrils",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            })),
        },
        ..Default::default()
    }
}

/// Study Break — {1}{W} Instant. Tap up to two target creatures, then Learn.
pub fn study_break() -> CardDefinition {
    CardDefinition {
        name: "Study Break",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            Effect::Tap {
                what: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
            },
            Effect::Learn { who: PlayerRef::You },
        ]),
        ..Default::default()
    }
}

/// Golden Ratio — {1}{G}{U} Sorcery. Draw a card for each different power
/// among creatures you control.
pub fn golden_ratio() -> CardDefinition {
    CardDefinition {
        name: "Golden Ratio",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw { who: Selector::You, amount: Value::DistinctPowerYouControl },
        ..Default::default()
    }
}

/// Elemental Masterpiece — {5}{U}{R} Sorcery. Create two 4/4 blue-and-red
/// Elemental creature tokens. (The discard-this-from-hand-for-a-Treasure
/// activated ability is dropped — no from-hand discard-cost mana rider yet.)
pub fn elemental_masterpiece() -> CardDefinition {
    let elemental_4_4 = TokenDefinition {
        name: "Elemental".into(),
        power: 4,
        toughness: 4,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Elemental Masterpiece",
        cost: cost(&[generic(5), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: elemental_4_4,
        },
        ..Default::default()
    }
}

// ── Creatures ────────────────────────────────────────────────────────────────

/// Gnarled Professor — {2}{G}{G} 5/4 Treefolk Druid with trample. ETB: Learn.
pub fn gnarled_professor() -> CardDefinition {
    CardDefinition {
        name: "Gnarled Professor",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Learn { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Dream Strix — {2}{U} 3/2 Bird Illusion with flying. When it becomes the
/// target of a spell, sacrifice it. When it dies, Learn.
pub fn dream_strix() -> CardDefinition {
    CardDefinition {
        name: "Dream Strix",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Illusion],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Graveyard },
            },
            on_dies(Effect::Learn { who: PlayerRef::You }),
        ],
        ..Default::default()
    }
}

/// Retriever Phoenix — {3}{R} 2/2 Phoenix with flying and haste. ETB, if you
/// cast it, Learn. (The graveyard "learn → return this instead" recursion
/// replacement is dropped.)
pub fn retriever_phoenix() -> CardDefinition {
    CardDefinition {
        name: "Retriever Phoenix",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phoenix], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // ETB Learn (the "if you cast it" gate collapses — token/blink
            // recursion isn't a path for this card).
            effect: Effect::Learn { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// Accomplished Alchemist — {3}{G} 2/5 Elf Druid. `{T}: Add one mana of any
/// color.` `{T}: Add X mana of any one color, where X is the amount of life
/// you gained this turn.`
pub fn accomplished_alchemist() -> CardDefinition {
    CardDefinition {
        name: "Accomplished Alchemist",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
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
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::LifeGainedThisTurn(PlayerRef::You)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oriq Loremage — {2}{B}{B} 3/3 Human Warlock. `{T}: Search your library
/// for a card, put it into your graveyard, then shuffle.` (The "if it's an
/// instant or sorcery, put a +1/+1 counter on this" rider is dropped — the
/// search result type isn't surfaced back to the activation.)
pub fn oriq_loremage() -> CardDefinition {
    CardDefinition {
        name: "Oriq Loremage",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Graveyard,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 3/2 red-and-white Spirit creature token (Illustrious Historian's mint —
/// no flying, distinct from `lorehold_spirit_token`).
fn spirit_3_2_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red, Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    }
}

/// Illustrious Historian — {1}{R} 2/1 Human Shaman. `{5}, Exile this card
/// from your graveyard: Create a tapped 3/2 red-and-white Spirit creature
/// token.`
pub fn illustrious_historian() -> CardDefinition {
    CardDefinition {
        name: "Illustrious Historian",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: spirit_3_2_token(),
                },
                Effect::Tap { what: Selector::LastCreatedToken },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Grinning Ignus — {2}{R} 2/2 Elemental. `{R}, Return this creature to its
/// owner's hand: Add {C}{C}{R}. Activate only as a sorcery.`
pub fn grinning_ignus() -> CardDefinition {
    CardDefinition {
        name: "Grinning Ignus",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            return_self_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rootha, Mercurial Artist — {1}{U}{R} 1/4 legendary Orc Shaman. `{2},
/// Return Rootha to its owner's hand: Copy target instant or sorcery spell
/// you control. You may choose new targets for the copy.`
pub fn rootha_mercurial_artist() -> CardDefinition {
    CardDefinition {
        name: "Rootha, Mercurial Artist",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            return_self_cost: true,
            effect: Effect::CopySpellMayChooseTargets {
                what: target_filtered(SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                )),
                count: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}