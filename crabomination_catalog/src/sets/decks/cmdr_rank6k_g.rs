//! Commander: one-card gaps of EDHREC's commanders ranked 5501-6500 (the
//! 2026-10-14 scan), seventh file. Tests in `tests/recent_b/cmdr_rank6k_g.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest, ZoneRef};
use crate::mana::{cost, generic, r, w};
use std::sync::Arc;

/// Island Sanctuary — {1}{W} enchantment. If you would draw a card during
/// your draw step, you may skip that draw instead; if you do, until your
/// next turn you can't be attacked except by creatures with flying and/or
/// islandwalk (CR 614.1a, `game::draw_skip_shield`).
pub fn island_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Island Sanctuary",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If you would draw a card during your draw step, instead you may skip that draw. If you do, until your next turn, you can't be attacked except by creatures with flying and/or islandwalk.",
            effect: StaticEffect::ControllerMaySkipDrawStepDrawForAttackShield {
                attackers: R::HasKeyword(Keyword::Flying).or(R::HasKeyword(Keyword::Landwalk(LandType::Island))),
            },
        }],
        ..Default::default()
    }
}

/// You've Been Caught Stealing — {1}{R} sorcery. Choose one: each creature
/// blocks this turn if able; or a Treasure for each opponent who was dealt
/// damage this turn.
pub fn youve_been_caught_stealing() -> CardDefinition {
    CardDefinition {
        name: "You've Been Caught Stealing",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::OpponentsDealtDamageThisTurn,
                definition: Arc::new(crabomination_base::tokens::treasure_token()),
            },
        ]),
        ..Default::default()
    }
}

/// Livio, Oathsworn Sentinel — {1}{W} 2/2 legendary Human Knight, partner.
/// {1}{W}: choose another target creature; its controller may exile it with
/// an aegis counter on it. {2}{W}, {T}: return all exiled cards with aegis
/// counters to the battlefield under their owners' control, all at once
/// (`OwnerOfMoved`, so each entry sees the others — CR 603.6a).
pub fn livio_oathsworn_sentinel() -> CardDefinition {
    CardDefinition {
        name: "Livio, Oathsworn Sentinel",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Partner],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::MayDoBy {
                    who: PlayerRef::ControllerOf(Box::new(target_filtered(R::Creature.and(R::OtherThanSource)))),
                    description: "Exile your creature with an aegis counter on it (Livio)?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile },
                        Effect::AddCounter { what: Selector::LastMoved, kind: CounterType::Aegis, amount: Value::ONE },
                    ])),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: Selector::EachMatching { zone: ZoneRef::Exile, filter: R::WithCounter(CounterType::Aegis) },
                    to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Permission Denied — {W}{U} instant. Counter target noncreature spell; your
/// opponents can't cast noncreature spells this turn (each opponent, even if
/// the spell can't be countered — rulings).
pub fn permission_denied() -> CardDefinition {
    CardDefinition {
        name: "Permission Denied",
        cost: cost(&[w(), crate::mana::u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell { what: target_filtered(R::Noncreature) },
            Effect::CantCastNoncreatureThisTurn { who: Selector::Player(PlayerRef::EachOpponent) },
        ]),
        ..Default::default()
    }
}

/// Helm of the Ghastlord — {3}{U/B} Aura. A blue host gets +1/+1 and draws
/// when it deals damage to an opponent; a black host gets +1/+1 and that
/// player discards. Both when it is both (rulings); the colour is read as
/// the damage is dealt.
pub fn helm_of_the_ghastlord() -> CardDefinition {
    use crate::card::{ConditionalEquipBonus, EnchantmentSubtype, EquipBonus};
    use crate::effect::{EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    use crate::mana::Color;
    let on_damage = |color: Color, effect: Effect| TriggeredAbility {
        event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource).with_filter(Predicate::All(vec![
            Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer },
            Predicate::EntityMatches { what: Selector::This, filter: R::HasColor(color) },
        ])),
        effect,
    };
    let bonus = |color: Color| ConditionalEquipBonus {
        host_filter: R::HasColor(color),
        power: 1,
        toughness: 1,
        ..Default::default()
    };
    CardDefinition {
        name: "Helm of the Ghastlord",
        cost: cost(&[generic(3), crate::mana::hybrid(Color::Blue, Color::Black)]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            conditional: vec![bonus(Color::Blue), bonus(Color::Black)],
            triggered_abilities: vec![
                on_damage(Color::Blue, Effect::Draw { who: Selector::You, amount: Value::ONE }),
                on_damage(
                    Color::Black,
                    Effect::Discard {
                        who: Selector::Player(PlayerRef::TriggerEventPlayer),
                        amount: Value::ONE,
                        random: false,
                    },
                ),
            ],
            ..Default::default()
        }),
        ..Default::default()
    }
}
