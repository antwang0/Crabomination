//! Commander: 2026-10-17 (`019Y6J4R`) closers, fourth batch. Tests in
//! `tests/recent_b/cmdr_019y_d.rs`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    StaticAbility, Subtypes, Supertype, TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, StaticEffect, ZoneDest,
};
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
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

/// Uldaros Theorix — {3}{U}{B}{B} legendary 5/5 flying Elder Sphinx. ETB, if
/// cast: exile up to one nonland card of each card type from your graveyard,
/// copy them, and cast any number of the copies with total mana value 6 or
/// less for free. (The exile picks aren't targeted — no graveyard card here
/// carries a targeting restriction.)
pub fn uldaros_theorix() -> CardDefinition {
    let types = [
        CardType::Artifact,
        CardType::Battle,
        CardType::Creature,
        CardType::Enchantment,
        CardType::Instant,
        CardType::Kindred,
        CardType::Planeswalker,
        CardType::Sorcery,
    ];
    let mut body: Vec<Effect> = types
        .into_iter()
        .map(|t| Effect::MoveChosen {
            from: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::HasCardType(t).and(R::Land.negate()),
            },
            filter: None,
            count: Value::ONE,
            up_to: true,
            to: ZoneDest::Exile,
        })
        .collect();
    body.push(Effect::CopyCardsAndCastFreeTotalMv {
        what: Selector::ExiledThisResolution { filter: R::Any },
        total: Value::Const(6),
    });
    let mut t = etb(Effect::Seq(body));
    t.event = t.event.with_filter(Predicate::SourceWasCast);
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![t],
        ..creature(
            "Uldaros Theorix",
            cost(&[generic(3), u(), b(), b()]),
            vec![CreatureType::Elder, CreatureType::Sphinx],
            5,
            5,
        )
    }
}

/// Beast, Erudite Aerialist — {3}{G/U} legendary 3/3. Has flying while it got
/// a +1/+1 counter on him this turn (`YouPutPlusOneCounterOnThisTurn`);
/// combat damage to a player draws a card.
pub fn beast_erudite_aerialist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![StaticAbility {
            description: "As long as you've put one or more +1/+1 counters on Beast this turn, he has flying.",
            effect: StaticEffect::SelfHasKeywordWhile {
                keyword: Keyword::Flying,
                condition: R::YouPutPlusOneCounterOnThisTurn,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..creature(
            "Beast, Erudite Aerialist",
            cost(&[generic(3), hybrid(Color::Green, Color::Blue)]),
            vec![CreatureType::Mutant, CreatureType::Scientist, CreatureType::Hero],
            3,
            3,
        )
    }
}

/// Hired Hexblade — {1}{B} 2/2 Elf Warlock. ETB, if Treasure mana was spent to
/// cast it: draw a card and lose 1 life.
pub fn hired_hexblade() -> CardDefinition {
    let mut t = etb(Effect::Seq(vec![
        Effect::Draw { who: Selector::You, amount: Value::ONE },
        Effect::LoseLife { who: Selector::You, amount: Value::ONE },
    ]));
    t.event = t.event.with_filter(Predicate::ValueAtLeast(Value::TreasureManaSpentToCastSource, Value::ONE));
    CardDefinition {
        triggered_abilities: vec![t],
        ..creature("Hired Hexblade", cost(&[generic(1), b()]), vec![CreatureType::Elf, CreatureType::Warlock], 2, 2)
    }
}

/// Ultron's Auxiliary — {1}{B} 1/1 menace artifact creature Robot Villain. An
/// artifact card put into your graveyard from anywhere (another one, from the
/// battlefield) adds a +1/+1 counter.
pub fn ultrons_auxiliary() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PutIntoGraveyard, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Artifact.and(R::OtherThanSource),
                },
            ),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..creature("Ultron's Auxiliary", cost(&[generic(1), b()]), vec![CreatureType::Robot, CreatureType::Villain], 1, 1)
    }
}

/// Ragamuffin Raptor — {4}{G} 4/3 Dinosaur. ETB: return up to one target
/// creature or Food card from your graveyard to your hand.
pub fn ragamuffin_raptor() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::InGraveyard
                .and(R::OwnedByYou)
                .and(R::Creature.or(R::HasArtifactSubtype(crate::card::ArtifactSubtype::Food))),
            effect: Box::new(Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::You) }),
        })],
        ..creature("Ragamuffin Raptor", cost(&[generic(4), g()]), vec![CreatureType::Dinosaur], 4, 3)
    }
}

/// Urban Retreat — land, enters tapped, {T}: {G}, {W} or {U}. From your hand:
/// {2}, return a tapped creature you control to its owner's hand: put it onto
/// the battlefield (sorcery speed).
pub fn urban_retreat() -> CardDefinition {
    CardDefinition {
        name: "Urban Retreat",
        card_types: vec![CardType::Land],
        static_abilities: vec![crate::sets::enters_tapped()],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColors(vec![Color::Green, Color::White, Color::Blue], Value::ONE),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2)]),
                from_hand: true,
                sorcery_speed: true,
                return_permanent_cost: Some((R::Creature.and(R::Tapped), 1)),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Soulbright Seeker — {R} 2/1 Elemental Sorcerer; as an additional cost,
/// behold an Elemental or pay {2}. {R}: target creature you control gains
/// trample until end of turn; the third resolution this turn also adds
/// {R}{R}{R}{R}.
pub fn soulbright_seeker() -> CardDefinition {
    let trample = || Effect::GrantKeyword {
        what: target_filtered(R::Creature.and(R::ControlledByYou)),
        keyword: Keyword::Trample,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::BeholdOrPay {
            filter: R::HasCreatureType(CreatureType::Elemental),
            pay: 2,
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::NthResolutionThisTurn {
                branches: vec![
                    trample(),
                    trample(),
                    Effect::Seq(vec![
                        trample(),
                        Effect::AddMana { who: PlayerRef::You, pool: ManaPayload::OfColor(Color::Red, Value::Const(4)) },
                    ]),
                    trample(),
                ],
            },
            ..Default::default()
        }],
        ..creature(
            "Soulbright Seeker",
            cost(&[r()]),
            vec![CreatureType::Elemental, CreatureType::Sorcerer],
            2,
            1,
        )
    }
}

/// Ith, High Arcanist — {5}{W}{U} legendary 3/5 vigilance Human Wizard. {T}:
/// untap target attacking creature; prevent all combat damage to and by it
/// this turn. Suspend 4—{W}{U}.
pub fn ith_high_arcanist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Suspend(4, cost(&[w(), u()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Untap { what: target_filtered(R::Creature.and(R::IsAttacking)), up_to: None },
                Effect::PreventAllCombatDamageInvolving { target: Selector::Target(0) },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Ith, High Arcanist",
            cost(&[generic(5), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            5,
        )
    }
}
