//! Mirage (MIR) — the block's first set. Tests in `classic_sets/mir`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    SelectionRequirement as R, StaticAbility, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::shortcut::{add_colorless, add_mana, etb, on_attack, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Predicate, Selector, StaticEffect, Value,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

fn creature(
    name: &'static str,
    c: ManaCost,
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

fn artifact_creature(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        ..creature(name, c, vec![CreatureType::Golem], p, t)
    }
}

fn instant(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Instant],
        effect,
        ..Default::default()
    }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, enchant: R, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(enchant) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
    }
}

/// "Has first strike as long as it's attacking" (Spirit of the Night).
fn first_strike_while_attacking() -> StaticAbility {
    StaticAbility {
        description: "Has first strike as long as it's attacking.",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::MatchingAmong {
                inner: Box::new(Selector::This),
                filter: R::IsAttacking,
            },
            keyword: Keyword::FirstStrike,
        },
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

/// Pump this creature until end of turn for a mana — the Shade/Nightstalker
/// firebreathing shape.
fn self_pump(mana: ManaCost, power: i32, toughness: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(toughness),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Grant this creature a keyword until end of turn for a mana.
fn self_keyword(mana: ManaCost, keyword: Keyword) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        effect: Effect::GrantKeyword { what: Selector::This, keyword, duration: Duration::EndOfTurn },
        ..Default::default()
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// The Mirage "slow fetch" cycle: enters tapped, then {T} + sacrifice fetches
/// an untapped land of either printed type.
fn slow_fetch(name: &'static str, a: LandType, b: LandType) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(a).or(R::HasLandType(b)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Bad River — fetches an Island or Swamp.
pub fn bad_river() -> CardDefinition {
    slow_fetch("Bad River", LandType::Island, LandType::Swamp)
}

/// Flood Plain — fetches a Plains or Island.
pub fn flood_plain() -> CardDefinition {
    slow_fetch("Flood Plain", LandType::Plains, LandType::Island)
}

/// Grasslands — fetches a Forest or Plains.
pub fn grasslands() -> CardDefinition {
    slow_fetch("Grasslands", LandType::Forest, LandType::Plains)
}

/// Mountain Valley — fetches a Mountain or Forest.
pub fn mountain_valley() -> CardDefinition {
    slow_fetch("Mountain Valley", LandType::Mountain, LandType::Forest)
}

/// Rocky Tar Pit — fetches a Swamp or Mountain.
pub fn rocky_tar_pit() -> CardDefinition {
    slow_fetch("Rocky Tar Pit", LandType::Swamp, LandType::Mountain)
}

/// Crystal Vein — taps for {C}, or cashes itself in for {C}{C}.
pub fn crystal_vein() -> CardDefinition {
    CardDefinition {
        name: "Crystal Vein",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: add_colorless(2),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Teferi's Isle — a phasing legendary land that taps for {U}{U}.
pub fn teferis_isle() -> CardDefinition {
    CardDefinition {
        name: "Teferi's Isle",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Phasing],
        static_abilities: vec![StaticAbility {
            description: "Teferi's Isle enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: add_mana(vec![Color::Blue, Color::Blue]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Bay Falcon — {1}{U} 1/1 flier with vigilance.
pub fn bay_falcon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Bay Falcon", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Blistering Barrier — {2}{R} 5/2 Wall.
pub fn blistering_barrier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        ..creature("Blistering Barrier", cost(&[generic(2), r()]), vec![CreatureType::Wall], 5, 2)
    }
}

/// Cerulean Wyvern — {4}{U} 3/3 flier with protection from green.
pub fn cerulean_wyvern() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Green)],
        ..creature("Cerulean Wyvern", cost(&[generic(4), u()]), vec![CreatureType::Drake], 3, 3)
    }
}

/// Crash of Rhinos — {6}{G}{G} 8/4 trampler.
pub fn crash_of_rhinos() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Crash of Rhinos",
            cost(&[generic(6), g(), g()]),
            vec![CreatureType::Rhino],
            8,
            4,
        )
    }
}

/// Giant Mantis — {3}{G} 2/4 with reach.
pub fn giant_mantis() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        ..creature("Giant Mantis", cost(&[generic(3), g()]), vec![CreatureType::Insect], 2, 4)
    }
}

/// Melesse Spirit — {3}{W}{W} 3/3 flier with protection from black.
pub fn melesse_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Black)],
        ..creature(
            "Melesse Spirit",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Spirit],
            3,
            3,
        )
    }
}

/// Hazerider Drake — {2}{W}{U} 2/3 flier with protection from red.
pub fn hazerider_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature(
            "Hazerider Drake",
            cost(&[generic(2), w(), u()]),
            vec![CreatureType::Drake],
            2,
            3,
        )
    }
}

/// Iron Tusk Elephant — {4}{W} 3/3 trampler.
pub fn iron_tusk_elephant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature(
            "Iron Tusk Elephant",
            cost(&[generic(4), w()]),
            vec![CreatureType::Elephant],
            3,
            3,
        )
    }
}

/// Wild Elephant — {3}{G} 3/3 trampler.
pub fn wild_elephant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        ..creature("Wild Elephant", cost(&[generic(3), g()]), vec![CreatureType::Elephant], 3, 3)
    }
}

/// Volcanic Dragon — {4}{R}{R} 4/4 flier with haste.
pub fn volcanic_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..creature(
            "Volcanic Dragon",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Karoo Meerkat — {1}{G} 2/1 with protection from blue.
pub fn karoo_meerkat() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Blue)],
        ..creature("Karoo Meerkat", cost(&[generic(1), g()]), vec![CreatureType::Mongoose], 2, 1)
    }
}

/// Windreaper Falcon — {1}{R}{G} 1/1 flier with protection from blue.
pub fn windreaper_falcon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Blue)],
        ..creature(
            "Windreaper Falcon",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Bird],
            1,
            1,
        )
    }
}

/// Femeref Scouts — {2}{W} 1/4.
pub fn femeref_scouts() -> CardDefinition {
    creature(
        "Femeref Scouts",
        cost(&[generic(2), w()]),
        vec![CreatureType::Human, CreatureType::Scout],
        1,
        4,
    )
}

/// Horrible Hordes — {3} 2/2 artifact creature with rampage 1.
pub fn horrible_hordes() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        keywords: vec![Keyword::Rampage(1)],
        ..artifact_creature("Horrible Hordes", cost(&[generic(3)]), 2, 2)
    }
}

/// Teeka's Dragon — {9} 5/5 flying trampler with rampage 4.
pub fn teekas_dragon() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        keywords: vec![Keyword::Flying, Keyword::Trample, Keyword::Rampage(4)],
        ..artifact_creature("Teeka's Dragon", cost(&[generic(9)]), 5, 5)
    }
}

/// Patagia Golem — {4} 2/3 artifact creature that can buy flight.
pub fn patagia_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword(cost(&[generic(3)]), Keyword::Flying)],
        ..artifact_creature("Patagia Golem", cost(&[generic(4)]), 2, 3)
    }
}

/// Igneous Golem — {5} 3/4 artifact creature that can buy trample.
pub fn igneous_golem() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword(cost(&[generic(2)]), Keyword::Trample)],
        ..artifact_creature("Igneous Golem", cost(&[generic(5)]), 3, 4)
    }
}

/// Mtenda Herder — {W} 1/1 flanker.
pub fn mtenda_herder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        ..creature(
            "Mtenda Herder",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Scout],
            1,
            1,
        )
    }
}

fn askari(name: &'static str, c: ManaCost, types: Vec<CreatureType>) -> CardDefinition {
    CardDefinition { keywords: vec![Keyword::Flanking], ..creature(name, c, types, 2, 2) }
}

/// Zhalfirin Knight — {2}{W} 2/2 flanker that can buy first strike.
pub fn zhalfirin_knight() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword(cost(&[w(), w()]), Keyword::FirstStrike)],
        ..askari(
            "Zhalfirin Knight",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Zhalfirin Commander — {2}{W} 2/2 flanker that pumps a Knight.
pub fn zhalfirin_commander() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w(), w()]),
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::HasCreatureType(CreatureType::Knight))),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..askari(
            "Zhalfirin Commander",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Femeref Knight — {2}{W} 2/2 flanker that can buy vigilance.
pub fn femeref_knight() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword(cost(&[w()]), Keyword::Vigilance)],
        ..askari(
            "Femeref Knight",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Cadaverous Knight — {2}{B} 2/2 flanker that regenerates.
pub fn cadaverous_knight() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b(), b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..askari(
            "Cadaverous Knight",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Knight],
        )
    }
}

/// Burning Shield Askari — {2}{R} 2/2 flanker that can buy first strike.
pub fn burning_shield_askari() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword(cost(&[r(), r()]), Keyword::FirstStrike)],
        ..askari(
            "Burning Shield Askari",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Searing Spear Askari — {2}{R} 2/2 flanker that can buy menace.
pub fn searing_spear_askari() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_keyword(cost(&[generic(1), r()]), Keyword::Menace)],
        ..askari(
            "Searing Spear Askari",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Jolrael's Centaur — {1}{G}{G} 2/2 with shroud and flanking.
pub fn jolraels_centaur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Shroud, Keyword::Flanking],
        ..creature(
            "Jolrael's Centaur",
            cost(&[generic(1), g(), g()]),
            vec![CreatureType::Centaur, CreatureType::Archer],
            2,
            2,
        )
    }
}

/// Telim'Tor — {4}{R} 2/2 flanker whose attacks rally the whole flanking team.
pub fn telimtor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::EachPermanent(
                R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flanking)),
            ),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        })],
        ..askari(
            "Telim'Tor",
            cost(&[generic(4), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Sidar Jabari — {3}{W} 2/2 flanker that taps a blocker as it attacks.
pub fn sidar_jabari() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        triggered_abilities: vec![on_attack(Effect::Tap {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..askari(
            "Sidar Jabari",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
        )
    }
}

/// Teremko Griffin — {3}{W} 2/2 flier with banding.
pub fn teremko_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Banding],
        ..creature("Teremko Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Noble Elephant — {3}{W} 2/2 trampler with banding.
pub fn noble_elephant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample, Keyword::Banding],
        ..creature("Noble Elephant", cost(&[generic(3), w()]), vec![CreatureType::Elephant], 2, 2)
    }
}

/// Merfolk Raiders — {1}{U} 2/3 islandwalker with phasing.
pub fn merfolk_raiders() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Island), Keyword::Phasing],
        ..creature(
            "Merfolk Raiders",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Dirtwater Wraith — {3}{B} 1/3 swampwalker with firebreathing.
pub fn dirtwater_wraith() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![self_pump(cost(&[b()]), 1, 0)],
        ..creature("Dirtwater Wraith", cost(&[generic(3), b()]), vec![CreatureType::Wraith], 1, 3)
    }
}

/// Fetid Horror — {3}{B} 1/2 Shade.
pub fn fetid_horror() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[b()]), 1, 1)],
        ..creature(
            "Fetid Horror",
            cost(&[generic(3), b()]),
            vec![CreatureType::Shade, CreatureType::Horror],
            1,
            2,
        )
    }
}

/// Breathstealer — {2}{B} 2/2 that trades toughness for power.
pub fn breathstealer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[b()]), 1, -1)],
        ..creature(
            "Breathstealer",
            cost(&[generic(2), b()]),
            vec![CreatureType::Nightstalker],
            2,
            2,
        )
    }
}

/// Restless Dead — {1}{B} 1/1 that regenerates.
pub fn restless_dead() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Restless Dead", cost(&[generic(1), b()]), vec![CreatureType::Skeleton], 1, 1)
    }
}

/// Jungle Troll — {1}{R}{G} 2/1 that regenerates off either colour.
pub fn jungle_troll() -> CardDefinition {
    let regen = |m: ManaCost| ActivatedAbility {
        mana_cost: m,
        effect: Effect::Regenerate { what: Selector::This },
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![regen(cost(&[r()])), regen(cost(&[g()]))],
        ..creature(
            "Jungle Troll",
            cost(&[generic(1), r(), g()]),
            vec![CreatureType::Troll],
            2,
            1,
        )
    }
}

/// Azimaet Drake — {2}{U} 1/3 flier with a once-per-turn pump.
pub fn azimaet_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            once_per_turn: true,
            ..self_pump(cost(&[u()]), 1, 0)
        }],
        ..creature("Azimaet Drake", cost(&[generic(2), u()]), vec![CreatureType::Drake], 1, 3)
    }
}

/// Wildfire Emissary — {3}{R} 2/4 with protection from white and firebreathing.
pub fn wildfire_emissary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::White)],
        activated_abilities: vec![self_pump(cost(&[generic(1), r()]), 1, 0)],
        ..creature("Wildfire Emissary", cost(&[generic(3), r()]), vec![CreatureType::Efreet], 2, 4)
    }
}

/// Pearl Dragon — {4}{W}{W} 4/4 flier that can buy toughness.
pub fn pearl_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![self_pump(cost(&[generic(1), w()]), 0, 1)],
        ..creature(
            "Pearl Dragon",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Sea Scryer — {1}{U} 1/1 that taps for {C}, or {1} more for {U}.
pub fn sea_scryer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: add_mana(vec![Color::Blue]),
                ..Default::default()
            },
        ],
        ..creature(
            "Sea Scryer",
            cost(&[generic(1), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Gibbering Hyenas — {2}{G} 3/2 that can't block black creatures.
pub fn gibbering_hyenas() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBlockMatching(Box::new(R::HasColor(Color::Black)))],
        ..creature("Gibbering Hyenas", cost(&[generic(2), g()]), vec![CreatureType::Hyena], 3, 2)
    }
}

/// Sunweb — {3}{W} 5/6 flying Wall that only stops real threats.
pub fn sunweb() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender, Keyword::Flying, Keyword::CantBlockPowerAtMost(2)],
        ..creature("Sunweb", cost(&[generic(3), w()]), vec![CreatureType::Wall], 5, 6)
    }
}

/// Stalking Tiger — {3}{G} 3/3 that can't be gang-blocked.
pub fn stalking_tiger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        ..creature("Stalking Tiger", cost(&[generic(3), g()]), vec![CreatureType::Cat], 3, 3)
    }
}

/// Spirit of the Night — {6}{B}{B}{B} 6/5 with the whole keyword pile.
pub fn spirit_of_the_night() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![
            Keyword::Flying,
            Keyword::Trample,
            Keyword::Haste,
            Keyword::Protection(Color::Black),
        ],
        static_abilities: vec![first_strike_while_attacking()],
        ..creature(
            "Spirit of the Night",
            cost(&[generic(6), b(), b(), b()]),
            vec![CreatureType::Demon, CreatureType::Spirit],
            6,
            5,
        )
    }
}

/// Zuberi, Golden Feather — {4}{W} 3/3 flier that anthems other Griffins.
pub fn zuberi_golden_feather() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Other Griffin creatures get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Griffin))
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..creature(
            "Zuberi, Golden Feather",
            cost(&[generic(4), w()]),
            vec![CreatureType::Griffin],
            3,
            3,
        )
    }
}

/// Zebra Unicorn — {2}{G}{W} 2/2 that turns its damage into life.
pub fn zebra_unicorn() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature(
            "Zebra Unicorn",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Unicorn],
            2,
            2,
        )
    }
}

/// Harbinger of Night — {2}{B}{B} 2/3 that grinds every board down.
pub fn harbinger_of_night() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature),
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Harbinger of Night",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Spirit],
            2,
            3,
        )
    }
}

/// Crypt Cobra — {3}{B} 3/3 that poisons an unblocked defender.
pub fn crypt_cobra() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::ONE,
            },
        }],
        ..creature("Crypt Cobra", cost(&[generic(3), b()]), vec![CreatureType::Snake], 3, 3)
    }
}

/// Skulking Ghost — {1}{B} 2/1 flier that dies to any attention.
pub fn skulking_ghost() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..creature("Skulking Ghost", cost(&[generic(1), b()]), vec![CreatureType::Spirit], 2, 1)
    }
}

/// Daring Apprentice — {1}{U}{U} 1/1 that cashes itself in for a counterspell.
pub fn daring_apprentice() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            effect: Effect::CounterSpell { what: Selector::Target(0) },
            ..Default::default()
        }],
        ..creature(
            "Daring Apprentice",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Uktabi Faerie — {1}{G} 1/1 flier that eats an artifact.
pub fn uktabi_faerie() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), g()]),
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact) },
            ..Default::default()
        }],
        ..creature("Uktabi Faerie", cost(&[generic(1), g()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Dwarven Miner — {1}{R} 1/2 that grinds nonbasic lands.
pub fn dwarven_miner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), r()]),
            tap_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::IsNonbasicLand),
            },
            ..Default::default()
        }],
        ..creature("Dwarven Miner", cost(&[generic(1), r()]), vec![CreatureType::Dwarf], 1, 2)
    }
}

/// Village Elder — {G} 1/1 that trades Forests for regeneration shields.
pub fn village_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_other_filter: Some((R::HasLandType(LandType::Forest), 1)),
            effect: Effect::Regenerate { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Village Elder",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Mire Shade — {1}{B} 1/1 that grows by eating Swamps.
pub fn mire_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sorcery_speed: true,
            sac_other_filter: Some((R::HasLandType(LandType::Swamp), 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..creature("Mire Shade", cost(&[generic(1), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Flame Elemental — {2}{R}{R} 3/2 that goes out in a blaze.
pub fn flame_elemental() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::PowerOf(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        ..creature(
            "Flame Elemental",
            cost(&[generic(2), r(), r()]),
            vec![CreatureType::Elemental],
            3,
            2,
        )
    }
}

/// Reckless Embermage — {3}{R} 2/2 that pings both ends.
pub fn reckless_embermage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_any(), amount: Value::ONE },
                Effect::DealDamage { to: Selector::This, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Reckless Embermage",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            2,
            2,
        )
    }
}

/// Suq'Ata Firewalker — {1}{U}{U} 0/1 pinger red can't touch.
pub fn suqata_firewalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::HexproofFromColor(Color::Red)],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Suq'Ata Firewalker",
            cost(&[generic(1), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            0,
            1,
        )
    }
}

/// Maro — {2}{G}{G} Elemental whose body is your hand.
pub fn maro() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Maro's power and toughness are each equal to the number of cards in your hand.",
            effect: StaticEffect::SelfBasePtFromValue {
                power: Value::HandSizeOf(PlayerRef::You),
                toughness: Value::HandSizeOf(PlayerRef::You),
            },
        }],
        ..creature("Maro", cost(&[generic(2), g(), g()]), vec![CreatureType::Elemental], 0, 0)
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Unyaro Bee Sting — {3}{G} sorcery for 2 damage anywhere.
pub fn unyaro_bee_sting() -> CardDefinition {
    sorcery(
        "Unyaro Bee Sting",
        cost(&[generic(3), g()]),
        Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
    )
}

/// Savage Twister — {X}{R}{G} sweeper for X.
pub fn savage_twister() -> CardDefinition {
    sorcery(
        "Savage Twister",
        cost(&[crate::mana::x(), r(), g()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(R::Creature),
            amount: Value::XFromCost,
        },
    )
}

/// Blinding Light — {2}{W} sorcery that taps every nonwhite creature.
pub fn blinding_light() -> CardDefinition {
    sorcery(
        "Blinding Light",
        cost(&[generic(2), w()]),
        Effect::Tap {
            what: Selector::EachPermanent(
                R::Creature.and(R::Not(Box::new(R::HasColor(Color::White)))),
            ),
        },
    )
}

/// Nocturnal Raid — {2}{B}{B} team pump for black.
pub fn nocturnal_raid() -> CardDefinition {
    instant(
        "Nocturnal Raid",
        cost(&[generic(2), b(), b()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::HasColor(Color::Black))),
            power: Value::Const(2),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Stupor — {2}{B} sorcery: a random discard, then a chosen one.
pub fn stupor() -> CardDefinition {
    sorcery(
        "Stupor",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: true,
            },
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
        ]),
    )
}

/// Infernal Contract — {B}{B}{B} sorcery: four cards for half your life.
pub fn infernal_contract() -> CardDefinition {
    sorcery(
        "Infernal Contract",
        cost(&[b(), b(), b()]),
        Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
            Effect::LoseHalfLife { who: Selector::You, rounded_up: true },
        ]),
    )
}

/// Vitalizing Cascade — {X}{G}{W} instant: gain X plus 3 life.
pub fn vitalizing_cascade() -> CardDefinition {
    instant(
        "Vitalizing Cascade",
        cost(&[crate::mana::x(), g(), w()]),
        Effect::GainLife {
            who: Selector::You,
            amount: Value::Sum(vec![Value::XFromCost, Value::Const(3)]),
        },
    )
}

/// Tidal Wave — {2}{U} instant: a 5/5 Wall for one combat.
pub fn tidal_wave() -> CardDefinition {
    instant(
        "Tidal Wave",
        cost(&[generic(2), u()]),
        Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Wall".into(),
                    power: 5,
                    toughness: 5,
                    keywords: vec![Keyword::Defender],
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Blue],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Wall],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            Effect::SacrificeLastCreatedTokensAtNextEndStep,
        ]),
    )
}

/// Goblin Scouts — {3}{R}{R} sorcery: three mountainwalking Goblins.
pub fn goblin_scouts() -> CardDefinition {
    sorcery(
        "Goblin Scouts",
        cost(&[generic(3), r(), r()]),
        Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(3),
            definition: TokenDefinition {
                name: "Goblin Scout".into(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Landwalk(LandType::Mountain)],
                card_types: vec![CardType::Creature],
                colors: vec![Color::Red],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Goblin, CreatureType::Scout],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
    )
}

/// Tranquil Domain — {1}{G} instant that sweeps non-Aura enchantments.
pub fn tranquil_domain() -> CardDefinition {
    instant(
        "Tranquil Domain",
        cost(&[generic(1), g()]),
        Effect::Destroy {
            what: Selector::EachPermanent(
                R::Enchantment.and(R::Not(Box::new(R::HasEnchantmentSubtype(
                    EnchantmentSubtype::Aura,
                )))),
            ),
        },
    )
}

/// Fallow Earth — {2}{G} sorcery that puts a land on top of its library.
pub fn fallow_earth() -> CardDefinition {
    sorcery(
        "Fallow Earth",
        cost(&[generic(2), g()]),
        Effect::Move {
            what: target_filtered(R::Land),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        },
    )
}

/// Disempower — {1}{W} instant that stacks an artifact or enchantment.
pub fn disempower() -> CardDefinition {
    instant(
        "Disempower",
        cost(&[generic(1), w()]),
        Effect::Move {
            what: target_filtered(R::Artifact.or(R::Enchantment)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::Top,
            },
        },
    )
}

/// Ashen Powder — {2}{B}{B} sorcery: reanimate out of an opponent's yard.
pub fn ashen_powder() -> CardDefinition {
    sorcery(
        "Ashen Powder",
        cost(&[generic(2), b(), b()]),
        Effect::Move {
            what: target_filtered(R::Creature.and(R::InOpponentGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Withering Boon — {1}{B} instant countering a creature spell for 3 life.
pub fn withering_boon() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::PayLife { amount: 3 }],
        ..instant(
            "Withering Boon",
            cost(&[generic(1), b()]),
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::HasCardType(CardType::Creature))),
            },
        )
    }
}

/// Superior Numbers — {G}{G} sorcery: damage by the creature-count gap.
pub fn superior_numbers() -> CardDefinition {
    sorcery(
        "Superior Numbers",
        cost(&[g(), g()]),
        Effect::DealDamage {
            to: target_filtered(R::Creature),
            amount: Value::NonNeg(Box::new(Value::Diff(
                Box::new(Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou),
                )))),
                Box::new(Value::CountOf(Box::new(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByOpponent),
                )))),
            ))),
        },
    )
}

// ── Artifacts & enchantments ────────────────────────────────────────────────

/// Mana Prism — {3} artifact that filters into any colour.
pub fn mana_prism() -> CardDefinition {
    CardDefinition {
        name: "Mana Prism",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility { tap_cost: true, effect: add_colorless(1), ..Default::default() },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: crate::effect::shortcut::add_any_one_color(1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Telim'Tor's Darts — {2} artifact that plinks a player each turn.
pub fn telimtors_darts() -> CardDefinition {
    CardDefinition {
        name: "Telim'Tor's Darts",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elixir of Vitality — {4} artifact that enters tapped and pays out life.
pub fn elixir_of_vitality() -> CardDefinition {
    let gain = |m: ManaCost, n: i32| ActivatedAbility {
        mana_cost: m,
        tap_cost: true,
        sac_cost: true,
        effect: Effect::GainLife { who: Selector::You, amount: Value::Const(n) },
        ..Default::default()
    };
    CardDefinition {
        name: "Elixir of Vitality",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "This artifact enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        activated_abilities: vec![gain(ManaCost::default(), 4), gain(cost(&[generic(8)]), 8)],
        ..Default::default()
    }
}

/// Enfeeblement — {B}{B} Aura for -2/-2.
pub fn enfeeblement() -> CardDefinition {
    aura(
        "Enfeeblement",
        cost(&[b(), b()]),
        R::Creature,
        EquipBonus { power: -2, toughness: -2, ..Default::default() },
    )
}

/// Ritual of Steel — {2}{W} Aura for +0/+2 that replaces itself.
pub fn ritual_of_steel() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::AtNextTurnsUpkeep {
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
        })],
        ..aura(
            "Ritual of Steel",
            cost(&[generic(2), w()]),
            R::Creature,
            EquipBonus { toughness: 2, ..Default::default() },
        )
    }
}

/// Agility — {1}{R} Aura granting +1/+1 and flanking.
pub fn agility() -> CardDefinition {
    aura(
        "Agility",
        cost(&[generic(1), r()]),
        R::Creature,
        EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Flanking],
            ..Default::default()
        },
    )
}

/// Cloak of Invisibility — {U} Aura: phasing plus a Wall-only blocker clause.
pub fn cloak_of_invisibility() -> CardDefinition {
    aura(
        "Cloak of Invisibility",
        cost(&[u()]),
        R::Creature,
        EquipBonus {
            keywords: vec![
                Keyword::Phasing,
                Keyword::CantBeBlockedBy(Box::new(R::Not(Box::new(R::HasCreatureType(
                    CreatureType::Wall,
                ))))),
            ],
            ..Default::default()
        },
    )
}

/// Teferi's Curse — {1}{U} Aura that gives an artifact or creature phasing.
pub fn teferis_curse() -> CardDefinition {
    aura(
        "Teferi's Curse",
        cost(&[generic(1), u()]),
        R::Artifact.or(R::Creature),
        EquipBonus { keywords: vec![Keyword::Phasing], ..Default::default() },
    )
}

/// Misers' Cage — {3} artifact that punishes a full grip.
pub fn misers_cage() -> CardDefinition {
    CardDefinition {
        name: "Misers' Cage",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            )
            .with_filter(Predicate::ValueAtLeast(
                Value::HandSizeOf(PlayerRef::ActivePlayer),
                Value::Const(5),
            )),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Paupers' Cage — {3} artifact that punishes an empty grip.
pub fn paupers_cage() -> CardDefinition {
    CardDefinition {
        name: "Paupers' Cage",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            )
            .with_filter(Predicate::ValueAtMost(
                Value::HandSizeOf(PlayerRef::ActivePlayer),
                Value::Const(2),
            )),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ActivePlayer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}
