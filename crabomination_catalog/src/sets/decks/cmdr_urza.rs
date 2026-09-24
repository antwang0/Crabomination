//! Commander: the cards the **Urza's Iron Alliance** precon (BRC, Urza, Chief
//! Artificer) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_urza.rs`.
//!
//! Residuals (each also on its card):
//! - **Sanwell, Avenger Ace** — the cast offer is the first matching card
//!   exiled, not a choice among them; the rest go to the bottom in exile
//!   order, not a random one.
//! - **Scholar of New Horizons** — when the Plains may go onto the
//!   battlefield, it always does.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::mana::{b, cost, generic, u, w, Color, ManaCost};
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

fn artifact_creature() -> R {
    R::Artifact.and(R::Creature)
}

fn negative(v: Value) -> Value {
    Value::Diff(Box::new(Value::Const(0)), Box::new(v))
}

/// A token copy of `source` for `who` — the plain "create a token that's a
/// copy of" with every rider off.
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

fn thopter() -> TokenDefinition {
    TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        ..Default::default()
    }
}

/// Alela, Artful Provocateur — flying, deathtouch, lifelink; your other
/// fliers get +1/+0; each artifact or enchantment spell you cast makes a 1/1
/// flying Faerie.
pub fn alela_artful_provocateur() -> CardDefinition {
    let faerie = TokenDefinition {
        name: "Faerie".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control with flying get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource)
                        .and(R::HasKeyword(Keyword::Flying)),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact.or(R::Enchantment))),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(faerie) },
        }],
        ..creature(
            "Alela, Artful Provocateur",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Faerie, CreatureType::Warlock],
            2,
            3,
        )
    }
}

/// Armix, Filigree Thrasher — attacking, you may discard a card; when you do,
/// a creature the defending player controls gets -X/-X, X = your artifacts
/// plus the artifact cards in your graveyard. Partner.
pub fn armix_filigree_thrasher() -> CardDefinition {
    let x = || {
        Value::Sum(vec![
            Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
            Value::CountOf(Box::new(Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::You),
                filter: R::Artifact,
            })),
        ])
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Partner],
        triggered_abilities: vec![on_attack(Effect::MayDiscard {
            description: "Discard a card to shrink a defending creature?".into(),
            count: Value::ONE,
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByDefendingPlayer)),
                power: negative(x()),
                toughness: negative(x()),
                duration: Duration::EndOfTurn,
            }),
            else_: None,
        })],
        ..creature(
            "Armix, Filigree Thrasher",
            cost(&[generic(2), b()]),
            vec![CreatureType::Golem],
            3,
            2,
        )
    }
}

/// Hexavus — enters with six +1/+1 counters; {1} and one of them: a flying
/// counter on another creature; {1} and a counter off another creature of
/// yours: a +1/+1 counter back.
pub fn hexavus() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(6))),
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::AddKeywordCounter {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    keyword: Keyword::Flying,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_among_filter: Some((None, 1, R::Creature.and(R::OtherThanSource))),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature("Hexavus", cost(&[generic(6)]), vec![CreatureType::Construct], 0, 0)
    }
}

/// Indomitable Archangel — flying; metalcraft: your artifacts have shroud.
pub fn indomitable_archangel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Metalcraft — Artifacts you control have shroud as long as you control \
                          three or more artifacts.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::MetalcraftActive { who: PlayerRef::You },
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)),
                    keyword: Keyword::Shroud,
                }),
            },
        }],
        ..creature(
            "Indomitable Archangel",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Angel],
            4,
            4,
        )
    }
}

/// Kayla's Music Box — {W}, {T}: exile your top card face down under it;
/// {T}: this turn, play the cards exiled with it (paying their costs).
pub fn kaylas_music_box() -> CardDefinition {
    CardDefinition {
        name: "Kayla's Music Box",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[w()]),
                tap_cost: true,
                effect: Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::ONE,
                    link_to_source: true,
                    face_down: true,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::GrantMayPlay {
                    what: Selector::CardExiledWithSource,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// March of Progress — a token copy of your artifact creature; overload
/// {6}{U} copies each of them.
pub fn march_of_progress() -> CardDefinition {
    let yours = || artifact_creature().and(R::ControlledByYou);
    CardDefinition {
        name: "March of Progress",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: token_copy(target_filtered(yours()), vec![]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(6), u()]),
            effect_override: Some(Effect::ForEach {
                selector: Selector::EachPermanent(yours()),
                body: Box::new(token_copy(Selector::TriggerSource, vec![])),
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// One with the Machine — draw the greatest mana value among your artifacts.
pub fn one_with_the_machine() -> CardDefinition {
    CardDefinition {
        name: "One with the Machine",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::You,
            amount: Value::HighestManaValueAmong(Box::new(Selector::EachPermanent(
                R::Artifact.and(R::ControlledByYou),
            ))),
        },
        ..Default::default()
    }
}

/// Sanwell, Avenger Ace — damage to it is prevented while an artifact
/// creature of yours attacks; whenever it becomes tapped, exile your top six
/// and you may cast a Vehicle or artifact creature from among them. Residuals:
/// the offer is the first matching card, and the rest are bottomed in exile
/// order.
pub fn sanwell_avenger_ace() -> CardDefinition {
    let exiled = |filter: R| Selector::EachMatching { zone: ZoneRef::Exile, filter: R::ExiledWithSource.and(filter) };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "As long as an artifact creature you control is attacking, prevent all \
                          damage that would be dealt to Sanwell.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    artifact_creature().and(R::ControlledByYou).and(R::IsAttacking),
                )),
                inner: Box::new(StaticEffect::PreventAllDamageToThis),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::Const(6),
                    link_to_source: true,
                    face_down: false,
                },
                Effect::CastWithoutPayingImmediate {
                    what: exiled(R::HasArtifactSubtype(ArtifactSubtype::Vehicle).or(artifact_creature())),
                    source_zone: Zone::Exile,
                    exile_after: false,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: true,
                },
                Effect::Move {
                    what: exiled(R::Any),
                    to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
                },
            ]),
        }],
        ..creature(
            "Sanwell, Avenger Ace",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Pilot],
            3,
            1,
        )
    }
}

/// Scholar of New Horizons — enters with a +1/+1 counter; {T}, remove a
/// counter from a permanent you control: a Plains, onto the battlefield tapped
/// if an opponent has more lands, else to hand. Residual: it always takes the
/// battlefield when it may.
pub fn scholar_of_new_horizons() -> CardDefinition {
    let plains = || R::HasLandType(LandType::Plains);
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_among_filter: Some((None, 1, R::Permanent)),
            effect: Effect::If {
                cond: Predicate::OpponentControlsMoreLandsThanYou,
                then: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: plains(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
                else_: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: plains(),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Scholar of New Horizons",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Tawnos, Solemn Survivor — {2}, {T}: copy up to one of your artifact tokens
/// and mill two; the sorcery-speed big one turns an exiled graveyard card into
/// an artifact token copy.
pub fn tawnos_solemn_survivor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        token_copy(
                            target_filtered(R::Artifact.and(R::IsToken).and(R::ControlledByYou)),
                            vec![],
                        ),
                        Effect::Mill { who: Selector::You, amount: Value::Const(2) },
                    ])),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w(), u(), b()]),
                tap_cost: true,
                sorcery_speed: true,
                sac_other_filter: Some((R::Artifact.and(R::IsToken), 2)),
                exile_other_filter: Some((R::Artifact.or(R::Creature).and(R::InYourGraveyard), 1)),
                effect: token_copy(Selector::CostExiledCards, vec![CardType::Artifact]),
                ..Default::default()
            },
        ],
        ..creature(
            "Tawnos, Solemn Survivor",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            1,
            3,
        )
    }
}

/// Teshar, Ancestor's Apostle — flying; each historic spell you cast returns a
/// creature card with mana value 3 or less from your graveyard.
pub fn teshar_ancestors_apostle() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(crate::card::EnchantmentSubtype::Saga));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(historic)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        }],
        ..creature(
            "Teshar, Ancestor's Apostle",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Thopter Shop — once a turn, a card when your artifact creatures die;
/// {2}{W}, {T}: a 1/1 flying Thopter.
pub fn thopter_shop() -> CardDefinition {
    CardDefinition {
        name: "Thopter Shop",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: artifact_creature() })
                .once_per_turn(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            tap_cost: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(thopter()) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Urza's Ruinous Blast — legendary sorcery: exile every nonland permanent
/// that isn't legendary.
pub fn urzas_ruinous_blast() -> CardDefinition {
    CardDefinition {
        name: "Urza's Ruinous Blast",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                R::Nonland.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
            ),
            body: Box::new(Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile }),
        },
        ..Default::default()
    }
}

/// Vedalken Humiliator — metalcraft: attacking, your opponents' creatures lose
/// all abilities and are base 1/1 until end of turn.
pub fn vedalken_humiliator() -> CardDefinition {
    let theirs = || Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent));
    let mut trigger = on_attack(Effect::Seq(vec![
        Effect::LoseAllAbilities { what: theirs(), duration: Duration::EndOfTurn },
        Effect::SetBasePT {
            what: theirs(),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    ]));
    trigger.event = trigger.event.with_filter(Predicate::MetalcraftActive { who: PlayerRef::You });
    CardDefinition {
        triggered_abilities: vec![trigger],
        ..creature(
            "Vedalken Humiliator",
            cost(&[generic(3), u()]),
            vec![CreatureType::Vedalken, CreatureType::Wizard],
            3,
            4,
        )
    }
}

/// Wire Surgeons — fear; each artifact creature card in your graveyard has
/// encore at its own mana cost.
pub fn wire_surgeons() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        static_abilities: vec![StaticAbility {
            description: "Each artifact creature card in your graveyard has encore. Its encore \
                          cost is equal to its mana cost.",
            effect: StaticEffect::GraveyardCardsHaveEncore { filter: artifact_creature(), mana_cost: true },
        }],
        ..creature(
            "Wire Surgeons",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            6,
            5,
        )
    }
}

/// Wreck Hunter — flash; on entry, a tapped Powerstone for each nonland card
/// that went to the target player's graveyard from the battlefield this turn.
pub fn wreck_hunter() -> CardDefinition {
    let mut stone = crate::game::effects::powerstone_token();
    stone.tapped = true;
    CardDefinition {
        keywords: vec![Keyword::Flash],
        // The chosen player is bound as `TriggerSource` so the target slot
        // is the `ForEach`'s own, which the target walkers answer.
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: target_filtered(R::Player),
            body: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    zone: Zone::Graveyard,
                    filter: R::Nonland.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
                })),
                definition: Arc::new(stone),
            }),
        })],
        ..creature(
            "Wreck Hunter",
            cost(&[b(), b()]),
            vec![CreatureType::Human, CreatureType::Artificer],
            2,
            2,
        )
    }
}
