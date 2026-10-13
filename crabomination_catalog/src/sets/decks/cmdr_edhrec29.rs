//! Commander: two-card gaps for Ojer Pakpatiq, Bruna, Maralen of the
//! Mornsong and Gilgamesh (`cmdr_edhrec28` is the previous batch). Tests in
//! `tests/recent_b/cmdr_edhrec29.rs`.

use crate::card::{
    ActivatedAbility, Adventure, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, MayPlayDuration, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{exalted, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, PlayerStaticTarget, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, w, Color, ManaCost};
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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

/// "Whenever a creature you control attacks alone" — the attacker is the
/// trigger source.
fn attacks_alone(filter: R, effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(Predicate::EntityMatches {
            what: Selector::TriggerSource,
            filter: filter.and(R::IsAttackingAlone),
        }),
        effect,
    }
}

/// Maralen and Mornsong Aria's draw-step tutor: the active player loses 3
/// life and searches for a card.
fn draw_step_tutor() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::AnyPlayer),
        effect: Effect::EachPlayerDoes {
            who: PlayerRef::ActivePlayer,
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Search { who: PlayerRef::You, filter: R::Any, to: ZoneDest::Hand(PlayerRef::You) },
            ])),
        },
    }
}

fn no_draws() -> StaticAbility {
    StaticAbility { description: "Players can't draw cards.", effect: StaticEffect::PlayersSkipDraws }
}

// ── Ojer Pakpatiq, Deepest Epoch ─────────────────────────────────────────────

/// Sword Coast Serpent // Capsizing Wave — {5}{U}{U} 6/6 Serpent Dragon,
/// unblockable once you've cast a noncreature spell this turn. Adventure
/// {1}{U} instant: return target creature to its owner's hand.
pub fn sword_coast_serpent() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature can't be blocked as long as you've cast a noncreature spell this turn.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Unblockable,
                condition: Predicate::NoncreatureSpellsCastThisTurnAtLeast { who: PlayerRef::You, at_least: Value::ONE },
            },
        }],
        adventure: Some(Box::new(Adventure {
            name: "Capsizing Wave",
            cost: cost(&[generic(1), u()]),
            card_types: vec![CardType::Instant],
            effect: Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        })),
        ..creature("Sword Coast Serpent", cost(&[generic(5), u(), u()]), vec![CreatureType::Serpent, CreatureType::Dragon], 6, 6)
    }
}

/// Supreme Will — {2}{U} instant. Choose one: counter target spell unless its
/// controller pays {3}; or look at the top four and take one, the rest on the
/// bottom.
pub fn supreme_will() -> CardDefinition {
    CardDefinition {
        name: "Supreme Will",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(4),
                ..Default::default()
            })),
        ]),
        ..Default::default()
    }
}

// ── Bruna, Light of Alabaster ────────────────────────────────────────────────

/// Sovereigns of Lost Alara — {4}{W}{U} 4/5 Spirit, exalted. A creature of
/// yours attacking alone may fetch an Aura from your library onto it.
pub fn sovereigns_of_lost_alara() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            exalted(),
            attacks_alone(
                R::Creature,
                Effect::MayDo {
                    description: "Search for an Aura to attach to the lone attacker?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Search {
                            who: PlayerRef::You,
                            filter: R::Enchantment.and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
                            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                        },
                        Effect::Attach { what: Selector::LastMoved, to: Selector::TriggerSource },
                    ])),
                },
            ),
        ],
        ..creature("Sovereigns of Lost Alara", cost(&[generic(4), w(), u()]), vec![CreatureType::Spirit], 4, 5)
    }
}

/// Curator's Ward — {2}{U} Aura (enchant permanent). Hexproof; when the
/// enchanted permanent leaves the battlefield, if it was historic, draw two.
pub fn curators_ward() -> CardDefinition {
    CardDefinition {
        name: "Curator's Ward",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Permanent) },
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Hexproof], ..Default::default() }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::EnchantedBySource)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::historic() }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

// ── Maralen of the Mornsong ──────────────────────────────────────────────────

/// Maralen of the Mornsong — {1}{B}{B} 2/3 legendary Elf Wizard. Players
/// can't draw cards. Each draw step, the active player loses 3 life and
/// tutors a card.
pub fn maralen_of_the_mornsong() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![no_draws()],
        triggered_abilities: vec![draw_step_tutor()],
        ..legend("Maralen of the Mornsong", cost(&[generic(1), b(), b()]), vec![CreatureType::Elf, CreatureType::Wizard], 2, 3)
    }
}

/// Mornsong Aria — {1}{B}{B} legendary enchantment. Players can't draw cards
/// or gain life; each draw step, the active player loses 3 life and tutors.
pub fn mornsong_aria() -> CardDefinition {
    CardDefinition {
        name: "Mornsong Aria",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            no_draws(),
            StaticAbility {
                description: "Players can't gain life.",
                effect: StaticEffect::PlayerCannotGainLife { target: PlayerStaticTarget::EachPlayer },
            },
        ],
        triggered_abilities: vec![draw_step_tutor()],
        ..Default::default()
    }
}

// ── Gilgamesh, Master-at-Arms ────────────────────────────────────────────────

/// Goro-Goro, Disciple of Ryusei — {1}{R} 2/2 legendary Goblin Samurai. {R}:
/// your creatures gain haste. {3}{R}{R}: a 5/5 red flying Dragon Spirit, only
/// while you control an attacking modified creature.
pub fn goro_goro_disciple_of_ryusei() -> CardDefinition {
    let dragon = Arc::new(TokenDefinition {
        name: "Dragon Spirit".into(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon, CreatureType::Spirit], ..Default::default() },
        ..Default::default()
    });
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::GrantKeyword {
                    what: yours(R::Creature),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(3), r(), r()]),
                condition: Some(Predicate::SelectorExists(yours(R::Creature.and(R::IsAttacking).and(R::IsModified)))),
                effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: dragon },
                ..Default::default()
            },
        ],
        ..legend("Goro-Goro, Disciple of Ryusei", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Samurai], 2, 2)
    }
}

/// Heiko Yamazaki, the General — {3}{R} 3/3 legendary Human Samurai,
/// trample. A Samurai or Warrior of yours attacking alone lets you cast
/// target artifact card from your graveyard this turn.
pub fn heiko_yamazaki_the_general() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![attacks_alone(
            R::HasCreatureType(CreatureType::Samurai).or(R::HasCreatureType(CreatureType::Warrior)),
            Effect::GrantMayPlay {
                what: target_filtered(R::Artifact.from_your_graveyard()),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        )],
        ..legend("Heiko Yamazaki, the General", cost(&[generic(3), r()]), vec![CreatureType::Human, CreatureType::Samurai], 3, 3)
    }
}
