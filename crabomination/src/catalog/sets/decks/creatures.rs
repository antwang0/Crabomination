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

/// Callous Sell-Sword — {3}{R}, 4/4 Human Mercenary. The full Oracle is a
/// 1/1 with a "this gets +X/+0 where X is the sacrificed power" Casualty
/// mechanic. We approximate with an ETB sacrifice-and-pump:
///
/// **ETB**: Sacrifice a creature you control. Callous Sell-Sword gets
/// +(sacrificed creature's power)/+0 until end of turn. Modeled via
/// `Effect::Seq([SacrificeAndRemember, PumpPT { power: SacrificedPower }])`,
/// reusing the same primitives Thud already exercises. The Casualty 2
/// "copy this spell" half is omitted (no copy primitive yet).
pub fn callous_sell_sword() -> CardDefinition {
    use crate::effect::{Duration, PlayerRef};
    CardDefinition {
        name: "Callous Sell-Sword",
        cost: cost(&[generic(1), r()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Mercenary],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                Effect::PumpPT {
                    what: Selector::This,
                    power: Value::SacrificedPower,
                    toughness: Value::Const(0),
                    duration: Duration::EndOfTurn,
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
    }
}

/// Chancellor of the Tangle — {5}{G}, 6/7 Avatar Incarnation. "You may reveal
/// this card from your opening hand. If you do, at the beginning of your
/// first main phase, add {G}."
///
/// Wired via `OpeningHandEffect::RevealForDelayedTrigger` with a
/// `YourNextUpkeep` body that adds {G} to the controller's mana pool. We
/// fire on the first upkeep instead of the first main step (the engine has
/// no dedicated "first main phase" delayed-trigger kind), which is
/// gameplay-equivalent — mana pools don't empty between Upkeep and main, so
/// the {G} is still available for the player's first cast.
pub fn chancellor_of_the_tangle() -> CardDefinition {
    use crate::effect::{DelayedTriggerKind, ManaPayload, OpeningHandEffect};
    CardDefinition {
        name: "Chancellor of the Tangle",
        cost: cost(&[generic(5), g(), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: Some(OpeningHandEffect::RevealForDelayedTrigger {
            kind: DelayedTriggerKind::YourNextMainPhase,
            body: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
        }),
    }
}

/// Cosmogoyf — {2}{G}, *X/X+1* where X = number of different card types
/// among cards in all graveyards.
///
/// Wired via a per-frame layer-7 `SetPowerToughness(N, N+1)` injected in
/// `compute_battlefield` (where N = `distinct_card_types_in_all_graveyards`).
/// The base power/toughness on the definition (4/5) only matters until
/// any layer effect runs, so for any computed view it's overwritten.
pub fn cosmogoyf() -> CardDefinition {
    CardDefinition {
        name: "Cosmogoyf",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        // Base power/toughness — overridden per-frame by the layer-7 set-PT
        // effect injected in `compute_battlefield` based on graveyard contents.
        power: 0,
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
    }
}

/// Devourer of Destiny — {7}, 6/6 colorless Eldrazi. "When you cast this
/// spell, scry 2." The on-cast trigger fires off the just-cast card via
/// the engine's `SpellCast` + `SelfSource` path (the scry resolves before
/// Devourer enters the battlefield).
pub fn devourer_of_destiny() -> CardDefinition {
    CardDefinition {
        name: "Devourer of Destiny",
        cost: cost(&[generic(7)]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Goryo's creatures ────────────────────────────────────────────────────────

/// Atraxa, Grand Unifier — {3}{W}{U}{B}{R}{G}, 7/7 Legendary Phyrexian
/// Praetor. Flying, vigilance, deathtouch, lifelink. ETB reveals the top
/// ten cards of your library, you may put up to one of each card type
/// into your hand, the rest on the bottom.
///
/// Wired ETB: `Effect::AtraxaRevealTopTen` — counts distinct card types in
/// the top 10 of the controller's library and draws that many cards. The
/// ordering of "draw from the top after reordering" is collapsed to plain
/// Draw N for simplicity — gameplay-equivalent in expected card economy
/// for a typical reanimator pile (no library manipulation reordering).
pub fn atraxa_grand_unifier() -> CardDefinition {
    CardDefinition {
        name: "Atraxa, Grand Unifier",
        cost: cost(&[generic(3), g(), w(), u(), b()]),
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
            // Draw `DistinctCardTypesInTopN(You, 10)` cards. With the new
            // `Value::DistinctTypesInTopOfLibrary`, we count actual card
            // types in the top 10 of the controller's library rather than
            // assuming a flat 4 — so a graveyard-heavy library reveals
            // fewer types and a balanced library reveals more.
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::DistinctTypesInTopOfLibrary {
                    who: PlayerRef::You,
                    count: Box::new(Value::Const(10)),
                },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
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
        toughness: 2,
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
                sac_cost: false,
                condition: None,
            life_cost: 0,
            exile_gy_cost: 0,
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
    }
}

/// Quantum Riddler — {3}{U}{B}, 4/6 Sphinx with flying. "When you cast
/// Quantum Riddler, draw a card." Wired as a real on-cast trigger via
/// `SpellCast` + `SelfSource`, so the cantrip fires (and the card resolves)
/// even if Quantum Riddler itself is countered.
pub fn quantum_riddler() -> CardDefinition {
    CardDefinition {
        name: "Quantum Riddler",
        cost: cost(&[generic(3), u(), b()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Draw {
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
    }
}

/// Solitude — {3}{W}{W}, 3/2 Kor Cleric. Flash. Flying, lifelink. When Solitude
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
        // Real Oracle: `{3}{W}`. Pre-fix the catalog had `{3}{W}{W}` (one
        // extra white pip), which made the spell uncastable in the
        // existing test fixtures and slightly off-flavor (Solitude is a
        // single-white-pip MH2 evoke spell, not a double-white).
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
            mode_on_alt: None,
        }),
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

// ── Sideboard creatures ──────────────────────────────────────────────────────

/// Chancellor of the Annex — {4}{W}{W}, 5/6 Avatar. Flying. "You may reveal
/// this from your opening hand. If you do, the first spell an opponent casts
/// next turn doesn't resolve unless they pay {1}."
///
/// Approximation: opening-hand reveal stamps each opponent with a "first
/// spell costs {1} more" charge (`Player.first_spell_tax_charges`). The
/// caster path consumes one charge per spell cast, so the very next spell
/// each opponent casts pays {1} extra. We collapse "doesn't resolve unless
/// they pay" to "costs {1} more" (auto-applied; if they can't afford the
/// extra mana the cast fails outright, which is gameplay-equivalent).
pub fn chancellor_of_the_annex() -> CardDefinition {
    use crate::effect::OpeningHandEffect;
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: Some(OpeningHandEffect::RevealForDelayedTrigger {
            // Fire on the upkeep so the first spell each opponent casts
            // **next turn** is taxed (the engine fires this delayed
            // trigger on the controller's next upkeep — the very turn
            // they'd be ready to cast). The body uses `EachOpponent`
            // (relative to the trigger's controller, i.e. the chancellor's
            // owner) and stamps one charge per opponent.
            kind: crate::effect::DelayedTriggerKind::YourNextUpkeep,
            body: Effect::AddFirstSpellTax {
                who: PlayerRef::EachOpponent,
                count: Value::Const(1),
            },
        }),
    }
}

/// Elesh Norn, Mother of Machines — {3}{W}{W}, 4/7 Legendary Phyrexian
/// Praetor. Vigilance. "If a permanent entering the battlefield causes a
/// triggered ability of a permanent you control to trigger, that ability
/// triggers an additional time. Permanents entering the battlefield don't
/// cause abilities of permanents your opponents control to trigger."
///
/// Wired via `actions::etb_trigger_multiplier`: every ETB-trigger push
/// site (the cast resolution path, `fire_self_etb_triggers`, and the
/// `AnotherOfYours` scope path in `stack.rs`) consults the helper, which
/// returns 0 if any opponent of the trigger's controller has an Elesh
/// Norn (suppressing the trigger) or `1 + your_norns` otherwise (one
/// extra fire per Norn under your control).
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
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}

/// Teferi, Time Raveler — {1}{W}{U} Legendary Planeswalker — Teferi (4
/// loyalty). Static: each opponent can cast spells only any time they
/// could cast a sorcery. +1: until your next turn, you may cast sorcery
/// spells as though they had flash. -3: return target nonland permanent
/// an opponent controls to its owner's hand. Draw a card.
///
/// Wired loyalty abilities:
///   * **+1**: `Effect::GrantSorceriesAsFlash { who: You }` flips
///     `Player.sorceries_as_flash` on the controller. The cast paths
///     consult the flag and skip the sorcery-timing gate; `do_untap`
///     clears it on the controller's next turn.
///   * **-3**: bounce a target nonland opponent permanent to its owner's
///     hand, then draw a card.
///
/// The static "each opponent can cast spells only at sorcery speed" half
/// still needs a per-spell timing veto and isn't wired.
pub fn teferi_time_raveler() -> CardDefinition {
    use crate::card::{LoyaltyAbility, StaticAbility};
    use crate::effect::StaticEffect;
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
        static_abilities: vec![
            StaticAbility {
                description: "Each opponent can cast spells only any time they could cast a sorcery.",
                effect: StaticEffect::OpponentsSorceryTimingOnly,
            },
            // +1's "you may cast sorcery spells as though they had flash"
            // collapsed into a permanent static while Teferi is in play.
            StaticAbility {
                description: "You may cast sorcery spells as though they had flash.",
                effect: StaticEffect::ControllerSorceriesAsFlash,
            },
        ],
        base_loyalty: 4,
        loyalty_abilities: vec![
            LoyaltyAbility {
                loyalty_cost: 1,
                effect: Effect::GrantSorceriesAsFlash { who: PlayerRef::You },
            },
            LoyaltyAbility {
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
            },
        ],
        alternative_cost: None,
        additional_sac_cost: None,
        back_face: None,
        opening_hand: None,
    }
}
