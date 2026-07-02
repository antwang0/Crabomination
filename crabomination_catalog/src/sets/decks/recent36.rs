//! Ramp / tokens / graveyard-fill commons plus a pair of punisher enchantments.
//! Wayward Swordtooth exercises the new `Keyword::CantAttackOrBlockUnlessCity-
//! Blessing` (CR 702.131). Tests in `tests/recent36.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope, EventSpec, Keyword,
    LandType, SelectionRequirement, Selector, StaticAbility, StaticEffect, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::mint_treasures;
use crate::effect::{PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

fn red_goblin_token() -> TokenDefinition {
    TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        ..Default::default()
    }
}

/// Hour of Promise — {4}{G} Sorcery. Search your library for up to two land
/// cards, put them onto the battlefield tapped; then if you control three or
/// more Deserts, create two 2/2 black Zombie tokens.
pub fn hour_of_promise() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    let deserts = Value::CountOf(Box::new(Selector::ControlledBy {
        who: PlayerRef::You,
        filter: SelectionRequirement::HasLandType(LandType::Desert),
    }));
    CardDefinition {
        name: "Hour of Promise",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(deserts, Value::Const(3)),
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: zombie,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Pir's Whim — {3}{G} Sorcery. You search your library for a land and put it
/// onto the battlefield tapped; each opponent sacrifices an artifact or
/// enchantment. (The full friend/foe vote is approximated as you=friend,
/// opponents=foe.)
pub fn pirs_whim() -> CardDefinition {
    CardDefinition {
        name: "Pir's Whim",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
            },
        ]),
        ..Default::default()
    }
}

/// Wayward Swordtooth — {2}{G} 5/5 Dinosaur. Ascend; you may play an additional
/// land each turn; can't attack or block unless you have the city's blessing.
pub fn wayward_swordtooth() -> CardDefinition {
    CardDefinition {
        name: "Wayward Swordtooth",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::CantAttackOrBlockUnlessCityBlessing],
        static_abilities: vec![StaticAbility {
            description: "You may play an additional land on each of your turns.",
            effect: StaticEffect::ExtraLandPerTurn,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Ascend { who: PlayerRef::You },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Ascend { who: PlayerRef::You },
            },
        ],
        ..Default::default()
    }
}

/// Gather the Pack — {1}{G} Sorcery. Reveal the top five cards of your library,
/// put a creature card among them into your hand, and the rest into your
/// graveyard. (Spell mastery's second creature is dropped.)
pub fn gather_the_pack() -> CardDefinition {
    CardDefinition {
        name: "Gather the Pack",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::MillThenToHand { amount: Value::Const(5), filter: SelectionRequirement::Creature },
        ..Default::default()
    }
}

/// Tracker's Instincts — {1}{G} Sorcery with flashback {2}{U}. Reveal the top
/// four cards of your library, put a creature card among them into your hand,
/// and the rest into your graveyard.
pub fn trackers_instincts() -> CardDefinition {
    CardDefinition {
        name: "Tracker's Instincts",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), u()]))],
        effect: Effect::MillThenToHand { amount: Value::Const(4), filter: SelectionRequirement::Creature },
        ..Default::default()
    }
}

/// Dictate of Kruphix — {1}{U}{U} Enchantment with flash. At the beginning of
/// each player's draw step, that player draws an additional card.
pub fn dictate_of_kruphix() -> CardDefinition {
    CardDefinition {
        name: "Dictate of Kruphix",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Draw),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Mogg Flunkies — {1}{R} 3/3 Goblin. Can't attack or block alone.
pub fn mogg_flunkies() -> CardDefinition {
    CardDefinition {
        name: "Mogg Flunkies",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::CantAttackOrBlockAlone],
        ..Default::default()
    }
}

/// Wily Goblin — {R}{R} 1/1 Goblin Pirate. ETB: create a Treasure.
pub fn wily_goblin() -> CardDefinition {
    CardDefinition {
        name: "Wily Goblin",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: mint_treasures(1),
        }],
        ..Default::default()
    }
}

/// Hunted Witness — {W} 1/1 Human. When it dies, create a 1/1 white Soldier
/// with lifelink.
pub fn hunted_witness() -> CardDefinition {
    let soldier = TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        keywords: vec![Keyword::Lifelink],
        ..Default::default()
    };
    CardDefinition {
        name: "Hunted Witness",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
        }],
        ..Default::default()
    }
}

/// Brindle Shoat — {1}{G} 1/1 Boar. When it dies, create a 3/3 green Boar.
pub fn brindle_shoat() -> CardDefinition {
    let boar = TokenDefinition {
        name: "Boar".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Brindle Shoat",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Boar], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: boar },
        }],
        ..Default::default()
    }
}

/// Goblin Assault — {2}{R} Enchantment. At the beginning of your upkeep, create
/// a 1/1 red Goblin with haste. Goblin creatures attack each combat if able.
pub fn goblin_assault() -> CardDefinition {
    let mut hasty = red_goblin_token();
    hasty.keywords = vec![Keyword::Haste];
    CardDefinition {
        name: "Goblin Assault",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::ActivePlayer,
            ),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: hasty },
        }],
        static_abilities: vec![StaticAbility {
            description: "Goblin creatures attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(SelectionRequirement::HasCreatureType(
                    CreatureType::Goblin,
                )),
                keyword: Keyword::MustAttack,
            },
        }],
        ..Default::default()
    }
}

/// Goblin Rally — {3}{R}{R} Sorcery. Create four 1/1 red Goblins.
pub fn goblin_rally() -> CardDefinition {
    CardDefinition {
        name: "Goblin Rally",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(4),
            definition: red_goblin_token(),
        },
        ..Default::default()
    }
}

/// Bottomless Pit — {1}{B}{B} Enchantment. At the beginning of each player's
/// upkeep, that player discards a card at random.
pub fn bottomless_pit() -> CardDefinition {
    CardDefinition {
        name: "Bottomless Pit",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(1),
                random: true,
            },
        }],
        ..Default::default()
    }
}
