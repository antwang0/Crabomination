//! A fourth wave of staples — reanimator pieces, sweepers, lock pieces, and
//! card-advantage spells that filled remaining gaps (Recurring Nightmare,
//! Survival of the Fittest, Living Death, Show and Tell, Deafening Silence, …).
//! Each card has a functionality test in `crabomination/src/tests/recent4.rs`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Predicate, SelectionRequirement, Selector, SpellSubtype, StaticAbility,
    Subtypes, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, Duration, LibraryPosition, PlayerRef, StaticEffect, ZoneDest};
use crate::mana::{b, cost, g, generic, u, w, x, Color};

/// Ritual of Soot — {2}{B}{B} Sorcery. Destroy all creatures with mana value
/// 3 or less.
pub fn ritual_of_soot() -> CardDefinition {
    CardDefinition {
        name: "Ritual of Soot",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ManaValueAtMost(3)),
            ),
        },
        ..Default::default()
    }
}

/// Recurring Nightmare — {2}{B} Enchantment. "Sacrifice a creature, Return
/// this enchantment to its owner's hand: Return target creature card from your
/// graveyard to the battlefield. Activate only as a sorcery."
pub fn recurring_nightmare() -> CardDefinition {
    CardDefinition {
        name: "Recurring Nightmare",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sorcery_speed: true,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            return_self_cost: true,
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Survival of the Fittest — {1}{G} Enchantment. "{G}, Discard a creature
/// card: Search your library for a creature card, reveal it, put it into your
/// hand, then shuffle." (The reveal is cosmetic.)
pub fn survival_of_the_fittest() -> CardDefinition {
    CardDefinition {
        name: "Survival of the Fittest",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            discard_cost: Some((SelectionRequirement::Creature, 1)),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Footsteps of the Goryo — {2}{B} Sorcery — Arcane. Return target creature
/// card from your graveyard to the battlefield; sacrifice it at the beginning
/// of the next end step.
pub fn footsteps_of_the_goryo() -> CardDefinition {
    CardDefinition {
        name: "Footsteps of the Goryo",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes { spell_subtypes: vec![SpellSubtype::Arcane], ..Default::default() },
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                ),
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::DelayUntil {
                kind: DelayedTriggerKind::NextEndStep,
                body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
            },
        ]),
        ..Default::default()
    }
}

/// Apprentice Necromancer — {1}{B} 1/1 Zombie Wizard. "{B}, {T}, Sacrifice
/// this creature: Return target creature card from your graveyard to the
/// battlefield. That creature gains haste. At the beginning of the next end
/// step, sacrifice it."
pub fn apprentice_necromancer() -> CardDefinition {
    CardDefinition {
        name: "Apprentice Necromancer",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[b()]),
            sac_cost: true,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::InYourGraveyard),
                    ),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
                Effect::GrantKeyword {
                    what: Selector::Target(0),
                    keyword: Keyword::Haste,
                    duration: Duration::EndOfTurn,
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::SacrificePermanent { what: Selector::Target(0) }),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Deafening Silence — {W} Enchantment. Each player can't cast more than one
/// noncreature spell each turn.
pub fn deafening_silence() -> CardDefinition {
    CardDefinition {
        name: "Deafening Silence",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Each player can't cast more than one noncreature spell each turn.",
            effect: StaticEffect::OneNoncreatureSpellPerTurn,
        }],
        ..Default::default()
    }
}

/// Ethersworn Canonist — {1}{W} 2/2 Human Cleric artifact creature. Each
/// player who has cast a nonartifact spell this turn can't cast additional
/// nonartifact spells.
pub fn ethersworn_canonist() -> CardDefinition {
    CardDefinition {
        name: "Ethersworn Canonist",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Each player who has cast a nonartifact spell this turn can't cast \
                additional nonartifact spells.",
            effect: StaticEffect::OneNonartifactSpellPerTurn,
        }],
        ..Default::default()
    }
}

/// Defense Grid — {2} Artifact. Each spell costs {3} more to cast except during
/// its controller's turn.
pub fn defense_grid() -> CardDefinition {
    CardDefinition {
        name: "Defense Grid",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Each spell costs {3} more to cast except during its controller's turn.",
            effect: StaticEffect::SpellsCostMoreExceptOnControllerTurn { amount: 3 },
        }],
        ..Default::default()
    }
}

/// Bontu's Last Reckoning — {1}{B}{B} Sorcery. Destroy all creatures. Lands you
/// control don't untap during your next untap step.
pub fn bontus_last_reckoning() -> CardDefinition {
    CardDefinition {
        name: "Bontu's Last Reckoning",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Destroy { what: Selector::EachPermanent(SelectionRequirement::Creature) },
            Effect::LandsDontUntapNextUntapStep { who: Selector::You },
        ]),
        ..Default::default()
    }
}

/// Syphon Mind — {3}{B} Sorcery. Each other player discards a card; you draw a
/// card for each card discarded this way.
pub fn syphon_mind() -> CardDefinition {
    CardDefinition {
        name: "Syphon Mind",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
                random: false,
            },
            Effect::Draw { who: Selector::You, amount: Value::CardsDiscardedThisEffect },
        ]),
        ..Default::default()
    }
}

/// Prosperity — {X}{U} Sorcery. Each player draws X cards.
pub fn prosperity() -> CardDefinition {
    CardDefinition {
        name: "Prosperity",
        cost: cost(&[x(), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Draw {
            who: Selector::Player(PlayerRef::EachPlayer),
            amount: Value::XFromCost,
        },
        ..Default::default()
    }
}

/// Ondu Giant — {3}{G} 2/4 Giant Druid. ETB: you may search your library for a
/// basic land card and put it onto the battlefield tapped.
pub fn ondu_giant() -> CardDefinition {
    CardDefinition {
        name: "Ondu Giant",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..Default::default()
    }
}

/// Roiling Regrowth — {2}{G} Instant. Sacrifice a land. Search your library for
/// up to two basic land cards, put them onto the battlefield tapped.
pub fn roiling_regrowth() -> CardDefinition {
    CardDefinition {
        name: "Roiling Regrowth",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: SelectionRequirement::Land,
            },
            Effect::SearchUpToN {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                count: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Roar of the Wurm — {6}{G} Sorcery. Create a 6/6 green Wurm token. Flashback
/// {3}{G}.
pub fn roar_of_the_wurm() -> CardDefinition {
    CardDefinition {
        name: "Roar of the Wurm",
        cost: cost(&[generic(6), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Flashback(cost(&[generic(3), g()]))],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Wurm".into(),
                power: 6,
                toughness: 6,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Wurm],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Living Death — {3}{B}{B} Sorcery. Each player exiles all creature cards from
/// their graveyard, then sacrifices all creatures they control, then puts all
/// cards they exiled this way onto the battlefield under their control.
pub fn living_death() -> CardDefinition {
    CardDefinition {
        name: "Living Death",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::LivingDeath,
        ..Default::default()
    }
}

/// Show and Tell — {2}{U} Sorcery. Each player may put an artifact, creature,
/// enchantment, or land card from their hand onto the battlefield.
pub fn show_and_tell() -> CardDefinition {
    CardDefinition {
        name: "Show and Tell",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::EachPlayerMayPutPermanentFromHand {
            filter: SelectionRequirement::Permanent,
        },
        ..Default::default()
    }
}

/// Chart a Course — {1}{U} Sorcery. Draw two cards. Then discard a card unless
/// you attacked this turn.
pub fn chart_a_course() -> CardDefinition {
    CardDefinition {
        name: "Chart a Course",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
            Effect::If {
                cond: Predicate::PlayerAttackedThisTurn { who: PlayerRef::You },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Sylvan Tutor — {G} Sorcery. Search your library for a creature card, reveal
/// it, then shuffle and put it on top.
pub fn sylvan_tutor() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Tutor",
        cost: cost(&[g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature,
            to: ZoneDest::Library { who: PlayerRef::You, pos: LibraryPosition::Top },
        },
        ..Default::default()
    }
}

/// Final Parting — {3}{B}{B} Sorcery. Search your library for two cards; put one
/// into your hand and the other into your graveyard, then shuffle.
pub fn final_parting() -> CardDefinition {
    CardDefinition {
        name: "Final Parting",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Any,
                to: ZoneDest::Graveyard,
            },
        ]),
        ..Default::default()
    }
}

/// Altar's Reap — {1}{B} Instant. As an additional cost, sacrifice a creature.
/// Draw two cards. (The additional sacrifice is modeled at resolution.)
pub fn altars_reap() -> CardDefinition {
    CardDefinition {
        name: "Altar's Reap",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: SelectionRequirement::Creature,
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        ]),
        ..Default::default()
    }
}

/// Corpse Knight — {W}{B} 2/2 Zombie Knight. Whenever another creature you
/// control enters, each opponent loses 1 life.
pub fn corpse_knight() -> CardDefinition {
    CardDefinition {
        name: "Corpse Knight",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Harvester of Souls — {4}{B}{B} 5/5 Demon with deathtouch. Whenever another
/// nontoken creature dies, you may draw a card.
pub fn harvester_of_souls() -> CardDefinition {
    CardDefinition {
        name: "Harvester of Souls",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Demon], ..Default::default() },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::OtherThanSource
                        .and(SelectionRequirement::NotToken),
                }),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::ONE }),
            },
        }],
        ..Default::default()
    }
}


/// Snap — {1}{U} Instant. Return target creature to its owner's hand. Untap up
/// to two lands.
pub fn snap() -> CardDefinition {
    CardDefinition {
        name: "Snap",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
            Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::Tapped),
                ),
                up_to: Some(Value::Const(2)),
            },
        ]),
        ..Default::default()
    }
}

/// Throttle — {4}{B} Instant. Target creature gets -4/-4 until end of turn.
pub fn throttle() -> CardDefinition {
    CardDefinition {
        name: "Throttle",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Instant],
        effect: Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-4),
            toughness: Value::Const(-4),
            duration: Duration::EndOfTurn,
        },
        ..Default::default()
    }
}

/// Trophy Mage — {2}{U} 2/2 Human Wizard. ETB: you may search your library for
/// an artifact card with mana value 3 and put it into your hand.
pub fn trophy_mage() -> CardDefinition {
    artifact_tutor_mage("Trophy Mage", 3)
}

/// Tribute Mage — {2}{U} 2/2 Human Wizard. ETB: you may search your library for
/// an artifact card with mana value 2 and put it into your hand.
pub fn tribute_mage() -> CardDefinition {
    artifact_tutor_mage("Tribute Mage", 2)
}

/// Shared body for the {2}{U} 2/2 "ETB: tutor an artifact with mana value N"
/// Wizards (Trophy / Tribute Mage).
fn artifact_tutor_mage(name: &'static str, mv: u32) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Artifact
                    .and(SelectionRequirement::ManaValueExactly(mv)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Thirst for Knowledge — {2}{U} Instant. Draw three cards, then discard two.
/// (The "unless you discard an artifact" reduction is approximated as a flat
/// discard two.)
pub fn thirst_for_knowledge() -> CardDefinition {
    CardDefinition {
        name: "Thirst for Knowledge",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(3) },
            Effect::Discard { who: Selector::You, amount: Value::Const(2), random: false },
        ]),
        ..Default::default()
    }
}

/// Kavu Predator — {1}{G} 2/2 Kavu with trample. Whenever an opponent gains
/// life, put that many +1/+1 counters on it.
pub fn kavu_predator() -> CardDefinition {
    CardDefinition {
        name: "Kavu Predator",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kavu], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::OpponentControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: crate::card::CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Seal Away — {1}{W} Enchantment with flash. ETB: exile target tapped creature
/// until Seal Away leaves the battlefield.
pub fn seal_away() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Seal Away",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Tapped)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Conclave Tribunal — {3}{W} Enchantment with convoke. ETB: exile target
/// nonland permanent an opponent controls until Conclave Tribunal leaves.
pub fn conclave_tribunal() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Conclave Tribunal",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Convoke],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}
