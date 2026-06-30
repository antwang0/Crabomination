//! A twenty-first staples wave — simple OTJ / recent-set cards riding existing
//! primitives (Convoke, Equipment, attack/death tokens, surveil, mill-to-hand,
//! a `CommittedCrime` untap). Tests in `crabomination/src/tests/recent21.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector, Subtypes,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::{Color, cost, g, generic, r, u, w};

/// 1/1 red Mercenary with a sorcery-speed tap pump (Wanted Griffin).
fn mercenary_token() -> TokenDefinition {
    TokenDefinition {
        name: "Mercenary".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Mercenary], ..Default::default() },
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::PumpPT {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Skyknight Vanguard — {R}{W} 1/2 Human Knight. Flying. When it attacks, make a
/// 1/1 white Soldier tapped and attacking.
pub fn skyknight_vanguard() -> CardDefinition {
    CardDefinition {
        name: "Skyknight Vanguard",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Soldier".into(),
                power: 1,
                toughness: 1,
                card_types: vec![CardType::Creature],
                colors: vec![Color::White],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Soldier],
                    ..Default::default()
                },
                ..Default::default()
            },
            cleanup: Default::default(),
        })],
        ..Default::default()
    }
}

/// Aerial Boost — {1}{W} Instant with Convoke. Target creature gets +2/+2 and
/// gains flying until end of turn.
pub fn aerial_boost() -> CardDefinition {
    CardDefinition {
        name: "Aerial Boost",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Convoke],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Boots of Speed — {R} Equipment. Equipped creature gets +1/+0 and has haste.
/// Equip {1}.
pub fn boots_of_speed() -> CardDefinition {
    CardDefinition {
        name: "Boots of Speed",
        cost: cost(&[r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(1)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Haste],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Ankle Biter — {G} 1/1 Snake with deathtouch.
pub fn ankle_biter() -> CardDefinition {
    CardDefinition {
        name: "Ankle Biter",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Trick Shot — {4}{R} Instant. Deal 6 damage to target creature. (The extra
/// "2 to another target creature token" rider is approximated away.)
pub fn trick_shot() -> CardDefinition {
    CardDefinition {
        name: "Trick Shot",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(6),
        },
        ..Default::default()
    }
}

/// Patient Naturalist — {2}{G} 2/3 Human Scout. ETB mill three; put a land from
/// among them into your hand. (The "else make a Treasure" rider is dropped.)
pub fn patient_naturalist() -> CardDefinition {
    CardDefinition {
        name: "Patient Naturalist",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![etb(Effect::MillThenToHand {
            amount: Value::Const(3),
            filter: SelectionRequirement::Land,
        })],
        ..Default::default()
    }
}

/// Plan the Heist — {2}{U}{U} Sorcery. Surveil 3 if you have no cards in hand,
/// then draw three.
pub fn plan_the_heist() -> CardDefinition {
    CardDefinition {
        name: "Plan the Heist",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::If {
                cond: Predicate::ValueAtMost(Value::HandSizeOf(PlayerRef::You), Value::Const(0)),
                then: Box::new(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
        ]),
        ..Default::default()
    }
}

/// Wanted Griffin — {3}{W} 3/2 Griffin with flying. When it dies, make a 1/1 red
/// Mercenary.
pub fn wanted_griffin() -> CardDefinition {
    CardDefinition {
        name: "Wanted Griffin",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: mercenary_token(),
        })],
        ..Default::default()
    }
}

/// Sterling Hound — {3} 3/2 Artifact Dog. ETB surveil 2.
pub fn sterling_hound() -> CardDefinition {
    CardDefinition {
        name: "Sterling Hound",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dog], ..Default::default() },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Surveil {
            who: PlayerRef::You,
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Hardbristle Bandit — {1}{G} 1/1 Plant Rogue. {T}: add one mana of any color.
/// Whenever you commit a crime, untap it (once each turn).
pub fn hardbristle_bandit() -> CardDefinition {
    CardDefinition {
        name: "Hardbristle Bandit",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CommittedCrime, EventScope::YourControl).once_per_turn(),
            effect: Effect::Untap { what: Selector::This, up_to: None },
        }],
        ..Default::default()
    }
}

/// Rumbling Rockslide — {3}{R} Sorcery. Deal damage equal to the number of lands
/// you control to target creature.
pub fn rumbling_rockslide() -> CardDefinition {
    CardDefinition {
        name: "Rumbling Rockslide",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(SelectionRequirement::Any)),
                filter: SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
            },
        },
        ..Default::default()
    }
}
