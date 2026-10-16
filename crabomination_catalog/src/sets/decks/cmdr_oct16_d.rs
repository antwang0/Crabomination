//! Commander: the 2026-10-16 rescan's short-text three- and four-card gaps
//! (Skeleton Ship, Reveka, Altanak, Achilles/Ezio, Beorn, Galion, Belzenlok).
//! Tests in `tests/recent_b/cmdr_oct16_d.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StateTriggeredAbility,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::shortcut::{on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, u};

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

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn your_creature() -> Selector {
    target_filtered(R::Creature.and(R::ControlledByYou))
}

fn their_creature() -> Selector {
    Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByOpponent) }
}

fn counters(what: Selector, n: Value) -> Effect {
    Effect::AddCounter { what, kind: CounterType::PlusOnePlusOne, amount: n }
}

/// Ferocious — "you control a creature with power 4 or greater".
fn ferocious() -> Predicate {
    Predicate::SelectorExists(Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::PowerAtLeast(4))))
}

// ── Skeleton Ship ───────────────────────────────────────────────────────────

/// Skeleton Ship — {3}{U}{B} legendary 0/3 Skeleton. When you control no
/// Islands, sacrifice it (a CR 603.8 state trigger). {T}: a -1/-1 counter on
/// target creature.
pub fn skeleton_ship() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        state_trigger: Some(StateTriggeredAbility {
            condition: Predicate::Not(Box::new(Predicate::SelectorExists(Selector::EachPermanent(
                R::HasLandType(LandType::Island).and(R::ControlledByYou),
            )))),
            effect: Effect::SacrificeSource,
        }),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Skeleton Ship", cost(&[generic(3), u(), b()]), vec![CreatureType::Skeleton], 0, 3)
    }
}

/// Cinderbones — {2}{B} 1/1 wither Elemental Skeleton. {1}{B}: regenerate.
pub fn cinderbones() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Wither],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Cinderbones", cost(&[generic(2), b()]), vec![CreatureType::Elemental, CreatureType::Skeleton], 1, 1)
    }
}

// ── Reveka, Wizard Savant ───────────────────────────────────────────────────

/// "{T}: 1 damage to any target", optionally gated to "only during your turn,
/// before attackers are declared".
fn pinger(name: &'static str, types: Vec<CreatureType>, before_attackers: bool) -> CardDefinition {
    let condition = before_attackers.then(|| {
        Predicate::All(vec![
            Predicate::IsTurnOf(PlayerRef::You),
            Predicate::Any(vec![
                Predicate::CurrentStepIs(TurnStep::Untap),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
                Predicate::CurrentStepIs(TurnStep::Draw),
                Predicate::CurrentStepIs(TurnStep::PreCombatMain),
                Predicate::CurrentStepIs(TurnStep::BeginCombat),
            ]),
        ])
    });
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(name, cost(&[generic(2), u()]), types, 1, 1)
    }
}

/// Apprentice Sorcerer — {2}{U} 1/1. {T}: 1 damage to any target, only during
/// your turn before attackers are declared.
pub fn apprentice_sorcerer() -> CardDefinition {
    pinger("Apprentice Sorcerer", vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer], true)
}

/// Capricious Sorcerer — Apprentice Sorcerer's reprint shape.
pub fn capricious_sorcerer() -> CardDefinition {
    pinger("Capricious Sorcerer", vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer], true)
}

/// Zuran Spellcaster — {2}{U} 1/1. {T}: 1 damage to any target.
pub fn zuran_spellcaster() -> CardDefinition {
    pinger("Zuran Spellcaster", vec![CreatureType::Human, CreatureType::Wizard], false)
}

/// Soul Barrier — {2}{U} enchantment. An opponent casting a creature spell
/// takes 2 from it unless they pay {2}.
pub fn soul_barrier() -> CardDefinition {
    CardDefinition {
        name: "Soul Barrier",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::MayPayBy {
                who: PlayerRef::Triggerer,
                description: "Pay {2} to avoid 2 damage from Soul Barrier?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Noop),
                else_: Some(Box::new(Effect::DealDamage { to: Selector::Player(PlayerRef::Triggerer), amount: Value::Const(2) })),
            },
        }],
        ..Default::default()
    }
}

// ── Altanak, the Thrice-Called ──────────────────────────────────────────────

/// Cloak of the Bat — {2} Equipment: flying and haste. Equip {2}.
pub fn cloak_of_the_bat() -> CardDefinition {
    CardDefinition {
        name: "Cloak of the Bat",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying, Keyword::Haste], ..Default::default() }),
        ..Default::default()
    }
}

/// Guardian of the Halls — {1}{G} 2/2 trample Elf Soldier. {5}{G}{G}: three
/// +1/+1 counters on it.
pub fn guardian_of_the_halls() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            effect: counters(Selector::This, Value::Const(3)),
            ..Default::default()
        }],
        ..creature("Guardian of the Halls", cost(&[generic(1), g()]), vec![CreatureType::Elf, CreatureType::Soldier], 2, 2)
    }
}

// ── Achilles Davenport / Ezio, Blade of Vengeance ───────────────────────────

/// Assassin Den — {3}{U} 2/4 defender Wall. {3}{U}, sorcery speed: a +1/+1
/// counter on target creature you control; it can't be blocked this turn.
pub fn assassin_den() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                counters(your_creature(), Value::ONE),
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Unblockable, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Assassin Den", cost(&[generic(3), u()]), vec![CreatureType::Wall], 2, 4)
    }
}

/// Hired Blade — {2}{B} 3/2 flash Human Assassin.
pub fn hired_blade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature("Hired Blade", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Assassin], 3, 2)
    }
}

/// Tranquilize — {1}{U} sorcery. Tap target creature an opponent controls and
/// put three stun counters on it.
pub fn tranquilize() -> CardDefinition {
    spell(
        "Tranquilize",
        cost(&[generic(1), u()]),
        false,
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::AddCounter { what: Selector::Target(0), kind: CounterType::Stun, amount: Value::Const(3) },
        ]),
    )
}

// ── Beorn, Reluctant Host / Galion, Elvenking's Butler ──────────────────────

/// Nasty Little Rabbit — {G} 1/2 Rabbit. Ferocious — at the beginning of combat
/// on your turn, if you control a creature with power 4 or greater, a +1/+1
/// counter on it.
pub fn nasty_little_rabbit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer).with_filter(ferocious()),
            effect: counters(Selector::This, Value::ONE),
        }],
        ..creature("Nasty Little Rabbit", cost(&[g()]), vec![CreatureType::Rabbit], 1, 2)
    }
}

/// Wargling — {1}{G} 2/2 Wolf. Ferocious — attacking while you control a
/// creature with power 4 or greater: +1/+0 and your creatures gain trample
/// until end of turn.
pub fn wargling() -> CardDefinition {
    let mut trigger = on_attack(Effect::Seq(vec![
        Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
        Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Trample,
            duration: Duration::EndOfTurn,
        },
    ]));
    trigger.event = trigger.event.with_filter(ferocious());
    CardDefinition { triggered_abilities: vec![trigger], ..creature("Wargling", cost(&[generic(1), g()]), vec![CreatureType::Wolf], 2, 2) }
}

/// Quarrel — {1}{G} instant. Target creature you control deals damage equal to
/// its power to target creature an opponent controls.
pub fn quarrel() -> CardDefinition {
    spell(
        "Quarrel",
        cost(&[generic(1), g()]),
        true,
        Effect::DealDamageEqualToPower { source: your_creature(), target: their_creature() },
    )
}

/// Soul's Might — {4}{G} sorcery. X +1/+1 counters on target creature, X its
/// power.
pub fn souls_might() -> CardDefinition {
    spell(
        "Soul's Might",
        cost(&[generic(4), g()]),
        false,
        counters(target_filtered(R::Creature), Value::PowerOf(Box::new(Selector::Target(0)))),
    )
}

/// Troll Negotiations — {2}{G}{G} sorcery. Two +1/+1 counters on target
/// creature you control, then it fights target creature an opponent controls.
pub fn troll_negotiations() -> CardDefinition {
    spell(
        "Troll Negotiations",
        cost(&[generic(2), g(), g()]),
        false,
        Effect::Seq(vec![
            counters(your_creature(), Value::Const(2)),
            Effect::Fight { attacker: Selector::Target(0), defender: their_creature() },
        ]),
    )
}

// ── Demonlord Belzenlok ─────────────────────────────────────────────────────

/// Ebon Stronghold — land, enters tapped. {T}: {B}. {T}, sacrifice it: {B}{B}.
pub fn ebon_stronghold() -> CardDefinition {
    CardDefinition {
        name: "Ebon Stronghold",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Black),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Black, Color::Black]) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Phantasmagorian — {5}{B}{B} 6/6 Horror. Cast: any player may discard three
/// cards; if one does, counter it. Discard three cards: return it from your
/// graveyard to your hand.
pub fn phantasmagorian() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::AnyPlayerMayAccept {
                who: PlayerRef::EachPlayer,
                prompt: "Discard three cards to counter Phantasmagorian?".into(),
                accepted: Box::new(Effect::If {
                    cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::AcceptingPlayer), Value::Const(3)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Discard { who: Selector::Player(PlayerRef::AcceptingPlayer), amount: Value::Const(3), random: false },
                        Effect::CounterSpell { what: Selector::TriggerSource },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
                otherwise: Box::new(Effect::Noop),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            discard_cost: Some((R::Any, 3)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..creature("Phantasmagorian", cost(&[generic(5), b(), b()]), vec![CreatureType::Horror], 6, 6)
    }
}

/// Soul Spike — {5}{B}{B} instant; or exile two black cards from your hand
/// instead of its mana cost. 4 damage to any target and you gain 4 life.
pub fn soul_spike() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            exile_filter: Some(R::HasColor(Color::Black)),
            exile_filter_extra: 1,
            ..Default::default()
        }),
        ..spell(
            "Soul Spike",
            cost(&[generic(5), b(), b()]),
            true,
            Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::Const(4) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
            ]),
        )
    }
}
