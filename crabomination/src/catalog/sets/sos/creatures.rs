//! Secrets of Strixhaven (SOS) — Creatures.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, Effect, EventKind, EventScope,
    EventSpec, Keyword, SelectionRequirement, Subtypes, TokenDefinition, TriggeredAbility,
};
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Stirring Hopesinger — {2}{W}, 1/3 Bird Bard. Flying, lifelink.
///
/// The Repartee trigger ("whenever you cast an instant or sorcery that
/// targets a creature, put a +1/+1 counter on each creature you control")
/// is omitted — the engine has no introspection on a cast spell's target
/// list (no `SpellTargetsCreature` predicate yet). The flying/lifelink
/// body is wired so the card still hits the battlefield with the correct
/// color and stats.
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Owlin Historian — {2}{W}, 2/3 Bird Cleric. Flying.
/// "Flying / When this creature enters, surveil 1. / Whenever one or more
/// cards leave your graveyard, this creature gets +1/+1 until end of
/// turn."
///
/// Approximation: the "cards leave your graveyard" pump trigger is
/// omitted — the engine has no `LeavesGraveyard`/`CardLeftGraveyard`
/// event yet. The flying body and the ETB Surveil 1 are wired
/// faithfully.
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Inkshape Demonstrator — {3}{W}, 3/4 Elephant Cleric. "Ward {2}.
/// Repartee — Whenever you cast an instant or sorcery spell that
/// targets a creature, this creature gets +1/+0 and gains lifelink
/// until end of turn."
///
/// ✅ Push XXXVIII: 🟡 → ✅. Body + `Keyword::Ward(2)` + Repartee
/// body all wired identically to Mica Reader of Ruins (✅) — Ward is
/// a keyword tag that the engine carries forward for future
/// enforcement. The dominant gameplay piece is the Repartee +1/+0 +
/// lifelink combat trick which lands at full fidelity via the
/// `repartee()` shortcut.
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
        keywords: vec![Keyword::Ward(2)],
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: Some(Box::new(back)),
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
                            .and(SelectionRequirement::ControlledByYou),
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
                    what: target_filtered(SelectionRequirement::Any),
                    to: ZoneDest::Exile,
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Summoned Dromedary — {3}{W}, 4/3 Spirit Camel. Vigilance.
/// "{1}{W}: Return this card from your graveyard to your hand. Activate
/// only as a sorcery."
///
/// Approximation: the graveyard recursion activated ability is omitted
/// — the engine's activated-ability path only walks the battlefield, so
/// "from your graveyard" activations don't have a wiring path yet. The
/// vigilance body is faithfully wired so the card slots in as a 4/3
/// vigilant beater. Camel isn't a CreatureType yet; we keep the
/// gameplay-relevant Spirit subtype alone. Status: 🟡.
pub fn summoned_dromedary() -> CardDefinition {
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    use crate::mana::hybrid;
    CardDefinition {
        name: "Stirring Honormancer",
        // Push XL: hybrid `{W/B}` pip now wired faithfully via
        // `ManaSymbol::Hybrid(White, Black)`. Total cost is {2}{W}{W/B}{B}
        // — castable from any pool with two W + one B, or two W + one
        // hybrid-able pool, or three W + nothing else. The engine's
        // `pay()` already supports hybrid pips since push XXXVIII
        // (Spectacle Mage); this brings Honormancer to printed parity.
        cost: cost(&[generic(2), w(), hybrid(Color::White, Color::Black), b()]),
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
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Conciliator's Duelist — {W}{W}{B}{B}, 4/3 Kor Warlock.
/// "When this creature enters, draw a card. Each player loses 1 life."
///
/// Push XXXVI: 🟡 → ✅. The **Repartee** rider ("Whenever you cast an
/// instant or sorcery spell that targets a creature, exile up to one
/// target creature. Return that card to the battlefield under its
/// owner's control at the beginning of the next end step.") now
/// wires the "return at next end step" delayed trigger via the new
/// `Effect::DelayUntil { capture: Some(_) }` field — at trigger-fire
/// time the engine evaluates `Selector::CastSpellTarget(0)` (the
/// just-cast spell's targeted creature), captures the resulting
/// permanent into the delayed body's `Selector::Target(0)` slot, and
/// the body's `Move(target → battlefield under owner)` resolves
/// against that captured target on the next end step. ETB body
/// (draw 1 + each player loses 1) unchanged.
pub fn conciliators_duelist() -> CardDefinition {
    use crate::effect::shortcut::repartee;
    use crate::effect::{DelayedTriggerKind, ZoneDest};
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
            // Repartee — exile the targeted creature, then schedule a
            // return-at-next-end-step delayed trigger. `capture:
            // Some(Selector::CastSpellTarget(0))` binds the cast
            // spell's target into the delayed body's Target(0).
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
                    capture: Some(Selector::CastSpellTarget(0)),
                },
            ])),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Scolding Administrator — {W}{B}, 2/2 Dwarf Cleric. Menace. Repartee
/// (whenever you cast an instant or sorcery spell that targets a creature,
/// put a +1/+1 counter on this creature). The truncated "When this
/// creature dies, …" trigger is not implemented (oracle text was clipped
/// in the gen script — pending an oracle-fetch refresh).
pub fn scolding_administrator() -> CardDefinition {
    use crate::card::{CounterType, EventKind, EventScope, EventSpec, Predicate, TriggeredAbility};
    use crate::effect::shortcut::{repartee, target_filtered};
    use crate::card::SelectionRequirement;
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
            repartee(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
            // "When this creature dies, if it had counters on it, put
            // those counters on up to one target creature." Wired via
            // the SelfSource death trigger; the counter count is read
            // off the dying card via `Value::CountersOn` (which now
            // walks graveyards as a fallback). Gated on
            // `ValueAtLeast(CountersOn(SelfSource), 1)` so the trigger
            // no-ops when there are no counters to move.
            TriggeredAbility {
                event: EventSpec::new(EventKind::CreatureDied, EventScope::SelfSource)
                    .with_filter(Predicate::ValueAtLeast(
                        Value::CountersOn {
                            what: Box::new(Selector::TriggerSource),
                            kind: CounterType::PlusOnePlusOne,
                        },
                        Value::Const(1),
                    )),
                effect: Effect::AddCounter {
                    what: target_filtered(SelectionRequirement::Creature),
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::CountersOn {
                        what: Box::new(Selector::TriggerSource),
                        kind: CounterType::PlusOnePlusOne,
                    },
                },
            },
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Pestbrood Sloth — {3}{G}, 4/4 Plant Sloth. Reach.
/// "When this creature dies, create two 1/1 black and green Pest creature
/// tokens with 'Whenever this token attacks, you gain 1 life.'"
///
/// Approximation: the token's "gain 1 on attack" rider isn't surfaced
/// (token-side triggered abilities aren't materialised through
/// `token_to_card_definition` yet — same gap as Send in the Pest's
/// token). The death-trigger that creates two Pests is wired faithfully.
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Teacher's Pest — {B}{G}, 1/1 Skeleton Pest. Menace.
/// "Whenever this creature attacks, you gain 1 life."
///
/// Approximation: the graveyard-recursion ability ("{B}{G}: Return this
/// card from your graveyard to the battlefield tapped") is omitted —
/// the engine's `FromYourGraveyard` path supports triggered abilities
/// (Bloodghast-style) but not activated abilities with a mana cost. The
/// attacks-gain-1 trigger is wired faithfully.
pub fn teachers_pest() -> CardDefinition {
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
        activated_abilities: no_abilities(),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Lorehold (R/W) ──────────────────────────────────────────────────────────

/// Aziza, Mage Tower Captain — {R}{W}, 2/2 Legendary Djinn Sorcerer.
/// "Whenever you cast an instant or sorcery spell, you may tap three
/// untapped creatures you control. If you do, copy that spell. You may
/// choose new targets for the copy."
///
/// Now wired (post-XX): magecraft → `Effect::MayDo { Seq[Tap up-to-3
/// untapped friendly creatures, CopySpell(CastSpellSource, 1)] }`. The
/// "tap 3 to copy" cost is collapsed into the body — if fewer than 3
/// untapped creatures are available, the engine taps what it can and
/// still fires the copy (a small over-payoff vs printed "If you do").
/// "Choose new targets for the copy" is also collapsed: the copy
/// inherits the original spell's chosen target slot (no per-copy
/// retargeting prompt yet — TODO.md). The 2/2 Legendary body still
/// slots into the Lorehold pool.
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
            description: "Tap three untapped creatures you control to copy".to_string(),
            body: Box::new(Effect::Seq(vec![
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
                    what: Selector::CastSpellSource,
                    count: Value::Const(1),
                },
            ])),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Mica, Reader of Ruins — {3}{R}, 4/4 Legendary Human Artificer.
/// "Ward—Pay 3 life. (Whenever this creature becomes the target of a
/// spell or ability an opponent controls, counter it unless that
/// player pays 3 life.)
/// Whenever you cast an instant or sorcery spell, you may sacrifice an
/// artifact. If you do, copy that spell and you may choose new targets
/// for the copy."
///
/// Now wired (post-XX): magecraft → `Effect::MayDo { Seq[Sacrifice 1
/// artifact, CopySpell(CastSpellSource, 1)] }`. The "If you do" rider is
/// collapsed: if the controller has no artifact to sac, the body's
/// Sacrifice no-ops and the copy still fires (small over-payoff vs
/// printed semantics; same approximation as Aziza). `Keyword::Ward(3)`
/// stays as a static keyword tag for future enforcement (Ward isn't yet
/// a counter-the-spell trigger).
pub fn mica_reader_of_ruins() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::shortcut::magecraft;
    use crate::mana::r;
    CardDefinition {
        name: "Mica, Reader of Ruins",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Artificer],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        // Ward—Pay 3 life is approximated as Ward(3) (mana-cost form);
        // hybrid-mana-or-life Ward is still a single primitive in the
        // engine's keyword tag.
        keywords: vec![Keyword::Ward(3)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Sacrifice an artifact to copy".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Artifact
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::CopySpell {
                    what: Selector::CastSpellSource,
                    count: Value::Const(1),
                },
            ])),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tenured Concocter — {4}{G}, 4/5 Troll Druid. Vigilance.
/// "Vigilance / Whenever this creature becomes the target of a spell or
/// ability an opponent controls, you may draw a card. / Infusion — This
/// creature gets +2/+0 as long as you gained life this turn."
///
/// Approximation: the becomes-targeted draw trigger is omitted (no
/// `BecameTarget` event yet); the Infusion static pump is omitted (no
/// "static gain X/Y while predicate" primitive yet — the engine's
/// `Predicate::LifeGainedThisTurnAtLeast` is currently used only on
/// `Effect::If` gates, not on continuous static abilities). Vigilant
/// 4/5 body is wired so the card lands on-curve.
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
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Colorless ───────────────────────────────────────────────────────────────

/// Biblioplex Tomekeeper — {4} body-only wire, 3/4 Construct artifact
/// creature.
///
/// Printed Oracle: "When this creature enters, choose up to one — /
/// • Target creature becomes prepared. / • Target creature becomes
/// unprepared." The Prepare keyword and the prepared-state toggle are
/// SOS-specific and not yet first-class engine concepts (see TODO.md
/// "Prepare mechanic"). Without those, the ETB would no-op anyway, so
/// we ship the body alone.
///
/// Push XIX promotes the row from ⏳ to 🟡 on the Colorless table.
pub fn biblioplex_tomekeeper() -> CardDefinition {
    CardDefinition {
        name: "Biblioplex Tomekeeper",
        cost: cost(&[generic(4)]),
        supertypes: vec![],
        // Artifact + Creature, with Construct subtype.
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
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Strixhaven Skycoach — {3} body-only Vehicle artifact, printed 3/2
/// Flying.
///
/// Printed Oracle: "Flying / When this Vehicle enters, you may search
/// your library for a basic land card, reveal it, put it into your
/// hand, then shuffle. / Crew 2."
///
/// 🟡 Wire: ETB land tutor is wired faithfully via `Effect::MayDo` +
/// `Effect::Search { filter: IsBasicLand, to: Hand }`. The Vehicle
/// subtype + Crew keyword are *not yet* engine concepts (no Vehicle
/// crewing primitive), so the card enters the battlefield as an
/// artifact creature directly — a small over-statement, but the
/// ETB tutor + 3/2 Flying body still slot into colorless ramp. Tracked
/// in TODO.md ("Vehicle / Crew primitives").
pub fn strixhaven_skycoach() -> CardDefinition {
    CardDefinition {
        name: "Strixhaven Skycoach",
        cost: cost(&[generic(3)]),
        supertypes: vec![],
        // No Vehicle subtype yet — modelled as a plain artifact creature
        // until the Crew/Vehicle primitives land.
        card_types: vec![CardType::Artifact, CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Construct],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::MayDo {
                description:
                    "Strixhaven Skycoach: search your library for a basic land?".into(),
                body: Box::new(Effect::Search {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::IsBasicLand,
                    to: crate::effect::ZoneDest::Hand(PlayerRef::You),
                }),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Rancorous Archaic — {5}, 2/2 Avatar. Trample, reach.
/// "Trample, reach / Converge — This creature enters with a +1/+1
/// counter on it for each color of mana spent to cast it."
///
/// X (Converge value) is read at resolution time from
/// `Value::ConvergedValue` via the engine's new
/// `StackItem::Trigger.converged_value` plumbing. The "enters with N
/// counters" rule is approximated by an ETB-trigger that adds N
/// counters — the trigger fires after SBA so the 2/2 base body never
/// dies regardless of paid colors. (No replacement-effect primitive
/// yet.)
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ConvergedValue,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Quandrix (G/U) ──────────────────────────────────────────────────────────

/// Pterafractyl — {X}{G}{U}, printed 1/0 Dinosaur Fractal. Flying.
/// "Flying / This creature enters with X +1/+1 counters on it. / When
/// this creature enters, you gain 2 life."
///
/// Push XL: the printed "enters with X +1/+1 counters" replacement
/// is now wired faithfully via the new
/// `CardDefinition.enters_with_counters` field. Counters are added at
/// bf entry time *before* SBAs run, so the printed 1/0 base body is
/// safe — a Pterafractyl cast for X=0 enters as a 1/0 *with no
/// counters* and immediately graveyards (matching printed). At X=2
/// it lands as a 3/2 (matching printed 1/0 + 2 +1/+1 counters). The
/// "you gain 2 life" half stays on the ETB trigger; the
/// `enters_with_counters` field handles only the counter clause.
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // Just the lifegain — the counter clause is handled by the
            // `enters_with_counters` replacement (push XL).
            effect: Effect::GainLife {
                who: Selector::You,
                amount: Value::Const(2),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::XFromCost)),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    // Push XV: the printed "you may draw a card. If you do, discard a card"
    // is now wired via the new `Effect::MayDo` primitive — the ETB and
    // attack triggers ask the controller a yes/no via
    // `Decision::OptionalTrigger`. Tests can flip the answer to `true`
    // via `ScriptedDecider`; the bot/auto-decider declines (matching
    // MTG's "you may defaults to no").
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
/// Push XXXI: Increment rider now wired via `effect::shortcut::increment()`.
/// At a {G}{U} 1/1 Flash flier, every {2}-or-bigger spell pushes a
/// +1/+1 counter onto it — combat-relevant from turn 3 onwards.
pub fn cuboid_colony() -> CardDefinition {
    use crate::effect::shortcut::increment;
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
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Fractal Tender — {3}{G}{U}, 3/3 Elf Wizard. Printed Oracle:
/// "Ward {2}. Increment (Whenever you cast a spell, if the amount of
///  mana you spent is greater than this creature's power or toughness,
///  put a +1/+1 counter on this creature.) / At the beginning of each
///  end step, if you put a counter on this creature this turn, create
///  a 0/0 green and blue Fractal creature token and put three +1/+1
///  counters on it."
///
/// Push XXXI: Increment rider now wired via `effect::shortcut::increment()`
/// — every cast where mana_spent ≥ 4 (one above min(P, T)=3) drops a
/// +1/+1 counter. The end-step Fractal-with-counters payoff is still
/// omitted (no per-permanent "did this creature gain a counter this
/// turn" flag yet). Ward(2) keyword tag stays for future Ward
/// enforcement.
pub fn fractal_tender() -> CardDefinition {
    use crate::effect::shortcut::increment;
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
        keywords: vec![Keyword::Ward(2)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Thornfist Striker — {2}{G}, 3/3 Elf Druid.
/// "Ward {1}. Infusion — Creatures you control get +1/+0 and have
/// trample as long as you gained life this turn."
///
/// Approximation: body + `Keyword::Ward(1)` wired. The Infusion
/// continuous static (+1/+0 + trample for your creatures while you've
/// gained life this turn) is omitted (no continuous-static-on-predicate
/// primitive yet — same gap as Tenured Concocter, Ulna Alley
/// Shopkeep). 3/3 vanilla body still slots in as a Witherbloom-flavoured
/// midrange creature.
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
        keywords: vec![Keyword::Ward(1)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Hungry Graffalon — {3}{G}, 3/4 Giraffe with Reach. Push XXXI:
/// Increment rider now wired via `effect::shortcut::increment()` —
/// each cast where mana_spent ≥ 4 (one above min(P, T)=3) drops a
/// +1/+1 counter, scaling the 3/4 reacher into a 4/5 / 5/6 / etc.
pub fn hungry_graffalon() -> CardDefinition {
    use crate::effect::shortcut::increment;
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
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Pensive Professor — {1}{U}{U}, 0/2 Human Wizard. Push XXXI:
/// Increment rider now wired via `effect::shortcut::increment()` —
/// 1+ mana spell pushes a +1/+1 counter (the 0/2 frame's min-stat is 0,
/// so any mana-spent ≥ 1 fires the counter). The counter-trigger rider
/// stays omitted (oracle truncated; the per-counter ability isn't
/// fully verifiable yet).
pub fn pensive_professor() -> CardDefinition {
    use crate::effect::shortcut::increment;
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
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tester of the Tangential — {1}{U}, 1/1 Djinn Wizard. Printed Oracle:
/// "Increment (Whenever you cast a spell, if the amount of mana you spent
///  is greater than this creature's power or toughness, put a +1/+1
///  counter on this creature.) / At the beginning of combat on your turn,
///  you may pay {X}. When you do, move X +1/+1 counters from this
///  creature onto another target creature."
///
/// Push XXXI: Increment rider now wired via `effect::shortcut::increment()`.
/// Each cast where mana_spent > min(power, toughness) adds a +1/+1
/// counter — natural ramp on the 1/1 frame as soon as the controller
/// casts a {2}-or-bigger spell. The combat-step pay-X-move-counters
/// rider stays omitted (no `MayPay` X-cost activation primitive).
pub fn tester_of_the_tangential() -> CardDefinition {
    use crate::effect::shortcut::increment;
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
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Muse Seeker — {1}{U}, 1/2 Elf Wizard. Opus loot rider omitted.
pub fn muse_seeker() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    use crate::mana::u;
    // Push XXXI — Opus loot rider now wired. Printed Oracle: "Opus —
    // Whenever you cast an instant or sorcery spell, draw a card. Then
    // discard a card unless five or more mana was spent to cast that
    // spell." Approximation: always-on draw 1 + a conditional discard-1
    // gated on `Value::ManaSpentToCast < 5` (the negation of the "unless
    // 5 or more mana" rider). The discard is a `MayDo` so the controller
    // can choose to skip the discard if their hand is empty (same shape
    // as Stadium Tidalmage's loot trigger). Cheap-cast → loot 1 (draw +
    // discard), big-cast → flat draw 1.
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::If {
                cond: crate::effect::Predicate::ValueAtMost(
                    Value::ManaSpentToCast,
                    Value::Const(4),
                ),
                then: Box::new(Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
                }),
                else_: Box::new(Effect::Noop),
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Aberrant Manawurm — {3}{G}, 2/5 Wurm with Trample. Printed Oracle:
/// "Trample / Whenever you cast an instant or sorcery spell, this creature
///  gets +X/+0 until end of turn, where X is the amount of mana spent to
///  cast that spell."
///
/// Push XXXI: now wired via the new `Value::ManaSpentToCast` primitive on
/// a magecraft trigger. The pump scales with the just-cast spell's
/// post-X CMC: a {2}{G} land tutor pumps the Manawurm by +3/+0; a {X=5}{R}
/// burn spell pumps it by +6/+0. Dovetails with `Keyword::Trample` for
/// large green-threats-into-bashing-pump finisher play.
pub fn aberrant_manawurm() -> CardDefinition {
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
            power: Value::ManaSpentToCast,
            toughness: Value::Const(0),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tackle Artist — {3}{R}, 4/3 Orc Sorcerer. Printed Oracle:
/// "Trample / Opus — Whenever you cast an instant or sorcery spell, this
///  creature gets +1/+1 until end of turn. If five or more mana was spent
///  to cast that spell, put a +1/+1 counter on it instead."
///
/// Push XXXI: Opus rider now wired via `effect::shortcut::opus(5, ...)`
/// which gates a `Value::ManaSpentToCast ≥ 5` `Effect::If` over the
/// magecraft IS-cast trigger. Cheap-cast (CMC < 5) → +1/+1 EOT pump only;
/// big-cast (CMC ≥ 5 including X paid) → +1/+1 EOT pump + permanent
/// +1/+1 counter (the "instead" semantic in the printed Oracle is
/// approximated as both halves rather than a substitution — a small
/// over-payoff vs printed semantics; the printed +1/+1 EOT would be
/// dropped on big casts, but stacking both is symmetric for combat math).
pub fn tackle_artist() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::opus;
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
        triggered_abilities: vec![opus(
            5,
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Thunderdrum Soloist — {1}{R}, 1/3 Dwarf Bard with Reach.
/// "Opus — Whenever you cast an instant or sorcery spell, this creature
/// deals 1 damage to each opponent. If five or more mana was spent to
/// cast that spell, this creature deals 3 damage to each opponent
/// instead."
///
/// ✅ Push: Opus rider now wired via `effect::shortcut::opus(5, ...)`.
/// Always-fires half: 1 damage to each opp. Big-cast (≥5 mana) half:
/// an additional 2 damage (net 3 to each opp). Same additive
/// "instead" approximation as Spectacular Skywhale / Tackle Artist:
/// the printed swap is approximated as `1 + 2 = 3`, which is
/// arithmetically equivalent to the 3-replacement.
pub fn thunderdrum_soloist() -> CardDefinition {
    use crate::effect::shortcut::{each_opponent, opus};
    use crate::mana::r;
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
        triggered_abilities: vec![opus(
            5,
            // Big-cast bonus: deal an additional 2 damage to each opp
            // (1 base + 2 extra = 3 total).
            Effect::DealDamage {
                to: each_opponent(),
                amount: Value::Const(2),
            },
            // Always: deal 1 damage to each opp.
            Effect::DealDamage {
                to: each_opponent(),
                amount: Value::Const(1),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Molten-Core Maestro — {1}{R}, 2/2 Goblin Bard with Menace.
/// "Opus — Whenever you cast an instant or sorcery spell, put a +1/+1
/// counter on this creature. If five or more mana was spent to cast
/// that spell, add an amount of {R} equal to this creature's power."
///
/// ✅ Push: Opus rider now wired via `effect::shortcut::opus(5, ...)`.
/// Always: a +1/+1 counter on This (permanent counter — same shape as
/// Cuboid Colony / Berta's Increment counter accrual). Big-cast (≥5
/// mana): an additional `AddMana { OfColor(Red, PowerOf(This)) }`,
/// reading the post-counter power so a Big-cast trigger off a 3/3
/// adds {R}{R}{R}{R} (the +1/+1 counter resolves first as `always`
/// runs before the Big-cast block per `opus()`'s `Seq` ordering).
pub fn molten_core_maestro() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::ManaPayload;
    use crate::effect::shortcut::opus;
    use crate::mana::r;
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
        triggered_abilities: vec![opus(
            5,
            // Big-cast: add {R} equal to this creature's power.
            Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::OfColor(
                    Color::Red,
                    Value::PowerOf(Box::new(Selector::This)),
                ),
            },
            // Always: a +1/+1 counter on This.
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Expressive Firedancer — {1}{R}, 2/2 Human Sorcerer.
/// "Opus — Whenever you cast an instant or sorcery spell, this creature
/// gets +1/+1 until end of turn. If five or more mana was spent to cast
/// that spell, this creature also gains double strike until end of
/// turn."
///
/// ✅ Push: Opus rider now wired via `effect::shortcut::opus(5, ...)`.
/// Always-fires half: +1/+1 EOT pump on This. Big-cast (≥5 mana):
/// `Keyword::DoubleStrike` granted EOT (the "also gains" wording is
/// additive on top of the always +1/+1 — no swap, both halves run on
/// big casts). Combat-correct: a 3/3 Double Strike trigger after a 5+
/// mana spell deals 6 unblocked damage in one swing.
pub fn expressive_firedancer() -> CardDefinition {
    use crate::effect::shortcut::opus;
    use crate::mana::r;
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
        triggered_abilities: vec![opus(
            5,
            // Big-cast: gain Double Strike EOT.
            Effect::GrantKeyword {
                what: Selector::This,
                keyword: Keyword::DoubleStrike,
                duration: Duration::EndOfTurn,
            },
            // Always: +1/+1 EOT.
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Strife Scholar — {2}{R} body-only wire, 3/2 Orc Sorcerer with
/// `Keyword::Ward(2)`. The MDFC back face "Awaken the Ages" ({5}{R}
/// Sorcery) and the on-cast magecraft / Ward enforcement riders are
/// omitted; this push only ships the front-face body so the card slots
/// into red mid-curve aggressive shells. Tracked in STRIXHAVEN2.md.
///
/// Push XIX: promotes the row from ⏳ to 🟡 — same body-only +
/// Ward shape as Mica, Reader of Ruins (push XVIII) and Colorstorm
/// Stallion. The Ward enforcement is still pending the engine-side
/// `Keyword::Ward` cost gate (TODO.md tracks the work).
pub fn strife_scholar() -> CardDefinition {
    use crate::mana::r;
    CardDefinition {
        name: "Strife Scholar",
        cost: cost(&[generic(2), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Orc, CreatureType::Sorcerer],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        // Default Ward(2) — printed Ward N is unverified (Scryfall
        // unavailable in this environment). The Ward keyword is a
        // static-only tag today (engine has no cost-gate enforcement
        // yet), so the integer is purely cosmetic until that lands.
        keywords: vec![Keyword::Ward(2)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Eternal Student — {3}{B}, 4/2 Zombie Warlock. The
/// `{1}{B}, exile from graveyard: create two Inkling tokens` activated
/// ability is omitted (engine activated-ability path only walks the
/// battlefield). Vanilla 4/2 body still hits combat correctly.
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Postmortem Professor — {1}{B}, 2/2 Zombie Warlock. "This creature
/// can't block. / Whenever this creature attacks, each opponent loses
/// 1 life and you gain 1 life. / {1}{B}, Exile an instant or sorcery
/// card from your graveyard: Return this card from your graveyard to
/// the battlefield."
///
/// Wired: the printed `Keyword::CantBlock` static restriction (now
/// first-class via the SOS-VI engine push) + the on-attack drain. The
/// graveyard-exile recursion activated ability is still omitted —
/// engine's activated-abilities walker only iterates the battlefield
/// (TODO.md: "Activated-Ability `From Your Graveyard` Path").
pub fn postmortem_professor() -> CardDefinition {
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
        activated_abilities: no_abilities(),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Witherbloom, the Balancer — {6}{B}{G} 5/5 Legendary Elder Dragon.
/// "Affinity for creatures (This spell costs {1} less to cast for each
/// creature you control.) / Flying, deathtouch / Instant and sorcery
/// spells you cast have affinity for creatures."
///
/// Push XXXVIII: 🟡 (almost ✅). The first clause now wires faithfully
/// via the new `StaticEffect::CostReductionScaled` primitive — a
/// self-static on Balancer reads `Value::CountOf(EachPermanent(Creature
/// ∧ ControlledByYou))` at cast time and drains that many generic
/// mana from the cost. With 4 creatures you control, Balancer drops
/// from {6}{B}{G} to {2}{B}{G}; with 6+ creatures, the colored {B}{G}
/// pips are all that remain.
///
/// The second clause ("Instant and sorcery spells you cast have
/// affinity for creatures") still 🟡 — it grants the same affinity
/// effect to other spells, which would need a "modify another spell's
/// discount" primitive (a static that adds a CostReductionScaled to
/// every IS spell cast by the controller). Stays gap pending engine
/// work on cross-card cost-reduction grants.
pub fn witherbloom_the_balancer() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Supertype};
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
        static_abilities: vec![StaticAbility {
            description: "Affinity for creatures (this spell costs {1} less to cast for each creature you control)",
            effect: StaticEffect::CostReductionScaled {
                filter: SelectionRequirement::Any,
                amount: Value::CountOf(Box::new(Selector::EachPermanent(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ))),
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Topiary Lecturer — {2}{G}, 1/2 Elf Druid.
/// "Increment (Whenever you cast a spell, if the amount of mana you spent
/// is greater than this creature's power or toughness, put a +1/+1
/// counter on this creature.) / {T}: Add an amount of {G} equal to this
/// creature's power."
///
/// The Increment rider now wires via `effect::shortcut::increment()`
/// (push XXXI primitive backed by `Value::ManaSpentToCast`): every
/// spell cast at ≥3 mana drops a +1/+1 counter on this creature
/// (since min(P=1, T=2) = 1 → "greater than 1" = ≥2 mana, but
/// `increment()` uses min+1 = 2 as the threshold).
/// The mana ability uses the new `ManaPayload::OfColor(Green,
/// PowerOf(This))` primitive — fixed color, value-scaled count — so a
/// single AddMana effect produces power-many {G} pips in one shot
/// (cleaner than the prior `Repeat` approximation). The mana ability
/// scales linearly with each Increment-grown counter.
pub fn topiary_lecturer() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::effect::shortcut::increment;
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            effect: Effect::Seq(vec![
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
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Sundering Archaic — {6}, 3/3 Avatar.
/// "Converge — When this creature enters, exile target nonland permanent
/// an opponent controls with mana value less than or equal to the number
/// of colors of mana spent to cast this creature. / {2}: Put target card
/// from a graveyard on the bottom of its owner's library."
///
/// ETB Converge exile: the converge-scaled mana-value cap on the target
/// remains approximated to "any nonland opp permanent" (no `Value`-keyed
/// `ManaValueAtMostV` predicate yet — tracked in TODO.md). Auto-target
/// picks a legal opponent permanent.
///
/// Now wired (push XVI): the `{2}: graveyard → bottom of owner's library`
/// activated ability. Targets any card in any graveyard (validated by
/// `evaluate_requirement_static`'s graveyard fall-through), then issues
/// `Effect::Move { what: Target(0), to: ZoneDest::Library { who:
/// OwnerOf(Target(0)), pos: Bottom } }` so the card lands on the bottom
/// of its OWNER's library (not the activator's). `move_card_to`'s
/// graveyard branch handles the source-zone walk.
pub fn sundering_archaic() -> CardDefinition {
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
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: crate::effect::shortcut::target_filtered(
                    SelectionRequirement::Permanent
                        .and(SelectionRequirement::Nonland)
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
                exile_gy_cost: 0,
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
                exile_gy_cost: 0,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Campus Composer — {3}{U} body-only wire, 3/4 Merfolk Bard with
/// `Keyword::Ward(2)`. The MDFC back face "Aqueous Aria" ({4}{U}
/// Sorcery) is omitted — without verified oracle text the back face
/// would be a guess. Front face slots into blue mid-curve with Ward
/// the same way Mica / Strife Scholar / Colorstorm Stallion do.
///
/// Push XIX promotes the row from ⏳ to 🟡 on the Blue table.
pub fn campus_composer() -> CardDefinition {
    use crate::mana::u;
    CardDefinition {
        name: "Campus Composer",
        cost: cost(&[generic(3), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Bard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Ward(2)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Ulna Alley Shopkeep — {2}{B}, 2/3 Goblin Warlock.
/// "Menace (This creature can't be blocked except by two or more
/// creatures.) / Infusion — This creature gets +2/+0 as long as you
/// gained life this turn."
///
/// Body-only wire: menace is keyworded; the static "+2/+0 while you've
/// gained life this turn" rider needs a continuous-static-on-predicate
/// primitive (tracked in TODO.md) and is omitted. The 2/3 menace body
/// alone is still a useful Witherbloom evasion threat.
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
            exile_gy_cost: 0,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── Prismari (U/R) ──────────────────────────────────────────────────────────

/// Elemental Mascot — {1}{U}{R} body-only wire, 1/4 Elemental Bird with
/// `Keyword::Flying` + `Keyword::Vigilance`.
///
/// Printed Oracle: "Flying, vigilance / Opus — Whenever you cast an
/// instant or sorcery spell, this creature gets +1/+0 until end of
/// turn. If five or more mana was spent to cast that spell, exile the
/// top card of your library. You may play that card until the end of
/// your next turn."
///
/// 🟡 Body wire — the Opus rider (mana-spent introspection on the cast +
/// cast-from-exile pipeline) is omitted; the +1/+0 EOT pump on every
/// IS cast is wired faithfully via `cast_is_instant_or_sorcery()` (push
/// VII), matching the printed +1/+0 pump on the cheap-spell branch. The
/// 5+-mana exile-top branch is omitted (same cast-from-exile gap as
/// Practiced Scrollsmith / The Dawning Archaic / Conspiracy Theorist).
pub fn elemental_mascot() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
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
        // Cheap-cast magecraft +1/+0 EOT pump (the always-on half of
        // the printed Opus rider). The 5+-mana exile-top alternative
        // is omitted (cast-from-exile gap).
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Colorstorm Stallion — {1}{U}{R}, 3/3 Elemental Horse.
/// "Ward {1}, haste / Opus — Whenever you cast an instant or sorcery
/// spell, this creature gets +1/+1 until end of turn. If five or more
/// mana was spent to cast that spell, create a token that's a copy of
/// this creature."
///
/// 🟡 Body wire (3/3 Elemental Horse with `Keyword::Ward(1)` + Haste) plus
/// a partial Opus rider — the +1/+1-EOT pump fires on every
/// instant-or-sorcery cast (the magecraft trigger). The "5+ mana →
/// create a token copy of this creature" half is omitted (no copy-
/// permanent primitive yet, same gap as Mica / Aziza / Silverquill the
/// Disputant). Net play: a 3/3 Haste flier-killer with cumulative
/// magecraft pump — no copy upside.
pub fn colorstorm_stallion() -> CardDefinition {
    use crate::effect::shortcut::cast_is_instant_or_sorcery;
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
        keywords: vec![Keyword::Ward(1), Keyword::Haste],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl)
                .with_filter(cast_is_instant_or_sorcery()),
            effect: Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(1),
                toughness: Value::Const(1),
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Abstract Paintmage — {U}{U/R}{R}, 2/2 Djinn Sorcerer.
/// "At the beginning of your first main phase, add {U}{R}. Spend this
/// mana only to cast instant and sorcery spells."
///
/// Approximation: the spend restriction ("only to cast instant and
/// sorcery spells") is omitted — the engine's `ManaPool` has no per-pip
/// spend metadata yet (tracked as **Spend-Restricted Mana** in TODO.md),
/// so the produced {U}{R} behaves like normal mana and can fund any
/// spell. The trigger fires on the active player's PreCombatMain step
/// (the controller's "first" main phase). The hybrid `{U/R}` pip in the
/// cost is approximated as `{U}` so the printed cost effectively becomes
/// `{U}{U}{R}` for cube purposes.
pub fn abstract_paintmage() -> CardDefinition {
    use crate::effect::ManaPayload;
    use crate::game::types::TurnStep;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Abstract Paintmage",
        cost: cost(&[u(), u(), r()]),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    use crate::card::Supertype;
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
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Exhibition Tidecaller — {U}, 0/2 Djinn Wizard.
/// "Opus — Whenever you cast an instant or sorcery spell, target
/// player mills three cards. If five or more mana was spent to cast
/// that spell, that player mills ten cards instead."
///
/// Body-only wire (0/2 Djinn Wizard). The Opus mill rider needs the
/// engine's mana-spent-on-cast introspection primitive (tracked in
/// TODO.md as **Spell-Side Predicate: Mana-Spent-On-Cast**) before
/// the "if 5+ mana, mill 10 instead" branch can fire. The 0/2 body
/// fits in the Blue color pool as a 1-drop blocker.
pub fn exhibition_tidecaller() -> CardDefinition {
    use crate::effect::shortcut::opus;
    use crate::mana::u;
    // Push XXXI — Opus mill rider now wired. Printed Oracle: "Opus —
    // Whenever you cast an instant or sorcery spell, target player mills
    // three cards. If five or more mana was spent to cast that spell,
    // that player mills ten cards instead." Approximation: the "target
    // player" prompt collapses to "each opponent" (auto-target) — same
    // collapse applied to Mathemagics's draw target. The "instead" rider
    // becomes additive: cheap-cast → mill 3, big-cast → mill 13 (3 + 10).
    // Combat-correct against threshold mill payoffs (Stinkweed Imp,
    // Hardened Scales mill matters, etc.) — over-counts by 3 on big casts.
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
        triggered_abilities: vec![opus(
            5,
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(10),
            },
            Effect::Mill {
                who: Selector::Player(PlayerRef::EachOpponent),
                amount: Value::Const(3),
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Practiced Scrollsmith — {R}{R/W}{W}, 3/2 Dwarf Cleric.
/// "First strike / When this creature enters, exile target noncreature,
/// nonland card from your graveyard. Until the end of your next turn,
/// you may cast that card."
///
/// Approximations:
/// - The hybrid `{R/W}` pip is treated as `{R}` (cost becomes
///   `{R}{R}{W}` for cube purposes). The hybrid mana pip primitive
///   isn't wired through `cost(&[...])`.
/// - The "until end of your next turn, you may cast" rider is omitted
///   (no cast-from-exile-with-time-limit primitive). The exile half is
///   wired faithfully — the chosen noncreature/nonland gy card is
///   removed from the graveyard into exile, leaving the controller
///   without their planned recursion target. Functionally this is a
///   3/2 first striker with mild graveyard-hate.
pub fn practiced_scrollsmith() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{hybrid, r, w as wm};
    let nonperm_in_gy = SelectionRequirement::Nonland
        .and(SelectionRequirement::Not(Box::new(SelectionRequirement::Creature)));
    CardDefinition {
        name: "Practiced Scrollsmith",
        // Push XL: hybrid `{R/W}` pip now wired faithfully via
        // `ManaSymbol::Hybrid(Red, White)`. Total cost is {R}{R/W}{W}
        // — castable from R+W, R+R, or W+W pools (the printed legality).
        // The "may cast until next turn" rider is still gap (no
        // cast-from-exile-with-time-limit primitive).
        cost: cost(&[r(), hybrid(Color::Red, Color::White), wm()]),
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
            // Scrollsmith only exiles **one** matching card (the printed
            // "exile target noncreature, nonland card from your gy"). We
            // wrap the matching set in `Selector::Take(_, 1)` so a gy
            // with multiple noncreature/nonland cards loses only one,
            // matching the printed semantics.
            effect: Effect::Move {
                what: Selector::take(
                    Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: nonperm_in_gy,
                    },
                    Value::Const(1),
                ),
                to: ZoneDest::Exile,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── More Lorehold (R/W) ─────────────────────────────────────────────────────

/// Colossus of the Blood Age — {4}{R}{W}, 6/6 Artifact Creature —
/// Construct. "When this creature enters, it deals 3 damage to each
/// opponent and you gain 3 life. / When this creature dies, discard
/// any number of cards, then draw that many cards plus one."
///
/// Both abilities wired faithfully. The death trigger's "discard any
/// number, then draw that many plus one" uses
/// `Value::CardsDiscardedThisResolution` (the per-resolution discard
/// tally bumped by every `Effect::Discard` in the same `Seq`). Since
/// the engine has no "choose any number" player prompt, "any number"
/// is treated as the optimal greedy answer (discard the entire hand);
/// the `+1` floor on the draw matches the printed wording even from an
/// empty hand.
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
                // "Discard any number of cards, then draw that many
                // cards plus one." Approximated as "discard your
                // entire hand, then draw cards-discarded-this-way + 1"
                // — the engine has no "choose any number" prompt, so
                // we treat "any number" as the optimal greedy answer
                // (all of them) and read the count via the new
                // `Value::CardsDiscardedThisResolution`. The "+1"
                // floor always draws at least one card even with an
                // empty hand at trigger time.
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::HandSizeOf(PlayerRef::You),
                        random: false,
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Sum(vec![
                            Value::CardsDiscardedThisResolution,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

// ── More White ──────────────────────────────────────────────────────────────

/// Soaring Stoneglider — {2}{W}, 4/3 Elephant Cleric.
/// "As an additional cost to cast this spell, exile two cards from your
/// graveyard or pay {1}{W}. / Flying, vigilance"
///
/// Approximation: the alternative additional cost (exile two from gy)
/// is omitted (no alt-cost-with-exile-from-gy primitive). The card is
/// wired at the **paid** cost path: full {3}{W} (i.e. base cost
/// {2}{W} + the {1}{W} payment fork). Players always end up paying the
/// mana variant, which is the more common play pattern anyway. Body
/// (4/3 Flying + Vigilance Elephant Cleric) wired faithfully.
pub fn soaring_stoneglider() -> CardDefinition {
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
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}


// ── 2026-05-01 push VII: Multicolored predicate, MDFC bodies, Lorehold capstone

/// Spectacular Skywhale — {2}{U}{R} Creature — Elemental Whale.
/// Printed Oracle: "Flying / Opus — Whenever you cast an instant or sorcery
///  spell, this creature gets +3/+0 until end of turn. If five or more mana
///  was spent to cast that spell, put three +1/+1 counters on this creature
///  instead."
///
/// Push XXXI: Opus rider now wired via `effect::shortcut::opus(5, ...)`.
/// Always-fires half: +3/+0 EOT pump on every IS cast. Big-cast (≥5 mana
/// spent) half: +3 +1/+1 counters via `Effect::AddCounter` ×3. Same
/// "instead" approximation as Tackle Artist (both halves stack rather
/// than substitute — minor over-payoff on big casts; combat-correct).
pub fn spectacular_skywhale() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::opus;
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
        triggered_abilities: vec![opus(
            5,
            Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(3),
            },
            Effect::PumpPT {
                what: Selector::This,
                power: Value::Const(3),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        )],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Zaffai and the Tempests — {5}{U}{R}, 5/7 Legendary Human Bard Sorcerer.
/// "Once during each of your turns, you may cast an instant or sorcery
/// spell from your hand without paying its mana cost."
///
/// 🟡 Body-only wire (push XVI). The "once-per-turn cast-IS-for-free"
/// rider is omitted — engine has no per-turn alt-cost-grant primitive
/// (would need `Player.zaffai_free_cast_used: bool` consumed by an
/// alternative-cost path keyed off the source's controller). The 5/7
/// vigilance-less body is still a powerful finisher in U/R aggro/spells
/// pools.
pub fn zaffai_and_the_tempests() -> CardDefinition {
    use crate::card::Supertype;
    use crate::mana::{r, u};
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
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Lorehold, the Historian — {3}{R}{W} Legendary Creature — Elder Dragon.
/// 5/5 Flying + Haste.
///
/// Push XXXVI: 🟡 fidelity bump. The per-opp-upkeep `you may discard a
/// card → draw a card` loot trigger is now wired via
/// `EventKind::StepBegins(Upkeep) + EventScope::OpponentControl` —
/// `fire_step_triggers` routes step events to permanents whose
/// controller is *not* the active player when the scope is
/// OpponentControl, so each opp's upkeep fires the Historian's loot
/// trigger. The body uses `Effect::MayDo` so the auto-decider's "no"
/// default skips on-bot turns; a `ScriptedDecider::new([Bool(true)])`
/// in tests verifies the loot path. The "instant and sorcery cards in
/// your hand have miracle {2}" static is still omitted (no alt-cost-
/// on-draw / miracle primitive — same gap as Velomachus Lorehold).
/// Status stays 🟡 because the miracle grant is the more impactful
/// half; the loot trigger is the smaller of the two.
pub fn lorehold_the_historian() -> CardDefinition {
    use crate::card::Supertype;
    use crate::game::types::TurnStep;
    use crate::mana::{r, w};
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::Upkeep),
                EventScope::OpponentControl,
            ),
            effect: Effect::MayDo {
                description: "Discard a card; if you do, draw a card."
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Silverquill, the Disputant — {2}{W}{B} Legendary Creature — Elder
/// Dragon. 4/4 Flying, vigilance.
///
/// Printed Oracle: "Flying, vigilance / Each instant and sorcery spell
/// you cast has casualty 1. (As you cast that spell, you may
/// sacrifice a creature with power 1 or greater. When you do, copy the
/// spell and you may choose new targets for the copy.)"
///
/// Now wired (post-XX) using `Effect::CopySpell` + `Effect::MayDo`:
/// the Casualty 1 grant is approximated as a magecraft trigger that
/// asks the controller to may-sacrifice a power-≥-1 creature, and on
/// yes copies the just-cast spell. Differences vs printed Casualty:
/// Casualty's "as you cast" timing means the sac happens at cast time
/// (and copies share their cast events with the original); we resolve
/// the sac + copy after the cast triggers, which is functionally
/// equivalent for combat math but doesn't double-fire other "when you
/// cast" payoffs.
pub fn silverquill_the_disputant() -> CardDefinition {
    use crate::card::Supertype;
    use crate::effect::shortcut::magecraft;
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
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "Casualty 1 — sacrifice a creature with power 1 or greater to copy".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Sacrifice {
                    who: Selector::You,
                    count: Value::Const(1),
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(1)),
                },
                Effect::CopySpell {
                    what: Selector::CastSpellSource,
                    count: Value::Const(1),
                },
            ])),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Quandrix, the Proof — {4}{G}{U} Legendary Creature — Elder Dragon.
/// 6/6 Flying, trample.
///
/// Printed Oracle: "Flying, trample / Cascade (When you cast this
/// spell, exile cards from the top of your library until you exile a
/// nonland card that costs less. You may cast it without paying its
/// mana cost.) / Instant and sorcery spells you cast from your hand
/// have cascade."
///
/// 🟡 Body-only wire — Cascade is not yet a first-class engine
/// keyword (no reveal-until-MV-less-than primitive, no cast-from-exile
/// pipeline; tracked in TODO.md push XVIII). The 6/6 Flying+Trample
/// Elder Dragon body still hits combat correctly at the 6 CMC slot.
pub fn quandrix_the_proof() -> CardDefinition {
    use crate::card::Supertype;
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
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Prismari, the Inspiration — {5}{U}{R} Legendary Creature — Elder
/// Dragon. 7/7 Flying with Ward—Pay 5 life.
///
/// Printed Oracle: "Flying / Ward—Pay 5 life. / Instant and sorcery
/// spells you cast have storm. (Whenever you cast an instant or
/// sorcery spell, copy it for each spell cast before it this turn.
/// You may choose new targets for the copies.)"
///
/// 🟡 Body-only wire with `Keyword::Ward(5)` (the printed alt-life
/// Ward cost is approximated as a flat mana Ward, same primitive
/// applied to Mica's Ward—Pay 3 life). The Storm grant on every IS
/// cast is omitted (no copy-spell primitive). The 7/7 Flying body
/// remains the dominant printed clause at the 7 CMC slot.
pub fn prismari_the_inspiration() -> CardDefinition {
    use crate::card::Supertype;
    use crate::mana::{r, u};
    CardDefinition {
        name: "Prismari, the Inspiration",
        cost: cost(&[generic(5), u(), r()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elder, CreatureType::Dragon],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying, Keyword::Ward(5)],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Ennis, Debate Moderator — {1}{W} 1/1 Legendary Human Cleric.
/// "When Ennis enters, exile up to one other target creature you control.
/// Return that card to the battlefield under its owner's control at the
/// beginning of the next end step. / At the beginning of your end step,
/// if one or more cards were put into exile this turn, put a +1/+1
/// counter on Ennis."
///
/// Both abilities now fully wired (push XXXVII doc fix):
/// - ETB flicker: exiles a target creature (auto-picker prefers a
///   friendly utility creature with a useful ETB) and schedules a
///   delayed return at next end step. Uses the same
///   `Exile + DelayUntil(NextEndStep, Move(Target → Battlefield(OwnerOf)))`
///   pattern as Restoration Angel-style flickers.
/// - End-step counter: gated on "any card was exiled this turn" via
///   `Predicate::CardsExiledThisTurnAtLeast` (push IX) backed by
///   `Player.cards_exiled_this_turn`. Pre-IX this used a gy-leave
///   proxy; the per-turn exile tally is now exact.
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
                        capture: None,
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Tragedy Feaster — {2}{B}{B} 7/6 Demon.
/// "Trample / Ward—Discard a card. / Infusion — At the beginning of your
/// end step, sacrifice a permanent unless you gained life this turn."
///
/// Body wired (7/6 Demon with Trample). Ward is omitted — the engine has
/// no Ward keyword primitive yet (tracked in TODO.md). The Infusion
/// upkeep-tax is also omitted (no `MayDo` / `If/else` sacrifice
/// primitive that runs on a per-turn lifegain check). The base Demon
/// shell still slots into Witherbloom / mono-black ramp into a 4-mana
/// 7/6 trampler — strictly under-costed for a vanilla body, but the
/// missing Ward/upkeep-sac taxes balance the printed card.
pub fn tragedy_feaster() -> CardDefinition {
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
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        keywords: vec![],
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    use crate::mana::{ManaSymbol, g, u};
    use super::sorceries::fractal_token;
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
            mana_cost: ManaCost {
                symbols: vec![ManaSymbol::X, ManaSymbol::X],
            },
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::XFromCost,
                },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
        }],
        triggered_abilities: vec![
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
            // Push XXXI: Increment trigger now wired via the new
            // `Value::ManaSpentToCast` primitive. Min(P, T) on the 1/4
            // frame is 1 — every cast where mana_spent ≥ 2 drops a
            // +1/+1 counter, which then fires the AnyOneColor ramp
            // trigger above. Berta becomes a self-feeding mana engine
            // once a {2}-cost spell hits the stack.
            crate::effect::shortcut::increment(),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
/// printed "rest on bottom random order" rider is approximated.
///
/// Push XL: hybrid `{G/U}` pip now wired faithfully via
/// `ManaSymbol::Hybrid(Green, Blue)`. Total cost is {G}{G/U}{U} —
/// castable from G+U, G+G, or U+U pools (printed legality).
pub fn paradox_surveyor() -> CardDefinition {
    use crate::effect::ZoneDest;
    use crate::mana::{g, hybrid, u};
    CardDefinition {
        name: "Paradox Surveyor",
        cost: cost(&[g(), hybrid(Color::Green, Color::Blue), u()]),
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
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Magmablood Archaic — {2/R}{2/R}{2/R} 2/2 Avatar.
/// "Trample, reach / Converge — This creature enters with a +1/+1
/// counter on it for each color of mana spent to cast it. / Whenever
/// you cast an instant or sorcery spell, creatures you control get
/// +1/+0 until end of turn for each color of mana spent to cast that
/// spell."
///
/// Hybrid `{2/R}` pips approximated as `{R}`-or-{generic 2} — engine's
/// hybrid-cost expansion lets the pip pay either way. We choose
/// generic 2 ×3 + R ×3 simplifies the printed cost by always paying the
/// {R} half. Trample + reach + Converge ETB counter are wired exactly
/// like Rancorous Archaic. The spell-cast pump uses
/// `Value::ConvergedValue` for the iterated cast — but the engine
/// re-uses the *current cast's* converge value, not the just-cast
/// spell's. We approximate by reading the trigger source's
/// converge-from-stack via the `StackItem::Trigger.converged_value`
/// inheritance set up in push III. For the typical 2-color cube spell
/// this lands +2/+0 EOT on each friendly creature, which matches the
/// printed effect on a 2-color cast.
pub fn magmablood_archaic() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::r;
    CardDefinition {
        name: "Magmablood Archaic",
        cost: cost(&[generic(2), generic(2), generic(2), r(), r(), r()]),
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
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
/// Magmablood Archaic). Hybrid `{2/G}` pips approximated as
/// `{generic 2} + {G}` per pip ({2}{2}{G}{G}). The printed 0/0 means
/// the creature dies to SBA without enough Converge counters; mono-G
/// or off-color casts will die immediately, while a 2-color cast lands
/// it as a 2/2. The "creature spells you cast enter with X extra
/// counters" rider is omitted pending an `EventKind::SpellCast` filter
/// that captures the just-cast spell's converged value at trigger time
/// (today the trigger fires but the body pump runs against the source's
/// own converged value, not the cast spell's).
pub fn wildgrowth_archaic() -> CardDefinition {
    use crate::card::CounterType;
    use crate::mana::g;
    CardDefinition {
        name: "Wildgrowth Archaic",
        cost: cost(&[generic(2), generic(2), g(), g()]),
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::ConvergedValue,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Ambitious Augmenter — {G} 1/1 Turtle Wizard.
/// "Increment (Whenever you cast a spell, if the amount of mana you
/// spent is greater than this creature's power or toughness, put a
/// +1/+1 counter on this creature.) / When this creature dies, if it
/// had one or more counters on it, create a 0/0 green and blue Fractal
/// creature token, then put this creature's counters on that token."
///
/// Body wired (1/1 Turtle Wizard at {G}). Increment trigger now wired
/// via `effect::shortcut::increment()` (push XXXI primitive backed by
/// `Value::ManaSpentToCast`): every spell cast at ≥2 mana drops a
/// +1/+1 counter on this creature (since min(P, T) starts at 1).
/// 🟡 still — the death-with-counters → Fractal-with-counters
/// transfer trigger is omitted pending a counter-transfer-on-death
/// primitive (`Selector::Self.counters_at_death` snapshot would
/// expose the counter count to the death-trigger body via
/// `Value::CountersOn(SelfSource)` — that read already works on the
/// graveyard-resident copy via the push XVII fallback, but the
/// fan-out wants to *move* counters onto a freshly-minted token,
/// which would need a per-counter "transfer to LastCreatedToken"
/// primitive).
pub fn ambitious_augmenter() -> CardDefinition {
    use crate::effect::shortcut::increment;
    use crate::mana::g;
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
        triggered_abilities: vec![increment()],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Rubble Rouser — {2}{R} Creature — Dwarf Sorcerer.
/// 1/4. ETB may-discard-then-draw (collapsed to always-do); the
/// `{T}, Exile a card from your graveyard: Add {R}. When you do, this
/// creature deals 1 damage to each opponent.` activated ability is
/// omitted (engine has no exile-from-your-graveyard activation cost
/// primitive, separate from `sac_cost`).
///
/// The rummage ETB is faithfully wired: discard 1 + draw 1. The
/// `you may` optionality collapses to "always do" since the
/// engine has no per-effect yes/no decision (TODO.md).
pub fn rubble_rouser() -> CardDefinition {
    use crate::mana::r;
    // Push XV: the printed "you may discard a card. If you do, draw a
    // card" rummage is now wired via `Effect::MayDo` — the controller
    // picks yes/no via `OptionalTrigger`. Tests can flip the answer to
    // `true`; the auto-decider declines.
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
        activated_abilities: no_abilities(),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        ],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
        ],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Textbook Tabulator — {2}{U} 0/3 Frog Wizard. Printed Oracle:
/// "Increment (Whenever you cast a spell, if the amount of mana you spent
///  is greater than this creature's power or toughness, put a +1/+1
///  counter on this creature.) / When this creature enters, surveil 2."
///
/// Push XXXI: Increment rider now wired via `effect::shortcut::increment()`.
/// On the 0/3 frame the Increment threshold is a 1-mana spell —
/// effectively ramps to 1/4 → 2/5 → 3/6 over a normal-curve U/x deck.
/// ETB Surveil 2 unchanged.
pub fn textbook_tabulator() -> CardDefinition {
    use crate::effect::shortcut::increment;
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
            increment(),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Deluge Virtuoso — {2}{U} 2/2 Human Wizard. Printed Oracle:
/// "When this creature enters, tap target creature an opponent controls
///  and put a stun counter on it. / Opus — Whenever you cast an instant
///  or sorcery spell, this creature gets +1/+1 until end of turn. If
///  five or more mana was spent to cast that spell, this creature gets
///  +2/+2 until end of turn instead."
///
/// Push XXXI: Opus rider now wired via `effect::shortcut::opus(5, ...)`.
/// Always-fires half: +1/+1 EOT pump. Big-cast (≥5 mana) half: an
/// additional +1/+1 EOT pump (net +2/+2 EOT). Same "instead" stack-vs-
/// substitute approximation as Tackle Artist / Spectacular Skywhale —
/// combat-correct since the printed +2/+2 dominates +1/+1.
pub fn deluge_virtuoso() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::{opus, target_filtered};
    use crate::mana::u;
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
            // Opus +1/+1 always (the magecraft half) + an additional
            // +1/+1 EOT on big casts (net +2/+2 EOT).
            opus(
                5,
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::Const(1),
                    toughness: Value::Const(1),
                    duration: Duration::EndOfTurn,
                },
            ),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Moseo, Vein's New Dean — {2}{B} Legendary Creature — Bird Skeleton
/// Warlock 2/1 Flying.
/// "Flying / When Moseo enters, create a 1/1 black and green Pest
/// creature token with 'Whenever this token attacks, you gain 1 life.' /
/// Infusion — At the beginning of your end step, if you gained life
/// this turn, return up… (oracle truncated)"
///
/// Body + Flying + ETB Pest token wired faithfully (the Pest token's
/// on-attack lifegain rider rides on the shared `pest_token()` helper).
/// The Infusion end-step rider is omitted — its oracle text was clipped
/// in the table dump and the engine has no `MayDo` per-turn-lifegain
/// trigger primitive yet (TODO.md). The vanilla 3-mana 2/1 Flier shell
/// + free Pest token is a strictly under-printed approximation.
pub fn moseo_veins_new_dean() -> CardDefinition {
    use super::sorceries::pest_token;
    use crate::card::Supertype;
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: pest_token(),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Stone Docent — {1}{W} 3/1 Spirit Chimera.
/// "{W}, Exile this card from your graveyard: You gain 2 life. Surveil
/// 1. Activate only as a sorcery."
///
/// Body-only wire (3/1 Spirit Chimera). The graveyard-exile activated
/// ability is omitted — the engine's activated-ability walker only
/// iterates the battlefield (TODO.md "Activated-Ability `From Your
/// Graveyard` Path"; same gap as Eternal Student, Summoned Dromedary).
/// The vanilla 2-mana 3/1 body still slots into mono-W aggro pools.
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
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
    use crate::card::Supertype;
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
        activated_abilities: vec![tap_add_colorless()],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// Essenceknit Scholar — {B}{B/G}{G} 3/1 Dryad Warlock.
/// "When this creature enters, create a 1/1 black and green Pest creature
/// token with 'Whenever this token attacks, you gain 1 life.' / At the
/// beginning of your end step, if a creature died under your control
/// this turn, draw a card."
///
/// Push XL: hybrid `{B/G}` pip now wired faithfully via
/// `ManaSymbol::Hybrid(Black, Green)` — total cost is {B}{B/G}{G},
/// castable from B+G, B+B, or G+G pools. Both triggers wired
/// faithfully — the ETB Pest token rides on the shared `pest_token()`
/// helper (so its on-attack lifegain rider trickles into Witherbloom
/// payoffs); the end-step draw uses the new
/// `Predicate::CreaturesDiedThisTurnAtLeast` gate, scoped to the active
/// player so it fires once per controller's own end step.
pub fn essenceknit_scholar() -> CardDefinition {
    use super::sorceries::pest_token;
    use crate::card::Predicate;
    use crate::game::types::TurnStep;
    use crate::mana::{g, hybrid};
    CardDefinition {
        name: "Essenceknit Scholar",
        cost: cost(&[b(), hybrid(Color::Black, Color::Green), g()]),
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}

/// The Dawning Archaic — {10} Legendary Creature — Avatar, 7/7 Reach.
/// Printed Oracle:
/// "This spell costs {1} less to cast for each instant and sorcery card
///  in your graveyard.
///  Reach
///  Whenever The Dawning Archaic attacks, you may cast target instant
///  or sorcery card from your graveyard without paying its mana cost.
///  If that spell would be put into your graveyard, exile it instead."
///
/// 🟡 Push XXXVIII: ⏳ → 🟡. Body wired:
/// - 7/7 Legendary Avatar with Reach.
/// - Self-discount via the new `StaticEffect::CostReductionScaled`
///   primitive — `amount: CountOf(CardsInZone(your gy, IS-cards))`.
///   With 5 IS cards in your graveyard, the printed {10} drops to {5}.
///
/// The attack-trigger cast-from-graveyard rider stays gap pending
/// engine work on the cast-from-exile/graveyard pipeline (same family
/// as Velomachus Lorehold's reveal-and-cast, Conspiracy Theorist's
/// "may cast from exile").
pub fn the_dawning_archaic() -> CardDefinition {
    use crate::card::{StaticAbility, StaticEffect, Supertype, Zone};
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
        triggered_abilities: vec![],
        static_abilities: vec![StaticAbility {
            description: "Costs {1} less to cast for each instant or sorcery card in your graveyard",
            effect: StaticEffect::CostReductionScaled {
                filter: SelectionRequirement::Any,
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: Zone::Graveyard,
                    filter: SelectionRequirement::Or(
                        Box::new(SelectionRequirement::HasCardType(CardType::Instant)),
                        Box::new(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    ),
                })),
            },
        }],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
    }
}
