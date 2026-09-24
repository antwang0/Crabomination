//! Commander: the cards the **Fae Dominion** precon (WOC, Tegwyll, Duke of
//! Splendor) needed beyond what the catalog had. Tests in
//! `tests/recent_b/cmdr_fae.rs`.
//!
//! Residuals (each also on its card):
//! - **Blightwing Bandit** — the stolen card is exiled face up.
//! - **Halo Forager** — a mana-value-0 card can't be cast (paying {0} is a
//!   decline).
//! - **Illusionist's Gambit** — the must-attack and can't-attack-you grants
//!   last the turn, not just the extra combat.
//! - **Puppeteer Clique** — the creature is exiled at the next end step, not
//!   necessarily yours.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    EntersAsCopy, EventKind, EventScope, EventSpec, Keyword, MayPlayDuration,
    SelectionRequirement as R, Selector, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost, Zone,
};
use crate::effect::shortcut::{draw, etb, target_filtered};
use crate::effect::{Duration, Effect, LookPick, PlayerRef, Predicate, ZoneDest, ZoneRef};
use crate::game::types::TurnStep;
use crate::mana::{b, cost, generic, hybrid, u, Color, ManaCost};
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

fn faerie(name: &str, color: Color, extra: Option<CreatureType>) -> Arc<TokenDefinition> {
    Arc::new(TokenDefinition {
        name: name.into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes {
            creature_types: std::iter::once(CreatureType::Faerie).chain(extra).collect(),
            ..Default::default()
        },
        ..Default::default()
    })
}

fn faerie_rogues(n: i32) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(n),
        definition: faerie("Faerie Rogue", Color::Black, Some(CreatureType::Rogue)),
    }
}

/// "Whenever you cast your first spell during each opponent's turn."
fn first_spell_each_opponents_turn(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(Predicate::All(vec![
            Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
            Predicate::SpellsCastThisTurnEquals { who: PlayerRef::You, count: Value::ONE },
        ])),
        effect,
    }
}

/// "Whenever one or more Faeries you control deal combat damage to a player"
/// — one fire per damaged player (CR 603.2c); that player is
/// `PlayerRef::TriggerEventPlayer`.
fn faeries_connect(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec {
            once_per_batch: true,
            ..EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Faerie),
                },
            )
        },
        effect,
    }
}

fn ward2() -> Keyword {
    Keyword::Ward(WardCost::generic(2))
}

/// Tegwyll, Duke of Splendor — flying, deathtouch; other Faeries you control
/// get +1/+1; another Faerie of yours dying draws a card and costs 1 life.
pub fn tegwyll_duke_of_splendor() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        static_abilities: vec![StaticAbility {
            description: "Other Faeries you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    R::Creature
                        .and(R::HasCreatureType(CreatureType::Faerie))
                        .and(R::ControlledByYou)
                        .and(R::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Faerie).and(R::OtherThanSource),
                },
            ),
            effect: Effect::Seq(vec![draw(1), Effect::LoseLife { who: Selector::You, amount: Value::ONE }]),
        }],
        ..creature(
            "Tegwyll, Duke of Splendor",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Faerie, CreatureType::Noble],
            2,
            3,
        )
    }
}

/// Alela, Cunning Conqueror — flying; your first spell on each opponent's
/// turn makes a Faerie Rogue; Faeries connecting goad a creature that player
/// controls.
pub fn alela_cunning_conqueror() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            first_spell_each_opponents_turn(faerie_rogues(1)),
            faeries_connect(Effect::Goad { what: target_filtered(R::Creature.and(R::ControlledByTriggerPlayer)) }),
        ],
        ..creature(
            "Alela, Cunning Conqueror",
            cost(&[generic(2), u(), b()]),
            vec![CreatureType::Faerie, CreatureType::Warlock],
            2,
            4,
        )
    }
}

/// Archmage of Echoes — flying, ward {2}; casting a Faerie or Wizard
/// permanent spell copies it (CR 707.10f — the copy becomes a token).
pub fn archmage_of_echoes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, ward2()],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::CastSpellMatches(
                    R::PermanentCard.and(
                        R::HasCreatureType(CreatureType::Faerie).or(R::HasCreatureType(CreatureType::Wizard)),
                    ),
                ),
            ),
            effect: Effect::CopySpell { what: Selector::TriggerSource, count: Value::ONE },
        }],
        ..creature(
            "Archmage of Echoes",
            cost(&[generic(4), u()]),
            vec![CreatureType::Faerie, CreatureType::Wizard],
            4,
            4,
        )
    }
}

/// Blightwing Bandit — flying, deathtouch; your first spell on each
/// opponent's turn exiles their top card, which you may play while it stays
/// exiled, with mana of any type (exiled face up).
pub fn blightwing_bandit() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        triggered_abilities: vec![first_spell_each_opponents_turn(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::ActivePlayer,
            count: Value::ONE,
            duration: MayPlayDuration::WhileExiled,
            pay_any_color: true,
            max_mana_value: None,
            pay_own_cost: false,
            uncast_penalty: None,
        })],
        ..creature("Blightwing Bandit", cost(&[generic(3), b()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 2)
    }
}

/// Faerie Bladecrafter — flying; Faeries connecting grow it; dying drains
/// each opponent for its power.
pub fn faerie_bladecrafter() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            faeries_connect(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Drain {
                    from: Selector::Player(PlayerRef::EachOpponent),
                    to: Selector::You,
                    amount: Value::PowerOf(Box::new(Selector::This)),
                },
            },
        ],
        ..creature("Faerie Bladecrafter", cost(&[generic(2), b()]), vec![CreatureType::Faerie, CreatureType::Rogue], 2, 2)
    }
}

/// Faerie Formation — flying; {3}{U}: a 1/1 Faerie and a card.
pub fn faerie_formation() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(3), u()]),
            effect: Effect::Seq(vec![
                Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: faerie("Faerie", Color::Blue, None) },
                draw(1),
            ]),
            ..Default::default()
        }],
        ..creature("Faerie Formation", cost(&[generic(4), u()]), vec![CreatureType::Faerie], 5, 4)
    }
}

/// Glen Elendra Liege — flying; your other blue and your other black
/// creatures each get +1/+1 (a blue-black creature gets both).
pub fn glen_elendra_liege() -> CardDefinition {
    let lord = |color, description| StaticAbility {
        description,
        effect: StaticEffect::PumpPT {
            applies_to: Selector::EachPermanent(
                R::Creature.and(R::HasColor(color)).and(R::ControlledByYou).and(R::OtherThanSource),
            ),
            power: 1,
            toughness: 1,
        },
    };
    let ub = || hybrid(Color::Blue, Color::Black);
    CardDefinition {
        keywords: vec![Keyword::Flying],
        static_abilities: vec![
            lord(Color::Blue, "Other blue creatures you control get +1/+1."),
            lord(Color::Black, "Other black creatures you control get +1/+1."),
        ],
        ..creature(
            "Glen Elendra Liege",
            cost(&[generic(1), ub(), ub(), ub()]),
            vec![CreatureType::Faerie, CreatureType::Knight],
            2,
            3,
        )
    }
}

/// Halo Forager — flying; entering, pay {X} to cast an instant or sorcery
/// card with mana value X from a graveyard for free, exiling it after.
pub fn halo_forager() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        // CR 603.7 — the target is chosen when the reflexive "when you do"
        // resolves, once X is known.
        triggered_abilities: vec![etb(Effect::MayPayX {
            description: "Pay {X} to cast an instant or sorcery with mana value X from a graveyard?".into(),
            body: Box::new(Effect::Reflexive {
                body: Box::new(Effect::CastWithoutPayingImmediate {
                    what: target_filtered(
                        R::HasCardType(CardType::Instant)
                            .or(R::HasCardType(CardType::Sorcery))
                            .and(R::ManaValueExactlyXFromCost)
                            .from_any_graveyard(),
                    ),
                    source_zone: Zone::Graveyard,
                    exile_after: true,
                    copy: false,
                    reduce_generic: 0,
                    pay_own_cost: false,
                }),
            }),
        })],
        ..creature(
            "Halo Forager",
            cost(&[generic(1), u(), b()]),
            vec![CreatureType::Faerie, CreatureType::Rogue],
            3,
            1,
        )
    }
}

/// Illusionist's Gambit — on an opponent's turn, in the declare blockers
/// step: the attackers leave combat and untap, and attack again in an extra
/// combat, anyone but you (the grants last the turn).
pub fn illusionists_gambit() -> CardDefinition {
    let attackers = || Selector::EachPermanent(R::Creature.and(R::IsAttacking));
    CardDefinition {
        name: "Illusionist's Gambit",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Instant],
        cast_condition: Some(Predicate::All(vec![
            Predicate::CurrentStepIs(TurnStep::DeclareBlockers),
            Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You))),
        ])),
        effect: Effect::Seq(vec![
            Effect::GrantKeyword { what: attackers(), keyword: Keyword::MustAttack, duration: Duration::EndOfTurn },
            Effect::GrantCantAttackYou { what: attackers(), duration: Duration::EndOfTurn },
            Effect::Untap { what: attackers(), up_to: None },
            Effect::RemoveFromCombat { what: attackers() },
            Effect::AdditionalCombatPhase { count: Value::ONE },
        ]),
        ..Default::default()
    }
}

/// Malleable Impostor — flash, flying; may enter as a copy of an opponent's
/// creature that is also a Faerie Shapeshifter with flying.
pub fn malleable_impostor() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Flying],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.and(R::ControlledByOpponent),
            extra_creature_types: vec![CreatureType::Faerie, CreatureType::Shapeshifter],
            extra_keywords: vec![Keyword::Flying],
            ..Default::default()
        }),
        ..creature(
            "Malleable Impostor",
            cost(&[generic(3), u()]),
            vec![CreatureType::Faerie, CreatureType::Shapeshifter],
            0,
            0,
        )
    }
}

/// Nettling Nuisance — flying; Faeries connecting give that player a 4/2
/// Pirate that can't block, goaded for the rest of the game.
pub fn nettling_nuisance() -> CardDefinition {
    let pirate = TokenDefinition {
        name: "Pirate".into(),
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::CantBlock],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Red],
        subtypes: Subtypes { creature_types: vec![CreatureType::Pirate], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![faeries_connect(Effect::Seq(vec![
            Effect::CreateToken { who: PlayerRef::TriggerEventPlayer, count: Value::ONE, definition: Arc::new(pirate) },
            Effect::GoadForTheGame { what: Selector::LastCreatedToken },
        ]))],
        ..creature("Nettling Nuisance", cost(&[generic(2), b()]), vec![CreatureType::Faerie, CreatureType::Rogue], 3, 1)
    }
}

/// Nymris, Oona's Trickster — flash, flying; your first spell on each
/// opponent's turn looks at two: one to hand, one to the graveyard.
pub fn nymris_oonas_trickster() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![first_spell_each_opponents_turn(Effect::LookPickToHand(Box::new(LookPick {
            who: PlayerRef::You,
            count: Value::Const(2),
            rest_to_graveyard: true,
            ..Default::default()
        })))],
        ..creature(
            "Nymris, Oona's Trickster",
            cost(&[generic(3), u(), b()]),
            vec![CreatureType::Faerie, CreatureType::Knight],
            1,
            6,
        )
    }
}

/// Puppeteer Clique — flying, persist; entering, an opponent's creature card
/// joins you with haste until the next end step.
pub fn puppeteer_clique() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying, Keyword::Persist],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.from_any_graveyard().and(R::Not(Box::new(R::OwnedByYou)))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::GrantKeyword { what: Selector::LastMoved, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
            Effect::ExileAtNextEndStep { what: Selector::LastMoved },
        ]))],
        ..creature(
            "Puppeteer Clique",
            cost(&[generic(3), b(), b()]),
            vec![CreatureType::Faerie, CreatureType::Wizard],
            3,
            2,
        )
    }
}

/// Rankle, Master of Pranks — flying, haste; connecting, choose any number:
/// everyone discards, everyone loses 1 and draws, everyone sacrifices a
/// creature.
pub fn rankle_master_of_pranks() -> CardDefinition {
    let each = |body| Effect::EachPlayerDoes { who: PlayerRef::EachPlayer, body: Box::new(body) };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::ChooseUpToN {
                max: Box::new(Value::Const(3)),
                modes: vec![
                    each(Effect::Discard { who: Selector::You, amount: Value::ONE, random: false }),
                    each(Effect::Seq(vec![Effect::LoseLife { who: Selector::You, amount: Value::ONE }, draw(1)])),
                    each(Effect::Sacrifice { who: Selector::You, count: Value::ONE, filter: R::Creature }),
                ],
            },
        }],
        ..creature(
            "Rankle, Master of Pranks",
            cost(&[generic(2), b(), b()]),
            vec![CreatureType::Faerie, CreatureType::Rogue],
            3,
            3,
        )
    }
}

/// Shadow Puppeteers — flying, ward {2}; two Faerie Rogues on entry; a flier
/// of yours attacking may become a 4/4 red Dragon until end of turn.
pub fn shadow_puppeteers() -> CardDefinition {
    let it = || Selector::TriggerSource;
    CardDefinition {
        keywords: vec![Keyword::Flying, ward2()],
        triggered_abilities: vec![
            etb(faerie_rogues(2)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches { what: it(), filter: R::HasKeyword(Keyword::Flying) },
                ),
                effect: Effect::MayDo {
                    description: "Have it become a 4/4 red Dragon until end of turn?".into(),
                    body: Box::new(Effect::Seq(vec![
                        Effect::BecomeColor {
                            what: it(),
                            colors: vec![Color::Red],
                            duration: Duration::EndOfTurn,
                            additive: true,
                        },
                        Effect::AddCreatureTypes {
                            what: it(),
                            creature_types: vec![CreatureType::Dragon],
                            duration: Duration::EndOfTurn,
                        },
                        Effect::SetBasePT {
                            what: it(),
                            power: Value::Const(4),
                            toughness: Value::Const(4),
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            },
        ],
        ..creature("Shadow Puppeteers", cost(&[generic(6), u()]), vec![CreatureType::Faerie, CreatureType::Wizard], 4, 4)
    }
}

/// Tegwyll's Scouring — destroy all creatures, then three Faerie Rogues;
/// castable as though it had flash by tapping three of your fliers.
pub fn tegwylls_scouring() -> CardDefinition {
    CardDefinition {
        name: "Tegwyll's Scouring",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Sorcery],
        flash_additional_cost: Some(AdditionalCastCost::TapPermanents {
            filter: R::Creature.and(R::HasKeyword(Keyword::Flying)).and(R::Untapped),
            count: 3,
        }),
        effect: Effect::Seq(vec![Effect::Destroy { what: Selector::EachPermanent(R::Creature) }, faerie_rogues(3)]),
        ..Default::default()
    }
}

/// Thrilling Encore — every creature card put into any graveyard from the
/// battlefield this turn returns under your control.
pub fn thrilling_encore() -> CardDefinition {
    CardDefinition {
        name: "Thrilling Encore",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Move {
            what: Selector::EachMatching {
                zone: ZoneRef::Graveyard(PlayerRef::EachPlayer),
                filter: R::Creature.and(R::PutIntoGraveyardFromBattlefieldThisTurn),
            },
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
        },
        ..Default::default()
    }
}
