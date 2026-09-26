//! Commander: the cards the **Riders of Rohan** precon (LTC, Éowyn,
//! Shieldmaiden) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_eowyn.rs`.
//!
//! Residuals (each also on its card):
//! - **Call for Aid** — nothing stops you sacrificing the borrowed creatures.
//! - **Denethor, Stone Seer** and **Éomer, King of Rohan** — you become the
//!   monarch (the printed "target player").
//! - **Fealty to the Realm** — the Aura's controller controls the creature,
//!   not whoever is the monarch.
//! - **Gilraen, Dúnedain Protector** — the creature always comes back at the
//!   next end step with its counters (never at once).
//! - **Visions of Glory** — its flashback isn't discounted by your
//!   commander's mana value.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{dash, etb, on_attack, target_any, target_filtered};
use crate::effect::{
    DelayedTriggerKind, Duration, Effect, LibraryPosition, LookPick, PlayerRef, Predicate, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{cost, generic, r, u, w, x, Color, ManaCost};

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

fn enchantment(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn human_token(name: &str, color: Color, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    })
}

/// Two (or `n`) 2/2 red Human Knights with trample and haste.
fn knights(n: Value) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: human_token(
            "Human Knight",
            Color::Red,
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
            vec![Keyword::Trample, Keyword::Haste],
        ),
    }
}

fn soldiers(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: human_token(
            "Human Soldier",
            Color::White,
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
            vec![],
        ),
    }
}

fn monarch() -> Effect {
    Effect::BecomeMonarch { who: PlayerRef::You }
}

fn you_are_monarch() -> Predicate {
    Predicate::IsMonarch { who: PlayerRef::You }
}

fn no_monarch() -> Predicate {
    Predicate::Not(Box::new(Predicate::IsMonarch { who: PlayerRef::EachPlayer }))
}

fn human() -> R {
    R::HasCreatureType(CreatureType::Human)
}

fn yours() -> R {
    R::Creature.and(R::ControlledByYou)
}

/// "Whenever this or another Human you control enters, …"
fn human_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: human() }),
        effect,
    }
}

fn if_then(cond: Predicate, then: Effect) -> Effect {
    Effect::If { cond, then: Box::new(then), else_: Box::new(Effect::Noop) }
}

/// Éowyn, Shieldmaiden — first strike; each of your combats after another
/// Human entered this turn, two Knights, and a card at six Humans.
pub fn eowyn_shieldmaiden() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer)
                .with_filter(Predicate::CreatureEnteredThisTurnMatching {
                    who: PlayerRef::You,
                    filter: human().and(R::OtherThanSource),
                }),
            effect: Effect::Seq(vec![
                knights(Value::Const(2)),
                if_then(
                    Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(yours().and(human())),
                        n: Value::Const(6),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ),
            ]),
        }],
        ..legend(
            "Éowyn, Shieldmaiden",
            cost(&[generic(2), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            5,
            4,
        )
    }
}

/// Aragorn, King of Gondor — vigilance, lifelink; entering makes you the
/// monarch; attacking stops a blocker, or every blocker while you're the
/// monarch.
pub fn aragorn_king_of_gondor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        triggered_abilities: vec![
            etb(monarch()),
            on_attack(Effect::Seq(vec![
                Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature.and(R::ControlledByOpponent),
                    effect: Box::new(Effect::GrantKeyword {
                        what: Selector::Target(0),
                        keyword: Keyword::CantBlock,
                        duration: Duration::EndOfTurn,
                    }),
                },
                if_then(
                    you_are_monarch(),
                    Effect::GrantKeyword {
                        what: Selector::EachPermanent(R::Creature),
                        keyword: Keyword::CantBlock,
                        duration: Duration::EndOfTurn,
                    },
                ),
            ])),
        ],
        ..legend(
            "Aragorn, King of Gondor",
            cost(&[generic(1), u(), r(), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            4,
            4,
        )
    }
}

/// Archivist of Gondor — your commander connecting with no monarch makes you
/// the monarch; the monarch draws at their end step.
pub fn archivist_of_gondor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::All(vec![
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander },
                        no_monarch(),
                    ]),
                ),
                effect: monarch(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer)
                    .with_filter(Predicate::IsMonarch { who: PlayerRef::ActivePlayer }),
                effect: Effect::Draw { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
            },
        ],
        ..creature("Archivist of Gondor", cost(&[generic(2), u()]), vec![CreatureType::Human, CreatureType::Advisor], 2, 3)
    }
}

/// Beregond of the Guard — a Human of yours entering pumps your team +1/+1
/// with vigilance.
pub fn beregond_of_the_guard() -> CardDefinition {
    let team = || Selector::EachPermanent(yours());
    CardDefinition {
        triggered_abilities: vec![human_enters(Effect::Seq(vec![
            Effect::PumpPT { what: team(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: team(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
        ]))],
        ..legend(
            "Beregond of the Guard",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Boromir, Gondor's Hope — entering or attacking digs six for a Human or
/// artifact.
pub fn boromir_gondors_hope() -> CardDefinition {
    let dig = || {
        Effect::LookPickToHand(Box::new(LookPick {
            count: Value::Const(6),
            pick_filter: Some(human().or(R::Artifact)),
            rest_bottom_random: true,
            optional: true,
            ..Default::default()
        }))
    };
    CardDefinition {
        triggered_abilities: vec![etb(dig()), on_attack(dig())],
        ..legend(
            "Boromir, Gondor's Hope",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Call for Aid — borrow all of an opponent's creatures for the turn,
/// untapped and hasty; you can't attack that player.
///
/// Residual: nothing stops you sacrificing them.
pub fn call_for_aid() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature };
    spell(
        "Call for Aid",
        cost(&[generic(4), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Untap { what: theirs(), up_to: None },
            Effect::GrantKeyword { what: theirs(), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::CantAttackPlayerThisTurn { who: PlayerRef::You, defender: PlayerRef::Target(0) },
            Effect::GainControl { what: theirs(), to: None, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Champions of Minas Tirith — entering makes you the monarch; while you
/// are, an opponent's combat can't come at you unless they pay {X} (X = cards
/// in their hand).
pub fn champions_of_minas_tirith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(monarch()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::OpponentControl)
                    .with_filter(you_are_monarch()),
                // "That opponent may pay {X}, where X is the number of cards in
                // their hand. If they don't, they can't attack you this combat."
                effect: Effect::WithX {
                    x: Value::HandSizeOf(PlayerRef::ActivePlayer),
                    body: Box::new(Effect::UnlessPlayerPays {
                        who: PlayerRef::ActivePlayer,
                        cost: crate::card::WardCost::GenericXFromCost,
                        then: Box::new(Effect::CantAttackPlayerThisTurn {
                            who: PlayerRef::ActivePlayer,
                            defender: PlayerRef::You,
                        }),
                        if_paid: None,
                    }),
                },
            },
        ],
        ..creature(
            "Champions of Minas Tirith",
            cost(&[generic(5), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            6,
        )
    }
}

/// Court of Ire — entering makes you the monarch; your upkeep deals 2 to any
/// target, 7 while you're the monarch.
pub fn court_of_ire() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(monarch()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::IfPred {
                        pred: Box::new(you_are_monarch()),
                        then: Box::new(Value::Const(7)),
                        else_: Box::new(Value::Const(2)),
                    },
                },
            },
        ],
        ..enchantment("Court of Ire", cost(&[generic(3), r(), r()]))
    }
}

/// Crown of Gondor — +1/+1 per creature you control; a legend of yours
/// entering with no monarch makes you the monarch; equip {4}, {3} less while
/// you're the monarch.
pub fn crown_of_gondor() -> CardDefinition {
    CardDefinition {
        name: "Crown of Gondor",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(4)]))],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Equip costs {3} less to activate if you're the monarch.",
            effect: crate::card::StaticEffect::EquipCostReducedWhile {
                condition: Predicate::IsMonarch { who: PlayerRef::You },
                amount: 3,
            },
        }],
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale { filter: yours(), per_power: 1, per_toughness: 1, ..Default::default() }),
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                },
                no_monarch(),
            ])),
            effect: monarch(),
        }],
        ..Default::default()
    }
}

/// Denethor, Stone Seer — scry 2 on entering; {3}{R}, {T}, sacrifice it:
/// become the monarch and deal 3 to any target.
///
/// Residual: you become the monarch (the printed "target player").
pub fn denethor_stone_seer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Seq(vec![monarch(), Effect::DealDamage { to: target_any(), amount: Value::Const(3) }]),
            ..Default::default()
        }],
        ..legend("Denethor, Stone Seer", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Noble], 1, 3)
    }
}

/// Faramir, Steward of Gondor — a big legend of yours entering makes you the
/// monarch; the monarch's end step makes two Soldiers.
pub fn faramir_steward_of_gondor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::Not(Box::new(R::ManaValueAtMost(3)))),
                    },
                ),
                effect: monarch(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                    .with_filter(you_are_monarch()),
                effect: soldiers(2),
            },
        ],
        ..legend(
            "Faramir, Steward of Gondor",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            2,
        )
    }
}

/// Fealty to the Realm — makes you the monarch; the monarch controls the
/// enchanted creature, which attacks each combat.
///
/// Residual: the Aura's controller controls it, not the monarch.
pub fn fealty_to_the_realm() -> CardDefinition {
    CardDefinition {
        name: "Fealty to the Realm",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::MustAttack], ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            monarch(),
            Effect::GainControlWhileSourceRemains { what: Selector::attached_to(Selector::This) },
        ]))],
        ..Default::default()
    }
}

/// Forth Eorlingas! — X hasty trampling Knights; your creatures connecting
/// this turn make you the monarch.
pub fn forth_eorlingas() -> CardDefinition {
    spell(
        "Forth Eorlingas!",
        cost(&[x(), r(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            knights(Value::XFromCost),
            Effect::CreaturesYouControlDealingCombatDamageThisTurn { body: Box::new(monarch()) },
        ]),
    )
}

/// Gilraen, Dúnedain Protector — {2}, {T}: blink another creature of yours
/// through the next end step, back with vigilance and lifelink counters.
///
/// Residual: it always comes back at the next end step.
pub fn gilraen_dunedain_protector() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Exile { what: target_filtered(yours().and(R::OtherThanSource)) },
                Effect::DelayUntilWithCapture {
                    kind: DelayedTriggerKind::NextEndStep,
                    capture: Selector::Target(0),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::TargetFiltered { slot: 0, filter: R::InExile },
                            to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                        },
                        Effect::AddKeywordCounter {
                            what: Selector::Target(0),
                            keyword: Keyword::Vigilance,
                            amount: Value::ONE,
                        },
                        Effect::AddKeywordCounter {
                            what: Selector::Target(0),
                            keyword: Keyword::Lifelink,
                            amount: Value::ONE,
                        },
                    ])),
                },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Gilraen, Dúnedain Protector",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            3,
        )
    }
}

/// Gimli of the Glittering Caves — double strike; another legend of yours
/// entering grows it; its combat damage to a player makes a Treasure.
pub fn gimli_of_the_glittering_caves() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)),
                    },
                ),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(crabomination_base::tokens::treasure_token()),
                },
            },
        ],
        ..legend(
            "Gimli of the Glittering Caves",
            cost(&[generic(2), r()]),
            vec![CreatureType::Dwarf, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Grey Host Reinforcements — flying, ward {3}; entering exiles a player's
/// graveyard and grows per creature card exiled.
pub fn grey_host_reinforcements() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(3)])))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0), filter: None },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::ExiledThisResolution { filter: R::Creature }),
            },
        ]))],
        ..creature(
            "Grey Host Reinforcements",
            cost(&[generic(3), w()]),
            vec![CreatureType::Spirit, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Lost to Legend — a nonland historic permanent goes fourth from the top of
/// its owner's library.
pub fn lost_to_legend() -> CardDefinition {
    let historic = R::Artifact
        .or(R::HasSupertype(Supertype::Legendary))
        .or(R::HasEnchantmentSubtype(EnchantmentSubtype::Saga));
    spell(
        "Lost to Legend",
        cost(&[w(), w()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::Permanent.and(R::Not(Box::new(R::Land))).and(historic)),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::FromTop(3) },
        },
    )
}

/// Oath of Eorl — Saga: two Soldiers; two Knights; an indestructible
/// counter on a Human and the monarchy.
pub fn oath_of_eorl() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, soldiers(2)),
            (2, knights(Value::Const(2))),
            (
                3,
                Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 1,
                        min_targets: 0,
                        filter: yours().and(human()),
                        effect: Box::new(Effect::AddKeywordCounter {
                            what: Selector::Target(0),
                            keyword: Keyword::Indestructible,
                            amount: Value::ONE,
                        }),
                    },
                    monarch(),
                ]),
            ),
        ],
        ..enchantment("Oath of Eorl", cost(&[generic(3), r(), w()]))
    }
}

/// Riders of Rohan — two Knights on entering; dash {4}{R}{W}.
pub fn riders_of_rohan() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(dash(cost(&[generic(4), r(), w()]))),
        triggered_abilities: vec![etb(knights(Value::Const(2)))],
        ..creature("Riders of Rohan", cost(&[generic(3), r(), w()]), vec![CreatureType::Human, CreatureType::Knight], 4, 4)
    }
}

/// Taunt from the Rampart — goad every opposing creature; they can't block
/// until your next turn.
pub fn taunt_from_the_rampart() -> CardDefinition {
    let theirs = || Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent));
    spell(
        "Taunt from the Rampart",
        cost(&[generic(3), r(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Goad { what: theirs() },
            Effect::GrantKeyword { what: theirs(), keyword: Keyword::CantBlock, duration: Duration::UntilYourNextUntap },
        ]),
    )
}

/// Théoden, King of Rohan — a Human of yours entering gives a creature
/// double strike.
pub fn theoden_king_of_rohan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![human_enters(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::DoubleStrike,
            duration: Duration::EndOfTurn,
        })],
        ..legend(
            "Théoden, King of Rohan",
            cost(&[generic(1), r(), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            3,
        )
    }
}

/// Visions of Glory — a 1/1 Human per creature you control; flashback
/// {8}{W}{W}.
///
/// Residual: the flashback isn't discounted by your commander's mana value.
pub fn visions_of_glory() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(8), w(), w()]))],
        ..spell(
            "Visions of Glory",
            cost(&[generic(4), w()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::count(Selector::EachPermanent(yours())),
                definition: human_token("Human", Color::White, vec![CreatureType::Human], 1, 1, vec![]),
            },
        )
    }
}

/// Éomer, King of Rohan — double strike; enters with a counter per other
/// Human; makes you the monarch and deals its power to any target.
///
/// Residual: you become the monarch (the printed "target player").
pub fn eomer_king_of_rohan() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::count(Selector::EachPermanent(yours().and(human()).and(R::OtherThanSource))),
        )),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            monarch(),
            Effect::DealDamage { to: target_any(), amount: Value::PowerOf(Box::new(Selector::This)) },
        ]))],
        ..legend(
            "Éomer, King of Rohan",
            cost(&[generic(3), r(), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            2,
            2,
        )
    }
}
