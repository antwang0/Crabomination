//! Final Fantasy (FIN) gap closure: the Town // Adventure land cycle and the
//! Sidequest transforming enchantments. Tests in `classic_sets/fin2`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, DynamicPt, EquipBonus, EventKind, EventScope, EventSpec, Keyword, LandType,
    Predicate, SelectionRequirement as R, StaticAbility, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest,
    shortcut::{discard, draw, etb, on_attack, on_dies, target_filtered},
};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w};

// ── Town // Adventure lands (CR 715.3d — the land half is *played* from
//    adventure exile, not cast) ─────────────────────────────────────────────

/// A FIN "Land — Town // Adventure" rare: an enters-tapped mono-color Town
/// whose Adventure half is cast from hand, exiling the land to be played later.
fn town_adventure(
    name: &'static str,
    color: Color,
    adventure: Adventure,
) -> CardDefinition {
    CardDefinition {
        name,
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Town],
            ..Default::default()
        },
        activated_abilities: vec![super::tap_add(color)],
        static_abilities: vec![super::fin::enters_tapped()],
        adventure: Some(Box::new(adventure)),
        ..Default::default()
    }
}

/// Ishgard, the Holy See // Faith & Grief — {3}{W}{W} Sorcery: return up to two
/// target artifact and/or enchantment cards from your graveyard to your hand.
pub fn ishgard_the_holy_see() -> CardDefinition {
    town_adventure(
        "Ishgard, the Holy See",
        Color::White,
        Adventure {
            name: "Faith & Grief",
            cost: cost(&[generic(3), w(), w()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::InYourGraveyard.and(R::Artifact.or(R::Enchantment)),
                effect: Box::new(Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Hand(PlayerRef::You),
                }),
            },
        },
    )
}

/// Jidoor, Aristocratic Capital // Overture — {4}{U}{U} Sorcery: target opponent
/// mills half their library, rounded down.
pub fn jidoor_aristocratic_capital() -> CardDefinition {
    town_adventure(
        "Jidoor, Aristocratic Capital",
        Color::Blue,
        Adventure {
            name: "Overture",
            cost: cost(&[generic(4), u(), u()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::MillHalf {
                who: target_filtered(R::OpponentPlayer),
                rounded_up: false,
            },
        },
    )
}

/// Lindblum, Industrial Regency // Mage Siege — {2}{R} Instant: create a 0/1
/// black Wizard token that pings each opponent on your noncreature casts.
pub fn lindblum_industrial_regency() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Wizard".into(),
        power: 0,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wizard],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    town_adventure(
        "Lindblum, Industrial Regency",
        Color::Red,
        Adventure {
            name: "Mage Siege",
            cost: cost(&[generic(2), r()]),
            card_types: vec![CardType::Instant],
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: wizard,
            },
        },
    )
}

/// Midgar, City of Mako // Reactor Raid — {2}{B} Sorcery: you may sacrifice an
/// artifact or creature. If you do, draw two cards.
pub fn midgar_city_of_mako() -> CardDefinition {
    town_adventure(
        "Midgar, City of Mako",
        Color::Black,
        Adventure {
            name: "Reactor Raid",
            cost: cost(&[generic(2), b()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::MaySacrifice {
                description: "Sacrifice an artifact or creature?".into(),
                filter: R::Artifact.or(R::Creature),
                count: Value::ONE,
                then: Box::new(draw(2)),
                else_: None,
            },
        },
    )
}

/// Zanarkand, Ancient Metropolis // Lasting Fayth — {4}{G}{G} Sorcery: create a
/// 1/1 colorless Hero token with a +1/+1 counter for each land you control.
pub fn zanarkand_ancient_metropolis() -> CardDefinition {
    town_adventure(
        "Zanarkand, Ancient Metropolis",
        Color::Green,
        Adventure {
            name: "Lasting Fayth",
            cost: cost(&[generic(4), g(), g()]),
            card_types: vec![CardType::Sorcery],
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: TokenDefinition {
                        name: "Hero".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Hero],
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::PermanentCountControlledByMatching(PlayerRef::You, R::Land),
                },
            ]),
        },
    )
}

/// Balamb Garden, SeeD Academy // Balamb Garden, Airborne — a Town that taps for
/// {G}/{U} and transforms into a 5/4 flying Vehicle; the transform cost drops
/// {1} per other Town you control.
pub fn balamb_garden_seed_academy() -> CardDefinition {
    let airborne = CardDefinition {
        name: "Balamb Garden, Airborne",
        card_types: vec![CardType::Artifact],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            artifact_subtypes: vec![ArtifactSubtype::Vehicle],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: draw(1),
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Balamb Garden, SeeD Academy",
        card_types: vec![CardType::Land],
        subtypes: Subtypes {
            land_types: vec![LandType::Town],
            ..Default::default()
        },
        activated_abilities: vec![
            super::tap_add(Color::Green),
            super::tap_add(Color::Blue),
            ActivatedAbility {
                mana_cost: cost(&[generic(5), g(), u()]),
                tap_cost: true,
                cost_reduction_per: Some(
                    R::HasLandType(LandType::Town).and(R::OtherThanSource),
                ),
                effect: Effect::Transform { what: Selector::This },
                ..Default::default()
            },
        ],
        static_abilities: vec![super::fin::enters_tapped()],
        back_face: Some(Box::new(airborne)),
        ..Default::default()
    }
}

// ── Sidequest transforming enchantments ─────────────────────────────────────

/// "At the beginning of `step`, if `cond`, transform this enchantment."
fn transform_when(step: TurnStep, cond: Predicate, before: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(step), EventScope::YourControl)
            .with_filter(cond),
        effect: Effect::Seq(vec![before, Effect::Transform { what: Selector::This }]),
    }
}

fn sidequest(
    name: &'static str,
    mana: crate::mana::ManaCost,
    triggers: Vec<TriggeredAbility>,
    back: CardDefinition,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Enchantment],
        triggered_abilities: triggers,
        back_face: Some(Box::new(back)),
        ..Default::default()
    }
}

/// Sidequest: Card Collection // Magicked Card — {3}{U}. ETB draw three, discard
/// two; transforms at your end step with eight+ cards in your graveyard into a
/// 4/4 flying Vehicle with crew 1.
pub fn sidequest_card_collection() -> CardDefinition {
    sidequest(
        "Sidequest: Card Collection",
        cost(&[generic(3), u()]),
        vec![
            etb(Effect::Seq(vec![
                draw(3),
                discard(Selector::You, 2, false),
            ])),
            transform_when(
                TurnStep::End,
                Predicate::ValueAtLeast(
                    Value::GraveyardSizeOf(PlayerRef::You),
                    Value::Const(8),
                ),
                Effect::Noop,
            ),
        ],
        CardDefinition {
            name: "Magicked Card",
            card_types: vec![CardType::Artifact],
            subtypes: Subtypes {
                artifact_subtypes: vec![ArtifactSubtype::Vehicle],
                ..Default::default()
            },
            power: 4,
            toughness: 4,
            keywords: vec![Keyword::Flying, Keyword::Crew(1)],
            ..Default::default()
        },
    )
}

/// Sidequest: Catch a Fish // Cooking Campsite — {2}{W}. Upkeep: look at the top
/// card; if it's an artifact or creature you may take it, then make a Food and
/// transform into a land that pumps the team.
pub fn sidequest_catch_a_fish() -> CardDefinition {
    sidequest(
        "Sidequest: Catch a Fish",
        cost(&[generic(2), w()]),
        vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::ONE,
                pick_filter: Some(R::Artifact.or(R::Creature)),
                optional: true,
                then_if_picked: Some(Box::new(Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: crabomination_base::tokens::food_token(),
                    },
                    Effect::Transform { what: Selector::This },
                ]))),
                rest_to_graveyard: false,
                take: None,
                to_battlefield: false,
                gain_life_if_pick: None,
                gain_life_greatest_power_rest: false,
                picked_lands_to_battlefield: false,
                rest_bottom_random: false,
                rest_to_exile: false,
            },
        }],
        CardDefinition {
            name: "Cooking Campsite",
            card_types: vec![CardType::Land],
            activated_abilities: vec![
                super::tap_add(Color::White),
                ActivatedAbility {
                    mana_cost: cost(&[generic(3)]),
                    tap_cost: true,
                    sac_other_filter: Some((R::Artifact, 1)),
                    sorcery_speed: true,
                    effect: Effect::AddCounter {
                        what: Selector::ControlledBy { who: PlayerRef::You, filter: R::Creature },
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::ONE,
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
    )
}

/// Sidequest: Hunt the Mark // Yiazmat, Ultimate Mark — {3}{B}{B}. ETB destroy up
/// to one creature; at your end step make a Treasure if an opponent's creature
/// died, then transform on three Treasures into a 5/6 Dragon.
pub fn sidequest_hunt_the_mark() -> CardDefinition {
    let treasures_at_least_three = Predicate::ValueAtLeast(
        Value::PermanentCountControlledByMatching(
            PlayerRef::You,
            R::HasArtifactSubtype(ArtifactSubtype::Treasure),
        ),
        Value::Const(3),
    );
    sidequest(
        "Sidequest: Hunt the Mark",
        cost(&[generic(3), b(), b()]),
        vec![
            etb(Effect::OptionalTargets {
                min: 0,
                body: Box::new(Effect::Destroy { what: target_filtered(R::Creature) }),
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::CreatureDiedThisTurnMatching {
                    filter: R::ControlledByOpponent,
                }),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::ONE,
                        definition: crabomination_base::tokens::treasure_token(),
                    },
                    Effect::If {
                        cond: treasures_at_least_three,
                        then: Box::new(Effect::Transform { what: Selector::This }),
                        else_: Box::new(Effect::Noop),
                    },
                ]),
            },
        ],
        CardDefinition {
            name: "Yiazmat, Ultimate Mark",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Dragon],
                ..Default::default()
            },
            power: 5,
            toughness: 6,
            activated_abilities: vec![ActivatedAbility {
                mana_cost: cost(&[generic(1), b()]),
                sac_other_filter: Some((R::Creature.or(R::Artifact), 1)),
                effect: Effect::Seq(vec![
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Indestructible,
                        duration: Duration::EndOfTurn,
                    },
                    Effect::Tap { what: Selector::This },
                ]),
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

/// Sidequest: Play Blitzball // World Champion, Celestial Weapon — {2}{R}. Pumps
/// an attacker each combat; transforms into an Equipment (+2/+0, double strike)
/// and attaches once a player took 6+ combat damage this turn.
pub fn sidequest_play_blitzball() -> CardDefinition {
    sidequest(
        "Sidequest: Play Blitzball",
        cost(&[generic(2), r()]),
        vec![
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::YourControl,
                ),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByYou)),
                    power: Value::Const(2),
                    toughness: Value::ZERO,
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::EndCombat),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::CombatDamageTakenThisTurn(PlayerRef::EachPlayer),
                    Value::Const(6),
                )),
                effect: Effect::Seq(vec![
                    Effect::Transform { what: Selector::This },
                    Effect::AttachSourceTo {
                        host: Selector::GreatestPowerYouControl,
                    },
                ]),
            },
        ],
        CardDefinition {
            name: "World Champion, Celestial Weapon",
            card_types: vec![CardType::Artifact],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                artifact_subtypes: vec![ArtifactSubtype::Equipment],
                ..Default::default()
            },
            keywords: vec![Keyword::Equip(cost(&[generic(3)]))],
            equipped_bonus: Some(EquipBonus {
                power: 2,
                toughness: 0,
                keywords: vec![Keyword::DoubleStrike],
                ..Default::default()
            }),
            ..Default::default()
        },
    )
}

/// Sidequest: Raise a Chocobo // Black Chocobo — {1}{G}. ETB makes a 2/2 Bird
/// that grows on landfall; transforms at your main phase with four+ Birds into a
/// Bird that fetches a land and pumps the flock on landfall.
pub fn sidequest_raise_a_chocobo() -> CardDefinition {
    let bird = TokenDefinition {
        name: "Bird".into(),
        power: 2,
        toughness: 2,
        colors: vec![Color::Green],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    };
    sidequest(
        "Sidequest: Raise a Chocobo",
        cost(&[generic(1), g()]),
        vec![
            etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: bird,
            }),
            transform_when(
                TurnStep::PreCombatMain,
                Predicate::ValueAtLeast(
                    Value::PermanentCountControlledByMatching(
                        PlayerRef::You,
                        R::HasCreatureType(CreatureType::Bird),
                    ),
                    Value::Const(4),
                ),
                Effect::Noop,
            ),
        ],
        CardDefinition {
            name: "Black Chocobo",
            card_types: vec![CardType::Creature],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Bird],
                ..Default::default()
            },
            power: 2,
            toughness: 2,
            triggered_abilities: vec![
                TriggeredAbility {
                    event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                    effect: Effect::Search {
                        who: PlayerRef::You,
                        filter: R::Land,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: true,
                        },
                    },
                },
                TriggeredAbility {
                    event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                    effect: Effect::PumpPT {
                        what: Selector::ControlledBy {
                            who: PlayerRef::You,
                            filter: R::HasCreatureType(CreatureType::Bird),
                        },
                        power: Value::ONE,
                        toughness: Value::ZERO,
                        duration: Duration::EndOfTurn,
                    },
                },
            ],
            ..Default::default()
        },
    )
}

// ── Transforming legends ────────────────────────────────────────────────────

/// Exdeath, Void Warlock // Neo Exdeath, Dimension's End — {1}{B}{G} 3/3 that
/// gains 3 life on entry and flips at your end step on six permanent cards in
/// your graveyard into a trampling */3 sized by that pile.
pub fn exdeath_void_warlock() -> CardDefinition {
    CardDefinition {
        name: "Exdeath, Void Warlock",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
            transform_when(
                TurnStep::End,
                Predicate::ValueAtLeast(
                    Value::CardsInGraveyardMatching {
                        who: PlayerRef::You,
                        filter: R::PermanentCard,
                    },
                    Value::Const(6),
                ),
                Effect::Noop,
            ),
        ],
        back_face: Some(Box::new(CardDefinition {
            name: "Neo Exdeath, Dimension's End",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Spirit, CreatureType::Avatar],
                ..Default::default()
            },
            power: 0,
            toughness: 3,
            keywords: vec![Keyword::Trample],
            dynamic_pt: Some(DynamicPt::PermanentCardsInControllerGraveyard {
                base_p: 0,
                base_t: 3,
            }),
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Emet-Selch, Unsundered // Hades, Sorcerer of Eld — {1}{U}{B} 2/4 looter that
/// flips on a fourteen-card graveyard into a 6/6 that plays out of the
/// graveyard and exiles everything bound for it.
pub fn emet_selch_unsundered() -> CardDefinition {
    CardDefinition {
        name: "Emet-Selch, Unsundered",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            etb(Effect::Seq(vec![draw(1), discard(Selector::You, 1, false)])),
            on_attack(Effect::Seq(vec![draw(1), discard(Selector::You, 1, false)])),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                )
                .with_filter(Predicate::ValueAtLeast(
                    Value::GraveyardSizeOf(PlayerRef::You),
                    Value::Const(14),
                )),
                effect: Effect::MayDo {
                    description: "Transform Emet-Selch?".into(),
                    body: Box::new(Effect::Transform { what: Selector::This }),
                },
            },
        ],
        back_face: Some(Box::new(CardDefinition {
            name: "Hades, Sorcerer of Eld",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Avatar],
                ..Default::default()
            },
            power: 6,
            toughness: 6,
            keywords: vec![Keyword::Vigilance],
            static_abilities: vec![
                StaticAbility {
                    description: "During your turn, you may play cards from your graveyard.",
                    effect: StaticEffect::PlayCardsFromGraveyardDuringYourTurn,
                },
                StaticAbility {
                    description: "If a card or token would be put into your graveyard from \
                                  anywhere, exile it instead.",
                    effect: StaticEffect::ExileCardsBoundForGraveyard {
                        opponents_only: false,
                        own_only: true,
                        colors: None,
                        card_types: None,
                        void_counter: false,
                        stamp_source: false,
                    },
                },
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Kuja, Genome Sorcerer // Trance Kuja, Fate Defied — {2}{B}{R} 3/4 that mints
/// a pinging Wizard each end step and flips on four Wizards into a 4/6 that
/// doubles your Wizards' damage.
pub fn kuja_genome_sorcerer() -> CardDefinition {
    let wizard = TokenDefinition {
        name: "Wizard".into(),
        power: 0,
        toughness: 1,
        colors: vec![Color::Black],
        card_types: vec![CardType::Creature],
        tapped: true,
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wizard],
            ..Default::default()
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    };
    CardDefinition {
        name: "Kuja, Genome Sorcerer",
        cost: cost(&[generic(2), b(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Mutant,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    definition: wizard,
                },
                Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::PermanentCountControlledByMatching(
                            PlayerRef::You,
                            R::HasCreatureType(CreatureType::Wizard),
                        ),
                        Value::Const(4),
                    ),
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(CardDefinition {
            name: "Trance Kuja, Fate Defied",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Avatar, CreatureType::Wizard],
                ..Default::default()
            },
            power: 4,
            toughness: 6,
            static_abilities: vec![StaticAbility {
                description: "Flare Star — If a Wizard you control would deal damage to a \
                              permanent or player, it deals double that damage instead.",
                effect: StaticEffect::DoubleDamageFromControlledMatching {
                    filter: R::HasCreatureType(CreatureType::Wizard),
                },
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// The Emperor of Palamecia // The Lord Master of Hell — {U}{R} 2/2 that taps
/// for noncreature-only mana and grows on big noncreature casts, flipping at
/// three counters into a 3/3 that burns for your graveyard's spell count.
pub fn the_emperor_of_palamecia() -> CardDefinition {
    CardDefinition {
        name: "The Emperor of Palamecia",
        cost: cost(&[u(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: crate::effect::ManaPayload::Restricted(
                    Box::new(crate::effect::ManaPayload::OfColors(
                        vec![Color::Blue, Color::Red],
                        Value::ONE,
                    )),
                    crate::mana::SpendRestriction::NoncreatureSpellsOnly,
                ),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::All(vec![
                    Predicate::CastSpellMatches(R::Noncreature),
                    Predicate::CastSpellManaSpentAtLeast(4),
                ])),
            effect: Effect::Seq(vec![
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::If {
                    cond: Predicate::SourceHasCountersAtLeast {
                        counter: CounterType::PlusOnePlusOne,
                        n: 3,
                    },
                    then: Box::new(Effect::Transform { what: Selector::This }),
                    else_: Box::new(Effect::Noop),
                },
            ]),
        }],
        back_face: Some(Box::new(CardDefinition {
            name: "The Lord Master of Hell",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![
                    CreatureType::Demon,
                    CreatureType::Noble,
                    CreatureType::Wizard,
                ],
                ..Default::default()
            },
            power: 3,
            toughness: 3,
            triggered_abilities: vec![on_attack(Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::CardsInGraveyardMatching {
                    who: PlayerRef::You,
                    filter: R::Noncreature.and(R::Nonland),
                },
            })],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Vincent Valentine // Galian Beast — {2}{B}{B} 2/2 that eats the power of
/// dying opponent creatures and may flip when it attacks into a trampling
/// lifelinker that comes back tapped, front face up.
pub fn vincent_valentine() -> CardDefinition {
    CardDefinition {
        name: "Vincent Valentine",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::PowerOf(Box::new(Selector::TriggerSource)),
                },
            },
            on_attack(Effect::MayDo {
                description: "Transform Vincent Valentine?".into(),
                body: Box::new(Effect::Transform { what: Selector::This }),
            }),
        ],
        back_face: Some(Box::new(CardDefinition {
            name: "Galian Beast",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Werewolf, CreatureType::Beast],
                ..Default::default()
            },
            power: 3,
            toughness: 2,
            keywords: vec![Keyword::Trample, Keyword::Lifelink],
            triggered_abilities: vec![on_dies(Effect::ReturnSelfTapped)],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Ultimecia, Time Sorceress // Ultimecia, Omnipotent — {3}{U}{B} 4/5 surveiller
/// that buys its flip at your end step with {4}{U}{U}{B}{B} and eight graveyard
/// cards, becoming a 7/7 menace that takes an extra turn.
pub fn ultimecia_time_sorceress() -> CardDefinition {
    CardDefinition {
        name: "Ultimecia, Time Sorceress",
        cost: cost(&[generic(3), u(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        triggered_abilities: vec![
            etb(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
            on_attack(Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::YourControl,
                ),
                effect: Effect::MayPay {
                    description: "Pay {4}{U}{U}{B}{B} and exile eight graveyard cards?".into(),
                    mana_cost: cost(&[generic(4), u(), u(), b(), b()]),
                    body: Box::new(Effect::Seq(vec![
                        Effect::Move {
                            what: Selector::Take {
                                inner: Box::new(Selector::CardsInZone {
                                    who: PlayerRef::You,
                                    zone: crate::card::Zone::Graveyard,
                                    filter: R::Any,
                                }),
                                count: Box::new(Value::Const(8)),
                            },
                            to: ZoneDest::Exile,
                        },
                        Effect::Transform { what: Selector::This },
                    ])),
                    else_: None,
                },
            },
        ],
        back_face: Some(Box::new(CardDefinition {
            name: "Ultimecia, Omnipotent",
            card_types: vec![CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Nightmare, CreatureType::Warlock],
                ..Default::default()
            },
            power: 7,
            toughness: 7,
            keywords: vec![Keyword::Menace],
            triggered_abilities: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::Transformed, EventScope::SelfSource),
                effect: Effect::TakeExtraTurn { who: PlayerRef::You, count: Value::ONE },
            }],
            ..Default::default()
        })),
        ..Default::default()
    }
}

// ── The "Dominant" cycle (Legendary Creature // Saga creature) ───────────────

/// The shared front-face flip line: "{cost}, {T}: Exile this, then return it to
/// the battlefield transformed under its owner's control. Sorcery only."
fn dominant_flip(mana: crate::mana::ManaCost) -> ActivatedAbility {
    ActivatedAbility {
        mana_cost: mana,
        tap_cost: true,
        sorcery_speed: true,
        effect: Effect::ExileSelfReturnTransformed,
        ..Default::default()
    }
}

/// Clive, Ifrit's Dominant // Ifrit, Warden of Inferno — {4}{R}{R} 5/5 that may
/// refill off red devotion; the 9/9 Saga back fights, then rituals for {R}{R}{R}{R}
/// and resets itself once it has three lore counters.
pub fn clive_ifrits_dominant() -> CardDefinition {
    let brimstone = Effect::Seq(vec![
        Effect::AddMana {
            who: PlayerRef::You,
            pool: crate::effect::ManaPayload::OfColor(Color::Red, Value::Const(4)),
        },
        Effect::If {
            cond: Predicate::SourceHasCountersAtLeast { counter: CounterType::Lore, n: 3 },
            then: Box::new(Effect::ExileSelfReturnFrontFace),
            else_: Box::new(Effect::Noop),
        },
    ]);
    CardDefinition {
        name: "Clive, Ifrit's Dominant",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![etb(Effect::MayDo {
            description: "Discard your hand and draw that many?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::DiscardHandDrawThatMany { who: Selector::You },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::DevotionTo(vec![Color::Red]),
                },
            ])),
        })],
        activated_abilities: vec![dominant_flip(cost(&[generic(4), r(), r()]))],
        back_face: Some(Box::new(CardDefinition {
            name: "Ifrit, Warden of Inferno",
            card_types: vec![CardType::Enchantment, CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Demon],
                enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
                ..Default::default()
            },
            power: 9,
            toughness: 9,
            saga_chapters: vec![
                (
                    1,
                    Effect::Fight {
                        attacker: Selector::This,
                        defender: target_filtered(R::Creature.and(R::OtherThanSource)),
                    },
                ),
                (2, brimstone.clone()),
                (3, brimstone),
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Dion, Bahamut's Dominant // Bahamut, Warden of Light — {3}{W} 3/3 Knight lord
/// whose 5/5 Saga back pumps the team, then destroys a permanent and resets.
pub fn dion_bahamuts_dominant() -> CardDefinition {
    let wings = Effect::Seq(vec![
        Effect::AddCounter {
            what: Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::OtherThanSource),
            },
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        },
        Effect::GrantKeyword {
            what: Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::OtherThanSource),
            },
            keyword: Keyword::Flying,
            duration: Duration::EndOfTurn,
        },
    ]);
    CardDefinition {
        name: "Dion, Bahamut's Dominant",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Knight,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Dragonfire Dive — During your turn, Dion and other Knights you \
                          control have flying.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::HasCreatureType(CreatureType::Knight).and(R::ControlledByYou),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Flying],
                opponents: false,
                all_players: false,
                only_your_turn: true,
                scale_by_counters_on_self: None,
            },
        }],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Knight".into(),
                power: 2,
                toughness: 2,
                colors: vec![Color::White],
                card_types: vec![CardType::Creature],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Knight],
                    ..Default::default()
                },
                ..Default::default()
            },
        })],
        activated_abilities: vec![dominant_flip(cost(&[generic(4), w(), w()]))],
        back_face: Some(Box::new(CardDefinition {
            name: "Bahamut, Warden of Light",
            card_types: vec![CardType::Enchantment, CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Dragon],
                enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
                ..Default::default()
            },
            power: 5,
            toughness: 5,
            keywords: vec![Keyword::Flying],
            saga_chapters: vec![
                (1, wings.clone()),
                (2, wings),
                (
                    3,
                    Effect::Seq(vec![
                        Effect::Destroy { what: target_filtered(R::Permanent) },
                        Effect::ExileSelfReturnFrontFace,
                    ]),
                ),
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Jill, Shiva's Dominant // Shiva, Warden of Ice — {2}{U} 2/2 bouncer whose
/// 4/5 Saga back unblocks a creature twice, then freezes opposing lands.
pub fn jill_shivas_dominant() -> CardDefinition {
    let mesmerize = Effect::GrantKeyword {
        what: target_filtered(R::Creature),
        keyword: Keyword::Unblockable,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Jill, Shiva's Dominant",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Warrior,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::OptionalTargets {
            min: 0,
            body: Box::new(Effect::Move {
                what: target_filtered(R::Nonland.and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
        })],
        activated_abilities: vec![dominant_flip(cost(&[generic(3), u(), u()]))],
        back_face: Some(Box::new(CardDefinition {
            name: "Shiva, Warden of Ice",
            card_types: vec![CardType::Enchantment, CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Elemental],
                enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
                ..Default::default()
            },
            power: 4,
            toughness: 5,
            saga_chapters: vec![
                (1, mesmerize.clone()),
                (2, mesmerize),
                (
                    3,
                    Effect::Seq(vec![
                        Effect::Tap {
                            what: Selector::ControlledBy {
                                who: PlayerRef::EachOpponent,
                                filter: R::Land,
                            },
                        },
                        Effect::ExileSelfReturnFrontFace,
                    ]),
                ),
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Joshua, Phoenix's Dominant // Phoenix, Warden of Fire — {1}{R}{W} 3/4 looter
/// whose 4/4 flying lifelinking Saga back burns opponents twice, then resets.
pub fn joshua_phoenixs_dominant() -> CardDefinition {
    let rising_flames = Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount: Value::Const(2),
    };
    CardDefinition {
        name: "Joshua, Phoenix's Dominant",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Noble,
                CreatureType::Wizard,
            ],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        // "Discard up to two, then draw that many" — modeled as the mandatory
        // two-for-two loot; declining a smaller discard isn't expressible yet.
        triggered_abilities: vec![etb(Effect::Seq(vec![
            discard(Selector::You, 2, false),
            draw(2),
        ]))],
        activated_abilities: vec![dominant_flip(cost(&[generic(3), r(), w()]))],
        back_face: Some(Box::new(CardDefinition {
            name: "Phoenix, Warden of Fire",
            card_types: vec![CardType::Enchantment, CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Phoenix],
                enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
                ..Default::default()
            },
            power: 4,
            toughness: 4,
            keywords: vec![Keyword::Flying, Keyword::Lifelink],
            saga_chapters: vec![
                (1, rising_flames.clone()),
                (2, rising_flames),
                (
                    3,
                    Effect::Seq(vec![
                        Effect::ReturnGraveyardCreaturesUpToTotalManaValue {
                            max_total: Value::Const(6),
                            max_count: Value::Const(99),
                            counters: 0,
                        },
                        Effect::ExileSelfReturnFrontFace,
                    ]),
                ),
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}

/// Jecht, Reluctant Guardian // Braska's Final Aeon — {3}{B} 4/3 menace that may
/// flip on combat damage into a 7/7 Saga that strips hands, then edicts twice.
pub fn jecht_reluctant_guardian() -> CardDefinition {
    let jecht_beam = Effect::Seq(vec![
        Effect::Discard {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::ONE,
            random: false,
        },
        draw(1),
    ]);
    CardDefinition {
        name: "Jecht, Reluctant Guardian",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Transform Jecht?".into(),
                body: Box::new(Effect::ExileSelfReturnTransformed),
            },
        }],
        back_face: Some(Box::new(CardDefinition {
            name: "Braska's Final Aeon",
            card_types: vec![CardType::Enchantment, CardType::Creature],
            supertypes: vec![Supertype::Legendary],
            subtypes: Subtypes {
                creature_types: vec![CreatureType::Nightmare],
                enchantment_subtypes: vec![crate::card::EnchantmentSubtype::Saga],
                ..Default::default()
            },
            power: 7,
            toughness: 7,
            keywords: vec![Keyword::Menace],
            saga_chapters: vec![
                (1, jecht_beam.clone()),
                (2, jecht_beam),
                (
                    3,
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        count: Value::Const(2),
                        filter: R::Creature,
                    },
                ),
            ],
            ..Default::default()
        })),
        ..Default::default()
    }
}
