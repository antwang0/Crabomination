//! Commander: the cards the **Spirit Squadron** precon (VOC, Millicent,
//! Restless Revenant) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_millicent.rs`.
//!
//! Residuals (each also on its card):
//! - **Donal, Herald of Wings** — "only once each turn" is spent when the
//!   trigger fires, even if you decline the copy.
//! - **Haunting Imitation** — the top cards aren't revealed, only read.
//! - **Spectral Arcanist** — the graveyard spell is chosen as a target when
//!   the trigger goes on the stack, not as it resolves.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, PlayerTally, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, partner_with_search, target_filtered, target_n};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, u, w, x};
use crabomination_base::tokens::clue_token;
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant {
            CardType::Instant
        } else {
            CardType::Sorcery
        }],
        effect,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

fn spirit() -> R {
    R::HasCreatureType(CreatureType::Spirit)
}

/// The 1/1 white Spirit with flying the deck mints.
fn spirit_flyer() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Spirit".to_string(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        ..Default::default()
    })
}

fn make_spirits(count: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count,
        definition: spirit_flyer(),
    }
}

fn clues(count: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count,
        definition: Arc::new(clue_token()),
    }
}

/// "Tap up to one target creature."
fn tap_up_to_one() -> Effect {
    Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: R::Creature,
        effect: Box::new(Effect::Tap { what: target_n(0) }),
    }
}

/// Angel of Flight Alabaster — flying; at your upkeep, return target Spirit
/// card from your graveyard to your hand.
pub fn angel_of_flight_alabaster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Move {
                what: target_filtered(spirit().from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..creature(
            "Angel of Flight Alabaster",
            cost(&[generic(4), w()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Breath of the Sleepless — Spirit spells have flash; casting a creature
/// spell on an opponent's turn taps up to one target creature.
pub fn breath_of_the_sleepless() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You may cast Spirit spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: spirit() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::All(vec![
                    Predicate::CastSpellMatches(R::Creature),
                    Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
                ]),
            ),
            effect: tap_up_to_one(),
        }],
        ..enchantment("Breath of the Sleepless", cost(&[generic(3), u()]))
    }
}

/// Disorder in the Court — exile X target creatures, investigate X times;
/// the creatures return tapped at the next end step.
pub fn disorder_in_the_court() -> CardDefinition {
    spell(
        "Disorder in the Court",
        cost(&[x(), w(), u()]),
        true,
        Effect::Seq(vec![
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 8,
                    filter: R::Creature,
                    effect: Box::new(Effect::ExileReturnToOwnerNextEndStep {
                        what: target_n(0),
                        tapped: true,
                    }),
                }),
            },
            clues(Value::XFromCost),
        ]),
    )
}

/// Donal, Herald of Wings — once each turn, you may copy a nonlegendary
/// creature spell with flying you cast; the copy is a 1/1 Spirit too.
///
/// ⚠ Residual: the once-a-turn use is spent when the trigger fires, even if
/// you decline the copy.
pub fn donal_herald_of_wings() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(
                    R::Creature
                        .and(R::HasSupertype(Supertype::Legendary).negate())
                        .and(R::HasKeyword(Keyword::Flying)),
                ))
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Copy that spell as a 1/1 Spirit?".into(),
                body: Box::new(Effect::CopySpellAsOneOneSpirit {
                    what: Selector::TriggerSource,
                }),
            },
        }],
        ..creature(
            "Donal, Herald of Wings",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Drogskol Reinforcements — melee; other Spirits you control have melee;
/// prevent all noncombat damage to Spirits you control.
pub fn drogskol_reinforcements() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Melee],
        static_abilities: vec![
            StaticAbility {
                description: "Other Spirits you control have melee.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(
                        spirit().and(R::ControlledByYou).and(R::OtherThanSource),
                    ),
                    keyword: Keyword::Melee,
                },
            },
            StaticAbility {
                description: "Prevent all noncombat damage that would be dealt to Spirits you control.",
                effect: StaticEffect::PreventNoncombatDamageToMatching {
                    filter: spirit().and(R::ControlledByYou),
                },
            },
        ],
        ..creature(
            "Drogskol Reinforcements",
            cost(&[generic(3), w()]),
            vec![CreatureType::Spirit, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Ethereal Investigator — flying; enters investigating once per opponent;
/// your second draw each turn makes a 1/1 flying Spirit.
pub fn ethereal_investigator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(clues(Value::OpponentCount)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::YourControl),
                effect: make_spirits(Value::ONE),
            },
        ],
        ..creature(
            "Ethereal Investigator",
            cost(&[generic(3), u()]),
            vec![CreatureType::Spirit],
            2,
            3,
        )
    }
}

/// Flood of Tears — return all nonland permanents to their owners' hands;
/// if four or more were nontoken permanents you control, you may put a
/// permanent card from your hand onto the battlefield.
pub fn flood_of_tears() -> CardDefinition {
    let bounce = Effect::Move {
        what: Selector::EachPermanent(R::Nonland),
        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
    };
    spell(
        "Flood of Tears",
        cost(&[generic(4), u(), u()]),
        false,
        Effect::If {
            cond: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(R::Nonland.and(R::NotToken).and(R::ControlledByYou)),
                n: Value::Const(4),
            },
            then: Box::new(Effect::Seq(vec![
                bounce.clone(),
                Effect::PutFromHandOntoBattlefield {
                    who: PlayerRef::You,
                    filter: R::PermanentCard,
                    count: Value::ONE,
                    tapped: false,
                    haste: false,
                    sacrifice_eot: false,
                    return_eot: false,
                    then: None,
                },
            ])),
            else_: Box::new(bounce),
        },
    )
}

/// Ghostly Pilferer — untapping, you may pay {2} to draw; an opponent casting
/// a spell from outside their hand draws you a card; discard a card: it
/// can't be blocked this turn.
pub fn ghostly_pilferer() -> CardDefinition {
    let draw = || Effect::Draw {
        who: Selector::You,
        amount: Value::ONE,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "Pay {2} to draw a card?".into(),
                    mana_cost: cost(&[generic(2)]),
                    body: Box::new(draw()),
                    else_: None,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::Not(Box::new(Predicate::CastFromHand))),
                effect: draw(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Ghostly Pilferer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Spirit, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Haunted Library — an opponent's creature dying, you may pay {1} for a
/// 1/1 flying Spirit.
pub fn haunted_library() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::MayPay {
                description: "Pay {1} to create a 1/1 Spirit?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(make_spirits(Value::ONE)),
                else_: None,
            },
        }],
        ..enchantment("Haunted Library", cost(&[generic(1), w()]))
    }
}

/// Haunting Imitation — each player's top card: a 1/1 flying Spirit token
/// copy of each creature card; none, and it returns to its owner's hand.
///
/// ⚠ Residual: the top cards are read, not revealed.
pub fn haunting_imitation() -> CardDefinition {
    let creatures_on_top = || Selector::MatchingAmong {
        inner: Box::new(Selector::TopOfLibrary {
            who: PlayerRef::EachPlayer,
            count: Value::ONE,
        }),
        filter: R::Creature,
    };
    spell(
        "Haunting Imitation",
        cost(&[generic(2), u()]),
        false,
        Effect::If {
            cond: Predicate::SelectorExists(creatures_on_top()),
            then: Box::new(Effect::ForEach {
                selector: creatures_on_top(),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![CreatureType::Spirit],
                    extra_card_types: vec![],
                    override_pt: Some((1, 1)),
                    override_colors: None,
                    enters_tapped: false,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![Keyword::Flying],
                }),
            }),
            else_: Box::new(Effect::ReturnResolvingSpellToHand),
        },
    )
}

/// Nebelgast Herald — flash, flying; it or another Spirit entering under
/// your control taps target creature an opponent controls.
pub fn nebelgast_herald() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: spirit(),
                }),
            effect: Effect::Tap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..creature(
            "Nebelgast Herald",
            cost(&[generic(2), u()]),
            vec![CreatureType::Spirit],
            2,
            1,
        )
    }
}

/// Occult Epiphany — draw X, discard X, then a 1/1 flying Spirit per card
/// type among the discarded cards.
pub fn occult_epiphany() -> CardDefinition {
    spell(
        "Occult Epiphany",
        cost(&[x(), u()]),
        true,
        Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::XFromCost,
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::XFromCost,
                random: false,
            },
            make_spirits(Value::CardTypesAmong(Box::new(
                Selector::DiscardedThisResolution { filter: R::Any },
            ))),
        ]),
    )
}

/// Priest of the Blessed Graf — at your end step, a 1/1 flying Spirit per
/// opponent who controls more lands than you.
pub fn priest_of_the_blessed_graf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::YourControl,
            ),
            effect: make_spirits(Value::PlayersWithGreaterTally(PlayerTally::LandsControlled)),
        }],
        ..creature(
            "Priest of the Blessed Graf",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

/// Rhoda, Geist Avenger — partner with Timin; vigilance; an opponent's
/// creature becoming tapped other than by attacking grows it.
pub fn rhoda_geist_avenger() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::PartnerWith("Timin, Youthful Geist".into()),
            Keyword::Vigilance,
        ],
        triggered_abilities: vec![
            partner_with_search("Timin, Youthful Geist"),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer)
                    .not_as_attacker()
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::ControlledByOpponent),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Rhoda, Geist Avenger",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Spectral Arcanist — flying; entering, you may cast an instant or sorcery
/// with mana value up to your Spirit count from a graveyard for free, exiled
/// after.
///
/// ⚠ Residual: the spell is chosen as a target when the trigger goes on the
/// stack, not as it resolves.
pub fn spectral_arcanist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::CastWithoutPayingImmediate {
            what: target_filtered(
                R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::ManaValueAtMostYourCount(Box::new(spirit())))
                    .from_any_graveyard(),
            ),
            source_zone: Zone::Graveyard,
            exile_after: true,
            copy: false,
            reduce_generic: 0,
            pay_own_cost: false,
        })],
        ..creature(
            "Spectral Arcanist",
            cost(&[generic(3), u()]),
            vec![CreatureType::Spirit, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Spectral Shepherd — flying; {1}{U}: return target Spirit you control to
/// its owner's hand.
pub fn spectral_shepherd() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::Move {
                what: target_filtered(spirit().and(R::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature(
            "Spectral Shepherd",
            cost(&[generic(2), w()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Sudden Salvation — up to three permanent cards put into graveyards from
/// the battlefield this turn return tapped under their owners' control; draw
/// a card per opponent who controls one of them.
pub fn sudden_salvation() -> CardDefinition {
    spell(
        "Sudden Salvation",
        cost(&[generic(2), w(), w()]),
        true,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 3,
                min_targets: 0,
                filter: R::PermanentCard
                    .and(R::PutIntoGraveyardFromBattlefieldThisTurn)
                    .from_any_graveyard(),
                effect: Box::new(Effect::Move {
                    what: target_n(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::OwnerOfMoved,
                        tapped: true,
                    },
                }),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::OpponentsControllingAnyOf(Box::new(Selector::AllTargets)),
            },
        ]),
    )
}

/// Timin, Youthful Geist — partner with Rhoda; flying; at the beginning of
/// each combat, tap up to one target creature.
pub fn timin_youthful_geist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::PartnerWith("Rhoda, Geist Avenger".into()),
            Keyword::Flying,
        ],
        triggered_abilities: vec![
            partner_with_search("Rhoda, Geist Avenger"),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::AnyPlayer,
                ),
                effect: tap_up_to_one(),
            },
        ],
        ..creature(
            "Timin, Youthful Geist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Spirit],
            3,
            4,
        )
    }
}
