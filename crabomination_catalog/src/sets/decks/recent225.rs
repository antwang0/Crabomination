//! Gap batch — Savage Ventmaw + Fake Your Own Death clear FDN cards that were
//! blocked on one primitive each (persistent mana, plain self-return-tapped);
//! the rest are clean MKM/OTJ/DSK/BIG gaps on existing primitives. Tests in
//! `tests/recent225.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    ExileReturnZone, Keyword, LandType, SelectionRequirement as R, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{
    attacks_while_saddled, battalion, investigate, mint_treasures, on_attack,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, Value, ZoneDest, ZoneRef,
};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

/// Savage Ventmaw — {4}{R}{G} 4/4 Dragon. Flying; whenever it attacks, add
/// {R}{R}{R}{G}{G}{G} that doesn't empty as steps and phases end this turn.
pub fn savage_ventmaw() -> CardDefinition {
    CardDefinition {
        name: "Savage Ventmaw",
        cost: cost(&[generic(4), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::AddManaKeptThisTurn {
            who: PlayerRef::You,
            colors: vec![
                Color::Red,
                Color::Red,
                Color::Red,
                Color::Green,
                Color::Green,
                Color::Green,
            ],
        })],
        ..Default::default()
    }
}

/// Fake Your Own Death — {1}{B} Instant. Target creature gets +2/+0 and gains
/// "when this dies, return it to the battlefield tapped and create a Treasure."
pub fn fake_your_own_death() -> CardDefinition {
    let revive = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::Seq(vec![Effect::ReturnSelfTapped, mint_treasures(1)]),
    };
    CardDefinition {
        name: "Fake Your Own Death",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature,
                },
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantTriggeredAbility {
                what: Selector::Target(0),
                trigger: Box::new(revive),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

fn zombie_2_2_tapped() -> TokenDefinition {
    TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        tapped: true,
        ..Default::default()
    }
}

/// Dread Summons — {X}{B}{B} Sorcery. Each player mills X cards; for each
/// creature card milled this way, create a tapped 2/2 black Zombie.
pub fn dread_summons() -> CardDefinition {
    CardDefinition {
        name: "Dread Summons",
        cost: cost(&[x(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::XFromCost,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CreatureCardsMilledThisEffect,
                definition: Box::new(zombie_2_2_tapped()),
            },
        ]),
        ..Default::default()
    }
}

/// On the Job — {2}{W}{W} Instant. Creatures you control get +2/+1; investigate.
pub fn on_the_job() -> CardDefinition {
    CardDefinition {
        name: "On the Job",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachMatching {
                    zone: ZoneRef::Battlefield,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                power: Value::Const(2),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            investigate(1),
        ]),
        ..Default::default()
    }
}

/// Makeshift Binding — {2}{W} Enchantment. ETB: exile target creature an
/// opponent controls until this leaves; gain 2 life.
pub fn makeshift_binding() -> CardDefinition {
    CardDefinition {
        name: "Makeshift Binding",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileUntilSourceLeaves {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    },
                    return_to: ExileReturnZone::Battlefield,
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Seasoned Consultant — {1}{W} 1/3 Human Detective. Whenever you attack with
/// three or more creatures, it gets +2/+0.
pub fn seasoned_consultant() -> CardDefinition {
    CardDefinition {
        name: "Seasoned Consultant",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Detective],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![battalion(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Unauthorized Exit — {1}{U} Instant. Return target nonland permanent to its
/// owner's hand; surveil 1.
pub fn unauthorized_exit() -> CardDefinition {
    CardDefinition {
        name: "Unauthorized Exit",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Permanent.and(R::Nonland),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        ..Default::default()
    }
}

/// Long Goodbye — {1}{B} Instant. Can't be countered. Destroy target creature
/// or planeswalker with mana value 3 or less.
pub fn long_goodbye() -> CardDefinition {
    CardDefinition {
        name: "Long Goodbye",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Destroy {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.or(R::Planeswalker).and(R::ManaValueAtMost(3)),
            },
        },
        ..Default::default()
    }
}

/// It Doesn't Add Up — {3}{B}{B} Instant. Return target creature card from your
/// graveyard to the battlefield; suspect it.
pub fn it_doesnt_add_up() -> CardDefinition {
    CardDefinition {
        name: "It Doesn't Add Up",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            Effect::Suspect {
                what: Selector::LastMoved,
            },
        ]),
        ..Default::default()
    }
}

/// Eliminate the Impossible — {1}{U} Instant. Investigate; creatures your
/// opponents control get -2/-0; clear their suspected status.
pub fn eliminate_the_impossible() -> CardDefinition {
    let opp_creatures = Selector::EachMatching {
        zone: ZoneRef::Battlefield,
        filter: R::Creature.and(R::ControlledByOpponent),
    };
    CardDefinition {
        name: "Eliminate the Impossible",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            investigate(1),
            Effect::PumpPT {
                what: opp_creatures.clone(),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::ClearSuspected {
                what: opp_creatures,
            },
        ]),
        ..Default::default()
    }
}

/// Mirage Mesa — Desert land. Enters tapped; as it enters, choose a color.
/// {T}: Add one mana of the chosen color.
pub fn mirage_mesa() -> CardDefinition {
    CardDefinition {
        name: "Mirage Mesa",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Desert],
            ..Default::default()
        },
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Tap {
                    what: Selector::This,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ChooseColorForSelf,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::ChosenColorOfSource,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn angel_3_3_flying() -> TokenDefinition {
    TokenDefinition {
        name: "Angel".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Seraphic Steed — {G}{W} 2/2 Unicorn Mount. First strike, lifelink; whenever
/// it attacks while saddled, create a 3/3 white Angel with flying. Saddle 4.
pub fn seraphic_steed() -> CardDefinition {
    CardDefinition {
        name: "Seraphic Steed",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Unicorn, CreatureType::Mount],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink, Keyword::Saddle(4)],
        triggered_abilities: vec![attacks_while_saddled(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: Box::new(angel_3_3_flying()),
        })],
        ..Default::default()
    }
}

/// Terramorphic Expanse — Land. {T}, Sacrifice this land: Search your library
/// for a basic land, put it onto the battlefield tapped, then shuffle.
pub fn terramorphic_expanse() -> CardDefinition {
    CardDefinition {
        name: "Terramorphic Expanse",
        card_types: vec![CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Valgavoth's Lair — Enchantment Land. Hexproof; enters tapped; as it enters,
/// choose a color. {T}: Add one mana of the chosen color.
pub fn valgavoths_lair() -> CardDefinition {
    CardDefinition {
        name: "Valgavoth's Lair",
        card_types: vec![CardType::Enchantment, CardType::Land],
        keywords: vec![Keyword::Hexproof],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Tap {
                    what: Selector::This,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ChooseColorForSelf,
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::ChosenColorOfSource,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pitiless Carnage — {3}{B} Sorcery. Sacrifice any number of permanents you
/// control, then draw that many cards. Plot {1}{B}{B}.
pub fn pitiless_carnage() -> CardDefinition {
    CardDefinition {
        name: "Pitiless Carnage",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        plot_cost: Some(cost(&[generic(1), b(), b()])),
        effect: Effect::SacrificeAnyNumber {
            who: PlayerRef::You,
            filter: R::Permanent.and(R::ControlledByYou),
            per_each: Box::new(Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            }),
        },
        ..Default::default()
    }
}

fn golem_3_3() -> TokenDefinition {
    TokenDefinition {
        name: "Golem".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        ..Default::default()
    }
}

/// Sandstorm Salvager — {2}{G} 1/1 Human Artificer. ETB: create a 3/3 Golem.
/// {2}, {T}: Put a +1/+1 counter on each creature token you control; they gain
/// trample.
pub fn sandstorm_salvager() -> CardDefinition {
    let token_creatures = Selector::EachMatching {
        zone: ZoneRef::Battlefield,
        filter: R::Creature.and(R::ControlledByYou).and(R::IsToken),
    };
    CardDefinition {
        name: "Sandstorm Salvager",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: Box::new(golem_3_3()),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: token_creatures.clone(),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GrantKeyword {
                    what: token_creatures,
                    keyword: Keyword::Trample,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Nightdrinker Moroii — {3}{B} 4/2 Vampire. Flying; ETB lose 3 life. Disguise
/// {B}{B}.
pub fn nightdrinker_moroii() -> CardDefinition {
    CardDefinition {
        name: "Nightdrinker Moroii",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Disguise(cost(&[b(), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

/// Wojek Investigator — {2}{W} 2/4 Angel Detective. Flying, vigilance; at the
/// beginning of your upkeep, if an opponent has more cards in hand, investigate.
pub fn wojek_investigator() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Wojek Investigator",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::ActivePlayer,
            )
            .with_filter(Predicate::AnOpponentHasMoreCardsInHand),
            effect: investigate(1),
        }],
        ..Default::default()
    }
}

/// Sandstorm Verge — Desert land. {T}: Add {C}. {3}, {T}: Target creature can't
/// block this turn. Activate only as a sorcery.
pub fn sandstorm_verge() -> CardDefinition {
    CardDefinition {
        name: "Sandstorm Verge",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Desert],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sorcery_speed: true,
                effect: Effect::GrantKeyword {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature,
                    },
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
