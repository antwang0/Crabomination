//! Tempest (TMP) — the set-closing wave. Tests in `classic_sets/tmp`.

use crate::card::{
    ActivatedAbility, AdditionalCastCost, CardDefinition, CardType, CounterType, CreatureType,
    Keyword, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::{draw, target_any, target_filtered};
use crate::effect::{
    Duration, Effect, MillShareAxis, PlayerRef, Predicate, Selector, Value, ZoneDest,
};
use crate::mana::{ManaCost, b, cost, generic, r, u, w};

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

// ── Wave 2 ──────────────────────────────────────────────────────────────────

/// Static Orb — {3}. While untapped, nobody untaps more than two permanents
/// per untap step.
pub fn static_orb() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Players can't untap more than two permanents during their untap steps.",
            effect: StaticEffect::WhileCondition {
                condition: Predicate::EntityMatches {
                    what: Selector::This,
                    filter: R::Untapped,
                },
                inner: Box::new(StaticEffect::MaxUntapsPerStep { filter: R::Any, max: 2 }),
            },
        }],
        ..artifact("Static Orb", cost(&[generic(3)]), vec![])
    }
}

/// Hand to Hand — {2}{R}. During combat nobody casts instants or activates
/// non-mana abilities.
pub fn hand_to_hand() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Hand to Hand",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "During combat, players can't cast instants or activate non-mana abilities.",
            effect: StaticEffect::NoInstantsOrAbilitiesDuringCombat,
        }],
        ..Default::default()
    }
}

/// Pallimud — {2}{R} */3 whose power is the tapped lands of the opponent
/// chosen as it entered.
pub fn pallimud() -> CardDefinition {
    CardDefinition {
        as_enters_effect: Some(Effect::RememberPlayerOnSource {
            who: PlayerRef::EachOpponent,
        }),
        dynamic_pt: Some(crate::card::DynamicPt::TappedLandsChosenPlayerControls { base_t: 3 }),
        ..creature("Pallimud", cost(&[generic(2), r()]), vec![CreatureType::Beast], 0, 3)
    }
}

/// Dracoplasm — {U}{R} flier that enters as the sum of the creatures
/// sacrificed for it.
pub fn dracoplasm() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        as_enters_effect: Some(Effect::AsEntersSacrificeForTotalPt),
        dynamic_pt: Some(crate::card::DynamicPt::EnteredTotals),
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[r()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..creature("Dracoplasm", cost(&[u(), r()]), vec![CreatureType::Shapeshifter], 0, 0)
    }
}

/// Escaped Shapeshifter — {3}{U}{U} 3/4 that mirrors the keywords on
/// opponents' creatures.
pub fn escaped_shapeshifter() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::StaticEffect;
    let mirror = |kw: Keyword| StaticAbility {
        description: "Mirrors a keyword an opponent's creature has.",
        effect: StaticEffect::WhileCondition {
            condition: Predicate::SelectorCountAtLeast {
                sel: Selector::EachPermanent(
                    R::Creature
                        .and(R::ControlledByOpponent)
                        .and(R::HasKeyword(kw.clone()))
                        .and(R::Not(Box::new(R::HasName("Escaped Shapeshifter".into())))),
                ),
                n: Value::ONE,
            },
            inner: Box::new(StaticEffect::GrantKeyword {
                applies_to: Selector::This,
                keyword: kw,
            }),
        },
    };
    CardDefinition {
        static_abilities: [Keyword::Flying, Keyword::FirstStrike, Keyword::Trample]
            .into_iter()
            .map(mirror)
            .collect(),
        ..creature(
            "Escaped Shapeshifter",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Shapeshifter],
            3,
            4,
        )
    }
}

/// Flowstone Sculpture — {6} 4/4. {2}, Discard a card: grow, or gain flying,
/// first strike or trample for good.
pub fn flowstone_sculpture() -> CardDefinition {
    use crate::effect::Duration;
    let grant = |kw: Keyword| Effect::GrantKeyword {
        what: Selector::This,
        keyword: kw,
        duration: Duration::Permanent,
    };
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            discard_cost: Some((R::Any, 1)),
            effect: Effect::ChooseMode(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                grant(Keyword::Flying),
                grant(Keyword::FirstStrike),
                grant(Keyword::Trample),
            ]),
            ..Default::default()
        }],
        ..creature(
            "Flowstone Sculpture",
            cost(&[generic(6)]),
            vec![CreatureType::Shapeshifter],
            4,
            4,
        )
    }
}

/// Excavator — {2}. {T}, Sacrifice a basic land: target creature gains that
/// land's landwalk until end of turn.
pub fn excavator() -> CardDefinition {
    artifact(
        "Excavator",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            tap_cost: true,
            sac_other_filter: Some((R::Land.and(R::IsBasicLand), 1)),
            effect: Effect::GrantSacrificedLandTypesLandwalk {
                what: target_filtered(R::Creature),
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
    )
}

/// Soltari Guerrillas — {2}{R}{W} 3/2 shadow. {0}: redirect its next combat
/// damage to a creature.
pub fn soltari_guerrillas() -> CardDefinition {
    use crate::mana::w;
    CardDefinition {
        keywords: vec![Keyword::Shadow],
        activated_abilities: vec![ActivatedAbility {
            effect: Effect::RedirectNextCombatDamageTo {
                what: Selector::This,
                to: target_filtered(R::Creature),
            },
            ..Default::default()
        }],
        ..creature(
            "Soltari Guerrillas",
            cost(&[generic(2), r(), w()]),
            vec![CreatureType::Soltari, CreatureType::Soldier],
            3,
            2,
        )
    }
}

/// No Quarter — {3}{R}. A creature blocked by something weaker kills it, and a
/// blocker that bites off more than it can chew dies too.
pub fn no_quarter() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    CardDefinition {
        name: "No Quarter",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecomesBlocked, EventScope::AnyPlayer),
                effect: Effect::DestroyBlockPairWeakerSide { attacker_side: false },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Blocks, EventScope::AnyPlayer),
                effect: Effect::DestroyBlockPairWeakerSide { attacker_side: true },
            },
        ],
        ..Default::default()
    }
}

// ── Licids with riders ──────────────────────────────────────────────────────

fn rider_licid(
    name: &'static str,
    color: crate::mana::ManaSymbol,
    attach_cost: ManaCost,
    extra: Vec<ActivatedAbility>,
    triggers: Vec<TriggeredAbility>,
) -> CardDefinition {
    let mut abilities = vec![ActivatedAbility {
        mana_cost: attach_cost,
        tap_cost: true,
        effect: Effect::LicidAttach {
            host: target_filtered(R::Creature),
            end_cost: cost(&[color]),
        },
        ..Default::default()
    }];
    abilities.extend(extra);
    CardDefinition {
        activated_abilities: abilities,
        triggered_abilities: triggers,
        ..creature(name, cost(&[generic(1), color]), vec![CreatureType::Licid], 1, 1)
    }
}

/// Leeching Licid — {1}{B}. As an Aura it pings the host's controller each of
/// their upkeeps.
pub fn leeching_licid() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    use crate::game::TurnStep;
    let host = || Selector::AttachedTo(Box::new(Selector::This));
    rider_licid(
        "Leeching Licid",
        b(),
        cost(&[b()]),
        vec![],
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::AnyPlayer)
                .with_filter(Predicate::IsTurnOf(PlayerRef::ControllerOf(Box::new(host())))),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(host()))),
                amount: Value::ONE,
            },
        }],
    )
}

/// Nurturing Licid — {1}{G}. As an Aura, {G} regenerates the host.
pub fn nurturing_licid() -> CardDefinition {
    use crate::mana::g;
    rider_licid(
        "Nurturing Licid",
        g(),
        cost(&[g()]),
        vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            effect: Effect::Regenerate {
                what: Selector::AttachedTo(Box::new(Selector::This)),
            },
            ..Default::default()
        }],
        vec![],
    )
}

/// Stinging Licid — {1}{U}. As an Aura it shocks the host's controller
/// whenever the host taps.
pub fn stinging_licid() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    rider_licid(
        "Stinging Licid",
        u(),
        cost(&[generic(1), u()]),
        vec![],
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Tapped, EventScope::AnyPlayer)
                .with_filter(Predicate::TriggerSourceIsSourceHost),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::AttachedTo(
                    Box::new(Selector::This),
                )))),
                amount: Value::Const(2),
            },
        }],
    )
}

// ── Wave 3 ──────────────────────────────────────────────────────────────────

/// Thalakos Dreamsower — {2}{U} 1/1 shadow. Its damage taps a creature and
/// pins it while the Dreamsower stays tapped.
pub fn thalakos_dreamsower() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    CardDefinition {
        keywords: vec![Keyword::Shadow, Keyword::MayChooseNotToUntap],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::PlayerDamaged, EventScope::SelfSource),
            effect: Effect::TapAndHoldWhileSourceTapped {
                what: target_filtered(R::Creature),
            },
        }],
        ..creature(
            "Thalakos Dreamsower",
            cost(&[generic(2), u()]),
            vec![CreatureType::Thalakos, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Volrath's Curse — {1}{U} Aura. The host is shut off until its controller
/// sacrifices a permanent to shrug the Curse off for the turn.
pub fn volraths_curse() -> CardDefinition {
    use crate::card::{EnchantmentSubtype, EquipBonus};
    CardDefinition {
        name: "Volrath's Curse",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach { what: Selector::This, to: target_filtered(R::Creature) },
        equipped_bonus: Some(EquipBonus {
            keywords: vec![
                Keyword::CantAttack,
                Keyword::CantBlock,
                Keyword::CantActivateAbilities,
            ],
            ..Default::default()
        }),
        activated_abilities: vec![
            ActivatedAbility {
                // The printed line is a special action for the host's
                // controller; modeled as an any-player sacrifice activation.
                any_player: true,
                sac_other_filter: Some((R::Any, 1)),
                effect: Effect::IgnoreStaticFromSourceThisTurn,
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1), u()]),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::This))),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Coffin Queen — {2}{B} 1/1. Reanimates out of any graveyard, and the loan is
/// called in the moment she untaps.
pub fn coffin_queen() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    let recall = |event| TriggeredAbility {
        event,
        effect: Effect::Exile { what: Selector::ChosenPermanentOfSource },
    };
    CardDefinition {
        keywords: vec![Keyword::MayChooseNotToUntap],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(R::Creature.and(R::InGraveyard)),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::RememberPermanentOnSource { what: Selector::LastMoved },
            ]),
            ..Default::default()
        }],
        triggered_abilities: vec![
            recall(EventSpec::new(EventKind::BecomesUntapped, EventScope::SelfSource)),
            recall(EventSpec::new(
                EventKind::PermanentLeavesBattlefield,
                EventScope::SelfSource,
            )),
        ],
        ..creature(
            "Coffin Queen",
            cost(&[generic(2), b()]),
            vec![CreatureType::Zombie, CreatureType::Wizard],
            1,
            1,
        )
    }
}

/// Maddening Imp — {2}{B} 1/1 flier. Forces the active player's non-Walls into
/// combat and kills the ones that stay home.
pub fn maddening_imp() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::Seq(vec![
                Effect::GrantKeyword {
                    what: Selector::ControlledBy {
                        who: PlayerRef::ActivePlayer,
                        filter: R::Creature
                            .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Wall)))),
                    },
                    keyword: Keyword::MustAttack,
                    duration: Duration::EndOfTurn,
                },
                Effect::AtNextEndStep {
                    body: Box::new(Effect::Destroy {
                        what: Selector::EachPermanent(
                            R::Creature
                                .and(R::ControlledByActivePlayer)
                                .and(R::Not(Box::new(R::HasCreatureType(CreatureType::Wall))))
                                .and(R::Not(Box::new(R::AttackedThisTurn))),
                        ),
                    }),
                },
            ]),
            ..Default::default()
        }],
        ..creature("Maddening Imp", cost(&[generic(2), b()]), vec![CreatureType::Imp], 1, 1)
    }
}

/// Phyrexian Splicer — {2}. {2}, {T}: move one of four evasion keywords from
/// one creature to another for the turn.
pub fn phyrexian_splicer() -> CardDefinition {
    artifact(
        "Phyrexian Splicer",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            tap_cost: true,
            effect: Effect::MoveChosenKeyword {
                options: vec![
                    Keyword::Flying,
                    Keyword::FirstStrike,
                    Keyword::Trample,
                    Keyword::Shadow,
                ],
                from: Selector::Target(0),
                to: Selector::Target(1),
            },
            ..Default::default()
        }],
    )
}

/// Scroll Rack — {2}. {1}, {T}: swap any number of cards from your hand for the
/// same number off the top, then stack the exiled ones back on top.
pub fn scroll_rack() -> CardDefinition {
    artifact(
        "Scroll Rack",
        cost(&[generic(2)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::ScrollRack,
            ..Default::default()
        }],
    )
}

/// Echo Chamber — {4}. {4}, {T}: an opponent picks one of their creatures and
/// you get a hasty token copy until the end step. Sorcery speed only.
pub fn echo_chamber() -> CardDefinition {
    artifact(
        "Echo Chamber",
        cost(&[generic(4)]),
        vec![ActivatedAbility {
            mana_cost: cost(&[generic(4)]),
            tap_cost: true,
            sorcery_speed: true,
            effect: Effect::TokenCopyOfOpponentChoice { who: PlayerRef::EachOpponent },
            ..Default::default()
        }],
    )
}

// ── Wave 4 ──────────────────────────────────────────────────────────────────

/// Whim of Volrath — {U} Instant, buyback {2}. Rewrite one colour word or one
/// basic land type on a permanent for the turn.
pub fn whim_of_volrath() -> CardDefinition {
    let what = || target_filtered(R::Permanent);
    CardDefinition {
        name: "Whim of Volrath",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        keywords: vec![Keyword::Buyback(cost(&[generic(2)]))],
        effect: Effect::ChooseMode(vec![
            Effect::ReplaceColorWord { what: what(), duration: Duration::EndOfTurn },
            Effect::ReplaceBasicLandType { what: what(), duration: Duration::EndOfTurn },
        ]),
        ..Default::default()
    }
}

/// Magnetic Web — {2}. Magnet counters lock their bearers into attacking and
/// blocking together.
pub fn magnetic_web() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, StaticAbility};
    use crate::effect::StaticEffect;
    let magnets = || R::Creature.and(R::WithCounter(CounterType::Magnet));
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "Creatures with magnet counters attack together.",
            effect: StaticEffect::AttackTogether { filter: magnets() },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::AnyPlayer).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::WithCounter(CounterType::Magnet),
                },
            ),
            effect: Effect::GrantKeyword {
                what: Selector::EachPermanent(magnets().and(R::Not(Box::new(R::IsAttacking)))),
                keyword: Keyword::MustBlock,
                duration: Duration::EndOfTurn,
            },
        }],
        ..artifact(
            "Magnetic Web",
            cost(&[generic(2)]),
            vec![ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                tap_cost: true,
                effect: Effect::AddCounter {
                    what: target_filtered(R::Creature),
                    kind: CounterType::Magnet,
                    amount: Value::ONE,
                },
                ..Default::default()
            }],
        )
    }
}

/// Booby Trap — {6}. Name a card as it enters; when the chosen opponent draws
/// it, the Trap goes off for 10.
pub fn booby_trap() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, StaticAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        as_enters_effect: Some(Effect::Seq(vec![
            Effect::RememberPlayerOnSource { who: PlayerRef::EachOpponent },
            Effect::NameCard {
                what: Selector::This,
                restrict_to: Some(R::Not(Box::new(R::Land.and(R::IsBasicLand)))),
            },
        ])),
        static_abilities: vec![StaticAbility {
            description: "The chosen player reveals each card they draw.",
            effect: StaticEffect::OpponentsPlayWithHandsRevealed,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl).with_filter(
                Predicate::EntityMatchesAny {
                    what: Selector::TriggerSource,
                    filter: R::NamedBySource,
                },
            ),
            effect: Effect::Seq(vec![
                Effect::SacrificeSelected { what: Selector::This },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::ChosenPlayerOfSource),
                    amount: Value::Const(10),
                },
            ]),
        }],
        ..artifact("Booby Trap", cost(&[generic(6)]), vec![])
    }
}

// ── Wave 5: the set's last three ────────────────────────────────────────────

/// Duplicity — {3}{U}{U}. A five-card face-down reserve you swap your hand
/// with each upkeep, paid for with a card every end step.
pub fn duplicity() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec};
    fn bin_the_pile() -> Effect {
        Effect::Move {
            what: Selector::CardsInZone {
                who: PlayerRef::You,
                zone: crate::card::Zone::Exile,
                filter: R::ExiledWithSource,
            },
            to: ZoneDest::Graveyard,
        }
    }
    CardDefinition {
        name: "Duplicity",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::ExileTopOfLibrary {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(5),
                    link_to_source: true,
                    face_down: true,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::ExileHandThenReclaimLinked,
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::TurnStep::End),
                    EventScope::YourControl,
                ),
                effect: Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            },
            // "When you lose control of this" — both ways to lose it: the
            // permanent leaving, and a control change with it still in play.
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentLeavesBattlefield, EventScope::SelfSource),
                effect: bin_the_pile(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::LostControlOfThis, EventScope::SelfSource),
                effect: bin_the_pile(),
            },
        ],
        ..Default::default()
    }
}

/// Oracle en-Vec — {1}{W} 1/1. Names an opponent's attackers for their next
/// turn; the ones that stay home die.
pub fn oracle_en_vec() -> CardDefinition {
    use crate::card::{CreatureType, Subtypes};
    CardDefinition {
        name: "Oracle en-Vec",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::AttackMandateNextTurn { who: PlayerRef::Target(0) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ertai's Meddling — {X}{U} Instant. Puts a spell on ice for X of its
/// controller's upkeeps, then hands it back.
pub fn ertais_meddling() -> CardDefinition {
    CardDefinition {
        name: "Ertai's Meddling",
        cost: cost(&[crate::mana::x(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileSpellWithDelayCounters {
            what: target_filtered(R::IsSpellOnStack),
            count: Value::XFromCost,
        },
        ..Default::default()
    }
}
