//! Secrets of Strixhaven (SOS) — Creatures.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility,
};
use crate::effect::shortcut::etb_gain_life;
use crate::effect::{Duration, PlayerRef, Selector, Value};
use crate::mana::{Color, ManaCost, b, cost, generic, w};

// ── Strixhaven token helpers ────────────────────────────────────────────────

/// 1/1 white-and-black Inkling creature token with flying. Used by several
/// SOS Silverquill / White cards.
pub fn inkling_token() -> TokenDefinition {
    TokenDefinition {
        name: "Inkling".into(),
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        card_types: vec![CardType::Creature],
        colors: vec![Color::White, Color::Black],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

// ── White ───────────────────────────────────────────────────────────────────

/// Eager Glyphmage — {3}{W}, 3/3 Cat Cleric.
/// "When this creature enters, create a 1/1 white and black Inkling creature
/// token with flying."
pub fn eager_glyphmage() -> CardDefinition {
    CardDefinition {
        name: "Eager Glyphmage",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Cat, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: inkling_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Stirring Hopesinger — {2}{W}, 1/3 Bird Bard. Flying, lifelink.
///
/// Fully wired: flying + lifelink body, plus the Repartee trigger
/// ("whenever you cast an instant or sorcery that targets a creature,
/// put a +1/+1 counter on each creature you control") via the
/// `repartee()` shortcut chained over a `ForEach` of your creatures.
pub fn stirring_hopesinger() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Stirring Hopesinger",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Bard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Repartee — "Whenever you cast an instant or sorcery spell that
        // targets a creature, put a +1/+1 counter on each creature you
        // control." Iterate via `ForEach` over creatures controlled by
        // the trigger's controller; the body inherits `TriggerSource`
        // bound to each iterated creature, so a per-iteration
        // `AddCounter { what: TriggerSource }` lands the counters one at
        // a time.
        triggered_abilities: vec![repartee(Effect::ForEach {
            selector: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Rehearsed Debater — {2}{W}, 3/3 Djinn Bard. Vigilance.
/// "Repartee — Whenever you cast an instant or sorcery spell that targets
/// a creature, this creature gets +1/+1 until end of turn."
///
/// Refactored to use the `repartee_self_pump` helper — was a 5-line
/// `Effect::PumpPT { what: Selector::This, … }` boilerplate, now one
/// line. The semantics are identical.
pub fn rehearsed_debater() -> CardDefinition {
    use crate::effect::shortcut::repartee_self_pump;
    CardDefinition {
        name: "Rehearsed Debater",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee_self_pump(1, 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Informed Inkwright — {1}{W}, 2/2 Human Wizard. Vigilance.
/// "Repartee — Whenever you cast an instant or sorcery spell that
/// targets a creature, create a 1/1 white and black Inkling creature
/// token with flying."
///
/// Wired via the `repartee()` shortcut (instant-or-sorcery + spell-
/// targets-creature predicate) plus `Effect::CreateToken` minting the
/// shared Inkling token under the controller.
pub fn informed_inkwright() -> CardDefinition {
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Informed Inkwright",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::CreateToken {
            who: PlayerRef::You,
            count: Value::Const(1),
            definition: inkling_token(),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Owlin Historian — {2}{W}, 2/3 Bird Cleric. Flying.
/// "Flying / When this creature enters, surveil 1. / Whenever one or more
/// cards leave your graveyard, this creature gets +1/+1 until end of
/// turn."
///
/// Fully wired: flying body, the ETB Surveil 1, and the "whenever one or
/// more cards leave your graveyard, +1/+1 until end of turn" pump via an
/// `EventKind::CardLeftGraveyard` trigger.
pub fn owlin_historian() -> CardDefinition {
    CardDefinition {
        name: "Owlin Historian",
        cost: cost(&[generic(2), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: Surveil 1.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            },
            // Whenever one or more cards leave your graveyard, this
            // creature gets +1/+1 EOT (per-card emission, see
            // `EventKind::CardLeftGraveyard` in TODO.md).
            TriggeredAbility {
                event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
                effect: Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Inkshape Demonstrator — {3}{W}, 3/4 Elephant Cleric. "Ward {2}.
/// Repartee — Whenever you cast an instant or sorcery spell that
/// targets a creature, this creature gets +1/+0 and gains lifelink
/// until end of turn."
///
/// Fully wired: `Ward {2}` via `Keyword::Ward(WardCost::generic(2))`, and
/// the Repartee body via the `repartee()` shortcut (pump +1/+0 on the
/// source + grant Lifelink until end of turn). 3/4 Elephant Cleric body.
pub fn inkshape_demonstrator() -> CardDefinition {
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Inkshape Demonstrator",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Ward(crate::card::WardCost::generic(2))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::Seq(vec![
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Lifelink,
                duration: Duration::EndOfTurn,
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Silverquill (White-Black) ───────────────────────────────────────────────

/// Inkling Mascot — {W}{B}, 2/2 Inkling Cat.
/// "Repartee — Whenever you cast an instant or sorcery spell that targets
/// a creature, this creature gains flying until end of turn. Surveil 1."
///
/// Wired via the `repartee()` shortcut: the trigger fires when the
/// controller casts an instant/sorcery targeting a creature, granting
/// the source flying until end of turn and following with a Surveil 1.
pub fn inkling_mascot() -> CardDefinition {
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Inkling Mascot",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Inkling, CreatureType::Cat],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::Seq(vec![
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Flying,
                duration: Duration::EndOfTurn,
            },
            Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Imperious Inkmage — {1}{W}{B}, 3/3 Orc Warlock. Vigilance.
/// "When this creature enters, surveil 2."
pub fn imperious_inkmage() -> CardDefinition {
    CardDefinition {
        name: "Imperious Inkmage",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Surveil {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Black ───────────────────────────────────────────────────────────────────

/// Sneering Shadewriter — {4}{B}, 3/3 Vampire Warlock. Flying.
/// "When this creature enters, each opponent loses 2 life and you gain 2
/// life."
pub fn sneering_shadewriter() -> CardDefinition {
    CardDefinition {
        name: "Sneering Shadewriter",
        cost: cost(&[generic(4), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::LoseLife {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(2),
                },
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Burrog Banemaker — {B}, 1/1 Frog Warlock. Deathtouch.
/// "{1}{B}: This creature gets +1/+1 until end of turn."
pub fn burrog_banemaker() -> CardDefinition {
    CardDefinition {
        name: "Burrog Banemaker",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Green ───────────────────────────────────────────────────────────────────

/// Noxious Newt — {1}{G}, 1/2 Salamander. Deathtouch. "{T}: Add {G}."
pub fn noxious_newt() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::mana::g;
    CardDefinition {
        name: "Noxious Newt",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Salamander],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Mindful Biomancer — {1}{G}, 2/2 Dryad Druid.
/// "When this creature enters, you gain 1 life. / {2}{G}: This creature
/// gets +2/+2 until end of turn. Activate only once each turn."
pub fn mindful_biomancer() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Mindful Biomancer",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // Dryad isn't in CreatureType yet; bridge through Druid which is.
            creature_types: vec![CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2), g()]),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(2),
                toughness: Value::Const(2),
                duration: Duration::EndOfTurn,
            },
            once_per_turn: true,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![etb_gain_life(1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Shopkeeper's Bane — {2}{G}, 4/2 Badger Pest. Trample.
/// "Whenever this creature attacks, you gain 2 life."
pub fn shopkeepers_bane() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Shopkeeper's Bane",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Badger, CreatureType::Pest],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Studious First-Year // Rampant Growth — modal-double-faced.
///
/// Front: 1/1 Bear Wizard at {G}. Vanilla body — printed has no rules
/// text (the back face carries the spell side).
///
/// Back: {1}{G} Sorcery — search your library for a basic land card,
/// put it onto the battlefield tapped, then shuffle.
///
/// Wired via the engine's existing MDFC plumbing: the front-face
/// `CardDefinition` carries a `back_face: Some(Box<...>)` pointer to a
/// freshly-built sorcery `CardDefinition`. Players cast either face by
/// emitting `GameAction::CastSpell` (front) or
/// `GameAction::CastSpellBack` (back, added in this push). The back's
/// effect is the same body Rampant Growth uses.
pub fn studious_first_year() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::ZoneDest;
    use crate::mana::g;
    let back = CardDefinition {
        name: "Rampant Growth",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Search {
            who: PlayerRef::You,
            filter: SelectionRequirement::IsBasicLand,
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: true,
            },
        },
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    };
    CardDefinition {
        name: "Studious First-Year",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bear, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: Some(Box::new(back)),
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Witherbloom (B/G) ───────────────────────────────────────────────────────

/// Bogwater Lumaret — {B}{G}, 2/2 Spirit Frog.
/// "Whenever this creature or another creature you control enters, you gain
/// 1 life."
pub fn bogwater_lumaret() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::Predicate;
    use crate::mana::g;
    CardDefinition {
        name: "Bogwater Lumaret",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Frog],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Creature,
                }),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Pest Mascot — {1}{B}{G}, 2/3 Pest Ape. Trample.
/// "Whenever you gain life, put a +1/+1 counter on this creature."
pub fn pest_mascot() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::g;
    CardDefinition {
        name: "Pest Mascot",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest, CreatureType::Ape],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Blech, Loafing Pest — {1}{B}{G}, 3/4 Legendary Creature — Pest.
/// "Whenever you gain life, put a +1/+1 counter on each Pest, Bat,
/// Insect, Snake, and Spider you control."
///
/// Implementation: a `LifeGained` (`EventScope::YourControl`) trigger
/// fans out via `ForEach` over creatures controlled by the trigger's
/// controller filtered to any of the printed five creature types
/// (Pest / Bat / Insect / Snake / Spider). Each iteration drops one
/// `+1/+1` counter on the iterated creature. The lifegain event itself
/// already coalesces the gain into a single trigger fire (per MTG rules
/// 119.10), so a 5-life gain doesn't stack-fire 5 trigger copies.
pub fn blech_loafing_pest() -> CardDefinition {
    use crate::card::{CounterType, Supertype};
    use crate::mana::g;
    CardDefinition {
        name: "Blech, Loafing Pest",
        cost: cost(&[generic(1), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Pest],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::LifeGained, EventScope::YourControl),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(
                            SelectionRequirement::HasCreatureType(CreatureType::Pest)
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Bat))
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Insect))
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Snake))
                                .or(SelectionRequirement::HasCreatureType(CreatureType::Spider)),
                        ),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Red ─────────────────────────────────────────────────────────────────────

/// Rearing Embermare — {4}{R}, 4/5 Horse Beast. Reach, haste.
pub fn rearing_embermare() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Rearing Embermare",
        cost: cost(&[generic(4), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Horse, CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Reach, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional White ────────────────────────────────────────────────────────

/// Ascendant Dustspeaker — {4}{W}, 3/4 Orc Cleric. Flying.
/// "When this creature enters, put a +1/+1 counter on another target
/// creature you control. / At the beginning of combat on your turn, exile
/// up to one target card from a graveyard."
///
/// Wired with both triggers. The combat trigger uses a graveyard-card
/// target; AutoDecider picks the first eligible card if any are
/// available, or no-ops the trigger when graveyards are empty.
pub fn ascendant_dustspeaker() -> CardDefinition {
    use crate::card::{CounterType, SelectionRequirement};
    use crate::effect::{ZoneDest};
    use crate::effect::shortcut::target_filtered;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Ascendant Dustspeaker",
        cost: cost(&[generic(4), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou)
                            .and(SelectionRequirement::OtherThanSource),
                    ),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::InGraveyard),
                    to: ZoneDest::Exile,
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Shattered Acolyte — {1}{W}, 2/2 Dwarf Warlock. Lifelink.
/// "{1}, Sacrifice this creature: Destroy target artifact or enchantment."
pub fn shattered_acolyte() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Shattered Acolyte",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // No "Dwarf" subtype yet — Warlock alone is the gameplay-relevant one.
            creature_types: vec![CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1)]),
            effect: Effect::Destroy {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Artifact)
                        .or(SelectionRequirement::HasCardType(CardType::Enchantment)),
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
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Summoned Dromedary — {3}{W}, 4/3 Spirit Camel. Vigilance.
/// "{1}{W}: Return this card from your graveyard to your hand. Activate
/// only as a sorcery."
///
/// Wired in push XVII: the graveyard-recursion activated ability uses
/// the new `ActivatedAbility.from_graveyard: bool` field. The
/// `activate_ability` engine path now walks the graveyard when the
/// ability is flagged. Cost `{1}{W}` + sorcery-speed + effect
/// `Move(Self → Hand(You))`. The source is found in the graveyard via
/// `move_card_to`'s existing graveyard branch.
pub fn summoned_dromedary() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Summoned Dromedary",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), w()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Hand(PlayerRef::You),
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Silverquill (W/B) ────────────────────────────────────────────

/// Stirring Honormancer — {2}{W}{W/B}{B}, 4/5 Rhino Bard.
/// "When this creature enters, look at the top X cards of your library,
/// where X is the number of creatures you control. Put one of those
/// cards into your hand and the rest into your graveyard."
///
/// Approximation: implemented via `Effect::RevealUntilFind` with a
/// `Creature` filter and `cap = CountOf(EachPermanent(Creature &
/// ControlledByYou))`. The found creature card goes to your hand; cards
/// revealed *before* it are milled. Per-card semantics match the
/// printed card most of the time (when at least one card in the top X
/// is a creature). The deviation: cards *after* the found creature
/// stay on top of your library instead of going to the graveyard. This
/// is a small fidelity gap but doesn't affect the immediate-gain side
/// (a creature in hand) — and the typical case (X = 2-4 creatures and
/// the top card is the chosen creature) matches the printed result.
pub fn stirring_honormancer() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Stirring Honormancer",
        // {2}{W}{W/B}{B}: the {W/B} pip is a real `ManaSymbol::Hybrid`
        // (CMC 5), payable with either white or black.
        cost: cost(&[generic(2), w(), crate::mana::hybrid(Color::White, Color::Black), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // No "Rhino" subtype yet — bridge through Bard alone.
            creature_types: vec![CreatureType::Bard],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Creature,
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ))),
                life_per_revealed: 0,
                miss_dest: crate::effect::RevealMissDest::Graveyard,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Conciliator's Duelist — {W}{W}{B}{B}, 4/3 Kor Warlock.
/// "When this creature enters, draw a card. Each player loses 1 life."
///
/// **Repartee** — "Whenever you cast an instant or sorcery spell that
/// targets a creature, exile up to one target creature. Return that
/// card to the battlefield under its owner's control at the beginning
/// of the next end step."
///
/// Push (modern_decks): the "return at next end step" delayed rider is
/// **now wired** via an extension to `Effect::DelayUntil` that falls
/// back to `Selector::CastSpellTarget(0)` (the just-cast spell's
/// target) when `ctx.targets` is empty. The Repartee trigger fires
/// `Seq(Exile(CastSpellTarget(0)) + DelayUntil(NextEndStep, Move →
/// Battlefield(Owner)))`; the DelayUntil capture-fallback pulls the
/// cast spell's target off the stack and stashes it so the next-end-
/// step body's `Selector::Target(0)` resolves back to the exiled
/// creature.
pub fn conciliators_duelist() -> CardDefinition {
    use crate::effect::{shortcut::repartee, DelayedTriggerKind, ZoneDest};
    CardDefinition {
        name: "Conciliator's Duelist",
        cost: cost(&[w(), w(), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // No "Kor" subtype yet — Warlock alone covers the gameplay-
            // relevant interactions (Witherbloom payoffs, etc.).
            creature_types: vec![CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            // Repartee — exile the cast spell's target creature, then
            // bring it back at next end step. The DelayUntil captures
            // the cast-spell target via the CastSpellTarget(0) fallback.
            repartee(Effect::Seq(vec![
                Effect::Exile {
                    what: Selector::CastSpellTarget(0),
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
            ])),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Black ────────────────────────────────────────────────────────

/// Lecturing Scornmage — {B}, 1/1 Human Warlock.
/// "Repartee — Whenever you cast an instant or sorcery spell that targets
/// a creature, put a +1/+1 counter on this creature."
pub fn lecturing_scornmage() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Lecturing Scornmage",
        cost: cost(&[b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Warlock],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::AddCounter {
            what: Selector::This,
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Melancholic Poet — {1}{B}, 2/2 Elf Bard.
/// "Repartee — Whenever you cast an instant or sorcery spell that targets
/// a creature, each opponent loses 1 life and you gain 1 life."
pub fn melancholic_poet() -> CardDefinition {
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Melancholic Poet",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::Drain {
            from: Selector::Player(PlayerRef::EachOpponent),
            to: Selector::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Silverquill (W/B) ────────────────────────────────────────────

/// Snooping Page — {1}{W}{B}, 2/3 Human Cleric.
/// "Repartee — Whenever you cast an instant or sorcery spell that targets
/// a creature, this creature can't be blocked this turn. / Whenever this
/// creature deals combat damage to a player, you draw a card and lose 1
/// life."
///
/// The Repartee trigger grants `Keyword::Unblockable` to the source
/// until end of turn (engine reads `Unblockable` at block-declaration
/// time). The combat-damage trigger is wired with the standard
/// `DealsCombatDamageToPlayer` event.
pub fn snooping_page() -> CardDefinition {
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Snooping Page",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            repartee(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            }),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::DealsCombatDamageToPlayer,
                    EventScope::SelfSource,
                ),
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Scolding Administrator — {W}{B}, 2/2 Dwarf Cleric. Menace. Repartee
/// (whenever you cast an instant or sorcery spell that targets a
/// creature, put a +1/+1 counter on this creature). When this creature
/// dies, if it had counters on it, put those counters on up to one
/// target creature.
///
/// ✅ All three abilities wired. The dies-trigger uses the cross-zone-
/// search behavior of `Value::CountersOn` (push XXIII) — after death
/// the source is in the graveyard, but counters persist on its
/// `CardInstance` and the lookup walks the gy, returning the live
/// counter count. The trigger is gated via `Effect::If` on
/// `ValueAtLeast(CountersOn(This), 1)` so the trigger only fires the
/// AddCounter body when Scolding Administrator had at least one
/// +1/+1 counter at death. The printed "up to one target creature"
/// collapses to a required Creature target (engine has no "up to one"
/// optional-target primitive — the trigger fizzles benignly if no
/// legal creature exists at resolution).
///
/// Per CR 122.8, the printed "those counters" wording would cancel
/// the move if the source had left the battlefield; the engine's
/// cross-zone counter lookup re-reads the dying source's counters
/// from the graveyard, which is the gameplay-equivalent of Wizards'
/// errata-pattern of restating "transfer those counters" as
/// "put N counters" with N captured at trigger-fire time. Star
/// Pupil takes the more conservative "single +1/+1 counter" route;
/// Scolding Administrator preserves variability (Repartee can stack
/// multiple counters on it before death).
pub fn scolding_administrator() -> CardDefinition {
    use crate::card::{CounterType, Predicate, SelectionRequirement};
    use crate::effect::shortcut::{repartee, target_filtered};
    CardDefinition {
        name: "Scolding Administrator",
        cost: cost(&[w(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // Repartee — +1/+1 counter on this creature.
            repartee(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            // Dies → if it had counters, put those counters on a target
            // creature. Wrapped in `Effect::If` so the body only runs
            // when the source actually died with +1/+1 counters.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::If {
                    cond: Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::PlusOnePlusOne,
                        },
                        Value::Const(1),
                    ),
                    then: Box::new(Effect::AddCounter {
                        what: target_filtered(SelectionRequirement::Creature),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::CountersOn {
                            what: Box::new(Selector::This),
                            kind: CounterType::PlusOnePlusOne,
                        },
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Red ──────────────────────────────────────────────────────────

/// Zealous Lorecaster — {5}{R}, 4/4 Giant Sorcerer.
/// "When this creature enters, return target instant or sorcery card from
/// your graveyard to your hand."
pub fn zealous_lorecaster() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::ZoneDest;
    use crate::effect::shortcut::target_filtered;
    use crate::mana::r;
    CardDefinition {
        name: "Zealous Lorecaster",
        cost: cost(&[generic(5), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giant, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Move {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Green ────────────────────────────────────────────────────────

/// Environmental Scientist — {1}{G}, 2/2 Human Druid.
/// "When this creature enters, you may search your library for a basic
/// land card, reveal it, put it into your hand, then shuffle."
pub fn environmental_scientist() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Environmental Scientist",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Pestbrood Sloth — {3}{G}, 4/4 Plant Sloth. Reach.
/// "When this creature dies, create two 1/1 black and green Pest creature
/// tokens with 'Whenever this token attacks, you gain 1 life.'"
///
/// Fully wired: the dies-trigger creates two `pest_token()`s, each of
/// which carries its "gain 1 on attack" rider (token triggered abilities
/// are materialised through `token_to_card_definition`).
pub fn pestbrood_sloth() -> CardDefinition {
    use crate::mana::g;
    use super::sorceries::pest_token;
    CardDefinition {
        name: "Pestbrood Sloth",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // No "Sloth" type yet — bridge through Plant alone.
            creature_types: vec![CreatureType::Plant],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: pest_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Witherbloom (B/G) ────────────────────────────────────────────

/// Old-Growth Educator — {2}{B}{G}, 4/4 Treefolk Druid. Vigilance, reach.
/// "Infusion — When this creature enters, put two +1/+1 counters on it
/// if you gained life this turn."
///
/// The Infusion clause is wired via the `LifeGainedThisTurnAtLeast`
/// predicate (engine added alongside this card). At ETB time the trigger
/// checks whether the controller's `life_gained_this_turn` ≥ 1 and adds
/// two +1/+1 counters when true.
pub fn old_growth_educator() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Predicate;
    use crate::mana::g;
    CardDefinition {
        name: "Old-Growth Educator",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Treefolk, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Vigilance, Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Teacher's Pest — {B}{G}, 1/1 Skeleton Pest. Menace.
/// "Whenever this creature attacks, you gain 1 life. / {B}{G}: Return
/// this card from your graveyard to the battlefield tapped."
pub fn teachers_pest() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::g;
    CardDefinition {
        name: "Teacher's Pest",
        cost: cost(&[b(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Skeleton, CreatureType::Pest],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[b(), g()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Black ────────────────────────────────────────────────────────

/// Arnyn, Deathbloom Botanist — {2}{B}, 2/2 Vampire Druid. Deathtouch.
/// "Deathtouch / Whenever a creature you control with power or
/// toughness 1 or less dies, target opponent loses 2 life and you gain
/// 2 life."
///
/// Wired with deathtouch + a `CreatureDied/AnotherOfYours`-scoped
/// trigger filtered by the dying creature's P or T being ≤ 1 via
/// `Predicate::EntityMatches { what: TriggerSource, filter: PowerAtMost
/// (1).or(ToughnessAtMost(1)) }`. The drain uses `Effect::Drain` from
/// each opponent to the controller.
pub fn arnyn_deathbloom_botanist() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Arnyn, Deathbloom Botanist",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Vampire, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CreatureDied, EventScope::AnotherOfYours)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::PowerAtMost(1)
                        .or(SelectionRequirement::ToughnessAtMost(1)),
                }),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Aziza, Mage Tower Captain — {R}{W}, 2/2 Legendary Djinn Sorcerer.
/// "Whenever you cast an instant or sorcery spell, you may tap three
/// untapped creatures you control. If you do, copy that spell. You may
/// choose new targets for the copy."
///
/// The "may tap three" optional cost is approximated as `Effect::MayDo`:
/// the cost-and-effect ordering ("if you do, copy" vs "you may copy if
/// you do") is collapsed into one decision shape.
pub fn aziza_mage_tower_captain() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::shortcut::magecraft;
    use crate::mana::r;
    CardDefinition {
        name: "Aziza, Mage Tower Captain",
        cost: cost(&[r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Tap three untapped creatures you control to copy that spell?"
                .to_string(),
            body: Box::new(Effect::Seq(vec![
                // Tap 3 untapped creatures you control (approximation: tap
                // up to 3 — if fewer than 3 available, still copy since the
                // engine has no "may pay" all-or-nothing primitive for
                // creature-tap costs).
                Effect::Tap {
                    what: Selector::take(
                        Selector::EachPermanent(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou)
                                .and(SelectionRequirement::Untapped),
                        ),
                        Value::Const(3),
                    ),
                },
                Effect::CopySpell {
                    what: Selector::TriggerSource,
                    count: Value::Const(1),
                },
            ])),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Startled Relic Sloth — {2}{R}{W}, 4/4 Sloth Beast. Trample, lifelink.
/// "Trample, lifelink / At the beginning of combat on your turn, exile up
/// to one target card from a graveyard."
///
/// Wired with both keywords on the body and the begin-combat exile
/// trigger. Same pattern as Ascendant Dustspeaker — the auto-decider
/// picks the first eligible graveyard card or no-ops the trigger when
/// graveyards are empty.
pub fn startled_relic_sloth() -> CardDefinition {
    use crate::card::SelectionRequirement;
    use crate::effect::ZoneDest;
    use crate::effect::shortcut::target_filtered;
    use crate::game::types::TurnStep;
    use crate::mana::r;
    CardDefinition {
        name: "Startled Relic Sloth",
        cost: cost(&[generic(2), r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // No "Sloth" creature subtype yet — bridge through Beast.
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::BeginCombat),
                EventScope::ActivePlayer,
            ),
            effect: Effect::Move {
                what: target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Exile,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Hardened Academic — {R}{W}, 2/1 Bird Cleric. Flying, haste.
/// "Flying, haste / Discard a card: This creature gains lifelink until
/// end of turn. / Whenever one or more cards leave your graveyard, put
/// a +1/+1 counter on target creature you control."
///
/// Now fully wired against the new `EventKind::CardLeftGraveyard` —
/// every time a card leaves Hardened Academic's controller's
/// graveyard (returned-to-hand, exiled-from-gy, reanimated, flashback
/// cast, persist/undying return), the trigger fires and lands a
/// +1/+1 counter on a target friendly creature.
///
/// Caveat: the printed Oracle says "one or more cards leave your
/// graveyard" (a single batched trigger no matter how many cards left).
/// The engine emits one `CardLeftGraveyard` event per card and so
/// fires the trigger per-card; in 2-player play this is a strict
/// power upgrade on multi-card-removal turns (Pull from the Grave
/// returns 1 card today; future multi-target cards would scale extra
/// counters).
pub fn hardened_academic() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::shortcut::target_filtered;
    use crate::mana::r;
    CardDefinition {
        name: "Hardened Academic",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Bird, CreatureType::Cleric],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Keyword::Lifelink,
                    duration: Duration::EndOfTurn,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Additional Green ────────────────────────────────────────────────────────

/// Slumbering Trudge — {X}{G}, 6/6 Plant Beast.
/// "This creature enters with a number of stun counters on it equal to
/// three minus X. If X is 2 or less, it enters tapped."
///
/// X is read at resolution time from `Value::XFromCost` via the
/// engine's new `StackItem::Trigger.x_value` plumbing. Stun-counter
/// count is computed as `max(0, 3 - X)` so X≥3 leaves the trudge
/// counter-free. The "enters tapped if X ≤ 2" half is approximated by
/// always tapping itself on ETB and letting the stun counters keep it
/// down for the right number of turns (X=3 leaves it untapped via the
/// untap step on the next turn anyway since 0 stun counters lift the
/// can't-untap effect).
pub fn slumbering_trudge() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::{ManaSymbol, g};
    let mut spell_cost = cost(&[g()]);
    spell_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Slumbering Trudge",
        cost: spell_cost,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Plant, CreatureType::Beast],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::This,
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::Stun,
                    amount: Value::NonNeg(Box::new(Value::Diff(
                        Box::new(Value::Const(3)),
                        Box::new(Value::XFromCost),
                    ))),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Tenured Concocter — {4}{G}, 4/5 Troll Druid. Vigilance.
/// "Vigilance / Whenever this creature becomes the target of a spell or
/// ability an opponent controls, you may draw a card. / Infusion — This
/// creature gets +2/+0 as long as you gained life this turn."
///
/// ✅ (was 🟡): the becomes-targeted draw trigger is now wired via the
/// new `EventKind::BecameTarget` (push: modern_decks). Spec is
/// `BecameTarget + OpponentControl`; the dispatcher checks
/// `target == source.id` implicitly and the scope refines on the
/// caster (caster must be an opponent). Effect wraps `Draw 1` in
/// `Effect::MayDo` so the printed "you may" optionality is honored —
/// AutoDecider declines (skips the draw); `ScriptedDecider` can flip
/// to "yes". The Infusion +2/+0 lives in the
/// `lifegain_selfpump_for_name` helper table (compute-time injection
/// while you gained life this turn). All three printed clauses ship.
pub fn tenured_concocter() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Tenured Concocter",
        cost: cost(&[generic(4), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Druid],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::BecameTarget, EventScope::OpponentControl),
            effect: Effect::MayDo {
                description: "Tenured Concocter: may draw a card?".into(),
                body: Box::new(Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Colorless ───────────────────────────────────────────────────────────────

/// Rancorous Archaic — {5}, 2/2 Avatar. Trample, reach.
/// "Trample, reach / Converge — This creature enters with a +1/+1
/// counter on it for each color of mana spent to cast it."
///
/// Push: Converge "enters with" now lands faithfully via the
/// `CardDefinition.enters_with_counters` field (CR 614.12) keyed off
/// `Value::ConvergedValue`. The counters land before the new
/// permanent is exposed to state-based-action sweeps, matching the
/// printed Oracle exactly. (Was an ETB trigger AddCounter; the trigger
/// fired after SBA — the 2/2 base body always survived, but the
/// timing was wrong relative to other ETB triggers / replacement
/// effects.)
pub fn rancorous_archaic() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Rancorous Archaic",
        cost: cost(&[generic(5)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::ConvergedValue)),
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

/// Pterafractyl — {X}{G}{U}, printed 1/0 Dinosaur Fractal. Flying.
/// "Flying / This creature enters with X +1/+1 counters on it. / When
/// this creature enters, you gain 2 life."
///
/// Push: Pterafractyl's printed 1/0 base now lands faithfully via the
/// new `CardDefinition.enters_with_counters` field (CR 614.12
/// replacement). The X +1/+1 counters arrive BEFORE the first SBA
/// check, so a 1/0 body with X≥1 survives ETB (1/0 + X +1/+1 = (X+1)/X
/// — printed P/T exactly). X=0 lethally dies to SBA (also matches
/// printed). The "gain 2 life" rider stays on the ETB trigger.
pub fn pterafractyl() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::{ManaSymbol, g, u};
    let mut pterafractyl_cost = cost(&[g(), u()]);
    pterafractyl_cost.symbols.insert(0, ManaSymbol::X);
    CardDefinition {
        name: "Pterafractyl",
        cost: pterafractyl_cost,
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dinosaur, CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 0,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![etb_gain_life(2)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        // CR 614.12 — "enters with X +1/+1 counters on it" reads the
        // cast's `Value::XFromCost`, applied before SBA / ETB.
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Fractal Mascot — {4}{G}{U}, 6/6 Fractal Elk. Trample.
/// "Trample / When this creature enters, tap target creature an opponent
/// controls. Put a stun counter on it."
pub fn fractal_mascot() -> CardDefinition {
    use crate::card::{CounterType, SelectionRequirement};
    use crate::effect::shortcut::target_filtered;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Fractal Mascot",
        cost: cost(&[generic(4), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Elk],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                },
                Effect::AddCounter {
                    what: Selector::Target(0),
                    kind: CounterType::Stun,
                    amount: Value::Const(1),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

/// Stadium Tidalmage — {2}{U}{R}, 4/4 Djinn Sorcerer.
/// "Whenever this creature enters or attacks, you may draw a card. If you
/// do, discard a card."
///
/// Approximation: the "you may" is collapsed to "always loot" — the
/// engine has no may-do primitive yet, so we always perform the
/// draw+discard pair. Both ETB and Attacks triggers fire the loot.
pub fn stadium_tidalmage() -> CardDefinition {
    use crate::mana::{r, u};
    let loot_body = Effect::Seq(vec![
        Effect::Draw {
            who: Selector::You,
            amount: Value::Const(1),
        },
        Effect::Discard {
            who: Selector::You,
            amount: Value::Const(1),
            random: false,
        },
    ]);
    let may_loot = Effect::MayDo {
        description: "Stadium Tidalmage: draw a card, then discard a card?".into(),
        body: Box::new(loot_body),
    };
    CardDefinition {
        name: "Stadium Tidalmage",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: may_loot.clone(),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: may_loot,
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Charging Strifeknight — {2}{R}, 3/3 Spirit Knight. Haste.
/// "{T}, Discard a card: Draw a card."
pub fn charging_strifeknight() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Charging Strifeknight",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Knight],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}


// ── Body-only batch: Increment / Opus / mana-spent rider creatures ─────────
//
// All twelve creatures below ship with their printed cost / type line / P/T /
// keywords correct, but their main ability — Increment, Opus, or a "mana
// spent to cast" pump — is omitted. Each rider needs an engine primitive
// (mana-paid introspection on cast, plus per-card "compare-spent-to-PT"
// gate) tracked in TODO.md. The vanilla bodies fill out the cube color
// pools, take combat correctly, and can be promoted to full effect once
// the engine grows the right hooks. See STRIXHAVEN2.md rows tagged
// "Standard primitives — should be straightforward to wire".

/// Cuboid Colony — {G}{U}, 1/1 Insect with Flash, Flying, and Trample.
/// "Increment (Whenever you cast a spell, if the amount of mana you
/// spent is greater than this creature's power or toughness, put a
/// +1/+1 counter on this creature.)"
///
/// Fully wired via `shortcut::increment_self_plus_one()` — uses the
/// new `Predicate::IncrementSatisfied` to gate on mana-spent > P or T.
pub fn cuboid_colony() -> CardDefinition {
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Cuboid Colony",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Insect],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![increment_self_plus_one()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Fractal Tender — {3}{G}{U}, 3/3 Elf Wizard.
/// "Ward {2}. Increment (...) / At the beginning of each end step, if
/// you put a counter on this creature this turn, create a 0/0 green
/// and blue Fractal creature token and put three +1/+1 counters on
/// it."
///
/// Increment wired via `shortcut::increment_self_plus_one()`. The
/// end-step Fractal-with-counters payoff is still omitted (no
/// per-permanent "got-a-counter-this-turn" flag yet — tracked in
/// TODO.md). 3/3 Ward {2} body remains.
/// Approximation: body + `Keyword::Ward(crate::card::WardCost::generic(2))` wired. Increment trigger
/// and end-step Fractal-with-counters payoff are both omitted (Increment
/// needs mana-spent introspection on cast; the end-step trigger
/// needs a "did this creature gain a counter this turn"
/// per-permanent flag the engine doesn't track yet). The card still
/// slots into Quandrix as a 3/3 attacker with a Ward stub, and the
/// keyword is wired so Ward enforcement picks it up.
pub fn fractal_tender() -> CardDefinition {
    use crate::card::Predicate;
    use crate::catalog::sets::sos::sorceries::fractal_token;
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::game::types::TurnStep;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Fractal Tender",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Ward(crate::card::WardCost::generic(2))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            increment_self_plus_one(),
            // Push (modern_decks, batch 82): "At the beginning of each
            // end step, if you put a counter on this creature this turn,
            // create a 0/0 G/U Fractal token and put three +1/+1
            // counters on it." Wired as a StepBegins(End)/ActivePlayer
            // trigger gated on the new
            // `Predicate::SourceGainedCounterThisTurn`. The trigger
            // body mints a Fractal via the shared `fractal_token()` and
            // piles 3 +1/+1 counters via `Selector::LastCreatedToken`.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::AnyPlayer,
                )
                .with_filter(Predicate::SourceGainedCounterThisTurn),
                effect: Effect::Seq(vec![
                    Effect::CreateToken {
                        who: PlayerRef::You,
                        count: Value::Const(1),
                        definition: fractal_token(),
                    },
                    Effect::AddCounter {
                        what: Selector::LastCreatedToken,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::Const(3),
                    },
                ]),
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Thornfist Striker — {2}{G}, 3/3 Elf Druid.
/// "Ward {1}. Infusion — Creatures you control get +1/+0 and have
/// trample as long as you gained life this turn."
///
/// Fully wired: `Ward {1}` plus the Infusion static (+1/+0 and trample to
/// your creatures while you've gained life this turn), emitted each layer
/// recompute via the `lifegain_anthem_for_name` table (gated on
/// `Player.life_gained_this_turn > 0`). 3/3 Elf Druid body.
pub fn thornfist_striker() -> CardDefinition {
    use crate::mana::g;
    CardDefinition {
        name: "Thornfist Striker",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Ward(crate::card::WardCost::generic(1))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Hungry Graffalon — {3}{G}, 3/4 Giraffe with Reach.
/// "Increment (Whenever you cast a spell, if the amount of mana you
/// spent is greater than this creature's power or toughness, put a
/// +1/+1 counter on this creature.)"
///
/// Fully wired via `shortcut::increment_self_plus_one()` — first
/// 5-mana+ spell after dropping the Giraffe lands a counter.
pub fn hungry_graffalon() -> CardDefinition {
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::mana::g;
    CardDefinition {
        name: "Hungry Graffalon",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Giraffe],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![increment_self_plus_one()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Pensive Professor — {1}{U}{U}, 0/2 Human Wizard.
/// "Increment (Whenever you cast a spell, if the amount of mana you
/// spent is greater than this creature's power or toughness, put a
/// +1/+1 counter on this creature.)"
///
/// Increment wired via `shortcut::increment_self_plus_one()`. The
/// secondary "whenever one or more +1/+1 counters are put on this
/// creature, …" rider (oracle previously truncated) stays omitted
/// pending re-fetch.
pub fn pensive_professor() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::mana::u;
    CardDefinition {
        name: "Pensive Professor",
        cost: cost(&[generic(1), u(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            increment_self_plus_one(),
            // Secondary rider: "Whenever one or more +1/+1 counters are
            // put on this creature, you may draw a card." Wired via
            // `EventKind::CounterAdded(PlusOnePlusOne)` + `SelfSource`
            // event scope so it only fires when counters land on the
            // Professor itself. Wrapped in `Effect::MayDo` to honor the
            // printed "you may" optionality.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::MayDo {
                    description: "Draw a card?".into(),
                    body: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Tester of the Tangential — {1}{U}, 1/1 Djinn Wizard.
/// "Increment (...) / At the beginning of combat on your turn, you may
/// pay {X}. When you do, move X +1/+1 counters from this creature
/// onto another target creature."
///
/// Increment wired via `shortcut::increment_self_plus_one()`. The
/// pay-X-to-move-counters combat trigger stays omitted (no X-cost
/// optional trigger primitive yet — same engine gap as Berta's
/// activation's X resolution).
pub fn tester_of_the_tangential() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::shortcut::{increment_self_plus_one, target_filtered};
    use crate::game::types::TurnStep;
    use crate::mana::u;
    CardDefinition {
        name: "Tester of the Tangential",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            increment_self_plus_one(),
            // Push (modern_decks, batch 86): "At the beginning of combat
            // on your turn, you may pay {X}. When you do, move X +1/+1
            // counters from this creature onto another target creature."
            // Approximation: X is collapsed to 1 (the engine has no
            // X-cost optional trigger primitive). The trigger fires at
            // BeginCombat/ActivePlayer, wraps a `MayPay { {1}, … }`
            // around `MoveCounter(self → target friendly creature, 1)`.
            // AutoDecider declines (Bool(false)); ScriptedDecider can
            // accept to move 1 counter at a time. For typical play
            // patterns this captures the "redistribute counter to a
            // bigger attacker" spirit even though the X-scaling is
            // omitted.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::BeginCombat),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::MayPay {
                    description: "Pay {1}: move a +1/+1 counter from Tester to another creature?"
                        .into(),
                    mana_cost: cost(&[generic(1)]),
                    body: Box::new(Effect::MoveCounter {
                        from: Selector::This,
                        to: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::OtherThanSource),
                        ),
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Muse Seeker — {1}{U}, 1/2 Elf Wizard.
/// "Opus — Whenever you cast an instant or sorcery spell, draw a
/// card. Then discard a card unless five or more mana was spent to
/// cast that spell."
///
/// Wired via `shortcut::opus_trigger`. Both branches draw first;
/// the small branch (mana_spent < 5) then forces a discard, while
/// the big branch (≥5) skips the discard. The "discard a card" half
/// has no `random` flag so the discarder picks via the existing
/// `Effect::Discard` UI flow.
pub fn muse_seeker() -> CardDefinition {
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::u;
    let draw_one = || Effect::Draw {
        who: Selector::You,
        amount: Value::Const(1),
    };
    let discard_one = || Effect::Discard {
        who: Selector::You,
        amount: Value::Const(1),
        random: false,
    };
    CardDefinition {
        name: "Muse Seeker",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // small body: draw + discard.
            Effect::Seq(vec![draw_one(), discard_one()]),
            // big body (≥5 mana): just draw (skip discard).
            draw_one(),
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Aberrant Manawurm — {3}{G}, 2/5 Wurm with Trample.
/// "Whenever you cast an instant or sorcery spell, this creature gets
/// +X/+0 until end of turn, where X is the amount of mana spent to
/// cast that spell."
///
/// Wired via `shortcut::magecraft(...)` (cast an IS spell) +
/// `Effect::PumpPT` with `power: Value::CastSpellManaSpent`. The pump
/// reads the just-cast spell's stashed `mana_spent` so casting a
/// 5-mana spell gives the wurm +5/+0 EOT.
pub fn aberrant_manawurm() -> CardDefinition {
    use crate::effect::Duration;
    use crate::effect::shortcut::magecraft;
    use crate::mana::g;
    CardDefinition {
        name: "Aberrant Manawurm",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Wurm],
            ..Default::default()
        },
        power: 2,
        toughness: 5,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: Selector::This,
            power: Value::CastSpellManaSpent,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Tackle Artist — {3}{R}, 4/3 Orc Sorcerer with Trample.
/// "Opus — Whenever you cast an instant or sorcery spell, put a +1/+1
/// counter on this creature. If five or more mana was spent to cast
/// that spell, put two +1/+1 counters on this creature instead."
///
/// Wired via `shortcut::opus_trigger` — small body is one +1/+1
/// counter on `Selector::This`, big body (≥5 mana) is two counters
/// instead.
pub fn tackle_artist() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::r;
    CardDefinition {
        name: "Tackle Artist",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // small body: +1/+1 counter
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            // big body (≥5 mana): two +1/+1 counters
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Thunderdrum Soloist — {1}{R}, 1/3 Dwarf Bard with Reach.
/// "Opus — Whenever you cast an instant or sorcery spell, this
/// creature deals 1 damage to each opponent. If five or more mana
/// was spent to cast that spell, this creature deals 3 damage to
/// each opponent instead."
///
/// Wired via `shortcut::opus_trigger` — small body pings each
/// opponent for 1, big body (≥5 mana) pings for 3.
pub fn thunderdrum_soloist() -> CardDefinition {
    use crate::effect::Selector as Sel;
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::r;
    let each_opp = || Sel::Player(PlayerRef::EachOpponent);
    CardDefinition {
        name: "Thunderdrum Soloist",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Bard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // small body: 1 damage to each opponent
            Effect::DealDamage {
                to: each_opp(),
                amount: Value::Const(1),
            },
            // big body (≥5 mana): 3 damage to each opponent instead
            Effect::DealDamage {
                to: each_opp(),
                amount: Value::Const(3),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Molten-Core Maestro — {1}{R}, 2/2 Goblin Bard with Menace.
/// "Opus — Whenever you cast an instant or sorcery spell, put a +1/+1
/// counter on this creature. If five or more mana was spent to cast
/// that spell, add an amount of {R} equal to this creature's power."
///
/// Wired via `shortcut::opus_trigger`. Small body lands one +1/+1
/// counter. Big body (≥5 mana) lands the counter and adds {R} equal
/// to the creature's current power (read live via
/// `ManaPayload::OfColor(Red, PowerOf(This))`).
pub fn molten_core_maestro() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::ManaPayload;
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::r;
    let counter = || Effect::AddCounter {
        what: Selector::This,
        kind: CounterType::PlusOnePlusOne,
        amount: Value::Const(1),
    };
    CardDefinition {
        name: "Molten-Core Maestro",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // small: +1/+1 counter only.
            counter(),
            // big (≥5 mana): counter, then add {R} × power.
            Effect::Seq(vec![
                counter(),
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::OfColor(
                        Color::Red,
                        Value::PowerOf(Box::new(Selector::This)),
                    ),
                },
            ]),
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Expressive Firedancer — {1}{R}, 2/2 Human Sorcerer.
/// "Opus — Whenever you cast an instant or sorcery spell, this
/// creature gets +1/+1 until end of turn. If five or more mana was
/// spent to cast that spell, this creature also gains double strike
/// until end of turn."
///
/// Wired via `shortcut::opus_trigger`. Small body is just +1/+1 EOT;
/// big body (≥5 mana) is +1/+1 EOT + GrantKeyword(DoubleStrike, EOT).
pub fn expressive_firedancer() -> CardDefinition {
    use crate::card::Keyword as Kw;
    use crate::effect::Duration;
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::r;
    let small_pump = || Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(1),
        toughness: Value::Const(1),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Expressive Firedancer",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // small body: +1/+1 EOT.
            small_pump(),
            // big body (≥5 mana): +1/+1 EOT + double strike EOT.
            Effect::Seq(vec![
                small_pump(),
                Effect::GrantKeyword {
                    what: Selector::This,
                    keyword: Kw::DoubleStrike,
                    duration: Duration::EndOfTurn,
                },
            ]),
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Eternal Student — {3}{B}, 4/2 Zombie Warlock.
/// "{1}{B}, Exile this card from your graveyard: Create two 1/1 white
/// and black Inkling creature tokens with flying."
///
pub fn eternal_student() -> CardDefinition {
    CardDefinition {
        name: "Eternal Student",
        cost: cost(&[generic(3), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 4,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: inkling_token(),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: true, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Postmortem Professor — {1}{B}, 2/2 Zombie Warlock. "This creature
/// can't block. / Whenever this creature attacks, each opponent loses
/// 1 life and you gain 1 life. / {1}{B}, Exile an instant or sorcery
/// card from your graveyard: Return this card from your graveyard to
/// the battlefield."
///
/// Fully wired: the printed `Keyword::CantBlock` static + on-attack
/// drain + graveyard-recursion activation. The activation uses the new
/// `ActivatedAbility.exile_other_filter` cost primitive (paired with
/// `from_graveyard: true`): pay `{1}{B}` and exile an instant/sorcery
/// card from your graveyard (other than the Professor itself), then
/// return the Professor from graveyard to battlefield.
pub fn postmortem_professor() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Postmortem Professor",
        cost: cost(&[generic(1), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Zombie, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::CantBlock],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), b()]),
            effect: Effect::Move {
                what: Selector::This,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: false,
                },
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: false,
            exile_other_filter: Some((
                SelectionRequirement::HasCardType(CardType::Instant)
                    .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                1,
            )),
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::Drain {
                from: Selector::Player(PlayerRef::EachOpponent),
                to: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Spirit Mascot — {R}{W}, 2/2 Spirit Ox.
/// "Whenever one or more cards leave your graveyard, put a +1/+1 counter
/// on this creature."
///
/// Wired against the new `EventKind::CardLeftGraveyard` — every card
/// removal from the controller's graveyard puts a +1/+1 counter on the
/// Mascot. Per-card emission means a multi-card return (future
/// Borrowed-Knowledge-style) drops more counters than the printed
/// "one or more" wording promises; this is a strict upgrade and stays
/// aligned with the typical 1-card-per-effect Strixhaven game flow.
pub fn spirit_mascot() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::r;
    CardDefinition {
        name: "Spirit Mascot",
        cost: cost(&[r(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Spirit, CreatureType::Ox],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Witherbloom, the Balancer — {6}{B}{G} 5/5 Legendary Elder Dragon.
/// "Affinity for creatures (This spell costs {1} less to cast for each
/// creature you control.) / Flying, deathtouch / Instant and sorcery
/// spells you cast have affinity for creatures."
///
/// Body wired faithfully (Flying, Deathtouch, 5/5 Legendary Elder
/// Dragon). The two "affinity for creatures" cost-reduction clauses
/// are omitted — the engine has no per-cast cost reduction whose
/// discount scales off the caster's permanent count. Tracked in
/// TODO.md under "Affinity / Self-Permanent-Scaled Cost Reduction".
///
/// Even at the printed {6}{B}{G} the dragon is a high-impact finisher
/// in Witherbloom's late game and slots into the school's deathtouch
/// + lifegain themes (Bogwater Lumaret's friendly-ETB lifegain, Pest
///   Mascot's lifegain → +1/+1 counters, etc.).
/// and lifegain themes (Bogwater Lumaret's friendly-ETB lifegain, Pest
/// Mascot's lifegain into +1/+1 counters, etc.).
pub fn witherbloom_the_balancer() -> CardDefinition {
    use crate::card::{SelectionRequirement, Supertype};
    use crate::effect::{StaticAbility, StaticEffect};
    use crate::mana::g;
    CardDefinition {
        name: "Witherbloom, the Balancer",
        cost: cost(&[generic(6), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Deathtouch],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        // "Instant and sorcery spells you cast have affinity for creatures."
        // Wired via `StaticEffect::GrantAffinityToISSpells` (batch 25): at
        // cast time, every IS spell the controller casts gets {1} less per
        // battlefield permanent matching `permanent_filter`. Restricts to
        // creatures you control via `ControlledByYou`.
        static_abilities: vec![StaticAbility {
            description: "Instant and sorcery spells you cast have affinity for creatures.",
            effect: StaticEffect::GrantAffinityToISSpells {
                permanent_filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou),
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        // "Affinity for creatures (This spell costs {1} less to cast for each
        // creature you control.)" — counts only creatures you control, per
        // CR 702.40b. The card-intrinsic `affinity_filter` covers Witherbloom
        // the Balancer's *own* self-cast discount; the static above covers
        // every other IS spell the controller casts.
        affinity_filter: Some(
            SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
        ),
        equipped_bonus: None,
    }
}

/// Quandrix, the Proof — {4}{G}{U} 6/6 Legendary Elder Dragon.
/// "Flying, trample / Cascade (When you cast this spell, exile cards
/// from the top of your library until you exile a nonland card that
/// costs less. You may cast it without paying its mana cost. Put the
/// exiled cards on the bottom in a random order.) / Instant and
/// sorcery spells you cast from your hand have cascade."
///
/// 🟡: 6/6 Flying/Trample with its own Cascade wired (SpellCast trigger →
/// RevealUntilFind a cheaper nonland → may-play). The "IS spells you cast
/// have cascade" granting static is still omitted.
pub fn quandrix_the_proof() -> CardDefinition {
    use crate::card::{MayPlayDuration, Supertype};
    use crate::effect::{RevealMissDest, ZoneDest};
    use crate::mana::{g, u};
    CardDefinition {
        name: "Quandrix, the Proof",
        cost: cost(&[generic(4), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying, Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // Push (modern_decks, batch 79): Cascade. "When you cast this
        // spell, exile cards from the top of your library until you
        // exile a nonland card with mana value less than this spell's
        // mana value. You may cast that card without paying its mana
        // cost. Put the rest on the bottom of your library in a random
        // order." Wired as SpellCast/SelfSource trigger →
        // RevealUntilFind { Nonland ∧ MV ≤ 5 (printed CMC 6 − 1), to:
        // Exile, miss_dest: BottomRandom } + GrantMayPlay {LastMoved,
        // EndOfThisTurn}. Same primitive chain as Velomachus Lorehold.
        // Approximation: the "less than the spell's mana value" cap is
        // hard-coded to ManaValueAtMost(5) (Quandrix the Proof's
        // printed CMC is 6, so "less than 6" = "≤ 5"). The cap doesn't
        // shift if X-cost extensions enter the picture (Quandrix the
        // Proof has no X in its cost, so this is exact for the printed
        // card).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::RevealUntilFind {
                    who: PlayerRef::You,
                    find: SelectionRequirement::Nonland
                        .and(SelectionRequirement::ManaValueAtMost(5)),
                    to: ZoneDest::Exile,
                    cap: Value::Const(60),
                    life_per_revealed: 0,
                    miss_dest: RevealMissDest::BottomRandom,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Garrison Excavator — {3}{R}, 3/4 Orc Sorcerer with menace.
/// "Menace / Whenever one or more cards leave your graveyard, create a
/// 2/2 red and white Spirit creature token."
///
/// Wired against the new `EventKind::CardLeftGraveyard`. Each card
/// removal from the controller's graveyard mints a 2/2 R/W Spirit
/// token (shared `spirit_token()` from the sorceries module).
pub fn garrison_excavator() -> CardDefinition {
    use super::sorceries::spirit_token;
    use crate::mana::r;
    CardDefinition {
        name: "Garrison Excavator",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::CardLeftGraveyard, EventScope::YourControl),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: spirit_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Topiary Lecturer — {2}{G}, 1/2 Elf Druid.
/// "Increment (Whenever you cast a spell, if the amount of mana you spent
/// is greater than this creature's power or toughness, put a +1/+1
/// counter on this creature.) / {T}: Add an amount of {G} equal to this
/// creature's power."
///
/// Both abilities now wired. Increment uses
/// `shortcut::increment_self_plus_one()`. The mana ability uses
/// `ManaPayload::OfColor(Green, PowerOf(This))` — fixed color,
/// value-scaled count — so a single AddMana effect produces power-many
/// {G} pips. Each Increment counter scales the mana ability too, since
/// `PowerOf(This)` reads live P at activation time.
pub fn topiary_lecturer() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::mana::g;
    CardDefinition {
        name: "Topiary Lecturer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Green,
                    Value::PowerOf(Box::new(Selector::This)),
                ),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![increment_self_plus_one()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Transcendent Archaic — {7}, 6/6 Avatar with vigilance.
/// "Vigilance / Converge — When this creature enters, you may draw X
/// cards, where X is the number of colors of mana spent to cast this
/// spell. If you draw one or more cards this way, discard two cards."
///
/// `Effect::Draw` with `Value::ConvergedValue` + a follow-up
/// conditional `Discard 2` gated on `ConvergedValue ≥ 1`. The "you
/// may" optionality is collapsed to always-draw-when-X-≥-1 (no may-do
/// primitive yet); at X=0 the draw and discard both no-op. ConvergedValue
/// rides on the `StackItem::Trigger.converged_value` plumbing already in
/// place for Rancorous Archaic.
pub fn transcendent_archaic() -> CardDefinition {
    use crate::effect::Predicate;
    CardDefinition {
        name: "Transcendent Archaic",
        cost: cost(&[generic(7)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // Push (modern_decks): printed "you may" optionality now honored via
            // `Effect::MayDo`. AutoDecider answers "no" by default (skipping
            // the whole draw + discard chain); ScriptedDecider can flip to
            // "yes" via `DecisionAnswer::Bool(true)`. The conditional discard
            // 2 still rides on the same `If(ConvergedValue ≥ 1)` gate so the
            // printed "if you draw one or more cards this way, discard two
            // cards" tail is preserved.
            effect: Effect::MayDo {
                description: "Draw X cards (where X is the number of colors of mana spent) \
                              and then discard two cards if you drew at least one."
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::ConvergedValue,
                    },
                    Effect::If {
                        cond: Predicate::ValueAtLeast(Value::ConvergedValue, Value::Const(1)),
                        then: Box::new(Effect::Discard {
                            who: Selector::You,
                            amount: Value::Const(2),
                            random: false,
                        }),
                        else_: Box::new(Effect::Noop),
                    },
                ])),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Sundering Archaic — {6}, 3/3 Avatar.
/// "Converge — When this creature enters, exile target nonland permanent
/// an opponent controls with mana value less than or equal to the number
/// of colors of mana spent to cast this creature. / {2}: Put target card
/// from a graveyard on the bottom of its owner's library."
///
/// Push (modern_decks): the converge-scaled mana-value cap is **now
/// wired** via `Effect::If { cond: ValueAtMost(ManaValueOf(Target(0)),
/// ConvergedValue), then: Exile(Target(0)), else_: Noop }`. The trigger
/// no-ops cleanly when the target's MV exceeds ConvergedValue (e.g.
/// mono-colorless cast → ConvergedValue = 0 → only MV-0 permanents
/// are legitimate exile targets; at 1 color → MV ≤ 1; at 5 colors →
/// MV ≤ 5). Auto-target picks any legal opp permanent first; the
/// resolve-time gate then enforces the cap.
///
/// The `{2}: graveyard → bottom of owner's library` activated ability
/// is unchanged.
pub fn sundering_archaic() -> CardDefinition {
    use crate::card::Predicate;
    use crate::effect::{LibraryPosition, ZoneDest};
    CardDefinition {
        name: "Sundering Archaic",
        cost: cost(&[generic(6)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::new(vec![generic(2)]),
            effect: Effect::Move {
                what: crate::effect::shortcut::target_filtered(SelectionRequirement::Any),
                to: ZoneDest::Library {
                    who: PlayerRef::OwnerOf(Box::new(Selector::Target(0))),
                    pos: LibraryPosition::Bottom,
                },
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::ValueAtMost(
                    Value::ManaValueOf(Box::new(Selector::Target(0))),
                    Value::ConvergedValue,
                ),
                then: Box::new(Effect::Exile {
                    what: crate::effect::shortcut::target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Poisoner's Apprentice — {2}{B}, 2/2 Orc Warlock.
/// "Infusion — When this creature enters, target creature an opponent
/// controls gets -4/-4 until end of turn if you gained life this turn."
///
/// Wired with the `Predicate::LifeGainedThisTurnAtLeast` Infusion gate
/// — same as Foolish Fate / Old-Growth Educator / Efflorescence. The
/// trigger fires on ETB; the body is a conditional pump-by-(-4/-4) on a
/// target opponent creature. If you haven't gained life this turn the
/// trigger resolves into a Noop so the cast still goes off.
pub fn poisoners_apprentice() -> CardDefinition {
    use crate::effect::Predicate;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Poisoner's Apprentice",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::PumpPT {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    power: Value::Const(-4),
                    toughness: Value::Const(-4),
                    duration: Duration::EndOfTurn,
                }),
                else_: Box::new(Effect::Noop),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Hydro-Channeler — {1}{U}, 1/3 Merfolk Wizard.
/// "{T}: Add {U}. Spend this mana only to cast an instant or sorcery
/// spell. / {1}, {T}: Add one mana of any color. Spend this mana only to
/// cast an instant or sorcery spell."
///
/// Approximation: the "spend only to cast an instant or sorcery"
/// mana-spend restriction is omitted (engine has no spend-restricted mana
/// primitive). Both abilities are wired as plain mana adders — `{T}: Add
/// {U}` and `{1},{T}: Add one mana of any color`. This is over-flexible
/// (the produced mana can be spent on creatures), but the typical play
/// pattern (cast IS spells) is unaffected.
pub fn hydro_channeler() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::mana::u;
    CardDefinition {
        name: "Hydro-Channeler",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: ManaCost::default(),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Blue]),
                },
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
            },
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(1)]),
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
            tap_other_filter: None,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Ulna Alley Shopkeep — {2}{B}, 2/3 Goblin Warlock.
/// "Menace (This creature can't be blocked except by two or more
/// creatures.) / Infusion — This creature gets +2/+0 as long as you
/// gained life this turn."
///
/// ✅ Menace keyworded on the body; the conditional Infusion `+2/+0`
/// rider is wired via a compute-time injection in
/// `GameState::compute_battlefield` (same pattern as Honor Troll):
/// when `Player.life_gained_this_turn > 0`, layer 7b adds
/// `ModifyPowerToughness(+2, +0)` targeting the source. The gate
/// re-evaluates every recompute, so a mid-turn lifegain flips it on
/// (Shopkeep goes 2/3 → 4/3) and `do_untap` resets the tally at the
/// next untap step, dropping back to 2/3.
pub fn ulna_alley_shopkeep() -> CardDefinition {
    CardDefinition {
        name: "Ulna Alley Shopkeep",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Warlock],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Menace],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Emil, Vastlands Roamer — {2}{G}, 3/3 Legendary Elf Druid.
/// "Creatures you control with +1/+1 counters on them have trample.
/// {4}{G}, {T}: Create a 0/0 green and blue Fractal creature token.
/// Put X +1/+1 counters on it, where X is the number of differently
/// named lands you control."
///
/// Approximation: the "differently named" filter on the activated
/// ability's X value is collapsed to **all** lands you control — the
/// engine has no `Value::DistinctNamesIn(...)` primitive yet, and in
/// the typical cube game each land slot is unique anyway, so the
/// behavior matches in practice. The static "trample for creatures
/// with +1/+1 counters" is wired faithfully via `StaticEffect::
/// GrantKeyword` filtered to `WithCounter(PlusOnePlusOne)`.
pub fn emil_vastlands_roamer() -> CardDefinition {
    use crate::card::{CounterType, StaticAbility, StaticEffect, Supertype};
    use crate::catalog::sets::sos::sorceries::fractal_token;
    use crate::mana::g;
    CardDefinition {
        name: "Emil, Vastlands Roamer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(4), g()]),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountOf(Box::new(Selector::EachPermanent(
                        SelectionRequirement::Land
                            .and(SelectionRequirement::ControlledByYou),
                    ))),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Creatures you control with +1/+1 counters on them have trample.",
            effect: StaticEffect::GrantKeyword {
                applies_to: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::WithCounter(
                            CounterType::PlusOnePlusOne,
                        )),
                ),
                keyword: Keyword::Trample,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

/// Abstract Paintmage — {U}{U/R}{R}, 2/2 Djinn Sorcerer.
/// "At the beginning of your first main phase, add {U}{R}. Spend this
/// mana only to cast instant and sorcery spells."
///
/// Approximation: the spend restriction ("only to cast instant and
/// sorcery spells") is omitted — the engine's `ManaPool` has no per-pip
/// spend metadata yet (tracked as **Spend-Restricted Mana** in TODO.md),
/// so the produced {U}{R} behaves like normal mana and can fund any
/// spell. The trigger fires on the active player's PreCombatMain step
/// (the controller's "first" main phase). The `{U/R}` pip is a real
/// `ManaSymbol::Hybrid(Blue, Red)`, payable with either blue or red.
pub fn abstract_paintmage() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::game::types::TurnStep;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Abstract Paintmage",
        cost: cost(&[u(), crate::mana::hybrid(Color::Blue, Color::Red), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::ActivePlayer,
            ),
            effect: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Blue, Color::Red]),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Geometer's Arthropod — {G}{U}, 1/4 Fractal Crab.
/// "Whenever you cast a spell with {X} in its mana cost, look at the top
/// X cards of your library. Put one of them into your hand and the rest
/// on the bottom of your library in a random order."
///
/// Now wired (push XVI): the `SpellCast` filter uses the new
/// `Predicate::CastSpellHasX` primitive; the body approximates the
/// "look X, pick 1, rest to bottom" shape with `RevealUntilFind { find:
/// Any, cap: XFromCost, to: Hand }`. The trigger inherits the cast
/// spell's X via `StackItem::Trigger.x_value`, so the cap reflects the
/// real X paid. Misses go to graveyard (engine default for
/// `RevealUntilFind`); the printed "rest to bottom random" rider is an
/// approximation since the engine has no random-bottom primitive yet.
pub fn geometers_arthropod() -> CardDefinition {
    use crate::effect::shortcut::cast_has_x_trigger;
    use crate::effect::ZoneDest;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Geometer's Arthropod",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Crab],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![cast_has_x_trigger(Effect::RevealUntilFind {
            who: PlayerRef::You,
            find: SelectionRequirement::Any,
            to: ZoneDest::Hand(PlayerRef::You),
            cap: Value::XFromCost,
            life_per_revealed: 0,
            // Printed: "Put one of them into your hand and the rest on
            // the bottom of your library in a random order."
            miss_dest: crate::effect::RevealMissDest::BottomRandom,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── More Blue ───────────────────────────────────────────────────────────────

/// Matterbending Mage — {2}{U}, 2/2 Human Wizard.
/// "When this creature enters, return up to one other target creature
/// to its owner's hand. / Whenever you cast a spell with {X} in its
/// mana cost, this creature can't be blocked this turn."
///
/// Now wired (push XVI): both abilities. ETB bounce wired faithfully
/// (target a creature other than this one; the auto-target picker
/// prefers another creature when one exists). The X-cost spell-cast
/// trigger uses the new `Predicate::CastSpellHasX` primitive + grants
/// `Keyword::Unblockable` (EOT) to the Mage itself via
/// `Selector::This`.
pub fn matterbending_mage() -> CardDefinition {
    use crate::effect::shortcut::{cast_has_x_trigger, target_filtered};
    use crate::effect::ZoneDest;
    use crate::mana::u;
    CardDefinition {
        name: "Matterbending Mage",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Move {
                    what: target_filtered(SelectionRequirement::Creature),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
            },
            cast_has_x_trigger(Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::Unblockable,
                duration: Duration::EndOfTurn,
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Orysa, Tide Choreographer — {4}{U}, 2/2 Legendary Merfolk Bard.
/// "This spell costs {3} less to cast if creatures you control have
/// total toughness 10 or greater. / When Orysa enters, draw two cards."
///
/// Approximation: the conditional "{3} less if total toughness ≥ 10"
/// alternative cost is omitted — the engine has no
/// "alt-cost-with-board-state-predicate" primitive (tracked in TODO.md
/// alongside Mavinda, Killian, and Ajani's Response). The ETB draw 2
/// is wired faithfully and the printed full cost is paid
/// unconditionally.
pub fn orysa_tide_choreographer() -> CardDefinition {
    use crate::card::{AlternativeCost, Supertype};
    use crate::effect::Predicate;
    use crate::mana::u;
    CardDefinition {
        name: "Orysa, Tide Choreographer",
        cost: cost(&[generic(4), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Bard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        // Push (modern_decks): "{3} less if creatures you control have
        // total toughness 10 or greater" alt-cost rider wired via
        // `AlternativeCost.condition`. The gate evaluates
        // `ValueAtLeast(Sum of friendly creature toughness, 10)` against
        // the cast-time context. Alt cost is {1}{U} ({3} less than the
        // printed {4}{U}). Lets the printed Oracle cast Orysa as an
        // ETB-draw-2 finisher off a wide go-wide board.
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(1), u()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: Some(Predicate::ValueAtLeast(
                Value::ToughnessOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
                Value::Const(10),
            )),
                    exile_from_graveyard_count: 0,
            effect_override: None,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Exhibition Tidecaller — {U}, 0/2 Djinn Wizard.
/// "Opus — Whenever you cast an instant or sorcery spell, target
/// player mills three cards. If five or more mana was spent to cast
/// that spell, that player mills ten cards instead."
///
/// Wired via `shortcut::opus_trigger`. Auto-target picks an opponent
/// for the mill via `Selector::Player(PlayerRef::EachOpponent)` — the
/// engine's auto-target walker picks the first opponent. Small body
/// mills 3, big body (≥5 mana) mills 10.
pub fn exhibition_tidecaller() -> CardDefinition {
    use crate::effect::Selector as Sel;
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::u;
    // Use `PlayerRef::Target(0)` so the auto-targeter picks a player
    // slot at trigger time; falls back to an opponent.
    let target_player = || Sel::Player(PlayerRef::Target(0));
    CardDefinition {
        name: "Exhibition Tidecaller",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Djinn, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            Effect::Mill {
                who: target_player(),
                amount: Value::Const(3),
            },
            Effect::Mill {
                who: target_player(),
                amount: Value::Const(10),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Practiced Scrollsmith — {R}{R/W}{W}, 3/2 Dwarf Cleric.
/// "First strike / When this creature enters, exile target noncreature,
/// nonland card from your graveyard. Until the end of your next turn,
/// you may cast that card."
///
/// Push (modern_decks): the "until end of your next turn, you may cast"
/// rider is **now wired** via the new `Effect::GrantMayPlay` primitive.
/// ETB body is `Seq([Move(target → Exile), GrantMayPlay(...,
/// EndOfControllersNextTurn)])`. The same `Selector::Take(_, 1)` selects
/// the lifted card so the permission targets exactly the card that just
/// went to exile. The controller then invokes
/// `GameAction::CastFromZoneWithoutPaying` during a later sorcery-speed
/// window to recur the card for free. The `{R/W}` pip is a real
/// `ManaSymbol::Hybrid(Red, White)`, payable with either red or white.
pub fn practiced_scrollsmith() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{r, w as wm};
    let nonperm_in_gy = SelectionRequirement::Nonland
        .and(SelectionRequirement::Not(Box::new(SelectionRequirement::Creature)));
    let target_card = Selector::take(
        Selector::CardsInZone {
            who: PlayerRef::You,
            zone: crate::card::Zone::Graveyard,
            filter: nonperm_in_gy,
        },
        Value::Const(1),
    );
    CardDefinition {
        name: "Practiced Scrollsmith",
        cost: cost(&[r(), crate::mana::hybrid(Color::Red, Color::White), wm()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::FirstStrike],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_card,
                    to: ZoneDest::Exile,
                },
                // Read the card we just moved via the resolution-scoped
                // `Selector::LastMoved` scratch — the second step in the
                // Seq targets exactly the card the first step lifted.
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                    to_owner: false,
                    exile_after: false,
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── More Lorehold (R/W) ─────────────────────────────────────────────────────

/// Colossus of the Blood Age — {4}{R}{W}, 6/6 Artifact Creature —
/// Construct. "When this creature enters, it deals 3 damage to each
/// opponent and you gain 3 life. / When this creature dies, discard
/// any number of cards, then draw that many cards plus one."
///
/// Push (modern_decks): death trigger now uses the new
/// `Effect::DiscardAnyNumber` primitive — the player chooses how many
/// cards to discard (0 to hand size), and the follow-up `Draw` reads
/// `Value::CardsDiscardedThisEffect + 1` so the draw count matches the
/// actual discard count plus one. AutoDecider picks 0 (conservative
/// default — discard nothing, still draw 1). ScriptedDecider can supply
/// a `DecisionAnswer::Discard(picked_ids)` to opt into discarding any
/// subset of the hand. Tests: `colossus_of_the_blood_age_death_trigger_*`.
pub fn colossus_of_the_blood_age() -> CardDefinition {
    use crate::mana::{r, w as wm};
    CardDefinition {
        name: "Colossus of the Blood Age",
        cost: cost(&[generic(4), r(), wm()]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::DealDamage {
                        to: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(3),
                    },
                    Effect::GainLife {
                        who: Selector::You,
                        amount: Value::Const(3),
                    },
                ]),
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::DiscardAnyNumber { who: Selector::You },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Sum(vec![
                            Value::CardsDiscardedThisEffect,
                            Value::Const(1),
                        ]),
                    },
                ]),
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── More White ──────────────────────────────────────────────────────────────

/// Soaring Stoneglider — {2}{W}, 4/3 Elephant Cleric.
/// "As an additional cost to cast this spell, exile two cards from your
/// graveyard or pay {1}{W}. / Flying, vigilance"
///
/// Push (modern_decks): the alt additional cost (exile two cards from
/// your graveyard) is **now wired** via the new
/// `AlternativeCost.exile_from_graveyard_count: u32` field. The
/// printed cost is modeled as **{3}{W}** (base {2}{W} + the {1}{W}
/// mana fork — auto-decider's default path). The alt cost path
/// {2}{W} with `exile_from_graveyard_count: 2` is available when the
/// caster's graveyard has at least 2 cards. Auto-picker takes the
/// lowest-CMC cards. Body (4/3 Flying + Vigilance) unchanged.
pub fn soaring_stoneglider() -> CardDefinition {
    use crate::card::AlternativeCost;
    use crate::mana::w;
    CardDefinition {
        name: "Soaring Stoneglider",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elephant, CreatureType::Cleric],
            ..Default::default()
        },
        power: 4,
        toughness: 3,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: cost(&[generic(2), w()]),
            life_cost: 0,
            exile_filter: None,
            evoke_sacrifice: false,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
            exile_from_graveyard_count: 2,
            effect_override: None,
        }),
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}


// ── 2026-05-01 push VII: Multicolored predicate, MDFC bodies, Lorehold capstone

/// Spectacular Skywhale — {2}{U}{R} Creature — Elemental Whale.
/// 1/4 Flying. Opus rider omitted.
///
/// Body wired in `catalog::sets::sos::creatures` as a 1/4 flying U/R
/// Elemental Whale. The "Opus — Whenever you cast an instant or sorcery
/// spell, this creature gets +3/+0 EOT (or 3 +1/+1 counters at 5+ mana
/// spent)" rider is omitted (mana-spent introspection on cast — same gap
/// as Aberrant Manawurm, Tackle Artist, Expressive Firedancer).
pub fn spectacular_skywhale() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Duration;
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Spectacular Skywhale",
        cost: cost(&[generic(2), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Whale],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        // ✅ Opus fully wired via `shortcut::opus_trigger`. Small body
        // (<5 mana) is +3/+0 EOT on the Skywhale. Big body (≥5 mana)
        // replaces the pump with three +1/+1 counters — the printed
        // Oracle says "instead", so the big branch swaps in rather than
        // stacking, matching the `If` semantics inside `opus_trigger`.
        triggered_abilities: vec![opus_trigger(
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Colorstorm Stallion — {1}{U}{R}, 3/3 Elemental Horse.
/// Ward {1}, haste. "Opus — Whenever you cast an instant or sorcery spell,
/// this creature gets +1/+1 until end of turn. If five or more mana was
/// spent to cast that spell, create a token that's a copy of this creature."
///
/// Body wired with Ward {1}, haste, plus the full Opus payoff: the
/// magecraft +1/+1 self-pump, and — when five or more mana was spent to
/// cast the instant/sorcery — a token that's a copy of this creature
/// (via `Effect::CreateTokenCopyOf { source: This }`). The copy-token
/// rider was previously omitted for lack of a copy-permanent primitive;
/// it now uses the engine's `CreateTokenCopyOf`.
pub fn colorstorm_stallion() -> CardDefinition {
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Colorstorm Stallion",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Horse],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Ward(crate::card::WardCost::generic(1)), Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // Small body: +1/+1 until end of turn.
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
            // Big body (≥5 mana spent): pump, then mint a copy token.
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::CreateTokenCopyOf {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    source: Selector::This,
                    extra_creature_types: vec![],
                    override_pt: None,
                },
            ]),
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Elemental Mascot — {1}{U}{R}, 1/4 Elemental Bird.
/// Flying, vigilance. "Opus — Whenever you cast an instant or sorcery spell,
/// this creature gets +1/+0 until end of turn. If five or more mana was spent
/// to cast that spell, exile the top card of your library. You may play that
/// card until the end of your next turn."
///
/// Body wired with Flying, Vigilance, plus the full Opus payoff: the
/// magecraft +1/+0 self-pump, and — when five or more mana was spent to
/// cast the instant/sorcery — exile the top card of the library and grant
/// "you may play that card until the end of your next turn" via
/// `Effect::ExileTopAndGrantMayPlay`. The cast-from-exile rider was
/// previously omitted; it now uses the engine's existing primitive.
pub fn elemental_mascot() -> CardDefinition {
    use crate::effect::shortcut::opus_trigger;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Elemental Mascot",
        cost: cost(&[generic(1), u(), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Bird],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![opus_trigger(
            // Small body: +1/+0 until end of turn.
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
            // Big body (≥5 mana spent): pump, then exile-top + may-play.
            Effect::Seq(vec![
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
                },
                Effect::ExileTopAndGrantMayPlay {
                    who: PlayerRef::You,
                    duration: crate::card::MayPlayDuration::EndOfControllersNextTurn,
                },
            ]),
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        ..Default::default()
    }
}

/// Zaffai and the Tempests — {5}{U}{R}, 5/7 Legendary Human Bard Sorcerer.
/// "Once during each of your turns, you may cast an instant or sorcery
/// spell from your hand without paying its mana cost."
///
/// 5/7 Legendary Human Bard Sorcerer. "Once during each of your turns,
/// you may cast an instant or sorcery spell from your hand without paying
/// its mana cost" wired as a PreCombatMain StepBegins trigger granting a
/// one-shot free-cast (`GrantMayPlay`) on an auto-picked IS card in hand.
pub fn zaffai_and_the_tempests() -> CardDefinition {
    use crate::card::{MayPlayDuration, Supertype, Zone};
    use crate::game::types::TurnStep;
    use crate::mana::{r, u};
    // Push (modern_decks, batch 100): "Once during each of your turns,
    // you may cast an instant or sorcery spell from your hand without
    // paying its mana cost." Wired as a `StepBegins(PreCombatMain)/
    // ActivePlayer` trigger on Zaffai that grants `MayPlay {
    // EndOfThisTurn, exile_after: false }` on one IS card in hand
    // (auto-picked by the engine — typically the highest-CMC IS card,
    // matching the printed "save mana on a big spell" play pattern).
    // Approximation: the engine picks the card upfront rather than
    // letting the controller choose at cast time; the controller gets
    // exactly one free IS cast per turn from the picked card, which
    // expires at EOT cleanup. Same approximation strategy as Flashback
    // (the spell) and Lorehold the Historian's miracle grant.
    CardDefinition {
        name: "Zaffai and the Tempests",
        cost: cost(&[generic(5), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Human,
                CreatureType::Bard,
                CreatureType::Sorcerer,
            ],
            ..Default::default()
        },
        power: 5,
        toughness: 7,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::ActivePlayer,
            ),
            effect: Effect::GrantMayPlay {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: Zone::Hand,
                        filter: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    },
                    Value::Const(1),
                ),
                duration: MayPlayDuration::EndOfThisTurn,
                to_owner: false,
                exile_after: false,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Lorehold, the Historian — {3}{R}{W} Legendary Creature — Elder Dragon.
/// 5/5 Flying, haste.
///
/// Push (modern_decks): the per-opp-upkeep loot trigger **is now wired**
/// via `EventSpec::new(StepBegins(Upkeep), EventScope::OpponentControl)`
/// — the engine's step-trigger dispatcher fires `OpponentControl`-scoped
/// triggers whose source's controller is NOT the active player, which
/// matches the printed "at the beginning of each opponent's upkeep"
/// wording for non-active sources. Body is `MayDo(Seq(Discard 1, Draw
/// 1))` so the controller opts into the loot.
///
/// The "instant and sorcery cards in your hand have miracle {2}" static
/// is still ⏳ (no Miracle keyword / alt-cost-on-draw primitive). The
/// vanilla 5/5 Flying+Haste body is the headline play pattern; the
/// per-opp-turn loot adds free card velocity.
pub fn lorehold_the_historian() -> CardDefinition {
    use crate::card::{MayPlayDuration, Predicate, Supertype};
    use crate::game::types::TurnStep;
    use crate::mana::{r, w};
    // Push (modern_decks, batch 93): the "Each instant and sorcery card
    // in your hand has miracle {2}" grant is wired as a CardDrawn/
    // YourControl trigger gated on (a) drawn card is IS and (b) it's
    // the first card you drew this turn. Body grants `MayPlay {
    // EndOfThisTurn, exile_after: false }` on the drawn card. The
    // engine has no `Miracle {N}` alt-cost primitive (no separate
    // miracle-cost path) so the {2} miracle-cost is approximated as
    // *free* — overpowered relative to printed but functional. Future
    // work: add a `MayPlayPermission.alt_cost: Option<ManaCost>` field
    // so the may-cast path can require a non-zero payment. Engine
    // tweak in this batch: `event_subject` for CardDrawn now uses
    // `card_id` (not player) so `Selector::TriggerSource` resolves to
    // the drawn card.
    let miracle_grant = TriggeredAbility {
        event: EventSpec::new(EventKind::CardDrawn, EventScope::YourControl).with_filter(
            Predicate::All(vec![
                Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                },
                Predicate::ValueEquals(
                    Value::CardsDrawnThisTurn(PlayerRef::You),
                    Value::Const(1),
                ),
            ]),
        ),
        effect: Effect::GrantMayPlay {
            what: Selector::TriggerSource,
            duration: MayPlayDuration::EndOfThisTurn,
            to_owner: false,
            exile_after: false,
        },
    };
    CardDefinition {
        name: "Lorehold, the Historian",
        cost: cost(&[generic(3), r(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying, Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![miracle_grant, TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::MayDo {
                description: "Lorehold, the Historian: discard a card to draw a card?"
                    .into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ])),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Mage Tower Referee — {2} Artifact Creature — Construct.
/// 2/1. "Whenever you cast a multicolored spell, put a +1/+1 counter on
/// this creature."
///
/// Wired against the new `SelectionRequirement::Multicolored` predicate
/// — the magecraft-style cast trigger filters its `EntityMatches` clause
/// on `TriggerSource` having ≥ 2 distinct colored pips. Hybrid pips (e.g.
/// {W/B}) count both halves; Phyrexian counts the colored side. So a
/// Lorehold Charm ({R}{W}) or Silverquill Charm ({W}{B}) bumps the
/// Referee, but a colorless artifact (Sol Ring) or mono-color spell
/// (Lightning Bolt) does not.
pub fn mage_tower_referee() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Predicate;
    CardDefinition {
        name: "Mage Tower Referee",
        cost: cost(&[generic(2)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::EntityMatches {
                    what: Selector::TriggerSource,
                    filter: SelectionRequirement::Multicolored,
                }),
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
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Ennis, Debate Moderator — {1}{W} 1/1 Legendary Human Cleric.
/// "When Ennis enters, exile up to one other target creature you control.
/// Return that card to the battlefield under its owner's control at the
/// beginning of the next end step. / At the beginning of your end step,
/// if one or more cards were put into exile this turn, put a +1/+1
/// counter on Ennis."
///
/// Both abilities partially wired:
/// - ETB flicker: exiles a target creature (auto-picker prefers a
///   friendly utility creature with a useful ETB) and schedules a
///   delayed return at next end step. Uses the same
///   `Exile + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf)))`
///   pattern as Restoration Angel-style flickers.
/// - End-step counter: gated on "any card was exiled this turn". The
///   engine doesn't yet track per-turn exile count as a `Value`, so we
///   approximate this by using `CardsLeftGraveyardThisTurnAtLeast` as a
///   proxy (most sources of "card put into exile" pass through gy first
///   in our engine — flicker exiles, exile-from-gy effects, etc.). The
///   approximation under-counts pure hand-exile and bounce-to-exile
///   effects but covers the common case (Ennis's own ETB exile triggers
///   the predicate via the gy-leave fired by the delayed return).
pub fn ennis_debate_moderator() -> CardDefinition {
    use crate::card::{CounterType, Predicate, Supertype};
    use crate::effect::{DelayedTriggerKind, ZoneDest};
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Ennis, Debate Moderator",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Cleric],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: exile a target creature you control + delayed return at
            // next end step.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: crate::effect::shortcut::target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByYou),
                        ),
                        to: ZoneDest::Exile,
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
            },
            // Your end step: if one or more cards were put into exile
            // this turn, +1/+1 counter on Ennis. Uses the exact-printed
            // `Predicate::CardsExiledThisTurnAtLeast` (added in push IX
            // alongside `Player.cards_exiled_this_turn`); previously
            // approximated via `CardsLeftGraveyardThisTurnAtLeast`,
            // which under-counted exile-from-hand or exile-from-library
            // events.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::CardsExiledThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::AddCounter {
                        what: Selector::This,
                        kind: CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Tragedy Feaster — {2}{B}{B} 7/6 Demon.
/// "Trample / Ward—Discard a card. / Infusion — At the beginning of your
/// end step, sacrifice a permanent unless you gained life this turn."
///
/// Body wired (7/6 Demon with Trample). **Ward — Discard a card is now
/// enforced** via `Keyword::Ward(WardCost::Discard(1))` (the same
/// counter-unless-discard ward path Forum Necroscribe uses). The Infusion
/// **end-step sacrifice-unless-lifegain rider** is wired: a
/// `StepBegins(End) / ActivePlayer` trigger fires for the active-player,
/// and the body is an `Effect::If` gated on
/// `Predicate::LifeGainedThisTurnAtLeast(You, 1)` — when the controller
/// has gained life this turn, the trigger resolves as Noop; otherwise it
/// forces `Effect::Sacrifice { who: You, count: 1, filter: Permanent }`.
/// The card is now strictly faithful to the printed Oracle.
pub fn tragedy_feaster() -> CardDefinition {
    use crate::card::Predicate;
    use crate::game::types::TurnStep;
    CardDefinition {
        name: "Tragedy Feaster",
        cost: cost(&[generic(2), b(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 7,
        toughness: 6,
        keywords: vec![Keyword::Trample, Keyword::Ward(crate::card::WardCost::Discard(1))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::End),
                EventScope::ActivePlayer,
            ),
            effect: Effect::If {
                cond: Predicate::LifeGainedThisTurnAtLeast {
                    who: PlayerRef::You,
                    at_least: Value::Const(1),
                },
                then: Box::new(Effect::Noop),
                else_: Box::new(Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Permanent,
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Forum Necroscribe — {5}{B} 5/4 Troll Warlock.
/// "Ward—Discard a card. / Repartee — Whenever you cast an instant or
/// sorcery spell that targets a creature, return target creature card
/// from your graveyard to the battlefield."
///
/// Body + Repartee wired. The Ward—Discard a card rider is omitted (no
/// Ward keyword primitive yet — tracked in TODO.md). The Repartee
/// trigger uses the existing `repartee()` shortcut chained with a
/// graveyard → battlefield Move. The auto-target picker chooses a
/// creature card from your graveyard (Repartee fires off any IS spell
/// targeting a creature, and the body picks the highest-impact eligible
/// gy card).
pub fn forum_necroscribe() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::effect::shortcut::repartee;
    CardDefinition {
        name: "Forum Necroscribe",
        cost: cost(&[generic(5), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Troll, CreatureType::Warlock],
            ..Default::default()
        },
        power: 5,
        toughness: 4,
        keywords: vec![Keyword::Ward(crate::card::WardCost::Discard(1))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![repartee(Effect::Move {
            what: crate::effect::shortcut::target_filtered(SelectionRequirement::Creature),
            to: ZoneDest::Battlefield {
                controller: PlayerRef::You,
                tapped: false,
            },
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Berta, Wise Extrapolator — {2}{G}{U} 1/4 Legendary Frog Druid.
/// "Increment / Whenever one or more +1/+1 counters are put on Berta,
/// add one mana of any color. / {X}, {T}: Create a 0/0 green and blue
/// Fractal creature token and put X +1/+1 counters on it."
///
/// Two of three abilities wired:
/// - Counter-add trigger: fires off any +1/+1 counter landing on Berta
///   (`EventKind::CounterAdded(PlusOnePlusOne)` + new `SelfSource`
///   recognition for CounterAdded events). Adds 1 mana of any color
///   (player picks via the engine's `ChooseColor` decision flow).
/// - X-cost activation: tap + pay X generic, mint a 0/0 G/U Fractal
///   token + add X +1/+1 counters on the freshly-minted token via
///   `Selector::LastCreatedToken`. The X-from-cost path uses
///   `Value::XFromCost` keyed off the activation's mana payment.
///
/// The Increment rider (whenever you cast a spell, if mana spent > P
/// or T, +1/+1 counter on Berta) is omitted pending the SOS Increment
/// engine primitive (mana-spent-on-cast introspection — tracked in
/// TODO.md).
pub fn berta_wise_extrapolator() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType, Supertype};
    use crate::effect::ManaPayload;
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::mana::{g, u};
    use super::sorceries::fractal_token;
    // Push (modern_decks, batch 87): the printed `{X}, {T}: Create a
    // 0/0 G/U Fractal token and put X +1/+1 counters on it` activation
    // is approximated as a fixed `{2}, {T}: Create a Fractal + 2
    // counters`. The engine's `activate_ability` path doesn't accept an
    // x_value (X resolves to 0 for activated abilities, which would
    // mint a 0/0 token that immediately dies to SBA — strictly
    // worse than the printed Oracle in every scenario). The fixed
    // {2} approximation captures a typical mid-game play pattern (a
    // 2/2 Fractal for 2 mana via Berta's tap); the X-scaling at higher
    // mana counts is the remaining engine gap (same X-cost activation
    // gap as Tester of the Tangential's combat-step pay-X trigger).
    CardDefinition {
        name: "Berta, Wise Extrapolator",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(2),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![
            // Increment (push XVII): "Whenever you cast a spell, if the
            // amount of mana you spent is greater than this creature's
            // power or toughness, put a +1/+1 counter on this creature."
            // The CounterAdded → AddMana trigger below then fires off
            // each Increment counter, producing the printed "extra mana
            // for big spells" payoff.
            increment_self_plus_one(),
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                    EventScope::SelfSource,
                ),
                effect: Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::AnyOneColor(Value::Const(1)),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Paradox Surveyor — {G}{G/U}{U} 3/3 Elf Druid.
/// "Reach / When this creature enters, look at the top five cards of
/// your library. You may reveal a land card or a card with {X} in its
/// mana cost from among them and put it into your hand. Put the rest on
/// the bottom of your library in a random order."
///
/// Now wired (push XVI): the "land OR card with {X}" filter uses the
/// new `SelectionRequirement::HasXInCost` predicate ORed with `Land`.
/// Misses go to graveyard (engine default for `RevealUntilFind`); the
/// printed "rest on bottom random order" rider is approximated. The
/// `{G/U}` pip is a real `ManaSymbol::Hybrid(Green, Blue)`, payable with
/// either green or blue.
pub fn paradox_surveyor() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{g, u};
    CardDefinition {
        name: "Paradox Surveyor",
        cost: cost(&[g(), crate::mana::hybrid(Color::Green, Color::Blue), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::RevealUntilFind {
                who: PlayerRef::You,
                find: SelectionRequirement::Land.or(SelectionRequirement::HasXInCost),
                to: ZoneDest::Hand(PlayerRef::You),
                cap: Value::Const(5),
                life_per_revealed: 0,
                // Printed: "Put the rest on the bottom of your library
                // in a random order."
                miss_dest: crate::effect::RevealMissDest::BottomRandom,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Magmablood Archaic — {2/R}{2/R}{2/R} 2/2 Avatar.
/// "Trample, reach / Converge — This creature enters with a +1/+1
/// counter on it for each color of mana spent to cast it. / Whenever
/// you cast an instant or sorcery spell, creatures you control get
/// +1/+0 until end of turn for each color of mana spent to cast that
/// spell."
///
/// The `{2/R}` pips are real `ManaSymbol::MonoHybrid(2, Red)` — each pip
/// is payable with either {2} generic or one red, and the mana value is
/// 6 (CR 202.3f). Trample + reach + Converge ETB counter are wired
/// exactly like Rancorous Archaic. The spell-cast pump uses
/// `Value::ConvergedValue` for the iterated cast — but the engine
/// re-uses the *current cast's* converge value, not the just-cast
/// spell's. We approximate by reading the trigger source's
/// converge-from-stack via the `StackItem::Trigger.converged_value`
/// inheritance set up in push III. For the typical 2-color cube spell
/// this lands +2/+0 EOT on each friendly creature, which matches the
/// printed effect on a 2-color cast.
pub fn magmablood_archaic() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::magecraft;
    use crate::mana::mono_hybrid;
    CardDefinition {
        name: "Magmablood Archaic",
        cost: cost(&[
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::Red),
            mono_hybrid(2, Color::Red),
        ]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // Converge ETB — gain +1/+1 counters equal to colors spent.
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ConvergedValue,
                },
            },
            // Push (modern_decks): per-cast IS pump. Each friendly creature
            // gets +X/+0 EOT where X = colors spent on the iterated spell.
            // Reads `Value::ConvergedValue` which is now threaded onto the
            // spell-cast trigger via `fire_spell_cast_triggers` so each
            // iterated cast's converge count is correctly observed.
            magecraft(Effect::PumpPT {
                what: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                power: Value::ConvergedValue,
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Wildgrowth Archaic — {2/G}{2/G} 0/0 Avatar.
/// "Trample, reach / Converge — This creature enters with a +1/+1
/// counter on it for each color of mana spent to cast it. / Whenever
/// you cast a creature spell, that creature enters with X additional
/// +1/+1 counters on it, where X is the number of colors of mana spent
/// to cast it."
///
/// Body + Converge ETB wired (same pattern as Rancorous Archaic /
/// Magmablood Archaic). The `{2/G}` pips are real
/// `ManaSymbol::MonoHybrid(2, Green)` (CMC 4, payable with {2} or {G}
/// per pip). The printed 0/0 means
/// the creature dies to SBA without enough Converge counters; mono-G
/// or off-color casts will die immediately, while a 2-color cast lands
/// it as a 2/2. The "creature spells you cast enter with X extra
/// counters" rider is omitted pending an `EventKind::SpellCast` filter
/// that captures the just-cast spell's converged value at trigger time
/// (today the trigger fires but the body pump runs against the source's
/// own converged value, not the cast spell's).
pub fn wildgrowth_archaic() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::{StaticAbility, StaticEffect};
    use crate::mana::mono_hybrid;
    CardDefinition {
        name: "Wildgrowth Archaic",
        cost: cost(&[mono_hybrid(2, Color::Green), mono_hybrid(2, Color::Green)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample, Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        // Two statics: (a) the Converge "enters with X counters on it"
        // for the Archaic itself (CR 614.12 — counters land before
        // SBA so 0/0 + X survives); (b) the "creature spells you cast
        // enter with X additional +1/+1 counters" rider for OTHER
        // creature spells the controller casts. Both rely on the
        // engine's spell-side `Value::ConvergedValue` introspection.
        // Push (modern_decks batch 30): both halves now ship via the
        // new `StaticEffect::ExtraEtbCountersForCreatureCasts`
        // primitive — the static fires for every creature spell the
        // controller casts (including the Archaic itself; the engine
        // walks the controller's battlefield at resolve-spell time
        // after the new permanent has been pushed, so the fresh
        // Archaic also sees its own static).
        static_abilities: vec![StaticAbility {
            description: "Whenever you cast a creature spell, that creature \
                          enters with X additional +1/+1 counters on it, \
                          where X is the number of colors of mana spent to \
                          cast it.",
            effect: StaticEffect::ExtraEtbCountersForCreatureCasts {
                kind: CounterType::PlusOnePlusOne,
                value: Value::ConvergedValue,
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Ambitious Augmenter — {G} 1/1 Turtle Wizard.
/// "Increment (Whenever you cast a spell, if the amount of mana you
/// spent is greater than this creature's power or toughness, put a
/// +1/+1 counter on this creature.) / When this creature dies, if it
/// had one or more counters on it, create a 0/0 green and blue Fractal
/// creature token, then put this creature's counters on that token."
///
/// Body wired (1/1 Turtle Wizard at {G} — Increment-grown shell).
/// Increment now uses `shortcut::increment_self_plus_one()`. The
/// death-with-counters → Fractal-with-counters trigger is still
/// omitted (engine has no `Selector::Self.counters_at_death` snapshot
/// — we'd need a counter-transfer-on-death primitive, tracked
/// separately).
pub fn ambitious_augmenter() -> CardDefinition {
    use crate::card::CounterType;
    use crate::catalog::sets::sos::sorceries::fractal_token;
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::effect::Predicate;
    use crate::mana::g;
    // Death trigger: if the dying creature had one or more counters on
    // it, mint a 0/0 Fractal token and transfer the counters to it
    // (CR 122.2 — counters persist on a card's CardInstance across the
    // bf → gy zone change, so `Value::CountersOn(This)` reads the
    // gy-resident card's preserved counter count at trigger-resolve
    // time).
    let death_trigger = TriggeredAbility {
        event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource),
        effect: Effect::If {
            cond: Predicate::ValueAtLeast(
                Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
                Value::Const(1),
            ),
            then: Box::new(Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::This),
                        kind: CounterType::PlusOnePlusOne,
                    },
                },
            ])),
            else_: Box::new(Effect::Noop),
        },
    };
    CardDefinition {
        name: "Ambitious Augmenter",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Turtle, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![increment_self_plus_one(), death_trigger],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Rubble Rouser — {2}{R} Creature — Dwarf Sorcerer.
/// 1/4. ETB may-discard-then-draw + `{T}, Exile a card from your
/// graveyard: Add {R}. When you do, this creature deals 1 damage to
/// each opponent.`
///
/// Push (modern_decks): the `{T}, Exile a card from your graveyard:`
/// activation is **now wired** via the existing
/// `ActivatedAbility.exile_other_filter: Some(Any)` field (engine's
/// "exile another card from your gy as cost" primitive, same one
/// powering Postmortem Professor + Lorehold Pledgemage). The body
/// folds the `When you do` sub-trigger into the activation's main
/// effect — once the cost is paid the engine resolves `AddMana` plus
/// the 1-damage-each-opp simultaneously (CR 603's separate sub-trigger
/// would resolve them on the stack independently; this approximation
/// preserves the printed payoff with a slightly tighter timing).
pub fn rubble_rouser() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::ManaPayload;
    use crate::mana::{r, Color, ManaCost};
    CardDefinition {
        name: "Rubble Rouser",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dwarf, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 1,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: ManaCost::default(),
            effect: Effect::Seq(vec![
                Effect::AddMana {
                    who: PlayerRef::You,
                    pool: ManaPayload::Colors(vec![Color::Red]),
                },
                Effect::DealDamage {
                    to: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: Some((SelectionRequirement::Any, 1)),
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description: "Rubble Rouser ETB: discard a card, then draw a card?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ])),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Witherbloom finisher ────────────────────────────────────────────────────

/// Professor Dellian Fel — {2}{B}{G} Legendary Planeswalker — Dellian [5].
///
/// "+2: You gain 3 life. / 0: You draw a card and lose 1 life. / -3:
/// Destroy target creature. / -7: You get an emblem with 'Whenever you
/// gain life, target opponent loses that much life.'"
///
/// Wired with the three numerical abilities (`+2 gain 3`, `0 draw 1 / lose
/// 1`, `-3 destroy creature`). The `-7` emblem clause is omitted —
/// emblems aren't a modelled zone yet (see TODO.md "Planeswalker
/// Interactions"). The base shell is the standard Witherbloom
/// removal-and-card-draw planeswalker, leveraging the existing engine
/// loyalty-ability machinery; the `-3` ability uses
/// `target_filtered(Creature)` so the auto-target picker takes a
/// hostile creature when one is available.
pub fn professor_dellian_fel() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    use crate::effect::shortcut::target_filtered;
    use crate::mana::g;
    CardDefinition {
        name: "Professor Dellian Fel",
        cost: cost(&[generic(2), b(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Dellian],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 5,
        loyalty_abilities: vec![
            // +2: You gain 3 life.
            LoyaltyAbility {
                loyalty_cost: 2,
                effect: Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(3),
                },
            },
            // 0: You draw a card and lose 1 life.
            LoyaltyAbility {
                loyalty_cost: 0,
                effect: Effect::Seq(vec![
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                    Effect::LoseLife {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            // -3: Destroy target creature.
            LoyaltyAbility {
                loyalty_cost: -3,
                effect: Effect::Destroy {
                    what: target_filtered(SelectionRequirement::Creature),
                },
            },
            // -6: You get an emblem with "Whenever you gain life,
            // target opponent loses that much life." Approximated as a
            // per-player flag `Player.dellian_fel_emblem` that the
            // unified dispatcher reads on LifeGained events.
            LoyaltyAbility {
                loyalty_cost: -6,
                effect: Effect::ActivateDellianEmblem {
                    who: PlayerRef::You,
                },
            },
        ],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Ral Zarek, Guest Lecturer — {1}{B}{B} Legendary Planeswalker — Ral [3].
///
/// "+1: Surveil 2. / -1: Any number of target players each discard a
/// card. / -2: Return target creature card with mana value 3 or less
/// from your graveyard to the battlefield. / -7: Flip five coins.
/// Target opponent skips their next X turns, where X is the number of
/// coins that came up heads."
///
/// Wired with the +1 (Surveil 2), -1 (each opponent discards 1, single-
/// target collapse of the printed "any number of target players"), and
/// -2 (return ≤3-MV creature card from your gy → bf). The -7 ult is
/// omitted — engine has no coin-flip primitive nor a "skip turns"
/// modifier (TODO.md). Note the printed cost is `{1}{B}{B}` despite
/// the Ral subtype, matching this Witherbloom-flavoured Ral variant.
pub fn ral_zarek_guest_lecturer() -> CardDefinition {
    use crate::card::{LoyaltyAbility, PlaneswalkerSubtype, Supertype};
    use crate::effect::ZoneDest;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Ral Zarek, Guest Lecturer",
        cost: cost(&[generic(1), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Ral],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 3,
        loyalty_abilities: vec![
            // +1: Surveil 2.
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            },
            // -1: Each opponent discards a card (single-target collapse
            // of the printed "any number of target players each discard
            // a card" — no multi-target prompt for instants/sorceries).
            LoyaltyAbility {
                loyalty_cost: -1,
                effect: Effect::Discard {
                    who: Selector::Player(PlayerRef::EachOpponent),
                    amount: Value::Const(1),
                    random: false,
                },
            },
            // -2: Return target creature card with MV ≤ 3 from your
            // graveyard to the battlefield.
            LoyaltyAbility {
                loyalty_cost: -2,
                effect: Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ManaValueAtMost(3)),
                    ),
                    to: ZoneDest::Battlefield {
                        controller: PlayerRef::You,
                        tapped: false,
                    },
                },
            },
            // -7: Flip five coins. Target opponent skips their next X
            // turns, where X is the number of coins that came up heads.
            // FlipCoin's on_heads/on_tails branches each fire once per
            // flip; on heads we SkipTurns(target opp, 1). After 5 flips
            // the opp's `skip_turns` counter accumulates the heads-count,
            // which the turn-advance logic decrements.
            LoyaltyAbility {
                loyalty_cost: -7,
                effect: Effect::FlipCoin {
                    count: Value::Const(5),
                    on_heads: Box::new(Effect::SkipTurns {
                        who: PlayerRef::EachOpponent,
                        count: Value::Const(1),
                    }),
                    on_tails: Box::new(Effect::Noop),
                },
            },
        ],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Textbook Tabulator — {2}{U} 0/3 Frog Wizard.
/// "Increment (Whenever you cast a spell, if the amount of mana you spent
/// is greater than this creature's power or toughness, put a +1/+1
/// counter on this creature.) / When this creature enters, surveil 2."
///
/// Body wired with the printed 0/3 Frog Wizard stats; the ETB Surveil 2
/// is wired faithfully via `Effect::Surveil`. The Increment rider is
/// omitted (no per-cast mana-spent introspection — same gap as
/// Pensive Professor / Hungry Graffalon / Tester of the Tangential).
pub fn textbook_tabulator() -> CardDefinition {
    use crate::effect::shortcut::increment_self_plus_one;
    use crate::mana::u;
    CardDefinition {
        name: "Textbook Tabulator",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            },
            // Increment rider: "Whenever you cast a spell, if the amount
            // of mana you spent is greater than this creature's power or
            // toughness, put a +1/+1 counter on this creature." Uses the
            // shared `increment_self_plus_one()` shortcut so the
            // SBA-tracked Frog grows into a real 3-toughness wall the
            // longer the spellslinger game runs.
            increment_self_plus_one(),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Deluge Virtuoso — {2}{U} 2/2 Human Wizard.
/// "When this creature enters, tap target creature an opponent controls
/// and put a stun counter on it. / Opus — Whenever you cast an instant
/// or sorcery spell, this creature gets +1/+1 until end of turn. If
/// five or more mana was spent to cast that spell, this creature gets
/// +2/+2 until end of turn instead."
///
/// ETB tap+stun wired (same shape as Fractal Mascot's Quandrix variant).
/// Opus rider now wired via `shortcut::opus_trigger`: +1/+1 EOT, or
/// +2/+2 EOT for ≥5-mana IS spells.
pub fn deluge_virtuoso() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::Duration;
    use crate::effect::shortcut::{opus_trigger, target_filtered};
    use crate::mana::u;
    let pump = |amt: i32| Effect::PumpPT {
        what: Selector::This,
        power: Value::Const(amt),
        toughness: Value::Const(amt),
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Deluge Virtuoso",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Seq(vec![
                    Effect::Tap {
                        what: target_filtered(
                            SelectionRequirement::Creature
                                .and(SelectionRequirement::ControlledByOpponent),
                        ),
                    },
                    Effect::AddCounter {
                        what: Selector::Target(0),
                        kind: CounterType::Stun,
                        amount: Value::Const(1),
                    },
                ]),
            },
            opus_trigger(pump(1), pump(2)),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Moseo, Vein's New Dean — {2}{B} Legendary Creature — Bird Skeleton
/// Warlock 2/1 Flying.
/// "Flying / When Moseo enters, create a 1/1 black and green Pest
/// creature token with 'Whenever this token attacks, you gain 1 life.' /
/// Infusion — At the beginning of your end step, if you gained life
/// this turn, return up… (oracle truncated)"
///
/// Body + Flying + ETB Pest token wired faithfully. The Infusion end-step
/// rider — "if you gained life this turn, return up to one target creature
/// card from your graveyard to your hand" — is now also wired (push
/// modern_decks). The end-step trigger fires for the active player; the
/// body is gated on `Predicate::LifeGainedThisTurnAtLeast(You, 1)` (the
/// canonical Infusion gate, also used by Foolish Fate, Old-Growth
/// Educator, Efflorescence, Poisoner's Apprentice). Inside the gate,
/// `Effect::Move { what: take(1, CardsInZone(Graveyard, Creature)), to:
/// Hand(You) }` reanimates-to-hand the top matching creature card. The
/// "up to one" semantics fall out naturally — when the graveyard has no
/// matching cards, the move resolves to nothing.
pub fn moseo_veins_new_dean() -> CardDefinition {
    use super::sorceries::pest_token;
    use crate::card::{Predicate, Supertype, Zone};
    use crate::effect::{Duration as _Duration, ZoneDest};
    let _ = _Duration::EndOfTurn;
    CardDefinition {
        name: "Moseo, Vein's New Dean",
        cost: cost(&[generic(2), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![
                CreatureType::Bird,
                CreatureType::Skeleton,
                CreatureType::Warlock,
            ],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: pest_token(),
                },
            },
            // Infusion end-step: if you gained life this turn, return up
            // to one creature card from your graveyard to your hand.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(crate::game::types::TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::LifeGainedThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::Move {
                        what: Selector::take(
                            Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: Zone::Graveyard,
                                filter: SelectionRequirement::Creature,
                            },
                            Value::Const(1),
                        ),
                        to: ZoneDest::Hand(PlayerRef::You),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Stone Docent — {1}{W} 3/1 Spirit Chimera.
/// "{W}, Exile this card from your graveyard: You gain 2 life. Surveil
/// 1. Activate only as a sorcery."
///
pub fn stone_docent() -> CardDefinition {
    CardDefinition {
        name: "Stone Docent",
        cost: cost(&[generic(1), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            // No "Chimera" creature type yet — bridge through Spirit alone.
            creature_types: vec![CreatureType::Spirit],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[w()]),
            effect: Effect::Seq(vec![
                Effect::GainLife {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Surveil {
                    who: PlayerRef::You,
                    amount: Value::Const(1),
                },
            ]),
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: true,
            exile_self_cost: true, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Page, Loose Leaf — {2} Legendary Artifact Creature — Construct 0/2.
/// "{T}: Add {C}. / Grandeur — Discard another card named Page, Loose
/// Leaf: Reveal cards from the top of your library until you reveal an
/// instant or sorcery card. Put that card into your hand and the rest
/// on the bottom of your library in a random order."
///
/// Body wired (0/2 Legendary Construct) + the printed `{T}: Add {C}`
/// mana ability via the shared `tap_add_colorless()` helper. The
/// Grandeur ability (discard-named-this for impulsive draw) is omitted
/// — Grandeur is a singleton-set-discount mechanic with no engine
/// equivalent yet (no card-name-as-cost activation). The mana-rock body
/// still slots into colorless utility pools.
pub fn page_loose_leaf() -> CardDefinition {
    use super::super::tap_add_colorless;
    use crate::card::{ActivatedAbility, Predicate, Supertype, Zone};
    use crate::effect::{RevealMissDest, ZoneDest};
    // Push (modern_decks, batch 92): Grandeur "Discard another card
    // named Page, Loose Leaf: reveal until creature → bf, rest →
    // bottom random" wired as an activated ability with zero mana
    // cost, gated on `Predicate::SameNamedInZoneAtLeast { who: You,
    // zone: Hand, at_least: 1 }` (≥ 1 other Page in hand), and body
    // = `Seq(Discard 1, RevealUntilFind(Creature, → bf,
    // miss=BottomRandom))`. The "discard another Page" cost is
    // approximated by gating on another Page in hand + auto-discarding
    // 1 card — the auto-decider picks the first hand card, which in a
    // tested deck with multiple Pages will frequently be the other
    // Page. The Predicate now reads from `ctx.source` as a fallback
    // (engine-wide tweak in batch 92 alongside this card) so the
    // activation-time gate fires correctly for non-spell paths.
    CardDefinition {
        name: "Page, Loose Leaf",
        cost: cost(&[generic(2)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 0,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![
            tap_add_colorless(),
            ActivatedAbility {
                tap_cost: false,
                mana_cost: ManaCost::default(),
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::RevealUntilFind {
                        who: PlayerRef::You,
                        find: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                        to: ZoneDest::Hand(PlayerRef::You),
                        cap: Value::Const(60),
                        life_per_revealed: 0,
                        miss_dest: RevealMissDest::BottomRandom,
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
                sac_cost: false,
                condition: Some(Predicate::SameNamedInZoneAtLeast {
                    who: PlayerRef::You,
                    zone: Zone::Hand,
                    at_least: Value::Const(1),
                }),
                life_cost: 0,
                from_graveyard: false,
                exile_self_cost: false,
                exile_other_filter: None,
                self_counter_cost_reduction: None, sac_other_filter: None,
                tap_other_filter: None,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

/// Essenceknit Scholar — {B}{B/G}{G} 3/1 Dryad Warlock.
/// "When this creature enters, create a 1/1 black and green Pest creature
/// token with 'Whenever this token attacks, you gain 1 life.' / At the
/// beginning of your end step, if a creature died under your control
/// this turn, draw a card."
///
/// The `{B/G}` pip is a real `ManaSymbol::Hybrid(Black, Green)`, payable
/// with either black or green. Both
/// triggers wired faithfully — the ETB Pest token rides on the shared
/// `pest_token()` helper (so its on-attack lifegain rider trickles into
/// Witherbloom payoffs); the end-step draw uses the new
/// `Predicate::CreaturesDiedThisTurnAtLeast` gate, scoped to the active
/// player so it fires once per controller's own end step.
pub fn essenceknit_scholar() -> CardDefinition {
    use super::sorceries::pest_token;
    use crate::card::Predicate;
    use crate::game::types::TurnStep;
    use crate::mana::g;
    CardDefinition {
        name: "Essenceknit Scholar",
        cost: cost(&[b(), crate::mana::hybrid(Color::Black, Color::Green), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dryad, CreatureType::Warlock],
            ..Default::default()
        },
        power: 3,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // ETB: create a 1/1 B/G Pest token (with on-attack lifegain rider).
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: pest_token(),
                },
            },
            // Your end step: if a creature died under your control this
            // turn, draw a card.
            TriggeredAbility {
                event: EventSpec::new(
                    EventKind::StepBegins(TurnStep::End),
                    EventScope::ActivePlayer,
                ),
                effect: Effect::If {
                    cond: Predicate::CreaturesDiedThisTurnAtLeast {
                        who: PlayerRef::You,
                        at_least: Value::Const(1),
                    },
                    then: Box::new(Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    }),
                    else_: Box::new(Effect::Noop),
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Biblioplex Tomekeeper ───────────────────────────────────────────────────

/// Biblioplex Tomekeeper — {4} Artifact Creature — Construct 3/4.
/// "When this creature enters, choose up to one — / • Target creature
/// becomes prepared. / • Target creature becomes unprepared."
///
/// ✅ ETB ChooseMode wired via `AddCounter`/`RemoveCounter` of
/// `CounterType::Prepared`. "Choose up to one" is approximated as
/// "choose exactly one" — auto-decider picks mode 0 (prepare) by
/// default; ScriptedDecider can switch to mode 1 (unprepare).
pub fn biblioplex_tomekeeper() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::target_filtered;
    // Printed reminder: "(Only creatures with prepare spells can
    // become prepared.)" In this set, a "prepare spell" is a back-face
    // spell on a creature, so the legal-target rule reduces to
    // `Creature ∧ HasBackFace`. Both modes share the same filter —
    // unpreparing a creature with no back face is also illegal per
    // the reminder text (and a no-op anyway since such a creature
    // can never have acquired the counter through legal play).
    let prepare_target = || {
        target_filtered(
            SelectionRequirement::Creature.and(SelectionRequirement::HasBackFace),
        )
    };
    CardDefinition {
        name: "Biblioplex Tomekeeper",
        cost: cost(&[generic(4)]),
        supertypes: vec![],
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ChooseMode(vec![
                // Mode 0: target creature becomes prepared.
                Effect::AddCounter {
                    what: prepare_target(),
                    kind: CounterType::Prepared,
                    amount: Value::Const(1),
                },
                // Mode 1: target creature becomes unprepared.
                Effect::RemoveCounter {
                    what: prepare_target(),
                    kind: CounterType::Prepared,
                    amount: Value::Const(1),
                },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── The Dawning Archaic ─────────────────────────────────────────────────────

/// The Dawning Archaic — {10} Legendary Creature — Avatar 7/7.
/// "This spell costs {1} less to cast for each instant and sorcery card
/// in your graveyard. / Reach / Whenever The Dawning Archaic attacks,
/// you may cast target instant or sorcery card from your graveyard
/// without paying its mana cost. If that spell would be put into your
/// graveyard, exile it instead."
///
/// Push (modern_decks): the attack-triggered free-cast-from-graveyard
/// rider is **now wired** via `Effect::CastWithoutPayingImmediate`
/// targeting a target IS card in the controller's graveyard, with
/// `exile_after = true` (per printed "if that spell would go to a
/// graveyard, exile it instead"). The IS-in-gy cost-reduction static
/// is still omitted — engine has no per-graveyard-IS-count cost-
/// reduction primitive (tracked in TODO.md).
pub fn the_dawning_archaic() -> CardDefinition {
    use crate::card::{Supertype, Zone};
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "The Dawning Archaic",
        cost: cost(&[generic(10)]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
            effect: Effect::CastWithoutPayingImmediate {
                what: target_filtered(
                    SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                ),
                source_zone: Zone::Graveyard,
                exile_after: true,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Silverquill, the Disputant ──────────────────────────────────────────────

/// Silverquill, the Disputant — {2}{W}{B} Legendary Creature — Elder Dragon 4/4.
/// "Flying, vigilance / Each instant and sorcery spell you cast has
/// casualty 1."
///
/// Silverquill, the Disputant — 4/4 Legendary Elder Dragon, Flying +
/// Vigilance. "Each instant and sorcery spell you cast has casualty 1"
/// wired as a SpellCast/YourControl trigger → may-sacrifice a power-≥1
/// creature → copy the spell.
pub fn silverquill_the_disputant() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, Supertype, TriggeredAbility};
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    // Push (modern_decks, batch 91): "Each instant and sorcery spell
    // you cast has casualty 1." Wired as a SpellCast/YourControl
    // trigger gated on `cast_is_instant_or_sorcery()` + `Effect::MayDo
    // { Seq([Sacrifice(Creature with power ≥ 1), CopySpell(TriggerSource)]) }`.
    // AutoDecider declines (the printed casualty is a "you may" cost);
    // ScriptedDecider can accept to exercise the sac+copy path. The
    // power-≥-1 sub-filter on the sacrifice picker is implemented via
    // `Sacrifice { filter: PowerAtLeast(1) }`. Copy inherits original
    // targets (engine-wide gap shared with all CopySpell users).
    CardDefinition {
        name: "Silverquill, the Disputant",
        cost: cost(&[generic(2), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::MayDo {
                description: "Casualty 1: sacrifice a creature with power 1 or greater to copy the spell?".into(),
                body: Box::new(Effect::Seq(vec![
                    Effect::Sacrifice {
                        who: Selector::You,
                        count: Value::Const(1),
                        filter: SelectionRequirement::Creature
                            .and(SelectionRequirement::PowerAtLeast(1)),
                    },
                    Effect::CopySpell {
                        what: Selector::TriggerSource,
                        count: Value::Const(1),
                    },
                ])),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Nita, Forum Conciliator ─────────────────────────────────────────────────

/// Nita, Forum Conciliator — {1}{W}{B} Legendary Creature — Human Advisor 2/3.
/// "Whenever you cast a spell you don't own, put a +1/+1 counter on
/// each creature you control. / {2}, Sacrifice another creature: Exile
/// target instant or sorcery card from an opponent's graveyard. You
/// may cast it this turn, and mana of any type can be spent to cast
/// that spell. Activate only as a sorcery."
///
/// Push (modern_decks): the `{2}, Sacrifice another creature` activation
/// is **now wired** via the new cast-from-exile primitives. The activation
/// exiles a target IS card from an opponent's graveyard and grants
/// `may_play_until: EndOfThisTurn` with `exile_after: true` (so the
/// resolved spell routes to exile, matching "if that spell would be put
/// into a graveyard, exile it instead"). Sorcery-speed gate via
/// `sorcery_speed: true`; sacrifice-another-creature cost via
/// `sac_cost: true`.
///
/// Push (modern_decks, batch 72): the "Whenever you cast a spell you
/// don't own" trigger is **now wired** via the new
/// `Predicate::CastSpellNotOwnedByYou` predicate. Trigger body fans
/// out +1/+1 counters across each friendly creature via
/// `ForEach(Creature & ControlledByYou) → AddCounter(+1/+1)`. The
/// "mana of any type" rider on the activation is auto-satisfied since
/// the free-cast path skips mana payment.
pub fn nita_forum_conciliator() -> CardDefinition {
    use crate::card::{CounterType, Predicate, Supertype};
    use crate::effect::{EventKind, EventScope, EventSpec, TriggeredAbility, ZoneDest};
    let is_in_opp_gy = SelectionRequirement::HasCardType(CardType::Instant)
        .or(SelectionRequirement::HasCardType(CardType::Sorcery));
    let target_card = Selector::take(
        Selector::CardsInZone {
            who: PlayerRef::EachOpponent,
            zone: crate::card::Zone::Graveyard,
            filter: is_in_opp_gy,
        },
        Value::Const(1),
    );
    CardDefinition {
        name: "Nita, Forum Conciliator",
        cost: cost(&[generic(1), w(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Advisor],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(2)]),
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_card,
                    to: ZoneDest::Exile,
                },
                Effect::GrantMayPlay {
                    what: Selector::LastMoved,
                    duration: crate::card::MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: true,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: true,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None,
        }],
        triggered_abilities: vec![TriggeredAbility {
            // "Whenever you cast a spell you don't own, put a +1/+1
            // counter on each creature you control."
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(Predicate::CastSpellNotOwnedByYou),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                body: Box::new(Effect::AddCounter {
                    what: Selector::TriggerSource,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Mica, Reader of Ruins ───────────────────────────────────────────────────

/// Mica, Reader of Ruins — {3}{R} Legendary Creature — Human Artificer 4/4.
/// "Ward—Pay 3 life. / Whenever you cast an instant or sorcery spell,
/// you may sacrifice an artifact. If you do, copy that spell and you
/// may choose new targets for the copy."
///
/// Mica, Reader of Ruins — 4/4 Legendary Human Artificer, Ward—Pay 3 life
/// (enforced). Magecraft trigger wraps `Effect::MayDo` around
/// `Seq(Sacrifice(Artifact) + CopySpell(TriggerSource))`; the auto-decider
/// declines by default, so the copy fires only when scripted yes.
pub fn mica_reader_of_ruins() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    use crate::mana::r;
    CardDefinition {
        name: "Mica, Reader of Ruins",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Ward(crate::card::WardCost::Life(3))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Mica, Reader of Ruins: sacrifice an artifact to copy the spell?".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::HasCardType(CardType::Artifact),
                },
                Effect::CopySpell {
                    what: Selector::TriggerSource,
                    count: Value::Const(1),
                },
            ])),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Colorstorm Stallion ─────────────────────────────────────────────────────

// ── Elemental Mascot ────────────────────────────────────────────────────────

// ── Prismari, the Inspiration ───────────────────────────────────────────────

/// Prismari, the Inspiration — {5}{U}{R} Legendary Creature — Elder Dragon
/// 7/7. "Flying / Ward—Pay 5 life. / Instant and sorcery spells you cast
/// have storm."
///
/// Body wired: 7/7 Flying Legendary Elder Dragon with
/// `Keyword::Ward(WardCost::Life(5))` (Ward—Pay 5 life, now enforced).
/// The "your IS spells have storm" static is wired via a per-cast
/// `CopySpell { count: StormCount }` trigger (see the inline note
/// below).
pub fn prismari_the_inspiration() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
    use crate::mana::{r, u};
    // Push (modern_decks, batch 89): "Instant and sorcery spells you
    // cast have storm" is wired via a SpellCast/YourControl trigger
    // gated on `cast_is_instant_or_sorcery` + `Effect::CopySpell {
    // what: TriggerSource, count: Value::StormCount }`. Each IS spell
    // cast while Prismari is on the bf fires a copy-it-N-times trigger
    // (N = spells_cast_this_turn − 1). The copy targets default to the
    // original's targets (engine-wide gap shared with all CopySpell
    // users — "you may choose new targets" is not modeled). When
    // Prismari leaves the bf, the trigger is no longer on her, so
    // future casts don't get the storm grant.
    CardDefinition {
        name: "Prismari, the Inspiration",
        cost: cost(&[generic(5), u(), r()]),
        supertypes: vec![crate::card::Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Elder],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying, Keyword::Ward(crate::card::WardCost::Life(5))],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::CopySpell {
                what: Selector::TriggerSource,
                count: Value::StormCount,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        max_counters_of_kind: None,
        exile_on_resolve: false,
        affinity_filter: None,
        equipped_bonus: None,
    }
}

// ── Colorless Artifact Creatures ───────────────────────────────────────────

// ── Colorless Legendary Creatures ──────────────────────────────────────────

// ── Multicolor Legendary Elder Dragons / Legendary Creatures ──────────────
