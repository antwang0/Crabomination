//! DSK (Duskmourn) gap batch — Survival creatures, Delirium payoffs, a
//! modal redirect, and Norin's blocked-creature blink. Tests in
//! `tests/recent239.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, MayPlayDuration, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{animate_land, deal, target_filtered, valiant};
use crate::game::types::TurnStep;
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, EventKind, EventScope, EventSpec, ManaPayload,
    OpeningHandEffect, PlayerRef, PlayerStaticTarget, Predicate, Selector, SpreeMode, StaticEffect,
    Value, ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color, SpendRestriction};

/// DSK **Survival** — "At the beginning of your second main phase, if this
/// creature is tapped, …". Models to a PostCombatMain trigger gated on the
/// source being tapped.
fn survival(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::PostCombatMain), EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::Tapped }),
        effect: body,
    }
}

/// Betrayer's Bargain — {1}{R} Instant. Additional cost: sacrifice a creature
/// or enchantment or pay {2}. Deal 5 to target creature; if it would die this
/// turn, exile it instead.
pub fn betrayers_bargain() -> CardDefinition {
    CardDefinition {
        name: "Betrayer's Bargain",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificeOrPay {
            filter: R::Creature.or(R::Enchantment),
            pay: 2,
        }],
        effect: Effect::Seq(vec![
            Effect::ExileIfWouldDieThisTurn { what: Selector::Target(0) },
            deal(5, target_filtered(R::Creature)),
        ]),
        ..Default::default()
    }
}

/// Untimely Malfunction — {1}{R} Instant. Choose one — destroy target artifact;
/// choose new targets for target spell; or one or two target creatures can't
/// block this turn.
pub fn untimely_malfunction() -> CardDefinition {
    CardDefinition {
        name: "Untimely Malfunction",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::ChooseNewTargetsForSpell { what: target_filtered(R::IsSpellOnStack) },
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature,
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Omnivorous Flytrap — {2}{G} Plant 2/4. Delirium — Whenever it enters or
/// attacks, if 4+ card types in your graveyard, distribute two +1/+1 counters
/// among one or two target creatures; then if 6+ types, double the +1/+1
/// counters on those creatures.
pub fn omnivorous_flytrap() -> CardDefinition {
    let delirium_body = || {
        Effect::Seq(vec![
            Effect::DistributeCounters {
                total: Value::Const(2),
                counter: CounterType::PlusOnePlusOne,
                filter: R::Creature,
                max_targets: 2,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CardTypesInGraveyard(PlayerRef::You),
                    Value::Const(6),
                ),
                then: Box::new(Effect::DoubleCountersOnEach {
                    what: Selector::AllTargets,
                    kind: CounterType::PlusOnePlusOne,
                }),
                else_: Box::new(Effect::Noop),
            },
        ])
    };
    let delirium = || Predicate::DeliriumActive { who: PlayerRef::You };
    CardDefinition {
        name: "Omnivorous Flytrap",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                    .with_filter(delirium()),
                effect: delirium_body(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                    .with_filter(delirium()),
                effect: delirium_body(),
            },
        ],
        ..Default::default()
    }
}

/// Norin, Swift Survivalist — {R} Human Coward 2/1. Can't block. Whenever a
/// creature you control becomes blocked, you may exile it, then play it from
/// exile this turn.
pub fn norin_swift_survivalist() -> CardDefinition {
    CardDefinition {
        name: "Norin, Swift Survivalist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Coward],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Exile it, then play it from exile this turn".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: MayPlayDuration::EndOfThisTurn,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Outlaw Stitcher — {3}{U} Human Warlock 1/4. When it enters, create a 2/2
/// blue-black Zombie Rogue, then put two +1/+1 counters on it for each spell
/// you've cast this turn other than the first. Plot {4}{U}.
pub fn outlaw_stitcher() -> CardDefinition {
    CardDefinition {
        name: "Outlaw Stitcher",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        plot_cost: Some(cost(&[generic(4), u()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Zombie Rogue".into(),
                        power: 2,
                        toughness: 2,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue, Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Zombie, CreatureType::Rogue],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(Value::NonNeg(Box::new(Value::Diff(
                            Box::new(Value::SpellsCastThisTurn(PlayerRef::You)),
                            Box::new(Value::ONE),
                        )))),
                    ),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Kutzil's Flanker — {2}{W} Cat Warrior 3/1. Flash. When it enters, choose one
/// — a +1/+1 counter per creature that left your control this turn (approximated
/// as creatures that died this turn); gain 2 life and scry 2; or exile target
/// player's graveyard.
pub fn kutzils_flanker() -> CardDefinition {
    CardDefinition {
        name: "Kutzil's Flanker",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CreaturesDiedThisTurn(PlayerRef::You),
                },
                Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                ]),
                Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0) },
            ]),
        }],
        ..Default::default()
    }
}

/// Stubborn Burrowfiend — {1}{G} Badger Beast Mount 2/2. Saddle 2. The first
/// time it becomes saddled each turn, mill two, then it gets +X/+X until end of
/// turn, where X is the number of creature cards in your graveyard.
pub fn stubborn_burrowfiend() -> CardDefinition {
    let gy_creatures =
        || Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature };
    CardDefinition {
        name: "Stubborn Burrowfiend",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger, CreatureType::Beast, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Saddle(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CrewsOrSaddles, EventScope::SelfSource).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                Effect::PumpPT {
                    what: Selector::This,
                    power: gy_creatures(),
                    toughness: gy_creatures(),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Unscrupulous Contractor — {2}{B} Human Assassin 3/2. When it enters, you may
/// sacrifice a creature; if you do, target player draws two cards and loses 2
/// life. Plot {2}{B}.
pub fn unscrupulous_contractor() -> CardDefinition {
    CardDefinition {
        name: "Unscrupulous Contractor",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        plot_cost: Some(cost(&[generic(2), b()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a creature: target player draws two cards and loses 2 life".into(),
                filter: R::Creature,
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
                    Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(2) },
                ])),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Tumbleweed Rising — {1}{G} Sorcery. Create an X/X green Elemental, X = the
/// greatest power among creatures you control. Plot {2}{G}.
pub fn tumbleweed_rising() -> CardDefinition {
    CardDefinition {
        name: "Tumbleweed Rising",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(2), g()])),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Elemental".into(),
                power: 0,
                toughness: 0,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Elemental],
                    ..Default::default()
                },
                dynamic_pt: Some((
                    Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                    Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
                )),
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Bite Down on Crime — {3}{G} Sorcery. Optional additional cost: collect
/// evidence 6, for {2} less. Target creature you control gets +2/+0, then deals
/// damage equal to its power to target creature you don't control.
pub fn bite_down_on_crime() -> CardDefinition {
    CardDefinition {
        name: "Bite Down on Crime",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::CollectEvidence { amount: 6, optional: true }],
        self_cost_reduction_if_collect_evidence: Some(2),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
        ]),
        ..Default::default()
    }
}

/// Behind the Mask — {U} Instant. Optional additional cost: collect evidence 6.
/// Until end of turn, target artifact or creature becomes an artifact creature
/// with base P/T 4/3 — or 1/1 instead if evidence was collected. (The added
/// artifact type on a nonartifact target is approximated — `BecomeCreature`
/// keeps its printed types and animates it.)
pub fn behind_the_mask() -> CardDefinition {
    let animate = |power| Effect::BecomeCreature {
        what: Selector::TargetFiltered { slot: 0, filter: R::Artifact.or(R::Creature) },
        power: Value::Const(power),
        toughness: Value::Const(if power == 1 { 1 } else { 3 }),
        creature_types: vec![],
        keywords: vec![],
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Behind the Mask",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::CollectEvidence { amount: 6, optional: true }],
        effect: Effect::If {
            cond: Predicate::SpellCollectedEvidence,
            then: Box::new(animate(1)),
            else_: Box::new(animate(4)),
        },
        ..Default::default()
    }
}

/// Analyze the Pollen — {G} Sorcery. Optional additional cost: collect evidence
/// 8. Search your library for a basic land card — or a creature or land card
/// instead if evidence was collected — reveal it, put it into your hand, shuffle.
pub fn analyze_the_pollen() -> CardDefinition {
    CardDefinition {
        name: "Analyze the Pollen",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::CollectEvidence { amount: 8, optional: true }],
        effect: Effect::If {
            cond: Predicate::SpellCollectedEvidence,
            then: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Creature.or(R::Land),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Mudflat Village — Land. {T}: Add {C}. {T}: Add {B}, creature-spells only.
/// {1}{B}, {T}, Sacrifice: return a Bat/Lizard/Rat/Squirrel card from your
/// graveyard to your hand.
pub fn mudflat_village() -> CardDefinition {
    let kindred = || {
        R::HasCreatureType(CreatureType::Bat)
            .or(R::HasCreatureType(CreatureType::Lizard))
            .or(R::HasCreatureType(CreatureType::Rat))
            .or(R::HasCreatureType(CreatureType::Squirrel))
    };
    CardDefinition {
        name: "Mudflat Village",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Black])),
                        SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), b()]),
                sac_cost: true,
                effect: Effect::Move {
                    what: target_filtered(kindred()),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Oakhollow Village — Land. {T}: Add {C}. {T}: Add {G}, creature-spells only.
/// {G}, {T}: put a +1/+1 counter on each Frog/Rabbit/Raccoon/Squirrel you
/// control that entered this turn.
pub fn oakhollow_village() -> CardDefinition {
    let kindred = R::HasCreatureType(CreatureType::Frog)
        .or(R::HasCreatureType(CreatureType::Rabbit))
        .or(R::HasCreatureType(CreatureType::Raccoon))
        .or(R::HasCreatureType(CreatureType::Squirrel));
    CardDefinition {
        name: "Oakhollow Village",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Green])),
                        SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[g()]),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        kindred.and(R::ControlledByYou).and(R::EnteredThisTurn),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lupinflower Village — Land. {T}: Add {C}. {T}: Add {W}, creature-spells only.
/// {1}{W}, {T}, Sacrifice: look at the top six cards, put a Bat/Bird/Mouse/Rabbit
/// card into your hand, bottom the rest in a random order.
pub fn lupinflower_village() -> CardDefinition {
    let kindred = R::HasCreatureType(CreatureType::Bat)
        .or(R::HasCreatureType(CreatureType::Bird))
        .or(R::HasCreatureType(CreatureType::Mouse))
        .or(R::HasCreatureType(CreatureType::Rabbit));
    CardDefinition {
        name: "Lupinflower Village",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::White])),
                        SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1), w()]),
                sac_cost: true,
                effect: Effect::LookPickToHand {
                    who: PlayerRef::You,
                    count: Value::Const(6),
                    rest_to_graveyard: false,
                    pick_filter: Some(kindred),
                    take: None,
                    to_battlefield: false,
                    gain_life_if_pick: None,
                    gain_life_greatest_power_rest: false,
                    optional: true,
                    picked_lands_to_battlefield: false,
                    rest_bottom_random: true,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Lilypad Village — Land. {T}: Add {C}. {T}: Add {U}, creature-spells only.
/// {U}, {T}: Surveil 2. Activate only if a Bird/Frog/Otter/Rat you control
/// entered this turn. (The "entered this turn" gate reads a kindred creature you
/// currently control — one that entered and then left is approximated away.)
pub fn lilypad_village() -> CardDefinition {
    let kindred = R::HasCreatureType(CreatureType::Bird)
        .or(R::HasCreatureType(CreatureType::Frog))
        .or(R::HasCreatureType(CreatureType::Otter))
        .or(R::HasCreatureType(CreatureType::Rat));
    CardDefinition {
        name: "Lilypad Village",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Blue])),
                        SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[u()]),
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        kindred.and(R::ControlledByYou).and(R::EnteredThisTurn),
                    ),
                    n: Value::ONE,
                }),
                effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Rockface Village — Land. {T}: Add {C}. {T}: Add {R}, creature-spells only.
/// {R}, {T}: target Lizard/Mouse/Otter/Raccoon you control gets +1/+0 and haste
/// until end of turn (sorcery speed).
pub fn rockface_village() -> CardDefinition {
    let kindred = R::HasCreatureType(CreatureType::Lizard)
        .or(R::HasCreatureType(CreatureType::Mouse))
        .or(R::HasCreatureType(CreatureType::Otter))
        .or(R::HasCreatureType(CreatureType::Raccoon));
    CardDefinition {
        name: "Rockface Village",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Red])),
                        SpendRestriction::CreatureOnly,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[r()]),
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: target_filtered(kindred.and(R::ControlledByYou)),
                        power: Value::ONE,
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Whiskervale Forerunner — {3}{W} Mouse Bard 3/4. Valiant — the first time it
/// becomes the target of your spell/ability each turn, look at the top five,
/// reveal a creature card with mana value 3 or less, and put it onto the
/// battlefield (approximating the "if it's your turn, else hand" routing);
/// bottom the rest at random.
pub fn whiskervale_forerunner() -> CardDefinition {
    CardDefinition {
        name: "Whiskervale Forerunner",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![valiant(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: Some(R::Creature.and(R::ManaValueAtMost(3))),
            take: None,
            to_battlefield: true,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            optional: true,
            picked_lands_to_battlefield: false,
            rest_bottom_random: true,
        })],
        ..Default::default()
    }
}

/// Hollow Marauder — {6}{B} Specter Rogue 4/2. Costs {1} less per creature card
/// in your graveyard. Flying. ETB: each opponent discards a card, and you draw a
/// card for each who discarded a card with mana value 3 or less. (The "any
/// number of target opponents" slot collapses to each opponent — 1v1-faithful.)
pub fn hollow_marauder() -> CardDefinition {
    CardDefinition {
        name: "Hollow Marauder",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        affinity_graveyard_filter: Some(R::Creature),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                    random: false,
                },
                Effect::If {
                    cond: Predicate::LastDiscardedManaValueAtMost(3),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Freestrider Commando — {2}{G} Centaur Mercenary 3/3. Plot {3}{G}. Enters with
/// two +1/+1 counters if no mana was spent to cast it (a plotted/free cast) or
/// it wasn't cast at all.
pub fn freestrider_commando() -> CardDefinition {
    CardDefinition {
        name: "Freestrider Commando",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        plot_cost: Some(cost(&[generic(3), g()])),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                // "wasn't cast" OR "no mana spent" — either leaves mana_spent 0.
                cond: Predicate::Not(Box::new(Predicate::CastSpellManaSpentAtLeast(1))),
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Crimestopper Sprite — {2}{U} Faerie Detective 2/2. Flying. Optional
/// additional cost: collect evidence 6. ETB: tap target creature; if evidence
/// was collected, also put a stun counter on it.
pub fn crimestopper_sprite() -> CardDefinition {
    CardDefinition {
        name: "Crimestopper Sprite",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        additional_cast_cost: vec![AdditionalCastCost::CollectEvidence { amount: 6, optional: true }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::If {
                    cond: Predicate::SpellCollectedEvidence,
                    then: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Feed the Cycle — {1}{B} Instant. Additional cost: forage or pay {B} (folded
/// as {1} generic). Destroy target creature or planeswalker.
pub fn feed_the_cycle() -> CardDefinition {
    CardDefinition {
        name: "Feed the Cycle",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::ForageOrPay { pay: 1 }],
        effect: Effect::Destroy {
            what: target_filtered(R::Creature.or(R::Planeswalker)),
        },
        ..Default::default()
    }
}

/// Fear of Burning Alive — {4}{R}{R} Enchantment Creature — Nightmare 4/4. ETB:
/// deals 4 to each opponent. Delirium — whenever a source you control deals
/// noncombat damage to an opponent, if delirium, deal that much to a creature
/// that player controls. (The "source you control" clause collapses to "an
/// opponent is dealt noncombat damage", matching the catalog's other
/// noncombat-damage triggers.)
pub fn fear_of_burning_alive() -> CardDefinition {
    CardDefinition {
        name: "Fear of Burning Alive",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(4) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::OpponentControl)
                    .with_filter(Predicate::DeliriumActive { who: PlayerRef::You }),
                effect: Effect::DealDamage {
                    to: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByOpponent) },
                    amount: Value::TriggerEventAmount,
                },
            },
        ],
        ..Default::default()
    }
}

/// Creeping Peeper — {1}{U} Eye 2/1. {T}: Add {U}. Spend only to cast an
/// enchantment spell, unlock a door, or turn a permanent face up. (Only the
/// enchantment-spell half of the restriction is enforced.)
pub fn creeping_peeper() -> CardDefinition {
    CardDefinition {
        name: "Creeping Peeper",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eye], ..Default::default() },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Blue])),
                    SpendRestriction::EnchantmentSpell,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leyline of Hope — {2}{W}{W} Enchantment. May begin the game in play. Your
/// life gain is boosted by 1; while you have 7+ life above your starting total,
/// creatures you control get +2/+2.
pub fn leyline_of_hope() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Hope",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        opening_hand: Some(OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        static_abilities: vec![
            StaticAbility {
                description: "If you would gain life, gain that much plus 1 instead.",
                effect: StaticEffect::LifeGainBonus {
                    target: PlayerStaticTarget::Controller,
                    amount: 1,
                },
            },
            StaticAbility {
                description: "With 7+ life above starting, your creatures get +2/+2.",
                effect: StaticEffect::PumpTeamIf {
                    condition: Predicate::PlayerLifeAtLeastAboveStarting { who: PlayerRef::You, delta: 7 },
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: 2,
                    toughness: 2,
                    keywords: vec![],
                },
            },
        ],
        ..Default::default()
    }
}

/// Monstrous Emergence — {1}{G} Sorcery. Additional cost: choose a creature you
/// control or reveal a creature card from your hand. Deal damage equal to that
/// creature's power to target creature.
pub fn monstrous_emergence() -> CardDefinition {
    CardDefinition {
        name: "Monstrous Emergence",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::ChooseOrRevealCreature],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Oblivious Bookworm — {G}{U} Human Wizard 2/3. At the beginning of your end
/// step, you may draw a card; if you do, discard a card unless a permanent
/// entered face down under your control this turn or you turned a permanent
/// face up this turn.
pub fn oblivious_bookworm() -> CardDefinition {
    CardDefinition {
        name: "Oblivious Bookworm",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Draw a card (then discard unless you had face-down activity)?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::If {
                        cond: Predicate::FaceDownActivityThisTurn { who: PlayerRef::You },
                        then: Box::new(Effect::Noop),
                        else_: Box::new(Effect::Discard {
                            who: Selector::You,
                            amount: Value::Const(1),
                            random: false,
                        }),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Axebane Ferox — {2}{G}{G} Beast 4/4. Deathtouch, haste, Ward—Collect
/// evidence 4.
pub fn axebane_ferox() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Axebane Ferox",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch, Keyword::Haste, Keyword::Ward(WardCost::CollectEvidence(4))],
        ..Default::default()
    }
}

/// Paranormal Analyst — {1}{U} Human Detective 1/3. Whenever you manifest
/// dread, put the card you put into your graveyard this way into your hand.
pub fn paranormal_analyst() -> CardDefinition {
    CardDefinition {
        name: "Paranormal Analyst",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ManifestedDread, EventScope::YourControl),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Trial of Agony — {R} Sorcery. Two target creatures an opponent controls: 5
/// damage to one, the other can't block this turn. (The "same opponent" and
/// "that player chooses" clauses are collapsed to the caster's pick, matching
/// the catalog's other opponent-choice removal.)
pub fn trial_of_agony() -> CardDefinition {
    let opp_creature = || R::Creature.and(R::ControlledByOpponent);
    CardDefinition {
        name: "Trial of Agony",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 0, filter: opp_creature() },
                amount: Value::Const(5),
            },
            Effect::GrantKeyword {
                what: Selector::TargetFiltered { slot: 1, filter: opp_creature() },
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Getaway Glamer — {W} Instant. Spree — +{1} blink target nontoken creature
/// (returns at the next end step); +{2} destroy target creature if no other
/// creature has greater power.
pub fn getaway_glamer() -> CardDefinition {
    CardDefinition {
        name: "Getaway Glamer",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Spree {
            modes: vec![
                SpreeMode {
                    cost: cost(&[generic(1)]),
                    effect: Effect::ExileReturnNextEndStep {
                        what: target_filtered(R::Creature.and(R::NotToken)),
                    },
                },
                SpreeMode {
                    cost: cost(&[generic(2)]),
                    effect: Effect::If {
                        cond: Predicate::EntityMatches {
                            what: Selector::Target(0),
                            filter: R::HasGreatestPowerAmongAllCreatures,
                        },
                        then: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                        else_: Box::new(Effect::Noop),
                    },
                },
            ],
        },
        ..Default::default()
    }
}

/// Rootwise Survivor — {3}{G}{G} Human Survivor 3/4. Haste. Survival — put
/// three +1/+1 counters on up to one target land you control; it becomes a 0/0
/// Elemental in addition to its types and gains haste. (Haste is modeled as
/// permanent rather than until-your-next-turn.)
pub fn rootwise_survivor() -> CardDefinition {
    CardDefinition {
        name: "Rootwise Survivor",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Survivor],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![survival(animate_land(0, 3))],
        ..Default::default()
    }
}

/// Reluctant Role Model — {1}{W} Human Survivor 2/2. Survival — put a flying,
/// lifelink, or +1/+1 counter on it. Whenever it or another creature you
/// control dies, put those counters on up to one target creature. ("Up to one"
/// is modeled as a required target.)
pub fn reluctant_role_model() -> CardDefinition {
    CardDefinition {
        name: "Reluctant Role Model",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Survivor],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            survival(Effect::ChooseMode(vec![
                Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Flying, amount: Value::ONE },
                Effect::AddKeywordCounter { what: Selector::This, keyword: Keyword::Lifelink, amount: Value::ONE },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
                effect: Effect::MoveAllCounters {
                    from: Selector::TriggerSource,
                    to: target_filtered(R::Creature),
                },
            },
        ],
        ..Default::default()
    }
}

/// Come Back Wrong — {2}{B} Sorcery. Destroy target creature. If a creature
/// card is put into a graveyard this way, return it to the battlefield under
/// your control, then sacrifice it at your next end step.
pub fn come_back_wrong() -> CardDefinition {
    CardDefinition {
        name: "Come Back Wrong",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::Target(0) },
            Effect::If {
                // Only reanimate if the destroy actually buried a creature card
                // (indestructible/regenerated creatures stay put; tokens vanish).
                cond: Predicate::EntityMatches {
                    what: Selector::Target(0),
                    filter: R::Creature.and(R::InGraveyard),
                },
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::NextEndStep,
                        body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Valgavoth's Onslaught — {X}{X}{G} Sorcery. Manifest dread X times, then put
/// X +1/+1 counters on each of those creatures.
pub fn valgavoths_onslaught() -> CardDefinition {
    CardDefinition {
        name: "Valgavoth's Onslaught",
        cost: cost(&[x(), x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ManifestDreadRepeatThenCounters {
            count: Value::XFromCost,
            counters: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Altanak, the Thrice-Called — {5}{G}{G} Insect Beast 9/9. Trample. Whenever
/// it becomes the target of a spell/ability an opponent controls, draw a card.
/// {1}{G}, Discard this card: return target land card from your graveyard to
/// the battlefield tapped. (The draw trigger reuses the Battle Mammoth scope,
/// which fires for any permanent you control.)
pub fn altanak_the_thrice_called() -> CardDefinition {
    CardDefinition {
        name: "Altanak, the Thrice-Called",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Beast],
            ..Default::default()
        },
        power: 9,
        toughness: 9,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::BecameTarget,
                EventScope::YourPermanentTargetedByOpponent,
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            from_hand: true,
            discard_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Land.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
