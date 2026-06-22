//! Recent-set staples (MH3 / BLB / DSK / OTJ / FDN / DFT / TDM …) that fill
//! gaps in the Modern-playable pool. Each card has at least one functionality
//! test in `crabomination/src/tests/recent.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind, EventScope, EventSpec,
    Keyword, MayPlayDuration, Predicate, Selector, SelectionRequirement, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, PlayerRef};
use crate::mana::{b, colorless, cost, g, generic, r, u, w};

/// Questing Beast — {2}{G}{G} 4/4 Legendary Beast. Vigilance, deathtouch,
/// haste; can't be blocked by creatures with power 2 or less. (The
/// "combat damage can't be prevented" and planeswalker-redirect riders are
/// omitted.)
pub fn questing_beast() -> CardDefinition {
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
/// combat damage to a player, they lose half their life, rounded up. (The
/// dies-return-with-stun-counters rider is omitted.)
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseHalfLife {
                who: Selector::Player(PlayerRef::Target(0)),
                rounded_up: true,
            },
        }],
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
/// you don't control; if cast in your main phase, yours gets +1/+1 first.
/// (The main-phase pump rider is omitted.)
pub fn tail_swipe() -> CardDefinition {
    CardDefinition {
        name: "Tail Swipe",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Fight {
            attacker: Selector::TargetFiltered {
                slot: 0,
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            },
            defender: Selector::TargetFiltered {
                slot: 1,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByOpponent),
            },
        },
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
