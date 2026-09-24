//! Commander: the cards the **Evasive Maneuvers** precon (C13, Derevi,
//! Empyrial Tactician) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_derevi.rs`.
//!
//! Residuals (each also on its card):
//! - **Curse of Inertia** — the attacking player's tap-or-untap is the
//!   engine's pick of permanent and direction.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EnchantmentSubtype, EventKind,
    EventScope, EventSpec, Keyword, LandType, SelectionRequirement as R, Selector,
    Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, LibraryPosition, PlayerRef, Predicate, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{ManaCost, cost, g, generic, u, w, x};

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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn basic() -> R {
    R::HasSupertype(Supertype::Basic).and(R::Land)
}

/// An Aura Curse: "Enchant player", cast by attaching to a target player.
fn curse(name: &'static str, mana: ManaCost, triggered_abilities: Vec<TriggeredAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura, EnchantmentSubtype::Curse],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Player) },
        triggered_abilities,
        ..Default::default()
    }
}

/// "Whenever a creature attacks enchanted player" — per attacker.
fn creature_attacks_enchanted_player(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(Predicate::SamePlayer(
            PlayerRef::DefendingPlayer,
            PlayerRef::EnchantedPlayer,
        )),
        effect,
    }
}

/// "Activate only during combat" — the combat phase's steps.
fn during_combat() -> Predicate {
    Predicate::Any(
        [
            TurnStep::BeginCombat,
            TurnStep::DeclareAttackers,
            TurnStep::DeclareBlockers,
            TurnStep::FirstStrikeDamage,
            TurnStep::CombatDamage,
            TurnStep::EndCombat,
        ]
        .into_iter()
        .map(Predicate::CurrentStepIs)
        .collect(),
    )
}

/// Aerie Mystics — flying; {1}{G}{U}: your creatures gain shroud until end of
/// turn.
pub fn aerie_mystics() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), u()]),
            effect: Effect::GrantKeyword { what: yours(R::Creature), keyword: Keyword::Shroud, duration: Duration::EndOfTurn },
            ..Default::default()
        }],
        ..creature("Aerie Mystics", cost(&[generic(4), w()]), vec![CreatureType::Bird, CreatureType::Wizard], 3, 3)
    }
}

/// Bant Panorama — {T}: {C}; {1}, {T}, sacrifice it: a basic Forest, Plains
/// or Island onto the battlefield tapped.
pub fn bant_panorama() -> CardDefinition {
    CardDefinition {
        name: "Bant Panorama",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: basic().and(
                        R::HasLandType(LandType::Forest)
                            .or(R::HasLandType(LandType::Plains))
                            .or(R::HasLandType(LandType::Island)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Borrowing 100,000 Arrows — draw a card per tapped creature target opponent
/// controls.
pub fn borrowing_100_000_arrows() -> CardDefinition {
    spell(
        "Borrowing 100,000 Arrows",
        cost(&[generic(2), u()]),
        CardType::Sorcery,
        Effect::Draw {
            who: Selector::You,
            amount: Value::CountOf(Box::new(Selector::ControlledBy {
                who: PlayerRef::ControllerOf(Box::new(target_filtered(R::OpponentPlayer))),
                filter: R::Creature.and(R::Tapped),
            })),
        },
    )
}

/// Control Magic — you control enchanted creature.
pub fn control_magic() -> CardDefinition {
    CardDefinition {
        name: "Control Magic",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: Selector::attached_to(Selector::This),
        })],
        ..Default::default()
    }
}

/// Curse of Inertia — a player attacking the cursed player may tap or untap
/// target permanent.
/// Residual: the permanent and the direction are the engine's pick.
pub fn curse_of_inertia() -> CardDefinition {
    curse(
        "Curse of Inertia",
        cost(&[generic(2), u()]),
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::AnyPlayer).with_filter(
                Predicate::AttackedDefenderWithCountAtLeast {
                    who: PlayerRef::ActivePlayer,
                    defender: PlayerRef::EnchantedPlayer,
                    at_least: 1,
                    include_planeswalkers: false,
                },
            ),
            effect: Effect::MayDoBy {
                who: PlayerRef::ActivePlayer,
                description: "Tap or untap target permanent?".into(),
                body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Permanent) }),
            },
        }],
    )
}

/// Curse of Predation — a creature attacking the cursed player gets a +1/+1
/// counter.
pub fn curse_of_predation() -> CardDefinition {
    curse(
        "Curse of Predation",
        cost(&[generic(2), g()]),
        vec![creature_attacks_enchanted_player(Effect::AddCounter {
            what: Selector::TriggerSource,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
    )
}

/// Curse of the Forsaken — a creature attacking the cursed player gains its
/// controller 1 life.
pub fn curse_of_the_forsaken() -> CardDefinition {
    curse(
        "Curse of the Forsaken",
        cost(&[generic(2), w()]),
        vec![creature_attacks_enchanted_player(Effect::GainLife {
            who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
            amount: Value::ONE,
        })],
    )
}

/// Derevi, Empyrial Tactician — flying; entering, and whenever a creature you
/// control deals combat damage to a player, you may tap or untap target
/// permanent; {1}{G}{W}{U}: put it onto the battlefield from the command zone.
pub fn derevi_empyrial_tactician() -> CardDefinition {
    let tap_or_untap = || Effect::MayDo {
        description: "Tap or untap target permanent?".into(),
        body: Box::new(Effect::TapOrUntap { what: target_filtered(R::Permanent) }),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(tap_or_untap()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Creature },
                ),
                effect: tap_or_untap(),
            },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), w(), u()]),
            from_command_zone: true,
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..creature(
            "Derevi, Empyrial Tactician",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Bird, CreatureType::Wizard],
            2,
            3,
        )
    }
}

/// Diviner Spirit — combat damage to a player: you and that player each draw
/// that many.
pub fn diviner_spirit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::TriggerEventAmount },
                Effect::Draw {
                    who: Selector::Player(PlayerRef::TriggerEventPlayer),
                    amount: Value::TriggerEventAmount,
                },
            ]),
        }],
        ..creature("Diviner Spirit", cost(&[generic(4), u()]), vec![CreatureType::Spirit], 2, 4)
    }
}

/// Djinn of Infinite Deceits — flying; {T}: exchange control of two target
/// nonlegendary creatures; not during combat.
pub fn djinn_of_infinite_deceits() -> CardDefinition {
    let nonlegendary = || R::Creature.and(R::Not(Box::new(R::HasSupertype(Supertype::Legendary))));
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::Not(Box::new(during_combat()))),
            effect: Effect::ExchangeControl {
                a: Selector::TargetFiltered { slot: 0, filter: nonlegendary() },
                b: Selector::TargetFiltered { slot: 1, filter: nonlegendary() },
            },
            ..Default::default()
        }],
        ..creature("Djinn of Infinite Deceits", cost(&[generic(4), u(), u()]), vec![CreatureType::Djinn], 2, 7)
    }
}

/// Dungeon Geists — flying; enters: tap target creature an opponent controls;
/// it doesn't untap while you control the Geists.
pub fn dungeon_geists() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::TapAndLockWhileSourcePresent {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..creature("Dungeon Geists", cost(&[generic(2), u(), u()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Hada Spy Patrol — level up {2}{U}; level 1-2: 2/2 unblockable; level 3+:
/// 3/3 unblockable with shroud (CR 702.87).
pub fn hada_spy_patrol() -> CardDefinition {
    use crate::card::LevelBand;
    CardDefinition {
        level_bands: vec![
            LevelBand { min: 1, max: Some(2), power: 2, toughness: 2, keywords: vec![Keyword::Unblockable] },
            LevelBand { min: 3, max: None, power: 3, toughness: 3, keywords: vec![Keyword::Unblockable, Keyword::Shroud] },
        ],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            sorcery_speed: true,
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::Level, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Hada Spy Patrol", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Rogue], 1, 1)
    }
}

/// Lu Xun, Scholar General — horsemanship; damage to an opponent: you may
/// draw a card.
pub fn lu_xun_scholar_general() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Horsemanship],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::TriggerEventPlayer }),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..creature(
            "Lu Xun, Scholar General",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            3,
        )
    }
}

/// Restore — a land card from any graveyard onto the battlefield under your
/// control.
pub fn restore() -> CardDefinition {
    spell(
        "Restore",
        cost(&[generic(1), g()]),
        CardType::Sorcery,
        Effect::Move {
            what: target_filtered(R::Land.and(R::InGraveyard)),
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
    )
}

/// Roon of the Hidden Realm — vigilance, trample; {2},{T}: flicker another
/// target creature until the next end step.
pub fn roon_of_the_hidden_realm() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::ExileReturnToOwnerNextEndStep {
                what: target_filtered(R::Creature.and(R::OtherThanSource)),
                tapped: false,
            },
            ..Default::default()
        }],
        ..creature(
            "Roon of the Hidden Realm",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Rhino, CreatureType::Soldier],
            4,
            4,
        )
    }
}

/// Skyward Eye Prophets — vigilance; {T}: reveal the top card; a land goes
/// onto the battlefield, anything else to hand.
pub fn skyward_eye_prophets() -> CardDefinition {
    let top = || Selector::TopOfLibrary { who: PlayerRef::You, count: Value::ONE };
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::If {
                cond: Predicate::EntityMatches { what: top(), filter: R::Land },
                then: Box::new(Effect::Move {
                    what: top(),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
                else_: Box::new(Effect::Move { what: top(), to: ZoneDest::Hand(PlayerRef::You) }),
            },
            ..Default::default()
        }],
        ..creature(
            "Skyward Eye Prophets",
            cost(&[generic(3), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    }
}

/// Surveyor's Scope — {T}, exile it: search for up to X basic lands onto the
/// battlefield, X = players with at least two more lands than you.
pub fn surveyors_scope() -> CardDefinition {
    CardDefinition {
        name: "Surveyor's Scope",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            exile_self_cost: true,
            effect: Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: basic(),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                count: Value::PlayersWithLandsAtLeastMore(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Unexpectedly Absent — put target nonland permanent into its owner's
/// library just beneath the top X cards.
pub fn unexpectedly_absent() -> CardDefinition {
    spell(
        "Unexpectedly Absent",
        cost(&[x(), w(), w()]),
        CardType::Instant,
        Effect::Move {
            what: target_filtered(R::Permanent.and(R::Nonland)),
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                pos: LibraryPosition::BeneathTopX,
            },
        },
    )
}

/// Winged Coatl — flash, flying, deathtouch.
pub fn winged_coatl() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Deathtouch],
        ..creature("Winged Coatl", cost(&[generic(1), g(), u()]), vec![CreatureType::Snake], 1, 1)
    }
}
