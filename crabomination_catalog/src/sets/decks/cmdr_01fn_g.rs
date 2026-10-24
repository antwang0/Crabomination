//! Commander: partner-pair closers, batch four (`01FnAVVR`). Tests in
//! `tests/recent_b/cmdr_01fn_a.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, Keyword,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Zone,
};
use crate::effect::shortcut::{etb, investigate, on_attack, on_dies, target_filtered, unearth};
use crate::effect::{Duration, Effect, EventKind, EventScope, EventSpec, PlayerRef, Selector, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, w};
use crabomination_base::tokens::treasure_token;

fn creature(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

/// "The number of permanents you control of the chosen type."
fn chosen_type_count() -> Value {
    Value::PermanentCountControlledByMatching(PlayerRef::You, R::IsSourceChosenCreatureType)
}

/// Wanted Scoundrels — {1}{B} 4/3 Human Pirate. Dies: target opponent makes
/// two Treasures.
pub fn wanted_scoundrels() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_dies(Effect::Seq(vec![
            crate::effect::shortcut::declare_target_opponent(),
            Effect::CreateToken {
                who: PlayerRef::Target(0),
                count: Value::Const(2),
                definition: std::sync::Arc::new(treasure_token()),
            },
        ]))],
        ..creature("Wanted Scoundrels", cost(&[generic(1), b()]), vec![CreatureType::Human, CreatureType::Pirate], 4, 3)
    }
}

/// Overcome — {3}{G}{G} sorcery. Creatures you control get +2/+2 and gain
/// trample until end of turn.
pub fn overcome() -> CardDefinition {
    let yours = || Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    spell(
        "Overcome",
        cost(&[generic(3), g(), g()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::PumpPT { what: yours(), power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: yours(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
        ]),
    )
}

/// Hell's Thunder — {1}{R}{R} 4/4 flying, haste Elemental, sacrificed at the
/// beginning of the end step; unearth {4}{R}.
pub fn hells_thunder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![crate::card::TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer),
            effect: Effect::SacrificeSource,
        }],
        activated_abilities: vec![unearth(cost(&[generic(4), r()]))],
        ..creature("Hell's Thunder", cost(&[generic(1), r(), r()]), vec![CreatureType::Elemental], 4, 4)
    }
}

/// Preyseizer Dragon — {4}{R}{R} 4/4 flying Dragon, devour 2. Attacking, it
/// deals damage to any target equal to its +1/+1 counters.
pub fn preyseizer_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(crate::effect::shortcut::devour(2)),
        triggered_abilities: vec![on_attack(Effect::DealDamage {
            to: target_filtered(R::Any),
            amount: Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::PlusOnePlusOne },
        })],
        ..creature("Preyseizer Dragon", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 4, 4)
    }
}

/// Conservatory — land, enters tapped. {T}: add {G} or {W}. {4}, {T}:
/// investigate.
pub fn conservatory() -> CardDefinition {
    CardDefinition {
        name: "Conservatory",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            crate::sets::tap_add(Color::Green),
            crate::sets::tap_add(Color::White),
            ActivatedAbility { mana_cost: cost(&[generic(4)]), tap_cost: true, effect: investigate(1), ..Default::default() },
        ],
        ..Default::default()
    }
}

/// Pack's Disdain — {1}{B} instant. Choose a creature type: target creature
/// gets -1/-1 for each permanent of that type you control.
pub fn packs_disdain() -> CardDefinition {
    let minus = || Value::Diff(Box::new(Value::ZERO), Box::new(chosen_type_count()));
    spell(
        "Pack's Disdain",
        cost(&[generic(1), b()]),
        CardType::Instant,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: minus(),
                toughness: minus(),
                duration: Duration::EndOfTurn,
            }),
        },
    )
}

/// Roar of the Crowd — {3}{R} sorcery. Choose a creature type: damage to any
/// target equal to the permanents of that type you control.
pub fn roar_of_the_crowd() -> CardDefinition {
    spell(
        "Roar of the Crowd",
        cost(&[generic(3), r()]),
        CardType::Sorcery,
        Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::DealDamage { to: target_filtered(R::any_target()), amount: chosen_type_count() }),
        },
    )
}

/// Ravager's Mace — {1}{B}{R} Equipment, equip {2}{B}{R}. ETB: attach to
/// target creature you control. Equipped creature: +1/+0 per party member,
/// and menace.
pub fn ravagers_mace() -> CardDefinition {
    let equipped = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        name: "Ravager's Mace",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2), b(), r()]))],
        triggered_abilities: vec![etb(Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
        })],
        equipped_bonus: Some(EquipBonus { keywords: vec![Keyword::Menace], ..Default::default() }),
        static_abilities: vec![StaticAbility {
            description: "Equipped creature gets +1/+0 for each creature in your party.",
            effect: StaticEffect::PumpPTByValue { applies_to: equipped(), power: Value::PartyCount, toughness: Value::ZERO },
        }],
        ..Default::default()
    }
}

/// Basri's Aegis — {2}{W}{W} sorcery. A +1/+1 counter on each of up to two
/// target creatures; you may search your library and/or graveyard for Basri,
/// Devoted Paladin.
pub fn basris_aegis() -> CardDefinition {
    spell(
        "Basri's Aegis",
        cost(&[generic(2), w(), w()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::ApplyToTargets {
                min_targets: 0,
                max_targets: 2,
                filter: R::Creature,
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
            Effect::MayDo {
                description: "Search for Basri, Devoted Paladin?".into(),
                body: Box::new(Effect::SearchZones {
                    who: PlayerRef::You,
                    zones: vec![Zone::Library, Zone::Graveyard],
                    filter: R::HasName("Basri, Devoted Paladin".into()),
                    to: crate::effect::ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ]),
    )
}
