//! Aetherdrift (DFT) gap batch: batched-discard burn (Magmakin), a max-speed
//! "each player without max speed" enchantment (Outpace Oblivion), an
//! attacked-by debuff (Sabotage Strategist), a copy-an-artifact-or-creature
//! Shapeshifter (Waxen Shapethief), mill+conditional-destroy (Quag Feast), a
//! modal fight/destroy (Plow Through), a blink+board-wipe (Explosive Getaway),
//! a Start-your-engines Aura (Lightwheel Enhancements), a second-draw Thopter
//! maker (Thopter Fabricator), a temporary-reanimator (Coalstoke Gearhulk), a
//! team base-6/6-Ooze anthem (March of the World Ooze), a theft Vehicle
//! (Possession Engine), a coercive discard-then-draw (Oildeep Gearhulk), a
//! sac-to-tutor-MV+1 artifact (Repurposing Bay), a graveyard-hate Construct
//! (Wreck Remover), a saddled Mount (Lagorin, Soul of Alacria), a
//! mill-each-opponent artifact (Riverchurn Monument), and a tap-down/lose-
//! abilities Aura (Flood the Engine). Tests in
//! `crabomination/src/tests/recent175.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EnchantmentSubtype, EntersAsCopy, EquipBonus, EventKind, EventScope, EventSpec, Keyword,
    Predicate, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype,
    TokenDefinition, TriggeredAbility, Value, WardCost,
};
use crate::effect::shortcut::{etb, target_filtered};
use crate::effect::{Duration, Effect, PlayerRef, Selector, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w, Color};

/// Magmakin Artillerist — {2}{R} 1/4 Elemental Pirate. Whenever you discard one
/// or more cards, deal that much damage to each opponent. Cycling {1}{R}. When
/// you cycle this card, it deals 1 damage to each opponent.
pub fn magmakin_artillerist() -> CardDefinition {
    let bolt_opponents = |amount: Value| Effect::DealDamage {
        to: Selector::Player(PlayerRef::EachOpponent),
        amount,
    };
    CardDefinition {
        name: "Magmakin Artillerist",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[generic(1), r()]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::DiscardedOneOrMore, EventScope::YourControl),
                effect: bolt_opponents(Value::TriggerEventAmount),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: bolt_opponents(Value::ONE),
            },
        ],
        ..Default::default()
    }
}

/// Outpace Oblivion — {2}{R} Enchantment. Start your engines! ETB: deal 5 damage
/// to up to one target creature or planeswalker. {2}, Sacrifice this: deal 2
/// damage to each player who doesn't have max speed.
pub fn outpace_oblivion() -> CardDefinition {
    CardDefinition {
        name: "Outpace Oblivion",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::StartYourEngines],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::Creature.or(R::Planeswalker),
            effect: Box::new(Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(5) }),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachPlayerWithoutMaxSpeed),
                amount: Value::Const(2),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sabotage Strategist — {2}{U}{U} 2/2 Vedalken Ranger. Flying, vigilance.
/// Whenever one or more creatures attack you, those creatures get -1/-0 until
/// end of turn. Exhaust — {5}{U}{U}: put three +1/+1 counters on this.
/// (Attacks on a planeswalker you control also fire it — a slight over-fire.)
pub fn sabotage_strategist() -> CardDefinition {
    CardDefinition {
        name: "Sabotage Strategist",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Ranger],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::PumpPT {
                what: Selector::TriggerSource,
                power: Value::Const(-1),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(5), u(), u()]),
            exhaust: true,
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Waxen Shapethief — {3}{U} 0/0 Shapeshifter with flash. Enters as a copy of an
/// artifact or creature you control (may). Cycling {2}.
pub fn waxen_shapethief() -> CardDefinition {
    CardDefinition {
        name: "Waxen Shapethief",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Shapeshifter], ..Default::default() },
        keywords: vec![Keyword::Flash, Keyword::Cycling(cost(&[generic(2)]))],
        enters_as_copy: Some(EntersAsCopy {
            filter: R::Creature.or(R::Artifact).and(R::ControlledByYou),
            extra_creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Quag Feast — {1}{B} Sorcery. Choose target creature, planeswalker, or
/// Vehicle. Mill two, then destroy it if its mana value ≤ cards in your
/// graveyard.
pub fn quag_feast() -> CardDefinition {
    let target = Selector::TargetFiltered {
        slot: 0,
        filter: R::Creature.or(R::Planeswalker).or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
    };
    CardDefinition {
        name: "Quag Feast",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Mill { who: Selector::You, amount: Value::Const(2) },
            Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(target.clone())),
                    Value::GraveyardSizeOf(PlayerRef::You),
                ),
                then: Box::new(Effect::Destroy { what: target.clone() }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Plow Through — {G} Sorcery. Choose one — your creature fights an opponent's
/// creature; or destroy target Vehicle.
pub fn plow_through() -> CardDefinition {
    CardDefinition {
        name: "Plow Through",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ChooseMode(vec![
            Effect::Fight {
                attacker: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::Creature.and(R::ControlledByYou),
                },
                defender: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.and(R::ControlledByOpponent),
                },
            },
            Effect::Destroy {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::HasArtifactSubtype(ArtifactSubtype::Vehicle),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Explosive Getaway — {3}{R}{W} Sorcery. Exile up to one target artifact or
/// creature, returning it at the next end step; deal 4 damage to each creature.
pub fn explosive_getaway() -> CardDefinition {
    CardDefinition {
        name: "Explosive Getaway",
        cost: cost(&[generic(3), r(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::ApplyToTargets {
                max_targets: 1,
                min_targets: 0,
                filter: R::Artifact.or(R::Creature),
                effect: Box::new(Effect::ExileReturnNextEndStep { what: Selector::Target(0) }),
            },
            Effect::ForEach {
                selector: Selector::EachPermanent(R::Creature),
                body: Box::new(Effect::DealDamage { to: Selector::TriggerSource, amount: Value::Const(4) }),
            },
        ]),
        ..Default::default()
    }
}

/// Lightwheel Enhancements — {W} Aura. Start your engines! Enchant creature or
/// Vehicle; it gets +1/+1 and has vigilance. (Max-speed graveyard-cast dropped.)
pub fn lightwheel_enhancements() -> CardDefinition {
    CardDefinition {
        name: "Lightwheel Enhancements",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        keywords: vec![Keyword::StartYourEngines],
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
            },
        },
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            keywords: vec![Keyword::Vigilance],
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Thopter Fabricator — {2}{U} Artifact — Vehicle 4/4. Flying. Whenever you draw
/// your second card each turn, create a 1/1 colorless Thopter artifact creature
/// token with flying. Crew 2.
pub fn thopter_fabricator() -> CardDefinition {
    let thopter = TokenDefinition {
        name: "Thopter".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Thopter], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    };
    CardDefinition {
        name: "Thopter Fabricator",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Crew(2)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: thopter },
        }],
        ..Default::default()
    }
}

/// Coalstoke Gearhulk — {1}{B}{B}{R}{R} 5/4 Construct. Menace, deathtouch. ETB:
/// put target creature card with mana value 4 or less from a graveyard onto the
/// battlefield under your control with a finality counter; it gains menace,
/// deathtouch, and haste. Exile it at the beginning of your next end step.
pub fn coalstoke_gearhulk() -> CardDefinition {
    let grant = |kw: Keyword| Effect::GrantKeyword {
        what: Selector::Target(0),
        keyword: kw,
        duration: Duration::Permanent,
    };
    CardDefinition {
        name: "Coalstoke Gearhulk",
        cost: cost(&[generic(1), b(), b(), r(), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Menace, Keyword::Deathtouch],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(R::Creature.and(R::InGraveyard).and(R::ManaValueAtMost(4))),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::Finality,
                amount: Value::ONE,
            },
            grant(Keyword::Menace),
            grant(Keyword::Deathtouch),
            grant(Keyword::Haste),
            Effect::AtNextEndStep { body: Box::new(Effect::Exile { what: Selector::Target(0) }) },
        ]))],
        ..Default::default()
    }
}

/// March of the World Ooze — {3}{G}{G}{G} Enchantment. Creatures you control
/// have base power and toughness 6/6 and are Oozes in addition to their other
/// types. Whenever an opponent casts a spell during a turn that isn't theirs,
/// create a 3/3 green Elephant.
pub fn march_of_the_world_ooze() -> CardDefinition {
    let mine = || R::Creature.and(R::ControlledByYou);
    let elephant = TokenDefinition {
        name: "Elephant".into(),
        power: 3,
        toughness: 3,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elephant], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "March of the World Ooze",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control have base power and toughness 6/6.",
                effect: StaticEffect::SetBasePtForFilter {
                    applies_to: Selector::EachPermanent(mine()),
                    power: 6,
                    toughness: 6,
                },
            },
            StaticAbility {
                description: "Creatures you control are Oozes in addition to their other types.",
                effect: StaticEffect::AddCreatureTypeToMatching {
                    applies_to: Selector::EachPermanent(mine()),
                    creature_type: CreatureType::Ooze,
                },
            },
        ],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::OpponentControl)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::Triggerer)))),
            effect: Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: elephant },
        }],
        ..Default::default()
    }
}

/// Possession Engine — {3}{U}{U} Artifact — Vehicle 5/5. Crew 3. ETB: gain
/// control of target creature an opponent controls for as long as you control
/// this Vehicle. (The "that creature can't attack or block" rider is dropped —
/// no source-linked keyword duration yet.)
pub fn possession_engine() -> CardDefinition {
    CardDefinition {
        name: "Possession Engine",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Vehicle], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Crew(3)],
        triggered_abilities: vec![etb(Effect::GainControlWhileSourceRemains {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
        })],
        ..Default::default()
    }
}

/// Oildeep Gearhulk — {U}{U}{B}{B} 4/4 Construct. Lifelink, ward {1}. ETB: look
/// at an opponent's hand, choose a card; they discard it, then draw a card.
/// (ETB player-targeting uses the opponent per engine convention.)
pub fn oildeep_gearhulk() -> CardDefinition {
    CardDefinition {
        name: "Oildeep Gearhulk",
        cost: cost(&[u(), u(), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Lifelink, Keyword::Ward(WardCost::Mana(cost(&[generic(1)])))],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::ONE,
                filter: R::Any,
            },
            Effect::Draw { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::ONE },
        ]))],
        ..Default::default()
    }
}

/// Repurposing Bay — {2}{U} Artifact. {2}, {T}, Sacrifice another artifact:
/// Search your library for an artifact card with mana value equal to 1 plus the
/// sacrificed artifact's mana value, put it onto the battlefield, then shuffle.
/// Sorcery speed.
pub fn repurposing_bay() -> CardDefinition {
    CardDefinition {
        name: "Repurposing Bay",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            sorcery_speed: true,
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::OtherThanSource),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Artifact.and(R::ManaValueEqualsSacrificedPlus(1)),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wreck Remover — {4} 3/4 Construct. Whenever this enters or attacks, exile up
/// to one target card from a graveyard and gain 1 life. Cycling {2}.
pub fn wreck_remover() -> CardDefinition {
    let ability = || Effect::Seq(vec![
        Effect::ApplyToTargets {
            max_targets: 1,
            min_targets: 0,
            filter: R::InGraveyard,
            effect: Box::new(Effect::Exile { what: Selector::Target(0) }),
        },
        Effect::GainLife { who: Selector::You, amount: Value::ONE },
    ]);
    CardDefinition {
        name: "Wreck Remover",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: ability(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: ability(),
            },
        ],
        ..Default::default()
    }
}

/// Lagorin, Soul of Alacria — {G}{W} 1/1 Legendary Beast Mount. Flying, Saddle 1.
/// Whenever Lagorin attacks while saddled, put a +1/+1 counter on each of up to
/// two target Mounts and/or Vehicles.
pub fn lagorin_soul_of_alacria() -> CardDefinition {
    CardDefinition {
        name: "Lagorin, Soul of Alacria",
        cost: cost(&[g(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast, CreatureType::Mount],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Saddle(1)],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource)
                .with_filter(Predicate::SourceSaddled),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::HasCreatureType(CreatureType::Mount)
                    .or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle)),
                effect: Box::new(Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Riverchurn Monument — {1}{U} Artifact. {1}, {T}: each opponent mills two.
/// Exhaust — {2}{U}{U}, {T}: each opponent mills cards equal to the number of
/// cards in their graveyard. ("Any number of target players" is modeled as each
/// opponent — the standard aggressive line; faithful in 1v1.)
pub fn riverchurn_monument() -> CardDefinition {
    CardDefinition {
        name: "Riverchurn Monument",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                ..Default::default()
            },
            ActivatedAbility {
                tap_cost: true,
                exhaust: true,
                mana_cost: cost(&[generic(2), u(), u()]),
                effect: Effect::Mill {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::GraveyardSizeOf(PlayerRef::EachOpponent),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Flood the Engine — {2}{U} Aura. Enchant creature or Vehicle. ETB: tap the
/// enchanted permanent. It loses all abilities and doesn't untap during its
/// controller's untap step.
pub fn flood_the_engine() -> CardDefinition {
    CardDefinition {
        name: "Flood the Engine",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { enchantment_subtypes: vec![EnchantmentSubtype::Aura], ..Default::default() },
        effect: Effect::Attach {
            what: Selector::This,
            to: target_filtered(R::Creature.or(R::HasArtifactSubtype(ArtifactSubtype::Vehicle))),
        },
        equipped_bonus: Some(EquipBonus { remove_abilities: true, ..Default::default() }),
        triggered_abilities: vec![etb(Effect::Tap {
            what: Selector::AttachedTo(Box::new(Selector::This)),
        })],
        static_abilities: vec![StaticAbility {
            description: "Enchanted permanent doesn't untap during its controller's untap step.",
            effect: StaticEffect::PreventUntap {
                applies_to: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}
