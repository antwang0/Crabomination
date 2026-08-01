//! Mercadian Masques (MMQ) gap closure, second wave. Tests in
//! `classic_sets/mmq2`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, Predicate,
    SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, TokenDefinition,
    TriggeredAbility, Value, WardCost,
};
use crate::effect::{
    Duration, Effect, ManaPayload, PlayerRef, Selector, ZoneDest,
    shortcut::{etb, target, target_any, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

// ── Shared shapes ───────────────────────────────────────────────────────────

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

fn artifact(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Artifact], ..Default::default() }
}

fn enchantment(name: &'static str, c: ManaCost) -> CardDefinition {
    CardDefinition { name, cost: c, card_types: vec![CardType::Enchantment], ..Default::default() }
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

fn each_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer),
        effect,
    }
}

fn your_upkeep(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
        effect,
    }
}

/// "You may sacrifice `count` `land_type`s rather than pay this spell's mana
/// cost" — the MMQ free-spell cycle.
fn sac_lands_alt(land_type: LandType, count: u32) -> AlternativeCost {
    AlternativeCost {
        sacrifice_permanents: Some((R::HasLandType(land_type), count)),
        ..Default::default()
    }
}

/// "You may exile a `color` card from your hand rather than pay this spell's
/// mana cost" — the MMQ pitch cycle.
fn pitch_alt(color: Color) -> AlternativeCost {
    AlternativeCost { exile_filter: Some(R::HasColor(color)), ..Default::default() }
}

/// The MMQ Monger cycle's shared line: `{2}: [effect]. Any player may activate
/// this ability.`
fn monger_ability(effect: Effect) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: cost(&[generic(2)]),
        any_player: true,
        effect,
        ..Default::default()
    }
}

/// The Flailing cycle's pair of any-player pumps.
fn flailing_pumps() -> Vec<ActivatedAbility> {
    let pump = |power: i32| ActivatedAbility {
        mana_cost: cost(&[generic(1)]),
        any_player: true,
        effect: Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(power),
            toughness: Value::Const(power),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    };
    vec![pump(1), pump(-1)]
}

// ── Free / alternative-cost spells ──────────────────────────────────────────

/// Thunderclap — {2}{R} or a Mountain. Three damage to a creature.
pub fn thunderclap() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sac_lands_alt(LandType::Mountain, 1)),
        ..instant(
            "Thunderclap",
            cost(&[generic(2), r()]),
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(3),
            },
        )
    }
}

/// Crash — {2}{R} or a Mountain. Destroy target artifact.
pub fn crash() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sac_lands_alt(LandType::Mountain, 1)),
        ..instant(
            "Crash",
            cost(&[generic(2), r()]),
            Effect::Destroy { what: target_filtered(R::Artifact) },
        )
    }
}

/// Pulverize — {4}{R}{R} or two Mountains. Destroy all artifacts.
pub fn pulverize() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(sac_lands_alt(LandType::Mountain, 2)),
        ..sorcery(
            "Pulverize",
            cost(&[generic(4), r(), r()]),
            Effect::Destroy { what: Selector::EachPermanent(R::Artifact) },
        )
    }
}

/// Thwart — {2}{U}{U} or three Islands to hand. Counter target spell.
pub fn thwart() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            return_to_hand: Some((R::HasLandType(LandType::Island), 3)),
            ..Default::default()
        }),
        ..instant(
            "Thwart",
            cost(&[generic(2), u(), u()]),
            Effect::CounterSpell { what: target() },
        )
    }
}

/// Tidal Bore — {1}{U} or an Island to hand. Tap or untap a creature.
pub fn tidal_bore() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            return_to_hand: Some((R::HasLandType(LandType::Island), 1)),
            ..Default::default()
        }),
        ..instant(
            "Tidal Bore",
            cost(&[generic(1), u()]),
            Effect::TapOrUntap { what: target_filtered(R::Creature) },
        )
    }
}

/// Cave-In — {3}{R}{R} or a red card from hand. Two damage to everything.
pub fn cave_in() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(pitch_alt(Color::Red)),
        ..sorcery(
            "Cave-In",
            cost(&[generic(3), r(), r()]),
            Effect::Seq(vec![
                Effect::DealDamage {
                    to: Selector::EachPermanent(R::Creature),
                    amount: Value::Const(2),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachPlayer),
                    amount: Value::Const(2),
                },
            ]),
        )
    }
}

/// Reverent Mantra — {3}{W} or a white card from hand. Team-wide protection.
pub fn reverent_mantra() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(pitch_alt(Color::White)),
        ..instant(
            "Reverent Mantra",
            cost(&[generic(3), w()]),
            Effect::GrantProtectionFromChosenColor {
                what: Selector::EachPermanent(R::Creature),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

/// Vine Dryad — {3}{G} or a green card from hand. 1/3 flash forestwalker.
pub fn vine_dryad() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Landwalk(LandType::Forest)],
        alternative_cost: Some(pitch_alt(Color::Green)),
        ..creature("Vine Dryad", cost(&[generic(3), g()]), vec![CreatureType::Dryad], 1, 3)
    }
}

/// Delraich — {6}{B} or three black creatures. 6/6 trampler.
pub fn delraich() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        alternative_cost: Some(AlternativeCost {
            sacrifice_permanents: Some((R::Creature.and(R::HasColor(Color::Black)), 3)),
            ..Default::default()
        }),
        ..creature("Delraich", cost(&[generic(6), b()]), vec![CreatureType::Horror], 6, 6)
    }
}

/// Rouse — {1}{B} or 2 life while you control a Swamp. +2/+0.
pub fn rouse() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            life_cost: 2,
            condition: Some(Predicate::ValueAtLeast(
                Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(
                        R::HasLandType(LandType::Swamp).and(R::ControlledByYou),
                    )),
                    filter: R::Any,
                },
                Value::Const(1),
            )),
            ..Default::default()
        }),
        ..instant(
            "Rouse",
            cost(&[generic(1), b()]),
            Effect::PumpPT {
                what: target_filtered(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        )
    }
}

// ── Mongers ("Any player may activate") ─────────────────────────────────────

/// Sailmonger — {3}{U} 3/3. Anyone can buy flying off it.
pub fn sailmonger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monger_ability(Effect::GrantKeyword {
            what: target_filtered(R::Creature),
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Sailmonger",
            cost(&[generic(3), u()]),
            vec![CreatureType::Human, CreatureType::Monger],
            3,
            3,
        )
    }
}

/// Warmonger — {3}{R} 3/3. Anyone can rake the ground and every player.
pub fn warmonger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monger_ability(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(
                    R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                ),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(1),
            },
        ]))],
        ..creature(
            "Warmonger",
            cost(&[generic(3), r()]),
            vec![CreatureType::Minotaur, CreatureType::Monger],
            3,
            3,
        )
    }
}

/// Squallmonger — {3}{G} 3/3. The anti-air half of the Monger cycle.
pub fn squallmonger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monger_ability(Effect::Seq(vec![
            Effect::DealDamage {
                to: Selector::EachPermanent(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                amount: Value::Const(1),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(1),
            },
        ]))],
        ..creature(
            "Squallmonger",
            cost(&[generic(3), g()]),
            vec![CreatureType::Monger],
            3,
            3,
        )
    }
}

/// Wishmonger — {3}{W} 3/3. Anyone can buy a colour of protection.
pub fn wishmonger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![monger_ability(Effect::GrantProtectionFromChosenColor {
            what: target_filtered(R::Creature),
            duration: Duration::EndOfTurn,
        })],
        ..creature(
            "Wishmonger",
            cost(&[generic(3), w()]),
            vec![CreatureType::Unicorn, CreatureType::Monger],
            3,
            3,
        )
    }
}

/// Scandalmonger — {3}{B} 3/3. Anyone can strip a card, at sorcery speed.
pub fn scandalmonger() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            ..monger_ability(Effect::Discard {
                who: target_filtered(R::Player),
                amount: Value::ONE,
                random: false,
            })
        }],
        ..creature(
            "Scandalmonger",
            cost(&[generic(3), b()]),
            vec![CreatureType::Boar, CreatureType::Monger],
            3,
            3,
        )
    }
}

/// Flailing Ogre — {2}{R} 3/3 anyone can pump or shrink.
pub fn flailing_ogre() -> CardDefinition {
    CardDefinition {
        activated_abilities: flailing_pumps(),
        ..creature("Flailing Ogre", cost(&[generic(2), r()]), vec![CreatureType::Ogre], 3, 3)
    }
}

/// Flailing Soldier — {R} 2/2 with the same two-way handle.
pub fn flailing_soldier() -> CardDefinition {
    CardDefinition {
        activated_abilities: flailing_pumps(),
        ..creature(
            "Flailing Soldier",
            cost(&[r()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Flailing Manticore — {3}{R} 3/3 flying first striker with the handle.
pub fn flailing_manticore() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        activated_abilities: flailing_pumps(),
        ..creature(
            "Flailing Manticore",
            cost(&[generic(3), r()]),
            vec![CreatureType::Manticore],
            3,
            3,
        )
    }
}

// ── Rishadan pirates ────────────────────────────────────────────────────────

fn rishadan(
    name: &'static str,
    c: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
    tax: u32,
) -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ForEach {
            selector: Selector::Player(PlayerRef::EachOpponent),
            body: Box::new(Effect::UnlessPlayerPays {
                who: PlayerRef::Triggerer,
                cost: WardCost::Mana(cost(&[generic(tax)])),
                then: Box::new(Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::Triggerer),
                    count: Value::ONE,
                    filter: R::Permanent,
                }),
                if_paid: None,
            }),
        })],
        ..creature(name, c, types, p, t)
    }
}

/// Rishadan Cutpurse — {2}{U} 1/1. Each opponent pays {1} or loses a permanent.
pub fn rishadan_cutpurse() -> CardDefinition {
    rishadan(
        "Rishadan Cutpurse",
        cost(&[generic(2), u()]),
        vec![CreatureType::Human, CreatureType::Pirate],
        1,
        1,
        1,
    )
}

/// Rishadan Footpad — {3}{U} 2/2 at a {2} toll.
pub fn rishadan_footpad() -> CardDefinition {
    rishadan(
        "Rishadan Footpad",
        cost(&[generic(3), u()]),
        vec![CreatureType::Human, CreatureType::Pirate],
        2,
        2,
        2,
    )
}

/// Rishadan Brigand — {4}{U} 3/2 flier at a {3} toll; blocks only fliers.
pub fn rishadan_brigand() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::CanBlockOnlyFlying],
        ..rishadan(
            "Rishadan Brigand",
            cost(&[generic(4), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            3,
            2,
            3,
        )
    }
}

// ── Enchantments ────────────────────────────────────────────────────────────

/// Uphill Battle — {2}{R}. Opponents' creatures arrive tapped.
pub fn uphill_battle() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures played by your opponents enter tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByOpponent),
                ),
            },
        }],
        ..enchantment("Uphill Battle", cost(&[generic(2), r()]))
    }
}

/// Fountain Watch — {3}{W}{W} 2/4. Your artifacts and enchantments get shroud.
pub fn fountain_watch() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Artifacts and enchantments you control have shroud.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Artifact.or(R::Enchantment).and(R::ControlledByYou),
                ),
                keyword: Keyword::Shroud,
            },
        }],
        ..creature(
            "Fountain Watch",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            2,
            4,
        )
    }
}

/// Vernal Equinox — {3}{G}. Creature and enchantment spells gain flash.
pub fn vernal_equinox() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Any player may cast creature and enchantment spells as though they had \
                          flash.",
            effect: StaticEffect::AnyPlayerSpellsHaveFlash {
                filter: R::Creature.or(R::Enchantment),
            },
        }],
        ..enchantment("Vernal Equinox", cost(&[generic(3), g()]))
    }
}

/// Noble Purpose — {3}{W}{W}. Your creatures' combat damage feeds you.
pub fn noble_purpose() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamage, EventScope::YourControl),
            effect: Effect::GainLife { who: Selector::You, amount: Value::TriggerEventAmount },
        }],
        ..enchantment("Noble Purpose", cost(&[generic(3), w(), w()]))
    }
}

/// Furious Assault — {2}{R}. Every creature spell you cast throws a spark.
pub fn furious_assault() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::ONE,
            },
        }],
        ..enchantment("Furious Assault", cost(&[generic(2), r()]))
    }
}

/// Kyren Negotiations — {2}{R}{R}. Tap your team for reach.
pub fn kyren_negotiations() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_other_filter: Some(R::Creature),
            effect: Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..enchantment("Kyren Negotiations", cost(&[generic(2), r(), r()]))
    }
}

/// Snake Pit — {3}{G}. Punish blue and black.
pub fn snake_pit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                Predicate::CastSpellMatches(
                    R::HasColor(Color::Blue).or(R::HasColor(Color::Black)),
                ),
            ),
            effect: Effect::MayDo {
                description: "Create a 1/1 green Snake creature token".into(),
                body: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Snake".into(),
                        power: 1,
                        toughness: 1,
                        colors: vec![Color::Green],
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Snake],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                }),
            },
        }],
        ..enchantment("Snake Pit", cost(&[generic(3), g()]))
    }
}

/// Embargo — {3}{U}. Nothing untaps, and it bleeds you dry.
pub fn embargo() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Nonland permanents don't untap during their controllers' untap steps.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::EachPermanent(R::Nonland),
            },
        }],
        triggered_abilities: vec![your_upkeep(Effect::LoseLife {
            who: Selector::You,
            amount: Value::Const(2),
        })],
        ..enchantment("Embargo", cost(&[generic(3), u()]))
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Worry Beads — {3}. Everyone mills one on their upkeep.
pub fn worry_beads() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![each_upkeep(Effect::Mill {
            who: Selector::Player(PlayerRef::ActivePlayer),
            amount: Value::ONE,
        })],
        ..artifact("Worry Beads", cost(&[generic(3)]))
    }
}

/// Distorting Lens — {2}. Repaint any permanent for a turn.
pub fn distorting_lens() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeChosenColor {
                what: target_filtered(R::Permanent),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..artifact("Distorting Lens", cost(&[generic(2)]))
    }
}

/// Kyren Toy — {3}. Bank charge counters, cash them in for {C} plus one.
pub fn kyren_toy() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_x: Some(CounterType::Charge),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Sum(vec![
                        Value::XFromCost,
                        Value::ONE,
                    ])),
                },
                ..Default::default()
            },
        ],
        ..artifact("Kyren Toy", cost(&[generic(3)]))
    }
}

/// Magistrate's Scepter — {3}. Three charge counters buy an extra turn.
pub fn magistrates_scepter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Charge,
                    amount: Value::ONE,
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                remove_counter_cost: Some((CounterType::Charge, 3)),
                effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
                ..Default::default()
            },
        ],
        ..artifact("Magistrate's Scepter", cost(&[generic(3)]))
    }
}

/// Rishadan Pawnshop — {2}. Shuffle your own permanent away.
pub fn rishadan_pawnshop() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::NotToken)),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOfMoved,
                    pos: crate::effect::LibraryPosition::Shuffled,
                },
            },
            ..Default::default()
        }],
        ..artifact("Rishadan Pawnshop", cost(&[generic(2)]))
    }
}

/// Mercadian Atlas — {5}. A card for a turn you didn't make a land drop.
pub fn mercadian_atlas() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::ValueAtMost(
                    Value::LandsPlayedThisTurn(PlayerRef::You),
                    Value::Const(0),
                )),
            effect: Effect::MayDo {
                description: "Draw a card".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..artifact("Mercadian Atlas", cost(&[generic(5)]))
    }
}

/// Panacea — {4}. `{X}{X}, {T}`: prevent the next X damage.
pub fn panacea() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[x(), x()]),
            tap_cost: true,
            effect: Effect::PreventNextDamage { target: target_any(), amount: Value::XFromCost },
            ..Default::default()
        }],
        ..artifact("Panacea", cost(&[generic(4)]))
    }
}

/// Tower of the Magistrate — a land that shrugs off Equipment.
pub fn tower_of_the_magistrate() -> CardDefinition {
    CardDefinition {
        name: "Tower of the Magistrate",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            super::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::GrantKeyword {
                    what: target_filtered(R::Creature),
                    keyword: Keyword::ProtectionFromMatching(Box::new(R::Artifact)),
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Blockade Runner — {3}{U} 2/2 that buys its way through.
pub fn blockade_runner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Blockade Runner", cost(&[generic(3), u()]), vec![CreatureType::Merfolk], 2, 2)
    }
}

/// Kyren Sniper — {2}{R} 1/1 that plinks each upkeep.
pub fn kyren_sniper() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_upkeep(Effect::MayDo {
            description: "Deal 1 damage to target player or planeswalker".into(),
            body: Box::new(Effect::DealDamage {
                to: target_filtered(R::Player.or(R::Planeswalker)),
                amount: Value::ONE,
            }),
        })],
        ..creature("Kyren Sniper", cost(&[generic(2), r()]), vec![CreatureType::Goblin], 1, 1)
    }
}

/// Silverglade Elemental — {4}{G} 4/4 that fetches a Forest.
pub fn silverglade_elemental() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search your library for a Forest card and put it onto the battlefield"
                .into(),
            body: Box::new(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Forest),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        ..creature(
            "Silverglade Elemental",
            cost(&[generic(4), g()]),
            vec![CreatureType::Elemental],
            4,
            4,
        )
    }
}

/// Howling Wolf — {2}{G}{G} 2/2 that calls up to three of its packmates.
pub fn howling_wolf() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Search your library for up to three cards named Howling Wolf".into(),
            body: Box::new(Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: R::HasName("Howling Wolf".into()),
                to: ZoneDest::Hand(PlayerRef::You),
                count: Value::Const(3),
            }),
        })],
        ..creature("Howling Wolf", cost(&[generic(2), g(), g()]), vec![CreatureType::Wolf], 2, 2)
    }
}

/// Groundskeeper — {G} 1/1 that recycles basics.
pub fn groundskeeper() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::IsBasicLand)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..creature(
            "Groundskeeper",
            cost(&[g()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

/// Trap Runner — {2}{W}{W} 2/3 that pins an unblocked attacker.
pub fn trap_runner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::BecomeBlocked {
                what: target_filtered(R::Creature.and(R::IsAttacking).and(R::IsUnblocked)),
            },
            ..Default::default()
        }],
        ..creature(
            "Trap Runner",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            2,
            3,
        )
    }
}

/// Wall of Distortion — {2}{B}{B} 1/3 Wall with a sorcery-speed strip.
pub fn wall_of_distortion() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::Discard {
                who: target_filtered(R::Player),
                amount: Value::ONE,
                random: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Wall of Distortion",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Wall],
            1,
            3,
        )
    }
}

/// Silent Assassin — {B}{B} 2/1 that murders a blocker after combat.
pub fn silent_assassin() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), b()]),
            effect: Effect::AtEndOfCombat {
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Creature.and(R::IsBlocking)),
                }),
            },
            ..Default::default()
        }],
        ..creature(
            "Silent Assassin",
            cost(&[b(), b()]),
            vec![CreatureType::Human, CreatureType::Mercenary, CreatureType::Assassin],
            2,
            1,
        )
    }
}

/// Revered Elder — {2}{W} 1/2 that shrugs off pings.
pub fn revered_elder() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            effect: Effect::PreventNextDamage { target: Selector::This, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature(
            "Revered Elder",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            2,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Land Grant — {1}{G}, free on an empty land hand. Fetch a Forest.
pub fn land_grant() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            condition: Some(Predicate::ValueAtMost(
                Value::CardsInHandMatching { who: PlayerRef::You, filter: R::Land },
                Value::Const(0),
            )),
            ..Default::default()
        }),
        ..sorcery(
            "Land Grant",
            cost(&[generic(1), g()]),
            Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Forest),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        )
    }
}

/// Moment of Silence — {W}. Skip a player's next combat phase.
pub fn moment_of_silence() -> CardDefinition {
    instant(
        "Moment of Silence",
        cost(&[w()]),
        Effect::SkipNextCombatPhase { who: PlayerRef::Target(0) },
    )
}

/// Misstep — {1}{U}. A player's creatures stay down.
pub fn misstep() -> CardDefinition {
    sorcery(
        "Misstep",
        cost(&[generic(1), u()]),
        Effect::CreaturesDontUntapNextUntapStep { who: target_filtered(R::Player) },
    )
}

/// Natural Affinity — {2}{G}. All lands become 2/2 creatures.
pub fn natural_affinity() -> CardDefinition {
    instant(
        "Natural Affinity",
        cost(&[generic(2), g()]),
        Effect::BecomeCreature {
            what: Selector::EachPermanent(R::Land),
            power: Value::Const(2),
            toughness: Value::Const(2),
            creature_types: vec![],
            keywords: vec![],
            duration: Duration::EndOfTurn,
        },
    )
}

/// Tectonic Break — {X}{R}{R}. Each player sacrifices X lands.
pub fn tectonic_break() -> CardDefinition {
    sorcery(
        "Tectonic Break",
        cost(&[x(), r(), r()]),
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::XFromCost,
            filter: R::Land,
        },
    )
}

/// Honor the Fallen — {1}{W}. Exile every creature card in every graveyard.
pub fn honor_the_fallen() -> CardDefinition {
    instant(
        "Honor the Fallen",
        cost(&[generic(1), w()]),
        Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::CountMatching {
                    sel: Box::new(Selector::CardsInZone {
                        who: PlayerRef::EachPlayer,
                        zone: crate::card::Zone::Graveyard,
                        filter: R::Creature,
                    }),
                    filter: R::Any,
                },
            },
            Effect::Move {
                what: Selector::CardsInZone {
                    who: PlayerRef::EachPlayer,
                    zone: crate::card::Zone::Graveyard,
                    filter: R::Creature,
                },
                to: ZoneDest::Exile,
            },
        ]),
    )
}

/// Haunted Crossroads — {2}{B}. Buy back a creature onto your library.
pub fn haunted_crossroads() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[b()]),
            effect: Effect::Move {
                what: target_filtered(R::InYourGraveyard.and(R::Creature)),
                to: ZoneDest::Library {
                    who: PlayerRef::You,
                    pos: crate::effect::LibraryPosition::Top,
                },
            },
            ..Default::default()
        }],
        ..enchantment("Haunted Crossroads", cost(&[generic(2), b()]))
    }
}

/// Trade Routes — {1}{U}. Bounce your lands, or trade one for a card.
pub fn trade_routes() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Move {
                    what: target_filtered(R::Land.and(R::ControlledByYou)),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                discard_cost: Some((R::Land, 1)),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                ..Default::default()
            },
        ],
        ..enchantment("Trade Routes", cost(&[generic(1), u()]))
    }
}
