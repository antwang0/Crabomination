//! Commander: the cards the **Peer Through Time** precon (C14, Teferi,
//! Temporal Archmage) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_teferi.rs`.
//!
//! Residuals (each also on its card):
//! - **Domineering Will** — "target player" is always you (the engine's
//!   recipient); the three creatures are the real targets.
//! - **Intellectual Offering** — each "choose an opponent" is the engine's
//!   pick (`ChooseOpponentThen`: the one with the fewest creatures).
//! - **Shaper Parasite** — +2/−2 or −2/+2 is picked as the trigger is put on
//!   the stack, not as it resolves.
//! - **Infinite Reflection** — the ETB copies onto every nontoken creature you
//!   control, the enchanted one included (a copy of itself changes nothing).

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R,
    Selector, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility,
    Value, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, generic, u};
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

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition { name, cost: mana, card_types: vec![kind], effect, ..Default::default() }
}

fn aura(name: &'static str, mana: ManaCost) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        ..Default::default()
    }
}

/// "When this creature is turned face up, [effect]." (CR 708.8)
fn on_turn_up(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::TurnedFaceUp, EventScope::SelfSource), effect }
}

fn draw(n: i32) -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::Const(n) }
}

fn enchanted() -> Selector {
    Selector::AttachedTo(Box::new(Selector::This))
}

/// Aether Gale — return exactly six target nonland permanents to their
/// owners' hands.
pub fn aether_gale() -> CardDefinition {
    spell(
        "Aether Gale",
        cost(&[generic(3), u(), u()]),
        CardType::Sorcery,
        Effect::ApplyToTargets {
            max_targets: 6,
            min_targets: 6,
            filter: R::Permanent.and(R::Nonland),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        },
    )
}

/// Azure Mage — {3}{U}: draw a card.
pub fn azure_mage() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: draw(1),
            ..Default::default()
        }],
        ..creature("Azure Mage", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Breaching Leviathan — cast from your hand, it taps every nonblue creature
/// and they skip their next untap (the ruling: yours too).
pub fn breaching_leviathan() -> CardDefinition {
    let nonblue = || Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasColor(Color::Blue)))));
    let mut t = etb(Effect::Seq(vec![
        Effect::Tap { what: nonblue() },
        Effect::SkipNextUntap { what: nonblue() },
    ]));
    t.event = t.event.with_filter(Predicate::SourceCastFromOwnersHand);
    CardDefinition {
        triggered_abilities: vec![t],
        ..creature("Breaching Leviathan", cost(&[generic(7), u(), u()]), vec![CreatureType::Leviathan], 9, 9)
    }
}

/// Brine Elemental — morph {5}{U}{U}; turned face up, each opponent skips
/// their next untap step (cumulative, per the ruling).
pub fn brine_elemental() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(5), u(), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::SkipPlayerUntapStep { player: PlayerRef::EachOpponent })],
        ..creature("Brine Elemental", cost(&[generic(4), u(), u()]), vec![CreatureType::Elemental], 5, 4)
    }
}

/// Crown of Doom — attackers of you or your planeswalkers get +2/+0; {2} on
/// your turn: a player other than its owner gains control of it.
pub fn crown_of_doom() -> CardDefinition {
    let recipient = Selector::TargetFiltered {
        slot: 0,
        filter: R::Player.and(R::Not(Box::new(R::SourceOwnerPlayer))),
    };
    CardDefinition {
        name: "Crown of Doom",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::GainControl {
                what: Selector::This,
                to: Some(PlayerRef::ControllerOf(Box::new(recipient))),
                duration: Duration::Permanent,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Domineering Will — up to three nonattacking creatures: you control them
/// until end of turn, untap them, and they block this turn if able.
/// Residual: "target player" is always you.
pub fn domineering_will() -> CardDefinition {
    spell(
        "Domineering Will",
        cost(&[generic(3), u()]),
        CardType::Instant,
        Effect::ApplyToTargets {
            max_targets: 3,
            min_targets: 0,
            filter: R::Creature.and(R::Not(Box::new(R::IsAttacking))),
            effect: Box::new(Effect::Seq(vec![
                Effect::GainControl { what: Selector::Target(0), to: None, duration: Duration::EndOfTurn },
                Effect::Untap { what: Selector::Target(0), up_to: None },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::MustBlock,
                    duration: Duration::EndOfTurn,
                },
            ])),
        },
    )
}

/// Dulcet Sirens — {U},{T}: target creature attacks target opponent this turn
/// if able; morph {U}.
pub fn dulcet_sirens() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[u()]))],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[u()]),
            effect: Effect::MustAttackPlayerThisTurn {
                attacker: target_filtered(R::Creature),
                defender: Selector::TargetFiltered { slot: 1, filter: R::OpponentPlayer },
            },
            ..Default::default()
        }],
        ..creature("Dulcet Sirens", cost(&[generic(2), u()]), vec![CreatureType::Siren], 1, 3)
    }
}

/// Fathom Seer — morph: return two Islands you control; turned face up, draw
/// two cards.
pub fn fathom_seer() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::MorphCost(Box::new(WardCost::ReturnMatchingToHand(
            Box::new(R::HasLandType(crate::card::LandType::Island)),
            2,
        )))],
        triggered_abilities: vec![on_turn_up(draw(2))],
        ..creature("Fathom Seer", cost(&[generic(1), u()]), vec![CreatureType::Illusion], 1, 3)
    }
}

/// Fool's Demise — when the enchanted creature dies, it returns under your
/// control; when this Aura goes to a graveyard from play, it returns to hand.
pub fn fools_demise() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::EnchantedBySource),
                effect: Effect::Move {
                    what: Selector::TriggerSource,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentDied, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
            },
        ],
        ..aura("Fool's Demise", cost(&[generic(4), u()]))
    }
}

/// Infinite Reflection — entering attached, your other nontoken creatures
/// become copies of the enchanted creature; your nontoken creatures enter as
/// copies of it.
pub fn infinite_reflection() -> CardDefinition {
    let yours = || R::Creature.and(R::ControlledByYou).and(R::Not(Box::new(R::IsToken)));
    CardDefinition {
        triggered_abilities: vec![etb(Effect::BecomeCopyOf {
            what: Selector::EachPermanent(yours()),
            source: enchanted(),
            extra_creature_types: vec![],
            keep_own_triggered: false,
            keep_own_activated: false,
        })],
        static_abilities: vec![StaticAbility {
            description: "Nontoken creatures you control enter as a copy of enchanted creature.",
            effect: StaticEffect::CreaturesEnterAsCopyOf { filter: yours(), of: enchanted() },
        }],
        ..aura("Infinite Reflection", cost(&[generic(5), u()]))
    }
}

/// Intellectual Offering — you and a chosen opponent draw three; you and a
/// chosen opponent untap your nonland permanents.
pub fn intellectual_offering() -> CardDefinition {
    let chosen = || PlayerRef::ChosenPlayerOfSource;
    let untap = |who: PlayerRef| Effect::Untap {
        what: Selector::ControlledBy { who, filter: R::Nonland.and(R::Permanent) },
        up_to: None,
    };
    spell(
        "Intellectual Offering",
        cost(&[generic(4), u()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::Seq(vec![
                    draw(3),
                    Effect::Draw { who: Selector::Player(chosen()), amount: Value::Const(3) },
                ])),
            },
            Effect::ChooseOpponentThen {
                then: Box::new(Effect::Seq(vec![untap(PlayerRef::You), untap(chosen())])),
            },
        ]),
    )
}

/// Shaper Parasite — morph {2}{U}; turned face up, target creature gets
/// +2/−2 or −2/+2 until end of turn.
pub fn shaper_parasite() -> CardDefinition {
    let pump = |p: i32, t: i32| Effect::PumpPT {
        what: target_filtered(R::Creature),
        power: Value::Const(p),
        toughness: Value::Const(t),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        keywords: vec![Keyword::Morph(cost(&[generic(2), u()]))],
        triggered_abilities: vec![on_turn_up(Effect::ChooseMode(vec![pump(2, -2), pump(-2, 2)]))],
        ..creature("Shaper Parasite", cost(&[generic(1), u(), u()]), vec![CreatureType::Illusion], 2, 3)
    }
}

/// Sphinx of Magosi — flying; {2}{U}: draw, then a +1/+1 counter on it.
pub fn sphinx_of_magosi() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::AddCounter {
                    what: Selector::This,
                    kind: crate::card::CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..creature("Sphinx of Magosi", cost(&[generic(3), u(), u(), u()]), vec![CreatureType::Sphinx], 6, 6)
    }
}

/// Stitcher Geralf — each player mills three; up to two creature cards milled
/// this way are exiled (the two strongest); an X/X Zombie, X their total power.
pub fn stitcher_geralf() -> CardDefinition {
    let x = || Value::PowerOf(Box::new(Selector::ExiledThisResolution { filter: R::Creature }));
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::Mill { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(3) },
                Effect::Move {
                    what: Selector::TakeGreatestPower {
                        inner: Box::new(Selector::MatchingAmong {
                            inner: Box::new(Selector::LastMoved),
                            filter: R::Creature,
                        }),
                        count: Box::new(Value::Const(2)),
                    },
                    to: ZoneDest::Exile,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: Arc::new(TokenDefinition {
                        name: "Zombie".into(),
                        card_types: vec![CardType::Creature],
                        colors: vec![Color::Blue],
                        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
                        dynamic_pt: Some((x(), x())),
                        ..Default::default()
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Stitcher Geralf", cost(&[generic(3), u(), u()]), vec![CreatureType::Human, CreatureType::Wizard], 3, 4)
    }
}

/// Stormsurge Kraken — hexproof; lieutenant: +2/+2 and "whenever this becomes
/// blocked, you may draw two cards".
pub fn stormsurge_kraken() -> CardDefinition {
    let lieutenant = || Predicate::ControlsOwnCommander { who: PlayerRef::You };
    CardDefinition {
        keywords: vec![Keyword::Hexproof],
        static_abilities: vec![StaticAbility {
            description: "Lieutenant — this creature gets +2/+2.",
            effect: StaticEffect::PumpSelfIf { condition: lieutenant(), power: 2, toughness: 2, keywords: vec![] },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource).with_filter(lieutenant()),
            effect: Effect::MayDo { description: "Draw two cards?".into(), body: Box::new(draw(2)) },
        }],
        ..creature("Stormsurge Kraken", cost(&[generic(3), u(), u()]), vec![CreatureType::Kraken], 5, 5)
    }
}

/// Teferi, Temporal Archmage — +1: of the top two, one to hand, one to the
/// bottom; −1: untap up to four target permanents; −10: an emblem letting you
/// activate loyalty abilities at instant speed. Can be your commander.
pub fn teferi_temporal_archmage() -> CardDefinition {
    CardDefinition {
        name: "Teferi, Temporal Archmage",
        cost: cost(&[generic(4), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Teferi], ..Default::default() },
        base_loyalty: 5,
        can_be_commander: true,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::LookPickToHand(Box::new(LookPick {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    ..Default::default()
                })),
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::ApplyToTargets {
                    max_targets: 4,
                    min_targets: 0,
                    filter: R::Permanent,
                    effect: Box::new(Effect::Untap { what: Selector::Target(0), up_to: None }),
                },
                ..Default::default()
            },
            LoyaltyAbility {
                loyalty_cost: -10,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Teferi, Temporal Archmage".into(),
                    triggered: vec![],
                    statics: vec![StaticAbility {
                        description: "You may activate loyalty abilities of planeswalkers you control \
                                      on any player's turn any time you could cast an instant.",
                        effect: StaticEffect::LoyaltyAbilitiesAtInstantSpeed,
                    }],
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Well of Ideas — ETB draw two; each other player draws an extra card in
/// their draw step, you draw two extra in yours.
pub fn well_of_ideas() -> CardDefinition {
    let draw_step = |scope| EventSpec::new(EventKind::StepBegins(TurnStep::Draw), scope);
    CardDefinition {
        name: "Well of Ideas",
        cost: cost(&[generic(5), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(draw(2)),
            TriggeredAbility {
                event: draw_step(EventScope::OpponentControl),
                effect: Effect::Draw { who: Selector::Player(PlayerRef::ActivePlayer), amount: Value::ONE },
            },
            TriggeredAbility { event: draw_step(EventScope::YourControl), effect: draw(2) },
        ],
        ..Default::default()
    }
}
