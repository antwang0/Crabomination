//! Dissension (DIS) gap wave 6. Tests in `classic_sets/dis`.

use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CreatureType, EventKind, EventScope, EventSpec,
    Keyword, Predicate, SelectionRequirement as R, Subtypes, Supertype, TriggeredAbility, Value,
};
use crate::effect::{Effect, LibraryPosition, PlayerRef, Selector, ZoneDest};
use crate::game::types::TurnStep;
use crate::mana::{Color, b, cost, g, generic, hybrid, r, u, w};

/// Momir Vig, Simic Visionary — {3}{G}{U} 2/2 Elf Wizard. Casting a green
/// creature spell tutors a creature to the top of your library; casting a blue
/// creature spell reveals the top card and takes it if it's a creature.
pub fn momir_vig_simic_visionary() -> CardDefinition {
    CardDefinition {
        name: "Momir Vig, Simic Visionary",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::Creature.and(R::HasColor(Color::Green))),
                ),
                effect: Effect::Search {
                    who: PlayerRef::You,
                    filter: R::Creature,
                    to: ZoneDest::Library {
                        who: PlayerRef::You,
                        pos: LibraryPosition::Top,
                    },
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::SpellCast, EventScope::YourControl).with_filter(
                    Predicate::CastSpellMatches(R::Creature.and(R::HasColor(Color::Blue))),
                ),
                effect: Effect::RevealTopTakeMatchingToHand {
                    who: PlayerRef::You,
                    count: Value::ONE,
                    filter: R::Creature,
                    distinct_powers: false,
                },
            },
        ],
        ..Default::default()
    }
}

/// Sphinx of the Chimes — {4}{U}{U} 5/6 Sphinx. Flying; discard two nonland
/// cards with the same name to draw four cards.
pub fn sphinx_of_the_chimes() -> CardDefinition {
    CardDefinition {
        name: "Sphinx of the Chimes",
        cost: cost(&[generic(4), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        activated_abilities: vec![ActivatedAbility {
            discard_cost: Some((R::Nonland, 2)),
            discard_cost_same_name: true,
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Elemental Resonance — {2}{G}{G} Aura. Enchant permanent; at the beginning of
/// your first main phase, add mana equal to the enchanted permanent's cost.
pub fn elemental_resonance() -> CardDefinition {
    use crate::card::EnchantmentSubtype;
    CardDefinition {
        name: "Elemental Resonance",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Enchantment],
        subtypes: Subtypes {
            enchantment_subtypes: vec![EnchantmentSubtype::Aura],
            ..Default::default()
        },
        effect: Effect::Attach {
            what: Selector::This,
            to: Selector::TargetFiltered {
                slot: 0,
                filter: R::Permanent,
            },
        },
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::StepBegins(TurnStep::PreCombatMain),
                EventScope::YourControl,
            ),
            effect: Effect::AddManaEqualToPermanentCost {
                permanent: Selector::AttachedTo(Box::new(Selector::This)),
            },
        }],
        ..Default::default()
    }
}

/// Vigean Intuition — {3}{G}{U} Instant. Choose a card type, then reveal the
/// top four cards of your library; put those of the chosen type into your hand
/// and the rest into your graveyard.
pub fn vigean_intuition() -> CardDefinition {
    CardDefinition {
        name: "Vigean Intuition",
        cost: cost(&[generic(3), g(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::ChooseTypeRevealTopPartition {
            count: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Fertile Imagination — {2}{G}{G} Sorcery. Choose a card type; target opponent
/// reveals their hand; create two 1/1 green Saproling tokens for each card of
/// the chosen type revealed this way.
pub fn fertile_imagination() -> CardDefinition {
    CardDefinition {
        name: "Fertile Imagination",
        cost: cost(&[generic(2), g(), g()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::FertileImagination {
            per: Value::Const(2),
        },
        ..Default::default()
    }
}

/// Aethermage's Touch — {2}{W}{U} Instant. Reveal the top four cards of your
/// library; put a creature from among them onto the battlefield with "return
/// it to your hand at your end step," then bottom the rest.
pub fn aethermages_touch() -> CardDefinition {
    CardDefinition {
        name: "Aethermage's Touch",
        cost: cost(&[generic(2), w(), u()]),
        card_types: vec![CardType::Instant],
        effect: Effect::AethermagesTouch {
            count: Value::Const(4),
        },
        ..Default::default()
    }
}

/// Infernal Tutor — {1}{B} Sorcery. Reveal a card from your hand and search for
/// a card with the same name; Hellbent (empty hand) instead tutors any card.
pub fn infernal_tutor() -> CardDefinition {
    CardDefinition {
        name: "Infernal Tutor",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::InfernalTutor,
        ..Default::default()
    }
}

/// Ignorant Bliss — {1}{R} Instant. Exile your hand face down; at the next end
/// step return those cards to your hand, then draw a card.
pub fn ignorant_bliss() -> CardDefinition {
    CardDefinition {
        name: "Ignorant Bliss",
        cost: cost(&[generic(1), r()]),
        card_types: vec![CardType::Instant],
        effect: Effect::IgnorantBliss,
        ..Default::default()
    }
}

/// Dovescape — {3}{W/U}{W/U}{W/U} Enchantment. Whenever a player casts a
/// noncreature spell, counter it; that player makes a 1/1 white-and-blue flying
/// Bird per point of the spell's mana value.
pub fn dovescape() -> CardDefinition {
    let wu = || hybrid(Color::White, Color::Blue);
    CardDefinition {
        name: "Dovescape",
        cost: cost(&[generic(3), wu(), wu(), wu()]),
        card_types: vec![CardType::Enchantment],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::AnyPlayer)
                .with_filter(Predicate::CastSpellMatches(R::Noncreature)),
            effect: Effect::Dovescape,
        }],
        ..Default::default()
    }
}

/// Muse Vessel — {4} Artifact. {3}, {T} (sorcery speed): target player exiles a
/// card from their hand under this. {1}: choose a card exiled with Muse Vessel;
/// you may play it this turn.
pub fn muse_vessel() -> CardDefinition {
    use crate::card::{ActivatedAbility, MayPlayDuration};
    CardDefinition {
        name: "Muse Vessel",
        cost: cost(&[generic(4)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![
            ActivatedAbility {
                tap_cost: true,
                mana_cost: cost(&[generic(3)]),
                sorcery_speed: true,
                effect: Effect::ExileChosenFromHand {
                    from: Selector::Player(PlayerRef::Target(0)),
                    count: Value::ONE,
                    filter: R::Any,
                    link_to_source: true,
                    face_down: false,
                },
                ..Default::default()
            },
            ActivatedAbility {
                mana_cost: cost(&[generic(1)]),
                effect: Effect::GrantMayPlay {
                    what: Selector::one_of(Selector::CardExiledWithSource),
                    duration: MayPlayDuration::EndOfThisTurn,
                    to_owner: false,
                    exile_after: false,
                    pay_own_cost: true,
                    any_color: false,
                },
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Isperia the Inscrutable — {1}{W}{W}{U}{U} 3/6 Sphinx. Flying; combat damage
/// to a player lets you name a card — if they reveal it, tutor a flyer.
pub fn isperia_the_inscrutable() -> CardDefinition {
    CardDefinition {
        name: "Isperia the Inscrutable",
        cost: cost(&[generic(1), w(), w(), u(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 3,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::IsperiaReveal,
        }],
        ..Default::default()
    }
}

/// Simic Basilisk — {4}{G}{G} 0/0 Basilisk Mutant. Graft 3; {1}{G}: until end
/// of turn, target creature with a +1/+1 counter gains deathtouch (modeling
/// "destroy the creature it damages at end of combat").
pub fn simic_basilisk() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::Duration;
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Simic Basilisk",
        cost: cost(&[generic(4), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Basilisk, CreatureType::Mutant],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        triggered_abilities: vec![crate::effect::shortcut::graft()],
        activated_abilities: vec![ActivatedAbility {
            mana_cost: cost(&[generic(1), g()]),
            effect: Effect::GrantKeyword {
                what: target_filtered(R::Creature.and(R::WithCounter(CounterType::PlusOnePlusOne))),
                keyword: Keyword::Deathtouch,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Evolution Vat — {3} Artifact. {3}, {T}: tap target creature and put a +1/+1
/// counter on it; it gains "{2}{G}{U}: double its +1/+1 counters" until end
/// of turn.
pub fn evolution_vat() -> CardDefinition {
    use crate::card::{ActivatedAbility, CounterType};
    use crate::effect::shortcut::target_filtered;
    CardDefinition {
        name: "Evolution Vat",
        cost: cost(&[generic(3)]),
        card_types: vec![CardType::Artifact],
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(3)]),
            effect: Effect::Seq(vec![
                Effect::Tap {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature,
                    },
                },
                Effect::AddCounter {
                    what: Selector::TargetFiltered {
                        slot: 0,
                        filter: R::Creature,
                    },
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::ONE,
                },
                Effect::GainActivatedAbility {
                    what: target_filtered(R::Creature),
                    ability: Box::new(ActivatedAbility {
                        mana_cost: cost(&[generic(2), g(), u()]),
                        effect: Effect::DoubleCountersOnEach {
                            what: Selector::This,
                            kind: CounterType::PlusOnePlusOne,
                        },
                        ..Default::default()
                    }),
                    duration: crate::effect::Duration::EndOfTurn,
                },
            ]),
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Kindle the Carnage — {1}{R}{R} Sorcery. Discard a card at random; if you do,
/// deal damage equal to its mana value to each creature. You may repeat any
/// number of times.
pub fn kindle_the_carnage() -> CardDefinition {
    CardDefinition {
        name: "Kindle the Carnage",
        cost: cost(&[generic(1), r(), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::KindleTheCarnage,
        ..Default::default()
    }
}
