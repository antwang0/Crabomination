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
use crate::mana::{ManaCost, b, cost, g, generic, w};

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

/// Bloodghast — {B}{B}, 2/1 Vampire Spirit with Haste while opponent has
/// 10 or fewer life. Approximation: ships with `Haste` always — the
/// "haste while opp ≤ 10" gating needs a static-keyword-with-condition
/// primitive we don't have yet. The "return from graveyard when you play a
/// land" mechanic is also omitted.
pub fn bloodghast() -> CardDefinition {
    CardDefinition {
        name: "Bloodghast",
        cost: cost(&[b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Spirit],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Haste],
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

/// Loran of the Third Path — {1}{W}, 1/3 Legendary Human Artificer. Vigilance.
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
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
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
                    name: "Citizen",
                    power: 1,
                    toughness: 1,
                    keywords: vec![],
                    card_types: vec![CardType::Creature],
                    colors: vec![crate::mana::Color::Green],
                    supertypes: vec![],
                    subtypes: Subtypes::default(),
                    activated_abilities: vec![],
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

/// Cathar Commando — {2}{W}, 3/1 Human Soldier with Flash. {1}, Sacrifice
/// this: Destroy target artifact or enchantment. Uses the new
/// `sac_cost: true` flag so paying the activation cost sacrifices Cathar
/// Commando before its destroy effect resolves.
pub fn cathar_commando() -> CardDefinition {
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Cathar Commando",
        cost: cost(&[generic(2), w()]),
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
        cost: cost(&[r(), b()]),
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
pub fn temur_ascendancy() -> CardDefinition {
    use crate::effect::{Predicate, Selector as Sel, StaticEffect};
    use crate::mana::{r, u};
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
