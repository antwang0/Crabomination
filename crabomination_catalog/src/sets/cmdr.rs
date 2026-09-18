//! Commander-format cards — the ones whose text is *about* the format
//! (CR 903): commanders that are not legendary creatures, cards that read the
//! command zone, cards that read your commander.
//!
//! Their own file rather than a set file, because what they share is the rule
//! they exercise, not the set they were printed in.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Keyword, LoyaltyAbility,
    PlaneswalkerSubtype, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, StaticAbility,
    StaticEffect, ZoneDest,
};
use crate::mana::{Color, b, cost, g, generic, r, u, w};
use crate::game::TurnStep;

/// Freyalise, Llanowar's Fury — {3}{G}{G} Legendary Planeswalker, loyalty 3.
/// "+2: Create a 1/1 green Elf Druid creature token with '{T}: Add {G}.'
/// −2: Destroy target artifact or enchantment. −6: Draw a card for each green
/// creature you control. Freyalise, Llanowar's Fury can be your commander."
///
/// CR 903.3a — the `can_be_commander` half is what lets a non-creature lead a
/// deck; `format::validate_commander_deck` reads it.
pub fn freyalise_llanowars_fury() -> CardDefinition {
    let elf_druid = TokenDefinition {
        name: "Elf Druid".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        activated_abilities: vec![super::tap_add(Color::Green)],
        ..Default::default()
    };
    CardDefinition {
        name: "Freyalise, Llanowar's Fury",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Freyalise],
            ..Default::default()
        },
        base_loyalty: 3,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(elf_druid),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::count(Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::HasColor(Color::Green)),
                    )),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Edgar Markov — {3}{R}{W}{B} Legendary Creature — Vampire Knight 4/4.
/// "Eminence — Whenever you cast another Vampire spell, if Edgar Markov is in
/// the command zone or on the battlefield, create a 1/1 black Vampire creature
/// token. First strike, haste. Whenever Edgar Markov attacks, put a +1/+1
/// counter on each Vampire you control."
///
/// CR 113.6b — the eminence trigger carries `TriggerZone::CommandZoneToo`, so
/// it functions from the command zone. "Another" needs no filter: while Edgar
/// himself is being cast he is on the stack, which is neither of the two zones
/// the ability names, so his own cast can never see it.
pub fn edgar_markov() -> CardDefinition {
    let vampire = TokenDefinition {
        name: "Vampire".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Edgar Markov",
        cost: cost(&[generic(3), r(), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(R::HasCreatureType(
                        CreatureType::Vampire,
                    )))
                    .in_command_zone(),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(vampire),
                },
            },
            crate::effect::shortcut::on_attack(Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::HasCreatureType(CreatureType::Vampire)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        ],
        ..Default::default()
    }
}

/// Oloro, Ageless Ascetic — {3}{W}{U}{B} Legendary Creature — Giant Soldier
/// 4/5. "At the beginning of your upkeep, you gain 2 life. Whenever you gain
/// life, you may pay {1}. If you do, draw a card and each opponent loses 1
/// life. At the beginning of your upkeep, if Oloro, Ageless Ascetic is in the
/// command zone, you gain 2 life."
///
/// CR 113.6b — the third ability names the command zone and *not* the
/// battlefield, so it is `TriggerZone::CommandZoneOnly`: the two upkeep
/// triggers never both fire. The first two function only on the battlefield,
/// which is why the zone is a property of each ability rather than of the card.
pub fn oloro_ageless_ascetic() -> CardDefinition {
    let gain_two = |zone_only: bool| TriggeredAbility {
        event: {
            let e = EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl);
            if zone_only { e.command_zone_only() } else { e }
        },
        effect: Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        },
    };
    CardDefinition {
        name: "Oloro, Ageless Ascetic",
        cost: cost(&[generic(3), w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Soldier],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![
            gain_two(false),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::MayPay {
                    description: "Pay {1} to draw a card and drain each opponent for 1?".into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::LoseLife {
                            who: Selector::Player(PlayerRef::EachOpponent),
                            amount: Value::ONE,
                        },
                    ])),
                    else_: None,
                },
            },
            gain_two(true),
        ],
        ..Default::default()
    }
}

/// Arahbo, Roar of the World — {3}{G}{W} Legendary Creature — Cat Avatar 5/5.
/// "Eminence — At the beginning of combat on your turn, if Arahbo is in the
/// command zone or on the battlefield, another target Cat you control gets
/// +3/+3 until end of turn. Whenever another Cat you control attacks, you may
/// pay {1}{G}{W}. If you do, it gains trample and gets +X/+X until end of
/// turn, where X is its power."
pub fn arahbo_roar_of_the_world() -> CardDefinition {
    CardDefinition {
        name: "Arahbo, Roar of the World",
        cost: cost(&[generic(3), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                )
                .in_command_zone(),
                effect: Effect::PumpPT {
                    what: target_filtered(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::HasCreatureType(CreatureType::Cat))
                            .and(R::OtherThanSource),
                    ),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Cat).and(R::OtherThanSource),
                    },
                ),
                effect: Effect::MayPay {
                    description: "Pay {1}{G}{W} for trample and +X/+X?".into(),
                    mana_cost: cost(&[generic(1), g(), w()]),
                    body: Box::new(Effect::Seq(vec![
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource,
                            keyword: Keyword::Trample,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::PumpPT {
                            what: Selector::TriggerSource,
                            power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                            toughness: Value::PowerOf(Box::new(Selector::TriggerSource)),
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Yuriko, the Tiger's Shadow — {1}{U}{B} Legendary Creature — Human Ninja
/// 1/3. "Commander ninjutsu {U}{B}. Whenever a Ninja you control deals combat
/// damage to a player, reveal the top card of your library and put that card
/// into your hand. Each opponent loses life equal to that card's mana value."
///
/// CR 702.49d — the commander-ninjutsu keyword is what makes the command zone
/// a legal source; putting her onto the battlefield that way is not a cast, so
/// CR 903.8's tax never applies.
pub fn yuriko_the_tigers_shadow() -> CardDefinition {
    CardDefinition {
        name: "Yuriko, the Tiger's Shadow",
        cost: cost(&[generic(1), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::CommanderNinjutsu(cost(&[u(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            // `dealt_by` is the dealer gate; at resolution `TriggerSource`
            // binds the damaged player, which this body doesn't read.
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .dealt_by(R::HasCreatureType(CreatureType::Ninja)),
            effect: Effect::RevealTopToHandLoseMv {
                who: PlayerRef::EachOpponent,
                you_gain: false,
            },
        }],
        ..Default::default()
    }
}

/// The Ur-Dragon — {4}{W}{U}{B}{R}{G} Legendary Creature — Dragon Avatar
/// 10/10. "Eminence — As long as The Ur-Dragon is in the command zone or on
/// the battlefield, other Dragon spells you cast cost {1} less to cast.
/// Flying. Whenever one or more Dragons you control attack, draw that many
/// cards, then you may put a permanent card from your hand onto the
/// battlefield."
///
/// CR 113.6b — `statics_in_command_zone` is what makes the eminence half
/// function off the battlefield; the flag is card-level and this card's only
/// static is that one. "Other" is a name exclusion, since a cost-reduction
/// filter is evaluated against the cast card with no source in scope.
pub fn the_ur_dragon() -> CardDefinition {
    CardDefinition {
        name: "The Ur-Dragon",
        cost: cost(&[generic(4), w(), u(), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Avatar],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        keywords: vec![Keyword::Flying],
        statics_in_command_zone: true,
        static_abilities: vec![StaticAbility {
            description: "Other Dragon spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::HasCreatureType(CreatureType::Dragon)
                    .and(R::HasName("The Ur-Dragon".into()).negate()),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                })
                .once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::count(Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::HasCreatureType(CreatureType::Dragon))
                            .and(R::IsAttacking),
                    )),
                },
                Effect::MayDo {
                    description: "Put a permanent card from your hand onto the battlefield?"
                        .into(),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::Permanent,
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Minds Aglow — {U} Sorcery. "Join forces — Starting with you, each player
/// may pay any amount of mana. Each player draws X cards, where X is the total
/// amount of mana paid this way."
pub fn minds_aglow() -> CardDefinition {
    CardDefinition {
        name: "Minds Aglow",
        cost: cost(&[u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::JoinForces {
            description: "Join forces — pay any amount of mana; each player draws that many"
                .into(),
            body: Box::new(Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::TriggerEventAmount,
            }),
        },
        ..Default::default()
    }
}

/// Collective Voyage — {G} Sorcery. "Join forces — Starting with you, each
/// player may pay any amount of mana. Each player searches their library for
/// up to X basic land cards, puts them onto the battlefield tapped, then
/// shuffles."
pub fn collective_voyage() -> CardDefinition {
    CardDefinition {
        name: "Collective Voyage",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::JoinForces {
            description: "Join forces — pay any amount of mana; each player ramps that many"
                .into(),
            body: Box::new(Effect::SearchUpToN {
                who: PlayerRef::EachPlayer,
                filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::TriggerEventAmount,
            }),
        },
        ..Default::default()
    }
}

/// Mana-Charged Dragon — {4}{R}{R} Creature — Dragon 5/5. "Flying, trample.
/// Join forces — Whenever this creature attacks or blocks, each player
/// starting with you may pay any amount of mana. This creature gets +X/+0
/// until end of turn, where X is the total amount of mana paid this way."
pub fn mana_charged_dragon() -> CardDefinition {
    let join = |what: &'static str| Effect::JoinForces {
        description: format!("Join forces — pay any amount of mana; the Dragon gets +X/+0 ({what})"),
        body: Box::new(Effect::PumpPT {
            what: Selector::This,
            power: Value::TriggerEventAmount,
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        }),
    };
    CardDefinition {
        name: "Mana-Charged Dragon",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![
            crate::effect::shortcut::on_attack(join("attacks")),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: join("blocks"),
            },
        ],
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CR 903.3a — the flag is what makes a non-creature eligible.
    #[test]
    fn freyalise_prints_can_be_your_commander() {
        let def = freyalise_llanowars_fury();
        assert!(def.can_be_commander);
        assert!(def.is_legendary() && !def.is_creature());
        assert_eq!(def.base_loyalty, 3);
        assert_eq!(def.loyalty_abilities.len(), 3);
    }
}
