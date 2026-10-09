//! Commander: the cards that stood between a most-built commander's EDHREC
//! average deck and a complete pod seat — one or two per deck, gathered here
//! rather than beside a commander of their own. Tests in
//! `tests/recent_b/cmdr_edhrec.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color};
use std::sync::Arc;

fn creature_types(types: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: types, ..Default::default() }
}

fn token_1_1(name: &str, color: Color, kind: CreatureType) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: creature_types(vec![kind]),
        ..Default::default()
    }
}

fn mint(token: TokenDefinition, count: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition: Arc::new(token) }
}

/// Cadira, Caller of the Small — trample; combat damage to a player makes a
/// 1/1 Rabbit for each token you control.
pub fn cadira_caller_of_the_small() -> CardDefinition {
    let tokens = Value::count(Selector::EachPermanent(R::ControlledByYou.and(R::IsToken)));
    CardDefinition {
        name: "Cadira, Caller of the Small",
        cost: cost(&[generic(1), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Orc, CreatureType::Ranger]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: mint(token_1_1("Rabbit", Color::White, CreatureType::Rabbit), tokens),
        }],
        ..Default::default()
    }
}

/// The Unbeatable Squirrel Girl — a Squirrel on entering and attacking;
/// {1}{G}{G}{G} makes one for each Squirrel you control.
pub fn the_unbeatable_squirrel_girl() -> CardDefinition {
    let squirrel = || token_1_1("Squirrel", Color::Green, CreatureType::Squirrel);
    let squirrels = Value::count(Selector::EachPermanent(
        R::ControlledByYou.and(R::HasCreatureType(CreatureType::Squirrel)),
    ));
    CardDefinition {
        name: "The Unbeatable Squirrel Girl",
        cost: cost(&[generic(1), g(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Squirrel, CreatureType::Human, CreatureType::Hero]),
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(mint(squirrel(), Value::ONE)), on_attack(mint(squirrel(), Value::ONE))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), g(), g()]),
            effect: mint(squirrel(), squirrels),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Forced Fruition — each spell an opponent casts draws them seven.
pub fn forced_fruition() -> CardDefinition {
    CardDefinition {
        name: "Forced Fruition",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::Draw { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(7) },
        }],
        ..Default::default()
    }
}

/// Lesser Masticore — discard a card to cast it; {4}: 1 damage to target
/// creature; persist.
pub fn lesser_masticore() -> CardDefinition {
    CardDefinition {
        name: "Lesser Masticore",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Masticore]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        additional_cast_cost: vec![AdditionalCastCost::Discard { count: 1, filter: None }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            effect: Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dark Deal — each player discards their hand and draws one fewer.
pub fn dark_deal() -> CardDefinition {
    CardDefinition {
        name: "Dark Deal",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardHandDrawThatManyLess { who: Selector::Player(PlayerRef::EachPlayer), less: 1 },
        ..Default::default()
    }
}

/// Dream Stalker — entering returns a permanent you control to its owner's
/// hand.
pub fn dream_stalker() -> CardDefinition {
    CardDefinition {
        name: "Dream Stalker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Illusion]),
        power: 1,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::ReturnOneYouControl { filter: R::Permanent, keep_best: false })],
        ..Default::default()
    }
}

/// Agent of Treachery — entering steals target permanent; your end step
/// draws three while you control three or more permanents you don't own.
pub fn agent_of_treachery() -> CardDefinition {
    let three_stolen = || Predicate::SelectorCountAtLeast {
        sel: Selector::EachPermanent(R::ControlledByYou.and(R::Not(Box::new(R::OwnedByYou)))),
        n: Value::Const(3),
    };
    CardDefinition {
        name: "Agent of Treachery",
        cost: cost(&[generic(5), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Rogue]),
        power: 2,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::GainControl { what: target_filtered(R::Permanent), to: None, duration: Duration::Permanent }),
            TriggeredAbility {
                // CR 603.4 — checked as it triggers and again as it resolves.
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(three_stolen()),
                effect: Effect::If {
                    cond: three_stolen(),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Worldfire — exile every permanent, hand and graveyard; each life total
/// becomes 1.
pub fn worldfire() -> CardDefinition {
    let zone = |zone| Selector::CardsInZone { who: PlayerRef::EachPlayer, zone, filter: R::Any };
    CardDefinition {
        name: "Worldfire",
        cost: cost(&[generic(6), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile { what: Selector::EachPermanent(R::Permanent) },
            Effect::Move { what: zone(crate::card::Zone::Hand), to: ZoneDest::Exile },
            Effect::Move { what: zone(crate::card::Zone::Graveyard), to: ZoneDest::Exile },
            Effect::SetLifeTotal { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Ganax, Astral Hunter — flying; it or another Dragon of yours entering
/// makes a Treasure. Choose a Background.
pub fn ganax_astral_hunter() -> CardDefinition {
    CardDefinition {
        name: "Ganax, Astral Hunter",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Dragon]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::ChooseABackground],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                },
            ),
            effect: mint(crabomination_base::tokens::treasure_token(), Value::ONE),
        }],
        ..Default::default()
    }
}

/// Afterlife Insurance — your creatures gain afterlife 1 until end of turn;
/// draw a card.
pub fn afterlife_insurance() -> CardDefinition {
    CardDefinition {
        name: "Afterlife Insurance",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantTriggeredAbility {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                trigger: Box::new(crate::effect::shortcut::afterlife(1)),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Dockside Chef — {1}{B}, sacrifice an artifact or creature: draw a card.
pub fn dockside_chef() -> CardDefinition {
    CardDefinition {
        name: "Dockside Chef",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Citizen]),
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Artifact.or(R::Creature), 1)),
            sac_other_may_be_source: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wizard's Staff — equipped creature has prowess and its triggered
/// abilities trigger an additional time (CR 603.2d). Equip Wizard {1},
/// equip {3}.
pub fn wizards_staff() -> CardDefinition {
    CardDefinition {
        name: "Wizard's Staff",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equip_filtered_cost: Some((R::HasCreatureType(CreatureType::Wizard), cost(&[generic(1)]))),
        equipped_bonus: Some(crate::card::EquipBonus { keywords: vec![Keyword::Prowess], ..Default::default() }),
        static_abilities: vec![crate::card::StaticAbility {
            description: "If a triggered ability of equipped creature triggers, it triggers an additional time.",
            effect: crate::card::StaticEffect::DoubleControllerTriggersMatching { filter: R::IsHostOfSource },
        }],
        ..Default::default()
    }
}

/// Sorcerer Class — loot two on entering; level 2: your creatures tap for
/// {U} or {R} to spend on instants, sorceries and Class levels; level 3:
/// each instant or sorcery you cast deals damage to each opponent equal to
/// the instants and sorceries you've cast this turn.
pub fn sorcerer_class() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::ManaPayload;
    let level_up = |mana, from| ActivatedAbility {
        mana_cost: mana,
        sorcery_speed: true,
        condition: Some(Predicate::SourceClassLevelIs(from)),
        effect: Effect::AdvanceClassLevel,
        ..Default::default()
    };
    let tap_for_mana = ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Restricted(
                Box::new(ManaPayload::OfColors(vec![Color::Blue, Color::Red], Value::ONE)),
                crate::mana::SpendRestriction::InstantSorceryOrClassLevel,
            ),
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Sorcerer Class",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Class],
            ..Default::default()
        },
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                    Predicate::SourceClassLevelAtLeast(3),
                    crate::effect::shortcut::cast_is_instant_or_sorcery(),
                ])),
                effect: Effect::DealDamageFrom {
                    source: Selector::TriggerSource,
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::InstantsOrSorceriesCastThisTurn(PlayerRef::You),
                },
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have \"{T}: Add {U} or {R}. Spend this mana only to cast an \
                          instant or sorcery spell or to gain a Class level.\"",
            effect: StaticEffect::WhileClassLevelAtLeast {
                n: 2,
                inner: Box::new(StaticEffect::GrantActivatedAbility {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    ability: tap_for_mana,
                    condition: None,
                }),
            },
        }],
        activated_abilities: vec![
            level_up(cost(&[u(), r()]), 1),
            level_up(cost(&[generic(3), u(), r()]), 2),
        ],
        ..Default::default()
    }
}

/// Echo of Eons — each player shuffles hand and graveyard into their
/// library and draws seven; flashback {2}{U}.
pub fn echo_of_eons() -> CardDefinition {
    CardDefinition {
        name: "Echo of Eons",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), u()]))],
        effect: Effect::Seq(vec![
            Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
            Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
        ]),
        ..Default::default()
    }
}

/// Nine-Lives Familiar — enters with eight revival counters if cast; dying
/// with one, it returns at the next end step with one fewer.
pub fn nine_lives_familiar() -> CardDefinition {
    use crate::card::CounterType;
    let revival = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Revival };
    CardDefinition {
        name: "Nine-Lives Familiar",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Cat]),
        power: 1,
        toughness: 1,
        enters_with_counters: Some((
            CounterType::Revival,
            Value::IfPred {
                pred: Box::new(Predicate::SourceWasCast),
                then: Box::new(Value::Const(8)),
                else_: Box::new(Value::ZERO),
            },
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                .with_filter(Predicate::ValueAtLeast(revival(), Value::ONE)),
            // CR 603.7c — the count is fixed as the delayed trigger is made.
            effect: Effect::WithX {
                x: Value::Diff(Box::new(revival()), Box::new(Value::ONE)),
                body: Box::new(Effect::AtNextEndStep {
                    body: Box::new(Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::This, filter: R::InGraveyard },
                        then: Box::new(Effect::Seq(vec![
                            Effect::Move {
                                what: Selector::This,
                                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                            },
                            Effect::AddCounter {
                                what: Selector::This,
                                kind: CounterType::Revival,
                                amount: Value::XFromCost,
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    }),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Protean Hydra — enters with X +1/+1 counters; damage to it is prevented
/// and removes that many; each counter removed comes back as two at the next
/// end step.
pub fn protean_hydra() -> CardDefinition {
    use crate::card::{CounterType, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Protean Hydra",
        cost: cost(&[crate::mana::x(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Hydra]),
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to this creature, prevent that damage and remove that many +1/+1 counters from it.",
            effect: StaticEffect::PreventDamageByRemovingCounters {
                kind: CounterType::PlusOnePlusOne,
                single: false,
                even_without: true,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterRemoved(CounterType::PlusOnePlusOne), EventScope::SelfSource),
            // One trigger per counter (CR 603.2c): the event carries the
            // count, and each counter is two back.
            effect: Effect::WithX {
                x: Value::Times(Box::new(Value::TriggerEventAmount), Box::new(Value::Const(2))),
                body: Box::new(Effect::AtNextEndStep {
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::XFromCost,
                    }),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Valakut Exploration — landfall exiles your top card, playable while it
/// stays exiled; your end step bins what's left and deals that much to each
/// opponent.
pub fn valakut_exploration() -> CardDefinition {
    use crate::effect::ZoneRef;
    let exiled = || Selector::EachMatching { zone: ZoneRef::Exile, filter: R::ExiledWithSource };
    CardDefinition {
        name: "Valakut Exploration",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land },
                ),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE },
                        to: ZoneDest::ExileWithSourceStamp,
                    },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: crate::card::MayPlayDuration::WhileExiled,
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(Value::CardsExiledWithSourceCount, Value::ONE)),
                effect: Effect::WithX {
                    x: Value::CardsExiledWithSourceCount,
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move { what: exiled(), to: ZoneDest::Graveyard },
                        Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::XFromCost },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// The Queen of Dale — each opponent's first noncreature spell each turn
/// makes you recruit (draw, discard; a nonland discard makes a 1/1 Human
/// Soldier).
pub fn the_queen_of_dale() -> CardDefinition {
    let soldier = TokenDefinition {
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Soldier]),
        ..token_1_1("Human Soldier", Color::White, CreatureType::Human)
    };
    CardDefinition {
        name: "The Queen of Dale",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Noble]),
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(R::Noncreature),
                Predicate::ValueEquals(Value::NoncreatureSpellsCastThisTurn(PlayerRef::Triggerer), Value::ONE),
            ])),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: R::Nonland }),
                    then: Box::new(mint(soldier, Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Liberator, Urza's Battlethopter — flash, flying; colorless and artifact
/// spells have flash for you; a spell you cast with more mana spent than its
/// power grows it.
pub fn liberator_urzas_battlethopter() -> CardDefinition {
    use crate::card::{CounterType, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Liberator, Urza's Battlethopter",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Thopter]),
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You may cast colorless spells and artifact spells as though they had flash.",
            effect: StaticEffect::ControllerSpellsHaveFlash { filter: R::Colorless.or(R::Artifact) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::ValueAtLeast(
                Value::CastSpellManaSpent,
                Value::Sum(vec![Value::PowerOf(Box::new(Selector::This)), Value::ONE]),
            )),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Ratadrabik of Urborg — vigilance, ward {2}; other Zombies have vigilance;
/// another legendary creature of yours dying leaves a nonlegendary 2/2 black
/// Zombie copy.
pub fn ratadrabik_of_urborg() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let copy = Effect::CreateTokenCopyOf {
        who: PlayerRef::You,
        count: Value::ONE,
        source: Selector::TriggerSource,
        extra_creature_types: vec![CreatureType::Zombie],
        extra_card_types: vec![],
        override_pt: Some((2, 2)),
        override_colors: None,
        enters_tapped: false,
        non_legendary: true,
        legendary: false,
        extra_keywords: vec![],
        no_mana_cost: false,
        enters_with_counters: None,
        remove_keywords: vec![],
    };
    CardDefinition {
        name: "Ratadrabik of Urborg",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Zombie, CreatureType::Wizard]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance, Keyword::Ward(crate::card::WardCost::Mana(cost(&[generic(2)])))],
        static_abilities: vec![StaticAbility {
            description: "Other Zombies you control have vigilance.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Zombie)
                    .and(R::Creature)
                    .and(R::ControlledByYou)
                    .and(R::OtherThanSource),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Vigilance],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasSupertype(Supertype::Legendary),
                },
            ),
            effect: Effect::Seq(vec![
                copy,
                Effect::AmendCopiableValues {
                    what: Selector::LastCreatedToken,
                    name: None,
                    set_creature_types: None,
                    add_creature_types: vec![],
                    legendary: false,
                    add_colors: vec![Color::Black],
                    set_card_types: None,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// The Cabbage Merchant — an opponent's noncreature spell makes you a Food;
/// combat damage to you costs a Food; tap two untapped Foods for one mana of
/// any color.
pub fn the_cabbage_merchant() -> CardDefinition {
    use crate::effect::ManaPayload;
    let food = || R::HasArtifactSubtype(crate::card::ArtifactSubtype::Food).and(R::ControlledByYou);
    CardDefinition {
        name: "The Cabbage Merchant",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Citizen]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
                effect: mint(crabomination_base::tokens::food_token(), Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::SelfSource),
                effect: Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: food().and(R::IsToken) },
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((food().and(R::Untapped), 2)),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Koth, the Geomancer — reach; landfall deals 1 to each opponent, and a
/// Mountain adds {R}.
pub fn koth_the_geomancer() -> CardDefinition {
    CardDefinition {
        name: "Koth, the Geomancer",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Warrior]),
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![crate::effect::shortcut::landfall(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            Effect::If {
                cond: Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasLandType(crate::card::LandType::Mountain),
                },
                then: Box::new(Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colors(vec![Color::Red]),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..Default::default()
    }
}

/// Gaea's Gift — a +1/+1 counter, then reach, trample, hexproof and
/// indestructible until end of turn, on a creature you control.
pub fn gaeas_gift() -> CardDefinition {
    CardDefinition {
        name: "Gaea's Gift",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::GrantKeywords {
                what: Selector::Target(0),
                keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Hexproof, Keyword::Indestructible],
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Conflux — tutor one card of each color.
pub fn conflux() -> CardDefinition {
    CardDefinition {
        name: "Conflux",
        cost: cost(&[generic(3), w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(
            [Color::White, Color::Blue, Color::Black, Color::Red, Color::Green]
                .into_iter()
                .map(|c| Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasColor(c),
                    to: ZoneDest::Hand(PlayerRef::You),
                })
                .collect(),
        ),
        ..Default::default()
    }
}

/// Underrealm Lich — your draws become "look at the top three, keep one, the
/// rest into your graveyard"; pay 4 life for indestructible (and tap it).
pub fn underrealm_lich() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Underrealm Lich",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Zombie, CreatureType::Elf, CreatureType::Shaman]),
        power: 4,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, instead look at the top three cards of your library, then \
                          put one into your hand and the rest into your graveyard.",
            effect: StaticEffect::ReplaceDrawWithLookN { count: 3, rest_to_graveyard: true },
        }],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 4,
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eruth, Tormented Prophet — your draws become "exile the top two, play
/// them this turn".
pub fn eruth_tormented_prophet() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Eruth, Tormented Prophet",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, exile the top two cards of your library instead. You may \
                          play those cards this turn.",
            effect: StaticEffect::ReplaceDrawWithImpulse { count: 2 },
        }],
        ..Default::default()
    }
}

/// Liesa, Forgotten Archangel — flying, lifelink; another nontoken creature
/// of yours dying returns to hand at the next end step; an opponent's dying
/// creature is exiled instead.
pub fn liesa_forgotten_archangel() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Liesa, Forgotten Archangel",
        cost: cost(&[generic(2), w(), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Angel]),
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If a creature an opponent controls would die, exile it instead.",
            effect: StaticEffect::ExileDyingOpponentCreatures { when_you_do: None, tokens_too: true },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::NotToken }),
            effect: Effect::ReturnToOwnersHandAtNextEndStep { what: Selector::TriggerSource },
        }],
        ..Default::default()
    }
}

/// Virtue of Strength // Garenbrig Growth — basic lands you tap make three
/// times the mana; the adventure returns a creature or land card to hand.
pub fn virtue_of_strength() -> CardDefinition {
    use crate::card::{Adventure, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Virtue of Strength",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you tap a basic land for mana, it produces three times as much of that mana instead.",
            effect: StaticEffect::BasicLandManaTripled,
        }],
        adventure: Some(Box::new(Adventure {
            name: "Garenbrig Growth",
            cost: cost(&[g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Move {
                what: target_filtered(R::Creature.or(R::Land).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        })),
        ..Default::default()
    }
}

/// Deathgreeter — another creature dying may gain you 1 life.
pub fn deathgreeter() -> CardDefinition {
    CardDefinition {
        name: "Deathgreeter",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Shaman]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
            effect: Effect::MayDo {
                description: "Gain 1 life?".into(),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

/// Virulent Emissary — deathtouch; another creature of yours entering gains
/// you 1 life.
pub fn virulent_emissary() -> CardDefinition {
    CardDefinition {
        name: "Virulent Emissary",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Assassin]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// "Discard a card: this creature gets +1/+1 until end of turn."
fn discard_pump() -> ActivatedAbility {
    ActivatedAbility {
        discard_cost: Some((R::Any, 1)),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Noose Constrictor — reach; discard a card for +1/+1.
pub fn noose_constrictor() -> CardDefinition {
    CardDefinition {
        name: "Noose Constrictor",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Snake]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![discard_pump()],
        ..Default::default()
    }
}

/// Oblivion Crown — flash Aura: enchanted creature discards cards for +1/+1.
pub fn oblivion_crown() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Oblivion Crown",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has \"Discard a card: This creature gets +1/+1 until end of turn.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: discard_pump(),
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Ancestral Statue — entering returns a nonland permanent you control to
/// its owner's hand.
pub fn ancestral_statue() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Statue",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Golem]),
        power: 3,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::ReturnOneYouControl { filter: R::Nonland, keep_best: false })],
        ..Default::default()
    }
}

/// Rattleclaw Mystic — taps for {G}, {U} or {R}; morph {2}; turned face up,
/// adds {G}{U}{R}.
pub fn rattleclaw_mystic() -> CardDefinition {
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Rattleclaw Mystic",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Shaman]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Morph(cost(&[generic(2)]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(vec![Color::Green, Color::Blue, Color::Red], Value::ONE),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green, Color::Blue, Color::Red]),
            },
        }],
        ..Default::default()
    }
}

/// Basal Sliver — every Sliver has "Sacrifice this permanent: Add {B}{B}."
pub fn basal_sliver() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Basal Sliver",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Sliver]),
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"Sacrifice this permanent: Add {B}{B}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Sliver)),
                ability: ActivatedAbility {
                    sac_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black, Color::Black]) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..Default::default()
    }
}

/// Training Grounds — your creatures' activated abilities cost {2} less, to
/// no less than one mana.
pub fn training_grounds() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Training Grounds",
        cost: cost(&[u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of creatures you control cost {2} less to activate. This effect \
                          can't reduce the mana in that cost to less than one mana.",
            effect: StaticEffect::YourCreatureActivatedAbilitiesCostLess { amount: 2 },
        }],
        ..Default::default()
    }
}

/// Circle of Flame — a creature without flying attacking you or your
/// planeswalker takes 1 damage.
pub fn circle_of_flame() -> CardDefinition {
    CardDefinition {
        name: "Circle of Flame",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Not(Box::new(R::HasKeyword(Keyword::Flying))),
                },
            ),
            effect: Effect::DealDamage { to: Selector::TriggerSource, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Smash to Dust — destroy an artifact or a defender, or 1 damage to each
/// creature your opponents control.
pub fn smash_to_dust() -> CardDefinition {
    CardDefinition {
        name: "Smash to Dust",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Artifact) },
            Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Defender))) },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Starnheim Courser — flying; artifact and enchantment spells cost {1}
/// less.
pub fn starnheim_courser() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Starnheim Courser",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Pegasus]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Artifact and enchantment spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction { filter: R::Artifact.or(R::Enchantment), amount: 1 },
        }],
        ..Default::default()
    }
}

/// Diabolic Revelation — {X}{3}{B}{B} Sorcery. Search your library for up to
/// X cards, put them into your hand, then shuffle.
pub fn diabolic_revelation() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Diabolic Revelation",
        cost: cost(&[x(), generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Any,
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Gelatinous Genesis — {X}{X}{G} Sorcery. Create X X/X green Ooze creature
/// tokens.
pub fn gelatinous_genesis() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Gelatinous Genesis",
        cost: cost(&[x(), x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::XFromCost,
            definition: Arc::new(TokenDefinition {
                name: "Ooze".into(),
                power: 0,
                toughness: 0,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: creature_types(vec![CreatureType::Ooze]),
                dynamic_pt: Some((Value::XFromCost, Value::XFromCost)),
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}

/// Children of Korlis — {W} 1/1. Sacrifice it: you gain life equal to the
/// life you've lost this turn.
pub fn children_of_korlis() -> CardDefinition {
    CardDefinition {
        name: "Children of Korlis",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Cleric]),
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::LifeLostThisTurn(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rodolf Duskbringer — {5}{B} 4/4 flying, deathtouch, lifelink. Gaining
/// life makes it indestructible this turn; at your end step you may pay
/// {1}{W/B} to return a creature card with mana value up to the life you
/// gained this turn from your graveyard to the battlefield.
pub fn rodolf_duskbringer() -> CardDefinition {
    CardDefinition {
        name: "Rodolf Duskbringer",
        cost: cost(&[generic(5), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Vampire, CreatureType::Angel]),
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Indestructible,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::MayPay {
                    description: "Pay {1}{W/B} to return a creature card?".into(),
                    mana_cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
                    body: Box::new(Effect::Reflexive {
                        body: Box::new(Effect::Move {
                            what: target_filtered(
                                R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMostLifeGainedThisTurn),
                            ),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    }),
                    else_: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Tor Wauki the Younger — {3}{B}{R} 3/3 reach, lifelink. Your other
/// sources' noncombat damage gets +1; casting an instant or sorcery shoots
/// any target for 2.
pub fn tor_wauki_the_younger() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Tor Wauki the Younger",
        cost: cost(&[generic(3), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Archer]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "If another source you control would deal noncombat damage to a permanent or player, it deals that much damage plus 1 instead.",
            effect: StaticEffect::NoncombatDamageFromOtherSourcesBonus { amount: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(crate::effect::shortcut::cast_is_instant_or_sorcery()),
            effect: Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Linvala, Keeper of Silence — {2}{W}{W} 3/4 flying. Activated abilities of
/// creatures your opponents control can't be activated.
pub fn linvala_keeper_of_silence() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Linvala, Keeper of Silence",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Angel]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of creatures your opponents control can't be activated.",
            effect: StaticEffect::OpponentsCreatureAbilitiesLocked,
        }],
        ..Default::default()
    }
}

/// Jin-Gitaxias, Progress Tyrant — {5}{U}{U} 5/5. Once each turn, copy your
/// artifact, instant, or sorcery spell (a permanent copy becomes a token,
/// CR 707.10f); once each turn, counter an opponent's.
pub fn jin_gitaxias_progress_tyrant() -> CardDefinition {
    let ais = || {
        Predicate::CastSpellMatches(
            R::HasCardType(CardType::Artifact)
                .or(R::HasCardType(CardType::Instant))
                .or(R::HasCardType(CardType::Sorcery)),
        )
    };
    CardDefinition {
        name: "Jin-Gitaxias, Progress Tyrant",
        cost: cost(&[generic(5), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Phyrexian, CreatureType::Praetor]),
        power: 5,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(ais()).once_per_turn(),
                effect: Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(ais()).once_per_turn(),
                effect: Effect::CounterSpell { what: Selector::TriggerSource },
            },
        ],
        ..Default::default()
    }
}

/// Cerulean Wisps — {U} Instant. Target creature becomes blue until end of
/// turn; untap it. Draw a card.
pub fn cerulean_wisps() -> CardDefinition {
    CardDefinition {
        name: "Cerulean Wisps",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Blue],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Phylath, World Sculptor — {4}{R}{G} 5/5. ETB: a 0/1 Plant per basic land
/// you control; landfall: four +1/+1 counters on target Plant you control.
pub fn phylath_world_sculptor() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::landfall;
    CardDefinition {
        name: "Phylath, World Sculptor",
        cost: cost(&[generic(4), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elemental]),
        power: 5,
        toughness: 5,
        triggered_abilities: vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Land.and(R::HasSupertype(Supertype::Basic)).and(R::ControlledByYou),
                ))),
                definition: Arc::new(TokenDefinition {
                    name: "Plant".into(),
                    power: 0,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Green],
                    subtypes: creature_types(vec![CreatureType::Plant]),
                    ..Default::default()
                }),
            }),
            landfall(Effect::AddCounter {
                what: target_filtered(R::HasCreatureType(CreatureType::Plant).and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(4),
            }),
        ],
        ..Default::default()
    }
}

/// Nissa, Vital Force — {3}{G}{G}, loyalty 5. +1: untap target land you
/// control; until your next turn it's a 5/5 Elemental with haste. −3: return
/// a permanent card from your graveyard to hand. −6: emblem "whenever a land
/// you control enters, you may draw a card."
pub fn nissa_vital_force() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    CardDefinition {
        name: "Nissa, Vital Force",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Nissa], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Untap { what: target_filtered(R::Land.and(R::ControlledByYou)), up_to: None },
                    Effect::BecomeCreature {
                        what: Selector::Target(0),
                        power: Value::Const(5),
                        toughness: Value::Const(5),
                        creature_types: vec![CreatureType::Elemental],
                        keywords: vec![Keyword::Haste],
                        duration: Duration::UntilNextTurn,
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Move {
                    what: target_filtered(R::Permanent.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Nissa, Vital Force".into(),
                    triggered: vec![crate::effect::shortcut::landfall(Effect::MayDo {
                        description: "Draw a card?".into(),
                        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    })],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Nissa of Shadowed Boughs — {2}{B}{G}, loyalty 4. Landfall adds a loyalty
/// counter. +1: untap target land you control; you may make it a 3/3 haste,
/// menace Elemental until end of turn. −5: you may put a creature card with
/// mana value up to your land count from your hand or graveyard onto the
/// battlefield with two +1/+1 counters (placed as it lands, not an
/// enters-with replacement).
pub fn nissa_of_shadowed_boughs() -> CardDefinition {
    use crate::card::{CounterType, LoyaltyAbility, PlaneswalkerSubtype, Zone};
    use crate::effect::shortcut::{choose_one_then, chosen_one, landfall};
    let creature_cards = || {
        let card = |zone| Selector::CardsInZone {
            who: PlayerRef::You,
            zone,
            filter: R::Creature.and(R::ManaValueAtMostYourCount(Box::new(R::Land))),
        };
        Selector::Both(Box::new(card(Zone::Hand)), Box::new(card(Zone::Graveyard)))
    };
    CardDefinition {
        name: "Nissa of Shadowed Boughs",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Nissa], ..Default::default() },
        base_loyalty: 4,
        triggered_abilities: vec![landfall(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::Loyalty,
            amount: Value::ONE,
        })],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Untap { what: target_filtered(R::Land.and(R::ControlledByYou)), up_to: None },
                    Effect::MayDo {
                        description: "Make it a 3/3 Elemental with haste and menace?".into(),
                        body: Box::new(Effect::BecomeCreature {
                            what: Selector::Target(0),
                            power: Value::Const(3),
                            toughness: Value::Const(3),
                            creature_types: vec![CreatureType::Elemental],
                            keywords: vec![Keyword::Haste, Keyword::Menace],
                            duration: Duration::EndOfTurn,
                        }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -5,
                effect: Effect::If {
                    cond: Predicate::SelectorExists(creature_cards()),
                    then: Box::new(Effect::MayDo {
                        description: "Put a creature card from your hand or graveyard onto the battlefield?".into(),
                        body: Box::new(choose_one_then(
                            creature_cards(),
                            PlayerRef::You,
                            Effect::Seq(vec![
                                Effect::Move {
                                    what: chosen_one(),
                                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                                },
                                Effect::AddCounter {
                                    what: Selector::LastMoved,
                                    kind: CounterType::PlusOnePlusOne,
                                    amount: Value::Const(2),
                                },
                            ]),
                        )),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Mikaeus, the Unhallowed — {3}{B}{B}{B} 5/5 intimidate. A Human that
/// damages you is destroyed (opponents' Humans: the `OpponentSourceDamagedYou`
/// scope; a Human of your own hitting you is not covered). Other non-Human
/// creatures you control get +1/+1 and have undying.
pub fn mikaeus_the_unhallowed() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Mikaeus, the Unhallowed",
        cost: cost(&[generic(3), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Zombie, CreatureType::Cleric]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Intimidate],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::OpponentSourceDamagedYou).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Human),
                },
            ),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        static_abilities: vec![StaticAbility {
            description: "Other non-Human creatures you control get +1/+1 and have undying.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature
                    .and(R::OtherThanSource)
                    .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human)))),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Undying],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Lich's Mastery — {3}{B}{B}{B} legendary enchantment, hexproof. You can't
/// lose the game; gaining life draws that many; each 1 life lost exiles a
/// permanent you control or a card from your hand or graveyard (graveyard
/// cards offered first); leaving the battlefield loses you the game.
pub fn lichs_mastery() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Zone};
    use crate::effect::shortcut::{choose_some_then, chosen_one};
    let card = |zone| Selector::CardsInZone { who: PlayerRef::You, zone, filter: R::Any };
    let fodder = Selector::Both(
        Box::new(card(Zone::Graveyard)),
        Box::new(Selector::Both(
            Box::new(card(Zone::Hand)),
            Box::new(Selector::EachPermanent(R::Permanent.and(R::ControlledByYou))),
        )),
    );
    CardDefinition {
        name: "Lich's Mastery",
        cost: cost(&[generic(3), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![StaticAbility {
            description: "You can't lose the game.",
            effect: StaticEffect::ControllerCantLoseGame,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeLost, EventScope::YourControl),
                effect: choose_some_then(
                    fodder,
                    PlayerRef::You,
                    Value::TriggerEventAmount,
                    false,
                    Effect::Move { what: chosen_one(), to: ZoneDest::Exile },
                ),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: Effect::LoseGame { who: PlayerRef::You },
            },
        ],
        ..Default::default()
    }
}

/// Talion, the Kindly Lord — {2}{U}{B} 3/4 flying. As it enters, choose a
/// number from 1 to 10; whenever an opponent casts a spell with that mana
/// value, power, or toughness, they lose 2 life and you draw a card.
pub fn talion_the_kindly_lord() -> CardDefinition {
    let spell_stat_is_chosen = |stat: Value| Predicate::ValueEquals(stat, Value::ChosenNumberOfSource);
    let spell = || Box::new(Selector::TriggerSource);
    CardDefinition {
        name: "Talion, the Kindly Lord",
        cost: cost(&[generic(2), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Faerie, CreatureType::Noble]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::ChooseNumberForSource { max: 10, pays_life: false, min: 1 }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(Predicate::Any(vec![
                spell_stat_is_chosen(Value::ManaValueOf(spell())),
                spell_stat_is_chosen(Value::PowerOf(spell())),
                spell_stat_is_chosen(Value::ToughnessOf(spell())),
            ])),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..Default::default()
    }
}

/// Ruthless Technomancer — {3}{B} 2/4. ETB: you may sacrifice another
/// creature for Treasures equal to its power. {2}{B}, sacrifice X artifacts
/// (X ≥ 1): return a creature card with power X or less from your graveyard
/// to the battlefield.
pub fn ruthless_technomancer() -> CardDefinition {
    CardDefinition {
        name: "Ruthless Technomancer",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice another creature for Treasures equal to its power?".into(),
            filter: R::Creature.and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(mint(crabomination_base::tokens::treasure_token(), Value::SacrificedPower)),
            else_: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_other_filter: Some((R::Artifact, 1)),
            sac_other_x: true,
            x_nonzero: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::PowerAtMostXFromCost)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gold-Forged Thopteryx — {W}{U} 1/3 flying, lifelink artifact creature.
/// Each legendary permanent you control has ward {2}.
pub fn gold_forged_thopteryx() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, WardCost};
    CardDefinition {
        name: "Gold-Forged Thopteryx",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Dinosaur, CreatureType::Thopter]),
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Each legendary permanent you control has ward {2}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::HasSupertype(Supertype::Legendary).and(R::ControlledByYou)),
                keyword: Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))),
            },
        }],
        ..Default::default()
    }
}

/// Will, Scion of Peace — {1}{W}{U} 2/4 vigilance. {T} (sorcery speed):
/// white and/or blue spells you cast this turn cost {X} less, X = the life
/// you gained this turn, fixed as it resolves (the Rowan, Scion of War
/// ruling).
pub fn will_scion_of_peace() -> CardDefinition {
    CardDefinition {
        name: "Will, Scion of Peace",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::SpellsCostLessThisTurnByValue {
                filter: R::HasColor(Color::White).or(R::HasColor(Color::Blue)),
                amount: Value::LifeGainedThisTurn(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Emiel the Blessed — {2}{W}{W} 4/4 Unicorn. {3}: blink another target
/// creature you control. Whenever another creature you control enters, you
/// may pay {G/W} for a +1/+1 counter on it (two if it's a Unicorn).
pub fn emiel_the_blessed() -> CardDefinition {
    use crate::card::CounterType;
    let counters = |n: i32| Effect::AddCounter {
        what: Selector::TriggerSource,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Emiel the Blessed",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Unicorn]),
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::ExileAndReturnToOwner {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::OtherThanSource) },
            ),
            effect: Effect::MayPay {
                description: "Pay {G/W} for a +1/+1 counter?".into(),
                mana_cost: cost(&[hybrid(Color::Green, Color::White)]),
                body: Box::new(Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Unicorn),
                    },
                    then: Box::new(counters(2)),
                    else_: Box::new(counters(1)),
                }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Misthollow Griffin — {2}{U}{U} 3/3 flying; may be cast from exile.
pub fn misthollow_griffin() -> CardDefinition {
    CardDefinition {
        name: "Misthollow Griffin",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Griffin]),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::ExileCast],
        ..Default::default()
    }
}

/// Mnemonic Deluge — {6}{U}{U}{U} Sorcery. Exile target instant or sorcery
/// card from a graveyard; cast up to three copies of it free (CR 707.12).
/// Exile Mnemonic Deluge.
pub fn mnemonic_deluge() -> CardDefinition {
    use crate::card::Zone;
    let free_copy = || Effect::CastWithoutPayingImmediate {
        what: Selector::LastMoved,
        source_zone: Zone::Exile,
        exile_after: false,
        copy: true,
        reduce_generic: 0,
        pay_own_cost: false,
    };
    CardDefinition {
        name: "Mnemonic Deluge",
        cost: cost(&[generic(6), u(), u(), u()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).and(R::InGraveyard),
                ),
                to: ZoneDest::Exile,
            },
            free_copy(),
            free_copy(),
            free_copy(),
        ]),
        ..Default::default()
    }
}

/// Cultivator Colossus — {4}{G}{G}{G} */* trample, P/T = lands you control.
/// ETB: you may put a land card from your hand onto the battlefield tapped;
/// if you do, draw a card and repeat (unrolled twenty deep — a declined or
/// empty pick ends the chain).
pub fn cultivator_colossus() -> CardDefinition {
    use crate::card::DynamicPt;
    fn process(depth: u32) -> Effect {
        let draw = Effect::Draw { who: Selector::You, amount: Value::ONE };
        Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Land,
            count: Value::ONE,
            tapped: true,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: Some(Box::new(if depth == 0 { draw } else { Effect::Seq(vec![draw, process(depth - 1)]) })),
        }
    }
    CardDefinition {
        name: "Cultivator Colossus",
        cost: cost(&[generic(4), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Plant, CreatureType::Beast]),
        keywords: vec![Keyword::Trample],
        dynamic_pt: Some(DynamicPt::LandsControlled { base: 0 }),
        triggered_abilities: vec![etb(process(19))],
        ..Default::default()
    }
}

/// Iridescent Hornbeetle — {4}{G} 3/4. At your end step, a 1/1 Insect for
/// each +1/+1 counter you put on creatures you control this turn.
pub fn iridescent_hornbeetle() -> CardDefinition {
    CardDefinition {
        name: "Iridescent Hornbeetle",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Insect]),
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: mint(
                token_1_1("Insect", Color::Green, CreatureType::Insect),
                Value::PlusOneCountersPutOnYourCreaturesThisTurn(PlayerRef::You),
            ),
        }],
        ..Default::default()
    }
}

/// South Wind Avatar — {3}{B} 3/4 deathtouch. Another creature of yours
/// dying gains you its toughness; each life gain drains each opponent 1.
pub fn south_wind_avatar() -> CardDefinition {
    CardDefinition {
        name: "South Wind Avatar",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Snake, CreatureType::Spirit, CreatureType::Avatar]),
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

/// Mirkwood Spider — {G} 1/1 deathtouch Spider. Attacking, it gives target
/// legendary creature you control deathtouch until end of turn.
pub fn mirkwood_spider() -> CardDefinition {
    CardDefinition {
        name: "Mirkwood Spider",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Spider]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou)),
            keyword: Keyword::Deathtouch,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Oakhame Adversary — {3}{G} 2/3 deathtouch; {2} less if an opponent
/// controls a green permanent; combat damage to a player draws a card.
pub fn oakhame_adversary() -> CardDefinition {
    CardDefinition {
        name: "Oakhame Adversary",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Warrior]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        self_cost_reduction_if: Some((
            Predicate::SelectorExists(Selector::EachPermanent(R::HasColor(Color::Green).and(R::ControlledByOpponent))),
            2,
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Tajuru Blightblade — {G} 1/1 deathtouch Elf Rogue.
pub fn tajuru_blightblade() -> CardDefinition {
    CardDefinition {
        name: "Tajuru Blightblade",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Rogue]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Dreadmaw's Ire — {R} Instant. Target attacking creature gets +2/+2 and
/// trample, and "deals combat damage to a player: destroy target artifact
/// that player controls," until end of turn.
pub fn dreadmaws_ire() -> CardDefinition {
    let target = || Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::IsAttacking) };
    CardDefinition {
        name: "Dreadmaw's Ire",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT { what: target(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: target(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantTriggeredAbility {
                what: target(),
                trigger: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                    effect: Effect::Destroy { what: target_filtered(R::Artifact.and(R::ControlledByDefendingPlayer)) },
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Sheltering Light — {W} Instant. Target creature gains indestructible
/// until end of turn. Scry 1.
pub fn sheltering_light() -> CardDefinition {
    CardDefinition {
        name: "Sheltering Light",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Angelfire Ignition — {1}{R}{W} Sorcery. Two +1/+1 counters on target
/// creature; it gains vigilance, trample, lifelink, indestructible, and
/// haste until end of turn. Flashback {2}{R}{W}.
pub fn angelfire_ignition() -> CardDefinition {
    use crate::card::CounterType;
    let target = || Selector::TargetFiltered { slot: 0, filter: R::Creature };
    let mut steps = vec![Effect::AddCounter { what: target(), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2) }];
    for keyword in [Keyword::Vigilance, Keyword::Trample, Keyword::Lifelink, Keyword::Indestructible, Keyword::Haste] {
        steps.push(Effect::GrantKeyword { what: target(), keyword, duration: Duration::EndOfTurn });
    }
    CardDefinition {
        name: "Angelfire Ignition",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r(), w()]))],
        effect: Effect::Seq(steps),
        ..Default::default()
    }
}

/// Khenra Spellspear // Gitaxian Spellstalker — {1}{R} 2/2 trample,
/// prowess; {3}{U/P} (sorcery speed): transform into a 3/3 trample, ward
/// {2} with two instances of prowess (each triggers, CR 702.108b).
pub fn khenra_spellspear() -> CardDefinition {
    use crate::card::WardCost;
    use crate::effect::shortcut::prowess;
    use crate::mana::phyrexian;
    let stalker = CardDefinition {
        name: "Gitaxian Spellstalker",
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Phyrexian, CreatureType::Jackal]),
        color_indicator: vec![Color::Blue, Color::Red],
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample, Keyword::Ward(WardCost::Mana(cost(&[generic(2)]))), Keyword::Prowess],
        // Two printed prowess instances: explicit pumps, each its own trigger.
        triggered_abilities: vec![prowess(), prowess()],
        ..Default::default()
    };
    CardDefinition {
        name: "Khenra Spellspear",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Jackal, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Prowess],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), phyrexian(Color::Blue)]),
            sorcery_speed: true,
            effect: Effect::Transform { what: Selector::This },
            ..Default::default()
        }],
        back_face: Some(Box::new(stalker)),
        ..Default::default()
    }
}

/// Vraska, Soul of Stone — {U}{R}{W} 3/3. Artifact creatures you control
/// have vigilance; each noncreature spell you cast makes a 1/1 Sculpture
/// Treasure artifact creature.
pub fn vraska_soul_of_stone() -> CardDefinition {
    use crate::card::{ArtifactSubtype, StaticAbility, StaticEffect};
    let mut sculpture = crabomination_base::tokens::treasure_token();
    sculpture.name = "Sculpture".into();
    sculpture.power = 1;
    sculpture.toughness = 1;
    sculpture.card_types = vec![CardType::Artifact, CardType::Creature];
    sculpture.subtypes = Subtypes {
        artifact_subtypes: vec![ArtifactSubtype::Treasure],
        creature_types: vec![CreatureType::Sculpture],
        ..Default::default()
    };
    CardDefinition {
        name: "Vraska, Soul of Stone",
        cost: cost(&[u(), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Gorgon, CreatureType::Wizard]),
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Artifact creatures you control have vigilance.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                keyword: Keyword::Vigilance,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCardType(CardType::Creature).negate(),
            }),
            effect: mint(sculpture, Value::ONE),
        }],
        ..Default::default()
    }
}

/// See the Truth — {1}{U} Sorcery. Look at the top three; one to hand, the
/// rest on the bottom — all three to hand if it wasn't cast from your hand.
pub fn see_the_truth() -> CardDefinition {
    CardDefinition {
        name: "See the Truth",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::CastFromHand,
            then: Box::new(Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(3),
                ..Default::default()
            }))),
            // Put into hand, not drawn (no draw triggers).
            else_: Box::new(Effect::Move {
                what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(3) },
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
        ..Default::default()
    }
}

/// Simulacrum Synthesizer — {2}{U} artifact. ETB scry 2; another artifact
/// you control with mana value 3+ entering makes a 0/0 Construct that gets
/// +1/+1 per artifact you control.
pub fn simulacrum_synthesizer() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let construct = TokenDefinition {
        name: "Construct".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Construct]),
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+1 for each artifact you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents { filter: R::Artifact, per_power: 1, per_toughness: 1 },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Simulacrum Synthesizer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.and(R::OtherThanSource).and(R::ManaValueAtLeast(3)),
                    },
                ),
                effect: mint(construct, Value::ONE),
            },
        ],
        ..Default::default()
    }
}

/// Stoneskin — {2}{W} Aura with flash. Enchanted creature gets +0/+10.
pub fn stoneskin() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Stoneskin",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: R::Creature } },
        equipped_bonus: Some(EquipBonus { power: 0, toughness: 10, ..Default::default() }),
        ..Default::default()
    }
}

/// Malcolm, Alluring Scoundrel — {1}{U} 2/1 flash, flying. Its combat
/// damage to a player adds a chorus counter and loots; with four or more
/// chorus counters you may cast the discarded card free.
pub fn malcolm_alluring_scoundrel() -> CardDefinition {
    use crate::card::{CounterType, Zone};
    CardDefinition {
        name: "Malcolm, Alluring Scoundrel",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Siren, CreatureType::Pirate]),
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Chorus, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Chorus },
                        Value::Const(4),
                    ),
                    then: Box::new(Effect::CastWithoutPayingImmediate {
                        what: Selector::DiscardedThisResolution { filter: R::Any },
                        source_zone: Zone::Graveyard,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Likeness Looter — {U}{B} 1/1 flying. {T}: loot. {X} (sorcery speed):
/// becomes a copy of target creature card in your graveyard with mana value
/// X, except it has flying and this ability (CR 707.9a).
pub fn likeness_looter() -> CardDefinition {
    use crate::mana::x;
    CardDefinition {
        name: "Likeness Looter",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Faerie, CreatureType::Shapeshifter]),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[x()]),
                sorcery_speed: true,
                effect: Effect::Seq(vec![
                    Effect::BecomeCopyOf {
                        what: Selector::This,
                        source: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::ManaValueExactlyXFromCost)),
                        extra_creature_types: vec![],
                        keep_own_triggered: false,
                        keep_own_activated: false,
                        keep_name: false,
                    },
                    Effect::AmendCopy {
                        what: Selector::This,
                        keep_activated: vec![1],
                        keep_triggered: vec![],
                        pt: None,
                        keep_name: false,
                        legendary: false,
                        keywords: vec![Keyword::Flying],
                        card_types: vec![],
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Thornbite Staff — {2} Kindred Artifact — Shaman Equipment. Equipped
/// creature has "{2}, {T}: 1 damage to any target" and "whenever a creature
/// dies, untap this creature." A Shaman creature entering under your control
/// may pick it up. Equip {4}.
pub fn thornbite_staff() -> CardDefinition {
    use crate::card::{ArtifactSubtype, EquipBonus};
    CardDefinition {
        name: "Thornbite Staff",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Kindred, CardType::Artifact],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shaman],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::DealDamage { to: target_filtered(R::any_target()), amount: Value::ONE },
                ..Default::default()
            }],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
                effect: Effect::Untap { what: Selector::This, up_to: None },
            }],
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Shaman)),
                },
            ),
            effect: Effect::MayDo {
                description: "Attach Thornbite Staff to the entering Shaman?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..Default::default()
    }
}

/// Life Finds a Way — {2}{G} Enchantment. Whenever a nontoken creature with
/// power 4 or greater enters under your control, populate.
pub fn life_finds_a_way() -> CardDefinition {
    CardDefinition {
        name: "Life Finds a Way",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::PowerAtLeast(4)),
                },
            ),
            effect: Effect::Populate { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// Terisian Mindbreaker — {7} 6/4 artifact Juggernaut. Attacking, the
/// defending player mills half their library, rounded up. Unearth
/// {1}{U}{U}{U}.
pub fn terisian_mindbreaker() -> CardDefinition {
    CardDefinition {
        name: "Terisian Mindbreaker",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Juggernaut]),
        power: 6,
        toughness: 4,
        triggered_abilities: vec![on_attack(Effect::MillHalf {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            rounded_up: true,
        })],
        activated_abilities: vec![crate::effect::shortcut::unearth(cost(&[generic(1), u(), u(), u()]))],
        ..Default::default()
    }
}

/// Vantress Gargoyle — {1}{U} 5/4 flying artifact Gargoyle. Can't attack
/// unless defending player has seven or more cards in their graveyard; can't
/// block unless you have four or more cards in hand. {T}: each player mills
/// a card.
pub fn vantress_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Vantress Gargoyle",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Gargoyle]),
        power: 5,
        toughness: 4,
        keywords: vec![
            Keyword::Flying,
            Keyword::CantAttackUnlessDefenderGraveyardAtLeast(7),
            Keyword::CantBlockUnlessHandSizeAtLeast(4),
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rammas Echor, Ancient Shield — {3}{W} legendary artifact. Your second
/// spell each turn draws a card and makes a 0/3 Wall with defender; at the
/// beginning of combat on your turn, your defenders gain exalted until end
/// of turn.
pub fn rammas_echor_ancient_shield() -> CardDefinition {
    use crate::effect::shortcut::{exalted, flurry};
    let wall = TokenDefinition {
        name: "Wall".into(),
        power: 0,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: creature_types(vec![CreatureType::Wall]),
        keywords: vec![Keyword::Defender],
        ..Default::default()
    };
    CardDefinition {
        name: "Rammas Echor, Ancient Shield",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            flurry(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                mint(wall, Value::ONE),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::GrantTriggeredAbility {
                    what: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Defender)).and(R::ControlledByYou)),
                    trigger: Box::new(exalted()),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Ghalta the Immovable — {8}{W} 0/7 Elder Dinosaur. Costs {X} less, X =
/// the greatest toughness among creatures you control. Your creatures can
/// attack as though they lacked defender; those with toughness greater than
/// power assign combat damage equal to their toughness (CR 510.1c).
pub fn ghalta_the_immovable() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Ghalta the Immovable",
        cost: cost(&[generic(8), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elder, CreatureType::Dinosaur]),
        power: 0,
        toughness: 7,
        self_cost_reduction_per: Some((Value::ToughnessOf(Box::new(Selector::GreatestToughnessYouControl)), 1)),
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control can attack as though they didn't have defender.",
                effect: StaticEffect::YourCreaturesCanAttackAsThoughNoDefender,
            },
            StaticAbility {
                description: "Each creature you control with toughness greater than its power assigns combat damage equal to its toughness rather than its power.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::ToughnessGreaterThanPower).and(R::ControlledByYou)),
                    keyword: Keyword::AssignsCombatDamageByToughness,
                },
            },
        ],
        ..Default::default()
    }
}

/// Echoing Deeps — Land — Cave. You may have it enter tapped as a copy of
/// any land card in a graveyard (the "Cave in addition" rider is not
/// stamped on the copy). {T}: Add {C}.
pub fn echoing_deeps() -> CardDefinition {
    use crate::card::{EntersAsCopy, LandType};
    CardDefinition {
        name: "Echoing Deeps",
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Cave], ..Default::default() },
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Land,
            from_graveyards: true,
            tapped: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Blossoming Tortoise — {2}{G}{G} 3/3 Turtle. Entering or attacking, mill
/// three, then return a land card from your graveyard to the battlefield
/// tapped. Activated abilities of your lands cost {1} less; land creatures
/// you control get +1/+1.
pub fn blossoming_tortoise() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Zone};
    use crate::effect::shortcut::{choose_one_then, chosen_one};
    let dig = || {
        Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            choose_one_then(
                Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Land },
                PlayerRef::You,
                Effect::Move { what: chosen_one(), to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true } },
            ),
        ])
    };
    CardDefinition {
        name: "Blossoming Tortoise",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Turtle]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(dig()), on_attack(dig())],
        static_abilities: vec![
            StaticAbility {
                description: "Activated abilities of lands you control cost {1} less to activate.",
                effect: StaticEffect::MatchingActivatedAbilitiesCostLess { filter: R::Land, amount: 1 },
            },
            StaticAbility {
                description: "Land creatures you control get +1/+1.",
                effect: StaticEffect::AnthemForFilter {
                    filter: R::Land.and(R::Creature),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    opponents: false,
                    all_players: false,
                    only_your_turn: false,
                    scale_by_counters_on_self: None,
                },
            },
        ],
        ..Default::default()
    }
}

/// Thran Vigil — {1}{B} Enchantment. Whenever one or more artifact and/or
/// creature cards leave your graveyard during your turn, put a +1/+1 counter
/// on target creature you control (CR 603.2c — once per batch).
pub fn thran_vigil() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Thran Vigil",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.or(R::Creature) },
                ]))
                .once_per_batch(),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Vohar, Vodalian Desecrator — {U}{B} 1/2. {T}: loot; discarding an instant
/// or sorcery drains each opponent 1. {2}, sacrifice Vohar (sorcery speed):
/// you may cast target instant or sorcery card from your graveyard this
/// turn, exiled if it would hit the graveyard.
pub fn vohar_vodalian_desecrator() -> CardDefinition {
    use crate::card::MayPlayDuration;
    let is = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        name: "Vohar, Vodalian Desecrator",
        cost: cost(&[u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Phyrexian, CreatureType::Merfolk, CreatureType::Wizard]),
        power: 1,
        toughness: 2,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    Effect::If {
                        cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: is() }),
                        then: Box::new(Effect::Seq(vec![
                            Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
                            Effect::GainLife { who: Selector::You, amount: Value::ONE },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                sac_cost: true,
                sorcery_speed: true,
                effect: Effect::GrantMayPlay {
                    what: target_filtered(is().and(R::InYourGraveyard)),
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: true,
                    pay_own_cost: false,
                    any_color: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Iron Spider, Stark Upgrade — {3} 2/3 vigilance artifact creature. {T}:
/// a +1/+1 counter on each artifact creature and/or Vehicle you control.
/// {2}, remove two +1/+1 counters from among your artifacts: draw a card.
pub fn iron_spider_stark_upgrade() -> CardDefinition {
    use crate::card::{ArtifactSubtype, CounterType};
    CardDefinition {
        name: "Iron Spider, Stark Upgrade",
        cost: cost(&[generic(3)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Spider, CreatureType::Hero]),
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(
                        R::ControlledByYou.and(R::Artifact.and(R::Creature).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                remove_counter_among_filter: Some((Some(CounterType::PlusOnePlusOne), 2, R::Artifact)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Zur, Eternal Schemer — {W}{U}{B} 1/4 flying. Enchantment creatures you
/// control have deathtouch, lifelink, and hexproof. {1}{W}: target non-Aura
/// enchantment you control becomes a creature in addition to its other
/// types with base P/T equal to its mana value.
pub fn zur_eternal_schemer() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility, StaticEffect};
    let ench_creatures = || Selector::EachPermanent(R::Enchantment.and(R::Creature).and(R::ControlledByYou));
    let grant = |keyword: Keyword, description: &'static str| StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword { applies_to: ench_creatures(), keyword },
    };
    CardDefinition {
        name: "Zur, Eternal Schemer",
        cost: cost(&[w(), u(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            grant(Keyword::Deathtouch, "Enchantment creatures you control have deathtouch."),
            grant(Keyword::Lifelink, "Enchantment creatures you control have lifelink."),
            grant(Keyword::Hexproof, "Enchantment creatures you control have hexproof."),
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::BecomeCreature {
                what: target_filtered(
                    R::Enchantment
                        .and(R::Not(Box::new(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura))))
                        .and(R::ControlledByYou),
                ),
                power: Value::ManaValueOf(Box::new(Selector::Target(0))),
                toughness: Value::ManaValueOf(Box::new(Selector::Target(0))),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Karn, Legacy Reforged — {5} */* legendary artifact Golem, P/T = the
/// greatest mana value among artifacts you control. Your upkeep adds {C}
/// per artifact you control, kept through the turn's steps and phases and
/// spendable on artifacts (`ArtifactOnly` — the card also allows other
/// abilities; it only forbids nonartifact spells).
pub fn karn_legacy_reforged() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::mana::SpendRestriction;
    let greatest_mv = Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
        who: PlayerRef::You,
        filter: R::Artifact,
    }));
    CardDefinition {
        name: "Karn, Legacy Reforged",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Golem]),
        power: 0,
        toughness: 0,
        static_abilities: vec![StaticAbility {
            description: "Karn's power and toughness are each equal to the greatest mana value among artifacts you control.",
            effect: StaticEffect::PumpSelfByValue { amount: greatest_mv, per_power: 1, per_toughness: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::AddColorlessKeptThisTurn {
                who: PlayerRef::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Artifact.and(R::ControlledByYou)))),
                restriction: Some(SpendRestriction::ArtifactOnly),
            },
        }],
        ..Default::default()
    }
}

/// Scorn-Blade Berserker — {B} 0/1. Backup 1 (CR 702.164): the backed-up
/// creature, if another, gains "{1}, sacrifice this creature: draw a card"
/// until end of turn — which it prints itself.
pub fn scorn_blade_berserker() -> CardDefinition {
    use crate::card::CounterType;
    let sac_draw = ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        sac_cost: true,
        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        ..Default::default()
    };
    let target = || Selector::TargetFiltered { slot: 0, filter: R::Creature };
    CardDefinition {
        name: "Scorn-Blade Berserker",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Berserker]),
        power: 0,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter { what: target(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::OtherThanSource },
                then: Box::new(Effect::GainActivatedAbility {
                    what: target(),
                    ability: Box::new(sac_draw.clone()),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        activated_abilities: vec![sac_draw],
        ..Default::default()
    }
}

/// Reality Acid — {2}{U} Aura, enchant permanent, vanishing 3. When it
/// leaves the battlefield, the enchanted permanent's controller sacrifices
/// it.
pub fn reality_acid() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Reality Acid",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Vanishing(3)],
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: R::Permanent } },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::SacrificePermanent { what: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        ..Default::default()
    }
}

/// Venser, the Sojourner — {3}{W}{U}, loyalty 3. +2: exile target permanent
/// you own; it returns under your control at the next end step. −1:
/// creatures can't be blocked this turn (granted to the creatures on the
/// battlefield as it resolves). −8: emblem — whenever you cast a spell,
/// exile target permanent.
pub fn venser_the_sojourner() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype};
    use crate::effect::DelayedTriggerKind;
    CardDefinition {
        name: "Venser, the Sojourner",
        cost: cost(&[generic(3), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Venser], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::Exile { what: target_filtered(R::Permanent.and(R::OwnedByYou)) },
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::NextEndStep,
                        body: Box::new(Effect::Move {
                            what: Selector::Target(0),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        }),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::GrantKeyword {
                    what: Selector::EachPermanent(R::Creature),
                    keyword: Keyword::Unblockable,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Venser, the Sojourner".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
                        effect: Effect::Exile { what: target_filtered(R::Permanent) },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Queen Allenal of Ruadach — {G}{W}{W} */* Elf Noble, P/T = creatures you
/// control. Creature tokens created under your control come with a 1/1 white
/// Soldier (CR 614.1a, once per resolution).
pub fn queen_allenal_of_ruadach() -> CardDefinition {
    use crate::card::{DynamicPt, StaticAbility, StaticEffect};
    CardDefinition {
        name: "Queen Allenal of Ruadach",
        cost: cost(&[g(), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Elf, CreatureType::Noble]),
        dynamic_pt: Some(DynamicPt::CreaturesControlled { base: 0 }),
        static_abilities: vec![StaticAbility {
            description: "If one or more creature tokens would be created under your control, those tokens plus a 1/1 white Soldier creature token are created instead.",
            effect: StaticEffect::CreatureTokenCreationAddsToken {
                definition: token_1_1("Soldier", Color::White, CreatureType::Soldier),
            },
        }],
        ..Default::default()
    }
}

/// Rabble Rousing — {4}{W} Enchantment, hideaway 5. Attacking with one or
/// more creatures makes that many 1/1 green-white Citizens; then with ten
/// or more creatures you may play the hidden card free.
pub fn rabble_rousing() -> CardDefinition {
    let mut citizen = token_1_1("Citizen", Color::Green, CreatureType::Citizen);
    citizen.colors = vec![Color::Green, Color::White];
    let attackers = Value::CountOf(Box::new(Selector::EachPermanent(
        R::Creature.and(R::IsAttacking).and(R::ControlledByYou),
    )));
    CardDefinition {
        name: "Rabble Rousing",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Hideaway { count: Value::Const(5) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).once_per_batch(),
                effect: Effect::Seq(vec![
                    mint(citizen, attackers),
                    Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))),
                            Value::Const(10),
                        ),
                        then: Box::new(Effect::Seq(vec![
                            Effect::PlayLandAmongNow { what: Selector::CardExiledWithSource },
                            Effect::CastWithoutPayingImmediate {
                                what: Selector::CardExiledWithSource,
                                source_zone: crate::card::Zone::Exile,
                                exile_after: false,
                                copy: false,
                                reduce_generic: 0,
                                pay_own_cost: false,
                            },
                        ])),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Tinybones, Bauble Burglar — {1}{B} 1/3. An opponent's discarded card is
/// exiled with a stash counter. During your turn you may play stashed cards
/// you don't own, spending mana as though of any type: each of your upkeeps
/// grants this turn's play on all of them, and a card stashed on your turn is
/// granted at once (a grant outlives Tinybones leaving mid-turn). {3}{B},
/// {T} (sorcery speed): each opponent discards a card.
pub fn tinybones_bauble_burglar() -> CardDefinition {
    use crate::card::{CounterType, MayPlayDuration};
    use crate::effect::ZoneRef;
    let grant = |what: Selector| Effect::GrantMayPlay {
        what,
        duration: MayPlayDuration::EndOfThisTurn,
        to_owner: false,
        exile_after: false,
        pay_own_cost: true,
        any_color: true,
    };
    CardDefinition {
        name: "Tinybones, Bauble Burglar",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Skeleton, CreatureType::Rogue]),
        power: 1,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
                effect: Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Stash, amount: Value::ONE },
                    Effect::If {
                        cond: Predicate::IsTurnOf(PlayerRef::You),
                        then: Box::new(grant(Selector::LastMoved)),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: grant(Selector::EachMatching {
                    zone: ZoneRef::Exile,
                    filter: R::WithCounter(CounterType::Stash).and(R::Not(Box::new(R::OwnedByYou))),
                }),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Discard { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE, random: false },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Grolnok, the Omnivore — {2}{G}{U} 3/3 Frog. A Frog you control attacking
/// mills three; a permanent card milled into your graveyard is exiled with a
/// croak counter, and you may play it while Grolnok remains (the grant is
/// stamped as it is exiled and bound to this Grolnok).
pub fn grolnok_the_omnivore() -> CardDefinition {
    use crate::card::{CounterType, MayPlayDuration};
    CardDefinition {
        name: "Grolnok, the Omnivore",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Frog]),
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Frog)),
                }),
                effect: Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardMilled, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::PermanentCard,
                }),
                effect: Effect::Seq(vec![
                    Effect::Move { what: Selector::TriggerSource, to: ZoneDest::Exile },
                    Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Croak, amount: Value::ONE },
                    Effect::GrantMayPlay {
                        what: Selector::LastMoved,
                        duration: MayPlayDuration::WhileSourceOnBattlefield { source: crate::card::CardId(0), one_spell_a_turn: false },
                        to_owner: false,
                        exile_after: false,
                        pay_own_cost: true,
                        any_color: false,
                    },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Mnemonic Betrayal — {1}{U}{B} sorcery. Exile all opponents' graveyards;
/// this turn you may cast spells from among them with mana of any type, and
/// at the next end step the ones still exiled go back. Exiles itself.
pub fn mnemonic_betrayal() -> CardDefinition {
    use crate::card::MayPlayDuration;
    use crate::effect::{DelayedTriggerKind, ZoneRef};
    CardDefinition {
        name: "Mnemonic Betrayal",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Sorcery],
        exile_on_resolve: true,
        effect: Effect::Seq(vec![
            Effect::ExileLinked {
                what: Selector::EachMatching { zone: ZoneRef::Graveyard(PlayerRef::EachOpponent), filter: R::Any },
            },
            // "Cast spells": lands can't be cast, so they get no grant.
            Effect::GrantMayPlay {
                what: Selector::ExiledThisResolution { filter: R::Not(Box::new(R::Land)) },
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Move { what: Selector::CardExiledWithSource, to: ZoneDest::Graveyard }),
            },
        ]),
        ..Default::default()
    }
}

/// Rona, Herald of Invasion // Rona, Tolarian Obliterator — {1}{U} 1/3.
/// Untaps when you cast a legendary spell; {T}: loot; {5}{B/P} (sorcery
/// speed): transform. Back: 5/5 trample; whenever a source deals damage to
/// it, that source's controller exiles a card at random from hand — you may
/// put a land onto the battlefield or cast anything else free.
pub fn rona_herald_of_invasion() -> CardDefinition {
    use crate::mana::ManaSymbol;
    let may = |description: &str, body: Effect| Effect::MayDo { description: description.into(), body: Box::new(body) };
    let damager = PlayerRef::LastDamagerControllerOf(Box::new(Selector::This));
    let back = CardDefinition {
        name: "Rona, Tolarian Obliterator",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Phyrexian, CreatureType::Wizard]),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::RandomOf(Box::new(Selector::EachMatching { zone: crate::effect::ZoneRef::Hand(damager), filter: R::Any })),
                    to: ZoneDest::Exile,
                },
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: R::Land },
                    then: Box::new(may(
                        "Put the exiled land onto the battlefield under your control?",
                        Effect::Move { what: Selector::LastMoved, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } },
                    )),
                    // The free cast asks its own "may".
                    else_: Box::new(Effect::CastWithoutPayingImmediate {
                        what: Selector::LastMoved,
                        source_zone: crate::card::Zone::Exile,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    }),
                },
            ]),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Rona, Herald of Invasion",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::Wizard]),
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasSupertype(Supertype::Legendary),
            }),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: crate::mana::ManaCost { symbols: vec![ManaSymbol::Generic(5), ManaSymbol::Phyrexian(Color::Black)] },
                sorcery_speed: true,
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Mech Hangar — Land. {T}: {C}; {T}: one mana of any color, spent only on a
/// Pilot or Vehicle spell; {3}, {T}: target Vehicle becomes an artifact
/// creature until end of turn.
pub fn mech_hangar() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    use crate::effect::ManaPayload;
    use crate::mana::SpendRestriction;
    CardDefinition {
        name: "Mech Hangar",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                        SpendRestriction::PilotOrVehicleSpells,
                    ),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                tap_cost: true,
                effect: Effect::AnimateAsCreature {
                    what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Inga and Esika — {2}{G}{U} 4/4 Human God. Creatures you control have
/// vigilance and "{T}: Add one mana of any color, only for a creature spell";
/// a creature spell cast with three or more mana from creatures draws a card.
pub fn inga_and_esika() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    use crate::effect::ManaPayload;
    use crate::mana::SpendRestriction;
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Inga and Esika",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Human, CreatureType::God]),
        power: 4,
        toughness: 4,
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control have vigilance.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(), keyword: Keyword::Vigilance },
            },
            StaticAbility {
                description: "Creatures you control have \"{T}: Add one mana of any color. Spend this mana only to cast a creature spell.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: yours(),
                    ability: ActivatedAbility {
                        tap_cost: true,
                        effect: Effect::AddMana {
                            who: PlayerRef::You,
                            pool: ManaPayload::Restricted(
                                Box::new(ManaPayload::AnyOneColor(Value::ONE)),
                                SpendRestriction::CreatureOnlyFromCreature,
                            ),
                        },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                Predicate::ValueAtLeast(Value::CreatureManaSpentToCastTriggerSource, Value::Const(3)),
            ])),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Tam, Mindful First-Year — {1}{G/U} 2/2 Gorgon Wizard. Each other creature
/// you control has hexproof from each of its colors; {T}: target creature you
/// control becomes all colors until end of turn.
pub fn tam_mindful_first_year() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Tam, Mindful First-Year",
        cost: cost(&[generic(1), hybrid(Color::Green, Color::Blue)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Gorgon, CreatureType::Wizard]),
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control has hexproof from each of its colors.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::HexproofFromItsColors,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeColor {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                colors: Color::ALL.to_vec(),
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The Skullspore Nexus — {6}{G}{G} Legendary Artifact; costs {X} less, X the
/// greatest power among your creatures. One or more of your nontoken
/// creatures dying makes a green Fungus Dinosaur whose base P/T is their total
/// power; {2}, {T}: double target creature's power until end of turn.
pub fn the_skullspore_nexus() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let fungus = TokenDefinition {
        name: "Fungus Dinosaur".into(),
        power: 0,
        toughness: 0,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: creature_types(vec![CreatureType::Fungus, CreatureType::Dinosaur]),
        ..Default::default()
    };
    let total = || Value::PowerOf(Box::new(Selector::BoundTriggerBatch));
    CardDefinition {
        name: "The Skullspore Nexus",
        cost: cost(&[generic(6), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is the greatest power among creatures you control.",
            effect: StaticEffect::SelfCostReducedByGreatestPower,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::IsToken)) })
                .once_per_batch(),
            effect: Effect::WithTriggerBatch {
                body: Box::new(Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(fungus) },
                    Effect::SetBasePT { what: Selector::LastCreatedToken, power: total(), toughness: total(), duration: Duration::Permanent },
                ])),
                ids: vec![],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::PowerOf(Box::new(Selector::Target(0))),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Welcome to . . . // Jurassic Park — {1}{G}{G} Saga. I: for each opponent,
/// up to one target noncreature artifact of theirs becomes a 0/4 defender
/// Wall artifact creature while this Saga remains (a control change of the
/// Saga isn't watched); II: a 3/3 trample Dinosaur with haste this turn; III:
/// destroy all Walls, then exile and return transformed. Jurassic Park
/// (Legendary Land): your graveyard's Dinosaur cards have escape (mana cost
/// plus exiling three others); {T}: {G} per Dinosaur you control.
pub fn welcome_to() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, StaticAbility, StaticEffect};
    use crate::effect::ManaPayload;
    let dino = TokenDefinition {
        name: "Dinosaur".into(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: creature_types(vec![CreatureType::Dinosaur]),
        ..Default::default()
    };
    let park = CardDefinition {
        name: "Jurassic Park",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "Each Dinosaur card in your graveyard has escape: its mana cost plus exile three other cards from your graveyard.",
            effect: StaticEffect::GraveyardCardsHaveEscapeMatching {
                filter: R::HasCreatureType(CreatureType::Dinosaur),
                exile_count: 3,
                your_turn_only: false,
                once_per_turn: false,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        R::Creature.and(R::HasCreatureType(CreatureType::Dinosaur)).and(R::ControlledByYou),
                    ))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Welcome to . . .",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ControlledByOpponent),
                        effect: Box::new(Effect::BecomeCreature {
                            what: Selector::Target(0),
                            power: Value::Const(0),
                            toughness: Value::Const(4),
                            creature_types: vec![CreatureType::Wall],
                            keywords: vec![Keyword::Defender],
                            duration: Duration::WhileSourceOnBattlefield,
                        }),
                    }),
                },
            ),
            (
                2,
                Effect::Seq(vec![
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(dino) },
                    Effect::GrantKeyword { what: Selector::LastCreatedToken, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            ),
            (
                3,
                Effect::Seq(vec![
                    Effect::Destroy { what: Selector::EachPermanent(R::HasCreatureType(CreatureType::Wall)) },
                    Effect::ExileSelfReturnTransformed,
                ]),
            ),
        ],
        back_face: Some(Box::new(park)),
        ..Default::default()
    }
}

/// Mechtitan Core — {2} Vehicle 2/4, crew 2. {5}, exile it and four other
/// artifact creatures and/or Vehicles you control: create Mechtitan, a
/// legendary all-colors 10/10 Construct with flying, vigilance, trample,
/// lifelink and haste; when it leaves, the others return tapped under their
/// owners' control (the Core stays exiled). The four are exiled as the ability
/// resolves, gated on four being there (INCOMPLETE_CARDS).
pub fn mechtitan_core() -> CardDefinition {
    use crate::card::ArtifactSubtype;
    let fuel = || R::Artifact.and(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)))
        .and(R::ControlledByYou)
        .and(R::OtherThanSource);
    let mechtitan = TokenDefinition {
        name: "Mechtitan".into(),
        power: 10,
        toughness: 10,
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Trample, Keyword::Lifelink, Keyword::Haste],
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: Color::ALL.to_vec(),
        subtypes: creature_types(vec![CreatureType::Construct]),
        ..Default::default()
    };
    CardDefinition {
        name: "Mechtitan Core",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Crew(2)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            exile_self_cost: true,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountOf(Box::new(Selector::EachPermanent(fuel()))),
                Value::Const(4),
            )),
            effect: Effect::Seq(vec![
                Effect::ExileLinked {
                    what: Selector::Take {
                        inner: Box::new(Selector::EachPermanent(R::Artifact.and(
                            R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                        ).and(R::ControlledByYou))),
                        count: Box::new(Value::Const(4)),
                    },
                },
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(mechtitan) },
                Effect::WhenLastCreatedTokenLeaves {
                    body: Box::new(Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: true },
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Efficient Construction — {3}{U} Enchantment. Casting an artifact spell
/// makes a 1/1 flying Thopter.
pub fn efficient_construction() -> CardDefinition {
    CardDefinition {
        name: "Efficient Construction",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(TokenDefinition {
                    name: "Thopter".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Artifact, CardType::Creature],
                    subtypes: creature_types(vec![CreatureType::Thopter]),
                    ..Default::default()
                }),
            },
        }],
        ..Default::default()
    }
}

/// Crystal Skull, Isu Spyglass — {2}{U}{U} Legendary Artifact. Look at your
/// library's top any time; play historic lands and cast historic spells from
/// there; {T}: {U}.
pub fn crystal_skull_isu_spyglass() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Crystal Skull, Isu Spyglass",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "You may play historic lands and cast historic spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop { filter: R::historic() },
            },
        ],
        activated_abilities: vec![crate::sets::tap_add(Color::Blue)],
        ..Default::default()
    }
}

/// Mirrodin Besieged — {2}{U} Enchantment; as it enters, choose Mirran (an
/// artifact spell makes a 1/1 Myr) or Phyrexian (your end step loots, then
/// with fifteen or more artifact cards in your graveyard target opponent
/// loses the game).
pub fn mirrodin_besieged() -> CardDefinition {
    use crate::card::EnterMode;
    let myr = TokenDefinition {
        name: "Myr".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Myr]),
        ..Default::default()
    };
    CardDefinition {
        name: "Mirrodin Besieged",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        enter_modes: Some(vec![
            EnterMode {
                label: "Mirran",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                        .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
                    effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(myr) },
                }],
                ..Default::default()
            },
            EnterMode {
                label: "Phyrexian",
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                    effect: Effect::Seq(vec![
                        Effect::Draw { who: Selector::You, amount: Value::ONE },
                        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                        Effect::If {
                            cond: Predicate::ValueAtLeast(
                                Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Artifact },
                                Value::Const(15),
                            ),
                            // "Target opponent" — a player slot read through
                            // `ControllerOf`, as Tariel's.
                            then: Box::new(Effect::LoseGame {
                                who: PlayerRef::ControllerOf(Box::new(target_filtered(R::OpponentPlayer))),
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    ]),
                }],
                ..Default::default()
            },
        ]),
        ..Default::default()
    }
}

/// Rings of Brighthearth — {3} Artifact. Whenever you activate a nonmana
/// ability, you may pay {2} to copy it (new targets allowed).
pub fn rings_of_brighthearth() -> CardDefinition {
    CardDefinition {
        name: "Rings of Brighthearth",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {2} to copy that ability?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::CopyActivatedAbilityMayChooseTargets),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// The Reality Chip — {1}{U} Legendary Artifact Creature — Equipment Jellyfish
/// 0/4. Look at your library's top any time; while attached to a creature, play
/// lands and cast spells from there. Reconfigure {2}{U}.
pub fn the_reality_chip() -> CardDefinition {
    use crate::card::{ArtifactSubtype, StaticAbility, StaticEffect};
    CardDefinition {
        name: "The Reality Chip",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            creature_types: vec![CreatureType::Jellyfish],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Reconfigure(cost(&[generic(2), u()]))],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::TopOfLibraryRevealed,
            },
            StaticAbility {
                description: "As long as The Reality Chip is attached to a creature, you may play lands and cast spells from the top of your library.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::EntityMatches { what: Selector::This, filter: R::AttachedToCreature },
                    inner: Box::new(StaticEffect::PlayFromLibraryTop { filter: R::Any }),
                },
            },
        ],
        ..Default::default()
    }
}

/// Illustrious Wanderglyph — {4}{W} 2/2 Golem, ascend. Other artifact
/// creatures you control get +2/+2 with the city's blessing; each upkeep makes
/// a 1/1 Gnome artifact creature.
pub fn illustrious_wanderglyph() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    let gnome = TokenDefinition {
        name: "Gnome".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Gnome]),
        ..Default::default()
    };
    CardDefinition {
        name: "Illustrious Wanderglyph",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Golem]),
        power: 2,
        toughness: 2,
        static_abilities: vec![
            StaticAbility { description: "Ascend", effect: StaticEffect::Ascend },
            StaticAbility {
                description: "Other artifact creatures you control get +2/+2 as long as you have the city's blessing.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasCityBlessing { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::PumpPT {
                        applies_to: Selector::EachPermanent(
                            R::Artifact.and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                        ),
                        power: 2,
                        toughness: 2,
                    }),
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(gnome) },
        }],
        ..Default::default()
    }
}

/// The Gnome Soldier both faces of Thousand Moons Smithy make: P/T each equal
/// to the artifacts and/or creatures you control.
fn gnome_soldier() -> TokenDefinition {
    use crate::card::{StaticAbility, StaticEffect};
    TokenDefinition {
        name: "Gnome Soldier".into(),
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![Color::White],
        subtypes: creature_types(vec![CreatureType::Gnome, CreatureType::Soldier]),
        static_abilities: vec![StaticAbility {
            description: "This token's power and toughness are each equal to the number of artifacts and/or creatures you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents { filter: R::Artifact.or(R::Creature), per_power: 1, per_toughness: 1 },
        }],
        ..Default::default()
    }
}

/// Thousand Moons Smithy // Barracks of the Thousand — {2}{W}{W} Legendary
/// Artifact. ETB: a Gnome Soldier; at your first main phase you may tap five
/// untapped artifacts and/or creatures to transform. Barracks (Legendary
/// Artifact Land): {T}: {W}; an artifact or creature spell cast with its mana
/// makes a Gnome Soldier (`SpendRestriction::MarksCast`).
pub fn thousand_moons_smithy() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::mana::SpendRestriction;
    let barracks = CardDefinition {
        name: "Barracks of the Thousand",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Land],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(Box::new(ManaPayload::Colors(vec![Color::White])), SpendRestriction::MarksCast),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.or(R::Creature) },
                Predicate::CastWithMarkedMana { what: Selector::TriggerSource },
            ])),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(gnome_soldier()) },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Thousand Moons Smithy",
        cost: cost(&[generic(2), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(gnome_soldier()) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
                effect: Effect::MayTap {
                    description: "Tap five untapped artifacts and/or creatures to transform Thousand Moons Smithy?".into(),
                    filter: R::Artifact.or(R::Creature).and(R::ControlledByYou),
                    count: Value::Const(5),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: None,
                },
            },
        ],
        back_face: Some(Box::new(barracks)),
        ..Default::default()
    }
}

/// Raid Bombardment — {2}{R} Enchantment. A creature you control with power 2
/// or less attacking deals 1 to the player or planeswalker it attacks (from
/// this enchantment).
pub fn raid_bombardment() -> CardDefinition {
    CardDefinition {
        name: "Raid Bombardment",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::PowerAtMost(2) }),
            effect: Effect::DealDamage { to: Selector::AttackedByTriggerSource, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Patriar's Seal — {3} Artifact. {T}: any color; {1}, {T}: untap target
/// legendary creature you control.
pub fn patriars_seal() -> CardDefinition {
    CardDefinition {
        name: "Patriar's Seal",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            crate::sets::tap_add_any_color(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::Untap {
                    what: target_filtered(R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou)),
                    up_to: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ashcoat of the Shadow Swarm — {3}{B} 3/4 Rat Warlock. Attacking or
/// blocking, other Rats you control get +X/+X (X = your Rats); your end step
/// may mill four to return up to two Rat creature cards to hand.
pub fn ashcoat_of_the_shadow_swarm() -> CardDefinition {
    let rats = || R::HasCreatureType(CreatureType::Rat).and(R::ControlledByYou);
    let x = || Value::CountOf(Box::new(Selector::EachPermanent(rats())));
    let pump = || Effect::PumpPT {
        what: Selector::EachPermanent(rats().and(R::OtherThanSource)),
        power: x(),
        toughness: x(),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Ashcoat of the Shadow Swarm",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Rat, CreatureType::Warlock]),
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource), effect: pump() },
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: pump() },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Mill four and return up to two Rat creature cards?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Mill { who: Selector::You, amount: Value::Const(4) },
                        Effect::Move {
                            what: Selector::Take {
                                inner: Box::new(Selector::EachMatching {
                                    zone: crate::effect::ZoneRef::Graveyard(PlayerRef::You),
                                    filter: R::Creature.and(R::HasCreatureType(CreatureType::Rat)),
                                }),
                                count: Box::new(Value::Const(2)),
                            },
                            to: ZoneDest::Hand(PlayerRef::You),
                        },
                    ])),
                },
            },
        ],
        ..Default::default()
    }
}

/// Rat King, Verminister — {1}{B} 1/1 Rat Avatar. Disappear — your end step,
/// if a permanent left the battlefield under your control this turn: a 1/1
/// Rat and a +1/+1 counter. {T}, sacrifice three Rats: return target creature
/// card and every other card with its name from your graveyard, tapped.
pub fn rat_king_verminister() -> CardDefinition {
    use crate::card::CounterType;
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: creature_types(vec![CreatureType::Rat]),
        ..Default::default()
    };
    let tapped = || ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true };
    CardDefinition {
        name: "Rat King, Verminister",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: creature_types(vec![CreatureType::Rat, CreatureType::Avatar]),
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(rat) },
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Rat), 3)),
            sac_other_may_be_source: true,
            // The target and every card sharing its name, in its owner's
            // (your) graveyard.
            effect: Effect::Move {
                what: Selector::SharingNameWith(Box::new(target_filtered(R::Creature.and(R::InYourGraveyard)))),
                to: tapped(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Plague of Vermin — {6}{B} Sorcery. Starting with you, each player may pay
/// any amount of life, repeating until no one pays; each makes a 1/1 Rat per
/// life paid (`Effect::EachPlayerPaysLifeForTokens`).
pub fn plague_of_vermin() -> CardDefinition {
    CardDefinition {
        name: "Plague of Vermin",
        cost: cost(&[generic(6), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerPaysLifeForTokens {
            definition: Arc::new(TokenDefinition {
                name: "Rat".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Black],
                subtypes: creature_types(vec![CreatureType::Rat]),
                ..Default::default()
            }),
        },
        ..Default::default()
    }
}
