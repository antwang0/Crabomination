//! Retro batch 2 — flyers, Rampage giants, landwalk + protection, an
//! attack-despite-defender Wall, self-untap and evasion Auras, and classic
//! vanillas / French vanillas. Tests in `tests/recent78.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventScope, EventSpec, Keyword, LandType, Predicate, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_dies, target_filtered};
use crate::effect::{Duration, Effect, EventKind, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, Color, ManaCost};

/// Vanilla / French-vanilla creature helper.
fn vanilla(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
    keywords: Vec<Keyword>,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords,
        ..Default::default()
    }
}


/// Azure Drake — {3}{U} 2/4 Drake. Flying.
pub fn azure_drake() -> CardDefinition {
    vanilla("Azure Drake", cost(&[generic(3), u()]), vec![CreatureType::Drake], 2, 4, vec![Keyword::Flying])
}


/// Dakmor Bat — {1}{B} 1/1 Bat. Flying.
pub fn dakmor_bat() -> CardDefinition {
    vanilla("Dakmor Bat", cost(&[generic(1), b()]), vec![CreatureType::Bat], 1, 1, vec![Keyword::Flying])
}

/// War Mammoth — {3}{G} 3/3 Elephant. Trample.
pub fn war_mammoth() -> CardDefinition {
    vanilla("War Mammoth", cost(&[generic(3), g()]), vec![CreatureType::Elephant], 3, 3, vec![Keyword::Trample])
}

/// Viashino Warrior — {3}{R} 4/2 Lizard Warrior (vanilla).
pub fn viashino_warrior() -> CardDefinition {
    vanilla("Viashino Warrior", cost(&[generic(3), r()]), vec![CreatureType::Lizard, CreatureType::Warrior], 4, 2, vec![])
}

/// Barbtooth Wurm — {5}{G} 6/4 Wurm (vanilla).
pub fn barbtooth_wurm() -> CardDefinition {
    vanilla("Barbtooth Wurm", cost(&[generic(5), g()]), vec![CreatureType::Wurm], 6, 4, vec![])
}

/// Goblin Hero — {2}{R} 2/2 Goblin (vanilla).
pub fn goblin_hero() -> CardDefinition {
    vanilla("Goblin Hero", cost(&[generic(2), r()]), vec![CreatureType::Goblin], 2, 2, vec![])
}

/// Dread Reaper — {3}{B}{B}{B} 6/5 Horror. Flying. ETB: you lose 5 life.
pub fn dread_reaper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::LoseLife { who: Selector::You, amount: Value::Const(5) })],
        ..vanilla("Dread Reaper", cost(&[generic(3), b(), b(), b()]), vec![CreatureType::Horror], 6, 5, vec![Keyword::Flying])
    }
}

/// Foul Familiar — {2}{B} 3/1 Spirit. Can't block. {B}, Pay 1 life: Return
/// this creature to its owner's hand.
pub fn foul_familiar() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            life_cost: 1,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..vanilla("Foul Familiar", cost(&[generic(2), b()]), vec![CreatureType::Spirit], 3, 1, vec![Keyword::CantBlock])
    }
}

/// Fire Snake — {4}{R} 3/1 Snake. When it dies, destroy target land.
pub fn fire_snake() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Destroy { what: target_filtered(R::Land) })],
        ..vanilla("Fire Snake", cost(&[generic(4), r()]), vec![CreatureType::Snake], 3, 1, vec![])
    }
}

/// Elven Cache — {2}{G}{G} Sorcery. Return target card from your graveyard to
/// your hand.
pub fn elven_cache() -> CardDefinition {
    CardDefinition {
        name: "Elven Cache",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(R::InYourGraveyard),
            to: ZoneDest::Hand(PlayerRef::You),
        },
        ..Default::default()
    }
}





/// Talas Warrior — {1}{U}{U} 2/2 Human Pirate Warrior. Can't be blocked.
pub fn talas_warrior() -> CardDefinition {
    vanilla("Talas Warrior", cost(&[generic(1), u(), u()]),
        vec![CreatureType::Human, CreatureType::Pirate, CreatureType::Warrior], 2, 2, vec![Keyword::Unblockable])
}



/// Mountain Yeti — {2}{R}{R} 3/3 Yeti. Mountainwalk, protection from white.
pub fn mountain_yeti() -> CardDefinition {
    vanilla("Mountain Yeti", cost(&[generic(2), r(), r()]), vec![CreatureType::Yeti], 3, 3,
        vec![Keyword::Landwalk(LandType::Mountain), Keyword::Protection(Color::White)])
}

/// Giant Crab — {4}{U} 3/3 Crab. {U}: gains shroud until end of turn.
pub fn giant_crab() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Shroud, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..vanilla("Giant Crab", cost(&[generic(4), u()]), vec![CreatureType::Crab], 3, 3, vec![])
    }
}

/// Dwarven Soldier — {1}{R} 2/1 Dwarf Soldier. Whenever it blocks or becomes
/// blocked by one or more Orcs, it gets +0/+2 until end of turn.
pub fn dwarven_soldier() -> CardDefinition {
    let pump = || Effect::PumpPT { what: Selector::This, power: Value::Const(0), toughness: Value::Const(2), duration: Duration::EndOfTurn };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches { what: Selector::BlockedAttacker, filter: R::HasCreatureType(CreatureType::Orc) },
                ),
                effect: pump(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource).with_filter(
                    Predicate::EntityMatches { what: Selector::BlockingCreatures, filter: R::HasCreatureType(CreatureType::Orc) },
                ),
                effect: pump(),
            },
        ],
        ..vanilla("Dwarven Soldier", cost(&[generic(1), r()]), vec![CreatureType::Dwarf, CreatureType::Soldier], 2, 1, vec![])
    }
}

/// Erg Raiders — {1}{B} 2/3 Human Warrior. At your end step, if it didn't
/// attack this turn, it deals 2 damage to you — unless it entered this turn.
pub fn erg_raiders() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::Not(Box::new(Predicate::SourceAttackedThisTurn)),
                    Predicate::Not(Box::new(Predicate::EntityMatches { what: Selector::This, filter: R::EnteredThisTurn })),
                ])),
            effect: Effect::DealDamage { to: Selector::You, amount: Value::Const(2) },
        }],
        ..vanilla("Erg Raiders", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Warrior], 2, 3, vec![])
    }
}

/// Wall of Wonder — {2}{U}{U} 1/5 Wall. Defender. {2}{U}{U}: it gets +4/-4
/// until end of turn and can attack this turn as though it didn't have defender.
pub fn wall_of_wonder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u(), u()]),
            effect: Effect::Seq(vec![
                Effect::PumpPT { what: Selector::This, power: Value::Const(4), toughness: Value::Const(-4), duration: Duration::EndOfTurn },
                Effect::AttackDespiteDefenderThisTurn { what: Selector::This },
            ]),
            ..Default::default()
        }],
        ..vanilla("Wall of Wonder", cost(&[generic(2), u(), u()]), vec![CreatureType::Wall], 1, 5, vec![Keyword::Defender])
    }
}

/// Fear — {B}{B} Aura. Enchanted creature has fear.
pub fn fear() -> CardDefinition {
    CardDefinition {
        name: "Fear",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Fear], ..Default::default() }),
        ..Default::default()
    }
}

/// Instill Energy — {G} Aura. Enchanted creature can attack as though it had
/// haste; "{0}: Untap enchanted creature," once each turn during your turn.
pub fn instill_energy() -> CardDefinition {
    CardDefinition {
        name: "Instill Energy",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Haste], ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::Untap { what: Selector::AttachedTo(Box::new(Selector::This)), up_to: None },
            once_per_turn: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            ..Default::default()
        }],
        ..Default::default()
    }
}
