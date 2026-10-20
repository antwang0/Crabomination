//! Commander: 2026-10-17 (`019Y6J4R`) closers, thirteenth batch — each the
//! one missing card of an otherwise-buildable EDHREC average deck. Tests in
//! `tests/recent_b/cmdr_019y_m.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype,
    Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Selector, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, generic, hybrid, r, u};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn discard_one() -> Effect {
    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }
}

fn creature_cards_in_graveyard(who: PlayerRef) -> Value {
    Value::CountOf(Box::new(Selector::CardsInZone { who, zone: Zone::Graveyard, filter: R::Creature }))
}

/// Metalhead — {4}{U} legendary 4/4 artifact creature Robot Turtle. ETB:
/// return up to one other target artifact or creature to its owner's hand.
/// {R}, sacrifice another artifact: a +1/+1 counter; menace and haste until
/// end of turn.
pub fn metalhead() -> CardDefinition {
    let eot = |k| Effect::GrantKeyword { what: Selector::This, keyword: k, duration: Duration::EndOfTurn };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Artifact.or(R::Creature).and(R::OtherThanSource),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            sac_other_filter: Some((R::Artifact, 1)),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                eot(Keyword::Menace),
                eot(Keyword::Haste),
            ]),
            ..Default::default()
        }],
        ..creature("Metalhead", cost(&[generic(4), u()]), vec![CreatureType::Robot, CreatureType::Turtle], 4, 4)
    }
}

/// U.S.Agent, John Walker — {3}{W/B} legendary 3/2 Human Soldier Hero. ETB:
/// create a colorless Equipment token named Sturdy Shield ("Equipped creature
/// gets +1/+2", equip {2}) attached to it.
pub fn u_s_agent_john_walker() -> CardDefinition {
    let shield = TokenDefinition {
        name: "Sturdy Shield".into(),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus { power: 1, toughness: 2, ..Default::default() }),
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![etb(Effect::CreateTokenAttachedTo {
            target: Selector::This,
            definition: Arc::new(shield),
        })],
        ..creature(
            "U.S.Agent, John Walker",
            cost(&[generic(3), hybrid(Color::White, Color::Black)]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Hero],
            3,
            2,
        )
    }
}

/// Bladed Battle-Fan — {1}{B} flash Equipment. ETB: attach it to target
/// creature you control; it gains indestructible until end of turn. Equipped
/// creature gets +1/+0. Equip {1}.
pub fn bladed_battle_fan() -> CardDefinition {
    CardDefinition {
        name: "Bladed Battle-Fan",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus { power: 1, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Attach { what: Selector::This, to: target_filtered(R::Creature.and(R::ControlledByYou)) },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Indestructible,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

/// Arnim Zola, Bio-Fanatic — {2}{B} legendary 2/3 artifact creature Scientist
/// Villain. {3}, {T}: create a tapped 2/1 black Villain token with menace;
/// only with two or more creature cards in your graveyard.
pub fn arnim_zola_bio_fanatic() -> CardDefinition {
    let villain = TokenDefinition {
        name: "Villain".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Villain], ..Default::default() },
        keywords: vec![Keyword::Menace],
        tapped: true,
        ..Default::default()
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            condition: Some(Predicate::ValueAtLeast(creature_cards_in_graveyard(PlayerRef::You), Value::Const(2))),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(villain) },
            ..Default::default()
        }],
        ..creature(
            "Arnim Zola, Bio-Fanatic",
            cost(&[generic(2), b()]),
            vec![CreatureType::Scientist, CreatureType::Villain],
            2,
            3,
        )
    }
}

/// Liliana, Death Mage — {4}{B}{B} Liliana, loyalty 4. +1: return up to one
/// target creature card from your graveyard to your hand. −3: destroy target
/// creature; its controller loses 2 life. −7: target opponent loses 2 life for
/// each creature card in their graveyard.
pub fn liliana_death_mage() -> CardDefinition {
    CardDefinition {
        name: "Liliana, Death Mage",
        cost: cost(&[generic(4), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Liliana], ..Default::default() },
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility { condition: None,
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::ApplyToTargets {
                    max_targets: 1,
                    min_targets: 0,
                    filter: R::InGraveyard.and(R::OwnedByYou).and(R::Creature),
                    effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
                },
            },
            LoyaltyAbility { condition: None,
                x_cost: false,
                loyalty_cost: -3,
                effect: Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::ControllerOf(Box::new(target_filtered(R::Creature)))),
                        amount: Value::Const(2),
                    },
                    Effect::Destroy { what: Selector::Target(0) },
                ]),
            },
            LoyaltyAbility { condition: None,
                x_cost: false,
                loyalty_cost: -7,
                effect: Effect::LoseLife {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::Times(
                        Box::new(Value::Const(2)),
                        Box::new(creature_cards_in_graveyard(PlayerRef::Target(0))),
                    ),
                },
            },
        ],
        ..Default::default()
    }
}

/// Tragic Lesson — {2}{U} instant. Draw two cards. Then discard a card unless
/// you return a land you control to its owner's hand.
pub fn tragic_lesson() -> CardDefinition {
    spell(
        "Tragic Lesson",
        cost(&[generic(2), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            draw(2),
            Effect::If {
                cond: Predicate::SelectorExists(Selector::ControlledBy { who: PlayerRef::You, filter: R::Land }),
                then: Box::new(Effect::MayDoElse {
                    description: "Return a land you control to its owner's hand instead of discarding?".into(),
                    body: Box::new(Effect::MoveChosen {
                        from: Selector::ControlledBy { who: PlayerRef::You, filter: R::Land },
                        filter: None,
                        count: Value::ONE,
                        up_to: false,
                        to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                    }),
                    else_: Box::new(discard_one()),
                }),
                else_: Box::new(discard_one()),
            },
        ]),
    )
}

/// Assimilate Essence — {1}{U} instant. Counter target creature or battle
/// spell unless its controller pays {4}. If they do, you incubate 2.
pub fn assimilate_essence() -> CardDefinition {
    spell(
        "Assimilate Essence",
        cost(&[generic(1), u()]),
        CardType::Instant,
        Effect::CounterUnlessPaid {
            what: target_filtered(R::IsSpellOnStack.and(R::Creature.or(R::HasCardType(CardType::Battle)))),
            mana_cost: cost(&[generic(4)]),
            exile: false,
            extra_generic: None,
            if_paid: Some(Box::new(Effect::Incubate { who: PlayerRef::You, amount: Value::Const(2) })),
        },
    )
}

/// Spring Cleaning — {1}{G} instant. Destroy target enchantment. Clash with an
/// opponent; if you win, destroy all enchantments your opponents control.
pub fn spring_cleaning() -> CardDefinition {
    spell(
        "Spring Cleaning",
        cost(&[generic(1), crate::mana::g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::Destroy { what: target_filtered(R::Enchantment) },
            Effect::ClashWithOpponent {
                on_win: Box::new(Effect::Destroy {
                    what: Selector::EachPermanent(R::Enchantment.and(R::ControlledByOpponent)),
                }),
                on_lose: None,
                with: None,
            },
        ]),
    )
}

/// Flaring Cinder — {1}{U/R}{U/R} 3/2 Elemental Sorcerer. When it enters and
/// whenever you cast a spell with mana value 4 or greater, you may discard a
/// card; if you do, draw a card.
pub fn flaring_cinder() -> CardDefinition {
    let ur = || hybrid(Color::Blue, Color::Red);
    let rummage = || Effect::If {
        cond: Predicate::ValueAtLeast(Value::HandSizeOf(PlayerRef::You), Value::ONE),
        then: Box::new(Effect::MayDo {
            description: "Discard a card to draw a card?".into(),
            body: Box::new(Effect::Seq(vec![discard_one(), draw(1)])),
        }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(rummage()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::ManaValueAtLeast(4) },
                ),
                effect: rummage(),
            },
        ],
        ..creature(
            "Flaring Cinder",
            cost(&[generic(1), ur(), ur()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            3,
            2,
        )
    }
}

/// Arcane Investigator — {1}{U} 2/1 Elf Wizard. Search the Room — {5}{U}: roll
/// a d20. 1—9: draw a card. 10—20: look at the top three, put one into your
/// hand and the rest on the bottom in any order.
pub fn arcane_investigator() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u()]),
            effect: Effect::RollDie {
                sides: 20,
                count: Value::ONE,
                modifier: Value::ZERO,
                reroll_at_most: 0,
                ignore_lowest: 0,
                on_doubles: None,
                results: vec![
                    (1, 9, draw(1)),
                    (
                        10,
                        20,
                        Effect::LookPickToHand(Box::new(LookPick {
                            who: PlayerRef::You,
                            count: Value::Const(3),
                            ..Default::default()
                        })),
                    ),
                ],
            },
            ..Default::default()
        }],
        ..creature("Arcane Investigator", cost(&[generic(1), u()]), vec![CreatureType::Elf, CreatureType::Wizard], 2, 1)
    }
}

