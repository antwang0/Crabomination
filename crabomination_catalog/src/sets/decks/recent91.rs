//! Izzet spellslinger legends & payoffs (batch 2). Rides existing primitives
//! plus two new ones: `Effect::ReturnRandomFromGraveyard` (Charmbreaker Devils)
//! and `SelectionRequirement::SharesNameWithControllerGraveyardCard`
//! (Pyromancer Ascension). Tests in `tests/recent91.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{
    cast_is_instant_or_sorcery, cast_is_noncreature, deal, draw, target_any,
};
use crate::effect::{Duration, ManaPayload, PlayerRef};
use crate::mana::{Color, cost, generic, hybrid, r, u, w};

/// A {U/R} hybrid mana symbol.
fn hybrid_ur() -> crate::mana::ManaSymbol {
    hybrid(Color::Blue, Color::Red)
}

/// 1/1 white Spirit with flying (Kykar).
fn white_spirit_token() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Kykar, Wind's Fury — {1}{U}{R}{W} 3/3 Bird Wizard, flying. Cast a noncreature
/// spell → 1/1 white Spirit flier. Sacrifice a Spirit: add {R}.
pub fn kykar_winds_fury() -> CardDefinition {
    CardDefinition {
        name: "Kykar, Wind's Fury",
        cost: cost(&[generic(1), u(), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_noncreature()),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(white_spirit_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Spirit), 1)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Red, Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Niv-Mizzet, Parun — {U}{U}{U}{R}{R}{R} 5/5 Dragon Wizard, flying, can't be
/// countered. Draw a card → deal 1 to any target. Any player casts an I/S → you
/// draw.
pub fn nivmizzet_parun() -> CardDefinition {
    CardDefinition {
        name: "Niv-Mizzet, Parun",
        cost: cost(&[u(), u(), u(), r(), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::CantBeCountered],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: deal(1, target_any()),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                    .with_filter(cast_is_instant_or_sorcery()),
                effect: draw(1),
            },
        ],
        ..Default::default()
    }
}

/// 1/1 blue-red Insect with flying and haste (The Locust God).
fn insect_token() -> TokenDefinition {
    TokenDefinition {
        name: "Insect".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue, Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..Default::default()
    }
}

/// The Locust God — {4}{U}{R} 4/4 God, flying & haste. Draw a card → 1/1 U/R
/// Insect flier with haste. {2}{U}{R}: draw a card, then discard a card. (The
/// dies → return-to-hand-at-next-end-step recursion clause is dropped.)
pub fn the_locust_god() -> CardDefinition {
    CardDefinition {
        name: "The Locust God",
        cost: cost(&[generic(4), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::God],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(insect_token()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), r()]),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Copy target instant or sorcery spell you control with mana value 2 or less.
/// The copy may choose new targets.
fn copy_low_mv_is_ability(mana: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::CopySpellMayChooseTargets {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::IsSpellOnStack
                    .and(R::ControlledByYou)
                    .and(R::ManaValueAtMost(2))
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            },
            count: Value::Const(1),
        },
        ..Default::default()
    }
}

/// Izzet Guildmage — {U/R}{U/R} 2/2 Human Wizard. {2}{U}: copy a target I/S you
/// control with mv 2 or less; {2}{R}: same. (Both slots accept either type — a
/// harmless broadening of the printed instant-only / sorcery-only split.)
pub fn izzet_guildmage() -> CardDefinition {
    CardDefinition {
        name: "Izzet Guildmage",
        cost: cost(&[hybrid_ur(), hybrid_ur()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![
            copy_low_mv_is_ability(cost(&[generic(2), u()])),
            copy_low_mv_is_ability(cost(&[generic(2), r()])),
        ],
        ..Default::default()
    }
}

/// Veyran, Voice of Duality — {1}{U}{R} 2/2 Efreet Wizard, prowess. Magecraft:
/// cast or copy an I/S → +1/+1 until end of turn. (The trigger-doubling half is
/// dropped — no magecraft-trigger doubler static yet.)
pub fn veyran_voice_of_duality() -> CardDefinition {
    CardDefinition {
        name: "Veyran, Voice of Duality",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Efreet, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Charmbreaker Devils — {5}{R} 4/4 Devil. Upkeep: return an I/S at random from
/// your graveyard to your hand. Cast an I/S → +4/+0 until end of turn.
pub fn charmbreaker_devils() -> CardDefinition {
    CardDefinition {
        name: "Charmbreaker Devils",
        cost: cost(&[generic(5), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::ReturnRandomFromGraveyard {
                    who: PlayerRef::You,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    count: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_instant_or_sorcery()),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(4),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Pyromancer Ascension — {1}{R} Enchantment. Cast an I/S that shares a name
/// with a card in your graveyard → put a quest counter. Cast an I/S while it
/// has 2+ quest counters → copy that spell. (Both "may" clauses are taken
/// automatically.)
pub fn pyromancer_ascension() -> CardDefinition {
    CardDefinition {
        name: "Pyromancer Ascension",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        cast_is_instant_or_sorcery(),
                        Predicate::EntityMatches {
                            what: Selector::TriggerSource,
                            filter: R::SharesNameWithControllerGraveyardCard,
                        },
                    ]),
                ),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Quest,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_instant_or_sorcery()),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Quest,
                        },
                        Value::Const(2),
                    ),
                    then: Box::new(Effect::CopySpellMayChooseTargets {
                        what: Selector::TriggerSource,
                        count: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}
