//! Commander: two-card gaps of EDHREC's commanders ranked 3501-4500 (the
//! 2026-10-14 scan), sixth file. Tests in `tests/recent_b/cmdr_rank4k_f.rs`.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CounterType, CreatureType,
    EventKind, EventScope, EventSpec, Keyword, LandType, MayPlayDuration, Predicate,
    ExileReturnZone, SelectionRequirement as R, StaticAbility, StaticEffect, Subtypes, Supertype, TokenDefinition,
    TriggeredAbility, Value, Zone,
};
use crate::effect::shortcut::{enrage, etb, on_attack, target_filtered};
use crate::effect::{
    Duration, Effect, LibraryPosition, PlayerRef, Selector, ZoneDest,
};
use crate::game::types::TurnStep;
use crate::mana::{Color, ManaCost, b, cost, g, generic, hybrid, r, u, w};
use std::sync::Arc;

fn creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        power: p,
        toughness: t,
        ..Default::default()
    }
}

fn legend(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    p: i32,
    t: i32,
) -> CardDefinition {
    CardDefinition {
        supertypes: vec![Supertype::Legendary],
        ..creature(name, mana, types, p, t)
    }
}

fn spell(name: &'static str, mana: ManaCost, kind: CardType, effect: Effect) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![kind],
        effect,
        ..Default::default()
    }
}

fn yours(filter: R) -> Selector {
    Selector::EachPermanent(filter.and(R::ControlledByYou))
}

fn make(n: Value, def: TokenDefinition) -> Effect {
    Effect::CreateToken {
        who: PlayerRef::You,
        count: n,
        definition: Arc::new(def),
    }
}

fn plus_one(what: Selector) -> Effect {
    Effect::AddCounter {
        what,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::ONE,
    }
}

fn draw(n: i32) -> Effect {
    Effect::Draw {
        who: Selector::You,
        amount: Value::Const(n),
    }
}

fn token(name: &str, p: i32, t: i32, color: Color, types: Vec<CreatureType>) -> TokenDefinition {
    TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![color],
        subtypes: Subtypes {
            creature_types: types,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn on_noncreature_cast(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
            .with_filter(crate::effect::shortcut::cast_is_noncreature()),
        effect,
    }
}

fn your_end_step(effect: Effect) -> TriggeredAbility {
    TriggeredAbility {
        event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl),
        effect,
    }
}

fn recruit() -> Effect {
    Effect::Seq(vec![
        draw(1),
        Effect::Discard {
            who: Selector::You,
            amount: Value::ONE,
            random: false,
        },
        Effect::If {
            cond: Predicate::DiscardedNonlandThisEffect { who: PlayerRef::You },
            then: Box::new(make(
                Value::ONE,
                token(
                    "Human Soldier",
                    1,
                    1,
                    Color::White,
                    vec![CreatureType::Human, CreatureType::Soldier],
                ),
            )),
            else_: Box::new(Effect::Noop),
        },
    ])
}

// ── Veldrane of Sengir ───────────────────────────────────────────────────────

/// Mystic Compass — {2} artifact. {1}, {T}: target land becomes the basic land
/// type of your choice until end of turn.
pub fn mystic_compass() -> CardDefinition {
    let mode = |lt: LandType| Effect::BecomeBasicLand {
        what: target_filtered(R::Land),
        land_type: lt,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::ChooseMode(vec![
                mode(LandType::Plains),
                mode(LandType::Island),
                mode(LandType::Swamp),
                mode(LandType::Mountain),
                mode(LandType::Forest),
            ]),
            ..Default::default()
        }],
        ..spell("Mystic Compass", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

/// Boggart Loggers — {2}{B} 2/1 forestwalk Goblin Rogue. {2}{B}, sacrifice it:
/// destroy target Treefolk or Forest.
pub fn boggart_loggers() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Landwalk(LandType::Forest)],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2), b()]),
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    R::HasCreatureType(CreatureType::Treefolk).or(R::HasLandType(LandType::Forest)),
                ),
            },
            ..Default::default()
        }],
        ..creature(
            "Boggart Loggers",
            cost(&[generic(2), b()]),
            vec![CreatureType::Goblin, CreatureType::Rogue],
            2,
            1,
        )
    }
}

// ── Shocker, Unshakable ──────────────────────────────────────────────────────

/// Shocker, Unshakable — {4}{R}{R} legendary 5/5 Human Rogue Villain. First
/// strike during your turn. ETB: 2 damage to target creature and 2 damage to
/// that creature's controller.
pub fn shocker_unshakable() -> CardDefinition {
    CardDefinition {
        static_abilities: vec![StaticAbility {
            description: "During your turn, Shocker has first strike.",
            effect: StaticEffect::WhileYourTurn {
                inner: Box::new(StaticEffect::GrantKeyword {
                    applies_to: Selector::This,
                    keyword: Keyword::FirstStrike,
                }),
            },
        }],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::DealDamage {
                to: target_filtered(R::Creature),
                amount: Value::Const(2),
            },
            Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
        ]))],
        ..legend(
            "Shocker, Unshakable",
            cost(&[generic(4), r(), r()]),
            vec![CreatureType::Human, CreatureType::Rogue, CreatureType::Villain],
            5,
            5,
        )
    }
}

/// Artifist Acumen — {R} sorcery. Creatures you control gain first strike until
/// end of turn. Draw a card.
pub fn artifist_acumen() -> CardDefinition {
    spell(
        "Artifist Acumen",
        cost(&[r()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::GrantKeyword {
                what: yours(R::Creature),
                keyword: Keyword::FirstStrike,
                duration: Duration::EndOfTurn,
            },
            draw(1),
        ]),
    )
}

// ── Lita, Little Orphan Amphibian ────────────────────────────────────────────

/// Lake-town Lookout — {W} 1/1 Human Scout. Dies: recruit.
pub fn lake_town_lookout() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: recruit(),
        }],
        ..creature(
            "Lake-town Lookout",
            cost(&[w()]),
            vec![CreatureType::Human, CreatureType::Scout],
            1,
            1,
        )
    }
}

/// Mighty Mutanimals — {2}{W}{W} 2/1 Mutant Rebel. ETB: a 2/2 red Mutant.
/// Alliance — whenever another creature you control enters, a +1/+1 counter on
/// target creature you control.
pub fn mighty_mutanimals() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make(
                Value::ONE,
                token("Mutant", 2, 2, Color::Red, vec![CreatureType::Mutant]),
            )),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Creature,
                    }),
                effect: plus_one(target_filtered(R::Creature.and(R::ControlledByYou))),
            },
        ],
        ..creature(
            "Mighty Mutanimals",
            cost(&[generic(2), w(), w()]),
            vec![CreatureType::Mutant, CreatureType::Rebel],
            2,
            1,
        )
    }
}

// ── Ray Fillet, Wave Warrior ─────────────────────────────────────────────────

/// Ravenhill Flock — {3}{U} 1/2 flying Bird. Whenever you draw a card, a +1/+1
/// counter on it.
pub fn ravenhill_flock() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
            effect: plus_one(Selector::This),
        }],
        ..creature(
            "Ravenhill Flock",
            cost(&[generic(3), u()]),
            vec![CreatureType::Bird],
            1,
            2,
        )
    }
}

/// Ray Fillet, Man Ray — {3}{U} legendary 3/3 flying Fish Mutant. ETB: a
/// Mutagen token. {2}, remove a +1/+1 counter from a creature you control:
/// draw a card.
pub fn ray_fillet_man_ray() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(make(
            Value::ONE,
            crabomination_base::tokens::mutagen_token(),
        ))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            remove_counter_among_filter: Some((
                Some(CounterType::PlusOnePlusOne),
                1,
                R::Creature.and(R::ControlledByYou),
            )),
            effect: draw(1),
            ..Default::default()
        }],
        ..legend(
            "Ray Fillet, Man Ray",
            cost(&[generic(3), u()]),
            vec![CreatureType::Fish, CreatureType::Mutant],
            3,
            3,
        )
    }
}

// ── Ur-Drago ─────────────────────────────────────────────────────────────────

/// Dragonborn Looter — {1}{U} 1/2 Dragon Rogue. {1}, {T}: draw, then discard.
pub fn dragonborn_looter() -> CardDefinition {
    CardDefinition {
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1)]),
            tap_cost: true,
            effect: Effect::Seq(vec![
                draw(1),
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
            ]),
            ..Default::default()
        }],
        ..creature(
            "Dragonborn Looter",
            cost(&[generic(1), u()]),
            vec![CreatureType::Dragon, CreatureType::Rogue],
            1,
            2,
        )
    }
}

/// Icefall Regent — {3}{U}{U} 4/3 flying Dragon. ETB: tap target creature an
/// opponent controls; it doesn't untap while you control the Regent. Spells
/// your opponents cast that target it cost {2} more.
pub fn icefall_regent() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::TapAndLockWhileSourcePresent {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            while_you_control: true,
        })],
        static_abilities: vec![StaticAbility {
            description: "Spells your opponents cast that target this creature cost {2} more to cast.",
            effect: StaticEffect::TaxOpponentSpellsTargetingThis { amount: 2 },
        }],
        ..creature(
            "Icefall Regent",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Dragon],
            4,
            3,
        )
    }
}

// ── Agent Bishop, Man in Black ───────────────────────────────────────────────

/// Agent Bishop, Man in Black — {2}{W} legendary 1/2 Human Soldier. At the
/// beginning of combat on your turn, a +1/+1 counter on each of up to two
/// target creatures.
pub fn agent_bishop_man_in_black() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::ApplyToTargets {
                max_targets: 2,
                min_targets: 0,
                filter: R::Creature,
                effect: Box::new(plus_one(Selector::Target(0))),
            },
        }],
        ..legend(
            "Agent Bishop, Man in Black",
            cost(&[generic(2), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            2,
        )
    }
}

/// Sally Pride, Lioness Leader — {3}{W}{W} legendary 2/4 Cat Mutant Rebel.
/// ETB: X 2/2 red Mutants, X = nontoken creatures you control. Attacks: a
/// +1/+1 counter on each creature you control.
pub fn sally_pride_lioness_leader() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![
            etb(make(
                Value::PermanentCountControlledByMatching(
                    PlayerRef::You,
                    R::Creature.and(R::Not(Box::new(R::IsToken))),
                ),
                token("Mutant", 2, 2, Color::Red, vec![CreatureType::Mutant]),
            )),
            on_attack(plus_one(yours(R::Creature))),
        ],
        ..legend(
            "Sally Pride, Lioness Leader",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Cat, CreatureType::Mutant, CreatureType::Rebel],
            2,
            4,
        )
    }
}

// ── The First Doctor // Graham O'Brien ───────────────────────────────────────

/// Stormcaller's Boon — {2}{W}{U} enchantment, cascade. Sacrifice it: creatures
/// you control gain flying until end of turn.
pub fn stormcallers_boon() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: yours(R::Creature),
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..spell(
            "Stormcaller's Boon",
            cost(&[generic(2), w(), u()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Captured Sunlight — {2}{G}{W} sorcery, cascade. You gain 4 life.
pub fn captured_sunlight() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Cascade],
        ..spell(
            "Captured Sunlight",
            cost(&[generic(2), g(), w()]),
            CardType::Sorcery,
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(4),
            },
        )
    }
}

// ── The First Doctor // Amy Pond ─────────────────────────────────────────────

/// Federation Field Medic — {3}{W} 3/2 vigilance Human Doctor. ETB: choose one —
/// a +1/+1 counter on target creature and it gains vigilance until end of
/// turn; or draw a card if you control a permanent with a counter on it.
pub fn federation_field_medic() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![etb(Effect::ChooseMode(vec![
            Effect::Seq(vec![
                plus_one(target_filtered(R::Creature)),
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Vigilance,
                    duration: Duration::EndOfTurn,
                },
            ]),
            Effect::If {
                cond: Predicate::SelectorExists(yours(R::Permanent.and(R::WithAnyCounter))),
                then: Box::new(draw(1)),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        ..creature(
            "Federation Field Medic",
            cost(&[generic(3), w()]),
            vec![CreatureType::Human, CreatureType::Doctor],
            3,
            2,
        )
    }
}

/// Sickbay Orderly — {2}{W} 3/2 lifelink Caitian Doctor.
pub fn sickbay_orderly() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Lifelink],
        ..creature(
            "Sickbay Orderly",
            cost(&[generic(2), w()]),
            vec![CreatureType::Caitian, CreatureType::Doctor],
            3,
            2,
        )
    }
}

// ── Grandmother Sengir ───────────────────────────────────────────────────────

/// Long-Bodied Grey Dog — {3} 2/2 flash, reach Dog artifact-less creature.
/// ETB: a tapped Treasure.
pub fn long_bodied_grey_dog() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash, Keyword::Reach],
        triggered_abilities: vec![etb(make(
            Value::ONE,
            TokenDefinition {
                tapped: true,
                ..crabomination_base::tokens::treasure_token()
            },
        ))],
        ..creature(
            "Long-Bodied Grey Dog",
            cost(&[generic(3)]),
            vec![CreatureType::Dog],
            2,
            2,
        )
    }
}

/// Old Thrush — {2} 1/2 flying Bird. ETB: gain 2 life; you may search your
/// library for a basic land card, reveal it, then shuffle and put it on top.
pub fn old_thrush() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
            Effect::MayDo {
                description: "Search for a basic land to put on top?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: LibraryPosition::Top,
                    },
                }),
            },
        ]))],
        ..creature("Old Thrush", cost(&[generic(2)]), vec![CreatureType::Bird], 1, 2)
    }
}

// ── Bumi, King of Three Trials ───────────────────────────────────────────────

/// Seed of Hope — {G} instant. Mill two; you may put a permanent card from
/// among them into your hand. You gain 2 life.
pub fn seed_of_hope() -> CardDefinition {
    spell(
        "Seed of Hope",
        cost(&[g()]),
        CardType::Instant,
        Effect::Seq(vec![
            Effect::MillThenToHand {
                amount: Value::Const(2),
                filter: R::PermanentCard,
                otherwise: None,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        ]),
    )
}

/// Wasteful Harvest — {2}{G} instant. Mill five; you may put a permanent card
/// from among them into your hand.
pub fn wasteful_harvest() -> CardDefinition {
    spell(
        "Wasteful Harvest",
        cost(&[generic(2), g()]),
        CardType::Instant,
        Effect::MillThenToHand {
            amount: Value::Const(5),
            filter: R::PermanentCard,
            otherwise: None,
        },
    )
}

// ── Vision, Spectral Synthezoid ──────────────────────────────────────────────

/// Vision, Spectral Synthezoid — {6}{U}{U} legendary 2/5 flying Robot Hero
/// artifact creature. Once during each of your turns, you may cast a
/// noncreature or Robot spell from your hand without paying its mana cost.
pub fn vision_spectral_synthezoid() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Once during each of your turns, you may cast a noncreature or Robot spell from your hand without paying its mana cost.",
            effect: StaticEffect::ZeroAlternativeCostOncePerYourTurn {
                filter: R::Noncreature.or(R::HasCreatureType(CreatureType::Robot)),
            },
        }],
        ..legend(
            "Vision, Spectral Synthezoid",
            cost(&[generic(6), u(), u()]),
            vec![CreatureType::Robot, CreatureType::Hero],
            2,
            5,
        )
    }
}

/// Repulsor Bots — {4}{U}{U} 4/4 flying Robot Hero artifact creature. ETB:
/// return up to two other target artifacts and/or creatures to their owners'
/// hands.
pub fn repulsor_bots() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: (R::Artifact.or(R::Creature)).and(R::OtherThanSource),
            effect: Box::new(Effect::Move {
                what: Selector::Target(0),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            }),
        })],
        ..creature(
            "Repulsor Bots",
            cost(&[generic(4), u(), u()]),
            vec![CreatureType::Robot, CreatureType::Hero],
            4,
            4,
        )
    }
}

// ── Viconia, Drow Apostate ───────────────────────────────────────────────────

/// Dogged Pursuit — {3}{B} enchantment. At the beginning of your end step,
/// each opponent loses 1 life and you gain 1 life.
pub fn dogged_pursuit() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![your_end_step(Effect::Seq(vec![
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::ONE,
            },
        ]))],
        ..spell(
            "Dogged Pursuit",
            cost(&[generic(3), b()]),
            CardType::Enchantment,
            Effect::Noop,
        )
    }
}

/// Summon Undead — {4}{B} sorcery. You may mill three. Then return a creature
/// card from your graveyard to the battlefield.
pub fn summon_undead() -> CardDefinition {
    spell(
        "Summon Undead",
        cost(&[generic(4), b()]),
        CardType::Sorcery,
        Effect::Seq(vec![
            Effect::MayDo {
                description: "Mill three cards?".into(),
                body: Box::new(Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(3),
                }),
            },
            crate::effect::shortcut::choose_one_then(
                Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: R::Creature,
                },
                PlayerRef::You,
                Effect::Move {
                    what: crate::effect::shortcut::chosen_one(),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ),
        ]),
    )
}

// ── Renet, Temporal Apprentice ───────────────────────────────────────────────

/// Renet, Temporal Apprentice — {3}{U}{U} legendary 4/3 flash Human Wizard.
/// ETB: return each other nonland permanent that entered this turn to its
/// owner's hand.
pub fn renet_temporal_apprentice() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::EachPermanent(
                R::Permanent
                    .and(R::Nonland)
                    .and(R::EnteredThisTurn)
                    .and(R::OtherThanSource),
            ),
            to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
        })],
        ..legend(
            "Renet, Temporal Apprentice",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Human, CreatureType::Wizard],
            4,
            3,
        )
    }
}

/// Salvager of Secrets — {3}{U}{U} 2/2 Merfolk Wizard. ETB: return target
/// instant or sorcery card from your graveyard to your hand.
pub fn salvager_of_secrets() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered((R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery))).and(R::InYourGraveyard)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..creature(
            "Salvager of Secrets",
            cost(&[generic(3), u(), u()]),
            vec![CreatureType::Merfolk, CreatureType::Wizard],
            2,
            2,
        )
    }
}

// ── Living Brain, Mechanical Marvel ──────────────────────────────────────────

/// Living Brain, Mechanical Marvel — {4} legendary 3/3 Robot Villain artifact
/// creature. At the beginning of combat on your turn, target non-Equipment
/// artifact you control becomes an artifact creature with base 3/3 until end
/// of turn; untap it.
pub fn living_brain_mechanical_marvel() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::BecomeCreature {
                    what: target_filtered(
                        R::Artifact
                            .and(R::ControlledByYou)
                            .and(R::Not(Box::new(R::HasArtifactSubtype(
                                crate::card::ArtifactSubtype::Equipment,
                            )))),
                    ),
                    power: Value::Const(3),
                    toughness: Value::Const(3),
                    creature_types: vec![],
                    keywords: vec![],
                    duration: Duration::EndOfTurn,
                },
                Effect::Untap {
                    what: Selector::Target(0),
                    up_to: None,
                },
            ]),
        }],
        ..legend(
            "Living Brain, Mechanical Marvel",
            cost(&[generic(4)]),
            vec![CreatureType::Robot, CreatureType::Villain],
            3,
            3,
        )
    }
}

/// Levitating Statue — {2} flying artifact. Whenever you cast a noncreature
/// spell, a +1/+1 counter on it. {2}: it becomes a 1/1 Construct artifact
/// creature until end of turn.
pub fn levitating_statue() -> CardDefinition {
    CardDefinition {
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_noncreature_cast(plus_one(Selector::This))],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(2)]),
            effect: Effect::BecomeCreature {
                what: Selector::This,
                power: Value::ONE,
                toughness: Value::ONE,
                creature_types: vec![CreatureType::Construct],
                keywords: vec![],
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..spell("Levitating Statue", cost(&[generic(2)]), CardType::Artifact, Effect::Noop)
    }
}

// ── Iron Lad, Young Avenger ──────────────────────────────────────────────────

/// Iron Lad, Young Avenger — {2}{U/R} legendary 2/2 flying Human Hero artifact
/// creature. Noncreature spells you cast cost {1} less.
pub fn iron_lad_young_avenger() -> CardDefinition {
    CardDefinition {
        card_types: vec![CardType::Artifact, CardType::Creature],
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: R::Noncreature,
                amount: 1,
            },
        }],
        ..legend(
            "Iron Lad, Young Avenger",
            ManaCost {
                symbols: vec![generic(2), hybrid(Color::Blue, Color::Red)],
            },
            vec![CreatureType::Human, CreatureType::Hero],
            2,
            2,
        )
    }
}

/// Wiccan, Young Avenger — {3}{R} legendary 3/3 Mutant Warlock Hero. Whenever
/// you cast a noncreature spell, exile the top card of your library; you may
/// play it until your next end step.
pub fn wiccan_young_avenger() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![on_noncreature_cast(Effect::ExileTopAndGrantMayPlay {
            who: PlayerRef::You,
            count: Value::ONE,
            duration: MayPlayDuration::UntilYourNextEndStep,
            pay_any_color: false,
            max_mana_value: None,
            pay_own_cost: true,
            uncast_penalty: None,
        })],
        ..legend(
            "Wiccan, Young Avenger",
            cost(&[generic(3), r()]),
            vec![CreatureType::Mutant, CreatureType::Warlock, CreatureType::Hero],
            3,
            3,
        )
    }
}

// ── Daughter of Autumn ───────────────────────────────────────────────────────

/// Siegehorn Ceratops — {G}{W} 2/2 Dinosaur. Enrage — two +1/+1 counters.
pub fn siegehorn_ceratops() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
        })],
        ..creature(
            "Siegehorn Ceratops",
            cost(&[g(), w()]),
            vec![CreatureType::Dinosaur],
            2,
            2,
        )
    }
}

/// Trapjaw Tyrant — {3}{W}{W} 5/5 Dinosaur. Enrage — exile target creature an
/// opponent controls until it leaves the battlefield.
pub fn trapjaw_tyrant() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![enrage(Effect::ExileUntilSourceLeaves {
            what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            return_to: ExileReturnZone::Battlefield,
        })],
        ..creature(
            "Trapjaw Tyrant",
            cost(&[generic(3), w(), w()]),
            vec![CreatureType::Dinosaur],
            5,
            5,
        )
    }
}

// ── Spider-UK ────────────────────────────────────────────────────────────────

/// Spider-UK — {3}{W} legendary 3/4 Spider Human Hero; web-slinging {2}{W}. At
/// the beginning of your end step, if two or more creatures entered under your
/// control this turn, draw a card and gain 2 life.
pub fn spider_uk() -> CardDefinition {
    CardDefinition {
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), w()]),
            return_to_hand: Some((R::Creature.and(R::Tapped), 1)),
            ..Default::default()
        }),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(
                    Value::CreaturesEnteredThisTurn(PlayerRef::You),
                    Value::Const(2),
                )),
            effect: Effect::Seq(vec![
                draw(1),
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..legend(
            "Spider-UK",
            cost(&[generic(3), w()]),
            vec![CreatureType::Spider, CreatureType::Human, CreatureType::Hero],
            3,
            4,
        )
    }
}

/// Wakandan Shield Guard — {1}{W} 1/1 Human Soldier. ETB: a 1/1 white Soldier.
pub fn wakandan_shield_guard() -> CardDefinition {
    CardDefinition {
        triggered_abilities: vec![etb(make(
            Value::ONE,
            token("Soldier", 1, 1, Color::White, vec![CreatureType::Soldier]),
        ))],
        ..creature(
            "Wakandan Shield Guard",
            cost(&[generic(1), w()]),
            vec![CreatureType::Human, CreatureType::Soldier],
            1,
            1,
        )
    }
}
