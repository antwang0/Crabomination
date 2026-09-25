//! Commander: the cards the **Grand Larceny** precon (OTC, Gonti, Canny
//! Acquisitor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_gonti.rs`.
//!
//! Residuals (each also on its card):
//! - **Bladegriff Prototype** — the choosing opponent is the engine's pick,
//!   not necessarily the damaged player.
//! - **Extract Brain** — both the opponent's X cards and your cast are the
//!   engine's picks.
//! - **Nashi, Moon Sage's Scion** — you may play any of the exiled cards, not
//!   only one.
//! - **Siphon Insight** — the exiled card may be cast with mana of any type
//!   (the printed "as though any color" excludes colorless).
//! - **Thief of Sanity** / **Siphon Insight** — the exiled card is the
//!   engine's pick.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, SpendRestriction, b, cost, g, generic, u, x};
use std::sync::Arc;

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

fn treasure() -> Arc<TokenDefinition> {
    Arc::new(crabomination_base::tokens::treasure_token())
}

/// "Whenever one or more creatures you control deal combat damage to a
/// player" — one fire per damaged player (CR 603.2c); that player is slot 0.
fn your_creatures_hit() -> EventSpec {
    EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch()
}

fn hits_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

/// "Target opponent": a zero-card draw names slot 0's filter.
fn target_opponent() -> Effect {
    Effect::Draw { who: Selector::TargetFiltered { slot: 0, filter: R::OpponentPlayer }, amount: Value::Const(0) }
}

/// Gonti, Canny Acquisitor — {2}{B}{G}{U} 5/5 Aetherborn Rogue. Spells you
/// cast but don't own cost {1} less; your creatures hitting a player exile
/// that player's top card face down, playable by you with mana of any type.
pub fn gonti_canny_acquisitor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        can_be_commander: true,
        static_abilities: vec![StaticAbility {
            description: "Spells you cast but don't own cost {1} less to cast.",
            effect: StaticEffect::SpellsYouDontOwnCostLess { amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_creatures_hit(),
            effect: Effect::ExileTopFaceDownGrantPlay { library: PlayerRef::Target(0), grantee: PlayerRef::You },
        }],
        ..creature(
            "Gonti, Canny Acquisitor",
            cost(&[generic(2), b(), g(), u()]),
            vec![CreatureType::Aetherborn, CreatureType::Rogue],
            5,
            5,
        )
    }
}

/// Arcane Heist — {2}{U}{U} sorcery. You may cast an instant or sorcery card
/// from an opponent's graveyard free, exiled instead of going to a graveyard;
/// cipher (CR 702.99).
pub fn arcane_heist() -> CardDefinition {
    CardDefinition {
        name: "Arcane Heist",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            // "You may cast" — the free-cast handler asks.
            Effect::CastWithoutPayingImmediate {
                what: target_filtered(
                    R::HasCardType(CardType::Instant)
                        .or(R::HasCardType(CardType::Sorcery))
                        .and(R::InOpponentGraveyard),
                ),
                source_zone: crate::card::Zone::Graveyard,
                exile_after: true,
                copy: false,
                reduce_generic: 0,
                pay_own_cost: false,
            },
            Effect::Cipher,
        ]),
        ..Default::default()
    }
}

/// Bladegriff Prototype — {5} 3/2 artifact Griffin, flying. Hitting a player,
/// it destroys a nonland permanent one of your opponents controls, of an
/// opponent's choice.
/// Residual: the choosing opponent is the engine's pick.
pub fn bladegriff_prototype() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![hits_player(Effect::OpponentChoosesPermanentThen {
            filter: R::Nonland.and(R::ControlledByOpponent),
            body: Box::new(Effect::Destroy { what: Selector::Target(0) }),
        })],
        ..creature("Bladegriff Prototype", cost(&[generic(5)]), vec![CreatureType::Griffin], 3, 2)
    }
}

/// Dream-Thief's Bandana — {2} Equipment, equip {1}. The equipped creature
/// hitting a player exiles that player's top card face down, playable by you
/// with mana of any type.
pub fn dream_thiefs_bandana() -> CardDefinition {
    CardDefinition {
        name: "Dream-Thief's Bandana",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![hits_player(Effect::ExileTopFaceDownGrantPlay {
                library: PlayerRef::Target(0),
                grantee: PlayerRef::You,
            })],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Extract Brain — {X}{U}{B} sorcery. Target opponent chooses X cards from
/// their hand; you may cast a spell from among them free.
/// Residual: both picks are the engine's.
pub fn extract_brain() -> CardDefinition {
    CardDefinition {
        name: "Extract Brain",
        cost: cost(&[x(), u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            target_opponent(),
            Effect::OpponentChoosesXFromHandCastOneFree { who: PlayerRef::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Felix Five-Boots — {2}{B}{G}{U} 5/4 Ooze Rogue. Menace, ward {2}; a
/// trigger of yours caused by your creature hitting a player triggers once
/// more.
pub fn felix_five_boots() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        static_abilities: vec![StaticAbility {
            description: "Your creatures' combat damage to a player triggers your abilities an additional time.",
            effect: StaticEffect::DoubleControllerCombatDamageToPlayerTriggers,
        }],
        ..creature(
            "Felix Five-Boots",
            cost(&[generic(2), b(), g(), u()]),
            vec![CreatureType::Ooze, CreatureType::Rogue],
            5,
            4,
        )
    }
}

/// Heartless Conscription — {6}{B}{B} sorcery. Exile all creatures; you may
/// play each card exiled this way while it stays exiled, with mana of any
/// type. Exile this.
pub fn heartless_conscription() -> CardDefinition {
    CardDefinition {
        name: "Heartless Conscription",
        cost: cost(&[generic(6), b(), b()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::Move { what: Selector::EachPermanent(R::Creature), to: ZoneDest::Exile },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: false,
                any_color: true,
            },
        ]),
        ..Default::default()
    }
}

/// Mind's Dilation — {5}{U}{U} enchantment. An opponent's first spell each
/// turn: they exile their top card; if it's nonland, you may cast it free.
pub fn minds_dilation() -> CardDefinition {
    CardDefinition {
        name: "Mind's Dilation",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::ValueAtMost(
                Value::SpellsCastThisTurn(PlayerRef::TriggerEventPlayer),
                Value::ONE,
            )),
            effect: Effect::ExileTopMayCastFreeIfNonland { who: PlayerRef::TriggerEventPlayer },
        }],
        ..Default::default()
    }
}

/// Nashi, Moon Sage's Scion — {1}{B}{B} 3/2 Rat Ninja. Ninjutsu {3}{B};
/// hitting a player, it exiles each player's top card, playable this turn
/// for life equal to a spell's mana value.
/// Residual: any of them may be played, not only one.
pub fn nashi_moon_sages_scion() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), b()]))],
        triggered_abilities: vec![hits_player(Effect::ExileTopOfEachLibraryMayPlayForLife)],
        ..creature(
            "Nashi, Moon Sage's Scion",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Rat, CreatureType::Ninja],
            3,
            2,
        )
    }
}

/// Orochi Soul-Reaver — {5}{B} 5/4 Snake Ninja Rogue. Ninjutsu {3}{B}; your
/// creatures hitting a player make a Treasure and manifest that player's top
/// card (CR 701.34).
pub fn orochi_soul_reaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(3), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: your_creatures_hit(),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: treasure() },
                Effect::ManifestTopOfLibraryUnderYou { who: PlayerRef::Target(0) },
            ]),
        }],
        ..creature(
            "Orochi Soul-Reaver",
            cost(&[generic(5), b()]),
            vec![CreatureType::Snake, CreatureType::Ninja, CreatureType::Rogue],
            5,
            4,
        )
    }
}

/// Savvy Trader — {3}{G} 3/3 Human Citizen. Entering, it exiles a permanent
/// card from your graveyard you may play while it stays exiled; spells you
/// cast from anywhere but your hand cost {1} less.
pub fn savvy_trader() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::PermanentCard.from_your_graveyard()), to: ZoneDest::Exile },
            Effect::GrantMayPlay {
                what: Selector::Target(0),
                duration: MayPlayDuration::WhileExiled,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Spells you cast from anywhere other than your hand cost {1} less to cast.",
            effect: StaticEffect::NonHandCastCostReduction { amount: 1 },
        }],
        ..creature(
            "Savvy Trader",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Citizen],
            3,
            3,
        )
    }
}

/// Siphon Insight — {U}{B} instant. Look at the top two of target opponent's
/// library, exile one face down (playable by you, any mana), the other to the
/// bottom. Flashback {1}{U}{B}.
/// Residual: the exiled card is the engine's pick and takes mana of any type.
pub fn siphon_insight() -> CardDefinition {
    CardDefinition {
        name: "Siphon Insight",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Flashback(cost(&[generic(1), u(), b()]))],
        effect: Effect::Seq(vec![
            target_opponent(),
            Effect::LookTopExileOneFaceDownMayPlay {
                who: PlayerRef::Target(0),
                count: Value::Const(2),
                rest_to_graveyard: false,
            },
        ]),
        ..Default::default()
    }
}

/// Smirking Spelljacker — {4}{U} 3/3 Djinn Wizard Rogue. Flash, flying;
/// entering, it exiles target spell an opponent controls; attacking with a
/// card exiled with it, you may cast that card free.
pub fn smirking_spelljacker() -> CardDefinition {
    // "You may cast" — the free-cast handler asks.
    let mut attack = on_attack(Effect::CastWithoutPayingImmediate {
        what: Selector::CardExiledWithSource,
        source_zone: crate::card::Zone::Exile,
        exile_after: false,
        copy: false,
        reduce_generic: 0,
        pay_own_cost: false,
    });
    attack.event = attack.event.with_filter(Predicate::ValueAtLeast(Value::CardsExiledWithSourceCount, Value::ONE));
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::ExileSpellLinked {
                what: target_filtered(R::IsSpellOnStack.and(R::ControlledByOpponent)),
            }),
            attack,
        ],
        ..creature(
            "Smirking Spelljacker",
            cost(&[generic(4), u()]),
            vec![CreatureType::Djinn, CreatureType::Wizard, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Thief of Sanity — {1}{U}{B} 2/2 Specter, flying. Hitting a player, it
/// looks at their top three, exiles one face down (castable by you, any
/// mana), the rest to their graveyard.
/// Residual: the exiled card is the engine's pick.
pub fn thief_of_sanity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![hits_player(Effect::LookTopExileOneFaceDownMayPlay {
            who: PlayerRef::Target(0),
            count: Value::Const(3),
            rest_to_graveyard: true,
        })],
        ..creature("Thief of Sanity", cost(&[generic(1), u(), b()]), vec![CreatureType::Specter], 2, 2)
    }
}

/// Thieving Amalgam — {5}{B}{B} 6/7 Ape Snake. At each opponent's upkeep you
/// manifest their top card (CR 701.34); a creature you control but don't own
/// dying drains its owner for 2.
pub fn thieving_amalgam() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::OpponentControl),
                effect: Effect::ManifestTopOfLibraryUnderYou { who: PlayerRef::ActivePlayer },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::Not(Box::new(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::OwnedByYou,
                    })),
                ),
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ]),
            },
        ],
        ..creature(
            "Thieving Amalgam",
            cost(&[generic(5), b(), b()]),
            vec![CreatureType::Ape, CreatureType::Snake],
            6,
            7,
        )
    }
}

/// Thieving Skydiver — {1}{U} 2/1 Merfolk Rogue, flying, kicker {X} (X can't
/// be 0). Kicked, it takes an artifact with mana value X or less, attaching
/// it if it's an Equipment.
pub fn thieving_skydiver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[x()]))],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::All(vec![
                Predicate::SpellWasKicked,
                Predicate::ValueAtLeast(Value::XFromCost, Value::ONE),
            ]),
            then: Box::new(Effect::Seq(vec![
                Effect::GainControl {
                    what: target_filtered(R::Artifact.and(R::ManaValueAtMostXFromCost)),
                    to: None,
                    duration: Duration::Permanent,
                },
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Equipment),
                    },
                    then: Box::new(Effect::Attach { what: Selector::Target(0), to: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..creature(
            "Thieving Skydiver",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Thieving Varmint — {1}{B} 2/1, deathtouch, lifelink. {T}, pay 1 life: two
/// mana of any one color, only for spells you don't own.
pub fn thieving_varmint() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            life_cost: 1,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyOneColor(Value::Const(2))),
                    SpendRestriction::SpellsYouDontOwn,
                ),
            },
            ..Default::default()
        }],
        ..creature("Thieving Varmint", cost(&[generic(1), b()]), vec![CreatureType::Varmint], 2, 1)
    }
}

/// Tower Winder — {1}{G} 1/1 Snake, reach, deathtouch. Entering, it finds a
/// Command Tower in your graveyard, else in your library.
pub fn tower_winder() -> CardDefinition {
    let tower = || R::HasName("Command Tower".into());
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: tower() },
                Value::ONE,
            ),
            then: Box::new(Effect::Move {
                what: Selector::CardsInZone { who: PlayerRef::You, zone: crate::card::Zone::Graveyard, filter: tower() },
                to: ZoneDest::Hand(PlayerRef::You),
            }),
            else_: Box::new(Effect::Search { who: PlayerRef::You, filter: tower(), to: ZoneDest::Hand(PlayerRef::You) }),
        })],
        ..creature("Tower Winder", cost(&[generic(1), g()]), vec![CreatureType::Snake], 1, 1)
    }
}
