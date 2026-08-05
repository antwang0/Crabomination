//! Final Fantasy (FIN) gap closure: the Town // Adventure land cycle and the
//! Sidequest transforming enchantments. Tests in `classic_sets/fin2`.

use crate::card::{
    ActivatedAbility, Adventure, ArtifactSubtype, CardDefinition, CardType, CounterType,
    CreatureType, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    LandType, Predicate, SelectionRequirement as R, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility,
};
use crate::effect::{
    Duration, Effect, PlayerRef, Selector, Value, ZoneDest,
    shortcut::{discard, draw, etb, target_filtered},
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
            name: "Faith & Grief".into(),
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
            name: "Overture".into(),
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
            name: "Mage Siege".into(),
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
            name: "Reactor Raid".into(),
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
            name: "Lasting Fayth".into(),
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
