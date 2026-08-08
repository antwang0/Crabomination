//! Scourge (SCG), closing wave — the rares that each needed a primitive of
//! their own: Karona, the Dragon enchantments, the morph payoffs and the
//! oddball locks. Tests in `classic_sets/scg2`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    Keyword, LandType, Predicate, SelectionRequirement as R, StateTriggeredAbility,
    StaticAbility, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, EventKind, EventScope, EventSpec, LibraryPosition, PlayerRef, Selector,
    StaticEffect, Value, ZoneDest,
    shortcut::{target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        ..Default::default()
    }
}

fn aura(name: &'static str, c: ManaCost, attach_to: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(attach_to) },
        ..Default::default()
    }
}

/// "When this creature is turned face up, [effect]." (CR 708.8)
fn on_turn_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource),
        effect,
    }
}

fn at_each_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
        effect,
    }
}

fn at_each_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
        effect,
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Ageless Sentinels — a blocking Wall that permanently sheds its type line.
pub fn ageless_sentinels() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::BecomeCreatureType {
                    what: Selector::This,
                    creature_types: vec![CreatureType::Bird, CreatureType::Giant],
                    duration: Duration::Permanent,
                },
                Effect::LoseKeyword {
                    what: Selector::This,
                    keyword: Keyword::Defender,
                    duration: Duration::Permanent,
                },
            ]),
        }],
        ..creature("Ageless Sentinels", cost(&[generic(3), w()]), vec![CreatureType::Wall], 4, 4)
    }
}

/// Dimensional Breach — exile the board; each upkeep its owner rebuilds one.
pub fn dimensional_breach() -> CardDefinition {
    sorcery(
        "Dimensional Breach",
        cost(&[generic(5), w(), w()]),
        Effect::DimensionalBreach,
    )
}

/// Force Bubble — damage to you becomes depletion counters; four pops it.
pub fn force_bubble() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to you, put that many depletion counters on \
                          this enchantment instead.",
            effect: StaticEffect::ReplaceDamageToYouWithCountersOnSource {
                kind: CounterType::Depletion,
            },
        }],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::SourceHasCountersAtLeast {
                counter: CounterType::Depletion,
                n: 4,
            },
            effect: Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::IsSource,
            },
        }),
        triggered_abilities: vec![at_each_end_step(Effect::RemoveAllCounters {
            what: Selector::This,
        })],
        ..enchantment("Force Bubble", cost(&[generic(2), w(), w()]))
    }
}

/// Gilded Light — shroud for a turn, or a cantrip when you don't need it.
pub fn gilded_light() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        ..instant(
            "Gilded Light",
            cost(&[generic(1), w()]),
            Effect::PlayerGainsShroudThisTurn { who: PlayerRef::You },
        )
    }
}

/// Karona's Zealot — Morph {3}{W}{W}: this turn its damage lands elsewhere.
pub fn karonas_zealot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(3), w(), w()]))],
        triggered_abilities: vec![on_turn_up(Effect::RedirectDamageToThisThisTurn {
            to: target_filtered(R::Creature),
        })],
        ..creature(
            "Karona's Zealot",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            5,
        )
    }
}

/// Trap Digger — lay trap counters on your lands, then spring them.
pub fn trap_digger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2), w()]),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    kind: CounterType::Trap,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_other_filter: Some((R::Land.and(R::WithCounter(CounterType::Trap)), 1)),
                effect: Effect::DealDamage {
                    to: target_filtered(
                        R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying).negate()),
                    ),
                    amount: Value::Const(3),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Trap Digger",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            3,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Day of the Dragons — swap your board for a flight of 5/5 Dragons.
pub fn day_of_the_dragons() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ExileYourCreaturesForDragons {
                    token: Box::new(TokenDefinition {
                        name: "Dragon".into(),
                        power: 5,
                        toughness: 5,
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Red],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Dragon],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::PermanentLeavesBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Seq(vec![
                    Effect::SacrificeAllMatching {
                        who: Selector::You,
                        filter: R::HasCreatureType(CreatureType::Dragon),
                    },
                    Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                ]),
            },
        ],
        ..enchantment("Day of the Dragons", cost(&[generic(4), u(), u(), u()]))
    }
}

/// Faces of the Past — every death ripples through its creature type.
pub fn faces_of_the_past() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::ChooseMode(vec![
                Effect::Tap {
                    what: Selector::SharingCreatureTypeWith(Box::new(Selector::TriggerSource)),
                },
                Effect::Untap {
                    what: Selector::SharingCreatureTypeWith(Box::new(Selector::TriggerSource)),
                    up_to: None,
                },
            ]),
        }],
        ..enchantment("Faces of the Past", cost(&[generic(2), u()]))
    }
}

/// Long-Term Plans — tutor anything, but it lands third from the top.
pub fn long_term_plans() -> CardDefinition {
    instant(
        "Long-Term Plans",
        cost(&[generic(2), u()]),
        Effect::Search {
            who: PlayerRef::You,
            filter: R::Any,
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::FromTop(2) },
        },
    )
}

/// Metamorphose — bounce a permanent to the top, and they may redeploy.
pub fn metamorphose() -> CardDefinition {
    instant(
        "Metamorphose",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByOpponent)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Top,
                },
            },
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::OwnerOfMoved,
                filter: R::Artifact
                    .or(R::Creature)
                    .or(R::Enchantment)
                    .or(R::Land),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        ]),
    )
}

/// Mischievous Quanar — re-morph at will to fork the next spell.
pub fn mischievous_quanar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(1), u(), u()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u(), u()]),
            effect: Effect::TurnFaceDown { what: Selector::This },
            ..Default::default()
        }],
        triggered_abilities: vec![on_turn_up(Effect::CopySpell {
            what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(
                R::HasCardType(CardType::Sorcery),
            ))),
            count: Value::ONE,
        })],
        ..creature("Mischievous Quanar", cost(&[generic(4), u()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Mistform Warchief — discounts whatever type it happens to be wearing.
pub fn mistform_warchief() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creature spells you cast that share a creature type with this creature \
                          cost {1} less to cast.",
            effect: StaticEffect::SharedCreatureTypeSpellCostReduction { amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeChosenCreatureType {
                what: Selector::This,
                duration: Duration::EndOfTurn,
                excluded: vec![],
            },
            ..Default::default()
        }],
        ..creature("Mistform Warchief", cost(&[generic(2), u()]), vec![CreatureType::Illusion], 1, 3)
    }
}

/// Parallel Thoughts — draw off a seven-card pile you stacked yourself.
pub fn parallel_thoughts() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileFaceDownDrawPile { count: Value::Const(7) },
        }],
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card, you may instead put the top card of the pile \
                          you exiled into your hand.",
            effect: StaticEffect::MayDrawFromSourceExilePile,
        }],
        ..enchantment("Parallel Thoughts", cost(&[generic(3), u(), u()]))
    }
}

/// Pemmin's Aura — the blue Swiss army knife bolted onto one creature.
pub fn pemmins_aura() -> CardDefinition {
    let pump = |power: i32, toughness: i32| ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        effect: Effect::PumpPT {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    let grant = |keyword: Keyword| ActivatedAbility {
        mana_cost: cost(&[u()]),
        effect: Effect::GrantKeyword {
            what: Selector::AttachedTo(Box::new(Selector::This)),
            keyword,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[u()]),
                effect: Effect::Untap {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                    up_to: None,
                },
                ..Default::default()
            },
            grant(Keyword::Flying),
            grant(Keyword::Shroud),
            pump(1, -1),
            pump(-1, 1),
        ],
        ..aura("Pemmin's Aura", cost(&[generic(1), u(), u()]), R::Creature)
    }
}

/// Proteus Machine — Morph {0}: flip it up and name its type for keeps.
pub fn proteus_machine() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Morph(ManaCost::default())],
        triggered_abilities: vec![on_turn_up(Effect::BecomeChosenCreatureType {
            what: Selector::This,
            duration: Duration::Permanent,
            excluded: vec![],
        })],
        ..creature(
            "Proteus Machine",
            cost(&[generic(3)]),
            vec![CreatureType::Shapeshifter],
            2,
            2,
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Call to the Grave — a Zombie-only board, and it folds once it's empty.
pub fn call_to_the_grave() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            at_each_upkeep(Effect::Sacrifice {
                who: Selector::Player(PlayerRef::ActivePlayer),
                count: Value::ONE,
                filter: R::Creature.and(R::HasCreatureType(CreatureType::Zombie).negate()),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(R::Creature),
                )))),
                effect: Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::IsSource,
                },
            },
        ],
        ..enchantment("Call to the Grave", cost(&[generic(4), b()]))
    }
}

/// Fatal Mutation — a one-mana answer aimed squarely at morphs.
pub fn fatal_mutation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::EnchantedBySource),
            effect: Effect::DestroyNoRegen {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..aura("Fatal Mutation", cost(&[b()]), R::Creature)
    }
}

/// Lethal Vapors — nothing may enter, and undoing it costs you a turn.
pub fn lethal_vapors() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Destroy { what: Selector::TriggerSource },
        }],
        activated_abilities: vec![ActivatedAbility {
            any_player: true,
            effect: Effect::Seq(vec![
                Effect::Destroy { what: Selector::This },
                Effect::SkipTurns { who: PlayerRef::You, count: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..enchantment("Lethal Vapors", cost(&[generic(2), b(), b()]))
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Dragonstorm — one copy per spell before it, each fetching a Dragon.
pub fn dragonstorm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..sorcery(
            "Dragonstorm",
            cost(&[generic(8), r()]),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::PermanentCard.and(R::HasCreatureType(CreatureType::Dragon)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        )
    }
}

/// Form of the Dragon — you become the Dragon: 5 damage a turn, 5 life a turn.
pub fn form_of_the_dragon() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                ),
                effect: Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
            },
            at_each_end_step(Effect::SetLifeTotal {
                who: Selector::You,
                amount: Value::Const(5),
            }),
        ],
        static_abilities: vec![StaticAbility {
            description: "Creatures without flying can't attack you.",
            effect: StaticEffect::CreaturesCantAttackController {
                protect_planeswalkers: false,
                filter: Some(R::HasKeyword(Keyword::Flying).negate()),
            },
        }],
        ..enchantment("Form of the Dragon", cost(&[generic(4), r(), r(), r()]))
    }
}

/// Goblin Psychopath — a 5/5 that flips a coin over whom it hits.
pub fn goblin_psychopath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Noop),
                on_tails: Box::new(Effect::RedirectNextCombatDamageToController {
                    what: Selector::This,
                }),
            },
        }],
        ..creature(
            "Goblin Psychopath",
            cost(&[generic(3), r()]),
            vec![CreatureType::Goblin, CreatureType::Mutant],
            5,
            5,
        )
    }
}

/// Grip of Chaos — every single-target spell or ability rerolls its target.
pub fn grip_of_chaos() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Whenever a spell or ability is put onto the stack, if it has a single \
                          target, reselect its target at random.",
            effect: StaticEffect::RandomizeSingleTargets,
        }],
        ..enchantment("Grip of Chaos", cost(&[generic(4), r(), r()]))
    }
}

/// Rock Jockey — a 3/3 that fights your land drop for the turn.
pub fn rock_jockey() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::ValueAtMost(
            Value::LandsPlayedThisTurn(PlayerRef::You),
            Value::ZERO,
        )),
        static_abilities: vec![StaticAbility {
            description: "You can't play lands if this creature was cast this turn.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::All(vec![
                    Predicate::SourceWasCast,
                    Predicate::EntityMatches {
                        what: Selector::This,
                        filter: R::EnteredThisTurn,
                    },
                ]),
                inner: Box::new(StaticEffect::ControllerCantPlayLands),
            },
        }],
        ..creature("Rock Jockey", cost(&[generic(2), r()]), vec![CreatureType::Goblin], 3, 3)
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Ambush Commander — your Forests stand up as Elves and can be eaten.
pub fn ambush_commander() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Forests you control are 1/1 green Elf creatures that are still lands.",
            effect: StaticEffect::MatchingLandsAreCreatures {
                filter: R::HasLandType(LandType::Forest).and(R::ControlledByYou),
                power: 1,
                toughness: 1,
                keywords: vec![],
                creature_types: vec![CreatureType::Elf],
                colors: vec![Color::Green],
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Elf), 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Ambush Commander", cost(&[generic(3), g(), g()]), vec![CreatureType::Elf], 2, 2)
    }
}

/// Divergent Growth — every land you control turns into any-colour mana.
pub fn divergent_growth() -> CardDefinition {
    instant(
        "Divergent Growth",
        cost(&[g()]),
        Effect::GrantActivatedAbilityToMatching {
            filter: R::Land.and(R::ControlledByYou),
            ability: Box::new(ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Forgotten Ancient — it swells on every spell and hands the counters out.
pub fn forgotten_ancient() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
                effect: Effect::MayDo {
                    description: "Put a +1/+1 counter on Forgotten Ancient?".into(),
                    body: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::SelfSource,
                ),
                effect: Effect::DistributeCountersFromSource {
                    kind: CounterType::PlusOnePlusOne,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            },
        ],
        ..creature("Forgotten Ancient", cost(&[generic(3), g()]), vec![CreatureType::Elemental], 0, 3)
    }
}

/// Primitive Etchings — reveal each turn's first draw and chain off creatures.
pub fn primitive_etchings() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::FirstCardDrawnThisTurn, EventScope::YourControl),
            effect: Effect::RevealDrawnCardThenIf {
                filter: R::Creature,
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..enchantment("Primitive Etchings", cost(&[generic(2), g(), g()]))
    }
}

/// Root Elemental — Morph {5}{G}{G}: flip it up and cheat a fatty in.
pub fn root_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(5), g(), g()]))],
        triggered_abilities: vec![on_turn_up(Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Creature,
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: None,
        })],
        ..creature(
            "Root Elemental",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Elemental],
            6,
            5,
        )
    }
}

/// Xantid Swarm — a 0/1 flier that shuts the blocker's hand for a turn.
pub fn xantid_swarm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PlayerCantCastMatchingThisTurn {
                who: PlayerRef::DefendingPlayer,
                filter: R::Any,
            },
        }],
        ..creature("Xantid Swarm", cost(&[g()]), vec![CreatureType::Insect], 0, 1)
    }
}

// ── Gold / colourless ───────────────────────────────────────────────────────

/// Karona, False God — she changes hands every upkeep and pumps a whole type.
pub fn karona_false_god() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![
            at_each_upkeep(Effect::Seq(vec![
                Effect::Untap { what: Selector::This, up_to: None },
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::ActivePlayer),
                    duration: Duration::Permanent,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::ChooseCreatureTypeThen {
                    who: PlayerRef::You,
                    then: Box::new(Effect::PumpPT {
                        what: Selector::EachPermanent(R::Creature.and(R::IsSourceChosenCreatureType)),
                        power: Value::Const(3),
                        toughness: Value::Const(3),
                        duration: Duration::EndOfTurn,
                    }),
                },
            },
        ],
        ..creature(
            "Karona, False God",
            cost(&[generic(1), w(), u(), b(), r(), g()]),
            vec![CreatureType::Avatar],
            5,
            5,
        )
    }
}

/// Sliver Overlord — the five-colour Sliver toolbox: tutor, then take.
pub fn sliver_overlord() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Sliver),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::GainControl {
                    what: target_filtered(R::HasCreatureType(CreatureType::Sliver)),
                    to: None,
                    duration: Duration::Permanent,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Sliver Overlord",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Sliver, CreatureType::Mutant],
            7,
            7,
        )
    }
}

/// Uncontrolled Infestation — a one-shot answer that waits for the tap.
pub fn uncontrolled_infestation() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
            effect: Effect::Destroy { what: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        ..aura("Uncontrolled Infestation", cost(&[generic(1), r()]), R::IsNonbasicLand)
    }
}
