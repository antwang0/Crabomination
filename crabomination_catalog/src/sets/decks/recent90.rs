//! Izzet spells-matter batch: Wizards-tribal payoffs (Adeliz, Balmor, Naru
//! Meha, Docent of Perfection), prowess bodies (Bloodwater Entity, Harmonic
//! Prodigy), spell-copy (Dualcaster Mage), counter engines (Runaway Steam-Kin),
//! I/S-graveyard scalers (Spellheart Chimera, Beacon Bolt, Rise from the
//! Tides), and burn/utility pieces. Tests in `tests/recent90.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, Effect,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{cast_is_instant_or_sorcery, deal, draw, etb, on_dies, target_any};
use crate::effect::{Duration, LibraryPosition, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, u, x, Color};

/// Trigger on "whenever you cast an instant or sorcery spell you control."
fn on_cast_is(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(cast_is_instant_or_sorcery()),
        effect,
    }
}

/// A 1/1 blue Human Wizard token (Docent of Perfection).
fn human_wizard_token() -> TokenDefinition {
    TokenDefinition {
        name: "Human Wizard".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        ..Default::default()
    }
}

fn your_wizards() -> Selector {
    Selector::EachPermanent(R::HasCreatureType(CreatureType::Wizard).and(R::ControlledByYou))
}
fn other_wizards() -> Selector {
    Selector::EachPermanent(
        R::HasCreatureType(CreatureType::Wizard)
            .and(R::ControlledByYou)
            .and(R::OtherThanSource),
    )
}

/// Adeliz, the Cinder Wind — {1}{U}{R} 2/2 Legendary Human Wizard, flying &
/// haste. Cast an I/S → Wizards you control get +1/+1 until end of turn.
pub fn adeliz_the_cinder_wind() -> CardDefinition {
    CardDefinition {
        name: "Adeliz, the Cinder Wind",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![on_cast_is(Effect::PumpPT {
            what: your_wizards(),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Balmor, Battlemage Captain — {U}{R} 1/3 Legendary Bird Wizard, flying. Cast
/// an I/S → creatures you control get +1/+0 and gain trample until end of turn.
pub fn balmor_battlemage_captain() -> CardDefinition {
    let your_creatures =
        || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Balmor, Battlemage Captain",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_cast_is(Effect::Seq(vec![
            Effect::PumpPT {
                what: your_creatures(),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: your_creatures(),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Bloodwater Entity — {1}{U}{R} 2/2 Elemental, flying & prowess. ETB, you may
/// put target I/S card from your graveyard on top of your library.
pub fn bloodwater_entity() -> CardDefinition {
    CardDefinition {
        name: "Bloodwater Entity",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Prowess],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "put target instant or sorcery card from your graveyard on top of your library".into(),
            body: Box::new(Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                },
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            }),
        })],
        ..Default::default()
    }
}

/// Improbable Alliance — {U}{R} Enchantment. Draw your second card each turn →
/// create a 1/1 blue Faerie with flying. {4}{U}{R}: Draw a card, then discard.
pub fn improbable_alliance() -> CardDefinition {
    let faerie = TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Improbable Alliance",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::You, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: faerie },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u(), r()]),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Runaway Steam-Kin — {1}{R} 1/1 Elemental. Cast a red spell → if it has fewer
/// than three +1/+1 counters, put one on it. Remove three +1/+1 counters: add
/// {R}{R}{R}.
pub fn runaway_steam_kin() -> CardDefinition {
    CardDefinition {
        name: "Runaway Steam-Kin",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(Color::Red))),
            effect: Effect::If {
                // "fewer than three" = current count ≤ 2.
                cond: Predicate::ValueAtLeast(
                    Value::Const(2),
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                ),
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 3)),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Red, Color::Red, Color::Red]),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Harmonic Prodigy — {1}{R} 1/3 Human Wizard, prowess. If a triggered ability
/// of a Shaman or another Wizard you control triggers, it triggers an
/// additional time.
pub fn harmonic_prodigy() -> CardDefinition {
    CardDefinition {
        name: "Harmonic Prodigy",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Prowess],
        static_abilities: vec![StaticAbility {
            description: "If a triggered ability of a Shaman or another Wizard you control triggers, that ability triggers an additional time.",
            effect: StaticEffect::DoubleControllerTriggersOfType {
                types: vec![CreatureType::Shaman, CreatureType::Wizard],
                exclude_source: true,
            },
        }],
        ..Default::default()
    }
}

/// Spellheart Chimera — {1}{U}{R} */3 Chimera, flying & trample. Power = the
/// number of instant and sorcery cards in your graveyard.
pub fn spellheart_chimera() -> CardDefinition {
    CardDefinition {
        name: "Spellheart Chimera",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Chimera], ..Default::default() },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        dynamic_pt: Some(DynamicPt::InstantsSorceriesInControllerGraveyard { base_t: 3 }),
        ..Default::default()
    }
}

/// Roil Eruption — {1}{R} Sorcery. Kicker {5}. Deals 3 damage to any target;
/// 5 instead if kicked.
pub fn roil_eruption() -> CardDefinition {
    CardDefinition {
        name: "Roil Eruption",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(5)]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(deal(5, target_any())),
            else_: Box::new(deal(3, target_any())),
        },
        ..Default::default()
    }
}

/// Dualcaster Mage — {1}{R}{R} 2/2 Human Wizard, flash. ETB, copy target I/S
/// spell; you may choose new targets for the copy.
pub fn dualcaster_mage() -> CardDefinition {
    CardDefinition {
        name: "Dualcaster Mage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CopySpellMayChooseTargets {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::IsSpellOnStack.and(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                ),
            },
            count: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Naru Meha, Master Wizard — {2}{U}{U} 3/3 Legendary Human Wizard, flash. ETB,
/// copy target I/S spell you control (may choose new targets). Other Wizards
/// you control get +1/+1.
pub fn naru_meha_master_wizard() -> CardDefinition {
    CardDefinition {
        name: "Naru Meha, Master Wizard",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "Other Wizards you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: other_wizards(), power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![etb(Effect::CopySpellMayChooseTargets {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::IsSpellOnStack
                    .and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)))
                    .and(R::ControlledByYou),
            },
            count: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Docent of Perfection // Final Iteration — {3}{U}{U} 5/4 Insect Horror,
/// flying. Cast an I/S → make a 1/1 blue Human Wizard, then if you control
/// three or more Wizards, transform. Back: Final Iteration (6/5) — Wizards you
/// control get +2/+1 and flying; cast an I/S → make a Wizard.
pub fn docent_of_perfection() -> CardDefinition {
    let make_wizard = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: human_wizard_token(),
    };
    let final_iteration = CardDefinition {
        name: "Final Iteration",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Insect],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Wizards you control get +2/+1.",
                effect: StaticEffect::PumpPT { applies_to: your_wizards(), power: 2, toughness: 1 },
            },
            StaticAbility {
                description: "Wizards you control have flying.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: your_wizards(),
                    keyword: Keyword::Flying,
                },
            },
        ],
        triggered_abilities: vec![on_cast_is(make_wizard())],
        ..Default::default()
    };
    CardDefinition {
        name: "Docent of Perfection",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect, CreatureType::Horror],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_cast_is(Effect::Seq(vec![
            make_wizard(),
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: your_wizards(),
                    n: Value::Const(3),
                },
                then: Box::new(Effect::Transform { what: Selector::This }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        back_face: Some(Box::new(final_iteration)),
        ..Default::default()
    }
}

/// Beacon Bolt — {1}{U}{R} Sorcery. Deals damage to target creature equal to
/// the number of I/S cards you own in exile and in your graveyard. Jump-start.
pub fn beacon_bolt() -> CardDefinition {
    CardDefinition {
        name: "Beacon Bolt",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::JumpStart],
        effect: Effect::DealDamage {
            to: Selector::TargetFiltered { slot: 0, filter: R::Creature },
            amount: {
                let is = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
                Value::Sum(vec![
                    Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: is(),
                    }),
                    Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Exile,
                        filter: is(),
                    }),
                ])
            },
        },
        ..Default::default()
    }
}


/// Archaeomancer — {2}{U}{U} 1/2 Human Wizard. ETB, return target instant or
/// sorcery card from your graveyard to your hand.
pub fn archaeomancer() -> CardDefinition {
    CardDefinition {
        name: "Archaeomancer",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::InYourGraveyard.and(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                ),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Magmatic Insight — {R} Sorcery. Additional cost: discard a land card. Draw
/// two cards.
pub fn magmatic_insight() -> CardDefinition {
    CardDefinition {
        name: "Magmatic Insight",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::Discard {
            count: 1,
            filter: Some(R::Land),
        }],
        effect: draw(2),
        ..Default::default()
    }
}

/// Niv-Mizzet, the Firemind — {2}{U}{U}{R}{R} 4/4 Legendary Dragon Wizard,
/// flying. Whenever you draw a card, deal 1 damage to any target. {T}: Draw.
pub fn niv_mizzet_the_firemind() -> CardDefinition {
    CardDefinition {
        name: "Niv-Mizzet, the Firemind",
        cost: cost(&[generic(2), u(), u(), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: deal(1, target_any()),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..Default::default()
    }
}



/// Cloud Sprite — {U} 1/1 Faerie with flying that can block only flyers.
pub fn cloud_sprite() -> CardDefinition {
    CardDefinition {
        name: "Cloud Sprite",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..Default::default()
    }
}

/// Cinder Elemental — {3}{R} 2/2 Elemental. {X}{R}, {T}, Sacrifice this: it
/// deals X damage to any target.
pub fn cinder_elemental() -> CardDefinition {
    CardDefinition {
        name: "Cinder Elemental",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), r()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Living Lightning — {3}{R} 3/2 Elemental Shaman. When it dies, return target
/// instant or sorcery card from your graveyard to your hand.
pub fn living_lightning() -> CardDefinition {
    CardDefinition {
        name: "Living Lightning",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::TargetFiltered {
                slot: 0,
                filter: R::InYourGraveyard.and(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                ),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Needle Drop — {R} Instant. Deals 1 damage to any target that was dealt
/// damage this turn. Draw a card.
pub fn needle_drop() -> CardDefinition {
    CardDefinition {
        name: "Needle Drop",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::DealtDamageThisTurn.and(
                        R::Creature.or(R::Player).or(R::Planeswalker),
                    ),
                },
                amount: Value::Const(1),
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Rise from the Tides — {5}{U} Sorcery. Create a tapped 2/2 black Zombie for
/// each instant and sorcery card in your graveyard.
pub fn rise_from_the_tides() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        name: "Rise from the Tides",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::count(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Graveyard,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            definition: zombie,
        },
        ..Default::default()
    }
}

/// Storm Fleet Aerialist — {1}{U} 1/2 Human Pirate, flying. Raid — enters with
/// a +1/+1 counter if you attacked this turn.
pub fn storm_fleet_aerialist() -> CardDefinition {
    CardDefinition {
        name: "Storm Fleet Aerialist",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
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

/// Chandra's Spitfire — {2}{R} 1/3 Elemental, flying. Whenever an opponent is
/// dealt noncombat damage, this creature gets +3/+0 until end of turn.
pub fn chandras_spitfire() -> CardDefinition {
    CardDefinition {
        name: "Chandra's Spitfire",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::PlayerDealtNoncombatDamage,
                EventScope::OpponentControl,
            ),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}
