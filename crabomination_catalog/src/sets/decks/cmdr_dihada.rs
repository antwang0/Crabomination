//! Commander: the cards the **Legends' Legacy** precon (DMC, Dihada, Binder of
//! Wills) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_dihada.rs`.
//!
//! Residuals (each also on its card):
//! - **Bell Borca** — the noted mana values are every card exiled this turn,
//!   including before Bell Borca entered.
//! - **Bladewing** — damage to a planeswalker doesn't trigger it.
//! - **The Peregrine Dynamo** — the ability copy is Strionic Resonator's (the
//!   target is the ability's source permanent, the copy keeps its targets).
//! - **Verrak** — only a fixed life cost counts as "life paid" (not X or half
//!   your life), and the copy keeps its targets.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mint_treasures, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, w, Color, ManaCost};
use std::sync::Arc;

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legendary() -> R {
    R::HasSupertype(Supertype::Legendary)
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn draw_lose_one() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
    ])
}

fn knight_token() -> TokenDefinition {
    TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        ..Default::default()
    }
}

fn mint(count: Value, token: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(token) }
}

/// Dihada, Binder of Wills — +2 protects a legend until your next turn; −3
/// digs for legends and makes a Treasure per miss; −11 borrows every nonland
/// permanent for the turn.
pub fn dihada_binder_of_wills() -> CardDefinition {
    let until_next = |k: Keyword| Effect::GrantKeyword {
        what: Selector::Target(0),
        keyword: k,
        duration: Duration::UntilYourNextUntap,
    };
    let top_four = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(4) };
    CardDefinition {
        name: "Dihada, Binder of Wills",
        cost: cost(&[generic(1), r(), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Dihada], ..Default::default() },
        base_loyalty: 5,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: target_filtered(R::Creature.and(legendary())),
                        keyword: Keyword::Vigilance,
                        duration: Duration::UntilYourNextUntap,
                    },
                    until_next(Keyword::Lifelink),
                    until_next(Keyword::Indestructible),
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                // The Treasures count the non-legendary cards among the four
                // first: they are exactly the ones the dig bins.
                effect: Effect::Seq(vec![
                    mint(
                        Value::CountOf(Box::new(Selector::MatchingAmong {
                            inner: Box::new(top_four()),
                            filter: R::Not(Box::new(legendary())),
                        })),
                        crate::game::effects::treasure_token(),
                    ),
                    Effect::LookPickToHand(Box::new(LookPick {
                        who: PlayerRef::You,
                        count: Value::Const(4),
                        rest_to_graveyard: true,
                        pick_filter: Some(legendary()),
                        take: Some(Value::Const(4)),
                        ..Default::default()
                    })),
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -11,
                effect: Effect::Seq(vec![
                    Effect::GainControl {
                        what: Selector::EachPermanent(R::Nonland),
                        to: None,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap { what: Selector::EachPermanent(R::Nonland.and(R::ControlledByYou)), up_to: None },
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(R::Nonland.and(R::ControlledByYou)),
                        keyword: Keyword::Haste,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Arvad the Cursed — deathtouch, lifelink; other legendary creatures you
/// control get +2/+2.
pub fn arvad_the_cursed() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Other legendary creatures you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: yours(R::Creature.and(legendary()).and(R::OtherThanSource)),
                power: 2,
                toughness: 2,
            },
        }],
        ..legend(
            "Arvad the Cursed",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Vampire, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Ashling the Pilgrim — {1}{R}: a +1/+1 counter; the third resolution in a
/// turn cashes the counters in as damage to each creature and each player.
pub fn ashling_the_pilgrim() -> CardDefinition {
    let counters = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::NthResolutionThisTurn {
                    branches: vec![
                        Effect::Noop,
                        Effect::Noop,
                        Effect::Seq(vec![
                            Effect::DealDamage {
                                to: Selector::Both(
                                    Box::new(Selector::EachPermanent(R::Creature)),
                                    Box::new(Selector::Player(PlayerRef::EachPlayer)),
                                ),
                                amount: counters(),
                            },
                            Effect::RemoveCounter {
                                what: Selector::This,
                                kind: CounterType::PlusOnePlusOne,
                                amount: counters(),
                            },
                        ]),
                    ],
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Ashling the Pilgrim",
            cost(&[generic(1), r()]),
            vec![CreatureType::Elemental, CreatureType::Shaman],
            1,
            1,
        )
    }
}

/// Bell Borca, Spectral Sergeant — power is the greatest mana value noted as
/// cards were exiled this turn; an impulse draw each upkeep. Residual: the
/// note includes cards exiled before it entered.
pub fn bell_borca_spectral_sergeant() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Bell Borca's power is equal to the greatest number noted for it this turn.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::GreatestManaValueExiledThisTurn,
                toughness: Value::Const(5),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        ..legend(
            "Bell Borca, Spectral Sergeant",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Spirit, CreatureType::Soldier],
            0,
            5,
        )
    }
}

/// Bladewing, Deathless Tyrant — flying, haste; connecting makes a 2/2
/// menace Zombie Knight per creature card in your graveyard.
pub fn bladewing_deathless_tyrant() -> CardDefinition {
    let zombie_knight = TokenDefinition {
        name: "Zombie Knight".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie, CreatureType::Knight], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: mint(
                Value::CountOf(Box::new(Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: R::Creature,
                })),
                zombie_knight,
            ),
        }],
        ..legend(
            "Bladewing, Deathless Tyrant",
            cost(&[generic(5), b(), r()]),
            vec![CreatureType::Dragon, CreatureType::Skeleton],
            6,
            6,
        )
    }
}

/// Cadric, Soul Kindler — the legend rule skips your tokens; {1} copies each
/// other nontoken legend that enters, hasty until the next end step.
pub fn cadric_soul_kindler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "The \"legend rule\" doesn't apply to tokens you control.",
            effect: StaticEffect::LegendRuleDoesntApplyToYourMatching(R::IsToken),
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken.and(legendary()) },
            ),
            effect: Effect::MayPay {
                description: "Pay {1} to copy it?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::CreateTokenCopiesHasteSac {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    exile: false,
                }),
                else_: None,
            },
        }],
        ..legend(
            "Cadric, Soul Kindler",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Dwarf, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Captain Lannery Storm — haste; a Treasure each attack; +1/+0 whenever you
/// sacrifice a Treasure.
pub fn captain_lannery_storm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: mint_treasures(1),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Treasure),
                    },
                ),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Captain Lannery Storm",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            2,
        )
    }
}

/// Garna, the Bloodflame — flash; entering returns this turn's creature cards
/// from your graveyard to hand; your other creatures have haste.
pub fn garna_the_bloodflame() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Creature.and(R::PutIntoGraveyardThisTurn),
            },
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::OtherThanSource)),
                keyword: Keyword::Haste,
            },
        }],
        ..legend(
            "Garna, the Bloodflame",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Gerrard's Hourglass Pendant — flash; extra turns are skipped; {4}, {T},
/// exile it: this turn's dead permanents come back tapped.
pub fn gerrards_hourglass_pendant() -> CardDefinition {
    let permanent = R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Land);
    CardDefinition {
        name: "Gerrard's Hourglass Pendant",
        cost: cost(&[generic(1)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If a player would begin an extra turn, that player skips that turn instead.",
            effect: StaticEffect::PlayersSkipExtraTurns,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::Move {
                what: Selector::EachMatching {
                    zone: ZoneRef::Graveyard(PlayerRef::You),
                    filter: permanent.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                },
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kothophed, Soul Hoarder — flying; each permanent another player owns going
/// to a graveyard from the battlefield draws you a card for 1 life.
pub fn kothophed_soul_hoarder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::OwnedByYou)) },
            ),
            effect: draw_lose_one(),
        }],
        ..legend("Kothophed, Soul Hoarder", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Moira, Urborg Haunt — menace; connecting returns a creature card that died
/// this turn from your graveyard to the battlefield.
pub fn moira_urborg_haunt() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::InYourGraveyard).and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..legend(
            "Moira, Urborg Haunt",
            cost(&[generic(2), b()]),
            vec![CreatureType::Spirit, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Primevals' Glorious Rebirth — legendary sorcery: every legendary permanent
/// card in your graveyard returns.
pub fn primevals_glorious_rebirth() -> CardDefinition {
    CardDefinition {
        name: "Primevals' Glorious Rebirth",
        cost: cost(&[generic(5), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Permanent.and(legendary()),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Shanid, Sleepers' Scourge — menace for your other legends; each legendary
/// land played or legendary spell cast draws a card for 1 life.
pub fn shanid_sleepers_scourge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Other legendary creatures you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(legendary()).and(R::OtherThanSource)),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::CastSpellMatches(legendary())),
                effect: draw_lose_one(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: legendary() },
                ),
                effect: draw_lose_one(),
            },
        ],
        ..legend(
            "Shanid, Sleepers' Scourge",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            4,
        )
    }
}

/// The Circle of Loyalty — affinity for Knights; +1/+1 anthem; a Knight per
/// legendary spell and for {3}{W}, {T}.
pub fn the_circle_of_loyalty() -> CardDefinition {
    CardDefinition {
        name: "The Circle of Loyalty",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        affinity_filter: Some(R::HasCreatureType(CreatureType::Knight).and(R::ControlledByYou)),
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: yours(R::Creature), power: 1, toughness: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(legendary())),
            effect: mint(Value::ONE, knight_token()),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            tap_cost: true,
            effect: mint(Value::ONE, knight_token()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Peregrine Dynamo — haste; {1}, {T}: copy an ability of another
/// non-commander legendary source you control. Residual: Strionic Resonator's
/// copy.
pub fn the_peregrine_dynamo() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::CopyAbility {
                what: target_filtered(
                    R::HasAbilityOnStack
                        .and(R::ControlledByYou)
                        .and(legendary())
                        .and(R::Not(Box::new(R::IsCommander)))
                        .and(R::OtherThanSource),
                ),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("The Peregrine Dynamo", cost(&[generic(3)]), vec![CreatureType::Construct], 1, 5)
    }
}

/// Tyrite Sanctum — {C}; {2}, {T}: a legendary creature becomes a God with a
/// +1/+1 counter; {4}, {T}, sacrifice it: an indestructible counter on a God.
pub fn tyrite_sanctum() -> CardDefinition {
    CardDefinition {
        name: "Tyrite Sanctum",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::AddCreatureTypes {
                        what: target_filtered(R::Creature.and(legendary())),
                        creature_types: vec![CreatureType::God],
                        duration: Duration::Permanent,
                    },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::HasCreatureType(CreatureType::God)),
                    kind: CounterType::Indestructible,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Verrak, Warped Sengir — flying, deathtouch, lifelink; an ability you paid
/// life for may be copied by paying that much again. Residual: fixed life
/// costs only; the copy keeps its targets.
pub fn verrak_warped_sengir() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivatedWithLifePaid, EventScope::YourControl),
            effect: Effect::MayPayLife {
                description: "Pay that much life again to copy the ability?".into(),
                amount: Value::TriggerEventAmount,
                body: Box::new(Effect::CopyAbility { what: Selector::TriggerSource, times: Value::ONE }),
                else_: None,
            },
        }],
        ..legend("Verrak, Warped Sengir", cost(&[generic(1), w(), b()]), vec![CreatureType::Vampire], 2, 2)
    }
}

/// Zeriam, Golden Wind — flying; each Griffin of yours that connects makes a
/// 2/2 flying Griffin.
pub fn zeriam_golden_wind() -> CardDefinition {
    let griffin = TokenDefinition {
        name: "Griffin".into(),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Griffin),
                },
            ),
            effect: mint(Value::ONE, griffin),
        }],
        ..legend("Zeriam, Golden Wind", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 3, 4)
    }
}
