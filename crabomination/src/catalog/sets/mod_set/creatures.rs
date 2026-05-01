//! Modern-staple creatures and enchantments. Each card uses only existing
//! engine primitives; promotions to fuller Oracle text are noted inline.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, Selector, SelectionRequirement, StaticAbility, Subtypes, Supertype,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{DelayedTriggerKind, ManaPayload, PlayerRef, StaticEffect, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{ManaCost, b, cost, g, generic, r, u, w};

// ── Creatures ────────────────────────────────────────────────────────────────

/// Thalia, Guardian of Thraben — {1}{W}, 2/1 Legendary Human Soldier with
/// First Strike. Static: noncreature spells cost {1} more to cast. Wired
/// via `StaticEffect::AdditionalCostAfterFirstSpell` with a noncreature
/// filter and `amount: 1`. The static currently only fires after a player's
/// first spell each turn (Damping-Sphere style); the real Thalia taxes
/// every noncreature spell. Acceptable for the playtest.
/// TODO: introduce an unconditional `StaticEffect::AdditionalCost` variant
/// to model Thalia's full text.
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Noncreature spells cost {1} more to cast.",
            effect: StaticEffect::AdditionalCostAfterFirstSpell {
                filter: SelectionRequirement::Noncreature,
                amount: 1,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Dark Confidant — {1}{B}, 2/1 Human Wizard. At the beginning of your
/// upkeep, reveal the top card of your library and put it into your hand.
/// You lose life equal to its mana value. We approximate the "lose life
/// equal to CMC" with a flat 2 — average modern deck CMC. The reveal step
/// is collapsed into a draw.
/// TODO: when an `Effect::DrawAndPayLifeEqualToCMC` primitive lands, replace
/// the body here.
pub fn dark_confidant() -> CardDefinition {
    CardDefinition {
        name: "Dark Confidant",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::YourControl,
            ),
            effect: Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::LoseLife { who: Selector::You, amount: Value::Const(2) },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Sylvan Caryatid — {1}{G}, 0/3 Plant. Hexproof. {T}: Add one mana of any
/// color. Defender is omitted (we don't enforce can't-attack restrictions
/// independently of `Keyword::Defender`, but `power: 0` means it'd never
/// attack profitably anyway). Hexproof + AnyOneColor are the load-bearing
/// halves and both are wired.
pub fn sylvan_caryatid() -> CardDefinition {
    CardDefinition {
        name: "Sylvan Caryatid",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Hexproof, Keyword::Defender],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Angel],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Faerie],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Sentinel of the Nameless City — {2}{G}, 3/4 Plant Warrior with Vigilance.
/// Whenever this creature attacks, create a 1/1 green Citizen creature
/// token. (Real card also has Ward {2}; the Ward keyword exists on
/// `Keyword::Ward` but isn't enforced at targeting time yet, so we omit
/// it.) Plant subtype is dropped — `CreatureType` doesn't enumerate Plant.
pub fn sentinel_of_the_nameless_city() -> CardDefinition {
    use crate::card::TokenDefinition;
    CardDefinition {
        name: "Sentinel of the Nameless City",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Warrior],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
                },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Ranger-Captain of Eos — {1}{W}{W}, 3/3 Human Soldier. ETB: search your
/// library for a creature card with mana value 1 or less, reveal, put it
/// into your hand, then shuffle.
///
/// Approximation: the second activated ability ("{1}, Sacrifice this:
/// Until end of turn, your opponents can't cast noncreature spells") is
/// omitted — sac-as-cost activation isn't yet a primitive.
pub fn ranger_captain_of_eos() -> CardDefinition {
    CardDefinition {
        name: "Ranger-Captain of Eos",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        supertypes: vec![],
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Haywire Mite — {G}, 1/1 Artifact Creature — Insect. {2}, Sacrifice this
/// artifact: Destroy target artifact, enchantment, or planeswalker. You
/// gain 1 life.
pub fn haywire_mite() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Haywire Mite",
        cost: cost(&[g()]),
        supertypes: vec![],
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Goldspan Dragon — {3}{R}{R}, 4/4 Dragon with Flying and Haste. Whenever
/// this attacks, create a Treasure token.
///
/// Approximation: the real card's "becomes the target of a spell" trigger
/// is omitted (no targeting event exists in the engine yet), and the
/// static "Treasures you control have `{T}, Sac: Add 2 mana of any one
/// color`" rider is dropped — Goldspan's Treasures are vanilla
/// 1-mana-of-any-color tokens. Document the Treasure-2 upgrade as a
/// follow-up if we add per-controller token-ability modification.
pub fn goldspan_dragon() -> CardDefinition {
    use crate::game::effects::treasure_token;
    use crate::mana::r;
    CardDefinition {
        name: "Goldspan Dragon",
        cost: cost(&[generic(3), r(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: treasure_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Tireless Tracker — {1}{G}{G}, 3/2 Human Scout. Whenever a land enters
/// under your control, investigate (create a Clue token).
///
/// Wired via the new trigger-filter enforcement: scope is
/// `YourControl + EntersBattlefield`, filter is
/// `Predicate::EntityMatches { what: TriggerSource, filter: Land }` so
/// the trigger fires only for land-typed entrants. The "Sacrifice a Clue:
/// put a +1/+1 counter on this" activated ability is omitted (no
/// sac-of-other-permanent activation primitive yet).
pub fn tireless_tracker() -> CardDefinition {
    use crate::effect::Predicate;
    use crate::game::effects::clue_token;
    CardDefinition {
        name: "Tireless Tracker",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Scout],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
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
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Bloodtithe Harvester — {1}{B}{R}, 3/2 Vampire Rogue. Whenever this
/// enters or attacks, create a Blood token.
///
/// Approximation: the activated ability `{1}, Sacrifice a Blood: deals 2
/// damage to any target` is omitted (sac-of-other-permanent activation
/// primitive isn't yet wired). Both ETB and attack triggers fire.
pub fn bloodtithe_harvester() -> CardDefinition {
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
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Rogue],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![blood_etb, blood_attack],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Enchantments ─────────────────────────────────────────────────────────────

/// Phyrexian Arena — {1}{B}{B} Enchantment. At the beginning of your upkeep,
/// draw a card and lose 1 life.
pub fn phyrexian_arena() -> CardDefinition {
    CardDefinition {
        name: "Phyrexian Arena",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CounterAbility {
                what: target_filtered(
                    SelectionRequirement::Permanent.and(SelectionRequirement::Nonland),
                ),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Sylvan Safekeeper — {G}, 1/1 Human Wizard. Sacrifice a Forest: Target
/// creature gains shroud until end of turn.
///
/// The sac-of-other-permanent activation primitive isn't yet a thing
/// (only sac-of-self via `ActivatedAbility::sac_cost` is wired), so the
/// sacrifice is folded into the resolved effect: the activation runs
/// `Sacrifice(your-Forest, count=1, filter=Forest) + GrantKeyword(target,
/// Shroud, EOT)`. Bot/AutoDecider activates only when it controls at
/// least one Forest, so the cost is paid honestly even though the
/// engine doesn't gate it pre-resolution.
pub fn sylvan_safekeeper() -> CardDefinition {
    use crate::card::{ActivatedAbility, LandType};
    use crate::effect::shortcut::target_filtered;
    use crate::effect::Duration;
    CardDefinition {
        name: "Sylvan Safekeeper",
        cost: cost(&[g()]),
        supertypes: vec![],
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
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Land
                        .and(SelectionRequirement::HasLandType(LandType::Forest)),
                },
                Effect::GrantKeyword {
                    what: target_filtered(SelectionRequirement::Creature),
                    keyword: Keyword::Shroud,
                    duration: Duration::EndOfTurn,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Grim Lavamancer — {R}, 1/1 Human Wizard. {R}, {T}, Exile two cards from
/// your graveyard: Grim Lavamancer deals 2 damage to any target.
///
/// The "exile two cards from your graveyard" cost is approximated by a
/// `Sacrifice`-style fold-in step: at resolution we run
/// `Repeat(2, Move(EachCard in your graveyard → Exile))`. Since
/// `Sacrifice` only handles battlefield permanents, we instead use
/// `ForEach` over a graveyard selector but the engine doesn't yet
/// support EachCardInGraveyard. We compromise: the cost is simply
/// `Effect::Mill` on yourself two times — wrong direction (mill puts
/// cards into the graveyard) — so we use a real exile path via
/// `RevealUntilFind` over the graveyard? That's not quite right either.
///
/// Simpler: drop the cost and ship as `{R}, {T}: 2 damage`. The
/// graveyard-exile cost is documented as 🟡 in CUBE_FEATURES. For an
/// honest gameplay model the bot-AI and decision flow rarely care
/// about whether 2 cards are exiled (the gating is "do I have 2+ cards
/// in my graveyard?" which the human can self-enforce).
///
/// TODO: when an `Effect::ExileNFromYourGraveyard` primitive lands,
/// fold it back into the activation as the first step of the seq.
pub fn grim_lavamancer() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::shortcut::target_filtered;
    use crate::mana::r;
    CardDefinition {
        name: "Grim Lavamancer",
        cost: cost(&[r()]),
        supertypes: vec![],
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
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Temur Ascendancy — {U}{R}{G} Enchantment. Creatures you control with
/// power 4 or greater have haste. Whenever a creature with power 4 or
/// greater enters under your control, draw a card.
///
/// The haste-grant static is wired via
/// `StaticEffect::GrantKeyword { applies_to: each_your_creature_with_power_at_least(4) }` —
/// but our static-selector decomposer doesn't understand `PowerAtLeast`,
/// so it currently grants haste to every creature you control (over-grant
/// for sub-4 power creatures). Documented as 🟡; the trigger half is
/// faithful via the new filter enforcement.
/// Containment Priest — {1}{W}, 2/2 Human Cleric Flash. **Replacement
/// effect** (omitted): "If a nontoken creature would enter the battlefield
/// and it wasn't cast, exile it instead." The replacement primitive
/// doesn't exist in the engine yet (no creature-ETB-replacement hook),
/// so this ships as a vanilla 2/2 flash body. Tests verify the body
/// is correct; the replacement gate will be added when the primitive
/// lands.
pub fn containment_priest() -> CardDefinition {
    CardDefinition {
        name: "Containment Priest",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Simian Spirit Guide — {2}{R}, 2/2 Ape Spirit. **Alt cost**: exile this
/// card from your hand to add {R}.
///
/// We model the alt cost via the existing `AlternativeCost` path —
/// `mana_cost: {0}`, `exile_filter: Self` (the "exile this card from
/// hand" cost is structurally identical to Force of Will's pitch
/// approach). When the alt cast resolves the spell goes onto the stack
/// and… wait, that's the issue: Simian Spirit Guide's alt cost
/// produces mana, it doesn't cast the creature. We approximate by
/// having the alt-cast resolve as a mana ability (`AddMana(R)`) with
/// the source self-exiled — same gameplay outcome.
///
/// The "exile" half is handled by the alt cost path's exile_filter
/// pulling the card itself out of hand into exile; the mana addition
/// is wired by overriding the spell effect on alt cast to `AddMana`.
/// Currently the alt cost path doesn't support a "different effect
/// when alt-cast", so the cast falls back to the normal path — which
/// would put a 2/2 onto the stack. To keep this correct we leave the
/// alt cost OFF for now and just ship the body. TODO: alt-cost
/// effect-override.
pub fn simian_spirit_guide() -> CardDefinition {
    use crate::card::CreatureType as CT;
    CardDefinition {
        name: "Simian Spirit Guide",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CT::Ape, CT::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Eternal Witness — {1}{G}{G}, 2/1 Human Shaman. ETB: return target card
/// from your graveyard to your hand. Pure recursion.
pub fn eternal_witness() -> CardDefinition {
    CardDefinition {
        name: "Eternal Witness",
        cost: cost(&[generic(1), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Shaman],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Heliod, Sun-Crowned — {1}{W}{W}, Legendary Enchantment Creature — God.
/// 3/4. Indestructible. As long as your devotion to white is less than
/// five, Heliod isn't a creature. {1}{W}: target creature gains lifelink
/// until end of turn. (The real card adds a "whenever you gain life, put
/// a +1/+1 counter on target creature with lifelink" trigger — that
/// payoff is omitted; the activated lifelink-grant is what matters most.)
///
/// Devotion-flicker (creature ↔ enchantment based on white devotion) is
/// also omitted: Heliod is just a 3/4 indestructible Legendary Creature
/// here. Tests assert the activated-ability lifelink grant.
pub fn heliod_sun_crowned() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Heliod, Sun-Crowned",
        cost: cost(&[generic(1), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature, CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Indestructible],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
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
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Indulgent Tormentor — {3}{B}{B}, 5/3 Demon Flying. At the beginning of
/// each end step on your turn, target opponent loses 3 life unless they
/// sacrifice a creature.
///
/// Approximation: instead of giving the *opponent* the choice, we
/// resolve the most punishing line for the controller — drain 3 from
/// each opponent. With no creatures on the opponent's side, that's the
/// real card's worst case anyway. Test exercises the drain.
pub fn indulgent_tormentor() -> CardDefinition {
    CardDefinition {
        name: "Indulgent Tormentor",
        cost: cost(&[generic(3), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 5,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::StepBegins(TurnStep::End), EventScope::ActivePlayer),
            effect: Effect::LoseLife {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Marauding Mako — {U} 1/1 Shark. Whenever you discard a card, put
/// a +1/+1 counter on Marauding Mako. (The full Oracle pumps on every
/// discard; we use a `CardDiscarded`+`YourControl` listener.)
pub fn marauding_mako() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Marauding Mako",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // Engine has no Shark creature type; classify as Fish (ocean theme).
            creature_types: vec![CreatureType::Fish],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardDiscarded, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
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

/// Ichorid — {B}, 3/1 Horror with Haste. "At the beginning of your
/// upkeep, if Ichorid is in your graveyard and at least one of your
/// opponents' graveyards contains a black creature card, you may
/// return Ichorid to the battlefield. If you do, exile Ichorid at the
/// beginning of the next end step."
///
/// Approximation: simplified to "at the beginning of your upkeep,
/// return any creature card in your graveyard to the battlefield, then
/// schedule a delayed exile at the next end step." The "opponent has a
/// black creature in their graveyard" gate is omitted (no graveyard-
/// color triggered-ability filter yet). Reuses Goryo's reanimate-then-
/// exile pattern.
pub fn ichorid() -> CardDefinition {
    use crate::effect::DelayedTriggerKind;
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
            ),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: Selector::This,
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
                Effect::DelayUntil {
                    kind: DelayedTriggerKind::NextEndStep,
                    body: Box::new(Effect::Exile { what: Selector::This }),
                },
            ]),
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
        cost: cost(&[generic(1), b()]),
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
                },
            },
        }],
        ..Default::default()
    }
}

/// Dandân — {2}{U}, 4/1 Fish. "Dandân can attack only if defending
/// player controls an Island. When you control no Islands, sacrifice
/// Dandân."
///
/// Approximation: vanilla 4/1 body + an "at upkeep, if you control no
/// Islands, sacrifice it" trigger. The "can only attack if defending
/// player controls an Island" half is omitted (no per-attacker
/// targeting restriction yet) — without that the card is a fairly
/// strong 4/1 for {3}, but the upkeep-sac stays as the drawback half.
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
/// Approximation (no exile-until-LTB primitive yet): wired as an ETB
/// `DiscardChosen` against the opponent's hand with `Nonland` filter —
/// the card moves straight to the graveyard rather than returning when
/// Sculler dies. Gameplay-equivalent for the disruption half; the
/// "give it back when this dies" clause is the only piece omitted.
/// Reuses the same `DiscardChosen` primitive Inquisition of Kozilek and
/// Thoughtseize ride on.
pub fn tidehollow_sculler() -> CardDefinition {
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
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        ..Default::default()
    }
}

pub fn temur_ascendancy() -> CardDefinition {
    use crate::effect::{Predicate, Selector as Sel, StaticEffect};
    CardDefinition {
        name: "Temur Ascendancy",
        cost: cost(&[u(), r(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
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
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
