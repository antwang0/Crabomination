//! Commander: the cards the **The Ruinous Powers** precon (40K, Abaddon the
//! Despoiler) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_abaddon.rs`.
//!
//! Residuals (each also on its card):
//! - **Bloodthirster** — it may attack a player it already attacked this
//!   turn.
//! - **Chaos Mutation** — two targets may share a controller.
//! - **Khârn the Betrayer** — the next opponent in turn order gains control,
//!   not an opponent of your choice.
//! - **The Horus Heresy** — chapter III's choices start with the next
//!   opponent, not with you.
//! - **The Lost and the Damned** — a land played from outside your hand
//!   (graveyard, exile) doesn't count.
//! - **The Ruinous Powers** — the life-loss rider reads any spell you cast
//!   from exile that you don't own, and only nonland cards can be played.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::card::ArtifactSubtype;
use crate::effect::shortcut::{cascade, cast_is_instant_or_sorcery, etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, PlayerStaticTarget, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, generic, r, u, x};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn artifact_creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Artifact, CardType::Creature], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn demon() -> R {
    R::HasCreatureType(CreatureType::Demon)
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

fn draw(n: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount: n }
}

fn lose(n: Value) -> Effect {
    Effect::LoseLife { who: Selector::You, amount: n }
}

fn make(token: TokenDefinition, n: Value) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(token) }
}

fn step(s: TurnStep, effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::StepBegins(s), EventScope::YourControl), effect }
}

fn on_combat_damage_to_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

fn on_instant_or_sorcery_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(cast_is_instant_or_sorcery()),
        effect,
    }
}

/// "The next [filter] spell you cast this turn has cascade" (CR 702.85a —
/// the cascade reads the cast spell's own mana value).
fn next_spell_cascades(filter: R) -> Effect {
    Effect::OnYourNextSpellMatchingThisTurn {
        filter,
        body: Box::new(Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)) }),
    }
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

/// A 1/3 black Demon named Plaguebearer of Nurgle.
fn plaguebearer() -> TokenDefinition {
    token("Plaguebearer of Nurgle", vec![Color::Black], vec![CreatureType::Demon], 1, 3)
}

/// A 3/3 red Spawn.
fn spawn() -> TokenDefinition {
    token("Spawn", vec![Color::Red], vec![CreatureType::Spawn], 3, 3)
}

fn saga(name: &'static str, mana: ManaCost, chapters: Vec<(u32, Effect)>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: chapters,
        ..Default::default()
    }
}

/// Abaddon the Despoiler — trample; during your turn, spells you cast from
/// your hand with mana value up to the life your opponents lost this turn
/// have cascade.
pub fn abaddon_the_despoiler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CastFromHand,
                Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    Value::TotalLifeLostThisTurn(PlayerRef::EachOpponent),
                ),
            ])),
            effect: Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)) },
        }],
        ..legend(
            "Abaddon the Despoiler",
            cost(&[generic(2), u(), b(), r()]),
            vec![CreatureType::Astartes, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Aspiring Champion — menace; its combat damage to a player sacrifices it
/// to reveal until a creature card, put onto the battlefield; a Demon deals
/// its power to each opponent.
pub fn aspiring_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::SacrificeSource,
            Effect::If {
                cond: Predicate::PlayerSacrificedThisResolution(PlayerRef::You),
                then: Box::new(Effect::Seq(vec![
                    Effect::RevealUntilMatchingToBattlefield { filter: R::Creature, count: Value::ONE, rest_bottom: false },
                    Effect::If {
                        cond: Predicate::EntityMatches { what: Selector::LastMoved, filter: demon() },
                        then: Box::new(Effect::DealDamageEqualToPowerToEach {
                            source: Selector::LastMoved,
                            targets: Selector::None,
                            each_opponent: true,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature(
            "Aspiring Champion",
            cost(&[generic(3), r()]),
            vec![CreatureType::Astartes, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Be'lakor, the Dark Master — flying; entering draws and loses X (X = your
/// Demons); another Demon of yours entering deals its power to any target.
pub fn belakor_the_dark_master() -> CardDefinition {
    let demons = || Value::CountOf(Box::new(Selector::EachPermanent(yours(demon()))));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![draw(demons()), lose(demons())])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: demon() }),
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::TriggerSource,
                    target: target_filtered(R::Any),
                },
            },
        ],
        ..legend(
            "Be'lakor, the Dark Master",
            cost(&[generic(3), u(), b(), r()]),
            vec![CreatureType::Demon, CreatureType::Noble],
            6,
            5,
        )
    }
}

/// Blight Grenade — destroy target creature; all creatures get -3/-3.
pub fn blight_grenade() -> CardDefinition {
    spell(
        "Blight Grenade",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Creature) },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature),
                power: Value::Const(-3),
                toughness: Value::Const(-3),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Blood for the Blood God! — {1} less per creature that died this turn;
/// discard your hand, draw eight, 8 damage to each opponent, exile it.
pub fn blood_for_the_blood_god() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature that died this turn.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::CreaturesDiedThisTurnTotal },
        }],
        ..spell(
            "Blood for the Blood God!",
            cost(&[generic(8), b(), b(), r()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                draw(Value::Const(8)),
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(8) },
                Effect::ExileResolvingSpell,
            ]),
        )
    }
}

/// Bloodcrusher of Khorne — trample; your other creatures have trample.
pub fn bloodcrusher_of_khorne() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature).and(R::OtherThanSource)),
                keyword: Keyword::Trample,
            },
        }],
        ..creature(
            "Bloodcrusher of Khorne",
            cost(&[generic(3), r()]),
            vec![CreatureType::Demon, CreatureType::Knight],
            3,
            3,
        )
    }
}

/// Bloodthirster — flying, trample; its combat damage to a player untaps it
/// and adds a combat phase, and it can't attack a player it already attacked
/// this turn (Port Razer's keyword), which ends the loop.
pub fn bloodthirster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::CantAttackPlayerAttackedThisTurn],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::Seq(vec![
            Effect::Untap { what: Selector::This, up_to: None },
            Effect::AdditionalCombatPhase { count: Value::ONE },
        ]))],
        ..creature("Bloodthirster", cost(&[generic(5), r()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Chaos Defiler — trample; entering or dying, choose a nonland permanent
/// of each opponent and destroy one of them at random.
pub fn chaos_defiler() -> CardDefinition {
    let cannon = || Effect::DestroyOnePerOpponent { filter: R::Nonland, random_one: true };
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(cannon()), on_dies(cannon())],
        ..artifact_creature(
            "Chaos Defiler",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Demon, CreatureType::Construct],
            5,
            4,
        )
    }
}

/// Chaos Mutation — exile any number of target creatures controlled by
/// different players (CR 601.2c, `ForEachPlayerTarget`); each controller
/// reveals until a creature card and puts it onto the battlefield, the rest on
/// the bottom.
pub fn chaos_mutation() -> CardDefinition {
    spell(
        "Chaos Mutation",
        cost(&[generic(3), u(), r()]),
        CardType::Instant,
        Effect::ForEachPlayerTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::AsPlayer {
                    who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                        Effect::RevealUntilOneToBattlefieldRestBottom { filter: R::Creature, damage_controller: false },
                    ])),
                }),
            }),
        },
    )
}

/// Chaos Terminator Lord — at the beginning of combat on your turn, another
/// target creature of yours gains double strike.
pub fn chaos_terminator_lord() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![step(
            TurnStep::BeginCombat,
            Effect::GrantKeyword {
                what: target_filtered(yours(R::Creature).and(R::OtherThanSource)),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Chaos Terminator Lord",
            cost(&[generic(3), r()]),
            vec![CreatureType::Astartes, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Dark Apostle — {3}, {T}: the next noncreature spell you cast this turn
/// has cascade.
pub fn dark_apostle() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: next_spell_cascades(R::Noncreature),
            ..Default::default()
        }],
        ..creature("Dark Apostle", cost(&[generic(3), r()]), vec![CreatureType::Astartes, CreatureType::Warlock], 3, 3)
    }
}

/// Deny Reality — cascade; return target permanent to its owner's hand.
pub fn deny_reality() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![cascade(5)],
        ..spell(
            "Deny Reality",
            cost(&[generic(3), u(), b()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(R::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        )
    }
}

/// Drach'Nyen — entering exiles up to one target creature; equipped creature
/// has menace and gets +X/+0 (X = the exiled card's power); equip {2}.
pub fn drachnyen() -> CardDefinition {
    CardDefinition {
        name: "Drach'Nyen",
        cost: cost(&[generic(4), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Menace], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +X/+0, where X is the exiled card's power.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: Value::PowerOf(Box::new(Selector::CardExiledWithSource)),
                toughness: Value::ZERO,
            },
        }],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move { what: target_filtered(R::Creature), to: ZoneDest::ExileWithSourceStamp }),
        })],
        ..Default::default()
    }
}

/// Exalted Flamer of Tzeentch — your upkeep returns a random instant or
/// sorcery card from your graveyard; your instants and sorceries deal 1 to
/// each opponent.
pub fn exalted_flamer_of_tzeentch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            step(
                TurnStep::Upkeep,
                Effect::ReturnRandomFromGraveyard { who: PlayerRef::You, filter: instant_or_sorcery(), count: Value::ONE },
            ),
            on_instant_or_sorcery_cast(Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            }),
        ],
        ..creature("Exalted Flamer of Tzeentch", cost(&[generic(2), u(), r()]), vec![CreatureType::Demon], 2, 4)
    }
}

/// Great Unclean One — your end step drains each opponent 2 life, then a
/// Plaguebearer of Nurgle per opponent with less life than you.
pub fn great_unclean_one() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![step(
            TurnStep::End,
            Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                Effect::ForEachOpponent {
                    body: Box::new(Effect::If {
                        cond: Predicate::ValueAtLeast(
                            Value::LifeOf(PlayerRef::You),
                            Value::Sum(vec![Value::LifeOf(PlayerRef::Triggerer), Value::ONE]),
                        ),
                        then: Box::new(make(plaguebearer(), Value::ONE)),
                        else_: Box::new(Effect::Noop),
                    }),
                },
            ]),
        )],
        ..creature("Great Unclean One", cost(&[generic(4), b()]), vec![CreatureType::Demon], 4, 5)
    }
}

/// Helbrute — haste; castable from your graveyard by also exiling another
/// creature card from it (Sarcophagus, as escape with a creature filter).
pub fn helbrute() -> CardDefinition {
    let mana = cost(&[generic(3), b(), r()]);
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Escape(mana.clone(), 1)],
        escape_exile_filter: Some(R::Creature),
        ..artifact_creature("Helbrute", mana, vec![CreatureType::Astartes, CreatureType::Dreadnought], 5, 4)
    }
}

/// Herald of Slaanesh — Demon spells cost {2} less; other Demons of yours
/// have haste.
pub fn herald_of_slaanesh() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Demon spells you cast cost {2} less to cast.",
                effect: StaticEffect::CostReduction { filter: demon(), amount: 2 },
            },
            StaticAbility {
                description: "Other Demons you control have haste.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(yours(demon()).and(R::OtherThanSource)),
                    keyword: Keyword::Haste,
                },
            },
        ],
        ..creature("Herald of Slaanesh", cost(&[generic(2), r()]), vec![CreatureType::Demon], 2, 2)
    }
}

/// Heralds of Tzeentch — flying; cascade.
pub fn heralds_of_tzeentch() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Cascade],
        triggered_abilities: vec![cascade(5)],
        ..creature("Heralds of Tzeentch", cost(&[generic(4), u()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Khârn the Betrayer — attacks or blocks each combat if able; losing
/// control of it draws you two; damage to it is prevented and an opponent
/// gains control of it. Residual: the next opponent in turn order, not one
/// of your choice.
pub fn kharn_the_betrayer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MustAttackOrBlock],
        static_abilities: vec![StaticAbility {
            description: "If damage would be dealt to Khârn the Betrayer, prevent that damage and an opponent of your choice gains control of it.",
            effect: StaticEffect::PreventDamageToSelfOpponentGainsControl,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LostControlOfThis, EventScope::SelfSource),
            effect: draw(Value::Const(2)),
        }],
        ..legend(
            "Khârn the Betrayer",
            cost(&[generic(3), r()]),
            vec![CreatureType::Astartes, CreatureType::Berserker],
            5,
            1,
        )
    }
}

/// Kill! Maim! Burn! — choose one or more: destroy target artifact; destroy
/// target creature; 3 damage to target player.
pub fn kill_maim_burn() -> CardDefinition {
    spell(
        "Kill! Maim! Burn!",
        cost(&[generic(3), b(), r()]),
        CardType::Instant,
        Effect::ChooseModesCast {
            modes: vec![
                Effect::Destroy { what: target_filtered(R::Artifact) },
                Effect::Destroy { what: target_filtered(R::Creature) },
                Effect::DealDamage { to: target_filtered(R::Player), amount: Value::Const(3) },
            ],
            min: 1,
            max: 3,
            allow_repeats: false,
        },
    )
}

/// Knight Rampager — trample; attacks an opponent chosen at random each of
/// your combats if able; dying deals 4 to an opponent chosen at random.
pub fn knight_rampager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::MustAttackChosenPlayer],
        triggered_abilities: vec![
            step(TurnStep::BeginCombat, Effect::RememberPlayerOnSource { who: PlayerRef::RandomOpponent }),
            on_dies(Effect::DealDamage { to: Selector::Player(PlayerRef::RandomOpponent), amount: Value::Const(4) }),
        ],
        ..artifact_creature("Knight Rampager", cost(&[generic(4), r()]), vec![CreatureType::Knight], 6, 5)
    }
}

/// Let the Galaxy Burn — cascade; X + 2 damage to each creature that didn't
/// enter this turn.
pub fn let_the_galaxy_burn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)) },
        }],
        ..spell(
            "Let the Galaxy Burn",
            cost(&[x(), generic(5), r()]),
            CardType::Sorcery,
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::EnteredThisTurn)))),
                amount: Value::Sum(vec![Value::XFromCost, Value::Const(2)]),
            },
        )
    }
}

/// Lord of Change — flying, ward {3}; entering draws three.
pub fn lord_of_change() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(3)])))],
        triggered_abilities: vec![etb(draw(Value::Const(3)))],
        ..creature("Lord of Change", cost(&[generic(6), u()]), vec![CreatureType::Demon], 6, 6)
    }
}

/// Lucius the Eternal — haste; dying exiles it, watching a target creature
/// an opponent controls: when that creature leaves, Lucius returns.
pub fn lucius_the_eternal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            Effect::Move { what: Selector::This, to: ZoneDest::Exile },
            Effect::ReturnSourceWhenTargetLeaves { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        ]))],
        ..legend(
            "Lucius the Eternal",
            cost(&[generic(3), b(), r()]),
            vec![CreatureType::Astartes, CreatureType::Warrior],
            5,
            3,
        )
    }
}

/// Magnus the Red — flying; your instants and sorceries cost {1} less per
/// creature token you control; his combat damage to a player makes a 3/3
/// Spawn.
pub fn magnus_the_red() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast cost {1} less to cast for each creature token you control.",
            effect: StaticEffect::CostReductionByValue {
                filter: instant_or_sorcery(),
                amount: Value::CountOf(Box::new(Selector::EachPermanent(yours(R::Creature.and(R::IsToken))))),
            },
        }],
        triggered_abilities: vec![on_combat_damage_to_player(make(spawn(), Value::ONE))],
        ..legend(
            "Magnus the Red",
            cost(&[generic(3), u(), r()]),
            vec![CreatureType::Demon, CreatureType::Primarch],
            4,
            5,
        )
    }
}

/// Mortarion, Daemon Primarch — flying; your end step may pay {X} (X up to
/// the life you lost this turn) for X 2/2 menace Astartes Warriors.
pub fn mortarion_daemon_primarch() -> CardDefinition {
    let warrior = TokenDefinition {
        keywords: vec![Keyword::Menace],
        ..token("Astartes Warrior", vec![Color::Black], vec![CreatureType::Astartes, CreatureType::Warrior], 2, 2)
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![step(
            TurnStep::End,
            Effect::MayPayGenericUpTo {
                max: Value::LifeLostThisTurn(PlayerRef::You),
                body: Box::new(make(warrior, Value::TriggerEventAmount)),
            },
        )],
        ..legend(
            "Mortarion, Daemon Primarch",
            cost(&[generic(5), b()]),
            vec![CreatureType::Demon, CreatureType::Primarch],
            5,
            6,
        )
    }
}

/// Mutalith Vortex Beast — trample; entering flips a coin per opponent: a
/// win draws you a card, a loss deals 3 to that player.
pub fn mutalith_vortex_beast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::ForEachOpponent {
            body: Box::new(Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(draw(Value::ONE)),
                on_tails: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(3),
                }),
            }),
        })],
        ..creature(
            "Mutalith Vortex Beast",
            cost(&[generic(4), u(), r()]),
            vec![CreatureType::Mutant, CreatureType::Beast],
            6,
            6,
        )
    }
}

/// Noise Marine — cascade; entering deals damage equal to the spells you've
/// cast this turn to any target.
pub fn noise_marine() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        triggered_abilities: vec![
            cascade(5),
            etb(Effect::DealDamage {
                to: target_filtered(R::Any),
                amount: Value::SpellsCastThisTurn(PlayerRef::You),
            }),
        ],
        ..creature("Noise Marine", cost(&[generic(4), r()]), vec![CreatureType::Astartes, CreatureType::Warrior], 3, 2)
    }
}

/// Nurgle's Conscription — put target creature card from an opponent's
/// graveyard onto the battlefield tapped under your control, then exile that
/// player's graveyard.
pub fn nurgles_conscription() -> CardDefinition {
    spell(
        "Nurgle's Conscription",
        cost(&[generic(4), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InOpponentGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::ExilePlayerGraveyard {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                filter: None,
            },
        ]),
    )
}

/// Nurgle's Rot — enchant creature an opponent controls; when it dies, this
/// returns to your hand and you make a Plaguebearer of Nurgle.
pub fn nurgles_rot() -> CardDefinition {
    CardDefinition {
        name: "Nurgle's Rot",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))) },
                make(plaguebearer(), Value::ONE),
            ]),
        }],
        ..Default::default()
    }
}

/// Pink Horror — your instants and sorceries deal 2 to any target; dying
/// splits into two 2/2 Blue Horrors that each deal 1 the same way.
pub fn pink_horror() -> CardDefinition {
    let blue = TokenDefinition {
        triggered_abilities: vec![on_instant_or_sorcery_cast(Effect::DealDamage {
            to: target_filtered(R::Any),
            amount: Value::ONE,
        })],
        ..token("Blue Horror", vec![Color::Blue, Color::Red], vec![CreatureType::Demon, CreatureType::Horror], 2, 2)
    };
    CardDefinition {
        triggered_abilities: vec![
            on_instant_or_sorcery_cast(Effect::DealDamage { to: target_filtered(R::Any), amount: Value::Const(2) }),
            on_dies(make(blue, Value::Const(2))),
        ],
        ..creature("Pink Horror", cost(&[generic(3), u(), r()]), vec![CreatureType::Demon, CreatureType::Horror], 4, 4)
    }
}

/// Plague Drone — flying; an opponent's life gain is life loss instead.
pub fn plague_drone() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would gain life, that player loses that much life instead.",
            effect: StaticEffect::LifeGainBecomesLoss { target: PlayerStaticTarget::EachOpponent },
        }],
        ..creature("Plague Drone", cost(&[generic(3), b()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Poxwalkers — deathtouch; casting a spell from anywhere but your hand
/// returns it from your graveyard tapped.
pub fn poxwalkers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::SpellNotCastFromHand }),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..creature("Poxwalkers", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 3, 1)
    }
}

/// Seeker of Slaanesh — haste; each opponent must attack with at least one
/// creature each combat if able.
pub fn seeker_of_slaanesh() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "Each opponent must attack with at least one creature each combat if able.",
            effect: StaticEffect::OpponentsMustAttackWithAtLeastOne,
        }],
        ..creature("Seeker of Slaanesh", cost(&[generic(3), r()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Sloppity Bilepiper — {2}, {T}, sacrifice a creature: the next creature
/// spell you cast this turn has cascade.
pub fn sloppity_bilepiper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature, 1)),
            sac_other_may_be_source: true,
            effect: next_spell_cascades(R::Creature),
            ..Default::default()
        }],
        ..creature("Sloppity Bilepiper", cost(&[generic(3), b()]), vec![CreatureType::Demon], 3, 3)
    }
}

/// Tallyman of Nurgle — lifelink; your end step after a death draws one and
/// loses 1 — or, after seven deaths, draws seven and loses 7.
pub fn tallyman_of_nurgle() -> CardDefinition {
    let pay = |n: i32| Effect::Seq(vec![draw(Value::Const(n)), lose(Value::Const(n))]);
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
            effect: Effect::If {
                cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::Const(7) },
                then: Box::new(pay(7)),
                else_: Box::new(pay(1)),
            },
        }],
        ..creature(
            "Tallyman of Nurgle",
            cost(&[generic(2), b()]),
            vec![CreatureType::Astartes, CreatureType::Warrior],
            2,
            3,
        )
    }
}

/// The Horus Heresy — I: for each opponent, gain control of up to one target
/// nonlegendary creature of theirs while this remains. II: draw a card per
/// creature you control but don't own. III: each player chooses a creature;
/// destroy them. Residual: III's choices start with the next opponent.
pub fn the_horus_heresy() -> CardDefinition {
    saga(
        "The Horus Heresy",
        cost(&[generic(3), u(), b(), r()]),
        vec![
            (
                1,
                Effect::ForEachOpponentTarget {
                    body: Box::new(Effect::ApplyToTargets {
                        max_targets: 8,
                        min_targets: 0,
                        filter: R::Creature
                            .and(R::ControlledByOpponent)
                            .and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
                        effect: Box::new(Effect::GainControlWhileSourceRemains { what: Selector::Target(0) }),
                    }),
                },
            ),
            (
                2,
                draw(Value::CountOf(Box::new(Selector::EachPermanent(
                    yours(R::Creature).and(R::Not(Box::new(R::OwnedByYou))),
                )))),
            ),
            (3, Effect::EachPlayerChoosesToDestroy { filter: R::Creature }),
        ],
    )
}

/// The Lost and the Damned — a land of yours entering without being played,
/// or a spell you cast from anywhere but your hand, makes a 3/3 Spawn.
/// Residual: a land played from graveyard or exile doesn't count.
pub fn the_lost_and_the_damned() -> CardDefinition {
    CardDefinition {
        name: "The Lost and the Damned",
        cost: cost(&[generic(1), u(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl)
                    .with_filter(Predicate::ValueAtMost(Value::TriggerEventAmount, Value::Const(0))),
                effect: make(spawn(), Value::ONE),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::SpellNotCastFromHand },
                ),
                effect: make(spawn(), Value::ONE),
            },
        ],
        ..Default::default()
    }
}

/// The Ruinous Powers — your upkeep exiles the top card of a random
/// opponent's library; you may cast it this turn with mana of any type, and
/// its owner loses life equal to its mana value. Residual: the rider reads
/// any spell you cast from exile that you don't own; lands can't be played.
pub fn the_ruinous_powers() -> CardDefinition {
    CardDefinition {
        name: "The Ruinous Powers",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            step(
                TurnStep::Upkeep,
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::RandomOpponent,
                    count: Value::ONE,
                    duration: MayPlayDuration::EndOfThisTurn,
                    pay_any_color: true,
                    max_mana_value: None,
                    pay_own_cost: false,
                    uncast_penalty: None,
                },
            ),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::All(vec![Predicate::CastSpellFromExile, Predicate::CastSpellNotOwnedByYou])),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Tzaangor Shaman — flying; its combat damage to a player copies the next
/// instant or sorcery you cast this turn.
pub fn tzaangor_shaman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_combat_damage_to_player(Effect::OnYourNextInstantSorceryThisTurn {
            body: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE }),
        })],
        ..creature("Tzaangor Shaman", cost(&[generic(2), u(), r()]), vec![CreatureType::Mutant, CreatureType::Shaman], 3, 3)
    }
}

/// Venomcrawler — lifelink; another creature dying puts a +1/+1 counter on
/// it.
pub fn venomcrawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::Not(Box::new(Predicate::TriggerSourceIsSelf))),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..artifact_creature("Venomcrawler", cost(&[generic(3), b()]), vec![CreatureType::Demon], 2, 2)
    }
}
