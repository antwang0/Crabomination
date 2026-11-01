//! Star Trek (TRK/TRC) — the fourth batch of EDHREC Star Trek closers: each
//! card here was the last one missing from a single-commander average deck
//! (scan of 2026-11-01). Tests in `tests/recent_b/trek_singles4.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    MayPlayDuration, Predicate, SelectionRequirement as R, Selector, StaticAbility, StationBand, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, station, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, StaticEffect};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

/// A 1/1 blue Faerie creature token with flying (Worf, La Forge).
fn faerie() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    })
}

fn unblockable_this_turn() -> Effect {
    Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn }
}

/// Pavel Chekov, First Posting — {2}{G}{U} legendary 1/4 Human Pilot. Each
/// draw may instead reveal until a land / nonland (Abundance's replacement).
pub fn pavel_chekov_first_posting() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If you would draw, you may instead reveal until a land/nonland.",
            effect: StaticEffect::MayReplaceDrawWithRevealUntilKind,
        }],
        ..legend(
            "Pavel Chekov, First Posting",
            cost(&[generic(2), g(), u()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            1,
            4,
        )
    }
}

/// Scotty, Chief Engineer — {3}{U} legendary 3/4 Human Artificer. Tap an
/// untapped artifact you control: target creature can't be blocked this turn.
pub fn scotty_chief_engineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Artifact.and(R::ControlledByYou), 1)),
            effect: unblockable_this_turn(),
            ..Default::default()
        }],
        ..legend(
            "Scotty, Chief Engineer",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            3,
            4,
        )
    }
}

/// Lieutenant Hikaru Sulu — {2}{G} legendary 3/3 Human Pilot. Explores
/// whenever he becomes tapped; {2}{U}: target creature can't be blocked.
pub fn lieutenant_hikaru_sulu() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Explore { who: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: unblockable_this_turn(),
            ..Default::default()
        }],
        ..legend(
            "Lieutenant Hikaru Sulu",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            3,
            3,
        )
    }
}

/// Worf, Still Not Merry — {3}{R}{W} legendary 5/5 Klingon Warrior Officer.
/// Attacks: a tapped and attacking 1/1 blue flying Faerie per attacking
/// non-Faerie creature, counted as the trigger resolves; each token's
/// defender is chosen (CR 508.4).
pub fn worf_still_not_merry() -> CardDefinition {
    let non_faerie_attackers = R::Creature
        .and(R::IsAttacking)
        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Faerie))));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            // CR 508.4 — no defender is named, so each token's controller
            // picks a defending player (or one's planeswalker).
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CountOf(Box::new(Selector::EachPermanent(non_faerie_attackers))),
                    definition: Arc::new(TokenDefinition { tapped: true, ..(*faerie()).clone() }),
                },
                Effect::JoinCombatAttackingChosen { what: Selector::LastCreatedTokens, cleanup: Default::default() },
            ]),
        }],
        ..legend(
            "Worf, Still Not Merry",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Klingon, CreatureType::Warrior, CreatureType::Officer],
            5,
            5,
        )
    }
}

/// Subcommander T'Pol — {2}{U} legendary 3/1 Vulcan Scientist with flash. ETB
/// modal: counter target activated or triggered ability, or tap up to one
/// target artifact or creature and put a stun counter on it.
pub fn subcommander_t_pol() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::CounterAbility { what: target_filtered(R::HasAbilityOnStack) },
                Effect::ApplyToTargets {
                    min_targets: 0,
                    max_targets: 1,
                    filter: R::Artifact.or(R::Creature),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Tap { what: Selector::Target(0) },
                        Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::ONE },
                    ])),
                },
            ]),
        }],
        ..legend(
            "Subcommander T'Pol",
            cost(&[generic(2), u()]),
            vec![CreatureType::Vulcan, CreatureType::Scientist],
            3,
            1,
        )
    }
}

/// La Forge, Thoughtful as Ever — {3}{U} legendary 2/4 Human Artificer.
/// {T}: a 1/1 blue flying Faerie per permanent you control that you've put a
/// counter on this turn (any kind; `R::YouPutCounterOnThisTurn`).
pub fn la_forge_thoughtful_as_ever() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::ControlledByYou.and(R::YouPutCounterOnThisTurn),
                ))),
                definition: faerie(),
            },
            ..Default::default()
        }],
        ..legend(
            "La Forge, Thoughtful as Ever",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            4,
        )
    }
}

/// Starfleet Communicator — {3} Communicator Equipment, equip {1}. Casting it
/// copies it (the copy is a token, CR 707.10f). Equipped creature has
/// "attacks: draw X, then put up to X lands from hand tapped", X the other
/// creatures you control with a Communicator attached.
pub fn starfleet_communicator() -> CardDefinition {
    let x = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::Creature
                .and(R::ControlledByYou)
                .and(R::OtherThanSource)
                .and(R::EquippedByMatching(Box::new(R::HasArtifactSubtype(ArtifactSubtype::Communicator)))),
        )))
    };
    CardDefinition {
        name: "Starfleet Communicator",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Communicator, ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::CopySpell { what: Selector::This, count: Value::ONE },
        }],
        static_abilities: vec![StaticAbility {
            description: "Equipped creature has \"Whenever this creature attacks, draw X cards, then you may put up to X land cards from your hand onto the battlefield tapped.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::IsHostOfSource,
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: x() },
                        Effect::PutFromHandOntoBattlefield {
                            who: PlayerRef::You,
                            filter: R::Land,
                            count: x(),
                            tapped: true,
                            haste: false,
                            sacrifice_eot: false,
                            return_eot: false,
                            then: None,
                        },
                    ]),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Antagonist Incorporator — {1}{W} 2/3 Borg artifact creature. Artifact
/// creatures you control have "combat damage to a player: you may pay {1} to
/// create a 1/1 colorless Borg artifact creature token".
pub fn antagonist_incorporator() -> CardDefinition {
    let borg = Arc::new(TokenDefinition {
        name: "Borg".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Borg], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "Antagonist Incorporator",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Borg], ..Default::default() },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, you may pay {1}. If you do, create a 1/1 Borg.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Artifact.and(R::Creature).and(R::ControlledByYou),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::MayPay {
                        description: "Pay {1} to create a 1/1 Borg artifact creature token?".into(),
                        mana_cost: cost(&[generic(1)]),
                        body: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: borg }),
                        else_: None,
                    },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Borg Cube — {5}{W}{B} Spacecraft, affinity for artifact creatures.
/// Entering or attacking, assimilate a creature card exiled with it; if you
/// can't, exile up to one target creature (linked, CR 607.2a). Station 12+: a
/// 6/6, and artifact creatures you control have flying.
pub fn borg_cube() -> CardDefinition {
    let your_artifact_creatures = || R::Artifact.and(R::Creature).and(R::ControlledByYou);
    let body = || Effect::If {
        cond: Predicate::ValueAtLeast(Value::CardsExiledWithSourceMatching(R::Creature), Value::ONE),
        then: Box::new(Effect::ChooseOneAmong {
            what: Selector::MatchingAmong { inner: Box::new(Selector::CardExiledWithSource), filter: R::Creature },
            chooser: PlayerRef::You,
            chosen: Box::new(Effect::Assimilate { what: Selector::SeparatedPile { chosen: true } }),
            other: Box::new(Effect::Noop),
        }),
        else_: Box::new(Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 1,
            filter: R::Creature,
            effect: Box::new(Effect::ExileWithSource { what: Selector::Target(0) }),
        }),
    };
    CardDefinition {
        name: "Borg Cube",
        cost: cost(&[generic(5), w(), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        affinity_filter: Some(your_artifact_creatures()),
        triggered_abilities: vec![etb(body()), on_attack(body())],
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 12,
            pt: Some((6, 6)),
            statics: vec![StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(your_artifact_creatures()),
                keyword: Keyword::Flying,
            }],
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Death Is Irrelevant — {6}{B}{B} enchantment. Artifact creatures you control
/// have indestructible; at your end step, assimilate target creature card
/// from a graveyard.
pub fn death_is_irrelevant() -> CardDefinition {
    CardDefinition {
        name: "Death Is Irrelevant",
        cost: cost(&[generic(6), b(), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have indestructible.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                keyword: Keyword::Indestructible,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Assimilate { what: target_filtered(R::Creature.and(R::InGraveyard)) },
        }],
        ..Default::default()
    }
}

/// Gwyndala, the Progeny — {1}{U} legendary 1/1 Vau-N'Akat Advisor. Entering
/// makes Neuroflux Heirloom (an Equipment token, +1/+1, equip {2}) attached
/// to her. {2}: each Equipment token you control becomes a copy of target
/// Equipment you control until end of turn.
pub fn gwyndala_the_progeny() -> CardDefinition {
    let heirloom = TokenDefinition {
        name: "Neuroflux Heirloom".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        ..Default::default()
    };
    let equipment = || R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::ControlledByYou);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo { target: Selector::This, definition: Arc::new(heirloom) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeCopyOfFor {
                what: Selector::EachPermanent(equipment().and(R::IsToken)),
                source: target_filtered(equipment()),
                duration: Duration::EndOfTurn,
                non_legendary: false,
            },
            ..Default::default()
        }],
        ..legend(
            "Gwyndala, the Progeny",
            cost(&[generic(1), u()]),
            vec![CreatureType::VauNAkat, CreatureType::Advisor],
            1,
            1,
        )
    }
}

/// Ensign Pavel Chekov — {3}{U} legendary 1/5 Human Pilot. Entering, each
/// opponent exiles their top X (X = differently named lands you control); for
/// each, you choose a land card of theirs exiled this way and may play it for
/// as long as it remains exiled.
pub fn ensign_pavel_chekov() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEachOpponent {
            body: Box::new(Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::DistinctNamesControlledMatching(R::Land),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::ChooseOneAmong {
                    what: Selector::MatchingAmong { inner: Box::new(Selector::LastMoved), filter: R::Land },
                    chooser: PlayerRef::You,
                    chosen: Box::new(Effect::GrantMayPlay {
                        what: Selector::SeparatedPile { chosen: true },
                        duration: MayPlayDuration::WhileExiled,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: false,
                        any_color: false,
                    }),
                    other: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..legend(
            "Ensign Pavel Chekov",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            1,
            5,
        )
    }
}

/// Borg Queen, Perfection Manifest — {4}{B}{B} legendary 1/4 Borg Noble
/// artifact creature. Artifact creatures you control get +2/+0; entering
/// assimilates target creature card from an opponent's graveyard.
pub fn borg_queen_perfection_manifest() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control get +2/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                power: 2,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![etb(Effect::Assimilate {
            what: target_filtered(R::Creature.and(R::InGraveyard).and(R::Not(Box::new(R::OwnedByYou)))),
        })],
        ..legend(
            "Borg Queen, Perfection Manifest",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Borg, CreatureType::Noble],
            1,
            4,
        )
    }
}

/// Q, Stacking the Deck — {2}{R}{R} legendary 4/4 Q. Entering, shuffle, then
/// exile until a nonland card, which gains "Exile a card from your hand: copy
/// this card; you may cast the copy" — only during your turn, only while it's
/// exiled (CR 113.6; `Effect::CopyCardMayCast`).
pub fn q_stacking_the_deck() -> CardDefinition {
    let gained = ActivatedAbility {
        from_exile: true,
        exile_from_hand_cost: Some(R::Any),
        condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
        effect: Effect::CopyCardMayCast { what: Selector::This },
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ShuffleLibrary { who: PlayerRef::You },
            Effect::ExileTopUntilNonland { who: PlayerRef::You },
            Effect::GainActivatedAbility {
                what: Selector::ExiledThisResolution { filter: R::Nonland },
                ability: Box::new(gained),
                duration: Duration::Permanent,
            },
        ]))],
        ..legend("Q, Stacking the Deck", cost(&[generic(2), r(), r()]), vec![CreatureType::Q], 4, 4)
    }
}
