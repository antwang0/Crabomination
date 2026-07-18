//! MKM (Murders at Karlov Manor) gap batch — Ravnica guild legends.
//! Tests in `tests/recent_b/recent253.rs`.

use crate::card::{CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement as R, Subtypes, Supertype};
use crate::effect::shortcut::{etb, investigate, target_filtered};
use crate::effect::{
    ActivatedAbility, Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate,
    Selector, TriggeredAbility, Value,
};
use crate::mana::{cost, g, generic, hybrid, r, u, w, Color};

fn gw() -> crate::mana::ManaSymbol {
    hybrid(Color::Green, Color::White)
}

/// Trostani, Three Whispers — {G}{G/W}{W} Legendary Creature — Dryad 4/4.
/// {1}{G}: Target creature gains deathtouch until end of turn.
/// {G/W}: Target creature gains vigilance until end of turn.
/// {2}{W}: Target creature gains double strike until end of turn.
pub fn trostani_three_whispers() -> CardDefinition {
    let grant = |mana, kw| ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: kw,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    CardDefinition {
        name: "Trostani, Three Whispers",
        cost: cost(&[g(), gw(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dryad], ..Default::default() },
        power: 4,
        toughness: 4,
        activated_abilities: vec![
            grant(cost(&[generic(1), g()]), Keyword::Deathtouch),
            grant(cost(&[gw()]), Keyword::Vigilance),
            grant(cost(&[generic(2), w()]), Keyword::DoubleStrike),
        ],
        ..Default::default()
    }
}

/// Ezrim, Agency Chief — {1}{W}{W}{U}{U} Legendary Creature — Archon Detective
/// 5/5, flying. When Ezrim enters, investigate twice. {1}, Sacrifice an
/// artifact: Ezrim gains your choice of vigilance, lifelink, or hexproof until
/// end of turn.
pub fn ezrim_agency_chief() -> CardDefinition {
    CardDefinition {
        name: "Ezrim, Agency Chief",
        cost: cost(&[generic(1), w(), w(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Archon, CreatureType::Detective],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(investigate(2))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_other_filter: Some((R::Artifact.and(R::ControlledByYou), 1)),
            effect: Effect::ChooseMode(vec![
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
                Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Hexproof, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Agrus Kos, Spirit of Justice — {2}{R}{W} Legendary Creature — Spirit
/// Detective 2/4, double strike, vigilance. Whenever Agrus Kos enters or
/// attacks, choose up to one target creature. If it's suspected, exile it.
/// Otherwise, suspect it.
pub fn agrus_kos_spirit_of_justice() -> CardDefinition {
    let interrogate = Effect::ApplyToTargets {
        max_targets: 1,
        min_targets: 0,
        filter: R::Creature,
        effect: Box::new(Effect::If {
            cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsSuspected },
            then: Box::new(Effect::Exile { what: Selector::Target(0) }),
            else_: Box::new(Effect::Suspect { what: Selector::Target(0) }),
        }),
    };
    CardDefinition {
        name: "Agrus Kos, Spirit of Justice",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Detective],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::DoubleStrike, Keyword::Vigilance],
        triggered_abilities: vec![
            etb(interrogate.clone()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
                effect: interrogate,
            },
        ],
        ..Default::default()
    }
}

/// Aurelia, the Law Above — {3}{R}{W} Legendary Creature — Angel 4/4, flying,
/// vigilance, haste. Whenever a player attacks with three or more creatures,
/// you draw a card. Whenever a player attacks with five or more creatures,
/// Aurelia deals 3 damage to each of your opponents and you gain 3 life.
pub fn aurelia_the_law_above() -> CardDefinition {
    CardDefinition {
        name: "Aurelia, the Law Above",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Haste],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                    Predicate::AttackedWithCountAtLeast { who: PlayerRef::ActivePlayer, at_least: 3 },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                    Predicate::AttackedWithCountAtLeast { who: PlayerRef::ActivePlayer, at_least: 5 },
                ),
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(3),
                    },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
                ]),
            },
        ],
        ..Default::default()
    }
}

/// Rakdos, Patron of Chaos — {4}{B}{R} Legendary Creature — Demon 6/6, flying,
/// trample. At the beginning of your end step, target opponent may sacrifice
/// two nonland, nontoken permanents of their choice. If they don't, you draw
/// two cards. (Modeled via `Effect::Punisher`: the opponent sacrifices when
/// able, otherwise you draw.)
pub fn rakdos_patron_of_chaos() -> CardDefinition {
    use crate::game::TurnStep;
    use crate::mana::b;
    CardDefinition {
        name: "Rakdos, Patron of Chaos",
        cost: cost(&[generic(4), b(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(2),
                    filter: R::Nonland.and(R::NotToken),
                }],
                otherwise: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(2) }),
            },
        }],
        ..Default::default()
    }
}

/// Voja, Jaws of the Conclave — {2}{R}{G}{W} Legendary Creature — Wolf 5/5,
/// vigilance, trample, ward {3}. Whenever Voja attacks, put X +1/+1 counters on
/// each creature you control, where X is the number of Elves you control; then
/// draw a card for each Wolf you control.
pub fn voja_jaws_of_the_conclave() -> CardDefinition {
    use crate::card::WardCost;
    use crate::mana::r;
    let elves = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Any)),
        filter: R::HasCreatureType(CreatureType::Elf).and(R::ControlledByYou),
    };
    let wolves = Value::CountMatching {
        sel: Box::new(Selector::EachPermanent(R::Any)),
        filter: R::HasCreatureType(CreatureType::Wolf).and(R::ControlledByYou),
    };
    CardDefinition {
        name: "Voja, Jaws of the Conclave",
        cost: cost(&[generic(2), r(), g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wolf], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![
            Keyword::Vigilance,
            Keyword::Trample,
            Keyword::Ward(WardCost::Mana(cost(&[generic(3)]))),
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: elves,
                },
                Effect::Draw { who: Selector::You, amount: wolves },
            ]),
        }],
        ..Default::default()
    }
}
