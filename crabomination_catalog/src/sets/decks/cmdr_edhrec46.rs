//! Commander: the 2026-10-13 rescan's gaps for Samut, Vizier of Naktamun;
//! The Walls of Ba Sing Se; and Sasaya, Orochi Ascendant. Tests in
//! `tests/recent_b/cmdr_edhrec46.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EquipBonus, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, PlayerRef, ZoneDest};
use crate::mana::{cost, g, generic, hybrid, r, Color, ManaCost};
use std::sync::Arc;

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

fn wall(name: &'static str) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender, Keyword::Reach],
        ..creature(name, cost(&[generic(2)]), vec![CreatureType::Wall], 0, 4)
    }
}

/// "Whenever [it or] another creature you control enters, that creature gets
/// +`p`/+`t` until end of turn."
fn entering_pump(scope: EventScope, p: i32, t: i32) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, scope)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature }),
        effect: Effect::PumpPT {
            what: Selector::TriggerSource,
            power: Value::Const(p),
            toughness: Value::Const(t),
            duration: Duration::EndOfTurn,
        },
    }
}

fn token(name: &str, color: Color, types: Vec<CreatureType>, p: i32, t: i32, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// "Untap target land you control. It becomes a `p`/`p` Elemental creature
/// with `keywords` … It's still a land."
fn awaken_land(p: i32, keywords: Vec<Keyword>, duration: Duration) -> Effect {
    Effect::Seq(vec![
        Effect::Untap { what: target_filtered(R::Land.and(R::ControlledByYou)), up_to: None },
        Effect::BecomeCreature {
            what: Selector::Target(0),
            power: Value::Const(p),
            toughness: Value::Const(p),
            creature_types: vec![CreatureType::Elemental],
            keywords,
            duration,
        },
    ])
}

// ── Samut, Vizier of Naktamun ────────────────────────────────────────────────

/// Ardoz, Cobbler of War — {1}{R} legendary 1/1 Goblin Shaman, haste. It or
/// another creature of yours entering gets +2/+0 this turn. {3}{R}, sorcery
/// speed: a 1/1 red Goblin with haste.
pub fn ardoz_cobbler_of_war() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![entering_pump(EventScope::YourControl, 2, 0)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), r()]),
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: Arc::new(token("Goblin", Color::Red, vec![CreatureType::Goblin], 1, 1, vec![Keyword::Haste])),
            },
            ..Default::default()
        }],
        ..creature("Ardoz, Cobbler of War", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Shaman], 1, 1)
    }
}

/// Primal Forcemage — {2}{G} 2/2 Elf Shaman. Another creature of yours
/// entering gets +3/+3 this turn.
pub fn primal_forcemage() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![entering_pump(EventScope::AnotherOfYours, 3, 3)],
        ..creature("Primal Forcemage", cost(&[generic(2), g()]), vec![CreatureType::Elf, CreatureType::Shaman], 2, 2)
    }
}

/// Stampede Surfer — {3}{R/G}{R/G} 4/4 Human Warrior, haste. Attacking: a 2/2
/// green Boar tapped and attacking each opponent.
pub fn stampede_surfer() -> CardDefinition {
    let rg = || hybrid(Color::Red, Color::Green);
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ForEachOpponent {
                body: Box::new(Effect::CreateTokenAttacking {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(token("Boar", Color::Green, vec![CreatureType::Boar], 2, 2, vec![])),
                    cleanup: Default::default(),
                    defender: Some(PlayerRef::Triggerer),
                }),
            },
        }],
        ..creature(
            "Stampede Surfer",
            cost(&[generic(3), rg(), rg()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            4,
            4,
        )
    }
}

// ── The Walls of Ba Sing Se ──────────────────────────────────────────────────

/// Warmonger's Chariot — {2} Equipment, equip {3}. Equipped creature gets
/// +2/+2 and can attack as though it didn't have defender.
pub fn warmongers_chariot() -> CardDefinition {
    CardDefinition {
        name: "Warmonger's Chariot",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::AttacksAsThoughNoDefender],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Consulate Skygate — {2} 0/4 artifact creature Wall, defender, reach.
pub fn consulate_skygate() -> CardDefinition {
    wall("Consulate Skygate")
}

/// Towering Viewpoint — {2} 0/4 artifact creature Wall, defender, reach.
/// Leap of Faith — {3}: target creature gains flying this turn.
pub fn towering_viewpoint() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..wall("Towering Viewpoint")
    }
}

// ── Sasaya, Orochi Ascendant ─────────────────────────────────────────────────

/// Silvanus's Invoker — {2}{G} 3/2 Dragon Druid. Conjure Elemental — {8}: untap
/// a land of yours; it's an 8/8 Elemental creature with trample and haste this
/// turn, still a land.
pub fn silvanuss_invoker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(8)]),
            effect: awaken_land(8, vec![Keyword::Trample, Keyword::Haste], Duration::EndOfTurn),
            ..Default::default()
        }],
        ..creature("Silvanus's Invoker", cost(&[generic(2), g()]), vec![CreatureType::Dragon, CreatureType::Druid], 3, 2)
    }
}

/// Wakeroot Elemental — {4}{G}{G} 5/5 Elemental. {G}{G}{G}{G}{G}: untap a land
/// of yours; it's a 5/5 Elemental creature with haste for as long as it
/// remains on the battlefield, still a land.
pub fn wakeroot_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), g(), g(), g(), g()]),
            effect: awaken_land(5, vec![Keyword::Haste], Duration::Permanent),
            ..Default::default()
        }],
        ..creature("Wakeroot Elemental", cost(&[generic(4), g(), g()]), vec![CreatureType::Elemental], 5, 5)
    }
}

/// You Happen On a Glade — {2}{G} instant. Choose one — up to two basic land
/// cards to hand; or a permanent card from your graveyard to hand.
pub fn you_happen_on_a_glade() -> CardDefinition {
    CardDefinition {
        name: "You Happen On a Glade",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(2),
            },
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}
