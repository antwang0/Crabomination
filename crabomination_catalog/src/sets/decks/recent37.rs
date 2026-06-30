//! The Enchantress card-draw cycle, three black board wipes, and a couple of
//! white/equipment value pieces. Tests in `tests/recent37.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, Effect, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, LandType, SelectionRequirement, Selector, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::draw;
use crate::effect::{Duration, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, w, Color};

/// "Whenever you cast an enchantment spell, draw a card."
fn enchantress_draw_trigger() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(Predicate::CastSpellMatches(SelectionRequirement::Enchantment)),
        effect: draw(1),
    }
}

/// Mesa Enchantress — {1}{W}{W} 0/2 Human Druid. Whenever you cast an
/// enchantment spell, draw a card.
pub fn mesa_enchantress() -> CardDefinition {
    CardDefinition {
        name: "Mesa Enchantress",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![enchantress_draw_trigger()],
        ..Default::default()
    }
}

/// Verduran Enchantress — {1}{G}{G} 0/2 Human Druid. Whenever you cast an
/// enchantment spell, draw a card.
pub fn verduran_enchantress() -> CardDefinition {
    CardDefinition {
        name: "Verduran Enchantress",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        triggered_abilities: vec![enchantress_draw_trigger()],
        ..Default::default()
    }
}

/// Femeref Enchantress — {G}{W} 1/2 Human Druid. Whenever an enchantment is put
/// into a graveyard from the battlefield, draw a card.
pub fn femeref_enchantress() -> CardDefinition {
    CardDefinition {
        name: "Femeref Enchantress",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                },
            ),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// Eidolon of Blossoms — {2}{G}{G} 2/2 Enchantment Creature — Spirit.
/// Constellation — whenever this or another enchantment you control enters,
/// draw a card.
pub fn eidolon_of_blossoms() -> CardDefinition {
    CardDefinition {
        name: "Eidolon of Blossoms",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment,
                },
            ),
            effect: draw(1),
        }],
        ..Default::default()
    }
}

/// All creatures get `pt`/`pt` until end of turn (a fixed or dynamic board wipe).
fn wipe(power: Value, toughness: Value) -> Effect {
    Effect::PumpPT {
        what: Selector::EachPermanent(SelectionRequirement::Creature),
        power,
        toughness,
        duration: Duration::EndOfTurn,
    }
}

/// Mutilate — {2}{B}{B} Sorcery. All creatures get -1/-1 until end of turn for
/// each Swamp you control.
pub fn mutilate() -> CardDefinition {
    let swamps = Value::CountOf(Box::new(Selector::ControlledBy {
        who: PlayerRef::You,
        filter: SelectionRequirement::HasLandType(LandType::Swamp),
    }));
    let minus = Value::Times(Box::new(Value::Const(-1)), Box::new(swamps));
    CardDefinition {
        name: "Mutilate",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: wipe(minus.clone(), minus),
        ..Default::default()
    }
}

/// Golden Demise — {1}{B}{B} Sorcery. All creatures get -2/-2 until end of turn.
/// (The city's-blessing "opponents only" rider is dropped.)
pub fn golden_demise() -> CardDefinition {
    CardDefinition {
        name: "Golden Demise",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: wipe(Value::Const(-2), Value::Const(-2)),
        ..Default::default()
    }
}

/// Yahenni's Expertise — {2}{B}{B} Sorcery. All creatures get -3/-3 until end of
/// turn. (The free-cast rider is dropped.)
pub fn yahennis_expertise() -> CardDefinition {
    CardDefinition {
        name: "Yahenni's Expertise",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: wipe(Value::Const(-3), Value::Const(-3)),
        ..Default::default()
    }
}

/// Sword of the Animist — {2} Legendary Equipment. Equipped creature gets +1/+1.
/// Whenever it attacks, search your library for a basic land and put it onto the
/// battlefield tapped. Equip {2}.
pub fn sword_of_the_animist() -> CardDefinition {
    CardDefinition {
        name: "Sword of the Animist",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Equipment],
            ..Default::default()
        },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Dawn of Hope — {1}{W} Enchantment. Whenever you gain life, you may pay {2} to
/// draw a card. {3}{W}: create a 1/1 white Soldier with lifelink.
pub fn dawn_of_hope() -> CardDefinition {
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
        name: "Dawn of Hope",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::MayPay {
                description: "Pay {2}: draw a card.".into(),
                mana_cost: cost(&[generic(2)]),
                body: Box::new(draw(1)),
                else_: None,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), w()]),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::Const(1), definition: soldier },
            ..Default::default()
        }],
        ..Default::default()
    }
}
