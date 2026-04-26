//! Creatures used by the BRG and Goryo's demo decks.
//!
//! Many of these have triggered/static abilities or alternative costs the
//! engine doesn't support yet — those abilities are stubbed with `Effect::Noop`
//! and a doc-comment marking the omission. The bodies (cost, P/T, keywords,
//! creature subtypes) are correct so they animate and combat properly.

use super::super::no_abilities;
use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, PlaneswalkerSubtype, Selector, SelectionRequirement, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

// ── BRG creatures ────────────────────────────────────────────────────────────

/// Callous Sell-Sword — {3}{R}, 4/4 Human Mercenary. Casualty 2: copy with
/// modal-cast on a sacrificed creature. Stub: vanilla 4/4 (casualty omitted).
/// TODO: wire casualty mechanic.
pub fn callous_sell_sword() -> CardDefinition {
    CardDefinition {
        name: "Callous Sell-Sword",
        cost: cost(&[generic(3), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Chancellor of the Tangle — {5}{G}, 6/7 Avatar Incarnation. "You may reveal
/// this card from your opening hand. If you do, at the beginning of your
/// first main phase, add {G}." Stub: vanilla 6/7 (opening-hand mana omitted).
/// TODO: opening-hand reveal trigger that grants {G} on turn 1 main.
pub fn chancellor_of_the_tangle() -> CardDefinition {
    CardDefinition {
        name: "Chancellor of the Tangle",
        cost: cost(&[generic(5), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 6,
        toughness: 7,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Cosmogoyf — {2}{G}, *X/X+1* where X = number of different card types
/// among cards in all graveyards. Stub: 4/5 vanilla — actual variable P/T
/// requires graveyard inspection in the layer system.
/// TODO: wire dynamic P/T like Tarmogoyf.
pub fn cosmogoyf() -> CardDefinition {
    CardDefinition {
        name: "Cosmogoyf",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 5,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Devourer of Destiny — {5}, 7/5 colorless Eldrazi. Real Oracle has a
/// "scry 2 when you cast this" trigger; we approximate as ETB Scry 2 (the
/// engine doesn't track on-cast triggers from hand yet — gameplay-equivalent
/// except a counter would still let the scry happen on Oracle).
pub fn devourer_of_destiny() -> CardDefinition {
    CardDefinition {
        name: "Devourer of Destiny",
        cost: cost(&[generic(5)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 7,
        toughness: 5,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

// ── Goryo's creatures ────────────────────────────────────────────────────────

/// Atraxa, Grand Unifier — {3}{W}{U}{B}{R}{G}, 7/7 Legendary Phyrexian
/// Praetor. Flying, vigilance, deathtouch, lifelink. ETB reveals the top
/// ten cards of your library, you may put up to one of each card type
/// into your hand, the rest on the bottom.
///
/// Approximation: ETB Draw 4 — the average reveal-and-sort yield in a
/// modern reanimator deck (lands + creatures + spell types). Skips the
/// reveal-and-pick machinery, which would require a typed multi-pick
/// decision the engine doesn't expose yet.
/// TODO: implement the real reveal-and-sort ETB.
pub fn atraxa_grand_unifier() -> CardDefinition {
    CardDefinition {
        name: "Atraxa, Grand Unifier",
        cost: cost(&[generic(3), w(), u(), b(), r(), g()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian, CreatureType::Angel],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![
            Keyword::Flying,
            Keyword::Vigilance,
            Keyword::Deathtouch,
            Keyword::Lifelink,
        ],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(4),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Griselbrand — {4}{B}{B}{B}{B}, 7/7 Legendary Demon. Flying, lifelink. Pay
/// 7 life: draw seven cards. Stub: vanilla 7/7 with flying + lifelink; the
/// activated draw-7-pay-7 ability is wired.
pub fn griselbrand() -> CardDefinition {
    use crate::card::ActivatedAbility;
    use crate::effect::{Selector, Value};
    use crate::mana::ManaCost;
    CardDefinition {
        name: "Griselbrand",
        cost: cost(&[generic(4), b(), b(), b(), b()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Demon],
            ..Default::default()
        },
        power: 7,
        toughness: 7,
        keywords: vec![Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: ManaCost::default(),
            // "Pay 7 life: Draw seven cards."
            effect: Effect::Seq(vec![
                Effect::LoseLife { who: Selector::You, amount: Value::Const(7) },
                Effect::Draw { who: Selector::You, amount: Value::Const(7) },
            ]),
            once_per_turn: false,
            sorcery_speed: false,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Psychic Frog — {U}{B}, 1/3 Frog. Flying. "Discard a card: Psychic Frog
/// gets +1/+1 until end of turn." "Sacrifice Psychic Frog: Each opponent
/// mills 4 cards." Both costs are modeled as the first step of the resolved
/// effect (rather than at activation time), which is gameplay-equivalent
/// here — the bot/UI never tries to interrupt between cost payment and
/// resolution.
pub fn psychic_frog() -> CardDefinition {
    CardDefinition {
        name: "Psychic Frog",
        cost: cost(&[u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: vec![
            // "Discard a card: Psychic Frog gets +1/+1 until end of turn."
            ActivatedAbility {
                tap_cost: false,
                mana_cost: ManaCost::default(),
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::PumpPT {
                        what: Selector::This,
                        power: Value::Const(1),
                        toughness: Value::Const(1),
                        duration: Duration::EndOfTurn,
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
            },
            // "Sacrifice Psychic Frog: Each opponent mills 4 cards."
            ActivatedAbility {
                tap_cost: false,
                mana_cost: ManaCost::default(),
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::This,
                        to: ZoneDest::Graveyard,
                    },
                    Effect::Mill {
                        who: Selector::Player(PlayerRef::EachOpponent),
                        amount: Value::Const(4),
                    },
                ]),
                once_per_turn: false,
                sorcery_speed: false,
            },
        ],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Quantum Riddler — {1}{U}{B}, 4/4 Sphinx with flying (approximation).
/// "When Quantum Riddler enters, draw a card." Models the on-cast cantrip
/// as an ETB Draw 1 trigger — gameplay-equivalent for the standard line.
pub fn quantum_riddler() -> CardDefinition {
    CardDefinition {
        name: "Quantum Riddler",
        cost: cost(&[generic(1), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Solitude — {3}{W}, 3/2 Kor Cleric. Flash. Flying, lifelink. When Solitude
/// enters, exile target nonwhite creature an opponent controls. Evoke: exile
/// a white card from your hand (pitch alt cost; Solitude is sacrificed on
/// ETB after its triggers fire).
///
/// "Nonwhite creature an opponent controls" is approximated by the
/// `Creature.and(ControlledByOpponent)` filter — non-white isn't enforced
/// (the engine has only `HasColor`, not `Not(HasColor)` cleanly composed
/// here).
pub fn solitude() -> CardDefinition {
    CardDefinition {
        name: "Solitude",
        cost: cost(&[generic(3), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying, Keyword::Lifelink],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 0,
            exile_filter: Some(SelectionRequirement::HasColor(Color::White)),
            evoke_sacrifice: true,
            not_your_turn_only: false,
            target_filter: None,
        }),
        back_face: None,
    }
}

// ── Sideboard creatures ──────────────────────────────────────────────────────

/// Chancellor of the Annex — {4}{W}{W}, 5/6 Avatar. Flying. "You may reveal
/// this from your opening hand. If you do, the first spell an opponent casts
/// next turn doesn't resolve unless they pay {1}." Stub: vanilla 5/6 flier.
/// TODO: opening-hand annex tax.
pub fn chancellor_of_the_annex() -> CardDefinition {
    CardDefinition {
        name: "Chancellor of the Annex",
        cost: cost(&[generic(4), w(), w()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Elesh Norn, Mother of Machines — {3}{W}{W}, 4/7 Legendary Phyrexian
/// Praetor. Vigilance. "If a permanent entering the battlefield causes a
/// triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time. Permanents entering the battlefield don't
/// cause abilities of permanents your opponents control to trigger."
/// Stub: vanilla 4/7 vigilance; static ETB-trigger replacements omitted.
/// TODO: implement ETB-trigger doubling/suppression static.
pub fn elesh_norn_mother_of_machines() -> CardDefinition {
    CardDefinition {
        name: "Elesh Norn, Mother of Machines",
        cost: cost(&[generic(3), w(), w()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Phyrexian],
            ..Default::default()
        },
        power: 4,
        toughness: 7,
        keywords: vec![Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
    }
}

/// Teferi, Time Raveler — {1}{W}{U} Legendary Planeswalker — Teferi (4
/// loyalty). Static: each opponent can cast spells only any time they could
/// cast a sorcery. +1: until your next turn, you may cast sorcery spells as
/// though they had flash. -3: return target nonland permanent an opponent
/// controls to its owner's hand. Draw a card.
///
/// Wired loyalty ability: **-3 bounce + draw**. The +1 flash-on-sorceries
/// and the static spell-timing restriction still need engine support
/// (sorcery-timing override + per-spell timing veto).
pub fn teferi_time_raveler() -> CardDefinition {
    use crate::card::LoyaltyAbility;
    CardDefinition {
        name: "Teferi, Time Raveler",
        cost: cost(&[generic(1), w(), u()]),
        supertypes: vec![Supertype::Legendary],
        card_types: vec![CardType::Planeswalker],
        subtypes: Subtypes {
            planeswalker_subtypes: vec![PlaneswalkerSubtype::Teferi],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 4,
        loyalty_abilities: vec![LoyaltyAbility {
            loyalty_cost: -3,
            effect: Effect::Seq(vec![
                Effect::Move {
                    what: target_filtered(
                        SelectionRequirement::Permanent
                            .and(SelectionRequirement::Nonland)
                            .and(SelectionRequirement::ControlledByOpponent),
                    ),
                    to: ZoneDest::Hand(PlayerRef::OwnerOf(Box::new(Selector::Target(0)))),
                },
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
            ]),
        }],
        alternative_cost: None,
        back_face: None,
    }
}
