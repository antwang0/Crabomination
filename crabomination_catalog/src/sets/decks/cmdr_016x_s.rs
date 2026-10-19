//! Commander: 2026-10-19 (`016Xh16H`) — Farideh, Devil's Chosen's d20s and
//! Monoxa, Midway Manager's midway (CR 706, 701.51-52). Tests in
//! `tests/recent_b/cmdr_016x_a.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TriggeredAbility,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, KeywordAct, LibraryPosition, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, generic, r, u};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn roll(sides: u8, results: Vec<(u8, u8, Effect)>) -> Effect {
    Effect::RollDie {
        sides,
        count: Value::ONE,
        modifier: Value::Const(0),
        reroll_at_most: 0,
        ignore_lowest: 0,
        results,
        on_doubles: None,
    }
}

fn grant(keyword: Keyword) -> Effect {
    Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn }
}

fn if_rolled_at_least(n: i32, then: Effect) -> Effect {
    Effect::If {
        cond: Predicate::ValueAtLeast(Value::TriggerEventAmount, Value::Const(n)),
        then: Box::new(then),
        else_: Box::new(Effect::Noop),
    }
}

fn treasure() -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(crabomination_base::tokens::treasure_token()) }
}

fn draw_lose(n: i32) -> Effect {
    Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::Const(n) },
        Effect::LoseLife { who: Selector::You, amount: Value::Const(n) },
    ])
}

fn scry(n: i32) -> Effect {
    Effect::Scry { who: PlayerRef::You, amount: Value::Const(n) }
}

// ── Farideh, Devil's Chosen ─────────────────────────────────────────────────

/// Farideh, Devil's Chosen — {2}{U}{R} 3/3 legendary Tiefling Warlock.
/// Whenever you roll one or more dice, flying and menace until end of turn;
/// if a result was 10 or higher, draw a card.
pub fn farideh_devils_chosen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::RolledDice, EventScope::YourControl),
            effect: Effect::Seq(vec![
                grant(Keyword::Flying),
                grant(Keyword::Menace),
                if_rolled_at_least(10, Effect::Draw { who: Selector::You, amount: Value::ONE }),
            ]),
        }],
        ..creature(
            "Farideh, Devil's Chosen",
            cost(&[generic(2), u(), r()]),
            vec![CreatureType::Tiefling, CreatureType::Warlock],
            3,
            3,
        )
    }
}

/// Spiked Pit Trap — {1} flash artifact. {5}, {T}, sacrifice it: target
/// creature; roll a d20: 5 damage to it, and on 10-20 a Treasure too.
pub fn spiked_pit_trap() -> CardDefinition {
    let hit = || Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(5) };
    CardDefinition {
        name: "Spiked Pit Trap",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        keywords: vec![Keyword::Flash],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            sac_cost: true,
            effect: roll(20, vec![(1, 9, hit()), (10, 20, Effect::Seq(vec![hit(), treasure()]))]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Aberrant Mind Sorcerer — {4}{U} 3/4 Human Elf Shaman Sorcerer. ETB target
/// instant or sorcery card in your graveyard; roll a d20: 1-9 you may put it
/// on top of your library, 10-20 return it to your hand.
pub fn aberrant_mind_sorcerer() -> CardDefinition {
    let spell = || target_filtered(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)).from_your_graveyard());
    CardDefinition {
        triggered_abilities: vec![etb(roll(
            20,
            vec![
                (
                    1,
                    9,
                    Effect::MayDo {
                        description: "Put that card on top of your library?".into(),
                        body: Box::new(Effect::Move {
                            what: spell(),
                            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
                        }),
                    },
                ),
                (10, 20, Effect::Move { what: spell(), to: ZoneDest::Hand(PlayerRef::You) }),
            ],
        ))],
        ..creature(
            "Aberrant Mind Sorcerer",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Elf, CreatureType::Shaman, CreatureType::Sorcerer],
            3,
            4,
        )
    }
}

/// Bamboozling Beeble — {U} 1/1 Beeble, protection from Robots. {1}, {T}:
/// the next time target player rolls dice this turn they roll one more and
/// you choose one to ignore.
pub fn bamboozling_beeble() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCreatureType(CreatureType::Robot)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::TargetPlayerThen {
                filter: R::Player,
                then: Box::new(Effect::NextRollExtraDieThisTurn { who: PlayerRef::Target(0) }),
            },
            ..Default::default()
        }],
        ..creature("Bamboozling Beeble", cost(&[u()]), vec![CreatureType::Beeble], 1, 1)
    }
}

/// Djinni Windseer — {3}{U} 3/3 flying Djinn. ETB roll a d20: scry 1 / 2 /
/// 3 on 1-9 / 10-19 / 20.
pub fn djinni_windseer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(roll(20, vec![(1, 9, scry(1)), (10, 19, scry(2)), (20, 20, scry(3))]))],
        ..creature("Djinni Windseer", cost(&[generic(3), u()]), vec![CreatureType::Djinn], 3, 3)
    }
}

/// Scion of Stygia — {2}{U} 2/1 flash Tiefling Shaman. ETB target creature an
/// opponent controls; roll a d20: tap it, and on 10-20 it skips its next
/// untap.
pub fn scion_of_stygia() -> CardDefinition {
    let them = || target_filtered(R::Creature.and(R::ControlledByOpponent));
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(roll(
            20,
            vec![
                (1, 9, Effect::Tap { what: them() }),
                (10, 20, Effect::Seq(vec![Effect::Tap { what: them() }, Effect::SkipNextUntap { what: Selector::Target(0) }])),
            ],
        ))],
        ..creature(
            "Scion of Stygia",
            cost(&[generic(2), u()]),
            vec![CreatureType::Tiefling, CreatureType::Shaman],
            2,
            1,
        )
    }
}

/// Boing! — {1}{U} instant. Return target creature to its owner's hand, then
/// roll a d6; on 3 or less, scry that many.
pub fn boing() -> CardDefinition {
    CardDefinition {
        name: "Boing!",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            roll(6, vec![(1, 3, Effect::Scry { who: PlayerRef::You, amount: Value::LastDieRoll })]),
        ]),
        ..Default::default()
    }
}

// ── Monoxa, Midway Manager ──────────────────────────────────────────────────

/// Monoxa, Midway Manager — {2}{B}{R} 3/3 legendary Vampire Employee.
/// Whenever you roll a 3 or higher: first strike; 4+ also menace; 5+ also
/// lifelink, until end of turn. {6}: roll a d6.
pub fn monoxa_midway_manager() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Performed(KeywordAct::DieResult), EventScope::YourControl)
                .with_filter(Predicate::DieResultAtLeast(3)),
            effect: Effect::Seq(vec![
                grant(Keyword::FirstStrike),
                if_rolled_at_least(4, grant(Keyword::Menace)),
                if_rolled_at_least(5, grant(Keyword::Lifelink)),
            ]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(6)]),
            effect: roll(6, vec![]),
            ..Default::default()
        }],
        ..creature(
            "Monoxa, Midway Manager",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Vampire, CreatureType::Employee],
            3,
            3,
        )
    }
}

/// Discourtesy Clerk — {3}{B} 3/2 Vampire Employee. ETB open an Attraction;
/// at your end step, with three or more Attractions, draw and lose 1 life.
pub fn discourtesy_clerk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::OpenAnAttraction),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::ValueAtLeast(
                        Value::count(Selector::EachPermanent(
                            R::HasArtifactSubtype(crate::card::ArtifactSubtype::Attraction).and(R::ControlledByYou),
                        )),
                        Value::Const(3),
                    ),
                ),
                effect: draw_lose(1),
            },
        ],
        ..creature(
            "Discourtesy Clerk",
            cost(&[generic(3), b()]),
            vec![CreatureType::Vampire, CreatureType::Employee],
            3,
            2,
        )
    }
}

/// Herald of Hadar — {4}{B} 4/4 Human Warlock. {5}{B}: roll a d20 — each
/// opponent loses 2; 10+ you also gain 2; 20 also two Treasures.
pub fn herald_of_hadar() -> CardDefinition {
    let drain = || Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(2) };
    let gain = || Effect::GainLife { who: Selector::You, amount: Value::Const(2) };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), b()]),
            effect: roll(
                20,
                vec![
                    (1, 9, drain()),
                    (10, 19, Effect::Seq(vec![drain(), gain()])),
                    (20, 20, Effect::Seq(vec![drain(), gain(), treasure(), treasure()])),
                ],
            ),
            ..Default::default()
        }],
        ..creature("Herald of Hadar", cost(&[generic(4), b()]), vec![CreatureType::Human, CreatureType::Warlock], 4, 4)
    }
}

/// Line Cutter — {3}{B} 3/4 Human Rogue Guest. ETB roll to visit your
/// Attractions.
pub fn line_cutter() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::RollToVisitAttractions)],
        ..creature(
            "Line Cutter",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Guest],
            3,
            4,
        )
    }
}

/// Nothic — {4}{B} 4/3 Horror. Dying, roll a d20: draw/lose 1, 2, or 7.
pub fn nothic() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: roll(20, vec![(1, 9, draw_lose(1)), (10, 19, draw_lose(2)), (20, 20, draw_lose(7))]),
        }],
        ..creature("Nothic", cost(&[generic(4), b()]), vec![CreatureType::Horror], 4, 3)
    }
}

/// Xenosquirrels — {1}{B} 0/0 Alien Squirrel with two +1/+1 counters. After
/// you roll a die, you may remove a +1/+1 counter to move the result by 1.
pub fn xenosquirrels() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        static_abilities: vec![StaticAbility {
            description: "After you roll a die, you may remove a +1/+1 counter to increase or decrease the result by 1.",
            effect: StaticEffect::RemoveCounterToShiftDie(CounterType::PlusOnePlusOne),
        }],
        ..creature("Xenosquirrels", cost(&[generic(1), b()]), vec![CreatureType::Alien, CreatureType::Squirrel], 0, 0)
    }
}

/// Step Right Up — {3}{B} sorcery. Open two Attractions.
pub fn step_right_up() -> CardDefinition {
    CardDefinition {
        name: "Step Right Up",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![Effect::OpenAnAttraction, Effect::OpenAnAttraction]),
        ..Default::default()
    }
}
