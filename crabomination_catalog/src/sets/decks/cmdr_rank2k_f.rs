//! Commander: gaps of EDHREC's commanders ranked 1501-2000 (the 2026-10-13
//! twenty-page rescan), sixth file — mostly three-card gaps. Tests in
//! `tests/recent_b/cmdr_rank2k_f.rs`.

use crate::card::{
    AdditionalCastCost, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, mint_token, mint_treasures, on_attack, on_dies, target_filtered};
use crate::effect::{ActivatedAbility, Duration, Effect, LookPick, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, r, u, w, Color, ManaCost};
use crabomination_base::tokens::{food_token, powerstone_token, tapped_treasure_token};
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

fn legend(name: &'static str, mana: ManaCost, types: Vec<CreatureType>, p: i32, t: i32) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..creature(name, mana, types, p, t) }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn static_ab(description: &'static str, effect: StaticEffect) -> StaticAbility {
    StaticAbility { description, effect }
}

fn artifact(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost, host: R) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(host) },
        ..Default::default()
    }
}

fn yours(filter: R) -> R {
    filter.and(R::ControlledByYou)
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn pump_eot(what: Selector, p: i32, t: i32) -> Effect {
    Effect::PumpPT { what, power: Value::Const(p), toughness: Value::Const(t), duration: Duration::EndOfTurn }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        colors: vec![color],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn soldiers() -> R {
    R::Creature.and(R::HasCreatureType(CreatureType::Soldier))
}

/// Angel of Condemnation — {2}{W}{W} 3/3 flying, vigilance Angel. {2}{W},
/// {T}: flicker another creature until the next end step. {2}{W}, {T}, exert
/// it: exile another creature until this leaves the battlefield.
pub fn angel_of_condemnation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                tap_cost: true,
                effect: Effect::ExileReturnToOwnerNextEndStep {
                    what: target_filtered(R::Creature.and(R::OtherThanSource)),
                    tapped: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), w()]),
                tap_cost: true,
                // CR 701.43a — exerting is "it won't untap during your next
                // untap step", paid beside the effect (no card here watches
                // "whenever you exert").
                effect: Effect::Seq(vec![
                    Effect::SkipNextUntap { what: Selector::This },
                    Effect::ExileUntilSourceLeaves {
                        what: target_filtered(R::Creature.and(R::OtherThanSource)),
                        return_to: crate::card::ExileReturnZone::Battlefield,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..creature("Angel of Condemnation", cost(&[generic(2), w(), w()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Barge In — {R} instant: an attacking creature gets +2/+2; each attacking
/// non-Human creature gains trample until end of turn.
pub fn barge_in() -> CardDefinition {
    spell(
        "Barge In",
        cost(&[r()]),
        CardType::Instant,
        Effect::Seq(vec![
            pump_eot(target_filtered(R::Creature.and(R::IsAttacking)), 2, 2),
            Effect::GrantKeyword {
                what: Selector::EachPermanent(
                    R::Creature.and(R::IsAttacking).and(R::HasCreatureType(CreatureType::Human).negate()),
                ),
                keyword: Keyword::Trample,
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Bartered Cow — {3}{W} 3/3 Ox; dying, or being discarded, makes a Food.
pub fn bartered_cow() -> CardDefinition {
    let food = || mint_token(food_token(), 1);
    CardDefinition {
        triggered_abilities: vec![
            on_dies(food()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDiscarded, EventScope::FromYourGraveyard)
                    .with_filter(Predicate::TriggerSourceIsSelf),
                effect: food(),
            },
        ],
        ..creature("Bartered Cow", cost(&[generic(3), w()]), vec![CreatureType::Ox], 3, 3)
    }
}

/// Blackbloom Rogue // Blackbloom Bog — {2}{B} 2/3 menace Human Rogue, +3/+0
/// while an opponent has eight or more cards in their graveyard; or a tapped
/// land for {B}.
pub fn blackbloom_rogue() -> CardDefinition {
    let bog = CardDefinition {
        name: "Blackbloom Bog",
        card_types: vec![CardType::Land],
        static_abilities: vec![static_ab("This land enters tapped.", StaticEffect::EntersTapped { applies_to: Selector::This })],
        activated_abilities: vec![crate::sets::tap_add(Color::Black)],
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Menace],
        static_abilities: vec![static_ab(
            "This creature gets +3/+0 as long as an opponent has eight or more cards in their graveyard.",
            StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::GreatestGraveyardSizeAmong(PlayerRef::EachOpponent),
                    Value::Const(8),
                ),
                power: 3,
                toughness: 0,
                keywords: vec![],
            },
        )],
        back_face: Some(Box::new(bog)),
        ..creature("Blackbloom Rogue", cost(&[generic(2), b()]), vec![CreatureType::Human, CreatureType::Rogue], 2, 3)
    }
}

/// Dragonspark Reactor — {1}{R} artifact: it or another artifact of yours
/// entering adds a charge counter. {4}, sacrifice it: that many damage to a
/// player and to up to one creature.
pub fn dragonspark_reactor() -> CardDefinition {
    let charges = || Value::CountersOn { what: Box::new(Selector::This), kind: CounterType::Charge };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Charge, amount: Value::ONE },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::DealDamage { to: target_filtered(R::Player), amount: charges() },
                Effect::OptionalTargets {
                    min: 0,
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TargetFiltered { slot: 1, filter: R::Creature },
                        amount: charges(),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..artifact("Dragonspark Reactor", cost(&[generic(1), r()]))
    }
}

/// Earth-Origin Yak — {3}{W} 2/4 Ox; entering pumps your creatures +1/+1
/// until end of turn.
pub fn earth_origin_yak() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(pump_eot(Selector::EachPermanent(yours(R::Creature)), 1, 1))],
        ..creature("Earth-Origin Yak", cost(&[generic(3), w()]), vec![CreatureType::Ox], 2, 4)
    }
}

/// Emblem of the Warmind — {1}{R} Aura on a creature you control: your
/// creatures have haste.
pub fn emblem_of_the_warmind() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Creatures you control have haste.",
            StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(yours(R::Creature)),
                keyword: Keyword::Haste,
            },
        )],
        ..aura("Emblem of the Warmind", cost(&[generic(1), r()]), yours(R::Creature))
    }
}

/// Evin, Waterdeep Opportunist — {3}{B} legendary 2/4 menace Human Rogue,
/// ward—sacrifice a creature; +2/+0 per Treasure you control; once each
/// turn, a creature sacrifice makes you a tapped Treasure.
pub fn evin_waterdeep_opportunist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Menace, Keyword::Ward(WardCost::SacrificeCreature)],
        static_abilities: vec![static_ab(
            "Evin gets +2/+0 for each Treasure you control.",
            StaticEffect::PumpSelfByControlledPermanents {
                filter: R::HasArtifactSubtype(ArtifactSubtype::Treasure),
                per_power: 2,
                per_toughness: 0,
            },
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureSacrificed, EventScope::AnyPlayer).once_per_turn(),
            effect: mint_token(tapped_treasure_token(), 1),
        }],
        ..legend(
            "Evin, Waterdeep Opportunist",
            cost(&[generic(3), b()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            4,
        )
    }
}

/// Fathom Fleet Swordjack — {3}{R} 4/3 Orc Pirate; attacking, it deals damage
/// equal to your artifacts to the player or planeswalker it attacks. Encore
/// {5}{R}.
pub fn fathom_fleet_swordjack() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::DealDamage {
            to: Selector::AttackedBySource,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(yours(R::Artifact)))),
        })],
        activated_abilities: vec![crate::effect::shortcut::encore(cost(&[generic(5), r()]))],
        ..creature("Fathom Fleet Swordjack", cost(&[generic(3), r()]), vec![CreatureType::Orc, CreatureType::Pirate], 4, 3)
    }
}

/// Fortified Beachhead — land: enters tapped unless you reveal a Soldier card
/// or control a Soldier; {T}: {W} or {U}; {5}, {T}: your Soldiers get +1/+1.
pub fn fortified_beachhead() -> CardDefinition {
    CardDefinition {
        name: "Fortified Beachhead",
        card_types: vec![CardType::Land],
        static_abilities: vec![static_ab(
            "Enters tapped unless you reveal a Soldier card from your hand or control a Soldier.",
            StaticEffect::EntersTappedUnless {
                applies_to: Selector::This,
                condition: Predicate::Any(vec![
                    Predicate::SelectorExists(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: R::HasCreatureType(CreatureType::Soldier),
                    }),
                    Predicate::SelectorExists(Selector::EachPermanent(yours(soldiers()))),
                ]),
            },
        )],
        activated_abilities: vec![
            crate::sets::tap_add(Color::White),
            crate::sets::tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(5)]),
                tap_cost: true,
                effect: pump_eot(Selector::EachPermanent(yours(soldiers())), 1, 1),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Guardian of Ghirapur — {2}{W} 3/3 flying Angel; entering flickers up to one
/// other creature or artifact you control until the next end step.
pub fn guardian_of_ghirapur() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::ExileReturnNextEndStep {
                what: target_filtered(yours(R::Creature.or(R::Artifact)).and(R::OtherThanSource)),
            }),
        })],
        ..creature("Guardian of Ghirapur", cost(&[generic(2), w()]), vec![CreatureType::Angel], 3, 3)
    }
}

/// Inquisitor's Ox — {3}{W} 2/5 Ox; delirium: +1/+0 and vigilance.
pub fn inquisitors_ox() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Delirium — +1/+0 and vigilance while four or more card types are among cards in your graveyard.",
            StaticEffect::PumpSelfIf {
                condition: Predicate::DeliriumActive { who: PlayerRef::You },
                power: 1,
                toughness: 0,
                keywords: vec![Keyword::Vigilance],
            },
        )],
        ..creature("Inquisitor's Ox", cost(&[generic(3), w()]), vec![CreatureType::Ox], 2, 5)
    }
}

/// Kaalia, Zenith Seeker — {R}{W}{B} legendary 3/3 flying, vigilance Human
/// Cleric. Entering, look at the top six: an Angel, a Demon and/or a Dragon
/// to hand, the rest to the bottom at random.
pub fn kaalia_zenith_seeker() -> CardDefinition {
    let kinds = [CreatureType::Angel, CreatureType::Demon, CreatureType::Dragon];
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(6),
            take: Some(Value::Const(3)),
            one_each: kinds.iter().map(|&t| R::HasCreatureType(t)).collect(),
            optional: true,
            rest_bottom_random: true,
            ..Default::default()
        })))],
        ..legend(
            "Kaalia, Zenith Seeker",
            cost(&[r(), w(), b()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            3,
            3,
        )
    }
}

/// Kuldotha Rebirth — {R} sorcery, sacrifice an artifact as an additional
/// cost: three 1/1 red Goblins.
pub fn kuldotha_rebirth() -> CardDefinition {
    CardDefinition {
        additional_cast_cost: vec![AdditionalCastCost::SacrificePermanent { filter: R::Artifact, count: 1 }],
        ..spell(
            "Kuldotha Rebirth",
            cost(&[r()]),
            CardType::Sorcery,
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: Arc::new(token("Goblin", 1, 1, Color::Red, vec![CreatureType::Goblin])),
            },
        )
    }
}

/// Noble's Purse — {2} artifact; enters tapped with three coin counters. {T},
/// remove a coin counter: a Treasure.
pub fn nobles_purse() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![crate::sets::enters_tapped()],
        enters_with_counters: Some((CounterType::Coin, Value::Const(3))),
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            remove_counter_cost: Some((CounterType::Coin, 1)),
            effect: mint_treasures(1),
            ..Default::default()
        }],
        ..artifact("Noble's Purse", cost(&[generic(2)]))
    }
}

/// Pirate's Prize — {3}{U} sorcery: draw two and make a Treasure.
pub fn pirates_prize() -> CardDefinition {
    spell("Pirate's Prize", cost(&[generic(3), u()]), CardType::Sorcery, Effect::Seq(vec![draw(2), mint_treasures(1)]))
}

/// Rune of Speed — {1}{R} Aura on a permanent: entering draws a card; a
/// creature host gets +1/+0 and haste, an Equipment host grants that.
pub fn rune_of_speed() -> CardDefinition {
    let mut c = CardDefinition {
        triggered_abilities: vec![etb(draw(1))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            keywords: vec![Keyword::Haste],
            relays_through_equipment: true,
            ..Default::default()
        }),
        ..aura("Rune of Speed", cost(&[generic(1), r()]), R::Permanent)
    };
    c.subtypes.enchantment_subtypes.push(crate::card::EnchantmentSubtype::Rune);
    c
}

/// Scrap Welder — {2}{R} 3/3 Goblin Artificer. {T}, sacrifice an artifact
/// with mana value X: target artifact card with mana value less than X returns
/// from your graveyard with haste. The target needs an artifact of yours above
/// its mana value, and only such an artifact pays (`sac_other_mv_above_target`).
pub fn scrap_welder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Artifact, 1)),
            sac_other_mv_above_target: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        R::Artifact
                            .from_your_graveyard()
                            .and(R::ManaValueLessThanOneAmong(Box::new(R::Artifact.and(R::ControlledByYou)))),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            ]),
            ..Default::default()
        }],
        ..creature("Scrap Welder", cost(&[generic(2), r()]), vec![CreatureType::Goblin, CreatureType::Artificer], 3, 3)
    }
}

/// Scuzzback Scrounger — {1}{R} 3/2 Goblin Warrior. At your first main phase
/// you may blight 1 for a Treasure.
pub fn scuzzback_scrounger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::PreCombatMain), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Blight 1 for a Treasure?".into(),
                body: Box::new(Effect::Seq(vec![Effect::Blight { n: Value::ONE }, mint_treasures(1)])),
            },
        }],
        ..creature("Scuzzback Scrounger", cost(&[generic(1), r()]), vec![CreatureType::Goblin, CreatureType::Warrior], 3, 2)
    }
}

/// Slagstone Refinery — {4} artifact: it or another nontoken artifact of yours
/// going to the graveyard or exile from the battlefield makes a tapped
/// Powerstone.
pub fn slagstone_refinery() -> CardDefinition {
    let stone = || mint_token(TokenDefinition { tapped: true, ..powerstone_token() }, 1);
    let nontoken_artifact = || Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact.and(R::NotToken) };
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::YourControl).with_filter(nontoken_artifact()),
                effect: stone(),
            },
            // An exiled card has no controller; its owner stands in.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardExiledFrom(crate::effect::exile_from::BATTLEFIELD), EventScope::AnyPlayer)
                    .with_filter(Predicate::All(vec![
                        nontoken_artifact(),
                        Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::OwnedByYou },
                    ])),
                effect: stone(),
            },
        ],
        ..artifact("Slagstone Refinery", cost(&[generic(4)]))
    }
}

/// Storm Fleet Sprinter — {1}{U}{R} 2/2 haste, unblockable Human Pirate.
pub fn storm_fleet_sprinter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste, Keyword::Unblockable],
        ..creature("Storm Fleet Sprinter", cost(&[generic(1), u(), r()]), vec![CreatureType::Human, CreatureType::Pirate], 2, 2)
    }
}

/// Yotian Tactician — {2}{W}{U} 3/4 Human Soldier; your other Soldiers get
/// +1/+1.
pub fn yotian_tactician() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![static_ab(
            "Other Soldiers you control get +1/+1.",
            StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(yours(soldiers()).and(R::OtherThanSource)),
                power: 1,
                toughness: 1,
            },
        )],
        ..creature("Yotian Tactician", cost(&[generic(2), w(), u()]), vec![CreatureType::Human, CreatureType::Soldier], 3, 4)
    }
}

/// Zephyr Sentinel — {1}{U} 2/1 flash, flying Human Soldier. Entering, return
/// up to one other creature you control to hand; a Soldier returned gives it
/// a +1/+1 counter.
pub fn zephyr_sentinel() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: yours(R::Creature).and(R::OtherThanSource),
            effect: Box::new(Effect::Seq(vec![
                Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::HasCreatureType(CreatureType::Soldier),
                    },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                    else_: Box::new(Effect::Noop),
                },
                Effect::Move { what: Selector::Target(0), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ])),
        })],
        ..creature("Zephyr Sentinel", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Soldier], 2, 1)
    }
}

/// Zhentarim Bandit — {1}{B} 2/1 Halfling Rogue; attacking, you may pay 1 life
/// for a Treasure.
pub fn zhentarim_bandit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Pay 1 life for a Treasure?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::ONE },
                mint_treasures(1),
            ])),
        })],
        ..creature("Zhentarim Bandit", cost(&[generic(1), b()]), vec![CreatureType::Halfling, CreatureType::Rogue], 2, 1)
    }
}
