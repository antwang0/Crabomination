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

/// Callous Sell-Sword — {1}{R} Human Mercenary, 2/1, with *Casualty 2*
/// + an ETB pump rolled into one approximation. The real card is a Modal-
/// DFC whose front face is "When this enters, target creature gets +X/+0
/// until end of turn, where X is its power"; *Casualty 2* lets you
/// sacrifice a 2+ power creature as you cast to copy the spell.
///
/// Approximation: ETB sacrifices a 2+ power creature you control (via
/// `SacrificeAndRemember`) and pumps Self by `Value::SacrificedPower`
/// until end of turn — collapsing both the casualty branch and the ETB
/// pump onto a single permanent. AutoDecider sacrifices the first
/// eligible creature; if no 2+ power creature is available the casualty
/// branch silently no-ops and Sell-Sword resolves without a buff. The
/// back-face Burning Cinder Fury isn't modeled.
pub fn callous_sell_sword() -> CardDefinition {
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
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            // Casualty 2 + ETB pump rolled together: sacrifice a 2+ power
            // creature, then pump Self by that power until end of turn.
            effect: Effect::Seq(vec![
                Effect::SacrificeAndRemember {
                    who: PlayerRef::You,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou)
                        .and(SelectionRequirement::PowerAtLeast(2)),
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
        back_face: None,
        start_of_game_effect: None,
    }
}

/// Chancellor of the Tangle — {5}{G}, 6/7 Avatar Incarnation. "You may reveal
/// this card from your opening hand. If you do, at the beginning of your
/// first main phase, add {G}."
///
/// Modeled by adding {G} to the controller's mana pool at game start (the
/// engine's first state once mulligans finish is `step = PreCombatMain`,
/// so the {G} is available for the first cast on turn 1). The "may
/// reveal" choice is collapsed to always-yes — revealing is universally
/// good for the Chancellor's owner.
pub fn chancellor_of_the_tangle() -> CardDefinition {
    use crate::effect::ManaPayload;
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
        start_of_game_effect: Some(Effect::AddMana {
            who: PlayerRef::You,
            pool: ManaPayload::Colors(vec![Color::Green]),
        }),
    }
}

/// Cosmogoyf — {2}{G}, *X/X+1* where X = number of different card types
/// among cards in all graveyards. Wired live in `compute_battlefield`: a
/// per-card layer-7 `SetPowerToughness(N, N+1)` effect is injected at
/// compute-time, where N is `GameState::distinct_card_types_in_all_graveyards`.
/// Test: `cosmogoyf_pt_scales_with_card_types_in_graveyards`.
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
        back_face: None,
        start_of_game_effect: None,
    }
}

/// Devourer of Destiny — {5}, 7/5 colorless Eldrazi. "When you cast this
/// spell, scry 2." The on-cast trigger fires off the just-cast card via
/// the engine's `SpellCast` + `SelfSource` path (the scry resolves before
/// Devourer enters the battlefield).
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
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        start_of_game_effect: None,
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
        start_of_game_effect: None,
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
        start_of_game_effect: None,
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
        start_of_game_effect: None,
    }
}

/// Quantum Riddler — {1}{U}{B}, 4/4 Sphinx with flying. "When you cast
/// Quantum Riddler, draw a card." Wired as a real on-cast trigger via
/// `SpellCast` + `SelfSource`, so the cantrip fires (and the card resolves)
/// even if Quantum Riddler itself is countered.
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
        back_face: None,
        start_of_game_effect: None,
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
        start_of_game_effect: None,
    }
}

// ── Sideboard creatures ──────────────────────────────────────────────────────

/// Chancellor of the Annex — {4}{W}{W}, 5/6 Avatar. Flying. "You may reveal
/// this from your opening hand. If you do, the first spell an opponent casts
/// next turn doesn't resolve unless they pay {1}."
///
/// Approximation: the start-of-game pass schedules each opponent's
/// `Player.first_spell_tax_remaining = 1`, so their next cast simply
/// pays {1} more. The "doesn't resolve unless they pay" semantics are
/// collapsed to a flat cost increase (real Oracle interrupts after they
/// cast and asks for the tax) — gameplay-equivalent for the demo.
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
        start_of_game_effect: Some(Effect::ScheduleFirstSpellTax {
            who: PlayerRef::EachOpponent,
            amount: 1,
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
/// site (the cast resolution path, `fire_self_etb_triggers`, etc.) consults
/// the helper, which returns 0 if any opponent of the trigger's controller
/// has an Elesh Norn (suppressing the trigger) or `1 + your_norns`
/// otherwise (one extra fire per Norn under your control). Currently
/// covers self-source ETB triggers; the AnotherOfYours scope is still
/// unmodified (TODO).
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
        start_of_game_effect: None,
    }
}

/// Teferi, Time Raveler — {1}{W}{U} Legendary Planeswalker — Teferi (4
/// loyalty). Static: each opponent can cast spells only any time they
/// could cast a sorcery. +1: until your next turn, you may cast sorcery
/// spells as though they had flash. -3: return target nonland permanent
/// an opponent controls to its owner's hand. Draw a card.
///
/// All three abilities wired:
///
/// - **Static** "each opponent can cast spells only any time they could
///   cast a sorcery": `StaticEffect::OpponentsSorceryTimingOnly`. The
///   cast paths consult `player_locked_to_sorcery_timing` and reject
///   instant/flash casts from a restricted player at non-sorcery timing.
/// - **+1** "until your next turn, you may cast sorcery spells as though
///   they had flash": `StaticEffect::ControllerSorceriesAsFlash` on the
///   static_abilities array. The cast paths' timing check treats
///   Sorcery cards as instant-speed when the caster has the static.
///   Currently the engine grants this permanently while Teferi is in
///   play (no until-your-next-turn duration on statics yet); the +1
///   loyalty ability still costs +1 loyalty when activated.
/// - **-3**: `Move(target nonland opp permanent → owner's hand) + Draw 1`.
///
/// Tests: `teferi_minus_three_returns_target_and_draws`,
/// `teferi_plus_one_lets_you_cast_sorceries_at_instant_speed`,
/// `teferi_static_locks_opponents_to_sorcery_timing`.
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
                // +1: until your next turn, sorceries gain flash. The
                // static above already grants this permanently; +1's
                // body is `Noop` but the loyalty bump still applies.
                loyalty_cost: 1,
                effect: Effect::Noop,
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
        back_face: None,
        start_of_game_effect: None,
    }
}
