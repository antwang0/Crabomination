//! A fifth wave of staples — modern/cube cards that filled remaining gaps
//! (Toski, Venser, Hullbreaker Horror, the green draw-X package, Birthing-Pod
//! style tutors, …). Each card has a functionality test in
//! `crabomination/src/tests/recent5.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, phyrexian, u, w};

/// Plaguecrafter — {2}{B} 3/2. ETB: each player sacrifices a creature or
/// planeswalker. (The "each player who can't, discards" rider is dropped.)
pub fn plaguecrafter() -> CardDefinition {
    CardDefinition {
        name: "Plaguecrafter",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
        })],
        ..Default::default()
    }
}

/// Wither and Bloom — {1}{B} Instant. Target creature gets -3/-3. `{1}{B},
/// Exile this from your graveyard`: put a +1/+1 counter on target creature you
/// control (sorcery speed).
pub fn wither_and_bloom() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Wither and Bloom",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-3),
            toughness: Value::Const(-3),
            duration: Duration::EndOfTurn,
        },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sythis, Harvest's Hand — {G}{W} 1/2 Nymph. Whenever you cast an enchantment
/// spell, gain 1 life and draw a card.
pub fn sythis_harvests_hand() -> CardDefinition {
    CardDefinition {
        name: "Sythis, Harvest's Hand",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nymph],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Toski, Bearer of Secrets — {3}{G} 1/1 Squirrel. Can't be countered;
/// indestructible; attacks each combat if able; whenever a creature you control
/// deals combat damage to a player, draw a card.
pub fn toski_bearer_of_secrets() -> CardDefinition {
    CardDefinition {
        name: "Toski, Bearer of Secrets",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![
            Keyword::CantBeCountered,
            Keyword::Indestructible,
            Keyword::MustAttack,
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            ),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Misdirection — {3}{U}{U} Instant. Pitch a blue card from hand instead of
/// paying. Change the target of target spell with a single target.
pub fn misdirection() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Misdirection",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChangeSpellTarget {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        },
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            exile_filter: Some(SelectionRequirement::HasColor(Color::Blue)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Flawless Maneuver — {2}{W} Instant. Creatures you control gain
/// indestructible until end of turn. (The free-if-you-control-a-commander
/// alternative cost is dropped.)
pub fn flawless_maneuver() -> CardDefinition {
    CardDefinition {
        name: "Flawless Maneuver",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeyword {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            keyword: Keyword::Indestructible,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Venser, Shaper Savant — {2}{U}{U} 2/2 Wizard, Flash. ETB: return target
/// permanent to its owner's hand. (Printed "spell or permanent"; modeled as a
/// permanent, like the engine's other spell-or-permanent bounces.)
pub fn venser_shaper_savant() -> CardDefinition {
    CardDefinition {
        name: "Venser, Shaper Savant",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Permanent),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..Default::default()
    }
}

/// Hullbreaker Horror — {5}{U}{U} 7/8 Kraken, Flash, can't be countered.
/// Whenever you cast a spell, you may return target nonland permanent to its
/// owner's hand. (The "return target spell you don't control" mode is dropped.)
pub fn hullbreaker_horror() -> CardDefinition {
    CardDefinition {
        name: "Hullbreaker Horror",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kraken, CreatureType::Horror],
            ..Default::default()
        },
        power: 7,
        toughness: 8,
        keywords: vec![Keyword::Flash, Keyword::CantBeCountered],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Return target nonland permanent to its owner's hand".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Drown in Sorrow — {1}{B}{B} Sorcery. All creatures get -2/-2 until end of
/// turn. Scry 1.
pub fn drown_in_sorrow() -> CardDefinition {
    CardDefinition {
        name: "Drown in Sorrow",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Shamanic Revelation — {3}{G}{G} Sorcery. Draw a card for each creature you
/// control. Ferocious — you gain 4 life for each creature you control with
/// power 4 or greater.
pub fn shamanic_revelation() -> CardDefinition {
    let mine = || SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou);
    CardDefinition {
        name: "Shamanic Revelation",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(mine())),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    mine().and(SelectionRequirement::PowerAtLeast(4)),
                ),
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(4),
                }),
            },
        ]),
        ..Default::default()
    }
}

/// End-Raze Forerunners — {5}{G}{G}{G} 7/7, Vigilance, trample, haste. ETB:
/// other creatures you control get +2/+2 and gain vigilance and trample until
/// end of turn.
pub fn end_raze_forerunners() -> CardDefinition {
    let others = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(SelectionRequirement::OtherThanSource),
        )
    };
    CardDefinition {
        name: "End-Raze Forerunners",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Boar],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Vigilance, Keyword::Trample, Keyword::Haste],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::PumpPT {
                what: others(),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: others(),
                keyword: Keyword::Vigilance,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: others(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Garruk's Uprising — {2}{G} Enchantment. ETB: if you control a creature with
/// power 4+, draw. Creatures you control have trample. Whenever a creature you
/// control with power 4+ enters, draw a card.
pub fn garruks_uprising() -> CardDefinition {
    let power4 = || SelectionRequirement::Creature.and(SelectionRequirement::PowerAtLeast(4));
    CardDefinition {
        name: "Garruk's Uprising",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Trample,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(power4()),
                        n: Value::ONE,
                    },
                    then: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: power4(),
                    }),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Guardian Project — {3}{G} Enchantment. Whenever a nontoken creature you
/// control enters, draw a card. (The same-name exclusion is dropped.)
pub fn guardian_project() -> CardDefinition {
    CardDefinition {
        name: "Guardian Project",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::Not(
                        Box::new(SelectionRequirement::IsToken),
                    )),
                }),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Neoform — {G}{U} Sorcery. Additional cost: sacrifice a creature. Search your
/// library for a creature card with mana value equal to 1 plus the sacrificed
/// creature's, put it onto the battlefield with a +1/+1 counter, then shuffle.
pub fn neoform() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Neoform",
        cost: cost(&[g(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueEqualsSacrificedPlus(1)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            // The searched creature is the only one to enter this turn under a
            // normal Neoform cast (the sacrifice happened first).
            Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::EnteredThisTurn),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Eldritch Evolution — {1}{G}{G} Sorcery. Additional cost: sacrifice a
/// creature. Search your library for a creature card with mana value 2 or less
/// plus the sacrificed creature's, put it onto the battlefield, then shuffle.
/// Exile Eldritch Evolution.
pub fn eldritch_evolution() -> CardDefinition {
    CardDefinition {
        name: "Eldritch Evolution",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMostSacrificedPlus(2)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::ExileResolvingSpell,
        ]),
        ..Default::default()
    }
}

/// Skrelv, Defector Mite — {W} 1/1 Phyrexian Mite. Toxic 1; can't block.
/// `{W/P}, {T}`: another target creature you control gains hexproof until end
/// of turn. (The toxic / unblockable-by-color grant is simplified to hexproof.)
pub fn skrelv_defector_mite() -> CardDefinition {
    CardDefinition {
        name: "Skrelv, Defector Mite",
        cost: cost(&[w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Toxic(1), Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::new(vec![phyrexian(Color::White)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soul's Majesty — {3}{G}{G} Sorcery. Draw cards equal to the power of target
/// creature you control.
pub fn souls_majesty() -> CardDefinition {
    CardDefinition {
        name: "Soul's Majesty",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::PowerOf(Box::new(Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            })),
        },
        ..Default::default()
    }
}

/// Momentous Fall — {1}{G}{G} Instant. As an additional cost, sacrifice a
/// creature. Draw cards equal to its power, then gain life equal to its
/// toughness.
pub fn momentous_fall() -> CardDefinition {
    CardDefinition {
        name: "Momentous Fall",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::SacrificedPower,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::SacrificedToughness,
            },
        ]),
        ..Default::default()
    }
}

/// Life's Legacy — {1}{G} Sorcery. As an additional cost, sacrifice a creature.
/// Draw cards equal to its power.
pub fn lifes_legacy() -> CardDefinition {
    CardDefinition {
        name: "Life's Legacy",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::SacrificedPower,
            },
        ]),
        ..Default::default()
    }
}

/// Return of the Wildspeaker — {4}{G} Sorcery. Choose one — draw cards equal to
/// the greatest power among creatures you control; or creatures you control get
/// +3/+3 until end of turn.
pub fn return_of_the_wildspeaker() -> CardDefinition {
    CardDefinition {
        name: "Return of the Wildspeaker",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::PowerOf(Box::new(Selector::GreatestPowerYouControl)),
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Overrun — {2}{G}{G}{G} Sorcery. Creatures you control get +3/+3 and gain
/// trample until end of turn.
pub fn overrun() -> CardDefinition {
    let mine = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        )
    };
    CardDefinition {
        name: "Overrun",
        cost: cost(&[generic(2), g(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: mine(),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: mine(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Larger Than Life — {1}{G} Sorcery. Target creature gets +4/+4 and gains
/// trample until end of turn.
pub fn larger_than_life() -> CardDefinition {
    CardDefinition {
        name: "Larger Than Life",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(4),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Prey's Vengeance — {G} Instant. Target creature gets +2/+2 until end of
/// turn. Rebound.
pub fn preys_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Prey's Vengeance",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Rebound],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Savage Smash — {1}{R}{G} Sorcery. Target creature you control gets +2/+2,
/// then fights target creature you don't control.
pub fn savage_smash() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Savage Smash",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Bite Down — {1}{G} Instant. Target creature you control deals damage equal
/// to its power to target creature or planeswalker you don't control. (Modeled
/// as the spell dealing the damage.)
pub fn bite_down() -> CardDefinition {
    CardDefinition {
        name: "Bite Down",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
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

/// Crushing Vines — {2}{G} Instant. Choose one — destroy target creature with
/// flying; or destroy target artifact.
pub fn crushing_vines() -> CardDefinition {
    CardDefinition {
        name: "Crushing Vines",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Artifact),
            },
        ]),
        ..Default::default()
    }
}

/// Inspiring Call — {2}{G} Instant. Draw a card for each creature you control
/// with a +1/+1 counter on it; those creatures gain indestructible until end
/// of turn.
pub fn inspiring_call() -> CardDefinition {
    use crate::card::CounterType;
    let countered = || {
        SelectionRequirement::Creature
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::WithCounter(
                CounterType::PlusOnePlusOne,
            ))
    };
    CardDefinition {
        name: "Inspiring Call",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::count(Selector::EachPermanent(countered())),
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(countered()),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}
