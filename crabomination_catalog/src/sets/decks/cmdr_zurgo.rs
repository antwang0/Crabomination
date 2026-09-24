//! Commander: the cards the **Mardu Surge** precon (TDC, Zurgo Stormrender)
//! needed beyond what the catalog had. Tests in `tests/recent_b/cmdr_zurgo.rs`.
//!
//! Residuals (each also on its card):
//! - **Gix, Yawgmoth Praetor** — the exiled cards stay playable for free for
//!   the rest of the turn, not only as the ability resolves.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, MayPlayDuration, PlaneswalkerSubtype,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, mobilize, mobilize_value, myriad, on_attack, on_you_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate};
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
        ..Default::default()
    }
}

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn token(name: &str, color: Color, p: i32, t: i32, kind: CreatureType, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: vec![kind], ..Default::default() },
        ..Default::default()
    }
}

fn warrior() -> Arc<TokenDefinition> {
    Arc::new(token("Warrior", Color::Red, 1, 1, CreatureType::Warrior, vec![]))
}

fn goblin() -> Arc<TokenDefinition> {
    Arc::new(token("Goblin", Color::Red, 1, 1, CreatureType::Goblin, vec![]))
}

fn spirit(tapped: bool) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        tapped,
        ..token("Spirit", Color::White, 1, 1, CreatureType::Spirit, vec![Keyword::Flying])
    })
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(count: Value, definition: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count, definition }
}

/// Zurgo Stormrender — mobilize 1; a creature token of yours leaving draws a
/// card if it was attacking, else drains each opponent for 1. Two triggers:
/// "if it was attacking" is last-known information (CR 603.10), so each
/// half's filter reads the leave event rather than the resolution.
pub fn zurgo_stormrender() -> CardDefinition {
    let token_left = |attacking: R, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::YourControl).with_filter(
            Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::IsToken).and(attacking) },
        ),
        effect,
    };
    CardDefinition {
        triggered_abilities: vec![
            mobilize(1),
            token_left(R::IsAttacking, Effect::Draw { who: Selector::You, amount: Value::ONE }),
            token_left(
                R::Not(Box::new(R::IsAttacking)),
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
            ),
        ],
        ..legendary(creature(
            "Zurgo Stormrender",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Orc, CreatureType::Warrior],
            3,
            3,
        ))
    }
}

/// Ainok Strike Leader — when you attack with it and/or your commander, a
/// Goblin tapped and attacking each opponent; sacrifice it: your creature
/// tokens gain indestructible.
pub fn ainok_strike_leader() -> CardDefinition {
    let leads = R::IsAttacking.and(R::IsSource.or(R::IsCommander.and(R::OwnedByYou)));
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource).with_filter(
                Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(leads), n: Value::ONE },
            ),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: goblin(),
                    cleanup: Default::default(),
                    defender: Some(PlayerRef::Triggerer),
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: yours(R::Creature.and(R::IsToken)),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Ainok Strike Leader",
            cost(&[generic(1), w()]),
            vec![CreatureType::Dog, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Aron, Benalia's Ruin — menace; {W}{B}, {T}, sacrifice another creature: a
/// +1/+1 counter on each creature you control.
pub fn aron_benalias_ruin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), b()]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: yours(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Aron, Benalia's Ruin",
            cost(&[w(), w(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Human],
            3,
            3,
        ))
    }
}

/// Bone Devourer — flash, flying; enters with a counter per creature that
/// died this turn; dying draws X and loses X for its counters.
pub fn bone_devourer() -> CardDefinition {
    let x = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne };
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CreaturesDiedThisTurnTotal)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: x() },
                Effect::LoseLife { who: Selector::You, amount: x() },
            ]),
        }],
        ..creature("Bone Devourer", cost(&[generic(3), b()]), vec![CreatureType::Dragon], 2, 2)
    }
}

/// Divine Visitation — your creature tokens are created as 4/4 flying,
/// vigilance Angels instead.
pub fn divine_visitation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If one or more creature tokens would be created under your control, that many 4/4 white Angel creature tokens with flying and vigilance are created instead.",
            effect: StaticEffect::CreatureTokensBecome {
                into: token("Angel", Color::White, 4, 4, CreatureType::Angel, vec![Keyword::Flying, Keyword::Vigilance]),
            },
        }],
        ..spell("Divine Visitation", cost(&[generic(3), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Eliminate the Competition — sacrifice X creatures, destroy X creatures.
pub fn eliminate_the_competition() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificeAnyNumber { filter: R::Creature }],
        ..spell(
            "Eliminate the Competition",
            cost(&[generic(4), b()]),
            CardType::Sorcery,
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                }),
            },
        )
    }
}

/// Gix, Yawgmoth Praetor — any creature connecting with one of your
/// opponents lets its controller pay 1 life to draw; {4}{B}{B}{B}, discard X:
/// exile an opponent's top X, free to play. ⚠ Playable for the rest of the
/// turn rather than only as the ability resolves.
pub fn gix_yawgmoth_praetor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .dealt_by(R::Creature)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::Triggerer,
                body: Box::new(Effect::MayPayLife {
                    description: "Pay 1 life to draw a card?".into(),
                    amount: Value::ONE,
                    body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: None,
                }),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b(), b(), b()]),
            discard_cost: Some((R::Any, 0)),
            discard_cost_x: true,
            effect: Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::Target(0),
                    count: Value::XFromCost,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: false,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                }),
            },
            ..Default::default()
        }],
        ..legendary(creature(
            "Gix, Yawgmoth Praetor",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Praetor],
            3,
            3,
        ))
    }
}

/// Infantry Shield — equipped creature has menace and mobilize X, X its
/// power. Equip {2}.
pub fn infantry_shield() -> CardDefinition {
    CardDefinition {
        name: "Infantry Shield",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Menace],
            triggered_abilities: vec![mobilize_value(Value::PowerOf(Box::new(Selector::This)))],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ironwill Forger — lieutenant: at the beginning of your combat, with your
/// commander out, a nonlegendary creature of yours gains myriad.
pub fn ironwill_forger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl)
                .with_filter(Predicate::ControlsOwnCommander { who: PlayerRef::You }),
            effect: Effect::GrantTriggeredAbility {
                what: target_filtered(
                    R::Creature.and(R::ControlledByYou).and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
                ),
                trigger: Box::new(myriad()),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Ironwill Forger",
            cost(&[generic(3), w()]),
            vec![CreatureType::Orc, CreatureType::Artificer],
            3,
            3,
        )
    }
}

/// Kaya, Geist Hunter — +1 deathtouch and a counter on a token; −2 doubles
/// your tokens this turn; −6 exiles every graveyard for a Spirit apiece.
pub fn kaya_geist_hunter() -> CardDefinition {
    let ability = |loyalty_cost: i32, effect: Effect| LoyaltyAbility { x_cost: false, loyalty_cost, effect };
    CardDefinition {
        name: "Kaya, Geist Hunter",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Kaya], ..Default::default() },
        base_loyalty: 3,
        loyalty_abilities: vec![
            ability(
                1,
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::Seq(vec![
                        Effect::GrantKeyword {
                            what: yours(R::Creature),
                            keyword: Keyword::Deathtouch,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::AddCounter {
                            what: target_filtered(R::Creature.and(R::IsToken).and(R::ControlledByYou)),
                            kind: CounterType::PlusOnePlusOne,
                            amount: Value::ONE,
                        },
                    ])),
                },
            ),
            ability(-2, Effect::DoubleTokensThisTurn { who: PlayerRef::You }),
            ability(
                -6,
                Effect::Seq(vec![
                    make(Value::CardsInAllGraveyardsMatching { filter: R::Any }, spirit(false)),
                    Effect::ExileAllGraveyards { filter: None, opponents_only: false },
                ]),
            ),
        ],
        ..Default::default()
    }
}

/// Legion Loyalty — creatures you control have myriad.
pub fn legion_loyalty() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have myriad.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou),
                ability: Box::new(myriad()),
            },
        }],
        ..spell("Legion Loyalty", cost(&[generic(6), w(), w()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Mindblade Render — once per combat-damage step, if a Warrior dealt your
/// opponents combat damage, you draw and lose 1 life.
pub fn mindblade_render() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer)
                .dealt_by(R::HasCreatureType(CreatureType::Warrior))
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer })
                .once_per_batch_across_players(),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
            ]),
        }],
        ..creature(
            "Mindblade Render",
            cost(&[generic(1), b()]),
            vec![CreatureType::Azra, CreatureType::Warrior],
            1,
            3,
        )
    }
}

/// Neriv, Crackling Vanguard — flying, deathtouch; enters with two Goblins;
/// attacking exiles one card per differently named token you control,
/// playable during any turn you attacked with a commander.
pub fn neriv_crackling_vanguard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![
            etb(make(Value::Const(2), goblin())),
            on_attack(Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::DistinctNamesControlledMatching(R::IsToken),
                duration: MayPlayDuration::TurnsHolderAttacksWithACommander { holder: 0 },
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            }),
        ],
        ..legendary(creature(
            "Neriv, Crackling Vanguard",
            cost(&[generic(2), r(), w(), b()]),
            vec![CreatureType::Spirit, CreatureType::Dragon],
            4,
            4,
        ))
    }
}

/// Ogre Battledriver — another creature of yours entering gets +2/+0 and
/// haste until end of turn.
pub fn ogre_battledriver() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::TriggerSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..creature(
            "Ogre Battledriver",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Ogre, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Redoubled Stormsinger — first strike; attacking copies each creature
/// token of yours that entered this turn, tapped and attacking, sacrificed at
/// the next end step.
pub fn redoubled_stormsinger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::ForEach {
                selector: yours(R::Creature.and(R::IsToken).and(R::EnteredThisTurn)),
                body: Box::new(Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    source: Selector::TriggerSource,
                    extra_creature_types: vec![],
                    extra_card_types: vec![],
                    override_pt: None,
                    override_colors: None,
                    enters_tapped: true,
                    non_legendary: false,
                    legendary: false,
                    extra_keywords: vec![],
                }),
            },
            Effect::JoinCombatAttacking { what: Selector::LastCreatedTokens },
            Effect::SacrificeLastCreatedTokensAtNextEndStep,
        ]))],
        ..creature(
            "Redoubled Stormsinger",
            cost(&[generic(2), r()]),
            vec![CreatureType::Orc, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Shadow Summoning — two tapped 1/1 flying Spirits.
pub fn shadow_summoning() -> CardDefinition {
    spell("Shadow Summoning", cost(&[w(), b()]), CardType::Sorcery, make(Value::Const(2), spirit(true)))
}

/// Thalisse, Reverent Medium — each end step, a flying Spirit per token you
/// created this turn.
pub fn thalisse_reverent_medium() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: make(Value::TokensCreatedThisTurn(PlayerRef::You), spirit(false)),
        }],
        ..legendary(creature(
            "Thalisse, Reverent Medium",
            cost(&[generic(3), w(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            4,
        ))
    }
}

/// Will of the Mardu — Warriors for a player's creature count, and/or damage
/// to a creature for yours; both with a commander out.
pub fn will_of_the_mardu() -> CardDefinition {
    let modes = || {
        vec![
            Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(make(Value::CreatureCountControlledBy(PlayerRef::Target(0)), warrior())),
            },
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::CountOf(Box::new(yours(R::Creature))),
            },
        ]
    };
    spell(
        "Will of the Mardu",
        cost(&[generic(2), w()]),
        CardType::Instant,
        Effect::If {
            cond: Predicate::YouControlACommander,
            then: Box::new(Effect::ChooseN { picks: vec![0, 1], modes: modes() }),
            else_: Box::new(Effect::ChooseMode(modes())),
        },
    )
}

/// Within Range — two Warriors on entry; whenever you attack, each opponent
/// loses life per creature attacking them.
pub fn within_range() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make(Value::Const(2), warrior())),
            on_you_attack(Effect::ForEachOpponent {
                body: Box::new(Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::CreaturesAttackingPlayer(PlayerRef::Triggerer),
                }),
            }),
        ],
        ..spell("Within Range", cost(&[generic(3), b()]), CardType::Enchantment, Effect::Noop)
    }
}
