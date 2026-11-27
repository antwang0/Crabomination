//! Commander: closers for McCoy (Not a Bricklayer), Phlox, Saru
//! (Unthreatened) and Captain Benjamin Sisko's EDHREC average decks (scan of
//! 2026-11-27). Tests in `tests/recent_b/cmdr_01wr6s_a.rs`.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EquipBonus,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Value};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

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

/// A 1/1 red Officer creature token.
fn officer_token() -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: "Officer".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Officer], ..Default::default() },
        ..Default::default()
    })
}

/// "Whenever a creature you control enters".
fn your_creature_enters(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect,
    }
}

/// An Aura with flash and "enchant creature" granting `bonus`.
fn flash_aura(name: &'static str, mana: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::Flash],
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

fn hexproof_eot(what: Selector) -> Effect {
    Effect::GrantKeyword { what, keyword: Keyword::Hexproof, duration: Duration::EndOfTurn }
}

/// Sickbay — {5} artifact. Whenever a creature you control enters, gain 1
/// life. At the beginning of combat on your turn, X +1/+1 counters on target
/// creature you control, X = 1 + the life you gained this turn.
pub fn sickbay() -> CardDefinition {
    CardDefinition {
        name: "Sickbay",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            your_creature_enters(Effect::GainLife { who: Selector::You, amount: Value::ONE }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Sum(vec![Value::ONE, Value::LifeGainedThisTurn(PlayerRef::You)]),
                },
            },
        ],
        ..Default::default()
    }
}

/// Emergency Medical Hologram — {W} 0/1 artifact Illusion Doctor. ETB: a
/// +1/+1 counter on target creature; you gain 1 life.
pub fn emergency_medical_hologram() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::AddCounter { what: target_filtered(R::Creature), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            Effect::GainLife { who: Selector::You, amount: Value::ONE },
        ]))],
        ..creature(
            "Emergency Medical Hologram",
            cost(&[w()]),
            vec![CreatureType::Illusion, CreatureType::Doctor],
            0,
            1,
        )
    }
}

/// Photosynthetic Physician — {4}{G} 5/5 Plant Doctor, reach. {G}, exile it
/// from your graveyard: a 1/1 red Officer token; sorcery speed.
pub fn photosynthetic_physician() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            from_graveyard: true,
            exile_self_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: officer_token() },
            ..Default::default()
        }],
        ..creature(
            "Photosynthetic Physician",
            cost(&[generic(4), g()]),
            vec![CreatureType::Plant, CreatureType::Doctor],
            5,
            5,
        )
    }
}

/// I'm a Doctor, Not a . . . — {G} instant. A +1/+1 counter on target
/// creature you control; untap it; until end of turn it gets +1/+1 and is a
/// Doctor (no other creature types).
pub fn im_a_doctor_not_a() -> CardDefinition {
    let it = || Selector::Target(0);
    CardDefinition {
        name: "I'm a Doctor, Not a . . .",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::Untap { what: it(), up_to: None },
            Effect::PumpPT { what: it(), power: Value::ONE, toughness: Value::ONE, duration: Duration::EndOfTurn },
            Effect::BecomeCreatureType {
                what: it(),
                creature_types: vec![CreatureType::Doctor],
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Support Mission — {2}{G} enchantment. Whenever a creature you control
/// enters, a quest counter here and a +1/+1 counter on that creature — two
/// with five or more quest counters.
pub fn support_mission() -> CardDefinition {
    let counters = |n: i32| Effect::AddCounter {
        what: Selector::TriggerSource,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(n),
    };
    CardDefinition {
        name: "Support Mission",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![your_creature_enters(Effect::Seq(vec![
            Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
            Effect::If {
                cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Quest, n: 5 },
                then: Box::new(counters(2)),
                else_: Box::new(counters(1)),
            },
        ]))],
        ..Default::default()
    }
}

/// Saru, Unthreatened — {2}{G} legendary 3/2 Kelpien Officer, flash. ETB:
/// another target creature you control gains hexproof until end of turn.
pub fn saru_unthreatened() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(hexproof_eot(target_filtered(
            R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
        )))],
        ..creature(
            "Saru, Unthreatened",
            cost(&[generic(2), g()]),
            vec![CreatureType::Kelpien, CreatureType::Officer],
            3,
            2,
        )
    }
}

/// Aether Web — {1}{G} Aura, flash. Enchanted creature gets +1/+1, has reach
/// and can block creatures with shadow.
pub fn aether_web() -> CardDefinition {
    flash_aura(
        "Aether Web",
        cost(&[generic(1), g()]),
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Reach, Keyword::CanBlockShadow],
            ..Default::default()
        },
    )
}

/// Gilt-Leaf's Embrace — {2}{G} Aura, flash. ETB: the enchanted creature
/// gains trample and indestructible until end of turn. It gets +2/+0.
pub fn gilt_leafs_embrace() -> CardDefinition {
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GrantKeyword { what: host(), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            Effect::GrantKeyword { what: host(), keyword: Keyword::Indestructible, duration: Duration::EndOfTurn },
        ]))],
        ..flash_aura(
            "Gilt-Leaf's Embrace",
            cost(&[generic(2), g()]),
            EquipBonus { power: 2, toughness: 0, ..Default::default() },
        )
    }
}

/// Hapatra's Mark — {G} instant. Target creature you control gains hexproof
/// until end of turn; remove all -1/-1 counters from it.
pub fn hapatras_mark() -> CardDefinition {
    CardDefinition {
        name: "Hapatra's Mark",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            hexproof_eot(target_filtered(R::Creature.and(R::ControlledByYou))),
            Effect::RemoveCounter {
                what: Selector::Target(0),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::Const(i32::MAX),
            },
        ]),
        ..Default::default()
    }
}

/// The "enters tapped, gain 1 life" Planet dual.
fn planet(name: &'static str, a: Color, b: Color) -> CardDefinition {
    let mut d = crate::sets::tapland_untyped(name, a, b, crate::sets::etb_gain_one());
    d.subtypes.land_types.push(crate::card::LandType::Planet);
    d
}

/// Bajor — land, Planet. Enters tapped; gain 1 life; {T}: {R} or {W}.
pub fn bajor() -> CardDefinition {
    planet("Bajor", Color::Red, Color::White)
}

/// Ferenginar — land, Planet. Enters tapped; gain 1 life; {T}: {B} or {G}.
pub fn ferenginar() -> CardDefinition {
    planet("Ferenginar", Color::Black, Color::Green)
}

/// Dathon and Picard at El-Adrel — {2}{G} instant. Choose one — target
/// creature gets +3/+3 and trample until end of turn; or each opponent
/// sacrifices a nontoken artifact or enchantment.
pub fn dathon_and_picard_at_el_adrel() -> CardDefinition {
    CardDefinition {
        name: "Dathon and Picard at El-Adrel",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword { what: Selector::Target(0), keyword: Keyword::Trample, duration: Duration::EndOfTurn },
            ]),
            Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Artifact.or(R::Enchantment).and(R::Not(Box::new(R::IsToken))),
            },
        ]),
        ..Default::default()
    }
}

/// Captain Benjamin Sisko — {1}{W} legendary 2/2 Human Officer. At the
/// beginning of your end step, a 1/1 red Officer per dilemma you faced this
/// turn. {2}{W}{U}{B}{R}{G}, {T}: exile the top five; cast any number of
/// spells with mana value at most your Officers from among them free.
pub fn captain_benjamin_sisko() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::DilemmasFacedThisTurn(PlayerRef::You),
                definition: officer_token(),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w(), u(), b(), r(), g()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::ClearLastMoved,
                Effect::ExileTopOfLibrary {
                    who: Selector::You,
                    amount: Value::Const(5),
                    link_to_source: false,
                    face_down: false,
                },
                Effect::CastAnyOrderWithoutPaying {
                    what: Selector::LastMoved,
                    source_zone: Zone::Exile,
                    filter: Some(R::ManaValueAtMostYourCount(Box::new(R::HasCreatureType(CreatureType::Officer)))),
                    cap: None,
                    total_mana_value: None,
                    exile_after: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Captain Benjamin Sisko",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Officer],
            2,
            2,
        )
    }
}
