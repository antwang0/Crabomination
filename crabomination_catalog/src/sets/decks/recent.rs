//! Recent-set staples (MH3 / BLB / DSK / OTJ / FDN / DFT / TDM …) that fill
//! gaps in the Modern-playable pool. Each card has at least one functionality
//! test in `crabomination/src/tests/recent.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, Predicate, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, colorless, cost, g, generic, r, u, w};

/// Questing Beast — {2}{G}{G} 4/4 Legendary Beast. Vigilance, deathtouch,
/// haste; can't be blocked by creatures with power 2 or less; combat damage
/// dealt by creatures you control can't be prevented. (The planeswalker-redirect
/// rider is omitted — planeswalkers aren't attack targets yet.)
pub fn questing_beast() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Questing Beast",
        cost: cost(&[generic(2), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Deathtouch,
            Keyword::Haste,
            Keyword::CantBeBlockedByPowerAtMost(2),
        ],
        static_abilities: vec![StaticAbility {
            description: "Combat damage that would be dealt by creatures you control can't be prevented.",
            effect: StaticEffect::ControllerCreaturesCombatDamageCantBePrevented,
        }],
        ..Default::default()
    }
}

/// Cackling Slasher — {3}{B} 3/3 Human Assassin. Deathtouch; enters with a
/// +1/+1 counter if a creature died this turn.
pub fn cackling_slasher() -> CardDefinition {
    CardDefinition {
        name: "Cackling Slasher",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Assassin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
            then: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Vaultborn Tyrant — {5}{G}{G} 6/6 Dinosaur. Trample. Whenever this or
/// another creature you control with power 4+ enters, gain 3 life and draw.
/// When it dies (if not a token), create a token copy of it.
pub fn vaultborn_tyrant() -> CardDefinition {
    CardDefinition {
        name: "Vaultborn Tyrant",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::PowerOf(Box::new(Selector::TriggerSource)),
                        Value::Const(4),
                    )),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                ]),
            },
            // The token copy "is an artifact in addition" rider is omitted —
            // `CreateTokenCopyOf` doesn't add card types.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::NotToken,
                    }),
                effect: Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    override_pt: None,
                    non_legendary: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Emberheart Challenger — {1}{R} 2/2 Mouse Warrior. Haste, prowess; Valiant
/// — the first time it becomes the target of your spell/ability each turn,
/// exile the top card of your library; you may play it this turn.
pub fn emberheart_challenger() -> CardDefinition {
    CardDefinition {
        name: "Emberheart Challenger",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            crate::effect::shortcut::prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl)
                    .once_per_turn(),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Eldrazi Linebreaker — {1}{C}{R} 3/3 Eldrazi. Devoid, trample. At the
/// beginning of combat on your turn, target creature you control gains haste
/// and gets +X/+0 until end of turn, where X is the number of Eldrazi you
/// control.
pub fn eldrazi_linebreaker() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Linebreaker",
        cost: cost(&[generic(1), colorless(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Devoid, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::CountMatching {
                        sel: Box::new(Selector::EachPermanent(
                            SelectionRequirement::ControlledByYou,
                        )),
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Eldrazi),
                    },
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// No More Lies — {W}{U} Instant. Counter target spell unless its controller
/// pays {3}. If countered this way, exile it instead.
pub fn no_more_lies() -> CardDefinition {
    CardDefinition {
        name: "No More Lies",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(3)]),
            exile: true,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Unstoppable Slasher — {2}{B} 2/3 Zombie Assassin. Deathtouch; when it deals
/// combat damage to a player, they lose half their life, rounded up. When it
/// dies, return it tapped with two stun counters under its owner's control.
pub fn unstoppable_slasher() -> CardDefinition {
    CardDefinition {
        name: "Unstoppable Slasher",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::LoseHalfLife {
                    who: Selector::Player(PlayerRef::Target(0)),
                    rounded_up: true,
                },
            },
            on_dies(Effect::ReturnSelfTappedWithCounters {
                kind: CounterType::Stun,
                amount: 2,
            }),
        ],
        ..Default::default()
    }
}

/// Enduring Curiosity — {2}{U}{U} 4/3 Cat Glimmer enchantment creature. Flash;
/// whenever a creature you control deals combat damage to a player, draw a
/// card. When it dies, return it to the battlefield as an enchantment.
pub fn enduring_curiosity() -> CardDefinition {
    CardDefinition {
        name: "Enduring Curiosity",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Glimmer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::YourControl,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
            on_dies(Effect::ReturnSelfAsEnchantment),
        ],
        ..Default::default()
    }
}

/// Galvanic Relay — {2}{R} Sorcery. Exile the top card of your library; you
/// may play it during your next turn. Storm.
pub fn galvanic_relay() -> CardDefinition {
    CardDefinition {
        name: "Galvanic Relay",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Storm],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(1),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        },
        ..Default::default()
    }
}

/// The Necrobloom — {1}{W}{B}{G} 2/7 Legendary Plant. Landfall — whenever a
/// land you control enters, create a 0/1 green Plant token; if you control 7+
/// lands with different names, a 2/2 Zombie instead. (The "lands in your
/// graveyard have dredge 2" static is omitted.)
pub fn the_necrobloom() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let plant = TokenDefinition {
        name: "Plant".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "The Necrobloom",
        cost: cost(&[generic(1), w(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 2,
        toughness: 7,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: plant },
        }],
        ..Default::default()
    }
}

/// Tyvar's Stand — {X}{G} Instant. Target creature you control gets +X/+X and
/// gains hexproof and indestructible until end of turn.
pub fn tyvars_stand() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Tyvar's Stand",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::XFromCost,
                toughness: Value::XFromCost,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Gird for Battle — {W} Sorcery. Put a +1/+1 counter on each of up to two
/// target creatures.
pub fn gird_for_battle() -> CardDefinition {
    CardDefinition {
        name: "Gird for Battle",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            filter: SelectionRequirement::Creature,
            effect: Box::new(Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}

/// Stock Up — {2}{U} Sorcery. Look at the top five cards of your library, put
/// two into your hand and the rest on the bottom.
pub fn stock_up() -> CardDefinition {
    CardDefinition {
        name: "Stock Up",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(5),
            rest_to_graveyard: false,
            pick_filter: None,
            take: Some(Value::Const(2)),
            to_battlefield: false,
        },
        ..Default::default()
    }
}

/// Shelter — {1}{W} Instant. Target creature you control gains protection from
/// the color of your choice until end of turn. Draw a card.
pub fn shelter() -> CardDefinition {
    CardDefinition {
        name: "Shelter",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantProtectionFromChosenColor {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Pick Your Poison — {G} Sorcery. Choose one — each opponent sacrifices an
/// artifact / an enchantment / a creature with flying, their choice.
pub fn pick_your_poison() -> CardDefinition {
    let edict = |filter: SelectionRequirement| Effect::Sacrifice {
        who: Selector::Player(PlayerRef::EachOpponent),
        count: Value::Const(1),
        filter,
    };
    CardDefinition {
        name: "Pick Your Poison",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            edict(SelectionRequirement::Artifact),
            edict(SelectionRequirement::Enchantment),
            edict(SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying))),
        ]),
        ..Default::default()
    }
}

/// Tail Swipe — {G} Instant. Target creature you control fights target creature
/// you don't control; if cast in your main phase, yours gets +1/+1 until end of
/// turn first.
pub fn tail_swipe() -> CardDefinition {
    use crate::effect::Predicate;
    use crate::game::TurnStep;
    let attacker = Selector::TargetFiltered {
        slot: 0,
        filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    };
    CardDefinition {
        name: "Tail Swipe",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::Any(vec![
                        Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                        Predicate::CurrentStepIs(TurnStep::PostCombatMain),
                    ]),
                ]),
                then: Box::new(Effect::PumpPT {
                    what: attacker.clone(),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight {
                attacker,
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

/// Lightning Axe — {R} Instant. (As an additional cost, discard a card.)
/// Deals 5 damage to target creature. (The "or pay {5}" alternative is
/// omitted — the discard is taken at resolution, Deadly-Dispute style.)
pub fn lightning_axe() -> CardDefinition {
    CardDefinition {
        name: "Lightning Axe",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(5),
            },
        ]),
        ..Default::default()
    }
}

/// Stormsplitter — {3}{R} 1/4 Otter Wizard. Haste. Whenever you cast an instant
/// or sorcery spell, create a token copy of this creature; exile it at the
/// beginning of the next end step.
pub fn stormsplitter() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        name: "Stormsplitter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
            ),
            effect: Effect::Seq(vec![
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    override_pt: None,
                    non_legendary: false,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Exile { what: Selector::LastCreatedToken }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Unburden — {1}{B}{B} Sorcery. Target player discards two cards. Cycling {2}.
pub fn unburden() -> CardDefinition {
    CardDefinition {
        name: "Unburden",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Const(2),
            random: false,
        },
        ..Default::default()
    }
}

/// Goblin Anarchomancer — {R}{G} 2/2 Goblin Shaman. Each red or green spell you
/// cast costs {1} less to cast.
pub fn goblin_anarchomancer() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::mana::Color;
    CardDefinition {
        name: "Goblin Anarchomancer",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Red or green spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasColor(Color::Red)
                    .or(SelectionRequirement::HasColor(Color::Green)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Beza, the Bounding Spring — {2}{W}{W} 4/5 Legendary Elemental Elk. ETB: a
/// Treasure if an opponent has more lands; gain 4 if an opponent has more life;
/// two 1/1 Fish if an opponent has more creatures; draw if an opponent has more
/// cards in hand.
pub fn beza_the_bounding_spring() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let if_then = |cond: Predicate, then: Effect| Effect::If {
        cond,
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    };
    let fish = TokenDefinition {
        name: "Fish".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Fish], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Beza, the Bounding Spring",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Elk],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            if_then(
                Predicate::OpponentControlsMoreLandsThanYou,
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
            ),
            if_then(
                Predicate::AnOpponentHasMoreLife,
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ),
            if_then(
                Predicate::AnOpponentControlsMoreCreatures,
                Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: fish },
            ),
            if_then(
                Predicate::AnOpponentHasMoreCardsInHand,
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ),
        ]))],
        ..Default::default()
    }
}

/// Optimistic Scavenger — {W} 1/1 Human Scout. Eerie — whenever an enchantment
/// you control enters, put a +1/+1 counter on target creature. (The
/// fully-unlock-a-Room half is omitted.)
pub fn optimistic_scavenger() -> CardDefinition {
    CardDefinition {
        name: "Optimistic Scavenger",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Frilled Sandwalla — {G} 1/1 Lizard. {1}{G}: +2/+2 until end of turn,
/// once each turn.
pub fn frilled_sandwalla() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Frilled Sandwalla",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lizard], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spell Stutter — {1}{U} Instant. Counter target spell unless its controller
/// pays {2} plus {1} for each Faerie you control.
pub fn spell_stutter() -> CardDefinition {
    CardDefinition {
        name: "Spell Stutter",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
            mana_cost: cost(&[generic(2)]),
            exile: false,
            extra_generic: Some(Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
                filter: SelectionRequirement::HasCreatureType(CreatureType::Faerie),
            }),
        },
        ..Default::default()
    }
}

/// Spectral Interference — {1}{U} Instant. Counter target artifact or creature
/// spell unless its controller pays {4}.
pub fn spectral_interference() -> CardDefinition {
    CardDefinition {
        name: "Spectral Interference",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(
                SelectionRequirement::IsSpellOnStack.and(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::HasCardType(CardType::Creature)),
                ),
            ),
            mana_cost: cost(&[generic(4)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Refute — {1}{U}{U} Instant. Counter target spell. Draw a card, then discard.
pub fn refute() -> CardDefinition {
    CardDefinition {
        name: "Refute",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(SelectionRequirement::IsSpellOnStack) },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        ]),
        ..Default::default()
    }
}

/// Skullcap Snail — {1}{B} 1/1 Fungus Snail. ETB: target opponent exiles a
/// card from their hand.
pub fn skullcap_snail() -> CardDefinition {
    CardDefinition {
        name: "Skullcap Snail",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fungus, CreatureType::Snail],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::ExileFromHand {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Aspirant's Ascent — {U} Instant. Target creature gets +1/+3 and gains flying
/// and toxic 1 until end of turn.
pub fn aspirants_ascent() -> CardDefinition {
    CardDefinition {
        name: "Aspirant's Ascent",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(1),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Toxic(1),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Take the Fall — {U} Instant. Target creature gets -1/-0 (or -4/-0 if you
/// control an outlaw) until end of turn. Draw a card.
pub fn take_the_fall() -> CardDefinition {
    let outlaw = SelectionRequirement::Creature
        .and(SelectionRequirement::ControlledByYou)
        .and(
            SelectionRequirement::HasCreatureType(CreatureType::Assassin)
                .or(SelectionRequirement::HasCreatureType(CreatureType::Mercenary))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Pirate))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Rogue))
                .or(SelectionRequirement::HasCreatureType(CreatureType::Warlock)),
        );
    CardDefinition {
        name: "Take the Fall",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(outlaw),
                    n: Value::Const(1),
                },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(-3),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Hopeful Vigil — {1}{W} Enchantment. ETB: create a 2/2 white Knight with
/// vigilance. When it leaves the battlefield, scry 2. {2}{W}: sacrifice it.
pub fn hopeful_vigil() -> CardDefinition {
    use crate::card::{ActivatedAbility, TokenDefinition};
    use crate::mana::Color;
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    CardDefinition {
        name: "Hopeful Vigil",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: knight }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hopeless Nightmare — {B} Enchantment. ETB: each opponent discards a card and
/// loses 2 life. When it leaves the battlefield, scry 2. {2}{B}: sacrifice it.
pub fn hopeless_nightmare() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Hopeless Nightmare",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::Noop,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hangar Scrounger — {2}{R} 2/1 Dwarf Pilot. Backup 1. Whenever it becomes
/// tapped, you may discard a card; if you do, draw a card. (The backup-grant
/// of the loot ability to the backed-up creature is omitted.)
pub fn hangar_scrounger() -> CardDefinition {
    let loot = TriggeredAbility {
        event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
        effect: Effect::MayDo {
            description: "discard a card, then draw a card".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ])),
        },
    };
    CardDefinition {
        name: "Hangar Scrounger",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Pilot],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::backup(1, vec![]), loot],
        ..Default::default()
    }
}

/// Bristlebud Farmer — {2}{G}{G} 5/5 Plant Druid. Trample. ETB: create two
/// Food tokens. (The attack "sac a Food → mill three, grab a permanent" rider
/// is omitted.)
pub fn bristlebud_farmer() -> CardDefinition {
    CardDefinition {
        name: "Bristlebud Farmer",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Druid],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Outcaster Greenblade — {2}{G} 1/2 Human Mercenary. ETB: search your library
/// for a basic land or Desert card and put it into your hand. Gets +1/+1 for
/// each Desert you control.
pub fn outcaster_greenblade() -> CardDefinition {
    use crate::card::{DynamicPt, LandType};
    CardDefinition {
        name: "Outcaster Greenblade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled {
            land_type: LandType::Desert,
            base_p: 1,
            base_t: 2,
        }),
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand
                .or(SelectionRequirement::HasLandType(LandType::Desert)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Mizzium Skin — {U} Instant. Target creature you control gets +0/+1 and gains
/// hexproof until end of turn. Overload {3}{U}: each creature you control
/// instead.
pub fn mizzium_skin() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Mizzium Skin",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(0),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                duration: Duration::EndOfTurn,
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(3), u()]),
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::Const(0),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Hexproof,
                        duration: Duration::EndOfTurn,
                    },
                ])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Demand Answers — {1}{R} Instant. (As an additional cost, discard a card —
/// the "sacrifice an artifact" alternative is omitted.) Draw two cards.
pub fn demand_answers() -> CardDefinition {
    CardDefinition {
        name: "Demand Answers",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Boltwave — {R} Sorcery. Deals 3 damage to each opponent.
pub fn boltwave() -> CardDefinition {
    CardDefinition {
        name: "Boltwave",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Strike It Rich — {R} Sorcery. Create a Treasure token. Flashback {2}{R}.
pub fn strike_it_rich() -> CardDefinition {
    CardDefinition {
        name: "Strike It Rich",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::game::effects::treasure_token(),
        },
        ..Default::default()
    }
}

/// Brotherhood's End — {1}{R}{R} Sorcery. Choose one — 3 damage to each
/// creature and planeswalker; or destroy all artifacts with mana value 3 or
/// less.
pub fn brotherhoods_end() -> CardDefinition {
    CardDefinition {
        name: "Brotherhood's End",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
                ),
                amount: Value::Const(3),
            },
            Effect::Destroy {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact.and(SelectionRequirement::ManaValueAtMost(3)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Boon-Bringer Valkyrie — {3}{W}{W} 4/4 Angel Warrior. Flying, first strike,
/// lifelink. Backup 1 (grants those abilities to the backed-up creature).
pub fn boon_bringer_valkyrie() -> CardDefinition {
    CardDefinition {
        name: "Boon-Bringer Valkyrie",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        triggered_abilities: vec![crate::effect::shortcut::backup(
            1,
            vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Lifelink],
        )],
        ..Default::default()
    }
}

/// Inti, Seneschal of the Sun — {1}{R} 2/2 Legendary Human Knight. Whenever you
/// attack, you may discard a card to put a +1/+1 counter on target attacking
/// creature and give it trample. Whenever you discard a card, exile the top of
/// your library; you may play it until your next end step. ("Whenever you
/// attack" is approximated as once per turn.)
pub fn inti_seneschal_of_the_sun() -> CardDefinition {
    CardDefinition {
        name: "Inti, Seneschal of the Sun",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).once_per_turn(),
                effect: Effect::MayDo {
                    description: "Discard a card to grow a target attacking creature".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                        Effect::AddCounter {
                            what: target_filtered(SelectionRequirement::IsAttacking),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::Const(1),
                        },
                        Effect::GrantKeyword {
                            what: Selector::Target(0),
                            keyword: Keyword::Trample,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    pay_any_color: false,
                    uncast_penalty: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Warren Soultrader — {2}{B} 3/3 Zombie Goblin Wizard. Pay 1 life, Sacrifice
/// another creature: Create a Treasure token.
pub fn warren_soultrader() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Warren Soultrader",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hostile Investigator — {3}{B} 4/3 Ogre Rogue Detective. When it enters,
/// target opponent discards a card. Whenever one or more players discard one or
/// more cards, investigate (once each turn).
pub fn hostile_investigator() -> CardDefinition {
    CardDefinition {
        name: "Hostile Investigator",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Rogue, CreatureType::Detective],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::AnyPlayer)
                    .once_per_turn(),
                effect: crate::effect::shortcut::investigate(1),
            },
        ],
        ..Default::default()
    }
}

/// Marshal of Zhalfir — {W}{U} 2/2 Human Knight. Other Knights you control get
/// +1/+1. {W}{U}, {T}: Tap another target creature.
pub fn marshal_of_zhalfir() -> CardDefinition {
    use crate::card::{ActivatedAbility, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Marshal of Zhalfir",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Knights you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource)
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Knight)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u()]),
            tap_cost: true,
            effect: Effect::Tap {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pawpatch Recruit — {G} 2/1 Rabbit Warrior with trample. Offspring {2}.
/// Whenever a creature you control becomes the target of a spell or ability an
/// opponent controls, put a +1/+1 counter on target creature you control other
/// than that creature.
pub fn pawpatch_recruit() -> CardDefinition {
    CardDefinition {
        name: "Pawpatch Recruit",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rabbit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Offspring(cost(&[generic(2)]))],
        triggered_abilities: vec![
            // Offspring (CR 702.166): if its cost was paid, mint a 1/1 copy.
            etb(Effect::If {
                cond: Predicate::SpellWasKicked,
                then: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    override_pt: Some((1, 1)),
                    non_legendary: false,
                }),
                else_: Box::new(Effect::Noop),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::BecameTarget,
                    EventScope::YourPermanentTargetedByOpponent,
                ),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Helping Hand — {W} Sorcery. Return target creature card with mana value 3 or
/// less from your graveyard to the battlefield tapped.
pub fn helping_hand() -> CardDefinition {
    CardDefinition {
        name: "Helping Hand",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(3))
                    .and(SelectionRequirement::InYourGraveyard),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Diversion Unit — {1}{U} 2/1 Robot artifact creature with flying. {U},
/// Sacrifice this creature: Counter target instant or sorcery spell unless its
/// controller pays {3}.
pub fn diversion_unit() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Diversion Unit",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Robot], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                ),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Final Vengeance — {B} Sorcery. As an additional cost, sacrifice a creature
/// or enchantment. Exile target creature.
pub fn final_vengeance() -> CardDefinition {
    CardDefinition {
        name: "Final Vengeance",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: (SelectionRequirement::Creature.or(SelectionRequirement::Enchantment))
                .and(SelectionRequirement::ControlledByYou),
            count: 1,
        }],
        effect: Effect::Exile { what: target_filtered(SelectionRequirement::Creature) },
        ..Default::default()
    }
}

/// Roughshod Mentor — {5}{G} 5/4 Giant Warrior. Green creatures you control
/// have trample.
pub fn roughshod_mentor() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    use crate::mana::Color;
    CardDefinition {
        name: "Roughshod Mentor",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Green creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasColor(Color::Green)),
                ),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Innocuous Rat — {1}{B} 1/1 Rat. When it dies, manifest dread.
pub fn innocuous_rat() -> CardDefinition {
    CardDefinition {
        name: "Innocuous Rat",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::ManifestDread { who: PlayerRef::You })],
        ..Default::default()
    }
}

/// Quaketusk Boar — {3}{R}{R} 5/5 Elemental Boar with reach, trample, haste.
pub fn quaketusk_boar() -> CardDefinition {
    CardDefinition {
        name: "Quaketusk Boar",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Boar],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Haste],
        ..Default::default()
    }
}

/// Veteran Guardmouse — {3}{R/W} 3/4 Mouse Soldier. Valiant — the first time it
/// becomes the target of a spell or ability you control each turn, it gets
/// +1/+0 and gains first strike until end of turn, then scry 1.
pub fn veteran_guardmouse() -> CardDefinition {
    use crate::mana::hybrid;
    use crate::mana::Color::{Red, White};
    CardDefinition {
        name: "Veteran Guardmouse",
        cost: cost(&[generic(3), hybrid(Red, White)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Mouse, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YourControl).once_per_turn(),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::FirstStrike,
                    duration: Duration::EndOfTurn,
                },
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Polliwallop — {3}{G} Instant. Affinity for Frogs. Target creature you
/// control deals damage equal to twice its power to target creature you don't
/// control. (Damage is dealt by the spell rather than the creature.)
pub fn polliwallop() -> CardDefinition {
    CardDefinition {
        name: "Polliwallop",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Frog)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
            amount: Value::Times(
                Box::new(Value::PowerOf(Box::new(Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                }))),
                Box::new(Value::Const(2)),
            ),
        },
        ..Default::default()
    }
}

/// Coiling Rebirth — {3}{B}{B} Sorcery. Gift a card. Return target creature card
/// from your graveyard to the battlefield. If the gift was promised and that
/// creature isn't legendary, also create a 1/1 token copy of it.
pub fn coiling_rebirth() -> CardDefinition {
    use crate::card::Gift;
    let reanimate = Effect::Move {
        what: Selector::TargetFiltered {
            slot: 0,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
        },
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Coiling Rebirth",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: reanimate.clone(),
        gift: Some(Box::new(Gift {
            label: "a card",
            gifted_effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(1) },
                reanimate,
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::Target(0),
                    extra_creature_types: vec![],
                    override_pt: Some((1, 1)),
                    non_legendary: true,
                },
            ]),
        })),
        ..Default::default()
    }
}

/// Pearl of Wisdom — {2}{U} Sorcery. Costs {1} less if you control an Otter.
/// Draw two cards.
pub fn pearl_of_wisdom() -> CardDefinition {
    CardDefinition {
        name: "Pearl of Wisdom",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        self_cost_reduction_if_control: Some((
            SelectionRequirement::HasCreatureType(CreatureType::Otter)
                .and(SelectionRequirement::ControlledByYou),
            1,
        )),
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Ride's End — {4}{W} Instant. Costs {3} less to cast if it targets a tapped
/// permanent. Exile target creature or Vehicle.
pub fn rides_end() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    CardDefinition {
        name: "Ride's End",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Instant],
        self_cost_reduction_if_target: Some((SelectionRequirement::Tapped, 3)),
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature
                    .or(SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            ),
        },
        ..Default::default()
    }
}

/// Nurturing Pixie — {W} 1/1 Faerie Rogue with flying. When it enters, return
/// up to one target non-Faerie, nonland permanent you control to its owner's
/// hand; if one was returned, put a +1/+1 counter on this creature.
pub fn nurturing_pixie() -> CardDefinition {
    CardDefinition {
        name: "Nurturing Pixie",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Bounce a nonland permanent you control to grow the Pixie".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Permanent
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::Not(Box::new(
                                SelectionRequirement::HasCreatureType(CreatureType::Faerie),
                            ))),
                    },
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            ])),
        })],
        ..Default::default()
    }
}

/// Stab — {B} Instant. Target creature gets -2/-2 until end of turn.
pub fn stab() -> CardDefinition {
    CardDefinition {
        name: "Stab",
        cost: cost(&[b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-2),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Slumbering Keepguard — {W} 1/1 Human Knight. Whenever an enchantment you
/// control enters, scry 1. {2}{W}: This creature gets +1/+1 until end of turn
/// for each enchantment you control.
pub fn slumbering_keepguard() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let enchant_count = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(SelectionRequirement::ControlledByYou)),
        filter: SelectionRequirement::Enchantment,
    };
    CardDefinition {
        name: "Slumbering Keepguard",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                }),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: enchant_count.clone(),
                toughness: enchant_count,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ruby, Daring Tracker — {R}{G} 1/2 Legendary Human Scout with haste. Whenever
/// Ruby attacks while you control a creature with power 4 or greater, it gets
/// +2/+2 until end of turn. {T}: Add {R} or {G}.
pub fn ruby_daring_tracker() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Ruby, Daring Tracker",
        cost: cost(&[r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(4)),
                ),
                n: Value::Const(1),
            },
            then: Box::new(Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            crate::sets::tap_add(Color::Green),
        ],
        ..Default::default()
    }
}


/// Anoint with Affliction — {1}{B} Instant. Exile target creature with mana
/// value 3 or less. (The Corrupted "any creature if its controller has 3+
/// poison" rider is dropped; the base mode caps the target at MV 3.)
pub fn anoint_with_affliction() -> CardDefinition {
    CardDefinition {
        name: "Anoint with Affliction",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Wing It — {1}{W} Instant. Target creature gets +2/+2 until end of turn, gets
/// a flying counter, then scry 1.
pub fn wing_it() -> CardDefinition {
    CardDefinition {
        name: "Wing It",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                amount: Value::Const(1),
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
        ]),
        ..Default::default()
    }
}

/// Cackling Prowler — {3}{G} 4/3 Hyena Rogue. Ward {2}. Morbid — at the
/// beginning of your end step, if a creature died this turn, put a +1/+1
/// counter on it.
pub fn cackling_prowler() -> CardDefinition {
    use crate::card::WardCost;
    CardDefinition {
        name: "Cackling Prowler",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hyena, CreatureType::Rogue],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::YourControl,
            )
            .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Glimmerlight — {2} Equipment. When it enters, create a 1/1 white Glimmer
/// enchantment creature token. Equipped creature gets +1/+1. Equip {1}.
pub fn glimmerlight() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus, TokenDefinition};
    use crate::mana::Color;
    CardDefinition {
        name: "Glimmerlight",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Glimmer".into(),
                card_types: vec![CardType::Enchantment, CardType::Creature],
                colors: vec![Color::White],
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        })],
        ..Default::default()
    }
}

/// Demonic Ruckus — {1}{R} Aura. Enchanted creature gets +1/+1 and has menace
/// and trample. When this Aura is put into a graveyard from the battlefield,
/// draw a card. Plot {R}.
pub fn demonic_ruckus() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Demonic Ruckus",
        cost: cost(&[generic(1), r()]),
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
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Menace, Keyword::Trample],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        plot_cost: Some(cost(&[r()])),
        ..Default::default()
    }
}

/// Hugs, Grisly Guardian — {X}{R}{R}{G}{G} 5/5 Legendary Badger Warrior with
/// trample. When it enters, exile the top X cards of your library; you may play
/// them until your next end step. You may play an additional land each turn.
pub fn hugs_grisly_guardian() -> CardDefinition {
    use crate::effect::StaticEffect;
    use crate::mana::x;
    CardDefinition {
        name: "Hugs, Grisly Guardian",
        cost: cost(&[x(), r(), r(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger, CreatureType::Warrior],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![crate::card::StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::XFromCost,
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            uncast_penalty: None,
        })],
        ..Default::default()
    }
}

/// Gloomfang Mauler — {5}{B}{B} 5/5 Nightmare. Swampcycling {2}. Backup 2.
pub fn gloomfang_mauler() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Gloomfang Mauler",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Nightmare], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Landcycling(cost(&[generic(2)]), LandType::Swamp)],
        triggered_abilities: vec![crate::effect::shortcut::backup(2, vec![])],
        ..Default::default()
    }
}

/// Audacity — {G} Aura. Enchanted creature gets +2/+0 and has trample. When
/// this Aura is put into a graveyard from the battlefield, draw a card.
pub fn audacity() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Audacity",
        cost: cost(&[g()]),
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
            power: 2,
            toughness: 0,
            keywords: vec![Keyword::Trample],
            scale: None,
            triggered_abilities: vec![],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Felonious Rage — {R} Instant. Target creature you control gets +2/+0 and
/// gains haste until end of turn. When that creature dies this turn, create a
/// 2/2 white and blue Detective creature token.
pub fn felonious_rage() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Felonious Rage",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::WhenTargetDiesThisTurn {
                slot: 0,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Detective".into(),
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::White, Color::Blue],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Detective],
                            ..Default::default()
                        },
                        power: 2,
                        toughness: 2,
                        ..Default::default()
                    },
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Razorkin Hordecaller — {4}{R} 4/4 Human Clown Berserker with haste. Whenever
/// you attack, create a 1/1 red Gremlin creature token. (Modeled once per turn.)
pub fn razorkin_hordecaller() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    CardDefinition {
        name: "Razorkin Hordecaller",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Clown, CreatureType::Berserker],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Gremlin".into(),
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Gremlin],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Goldvein Pick — {2} Equipment. Equipped creature gets +1/+1 and, whenever it
/// deals combat damage to a player, creates a Treasure token. Equip {1}.
pub fn goldvein_pick() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Goldvein Pick",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![],
            scale: None,
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

// ── Tarkir: Dragonstorm + recent-set batch (claude/modern_decks) ─────────────

/// Boulderborn Dragon — {5} Artifact Dragon 3/3. Flying, vigilance; attacks →
/// surveil 1.
pub fn boulderborn_dragon() -> CardDefinition {
    CardDefinition {
        name: "Boulderborn Dragon",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Scales of Shale — {2}{B} Instant. Affinity for Lizards. Target creature gets
/// +2/+0 and gains lifelink and indestructible until end of turn.
pub fn scales_of_shale() -> CardDefinition {
    CardDefinition {
        name: "Scales of Shale",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        affinity_filter: Some(
            SelectionRequirement::HasCreatureType(CreatureType::Lizard)
                .and(SelectionRequirement::ControlledByYou),
        ),
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sunset Strikemaster — {1}{R} 3/1 Human Monk. {T}: Add {R}. {2}{R}, {T},
/// Sacrifice this: it deals 6 damage to target creature with flying.
pub fn sunset_strikemaster() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Sunset Strikemaster",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(crate::mana::Color::Red, Value::Const(1)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                    ),
                    amount: Value::Const(6),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Wardens of the Cycle — {1}{B}{G}{G} 3/4 Elf Warlock. Morbid — at your end
/// step, if a creature died this turn, gain 2 life, or draw a card and lose 1.
pub fn wardens_of_the_cycle() -> CardDefinition {
    CardDefinition {
        name: "Wardens of the Cycle",
        cost: cost(&[generic(1), b(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(1) },
                then: Box::new(Effect::ChooseMode(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                        Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                    ]),
                ])),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Roiling Dragonstorm — {1}{U} Enchantment. ETB: draw two, then discard a
/// card. When a Dragon you control enters, return this to its owner's hand.
pub fn roiling_dragonstorm() -> CardDefinition {
    CardDefinition {
        name: "Roiling Dragonstorm",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                    }),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            },
        ],
        ..Default::default()
    }
}

/// Stormcatch Mentor — {U}{R} 1/1 Otter Wizard. Haste, prowess; instant and
/// sorcery spells you cast cost {1} less.
pub fn stormcatch_mentor() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Stormcatch Mentor",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Otter, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste, Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Gurmag Drowner — {3}{U} 2/4 Snake Wizard. Exploit; when it exploits a
/// creature, look at the top four cards, put one into your hand, the rest on
/// the bottom.
pub fn gurmag_drowner() -> CardDefinition {
    CardDefinition {
        name: "Gurmag Drowner",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Snake, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![crate::effect::shortcut::exploit(Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            rest_to_graveyard: false,
            pick_filter: None,
            take: None,
            to_battlefield: false,
        })],
        ..Default::default()
    }
}

/// Temur Battlecrier — {G}{U}{R} 4/3 Orc Ranger. Spells you cast cost {1} less
/// for each creature you control with power 4 or greater. (The "during your
/// turn" gate is approximated as always-on.)
pub fn temur_battlecrier() -> CardDefinition {
    CardDefinition {
        name: "Temur Battlecrier",
        cost: cost(&[g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Ranger],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        affinity_filter: Some(
            SelectionRequirement::Creature
                .and(SelectionRequirement::PowerAtMost(3).negate())
                .and(SelectionRequirement::ControlledByYou),
        ),
        ..Default::default()
    }
}

/// Nullpriest of Oblivion — {1}{B} 2/1 Vampire Cleric. Kicker {3}{B}. Lifelink,
/// menace. ETB, if kicked: return target creature card from your graveyard to
/// the battlefield.
pub fn nullpriest_of_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Nullpriest of Oblivion",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink, Keyword::Menace, Keyword::Kicker(cost(&[generic(3), b()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Ureni, the Song Unending — {5}{G}{U}{R} 10/10 Spirit Dragon. Flying,
/// protection from white and from black. ETB: deal X damage divided as you
/// choose among any number of target creatures/planeswalkers opponents control,
/// where X is the number of lands you control.
pub fn ureni_the_song_unending() -> CardDefinition {
    use crate::card::CardType as CT;
    CardDefinition {
        name: "Ureni, the Song Unending",
        cost: cost(&[generic(5), g(), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dragon],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        keywords: vec![
            Keyword::Flying,
            Keyword::Protection(crate::mana::Color::White),
            Keyword::Protection(crate::mana::Color::Black),
        ],
        triggered_abilities: vec![etb(Effect::DealDamageDivided {
            total: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            ))),
            filter: (SelectionRequirement::Creature
                .or(SelectionRequirement::HasCardType(CT::Planeswalker)))
            .and(SelectionRequirement::ControlledByOpponent),
            max_targets: 10,
        })],
        ..Default::default()
    }
}

/// Elspeth, Storm Slayer — {3}{W}{W} Legendary Planeswalker. Tokens you create
/// are doubled. +1: make a 1/1 Soldier. 0: +1/+1 on each creature you control,
/// they gain flying until your next turn. −3: destroy target creature an
/// opponent controls with mana value 3 or greater.
pub fn elspeth_storm_slayer() -> CardDefinition {
    use crate::card::{
        LoyaltyAbility, PlaneswalkerSubtype, StaticAbility, StaticEffect, TokenDefinition,
    };
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Elspeth, Storm Slayer",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Elspeth],
            ..Default::default()
        },
        base_loyalty: 5,
        static_abilities: vec![StaticAbility {
            description: "If one or more tokens would be created under your control, twice that many are created instead.",
            effect: StaticEffect::DoubleTokens,
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        keyword: Keyword::Flying,
                        duration: Duration::UntilNextTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent)
                            .and(SelectionRequirement::ManaValueAtLeast(3)),
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Betor, Kin to All — {2}{W}{B}{G} 5/7 Spirit Dragon. Flying. At your end step:
/// if your creatures' total toughness ≥10 draw a card; then ≥20 untap each
/// creature you control; then ≥40 each opponent loses half their life, rounded
/// up.
pub fn betor_kin_to_all() -> CardDefinition {
    CardDefinition {
        name: "Betor, Kin to All",
        cost: cost(&[generic(2), w(), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(10)),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(20)),
                    then: Box::new(Effect::Untap {
                        what: Selector::EachPermanent(
                            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                        ),
                        up_to: None,
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(Value::TotalToughnessControlled, Value::Const(40)),
                    then: Box::new(Effect::LoseHalfLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        rounded_up: true,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Mistmoon Griffin — {3}{W} 2/2 Griffin. Flying. When it dies, return the top
/// creature card of your graveyard to the battlefield.
pub fn mistmoon_griffin() -> CardDefinition {
    CardDefinition {
        name: "Mistmoon Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::ReturnTopCreatureFromGraveyard {
            who: PlayerRef::You,
        })],
        ..Default::default()
    }
}

/// Dalek Squadron — {2}{B} 3/3 Artifact Dalek. Menace, myriad.
pub fn dalek_squadron() -> CardDefinition {
    CardDefinition {
        name: "Dalek Squadron",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dalek], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Myriad,
        }],
        ..Default::default()
    }
}

/// Perennation — {3}{W}{B}{G} Sorcery. Return target permanent card from your
/// graveyard to the battlefield with a hexproof counter and an indestructible
/// counter on it.
pub fn perennation() -> CardDefinition {
    CardDefinition {
        name: "Perennation",
        cost: cost(&[generic(3), w(), b(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: SelectionRequirement::Permanent
                        .and(SelectionRequirement::InYourGraveyard),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Hexproof,
                amount: Value::Const(1),
            },
            Effect::AddKeywordCounter {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Karakyk Guardian — {3}{G}{U}{R} 6/5 Dragon. Flying, vigilance, trample.
/// (Its conditional hexproof-while-it-hasn't-dealt-damage rider is omitted —
/// no lifetime damage-dealt tracking yet.)
pub fn karakyk_guardian() -> CardDefinition {
    CardDefinition {
        name: "Karakyk Guardian",
        cost: cost(&[generic(3), g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample],
        ..Default::default()
    }
}

/// Sarkhan, Soul Aflame — {1}{U}{R} 2/4 Human Shaman. Dragon spells you cast
/// cost {1} less. Whenever a Dragon you control enters, you may have Sarkhan
/// become a copy of it until end of turn. (The copy keeps the Dragon's name —
/// the printed "name stays Sarkhan" override is approximated.)
pub fn sarkhan_soul_aflame() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Sarkhan, Soul Aflame",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Dragon spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                amount: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCreatureType(CreatureType::Dragon),
                }),
            effect: Effect::MayDo {
                description: "have Sarkhan become a copy of that Dragon until end of turn".into(),
                body: Box::new(Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::TriggerSource,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                }),
            },
        }],
        ..Default::default()
    }
}
