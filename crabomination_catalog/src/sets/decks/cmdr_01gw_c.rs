//! Commander: 2026-10-19 (`01GwrDKP`) closers, third batch — Cabal
//! Patriarch's Cabal, Ulrich of the Krallenhorde's werewolves. Tests in
//! `tests/recent_b/cmdr_01gw_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement as R, Subtypes, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{target_filtered, werewolf_day_transform, werewolf_night_transform};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, g, generic, r, Color, ManaCost};
use std::sync::Arc;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CardType>, subtypes: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: types,
        subtypes: Subtypes { creature_types: subtypes, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Aether Theorist — {1}{U} 1/3 Vedalken Rogue. ETB, {E}{E}{E}. {T}, pay
/// Electrostatic Pummeler — {3} 1/1 artifact creature Construct. ETB,
/// Minister of Inquiries — {U} 1/2 Vedalken Advisor. ETB, {E}{E}. {T}, pay
/// Consulate Surveillance — {3}{W} enchantment. ETB, {E}{E}{E}{E}. Pay
/// Cabal Evangel — {1}{B} 2/2 Human Cleric.
pub fn cabal_evangel() -> CardDefinition {
    creature("Cabal Evangel", cost(&[generic(1), b()]), vec![CardType::Creature], vec![CreatureType::Human, CreatureType::Cleric], 2, 2)
}

/// Cabal Therapist — {B} 1/1 menace Horror. At your first main phase, you may
/// sacrifice a creature; when you do, name a nonland card — target player
/// reveals their hand and discards every card with that name.
pub fn cabal_therapist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a creature for Cabal Therapist?".into(),
                filter: R::Creature.and(R::ControlledByYou),
                count: Value::ONE,
                then: Box::new(Effect::Reflexive { body: Box::new(Effect::NameCardTargetDiscardsMatching) }),
                else_: None,
            },
        }],
        ..creature("Cabal Therapist", cost(&[b()]), vec![CardType::Creature], vec![CreatureType::Horror], 1, 1)
    }
}

/// Rite of Belzenlok — {2}{B}{B} Saga. I, II: two 0/1 black Clerics. III: a
/// 6/6 black flying trample Demon that, at your upkeep, has you sacrifice
/// another creature or deals 6 damage to you.
pub fn rite_of_belzenlok() -> CardDefinition {
    let clerics = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: Arc::new(TokenDefinition {
            name: "Cleric".into(),
            power: 0,
            toughness: 1,
            colors: vec![Color::Black],
            card_types: vec![CardType::Creature],
            subtypes: Subtypes { creature_types: vec![CreatureType::Cleric], ..Default::default() },
            ..Default::default()
        }),
    };
    let demon = TokenDefinition {
        name: "Demon".into(),
        power: 6,
        toughness: 6,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::SelectorExists(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                )),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::ONE,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
                else_: Box::new(Effect::DealDamage { to: Selector::You, amount: Value::Const(6) }),
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Rite of Belzenlok",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Saga], ..Default::default() },
        saga_chapters: vec![
            (1, clerics()),
            (2, clerics()),
            (3, Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(demon) }),
        ],
        ..Default::default()
    }
}

/// Curse of the Cabal — {9}{B} sorcery, suspend 2—{2}{B}{B}. Target player
/// sacrifices half the permanents they control, rounded down. At the
/// beginning of each player's upkeep, if it is suspended, that player may
/// sacrifice a permanent to put two time counters on it.
pub fn curse_of_the_cabal() -> CardDefinition {
    CardDefinition {
        name: "Curse of the Cabal",
        cost: cost(&[generic(9), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Suspend(2, cost(&[generic(2), b(), b()]))],
        effect: Effect::SacrificeHalf {
            who: Selector::Player(PlayerRef::Target(0)),
            filter: R::Permanent,
            rounded_up: false,
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer).while_suspended(),
            effect: Effect::MayDoBy {
                who: PlayerRef::ActivePlayer,
                description: "Sacrifice a permanent to delay Curse of the Cabal?".into(),
                body: Box::new(Effect::If {
                    cond: Predicate::SelectorExists(Selector::EachPermanent(R::ControlledByYou)),
                    then: Box::new(Effect::Seq(vec![
                        Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Permanent },
                        Effect::AddCounter {
                            what: Selector::This,
                            kind: crate::card::CounterType::Time,
                            amount: Value::Const(2),
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Daybreak Ranger // Nightfall Predator — {2}{G} 2/2 Human Archer Ranger
/// Werewolf: {T}: 2 damage to target creature with flying. Night (green 4/4
/// Werewolf): {R}, {T}: it fights target creature. Classic werewolf flips.
pub fn daybreak_ranger() -> CardDefinition {
    let predator = CardDefinition {
        name: "Nightfall Predator",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        color_indicator: vec![Color::Green],
        power: 4,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            effect: Effect::Fight { attacker: Selector::This, defender: target_filtered(R::Creature) },
            ..Default::default()
        }],
        triggered_abilities: vec![werewolf_night_transform()],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![werewolf_day_transform()],
        back_face: Some(Box::new(predator)),
        ..creature(
            "Daybreak Ranger",
            cost(&[generic(2), g()]),
            vec![CardType::Creature],
            vec![CreatureType::Human, CreatureType::Archer, CreatureType::Ranger, CreatureType::Werewolf],
            2,
            2,
        )
    }
}

/// Afflicted Deserter // Werewolf Ransacker — {3}{R} 3/2 Human Werewolf. Night
/// (red 5/4 Werewolf): transforming into it, you may destroy target artifact;
/// if it went to a graveyard, 3 damage to its owner.
pub fn afflicted_deserter() -> CardDefinition {
    let ransacker = CardDefinition {
        name: "Werewolf Ransacker",
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Werewolf], ..Default::default() },
        color_indicator: vec![Color::Red],
        power: 5,
        toughness: 4,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: Effect::MayDo {
                    description: "Destroy target artifact?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Destroy { what: target_filtered(R::Artifact) },
                        Effect::If {
                            cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::InGraveyard },
                            then: Box::new(Effect::DealDamage {
                                to: Selector::Player(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                                amount: Value::Const(3),
                            }),
                            else_: Box::new(Effect::Noop),
                        },
                    ])),
                },
            },
            werewolf_night_transform(),
        ],
        ..Default::default()
    };
    CardDefinition {
        triggered_abilities: vec![werewolf_day_transform()],
        back_face: Some(Box::new(ransacker)),
        ..creature(
            "Afflicted Deserter",
            cost(&[generic(3), r()]),
            vec![CardType::Creature],
            vec![CreatureType::Human, CreatureType::Werewolf],
            3,
            2,
        )
    }
}

/// Neglected Heirloom // Ashmouth Blade — {1} Equipment, +1/+1, equip {1}; it
/// transforms when the equipped creature does. Ashmouth Blade: +3/+3 and first
/// strike, equip {3}.
pub fn neglected_heirloom() -> CardDefinition {
    let follows = || TriggeredAbility {
        event: EventSpec::new(EventKind::Transformed, EventScope::AnyPlayer)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsHostOfSource }),
        effect: Effect::Transform { what: Selector::This },
    };
    let blade = CardDefinition {
        name: "Ashmouth Blade",
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        equipped_bonus: Some(EquipBonus { power: 3, toughness: 3, keywords: vec![Keyword::FirstStrike], ..Default::default() }),
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        triggered_abilities: vec![follows()],
        ..Default::default()
    };
    CardDefinition {
        name: "Neglected Heirloom",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 1, ..Default::default() }),
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        triggered_abilities: vec![follows()],
        back_face: Some(Box::new(blade)),
        ..Default::default()
    }
}

/// Ulrich's Kindred — {2}{R} 3/2 trample Wolf. {3}{G}: target attacking Wolf
/// or Werewolf gains indestructible until end of turn.
pub fn ulrichs_kindred() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    R::IsAttacking.and(R::HasCreatureType(CreatureType::Wolf).or(R::HasCreatureType(CreatureType::Werewolf))),
                ),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Ulrich's Kindred", cost(&[generic(2), r()]), vec![CardType::Creature], vec![CreatureType::Wolf], 3, 2)
    }
}
