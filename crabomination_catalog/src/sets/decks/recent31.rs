//! Modern-supplement batch: the wedge/guild modal "charms" and "commands"
//! (`Effect::ChooseMode` / `Effect::ChooseN`), the graveyard-CDA *goyf* family
//! (`DynamicPt::CreatureCardsInAllGraveyards` and
//! `BasePlusCreaturesInControllerGraveyard`),
//! and assorted multicolor staples. Tracked in `DECK_FEATURES.md` under the
//! Modern supplement. Tests in `tests/recent31.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, DynamicPt, Effect,
    EventKind, EventScope, EventSpec, Keyword, SelectionRequirement, Selector, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{cascade, etb, target_filtered};
use crate::effect::{Duration, PlayerRef, Predicate, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

fn any_creature_target() -> Selector {
    target_filtered(SelectionRequirement::Creature)
}

// ── Single-mode charms (Effect::ChooseMode) ──────────────────────────────────

/// Gruul Charm — {R}{G} Instant. Choose one — creatures without flying can't
/// block this turn; or gain control of all permanents you own; or 3 damage to
/// each creature with flying.
pub fn gruul_charm() -> CardDefinition {
    CardDefinition {
        name: "Gruul Charm",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasKeyword(Keyword::Flying)))),
                ),
                body: Box::new(Effect::GrantKeyword {
                    what: Selector::TriggerSource, keyword: Keyword::CantBlock, duration: Duration::EndOfTurn,
                }),
            },
            Effect::GainControl {
                what: Selector::EachPermanent(SelectionRequirement::OwnedByYou),
                to: Some(PlayerRef::You),
                duration: Duration::Permanent,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::HasKeyword(Keyword::Flying)),
                ),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(3) }),
            },
        ]),
        ..Default::default()
    }
}

/// Dimir Charm — {U}{B} Instant. Choose one — counter target sorcery; or
/// destroy target creature with power 2 or less; or mill the top of target
/// player's library (the look-and-keep-one is approximated as mill 2).
pub fn dimir_charm() -> CardDefinition {
    CardDefinition {
        name: "Dimir Charm",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack.and(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
            },
            Effect::Destroy {
                what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2))),
            },
            Effect::Mill { who: target_filtered(SelectionRequirement::Player), amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Orzhov Charm — {W}{B} Instant. Choose one — return target creature you
/// control to hand; or destroy target creature and lose life equal to its
/// toughness; or return target creature card with mana value 1 or less from
/// your graveyard to the battlefield.
pub fn orzhov_charm() -> CardDefinition {
    CardDefinition {
        name: "Orzhov Charm",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Seq(vec![
                Effect::DestroyAndRemember { what: any_creature_target() },
                Effect::LoseLife { who: Selector::You, amount: Value::SacrificedToughness },
            ]),
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::InYourGraveyard
                        .and(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ManaValueAtMost(1)),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
        ]),
        ..Default::default()
    }
}

/// Naya Charm — {R}{G}{W} Instant. Choose one — 3 damage to target creature; or
/// return target card from a graveyard to its owner's hand; or tap all
/// creatures your opponents control (the any-player target is approximated).
pub fn naya_charm() -> CardDefinition {
    CardDefinition {
        name: "Naya Charm",
        cost: cost(&[r(), g(), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::DealDamage { to: any_creature_target(), amount: Value::Const(3) },
            Effect::Move {
                what: target_filtered(SelectionRequirement::InGraveyard),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::Tap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        ]),
        ..Default::default()
    }
}

/// Jund Charm — {B}{R}{G} Instant. Choose one — exile target player's
/// graveyard; or 2 damage to each creature; or put two +1/+1 counters on
/// target creature.
pub fn jund_charm() -> CardDefinition {
    CardDefinition {
        name: "Jund Charm",
        cost: cost(&[b(), r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::ExilePlayerGraveyard { who: PlayerRef::Target(0) },
            Effect::ForEach {
                selector: Selector::EachPermanent(SelectionRequirement::Creature),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(2) }),
            },
            Effect::AddCounter {
                what: any_creature_target(), kind: CounterType::PlusOnePlusOne, amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Grixis Charm — {U}{B}{R} Instant. Choose one — return target permanent to
/// hand; or target creature gets -4/-4; or creatures you control get +2/+0.
pub fn grixis_charm() -> CardDefinition {
    CardDefinition {
        name: "Grixis Charm",
        cost: cost(&[u(), b(), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseMode(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Permanent),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            Effect::PumpPT {
                what: any_creature_target(),
                power: Value::Const(-4), toughness: Value::Const(-4), duration: Duration::EndOfTurn,
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::Const(2), toughness: Value::Const(0), duration: Duration::EndOfTurn,
                }),
            },
        ]),
        ..Default::default()
    }
}

// ── Commands (Effect::ChooseN — choose two) ──────────────────────────────────

/// Silumgar's Command — {3}{U}{B} Instant. Choose two — counter target
/// noncreature spell; return target permanent to hand; target creature gets
/// -3/-3; destroy target planeswalker.
pub fn silumgars_command() -> CardDefinition {
    CardDefinition {
        name: "Silumgar's Command",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![2, 3],
            modes: vec![
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::Not(Box::new(SelectionRequirement::HasCardType(CardType::Creature))),
                    )),
                },
                Effect::Move {
                    what: target_filtered(SelectionRequirement::Permanent),
                    to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
                },
                Effect::PumpPT {
                    what: any_creature_target(),
                    power: Value::Const(-3), toughness: Value::Const(-3), duration: Duration::EndOfTurn,
                },
                Effect::Destroy { what: target_filtered(SelectionRequirement::Planeswalker) },
            ],
        },
        ..Default::default()
    }
}

/// Ojutai's Command — {2}{W}{U} Instant. Choose two — return target creature
/// card with mana value 2 or less from your graveyard to the battlefield; gain
/// 4 life; counter target creature spell; draw a card.
pub fn ojutais_command() -> CardDefinition {
    CardDefinition {
        name: "Ojutai's Command",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![1, 3],
            modes: vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::InYourGraveyard
                            .and(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ManaValueAtMost(2)),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
                Effect::CounterSpell {
                    what: target_filtered(SelectionRequirement::IsSpellOnStack.and(
                        SelectionRequirement::HasCardType(CardType::Creature),
                    )),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ],
        },
        ..Default::default()
    }
}

/// Atarka's Command — {R}{G} Instant. Choose two — your opponents can't gain
/// life this turn; 3 damage to each opponent; you may put a land from your
/// hand onto the battlefield; creatures you control get +1/+1 and gain reach.
pub fn atarkas_command() -> CardDefinition {
    CardDefinition {
        name: "Atarka's Command",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseN {
            picks: vec![1, 3],
            modes: vec![
                Effect::LifeGainLockThisTurn { who: Selector::Player(PlayerRef::EachOpponent) },
                Effect::DealDamage { to: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
                // "Put a land from your hand onto the battlefield" — modeled via
                // the hand-or-graveyard put primitive (graveyard option unused).
                Effect::PutFromHandOrGraveyardOntoBattlefield { filter: SelectionRequirement::Land },
                Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
                    body: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::TriggerSource,
                            power: Value::Const(1), toughness: Value::Const(1), duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource, keyword: Keyword::Reach, duration: Duration::EndOfTurn,
                        },
                    ])),
                },
            ],
        },
        ..Default::default()
    }
}

// ── Graveyard-CDA creatures (*goyf family) ───────────────────────────────────

fn goyf_body(name: &'static str, c: crate::mana::ManaCost, types: Vec<CreatureType>, formula: DynamicPt)
    -> CardDefinition
{
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: 0,
        toughness: 0,
        dynamic_pt: Some(formula),
        ..Default::default()
    }
}

/// Lhurgoyf — {2}{G}{G} */1+*. Power = creature cards in all graveyards;
/// toughness = that number plus 1.
pub fn lhurgoyf() -> CardDefinition {
    goyf_body("Lhurgoyf", cost(&[generic(2), g(), g()]), vec![CreatureType::Lhurgoyf],
        DynamicPt::CreatureCardsInAllGraveyards { base_p: 0, base_t: 1 })
}

/// Mortivore — {2}{B}{B} */*. P/T = creature cards in all graveyards.
/// {B}: Regenerate this creature.
pub fn mortivore() -> CardDefinition {
    let mut def = goyf_body("Mortivore", cost(&[generic(2), b(), b()]), vec![CreatureType::Lhurgoyf],
        DynamicPt::CreatureCardsInAllGraveyards { base_p: 0, base_t: 0 });
    def.activated_abilities = vec![ActivatedAbility {
        mana_cost: cost(&[b()]),
        effect: Effect::Regenerate { what: Selector::This },
        ..Default::default()
    }];
    def
}

/// Boneyard Wurm — {1}{G} */*. P/T = creature cards in your graveyard.
pub fn boneyard_wurm() -> CardDefinition {
    goyf_body("Boneyard Wurm", cost(&[generic(1), g()]), vec![CreatureType::Wurm],
        DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 0 })
}

/// Splinterfright — {2}{G} */* Elemental. Trample. P/T = creature cards in your
/// graveyard. At the beginning of your upkeep, mill two cards.
pub fn splinterfright() -> CardDefinition {
    let mut def = goyf_body("Splinterfright", cost(&[generic(2), g()]), vec![CreatureType::Elemental],
        DynamicPt::BasePlusCreaturesInControllerGraveyard { base: 0 });
    def.keywords = vec![Keyword::Trample];
    def.triggered_abilities = vec![TriggeredAbility {
        event: EventSpec::new(
            EventKind::StepBegins(crate::game::types::TurnStep::Upkeep), EventScope::ActivePlayer),
        effect: Effect::Mill { who: Selector::You, amount: Value::Const(2) },
    }];
    def
}

// ── Other multicolor staples ─────────────────────────────────────────────────

/// Disciple of Bolas — {3}{B} 2/1 Human Wizard. ETB: sacrifice another
/// creature; gain life and draw cards equal to its power.
pub fn disciple_of_bolas() -> CardDefinition {
    CardDefinition {
        name: "Disciple of Bolas",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::SacrificeAndRemember {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource),
            },
            Effect::GainLife { who: Selector::You, amount: Value::SacrificedPower },
            Effect::Draw { who: Selector::You, amount: Value::SacrificedPower },
        ]))],
        ..Default::default()
    }
}

/// Agony Warp — {U}{B} Instant. Target creature gets -3/-0 and another (or the
/// same) target creature gets -0/-3 until end of turn.
pub fn agony_warp() -> CardDefinition {
    CardDefinition {
        name: "Agony Warp",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 0, filter: SelectionRequirement::Creature },
                power: Value::Const(-3), toughness: Value::Const(0), duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: Selector::TargetFiltered { slot: 1, filter: SelectionRequirement::Creature },
                power: Value::Const(0), toughness: Value::Const(-3), duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Savage Knuckleblade — {G}{U}{R} 4/4 Ogre Warrior. {2}{G}: +2/+2 (once each
/// turn). {2}{U}: return to hand. {R}: gains haste.
pub fn savage_knuckleblade() -> CardDefinition {
    CardDefinition {
        name: "Savage Knuckleblade",
        cost: cost(&[g(), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ogre, CreatureType::Warrior], ..Default::default()
        },
        power: 4,
        toughness: 4,
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                once_per_turn: true,
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(2), toughness: Value::Const(2), duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), u()]),
                effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::OwnerOfMoved) },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[r()]),
                effect: Effect::GrantKeyword {
                    what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Butcher of the Horde — {1}{R}{W}{B} 5/4 Demon. Flying. Sacrifice another
/// creature: this gains your choice of vigilance, lifelink, or haste until end
/// of turn.
pub fn butcher_of_the_horde() -> CardDefinition {
    CardDefinition {
        name: "Butcher of the Horde",
        cost: cost(&[generic(1), r(), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            // Sacrifice-another cost folded as the effect's first step.
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                },
                Effect::ChooseMode(vec![
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Vigilance, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Lifelink, duration: Duration::EndOfTurn },
                    Effect::GrantKeyword { what: Selector::This, keyword: Keyword::Haste, duration: Duration::EndOfTurn },
                ]),
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Demonic Dread — {1}{B}{R} Sorcery. Cascade. Target creature you control
/// gains fear until end of turn.
pub fn demonic_dread() -> CardDefinition {
    CardDefinition {
        name: "Demonic Dread",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::GrantKeyword {
            what: target_filtered(SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
            keyword: Keyword::Fear,
            duration: Duration::EndOfTurn,
        },
        triggered_abilities: vec![cascade(3)],
        ..Default::default()
    }
}

/// Glory — {3}{W}{W} 3/3 Incarnation. Flying. {2}{W}, only while in your
/// graveyard: choose a color; creatures you control gain protection from it
/// until end of turn.
pub fn glory() -> CardDefinition {
    CardDefinition {
        name: "Glory",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Incarnation], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), w()]),
            from_graveyard: true,
            effect: Effect::GrantProtectionFromChosenColor {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou)),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Foul-Tongue Invocation — {2}{B} Instant. Target player sacrifices a creature
/// of their choice. If you control a Dragon, you gain 4 life. (The "reveal a
/// Dragon from hand" alt-trigger is approximated as the control check.)
pub fn foul_tongue_invocation() -> CardDefinition {
    CardDefinition {
        name: "Foul-Tongue Invocation",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: target_filtered(SelectionRequirement::Player),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::CountOf(Box::new(Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Dragon)
                            .and(SelectionRequirement::ControlledByYou),
                    ))),
                    Value::Const(1),
                ),
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(4) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// The First Sliver — {W}{U}{B}{R}{G} Legendary 7/7 Sliver. Cascade. Sliver
/// spells you cast have cascade.
pub fn the_first_sliver() -> CardDefinition {
    CardDefinition {
        name: "The First Sliver",
        cost: cost(&[w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes { creature_types: vec![CreatureType::Sliver], ..Default::default() },
        power: 7,
        toughness: 7,
        triggered_abilities: vec![
            cascade(10),
            // "Sliver spells you cast have cascade." Only active while The First
            // Sliver is on the battlefield, so its own cast uses just cascade(10).
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Sliver),
                    },
                ),
                effect: Effect::Cascade { max_mv: Value::ManaValueOf(Box::new(Selector::TriggerSource)) },
            },
        ],
        ..Default::default()
    }
}
