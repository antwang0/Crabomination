//! Commander: the cards the **Exquisite Invention** precon (C18, Saheeli, the
//! Gifted) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_saheeli.rs`.
//!
//! Residuals (each also on its card):
//! - **Brudiclad, Telchor Engineer** — the token the others copy is your
//!   greatest-power token, not a free choice.
//! - **Prototype Portal** — the imprint takes the first artifact card in hand.
//! - **Tawnos, Urza's Apprentice** — Gogo's `CopyAbility`: the target is the
//!   ability's source permanent, and the copy keeps the original's targets.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::mana::{cost, generic, r, u, x, Color, ManaCost};
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

fn token_copy(source: Selector, extra_card_types: Vec<CardType>) -> Effect {
    Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source,
        extra_creature_types: vec![],
        extra_card_types,
        override_pt: None,
        override_colors: None,
        enters_tapped: false,
        non_legendary: false,
        legendary: false,
        extra_keywords: vec![],
    }
}

fn artifact_token(name: &str, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn servo() -> TokenDefinition {
    artifact_token("Servo", vec![CreatureType::Servo], 1, 1, vec![])
}

fn mint(token: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(token) }
}

fn your_combat() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(crate::game::types::TurnStep::BeginCombat), EventScope::YourControl)
}

/// Saheeli, the Gifted — +1: a Servo; +1: the next spell this turn has
/// affinity for artifacts; −7: a hasty token copy of each of your artifacts,
/// exiled at the next end step. Can be your commander.
pub fn saheeli_the_gifted() -> CardDefinition {
    CardDefinition {
        name: "Saheeli, the Gifted",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Saheeli], ..Default::default() },
        base_loyalty: 4,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility { loyalty_cost: 1, effect: mint(servo()), ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::NextSpellHasAffinityForArtifacts,
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::ForEach {
                    selector: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    body: Box::new(Effect::CreateTokenCopiesHasteSac {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        source: Selector::TriggerSource,
                        exile: true,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Brudiclad, Telchor Engineer — your creature tokens have haste; each combat
/// on your turn a 2/1 Phyrexian Myr, then your other tokens may all become
/// copies of one of them. Residual: the model is your greatest-power token.
pub fn brudiclad_telchor_engineer() -> CardDefinition {
    let mut myr = artifact_token("Phyrexian Myr", vec![CreatureType::Phyrexian, CreatureType::Myr], 2, 1, vec![]);
    myr.colors = vec![Color::Blue];
    let yours = || R::IsToken.and(R::ControlledByYou);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        static_abilities: vec![StaticAbility {
            description: "Creature tokens you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(yours())),
                keyword: Keyword::Haste,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: your_combat(),
            effect: Effect::Seq(vec![
                mint(myr),
                Effect::MayDo {
                    description: "Have each other token you control become a copy of one of them?".into(),
                    body: Box::new(Effect::ForEach {
                        selector: Selector::EachPermanent(yours()),
                        body: Box::new(Effect::BecomeCopyOf {
                            what: Selector::TriggerSource,
                            source: Selector::GreatestPowerControlledMatching(yours()),
                            extra_creature_types: vec![],
                            keep_own_triggered: false,
                            keep_own_activated: false,
                        }),
                    }),
                },
            ]),
        }],
        ..creature(
            "Brudiclad, Telchor Engineer",
            cost(&[generic(4), u(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Artificer],
            4,
            4,
        )
    }
}

/// Echo Storm — a token copy of target artifact, copied once more per
/// command-zone cast of your commander.
pub fn echo_storm() -> CardDefinition {
    CardDefinition {
        name: "Echo Storm",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: token_copy(target_filtered(R::Artifact), vec![]),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::This,
                count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Forge of Heroes — {C}, or a +1/+1 counter (creature) or loyalty counter
/// (planeswalker) on a commander that entered this turn.
pub fn forge_of_heroes() -> CardDefinition {
    CardDefinition {
        name: "Forge of Heroes",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::Creature },
                    then: Box::new(Effect::AddCounter {
                        what: target_filtered(R::IsCommander.and(R::EnteredThisTurn)),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Loyalty,
                        amount: Value::ONE,
                    }),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Geode Golem — trample; connecting, you may cast your commander from the
/// command zone without paying its mana cost (the tax is still owed).
pub fn geode_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Cast your commander without paying its mana cost?".into(),
                body: Box::new(Effect::CastCommanderWithoutPaying),
            },
        }],
        ..creature("Geode Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 5, 3)
    }
}

/// Highland Lake — enters tapped; {U} or {R}.
pub fn highland_lake() -> CardDefinition {
    let mut d = crate::sets::dual_land_untyped("Highland Lake", Color::Blue, Color::Red, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d
}

/// Inkwell Leviathan — trample, islandwalk, shroud.
pub fn inkwell_leviathan() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Trample, Keyword::Landwalk(LandType::Island), Keyword::Shroud],
        ..creature(
            "Inkwell Leviathan",
            cost(&[generic(7), u(), u()]),
            vec![CreatureType::Leviathan],
            7,
            11,
        )
    }
}

/// Loyal Drake — flying; lieutenant: draw a card each combat on your turn.
pub fn loyal_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: your_combat().with_filter(Predicate::ControlsOwnCommander { who: PlayerRef::You }),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature("Loyal Drake", cost(&[generic(2), u()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Prototype Portal — imprint an artifact card from hand; {X}, {T}: a token
/// copy of it, X its mana value. Residual: it imprints the first artifact
/// card in hand.
pub fn prototype_portal() -> CardDefinition {
    CardDefinition {
        name: "Prototype Portal",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Imprint: exile an artifact card from your hand?".into(),
            body: Box::new(Effect::ExileTaggedWithSource {
                what: Selector::take(
                    Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Hand, filter: R::Artifact },
                    Value::ONE,
                ),
            }),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost_increase: Some(Value::ManaValueOf(Box::new(Selector::CardExiledWithSource))),
            effect: token_copy(Selector::CardExiledWithSource, vec![]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Retrofitter Foundry — {3}: untap; {2}, {T}: a Servo; {1}, {T}, a Servo: a
/// Thopter; {T}, a Thopter: a 4/4 Construct.
pub fn retrofitter_foundry() -> CardDefinition {
    CardDefinition {
        name: "Retrofitter Foundry",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Untap { what: Selector::This, up_to: None },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: mint(servo()),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Servo), 1)),
                effect: mint(artifact_token("Thopter", vec![CreatureType::Thopter], 1, 1, vec![Keyword::Flying])),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_other_filter: Some((R::HasCreatureType(CreatureType::Thopter), 1)),
                effect: mint(artifact_token("Construct", vec![CreatureType::Construct], 4, 4, vec![])),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Reverse Engineer — improvise; draw three.
pub fn reverse_engineer() -> CardDefinition {
    CardDefinition {
        name: "Reverse Engineer",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Improvise],
        effect: Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ..Default::default()
    }
}

/// Saheeli's Artistry — choose one or both: a token copy of target artifact;
/// a token copy of target creature that's also an artifact.
pub fn saheelis_artistry() -> CardDefinition {
    CardDefinition {
        name: "Saheeli's Artistry",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseN {
            picks: vec![0, 1],
            modes: vec![
                token_copy(target_filtered(R::Artifact), vec![]),
                token_copy(target_filtered(R::Creature), vec![CardType::Artifact]),
            ],
        },
        ..Default::default()
    }
}

/// Saheeli's Directive — improvise; reveal the top X, deploy the artifact
/// cards with mana value X or less, bin the rest.
pub fn saheelis_directive() -> CardDefinition {
    CardDefinition {
        name: "Saheeli's Directive",
        cost: cost(&[x(), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Improvise],
        effect: Effect::LookTopPutMatchingOntoBattlefield {
            count: Value::XFromCost,
            filter: R::Artifact.and(R::ManaValueAtMostXFromCost),
            then: None,
            max: None,
            tapped: false,
            exile_rest: false,
            rest_to_graveyard: true,
        },
        ..Default::default()
    }
}

/// Tawnos, Urza's Apprentice — haste; {U}{R}, {T}: copy an activated or
/// triggered ability you control from an artifact source. Residual: as
/// Strionic Resonator, the target is the source permanent and the copy keeps
/// its targets.
pub fn tawnos_urzas_apprentice() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), r()]),
            tap_cost: true,
            effect: Effect::CopyAbility {
                what: target_filtered(R::HasAbilityOnStack.and(R::ControlledByYou).and(R::Artifact)),
                times: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Tawnos, Urza's Apprentice",
            cost(&[u(), r()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Treasure Nabber — an opponent's artifact tapped for mana is yours until
/// the end of your next turn.
pub fn treasure_nabber() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TappedForMana, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact },
            ),
            effect: Effect::GainControl {
                what: Selector::TriggerSource,
                to: None,
                duration: Duration::UntilEndOfYourNextTurn,
            },
        }],
        ..creature(
            "Treasure Nabber",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            3,
            2,
        )
    }
}

/// Varchild, Betrayer of Kjeldor — a player it hits makes that many 1/1
/// Survivors; opponents' Survivors can't block or attack you; when Varchild
/// leaves, you take every Survivor.
pub fn varchild_betrayer_of_kjeldor() -> CardDefinition {
    let survivor = TokenDefinition {
        name: "Survivor".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Survivor], ..Default::default() },
        ..Default::default()
    };
    let theirs = || R::HasCreatureType(CreatureType::Survivor).and(R::ControlledByOpponent);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Survivors your opponents control can't block.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(theirs()),
                    keyword: Keyword::CantBlock,
                },
            },
            StaticAbility {
                description: "Survivors your opponents control can't attack you or planeswalkers you control.",
                effect: StaticEffect::CreaturesCantAttackController {
                    protect_planeswalkers: true,
                    filter: Some(theirs()),
                },
            },
        ],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::TriggerEventPlayer,
                    count: Value::TriggerEventAmount,
                    definition: Arc::new(survivor),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::GainControl {
                    what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Survivor)),
                    to: None,
                    duration: Duration::Permanent,
                },
            },
        ],
        ..creature(
            "Varchild, Betrayer of Kjeldor",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Vessel of Endless Rest — on entry, a card from a graveyard goes to the
/// bottom of its owner's library; taps for any color.
pub fn vessel_of_endless_rest() -> CardDefinition {
    CardDefinition {
        name: "Vessel of Endless Rest",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::InGraveyard),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Bottom,
            },
        })],
        activated_abilities: vec![crate::sets::tap_add_any_color()],
        ..Default::default()
    }
}
