//! Commander: the cards the **Planeswalker Party** precon (CMM, Commodore
//! Guff) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_guff.rs`.
//!
//! Residuals (each also on its card):
//! - **Chandra, Legacy of Fire** — the 0 removes a loyalty counter from each
//!   planeswalker you control with two or more (never "any number" chosen).
//! - **Guff Rewrites History** — only opponents' permanents are chosen (not
//!   your own), and the exiled lands go to the bottom in exile order.
//! - **Leori** — the planeswalker type is the one most common among yours on
//!   the battlefield and in hand, not a free choice.
//! - **Narset of the Ancient Way** — the −2's damage target is chosen as the
//!   ability is activated, not by a reflexive trigger.
//! - **Repeated Reverberation** — its instant, sorcery and loyalty halves are
//!   three separate "next" riders; each can fire.
//! - **Sparkshaper Visionary** — all or none of your planeswalkers become
//!   Birds; they stay their colour and lack the scry trigger.
//! - **Vronos** — the +1 phases out every other planeswalker you control (not
//!   up to two targets).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, Color, ManaCost, SpendRestriction};
use std::sync::Arc;

fn walker(
    name: &'static str,
    mana: ManaCost,
    subtype: PlaneswalkerSubtype,
    loyalty: u32,
    abilities: Vec<LoyaltyAbility>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![subtype], ..Default::default() },
        base_loyalty: loyalty,
        loyalty_abilities: abilities,
        ..Default::default()
    }
}

fn la(loyalty_cost: i32, effect: Effect) -> LoyaltyAbility {
    LoyaltyAbility { loyalty_cost, effect, ..Default::default() }
}

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn your_walkers() -> R {
    R::Planeswalker.and(R::ControlledByYou)
}

fn count_your_walkers() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(your_walkers())))
}

fn your_step(step: TurnStep) -> EventSpec {
    EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
}

/// Commodore Guff — a loyalty counter on another walker each end step; +1 a
/// Wizard that taps for planeswalker mana; −3 draws and burns per walker.
pub fn commodore_guff() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Wizard".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wizard], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::OfColor(Color::Red, Value::ONE)),
                    SpendRestriction::PlaneswalkerSpellsOnly,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        can_be_commander: true,
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::AddCounter {
                what: target_filtered(your_walkers().and(R::OtherThanSource)),
                kind: CounterType::Loyalty,
                amount: Value::ONE,
            },
        }],
        ..walker(
            "Commodore Guff",
            cost(&[generic(1), u(), r(), w()]),
            PlaneswalkerSubtype::Guff,
            5,
            vec![
                la(1, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(wizard) }),
                la(
                    -3,
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: count_your_walkers() },
                        Effect::DealDamage {
                            to: Selector::Player(PlayerRef::EachOpponent),
                            amount: count_your_walkers(),
                        },
                    ]),
                ),
            ],
        )
    }
}

/// Chandra, Awakened Inferno — can't be countered; +2 burn emblems for each
/// opponent; −3 sweeps non-Elementals; −X exiles what it kills.
pub fn chandra_awakened_inferno() -> CardDefinition {
    let emblem = TriggeredAbility {
        event: your_step(TurnStep::Upkeep),
        effect: Effect::DealDamage { to: Selector::You, amount: Value::ONE },
    };
    CardDefinition {
        keywords: vec![Keyword::CantBeCountered],
        ..walker(
            "Chandra, Awakened Inferno",
            cost(&[generic(4), r(), r()]),
            PlaneswalkerSubtype::Chandra,
            6,
            vec![
                la(
                    2,
                    Effect::CreateEmblem {
                        who: PlayerRef::EachOpponent,
                        name: "Chandra, Awakened Inferno".into(),
                        triggered: vec![emblem],
                        statics: vec![],
                    },
                ),
                la(
                    -3,
                    Effect::DealDamage {
                        to: Selector::EachPermanent(
                            R::Creature.and(R::Not(Box::new(R::HasCreatureType(CreatureType::Elemental)))),
                        ),
                        amount: Value::Const(3),
                    },
                ),
                LoyaltyAbility {
                    loyalty_cost: 0,
                    x_cost: true,
                    effect: Effect::Seq(vec![
                        Effect::ExileIfWouldDieThisTurn {
                            what: target_filtered(R::Creature.or(R::Planeswalker)),
                        },
                        Effect::DealDamage { to: Selector::Target(0), amount: Value::XFromCost },
                    ]),
                },
            ],
        )
    }
}

/// Chandra, Legacy of Fire — each end step burns each opponent per walker;
/// +1 {R} per walker; 0 trades loyalty for impulse draws. Residual: the 0
/// takes one from each walker with two or more.
pub fn chandra_legacy_of_fire() -> CardDefinition {
    let spare = || your_walkers().and(R::WithCounterAtLeast(CounterType::Loyalty, 2));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::End),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: count_your_walkers(),
            },
        }],
        ..walker(
            "Chandra, Legacy of Fire",
            cost(&[generic(4), r()]),
            PlaneswalkerSubtype::Chandra,
            3,
            vec![
                la(
                    1,
                    Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, count_your_walkers()) },
                ),
                la(
                    0,
                    Effect::Seq(vec![
                        Effect::ExileTopAndGrantMayPlay {
                            who: PlayerRef::You,
                            count: Value::CountOf(Box::new(Selector::EachPermanent(spare()))),
                            duration: crate::card::MayPlayDuration::EndOfThisTurn,
                            pay_any_color: false,
                            max_mana_value: None,
                            pay_own_cost: true,
                            uncast_penalty: None,
                        },
                        Effect::RemoveCounter {
                            what: Selector::EachPermanent(spare()),
                            kind: CounterType::Loyalty,
                            amount: Value::ONE,
                        },
                    ]),
                ),
            ],
        )
    }
}

/// Deploy the Gatewatch — up to two planeswalkers from the top seven onto
/// the battlefield, the rest to the bottom at random.
pub fn deploy_the_gatewatch() -> CardDefinition {
    CardDefinition {
        name: "Deploy the Gatewatch",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(7),
            pick_filter: Some(R::Planeswalker),
            take: Some(Value::Const(2)),
            to_battlefield: true,
            rest_bottom_random: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Gatewatch Beacon — enters with three loyalty counters; {T}: {W}; may move
/// one onto each planeswalker that enters.
pub fn gatewatch_beacon() -> CardDefinition {
    CardDefinition {
        name: "Gatewatch Beacon",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Loyalty, Value::Const(3))),
        activated_abilities: vec![crate::sets::tap_add(Color::White)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Planeswalker },
                    Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Loyalty },
                        Value::ONE,
                    ),
                ]),
            ),
            effect: Effect::MayDo {
                description: "Move a loyalty counter onto that planeswalker?".into(),
                body: Box::new(Effect::MoveCounters {
                    from: Selector::This,
                    to: Selector::TriggerSource,
                    counter: CounterType::Loyalty,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Guff Rewrites History — each player's nonenchantment, nonland permanent is
/// shuffled away; each of them flips a free spell off the top.
pub fn guff_rewrites_history() -> CardDefinition {
    CardDefinition {
        name: "Guff Rewrites History",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ForEachPlayerTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Nonland.and(R::Not(Box::new(R::Enchantment))),
                effect: Box::new(Effect::ShuffleInThenCastFromTopFree { what: Selector::Target(0) }),
            }),
        },
        ..Default::default()
    }
}

/// Jace, Mirror Mage — kicker {2} for a non-legendary one-loyalty copy; +1
/// scry 2; 0 draws and pays the card's mana value in loyalty.
pub fn jace_mirror_mage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2)]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::This,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: true,
                    legendary: false,
                    extra_keywords: vec![],
                },
                Effect::SetLoyalty { what: Selector::LastCreatedToken, value: Value::ONE },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..walker(
            "Jace, Mirror Mage",
            cost(&[generic(1), u(), u()]),
            PlaneswalkerSubtype::Jace,
            4,
            vec![
                la(1, Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
                la(
                    0,
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::RemoveCounter {
                            what: Selector::This,
                            kind: CounterType::Loyalty,
                            amount: Value::ManaValueOf(Box::new(Selector::LastCardYouDrew)),
                        },
                    ]),
                ),
            ],
        )
    }
}

/// Jaya's Phoenix — flying, haste; connecting copies your next loyalty
/// ability this turn; a planeswalker spell may bring it back.
pub fn jayas_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CopyNextLoyaltyAbility { copies: 1 },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Planeswalker },
                ),
                effect: Effect::MayDo {
                    description: "Return Jaya's Phoenix to the battlefield?".into(),
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                },
            },
        ],
        ..creature("Jaya's Phoenix", cost(&[generic(4), r()]), vec![CreatureType::Phoenix], 3, 3)
    }
}

/// Leori, Sparktouched Hunter — flying, vigilance; connecting copies this
/// turn's abilities of one planeswalker type. Residual: the type isn't a free
/// choice.
pub fn leori_sparktouched_hunter() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CopyLoyaltyAbilitiesOfChosenTypeThisTurn,
        }],
        ..creature(
            "Leori, Sparktouched Hunter",
            cost(&[u(), r(), w()]),
            vec![CreatureType::Elemental, CreatureType::Cat],
            3,
            3,
        )
    }
}

/// Narset of the Ancient Way — +1 life and noncreature mana; −2 loot into
/// damage; −6 a noncreature-spell burn emblem.
pub fn narset_of_the_ancient_way() -> CardDefinition {
    let emblem = TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Not(Box::new(R::Creature)))),
        effect: Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(2) },
    };
    walker(
        "Narset of the Ancient Way",
        cost(&[generic(1), u(), r(), w()]),
        PlaneswalkerSubtype::Narset,
        4,
        vec![
            la(
                1,
                Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::Restricted(
                            Box::new(ManaPayload::OfColors(vec![Color::Blue, Color::Red, Color::White], Value::ONE)),
                            SpendRestriction::NoncreatureSpellsOnly,
                        ),
                    },
                ]),
            ),
            la(
                -2,
                Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::MayDo {
                        description: "Discard a card?".into(),
                        body: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                    },
                    Effect::DealDamage {
                        to: target_filtered(R::Creature.or(R::Planeswalker)),
                        amount: Value::GreatestDiscardedManaValueThisEffect,
                    },
                ]),
            ),
            la(
                -6,
                Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Narset of the Ancient Way".into(),
                    triggered: vec![emblem],
                    statics: vec![],
                },
            ),
        ],
    )
}

/// Narset, Enlightened Master — first strike, hexproof; attacking exiles the
/// top four and frees their noncreature spells this turn.
pub fn narset_enlightened_master() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike, Keyword::Hexproof],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::Const(4),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::GrantMayPlay {
                    what: Selector::ExiledThisResolution {
                        filter: R::Nonland.and(R::Not(Box::new(R::Creature))),
                    },
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: false,
                    any_color: false,
                },
            ]),
        }],
        ..creature(
            "Narset, Enlightened Master",
            cost(&[generic(3), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Monk],
            3,
            2,
        )
    }
}

/// Oath of Teferi — flickers another permanent of yours until the end step;
/// your planeswalkers' loyalty abilities work twice a turn.
pub fn oath_of_teferi() -> CardDefinition {
    CardDefinition {
        name: "Oath of Teferi",
        cost: cost(&[generic(3), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::ExileReturnToOwnerNextEndStep {
            what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource)),
            tapped: false,
        })],
        static_abilities: vec![StaticAbility {
            description: "You may activate the loyalty abilities of planeswalkers you control twice each turn.",
            effect: StaticEffect::LoyaltyAbilitiesTwiceEachTurn,
        }],
        ..Default::default()
    }
}

/// Onakke Oathkeeper — attacking your planeswalkers costs {1} per creature;
/// from the graveyard, {4}{W}{W} and exile it: return a planeswalker card.
pub fn onakke_oathkeeper() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack planeswalkers you control unless their controller pays {1} for each.",
            effect: StaticEffect::AttackTaxOnYourPlaneswalkers { amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w(), w()]),
            from_graveyard: true,
            exile_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Planeswalker.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Onakke Oathkeeper",
            cost(&[generic(1), w()]),
            vec![CreatureType::Ogre, CreatureType::Spirit],
            0,
            4,
        )
    }
}

/// Repeated Reverberation — the next instant, sorcery or loyalty ability this
/// turn is copied twice. Residual: three separate riders.
pub fn repeated_reverberation() -> CardDefinition {
    let copy_twice = |t: CardType| Effect::OnYourNextSpellOfTypeThisTurn {
        card_type: t,
        body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::Const(2) }),
    };
    CardDefinition {
        name: "Repeated Reverberation",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            copy_twice(CardType::Instant),
            copy_twice(CardType::Sorcery),
            Effect::CopyNextLoyaltyAbility { copies: 2 },
        ]),
        ..Default::default()
    }
}

/// Sparkshaper Visionary — each combat on your turn your planeswalkers may
/// become 3/3 flying, hexproof Birds until end of turn. Residual: all or none;
/// no colour change or scry trigger.
pub fn sparkshaper_visionary() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_step(TurnStep::BeginCombat),
            effect: Effect::MayDo {
                description: "Turn your planeswalkers into 3/3 flying Birds?".into(),
                body: Box::new(Effect::BecomeCreature {
                    what: Selector::EachPermanent(your_walkers()),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![CreatureType::Bird],
                    keywords: vec![Keyword::Flying, Keyword::Hexproof],
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..creature(
            "Sparkshaper Visionary",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            5,
        )
    }
}

/// Teyo, Geometric Tactician — enters with a 0/4 flying Wall; +1 you and an
/// opponent draw; −2 fixes the attack direction until your next turn.
pub fn teyo_geometric_tactician() -> CardDefinition {
    let wall = TokenDefinition {
        name: "Wall".into(),
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender, Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: Arc::new(wall),
        })],
        ..walker(
            "Teyo, Geometric Tactician",
            cost(&[generic(2), w()]),
            PlaneswalkerSubtype::Teyo,
            3,
            vec![
                la(
                    1,
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::ONE },
                    ]),
                ),
                la(-2, Effect::ChooseAttackDirectionUntilYourNextTurn),
            ],
        )
    }
}

/// Vronos, Masked Inquisitor — +1 hides your other planeswalkers at the end
/// step; −2 bounces a nonland permanent per opponent; −7 makes an artifact a
/// 9/9 unblockable Construct. Residual: the +1 phases out all your others.
pub fn vronos_masked_inquisitor() -> CardDefinition {
    walker(
        "Vronos, Masked Inquisitor",
        cost(&[generic(3), u(), u()]),
        PlaneswalkerSubtype::Vronos,
        5,
        vec![
            la(
                1,
                Effect::AtNextEndStep {
                    body: Box::new(Effect::PhaseOut {
                        what: Selector::EachPermanent(your_walkers().and(R::OtherThanSource)),
                        until_source_leaves: false,
                    }),
                },
            ),
            la(
                -2,
                Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 5,
                        min_targets: 0,
                        filter: R::Nonland.and(R::ControlledByOpponent),
                        effect: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                        }),
                    }),
                },
            ),
            la(
                -7,
                Effect::BecomeCreature {
                    what: target_filtered(R::Artifact.and(R::ControlledByYou)),
                    power: Value::Const(9),
                    toughness: Value::Const(9),
                    creature_types: vec![CreatureType::Construct],
                    keywords: vec![Keyword::Vigilance, Keyword::Indestructible, Keyword::Unblockable],
                    duration: Duration::Permanent,
                },
            ),
        ],
    )
}
