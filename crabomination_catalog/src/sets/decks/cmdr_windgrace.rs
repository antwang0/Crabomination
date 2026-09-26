//! Commander: the cards the **Nature's Vengeance** precon (C18, Lord
//! Windgrace) needed beyond what the catalog had (Forge of Heroes and Moldgraf
//! Monstrosity landed with Exquisite Invention and Death Toll). Tests in
//! `tests/recent_b/cmdr_windgrace.rs`.
//!
//! Residuals (each also on its card):
//! - **Emissary of Grudges** — the choice isn't secret, and its reveal
//!   redirects any spell that targets you or your permanents, not only the
//!   chosen player's.
//! - **Flameblast Dragon** — {X} is asked before {R}, so a bot seat, which
//!   answers X out of floating mana only, rarely pays.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, EventKind, EventScope,
    EventSpec, Keyword, LandType, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector,
    StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, x, Color, ManaCost};
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

fn sorcery(name: &'static str, mana: ManaCost, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![CardType::Sorcery], effect, ..Default::default() }
}

fn land(name: &'static str, activated_abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition { name, card_types: vec![CardType::Land], activated_abilities, ..Default::default() }
}

fn token(name: &str, p: i32, t: i32, types: Vec<CreatureType>, keywords: Vec<Keyword>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        keywords,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn is(what: Selector, filter: R) -> Predicate {
    Predicate::EntityMatches { what, filter }
}

fn basic() -> R {
    R::Land.and(R::HasSupertype(Supertype::Basic))
}

/// "{N}, {T}, Sacrifice this land: search for a [filter] card, put it onto the
/// battlefield tapped, then shuffle."
fn fetch(n: u32, filter: R) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(n)]),
        tap_cost: true,
        sac_cost: true,
        effect: Effect::Search {
            who: PlayerRef::You,
            filter,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        ..Default::default()
    }
}

/// Lord Windgrace — {2}{B}{R}{G} loyalty-5 planeswalker that can be your
/// commander. +2: discard, then draw (two if the discard was a land). −3: up
/// to two land cards from your graveyard onto the battlefield. −11: destroy
/// up to six nonland permanents, then six 2/2 forestwalk Cat Warriors.
pub fn lord_windgrace() -> CardDefinition {
    let cat = TokenDefinition {
        colors: vec![Color::Green],
        ..token("Cat Warrior", 2, 2, vec![CreatureType::Cat, CreatureType::Warrior], vec![Keyword::Landwalk(LandType::Forest)])
    };
    let draw = || Effect::Draw { who: Selector::You, amount: Value::ONE };
    CardDefinition {
        name: "Lord Windgrace",
        cost: cost(&[generic(2), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Windgrace], ..Default::default() },
        base_loyalty: 5,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::Seq(vec![
                    Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
                    draw(),
                    Effect::If {
                        cond: Predicate::SelectorCountAtLeast {
                            sel: Selector::DiscardedThisResolution { filter: R::Land },
                            n: Value::ONE,
                        },
                        then: Box::new(draw()),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Land.and(R::InYourGraveyard),
                    effect: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                    }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -11,
                effect: Effect::Seq(vec![
                    Effect::ApplyToTargets {
                        max_targets: 6,
                        min_targets: 0,
                        filter: R::Permanent.and(R::Nonland),
                        effect: Box::new(Effect::Destroy { what: Selector::Target(0) }),
                    },
                    Effect::CreateToken { who: PlayerRef::You, count: Value::Const(6), definition: Arc::new(cat) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Charnelhoard Wurm — {4}{B}{R}{G} 6/6 trample. Dealing damage to an
/// opponent: you may return a card from your graveyard to hand.
pub fn charnelhoard_wurm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::MayDo {
                description: "Return a card from your graveyard to your hand?".into(),
                body: Box::new(Effect::Move {
                    what: target_filtered(R::InYourGraveyard),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        ..creature("Charnelhoard Wurm", cost(&[generic(4), b(), r(), g()]), vec![CreatureType::Wurm], 6, 6)
    }
}

/// Crash of Rhino Beetles — {4}{G} 5/5 trample Insect; +10/+10 while you
/// control ten or more lands.
pub fn crash_of_rhino_beetles() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        static_abilities: vec![StaticAbility {
            description: "This creature gets +10/+10 as long as you control ten or more lands.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::ValueAtLeast(
                    Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land),
                    Value::Const(10),
                ),
                power: 10,
                toughness: 10,
                keywords: vec![],
            },
        }],
        ..creature("Crash of Rhino Beetles", cost(&[generic(4), g()]), vec![CreatureType::Insect], 5, 5)
    }
}

/// Emissary of Grudges — {5}{R} 6/5 flying haste Efreet. Choose an opponent
/// as it enters; once, redirect a spell that targets you or a permanent you
/// control. (Residual: the choice is open, and any such spell qualifies.)
pub fn emissary_of_grudges() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Haste],
        as_enters_effect: Some(Effect::ChoosePlayerForSource { opponent: true }),
        activated_abilities: vec![ActivatedAbility {
            activate_once: true,
            effect: Effect::ChooseNewTargetsForSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::SpellTargetsControllerOrControlled)),
            },
            ..Default::default()
        }],
        ..creature("Emissary of Grudges", cost(&[generic(5), r()]), vec![CreatureType::Efreet], 6, 5)
    }
}

/// Flameblast Dragon — {4}{R}{R} 5/5 flier. Attacking: you may pay {X}{R} for
/// X damage to any target (one payment: X is sized after the {R}).
pub fn flameblast_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayPayXPlus {
                extra: cost(&[r()]),
                description: "Pay {X}{R}: X damage to any target".into(),
                body: Box::new(Effect::DealDamage {
                    to: target_filtered(R::Creature.or(R::Planeswalker).or(R::Player)),
                    amount: Value::XFromCost,
                }),
            },
        }],
        ..creature("Flameblast Dragon", cost(&[generic(4), r(), r()]), vec![CreatureType::Dragon], 5, 5)
    }
}

/// Fury Storm — {2}{R}{R} instant: copy target instant or sorcery spell. Cast,
/// it copies itself once per command-zone cast of your commander.
pub fn fury_storm() -> CardDefinition {
    CardDefinition {
        name: "Fury Storm",
        cost: cost(&[generic(2), r(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::CopySpellMayChooseTargets {
            what: target_filtered(
                R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
            ),
            count: Value::ONE,
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::CopySpellMayChooseTargets {
                what: Selector::This,
                count: Value::CommanderCastsFromCommandZone(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Gyrus, Waker of Corpses — {X}{B}{R}{G} 0/0 Hydra with a +1/+1 counter per
/// mana spent. Attacking: exile a lesser-power creature card from your
/// graveyard for a token copy of it, tapped and attacking, gone at end of
/// combat.
pub fn gyrus_waker_of_corpses() -> CardDefinition {
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::CastSpellManaSpent)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Exile a creature card with lesser power for an attacking copy?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Move {
                        what: target_filtered(R::Creature.and(R::InYourGraveyard).and(R::PowerLessThanSource)),
                        to: ZoneDest::Exile,
                    },
                    Effect::TokenCopyAttackingUntilEndOfCombat { source: Selector::LastMoved },
                ])),
            },
        }],
        ..legend("Gyrus, Waker of Corpses", cost(&[x(), b(), r(), g()]), vec![CreatureType::Hydra], 0, 0)
    }
}

/// Hunting Wilds — {3}{G} sorcery, kicker {3}{G}: up to two Forests onto the
/// battlefield tapped; kicked, they untap and become 3/3 green haste
/// creatures.
pub fn hunting_wilds() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Kicker(cost(&[generic(3), g()]))],
        ..sorcery(
            "Hunting Wilds",
            cost(&[generic(3), g()]),
            Effect::Seq(vec![
                Effect::SearchUpToN {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasLandType(LandType::Forest)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                    count: Value::Const(2),
                },
                Effect::If {
                    cond: Predicate::SpellWasKicked,
                    then: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: Selector::LastMoved, up_to: None },
                        Effect::BecomeCreature {
                            what: Selector::LastMoved,
                            power: Value::Const(3),
                            toughness: Value::Const(3),
                            creature_types: vec![],
                            keywords: vec![Keyword::Haste],
                            duration: Duration::Permanent,
                        },
                        Effect::BecomeColor {
                            what: Selector::LastMoved,
                            colors: vec![Color::Green],
                            duration: Duration::Permanent,
                            additive: false,
                        },
                    ])),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        )
    }
}

/// Jund Panorama — {C}, or {1}, sacrifice for a tapped basic Swamp, Mountain
/// or Forest.
pub fn jund_panorama() -> CardDefinition {
    let types = R::HasLandType(LandType::Swamp)
        .or(R::HasLandType(LandType::Mountain))
        .or(R::HasLandType(LandType::Forest));
    land("Jund Panorama", vec![crate::sets::tap_add_colorless(), fetch(1, basic().and(types))])
}

/// Nesting Dragon — {3}{R}{R} 5/4 flier. Landfall: a 0/2 defender Dragon Egg
/// that, dying, becomes a 2/2 flying firebreathing Dragon.
pub fn nesting_dragon() -> CardDefinition {
    let dragon = TokenDefinition {
        colors: vec![Color::Red],
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
        ..token("Dragon", 2, 2, vec![CreatureType::Dragon], vec![Keyword::Flying])
    };
    let egg = TokenDefinition {
        colors: vec![Color::Red],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(dragon) },
        }],
        ..token("Dragon Egg", 0, 2, vec![CreatureType::Dragon], vec![Keyword::Defender])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(is(Selector::TriggerSource, R::Land)),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(egg) },
        }],
        ..creature("Nesting Dragon", cost(&[generic(3), r(), r()]), vec![CreatureType::Dragon], 5, 4)
    }
}

/// Reality Scramble — {2}{R}{R} sorcery, retrace: put a permanent you own on
/// the bottom of your library, then reveal until a card sharing a card type
/// with it and put that onto the battlefield.
pub fn reality_scramble() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Retrace],
        ..sorcery(
            "Reality Scramble",
            cost(&[generic(2), r(), r()]),
            Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Permanent.and(R::OwnedByYou)),
                    to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::Bottom },
                },
                Effect::RevealUntilSharesCardTypeToBattlefield { with: Selector::LastMoved },
            ]),
        )
    }
}

/// Thantis, the Warweaver — {3}{B}{R}{G} 5/5 reach, vigilance Spider. All
/// creatures attack each combat if able; each creature attacking you or your
/// planeswalker grows it.
pub fn thantis_the_warweaver() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "All creatures attack each combat if able.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(R::Creature),
                keyword: Keyword::MustAttack,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
        }],
        ..legend("Thantis, the Warweaver", cost(&[generic(3), b(), r(), g()]), vec![CreatureType::Spider], 5, 5)
    }
}

/// Turntimber Sower — {2}{G} 3/3 Elf Druid. Land cards to your graveyard: a
/// 0/1 Plant. {G}, sacrifice three creatures: a land card from your graveyard
/// to hand.
pub fn turntimber_sower() -> CardDefinition {
    let plant = TokenDefinition {
        colors: vec![Color::Green],
        ..token("Plant", 0, 1, vec![CreatureType::Plant], vec![])
    };
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPutIntoGraveyard, EventScope::YourControl).once_per_batch(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(plant) },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::Creature, 3)),
            sac_other_may_be_source: true,
            effect: Effect::Move {
                what: target_filtered(R::Land.and(R::InYourGraveyard)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Turntimber Sower",
            cost(&[generic(2), g()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Warped Landscape — {C}, or {2}, sacrifice for a tapped basic land.
pub fn warped_landscape() -> CardDefinition {
    land("Warped Landscape", vec![crate::sets::tap_add_colorless(), fetch(2, basic())])
}

/// Xantcha, Sleeper Agent — {1}{B}{R} 5/5 legendary Phyrexian Minion. It
/// enters under an opponent's control, attacks each combat, never its owner;
/// any player may pay {3}: its controller loses 2 life and the payer draws.
pub fn xantcha_sleeper_agent() -> CardDefinition {
    CardDefinition {
        enters_under_opponent_control: true,
        keywords: vec![Keyword::MustAttack, Keyword::CantAttackOwner],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            any_player: true,
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::This))), amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: Value::ONE },
            ]),
            ..Default::default()
        }],
        ..legend(
            "Xantcha, Sleeper Agent",
            cost(&[generic(1), b(), r()]),
            vec![CreatureType::Phyrexian, CreatureType::Minion],
            5,
            5,
        )
    }
}

/// Zendikar Incarnate — {2}{R}{G} */4 Elemental; power is the lands you
/// control.
pub fn zendikar_incarnate() -> CardDefinition {
    CardDefinition {
        dynamic_pt: Some(DynamicPt::LandsControlledPower { base_p: 0, base_t: 4 }),
        ..creature("Zendikar Incarnate", cost(&[generic(2), r(), g()]), vec![CreatureType::Elemental], 0, 4)
    }
}
