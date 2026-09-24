//! Commander: the cards the **Jeskai Striker** precon (TDC, Shiko and Narset,
//! Unified) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{flurry, target_filtered};
use crate::effect::{CounteredSpellZone, Duration, Effect, PlayerRef, Predicate};
use crate::mana::{Color, cost, generic, hybrid, r, u, w, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// "Whenever you cast an instant or sorcery spell."
fn on_instant_or_sorcery(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
            Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            },
        ),
        effect,
    }
}

/// A 1/1 white Monk with prowess.
fn monk() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Monk".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Monk], ..Default::default() },
        keywords: vec![Keyword::Prowess],
        ..Default::default()
    })
}

/// Caldera Pyremaw — grows with each instant or sorcery, then hits an
/// opponent for its power.
pub fn caldera_pyremaw() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_instant_or_sorcery(Effect::Seq(vec![
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::DealDamage {
                to: target_filtered(R::OpponentPlayer),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
        ]))],
        ..creature("Caldera Pyremaw", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Elsha, Threefold Master — a prowess trampler that makes a Monk per point
/// of combat damage to a player.
pub fn elsha_threefold_master() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample, Keyword::Prowess],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                definition: monk(),
            },
        }],
        ..creature(
            "Elsha, Threefold Master",
            cost(&[u(), r(), w()]),
            vec![CreatureType::Djinn, CreatureType::Monk],
            1,
            1,
        )
    }
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn enchantment(name: &'static str, mana: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// Adaptive Training Post — banks up to three charge counters off your
/// instants and sorceries; spend three to copy the next one this turn.
pub fn adaptive_training_post() -> CardDefinition {
    CardDefinition {
        name: "Adaptive Training Post",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            // CR 603.4 — an intervening "if": below three counters.
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: instant_or_sorcery() },
                    Predicate::ValueAtMost(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge },
                        Value::Const(2),
                    ),
                ]),
            ),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Charge, 3)),
            effect: Effect::OnYourNextInstantSorceryThisTurn {
                body: Box::new(Effect::CopySpellMayChooseTargets {
                    what: Selector::TriggerSource,
                    count: Value::ONE,
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aligned Heart — flurry: a rally counter, then a prowess Monk per counter.
pub fn aligned_heart() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![flurry(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::Rally, amount: Value::ONE },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Rally },
                definition: monk(),
            },
        ]))],
        ..enchantment("Aligned Heart", cost(&[generic(2), w()]))
    }
}

/// Baral and Kari Zev — the first instant or sorcery each turn offers a free
/// cheaper spell of the same type from hand, or First Mate Ragavan.
pub fn baral_and_kari_zev() -> CardDefinition {
    let ragavan = Arc::new(TokenDefinition {
        name: "First Mate Ragavan".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Monkey, CreatureType::Pirate],
            ..Default::default()
        },
        ..Default::default()
    });
    let or_ragavan = || {
        Box::new(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::clone(&ragavan) },
            Effect::GrantKeyword {
                what: Selector::LastCreatedToken,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))
    };
    // "Lesser mana value" and "shares a card type": the trigger's event
    // amount is the cast spell's mana value.
    let lesser = || Value::Sum(vec![Value::TriggerEventAmount, Value::Const(-1)]);
    let free = |t: CardType| Effect::MayCastFromHandFreeMatching {
        filter: R::HasCardType(t),
        max_mv: lesser(),
        else_: or_ragavan(),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike, Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellFirstMatchingThisTurn(instant_or_sorcery())),
            effect: Effect::If {
                cond: Predicate::CastSpellMatches(R::HasCardType(CardType::Instant)),
                then: Box::new(free(CardType::Instant)),
                else_: Box::new(free(CardType::Sorcery)),
            },
        }],
        ..creature(
            "Baral and Kari Zev",
            cost(&[generic(1), u(), r()]),
            vec![CreatureType::Human],
            2,
            4,
        )
    }
}

/// Baral's Expertise — bounce up to three artifacts/creatures, then a free
/// spell of mana value 4 or less.
pub fn barals_expertise() -> CardDefinition {
    CardDefinition {
        name: "Baral's Expertise",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::Artifact.or(R::Creature),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: crate::effect::ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
            },
            Effect::CastFromHandWithoutPaying { filter: Some(R::ManaValueAtMost(4)) },
        ]),
        ..Default::default()
    }
}

/// Curse of Opulence — whenever the cursed player is attacked, you get a Gold,
/// and so does the attacker if they're your opponent.
pub fn curse_of_opulence() -> CardDefinition {
    let gold = || Arc::new(crate::sets::thb::gold_token());
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedDefenderWithCountAtLeast {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::EnchantedPlayer,
                    at_least: 1,
                    include_planeswalkers: false,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: gold() },
                Effect::If {
                    cond: Predicate::PlayerIsOpponent { who: PlayerRef::ActivePlayer },
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::ActivePlayer,
                        count: Value::ONE,
                        definition: gold(),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..enchantment("Curse of Opulence", cost(&[r()]))
    }
}

/// Dismantling Wave — an artifact or enchantment from each opponent; cycle it
/// for a full sweep.
pub fn dismantling_wave() -> CardDefinition {
    let any_ae = || R::Artifact.or(R::Enchantment);
    CardDefinition {
        name: "Dismantling Wave",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(6), w(), w()]))],
        effect: Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: any_ae().and(R::ControlledByOpponent),
                effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
            }),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
            effect: Effect::Destroy { what: Selector::EachPermanent(any_ae()) },
        }],
        ..Default::default()
    }
}

/// Expansion // Explosion — copy a cheap instant or sorcery, or X damage and
/// X cards.
pub fn expansion_explosion() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    CardDefinition {
        name: "Expansion // Explosion",
        cost: cost(&[ur(), ur()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CopySpellMayChooseTargets {
            what: target_filtered(R::IsSpellOnStack.and(instant_or_sorcery()).and(R::ManaValueAtMost(4))),
            count: Value::ONE,
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[x(), u(), u(), r(), r()]),
                card_types: vec![CardType::Instant],
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                        amount: Value::XFromCost,
                    },
                    Effect::Draw {
                        who: Selector::TargetFiltered { slot: 1, filter: R::Player },
                        amount: Value::XFromCost,
                    },
                ]),
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Haughty Djinn — as strong as your graveyard's instants and sorceries, and
/// they cost {1} less.
pub fn haughty_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::InstantsSorceriesInControllerGraveyard { base_t: 4 }),
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: instant_or_sorcery(), amount: 1 },
        }],
        ..creature("Haughty Djinn", cost(&[generic(1), u(), u()]), vec![CreatureType::Djinn], 0, 4)
    }
}

/// Shiny Impetus — +2/+2, goaded, and a Treasure for you each time it
/// attacks.
pub fn shiny_impetus() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            triggers_on_equipment: true,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::ControllerOf(Box::new(Selector::This)),
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            }],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        ..enchantment("Shiny Impetus", cost(&[generic(2), r()]))
    }
}

/// Tempest Technique — storm, and +1/+1 per enchantment you control. ⚠ The
/// storm copies keep the original's target (choosing the same creature again
/// is one of the legal choices).
pub fn tempest_technique() -> CardDefinition {
    let n = || Value::CountOf(Box::new(Selector::EachPermanent(R::Enchantment.and(R::ControlledByYou))));
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Storm],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +1/+1 for each enchantment you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: n(),
                toughness: n(),
            },
        }],
        ..enchantment("Tempest Technique", cost(&[generic(3), w()]))
    }
}

/// Transcendent Dragon — flash; cast, it counters a spell into exile and you
/// may cast that spell free.
pub fn transcendent_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SourceWasCast),
            effect: Effect::Seq(vec![
                Effect::CounterSpellToZone {
                    what: target_filtered(R::IsSpellOnStack),
                    zone: CounteredSpellZone::ExileWithSource,
                },
                Effect::CastWithoutPayingImmediate {
                    what: Selector::CardExiledWithSource,
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                },
            ]),
        }],
        ..creature("Transcendent Dragon", cost(&[generic(4), u(), u()]), vec![CreatureType::Dragon], 4, 3)
    }
}

/// Voracious Bibliophile — a card per target of each spell you cast.
pub fn voracious_bibliophile() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::AllCastSpellTargets)),
            },
        }],
        ..creature(
            "Voracious Bibliophile",
            cost(&[generic(3), u()]),
            vec![CreatureType::Dragon],
            3,
            3,
        )
    }
}

/// Will of the Jeskai — a table-wide optional wheel for five, or flashback
/// for your graveyard's instants and sorceries; both with a commander.
pub fn will_of_the_jeskai() -> CardDefinition {
    let modes = || {
        vec![
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::MayDo {
                    description: "Discard your hand and draw five cards?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard {
                            who: Selector::You,
                            amount: Value::HandSizeOf(PlayerRef::You),
                            random: false,
                        },
                        Effect::Draw { who: Selector::You, amount: Value::Const(5) },
                    ])),
                }),
            },
            Effect::GrantFlashbackThisTurn {
                what: Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: instant_or_sorcery(),
                },
            },
        ]
    };
    CardDefinition {
        name: "Will of the Jeskai",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
        ..Default::default()
    }
}
