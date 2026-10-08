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
