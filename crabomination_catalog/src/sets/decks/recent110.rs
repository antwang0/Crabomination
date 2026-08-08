//! Modern-deck staples batch 110 — Fortify (CR 702.71), the remaining
//! no-mana-cost suspend classics (Restore Balance / Wheel of Fate /
//! Hypergenesis), and assorted archetype staples. Tests in
//! `tests/recent110.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement,
    StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::ManaPayload;
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::mana::{
    Color, ManaCost, ManaSymbol, SpendRestriction, b, cost, g, generic, hybrid, r, u, w,
};
use crate::sets::tap_add;

fn target_creature() -> Selector {
    Selector::TargetFiltered {
        slot: 0,
        filter: SelectionRequirement::Creature,
    }
}

/// Darksteel Garrison — {2} Artifact — Fortification. Fortified land has
/// indestructible; when it becomes tapped, target creature gets +1/+1 EOT.
/// Fortify {3} (CR 702.71).
pub fn darksteel_garrison() -> CardDefinition {
    CardDefinition {
        name: "Darksteel Garrison",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Fortification],
            ..Default::default()
        },
        keywords: vec![Keyword::Fortify(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Indestructible],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: target_creature(),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Saffi Eriksdotter — {G}{W} 2/2 Legendary Human Scout. Sacrifice: when
/// target creature dies this turn, return it to the battlefield.
pub fn saffi_eriksdotter() -> CardDefinition {
    CardDefinition {
        name: "Saffi Eriksdotter",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: false,
                    },
                }),
                slot: 0,
                filter: Some(SelectionRequirement::Creature),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Restore Balance — Sorcery, no mana cost. Suspend 6—{W}. Balance lands,
/// creatures, and hands down to the fewest.
pub fn restore_balance() -> CardDefinition {
    CardDefinition {
        name: "Restore Balance",
        no_mana_cost: true,
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(6, cost(&[w()]))],
        effect: Effect::Balance,
        ..Default::default()
    }
}

/// Wheel of Fate — Sorcery, no mana cost. Suspend 4—{1}{R}. Each player
/// discards their hand, then draws seven.
pub fn wheel_of_fate() -> CardDefinition {
    CardDefinition {
        name: "Wheel of Fate",
        no_mana_cost: true,
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(4, cost(&[generic(1), r()]))],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(100),
                random: false,
            },
            Effect::Draw {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(7),
            },
        ]),
        ..Default::default()
    }
}

/// Hypergenesis — Sorcery, no mana cost. Suspend 3—{1}{G}{G}. Each player
/// puts artifact/creature/enchantment/land cards from hand onto the
/// battlefield (the alternating one-at-a-time loop collapses to "all").
pub fn hypergenesis() -> CardDefinition {
    CardDefinition {
        name: "Hypergenesis",
        no_mana_cost: true,
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(3, cost(&[generic(1), g(), g()]))],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::EachPlayer,
                zone: crate::card::Zone::Hand,
                filter: SelectionRequirement::Or(
                    Box::new(SelectionRequirement::Or(
                        Box::new(SelectionRequirement::Artifact),
                        Box::new(SelectionRequirement::Creature),
                    )),
                    Box::new(SelectionRequirement::Or(
                        Box::new(SelectionRequirement::Enchantment),
                        Box::new(SelectionRequirement::Land),
                    )),
                ),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::OwnerOfMoved,
                tapped: false,
            },
        },
        ..Default::default()
    }
}

/// Kumena's Speaker — {G} 1/1 Merfolk Shaman; +1/+1 while you control
/// another Merfolk or an Island.
pub fn kumenas_speaker() -> CardDefinition {
    CardDefinition {
        name: "Kumena's Speaker",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "+1/+1 as long as you control another Merfolk or an Island.",
            effect: StaticEffect::PumpTeamIf {
                applies_to: Selector::This,
                power: 1,
                toughness: 1,
                keywords: vec![],
                condition: Predicate::Any(vec![
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Merfolk)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    )),
                    Predicate::SelectorExists(Selector::EachPermanent(
                        SelectionRequirement::HasLandType(LandType::Island)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                ]),
            },
        }],
        ..Default::default()
    }
}

/// Wanderwine Hub — Land. Reveal a Merfolk from hand or enter tapped;
/// {T}: Add {W} or {U}.
pub fn wanderwine_hub() -> CardDefinition {
    CardDefinition {
        name: "Wanderwine Hub",
        card_types: vec![CardType::Land],
        activated_abilities: vec![tap_add(Color::White), tap_add(Color::Blue)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::IfRevealFromHand {
                filter: SelectionRequirement::HasCreatureType(CreatureType::Merfolk),
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Tap {
                    what: Selector::This,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Shriekhorn — {1} Artifact; enters with three charge counters.
/// {T}, remove one: target player mills two.
pub fn shriekhorn() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Shriekhorn",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Charge, 1)),
            effect: Effect::Mill {
                who: Selector::Target(0),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Emrakul, the Promised End — {13} 13/13 Legendary Eldrazi; costs {1} less
/// per card type in your graveyard; flying, trample, protection from
/// instants. The cast-trigger mind-control turn is unmodeled.
pub fn emrakul_the_promised_end() -> CardDefinition {
    CardDefinition {
        name: "Emrakul, the Promised End",
        cost: cost(&[generic(13)]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 13,
        toughness: 13,
        keywords: vec![
            Keyword::Flying,
            Keyword::Trample,
            Keyword::ProtectionFromInstants,
        ],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each card type among cards in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerCardTypeInGraveyard,
        }],
        ..Default::default()
    }
}

/// Worldspine Wurm — {8}{G}{G}{G} 15/15 trample; dies → three 5/5 Wurm
/// tokens with trample; shuffled back when put into a graveyard.
pub fn worldspine_wurm() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Worldspine Wurm",
        cost: cost(&[generic(8), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 15,
        toughness: 15,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOfMoved,
                        pos: crate::effect::LibraryPosition::Shuffled,
                    },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(3),
                    definition: Box::new(TokenDefinition {
                        name: "Wurm".into(),
                        power: 5,
                        toughness: 5,
                        colors: vec![Color::Green],
                        card_types: vec![CardType::Creature],
                        keywords: vec![Keyword::Trample],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Wurm],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Martyr of Sands — {W} 1/1 Human Cleric. {1}, Sacrifice: gain 3 life per
/// white card in your hand (auto-revealed).
pub fn martyr_of_sands() -> CardDefinition {
    CardDefinition {
        name: "Martyr of Sands",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::CardsInHandMatching {
                        who: PlayerRef::You,
                        filter: SelectionRequirement::HasColor(Color::White),
                    }),
                    Box::new(Value::Const(3)),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Proclamation of Rebirth — {2}{W} Sorcery. Return up to three MV≤1
/// creature cards from your graveyard to the battlefield. Forecast —
/// {5}{W}: return one.
pub fn proclamation_of_rebirth() -> CardDefinition {
    let small = SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(1));
    CardDefinition {
        name: "Proclamation of Rebirth",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::Take {
                inner: Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: small.clone(),
                }),
                count: Box::new(Value::Const(3)),
            },
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        },
        activated_abilities: vec![crate::effect::shortcut::forecast(
            cost(&[generic(5), w()]),
            Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: small,
                    }),
                    count: Box::new(Value::Const(1)),
                },
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        )],
        ..Default::default()
    }
}

/// Prismatic Omen — {1}{G} Enchantment. Lands you control are every basic
/// land type in addition to their other types.
pub fn prismatic_omen() -> CardDefinition {
    CardDefinition {
        name: "Prismatic Omen",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Lands you control are every basic land type in addition to their other types.",
            effect: StaticEffect::GrantAllBasicLandTypes {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Norin the Wary — {R} 2/1 Legendary Human Warrior. Any spell cast or
/// creature attacking exiles him until the next end step.
pub fn norin_the_wary() -> CardDefinition {
    let dodge = Effect::ExileReturnNextEndStep {
        what: Selector::This,
    };
    CardDefinition {
        name: "Norin the Wary",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                effect: dodge.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer),
                effect: dodge,
            },
        ],
        ..Default::default()
    }
}

/// Genesis Chamber — {2} Artifact. Whenever a nontoken creature enters, if
/// this is untapped, its controller creates a 1/1 colorless Myr.
pub fn genesis_chamber() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Genesis Chamber",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature.and(SelectionRequirement::Not(
                            Box::new(SelectionRequirement::IsToken),
                        )),
                    },
                    Predicate::EntityMatches {
                        what: Selector::This,
                        filter: SelectionRequirement::Untapped,
                    },
                ]),
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                count: Value::Const(1),
                definition: Box::new(TokenDefinition {
                    name: "Myr".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Myr],
                        ..Default::default()
                    },
                    ..Default::default()
                }),
            },
        }],
        ..Default::default()
    }
}

/// Entreat the Angels — {X}{X}{W}{W}{W} Sorcery. Create X 4/4 white Angels
/// with flying. Miracle {X}{W}{W}.
pub fn entreat_the_angels() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Entreat the Angels",
        cost: ManaCost::new(vec![
            ManaSymbol::X,
            ManaSymbol::X,
            ManaSymbol::Colored(Color::White),
            ManaSymbol::Colored(Color::White),
            ManaSymbol::Colored(Color::White),
        ]),
        card_types: vec![CardType::Sorcery],
        miracle: Some(ManaCost::new(vec![
            ManaSymbol::X,
            ManaSymbol::Colored(Color::White),
            ManaSymbol::Colored(Color::White),
        ])),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: Box::new(TokenDefinition {
                name: "Angel".into(),
                power: 4,
                toughness: 4,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                keywords: vec![Keyword::Flying],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Angel],
                    ..Default::default()
                },
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}

/// Fracturing Gust — {2}{G/W}{G/W}{G/W} Instant. Destroy all artifacts and
/// enchantments; gain 2 life per permanent destroyed.
pub fn fracturing_gust() -> CardDefinition {
    CardDefinition {
        name: "Fracturing Gust",
        cost: cost(&[
            generic(2),
            hybrid(Color::Green, Color::White),
            hybrid(Color::Green, Color::White),
            hybrid(Color::Green, Color::White),
        ]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::EachPermanent(SelectionRequirement::Or(
                    Box::new(SelectionRequirement::Artifact),
                    Box::new(SelectionRequirement::Enchantment),
                )),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Times(
                    Box::new(Value::PermanentsDestroyedThisResolution),
                    Box::new(Value::Const(2)),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Hurkyl's Recall — {1}{U} Instant. Return all artifacts target player
/// owns to their hand.
pub fn hurkyls_recall() -> CardDefinition {
    CardDefinition {
        name: "Hurkyl's Recall",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        // "owns" approximated as "controls".
        effect: Effect::Move {
            what: Selector::ControlledBy {
                who: PlayerRef::Target(0),
                filter: SelectionRequirement::Artifact,
            },
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Slippery Scoundrel — {2}{U} 2/2 Human Pirate. Ascend; hexproof and
/// unblockable with the city's blessing.
pub fn slippery_scoundrel() -> CardDefinition {
    CardDefinition {
        name: "Slippery Scoundrel",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Ascend {
                    who: PlayerRef::You,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Ascend {
                    who: PlayerRef::You,
                },
            },
        ],
        static_abilities: vec![
            StaticAbility {
                description: "Hexproof and can't be blocked while you have the city's blessing.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Hexproof,
                    condition: Predicate::HasCityBlessing {
                        who: PlayerRef::You,
                    },
                },
            },
            StaticAbility {
                description: "",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Unblockable,
                    condition: Predicate::HasCityBlessing {
                        who: PlayerRef::You,
                    },
                },
            },
        ],
        ..Default::default()
    }
}

/// Tempest Djinn — {U}{U}{U} 0/4 flying Djinn; +1/+0 per basic Island you
/// control.
pub fn tempest_djinn() -> CardDefinition {
    CardDefinition {
        name: "Tempest Djinn",
        cost: cost(&[u(), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(crate::card::DynamicPt::PowerPlusLandsOfTypeControlled {
            land_type: LandType::Island,
            base_p: 0,
            base_t: 4,
        }),
        ..Default::default()
    }
}

/// Undercity Informer — {2}{B} 2/3 Human Rogue. {1}, Sacrifice a creature:
/// target player mills until they hit a land.
pub fn undercity_informer() -> CardDefinition {
    CardDefinition {
        name: "Undercity Informer",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::MillUntilLands {
                who: Selector::Target(0),
                lands: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Runeflare Trap — {4}{R}{R} Instant — Trap; {R} if an opponent drew 3+
/// this turn. Damage to target player = their hand size.
pub fn runeflare_trap() -> CardDefinition {
    use crate::card::AlternativeCost;
    CardDefinition {
        name: "Runeflare Trap",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Instant],
        subtypes: Subtypes {
            spell_subtypes: vec![crate::card::SpellSubtype::Trap],
            ..Default::default()
        },
        alternative_cost: Some(AlternativeCost {
            awaken: false,
            mana_cost: cost(&[r()]),
            condition: Some(Predicate::PlayerDrewAtLeastThisTurn {
                who: PlayerRef::EachOpponent,
                n: 3,
            }),
            ..Default::default()
        }),
        effect: Effect::DealDamage {
            to: Selector::Target(0),
            amount: Value::HandSizeOf(PlayerRef::Target(0)),
        },
        ..Default::default()
    }
}

/// Molten Psyche — {1}{R}{R} Sorcery. Each player shuffles their hand into
/// their library and draws that many. Metalcraft — each opponent takes
/// damage equal to their cards drawn this turn (exact in 1v1).
pub fn molten_psyche() -> CardDefinition {
    CardDefinition {
        name: "Molten Psyche",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ShuffleHandsDrawSame {
                who: PlayerRef::EachPlayer,
            },
            Effect::If {
                cond: Predicate::MetalcraftActive {
                    who: PlayerRef::You,
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::CardsDrawnThisTurn(PlayerRef::EachOpponent),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Fevered Visions — {1}{U}{R} Enchantment. Each player's end step: they
/// draw; an opponent holding 4+ cards takes 2.
pub fn fevered_visions() -> CardDefinition {
    CardDefinition {
        name: "Fevered Visions",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::End),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(1),
                },
                Effect::If {
                    cond: Predicate::All(vec![
                        Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                        Predicate::ValueAtLeast(
                            Value::HandSizeOf(PlayerRef::ActivePlayer),
                            Value::Const(4),
                        ),
                    ]),
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ActivePlayer),
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Master of the Feast — {1}{B}{B} 5/5 flying Demon; your upkeep: each
/// opponent draws a card.
pub fn master_of_the_feast() -> CardDefinition {
    CardDefinition {
        name: "Master of the Feast",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::types::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Spiteful Visions — {2}{B/R}{B/R} Enchantment. Each draw step: extra
/// draw; every draw pings its drawer for 1.
pub fn spiteful_visions() -> CardDefinition {
    CardDefinition {
        name: "Spiteful Visions",
        cost: cost(&[
            generic(2),
            hybrid(Color::Black, Color::Red),
            hybrid(Color::Black, Color::Red),
        ]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::Draw),
                    EventScope::AnyPlayer,
                ),
                effect: Effect::Draw {
                    who: Selector::Player(PlayerRef::ActivePlayer),
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::AnyPlayer),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Tolaria West — Land, enters tapped; {T}: Add {U}. Transmute {1}{U}{U}
/// (searches MV 0).
pub fn tolaria_west() -> CardDefinition {
    use crate::effect::shortcut::transmute;
    CardDefinition {
        name: "Tolaria West",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add(Color::Blue),
            transmute(cost(&[generic(1), u(), u()]), 0),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::This,
            },
        }],
        ..Default::default()
    }
}

/// Boseiju, Who Shelters All — Legendary Land, enters tapped. {T}, pay 2
/// life: add {C}; an instant/sorcery it funds can't be countered.
pub fn boseiju_who_shelters_all() -> CardDefinition {
    CardDefinition {
        name: "Boseiju, Who Shelters All",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 2,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::Const(1))),
                    SpendRestriction::InstantSorceryUncounterable,
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: Selector::This,
            },
        }],
        ..Default::default()
    }
}

/// Pendelhaven — Legendary Land. {T}: Add {G}; {T}: target 1/1 creature
/// gets +1/+2 until end of turn.
pub fn pendelhaven() -> CardDefinition {
    CardDefinition {
        name: "Pendelhaven",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            tap_add(Color::Green),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::PumpPT {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::PowerAtMost(1))
                            .and(SelectionRequirement::PowerAtLeast(1))
                            .and(SelectionRequirement::ToughnessAtMost(1))
                            .and(SelectionRequirement::ToughnessAtLeast(1)),
                    },
                    power: Value::Const(1),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
