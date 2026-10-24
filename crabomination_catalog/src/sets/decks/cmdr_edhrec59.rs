//! Commander: EDHREC ranks 2001-2500, one-card gaps (fifth batch) — Takeno's
//! Ancestral Katana, Sekki's Myojin of Towering Might, Tromokratis's Ghostly
//! Touch, Goro-Goro's Ogre-Head Helm, Torgaar, Kira's Jace Unraveler of
//! Secrets, Taigam Sidisi's Hand's Vengeful Pharaoh. Tests in
//! `tests/recent_b/cmdr_edhrec59.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, ManaCost};

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

/// Ancestral Katana — {1}{W} Equipment. Whenever a Samurai or Warrior you
/// control attacks alone, you may pay {1}; when you do, attach this to it.
/// Equipped creature gets +2/+1. Equip {3}.
pub fn ancestral_katana() -> CardDefinition {
    CardDefinition {
        name: "Ancestral Katana",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus { power: 2, toughness: 1, ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::All(vec![
                Predicate::AttackingAlone,
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Samurai).or(R::HasCreatureType(CreatureType::Warrior)),
                },
            ])),
            effect: Effect::MayPay {
                description: "Pay {1} to attach Ancestral Katana to the attacker?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Myojin of Towering Might — {5}{G}{G}{G} legendary 8/8 Spirit. Enters with
/// an indestructible counter if cast from hand (CR 122.1b). Remove it: distribute eight
/// +1/+1 counters among any number of target creatures you control; they gain
/// trample until end of turn.
pub fn myojin_of_towering_might() -> CardDefinition {
    let mut body = vec![Effect::DistributeCounters {
        total: Value::Const(8),
        counter: CounterType::PlusOnePlusOne,
        filter: R::Creature.and(R::ControlledByYou),
        max_targets: 8,
    }];
    body.extend((0..8).map(|i| Effect::GrantKeyword {
        what: Selector::Target(i),
        keyword: Keyword::Trample,
        duration: Duration::EndOfTurn,
    }));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::EntityMatches { what: Selector::This, filter: R::WasCastFromHand },
            then: Box::new(Effect::AddCounter { what: Selector::This, kind: CounterType::Indestructible, amount: Value::ONE }),
            else_: Box::new(Effect::Noop),
        })],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Indestructible, 1)),
            effect: Effect::Seq(body),
            ..Default::default()
        }],
        ..legend("Myojin of Towering Might", cost(&[generic(5), g(), g(), g()]), vec![CreatureType::Spirit], 8, 8)
    }
}

/// Ghostly Touch — {1}{U} Aura. Enchanted creature has "Whenever this
/// creature attacks, you may tap or untap target permanent."
pub fn ghostly_touch() -> CardDefinition {
    CardDefinition {
        name: "Ghostly Touch",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Tap or untap target permanent?".into(),
                    body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Permanent) }),
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ogre-Head Helm — {1}{R} 2/2 artifact creature Equipment Ogre. Equipped
/// creature gets +2/+2. Whenever it or the equipped creature deals combat
/// damage to a player, you may sacrifice it; if you do, discard your hand,
/// then draw three. Reconfigure {3} (CR 702.151).
pub fn ogre_head_helm() -> CardDefinition {
    let wheel = || Effect::MayDo {
        description: "Sacrifice Ogre-Head Helm to discard your hand and draw three?".into(),
        body: Box::new(Effect::Seq(vec![
            Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::IsSource },
            Effect::Discard { who: Selector::You, amount: Value::HandSizeOf(PlayerRef::You), random: false },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ])),
    };
    let on_damage = || TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: wheel(),
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            creature_types: vec![CreatureType::Ogre],
            ..Default::default()
        },
        keywords: vec![Keyword::Reconfigure(cost(&[generic(3)]))],
        triggered_abilities: vec![on_damage()],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            triggered_abilities: vec![on_damage()],
            triggers_on_equipment: true,
            ..Default::default()
        }),
        ..creature("Ogre-Head Helm", cost(&[generic(1), r()]), vec![CreatureType::Ogre], 2, 2)
    }
}

/// Torgaar, Famine Incarnate — {6}{B}{B} legendary 7/6 Avatar. As an
/// additional cost you may sacrifice any number of creatures; it costs {2}
/// less for each. Enters: up to one target player's life total becomes half
/// their starting life total, rounded down.
pub fn torgaar_famine_incarnate() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificeAnyNumber { filter: R::Creature }],
        self_cost_reduction_per_sacrificed: true,
        self_cost_reduction_per_sacrificed_amount: 2,
        triggered_abilities: vec![etb(Effect::SetLifeTotal {
            who: target_filtered(R::Player),
            amount: Value::DivDown(Box::new(Value::StartingLifeTotal), 2),
        })],
        ..legend("Torgaar, Famine Incarnate", cost(&[generic(6), b(), b()]), vec![CreatureType::Avatar], 7, 6)
    }
}

/// Jace, Unraveler of Secrets — {3}{U}{U} legendary planeswalker Jace, loyalty
/// 5. +1: scry 1, then draw. −2: return target creature to its owner's hand.
/// −8: an emblem with "Whenever an opponent casts their first spell each turn,
/// counter that spell."
pub fn jace_unraveler_of_secrets() -> CardDefinition {
    CardDefinition {
        name: "Jace, Unraveler of Secrets",
        cost: cost(&[generic(3), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Jace], ..Default::default() },
        base_loyalty: 5,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
                    Effect::Draw { who: Selector::You, amount: Value::ONE },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -8,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Jace, Unraveler of Secrets".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                            Predicate::SpellsCastThisTurnEquals { who: PlayerRef::Triggerer, count: Value::ONE },
                        ),
                        effect: Effect::CounterSpell { what: Selector::TriggerSource },
                    }],
                    statics: vec![],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Vengeful Pharaoh — {2}{B}{B}{B} 5/4 Zombie, deathtouch. Whenever combat
/// damage is dealt to you or a planeswalker you control, if this card is in
/// your graveyard, destroy target attacking creature, then put this card on
/// top of your library.
pub fn vengeful_pharaoh() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::ControllerDealtCombatDamage, EventScope::FromYourGraveyard).once_per_batch(),
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Creature.and(R::IsAttacking)) },
                Effect::Move { what: Selector::This, to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top } },
            ]),
        }],
        ..creature("Vengeful Pharaoh", cost(&[generic(2), b(), b(), b()]), vec![CreatureType::Zombie], 5, 4)
    }
}
