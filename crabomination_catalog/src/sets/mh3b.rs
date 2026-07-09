//! Modern Horizons 3 (MH3), batch 2 — Eldrazi/colorless matters, adapt/modified
//! payoffs, living-weapon equipment, and modal/overload spells. All ride
//! existing engine primitives. Tests in `tests/mh3b.rs`.

use crate::card::{
    ActivatedAbility, ArtifactSubtype, CardDefinition, CardType, CounterType, CreatureType,
    EquipBonus, EventKind, EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R,
    Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::{adapt, drain, etb, on_attack, on_cast, target_filtered, unearth};
use crate::effect::{Duration, Effect, PlayerRef, Selector, StaticEffect, Value, ZoneDest};
use crate::mana::{b, colorless, cost, g, generic, r, u, w, Color};

fn germ() -> TokenDefinition {
    TokenDefinition {
        name: "Phyrexian Germ".into(),
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Phyrexian], ..Default::default() },
        ..Default::default()
    }
}

/// Living-weapon ETB: mint a Germ and attach this Equipment to it.
fn living_weapon() -> TriggeredAbility {
    etb(Effect::Seq(vec![
        Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: germ() },
        Effect::Attach { what: Selector::This, to: Selector::LastCreatedToken },
    ]))
}

// ── Eldrazi / colorless ──────────────────────────────────────────────────────

/// Eldrazi Ravager — {5}{C} 6/6 Eldrazi. Annihilator 1. Sacrifice two Eldrazi:
/// return this from your graveyard to your hand. Cycling {2}.
pub fn eldrazi_ravager() -> CardDefinition {
    CardDefinition {
        name: "Eldrazi Ravager",
        cost: cost(&[generic(5), colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Annihilator(1), Keyword::Cycling(cost(&[generic(2)]))],
        activated_abilities: vec![ActivatedAbility {
            from_graveyard: true,
            sac_other_filter: Some((R::HasCreatureType(CreatureType::Eldrazi), 2)),
            effect: Effect::Move { what: Selector::This, to: ZoneDest::Hand(PlayerRef::You) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Breaker of Creation — {6}{C}{C} 8/4 Eldrazi. When you cast this, gain 1 life
/// for each colorless permanent you control. Hexproof from each color.
/// Annihilator 2.
pub fn breaker_of_creation() -> CardDefinition {
    CardDefinition {
        name: "Breaker of Creation",
        cost: cost(&[generic(6), colorless(1), colorless(1)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 8,
        toughness: 4,
        keywords: vec![
            Keyword::Annihilator(2),
            Keyword::HexproofFromColor(Color::White),
            Keyword::HexproofFromColor(Color::Blue),
            Keyword::HexproofFromColor(Color::Black),
            Keyword::HexproofFromColor(Color::Red),
            Keyword::HexproofFromColor(Color::Green),
        ],
        triggered_abilities: vec![on_cast(Effect::GainLife {
            who: Selector::You,
            amount: Value::CountMatching {
                sel: Box::new(Selector::EachPermanent(R::ControlledByYou.and(R::Colorless))),
                filter: R::ControlledByYou.and(R::Colorless),
            },
        })],
        ..Default::default()
    }
}

/// Drownyard Lurker — {7} 7/7 Eldrazi Trilobite. Vigilance. When you cast or
/// cycle this, create a 0/1 Eldrazi Spawn. Cycling {2}{U}.
pub fn drownyard_lurker() -> CardDefinition {
    let make_spawn = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::ONE,
        definition: crate::game::effects::eldrazi_spawn_token(),
    };
    CardDefinition {
        name: "Drownyard Lurker",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Trilobite],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Vigilance, Keyword::Cycling(cost(&[generic(2), u()]))],
        triggered_abilities: vec![
            on_cast(make_spawn()),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardCycled, EventScope::SelfSource),
                effect: make_spawn(),
            },
        ],
        ..Default::default()
    }
}

/// Emrakul's Messenger — {1}{U} 2/1 Devoid Eldrazi Faerie Rogue. Flying.
/// Whenever you draw your second card each turn, create a 0/1 Eldrazi Spawn.
pub fn emrakuls_messenger() -> CardDefinition {
    CardDefinition {
        name: "Emrakul's Messenger",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Devoid, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 2 })
                .once_per_turn(),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: crate::game::effects::eldrazi_spawn_token(),
            },
        }],
        ..Default::default()
    }
}

/// Petrifying Meddler — {4}{U} 4/5 Devoid Eldrazi. Reach. When you cast this,
/// tap up to one target creature and put a stun counter on it.
pub fn petrifying_meddler() -> CardDefinition {
    CardDefinition {
        name: "Petrifying Meddler",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Eldrazi], ..Default::default() },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Devoid, Keyword::Reach],
        triggered_abilities: vec![on_cast(Effect::Seq(vec![
            Effect::Tap { what: target_filtered(R::Creature) },
            Effect::AddCounter {
                what: target_filtered(R::Creature),
                kind: CounterType::Stun,
                amount: Value::ONE,
            },
        ]))],
        ..Default::default()
    }
}

/// Hope-Ender Coatl — {2}{U} 2/2 Devoid Eldrazi Snake. Flash, Flying. When you
/// cast this, counter target spell an opponent controls unless they pay {1}.
pub fn hope_ender_coatl() -> CardDefinition {
    CardDefinition {
        name: "Hope-Ender Coatl",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Snake],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Devoid, Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![on_cast(Effect::CounterUnlessPaid {
            what: Selector::TargetFiltered { slot: 0, filter: R::ControlledByOpponent },
            mana_cost: cost(&[generic(1)]),
            exile: false,
            extra_generic: None,
        })],
        ..Default::default()
    }
}

// ── Adapt / modified matters ─────────────────────────────────────────────────

/// Dreamdrinker Vampire — {1}{B} 2/1 Vampire. Lifelink. {1}{B}: Adapt 1.
/// Whenever one or more +1/+1 counters are put on this, it gains menace EOT.
pub fn dreamdrinker_vampire() -> CardDefinition {
    CardDefinition {
        name: "Dreamdrinker Vampire",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Lifelink],
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(1), b()]), effect: adapt(1), ..Default::default() }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::SelfSource),
            effect: Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Menace,
                duration: Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Evolution Witness — {2}{G} 2/1 Elf Shaman Mutant. {1}{G}: Adapt 2. Whenever
/// one or more +1/+1 counters are put on this, return target permanent card
/// from your graveyard to your hand.
pub fn evolution_witness() -> CardDefinition {
    CardDefinition {
        name: "Evolution Witness",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman, CreatureType::Mutant],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[generic(1), g()]), effect: adapt(2), ..Default::default() }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CounterAdded(CounterType::PlusOnePlusOne), EventScope::SelfSource),
            effect: Effect::Move {
                what: Selector::TargetFiltered {
                    slot: 0,
                    filter: R::InYourGraveyard.and(R::PermanentCard),
                },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Expanding Ooze — {1}{B}{G} 3/3 Ooze. {B}{G}: Adapt 1. Whenever this attacks,
/// put a +1/+1 counter on target modified creature you control.
pub fn expanding_ooze() -> CardDefinition {
    CardDefinition {
        name: "Expanding Ooze",
        cost: cost(&[generic(1), b(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Ooze], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility { mana_cost: cost(&[b(), g()]), effect: adapt(1), ..Default::default() }],
        triggered_abilities: vec![on_attack(Effect::AddCounter {
            what: target_filtered(R::Creature.and(R::ControlledByYou).and(R::IsModified)),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::ONE,
        })],
        ..Default::default()
    }
}

/// Envoy of the Ancestors — {2}{W} 2/3 Human Cleric. Outlast {W}. Modified
/// creatures you control have lifelink.
pub fn envoy_of_the_ancestors() -> CardDefinition {
    use crate::effect::shortcut::outlast;
    CardDefinition {
        name: "Envoy of the Ancestors",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![outlast(cost(&[w()]))],
        static_abilities: vec![crate::card::StaticAbility {
            description: "Modified creatures you control have lifelink.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    R::Creature.and(R::ControlledByYou).and(R::IsModified),
                ),
                keyword: Keyword::Lifelink,
            },
        }],
        ..Default::default()
    }
}

/// Guardian of the Forgotten — {3}{W} 4/4 Elephant Warrior. Vigilance. Whenever
/// a modified creature you control dies, manifest the top card of your library.
pub fn guardian_of_the_forgotten() -> CardDefinition {
    CardDefinition {
        name: "Guardian of the Forgotten",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl).with_filter(
                Predicate::EntityMatches { what: Selector::TriggerSource, filter: R::IsModified },
            ),
            effect: Effect::Manifest { who: PlayerRef::You, amount: Value::ONE },
        }],
        ..Default::default()
    }
}

// ── Value creatures ──────────────────────────────────────────────────────────

/// Grim Servant — {3}{B} 3/2 Zombie Warlock. Menace. ETB: search your library
/// for a card with mana value ≤ your devotion to black to your hand, then
/// shuffle. You lose 3 life.
pub fn grim_servant() -> CardDefinition {
    CardDefinition {
        name: "Grim Servant",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: R::ManaValueAtMostDevotion(Color::Black),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(3) },
        ]))],
        ..Default::default()
    }
}

/// Marionette Apprentice — {1}{B} 1/2 Human Artificer. Fabricate 1. Whenever
/// another creature or artifact you control is put into a graveyard from the
/// battlefield, each opponent loses 1 life.
pub fn marionette_apprentice() -> CardDefinition {
    use crate::effect::shortcut::fabricate;
    CardDefinition {
        name: "Marionette Apprentice",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![
            fabricate(1),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureOrArtifactDied, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::OtherThanSource,
                    }),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::ONE,
                },
            },
        ],
        ..Default::default()
    }
}

/// Molten Gatekeeper — {2}{R} 2/3 Golem. Whenever another creature you control
/// enters, this deals 1 damage to each opponent. Unearth {R}.
pub fn molten_gatekeeper() -> CardDefinition {
    CardDefinition {
        name: "Molten Gatekeeper",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 2,
        toughness: 3,
        activated_abilities: vec![unearth(cost(&[r()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                },
            ),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Kami of Jealous Thirst — {2}{B} 1/3 Spirit. Deathtouch. {4}{B}: each
/// opponent loses 2 life and you gain 2 life. Activate only once each turn.
pub fn kami_of_jealous_thirst() -> CardDefinition {
    CardDefinition {
        name: "Kami of Jealous Thirst",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), b()]),
            once_per_turn: true,
            effect: drain(2),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Infernal Captor — {3}{R} 3/3 Devil Rogue. Exploit. When this exploits a
/// creature, gain control of target artifact or creature until end of turn.
/// Untap it. It gains haste until end of turn.
pub fn infernal_captor() -> CardDefinition {
    use crate::effect::shortcut::exploit;
    CardDefinition {
        name: "Infernal Captor",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Devil, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![exploit(Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Artifact.or(R::Creature)),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: target_filtered(R::Artifact.or(R::Creature)), up_to: None },
            Effect::GrantKeyword {
                what: target_filtered(R::Artifact.or(R::Creature)),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]))],
        ..Default::default()
    }
}

// ── Equipment (living weapon) ────────────────────────────────────────────────

/// Colossal Dreadmask — {4}{G}{G} Equipment. Living weapon. Equipped creature
/// gets +6/+6 and has trample. Equip {3}{G}{G}.
pub fn colossal_dreadmask() -> CardDefinition {
    CardDefinition {
        name: "Colossal Dreadmask",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(3), g(), g()]))],
        equipped_bonus: Some(EquipBonus { power: 6, toughness: 6, keywords: vec![Keyword::Trample], ..Default::default() }),
        triggered_abilities: vec![living_weapon()],
        ..Default::default()
    }
}

/// Drossclaw — {1}{B} Equipment. Living weapon. Equipped creature gets +1/+1.
/// Whenever equipped creature attacks, each opponent loses 1 life. Equip {2}.
pub fn drossclaw() -> CardDefinition {
    CardDefinition {
        name: "Drossclaw",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact],
        subtypes: Subtypes { artifact_subtypes: vec![ArtifactSubtype::Equipment], ..Default::default() },
        keywords: vec![Keyword::Equip(cost(&[generic(2)]))],
        equipped_bonus: Some(EquipBonus {
            power: 1,
            toughness: 1,
            triggered_abilities: vec![on_attack(Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            })],
            ..Default::default()
        }),
        triggered_abilities: vec![living_weapon()],
        ..Default::default()
    }
}

// ── Spells ───────────────────────────────────────────────────────────────────

/// Horrific Assault — {G} Sorcery. Target creature you control deals damage
/// equal to its power to target creature or planeswalker you don't control. If
/// you control an Eldrazi, you gain 3 life.
pub fn horrific_assault() -> CardDefinition {
    CardDefinition {
        name: "Horrific Assault",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::DealDamageEqualToPower {
                source: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::ControlledByYou) },
                target: Selector::TargetFiltered {
                    slot: 1,
                    filter: R::Creature.or(R::Planeswalker).and(R::ControlledByOpponent),
                },
            },
            Effect::If {
                cond: Predicate::SelectorCountAtLeast {
                    sel: Selector::EachPermanent(
                        R::HasCreatureType(CreatureType::Eldrazi).and(R::ControlledByYou),
                    ),
                    n: Value::ONE,
                },
                then: Box::new(Effect::GainLife { who: Selector::You, amount: Value::Const(3) }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

/// Brainsurge — {2}{U} Instant. Draw four cards, then put two cards from your
/// hand on top of your library in any order.
pub fn brainsurge() -> CardDefinition {
    CardDefinition {
        name: "Brainsurge",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(4) },
            Effect::PutOnLibraryFromHand { who: PlayerRef::You, count: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Fangs of Kalonia — {1}{G} Sorcery. Put a +1/+1 counter on target creature
/// you control, then double the number of +1/+1 counters on each creature that
/// had a counter put on it this way. Overload {4}{G}{G}.
pub fn fangs_of_kalonia() -> CardDefinition {
    use crate::card::AlternativeCost;
    let each = Selector::EachPermanent(R::Creature.and(R::ControlledByYou));
    CardDefinition {
        name: "Fangs of Kalonia",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            Effect::DoubleCountersOnEach {
                what: target_filtered(R::Creature.and(R::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
            },
        ]),
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(4), g(), g()]),
            effect_override: Some(Effect::Seq(vec![
                Effect::AddCounter { what: each.clone(), kind: CounterType::PlusOnePlusOne, amount: Value::ONE },
                Effect::DoubleCountersOnEach { what: each, kind: CounterType::PlusOnePlusOne },
            ])),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Gravedig — {1}{B} Sorcery. Choose one — target player creates a 2/2 black
/// Zombie; or return target creature card from your graveyard to your hand.
/// Entwine {2}.
pub fn gravedig() -> CardDefinition {
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Black],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        ..Default::default()
    };
    CardDefinition {
        name: "Gravedig",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Entwine(cost(&[generic(2)]))],
        effect: Effect::ChooseMode(vec![
            Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: zombie },
            Effect::Move {
                what: Selector::TargetFiltered { slot: 0, filter: R::Creature.and(R::InYourGraveyard) },
                to: ZoneDest::Hand(PlayerRef::You),
            },
        ]),
        ..Default::default()
    }
}

// ── Batch 2 ──────────────────────────────────────────────────────────────────

use crate::card::StaticAbility;
use crate::effect::shortcut::{amass_zombies, modular_dies};
use crate::game::TurnStep;

fn spirit_flyer() -> TokenDefinition {
    TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Metastatic Evangel — {1}{W} 3/1 Phyrexian Human Cleric. Whenever another
/// nontoken creature you control enters, proliferate.
pub fn metastatic_evangel() -> CardDefinition {
    CardDefinition {
        name: "Metastatic Evangel",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::NotToken).and(R::OtherThanSource),
                },
            ),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Muster the Departed — {2}{W} Enchantment. ETB: create a 1/1 white flying
/// Spirit. Morbid — at the beginning of your end step, if a creature died this
/// turn, populate.
pub fn muster_the_departed() -> CardDefinition {
    CardDefinition {
        name: "Muster the Departed",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(Effect::CreateToken { who: PlayerRef::You, count: Value::ONE, definition: spirit_flyer() }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::SelfSource)
                    .with_filter(Predicate::IsTurnOf(PlayerRef::You)),
                effect: Effect::If {
                    cond: Predicate::CreaturesDiedThisTurnTotalAtLeast { at_least: Value::ONE },
                    then: Box::new(Effect::Populate { who: PlayerRef::You }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        ..Default::default()
    }
}

/// Obstinate Gargoyle — {1}{W}{B} 2/2 Gargoyle. Flying while modified. Persist.
pub fn obstinate_gargoyle() -> CardDefinition {
    CardDefinition {
        name: "Obstinate Gargoyle",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Gargoyle], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Persist],
        static_abilities: vec![StaticAbility {
            description: "This creature has flying as long as it's modified.",
            effect: StaticEffect::SelfHasKeywordWhile { keyword: Keyword::Flying, condition: R::IsModified },
        }],
        ..Default::default()
    }
}

/// Arcbound Condor — {2}{B}{B} 0/0 Artifact Bird. Flying. Modular 3. Whenever
/// another artifact you control enters, target creature an opponent controls
/// gets -1/-1 until end of turn.
pub fn arcbound_condor() -> CardDefinition {
    CardDefinition {
        name: "Arcbound Condor",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        keywords: vec![Keyword::Flying],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![
            modular_dies(),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl).with_filter(
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: R::Artifact.and(R::OtherThanSource),
                    },
                ),
                effect: Effect::PumpPT {
                    what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
                    power: Value::Const(-1),
                    toughness: Value::Const(-1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        ..Default::default()
    }
}

/// Kozilek's Unsealing — {2}{U} Devoid Enchantment. Cast a creature with MV
/// 4–6: create two Eldrazi Spawn. Cast a creature with MV 7+: draw three.
pub fn kozileks_unsealing() -> CardDefinition {
    let cast_creature_mv = |lo: u32, hi: Option<u32>, effect: Effect| {
        let mut f = R::Creature.and(R::ManaValueAtLeast(lo));
        if let Some(h) = hi {
            f = f.and(R::ManaValueAtMost(h));
        }
        TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches { what: Selector::TriggerSource, filter: f }),
            effect,
        }
    };
    CardDefinition {
        name: "Kozilek's Unsealing",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Devoid],
        triggered_abilities: vec![
            cast_creature_mv(
                4,
                Some(6),
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(2),
                    definition: crate::game::effects::eldrazi_spawn_token(),
                },
            ),
            cast_creature_mv(7, None, Effect::Draw { who: Selector::You, amount: Value::Const(3) }),
        ],
        ..Default::default()
    }
}

/// Mindless Conscription — {2}{B} Enchantment. When it enters and whenever you
/// draw your third card each turn, amass Zombies 3.
pub fn mindless_conscription() -> CardDefinition {
    CardDefinition {
        name: "Mindless Conscription",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![
            etb(amass_zombies(3)),
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl)
                    .with_filter(Predicate::PlayerDrewAtLeastThisTurn { who: PlayerRef::Triggerer, n: 3 })
                    .once_per_turn(),
                effect: amass_zombies(3),
            },
        ],
        ..Default::default()
    }
}

/// Essence Reliquary — {2}{W} Artifact. {T}: return another target permanent
/// you control to its owner's hand. Activate only during your turn.
pub fn essence_reliquary() -> CardDefinition {
    CardDefinition {
        name: "Essence Reliquary",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            condition: Some(Predicate::IsTurnOf(PlayerRef::You)),
            effect: Effect::Move {
                what: target_filtered(R::Permanent.and(R::ControlledByYou).and(R::OtherThanSource)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Etched Slith — {1}{B} 1/1 Artifact Phyrexian Slith. Menace. Whenever it
/// deals combat damage to a player, put a +1/+1 counter on it.
pub fn etched_slith() -> CardDefinition {
    CardDefinition {
        name: "Etched Slith",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Slith],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}
