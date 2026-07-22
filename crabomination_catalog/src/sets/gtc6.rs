//! Gatecrash (GTC) wave 6: Bloodrush tricks, evasion/keyword Auras, a modal
//! pump, and combat triggers. Tests in `classic_sets/gtc`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Subtypes,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

fn creatures(t: Vec<CreatureType>) -> Subtypes {
    Subtypes { creature_types: t, ..Default::default() }
}
fn aura() -> Subtypes {
    Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() }
}

/// Bloodrush (CR ability word): a from-hand, discard-this-as-cost activated
/// ability that pumps a target attacking creature until end of turn.
fn bloodrush(mana: crate::mana::ManaCost, power: i32, toughness: i32, extra: Vec<Keyword>) -> ActivatedAbility {
    let mut body = vec![Effect::PumpPT {
        what: target_filtered(R::Creature.and(R::IsAttacking)),
        power: Value::Const(power),
        toughness: Value::Const(toughness),
        duration: Duration::EndOfTurn,
    }];
    body.extend(extra.into_iter().map(|k| Effect::GrantKeyword {
        what: Selector::Target(0),
        keyword: k,
        duration: Duration::EndOfTurn,
    }));
    ActivatedAbility {
        mana_cost: mana,
        from_hand: true,
        discard_self_cost: true,
        effect: Effect::Seq(body),
        ..Default::default()
    }
}

/// Scab-Clan Charger — {3}{G} 2/4 Centaur Warrior. Bloodrush — {1}{G}: target
/// attacking creature gets +2/+4.
pub fn scab_clan_charger() -> CardDefinition {
    CardDefinition {
        name: "Scab-Clan Charger",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Centaur, CreatureType::Warrior]),
        power: 2,
        toughness: 4,
        activated_abilities: vec![bloodrush(cost(&[generic(1), g()]), 2, 4, vec![])],
        ..Default::default()
    }
}

/// Scorchwalker — {3}{R} 5/1 Elemental. Bloodrush — {1}{R}{R}: target attacking
/// creature gets +5/+1.
pub fn scorchwalker() -> CardDefinition {
    CardDefinition {
        name: "Scorchwalker",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elemental]),
        power: 5,
        toughness: 1,
        activated_abilities: vec![bloodrush(cost(&[generic(1), r(), r()]), 5, 1, vec![])],
        ..Default::default()
    }
}

/// Leyline Phantom — {4}{U} 5/5 Illusion. When it deals combat damage, return it
/// to its owner's hand.
pub fn leyline_phantom() -> CardDefinition {
    let bounce = || TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
        effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
    };
    let bounce_creature = TriggeredAbility {
        event: EventSpec::new(EventKind::DealsCombatDamageToCreature, EventScope::SelfSource),
        effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
    };
    CardDefinition {
        name: "Leyline Phantom",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Illusion]),
        power: 5,
        toughness: 5,
        triggered_abilities: vec![bounce(), bounce_creature],
        ..Default::default()
    }
}

/// Martial Glory — {R}{W} Instant. Choose one — target creature gets +3/+0; or
/// target creature gets +0/+3.
pub fn martial_glory() -> CardDefinition {
    let pump = |p, t| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Martial Glory",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![pump(3, 0), pump(0, 3)]),
        ..Default::default()
    }
}

/// Alpha Authority — {1}{G} Aura. Enchanted creature has hexproof and can't be
/// blocked by more than one creature.
pub fn alpha_authority() -> CardDefinition {
    CardDefinition {
        name: "Alpha Authority",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![Keyword::Hexproof, Keyword::CantBeBlockedByMoreThanOne],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Agoraphobia — {1}{U} Aura. Enchanted creature gets -5/-0. {2}{U}: Return this
/// Aura to its owner's hand.
pub fn agoraphobia() -> CardDefinition {
    CardDefinition {
        name: "Agoraphobia",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: aura(),
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power: -5, ..Default::default() }),
        // The activation is the Aura's own ("Return *this Aura* to hand").
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Greenside Watcher — {1}{G} 2/1 Elf Druid. {T}: Untap target Gate.
pub fn greenside_watcher() -> CardDefinition {
    CardDefinition {
        name: "Greenside Watcher",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Elf, CreatureType::Druid]),
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Untap { what: target_filtered(R::HasLandType(LandType::Gate)), up_to: None },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Slate Street Ruffian — {2}{B} 2/2 Human Warrior. Whenever it becomes blocked,
/// the defending player discards a card.
pub fn slate_street_ruffian() -> CardDefinition {
    CardDefinition {
        name: "Slate Street Ruffian",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: creatures(vec![CreatureType::Human, CreatureType::Warrior]),
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::ONE,
                random: false,
            },
        }],
        ..Default::default()
    }
}
