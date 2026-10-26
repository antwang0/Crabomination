//! Commander: the 2026-10-13 rescan's gaps for Raphael, Fiendish Savior;
//! Mishra, Claimed by Gix; and Tom, Bert, and William. Tests in
//! `tests/recent_b/cmdr_edhrec42.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_any, target_filtered};
use crate::effect::{AttackingTokenCleanup, Duration, Effect, ManaPayload, PlayerRef, RevealMissDest, ZoneDest};
use crate::mana::{b, cost, g, generic, hybrid, r, Color, ManaCost};
use crabomination_base::turn_step::TurnStep;
use std::sync::Arc;

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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

/// A 1/1 red Devil with "When this token dies, it deals 1 damage to any target."
fn devil() -> TokenDefinition {
    TokenDefinition {
        name: "Devil".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Devil], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
        }],
        ..Default::default()
    }
}

fn devils(n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: Arc::new(devil()) }
}

// ── Raphael, Fiendish Savior / Mishra, Claimed by Gix ────────────────────────

/// Dance with Devils — {3}{R} instant. Two 1/1 red Devils with "When this
/// token dies, it deals 1 damage to any target."
pub fn dance_with_devils() -> CardDefinition {
    CardDefinition {
        name: "Dance with Devils",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Instant],
        effect: devils(2),
        ..Default::default()
    }
}

/// Devils' Playground — {4}{R}{R} sorcery. Four of those Devils.
pub fn devils_playground() -> CardDefinition {
    CardDefinition {
        name: "Devils' Playground",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: devils(4),
        ..Default::default()
    }
}

/// Zariel, Archduke of Avernus — {2}{R}{R} planeswalker, loyalty 4. +1: your
/// creatures get +1/+0 and haste this turn; 0: a Devil; −6: an emblem that
/// untaps a creature of yours at the end of your first combat each turn and
/// adds a combat after it.
pub fn zariel_archduke_of_avernus() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Zariel, Archduke of Avernus",
        cost: cost(&[generic(2), r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Zariel], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::PumpPT { what: yours(), power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: 0, effect: devils(1), ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Zariel, Archduke of Avernus".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::StepBegins(TurnStep::EndCombat), EventScope::YourControl)
                            .with_filter(Predicate::IsFirstCombatPhaseThisTurn),
                        effect: Effect::Seq(vec![
                            Effect::Untap {
                                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                                up_to: None,
                            },
                            Effect::AdditionalCombatPhase { count: Value::ONE },
                        ]),
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hanweir Battlements — land. {T}: {C}. {R}, {T}: target creature gains
/// haste this turn. {3}{R}{R}, {T}: meld with Hanweir Garrison into Hanweir,
/// the Writhing Township (CR 701.42).
pub fn hanweir_battlements() -> CardDefinition {
    CardDefinition {
        name: "Hanweir Battlements",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[r()]),
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3), r(), r()]),
                effect: Effect::Meld {
                    partner: "Hanweir Garrison".into(),
                    into: "Hanweir, the Writhing Township".into(),
                    attacking: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Hanweir, the Writhing Township — the melded Battlements + Garrison: a 7/4
/// legendary Eldrazi Ooze, trample, haste; attacking, two 3/2 colorless
/// Eldrazi Horrors tapped and attacking.
pub fn hanweir_the_writhing_township() -> CardDefinition {
    let horror = TokenDefinition {
        name: "Eldrazi Horror".into(),
        power: 3,
        toughness: 2,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Horror],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        color_indicator: vec![Color::Red],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: Arc::new(horror),
                cleanup: AttackingTokenCleanup::None,
                defender: None,
            },
        }],
        ..legend(
            "Hanweir, the Writhing Township",
            ManaCost::default(),
            vec![CreatureType::Eldrazi, CreatureType::Ooze],
            7,
            4,
        )
    }
}

/// Tilonalli's Summoner — {1}{R} 1/1 Human Shaman, ascend. Attacking, you may
/// pay {X}{R} for X 1/1 red Elementals tapped and attacking, exiled at the next
/// end step unless you have the city's blessing then.
pub fn tilonallis_summoner() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![crate::sets::ascend()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPayXPlus {
                extra: cost(&[r()]),
                description: "Pay {X}{R} for X attacking Elementals?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateTokenAttacking {
                        who: PlayerRef::You,
                        count: Value::XFromCost,
                        definition: Arc::new(elemental),
                        cleanup: AttackingTokenCleanup::None,
                        defender: None,
                    },
                    Effect::ExileAtNextEndStepUnless {
                        what: Selector::LastCreatedTokens,
                        unless: Predicate::HasCityBlessing { who: PlayerRef::You },
                    },
                ])),
            },
        }],
        ..creature(
            "Tilonalli's Summoner",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Shaman],
            1,
            1,
        )
    }
}

// ── Tom, Bert, and William ───────────────────────────────────────────────────

/// Tom, Bert, and William — {3}{B}{G} legendary 5/5 Troll. {1}, sacrifice
/// another creature: draw its power, then discard a card. Dying as a creature,
/// they return to the battlefield as an artifact, no longer a creature.
pub fn tom_bert_and_william() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Creature.and(R::ControlledByYou).and(R::OtherThanSource), 1)),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOf(Box::new(Selector::This)),
                        tapped: false,
                    },
                },
                Effect::SetCardTypesTo { what: Selector::This, card_types: vec![CardType::Artifact] },
            ]),
        }],
        ..legend(
            "Tom, Bert, and William",
            cost(&[generic(3), b(), g()]),
            vec![CreatureType::Troll],
            5,
            5,
        )
    }
}

/// Bebop & Rocksteady — {1}{B/G}{B/G} legendary 7/5 Boar Rhino Mutant.
/// Attacking or blocking: sacrifice a permanent unless you discard a card.
pub fn bebop_rocksteady() -> CardDefinition {
    let toll = || Effect::MayDiscard {
        description: "Discard a card rather than sacrifice a permanent?".into(),
        count: Value::ONE,
        then: Box::new(Effect::Noop),
        else_: Some(Box::new(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent })),
    };
    let bg = || hybrid(Color::Black, Color::Green);
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: toll() },
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: toll() },
        ],
        ..legend(
            "Bebop & Rocksteady",
            cost(&[generic(1), bg(), bg()]),
            vec![CreatureType::Boar, CreatureType::Rhino, CreatureType::Mutant],
            7,
            5,
        )
    }
}

/// Part in Friendship — {4}{G} enchantment. Once each turn, a nontoken
/// creature of yours dying reveals until a creature card: onto the battlefield
/// if its mana value is at most your land count, else to hand; the rest to the
/// bottom in a random order.
pub fn part_in_friendship() -> CardDefinition {
    CardDefinition {
        name: "Part in Friendship",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken })
                .once_per_turn(),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature,
                to: ZoneDest::IfCard {
                    filter: Box::new(R::ManaValueAtMostYourCount(Box::new(R::Land))),
                    then: Box::new(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }),
                    else_: Box::new(ZoneDest::Hand(PlayerRef::You)),
                },
                cap: Value::Const(200),
                life_per_revealed: 0,
                miss_dest: RevealMissDest::BottomRandom,
            },
        }],
        ..Default::default()
    }
}
