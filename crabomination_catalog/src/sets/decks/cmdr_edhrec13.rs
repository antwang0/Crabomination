//! Commander: the primitive-blocked one-card gaps of EDHREC's top 1000 —
//! each card here completes one EDHREC average deck the 2026-10-12 rescans
//! left a single card short. Tests in `tests/recent_b/cmdr_edhrec13.rs`.

use crate::card::{
    CardDefinition, MayPlayDuration, WardCost, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec, Keyword,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{on_cast, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, ManaPayload, PlayerRef, Predicate};
use crate::mana::{b, cost, g, generic, u, w, Color, ManaCost};

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

/// Tyvar the Bellicose — {2}{B}{G} 5/4 legendary Elf Warrior. Attacking Elves
/// gain deathtouch; each creature you control has "whenever a mana ability of
/// this creature resolves, put that many +1/+1 counters on it" once a turn.
pub fn tyvar_the_bellicose() -> CardDefinition {
    let elf = || R::HasCreatureType(CreatureType::Elf);
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource)
                .with_filter(Predicate::AttackedWithCreatureMatching { who: PlayerRef::You, filter: elf() }),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(R::IsAttacking.and(R::ControlledByYou).and(elf())),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![StaticAbility {
            description: "Each creature you control has \"Whenever a mana ability of this creature resolves, put a number of +1/+1 counters on it equal to the amount of mana this creature produced. This ability triggers only once each turn.\"",
            effect: StaticEffect::GrantTriggeredAbility {
                filter: R::Creature.and(R::ControlledByYou),
                ability: Box::new(TriggeredAbility {
                    event: EventSpec::new(EventKind::TappedForMana, EventScope::SelfSource).once_per_turn(),
                    effect: Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::TriggerManaProduced,
                    },
                }),
            },
        }],
        ..legend(
            "Tyvar the Bellicose",
            cost(&[generic(2), b(), g()]),
            vec![CreatureType::Elf, CreatureType::Warrior],
            5,
            4,
        )
    }
}

/// Angel of Jubilation — {1}{W}{W}{W} 3/3 flying Angel. Other nonblack
/// creatures you control get +1/+1; no one may pay life or sacrifice creatures
/// to cast spells or activate abilities (CR 119.4b).
pub fn angel_of_jubilation() -> CardDefinition {
    CardDefinition {
        name: "Angel of Jubilation",
        cost: cost(&[generic(1), w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            StaticAbility {
                description: "Other nonblack creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        R::Creature
                            .and(R::ControlledByYou)
                            .and(R::Not(Box::new(R::HasColor(Color::Black))))
                            .and(R::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
            StaticAbility {
                description: "Players can't pay life or sacrifice creatures to cast spells or activate abilities.",
                effect: StaticEffect::PlayersCantPayLifeOrSacrificeCreaturesForCosts,
            },
        ],
        ..Default::default()
    }
}

/// Emrakul, the Exigent Doom — {10} 12/12 legendary Eldrazi: flying, trample,
/// ward—sacrifice three permanents; casting it untaps your lands. From hand,
/// {3} and exile it: a land gains "{T}: Add {C}{C}" until this card is cast
/// from exile, and it may be cast from exile.
pub fn emrakul_the_exigent_doom() -> CardDefinition {
    let tap_cc = ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::Const(2)) },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Ward(WardCost::SacrificePermanents(3))],
        triggered_abilities: vec![on_cast(Effect::Untap {
            what: Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            up_to: None,
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            from_hand: true,
            exile_self_cost: true,
            effect: Effect::Seq(vec![
                Effect::GainActivatedAbility {
                    what: target_filtered(R::Land),
                    ability: Box::new(tap_cc),
                    duration: Duration::UntilSourceCast,
                },
                Effect::GrantMayPlay {
                    what: Selector::This,
                    duration: MayPlayDuration::WhileExiled,
                    to_owner: true,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
            ]),
            ..Default::default()
        }],
        ..legend("Emrakul, the Exigent Doom", cost(&[generic(10)]), vec![CreatureType::Eldrazi], 12, 12)
    }
}

/// Mystic Reflection — {1}{U} instant, foretell {U}. Choose target nonlegendary
/// creature; the next time one or more creatures or planeswalkers enter this
/// turn, they enter as copies of it.
pub fn mystic_reflection() -> CardDefinition {
    CardDefinition {
        name: "Mystic Reflection",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        foretell_cost: Some(cost(&[u()])),
        effect: Effect::NextEntriesEnterAsCopy {
            of: target_filtered(R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary))))),
        },
        ..Default::default()
    }
}

/// Painter's Servant — {2} 1/3 artifact creature Scarecrow. As it enters,
/// choose a color: every card off the battlefield, spell and permanent is that
/// color in addition to its others.
pub fn painters_servant() -> CardDefinition {
    CardDefinition {
        name: "Painter's Servant",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Scarecrow], ..Default::default() },
        power: 1,
        toughness: 3,
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "All cards that aren't on the battlefield, spells, and permanents are the chosen color in addition to their other colors.",
            effect: StaticEffect::ChosenColorEverywhere,
        }],
        ..Default::default()
    }
}
