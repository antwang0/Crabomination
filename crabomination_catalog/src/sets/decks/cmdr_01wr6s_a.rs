//! Commander: closers for Sulu, Heroic Helmsman; Commander Spock; and Seven of
//! Nine's EDHREC average decks (scan of 2026-11-27). Tests in
//! `tests/recent_b/cmdr_01wr6s_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind,
    EventScope, EventSpec, Keyword, SelectionRequirement as R, Selector, StaticAbility, Subtypes,
    Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{draw, etb, on_attack, target_filtered};
use crate::effect::{Duration, Effect, KeywordAct, PlayerRef, StaticEffect, Value, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, u, w};

fn creature(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(name: &'static str, mana: crate::mana::ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

/// "Whenever you face a dilemma" (CR 700.2).
fn on_dilemma(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Performed(KeywordAct::FacedDilemma), EventScope::YourControl),
        effect,
    }
}

/// The "enters tapped, gain 1 life" Planet dual (Risa, Vulcan).
fn planet(name: &'static str, a: Color, b: Color) -> CardDefinition {
    let mut d = crate::sets::tapland_untyped(name, a, b, crate::sets::etb_gain_one());
    d.subtypes.land_types.push(crate::card::LandType::Planet);
    d
}

/// Sulu, Heroic Helmsman — {1}{W} legendary 2/2 Human Pilot. He stations
/// Spacecraft and crews Vehicles as though his power were tripled.
pub fn sulu_heroic_helmsman() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sulu stations Spacecraft and crews Vehicles as though his power were tripled.",
            effect: StaticEffect::SelfCrewsStationsPowerTimes { factor: 3 },
        }],
        ..legend("Sulu, Heroic Helmsman", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Pilot], 2, 2)
    }
}

/// DOT-7 Repair Squad — {3}{W} 3/3 artifact Robot, flying. {1}{W}: untap
/// target creature; sorcery speed, once each turn.
pub fn dot_7_repair_squad() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Untap { what: target_filtered(R::Creature), up_to: None },
            sorcery_speed: true,
            once_per_turn: true,
            ..Default::default()
        }],
        ..creature("DOT-7 Repair Squad", cost(&[generic(3), w()]), vec![CreatureType::Robot], 3, 3)
    }
}

/// Commander Spock — {2}{G}{W}{U} legendary 3/5 Vulcan Human Scientist.
/// Enters or attacks: reveal the top card; cast it free if its mana value is
/// at most your differently named lands, or put a land onto the battlefield
/// tapped; if it is still on top, draw a card.
pub fn commander_spock() -> CardDefinition {
    let strange_new_worlds = || Effect::RevealTopCastOrLandElseDraw {
        max_mv: Value::DistinctNamesControlledMatching(R::Land),
    };
    CardDefinition {
        triggered_abilities: vec![etb(strange_new_worlds()), on_attack(strange_new_worlds())],
        ..legend(
            "Commander Spock",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Vulcan, CreatureType::Human, CreatureType::Scientist],
            3,
            5,
        )
    }
}

/// Risa — land, Planet. Enters tapped; gain 1 life; {T}: {G} or {W}.
pub fn risa() -> CardDefinition {
    planet("Risa", Color::Green, Color::White)
}

/// Vulcan — land, Planet. Enters tapped; gain 1 life; {T}: {W} or {U}.
pub fn vulcan() -> CardDefinition {
    planet("Vulcan", Color::White, Color::Blue)
}

/// Captain Kirk, Boldly Going — {1}{G}{W}{U} legendary 3/4 Human Officer,
/// vigilance. Spacecraft creatures you control get +3/+3. Enters or attacks:
/// a basic land card or a card named Starship Enterprise from your library
/// and/or graveyard to your hand.
pub fn captain_kirk_boldly_going() -> CardDefinition {
    let fetch = || {
        crate::sets::search_library_and_or_graveyard(
            R::IsBasicLand.or(R::HasName("Starship Enterprise".into())),
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Spacecraft creatures you control get +3/+3.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft)).and(R::ControlledByYou),
                power: 3,
                toughness: 3,
                keywords: vec![],
                opponents: false,
                all_players: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![etb(fetch()), on_attack(fetch())],
        ..legend(
            "Captain Kirk, Boldly Going",
            cost(&[generic(1), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Officer],
            3,
            4,
        )
    }
}

/// Seven of Nine — {2}{U}{B} legendary 2/4 artifact Human Scientist.
/// Whenever you face a dilemma, draw a card.
pub fn seven_of_nine() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_dilemma(draw(1))],
        ..legend(
            "Seven of Nine",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Human, CreatureType::Scientist],
            2,
            4,
        )
    }
}

/// Melkotian Adjudicator — {2}{U} 2/1 Melkotian Advisor, flying, vigilance.
/// Whenever you face a dilemma, a +1/+1 counter on it.
pub fn melkotian_adjudicator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![on_dilemma(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..creature(
            "Melkotian Adjudicator",
            cost(&[generic(2), u()]),
            vec![CreatureType::Melkotian, CreatureType::Advisor],
            2,
            1,
        )
    }
}

/// Tal Shiar Operative — {1}{B} 2/2 Romulan Officer Spy. Attacks: choose
/// one — target creature you control gains deathtouch until end of turn; or
/// surveil 1.
pub fn tal_shiar_operative() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        ]))],
        ..creature(
            "Tal Shiar Operative",
            cost(&[generic(1), b()]),
            vec![CreatureType::Romulan, CreatureType::Officer, CreatureType::Spy],
            2,
            2,
        )
    }
}

/// Tell Me What You See — {1}{B} enchantment. ETB: target opponent may have
/// you draw a card; if they don't, they lose 4 life and you surveil 1.
/// Whenever you face a dilemma, you may return it to its owner's hand.
pub fn tell_me_what_you_see() -> CardDefinition {
    CardDefinition {
        name: "Tell Me What You See",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::TargetPlayerThen {
                filter: R::OpponentPlayer,
                then: Box::new(Effect::AnyPlayerMayAccept {
                    who: PlayerRef::Target(0),
                    prompt: "Have them draw a card (else you lose 4 life and they surveil 1)?".into(),
                    accepted: Box::new(draw(1)),
                    otherwise: Box::new(Effect::Seq(vec![
                        Effect::LoseLife { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(4) },
                        Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
                    ])),
                }),
            }),
            on_dilemma(Effect::MayDo {
                description: "Return Tell Me What You See to its owner's hand?".into(),
                body: Box::new(Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) }),
            }),
        ],
        ..Default::default()
    }
}

/// Resistance Is Futile — {1}{B} sorcery. Choose one — destroy target
/// nonartifact creature; or return target artifact creature card with mana
/// value 3 or less from your graveyard to the battlefield.
pub fn resistance_is_futile() -> CardDefinition {
    CardDefinition {
        name: "Resistance Is Futile",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Destroy { what: target_filtered(R::Creature.and(R::Not(Box::new(R::Artifact)))) },
            Effect::Move {
                what: target_filtered(
                    R::Artifact.and(R::Creature).and(R::ManaValueAtMost(3)).from_your_graveyard(),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]),
        ..Default::default()
    }
}
