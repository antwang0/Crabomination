//! Guildpact (GPT) gap wave 6: a Rhystic-tax edict and a sacrifice-to-retarget
//! Goblin. Reuses `Effect::UnlessPlayerPays` and `ChooseNewTargetsForSpell`.
//! Tests in `classic_sets/gpt`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, SelectionRequirement as R, Subtypes, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector};
use crate::mana::{b, cost, generic, r, u, w, x};

/// Spelltithe Enforcer — {3}{W}{W} 3/3 Elephant Wizard. Whenever an opponent
/// casts a spell, that player sacrifices a permanent of their choice unless
/// they pay {1}.
pub fn spelltithe_enforcer() -> CardDefinition {
    CardDefinition {
        name: "Spelltithe Enforcer",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl),
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: crate::card::WardCost::generic(1),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    count: Value::ONE,
                    filter: R::Permanent,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Goblin Flectomancer — {U}{R}{R} 2/2 Goblin Wizard. Sacrifice this creature:
/// You may change the targets of target instant or sorcery spell.
pub fn goblin_flectomancer() -> CardDefinition {
    CardDefinition {
        name: "Goblin Flectomancer",
        cost: cost(&[u(), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::ChooseNewTargetsForSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Conjurer's Ban — {W}{B} Sorcery. Choose a card name; until your next turn,
/// spells with the chosen name can't be cast and lands with the chosen name
/// can't be played. Draw a card. (Modeled as the opponent-scoped cast lock;
/// the symmetric "you too" and land-play clauses are omitted.)
pub fn conjurers_ban() -> CardDefinition {
    CardDefinition {
        name: "Conjurer's Ban",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::NameOpponentCastLock,
            Effect::Draw {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]),
        ..Default::default()
    }
}

/// Droning Bureaucrats — {3}{W} 1/4 Human Advisor. {X}, {T}: Each creature with
/// mana value X can't attack or block this turn.
pub fn droning_bureaucrats() -> CardDefinition {
    CardDefinition {
        name: "Droning Bureaucrats",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x()]),
            tap_cost: true,
            effect: Effect::GrantKeywords {
                what: Selector::EachPermanent(R::Creature.and(R::ManaValueExactlyXFromCost)),
                keywords: vec![Keyword::CantAttack, Keyword::CantBlock],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}
