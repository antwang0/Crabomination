//! Commander: the 2026-10-13 rescan's one-card gaps that needed more than a
//! catalog entry — Raised by Giants, Kamahl's Druidic Vow, Demonic Embrace
//! and friends (`cmdr_edhrec22` is batch A). Tests in
//! `tests/recent_b/cmdr_edhrec23.rs`.

use crate::card::{
    AlternativeCost, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LookPick, PlayerRef};
use crate::mana::{b, cost, g, generic, r, x, ManaCost};

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

fn your_commanders() -> Selector {
    Selector::EachPermanent(R::Creature.and(R::IsCommander).and(R::OwnedByYou))
}

/// Raised by Giants — {5}{G} legendary enchantment — Background. Commander
/// creatures you own have base power and toughness 10/10 and are Giants.
pub fn raised_by_giants() -> CardDefinition {
    CardDefinition {
        name: "Raised by Giants",
        cost: cost(&[generic(5), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Background], ..Default::default() },
        static_abilities: vec![
            StaticAbility {
                description: "Commander creatures you own have base power and toughness 10/10.",
                effect: StaticEffect::SetBasePtForFilter { applies_to: your_commanders(), power: 10, toughness: 10 },
            },
            StaticAbility {
                description: "Commander creatures you own are Giants in addition to their other types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: your_commanders(),
                    creature_type: CreatureType::Giant,
                },
            },
        ],
        ..Default::default()
    }
}

/// Kamahl's Druidic Vow — {X}{G}{G} legendary sorcery. Look at the top X;
/// put any number of land and/or legendary permanent cards with mana value X
/// or less onto the battlefield; the rest into your graveyard.
pub fn kamahls_druidic_vow() -> CardDefinition {
    let legendary_permanent = R::PermanentCard.and(R::HasSupertype(Supertype::Legendary));
    CardDefinition {
        name: "Kamahl's Druidic Vow",
        cost: cost(&[x(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::XFromCost,
            take: Some(Value::XFromCost),
            pick_filter: Some(R::Land.or(legendary_permanent).and(R::ManaValueAtMostXFromCost)),
            to_battlefield: true,
            rest_to_graveyard: true,
            optional: true,
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Demonic Embrace — {1}{B}{B} Aura. Enchanted creature gets +3/+1, has
/// flying, and is a Demon too. You may cast it from your graveyard by paying
/// 3 life and discarding a card in addition to its other costs.
pub fn demonic_embrace() -> CardDefinition {
    let mana = cost(&[generic(1), b(), b()]);
    CardDefinition {
        name: "Demonic Embrace",
        cost: mana.clone(),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 3,
            toughness: 1,
            keywords: vec![Keyword::Flying],
            add_creature_types: vec![CreatureType::Demon],
            ..Default::default()
        }),
        alternative_cost: Some(AlternativeCost {
            from_graveyard: true,
            mana_cost: mana,
            life_cost: 3,
            discard_filters: vec![(R::Any, 1)],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Trepanation Blade — {3} Equipment. Whenever equipped creature attacks,
/// the defending player reveals until a land and bins it all; the creature
/// gets +1/+0 per card revealed. Equip {2}.
pub fn trepanation_blade() -> CardDefinition {
    CardDefinition {
        name: "Trepanation Blade",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::MillUntilLands { who: Selector::Player(PlayerRef::DefendingPlayer), lands: Value::ONE },
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::CardsMilledThisEffectMatching { filter: R::Any },
                        toughness: Value::Const(0),
                        duration: Duration::EndOfTurn,
                    },
                ]),
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Kraven the Hunter — {1}{B}{G} 4/3 legendary Human Warrior Villain,
/// trample. Whenever a creature an opponent controls with the greatest power
/// among creatures that player controls dies, draw a card and put a +1/+1
/// counter on Kraven.
pub fn kraven_the_hunter() -> CardDefinition {
    let died = || Selector::TriggerSource;
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl).with_filter(
                Predicate::ValueAtLeast(
                    Value::PowerOf(Box::new(died())),
                    Value::GreatestPowerControlled { who: PlayerRef::ControllerOf(Box::new(died())) },
                ),
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
        }],
        ..creature(
            "Kraven the Hunter",
            cost(&[generic(1), b(), g()]),
            vec![CreatureType::Human, CreatureType::Warrior, CreatureType::Villain],
            4,
            3,
        )
    }
}

/// Display of Power — {1}{R}{R} instant; can't be copied. Copy any number of
/// target instant and/or sorcery spells; you may choose new targets.
pub fn display_of_power() -> CardDefinition {
    CardDefinition {
        name: "Display of Power",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::CantBeCopied],
        effect: Effect::ApplyToTargets {
            max_targets: 8,
            min_targets: 0,
            filter: R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            effect: Box::new(Effect::CopySpellMayChooseTargets { what: Selector::Target(0), count: Value::ONE }),
        },
        ..Default::default()
    }
}

/// Fires of Invention — {3}{R} enchantment. You can cast spells only during
/// your turn and no more than two spells each turn. You may cast spells with
/// mana value up to the number of lands you control without paying their
/// mana costs.
pub fn fires_of_invention() -> CardDefinition {
    CardDefinition {
        name: "Fires of Invention",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "You can cast spells only during your turn and you can cast no more than two spells each turn.",
                effect: StaticEffect::ControllerCastsOnlyOnTurnUpTo { max: 2 },
            },
            StaticAbility {
                description: "You may cast spells with mana value less than or equal to the number of lands you control without paying their mana costs.",
                effect: StaticEffect::CastFilteredSpellsFree {
                    filter: R::ManaValueAtMostYourCount(Box::new(R::Land)),
                },
            },
        ],
        ..Default::default()
    }
}

/// Interdimensional Web Watch — {4} artifact. ETB: exile the top two cards
/// of your library; you may play them until the end of your next turn. {T}:
/// add two mana in any combination of colors, spent only on spells cast from
/// exile.
pub fn interdimensional_web_watch() -> CardDefinition {
    use crate::card::{ActivatedAbility, MayPlayDuration};
    use crate::effect::ManaPayload;
    CardDefinition {
        name: "Interdimensional Web Watch",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::Const(2),
            duration: MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::AnyColors(Value::Const(2))),
                    crate::mana::SpendRestriction::SpellFromExile,
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
