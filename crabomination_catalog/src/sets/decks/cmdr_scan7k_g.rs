//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, sixth
//! batch. Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    SpellSubtype, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value, Waterbend,
};
use crate::effect::shortcut::{etb, sneak, target_filtered};
use crate::effect::{Duration, Effect, LookPick, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, u, w, x};

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

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn tap_mana(mana: ManaCost, pool: ManaPayload) -> ActivatedAbility {
    ActivatedAbility { mana_cost: mana, tap_cost: true, effect: Effect::AddMana { who: PlayerRef::You, pool }, ..Default::default() }
}

fn naturalize_gain(name: &'static str, life: i32) -> CardDefinition {
    spell(
        name,
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Enchantment))),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(life) },
        ]),
    )
}

/// Appetite for the Unnatural — {2}{G} instant: destroy target artifact or
/// enchantment; gain 2 life. (Titanbones, Towering Heart.)
pub fn appetite_for_the_unnatural() -> CardDefinition {
    naturalize_gain("Appetite for the Unnatural", 2)
}

/// Natural End — {2}{G} instant: destroy target artifact or enchantment;
/// gain 3 life. (Titanbones, Towering Heart.)
pub fn natural_end() -> CardDefinition {
    naturalize_gain("Natural End", 3)
}

/// Well-Worn Spatula — {1} Equipment; ETB gain 2; +1/+1; equip {1}. (Zhang
/// Fei, Fierce Warrior.)
pub fn well_worn_spatula() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) })],
        ..equipment(
            "Well-Worn Spatula",
            cost(&[generic(1)]),
            cost(&[generic(1)]),
            EquipBonus { power: 1, toughness: 1, ..Default::default() },
        )
    }
}

/// Guan Yu's 1,000-Li March — {4}{W}{W} sorcery: destroy all tapped
/// creatures. (Zhang Fei, Fierce Warrior.)
pub fn guan_yus_1_000_li_march() -> CardDefinition {
    spell(
        "Guan Yu's 1,000-Li March",
        cost(&[generic(4), w(), w()]),
        CardType::Sorcery,
        Effect::Destroy { what: Selector::EachPermanent(R::Creature.and(R::Tapped)) },
    )
}

/// Quintessential Katana — {W} Equipment. Equipped creature gets +1/+1 and
/// has "whenever this creature deals combat damage, untap it and you gain 2
/// life." Whenever a Ninja you control enters, you may attach it. Equip {2}.
/// (Leonardo, Cutting Edge.)
pub fn quintessential_katana() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasCreatureType(CreatureType::Ninja),
            }),
            effect: Effect::MayDo {
                description: "Attach Quintessential Katana to the Ninja?".into(),
                body: Box::new(Effect::Attach { what: Selector::This, to: Selector::TriggerSource }),
            },
        }],
        ..equipment(
            "Quintessential Katana",
            cost(&[w()]),
            cost(&[generic(2)]),
            EquipBonus {
                power: 1,
                toughness: 1,
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource).once_per_batch(),
                    effect: Effect::Seq(vec![
                        Effect::Untap { what: Selector::This, up_to: None },
                        Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                    ]),
                }],
                ..Default::default()
            },
        )
    }
}

/// Leonardo's Technique — {3}{W} sorcery; sneak {1}{W}. Return one or two
/// target creature cards each with mana value 3 or less from your graveyard
/// to the battlefield. (Leonardo, Cutting Edge.)
pub fn leonardos_technique() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sneak(cost(&[generic(1), w()]))),
        ..spell(
            "Leonardo's Technique",
            cost(&[generic(3), w()]),
            CardType::Sorcery,
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 1,
                filter: R::Creature.and(R::InYourGraveyard).and(R::ManaValueAtMost(3)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        )
    }
}

/// Stature, Size Shifter — {U} legendary 1/1 Human Hero; can't be blocked
/// while her power is 1 or less. Power-up — {X}{U}{U}: X +1/+1 counters.
pub fn stature_size_shifter() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Stature can't be blocked if her power is 1 or less.",
            effect: StaticEffect::SelfHasKeywordWhile { keyword: Keyword::Unblockable, condition: R::PowerAtMost(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), u(), u()]),
            power_up: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::XFromCost },
            ..Default::default()
        }],
        ..legend("Stature, Size Shifter", cost(&[u()]), vec![CreatureType::Human, CreatureType::Hero], 1, 1)
    }
}

/// Secret Identity — {U} instant. Conceal: target creature you control
/// becomes a 1/1 Citizen with hexproof until end of turn. Reveal: it becomes
/// a 3/4 Hero with flying and vigilance. (Stature.)
pub fn secret_identity() -> CardDefinition {
    let form = |kind: CreatureType, p: i32, t: i32, keywords: &[Keyword]| {
        let mut steps = vec![
            Effect::BecomeCreatureType {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                creature_types: vec![kind],
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT { what: Selector::Target(0), power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn },
        ];
        steps.extend(keywords.iter().map(|k| Effect::GrantKeyword { what: Selector::Target(0), keyword: k.clone(), duration: Duration::EndOfTurn }));
        Effect::Seq(steps)
    };
    spell(
        "Secret Identity",
        cost(&[u()]),
        CardType::Instant,
        Effect::ChooseMode(vec![
            form(CreatureType::Citizen, 1, 1, &[Keyword::Hexproof]),
            form(CreatureType::Hero, 3, 4, &[Keyword::Flying, Keyword::Vigilance]),
        ]),
    )
}

/// Katara, Seeking Revenge — {3}{U/B} legendary 3/3 Human Warrior Ally; you
/// may waterbend {2} as you cast her. ETB: draw, then discard unless that was
/// paid. +1/+1 for each Lesson card in your graveyard.
pub fn katara_seeking_revenge() -> CardDefinition {
    let lessons = Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::HasSpellSubtype(SpellSubtype::Lesson) };
    CardDefinition {
        waterbend: Some(Waterbend { amount: Value::Const(2), optional: true }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::ONE },
            Effect::If {
                cond: Predicate::SpellWasWaterbend,
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
            },
        ]))],
        static_abilities: vec![StaticAbility {
            description: "Katara gets +1/+1 for each Lesson card in your graveyard.",
            effect: StaticEffect::PumpPTByValue { applies_to: Selector::This, power: lessons.clone(), toughness: lessons },
        }],
        ..legend(
            "Katara, Seeking Revenge",
            cost(&[generic(3), hybrid(Color::Blue, Color::Black)]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Ally],
            3,
            3,
        )
    }
}

/// Scarring Memories — {3}{B} sorcery Lesson, castable as though it had
/// flash while you control an attacking legendary creature: target opponent
/// sacrifices a creature, discards a card, and loses 3 life. (Katara.)
pub fn scarring_memories() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Lesson], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "You may cast this spell as though it had flash if you control an attacking legendary creature.",
            effect: StaticEffect::SelfFlashIf {
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::Creature.and(R::IsAttacking).and(R::HasSupertype(Supertype::Legendary)).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
            },
        }],
        ..spell(
            "Scarring Memories",
            cost(&[generic(3), b()]),
            CardType::Sorcery,
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 1,
                filter: R::OpponentPlayer,
                effect: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice { who: Selector::Target(0), count: Value::ONE, filter: R::Creature },
                    Effect::Discard { who: Selector::Target(0), amount: Value::ONE, random: false },
                    Effect::LoseLife { who: Selector::Target(0), amount: Value::Const(3) },
                ])),
            },
        )
    }
}

/// Iron-Shield Elf — {1}{B} 3/1 Elf Warrior. Discard a card: indestructible
/// until end of turn; tap it. (Moonstone, Harsh Mistress.)
pub fn iron_shield_elf() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
                Effect::Tap { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..creature("Iron-Shield Elf", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Warrior], 3, 1)
    }
}

/// Minion Missile — {1}{B} sorcery; as an additional cost sacrifice a
/// creature or discard a card. Destroy target creature; 2 damage to its
/// controller. (Moonstone, Harsh Mistress.)
pub fn minion_missile() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::OneOf(vec![
            AdditionalCastCost::SacrificePermanent { filter: R::Creature, count: 1 },
            AdditionalCastCost::Discard { count: 1, filter: None },
        ])],
        ..spell(
            "Minion Missile",
            cost(&[generic(1), b()]),
            CardType::Sorcery,
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::Const(2),
                },
                Effect::Destroy { what: target_filtered(R::Creature) },
            ]),
        )
    }
}

/// Shadow Sliver — {2}{U} 1/1 Sliver; all Sliver creatures have shadow.
/// (Mistform Ultimus.)
pub fn shadow_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Sliver creatures have shadow.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasCreatureType(CreatureType::Sliver))),
                keyword: Keyword::Shadow,
            },
        }],
        ..creature("Shadow Sliver", cost(&[generic(2), u()]), vec![CreatureType::Sliver], 1, 1)
    }
}

/// Call to the Kindred — {3}{U} Aura. Your upkeep: you may look at the top
/// five; a creature card sharing a type with the enchanted creature may go
/// onto the battlefield, the rest on the bottom. (Mistform Ultimus.)
pub fn call_to_the_kindred() -> CardDefinition {
    CardDefinition {
        name: "Call to the Kindred",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(5),
                pick_filter: Some(R::Creature.and(R::SharesCreatureTypeWithAttachedHost)),
                to_battlefield: true,
                optional: true,
                ..Default::default()
            })),
        }],
        ..Default::default()
    }
}

/// The Blackstaff of Waterdeep — {U} legendary artifact; you may choose not
/// to untap it. {1}{U}, {T}: another target nontoken artifact you control is
/// a 4/4 artifact creature for as long as the Blackstaff stays tapped
/// (sorcery). (Katsumasa, the Animator.)
pub fn the_blackstaff_of_waterdeep() -> CardDefinition {
    CardDefinition {
        name: "The Blackstaff of Waterdeep",
        cost: cost(&[u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(
                    R::HasCardType(CardType::Artifact).and(R::ControlledByYou).and(R::OtherThanSource).and(R::Not(Box::new(R::IsToken))),
                ),
                power: Value::Const(4),
                toughness: Value::Const(4),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::WhileSourceTapped,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Reality Heist — {5}{U}{U} instant; affinity for artifacts. Look at the top
/// seven; up to two artifact cards to hand, the rest on the bottom.
/// (Katsumasa, the Animator.)
pub fn reality_heist() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Affinity for artifacts",
            effect: StaticEffect::SelfCostReducedPerPermanentMatching { filter: R::HasCardType(CardType::Artifact), per: 1 },
        }],
        ..spell(
            "Reality Heist",
            cost(&[generic(5), u(), u()]),
            CardType::Instant,
            Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(7),
                take: Some(Value::Const(2)),
                pick_filter: Some(R::HasCardType(CardType::Artifact)),
                optional: true,
                ..Default::default()
            })),
        )
    }
}

/// Meticulous Commons — land; enters tapped unless you control a
/// planeswalker; {T}: {W} or {B}. (Edgar, Ancient Bloodlord.)
pub fn meticulous_commons() -> CardDefinition {
    CardDefinition {
        name: "Meticulous Commons",
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped unless you control a planeswalker.",
            effect: StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(R::Planeswalker.and(R::ControlledByYou)),
                    n: Value::ONE,
                },
            },
        }],
        activated_abilities: vec![
            tap_mana(ManaCost::default(), ManaPayload::Colors(vec![Color::White])),
            tap_mana(ManaCost::default(), ManaPayload::Colors(vec![Color::Black])),
        ],
        ..Default::default()
    }
}

/// Massacre Girl, Most Wanted — {4}{B} legendary 4/4 Human Assassin.
/// Whenever another creature or planeswalker you control dies, 1 damage to
/// target opponent and you gain 1. Whenever an opponent is dealt noncombat
/// damage, a +1/+1 counter on her. (Edgar, Ancient Bloodlord.)
pub fn massacre_girl_most_wanted() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: (R::Creature.or(R::Planeswalker)).and(R::OtherThanSource),
                }),
                effect: Effect::Seq(vec![
                    Effect::DealDamage { to: target_filtered(R::OpponentPlayer), amount: Value::ONE },
                    Effect::GainLife { who: Selector::You, amount: Value::ONE },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PlayerDealtNoncombatDamage, EventScope::OpponentControl),
                effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            },
        ],
        ..legend("Massacre Girl, Most Wanted", cost(&[generic(4), b()]), vec![CreatureType::Human, CreatureType::Assassin], 4, 4)
    }
}

/// Wispweaver Angel — {4}{W}{W} 4/4 Angel; flying. ETB: you may blink
/// another target creature you control. (Dack Fayden, Helping Hand.)
pub fn wispweaver_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::ExileAndReturnToOwner {
                what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
            }),
        })],
        ..creature("Wispweaver Angel", cost(&[generic(4), w(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Northampton Farm — land. {T}: {C}. {1}, {T}: exile target creature you
/// own. {2}, {T}, sacrifice it: a creature card exiled with it returns under
/// your control and every other card exiled with it goes to its owner's hand.
/// (Dack Fayden, Helping Hand.)
pub fn northampton_farm() -> CardDefinition {
    let exiled = |filter: R| Selector::EachMatching { zone: crate::effect::ZoneRef::Exile, filter: filter.and(R::ExiledWithSource) };
    CardDefinition {
        name: "Northampton Farm",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_mana(ManaCost::default(), ManaPayload::Colorless(Value::ONE)),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::ExileWithSource { what: target_filtered(R::Creature.and(R::OwnedByYou)) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Seq(vec![
                    crate::effect::shortcut::choose_one_then(
                        exiled(R::Creature),
                        PlayerRef::You,
                        Effect::Move {
                            what: crate::effect::shortcut::chosen_one(),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                    ),
                    Effect::Move {
                        what: exiled(R::Any),
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}
