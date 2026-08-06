//! Stronghold (STH) — the Tempest block's second set. Flowstone pumps,
//! Spikes, the en-Kor damage-shifters, buyback spells and the Volrath
//! artifacts. Tests in `classic_sets/sth`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EquipBonus,
    EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{deal, draw, target_filtered, target_n};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Predicate, Selector, StaticEffect, Value, ZoneDest,
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

/// "{cost}: This creature gets +p/+t until end of turn."
fn self_pump(c: ManaCost, p: i32, t: i32) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: c,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(p),
            toughness: Value::Const(t),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

// ── Flowstone / pump creatures ──────────────────────────────────────────────

/// Flowstone Shambler — {2}{R} 2/2. {R}: +1/-1.
pub fn flowstone_shambler() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[r()]), 1, -1)],
        ..creature("Flowstone Shambler", cost(&[generic(2), r()]), vec![CreatureType::Beast], 2, 2)
    }
}

/// Flowstone Hellion — {4}{R} 3/3 haste. {0}: +1/-1.
pub fn flowstone_hellion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![self_pump(ManaCost::new(vec![]), 1, -1)],
        ..creature(
            "Flowstone Hellion",
            cost(&[generic(4), r()]),
            vec![CreatureType::Hellion, CreatureType::Beast],
            3,
            3,
        )
    }
}

/// Flowstone Mauler — {4}{R}{R} 4/5 trample. {R}: +1/-1.
pub fn flowstone_mauler() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![self_pump(cost(&[r()]), 1, -1)],
        ..creature(
            "Flowstone Mauler",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Beast],
            4,
            5,
        )
    }
}

/// Furnace Spirit — {2}{R} 1/1 haste. {R}: +1/+0.
pub fn furnace_spirit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![self_pump(cost(&[r()]), 1, 0)],
        ..creature("Furnace Spirit", cost(&[generic(2), r()]), vec![CreatureType::Spirit], 1, 1)
    }
}

/// Honor Guard — {W} 1/1. {W}: +0/+1.
pub fn honor_guard() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![self_pump(cost(&[w()]), 0, 1)],
        ..creature(
            "Honor Guard",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Dungeon Shade — {3}{B} 1/1 flier. {B}: +1/+1.
pub fn dungeon_shade() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![self_pump(cost(&[b()]), 1, 1)],
        ..creature(
            "Dungeon Shade",
            cost(&[generic(3), b()]),
            vec![CreatureType::Shade, CreatureType::Spirit],
            1,
            1,
        )
    }
}

/// Carnassid — {4}{G}{G} 5/4 trample that regenerates for {1}{G}.
pub fn carnassid() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Regenerate { what: Selector::This },
            ..Default::default()
        }],
        ..creature("Carnassid", cost(&[generic(4), g(), g()]), vec![CreatureType::Beast], 5, 4)
    }
}

// ── Small utility creatures ─────────────────────────────────────────────────

/// Skyshroud Falcon — {1}{W} 1/1 flying, vigilance.
pub fn skyshroud_falcon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        ..creature("Skyshroud Falcon", cost(&[generic(1), w()]), vec![CreatureType::Bird], 1, 1)
    }
}

/// Skyshroud Troopers — {3}{G} 3/3 that taps for {G}.
pub fn skyshroud_troopers() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
            ..Default::default()
        }],
        ..creature(
            "Skyshroud Troopers",
            cost(&[generic(3), g()]),
            vec![CreatureType::Elf, CreatureType::Druid, CreatureType::Warrior],
            3,
            3,
        )
    }
}

/// Skyshroud Archer — {G} 1/1. {T}: target creature with flying gets -1/-1.
pub fn skyshroud_archer() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::HasKeyword(Keyword::Flying)),
                },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Skyshroud Archer",
            cost(&[g()]),
            vec![CreatureType::Elf, CreatureType::Archer],
            1,
            1,
        )
    }
}

/// Rabid Rats — {1}{B} 1/1. {T}: target blocking creature gets -1/-1.
pub fn rabid_rats() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::IsBlocking) },
                power: Value::Const(-1),
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Rabid Rats", cost(&[generic(1), b()]), vec![CreatureType::Rat], 1, 1)
    }
}

/// Dauthi Trapper — {2}{B} 1/1. {T}: target creature gains shadow.
pub fn dauthi_trapper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Shadow,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Dauthi Trapper",
            cost(&[generic(2), b()]),
            vec![CreatureType::Dauthi, CreatureType::Minion],
            1,
            1,
        )
    }
}

/// Tidal Warrior — {U} 1/1. {T}: target land becomes an Island until end of
/// turn.
pub fn tidal_warrior() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeBasicLand {
                what: target_filtered(R::Land),
                land_type: LandType::Island,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Tidal Warrior",
            cost(&[u()]),
            vec![CreatureType::Merfolk, CreatureType::Warrior],
            1,
            1,
        )
    }
}

/// Morgue Thrull — {2}{B} 2/2. Sacrifice: mill three cards.
pub fn morgue_thrull() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Mill { who: Selector::You, amount: Value::Const(3) },
            ..Default::default()
        }],
        ..creature("Morgue Thrull", cost(&[generic(2), b()]), vec![CreatureType::Thrull], 2, 2)
    }
}

/// Stronghold Assassin — {1}{B}{B} 2/1. {T}, sacrifice a creature: destroy
/// target nonblack creature.
pub fn stronghold_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Destroy {
                what: target_filtered(R::Creature.and(R::HasColor(Color::Black).negate())),
            },
            ..Default::default()
        }],
        ..creature(
            "Stronghold Assassin",
            cost(&[generic(1), b(), b()]),
            vec![CreatureType::Phyrexian, CreatureType::Zombie, CreatureType::Assassin],
            2,
            1,
        )
    }
}

/// Stronghold Taskmaster — {2}{B}{B} 4/3 that shrinks every *other* black
/// creature.
pub fn stronghold_taskmaster() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Other black creatures get -1/-1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasColor(Color::Black)).and(R::OtherThanSource),
                ),
                power: -1,
                toughness: -1,
            },
        }],
        ..creature(
            "Stronghold Taskmaster",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Giant, CreatureType::Minion],
            4,
            3,
        )
    }
}

/// Revenant — {4}{B} flier whose power and toughness are each the number of
/// creature cards in your graveyard.
pub fn revenant() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        dynamic_pt: Some(DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 0 }),
        ..creature("Revenant", cost(&[generic(4), b()]), vec![CreatureType::Spirit], 0, 0)
    }
}

/// Hermit Druid — {1}{G} 1/1. {G}, {T}: dig to a basic land, binning the
/// rest.
pub fn hermit_druid() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            tap_cost: true,
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: R::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(200),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::Graveyard,
            },
            ..Default::default()
        }],
        ..creature(
            "Hermit Druid",
            cost(&[generic(1), g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Dream Prowler — {2}{U}{U} 1/5 that can't be blocked while attacking alone.
pub fn dream_prowler() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This creature can't be blocked as long as it's attacking alone.",
            effect: StaticEffect::SelfHasKeywordIf {
                keyword: Keyword::Unblockable,
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttackingAlone,
                },
            },
        }],
        ..creature(
            "Dream Prowler",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Illusion],
            1,
            5,
        )
    }
}

/// Hammerhead Shark — {1}{U} 2/3 that can't attack unless the defender has an
/// Island.
pub fn hammerhead_shark() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CantAttackUnlessDefenderControlsLandType(LandType::Island)],
        ..creature("Hammerhead Shark", cost(&[generic(1), u()]), vec![CreatureType::Shark], 2, 3)
    }
}

/// Spindrift Drake — {U} 2/1 flier that wants {U} each upkeep.
pub fn spindrift_drake() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::SacrificeSourceUnlessPay { cost: cost(&[u()]) },
        }],
        ..creature("Spindrift Drake", cost(&[u()]), vec![CreatureType::Drake], 2, 1)
    }
}

// ── Walls ───────────────────────────────────────────────────────────────────

fn wall(name: &'static str, c: ManaCost, p: i32, t: i32) -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        ..creature(name, c, vec![CreatureType::Wall], p, t)
    }
}

/// Wall of Razors — {1}{R} 4/1 defender with first strike.
pub fn wall_of_razors() -> CardDefinition {
    let mut def = wall("Wall of Razors", cost(&[generic(1), r()]), 4, 1);
    def.keywords.push(Keyword::FirstStrike);
    def
}

/// Wall of Essence — {1}{W} 0/4 defender; combat damage it takes is life you
/// gain.
pub fn wall_of_essence() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtCombatDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..wall("Wall of Essence", cost(&[generic(1), w()]), 0, 4)
    }
}

/// Wall of Souls — {1}{B} 0/4 defender that mirrors the combat damage it
/// takes at an opponent.
pub fn wall_of_souls() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtCombatDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..wall("Wall of Souls", cost(&[generic(1), b()]), 0, 4)
    }
}

/// Wall of Tears — {1}{U} 0/4 defender that bounces what it blocks.
pub fn wall_of_tears() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Move {
                    what: Selector::BlockedAttacker,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::BlockedAttacker))),
                }),
            },
        }],
        ..wall("Wall of Tears", cost(&[generic(1), u()]), 0, 4)
    }
}

/// Shifting Wall — {X} artifact Wall that enters with X +1/+1 counters.
pub fn shifting_wall() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Defender],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        ..creature("Shifting Wall", cost(&[crate::mana::x()]), vec![CreatureType::Wall], 0, 0)
    }
}

// ── Spikes ──────────────────────────────────────────────────────────────────

/// "{2}, Remove a +1/+1 counter: put a +1/+1 counter on target creature."
fn spike_transfer() -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
        effect: Effect::AddCounter {
            what: target_filtered(R::Creature),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        ..Default::default()
    }
}

fn spike(name: &'static str, c: ManaCost, types: Vec<CreatureType>, counters: u32) -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(counters as i32))),
        activated_abilities: vec![spike_transfer()],
        ..creature(name, c, types, 0, 0)
    }
}

/// Spike Worker — {2}{G} 0/0 with two counters to hand out.
pub fn spike_worker() -> CardDefinition {
    spike("Spike Worker", cost(&[generic(2), g()]), vec![CreatureType::Spike], 2)
}

/// Spike Colony — {4}{G} 0/0 with four counters to hand out.
pub fn spike_colony() -> CardDefinition {
    spike("Spike Colony", cost(&[generic(4), g()]), vec![CreatureType::Spike], 4)
}

/// Spike Soldier — {2}{G}{G} 0/0; its counters also buy +2/+2 for the turn.
pub fn spike_soldier() -> CardDefinition {
    let mut def = spike(
        "Spike Soldier",
        cost(&[generic(2), g(), g()]),
        vec![CreatureType::Spike, CreatureType::Soldier],
        3,
    );
    def.activated_abilities.push(ActivatedAbility {
        remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    });
    def
}

/// Spike Breeder — {3}{G} 0/0; its counters buy counters or 1/1 Spikes.
pub fn spike_breeder() -> CardDefinition {
    let mut def =
        spike("Spike Breeder", cost(&[generic(3), g()]), vec![CreatureType::Spike], 3);
    def.activated_abilities.push(ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Spike".into(),
                colors: vec![Color::Green],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Spike],
                    ..Default::default()
                },
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        },
        ..Default::default()
    });
    def
}

/// Spitting Hydra — {3}{R}{R} 0/0 with four counters it spits as damage.
pub fn spitting_hydra() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            effect: deal(1, target_filtered(R::Creature)),
            ..Default::default()
        }],
        ..creature(
            "Spitting Hydra",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Hydra],
            0,
            0,
        )
    }
}

/// Mindwarper — {2}{B}{B} 0/0 with three counters that buy discards.
pub fn mindwarper() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            remove_counter_cost: Some((CounterType::PlusOnePlusOne, 1)),
            sorcery_speed: true,
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..creature("Mindwarper", cost(&[generic(2), b(), b()]), vec![CreatureType::Spirit], 0, 0)
    }
}

/// Sliver Queen — the five-colour 7/7 that mints 1/1 Slivers for {2}.
pub fn sliver_queen() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Sliver".into(),
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Sliver],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
            ..Default::default()
        }],
        ..creature(
            "Sliver Queen",
            cost(&[w(), u(), b(), r(), g()]),
            vec![CreatureType::Sliver],
            7,
            7,
        )
    }
}

/// Mogg Maniac — {1}{R} 1/1 that reflects the damage it takes.
pub fn mogg_maniac() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealtDamage, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::TriggerEventAmount,
            },
        }],
        ..creature("Mogg Maniac", cost(&[generic(1), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Warrior Angel — {4}{W}{W} 3/4 flier; all the damage it deals is life.
pub fn warrior_angel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamage, EventScope::SelfSource),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..creature(
            "Warrior Angel",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Angel, CreatureType::Warrior],
            3,
            4,
        )
    }
}

/// Mogg Bombers — {3}{R} 3/4 that blows up the moment anything else enters.
pub fn mogg_bombers() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::Seq(vec![
                Effect::SacrificePermanent { what: Selector::This },
                deal(3, target_filtered(R::Player.or(R::Planeswalker))),
            ]),
        }],
        ..creature("Mogg Bombers", cost(&[generic(3), r()]), vec![CreatureType::Goblin], 3, 4)
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Horn of Greed — {3} Artifact. Every land play draws its player a card.
pub fn horn_of_greed() -> CardDefinition {
    CardDefinition {
        name: "Horn of Greed",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::AnyPlayer),
            effect: Effect::Draw {
                who: Selector::Player(PlayerRef::TriggerEventPlayer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Heartstone — {3} Artifact. Creatures' activated abilities cost {1} less.
pub fn heartstone() -> CardDefinition {
    CardDefinition {
        name: "Heartstone",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Activated abilities of creatures cost {1} less to activate.",
            effect: StaticEffect::YourCreatureActivatedAbilitiesCostLess { amount: 1 },
        }],
        ..Default::default()
    }
}

/// Bullwhip — {4} Artifact. {2}, {T}: ping a creature and drag it into combat.
pub fn bullwhip() -> CardDefinition {
    CardDefinition {
        name: "Bullwhip",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                deal(1, target_filtered(R::Creature)),
                Effect::GrantKeyword {
                    what: target_n(0),
                    keyword: Keyword::MustAttack,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sword of the Chosen — {2} legendary Artifact. {T}: a legend gets +2/+2.
pub fn sword_of_the_chosen() -> CardDefinition {
    CardDefinition {
        name: "Sword of the Chosen",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::HasSupertype(Supertype::Legendary)) },
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Contemplation — {1}{W}{W} Enchantment. Every spell you cast gains 1 life.
pub fn contemplation() -> CardDefinition {
    CardDefinition {
        name: "Contemplation",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

/// Primal Rage — {1}{G} Enchantment. Your creatures have trample.
pub fn primal_rage() -> CardDefinition {
    CardDefinition {
        name: "Primal Rage",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
                keyword: Keyword::Trample,
            },
        }],
        ..Default::default()
    }
}

/// Heat of Battle — {1}{R} Enchantment. Every block singes the blocker's
/// controller.
pub fn heat_of_battle() -> CardDefinition {
    CardDefinition {
        name: "Heat of Battle",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Mortuary — {3}{B} Enchantment. Your dead creatures go back on top of your
/// library instead of staying dead.
pub fn mortuary() -> CardDefinition {
    CardDefinition {
        name: "Mortuary",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl),
            effect: Effect::Move {
                what: Selector::TriggerSource,
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::TriggerSource)),
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
        }],
        ..Default::default()
    }
}

/// Tortured Existence — {B} Enchantment. {B}, discard a creature card: buy a
/// creature card back from your graveyard.
pub fn tortured_existence() -> CardDefinition {
    CardDefinition {
        name: "Tortured Existence",
        cost: cost(&[b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            discard_cost: Some((R::Creature, 1)),
            effect: Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Rolling Stones — {1}{W} Enchantment. Walls can attack.
pub fn rolling_stones() -> CardDefinition {
    CardDefinition {
        name: "Rolling Stones",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Wall creatures can attack as though they didn't have defender.",
            effect: StaticEffect::LoseKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::HasCreatureType(CreatureType::Wall)),
                ),
                keyword: Keyword::Defender,
            },
        }],
        ..Default::default()
    }
}

/// Torment — {1}{B} Aura. Enchanted creature gets -3/-0.
pub fn torment() -> CardDefinition {
    CardDefinition {
        name: "Torment",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: -3, toughness: 0, ..Default::default() }),
        ..Default::default()
    }
}

/// Conviction — {1}{W} Aura. +1/+3, and {W} buys it back.
pub fn conviction() -> CardDefinition {
    CardDefinition {
        name: "Conviction",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(crate::card::EquipBonus { power: 1, toughness: 3, ..Default::default() }),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Death Stroke — {B}{B} Sorcery. Destroy target tapped creature.
pub fn death_stroke() -> CardDefinition {
    CardDefinition {
        name: "Death Stroke",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy { what: target_filtered(R::Creature.and(R::Tapped)) },
        ..Default::default()
    }
}

/// Ruination — {3}{R} Sorcery. Destroy all nonbasic lands.
pub fn ruination() -> CardDefinition {
    CardDefinition {
        name: "Ruination",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(R::Land.and(R::IsBasicLand.negate())),
        },
        ..Default::default()
    }
}

/// Mob Justice — {1}{R} Sorcery. Damage equal to your creature count.
pub fn mob_justice() -> CardDefinition {
    CardDefinition {
        name: "Mob Justice",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DealDamage {
            to: target_filtered(R::Player.or(R::Planeswalker)),
            amount: Value::CreatureCountControlledBy(PlayerRef::You),
        },
        ..Default::default()
    }
}

/// Sift — {3}{U} Sorcery. Draw three, discard one.
pub fn sift() -> CardDefinition {
    CardDefinition {
        name: "Sift",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            draw(3),
            Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
        ]),
        ..Default::default()
    }
}

/// Leap — {U} Instant. Flying until end of turn, then draw.
pub fn leap() -> CardDefinition {
    CardDefinition {
        name: "Leap",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Crossbow Ambush — {G} Instant. Your creatures gain reach.
pub fn crossbow_ambush() -> CardDefinition {
    CardDefinition {
        name: "Crossbow Ambush",
        cost: cost(&[g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::GrantKeyword {
            what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
            keyword: Keyword::Reach,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Temper — {X}{1}{W} Instant. Prevent the next X damage to a creature and
/// bank each point prevented as a +1/+1 counter.
pub fn temper() -> CardDefinition {
    CardDefinition {
        name: "Temper",
        cost: cost(&[crate::mana::x(), generic(1), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PreventNextDamageWithCounters {
            target: target_filtered(R::Creature),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Flame Wave — {3}{R}{R}{R}{R} Sorcery. Four to a player and to each
/// creature they control.
pub fn flame_wave() -> CardDefinition {
    CardDefinition {
        name: "Flame Wave",
        cost: cost(&[generic(3), r(), r(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            deal(4, target_filtered(R::Player.or(R::Planeswalker))),
            Effect::DealDamage {
                to: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                amount: Value::Const(4),
            },
        ]),
        ..Default::default()
    }
}

// ── Buyback spells ──────────────────────────────────────────────────────────

/// Brush with Death — {2}{B} Sorcery with buyback {2}{B}{B}. Drain 2.
pub fn brush_with_death() -> CardDefinition {
    CardDefinition {
        name: "Brush with Death",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(2), b(), b()]))],
        effect: Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::Const(2),
            },
            Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Change of Heart — {W} Instant with buyback {3}. A creature can't attack.
pub fn change_of_heart() -> CardDefinition {
    CardDefinition {
        name: "Change of Heart",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::CantAttackThisTurn { what: target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// Constant Mists — {1}{G} Instant. Fog, with a land sacrifice for buyback.
pub fn constant_mists() -> CardDefinition {
    CardDefinition {
        name: "Constant Mists",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(ManaCost::new(vec![]))],
        buyback_additional_cost: Some(crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Land,
            count: 1,
        }),
        effect: Effect::PreventAllCombatDamageThisTurn,
        ..Default::default()
    }
}

/// Fanning the Flames — {X}{R}{R} Sorcery with buyback {3}. X damage.
pub fn fanning_the_flames() -> CardDefinition {
    CardDefinition {
        name: "Fanning the Flames",
        cost: cost(&[crate::mana::x(), r(), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::DealDamage {
            to: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Lab Rats — {B} Sorcery with buyback {4}. One 1/1 Rat per cast.
pub fn lab_rats() -> CardDefinition {
    CardDefinition {
        name: "Lab Rats",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(4)]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Rat".into(),
                colors: vec![Color::Black],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
                power: 1,
                toughness: 1,
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Mind Games — {U} Instant with buyback {2}{U}. Tap a permanent.
pub fn mind_games() -> CardDefinition {
    CardDefinition {
        name: "Mind Games",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(cost(&[generic(2), u()]))],
        effect: Effect::Tap {
            what: target_filtered(R::Artifact.or(R::Creature).or(R::Land)),
        },
        ..Default::default()
    }
}

/// Mind Peel — {B} Sorcery with buyback {2}{B}{B}. A discard per cast.
pub fn mind_peel() -> CardDefinition {
    CardDefinition {
        name: "Mind Peel",
        cost: cost(&[b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(2), b(), b()]))],
        effect: Effect::Discard {
            who: Selector::Player(PlayerRef::Target(0)),
            amount: Value::ONE,
            random: false,
        },
        ..Default::default()
    }
}

/// Seething Anger — {R} Sorcery with buyback {3}. +3/+0.
pub fn seething_anger() -> CardDefinition {
    CardDefinition {
        name: "Seething Anger",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::PumpPT {
            what: target_filtered(R::Creature),
            power: Value::Const(3),
            toughness: Value::ZERO,
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Verdant Touch — {1}{G} Sorcery with buyback {3}. A land becomes a 2/2 that
/// is still a land, indefinitely.
pub fn verdant_touch() -> CardDefinition {
    CardDefinition {
        name: "Verdant Touch",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Buyback(cost(&[generic(3)]))],
        effect: Effect::BecomeCreature {
            what: target_filtered(R::Land),
            power: Value::Const(2),
            toughness: Value::Const(2),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::Permanent,
        },
        ..Default::default()
    }
}

// ── The rest of the spells ──────────────────────────────────────────────────

/// Cannibalize — {1}{B} Sorcery. One of a player's creatures eats the other.
pub fn cannibalize() -> CardDefinition {
    CardDefinition {
        name: "Cannibalize",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Exile { what: Selector::Target(0) },
            Effect::AddCounter {
                what: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Elven Rite — {1}{G} Sorcery. Two +1/+1 counters split over one or two
/// creatures.
pub fn elven_rite() -> CardDefinition {
    CardDefinition {
        name: "Elven Rite",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DistributeCounters {
            total: Value::Const(2),
            counter: CounterType::PlusOnePlusOne,
            filter: R::Creature,
            max_targets: 2,
        },
        ..Default::default()
    }
}

/// Scapegoat — {W} Instant. Sacrifice a creature to bounce any number of the
/// rest.
pub fn scapegoat() -> CardDefinition {
    CardDefinition {
        name: "Scapegoat",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        effect: Effect::ApplyToTargets {
            min_targets: 0,
            max_targets: 6,
            filter: R::Creature.and(R::ControlledByYou),
            effect: Box::new(Effect::Move {
                what: target_n(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(target_n(0)))),
            }),
        },
        ..Default::default()
    }
}

/// Mask of the Mimic — {U} Instant. Sacrifice a creature to fetch a copy of
/// any nontoken one.
pub fn mask_of_the_mimic() -> CardDefinition {
    CardDefinition {
        name: "Mask of the Mimic",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        additional_cast_cost: vec![crate::card::AdditionalCastCost::SacrificePermanent {
            filter: R::Creature,
            count: 1,
        }],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::SameNameAsTarget.and(R::Creature),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            count: Value::ONE,
        },
        ..Default::default()
    }
}

/// Mogg Infestation — {3}{R}{R} Sorcery. Wipe a player's board and pay them
/// back in Goblins.
pub fn mogg_infestation() -> CardDefinition {
    CardDefinition {
        name: "Mogg Infestation",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
            },
            Effect::CreateToken {
                who: PlayerRef::Target(0),
                count: Value::Times(
                    Box::new(Value::CreaturesDiedThisTurn(PlayerRef::Target(0))),
                    Box::new(Value::Const(2)),
                ),
                definition: TokenDefinition {
                    name: "Goblin".into(),
                    colors: vec![Color::Red],
                    card_types: vec![CardType::Creature],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Goblin],
                        ..Default::default()
                    },
                    power: 1,
                    toughness: 1,
                    ..Default::default()
                },
            },
        ]),
        ..Default::default()
    }
}

/// Provoke — {1}{G} Instant. Untap a creature you don't control and force it
/// to block, then draw.
pub fn provoke() -> CardDefinition {
    CardDefinition {
        name: "Provoke",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                up_to: None,
            },
            Effect::MustBlockSource { what: target_n(0) },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Ransack — {3}{U} Sorcery. Sort the top five of a library.
pub fn ransack() -> CardDefinition {
    CardDefinition {
        name: "Ransack",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::RearrangeTop { who: PlayerRef::Target(0), amount: Value::Const(5) },
        ..Default::default()
    }
}

// ── The en-Kor damage shifters ──────────────────────────────────────────────

/// "{0}: The next 1 damage that would be dealt to this creature this turn is
/// dealt to target creature you control instead."
fn en_kor_shield() -> ActivatedAbility {
    ActivatedAbility {
        effect: Effect::RedirectNextDamage {
            target: Selector::This,
            to: target_filtered(R::Creature.and(R::ControlledByYou)),
            amount: Value::ONE,
        },
        ..Default::default()
    }
}

/// Nomads en-Kor — {W} 1/1 that shrugs damage onto your other creatures.
pub fn nomads_en_kor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![en_kor_shield()],
        ..creature(
            "Nomads en-Kor",
            cost(&[w()]),
            vec![CreatureType::Kor, CreatureType::Nomad, CreatureType::Soldier],
            1,
            1,
        )
    }
}

/// Warrior en-Kor — {W}{W} 2/2 with the en-Kor shield.
pub fn warrior_en_kor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![en_kor_shield()],
        ..creature(
            "Warrior en-Kor",
            cost(&[w(), w()]),
            vec![CreatureType::Kor, CreatureType::Warrior, CreatureType::Knight],
            2,
            2,
        )
    }
}

/// Spirit en-Kor — {3}{W} 2/2 flier with the en-Kor shield.
pub fn spirit_en_kor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![en_kor_shield()],
        ..creature(
            "Spirit en-Kor",
            cost(&[generic(3), w()]),
            vec![CreatureType::Kor, CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Lancers en-Kor — {3}{W}{W} 3/3 trampler with the en-Kor shield.
pub fn lancers_en_kor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        activated_abilities: vec![en_kor_shield()],
        ..creature(
            "Lancers en-Kor",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Kor, CreatureType::Soldier],
            3,
            3,
        )
    }
}

/// Shaman en-Kor — {1}{W} 1/2 that both sheds damage and soaks it up.
pub fn shaman_en_kor() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            en_kor_shield(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1), w()]),
                effect: Effect::RedirectNextDamage {
                    target: target_filtered(R::Creature),
                    to: Selector::This,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Shaman en-Kor",
            cost(&[generic(1), w()]),
            vec![CreatureType::Kor, CreatureType::Cleric, CreatureType::Shaman],
            1,
            2,
        )
    }
}

// ── The rest of the creatures ───────────────────────────────────────────────

/// Crovax the Cursed — {2}{B}{B} 0/0 legend that eats a creature each upkeep
/// or wastes away.
pub fn crovax_the_cursed() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::MaySacrifice {
                description: "Sacrifice a creature to grow Crovax?".to_string(),
                filter: R::Creature.and(R::ControlledByYou),
                count: Value::ONE,
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Some(Box::new(Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                })),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature(
            "Crovax the Cursed",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Vampire, CreatureType::Noble],
            0,
            0,
        )
    }
}

/// Endangered Armodon — {2}{G}{G} 4/5 that bolts the moment you control
/// anything fragile.
pub fn endangered_armodon() -> CardDefinition {
    CardDefinition {
        state_trigger: Some(crate::card::StateTriggeredAbility {
            condition: Predicate::SelectorExists(Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::ToughnessAtMost(2)),
            }),
            effect: Effect::SacrificeSource,
        }),
        ..creature(
            "Endangered Armodon",
            cost(&[generic(2), g(), g()]),
            vec![CreatureType::Elephant],
            4,
            5,
        )
    }
}

/// Lowland Basilisk — {2}{G} 1/3 whose bites are lethal at end of combat.
pub fn lowland_basilisk() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToCreature, EventScope::SelfSource),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Destroy { what: Selector::TriggerSource }),
            },
        }],
        ..creature(
            "Lowland Basilisk",
            cost(&[generic(2), g()]),
            vec![CreatureType::Basilisk],
            1,
            3,
        )
    }
}

/// Walking Dream — {3}{U} 3/3 unblockable that stays tapped while an opponent
/// has a real board.
pub fn walking_dream() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        static_abilities: vec![StaticAbility {
            description: "This creature doesn't untap during your untap step if an \
                          opponent controls two or more creatures.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::This,
                condition: Some(Predicate::SelectorCountAtLeast {
                    sel: Selector::ControlledBy {
                        who: PlayerRef::EachOpponent,
                        filter: R::Creature,
                    },
                    n: Value::Const(2),
                }),
            },
        }],
        ..creature("Walking Dream", cost(&[generic(3), u()]), vec![CreatureType::Illusion], 3, 3)
    }
}

/// Shard Phoenix — {4}{R} 2/2 flier that sweeps the ground and comes back.
pub fn shard_phoenix() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::DealDamage {
                    to: Selector::EachPermanent(
                        R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
                    ),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r(), r(), r()]),
                from_graveyard: true,
                condition: Some(Predicate::All(vec![
                    Predicate::IsTurnOf(PlayerRef::You),
                    Predicate::CurrentStepIs(TurnStep::Upkeep),
                ])),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                ..Default::default()
            },
        ],
        ..creature("Shard Phoenix", cost(&[generic(4), r()]), vec![CreatureType::Phoenix], 2, 2)
    }
}

/// Silver Wyvern — {3}{U}{U} 4/3 flier that shrugs off anything aimed at it.
pub fn silver_wyvern() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::ChangeSpellTarget {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsOnlySource)),
            },
            ..Default::default()
        }],
        ..creature("Silver Wyvern", cost(&[generic(3), u(), u()]), vec![CreatureType::Drake], 4, 3)
    }
}

/// Victual Sliver — {G}{W} 2/2 giving every Sliver a life-gain sacrifice.
pub fn victual_sliver() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "All Slivers have \"{2}, Sacrifice this permanent: You gain 4 life.\"",
            effect: StaticEffect::GrantActivatedAbility {
                applies_to: Selector::EachPermanent(R::HasCreatureType(CreatureType::Sliver)),
                ability: ActivatedAbility {
                    mana_cost: cost(&[generic(2)]),
                    sac_cost: true,
                    effect: Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
                    ..Default::default()
                },
                condition: None,
            },
        }],
        ..creature("Victual Sliver", cost(&[g(), w()]), vec![CreatureType::Sliver], 2, 2)
    }
}

// ── Artifacts / lands ───────────────────────────────────────────────────────

/// Volrath's Stronghold — legendary land that recurs a creature card.
pub fn volraths_stronghold() -> CardDefinition {
    CardDefinition {
        name: "Volrath's Stronghold",
        card_types: vec![CardType::Land],
        supertypes: vec![Supertype::Legendary],
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
                mana_cost: cost(&[generic(1), b()]),
                tap_cost: true,
                effect: Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Library {
                        who: PlayerRef::OwnerOf(Box::new(target_n(0))),
                        pos: crate::effect::LibraryPosition::Top,
                    },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Volrath's Laboratory — {5} Artifact. Names a colour and a creature type on
/// entry, then prints 2/2s of it.
pub fn volraths_laboratory() -> CardDefinition {
    CardDefinition {
        name: "Volrath's Laboratory",
        cost: cost(&[generic(5)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Seq(vec![
            Effect::ChooseColorForSelf,
            Effect::NameCreatureType { what: Selector::This },
        ]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5)]),
            tap_cost: true,
            effect: Effect::CreateTokenOfChosenColorAndType { pt: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hornet Cannon — {4} Artifact. {3}, {T}: a Hornet that lives for one turn.
pub fn hornet_cannon() -> CardDefinition {
    CardDefinition {
        name: "Hornet Cannon",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Hornet".into(),
                        card_types: vec![CardType::Artifact, CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Insect],
                            ..Default::default()
                        },
                        power: 1,
                        toughness: 1,
                        keywords: vec![Keyword::Flying, Keyword::Haste],
                        ..Default::default()
                    },
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy { what: Selector::LastMoved }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Portcullis — {4} Artifact. A crowded board swallows the next arrival until
/// the Portcullis goes.
pub fn portcullis() -> CardDefinition {
    CardDefinition {
        name: "Portcullis",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    },
                    Predicate::SelectorCountAtLeast {
                        sel: Selector::EachPermanent(R::Creature),
                        n: Value::Const(3),
                    },
                ])),
            effect: Effect::ExileUntilSourceLeaves {
                what: Selector::TriggerSource,
                return_to: crate::card::ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Awakening — {2}{G}{G} Enchantment. Everyone's creatures and lands untap on
/// every upkeep.
pub fn awakening() -> CardDefinition {
    CardDefinition {
        name: "Awakening",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
            effect: Effect::Untap {
                what: Selector::EachPermanent(R::Creature.or(R::Land)),
                up_to: None,
            },
        }],
        ..Default::default()
    }
}

/// Intruder Alarm — {2}{U} Enchantment. Nothing untaps normally; every
/// creature entering untaps the whole board.
pub fn intruder_alarm() -> CardDefinition {
    CardDefinition {
        name: "Intruder Alarm",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntapGlobal {
                applies_to: Selector::EachPermanent(R::Creature),
                condition: None,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature,
                }),
            effect: Effect::Untap { what: Selector::EachPermanent(R::Creature), up_to: None },
        }],
        ..Default::default()
    }
}

/// Amok — {1}{R} Enchantment. Random discards buy +1/+1 counters.
pub fn amok() -> CardDefinition {
    CardDefinition {
        name: "Amok",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            discard_cost: Some((R::Any, 1)),
            discard_cost_random: true,
            effect: Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Hesitation — {1}{U} Enchantment. The next spell anyone casts eats it.
pub fn hesitation() -> CardDefinition {
    CardDefinition {
        name: "Hesitation",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer),
            effect: Effect::Seq(vec![
                Effect::SacrificePermanent { what: Selector::This },
                Effect::CounterSpell { what: Selector::TriggerSource },
            ]),
        }],
        ..Default::default()
    }
}

/// Volrath's Gardens — {1}{G} Enchantment. Tap a creature for 2 life, at
/// sorcery speed.
pub fn volraths_gardens() -> CardDefinition {
    CardDefinition {
        name: "Volrath's Gardens",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            sorcery_speed: true,
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Flowstone Blade — {R} Aura. {R}: the host gets +1/-1.
pub fn flowstone_blade() -> CardDefinition {
    CardDefinition {
        name: "Flowstone Blade",
        cost: cost(&[r()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::attached_to(Selector::This),
                power: Value::ONE,
                toughness: Value::Const(-1),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Bandage — {W} Instant. Prevent 1, then draw.
pub fn bandage() -> CardDefinition {
    CardDefinition {
        name: "Bandage",
        cost: cost(&[w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PreventNextDamage {
                target: target_filtered(R::Creature.or(R::Player).or(R::Planeswalker)),
                amount: Value::ONE,
            },
            draw(1),
        ]),
        ..Default::default()
    }
}

/// Rebound — {1}{U} Instant. Point a player-targeting spell somewhere else.
pub fn rebound() -> CardDefinition {
    CardDefinition {
        name: "Rebound",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChangeSpellTarget { what: target_filtered(R::IsSpellOnStack) },
        ..Default::default()
    }
}

/// Reins of Power — {2}{U}{U} Instant. Swap armies with an opponent for the
/// turn, untapped and hasty.
pub fn reins_of_power() -> CardDefinition {
    CardDefinition {
        name: "Reins of Power",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Untap {
                what: Selector::EachPermanent(R::Creature),
                up_to: None,
            },
            Effect::GainControl {
                what: Selector::ControlledBy { who: PlayerRef::Target(0), filter: R::Creature },
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::GainControl {
                what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                to: Some(PlayerRef::Target(0)),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::EachPermanent(R::Creature),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Licids ──────────────────────────────────────────────────────────────────

/// A Licid: a 2/2 that can pay `c` and tap to become an Aura granting
/// `bonus`, and pay `c` again to go back to being a creature.
fn licid(name: &'static str, color: crate::mana::ManaSymbol, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[color]),
            tap_cost: true,
            effect: Effect::LicidAttach {
                host: target_filtered(R::Creature),
                end_cost: cost(&[color]),
            },
            ..Default::default()
        }],
        equipped_bonus: Some(bonus),
        ..creature(name, cost(&[generic(2), color]), vec![CreatureType::Licid], 2, 2)
    }
}

/// Calming Licid — {2}{W}. As an Aura, the host can't attack.
pub fn calming_licid() -> CardDefinition {
    licid(
        "Calming Licid",
        w(),
        EquipBonus { keywords: vec![Keyword::CantAttack], ..Default::default() },
    )
}

/// Gliding Licid — {2}{U}. As an Aura, the host flies.
pub fn gliding_licid() -> CardDefinition {
    licid(
        "Gliding Licid",
        u(),
        EquipBonus { keywords: vec![Keyword::Flying], ..Default::default() },
    )
}

/// Corrupting Licid — {2}{B}. As an Aura, the host has fear.
pub fn corrupting_licid() -> CardDefinition {
    licid(
        "Corrupting Licid",
        b(),
        EquipBonus { keywords: vec![Keyword::Fear], ..Default::default() },
    )
}

/// Convulsing Licid — {2}{R}. As an Aura, the host can't block.
pub fn convulsing_licid() -> CardDefinition {
    licid(
        "Convulsing Licid",
        r(),
        EquipBonus { keywords: vec![Keyword::CantBlock], ..Default::default() },
    )
}

/// Tempting Licid — {2}{G}. As an Aura, everything able to block the host
/// does so.
pub fn tempting_licid() -> CardDefinition {
    licid(
        "Tempting Licid",
        g(),
        EquipBonus { keywords: vec![Keyword::AllMustBlock], ..Default::default() },
    )
}
