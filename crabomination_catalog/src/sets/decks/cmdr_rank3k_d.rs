//! Commander: two-card gaps of EDHREC's commanders ranked 2501-3500 (the
//! 2026-10-14 scan), fourth file — pairs whose two cards ride existing
//! primitives. Tests in `tests/recent_b/cmdr_rank3k_d.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{battalion, etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, hybrid, r, u, w, Color, ManaCost};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn lesson() -> Subtypes {
    Subtypes { spell_subtypes: vec![crate::card::SpellSubtype::Lesson], ..Default::default() }
}

fn trigger_is(filter: R) -> Predicate {
    Predicate::EntityMatches { what: Selector::TriggerSource, filter }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn token(name: &str, colors: Vec<Color>, types: Vec<CreatureType>, p: i32, t: i32, kws: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        keywords: kws,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

fn make(n: Value, def: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: def }
}

fn food() -> Effect {
    make(Value::ONE, Arc::new(crabomination_base::tokens::food_token()))
}

fn pump(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn grant(what: Selector, keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what, keyword, duration: Duration::EndOfTurn }
}

fn elf_knight() -> Arc<TokenDefinition> {
    token("Elf Knight", vec![Color::Green, Color::White], vec![CreatureType::Elf, CreatureType::Knight], 2, 2, vec![Keyword::Vigilance])
}

/// "Your turn, before attackers are declared" (CR 507/508 timing riders).
fn before_attackers() -> Predicate {
    Predicate::All(vec![
        Predicate::IsTurnOf(PlayerRef::You),
        Predicate::Any(vec![
            Predicate::CurrentStepIs(TurnStep::Upkeep),
            Predicate::CurrentStepIs(TurnStep::Draw),
            Predicate::CurrentStepIs(TurnStep::PreCombatMain),
            Predicate::CurrentStepIs(TurnStep::BeginCombat),
        ]),
    ])
}

// ── Tolsimir Wolfblood ───────────────────────────────────────────────────────

/// Conclave Cavalier — {G}{G}{W}{W} 4/4 vigilant Centaur Knight; dies: two 2/2
/// green and white vigilant Elf Knights.
pub fn conclave_cavalier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![on_dies(make(Value::Const(2), elf_knight()))],
        ..creature("Conclave Cavalier", cost(&[g(), g(), w(), w()]), vec![CreatureType::Centaur, CreatureType::Knight], 4, 4)
    }
}

/// Conclave Guildmage — {G}{W} 2/2 Elf Cleric. {G}, {T}: your creatures gain
/// trample until end of turn. {5}{W}, {T}: a 2/2 vigilant Elf Knight.
pub fn conclave_guildmage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[g()]),
                tap_cost: true,
                effect: grant(yours(R::Creature), Keyword::Trample),
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(5), w()]),
                tap_cost: true,
                effect: make(Value::ONE, elf_knight()),
                ..Default::default()
            },
        ],
        ..creature("Conclave Guildmage", cost(&[g(), w()]), vec![CreatureType::Elf, CreatureType::Cleric], 2, 2)
    }
}

// ── Sokka, Swordmaster ───────────────────────────────────────────────────────

/// Inspired Insurgent — {1}{W} 2/2 Human Peasant Ally. {1}, sacrifice it:
/// destroy target artifact or enchantment.
pub fn inspired_insurgent() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..creature(
            "Inspired Insurgent",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Peasant, CreatureType::Ally],
            2,
            2,
        )
    }
}

/// Suki, Kyoshi Captain — {2}{W} legendary 3/3. Other Warriors you control get
/// +1/+1. {3}{W}: attacking Warriors you control gain double strike until end
/// of turn.
pub fn suki_kyoshi_captain() -> CardDefinition {
    let warriors = || R::Creature.and(R::HasCreatureType(CreatureType::Warrior));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other Warriors you control get +1/+1.",
            effect: StaticEffect::PumpPT { applies_to: yours(warriors().and(R::OtherThanSource)), power: 1, toughness: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: grant(yours(warriors().and(R::IsAttacking)), Keyword::DoubleStrike),
            ..Default::default()
        }],
        ..legend(
            "Suki, Kyoshi Captain",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            3,
            3,
        )
    }
}

// ── Harald, King of Skemfar ──────────────────────────────────────────────────

/// Elven Bow — {G} Equipment: +1/+2 and reach. ETB: you may pay {2} for a 1/1
/// green Elf Warrior and attach it. Equip {3}.
pub fn elven_bow() -> CardDefinition {
    CardDefinition {
        name: "Elven Bow",
        cost: cost(&[g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 2, keywords: vec![Keyword::Reach], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::MayPay {
            description: "Pay {2} for an Elf Warrior wearing Elven Bow?".into(),
            mana_cost: cost(&[generic(2)]),
            body: Box::new(Effect::Seq(vec![
                make(Value::ONE, token("Elf Warrior", vec![Color::Green], vec![CreatureType::Elf, CreatureType::Warrior], 1, 1, vec![])),
                Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
            ])),
            else_: None,
        })],
        ..Default::default()
    }
}

/// King Harald's Revenge — {2}{G} sorcery: until end of turn, target creature
/// gets +1/+1 for each creature you control and trample, and must be blocked
/// if able.
pub fn king_haralds_revenge() -> CardDefinition {
    let n = || Value::count(yours(R::Creature));
    spell(
        "King Harald's Revenge",
        cost(&[generic(2), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT { what: target_filtered(R::Creature), power: n(), toughness: n(), duration: Duration::EndOfTurn },
            grant(Selector::Target(0), Keyword::Trample),
            grant(Selector::Target(0), Keyword::MustBeBlocked),
        ]),
    )
}

// ── Kolaghan, the Storm's Fury / Dragonlord Kolaghan ─────────────────────────

/// Kolaghan Monument — {3} artifact. {T}: add {B} or {R}. {4}{B}{R}: it becomes
/// a 4/4 black and red flying Dragon artifact creature until end of turn.
pub fn kolaghan_monument() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Black, Color::Red], Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), b(), r()]),
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![CreatureType::Dragon],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::BecomeColor {
                        what: Selector::This,
                        colors: vec![Color::Black, Color::Red],
                        duration: Duration::EndOfTurn,
                        additive: false,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..spell("Kolaghan Monument", cost(&[generic(3)]), CardType::Artifact, Effect::Noop)
    }
}

/// Boltwing Marauder — {3}{B}{R} 5/4 flying Dragon. Another creature you
/// control enters: target creature gets +2/+0 until end of turn.
pub fn boltwing_marauder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(trigger_is(R::Creature)),
            effect: pump(target_filtered(R::Creature), 2, 0),
        }],
        ..creature("Boltwing Marauder", cost(&[generic(3), b(), r()]), vec![CreatureType::Dragon], 5, 4)
    }
}

// ── Avalanche of Sector 7 ────────────────────────────────────────────────────

/// Relic Robber — {2}{R} 2/2 haste Goblin Rogue. Combat damage to a player:
/// that player creates a 0/1 colorless Goblin Construct artifact creature that
/// can't block and deals 1 damage to its controller each upkeep.
pub fn relic_robber() -> CardDefinition {
    let construct = Arc::new(TokenDefinition {
        name: "Goblin Construct".into(),
        power: 0,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBlock],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Construct], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::DealDamage { to: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    });
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::TriggerEventPlayer, count: Value::ONE, definition: construct },
        }],
        ..creature("Relic Robber", cost(&[generic(2), r()]), vec![CreatureType::Goblin, CreatureType::Rogue], 2, 2)
    }
}

/// Sardian Avenger — {1}{R} 1/1 first strike, trample Goblin Warrior. Attacks:
/// +X/+0, X = artifacts your opponents control. An opponent's artifact goes to
/// a graveyard from the battlefield: 1 damage to that player.
pub fn sardian_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike, Keyword::Trample],
        triggered_abilities: vec![
            on_attack(Effect::PumpPT {
                what: Selector::This,
                power: Value::count(Selector::EachPermanent(R::Artifact.and(R::ControlledByOpponent))),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::OpponentControl).with_filter(trigger_is(R::Artifact)),
                effect: Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                    amount: Value::ONE,
                },
            },
        ],
        ..creature("Sardian Avenger", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 1, 1)
    }
}

// ── Cao Cao, Lord of Wei ─────────────────────────────────────────────────────

/// Cunning Advisor — {3}{B} 1/1 Human Advisor. {T}: target opponent discards a
/// card; only during your turn, before attackers are declared.
pub fn cunning_advisor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(before_attackers()),
            effect: Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::ONE, random: false },
            ..Default::default()
        }],
        ..creature("Cunning Advisor", cost(&[generic(3), b()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 1)
    }
}

/// Wei Night Raiders — {2}{B}{B} 2/2 horsemanship. Whenever it deals damage to
/// an opponent, that player discards a card.
pub fn wei_night_raiders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature("Wei Night Raiders", cost(&[generic(2), b(), b()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

// ── Anthousa, Setessan Hero ──────────────────────────────────────────────────

/// Verdant Rebirth — {1}{G} instant: until end of turn, target creature gains
/// "When this creature dies, return it to its owner's hand." Draw a card.
pub fn verdant_rebirth() -> CardDefinition {
    spell(
        "Verdant Rebirth",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::GrantTriggeredAbility {
                what: target_filtered(R::Creature),
                trigger: Box::new(on_dies(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                })),
                duration: Duration::EndOfTurn,
            },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

/// Viridescent Wisps — {G} instant: target creature becomes green and gets
/// +1/+0 until end of turn. Draw a card.
pub fn viridescent_wisps() -> CardDefinition {
    spell(
        "Viridescent Wisps",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Green],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            pump(Selector::Target(0), 1, 0),
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

// ── Cloudhoof Kirin ──────────────────────────────────────────────────────────

/// Mindshrieker — {1}{U} 1/1 flying Spirit Bird. {2}: target player mills a
/// card; it gets +X/+X until end of turn, X the milled card's mana value.
pub fn mindshrieker() -> CardDefinition {
    let x = || Value::ManaValueOf(Box::new(Selector::LastMoved));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::Mill { who: target_filtered(R::Player), amount: Value::ONE },
                Effect::PumpPT { what: Selector::This, power: x(), toughness: x(), duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Mindshrieker", cost(&[generic(1), u()]), vec![CreatureType::Spirit, CreatureType::Bird], 1, 1)
    }
}

/// Shriekgeist — {1}{U} 1/1 flying Spirit; combat damage to a player: that
/// player mills two.
pub fn shriekgeist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Mill { who: Selector::Player(PlayerRef::TriggerEventPlayer), amount: Value::Const(2) },
        }],
        ..creature("Shriekgeist", cost(&[generic(1), u()]), vec![CreatureType::Spirit], 1, 1)
    }
}

// ── Icingdeath, Frost Tyrant ─────────────────────────────────────────────────

/// Crystal Dragon // Rob the Hoard — {4}{W}{W} 4/4 flying, vigilance Dragon.
/// Adventure {1}{W} sorcery: return target artifact, enchantment or legendary
/// card from your graveyard to your hand.
pub fn crystal_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        adventure: Some(Box::new(Adventure {
            name: "Rob the Hoard",
            cost: cost(&[generic(1), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Move {
                what: target_filtered(
                    R::Artifact.or(R::Enchantment).or(R::HasSupertype(Supertype::Legendary)).and(R::InYourGraveyard),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        })),
        ..creature("Crystal Dragon", cost(&[generic(4), w(), w()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// White Dragon — {4}{W}{W} 4/4 flying Dragon. Cold Breath — ETB: tap target
/// creature an opponent controls; it doesn't untap during its controller's next
/// untap step.
pub fn white_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::SkipNextUntap { what: Selector::Target(0) },
        ]))],
        ..creature("White Dragon", cost(&[generic(4), w(), w()]), vec![CreatureType::Dragon], 4, 4)
    }
}

// ── Tetzimoc, Primal Death ───────────────────────────────────────────────────

/// Foul Renewal — {3}{B} instant: return target creature card from your
/// graveyard to your hand; target creature gets -X/-X until end of turn, X
/// the returned card's toughness.
pub fn foul_renewal() -> CardDefinition {
    let x = || Value::Negate(Box::new(Value::ToughnessOf(Box::new(Selector::Target(0)))));
    spell(
        "Foul Renewal",
        cost(&[generic(3), b()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Creature.and(R::InYourGraveyard)), to: ZoneDest::Hand(PlayerRef::You) },
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                power: x(),
                toughness: x(),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Blood Beckoning — {B} sorcery, kicker {3}: return target creature card from
/// your graveyard to your hand; kicked, two target creature cards instead.
pub fn blood_beckoning() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        effect: Effect::CapTargetsAt {
            amount: Value::Sum(vec![Value::ONE, Value::OneIf(Box::new(Predicate::SpellWasKicked))]),
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature.and(R::InYourGraveyard),
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
            }),
        },
        ..spell("Blood Beckoning", cost(&[b()]), CardType::Sorcery, Effect::Noop)
    }
}

// ── The Lion-Turtle / Haru, Hidden Talent ────────────────────────────────────

/// The Art of Tea — {1}{G} instant — Lesson: a +1/+1 counter on up to one
/// target creature you control; create a Food.
pub fn the_art_of_tea() -> CardDefinition {
    CardDefinition {
        subtypes: lesson(),
        ..spell(
            "The Art of Tea",
            cost(&[generic(1), g()]),
            CardType::Instant,
            Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
                food(),
            ]),
        )
    }
}

/// Match the Odds — {2}{G} sorcery — Lesson: a 1/1 white Ally token with a
/// +1/+1 counter for each creature your opponents control.
pub fn match_the_odds() -> CardDefinition {
    CardDefinition {
        subtypes: lesson(),
        ..spell(
            "Match the Odds",
            cost(&[generic(2), g()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                make(Value::ONE, token("Ally", vec![Color::White], vec![CreatureType::Ally], 1, 1, vec![])),
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::count(Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent))),
                },
            ]),
        )
    }
}

/// Kyoshi Warrior Exemplars — {3}{G} 4/3 Human Warrior Ally. Attacks, if you
/// control eight or more lands: your creatures get +2/+2 until end of turn.
pub fn kyoshi_warrior_exemplars() -> CardDefinition {
    let mut t = on_attack(pump(yours(R::Creature), 2, 2));
    t.event = t.event.with_filter(Predicate::ValueAtLeast(Value::count(yours(R::Land)), Value::Const(8)));
    CardDefinition {
        triggered_abilities: vec![t],
        ..creature(
            "Kyoshi Warrior Exemplars",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            4,
            3,
        )
    }
}

// ── Strider, Ranger of the North ─────────────────────────────────────────────

/// Strider, Ranger of the North — {2}{R}{G} legendary 4/4. Landfall: target
/// creature gets +1/+1 until end of turn, then first strike if its power is 4
/// or greater.
pub fn strider_ranger_of_the_north() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(trigger_is(R::Land)),
            effect: Effect::Seq(vec![
                pump(target_filtered(R::Creature), 1, 1),
                Effect::If {
                    cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::PowerAtLeast(4) },
                    then: Box::new(grant(Selector::Target(0), Keyword::FirstStrike)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Strider, Ranger of the North",
            cost(&[generic(2), r(), g()]),
            vec![CreatureType::Human, CreatureType::Ranger],
            4,
            4,
        )
    }
}

/// Gimli, Counter of Kills — {3}{R} legendary 4/3 trample. Whenever a creature
/// an opponent controls dies, 1 damage to that creature's controller.
pub fn gimli_counter_of_kills() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..legend("Gimli, Counter of Kills", cost(&[generic(3), r()]), vec![CreatureType::Dwarf, CreatureType::Warrior], 4, 3)
    }
}

// ── Purraj of Urborg ─────────────────────────────────────────────────────────

/// Basal Thrull — {B}{B} 1/2 Thrull. {T}, sacrifice it: add {B}{B}.
pub fn basal_thrull() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black, Color::Black]) },
            ..Default::default()
        }],
        ..creature("Basal Thrull", cost(&[b(), b()]), vec![CreatureType::Thrull], 1, 2)
    }
}

/// Viper, Cruel Conspirator — {1}{B} legendary 2/2. {B}: target creature
/// attacking alone gets +1/+1 until end of turn. {B}: it gains your choice of
/// deathtouch or lifelink until end of turn.
pub fn viper_cruel_conspirator() -> CardDefinition {
    let alone = || R::Creature.and(R::IsAttackingAlone);
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility { mana_cost: cost(&[b()]), effect: pump(target_filtered(alone()), 1, 1), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::ChooseMode(vec![
                    grant(target_filtered(alone()), Keyword::Deathtouch),
                    grant(target_filtered(alone()), Keyword::Lifelink),
                ]),
                ..Default::default()
            },
        ],
        ..legend(
            "Viper, Cruel Conspirator",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Villain],
            2,
            2,
        )
    }
}

// ── Agrus Kos, Wojek Veteran ─────────────────────────────────────────────────

/// Nobilis of War — {R/W}x5 3/4 flying Spirit Avatar; attacking creatures you
/// control get +2/+0.
pub fn nobilis_of_war() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control get +2/+0.",
            effect: StaticEffect::PumpPT { applies_to: yours(R::Creature.and(R::IsAttacking)), power: 2, toughness: 0 },
        }],
        ..creature("Nobilis of War", cost(&[rw(), rw(), rw(), rw(), rw()]), vec![CreatureType::Spirit, CreatureType::Avatar], 3, 4)
    }
}

/// Rise of the Hobgoblins — {R/W}{R/W} enchantment. ETB: you may pay {X} for X
/// 1/1 red and white Goblin Soldiers. {R/W}: red creatures and white creatures
/// you control gain first strike until end of turn.
pub fn rise_of_the_hobgoblins() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    let goblin = token("Goblin Soldier", vec![Color::Red, Color::White], vec![CreatureType::Goblin, CreatureType::Soldier], 1, 1, vec![]);
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayPayX {
            description: "Pay {X} for X Goblin Soldiers?".into(),
            body: Box::new(make(Value::XFromCost, goblin)),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[rw()]),
            effect: grant(yours(R::Creature.and(R::HasColor(Color::Red).or(R::HasColor(Color::White)))), Keyword::FirstStrike),
            ..Default::default()
        }],
        ..spell("Rise of the Hobgoblins", cost(&[rw(), rw()]), CardType::Enchantment, Effect::Noop)
    }
}

// ── Ty Lee, Artful Acrobat ───────────────────────────────────────────────────

/// Blood Mist — {3}{R} enchantment: at the beginning of combat on your turn,
/// target creature you control gains double strike until end of turn.
pub fn blood_mist() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: grant(target_filtered(R::Creature.and(R::ControlledByYou)), Keyword::DoubleStrike),
        }],
        ..spell("Blood Mist", cost(&[generic(3), r()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Rush of Adrenaline — {R} instant: target creature gets +2/+1 and trample
/// until end of turn.
pub fn rush_of_adrenaline() -> CardDefinition {
    spell(
        "Rush of Adrenaline",
        cost(&[r()]),
        CardType::Instant,
        Effect::Seq(vec![pump(target_filtered(R::Creature), 2, 1), grant(Selector::Target(0), Keyword::Trample)]),
    )
}

// ── Attuma, Atlantean Warlord ────────────────────────────────────────────────

/// Gray Harbor Merfolk — {1}{U} 0/3 unblockable Merfolk Rogue; +2/+0 while you
/// control a commander that's a creature or planeswalker.
pub fn gray_harbor_merfolk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+0 as long as you control a commander that's a creature or planeswalker.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SelectorExists(yours(R::IsCommander.and(R::Creature.or(R::Planeswalker)))),
                power: 2,
                toughness: 0,
                keywords: vec![],
            },
        }],
        ..creature("Gray Harbor Merfolk", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Rogue], 0, 3)
    }
}

/// Streambed Aquitects — {1}{U}{U} 2/3 Merfolk Scout. {T}: target Merfolk gets
/// +1/+1 and islandwalk until end of turn. {T}: target land becomes an Island
/// until end of turn.
pub fn streambed_aquitects() -> CardDefinition {
    let merfolk = || R::Creature.and(R::HasCreatureType(CreatureType::Merfolk));
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    pump(target_filtered(merfolk()), 1, 1),
                    grant(Selector::Target(0), Keyword::Landwalk(LandType::Island)),
                ]),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::BecomeBasicLand {
                    what: target_filtered(R::Land),
                    land_type: LandType::Island,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..creature("Streambed Aquitects", cost(&[generic(1), u(), u()]), vec![CreatureType::Merfolk, CreatureType::Scout], 2, 3)
    }
}

// ── Leatherhead, Swamp Stalker ───────────────────────────────────────────────

/// Bushmaster, Coiled Henchman — {1}{G} legendary 2/1 deathtouch. Each other
/// creature you control with a +1/+1 counter on it has deathtouch.
pub fn bushmaster_coiled_henchman() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "Each other creature you control with a +1/+1 counter on it has deathtouch.",
            effect: StaticEffect::GrantKeyword {
                applies_to: yours(R::Creature.and(R::OtherThanSource).and(R::WithCounter(CounterType::PlusOnePlusOne))),
                keyword: Keyword::Deathtouch,
            },
        }],
        ..legend(
            "Bushmaster, Coiled Henchman",
            cost(&[generic(1), g()]),
            vec![CreatureType::Snake, CreatureType::Human, CreatureType::Villain],
            2,
            1,
        )
    }
}

/// Tenderize — {1}{G} instant: target creature you control deals damage equal
/// to its power to target creature an opponent controls.
pub fn tenderize() -> CardDefinition {
    spell(
        "Tenderize",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::DealDamageFrom {
            source: target_filtered(R::Creature.and(R::ControlledByYou)),
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            amount: Value::PowerOf(Box::new(Selector::Target(0))),
        },
    )
}

// ── Old Flitterfang ──────────────────────────────────────────────────────────

/// Giant's Skewer — {1}{B} Equipment: +2/+1; whenever the equipped creature
/// deals combat damage to a creature, create a Food. Equip {3}.
pub fn giants_skewer() -> CardDefinition {
    CardDefinition {
        name: "Giant's Skewer",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
                effect: food(),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Tempting Witch — {2}{B} 1/3 Human Warlock. ETB: a Food. {2}, {T}, sacrifice
/// a Food: target player loses 3 life.
pub fn tempting_witch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(food())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::HasArtifactSubtype(ArtifactSubtype::Food), 1)),
            effect: Effect::LoseLife { who: target_filtered(R::Player), amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature("Tempting Witch", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Warlock], 1, 3)
    }
}

// ── Zuko, Exiled Prince ──────────────────────────────────────────────────────

/// Fire Nation Archers — {3}{R} 3/4 reach Human Archer. {5}: 2 damage to each
/// opponent and a 2/2 red Soldier.
pub fn fire_nation_archers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) },
                make(Value::ONE, token("Soldier", vec![Color::Red], vec![CreatureType::Soldier], 2, 2, vec![])),
            ]),
            ..Default::default()
        }],
        ..creature("Fire Nation Archers", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Archer], 3, 4)
    }
}

/// Loyal Fire Sage — {2}{R} 3/3 Human Cleric Ally, firebending 1. {5}: a 1/1
/// white Ally.
pub fn loyal_fire_sage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Firebending(1)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            effect: make(Value::ONE, token("Ally", vec![Color::White], vec![CreatureType::Ally], 1, 1, vec![])),
            ..Default::default()
        }],
        ..creature(
            "Loyal Fire Sage",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Cleric, CreatureType::Ally],
            3,
            3,
        )
    }
}

// ── Paladin Elizabeth Taggerdy ───────────────────────────────────────────────

/// General Thunderbolt Ross — {W} legendary 1/1 first strike. Battalion:
/// attacking creatures get +1/+0 until end of turn.
pub fn general_thunderbolt_ross() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![battalion(pump(Selector::EachPermanent(R::Creature.and(R::IsAttacking)), 1, 0))],
        ..legend("General Thunderbolt Ross", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Horn of the Mark — {2} legendary artifact. Whenever two or more creatures
/// you control attack a player, look at the top five; you may reveal a creature
/// card and put it into your hand; the rest on the bottom in a random order.
pub fn horn_of_the_mark() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            // CR 506.2 / 603.2c — two or more at ONE player, once per such player.
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    trigger_is(R::IsAttackingOpponentPlayer),
                    Predicate::AttackedDefenderWithCountAtLeast {
                        who: PlayerRef::You,
                        defender: PlayerRef::DefendingPlayer,
                        at_least: 2,
                        include_planeswalkers: false,
                    },
                ]))
                .once_per_batch_per_defender(),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::Creature),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            })),
        }],
        ..spell("Horn of the Mark", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}
