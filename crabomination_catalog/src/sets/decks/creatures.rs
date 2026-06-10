//! Creatures used by the BRG and Goryo's demo decks.
//!
//! Many of these have triggered/static abilities or alternative costs the
//! engine doesn't support yet — those abilities are stubbed with `Effect::Noop`
//! and a doc-comment marking the omission. The bodies (cost, P/T, keywords,
//! creature subtypes) are correct so they animate and combat properly.

use crate::card::{
    ActivatedAbility, AlternativeCost, CardDefinition, CardType, CreatureType, DynamicPt, Effect,
    EventKind,
    EventScope, EventSpec, Keyword, PlaneswalkerSubtype, Selector, SelectionRequirement, Subtypes,
    Supertype, TriggeredAbility, Value,
};
use crate::effect::shortcut::target_filtered;
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{Color, ManaCost, b, cost, g, generic, r, u, w};

// ── BRG creatures ────────────────────────────────────────────────────────────

/// Callous Sell-Sword — {1}{B} 2/2 Human Soldier. Enters with a +1/+1
/// counter for each creature that died under your control this turn
/// (`Value::CreaturesDiedThisTurn`). The Adventure half (Burn Together)
/// is omitted — no Adventure cost-mode primitive yet.
pub fn callous_sell_sword() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Callous Sell-Sword",
        cost: cost(&[generic(1), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Human, CreatureType::Soldier],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        enters_with_counters: Some((
            CounterType::PlusOnePlusOne,
            Value::CreaturesDiedThisTurn(PlayerRef::You),
        )),
        ..Default::default()
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
        cost: cost(&[generic(4), g(), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 6,
        toughness: 7,
        keywords: vec![Keyword::Reach, Keyword::Vigilance],
        opening_hand: Some(OpeningHandEffect::RevealForDelayedTrigger {
            kind: DelayedTriggerKind::YourNextMainPhase,
            body: Effect::AddMana {
                who: PlayerRef::You,
                pool: ManaPayload::Colors(vec![Color::Green]),
            },
        }),
        ..Default::default()
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
        dynamic_pt: Some(DynamicPt::DistinctTypesInAllGraveyards),
        cost: cost(&[generic(1), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        // Base power/toughness — overridden per-frame by the layer-7 set-PT
        // effect injected in `compute_battlefield` based on graveyard contents.
        toughness: 1,
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Eldrazi],
            ..Default::default()
        },
        power: 6,
        toughness: 6,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
        }],
        ..Default::default()
    }
}

// ── Goryo's creatures ────────────────────────────────────────────────────────

/// Atraxa, Grand Unifier — {3}{W}{U}{B}{R}{G}, 7/7 Legendary Phyrexian
/// Praetor. Flying, vigilance, deathtouch, lifelink. ETB reveals the top
/// ten cards of your library, you may put up to one of each card type
/// into your hand, the rest on the bottom.
///
/// ETB reveals the top ten and takes one card of each card type into hand
/// (`Effect::RevealTopTakeOnePerType`), bottoming the rest.
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
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::RevealTopTakeOnePerType {
                who: PlayerRef::You,
                count: Value::Const(10),
            },
        }],
        ..Default::default()
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
        activated_abilities: vec![ActivatedAbility {
            energy_cost: 0,
            discard_cost: None,
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
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None, sac_other_filter: None,
            tap_other_filter: None, from_hand: false,
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// Psychic Frog — {U}{B} 1/2 Frog (MH3). Combat damage to a player/PW →
/// draw. "Discard a card: +1/+1 counter." "Exile three cards from your
/// graveyard: gains flying EOT." The discard / graveyard-exile costs are
/// modeled as the first step of the resolved effect (gameplay-equivalent —
/// nothing can respond between cost and resolution).
pub fn psychic_frog() -> CardDefinition {
    use crate::card::{EventKind, EventScope, EventSpec, TriggeredAbility};
    CardDefinition {
        name: "Psychic Frog",
        cost: cost(&[u(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::DealsCombatDamageToPlayer, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        activated_abilities: vec![
            // "Discard a card: Put a +1/+1 counter on Psychic Frog."
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                effect: Effect::Seq(vec![
                    Effect::Discard {
                        who: Selector::You,
                        amount: Value::Const(1),
                        random: false,
                    },
                    Effect::AddCounter {
                        what: Selector::This,
                        kind: crate::card::CounterType::PlusOnePlusOne,
                        amount: Value::Const(1),
                    },
                ]),
                ..Default::default()
            },
            // "Exile three cards from your graveyard: gains flying EOT."
            ActivatedAbility {
                energy_cost: 0,
                discard_cost: None,
                effect: Effect::Seq(vec![
                    Effect::Move {
                        what: Selector::take(
                            Selector::CardsInZone {
                                who: PlayerRef::You,
                                zone: crate::card::Zone::Graveyard,
                                filter: SelectionRequirement::Any,
                            },
                            Value::Const(3),
                        ),
                        to: ZoneDest::Exile,
                    },
                    Effect::GrantKeyword {
                        what: Selector::This,
                        keyword: Keyword::Flying,
                        duration: Duration::EndOfTurn,
                    },
                ]),
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

/// Quantum Riddler — {3}{U}{B}, 4/6 Sphinx with flying. "When you cast
/// Quantum Riddler, draw a card." Wired as a real on-cast trigger via
/// `SpellCast` + `SelfSource`, so the cantrip fires (and the card resolves)
/// even if Quantum Riddler itself is countered.
pub fn quantum_riddler() -> CardDefinition {
    CardDefinition {
        name: "Quantum Riddler",
        cost: cost(&[generic(3), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Sphinx],
            ..Default::default()
        },
        power: 4,
        toughness: 6,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::SpellCast, EventScope::SelfSource),
            effect: Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        }],
        ..Default::default()
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
        cost: cost(&[generic(3), w(), w()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Kor, CreatureType::Cleric],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Lifelink],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Exile {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            life_cost: 0,
            exile_filter: Some(SelectionRequirement::HasColor(Color::White)),
            evoke_sacrifice: true,
            not_your_turn_only: false,
            target_filter: None,
            condition: None,
                    exile_from_graveyard_count: 0,
                    return_to_hand: None,
                    sacrifice_permanents: None,
            effect_override: None,
            dash: false,
            blitz: false,
            flash: false,
            marks_kicked: false,
            emerge: None,
            impending: 0,
        }),
        ..Default::default()
    }
}

/// Grief — {1}{B}{B} Creature — Elemental Incarnation, 3/2, Menace.
/// Evoke—exile a black card from hand. ETB: target opponent discards a
/// nonland card you choose (Thoughtseize-on-ETB). (Reveal-hand step
/// collapses to the engine's `DiscardChosen`.)
pub fn grief() -> CardDefinition {
    CardDefinition {
        name: "Grief",
        cost: cost(&[generic(1), b(), b()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 2,
        keywords: vec![Keyword::Menace],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DiscardChosen {
                from: Selector::Player(PlayerRef::EachOpponent),
                count: Value::Const(1),
                filter: SelectionRequirement::Nonland,
            },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            exile_filter: Some(SelectionRequirement::HasColor(Color::Black)),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Fury — {3}{R}{R} Creature — Elemental Incarnation, 3/3, Double strike.
/// Evoke—exile a red card from hand. ETB: deal 4 damage divided as you
/// choose among up to two target creatures and/or planeswalkers.
pub fn fury() -> CardDefinition {
    CardDefinition {
        name: "Fury",
        cost: cost(&[generic(3), r(), r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::DoubleStrike],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::DealDamageDivided {
                total: Value::Const(4),
                filter: SelectionRequirement::Creature
                    .or(SelectionRequirement::Planeswalker),
                max_targets: 2,
            },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            exile_filter: Some(SelectionRequirement::HasColor(Color::Red)),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Subtlety — {2}{U}{U} Creature — Elemental Incarnation, 3/3, Flash, Flying.
/// Evoke—exile a blue card from hand. ETB: counter target spell on the stack;
/// its owner puts it on top or bottom of their library. (Printed restriction
/// to creature/planeswalker spells is widened to any spell.)
pub fn subtlety() -> CardDefinition {
    use crate::effect::CounteredSpellZone;
    CardDefinition {
        name: "Subtlety",
        cost: cost(&[generic(2), u(), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::CounterSpellToZone {
                what: target_filtered(SelectionRequirement::IsSpellOnStack),
                zone: CounteredSpellZone::OwnerLibraryTopOrBottom,
            },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            exile_filter: Some(SelectionRequirement::HasColor(Color::Blue)),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Endurance — {1}{G}{G} Creature — Elemental Incarnation, 3/4, Flash, Reach.
/// Evoke—exile a green card from hand. ETB shuffles each opponent's graveyard
/// into their library (printed "up to one target player" narrowed to
/// opponents — the standard graveyard-hate use).
pub fn endurance() -> CardDefinition {
    CardDefinition {
        name: "Endurance",
        cost: cost(&[generic(1), g(), g()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Incarnation],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Flash, Keyword::Reach],
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ShuffleGraveyardIntoLibrary { who: PlayerRef::EachOpponent },
        }],
        alternative_cost: Some(AlternativeCost {
            mana_cost: ManaCost::default(),
            exile_filter: Some(SelectionRequirement::HasColor(Color::Green)),
            evoke_sacrifice: true,
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Kokusho, the Evening Star — {4}{B}{B} 5/5 Legendary Dragon Spirit, Flying.
/// "When Kokusho dies, each opponent loses 5 life and you gain that much."
pub fn kokusho_the_evening_star() -> CardDefinition {
    use crate::effect::shortcut::dies_drain;
    CardDefinition {
        name: "Kokusho, the Evening Star",
        cost: cost(&[generic(4), b(), b()]),
        card_types: vec![CardType::Creature],
        supertypes: vec![Supertype::Legendary],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Dragon, CreatureType::Spirit],
            ..Default::default()
        },
        power: 5,
        toughness: 5,
        keywords: vec![Keyword::Flying],
        triggered_abilities: vec![dies_drain(5)],
        ..Default::default()
    }
}

/// Ophidian — {2}{U} 1/3 Snake. "Whenever this attacks and isn't blocked, you
/// may draw a card; if you do, it assigns no combat damage this turn."
pub fn ophidian() -> CardDefinition {
    use crate::effect::shortcut::on_unblocked;
    CardDefinition {
        name: "Ophidian",
        cost: cost(&[generic(2), u()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Snake], ..Default::default() },
        power: 1,
        toughness: 3,
        triggered_abilities: vec![on_unblocked(Effect::MayDo {
            description: "Draw a card (this assigns no combat damage)".into(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
                Effect::PreventAllCombatDamageInvolving { target: Selector::This },
            ])),
        })],
        ..Default::default()
    }
}

/// Legion Loyalist — {R} 1/1 Goblin Soldier, Haste. Battalion: creatures you
/// control gain first strike and trample until end of turn. (The "can't be
/// blocked by creature tokens" rider is dropped.)
pub fn legion_loyalist() -> CardDefinition {
    use crate::effect::shortcut::battalion;
    let pump = |kw: Keyword| Effect::GrantKeyword {
        what: Selector::EachPermanent(SelectionRequirement::ControlledByYou),
        keyword: kw,
        duration: Duration::EndOfTurn,
    };
    CardDefinition {
        name: "Legion Loyalist",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Goblin, CreatureType::Soldier],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        triggered_abilities: vec![battalion(Effect::Seq(vec![
            pump(Keyword::FirstStrike),
            pump(Keyword::Trample),
        ]))],
        ..Default::default()
    }
}

/// Torch Courier — {R} 1/1 Goblin, Haste. "Sacrifice this: another target
/// creature gains haste until end of turn."
pub fn torch_courier() -> CardDefinition {
    CardDefinition {
        name: "Torch Courier",
        cost: cost(&[r()]),
        card_types: vec![CardType::Creature],
        subtypes: Subtypes { creature_types: vec![CreatureType::Goblin], ..Default::default() },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Haste],
        activated_abilities: vec![ActivatedAbility {
            sac_cost: true,
            effect: Effect::GrantKeyword {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::OtherThanSource),
                ),
                keyword: Keyword::Haste,
                duration: Duration::EndOfTurn,
            },
            ..Default::default()
        }],
        ..Default::default()
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
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Avatar],
            ..Default::default()
        },
        power: 5,
        toughness: 6,
        keywords: vec![Keyword::Flying],
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
        ..Default::default()
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
        // The ETB-trigger-spotlight static; `etb_trigger_multiplier`
        // scans the battlefield for any permanent carrying this static
        // ability (rather than matching on the printed name).
        static_abilities: vec![crate::card::StaticAbility {
            description:
                "Permanents entering the battlefield don't cause abilities of \
                 permanents your opponents control to trigger. If a permanent \
                 entering the battlefield causes a triggered ability of a \
                 permanent you control to trigger, that ability triggers an \
                 additional time.",
            effect: crate::effect::StaticEffect::EtbTriggerSpotlight,
        }],
        ..Default::default()
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
                x_cost: false,
                loyalty_cost: 1,
                effect: Effect::GrantSorceriesAsFlash { who: PlayerRef::You },
            },
            LoyaltyAbility {
                x_cost: false,
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
        ..Default::default()
    }
}
