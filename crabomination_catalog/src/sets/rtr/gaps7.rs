//! Return to Ravnica (RTR) gap wave 8: the guild legends, rares, and mythics —
//! anthems, detain payoffs, populate legends, threaten, flash-sorcery, and a
//! self-recurring flier. Tests in `classic_sets/rtr`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Predicate, SelectionRequirement as R, StaticAbility, Subtypes,
    Supertype, TokenDefinition, TriggeredAbility, Value,
};
use crate::effect::shortcut::{etb, on_attack, on_dies, target_filtered, unleash};
use crate::effect::{
    Duration, LibraryPosition, ManaPayload, PlayerRef, PlayerStaticTarget, Selector, StaticEffect,
    ZoneDest,
};
use crate::game::TurnStep;
use crate::mana::{Color, b, cost, g, generic, r, u, w, x};

fn wurm_token() -> TokenDefinition {
    TokenDefinition {
        name: "Wurm".into(),
        power: 5,
        toughness: 5,
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Collective Blessing — {3}{G}{G}{W} Enchantment. Creatures you control get +3/+3.
pub fn collective_blessing() -> CardDefinition {
    CardDefinition {
        name: "Collective Blessing",
        cost: cost(&[generic(3), g(), g(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +3/+3.",
            effect: StaticEffect::AnthemForFilter {
                filter: R::Creature.and(R::ControlledByYou),
                power: 3,
                toughness: 3,
                keywords: vec![],
                opponents: false,
                only_your_turn: false,
                scale_by_counters_on_self: None,
            },
        }],
        ..Default::default()
    }
}

/// Fall of the Gavel — {3}{W}{U} Instant. Counter target spell. You gain 5 life.
pub fn fall_of_the_gavel() -> CardDefinition {
    CardDefinition {
        name: "Fall of the Gavel",
        cost: cost(&[generic(3), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::Seq(vec![
            Effect::CounterSpell {
                what: target_filtered(R::Any),
            },
            Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(5),
            },
        ]),
        ..Default::default()
    }
}

/// Slime Molding — {X}{G} Sorcery. Create an X/X green Ooze creature token.
pub fn slime_molding() -> CardDefinition {
    CardDefinition {
        name: "Slime Molding",
        cost: cost(&[x(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Ooze".into(),
                power: 0,
                toughness: 0,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Ooze],
                    ..Default::default()
                },
                dynamic_pt: Some((Value::XFromCost, Value::XFromCost)),
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Call of the Conclave — {G}{W} Sorcery. Create a 3/3 green Centaur token.
pub fn call_of_the_conclave() -> CardDefinition {
    CardDefinition {
        name: "Call of the Conclave",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: TokenDefinition {
                name: "Centaur".into(),
                power: 3,
                toughness: 3,
                card_types: vec![CardType::Creature],
                colors: vec![Color::Green],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Centaur],
                    ..Default::default()
                },
                ..Default::default()
            },
        },
        ..Default::default()
    }
}

/// Armada Wurm — {2}{G}{G}{W}{W} 5/5 Wurm with trample. ETB: create a 5/5 green
/// Wurm token with trample.
pub fn armada_wurm() -> CardDefinition {
    CardDefinition {
        name: "Armada Wurm",
        cost: cost(&[generic(2), g(), g(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::ONE,
            definition: wurm_token(),
        })],
        ..Default::default()
    }
}

/// Dark Revenant — {3}{B} 2/2 Spirit with flying. When it dies, put it on top of
/// its owner's library.
pub fn dark_revenant() -> CardDefinition {
    CardDefinition {
        name: "Dark Revenant",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::Move {
            what: Selector::This,
            to: ZoneDest::Library {
                who: PlayerRef::OwnerOfMoved,
                pos: LibraryPosition::Top,
            },
        })],
        ..Default::default()
    }
}

/// Gobbling Ooze — {4}{G} 3/3 Ooze. {G}, Sacrifice another creature: Put a +1/+1
/// counter on this creature.
pub fn gobbling_ooze() -> CardDefinition {
    CardDefinition {
        name: "Gobbling Ooze",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ooze],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g()]),
            sac_other_filter: Some((R::Creature, 1)),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ONE,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seek the Horizon — {3}{G} Sorcery. Search your library for up to three basic
/// land cards, reveal them, put them into your hand, then shuffle.
pub fn seek_the_horizon() -> CardDefinition {
    CardDefinition {
        name: "Seek the Horizon",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::SearchUpToN {
            who: PlayerRef::You,
            filter: R::Land.and(R::HasSupertype(Supertype::Basic)),
            to: ZoneDest::Hand(PlayerRef::You),
            count: Value::Const(3),
        },
        ..Default::default()
    }
}

/// Havoc Festival — {4}{B}{R} Enchantment. Players can't gain life. At the
/// beginning of each player's upkeep, that player loses half their life,
/// rounded up.
pub fn havoc_festival() -> CardDefinition {
    CardDefinition {
        name: "Havoc Festival",
        cost: cost(&[generic(4), b(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: PlayerStaticTarget::EachPlayer,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::LoseHalfLife {
                who: Selector::Player(PlayerRef::ActivePlayer),
                rounded_up: true,
            },
        }],
        ..Default::default()
    }
}

/// Faerie Impostor — {U} 2/1 Faerie Rogue with flying. ETB: sacrifice it unless
/// you return another creature you control to its owner's hand.
pub fn faerie_impostor() -> CardDefinition {
    CardDefinition {
        name: "Faerie Impostor",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        // "Sacrifice it unless you return another creature" — mandatory bounce
        // when you control one, else sacrifice (the Quickling pattern).
        triggered_abilities: vec![etb(Effect::If {
            cond: Predicate::SelectorExists(Selector::EachPermanent(
                R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
            )),
            then: Box::new(Effect::Move {
                what: Selector::Take {
                    inner: Box::new(Selector::EachPermanent(
                        R::Creature.and(R::ControlledByYou).and(R::OtherThanSource),
                    )),
                    count: Box::new(Value::ONE),
                },
                to: ZoneDest::Hand(PlayerRef::OwnerOfMoved),
            }),
            else_: Box::new(Effect::SacrificeSource),
        })],
        ..Default::default()
    }
}

/// Launch Party — {3}{B} Instant. Additional cost: sacrifice a creature. Destroy
/// target creature. Its controller loses 2 life.
pub fn launch_party() -> CardDefinition {
    CardDefinition {
        name: "Launch Party",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Instant],
        // The additional sacrifice is folded into resolution (the Thud/Bone
        // Shards pattern — the engine has no true cast-time additional cost).
        effect: Effect::Seq(vec![
            Effect::Sacrifice {
                who: Selector::You,
                count: Value::ONE,
                filter: R::Creature,
            },
            Effect::Destroy {
                what: target_filtered(R::Creature),
            },
            Effect::LoseLife {
                who: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::Target(0)))),
                amount: Value::Const(2),
            },
        ]),
        ..Default::default()
    }
}

/// Traitorous Instinct — {3}{R} Sorcery. Gain control of target creature until
/// end of turn. Untap it. Until end of turn it gets +2/+0 and gains haste.
pub fn traitorous_instinct() -> CardDefinition {
    CardDefinition {
        name: "Traitorous Instinct",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: Some(PlayerRef::You),
                duration: Duration::EndOfTurn,
            },
            Effect::Untap {
                what: Selector::Target(0),
                up_to: None,
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::Const(2),
                toughness: Value::ZERO,
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
        ]),
        ..Default::default()
    }
}

/// Hypersonic Dragon — {3}{U}{R} 4/4 Dragon with flying and haste. You may cast
/// sorcery spells as though they had flash.
pub fn hypersonic_dragon() -> CardDefinition {
    CardDefinition {
        name: "Hypersonic Dragon",
        cost: cost(&[generic(3), u(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        static_abilities: vec![StaticAbility {
            description: "You may cast sorcery spells as though they had flash.",
            effect: StaticEffect::ControllerSorceriesAsFlash,
        }],
        ..Default::default()
    }
}

/// Carnival Hellsteed — {4}{B}{R} 5/4 Nightmare Horse with first strike, haste,
/// and unleash.
pub fn carnival_hellsteed() -> CardDefinition {
    CardDefinition {
        name: "Carnival Hellsteed",
        cost: cost(&[generic(4), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Horse],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::FirstStrike, Keyword::Haste],
        triggered_abilities: vec![unleash()],
        ..Default::default()
    }
}

/// Azorius Justiciar — {2}{W}{W} 2/2 Human Wizard. ETB: detain up to two target
/// creatures your opponents control.
pub fn azorius_justiciar() -> CardDefinition {
    CardDefinition {
        name: "Azorius Justiciar",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Creature.and(R::ControlledByOpponent),
            effect: Box::new(Effect::Detain {
                what: Selector::Target(0),
            }),
        })],
        ..Default::default()
    }
}

/// Archon of the Triumvirate — {5}{W}{U} 4/5 Archon with flying. Whenever it
/// attacks, detain up to two target nonland permanents your opponents control.
pub fn archon_of_the_triumvirate() -> CardDefinition {
    CardDefinition {
        name: "Archon of the Triumvirate",
        cost: cost(&[generic(5), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Archon],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_attack(Effect::ApplyToTargets {
            max_targets: 2,
            min_targets: 0,
            filter: R::Permanent
                .and(R::Not(Box::new(R::Land)))
                .and(R::ControlledByOpponent),
            effect: Box::new(Effect::Detain {
                what: Selector::Target(0),
            }),
        })],
        ..Default::default()
    }
}

/// Isperia, Supreme Judge — {2}{W}{W}{U}{U} 6/4 Sphinx with flying. Whenever a
/// creature attacks you or a planeswalker you control, you may draw a card.
pub fn isperia_supreme_judge() -> CardDefinition {
    CardDefinition {
        name: "Isperia, Supreme Judge",
        cost: cost(&[generic(2), w(), w(), u(), u()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 6,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::ControllerAttackedByOpponent),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::ONE,
                }),
            },
        }],
        ..Default::default()
    }
}

/// Transguild Promenade — Land. Enters tapped. When it enters, sacrifice it
/// unless you pay {1}. {T}: Add one mana of any color.
pub fn transguild_promenade() -> CardDefinition {
    CardDefinition {
        name: "Transguild Promenade",
        cost: cost(&[]),
        card_types: vec![CardType::Land],
        static_abilities: vec![StaticAbility {
            description: "This land enters tapped.",
            effect: StaticEffect::EntersTapped {
                applies_to: Selector::This,
            },
        }],
        triggered_abilities: vec![etb(Effect::SacrificeSourceUnlessPay {
            cost: cost(&[generic(1)]),
        })],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Psychic Spiral — {4}{U} Instant. Shuffle all cards from your graveyard into
/// your library. Target player mills that many cards.
pub fn psychic_spiral() -> CardDefinition {
    CardDefinition {
        name: "Psychic Spiral",
        cost: cost(&[generic(4), u()]),
        card_types: vec![CardType::Instant],
        // Mill reads the graveyard size before the shuffle empties it; the two
        // halves touch different zones, so ordering is rules-invisible.
        effect: Effect::Seq(vec![
            Effect::Mill {
                who: Selector::Player(PlayerRef::Target(0)),
                amount: Value::GraveyardSizeOf(PlayerRef::You),
            },
            Effect::ShuffleGraveyardIntoLibrary {
                who: PlayerRef::You,
            },
        ]),
        ..Default::default()
    }
}

/// Wild Beastmaster — {2}{G} 1/1 Human Shaman. Whenever it attacks, each other
/// creature you control gets +X/+X until end of turn, where X is its power.
pub fn wild_beastmaster() -> CardDefinition {
    CardDefinition {
        name: "Wild Beastmaster",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_attack(Effect::PumpPT {
            what: Selector::ControlledBy {
                who: PlayerRef::You,
                filter: R::Creature.and(R::OtherThanSource),
            },
            power: Value::PowerOf(Box::new(Selector::This)),
            toughness: Value::PowerOf(Box::new(Selector::This)),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Martial Law — {2}{W}{W} Enchantment. At the beginning of your upkeep, detain
/// target creature an opponent controls.
pub fn martial_law() -> CardDefinition {
    CardDefinition {
        name: "Martial Law",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Detain {
                what: target_filtered(R::Creature.and(R::ControlledByOpponent)),
            },
        }],
        ..Default::default()
    }
}

/// Utvara Hellkite — {6}{R}{R} 6/6 Dragon with flying. Whenever a Dragon you
/// control attacks, create a 6/6 red Dragon token with flying.
pub fn utvara_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Utvara Hellkite",
        cost: cost(&[generic(6), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::HasCreatureType(CreatureType::Dragon),
                },
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::ONE,
                definition: TokenDefinition {
                    name: "Dragon".into(),
                    power: 6,
                    toughness: 6,
                    card_types: vec![CardType::Creature],
                    colors: vec![Color::Red],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Dragon],
                        ..Default::default()
                    },
                    keywords: vec![Keyword::Flying],
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Necropolis Regent — {3}{B}{B}{B} 6/5 Vampire with flying. Whenever a creature
/// you control deals combat damage to a player, put that many +1/+1 counters on it.
pub fn necropolis_regent() -> CardDefinition {
    CardDefinition {
        name: "Necropolis Regent",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::DealsCombatDamageToPlayer,
                EventScope::YourControl,
            ),
            // CR 119.3 — "that many" reads the actual combat damage dealt,
            // carried on the trigger as `Value::TriggerEventAmount`.
            effect: Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::TriggerEventAmount,
            },
        }],
        ..Default::default()
    }
}

/// Trostani, Selesnya's Voice — {G}{G}{W}{W} 2/5 Dryad. Whenever another creature
/// you control enters, gain life equal to its toughness. {1}{G}{W}, {T}: Populate.
pub fn trostani_selesnyas_voice() -> CardDefinition {
    CardDefinition {
        name: "Trostani, Selesnya's Voice",
        cost: cost(&[g(), g(), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: R::Creature.and(R::OtherThanSource),
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::ToughnessOf(Box::new(Selector::TriggerSource)),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g(), w()]),
            tap_cost: true,
            effect: Effect::Populate {
                who: PlayerRef::You,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Wayfaring Temple — {1}{G}{W} */* Elemental. Its P/T each equal the number of
/// creatures you control. Combat damage to a player: populate.
pub fn wayfaring_temple() -> CardDefinition {
    CardDefinition {
        name: "Wayfaring Temple",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        static_abilities: vec![StaticAbility {
            description: "Wayfaring Temple's power and toughness are each equal to the number of creatures you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: R::Creature.and(R::ControlledByYou),
                per_power: 1,
                per_toughness: 1,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Populate {
                who: PlayerRef::You,
            },
        }],
        ..Default::default()
    }
}
