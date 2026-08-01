//! Mercadian Masques (MMQ) gap closure, third wave — the Legate free-cast
//! cycle, combat-punisher creatures, and the remaining enchantments. Tests in
//! `classic_sets/mmq3`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{etb, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

// ── Shared shapes ───────────────────────────────────────────────────────────

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// "You control at least one land of `lt`."
fn controls_land(lt: LandType, opponent: bool) -> Predicate {
    let owner = if opponent { R::ControlledByOpponent } else { R::ControlledByYou };
    Predicate::SelectorExists(Selector::EachPermanent(R::HasLandType(lt).and(owner)))
}

/// The MMQ Legate cycle: "If an opponent controls a `theirs` and you control a
/// `yours`, you may cast this spell without paying its mana cost."
fn legate_alt(theirs: LandType, yours: LandType) -> AlternativeCost {
    AlternativeCost {
        condition: Some(Predicate::All(vec![
            controls_land(theirs, true),
            controls_land(yours, false),
        ])),
        ..Default::default()
    }
}

/// "Whenever this creature blocks or becomes blocked by a [`filter`] creature,
/// destroy that creature at end of combat."
fn combat_partner_punisher(filter: R) -> Vec<TriggeredAbility> {
    // The partner is captured now: by the end-of-combat step the block map is
    // gone, so a `CreaturesInCombatWith` read there would find nothing.
    let body = Effect::Seq(vec![
        Effect::RememberPermanentOnSource {
            what: Selector::MatchingAmong {
                inner: Box::new(Selector::CreaturesInCombatWith(Box::new(Selector::This))),
                filter,
            },
        },
        Effect::AtEndOfCombat {
            body: Box::new(Effect::Destroy { what: Selector::ChosenPermanentOfSource }),
        },
    ]);
    [EventKind::Blocks, EventKind::BecomesBlocked]
        .into_iter()
        .map(|kind| TriggeredAbility {
            event: EventSpec::new(kind, EventScope::SelfSource),
            effect: body.clone(),
        })
        .collect()
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

// ── Legates ─────────────────────────────────────────────────────────────────

/// Cho-Arrim Legate — {2}{W} 1/2 with protection from black; free against a
/// Swamp opponent.
pub fn cho_arrim_legate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        alternative_cost: Some(legate_alt(LandType::Swamp, LandType::Plains)),
        ..creature(
            "Cho-Arrim Legate",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Deepwood Legate — {3}{B} 1/1 Shade; free against a Forest opponent.
pub fn deepwood_legate() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(legate_alt(LandType::Forest, LandType::Swamp)),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Deepwood Legate", cost(&[generic(3), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Kyren Legate — {1}{R} 1/1 hasty Goblin; free against a Plains opponent.
pub fn kyren_legate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        alternative_cost: Some(legate_alt(LandType::Plains, LandType::Mountain)),
        ..creature("Kyren Legate", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Rushwood Legate — {2}{G} 2/1 Dryad; free against an Island opponent.
pub fn rushwood_legate() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(legate_alt(LandType::Island, LandType::Forest)),
        ..creature("Rushwood Legate", cost(&[generic(2), g()]), vec![CreatureType::Dryad], 2, 1)
    }
}

/// Saprazzan Legate — {3}{U} 1/3 flier; free against a Mountain opponent.
pub fn saprazzan_legate() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        alternative_cost: Some(legate_alt(LandType::Mountain, LandType::Island)),
        ..creature(
            "Saprazzan Legate",
            cost(&[generic(3), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            1,
            3,
        )
    }
}

// ── Combat punishers ────────────────────────────────────────────────────────

/// Deathgazer — {3}{B} 2/2 that kills the nonblack creature it meets.
pub fn deathgazer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: combat_partner_punisher(R::Not(Box::new(R::HasColor(Color::Black)))),
        ..creature("Deathgazer", cost(&[generic(3), b()]), vec![CreatureType::Lizard], 2, 2)
    }
}

/// Venomous Dragonfly — {3}{G} 1/1 flier that kills whatever it meets.
pub fn venomous_dragonfly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: combat_partner_punisher(R::Creature),
        ..creature(
            "Venomous Dragonfly",
            cost(&[generic(3), g()]),
            vec![CreatureType::Insect],
            1,
            1,
        )
    }
}

/// Ceremonial Guard — {2}{R} 3/4 that dies for joining combat at all.
pub fn ceremonial_guard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: [EventKind::Attacks, EventKind::Blocks]
            .into_iter()
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Destroy { what: Selector::This }),
                },
            })
            .collect(),
        ..creature(
            "Ceremonial Guard",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            4,
        )
    }
}

/// Saprazzan Outrigger — {3}{U} 5/5 that goes back on the library after combat.
pub fn saprazzan_outrigger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: [EventKind::Attacks, EventKind::Blocks]
            .into_iter()
            .map(|kind| TriggeredAbility {
                event: EventSpec::new(kind, EventScope::SelfSource),
                effect: Effect::AtEndOfCombat {
                    body: Box::new(Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Library {
                            who: PlayerRef::OwnerOfMoved,
                            pos: crate::effect::LibraryPosition::Top,
                        },
                    }),
                },
            })
            .collect(),
        ..creature(
            "Saprazzan Outrigger",
            cost(&[generic(3), u()]),
            vec![CreatureType::Merfolk],
            5,
            5,
        )
    }
}

/// Cavern Crawler — {2}{R} 0/3 mountainwalker with a reckless pump.
pub fn cavern_crawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Cavern Crawler", cost(&[generic(2), r()]), vec![CreatureType::Insect], 0, 3)
    }
}

/// Port Inspector — {1}{U} 1/2 that peeks when it's blocked.
pub fn port_inspector() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Look at defending player's hand".into(),
                body: Box::new(Effect::LookAtHand {
                    who: Selector::Player(PlayerRef::DefendingPlayer),
                }),
            },
        }],
        ..creature("Port Inspector", cost(&[generic(1), u()]), vec![CreatureType::Human], 1, 2)
    }
}

/// Robber Fly — {2}{R} 1/1 flier that churns the defender's hand.
pub fn robber_fly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DiscardHandDrawThatMany {
                who: Selector::Player(PlayerRef::DefendingPlayer),
            },
        }],
        ..creature("Robber Fly", cost(&[generic(2), r()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Caustic Wasps — {2}{G} 1/1 flier that eats an artifact on connection.
pub fn caustic_wasps() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Destroy target artifact that player controls".into(),
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Artifact.and(R::ControlledByOpponent)),
                }),
            },
        }],
        ..creature("Caustic Wasps", cost(&[generic(2), g()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Cho-Arrim Bruiser — {5}{W} 3/4 that taps two blockers as it swings.
pub fn cho_arrim_bruiser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::Tap { what: Selector::AllTargets }),
            },
        }],
        ..creature(
            "Cho-Arrim Bruiser",
            cost(&[generic(5), w()]),
            vec![CreatureType::Ogre, CreatureType::Rebel],
            3,
            4,
        )
    }
}

/// Lithophage — {3}{R}{R} 7/7 that eats a Mountain each upkeep.
pub fn lithophage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::SacrificeSourceUnlessSacrifice {
            filter: R::HasLandType(LandType::Mountain),
        })],
        ..creature(
            "Lithophage",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Insect],
            7,
            7,
        )
    }
}

/// Rushwood Elemental — {G}{G}{G}{G}{G} 4/4 trampler that grows each upkeep.
pub fn rushwood_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![your_upkeep(Effect::MayDo {
            description: "Put a +1/+1 counter on this creature".into(),
            body: Box::new(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
        })],
        ..creature(
            "Rushwood Elemental",
            cost(&[g(), g(), g(), g(), g()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Arms Dealer — {2}{R} 1/1 that fires Goblins at creatures.
pub fn arms_dealer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Goblin), 1)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(4),
            },
            ..Default::default()
        }],
        ..creature(
            "Arms Dealer",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            1,
            1,
        )
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Briar Patch — {1}{G}{G}. Attackers arrive a point weaker. ("attacks you"
/// reads as "an opponent's creature attacks" — exact heads-up.)
pub fn briar_patch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::ControlledByOpponent,
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..enchantment("Briar Patch", cost(&[generic(1), g(), g()]))
    }
}

/// Close Quarters — {2}{R}{R}. Every block you draw throws a spark.
pub fn close_quarters() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::YourControl),
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
        }],
        ..enchantment("Close Quarters", cost(&[generic(2), r(), r()]))
    }
}

/// Liability — {1}{B}{B}. Every nontoken permanent that dies costs its
/// controller a life.
pub fn liability() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::NotToken,
                }),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..enchantment("Liability", cost(&[generic(1), b(), b()]))
    }
}

/// Putrefaction — {4}{B}. Green and white spells cost their caster a card.
pub fn putrefaction() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::CastSpellMatches(
                    R::HasColor(Color::Green).or(R::HasColor(Color::White)),
                ),
            ),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..enchantment("Putrefaction", cost(&[generic(4), b()]))
    }
}

/// Black Market — {3}{B}{B}. Bank a charge counter per death, cash them in for
/// black mana at your first main phase.
pub fn black_market() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::PreCombatMain),
                    EventScope::YourControl,
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        Color::Black,
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::Charge,
                        },
                    ),
                },
            },
        ],
        ..enchantment("Black Market", cost(&[generic(3), b(), b()]))
    }
}

/// Armistice — {2}{W}. Buy a card; the opponent buys 3 life.
pub fn armistice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w(), w()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::GainLife {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::Const(3),
                },
            ]),
            ..Default::default()
        }],
        ..enchantment("Armistice", cost(&[generic(2), w()]))
    }
}

/// Security Detail — {3}{W}. A Soldier a turn, but only from an empty board.
pub fn security_detail() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), w()]),
            once_per_turn: true,
            condition: Some(Predicate::Not(Box::new(Predicate::SelectorExists(
                Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            )))),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::White],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Soldier],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..enchantment("Security Detail", cost(&[generic(3), w()]))
    }
}

/// Customs Depot — {1}{U}. Pay {1} on each creature spell to loot.
pub fn customs_depot() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::MayPay {
                description: "Pay {1} to draw a card, then discard a card".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ])),
                else_: None,
            },
        }],
        ..enchantment("Customs Depot", cost(&[generic(1), u()]))
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Bifurcate — {3}{G}. Fetch a twin for a creature already in play.
pub fn bifurcate() -> CardDefinition {
    sorcery(
        "Bifurcate",
        cost(&[generic(3), g()]),
        Effect::SearchSameNameAs {
            who: PlayerRef::You,
            subject: target_filtered(R::Creature.and(R::NotToken)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Midnight Ritual — {X}{2}{B}. Trade X graveyard creatures for X Zombies.
pub fn midnight_ritual() -> CardDefinition {
    CardDefinition {
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 1,
                filter: R::InYourGraveyard.and(R::Creature),
                effect: Box::new(Effect::Exile { what: Selector::AllTargets }),
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    colors: vec![Color::Black],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        ]),
        ..sorcery(
            "Midnight Ritual",
            cost(&[crate::mana::x(), generic(2), b()]),
            Effect::Noop,
        )
    }
}

/// Soothsaying — {U}. Shuffle, or set the top of your library.
pub fn soothsaying() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3), u(), u()]),
                effect: Effect::ShuffleLibrary { who: PlayerRef::You },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[crate::mana::x()]),
                effect: Effect::RearrangeTop { who: PlayerRef::You, amount: Value::XFromCost },
                ..Default::default()
            },
        ],
        ..enchantment("Soothsaying", cost(&[u()]))
    }
}

/// Enslaved Horror — {3}{B} 4/4 that reanimates for everyone else.
pub fn enslaved_horror() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::EachPlayerReanimateCreatureMaxMv {
            max_mv: u32::MAX,
        })],
        ..creature("Enslaved Horror", cost(&[generic(3), b()]), vec![CreatureType::Horror], 4, 4)
    }
}
