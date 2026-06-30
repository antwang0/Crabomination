use crate::card::{
    CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement, Subtypes, TriggeredAbility,
};
use crate::effect::shortcut::on_dies;
use crate::effect::{Effect, PlayerRef, Selector, Value};
use crate::mana::{Color, b, cost, generic, hybrid, w};

// CR 702.55 — Haunt. A haunt creature is exiled (haunting a creature) when it
// dies instead of going to the graveyard; a haunt instant/sorcery does the same
// after it resolves. When the haunted creature dies, the haunt body fires.
// `Effect::HauntCreature { body }` performs the exile + death-watch; `body` is
// the "when the haunted creature dies" effect.

/// Mourning Thrull — {1}{W/B} 1/1 Thrull. Flying, haunt. When it enters or the
/// creature it haunts dies, you gain 2 life and draw a card.
pub fn mourning_thrull() -> CardDefinition {
    let payoff = Effect::Seq(vec![
        Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
    ]);
    CardDefinition {
        name: "Mourning Thrull",
        cost: cost(&[generic(1), hybrid(Color::White, Color::Black)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thrull], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: payoff.clone(),
            },
            on_dies(Effect::HauntCreature { body: Box::new(payoff) }),
        ],
        ..Default::default()
    }
}

/// Absolver Thrull — {2}{W} 2/2 Thrull. Haunt. When it enters or the creature
/// it haunts dies, destroy target enchantment.
pub fn absolver_thrull() -> CardDefinition {
    let destroy = Effect::Destroy {
        what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Enchantment },
    };
    CardDefinition {
        name: "Absolver Thrull",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thrull], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: destroy.clone(),
            },
            on_dies(Effect::HauntCreature { body: Box::new(destroy) }),
        ],
        ..Default::default()
    }
}

/// Shrieking Grotesque — {2}{W} 2/1 Gargoyle. Flying, haunt. When the creature
/// it haunts dies, target player discards a card.
pub fn shrieking_grotesque() -> CardDefinition {
    CardDefinition {
        name: "Shrieking Grotesque",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::HauntCreature {
            body: Box::new(Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            }),
        })],
        ..Default::default()
    }
}

/// Cry of Contrition — {B} Sorcery. Target player discards a card. Haunt — when
/// the creature it haunts dies, target player discards a card.
pub fn cry_of_contrition() -> CardDefinition {
    CardDefinition {
        name: "Cry of Contrition",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(1),
                random: false,
            },
            Effect::HauntCreature {
                body: Box::new(Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: false,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Douse in Gloom — {2}{B} Instant. Deal 2 to target creature, gain 2 life.
/// Haunt — when the creature it haunts dies, deal 2 to each opponent, gain 2
/// (printed "that creature's controller"; modeled as each opponent).
pub fn douse_in_gloom() -> CardDefinition {
    CardDefinition {
        name: "Douse in Gloom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            Effect::HauntCreature {
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(2),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ])),
            },
        ]),
        ..Default::default()
    }
}

/// Castigate — {W}{B} Sorcery. Target opponent reveals their hand; exile a
/// nonland card from it. Haunt — repeat when the creature it haunts dies.
pub fn castigate() -> CardDefinition {
    CardDefinition {
        name: "Castigate",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ExileChosenFromHand {
                from: Selector::Player(PlayerRef::Target(0)),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
            Effect::HauntCreature {
                body: Box::new(Effect::ExileChosenFromHand {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(1),
                    filter: SelectionRequirement::Nonland,
                }),
            },
        ]),
        ..Default::default()
    }
}
