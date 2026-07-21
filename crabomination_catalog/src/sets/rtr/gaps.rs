//! Return to Ravnica (RTR) gap wave 1: a spread of simple commons/uncommons on
//! existing primitives. Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Keyword,
    SelectionRequirement as R, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::shortcut::{etb, on_dies, target_filtered, unleash};
use crate::effect::{Duration, PlayerRef, Selector};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Bellows Lizard — {R} 1/1 Lizard with firebreathing ({1}{R}: +1/+0).
pub fn bellows_lizard() -> CardDefinition {
    CardDefinition {
        name: "Bellows Lizard",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Lizard], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Concordia Pegasus — {1}{W} 1/3 Pegasus with flying.
pub fn concordia_pegasus() -> CardDefinition {
    CardDefinition {
        name: "Concordia Pegasus",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pegasus], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Catacomb Slug — {4}{B} 2/6 Slug (vanilla).
pub fn catacomb_slug() -> CardDefinition {
    CardDefinition {
        name: "Catacomb Slug",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Slug], ..Default::default() },
        power: 2,
        toughness: 6,
        ..Default::default()
    }
}

/// Brushstrider — {1}{G} 3/1 Beast with vigilance.
pub fn brushstrider() -> CardDefinition {
    CardDefinition {
        name: "Brushstrider",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        ..Default::default()
    }
}

/// Daggerdrome Imp — {1}{B} 1/1 Imp with flying and lifelink.
pub fn daggerdrome_imp() -> CardDefinition {
    CardDefinition {
        name: "Daggerdrome Imp",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Imp], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Centaur Healer — {1}{G}{W} 3/3 Centaur Cleric. When it enters, you gain 3 life.
pub fn centaur_healer() -> CardDefinition {
    CardDefinition {
        name: "Centaur Healer",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(3),
        })],
        ..Default::default()
    }
}

/// Batterhorn — {4}{R} 4/3 Beast. When it enters, you may destroy target artifact.
pub fn batterhorn() -> CardDefinition {
    CardDefinition {
        name: "Batterhorn",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Destroy target artifact".into(),
            body: Box::new(Effect::Destroy { what: target_filtered(R::Artifact) }),
        })],
        ..Default::default()
    }
}

/// Crosstown Courier — {1}{U} 2/1 Vedalken. Whenever it deals combat damage to
/// a player, that player mills that many cards.
pub fn crosstown_courier() -> CardDefinition {
    CardDefinition {
        name: "Crosstown Courier",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vedalken], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Aquus Steed — {3}{U} 1/3 Beast. {2}{U}, {T}: target creature gets -2/-0
/// until end of turn.
pub fn aquus_steed() -> CardDefinition {
    CardDefinition {
        name: "Aquus Steed",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 1,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// 3/3 green Centaur creature token.
fn centaur_token() -> TokenDefinition {
    TokenDefinition {
        name: "Centaur".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Centaur], ..Default::default() },
        ..Default::default()
    }
}

/// Centaur's Herald — {G} 0/1 Elf Scout. {2}{G}, Sacrifice this: create a 3/3
/// green Centaur creature token.
pub fn centaurs_herald() -> CardDefinition {
    CardDefinition {
        name: "Centaur's Herald",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), g()]),
            sac_cost: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: centaur_token(),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Drainpipe Vermin — {B} 1/1 Rat. When it dies, you may pay {B}; if you do,
/// target player discards a card.
pub fn drainpipe_vermin() -> CardDefinition {
    CardDefinition {
        name: "Drainpipe Vermin",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::MayPay {
            description: "Pay {B}: target player discards a card".into(),
            mana_cost: cost(&[b()]),
            body: Box::new(Effect::Discard { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE, random: false }),
            else_: None,
        })],
        ..Default::default()
    }
}

/// Dead Reveler — {2}{B} 2/3 Zombie with unleash.
pub fn dead_reveler() -> CardDefinition {
    CardDefinition {
        name: "Dead Reveler",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Unleash],
        triggered_abilities: vec![unleash()],
        ..Default::default()
    }
}

/// Doorkeeper — {1}{U} 0/4 Homunculus with defender. {2}{U}, {T}: target player
/// mills X, where X is the number of creatures you control with defender.
pub fn doorkeeper() -> CardDefinition {
    CardDefinition {
        name: "Doorkeeper",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Homunculus], ..Default::default() },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            tap_cost: true,
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::count(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::HasKeyword(Keyword::Defender)),
                )),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
