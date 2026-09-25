//! Commander: the cards the **Wakanda Forever** precon (MSC, T'Challa, the
//! Black Panther) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_tchalla.rs`.
//!
//! Residuals (each also on its card):
//! - **Ancestral Communion** — its copy keeps the original's target unless a
//!   prompting seat re-aims it.
//! - **Heart-Shaped Herb** — the sacrificed creature is the auto-pick, and it
//!   returns under your control.
//! - **Panther Habit** — a replacement, so damage that can't be prevented still
//!   becomes counters.
//! - **Wakanda Forever!** — the two picks are the highest-mana-value
//!   permanent cards revealed.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, cost, g, generic, w, x};
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

fn spell(name: &'static str, mana: ManaCost, ty: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![ty], effect, ..Default::default() }
}

fn artifact_sub(name: &'static str, mana: ManaCost, sub: ArtifactSubtype) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![sub], ..Default::default() },
        ..Default::default()
    }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn count(filter: R) -> Value {
    Value::count(Selector::EachPermanent(filter))
}

fn controls_at_least(filter: R, n: i32) -> Predicate {
    Predicate::SelectorCountAtLeast { sel: Selector::EachPermanent(filter), n: Value::Const(n) }
}

fn you_become_monarch() -> Effect {
    Effect::BecomeMonarch { who: PlayerRef::You }
}

fn no_monarch() -> Predicate {
    Predicate::Not(Box::new(Predicate::IsMonarch { who: PlayerRef::EachPlayer }))
}

fn begin_combat_on_your_turn(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
        effect,
    }
}

fn with_filter(t: TriggeredAbility, p: Predicate) -> TriggeredAbility {
    TriggeredAbility { event: t.event.with_filter(p), effect: t.effect }
}

fn counter(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

/// The Vibranium token: an indestructible artifact with "{T}: Add {C}. This
/// mana can't be spent to cast a nonartifact spell."
fn vibranium(tapped: bool) -> TokenDefinition {
    TokenDefinition {
        name: "Vibranium".into(),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Indestructible],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colorless(Value::ONE)),
                    SpendRestriction::NoNonartifactSpells,
                ),
            },
            ..Default::default()
        }],
        tapped,
        ..Default::default()
    }
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: n, definition: Arc::new(def) }
}

fn tapped_vibranium() -> Effect {
    make(Value::ONE, vibranium(true))
}

fn soldier() -> TokenDefinition {
    TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::White],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

fn big_artifact_cast() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(R::Artifact.and(R::ManaValueAtLeast(4)))),
        effect: Effect::Noop,
    }
}

fn vehicle(name: &'static str, mana: ManaCost, p: i32, t: i32, crew: u32) -> CardDefinition {
    CardDefinition {
        power: p,
        toughness: t,
        keywords: vec![Keyword::Crew(crew)],
        ..artifact_sub(name, mana, ArtifactSubtype::Vehicle)
    }
}

/// T'Challa, the Black Panther — Vibranium on entry and attack; grows with
/// each artifact spell of mana value 4 or greater.
pub fn tchalla_the_black_panther() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(tapped_vibranium()),
            on_attack(tapped_vibranium()),
            TriggeredAbility { effect: counter(Selector::This, Value::Const(2)), ..big_artifact_cast() },
        ],
        ..legendary(creature(
            "T'Challa, the Black Panther",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Hero],
            2,
            2,
        ))
    }
}

/// Ancestral Communion — regrow a permanent card; copied while you control
/// your commander. Residual: the copy keeps its target on auto seats.
pub fn ancestral_communion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource)
                .with_filter(Predicate::ControlsOwnCommander { who: PlayerRef::You }),
            effect: Effect::CopySpellMayChooseTargets { what: Selector::This, count: Value::ONE },
        }],
        ..spell(
            "Ancestral Communion",
            cost(&[generic(1), g()]),
            CardType::Sorcery,
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Bast, Panther Goddess — fights only beside two others; each attack pumps
/// an attacker by your creature count.
pub fn bast_panther_goddess() -> CardDefinition {
    let gated = |keyword| StaticAbility {
        description: "Bast can't attack or block unless you control three or more creatures.",
        effect: StaticEffect::WhileCondition {
            condition: Predicate::Not(Box::new(controls_at_least(yours(R::Creature), 3))),
            inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword }),
        },
    };
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample, Keyword::Indestructible],
        static_abilities: vec![gated(Keyword::CantAttack), gated(Keyword::CantBlock)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: count(yours(R::Creature)),
                toughness: count(yours(R::Creature)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legendary(creature(
            "Bast, Panther Goddess",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Cat, CreatureType::God],
            4,
            4,
        ))
    }
}

/// Dora Milaje Elite — Vibranium when behind on lands; sacrifices to make your
/// legends indestructible.
pub fn dora_milaje_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![with_filter(etb(tapped_vibranium()), Predicate::OpponentControlsMoreLandsThanYou)],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(yours(R::HasSupertype(Supertype::Legendary))),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dora Milaje Elite", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

/// Everett K. Ross, Hapless Attaché — your commanders get +1/+1 and lifelink;
/// being attacked by two or more draws a card.
pub fn everett_k_ross_hapless_attache() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Commander creatures you control get +1/+1 and have lifelink.",
            effect: StaticEffect::AnthemForFilter {
                filter: yours(R::Creature.and(R::IsCommander)),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Lifelink],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).once_per_batch(),
            effect: Effect::If {
                cond: Predicate::AttackedDefenderWithCountAtLeast {
                    who: PlayerRef::Triggerer,
                    defender: PlayerRef::You,
                    at_least: 2,
                    include_planeswalkers: false,
                },
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..legendary(creature(
            "Everett K. Ross, Hapless Attaché",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            1,
            3,
        ))
    }
}

/// Fight for the Throne — grow and fight; if the loser dies while you have
/// your commander, you become the monarch.
pub fn fight_for_the_throne() -> CardDefinition {
    spell(
        "Fight for the Throne",
        cost(&[generic(1), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            counter(target_filtered(yours(R::Creature)), Value::ONE),
            Effect::Fight {
                attacker: Selector::Target(0),
                defender: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) },
            },
            Effect::WhenTargetDiesThisTurn {
                body: Box::new(Effect::If {
                    cond: Predicate::ControlsOwnCommander { who: PlayerRef::You },
                    then: Box::new(you_become_monarch()),
                    else_: Box::new(Effect::Noop),
                }),
                slot: 1,
                filter: None,
            },
        ]),
    )
}

/// Hatut Zeraze Strike Force — commander storm; each copy may destroy an
/// artifact or enchantment.
pub fn hatut_zeraze_strike_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
                effect: Effect::CopySpell {
                    what: Selector::This,
                    count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
                },
            },
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) }),
            }),
        ],
        ..creature(
            "Hatut Zeraze Strike Force",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Warrior],
            2,
            2,
        )
    }
}

/// Heart-Shaped Herb — opposing damage to you is 1 less; it can re-deploy a
/// creature with three counters and crown you.
/// Residual: the creature is the auto-pick, and returns under your control.
pub fn heart_shaped_herb() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "If a source an opponent controls would deal damage to you, prevent 1 of that damage.",
            effect: StaticEffect::ReduceDamageToControllerFromSource { filter: R::ControlledByOpponent, amount: 1 },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::MaySacrifice {
                description: "Sacrifice a creature to return it with three +1/+1 counters?".into(),
                filter: R::Creature,
                count: Value::ONE,
                then: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::SacrificedThisResolution { filter: R::Creature },
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    },
                    counter(Selector::LastMoved, Value::Const(3)),
                    you_become_monarch(),
                ])),
                else_: None,
            },
            ..Default::default()
        }],
        ..spell("Heart-Shaped Herb", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Kimoyo Beads — each end step a bead not yet chosen: a card, two Soldiers,
/// or 3 life and a blink.
pub fn kimoyo_beads() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                    make(Value::Const(2), soldier()),
                    Effect::Seq(vec![
                        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                        Effect::ExileAndReturnToOwner { what: Selector::This },
                    ]),
                ],
            },
        }],
        ..spell("Kimoyo Beads", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// King Solomon's Frogs — cast, it exiles up to one big permanent of each
/// opponent, who each draw; later it can crown you.
pub fn king_solomons_frogs() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![with_filter(
            etb(Effect::ForEachOpponentTarget {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Permanent.and(R::ControlledByOpponent).and(R::ManaValueAtLeast(3)),
                    effect: Box::new(Effect::Seq(vec![
                        Effect::Draw {
                            who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                            amount: Value::ONE,
                        },
                        Effect::Exile { what: Selector::Target(0) },
                    ])),
                }),
            }),
            Predicate::SourceWasCast,
        )],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            exile_self_cost: true,
            effect: you_become_monarch(),
            ..Default::default()
        }],
        ..spell("King Solomon's Frogs", cost(&[generic(3), w()]), CardType::Artifact, Effect::Noop)
    }
}

/// Loyal Retainers — sacrifice before attacks to reanimate a legend.
pub fn loyal_retainers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::Any(vec![
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                    Predicate::CurrentStepIs(TurnStep::Draw),
                    Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                    Predicate::CurrentStepIs(TurnStep::BeginCombat),
                ]),
            ])),
            effect: Effect::Move {
                what: target_filtered(
                    R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Loyal Retainers", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Advisor], 1, 1)
    }
}

/// M'Baku, Jabari Chieftain — crowns an opponent when nobody is monarch;
/// creatures attacking the monarch get +1/+1 and trample.
pub fn mbaku_jabari_chieftain() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                    .with_filter(no_monarch()),
                effect: Effect::TargetPlayerThen {
                    filter: R::OpponentPlayer,
                    then: Box::new(Effect::BecomeMonarch { who: PlayerRef::Target(0) }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                    Predicate::PlayerIsOpponent { who: PlayerRef::DefendingPlayer },
                    Predicate::IsMonarch { who: PlayerRef::DefendingPlayer },
                ])),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: Selector::TriggerSource,
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: Selector::TriggerSource,
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
        ],
        ..legendary(creature(
            "M'Baku, Jabari Chieftain",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Warrior],
            4,
            3,
        ))
    }
}

/// Midnight Angel Armor — arrives on a fresh Soldier; +3/+3, flying,
/// vigilance.
pub fn midnight_angel_armor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            make(Value::ONE, soldier()),
            Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
        ]))],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 3,
            keywords: vec![Keyword::Flying, Keyword::Vigilance],
            ..Default::default()
        }),
        ..artifact_sub("Midnight Angel Armor", cost(&[generic(3), w(), w()]), ArtifactSubtype::Equipment)
    }
}

/// N'Yami-Class Mother Ship — a hit looks at the top card: a permanent may
/// deploy, anything else goes to hand.
pub fn nyami_class_mother_ship() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    let to_hand = || Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste, Keyword::Crew(3)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::Permanent },
                then: Box::new(Effect::MayDoElse {
                    description: "Put the top card onto the battlefield?".into(),
                    body: Box::new(Effect::Move {
                        what: top(),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                    else_: Box::new(to_hand()),
                }),
                else_: Box::new(to_hand()),
            },
        }],
        ..vehicle("N'Yami-Class Mother Ship", cost(&[generic(6)]), 5, 7, 3)
    }
}

/// Nakia, Wakandan Operative — your commander's entry crowns you; grows a
/// creature or Vehicle at sorcery speed.
pub fn nakia_wakandan_operative() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander.and(R::OwnedByYou) },
            ),
            effect: you_become_monarch(),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: counter(
                target_filtered(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))),
                Value::Const(2),
            ),
            ..Default::default()
        }],
        ..legendary(creature(
            "Nakia, Wakandan Operative",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            3,
            3,
        ))
    }
}

/// Okoye, Mighty and Adored — crowns you; each combat a counter, and that
/// creature double-strikes with trample into the monarch.
pub fn okoye_mighty_and_adored() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(you_become_monarch()),
            begin_combat_on_your_turn(Effect::Seq(vec![
                counter(target_filtered(R::Creature), Value::ONE),
                Effect::GrantTriggeredAbility {
                    what: Selector::Target(0),
                    trigger: Box::new(TriggeredAbility {
                        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                            .with_filter(Predicate::IsMonarch { who: PlayerRef::DefendingPlayer }),
                        effect: Effect::Seq(vec![
                            Effect::GrantKeyword {
                                what: Selector::This,
                                keyword: Keyword::DoubleStrike,
                                duration: Duration::EndOfTurn,
                            },
                            Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                        ]),
                    }),
                    duration: Duration::EndOfTurn,
                },
            ])),
        ],
        ..legendary(creature(
            "Okoye, Mighty and Adored",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            3,
            3,
        ))
    }
}

/// Panther Habit — damage to the equipped creature becomes +1/+1 counters.
/// Residual: a replacement, so unpreventable damage converts too.
pub fn panther_habit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        static_abilities: vec![StaticAbility {
            description: "If equipped creature would be dealt damage, prevent that damage and put that many +1/+1 counters on it.",
            effect: StaticEffect::ReplaceDamageToAttachedWithCounters { kind: CounterType::PlusOnePlusOne },
        }],
        equipped_bonus: Some(EquipBonus::default()),
        ..artifact_sub("Panther Habit", cost(&[generic(4)]), ArtifactSubtype::Equipment)
    }
}

/// Panther Robot — affinity for artifacts; an 8/8 with reach and trample.
pub fn panther_robot() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Reach, Keyword::Trample],
        affinity_filter: Some(R::Artifact),
        ..creature("Panther Robot", cost(&[generic(10)]), vec![CreatureType::Cat, CreatureType::Robot], 8, 8)
    }
}

/// Queen Mother Ramonda — crowns you; while you're the monarch, small
/// creatures can't attack you.
pub fn queen_mother_ramonda() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(you_become_monarch())],
        static_abilities: vec![StaticAbility {
            description: "As long as you're the monarch, creatures with power 2 or less can't attack you.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::IsMonarch { who: PlayerRef::You },
                inner: Box::new(StaticEffect::CreaturesCantAttackController {
                    protect_planeswalkers: false,
                    filter: Some(R::PowerAtMost(2)),
                }),
            },
        }],
        ..legendary(creature(
            "Queen Mother Ramonda",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Noble],
            3,
            5,
        ))
    }
}

/// Royal Talon Fighter Jet — enters with X counters; a Soldier per counter
/// when it enters or attacks.
pub fn royal_talon_fighter_jet() -> CardDefinition {
    let soldiers = || {
        make(
            Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
            soldier(),
        )
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        triggered_abilities: vec![etb(soldiers()), on_attack(soldiers())],
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        ..vehicle("Royal Talon Fighter Jet", cost(&[x(), w(), w()]), 1, 1, 2)
    }
}

/// Scourglass — in your upkeep, destroy everything but artifacts and lands.
pub fn scourglass() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
            ])),
            effect: Effect::Destroy {
                what: Selector::EachPermanent(R::Not(Box::new(R::Artifact.or(R::Land)))),
            },
            ..Default::default()
        }],
        ..spell("Scourglass", cost(&[generic(3), w(), w()]), CardType::Artifact, Effect::Noop)
    }
}

/// Shuri's Fabricator — two Vibranium; rebuilds an artifact with a finality
/// counter.
pub fn shuris_fabricator() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(Value::Const(2), vibranium(true)))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Finality, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..spell("Shuri's Fabricator", cost(&[generic(4)]), CardType::Artifact, Effect::Noop)
    }
}

/// Shuri, the Black Panther — attacks draw with three artifacts and pump the
/// team with six.
pub fn shuri_the_black_panther() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![on_attack(Effect::Seq(vec![
            Effect::If {
                cond: controls_at_least(yours(R::Artifact), 3),
                then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                else_: Box::new(Effect::Noop),
            },
            Effect::If {
                cond: controls_at_least(yours(R::Artifact), 6),
                then: Box::new(Effect::PumpPT {
                    what: Selector::EachPermanent(yours(R::Creature)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..legendary(creature(
            "Shuri, the Black Panther",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Hero],
            2,
            3,
        ))
    }
}

/// Storm, Queen of Wakanda — lends flight and her power to an attacker;
/// punishes fliers attacking you.
pub fn storm_queen_of_wakanda() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            on_attack(Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::PowerOf(Box::new(Selector::This)),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasKeyword(Keyword::Flying).and(R::IsAttackingYou),
                    },
                ),
                effect: Effect::DealDamage {
                    to: Selector::TriggerSource,
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            },
        ],
        ..legendary(creature(
            "Storm, Queen of Wakanda",
            cost(&[generic(3), g(), w()]),
            vec![CreatureType::Mutant, CreatureType::Noble, CreatureType::Hero],
            4,
            5,
        ))
    }
}

/// T'Chaka, Venerable King — mills three and keeps an artifact or land; from
/// the graveyard, crowns you while your commander is out.
pub fn tchaka_venerable_king() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            Effect::MoveChosen {
                from: Selector::LastMoved,
                filter: Some(R::Artifact.or(R::Land)),
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            from_graveyard: true,
            exile_self_cost: true,
            condition: Some(Predicate::ControlsOwnCommander { who: PlayerRef::You }),
            effect: you_become_monarch(),
            ..Default::default()
        }],
        ..legendary(creature(
            "T'Chaka, Venerable King",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Noble, CreatureType::Hero],
            2,
            2,
        ))
    }
}

/// The Great Mound — {C}; makes Vibranium for {3}; draws for {6}.
pub fn the_great_mound() -> CardDefinition {
    CardDefinition {
        name: "The Great Mound",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility { mana_cost: cost(&[generic(3)]), tap_cost: true, effect: tapped_vibranium(), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(6)]),
                tap_cost: true,
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Spear of Bashenga — crowns you if nobody is; +2/+2 and vigilance; an
/// attack on the monarch destroys one of their tapped permanents.
pub fn the_spear_of_bashenga() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        triggered_abilities: vec![with_filter(etb(you_become_monarch()), no_monarch())],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Vigilance],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                    .with_filter(Predicate::IsMonarch { who: PlayerRef::DefendingPlayer }),
                effect: Effect::Destroy {
                    what: target_filtered(R::Tapped.and(R::Nonland).and(R::ControlledByDefendingPlayer)),
                },
            }],
            ..Default::default()
        }),
        ..artifact_sub("The Spear of Bashenga", cost(&[generic(4), w()]), ArtifactSubtype::Equipment)
    }
}

/// Vibranium Mining Mech — Vibranium on entry and attack; firebreathing
/// power.
pub fn vibranium_mining_mech() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Crew(2)],
        triggered_abilities: vec![etb(tapped_vibranium()), on_attack(tapped_vibranium())],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..vehicle("Vibranium Mining Mech", cost(&[generic(4)]), 6, 6, 2)
    }
}

/// Vibranium Strike Gauntlets — flashes onto a creature: +3/+0, trample, and
/// a card per hit.
pub fn vibranium_strike_gauntlets() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Attach { what: Selector::This, to: target_filtered(yours(R::Creature)) })],
        equipped_bonus: Some(EquipBonus {
            power: 3,
            keywords: vec![Keyword::Trample],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            }],
            ..Default::default()
        }),
        ..artifact_sub("Vibranium Strike Gauntlets", cost(&[generic(4)]), ArtifactSubtype::Equipment)
    }
}

/// W'Kabi, Shield of the Nation — attacking with your commander beside a big
/// artifact makes a 4/4 Rhino.
pub fn wkabi_shield_of_the_nation() -> CardDefinition {
    let rhino = TokenDefinition {
        name: "Rhino".into(),
        power: 4,
        toughness: 4,
        colors: vec![Color::Green],
        keywords: vec![Keyword::Trample],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rhino], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: R::IsCommander },
                Predicate::SelectorExists(Selector::EachPermanent(yours(R::Artifact.and(R::ManaValueAtLeast(4))))),
            ])),
            effect: make(Value::ONE, rhino),
        }],
        ..legendary(creature(
            "W'Kabi, Shield of the Nation",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            2,
            3,
        ))
    }
}

/// Wakanda Forever! — reveal six: a permanent enters indestructible, another
/// goes to hand. Residual: the picks are the highest mana values.
pub fn wakanda_forever() -> CardDefinition {
    spell(
        "Wakanda Forever!",
        cost(&[generic(4), g(), g()]),
        CardType::Sorcery,
        Effect::RevealDeployOneTakeOne { count: Value::Const(6), indestructible: true },
    )
}

/// Zuri, Warrior of Wakanda — each big artifact spell grows your team.
pub fn zuri_warrior_of_wakanda() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            effect: counter(Selector::EachPermanent(yours(R::Creature)), Value::ONE),
            ..big_artifact_cast()
        }],
        ..legendary(creature(
            "Zuri, Warrior of Wakanda",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            2,
            2,
        ))
    }
}
