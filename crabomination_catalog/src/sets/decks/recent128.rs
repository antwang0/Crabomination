//! A Wilds of Eldraine (WOE) wave: Celebration, Bargain, and Role tokens.
//! Introduces `Predicate::CelebrationActive` (two nonland permanents entered
//! this turn). Tests in `crabomination/src/tests/recent128.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn monster_role() -> TokenDefinition {
    TokenDefinition {
        name: "Monster".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Trample],
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn young_hero_role() -> TokenDefinition {
    TokenDefinition {
        name: "Young Hero".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::White],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus {
            // Printed: "if its toughness is 3 or less, put a +1/+1 counter"; the
            // toughness gate is approximated as unconditional.
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn cursed_role() -> TokenDefinition {
    TokenDefinition {
        name: "Cursed".into(),
        card_types: vec![CardType::Enchantment],
        colors: vec![Color::Black],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Role],
            ..Default::default()
        },
        equipped_bonus: Some(EquipBonus { set_base_pt: Some((1, 1)), ..Default::default() }),
        ..Default::default()
    }
}

/// Armory Mice — {1}{W} 3/1 Mouse. Celebration — +0/+2 while two or more nonland
/// permanents entered under your control this turn.
pub fn armory_mice() -> CardDefinition {
    CardDefinition {
        name: "Armory Mice",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mouse], ..Default::default() },
        power: 3,
        toughness: 1,
        static_abilities: vec![StaticAbility {
            description: "Celebration — Armory Mice gets +0/+2 as long as two or more nonland permanents entered under your control this turn.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::CelebrationActive { who: PlayerRef::You },
                power: 0,
                toughness: 2,
                keywords: vec![],
            },
        }],
        ..Default::default()
    }
}

/// Belligerent of the Ball — {2}{R} 3/3 Ogre Warrior. Celebration — at combat on
/// your turn, if two nonland permanents entered this turn, a creature you control
/// gets +1/+0 and menace.
pub fn belligerent_of_the_ball() -> CardDefinition {
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Belligerent of the Ball",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::ActivePlayer)
                .with_filter(Predicate::CelebrationActive { who: PlayerRef::You }),
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::ONE,
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Menace,
                    duration: Duration::EndOfTurn,
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Archive Dragon — {4}{U}{U} 4/6 Dragon Wizard. Flying, Ward {2}. ETB: scry 2.
pub fn archive_dragon() -> CardDefinition {
    CardDefinition {
        name: "Archive Dragon",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::Mana(cost(&[generic(2)])))],
        triggered_abilities: vec![etb(Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) })],
        ..Default::default()
    }
}

/// Barrow Naughty — {1}{B} 1/3 Faerie with flying. {2}{B}: +1/+0 until end of
/// turn. (The "has lifelink while you control another Faerie" static is dropped
/// — no control-another-of-type filter primitive yet.)
pub fn barrow_naughty() -> CardDefinition {
    CardDefinition {
        name: "Barrow Naughty",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Agatha's Champion — {4}{G} 4/4 Human Knight with trample and Bargain. ETB, if
/// bargained: it fights up to one target creature you don't control.
pub fn agathas_champion() -> CardDefinition {
    CardDefinition {
        name: "Agatha's Champion",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Bargain],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasBargained),
            effect: Effect::Fight {
                attacker: Selector::This,
                defender: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..Default::default()
    }
}

/// Cut In — {3}{R} Sorcery. Deals 4 damage to target creature; create a Young
/// Hero Role token attached to up to one target creature you control.
pub fn cut_in() -> CardDefinition {
    CardDefinition {
        name: "Cut In",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(4) },
            Effect::CreateTokenAttachedTo {
                target: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::ControlledByYou) },
                definition: young_hero_role(),
            },
        ]),
        ..Default::default()
    }
}

/// Become Brutes — {1}{R} Sorcery. One or two target creatures each gain haste;
/// for each, create a Monster Role token attached to it.
pub fn become_brutes() -> CardDefinition {
    CardDefinition {
        name: "Become Brutes",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateTokenAttachedTo {
                target: Selector::Target(0),
                definition: monster_role(),
            },
        ]),
        ..Default::default()
    }
}

/// Diminisher Witch — {2}{U} 3/2 Human Warlock with Bargain. ETB, if bargained:
/// create a Cursed Role token attached to target creature an opponent controls.
pub fn diminisher_witch() -> CardDefinition {
    CardDefinition {
        name: "Diminisher Witch",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Bargain],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasBargained),
            effect: Effect::CreateTokenAttachedTo {
                target: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                definition: cursed_role(),
            },
        }],
        ..Default::default()
    }
}

/// Ego Drain — {B} Sorcery. Target opponent reveals their hand; you choose a
/// nonland card and they discard it. (The "if you don't control a Faerie, exile
/// a card from your hand" downside is dropped.)
pub fn ego_drain() -> CardDefinition {
    CardDefinition {
        name: "Ego Drain",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::ONE,
            filter: R::Nonland,
        },
        ..Default::default()
    }
}

/// Charging Hooligan — {3}{R} 3/3 Human Peasant. Whenever it attacks, it gets
/// +1/+0 for each attacking creature until end of turn.
pub fn charging_hooligan() -> CardDefinition {
    CardDefinition {
        name: "Charging Hooligan",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Peasant],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::IsAttacking)))),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

