//! MKM (Murders at Karlov Manor) gap batch — Selesnya value, Rakdos payoff,
//! disguise bodies, and a Goblin engine. Tests in `tests/recent_b/recent252.rs`.

use crate::card::{
    ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EquipBonus,
    Keyword, LandType, SelectionRequirement as R, Subtypes, TokenDefinition,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, TriggeredAbility, Value,
    ZoneDest,
};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Treacherous Greed — {1}{W}{B} Instant. Additional cost: sacrifice a creature
/// that dealt damage this turn. Draw three cards. Each opponent loses 3 life
/// and you gain 3 life.
pub fn treacherous_greed() -> CardDefinition {
    use crate::card::AdditionalCastCost;
    CardDefinition {
        name: "Treacherous Greed",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.and(R::DealtDamageThisTurn),
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(3),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Flourishing Bloom-Kin — {1}{G} Creature — Plant Elemental 0/0. Gets +1/+1 for
/// each Forest you control. Disguise {4}{G}. When turned face up, search your
/// library for a Forest onto the battlefield tapped and another into your hand.
pub fn flourishing_bloom_kin() -> CardDefinition {
    CardDefinition {
        name: "Flourishing Bloom-Kin",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Elemental],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        dynamic_pt: Some(DynamicPt::BasePlusLandsOfTypeControlled {
            land_type: LandType::Forest,
            base_p: 0,
            base_t: 0,
        }),
        keywords: vec![Keyword::Disguise(cost(&[generic(4), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasLandType(LandType::Forest)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: true,
                    },
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasLandType(LandType::Forest)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Concealed Weapon — {1}{R} Artifact — Equipment. Equipped creature gets +3/+0.
/// Disguise {2}{R}. When turned face up, attach it to target creature you
/// control. Equip {1}{R}.
pub fn concealed_weapon() -> CardDefinition {
    CardDefinition {
        name: "Concealed Weapon",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![
            Keyword::Equip(cost(&[generic(1), r()])),
            Keyword::Disguise(cost(&[generic(2), r()])),
        ],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 0,
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature.and(R::ControlledByYou)),
            },
        }],
        ..Default::default()
    }
}

/// Lumbering Laundry — {5} Artifact Creature — Golem 4/5. Disguise {5}. (Its
/// "{2}: look at face-down creatures you don't control" is an info-only ability
/// with no rules-visible effect in this engine and is omitted.)
pub fn lumbering_laundry() -> CardDefinition {
    CardDefinition {
        name: "Lumbering Laundry",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Golem],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Disguise(cost(&[generic(5)]))],
        ..Default::default()
    }
}

/// Audience with Trostani — {2}{G} Sorcery. Create a 0/1 green Plant creature
/// token, then draw cards equal to the number of differently named creature
/// tokens you control.
pub fn audience_with_trostani() -> CardDefinition {
    CardDefinition {
        name: "Audience with Trostani",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: plant_0_1_token(),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::DifferentlyNamedCreatureTokensControlled,
            },
        ]),
        ..Default::default()
    }
}

/// Krenko, Baron of Tin Street — {2}{R} Legendary Creature — Goblin 3/3, haste.
/// {T}, Sacrifice an artifact: Put a +1/+1 counter on each Goblin you control.
/// Whenever an artifact is put into a graveyard from the battlefield, you may
/// pay {R}. If you do, create a 1/1 red Goblin creature token with haste.
pub fn krenko_baron_of_tin_street() -> CardDefinition {
    use crate::effect::ActivatedAbility;
    CardDefinition {
        name: "Krenko, Baron of Tin Street",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact.and(R::ControlledByYou), 1)),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    R::HasCreatureType(CreatureType::Goblin).and(R::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact,
                },
            ),
            effect: Effect::MayPay {
                description: "Pay {R} to create a hasty Goblin?".into(),
                mana_cost: cost(&[r()]),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: hasty_goblin_token(),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Cryptex — {2} Artifact. {T}, Collect evidence 3: Add one mana of any color
/// and put an unlock counter on this. Sacrifice this: Surveil 3, then draw
/// three cards. Activate only if it has five or more unlock counters.
pub fn cryptex() -> CardDefinition {
    use crate::effect::ActivatedAbility;
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Cryptex",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                collect_evidence_cost: Some(3),
                effect: Effect::Seq(vec![
                    Effect::AddMana {
                        who: PlayerRef::You,
                        pool: ManaPayload::AnyOneColor(Value::ONE),
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::Unlock,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                condition: Some(crate::effect::Predicate::SourceHasCountersAtLeast {
                    counter: CounterType::Unlock,
                    n: 5,
                }),
                effect: Effect::Seq(vec![
                    Effect::Surveil {
                        who: PlayerRef::You,
                        amount: Value::Const(3),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(3),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Detective's Satchel — {2}{U}{R} Artifact. When it enters, investigate twice.
/// {T}: Create a 1/1 colorless Thopter artifact creature token with flying.
/// Activate only if you've sacrificed an artifact this turn.
pub fn detectives_satchel() -> CardDefinition {
    use crate::effect::ActivatedAbility;
    use crate::effect::shortcut::investigate;
    CardDefinition {
        name: "Detective's Satchel",
        cost: cost(&[generic(2), u(), r()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(investigate(2))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(crate::effect::Predicate::SacrificedArtifactThisTurn {
                who: PlayerRef::You,
            }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: thopter_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Polygraph Orb — {4}{B} Artifact. When it enters, look at the top four cards
/// of your library, put two into your hand and the rest into your graveyard;
/// you lose 2 life. {2}, {T}, Collect evidence 3: Each opponent loses 3 life
/// unless they discard a card or sacrifice a creature.
pub fn polygraph_orb() -> CardDefinition {
    use crate::effect::ActivatedAbility;
    CardDefinition {
        name: "Polygraph Orb",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(4),
                rest_to_graveyard: true,
                pick_filter: None,
                take: Some(Value::Const(2)),
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
            },
            Effect::LoseLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            collect_evidence_cost: Some(3),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::ONE,
                        filter: R::Creature,
                    },
                ],
                otherwise: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(3),
                }),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Undergrowth Recon — {1}{G}{G} Enchantment. At the beginning of your upkeep,
/// return target land card from your graveyard to the battlefield tapped.
pub fn undergrowth_recon() -> CardDefinition {
    CardDefinition {
        name: "Undergrowth Recon",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Move {
                what: target_filtered(R::Land),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        }],
        ..Default::default()
    }
}

/// Dramatic Accusation — {2}{U} Aura. Enchant creature. When it enters, tap the
/// enchanted creature; the enchanted creature doesn't untap (modeled by tapping
/// it each upkeep, as Narcolepsy does). {U}{U}: Shuffle enchanted creature into
/// its owner's library.
pub fn dramatic_accusation() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    use crate::effect::{ActivatedAbility, LibraryPosition};
    CardDefinition {
        name: "Dramatic Accusation",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Seq(vec![
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
            Effect::Tap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        ]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Tap {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::AttachedTo(Box::new(
                        Selector::This,
                    )))),
                    pos: LibraryPosition::Shuffled,
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn thopter_token() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thopter],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

fn plant_0_1_token() -> TokenDefinition {
    TokenDefinition {
        name: "Plant".into(),
        colors: vec![crate::mana::Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        ..Default::default()
    }
}

fn hasty_goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".into(),
        colors: vec![crate::mana::Color::Red],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Lamplight Phoenix — {1}{R}{R} Creature — Phoenix 3/3, flying. When it dies,
/// you may exile it and collect evidence 4. If you do, return it to the
/// battlefield tapped.
pub fn lamplight_phoenix() -> CardDefinition {
    CardDefinition {
        name: "Lamplight Phoenix",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phoenix],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            // `CollectEvidence` carries the printed "you may" via its own
            // optional prompt; paying it returns the phoenix.
            effect: Effect::CollectEvidence {
                amount: Value::Const(4),
                then: Box::new(Effect::ReturnSelfTapped),
            },
        }],
        ..Default::default()
    }
}

/// Slime Against Humanity — {2}{G} Sorcery. Create a 0/0 green Ooze creature
/// token with trample, then put X +1/+1 counters on it, where X is two plus the
/// number of cards you own in exile and in your graveyard that are Oozes or are
/// named Slime Against Humanity.
pub fn slime_against_humanity() -> CardDefinition {
    let ooze = TokenDefinition {
        name: "Ooze".into(),
        colors: vec![crate::mana::Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    };
    CardDefinition {
        name: "Slime Against Humanity",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: ooze,
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Sum(vec![Value::Const(2), Value::OozesInExileAndGraveyard]),
            },
        ]),
        ..Default::default()
    }
}

/// Magnetic Snuffler — {5} Artifact Creature — Construct 4/4. When it enters,
/// return target Equipment card from your graveyard to the battlefield attached
/// to this creature. Whenever you sacrifice an artifact, put a +1/+1 counter on
/// this creature.
pub fn magnetic_snuffler() -> CardDefinition {
    CardDefinition {
        name: "Magnetic Snuffler",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::Attach {
                    what: Selector::LastMoved,
                    to: Selector::This,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(crate::effect::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact,
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Cryptic Coat — {2}{U} Artifact — Equipment. When it enters, cloak the top
/// card of your library, then attach this to it. Equipped creature gets +1/+0
/// and can't be blocked. {1}{U}: Return this Equipment to its owner's hand.
pub fn cryptic_coat() -> CardDefinition {
    use crate::effect::ActivatedAbility;
    CardDefinition {
        name: "Cryptic Coat",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 0,
            keywords: vec![Keyword::Unblockable],
            ..Default::default()
        }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Cloak {
                who: PlayerRef::You,
                amount: Value::ONE,
            },
            Effect::Attach {
                what: Selector::This,
                to: Selector::LastMoved,
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Outrageous Robbery — {X}{B}{B} Instant. Target opponent exiles the top X
/// cards of their library face down. You may look at and play those cards for
/// as long as they remain exiled, spending mana as though it were any type to
/// cast them. (Player target modeled as "target player".)
pub fn outrageous_robbery() -> CardDefinition {
    use crate::card::MayPlayDuration;
    CardDefinition {
        name: "Outrageous Robbery",
        cost: cost(&[crate::mana::x(), b(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::Target(0),
            count: Value::XFromCost,
            duration: MayPlayDuration::WhileExiled,
            pay_any_color: true,
            pay_own_cost: false,
            uncast_penalty: None,
        },
        ..Default::default()
    }
}

/// Presumed Dead — {1}{B} Instant. Until end of turn, target creature gets +2/+0
/// and gains "When this creature dies, return it to the battlefield under its
/// owner's control and suspect it."
pub fn presumed_dead() -> CardDefinition {
    use crate::effect::Duration;
    let revive = crate::card::TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::Seq(vec![
            Effect::ReturnSelf,
            Effect::Suspect {
                what: Selector::This,
            },
        ]),
    };
    CardDefinition {
        name: "Presumed Dead",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
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
