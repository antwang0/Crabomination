//! March of the Machine **Battle — Siege** cards (CR 310). Each enters with
//! defense counters and a chosen protector; when defeated it's exiled and its
//! transformed back face enters. Tests in `crabomination/src/tests/mom.rs`.

use crate::card::{
    ActivatedAbility, BattleSubtype, CardDefinition, CardType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target, target_filtered};
use crate::effect::{Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

/// Shared Siege subtypes block.
fn siege() -> Subtypes {
    Subtypes { battle_subtypes: vec![BattleSubtype::Siege], ..Default::default() }
}

/// Invasion of Zendikar // Awakened Skyclave — {3}{G} Siege, defense 3. ETB:
/// search up to two basic lands onto the battlefield tapped. Back: a 4/4
/// Elemental with vigilance, haste, and a {T}: any-color mana ability (the
/// "it's also a land" rider is omitted).
pub fn invasion_of_zendikar() -> CardDefinition {
    let skyclave = CardDefinition {
        name: "Awakened Skyclave",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Zendikar",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Battle],
        subtypes: siege(),
        defense: 3,
        triggered_abilities: vec![etb(Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::Const(2),
        })],
        back_face: Some(Box::new(skyclave)),
        ..Default::default()
    }
}

/// Invasion of Kaladesh // Aetherwing, Golden-Scale Flagship — {U}{R} Siege,
/// defense 4. ETB: make a 1/1 flying Thopter. Back: a Legendary Artifact 4/4
/// flier (modeled as a flying artifact creature; Crew / power-counts-artifacts
/// are omitted).
pub fn invasion_of_kaladesh() -> CardDefinition {
    let thopter = crate::card::TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    let aetherwing = CardDefinition {
        name: "Aetherwing, Golden-Scale Flagship",
        card_types: vec![CardType::Artifact, CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        color_indicator: vec![Color::Blue, Color::Red],
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Kaladesh",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Battle],
        subtypes: siege(),
        defense: 4,
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: thopter,
        })],
        back_face: Some(Box::new(aetherwing)),
        ..Default::default()
    }
}

/// Invasion of Amonkhet // Lazotep Convert — {1}{U}{B} Siege, defense 4. ETB:
/// each player mills three, then each opponent discards a card and you draw.
/// Back: a 4/4 black Zombie (the enter-as-a-copy clause is omitted).
pub fn invasion_of_amonkhet() -> CardDefinition {
    let convert = CardDefinition {
        name: "Lazotep Convert",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 4,
        toughness: 4,
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Amonkhet",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Battle],
        subtypes: siege(),
        defense: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: false,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        back_face: Some(Box::new(convert)),
        ..Default::default()
    }
}

/// Invasion of Ravnica // Guildpact Paragon — {5} Siege, defense 4. ETB: exile
/// target nonland permanent an opponent controls (the "isn't exactly two
/// colors" restriction is omitted). Back: a 5/5 artifact Construct (its
/// two-color cast trigger is omitted).
pub fn invasion_of_ravnica() -> CardDefinition {
    let paragon = CardDefinition {
        name: "Guildpact Paragon",
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 5,
        toughness: 5,
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Ravnica",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Battle],
        subtypes: siege(),
        defense: 4,
        triggered_abilities: vec![etb(Effect::Exile {
            what: target_filtered(
                SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
            ),
        })],
        back_face: Some(Box::new(paragon)),
        ..Default::default()
    }
}

/// Invasion of Theros // Ephara, Ever-Sheltering — {2}{W} Siege, defense 4.
/// ETB: search your library for a God card to hand (the Aura/Demigod options
/// are omitted). Back: a 4/4 God that draws when another enchantment you
/// control enters.
pub fn invasion_of_theros() -> CardDefinition {
    let ephara = CardDefinition {
        name: "Ephara, Ever-Sheltering",
        card_types: vec![CardType::Enchantment, CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        color_indicator: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::God], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Enchantment
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Theros",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Battle],
        subtypes: siege(),
        defense: 4,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCreatureType(CreatureType::God),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        back_face: Some(Box::new(ephara)),
        ..Default::default()
    }
}

/// Invasion of Tarkir // Defiant Thundermaw — {1}{R} Siege, defense 5. ETB:
/// deal 2 damage to any target (the "reveal Dragons for +X" rider is collapsed
/// to the X=0 base). Back: a 4/4 Dragon with flying and trample.
pub fn invasion_of_tarkir() -> CardDefinition {
    let thundermaw = CardDefinition {
        name: "Defiant Thundermaw",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dragon], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        ..Default::default()
    };
    CardDefinition {
        name: "Invasion of Tarkir",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Battle],
        subtypes: siege(),
        defense: 5,
        triggered_abilities: vec![etb(Effect::DealDamage {
            to: target(),
            amount: Value::Const(2),
        })],
        back_face: Some(Box::new(thundermaw)),
        ..Default::default()
    }
}
