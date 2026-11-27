//! EDHREC single-commander average decks short only their commander (scan of
//! 2026-11-27) and the two-card gaps' closers: Itazura, Miss Highwater, Sash
//! and Waistcoat, Glava, Matoc, The Madcap Jester, Chira. Tests in
//! `tests/recent_b/cmdr_nov26.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{once_each_turn_on_take, target_filtered};
use crate::effect::{Duration, Effect, KeywordAct, PlayerRef, Value, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, u};

/// Itazura, Lingering Wick — {2}{U}{R} legendary 1/5 Spirit. At the beginning
/// of your upkeep, exile the top three cards of your library; each opponent
/// secretly bids a number, the highest takes that much damage and may cast one
/// of them free, and you put one that wasn't cast into your hand.
pub fn itazura_lingering_wick() -> CardDefinition {
    CardDefinition {
        name: "Itazura, Lingering Wick",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::You, count: Value::Const(3) },
                    to: ZoneDest::Exile,
                },
                Effect::OpponentsBidDamageForFreeCast { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
        }],
        ..Default::default()
    }
}

/// Miss Highwater — {2}{B}{B} legendary 5/3 Demon Advisor with menace.
/// Combat damage to a player without a contract counter offers them a wheel
/// (discard their hand, draw seven) for a contract counter: while they hold
/// it, when they lose the game, you create a token copy of each artifact and
/// creature they controlled (`GameState::queue_contract_copies`).
pub fn miss_highwater() -> CardDefinition {
    let them = || Selector::Player(PlayerRef::TriggerEventPlayer);
    CardDefinition {
        name: "Miss Highwater",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon, CreatureType::Advisor], ..Default::default() },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).with_filter(
                Predicate::ValueEquals(Value::ContractCountersOf(PlayerRef::TriggerEventPlayer), Value::Const(0)),
            ),
            effect: Effect::MayDoBy {
                who: PlayerRef::TriggerEventPlayer,
                description: "Discard your hand and draw seven cards? (You get a contract counter.)".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: them(), amount: Value::Const(100), random: false },
                    Effect::Draw { who: them(), amount: Value::Const(7) },
                    Effect::GiveContractCounter {
                        who: PlayerRef::TriggerEventPlayer,
                        beneficiary: PlayerRef::ControllerOf(Box::new(Selector::This)),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Sash and Waistcoat, Unmen — {1}{U} legendary 2/2 Insect Rogue. Whenever a
/// face-down creature attacks one of your opponents, it can't be blocked this
/// combat. {3}: manifest the top card of your library — any player may
/// activate it, as a sorcery (the activator is "you").
pub fn sash_and_waistcoat_unmen() -> CardDefinition {
    CardDefinition {
        name: "Sash and Waistcoat, Unmen",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect, CreatureType::Rogue], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::FaceDown.and(R::IsAttackingOpponentPlayer),
            }),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            sorcery_speed: true,
            effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Write into Being — {2}{U} sorcery. Look at the top two cards of your
/// library; manifest one, put the other on the top or bottom.
pub fn write_into_being() -> CardDefinition {
    CardDefinition {
        name: "Write into Being",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ManifestOneOfTopTwoRestTopOrBottom { who: PlayerRef::You },
        ..Default::default()
    }
}

/// Glava, Five-Advents Mage — {5}{G}{G} legendary 5/5 Human Wizard. Whenever
/// you cast a permanent spell with {X} in its mana cost or activate a non-mana
/// ability with {X} in its cost, you may have X become 5; do this only once
/// each turn (spent where the may is taken, `once_each_turn_on_take`).
pub fn glava_five_advents_mage() -> CardDefinition {
    let five = |event: EventSpec| {
        once_each_turn_on_take(TriggeredAbility {
            event,
            effect: Effect::MayDo {
                description: "Have the value of X become 5?".into(),
                body: Box::new(Effect::RaiseXOfTriggering { to: 5 }),
            },
        })
    };
    CardDefinition {
        name: "Glava, Five-Advents Mage",
        cost: cost(&[generic(5), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![
            five(EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellHasX,
                Predicate::CastSpellMatches(R::PermanentCard),
            ]))),
            five(EventSpec::new(EventKind::AbilityActivated, EventScope::YourControl).with_filter(Predicate::ActivationCostHasX)),
        ],
        ..Default::default()
    }
}

/// Savage Summoning — {G} instant; can't be countered. Your next creature
/// spell this turn has flash, can't be countered, and enters with an
/// additional +1/+1 counter.
pub fn savage_summoning() -> CardDefinition {
    CardDefinition {
        name: "Savage Summoning",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCountered],
        effect: Effect::Seq(vec![
            Effect::NextCreatureSpellHasFlashThisTurn,
            Effect::NextSpellCantBeCountered { filter: R::Creature },
            Effect::GrantNextCreatureSpellCounters { kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Matoc, Lavamancer — {3}{R} legendary 2/3 Human Barbarian Wizard. Whenever a
/// source you control deals damage, if a permanent or player was dealt exactly
/// 2 of it, exile your top card; you may play it until the end of your next
/// turn. {R}, {T}, exile two cards from your graveyard: 2 damage to any other
/// target.
pub fn matoc_lavamancer() -> CardDefinition {
    CardDefinition {
        name: "Matoc, Lavamancer",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Barbarian, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::YourControl)
                .with_filter(Predicate::ValueEquals(Value::TriggerEventAmount, Value::Const(2))),
            effect: Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::ONE,
                duration: MayPlayDuration::EndOfControllersNextTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            exile_other_filter: Some((R::Any, 2)),
            effect: Effect::DealDamage {
                to: target_filtered(R::any_target().and(R::OtherThanSource)),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Leyline of Combustion — {2}{R}{R} enchantment; may start the game in play.
/// Whenever you or a permanent you control becomes the target of a spell or
/// ability an opponent controls, it deals 2 damage to that player (once per
/// such target).
pub fn leyline_of_combustion() -> CardDefinition {
    CardDefinition {
        name: "Leyline of Combustion",
        opening_hand: Some(crate::effect::OpeningHandEffect::StartInPlay { tapped: false, extra: Effect::Noop }),
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::YouOrYourPermanentTargetedByOpponent),
            effect: Effect::DealDamage { to: Selector::Player(PlayerRef::TriggerEventPlayer), amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// The Madcap Jester — {1}{U}{B}{R} legendary 3/2 Clown Wizard. Whenever it
/// attacks, each opponent exiles their top card. Whenever a spell or ability
/// you control exiles cards from an opponent's library, you may play them
/// until the end of your next turn, spending mana as any type
/// (`exile_from::OPPONENT_LIBRARY_BY_YOU`).
pub fn the_madcap_jester() -> CardDefinition {
    CardDefinition {
        name: "The Madcap Jester",
        cost: cost(&[generic(1), u(), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Clown, CreatureType::Wizard], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::TopOfLibrary { who: PlayerRef::EachOpponent, count: Value::ONE },
                    to: ZoneDest::Exile,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CardExiledFrom(crate::effect::exile_from::OPPONENT_LIBRARY_BY_YOU),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::You)),
                effect: Effect::GrantMayPlay {
                    what: Selector::TriggerSource,
                    duration: MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: true,
                },
            },
        ],
        ..Default::default()
    }
}

/// Bitter Ordeal — {2}{B} sorcery with gravestorm. Search target player's
/// library for a card and exile it; then they shuffle.
pub fn bitter_ordeal() -> CardDefinition {
    CardDefinition {
        name: "Bitter Ordeal",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Gravestorm],
        effect: Effect::Search { who: PlayerRef::Target(0), filter: R::Any, to: ZoneDest::Exile },
        ..Default::default()
    }
}

/// Chira, All In — {3}{R} legendary 3/3 Human Rogue. Whenever you discard at
/// random, choose modes at random, flip coins or roll dice, put a +1/+1
/// counter on Chira and create a tapped Treasure (`KeywordAct::DidAtRandom`:
/// once per kind of act per resolution).
pub fn chira_all_in() -> CardDefinition {
    CardDefinition {
        name: "Chira, All In",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rogue], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(KeywordAct::DidAtRandom), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: std::sync::Arc::new(TokenDefinition {
                        tapped: true,
                        ..crabomination_base::tokens::treasure_token()
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Tibalt, the Fiend-Blooded — {R}{R} legendary planeswalker, loyalty 2.
/// +1: draw a card, then discard a card at random. −4: damage to target player
/// equal to the cards in their hand. −6: gain control of all creatures until
/// end of turn, untap them, they gain haste.
pub fn tibalt_the_fiend_blooded() -> CardDefinition {
    let all = || Selector::EachPermanent(R::Creature);
    CardDefinition {
        name: "Tibalt, the Fiend-Blooded",
        cost: cost(&[r(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tibalt], ..Default::default() },
        base_loyalty: 2,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: true },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -4,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player),
                    amount: Value::HandSizeOf(PlayerRef::Target(0)),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::Seq(vec![
                    Effect::GainControl { what: all(), to: None, duration: Duration::EndOfTurn },
                    Effect::Untap { what: all(), up_to: None },
                    Effect::GrantKeyword { what: all(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
