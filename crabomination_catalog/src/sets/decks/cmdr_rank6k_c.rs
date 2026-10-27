//! Commander: three-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), third file. Tests in `tests/recent_b/cmdr_rank6k_c.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, evolve, target_any, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, ManaSymbol, cost, g, generic, w, x};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn power_up(mana: ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        power_up: true,
        effect,
        ..Default::default()
    }
}

fn counters(n: i32) -> Effect {
    Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn ward(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature has protection from that color. This effect doesn't remove this Aura.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                keyword: Keyword::Protection(color),
            },
        }],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        ..spell(
            name,
            cost(&[w()]),
            CardType::Enchantment,
            Effect::Attach {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
        )
    }
}

fn grant_eot(keyword: Keyword) -> Effect {
    Effect::GrantKeyword {
        what: Selector::This,
        keyword,
        duration: Duration::EndOfTurn,
    }
}

// ── Hawkeye, Clint Barton ────────────────────────────────────────────────────

/// Hawkeye, Clint Barton — {3}{W} legendary 3/5 vigilance Human Archer Hero.
pub fn hawkeye_clint_barton() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..legend(
            "Hawkeye, Clint Barton",
            cost(&[generic(3), w()]),
            vec![
                CreatureType::Human,
                CreatureType::Archer,
                CreatureType::Hero,
            ],
            3,
            5,
        )
    }
}

/// Black Ward — {W} Aura: enchanted creature has protection from black.
pub fn black_ward() -> CardDefinition {
    ward("Black Ward", Color::Black)
}

/// Green Ward — {W} Aura: enchanted creature has protection from green.
pub fn green_ward() -> CardDefinition {
    ward("Green Ward", Color::Green)
}

// ── Hercules, Prince of Power ────────────────────────────────────────────────

/// Hercules, Prince of Power — {2}{G} legendary 3/3 Demigod Warrior Hero.
/// Power-up {4}{G}: a +1/+1 counter on him; he gains vigilance, indestructible
/// and haste until end of turn.
pub fn hercules_prince_of_power() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![power_up(
            cost(&[generic(4), g()]),
            Effect::Seq(vec![
                counters(1),
                grant_eot(Keyword::Vigilance),
                grant_eot(Keyword::Indestructible),
                grant_eot(Keyword::Haste),
            ]),
        )],
        ..legend(
            "Hercules, Prince of Power",
            cost(&[generic(2), g()]),
            vec![
                CreatureType::Demigod,
                CreatureType::Warrior,
                CreatureType::Hero,
            ],
            3,
            3,
        )
    }
}

/// Gamma Grotesque — {2}{G} 3/3 vigilance Gamma Horror Villain. Power-up
/// {4}{G}{G}: three +1/+1 counters on it, then draw a card for each creature
/// you control with a counter on it.
pub fn gamma_grotesque() -> CardDefinition {
    let countered =
        Selector::EachPermanent(R::Creature.and(R::ControlledByYou).and(R::WithAnyCounter));
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![power_up(
            cost(&[generic(4), g(), g()]),
            Effect::Seq(vec![
                counters(3),
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::CountOf(Box::new(countered)),
                },
            ]),
        )],
        ..creature(
            "Gamma Grotesque",
            cost(&[generic(2), g()]),
            vec![
                CreatureType::Gamma,
                CreatureType::Horror,
                CreatureType::Villain,
            ],
            3,
            3,
        )
    }
}

// ── Everett K. Ross, Hapless Attaché ─────────────────────────────────────────

/// +2 Mace — {1}{W} Equipment: +2/+2. Equip {3}.
pub fn plus_two_mace() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            ..Default::default()
        }),
        ..spell(
            "+2 Mace",
            cost(&[generic(1), w()]),
            CardType::Artifact,
            Effect::Noop,
        )
    }
}

/// Scrying Sheets — Snow Land. {T}: add {C}. {1}{S}, {T}: look at the top card
/// of your library; if it's snow, you may reveal it and put it into your hand.
pub fn scrying_sheets() -> CardDefinition {
    let top = || Selector::TopOfLibrary {
        who: PlayerRef::You,
        count: Value::ONE,
    };
    CardDefinition {
        name: "Scrying Sheets",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Snow],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: crate::effect::ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: ManaCost {
                    symbols: vec![generic(1), ManaSymbol::Snow],
                },
                tap_cost: true,
                effect: Effect::If {
                    cond: Predicate::EntityMatches {
                        what: top(),
                        filter: R::HasSupertype(Supertype::Snow),
                    },
                    then: Box::new(Effect::MayDo {
                        description: "Reveal the snow card and put it into your hand?".into(),
                        body: Box::new(Effect::Move {
                            what: top(),
                            to: ZoneDest::Hand(PlayerRef::You),
                        }),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Blessed Reversal — {1}{W} instant: gain 3 life for each creature attacking
/// you.
pub fn blessed_reversal() -> CardDefinition {
    let attackers = Selector::EachPermanent(R::Creature.and(R::IsAttackingYou));
    spell(
        "Blessed Reversal",
        cost(&[generic(1), w()]),
        CardType::Instant,
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Times(
                Box::new(Value::CountOf(Box::new(attackers))),
                Box::new(Value::Const(3)),
            ),
        },
    )
}

// ── Doctor Jane Foster ───────────────────────────────────────────────────────

/// Doctor Jane Foster — {3}{W} legendary 2/3 vigilance Human Doctor. ETB:
/// return target creature card with mana value 3 or less from your graveyard
/// to your hand; if you gained life this turn, to the battlefield instead.
pub fn doctor_jane_foster() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        // "Return that card to the battlefield instead" — one move, straight
        // from the graveyard, never through the hand.
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::PlayerGainedLifeThisTurn { who: PlayerRef::You },
            then: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).from_your_graveyard()),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
            else_: Box::new(Effect::Move {
                what: target_filtered(R::Creature.and(R::ManaValueAtMost(3)).from_your_graveyard()),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..legend(
            "Doctor Jane Foster",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            2,
            3,
        )
    }
}

/// Heal — {W} instant. Prevent the next 1 damage to any target this turn. Draw
/// a card at the beginning of the next turn's upkeep.
pub fn heal() -> CardDefinition {
    spell(
        "Heal",
        cost(&[w()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::PreventNextDamage {
                target: target_any(),
                amount: Value::ONE,
            },
            Effect::AtNextTurnsUpkeep {
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        ]),
    )
}

/// Healing Hands — {2}{W} sorcery: target player gains 4 life; draw a card.
pub fn healing_hands() -> CardDefinition {
    spell(
        "Healing Hands",
        cost(&[generic(2), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GainLife {
                who: target_filtered(R::Player),
                amount: Value::Const(4),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
    )
}

// ── Darwin, Adaptive Mutant ──────────────────────────────────────────────────

/// Darwin, Adaptive Mutant — {1}{G} legendary 2/1 Mutant Hero; evolve. Remove
/// two +1/+1 counters from him: indestructible until end of turn.
pub fn darwin_adaptive_mutant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![evolve()],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 2)),
            effect: grant_eot(Keyword::Indestructible),
            ..Default::default()
        }],
        ..legend(
            "Darwin, Adaptive Mutant",
            cost(&[generic(1), g()]),
            vec![CreatureType::Mutant, CreatureType::Hero],
            2,
            1,
        )
    }
}

/// Beorn's Hospitality — {1}{G} enchantment. Landfall — a +1/+1 counter on
/// target creature you control. {5}{G}{G}: it becomes a Bear creature with
/// power and toughness equal to the lands you control, re-read as lands
/// come and go (`SetBasePtLive`).
pub fn beorns_hospitality() -> CardDefinition {
    let lands = || {
        Value::CountOf(Box::new(Selector::EachPermanent(
            R::Land.and(R::ControlledByYou),
        )))
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Land,
                }),
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), g(), g()]),
            // The gained P/T ability keeps counting (`SetBasePtLive`).
            effect: Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: Selector::This,
                    power: Value::Const(0),
                    toughness: Value::Const(0),
                    creature_types: vec![CreatureType::Bear],
                    keywords: vec![],
                    duration: Duration::Permanent,
                },
                Effect::SetBasePtLive { what: Selector::This, power: lands(), toughness: lands(), duration: Duration::Permanent },
            ]),
            ..Default::default()
        }],
        ..spell(
            "Beorn's Hospitality",
            cost(&[generic(1), g()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Wurmcalling — {X}{G} sorcery; buyback {2}{G}. Create an X/X green Wurm.
pub fn wurmcalling() -> CardDefinition {
    let wurm = TokenDefinition {
        name: "Wurm".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        dynamic_pt: Some((Value::XFromCost, Value::XFromCost)),
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Buyback(cost(&[generic(2), g()]))],
        ..spell(
            "Wurmcalling",
            cost(&[x(), g()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(wurm),
            },
        )
    }
}
