//! Invasion (INV) gap-closing wave: the taplands and sac-lands, the Cameos and
//! Attendants, the kicker commons and the multicolour utility shell.
//! Tests in `classic_sets/inv_gaps`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{draw, etb, target_filtered},
};
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
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Sorcery],
        effect,
        ..Default::default()
    }
}

fn enters_tapped() -> StaticAbility {
    StaticAbility {
        description: "This land enters tapped.",
        effect: StaticEffect::EntersTapped { applies_to: Selector::This },
    }
}

fn tap_for(colors: Vec<Color>) -> ActivatedAbility {
    ActivatedAbility {
        tap_cost: true,
        effect: Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::OfColors(colors, Value::ONE),
        },
        ..Default::default()
    }
}

/// The Invasion tapland cycle: enters tapped, taps for either of two colours.
fn tapland(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![tap_for(colors)],
        ..Default::default()
    }
}

/// Coastal Tower — the Azorius tapland.
pub fn coastal_tower() -> CardDefinition {
    tapland("Coastal Tower", vec![Color::White, Color::Blue])
}

/// Elfhame Palace — the Selesnya tapland.
pub fn elfhame_palace() -> CardDefinition {
    tapland("Elfhame Palace", vec![Color::Green, Color::White])
}

/// The Invasion sac-land cycle: taps for one colour, or cracks for two others.
fn sac_land(name: &'static str, taps: Color, cracks: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![enters_tapped()],
        activated_abilities: vec![
            tap_for(vec![taps]),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(cracks),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Ancient Spring — {U}, or crack for {W}{B}.
pub fn ancient_spring() -> CardDefinition {
    sac_land("Ancient Spring", Color::Blue, vec![Color::White, Color::Black])
}

/// Geothermal Crevice — {R}, or crack for {B}{G}.
pub fn geothermal_crevice() -> CardDefinition {
    sac_land("Geothermal Crevice", Color::Red, vec![Color::Black, Color::Green])
}

/// Archaeological Dig — colorless, or crack for any one colour.
pub fn archaeological_dig() -> CardDefinition {
    CardDefinition {
        name: "Archaeological Dig",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::ONE),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// The Cameo cycle: a {3} rock that taps for either of two colours.
fn cameo(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![tap_for(colors)],
        ..Default::default()
    }
}

/// Bloodstone Cameo — {B} or {R}.
pub fn bloodstone_cameo() -> CardDefinition {
    cameo("Bloodstone Cameo", vec![Color::Black, Color::Red])
}

/// Drake-Skull Cameo — {U} or {B}.
pub fn drake_skull_cameo() -> CardDefinition {
    cameo("Drake-Skull Cameo", vec![Color::Blue, Color::Black])
}

/// The Attendant cycle: a {5} 3/3 Golem that cracks for its dragon's colours.
fn attendant(name: &'static str, colors: Vec<Color>) -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            sac_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colors(colors) },
            ..Default::default()
        }],
        ..creature(name, cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Crosis's Attendant — {U}{B}{R}.
pub fn crosiss_attendant() -> CardDefinition {
    attendant("Crosis's Attendant", vec![Color::Blue, Color::Black, Color::Red])
}

/// Darigaaz's Attendant — {B}{R}{G}.
pub fn darigaazs_attendant() -> CardDefinition {
    attendant("Darigaaz's Attendant", vec![Color::Black, Color::Red, Color::Green])
}

/// Dromar's Attendant — {W}{U}{B}.
pub fn dromars_attendant() -> CardDefinition {
    attendant("Dromar's Attendant", vec![Color::White, Color::Blue, Color::Black])
}

/// Alloy Golem — {6} 4/4 that picks its own colour on the way in.
pub fn alloy_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        as_enters_effect: Some(Effect::ChooseColorForSelf),
        static_abilities: vec![StaticAbility {
            description: "Alloy Golem is the chosen color.",
            effect: StaticEffect::SetColorOfMatchingToChosen { applies_to: Selector::This },
        }],
        ..creature("Alloy Golem", cost(&[generic(6)]), vec![CreatureType::Golem], 4, 4)
    }
}

/// Ancient Kavu — {3}{R} 3/3 that can duck colour-based removal.
pub fn ancient_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeColor {
                what: Selector::This,
                colors: vec![],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature("Ancient Kavu", cost(&[generic(3), r()]), vec![CreatureType::Kavu], 3, 3)
    }
}

/// "If this creature was kicked, it enters with `counters` +1/+1 counters on it
/// and with [keywords]."
fn kicked_etb(counters: i32, keywords: Vec<Keyword>) -> TriggeredAbility {
    let mut body = vec![Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(counters),
    }];
    body.extend(keywords.into_iter().map(|keyword| Effect::GrantKeyword {
        what: Selector::This,
        keyword,
        duration: Duration::Permanent,
    }));
    etb(Effect::If {
        cond: Predicate::SpellWasKicked,
        then: Box::new(Effect::Seq(body)),
        else_: Box::new(Effect::Noop),
    })
}

/// Ardent Soldier — {1}{W} 1/2 vigilant. Kicker {2} for a counter.
pub fn ardent_soldier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance, Keyword::Kicker(cost(&[generic(2)]))],
        triggered_abilities: vec![kicked_etb(1, vec![])],
        ..creature(
            "Ardent Soldier",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Benalish Lancer — {2}{W} 2/2. Kicker {2}{W} for two counters and first strike.
pub fn benalish_lancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), w()]))],
        triggered_abilities: vec![kicked_etb(2, vec![Keyword::FirstStrike])],
        ..creature(
            "Benalish Lancer",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Duskwalker — {B} 1/1. Kicker {3}{B} for two counters and fear.
pub fn duskwalker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), b()]))],
        triggered_abilities: vec![kicked_etb(2, vec![Keyword::Fear])],
        ..creature(
            "Duskwalker",
            cost(&[b()]),
            vec![CreatureType::Human, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Faerie Squadron — {U} 1/1. Kicker {3}{U} for two counters and flying.
pub fn faerie_squadron() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), u()]))],
        triggered_abilities: vec![kicked_etb(2, vec![Keyword::Flying])],
        ..creature("Faerie Squadron", cost(&[u()]), vec![CreatureType::Faerie], 1, 1)
    }
}

/// Benalish Emissary — {2}{W} 1/4. Kicker {1}{G} blows up a land.
pub fn benalish_emissary() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), g()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource)
                .with_filter(Predicate::SpellWasKicked),
            effect: Effect::Destroy { what: target_filtered(R::Land) },
        }],
        ..creature(
            "Benalish Emissary",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            4,
        )
    }
}

/// Benalish Heralds — {3}{W} 2/4 that splashes blue for cards.
pub fn benalish_heralds() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: draw(1),
            ..Default::default()
        }],
        ..creature(
            "Benalish Heralds",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            4,
        )
    }
}

/// Benalish Trapper — {1}{W} 1/2 tapper.
pub fn benalish_trapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Tap { what: target_filtered(R::Creature) },
            ..Default::default()
        }],
        ..creature(
            "Benalish Trapper",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Bog Initiate — {1}{B} 1/1 filter.
pub fn bog_initiate() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(Color::Black, Value::ONE),
            },
            ..Default::default()
        }],
        ..creature(
            "Bog Initiate",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Capashen Unicorn — {1}{W} 1/2 that cracks for a Disenchant.
pub fn capashen_unicorn() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
            ..Default::default()
        }],
        ..creature(
            "Capashen Unicorn",
            cost(&[generic(1), w()]),
            vec![CreatureType::Unicorn],
            1,
            2,
        )
    }
}

/// Charging Troll — {2}{G}{W} 3/3 vigilant regenerator.
pub fn charging_troll() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Charging Troll",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Troll],
            3,
            3,
        )
    }
}

/// Cinder Shade — {1}{B}{R} 1/1 Shade that can throw itself.
pub fn cinder_shade() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[b()]),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::ONE,
                    toughness: Value::ONE,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                sac_cost: true,
                effect: Effect::DealDamageEqualToPower {
                    source: Selector::This,
                    target: target_filtered(R::Creature),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Cinder Shade",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Shade],
            1,
            1,
        )
    }
}

/// Crimson Acolyte — {1}{W} 1/1 that hands out protection from red.
pub fn crimson_acolyte() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Protection(Color::Red),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Crimson Acolyte",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Dream Thrush — {1}{U} 1/1 flier that rewrites a land's type.
pub fn dream_thrush() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::LandsBecomeChosenBasicType {
                what: target_filtered(R::Land),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dream Thrush", cost(&[generic(1), u()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Firebrand Ranger — {1}{R} 2/1 that splashes green for a land drop.
pub fn firebrand_ranger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Land.and(R::IsBasicLand),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Firebrand Ranger",
            cost(&[generic(1), r()]),
            vec![CreatureType::Human, CreatureType::Soldier, CreatureType::Scout],
            2,
            1,
        )
    }
}

/// Firescreamer — {3}{B} 2/2 Kavu that splashes red.
pub fn firescreamer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Firescreamer", cost(&[generic(3), b()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// Galina's Knight — {W}{U} 2/2 with protection from red.
pub fn galinas_knight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Red)],
        ..creature(
            "Galina's Knight",
            cost(&[w(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Glimmering Angel — {3}{W} 2/2 flier that can duck targeting.
pub fn glimmering_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Shroud,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Glimmering Angel", cost(&[generic(3), w()]), vec![CreatureType::Angel], 2, 2)
    }
}

/// Goblin Spy — {R} 1/1 that plays open-handed.
pub fn goblin_spy() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Play with the top card of your library revealed.",
            effect: StaticEffect::TopOfLibraryRevealed,
        }],
        ..creature(
            "Goblin Spy",
            cost(&[r()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            1,
            1,
        )
    }
}

/// Hate Weaver — {1}{B} 2/1 that pumps the Izzet side.
pub fn hate_weaver() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::PumpPT {
                what: target_filtered(
                    R::Creature.and(R::HasColor(Color::Blue).or(R::HasColor(Color::Red))),
                ),
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Hate Weaver",
            cost(&[generic(1), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            2,
            1,
        )
    }
}

/// Hooded Kavu — {2}{R} 2/2 that splashes black for evasion.
pub fn hooded_kavu() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Fear,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Hooded Kavu", cost(&[generic(2), r()]), vec![CreatureType::Kavu], 2, 2)
    }
}

/// "Whenever this creature deals damage, you gain that much life."
fn damage_lifelink() -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
        effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
    }
}

/// Horned Cheetah — {2}{G}{W} 2/2 with the Invasion lifelink wording.
pub fn horned_cheetah() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![damage_lifelink()],
        ..creature(
            "Horned Cheetah",
            cost(&[generic(2), g(), w()]),
            vec![CreatureType::Cat],
            2,
            2,
        )
    }
}

/// Armadillo Cloak — {1}{G}{W} Aura. +2/+2, trample and the damage drain.
pub fn armadillo_cloak() -> CardDefinition {
    CardDefinition {
        name: "Armadillo Cloak",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            power: 2,
            toughness: 2,
            keywords: vec![Keyword::Trample],
            triggered_abilities: vec![damage_lifelink()],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Exotic Curse — {2}{B} Aura. Domain-sized shrink.
pub fn exotic_curse() -> CardDefinition {
    CardDefinition {
        name: "Exotic Curse",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        static_abilities: vec![StaticAbility {
            description: "Domain — Enchanted creature gets -1/-1 for each basic land type among \
                          lands you control.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: Selector::attached_to(Selector::This),
                power: Value::Times(
                    Box::new(Value::Const(-1)),
                    Box::new(Value::DomainCount(PlayerRef::You)),
                ),
                toughness: Value::Times(
                    Box::new(Value::Const(-1)),
                    Box::new(Value::DomainCount(PlayerRef::You)),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Angelic Shield — {W}{U}. A toughness anthem you can cash in for a bounce.
pub fn angelic_shield() -> CardDefinition {
    CardDefinition {
        name: "Angelic Shield",
        cost: cost(&[w(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 0,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elfhame Sanctuary — {1}{G}. Trade your draw step for a basic land.
pub fn elfhame_sanctuary() -> CardDefinition {
    CardDefinition {
        name: "Elfhame Sanctuary",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::MayDo {
                description: "Search for a basic land and skip your draw step?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Land.and(R::IsBasicLand),
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                    Effect::SkipPlayerDrawStep { player: PlayerRef::You },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Addle — {1}{B}. Reveal their hand and take a card. (The printed colour is
/// named before the reveal; a hand-zone filter can't read the source's chosen
/// colour yet — see TODO.md — so the caster's pick stands in for it.)
pub fn addle() -> CardDefinition {
    sorcery(
        "Addle",
        cost(&[generic(1), b()]),
        Effect::DiscardChosen {
            from: Selector::Target(0),
            count: Value::ONE,
            filter: R::Any,
        },
    )
}

/// Annihilate — {3}{B}{B}. Kill and cantrip.
pub fn annihilate() -> CardDefinition {
    instant(
        "Annihilate",
        cost(&[generic(3), b(), b()]),
        Effect::Seq(vec![
            Effect::DestroyNoRegen {
                what: target_filtered(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black))))),
            },
            draw(1),
        ]),
    )
}

/// Agonizing Demise — {3}{B}. Kicker {1}{R} turns the corpse into burn.
pub fn agonizing_demise() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(1), r()]))],
        ..instant(
            "Agonizing Demise",
            cost(&[generic(3), b()]),
            Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::DealDamage {
                        to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                        amount: Value::PowerOf(Box::new(Selector::Target(0))),
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::DestroyNoRegen {
                    what: target_filtered(
                        R::Creature.and(R::Not(Box::new(R::HasColor(Color::Black)))),
                    ),
                },
            ]),
        )
    }
}

/// Backlash — {1}{B}{R}. Their creature punches its own controller.
pub fn backlash() -> CardDefinition {
    instant(
        "Backlash",
        cost(&[generic(1), b(), r()]),
        Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature.and(R::Untapped)) },
            Effect::DealDamageEqualToPower {
                source: Selector::Target(0),
                target: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
            },
        ]),
    )
}

/// Dismantling Blow — {2}{W}. Kicker {2}{U} draws two.
pub fn dismantling_blow() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2), u()]))],
        ..instant(
            "Dismantling Blow",
            cost(&[generic(2), w()]),
            Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(draw(2)),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Dredge — {B}. A land or a body for a card.
pub fn dredge() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature.or(R::Land),
            count: 1,
        }],
        ..instant("Dredge", cost(&[b()]), draw(1))
    }
}

/// Explosive Growth — {G}. Kicker {5} for a much bigger pump.
pub fn explosive_growth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(5)]))],
        ..instant(
            "Explosive Growth",
            cost(&[g()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(5)),
                    else_: Box::new(Value::Const(2)),
                },
                toughness: Value::IfPred {
                    pred: Box::new(Predicate::SpellWasKicked),
                    then: Box::new(Value::Const(5)),
                    else_: Box::new(Value::Const(2)),
                },
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// The Invasion sweeper pair: 1 damage (4 when kicked) to a slice of the board
/// and every player.
fn kicked_sweeper(
    name: &'static str,
    c: ManaCost,
    victims: R,
) -> CardDefinition {
    let amount = Value::IfPred {
        pred: Box::new(Predicate::SpellWasKicked),
        then: Box::new(Value::Const(4)),
        else_: Box::new(Value::ONE),
    };
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(2)]))],
        ..sorcery(
            name,
            c,
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(victims),
                    amount: amount.clone(),
                },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachPlayer), amount },
            ]),
        )
    }
}

/// Breath of Darigaaz — {1}{R}. Hits the ground.
pub fn breath_of_darigaaz() -> CardDefinition {
    kicked_sweeper(
        "Breath of Darigaaz",
        cost(&[generic(1), r()]),
        R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
    )
}

/// Canopy Surge — {1}{G}. Hits the air.
pub fn canopy_surge() -> CardDefinition {
    kicked_sweeper(
        "Canopy Surge",
        cost(&[generic(1), g()]),
        R::Creature.and(R::HasKeyword(Keyword::Flying)),
    )
}

/// Defiling Tears — {2}{B}. Repaints a creature and lends it a regenerate.
pub fn defiling_tears() -> CardDefinition {
    instant(
        "Defiling Tears",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::BecomeColor {
                what: target_filtered(R::Creature),
                colors: vec![Color::Black],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::GainActivatedAbility {
                what: target_filtered(R::Creature),
                ability: Box::new(ActivatedAbility {
                    mana_cost: cost(&[b()]),
                    effect: Effect::Regenerate { what: Selector::This },
                    ..Default::default()
                }),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}
