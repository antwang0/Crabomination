//! Commander: the cards the **Wade into Battle** precon (C15, Kalemne,
//! Disciple of Iroas) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kalemne.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, Predicate, ZoneDest};
use crate::mana::{Color, ManaCost, cost, generic, r, w};
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

fn giant() -> R {
    R::HasCreatureType(CreatureType::Giant)
}

fn giants_you_control() -> Value {
    Value::CountOf(Box::new(Selector::EachPermanent(giant().and(R::Creature).and(R::ControlledByYou))))
}

/// Anya, Merciless Angel — +3/+3 per opponent below half their starting life;
/// indestructible while any is.
pub fn anya_merciless_angel() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Anya gets +3/+3 for each opponent whose life total is less than half their starting life total.",
                effect: StaticEffect::PumpSelfByValue {
                    amount: Value::OpponentsBelowHalfStartingLife,
                    per_power: 3,
                    per_toughness: 3,
                },
            },
            StaticAbility {
                description: "As long as an opponent's life total is less than half their starting life total, Anya has indestructible.",
                effect: StaticEffect::SelfHasKeywordWhilePredicate {
                    keyword: Keyword::Indestructible,
                    condition: Predicate::ValueAtLeast(Value::OpponentsBelowHalfStartingLife, Value::ONE),
                },
            },
        ],
        ..creature("Anya, Merciless Angel", cost(&[generic(3), r(), w()]), vec![CreatureType::Angel], 4, 4)
    }
}

/// Arbiter of Knollridge — ETB each player's life total becomes the highest.
pub fn arbiter_of_knollridge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::SetLifeTotal {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::HighestLifeTotal,
        })],
        ..creature(
            "Arbiter of Knollridge",
            cost(&[generic(6), w()]),
            vec![CreatureType::Giant, CreatureType::Wizard],
            5,
            5,
        )
    }
}

/// Borderland Behemoth — trample; +4/+4 for each other Giant you control.
pub fn borderland_behemoth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +4/+4 for each other Giant you control.",
            effect: StaticEffect::PumpSelfByValue {
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    giant().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource),
                ))),
                per_power: 4,
                per_toughness: 4,
            },
        }],
        ..creature(
            "Borderland Behemoth",
            cost(&[generic(5), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            4,
            4,
        )
    }
}

/// Curse of the Nightly Hunt — the enchanted player's creatures attack each
/// combat if able.
pub fn curse_of_the_nightly_hunt() -> CardDefinition {
    CardDefinition {
        name: "Curse of the Nightly Hunt",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        static_abilities: vec![StaticAbility {
            description: "Creatures enchanted player controls attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::ControlledBy { who: PlayerRef::EnchantedPlayer, filter: R::Creature },
                keyword: Keyword::MustAttack,
            },
        }],
        ..Default::default()
    }
}

/// Disaster Radius — reveal a creature card from hand; X damage to each
/// creature your opponents control, X its mana value.
pub fn disaster_radius() -> CardDefinition {
    CardDefinition {
        name: "Disaster Radius",
        cost: cost(&[generic(5), r(), r()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::RevealFromHand { filter: R::Creature }],
        effect: Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
            amount: Value::RevealedForCostManaValue,
        },
        ..Default::default()
    }
}

/// Dream Pillager — flying; combat damage to a player exiles that many cards
/// off your library, castable (not playable) this turn.
pub fn dream_pillager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::ExileTopAndGrantMayPlay {
                who: PlayerRef::You,
                count: Value::TriggerEventAmount,
                duration: MayPlayDuration::EndOfThisTurn,
                pay_any_color: false,
                max_mana_value: None,
                pay_own_cost: true,
                uncast_penalty: None,
            },
                Effect::RestrictMayPlayToCasting { what: Selector::ExiledThisResolution { filter: R::Any } },
            ]),
        }],
        ..creature("Dream Pillager", cost(&[generic(5), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Hamletback Goliath — another creature entering may put its power in
/// +1/+1 counters on this.
pub fn hamletback_goliath() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature.and(R::OtherThanSource) },
            ),
            effect: Effect::MayDo {
                description: "Put +1/+1 counters equal to its power on Hamletback Goliath?".into(),
                body: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                }),
            },
        }],
        ..creature(
            "Hamletback Goliath",
            cost(&[generic(6), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            6,
            6,
        )
    }
}

/// Hostility — haste; your spells' damage to opponents becomes 3/1 hasty
/// Elemental Shamans; put into a graveyard from anywhere, it shuffles home.
pub fn hostility() -> CardDefinition {
    let shaman = TokenDefinition {
        name: "Elemental Shaman".into(),
        power: 3,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Shaman],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "If a spell you control would deal damage to an opponent, prevent that damage. \
                          Create a 3/1 red Elemental Shaman creature token with haste for each 1 damage prevented this way.",
            effect: StaticEffect::SpellDamageToOpponentsBecomesTokens { token: Arc::new(shaman) },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::This)),
                    pos: crate::effect::LibraryPosition::Shuffled,
                },
            },
        }],
        ..creature(
            "Hostility",
            cost(&[generic(3), r(), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Incarnation],
            6,
            6,
        )
    }
}

/// Kalemne's Captain — vigilance; monstrosity 3; becoming monstrous exiles
/// all artifacts and enchantments.
pub fn kalemnes_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), w(), w()]),
            effect: Effect::Monstrosity { n: Value::Const(3) },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameMonstrous, EventScope::SelfSource),
            effect: Effect::Exile { what: Selector::EachPermanent(R::Artifact.or(R::Enchantment)) },
        }],
        ..creature(
            "Kalemne's Captain",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Giant, CreatureType::Soldier],
            5,
            5,
        )
    }
}

/// Magma Giant — ETB 2 damage to each creature and each player.
pub fn magma_giant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage { to: Selector::EachPermanent(R::Creature), amount: Value::Const(2) },
            Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(2) },
        ]))],
        ..creature("Magma Giant", cost(&[generic(5), r(), r()]), vec![CreatureType::Giant], 5, 5)
    }
}

/// Stinkdrinker Daredevil — Giant spells you cast cost {2} less.
pub fn stinkdrinker_daredevil() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Giant spells you cast cost {2} less to cast.",
            effect: StaticEffect::CostReduction { filter: giant(), amount: 2 },
        }],
        ..creature(
            "Stinkdrinker Daredevil",
            cost(&[generic(2), r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            1,
            3,
        )
    }
}

/// Sunrise Sovereign — other Giants you control get +2/+2 and trample.
pub fn sunrise_sovereign() -> CardDefinition {
    let others = || Selector::EachPermanent(giant().and(R::Creature).and(R::ControlledByYou).and(R::OtherThanSource));
    CardDefinition {
        static_abilities: vec![
            StaticAbility {
                description: "Other Giant creatures you control get +2/+2.",
                effect: StaticEffect::PumpPT { applies_to: others(), power: 2, toughness: 2 },
            },
            StaticAbility {
                description: "Other Giant creatures you control have trample.",
                effect: StaticEffect::GrantKeyword { applies_to: others(), keyword: Keyword::Trample },
            },
        ],
        ..creature(
            "Sunrise Sovereign",
            cost(&[generic(5), r()]),
            vec![CreatureType::Giant, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Thundercloud Shaman — ETB damage equal to your Giants to each non-Giant
/// creature.
pub fn thundercloud_shaman() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(giant())))),
            amount: giants_you_control(),
        })],
        ..creature(
            "Thundercloud Shaman",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Giant, CreatureType::Shaman],
            4,
            4,
        )
    }
}
