//! Khans of Tarkir block cards.
//!
//! Showcases Dash (CR 702.110, `shortcut::dash`): cast for the dash cost, the
//! creature enters with haste and returns to its owner's hand at the next end
//! step. Plus a few Jeskai prowess/tempo bodies built on existing primitives.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{dash, etb, on_attack, raid_etb, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, Value};
use crate::mana::{b, cost, generic, r, u, w};

/// Screamreach Brawler — {2}{R} 3/3 Orc Berserker. Dash {1}{R}.
pub fn screamreach_brawler() -> CardDefinition {
    CardDefinition {
        name: "Screamreach Brawler",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Mardu Scout — {2}{R} 3/1 Human Warrior. Dash {R}.
pub fn mardu_scout() -> CardDefinition {
    CardDefinition {
        name: "Mardu Scout",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        alternative_cost: Some(dash(cost(&[r()]))),
        ..Default::default()
    }
}

/// Zurgo Bellstriker — {R} 2/2 Legendary Goblin Warrior. Dash {1}{R}.
/// (The "can't block creatures with power 2 or greater" rider collapses —
/// no power-gated block restriction primitive.)
pub fn zurgo_bellstriker() -> CardDefinition {
    CardDefinition {
        name: "Zurgo Bellstriker",
        cost: cost(&[r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Goblin Heelcutter — {3}{R} 3/2 Goblin Berserker. Whenever this attacks,
/// target creature can't block this turn. Dash {1}{R}.
pub fn goblin_heelcutter() -> CardDefinition {
    CardDefinition {
        name: "Goblin Heelcutter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![on_attack(Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature),
            keyword: Keyword::CantBlock,
            duration: Duration::EndOfTurn,
        })],
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Ponyback Brigade — {3}{B}{R} 2/2 Goblin. When this enters, create three
/// 1/1 red Goblin creature tokens. Dash {4}{B}{R}.
pub fn ponyback_brigade() -> CardDefinition {
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Ponyback Brigade",
        cost: cost(&[generic(3), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: goblin,
        })],
        alternative_cost: Some(dash(cost(&[generic(4), b(), r()]))),
        ..Default::default()
    }
}

/// Lightning Berserker — {R} 1/1 Human Berserker. {R}: +1/+0 until end of
/// turn. Dash {R}.
pub fn lightning_berserker() -> CardDefinition {
    CardDefinition {
        name: "Lightning Berserker",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Berserker],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        alternative_cost: Some(dash(cost(&[r()]))),
        ..Default::default()
    }
}

/// Alesha, Who Smiles at Death — {2}{R} 3/2 Legendary Human Warrior with
/// First strike. Dash {1}{R}. (The attack-trigger reanimation of a power-≤2
/// creature is omitted — no targeted graveyard-to-attacking-battlefield
/// reanimate primitive yet.)
pub fn alesha_who_smiles_at_death() -> CardDefinition {
    CardDefinition {
        name: "Alesha, Who Smiles at Death",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Seeker of the Way — {1}{W} 2/2 Human Monk with Prowess. Whenever you cast
/// a noncreature spell, this gains lifelink until end of turn.
pub fn seeker_of_the_way() -> CardDefinition {
    use crate::effect::shortcut::{cast_is_noncreature, prowess};
    CardDefinition {
        name: "Seeker of the Way",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![
            prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(cast_is_noncreature()),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Jeskai Elder — {1}{U} 2/1 Human Monk with Prowess. Whenever this deals
/// combat damage to a player, you may loot (draw a card, then discard one).
pub fn jeskai_elder() -> CardDefinition {
    CardDefinition {
        name: "Jeskai Elder",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Monk],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Prowess],
        triggered_abilities: vec![
            crate::effect::shortcut::prowess(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Bloodsoaked Champion — {R} 1/1 Human Warrior. This can't block. Raid —
/// {1}{B}: Return Bloodsoaked Champion from your graveyard to the battlefield.
/// Activate only if you attacked this turn.
pub fn bloodsoaked_champion() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Predicate, ZoneDest};
    CardDefinition {
        name: "Bloodsoaked Champion",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::CantBlock],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            from_graveyard: true,
            condition: Some(Predicate::PlayerAttackedThisTurn { who: PlayerRef::You }),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Mardu Heart-Piercer — {3}{R} 3/2 Human Warrior. Raid — When this enters,
/// if you attacked this turn, it deals 2 damage to any target.
pub fn mardu_heart_piercer() -> CardDefinition {
    CardDefinition {
        name: "Mardu Heart-Piercer",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        triggered_abilities: vec![raid_etb(Effect::DealDamage {
            to: target_any(),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Every KTK factory, for snapshot name→factory registration.
pub fn all_ktk_card_factories() -> &'static [crate::CardFactory] {
    &[
        screamreach_brawler,
        mardu_scout,
        zurgo_bellstriker,
        goblin_heelcutter,
        ponyback_brigade,
        lightning_berserker,
        alesha_who_smiles_at_death,
        seeker_of_the_way,
        jeskai_elder,
        mardu_heart_piercer,
        bloodsoaked_champion,
    ]
}
