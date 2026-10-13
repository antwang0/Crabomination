//! Commander: the 1500 rescan's two-card gaps for Omarthis, Setzer, Atemsis,
//! Serah Farron, Lazav (Wearer of Faces) and Kaza (`cmdr_edhrec32` is the
//! previous batch). Tests in `tests/recent_b/cmdr_edhrec33.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType, EquipBonus, EventKind,
    EventScope, EventSpec, Keyword, LandType, Predicate, SelectionRequirement as R, Selector, StaticAbility,
    StaticEffect, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::{investigate, target_filtered};
use crate::effect::{Duration, Effect, ManaPayload, PlayerRef, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{cost, g, generic, r, u, Color, ManaCost};

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

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn clue() -> R {
    R::HasArtifactSubtype(ArtifactSubtype::Clue)
}

/// "Whenever you sacrifice a Clue".
fn clue_sacrificed(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
            .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: clue() }),
        effect,
    }
}

// ── Omarthis, Ghostfire Initiate ─────────────────────────────────────────────

/// Drannith Ruins — land. {T}: Add {C}. {2}, {T}: put two +1/+1 counters on
/// target non-Human creature that entered this turn.
pub fn drannith_ruins() -> CardDefinition {
    CardDefinition {
        name: "Drannith Ruins",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            crate::sets::tap_add_colorless(),
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(2)]),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        R::Creature
                            .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Human))))
                            .and(R::EnteredThisTurn),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Chronomaton — {1} 1/1 artifact creature Golem. {1}, {T}: put a +1/+1
/// counter on it.
pub fn chronomaton() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::AddCounter { what: Selector::This, kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
            ..Default::default()
        }],
        ..creature("Chronomaton", cost(&[generic(1)]), vec![CreatureType::Golem], 1, 1)
    }
}

// ── Setzer, Wandering Gambler ────────────────────────────────────────────────

/// Speedway Fanatic — {1}{R} 2/1 Human Pilot, haste. Whenever it crews a
/// Vehicle, that Vehicle gains haste until end of turn.
pub fn speedway_fanatic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CrewsOrSaddles { main_phase_only: false }, EventScope::SelfSource)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                }),
            effect: Effect::GrantKeyword {
                what: Selector::TriggerSource,
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature("Speedway Fanatic", cost(&[generic(1), r()]), vec![CreatureType::Human, CreatureType::Pilot], 2, 1)
    }
}

/// Start Your Engines — {3}{R} sorcery. Vehicles you control become artifact
/// creatures until end of turn; creatures you control get +2/+0.
pub fn start_your_engines() -> CardDefinition {
    CardDefinition {
        name: "Start Your Engines",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AnimateAsCreature {
                what: yours(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                duration: Duration::EndOfTurn,
            },
            Effect::PumpPT {
                what: yours(R::Creature),
                power: Value::Const(2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

// ── Atemsis, All-Seeing ──────────────────────────────────────────────────────

/// Atemsis, All-Seeing — {3}{U}{U}{U} legendary 4/5 Sphinx, flying. {2}{U},
/// {T}: draw two, then discard one. Whenever it deals damage to an opponent,
/// you may reveal your hand; with six different mana values revealed, that
/// player loses the game.
pub fn atemsis_all_seeing() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), u()]),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::Discard { who: Selector::You, amount: Value::ONE, random: false },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsDamageToPlayer, EventScope::SelfSource)
                .with_filter(Predicate::PlayerIsOpponent { who: PlayerRef::Target(0) }),
            // Revealing only matters when it wins, so the "may" is taken then.
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::DistinctManaValuesInHand(PlayerRef::You), Value::Const(6)),
                then: Box::new(Effect::LoseGame { who: PlayerRef::Target(0) }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..creature("Atemsis, All-Seeing", cost(&[generic(3), u(), u(), u()]), vec![CreatureType::Sphinx], 4, 5)
    }
}

/// Sorcerer's Wand — {1} Equipment. Equipped creature has "{T}: This creature
/// deals 1 damage to target player or planeswalker, 2 if it's a Wizard."
/// Equip {3}.
pub fn sorcerers_wand() -> CardDefinition {
    let wizard = Predicate::EntityMatches { what: Selector::This, filter: R::HasCreatureType(CreatureType::Wizard) };
    CardDefinition {
        name: "Sorcerer's Wand",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
        equipped_bonus: Some(EquipBonus {
            activated_abilities: vec![ActivatedAbility {
                tap_cost: true,
                effect: Effect::DealDamage {
                    to: target_filtered(R::Player.or(R::Planeswalker)),
                    amount: Value::IfPred {
                        pred: Box::new(wizard),
                        then: Box::new(Value::Const(2)),
                        else_: Box::new(Value::ONE),
                    },
                },
                ..Default::default()
            }],
            ..Default::default()
        }),
        ..Default::default()
    }
}

// ── Serah Farron ─────────────────────────────────────────────────────────────

/// Casal, Lurkwood Pathfinder // Casal, Pathbreaker Owlbear — {3}{G} legendary
/// 3/3 Tiefling Druid, vigilance. Enters: a Forest card onto the battlefield
/// tapped. Attacks: you may pay {1}{G} to transform her. Back: 6/6 Bird Bear,
/// vigilance, trample; transforming into it gives your other legendary
/// creatures +2/+2 and trample until end of turn; at your upkeep, transform.
pub fn casal_lurkwood_pathfinder() -> CardDefinition {
    let owlbear = CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance, Keyword::Trample],
        color_indicator: vec![Color::Green],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::PumpPT {
                        what: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::OtherThanSource)),
                        power: Value::Const(2),
                        toughness: Value::Const(2),
                        duration: Duration::EndOfTurn,
                    },
                    Effect::GrantKeyword {
                        what: yours(R::Creature.and(R::HasSupertype(Supertype::Legendary)).and(R::OtherThanSource)),
                        keyword: Keyword::Trample,
                        duration: Duration::EndOfTurn,
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
                effect: Effect::Transform { what: Selector::This },
            },
        ],
        ..creature(
            "Casal, Pathbreaker Owlbear",
            ManaCost::default(),
            vec![CreatureType::Bird, CreatureType::Bear],
            6,
            6,
        )
    };
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::Search {
                who: PlayerRef::You,
                filter: R::HasLandType(LandType::Forest),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::MayPay {
                    description: "Pay {1}{G} to transform Casal".into(),
                    mana_cost: cost(&[generic(1), g()]),
                    body: Box::new(Effect::Transform { what: Selector::This }),
                    else_: None,
                },
            },
        ],
        back_face: Some(Box::new(owlbear)),
        ..creature(
            "Casal, Lurkwood Pathfinder",
            cost(&[generic(3), g()]),
            vec![CreatureType::Tiefling, CreatureType::Druid],
            3,
            3,
        )
    }
}

/// Master's Guidance — {2}{G} enchantment. Whenever you attack with two or
/// more legendary creatures, a +1/+1 counter on each of up to two target
/// attacking creatures. At your end step, if you control a creature with
/// power 4 or greater, draw a card.
pub fn masters_guidance() -> CardDefinition {
    CardDefinition {
        name: "Master's Guidance",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::YouAttack, EventScope::YourControl).with_filter(
                    Predicate::SelectorCountAtLeast {
                        sel: yours(R::Creature.and(R::IsAttacking).and(R::HasSupertype(Supertype::Legendary))),
                        n: Value::Const(2),
                    },
                ),
                effect: Effect::ApplyToTargets {
                    max_targets: 2,
                    min_targets: 0,
                    filter: R::Creature.and(R::IsAttacking),
                    effect: Box::new(Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl).with_filter(
                    Predicate::SelectorExists(Selector::ControlledBy {
                        who: PlayerRef::You,
                        filter: R::Creature.and(R::PowerAtLeast(4)),
                    }),
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
            },
        ],
        ..Default::default()
    }
}

// ── Lazav, Wearer of Faces ───────────────────────────────────────────────────

/// Senator Peacock — {3}{U}{U} legendary 3/4 Human Advisor. Artifacts you
/// control are Clues and have "{2}, Sacrifice this artifact: Draw a card."
/// Whenever you sacrifice a Clue, target creature can't be blocked this turn.
pub fn senator_peacock() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        static_abilities: vec![
            StaticAbility {
                description: "Artifacts you control are Clues in addition to their other types.",
                effect: StaticEffect::AddCardTypeToMatching {
                    applies_to: yours(R::Artifact),
                    card_type: CardType::Artifact,
                    artifact_subtype: Some(ArtifactSubtype::Clue),
                },
            },
            StaticAbility {
                description: "Artifacts you control have \"{2}, Sacrifice this artifact: Draw a card.\"",
                effect: StaticEffect::GrantActivatedAbility {
                    applies_to: yours(R::Artifact),
                    ability: ActivatedAbility {
                        mana_cost: cost(&[generic(2)]),
                        sac_cost: true,
                        effect: Effect::Draw { who: Selector::You, amount: Value::ONE },
                        ..Default::default()
                    },
                    condition: None,
                },
            },
        ],
        // Every artifact you control is a Clue while Peacock is here, so "you
        // sacrifice a Clue" is "you sacrifice an artifact" for its own trigger
        // (the sacrificed card no longer carries the granted subtype).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::Artifact }),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature),
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            },
        }],
        ..creature(
            "Senator Peacock",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Advisor],
            3,
            4,
        )
    }
}

/// Fleeting Memories — {2}{U} enchantment. Enters: investigate. Whenever you
/// sacrifice a Clue, target player mills three cards.
pub fn fleeting_memories() -> CardDefinition {
    CardDefinition {
        name: "Fleeting Memories",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            crate::effect::shortcut::etb(investigate(1)),
            clue_sacrificed(Effect::Mill { who: Selector::Player(PlayerRef::Target(0)), amount: Value::Const(3) }),
        ],
        ..Default::default()
    }
}

// ── Kaza, Roil Chaser ────────────────────────────────────────────────────────

/// Malevolent Hermit // Benevolent Geist — {1}{U} 2/1 Human Wizard. {U},
/// sacrifice it: counter target noncreature spell unless its controller pays
/// {3}. Disturb {2}{U}: a 2/2 flying Spirit Wizard; noncreature spells you
/// control can't be countered.
pub fn malevolent_hermit() -> CardDefinition {
    let geist = CardDefinition {
        keywords: vec![Keyword::Flying],
        color_indicator: vec![Color::Blue],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you control can't be countered.",
            effect: StaticEffect::SpellsCantBeCounteredMatching { filter: R::Noncreature.and(R::ControlledByYou) },
        }],
        ..creature(
            "Benevolent Geist",
            ManaCost::default(),
            vec![CreatureType::Spirit, CreatureType::Wizard],
            2,
            2,
        )
    };
    CardDefinition {
        keywords: vec![Keyword::Disturb(cost(&[generic(2), u()]))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterUnlessPaid {
                what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
                mana_cost: cost(&[generic(3)]),
                exile: false,
                extra_generic: None,
                if_paid: None,
            },
            ..Default::default()
        }],
        back_face: Some(Box::new(geist)),
        ..creature("Malevolent Hermit", cost(&[generic(1), u()]), vec![CreatureType::Human, CreatureType::Wizard], 2, 1)
    }
}

/// Volshe Tideturner — {1}{U} 1/3 Merfolk Wizard. {T}: Add {U}; spend it only
/// on an instant or sorcery spell or a kicked spell.
pub fn volshe_tideturner() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Restricted(
                    Box::new(ManaPayload::Colors(vec![Color::Blue])),
                    crate::mana::SpendRestriction::InstantSorceryOrKicked,
                ),
            },
            ..Default::default()
        }],
        ..creature("Volshe Tideturner", cost(&[generic(1), u()]), vec![CreatureType::Merfolk, CreatureType::Wizard], 1, 3)
    }
}
