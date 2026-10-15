//! Commander: two-card gaps of the 2026-10-15 full-list EDHREC scan, third
//! batch. Tests in `tests/recent_b/cmdr_scan7k_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, CumulativeUpkeepCost, DynamicPt,
    EnchantmentSubtype, EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{bolster, etb, on_attack, on_dies, target_filtered};
use crate::effect::{CounteredSpellZone, Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// "Enchant creature. Enchanted creature gets +P/+T and has [keyword]."
fn aura(name: &'static str, mana: ManaCost, p: i32, t: i32, keyword: Keyword, text: [&'static str; 2]) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: [
            (p != 0 || t != 0).then_some(StaticAbility {
                description: text[0],
                effect: StaticEffect::PumpPT { applies_to: enchanted(), power: p, toughness: t },
            }),
            Some(StaticAbility {
                description: text[1],
                effect: StaticEffect::GrantKeyword { applies_to: enchanted(), keyword },
            }),
        ]
        .into_iter()
        .flatten()
        .collect(),
        ..Default::default()
    }
}

const PLUS_TWO_ONE_MENACE: [&str; 2] = ["Enchanted creature gets +2/+1.", "Enchanted creature has menace."];

/// Frenzied Rage — {1}{R} Aura: +2/+1 and menace. (Tahngarth, First Mate.)
pub fn frenzied_rage() -> CardDefinition {
    aura("Frenzied Rage", cost(&[generic(1), r()]), 2, 1, Keyword::Menace, PLUS_TWO_ONE_MENACE)
}

/// Giant Spectacle — {1}{R} Aura: +2/+1 and menace. (Tahngarth, First Mate.)
pub fn giant_spectacle() -> CardDefinition {
    aura("Giant Spectacle", cost(&[generic(1), r()]), 2, 1, Keyword::Menace, PLUS_TWO_ONE_MENACE)
}

/// Invisibility — {U}{U} Aura: can't be blocked except by Walls. (Callaphe.)
pub fn invisibility() -> CardDefinition {
    aura(
        "Invisibility",
        cost(&[u(), u()]),
        0,
        0,
        Keyword::CantBeBlockedExceptBy(Box::new(R::HasCreatureType(CreatureType::Wall))),
        ["", "Enchanted creature can't be blocked except by Walls."],
    )
}

/// One With the Wind — {1}{U} Aura: +2/+2 and flying. (Callaphe.)
pub fn one_with_the_wind() -> CardDefinition {
    aura("One With the Wind", cost(&[generic(1), u()]), 2, 2, Keyword::Flying, ["Enchanted creature gets +2/+2.", "Enchanted creature has flying."])
}

/// Stonebrow, Krosan Hero — {3}{R}{G} legendary 4/4 Centaur Warrior;
/// trample. Whenever a creature you control with trample attacks, it gets
/// +2/+2 until end of turn.
pub fn stonebrow_krosan_hero() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: R::HasKeyword(Keyword::Trample),
            }),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend("Stonebrow, Krosan Hero", cost(&[generic(3), r(), g()]), vec![CreatureType::Centaur, CreatureType::Warrior], 4, 4)
    }
}

/// Khenra Charioteer — {1}{R}{G} 3/3 Jackal Warrior; trample; other
/// creatures you control have trample. (Stonebrow.)
pub fn khenra_charioteer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control have trample.",
            effect: StaticEffect::GrantKeyword { applies_to: yours(R::Creature.and(R::OtherThanSource)), keyword: Keyword::Trample },
        }],
        ..creature("Khenra Charioteer", cost(&[generic(1), r(), g()]), vec![CreatureType::Jackal, CreatureType::Warrior], 3, 3)
    }
}

/// Lurching Rotbeast — {3}{B} 4/2 Zombie Beast; cycling {B}. (Gorex.)
pub fn lurching_rotbeast() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cycling(cost(&[b()]))],
        ..creature("Lurching Rotbeast", cost(&[generic(3), b()]), vec![CreatureType::Zombie, CreatureType::Beast], 4, 2)
    }
}

/// Overwhelming Remorse — {4}{B} instant, {1} less per creature card in
/// your graveyard: exile target creature or planeswalker. (Gorex, Egon.)
pub fn overwhelming_remorse() -> CardDefinition {
    CardDefinition {
        name: "Overwhelming Remorse",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each creature card in your graveyard.",
            effect: StaticEffect::SelfCostReducedPerCreatureInGraveyard,
        }],
        effect: Effect::Exile { what: target_filtered(R::Creature.or(R::Planeswalker)) },
        ..Default::default()
    }
}

/// Dragon Egg — {2}{R} 0/2 Dragon Egg; defender. Dies: a 2/2 red flying
/// Dragon with "{R}: +1/+0 until end of turn." (Verix Bladewing.)
pub fn dragon_egg() -> CardDefinition {
    let dragon = TokenDefinition {
        name: "Dragon".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        keywords: vec![Keyword::Flying],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT { what: Selector::This, power: Value::ONE, toughness: Value::Const(0), duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![on_dies(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(dragon) })],
        ..creature("Dragon Egg", cost(&[generic(2), r()]), vec![CreatureType::Dragon, CreatureType::Egg], 0, 2)
    }
}

/// Burn, Burn, Tree and Fern — {3}{R} Saga (IV). I: 6 damage to target
/// creature an opponent controls. II: destroy target artifact an opponent
/// controls. III, IV: add {R}. (Verix Bladewing.)
pub fn burn_burn_tree_and_fern() -> CardDefinition {
    let red = || Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Red]) };
    CardDefinition {
        name: "Burn, Burn, Tree and Fern",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, Effect::DealDamage { to: target_filtered(R::Creature.and(R::ControlledByOpponent)), amount: Value::Const(6) }),
            (2, Effect::Destroy { what: target_filtered(R::HasCardType(CardType::Artifact).and(R::ControlledByOpponent)) }),
            (3, red()),
            (4, red()),
        ],
        ..Default::default()
    }
}

/// Daghatar the Adamant — {3}{W} legendary 0/0 Human Warrior; vigilance;
/// enters with four +1/+1 counters. {1}{B/G}{B/G}: move a +1/+1 counter from
/// target creature onto a second target creature.
pub fn daghatar_the_adamant() -> CardDefinition {
    use crate::mana::hybrid;
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
            effect: Effect::MoveCounter {
                from: target_filtered(R::Creature),
                to: Selector::TargetFiltered { slot: 1, filter: R::Creature.and(R::OtherThanTargetSlot(0)) },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..legend("Daghatar the Adamant", cost(&[generic(3), w()]), vec![CreatureType::Human, CreatureType::Warrior], 0, 0)
    }
}

/// Map the Wastes — {2}{G} sorcery: a basic land onto the battlefield
/// tapped, then bolster 1. (Daghatar.)
pub fn map_the_wastes() -> CardDefinition {
    CardDefinition {
        name: "Map the Wastes",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            bolster(1),
        ]),
        ..Default::default()
    }
}

/// Wall of Shards — {1}{W} snow 1/8 Wall; defender, flying; cumulative
/// upkeep—an opponent gains 1 life. (Ghalta, the Immovable.)
pub fn wall_of_shards() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Snow],
        keywords: vec![Keyword::Defender, Keyword::Flying, Keyword::CumulativeUpkeep(CumulativeUpkeepCost::OpponentGainsLife(1))],
        ..creature("Wall of Shards", cost(&[generic(1), w()]), vec![CreatureType::Wall], 1, 8)
    }
}

/// Refuse to Yield — {1}{W} instant: target creature gets +2/+7 until end
/// of turn; untap it. (Ghalta, the Immovable.)
pub fn refuse_to_yield() -> CardDefinition {
    CardDefinition {
        name: "Refuse to Yield",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(7),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
        ]),
        ..Default::default()
    }
}

/// Lord of Tresserhorn — {1}{U}{B}{R} legendary 10/4 Zombie. ETB: you lose
/// 2 life, sacrifice two creatures, and target opponent draws two. {B}:
/// regenerate it.
pub fn lord_of_tresserhorn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            Effect::Sacrifice { who: Selector::You, count: Value::Const(2), filter: R::Creature },
            Effect::Draw { who: target_filtered(R::OpponentPlayer), amount: Value::Const(2) },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..legend("Lord of Tresserhorn", cost(&[generic(1), u(), b(), r()]), vec![CreatureType::Zombie], 10, 4)
    }
}

/// Defabricate — {1}{U} instant. Choose one: counter target artifact or
/// enchantment spell, exiling it; or counter target activated or triggered
/// ability. (Lord of Tresserhorn.)
pub fn defabricate() -> CardDefinition {
    CardDefinition {
        name: "Defabricate",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpellToZone {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Artifact).or(R::HasCardType(CardType::Enchantment))),
                ),
                zone: CounteredSpellZone::Exile,
            },
            Effect::CounterAbility { what: target_filtered(R::HasAbilityOnStack) },
        ]),
        ..Default::default()
    }
}

/// Frondland Felidar — {2}{G}{W} 3/5 Cat Beast; vigilance. Creatures you
/// control with vigilance have "{1}, {T}: Tap target creature." (Kaheera.)
pub fn frondland_felidar() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with vigilance have \"{1}, {T}: Tap target creature.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: yours(R::Creature.and(R::HasKeyword(Keyword::Vigilance))),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(1)]),
                    tap_cost: true,
                    effect: Effect::Tap { what: target_filtered(R::Creature) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..creature("Frondland Felidar", cost(&[generic(2), g(), w()]), vec![CreatureType::Cat, CreatureType::Beast], 3, 5)
    }
}

/// Generous Stray — {2}{G} 1/2 Cat; ETB draw a card. (Kaheera.)
pub fn generous_stray() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Draw { who: Selector::You, amount: Value::ONE })],
        ..creature("Generous Stray", cost(&[generic(2), g()]), vec![CreatureType::Cat], 1, 2)
    }
}

/// Old Man Willow — {2}{B}{G} legendary */* Treefolk, P/T the lands you
/// control. Attacking: you may sacrifice another creature or a token; when
/// you do, target creature an opponent controls gets -2/-2.
pub fn old_man_willow() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching { base_p: 0, base_t: 0, filter: Box::new(R::Land) }),
        triggered_abilities: vec![on_attack(Effect::MaySacrifice {
            description: "Sacrifice another creature or a token to shrink an opposing creature?".into(),
            filter: (R::Creature.and(R::OtherThanSource)).or(R::IsToken.and(R::OtherThanSource)),
            count: Value::ONE,
            then: Box::new(Effect::ReflexiveTrigger {
                body: Box::new(Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                }),
            }),
            else_: None,
        })],
        ..legend("Old Man Willow", cost(&[generic(2), b(), g()]), vec![CreatureType::Treefolk], 0, 0)
    }
}

/// Dauntless Dourbark — {3}{G} */* Treefolk Warrior, P/T the Forests plus the
/// Treefolk you control; trample while you control another Treefolk. (Old
/// Man Willow.)
pub fn dauntless_dourbark() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::PermanentsControlledMatching {
            base_p: 0,
            base_t: 0,
            filter: Box::new(R::HasLandType(LandType::Forest).or(R::HasCreatureType(CreatureType::Treefolk))),
        }),
        static_abilities: vec![StaticAbility {
            description: "This creature has trample as long as you control another Treefolk.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Trample,
                condition: Predicate::SelectorCountAtLeast {
                    sel: yours(R::HasCreatureType(CreatureType::Treefolk).and(R::OtherThanSource)),
                    n: Value::ONE,
                },
            },
        }],
        ..creature("Dauntless Dourbark", cost(&[generic(3), g()]), vec![CreatureType::Treefolk, CreatureType::Warrior], 0, 0)
    }
}
