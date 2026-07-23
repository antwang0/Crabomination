//! Dragon's Maze (DGM) spells, enchantments, and Auras. Tests in
//! `classic_sets/dgm`.

use crate::card::{
    AlternativeCost, CardDefinition, CardType, EnchantmentSubtype, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, PlayerRef, Selector, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn aura(name: &'static str, mc: crate::mana::ManaCost, enchant: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mc,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: Selector::TargetFiltered { slot: 0, filter: enchant } },
        ..Default::default()
    }
}

/// Phytoburst — {1}{G} Sorcery. Target creature gets +5/+5 until end of turn.
pub fn phytoburst() -> CardDefinition {
    CardDefinition {
        name: "Phytoburst",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(5),
            toughness: Value::Const(5),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Weapon Surge — {R} Instant. Target creature you control gets +1/+0 and gains
/// first strike until end of turn. Overload {1}{R}.
pub fn weapon_surge() -> CardDefinition {
    let body = |what: Selector| {
        Effect::Seq(vec![
            Effect::PumpPT { what: what.clone(), power: Value::Const(1), toughness: Value::Const(0), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what, keyword: Keyword::FirstStrike, duration: Duration::EndOfTurn },
        ])
    };
    CardDefinition {
        name: "Weapon Surge",
        cost: cost(&[r()]),
        card_types: vec![CardType::Instant],
        effect: body(target_filtered(R::Creature.and(R::ControlledByYou))),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), r()]),
            effect_override: Some(body(Selector::EachPermanent(R::Creature.and(R::ControlledByYou)))),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Clear a Path — {R} Sorcery. Destroy target creature with defender.
pub fn clear_a_path() -> CardDefinition {
    CardDefinition {
        name: "Clear a Path",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Defender))) },
        ..Default::default()
    }
}

/// Mending Touch — {G} Instant. Regenerate target creature.
pub fn mending_touch() -> CardDefinition {
    CardDefinition {
        name: "Mending Touch",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Regenerate { what: target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// Wake the Reflections — {W} Sorcery. Populate.
pub fn wake_the_reflections() -> CardDefinition {
    CardDefinition {
        name: "Wake the Reflections",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Populate { who: PlayerRef::You },
        ..Default::default()
    }
}

/// Awe for the Guilds — {2}{R} Sorcery. Monocolored creatures can't block this
/// turn.
pub fn awe_for_the_guilds() -> CardDefinition {
    CardDefinition {
        name: "Awe for the Guilds",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GrantKeyword {
            what: Selector::EachPermanent(R::Creature.and(R::Monocolored)),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Riot Control — {2}{W} Instant. Gain 1 life for each creature your opponents
/// control; prevent all damage that would be dealt to you this turn.
pub fn riot_control() -> CardDefinition {
    CardDefinition {
        name: "Riot Control",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)))),
            },
            Effect::PreventAllDamageThisTurn { target: Selector::Player(PlayerRef::You) },
        ]),
        ..Default::default()
    }
}

/// Punish the Enemy — {4}{R} Instant. Deal 3 damage to target player or
/// planeswalker and 3 damage to target creature.
pub fn punish_the_enemy() -> CardDefinition {
    CardDefinition {
        name: "Punish the Enemy",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 0, filter: R::Player.or(R::Planeswalker) },
                amount: Value::Const(3),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                amount: Value::Const(3),
            },
        ]),
        ..Default::default()
    }
}

/// Lyev Decree — {1}{W} Sorcery. Detain up to two target creatures your
/// opponents control.
pub fn lyev_decree() -> CardDefinition {
    CardDefinition {
        name: "Lyev Decree",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByOpponent),
            effect: Box::new(Effect::Detain { what: Selector::Target(0) }),
        },
        ..Default::default()
    }
}

/// Restore the Peace — {1}{W}{U} Instant. Return each creature that dealt damage
/// this turn to its owner's hand.
pub fn restore_the_peace() -> CardDefinition {
    CardDefinition {
        name: "Restore the Peace",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::EachPermanent(R::Creature.and(R::DealtDamageThisTurn)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
        ..Default::default()
    }
}

/// Fatal Fumes — {3}{B} Instant. Target creature gets -4/-2 until end of turn.
pub fn fatal_fumes() -> CardDefinition {
    CardDefinition {
        name: "Fatal Fumes",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Mindstatic — {3}{U} Instant. Counter target spell unless its controller pays
/// {6}.
pub fn mindstatic() -> CardDefinition {
    CardDefinition {
        name: "Mindstatic",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack),
            mana_cost: cost(&[generic(6)]),
            exile: false,
            extra_generic: None,
        },
        ..Default::default()
    }
}

/// Uncovered Clues — {2}{U} Sorcery. Look at the top four cards of your library;
/// put up to two instant/sorcery cards from among them into your hand, rest on
/// the bottom.
pub fn uncovered_clues() -> CardDefinition {
    CardDefinition {
        name: "Uncovered Clues",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LookPickToHand {
            who: PlayerRef::You,
            count: Value::Const(4),
            pick_filter: Some(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            take: Some(Value::Const(2)),
            optional: true,
            rest_to_graveyard: false,
            to_battlefield: false,
            gain_life_if_pick: None,
            gain_life_greatest_power_rest: false,
            picked_lands_to_battlefield: false,
            rest_bottom_random: false,
        },
        ..Default::default()
    }
}

/// Warped Physique — {U}{B} Instant. Target creature gets +X/-X until end of
/// turn, where X is the number of cards in your hand.
pub fn warped_physique() -> CardDefinition {
    let x = Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Any };
    CardDefinition {
        name: "Warped Physique",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: x.clone(),
            toughness: Value::Diff(Box::new(Value::Const(0)), Box::new(x)),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Morgue Burst — {4}{B}{R} Sorcery. Return target creature card from your
/// graveyard to your hand; deal damage to any target equal to its power.
pub fn morgue_burst() -> CardDefinition {
    CardDefinition {
        name: "Morgue Burst",
        cost: cost(&[generic(4), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.or(R::Player).or(R::Planeswalker) },
                amount: Value::PowerOf(Box::new(Selector::Target(0))),
            },
        ]),
        ..Default::default()
    }
}

/// Gruul War Chant — {2}{R}{G} Enchantment. Attacking creatures you control get
/// +1/+0 and have menace.
pub fn gruul_war_chant() -> CardDefinition {
    CardDefinition {
        name: "Gruul War Chant",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control get +1/+0 and have menace.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou).and(R::IsAttacking),
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Menace],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Bred for the Hunt — {1}{G}{U} Enchantment. Whenever a creature you control
/// with a +1/+1 counter on it deals combat damage to a player, you may draw a
/// card.
pub fn bred_for_the_hunt() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Bred for the Hunt",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl)
                .with_filter(crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::WithCounter(CounterType::PlusOnePlusOne),
                }),
            effect: Effect::MayDo {
                description: "draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}

/// Sinister Possession — {B} Aura. Enchant creature. Whenever enchanted creature
/// attacks or blocks, its controller loses 2 life.
pub fn sinister_possession() -> CardDefinition {
    use crate::card::EquipBonus;
    // Granted to the host, so `Attacks`/`Blocks` on SelfSource read the enchanted
    // creature and `Selector::You` resolves to its controller (as in Nettling Curse).
    let lose2 = |kind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::SelfSource),
        effect: Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
    };
    let mut c = aura("Sinister Possession", cost(&[b()]), R::Creature);
    c.equipped_bonus = Some(EquipBonus {
        triggered_abilities: vec![lose2(EventKind::Attacks), lose2(EventKind::Blocks)],
        ..Default::default()
    });
    c
}

/// Runner's Bane — {1}{U} Aura. Enchant creature with power 3 or less. ETB tap
/// it; it doesn't untap during its controller's untap step.
pub fn runners_bane() -> CardDefinition {
    let mut c = aura("Runner's Bane", cost(&[generic(1), u()]), R::Creature.and(R::PowerAtMost(3)));
    c.triggered_abilities = vec![etb(Effect::Tap { what: Selector::AttachedTo(Box::new(Selector::This)) })];
    c.static_abilities = vec![StaticAbility {
        description: "Enchanted creature doesn't untap during its controller's untap step.",
        effect: StaticEffect::PreventUntap { applies_to: Selector::AttachedTo(Box::new(Selector::This)) },
    }];
    c
}

/// Advent of the Wurm — {1}{G}{G}{W} Instant. Create a 5/5 green Wurm with
/// trample.
pub fn advent_of_the_wurm() -> CardDefinition {
    use crate::card::{CreatureType, TokenDefinition};
    let wurm = TokenDefinition {
        name: "Wurm".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        keywords: vec![Keyword::Trample],
        ..Default::default()
    };
    CardDefinition {
        name: "Advent of the Wurm",
        cost: cost(&[generic(1), g(), g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: wurm },
        ..Default::default()
    }
}

/// Renounce the Guilds — {1}{W} Instant. Each player sacrifices a multicolored
/// permanent of their choice.
pub fn renounce_the_guilds() -> CardDefinition {
    CardDefinition {
        name: "Renounce the Guilds",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Multicolored,
        },
        ..Default::default()
    }
}
