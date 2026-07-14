//! Foundations (FDN) gap batch 5 — reprint staples spanning existing
//! primitives: french-vanilla combat keywords, protection-from-everything,
//! ETB tutors, a non-flyer sweep, token/counter doubling, threaten, hand
//! disruption, and a kicker land-fetch. Tests in `tests/recent206.rs`.

use crate::card::{
    CardDefinition, CardType, CreatureType, Effect, Keyword, Predicate,
    SelectionRequirement as R, Selector, StaticAbility, Subtypes, Supertype, TriggeredAbility,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, EventKind, EventScope, EventSpec, PlayerRef, StaticEffect, Value, ZoneDest};
use crate::mana::{b, cost, g, generic, r, u, w};

/// Swiftblade Vindicator — {R}{W} 1/1. Double strike, vigilance, trample.
pub fn swiftblade_vindicator() -> CardDefinition {
    CardDefinition {
        name: "Swiftblade Vindicator",
        cost: cost(&[r(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::DoubleStrike, Keyword::Vigilance, Keyword::Trample],
        ..Default::default()
    }
}

/// Progenitus — {W}{W}{U}{U}{B}{B}{R}{R}{G}{G} 10/10. Protection from
/// everything; shuffles into its owner's library instead of dying.
pub fn progenitus() -> CardDefinition {
    CardDefinition {
        name: "Progenitus",
        cost: cost(&[w(), w(), u(), u(), b(), b(), r(), r(), g(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Hydra, CreatureType::Avatar],
            ..Default::default()
        },
        power: 10,
        toughness: 10,
        keywords: vec![Keyword::ProtectionFromEverything],
        shuffles_into_library_instead: true,
        ..Default::default()
    }
}

/// Rune-Scarred Demon — {5}{B}{B} 6/6. Flying; when it enters, search your
/// library for a card and put it into your hand.
pub fn rune_scarred_demon() -> CardDefinition {
    CardDefinition {
        name: "Rune-Scarred Demon",
        cost: cost(&[generic(5), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::Any,
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Micromancer — {3}{U} 3/3. When it enters, you may search your library for an
/// instant or sorcery with mana value 1 and put it into your hand.
pub fn micromancer() -> CardDefinition {
    CardDefinition {
        name: "Micromancer",
        cost: cost(&[generic(3), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: R::HasCardType(CardType::Instant)
                    .or(R::HasCardType(CardType::Sorcery))
                    .and(R::ManaValueAtMost(1)),
                to: ZoneDest::Hand(PlayerRef::You),
            },
        }],
        ..Default::default()
    }
}

/// Seismic Rupture — {2}{R} Sorcery. Deal 2 damage to each creature without
/// flying.
pub fn seismic_rupture() -> CardDefinition {
    CardDefinition {
        name: "Seismic Rupture",
        cost: cost(&[generic(2), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(
                R::Creature.and(R::HasKeyword(Keyword::Flying).negate()),
            ),
            body: Box::new(Effect::DealDamage {
                to: Selector::TriggerSource,
                amount: Value::Const(2),
            }),
        },
        ..Default::default()
    }
}

/// An Offer You Can't Refuse — {U} Instant. Counter target noncreature spell.
/// Its controller creates two Treasure tokens.
pub fn an_offer_you_cant_refuse() -> CardDefinition {
    CardDefinition {
        name: "An Offer You Can't Refuse",
        cost: cost(&[u()]),
        card_types: vec![CardType::Instant],
        // Mint the Treasures for the spell's controller while it's still on the
        // stack, then counter it.
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::ControllerOf(Box::new(Selector::Target(0))),
                count: Value::Const(2),
                definition: crabomination_base::tokens::treasure_token(),
            },
            Effect::CounterSpell {
                what: target_filtered(R::IsSpellOnStack.and(R::Noncreature)),
            },
        ]),
        ..Default::default()
    }
}

/// Involuntary Employment — {3}{R} Sorcery. Gain control of target creature
/// until end of turn, untap it, it gains haste, and create a Treasure token.
pub fn involuntary_employment() -> CardDefinition {
    CardDefinition {
        name: "Involuntary Employment",
        cost: cost(&[generic(3), r()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::Seq(vec![
            Effect::GainControl {
                what: target_filtered(R::Creature),
                to: None,
                duration: Duration::EndOfTurn,
            },
            Effect::Untap { what: Selector::Target(0), up_to: None },
            Effect::GrantKeyword {
                what: Selector::Target(0),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: crabomination_base::tokens::treasure_token(),
            },
        ]),
        ..Default::default()
    }
}

/// Pilfer — {1}{B} Sorcery. Target opponent reveals their hand; you choose a
/// nonland card from it; that player discards it.
pub fn pilfer() -> CardDefinition {
    CardDefinition {
        name: "Pilfer",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Sorcery],
        effect: Effect::DiscardChosen {
            from: Selector::Player(PlayerRef::EachOpponent),
            count: Value::Const(1),
            filter: R::Nonland,
        },
        ..Default::default()
    }
}

/// Grow from the Ashes — {2}{G} Sorcery. Kicker {2}. Search your library for a
/// basic land and put it onto the battlefield; two if kicked.
pub fn grow_from_the_ashes() -> CardDefinition {
    let fetch = || Effect::Search {
        who: PlayerRef::You,
        filter: R::IsBasicLand,
        to: ZoneDest::Battlefield { controller: PlayerRef::You, tapped: false },
    };
    CardDefinition {
        name: "Grow from the Ashes",
        cost: cost(&[generic(2), g()]),
        card_types: vec![CardType::Sorcery],
        keywords: vec![Keyword::Kicker(cost(&[generic(2)]))],
        effect: Effect::If {
            cond: Predicate::SpellWasKicked,
            then: Box::new(Effect::Seq(vec![fetch(), fetch()])),
            else_: Box::new(fetch()),
        },
        ..Default::default()
    }
}

/// Doubling Season — {4}{G} Enchantment. Doubles tokens created and counters
/// placed under your control.
pub fn doubling_season() -> CardDefinition {
    CardDefinition {
        name: "Doubling Season",
        cost: cost(&[generic(4), g()]),
        card_types: vec![CardType::Enchantment],
        static_abilities: vec![
            StaticAbility {
                description: "If an effect would create one or more tokens under \
                              your control, it creates twice that many instead.",
                effect: StaticEffect::DoubleTokens,
            },
            StaticAbility {
                description: "If an effect would put one or more counters on a \
                              permanent you control, it puts twice that many instead.",
                effect: StaticEffect::DoubleCounters,
            },
        ],
        ..Default::default()
    }
}
