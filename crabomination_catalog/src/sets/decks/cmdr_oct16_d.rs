//! Commander: the 2026-10-16 rescan's short-text three- and four-card gaps
//! (Skeleton Ship, Reveka, Altanak, Achilles/Ezio, Beorn, Galion, Belzenlok).
//! Tests in `tests/recent_b/cmdr_oct16_d.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, StateTriggeredAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use std::sync::Arc;
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::shortcut::{etb, on_attack, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};

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

// ── Three Dog, Galaxy News DJ / Cass, Hand of Vengeance ─────────────────────

fn token(name: &str, types: Vec<CreatureType>, keyword: Keyword) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords: vec![keyword],
        ..Default::default()
    })
}

fn make(t: Arc<TokenDefinition>) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: t }
}

fn spirit() -> Arc<TokenDefinition> {
    token("Spirit", vec![CreatureType::Spirit], Keyword::Flying)
}

/// A white Aura: enchant `host`, the bonus on the enchanted creature.
fn aura(name: &'static str, mana: ManaCost, host: R, cartouche: bool, bonus: EquipBonus) -> CardDefinition {
    let mut subtypes = vec![EnchantmentSubtype::Aura];
    if cartouche {
        subtypes.push(EnchantmentSubtype::Cartouche);
    }
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: subtypes, ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// Cartouche of Solidarity — {W} Aura Cartouche, enchant creature you
/// control: +1/+1 and first strike. Enters: a 1/1 white vigilance Warrior.
pub fn cartouche_of_solidarity() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(token("Warrior", vec![CreatureType::Warrior], Keyword::Vigilance)))],
        ..aura(
            "Cartouche of Solidarity",
            cost(&[w()]),
            R::Creature.and(R::ControlledByYou),
            true,
            EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::FirstStrike], ..Default::default() },
        )
    }
}

/// Nurturing Presence — {1}{W} Aura. Enchanted creature has "a creature you
/// control entering gives this +1/+1 until end of turn". Enters: a 1/1 white
/// flying Spirit.
pub fn nurturing_presence() -> CardDefinition {
    let grown = TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
    };
    CardDefinition {
        triggered_abilities: vec![etb(make(spirit()))],
        ..aura(
            "Nurturing Presence",
            cost(&[generic(1), w()]),
            R::Creature,
            false,
            EquipBonus { triggered_abilities: vec![grown], ..Default::default() },
        )
    }
}

/// Squire's Devotion — {2}{W} Aura: +1/+1 and lifelink. Enters: a 1/1 white
/// lifelink Vampire.
pub fn squires_devotion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(token("Vampire", vec![CreatureType::Vampire], Keyword::Lifelink)))],
        ..aura(
            "Squire's Devotion",
            cost(&[generic(2), w()]),
            R::Creature,
            false,
            EquipBonus { power: 1, toughness: 1, keywords: vec![Keyword::Lifelink], ..Default::default() },
        )
    }
}

/// Valor of the Worthy — {W} Aura: +1/+1. The enchanted creature leaving the
/// battlefield makes a 1/1 white flying Spirit.
pub fn valor_of_the_worthy() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::EnchantedBySource),
            effect: make(spirit()),
        }],
        ..aura("Valor of the Worthy", cost(&[w()]), R::Creature, false, EquipBonus { power: 1, toughness: 1, ..Default::default() })
    }
}

// ── Ohabi Caleria ───────────────────────────────────────────────────────────

/// Arbalest Elite — {2}{W}{W} 2/3 Human Archer. {2}{W}, {T}: 3 damage to target
/// attacking or blocking creature; it doesn't untap during your next untap step.
pub fn arbalest_elite() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))), amount: Value::Const(3) },
                Effect::SkipNextUntap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Arbalest Elite", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Archer], 2, 3)
    }
}

/// Palazzo Archers — {2}{G} 2/3 reach Human Archer. A flier attacking you or
/// your planeswalker takes damage equal to this creature's power.
pub fn palazzo_archers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasKeyword(Keyword::Flying),
            }),
            effect: Effect::DealDamage { to: Selector::TriggerSource, amount: Value::PowerOf(Box::new(Selector::This)) },
        }],
        ..creature("Palazzo Archers", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Archer], 2, 3)
    }
}

/// Skyway Sniper — {G} 1/2 reach Elf Archer. {2}{G}: 1 damage to target
/// creature with flying.
pub fn skyway_sniper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::DealDamage { to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Skyway Sniper", cost(&[g()]), vec![CreatureType::Elf, CreatureType::Archer], 1, 2)
    }
}

/// Poison the Blade — {1}{G} instant. Target creature gains deathtouch until
/// end of turn. Draw a card.
pub fn poison_the_blade() -> CardDefinition {
    spell(
        "Poison the Blade",
        cost(&[generic(1), g()]),
        true,
        Effect::Seq(vec![
            Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
            Effect::Draw { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

// ── Quicksilver, Brash Blur ─────────────────────────────────────────────────

/// Mirran Banesplitter — {R} flash Equipment: +2/+0. Enters: attach to target
/// creature you control. Equip {3}.
pub fn mirran_banesplitter() -> CardDefinition {
    CardDefinition {
        name: "Mirran Banesplitter",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![etb(Effect::Attach { what: Selector::This, to: your_creature() })],
        equipped_bonus: Some(EquipBonus { power: 2, ..Default::default() }),
        ..Default::default()
    }
}

/// Intimidator Initiate — {R} 1/1 Goblin Shaman. A player casting a red spell:
/// you may pay {1}; if you do, target creature can't block this turn.
pub fn intimidator_initiate() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::HasColor(Color::Red) }),
            effect: Effect::MayPay {
                description: "Pay {1}: target creature can't block this turn?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::CantBlock,
                    duration: Duration::EndOfTurn,
                }),
                else_: None,
            },
        }],
        ..creature("Intimidator Initiate", cost(&[r()]), vec![CreatureType::Goblin, CreatureType::Shaman], 1, 1)
    }
}

/// Ground Rift — {R} sorcery, storm. Target creature without flying can't
/// block this turn.
pub fn ground_rift() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Storm],
        ..spell(
            "Ground Rift",
            cost(&[r()]),
            false,
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying))))),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        )
    }
}

// ── Arni, Renowned Champion ─────────────────────────────────────────────────

/// Arni, Renowned Champion — {3}{R} legendary 1/5 trample Human Berserker.
/// Another creature you control entering: +X/+0 until end of turn, X its power.
pub fn arni_renowned_champion() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::Creature.and(R::OtherThanSource),
            }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::PowerOf(Box::new(Selector::TriggerSource)),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Arni, Renowned Champion", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Berserker], 1, 5)
    }
}

/// Nova Chaser — {3}{R} 10/2 trample Elemental Warrior; champion an Elemental.
pub fn nova_chaser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::Champion { filter: R::HasCreatureType(CreatureType::Elemental) })],
        ..creature("Nova Chaser", cost(&[generic(3), r()]), vec![CreatureType::Elemental, CreatureType::Warrior], 10, 2)
    }
}

/// Rakka Mar — {2}{R}{R} legendary 2/2 haste Human Shaman. {R}, {T}: a 3/1 red
/// haste Elemental.
pub fn rakka_mar() -> CardDefinition {
    let elemental = TokenDefinition {
        name: "Elemental".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r()]),
            effect: make(Arc::new(elemental)),
            ..Default::default()
        }],
        ..creature("Rakka Mar", cost(&[generic(2), r(), r()]), vec![CreatureType::Human, CreatureType::Shaman], 2, 2)
    }
}

// ── Sima Yi, Wei Field Marshal ──────────────────────────────────────────────

fn swamps() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(R::HasLandType(LandType::Swamp).and(R::ControlledByYou))))
}

/// A `*/4` whose power is the Swamps you control (CR 604.3).
fn swamp_power(name: &'static str, types: Vec<CreatureType>) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature's power is equal to the number of Swamps you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: swamps(), toughness: Value::Const(4) },
        }],
        ..creature(name, cost(&[generic(5), b()]), types, 0, 4)
    }
}

/// Sima Yi, Wei Field Marshal — {5}{B} legendary */4 Human Soldier.
pub fn sima_yi_wei_field_marshal() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..swamp_power("Sima Yi, Wei Field Marshal", vec![CreatureType::Human, CreatureType::Soldier])
    }
}

/// Dakmor Sorceress — {5}{B} */4 Human Wizard Sorcerer.
pub fn dakmor_sorceress() -> CardDefinition {
    swamp_power("Dakmor Sorceress", vec![CreatureType::Human, CreatureType::Wizard, CreatureType::Sorcerer])
}

/// Nightmare Incursion — {5}{B} sorcery. Search target player's library for up
/// to X cards, X the Swamps you control, and exile them.
pub fn nightmare_incursion() -> CardDefinition {
    spell(
        "Nightmare Incursion",
        cost(&[generic(5), b()]),
        false,
        Effect::SearchUpToN { who: PlayerRef::Target(0), filter: R::Any, to: ZoneDest::Exile, count: swamps() },
    )
}

// ── Llawan, Cephalid Empress / Blind Seer ───────────────────────────────────

/// Indigo Faerie — {1}{U} 1/1 flying Faerie Wizard. {U}: target permanent
/// becomes blue in addition to its other colors until end of turn.
pub fn indigo_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::BecomeColor {
                what: target_filtered(R::Permanent),
                colors: vec![Color::Blue],
                duration: Duration::EndOfTurn,
                additive: true,
            },
            ..Default::default()
        }],
        ..creature("Indigo Faerie", cost(&[generic(1), u()]), vec![CreatureType::Faerie, CreatureType::Wizard], 1, 1)
    }
}

fn color_word_swap() -> Effect {
    Effect::ReplaceColorWord { what: target_filtered(R::Permanent), duration: Duration::Permanent }
}

/// Glamerdye — {1}{U} instant, retrace. Change the text of target permanent by
/// replacing one color word with another (a spell can't be targeted here).
pub fn glamerdye() -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Retrace], ..spell("Glamerdye", cost(&[generic(1), u()]), true, color_word_swap()) }
}

/// Sleight of Mind — {U} instant. Glamerdye's text change, without retrace.
pub fn sleight_of_mind() -> CardDefinition {
    spell("Sleight of Mind", cost(&[u()]), true, color_word_swap())
}

/// Homarid Shaman — {2}{U}{U} 2/1 Homarid Shaman. {U}: tap target green
/// creature.
pub fn homarid_shaman() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Tap { what: target_filtered(R::Creature.and(R::HasColor(Color::Green))) },
            ..Default::default()
        }],
        ..creature("Homarid Shaman", cost(&[generic(2), u(), u()]), vec![CreatureType::Homarid, CreatureType::Shaman], 2, 1)
    }
}

/// Wrath of Marit Lage — {3}{U}{U} enchantment. Enters: tap all red creatures.
/// Red creatures don't untap during their controllers' untap steps.
pub fn wrath_of_marit_lage() -> CardDefinition {
    let red = || R::Creature.and(R::HasColor(Color::Red));
    CardDefinition {
        name: "Wrath of Marit Lage",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(Effect::Tap { what: Selector::EachPermanent(red()) })],
        static_abilities: vec![StaticAbility {
            description: "Red creatures don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::EachPermanent(red()) },
        }],
        ..Default::default()
    }
}

// ── Bullseye, Death Dealer ──────────────────────────────────────────────────

/// Bullseye, Death Dealer — {2}{B/R} legendary 2/3 Human Assassin Villain.
/// Enters: you may sacrifice an artifact or discard a nonland card; when you
/// do, 2 damage to any target. {3}, {T}, sacrifice an artifact or discard a
/// nonland card: 2 damage to any target (the "or" is two abilities).
pub fn bullseye_death_dealer() -> CardDefinition {
    let shot = || Effect::DealDamage { to: target_any(), amount: Value::Const(2) };
    let reflex = || Box::new(Effect::ReflexiveTrigger { body: Box::new(shot()) });
    let gun = |sac: bool| ActivatedAbility {
        tap_cost: true,
        mana_cost: cost(&[generic(3)]),
        sac_other_filter: sac.then(|| (R::Artifact, 1)),
        discard_cost: (!sac).then(|| (R::Nonland, 1)),
        effect: shot(),
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::MaySacrifice {
                description: "Sacrifice an artifact for 2 damage?".into(),
                filter: R::Artifact.and(R::ControlledByYou),
                count: Value::ONE,
                then: reflex(),
                else_: None,
            },
            Effect::MayDiscardMatching {
                description: "Discard a nonland card for 2 damage?".into(),
                count: Value::ONE,
                filter: R::Nonland,
                then: reflex(),
                else_: None,
            },
        ]))],
        activated_abilities: vec![gun(true), gun(false)],
        ..creature(
            "Bullseye, Death Dealer",
            cost(&[generic(2), hybrid(Color::Black, Color::Red)]),
            vec![CreatureType::Human, CreatureType::Assassin, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Hell's Kitchen — land, enters tapped; enters: gain 1 life. {T}: {B} or {R}.
pub fn hells_kitchen() -> CardDefinition {
    CardDefinition {
        name: "Hell's Kitchen",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::ONE })],
        activated_abilities: vec![crate::sets::tap_add(Color::Black), crate::sets::tap_add(Color::Red)],
        ..Default::default()
    }
}

/// Dark Deed — {1}{B} instant. Target creature gets -4/-4 until end of turn.
pub fn dark_deed() -> CardDefinition {
    spell(
        "Dark Deed",
        cost(&[generic(1), b()]),
        true,
        Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(-4), toughness: Value::Const(-4), duration: Duration::EndOfTurn },
    )
}

// ── Gallia, Tragic Host ─────────────────────────────────────────────────────

/// Gallia, Tragic Host — {1}{B} legendary 2/1 menace Zombie Satyr. {4}{B},
/// exile another creature card from your graveyard: return her from your
/// graveyard to the battlefield tapped with a +1/+1 counter.
pub fn gallia_tragic_host() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Menace],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            from_graveyard: true,
            exile_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true } },
                counters(Selector::LastMoved, Value::ONE),
            ]),
            ..Default::default()
        }],
        ..creature("Gallia, Tragic Host", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Satyr], 2, 1)
    }
}

/// Ritual of the Machine — {2}{B}{B} sorcery; sacrifice a creature as an
/// additional cost. Gain control of target nonartifact, nonblack creature.
pub fn ritual_of_the_machine() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 }],
        ..spell(
            "Ritual of the Machine",
            cost(&[generic(2), b(), b()]),
            false,
            Effect::GainControl {
                what: target_filtered(
                    R::Creature.and(R::Not(Box::new(R::Artifact))).and(R::Not(Box::new(R::HasColor(Color::Black)))),
                ),
                to: None,
                duration: Duration::Permanent,
            },
        )
    }
}
