//! Exodus (EXO) — the set-closing waves. Tests in `classic_sets/exo`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    DynamicPt, LandType, PlayerTally, SelectionRequirement as R, StaticAbility, Subtypes,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{deal, draw, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, StaticEffect, Value,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

/// An Aura that only carries a static rider on its host.
fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Cat Burglar — {3}{B} 2/2. Sorcery-speed tap to strip a card.
pub fn cat_burglar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Cat Burglar",
            cost(&[generic(3), b()]),
            vec![CreatureType::Kor, CreatureType::Rogue, CreatureType::Minion],
            2,
            2,
        )
    }
}

/// Cinder Crawler — {1}{R} 1/2 that only pumps once it's been blocked.
pub fn cinder_crawler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            condition: Some(Predicate::EntityMatches {
                what: Selector::This,
                filter: R::IsBlocked,
            }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Cinder Crawler", cost(&[generic(1), r()]), vec![CreatureType::Salamander], 1, 2)
    }
}

/// Elvish Berserker — {G} 1/1 that grows by the size of the gang blocking it.
pub fn elvish_berserker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::BlockersOf(Box::new(Selector::This)),
                toughness: Value::BlockersOf(Box::new(Selector::This)),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Elvish Berserker",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Berserker],
            1,
            1,
        )
    }
}

/// Jackalope Herd — {3}{G} 4/5 that bounces itself the moment you cast anything.
pub fn jackalope_herd() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature(
            "Jackalope Herd",
            cost(&[generic(3), g()]),
            vec![CreatureType::Rabbit, CreatureType::Beast],
            4,
            5,
        )
    }
}

/// Mirozel — {3}{U} 2/3 flier that flickers home when anything targets it.
pub fn mirozel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        }],
        ..creature("Mirozel", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 2, 3)
    }
}

/// Rootwater Mystic — {U} 1/1 that peeks at any library's top card.
pub fn rootwater_mystic() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::LookAtTop { who: PlayerRef::Target(0), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Rootwater Mystic",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Scalding Salamander — {2}{R} 2/1. Attacking sprays the ground blockers.
pub fn scalding_salamander() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Have Scalding Salamander deal 1 damage to each creature without flying defending player controls?".to_string(),
                body: Box::new(Effect::DealDamage {
                    to: Selector::ControlledBy {
                        who: PlayerRef::DefendingPlayer,
                        filter: R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
                    },
                    amount: Value::ONE,
                }),
            },
        }],
        ..creature("Scalding Salamander", cost(&[generic(2), r()]), vec![CreatureType::Salamander], 2, 1)
    }
}

/// Shield Mate — {W} 1/1 that throws itself in front of a blow.
pub fn shield_mate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ZERO,
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Shield Mate",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Soltari Visionary — {1}{W}{W} 2/2 shadow that eats an enchantment on connect.
pub fn soltari_visionary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(R::Enchantment.and(R::ControlledByTriggerPlayer)),
            },
        }],
        ..creature(
            "Soltari Visionary",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Soltari, CreatureType::Cleric],
            2,
            2,
        )
    }
}

/// Thalakos Drifters — {2}{U}{U} 3/3 that buys shadow by pitching cards.
pub fn thalakos_drifters() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Shadow,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Thalakos Drifters", cost(&[generic(2), u(), u()]), vec![CreatureType::Thalakos], 3, 3)
    }
}

/// Thalakos Scout — {2}{U} 2/1 shadow that discards its way back to hand.
pub fn thalakos_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature(
            "Thalakos Scout",
            cost(&[generic(2), u()]),
            vec![CreatureType::Thalakos, CreatureType::Soldier, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Thrull Surgeon — {1}{B} 1/1 that trades itself for the best card in a hand.
pub fn thrull_surgeon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::ONE,
                filter: R::Any,
            },
            ..Default::default()
        }],
        ..creature("Thrull Surgeon", cost(&[generic(1), b()]), vec![CreatureType::Thrull], 1, 1)
    }
}

/// Treasure Hunter — {2}{W} 2/2. ETB: may buy back an artifact card.
pub fn treasure_hunter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Return an artifact card from your graveyard to your hand?".to_string(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Artifact.and(R::InYourGraveyard)),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature("Treasure Hunter", cost(&[generic(2), w()]), vec![CreatureType::Human], 2, 2)
    }
}

/// Vampire Hounds — {2}{B} 2/2 that eats creature cards for +2/+2.
pub fn vampire_hounds() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Vampire Hounds",
            cost(&[generic(2), b()]),
            vec![CreatureType::Vampire, CreatureType::Dog],
            2,
            2,
        )
    }
}

/// Wayward Soul — {2}{U}{U} 3/2 flier that dodges removal onto the library.
pub fn wayward_soul() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
            },
            ..Default::default()
        }],
        ..creature("Wayward Soul", cost(&[generic(2), u(), u()]), vec![CreatureType::Spirit], 3, 2)
    }
}

/// Welkin Hawk — {1}{W} 1/1 flier that tutors up its twin when it dies.
pub fn welkin_hawk() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Search your library for a card named Welkin Hawk?".to_string(),
                body: Box::new(Effect::SearchSameNameAs {
                    who: PlayerRef::You,
                    subject: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                    count: None,
                }),
            },
        }],
        ..creature("Welkin Hawk", cost(&[generic(1), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Whiptongue Frog — {2}{U} 1/3 that can buy flying for a turn.
pub fn whiptongue_frog() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Whiptongue Frog", cost(&[generic(2), u()]), vec![CreatureType::Frog], 1, 3)
    }
}

/// Zealots en-Dal — {3}{W} 2/4 that pays out while your board stays mono-white.
pub fn zealots_en_dal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep().with_filter(Predicate::Not(Box::new(
                Predicate::SelectorExists(Selector::EachPermanent(
                    R::Nonland.and(R::ControlledByYou).and(R::HasColor(Color::White).negate()),
                )),
            ))),
            effect: crate::effect::shortcut::gain_life(1),
        }],
        ..creature(
            "Zealots en-Dal",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            4,
        )
    }
}

// ── Counter creatures (Spikes, Thopters, Workhorse) ─────────────────────────

/// Move a +1/+1 counter off the source onto a target creature.
fn feed_counter(mana: ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
        effect: Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        ..Default::default()
    }
}

/// Spike Hatcher — {6}{G} 0/0 with six +1/+1 counters it spends on pumps or
/// regeneration.
pub fn spike_hatcher() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(6))),
        activated_abilities: vec![
            feed_counter(cost(&[generic(2)])),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
        ],
        ..creature("Spike Hatcher", cost(&[generic(6), g()]), vec![CreatureType::Spike], 0, 0)
    }
}

/// Spike Rogue — {1}{G}{G} 0/0 with two +1/+1 counters that trade both ways.
pub fn spike_rogue() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![
            feed_counter(cost(&[generic(2)])),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                remove_counter_among_filter: Some((
                    Some(CounterType::PlusOnePlusOne),
                    1,
                    R::Creature,
                )),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature("Spike Rogue", cost(&[generic(1), g(), g()]), vec![CreatureType::Spike], 0, 0)
    }
}

/// Thopter Squadron — {5} 0/0 flier that converts counters into Thopters and
/// spare Thopters back into counters.
pub fn thopter_squadron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
                sorcery_speed: true,
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Box::new(TokenDefinition {
                        name: "Thopter".to_string(),
                        power: 1,
                        toughness: 1,
                        keywords: vec![Keyword::Flying],
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Thopter],
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                sac_other_filter: Some((
                    R::Creature.and(R::HasCreatureType(CreatureType::Thopter)),
                    1,
                )),
                sorcery_speed: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature("Thopter Squadron", cost(&[generic(5)]), vec![CreatureType::Thopter], 0, 0)
    }
}

/// Workhorse — {6} 0/0 that burns its four counters for colorless mana.
pub fn workhorse() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: crate::effect::shortcut::add_colorless(1),
            ..Default::default()
        }],
        ..creature("Workhorse", cost(&[generic(6)]), vec![CreatureType::Horse], 0, 0)
    }
}

// ── Auras & enchantments ────────────────────────────────────────────────────

/// Dizzying Gaze — {R} Aura on your own creature; it snipes fliers for {R}.
pub fn dizzying_gaze() -> CardDefinition {
    CardDefinition {
        attach_only_filter: Some(R::Creature.and(R::ControlledByYou)),
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        },
        ..aura(
            "Dizzying Gaze",
            cost(&[r()]),
            EquipBonus {
                activated_abilities: vec![ActivatedAbility {
                    mana_cost: cost(&[r()]),
                    effect: deal(
                        1,
                        target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                    ),
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
    }
}

/// Predatory Hunger — {G} Aura that fattens its host off opponents' creatures.
pub fn predatory_hunger() -> CardDefinition {
    aura(
        "Predatory Hunger",
        cost(&[g()]),
        EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                    .with_filter(Predicate::CastSpellMatches(R::Creature)),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        },
    )
}

/// Shackles — {2}{W} Aura that locks a creature down and can be picked back up.
pub fn shackles() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..aura("Shackles", cost(&[generic(2), w()]), EquipBonus::default())
    }
}

/// Spellshock — {2}{R}. Every spell anyone casts costs its caster 2 life.
pub fn spellshock() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: deal(2, Selector::Player(PlayerRef::Triggerer)),
        }],
        ..enchantment("Spellshock", cost(&[generic(2), r()]))
    }
}

/// Equilibrium — {1}{U}{U}. Each creature spell you cast can buy a bounce for {1}.
pub fn equilibrium() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::MayPay {
                description: "Pay {1} to return target creature to its owner's hand?".to_string(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                }),
                else_: None,
            },
        }],
        ..enchantment("Equilibrium", cost(&[generic(1), u(), u()]))
    }
}

/// Treasure Trove — {2}{U}{U}. A repeatable, expensive draw engine.
pub fn treasure_trove() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: draw(1),
            ..Default::default()
        }],
        ..enchantment("Treasure Trove", cost(&[generic(2), u(), u()]))
    }
}

/// Elven Palisade — {G}. Feed it Forests to shrink attackers.
pub fn elven_palisade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Land.and(R::HasLandType(LandType::Forest)), 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::Const(-3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..enchantment("Elven Palisade", cost(&[g()]))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Null Brooch — {4}. Pitch your hand to counter a noncreature spell.
pub fn null_brooch() -> CardDefinition {
    artifact(
        "Null Brooch",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            discard_hand_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Creature.negate())),
            },
            ..Default::default()
        }],
    )
}

/// Skyshaper — {2}. Sacrifice for a team-wide flying alpha strike.
pub fn skyshaper() -> CardDefinition {
    artifact(
        "Skyshaper",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Shattering Pulse — {1}{R} instant with buyback {3}. Artifact removal on a loop.
pub fn shattering_pulse() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        ..instant(
            "Shattering Pulse",
            cost(&[generic(1), r()]),
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Slaughter — {2}{B}{B} instant with buyback—pay 4 life. Unregenerable kill.
pub fn slaughter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Buyback(ManaCost::default())],
        buyback_additional_cost: Some(AdditionalCastCost::PayLife { amount: 4 }),
        ..instant(
            "Slaughter",
            cost(&[generic(2), b(), b()]),
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
            },
        )
    }
}

/// Flowstone Flood — {3}{R} sorcery with buyback—pay 3 life, discard at random.
pub fn flowstone_flood() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(ManaCost::default())],
        buyback_additional_cost: Some(AdditionalCastCost::PayLife { amount: 3 }),
        ..instant(
            "Flowstone Flood",
            cost(&[generic(3), r()]),
            Effect::Destroy { what: target_filtered(R::Land) },
        )
    }
}

/// Sonic Burst — {1}{R} instant. Four damage for a random card.
pub fn sonic_burst() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::DiscardRandom { count: 1 }],
        ..instant("Sonic Burst", cost(&[generic(1), r()]), deal(4, target_any()))
    }
}

/// Necrologia — {3}{B}{B} instant, end step only. Pay X life, draw X.
pub fn necrologia() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::All(vec![
            Predicate::IsTurnOf(PlayerRef::You),
            Predicate::CurrentStepIs(TurnStep::End),
        ])),
        additional_cost_pay_x_life: true,
        ..instant(
            "Necrologia",
            cost(&[generic(3), b(), b()]),
            Effect::Draw { who: Selector::You, amount: Value::XFromCost },
        )
    }
}

/// Aether Tide — {X}{U} sorcery. Pitch X creature cards to bounce X creatures.
pub fn aether_tide() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::DiscardXFromCost],
        ..instant(
            "Aether Tide",
            cost(&[generic(0), u()]),
            Effect::CapTargetsAtX {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 8,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                    }),
                }),
            },
        )
    }
}

/// Resuscitate — {1}{G} instant. Your team buys regeneration for the turn.
pub fn resuscitate() -> CardDefinition {
    instant(
        "Resuscitate",
        cost(&[generic(1), g()]),
        Effect::GrantActivatedAbilityToMatching {
            filter: R::Creature.and(R::ControlledByYou),
            ability: Box::new(ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Mind Maggots — {3}{B} 2/2 that eats creature cards for two counters each.
pub fn mind_maggots() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::DiscardAnyNumber { who: Selector::You, filter: R::Creature },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Times(
                        Box::new(Value::CardsDiscardedThisEffect),
                        Box::new(Value::Const(2)),
                    ),
                },
            ]),
        }],
        ..creature("Mind Maggots", cost(&[generic(3), b()]), vec![CreatureType::Insect], 2, 2)
    }
}

/// Song of Serenity — {1}{G}. Every enchanted creature is benched.
pub fn song_of_serenity() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Creatures that are enchanted can't attack.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::IsEnchanted)),
                    keyword: Keyword::CantAttack,
                },
            },
            StaticAbility {
                description: "Creatures that are enchanted can't block.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: Selector::EachPermanent(R::Creature.and(R::IsEnchanted)),
                    keyword: Keyword::CantBlock,
                },
            },
        ],
        ..enchantment("Song of Serenity", cost(&[generic(1), g()]))
    }
}

// ── The Oath cycle ──────────────────────────────────────────────────────────

/// "At the beginning of each player's upkeep, that player chooses target player
/// who leads them on `tally` and is their opponent. The first player may
/// `body`."
fn oath(name: &'static str, c: ManaCost, tally: PlayerTally, body: Effect) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::OathCatchUp { tally, body: Box::new(body) },
        }],
        ..enchantment(name, c)
    }
}

/// Oath of Druids — {1}{G}. Behind on creatures? Dig one straight onto the board.
pub fn oath_of_druids() -> CardDefinition {
    oath(
        "Oath of Druids",
        cost(&[generic(1), g()]),
        PlayerTally::CreaturesControlled,
        Effect::MayDo {
            description: "Reveal cards from the top of your library until you reveal a creature card?".to_string(),
            body: Box::new(Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::Creature,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                cap: Value::Const(i32::MAX),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::Graveyard,
            }),
        },
    )
}

/// Oath of Ghouls — {1}{B}. Behind on dead creatures? Buy one back each upkeep.
pub fn oath_of_ghouls() -> CardDefinition {
    oath(
        "Oath of Ghouls",
        cost(&[generic(1), b()]),
        PlayerTally::CreatureCardsInGraveyard,
        Effect::MayDo {
            description: "Return a creature card from your graveyard to your hand?".to_string(),
            body: Box::new(Effect::Move {
                what: Selector::one_of(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                }),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        },
    )
}

/// Oath of Lieges — {1}{W}. Behind on lands? Fetch a basic onto the battlefield.
pub fn oath_of_lieges() -> CardDefinition {
    oath(
        "Oath of Lieges",
        cost(&[generic(1), w()]),
        PlayerTally::LandsControlled,
        Effect::MayDo {
            description: "Search your library for a basic land card?".to_string(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::Land.and(R::IsBasicLand),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        },
    )
}

/// Oath of Mages — {1}{R}. Behind on life? Ping the leader each upkeep.
pub fn oath_of_mages() -> CardDefinition {
    oath(
        "Oath of Mages",
        cost(&[generic(1), r()]),
        PlayerTally::Life,
        Effect::MayDo {
            description: "Have Oath of Mages deal 1 damage to that player?".to_string(),
            body: Box::new(deal(1, Selector::Player(PlayerRef::Target(0)))),
        },
    )
}

/// Oath of Scholars — {3}{U}. Behind on cards? Trade your hand for three.
pub fn oath_of_scholars() -> CardDefinition {
    oath(
        "Oath of Scholars",
        cost(&[generic(3), u()]),
        PlayerTally::CardsInHand,
        Effect::MayDo {
            description: "Discard your hand and draw three cards?".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                draw(3),
            ])),
        },
    )
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// Avenging Druid — {2}{G} 1/3. Connecting digs a land onto the battlefield.
pub fn avenging_druid() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Reveal cards from the top of your library until you reveal a land card?".to_string(),
                body: Box::new(Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: R::Land,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    cap: Value::Const(i32::MAX),
                    life_per_revealed: 0,
                    miss_dest: crate::effect::RevealMissDest::Graveyard,
                }),
            },
        }],
        ..creature(
            "Avenging Druid",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            3,
        )
    }
}

/// Cataclysm — {2}{W}{W}. Each player keeps one of each permanent type.
pub fn cataclysm() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Sorcery],
        ..instant(
            "Cataclysm",
            cost(&[generic(2), w(), w()]),
            Effect::SacrificeAllButOnePerType {
                who: Selector::Player(PlayerRef::EachPlayer),
                include_land: true,
            },
        )
    }
}

/// Crashing Boars — {3}{G}{G} 4/4. Attacking drags an untapped blocker in.
pub fn crashing_boars() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MustBlockSource {
                what: Selector::ControlledBy {
                    who: PlayerRef::DefendingPlayer,
                    filter: R::Creature.and(R::Untapped),
                },
                chooser: Some(PlayerRef::DefendingPlayer),
            },
        }],
        ..creature("Crashing Boars", cost(&[generic(3), g(), g()]), vec![CreatureType::Boar], 4, 4)
    }
}

/// Cunning — {1}{U} Aura. +3/+3 until the host commits to combat.
pub fn cunning() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::EnchantedBySource),
            effect: Effect::SacrificeAtNextEndStep { what: Selector::This },
        }],
        ..aura(
            "Cunning",
            cost(&[generic(1), u()]),
            EquipBonus { power: 3, toughness: 3, ..Default::default() },
        )
    }
}

/// Entropic Specter — {3}{B}{B} */* flier sized to a chosen opponent's hand.
pub fn entropic_specter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::EachOpponent }),
        dynamic_pt: Some(DynamicPt::ChosenPlayerTally {
            base_p: 0,
            base_t: 0,
            what: PlayerTally::CardsInHand,
            power_only: false,
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Triggerer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..creature(
            "Entropic Specter",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Specter, CreatureType::Spirit],
            0,
            0,
        )
    }
}

/// Skyshroud War Beast — {1}{G} */* trampler sized to a chosen opponent's
/// nonbasic lands.
pub fn skyshroud_war_beast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        as_enters_effect: Some(Effect::RememberPlayerOnSource { who: PlayerRef::EachOpponent }),
        dynamic_pt: Some(DynamicPt::ChosenPlayerTally {
            base_p: 0,
            base_t: 0,
            what: PlayerTally::NonbasicLandsControlled,
            power_only: false,
        }),
        ..creature("Skyshroud War Beast", cost(&[generic(1), g()]), vec![CreatureType::Beast], 0, 0)
    }
}

/// Limited Resources — {W}. Everyone trims to five lands, and the tenth land
/// on the battlefield shuts land drops off.
pub fn limited_resources() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::etb(
            Effect::EachPlayerKeepsNSacrificesRest { keep: Value::Const(5), filter: None },
        )],
        static_abilities: vec![StaticAbility {
            description: "Players can't play lands as long as ten or more lands are on the battlefield.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Land),
                    n: Value::Const(10),
                },
                inner: Box::new(StaticEffect::NoPlayerCanPlayLands),
            },
        }],
        ..enchantment("Limited Resources", cost(&[w()]))
    }
}

/// Mind Over Matter — {2}{U}{U}{U}{U}. Cards become taps and untaps.
pub fn mind_over_matter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::TapOrUntap {
                what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
            },
            ..Default::default()
        }],
        ..enchantment("Mind Over Matter", cost(&[generic(2), u(), u(), u(), u()]))
    }
}

/// Monstrous Hound — {3}{R} 4/4 that only fights while you're ahead on lands.
pub fn monstrous_hound() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::CantAttackUnlessMoreLandsThanDefender,
            Keyword::CantBlockUnlessMoreLandsThanAttacker,
        ],
        ..creature("Monstrous Hound", cost(&[generic(3), r()]), vec![CreatureType::Dog], 4, 4)
    }
}

/// Pandemonium — {3}{R}. Every creature that enters gets one free shot.
pub fn pandemonium() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
            ),
            effect: Effect::MayDoBy {
                who: PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                description: "Have that creature deal damage equal to its power to any target?"
                    .to_string(),
                body: Box::new(Effect::DealDamageEqualToPower {
                    source: Selector::TriggerSource,
                    target: target_any(),
                }),
            },
        }],
        ..enchantment("Pandemonium", cost(&[generic(3), r()]))
    }
}

/// Paroxysm — {1}{R} Aura. Each upkeep the host either dies to a land or swells.
pub fn paroxysm() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::EnchantedBySource,
            ),
            effect: Effect::RevealTopThenIf {
                who: PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(Box::new(
                    Selector::This,
                )))),
                filter: R::Land,
                then: Box::new(Effect::Destroy {
                    what: Selector::AttachedTo(Box::new(Selector::This)),
                }),
            },
        }],
        ..aura("Paroxysm", cost(&[generic(1), r()]), EquipBonus::default())
    }
}

/// Pit Spawn — {4}{B}{B}{B} 6/4 first striker that exiles what it wounds.
pub fn pit_spawn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: your_upkeep(),
                effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[b(), b()]) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource),
                effect: Effect::Exile { what: Selector::Target(0) },
            },
        ],
        ..creature(
            "Pit Spawn",
            cost(&[generic(4), b(), b(), b()]),
            vec![CreatureType::Demon],
            6,
            4,
        )
    }
}

/// Plaguebearer — {1}{B} 1/1. {X}{X}{B} kills a nonblack creature of value X.
pub fn plaguebearer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x(), crate::mana::x(), b()]),
            effect: Effect::Destroy {
                what: target_filtered(
                    R::Creature
                        .and(R::HasColor(Color::Black).negate())
                        .and(R::ManaValueExactlyXFromCost),
                ),
            },
            ..Default::default()
        }],
        ..creature("Plaguebearer", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 1, 1)
    }
}

/// Reconnaissance — {W}. Pull an attacker back out of combat, untapped.
pub fn reconnaissance() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Seq(vec![
                Effect::RemoveFromCombat {
                    what: target_filtered(
                        R::Creature.and(R::IsAttacking).and(R::ControlledByYou),
                    ),
                },
                Effect::Untap { what: Selector::Target(0), up_to: None },
            ]),
            ..Default::default()
        }],
        ..enchantment("Reconnaissance", cost(&[w()]))
    }
}

/// Spike Cannibal — {1}{B}{B} 0/0 that hoovers up every +1/+1 counter in play.
pub fn spike_cannibal() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ONE)),
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::MoveAllCountersOfKind {
            from: Selector::EachPermanent(R::Creature),
            to: Selector::This,
            kind: CounterType::PlusOnePlusOne,
        })],
        ..creature("Spike Cannibal", cost(&[generic(1), b(), b()]), vec![CreatureType::Spike], 0, 0)
    }
}

/// Volrath's Dungeon — {2}{B}{B}. Grinds hands down; anyone can buy it off.
pub fn volraths_dungeon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                life_cost: 5,
                any_player: true,
                condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::Destroy { what: Selector::This },
                ..Default::default()
            },
            ActivatedAbility {
                discard_cost: Some((R::Any, 1)),
                sorcery_speed: true,
                effect: Effect::PutCardFromHandOnTopOfLibrary {
                    who: Selector::Player(PlayerRef::Target(0)),
                },
                ..Default::default()
            },
        ],
        ..enchantment("Volrath's Dungeon", cost(&[generic(2), b(), b()]))
    }
}

/// Wall of Nets — {1}{W}{W} 0/7 defender. Everything it blocks is exiled with it.
pub fn wall_of_nets() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::ExileUntilSourceLeaves {
                    what: Selector::EachPermanent(R::Creature.and(R::BlockedBySourceThisTurn)),
                    return_to: crate::card::ExileReturnZone::Battlefield,
                }),
            },
        }],
        ..creature("Wall of Nets", cost(&[generic(1), w(), w()]), vec![CreatureType::Wall], 0, 7)
    }
}

/// Mogg Assassin — {2}{R} 2/1. A coin flip decides whose pick dies.
pub fn mogg_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                on_tails: Box::new(Effect::Destroy { what: Selector::Target(1) }),
            },
            ..Default::default()
        }],
        ..creature(
            "Mogg Assassin",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Assassin],
            2,
            1,
        )
    }
}

// ── Wave 4: the closers ─────────────────────────────────────────────────────

/// A Licid: a creature that pays `attach` to become an Aura on a target
/// creature (running `extra` on the way) and `end` to climb back off.
fn licid(
    base: CardDefinition,
    attach: ManaCost,
    end: ManaCost,
    extra: Effect,
) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: attach,
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::LicidAttach { host: target_filtered(R::Creature), end_cost: end },
                extra,
            ]),
            ..Default::default()
        }],
        ..base
    }
}

/// Dominating Licid — {1}{U}{U} 1/1. As an Aura, you control its host.
pub fn dominating_licid() -> CardDefinition {
    licid(
        creature("Dominating Licid", cost(&[generic(1), u(), u()]), vec![CreatureType::Licid], 1, 1),
        cost(&[generic(1), u(), u()]),
        cost(&[u()]),
        Effect::GainControlWhileSourceAttached,
    )
}

/// Transmogrifying Licid — {3} 2/2 artifact. As an Aura, its host gets +1/+1
/// and becomes an artifact too.
pub fn transmogrifying_licid() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            add_card_types: vec![CardType::Artifact],
            ..Default::default()
        }),
        ..licid(
            creature("Transmogrifying Licid", cost(&[generic(3)]), vec![CreatureType::Licid], 2, 2),
            cost(&[generic(1)]),
            cost(&[generic(1)]),
            Effect::Noop,
        )
    }
}

/// Fade Away — {2}{U}. Every creature on the board taxes its controller {1}.
pub fn fade_away() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Sorcery],
        ..instant(
            "Fade Away",
            cost(&[generic(2), u()]),
            Effect::SacrificeEachUnlessPays { filter: R::Creature, cost: cost(&[generic(1)]) },
        )
    }
}

/// Fighting Chance — {R}. A coin flip per blocker decides whose damage lands.
pub fn fighting_chance() -> CardDefinition {
    instant(
        "Fighting Chance",
        cost(&[r()]),
        Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature.and(R::IsBlocking)),
            body: Box::new(Effect::FlipCoin {
                count: Value::ONE,
                on_heads: Box::new(Effect::PreventCombatDamageByTargetThisTurn {
                    target: Selector::TriggerSource,
                }),
                on_tails: Box::new(Effect::Noop),
            }),
        },
    )
}

/// Kor Chant — {2}{W}. Push the next hit off your creature onto another one.
///
/// Approximation: the chosen source collapses into "the next damage event",
/// so a second source can steal the redirect.
pub fn kor_chant() -> CardDefinition {
    instant(
        "Kor Chant",
        cost(&[generic(2), w()]),
        Effect::RedirectNextDamageTo {
            what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
            to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanSource) },
        },
    )
}

/// Penance — {2}{W}. Bank cards from hand against black and red damage.
pub fn penance() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            put_hand_on_library_cost: true,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::HasColor(Color::Black).or(R::HasColor(Color::Red)),
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
                exile_top_per_prevented: false,
            },
            ..Default::default()
        }],
        ..enchantment("Penance", cost(&[generic(2), w()]))
    }
}
