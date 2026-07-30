//! DMU/VOW gap batch — a magecraft-loot Wall, a kicker-wheel Bird, and a
//! death-dig Elf. All on existing primitives. Tests in
//! `tests/recent_b/recent273.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, TriggeredAbility,
};
use crate::card::{EventKind, EventScope, EventSpec};
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{cost, g, generic, r, u, w};

/// Academy Wall — {2}{U} 0/5 Wall. Defender. Whenever you cast an instant or
/// sorcery spell, you may draw a card, then discard a card. Once each turn.
pub fn academy_wall() -> CardDefinition {
    CardDefinition {
        name: "Academy Wall",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                })
                .once_per_turn(),
            effect: Effect::MayDo {
                description: "Draw a card, then discard a card".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ONE,
                    },
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::ONE,
                        random: false,
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Battlewing Mystic — {1}{U} 2/1 Bird Wizard, kicker {R}. Flying. When it
/// enters, if it was kicked, discard your hand, then draw two cards.
pub fn battlewing_mystic() -> CardDefinition {
    CardDefinition {
        name: "Battlewing Mystic",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Kicker(cost(&[r()]))],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::HandSizeOf(PlayerRef::You),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ])),
            else_: Box::new(Effect::Noop),
        })],
        ..Default::default()
    }
}

/// Brazen Upstart — {R}{G}{W} 4/2 Elf Shaman. Vigilance. When it dies, look at
/// the top five cards of your library; you may reveal a creature card from
/// among them and put it into your hand. Put the rest on the bottom in a
/// random order.
pub fn brazen_upstart() -> CardDefinition {
    CardDefinition {
        name: "Brazen Upstart",
        cost: cost(&[r(), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(5),
                rest_to_graveyard: false,
                pick_filter: Some(R::Creature),
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                optional: true,
                picked_lands_to_battlefield: false,
                rest_bottom_random: true,
            },
        }],
        ..Default::default()
    }
}
