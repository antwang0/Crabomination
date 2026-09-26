//! Commander: the cards the **Aura of Courage** precon (AFC, Galea, Kindler of
//! Hope) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_galea.rs`.
//!
//! Residuals (each also on its card):
//! - **Clay Golem** — the d8 is rolled as the ability resolves, not as a cost.
//! - **Song of Inspiration** — the cards come back before the roll (both
//!   results return them, so only the life total waits on it).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, on_attack, on_becomes_monstrous, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, u, w, x};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, host: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![crate::card::ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// CR 706 — one die, a results table.
fn roll(sides: u8, modifier: Value, results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides,
        count: Value::ONE,
        modifier,
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

fn draw(amount: Value) -> Effect {
    Effect::Draw { who: Selector::You, amount }
}

fn aura_card() -> R {
    R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)
}

/// Galea, Kindler of Hope — vigilance; look at your top card any time and
/// cast Auras and Equipment from it; an Equipment cast this way attaches to a
/// creature of yours as it enters.
pub fn galea_kindler_of_hope() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![
            StaticAbility {
                description: "You may look at the top card of your library any time.",
                effect: StaticEffect::MayLookAtOwnLibraryTop,
            },
            StaticAbility {
                description: "You may cast Aura and Equipment spells from the top of your library.",
                effect: StaticEffect::PlayFromLibraryTop {
                    filter: aura_card().or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment)),
                },
            },
            StaticAbility {
                description: "When you cast an Equipment spell this way, it gains \"When this Equipment enters, attach it to target creature you control.\"",
                effect: StaticEffect::LibraryTopEquipmentAttachesOnEntry,
            },
        ],
        ..creature(
            "Galea, Kindler of Hope",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Elf, CreatureType::Knight],
            4,
            4,
        )
    }
}

/// Abundant Growth — on a land: draw on entry; the land taps for any color.
pub fn abundant_growth() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(draw(Value::ONE))],
        static_abilities: vec![StaticAbility {
            description: "Enchanted land has \"{T}: Add one mana of any color.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..aura("Abundant Growth", cost(&[g()]), R::Land, EquipBonus::default())
    }
}

/// Belt of Giant Strength — base 10/10; equip {10}, {X} less for the target's
/// power.
pub fn belt_of_giant_strength() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Equip costs {X} less, where X is the power of the creature it targets.",
            effect: StaticEffect::EquipCostReducedByTargetPower,
        }],
        ..equipment(
            "Belt of Giant Strength",
            cost(&[generic(1), g()]),
            cost(&[generic(10)]),
            EquipBonus { set_base_pt: Some((10, 10)), ..Default::default() },
        )
    }
}

/// Catti-brie of Mithral Hall — reach, first strike; attacking, a counter per
/// Equipment on her; {1}, remove all +1/+1 counters: that much damage to an
/// opponent's attacking or blocking creature.
pub fn catti_brie_of_mithral_hall() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Reach, Keyword::FirstStrike],
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CountMatching {
                sel: Box::new(Selector::AttachedToMe(Box::new(Selector::This))),
                filter: R::HasArtifactSubtype(crate::card::ArtifactSubtype::Equipment),
            },
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            remove_all_counters_cost: Some(CounterType::PlusOnePlusOne),
            effect: Effect::DealDamage {
                to: target_filtered(
                    R::Creature.and(R::IsAttacking.or(R::IsBlocking)).and(R::ControlledByOpponent),
                ),
                amount: Value::CountersRemovedAsCost,
            },
            ..Default::default()
        }],
        ..creature(
            "Catti-brie of Mithral Hall",
            cost(&[g(), w()]),
            vec![CreatureType::Human, CreatureType::Archer],
            2,
            2,
        )
    }
}

/// Clay Golem — {6}, roll a d8: monstrosity X; becoming monstrous destroys a
/// permanent. ⚠ The roll happens as the ability resolves.
pub fn clay_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: roll(8, Value::Const(0), vec![(1, 8, Effect::Monstrosity { n: Value::LastDieRoll })]),
            ..Default::default()
        }],
        triggered_abilities: vec![on_becomes_monstrous(Effect::Destroy { what: target_filtered(R::Permanent) })],
        ..creature("Clay Golem", cost(&[generic(4)]), vec![CreatureType::Golem], 4, 4)
    }
}

/// Diviner's Portent — roll a d20 plus your hand size: draw X, or at 15+ scry
/// X then draw X.
pub fn diviners_portent() -> CardDefinition {
    let draw_x = || draw(Value::XFromCost);
    spell(
        "Diviner's Portent",
        cost(&[x(), u(), u(), u()]),
        CardType::Instant,
        roll(
            20,
            Value::HandSizeOf(PlayerRef::You),
            vec![
                (1, 14, draw_x()),
                (15, 255, Effect::Seq(vec![Effect::Scry { who: PlayerRef::You, amount: Value::XFromCost }, draw_x()])),
            ],
        ),
    )
}

/// Ebony Fly — enters tapped, taps for {C}; {4}: roll a d6 and become an X/X
/// flying Insect this turn; attacking, another attacker gains flying.
pub fn ebony_fly() -> CardDefinition {
    CardDefinition {
        name: "Ebony Fly",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: roll(
                    6,
                    Value::Const(0),
                    vec![(
                        1,
                        6,
                        Effect::BecomeCreature {
                            what: Selector::This,
                            power: Value::LastDieRoll,
                            toughness: Value::LastDieRoll,
                            creature_types: vec![CreatureType::Insect],
                            keywords: vec![Keyword::Flying],
                            duration: Duration::EndOfTurn,
                        },
                    )],
                ),
                ..Default::default()
            },
        ],
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(R::Creature.and(R::IsAttacking).and(R::OtherThanSource)),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Fey Steed — attacking, another attacker of yours gains indestructible;
/// when an opponent targets your creature or planeswalker, you may draw.
pub fn fey_steed() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::GrantKeyword {
                what: target_filtered(
                    R::Creature.and(R::IsAttacking).and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::YourPermanentTargetedByOpponent)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature.or(R::Planeswalker),
                    }),
                effect: Effect::MayDo { description: "Draw a card?".into(), body: Box::new(draw(Value::ONE)) },
            },
        ],
        ..creature("Fey Steed", cost(&[generic(2), w(), w()]), vec![CreatureType::Elk], 4, 4)
    }
}

/// Gryff's Boon — +1/+0 and flying; {3}{W}: return it from your graveyard
/// attached to target creature, as a sorcery.
pub fn gryffs_boon() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            from_graveyard: true,
            sorcery_speed: true,
            effect: Effect::ReturnSelfAttachedTo { host: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..aura(
            "Gryff's Boon",
            cost(&[w()]),
            R::Creature,
            EquipBonus { power: 1, keywords: vec![Keyword::Flying], ..Default::default() },
        )
    }
}

/// Holy Avenger — double strike; when the equipped creature deals combat
/// damage, you may put an Aura from hand onto the battlefield attached to it.
pub fn holy_avenger() -> CardDefinition {
    equipment(
        "Holy Avenger",
        cost(&[generic(2), w()]),
        cost(&[generic(2), w()]),
        EquipBonus {
            keywords: vec![Keyword::DoubleStrike],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::SelfSource),
                effect: Effect::PutAuraFromHandAttachedTo { host: Selector::This },
            }],
            ..Default::default()
        },
    )
}

/// Netherese Puzzle-Ward — your upkeep rolls a d4 and scries that much; a
/// die's highest natural result draws a card.
pub fn netherese_puzzle_ward() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: roll(
                    4,
                    Value::Const(0),
                    vec![(1, 4, Effect::Scry { who: PlayerRef::You, amount: Value::LastDieRoll })],
                ),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::RolledNaturalMax, EventScope::YourControl),
                effect: draw(Value::ONE),
            },
        ],
        ..spell("Netherese Puzzle-Ward", cost(&[generic(3), u()]), CardType::Enchantment, Effect::Noop)
    }
}

/// Psychic Impetus — +2/+2 and goaded; you scry 2 when it attacks.
pub fn psychic_impetus() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature is goaded.",
            effect: StaticEffect::AttachedIsGoaded,
        }],
        ..aura(
            "Psychic Impetus",
            cost(&[generic(2), u()]),
            R::Creature,
            EquipBonus {
                power: 2,
                toughness: 2,
                triggers_on_equipment: true,
                triggered_abilities: vec![TriggeredAbility {
                    event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                    effect: Effect::Scry {
                        who: PlayerRef::ControllerOf(Box::new(Selector::This)),
                        amount: Value::Const(2),
                    },
                }],
                ..Default::default()
            },
        )
    }
}

/// Ride the Avalanche — your next spell this turn has flash; when you next
/// cast a spell, up to one creature gets its mana value in +1/+1 counters.
pub fn ride_the_avalanche() -> CardDefinition {
    spell(
        "Ride the Avalanche",
        cost(&[g(), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::NextSpellHasFlashThisTurn,
            Effect::OnYourNextSpellCastThisTurn {
                body: Box::new(Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::Creature,
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::TriggerEventAmount,
                    }),
                }),
            },
        ]),
    )
}

/// Robe of Stars — +0/+3; {1}{W}: the equipped creature phases out. Equip {1}.
pub fn robe_of_stars() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PhaseOut {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                until_source_leaves: false,
            },
            ..Default::default()
        }],
        ..equipment(
            "Robe of Stars",
            cost(&[generic(1), w()]),
            cost(&[generic(1)]),
            EquipBonus { toughness: 3, ..Default::default() },
        )
    }
}

/// Song of Inspiration — up to two permanent cards from your graveyard to
/// hand; a d20 plus their total mana value at 15+ gains that much life.
/// ⚠ The cards return before the roll.
pub fn song_of_inspiration() -> CardDefinition {
    let total = || Value::TotalManaValueOf(Box::new(Selector::LastMoved));
    spell(
        "Song of Inspiration",
        cost(&[generic(3), g(), g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::PermanentCard.and(R::InYourGraveyard),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
            roll(
                20,
                total(),
                vec![(15, 255, Effect::GainLife { who: Selector::You, amount: total() })],
            ),
        ]),
    )
}

/// Storvald, Frost Giant Jarl — ward {3}, and your other creatures have it;
/// entering or attacking, one or both: a creature is 7/7, a creature is 1/1.
pub fn storvald_frost_giant_jarl() -> CardDefinition {
    let resize = || Effect::ChooseN {
        picks: vec![0, 1],
        modes: vec![
            Effect::SetBasePT {
                what: target_filtered(R::Creature),
                power: Value::Const(7),
                toughness: Value::Const(7),
                duration: Duration::EndOfTurn,
            },
            Effect::SetBasePT {
                what: target_filtered(R::Creature),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        ],
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Ward(WardCost::generic(3))],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have ward {3}.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::OtherThanSource)),
                keyword: Keyword::Ward(WardCost::generic(3)),
            },
        }],
        triggered_abilities: vec![etb(resize()), on_attack(resize())],
        ..creature(
            "Storvald, Frost Giant Jarl",
            cost(&[generic(4), g(), w(), u()]),
            vec![CreatureType::Giant],
            7,
            7,
        )
    }
}

/// Valiant Endeavor — roll two d6: destroy each creature with power at least
/// one result, then make Knights equal to the other.
pub fn valiant_endeavor() -> CardDefinition {
    let knight = TokenDefinition {
        name: "Knight".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Knight], ..Default::default() },
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    };
    spell(
        "Valiant Endeavor",
        cost(&[generic(4), w(), w()]),
        CardType::Sorcery,
        Effect::RollTwoDiceAssign {
            sides: 6,
            first: Box::new(Effect::DestroyEachMatchingWithPowerAtLeast {
                filter: R::Creature,
                value: Value::LastDieRoll,
            }),
            second: Box::new(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::LastDieRoll,
                definition: Arc::new(knight),
            }),
        },
    )
}
