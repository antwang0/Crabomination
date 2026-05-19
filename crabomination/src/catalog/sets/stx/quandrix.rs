//! Quandrix (G/U) college cards from Strixhaven.
//!
//! Quandrix cares about **Fractal tokens** (0/0 green-and-blue with
//! variable +1/+1 counters), spell-cast triggers, and X-cost scaling.
//! The first-pass set here covers the two college "Apprentice" /
//! "Pledgemage" creatures plus a couple of mono-flavour scaling cards.
//! Larger Fractal-creator effects (Body of Research, Fractal Anomaly)
//! are already wired in `mono` / SOS — those compose against the same
//! `LastCreatedToken` plumbing this module re-uses.

use super::no_abilities;
use crate::card::{
    ActivatedAbility, CardDefinition, CardType, CounterType, CreatureType, Effect, EventKind,
    EventScope, EventSpec, Keyword, Selector, SelectionRequirement, Subtypes, TokenDefinition,
    TriggeredAbility, Value,
};
use crate::effect::shortcut::{magecraft, magecraft_self_pump, target_filtered};
use crate::effect::{Duration, PlayerRef, ZoneDest};
use crate::mana::{cost, generic, g, u, Color, ManaCost};

// ── Quandrix Apprentice ─────────────────────────────────────────────────────

/// Quandrix Apprentice — {G}{U}, 1/1 Elf Druid.
/// "Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// target creature you control gets +1/+1 until end of turn."
///
/// Same shape as Eager First-Year (the Silverquill apprentice), just
/// gated to a creature you control rather than any creature. Wired via
/// the new `effect::shortcut::magecraft` helper plus
/// `Predicate::EntityMatches` on the cast.
pub fn quandrix_apprentice() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Apprentice",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::PumpPT {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            power: Value::Const(1),
            toughness: Value::Const(1),
            duration: Duration::EndOfTurn,
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Pledgemage ─────────────────────────────────────────────────────

/// Quandrix Pledgemage — {1}{G}{U}, 2/2 Fractal Wizard. "{1}{G}{U}: Put
/// a +1/+1 counter on Quandrix Pledgemage."
///
/// Pure activated +1/+1 counter pump. The Fractal subtype is already in
/// the engine (added with the SOS Fractal package), so the body and
/// counter accrual are faithful to the printed card.
pub fn quandrix_pledgemage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pledgemage",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: false,
            mana_cost: cost(&[generic(1), g(), u()]),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: false,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false, exile_other_filter: None,
            self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Decisive Denial ─────────────────────────────────────────────────────────

/// Decisive Denial — {G}{U} Instant. "Choose one — / • Counter target
/// noncreature spell unless its controller pays {2}. / • Target creature
/// you control deals damage equal to its power to target creature you
/// don't control."
///
/// Mode 1 is a Fight resolution; the printed "two target" prompt is
/// auto-resolved on the defender side, attacker is player-chosen via
/// `Target(0)`. Multi-target defender prompt remains a future engine
/// enhancement.
pub fn decisive_denial() -> CardDefinition {
    use crate::mana::{ManaCost, generic as gen_pip};
    let two = ManaCost { symbols: vec![gen_pip(2)] };
    CardDefinition {
        name: "Decisive Denial",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ChooseMode(vec![
            // Mode 0: counter target noncreature spell unless its controller
            // pays {2}.
            Effect::CounterUnlessPaid {
                what: target_filtered(
                    SelectionRequirement::IsSpellOnStack
                        .and(SelectionRequirement::HasCardType(CardType::Creature).negate()),
                ),
                mana_cost: two,
            },
            // Mode 1: target creature you control fights an auto-picked
            // opponent creature (same Chelonian Tackle pattern).
            Effect::Fight {
                attacker: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                defender: Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByOpponent),
                ),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Symmathematics ──────────────────────────────────────────────────────────

/// Symmathematics — {1}{G}{U}, Creature — Fractal (0/0). "Symmathematics
/// enters with two +1/+1 counters on it. / Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, double the number of +1/+1
/// counters on Symmathematics."
///
/// Body is a 0/0 Fractal that comes in as a 2/2 via the new
/// `CardDefinition.enters_with_counters` field (CR 614.12 replacement).
/// The two +1/+1 counters land **before** the new permanent is exposed
/// to state-based-action sweeps and before any ETB triggers fire, so a
/// printed 0/0 body survives ETB without the historic base-toughness
/// bump (was 1/1 base + ETB AddCounter approximation; now exact 0/0
/// printed with CR-compliant "enters with").
///
/// Magecraft is the standard `AddCounter { what: This, amount:
/// CountersOn(This, +1/+1) }` shape (same as Practical Research, Growth
/// Curve): adds N more counters where N is the current pile, producing
/// 2N total. `Selector::This` resolves to the trigger's listening
/// permanent (Symmathematics itself).
pub fn symmathematics() -> CardDefinition {
    CardDefinition {
        name: "Symmathematics",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        // Printed P/T is 0/0 — the +1/+1 counters from the CR 614.12
        // replacement now land before SBA, so the printed base survives.
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            // Magecraft: double the +1/+1 counters on Symmathematics.
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::This),
                    kind: CounterType::PlusOnePlusOne,
                },
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        // CR 614.12 "enters with two +1/+1 counters on it" replacement.
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Summoner (batch 15) ────────────────────────────────────────────

/// 0/0 G/U Fractal token used by the new Quandrix minters.
fn quandrix_fractal_token() -> TokenDefinition {
    TokenDefinition {
        name: "Fractal".to_string(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        card_types: vec![CardType::Creature],
        colors: vec![Color::Green, Color::Blue],
        supertypes: vec![],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        activated_abilities: vec![],
        triggered_abilities: vec![],
    }
}

/// Quandrix Summoner — {1}{G}{U}, 2/2 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, create
/// a 0/0 green and blue Fractal creature token, then put one +1/+1
/// counter on it."
///
/// Three-mana 2/2 + 1/1 Fractal — solid early Fractal-tribal play.
/// The Fractal scales with Quandrix +1/+1-counter doublers (Tanazir,
/// Symmathematics, Quandrix Doubler).
pub fn quandrix_summoner() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Summoner",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: create_token_with_counter(
                PlayerRef::You,
                1,
                quandrix_fractal_token(),
                CounterType::PlusOnePlusOne,
                1,
            ),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Scholar (batch 15) ─────────────────────────────────────────────

/// Quandrix Scholar — {G}{U}, 1/2 Elf Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on target
/// creature you control."
///
/// Two-mana Quandrix value engine — each cast pumps a creature you
/// control. Pairs with Quandrix Apprentice (similar +1/+1 EOT) for
/// double-counter chains via the same magecraft.
pub fn quandrix_scholar() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scholar",
        cost: cost(&[g(), u()]),
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
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Ecologist (batch 15) ───────────────────────────────────────────

/// Quandrix Ecologist — {3}{G}{U}, 4/4 Beast, Trample.
///
/// Printed Oracle (synthesised): "Trample / When this creature enters,
/// scry 2 and put a +1/+1 counter on this creature."
///
/// Five-mana Quandrix beater — a 5/5 Trample after ETB with built-in
/// smoothing. Solid mid-range finisher. The +1/+1 counter doubles
/// under Tanazir's attack trigger.
pub fn quandrix_ecologist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Ecologist",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Beast],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
                Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}


// ── Quandrix Symmetrist (batch 17) ──────────────────────────────────────────

/// Quandrix Symmetrist — {2}{G}{U}, 3/3 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2,
/// then draw a card."
///
/// Mid-curve Quandrix card-selection body. 3/3 for 4 with built-in
/// smoothing + cantrip — pure value. Pairs naturally with Magecraft
/// engines (every IS cast post-ETB is a free trigger).
pub fn quandrix_symmetrist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Symmetrist",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Scry { who: PlayerRef::You, amount: Value::Const(2) },
                Effect::Draw { who: Selector::You, amount: Value::Const(1) },
            ]),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Reckoner (batch 17) ────────────────────────────────────────────

/// Quandrix Reckoner — {1}{G}{U}, 2/2 Frog Druid, Trample.
///
/// Printed Oracle (synthesised): "Trample / Whenever this creature
/// attacks, put a +1/+1 counter on it."
///
/// Per-attack +1/+1 self-grower with Trample — a 2/2 → 3/3 → 4/4
/// progression that punches through chump blockers. Stacks with
/// Symmathematics / Tanazir's doubling effects for explosive late-game.
pub fn quandrix_reckoner() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Reckoner",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Reinforcement (batch 17) ────────────────────────────────────────

/// Fractal Reinforcement — {G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Put a +1/+1 counter on each creature
/// you control."
///
/// Strict anthem via counters — durable through layer effects (counters
/// aren't pumps that wear off at EOT). Pairs with Tanazir (doubles
/// counters on attack) and Symmathematics for fan-out scaling.
pub fn fractal_reinforcement() -> CardDefinition {
    CardDefinition {
        name: "Fractal Reinforcement",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::ForEach {
            selector: Selector::EachPermanent(SelectionRequirement::Creature
                .and(SelectionRequirement::ControlledByYou)),
            body: Box::new(Effect::AddCounter {
                what: Selector::TriggerSource,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Tutelary (batch 17) ────────────────────────────────────────────

/// Quandrix Tutelary — {G}{U}, 1/3 Elf Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, you may put a +1/+1 counter on
/// target Fractal you control."
///
/// Counter-on-Fractal magecraft trigger. Pairs with Fractal minters
/// (Body of Research, Fractal Anomaly, Quandrix Summoner) for snowball
/// growth. The optional `MayDo` is left as always-apply since the
/// always-yes is strictly better than skipping.
pub fn quandrix_tutelary() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tutelary",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(SelectionRequirement::Creature
                .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                .and(SelectionRequirement::ControlledByYou)),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Fractalflow (batch 18) ────────────────────────────────────────

/// Quandrix Fractalflow — {2}{G}{U}, 3/3 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 0/0 green and blue Fractal creature token. Put X +1/+1 counters on
/// it, where X is the number of cards in your hand."
///
/// Hand-size-scaling Fractal minter. At a typical {4} cast point with
/// ~3-4 cards in hand, the Fractal lands as a 3/3 or 4/4 — solid value
/// that snowballs when paired with Mind into Matter / Brilliant Plan.
pub fn quandrix_fractalflow() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalflow",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: quandrix_fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::HandSizeOf(PlayerRef::You),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Snake-Charmer (batch 18) ──────────────────────────────────────

/// Quandrix Scrycharmer — {G}{U}, 1/2 Elf Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, scry 1."
///
/// Cheap top-deck-shaping Quandrix body. Pure card-selection magecraft —
/// no late-game finisher payoff but reliably digs toward win conditions
/// in the spell-heavy Quandrix shell. Distinct from extras.rs's
/// `quandrix_snake_charmer` (a different card with a counter trigger).
pub fn quandrix_scrycharmer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scrycharmer",
        cost: cost(&[g(), u()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Crystallizer (batch 18) ───────────────────────────────────────

/// Quandrix Crystallizer — {2}{U}, 2/3 Crab.
///
/// Printed Oracle (synthesised): "Hexproof / {2}{G}{U}, {T}: Put a
/// +1/+1 counter on target creature you control. Activate only as
/// a sorcery."
///
/// Sticky hexproof body with a counter-pump activation. The sorcery-
/// speed gate keeps it from being instant-speed Tanazir; the hexproof
/// keeps it alive through removal so the activation can repeat
/// turn after turn. Synergises with Tanazir's attack-doubling.
pub fn quandrix_crystallizer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Crystallizer",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Crab],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![Keyword::Hexproof],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            tap_cost: true,
            mana_cost: cost(&[generic(2), g(), u()]),
            effect: Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            once_per_turn: false,
            sorcery_speed: true,
            sac_cost: false,
            condition: None,
            life_cost: 0,
            from_graveyard: false,
            exile_self_cost: false,
            exile_other_filter: None,
            self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Multibinding (batch 18) ───────────────────────────────────────

/// Quandrix Multibinding — {2}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Put two +1/+1 counters on target
/// creature you control. Then double the number of +1/+1 counters on
/// it."
///
/// Two-step counter accelerator — drops two +1/+1, then doubles them
/// (via `Value::CountersOn`). On a 0/0 Fractal: 2 → 4. On a 2/2 Bear
/// with one prior counter: 1 → 3 → 6 counters total. Pairs hard with
/// Quandrix Reckoner (per-attack counter) and Symmathematics (cast-
/// doubles counters).
pub fn quandrix_multibinding() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Multibinding",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            // Double counters: add (current count) more so net = 2 * current.
            Effect::AddCounter {
                what: Selector::Target(0),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Geomyst (batch 18) ────────────────────────────────────────────

/// Quandrix Geomyst — {3}{G}{U}, 4/4 Elemental Wizard, Reach.
///
/// Printed Oracle (synthesised): "Reach / When this creature enters,
/// draw a card."
///
/// Five-mana 4/4 reach with a built-in cantrip. Solid value-on-curve
/// that doubles as combat anchor (reach for fliers) + card advantage.
/// The Wizard subtype synergises with Archmage Emeritus and the
/// magecraft pump cycle.
pub fn quandrix_geomyst() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geomyst",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elemental, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Reach],
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
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Doublecaster (batch 19) ───────────────────────────────────────

/// Quandrix Doublecaster — {3}{G}{U}, 3/3 Fractal Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on this
/// creature."
///
/// Permanent self-grower in Quandrix's Fractal subtype. Five-mana 3/3
/// frame that snowballs across multi-spell turns. Pairs hard with
/// Symmathematics (DoubleCounters → each magecraft trigger places 2
/// counters) and Quandrix Multibinding for explosive scaling.
pub fn quandrix_doublecaster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublecaster",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Wavewright (batch 19) ─────────────────────────────────────────

/// Quandrix Wavewright — {2}{G}{U}, 2/3 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2,
/// then draw a card."
///
/// Four-mana 2/3 ETB card-velocity body. Scry 2 + draw 1 is the
/// stronger Symmetrist template (Symmetrist is scry 2 + draw 1 at the
/// same cost). Slots into Quandrix mid-game with no setup required.
pub fn quandrix_wavewright() -> CardDefinition {
    use crate::effect::PlayerRef;
    CardDefinition {
        name: "Quandrix Wavewright",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Sapsprout (batch 19) ──────────────────────────────────────────

/// Quandrix Sapsprout — {G}{U}, 1/2 Fractal.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or
/// copy an instant or sorcery spell, put a +1/+1 counter on this
/// creature."
///
/// One-mana magecraft self-grower. Smaller cousin of Quandrix
/// Doublecaster on a 2-mana frame — the Fractal subtype lets it scale
/// with Doubling Season effects and Symmathematics counter-doubling.
pub fn quandrix_sapsprout() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sapsprout",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Growth (batch 19+) ─────────────────────────────────────────────

/// Fractal Growth — {G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Put a +1/+1 counter on target
/// creature you control. Then that creature gets +1/+1 until end of
/// turn for each +1/+1 counter on it."
///
/// 2-mana combo trick — add 1 counter + temporarily double the body.
/// On a 2/2 Bear with 0 prior counters: 1 → +1/+1 EOT → 3/3 EOT.
/// On a creature with 3 prior counters: 4 → +4/+4 EOT → 8/8 EOT.
/// Pure tempo burst for Quandrix counter shells.
pub fn fractal_growth() -> CardDefinition {
    CardDefinition {
        name: "Fractal Growth",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::PumpPT {
                what: Selector::Target(0),
                power: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
                toughness: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
                duration: crate::effect::Duration::EndOfTurn,
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Calculus (batch 19+) ──────────────────────────────────────────

/// Quandrix Calculus — {2}{G}{U}, 2/2 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, mill
/// two cards. Then draw a card."
///
/// 4-mana 2/2 ETB self-mill-and-cantrip body. Mills 2 of your own
/// cards (graveyard fuel for Lorehold/Witherbloom recursion) and
/// draws 1. Fractal subtype tags into Quandrix counter shells.
pub fn quandrix_calculus() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculus",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
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
                Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
                Effect::Draw {
                    who: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Multiplier (batch 19) ──────────────────────────────────────────

/// Fractal Multiplier — {2}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Double the number of +1/+1 counters
/// on target creature you control."
///
/// Single-clause counter-doubler at the 4-mana slot. Reads current
/// +1/+1 counters via `Value::CountersOn` and adds the same amount —
/// net effect = doubling. On a 0/0 Fractal with 3 counters → 6
/// counters → 6/6. Quandrix's cleanest counter-explosion enabler.
pub fn fractal_multiplier() -> CardDefinition {
    CardDefinition {
        name: "Fractal Multiplier",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::CountersOn {
                what: Box::new(Selector::Target(0)),
                kind: CounterType::PlusOnePlusOne,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Bloom (batch 20) ───────────────────────────────────────────────

/// Fractal Bloom — {3}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is twice your
/// hand size."
///
/// 5-mana Body-of-Research-cousin — mints a Fractal scaled to 2 × hand
/// size. With 5 cards in hand → 10/10 token. Big finisher payoff for the
/// Quandrix card-advantage shell.
pub fn fractal_bloom() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Fractal Bloom",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Times(
                    Box::new(Value::Const(2)),
                    Box::new(Value::HandSizeOf(PlayerRef::You)),
                ),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Spellweaver (batch 20) ────────────────────────────────────────

/// Quandrix Spellweaver — {2}{G}{U}, 2/4 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw two
/// cards. Magecraft — Whenever you cast or copy an instant or sorcery
/// spell, put a +1/+1 counter on this creature."
///
/// 4-mana grindy value Quandrix card-engine — ETB nets two cards (one
/// net after the cast) and magecraft keeps a permanent counter ticking
/// up. Stacks with Symmathematics's counter-doubling for explosive
/// growth.
pub fn quandrix_spellweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Spellweaver",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Wavedancer (batch 20) ─────────────────────────────────────────

/// Quandrix Wavedancer — {1}{U}, 1/3 Merfolk Wizard with Flash.
///
/// Printed Oracle (synthesised): "Flash. When this creature enters,
/// scry 2."
///
/// 2-mana flash blocker + scry 2 ETB — combat-tempo card-selection
/// body. Sits behind the {U} pip cheap-flash and shapes the next two
/// turns of draws.
pub fn quandrix_wavedancer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavedancer",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Synthesis (batch 20) ───────────────────────────────────────────

/// Fractal Synthesis — {2}{G}{U} Instant.
///
/// Printed Oracle (synthesised): "Put two +1/+1 counters on target
/// creature. Draw a card."
///
/// 4-mana instant pump + cantrip. Net-neutral card economy, persistent
/// +2/+2 counters. Slots cleanly into the Quandrix counter-grow plan.
pub fn fractal_synthesis() -> CardDefinition {
    CardDefinition {
        name: "Fractal Synthesis",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(SelectionRequirement::Creature),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Hatchling (batch 20) ──────────────────────────────────────────

/// Quandrix Hatchling — {G}{U}, 0/0 Fractal.
///
/// Printed Oracle (synthesised): "Quandrix Hatchling enters with two
/// +1/+1 counters on it. Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on it."
///
/// 2-mana 2/2 magecraft-counter Fractal — enters with two counters
/// (engine `enters_with_counters` field, CR 614.12) and grows
/// permanently for every IS cast. Stacks with Symmathematics's
/// double-counter static.
pub fn quandrix_hatchling() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Hatchling",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
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
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(2))),
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Calibrator (batch 21) ─────────────────────────────────────────

/// Quandrix Calibrator — {2}{G}, 2/3 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, put a +1/+1
/// counter on target creature you control."
///
/// 3-mana ETB-stat-bump body — defensive Quandrix midrange that puts a
/// counter on any friendly creature (including itself). Works as a
/// repeatable counter source with flicker effects.
pub fn quandrix_calibrator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calibrator",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Resonance (batch 21) ───────────────────────────────────────────

/// Fractal Resonance — {1}{G}{U} Instant.
///
/// Printed Oracle (synthesised): "Put a +1/+1 counter on each creature you
/// control."
///
/// 3-mana team-wide counter pump at instant speed. Strong combat trick
/// that doubles as a permanent stat boost. Stacks with Witherbloom
/// Pestseed for fanout into more counters.
pub fn fractal_resonance() -> CardDefinition {
    use crate::effect::shortcut::each_your_creature;
    CardDefinition {
        name: "Fractal Resonance",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: each_your_creature(),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Mistweaver (batch 21) ─────────────────────────────────────────

/// Quandrix Mistweaver — {1}{U}, 1/2 Merfolk Wizard with Flash and Flying.
///
/// Printed Oracle (synthesised): "Flash, flying. When this creature enters,
/// draw a card."
///
/// 2-mana flash flier cantrip — replaces itself and gives a flying body
/// for chump-blocking or instant-speed pressure. Strong with Pop Quiz /
/// magecraft chain triggers since Flash means it can fire mid-stack.
pub fn quandrix_mistweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mistweaver",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![Keyword::Flash, Keyword::Flying],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Fractal Harvest (batch 21) ─────────────────────────────────────────────

/// Fractal Harvest — {3}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token. Put three +1/+1 counters on it. Draw a card."
///
/// 5-mana 3/3 Fractal + cantrip. Bigger fixed-size Fractal than the
/// X-scaling minters; replaces itself via the cantrip rider.
pub fn fractal_harvest() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Harvest",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            create_token_with_counter(
                PlayerRef::You,
                1,
                fractal_token(),
                CounterType::PlusOnePlusOne,
                3,
            ),
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix Sage (batch 21) ───────────────────────────────────────────────

/// Quandrix Sage — {1}{G}{U}, 2/2 Human Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, scry 1, then draw a card."
///
/// 3-mana magecraft card-quality engine — every IS cast scrys + draws.
/// Pairs perfectly with cantrip chains like Brainstorm + Ponder.
pub fn quandrix_sage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sage",
        cost: cost(&[generic(1), g(), u()]),
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
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Quandrix batch 22 ──────────────────────────────────────────────────────

/// Quandrix Counterbalance — {G}{U} Instant.
///
/// Printed Oracle (synthesised): "Put a +1/+1 counter on target creature
/// you control. Draw a card."
///
/// 2-mana counter + cantrip — the classic Quandrix shape but compressed
/// to instant speed at the curve-2 slot. Pure tempo combat trick.
pub fn quandrix_counterbalance() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Counterbalance",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Draw { who: Selector::You, amount: Value::Const(1) },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Bloom-Caller — {2}{G}{U}, 2/3 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a 0/0
/// green and blue Fractal creature token, then put two +1/+1 counters on
/// it."
///
/// 4-mana 2/3 + 2/2 Fractal token on arrival — two bodies for one card.
/// Both bodies fuel Quandrix counter-related synergies.
pub fn fractal_bloom_caller() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Fractal Bloom-Caller",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: create_token_with_counter(
                PlayerRef::You,
                1,
                quandrix_fractal_token(),
                CounterType::PlusOnePlusOne,
                2,
            ),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Synthesist — {1}{G}{U}, 2/2 Elf Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, put a +1/+1 counter on each creature
/// you control."
///
/// 3-mana magecraft anthem — every spell pumps the whole team. Hard
/// snowball with cheap cantrips; one cast → +1/+1 across the board.
pub fn quandrix_synthesist() -> CardDefinition {
    use crate::card::CounterType;
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Quandrix Synthesist",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: Selector::EachPermanent(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Tessellation — {3}{G}{U} Sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is the number of
/// lands you control."
///
/// 5-mana ramp-payoff scaling Fractal. On turn 5 with 5 lands it lands
/// a 5/5 Fractal; in a long game it scales to 8-10+/+/+/+.
pub fn fractal_tessellation() -> CardDefinition {
    use crate::card::CounterType;
    // Inline `Seq([CreateToken, AddCounter(LastCreatedToken)])` rather
    // than `shortcut::create_token_with_counter` since the helper takes
    // a const `counter_n: i32` and this card needs a `Value::CountOf`
    // for the X = lands-you-control scaling.
    CardDefinition {
        name: "Fractal Tessellation",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::HasCardType(CardType::Land)
                        .and(SelectionRequirement::ControlledByYou),
                )),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Mistshaper — {U}, 1/1 Merfolk Wizard with Flash.
///
/// Printed Oracle (synthesised): "Flash. Magecraft — Whenever you cast
/// or copy an instant or sorcery spell, this creature gets +1/+1 until
/// end of turn."
///
/// 1-mana magecraft-pump Flash body — flashes in to block, then keeps
/// growing on every IS cast. Tiny but snowball-able.
pub fn quandrix_mistshaper() -> CardDefinition {
    use crate::effect::shortcut::magecraft_self_pump;
    CardDefinition {
        name: "Quandrix Mistshaper",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 23: 5 new Quandrix cards ─────────────────────

/// Quandrix Polymath — {1}{G}{U}, 2/2 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, draw a card.
/// Then, you may put a +1/+1 counter on target creature you control."
///
/// 3-mana ETB cantrip + +1/+1 growth. Pairs with Tanazir / Symmathematics
/// counter doublers and rivals Quandrix Apprentice as a magecraft engine.
pub fn quandrix_polymath() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Polymath",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
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
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::AddCounter {
                    what: target_filtered(
                        SelectionRequirement::Creature
                            .and(SelectionRequirement::ControlledByYou),
                    ),
                    kind: CounterType::PlusOnePlusOne,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Avenger — {3}{G}{U}, 0/0 Fractal Soldier.
///
/// Printed Oracle (synthesised): "This creature enters with four +1/+1
/// counters on it. Trample."
///
/// 5-mana 4/4 trampler with growth potential. The base 0/0 + 4 counters
/// scales beautifully with Hardened Scales / Tanazir / Pestseed doublers
/// → an 8/8 trampler on cast.
pub fn fractal_avenger() -> CardDefinition {
    CardDefinition {
        name: "Fractal Avenger",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Soldier],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(4))),
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Cartographer — {2}{G}, 2/3 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, search your
/// library for a basic land card, reveal it, put it into your hand, then
/// shuffle."
///
/// 3-mana fixing ramp body — Quandrix's premier "find a basic" engine.
pub fn quandrix_cartographer() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Quandrix Cartographer",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasSupertype(Supertype::Basic)
                    .and(SelectionRequirement::HasCardType(CardType::Land)),
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Sovereign — {3}{G}{U}, 3/4 Fractal Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, target
/// creature you control gets +1/+1 counters equal to the number of lands
/// you control."
///
/// 5-mana ramp payoff — a 6/7 trampler with 3 lands feels like a real
/// finisher.
pub fn fractal_sovereign() -> CardDefinition {
    CardDefinition {
        name: "Fractal Sovereign",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::HasCardType(CardType::Land)
                        .and(SelectionRequirement::ControlledByYou),
                )),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Pairweaver — {G}{U}, instant.
///
/// Printed Oracle (synthesised): "Put a +1/+1 counter on each of up to two
/// target creatures you control."
///
/// 2-mana double pump — feeds Quandrix counter doublers and tribal Fractal
/// shells.
pub fn quandrix_pairweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pairweaver",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: Selector::TargetFiltered {
                    slot: 1,
                    filter: SelectionRequirement::Creature
                        .and(SelectionRequirement::ControlledByYou),
                },
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 24+: 2 more Quandrix cards ───────────────────

/// Quandrix Pondkeeper — {2}{U}, 1/3 Merfolk Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, create a
/// 0/0 green and blue Fractal creature token. Put X +1/+1 counters on
/// it, where X is the number of instant and sorcery cards in your
/// graveyard."
///
/// 3-mana ETB Fractal sized by your gy IS — strong late-game finisher
/// in spell-heavy shells (8+ IS in gy → 8/8 Fractal). Pairs with
/// Pondkeeper's own Wizard chain.
pub fn quandrix_pondkeeper() -> CardDefinition {
    use crate::card::CounterType;
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Quandrix Pondkeeper",
        cost: cost(&[generic(2), u()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
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
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Counterproof — {G}{U}, instant.
///
/// Printed Oracle (synthesised): "Put a +1/+1 counter on target creature
/// you control. Scry 1."
///
/// 2-mana counter + scry — bridges to the next turn's spell with a small
/// growth on the curve.
pub fn quandrix_counterproof() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Counterproof",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 24: 5 new Quandrix cards ─────────────────────

/// Quandrix Logician — {G}{U}, 2/2 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2.
/// Magecraft — Whenever you cast or copy an instant or sorcery spell,
/// put a +1/+1 counter on target Fractal you control."
///
/// 2-mana ETB selection body + per-cast Fractal pumper. Pairs with every
/// Quandrix Fractal minter for tribal grow plays.
pub fn quandrix_logician() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Logician",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
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
                effect: Effect::Scry {
                    who: PlayerRef::You,
                    amount: Value::Const(2),
                },
            },
            magecraft(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Echoist — {2}{G}{U}, 1/1 Fractal Wizard.
///
/// Printed Oracle (synthesised): "Fractal Echoist enters with X +1/+1
/// counters on it, where X is the number of instant and sorcery cards in
/// your graveyard. Whenever Fractal Echoist attacks, put a +1/+1 counter
/// on it."
///
/// Engine-simplification: the `enters_with_counters` field doesn't support
/// `Value::CountOf` yet for the count, so we collapse to a flat ETB
/// `Seq(GameEntered, AddCounter ×CountOf(IS in gy))` body. The 1/1 base
/// scales with delve-style gy fill.
pub fn fractal_echoist() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Fractal Echoist",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::count(Selector::CardsInZone {
                        who: PlayerRef::You,
                        zone: crate::card::Zone::Graveyard,
                        filter: SelectionRequirement::HasCardType(CardType::Instant)
                            .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                    }),
                },
            },
            TriggeredAbility {
                event: EventSpec::new(EventKind::Attacks, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Mathenotaur — {3}{G}{U}, 4/4 Centaur Wizard Trample.
///
/// Printed Oracle (synthesised): "Trample. When this creature enters,
/// double the number of +1/+1 counters on target creature you control."
///
/// 5-mana finisher that supercharges the Quandrix counters package —
/// drops on a Fractal with 4 counters → 4/4 Centaur + 8/8 Fractal.
pub fn quandrix_mathenotaur() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Mathenotaur",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Centaur, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Surge — {1}{G}{U}, sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is the number of
/// creatures you control."
///
/// 3-mana token-with-creature-count-counters — scales with go-wide
/// boards (5 creatures → 5/5 Fractal for 3 mana).
pub fn fractal_surge() -> CardDefinition {
    use crate::card::CounterType;
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Fractal Surge",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::count(Selector::EachPermanent(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                )),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Aether Adept — {U}, 0/3 Merfolk Wizard Defender.
///
/// Printed Oracle (synthesised): "Defender. {T}: Tap target creature."
///
/// 1-mana defensive tap-engine. Holds the line + repeatable tempo
/// disruption — turns into a wall every turn.
pub fn quandrix_aether_adept() -> CardDefinition {
    use crate::card::ActivatedAbility;
    CardDefinition {
        name: "Quandrix Aether Adept",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 3,
        keywords: vec![Keyword::Defender],
        effect: Effect::Noop,
        activated_abilities: vec![ActivatedAbility {
            mana_cost: ManaCost::default(),
            tap_cost: true,
            sac_cost: false,
            life_cost: 0,
            exile_other_filter: None,
            condition: None,
            exile_self_cost: false,
            from_graveyard: false,
            sorcery_speed: false,
            once_per_turn: false,
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
                    self_counter_cost_reduction: None,
        }],
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}


// ── Push (modern_decks) batch 24++: 1 more Quandrix card ───────────────────

/// Quandrix Symmetrycaster — {3}{G}{U}, 3/3 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, put X
/// +1/+1 counters on it, where X is the number of cards in your hand."
///
/// 5-mana hand-size-scaling body — drops on turn 5 with a typical 3-4
/// card hand → 6/6 to 7/7. Snowballs harder with Quandrix card-draw
/// engines.
pub fn quandrix_symmetrycaster() -> CardDefinition {
    use crate::card::CounterType;
    CardDefinition {
        name: "Quandrix Symmetrycaster",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 25: 5 more Quandrix cards ────────────────────
//
// Continuing Quandrix (G/U) buildout: 3 new creatures + 2 spells using
// existing Fractal token / +1/+1 counter / magecraft primitives. No new
// engine features required.

/// Quandrix Pondweaver — {G}{U}, 1/1 Elf Druid.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy
/// an instant or sorcery spell, scry 1."
///
/// 2-mana scry engine — every IS spell smooths future draws. Slots into
/// any blue-green spell-heavy / Fractal shell.
pub fn quandrix_pondweaver() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Quandrix Pondweaver",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Fractalseed — {1}{G}{U}, 2/2 Fractal.
///
/// Printed Oracle (synthesised): "When this creature enters, put a
/// +1/+1 counter on this creature for each instant and sorcery card in
/// your graveyard."
///
/// 3-mana counter-scaling Fractal — grows by the size of your IS gy
/// pile. Combines with Galvanic Iteration / Flashback to refill the
/// graveyard and pump itself.
pub fn quandrix_fractalseed() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalseed",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                })),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Mapmaker — {2}{G}, 2/3 Elf Druid.
///
/// Printed Oracle (synthesised): "When this creature enters, search your
/// library for a basic Forest or Island card, put it onto the battlefield
/// tapped, then shuffle."
///
/// 3-mana 2/3 ramper. Targeted basic-fetch for the next turn's color
/// fixing. Slots into any Quandrix curve.
pub fn quandrix_mapmaker() -> CardDefinition {
    use crate::effect::ZoneDest;
    CardDefinition {
        name: "Quandrix Mapmaker",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Fractalwave — {2}{G}{U}, sorcery.
///
/// Printed Oracle (synthesised): "Create a 0/0 green and blue Fractal
/// creature token. Put X +1/+1 counters on it, where X is the number of
/// instant and sorcery cards in your graveyard."
///
/// 4-mana Fractal-creator scaling on graveyard size. With a 3+ card IS
/// pile this becomes a 3/3+ for 4 mana. Slots into any spell-recursion
/// Quandrix shell.
pub fn quandrix_fractalwave() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalwave",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountOf(Box::new(Selector::CardsInZone {
                    who: PlayerRef::You,
                    zone: crate::card::Zone::Graveyard,
                    filter: SelectionRequirement::HasCardType(CardType::Instant)
                        .or(SelectionRequirement::HasCardType(CardType::Sorcery)),
                })),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Theorist — {2}{G}{U}, 3/3 Fractal Wizard Trample.
///
/// Printed Oracle (synthesised): "Trample. Whenever you cast or copy an
/// instant or sorcery spell, put a +1/+1 counter on target Fractal you
/// control."
///
/// 4-mana 3/3 trampler that pumps your Fractals on every IS cast. Pairs
/// with Quandrix Fractalseed / Quandrix Fractalwave for layered growth.
pub fn fractal_theorist() -> CardDefinition {
    use crate::effect::shortcut::magecraft;
    CardDefinition {
        name: "Fractal Theorist",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                    .and(SelectionRequirement::ControlledByYou),
            ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Push (modern_decks) batch 28: 5 more Quandrix cards ────────────────────
//
// Continuing Quandrix (G/U) buildout: 5 new cards using existing primitives.
// No new engine features required.

/// Quandrix Sumcaster — {G}{U}, 1/2 Elf Wizard.
///
/// Printed Oracle (synthesised): "Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, you may draw a card. If you do, discard a
/// card."
///
/// 2-mana magecraft looter — every IS cast offers a 1-for-1 filter. Pairs
/// with discard-matters payoffs (Tinybones, Smallpox).
pub fn quandrix_sumcaster() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Sumcaster",
        cost: cost(&[g(), u()]),
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
        triggered_abilities: vec![magecraft(Effect::MayDo {
            description: "draw a card, then discard a card".to_string(),
            body: Box::new(Effect::Seq(vec![
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Multiplicand — {2}{G}{U}, 0/0 Fractal Wizard with 3 +1/+1
/// counters.
///
/// Printed Oracle (synthesised): "This creature enters with three +1/+1
/// counters on it."
///
/// 4-mana 3/3 Fractal body via `enters_with_counters`. Substrate for the
/// counter-doubling lineage (Tanazir / Hardened Scales / Multibinding).
pub fn fractal_multiplicand() -> CardDefinition {
    CardDefinition {
        name: "Fractal Multiplicand",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
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
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Calculus-Mage — {3}{G}{U}, 4/4 Elf Wizard.
///
/// Printed Oracle (synthesised): "When this creature enters, scry 2, then
/// draw a card. Whenever you cast or copy an instant or sorcery spell, put
/// a +1/+1 counter on target Fractal you control."
///
/// 5-mana big-body card-velocity engine + Fractal grower.
pub fn quandrix_calculus_mage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Calculus-Mage",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
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
                effect: Effect::Seq(vec![
                    Effect::Scry {
                        who: PlayerRef::You,
                        amount: Value::Const(2),
                    },
                    Effect::Draw {
                        who: Selector::You,
                        amount: Value::Const(1),
                    },
                ]),
            },
            magecraft(Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature
                        .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal))
                        .and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Tidecaller — {1}{U}, 1/3 Merfolk Wizard Flash.
///
/// Printed Oracle (synthesised): "Flash. When this creature enters, tap
/// target creature."
///
/// 2-mana flash tempo body. Doubles as a flash blocker and a tap-down
/// tempo play during opp combat.
pub fn quandrix_tidecaller() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidecaller",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Tap {
                what: target_filtered(SelectionRequirement::Creature),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Spawning — {2}{G}{U}, sorcery.
///
/// Printed Oracle (synthesised): "Create two 0/0 green-and-blue Fractal
/// creature tokens. Put a +1/+1 counter on each of them."
///
/// 4-mana double-Fractal mint. Both Fractals get a +1/+1 counter via the
/// new `Selector::LastCreatedTokens` (plural) primitive — both survive
/// SBA at 1/1 each. Substrate for counter-doublers.
pub fn fractal_spawning() -> CardDefinition {
    CardDefinition {
        name: "Fractal Spawning",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(2),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedTokens,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 30: 5 new Quandrix cards ─────────────────────────────────────────

/// Quandrix Hydronaut — {1}{G}{U}, 2/2 Merfolk Wizard.
///
/// Synthesised Oracle: "When this creature enters, target creature you
/// control gets +1/+1 counter."
///
/// 3-mana grow body that immediately drops a counter on the chosen creature.
pub fn quandrix_hydronaut() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Hydronaut",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Fractalweaver — {3}{G}{U}, 3/3 Fractal Wizard.
///
/// Synthesised Oracle: "When this creature enters, mill 2. Magecraft —
/// Whenever you cast or copy an instant or sorcery spell, put a +1/+1
/// counter on this creature."
pub fn quandrix_fractalweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fractalweaver",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::Mill {
                    who: Selector::You,
                    amount: Value::Const(2),
                },
            },
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Geomancer — {2}{G}, 2/3 Elf Druid.
///
/// Synthesised Oracle: "When this creature enters, search your library for
/// a basic Forest or Island card, reveal it, put it into your hand, then
/// shuffle." Approximated as basic-land tutor → hand.
pub fn quandrix_geomancer_b30() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geomancer B30",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::IsBasicLand,
                to: crate::effect::ZoneDest::Hand(PlayerRef::You),
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Mindforge — {U}, instant.
///
/// Synthesised Oracle: "Scry 2, then draw a card."
///
/// 1-mana selection + cantrip. Same shape as Preordain.
pub fn quandrix_mindforge() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Mindforge",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Branchwarden — {2}{G}{U}, 3/4 Fractal Druid Reach.
///
/// Synthesised Oracle: "Reach. When this creature enters, draw a card."
///
/// 4-mana defensive body + cantrip. Trades a counter for raw card draw.
pub fn quandrix_branchwarden() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Branchwarden",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
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
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 32 (modern_decks) — Quandrix expansion ────────────────────────────

/// Quandrix Tidewright — {1}{U}, 2/1 Merfolk Wizard Flash.
/// Synthesised Oracle: "Flash. When this creature enters, target creature
/// gets -2/-0 until end of turn."
pub fn quandrix_tidewright() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tidewright",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 1,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::PumpPT {
                what: target_filtered(SelectionRequirement::Creature),
                power: Value::Const(-2),
                toughness: Value::Const(0),
                duration: Duration::EndOfTurn,
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Wavebreaker — {2}{G}{U}, 3/3 Fractal Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, put a +1/+1 counter on this creature."
pub fn quandrix_wavewriter() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavewriter",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Scribe — {G}{U}, 1/2 Elf Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, this creature gets +1/+1 until end of turn."
pub fn quandrix_scribe() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scribe",
        cost: cost(&[g(), u()]),
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
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Geometer — {3}{G}{U}, 4/4 Fractal Wizard.
/// Synthesised Oracle: "When this creature enters, create a 0/0 green-and-
/// blue Fractal creature token, then put X +1/+1 counters on it, where X
/// is the number of cards in your hand."
pub fn quandrix_handmage() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Handmage",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Seq(vec![
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: quandrix_fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedToken,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::HandSizeOf(PlayerRef::You),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Equation — {2}{G}{U}, sorcery.
/// Synthesised Oracle: "Draw a card, then put X +1/+1 counters on target
/// creature you control, where X is the number of cards in your hand."
pub fn quandrix_equipoise() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equipoise",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::HandSizeOf(PlayerRef::You),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Visionary — {U}, 1/1 Merfolk Wizard.
/// Synthesised Oracle: "When this creature enters, scry 1."
pub fn quandrix_visionary() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Visionary",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
                who: PlayerRef::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Wilderwright — {3}{G}, 3/4 Elf Druid Reach.
/// Synthesised Oracle: "When this creature enters, search your library for
/// a basic land card, reveal it, put it onto the battlefield tapped, then
/// shuffle."
pub fn quandrix_wilderwright() -> CardDefinition {
    use crate::card::Supertype;
    CardDefinition {
        name: "Quandrix Wilderwright",
        cost: cost(&[generic(3), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![Keyword::Reach],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Search {
                who: PlayerRef::You,
                filter: SelectionRequirement::HasSupertype(Supertype::Basic)
                    .and(SelectionRequirement::HasCardType(CardType::Land)),
                to: ZoneDest::Battlefield {
                    controller: PlayerRef::You,
                    tapped: true,
                },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 33: 3 new Quandrix cards ────────────────────────────────────

/// Quandrix Pulseweaver — {1}{G}{U}, 2/2 Fractal Wizard Flash.
/// Synthesised Oracle: "Flash / Magecraft — Whenever you cast or copy an
/// instant or sorcery spell, this creature gets +1/+1 until end of turn."
pub fn quandrix_pulseweaver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pulseweaver",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![Keyword::Flash],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft_self_pump(1, 1)],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Reckoner — {3}{G}{U}, 4/4 Fractal.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn fractal_reckoner() -> CardDefinition {
    CardDefinition {
        name: "Fractal Reckoner",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
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
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Inquiry — {U}, Instant.
/// Synthesised Oracle: "Draw a card. Scry 1."
pub fn quandrix_inquiry() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Inquiry",
        cost: cost(&[u()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Scry {
                who: PlayerRef::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Topologist — {2}{U}, 2/2 Merfolk Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card, then
/// discard a card."
pub fn quandrix_topologist() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Topologist",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
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
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::Discard {
                    who: Selector::You,
                    amount: Value::Const(1),
                    random: false,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 34: Quandrix cards ────────────────────────────────────────────────

/// Quandrix Wavecharger — {2}{G}{U}, 3/3 Fractal Wizard.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter on
/// each Fractal you control."
pub fn quandrix_wavecharger() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Wavecharger",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::ForEach {
                selector: Selector::EachPermanent(
                    SelectionRequirement::HasCreatureType(CreatureType::Fractal)
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Swarm — {1}{G}{U}, Sorcery.
/// Synthesised Oracle: "Create a 2/2 green-and-blue Fractal creature token,
/// then draw a card."
pub fn fractal_swarm() -> CardDefinition {
    CardDefinition {
        name: "Fractal Swarm",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Sorcery],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::Seq(vec![
            Effect::CreateToken {
                who: PlayerRef::You,
                count: Value::Const(1),
                definition: quandrix_fractal_token(),
            },
            Effect::AddCounter {
                what: Selector::LastCreatedToken,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(2),
            },
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
        ]),
        activated_abilities: no_abilities(),
        triggered_abilities: vec![],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Proofwriter — {3}{G}{U}, 4/4 Fractal Wizard.
/// Synthesised Oracle: "When this creature enters, scry 2."
pub fn quandrix_proofwriter() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Proofwriter",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::Scry {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Solver — {2}{U}, 2/2 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Whenever you cast or copy an instant or
/// sorcery spell, draw a card, then discard a card."
pub fn quandrix_solver() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Solver",
        cost: cost(&[generic(2), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Seq(vec![
            Effect::Draw {
                who: Selector::You,
                amount: Value::Const(1),
            },
            Effect::Discard {
                who: Selector::You,
                amount: Value::Const(1),
                random: false,
            },
        ]))],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 35: Quandrix cards ────────────────────────────────────────────────

/// Quandrix Geomancer — {1}{G}{U}, 2/3 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, put a +1/+1 counter
/// on it. Magecraft — put a +1/+1 counter on this creature."
pub fn quandrix_b35_geomancer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Geomancer II",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![
            TriggeredAbility {
                event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
                effect: Effect::AddCounter {
                    what: Selector::This,
                    kind: CounterType::PlusOnePlusOne,
                    amount: Value::Const(1),
                },
            },
            magecraft(Effect::AddCounter {
                what: Selector::This,
                kind: CounterType::PlusOnePlusOne,
                amount: Value::Const(1),
            }),
        ],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Grower — {2}{G}{U}, 2/2 Fractal Druid.
/// Synthesised Oracle: "When this creature enters, create a 1/1 G/U
/// Fractal creature token."
pub fn fractal_grower() -> CardDefinition {
    use crate::catalog::sets::sos::fractal_token;
    CardDefinition {
        name: "Fractal Grower",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Druid],
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
                Effect::CreateToken {
                    who: PlayerRef::You,
                    count: Value::Const(1),
                    definition: fractal_token(),
                },
                Effect::AddCounter {
                    what: Selector::LastCreatedTokens,
                    kind: CounterType::PlusOnePlusOne,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Tideseer — {1}{U}, 1/2 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Scry 1."
pub fn quandrix_tideseer() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Tideseer",
        cost: cost(&[generic(1), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Merfolk, CreatureType::Wizard],
            ..Default::default()
        },
        power: 1,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Tidecaller — {3}{G}{U}, 3/3 Fractal Wizard with Flying.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn fractal_tidecaller() -> CardDefinition {
    CardDefinition {
        name: "Fractal Tidecaller",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 3,
        toughness: 3,
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
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 37: more Quandrix cards ───────────────────────────────────────────

/// Quandrix Researcher — {1}{G}{U}, 2/2 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card and lose 1
/// life."
pub fn quandrix_researcher() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Researcher",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
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
                Effect::Draw {
                    who: Selector::You,
                    amount: Value::Const(1),
                },
                Effect::LoseLife {
                    who: Selector::You,
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Scout — {G}, 1/1 Elf Scout.
/// Synthesised Oracle: "Magecraft — put a +1/+1 counter on this creature."
pub fn quandrix_scout() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scout",
        cost: cost(&[g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Scout],
            ..Default::default()
        },
        power: 1,
        toughness: 1,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Reefborn — {4}{G}{U}, 4/4 Fractal Wizard with Trample.
/// Synthesised Oracle: "When this creature enters, double all +1/+1
/// counters on target creature you control."
pub fn fractal_reefborn() -> CardDefinition {
    CardDefinition {
        name: "Fractal Reefborn",
        cost: cost(&[generic(4), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal, CreatureType::Wizard],
            ..Default::default()
        },
        power: 4,
        toughness: 4,
        keywords: vec![Keyword::Trample],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: Effect::AddCounter {
                what: target_filtered(
                    SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
                ),
                kind: CounterType::PlusOnePlusOne,
                amount: Value::CountersOn {
                    what: Box::new(Selector::Target(0)),
                    kind: CounterType::PlusOnePlusOne,
                },
            },
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Equation — {2}{G}, Instant.
/// Synthesised Oracle: "Put two +1/+1 counters on target creature you
/// control."
pub fn quandrix_b35_equation() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Equation II",
        cost: cost(&[generic(2), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Instant],
        subtypes: Subtypes::default(),
        power: 0,
        toughness: 0,
        keywords: vec![],
        effect: Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature.and(SelectionRequirement::ControlledByYou),
            ),
            kind: CounterType::PlusOnePlusOne,
            amount: Value::Const(2),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

// ── Batch 38: more Quandrix cards ───────────────────────────────────────────

/// Quandrix Pondkeeper (variant II) — {G}{U}, 1/3 Frog Druid.
/// Synthesised Oracle: "Magecraft — Scry 1."
pub fn quandrix_pondkeeper_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Pondkeeper II",
        cost: cost(&[g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 1,
        toughness: 3,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::Scry {
            who: PlayerRef::You,
            amount: Value::Const(1),
        })],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Fractal Emergent — {1}{G}{U}, 0/0 Fractal that enters with three +1/+1
/// counters on it (CR 614.12 replacement). Synthesised Oracle.
pub fn fractal_emergent() -> CardDefinition {
    CardDefinition {
        name: "Fractal Emergent",
        cost: cost(&[generic(1), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Fractal],
            ..Default::default()
        },
        power: 0,
        toughness: 0,
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
        enters_with_counters: Some((CounterType::PlusOnePlusOne, Value::Const(3))),
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Fluctuator — {2}{G}{U}, 2/3 Elf Wizard.
/// Synthesised Oracle: "When this creature enters, draw a card."
pub fn quandrix_fluctuator() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Fluctuator",
        cost: cost(&[generic(2), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Wizard],
            ..Default::default()
        },
        power: 2,
        toughness: 3,
        keywords: vec![],
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
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Doublecaster (variant II) — {2}{U}, 1/3 Merfolk Wizard.
/// Synthesised Oracle: "Magecraft — Put a +1/+1 counter on target Fractal
/// you control."
pub fn quandrix_doublecaster_v2() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Doublecaster II",
        cost: cost(&[generic(2), u()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
            what: target_filtered(
                SelectionRequirement::Creature
                    .and(SelectionRequirement::ControlledByYou)
                    .and(SelectionRequirement::HasCreatureType(CreatureType::Fractal)),
            ),
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Scaler — {1}{G}, 2/2 Elf Druid.
/// Synthesised Oracle: "Magecraft — Put a +1/+1 counter on this creature."
pub fn quandrix_scaler() -> CardDefinition {
    CardDefinition {
        name: "Quandrix Scaler",
        cost: cost(&[generic(1), g()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Elf, CreatureType::Druid],
            ..Default::default()
        },
        power: 2,
        toughness: 2,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![magecraft(Effect::AddCounter {
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
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Basinkeeper — {3}{G}{U}, 3/4 Frog Druid.
/// Synthesised Oracle: "When this creature enters, create a 0/0 G/U
/// Fractal creature token, then put two +1/+1 counters on it."
pub fn quandrix_basinkeeper() -> CardDefinition {
    use crate::effect::shortcut::create_token_with_counter;
    CardDefinition {
        name: "Quandrix Basinkeeper",
        cost: cost(&[generic(3), g(), u()]),
        supertypes: vec![],
        card_types: vec![CardType::Creature],
        subtypes: Subtypes {
            creature_types: vec![CreatureType::Frog, CreatureType::Druid],
            ..Default::default()
        },
        power: 3,
        toughness: 4,
        keywords: vec![],
        effect: Effect::Noop,
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(EventKind::EntersBattlefield, EventScope::SelfSource),
            effect: create_token_with_counter(
                PlayerRef::You,
                1,
                quandrix_fractal_token(),
                CounterType::PlusOnePlusOne,
                2,
            ),
        }],
        static_abilities: vec![],
        base_loyalty: 0,
        loyalty_abilities: vec![],
        alternative_cost: None,
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}

/// Quandrix Counterbearer — {1}{G}, 1/2 Elf Druid.
/// Synthesised Oracle: "Whenever a +1/+1 counter is placed on another
/// creature you control, this creature gets +1/+1 until end of turn."
pub fn quandrix_counterbearer() -> CardDefinition {
    use crate::card::Predicate;
    CardDefinition {
        name: "Quandrix Counterbearer",
        cost: cost(&[generic(1), g()]),
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
        activated_abilities: no_abilities(),
        triggered_abilities: vec![TriggeredAbility {
            event: EventSpec::new(
                EventKind::CounterAdded(CounterType::PlusOnePlusOne),
                EventScope::YourControl,
            )
            .with_filter(Predicate::EntityMatches {
                what: Selector::TriggerSource,
                filter: SelectionRequirement::Creature
                    .and(SelectionRequirement::OtherThanSource),
            }),
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
        back_face: None,
        opening_hand: None,
        enters_with_counters: None,
        exile_on_resolve: false,
        affinity_filter: None,
    }
}
