//! Kamigawa: Neon Dynasty batch 3 — Ninjutsu with keyword-counter payoffs,
//! Channel spells, modified-matters and enchantment/artifact synergies. Rides
//! existing primitives. Tests in `tests/recent97.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate,
    SelectionRequirement as R, Selector, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Kappa Tech-Wrecker — {1}{G} 1/3 Turtle Ninja. Ninjutsu {1}{G}. Enters with a
/// deathtouch counter. Combat damage: may remove it to exile target artifact or
/// enchantment an opponent controls.
pub fn kappa_tech_wrecker() -> CardDefinition {
    CardDefinition {
        name: "Kappa Tech-Wrecker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle, CreatureType::Ninja],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), g()]))],
        triggered_abilities: vec![
            etb(Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Deathtouch,
                amount: Value::Const(1),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Remove a deathtouch counter to exile an artifact or enchantment?"
                        .into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveKeywordCounter {
                            what: Selector::This,
                            keyword: Keyword::Deathtouch,
                            amount: Value::Const(1),
                        },
                        Effect::Exile {
                            what: target_filtered(
                                R::Artifact.or(R::Enchantment).and(R::ControlledByOpponent),
                            ),
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Biting-Palm Ninja — {2}{B} 3/3 Human Ninja. Ninjutsu {2}{B}. Enters with a
/// menace counter. Combat damage: may remove it, then that opponent reveals
/// their hand and you exile a nonland card.
pub fn biting_palm_ninja() -> CardDefinition {
    CardDefinition {
        name: "Biting-Palm Ninja",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(2), b()]))],
        triggered_abilities: vec![
            etb(Effect::AddKeywordCounter {
                what: Selector::This,
                keyword: Keyword::Menace,
                amount: Value::Const(1),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Remove a menace counter to strip a card from their hand?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::RemoveKeywordCounter {
                            what: Selector::This,
                            keyword: Keyword::Menace,
                            amount: Value::Const(1),
                        },
                        Effect::ExileChosenFromHand {
                            from: Selector::Player(PlayerRef::EachOpponent),
                            count: Value::Const(1),
                            filter: R::Nonland,
                            link_to_source: false,
                            face_down: false,
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Dokuchi Silencer — {1}{B} 2/1 Human Ninja. Ninjutsu {1}{B}. Combat damage:
/// may discard a creature card to destroy target creature or planeswalker an
/// opponent controls.
pub fn dokuchi_silencer() -> CardDefinition {
    CardDefinition {
        name: "Dokuchi Silencer",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDiscard {
                description: "Discard a creature card to destroy a creature or planeswalker?"
                    .into(),
                count: Value::Const(1),
                then: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Destroy {
                        what: target_filtered(
                            R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                        ),
                    }),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Kami of Restless Shadows — {4}{B} 3/3 Spirit. ETB choose one: return up to
/// one Ninja or Rogue creature card from your graveyard to hand; or put target
/// creature card from your graveyard on top of your library.
pub fn kami_of_restless_shadows() -> CardDefinition {
    CardDefinition {
        name: "Kami of Restless Shadows",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::InYourGraveyard.and(R::Creature).and(
                    R::HasCreatureType(CreatureType::Ninja)
                        .or(R::HasCreatureType(CreatureType::Rogue)),
                ),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::Creature)),
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: LibraryPosition::Top,
                },
            },
        ]))],
        ..Default::default()
    }
}

/// Moonsnare Prototype — {U} Artifact. {T}, Tap an untapped artifact or creature
/// you control: Add {C}. Channel — {4}{U}, Discard this card: the owner of
/// target nonland permanent puts it on their choice of the top or bottom of
/// their library.
pub fn moonsnare_prototype() -> CardDefinition {
    CardDefinition {
        name: "Moonsnare Prototype",
        cost: cost(&[u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                tap_other_filter: Some(R::Artifact.or(R::Creature).and(R::ControlledByYou)),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), u()]),
                from_hand: true,
                discard_self_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Permanent.and(R::Nonland)),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                        pos: LibraryPosition::OwnerChoice,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Explosive Entry — {1}{R} Sorcery. Destroy target artifact and put a +1/+1
/// counter on target creature. (The printed "up to one" on each is modeled as
/// required targets.)
pub fn explosive_entry() -> CardDefinition {
    CardDefinition {
        name: "Explosive Entry",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Artifact,
                },
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature,
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Norika Yamazaki, the Poet — {2}{W} 3/2 Legendary Human Samurai, vigilance.
/// Whenever a Samurai or Warrior you control attacks alone, you may cast target
/// enchantment card from your graveyard this turn.
pub fn norika_yamazaki_the_poet() -> CardDefinition {
    CardDefinition {
        name: "Norika Yamazaki, the Poet",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::AttackingAlone,
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Samurai)
                            .or(R::HasCreatureType(CreatureType::Warrior)),
                    },
                ]),
            ),
            effect: Effect::GrantMayPlay {
                what: target_filtered(R::Enchantment.and(R::InYourGraveyard)),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        }],
        ..Default::default()
    }
}

/// Kami of Celebration — {4}{R} 3/3 Spirit. Whenever a modified creature you
/// control attacks, exile the top card of your library; you may play it this
/// turn. Whenever you cast a spell from exile, put a +1/+1 counter on target
/// creature you control.
pub fn kami_of_celebration() -> CardDefinition {
    CardDefinition {
        name: "Kami of Celebration",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::IsModified,
                    },
                ),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellFromExile),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Blade of the Oni — {1}{B} 3/1 Equipment Demon, menace. Equipped creature has
/// base power and toughness 5/5, menace, and is a black Demon. Reconfigure
/// {2}{B}{B}.
pub fn blade_of_the_oni() -> CardDefinition {
    CardDefinition {
        name: "Blade of the Oni",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![
            Keyword::Menace,
            Keyword::Reconfigure(cost(&[generic(2), b(), b()])),
        ],
        equipped_bonus: Some(EquipBonus {
            power: 0,
            toughness: 0,
            keywords: vec![Keyword::Menace],
            set_base_pt: Some((5, 5)),
            add_creature_types: vec![CreatureType::Demon],
            set_colors: Some(vec![Color::Black]),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Scrapwork Mutt — {2} 2/1 Artifact Dog. ETB: you may discard a card; if you
/// do, draw a card. Unearth {1}{R}.
pub fn scrapwork_mutt() -> CardDefinition {
    CardDefinition {
        name: "Scrapwork Mutt",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dog],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::MayDiscard {
            description: "Discard a card to draw a card?".into(),
            count: Value::Const(1),
            then: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            }),
            else_: None,
        })],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(1), r()]))],
        ..Default::default()
    }
}

/// Towashi Guide-Bot — {4} 2/1 Artifact Construct. ETB: put a +1/+1 counter on
/// target creature you control. {4}, {T}: Draw a card; costs {1} less to
/// activate for each modified creature you control.
pub fn towashi_guide_bot() -> CardDefinition {
    CardDefinition {
        name: "Towashi Guide-Bot",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            cost_reduction_per: Some(R::Creature.and(R::ControlledByYou).and(R::IsModified)),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Naomi, Pillar of Order — {3}{W}{B} 4/4 Legendary Human Advisor. Whenever
/// Naomi enters or attacks, if you control an artifact and an enchantment,
/// create a 2/2 white Samurai token with vigilance.
pub fn naomi_pillar_of_order() -> CardDefinition {
    let make_token = Effect::If {
        cond: Predicate::All(vec![
            Predicate::SelectorExists(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou))),
            Predicate::SelectorExists(Selector::EachPermanent(
                R::Enchantment.and(R::ControlledByYou),
            )),
        ]),
        then: Box::new(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: samurai_vigilance_token(),
        }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        name: "Naomi, Pillar of Order",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![crate::card::Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            etb(make_token.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: make_token,
            },
        ],
        ..Default::default()
    }
}

/// Jukai Trainee — {1}{G} 2/2 Human Samurai. Whenever it blocks or becomes
/// blocked, it gets +1/+1 until end of turn.
pub fn jukai_trainee() -> CardDefinition {
    let pump = Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(1),
        toughness: Value::Const(1),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Jukai Trainee",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Samurai],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: pump.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: pump,
            },
        ],
        ..Default::default()
    }
}

/// Gloomshrieker — {1}{B}{G} 2/1 Cat Beast enchantment creature, menace. ETB:
/// return target permanent card from your graveyard to your hand. If it would
/// die, exile it instead.
pub fn gloomshrieker() -> CardDefinition {
    CardDefinition {
        name: "Gloomshrieker",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Beast],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        dies_to_exile: true,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::InYourGraveyard),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Ecologist's Terrarium — {2} Artifact. ETB: you may search your library for a
/// basic land card, reveal it, put it into your hand, then shuffle. {2}, {T},
/// Sacrifice this: put a +1/+1 counter on target creature. Sorcery speed.
pub fn ecologists_terrarium() -> CardDefinition {
    CardDefinition {
        name: "Ecologist's Terrarium",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search for a basic land?".into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Colossal Skyturtle-style helper: a 2/2 white Samurai token with vigilance.
fn samurai_vigilance_token() -> TokenDefinition {
    TokenDefinition {
        name: "Samurai".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Samurai],
            ..Default::default()
        },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}
