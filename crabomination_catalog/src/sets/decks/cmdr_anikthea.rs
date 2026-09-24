//! Commander: the cards the **Enduring Enchantments** precon (CMM, Anikthea,
//! Hand of Erebos) needed beyond what the catalog had (Greater Tanuki is
//! Raining Cats and Dogs', `cmdr_rinseri.rs`). Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Battle at the Helvault** — chapters I and II target one permanent per
//!   *opponent*; your own permanents can't be picked.
//! - **Battle for Bretagard** — chapter III copies every artifact and creature
//!   token you control, duplicate names included.
//! - **Cacophony Unleashed** — the animated form isn't legendary.
//! - **Ghoulish Impetus** — the goad is re-applied each of your upkeeps rather
//!   than held by a static, so it outlives the Aura until your next turn.
//! - **Ondu Spiritdancer** — declining the copy still spends the turn's use.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, ExileReturnZone, Keyword, LandType, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mint_token, on_attack, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
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

fn enchantment_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Enchantment, CardType::Creature],
        ..creature(name, mana, types, p, t)
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            ..Default::default()
        },
        saga_chapters: chapters,
        ..enchantment(name, mana)
    }
}

fn token(
    name: &str,
    color: Color,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    keywords: Vec<Keyword>,
) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

fn copy_of(source: Selector) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types: vec![],
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
    }
}

/// "Whenever this or another enchantment you control enters, `body`."
fn constellation(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
        ),
        effect: body,
    }
}

fn search_to_battlefield_tapped(filter: R) -> Effect {
    Effect::Search {
        who: PlayerRef::You,
        filter,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
    }
}

fn put_from_hand_tapped(filter: R) -> Effect {
    Effect::PutFromHandOntoBattlefield {
        who: PlayerRef::You,
        filter,
        count: Value::ONE,
        tapped: true,
        haste: false,
        sacrifice_eot: false,
        return_eot: false,
        then: None,
    }
}

/// Anikthea, Hand of Erebos — menace for itself and your other enchantment
/// creatures; on entry or attack, exile up to one non-Aura enchantment card
/// from your graveyard and make a 3/3 black Zombie creature token copy of it.
pub fn anikthea_hand_of_erebos() -> CardDefinition {
    let reanimate = Effect::OptionalTargets {
        min: 0,
        body: Box::new(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::Enchantment
                        .and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura))))
                        .and(R::InYourGraveyard),
                ),
                to: ZoneDest::Exile,
            },
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::Target(0),
                extra_creature_types: vec![CreatureType::Zombie],
                extra_card_types: vec![CardType::Creature],
                override_pt: Some((3, 3)),
                override_colors: Some(vec![Color::Black]),
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
        ])),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Other enchantment creatures you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::Enchantment).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![etb(reanimate.clone()), on_attack(reanimate)],
        ..enchantment_creature(
            "Anikthea, Hand of Erebos",
            cost(&[generic(2), w(), b(), g()]),
            vec![CreatureType::Demigod],
            4,
            4,
        )
    }
}

/// Battle at the Helvault — I, II: exile up to one non-Saga nonland permanent
/// each opponent controls until this Saga leaves. III: Avacyn, a legendary
/// 8/8 flying, vigilant, indestructible Angel.
///
/// Approximation: "for each player" covers opponents only.
pub fn battle_at_the_helvault() -> CardDefinition {
    let exile = || Effect::ForEachOpponentTarget {
        body: Box::new(Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::Permanent
                .and(R::Nonland)
                .and(R::ControlledByOpponent)
                .and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga)))),
            effect: Box::new(Effect::ExileUntilSourceLeaves {
                what: Selector::Target(0),
                return_to: ExileReturnZone::Battlefield,
            }),
        }),
    };
    let avacyn = TokenDefinition {
        supertypes: vec![Supertype::Legendary],
        ..token(
            "Avacyn",
            Color::White,
            vec![CreatureType::Angel],
            8,
            8,
            vec![Keyword::Flying, Keyword::Vigilance, Keyword::Indestructible],
        )
    };
    saga(
        "Battle at the Helvault",
        cost(&[generic(4), w(), w()]),
        vec![(1, exile()), (2, exile()), (3, mint_token(avacyn, 1))],
    )
}

/// Battle for Bretagard — I: a 1/1 white Human Warrior. II: a 1/1 green Elf
/// Warrior. III: copy artifact and creature tokens you control.
///
/// Approximation: chapter III copies every such token, duplicate names too.
pub fn battle_for_bretagard() -> CardDefinition {
    saga(
        "Battle for Bretagard",
        cost(&[generic(1), g(), w()]),
        vec![
            (
                1,
                mint_token(
                    token(
                        "Human Warrior",
                        Color::White,
                        vec![CreatureType::Human, CreatureType::Warrior],
                        1,
                        1,
                        vec![],
                    ),
                    1,
                ),
            ),
            (
                2,
                mint_token(
                    token(
                        "Elf Warrior",
                        Color::Green,
                        vec![CreatureType::Elf, CreatureType::Warrior],
                        1,
                        1,
                        vec![],
                    ),
                    1,
                ),
            ),
            (
                3,
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        R::IsToken.and(R::ControlledByYou).and(R::Artifact.or(R::Creature)),
                    ),
                    body: Box::new(copy_of(Selector::TriggerSource)),
                },
            ),
        ],
    )
}

/// Binding the Old Gods — I: destroy a nonland permanent an opponent controls.
/// II: a Forest card onto the battlefield tapped. III: your creatures gain
/// deathtouch until end of turn.
pub fn binding_the_old_gods() -> CardDefinition {
    saga(
        "Binding the Old Gods",
        cost(&[generic(2), b(), g()]),
        vec![
            (
                1,
                Effect::Destroy {
                    what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
                },
            ),
            (2, search_to_battlefield_tapped(R::HasLandType(LandType::Forest))),
            (
                3,
                Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    keyword: Keyword::Deathtouch,
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
    )
}

/// Boon of the Spirit Realm — constellation: a blessing counter; your
/// creatures get +1/+1 per blessing counter.
pub fn boon_of_the_spirit_realm() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1 for each blessing counter on this.",
            effect: StaticEffect::PumpPTPerCounterOnSource {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::Blessing,
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![constellation(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Blessing,
            amount: Value::ONE,
        })],
        ..enchantment("Boon of the Spirit Realm", cost(&[generic(3), w(), w()]))
    }
}

/// Cacophony Unleashed — cast: destroy all nonenchantment creatures.
/// Constellation: it becomes a 6/6 Nightmare God with menace and deathtouch
/// until end of turn.
///
/// Approximation: the animated form isn't legendary.
pub fn cacophony_unleashed() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::If {
                cond: Predicate::SourceWasCast,
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        R::Creature.and(R::Not(Box::new(R::Enchantment))),
                    ),
                    body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
                }),
                else_: Box::new(Effect::Noop),
            }),
            constellation(Effect::BecomeCreature {
                what: Selector::This,
                power: Value::Const(6),
                toughness: Value::Const(6),
                creature_types: vec![CreatureType::Nightmare, CreatureType::God],
                keywords: vec![Keyword::Menace, Keyword::Deathtouch],
                duration: Duration::EndOfTurn,
            }),
        ],
        ..enchantment("Cacophony Unleashed", cost(&[generic(5), b(), b()]))
    }
}

/// Composer of Spring — constellation: put a land card (a creature or land
/// card with six or more enchantments) from your hand onto the battlefield
/// tapped.
pub fn composer_of_spring() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountOf(Box::new(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::Enchantment,
                    })),
                    Value::Const(6),
                ),
                then: Box::new(put_from_hand_tapped(R::Creature.or(R::Land))),
                else_: Box::new(put_from_hand_tapped(R::Land)),
            },
        }],
        ..creature(
            "Composer of Spring",
            cost(&[generic(1), g()]),
            vec![CreatureType::Satyr, CreatureType::Bard],
            1,
            3,
        )
    }
}

/// Demon of Fate's Design — flying, trample; once each of your turns cast an
/// enchantment spell for life equal to its mana value; {2}{B}, sacrifice
/// another enchantment: +X/+0, X its mana value.
pub fn demon_of_fates_design() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast an enchantment spell by paying life equal to its mana value.",
            effect: StaticEffect::LifeAlternativeCostOncePerYourTurn { filter: R::Enchantment },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Enchantment, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::SacrificedManaValue,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment_creature(
            "Demon of Fate's Design",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Demon],
            6,
            6,
        )
    }
}

/// Ghoulish Impetus — enchanted creature gets +1/+1, has deathtouch and is
/// goaded; when it dies, this returns at the next end step.
pub fn ghoulish_impetus() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Deathtouch],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::ReturnSelfAttachedToChoiceOf {
                        chooser: PlayerRef::You,
                        filter: R::Creature,
                    }),
                },
            },
        ],
        ..enchantment("Ghoulish Impetus", cost(&[generic(2), b()]))
    }
}

/// Love Song of Night and Day — read ahead. I: you and target opponent each
/// draw two. II: a 1/1 flying Bird. III: a +1/+1 counter on each of up to two
/// creatures.
pub fn love_song_of_night_and_day() -> CardDefinition {
    CardDefinition {
        read_ahead: true,
        ..saga(
            "Love Song of Night and Day",
            cost(&[generic(2), w()]),
            vec![
                (
                    1,
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                        Effect::Draw {
                            who: target_filtered(R::OpponentPlayer),
                            amount: Value::Const(2),
                        },
                    ]),
                ),
                (
                    2,
                    mint_token(
                        token("Bird", Color::White, vec![CreatureType::Bird], 1, 1, vec![
                            Keyword::Flying,
                        ]),
                        1,
                    ),
                ),
                (
                    3,
                    Effect::ApplyToTargets {
                        max_targets: 2,
                        min_targets: 0,
                        filter: R::Creature,
                        effect: Box::new(Effect::AddCounter {
                            what: Selector::Target(0),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        }),
                    },
                ),
            ],
        )
    }
}

/// Narci, Fable Singer — lifelink; sacrificing an enchantment draws a card; a
/// Saga's final chapter drains each opponent for its mana value.
pub fn narci_fable_singer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Whenever the final chapter ability of a Saga you control resolves, each opponent loses X life and you gain X life, where X is that Saga's mana value.",
            effect: StaticEffect::SagaFinalChapterRider(Box::new(Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ManaValueOf(Box::new(Selector::This)),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(Selector::This)),
                },
            ]))),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Narci, Fable Singer",
            cost(&[generic(1), w(), b(), g()]),
            vec![CreatureType::Human, CreatureType::Bard],
            3,
            3,
        )
    }
}

/// Nyxborn Behemoth — costs {X} less (X: total mana value of your noncreature
/// enchantments); trample; {1}{G}, sacrifice another enchantment:
/// indestructible until end of turn.
pub fn nyxborn_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the total mana value of noncreature enchantments you control.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::TotalManaValueOf(Box::new(Selector::ControlledBy {
                    who: PlayerRef::You,
                    filter: R::Enchantment.and(R::Not(Box::new(R::Creature))),
                })),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_other_filter: Some((R::Enchantment, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment_creature(
            "Nyxborn Behemoth",
            cost(&[generic(10), g(), g()]),
            vec![CreatureType::Beast],
            10,
            10,
        )
    }
}

/// Ondu Spiritdancer — once each turn, when an enchantment you control
/// enters, you may copy it.
///
/// Approximation: declining the copy still spends the turn's use.
pub fn ondu_spiritdancer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Enchantment,
                })
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Create a token copy of the entering enchantment?".into(),
                body: Box::new(copy_of(Selector::TriggerSource)),
            },
        }],
        ..creature(
            "Ondu Spiritdancer",
            cost(&[generic(4), w()]),
            vec![CreatureType::Kor, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Sandwurm Convergence — fliers can't attack you or your planeswalkers; a
/// 5/5 Wurm at your end step.
pub fn sandwurm_convergence() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures with flying can't attack you or planeswalkers you control.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: true,
                filter: Some(R::HasKeyword(Keyword::Flying)),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: mint_token(token("Wurm", Color::Green, vec![CreatureType::Wurm], 5, 5, vec![]), 1),
        }],
        ..enchantment("Sandwurm Convergence", cost(&[generic(6), g(), g()]))
    }
}

/// Satyr Enchanter — whenever you cast an enchantment spell, draw a card.
pub fn satyr_enchanter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Satyr Enchanter",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Satyr, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Starfield Mystic — your enchantment spells cost {1} less; a +1/+1 counter
/// whenever an enchantment you control goes to the graveyard from play.
pub fn starfield_mystic() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchantment spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Enchantment, amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Enchantment },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Starfield Mystic",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            2,
        )
    }
}
