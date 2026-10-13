//! Commander: EDHREC ranks 2001-2500, two-card gaps — Anax and Cymede (Cleon;
//! Emerge Unscathed is `cmdr_rank2k_j`'s), Fumiko the Lowblood (Bloodmark Mentor, Slumbering
//! Dragon), Celestial Kirin (Katilda, Faith's Reward) and Kataki (Treasure
//! Keeper, Faith's Reward). Tests in `tests/recent_b/cmdr_edhrec61.rs`.

use crate::card::{
    CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{heroic, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, r, w, Color, ManaCost};

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

/// Cleon, Merry Champion — {2}{R} legendary 2/2 Human Soldier, double strike.
/// Heroic: exile the top card of your library; you may play it until the end
/// of your next turn.
pub fn cleon_merry_champion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![heroic(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        ..legend("Cleon, Merry Champion", cost(&[generic(2), r()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 2)
    }
}

/// Bloodmark Mentor — {1}{R} 1/1 Goblin Warrior. Red creatures you control
/// have first strike.
pub fn bloodmark_mentor() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Red creatures you control have first strike.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Red)).and(R::ControlledByYou)),
                keyword: Keyword::FirstStrike,
            },
        }],
        ..creature("Bloodmark Mentor", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 1, 1)
    }
}

/// Slumbering Dragon — {R} 3/3 Dragon, flying. It can't attack or block
/// unless it has five or more +1/+1 counters. Whenever a creature attacks you
/// or a planeswalker you control, put a +1/+1 counter on it.
pub fn slumbering_dragon() -> CardDefinition {
    let asleep = || R::IsSource.and(R::WithCounterAtLeast(CounterType::PlusOnePlusOne, 5).negate());
    let lock = |keyword| StaticAbility {
        description: "This creature can't attack or block unless it has five or more +1/+1 counters on it.",
        effect: StaticEffect::GrantKeyword { applies_to: Selector::EachPermanent(asleep()), keyword },
    };
    let grow = || Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![lock(Keyword::CantAttack), lock(Keyword::CantBlock)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
                effect: grow(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::ControllerPlaneswalkerAttackedByOpponent),
                effect: grow(),
            },
        ],
        ..creature("Slumbering Dragon", cost(&[r()]), vec![CreatureType::Dragon], 3, 3)
    }
}

/// Faith's Reward — {3}{W} instant. Return to the battlefield all permanent
/// cards in your graveyard put there from the battlefield this turn.
pub fn faiths_reward() -> CardDefinition {
    CardDefinition {
        name: "Faith's Reward",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::Permanent.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// Treasure Keeper — {4} 3/3 artifact creature Construct. When it dies,
/// reveal from the top until a nonland card with mana value 3 or less; you may
/// cast it free; the rest go to the bottom at random. 🟡 The walk exiles the
/// cards as cascade does rather than revealing them (INCOMPLETE_CARDS).
pub fn treasure_keeper() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_dies(Effect::Cascade { max_mv: Value::Const(4), filter: None })],
        ..creature("Treasure Keeper", cost(&[generic(4)]), vec![CreatureType::Construct], 3, 3)
    }
}

/// Katilda, Dawnhart Martyr // Katilda's Rising Dawn — {1}{W}{W} legendary */*
/// Spirit Warlock with flying, lifelink and protection from Vampires, P/T the
/// number of permanents you control that are Spirits and/or enchantments.
/// Disturb {3}{W}{W}: a legendary Aura granting the same and +X/+X.
pub fn katilda_dawnhart_martyr() -> CardDefinition {
    let x = || {
        Value::count(Selector::ControlledBy {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Spirit).or(R::Enchantment),
        })
    };
    let kws = || vec![Keyword::Flying, Keyword::Lifelink, Keyword::ProtectionFromCreatureType(CreatureType::Vampire)];
    let dawn = CardDefinition {
        name: "Katilda's Rising Dawn",
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        color_indicator: vec![Color::White],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: kws(), ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Enchanted creature gets +X/+X, where X is the number of permanents you control that are \
                          Spirits and/or enchantments.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
                power: x(),
                toughness: x(),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: [kws(), vec![Keyword::Disturb(cost(&[generic(3), w(), w()]))]].concat(),
        static_abilities: vec![StaticAbility {
            description: "Katilda's power and toughness are each equal to the number of permanents you control that \
                          are Spirits and/or enchantments.",
            effect: StaticEffect::SelfBasePtFromValue { power: x(), toughness: x() },
        }],
        back_face: Some(Box::new(dawn)),
        ..legend(
            "Katilda, Dawnhart Martyr",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Spirit, CreatureType::Warlock],
            0,
            0,
        )
    }
}
