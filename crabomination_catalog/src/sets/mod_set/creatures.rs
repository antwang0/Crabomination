//! Modern-staple creatures and enchantments. Each card uses only existing
//! engine primitives; promotions to fuller Oracle text are noted inline.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Selector, SelectionRequirement, StaticAbility, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

// ── Creatures ────────────────────────────────────────────────────────────────

/// Thalia, Guardian of Thraben — {1}{W}, 2/1 Legendary Human Soldier with
/// First Strike. Static: noncreature spells cost {1} more to cast (every
/// such spell, via `StaticEffect::AdditionalCost`).
pub fn thalia_guardian_of_thraben() -> CardDefinition {
    CardDefinition {
        name: "Thalia, Guardian of Thraben",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells cost {1} more to cast.",
            effect: StaticEffect::AdditionalCost {
                filter: SelectionRequirement::Noncreature,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Dark Confidant — {1}{B}, 2/1 Human Wizard. At the beginning of your
/// upkeep, reveal the top card of your library and put it into your hand.
/// You lose life equal to its mana value.
///
/// Push (modern_decks batch 110): the "lose life equal to CMC" half is now
/// wired correctly via `LoseLife(ManaValueOf(TopOfLibrary))` *before* the
/// draw. Ordering matters: the life loss reads the live top of library at
/// resolution time, then the draw moves that same card to hand. Without
/// this ordering the `ManaValueOf` would see the *new* top card after the
/// draw.
pub fn dark_confidant() -> CardDefinition {
    CardDefinition {
        name: "Dark Confidant",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                // Read CMC from top of library *before* drawing — otherwise
                // the Draw moves the card to hand and `ManaValueOf(TopOfLibrary)`
                // would resolve against the *next* card.
                Effect::LoseLife {
                    who: Selector::You,
                    amount: Value::ManaValueOf(Box::new(Selector::TopOfLibrary {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                    })),
                },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Pridemalkin — {1}{W}, 2/2 Cat with Training (CR 702.149). The
/// "each creature you control with a +1/+1 counter has trample" static
/// is collapsed (kept as a vanilla Training body).
pub fn pridemalkin() -> CardDefinition {
    CardDefinition {
        name: "Pridemalkin",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::training()],
        ..Default::default()
    }
}

/// Aether Adept — {1}{U}{U}, 2/2 Human Wizard. "When this enters, return
/// target creature to its owner's hand."
pub fn aether_adept() -> CardDefinition {
    CardDefinition {
        name: "Aether Adept",
        cost: cost(&[generic(1), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Augury Owl — {1}{U}, 1/1 Bird with Flying. "When this enters, scry 3."
pub fn augury_owl() -> CardDefinition {
    CardDefinition {
        name: "Augury Owl",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(3) },
        }],
        ..Default::default()
    }
}

/// Cloudkin Seer — {2}{U}, 2/2 Elemental with Flying. "When this enters,
/// draw a card."
pub fn cloudkin_seer() -> CardDefinition {
    CardDefinition {
        name: "Cloudkin Seer",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Benthic Biomancer — {U}, 1/1 Merfolk Wizard. `{1}{U}: Adapt 1`
/// (CR 702.108); "whenever this adapts, draw a card, then discard a card."
/// The loot rides the same `If(no +1/+1 counter)` branch as the adapt.
pub fn benthic_biomancer() -> CardDefinition {
    CardDefinition {
        name: "Benthic Biomancer",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::If {
                cond: crate::card::Predicate::Not(Box::new(
                    crate::card::Predicate::EntityMatches {
                        what: Selector::This,
                        filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                    },
                )),
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                    Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
                ])),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Merfolk Skydiver — {1}{U}, 1/1 Merfolk with Flying. `{1}{U}: Adapt 1`
/// (CR 702.108); "whenever this adapts, proliferate." Since Adapt only puts
/// a counter on when there are none, the proliferate rides the same
/// `If(no +1/+1 counter)` branch as the adapt.
pub fn merfolk_skydiver() -> CardDefinition {
    CardDefinition {
        name: "Merfolk Skydiver",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), u()]),
            effect: Effect::If {
                cond: crate::card::Predicate::Not(Box::new(
                    crate::card::Predicate::EntityMatches {
                        what: Selector::This,
                        filter: SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne),
                    },
                )),
                then: Box::new(Effect::Seq(vec![
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                    Effect::Proliferate,
                ])),
                else_: Box::new(Effect::Noop),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Pteramander — {U}, 1/1 Salamander Drake with Flying. `{7}: Adapt 4`
/// (CR 702.108 — put four +1/+1 counters on it if it has none). The
/// printed "{7} costs {1} less per instant/sorcery in your graveyard"
/// rebate is collapsed to the flat {7} (no count-based cost-rebate
/// primitive yet).
pub fn pteramander() -> CardDefinition {
    CardDefinition {
        name: "Pteramander",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander, CreatureType::Drake],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(7)]),
            effect: crate::effect::shortcut::adapt(4),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sylvan Caryatid — {1}{G}, 0/3 Plant. Defender, Hexproof. {T}: Add one
/// mana of any color. Defender is enforced via `can_attack` (CR 702.3b —
/// a creature with Defender can't attack), Hexproof gates opponent
/// targeting, and the `{T}: Add any color` mana ability is wired.
pub fn sylvan_caryatid() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Caryatid",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Hexproof, Keyword::Defender],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::AnyOneColor(Value::Const(1)),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Restoration Angel — {3}{W}, 3/4 Flying, Flash. ETB: exile another target
/// non-Angel creature you control, then return that card to the battlefield
/// under your control. Reuses the Ephemerate `Exile + Move-back` flicker
/// pattern; the filter excludes Angel via
/// `HasCreatureType(Angel).negate()`. Note: the engine's auto-target picks
/// "another" via the same selector that excludes Angels — Restoration
/// Angel itself is a creature you control, but Angel-typed, so the filter
/// already keeps it from self-targeting.
pub fn restoration_angel() -> CardDefinition {
    CardDefinition {
        name: "Restoration Angel",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(
                                SelectionRequirement::HasCreatureType(CreatureType::Angel)
                                    .negate(),
                            ),
                    ),
                },
                Effect::Move {
                    what: Selector::Target(0),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Flickerwisp — {1}{W}{W}, 3/1 Flying Faerie. ETB: exile target permanent;
/// at the beginning of the next end step, return it to the battlefield
/// under its owner's control.
///
/// Wired as `Exile(target) + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf(Target))))`.
/// The captured target slot is preserved on the delayed trigger so the
/// same permanent (now in exile) is returned when the trigger fires.
pub fn flickerwisp() -> CardDefinition {
    CardDefinition {
        name: "Flickerwisp",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Seq(vec![
                Effect::Exile {
                    what: target_filtered(SelectionRequirement::Permanent),
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Move {
                        what: Selector::Target(0),
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                            tapped: false,
                        },
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Loran of the Third Path — {2}{W}, 2/1 Legendary Human Artificer. Vigilance.
/// ETB: destroy target artifact or enchantment. {T}: You and target
/// opponent each draw a card.
///
/// The activated ability requires a player target (the opponent who draws
/// alongside the controller); the engine validates target legality
/// (hexproof/shroud) but doesn't enforce "must be an opponent" — UI
/// constrains the choice in practice.
pub fn loran_of_the_third_path() -> CardDefinition {
    CardDefinition {
        name: "Loran of the Third Path",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Draw { who: Selector::Target(0), amount: Value::Const(1) },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Enchantment),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Sentinel of the Nameless City — {2}{G}, 3/4 Plant Warrior with Vigilance
/// and Ward {2}. Whenever this creature attacks, create a 1/1 green Citizen
/// creature token. Ward {2} is now wired (it's globally enforced at
/// targeting time per CR 702.21) and the Plant subtype is restored
/// (`CreatureType::Plant` is enumerated).
pub fn sentinel_of_the_nameless_city() -> CardDefinition {
    use crate::card::{TokenDefinition, WardCost};
    CardDefinition {
        name: "Sentinel of the Nameless City",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Ward(WardCost::generic(2))],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Citizen".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Green],
                    supertypes: vec![],
                    subtypes: Subtypes::default(),
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                static_abilities: vec![],

                },
            },
        }],
        ..Default::default()
    }
}

/// Ranger-Captain of Eos — {1}{W}{W}, 3/3 Human Soldier. ETB: search your
/// library for a creature card with mana value 1 or less, reveal, put it
/// into your hand, then shuffle. Sacrifice this: your opponents can't cast
/// noncreature spells this turn (`Effect::CantCastNoncreatureThisTurn`).
pub fn ranger_captain_of_eos() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ranger-Captain of Eos",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::EntersBattlefield,
                EventScope::SelfSource,
            ),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ManaValueAtMost(1)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::CantCastNoncreatureThisTurn {
                who: Selector::Player(PlayerRef::EachOpponent),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Cathar Commando — {1}{W}, 3/1 Human Soldier with Flash. {1}, Sacrifice
/// this: Destroy target artifact or enchantment. Uses the new
/// `sac_cost: true` flag so paying the activation cost sacrifices Cathar
/// Commando before its destroy effect resolves.
pub fn cathar_commando() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Cathar Commando",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact
                        .or(SelectionRequirement::Enchantment),
                ),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Haywire Mite — {G}, 1/1 Artifact Creature — Insect. {2}, Sacrifice this
/// artifact: Destroy target artifact, enchantment, or planeswalker. You
/// gain 1 life.
pub fn haywire_mite() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Haywire Mite",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact
                            .or(SelectionRequirement::Enchantment)
                            .or(SelectionRequirement::Planeswalker),
                    ),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Voldaren Epicure — {R}{B}, 1/1 Vampire. ETB: create a Blood token and
/// deal 1 damage to each opponent.
///
/// Blood tokens carry their canonical loot ability via
/// `TokenDefinition::activated_abilities`, so they enter as functional
/// loot artifacts (not just colorless flavor).
pub fn voldaren_epicure() -> CardDefinition {
    use crate::game::effects::blood_token;
    use crate::mana::r;
    CardDefinition {
        name: "Voldaren Epicure",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: blood_token(),
                },
                Effect::ForEach {
                    selector: Selector::Player(PlayerRef::EachOpponent),
                    body: Box::new(Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::Const(1),
                    }),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Goldspan Dragon — {3}{R}{R}, 4/4 Dragon with Flying and Haste. Whenever
/// this attacks or becomes the target of a spell, create a Treasure token.
/// Goldspan's Treasures tap+sac for two mana of one color
/// (`goldspan_treasure_token`). The static is modeled on the Treasures it
/// mints (the common case) rather than retagging every Treasure you control.
pub fn goldspan_dragon() -> CardDefinition {
    use crate::game::effects::goldspan_treasure_token;
    use crate::mana::r;
    let make_treasure = || Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(1),
        definition: goldspan_treasure_token(),
    };
    CardDefinition {
        name: "Goldspan Dragon",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: make_treasure(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                effect: make_treasure(),
            },
        ],
        ..Default::default()
    }
}

/// Tireless Tracker — {1}{G}{G}, 3/2 Human Scout. Whenever a land enters
/// under your control, investigate (create a Clue token).
///
/// Wired via the new trigger-filter enforcement: scope is
/// `YourControl + EntersBattlefield`, filter is
/// `Predicate::EntityMatches { what: TriggerSource, filter: Land }` so
/// the trigger fires only for land-typed entrants. The "Whenever you
/// sacrifice a Clue, put a +1/+1 counter on Tireless Tracker" half is
/// now wired via a `PermanentSacrificed + YourControl` trigger filtered
/// on the sacrificed permanent being a Clue (HasArtifactSubtype(Clue)).
pub fn tireless_tracker() -> CardDefinition {
    use crate::card::{ArtifactSubtype, CounterType};
    use crate::effect::Predicate;
    use crate::game::effects::clue_token;
    CardDefinition {
        name: "Tireless Tracker",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Land,
                    }),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: clue_token(),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::PermanentSacrificed, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Clue),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Bloodtithe Harvester — {1}{B}{R}, 3/2 Vampire Rogue. Whenever this
/// enters or attacks, create a Blood token.
///
/// The activated ability `{1}, Sacrifice a Blood: deals 2 damage to any
/// target` is now wired via `sac_other_filter:
/// HasArtifactSubtype(Blood)` — the sac-of-another-permanent activation
/// cost. Both ETB and attack triggers fire.
pub fn bloodtithe_harvester() -> CardDefinition {
    use crate::card::{ActivatedAbility, ArtifactSubtype};
    use crate::effect::shortcut::target_any;
    use crate::game::effects::blood_token;
    use crate::mana::r;
    let blood_etb = TriggeredAbility {
        event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: blood_token(),
        },
    };
    let blood_attack = TriggeredAbility {
        event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
        effect: Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: blood_token(),
        },
    };
    CardDefinition {
        name: "Bloodtithe Harvester",
        cost: cost(&[generic(1), b(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::DealDamage {
                to: target_any(),
                amount: Value::Const(2),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            // {1}, Sacrifice a Blood: deals 2 damage to any target.
            sac_other_filter: Some((
                SelectionRequirement::HasArtifactSubtype(ArtifactSubtype::Blood),
                1,
            )),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![blood_etb, blood_attack],
        ..Default::default()
    }
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Phyrexian Arena — {1}{B}{B} Enchantment. At the beginning of your upkeep,
/// draw a card and lose 1 life.
pub fn phyrexian_arena() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Arena",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Up the Beanstalk — {1}{G} Enchantment. When this enters, draw a card.
/// Whenever you cast a spell with mana value 5 or greater, draw a card.
///
/// The mana-value-5+ trigger is gated on `EventSpec::filter` =
/// `Predicate::ValueAtLeast(ManaValueOf(TriggerSource), 5)`. The
/// dispatcher binds the cast spell as `TriggerSource`, and the extended
/// `Value::ManaValueOf` lookup walks the stack so the filter can read the
/// mana value of a spell that's still on the stack.
pub fn up_the_beanstalk() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Up the Beanstalk",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![
            // ETB: draw a card.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::EntersBattlefield,
                    EventScope::SelfSource,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
            // Whenever you cast a spell with mana value ≥ 5, draw a card.
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                        Value::Const(5),
                    )),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

/// Tishana's Tidebinder — {1}{U}{U}, 3/2 Merfolk Wizard with Flash. ETB:
/// counter target activated or triggered ability of an artifact, creature,
/// enchantment, or planeswalker (a "nonland permanent" — Battles aren't
/// modeled).
///
/// Reuses `Effect::CounterAbility` (which Consign to Memory introduced),
/// targeting any nonland permanent and removing the topmost
/// `StackItem::Trigger` whose source matches. Auto-target picks the
/// most-recent opponent permanent's pending trigger first (via the
/// stack-aware fallback in `auto_target_for_effect`).
pub fn tishanas_tidebinder() -> CardDefinition {
    use crate::card::TriggeredAbility;
    use crate::effect::shortcut::target_filtered;
    use crate::mana::u;
    CardDefinition {
        name: "Tishana's Tidebinder",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CounterAbility {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Sylvan Safekeeper — {G}, 1/1 Human Wizard. Sacrifice a Forest: Target
/// creature gains shroud until end of turn.
///
/// The "Sacrifice a Forest" cost is now a proper pre-resolution
/// activation cost via `sac_other_filter: Some((Forest, 1))` — the
/// engine gates the activation on the controller actually owning a
/// Forest to sacrifice (rejecting cleanly otherwise) instead of folding
/// the sacrifice into resolution.
pub fn sylvan_safekeeper() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    CardDefinition {
        name: "Sylvan Safekeeper",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Shroud,
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None,
            // Sacrifice a Forest as an activation cost.
            sac_other_filter: Some((
                SelectionRequirement::Land
                    .and(SelectionRequirement::HasLandType(LandType::Forest)),
                1,
            )),
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Grim Lavamancer — {R}, 1/1 Human Wizard. {R}, {T}, Exile two cards from
/// your graveyard: deals 2 damage to any target. The exile-two cost is a
/// real activation cost (`exile_other_filter: (Any, 2)`) gated on having ≥ 2
/// other graveyard cards; the two are auto-picked lowest-CMC first.
pub fn grim_lavamancer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    use crate::mana::r;
    CardDefinition {
        name: "Grim Lavamancer",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[r()]),
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Any),
                amount: Value::Const(2),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            // Additional cost: exile two cards from your graveyard.
            exile_other_filter: Some((SelectionRequirement::Any, 2)),
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![],
        ..Default::default()
    }
}

/// Containment Priest — {1}{W}, 2/2 Human Cleric Flash. "If a nontoken
/// creature would enter the battlefield and it wasn't cast, exile it
/// instead" via `StaticEffect::ExileNontokenCreaturesNotCast`.
pub fn containment_priest() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Containment Priest",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If a nontoken creature would enter the battlefield and it wasn't cast, exile it instead.",
            effect: StaticEffect::ExileNontokenCreaturesNotCast,
        }],
        ..Default::default()
    }
}

/// Torpor Orb — {2} Artifact. "Creatures entering the battlefield don't
/// cause triggered abilities to trigger." (`SuppressCreatureEtbTriggers`.)
pub fn torpor_orb() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Torpor Orb",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        static_abilities: vec![StaticAbility {
            description: "Creatures entering the battlefield don't cause triggered abilities to trigger.",
            effect: StaticEffect::SuppressCreatureEtbTriggers { also_dies: false },
        }],
        ..Default::default()
    }
}

/// Tocatli Honor Guard — {1}{W}, 1/3 Human Soldier. Same ETB-trigger
/// suppression as Torpor Orb, on a creature body.
pub fn tocatli_honor_guard() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Tocatli Honor Guard",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Creatures entering the battlefield don't cause triggered abilities to trigger.",
            effect: StaticEffect::SuppressCreatureEtbTriggers { also_dies: false },
        }],
        ..Default::default()
    }
}

/// Hushbringer — {1}{W}, 1/2 Faerie with Flying and Lifelink. "Creatures
/// entering the battlefield or dying don't cause triggered abilities to
/// trigger."
pub fn hushbringer() -> CardDefinition {
    use crate::effect::{StaticAbility, StaticEffect};
    CardDefinition {
        name: "Hushbringer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        static_abilities: vec![StaticAbility {
            description: "Creatures entering the battlefield or dying don't cause triggered abilities to trigger.",
            effect: StaticEffect::SuppressCreatureEtbTriggers { also_dies: true },
        }],
        ..Default::default()
    }
}

/// Journey to Nowhere — {1}{W} Enchantment. ETB: exile target creature.
/// When Journey to Nowhere leaves the battlefield, return that card.
pub fn journey_to_nowhere() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Journey to Nowhere",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(SelectionRequirement::Creature),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Banishing Light — {2}{W} Enchantment. ETB: exile target nonland
/// permanent an opponent controls until Banishing Light leaves.
pub fn banishing_light() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Banishing Light",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileUntilSourceLeaves {
                what: target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                return_to: ExileReturnZone::Battlefield,
            },
        }],
        ..Default::default()
    }
}

/// Seal of Cleansing — {1}{W} Enchantment. "Sacrifice this: Destroy target
/// artifact or enchantment."
pub fn seal_of_cleansing() -> CardDefinition {
    CardDefinition {
        name: "Seal of Cleansing",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Soul Warden — {W}, 1/1 Human Cleric. "Whenever another creature enters,
/// you gain 1 life."
pub fn soul_warden() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Soul Warden",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Essence Warden — {G}, 1/1 Elf Shaman. "Whenever another creature enters,
/// you gain 1 life." (Soul Warden in green.)
pub fn essence_warden() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Essence Warden",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Silhana Ledgewalker — {1}{G}, 1/1 Elf Rogue. "Hexproof. Silhana
/// Ledgewalker can't be blocked except by creatures with flying." (RAV)
pub fn silhana_ledgewalker() -> CardDefinition {
    CardDefinition {
        name: "Silhana Ledgewalker",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![
            Keyword::Hexproof,
            Keyword::CantBeBlockedExceptBy(Box::new(SelectionRequirement::HasKeyword(
                Keyword::Flying,
            ))),
        ],
        ..Default::default()
    }
}

/// Steel Leaf Champion — {G}{G}{G}, 5/4 Elf Knight. "Steel Leaf Champion
/// can't be blocked by creatures with power 2 or less." (DAR)
pub fn steel_leaf_champion() -> CardDefinition {
    CardDefinition {
        name: "Steel Leaf Champion",
        cost: cost(&[g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Knight],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::CantBeBlockedBy(Box::new(
            SelectionRequirement::PowerAtMost(2),
        ))],
        ..Default::default()
    }
}

// ── modern_decks batch: high-confidence value bodies ────────────────────────

/// Cloudblazer — {3}{U}{U}, 2/2 Flying. "When this enters, you draw two
/// cards and you gain two life." (KLD)
pub fn cloudblazer() -> CardDefinition {
    CardDefinition {
        name: "Cloudblazer",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Scout], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        ..Default::default()
    }
}

/// Invisible Stalker — {1}{U}, 1/1. "Hexproof. Invisible Stalker can't be
/// blocked." (ISD)
pub fn invisible_stalker() -> CardDefinition {
    CardDefinition {
        name: "Invisible Stalker",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Rogue], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Hexproof, Keyword::Unblockable],
        ..Default::default()
    }
}

/// Slither Blade — {1}{U}, 1/1. "Slither Blade can't be blocked." (AKH)
pub fn slither_blade() -> CardDefinition {
    CardDefinition {
        name: "Slither Blade",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Unblockable],
        ..Default::default()
    }
}

/// Mistral Charger — {1}{W}, 2/1 Flying. (ORI)
pub fn mistral_charger() -> CardDefinition {
    CardDefinition {
        name: "Mistral Charger",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Vorstclaw — {4}{G}{G}, 6/4 Elemental. Vanilla green top-end. (AVR)
pub fn vorstclaw() -> CardDefinition {
    CardDefinition {
        name: "Vorstclaw",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 6,
        toughness: 4,
        ..Default::default()
    }
}

/// Shadowmage Infiltrator — {1}{U}{B}, 1/3 Fear. "Whenever this deals combat
/// damage to a player, you may draw a card." (DIS)
pub fn shadowmage_infiltrator() -> CardDefinition {
    CardDefinition {
        name: "Shadowmage Infiltrator",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Wizard], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Fear],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Liliana's Specter — {2}{B}{B}, 2/1 Flying. "When this enters, each
/// opponent discards a card." (M11)
pub fn lilianas_specter() -> CardDefinition {
    CardDefinition {
        name: "Liliana's Specter",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Specter], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(1),
                random: true,
            },
        }],
        ..Default::default()
    }
}

/// Bone Shredder — {2}{B}, 1/1 Flying. Echo {1}{B}. "When this enters,
/// destroy target nonartifact, nonblack creature." (UDS)
pub fn bone_shredder() -> CardDefinition {
    CardDefinition {
        name: "Bone Shredder",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Minion], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Echo(cost(&[generic(1), b()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Artifact.negate())
                        .and(SelectionRequirement::HasColor(crate::mana::Color::Black).negate()),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Goldnight Commander — {3}{W}, 2/2. "Whenever another creature you control
/// enters, creatures you control get +1/+1 until end of turn." (AVR)
pub fn goldnight_commander() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Goldnight Commander",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Elvish Archdruid — {1}{G}{G}, 2/2 Elf Druid. "Other Elf creatures you
/// control get +1/+1. {T}: Add {G} for each Elf you control." (M10)
pub fn elvish_archdruid() -> CardDefinition {
    CardDefinition {
        name: "Elvish Archdruid",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Druid], ..Default::default() },
        power: 2,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Other Elf creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Elf)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    crate::mana::Color::Green,
                    Value::count(Selector::EachPermanent(
                        SelectionRequirement::HasCreatureType(CreatureType::Elf)
                            .and(SelectionRequirement::ControlledByYou),
                    )),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Priest of Titania — {1}{G}, 1/1 Elf Druid. "{T}: Add {G} for each Elf on
/// the battlefield." (various)
pub fn priest_of_titania() -> CardDefinition {
    CardDefinition {
        name: "Priest of Titania",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Druid], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    crate::mana::Color::Green,
                    Value::count(Selector::EachPermanent(SelectionRequirement::HasCreatureType(
                        CreatureType::Elf,
                    ))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sylvan Ranger — {1}{G}, 1/1 Elf Scout. "When this enters, search your
/// library for a basic land card, reveal it, put it into your hand, then
/// shuffle." (various)
pub fn sylvan_ranger() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Ranger",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Scout], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Civic Wayfinder — {2}{G}, 2/2 Elf Warrior. "When this enters, search your
/// library for a basic land card, reveal it, put it into your hand, then
/// shuffle." (RAV)
pub fn civic_wayfinder() -> CardDefinition {
    CardDefinition {
        name: "Civic Wayfinder",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Elvish Warrior — {G}{G}, 2/3 Elf Warrior. Vanilla Elf body (Archdruid /
/// Priest of Titania payoff). (ONS)
pub fn elvish_warrior() -> CardDefinition {
    CardDefinition {
        name: "Elvish Warrior",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elf, CreatureType::Warrior], ..Default::default() },
        power: 2,
        toughness: 3,
        ..Default::default()
    }
}

/// Welkin Tern — {1}{U}, 2/1 Flying. "Welkin Tern can't block." (ISD)
pub fn welkin_tern() -> CardDefinition {
    CardDefinition {
        name: "Welkin Tern",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::CantBlock],
        ..Default::default()
    }
}

/// Aerial Responder — {1}{W}{W}, 2/3 Dwarf Soldier. Flying, Vigilance,
/// Lifelink. (KLD)
pub fn aerial_responder() -> CardDefinition {
    CardDefinition {
        name: "Aerial Responder",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dwarf, CreatureType::Soldier], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Knight of Meadowgrain — {1}{W}, 2/2 Kithkin Knight. First strike,
/// Lifelink. (LRW)
pub fn knight_of_meadowgrain() -> CardDefinition {
    CardDefinition {
        name: "Knight of Meadowgrain",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kithkin, CreatureType::Knight], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Skyhunter Patrol — {2}{W}, 2/2 Cat Knight. Flying, First strike. (MRD)
pub fn skyhunter_patrol() -> CardDefinition {
    CardDefinition {
        name: "Skyhunter Patrol",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat, CreatureType::Knight], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Veteran Armorer — {1}{W}, 2/3 Human Soldier. "Other creatures you control
/// get +0/+1." (M10)
pub fn veteran_armorer() -> CardDefinition {
    CardDefinition {
        name: "Veteran Armorer",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Soldier], ..Default::default() },
        power: 2,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +0/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 0,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Attended Knight — {2}{W}, 2/2 Human Knight. First strike. "When this
/// enters, create a 1/1 white Soldier creature token." (M13)
pub fn attended_knight() -> CardDefinition {
    CardDefinition {
        name: "Attended Knight",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Knight], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::card::TokenDefinition {
                    name: "Soldier".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::White],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Kor Hookmaster — {2}{W}, 2/2 Kor Soldier. "When this enters, tap target
/// creature an opponent controls. That creature doesn't untap during its
/// controller's next untap step." (ZEN)
pub fn kor_hookmaster() -> CardDefinition {
    CardDefinition {
        name: "Kor Hookmaster",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Kor, CreatureType::Soldier], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Dusk Legion Zealot — {B}{B}, 1/1 Vampire Soldier. "When this enters, you
/// draw a card and you lose 1 life." (RIX)
pub fn dusk_legion_zealot() -> CardDefinition {
    CardDefinition {
        name: "Dusk Legion Zealot",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Vampire, CreatureType::Soldier], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Phyrexian Gargantua — {4}{B}{B}, 4/4 Horror. "When this enters, you draw
/// two cards and you lose two life." (various)
pub fn phyrexian_gargantua() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Gargantua",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(2) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        ..Default::default()
    }
}

/// Frost Lynx — {2}{U}, 2/2 Cat. "When this enters, tap target creature an
/// opponent controls. That creature doesn't untap during its controller's
/// next untap step." (M15)
pub fn frost_lynx() -> CardDefinition {
    CardDefinition {
        name: "Frost Lynx",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Indrik Stomphowler — {4}{G}, 4/4 Beast. "When this enters, destroy target
/// artifact or enchantment." (M10)
pub fn indrik_stomphowler() -> CardDefinition {
    CardDefinition {
        name: "Indrik Stomphowler",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Ambassador Oak — {3}{G}, 2/2 Treefolk Shaman. "When this enters, create a
/// 1/1 green Elf Warrior creature token." (SHM)
pub fn ambassador_oak() -> CardDefinition {
    CardDefinition {
        name: "Ambassador Oak",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk, CreatureType::Shaman], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::card::TokenDefinition {
                    name: "Elf Warrior".into(),
                    power: 1,
                    toughness: 1,
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Green],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            },
        }],
        ..Default::default()
    }
}

/// Nessian Asp — {4}{G}, 4/5 Snake. "{6}{G}: Monstrosity 4." (THS)
pub fn nessian_asp() -> CardDefinition {
    CardDefinition {
        name: "Nessian Asp",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 4,
        toughness: 5,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(6), g()]),
            effect: Effect::Monstrosity { n: Value::Const(4) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Charging Rhino — {3}{G}{G}, 4/4 Rhino. "Charging Rhino can't be blocked
/// by more than one creature." (various)
pub fn charging_rhino() -> CardDefinition {
    CardDefinition {
        name: "Charging Rhino",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Rhino], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::CantBeBlockedByMoreThanOne],
        ..Default::default()
    }
}

/// Llanowar Visionary — {2}{G}, 2/2 Elf Druid. ETB: draw a card. "{T}: Add
/// {G}."
pub fn llanowar_visionary() -> CardDefinition {
    CardDefinition {
        name: "Llanowar Visionary",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![crate::mana::Color::Green]),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Augur of Bolas — {1}{U}, 1/3 Merfolk Wizard. ETB: look at the top three
/// cards; put an instant or sorcery from among them into your hand (rest to
/// the bottom — approximated by leaving them on top).
pub fn augur_of_bolas() -> CardDefinition {
    CardDefinition {
        name: "Augur of Bolas",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(3),
                rest_to_graveyard: false,
                pick_filter: None,
            
                take: None,
            },
        }],
        ..Default::default()
    }
}

/// Pestermite — {2}{U}, 2/1 Faerie with Flash and Flying. ETB: tap target
/// permanent. (The "it doesn't untap" rider is omitted.)
pub fn pestermite() -> CardDefinition {
    CardDefinition {
        name: "Pestermite",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Permanent) },
        }],
        ..Default::default()
    }
}

/// Suture Priest — {1}{W}, 1/1 Cleric. "Whenever another creature you
/// control enters, you gain 1 life." "Whenever a creature an opponent
/// controls enters, that player loses 1 life."
pub fn suture_priest() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Suture Priest",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::OtherThanSource),
                    }),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                    .with_filter(Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    }),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Knight of Autumn — {1}{G}{W}, 2/1 Dryad Knight. ETB — choose one: gain
/// 4 life; destroy target artifact or enchantment; or put two +1/+1
/// counters on this.
pub fn knight_of_autumn() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Knight of Autumn",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                Effect::GainLife { who: Selector::You, amount: Value::Const(4) },
                Effect::Destroy {
                    what: target_filtered(
                        SelectionRequirement::Artifact
                            .or(SelectionRequirement::Enchantment),
                    ),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Spark Double — {3}{U}, 0/0 Shapeshifter. "You may have Spark Double
/// enter as a copy of a creature you control, except it enters with an
/// additional +1/+1 counter on it." (The planeswalker-copy half is
/// omitted.) The extra counter rides an appended ETB trigger, which the
/// CR-707.5 copy path fires.
pub fn spark_double() -> CardDefinition {
    use crate::card::{CounterType, EntersAsCopy};
    CardDefinition {
        name: "Spark Double",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            extra_creature_types: vec![],
            extra_keywords: vec![],
            keep_name: false,
            extra_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            }],
            extra_card_types: vec![],
        }),
        ..Default::default()
    }
}

/// Reflector Mage — {1}{W}{U}, 2/3 Human Wizard. ETB: return target
/// creature an opponent controls to its owner's hand. (The "can't recast
/// until your next turn" rider is omitted.)
pub fn reflector_mage() -> CardDefinition {
    CardDefinition {
        name: "Reflector Mage",
        cost: cost(&[generic(1), w(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Man-o'-War — {2}{U}, 2/2 Jellyfish. ETB: return target creature to its
/// owner's hand.
pub fn man_o_war() -> CardDefinition {
    CardDefinition {
        name: "Man-o'-War",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Jellyfish],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Creature),
                to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
            },
        }],
        ..Default::default()
    }
}

/// Siege-Gang Commander — {3}{R}{R}, 2/2 Goblin. ETB: create three 1/1 red
/// Goblin tokens. "{1}{R}, Sacrifice a Goblin: it deals 2 damage to any
/// target."
pub fn siege_gang_commander() -> CardDefinition {
    use crate::card::TokenDefinition;
    let goblin = TokenDefinition {
        name: "Goblin".into(),
        power: 1,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Red],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Siege-Gang Commander",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(3),
                definition: goblin,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), r()]),
            sac_other_filter: Some((
                SelectionRequirement::HasCreatureType(CreatureType::Goblin),
                1,
            )),
            effect: Effect::DealDamage { to: Selector::Target(0), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sea Gate Oracle — {2}{U}, 1/3 Human Wizard. ETB: look at the top two
/// cards, put one into your hand. (The "other on the bottom" half is
/// approximated by leaving it on top.)
pub fn sea_gate_oracle() -> CardDefinition {
    CardDefinition {
        name: "Sea Gate Oracle",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::LookPickToHand {
                who: PlayerRef::You,
                count: Value::Const(2),
                rest_to_graveyard: false,
                pick_filter: None,
            
                take: None,
            },
        }],
        ..Default::default()
    }
}

/// Fertilid — {2}{G}, 1/1 Elemental that enters with two +1/+1 counters.
/// "{1}{G}, Remove a +1/+1 counter from this: Search your library for a
/// basic land card, put it onto the battlefield tapped, then shuffle."
pub fn fertilid() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Fertilid",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(1), g()]),
            condition: Some(crate::effect::Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
                Value::Const(1),
            )),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Inexorable Tide — {3}{U} Enchantment. "Whenever you cast a spell,
/// proliferate."
pub fn inexorable_tide() -> CardDefinition {
    CardDefinition {
        name: "Inexorable Tide",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Thrummingbird — {1}{U}, 1/1 Bird with Flying. "Whenever Thrummingbird
/// deals combat damage to a player, proliferate."
pub fn thrummingbird() -> CardDefinition {
    CardDefinition {
        name: "Thrummingbird",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Proliferate,
        }],
        ..Default::default()
    }
}

/// Spike Feeder — {2}{G}, 0/0 Spike that enters with two +1/+1 counters.
/// "Remove a +1/+1 counter from this: you gain 2 life."
pub fn spike_feeder() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Spike Feeder",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spike],
            ..Default::default()
        },
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            // "Remove a +1/+1 counter" is the cost — gate on having one so
            // the lifegain can't fire off an empty creature.
            condition: Some(crate::effect::Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
                Value::Const(1),
            )),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
                Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Sunhome Stalwart — {1}{W}, 2/2 Human Soldier with First strike + Mentor
/// (CR 702.134, via `shortcut::mentor()`).
pub fn sunhome_stalwart() -> CardDefinition {
    CardDefinition {
        name: "Sunhome Stalwart",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![crate::effect::shortcut::mentor()],
        ..Default::default()
    }
}

/// Clone — {3}{U}, 0/0 Shapeshifter. "You may have Clone enter the
/// battlefield as a copy of any creature on the battlefield." Wired via
/// the `enters_as_copy` CR-707 hook; with no creature to copy the 0/0
/// dies to SBA (the printed "you may" decline).
pub fn clone_card() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Clone",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature,
            extra_creature_types: vec![],
            extra_triggered: vec![],
            extra_keywords: vec![],
            keep_name: false,
            extra_card_types: vec![],
        }),
        ..Default::default()
    }
}

/// Phyrexian Metamorph — {3}{U/P}, 0/0 Phyrexian Shapeshifter artifact.
/// "You may have this enter as a copy of any artifact or creature, except
/// it's an artifact in addition to its other types." Rides the
/// `enters_as_copy` hook with an artifact/creature filter + `extra_card_types`.
pub fn phyrexian_metamorph() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Phyrexian Metamorph",
        cost: cost(&[generic(3), crate::mana::phyrexian(crate::mana::Color::Blue)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Artifact),
            extra_creature_types: vec![],
            extra_triggered: vec![],
            extra_keywords: vec![],
            keep_name: false,
            extra_card_types: vec![CardType::Artifact],
        }),
        ..Default::default()
    }
}

/// Mirror Image — {1}{U}, 0/0 Shapeshifter. "You may have Mirror Image
/// enter the battlefield as a copy of a creature you control." (The
/// "except it's not legendary" rider is omitted — no copy-supertype
/// override yet.)
pub fn mirror_image() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Mirror Image",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            extra_creature_types: vec![],
            extra_triggered: vec![],
            extra_keywords: vec![],
            keep_name: false,
            extra_card_types: vec![],
        }),
        ..Default::default()
    }
}

/// Stunt Double — {3}{U}, 0/0 Shapeshifter with Flash. "You may have Stunt
/// Double enter as a copy of any creature, except it has flash." The copy
/// keeps Flash via `extra_keywords` (printed Flash also lets it be cast at
/// instant speed).
pub fn stunt_double() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Stunt Double",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature,
            extra_creature_types: vec![],
            extra_triggered: vec![],
            extra_keywords: vec![Keyword::Flash],
            keep_name: false,
            extra_card_types: vec![],
        }),
        ..Default::default()
    }
}

/// Phantasmal Image — {U}, 0/0 Illusion. Enters as a copy of any creature,
/// except it's an Illusion in addition to its types and gains "When this
/// becomes the target of a spell or ability, sacrifice it." Both the
/// extra type and the sacrifice rider ride on the `enters_as_copy` hook.
pub fn phantasmal_image() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Phantasmal Image",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Illusion],
            ..Default::default()
        },
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature,
            extra_creature_types: vec![CreatureType::Illusion],
            extra_triggered: vec![TriggeredAbility {
                event: EventSpec::new(EventKind::BecameTarget, EventScope::SelfSource),
                effect: Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Graveyard,
                },
            }],
            extra_keywords: vec![],
            keep_name: false,
            extra_card_types: vec![],
        }),
        ..Default::default()
    }
}

/// Mockingbird — {1}{U}, 0/0 Shapeshifter. Flash. May enter as a copy of a
/// creature you control, except its name stays "Mockingbird" (CR 707.2
/// name-retention via `EntersAsCopy.keep_name`).
pub fn mockingbird() -> CardDefinition {
    use crate::card::EntersAsCopy;
    CardDefinition {
        name: "Mockingbird",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Shapeshifter],
            ..Default::default()
        },
        keywords: vec![Keyword::Flash],
        enters_as_copy: Some(EntersAsCopy {
            filter: SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou),
            extra_creature_types: vec![],
            extra_triggered: vec![],
            extra_keywords: vec![],
            keep_name: true,
            extra_card_types: vec![],
        }),
        ..Default::default()
    }
}

/// Simian Spirit Guide — {2}{R}, 2/2 Ape Spirit. Ships as the vanilla body;
/// the "exile from hand: add {R}" mana ability needs a from-hand activation
/// zone (tracked in TODO.md).
pub fn simian_spirit_guide() -> CardDefinition {
    use crate::card::CreatureType as CT;
    use crate::effect::shortcut::add_mana;
    use crate::mana::Color;
    CardDefinition {
        name: "Simian Spirit Guide",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CT::Ape, CT::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        // Exile this card from your hand: Add {R}.
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            effect: add_mana(vec![Color::Red]),
            from_hand: true,
            exile_self_cost: true,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Eternal Witness — {1}{G}{G}, 2/1 Human Shaman. ETB: return target card
/// from your graveyard to your hand. Pure recursion.
pub fn eternal_witness() -> CardDefinition {
    CardDefinition {
        name: "Eternal Witness",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // Filter excludes players — `Selector::TargetFiltered` with
            // `Not(Player)` matches any card (in battlefield or
            // graveyard, whatever evaluate_requirement_static walks).
            // `auto_target_for_effect` falls through to graveyards when
            // no battlefield permanent matches.
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Player.negate()),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Heliod, Sun-Crowned — {1}{W}{W}, Legendary Enchantment Creature — God.
/// 5/5. Indestructible. (Heliod, Sun-Crowned has no devotion clause —
/// it's always a creature.) {1}{W}: target creature gains lifelink until
/// end of turn. Whenever you gain life, put a +1/+1 counter on target
/// creature you control with lifelink.
pub fn heliod_sun_crowned() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Heliod, Sun-Crowned",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature, CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Indestructible],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(SelectionRequirement::Creature),
                keyword: Keyword::Lifelink,
                duration: crate::effect::Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        triggered_abilities: vec![TriggeredAbility {
            // "Whenever you gain life, put a +1/+1 counter on target
            // creature you control with lifelink." Pairs with the
            // activated ability and Walking Ballista to win on the spot.
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::HasKeyword(Keyword::Lifelink)),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Indulgent Tormentor — {3}{B}{B}, 5/3 Demon Flying. At the beginning of
/// your upkeep, draw a card unless an opponent sacrifices a creature or
/// pays 3 life. Wired via `Effect::Punisher`: each opponent dodges the
/// draw by paying 3 life (if it leaves them alive) or sacrificing a
/// creature; otherwise the controller draws.
pub fn indulgent_tormentor() -> CardDefinition {
    CardDefinition {
        name: "Indulgent Tormentor",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Punisher {
                chooser: Selector::Player(PlayerRef::EachOpponent),
                options: vec![
                    Effect::LoseLife { who: Selector::Player(PlayerRef::You), amount: Value::Const(3) },
                    Effect::Sacrifice {
                        who: Selector::Player(PlayerRef::You),
                        count: Value::Const(1),
                        filter: SelectionRequirement::Creature,
                    },
                ],
                otherwise: Box::new(Effect::Draw {
                    who: Selector::Player(PlayerRef::You),
                    amount: Value::Const(1),
                }),
            },
        }],
        ..Default::default()
    }
}

/// Basilica Screecher — {2}{B} 1/2 Bat. Flying, Extort (CR 702.99).
pub fn basilica_screecher() -> CardDefinition {
    CardDefinition {
        name: "Basilica Screecher",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bat],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::extort()],
        ..Default::default()
    }
}

/// Zhur-Taa Goblin — {R}{G} 2/2 Goblin Berserker with Riot (CR 702.137):
/// enters with a +1/+1 counter or haste (your choice).
pub fn zhur_taa_goblin() -> CardDefinition {
    CardDefinition {
        name: "Zhur-Taa Goblin",
        cost: cost(&[r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Berserker],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::riot()],
        ..Default::default()
    }
}

/// Syndic of Tithes — {1}{W} 2/3 Human Cleric with Extort.
pub fn syndic_of_tithes() -> CardDefinition {
    CardDefinition {
        name: "Syndic of Tithes",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::extort()],
        ..Default::default()
    }
}

/// Tithe Drinker — {W}{B} 1/2 Vampire Cleric with Lifelink and Extort.
pub fn tithe_drinker() -> CardDefinition {
    CardDefinition {
        name: "Tithe Drinker",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        triggered_abilities: vec![crate::effect::shortcut::extort()],
        ..Default::default()
    }
}

/// Kingpin's Pet — {1}{W}{B} 2/2 Imp with Flying and Extort.
pub fn kingpins_pet() -> CardDefinition {
    CardDefinition {
        name: "Kingpin's Pet",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::extort()],
        ..Default::default()
    }
}

/// Frenzied Arynx — {4}{R}{G} 4/3 Cat Beast with Riot. {3}{R}{G}: it gets
/// +2/+2 until end of turn.
pub fn frenzied_arynx() -> CardDefinition {
    CardDefinition {
        name: "Frenzied Arynx",
        cost: cost(&[generic(4), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::riot()],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(3), r(), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fleshbag Marauder — {2}{B} 2/2 Zombie. ETB: each player sacrifices a
/// creature.
pub fn fleshbag_marauder() -> CardDefinition {
    CardDefinition {
        name: "Fleshbag Marauder",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachPlayer),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature,
        })],
        ..Default::default()
    }
}

/// Kor Skyfisher — {1}{W} 2/3 Kor Soldier, Flying. ETB: return a permanent
/// you control to its owner's hand.
pub fn kor_skyfisher() -> CardDefinition {
    CardDefinition {
        name: "Kor Skyfisher",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Permanent.and(SelectionRequirement::ControlledByYou),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Skyknight Legionnaire — {1}{R}{W} 2/2 Human Knight with Flying and Haste.
pub fn skyknight_legionnaire() -> CardDefinition {
    CardDefinition {
        name: "Skyknight Legionnaire",
        cost: cost(&[generic(1), r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        ..Default::default()
    }
}

/// Mogg Fanatic — {R} 1/1 Goblin. Sacrifice this: deal 1 damage to any target.
pub fn mogg_fanatic() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Mogg Fanatic",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spectral Sailor — {U} 1/1 Spirit with Flash and Flying. {3}{U}, {T}:
/// draw a card.
pub fn spectral_sailor() -> CardDefinition {
    CardDefinition {
        name: "Spectral Sailor",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(3), u()]),
            tap_cost: true,
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Healer's Hawk — {W} 1/1 Bird with Flying and Lifelink.
pub fn healers_hawk() -> CardDefinition {
    CardDefinition {
        name: "Healer's Hawk",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Gnarlwood Dryad — {G} 1/1 Dryad with Deathtouch.
pub fn gnarlwood_dryad() -> CardDefinition {
    CardDefinition {
        name: "Gnarlwood Dryad",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Typhoid Rats — {B} 1/1 Rat with Deathtouch.
pub fn typhoid_rats() -> CardDefinition {
    CardDefinition {
        name: "Typhoid Rats",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Rat],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Lightning Elemental — {3}{R} 4/1 Elemental with Haste.
pub fn lightning_elemental() -> CardDefinition {
    CardDefinition {
        name: "Lightning Elemental",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Filigree Familiar — {2} 1/1 artifact Fox. ETB gain 2 life; when it
/// dies, draw a card.
pub fn filigree_familiar() -> CardDefinition {
    use crate::effect::shortcut::{etb, on_dies};
    CardDefinition {
        name: "Filigree Familiar",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fox],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            etb(Effect::GainLife { who: Selector::You, amount: Value::Const(2) }),
            on_dies(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        ],
        ..Default::default()
    }
}

/// Gladecover Scout — {G} 1/1 Elf Scout with Hexproof.
pub fn gladecover_scout() -> CardDefinition {
    CardDefinition {
        name: "Gladecover Scout",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Hexproof],
        ..Default::default()
    }
}

/// Deadly Recluse — {1}{G} 1/2 Spider with Reach and Deathtouch.
pub fn deadly_recluse() -> CardDefinition {
    CardDefinition {
        name: "Deadly Recluse",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Reach, Keyword::Deathtouch],
        ..Default::default()
    }
}

/// Sporemound — {4}{G} 3/3 Elemental. Landfall — whenever a land enters
/// under your control, create a 1/1 green Saproling creature token.
pub fn sporemound() -> CardDefinition {
    CardDefinition {
        name: "Sporemound",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::card::TokenDefinition {
                    name: "Saproling".to_string(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Green],
                    supertypes: vec![],
                    subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                
                    static_abilities: vec![],
                },
            },
        }],
        ..Default::default()
    }
}

/// Avenger of Zendikar — {5}{G}{G} 5/5 Elemental. ETB: create a 0/1 green Plant
/// token for each land you control. Landfall — whenever a land you control
/// enters, put a +1/+1 counter on each Plant you control.
pub fn avenger_of_zendikar() -> CardDefinition {
    let plant = crate::card::TokenDefinition {
        name: "Plant".to_string(),
        power: 0,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Green],
        supertypes: vec![],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        activated_abilities: vec![],
        triggered_abilities: vec![],
        static_abilities: vec![],
    };
    let your_plants = Selector::EachPermanent(
        SelectionRequirement::HasCreatureType(CreatureType::Plant)
            .and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Avenger of Zendikar",
        cost: cost(&[generic(5), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elemental], ..Default::default() },
        power: 5,
        toughness: 5,
        triggered_abilities: vec![
            // ETB: a Plant token for each land you control.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::count(Selector::EachPermanent(
                        SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                    )),
                    definition: plant,
                },
            },
            // Landfall: +1/+1 counter on each Plant you control.
            TriggeredAbility {
                event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
                effect: Effect::AddCounter {
                    what: your_plants,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Seedborn Muse — {3}{G}{G} 2/4 Spirit. "Untap all permanents you control
/// during each other player's untap step" (`StaticEffect::UntapAllYoursEach-
/// UntapStep`, consulted by `do_untap`).
pub fn seedborn_muse() -> CardDefinition {
    CardDefinition {
        name: "Seedborn Muse",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 4,
        static_abilities: vec![StaticAbility {
            description: "Untap all permanents you control during each other player's untap step.",
            effect: StaticEffect::UntapAllYoursEachUntapStep,
        }],
        ..Default::default()
    }
}

/// Centaur Courser — {2}{G} 3/3 Centaur Warrior (vanilla).
pub fn centaur_courser() -> CardDefinition {
    CardDefinition {
        name: "Centaur Courser",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Rootwater Hunter — {2}{U} 1/1 Merfolk. {T}: deal 1 damage to target
/// creature.
pub fn rootwater_hunter() -> CardDefinition {
    CardDefinition {
        name: "Rootwater Hunter",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            effect: Effect::DealDamage {
                to: target_filtered(SelectionRequirement::Creature),
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Stormfront Pegasus — {1}{W} 2/1 Pegasus with Flying.
pub fn stormfront_pegasus() -> CardDefinition {
    CardDefinition {
        name: "Stormfront Pegasus",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pegasus],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Suntail Hawk — {W} 1/1 Bird with Flying.
pub fn suntail_hawk() -> CardDefinition {
    CardDefinition {
        name: "Suntail Hawk",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        ..Default::default()
    }
}

/// Thundering Giant — {3}{R} 4/3 Giant with Haste.
pub fn thundering_giant() -> CardDefinition {
    CardDefinition {
        name: "Thundering Giant",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        ..Default::default()
    }
}

/// Pillarfield Ox — {3}{W} 2/4 Ox (vanilla).
pub fn pillarfield_ox() -> CardDefinition {
    CardDefinition {
        name: "Pillarfield Ox",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Ox],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        ..Default::default()
    }
}

/// Thieving Magpie — {2}{U} 1/3 Bird, Flying. Whenever it deals combat
/// damage to a player, draw a card.
pub fn thieving_magpie() -> CardDefinition {
    CardDefinition {
        name: "Thieving Magpie",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Abyssal Specter — {2}{B}{B} 2/3 Specter, Flying. Whenever it deals
/// combat damage to a player, that player discards a card.
pub fn abyssal_specter() -> CardDefinition {
    CardDefinition {
        name: "Abyssal Specter",
        cost: cost(&[generic(2), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Specter],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Bloodgift Demon — {3}{B}{B} 5/4 Demon, Flying. At the beginning of your
/// upkeep, draw a card and lose 1 life. (Printed "target player" collapses
/// to the controller — the usual line.)
pub fn bloodgift_demon() -> CardDefinition {
    CardDefinition {
        name: "Bloodgift Demon",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::ActivePlayer),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Whitemane Lion — {1}{W} 2/2 Cat, Flash. ETB: return a creature you
/// control to its owner's hand.
pub fn whitemane_lion() -> CardDefinition {
    CardDefinition {
        name: "Whitemane Lion",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Move {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Penumbra Spider — {3}{G} 2/4 Spider, Reach. When it dies, create a
/// 2/4 black Spider creature token with Reach.
pub fn penumbra_spider() -> CardDefinition {
    CardDefinition {
        name: "Penumbra Spider",
        cost: cost(&[generic(3), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spider],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![crate::effect::shortcut::on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crate::card::TokenDefinition {
                name: "Spider".to_string(),
                power: 2,
                toughness: 4,
                keywords: vec![Keyword::Reach],
                card_types: vec![CardType::Creature],
                colors: vec![crate::mana::Color::Black],
                supertypes: vec![],
                subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            
                static_abilities: vec![],
            },
        })],
        ..Default::default()
    }
}

/// Ember Hauler — {1}{R} 2/2 Goblin. {2}, Sacrifice this: deal 2 damage to
/// any target.
pub fn ember_hauler() -> CardDefinition {
    use crate::effect::shortcut::target_any;
    CardDefinition {
        name: "Ember Hauler",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(2)]),
            sac_cost: true,
            effect: Effect::DealDamage { to: target_any(), amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fire Imp — {2}{R} 2/1 Imp. ETB: deal 2 damage to target creature.
pub fn fire_imp() -> CardDefinition {
    CardDefinition {
        name: "Fire Imp",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Imp],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Creature),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Borderland Ranger — {2}{G} 2/2 Human Scout. ETB: search your library
/// for a basic land card, reveal it, put it into your hand, then shuffle.
pub fn borderland_ranger() -> CardDefinition {
    CardDefinition {
        name: "Borderland Ranger",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Viashino Pyromancer — {1}{R} 2/1 Lizard. ETB: deal 2 damage to target
/// player.
pub fn viashino_pyromancer() -> CardDefinition {
    CardDefinition {
        name: "Viashino Pyromancer",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::etb(Effect::DealDamage {
            to: target_filtered(SelectionRequirement::Player),
            amount: Value::Const(2),
        })],
        ..Default::default()
    }
}

/// Marauding Mako — {R} 1/1 Shark Pirate. Cycling {2}. Whenever you discard
/// one or more cards, put that many +1/+1 counters on it (`CardDiscarded`
/// fires per card, so one counter per discarded card sums correctly).
pub fn marauding_mako() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Marauding Mako",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // Engine has no Shark/Pirate creature type; classify as Fish.
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Cycling(cost(&[generic(2)]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Bloodghast — {B}{B}, 2/1 Vampire Spirit. "Landfall — Whenever a land
/// enters under your control, you may return Bloodghast from your
/// graveyard to the battlefield."
///
/// Wired via a `LandPlayed` + `YourControl` triggered ability whose
/// effect returns every Bloodghast (modeled as "creature card") in
/// your graveyard to the battlefield. Multiple Bloodghasts in
/// graveyard all return at once, faithful to landfall's per-copy
/// trigger. The "haste while opp ≤ 10 life" rider is omitted (no
/// conditional-keyword static yet).
pub fn bloodghast() -> CardDefinition {
    CardDefinition {
        name: "Bloodghast",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::FromYourGraveyard),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        }],
        ..Default::default()
    }
}

/// Ichorid — {3}{B}, 3/1 Horror with Haste. "At the beginning of the
/// end step, sacrifice this creature. At the beginning of your upkeep,
/// if this card is in your graveyard, you may exile a black creature
/// card other than this card from your graveyard. If you do, return
/// this card to the battlefield."
///
/// The exile-a-black-creature cost rides `SelectorExists(your GY ∩
/// Creature ∩ Black ∩ OtherThanSource)` as the trigger gate plus an
/// `Effect::Move(… → Exile)` cost before the return. The end-step body
/// **sacrifices** Ichorid (to the graveyard, scheduled on return) so it
/// recurs each upkeep while you still have black fodder.
pub fn ichorid() -> CardDefinition {
    use crate::card::{Predicate, Zone};
    use crate::mana::Color;
    let black_fodder = || Selector::CardsInZone {
        who: PlayerRef::You,
        zone: Zone::Graveyard,
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::HasColor(Color::Black))
            .and(SelectionRequirement::OtherThanSource),
    };
    CardDefinition {
        name: "Ichorid",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            )
            .with_filter(Predicate::SelectorExists(black_fodder())),
            effect: Effect::MayDo {
                description: "Exile a black creature card from your graveyard to return Ichorid?".into(),
                body: Box::new(Effect::Seq(vec![
                    // Cost: exile a black creature card other than Ichorid.
                    Effect::Move {
                        what: Selector::take(black_fodder(), Value::Const(1)),
                        to: ZoneDest::Exile,
                    },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Battlefield {
                            controller: PlayerRef::You,
                            tapped: false,
                        },
                    },
                    // "At the beginning of the end step, sacrifice this
                    // creature" — to the graveyard, so it recurs next upkeep.
                    Effect::DelayUntil {
                        kind: DelayedTriggerKind::NextEndStep,
                        body: Box::new(Effect::Move {
                            what: Selector::This,
                            to: ZoneDest::Graveyard,
                        }),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

/// Silversmote Ghoul — {2}{B}, 3/1 Zombie. "Whenever you gain life,
/// return Silversmote Ghoul from your graveyard to the battlefield."
///
/// Wired as a `LifeGained` + `YourControl` triggered ability whose
/// effect returns every creature in your graveyard to the battlefield.
/// Same simplification as Bloodghast: in practice the trigger only
/// fires off the Silversmote Ghoul copy, and your reanimator-shell
/// graveyard is loaded with the right targets.
pub fn silversmote_ghoul() -> CardDefinition {
    CardDefinition {
        name: "Silversmote Ghoul",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::FromYourGraveyard),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
        }],
        ..Default::default()
    }
}

/// Bitterbloom Bearer — {1}{B}, 1/1 Faerie Wizard with Flash and Flying. "When this
/// creature enters, create a 1/1 black Faerie creature token with flying."
pub fn bitterbloom_bearer() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Bitterbloom Bearer",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: TokenDefinition {
                    name: "Faerie".into(),
                    power: 1,
                    toughness: 1,
                    keywords: vec![Keyword::Flying],
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Black],
                    supertypes: vec![],
                    subtypes: Subtypes {
                        creature_types: vec![CreatureType::Faerie],
                        ..Default::default()
                    },
                    activated_abilities: vec![],
                    triggered_abilities: vec![],
                
                    static_abilities: vec![],
                },
            },
        }],
        ..Default::default()
    }
}

/// Dandân — {2}{U}, 4/1 Fish. "Dandân can attack only if defending
/// player controls an Island. When you control no Islands, sacrifice
/// Dandân."
pub fn dandan() -> CardDefinition {
    use crate::card::LandType;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Dandân",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        power: 4,
        toughness: 1,
        keywords: vec![Keyword::CanAttackOnlyIfDefenderControls(Box::new(
            SelectionRequirement::HasLandType(LandType::Island),
        ))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::If {
                cond: Predicate::Not(Box::new(Predicate::SelectorExists(
                    Selector::EachPermanent(
                        SelectionRequirement::HasLandType(LandType::Island)
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                ))),
                then: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Graveyard,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Tidehollow Sculler — {W}{B}, 2/2 Zombie. "When this creature enters,
/// target opponent reveals their hand and you choose a nonland card
/// from it. Exile that card until this creature leaves the battlefield.
/// When this creature leaves the battlefield, return the exiled card
/// to its owner's hand."
///
/// Wired via `Effect::ExileChosenUntilSourceLeaves` (CR 603.6e): the
/// chosen nonland card is exiled linked to the Sculler and returns to
/// its owner's hand when the Sculler leaves play.
pub fn tidehollow_sculler() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Tidehollow Sculler",
        cost: cost(&[w(), b()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileChosenUntilSourceLeaves {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
                return_to: ExileReturnZone::Hand,
            },
        }],
        ..Default::default()
    }
}

/// Temur Ascendancy — {U}{R}{G} Enchantment. Creatures you control with
/// power 4 or greater have haste; when one enters under your control, draw
/// a card. 🟡 the haste static over-grants (the `PowerAtLeast` selector
/// isn't decomposed), so it currently grants haste to all your creatures;
/// the draw trigger is filtered faithfully.
pub fn temur_ascendancy() -> CardDefinition {
    use crate::effect::{Predicate, Selector as Sel, StaticEffect};
    CardDefinition {
        name: "Temur Ascendancy",
        cost: cost(&[u(), r(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::ValueAtLeast(
                    crate::card::Value::PowerOf(Box::new(Sel::TriggerSource)),
                    crate::card::Value::Const(4),
                )),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Sel::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
    }
}

// ── Blade Splicer ──────────────────────────────────────────────────────────

/// Blade Splicer — {2}{W}, 1/1 Human Artificer. ETB: create a 3/3
/// colorless Golem artifact creature token.
pub fn blade_splicer() -> CardDefinition {
    CardDefinition {
        name: "Blade Splicer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::golem_3_3_token(),
            },
        }],
        ..Default::default()
    }
}

// ── Vendilion Clique ───────────────────────────────────────────────────────

/// Vendilion Clique — {1}{U}{U}, 3/1 Legendary Faerie Wizard with Flash
/// and Flying. Body only — the ETB hand-disruption ability is complex and
/// omitted for now.
pub fn vendilion_clique() -> CardDefinition {
    CardDefinition {
        name: "Vendilion Clique",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        ..Default::default()
    }
}

// ── Torrential Gearhulk ────────────────────────────────────────────────────

/// Torrential Gearhulk — {4}{U}{U}, 5/6 Artifact Creature — Construct
/// with Flash. Body only — the ETB "cast instant from graveyard" ability
/// is complex and omitted.
pub fn torrential_gearhulk() -> CardDefinition {
    CardDefinition {
        name: "Torrential Gearhulk",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flash],
        ..Default::default()
    }
}

// ── Kitesail Larcenist ─────────────────────────────────────────────────────

/// Kitesail Larcenist — {2}{U}, 2/3 Human Pirate with Flying. ETB: exile
/// target nonland permanent an opponent controls. No LTB return clause.
pub fn kitesail_larcenist() -> CardDefinition {
    CardDefinition {
        name: "Kitesail Larcenist",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Nonland
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
    }
}

// ── Grave Titan ────────────────────────────────────────────────────────────

/// Grave Titan — {4}{B}{B}, 6/6 Giant with Deathtouch. ETB + whenever
/// this attacks: create two 2/2 black Zombie creature tokens.
pub fn grave_titan() -> CardDefinition {
    use crate::card::TokenDefinition;
    let zombie = TokenDefinition {
        name: "Zombie".into(),
        power: 2,
        toughness: 2,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Black],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    let make_zombies = Effect::CreateToken {
        who: PlayerRef::You,
        count: Value::Const(2),
        definition: zombie,
    };
    CardDefinition {
        name: "Grave Titan",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: make_zombies.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: make_zombies,
            },
        ],
        ..Default::default()
    }
}

// ── Shriekmaw ──────────────────────────────────────────────────────────────

/// Shriekmaw — {4}{B}, 3/2 Elemental with Menace. ETB: destroy target
/// nonblack creature an opponent controls.
pub fn shriekmaw() -> CardDefinition {
    CardDefinition {
        name: "Shriekmaw",
        cost: cost(&[generic(4), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(crate::mana::Color::Black).negate())
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
    }
}

// ── Phyrexian Obliterator ──────────────────────────────────────────────────

/// Phyrexian Obliterator — {B}{B}{B}{B}, 5/8 Phyrexian Horror with
/// Trample. Body only — the damage-retaliation trigger is complex and
/// omitted.
pub fn phyrexian_obliterator() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Obliterator",
        cost: cost(&[b(), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Horror],
            ..Default::default()
        },
        power: 5,
        toughness: 8,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

// ── Glorybringer ───────────────────────────────────────────────────────────

/// Glorybringer — {3}{R}{R}, 4/4 Dragon with Flying and Haste. Whenever
/// this attacks, deal 4 damage to target creature an opponent controls.
pub fn glorybringer() -> CardDefinition {
    CardDefinition {
        name: "Glorybringer",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::DealDamage {
                to: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                amount: Value::Const(4),
            },
        }],
        ..Default::default()
    }
}

// ── Inferno Titan ──────────────────────────────────────────────────────────

/// Inferno Titan — {4}{R}{R}, 6/6 Giant. ETB + whenever this attacks:
/// deal 3 damage to target creature.
pub fn inferno_titan() -> CardDefinition {
    let burn = Effect::DealDamage {
        to: target_filtered(SelectionRequirement::Creature),
        amount: Value::Const(3),
    };
    CardDefinition {
        name: "Inferno Titan",
        cost: cost(&[generic(4), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: burn.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: burn,
            },
        ],
        ..Default::default()
    }
}

// ── Thundermaw Hellkite ────────────────────────────────────────────────────

/// Thundermaw Hellkite — {3}{R}{R}, 5/5 Dragon with Flying and Haste.
/// ETB: deal 1 damage to each creature with flying opponents control and
/// tap them.
pub fn thundermaw_hellkite() -> CardDefinition {
    CardDefinition {
        name: "Thundermaw Hellkite",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasKeyword(Keyword::Flying))
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
                body: Box::new(Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::TriggerSource,
                        amount: Value::Const(1),
                    },
                    Effect::Tap { what: Selector::TriggerSource },
                ])),
            },
        }],
        ..Default::default()
    }
}

// ── Craterhoof Behemoth ────────────────────────────────────────────────────

/// Craterhoof Behemoth — {5}{G}{G}{G}, 5/5 Beast with Haste and Trample.
/// ETB: each creature you control gets +X/+X until end of turn where X
/// is the number of creatures you control.
pub fn craterhoof_behemoth() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Craterhoof Behemoth",
        cost: cost(&[generic(5), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Haste, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::PumpPT {
                    what: Selector::TriggerSource,
                    power: Value::CountOf(Box::new(Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ))),
                    toughness: Value::CountOf(Box::new(Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ))),
                    duration: Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Thragtusk ──────────────────────────────────────────────────────────────

/// Thragtusk — {4}{G}, 5/3 Beast. ETB: gain 5 life. Death trigger:
/// create a 3/3 green Beast creature token.
pub fn thragtusk() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Thragtusk",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(5) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Beast".into(),
                        power: 3,
                        toughness: 3,
                        keywords: vec![],
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::Green],
                        supertypes: vec![],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Beast],
                            ..Default::default()
                        },
                        activated_abilities: vec![],
                        triggered_abilities: vec![],
                    
                        static_abilities: vec![],
                    },
                },
            },
        ],
        ..Default::default()
    }
}

// ── Courser of Kruphix ─────────────────────────────────────────────────────

/// Courser of Kruphix — {1}{G}{G}, 2/4 Centaur Enchantment Creature.
/// Landfall: whenever a land enters the battlefield under your control,
/// gain 1 life.
///
/// Wired via `EntersBattlefield` + `YourControl` scope with a
/// `Predicate::EntityMatches { what: TriggerSource, filter: Land }` filter,
/// matching the Tireless Tracker pattern. This catches both played and
/// fetched lands.
pub fn courser_of_kruphix() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Courser of Kruphix",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

// ── Wurmcoil Engine ────────────────────────────────────────────────────────

/// Wurmcoil Engine — {6}, 6/6 Artifact Creature — Phyrexian Wurm with
/// Deathtouch and Lifelink. Death trigger: create a 3/3 colorless Wurm
/// artifact creature token with Deathtouch and a 3/3 colorless Wurm
/// artifact creature token with Lifelink.
pub fn wurmcoil_engine() -> CardDefinition {
    use crate::card::TokenDefinition;
    let wurm_deathtouch = TokenDefinition {
        name: "Phyrexian Wurm".into(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch],
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    let wurm_lifelink = TokenDefinition {
        name: "Phyrexian Wurm".into(),
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Lifelink],
        card_types: vec![CardType::Artifact, CardType::Creature],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Wurmcoil Engine",
        cost: cost(&[generic(6)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: wurm_deathtouch,
                },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: wurm_lifelink,
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Vengevine ──────────────────────────────────────────────────────────────

/// Vengevine — {2}{G}{G}, 4/3 Elemental with Haste.
///
/// Oracle: "Haste. Whenever you cast a creature spell, if Vengevine is in
/// your graveyard and you've cast another creature spell this turn, you may
/// return Vengevine from your graveyard to the battlefield."
///
/// Wired via `SpellCast/FromYourGraveyard` trigger with a
/// `CreaturesCastThisTurnAtLeast(You, 2)` intervening-if gate.
pub fn vengevine() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Vengevine",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::FromYourGraveyard)
                .with_filter(Predicate::All(vec![
                    Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::Creature,
                    },
                    Predicate::CreaturesCastThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(2),
                    },
                ])),
            effect: Effect::MayDo {
                description: "Return Vengevine from your graveyard to the battlefield."
                    .to_string(),
                body: Box::new(Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Portal to Phyrexia ────────────────────────────────────────────────────

/// Portal to Phyrexia — {9}, Artifact.
///
/// Oracle: "When Portal to Phyrexia enters, each opponent sacrifices three
/// creatures. At the beginning of your upkeep, put target creature card
/// from a graveyard onto the battlefield under your control."
pub fn portal_to_phyrexia() -> CardDefinition {
    CardDefinition {
        name: "Portal to Phyrexia",
        cost: cost(&[generic(9)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Sacrifice {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    count: Value::Const(3),
                    filter: SelectionRequirement::Creature,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::Upkeep),
                    EventScope::YourControl,
                ),
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
                },
            },
        ],
        ..Default::default()
    }
}

// ── Finale of Devastation ──────────────────────────────────────────────────

/// Finale of Devastation — {X}{G}{G}, Sorcery.
///
/// Oracle: "Search your library and/or graveyard for a creature card with
/// mana value X or less and put it onto the battlefield. If X is 10 or
/// more, creatures you control get +X/+X and gain haste until end of turn.
/// Shuffle."
///
/// Approximation: always searches library (no gy search). The "+X/+X and
/// haste" rider uses `ForEach(your creature) → PumpPT(X, X, EOT) +
/// GrantKeyword(Haste, EOT)`.
pub fn finale_of_devastation() -> CardDefinition {
    use crate::effect::{Duration, Predicate};
    CardDefinition {
        name: "Finale of Devastation",
        cost: cost(&[crate::mana::x(), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Creature,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
            },
            Effect::If {
                cond: Predicate::ValueAtLeast(
                    Value::XFromCost,
                    Value::Const(10),
                ),
                then: Box::new(Effect::ForEach {
                    selector: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    body: Box::new(Effect::Seq(vec![
                        Effect::PumpPT {
                            what: Selector::TriggerSource,
                            power: Value::XFromCost,
                            toughness: Value::XFromCost,
                            duration: Duration::EndOfTurn,
                        },
                        Effect::GrantKeyword {
                            what: Selector::TriggerSource,
                            keyword: Keyword::Haste,
                            duration: Duration::EndOfTurn,
                        },
                    ])),
                }),
                else_: Box::new(Effect::Noop),
            },
        ]),
        ..Default::default()
    }
}

// ── Rishadan Port ──────────────────────────────────────────────────────────

/// Rishadan Port — Land.
///
/// Oracle: "{T}: Add {C}. {1}, {T}: Tap target land."
pub fn rishadan_port() -> CardDefinition {
    CardDefinition {
        name: "Rishadan Port",
        cost: ManaCost::default(),
        card_types: vec![CardType::Land],
        activated_abilities: vec![
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colorless(Value::Const(1)),
                },
                ..Default::default()
            },
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
                effect: Effect::Tap {
                    what: target_filtered(SelectionRequirement::Land),
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

// ── Horizon lands (Future Sight / Modern Horizons cycle) ───────────────────
// "{T}, Pay 1 life: Add {c1} or {c2}. {1}, {T}, Sacrifice this: Draw a card."

pub fn horizon_canopy() -> CardDefinition {
    use crate::mana::Color;
    super::super::horizon_land("Horizon Canopy", Color::Green, Color::White)
}

pub fn sunbaked_canyon() -> CardDefinition {
    use crate::mana::Color;
    super::super::horizon_land("Sunbaked Canyon", Color::Red, Color::White)
}

pub fn waterlogged_grove() -> CardDefinition {
    use crate::mana::Color;
    super::super::horizon_land("Waterlogged Grove", Color::Green, Color::Blue)
}

pub fn fiery_islet() -> CardDefinition {
    use crate::mana::Color;
    super::super::horizon_land("Fiery Islet", Color::Blue, Color::Red)
}

pub fn nurturing_peatland() -> CardDefinition {
    use crate::mana::Color;
    super::super::horizon_land("Nurturing Peatland", Color::Black, Color::Green)
}

pub fn silent_clearing() -> CardDefinition {
    use crate::mana::Color;
    super::super::horizon_land("Silent Clearing", Color::White, Color::Black)
}

// ── Koma, Cosmos Serpent ────────────────────────────────────────────────────

/// Koma, Cosmos Serpent — {3}{G}{G}{U}{U}, 6/6 Legendary Serpent.
///
/// Oracle: "This spell can't be countered. At the beginning of each upkeep,
/// create a 3/3 blue Serpent creature token named Koma's Coil.
/// Sacrifice another Serpent: Choose one — Tap target permanent. Its
/// activated abilities can't be activated this turn. / Koma, Cosmos Serpent
/// gains indestructible until end of turn."
///
/// Approximation: CantBeCountered + upkeep token mint. The sac-serpent
/// abilities are collapsed to a single sac-any-creature activation that
/// taps a target permanent (the most common mode).
pub fn koma_cosmos_serpent() -> CardDefinition {
    use crate::card::TokenDefinition;
    let coil = TokenDefinition {
        name: "Koma's Coil".to_string(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Serpent],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Koma, Cosmos Serpent",
        cost: cost(&[generic(3), g(), g(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Serpent],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::CantBeCountered],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: coil,
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None, from_hand: false,
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(SelectionRequirement::Permanent),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Mesmeric Orb ───────────────────────────────────────────────────────────

/// Mesmeric Orb — {2}, Artifact.
///
/// Oracle: "Whenever a permanent becomes untapped, that permanent's
/// controller mills a card."
///
/// Approximation: At the beginning of each player's upkeep, each player
/// mills 3 (approximates the mass-untap mill without needing an untap
/// event per permanent).
pub fn mesmeric_orb() -> CardDefinition {
    CardDefinition {
        name: "Mesmeric Orb",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::AnyPlayer,
            ),
            effect: Effect::Mill {
                who: Selector::Player(PlayerRef::EachPlayer),
                amount: Value::Const(3),
            },
        }],
        ..Default::default()
    }
}

// ── Chalice of the Void ────────────────────────────────────────────────────

/// Chalice of the Void — {X}{X}, Artifact.
///
/// Oracle: "Chalice of the Void enters with X charge counters on it.
/// Whenever a player casts a spell with mana value equal to the number of
/// charge counters on Chalice of the Void, counter that spell."
///
/// Approximation: enters with X charge counters. The counter-spells-with-
/// matching-MV trigger needs a SpellCast + MV-check predicate that reads
/// counters off this permanent — wired as a SpellCast/AnyPlayer trigger
/// gated on `ValueEquals(ManaValueOf(TriggerSource), CountersOn(This, Charge))`.
pub fn chalice_of_the_void() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Chalice of the Void",
        cost: cost(&[crate::mana::x(), crate::mana::x()]),
        card_types: vec![CardType::Artifact],
        enters_with_counters: Some((CounterType::Charge, Value::XFromCost)),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::ValueEquals(
                    Value::ManaValueOf(Box::new(Selector::TriggerSource)),
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::Charge,
                    },
                )),
            effect: Effect::CounterSpell {
                what: Selector::TriggerSource,
            },
        }],
        ..Default::default()
    }
}

// ── Candelabra of Tawnos ───────────────────────────────────────────────────

/// Candelabra of Tawnos — {1}, Artifact.
///
/// Oracle: "{X}, {T}: Untap X target lands."
///
/// Approximation: `{X}, {T}: Untap up to X lands you control` via the
/// existing `Untap { up_to }` with `Value::XFromCost`.
pub fn candelabra_of_tawnos() -> CardDefinition {
    CardDefinition {
        name: "Candelabra of Tawnos",
        cost: cost(&[generic(1)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[crate::mana::x()]),
            effect: Effect::Untap {
                what: Selector::EachPermanent(
                    SelectionRequirement::Land.and(SelectionRequirement::ControlledByYou),
                ),
                up_to: Some(Value::XFromCost),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Guardian Scalelord ──────────────────────────────────────────────────────

/// Guardian Scalelord — {3}{W}{W}, 4/4 Dragon with Flying.
///
/// Oracle: "Flying. Whenever this creature attacks, you may have target
/// creature you control gain flying until end of turn."
///
/// Wired with an `Attacks/SelfSource` trigger that fans out to a
/// `MayDo(GrantKeyword(Flying, EOT, target friendly creature))`. The
/// "another" / "you control" rider scopes the auto-target to creatures
/// the controller owns; the AutoDecider opts in by default (declining
/// flying-grant is a strict downside).
pub fn guardian_scalelord() -> CardDefinition {
    CardDefinition {
        name: "Guardian Scalelord",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Target creature you control gains flying until end of turn.".to_string(),
                body: Box::new(Effect::GrantKeyword {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    keyword: Keyword::Flying,
                    duration: crate::effect::Duration::EndOfTurn,
                }),
            },
        }],
        ..Default::default()
    }
}

// ── Growing Ranks ──────────────────────────────────────────────────────────

/// Growing Ranks — {2}{G}{W} Enchantment. At the beginning of your upkeep,
/// populate (CR 701.32 — `Effect::Populate`).
pub fn growing_ranks() -> CardDefinition {
    CardDefinition {
        name: "Growing Ranks",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Populate { who: PlayerRef::You },
        }],
        ..Default::default()
    }
}

// ── Master of Death ────────────────────────────────────────────────────────

/// Master of Death — {1}{U}{B} Creature — Zombie Wizard 3/1. At the
/// beginning of your upkeep, you may pay 1 life. If you do, return
/// Master of Death from your graveyard to your hand.
///
/// Recurring card-advantage creature. The upkeep trigger fires only
/// while Master of Death is in your graveyard (EventScope::
/// FromYourGraveyard). The "pay 1 life" gate is modeled as a MayDo
/// wrapping LoseLife + Move(This → Hand).
pub fn master_of_death() -> CardDefinition {
    CardDefinition {
        name: "Master of Death",
        cost: cost(&[generic(1), u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::FromYourGraveyard,
            ),
            effect: Effect::MayDo {
                description: "Pay 1 life to return Master of Death to your hand.".to_string(),
                body: Box::new(Effect::Seq(vec![
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Hand(PlayerRef::You),
                    },
                ])),
            },
        }],
        ..Default::default()
    }
}

// ── Basking Broodscale ─────────────────────────────────────────────────────

/// Basking Broodscale — {1}{G} Creature — Lizard 0/1.
/// ETB with 2 +1/+1 counters + creates 2 Eldrazi Spawn tokens.
pub fn basking_broodscale() -> CardDefinition {
    use crate::card::{CounterType, TokenDefinition};
    let spawn = TokenDefinition {
        name: "Eldrazi Spawn".to_string(),
        power: 0,
        toughness: 1,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            sac_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(1)),
            },
            ..Default::default()
        }],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Basking Broodscale",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Lizard],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: spawn,
            },
        }],
        ..Default::default()
    }
}

/// Sowing Mycospawn — {4}{G} Creature — Eldrazi Fungus 4/4.
/// ETB search land -> BF tapped.
pub fn sowing_mycospawn() -> CardDefinition {
    CardDefinition {
        name: "Sowing Mycospawn",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi, CreatureType::Fungus],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::Land,
                to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
            },
        }],
        ..Default::default()
    }
}

/// Ursine Monstrosity — {3}{G}{G} Creature — Bear 0/0.
/// Enters with 5 +1/+1 counters, Trample, ETB draw 1.
pub fn ursine_monstrosity() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Ursine Monstrosity",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(5))),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Moonshadow — {1}{B} Creature — Faerie Rogue 2/1 Flying.
/// Combat damage to player -> that player discards.
pub fn moonshadow() -> CardDefinition {
    CardDefinition {
        name: "Moonshadow",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Discard {
                who: Selector::Player(PlayerRef::DefendingPlayer),
                amount: Value::Const(1),
                random: false,
            },
        }],
        ..Default::default()
    }
}

/// Golos, Tireless Pilgrim — {5} Legendary Artifact Creature 3/5.
/// ETB search a land -> BF tapped.
pub fn golos_tireless_pilgrim() -> CardDefinition {
    CardDefinition {
        name: "Golos, Tireless Pilgrim",
        cost: cost(&[generic(5)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 5,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Search for a land, put it onto the battlefield tapped.".to_string(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Land,
                    to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
                }),
            },
        }],
        ..Default::default()
    }
}

/// Maelstrom Archangel — {W}{U}{B}{R}{G} 5/5 Flying Angel.
/// Combat damage to player -> draw 2 (approximation of free cast).
pub fn maelstrom_archangel() -> CardDefinition {
    CardDefinition {
        name: "Maelstrom Archangel",
        cost: cost(&[w(), u(), b(), r(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

/// Ramos, Dragon Engine — {6} Legendary Artifact Creature — Dragon 4/4 Flying.
/// Spell-cast -> +1/+1 counter. Tap, remove 5 counters: add WUBRG×2.
pub fn ramos_dragon_engine() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Ramos, Dragon Engine",
        cost: cost(&[generic(6)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::RemoveCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(5),
                },
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![
                        crate::mana::Color::White, crate::mana::Color::White,
                        crate::mana::Color::Blue, crate::mana::Color::Blue,
                        crate::mana::Color::Black, crate::mana::Color::Black,
                        crate::mana::Color::Red, crate::mana::Color::Red,
                        crate::mana::Color::Green, crate::mana::Color::Green,
                    ]),
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Descendant of Storms ──────────────────────────────────────────────────

/// Descendant of Storms — {2}{W}, 2/2 Spirit. Flying. When this creature
/// dies, create a 1/1 white Spirit token with flying.
pub fn descendant_of_storms() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::mana::Color;
    let spirit_token = TokenDefinition {
        name: "Spirit".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Descendant of Storms",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: spirit_token,
            },
        }],
        ..Default::default()
    }
}

/// Elite Spellbinder — {1}{W}{W}, 3/1 Human Cleric with Flying and Flash.
///
/// Approximation: body-only. The full Oracle text ("When this creature
/// enters, look at target opponent's hand, exile a nonland card from it;
/// that card costs {2} more to cast") is omitted — the engine has no
/// look-at-opponent-hand primitive and no per-card cost-tax static tied
/// to an exiled card. The 3/1 Flying Flash body is the load-bearing part
/// for the cube (efficient tempo creature).
/// Elite Spellbinder — {1}{W}{W}, 3/1 Human Cleric. Flash, Flying.
/// ETB: look at target opponent's hand and exile a nonland card.
/// Approximated as ETB discard-opponent-nonland.
pub fn elite_spellbinder() -> CardDefinition {
    CardDefinition {
        name: "Elite Spellbinder",
        cost: cost(&[generic(1), w(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        ..Default::default()
    }
}

/// Elder Gargaroth — {3}{G}{G}, 6/6 Beast with Vigilance, Reach, and
/// Trample.
///
/// "Whenever this creature attacks or blocks, choose one — Create a 3/3
/// green Beast creature token; or You gain 3 life; or Draw a card."
///
/// Approximation: the trigger fires only on attack (the engine has no
/// `Blocks` event kind); the three modes are wired via `ChooseMode`.
/// AutoDecider picks mode 0 (create a 3/3 Beast token).
pub fn elder_gargaroth() -> CardDefinition {
    let beast_token = crate::card::TokenDefinition {
        name: "Beast".into(),
        power: 3,
        toughness: 3,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::Green],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    
        static_abilities: vec![],
    };
    CardDefinition {
        name: "Elder Gargaroth",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                // Mode 0: Create a 3/3 green Beast creature token.
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: beast_token,
                },
                // Mode 1: You gain 3 life.
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
                // Mode 2: Draw a card.
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        ..Default::default()
    }
}

// ── Evolve (CR 702.100) ───────────────────────────────────────────────────────

/// Cloudfin Raptor — {U}, 0/1 Bird. "Evolve. Flying."
pub fn cloudfin_raptor() -> CardDefinition {
    use crate::effect::shortcut::evolve;
    CardDefinition {
        name: "Cloudfin Raptor",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Experiment One — {G}, 1/1 Human Ooze. "Evolve."
/// (The "Remove two +1/+1 counters: Regenerate" ability needs a
/// counter-removal activation cost the engine doesn't model yet.)
pub fn experiment_one() -> CardDefinition {
    use crate::effect::shortcut::evolve;
    CardDefinition {
        name: "Experiment One",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ooze],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![evolve()],
        ..Default::default()
    }
}

/// Fathom Mage — {1}{G}{U}, 1/1 Human Wizard. "Evolve. Whenever a +1/+1
/// counter is placed on Fathom Mage, draw a card."
pub fn fathom_mage() -> CardDefinition {
    use crate::effect::shortcut::evolve;
    CardDefinition {
        name: "Fathom Mage",
        cost: cost(&[generic(1), g(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            evolve(),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            },
        ],
        ..Default::default()
    }
}

// ── ETB / death-value creatures ────────────────────────────────────────────────

/// Phyrexian Rager — {2}{B}, 2/2. "When this creature enters, you draw a
/// card and you lose 1 life."
pub fn phyrexian_rager() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Phyrexian Rager",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horror],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Seq(vec![
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
        ]))],
        ..Default::default()
    }
}

/// Carven Caryatid — {1}{G}{G}, 0/5 Defender. "When this creature enters,
/// draw a card."
pub fn carven_caryatid() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Carven Caryatid",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wall],
            ..Default::default()
        },
        power: 0,
        toughness: 5,
        keywords: vec![Keyword::Defender],
        triggered_abilities: vec![etb(Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Doomed Traveler — {W}, 1/1. "When this creature dies, create a 1/1 white
/// Spirit creature token with flying."
pub fn doomed_traveler() -> CardDefinition {
    use crate::card::TokenDefinition;
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Doomed Traveler",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: TokenDefinition {
                name: "Spirit".into(),
                power: 1,
                toughness: 1,
                keywords: vec![Keyword::Flying],
                card_types: vec![CardType::Creature],
                colors: vec![crate::mana::Color::White],
                supertypes: vec![],
                subtypes: Subtypes {
                    creature_types: vec![CreatureType::Spirit],
                    ..Default::default()
                },
                activated_abilities: vec![],
                triggered_abilities: vec![],
            
                static_abilities: vec![],
            },
        })],
        ..Default::default()
    }
}

/// Festering Goblin — {B}, 1/1. "When this creature dies, target creature
/// gets -1/-1 until end of turn."
pub fn festering_goblin() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    use crate::effect::Duration;
    CardDefinition {
        name: "Festering Goblin",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Goblin],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![on_dies(Effect::PumpPT {
            what: target_filtered(SelectionRequirement::Creature),
            power: Value::Const(-1),
            toughness: Value::Const(-1),
            duration: Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Aven Fisher — {3}{U}, 2/2 Flying. "When this creature dies, you may
/// draw a card."
pub fn aven_fisher() -> CardDefinition {
    use crate::effect::shortcut::on_dies;
    CardDefinition {
        name: "Aven Fisher",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![on_dies(Effect::MayDo {
            description: "Aven Fisher: draw a card?".into(),
            body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
        })],
        ..Default::default()
    }
}

/// Prodigal Pyromancer — {2}{R}, 1/1. "{T}: This creature deals 1 damage
/// to any target." (Red "Tim".)
pub fn prodigal_pyromancer() -> CardDefinition {
    use crate::effect::shortcut::{deal, target_any};
    CardDefinition {
        name: "Prodigal Pyromancer",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: deal(1, target_any()),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gravedigger — {3}{B}, 2/2. "When this creature enters, you may return
/// target creature card from your graveyard to your hand."
pub fn gravedigger() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Gravedigger",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![etb(Effect::Move {
            what: target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Spore Frog — {G}, 1/1. "Sacrifice this creature: Prevent all combat
/// damage that would be dealt this turn." (CR 615.1)
pub fn spore_frog() -> CardDefinition {
    CardDefinition {
        name: "Spore Frog",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::PreventAllCombatDamageThisTurn,
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── Renown (CR 702.111) ───────────────────────────────────────────────────

/// Topan Freeblade — {1}{W} 2/2 Human Soldier with Vigilance, Renown 1.
pub fn topan_freeblade() -> CardDefinition {
    CardDefinition {
        name: "Topan Freeblade",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![crate::effect::shortcut::renown(1)],
        ..Default::default()
    }
}

/// Stalwart Aven — {2}{W} 2/2 Bird Soldier with Flying, Renown 1.
pub fn stalwart_aven() -> CardDefinition {
    CardDefinition {
        name: "Stalwart Aven",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::renown(1)],
        ..Default::default()
    }
}

/// Skyraker Giant — {4}{R} 4/4 Giant Warrior with Reach, Renown 4.
pub fn skyraker_giant() -> CardDefinition {
    CardDefinition {
        name: "Skyraker Giant",
        cost: cost(&[generic(4), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![crate::effect::shortcut::renown(4)],
        ..Default::default()
    }
}

// ── Outlast (CR 702.97) ────────────────────────────────────────────────────

/// "Each creature you control with a +1/+1 counter on it has [keyword]" —
/// the Khans Outlast lord static, via the layer system's `AllWithCounter`
/// decomposition.
fn counter_anthem(keyword: Keyword, description: &'static str) -> StaticAbility {
    StaticAbility {
        description,
        effect: StaticEffect::GrantKeyword {
            applies_to: Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
            ),
            keyword,
        },
    }
}

/// Ainok Bond-Kin — {1}{W} 2/2 Hound Soldier, Outlast {1}{W}. Creatures you
/// control with a +1/+1 counter have first strike.
pub fn ainok_bond_kin() -> CardDefinition {
    CardDefinition {
        name: "Ainok Bond-Kin",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hound, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![crate::effect::shortcut::outlast(cost(&[generic(1), w()]))],
        static_abilities: vec![counter_anthem(
            Keyword::FirstStrike,
            "Each creature you control with a +1/+1 counter on it has first strike.",
        )],
        ..Default::default()
    }
}

/// Tuskguard Captain — {2}{G} 2/2 Human Warrior, Outlast {1}{G}. Creatures
/// you control with a +1/+1 counter have trample.
pub fn tuskguard_captain() -> CardDefinition {
    CardDefinition {
        name: "Tuskguard Captain",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![crate::effect::shortcut::outlast(cost(&[generic(1), g()]))],
        static_abilities: vec![counter_anthem(
            Keyword::Trample,
            "Each creature you control with a +1/+1 counter on it has trample.",
        )],
        ..Default::default()
    }
}

/// Abzan Falconer — {2}{W} 2/3 Human Soldier, Outlast {W}. Creatures you
/// control with a +1/+1 counter have flying.
pub fn abzan_falconer() -> CardDefinition {
    CardDefinition {
        name: "Abzan Falconer",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![crate::effect::shortcut::outlast(cost(&[w()]))],
        static_abilities: vec![counter_anthem(
            Keyword::Flying,
            "Each creature you control with a +1/+1 counter on it has flying.",
        )],
        ..Default::default()
    }
}

/// Mer-Ek Nightblade — {3}{B} 2/3 Orc Assassin, Outlast {1}{B}. Creatures
/// you control with a +1/+1 counter have deathtouch.
pub fn mer_ek_nightblade() -> CardDefinition {
    CardDefinition {
        name: "Mer-Ek Nightblade",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Assassin],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        activated_abilities: vec![crate::effect::shortcut::outlast(cost(&[generic(1), b()]))],
        static_abilities: vec![counter_anthem(
            Keyword::Deathtouch,
            "Each creature you control with a +1/+1 counter on it has deathtouch.",
        )],
        ..Default::default()
    }
}

/// Knight of the Pilgrim's Road — {1}{W} 2/2 Human Knight, Renown 1.
pub fn knight_of_the_pilgrims_road() -> CardDefinition {
    CardDefinition {
        name: "Knight of the Pilgrim's Road",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::renown(1)],
        ..Default::default()
    }
}

/// Consul's Lieutenant — {1}{W} 2/1 Human Soldier, First Strike, Renown 1.
pub fn consuls_lieutenant() -> CardDefinition {
    CardDefinition {
        name: "Consul's Lieutenant",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::FirstStrike],
        triggered_abilities: vec![crate::effect::shortcut::renown(1)],
        ..Default::default()
    }
}

// ── Bloodthirst (CR 702.54) ───────────────────────────────────────────────

/// Scab-Clan Mauler — {1}{R} 2/2 Human Warrior, Bloodthirst 2.
pub fn scab_clan_mauler() -> CardDefinition {
    CardDefinition {
        name: "Scab-Clan Mauler",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warrior],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![crate::effect::shortcut::bloodthirst(2)],
        ..Default::default()
    }
}

/// Gorehorn Minotaurs — {2}{R} 3/3 Minotaur Warrior, Bloodthirst 2.
pub fn gorehorn_minotaurs() -> CardDefinition {
    CardDefinition {
        name: "Gorehorn Minotaurs",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Minotaur, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![crate::effect::shortcut::bloodthirst(2)],
        ..Default::default()
    }
}

/// Bloodfray Giant — {3}{R} 3/2 Giant Warrior with Trample, Bloodthirst 1.
pub fn bloodfray_giant() -> CardDefinition {
    CardDefinition {
        name: "Bloodfray Giant",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![crate::effect::shortcut::bloodthirst(1)],
        ..Default::default()
    }
}

/// Abzan Battle Priest — {3}{W} 3/2 Human Cleric, Outlast {2}{W}. Creatures
/// you control with a +1/+1 counter have lifelink.
pub fn abzan_battle_priest() -> CardDefinition {
    CardDefinition {
        name: "Abzan Battle Priest",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        activated_abilities: vec![crate::effect::shortcut::outlast(cost(&[generic(2), w()]))],
        static_abilities: vec![counter_anthem(
            Keyword::Lifelink,
            "Each creature you control with a +1/+1 counter on it has lifelink.",
        )],
        ..Default::default()
    }
}

/// Disowned Ancestor — {1}{B} 1/4 Spirit Warrior, Renown 1.
pub fn disowned_ancestor() -> CardDefinition {
    CardDefinition {
        name: "Disowned Ancestor",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        triggered_abilities: vec![crate::effect::shortcut::renown(1)],
        ..Default::default()
    }
}

/// Citadel Castellan — {1}{G}{W} 2/4 Human Knight, Renown 3.
pub fn citadel_castellan() -> CardDefinition {
    CardDefinition {
        name: "Citadel Castellan",
        cost: cost(&[generic(1), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        triggered_abilities: vec![crate::effect::shortcut::renown(3)],
        ..Default::default()
    }
}

/// Ledger Shredder — {1}{U} Creature — Bird Advisor 1/3, Flying. "Whenever a
/// player casts their second spell each turn, Ledger Shredder connives." (SNC)
pub fn ledger_shredder() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Ledger Shredder",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer).with_filter(
                Predicate::SpellsCastThisTurnEquals {
                    who: PlayerRef::Triggerer,
                    count: Value::Const(2),
                },
            ),
            effect: crate::effect::shortcut::connive(1),
        }],
        ..Default::default()
    }
}

/// Guttersnipe — {2}{R} Creature — Goblin Shaman 2/2. "Whenever you cast an
/// instant or sorcery spell, Guttersnipe deals 2 damage to each opponent." (RTR)
pub fn guttersnipe() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    CardDefinition {
        name: "Guttersnipe",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Sheoldred, the Apocalypse — {2}{B}{B} Legendary Creature — Phyrexian Praetor
/// 4/5, Deathtouch. "Whenever you draw a card, you gain 2 life. Whenever an
/// opponent draws a card, that player loses 2 life." (DMU)
pub fn sheoldred_the_apocalypse() -> CardDefinition {
    CardDefinition {
        name: "Sheoldred, the Apocalypse",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Praetor],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Deathtouch],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl),
                effect: Effect::GainLife { who: Selector::You, amount: Value::Const(2) },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Bitterblossom — {1}{B} Tribal Enchantment — Faerie. "At the beginning of
/// your upkeep, you lose 1 life and create a 1/1 black Faerie Rogue creature
/// token with flying." (MOR)
pub fn bitterblossom() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Bitterblossom",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes { creature_types: vec![CreatureType::Faerie], ..Default::default() },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: TokenDefinition {
                        name: "Faerie Rogue".into(),
                        power: 1,
                        toughness: 1,
                        card_types: vec![CardType::Creature],
                        colors: vec![crate::mana::Color::Black],
                        subtypes: Subtypes {
                            creature_types: vec![CreatureType::Faerie, CreatureType::Rogue],
                            ..Default::default()
                        },
                        keywords: vec![Keyword::Flying],
                        ..Default::default()
                    },
                },
            ]),
        }],
        ..Default::default()
    }
}

/// Brineborn Cutthroat — {1}{U} Creature — Merfolk Pirate 1/2, Flash. "Whenever
/// you cast a spell during an opponent's turn, put a +1/+1 counter on Brineborn
/// Cutthroat." (M20)
pub fn brineborn_cutthroat() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Brineborn Cutthroat",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Pirate],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::Not(Box::new(Predicate::IsTurnOf(PlayerRef::You)))),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Rotting Regisaur — {1}{B}{B} Creature — Zombie Dinosaur 7/6. "At the
/// beginning of your upkeep, discard a card." (M20)
pub fn rotting_regisaur() -> CardDefinition {
    CardDefinition {
        name: "Rotting Regisaur",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Dinosaur],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
        }],
        ..Default::default()
    }
}

/// Sun Titan — {4}{W}{W} Creature — Giant 6/6, Vigilance. "Whenever Sun Titan
/// enters or attacks, return target permanent card with mana value 3 or less
/// from your graveyard to the battlefield." (M11)
pub fn sun_titan() -> CardDefinition {
    let recur = || Effect::Move {
        what: target_filtered(
            SelectionRequirement::Permanent.and(SelectionRequirement::ManaValueAtMost(3)),
        ),
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Sun Titan",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: recur(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: recur(),
            },
        ],
        ..Default::default()
    }
}

/// Primeval Titan — {4}{G}{G} Creature — Giant 6/6, Trample. "Whenever Primeval
/// Titan enters or attacks, search your library for up to two land cards, put
/// them onto the battlefield tapped, then shuffle." (M11)
pub fn primeval_titan() -> CardDefinition {
    let fetch = || Effect::Seq(vec![
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Land,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
    ]);
    CardDefinition {
        name: "Primeval Titan",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Giant], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: fetch(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: fetch(),
            },
        ],
        ..Default::default()
    }
}

/// Archon of Cruelty — {6}{B}{B} Creature — Archon 6/6, Flying. "Whenever Archon
/// of Cruelty enters or attacks, target opponent sacrifices a creature or
/// planeswalker, loses 3 life, and discards a card. You draw a card and gain 3
/// life." (MH2; the target-opponent clause uses `EachOpponent` — faithful in
/// 1v1, fans out in multiplayer.)
pub fn archon_of_cruelty() -> CardDefinition {
    let body = || Effect::Seq(vec![
        Effect::Sacrifice {
            who: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Creature.or(SelectionRequirement::Planeswalker),
        },
        Effect::LoseLife { who: Selector::Player(PlayerRef::EachOpponent), amount: Value::Const(3) },
        Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: SelectionRequirement::Any,
        },
        Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        Effect::GainLife { who: Selector::You, amount: Value::Const(3) },
    ]);
    CardDefinition {
        name: "Archon of Cruelty",
        cost: cost(&[generic(6), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Archon], ..Default::default() },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: body(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: body(),
            },
        ],
        ..Default::default()
    }
}

/// Rampaging Ferocidon — {2}{R} Creature — Dinosaur 3/3, Menace. "Players can't
/// gain life. Whenever another creature enters, Rampaging Ferocidon deals 1
/// damage to that creature's controller." (XLN)
pub fn rampaging_ferocidon() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Rampaging Ferocidon",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Dinosaur], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        static_abilities: vec![StaticAbility {
            description: "Players can't gain life.",
            effect: StaticEffect::PlayerCannotGainLife {
                target: crate::effect::PlayerStaticTarget::EachPlayer,
            },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(crate::effect::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::ControllerOf(Box::new(Selector::TriggerSource))),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Massacre Wurm — {3}{B}{B}{B} Creature — Phyrexian Wurm 6/5. "When Massacre
/// Wurm enters, creatures your opponents control get -2/-2 until end of turn.
/// Whenever a creature an opponent controls dies, that player loses 2 life." (NPH)
pub fn massacre_wurm() -> CardDefinition {
    use crate::effect::Duration;
    CardDefinition {
        name: "Massacre Wurm",
        cost: cost(&[generic(3), b(), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Wurm],
            ..Default::default()
        },
        power: 6,
        toughness: 5,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::PumpPT {
                    what: Selector::EachPermanent(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    power: Value::Const(-2),
                    toughness: Value::Const(-2),
                    duration: Duration::EndOfTurn,
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::OpponentControl),
                effect: Effect::LoseLife {
                    who: Selector::Player(PlayerRef::Triggerer),
                    amount: Value::Const(2),
                },
            },
        ],
        ..Default::default()
    }
}

/// Meteor Golem — {7} Artifact Creature — Golem 3/3. "When Meteor Golem enters,
/// destroy target nonland permanent an opponent controls." (M19)
pub fn meteor_golem() -> CardDefinition {
    CardDefinition {
        name: "Meteor Golem",
        cost: cost(&[generic(7)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Golem], ..Default::default() },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Nonland.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        ..Default::default()
    }
}

/// Merciless Executioner — {2}{B} Creature — Goblin Warrior 3/1. "When Merciless
/// Executioner enters, each player sacrifices a creature." (FRF)
pub fn merciless_executioner() -> CardDefinition {
    CardDefinition {
        name: "Merciless Executioner",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Sacrifice {
                who: Selector::Player(PlayerRef::EachPlayer),
                count: Value::Const(1),
                filter: SelectionRequirement::Creature,
            },
        }],
        ..Default::default()
    }
}

/// Burnished Hart — {3} Artifact Creature — Elk 2/2. "{3}, Sacrifice Burnished
/// Hart: Search your library for up to two basic land cards, put them onto the
/// battlefield tapped, then shuffle." (THS)
pub fn burnished_hart() -> CardDefinition {
    use crate::card::ActivatedAbility;
    let fetch = Effect::Seq(vec![
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
        Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: true },
        },
    ]);
    CardDefinition {
        name: "Burnished Hart",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Elk], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            mana_cost: cost(&[generic(3)]),
            sac_cost: true,
            effect: fetch,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Doom Whisperer — {3}{B}{B} Creature — Nightmare Demon 6/6, Flying, Trample.
/// "Pay 2 life: Surveil 2." (GRN)
pub fn doom_whisperer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Doom Whisperer",
        cost: cost(&[generic(3), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Nightmare, CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            life_cost: 2,
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(2) },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Gifted Aetherborn — {B}{B} Creature — Aetherborn Vampire 2/3, Deathtouch,
/// Lifelink. (AER)
pub fn gifted_aetherborn() -> CardDefinition {
    CardDefinition {
        name: "Gifted Aetherborn",
        cost: cost(&[b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Aetherborn, CreatureType::Vampire],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Deathtouch, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Beast Whisperer — {2}{G}{G} Creature — Elf Druid 2/3. "Whenever you cast a
/// creature spell, draw a card." (M19)
pub fn beast_whisperer() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Beast Whisperer",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                },
            ),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Lotus Cobra — {1}{G} Creature — Snake 2/1. "Landfall — Whenever a land
/// enters the battlefield under your control, create a Treasure token." (ZEN)
pub fn lotus_cobra() -> CardDefinition {
    CardDefinition {
        name: "Lotus Cobra",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crate::game::effects::treasure_token(),
            },
        }],
        ..Default::default()
    }
}

/// Steel Overseer — {2} Artifact Creature — Construct 1/1. "{T}: Put a +1/+1
/// counter on each artifact creature you control." (M11)
pub fn steel_overseer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Steel Overseer",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Artifact
                        .and(SelectionRequirement::Creature)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Vault Skirge — {B/P} Artifact Creature — Imp 1/1, Flying, Lifelink. (MBS)
pub fn vault_skirge() -> CardDefinition {
    use crate::mana::ManaSymbol;
    CardDefinition {
        name: "Vault Skirge",
        cost: ManaCost { symbols: vec![ManaSymbol::Phyrexian(crate::mana::Color::Black)] },
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Imp], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        ..Default::default()
    }
}

/// Master of Etherium — {2}{U} Artifact Creature — Vedalken Artificer 0/0. "Its
/// power and toughness are each equal to the number of artifacts you control.
/// Other artifact creatures you control get +1/+1." (ALA)
pub fn master_of_etherium() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Master of Etherium",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vedalken, CreatureType::Artificer],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        static_abilities: vec![
            StaticAbility {
                description: "Power/toughness each equal the number of artifacts you control.",
                effect: StaticEffect::PumpSelfByControlledPermanents {
                    filter: SelectionRequirement::Artifact.and(SelectionRequirement::ControlledByYou),
                    per_power: 1,
                    per_toughness: 1,
                },
            },
            StaticAbility {
                description: "Other artifact creatures you control get +1/+1.",
                effect: StaticEffect::PumpPT {
                    applies_to: Selector::EachPermanent(
                        SelectionRequirement::Artifact
                            .and(SelectionRequirement::Creature)
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    power: 1,
                    toughness: 1,
                },
            },
        ],
        ..Default::default()
    }
}

/// Foundry Inspector — {3} Artifact Creature — Construct 3/2. "Artifact spells
/// you cast cost {1} less to cast." (KLD)
pub fn foundry_inspector() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Foundry Inspector",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Construct], ..Default::default() },
        power: 3,
        toughness: 2,
        static_abilities: vec![StaticAbility {
            description: "Artifact spells you cast cost {1} less to cast.",
            effect: StaticEffect::CostReduction {
                filter: SelectionRequirement::Artifact,
                amount: 1,
            },
        }],
        ..Default::default()
    }
}

/// Honor of the Pure — {1}{W} Enchantment. "White creatures you control get
/// +1/+1." (M10)
pub fn honor_of_the_pure() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Honor of the Pure",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "White creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(crate::mana::Color::White))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Benalish Marshal — {W}{W}{W} Creature — Human Knight 3/3. "Other creatures
/// you control get +1/+1." (DOM)
pub fn benalish_marshal() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Benalish Marshal",
        cost: cost(&[w(), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        static_abilities: vec![StaticAbility {
            description: "Other creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Luminarch Aspirant — {1}{W} Creature — Human Cleric 1/1. "At the beginning
/// of combat on your turn, put a +1/+1 counter on target creature you
/// control." (ZNR)
pub fn luminarch_aspirant() -> CardDefinition {
    CardDefinition {
        name: "Luminarch Aspirant",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::YourControl,
            ),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Crusade — {W}{W} Enchantment. "White creatures get +1/+1." (both players'). (LEA)
pub fn crusade() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Crusade",
        cost: cost(&[w(), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "White creatures get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(crate::mana::Color::White)),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Bad Moon — {1}{B} Enchantment. "Black creatures get +1/+0." (LEA)
pub fn bad_moon() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Bad Moon",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Black creatures get +1/+0.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasColor(crate::mana::Color::Black)),
                ),
                power: 1,
                toughness: 0,
            },
        }],
        ..Default::default()
    }
}

/// Dictate of Heliod — {3}{W}{W} Enchantment, Flash. "Creatures you control get
/// +2/+2." (JOU)
pub fn dictate_of_heliod() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Dictate of Heliod",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +2/+2.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: 2,
                toughness: 2,
            },
        }],
        ..Default::default()
    }
}

/// Gaea's Anthem — {1}{G}{G} Enchantment. "Creatures you control get +1/+1." (PLC)
pub fn gaeas_anthem() -> CardDefinition {
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Gaea's Anthem",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control get +1/+1.",
            effect: StaticEffect::PumpPT {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                power: 1,
                toughness: 1,
            },
        }],
        ..Default::default()
    }
}

/// Helper: "[type]s your opponents control enter the battlefield tapped."
fn opp_enters_tapped(req: SelectionRequirement, desc: &'static str) -> StaticAbility {
    StaticAbility {
        description: desc,
        effect: StaticEffect::EntersTapped {
            applies_to: Selector::EachPermanent(
                SelectionRequirement::ControlledByOpponent.and(req),
            ),
        },
    }
}

/// Imposing Sovereign — {1}{W} Creature — Human Noble 2/1. "Creatures your
/// opponents control enter the battlefield tapped." (M14)
pub fn imposing_sovereign() -> CardDefinition {
    CardDefinition {
        name: "Imposing Sovereign",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Noble],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        static_abilities: vec![opp_enters_tapped(
            SelectionRequirement::Creature,
            "Creatures your opponents control enter the battlefield tapped.",
        )],
        ..Default::default()
    }
}

/// Authority of the Consuls — {W} Enchantment. "Creatures your opponents
/// control enter the battlefield tapped. Whenever a creature an opponent
/// controls enters, you gain 1 life." (KLD)
pub fn authority_of_the_consuls() -> CardDefinition {
    CardDefinition {
        name: "Authority of the Consuls",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![opp_enters_tapped(
            SelectionRequirement::Creature,
            "Creatures your opponents control enter the battlefield tapped.",
        )],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::OpponentControl)
                .with_filter(crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Thalia, Heretic Cathar — {2}{W} Legendary Creature — Human Soldier 3/2,
/// First Strike. "Creatures and nonbasic lands your opponents control enter
/// the battlefield tapped." (EMN)
pub fn thalia_heretic_cathar() -> CardDefinition {
    CardDefinition {
        name: "Thalia, Heretic Cathar",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![opp_enters_tapped(
            SelectionRequirement::Creature.or(SelectionRequirement::IsNonbasicLand),
            "Creatures and nonbasic lands your opponents control enter the battlefield tapped.",
        )],
        ..Default::default()
    }
}

/// Blind Obedience — {1}{W} Enchantment with Extort. "Artifacts and creatures
/// your opponents control enter the battlefield tapped." (GTC)
pub fn blind_obedience() -> CardDefinition {
    CardDefinition {
        name: "Blind Obedience",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            opp_enters_tapped(
                SelectionRequirement::Artifact,
                "Artifacts your opponents control enter the battlefield tapped.",
            ),
            opp_enters_tapped(
                SelectionRequirement::Creature,
                "Creatures your opponents control enter the battlefield tapped.",
            ),
        ],
        triggered_abilities: vec![crate::effect::shortcut::extort()],
        ..Default::default()
    }
}

/// Helper: a vanilla creature body with Unleash (CR 702.98) — Keyword marker
/// plus the `shortcut::unleash()` ETB "may enter with a +1/+1 counter" trigger.
fn unleash_creature(
    name: &'static str,
    mana: ManaCost,
    types: Vec<CreatureType>,
    power: i32,
    toughness: i32,
) -> CardDefinition {
    CardDefinition {
        name,
        cost: mana,
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power,
        toughness,
        keywords: vec![Keyword::Unleash],
        triggered_abilities: vec![crate::effect::shortcut::unleash()],
        ..Default::default()
    }
}

/// Rakdos Cackler — {B/R} Creature — Devil 1/1 with Unleash. (RTR)
pub fn rakdos_cackler() -> CardDefinition {
    unleash_creature(
        "Rakdos Cackler",
        cost(&[crate::mana::hybrid(crate::mana::Color::Black, crate::mana::Color::Red)]),
        vec![CreatureType::Devil],
        1,
        1,
    )
}

/// Gore-House Chainwalker — {1}{R} Creature — Human Warrior 2/2 with Unleash. (RTR)
pub fn gore_house_chainwalker() -> CardDefinition {
    unleash_creature(
        "Gore-House Chainwalker",
        cost(&[generic(1), r()]),
        vec![CreatureType::Human, CreatureType::Warrior],
        2,
        2,
    )
}

/// Spawn of Rix Maadi — {3}{R} Creature — Zombie 2/2 with Unleash. (DIS)
pub fn spawn_of_rix_maadi() -> CardDefinition {
    unleash_creature(
        "Spawn of Rix Maadi",
        cost(&[generic(3), r()]),
        vec![CreatureType::Zombie],
        2,
        2,
    )
}

/// Helper: PumpPT + GrantKeyword(Vigilance) over a token/nontoken creature
/// anthem selector (Intangible Virtue / Always Watching share the shape).
fn vigilance_anthem(token: bool, pump_desc: &'static str) -> Vec<StaticAbility> {
    let sel = || {
        Selector::EachPermanent(
            SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)
                .and(if token {
                    SelectionRequirement::IsToken
                } else {
                    SelectionRequirement::NotToken
                }),
        )
    };
    vec![
        StaticAbility {
            description: pump_desc,
            effect: StaticEffect::PumpPT { applies_to: sel(), power: 1, toughness: 1 },
        },
        StaticAbility {
            description: "…and have vigilance.",
            effect: StaticEffect::GrantKeyword { applies_to: sel(), keyword: Keyword::Vigilance },
        },
    ]
}

/// Intangible Virtue — {1}{W} Enchantment. "Creature tokens you control get
/// +1/+1 and have vigilance." (ISD)
pub fn intangible_virtue() -> CardDefinition {
    CardDefinition {
        name: "Intangible Virtue",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vigilance_anthem(true, "Creature tokens you control get +1/+1."),
        ..Default::default()
    }
}

/// Always Watching — {2}{W} Enchantment. "Nontoken creatures you control get
/// +1/+1 and have vigilance." (SOI)
pub fn always_watching() -> CardDefinition {
    CardDefinition {
        name: "Always Watching",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vigilance_anthem(false, "Nontoken creatures you control get +1/+1."),
        ..Default::default()
    }
}

/// Kismet — {3}{W} Enchantment. "Artifacts, creatures, and lands your
/// opponents control enter the battlefield tapped." (LEG)
pub fn kismet() -> CardDefinition {
    CardDefinition {
        name: "Kismet",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            opp_enters_tapped(
                SelectionRequirement::Artifact,
                "Artifacts your opponents control enter the battlefield tapped.",
            ),
            opp_enters_tapped(
                SelectionRequirement::Creature,
                "Creatures your opponents control enter the battlefield tapped.",
            ),
            opp_enters_tapped(
                SelectionRequirement::Land,
                "Lands your opponents control enter the battlefield tapped.",
            ),
        ],
        ..Default::default()
    }
}

/// Viscera Seer — {B} Creature — Vampire Wizard 1/1. "Sacrifice a creature:
/// Scry 1." (M11)
pub fn viscera_seer() -> CardDefinition {
    CardDefinition {
        name: "Viscera Seer",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(1) },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: Some((SelectionRequirement::Creature, 1)),
            tap_other_filter: None,
            from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Thalia's Lieutenant — {1}{W}{W} Creature — Human Soldier 2/2. "When this
/// enters, put a +1/+1 counter on each other Human you control. Whenever
/// another Human enters under your control, put a +1/+1 counter on this." (EMN)
pub fn thalias_lieutenant() -> CardDefinition {
    CardDefinition {
        name: "Thalia's Lieutenant",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            crate::effect::shortcut::etb(Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Human)
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(crate::card::Predicate::EntityMatches {
                        what: Selector::TriggerSource,
                        filter: SelectionRequirement::HasCreatureType(CreatureType::Human)
                            .and(SelectionRequirement::OtherThanSource),
                    }),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
        ],
        ..Default::default()
    }
}

/// Mentor of the Meek — {2}{W} Creature — Human Soldier 2/2. "Whenever another
/// creature with power 2 or less enters under your control, you may pay {1}.
/// If you do, draw a card." (M13)
pub fn mentor_of_the_meek() -> CardDefinition {
    CardDefinition {
        name: "Mentor of the Meek",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::PowerAtMost(2))
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::MayPay {
                description: "Pay {1}: draw a card".into(),
                mana_cost: cost(&[generic(1)]),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

/// 1/1 white Soldier token (Hero of Bladehold).
fn white_soldier_token() -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: "Soldier".into(),
        power: 1,
        toughness: 1,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: vec![CreatureType::Soldier], ..Default::default() },
        ..Default::default()
    }
}

/// Hero of Bladehold — {2}{W}{W} Creature — Human Knight 3/4 with Battle cry.
/// "Whenever this attacks, create two 1/1 white Soldier creature tokens
/// tapped and attacking." (MBS)
pub fn hero_of_bladehold() -> CardDefinition {
    CardDefinition {
        name: "Hero of Bladehold",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        triggered_abilities: vec![
            crate::effect::shortcut::battle_cry(1),
            crate::effect::shortcut::on_attack(Effect::CreateTokenAttacking {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: white_soldier_token(),
                cleanup: Default::default(),
            }),
        ],
        ..Default::default()
    }
}

fn white_token(name: &'static str, p: i32, t: i32, types: Vec<CreatureType>,
    keywords: Vec<Keyword>) -> crate::card::TokenDefinition {
    crate::card::TokenDefinition {
        name: name.into(),
        power: p,
        toughness: t,
        card_types: vec![CardType::Creature],
        colors: vec![crate::mana::Color::White],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        keywords,
        ..Default::default()
    }
}

/// Knight Exemplar — {1}{W}{W} 2/2 Human Knight, First strike. Other Knights
/// you control get +1/+1 and have indestructible. (M11)
pub fn knight_exemplar() -> CardDefinition {
    let other_knights = || Selector::EachPermanent(
        SelectionRequirement::Creature
            .and(SelectionRequirement::HasCreatureType(CreatureType::Knight))
            .and(SelectionRequirement::ControlledByYou)
            .and(SelectionRequirement::OtherThanSource),
    );
    CardDefinition {
        name: "Knight Exemplar",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        static_abilities: vec![
            StaticAbility {
                description: "Other Knights you control get +1/+1.",
                effect: StaticEffect::PumpPT { applies_to: other_knights(), power: 1, toughness: 1 },
            },
            StaticAbility {
                description: "Other Knights you control have indestructible.",
                effect: StaticEffect::GrantKeyword {
                    applies_to: other_knights(),
                    keyword: Keyword::Indestructible,
                },
            },
        ],
        ..Default::default()
    }
}

/// Archangel of Thune — {3}{W}{W} 3/4 Angel, Flying + Lifelink. Whenever you
/// gain life, put a +1/+1 counter on each creature you control. (M14)
pub fn archangel_of_thune() -> CardDefinition {
    CardDefinition {
        name: "Archangel of Thune",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Angel], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Wingmate Roc — {3}{W}{W} 3/4 Bird, Flying. Raid: ETB if you attacked this
/// turn, create a 3/4 white Bird with flying. Whenever it attacks, gain 1
/// life for each attacking creature. (KTK)
pub fn wingmate_roc() -> CardDefinition {
    CardDefinition {
        name: "Wingmate Roc",
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Bird], ..Default::default() },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![
            crate::effect::shortcut::raid_etb(Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: white_token("Bird", 3, 4, vec![CreatureType::Bird], vec![Keyword::Flying]),
            }),
            crate::effect::shortcut::on_attack(Effect::GainLife {
                who: Selector::You,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::IsAttacking,
                ))),
            }),
        ],
        ..Default::default()
    }
}

/// Boros Elite — {W} 1/1 Human Soldier. Battalion — whenever this and at
/// least two other creatures attack, it gets +2/+2 until end of turn. (GTC)
pub fn boros_elite() -> CardDefinition {
    CardDefinition {
        name: "Boros Elite",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::battalion(Effect::PumpPT {
            what: Selector::This,
            power: Value::Const(2),
            toughness: Value::Const(2),
            duration: crate::effect::Duration::EndOfTurn,
        })],
        ..Default::default()
    }
}

/// Brimaz, King of Oreskos — {1}{W}{W} 3/4 Legendary Cat Soldier, Vigilance.
/// Whenever Brimaz attacks, create a 1/1 white Cat Soldier with vigilance
/// that's attacking. (The "blocks → blocking token" half is dropped — no
/// create-blocking-token primitive.) (BNG)
pub fn brimaz_king_of_oreskos() -> CardDefinition {
    CardDefinition {
        name: "Brimaz, King of Oreskos",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: white_token("Cat Soldier", 1, 1,
                vec![CreatureType::Cat, CreatureType::Soldier], vec![Keyword::Vigilance]),
            cleanup: Default::default(),
        })],
        ..Default::default()
    }
}

/// Adeline, Resplendent Cathar — {1}{W}{W} */4 Legendary Human Knight,
/// Vigilance. Her power equals the number of creatures you control. Whenever
/// she attacks, create a 1/1 white Human tapped and attacking (modeled per
/// the active opponent; the per-opponent fan-out collapses to one). (MID)
pub fn adeline_resplendent_cathar() -> CardDefinition {
    CardDefinition {
        name: "Adeline, Resplendent Cathar",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 0,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Adeline's power is equal to the number of creatures you control.",
            effect: StaticEffect::PumpSelfByControlledPermanents {
                filter: SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                per_power: 1,
                per_toughness: 0,
            },
        }],
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::CreateTokenAttacking {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: white_token("Human", 1, 1, vec![CreatureType::Human], vec![]),
            cleanup: Default::default(),
        })],
        ..Default::default()
    }
}

/// Mirran Crusader — {1}{W}{W} Creature — Human Knight 2/2 with Double strike,
/// protection from black, and protection from green. (MBS)
pub fn mirran_crusader() -> CardDefinition {
    CardDefinition {
        name: "Mirran Crusader",
        cost: cost(&[generic(1), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Knight],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![
            Keyword::DoubleStrike,
            Keyword::Protection(crate::mana::Color::Black),
            Keyword::Protection(crate::mana::Color::Green),
        ],
        ..Default::default()
    }
}

/// Plated Crusher — {4}{G}{G}{G} 7/6 Beast. Trample, Hexproof. (BFZ)
pub fn plated_crusher() -> CardDefinition {
    CardDefinition {
        name: "Plated Crusher",
        cost: cost(&[generic(4), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Hexproof],
        ..Default::default()
    }
}

/// Terra Stomper — {3}{G}{G}{G} 8/8 Beast. Can't be countered. Trample. (ZEN)
pub fn terra_stomper() -> CardDefinition {
    CardDefinition {
        name: "Terra Stomper",
        cost: cost(&[generic(3), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 8,
        toughness: 8,
        keywords: vec![Keyword::CantBeCountered, Keyword::Trample],
        ..Default::default()
    }
}

/// Rumbling Baloth — {2}{G}{G} 4/4 Beast. (M11)
pub fn rumbling_baloth() -> CardDefinition {
    CardDefinition {
        name: "Rumbling Baloth",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        ..Default::default()
    }
}

/// Charging Badger — {G} 1/1 Badger. Trample. (CNS)
pub fn charging_badger() -> CardDefinition {
    CardDefinition {
        name: "Charging Badger",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Badger], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// Bellowing Tanglewurm — {3}{G}{G} 4/4 Wurm. Intimidate. (SOM)
pub fn bellowing_tanglewurm() -> CardDefinition {
    CardDefinition {
        name: "Bellowing Tanglewurm",
        cost: cost(&[generic(3), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wurm], ..Default::default() },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Intimidate],
        ..Default::default()
    }
}

/// Vinelasher Kudzu — {1}{G} 1/1 Plant. Landfall — whenever a land you control
/// enters, put a +1/+1 counter on this creature. (ZEN)
pub fn vinelasher_kudzu() -> CardDefinition {
    CardDefinition {
        name: "Vinelasher Kudzu",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Plant], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LandPlayed, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Avatar of the Resolute — {G}{G} 3/2 Avatar. Reach, trample. Enters with a
/// +1/+1 counter for each other creature you control that has a +1/+1
/// counter (modeled as an ETB trigger). (AER)
pub fn avatar_of_the_resolute() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Avatar of the Resolute",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Avatar], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Reach, Keyword::Trample],
        triggered_abilities: vec![etb(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CountOf(Box::new(Selector::EachPermanent(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::OtherThanSource)
                    .and(SelectionRequirement::WithCounter(CounterType::PlusOnePlusOne)),
            ))),
        })],
        ..Default::default()
    }
}

/// Pelt Collector — {G} 1/1 Elf Warrior. Whenever another creature you
/// control enters or dies with greater power than this, put a +1/+1 counter
/// on Pelt Collector. While it has 3+ counters, it has trample. (GRN)
pub fn pelt_collector() -> CardDefinition {
    let bigger = || crate::card::Predicate::EntityMatches {
        what: Selector::TriggerSource,
        filter: SelectionRequirement::Creature
            .and(SelectionRequirement::OtherThanSource)
            .and(SelectionRequirement::PowerGreaterThanSource),
    };
    let grow = || Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Pelt Collector",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                    .with_filter(bigger()),
                effect: grow(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::YourControl)
                    .with_filter(bigger()),
                effect: grow(),
            },
        ],
        static_abilities: vec![StaticAbility {
            description: "As long as Pelt Collector has 3+ counters, it has trample.",
            effect: StaticEffect::PumpSelfIf {
                condition: crate::card::Predicate::ValueAtLeast(
                    Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                    Value::Const(3),
                ),
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Trample],
            },
        }],
        ..Default::default()
    }
}

/// Glen Elendra Archmage — {3}{U} 2/2 Faerie Wizard. Flying, Persist.
/// `{U}, Sacrifice this creature: Counter target noncreature spell.` (EVE)
pub fn glen_elendra_archmage() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Glen Elendra Archmage",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::Persist],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[u()]),
            sac_cost: true,
            effect: Effect::CounterSpell {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::Noncreature),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Helper: "{W}, {T}: Tap target creature." (Goldmeadow Harrier / Gideon's
/// Lawkeeper share this ability.)
fn tapper_1_1(name: &'static str, types: Vec<CreatureType>) -> CardDefinition {
    CardDefinition {
        name,
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: types, ..Default::default() },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            mana_cost: cost(&[w()]),
            effect: Effect::Tap { what: target_filtered(SelectionRequirement::Creature) },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
            sac_other_filter: None,
            tap_other_filter: None,
            from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Goldmeadow Harrier — {W} Creature — Kithkin Soldier 1/1. "{W}, {T}: Tap
/// target creature." (LRW)
pub fn goldmeadow_harrier() -> CardDefinition {
    tapper_1_1("Goldmeadow Harrier", vec![CreatureType::Kithkin, CreatureType::Soldier])
}

/// Gideon's Lawkeeper — {W} Creature — Human Soldier 1/1. "{W}, {T}: Tap
/// target creature." (ORI)
pub fn gideons_lawkeeper() -> CardDefinition {
    tapper_1_1("Gideon's Lawkeeper", vec![CreatureType::Human, CreatureType::Soldier])
}

/// Steppe Lynx — {W} Creature — Cat 0/1. "Landfall — Whenever a land you
/// control enters, this creature gets +2/+2 until end of turn." (ZEN)
pub fn steppe_lynx() -> CardDefinition {
    CardDefinition {
        name: "Steppe Lynx",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(crate::card::Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Land,
                }),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: crate::effect::Duration::EndOfTurn,
            },
        }],
        ..Default::default()
    }
}

/// Usher of the Fallen — {W} Creature — Human Soldier 1/1. "Boast — {1}{W}:
/// Create a 1/1 white Human Soldier creature token." (KHM)
pub fn usher_of_the_fallen() -> CardDefinition {
    CardDefinition {
        name: "Usher of the Fallen",
        cost: cost(&[w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![crate::effect::shortcut::boast(
            cost(&[generic(1), w()]),
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: white_soldier_token(),
            },
        )],
        ..Default::default()
    }
}

// ── Mana dorks & rocks ───────────────────────────────────────────────────────

/// Fyndhorn Elves — {G} Creature — Elf Druid 1/1. "{T}: Add {G}." (ICE)
pub fn fyndhorn_elves() -> CardDefinition {
    CardDefinition {
        name: "Fyndhorn Elves",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![crate::sets::tap_add(crate::mana::Color::Green)],
        ..Default::default()
    }
}

/// Druid of the Cowl — {1}{G} Creature — Elf Druid 1/3. "{T}: Add {G}." (M14)
pub fn druid_of_the_cowl() -> CardDefinition {
    CardDefinition {
        name: "Druid of the Cowl",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        activated_abilities: vec![crate::sets::tap_add(crate::mana::Color::Green)],
        ..Default::default()
    }
}

/// Manakin — {2} Artifact Creature — Construct 1/1. "{T}: Add {C}." (MIR)
pub fn manakin() -> CardDefinition {
    CardDefinition {
        name: "Manakin",
        cost: cost(&[generic(2)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![crate::sets::tap_add_colorless()],
        ..Default::default()
    }
}

/// Palladium Myr — {3} Artifact Creature — Myr 2/2. "{T}: Add {C}{C}." (SOM)
pub fn palladium_myr() -> CardDefinition {
    CardDefinition {
        name: "Palladium Myr",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Myr], ..Default::default() },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colorless(Value::Const(2)),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

// ── More black aggro / disruption ────────────────────────────────────────────

/// Diregraf Ghoul — {B} Creature — Zombie 2/2. "Diregraf Ghoul enters the
/// battlefield tapped." (ISD)
pub fn diregraf_ghoul() -> CardDefinition {
    CardDefinition {
        name: "Diregraf Ghoul",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Zombie], ..Default::default() },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap { what: Selector::This },
        }],
        ..Default::default()
    }
}

/// Pulse Tracker — {B} Creature — Vampire Rogue 1/1. "Whenever this attacks,
/// each opponent loses 1 life." (ZEN)
pub fn pulse_tracker() -> CardDefinition {
    CardDefinition {
        name: "Pulse Tracker",
        cost: cost(&[b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::on_attack(Effect::LoseLife {
            who: Selector::Player(PlayerRef::EachOpponent),
            amount: Value::Const(1),
        })],
        ..Default::default()
    }
}

/// Mesmeric Fiend — {1}{B} Creature — Horror 1/1. "When this enters, target
/// opponent reveals their hand and you choose a nonland card from it. Exile
/// that card until this creature leaves the battlefield." (TOR)
pub fn mesmeric_fiend() -> CardDefinition {
    use crate::card::ExileReturnZone;
    CardDefinition {
        name: "Mesmeric Fiend",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Horror], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ExileChosenUntilSourceLeaves {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
                return_to: ExileReturnZone::Hand,
            },
        }],
        ..Default::default()
    }
}

// ── Blue tempo / fliers / ninjas ─────────────────────────────────────────────

/// Faerie Seer — {U} Creature — Faerie Wizard 1/1 with Flying. "When this
/// enters, scry 2." (MH2)
pub fn faerie_seer() -> CardDefinition {
    CardDefinition {
        name: "Faerie Seer",
        cost: cost(&[u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![crate::effect::shortcut::etb_scry(2)],
        ..Default::default()
    }
}

/// Looter il-Kor — {1}{U} Creature — Kor Rogue 1/1 with Shadow. "Whenever
/// this deals combat damage to a player, draw a card, then discard a card." (TSP)
pub fn looter_il_kor() -> CardDefinition {
    CardDefinition {
        name: "Looter il-Kor",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Rogue],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Shadow],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::Discard { who: Selector::You, amount: Value::Const(1), random: false },
            ]),
        }],
        ..Default::default()
    }
}

/// Ninja of the Deep Hours — {1}{U} Creature — Human Ninja 2/2 with
/// Ninjutsu {1}{U}. "Whenever this deals combat damage to a player, you may
/// draw a card." (BOK)
pub fn ninja_of_the_deep_hours() -> CardDefinition {
    CardDefinition {
        name: "Ninja of the Deep Hours",
        cost: cost(&[generic(1), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Ninja],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Ninjutsu(cost(&[generic(1), u()]))],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Draw a card?".into(),
                body: Box::new(Effect::Draw { who: Selector::You, amount: Value::Const(1) }),
            },
        }],
        ..Default::default()
    }
}

// ── Green beaters ────────────────────────────────────────────────────────────

/// Kalonian Tusker — {G}{G} Creature — Beast 3/3 (vanilla). (M14)
pub fn kalonian_tusker() -> CardDefinition {
    CardDefinition {
        name: "Kalonian Tusker",
        cost: cost(&[g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 3,
        ..Default::default()
    }
}

/// Leatherback Baloth — {G}{G}{G} Creature — Beast 4/4 (vanilla). (ZEN)
pub fn leatherback_baloth() -> CardDefinition {
    CardDefinition {
        name: "Leatherback Baloth",
        cost: cost(&[g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 4,
        ..Default::default()
    }
}

/// Garruk's Companion — {1}{G} Creature — Beast 3/2 with Trample. (M11)
pub fn garruks_companion() -> CardDefinition {
    CardDefinition {
        name: "Garruk's Companion",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Beast], ..Default::default() },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        ..Default::default()
    }
}

/// River Boa — {1}{G} Creature — Snake 2/1 with Islandwalk and "{1}:
/// Regenerate this creature." (The printed regen cost is {G}; modeled with
/// the engine's generic `Keyword::Regenerate`.) (VIS)
pub fn river_boa() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "River Boa",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Landwalk(LandType::Island), Keyword::Regenerate(1)],
        ..Default::default()
    }
}

// ── French-vanilla filler ────────────────────────────────────────────────────

/// Razorfoot Griffin — {2}{W} Creature — Griffin 2/2 with Flying and First
/// strike. (7ED)
pub fn razorfoot_griffin() -> CardDefinition {
    CardDefinition {
        name: "Razorfoot Griffin",
        cost: cost(&[generic(2), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Griffin], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flying, Keyword::FirstStrike],
        ..Default::default()
    }
}

/// Canopy Spider — {1}{G} Creature — Spider 1/3 with Reach. (M10)
pub fn canopy_spider() -> CardDefinition {
    CardDefinition {
        name: "Canopy Spider",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spider], ..Default::default() },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        ..Default::default()
    }
}

/// Bog Wraith — {3}{B} Creature — Wraith 3/3 with Swampwalk. (LEA)
pub fn bog_wraith() -> CardDefinition {
    use crate::card::LandType;
    CardDefinition {
        name: "Bog Wraith",
        cost: cost(&[generic(3), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wraith], ..Default::default() },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Landwalk(LandType::Swamp)],
        ..Default::default()
    }
}

/// Spellseeker — {2}{U}, 1/1 Human Wizard. ETB: search your library for an
/// instant or sorcery with mana value 2 or less, put it into your hand.
pub fn spellseeker() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Spellseeker",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCardType(CardType::Instant)
                .or(SelectionRequirement::HasCardType(CardType::Sorcery))
                .and(SelectionRequirement::ManaValueAtMost(2)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Mystic Snake — {1}{G}{U}{U}, 2/2 Snake with Flash. ETB: counter target spell.
pub fn mystic_snake() -> CardDefinition {
    use crate::effect::shortcut::{etb, target_filtered};
    CardDefinition {
        name: "Mystic Snake",
        cost: cost(&[generic(1), g(), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        triggered_abilities: vec![etb(Effect::CounterSpell {
            what: target_filtered(SelectionRequirement::IsSpellOnStack),
        })],
        ..Default::default()
    }
}

/// Fauna Shaman — {1}{G}, 2/2 Elf Shaman. "{G}, {T}, Discard a creature
/// card: Search your library for a creature card, reveal it, put it into
/// your hand, then shuffle." Uses the `discard_cost` activation cost.
pub fn fauna_shaman() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Fauna Shaman",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
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

/// Ohran Frostfang — {3}{G}{G} Snow Snake 2/6. "Attacking creatures you
/// control have deathtouch. Whenever a creature you control deals combat
/// damage to a player, draw a card."
pub fn ohran_frostfang() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, StaticAbility, Supertype, TriggeredAbility};
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Ohran Frostfang",
        cost: cost(&[generic(3), g(), g()]),
        supertypes: vec![Supertype::Snow],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 2,
        toughness: 6,
        static_abilities: vec![StaticAbility {
            description: "Attacking creatures you control have deathtouch.",
            effect: StaticEffect::GrantKeywordToAttackers { keyword: Keyword::Deathtouch },
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::YourControl),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Ragavan, Nimble Pilferer — {R} Legendary Monkey Pirate 2/1. "Whenever
/// Ragavan deals combat damage to a player, create a Treasure token and
/// exile the top card of that player's library. Until end of turn, you may
/// cast that card." Dash {1}{R}.
pub fn ragavan_nimble_pilferer() -> CardDefinition {
    use crate::card::{
        EventKind, EventScope, EventSpec, MayPlayDuration, Supertype, TriggeredAbility,
    };
    use crate::effect::shortcut::dash;
    CardDefinition {
        name: "Ragavan, Nimble Pilferer",
        cost: cost(&[r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Monkey, CreatureType::Pirate],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: crate::game::effects::treasure_token(),
                },
                // Exile the top of the *damaged* player's library; you may
                // cast it until end of turn.
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::Target(0),
                    count: Value::Const(1),
                    duration: MayPlayDuration::EndOfThisTurn,
                },
            ]),
        }],
        alternative_cost: Some(dash(cost(&[generic(1), r()]))),
        ..Default::default()
    }
}

/// Dragon's Rage Channeler — {R} Human Shaman 1/1. "Whenever you cast a
/// noncreature spell, surveil 1. Delirium — As long as four or more card
/// types are among cards in your graveyard, this is a 3/3 with flying and
/// attacks each combat if able." Delirium rides `PumpSelfIf` (+2/+2 + Flying
/// + MustAttack while `DeliriumActive`).
pub fn dragons_rage_channeler() -> CardDefinition {
    use crate::card::{
        EventKind, EventScope, EventSpec, Predicate, StaticAbility, TriggeredAbility,
    };
    use crate::effect::{PlayerRef, StaticEffect};
    CardDefinition {
        name: "Dragon's Rage Channeler",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Noncreature,
                },
            ),
            effect: Effect::Surveil { who: PlayerRef::You, amount: Value::Const(1) },
        }],
        static_abilities: vec![StaticAbility {
            description: "Delirium — As long as there are four or more card types among \
                          cards in your graveyard, this creature is a 3/3, has flying, \
                          and attacks each combat if able.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::DeliriumActive { who: PlayerRef::You },
                power: 2,
                toughness: 2,
                keywords: vec![Keyword::Flying, Keyword::MustAttack],
            },
        }],
        ..Default::default()
    }
}

/// Glistener Elf — {G}, 1/1 Phyrexian Elf Warrior with Infect.
pub fn glistener_elf() -> CardDefinition {
    CardDefinition {
        name: "Glistener Elf",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Warrior],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Infect],
        ..Default::default()
    }
}

/// Imperial Recruiter — {2}{R}, 1/1 Human Advisor. ETB: search your library
/// for a creature with power 2 or less, put it into your hand.
pub fn imperial_recruiter() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Imperial Recruiter",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::Creature.and(SelectionRequirement::PowerAtMost(2)),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Goblin Matron — {2}{R}, 1/1 Goblin. ETB: search your library for a Goblin
/// card, put it into your hand.
pub fn goblin_matron() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Goblin Matron",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        triggered_abilities: vec![etb(Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::HasCreatureType(CreatureType::Goblin),
            to: ZoneDest::Hand(PlayerRef::You),
        })],
        ..Default::default()
    }
}

/// Loxodon Hierarch — {2}{G}{W}, 4/4 Elephant Cleric. ETB: gain 4 life.
/// "{G}{W}, Sacrifice this creature: Regenerate each creature you control."
pub fn loxodon_hierarch() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Loxodon Hierarch",
        cost: cost(&[generic(2), g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        triggered_abilities: vec![etb(Effect::GainLife {
            who: Selector::You,
            amount: Value::Const(4),
        })],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[g(), w()]),
            sac_cost: true,
            effect: Effect::Regenerate {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Fleecemane Lion — {G}{W}, 3/3 Cat. "{3}{G}{W}: Monstrosity 1. As long as
/// this creature is monstrous, it has hexproof and indestructible."
pub fn fleecemane_lion() -> CardDefinition {
    use crate::card::{Predicate, StaticAbility};
    use crate::effect::shortcut::monstrosity;
    use crate::effect::StaticEffect;
    CardDefinition {
        name: "Fleecemane Lion",
        cost: cost(&[g(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat], ..Default::default() },
        power: 3,
        toughness: 3,
        activated_abilities: vec![monstrosity(cost(&[generic(3), g(), w()]), 1)],
        static_abilities: vec![StaticAbility {
            description: "As long as this creature is monstrous, it has hexproof and \
                          indestructible.",
            effect: StaticEffect::PumpSelfIf {
                condition: Predicate::SourceIsMonstrous,
                power: 0,
                toughness: 0,
                keywords: vec![Keyword::Hexproof, Keyword::Indestructible],
            },
        }],
        ..Default::default()
    }
}

/// Ignoble Hierarch — {G} 0/1 Goblin Shaman. Exalted. {T}: Add {B}, {R}, or {G}.
pub fn ignoble_hierarch() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Ignoble Hierarch",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Shaman],
            ..Default::default()
        },
        power: 0,
        toughness: 1,
        triggered_abilities: vec![crate::effect::shortcut::exalted()],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColors(
                    vec![crate::mana::Color::Black, crate::mana::Color::Red, crate::mana::Color::Green],
                    Value::Const(1),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elves of Deep Shadow — {G} 1/1 Elf Druid. "{T}: Add {B}. This creature
/// deals 1 damage to you."
pub fn elves_of_deep_shadow() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Elves of Deep Shadow",
        cost: cost(&[g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![crate::mana::Color::Black]),
                },
                Effect::DealDamage { to: Selector::You, amount: Value::Const(1) },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Generous Ent — {5}{G} 5/7 Treefolk with Reach. ETB: create a Food token.
/// (Forestcycling is dropped — no typecycling primitive yet.)
pub fn generous_ent() -> CardDefinition {
    use crate::effect::shortcut::etb;
    CardDefinition {
        name: "Generous Ent",
        cost: cost(&[generic(5), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Treefolk], ..Default::default() },
        power: 5,
        toughness: 7,
        keywords: vec![Keyword::Reach],
        triggered_abilities: vec![etb(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: crabomination_base::tokens::food_token(),
        })],
        ..Default::default()
    }
}

/// Magus of the Mirror — {4}{B}{B} 4/2 Human Wizard. "{T}, Sacrifice this
/// creature: Exchange life totals with target opponent. Activate only during
/// your upkeep." A creature-bodied Mirror Universe (CR 119.7 + upkeep gate).
pub fn magus_of_the_mirror() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Magus of the Mirror",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            sac_cost: true,
            condition: Some(Predicate::All(vec![
                Predicate::IsTurnOf(PlayerRef::You),
                Predicate::CurrentStepIs(TurnStep::Upkeep),
            ])),
            effect: Effect::ExchangeLifeTotals {
                a: Selector::You,
                b: Selector::Player(PlayerRef::EachOpponent),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Underworld Dreams — {B}{B}{B} Enchantment. "Whenever an opponent draws a
/// card, this enchantment deals 1 damage to that player."
pub fn underworld_dreams() -> CardDefinition {
    CardDefinition {
        name: "Underworld Dreams",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDrawn, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
    }
}

/// Megrim — {2}{B} Enchantment. "Whenever an opponent discards a card, this
/// enchantment deals 2 damage to that player."
pub fn megrim() -> CardDefinition {
    CardDefinition {
        name: "Megrim",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::OpponentControl),
            effect: Effect::DealDamage {
                to: Selector::Player(PlayerRef::Triggerer),
                amount: Value::Const(2),
            },
        }],
        ..Default::default()
    }
}

/// Wall of Blood — {2}{B} 0/2 Wall with Defender. "Pay 1 life: This creature
/// gets +1/+1 until end of turn."
pub fn wall_of_blood() -> CardDefinition {
    CardDefinition {
        name: "Wall of Blood",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Wall], ..Default::default() },
        power: 0,
        toughness: 2,
        keywords: vec![Keyword::Defender],
        activated_abilities: vec![ActivatedAbility {
            life_cost: 1,
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Spidersilk Armor — {2}{G} Enchantment. "Creatures you control get +0/+1
/// and have reach."
pub fn spidersilk_armor() -> CardDefinition {
    let yours = || Selector::EachPermanent(
        SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
    );
    CardDefinition {
        name: "Spidersilk Armor",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "Creatures you control get +0/+1.",
                effect: StaticEffect::PumpPT { applies_to: yours(), power: 0, toughness: 1 },
            },
            StaticAbility {
                description: "Creatures you control have reach.",
                effect: StaticEffect::GrantKeyword { applies_to: yours(), keyword: Keyword::Reach },
            },
        ],
        ..Default::default()
    }
}

/// Fervor — {2}{R} Enchantment. "Creatures you control have haste."
pub fn fervor() -> CardDefinition {
    CardDefinition {
        name: "Fervor",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control have haste.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                keyword: Keyword::Haste,
            },
        }],
        ..Default::default()
    }
}

/// Furnace of Rath — {1}{R}{R}{R} Enchantment. "If a source would deal
/// damage to a permanent or player, it deals double that damage instead."
/// (CR 614.2 — `StaticEffect::DoubleDamageDealt`.)
pub fn furnace_of_rath() -> CardDefinition {
    CardDefinition {
        name: "Furnace of Rath",
        cost: cost(&[generic(1), r(), r(), r()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage, it deals double instead.",
            effect: StaticEffect::DoubleDamageDealt,
        }],
        ..Default::default()
    }
}

/// Dictate of the Twin Gods — {3}{R}{R} Enchantment with Flash. Doubles all
/// damage (CR 614.2), like Furnace of Rath but castable at instant speed.
pub fn dictate_of_the_twin_gods() -> CardDefinition {
    CardDefinition {
        name: "Dictate of the Twin Gods",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Enchantment],
        keywords: vec![Keyword::Flash],
        static_abilities: vec![StaticAbility {
            description: "If a source would deal damage, it deals double instead.",
            effect: StaticEffect::DoubleDamageDealt,
        }],
        ..Default::default()
    }
}

/// Windborn Muse — {3}{W} 2/3 Spirit with Flying and Propaganda's attack tax
/// (CR 508.1g — {2} per attacker to attack you).
pub fn windborn_muse() -> CardDefinition {
    CardDefinition {
        name: "Windborn Muse",
        cost: cost(&[generic(3), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Spirit], ..Default::default() },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you unless their controller pays {2} for each.",
            effect: StaticEffect::AttackTaxToController { amount: Value::Const(2), protect_planeswalkers: false },
        }],
        ..Default::default()
    }
}

/// Baird, Steward of Argive — {2}{W}{W} Legendary 2/4 Human Soldier with
/// Vigilance; creatures can't attack you or your planeswalkers unless their
/// controller pays {1} for each (CR 508.1g).
pub fn baird_steward_of_argive() -> CardDefinition {
    CardDefinition {
        name: "Baird, Steward of Argive",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        static_abilities: vec![StaticAbility {
            description: "Creatures can't attack you or your planeswalkers unless their controller pays {1} for each.",
            effect: StaticEffect::AttackTaxToController { amount: Value::Const(1), protect_planeswalkers: true },
        }],
        ..Default::default()
    }
}

/// Felidar Sovereign — {4}{W}{W} 4/6 Vigilance, Lifelink. At your upkeep, if
/// you have 40+ life, you win the game (CR 104.2a alt win-con).
pub fn felidar_sovereign() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Felidar Sovereign",
        cost: cost(&[generic(4), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Cat, CreatureType::Beast], ..Default::default() },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Vigilance, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::LifeOf(PlayerRef::You), Value::Const(40)),
                then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Test of Endurance — {2}{W}{W} Enchantment. At your upkeep, if you have 50+
/// life, you win the game.
pub fn test_of_endurance() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Test of Endurance",
        cost: cost(&[generic(2), w(), w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::Upkeep), EventScope::YourControl),
            effect: Effect::If {
                cond: Predicate::ValueAtLeast(Value::LifeOf(PlayerRef::You), Value::Const(50)),
                then: Box::new(Effect::WinGame { who: PlayerRef::You }),
                else_: Box::new(Effect::Noop),
            },
        }],
        ..Default::default()
    }
}

/// Seal of Doom — {2}{B} Enchantment. "Sacrifice this: Destroy target
/// nonblack creature." A pre-paid Doom Blade on a permanent.
pub fn seal_of_doom() -> CardDefinition {
    use crate::mana::Color;
    CardDefinition {
        name: "Seal of Doom",
        cost: cost(&[generic(2), b()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::Not(Box::new(SelectionRequirement::HasColor(Color::Black)))),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Seal of Primordium — {1}{G} Enchantment. "Sacrifice this: Destroy target
/// artifact or enchantment."
pub fn seal_of_primordium() -> CardDefinition {
    CardDefinition {
        name: "Seal of Primordium",
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::Artifact.or(SelectionRequirement::Enchantment),
                ),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Dark Prophecy — {B}{B}{B} Enchantment. Whenever a creature you control
/// dies, draw a card and lose 1 life.
pub fn dark_prophecy() -> CardDefinition {
    CardDefinition {
        name: "Dark Prophecy",
        cost: cost(&[b(), b(), b()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        ..Default::default()
    }
}

/// Seal of Strength — {G} Enchantment. "Sacrifice this: Target creature gets
/// +3/+3 until end of turn."
pub fn seal_of_strength() -> CardDefinition {
    CardDefinition {
        name: "Seal of Strength",
        cost: cost(&[g()]),
        card_types: vec![CardType::Enchantment],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(3),
                toughness: Value::Const(3),
                duration: crate::effect::Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Ajani's Welcome — {W} Enchantment. Whenever a creature enters under your
/// control, you gain 1 life.
pub fn ajanis_welcome() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Ajani's Welcome",
        cost: cost(&[w()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}

/// Impassioned Orator — {1}{W} 1/3 Human Cleric. Whenever another creature
/// enters under your control, you gain 1 life.
pub fn impassioned_orator() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Impassioned Orator",
        cost: cost(&[generic(1), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Human, CreatureType::Cleric], ..Default::default() },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::AnyPlayer)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::OtherThanSource),
                }),
            effect: Effect::GainLife { who: Selector::You, amount: Value::Const(1) },
        }],
        ..Default::default()
    }
}
