//! Star Trek (TRK/TRC) — the eighth batch of EDHREC Star Trek closers: six
//! commanders whose single-commander average decks miss only the commander,
//! and Georgiou's and Daniels' two-card gaps (scan of 2026-11-26). Tests in
//! `tests/recent_b/trek_singles8.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{add_any_one_color, etb, investigate, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u};

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

/// Commander Raffi Musiker — {2}{B} legendary 3/2 Human Officer. Entering,
/// choose one: exile up to one target card from a graveyard and gain 2 life;
/// or investigate and lose 2 life.
pub fn commander_raffi_musiker() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Seq(vec![
                    Effect::Move { what: target_filtered(R::Any.from_any_graveyard()), to: ZoneDest::Exile },
                    Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
                ])),
            },
            Effect::Seq(vec![
                investigate(1),
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        ]))],
        ..legend(
            "Commander Raffi Musiker",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Officer],
            3,
            2,
        )
    }
}

/// Culber and Stamets, Star-Crossed — {3}{G} legendary 3/4 Human Doctor
/// Artificer. Entering, choose one: a +1/+1 counter on target creature; or add
/// two mana of any one color.
pub fn culber_and_stamets_star_crossed() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            add_any_one_color(2),
        ]))],
        ..legend(
            "Culber and Stamets, Star-Crossed",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Doctor, CreatureType::Artificer],
            3,
            4,
        )
    }
}

/// Janice Rand, Essential Aide — {3}{G} legendary 3/3 Human Advisor. Whenever a
/// commander you control enters or attacks, a +1/+1 counter on it, then reveal
/// until a land: it enters tapped, the rest go to the bottom at random.
pub fn janice_rand_essential_aide() -> CardDefinition {
    let on = |kind: EventKind| TriggeredAbility {
        event: EventSpec::new(kind, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsCommander }),
        effect: Effect::Seq(vec![
            Effect::AddCounter { what: Selector::TriggerSource, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::RevealUntilLandsToBattlefield { count: Value::ONE, tapped: true },
        ]),
    };
    CardDefinition {
        triggered_abilities: vec![on(EventKind::EntersBattlefield), on(EventKind::Attacks)],
        ..legend(
            "Janice Rand, Essential Aide",
            cost(&[generic(3), g()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            3,
            3,
        )
    }
}

/// La'An Noonien-Singh, Security — {2}{R}{R} legendary 4/3 Human Soldier.
/// Whenever you attack while creatures you control have total power 8 or
/// greater, target creature can't block this turn.
pub fn laan_noonien_singh_security() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(Value::TotalPowerControlled, Value::Const(8))),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
        }],
        ..legend(
            "La'An Noonien-Singh, Security",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Lieutenant Nyota Uhura — {2}{U} legendary 2/3 Human Officer. Strange new
/// worlds — {4}, {T}: target opponent exiles the top X cards of their library,
/// X your differently named lands; you may cast an instant or sorcery among
/// them free; the rest go to the bottom of their library at random.
pub fn lieutenant_nyota_uhura() -> CardDefinition {
    let spell = || R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery));
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ExileTopOfLibrary {
                    who: target_filtered(R::OpponentPlayer),
                    amount: Value::DistinctNamesControlledMatching(R::Land),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::CastAnyOrderWithoutPaying {
                    what: Selector::ExiledThisResolution { filter: spell() },
                    source_zone: Zone::Exile,
                    filter: Some(spell()),
                    cap: Some(Value::ONE),
                    total_mana_value: None,
                    exile_after: false,
                },
                Effect::BottomInRandomOrder { what: Selector::ExiledThisResolution { filter: R::InExile } },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Lieutenant Nyota Uhura",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            3,
        )
    }
}

/// Rok-Tahk, Budding Scientist — {4}{G}{G} legendary 5/5 Brikar Scientist.
/// Creatures you control have "{T}: Add {G}{G}." Entering, create Murf, a
/// legendary 0/1 green Ooze Slug creature token with indestructible.
pub fn rok_tahk_budding_scientist() -> CardDefinition {
    let murf = TokenDefinition {
        name: "Murf".into(),
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Indestructible],
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze, CreatureType::Slug], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have \"{T}: Add {G}{G}.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                ability: ActivatedAbility {
                    tap_cost: true,
                    effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(vec![Color::Green, Color::Green]) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: std::sync::Arc::new(murf),
        })],
        ..legend(
            "Rok-Tahk, Budding Scientist",
            cost(&[generic(4), g(), g()]),
            vec![CreatureType::Brikar, CreatureType::Scientist],
            5,
            5,
        )
    }
}

/// Georgiou, Terran Emperor — {4}{B}{B} legendary 4/6 Human Rogue Noble. You
/// don't lose for 0 or less life. Your end step: lose 3, draw a card, then at
/// 0 or less life create ten 1/1 red Officers.
pub fn georgiou_terran_emperor() -> CardDefinition {
    let officer = TokenDefinition {
        name: "Officer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You don't lose the game for having 0 or less life.",
            effect: StaticEffect::ControllerDoesntLoseFromLife,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::ValueAtMost(Value::LifeOf(PlayerRef::You), Value::Const(0)),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(10),
                        definition: std::sync::Arc::new(officer),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        ..legend(
            "Georgiou, Terran Emperor",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Noble],
            4,
            6,
        )
    }
}

/// Cruel Bargain — {B}{B}{B} sorcery. Draw four cards; lose half your life,
/// rounded up.
pub fn cruel_bargain() -> CardDefinition {
    CardDefinition {
        name: "Cruel Bargain",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
            Effect::LoseHalfLife { who: Selector::You, rounded_up: true },
        ]),
        ..Default::default()
    }
}

/// Daniels, Temporal Agent — {2}{B/G}{B/G} legendary 3/1 Human Advisor.
/// Entering, return target creature card from your graveyard to hand. {2},
/// put this card from your graveyard on the bottom of your library: gain 2.
pub fn daniels_temporal_agent() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(R::Creature.and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            from_graveyard: true,
            bottom_self_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..legend(
            "Daniels, Temporal Agent",
            cost(&[generic(2), hybrid(Color::Black, Color::Green), hybrid(Color::Black, Color::Green)]),
            vec![CreatureType::Human, CreatureType::Advisor],
            3,
            1,
        )
    }
}

/// Stray from History — {2}{B} sorcery. Choose one: return one or two target
/// creature and/or Spacecraft cards from your graveyard to hand; or target
/// opponent reveals their hand and you exile a nonland card from it.
pub fn stray_from_history() -> CardDefinition {
    CardDefinition {
        name: "Stray from History",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::ApplyToTargets {
                min_targets: 1,
                max_targets: 2,
                filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft)).and(R::InYourGraveyard),
                effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
            },
            Effect::ExileChosenFromHand {
                from: target_filtered(R::OpponentPlayer),
                count: Value::ONE,
                filter: R::Nonland,
                link_to_source: false,
                face_down: false,
            },
        ]),
        ..Default::default()
    }
}
