//! EDHREC Star Trek single-commander average decks short two cards (scan of
//! 2026-11-26): Lieutenant Commander Data (Romulan Bird-of-Prey,
//! Anthropological Mission), Kang, Suspicious Captain (Boarding Action,
//! Planetary Bombardment), Erica Ortegas, Plucky Pilot (Horta, Brazen Bluff);
//! three short three: M'Ress, Crew Coordinator (+ Shuttlecraft Galileo, Subspace
//! Anomaly), Worf, Son of Mogh (Galileo, Academy Instructor, M'Ress), Brad
//! Boimler (Rutherford, Upgraded Ensign, and Brad); Tuvok, Chief Tactical
//! Officer (Invasion of Kamigawa, Salinae, the Ocean Mother).
//! Tests in `tests/recent_b/cmdr_nov26.rs`.

use std::sync::Arc;

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility, StationBand,
    Subtypes, Supertype, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{etb, on_attack, station, target_filtered};
use crate::effect::{
    ActivatedAbility, CounteredSpellZone, Duration, Effect, LookPick, ManaPayload, PlayerRef, StaticEffect, Value,
};
use crate::mana::{cost, generic, r, u, w, Color};

/// "This Spacecraft isn't a creature": below its creature band's threshold
/// (CR 722 — a Spacecraft is a creature only from its P/T band on).
fn below_band(min: u32) -> Predicate {
    Predicate::Not(Box::new(Predicate::SourceHasCountersAtLeast { counter: CounterType::Charge, n: min }))
}

fn noncreature_cast() -> EventSpec {
    EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
        .with_filter(Predicate::CastSpellMatches(R::Noncreature))
}

/// Romulan Bird-of-Prey — {1}{U} Spacecraft. While it isn't a creature it has
/// hexproof, and each noncreature spell you cast puts two charge counters on
/// it. Station; 8+: 4/4 flying.
pub fn romulan_bird_of_prey() -> CardDefinition {
    CardDefinition {
        name: "Romulan Bird-of-Prey",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        static_abilities: vec![StaticAbility {
            description: "Cloaking Device — As long as this Spacecraft isn't a creature, it has hexproof.",
            effect: StaticEffect::WhileCondition {
                condition: below_band(8),
                inner: Box::new(StaticEffect::GrantKeyword { applies_to: Selector::This, keyword: Keyword::Hexproof }),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: noncreature_cast().with_filter(Predicate::All(vec![
                Predicate::CastSpellMatches(R::Noncreature),
                below_band(8),
            ])),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::Const(2) },
        }],
        activated_abilities: vec![station()],
        station: vec![StationBand { min: 8, keywords: vec![Keyword::Flying], pt: Some((4, 4)), ..Default::default() }],
        ..Default::default()
    }
}

/// Anthropological Mission — {2}{U} enchantment. Whenever you cast a
/// noncreature spell, put a quest counter on it and draw a card, then discard
/// one unless it has three or more quest counters.
pub fn anthropological_mission() -> CardDefinition {
    CardDefinition {
        name: "Anthropological Mission",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: noncreature_cast(),
            effect: Effect::Seq(vec![
                Effect::AddCounter { what: Selector::This, kind: CounterType::Quest, amount: Value::ONE },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
                Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Quest, n: 3 },
                    then: Box::new(Effect::Noop),
                    else_: Box::new(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Boarding Action — {1}{R} sorcery. Create a 2/1 red Klingon Warrior token
/// with haste. Beam me up {2}{R}: flashback that also returns a creature you
/// control to its owner's hand.
pub fn boarding_action() -> CardDefinition {
    let warrior = Arc::new(TokenDefinition {
        name: "Klingon Warrior".into(),
        power: 2,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Klingon, CreatureType::Warrior],
            ..Default::default()
        },
        keywords: vec![Keyword::Haste],
        ..Default::default()
    });
    CardDefinition {
        name: "Boarding Action",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(2), r()]))],
        flashback_additional_cost: vec![AdditionalCastCost::ReturnToHand {
            filter: R::Creature.and(R::ControlledByYou),
            count: 1,
            count_x: false,
        }],
        effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: warrior },
        ..Default::default()
    }
}

/// Planetary Bombardment — {R} sorcery. 2 damage to target creature, 4 instead
/// if a creature died this turn.
pub fn planetary_bombardment() -> CardDefinition {
    let hit = |n| Box::new(Effect::DealDamage { to: target_filtered(R::Creature), amount: Value::Const(n) });
    CardDefinition {
        name: "Planetary Bombardment",
        cost: cost(&[r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::If {
            cond: Predicate::CreatureDiedThisTurnMatching { filter: R::Any },
            then: hit(4),
            else_: hit(2),
        },
        ..Default::default()
    }
}

/// Horta — {4}{R}{R} 7/6 Elemental, trample; basic landcycling {2}.
pub fn horta() -> CardDefinition {
    CardDefinition {
        name: "Horta",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Typecycling(Box::new((cost(&[generic(2)]), R::IsBasicLand)))],
        ..Default::default()
    }
}

/// Brazen Bluff — {R}{R} instant. Choose target noncreature spell an opponent
/// controls; its controller may exile it. If they don't, you may cast a
/// noncreature spell from your hand without paying its mana cost.
pub fn brazen_bluff() -> CardDefinition {
    CardDefinition {
        name: "Brazen Bluff",
        cost: cost(&[r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::MayDoBy {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                description: "Exile your spell? (If you don't, Brazen Bluff's caster may cast a noncreature spell free.)"
                    .into(),
                body: Box::new(Effect::MoveSpellToZone {
                    what: target_filtered(R::Noncreature.and(R::ControlledByOpponent)),
                    zone: CounteredSpellZone::Exile,
                }),
            },
            Effect::If {
                cond: Predicate::EntityMatches { what: Selector::Target(0), filter: R::IsSpellOnStack },
                then: Box::new(Effect::MayCastFromHandFreeMatching {
                    filter: R::Noncreature,
                    max_mv: Value::Const(99),
                    else_: Box::new(Effect::Noop),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

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

fn officers(n: i32) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(n), definition: officer_token() }
}

fn landfall(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Land }),
        effect,
    }
}

/// M'Ress, Crew Coordinator — {4}{W} legendary 4/4 Caitian Officer. Landfall:
/// two 1/1 red Officer tokens.
pub fn mress_crew_coordinator() -> CardDefinition {
    CardDefinition {
        name: "M'Ress, Crew Coordinator",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Caitian, CreatureType::Officer], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![landfall(officers(2))],
        ..Default::default()
    }
}

/// Shuttlecraft Galileo — {3}{W} legendary Spacecraft. Landfall: a 1/1 red
/// Officer. Station; 6+: 3/3 flying, and attacking gives up to seven target
/// attacking creatures flying until end of turn.
pub fn shuttlecraft_galileo() -> CardDefinition {
    CardDefinition {
        name: "Shuttlecraft Galileo",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Spacecraft], ..Default::default() },
        triggered_abilities: vec![landfall(officers(1))],
        activated_abilities: vec![station()],
        station: vec![StationBand {
            min: 6,
            keywords: vec![Keyword::Flying],
            pt: Some((3, 3)),
            triggers: vec![on_attack(Effect::ApplyToTargets {
                max_targets: 7,
                min_targets: 0,
                filter: R::Creature.and(R::IsAttacking),
                effect: Box::new(Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                }),
            })],
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Subspace Anomaly — land. {T}: {C}. {4}{W}, {T}, sacrifice it: two 1/1 red
/// Officers and a tapped Planet token "New Planet" ({T}: any color).
pub fn subspace_anomaly() -> CardDefinition {
    let planet = TokenDefinition {
        name: "New Planet".into(),
        card_types: vec![CardType::Land],
        subtypes: Subtypes { land_types: vec![LandType::Planet], ..Default::default() },
        tapped: true,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::AnyColors(Value::ONE) },
            ..Default::default()
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Subspace Anomaly",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::Colorless(Value::ONE) },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                sac_cost: true,
                mana_cost: cost(&[generic(4), w()]),
                effect: Effect::Seq(vec![
                    officers(2),
                    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(planet) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Academy Instructor — {2}{R} 0/3 Bolian Officer. +1/+0 for each Officer
/// token you control; each noncreature spell you cast makes a 1/1 red Officer.
pub fn academy_instructor() -> CardDefinition {
    CardDefinition {
        name: "Academy Instructor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bolian, CreatureType::Officer], ..Default::default() },
        power: 0,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "This creature gets +1/+0 for each Officer token you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasCreatureType(CreatureType::Officer).and(R::IsToken),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![TriggeredAbility { event: noncreature_cast(), effect: officers(1) }],
        ..Default::default()
    }
}

/// Rutherford, Upgraded Ensign — {2}{W} legendary 2/2 Human Artificer. ETB:
/// look at the top six, an artifact card may go to hand, the rest to the
/// bottom at random. Casting a Spacecraft spell puts a +1/+1 counter on target
/// creature.
pub fn rutherford_upgraded_ensign() -> CardDefinition {
    CardDefinition {
        name: "Rutherford, Upgraded Ensign",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Artificer], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            etb(Effect::LookPickToHand(Box::new(LookPick {
                who: PlayerRef::You,
                count: Value::Const(6),
                pick_filter: Some(R::Artifact),
                optional: true,
                rest_bottom_random: true,
                ..Default::default()
            }))),
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::HasArtifactSubtype(ArtifactSubtype::Spacecraft)),
                ),
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Brad Boimler, Eager Ensign — {1}{W} legendary 2/2 Human Officer, lifelink.
/// Whenever it becomes tapped, until end of turn counters put on permanents you
/// control come one extra of each kind (`GrantExtraCountersThisTurn`).
pub fn brad_boimler_eager_ensign() -> CardDefinition {
    CardDefinition {
        name: "Brad Boimler, Eager Ensign",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Officer], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::SelfSource),
            effect: Effect::GrantExtraCountersThisTurn { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

/// Salinae, the Ocean Mother — {6}{U}{U} legendary 5/7 Whale God, trample;
/// costs {X} less (X your devotion to blue). Whenever you cast an artifact or
/// creature spell, copy it as a 1/1 Fish creature in addition to its other
/// types (CR 707.9b).
pub fn salinae_the_ocean_mother() -> CardDefinition {
    CardDefinition {
        name: "Salinae, the Ocean Mother",
        cost: cost(&[generic(6), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Whale, CreatureType::God], ..Default::default() },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This spell costs {X} less to cast, where X is your devotion to blue.",
            effect: StaticEffect::SelfCostReducedByDevotion { colors: vec![Color::Blue] },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Artifact.or(R::Creature))),
            effect: Effect::CopySpellAsOneOne { what: Selector::TriggerSource, creature_type: CreatureType::Fish },
        }],
        ..Default::default()
    }
}

/// Invasion of Kamigawa // Rooftop Saboteurs — {3}{U} Siege, defense 4. ETB:
/// tap target artifact or creature an opponent controls and put a stun counter
/// on it. Back: a blue 2/3 Moonfolk Ninja, flying, drawing a card whenever it
/// deals combat damage to a player or battle.
pub fn invasion_of_kamigawa() -> CardDefinition {
    let saboteurs = CardDefinition {
        name: "Rooftop Saboteurs",
        card_types: vec![CardType::Creature],
        color_indicator: vec![Color::Blue],
        subtypes: Subtypes { creature_types: vec![CreatureType::Moonfolk, CreatureType::Ninja], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource).or_battle(),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..Default::default()
    };
    let victim = || target_filtered(R::Artifact.or(R::Creature).and(R::ControlledByOpponent));
    CardDefinition {
        name: "Invasion of Kamigawa",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Battle],
        subtypes: Subtypes { battle_subtypes: vec![crate::card::BattleSubtype::Siege], ..Default::default() },
        defense: 4,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Tap { what: victim() },
            Effect::AddCounter { what: victim(), kind: CounterType::Stun, amount: Value::ONE },
        ]))],
        back_face: Some(Box::new(saboteurs)),
        ..Default::default()
    }
}
