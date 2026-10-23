//! Commander: the 2026-10-16 rescan's short-text three- and four-card gaps
//! (Skeleton Ship, Reveka, Altanak, Achilles/Ezio, Beorn, Galion, Belzenlok).
//! Tests in `tests/recent_b/cmdr_oct16_d.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, Predicate, SelectionRequirement as R, StateTriggeredAbility,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, Zone,
};
use std::sync::Arc;
use crate::card::{StaticAbility, StaticEffect};
use crate::effect::shortcut::{etb, exalted, on_attack, target_any, target_filtered};
use crate::effect::{ActivationTrait, Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
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
        sac_other_filter: sac.then_some((R::Artifact, 1)),
        discard_cost: (!sac).then_some((R::Nonland, 1)),
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

// ── Nefarox, Overlord of Grixis ─────────────────────────────────────────────

/// Nefarox, Overlord of Grixis — {4}{B}{B} legendary 5/5 flying Demon,
/// exalted. Attacking alone: the defending player sacrifices a creature.
pub fn nefarox_overlord_of_grixis() -> CardDefinition {
    let mut alone = on_attack(Effect::Sacrifice {
        who: Selector::Player(PlayerRef::DefendingPlayer),
        count: Value::ONE,
        filter: R::Creature,
    });
    alone.event = alone.event.with_filter(Predicate::EntityMatches { what: Selector::This, filter: R::IsAttackingAlone });
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![exalted(), alone],
        ..creature("Nefarox, Overlord of Grixis", cost(&[generic(4), b(), b()]), vec![CreatureType::Demon], 5, 5)
    }
}

fn exalted_body(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32, more: Vec<Keyword>) -> CardDefinition {
    let keywords = more;
    CardDefinition { keywords, triggered_abilities: vec![exalted()], ..creature(name, mana, types, p, t) }
}

/// Duskmantle Prowler — {3}{B} 2/2 haste, exalted Vampire Rogue.
pub fn duskmantle_prowler() -> CardDefinition {
    exalted_body("Duskmantle Prowler", cost(&[generic(3), b()]), vec![CreatureType::Vampire, CreatureType::Rogue], 2, 2, vec![Keyword::Haste])
}

/// Knight of Infamy — {1}{B} 2/1 Human Knight, protection from white, exalted.
pub fn knight_of_infamy() -> CardDefinition {
    exalted_body(
        "Knight of Infamy",
        cost(&[generic(1), b()]),
        vec![CreatureType::Human, CreatureType::Knight],
        2,
        1,
        vec![Keyword::Protection(Color::White)],
    )
}

/// Servant of Nefarox — {2}{B} 3/1 exalted Human Cleric.
pub fn servant_of_nefarox() -> CardDefinition {
    exalted_body("Servant of Nefarox", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 1, vec![])
}

// ── Hythonia the Cruel ──────────────────────────────────────────────────────

/// Arisen Gorgon — {1}{B}{B} 3/3 Zombie Gorgon; deathtouch while you control a
/// Liliana planeswalker.
pub fn arisen_gorgon() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature has deathtouch as long as you control a Liliana planeswalker.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Deathtouch,
                condition: Predicate::SelectorExists(Selector::EachPermanent(
                    R::HasPlaneswalkerType(PlaneswalkerSubtype::Liliana).and(R::ControlledByYou),
                )),
            },
        }],
        ..creature("Arisen Gorgon", cost(&[generic(1), b(), b()]), vec![CreatureType::Zombie, CreatureType::Gorgon], 3, 3)
    }
}

/// Scourge of the Undercity — {1}{B} 2/1 lifelink Gorgon Vampire. Enters:
/// another target creature you control gains lifelink until end of turn.
pub fn scourge_of_the_undercity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![etb(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            keyword: Keyword::Lifelink,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Scourge of the Undercity", cost(&[generic(1), b()]), vec![CreatureType::Gorgon, CreatureType::Vampire], 2, 1)
    }
}

/// Vraska, Scheming Gorgon — {4}{B}{B} planeswalker, loyalty 5. +2: your
/// creatures get +1/+0. −3: destroy target creature. −10: your creatures gain
/// deathtouch and "damage to an opponent makes that player lose the game"
/// until end of turn.
pub fn vraska_scheming_gorgon() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    let lethal = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
            .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::Target(0) }),
        effect: Effect::LoseGame { who: PlayerRef::Target(0) },
    };
    CardDefinition {
        name: "Vraska, Scheming Gorgon",
        cost: cost(&[generic(4), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Vraska], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::PumpPT { what: yours(), power: Value::ONE, toughness: Value::ZERO, duration: Duration::EndOfTurn },
                ..Default::default()
            },
            LoyaltyAbility { loyalty_cost: -3, effect: Effect::Destroy { what: target_filtered(R::Creature) }, ..Default::default() },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword { what: yours(), keyword: Keyword::Deathtouch, duration: Duration::EndOfTurn },
                    Effect::GrantTriggeredAbility { what: yours(), trigger: Box::new(lethal), duration: Duration::EndOfTurn },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vraska's Scorn — {2}{B}{B} sorcery. Target opponent loses 4 life. You may
/// search your library and/or graveyard for Vraska, Scheming Gorgon and put it
/// into your hand.
pub fn vraskas_scorn() -> CardDefinition {
    spell(
        "Vraska's Scorn",
        cost(&[generic(2), b(), b()]),
        false,
        Effect::Seq(vec![
            Effect::LoseLife { who: target_filtered(R::OpponentPlayer), amount: Value::Const(4) },
            Effect::SearchZones {
                who: PlayerRef::You,
                zones: vec![Zone::Library, Zone::Graveyard],
                filter: R::HasName("Vraska, Scheming Gorgon".into()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
    )
}

// ── Telim'Tor ───────────────────────────────────────────────────────────────

/// Blazing Blade Askari — {2}{R} 2/2 flanking Human Knight. {2}: it becomes
/// colorless until end of turn.
pub fn blazing_blade_askari() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeColor { what: Selector::This, colors: vec![], duration: Duration::EndOfTurn, additive: false },
            ..Default::default()
        }],
        ..creature("Blazing Blade Askari", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Embereth Paladin — {3}{R} 4/1 haste Human Knight. Adamant — three red mana
/// spent: it enters with a +1/+1 counter.
pub fn embereth_paladin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred {
                pred: Box::new(Predicate::SourceCastWithColorSpent { color: Color::Red, at_least: 3 }),
                then: Box::new(Value::ONE),
                else_: Box::new(Value::Const(0)),
            },
        )),
        ..creature("Embereth Paladin", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Knight], 4, 1)
    }
}

/// Tilonalli's Skinshifter — {2}{R} 0/1 haste Human Shaman. Attacking: it
/// becomes a copy of another target nonlegendary attacking creature until end
/// of turn.
pub fn tilonallis_skinshifter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![on_attack(Effect::BecomeCopyOfFor {
            what: Selector::This,
            source: target_filtered(
                R::Creature.and(R::IsAttacking).and(R::OtherThanSource).and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary)))),
            ),
            duration: Duration::EndOfTurn,
            non_legendary: false,
        })],
        ..creature("Tilonalli's Skinshifter", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Shaman], 0, 1)
    }
}

/// Joust — {1}{R} sorcery. Target creature you control (+2/+1 first if it's a
/// Knight) fights target creature you don't control.
pub fn joust() -> CardDefinition {
    let theirs = Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::Not(Box::new(R::ControlledByYou))) };
    spell(
        "Joust",
        cost(&[generic(1), r()]),
        false,
        Effect::Seq(vec![
            Effect::If {
                cond: Predicate::EntityMatches { what: your_creature(), filter: R::HasCreatureType(CreatureType::Knight) },
                then: Box::new(Effect::PumpPT {
                    what: Selector::Target(0),
                    power: Value::Const(2),
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Fight { attacker: Selector::Target(0), defender: theirs },
        ]),
    )
}

// ── Depala, Pilot Exemplar ──────────────────────────────────────────────────

fn vehicle(name: &'static str, p: i32, t: i32, crew: u32, mut keywords: Vec<Keyword>) -> CardDefinition {
    keywords.push(Keyword::Crew(crew));
    CardDefinition {
        name,
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: p,
        toughness: t,
        keywords,
        ..Default::default()
    }
}

/// Fleetwheel Cruiser — {4} 5/3 trample, haste Vehicle, crew 2. Enters: it
/// becomes an artifact creature until end of turn.
pub fn fleetwheel_cruiser() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AnimateAsCreature { what: Selector::This, duration: Duration::EndOfTurn })],
        ..vehicle("Fleetwheel Cruiser", 5, 3, 2, vec![Keyword::Trample, Keyword::Haste])
    }
}

/// Untethered Express — {4} 4/4 trample Vehicle, crew 1. Attacking: a +1/+1
/// counter on it.
pub fn untethered_express() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(counters(Selector::This, Value::ONE))],
        ..vehicle("Untethered Express", 4, 4, 1, vec![Keyword::Trample])
    }
}

/// Aethershield Artificer — {3}{W} 3/3 Dwarf Artificer. At the beginning of
/// combat on your turn, target artifact creature you control gets +2/+2 and
/// indestructible until end of turn.
pub fn aethershield_artificer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Artifact.and(R::Creature).and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Aethershield Artificer", cost(&[generic(3), w()]), vec![CreatureType::Dwarf, CreatureType::Artificer], 3, 3)
    }
}

/// Renegade Wheelsmith — {1}{R}{W} 3/2 Dwarf Pilot. Becoming tapped: target
/// creature can't block this turn.
pub fn renegade_wheelsmith() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
        }],
        ..creature("Renegade Wheelsmith", cost(&[generic(1), r(), w()]), vec![CreatureType::Dwarf, CreatureType::Pilot], 3, 2)
    }
}

// ── Doc Ock, Evil Inventor ──────────────────────────────────────────────────

/// Doc Ock, Evil Inventor — {5}{U}{B} legendary 8/8 Human Scientist Villain. At
/// the beginning of combat on your turn, target noncreature artifact you
/// control becomes an 8/8 Robot Villain artifact creature (no duration).
pub fn doc_ock_evil_inventor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer),
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Artifact.and(R::Not(Box::new(R::Creature))).and(R::ControlledByYou)),
                power: Value::Const(8),
                toughness: Value::Const(8),
                creature_types: vec![CreatureType::Robot, CreatureType::Villain],
                keywords: vec![],
                duration: Duration::Permanent,
            },
        }],
        ..creature(
            "Doc Ock, Evil Inventor",
            cost(&[generic(5), u(), b()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            8,
            8,
        )
    }
}

/// Doc Ock, Sinister Scientist — {4}{U} legendary 4/5 Human Scientist Villain.
/// Base 8/8 with eight or more cards in your graveyard; hexproof while you
/// control another Villain.
pub fn doc_ock_sinister_scientist() -> CardDefinition {
    let big = || Box::new(Predicate::ValueAtLeast(Value::GraveyardSizeOf(PlayerRef::You), Value::Const(8)));
    let base = |n: i32, normal: i32| Value::IfPred { pred: big(), then: Box::new(Value::Const(n)), else_: Box::new(Value::Const(normal)) };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "As long as there are eight or more cards in your graveyard, Doc Ock has base power and toughness 8/8.",
                effect: StaticEffect::SelfBasePtFromValue { power: base(8, 4), toughness: base(8, 5) },
            },
            StaticAbility {
                description: "As long as you control another Villain, Doc Ock has hexproof.",
                effect: StaticEffect::SelfHasKeywordIf {
                    keyword: Keyword::Hexproof,
                    condition: Predicate::SelectorExists(Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Villain).and(R::ControlledByYou).and(R::OtherThanSource),
                    )),
                },
            },
        ],
        ..creature(
            "Doc Ock, Sinister Scientist",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Scientist, CreatureType::Villain],
            4,
            5,
        )
    }
}

/// Grasping Tentacles — {1}{U}{B} sorcery. Target opponent mills eight; you may
/// put an artifact card from their graveyard onto the battlefield under your
/// control.
pub fn grasping_tentacles() -> CardDefinition {
    spell(
        "Grasping Tentacles",
        cost(&[generic(1), u(), b()]),
        false,
        Effect::Seq(vec![
            Effect::Mill { who: target_filtered(R::OpponentPlayer), amount: Value::Const(8) },
            Effect::MoveChosen {
                from: Selector::CardsInZone { who: PlayerRef::Target(0), zone: Zone::Graveyard, filter: R::Artifact },
                filter: Some(R::Artifact),
                count: Value::ONE,
                up_to: true,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]),
    )
}

// ── Lord Dregg, Insect Invader ──────────────────────────────────────────────

/// Lord Dregg, Insect Invader — {3}{B} legendary 3/2 flying Insect Warrior.
/// Disappear — your end step, if a permanent left the battlefield under your
/// control this turn: a 1/1 black flying Insect Warrior. {3}{G}, sacrifice a
/// token: draw a card.
pub fn lord_dregg_insect_invader() -> CardDefinition {
    let insect = Arc::new(TokenDefinition {
        name: "Insect Warrior".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect, CreatureType::Warrior], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    });
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer)
                .with_filter(Predicate::RevoltActive { who: PlayerRef::You }),
            effect: make(insect),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            sac_other_filter: Some((R::IsToken, 1)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Lord Dregg, Insect Invader", cost(&[generic(3), b()]), vec![CreatureType::Insect, CreatureType::Warrior], 3, 2)
    }
}

/// Ice Cream Kitty — {1}{B/G} 1/3 Food Cat Mutant artifact creature. {2},
/// sacrifice another creature or token, sorcery speed: draw. {2}, {T},
/// sacrifice it: gain 3 life.
pub fn ice_cream_kitty() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Food],
            creature_types: vec![CreatureType::Cat, CreatureType::Mutant],
            ..Default::default()
        },
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                sorcery_speed: true,
                sac_other_filter: Some((R::Creature.or(R::IsToken).and(R::OtherThanSource), 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                ..Default::default()
            },
        ],
        ..creature("Ice Cream Kitty", cost(&[generic(1), hybrid(Color::Black, Color::Green)]), vec![], 1, 3)
    }
}

/// Mabel, Bitter Recluse — {B} legendary 1/1 deathtouch Mouse Warlock. Enters:
/// remove up to three counters from another target creature or planeswalker.
pub fn mabel_bitter_recluse() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::RemoveCountersUpTo {
            what: target_filtered(R::Creature.or(R::Planeswalker).and(R::OtherThanSource)),
            amount: Value::Const(3),
        })],
        ..creature("Mabel, Bitter Recluse", cost(&[b()]), vec![CreatureType::Mouse, CreatureType::Warlock], 1, 1)
    }
}

// ── Marvel Boy, Noh-Varr ────────────────────────────────────────────────────

/// "Whenever another creature you control enters, put a +1/+1 counter on
/// this" (Alliance).
fn alliance_counter() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: R::Creature.and(R::OtherThanSource),
        }),
        effect: counters(Selector::This, Value::ONE),
    }
}

/// Marvel Boy, Noh-Varr — {1}{R/W} legendary 1/1 Kree Soldier Hero. Another
/// creature of yours entering, and each power-up ability you activate: a
/// +1/+1 counter on it.
pub fn marvel_boy_noh_varr() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            alliance_counter(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::AbilityActivatedWith(ActivationTrait::PowerUp), EventScope::YourControl),
                effect: counters(Selector::This, Value::ONE),
            },
        ],
        ..creature(
            "Marvel Boy, Noh-Varr",
            cost(&[generic(1), hybrid(Color::Red, Color::White)]),
            vec![CreatureType::Kree, CreatureType::Soldier, CreatureType::Hero],
            1,
            1,
        )
    }
}

/// Brave Brawler — {1}{W} 2/1 lifelink Human Warrior Hero. Power-up — {4}{W}:
/// two +1/+1 counters on it.
pub fn brave_brawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), w()]),
            power_up: true,
            effect: counters(Selector::This, Value::Const(2)),
            ..Default::default()
        }],
        ..creature("Brave Brawler", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero], 2, 1)
    }
}

/// EPF Point Squad — {1}{R/W}{R/W} 2/1 Human Soldier. Alliance: a +1/+1
/// counter on it when another creature of yours enters.
pub fn epf_point_squad() -> CardDefinition {
    let rw = || hybrid(Color::Red, Color::White);
    CardDefinition {
        triggered_abilities: vec![alliance_counter()],
        ..creature("EPF Point Squad", cost(&[generic(1), rw(), rw()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    }
}

// ── Roshan, Hidden Magister ─────────────────────────────────────────────────

/// Qarsi High Priest — {B} 0/2 Human Cleric. {1}{B}, {T}, sacrifice another
/// creature: manifest the top card of your library.
pub fn qarsi_high_priest() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature.and(R::OtherThanSource), 1)),
            effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Qarsi High Priest", cost(&[b()]), vec![CreatureType::Human, CreatureType::Cleric], 0, 2)
    }
}

/// Sultai Emissary — {1}{B} 1/1 Zombie Warrior. Dies: manifest the top card.
pub fn sultai_emissary() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::Manifest { who: PlayerRef::You, amount: Value::ONE })],
        ..creature("Sultai Emissary", cost(&[generic(1), b()]), vec![CreatureType::Zombie, CreatureType::Warrior], 1, 1)
    }
}

/// Raise the Draugr — {1}{B} instant. Choose one: return target creature card
/// from your graveyard to your hand; or two target creature cards from your
/// graveyard that share a creature type.
pub fn raise_the_draugr() -> CardDefinition {
    let back = |what: Selector| Effect::Move { what, to: ZoneDest::Hand(PlayerRef::You) };
    let card = || R::Creature.from_your_graveyard();
    spell(
        "Raise the Draugr",
        cost(&[generic(1), b()]),
        true,
        Effect::ChooseMode(vec![
            back(target_filtered(card())),
            Effect::Seq(vec![
                back(target_filtered(card())),
                back(Selector::TargetFiltered {
                    slot: 1,
                    filter: card().and(R::SharesCreatureTypeWithTargetSlot(0)).and(R::OtherThanTargetSlot(0)),
                }),
            ]),
        ]),
    )
}

// ── Ghen, Arcanum Weaver ────────────────────────────────────────────────────

/// Ghen, Arcanum Weaver — {R}{W}{B} legendary 2/3 Human Wizard. {R}{W}{B},
/// {T}, sacrifice an enchantment: return target enchantment card from your
/// graveyard to the battlefield.
pub fn ghen_arcanum_weaver() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[r(), w(), b()]),
            sac_other_filter: Some((R::Enchantment, 1)),
            effect: Effect::Move {
                what: target_filtered(R::Enchantment.from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature("Ghen, Arcanum Weaver", cost(&[r(), w(), b()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 3)
    }
}

/// Demonic Lore — {2}{B} enchantment. Enters: draw three. Your end step: lose
/// 2 life for each card in your hand.
pub fn demonic_lore() -> CardDefinition {
    CardDefinition {
        name: "Demonic Lore",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
                effect: Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::Times(Box::new(Value::HandSizeOf(PlayerRef::You)), Box::new(Value::Const(2))),
                },
            },
        ],
        ..Default::default()
    }
}

/// Demonic Pact — {2}{B}{B} enchantment. Your upkeep: choose one that hasn't
/// been chosen — 4 damage to any target and gain 4; target opponent discards
/// two; draw two; you lose the game.
pub fn demonic_pact() -> CardDefinition {
    CardDefinition {
        name: "Demonic Pact",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::ChooseUnchosenMode {
                modes: vec![
                    Effect::Seq(vec![
                        Effect::DealDamage { to: target_any(), amount: Value::Const(4) },
                        Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
                    ]),
                    Effect::Discard { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2), random: false },
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::LoseGame { who: PlayerRef::You },
                ],
            },
        }],
        ..Default::default()
    }
}

/// Overwhelming Splendor — {6}{W}{W} Aura Curse, enchant player. Creatures the
/// enchanted player controls lose all abilities and have base 1/1; that player
/// can't activate abilities that aren't mana or loyalty abilities.
pub fn overwhelming_splendor() -> CardDefinition {
    let theirs = || Selector::ControlledBy { who: PlayerRef::EnchantedPlayer, filter: R::Creature };
    CardDefinition {
        name: "Overwhelming Splendor",
        cost: cost(&[generic(6), w(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        static_abilities: vec![
            StaticAbility {
                description: "Creatures enchanted player controls lose all abilities.",
                effect: StaticEffect::MatchingLoseAllAbilities { applies_to: theirs() },
            },
            StaticAbility {
                description: "Creatures enchanted player controls have base power and toughness 1/1.",
                effect: StaticEffect::SetBasePtForFilter { applies_to: theirs(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Enchanted player can't activate abilities that aren't mana abilities or loyalty abilities.",
                effect: StaticEffect::EnchantedPlayerCantActivateNonManaAbilities,
            },
        ],
        ..Default::default()
    }
}

// ── Ant-Man, Elusive Avenger / Fin Fang Foom ────────────────────────────────

fn instant_or_sorcery() -> R {
    R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))
}

/// Spinerock Tyrant — {3}{R}{R} 6/6 flying, wither Dragon. Casting an instant
/// or sorcery with a single target: you may copy it; if you do, both gain
/// wither (granted before the copy, which copies it).
pub fn spinerock_tyrant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Wither],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(instant_or_sorcery()),
                Predicate::CastSpellTargetsOnlyOneMatching(R::Any),
            ])),
            effect: Effect::MayDo {
                description: "Copy that spell? Both gain wither.".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::SpellGainsKeyword { what: Selector::TriggerSource, keyword: Keyword::Wither },
                    Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
                ])),
            },
        }],
        ..creature("Spinerock Tyrant", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 6, 6)
    }
}

/// Fin Fang Foom — {2}{R}{R} legendary 3/5 flying Alien Dragon Villain. Casting
/// an instant or sorcery that targets an artifact or land copies it (new
/// targets allowed) and puts two +1/+1 counters on Fin Fang Foom.
pub fn fin_fang_foom() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(instant_or_sorcery()),
                Predicate::CastSpellTargetsMatch(R::Artifact.or(R::Land)),
            ])),
            effect: Effect::Seq(vec![
                Effect::CopySpellMayChooseTargets { what: Selector::TriggerSource, count: Value::ONE },
                counters(Selector::This, Value::Const(2)),
            ]),
        }],
        ..creature(
            "Fin Fang Foom",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Alien, CreatureType::Dragon, CreatureType::Villain],
            3,
            5,
        )
    }
}

/// Smashing Success — {3}{R} instant. Destroy target artifact or land; an
/// artifact destroyed this way leaves a Treasure.
pub fn smashing_success() -> CardDefinition {
    spell(
        "Smashing Success",
        cost(&[generic(3), r()]),
        true,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Land)) },
            Effect::If {
                cond: Predicate::SelectorExists(Selector::DestroyedThisResolution { filter: R::Artifact }),
                then: Box::new(make(Arc::new(crabomination_base::tokens::treasure_token()))),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Hard-Won Jitte — {1}{R} Equipment: double strike. Equip {2}.
pub fn hard_won_jitte() -> CardDefinition {
    CardDefinition {
        name: "Hard-Won Jitte",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::DoubleStrike], ..Default::default() }),
        ..Default::default()
    }
}

/// Borrowed Hostility — {R} instant, escalate {3}. Choose one or both: target
/// creature gets +3/+0; target creature gains first strike (until end of turn).
pub fn borrowed_hostility() -> CardDefinition {
    CardDefinition {
        escalate: Some(crate::card::Escalate::Mana(cost(&[generic(3)]))),
        ..spell(
            "Borrowed Hostility",
            cost(&[r()]),
            true,
            Effect::ChooseModesCast {
                modes: vec![
                    Effect::PumpPT { what: target_filtered(R::Creature), power: Value::Const(3), toughness: Value::ZERO, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: target_filtered(R::Creature), keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                ],
                min: 1,
                max: 2,
                allow_repeats: false,
            },
        )
    }
}

// ── Mondo Gecko ─────────────────────────────────────────────────────────────

fn colors_among_yours() -> Value {
    Value::DistinctColorsAmong(Box::new(Selector::EachPermanent(R::Permanent.and(R::ControlledByYou))))
}

/// Mondo Gecko — {1}{U}{U} legendary 2/3 Lizard Mutant. {1}, discard a card:
/// until end of turn it becomes the color of your choice and gains hexproof
/// from that color. Combat damage to a player: draw a card per color among
/// permanents you control.
pub fn mondo_gecko() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::BecomeChosenColorWithHexproof { what: Selector::This, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: colors_among_yours() },
        }],
        ..creature("Mondo Gecko", cost(&[generic(1), u(), u()]), vec![CreatureType::Lizard, CreatureType::Mutant], 2, 3)
    }
}

/// Puca's Eye — {2} artifact. Enters: draw a card, then it becomes the color
/// of your choice. {3}, {T}: draw a card, only with five colors among
/// permanents you control.
pub fn pucas_eye() -> CardDefinition {
    CardDefinition {
        name: "Puca's Eye",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::BecomeChosenColor { what: Selector::This, duration: Duration::Permanent },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            condition: Some(Predicate::ValueAtLeast(colors_among_yours(), Value::Const(5))),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Scrapbasket — {4} 3/2 Scarecrow artifact creature. {1}: it becomes all
/// colors until end of turn.
pub fn scrapbasket() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::BecomeColor { what: Selector::This, colors: Color::ALL.to_vec(), duration: Duration::EndOfTurn, additive: false },
            ..Default::default()
        }],
        ..creature("Scrapbasket", cost(&[generic(4)]), vec![CreatureType::Scarecrow], 3, 2)
    }
}

// ── Renari, Merchant of Marvels ─────────────────────────────────────────────

fn opposing(slot: u8, other_than: &[u8]) -> Selector {
    let mut filter = R::Creature;
    for &s in other_than {
        filter = filter.and(R::OtherThanTargetSlot(s));
    }
    if slot == 0 {
        filter = filter.and(R::ControlledByOpponent);
    }
    Selector::TargetFiltered { slot, filter }
}

/// Blue Dragon — {5}{U}{U} 5/5 flying Dragon. Lightning Breath — enters: until
/// your next turn, target creature an opponent controls gets -3/-0, up to one
/// other target creature -2/-0, up to one other -1/-0.
pub fn blue_dragon() -> CardDefinition {
    let weaken = |what: Selector, n: i32| Effect::PumpPT { what, power: Value::Const(-n), toughness: Value::ZERO, duration: Duration::UntilNextTurn };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq(vec![
                weaken(opposing(0, &[]), 3),
                weaken(opposing(1, &[0]), 2),
                weaken(opposing(2, &[0, 1]), 1),
            ])),
        })],
        ..creature("Blue Dragon", cost(&[generic(5), u(), u()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Juvenile Mist Dragon — {3}{U}{U} 4/3 flying Dragon. Confounding Clouds —
/// enters: for each opponent, tap up to one target creature they control; it
/// doesn't untap during its controller's next untap step.
pub fn juvenile_mist_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 7,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Tap { what: Selector::Target(0) },
                    Effect::SkipNextUntap { what: Selector::Target(0) },
                ])),
            }),
        })],
        ..creature("Juvenile Mist Dragon", cost(&[generic(3), u(), u()]), vec![CreatureType::Dragon], 4, 3)
    }
}

/// Oceanus Dragon — {4}{U}{U} 3/5 flying Dragon. Enters: tap target creature
/// an opponent controls and goad it.
pub fn oceanus_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::ControlledByOpponent)) },
            Effect::Goad { what: Selector::Target(0) },
        ]))],
        ..creature("Oceanus Dragon", cost(&[generic(4), u(), u()]), vec![CreatureType::Dragon], 3, 5)
    }
}

/// Dragonlord's Prerogative — {4}{U}{U} instant. You may reveal a Dragon card
/// as you cast it; if you did or controlled a Dragon, it can't be countered.
/// Draw four cards.
pub fn dragonlords_prerogative() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeCounteredIfCastCondition],
        as_cast_condition: Some(crate::effect::shortcut::dragon_as_cast()),
        ..spell("Dragonlord's Prerogative", cost(&[generic(4), u(), u()]), true, Effect::Draw { who: Selector::You, amount: Value::Const(4) })
    }
}

// ── Doctor Spectrum ─────────────────────────────────────────────────────────

/// Hyperion, Supreme Hero — {4}{W}{W}{W} legendary 7/7 flash, flying Eternal
/// Hero. Damage to you or a Hero you control is prevented down to 1.
pub fn hyperion_supreme_hero() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage to you or a Hero you control, prevent all but 1 of that damage.",
            effect: StaticEffect::PreventAllButOneDamageToYouAndYourHeroes,
        }],
        ..creature("Hyperion, Supreme Hero", cost(&[generic(4), w(), w(), w()]), vec![CreatureType::Eternal, CreatureType::Hero], 7, 7)
    }
}

// ── Barktooth Warbeard, Lovisa Coldeyes, Tyrox: the Intimidators ────────────

fn warrior() -> R {
    R::HasCreatureType(CreatureType::Warrior)
}

/// "Cowards can't block Warriors." — every Warrior, whoever controls it,
/// can't be blocked by a Coward (CR 509.1b reads the blocker's computed types).
fn cowards_cant_block_warriors() -> StaticAbility {
    StaticAbility {
        description: "Cowards can't block Warriors.",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(R::Creature.and(warrior())),
            keyword: Keyword::CantBeBlockedByCreatureType(CreatureType::Coward),
        },
    }
}

/// CR 205.1b — "target creature becomes a [type]" replaces its creature types.
fn becomes(t: CreatureType) -> Effect {
    Effect::BecomeCreatureType { what: target_filtered(R::Creature), creature_types: vec![t], duration: Duration::EndOfTurn }
}

/// Obsidian Battle-Axe — {3} Kindred Artifact — Warrior Equipment. +2/+1 and
/// haste. Whenever a Warrior creature enters, you may attach it to it. Equip
/// {3}. (Only your own Warriors are offered: attaching to an opponent's is
/// legal but never what the "may" is for.)
pub fn obsidian_battle_axe() -> CardDefinition {
    CardDefinition {
        name: "Obsidian Battle-Axe",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Kindred, CardType::Artifact],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Warrior],
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(warrior()) }),
            effect: Effect::MayDo {
                description: "Attach Obsidian Battle-Axe to it?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, keywords: vec![Keyword::Haste], ..Default::default() }),
        ..Default::default()
    }
}

/// Boldwyr Intimidator — {5}{R}{R} 5/5 Giant Warrior. Cowards can't block
/// Warriors. {R}: target creature becomes a Coward until end of turn. {2}{R}:
/// target creature becomes a Warrior until end of turn.
pub fn boldwyr_intimidator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![cowards_cant_block_warriors()],
        activated_abilities: vec![
            ActivatedAbility { mana_cost: cost(&[r()]), effect: becomes(CreatureType::Coward), ..Default::default() },
            ActivatedAbility { mana_cost: cost(&[generic(2), r()]), effect: becomes(CreatureType::Warrior), ..Default::default() },
        ],
        ..creature("Boldwyr Intimidator", cost(&[generic(5), r(), r()]), vec![CreatureType::Giant, CreatureType::Warrior], 5, 5)
    }
}

/// Kargan Intimidator — {1}{R} 3/1 Human Warrior. Cowards can't block
/// Warriors. {1}: choose one that hasn't been chosen this turn — +1/+1 until
/// end of turn; target creature becomes a Coward; target Warrior gains
/// trample.
pub fn kargan_intimidator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![cowards_cant_block_warriors()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::ChooseUnchosenModeThisTurn {
                modes: vec![
                    Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                    becomes(CreatureType::Coward),
                    Effect::GrantKeyword { what: target_filtered(R::Creature.and(warrior())), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
                ],
            },
            ..Default::default()
        }],
        ..creature("Kargan Intimidator", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Warrior], 3, 1)
    }
}

/// Falkenrath Marauders — {3}{R}{R} 2/2 flying, haste Vampire Warrior. Combat
/// damage to a player: two +1/+1 counters on it.
pub fn falkenrath_marauders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: counters(Selector::This, Value::Const(2)),
        }],
        ..creature("Falkenrath Marauders", cost(&[generic(3), r(), r()]), vec![CreatureType::Vampire, CreatureType::Warrior], 2, 2)
    }
}

/// Dragon Grip — {2}{R} Aura. Ferocious: flash while you control a creature
/// with power 4 or greater. Enchanted creature gets +2/+0 and first strike.
pub fn dragon_grip() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Ferocious — you may cast this spell as though it had flash if you control a creature with power 4 or greater.",
            effect: StaticEffect::SelfFlashIf { condition: ferocious() },
        }],
        ..aura(
            "Dragon Grip",
            cost(&[generic(2), r()]),
            R::Creature,
            false,
            EquipBonus { power: 2, keywords: vec![Keyword::FirstStrike], ..Default::default() },
        )
    }
}

// ── Agent Phil Coulson, Hyperion: S.H.I.E.L.D. ──────────────────────────────

/// Nick Fury, Spymaster — {4}{W} legendary 4/4 first strike Human Spy Hero.
/// Whenever a creature you control attacks alone, draw a card; then you may
/// put a creature card with mana value 3 or less from your hand onto the
/// battlefield tapped and attacking, and it gains indestructible until end of
/// turn.
pub fn nick_fury_spymaster() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::AttackingAlone),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::DeployCreatureFromHandAttacking { filter: R::Creature.and(R::ManaValueAtMost(3)), return_to_hand_eot: false },
                Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
            ]),
        }],
        ..creature("Nick Fury, Spymaster", cost(&[generic(4), w()]), vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero], 4, 4)
    }
}

/// Zarda, the Power Princess — {3}{W} legendary 2/4 flying Human Warrior
/// Hero. Other Heroes you control have exalted (CR 702.83: each instance
/// triggers on its own).
pub fn zarda_the_power_princess() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Heroes you control have exalted.",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::HasCreatureType(CreatureType::Hero).and(R::ControlledByYou).and(R::OtherThanSource),
                ability: Box::new(exalted()),
            },
        }],
        ..creature(
            "Zarda, the Power Princess",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Hero],
            2,
            4,
        )
    }
}

/// Orbs of Warding — {5} artifact. You have hexproof. If a creature would deal
/// damage to you, prevent 1 of that damage.
pub fn orbs_of_warding() -> CardDefinition {
    CardDefinition {
        name: "Orbs of Warding",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![
            StaticAbility { description: "You have hexproof.", effect: StaticEffect::ControllerHasHexproof },
            StaticAbility {
                description: "If a creature would deal damage to you, prevent 1 of that damage.",
                effect: StaticEffect::ReduceDamageToControllerFromSource { filter: R::Creature, amount: 1 },
            },
        ],
        ..Default::default()
    }
}

// ── Ojutai, Soul of Winter ──────────────────────────────────────────────────

/// Ojutai Monument — {3} artifact. {T}: {W} or {U}. {4}{W}{U}: a 4/4 white and
/// blue flying Dragon artifact creature until end of turn.
pub fn ojutai_monument() -> CardDefinition {
    CardDefinition {
        name: "Ojutai Monument",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::White, Color::Blue], Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4), w(), u()]),
                effect: Effect::Seq(vec![
                    Effect::BecomeCreature {
                        what: Selector::This,
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![CreatureType::Dragon],
                        keywords: vec![Keyword::Flying],
                        duration: Duration::EndOfTurn,
                    },
                    Effect::BecomeColor { what: Selector::This, colors: vec![Color::White, Color::Blue], duration: Duration::EndOfTurn, additive: false },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ojutai's Breath — {2}{U} instant. Tap target creature; it doesn't untap
/// during its controller's next untap step. Rebound.
pub fn ojutais_breath() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Rebound],
        ..spell(
            "Ojutai's Breath",
            cost(&[generic(2), u()]),
            true,
            Effect::Seq(vec![
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::SkipNextUntap { what: Selector::Target(0) },
            ]),
        )
    }
}

// ── Gornog, the Red Reaper ──────────────────────────────────────────────────

/// Anep, Vizier of Hazoret — {2}{R} legendary 4/2 trample Jackal Warrior. You
/// may exert it as it attacks; when you do, exile the top two cards of your
/// library — you may play them until the end of your next turn.
pub fn anep_vizier_of_hazoret() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample, Keyword::Exert],
        triggered_abilities: vec![crate::effect::shortcut::on_exert(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        ..creature("Anep, Vizier of Hazoret", cost(&[generic(2), r()]), vec![CreatureType::Jackal, CreatureType::Warrior], 4, 2)
    }
}

/// Goma Fada Vanguard — {1}{R} 2/2 Human Warrior. Attacks: target creature an
/// opponent controls with power at most the number of Warriors you control
/// can't block this turn.
pub fn goma_fada_vanguard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(
                R::Creature.and(R::ControlledByOpponent).and(R::PowerAtMostYourCount(Box::new(R::Creature.and(warrior())))),
            ),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Goma Fada Vanguard", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 2)
    }
}

// ── Hixus, Prison Warden ────────────────────────────────────────────────────

/// Hixus, Prison Warden — {3}{W}{W} legendary 4/4 flash Human Soldier.
/// Whenever a creature deals combat damage to you, if Hixus entered this
/// turn, exile that creature until Hixus leaves the battlefield.
pub fn hixus_prison_warden() -> CardDefinition {
    let entered = Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::AnyPlayer).with_filter(Predicate::All(vec![
                Predicate::SamePlayer(PlayerRef::TriggerEventPlayer, PlayerRef::You),
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                entered.clone(),
            ])),
            effect: Effect::If {
                cond: entered,
                then: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::TriggerSource,
                    return_to: crate::card::ExileReturnZone::Battlefield,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Hixus, Prison Warden", cost(&[generic(3), w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 4, 4)
    }
}

/// Prison Sentence — {2}{W} Aura. Enters: scry 2. Enchanted creature can't
/// attack or block, and its activated abilities can't be activated.
pub fn prison_sentence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        ..aura(
            "Prison Sentence",
            cost(&[generic(2), w()]),
            R::Creature,
            false,
            EquipBonus { keywords: vec![Keyword::CantAttack, Keyword::CantBlock, Keyword::CantActivateAbilities], ..Default::default() },
        )
    }
}

// ── Orim, Samite Healer and Reidane: damage shields ─────────────────────────

/// "Prevent 1 of that damage" to you and to creatures you control — the two
/// halves of the Hedron-Field Purists shield, gated on `n` level counters.
fn purist_shield(n: u32) -> Vec<StaticAbility> {
    let gated = |inner: StaticEffect| StaticEffect::WhileCountersAtLeast { kind: CounterType::Level, n, inner: Box::new(inner) };
    vec![
        StaticAbility {
            description: "If a source would deal damage to you, prevent 1 of that damage.",
            effect: gated(StaticEffect::ReduceDamageToYouBy(1)),
        },
        StaticAbility {
            description: "If a source would deal damage to a creature you control, prevent 1 of that damage.",
            effect: gated(StaticEffect::ReduceDamageToYourCreaturesBy(1)),
        },
    ]
}

/// Hedron-Field Purists — {2}{W} 0/3 Human Cleric, level up {2}{W} (CR
/// 702.87). LEVEL 1-4: 1/4, prevent 1 of each damage to you or a creature you
/// control. LEVEL 5+: 2/5, prevent 2 (the level-1 and level-5 shields stack).
pub fn hedron_field_purists() -> CardDefinition {
    let mut statics = purist_shield(1);
    statics.extend(purist_shield(5));
    CardDefinition {
        level_bands: vec![
            crate::card::LevelBand { min: 1, max: Some(4), power: 1, toughness: 4, keywords: vec![] },
            crate::card::LevelBand { min: 5, max: None, power: 2, toughness: 5, keywords: vec![] },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            sorcery_speed: true,
            effect: counters_of(CounterType::Level),
            ..Default::default()
        }],
        static_abilities: statics,
        ..creature("Hedron-Field Purists", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Cleric], 0, 3)
    }
}

fn counters_of(kind: CounterType) -> Effect {
    Effect::AddCounter { what: Selector::This, kind, amount: Value::ONE }
}

fn opponents_sources_shave_one() -> StaticAbility {
    StaticAbility {
        description: "If a source an opponent controls would deal damage to you, prevent 1 of that damage.",
        effect: StaticEffect::ReduceDamageToControllerFromSource { filter: R::ControlledByOpponent, amount: 1 },
    }
}

/// Guardian Seraph — {2}{W}{W} 3/4 flying Angel. A source an opponent controls
/// deals 1 less damage to you.
pub fn guardian_seraph() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![opponents_sources_shave_one()],
        ..creature("Guardian Seraph", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 3, 4)
    }
}

/// Protection of the Hekma — {4}{W} enchantment. A source an opponent controls
/// deals 1 less damage to you.
pub fn protection_of_the_hekma() -> CardDefinition {
    CardDefinition {
        name: "Protection of the Hekma",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![opponents_sources_shave_one()],
        ..Default::default()
    }
}

// ── Alexi, Zephyr Mage and Hisoka, Minamo Sensei ────────────────────────────

fn loot_two() -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
    ])
}

/// All-Seeing Arbiter — {4}{U}{U} 5/4 flying Avatar. Enters or attacks: draw
/// two, discard one. Whenever you discard a card, target creature an opponent
/// controls gets -X/-0 until your next turn, X = the number of different mana
/// values among cards in your graveyard.
pub fn all_seeing_arbiter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(loot_two()),
            on_attack(loot_two()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Negate(Box::new(Value::DistinctManaValuesInGraveyard(PlayerRef::You))),
                    toughness: Value::ZERO,
                    duration: Duration::UntilNextTurn,
                },
            },
        ],
        ..creature("All-Seeing Arbiter", cost(&[generic(4), u(), u()]), vec![CreatureType::Avatar], 5, 4)
    }
}

/// Just the Wind — {1}{U} instant. Return target creature to its owner's
/// hand. Madness {U}.
pub fn just_the_wind() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[u()]))],
        ..spell(
            "Just the Wind",
            cost(&[generic(1), u()]),
            true,
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        )
    }
}

/// Nagging Thoughts — {1}{U} sorcery. Look at the top two: one to hand, the
/// other to the graveyard. Madness {1}{U}.
pub fn nagging_thoughts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Madness(cost(&[generic(1), u()]))],
        ..spell(
            "Nagging Thoughts",
            cost(&[generic(1), u()]),
            false,
            Effect::LookPickToHand(Box::new(crate::effect::LookPick {
                who: PlayerRef::You,
                count: Value::Const(2),
                rest_to_graveyard: true,
                ..Default::default()
            })),
        )
    }
}

/// Disciple of the Ring — {3}{U}{U} 3/4 Human Wizard. {1}, exile an instant or
/// sorcery card from your graveyard: choose one — counter target noncreature
/// spell unless its controller pays {2}; +1/+1 until end of turn; tap target
/// creature; untap target creature.
pub fn disciple_of_the_ring() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            exile_other_filter: Some((instant_or_sorcery(), 1)),
            effect: Effect::ChooseMode(vec![
                Effect::CounterUnlessPaid {
                    what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
                    mana_cost: cost(&[generic(2)]),
                    exile: false,
                    extra_generic: None,
                    if_paid: None,
                },
                Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
                Effect::Tap { what: target_filtered(R::Creature) },
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            ]),
            ..Default::default()
        }],
        ..creature("Disciple of the Ring", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 4)
    }
}

// ── Yasova Dragonclaw ───────────────────────────────────────────────────────

/// Yasova Dragonclaw — {2}{G} legendary 4/2 trample Human Warrior. At the
/// beginning of combat on your turn, you may pay {1}{U/R}{U/R}; if you do,
/// gain control of target creature an opponent controls with power less than
/// Yasova's until end of turn, untap it, and it gains haste.
pub fn yasova_dragonclaw() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {1}{U/R}{U/R} to borrow a smaller creature?".into(),
                mana_cost: cost(&[generic(1), hybrid(Color::Blue, Color::Red), hybrid(Color::Blue, Color::Red)]),
                body: Box::new(Effect::Seq(vec![
                    Effect::GainControl {
                        what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::PowerLessThanSource)),
                        to: None,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ])),
                else_: None,
            },
        }],
        ..creature("Yasova Dragonclaw", cost(&[generic(2), g()]), vec![CreatureType::Human, CreatureType::Warrior], 4, 2)
    }
}

/// Demonmail Hauberk — {4} Equipment. +4/+2. Equip—Sacrifice a creature.
pub fn demonmail_hauberk() -> CardDefinition {
    CardDefinition {
        name: "Demonmail Hauberk",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[]))],
        equip_sacrifice_filter: Some(R::Creature),
        equipped_bonus: Some(EquipBonus { power: 4, toughness: 2, ..Default::default() }),
        ..Default::default()
    }
}

/// Feed the Pack — {5}{G} enchantment. At the beginning of your end step, you
/// may sacrifice a nontoken creature; if you do, create X 2/2 green Wolves,
/// X = its toughness.
pub fn feed_the_pack() -> CardDefinition {
    let wolf = Arc::new(TokenDefinition {
        name: "Wolf".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        name: "Feed the Pack",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a nontoken creature for Wolves?".into(),
                filter: R::Creature.and(R::NotToken),
                count: Value::ONE,
                then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::SacrificedToughness, definition: wolf }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

// ── Yuan Shao, the Indecisive ───────────────────────────────────────────────

/// Yuan Shao, the Indecisive — {4}{R} legendary 2/3 horsemanship Human
/// Soldier. Each creature you control can't be blocked by more than one
/// creature.
pub fn yuan_shao_the_indecisive() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Horsemanship],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control can't be blocked by more than one creature.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::CantBeBlockedByMoreThanOne,
            },
        }],
        ..creature("Yuan Shao, the Indecisive", cost(&[generic(4), r()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 3)
    }
}

/// Horde of Boggarts — {3}{R} */* menace Goblin; P/T = red permanents you
/// control (CR 604.3 characteristic-defining).
pub fn horde_of_boggarts() -> CardDefinition {
    let reds = Value::CountOf(Box::new(Selector::EachPermanent(R::HasColor(Color::Red).and(R::ControlledByYou))));
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Horde of Boggarts's power and toughness are each equal to the number of red permanents you control.",
            effect: StaticEffect::SelfBasePtFromValue { power: reds.clone(), toughness: reds },
        }],
        ..creature("Horde of Boggarts", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 0, 0)
    }
}

/// Pyreheart Wolf — {2}{R} 1/1 undying Wolf. Attacks: creatures you control
/// gain menace until end of turn.
pub fn pyreheart_wolf() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Undying],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            keyword: Keyword::Menace,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Pyreheart Wolf", cost(&[generic(2), r()]), vec![CreatureType::Wolf], 1, 1)
    }
}

// ── Linvala, Shield of Sea Gate: party ──────────────────────────────────────

/// Emeria Captain — {3}{W} 1/1 flying, vigilance Angel Warrior. Enters: a
/// +1/+1 counter per creature in your party (CR 700.8).
pub fn emeria_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(counters(Selector::This, Value::PartyCount))],
        ..creature("Emeria Captain", cost(&[generic(3), w()]), vec![CreatureType::Angel, CreatureType::Warrior], 1, 1)
    }
}

/// Journey to Oblivion — {4}{W} enchantment, {1} less per creature in your
/// party. Enters: exile target nonland permanent an opponent controls until it
/// leaves.
pub fn journey_to_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Journey to Oblivion",
        cost: cost(&[generic(4), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature in your party.",
            effect: StaticEffect::SelfCostReducedByValue { amount: Value::PartyCount },
        }],
        triggered_abilities: vec![etb(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Nonland.and(R::ControlledByOpponent)),
            return_to: crate::card::ExileReturnZone::Battlefield,
        })],
        ..Default::default()
    }
}

/// Concerted Defense — {U} instant. Counter target noncreature spell unless
/// its controller pays {1} plus {1} per creature in your party.
pub fn concerted_defense() -> CardDefinition {
    spell(
        "Concerted Defense",
        cost(&[u()]),
        true,
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
            mana_cost: cost(&[generic(1)]),
            exile: false,
            extra_generic: Some(Value::PartyCount),
            if_paid: None,
        },
    )
}

// ── Aerid Konstrari: Heartwoods ─────────────────────────────────────────────

fn heartwood() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Heartwood".into(),
        card_types: vec![CardType::Artifact],
        colors: vec![Color::Red, Color::Green],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColors(vec![Color::Red, Color::Green], Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    })
}

/// Soul Tether — {2}{R/G} sorcery: create a Heartwood token (a prepare spell).
fn soul_tether() -> Arc<CardDefinition> {
    Arc::new(spell(
        "Soul Tether",
        cost(&[generic(2), hybrid(Color::Red, Color::Green)]),
        false,
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: heartwood() },
    ))
}

/// Hungering Puppetbeast — {3}{G}{G} 5/5 artifact Beast Construct. Enters: a
/// Heartwood token. {1}, sacrifice another artifact: a +1/+1 counter, and your
/// choice of trample, hexproof or haste until end of turn.
pub fn hungering_puppetbeast() -> CardDefinition {
    let gain = |k: Keyword| Effect::GrantKeyword { what: Selector::This, keyword: k, duration: Duration::EndOfTurn };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(make(heartwood()))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![
                counters(Selector::This, Value::ONE),
                Effect::ChooseMode(vec![gain(Keyword::Trample), gain(Keyword::Hexproof), gain(Keyword::Haste)]),
            ]),
            ..Default::default()
        }],
        ..creature("Hungering Puppetbeast", cost(&[generic(3), g(), g()]), vec![CreatureType::Beast, CreatureType::Construct], 5, 5)
    }
}

/// Tenured Tethermage — {1}{R}{G} 1/1 Human Artificer. Enters: you may
/// sacrifice a land; if you do, two tapped Heartwoods. Tap two untapped
/// artifacts you control: two +1/+1 counters on it.
pub fn tenured_tethermage() -> CardDefinition {
    let tapped = Arc::new(TokenDefinition { tapped: true, ..(*heartwood()).clone() });
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MaySacrifice {
            description: "Sacrifice a land for two tapped Heartwoods?".into(),
            filter: R::Land,
            count: Value::ONE,
            then: Box::new(Effect::CreateToken { who: PlayerRef::You, count: Value::Const(2), definition: tapped }),
            else_: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((R::Artifact, 2)),
            effect: counters(Selector::This, Value::Const(2)),
            ..Default::default()
        }],
        ..creature("Tenured Tethermage", cost(&[generic(1), r(), g()]), vec![CreatureType::Human, CreatureType::Artificer], 1, 1)
    }
}

/// Woodwork Prodigy — {2}{R/G} 3/3 Cat Druid with the prepare spell Soul
/// Tether. At the beginning of your upkeep, if it isn't prepared, it becomes
/// prepared.
pub fn woodwork_prodigy() -> CardDefinition {
    let unprepared = Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast { counter: CounterType::Prepared, n: 1 }));
    CardDefinition {
        prepare_spell: Some(soul_tether()),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl).with_filter(unprepared.clone()),
            effect: Effect::If { cond: unprepared, then: Box::new(counters_of(CounterType::Prepared)), else_: Box::new(Effect::Noop) },
        }],
        ..creature("Woodwork Prodigy", cost(&[generic(2), hybrid(Color::Red, Color::Green)]), vec![CreatureType::Cat, CreatureType::Druid], 3, 3)
    }
}

// ── Elrond, Lord of Rivendell: scry and surveil ─────────────────────────────

/// Diviner of Victory — {U} 1/1 Dwarf Wizard, enters prepared; prepare spell
/// Unwind History ({1}{U} sorcery: return target creature an opponent controls
/// with mana value 3 or less to its owner's hand, surveil 1). Whenever you
/// scry or surveil, +1/+1 until end of turn.
pub fn diviner_of_victory() -> CardDefinition {
    let unwind = spell(
        "Unwind History",
        cost(&[generic(1), u()]),
        false,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent).and(R::ManaValueAtMost(3))),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]),
    );
    CardDefinition {
        prepare_spell: Some(Arc::new(unwind)),
        enters_with_counters: Some((CounterType::Prepared, Value::ONE)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
        }],
        ..creature("Diviner of Victory", cost(&[u()]), vec![CreatureType::Dwarf, CreatureType::Wizard], 1, 1)
    }
}

/// Proft, Consulting Detective — {1}{U} legendary 2/2 Human Detective.
/// Whenever you scry or surveil, you may pay {2}; if you do, a +1/+1 counter
/// on Proft and draw a card.
pub fn proft_consulting_detective() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ScriedOrSurveiled, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {2}: a +1/+1 counter on Proft and draw a card?".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(Effect::Seq(vec![
                    counters(Selector::This, Value::ONE),
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ])),
                else_: None,
            },
        }],
        ..creature("Proft, Consulting Detective", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Detective], 2, 2)
    }
}

// ── Denzilore Fatehold ──────────────────────────────────────────────────────

/// Proctor of Potential — {W}{U} 3/1 Human Cleric. Whenever it or another
/// creature you control enters, surveil 1. {W}{U}: return it from your
/// graveyard to the battlefield with a finality counter — only if you've
/// scried or surveilled this turn.
pub fn proctor_of_potential() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w(), u()]),
            from_graveyard: true,
            condition: Some(Predicate::ScriedOrSurveiledThisTurn { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::SpellEntersWithCounters { what: Selector::This, kind: CounterType::Finality, amount: Value::ONE },
                Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false } },
            ]),
            ..Default::default()
        }],
        ..creature("Proctor of Potential", cost(&[w(), u()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 1)
    }
}

// ── Shessra, Death's Whisper: Skeletons ─────────────────────────────────────

fn skeleton() -> R {
    R::HasCreatureType(CreatureType::Skeleton)
}

/// Bone Dragon — {3}{B}{B} 5/4 flying Dragon Skeleton. {3}{B}{B}, exile seven
/// other cards from your graveyard: return this card from your graveyard to
/// the battlefield tapped.
pub fn bone_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), b()]),
            from_graveyard: true,
            exile_other_filter: Some((R::Any, 7)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true } },
            ..Default::default()
        }],
        ..creature("Bone Dragon", cost(&[generic(3), b(), b()]), vec![CreatureType::Dragon, CreatureType::Skeleton], 5, 4)
    }
}

/// Ghastly Death Tyrant — {4}{B}{B} 6/5 Beholder Skeleton. Enters, choose one:
/// destroy target enchantment an opponent controls and lose life equal to its
/// mana value; or creatures you control gain deathtouch until end of turn.
pub fn ghastly_death_tyrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Enchantment.and(R::ControlledByOpponent)) },
                Effect::LoseLife { who: Selector::You, amount: Value::ManaValueOf(Box::new(Selector::Target(0))) },
            ]),
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..creature("Ghastly Death Tyrant", cost(&[generic(4), b(), b()]), vec![CreatureType::Beholder, CreatureType::Skeleton], 6, 5)
    }
}

/// Moss-Pit Skeleton — {B}{G} 2/2 Plant Skeleton, kicker {3}: kicked, it
/// enters with three +1/+1 counters. Whenever one or more +1/+1 counters are
/// put on a creature you control, if this card is in your graveyard, you may
/// put it on top of your library.
pub fn moss_pit_skeleton() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3)]))],
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::IfPred { pred: Box::new(Predicate::SpellWasKicked), then: Box::new(Value::Const(3)), else_: Box::new(Value::ZERO) },
        )),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::FromYourGraveyard)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::ControlledByYou) })
                .once_per_batch(),
            effect: Effect::MayDo {
                description: "Put Moss-Pit Skeleton on top of your library?".into(),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Library { who: PlayerRef::You, pos: crate::effect::LibraryPosition::Top } }),
            },
        }],
        ..creature("Moss-Pit Skeleton", cost(&[b(), g()]), vec![CreatureType::Plant, CreatureType::Skeleton], 2, 2)
    }
}

/// Skeletal Swarming — {3}{B}{G} enchantment. Each Skeleton you control has
/// trample, attacks each combat if able, and gets +X/+0 (X = other Skeletons
/// you control). At the beginning of your end step, a tapped 1/1 black
/// Skeleton — two if a creature died this turn.
pub fn skeletal_swarming() -> CardDefinition {
    let yours = Selector::EachPermanent(R::Creature.and(skeleton()).and(R::ControlledByYou));
    let token = Arc::new(TokenDefinition {
        name: "Skeleton".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        name: "Skeletal Swarming",
        cost: cost(&[generic(3), b(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Each Skeleton you control has trample.",
                effect: StaticEffect::GrantKeyword { applies_to: yours.clone(), keyword: Keyword::Trample },
            },
            StaticAbility {
                description: "Each Skeleton you control attacks each combat if able.",
                effect: StaticEffect::GrantKeyword { applies_to: yours.clone(), keyword: Keyword::MustAttack },
            },
            StaticAbility {
                description: "Each Skeleton you control gets +X/+0, where X is the number of other Skeletons you control.",
                effect: StaticEffect::PumpPTByValue {
                    applies_to: yours.clone(),
                    power: Value::Diff(Box::new(Value::CountOf(Box::new(yours))), Box::new(Value::ONE)),
                    toughness: Value::ZERO,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::IfPred {
                    pred: Box::new(Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE }),
                    then: Box::new(Value::Const(2)),
                    else_: Box::new(Value::ONE),
                },
                definition: token,
            },
        }],
        ..Default::default()
    }
}

// ── Rosheen Meanderer and Rosheen, Roaring Prophet: X spells ────────────────

fn x_only(n: Value) -> Effect {
    Effect::AddMana {
        who: PlayerRef::You,
        pool: ManaPayload::Restricted(Box::new(ManaPayload::Colorless(n)), crate::mana::SpendRestriction::XCostsOnly),
    }
}

/// Rosheen Meanderer — {3}{R/G} legendary 4/4 Giant Shaman. {T}: add
/// {C}{C}{C}{C}, spendable only on costs that contain {X}.
pub fn rosheen_meanderer() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility { tap_cost: true, effect: x_only(Value::Const(4)), ..Default::default() }],
        ..creature("Rosheen Meanderer", cost(&[generic(3), hybrid(Color::Red, Color::Green)]), vec![CreatureType::Giant, CreatureType::Shaman], 4, 4)
    }
}

/// Rosheen, Roaring Prophet — {2}{R}{G} legendary 4/4 Giant Shaman. Enters:
/// mill six, you may put a card with {X} in its mana cost from among them into
/// your hand. {T}: reveal any number of {X} cards in your hand; add {C}{C} per
/// card revealed, spendable only on costs that contain {X} (every one is
/// revealed — more mana is never worse).
pub fn rosheen_roaring_prophet() -> CardDefinition {
    let x_cards_in_hand = Value::CountOf(Box::new(Selector::EachMatching { zone: crate::effect::ZoneRef::Hand(PlayerRef::You), filter: R::HasXInCost }));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::MillThenToHandN { amount: Value::Const(6), filter: R::HasXInCost, take: Value::ONE, otherwise: None })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: x_only(Value::Times(Box::new(Value::Const(2)), Box::new(x_cards_in_hand))),
            ..Default::default()
        }],
        ..creature("Rosheen, Roaring Prophet", cost(&[generic(2), r(), g()]), vec![CreatureType::Giant, CreatureType::Shaman], 4, 4)
    }
}

/// Reap the Past — {X}{R}{G} sorcery. Return X cards at random from your
/// graveyard to your hand. Exile Reap the Past.
pub fn reap_the_past() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Reap the Past",
            cost(&[crate::mana::x(), r(), g()]),
            false,
            Effect::ReturnRandomFromGraveyard { who: PlayerRef::You, filter: R::Any, count: Value::XFromCost },
        )
    }
}

/// Wildest Dreams — {X}{X}{G} sorcery. Return X target cards from your
/// graveyard to your hand. Exile Wildest Dreams.
pub fn wildest_dreams() -> CardDefinition {
    CardDefinition {
        exile_on_resolve: true,
        ..spell(
            "Wildest Dreams",
            cost(&[crate::mana::x(), crate::mana::x(), g()]),
            false,
            Effect::TargetsExactlyX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 20,
                    min_targets: 0,
                    filter: R::InYourGraveyard,
                    effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
                }),
            },
        )
    }
}

// ── Thranduil, Sindarin Liege ───────────────────────────────────────────────

fn elf() -> R {
    R::HasCreatureType(CreatureType::Elf)
}

/// Elvenking's Halls — land, enters tapped. {T}: {G} or {U}. {2}{G}{U}, {T},
/// sacrifice it: two +1/+1 counters on target Elf you control — sorcery speed.
pub fn elvenkings_halls() -> CardDefinition {
    let mut d = crate::sets::dual_land_untyped("Elvenking's Halls", Color::Green, Color::Blue, vec![]);
    d.static_abilities.push(crate::sets::enters_tapped());
    d.activated_abilities.push(ActivatedAbility {
        tap_cost: true,
        sac_cost: true,
        sorcery_speed: true,
        mana_cost: cost(&[generic(2), g(), u()]),
        effect: Effect::AddCounter {
            what: target_filtered(R::Creature.and(elf()).and(R::ControlledByYou)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        },
        ..Default::default()
    });
    d
}

/// Mirkwood Nurturer — {2}{G/U} 3/2 Elf Ranger. Enters: return up to one other
/// target permanent you control to its owner's hand; if you do, a +1/+1
/// counter on it.
pub fn mirkwood_nurturer() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::If {
                    cond: Predicate::SelectorExists(Selector::LastMoved),
                    then: Box::new(counters(Selector::This, Value::ONE)),
                    else_: Box::new(Effect::Noop),
                },
            ])),
        })],
        ..creature("Mirkwood Nurturer", cost(&[generic(2), hybrid(Color::Green, Color::Blue)]), vec![CreatureType::Elf, CreatureType::Ranger], 3, 2)
    }
}

/// Silvan Reveler — {2}{G}{U} 3/2 Elf Citizen. Enters: draw, then discard; a
/// discarded land goes onto the battlefield tapped. Landfall — if this card is
/// in your graveyard, you may pay {1}{G}{U} to return it to your hand.
pub fn silvan_reveler() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                Effect::Move {
                    what: Selector::DiscardedThisResolution { filter: R::Land },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ])),
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ControlledByYou }),
                effect: Effect::MayPay {
                    description: "Pay {1}{G}{U} to return Silvan Reveler to your hand?".into(),
                    mana_cost: cost(&[generic(1), g(), u()]),
                    body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) }),
                    else_: None,
                },
            },
        ],
        ..creature("Silvan Reveler", cost(&[generic(2), g(), u()]), vec![CreatureType::Elf, CreatureType::Citizen], 3, 2)
    }
}

/// Thranduil's Decree — {4}{U}{U} instant. Counter target spell; a permanent
/// spell countered this way is exiled instead, and you may cast it without
/// paying its mana cost for as long as it remains exiled.
pub fn thranduils_decree() -> CardDefinition {
    spell(
        "Thranduil's Decree",
        cost(&[generic(4), u(), u()]),
        true,
        Effect::CounterSpellToZone {
            what: target_filtered(R::IsSpellOnStack),
            zone: crate::effect::CounteredSpellZone::ExileCountererMayCastFreeIfMatching(Box::new(R::PermanentCard)),
        },
    )
}

// ── Proft, Consulting Detective: scry and surveil ───────────────────────────

fn bounce(filter: R) -> Effect {
    Effect::Move { what: target_filtered(filter), to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))) }
}

/// Geist of the Archives — {2}{U} 0/4 defender Spirit. At the beginning of
/// your upkeep, scry 1.
pub fn geist_of_the_archives() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..creature("Geist of the Archives", cost(&[generic(2), u()]), vec![CreatureType::Spirit], 0, 4)
    }
}

/// Surveillance Phantasm — {1}{U} 2/3 defender, flying, vigilance Bird
/// Illusion. Attacks as though it lacked defender once you've scried or
/// surveilled this turn. {3}{U}: surveil 1.
pub fn surveillance_phantasm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "As long as you've scried or surveilled this turn, this creature can attack as though it didn't have defender.",
            effect: StaticEffect::CanAttackIgnoringDefenderWhile { condition: Predicate::ScriedOrSurveiledThisTurn { who: PlayerRef::You } },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Surveillance Phantasm", cost(&[generic(1), u()]), vec![CreatureType::Bird, CreatureType::Illusion], 2, 3)
    }
}

/// Select for Inspection — {U} instant. Return target tapped creature to its
/// owner's hand. Scry 1.
pub fn select_for_inspection() -> CardDefinition {
    spell(
        "Select for Inspection",
        cost(&[u()]),
        true,
        Effect::Seq(vec![bounce(R::Creature.and(R::Tapped)), Effect::Scry { who: PlayerRef::You, amount: Value::ONE }]),
    )
}

/// Unexplained Disappearance — {1}{U} instant. Return target creature to its
/// owner's hand. Surveil 1.
pub fn unexplained_disappearance() -> CardDefinition {
    spell(
        "Unexplained Disappearance",
        cost(&[generic(1), u()]),
        true,
        Effect::Seq(vec![bounce(R::Creature), Effect::Surveil { who: PlayerRef::You, amount: Value::ONE }]),
    )
}

// ── Uchbenbak, the Great Mistake ────────────────────────────────────────────

fn regenerate(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, effect: Effect::Regenerate { what: Selector::This }, ..Default::default() }
}

/// Kathari Remnant — {2}{U}{B} 0/1 flying Bird Skeleton. {B}: regenerate.
/// Cascade.
pub fn kathari_remnant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Cascade],
        activated_abilities: vec![regenerate(cost(&[b()]))],
        ..creature("Kathari Remnant", cost(&[generic(2), u(), b()]), vec![CreatureType::Bird, CreatureType::Skeleton], 0, 1)
    }
}

/// Manor Skeleton — {1}{B} 1/1 haste Skeleton. {1}{B}: regenerate.
pub fn manor_skeleton() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![regenerate(cost(&[generic(1), b()]))],
        ..creature("Manor Skeleton", cost(&[generic(1), b()]), vec![CreatureType::Skeleton], 1, 1)
    }
}

/// Blood for Bones — {3}{B} sorcery; as an additional cost, sacrifice a
/// creature. Return a creature card from your graveyard to the battlefield,
/// then another to your hand (the greatest power first, a headless pick).
pub fn blood_for_bones() -> CardDefinition {
    let from_yard = |to: ZoneDest| Effect::MoveChosen {
        from: Selector::TakeGreatestPower {
            inner: Box::new(Selector::CardsInZone { who: PlayerRef::You, zone: Zone::Graveyard, filter: R::Creature }),
            count: Box::new(Value::Const(i32::MAX)),
        },
        filter: None,
        count: Value::ONE,
        up_to: false,
        to,
    };
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Creature.and(R::ControlledByYou), count: 1 }],
        ..spell(
            "Blood for Bones",
            cost(&[generic(3), b()]),
            false,
            Effect::Seq(vec![
                from_yard(ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false }),
                from_yard(ZoneDest::Hand(PlayerRef::You)),
            ]),
        )
    }
}

// ── Hapatra, the Desert Fang ────────────────────────────────────────────────

/// Hapatra, the Desert Fang — {2}{B}{B}{G} legendary 3/3 Human Cleric. Enters:
/// for each opponent, X -1/-1 counters on up to one target creature that
/// player controls, X = the greatest mana value among cards in your graveyard.
pub fn hapatra_the_desert_fang() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::ForEachOpponentTarget {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 7,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByOpponent),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::MinusOneMinusOne,
                    amount: Value::GreatestManaValueInGraveyard(PlayerRef::You),
                }),
            }),
        })],
        ..creature("Hapatra, the Desert Fang", cost(&[generic(2), b(), b(), g()]), vec![CreatureType::Human, CreatureType::Cleric], 3, 3)
    }
}

/// Altar of Bhaal — {1}{B} artifact. {2}{B}, {T}, exile a creature you
/// control: return target creature card from your graveyard to the
/// battlefield — sorcery speed. Adventure: Bone Offering {2}{B} sorcery, a
/// tapped 4/1 black menace Skeleton.
pub fn altar_of_bhaal() -> CardDefinition {
    let skeleton_token = Arc::new(TokenDefinition {
        name: "Skeleton".into(),
        power: 4,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Skeleton], ..Default::default() },
        keywords: vec![Keyword::Menace],
        tapped: true,
        ..Default::default()
    });
    CardDefinition {
        name: "Altar of Bhaal",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            mana_cost: cost(&[generic(2), b()]),
            exile_permanent_cost: Some((R::Creature.and(R::ControlledByYou), 1)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        adventure: Some(Box::new(crate::card::Adventure {
            name: "Bone Offering",
            cost: cost(&[generic(2), b()]),
            card_types: vec![CardType::Sorcery],
            effect: make(skeleton_token),
        })),
        ..Default::default()
    }
}

/// Something Worth Saving — {1}{G} instant. Mill four; you may put a permanent
/// card from among them into your hand. Gain 1 life.
pub fn something_worth_saving() -> CardDefinition {
    spell(
        "Something Worth Saving",
        cost(&[generic(1), g()]),
        true,
        Effect::Seq(vec![
            Effect::MillThenToHandN { amount: Value::Const(4), filter: R::PermanentCard, take: Value::ONE, otherwise: None },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]),
    )
}

// ── Captain Vargus Wrath: Pirates ───────────────────────────────────────────

/// Captain Vargus Wrath — {U}{R} legendary 1/1 Orc Pirate. Attacks: Pirates
/// you control get +1/+1 until end of turn for each time you've cast a
/// commander from the command zone this game.
pub fn captain_vargus_wrath() -> CardDefinition {
    let n = Value::CommanderCastsFromCommandZone(PlayerRef::You);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Pirate)).and(R::ControlledByYou)),
            power: n.clone(),
            toughness: n,
            duration: Duration::EndOfTurn,
        })],
        ..creature("Captain Vargus Wrath", cost(&[u(), r()]), vec![CreatureType::Orc, CreatureType::Pirate], 1, 1)
    }
}

/// Prosperous Pirates — {4}{U} 3/4 Human Pirate. Enters: two Treasures.
pub fn prosperous_pirates() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(2),
            definition: Arc::new(crabomination_base::tokens::treasure_token()),
        })],
        ..creature("Prosperous Pirates", cost(&[generic(4), u()]), vec![CreatureType::Human, CreatureType::Pirate], 3, 4)
    }
}

/// Protean Raider — {1}{U}{R} 2/2 Shapeshifter Pirate. Raid — if you attacked
/// this turn, you may have it enter as a copy of any creature on the
/// battlefield.
pub fn protean_raider() -> CardDefinition {
    CardDefinition {
        enters_as_copy: Some(crate::card::EntersAsCopy {
            filter: R::Creature,
            condition: Some(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            ..Default::default()
        }),
        ..creature("Protean Raider", cost(&[generic(1), u(), r()]), vec![CreatureType::Shapeshifter, CreatureType::Pirate], 2, 2)
    }
}

/// Mutiny — {R} sorcery. Target creature an opponent controls deals damage
/// equal to its power to another target creature that player controls.
pub fn mutiny() -> CardDefinition {
    spell(
        "Mutiny",
        cost(&[r()]),
        false,
        Effect::DealDamageEqualToPower {
            source: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            target: Selector::TargetFiltered {
                slot: 1,
                filter: R::Creature.and(R::SameControllerAsTargetSlot(0)).and(R::OtherThanTargetSlot(0)),
            },
        },
    )
}

// ── Neheb, the Worthy: Minotaurs and discards ───────────────────────────────

fn self_pump(p: i32) -> Effect {
    Effect::PumpPT { what: Selector::This, power: Value::Const(p), toughness: Value::ZERO, duration: Duration::EndOfTurn }
}

/// Burning-Fist Minotaur — {1}{R} 2/1 first strike Minotaur Wizard. {1}{R},
/// discard a card: +2/+0 until end of turn.
pub fn burning_fist_minotaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            discard_cost: Some((R::Any, 1)),
            effect: self_pump(2),
            ..Default::default()
        }],
        ..creature("Burning-Fist Minotaur", cost(&[generic(1), r()]), vec![CreatureType::Minotaur, CreatureType::Wizard], 2, 1)
    }
}

/// Grisly Survivor — {2}{B} 2/3 Minotaur Warrior. Whenever you cycle or
/// discard a card, +2/+0 until end of turn (cycling discards, CR 702.29a, so
/// one trigger covers both).
pub fn grisly_survivor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: self_pump(2),
        }],
        ..creature("Grisly Survivor", cost(&[generic(2), b()]), vec![CreatureType::Minotaur, CreatureType::Warrior], 2, 3)
    }
}

/// Merciless Javelineer — {2}{B}{R} 4/2 Minotaur Warrior. {2}, discard a card:
/// a -1/-1 counter on target creature; it can't block this turn.
pub fn merciless_javelineer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::MinusOneMinusOne, amount: Value::ONE },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::CantBlock, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Merciless Javelineer", cost(&[generic(2), b(), r()]), vec![CreatureType::Minotaur, CreatureType::Warrior], 4, 2)
    }
}

/// Minotaur Sureshot — {2}{R} 2/3 reach Minotaur Archer. {1}{R}: +1/+0 until
/// end of turn.
pub fn minotaur_sureshot() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(1), r()]), effect: self_pump(1), ..Default::default() }],
        ..creature("Minotaur Sureshot", cost(&[generic(2), r()]), vec![CreatureType::Minotaur, CreatureType::Archer], 2, 3)
    }
}

// ── Aradesh, the Founder: enlist ────────────────────────────────────────────

/// Benalish Faithbonder — {1}{W} 1/3 vigilance Human Cleric with enlist (CR
/// 702.154).
pub fn benalish_faithbonder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![crate::effect::shortcut::enlist()],
        ..creature("Benalish Faithbonder", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Cleric], 1, 3)
    }
}

/// Coalition Skyknight — {3}{W} 2/2 flying Human Knight with enlist.
pub fn coalition_skyknight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::enlist()],
        ..creature("Coalition Skyknight", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 2)
    }
}

/// Guardian of New Benalia — {1}{W} 2/2 Human Soldier. Enlist; whenever it
/// enlists a creature, scry 2. Discard a card: indestructible until end of
/// turn, then tap it.
pub fn guardian_of_new_benalia() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::EnlistThen {
            then: Box::new(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Guardian of New Benalia", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Veteran of the Depths — {3}{W} 2/2 Merfolk Soldier. Whenever it becomes
/// tapped, you may put a +1/+1 counter on it.
pub fn veteran_of_the_depths() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Put a +1/+1 counter on Veteran of the Depths?".into(),
                body: Box::new(counters(Selector::This, Value::ONE)),
            },
        }],
        ..creature("Veteran of the Depths", cost(&[generic(3), w()]), vec![CreatureType::Merfolk, CreatureType::Soldier], 2, 2)
    }
}

// ── The Archimandrite: Advisors, Artificers and Monks ───────────────────────

fn archimandrite_crew() -> R {
    R::Creature.and(
        R::HasCreatureType(CreatureType::Advisor)
            .or(R::HasCreatureType(CreatureType::Artificer))
            .or(R::HasCreatureType(CreatureType::Monk)),
    )
}

fn on_your_noncreature_spell(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Noncreature }),
        effect,
    }
}

/// The Archimandrite — {2}{U}{R}{W} legendary 0/5 Human Advisor. Upkeep: gain
/// life equal to cards in hand minus 4. Whenever you gain life, each Advisor,
/// Artificer and Monk you control gains vigilance and +X/+0 until end of turn
/// (X = the life gained). Tap three untapped Advisors, Artificers and/or Monks
/// you control: draw a card.
pub fn the_archimandrite() -> CardDefinition {
    let crew = Selector::EachPermanent(archimandrite_crew().and(R::ControlledByYou));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Max(
                        Box::new(Value::Diff(Box::new(Value::HandSizeOf(PlayerRef::You)), Box::new(Value::Const(4)))),
                        Box::new(Value::ZERO),
                    ),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword { what: crew.clone(), keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                    Effect::PumpPT { what: crew, power: Value::TriggerEventAmount, toughness: Value::ZERO, duration: Duration::EndOfTurn },
                ]),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            tap_others_cost: Some((archimandrite_crew(), 3)),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("The Archimandrite", cost(&[generic(2), u(), r(), w()]), vec![CreatureType::Human, CreatureType::Advisor], 0, 5)
    }
}

/// Ojutai Exemplars — {2}{W}{W} 4/4 Human Monk. Whenever you cast a noncreature
/// spell, choose one: tap target creature; first strike and lifelink until end
/// of turn; exile it, then return it tapped.
pub fn ojutai_exemplars() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_your_noncreature_spell(Effect::ChooseMode(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
            ]),
            Effect::Seq(vec![Effect::ExileAndReturnToOwner { what: Selector::This }, Effect::Tap { what: Selector::This }]),
        ]))],
        ..creature("Ojutai Exemplars", cost(&[generic(2), w(), w()]), vec![CreatureType::Human, CreatureType::Monk], 4, 4)
    }
}

/// Student of Ojutai — {3}{W} 2/4 Human Monk. Whenever you cast a noncreature
/// spell, gain 2 life.
pub fn student_of_ojutai() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_your_noncreature_spell(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })],
        ..creature("Student of Ojutai", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Monk], 2, 4)
    }
}

// ── Iizuka the Ruthless ─────────────────────────────────────────────────────

/// Upriser Renegade — {1}{R} 1/3 Human Samurai; +2/+0 for each other modified
/// creature you control (CR 700.9).
pub fn upriser_renegade() -> CardDefinition {
    let others = Value::CountOf(Box::new(Selector::EachPermanent(
        R::Creature.and(R::ControlledByYou).and(R::IsModified).and(R::OtherThanSource),
    )));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature gets +2/+0 for each other modified creature you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::This,
                power: Value::Times(Box::new(Value::Const(2)), Box::new(others)),
                toughness: Value::ZERO,
            },
        }],
        ..creature("Upriser Renegade", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Samurai], 1, 3)
    }
}

/// War-Spike Changeling — {3}{R} 3/3 changeling Shapeshifter. {R}: first
/// strike until end of turn.
pub fn war_spike_changeling() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Changeling],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("War-Spike Changeling", cost(&[generic(3), r()]), vec![CreatureType::Shapeshifter], 3, 3)
    }
}

/// The Shattered States Era — {4}{R} Saga. I: gain control of target creature
/// until end of turn, untap it, haste. II: your creatures get +1/+0 until end
/// of turn. III: exile it and return it transformed (Nameless Conqueror, a red
/// 3/3 trample, haste Human Samurai enchantment creature).
pub fn the_shattered_states_era() -> CardDefinition {
    CardDefinition {
        name: "The Shattered States Era",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (
                1,
                Effect::Seq(vec![
                    Effect::GainControl { what: target_filtered(R::Creature), to: None, duration: Duration::EndOfTurn },
                    Effect::Untap { what: Selector::Target(0), up_to: None },
                    Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            ),
            (
                2,
                Effect::PumpPT {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            ),
            (3, Effect::ExileSelfReturnTransformed),
        ],
        back_face: Some(Box::new(CardDefinition {
            color_indicator: vec![Color::Red],
            card_types: vec![CardType::Enchantment, CardType::Creature],
            keywords: vec![Keyword::Trample, Keyword::Haste],
            ..creature("Nameless Conqueror", ManaCost::default(), vec![CreatureType::Human, CreatureType::Samurai], 3, 3)
        })),
        ..Default::default()
    }
}

// ── Shagrat, Loot Bearer: Orcs and Armies ───────────────────────────────────

fn amass_orcs(n: Value) -> Effect {
    Effect::Amass { who: PlayerRef::You, count: n, extra_type: Some(CreatureType::Orc) }
}

/// Shagrat, Loot Bearer — {2}{B}{R} legendary 4/4 Orc Soldier. Attacks: attach
/// up to one target Equipment to it, then amass Orcs X (X = Equipment attached
/// to it).
pub fn shagrat_loot_bearer() -> CardDefinition {
    let attached = Value::CountOf(Box::new(Selector::EachPermanent(
        R::HasArtifactSubtype(ArtifactSubtype::Equipment).and(R::AttachedToSource),
    )));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Seq(vec![
                Effect::Attach { what: target_filtered(R::HasArtifactSubtype(ArtifactSubtype::Equipment)), to: Selector::This },
                amass_orcs(attached),
            ])),
        })],
        ..creature("Shagrat, Loot Bearer", cost(&[generic(2), b(), r()]), vec![CreatureType::Orc, CreatureType::Soldier], 4, 4)
    }
}

/// Uglúk of the White Hand — {2}{B}{R} legendary 3/3 Orc Soldier. Whenever
/// another creature you control dies, a +1/+1 counter on it — two if that
/// creature was a Goblin or Orc.
pub fn ugluk_of_the_white_hand() -> CardDefinition {
    // The Goblin/Orc check is the event filter, read as the creature dies: a
    // dying Orc Army token is gone by the time the trigger resolves.
    let goblin_or_orc = Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: R::HasCreatureType(CreatureType::Goblin).or(R::HasCreatureType(CreatureType::Orc)),
    };
    let died = |filter: Predicate, n: i32| TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours).with_filter(filter),
        effect: counters(Selector::This, Value::Const(n)),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![died(goblin_or_orc.clone(), 2), died(Predicate::Not(Box::new(goblin_or_orc)), 1)],
        ..creature("Uglúk of the White Hand", cost(&[generic(2), b(), r()]), vec![CreatureType::Orc, CreatureType::Soldier], 3, 3)
    }
}

/// Swarming of Moria — {2}{R} sorcery. Create a Treasure, then amass Orcs 2.
pub fn swarming_of_moria() -> CardDefinition {
    spell(
        "Swarming of Moria",
        cost(&[generic(2), r()]),
        false,
        Effect::Seq(vec![
            make(Arc::new(crabomination_base::tokens::treasure_token())),
            amass_orcs(Value::Const(2)),
        ]),
    )
}

// ── Rory Williams ───────────────────────────────────────────────────────────

/// Planar Atlas — {2} artifact, enters tapped. Enters: you may look at the top
/// four and keep up to one land card on top, the rest on the bottom in a
/// random order. {T}: {C}.
pub fn planar_atlas() -> CardDefinition {
    CardDefinition {
        name: "Planar Atlas",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![crate::sets::enters_tapped()],
        triggered_abilities: vec![etb(Effect::LookTopKeepOneRestToGraveyard {
            count: Value::Const(4),
            who: None,
            exile_rest: false,
            rest_bottom_random: true,
        })],
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}

/// Elite Inquisitor — {W}{W} 2/2 first strike, vigilance Human Soldier;
/// protection from Vampires, from Werewolves and from Zombies.
pub fn elite_inquisitor() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::FirstStrike,
            Keyword::Vigilance,
            Keyword::ProtectionFromCreatureType(CreatureType::Vampire),
            Keyword::ProtectionFromCreatureType(CreatureType::Werewolf),
            Keyword::ProtectionFromCreatureType(CreatureType::Zombie),
        ],
        ..creature("Elite Inquisitor", cost(&[w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Loyal Sentry — {W} 1/1 Human Soldier. When it blocks a creature, destroy
/// that creature and this creature.
pub fn loyal_sentry() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::blocks(Effect::Seq(vec![
            Effect::Destroy { what: Selector::BlockedAttacker },
            Effect::Destroy { what: Selector::This },
        ]))],
        ..creature("Loyal Sentry", cost(&[w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 1)
    }
}

/// Savior of Ollenbock — {1}{W}{W} 1/2 Human Soldier with training (CR
/// 702.149). Whenever it trains, exile up to one other target creature from
/// the battlefield or creature card from a graveyard until it leaves the
/// battlefield.
pub fn savior_of_ollenbock() -> CardDefinition {
    let mut trains = crate::effect::shortcut::training();
    if let Effect::If { then, .. } = &mut trains.effect {
        let counter = (**then).clone();
        **then = Effect::Seq(vec![
            counter,
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: target_filtered(
                        R::Creature.and(R::OtherThanSource).or(R::Creature.and(R::InGraveyard)),
                    ),
                    return_to: crate::card::ExileReturnZone::Battlefield,
                }),
            },
        ]);
    }
    CardDefinition {
        triggered_abilities: vec![trains],
        ..creature("Savior of Ollenbock", cost(&[generic(1), w(), w()]), vec![CreatureType::Human, CreatureType::Soldier], 1, 2)
    }
}
