//! Guildpact (GPT) gap cards filling the `set_gaps.py gpt` remainder: simple
//! creatures, a haunt reanimator, and a pair of drain/damage spells.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, LandType, SelectionRequirement as R, Subtypes, TriggeredAbility, Zone,
};
use crate::effect::shortcut::{bloodthirst, etb, on_dies, target_filtered};
use crate::effect::{Effect, PlayerRef, Selector, Value, ZoneDest};
use crate::mana::{Color, b, cost, g, generic, r, u, w};

/// Giant Solifuge — {2}{R/G}{R/G} 4/1 Insect with trample, haste, shroud.
pub fn giant_solifuge() -> CardDefinition {
    CardDefinition {
        name: "Giant Solifuge",
        cost: cost(&[
            generic(2),
            crate::mana::hybrid(Color::Red, Color::Green),
            crate::mana::hybrid(Color::Red, Color::Green),
        ]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Trample, Keyword::Haste, Keyword::Shroud],
        ..Default::default()
    }
}

/// Crystal Seer — {4}{U} 2/2 Vedalken Wizard. ETB: look at the top four cards
/// of your library, then put them back in any order. `{4}{U}: Return this to
/// its owner's hand.`
pub fn crystal_seer() -> CardDefinition {
    CardDefinition {
        name: "Crystal Seer",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::RearrangeTop {
            who: PlayerRef::You,
            amount: Value::Const(4),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(4), u()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Izzet Chronarch — {3}{U}{R} 2/2 Human Wizard. ETB: return target instant or
/// sorcery card from your graveyard to your hand.
pub fn izzet_chronarch() -> CardDefinition {
    CardDefinition {
        name: "Izzet Chronarch",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Move {
            what: Selector::one_of(Selector::CardsInZone {
                who: PlayerRef::You,
                zone: Zone::Graveyard,
                filter: R::HasCardType(CardType::Instant).or(R::HasCardType(CardType::Sorcery)),
            }),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Drowned Rusalka — {U} 1/1 Spirit. `{U}, Sacrifice a creature: Discard a
/// card, then draw a card.`
pub fn drowned_rusalka() -> CardDefinition {
    CardDefinition {
        name: "Drowned Rusalka",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::ONE,
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Crash Landing — {2}{G} Instant. Target creature with flying loses flying
/// until end of turn; deal damage to it equal to the number of Forests you
/// control.
pub fn crash_landing() -> CardDefinition {
    CardDefinition {
        name: "Crash Landing",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::LoseKeyword { duration: crate::effect::Duration::EndOfTurn,
                what: target_filtered(R::Creature.and(R::HasKeyword(Keyword::Flying))),
                keyword: Keyword::Flying,
            },
            Effect::DealDamage {
                to: Selector::Target(0),
                amount: Value::CountMatching {
                    sel: Box::new(Selector::EachPermanent(R::ControlledByYou)),
                    filter: R::HasLandType(LandType::Forest),
                },
            },
        ]),
        ..Default::default()
    }
}

/// Hissing Miasma — {1}{B}{B} Enchantment. Whenever a creature attacks you, its
/// controller loses 1 life.
pub fn hissing_miasma() -> CardDefinition {
    CardDefinition {
        name: "Hissing Miasma",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Agent of Masks — {3}{W}{B} 2/3 Human Advisor. At the beginning of your
/// upkeep, each opponent loses 1 life and you gain that much.
pub fn agent_of_masks() -> CardDefinition {
    CardDefinition {
        name: "Agent of Masks",
        cost: cost(&[generic(3), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(crate::game::TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Exhumer Thrull — {5}{B} 3/3 Thrull with haunt. When it enters or the
/// creature it haunts dies, return target creature card from your graveyard to
/// your hand.
pub fn exhumer_thrull() -> CardDefinition {
    let recur = || Effect::Move {
        what: Selector::one_of(Selector::CardsInZone {
            who: PlayerRef::You,
            zone: Zone::Graveyard,
            filter: R::Creature,
        }),
        to: ZoneDest::Hand(PlayerRef::You),
    };
    CardDefinition {
        name: "Exhumer Thrull",
        cost: cost(&[generic(5), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Thrull],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![
            etb(recur()),
            on_dies(Effect::HauntCreature {
                body: Box::new(recur()),
            }),
        ],
        ..Default::default()
    }
}

/// Benediction of Moons — {W} Sorcery with haunt. You gain 1 life for each
/// player; when the creature it haunts dies, you gain 1 life for each player.
pub fn benediction_of_moons() -> CardDefinition {
    CardDefinition {
        name: "Benediction of Moons",
        cost: cost(&[w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainLife {
                who: Selector::You,
                amount: Value::PlayerCount,
            },
            Effect::HauntCreature {
                body: Box::new(Effect::GainLife {
                    who: Selector::You,
                    amount: Value::PlayerCount,
                }),
            },
        ]),
        ..Default::default()
    }
}

/// Burning-Tree Shaman — {1}{R}{G} 3/4 Centaur Shaman. Whenever a player
/// activates an ability that isn't a mana ability, deal 1 damage to that player.
pub fn burning_tree_shaman() -> CardDefinition {
    CardDefinition {
        name: "Burning-Tree Shaman",
        cost: cost(&[generic(1), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Shaman],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::AbilityActivated, EventScope::AnyPlayer),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::ONE,
            },
        }],
        ..Default::default()
    }
}

/// Burning-Tree Bloodscale — {2}{R}{G} 2/2 Lizard Berserker with bloodthirst 1.
/// `{2}{R}: Target creature can't block this creature this turn.` /
/// `{2}{G}: Target creature blocks this creature this turn if able.`
pub fn burning_tree_bloodscale() -> CardDefinition {
    CardDefinition {
        name: "Burning-Tree Bloodscale",
        cost: cost(&[generic(2), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Bloodthirst(1)],
        triggered_abilities: vec![bloodthirst(1)],
        activated_abilities: vec![
            ActivatedAbility {
                mana_cost: cost(&[generic(2), r()]),
                effect: Effect::CantBlockSourceThisTurn {
                    target: target_filtered(R::Creature),
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(2), g()]),
                effect: Effect::MustBlockSource {
                    what: target_filtered(R::Creature),                    chooser: None,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Culling Sun — {2}{W}{W}{B} Sorcery. Destroy each creature with mana value 3
/// or less.
pub fn culling_sun() -> CardDefinition {
    CardDefinition {
        name: "Culling Sun",
        cost: cost(&[generic(2), w(), w(), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Destroy {
            what: Selector::EachPermanent(R::Creature.and(R::ManaValueAtMost(3))),
        },
        ..Default::default()
    }
}

/// Ghostway — {2}{W} Instant. Exile each creature you control, then return them
/// to the battlefield under their owner's control at the next end step.
pub fn ghostway() -> CardDefinition {
    CardDefinition {
        name: "Ghostway",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ExileReturnNextEndStep {
            what: Selector::EachPermanent(R::Creature.and(R::ControlledByYou)),
        },
        ..Default::default()
    }
}

/// Leyline of Lifeforce — {2}{G}{G} Enchantment. If in your opening hand, you
/// may begin the game with it in play. Creature spells can't be countered.
pub fn leyline_of_lifeforce() -> CardDefinition {
    use crate::card::StaticAbility;
    use crate::effect::{OpeningHandEffect, StaticEffect};
    CardDefinition {
        name: "Leyline of Lifeforce",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creature spells can't be countered.",
            effect: StaticEffect::CreatureSpellsCantBeCountered,
        }],
        opening_hand: Some(OpeningHandEffect::StartInPlay {
            tapped: false,
            extra: Effect::Noop,
        }),
        ..Default::default()
    }
}
