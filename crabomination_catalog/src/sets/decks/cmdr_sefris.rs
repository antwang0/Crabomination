//! Commander: the cards the **Dungeons of Death** precon (AFC, Sefris of the
//! Hidden Ways) needed beyond what the catalog had (Extract Brain is
//! `cmdr_gonti`'s). Tests in
//! `tests/recent_b/cmdr_sefris.rs`.
//!
//! Residuals (each also on its card):
//! - **Grave Endeavor** — the returned creature is the greatest-power one,
//!   not a free choice, and its counters go on after it enters rather than
//!   with it.
//! - **Nihiloor** — the creature tapped for the steal is always Nihiloor
//!   itself, and only one opponent's creature is taken.
//! - **Phantom Steed** — the attacking token copy isn't an Illusion in
//!   addition to its other types.
//! - **Rod of Absorption** — every instant or sorcery resolving while it is
//!   on the battlefield is exiled, including one cast before it arrived.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_you_attack, target_filtered};
use crate::effect::{DelayedTriggerKind, Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, generic, u, w, x, Color, ManaCost};

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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn draw(amount: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount }
}

fn loot() -> Effect {
    Effect::Seq(vec![
        draw(Value::ONE),
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
    ])
}

fn creature_card_in_your_graveyard() -> R {
    R::Creature.and(R::InYourGraveyard)
}

fn reanimate_target() -> Effect {
    Effect::Move {
        what: target_filtered(creature_card_in_your_graveyard()),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    }
}

/// Sefris of the Hidden Ways — once a turn, creature cards hitting your
/// graveyard from anywhere venture; completing a dungeon reanimates a
/// creature card.
pub fn sefris_of_the_hidden_ways() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                    .once_per_turn(),
                effect: Effect::Venture,
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DungeonCompleted, EventScope::YourControl),
                effect: reanimate_target(),
            },
        ],
        ..legend(
            "Sefris of the Hidden Ways",
            cost(&[w(), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Arcane Endeavor — two d8: draw one result, then cast an instant or
/// sorcery of mana value up to the other from your hand for free.
pub fn arcane_endeavor() -> CardDefinition {
    spell(
        "Arcane Endeavor",
        cost(&[generic(5), u(), u()]),
        CardType::Sorcery,
        Effect::RollTwoDiceAssign {
            sides: 8,
            first: Box::new(draw(Value::LastDieRoll)),
            second: Box::new(Effect::WithX {
                x: Value::LastDieRoll,
                body: Box::new(Effect::CastFromHandWithoutPaying {
                    filter: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::ManaValueAtMostXFromCost)),
                }),
            }),
        },
    )
}

/// Bucknard's Everfull Purse — roll a d4 for that many Treasures, then the
/// player to your right gains control of it.
pub fn bucknards_everfull_purse() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::RollDie {
                    sides: 4,
                    count: Value::ONE,
                    modifier: Value::ZERO,
                    reroll_at_most: 0,
                    ignore_lowest: 0,
                    results: vec![(
                        1,
                        4,
                        Effect::CreateToken {
                            who: PlayerRef::You,
                            count: Value::LastDieRoll,
                            definition: Arc::new(crabomination_base::tokens::treasure_token()),
                        },
                    )],
                    on_doubles: None,
                },
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::PlayerToYourRight),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Bucknard's Everfull Purse", cost(&[generic(2)]))
    }
}

/// Grave Endeavor — two d10: reanimate a creature card with one result in
/// +1/+1 counters, then drain each opponent by the other.
///
/// Residual: the creature returned is the greatest-power one, and its
/// counters are put on after it enters rather than with it.
pub fn grave_endeavor() -> CardDefinition {
    spell(
        "Grave Endeavor",
        cost(&[generic(5), b(), b()]),
        CardType::Instant,
        Effect::RollTwoDiceAssign {
            sides: 10,
            // "A creature card" — a choice, not a target (the targeted form
            // hid its slot inside the dice branch, where the cast couldn't
            // see it). The pick is the greatest power.
            first: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TakeGreatestPower {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter {
                    what: Selector::LastMoved,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::LastDieRoll,
                },
            ])),
            second: Box::new(Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::LastDieRoll,
            }),
        },
    )
}

/// Hama Pashar, Ruin Seeker — room abilities of your dungeons trigger an
/// additional time.
pub fn hama_pashar_ruin_seeker() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Room abilities of dungeons you own trigger an additional time.",
            effect: StaticEffect::DungeonRoomsTriggerTwice,
        }],
        ..legend(
            "Hama Pashar, Ruin Seeker",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Immovable Rod — may stay tapped; untapping ventures; while tapped after
/// its ability, another permanent loses all abilities and can't attack or
/// block.
pub fn immovable_rod() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
            effect: Effect::Venture,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::LoseAllAbilities {
                    what: target_filtered(R::Permanent.and(R::OtherThanSource)),
                    duration: Duration::WhileSourceTapped,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantAttack,
                    duration: Duration::WhileSourceTapped,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::CantBlock,
                    duration: Duration::WhileSourceTapped,
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Immovable Rod", cost(&[w()]))
    }
}

/// Midnight Pathlighter — your creatures can't be blocked except by
/// legendary creatures; combat damage to a player ventures.
pub fn midnight_pathlighter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control can't be blocked except by legendary creatures.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::CantBeBlockedExceptBy(Box::new(R::HasSupertype(Supertype::Legendary))),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
            effect: Effect::Venture,
        }],
        ..creature(
            "Midnight Pathlighter",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Minimus Containment — enchanted nonland permanent is a Treasure artifact
/// with the Treasure mana ability and nothing else.
pub fn minimus_containment() -> CardDefinition {
    CardDefinition {
        name: "Minimus Containment",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land)))) },
        equipped_bonus: Some(EquipBonus {
            set_card_types: Some(vec![CardType::Artifact]),
            set_artifact_types: Some(vec![ArtifactSubtype::Treasure]),
            remove_abilities: true,
            activated_abilities: crabomination_base::tokens::treasure_token().activated_abilities,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Minn, Wily Illusionist — your second draw each turn makes a growing
/// Illusion; an Illusion of yours dying lets you put a permanent card with
/// mana value up to its power from hand onto the battlefield.
pub fn minn_wily_illusionist() -> CardDefinition {
    let illusion = Arc::new(TokenDefinition {
        name: "Illusion".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Illusion], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "This token gets +1/+0 for each other Illusion you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasCreatureType(CreatureType::Illusion).and(R::OtherThanSource),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        ..Default::default()
    });
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::YourControl),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: illusion },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Illusion),
                    },
                ),
                effect: Effect::WithX {
                    x: Value::PowerOf(Box::new(Selector::TriggerSource)),
                    body: Box::new(Effect::PutFromHandOntoBattlefield {
                        who: PlayerRef::You,
                        filter: R::PermanentCard.and(R::ManaValueAtMostXFromCost),
                        count: Value::ONE,
                        tapped: false,
                        haste: false,
                        sacrifice_eot: false,
                        return_eot: false,
                        then: None,
                    }),
                },
            },
        ],
        ..legend(
            "Minn, Wily Illusionist",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Gnome, CreatureType::Wizard],
            1,
            3,
        )
    }
}

/// Murder of Crows — flying; whenever another creature dies, you may loot.
pub fn murder_of_crows() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OtherThanSource },
            ),
            effect: Effect::MayDo { description: "Draw a card, then discard a card?".into(), body: Box::new(loot()) },
        }],
        ..creature("Murder of Crows", cost(&[generic(3), u(), u()]), vec![CreatureType::Bird], 4, 4)
    }
}

/// Nihiloor — on entering, steals an opponent's creature no stronger than
/// the creature it taps while you control Nihiloor; attacking with a
/// creature an opponent owns drains its owner for 2.
///
/// Residual: the tapped creature is always Nihiloor, and only one
/// opponent's creature is taken.
pub fn nihiloor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::GainControlWhileYouControlSource {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerAtMostSourcePower)),
                },
                Effect::Tap { what: Selector::This },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Not(Box::new(R::OwnedByYou)),
                    },
                ),
                effect: Effect::Seq(vec![
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                        amount: Value::Const(2),
                    },
                ]),
            },
        ],
        ..legend(
            "Nihiloor",
            cost(&[generic(2), w(), u(), b()]),
            vec![CreatureType::Horror],
            3,
            5,
        )
    }
}

/// Nimbus Maze — {C}; {W} with an Island; {U} with a Plains.
pub fn nimbus_maze() -> CardDefinition {
    let gated = |color: Color, needs: LandType| ActivatedAbility {
        condition: Some(Predicate::SelectorExists(Selector::EachPermanent(
            R::HasLandType(needs).and(R::ControlledByYou),
        ))),
        ..crate::sets::tap_add(color)
    };
    CardDefinition {
        name: "Nimbus Maze",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            gated(Color::White, LandType::Island),
            gated(Color::Blue, LandType::Plains),
        ],
        ..Default::default()
    }
}

/// Obsessive Stitcher — tap to loot; {2}{U}{B}, tap, sacrifice it:
/// reanimate a creature card.
pub fn obsessive_stitcher() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: loot(), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u(), b()]),
                tap_cost: true,
                sac_cost: true,
                effect: reanimate_target(),
                ..Default::default()
            },
        ],
        ..creature(
            "Obsessive Stitcher",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            3,
        )
    }
}

/// Phantom Steed — flash; exiles another creature of yours while it
/// stays; attacking brings a token copy of that card in attacking until
/// end of combat.
///
/// Residual: the token isn't an Illusion in addition to its other types.
pub fn phantom_steed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![
            etb(Effect::ExileUntilSourceLeaves {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                return_to: crate::card::ExileReturnZone::Battlefield,
            }),
            on_attack(Effect::TokenCopyAttackingUntilEndOfCombat { source: Selector::CardExiledWithSource }),
        ],
        ..creature(
            "Phantom Steed",
            cost(&[generic(3), u()]),
            vec![CreatureType::Horse, CreatureType::Illusion],
            4,
            3,
        )
    }
}

/// Revivify — d20 plus your creature cards that died this turn: 1–14
/// returns them to hand, 15+ to the battlefield.
pub fn revivify() -> CardDefinition {
    let died = || R::Creature.and(R::PutIntoGraveyardFromBattlefieldThisTurn);
    let back = |to: ZoneDest| Effect::Move {
        what: Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: died() },
        to,
    };
    spell(
        "Revivify",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::RollDie {
            sides: 20,
            count: Value::ONE,
            modifier: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: died() },
            reroll_at_most: 0,
            ignore_lowest: 0,
            results: vec![
                (1, 14, back(ZoneDest::Hand(PlayerRef::You))),
                (15, u8::MAX, back(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false })),
            ],
            on_doubles: None,
        },
    )
}

/// Rod of Absorption — resolving instants and sorceries are exiled with
/// it; {X}, tap, sacrifice: cast any of them with total mana value X or
/// less for free.
///
/// Residual: it exiles every instant or sorcery that resolves while it is
/// on the battlefield, including one cast before it arrived.
pub fn rod_of_absorption() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a player casts an instant or sorcery spell, exile it instead of putting it \
                          into a graveyard as it resolves.",
            effect: StaticEffect::ExileResolvingInstantsAndSorceries,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::CastAnyOrderWithoutPaying {
                what: Selector::CardExiledWithSource,
                source_zone: Zone::Exile,
                filter: None,
                cap: None,
                total_mana_value: Some(Value::XFromCost),
            },
            ..Default::default()
        }],
        ..artifact("Rod of Absorption", cost(&[generic(2), u()]))
    }
}

/// Thorough Investigation — attacking investigates; sacrificing a Clue
/// ventures.
pub fn thorough_investigation() -> CardDefinition {
    CardDefinition {
        name: "Thorough Investigation",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            on_you_attack(investigate(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Clue),
                    },
                ),
                effect: Effect::Venture,
            },
        ],
        ..Default::default()
    }
}

/// Vanish into Memory — exile a creature and draw its power; it returns at
/// your next upkeep and you discard its toughness.
pub fn vanish_into_memory() -> CardDefinition {
    spell(
        "Vanish into Memory",
        cost(&[generic(2), w(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature) },
            draw(Value::PowerOf(Box::new(Selector::Target(0)))),
            Effect::DelayUntilWithCapture {
                kind: DelayedTriggerKind::YourNextUpkeep,
                capture: Selector::Target(0),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TargetFiltered { slot: 0, filter: R::InExile },
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ToughnessOf(Box::new(Selector::TargetFiltered {
                            slot: 0,
                            filter: R::OnBattlefield,
                        })),
                        random: false,
                    },
                ])),
            },
        ]),
    )
}

/// Wand of Orcus — the equipped creature attacking or blocking gives it and
/// your Zombies deathtouch; its combat damage to a player makes that many
/// 2/2 Zombies.
pub fn wand_of_orcus() -> CardDefinition {
    let zombie = Arc::new(TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    });
    let deathtouch = || {
        Effect::Seq(vec![
            Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Zombie).and(R::ControlledByYou)),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        ])
    };
    CardDefinition {
        name: "Wand of Orcus",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![
                on_attack(deathtouch()),
                crate::effect::shortcut::blocks(deathtouch()),
                TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::TriggerEventAmount,
                        definition: zombie,
                    },
                },
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}
