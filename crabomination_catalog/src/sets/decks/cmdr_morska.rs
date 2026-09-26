//! Commander: the cards the **Deep Clue Sea** precon (MKC, Morska, Undersea
//! Sleuth) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_morska.rs`.
//!
//! Residuals (each also on its card):
//! - **Aerial Extortionist** — "up to one target" always takes one.
//! - **Alandra, Sky Dreamer** — the fifth-card trigger reads "five or more,
//!   once a turn", so it fires on a later draw if Alandra arrived after the
//!   fifth.
//! - **Erdwal Illuminator** — "you investigate" reads "a Clue token is created
//!   under your control"; first-each-turn counts Clues made before it arrived.

use std::sync::Arc;

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LoyaltyAbility, PlaneswalkerSubtype, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{counter_target_spell, etb, investigate, on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, u, w, Color, ManaCost};

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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn clue() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Clue)
}

/// "Whenever you draw your second card each turn" (CR 121).
fn second_draw(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::SecondCardDrawnThisTurn, EventScope::YourControl), effect }
}

/// "Whenever you sacrifice a Clue".
fn clue_sacrificed(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: clue() }),
        effect,
    }
}

fn token(name: &str, p: i32, t: i32, colors: Vec<Color>, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors,
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        ..Default::default()
    }
}

fn make(definition: TokenDefinition) -> Effect {
    Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(definition) }
}

/// Morska, Undersea Sleuth — no maximum hand size; investigate each upkeep;
/// your second draw each turn puts two +1/+1 counters on it.
pub fn morska_undersea_sleuth() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "You have no maximum hand size.",
            effect: StaticEffect::NoMaximumHandSize,
        }],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: investigate(1),
            },
            second_draw(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            }),
        ],
        ..legend(
            "Morska, Undersea Sleuth",
            cost(&[g(), w(), u()]),
            vec![CreatureType::Vedalken, CreatureType::Fish, CreatureType::Detective],
            2,
            3,
        )
    }
}

/// Armed with Proof — investigate twice as it enters; your Clues are
/// Equipment with "Equipped creature gets +2/+0" and equip {2}.
pub fn armed_with_proof() -> CardDefinition {
    CardDefinition {
        name: "Armed with Proof",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![etb(investigate(2))],
        static_abilities: vec![StaticAbility {
            description: "Clues you control are Equipment in addition to their other types and have \"Equipped creature gets +2/+0\" and equip {2}.",
            effect: StaticEffect::MatchingArtifactsAreEquipment {
                filter: clue().and(R::ControlledByYou),
                equip: cost(&[generic(2)]),
                power: 2,
                filtered_equip: None,
            },
        }],
        ..Default::default()
    }
}

/// Serene Sleuth — investigates as it enters; at the beginning of combat on
/// your turn, investigate for each goaded creature you control, then each
/// creature you control is no longer goaded (CR 701.15).
pub fn serene_sleuth() -> CardDefinition {
    let mine = || R::Creature.and(R::ControlledByYou);
    CardDefinition {
        triggered_abilities: vec![
            etb(investigate(1)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::CountOf(Box::new(Selector::EachPermanent(mine().and(R::IsGoaded)))),
                        definition: Arc::new(crabomination_base::tokens::clue_token()),
                    },
                    Effect::Ungoad { what: Selector::EachPermanent(mine()) },
                ]),
            },
        ],
        ..creature("Serene Sleuth", cost(&[generic(1), w()]), vec![CreatureType::Human, CreatureType::Detective], 2, 2)
    }
}

/// Detective of the Month — ascend; with the city's blessing your Detectives
/// can't be blocked; your second draw each turn makes a 2/2 Detective.
pub fn detective_of_the_month() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![
            crate::sets::ascend(),
            StaticAbility {
                description: "As long as you have the city's blessing, Detectives you control can't be blocked.",
                effect: StaticEffect::WhileCondition {
                    condition: Predicate::HasCityBlessing { who: PlayerRef::You },
                    inner: Box::new(StaticEffect::GrantKeyword {
                        applies_to: yours(R::Creature.and(R::HasCreatureType(CreatureType::Detective))),
                        keyword: Keyword::Unblockable,
                    }),
                },
            },
        ],
        triggered_abilities: vec![second_draw(make(crabomination_base::tokens::detective_token()))],
        ..creature(
            "Detective of the Month",
            cost(&[generic(2), u()]),
            vec![CreatureType::Human, CreatureType::Detective],
            2,
            3,
        )
    }
}

/// Follow the Bodies — gravestorm; investigate.
pub fn follow_the_bodies() -> CardDefinition {
    CardDefinition {
        name: "Follow the Bodies",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Gravestorm],
        effect: investigate(1),
        ..Default::default()
    }
}

/// Tangletrove Kelp — ward {2}; at the beginning of each combat, your other
/// Clues become 6/6 Plant creatures until end of turn; {2}, sacrifice it: draw.
pub fn tangletrove_kelp() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant],
            artifact_subtypes: vec![ArtifactSubtype::Clue],
            ..Default::default()
        },
        keywords: vec![Keyword::Ward(WardCost::generic(2))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::AnyPlayer),
            effect: Effect::BecomeCreature {
                what: yours(clue().and(R::OtherThanSource)),
                power: Value::Const(6),
                toughness: Value::Const(6),
                creature_types: vec![CreatureType::Plant],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Tangletrove Kelp", cost(&[generic(5), u(), u()]), vec![], 6, 6)
    }
}

/// Innocuous Researcher — parley on attack, investigating per nonland card;
/// at your end step you may untap your lands, and then can't cast spells until
/// your next turn.
pub fn innocuous_researcher() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            on_attack(Effect::Parley {
                then: Box::new(Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::CardsRevealedThisEffect,
                    definition: Arc::new(crabomination_base::tokens::clue_token()),
                }),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::MayDo {
                    description: "Untap all lands you control? You can't cast spells until your next turn.".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Untap { what: yours(R::Land), up_to: None },
                        Effect::SilencePlayersUntilTheirNextTurn { who: PlayerRef::You },
                    ])),
                },
            },
        ],
        ..creature(
            "Innocuous Researcher",
            cost(&[generic(3), g()]),
            vec![CreatureType::Centaur, CreatureType::Detective],
            3,
            4,
        )
    }
}

/// Knowledge Is Power — your creatures get +X/+X, X = cards you've drawn this
/// turn.
pub fn knowledge_is_power() -> CardDefinition {
    CardDefinition {
        name: "Knowledge Is Power",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +X/+X, where X is the number of cards you've drawn this turn.",
            effect: StaticEffect::PumpPTByValue {
                applies_to: yours(R::Creature),
                power: Value::CardsDrawnThisTurn(PlayerRef::You),
                toughness: Value::CardsDrawnThisTurn(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Aerial Extortionist — flying; entering or hitting a player exiles up to one
/// target nonland permanent its owner may cast while it stays exiled; another
/// player casting a spell from anywhere but their hand draws you a card.
/// ⚠ "Up to one" always takes a target.
pub fn aerial_extortionist() -> CardDefinition {
    let extort = || {
        Effect::Seq(vec![
            Effect::Move { what: target_filtered(R::Permanent.and(R::Nonland)), to: ZoneDest::Exile },
            Effect::GrantMayPlay {
                what: Selector::LastMoved,
                duration: crate::card::MayPlayDuration::WhileExiled,
                to_owner: true,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        ])
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            etb(extort()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
                effect: extort(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::SpellNotCastFromHand },
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..creature(
            "Aerial Extortionist",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Bird, CreatureType::Soldier],
            4,
            3,
        )
    }
}

/// Bennie Bracks, Zoologist — convoke; at the beginning of each end step, if
/// you created a token this turn, draw a card.
pub fn bennie_bracks_zoologist() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::AnyPlayer).with_filter(
                Predicate::ValueAtLeast(Value::TokensCreatedThisTurn(PlayerRef::You), Value::ONE),
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
        }],
        ..legend(
            "Bennie Bracks, Zoologist",
            cost(&[generic(3), w()]),
            vec![CreatureType::Elf, CreatureType::Druid],
            3,
            2,
        )
    }
}

/// Alandra, Sky Dreamer — your second draw each turn makes a 2/2 flying
/// Drake; your fifth pumps Alandra and your Drakes +X/+X (X = hand size, locked
/// on resolution). ⚠ The fifth-card trigger is "five or more, once a turn".
pub fn alandra_sky_dreamer() -> CardDefinition {
    let drake =
        TokenDefinition { keywords: vec![Keyword::Flying], ..token("Drake", 2, 2, vec![Color::Blue], vec![CreatureType::Drake]) };
    let hand = || Value::HandSizeOf(PlayerRef::You);
    CardDefinition {
        triggered_abilities: vec![
            second_draw(make(drake)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::You, n: 5 })
                    .once_per_turn(),
                effect: Effect::PumpPT {
                    what: yours(R::Creature.and(R::IsSource.or(R::HasCreatureType(CreatureType::Drake)))),
                    power: hand(),
                    toughness: hand(),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..legend(
            "Alandra, Sky Dreamer",
            cost(&[generic(2), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            4,
        )
    }
}

/// Confirm Suspicions — counter target spell; investigate three times.
pub fn confirm_suspicions() -> CardDefinition {
    CardDefinition {
        name: "Confirm Suspicions",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![counter_target_spell(), investigate(3)]),
        ..Default::default()
    }
}

/// Erdwal Illuminator — flying; the first time you investigate each turn,
/// investigate again. ⚠ Reads a Clue token entering under your control.
pub fn erdwal_illuminator() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::TokenCreated, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: clue() })
                .once_per_turn(),
            effect: investigate(1),
        }],
        ..creature("Erdwal Illuminator", cost(&[generic(1), u()]), vec![CreatureType::Spirit], 1, 3)
    }
}

/// Graf Mole — whenever you sacrifice a Clue, gain 3 life.
pub fn graf_mole() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![clue_sacrificed(Effect::GainLife { who: Selector::You, amount: Value::Const(3) })],
        ..creature("Graf Mole", cost(&[generic(2), g()]), vec![CreatureType::Mole, CreatureType::Beast], 2, 4)
    }
}

/// Ulvenwald Mysteries — a nontoken creature of yours dying investigates;
/// sacrificing a Clue makes a 1/1 Human Soldier.
pub fn ulvenwald_mysteries() -> CardDefinition {
    CardDefinition {
        name: "Ulvenwald Mysteries",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Not(Box::new(R::IsToken)) },
                ),
                effect: investigate(1),
            },
            clue_sacrificed(make(token(
                "Human Soldier",
                1,
                1,
                vec![Color::White],
                vec![CreatureType::Human, CreatureType::Soldier],
            ))),
        ],
        ..Default::default()
    }
}

/// On the Trail — your second draw each turn lets you put a land card from
/// your hand onto the battlefield tapped.
pub fn on_the_trail() -> CardDefinition {
    CardDefinition {
        name: "On the Trail",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![second_draw(land_from_hand(true))],
        ..Default::default()
    }
}

fn land_from_hand(tapped: bool) -> Effect {
    Effect::PutFromHandOntoBattlefield {
        who: PlayerRef::You,
        filter: R::Land,
        count: Value::ONE,
        tapped,
        haste: false,
        sacrifice_eot: false,
        return_eot: false,
        then: None,
    }
}

/// Ongoing Investigation — one or more of your creatures hitting a player
/// investigates; {1}{G}, exile a creature card from your graveyard: investigate
/// and gain 2 life.
pub fn ongoing_investigation() -> CardDefinition {
    CardDefinition {
        name: "Ongoing Investigation",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).once_per_batch(),
            effect: investigate(1),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            exile_other_filter: Some((R::HasCardType(CardType::Creature), 1)),
            effect: Effect::Seq(vec![investigate(1), Effect::GainLife { who: Selector::You, amount: Value::Const(2) }]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Organic Extinction — improvise; destroy all nonartifact creatures.
pub fn organic_extinction() -> CardDefinition {
    CardDefinition {
        name: "Organic Extinction",
        cost: cost(&[generic(8), w(), w()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Improvise],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::Not(Box::new(R::HasCardType(CardType::Artifact))))),
        },
        ..Default::default()
    }
}

/// Chulane, Teller of Tales — vigilance; casting a creature spell draws a
/// card, then you may put a land from hand onto the battlefield; {3}, {T}:
/// return target creature you control to its owner's hand.
pub fn chulane_teller_of_tales() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Creature)),
            effect: Effect::Seq(vec![Effect::Draw { who: Selector::You, amount: Value::ONE }, land_from_hand(false)]),
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Move { what: target_filtered(R::Creature.and(R::ControlledByYou)), to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
            ..Default::default()
        }],
        ..legend(
            "Chulane, Teller of Tales",
            cost(&[generic(2), g(), w(), u()]),
            vec![CreatureType::Human, CreatureType::Druid],
            2,
            4,
        )
    }
}

/// Tezzeret, Betrayer of Flesh — the first artifact ability you activate each
/// turn costs {2} less. +1: draw two, then discard two unless you discard an
/// artifact. −2: target artifact becomes an artifact creature, base 4/4 unless
/// it's a Vehicle. −6: emblem — an artifact of yours becoming tapped draws a
/// card.
pub fn tezzeret_betrayer_of_flesh() -> CardDefinition {
    let artifact = || R::HasCardType(CardType::Artifact);
    CardDefinition {
        name: "Tezzeret, Betrayer of Flesh",
        cost: cost(&[generic(2), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes { planeswalker_subtypes: vec![PlaneswalkerSubtype::Tezzeret], ..Default::default() },
        base_loyalty: 4,
        static_abilities: vec![StaticAbility {
            description: "The first activated ability of an artifact you activate each turn costs {2} less to activate.",
            effect: StaticEffect::FirstArtifactAbilityEachTurnCostsLess { amount: 2 },
        }],
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Seq(vec![
                    Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                    Effect::DiscardUnlessKind { who: PlayerRef::You, count: Value::Const(2), instead: artifact() },
                ]),
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::If {
                    cond: Predicate::EntityMatches {
                        what: Selector::Target(0),
                        filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                    },
                    then: Box::new(Effect::AnimateAsCreature { what: target_filtered(artifact()), duration: Duration::Permanent }),
                    else_: Box::new(Effect::BecomeCreature {
                        what: target_filtered(artifact()),
                        power: Value::Const(4),
                        toughness: Value::Const(4),
                        creature_types: vec![],
                        keywords: vec![],
                        duration: Duration::Permanent,
                    }),
                },
                x_cost: false,
            },
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::CreateEmblem {
                    who: PlayerRef::You,
                    name: "Tezzeret, Betrayer of Flesh".into(),
                    triggered: vec![TriggeredAbility {
                        event: EventSpec::new(EventKind::Tapped, EventScope::YourControl)
                            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: artifact() }),
                        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                    }],
                    statics: vec![],
                },
                x_cost: false,
            },
        ],
        ..Default::default()
    }
}
