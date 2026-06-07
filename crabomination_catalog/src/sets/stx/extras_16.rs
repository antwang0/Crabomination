//! Strixhaven base-set (STX) cards — third batch of missing printed cards:
//! the remaining Lessons, X-spells, the spell-copy / spell-counter package,
//! and a spread of payoff creatures. Wired against existing primitives plus
//! `ActivatedAbility.return_self_cost`, `Value::LifeGainedThisTurn`, and
//! `Value::DistinctPowerYouControl`. Each ships with a test in
//! `crate::tests::stx`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, Keyword,
    Predicate, Selector, SelectionRequirement, SpellSubtype, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, gain_life, on_dies, target, target_filtered};
use crate::effect::{Duration, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, x, Color};

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
/// unless its controller pays {3}. If countered this way, exile it instead of
/// putting it into its owner's graveyard.
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
            exile: true,
        },
        ..Default::default()
    }
}

/// Devouring Tendrils — {1}{G} Sorcery. Target creature you control deals
/// damage equal to its power to target creature or planeswalker you don't
/// control. When that permanent dies this turn, you gain 2 life.
pub fn devouring_tendrils() -> CardDefinition {
    CardDefinition {
        name: "Devouring Tendrils",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // Register the death watch before dealing damage so the kill's
            // death event is caught (mirrors Searing Blood's ordering).
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::GainLife {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(2),
                }),
                slot: 1,
            },
            Effect::DealDamage {
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
        ]),
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

/// Geometric Nexus — {2} Artifact. Whenever a player casts an instant or
/// sorcery spell, put charge counters on this equal to that spell's mana value.
/// `{6}, {T}, Remove all charge counters: Create a 0/0 G/U Fractal with X +1/+1
/// counters, where X is the number removed.` (The removal is modeled as part of
/// the resolution rather than a paid cost.)
pub fn geometric_nexus() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    let charge_on_self = || Value::CountersOn {
        what: Box::new(Selector::This),
        kind: CounterType::Charge,
    };
    CardDefinition {
        name: "Geometric Nexus",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Charge,
                amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::catalog::sets::sos::fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: charge_on_self(),
                },
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: charge_on_self(),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Culmination of Studies — {X}{U}{R} Sorcery. Exile the top X cards of your
/// library. For each land exiled this way create a Treasure, for each blue card
/// draw a card, and for each red card deal 1 damage to each opponent.
pub fn culmination_of_studies() -> CardDefinition {
    let last = || Box::new(Selector::LastMoved);
    CardDefinition {
        name: "Culmination of Studies",
        cost: cost(&[x(), u(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileTopOfLibrary { who: Selector::You, amount: Value::XFromCost },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountMatching {
                    sel: last(),
                    filter: SelectionRequirement::HasCardType(CardType::Land),
                },
                definition: crate::game::effects::treasure_token(),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::CountMatching {
                    sel: last(),
                    filter: SelectionRequirement::HasColor(Color::Blue),
                },
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::CountMatching {
                    sel: last(),
                    filter: SelectionRequirement::HasColor(Color::Red),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Semester's End — {3}{W} Instant. Exile target creature or planeswalker you
/// control; at the next end step return it under its owner's control with an
/// extra +1/+1 (creature) or loyalty (planeswalker) counter. (Printed as "any
/// number of targets"; modeled single-target — no variable-target primitive.)
pub fn semesters_end() -> CardDefinition {
    CardDefinition {
        name: "Semester's End",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileReturnNextEndStep {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker)
                    .and(SelectionRequirement::ControlledByYou),
            ),
        },
        ..Default::default()
    }
}

/// Make Your Move — {2}{W} Instant. Destroy target artifact, enchantment, or
/// creature with power 4 or greater.
pub fn make_your_move() -> CardDefinition {
    CardDefinition {
        name: "Make Your Move",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Destroy {
            what: target_filtered(
                SelectionRequirement::Artifact
                    .or(SelectionRequirement::Enchantment)
                    .or(SelectionRequirement::Creature
                        .and(SelectionRequirement::PowerAtLeast(4))),
            ),
        },
        ..Default::default()
    }
}

/// Exponential Growth — {X}{X}{G}{G} Sorcery. Until end of turn, double target
/// creature's power X times.
pub fn exponential_growth() -> CardDefinition {
    CardDefinition {
        name: "Exponential Growth",
        cost: cost(&[x(), x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DoublePower {
            what: target_filtered(SelectionRequirement::Creature),
            times: Value::XFromCost,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Elemental Masterpiece — {5}{U}{R} Sorcery. Create two 4/4 blue-and-red
/// Elemental creature tokens. `{U/R}{U/R}, Discard this card: Create a Treasure
/// token.`
pub fn elemental_masterpiece() -> CardDefinition {
    use crate::card::ActivatedAbility;
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
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[hybrid(Color::Blue, Color::Red), hybrid(Color::Blue, Color::Red)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
            from_hand: true,
            discard_self_cost: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Detention Vortex — {W} Aura. Enchant nonland permanent. The enchanted
/// permanent can't attack or block, and its activated abilities can't be
/// activated (CR 602.5c). `{3}: Destroy this Aura.` — only your opponents may
/// activate this ability and only as a sorcery.
pub fn detention_vortex() -> CardDefinition {
    use crate::card::{ActivatedAbility, EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Detention Vortex",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Permanent.and(SelectionRequirement::Nonland)),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateAbilities,
            ],
            ..Default::default()
        }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Destroy { what: Selector::This },
            sorcery_speed: true,
            opponents_only: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sticky Fingers — {R} Aura. Enchant creature. The enchanted creature has
/// menace and "Whenever this creature deals combat damage to a player, create
/// a Treasure token."
pub fn sticky_fingers() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Sticky Fingers",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(SelectionRequirement::Creature),
        },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Menace],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            }],
            ..Default::default()
        }),
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
/// for a card, put it into your graveyard, then shuffle. If it's an instant
/// or sorcery card, put a +1/+1 counter on this creature.`
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
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Any,
                    to: ZoneDest::Graveyard,
                },
                Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::SelectorExists(Selector::LastMoved),
                        Predicate::EntityMatches {
                            what: Selector::LastMoved,
                            filter: SelectionRequirement::HasCardType(CardType::Instant)
                                .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        },
                    ]),
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
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

// ── More spells ──────────────────────────────────────────────────────────────

/// Deadly Brew — {B}{G} Sorcery. Each player sacrifices a creature or
/// planeswalker of their choice. If you sacrificed a permanent this way, you
/// may return a permanent card from your graveyard to your hand.
pub fn deadly_brew() -> CardDefinition {
    CardDefinition {
        name: "Deadly Brew",
        cost: cost(&[b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
            },
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::MayDo {
                    description: "Return a permanent card from your graveyard to your hand?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::take(
                            Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: SelectionRequirement::Permanent,
                            },
                            Value::Const(1),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Dramatic Finale — {W/B}{W/B}{W/B}{W/B} Enchantment. Creature tokens you
/// control get +1/+1. Whenever one or more nontoken creatures you control die,
/// create a 2/1 white-and-black Inkling with flying. Triggers only once each turn.
pub fn dramatic_finale() -> CardDefinition {
    CardDefinition {
        name: "Dramatic Finale",
        cost: cost(&[hybrid(Color::White, Color::Black); 4]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::IsToken),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::NotToken,
                })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_2_1_token(),
            },
        }],
        ..Default::default()
    }
}

/// Harness Infinity — {1}{B}{B}{B}{G}{G}{G} Instant. Exchange your hand and
/// graveyard, then exile Harness Infinity.
pub fn harness_infinity() -> CardDefinition {
    CardDefinition {
        name: "Harness Infinity",
        cost: cost(&[generic(1), b(), b(), b(), g(), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExchangeHandAndGraveyard { who: PlayerRef::You },
        exile_on_resolve: true,
        ..Default::default()
    }
}

// ── Planeswalker & Land ──────────────────────────────────────────────────────

/// Kasmina, Enigma Sage — {1}{G}{U} 5-loyalty Planeswalker. +2: Scry 1.
/// -X: Create a 0/0 G/U Fractal with X +1/+1 counters (approximated as a fixed
/// -2 → two counters; the engine has no variable-X loyalty path yet). -8:
/// Search your library for a card, put it into your hand, shuffle. (The "other
/// planeswalkers you control gain Kasmina's abilities" static is dropped.)
pub fn kasmina_enigma_sage() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    CardDefinition {
        name: "Kasmina, Enigma Sage",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Kasmina],
            ..Default::default()
        },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: crate::catalog::sets::sos::fractal_token(),
                    },
                    Effect::AddCounter {
                        what: Selector::LastCreatedToken,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(2),
                    },
                ]),
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Any,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// The Biblioplex — Land. `{T}: Add {C}.` `{2}, {T}: Look at the top card of
/// your library; if it's an instant or sorcery card, you may put it into your
/// hand. Otherwise it goes to the bottom.`
pub fn the_biblioplex() -> CardDefinition {
    CardDefinition {
        name: "The Biblioplex",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    rest_to_graveyard: false,
                    pick_filter: Some(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                    take: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}