//! Commander: the cards the **Obscura Operation** precon (NCC, Kamiz,
//! Obscura Oculus) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_kamiz.rs`.
//!
//! Residuals (each also on its card):
//! - **Commit // Memory** — Commit can't target a spell, only a nonland
//!   permanent.
//! - **Kamiz, Obscura Oculus** — the lesser-power attacker given double
//!   strike is the engine's pick.
//! - **Obscura Confluence** — its third mode returns a creature card from
//!   *your* graveyard (the engine's pick), not a target player's choice.
//! - **Oskar, Rubbish Reclaimer** — the discarded card may be cast from the
//!   graveyard until end of turn rather than right away.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CreatureType, EnchantmentSubtype,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, SplitCard, SplitHalf, StaticAbility, StaticEffect, Subtypes,
    Supertype, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, LookPick, PlayerRef, Predicate, VoteOption, VoteTally, ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{ManaCost, b, cost, generic, u, w, x};
use crabomination_base::tokens::{clue_token, treasure_token};
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

fn legendary(def: CardDefinition) -> CardDefinition {
    CardDefinition { supertypes: vec![Supertype::Legendary], ..def }
}

fn spell(name: &'static str, mana: ManaCost, instant: bool, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![if instant { CardType::Instant } else { CardType::Sorcery }],
        effect,
        ..Default::default()
    }
}

fn equipment(name: &'static str, mana: ManaCost, equip: ManaCost, bonus: EquipBonus) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(equip)],
        equipped_bonus: Some(bonus),
        ..Default::default()
    }
}

/// "Whenever this deals combat damage to a player".
fn on_combat_damage(effect: Effect) -> TriggeredAbility {
    TriggeredAbility { event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource), effect }
}

fn draw_one() -> Effect {
    Effect::Draw { who: Selector::You, amount: Value::ONE }
}

fn unblockable_eot(what: Selector) -> Effect {
    Effect::GrantKeyword { what, keyword: Keyword::Unblockable, duration: Duration::EndOfTurn }
}

/// Archon of Coronation — flying; entering makes you the monarch; while you
/// are, damage doesn't cause you to lose life.
pub fn archon_of_coronation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "As long as you're the monarch, damage doesn't cause you to lose life.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::IsMonarch { who: PlayerRef::You },
                inner: Box::new(StaticEffect::DamageDoesntCauseControllerLifeLoss),
            },
        }],
        triggered_abilities: vec![etb(Effect::BecomeMonarch { who: PlayerRef::You })],
        ..creature("Archon of Coronation", cost(&[generic(4), w(), w()]), vec![CreatureType::Archon], 5, 5)
    }
}

/// Cephalid Facetaker — can't be blocked; at the beginning of combat on your
/// turn it may become a copy of another creature until end of turn, still a
/// 1/4 that can't be blocked.
pub fn cephalid_facetaker() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Unblockable],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::BeginCombat), EventScope::YourControl),
            effect: Effect::MayDo {
                description: "Become a copy of another creature until end of turn?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::BecomeCopyOfFor {
                        what: Selector::This,
                        source: target_filtered(R::Creature.and(R::OtherThanSource)),
                        duration: Duration::EndOfTurn,
                        non_legendary: false,
                    },
                    Effect::SetBasePT {
                        what: Selector::This,
                        power: Value::ONE,
                        toughness: Value::Const(4),
                        duration: Duration::EndOfTurn,
                    },
                    unblockable_eot(Selector::This),
                ])),
            },
        }],
        ..creature(
            "Cephalid Facetaker",
            cost(&[generic(2), u()]),
            vec![CreatureType::Octopus, CreatureType::Rogue],
            1,
            4,
        )
    }
}

/// Change of Plans — X target creatures you control connive; you may phase
/// out any of them.
pub fn change_of_plans() -> CardDefinition {
    spell(
        "Change of Plans",
        cost(&[x(), generic(1), u()]),
        true,
        Effect::TargetsExactlyX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 8,
                min_targets: 0,
                filter: R::Creature.and(R::ControlledByYou),
                effect: Box::new(Effect::Seq(vec![
                    Effect::Connive { what: Selector::Target(0), amount: Value::ONE },
                    Effect::MayDo {
                        description: "Phase it out?".into(),
                        body: Box::new(Effect::PhaseOut { what: Selector::Target(0), until_source_leaves: false }),
                    },
                ])),
            }),
        },
    )
}

/// Commit // Memory — {3}{U} instant // {4}{U}{U} sorcery, aftermath.
/// Commit puts a nonland permanent second from the top of its owner's
/// library; Memory has each player shuffle hand and graveyard in and draw
/// seven.
///
/// ⚠ Residual: Commit can't target a spell.
pub fn commit_memory() -> CardDefinition {
    CardDefinition {
        split: Some(Box::new(SplitCard {
            right: SplitHalf {
                cost: cost(&[generic(4), u(), u()]),
                card_types: vec![CardType::Sorcery],
                effect: Effect::Seq(vec![
                    Effect::ShuffleHandAndGraveyardIntoLibrary { who: PlayerRef::EachPlayer },
                    Effect::Draw { who: Selector::Player(PlayerRef::EachPlayer), amount: Value::Const(7) },
                ]),
            },
            fuse: false,
            aftermath: true,
        })),
        ..spell(
            "Commit // Memory",
            cost(&[generic(3), u()]),
            true,
            Effect::Move {
                what: target_filtered(R::Permanent.and(R::Nonland)),
                to: ZoneDest::Library { who: PlayerRef::OwnerOfMoved, pos: LibraryPosition::FromTop(1) },
            },
        )
    }
}

/// Daring Saboteur — {2}{U}: can't be blocked this turn; connecting, you may
/// draw then discard.
pub fn daring_saboteur() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), u()]),
            effect: unblockable_eot(Selector::This),
            ..Default::default()
        }],
        triggered_abilities: vec![on_combat_damage(Effect::MayDo {
            description: "Draw a card, then discard a card?".into(),
            body: Box::new(Effect::Seq(vec![
                draw_one(),
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ])),
        })],
        ..creature(
            "Daring Saboteur",
            cost(&[generic(1), u()]),
            vec![CreatureType::Human, CreatureType::Pirate],
            2,
            1,
        )
    }
}

/// Dragonlord Ojutai — flying; hexproof while untapped; connecting, look at
/// the top three and take one.
pub fn dragonlord_ojutai() -> CardDefinition {
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Dragonlord Ojutai has hexproof as long as it's untapped.",
            effect: StaticEffect::SelfHasKeywordWhilePredicate {
                keyword: Keyword::Hexproof,
                condition: Predicate::EntityMatches { what: Selector::This, filter: R::Untapped },
            },
        }],
        triggered_abilities: vec![on_combat_damage(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(3),
            ..Default::default()
        })))],
        ..creature(
            "Dragonlord Ojutai",
            cost(&[generic(3), w(), u()]),
            vec![CreatureType::Elder, CreatureType::Dragon],
            5,
            4,
        )
    })
}

/// Graveblade Marauder — deathtouch; connecting, that player loses life
/// equal to the creature cards in your graveyard.
pub fn graveblade_marauder() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![on_combat_damage(Effect::LoseLife {
            who: Selector::Player(PlayerRef::TriggerEventPlayer),
            amount: Value::CardsInGraveyardMatching { who: PlayerRef::You, filter: R::Creature },
        })],
        ..creature(
            "Graveblade Marauder",
            cost(&[generic(2), b()]),
            vec![CreatureType::Human, CreatureType::Warrior],
            1,
            4,
        )
    }
}

/// Identity Thief — attacking, you may exile another nontoken creature; it
/// becomes a copy of it until end of turn, and the card returns at the next
/// end step.
pub fn identity_thief() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayDo {
            description: "Exile another nontoken creature and become a copy of it?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::ExileLinked {
                    what: target_filtered(R::Creature.and(R::NotToken).and(R::OtherThanSource)),
                },
                Effect::BecomeCopyOfFor {
                    what: Selector::This,
                    source: Selector::CardExiledWithSource,
                    duration: Duration::EndOfTurn,
                    non_legendary: false,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Move {
                        what: Selector::CardExiledWithSource,
                        to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
                    }),
                },
            ])),
        })],
        ..creature("Identity Thief", cost(&[generic(2), u(), u()]), vec![CreatureType::Shapeshifter], 0, 3)
    }
}

/// In Too Deep — split second; enchanted creature, planeswalker or Clue is a
/// colorless Clue artifact with only the Clue's ability.
pub fn in_too_deep() -> CardDefinition {
    CardDefinition {
        name: "In Too Deep",
        cost: cost(&[u(), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::SplitSecond],
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(
                R::Creature.or(R::Planeswalker).or(R::HasArtifactSubtype(ArtifactSubtype::Clue)),
            ),
        },
        equipped_bonus: Some(EquipBonus {
            set_card_types: Some(vec![CardType::Artifact]),
            set_artifact_types: Some(vec![ArtifactSubtype::Clue]),
            set_colors: Some(vec![]),
            remove_abilities: true,
            activated_abilities: clue_token().activated_abilities,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Jailbreak — return target permanent card from an opponent's graveyard to
/// the battlefield under their control; then return up to one permanent card
/// with equal or lesser mana value from your graveyard.
pub fn jailbreak() -> CardDefinition {
    spell(
        "Jailbreak",
        cost(&[generic(1), w()]),
        false,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::PermanentCard.and(R::InOpponentGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::OwnerOfMoved, tapped: false },
            },
            Effect::WithX {
                x: Value::ManaValueOf(Box::new(Selector::LastMoved)),
                body: Box::new(Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::PermanentCard.and(R::ManaValueAtMostXFromCost),
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        ]),
    )
}

/// Kamiz, Obscura Oculus — whenever you attack, target attacking creature
/// can't be blocked and connives; another attacker with lesser power gains
/// double strike.
///
/// ⚠ Residual: the lesser-power attacker is the engine's pick.
pub fn kamiz_obscura_oculus() -> CardDefinition {
    legendary(CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::YouAttack, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                unblockable_eot(target_filtered(R::IsAttacking.and(R::ControlledByYou))),
                Effect::Connive { what: Selector::Target(0), amount: Value::ONE },
                Effect::WithX {
                    x: Value::Diff(Box::new(Value::PowerOf(Box::new(Selector::Target(0)))), Box::new(Value::ONE)),
                    body: Box::new(Effect::GrantKeyword {
                        what: Selector::Take {
                            inner: Box::new(Selector::EachPermanent(
                                R::IsAttacking.and(R::ControlledByYou).and(R::PowerAtMostXFromCost),
                            )),
                            count: Box::new(Value::ONE),
                        },
                        keyword: Keyword::DoubleStrike,
                        duration: Duration::EndOfTurn,
                    }),
                },
            ]),
        }],
        ..creature(
            "Kamiz, Obscura Oculus",
            cost(&[generic(1), w(), u(), b()]),
            vec![CreatureType::Octopus, CreatureType::Rogue],
            2,
            4,
        )
    })
}

/// Mask of Riddles — equipped creature has fear and may draw on connecting;
/// equip {2}.
pub fn mask_of_riddles() -> CardDefinition {
    equipment(
        "Mask of Riddles",
        cost(&[u(), b()]),
        cost(&[generic(2)]),
        EquipBonus {
            keywords: vec![Keyword::Fear],
            triggered_abilities: vec![on_combat_damage(Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(draw_one()),
            })],
            ..Default::default()
        },
    )
}

/// Mask of the Schemer — equipped creature connecting connives X, X the
/// damage it dealt; equip {2}.
pub fn mask_of_the_schemer() -> CardDefinition {
    equipment(
        "Mask of the Schemer",
        cost(&[generic(2), u()]),
        cost(&[generic(2)]),
        EquipBonus {
            triggered_abilities: vec![on_combat_damage(Effect::Connive {
                what: Selector::This,
                amount: Value::TriggerEventAmount,
            })],
            ..Default::default()
        },
    )
}

/// Obscura Charm — choose one: return a multicolored permanent card with
/// mana value 3 or less from your graveyard tapped; counter an instant or
/// sorcery spell; destroy a creature or planeswalker with mana value 3 or
/// less.
pub fn obscura_charm() -> CardDefinition {
    spell(
        "Obscura Charm",
        cost(&[w(), u(), b()]),
        true,
        Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(
                    R::PermanentCard.and(R::Multicolored).and(R::ManaValueAtMost(3)).and(R::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))),
                ),
            },
            Effect::Destroy {
                what: target_filtered(R::Creature.or(R::Planeswalker).and(R::ManaValueAtMost(3))),
            },
        ]),
    )
}

/// Obscura Confluence — choose three, repeats allowed: a creature loses its
/// abilities and is 1/1 until end of turn; a creature connives; a creature
/// card returns from a graveyard to hand.
///
/// ⚠ Residual: the third mode returns a creature card from *your*
/// graveyard, the engine's pick.
pub fn obscura_confluence() -> CardDefinition {
    spell(
        "Obscura Confluence",
        cost(&[generic(1), w(), u(), b()]),
        true,
        Effect::ChooseN {
            picks: vec![0, 1, 2],
            modes: vec![
                Effect::Seq(vec![
                    Effect::LoseAllAbilities {
                        what: target_filtered(R::Creature),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::SetBasePT {
                        what: Selector::Target(0),
                        power: Value::ONE,
                        toughness: Value::ONE,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                Effect::Connive { what: target_filtered(R::Creature), amount: Value::ONE },
                Effect::Move {
                    what: Selector::Take {
                        inner: Box::new(Selector::CardsInZone {
                            who: PlayerRef::You,
                            zone: Zone::Graveyard,
                            filter: R::Creature,
                        }),
                        count: Box::new(Value::ONE),
                    },
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ],
        },
    )
}

/// Oskar, Rubbish Reclaimer — costs {1} less per distinct mana value in your
/// graveyard; discarding a nonland card, you may cast it from there.
///
/// ⚠ Residual: the card may be cast until end of turn, not right away.
pub fn oskar_rubbish_reclaimer() -> CardDefinition {
    legendary(CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "This spell costs {1} less to cast for each different mana value among cards in your graveyard.",
            effect: StaticEffect::SelfCostReducedByValue {
                amount: Value::DistinctManaValuesInGraveyard(PlayerRef::You),
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Nonland }),
            effect: Effect::GrantMayPlay {
                what: Selector::TriggerSource,
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
                pay_own_cost: true,
                any_color: false,
            },
        }],
        ..creature(
            "Oskar, Rubbish Reclaimer",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            3,
            3,
        )
    })
}

/// Skyway Robber — flying; escape {3}{U}, exile five other cards; escaped, it
/// connects to cast an artifact, instant or sorcery from among the cards
/// exiled with it for free.
pub fn skyway_robber() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Escape(cost(&[generic(3), u()]), 5)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::SourceCastFromEscape),
            effect: Effect::MayCastExiledWithSource {
                filter: R::HasCardType(CardType::Artifact)
                    .or(R::HasCardType(CardType::Instant))
                    .or(R::HasCardType(CardType::Sorcery)),
            },
        }],
        ..creature("Skyway Robber", cost(&[generic(3), u()]), vec![CreatureType::Bird, CreatureType::Rogue], 3, 3)
    }
}

/// Tivit, Seller of Secrets — flying, ward {3}; entering or connecting,
/// council's dilemma: evidence investigates, bribery makes a Treasure; you
/// vote an additional time.
pub fn tivit_seller_of_secrets() -> CardDefinition {
    let vote = || Effect::Vote {
        tally: VoteTally::PerVote,
        options: vec![
            VoteOption::new(
                "evidence",
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(clue_token()) },
            ),
            VoteOption::new(
                "bribery",
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: Arc::new(treasure_token()) },
            ),
        ],
    };
    legendary(CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Ward(WardCost::generic(3))],
        static_abilities: vec![StaticAbility {
            description: "While voting, you may vote an additional time.",
            effect: StaticEffect::AdditionalVotes(1),
        }],
        triggered_abilities: vec![etb(vote()), on_combat_damage(vote())],
        ..creature(
            "Tivit, Seller of Secrets",
            cost(&[generic(3), w(), u(), b()]),
            vec![CreatureType::Sphinx, CreatureType::Rogue],
            6,
            6,
        )
    })
}

/// Writ of Return — return target creature card from your graveyard to the
/// battlefield tapped; cipher.
pub fn writ_of_return() -> CardDefinition {
    spell(
        "Writ of Return",
        cost(&[generic(3), b(), b()]),
        false,
        Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InYourGraveyard)),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
            Effect::Cipher,
        ]),
    )
}
