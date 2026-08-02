//! Odyssey (ODY) gap-closing wave 1: the sac-land cycle, the Threshold and
//! flashback commons, and the graveyard-matters shell.
//! Tests in `classic_sets/ody`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R,
    StaticAbility, StaticEffect, Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::{
    Duration, Effect, LibraryPosition, ManaPayload, PlayerRef, Selector, ZoneDest,
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
    CardDefinition { name, cost: c, card_types: vec![CardType::Instant], effect, ..Default::default() }
}

fn sorcery(name: &'static str, c: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

/// "Threshold — as long as there are seven or more cards in your graveyard…"
fn threshold() -> Predicate {
    Predicate::ThresholdActive { who: PlayerRef::You }
}

/// Phantom Whelp's shared attack/block body.
fn bounce_at_end_of_combat() -> Effect {
    Effect::AtEndOfCombat {
        body: Box::new(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        }),
    }
}

/// Rabid Elephant — +2/+2 for each creature blocking it.
fn blocker_pump() -> Value {
    Value::Times(
        Box::new(Value::Const(2)),
        Box::new(Value::BlockersOf(Box::new(Selector::This))),
    )
}

// ── The sac-land cycle ──────────────────────────────────────────────────────

/// Enters tapped, taps for `taps`, or cracks for one mana of any colour.
fn sac_land(name: &'static str, taps: Color) -> CardDefinition {
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
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![taps], Value::ONE),
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

pub fn abandoned_outpost() -> CardDefinition {
    sac_land("Abandoned Outpost", Color::White)
}
pub fn seafloor_debris() -> CardDefinition {
    sac_land("Seafloor Debris", Color::Blue)
}
pub fn bog_wreckage() -> CardDefinition {
    sac_land("Bog Wreckage", Color::Black)
}
pub fn ravaged_highlands() -> CardDefinition {
    sac_land("Ravaged Highlands", Color::Red)
}
pub fn timberland_ruins() -> CardDefinition {
    sac_land("Timberland Ruins", Color::Green)
}

// ── White ───────────────────────────────────────────────────────────────────

/// Beloved Chaplain — {1}{W} 1/1 with protection from creatures.
pub fn beloved_chaplain() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::ProtectionFromCreatures],
        ..creature(
            "Beloved Chaplain",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Blessed Orator — {3}{W} 1/4 that toughens your other creatures.
pub fn blessed_orator() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: 0,
                toughness: 1,
            },
        }],
        ..creature(
            "Blessed Orator",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            4,
        )
    }
}

/// Dedicated Martyr — {W} 1/1 that cashes itself in for 3 life.
pub fn dedicated_martyr() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            sac_cost: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature(
            "Dedicated Martyr",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// The Mystic cycle: Threshold turns each into a bigger flier.
fn mystic(
    name: &'static str,
    c: ManaCost,
    p: i32,
    t: i32,
    printed: Vec<Keyword>,
    bonus: (i32, i32),
) -> CardDefinition {
    CardDefinition {
        keywords: printed,
        static_abilities: vec![StaticAbility {
            description: "Threshold — this creature gets the bonus and has flying.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: bonus.0,
                toughness: bonus.1,
                keywords: vec![Keyword::Flying],
            },
        }],
        ..creature(
            name,
            c,
            vec![CreatureType::Human, CreatureType::Nomad, CreatureType::Mystic],
            p,
            t,
        )
    }
}

pub fn mystic_penitent() -> CardDefinition {
    mystic("Mystic Penitent", cost(&[w()]), 1, 1, vec![Keyword::Vigilance], (1, 1))
}
pub fn mystic_visionary() -> CardDefinition {
    mystic("Mystic Visionary", cost(&[generic(1), w()]), 2, 1, vec![], (0, 0))
}
pub fn mystic_zealot() -> CardDefinition {
    mystic("Mystic Zealot", cost(&[generic(3), w()]), 2, 4, vec![], (1, 1))
}

/// Tireless Tribe — {W} 1/1 that pitches its hand for toughness.
pub fn tireless_tribe() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 1)),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::Const(4),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Tireless Tribe",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            1,
            1,
        )
    }
}

/// Soulcatcher — {1}{W} 1/1 flier that grows on every dying flier.
pub fn soulcatcher() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasKeyword(Keyword::Flying),
                }),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Soulcatcher",
            cost(&[generic(1), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Dogged Hunter — {2}{W} 1/1 that taps to eat a token.
pub fn dogged_hunter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::IsToken)) },
            ..Default::default()
        }],
        ..creature(
            "Dogged Hunter",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Nomad],
            1,
            1,
        )
    }
}

/// Ray of Distortion — {3}{W} naturalize with an expensive flashback.
pub fn ray_of_distortion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), w(), w()]))],
        ..instant(
            "Ray of Distortion",
            cost(&[generic(3), w()]),
            Effect::Destroy { what: target_filtered(R::Artifact.or(R::Enchantment)) },
        )
    }
}

/// Second Thoughts — {4}{W} exiles an attacker and replaces itself.
pub fn second_thoughts() -> CardDefinition {
    instant(
        "Second Thoughts",
        cost(&[generic(4), w()]),
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Creature.and(R::IsAttacking)) },
            draw(1),
        ]),
    )
}

/// Cease-Fire — {2}{W} shuts off a player's creature spells and cantrips.
pub fn cease_fire() -> CardDefinition {
    instant(
        "Cease-Fire",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::PlayerCantCastMatchingThisTurn {
                who: PlayerRef::Target(0),
                filter: R::Creature,
            },
            draw(1),
        ]),
    )
}

/// Aven Cloudchaser — {3}{W} 2/2 flier that eats an enchantment on arrival.
pub fn aven_cloudchaser() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Destroy {
            what: target_filtered(R::Enchantment),
        })],
        ..creature(
            "Aven Cloudchaser",
            cost(&[generic(3), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Aven Flock — {4}{W} 2/3 flier that pumps its own toughness.
pub fn aven_flock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(0),
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Aven Flock",
            cost(&[generic(4), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            2,
            3,
        )
    }
}

// ── Blue ────────────────────────────────────────────────────────────────────

/// Cephalid Looter — {2}{U} 2/1 that loots any player each turn.
pub fn cephalid_looter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::Player(PlayerRef::Target(0)), amount: Value::ONE },
                Effect::Discard {
                    who: Selector::Player(PlayerRef::Target(0)),
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Cephalid Looter",
            cost(&[generic(2), u()]),
            vec![CreatureType::Octopus, CreatureType::Rogue],
            2,
            1,
        )
    }
}

/// Cephalid Scout — {1}{U} 1/1 flier that cashes lands for cards.
pub fn cephalid_scout() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            sac_other_filter: Some((R::Land, 1)),
            effect: draw(1),
            ..Default::default()
        }],
        ..creature(
            "Cephalid Scout",
            cost(&[generic(1), u()]),
            vec![CreatureType::Octopus, CreatureType::Wizard, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Escape Artist — {1}{U} 1/1 unblockable that ducks back to hand.
pub fn escape_artist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..creature(
            "Escape Artist",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Words of Wisdom — {1}{U} draw two, everyone else draws one.
pub fn words_of_wisdom() -> CardDefinition {
    instant(
        "Words of Wisdom",
        cost(&[generic(1), u()]),
        Effect::Seq(vec![
            draw(2),
            Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        ]),
    )
}

/// Deluge — {2}{U} taps every ground creature.
pub fn deluge() -> CardDefinition {
    instant(
        "Deluge",
        cost(&[generic(2), u()]),
        Effect::Tap {
            what: Selector::EachPermanent(
                R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ),
        },
    )
}

/// Repel — {3}{U} puts a creature on top of its owner's library.
pub fn repel() -> CardDefinition {
    instant(
        "Repel",
        cost(&[generic(3), u()]),
        Effect::Move {
            what: target_filtered(R::Creature),
            to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Top },
        },
    )
}

/// Aven Windreader — {3}{U}{U} 3/3 flier that peeks at a library top.
pub fn aven_windreader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::RevealTopOfLibrary { who: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..creature(
            "Aven Windreader",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Bird, CreatureType::Soldier, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Thought Nibbler — {U} 1/1 flier that costs you two cards of hand size.
pub fn thought_nibbler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Your maximum hand size is reduced by two.",
            effect: StaticEffect::ControllerMaxHandSizeReduced(2),
        }],
        ..creature("Thought Nibbler", cost(&[u()]), vec![CreatureType::Beast], 1, 1)
    }
}

/// Phantom Whelp — {1}{U} 2/2 that bounces itself after fighting.
pub fn phantom_whelp() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: bounce_at_end_of_combat(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: bounce_at_end_of_combat(),
            },
        ],
        ..creature(
            "Phantom Whelp",
            cost(&[generic(1), u()]),
            vec![CreatureType::Illusion, CreatureType::Dog],
            2,
            2,
        )
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Dusk Imp — {2}{B} 2/1 flier.
pub fn dusk_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        ..creature("Dusk Imp", cost(&[generic(2), b()]), vec![CreatureType::Imp], 2, 1)
    }
}

/// Crypt Creeper — {1}{B} 2/1 that eats a graveyard card on the way out.
pub fn crypt_creeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::Exile },
            ..Default::default()
        }],
        ..creature("Crypt Creeper", cost(&[generic(1), b()]), vec![CreatureType::Zombie], 2, 1)
    }
}

/// Zombie Cannibal — {B} 1/1 that exiles from the graveyard it just hit.
pub fn zombie_cannibal() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Exile a card from that player's graveyard?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::InGraveyard.and(R::ControlledByTriggerPlayer)),
                    to: ZoneDest::Exile,
                }),
            },
        }],
        ..creature("Zombie Cannibal", cost(&[b()]), vec![CreatureType::Zombie], 1, 1)
    }
}

/// Coffin Purge — {B} graveyard hate that comes back once.
pub fn coffin_purge() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[b()]))],
        ..instant(
            "Coffin Purge",
            cost(&[b()]),
            Effect::Move { what: target_filtered(R::InGraveyard), to: ZoneDest::Exile },
        )
    }
}

/// Afflict — {2}{B} shrink a creature and replace itself.
pub fn afflict() -> CardDefinition {
    instant(
        "Afflict",
        cost(&[generic(2), b()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

/// Skull Fracture — {B} a discard with an expensive flashback.
pub fn skull_fracture() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(3), b()]))],
        ..sorcery(
            "Skull Fracture",
            cost(&[b()]),
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
        )
    }
}

/// Morgue Theft — {1}{B} raise-dead with flashback.
pub fn morgue_theft() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(4), b()]))],
        ..sorcery(
            "Morgue Theft",
            cost(&[generic(1), b()]),
            Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::InYourGraveyard),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
        )
    }
}

/// Decompose — {1}{B} exiles up to three cards from one graveyard.
pub fn decompose() -> CardDefinition {
    sorcery(
        "Decompose",
        cost(&[generic(1), b()]),
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::InGraveyard,
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Exile }),
        },
    )
}

/// Frightcrawler — {1}{B} 1/1 Fear that swells past Threshold.
pub fn frightcrawler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        static_abilities: vec![StaticAbility {
            description: "Threshold — +2/+2 and can't block.",
            effect: StaticEffect::PumpSelfIf {
                condition: threshold(),
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::CantBlock],
            },
        }],
        ..creature("Frightcrawler", cost(&[generic(1), b()]), vec![CreatureType::Horror], 1, 1)
    }
}

/// Filthy Cur — {1}{B} 2/2 that passes its damage on to you.
pub fn filthy_cur() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::LoseLife {
                who: Selector::You,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Filthy Cur", cost(&[generic(1), b()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Whispering Shade — {3}{B} 1/1 swampwalking Shade.
pub fn whispering_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Whispering Shade", cost(&[generic(3), b()]), vec![CreatureType::Shade], 1, 1)
    }
}

/// Ghastly Demise — {B} kills a small nonblack creature, scaled by your yard.
pub fn ghastly_demise() -> CardDefinition {
    instant(
        "Ghastly Demise",
        cost(&[b()]),
        Effect::Destroy {
            what: target_filtered(
                R::Creature
                    .and(R::Not(Box::new(R::HasColor(Color::Black))))
                    .and(R::ToughnessAtMostGraveyardCount),
            ),
        },
    )
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Halberdier — {3}{R} 3/1 first striker.
pub fn halberdier() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        ..creature(
            "Halberdier",
            cost(&[generic(3), r()]),
            vec![CreatureType::Human, CreatureType::Barbarian],
            3,
            1,
        )
    }
}

/// Mad Dog — {1}{R} 2/2 that must attack or be sacrificed.
pub fn mad_dog() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(crate::game::TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::SourceAttackedThisTurn))),
            effect: Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::IsSource },
        }],
        ..creature("Mad Dog", cost(&[generic(1), r()]), vec![CreatureType::Dog], 2, 2)
    }
}

/// Pardic Swordsmith — {2}{R} 1/1 that pitches at random for power.
pub fn pardic_swordsmith() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Pardic Swordsmith", cost(&[generic(2), r()]), vec![CreatureType::Dwarf], 1, 1)
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

fn squirrel() -> TokenDefinition {
    TokenDefinition {
        name: "Squirrel".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Squirrel], ..Default::default() },
        ..Default::default()
    }
}

/// Chatter of the Squirrel — {G} a Squirrel now and another from the yard.
pub fn chatter_of_the_squirrel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flashback(cost(&[generic(1), g()]))],
        ..sorcery(
            "Chatter of the Squirrel",
            cost(&[g()]),
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: squirrel() },
        )
    }
}

/// Woodland Druid — {G} 1/2.
pub fn woodland_druid() -> CardDefinition {
    creature(
        "Woodland Druid",
        cost(&[g()]),
        vec![CreatureType::Human, CreatureType::Druid],
        1,
        2,
    )
}

/// Druid Lyrist — {G} 1/1 that trades itself for an enchantment.
pub fn druid_lyrist() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            sac_cost: true,
            effect: Effect::Destroy { what: target_filtered(R::Enchantment) },
            ..Default::default()
        }],
        ..creature(
            "Druid Lyrist",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Leaf Dancer — {1}{G}{G} 2/2 forestwalker.
pub fn leaf_dancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        ..creature("Leaf Dancer", cost(&[generic(1), g(), g()]), vec![CreatureType::Centaur], 2, 2)
    }
}

/// Rabid Elephant — {4}{G} 3/4 that punishes gang blocks.
pub fn rabid_elephant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: blocker_pump(),
                toughness: blocker_pump(),
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Rabid Elephant", cost(&[generic(4), g()]), vec![CreatureType::Elephant], 3, 4)
    }
}

/// Nantuko Disciple — {3}{G} 2/2 repeatable pump.
pub fn nantuko_disciple() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Nantuko Disciple",
            cost(&[generic(3), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            2,
            2,
        )
    }
}

/// Nantuko Elder — {2}{G} 1/2 that taps for {C}{G}.
pub fn nantuko_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Green], Value::ONE),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Nantuko Elder",
            cost(&[generic(2), g()]),
            vec![CreatureType::Insect, CreatureType::Druid],
            1,
            2,
        )
    }
}

/// Krosan Avenger — {2}{G} 3/1 trampler that regenerates past Threshold.
pub fn krosan_avenger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            condition: Some(threshold()),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature(
            "Krosan Avenger",
            cost(&[generic(2), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            3,
            1,
        )
    }
}

/// Muscle Burst — {1}{G} +3/+3, plus one per copy in every graveyard.
pub fn muscle_burst() -> CardDefinition {
    let amount = Value::Sum(vec![
        Value::Const(3),
        Value::CardsNamedLikeSourceInAllGraveyards,
    ]);
    instant(
        "Muscle Burst",
        cost(&[generic(1), g()]),
        Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: amount.clone(),
            toughness: amount,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Refresh — {2}{G} regenerate and replace itself.
pub fn refresh() -> CardDefinition {
    instant(
        "Refresh",
        cost(&[generic(2), g()]),
        Effect::Seq(vec![
            Effect::Regenerate { what: target_filtered(R::Creature) },
            draw(1),
        ]),
    )
}

/// Simplify — {G} each player sacrifices an enchantment.
pub fn simplify() -> CardDefinition {
    sorcery(
        "Simplify",
        cost(&[g()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::ONE,
            filter: R::Enchantment,
        },
    )
}

/// Rites of Spring — {1}{G} trade a hand for basics.
pub fn rites_of_spring() -> CardDefinition {
    sorcery(
        "Rites of Spring",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::DiscardAnyNumber { who: Selector::You },
            Effect::Repeat {
                count: Value::CardsDiscardedThisEffect,
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::IsBasicLand,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ]),
    )
}

/// Life Burst — {1}{W} 4 life, plus 4 per copy in every graveyard.
pub fn life_burst() -> CardDefinition {
    instant(
        "Life Burst",
        cost(&[generic(1), w()]),
        Effect::GainLife {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Sum(vec![
                Value::Const(4),
                Value::Times(
                    Box::new(Value::Const(4)),
                    Box::new(Value::CardsNamedLikeSourceInAllGraveyards),
                ),
            ]),
        },
    )
}

/// Mind Burst — {1}{B} discard one, plus one per copy in every graveyard.
pub fn mind_burst() -> CardDefinition {
    sorcery(
        "Mind Burst",
        cost(&[generic(1), b()]),
        Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::Sum(vec![
                Value::ONE,
                Value::CardsNamedLikeSourceInAllGraveyards,
            ]),
            random: false,
        },
    )
}

/// The Sphere cycle — a colour-scoped 2-point damage shield around you.
fn sphere(name: &'static str, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If a source of the named colour would deal damage to you, prevent 2.",
            effect: StaticEffect::ReduceColorDamageToYouBy { color, amount: 2 },
        }],
        ..Default::default()
    }
}

pub fn sphere_of_duty() -> CardDefinition {
    sphere("Sphere of Duty", Color::Green)
}
pub fn sphere_of_grace() -> CardDefinition {
    sphere("Sphere of Grace", Color::Black)
}
pub fn sphere_of_law() -> CardDefinition {
    sphere("Sphere of Law", Color::Red)
}
pub fn sphere_of_reason() -> CardDefinition {
    sphere("Sphere of Reason", Color::Blue)
}
pub fn sphere_of_truth() -> CardDefinition {
    sphere("Sphere of Truth", Color::White)
}

/// Kirtar's Desire — {W} Aura that pacifies, and walls past Threshold.
pub fn kirtars_desire() -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus {
            keywords: vec![Keyword::CantAttack],
            ..Default::default()
        }),
        static_abilities: vec![StaticAbility {
            description: "Threshold — enchanted creature can't block.",
            effect: StaticEffect::PumpTeamIf {
                condition: threshold(),
                applies_to: Selector::attached_to(Selector::This),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::CantBlock],
            },
        }],
        ..CardDefinition {
            name: "Kirtar's Desire",
            cost: cost(&[w()]),
            card_types: vec![CardType::Enchantment],
            ..Default::default()
        }
    }
}

/// Zombie Infestation — {1}{B} pitch two cards for a 2/2 Zombie.
pub fn zombie_infestation() -> CardDefinition {
    CardDefinition {
        name: "Zombie Infestation",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Any, 2)),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Zombie".into(),
                    power: 2,
                    toughness: 2,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Black],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Zombie],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Standstill — {1}{U}: the next spell cast refills the caster's opponents.
pub fn standstill() -> CardDefinition {
    CardDefinition {
        name: "Standstill",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::IsSource },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::OpponentOf(Box::new(
                        PlayerRef::ControllerOf(Box::new(Selector::TriggerSource)),
                    ))),
                    amount: Value::Const(3),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Think Tank — {2}{U}: surveil 1 at each of your upkeeps.
pub fn think_tank() -> CardDefinition {
    CardDefinition {
        name: "Think Tank",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}
