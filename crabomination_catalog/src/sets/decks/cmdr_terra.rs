//! Commander: the cards the **Revival Trance** precon (FIC, Final Fantasy VI,
//! Terra, Herald of Hope) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_terra.rs`.
//!
//! Residuals (each also on its card):
//! - **Edgar, Master Machinist** — an artifact cast from the graveyard this
//!   way enters untapped.
//! - **Espers to Magicite** — the copied card is the first creature card
//!   exiled, not a chosen target; the artifact-only type is a layer-4 set,
//!   not a copiable value.
//! - **General Leo Cristophe** — the "up to one" return target is required
//!   when one exists.
//! - **Gogo, Mysterious Mime** — the copy takes the copied creature's name.
//! - **Legions to Ashes** — same-name tokens of every player are exiled, not
//!   only the target's controller's.
//! - **Locke, Treasure Hunter** — any of the milled cards may be played this
//!   turn, not just one spell.
//! - **Summon: Esper Valigarmanda** — chapter I exiles the first instant or
//!   sorcery card of each graveyard, not a chosen one.
//! - **The Warring Triad** — the mill is part of the effect, not a cost, and
//!   you are always the player who adds the mana.
//! - **Umaro, Raging Yeti** — the random mode is picked, and a damage target
//!   chosen, as the trigger resolves.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Predicate, RevealMissDest, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, w};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        supertypes: vec![Supertype::Legendary],
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn begin_combat_on_your_turn(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
        effect,
    }
}

fn your_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl), effect }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn mill(who: PlayerRef, n: i32) -> Effect {
    Effect::Mill { who: Selector::Player(who), amount: Value::Const(n) }
}

fn to_battlefield(tapped: bool) -> ZoneDest {
    ZoneDest::Battlefield { controller: PlayerRef::You, tapped }
}

fn treasures(n: i32, tapped: bool) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: Arc::new(TokenDefinition { tapped, ..crabomination_base::tokens::treasure_token() }),
    }
}

/// Terra, Herald of Hope — each combat mill two and fly; a hit may pay {2} to
/// bring back a creature with power 3 or less.
pub fn terra_herald_of_hope() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            begin_combat_on_your_turn(Effect::Seq(vec![
                mill(PlayerRef::You, 2),
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Flying, duration: Duration::EndOfTurn },
            ])),
            on_combat_damage_to_player(Effect::MayPay {
                description: "Pay {2} to return a creature card with power 3 or less?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Reflexive {
                    body: Box::new(Effect::Move {
                        what: target_filtered(R::Creature.and(R::PowerAtMost(3)).and(R::InYourGraveyard)),
                        to: to_battlefield(true),
                    }),
                }),
                else_: None,
            }),
        ],
        ..creature(
            "Terra, Herald of Hope",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Banon, the Returners' Leader — pray: once a turn, a creature put into your
/// graveyard this turn from anywhere but the battlefield is castable; each
/// attack may rummage for {1}.
pub fn banon_the_returners_leader() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Pray — Once during each of your turns, you may cast a creature spell from among cards in your graveyard that were put there from anywhere other than the battlefield this turn.",
            effect: StaticEffect::GraveyardCastOncePerTurn {
                mv_at_most_counters: None,
                filter: R::Creature
                    .and(R::PutIntoGraveyardThisTurn)
                    .and(R::Not(Box::new(R::PutIntoGraveyardFromBattlefieldThisTurn))),
                exile_after: false,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1} and discard a card to draw a card?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    draw(Value::ONE),
                ])),
                else_: None,
            },
        }],
        ..creature(
            "Banon, the Returners' Leader",
            cost(&[r(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Rebel],
            1,
            3,
        )
    }
}

/// Celes, Rune Knight — discard any number, draw that many plus one; your
/// team grows whenever others arrive from a graveyard.
pub fn celes_rune_knight() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::DiscardAnyNumber { who: Selector::You, filter: R::Any, max: None },
                draw(Value::Sum(vec![Value::CardsDiscardedThisEffect, Value::ONE])),
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::OtherThanSource).and(R::EnteredFromGraveyardThisTurn),
                    })
                    .once_per_batch(),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(yours(R::Creature)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature(
            "Celes, Rune Knight",
            cost(&[generic(1), r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Coin of Fate — surveil 1; later, exile two creature cards: an opponent
/// sends one to the bottom, the other returns; you become the monarch.
pub fn coin_of_fate() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Surveil { who: PlayerRef::You, amount: Value::ONE })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            tap_cost: true,
            sac_cost: true,
            exile_other_filter: Some((R::Creature, 2)),
            effect: Effect::Seq(vec![
                Effect::ChooseOneAmong {
                    what: Selector::CostExiledCards,
                    chooser: PlayerRef::OpponentOf(Box::new(PlayerRef::You)),
                    chosen: Box::new(Effect::Move {
                        what: Selector::SeparatedPile { chosen: true },
                        to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Bottom },
                    }),
                    other: Box::new(Effect::Move {
                        what: Selector::SeparatedPile { chosen: false },
                        to: to_battlefield(true),
                    }),
                },
                Effect::BecomeMonarch { who: PlayerRef::You },
            ]),
            ..Default::default()
        }],
        ..spell("Coin of Fate", cost(&[generic(1), w()]), CardType::Artifact, Effect::Noop)
    }
}

/// Cyan, Vengeful Samurai — cheaper per creature card in your graveyard;
/// double strike; grows as creature cards leave it.
pub fn cyan_vengeful_samurai() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature card in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerCreatureInGraveyard,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature })
                .once_per_batch(),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..creature(
            "Cyan, Vengeful Samurai",
            cost(&[generic(6), w()]),
            vec![CreatureType::Human, CreatureType::Samurai],
            3,
            3,
        )
    }
}

/// Edgar, Master Machinist — an artifact from your graveyard each turn;
/// attacks with +X/+0 for your biggest artifact.
/// Residual: the recast artifact enters untapped.
pub fn edgar_master_machinist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast an artifact spell from your graveyard.",
            effect: StaticEffect::GraveyardCastOncePerTurn { mv_at_most_counters: None, filter: R::Artifact, exile_after: false },
        }],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::This,
            power: Value::ManaValueOf(Box::new(Selector::GreatestManaValueControlledMatching {
                who: PlayerRef::You,
                filter: R::Artifact,
            })),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Edgar, Master Machinist",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Human, CreatureType::Artificer, CreatureType::Noble],
            2,
            4,
        )
    }
}

/// Espers to Magicite — exile each opponent's graveyard and copy a creature
/// card from it as a noncreature artifact token.
/// Residual: the copied card is the first creature card exiled.
pub fn espers_to_magicite() -> CardDefinition {
    spell(
        "Espers to Magicite",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ExilePlayerGraveyard { who: PlayerRef::EachOpponent, filter: None },
            Effect::CreateTokenCopyOf {
                who: PlayerRef::You,
                count: Value::ONE,
                source: Selector::Take {
                    inner: Box::new(Selector::ExiledThisResolution { filter: R::Creature }),
                    count: Box::new(Value::ONE),
                },
                extra_creature_types: vec![],
                extra_card_types: vec![],
                override_pt: None,
                override_colors: None,
                enters_tapped: false,
                non_legendary: false,
                legendary: false,
                extra_keywords: vec![],
            },
            Effect::SetCardTypesTo { what: Selector::LastCreatedTokens, card_types: vec![CardType::Artifact] },
        ]),
    )
}

/// Gau, Feral Youth — grows as it attacks; each end step after a card left
/// your graveyard, deals its power to each opponent.
pub fn gau_feral_youth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                    Predicate::CardsLeftGraveyardThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
                ),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            },
        ],
        ..creature(
            "Gau, Feral Youth",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Berserker],
            2,
            2,
        )
    }
}

/// General Leo Cristophe — returns a small creature, then a counter per
/// creature you control. Residual: the return target is required.
pub fn general_leo_cristophe() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).and(R::InYourGraveyard)),
                to: to_battlefield(false),
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(yours(R::Creature))),
            },
        ]))],
        ..creature(
            "General Leo Cristophe",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Gogo, Mysterious Mime — each combat may mimic another creature of yours;
/// both get +2/+0 and haste and must attack. Residual: the name changes too.
pub fn gogo_mysterious_mime() -> CardDefinition {
    let both = |e: fn(Selector) -> Effect| vec![e(Selector::This), e(Selector::Target(0))];
    let mut body = vec![Effect::BecomeCopyOfFor {
        what: Selector::This,
        source: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
        duration: Duration::EndOfTurn,
        non_legendary: false,
    }];
    body.extend(both(|s| Effect::PumpPT {
        what: s,
        power: Value::Const(2),
        toughness: Value::Const(0),
        duration: Duration::EndOfTurn,
    }));
    body.extend(both(|s| Effect::GrantKeyword { what: s, keyword: Keyword::Haste, duration: Duration::EndOfTurn }));
    body.extend(both(|s| Effect::GrantKeyword { what: s, keyword: Keyword::MustAttack, duration: Duration::EndOfTurn }));
    CardDefinition {
        triggered_abilities: vec![begin_combat_on_your_turn(Effect::MayDo {
            description: "Have Gogo become a copy of another creature you control?".into(),
            body: Box::new(Effect::Seq(body)),
        })],
        ..creature("Gogo, Mysterious Mime", cost(&[generic(3), r()]), vec![CreatureType::Wizard], 2, 2)
    }
}

/// Interceptor, Shadow's Hound — your Assassins have menace; it returns
/// attacking when you attack with a legendary creature.
pub fn interceptor_shadows_hound() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Assassins you control have menace.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Assassin))),
                keyword: Keyword::Menace,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::FromYourGraveyard).with_filter(
                Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: R::HasSupertype(Supertype::Legendary) },
            ),
            effect: Effect::MayPay {
                description: "Pay {2}{B} to return Interceptor tapped and attacking?".into(),
                mana_cost: cost(&[generic(2), b()]),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: Selector::This, to: to_battlefield(true) },
                    Effect::JoinCombatAttacking { what: Selector::This },
                ])),
                else_: None,
            },
        }],
        ..creature(
            "Interceptor, Shadow's Hound",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Dog],
            4,
            3,
        )
    }
}

/// Kefka, Dancing Mad — indestructible on your turn; each end step it steals
/// a random card from each opponent's graveyard to cast free, billing the
/// owner its mana value.
pub fn kefka_dancing_mad() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, Kefka has indestructible.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Indestructible,
                condition: Predicate::IsTurnOf(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![your_end_step(Effect::Seq(vec![
            Effect::ForEachOpponent {
                body: Box::new(Effect::Move {
                    what: Selector::TakeRandom {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::Triggerer,
                            zone: Zone::Graveyard,
                            filter: R::Any,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Exile,
                }),
            },
            Effect::CastExiledFreeOwnersLoseLife { what: Selector::ExiledThisResolution { filter: R::Any } },
        ]))],
        ..creature(
            "Kefka, Dancing Mad",
            cost(&[generic(5), b(), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            6,
            6,
        )
    }
}

/// Legions to Ashes — exile an opposing nonland permanent and its same-named
/// tokens. Residual: every player's same-named tokens go.
pub fn legions_to_ashes() -> CardDefinition {
    spell(
        "Legions to Ashes",
        cost(&[generic(1), w(), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ExileTokensSharingNameWith {
                what: target_filtered(R::Permanent.and(R::Nonland).and(R::ControlledByOpponent)),
            },
            Effect::Exile { what: Selector::Target(0) },
        ]),
    )
}

/// Locke, Treasure Hunter — skulk-like; each attack mills everyone, a milled
/// land makes a Treasure, and the milled cards are castable this turn.
/// Residual: any of them, not only one spell.
pub fn locke_treasure_hunter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Skulk],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            mill(PlayerRef::EachPlayer, 1),
            Effect::If {
                cond: Predicate::ValueAtLeast(Value::CardsMilledThisEffectMatching { filter: R::Land }, Value::ONE),
                then: Box::new(treasures(1, false)),
                else_: Box::new(Effect::Noop),
            },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ]))],
        ..creature(
            "Locke, Treasure Hunter",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            3,
        )
    }
}

fn moogle() -> TokenDefinition {
    TokenDefinition {
        name: "Moogle".into(),
        power: 1,
        toughness: 2,
        colors: vec![Color::White],
        keywords: vec![Keyword::Lifelink],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moogle], ..Default::default() },
        ..Default::default()
    }
}

/// Mog, Moogle Warrior — each end step everyone may rummage; a creature
/// discarded makes a Moogle, a noncreature grows your Moogles.
pub fn mog_moogle_warrior() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![your_end_step(Effect::Seq(vec![
            Effect::EachPlayerDoes {
                who: PlayerRef::EachPlayer,
                body: Box::new(Effect::MayDiscard {
                    description: "Discard a card to draw a card?".into(),
                    count: Value::ONE,
                    then: Box::new(draw(Value::ONE)),
                    else_: None,
                }),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: R::Creature }),
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(moogle()) }),
                else_: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DiscardedThisResolution { filter: R::Noncreature }),
                then: Box::new(Effect::AddCounter {
                    what: Selector::EachPermanent(yours(R::HasCreatureType(CreatureType::Moogle))),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature(
            "Mog, Moogle Warrior",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Moogle, CreatureType::Warrior],
            1,
            2,
        )
    }
}

/// Rejoin the Fight — mill three; each opponent hands back a creature card.
pub fn rejoin_the_fight() -> CardDefinition {
    spell(
        "Rejoin the Fight",
        cost(&[generic(5), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            mill(PlayerRef::You, 3),
            Effect::EachOpponentReturnsFromYourGraveyard { filter: R::Creature },
        ]),
    )
}

/// Sabin, Master Monk — double strike; blitz {2}{R}{R} and a discard, from
/// hand or graveyard.
pub fn sabin_master_monk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), r(), r()]),
            blitz: true,
            also_from_graveyard: true,
            discard_filters: vec![(R::Any, 1)],
            ..Default::default()
        }),
        ..CardDefinition {
            supertypes: vec![Supertype::Legendary],
            ..creature(
                "Sabin, Master Monk",
                cost(&[generic(4), r()]),
                vec![CreatureType::Human, CreatureType::Noble, CreatureType::Monk],
                4,
                3,
            )
        }
    }
}

fn blackjack() -> TokenDefinition {
    TokenDefinition {
        name: "The Blackjack".into(),
        power: 3,
        toughness: 3,
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        ..Default::default()
    }
}

/// Setzer, Wandering Gambler — brings The Blackjack; Vehicles hitting a
/// player flip a coin; each won flip makes two tapped Treasures.
pub fn setzer_wandering_gambler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(blackjack()) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                    },
                ),
                effect: Effect::FlipCoin { count: Value::ONE, on_heads: Box::new(Effect::Noop), on_tails: Box::new(Effect::Noop) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::WonCoinFlip, EventScope::YourControl),
                effect: treasures(2, true),
            },
        ],
        ..creature(
            "Setzer, Wandering Gambler",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Pilot],
            2,
            2,
        )
    }
}

/// Shadow, Mysterious Assassin — deathtouch; a hit may sacrifice a nonland
/// permanent to draw two and drain its mana value.
pub fn shadow_mysterious_assassin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::MaySacrifice {
            description: "Sacrifice another nonland permanent to draw two?".into(),
            filter: R::Permanent.and(R::Nonland).and(R::OtherThanSource),
            count: Value::ONE,
            then: Box::new(Effect::Seq(vec![
                draw(Value::Const(2)),
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::SacrificedManaValue },
            ])),
            else_: None,
        })],
        ..creature(
            "Shadow, Mysterious Assassin",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            3,
            3,
        )
    }
}

/// Siegfried, Famed Swordsman — mills three, then two counters per creature
/// card in your graveyard.
pub fn siegfried_famed_swordsman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            mill(PlayerRef::You, 3),
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature }),
                ),
            },
        ]))],
        ..creature(
            "Siegfried, Famed Swordsman",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Snort — each player may wheel into five; 5 damage to each opponent who
/// did. Flashback {5}{R}.
pub fn snort() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(5), r()]))],
        ..spell(
            "Snort",
            cost(&[generic(3), r()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::EachPlayerDoes {
                    who: PlayerRef::EachPlayer,
                    body: Box::new(Effect::MayDo {
                        description: "Discard your hand and draw five cards?".into(),
                        body: Box::new(Effect::Seq(vec![
                            Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                            draw(Value::Const(5)),
                        ])),
                    }),
                },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::If {
                        cond: Predicate::DiscardedThisEffect { who: PlayerRef::Triggerer },
                        then: Box::new(Effect::DealDamage {
                            to: Selector::Player(PlayerRef::Triggerer),
                            amount: Value::Const(5),
                        }),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        )
    }
}

/// Strago and Relm — an opponent digs to a spell or creature you may cast
/// free; a creature cast this way has haste and is sacrificed at end step.
pub fn strago_and_relm() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::Seq(vec![
                    Effect::RevealUntilFind {
                        who: PlayerRef::Target(0),
                        find: instant_or_sorcery().or(R::Creature),
                        to: ZoneDest::Exile,
                        cap: Value::Const(500),
                        life_per_revealed: 0,
                        miss_dest: RevealMissDest::Exile,
                    },
                    Effect::CastWithoutPayingImmediate {
                        what: Selector::LastMoved,
                        source_zone: Zone::Exile,
                        exile_after: false,
                        copy: false,
                        reduce_generic: 0,
                        pay_own_cost: false,
                    },
                    Effect::GrantCastSpellRiders { what: Selector::LastMoved, haste: true, sacrifice_eot: true },
                ])),
            },
            ..Default::default()
        }],
        ..creature("Strago and Relm", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Wizard], 1, 3)
    }
}

/// Summon: Esper Valigarmanda — a Saga Drake: exile an instant or sorcery from
/// each graveyard, then {R} per lore counter and those cards castable with
/// any mana. Residual: chapter I takes the first such card.
pub fn summon_esper_valigarmanda() -> CardDefinition {
    let spells_and_mana = || {
        Effect::Seq(vec![
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Red,
                    Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Lore },
                ),
            },
            Effect::GrantMayPlay {
                what: Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: Zone::Exile,
                    filter: R::ExiledWithSource.and(instant_or_sorcery()),
                },
                duration: crate::card::MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: true,
            },
        ])
    };
    CardDefinition {
        name: "Summon: Esper Valigarmanda",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Saga],
            creature_types: vec![CreatureType::Drake],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        saga_chapters: vec![
            (
                1,
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachPlayer),
                    body: Box::new(Effect::Move {
                        what: Selector::Take {
                            inner: Box::new(Selector::CardsInZone {
                                who: PlayerRef::Triggerer,
                                zone: Zone::Graveyard,
                                filter: instant_or_sorcery(),
                            }),
                            count: Box::new(Value::ONE),
                        },
                        to: ZoneDest::ExileWithSourceStamp,
                    }),
                },
            ),
            (2, spells_and_mana()),
            (3, spells_and_mana()),
            (4, spells_and_mana()),
        ],
        ..Default::default()
    }
}

/// The Warring Triad — a 5/5 flier only with eight cards in your graveyard;
/// taps and mills for any color. Residual: the mill is in the effect, and
/// the mana is yours.
pub fn the_warring_triad() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "As long as there are fewer than eight cards in your graveyard, The Warring Triad isn't a creature.",
            effect: StaticEffect::NotCreatureUnless {
                condition: Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Any },
                    Value::Const(8),
                ),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                mill(PlayerRef::You, 1),
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) },
            ]),
            ..Default::default()
        }],
        ..creature("The Warring Triad", cost(&[generic(3)]), vec![CreatureType::God], 5, 5)
    }
}

/// The Falcon, Airship Restored — a flying Vehicle that trades itself for a
/// creature card on a hit, and flies back from the graveyard for {4}{B}.
pub fn the_falcon_airship_restored() -> CardDefinition {
    CardDefinition {
        name: "The Falcon, Airship Restored",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::MaySacrificeSource {
            description: "Sacrifice The Falcon to return a creature card?".into(),
            then: Box::new(Effect::Reflexive {
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                    to: to_battlefield(false),
                }),
            }),
            else_: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            from_graveyard: true,
            effect: Effect::Move { what: Selector::This, to: to_battlefield(true) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Umaro, Raging Yeti — each combat one of three at random: a team pump,
/// a four-card wheel, or 5 damage. Residual: picked as it resolves.
pub fn umaro_raging_yeti() -> CardDefinition {
    let others = || Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource));
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![begin_combat_on_your_turn(Effect::ChooseModeAtRandom(vec![
            Effect::Seq(vec![
                Effect::PumpPT { what: others(), power: Value::Const(3), toughness: Value::Const(0), duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: others(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
            Effect::Seq(vec![
                Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
                draw(Value::Const(4)),
            ]),
            Effect::DealDamage { to: target_any(), amount: Value::Const(5) },
        ]))],
        ..creature(
            "Umaro, Raging Yeti",
            cost(&[generic(5), r()]),
            vec![CreatureType::Yeti, CreatureType::Berserker],
            6,
            6,
        )
    }
}
