//! Visions (VIS) — the Mirage block's second set. Tests in `classic_sets/vis`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    CumulativeUpkeepCost, EnchantmentSubtype, EquipBonus, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, StaticAbility, Subtypes, TriggeredAbility,
    WardCost,
};
use crate::effect::shortcut::{draw, etb, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
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

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
}

fn aura(name: &'static str, c: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(bonus),
        ..enchantment(name, c)
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

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

fn your_upkeep() -> EventSpec {
    EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
}

/// The "Karoo" bounce lands: enters tapped, costs an untapped basic of its
/// colour, and taps for {C} plus that colour.
fn karoo(name: &'static str, land: LandType, color: Color) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped { applies_to: Selector::This },
        }],
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessCost {
            cost: WardCost::ReturnMatchingToHand(
                Box::new(R::HasLandType(land).and(R::Untapped).and(R::OtherThanSource)),
                1,
            ),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                crate::effect::shortcut::add_colorless(1),
                crate::effect::shortcut::add_mana(vec![color]),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Coral Atoll — {C}{U} for an untapped Island.
pub fn coral_atoll() -> CardDefinition {
    karoo("Coral Atoll", LandType::Island, Color::Blue)
}

/// Dormant Volcano — {C}{R} for an untapped Mountain.
pub fn dormant_volcano() -> CardDefinition {
    karoo("Dormant Volcano", LandType::Mountain, Color::Red)
}

/// Everglades — {C}{B} for an untapped Swamp.
pub fn everglades() -> CardDefinition {
    karoo("Everglades", LandType::Swamp, Color::Black)
}

/// Jungle Basin — {C}{G} for an untapped Forest.
pub fn jungle_basin() -> CardDefinition {
    karoo("Jungle Basin", LandType::Forest, Color::Green)
}

/// Karoo — {C}{W} for an untapped Plains.
pub fn karoo_land() -> CardDefinition {
    karoo("Karoo", LandType::Plains, Color::White)
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Archangel — {5}{W}{W} 5/5 flier with vigilance.
pub fn archangel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Archangel", cost(&[generic(5), w(), w()]), vec![CreatureType::Angel], 5, 5)
    }
}

/// Tempest Drake — {1}{W}{U} 2/2 flier with vigilance.
pub fn tempest_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Tempest Drake", cost(&[generic(1), w(), u()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Freewind Falcon — {1}{W} 1/1 flier with protection from red.
pub fn freewind_falcon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Protection(Color::Red)],
        ..creature("Freewind Falcon", cost(&[generic(1), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Fallen Askari — {1}{B} 2/2 flanker that only ever attacks.
pub fn fallen_askari() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking, Keyword::CantBlock],
        ..creature(
            "Fallen Askari",
            cost(&[generic(1), b()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Suq'Ata Lancer — {2}{R} 2/2 with haste and flanking.
pub fn suqata_lancer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Flanking],
        ..creature(
            "Suq'Ata Lancer",
            cost(&[generic(2), r()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// King Cheetah — {3}{G} 3/2 with flash.
pub fn king_cheetah() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        ..creature("King Cheetah", cost(&[generic(3), g()]), vec![CreatureType::Cat], 3, 2)
    }
}

/// Infantry Veteran — {W} 1/1 that pumps an attacker each turn.
pub fn infantry_veteran() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::IsAttacking)),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Infantry Veteran",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Jamuraan Lion — {2}{W} 3/1 that walks a blocker out of the way.
pub fn jamuraan_lion() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::CantBlock,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Jamuraan Lion", cost(&[generic(2), w()]), vec![CreatureType::Cat], 3, 1)
    }
}

/// Keeper of Kookus — {R} 1/1 that hides from red.
pub fn keeper_of_kookus() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Protection(Color::Red),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Keeper of Kookus", cost(&[r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Army Ants — {1}{B}{R} 1/1 that trades lands one at a time.
pub fn army_ants() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Land, 1)),
            effect: Effect::Destroy { what: target_filtered(R::Land) },
            ..Default::default()
        }],
        ..creature("Army Ants", cost(&[generic(1), b(), r()]), vec![CreatureType::Insect], 1, 1)
    }
}

/// Daraja Griffin — {3}{W} 2/2 flier that cashes itself in for a black body.
pub fn daraja_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black))),
            },
            ..Default::default()
        }],
        ..creature("Daraja Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Resistance Fighter — {W} 1/1 that eats one creature's combat damage.
pub fn resistance_fighter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Resistance Fighter",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Wake of Vultures — {3}{B} 3/1 flier that eats its team to survive.
pub fn wake_of_vultures() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), b()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Wake of Vultures", cost(&[generic(3), b()]), vec![CreatureType::Bird], 3, 1)
    }
}

/// Urborg Mindsucker — {2}{B} 2/2 that cashes in for a random discard.
pub fn urborg_mindsucker() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: true,
            },
            ..Default::default()
        }],
        ..creature("Urborg Mindsucker", cost(&[generic(2), b()]), vec![CreatureType::Horror], 2, 2)
    }
}

/// Tar Pit Warrior — {2}{B} 3/4 that dies the moment anything points at it.
pub fn tar_pit_warrior() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
            effect: Effect::SacrificeSource,
        }],
        ..creature(
            "Tar Pit Warrior",
            cost(&[generic(2), b()]),
            vec![CreatureType::Cyclops, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Goblin Swine-Rider — {R} 1/1 that blows up the whole combat when blocked.
pub fn goblin_swine_rider() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                amount: Value::Const(2),
            },
        }],
        ..creature("Goblin Swine-Rider", cost(&[r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Stampeding Wildebeests — {2}{G}{G} 5/4 trampler that bounces a friend
/// every upkeep.
pub fn stampeding_wildebeests() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::Move {
                what: Selector::one_of(Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::HasColor(Color::Green)),
                )),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::LastMoved))),
            },
        }],
        ..creature(
            "Stampeding Wildebeests",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Antelope, CreatureType::Beast],
            5,
            4,
        )
    }
}

/// Aku Djinn — {3}{B}{B} 5/6 trampler that feeds the opposition every upkeep.
pub fn aku_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(R::Creature.and(R::ControlledByOpponent)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Aku Djinn", cost(&[generic(3), b(), b()]), vec![CreatureType::Djinn], 5, 6)
    }
}

/// Bull Elephant — {3}{G} 4/4 that costs two Forests off the battlefield.
pub fn bull_elephant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessCost {
            cost: WardCost::ReturnMatchingToHand(Box::new(R::HasLandType(LandType::Forest)), 2),
        })],
        ..creature("Bull Elephant", cost(&[generic(3), g()]), vec![CreatureType::Elephant], 4, 4)
    }
}

/// Crypt Rats — {2}{B} 1/1 that scales its own sweeper with black mana.
pub fn crypt_rats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            x_mana_color: Some(Color::Black),
            effect: Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::XFromCost,
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::XFromCost,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Crypt Rats", cost(&[generic(2), b()]), vec![CreatureType::Rat], 1, 1)
    }
}

/// Firestorm Hellkite — {4}{U}{R} 6/6 flying trampler on a two-colour clock.
pub fn firestorm_hellkite() -> CardDefinition {
    CardDefinition {
        keywords: vec![
            Keyword::Flying,
            Keyword::Trample,
            Keyword::CumulativeUpkeep(CumulativeUpkeepCost::Mana(cost(&[u(), r()]))),
        ],
        ..creature(
            "Firestorm Hellkite",
            cost(&[generic(4), u(), r()]),
            vec![CreatureType::Dragon],
            6,
            6,
        )
    }
}

/// Lichenthrope — {3}{G}{G} 5/5 that takes damage as counters and heals one
/// each upkeep.
pub fn lichenthrope() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::DamageBecomesMinusCounters],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::RemoveCounter {
                what: Selector::This,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..creature(
            "Lichenthrope",
            cost(&[generic(3), g(), g()]),
            vec![CreatureType::Plant, CreatureType::Fungus],
            5,
            5,
        )
    }
}

// ── Auras & enchantments ────────────────────────────────────────────────────

/// Dark Privilege — {1}{B} Aura. +1/+1 and a creature-fuelled regenerator.
pub fn dark_privilege() -> CardDefinition {
    aura(
        "Dark Privilege",
        cost(&[generic(1), b()]),
        EquipBonus {
            power: 1,
            toughness: 1,
            activated_abilities: vec![ActivatedAbility {
                sac_other_filter: Some((R::Creature, 1)),
                effect: Effect::Regenerate { what: Selector::This },
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Sun Clasp — {1}{W} Aura. +1/+3 with an escape hatch.
pub fn sun_clasp() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::Move {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::LastMoved))),
            },
            ..Default::default()
        }],
        ..aura(
            "Sun Clasp",
            cost(&[generic(1), w()]),
            EquipBonus { power: 1, toughness: 3, ..Default::default() },
        )
    }
}

/// Spider Climb — {G} Aura. +0/+3 and reach.
pub fn spider_climb() -> CardDefinition {
    aura(
        "Spider Climb",
        cost(&[g()]),
        EquipBonus { toughness: 3, keywords: vec![Keyword::Reach], ..Default::default() },
    )
}

/// Blanket of Night — {1}{B}{B}. Every land is a Swamp.
pub fn blanket_of_night() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Each land is a Swamp in addition to its other land types.",
            effect: StaticEffect::LandTypeChanger {
                applies_to: Selector::EachPermanent(R::Land),
                land_type: LandType::Swamp,
                replace: false,
            },
        }],
        ..enchantment("Blanket of Night", cost(&[generic(1), b(), b()]))
    }
}

/// Rowen — {2}{G}{G}. Your first draw each turn cantrips off a basic land.
pub fn rowen() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::FirstCardDrawnThisTurn, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::IsBasicLand,
                }),
            effect: draw(1),
        }],
        ..enchantment("Rowen", cost(&[generic(2), g(), g()]))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Sisay's Ring — {4}. Two colorless a turn.
pub fn sisays_ring() -> CardDefinition {
    artifact(
        "Sisay's Ring",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            tap_cost: true,
            effect: crate::effect::shortcut::add_colorless(2),
            ..Default::default()
        }],
    )
}

/// Helm of Awakening — {2}. Every spell on the table costs {1} less.
pub fn helm_of_awakening() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Spells cost {1} less to cast.",
            effect: StaticEffect::AllPlayersSpellsCostLess { amount: 1 },
        }],
        ..artifact("Helm of Awakening", cost(&[generic(2)]), vec![])
    }
}

/// Triangle of War — {1}. Cashes in for a fight.
pub fn triangle_of_war() -> CardDefinition {
    artifact(
        "Triangle of War",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Fight {
                attacker: target_filtered(R::Creature.and(R::ControlledByYou)),
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
            ..Default::default()
        }],
    )
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Coercion — {2}{B}. You pick what they pitch.
pub fn coercion() -> CardDefinition {
    sorcery(
        "Coercion",
        cost(&[generic(2), b()]),
        Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::Target(0)),
            count: Value::ONE,
            filter: R::Any,
        },
    )
}

/// Warrior's Honor — {2}{W}. A team pump.
pub fn warriors_honor() -> CardDefinition {
    instant(
        "Warrior's Honor",
        cost(&[generic(2), w()]),
        Effect::PumpPT {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
            power: Value::ONE,
            toughness: Value::ONE,
            duration: Duration::EndOfTurn,
        },
    )
}

/// Tremor — {R}. One damage to everything on the ground.
pub fn tremor() -> CardDefinition {
    sorcery(
        "Tremor",
        cost(&[r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(
                R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
            ),
            amount: Value::ONE,
        },
    )
}

/// Simoon — {R}{G}. One damage to one opponent's whole board.
pub fn simoon() -> CardDefinition {
    instant(
        "Simoon",
        cost(&[r(), g()]),
        Effect::DealDamage {
            to: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
            amount: Value::ONE,
        },
    )
}

/// Retribution of the Meek — {2}{W}. The big ones die.
pub fn retribution_of_the_meek() -> CardDefinition {
    sorcery(
        "Retribution of the Meek",
        cost(&[generic(2), w()]),
        Effect::DestroyNoRegen {
            what: Selector::EachPermanent(R::Creature.and(R::PowerAtLeast(4))),
        },
    )
}

/// Undo — {1}{U}{U}. Two creatures go home.
pub fn undo() -> CardDefinition {
    sorcery(
        "Undo",
        cost(&[generic(1), u(), u()]),
        Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 2,
            filter: R::Creature,
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::LastMoved))),
            }),
        },
    )
}

/// Wicked Reward — {1}{B}. A creature's corpse buys +4/+2.
pub fn wicked_reward() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        ..instant(
            "Wicked Reward",
            cost(&[generic(1), b()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(4),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Wind Shear — {2}{G}. Grounds and shrinks the attacking fliers.
pub fn wind_shear() -> CardDefinition {
    instant(
        "Wind Shear",
        cost(&[generic(2), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying)),
                ),
                power: Value::Const(-2),
                toughness: Value::Const(-2),
                duration: Duration::EndOfTurn,
            },
            Effect::LoseKeyword {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying)),
                ),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Summer Bloom — {1}{G}. Three extra land drops.
pub fn summer_bloom() -> CardDefinition {
    sorcery(
        "Summer Bloom",
        cost(&[generic(1), g()]),
        Effect::GrantExtraLandPlay { who: PlayerRef::You, count: Value::Const(3) },
    )
}

/// Kaervek's Spite — {B}{B}{B}. Everything you own for five life.
pub fn kaerveks_spite() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![
            AdditionalCastCost::SacrificeAll { filter: R::Permanent },
            AdditionalCastCost::Discard { count: 7, filter: None },
        ],
        ..instant(
            "Kaervek's Spite",
            cost(&[b(), b(), b()]),
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(5),
            },
        )
    }
}

/// Solfatara — {2}{R}. Locks a land drop and cantrips next upkeep.
pub fn solfatara() -> CardDefinition {
    instant(
        "Solfatara",
        cost(&[generic(2), r()]),
        Effect::Seq(vec![
            Effect::PlayerCantPlayLandsThisTurn { player: PlayerRef::Target(0) },
            Effect::AtNextTurnsUpkeep { body: Box::new(draw(1)) },
        ]),
    )
}

/// Feral Instinct — {1}{G}. A small pump that cantrips next upkeep.
pub fn feral_instinct() -> CardDefinition {
    instant(
        "Feral Instinct",
        cost(&[generic(1), g()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::ONE,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            Effect::AtNextTurnsUpkeep { body: Box::new(draw(1)) },
        ]),
    )
}

/// Hope Charm — {W}. First strike, two life, or an Aura.
pub fn hope_charm() -> CardDefinition {
    instant(
        "Hope Charm",
        cost(&[w()]),
        Effect::ChooseMode(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            Effect::Destroy {
                what: target_filtered(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura)),
            },
        ]),
    )
}

/// Funeral Charm — {B}. A discard, a shrink, or swampwalk.
pub fn funeral_charm() -> CardDefinition {
    instant(
        "Funeral Charm",
        cost(&[b()]),
        Effect::ChooseMode(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Landwalk(LandType::Swamp),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Hearth Charm — {R}. Artifact removal, a team pump, or evasion.
pub fn hearth_charm() -> CardDefinition {
    instant(
        "Hearth Charm",
        cost(&[r()]),
        Effect::ChooseMode(vec![
            Effect::Destroy {
                what: target_filtered(R::Artifact.and(R::Creature)),
            },
            Effect::PumpPT {
                what: Selector::EachPermanent(R::Creature.and(R::IsAttacking)),
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::PowerAtMost(2))),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Emerald Charm — {G}. An untap, enchantment removal, or grounding.
pub fn emerald_charm() -> CardDefinition {
    instant(
        "Emerald Charm",
        cost(&[g()]),
        Effect::ChooseMode(vec![
            Effect::Untap { what: target_filtered(R::Permanent), up_to: None },
            Effect::Destroy {
                what: target_filtered(
                    R::Enchantment
                        .and(R::HasEnchantmentSubtype(EnchantmentSubtype::Aura).negate()),
                ),
            },
            Effect::LoseKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Rock Slide — {X}{R}. X damage split among grounded combatants.
pub fn rock_slide() -> CardDefinition {
    instant(
        "Rock Slide",
        cost(&[crate::mana::x(), r()]),
        Effect::DealDamageDivided {
            total: Value::XFromCost,
            filter: R::Creature
                .and(R::IsAttacking.or(R::IsBlocking))
                .and(R::HasKeyword(Keyword::Flying).negate()),
            max_targets: 8,
            retaliate_to_source: false,
        },
    )
}

/// Remedy — {1}{W}. Five points of prevention, split as you like.
pub fn remedy() -> CardDefinition {
    instant(
        "Remedy",
        cost(&[generic(1), w()]),
        Effect::PreventNextDamageDivided {
            total: Value::Const(5),
            filter: R::Any,
            max_targets: 5,
        },
    )
}

/// Honorable Passage — {1}{W}. Prevents the next hit, and red sources eat it.
pub fn honorable_passage() -> CardDefinition {
    instant(
        "Honorable Passage",
        cost(&[generic(1), w()]),
        Effect::PreventNextDamageFromChosenSource {
            filter: R::Any,
            reflect: false,
            to: None,
            gain_life: false,
            redirect_to: None,
            whole_turn: false,
        },
    )
}

/// Teferi's Puzzle Box — {4}. Everyone's hand churns every draw step.
pub fn teferis_puzzle_box() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::AnyPlayer),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::ActivePlayer,
                body: Box::new(Effect::BottomHandThenDrawThatMany { who: PlayerRef::You }),
            },
        }],
        ..artifact("Teferi's Puzzle Box", cost(&[generic(4)]), vec![])
    }
}

/// Dragon Mask — {3}. A pump that sends the creature home afterwards.
pub fn dragon_mask() -> CardDefinition {
    artifact(
        "Dragon Mask",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::LastMoved))),
                    }),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Gossamer Chains — {W}{W}. Recurring combat-damage prevention.
pub fn gossamer_chains() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            return_self_cost: true,
            effect: Effect::PreventCombatDamageByTargetThisTurn {
                target: target_filtered(R::Creature.and(R::IsUnblocked)),
            },
            ..Default::default()
        }],
        ..enchantment("Gossamer Chains", cost(&[w(), w()]))
    }
}

/// Fireblast-style closer: Talruum Champion — {4}{R} 3/3 first striker that
/// strips first strike from whatever it meets.
pub fn talruum_champion() -> CardDefinition {
    let strip = || Effect::LoseKeyword {
        what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
        keyword: Keyword::FirstStrike,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: strip(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: strip(),
            },
        ],
        ..creature("Talruum Champion", cost(&[generic(4), r()]), vec![CreatureType::Minotaur], 3, 3)
    }
}

/// Dwarven Vigilantes — {2}{R} 2/2 that trades its damage for a shot.
pub fn dwarven_vigilantes() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![crate::effect::shortcut::on_unblocked(
            Effect::MayDealPowerThenNoCombatDamage {
                dealer: Selector::This,
                to: target_filtered(R::Creature),
            },
        )],
        ..creature("Dwarven Vigilantes", cost(&[generic(2), r()]), vec![CreatureType::Dwarf], 2, 2)
    }
}

/// Anvil of Bogardan — {2}. Nobody discards to hand size, everybody loots.
pub fn anvil_of_bogardan() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Draw), EventScope::AnyPlayer),
            effect: Effect::EachPlayerDoes {
                who: PlayerRef::ActivePlayer,
                body: Box::new(Effect::Seq(vec![
                    draw(1),
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                ])),
            },
        }],
        ..artifact("Anvil of Bogardan", cost(&[generic(2)]), vec![])
    }
}

/// Zhalfirin Crusader — {1}{W}{W} 2/2 flanker that deflects damage a point at
/// a time.
pub fn zhalfirin_crusader() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: Some(Selector::This),
                gain_life: false,
                redirect_to: Some(target_any()),
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Zhalfirin Crusader",
            cost(&[generic(1), w(), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Teferi's Honor Guard — {2}{W} 2/2 flanker that can blink out of trouble.
pub fn teferis_honor_guard() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
            ..Default::default()
        }],
        ..creature(
            "Teferi's Honor Guard",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Shimmering Efreet — {2}{U} 2/2 phasing flier that drags something out with
/// it each time it returns.
pub fn shimmering_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Phasing],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PhasesIn, EventScope::SelfSource),
            effect: Effect::PhaseOut {
                what: target_filtered(R::Creature),
                until_source_leaves: false,
            },
        }],
        ..creature("Shimmering Efreet", cost(&[generic(2), u()]), vec![CreatureType::Efreet], 2, 2)
    }
}

/// Python — {1}{B}{B} 3/2 Snake.
pub fn python() -> CardDefinition {
    creature("Python", cost(&[generic(1), b(), b()]), vec![CreatureType::Snake], 3, 2)
}

/// Raging Gorilla — {2}{R} 2/3 that swings to +2/-2 whenever it blocks or
/// becomes blocked.
pub fn raging_gorilla() -> CardDefinition {
    let swing = || Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(2),
        toughness: Value::Const(-2),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: swing(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource),
                effect: swing(),
            },
        ],
        ..creature("Raging Gorilla", cost(&[generic(2), r()]), vec![CreatureType::Ape], 2, 3)
    }
}

/// Suq'Ata Assassin — {1}{B}{B} 1/1 with fear that poisons on an unblocked
/// attack.
pub fn suqata_assassin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AttacksAndIsntBlocked, EventScope::SelfSource),
            effect: Effect::AddPoison {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(1),
            },
        }],
        ..creature(
            "Suq'Ata Assassin",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Human, CreatureType::Assassin],
            1,
            1,
        )
    }
}

/// Talruum Piper — {4}{R} 3/3 that every flier able to block must block.
pub fn talruum_piper() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::AllMustBlock],
        ..creature("Talruum Piper", cost(&[generic(4), r()]), vec![CreatureType::Minotaur], 3, 3)
    }
}

/// Waterspout Djinn — {2}{U}{U} 4/4 flier; each upkeep you bounce an untapped
/// Island or sacrifice it.
pub fn waterspout_djinn() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: your_upkeep(),
            effect: Effect::SacrificeSourceUnlessCost {
                cost: WardCost::ReturnMatchingToHand(
                    Box::new(R::HasLandType(LandType::Island).and(R::Untapped)),
                    1,
                ),
            },
        }],
        ..creature("Waterspout Djinn", cost(&[generic(2), u(), u()]), vec![CreatureType::Djinn], 4, 4)
    }
}

/// Giant Caterpillar — {3}{G} 3/3; sacrifice it for a 1/1 flying Butterfly at
/// the next end step.
pub fn giant_caterpillar() -> CardDefinition {
    let butterfly = crate::card::TokenDefinition {
        name: "Butterfly".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Insect], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_cost: true,
            effect: Effect::DelayUntil {
                kind: crate::effect::DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: butterfly,
                }),
            },
            ..Default::default()
        }],
        ..creature("Giant Caterpillar", cost(&[generic(3), g()]), vec![CreatureType::Insect], 3, 3)
    }
}

/// Kyscu Drake — {3}{G} 2/2 flier that can pump its toughness once a turn.
/// (The Viashivan Dragon assembly ability is dropped.)
pub fn kyscu_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ZERO,
                toughness: Value::ONE,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Kyscu Drake", cost(&[generic(3), g()]), vec![CreatureType::Drake], 2, 2)
    }
}

/// Necrosavant — {3}{B}{B}{B} 5/5 that buys itself back out of the graveyard
/// during your upkeep by eating a creature.
pub fn necrosavant() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b(), b()]),
            from_graveyard: true,
            sac_other_filter: Some((R::Creature, 1)),
            condition: Some(Predicate::CurrentStepIs(TurnStep::Upkeep)),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Necrosavant",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Zombie, CreatureType::Giant],
            5,
            5,
        )
    }
}

/// Parapet — {1}{W} Enchantment. Creatures you control get +0/+1.
pub fn parapet() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                power: 0,
                toughness: 1,
            },
        }],
        ..enchantment("Parapet", cost(&[generic(1), w()]))
    }
}

/// Mortal Wound — {G} Aura. When enchanted creature is dealt damage, destroy it.
pub fn mortal_wound() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::EnchantedBySource),
            effect: Effect::Destroy { what: Selector::AttachedTo(Box::new(Selector::This)) },
        }],
        ..aura("Mortal Wound", cost(&[g()]), EquipBonus::default())
    }
}

/// Mystic Veil — {1}{U} Aura granting shroud.
pub fn mystic_veil() -> CardDefinition {
    aura(
        "Mystic Veil",
        cost(&[generic(1), u()]),
        EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() },
    )
}

/// Relic Ward — {1}{W} Aura on an artifact, granting it shroud.
pub fn relic_ward() -> CardDefinition {
    CardDefinition {
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Artifact) },
        ..aura(
            "Relic Ward",
            cost(&[generic(1), w()]),
            EquipBonus { keywords: vec![Keyword::Shroud], ..Default::default() },
        )
    }
}

/// Betrayal — {U} Aura on an opponent's creature; you draw whenever it taps.
pub fn betrayal() -> CardDefinition {
    CardDefinition {
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::EnchantedBySource),
            effect: draw(1),
        }],
        ..aura("Betrayal", cost(&[u()]), EquipBonus::default())
    }
}

/// Scalebane's Elite — {3}{G}{W} 4/4 with protection from black.
pub fn scalebanes_elite() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Protection(Color::Black)],
        ..creature(
            "Scalebane's Elite",
            cost(&[generic(3), g(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Phyrexian Marauder — {X} 0/0 Construct entering with X +1/+1 counters that
/// can't block. (The "can't attack unless you pay {1} per counter" tax is
/// dropped.)
pub fn phyrexian_marauder() -> CardDefinition {
    CardDefinition {
        cost: cost(&[crate::mana::x()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::CantBlock],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..creature(
            "Phyrexian Marauder",
            cost(&[crate::mana::x()]),
            vec![CreatureType::Phyrexian, CreatureType::Construct],
            0,
            0,
        )
    }
}

/// Miraculous Recovery — {4}{W} Instant. Reanimate a creature card with a
/// +1/+1 counter on it.
pub fn miraculous_recovery() -> CardDefinition {
    instant(
        "Miraculous Recovery",
        cost(&[generic(4), w()]),
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::LastMoved,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Quicksand — a colorless land that eats a ground attacker.
pub fn quicksand() -> CardDefinition {
    CardDefinition {
        name: "Quicksand",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::PumpPT {
                    what: target_filtered(
                        R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Flying).negate()),
                    ),
                    power: Value::Const(-1),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Griffin Canyon — {T}: Add {C}; {T}: untap a Griffin and pump it.
pub fn griffin_canyon() -> CardDefinition {
    CardDefinition {
        name: "Griffin Canyon",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Untap {
                        what: target_filtered(R::HasCreatureType(CreatureType::Griffin)),
                        up_to: None,
                    },
                    Effect::PumpPT {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Magma Mine — {1} Artifact. Charge it up, then sacrifice it for that much
/// damage.
pub fn magma_mine() -> CardDefinition {
    artifact(
        "Magma Mine",
        cost(&[generic(1)]),
        vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Pressure,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: target_any(),
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Pressure,
                    },
                },
                ..Default::default()
            },
        ],
    )
}

/// Snake Basket — {4} Artifact. {X}, sacrifice it: X 1/1 green Snakes.
pub fn snake_basket() -> CardDefinition {
    let snake = crate::card::TokenDefinition {
        name: "Snake".into(),
        power: 1,
        toughness: 1,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        ..Default::default()
    };
    artifact(
        "Snake Basket",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[crate::mana::x()]),
            sac_cost: true,
            sorcery_speed: true,
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::XFromCost,
                definition: snake,
            },
            ..Default::default()
        }],
    )
}

/// Righteous War — {1}{W}{B} Enchantment. Your white creatures have protection
/// from black and your black creatures have protection from white.
pub fn righteous_war() -> CardDefinition {
    let grant = |have: Color, from: Color| StaticAbility {
        description: "Righteous War grants protection along the colour line.",
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::HasColor(have)),
            ),
            keyword: Keyword::Protection(from),
        },
    };
    CardDefinition {
        static_abilities: vec![
            grant(Color::White, Color::Black),
            grant(Color::Black, Color::White),
        ],
        ..enchantment("Righteous War", cost(&[generic(1), w(), b()]))
    }
}

/// Suleiman's Legacy — {R}{W} Enchantment. Wipes Djinns and Efreets on entry
/// and kills each one that enters afterwards.
pub fn suleimans_legacy() -> CardDefinition {
    let djinn_or_efreet = || {
        R::HasCreatureType(CreatureType::Djinn).or(R::HasCreatureType(CreatureType::Efreet))
    };
    CardDefinition {
        triggered_abilities: vec![
            etb(Effect::DestroyNoRegen {
                what: Selector::EachPermanent(djinn_or_efreet()),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: djinn_or_efreet(),
                    }),
                effect: Effect::DestroyNoRegen { what: Selector::TriggerSource },
            },
        ],
        ..enchantment("Suleiman's Legacy", cost(&[r(), w()]))
    }
}

/// Vanishing — {U} Aura. {U}{U}: the enchanted creature phases out.
pub fn vanishing() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::PhaseOut {
                what: Selector::AttachedTo(Box::new(Selector::This)),
                until_source_leaves: false,
            },
            ..Default::default()
        }],
        ..aura("Vanishing", cost(&[u()]), EquipBonus::default())
    }
}

/// Death Watch — {B} Aura. When the enchanted creature dies its controller
/// loses life equal to its power and you gain life equal to its toughness.
pub fn death_watch() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::ControllerOf(Box::new(
                        Selector::TriggerSource,
                    ))),
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
                },
            ]),
        }],
        ..aura("Death Watch", cost(&[b()]), EquipBonus::default())
    }
}

/// Flooded Shoreline — {U}{U} Enchantment. {U}{U}, return two Islands you
/// control to hand: bounce target creature.
pub fn flooded_shoreline() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            bounce_other_filter: Some((R::HasLandType(LandType::Island), 2)),
            effect: Effect::Move {
                what: target_filtered(R::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            ..Default::default()
        }],
        ..enchantment("Flooded Shoreline", cost(&[u(), u()]))
    }
}

/// Righteous Aura — {1}{W} Enchantment. {W}, pay 2 life: prevent the next
/// damage a source of your choice would deal to you this turn.
pub fn righteous_aura() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            life_cost: 2,
            effect: Effect::PreventNextDamageFromChosenSource {
                filter: R::Any,
                reflect: false,
                to: None,
                gain_life: false,
                redirect_to: None,
                whole_turn: false,
            },
            ..Default::default()
        }],
        ..enchantment("Righteous Aura", cost(&[generic(1), w()]))
    }
}

/// Quirion Druid — {2}{G} 1/2. {G}, {T}: a land becomes a 2/2 green creature
/// that's still a land.
pub fn quirion_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::BecomeCreature {
                what: target_filtered(R::Land),
                power: Value::Const(2),
                toughness: Value::Const(2),
                creature_types: vec![],
                keywords: vec![],
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..creature(
            "Quirion Druid",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            1,
            2,
        )
    }
}

/// Rainbow Efreet — {3}{U} 3/1 flier that can phase itself out.
pub fn rainbow_efreet() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u(), u()]),
            effect: Effect::PhaseOut { what: Selector::This, until_source_leaves: false },
            ..Default::default()
        }],
        ..creature("Rainbow Efreet", cost(&[generic(3), u()]), vec![CreatureType::Efreet], 3, 1)
    }
}

/// Knight of Valor — {2}{W} 2/2 with flanking; once a turn {1}{W} shrinks every
/// non-flanking creature blocking it.
pub fn knight_of_valor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), w()]),
            once_per_turn: true,
            effect: Effect::PumpPT {
                what: Selector::BlockingCreatures,
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Knight of Valor",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Knight of the Mists — {2}{U} 2/2 with flanking whose ETB destroys a Knight
/// unless you pay {U}.
pub fn knight_of_the_mists() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flanking],
        triggered_abilities: vec![etb(Effect::PayManaOrElse {
            mana_cost: cost(&[u()]),
            otherwise: Box::new(Effect::DestroyNoRegen {
                what: target_filtered(R::HasCreatureType(CreatureType::Knight)),
            }),
        })],
        ..creature(
            "Knight of the Mists",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Matopi Golem — {5} 3/3 that regenerates for {1} and shrinks each time.
pub fn matopi_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Regenerated, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::MinusOneMinusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Matopi Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}

/// Brood of Cockroaches — {1}{B} 1/1 that comes back at the next end step for
/// a life.
pub fn brood_of_cockroaches() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::DelayUntil {
                kind: crate::effect::DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
            },
        }],
        ..creature(
            "Brood of Cockroaches",
            cost(&[generic(1), b()]),
            vec![CreatureType::Insect],
            1,
            1,
        )
    }
}

/// Vampirism — {1}{B} Aura. The host grows per other creature you control while
/// the rest of your team shrinks. (The delayed ETB draw is dropped.)
pub fn vampirism() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                ),
                power: -1,
                toughness: -1,
            },
        }],
        ..aura(
            "Vampirism",
            cost(&[generic(1), b()]),
            EquipBonus {
                scale: Some(crate::card::EquipScale {
                    filter: R::Creature.and(R::OtherThanSource),
                    per_power: 1,
                    per_toughness: 1,
                    ..Default::default()
                }),
                ..Default::default()
            },
        )
    }
}
