//! Commander: the cards the **Feline Ferocity** precon (C17, Arahbo, Roar of
//! the World) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fdc.rs` (the precon-batch module).
//!
//! Residuals (each also on its card):
//! - **Divine Reckoning** — each player keeps their highest-mana-value
//!   creature (the engine's pick, as Deadly Vanity).
//! - **Stalking Leonin** — the opponent is chosen openly by the engine (the
//!   one with the fewest creatures), not secretly by the player.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Keyword, LandType,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, spell_mastery_gate, target_filtered};
use crate::effect::{
    Effect, EventKind, EventScope, EventSpec, PlayerRef, Predicate, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, cost, g, generic, w};
use std::sync::Arc;

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

fn cats() -> R {
    R::HasCreatureType(CreatureType::Cat).and(R::ControlledByYou)
}

/// Alms Collector — flash; an opponent's draw of two or more becomes one
/// card for them and one for you (CR 614.1a).
pub fn alms_collector() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If an opponent would draw two or more cards, instead you and that player each draw a card.",
            effect: StaticEffect::OpponentMultiDrawBecomesOneEach,
        }],
        ..creature(
            "Alms Collector",
            cost(&[generic(3), w()]),
            vec![CreatureType::Cat, CreatureType::Cleric],
            3,
            4,
        )
    }
}

/// Behemoth Sledge — +2/+2, trample, lifelink. Equip {3}.
pub fn behemoth_sledge() -> CardDefinition {
    super::modern::simple_equipment(
        "Behemoth Sledge",
        cost(&[generic(1), g(), w()]),
        cost(&[generic(3)]),
        2,
        2,
        vec![Keyword::Trample, Keyword::Lifelink],
    )
}

/// Curse of Bounty — when the cursed player is attacked, you (and an
/// attacking opponent) untap your nonland permanents.
pub fn curse_of_bounty() -> CardDefinition {
    super::cmdr_edgar_c17::curse("Curse of Bounty", cost(&[generic(1), g()]), |who| Effect::Untap {
        what: Selector::ControlledBy { who, filter: R::Nonland },
        up_to: None,
    })
}

/// Divine Reckoning — each player keeps one creature; destroy the rest.
/// Flashback {5}{W}{W}.
pub fn divine_reckoning() -> CardDefinition {
    CardDefinition {
        name: "Divine Reckoning",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(5), w(), w()]))],
        effect: Effect::EachPlayerKeepsOneSacrificeRest {
            who: Selector::Player(PlayerRef::EachPlayer),
            filter: R::Creature,
            destroy: true,
        },
        ..Default::default()
    }
}

/// Hungry Lynx — your Cats have protection from Rats; each of your end steps
/// hands an opponent a deathtouch Rat; each Rat death grows your Cats.
pub fn hungry_lynx() -> CardDefinition {
    let rat = TokenDefinition {
        name: "Rat".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rat], ..Default::default() },
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    };
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Cats you control have protection from Rats.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(cats()),
                keyword: Keyword::ProtectionFromCreatureType(CreatureType::Rat),
            },
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
                effect: Effect::TargetPlayerThen {
                    filter: R::Player.and(R::ControlledByOpponent),
                    then: Box::new(Effect::CreateToken {
                        who: PlayerRef::Target(0),
                        count: Value::ONE,
                        definition: Arc::new(rat),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::HasCreatureType(CreatureType::Rat),
                    },
                ),
                effect: Effect::AddCounter {
                    what: Selector::EachPermanent(cats().and(R::Creature)),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
            },
        ],
        ..creature("Hungry Lynx", cost(&[generic(1), g()]), vec![CreatureType::Cat], 2, 2)
    }
}

/// Jedit Ojanen of Efrava — forestwalk; attacking or blocking makes a 2/2
/// forestwalking Cat Warrior.
pub fn jedit_ojanen_of_efrava() -> CardDefinition {
    let make = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: Arc::new(TokenDefinition {
            name: "Cat Warrior".into(),
            power: 2,
            toughness: 2,
            card_types: vec![CardType::Creature],
            colors: vec![Color::Green],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Cat, CreatureType::Warrior],
                ..Default::default()
            },
            keywords: vec![Keyword::Landwalk(LandType::Forest)],
            ..Default::default()
        }),
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: make(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::SelfSource),
                effect: make(),
            },
        ],
        ..creature(
            "Jedit Ojanen of Efrava",
            cost(&[generic(3), g(), g(), g()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            5,
            5,
        )
    }
}

/// Kindred Summons — choose a type; reveal until X creature cards of it,
/// X = your creatures of that type; they enter, the rest shuffle in.
pub fn kindred_summons() -> CardDefinition {
    let chosen = || R::Creature.and(R::IsSourceChosenCreatureType);
    CardDefinition {
        name: "Kindred Summons",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseCreatureTypeThen {
            who: PlayerRef::You,
            then: Box::new(Effect::RevealUntilMatchingToBattlefield {
                filter: chosen(),
                count: Value::CountOf(Box::new(Selector::EachPermanent(
                    chosen().and(R::ControlledByYou),
                ))),
            }),
        },
        ..Default::default()
    }
}

/// Mirri, Weatherlight Duelist — first strike; on attack each opponent
/// blocks with at most one creature; while tapped, at most one creature can
/// attack you each combat.
pub fn mirri_weatherlight_duelist() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![StaticAbility {
            description: "As long as Mirri is tapped, no more than one creature can attack you each combat.",
            effect: StaticEffect::AttackerCapAgainstControllerWhileTapped { n: 1 },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::OpponentsBlockWithAtMost { n: 1 },
        }],
        ..creature(
            "Mirri, Weatherlight Duelist",
            cost(&[generic(1), g(), w()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            3,
            2,
        )
    }
}

/// Nissa's Pilgrimage — up to two basic Forests (three with spell mastery):
/// one onto the battlefield tapped, the rest to hand.
pub fn nissas_pilgrimage() -> CardDefinition {
    let forest = || R::IsBasicLand.and(R::HasLandType(LandType::Forest));
    let rest = |n: i32| Effect::SearchUpToN {
        who: PlayerRef::You,
        filter: forest(),
        to: ZoneDest::Hand(PlayerRef::You),
        count: Value::Const(n),
    };
    CardDefinition {
        name: "Nissa's Pilgrimage",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: forest(),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::ONE,
            },
            Effect::If {
                cond: spell_mastery_gate(),
                then: Box::new(rest(2)),
                else_: Box::new(rest(1)),
            },
        ]),
        ..Default::default()
    }
}

/// Qasali Slingers — reach; when it or another Cat of yours enters, you may
/// destroy an artifact or enchantment.
pub fn qasali_slingers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Cat),
                },
            ),
            effect: Effect::MayDo {
                description: "Destroy target artifact or enchantment?".into(),
                body: Box::new(Effect::Destroy {
                    what: target_filtered(R::Artifact.or(R::Enchantment)),
                }),
            },
        }],
        ..creature(
            "Qasali Slingers",
            cost(&[generic(4), g()]),
            vec![CreatureType::Cat, CreatureType::Warrior],
            3,
            5,
        )
    }
}

/// Quietus Spike — deathtouch; combat damage to a player halves their life,
/// rounded up. Equip {3}.
pub fn quietus_spike() -> CardDefinition {
    let mut d = super::modern::simple_equipment(
        "Quietus Spike",
        cost(&[generic(3)]),
        cost(&[generic(3)]),
        0,
        0,
        vec![Keyword::Deathtouch],
    );
    if let Some(b) = d.equipped_bonus.as_mut() {
        b.triggered_abilities.push(TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::LoseHalfLife {
                who: Selector::Player(PlayerRef::Target(0)),
                rounded_up: true,
            },
        });
    }
    d
}

/// Saltcrusted Steppe — {T}: {C}; {1}, {T}: a storage counter; {1}, remove
/// X storage counters: X mana in any mix of {G} and {W}.
pub fn saltcrusted_steppe() -> CardDefinition {
    crate::sets::storage_land("Saltcrusted Steppe", Color::Green, Color::White)
}

/// Seht's Tiger — flash; you gain protection from the color of your choice
/// until end of turn as it enters.
pub fn sehts_tiger() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::PlayerGainsProtectionFromChosenColor {
            who: PlayerRef::You,
        })],
        ..creature("Seht's Tiger", cost(&[generic(2), w(), w()]), vec![CreatureType::Cat], 3, 3)
    }
}

/// Spirit of the Hearth — flying; you have hexproof.
pub fn spirit_of_the_hearth() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "You have hexproof.",
            effect: StaticEffect::ControllerHasHexproof,
        }],
        ..creature(
            "Spirit of the Hearth",
            cost(&[generic(4), w(), w()]),
            vec![CreatureType::Cat, CreatureType::Spirit],
            4,
            5,
        )
    }
}

/// Stalking Leonin — chooses an opponent as it enters; once, exile a
/// creature attacking you that the chosen player controls.
pub fn stalking_leonin() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::ChooseOpponentThen {
            then: Box::new(Effect::RememberPlayerOnSource { who: PlayerRef::ChosenPlayerOfSource }),
        })],
        activated_abilities: vec![ActivatedAbility {
            activate_once: true,
            // Attacking creatures are the active player's (CR 506.2).
            effect: Effect::If {
                cond: Predicate::IsTurnOf(PlayerRef::ChosenPlayerOfSource),
                then: Box::new(Effect::Move {
                    what: target_filtered(R::Creature.and(R::IsAttackingYou)),
                    to: ZoneDest::Exile,
                }),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..creature(
            "Stalking Leonin",
            cost(&[generic(2), w()]),
            vec![CreatureType::Cat, CreatureType::Archer],
            3,
            3,
        )
    }
}

/// Sunspear Shikari — first strike and lifelink while equipped.
pub fn sunspear_shikari() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is equipped, it has first strike and lifelink.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsEquipped,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
            },
        }],
        ..creature(
            "Sunspear Shikari",
            cost(&[generic(1), w()]),
            vec![CreatureType::Cat, CreatureType::Soldier],
            2,
            2,
        )
    }
}

/// Traverse the Outlands — up to X basic lands onto the battlefield tapped,
/// X the greatest power among your creatures.
pub fn traverse_the_outlands() -> CardDefinition {
    CardDefinition {
        name: "Traverse the Outlands",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            count: Value::GreatestPowerControlled { who: PlayerRef::You },
        },
        ..Default::default()
    }
}
