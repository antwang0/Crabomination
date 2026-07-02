//! Retro batch — classic Auras (pump / drain / Forest-scaled), a can't-block
//! restriction (Ironclaw Orcs → `Keyword::CantBlockPowerAtLeast`), regenerating
//! Wall, deathtouch basilisks, mana Elf, artifacts (draw/ping/lifegain).
//! Tests in `tests/recent76.rs`.

use crate::card::SelectionRequirement as R;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::{deal, target_any, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Selector, Value};
use crate::mana::{b, cost, g, generic, r, w, Color, ManaCost};

/// Aura helper: enchant a creature and grant a flat P/T (plus keywords) bonus.
fn pump_aura(name: &'static str, mana: ManaCost, power: i32, toughness: i32, keywords: Vec<Keyword>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus { power, toughness, keywords, ..Default::default() }),
        ..Default::default()
    }
}

/// Giant Strength — {R}{R} Aura. Enchanted creature gets +2/+2.
pub fn giant_strength() -> CardDefinition {
    pump_aura("Giant Strength", cost(&[r(), r()]), 2, 2, vec![])
}

/// Web — {G} Aura. Enchanted creature gets +0/+2 and has reach.
pub fn web() -> CardDefinition {
    pump_aura("Web", cost(&[g()]), 0, 2, vec![Keyword::Reach])
}

/// Blanchwood Armor — {2}{G} Aura. Enchanted creature gets +1/+1 for each
/// Forest you control.
pub fn blanchwood_armor() -> CardDefinition {
    CardDefinition {
        name: "Blanchwood Armor",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            scale: Some(EquipScale {
                filter: R::HasLandType(LandType::Forest).and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
                count_self_counters: None,
                count_graveyard: None,
            }),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Wall of Brambles — {2}{G} 2/3 Plant Wall. Defender. {G}: Regenerate.
pub fn wall_of_brambles() -> CardDefinition {
    CardDefinition {
        name: "Wall of Brambles",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant, CreatureType::Wall], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ironclaw Orcs — {1}{R} 2/2 Orc. This creature can't block creatures with
/// power 2 or greater.
pub fn ironclaw_orcs() -> CardDefinition {
    CardDefinition {
        name: "Ironclaw Orcs",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Orc], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBlockPowerAtLeast(2)],
        ..Default::default()
    }
}

/// Dwarven Warriors — {2}{R} 1/1 Dwarf Warrior. {T}: Target creature with
/// power 2 or less can't be blocked this turn.
pub fn dwarven_warriors() -> CardDefinition {
    CardDefinition {
        name: "Dwarven Warriors",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Warrior], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Frozen Shade — {2}{B} 0/1 Shade. {B}: This creature gets +1/+1 until end of
/// turn.
pub fn frozen_shade() -> CardDefinition {
    CardDefinition {
        name: "Frozen Shade",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shade], ..Default::default() },
        power: 0,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Whirling Dervish — {G}{G} 1/1 Human Monk. Protection from black. Whenever it
/// deals combat damage to a player, put a +1/+1 counter on it. (Faithful to the
/// end-step "if it dealt damage this turn" grow; combat is the only source in
/// practice.)
pub fn whirling_dervish() -> CardDefinition {
    CardDefinition {
        name: "Whirling Dervish",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Monk], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Protection(Color::Black)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Femeref Archers — {2}{G} 2/2 Human Archer. {T}: Deals 4 damage to target
/// attacking creature with flying.
pub fn femeref_archers() -> CardDefinition {
    CardDefinition {
        name: "Femeref Archers",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Archer], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                amount: Value::Const(4),
                to: target_filtered(R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fyndhorn Elder — {2}{G} 1/1 Elf Druid. {T}: Add {G}{G}.
pub fn fyndhorn_elder() -> CardDefinition {
    CardDefinition {
        name: "Fyndhorn Elder",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Druid], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Green, Value::Const(2)) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thicket Basilisk — {3}{G}{G} 2/4 Basilisk. Destroys creatures it fights in
/// combat (modeled as deathtouch — it always deals combat damage).
pub fn thicket_basilisk() -> CardDefinition {
    CardDefinition {
        name: "Thicket Basilisk",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Basilisk], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Cockatrice — {3}{G}{G} 2/4 Cockatrice. Flying; deathtouch (see Thicket
/// Basilisk).
pub fn cockatrice() -> CardDefinition {
    CardDefinition {
        name: "Cockatrice",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cockatrice], ..Default::default() },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Goblin Elite Infantry — {1}{R} 2/2 Goblin Warrior. Whenever it blocks or
/// becomes blocked, it gets -1/-1 until end of turn.
pub fn goblin_elite_infantry() -> CardDefinition {
    let shrink = || Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(-1),
        toughness: Value::Const(-1),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Goblin Elite Infantry",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin, CreatureType::Warrior], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility { event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource), effect: shrink() },
            TriggeredAbility { event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource), effect: shrink() },
        ],
        ..Default::default()
    }
}

/// Alaborn Grenadier — {W}{W} 2/2 Human Soldier. Vigilance.
pub fn alaborn_grenadier() -> CardDefinition {
    CardDefinition {
        name: "Alaborn Grenadier",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Skeletal Snake — {1}{B} 2/1 Snake Skeleton (vanilla).
pub fn skeletal_snake() -> CardDefinition {
    CardDefinition {
        name: "Skeletal Snake",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake, CreatureType::Skeleton], ..Default::default() },
        power: 2,
        toughness: 1,
        ..Default::default()
    }
}

/// Jayemdae Tome — {4} Artifact. {4}, {T}: Draw a card.
pub fn jayemdae_tome() -> CardDefinition {
    CardDefinition {
        name: "Jayemdae Tome",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aladdin's Ring — {8} Artifact. {8}, {T}: Deals 4 damage to any target.
pub fn aladdins_ring() -> CardDefinition {
    CardDefinition {
        name: "Aladdin's Ring",
        cost: cost(&[generic(8)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            tap_cost: true,
            effect: deal(4, target_any()),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// The "mana battery" lifegain cycle (Throne of Bone / Wooden Sphere / Iron
/// Star / Crystal Rod / Ivory Cup): {1} artifact — whenever a player casts a
/// spell of the matching color, you may pay {1}; if you do, gain 1 life.
fn spell_color_lifegain(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::HasColor(color))),
            effect: Effect::MayPay {
                description: "Pay {1}: gain 1 life.".to_string(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
                else_: None,
            },
        }],
        ..Default::default()
    }
}

/// Throne of Bone — casts of black spells.
pub fn throne_of_bone() -> CardDefinition {
    spell_color_lifegain("Throne of Bone", Color::Black)
}

/// Wooden Sphere — casts of green spells.
pub fn wooden_sphere() -> CardDefinition {
    spell_color_lifegain("Wooden Sphere", Color::Green)
}

/// Iron Star — casts of red spells.
pub fn iron_star() -> CardDefinition {
    spell_color_lifegain("Iron Star", Color::Red)
}

/// Crystal Rod — casts of blue spells.
pub fn crystal_rod() -> CardDefinition {
    spell_color_lifegain("Crystal Rod", Color::Blue)
}

/// Ivory Cup — casts of white spells.
pub fn ivory_cup() -> CardDefinition {
    spell_color_lifegain("Ivory Cup", Color::White)
}

/// Wyluli Wolf — {1}{G} 1/1 Wolf. {T}: Target creature gets +1/+1 until end of
/// turn.
pub fn wyluli_wolf() -> CardDefinition {
    CardDefinition {
        name: "Wyluli Wolf",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// D'Avenant Archer — {2}{W} 1/2 Human Soldier Archer. {T}: Deals 1 damage to
/// target attacking or blocking creature.
pub fn davenant_archer() -> CardDefinition {
    CardDefinition {
        name: "D'Avenant Archer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Archer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: deal(1, target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking)))),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bird Maiden — {2}{R} 1/2 Human Bird. Flying.
pub fn bird_maiden() -> CardDefinition {
    CardDefinition {
        name: "Bird Maiden",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Bird], ..Default::default() },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

