//! Mercadian Masques (MMQ) gap closure, first wave. Tests in `classic_sets/mmq`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EquipScale, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{etb, target, target_any, target_filtered},
};
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

use super::tap_add;

// ── Shared shapes ───────────────────────────────────────────────────────────

fn creature(
    name: &'static str,
    c: crate::mana::ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn artifact(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: crate::mana::ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn instant(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: crate::mana::ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

/// An Aura that enchants a creature for `bonus`.
fn creature_aura(name: &'static str, c: crate::mana::ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// "Whenever this creature becomes blocked, [effect]" (CR 509.1h).
fn on_blocked(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
        effect,
    }
}

/// The Rebel / Mercenary tutor line: `{N}, {T}: Search your library for a
/// [tribe] permanent card with mana value `mv` or less, put it onto the
/// battlefield, then shuffle.`
fn tutor_chain(generic_cost: u32, tribe: CreatureType, mv: u32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(generic_cost)]),
        tap_cost: true,
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: R::PermanentCard.and(R::HasCreatureType(tribe)).and(R::ManaValueAtMost(mv)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}

/// A Spellshaper line: `{cost}, {T}, Discard a card: [effect]`.
fn spellshaper(c: crate::mana::ManaCost, effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        tap_cost: true,
        discard_cost: Some((R::Any, 1)),
        effect,
        ..Default::default()
    }
}

fn regenerate_for(c: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::Regenerate { what: Selector::This },
        ..Default::default()
    }
}

// ── Rebels (white) ──────────────────────────────────────────────────────────

/// Ramosian Sergeant — {W} 1/1, the bottom of the Rebel tutor chain.
pub fn ramosian_sergeant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tutor_chain(3, CreatureType::Rebel, 2)],
        ..creature(
            "Ramosian Sergeant",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            1,
            1,
        )
    }
}

/// Ramosian Lieutenant — {1}{W} 1/2; fetches Rebels with mana value 3 or less.
pub fn ramosian_lieutenant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tutor_chain(4, CreatureType::Rebel, 3)],
        ..creature(
            "Ramosian Lieutenant",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            1,
            2,
        )
    }
}

/// Ramosian Captain — {1}{W}{W} 2/2 first striker; fetches Rebels up to 4.
pub fn ramosian_captain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        activated_abilities: vec![tutor_chain(5, CreatureType::Rebel, 4)],
        ..creature(
            "Ramosian Captain",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Ramosian Commander — {2}{W}{W} 2/4; fetches Rebels up to 5.
pub fn ramosian_commander() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tutor_chain(6, CreatureType::Rebel, 5)],
        ..creature(
            "Ramosian Commander",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            4,
        )
    }
}

/// Ramosian Sky Marshal — {3}{W}{W} 3/3 flier; the top of the Rebel chain.
pub fn ramosian_sky_marshal() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![tutor_chain(7, CreatureType::Rebel, 6)],
        ..creature(
            "Ramosian Sky Marshal",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            3,
            3,
        )
    }
}

/// Fresh Volunteers — {1}{W} 2/2 vanilla Rebel.
pub fn fresh_volunteers() -> CardDefinition {
    creature(
        "Fresh Volunteers",
        cost(&[generic(1), w()]),
        vec![CreatureType::Human, CreatureType::Rebel],
        2,
        2,
    )
}

/// Jhovall Queen — {4}{W}{W} 4/7 vigilant Cat Rebel.
pub fn jhovall_queen() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature(
            "Jhovall Queen",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Cat, CreatureType::Rebel],
            4,
            7,
        )
    }
}

/// Jhovall Rider — {4}{W} 3/3 trampling Rebel.
pub fn jhovall_rider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Jhovall Rider",
            cost(&[generic(4), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            3,
            3,
        )
    }
}

/// Nightwind Glider — {2}{W} 2/1 flier with protection from black.
pub fn nightwind_glider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Black)],
        ..creature(
            "Nightwind Glider",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            1,
        )
    }
}

/// Thermal Glider — {2}{W} 2/1 flier with protection from red.
pub fn thermal_glider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature(
            "Thermal Glider",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            1,
        )
    }
}

/// Steadfast Guard — {W}{W} 2/2 vigilant Rebel.
pub fn steadfast_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        ..creature(
            "Steadfast Guard",
            cost(&[w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            2,
            2,
        )
    }
}

/// Task Force — {2}{W} 1/3 that hardens when it's targeted.
pub fn task_force() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Task Force",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Rebel],
            1,
            3,
        )
    }
}

/// Rappelling Scouts — {2}{W}{W} 1/4 flier that can dodge a colour each turn.
pub fn rappelling_scouts() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::This,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Rappelling Scouts",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Rebel, CreatureType::Scout],
            1,
            4,
        )
    }
}

// ── Mercenaries (black) ─────────────────────────────────────────────────────

/// Cateran Persuader — {B}{B} 2/1, the bottom of the Mercenary tutor chain.
pub fn cateran_persuader() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tutor_chain(1, CreatureType::Mercenary, 1)],
        ..creature(
            "Cateran Persuader",
            cost(&[b(), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            1,
        )
    }
}

/// Cateran Brute — {2}{B} 2/2; fetches Mercenaries with mana value 2 or less.
pub fn cateran_brute() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tutor_chain(2, CreatureType::Mercenary, 2)],
        ..creature(
            "Cateran Brute",
            cost(&[generic(2), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Cateran Kidnappers — {2}{B}{B} 4/2; fetches Mercenaries up to 3.
pub fn cateran_kidnappers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![tutor_chain(3, CreatureType::Mercenary, 3)],
        ..creature(
            "Cateran Kidnappers",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            4,
            2,
        )
    }
}

/// Cateran Enforcer — {3}{B}{B} 4/3 with fear; fetches Mercenaries up to 4.
pub fn cateran_enforcer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        activated_abilities: vec![tutor_chain(4, CreatureType::Mercenary, 4)],
        ..creature(
            "Cateran Enforcer",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            4,
            3,
        )
    }
}

/// Cateran Slaver — {4}{B}{B} 5/5 swampwalker; fetches Mercenaries up to 5.
pub fn cateran_slaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![tutor_chain(5, CreatureType::Mercenary, 5)],
        ..creature(
            "Cateran Slaver",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            5,
            5,
        )
    }
}

/// Cateran Overlord — {4}{B}{B}{B} 7/5 that eats creatures to regenerate.
pub fn cateran_overlord() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            },
            tutor_chain(6, CreatureType::Mercenary, 6),
        ],
        ..creature(
            "Cateran Overlord",
            cost(&[generic(4), b(), b(), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            7,
            5,
        )
    }
}

/// Cateran Summons — {B}. Tutor any Mercenary card to hand.
pub fn cateran_summons() -> CardDefinition {
    sorcery(
        "Cateran Summons",
        cost(&[b()]),
        Effect::Search {
            who: PlayerRef::You,
            filter: R::HasCreatureType(CreatureType::Mercenary),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Misshapen Fiend — {1}{B} 1/1 flying Mercenary.
pub fn misshapen_fiend() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature(
            "Misshapen Fiend",
            cost(&[generic(1), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            1,
            1,
        )
    }
}

/// Molting Harpy — {B} 2/1 flier that has to be fed {2} each upkeep.
pub fn molting_harpy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[generic(2)]) },
        }],
        ..creature(
            "Molting Harpy",
            cost(&[b()]),
            vec![CreatureType::Harpy, CreatureType::Mercenary],
            2,
            1,
        )
    }
}

/// Rampart Crawler — {B} 1/1 that slips past Walls.
pub fn rampart_crawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(R::HasCreatureType(CreatureType::Wall)))],
        ..creature(
            "Rampart Crawler",
            cost(&[b()]),
            vec![CreatureType::Lizard, CreatureType::Mercenary],
            1,
            1,
        )
    }
}

/// Primeval Shambler — {4}{B} 3/3 with a black-mana pump.
pub fn primeval_shambler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Primeval Shambler",
            cost(&[generic(4), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            3,
            3,
        )
    }
}

/// Bog Smugglers — {1}{B}{B} 2/2 swampwalker.
pub fn bog_smugglers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..creature(
            "Bog Smugglers",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Strongarm Thug — {2}{B} 1/1 that buys back a Mercenary on arrival.
pub fn strongarm_thug() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Return target Mercenary card from your graveyard to your hand".into(),
            body: Box::new(Effect::Move {
                what: target_filtered(
                    R::InYourGraveyard.and(R::HasCreatureType(CreatureType::Mercenary)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            }),
        })],
        ..creature(
            "Strongarm Thug",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            1,
            1,
        )
    }
}

/// Skulking Fugitive — {2}{B} 3/4 that dies the moment anything points at it.
pub fn skulking_fugitive() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..creature(
            "Skulking Fugitive",
            cost(&[generic(2), b()]),
            vec![CreatureType::Horror, CreatureType::Mercenary],
            3,
            4,
        )
    }
}

// ── Spellshapers ────────────────────────────────────────────────────────────

/// Bog Witch — {2}{B} 1/1. Discard a card for {B}{B}{B}.
pub fn bog_witch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[b()]),
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Black, Color::Black, Color::Black]),
            },
        )],
        ..creature(
            "Bog Witch",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Blaster Mage — {2}{R} 2/2. Discard a card to blow up a Wall.
pub fn blaster_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[r()]),
            Effect::Destroy { what: target_filtered(R::HasCreatureType(CreatureType::Wall)) },
        )],
        ..creature(
            "Blaster Mage",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            2,
            2,
        )
    }
}

/// Balloon Peddler — {2}{U} 2/2. Discard a card to give a creature flying.
pub fn balloon_peddler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[u()]),
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Balloon Peddler",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            2,
            2,
        )
    }
}

/// Deepwood Drummer — {1}{G} 1/1. Discard a card for +2/+2.
pub fn deepwood_drummer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[g()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        )],
        ..creature(
            "Deepwood Drummer",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Rushwood Herbalist — {2}{G} 2/2. Discard a card to regenerate a creature.
pub fn rushwood_herbalist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[g()]),
            Effect::Regenerate { what: target_filtered(R::Creature) },
        )],
        ..creature(
            "Rushwood Herbalist",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            2,
            2,
        )
    }
}

/// Devout Witness — {2}{W} 2/2. Discard a card for Disenchant.
pub fn devout_witness() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[generic(1), w()]),
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        )],
        ..creature(
            "Devout Witness",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            2,
            2,
        )
    }
}

/// Cackling Witch — {1}{B} 1/1. Discard a card for a scalable +X/+0.
pub fn cackling_witch() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), b()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::XFromCost,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Cackling Witch",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Notorious Assassin — {3}{B} 2/2. Discard a card for a hard kill.
pub fn notorious_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[generic(2), b()]),
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
        )],
        ..creature(
            "Notorious Assassin",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Spellshaper, CreatureType::Assassin],
            2,
            2,
        )
    }
}

/// Undertaker — {1}{B} 1/1. Discard a card to buy back a creature.
pub fn undertaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[b()]),
            Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::Creature)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )],
        ..creature(
            "Undertaker",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Waterfront Bouncer — {1}{U} 1/1. Discard a card to bounce a creature.
pub fn waterfront_bouncer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[u()]),
            Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )],
        ..creature(
            "Waterfront Bouncer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Tonic Peddler — {1}{W} 1/1. Discard a card for 3 life.
pub fn tonic_peddler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[w()]),
            Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(3) },
        )],
        ..creature(
            "Tonic Peddler",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Silverglade Pathfinder — {1}{G} 1/1. Discard a card to ramp a basic.
pub fn silverglade_pathfinder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[generic(1), g()]),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        )],
        ..creature(
            "Silverglade Pathfinder",
            cost(&[generic(1), g()]),
            vec![CreatureType::Dryad, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Dawnstrider — {1}{G} 1/1. Discard a card to fog.
pub fn dawnstrider() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(cost(&[g()]), Effect::PreventAllCombatDamageThisTurn)],
        ..creature(
            "Dawnstrider",
            cost(&[generic(1), g()]),
            vec![CreatureType::Dryad, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Hammer Mage — {1}{R} 1/1. Discard a card to sweep small artifacts.
pub fn hammer_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), r()]),
            tap_cost: true,
            discard_cost: Some((R::Any, 1)),
            effect: Effect::DestroyEachMatchingWithManaValue {
                filter: R::Artifact,
                value: Value::XFromCost,
            },
            ..Default::default()
        }],
        ..creature(
            "Hammer Mage",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

/// Overtaker — {1}{U} 1/1. Discard a card to Threaten a creature.
pub fn overtaker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![spellshaper(
            cost(&[generic(3), u()]),
            Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature), up_to: None },
                Effect::GainControl {
                    what: target(),
                    to: Some(PlayerRef::You),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeyword {
                    what: target(),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )],
        ..creature(
            "Overtaker",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Spellshaper],
            1,
            1,
        )
    }
}

// ── "Whenever this becomes blocked" ─────────────────────────────────────────

/// Sacred Prey — {G} 1/1 that pays 1 life for being chumped.
pub fn sacred_prey() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..creature("Sacred Prey", cost(&[g()]), vec![CreatureType::Horse], 1, 1)
    }
}

/// Deepwood Wolverine — {G} 1/1 that hits back harder when blocked.
pub fn deepwood_wolverine() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        ..creature("Deepwood Wolverine", cost(&[g()]), vec![CreatureType::Wolverine], 1, 1)
    }
}

/// Snorting Gahr — {2}{G}{G} 3/3 that grows when blocked.
pub fn snorting_gahr() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Snorting Gahr",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Rhino, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Deepwood Tantiv — {4}{G} 2/4 that gains 2 life when blocked.
pub fn deepwood_tantiv() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..creature("Deepwood Tantiv", cost(&[generic(4), g()]), vec![CreatureType::Beast], 2, 4)
    }
}

/// Chambered Nautilus — {2}{U} 2/2 that cantrips when blocked.
pub fn chambered_nautilus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::MayDo {
            description: "Draw a card".into(),
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        })],
        ..creature(
            "Chambered Nautilus",
            cost(&[generic(2), u()]),
            vec![CreatureType::Nautilus, CreatureType::Beast],
            2,
            2,
        )
    }
}

/// Saprazzan Heir — {1}{U} 1/1 that draws three cards when blocked.
pub fn saprazzan_heir() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::MayDo {
            description: "Draw three cards".into(),
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
        })],
        ..creature("Saprazzan Heir", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Alley Grifters — {1}{B}{B} 2/2 that strips a card when blocked.
pub fn alley_grifters() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::Discard {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::Const(1),
            random: false,
        })],
        ..creature(
            "Alley Grifters",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary],
            2,
            2,
        )
    }
}

/// Corrupt Official — {4}{B} 3/1 regenerator that mugs its blocker's controller.
pub fn corrupt_official() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![regenerate_for(cost(&[generic(2), b()]))],
        triggered_abilities: vec![on_blocked(Effect::Discard {
            who: Selector::Player(PlayerRef::DefendingPlayer),
            amount: Value::Const(1),
            random: true,
        })],
        ..creature(
            "Corrupt Official",
            cost(&[generic(4), b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            3,
            1,
        )
    }
}

/// Saprazzan Raider — {2}{U} 1/2 that bails out of a bad block.
pub fn saprazzan_raider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
        })],
        ..creature("Saprazzan Raider", cost(&[generic(2), u()]), vec![CreatureType::Merfolk], 1, 2)
    }
}

/// Ignoble Soldier — {2}{W} 3/1 that goes limp once it's blocked.
pub fn ignoble_soldier() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_blocked(Effect::PreventCombatDamageByTargetThisTurn {
            target: Selector::This,
        })],
        ..creature(
            "Ignoble Soldier",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            3,
            1,
        )
    }
}

// ── Auras ───────────────────────────────────────────────────────────────────

/// Muzzle — {1}{W}. The enchanted creature's damage is blanked.
pub fn muzzle() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt by enchanted creature.",
            effect: StaticEffect::PreventAllDamageByEnchanted,
        }],
        ..creature_aura("Muzzle", cost(&[generic(1), w()]), EquipBonus::default())
    }
}

/// Inviolability — {1}{W}. The enchanted creature can't be damaged.
pub fn inviolability() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Prevent all damage that would be dealt to enchanted creature.",
            effect: StaticEffect::PreventAllDamageToEnchanted,
        }],
        ..creature_aura("Inviolability", cost(&[generic(1), w()]), EquipBonus::default())
    }
}

/// Maggot Therapy — {2}{B} flash Aura for +2/-2.
pub fn maggot_therapy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature_aura(
            "Maggot Therapy",
            cost(&[generic(2), b()]),
            EquipBonus { power: 2, toughness: -2, ..Default::default() },
        )
    }
}

/// Flaming Sword — {1}{R} flash Aura for +1/+0 and first strike.
pub fn flaming_sword() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature_aura(
            "Flaming Sword",
            cost(&[generic(1), r()]),
            EquipBonus { power: 1, keywords: vec![Keyword::FirstStrike], ..Default::default() },
        )
    }
}

/// Buoyancy — {1}{U} flash Aura granting flying.
pub fn buoyancy() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature_aura(
            "Buoyancy",
            cost(&[generic(1), u()]),
            EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
        )
    }
}

/// Tiger Claws — {2}{G} flash Aura for +1/+1 and trample.
pub fn tiger_claws() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature_aura(
            "Tiger Claws",
            cost(&[generic(2), g()]),
            EquipBonus {
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Trample],
                ..Default::default()
            },
        )
    }
}

/// Cave Sense — {1}{R} Aura for +1/+1 and mountainwalk.
pub fn cave_sense() -> CardDefinition {
    creature_aura(
        "Cave Sense",
        cost(&[generic(1), r()]),
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Landwalk(LandType::Mountain)],
            ..Default::default()
        },
    )
}

/// Diplomatic Immunity — {1}{U}. Both the Aura and its host have shroud.
pub fn diplomatic_immunity() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud],
        ..creature_aura(
            "Diplomatic Immunity",
            cost(&[generic(1), u()]),
            EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() },
        )
    }
}

/// Ancestral Mask — {2}{G}. +2/+2 per *other* enchantment anywhere.
pub fn ancestral_mask() -> CardDefinition {
    creature_aura(
        "Ancestral Mask",
        cost(&[generic(2), g()]),
        EquipBonus {
            scale: Some(EquipScale {
                filter: R::Enchantment,
                per_power: 2,
                per_toughness: 2,
                count_all_controllers: true,
                exclude_source: true,
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Soul Channeling — {2}{B}. Pay 2 life to regenerate the host.
pub fn soul_channeling() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::Regenerate { what: Selector::AttachedToMe(Box::new(Selector::This)) },
            ..Default::default()
        }],
        ..creature_aura("Soul Channeling", cost(&[generic(2), b()]), EquipBonus::default())
    }
}

/// Stamina — {2}{G}. Vigilance, and the Aura can eat itself to regenerate.
pub fn stamina() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Regenerate { what: Selector::AttachedToMe(Box::new(Selector::This)) },
            ..Default::default()
        }],
        ..creature_aura(
            "Stamina",
            cost(&[generic(2), g()]),
            EquipBonus { keywords: vec![Keyword::Vigilance], ..Default::default() },
        )
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Moonlit Wake — {2}{W}. A life point per death, anywhere.
pub fn moonlit_wake() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..enchantment("Moonlit Wake", cost(&[generic(2), w()]))
    }
}

/// Intimidation — {2}{B}{B}{B}. Your team gains fear.
pub fn intimidation() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have fear.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Fear,
            },
        }],
        ..enchantment("Intimidation", cost(&[generic(2), b(), b(), b()]))
    }
}

/// Magistrate's Veto — {2}{R}. White and blue creatures can't block.
pub fn magistrates_veto() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "White creatures and blue creatures can't block.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::White).or(R::HasColor(Color::Blue))),
                ),
                keyword: Keyword::CantBlock,
            },
        }],
        ..enchantment("Magistrate's Veto", cost(&[generic(2), r()]))
    }
}

/// Lumbering Satyr — {2}{G}{G} 5/4 that hands *everyone* forestwalk.
pub fn lumbering_satyr() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All creatures have forestwalk.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Landwalk(LandType::Forest),
            },
        }],
        ..creature(
            "Lumbering Satyr",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Satyr, CreatureType::Beast],
            5,
            4,
        )
    }
}

/// Larceny — {3}{B}{B}. Connecting in combat strips a card.
pub fn larceny() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..enchantment("Larceny", cost(&[generic(3), b(), b()]))
    }
}

/// Squeeze — {3}{U}. Sorceries cost {3} more.
pub fn squeeze() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Sorcery spells cost {3} more to cast.",
            effect: StaticEffect::AdditionalCost {
                filter: R::HasCardType(CardType::Sorcery),
                amount: 3,
            },
        }],
        ..enchantment("Squeeze", cost(&[generic(3), u()]))
    }
}

/// High Seas — {2}{U}. Red and green creature spells cost {1} more.
pub fn high_seas() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Red creature spells and green creature spells cost {1} more to cast.",
            effect: StaticEffect::AdditionalCost {
                filter: R::Creature.and(R::HasColor(Color::Red).or(R::HasColor(Color::Green))),
                amount: 1,
            },
        }],
        ..enchantment("High Seas", cost(&[generic(2), u()]))
    }
}

/// Ivory Mask — {2}{W}{W}. You have shroud.
pub fn ivory_mask() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have shroud.",
            effect: StaticEffect::ControllerHasShroud,
        }],
        ..enchantment("Ivory Mask", cost(&[generic(2), w(), w()]))
    }
}

// ── Creatures (misc) ────────────────────────────────────────────────────────

/// Lightning Hounds — {2}{R}{R} 3/2 first striker.
pub fn lightning_hounds() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature("Lightning Hounds", cost(&[generic(2), r(), r()]), vec![CreatureType::Dog], 3, 2)
    }
}

/// Gerrard's Irregulars — {4}{R} 4/2 with trample and haste.
pub fn gerrards_irregulars() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Haste],
        ..creature(
            "Gerrard's Irregulars",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            2,
        )
    }
}

/// Wild Jhovall — {3}{R} 3/3 vanilla Cat.
pub fn wild_jhovall() -> CardDefinition {
    creature("Wild Jhovall", cost(&[generic(3), r()]), vec![CreatureType::Cat], 3, 3)
}

/// Horned Troll — {2}{G} 2/2 regenerator.
pub fn horned_troll() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![regenerate_for(cost(&[g()]))],
        ..creature("Horned Troll", cost(&[generic(2), g()]), vec![CreatureType::Troll], 2, 2)
    }
}

/// Deepwood Ghoul — {2}{B} 2/1 that regenerates for life.
pub fn deepwood_ghoul() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            life_cost: 2,
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Deepwood Ghoul", cost(&[generic(2), b()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Rock Badger — {4}{R} 3/3 mountainwalker.
pub fn rock_badger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Mountain)],
        ..creature(
            "Rock Badger",
            cost(&[generic(4), r()]),
            vec![CreatureType::Badger, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Kyren Glider — {1}{R} 1/1 flier that can't block.
pub fn kyren_glider() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        ..creature("Kyren Glider", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Tidal Kraken — {5}{U}{U}{U} 6/6 unblockable.
pub fn tidal_kraken() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        ..creature(
            "Tidal Kraken",
            cost(&[generic(5), u(), u(), u()]),
            vec![CreatureType::Kraken],
            6,
            6,
        )
    }
}

/// Darting Merfolk — {1}{U} 1/1 that ducks removal for {U}.
pub fn darting_merfolk() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
        },
            ..Default::default()
        }],
        ..creature("Darting Merfolk", cost(&[generic(1), u()]), vec![CreatureType::Merfolk], 1, 1)
    }
}

/// Boa Constrictor — {4}{G} 3/3 that taps for a big swing.
pub fn boa_constrictor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Boa Constrictor", cost(&[generic(4), g()]), vec![CreatureType::Snake], 3, 3)
    }
}

/// Crossbow Infantry — {1}{W} 1/1 that shoots into combat.
pub fn crossbow_infantry() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..creature(
            "Crossbow Infantry",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Alabaster Wall — {2}{W} 0/4 Wall that shaves a point of damage.
pub fn alabaster_wall() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..creature("Alabaster Wall", cost(&[generic(2), w()]), vec![CreatureType::Wall], 0, 4)
    }
}

/// Battle Rampart — {2}{R} 1/3 Wall that hands out haste.
pub fn battle_rampart() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Battle Rampart", cost(&[generic(2), r()]), vec![CreatureType::Wall], 1, 3)
    }
}

/// Stinging Barrier — {2}{U}{U} 0/4 Wall that pings.
pub fn stinging_barrier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..creature(
            "Stinging Barrier",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Wall],
            0,
            4,
        )
    }
}

/// Crenellated Wall — {4} 0/4 artifact Wall that lends toughness.
pub fn crenellated_wall() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(0),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Crenellated Wall", cost(&[generic(4)]), vec![CreatureType::Wall], 0, 4)
    }
}

/// Shock Troops — {3}{R} 2/2 that trades itself for 2 damage.
pub fn shock_troops() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..creature(
            "Shock Troops",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Thrashing Wumpus — {3}{B}{B} 3/3 repeatable sweeper.
pub fn thrashing_wumpus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::Const(1),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(1),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Thrashing Wumpus", cost(&[generic(3), b(), b()]), vec![CreatureType::Beast], 3, 3)
    }
}

/// Hunted Wumpus — {3}{G} 6/6 that lets everyone else cheat a creature in.
pub fn hunted_wumpus() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::EachPlayerMayPutPermanentFromHand {
            filter: R::Creature,
            others_only: true,
            repeat: false,
        })],
        ..creature("Hunted Wumpus", cost(&[generic(3), g()]), vec![CreatureType::Beast], 6, 6)
    }
}

/// Charmed Griffin — {3}{W} 3/3 flier; everyone else gets a free permanent.
pub fn charmed_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::EachPlayerMayPutPermanentFromHand {
            filter: R::Artifact.or(R::Enchantment),
            others_only: true,
            repeat: false,
        })],
        ..creature("Charmed Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 3, 3)
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Soothing Balm — {1}{W}. Target player gains 5 life.
pub fn soothing_balm() -> CardDefinition {
    instant(
        "Soothing Balm",
        cost(&[generic(1), w()]),
        Effect::GainLife { who: target_filtered(R::Player), amount: Value::Const(5) },
    )
}

/// Specter's Wail — {1}{B}. Target player discards at random.
pub fn specters_wail() -> CardDefinition {
    sorcery(
        "Specter's Wail",
        cost(&[generic(1), b()]),
        Effect::Discard { who: target_filtered(R::Player), amount: Value::Const(1), random: true },
    )
}

/// Sever Soul — {3}{B}{B}. Hard kill that pays back the creature's toughness.
pub fn sever_soul() -> CardDefinition {
    sorcery(
        "Sever Soul",
        cost(&[generic(3), b(), b()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(target())),
            },
        ]),
    )
}

/// Ghoul's Feast — {1}{B}. +X/+0 for each creature card in your graveyard.
pub fn ghouls_feast() -> CardDefinition {
    instant(
        "Ghoul's Feast",
        cost(&[generic(1), b()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::CardsInGraveyardMatching {
                who: PlayerRef::You,
                filter: R::Creature,
            },
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        },
    )
}

/// Hoodwink — {1}{U}. Bounce a noncreature permanent.
pub fn hoodwink() -> CardDefinition {
    instant(
        "Hoodwink",
        cost(&[generic(1), u()]),
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Enchantment).or(R::Land)),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        },
    )
}

/// Last Breath — {1}{W}. Exile a small creature; its controller gains 4.
pub fn last_breath() -> CardDefinition {
    instant(
        "Last Breath",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(target()))),
                amount: Value::Const(4),
            },
            Effect::Exile { what: target_filtered(R::Creature.and(R::PowerAtMost(2))) },
        ]),
    )
}

/// Revive — {1}{G}. Return a green card from your graveyard.
pub fn revive() -> CardDefinition {
    sorcery(
        "Revive",
        cost(&[generic(1), g()]),
        Effect::Move {
            what: target_filtered(R::InYourGraveyard.and(R::HasColor(Color::Green))),
            to: ZoneDest::Hand(PlayerRef::You),
        },
    )
}

/// Collective Unconscious — {4}{G}{G}. Draw a card per creature you control.
pub fn collective_unconscious() -> CardDefinition {
    sorcery(
        "Collective Unconscious",
        cost(&[generic(4), g(), g()]),
        Effect::Draw {
            who: Selector::You,
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::Creature.and(R::ControlledByYou))),
                filter: R::Any,
            },
        },
    )
}

/// Forced March — {X}{B}{B}{B}. Sweep everything with mana value X or less.
pub fn forced_march() -> CardDefinition {
    sorcery(
        "Forced March",
        cost(&[x(), b(), b(), b()]),
        Effect::DestroyEachCreatureWithManaValue { value: Value::XFromCost },
    )
}

/// Lunge — {2}{R}. Two damage to a creature and two to a player.
pub fn lunge() -> CardDefinition {
    instant(
        "Lunge",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::TargetFiltered { slot: 1, filter: R::Player },
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Spontaneous Generation — {3}{G}. A Saproling per card in hand.
pub fn spontaneous_generation() -> CardDefinition {
    sorcery(
        "Spontaneous Generation",
        cost(&[generic(3), g()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::HandSizeOf(PlayerRef::You),
            definition: TokenDefinition {
                name: "Saproling".into(),
                power: 1,
                toughness: 1,
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Saproling],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

/// Warpath — {3}{R}. Three damage to everything tangled up in combat.
pub fn warpath() -> CardDefinition {
    instant(
        "Warpath",
        cost(&[generic(3), r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature.and(R::IsBlocking.or(R::IsBlocked))),
            amount: Value::Const(3),
        },
    )
}

/// Wave of Reckoning — {4}{W}. Every creature shoots itself for its power.
pub fn wave_of_reckoning() -> CardDefinition {
    sorcery(
        "Wave of Reckoning",
        cost(&[generic(4), w()]),
        Effect::ForEach {
            selector: Selector::EachPermanent(R::Creature),
            body: Box::new(Effect::DealDamageEqualToPower {
                source: Selector::TriggerSource,
                target: Selector::TriggerSource,
            }),
        },
    )
}

// ── Artifacts & lands ───────────────────────────────────────────────────────

/// The Ramos artifact cycle: `{T}: Add [colour].` / `Sacrifice: Add [colour].`
fn ramos_stone(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            tap_add(color),
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![color]),
                },
                ..Default::default()
            },
        ],
        ..artifact(name, cost(&[generic(3)]))
    }
}

pub fn tooth_of_ramos() -> CardDefinition {
    ramos_stone("Tooth of Ramos", Color::White)
}
pub fn eye_of_ramos() -> CardDefinition {
    ramos_stone("Eye of Ramos", Color::Blue)
}
pub fn skull_of_ramos() -> CardDefinition {
    ramos_stone("Skull of Ramos", Color::Black)
}
pub fn heart_of_ramos() -> CardDefinition {
    ramos_stone("Heart of Ramos", Color::Red)
}
pub fn horn_of_ramos() -> CardDefinition {
    ramos_stone("Horn of Ramos", Color::Green)
}

/// Iron Lance — {2}. `{3}, {T}`: first strike for a turn.
pub fn iron_lance() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Iron Lance", cost(&[generic(2)]))
    }
}

/// Power Matrix — {4}. `{T}`: +1/+1 plus the full evasion package.
pub fn power_matrix() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature),
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::GrantKeywords {
                    what: target(),
                    keywords: vec![Keyword::Flying, Keyword::FirstStrike, Keyword::Trample],
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Power Matrix", cost(&[generic(4)]))
    }
}

/// Henge Guardian — {5} 3/4 artifact creature with a trample pump.
pub fn henge_guardian() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Henge Guardian",
            cost(&[generic(5)]),
            vec![CreatureType::Dragon, CreatureType::Wurm],
            3,
            4,
        )
    }
}

/// Henge of Ramos — a colorless land with a {2} filter for any colour.
pub fn henge_of_ramos() -> CardDefinition {
    CardDefinition {
        name: "Henge of Ramos",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyOneColor(Value::ONE) },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The MMQ storage-land cycle: enters tapped, banks storage counters, and
/// cashes any number of them in for that much coloured mana at once.
fn storage_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Storage,
                    amount: Value::Const(1),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_x: Some(CounterType::Storage),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(color, Value::XFromCost),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

pub fn fountain_of_cho() -> CardDefinition {
    storage_land("Fountain of Cho", Color::White)
}
pub fn saprazzan_cove() -> CardDefinition {
    storage_land("Saprazzan Cove", Color::Blue)
}
pub fn subterranean_hangar() -> CardDefinition {
    storage_land("Subterranean Hangar", Color::Black)
}
pub fn mercadian_bazaar() -> CardDefinition {
    storage_land("Mercadian Bazaar", Color::Red)
}
pub fn rushwood_grove() -> CardDefinition {
    storage_land("Rushwood Grove", Color::Green)
}

/// The MMQ depletion-land cycle: enters tapped with two depletion counters and
/// is sacrificed once the last one is spent.
fn depletion_land(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        enters_with_counters: Some((CounterType::Depletion, Value::Const(2))),
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped with two depletion counters on it.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Depletion, 1)),
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![color, color]),
                },
                Effect::If {
                    cond: crate::effect::Predicate::Not(Box::new(
                        crate::effect::Predicate::SourceHasCountersAtLeast {
                            counter: CounterType::Depletion,
                            n: 1,
                        },
                    )),
                    then: Box::new(Effect::SacrificeSource),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

pub fn remote_farm() -> CardDefinition {
    depletion_land("Remote Farm", Color::White)
}
pub fn saprazzan_skerry() -> CardDefinition {
    depletion_land("Saprazzan Skerry", Color::Blue)
}
pub fn peat_bog() -> CardDefinition {
    depletion_land("Peat Bog", Color::Black)
}
pub fn sandstone_needle() -> CardDefinition {
    depletion_land("Sandstone Needle", Color::Red)
}
pub fn hickory_woodlot() -> CardDefinition {
    depletion_land("Hickory Woodlot", Color::Green)
}
