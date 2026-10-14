//! Commander: four-card gaps of EDHREC's commanders ranked 4501-5500 (the
//! 2026-10-14 rescan). Tests in `tests/recent_b/cmdr_rank5k_e.rs`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{enrage, etb, exalted, on_attack, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, SpendRestriction, cost, generic, r, w};
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

fn make(def: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(def) }
}

fn treasure() -> Effect {
    make(crabomination_base::tokens::treasure_token())
}

fn dwarf_warrior() -> Vec<CreatureType> {
    vec![CreatureType::Dwarf, CreatureType::Warrior]
}

fn gamma() -> Vec<CreatureType> {
    vec![CreatureType::Gamma, CreatureType::Berserker]
}

fn targeted_by_your_spell(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellTargetsMatch(R::Creature.and(R::ControlledByYou))),
        effect,
    }
}

// ── Zhao Zilong, Tiger General ───────────────────────────────────────────────

/// Zhao Zilong, Tiger General — {3}{W}{W} legendary 3/3 Human Soldier
/// Warrior, horsemanship. Whenever he blocks, +1/+1 until end of turn.
pub fn zhao_zilong_tiger_general() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        triggered_abilities: vec![crate::effect::shortcut::blocks(Effect::PumpPT {
            what: Selector::This,
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..legend(
            "Zhao Zilong, Tiger General",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Shu Cavalry — {2}{W} 2/2 Human Soldier, horsemanship.
pub fn shu_cavalry() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Horsemanship],
        ..creature("Shu Cavalry", cost(&[generic(2), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Shu General — {3}{W} 2/2 Human Soldier; vigilance, horsemanship.
pub fn shu_general() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Horsemanship],
        ..creature("Shu General", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Riding Red Hare — {2}{W} sorcery. Target creature gets +3/+3 and gains
/// horsemanship until end of turn.
pub fn riding_red_hare() -> CardDefinition {
    spell(
        "Riding Red Hare",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Horsemanship, duration: Duration::EndOfTurn },
        ]),
    )
}

// ── Zhao, the Seething Flame ─────────────────────────────────────────────────

/// Zhao, the Seething Flame — {4}{R} legendary 5/5 Human Soldier, menace.
pub fn zhao_the_seething_flame() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        ..legend(
            "Zhao, the Seething Flame",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            5,
            5,
        )
    }
}

/// Reckless Ransacking — {1}{R} instant. Target creature gets +3/+2 until end
/// of turn. Create a Treasure.
pub fn reckless_ransacking() -> CardDefinition {
    spell(
        "Reckless Ransacking",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(3),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            treasure(),
        ]),
    )
}

/// Sear — {1}{R} instant. 4 damage to target creature or planeswalker.
pub fn sear() -> CardDefinition {
    spell(
        "Sear",
        cost(&[generic(1), r()]),
        CardType::Instant,
        Effect::DealDamage { to: target_filtered(R::Creature.or(R::Planeswalker)), amount: Value::Const(4) },
    )
}

/// Tweeze — {2}{R} instant. 3 damage to any target; you may discard a card
/// to draw a card.
pub fn tweeze() -> CardDefinition {
    spell(
        "Tweeze",
        cost(&[generic(2), r()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::DealDamage { to: crate::effect::shortcut::target_any(), amount: Value::Const(3) },
            Effect::MayDo {
                description: "Discard a card to draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    crate::effect::shortcut::if_discarded(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                ])),
            },
        ]),
    )
}

// ── Glóin, Éomer and the Hulks ───────────────────────────────────────────────

/// Glóin the Mighty // Easy Pickings — {3}{R} legendary 4/3 Dwarf Warrior. At
/// the beginning of your first main phase, add {R}{R}. Adventure {2}{R}
/// sorcery: 1 damage to each creature your opponents control.
pub fn gloin_the_mighty() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red, Color::Red]) },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Easy Pickings",
            cost: cost(&[generic(2), r()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                amount: Value::ONE,
            },
        })),
        ..legend("Glóin the Mighty", cost(&[generic(3), r()]), dwarf_warrior(), 4, 3)
    }
}

/// Dori, Bearer of Friends — {2}{R} legendary 3/2 trample Dwarf Warrior. When
/// she enters, create a Treasure.
pub fn dori_bearer_of_friends() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(treasure())],
        ..legend("Dori, Bearer of Friends", cost(&[generic(2), r()]), dwarf_warrior(), 3, 2)
    }
}

/// Dáin Ironfoot — {2}{R} legendary 1/4 Dwarf Warrior. When he enters, an Axe
/// Equipment token (+1/+0, equip {2}) attached to a creature you control.
/// Whenever he attacks, each equipped attacking creature gains double strike
/// until end of turn.
pub fn dain_ironfoot() -> CardDefinition {
    let axe = TokenDefinition {
        name: "Axe".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, ..Default::default() }),
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::Seq(vec![
                // "When you do, attach it to target creature you control":
                // the creature is chosen with the trigger, the Axe it gets is
                // the one just made.
                make(axe),
                Effect::Attach {
                    what: Selector::LastCreatedToken,
                    to: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
            ])),
            on_attack(Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsAttacking).and(R::IsEquipped).and(R::ControlledByYou),
                ),
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            }),
        ],
        ..legend("Dáin Ironfoot", cost(&[generic(2), r()]), dwarf_warrior(), 1, 4)
    }
}

/// Quinjet Technician — {1}{R} 2/1 Human Artificer. {T}: add {R}. {T}: add
/// {R}{R}, spendable only on power-up abilities.
pub fn quinjet_technician() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            crate::sets::tap_add(Color::Red),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Restricted(
                        Box::new(ManaPayload::Colors(vec![Color::Red, Color::Red])),
                        SpendRestriction::PowerUpOnly,
                    ),
                },
                ..Default::default()
            },
        ],
        ..creature("Quinjet Technician", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Artificer], 2, 1)
    }
}

/// Éomer of the Riddermark — {4}{R} legendary 5/4 haste Human Knight.
/// Whenever he attacks, if you control a creature with the greatest power
/// among creatures on the battlefield, a 1/1 white Human Soldier token.
pub fn eomer_of_the_riddermark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource).with_filter(Predicate::SelectorExists(
                Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::HasGreatestPowerAmongAllCreatures)),
            )),
            effect: make(TokenDefinition {
                name: "Human Soldier".into(),
                power: 1,
                toughness: 1,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Human, CreatureType::Soldier],
                    ..Default::default()
                },
                ..Default::default()
            }),
        }],
        ..legend(
            "Éomer of the Riddermark",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
            5,
            4,
        )
    }
}

/// Hulk, Bruce Banner — {3}{R}{R} legendary 7/4 trample Gamma Berserker Hero.
pub fn hulk_bruce_banner() -> CardDefinition {
    let mut types = gamma();
    types.push(CreatureType::Hero);
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..legend("Hulk, Bruce Banner", cost(&[generic(3), r(), r()]), types, 7, 4)
    }
}

/// Hulk, Always Angry — {5}{R}{R} legendary 9/9 trample Gamma Berserker Hero.
/// When he enters, destroy all artifacts. He attacks each combat if able.
pub fn hulk_always_angry() -> CardDefinition {
    let mut types = gamma();
    types.push(CreatureType::Hero);
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::MustAttack],
        triggered_abilities: vec![etb(Effect::Destroy { what: Selector::EachPermanent(R::Artifact) })],
        ..legend("Hulk, Always Angry", cost(&[generic(5), r(), r()]), types, 9, 9)
    }
}

/// Red Hulk — {4}{R}{R} legendary 6/7 Gamma Berserker Villain; reach,
/// trample. Enrage: a +1/+1 counter on him; when you do, he deals damage equal
/// to his +1/+1 counters to any other target.
pub fn red_hulk() -> CardDefinition {
    let mut types = gamma();
    types.push(CreatureType::Villain);
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![enrage(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Reflexive {
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::any_target().and(R::OtherThanSource)),
                    amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
                }),
            },
        ]))],
        ..legend("Red Hulk", cost(&[generic(4), r(), r()]), types, 6, 7)
    }
}

/// Structural Assault — {3}{R}{R} sorcery. Destroy all artifacts, then damage
/// to each creature equal to the artifacts put into graveyards from the
/// battlefield this turn.
pub fn structural_assault() -> CardDefinition {
    spell(
        "Structural Assault",
        cost(&[generic(3), r(), r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(R::Artifact) },
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature),
                amount: Value::ArtifactsToGraveyardFromBattlefieldThisTurn,
            },
        ]),
    )
}

// ── Mockingbird, Ace Agent ───────────────────────────────────────────────────

/// Mockingbird, Ace Agent — {3}{W} legendary 2/2 double strike Human Spy Hero.
/// Whenever you cast a spell that targets a creature you control, a +1/+1
/// counter on her.
pub fn mockingbird_ace_agent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![targeted_by_your_spell(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..legend(
            "Mockingbird, Ace Agent",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Spy, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Colleen Wing, Street Samurai — {1}{W} legendary 2/2 Human Samurai Hero.
/// Whenever you cast a spell that targets a creature you control, a +1/+1
/// counter on her, then scry 1.
pub fn colleen_wing_street_samurai() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![targeted_by_your_spell(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::Scry { who: PlayerRef::You, amount: Value::ONE },
        ]))],
        ..legend(
            "Colleen Wing, Street Samurai",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Samurai, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Knight of Glory — {1}{W} 2/1 Human Knight; protection from black, exalted.
pub fn knight_of_glory() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        triggered_abilities: vec![exalted()],
        ..creature("Knight of Glory", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Knight], 2, 1)
    }
}

/// Brilliant Wings — {1}{W} Aura with flash, enchant creature you control.
/// Enchanted creature has flying and hexproof. Whenever a creature you
/// control enters, you may pay {1} to attach this Aura to it.
pub fn brilliant_wings() -> CardDefinition {
    CardDefinition {
        name: "Brilliant Wings",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Flying, Keyword::Hexproof], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
            effect: Effect::MayPay {
                description: "Pay {1} to move Brilliant Wings onto the entering creature?".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}
