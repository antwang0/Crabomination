//! Urza-block echo classics — unblocked by the CR 702.29 echo turn-based
//! action (`process_echo`). Tests in `tests/echo.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Keyword, SelectionRequirement, Subtypes,
    TokenDefinition, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Effect, PlayerRef, StaticEffect};
use crate::mana::{cost, g, generic, r};

use SelectionRequirement as R;

fn echo_creature(
    name: &'static str,
    mana: &[crate::mana::ManaSymbol],
    types: Vec<CreatureType>,
    pt: (i32, i32),
    kws: Vec<Keyword>,
    etb_effect: Option<Effect>,
) -> CardDefinition {
    let mut keywords = kws;
    keywords.push(Keyword::Echo(cost(mana)));
    CardDefinition {
        name,
        cost: cost(mana),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: pt.0,
        toughness: pt.1,
        keywords,
        triggered_abilities: etb_effect.map(etb).into_iter().collect(),
        ..Default::default()
    }
}

/// Avalanche Riders — {3}{R} 2/2 haste, echo {3}{R}. ETB: destroy target land.
pub fn avalanche_riders() -> CardDefinition {
    echo_creature(
        "Avalanche Riders",
        &[generic(3), r()],
        vec![CreatureType::Human, CreatureType::Nomad],
        (2, 2),
        vec![Keyword::Haste],
        Some(Effect::Destroy {
            what: target_filtered(R::Land),
        }),
    )
}

/// Keldon Vandals — {2}{R} 4/1, echo {2}{R}. ETB: destroy target artifact.
pub fn keldon_vandals() -> CardDefinition {
    echo_creature(
        "Keldon Vandals",
        &[generic(2), r()],
        vec![CreatureType::Human, CreatureType::Rogue],
        (4, 1),
        vec![],
        Some(Effect::Destroy {
            what: target_filtered(R::Artifact),
        }),
    )
}

/// Deranged Hermit — {3}{G}{G} 1/1 Elf, echo {3}{G}{G}. ETB: four 1/1
/// Squirrels; Squirrel creatures get +1/+1.
pub fn deranged_hermit() -> CardDefinition {
    let squirrel = TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Squirrel],
            ..Default::default()
        },
        colors: vec![crate::mana::Color::Green],
        ..Default::default()
    };
    let mut def = echo_creature(
        "Deranged Hermit",
        &[generic(3), g(), g()],
        vec![CreatureType::Elf],
        (1, 1),
        vec![],
        Some(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(4),
            definition: squirrel,
        }),
    );
    def.static_abilities = vec![crate::card::StaticAbility {
        description: "Squirrel creatures get +1/+1.",
        effect: StaticEffect::AnthemForFilter {
            filter: R::HasCreatureType(CreatureType::Squirrel),
            power: 1,
            toughness: 1,
            keywords: vec![],
            opponents: false,
            all_players: true,
            only_your_turn: false,
            scale_by_counters_on_self: None,
        },
    }];
    def
}

/// Multani's Acolyte — {G}{G} 2/1, echo {G}{G}. ETB: draw a card.
pub fn multanis_acolyte() -> CardDefinition {
    echo_creature(
        "Multani's Acolyte",
        &[g(), g()],
        vec![CreatureType::Elf],
        (2, 1),
        vec![],
        Some(Effect::Draw {
            who: crate::effect::Selector::You,
            amount: Value::ONE,
        }),
    )
}

/// Radiant's Dragoons — {3}{W} 2/5, echo {3}{W}. ETB: gain 5 life.
pub fn radiants_dragoons() -> CardDefinition {
    echo_creature(
        "Radiant's Dragoons",
        &[generic(3), crate::mana::w()],
        vec![CreatureType::Human, CreatureType::Soldier],
        (2, 5),
        vec![],
        Some(Effect::GainLife {
            who: crate::effect::Selector::You,
            amount: Value::Const(5),
        }),
    )
}

/// Ticking Gnomes — {3} 3/3 artifact Gnome, echo {3}. Sacrifice: 1 damage
/// to any target.
pub fn ticking_gnomes() -> CardDefinition {
    let mut def = echo_creature(
        "Ticking Gnomes",
        &[generic(3)],
        vec![CreatureType::Gnome],
        (3, 3),
        vec![],
        None,
    );
    def.card_types = vec![CardType::Artifact, CardType::Creature];
    def.activated_abilities = vec![crate::card::ActivatedAbility {
        sac_cost: true,
        effect: Effect::DealDamage {
            amount: Value::ONE,
            to: crate::effect::shortcut::target_any(),
        },
        ..Default::default()
    }];
    def
}

/// Great Whale — {5}{U}{U} 5/5 Whale. ETB: untap up to seven lands
/// (modeled as your own — the printed "any lands" choice always picks yours).
pub fn great_whale() -> CardDefinition {
    CardDefinition {
        name: "Great Whale",
        cost: cost(&[generic(5), crate::mana::u(), crate::mana::u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Whale],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::Untap {
            what: crate::effect::Selector::EachPermanent(R::Land.and(R::ControlledByYou)),
            up_to: Some(Value::Const(7)),
        })],
        ..Default::default()
    }
}

/// Portent Tracker — {1}{G} 1/1 Satyr Scout. {T}: untap target land;
/// {T} (sorcery): tilt target battle's defense counters (CR 310.7).
pub fn portent_tracker() -> CardDefinition {
    CardDefinition {
        name: "Portent Tracker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Satyr, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![
            crate::card::ActivatedAbility {
                tap_cost: true,
                effect: Effect::Untap {
                    what: target_filtered(R::Land),
                    up_to: None,
                },
                ..Default::default()
            },
            crate::card::ActivatedAbility {
                tap_cost: true,
                sorcery_speed: true,
                effect: Effect::AdjustBattleDefense {
                    what: target_filtered(R::HasCardType(CardType::Battle)),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn urzas_legacy_manland(
    name: &'static str,
    color: crate::mana::Color,
    pt: (i32, i32),
    ctype: CreatureType,
    kws: Vec<Keyword>,
) -> CardDefinition {
    use crate::sets::{etb_tap, tap_add};
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            tap_add(color),
            crate::card::ActivatedAbility {
                mana_cost: cost(&[generic(1), crate::mana::ManaSymbol::Colored(color)]),
                effect: Effect::BecomeCreature {
                    what: crate::effect::Selector::This,
                    power: Value::Const(pt.0),
                    toughness: Value::Const(pt.1),
                    creature_types: vec![ctype],
                    keywords: kws,
                    duration: crate::effect::Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        triggered_abilities: vec![etb_tap()],
        ..Default::default()
    }
}

/// Ghitu Encampment — manland: {1}{R}: 2/1 first-strike Warrior EOT.
pub fn ghitu_encampment() -> CardDefinition {
    urzas_legacy_manland(
        "Ghitu Encampment",
        crate::mana::Color::Red,
        (2, 1),
        CreatureType::Warrior,
        vec![Keyword::FirstStrike],
    )
}

/// Forbidding Watchtower — manland: {1}{W}: 1/5 Soldier EOT.
pub fn forbidding_watchtower() -> CardDefinition {
    urzas_legacy_manland(
        "Forbidding Watchtower",
        crate::mana::Color::White,
        (1, 5),
        CreatureType::Soldier,
        vec![],
    )
}

/// Treetop Village — manland: {1}{G}: 3/3 trample Ape EOT.
pub fn treetop_village() -> CardDefinition {
    urzas_legacy_manland(
        "Treetop Village",
        crate::mana::Color::Green,
        (3, 3),
        CreatureType::Ape,
        vec![Keyword::Trample],
    )
}
