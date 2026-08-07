//! Mirage (MIR), fourth wave — the rares that lean on colour, control and
//! combat riders. Tests in `classic_sets/mir`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, EventKind, EventScope,
    EventSpec, Keyword, LandType, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{on_attack, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Predicate, Selector, Value, ZoneDest};
use crate::game::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w, x};

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
    CardDefinition { card_types: vec![CardType::Sorcery], ..instant(name, c, effect) }
}

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

/// "When a spell or ability an opponent controls causes you to discard this
/// card, [body]" (Mangara's Blessing, Sand Golem).
fn on_forced_discard(body: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::OpponentCausedYouToDiscard, EventScope::SelfSource),
        effect: body,
    }
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Raging Spirit — {3}{R} 3/3 that can shed its colour.
pub fn raging_spirit() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeColor {
                what: Selector::This,
                colors: vec![],
                duration: Duration::EndOfTurn,
                additive: false,
            },
            ..Default::default()
        }],
        ..creature("Raging Spirit", cost(&[generic(3), r()]), vec![CreatureType::Spirit], 3, 3)
    }
}

/// Leering Gargoyle — {1}{W}{U} 2/2 flier that can hunker down instead.
pub fn leering_gargoyle() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(-2),
                    toughness: Value::Const(2),
                    duration: Duration::EndOfTurn,
                },
                Effect::LoseKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Leering Gargoyle",
            cost(&[generic(1), w(), u()]),
            vec![CreatureType::Gargoyle],
            2,
            2,
        )
    }
}

/// Catacomb Dragon — {4}{B}{B} 4/4 flier that halves what dares block it.
pub fn catacomb_dragon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecomesBlocked, EventScope::SelfSource).with_filter(
                Predicate::EntityMatchesAny {
                    what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                    filter: R::Not(Box::new(R::Artifact))
                        .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Dragon)))),
                },
            ),
            effect: Effect::PumpPT {
                what: Selector::CreaturesInCombatWith(Box::new(Selector::This)),
                power: Value::Negate(Box::new(Value::HalvedRoundDown(Box::new(Value::PowerOf(
                    Box::new(Selector::CreaturesInCombatWith(Box::new(Selector::This))),
                ))))),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Catacomb Dragon",
            cost(&[generic(4), b(), b()]),
            vec![CreatureType::Dragon],
            4,
            4,
        )
    }
}

/// Discordant Spirit — {2}{B}{R} 2/2 that swells with the damage you take.
pub fn discordant_spirit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::DamageTakenThisTurn(PlayerRef::You),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
                effect: Effect::RemoveAllCounters { what: Selector::This },
            },
        ],
        ..creature(
            "Discordant Spirit",
            cost(&[generic(2), b(), r()]),
            vec![CreatureType::Spirit],
            2,
            2,
        )
    }
}

/// Mtenda Griffin — {3}{W} 2/2 flier that recycles itself and a friend.
pub fn mtenda_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[w()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                },
                Effect::Move {
                    what: target_filtered(
                        R::HasCreatureType(CreatureType::Griffin).and(R::InYourGraveyard),
                    ),
                    to: ZoneDest::Hand(PlayerRef::You),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Mtenda Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Unyaro Griffin — {3}{W} 2/2 flier that eats a red spell.
pub fn unyaro_griffin() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(
                    R::IsSpellOnStack.and(R::HasColor(Color::Red)).and(
                        R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
                    ),
                ),
            },
            ..Default::default()
        }],
        ..creature("Unyaro Griffin", cost(&[generic(3), w()]), vec![CreatureType::Griffin], 2, 2)
    }
}

/// Vigilant Martyr — {W} 1/1 that shields a creature or an enchantment.
pub fn vigilant_martyr() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::Regenerate { what: target_filtered(R::Creature) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[w(), w()]),
                tap_cost: true,
                sac_cost: true,
                effect: Effect::CounterSpell {
                    what: target_filtered(
                        R::IsSpellOnStack
                            .and(R::SpellTargetsMatching(Box::new(R::Enchantment))),
                    ),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Vigilant Martyr",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Cleric],
            1,
            1,
        )
    }
}

/// Sawback Manticore — {3}{R}{G} 2/4 that can fly or snipe mid-combat.
pub fn sawback_manticore() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                effect: Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Flying,
                    duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                once_per_turn: true,
                condition: Some(Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::IsAttacking.or(R::IsBlocking),
                }),
                effect: Effect::DealDamage {
                    to: target_filtered(R::Creature.and(R::IsAttacking.or(R::IsBlocking))),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..creature(
            "Sawback Manticore",
            cost(&[generic(3), r(), g()]),
            vec![CreatureType::Manticore],
            2,
            4,
        )
    }
}

/// Kukemssa Serpent — {3}{U} 4/3 that needs water on both sides.
pub fn kukemssa_serpent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(R::HasLandType(
            LandType::Island,
        )))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_other_filter: Some((R::HasLandType(LandType::Island), 1)),
            effect: Effect::GainLandType {
                what: target_filtered(R::Land.and(R::ControlledByOpponent)),
                land_type: LandType::Island,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl)
                .with_filter(Predicate::ValueAtMost(
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        R::HasLandType(LandType::Island).and(R::ControlledByYou),
                    ))),
                    Value::ZERO,
                )),
            effect: Effect::SacrificeSource,
        }],
        ..creature("Kukemssa Serpent", cost(&[generic(3), u()]), vec![CreatureType::Serpent], 4, 3)
    }
}

/// Sand Golem — {5} 3/3 that comes back bigger if they made you pitch it.
pub fn sand_golem() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![on_forced_discard(Effect::AtNextEndStep {
            body: Box::new(Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            }),
        })],
        ..creature("Sand Golem", cost(&[generic(5)]), vec![CreatureType::Golem], 3, 3)
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Mangara's Blessing — {2}{W}: five life, and a refund if they mill it out
/// of your hand.
pub fn mangaras_blessing() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_forced_discard(Effect::Seq(vec![
            crate::effect::shortcut::gain_life(2),
            Effect::AtNextEndStep {
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        ]))],
        ..instant(
            "Mangara's Blessing",
            cost(&[generic(2), w()]),
            crate::effect::shortcut::gain_life(5),
        )
    }
}

/// Torrent of Lava — {X}{R}{R}: X to everything on the ground.
pub fn torrent_of_lava() -> CardDefinition {
    sorcery(
        "Torrent of Lava",
        cost(&[x(), r(), r()]),
        Effect::DealDamage {
            to: Selector::EachPermanent(
                R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
            ),
            amount: Value::XFromCost,
        },
    )
}

/// Kaervek's Purge — {X}{B}{R}: kill a creature at exactly X, and burn its
/// controller for its power.
pub fn kaerveks_purge() -> CardDefinition {
    sorcery(
        "Kaervek's Purge",
        cost(&[x(), b(), r()]),
        Effect::Seq(vec![
            Effect::Destroy {
                what: target_filtered(R::Creature.and(R::ManaValueExactlyXFromCost)),
            },
            Effect::If {
                cond: Predicate::EntityMatchesAny {
                    what: Selector::Target(0),
                    filter: R::InGraveyard,
                },
                then: Box::new(Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    amount: Value::PowerOf(Box::new(Selector::Target(0))),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
    )
}

/// Energy Bolt — {X}{R}{W}: X damage, or X life.
pub fn energy_bolt() -> CardDefinition {
    sorcery(
        "Energy Bolt",
        cost(&[x(), r(), w()]),
        Effect::ChooseMode(vec![
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
            },
            Effect::GainLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::XFromCost,
            },
        ]),
    )
}

/// Prismatic Lace — {U}: repaint a permanent for good.
pub fn prismatic_lace() -> CardDefinition {
    instant(
        "Prismatic Lace",
        cost(&[u()]),
        Effect::BecomeChosenColor {
            what: target_filtered(R::Permanent),
            duration: Duration::Permanent,
        },
    )
}

/// Prismatic Boon — {X}{W}{U}: X creatures duck one colour.
pub fn prismatic_boon() -> CardDefinition {
    instant(
        "Prismatic Boon",
        cost(&[x(), w(), u()]),
        Effect::CapTargetsAtX {
            body: Box::new(Effect::ApplyToTargets {
                max_targets: 6,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(Effect::GrantProtectionFromChosenColor {
                    what: Selector::Target(0),
                    duration: Duration::EndOfTurn,
                }),
            }),
        },
    )
}

/// Telim'Tor's Edict — {R}: exile one of your own permanents, then replace
/// the card.
pub fn telimtors_edict() -> CardDefinition {
    instant(
        "Telim'Tor's Edict",
        cost(&[r()]),
        Effect::Seq(vec![
            Effect::Exile { what: target_filtered(R::Permanent.and(R::ControlledByYou)) },
            Effect::AtNextTurnsUpkeep { body: Box::new(crate::effect::shortcut::draw(1)) },
        ]),
    )
}

/// Political Trickery — {2}{U}: swap a land with an opponent, permanently.
pub fn political_trickery() -> CardDefinition {
    sorcery(
        "Political Trickery",
        cost(&[generic(2), u()]),
        Effect::ExchangeControl {
            a: target_filtered(R::Land.and(R::ControlledByYou)),
            b: Selector::TargetFiltered {
                slot: 1,
                filter: R::Land.and(R::ControlledByOpponent),
            },
        },
    )
}

/// Jabari's Influence — {3}{W}{W}: take a creature that came at you, and dull
/// its edge.
pub fn jabaris_influence() -> CardDefinition {
    instant(
        "Jabari's Influence",
        cost(&[generic(3), w(), w()]),
        Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(
                    R::Creature
                        .and(R::Not(Box::new(R::Artifact)))
                        .and(R::Not(Box::new(R::HasColor(Color::Black))))
                        .and(R::AttackedThisTurn),
                ),
                to: None,
                duration: Duration::Permanent,
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::MinusOneMinusZero,
                amount: Value::ONE,
            },
        ]),
    )
}

/// Yare — {2}{W}: a defender that swells and swallows the whole attack.
pub fn yare() -> CardDefinition {
    instant(
        "Yare",
        cost(&[generic(2), w()]),
        Effect::Seq(vec![
            Effect::PumpPT {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                power: Value::Const(3),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::CanBlockAdditional(2),
                duration: Duration::EndOfTurn,
            },
        ]),
    )
}

/// Flash — {1}{U}: a creature now, for its cost less {2}.
pub fn flash() -> CardDefinition {
    instant(
        "Flash",
        cost(&[generic(1), u()]),
        Effect::PutFromHandOntoBattlefield {
            who: PlayerRef::You,
            filter: R::Creature,
            count: Value::ONE,
            tapped: false,
            haste: false,
            sacrifice_eot: false,
            return_eot: false,
            then: None,
        },
    )
}

/// Lure of Prey — {2}{G}{G}: a free green body, once they've committed one.
pub fn lure_of_prey() -> CardDefinition {
    CardDefinition {
        cast_condition: Some(Predicate::SpellsCastThisTurnAtLeast {
            who: PlayerRef::EachOpponent,
            at_least: Value::ONE,
        }),
        ..instant(
            "Lure of Prey",
            cost(&[generic(2), g(), g()]),
            Effect::PutFromHandOntoBattlefield {
                who: PlayerRef::You,
                filter: R::Creature.and(R::HasColor(Color::Green)),
                count: Value::ONE,
                tapped: false,
                haste: false,
                sacrifice_eot: false,
                return_eot: false,
                then: None,
            },
        )
    }
}

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Unerring Sling — {3}: tap a creature to shoot down a flier.
pub fn unerring_sling() -> CardDefinition {
    artifact(
        "Unerring Sling",
        cost(&[generic(3)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            tap_other_filter: Some(R::Creature.and(R::ControlledByYou)),
            effect: Effect::DealDamage {
                to: target_filtered(
                    R::Creature
                        .and(R::IsAttacking.or(R::IsBlocking))
                        .and(R::HasKeyword(Keyword::Flying)),
                ),
                amount: Value::TappedForCostPower,
            },
            ..Default::default()
        }],
    )
}

/// Zirilan of the Claw — {3}{R}{R} 3/4 that borrows a Dragon for one swing.
pub fn zirilan_of_the_claw() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), r(), r()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::HasCreatureType(CreatureType::Dragon).and(R::PermanentCard),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::RememberPermanentOnSource { what: Selector::LastMoved },
                Effect::GrantKeyword {
                    what: Selector::ChosenPermanentOfSource,
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Exile { what: Selector::ChosenPermanentOfSource }),
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Zirilan of the Claw",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Lizard, CreatureType::Shaman],
            3,
            4,
        )
    }
}

/// Mtenda Lion — {G} 2/1 the defender can buy off for {U}.
pub fn mtenda_lion() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_attack(Effect::MayPayBy {
            who: PlayerRef::DefendingPlayer,
            description: "Pay {U} to blank this attacker?".into(),
            mana_cost: cost(&[u()]),
            body: Box::new(Effect::PreventCombatDamageByTargetThisTurn {
                target: Selector::This,
            }),
            else_: None,
        })],
        ..creature("Mtenda Lion", cost(&[g()]), vec![CreatureType::Cat], 2, 1)
    }
}
