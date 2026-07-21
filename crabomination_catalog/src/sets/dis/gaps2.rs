//! Dissension (DIS) second gap wave — split cards and effect-heavy spells
//! filling the remaining `set_gaps.py dis` list. Exercises the new
//! `SearchLibraryCreaturesUpToTotalManaValue`, `CounterAllOtherSpellsDrawPer`,
//! `RevealRandomDiscardNonland`, `SacrificedWasColor`, and
//! `LastDiscardedWasMulticolored` primitives.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, Predicate,
    SelectionRequirement as R, Selector, SplitCard, SplitHalf, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::{on_dies, target_any, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, x, Color};

/// Protean Hulk — {5}{G}{G} 6/6 Beast. When it dies, search your library for
/// any number of creature cards with total mana value 6 or less, put them onto
/// the battlefield, then shuffle.
pub fn protean_hulk() -> CardDefinition {
    CardDefinition {
        name: "Protean Hulk",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![on_dies(Effect::SearchLibraryCreaturesUpToTotalManaValue {
            max_total: Value::Const(6),
        })],
        ..Default::default()
    }
}

/// Swift Silence — {2}{W}{U}{U} Instant. Counter all other spells. Draw a card
/// for each spell countered this way.
pub fn swift_silence() -> CardDefinition {
    CardDefinition {
        name: "Swift Silence",
        cost: cost(&[generic(2), w(), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CounterAllOtherSpellsDrawPer,
        ..Default::default()
    }
}

/// Lyzolda, the Blood Witch — {1}{B}{R} 3/1 Legendary Human Cleric.
/// `{2}, Sacrifice a creature: Lyzolda deals 2 damage to any target if the
/// sacrificed creature was red. Draw a card if the sacrificed creature was
/// black.`
pub fn lyzolda_the_blood_witch() -> CardDefinition {
    CardDefinition {
        name: "Lyzolda, the Blood Witch",
        cost: cost(&[generic(1), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::SacrificedWasColor(Color::Red),
                    then: Box::new(Effect::DealDamage { to: target_any(), amount: Value::Const(2) }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::If {
                    cond: Predicate::SacrificedWasColor(Color::Black),
                    then: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stormscale Anarch — {2}{R}{R} 2/2 Lizard Shaman. `{2}{R}, Discard a card at
/// random: This creature deals 2 damage to any target. If the discarded card
/// was multicolored, it deals 4 damage instead.` (The random discard is
/// modeled as the lowest-value hand card.)
pub fn stormscale_anarch() -> CardDefinition {
    CardDefinition {
        name: "Stormscale Anarch",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::IfPred {
                    pred: Box::new(Predicate::LastDiscardedWasMulticolored),
                    then: Box::new(Value::Const(4)),
                    else_: Box::new(Value::Const(2)),
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crime // Punishment — {3}{W}{B} // {X}{B}{G} Sorcery // Sorcery. Crime puts
/// a creature or enchantment card from an opponent's graveyard onto the
/// battlefield under your control; Punishment destroys each artifact, creature,
/// and enchantment with mana value X.
pub fn crime_punishment() -> CardDefinition {
    CardDefinition {
        name: "Crime // Punishment",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Move {
            what: target_filtered(
                R::InOpponentGraveyard.and(R::Creature.or(R::Enchantment)),
            ),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[x(), b(), g()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Destroy {
                    what: Selector::EachPermanent(
                        R::Artifact
                            .or(R::Creature)
                            .or(R::Enchantment)
                            .and(R::ManaValueExactlyXFromCost),
                    ),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Hit // Run — {1}{B}{R} // {3}{R}{G} Instant // Instant. Hit makes a target
/// player sacrifice an artifact or creature, then deals damage to that player
/// equal to its mana value; Run pumps your attackers +1/+0 for each other
/// attacking creature.
pub fn hit_run() -> CardDefinition {
    CardDefinition {
        name: "Hit // Run",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::Target(0),
                filter: R::Artifact.or(R::Creature),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::SacrificedManaValue,
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(3), r(), g()]),
                card_types: vec![CardType::Instant],
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou)),
                    power: Value::Diff(
                        Box::new(Value::count(Selector::EachPermanent(R::IsAttacking))),
                        Box::new(Value::ONE),
                    ),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}

/// Rise // Fall — {U}{B} // {B}{R} Sorcery // Sorcery. Rise returns a creature
/// card from a graveyard and a creature on the battlefield to their owners'
/// hands; Fall makes a target player reveal two cards at random from their
/// hand and discard the nonland ones.
pub fn rise_fall() -> CardDefinition {
    CardDefinition {
        name: "Rise // Fall",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::InGraveyard.and(R::Creature)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Move {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(1)))),
            },
        ]),
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[b(), r()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::RevealRandomDiscardNonland {
                    who: Selector::Player(PlayerRef::Target(0)),
                    count: Value::Const(2),
                },
            },
            fuse: false,
            aftermath: false,
        })),
        ..Default::default()
    }
}
