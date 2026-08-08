//! Homelands (HML) — closing wave. Tests in `classic_sets/hml`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::target_filtered,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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

fn aura(name: &'static str, c: ManaCost, enchants: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchants) },
        ..Default::default()
    }
}

/// The Homelands cantrip rider: "Draw a card at the beginning of the next
/// turn's upkeep."
fn cantrip_next_upkeep() -> Effect {
    Effect::AtNextTurnsUpkeep {
        body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
    }
}

/// Headstone — exile a card from a graveyard, then cantrip next upkeep.
pub fn headstone() -> CardDefinition {
    CardDefinition {
        name: "Headstone",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::Exile },
            cantrip_next_upkeep(),
        ]),
        ..Default::default()
    }
}

/// Prophecy — peek at an opponent's top card, gain a life off a land, then
/// they shuffle; cantrip next upkeep.
pub fn prophecy() -> CardDefinition {
    CardDefinition {
        name: "Prophecy",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::RevealTopThenShuffle {
                who: PlayerRef::Target(0),
                filter: R::Land,
                on_match: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
            cantrip_next_upkeep(),
        ]),
        ..Default::default()
    }
}

/// Renewal — sacrifice a land to fetch a basic onto the battlefield, then
/// cantrip next upkeep.
pub fn renewal() -> CardDefinition {
    CardDefinition {
        name: "Renewal",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            cantrip_next_upkeep(),
        ]),
        ..Default::default()
    }
}

/// Leeches — strip a player's poison counters and burn them for that much.
pub fn leeches() -> CardDefinition {
    CardDefinition {
        name: "Leeches",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                amount: Value::PoisonCountersOf(PlayerRef::Target(0)),
                to: target_filtered(R::Player),
            },
            Effect::RemoveAllPoison { who: PlayerRef::Target(0) },
        ]),
        ..Default::default()
    }
}

/// Baki's Curse — 2 damage to each creature for each Aura on it.
pub fn bakis_curse() -> CardDefinition {
    CardDefinition {
        name: "Baki's Curse",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DamageEachCreaturePerAura { amount: Value::Const(2) },
        ..Default::default()
    }
}

/// Sea Troll — a regenerator, but only after meeting a blue creature in
/// combat.
pub fn sea_troll() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Regenerate { what: Selector::This },
            condition: Some(Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::BlockingOrBlockedBySource.and(R::HasColor(Color::Blue)),
                ),
                n: Value::ONE,
            }),
            ..Default::default()
        }],
        ..creature("Sea Troll", cost(&[generic(2), u()]), vec![CreatureType::Troll], 2, 1)
    }
}

/// Baron Sengir — grows off every creature its damage kills, and keeps the
/// rest of the Vampires alive.
pub fn baron_sengir() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::DamagedBySourceThisTurn,
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusTwoPlusTwo,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Regenerate {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Vampire).and(R::OtherThanSource),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Baron Sengir",
            cost(&[generic(5), b(), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Noble],
            5,
            5,
        )
    }
}

/// Black Carriage — a trampler that only untaps by eating a creature during
/// your upkeep.
pub fn black_carriage() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step.",
            effect: StaticEffect::PreventUntap { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Untap { what: Selector::This, up_to: None },
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            ..Default::default()
        }],
        ..creature("Black Carriage", cost(&[generic(3), b(), b()]), vec![CreatureType::Horse], 4, 4)
    }
}

/// Serra Bestiary — an upkeep tax that pins the enchanted creature down.
pub fn serra_bestiary() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[w(), w()]) },
        }],
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateTapAbilities,
            ],
            ..Default::default()
        }),
        ..aura("Serra Bestiary", cost(&[w(), w()]), R::Creature)
    }
}

/// Ironclaw Curse — shrinks the enchanted creature and keeps it off big
/// attackers.
pub fn ironclaw_curse() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            toughness: -1,
            keywords: vec![Keyword::CantBlockPowerAtLeastOwnToughness],
            ..Default::default()
        }),
        ..aura("Ironclaw Curse", cost(&[r()]), R::Creature)
    }
}

/// Mammoth Harness — grounds the enchanted creature and arms whatever it
/// meets in combat.
pub fn mammoth_harness() -> CardDefinition {
    CardDefinition {
        equipped_bonus: Some(EquipBonus {
            remove_keywords: vec![Keyword::Flying],
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                    effect: Effect::GrantKeyword {
                        what: Selector::BlockedAttacker,
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    },
                },
                TriggeredAbility {
                    event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                    effect: Effect::GrantKeyword {
                        what: Selector::BlockingCreatures,
                        keyword: Keyword::FirstStrike,
                        duration: Duration::EndOfTurn,
                    },
                },
            ],
            ..Default::default()
        }),
        ..aura("Mammoth Harness", cost(&[generic(3), g()]), R::Creature)
    }
}

/// Trade Caravan — banks currency and unlocks a basic land on an opponent's
/// upkeep.
pub fn trade_caravan() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::Currency,
                amount: Value::ONE,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            remove_counter_cost: Some((CounterType::Currency, 2)),
            effect: Effect::Untap { what: target_filtered(R::IsBasicLand), up_to: None },
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            ..Default::default()
        }],
        ..creature(
            "Trade Caravan",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            1,
            1,
        )
    }
}

/// An-Zerrin Ruins — name a creature type and freeze it solid.
pub fn an_zerrin_ruins() -> CardDefinition {
    CardDefinition {
        name: "An-Zerrin Ruins",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::NameCreatureType { what: Selector::This },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures of the chosen type don't untap during their \
                          controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::IsSourceChosenCreatureType),
            },
        }],
        ..Default::default()
    }
}

/// Koskun Falls — a World enchantment that taxes attackers and demands a
/// tapped creature every upkeep.
pub fn koskun_falls() -> CardDefinition {
    CardDefinition {
        name: "Koskun Falls",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::World],
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::SacrificeSourceUnlessTapCreature,
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you unless their controller pays {2} for each.",
            effect: StaticEffect::AttackTaxToController {
                amount: Value::Const(2),
                protect_planeswalkers: false,
                            filter: None,
            },
        }],
        ..Default::default()
    }
}

/// Daughter of Autumn — soaks a point of damage aimed at any white creature.
pub fn daughter_of_autumn() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::RedirectNextDamage {
                target: target_filtered(R::Creature.and(R::HasColor(Color::White))),
                to: Selector::This,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature(
            "Daughter of Autumn",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Avatar],
            2,
            4,
        )
    }
}

/// Hazduhr the Abbot — the same shield, sized by {X} and limited to your own
/// white creatures.
pub fn hazduhr_the_abbot() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[x()]),
            effect: Effect::RedirectNextDamage {
                target: target_filtered(
                    R::Creature.and(R::HasColor(Color::White)).and(R::ControlledByYou),
                ),
                to: Selector::This,
                amount: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..creature(
            "Hazduhr the Abbot",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            5,
        )
    }
}

/// Truce — everyone draws up to two, and is paid two life per card declined.
pub fn truce() -> CardDefinition {
    CardDefinition {
        name: "Truce",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::EachPlayerDrawsUpToElseGainsLife { max: 2, life_per_card: 2 },
        ..Default::default()
    }
}

/// Drudge Spell — turns your graveyard into regenerating Skeletons, and takes
/// them with it.
pub fn drudge_spell() -> CardDefinition {
    CardDefinition {
        name: "Drudge Spell",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            exile_other_filter: Some((R::HasCardType(CardType::Creature), 2)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Box::new(TokenDefinition {
                    name: "Skeleton".into(),
                    power: 1,
                    toughness: 1,
                    colors: vec![Color::Black],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Skeleton],
                        ..Default::default()
                    },
                    activated_abilities: vec![ActivatedAbility {
                        mana_cost: cost(&[b()]),
                        effect: Effect::Regenerate { what: Selector::This },
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
            effect: Effect::DestroyNoRegen {
                what: Selector::EachPermanent(
                    R::IsToken.and(R::HasCreatureType(CreatureType::Skeleton)),
                ),
            },
        }],
        ..Default::default()
    }
}
