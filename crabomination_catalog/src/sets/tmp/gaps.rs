//! Tempest (TMP) — the set-closing wave. Tests in `classic_sets/tmp`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{draw, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, MillShareAxis, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{ManaCost, b, cost, generic, r, u};

fn artifact(name: &'static str, c: ManaCost, abilities: Vec<ActivatedAbility>) -> CardDefinition {
    CardDefinition {
        name,
        cost: c,
        card_types: vec![CardType::Artifact],
        activated_abilities: abilities,
        ..Default::default()
    }
}

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

// ── Artifacts ───────────────────────────────────────────────────────────────

/// Grindstone — {1}. {3}, {T}: Target player mills two cards; repeat while the
/// pair shares a color.
pub fn grindstone() -> CardDefinition {
    artifact(
        "Grindstone",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::MillTwoRepeatSharing {
                who: target_filtered(R::Player),
                axis: MillShareAxis::AnyColor,
                draw_on_repeat: false,
            },
            ..Default::default()
        }],
    )
}

/// Cursed Scroll — {1}. {3}, {T}: Name a card, reveal one at random from your
/// hand; on a match, deal 2 damage to any target.
pub fn cursed_scroll() -> CardDefinition {
    artifact(
        "Cursed Scroll",
        cost(&[generic(1)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(3)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::NameCard { what: Selector::This, restrict_to: None },
                Effect::RevealRandomFromHand { who: Selector::You },
                Effect::If {
                    cond: Predicate::EntityMatchesAny {
                        what: Selector::LastRevealedCard,
                        filter: R::NamedBySource,
                    },
                    then: Box::new(Effect::DealDamage {
                        to: target_any(),
                        amount: Value::Const(2),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
            ..Default::default()
        }],
    )
}

/// Cold Storage — {4}. {3}: Exile target creature you control. Sacrifice this:
/// return every creature exiled with it.
pub fn cold_storage() -> CardDefinition {
    artifact(
        "Cold Storage",
        cost(&[generic(4)]),
        vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(3)]),
                effect: Effect::ExileWithSource {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                sac_cost: true,
                effect: Effect::ReturnExiledBySourceToBattlefield { decayed: false },
                ..Default::default()
            },
        ],
    )
}

/// Helm of Possession — {4}. {2}, {T}, Sacrifice a creature: gain control of
/// target creature while this stays tapped.
pub fn helm_of_possession() -> CardDefinition {
    artifact(
        "Helm of Possession",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::GainControlWhileSourceTapped {
                what: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
    )
}

// ── Creatures ───────────────────────────────────────────────────────────────

/// Minion of the Wastes — {3}{B}{B}{B} trampler whose P/T is the life paid as
/// it entered.
pub fn minion_of_the_wastes() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Trample],
        as_enters_effect: Some(Effect::PayAnyAmountOfLifeCapped {
            max: Value::LifeOf(PlayerRef::You),
        }),
        dynamic_pt: Some(crate::card::DynamicPt::ChosenNumberAsEntered),
        ..creature(
            "Minion of the Wastes",
            cost(&[generic(3), b(), b(), b()]),
            vec![CreatureType::Minion],
            0,
            0,
        )
    }
}

/// Unstable Shapeshifter — {3}{U} 0/1. Whenever another creature enters, it
/// becomes a copy of that creature, except it keeps this ability.
pub fn unstable_shapeshifter() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::BecomeCopyOf {
                what: Selector::This,
                source: Selector::TriggerSource,
                extra_creature_types: vec![],
                keep_own_triggered: true,
                keep_own_activated: false,
            },
        }],
        ..creature("Unstable Shapeshifter", cost(&[generic(3), u()]), vec![CreatureType::Shapeshifter], 0, 1)
    }
}

/// Carrionette — {1}{B} 1/1. From the graveyard, {2}{B}{B}: exile it and a
/// creature unless that creature's controller pays {2}.
pub fn carrionette() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b(), b()]),
            from_graveyard: true,
            effect: Effect::UnlessPlayerPays {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                cost: crate::card::WardCost::Mana(cost(&[generic(2)])),
                then: Box::new(Effect::Seq(vec![
                    Effect::Exile { what: Selector::Target(0) },
                    Effect::Exile { what: Selector::This },
                ])),
                if_paid: None,
            },
            ..Default::default()
        }],
        ..creature("Carrionette", cost(&[generic(1), b()]), vec![CreatureType::Skeleton], 1, 1)
    }
}

/// Starke of Rath — {1}{R}{R} 2/2. {T}: Destroy target artifact or creature;
/// its controller gains control of Starke.
pub fn starke_of_rath() -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Destroy { what: target_filtered(R::Artifact.or(R::Creature)) },
                Effect::GainControl {
                    what: Selector::This,
                    to: Some(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                    duration: Duration::Permanent,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Starke of Rath",
            cost(&[generic(1), r(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue],
            2,
            2,
        )
    }
}

/// Magmasaur — {3}{R}{R} 0/0 that enters with five +1/+1 counters and blows up
/// the board if you stop feeding it one each upkeep.
pub fn magmasaur() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    use crate::game::TurnStep;
    let counters = Value::CountersOn {
        what: Box::new(Selector::This),
        kind: CounterType::PlusOnePlusOne,
    };
    CardDefinition {
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::SelfSource),
            effect: Effect::MayDoElse {
                description: "Remove a +1/+1 counter from Magmasaur".into(),
                body: Box::new(Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
                else_: Box::new(Effect::Seq(vec![
                    Effect::SacrificeSelected { what: Selector::This },
                    Effect::DealDamage {
                        to: Selector::EachPermanent(
                            R::Creature.and(R::Not(Box::new(R::HasKeyword(Keyword::Flying)))),
                        ),
                        amount: counters.clone(),
                    },
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachPlayer),
                        amount: counters,
                    },
                ])),
            },
        }],
        ..creature(
            "Magmasaur",
            cost(&[generic(3), r(), r()]),
            vec![CreatureType::Elemental, CreatureType::Dinosaur],
            0,
            0,
        )
    }
}

/// Wood Sage — {G}{U} 1/1. {T}: Name a creature card, reveal the top four, take
/// every copy of that name and bin the rest.
pub fn wood_sage() -> CardDefinition {
    use crate::mana::{g, u};
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::NameCard { what: Selector::This, restrict_to: Some(R::Creature) },
                Effect::RevealTopTakeMatchingRestToGraveyard {
                    who: PlayerRef::You,
                    count: Value::Const(4),
                    filter: R::NamedBySource,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Wood Sage",
            cost(&[g(), u()]),
            vec![CreatureType::Human, CreatureType::Druid],
            1,
            1,
        )
    }
}

// ── Spells ──────────────────────────────────────────────────────────────────

/// Abandon Hope — {X}{1}{B} Sorcery. Discard X as an additional cost, then pick
/// X cards out of target opponent's hand for them to discard.
pub fn abandon_hope() -> CardDefinition {
    CardDefinition {
        name: "Abandon Hope",
        cost: cost(&[crate::mana::x(), generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        additional_cast_cost: vec![AdditionalCastCost::DiscardXFromCost],
        effect: Effect::DiscardChosen {
            from: target_filtered(R::OpponentPlayer),
            count: Value::XFromCost,
            filter: R::Any,
        },
        ..Default::default()
    }
}

// ── Lands ───────────────────────────────────────────────────────────────────

/// Maze of Shadows — {T}: Add {C}. {T}: Untap an attacking shadow creature and
/// fog it both ways.
pub fn maze_of_shadows() -> CardDefinition {
    CardDefinition {
        name: "Maze of Shadows",
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                effect: crate::effect::shortcut::add_colorless(1),
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                effect: Effect::Seq(vec![
                    Effect::Untap {
                        what: target_filtered(
                            R::Creature.and(R::IsAttacking).and(R::HasKeyword(Keyword::Shadow)),
                        ),
                        up_to: None,
                    },
                    Effect::PreventAllCombatDamageInvolving { target: Selector::Target(0) },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Phyrexian Grimoire — {3} Book. {4}, {T}: Target opponent picks one of the
/// top two cards of your graveyard to exile; the other goes to your hand.
pub fn phyrexian_grimoire() -> CardDefinition {
    use crate::card::{ArtifactSubtype, Subtypes};
    CardDefinition {
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Book],
            ..Default::default()
        },
        ..artifact(
            "Phyrexian Grimoire",
            cost(&[generic(3)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(4)]),
                tap_cost: true,
                effect: Effect::TopTwoGraveyardOpponentSplits { who: target_filtered(R::OpponentPlayer) },
                ..Default::default()
            }],
        )
    }
}

/// Reap — {1}{G} Instant. Slot 0 is the opponent; slots 1-4 are graveyard
/// cards, each returned only if the opponent's black permanent count reaches
/// that far (four is the practical ceiling for "up to X").
pub fn reap() -> CardDefinition {
    use crate::mana::g;
    let blacks = Value::CountOf(Box::new(Selector::ControlledBy {
        who: PlayerRef::Target(0),
        filter: R::HasColor(crate::mana::Color::Black),
    }));
    let take = |slot: u8| Effect::If {
        cond: Predicate::ValueAtLeast(blacks.clone(), Value::Const(slot as i32)),
        then: Box::new(Effect::Move {
            what: Selector::Target(slot),
            to: ZoneDest::Hand(PlayerRef::You),
        }),
        else_: Box::new(Effect::Noop),
    };
    CardDefinition {
        name: "Reap",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::OptionalTargets {
            min: 1,
            body: Box::new(Effect::Seq((1..=4).map(take).collect())),
        },
        ..Default::default()
    }
}

/// Interdict — {1}{U} Instant. Counter target activated ability and lock that
/// permanent's activated abilities for the turn, then draw.
pub fn interdict() -> CardDefinition {
    CardDefinition {
        name: "Interdict",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterAbility {
                what: target_filtered(
                    R::Artifact.or(R::Creature).or(R::Enchantment).or(R::Land),
                ),
            },
            Effect::LockActivatedAbilitiesThisTurn { what: Selector::Target(0) },
            draw(1),
        ]),
        ..Default::default()
    }
}
